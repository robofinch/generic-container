use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, PoisonError, TryLockError as StdTryLockError},
};

use crate::{locked_mutexes, mutex_id};
use crate::mutex_id::MutexID;
use crate::error::{AccessResult, LockError, LockResult, TryLockError, TryLockResult};


#[derive(Debug)]
pub struct ThreadCheckedMutex<T: ?Sized> {
    mutex_id: MutexID,
    mutex:    Mutex<T>,
}

impl<T> ThreadCheckedMutex<T> {
    #[inline]
    #[must_use]
    pub fn new(t: T) -> Self {
        Self {
            mutex_id: mutex_id::next_id(),
            mutex:    Mutex::new(t),
        }
    }
}

impl<T: ?Sized> ThreadCheckedMutex<T> {
    #[inline]
    const fn new_guard<'a>(&self, guard: MutexGuard<'a, T>) -> ThreadCheckedMutexGuard<'a, T> {
        ThreadCheckedMutexGuard {
            mutex_id: self.mutex_id,
            guard,
        }
    }

    #[inline]
    fn poisoned_guard<'a>(
        &self,
        poison: PoisonError<MutexGuard<'a, T>>,
    ) -> PoisonError<ThreadCheckedMutexGuard<'a, T>> {
        PoisonError::new(self.new_guard(poison.into_inner()))
    }
}

impl<T: ?Sized> ThreadCheckedMutex<T> {
    pub fn lock(&self) -> LockResult<ThreadCheckedMutexGuard<'_, T>> {
        if locked_mutexes::register_locked(self.mutex_id) {
            match self.mutex.lock() {
                Ok(guard)   => Ok(self.new_guard(guard)),
                Err(poison) => {
                    let poison = self.poisoned_guard(poison);
                    Err(LockError::Poisoned(poison))
                }
            }
        } else {
            Err(LockError::LockedByCurrentThread)
        }
    }

    pub fn try_lock(&self) -> TryLockResult<ThreadCheckedMutexGuard<'_, T>> {
        if self.locked_by_current_thread() {
            return Err(TryLockError::LockedByCurrentThread);
        }

        match self.mutex.try_lock() {
            Ok(guard) => {
                #[expect(
                    clippy::let_underscore_must_use,
                    clippy::redundant_type_annotations,
                    reason = "We already checked that the current thread hasn't locked the mutex, \
                              so this always returns true.",
                )]
                let _: bool = locked_mutexes::register_locked(self.mutex_id);
                Ok(self.new_guard(guard))
            }
            Err(StdTryLockError::Poisoned(poison)) => {
                #[expect(
                    clippy::let_underscore_must_use,
                    clippy::redundant_type_annotations,
                    reason = "We already checked that the current thread hasn't locked the mutex, \
                              so this always returns true.",
                )]
                let _: bool = locked_mutexes::register_locked(self.mutex_id);
                let poison = self.poisoned_guard(poison);
                Err(TryLockError::Poisoned(poison))
            }
            Err(StdTryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
        }
    }

    #[inline]
    #[must_use]
    pub fn locked_by_current_thread(&self) -> bool {
        locked_mutexes::locked_by_current_thread(self.mutex_id)
    }

    #[inline]
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.mutex.is_poisoned()
    }

    #[inline]
    pub fn clear_poison(&self) {
        self.mutex.clear_poison();
    }

    #[inline]
    pub fn into_inner(self) -> AccessResult<T>
    where
        T: Sized,
    {
        self.mutex.into_inner().map_err(Into::into)
    }

    #[inline]
    pub fn get_mut(&mut self) -> AccessResult<&mut T> {
        self.mutex.get_mut().map_err(Into::into)
    }
}

impl<T: Default> Default for ThreadCheckedMutex<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[must_use = "if unused the ThreadCheckedMutex will immediately unlock"]
#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct ThreadCheckedMutexGuard<'a, T: ?Sized> {
    mutex_id: MutexID,
    guard:    MutexGuard<'a, T>,
}

impl<T: ?Sized> Drop for ThreadCheckedMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let was_locked = locked_mutexes::register_unlocked(self.mutex_id);

        // This assertion should not fail unless someone used unsound unsafe code.
        debug_assert!(
            was_locked,
            "a ThreadCheckedMutexGuard was dropped in a thread which it was not locked in",
        );
    }
}

impl<T: ?Sized> Deref for ThreadCheckedMutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T: ?Sized> DerefMut for ThreadCheckedMutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<T: ?Sized + Display> Display for ThreadCheckedMutexGuard<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&*self.guard, f)
    }
}
