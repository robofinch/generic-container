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

// TODO: all the above modules need documentation. They also all need container kinds.


use std::sync::PoisonError;

/// Trait for unwrapping `Result<T, PoisonError<T>>`, as bug-free code should never allow a poison
/// error to occur anyway.
trait UnwrapPoisonResult<T> {
    /// Panic if a poison error is received, as bug-free code should never allow a poison error
    /// to occur anyway.
    #[must_use]
    fn panic_if_poisoned(self) -> T;
}

impl<T> UnwrapPoisonResult<T> for Result<T, PoisonError<T>> {
    #[inline]
    fn panic_if_poisoned(self) -> T {
        #[expect(
            clippy::unwrap_used,
            reason = "if a panic occurred, there's a bug in whatever code led to that panic",
        )]
        self.unwrap()
    }
}
