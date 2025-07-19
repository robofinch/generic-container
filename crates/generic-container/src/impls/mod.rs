// Most of the implementations here are trivial wrappers around the type's existing functionality,
// and thus should be inlined.
#![warn(clippy::missing_inline_in_public_items)]

mod t_itself;
mod box_container;
mod rc;
mod arc;
mod rc_refcell;
mod arc_rwlock;
mod arc_mutex;

mod checked_rc_refcell;
#[cfg(feature = "thread-checked-lock")]
mod arc_checked_mutex;


pub use self::checked_rc_refcell::CheckedRcRefCell;
#[cfg(feature = "thread-checked-lock")]
pub use self::arc_checked_mutex::ErasedLockError;


use std::sync::PoisonError;

/// Trait for handling `Result<T, PoisonError<T>>`, as bug-free code should never allow a poison
/// error to occur anyway. In most cases, we can panic if a poison error is encountered, but
/// in a few circumstances, we ignore the poison.
trait HandlePoisonedResult<T> {
    /// Panic if a poison error is received, as bug-free code should never allow a poison error
    /// to occur anyway.
    #[must_use]
    fn panic_if_poisoned(self) -> T;

    /// Return the `T` in the provided `Result<T, PoisonError<T>>` result, regardless of whether
    /// it's wrapped in a poison error.
    #[must_use]
    fn ignore_poisoned(self) -> T;
}

impl<T> HandlePoisonedResult<T> for Result<T, PoisonError<T>> {
    #[inline]
    fn panic_if_poisoned(self) -> T {
        #[expect(
            clippy::unwrap_used,
            reason = "if a panic occurred, there's a bug in whatever code led to that panic",
        )]
        self.unwrap()
    }

    #[inline]
    fn ignore_poisoned(self) -> T {
        match self {
            Ok(t)       => t,
            Err(poison) => poison.into_inner(),
        }
    }
}
