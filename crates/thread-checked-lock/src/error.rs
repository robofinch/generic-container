use std::{convert::Infallible, error::Error, sync::PoisonError};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};


/// Extension trait for [`Result`] which adds the ability to more conveniently handle the poison
/// errors returned by this crate's locks.
///
/// ## About Poison
///
/// When another thread panics while it holds a poisonable lock, the lock becomes poisoned (which
/// may be manually cleared). Attempts to acquire a poisoned lock (or otherwise access, via the
/// lock, the data protected by the lock) return poison errors, which act as speed bumps for
/// accessing data whose logical invariants are potentially broken. See `std`'s [`PoisonError`] for
/// more.
///
/// As a poison error is only returned when some thread has already panicked, it is common to
/// unconditionally panic in the current thread as well when poison is encountered, or to simply
/// ignore such a circumstance.
///
/// As a notable example, [`parking_lot`] does not provide poison errors at all, and does not care
/// whether a different thread panicked while holding a [`parking_lot`] mutex. This is roughly
/// equivalent to (but more performant than) using [`HandlePoisonResult::ignore_poison`]
/// everywhere.
///
///
/// [`parking_lot`]: https://docs.rs/parking_lot/
pub trait HandlePoisonResult {
    /// A variation of the `Self` result type which cannot possibly be a poison error.
    type PoisonlessResult;

    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the result unchanged.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[must_use]
    fn ignore_poison(self) -> Self::PoisonlessResult;

    /// Panics if the result was caused by poison, and otherwise returns the result unchanged.
    ///
    /// # Panics
    /// Panics if the result is an [`Err`] that was caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    fn panic_if_poison(self) -> Self::PoisonlessResult;
}

/// Helper function to coerce an uninhabited poison error into `!`.
#[inline]
fn prove_unreachable(poison: &PoisonError<Infallible>) -> ! {
    #[expect(clippy::uninhabited_references, reason = "this function is not reachable")]
    match *poison.get_ref() {}
}


/// The result type returned by [`ThreadCheckedMutex::lock`].
///
/// [`ThreadCheckedMutex::lock`]: super::mutex::ThreadCheckedMutex::lock
pub type LockResult<T> = Result<T, LockError<T>>;
/// A variation of [`LockResult<T>`] which cannot possibly be a poison error.
pub type PoisonlessLockResult<T> = Result<T, LockError<Infallible>>;

impl<T> HandlePoisonResult for LockResult<T> {
    type PoisonlessResult = PoisonlessLockResult<T>;

    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the result unchanged.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn ignore_poison(self) -> Self::PoisonlessResult {
        match self.map_err(LockError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// Panics if the result was caused by poison, and otherwise returns the result unchanged.
    ///
    /// # Panics
    /// Panics if the result is an [`Err`] that was caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn panic_if_poison(self) -> Self::PoisonlessResult {
        self.map_err(LockError::panic_if_poison)
    }
}

/// An error that may be returned by [`ThreadCheckedMutex::lock`].
///
/// [`ThreadCheckedMutex::lock`]: super::mutex::ThreadCheckedMutex::lock
pub enum LockError<T> {
    /// Returned when a lock was acquired, but the lock was poisoned.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    Poisoned(PoisonError<T>),
    /// Returned when a lock failed to be acquired because the thread attempting to acquire
    /// the lock was already holding the lock.
    LockedByCurrentThread,
}

impl<T> LockError<T> {
    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the error unchanged in an [`Err`].
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessLockResult<T> {
        match self {
            Self::Poisoned(poison)      => Ok(poison.into_inner()),
            Self::LockedByCurrentThread => Err(LockError::LockedByCurrentThread),
        }
    }

    /// Panics if the error was caused by poison, and otherwise returns the error unchanged.
    ///
    /// # Panics
    /// Panics if the error was caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    #[must_use]
    pub fn panic_if_poison(self) -> LockError<Infallible> {
        match self {
            #[expect(
                clippy::panic,
                reason = "library users will frequently want to panic on poison",
            )]
            Self::Poisoned(_)           => panic!("LockError was poison"),
            Self::LockedByCurrentThread => LockError::LockedByCurrentThread,
        }
    }
}

impl<T> From<PoisonError<T>> for LockError<T> {
    #[inline]
    fn from(poison: PoisonError<T>) -> Self {
        Self::Poisoned(poison)
    }
}

