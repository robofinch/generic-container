use std::{convert::Infallible, error::Error, sync::PoisonError};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};


pub trait HandlePoisonResult<R, S> {
    #[must_use]
    fn ignore_poison(self) -> S;
    fn panic_if_poison(self) -> S;
}

pub type LockResult<T> = Result<T, LockError<T>>;
pub type PoisonlessLockResult<T> = Result<T, LockError<Infallible>>;

impl<T> HandlePoisonResult<Self, PoisonlessLockResult<T>> for LockResult<T> {
    #[inline]
    fn ignore_poison(self) -> PoisonlessLockResult<T> {
        match self.map_err(LockError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// # Panics
    /// Panics if the result is an error that was caused by poison
    /// (another thread panicking while they held a mutex related
    /// to this result).
    #[inline]
    fn panic_if_poison(self) -> PoisonlessLockResult<T> {
        self.map_err(LockError::panic_if_poison)
    }
}

pub enum LockError<T> {
    Poisoned(PoisonError<T>),
    LockedByCurrentThread,
}

impl<T> LockError<T> {
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessLockResult<T> {
        match self {
            Self::Poisoned(poison)      => Ok(poison.into_inner()),
            Self::LockedByCurrentThread => Err(LockError::LockedByCurrentThread),
        }
    }

    /// # Panics
    /// Panics if the error was caused by poison (another thread
    /// panicking while they held a mutex related to this error).
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
            Self::Poisoned(poison) => {
                f.debug_tuple("Poisoned")
                    .field(&poison)
                    .finish()
            }
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

pub type TryLockResult<T> = Result<T, TryLockError<T>>;
pub type PoisonlessTryLockResult<T> = Result<T, TryLockError<Infallible>>;

impl<T> HandlePoisonResult<Self, PoisonlessTryLockResult<T>> for TryLockResult<T> {
    #[inline]
    fn ignore_poison(self) -> PoisonlessTryLockResult<T> {
        match self.map_err(TryLockError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// # Panics
    /// Panics if the result is an error that was caused by poison
    /// (another thread panicking while they held a mutex related
    /// to this result).
    #[inline]
    fn panic_if_poison(self) -> PoisonlessTryLockResult<T> {
        self.map_err(TryLockError::panic_if_poison)
    }
}

pub enum TryLockError<T> {
    Poisoned(PoisonError<T>),
    LockedByCurrentThread,
    WouldBlock,
}

impl<T> TryLockError<T> {
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessTryLockResult<T> {
        match self {
            Self::Poisoned(poison)      => Ok(poison.into_inner()),
            Self::LockedByCurrentThread => Err(TryLockError::LockedByCurrentThread),
            Self::WouldBlock            => Err(TryLockError::WouldBlock),
        }
    }

    /// # Panics
    /// Panics if the error was caused by poison (another thread
    /// panicking while they held a mutex related to this error).
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
            Self::Poisoned(poison) => {
                f.debug_tuple("Poisoned")
                    .field(&poison)
                    .finish()
            }
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

pub type AccessResult<T> = Result<T, AccessError<T>>;
pub type PoisonlessAccessResult<T> = Result<T, AccessError<Infallible>>;

impl<T> HandlePoisonResult<Self, PoisonlessAccessResult<T>> for AccessResult<T> {
    #[inline]
    fn ignore_poison(self) -> PoisonlessAccessResult<T> {
        match self.map_err(AccessError::ignore_poison) {
            Ok(t)                  => Ok(t),
            Err(poisonless_result) => poisonless_result,
        }
    }

    /// # Panics
    /// Panics if the result is an error variant that was caused by
    /// poison; note that `AccessError`s are all caused by poison.
    #[inline]
    fn panic_if_poison(self) -> PoisonlessAccessResult<T> {
        self.map_err(|err| AccessError::panic_if_poison(err))
    }
}

pub struct AccessError<T> {
    pub poison: PoisonError<T>,
}

impl<T> AccessError<T> {
    #[inline]
    pub fn ignore_poison(self) -> PoisonlessAccessResult<T> {
        Ok(self.poison.into_inner())
    }

    /// # Panics
    /// As an `AccessError` is always caused by a poison,
    /// this function always panics.
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

