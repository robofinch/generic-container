use std::sync::Arc;

use thread_checked_lock::{
    HandlePoisonResult as _, LockError, ThreadCheckedMutex, ThreadCheckedMutexGuard,
};

use crate::container_traits::{
    FragileTryContainer, FragileTryMutContainer, TryContainer, TryMutContainer,
};


/// A version of [`thread_checked_lock::LockError`] which does not allow a poison error to be
/// recovered into data.
#[derive(Debug, Clone, Copy)]
pub enum ErasedLockError {
    /// See [`LockError::Poisoned`]. However, the original poison error's data was already dropped.
    Poisoned,
    /// See [`LockError::LockedByCurrentThread`].
    LockedByCurrentThread,
}

impl ErasedLockError {
    /// Panics if the error was caused by poison, and otherwise returns the error unchanged.
    ///
    /// # Panics
    /// Panics if the error is the [`Poisoned`] variant.
    ///
    /// [`Poisoned`]: ErasedLockError::Poisoned
    #[inline]
    #[must_use]
    pub fn panic_if_poison(self) -> Self {
        match self {
            #[expect(
                clippy::panic,
                reason = "library users will frequently want to panic on poison",
            )]
            Self::Poisoned              => panic!("ErasedLockError was poison"),
            Self::LockedByCurrentThread => Self::LockedByCurrentThread,
        }
    }
}

impl<T> From<LockError<T>> for ErasedLockError {
    #[inline]
    fn from(value: LockError<T>) -> Self {
        match value {
            LockError::Poisoned(_)           => Self::Poisoned,
            LockError::LockedByCurrentThread => Self::LockedByCurrentThread,
        }
    }
}

impl<T: ?Sized> FragileTryContainer<T> for Arc<ThreadCheckedMutex<T>> {
    type Ref<'a>  = ThreadCheckedMutexGuard<'a, T> where T: 'a;
    type RefError = ErasedLockError;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(ThreadCheckedMutex::new(t))
    }

    /// Attempt to retrieve the inner `T` from the container.
    /// Behaves identically to [`Arc::into_inner`].
    ///
    /// Ignores any poison errors.
    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        let result = Self::into_inner(self)?
            .into_inner()
            .ignore_poison();

        // The result could only possibly be due to poison, so its `Err` is now uninhabited
        match result {
            Ok(t) => Some(t),
            #[expect(unreachable_code, reason = "yeah, that's the point")]
            Err(poisonless_poison) => match poisonless_poison.poison.into_inner() {},
        }
    }

    /// Attempt to immutably access the inner `T`.
    ///
    /// # Errors
    ///
    /// This function fails if and only if [`ThreadCheckedMutex::lock`] fails.
    ///
    /// A poison error is not ignored, nor does it trigger a panic.
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        self.lock().map_err(Into::into)
    }
}

impl<T: ?Sized> TryContainer<T> for Arc<ThreadCheckedMutex<T>> {}

impl<T: ?Sized> FragileTryMutContainer<T> for Arc<ThreadCheckedMutex<T>> {
    type RefMut<'a>  = ThreadCheckedMutexGuard<'a, T> where T: 'a;
    type RefMutError = ErasedLockError;

    /// Attempt to mutably access the inner `T`.
    ///
    /// # Errors
    ///
    /// This function fails if and only if [`ThreadCheckedMutex::lock`] fails.
    ///
    /// A poison error is not ignored, nor does it trigger a panic.
    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        self.lock().map_err(Into::into)
    }
}

impl<T: ?Sized> TryMutContainer<T> for Arc<ThreadCheckedMutex<T>> {}