impl<T> Debug for LockError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Poisoned(poison)      => f.debug_tuple("Poisoned").field(&poison).finish(),
            Self::LockedByCurrentThread => f.write_str("LockedByCurrentThread"),
        }
    }
}

impl<T> Display for LockError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Poisoned(_) => write!(
                f,
                "LockError due to poison (another thread panicked)",
            ),
            Self::LockedByCurrentThread => write!(
                f,
                "Failed to acquire a lock, because the same thread was holding it",
            ),
        }
    }
}

impl<T> Error for LockError<T> {}

impl PartialEq for LockError<Infallible> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        // There's only one inhabited variant of `LockError<Infallible>`, so this returns true.
        match self {
            Self::LockedByCurrentThread => true,
            Self::Poisoned(poison)      => prove_unreachable(poison),
        }
    }
}

impl Eq for LockError<Infallible> {}


/// The result type returned by [`ThreadCheckedMutex::try_lock`].
///
/// [`ThreadCheckedMutex::try_lock`]: super::mutex::ThreadCheckedMutex::try_lock
pub type TryLockResult<T> = Result<T, TryLockError<T>>;
/// A variation of [`TryLockResult<T>`] which cannot possibly be a poison error.
pub type PoisonlessTryLockResult<T> = Result<T, TryLockError<Infallible>>;

impl<T> HandlePoisonResult for TryLockResult<T> {
    type PoisonlessResult = PoisonlessTryLockResult<T>;

    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the result unchanged.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn ignore_poison(self) -> Self::PoisonlessResult {
        match self.map_err(TryLockError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// Panics if the result was caused by poison, and otherwise returns the result unchanged.
    ///
    /// # Panics
    /// Panics if the result is an [`Err`] that was caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn panic_if_poison(self) -> Self::PoisonlessResult {
        self.map_err(TryLockError::panic_if_poison)
    }
}

/// An error that may be returned by [`ThreadCheckedMutex::try_lock`].
///
/// [`ThreadCheckedMutex::try_lock`]: super::mutex::ThreadCheckedMutex::try_lock
pub enum TryLockError<T> {
    /// Returned when a lock was acquired, but the lock was poisoned.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    Poisoned(PoisonError<T>),
    /// Returned when a lock failed to be acquired because the thread attempting to acquire
    /// the lock was already holding the lock.
    LockedByCurrentThread,
    /// Returned when a lock failed to be acquired because the lock was already held by a thread
    /// (other than the thread attempting to acquire the lock).
    WouldBlock,
}

impl<T> TryLockError<T> {
    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the error unchanged in an [`Err`].
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessTryLockResult<T> {
        match self {
            Self::Poisoned(poison)      => Ok(poison.into_inner()),
            Self::LockedByCurrentThread => Err(TryLockError::LockedByCurrentThread),
            Self::WouldBlock            => Err(TryLockError::WouldBlock),
        }
    }

    /// Panics if the error was caused by poison, and otherwise returns the error unchanged.
    ///
    /// # Panics
    /// Panics if the error was caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    #[must_use]
    pub fn panic_if_poison(self) -> TryLockError<Infallible> {
        match self {
            #[expect(
                clippy::panic,
                reason = "library users will frequently want to panic on poison",
            )]
            Self::Poisoned(_)           => panic!("TryLockError was poison"),
            Self::LockedByCurrentThread => TryLockError::LockedByCurrentThread,
            Self::WouldBlock            => TryLockError::WouldBlock,
        }
    }
}

impl<T> From<PoisonError<T>> for TryLockError<T> {
    #[inline]
    fn from(poison: PoisonError<T>) -> Self {
        Self::Poisoned(poison)
    }
}

impl<T> Debug for TryLockError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Poisoned(poison)      => f.debug_tuple("Poisoned").field(&poison).finish(),
            Self::LockedByCurrentThread => f.write_str("LockedByCurrentThread"),
            Self::WouldBlock            => f.write_str("WouldBlock"),
        }
    }
}

impl<T> Display for TryLockError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Poisoned(_) => write!(
                f,
                "TryLockError due to poison (another thread panicked)",
            ),
            Self::LockedByCurrentThread => write!(
                f,
                "Failed to acquire a lock, because the same thread was holding it",
            ),
            Self::WouldBlock => write!(
                f,
                "Lock was held by a different thread, so acquiring it would block",
            ),
        }
    }
}

impl<T> Error for TryLockError<T> {}

impl PartialEq for TryLockError<Infallible> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::LockedByCurrentThread => matches!(other, Self::LockedByCurrentThread),
            Self::WouldBlock            => matches!(other, Self::WouldBlock),
            Self::Poisoned(poison)      => prove_unreachable(poison),
        }
    }
}

impl Eq for TryLockError<Infallible> {}


/// The result type returned by [`ThreadCheckedMutex::into_inner`] or
/// [`ThreadCheckedMutex::get_mut`].
///
/// [`ThreadCheckedMutex::into_inner`]: super::mutex::ThreadCheckedMutex::into_inner
/// [`ThreadCheckedMutex::get_mut`]: super::mutex::ThreadCheckedMutex::get_mut
pub type AccessResult<T> = Result<T, AccessError<T>>;
/// A variation of [`AccessResult<T>`] which cannot possibly be a poison error.
///
/// Note that every [`AccessError`] is caused by poison, so this result is always [`Ok`].
pub type PoisonlessAccessResult<T> = Result<T, AccessError<Infallible>>;

impl<T> HandlePoisonResult for AccessResult<T> {
    type PoisonlessResult = PoisonlessAccessResult<T>;

    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]), and otherwise returns the result unchanged.
    ///
    /// Since every [`AccessError`] is caused by poison, the returned result is always [`Ok`].
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn ignore_poison(self) -> Self::PoisonlessResult {
        match self.map_err(AccessError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// Panics if the error was caused by poison, and otherwise returns the error unchanged.
    ///
    /// Note that every [`AccessError`] is caused by poison, so this is similar to unwrapping the
    /// result.
    ///
    /// # Panics
    /// Panics if the result is an [`Err`], which was necessarily caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    fn panic_if_poison(self) -> Self::PoisonlessResult {
        self.map_err(|err| AccessError::panic_if_poison(err))
    }
}

/// Returned when a lock's data was accessed, but the lock was poisoned.
///
/// [Read more about poison.](HandlePoisonResult#about-poison)
///
/// This error may be returned by [`ThreadCheckedMutex::into_inner`] or
/// [`ThreadCheckedMutex::get_mut`].
///
/// [`ThreadCheckedMutex::into_inner`]: super::mutex::ThreadCheckedMutex::into_inner
/// [`ThreadCheckedMutex::get_mut`]: super::mutex::ThreadCheckedMutex::get_mut
pub struct AccessError<T> {
    /// The only possible cause of an `AccessError` is a poisoned lock.
    pub poison: PoisonError<T>,
}

impl<T> AccessError<T> {
    /// Silently converts any poison error into a successful result (see
    /// [`PoisonError::into_inner`]).
    ///
    /// Since every [`AccessError`] is caused by poison, the returned result is always [`Ok`].
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessAccessResult<T> {
        Ok(self.poison.into_inner())
    }

    /// Panics if the [`AccessError`] was caused by poison, which is always the case; this function
    /// always panics.
    ///
    /// # Panics
    /// Panics unconditionally, as the error is necessarily caused by poison.
    ///
    /// [Read more about poison.](HandlePoisonResult#about-poison)
    #[inline]
    pub fn panic_if_poison(self) -> ! {
        #![expect(
            clippy::panic,
            reason = "library users will frequently want to panic on poison",
        )]
        panic!("AccessError is poison")
    }
}

impl<T> From<PoisonError<T>> for AccessError<T> {
    #[inline]
    fn from(poison: PoisonError<T>) -> Self {
        Self { poison }
    }
}

impl<T> Debug for AccessError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("AccessError")
            .field("poison", &self.poison)
            .finish()
    }
}

impl<T> Display for AccessError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "AccessError due to poison (another thread panicked)")
    }
}

impl<T> Error for AccessError<T> {}

impl PartialEq for AccessError<Infallible> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        prove_unreachable(&self.poison)
    }
}

impl Eq for AccessError<Infallible> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_ignore_poison() {
        // Ok
        let res_o: LockResult<()> = Ok(());
        assert!(matches!(res_o.ignore_poison(), Ok(())));

        // Err but not poison
        let res_e: LockResult<()> = Err(LockError::LockedByCurrentThread);
        assert!(matches!(res_e.ignore_poison(), Err(LockError::LockedByCurrentThread)));

        // Poison
        let res_p: LockResult<()> = Err(PoisonError::new(()).into());
        assert!(matches!(res_p.ignore_poison(), Ok(())));
    }

    #[test]
    fn lock_panic_if_poison() {
        // Ok
        let res_o: LockResult<()> = Ok(());
        assert!(matches!(res_o.panic_if_poison(), Ok(())));

        // Err but not poison
        let res_e: LockResult<()> = Err(LockError::LockedByCurrentThread);
        assert!(matches!(res_e.panic_if_poison(), Err(LockError::LockedByCurrentThread)));
    }

    #[test]
    #[should_panic = "LockError was poison"]
    fn panicking_lock_panic_if_poison() {
        // Poison
        let res_p: LockResult<()> = Err(PoisonError::new(()).into());
        #[expect(
            clippy::let_underscore_must_use,
            clippy::let_underscore_untyped,
            reason = "function never returns",
        )]
        let _ = res_p.panic_if_poison();
    }

    #[test]
    fn try_lock_ignore_poison() {
        // Ok
        let res_o: TryLockResult<()> = Ok(());
        assert!(matches!(res_o.ignore_poison(), Ok(())));

        // Err but not poison
        let res_e: TryLockResult<()> = Err(TryLockError::LockedByCurrentThread);
        assert!(matches!(res_e.ignore_poison(), Err(TryLockError::LockedByCurrentThread)));

        // Poison
        let res_p: TryLockResult<()> = Err(PoisonError::new(()).into());
        assert!(matches!(res_p.ignore_poison(), Ok(())));
    }

    #[test]
    fn try_lock_panic_if_poison() {
        // Ok
        let res_o: TryLockResult<()> = Ok(());
        assert!(matches!(res_o.panic_if_poison(), Ok(())));

        // Err but not poison
        let res_e: TryLockResult<()> = Err(TryLockError::LockedByCurrentThread);
        assert!(matches!(res_e.panic_if_poison(), Err(TryLockError::LockedByCurrentThread)));
    }

    #[test]
    #[should_panic = "TryLockError was poison"]
    fn panicking_try_lock_panic_if_poison() {
        // Poison
        let res_p: TryLockResult<()> = Err(PoisonError::new(()).into());
        #[expect(
            clippy::let_underscore_must_use,
            clippy::let_underscore_untyped,
            reason = "function never returns",
        )]
        let _ = res_p.panic_if_poison();
    }

    #[test]
    fn access_ignore_poison() {
        // Ok
        let res_o: AccessResult<()> = Ok(());
        assert!(matches!(res_o.ignore_poison(), Ok(())));

        // Err but not poison.. is impossible.

        // Poison
        let res_p: AccessResult<()> = Err(PoisonError::new(()).into());
        assert!(matches!(res_p.ignore_poison(), Ok(())));
    }

    #[test]
    fn access_panic_if_poison() {
        // Ok
        let res_o: AccessResult<()> = Ok(());
        assert!(matches!(res_o.panic_if_poison(), Ok(())));

        // Err but not poison.. is impossible.
    }

    #[test]
    #[should_panic = "AccessError is poison"]
    fn panicking_access_panic_if_poison() {
        // Poison
        let res_p: AccessResult<()> = Err(PoisonError::new(()).into());
        #[expect(
            clippy::let_underscore_must_use,
            clippy::let_underscore_untyped,
            reason = "function never returns",
        )]
        let _ = res_p.panic_if_poison();
    }

    fn test_eq_impl<E: Eq, const N: usize>(errors: &[E; N]) {
        for (i, error) in errors.iter().enumerate() {
            for (j, other) in errors.iter().enumerate() {
                assert_eq!(i == j, error == other);
            }
        }
    }

    #[test]
    fn eq_impls() {
        // The `::<Infallible>`s are not strictly necessary, but make it more clear.
        test_eq_impl(&[
            LockError::<Infallible>::LockedByCurrentThread,
        ]);
        test_eq_impl(&[
            TryLockError::<Infallible>::LockedByCurrentThread,
            TryLockError::<Infallible>::WouldBlock,
        ]);
        // `AccessError<Infallible>` is uninhabited.
    }
}
