#![expect(
    unsafe_code,
    reason = "Using RefCell instead of UnsafeCell would be overkill",
)]
#![expect(
    clippy::redundant_pub_crate,
    reason = "reemphasize that these are all internals",
)]

use std::cell::UnsafeCell;

use crate::{locked_mutexes_inner::LockedMutexesInner, mutex_id::MutexID};


// 4 u64's seems like a reasonable number to keep inlined
type Inner = LockedMutexesInner<4>;

thread_local! {
    /// Follow the below requirement when using this static.
    /// # Safety Requirement
    /// This TLS must only be accessed by `access_locked_mutexes`.
    /// Unsafe code may rely on this requirement.
    static LOCKED_MUTEXES: UnsafeCell<Inner> = UnsafeCell::default();
}


/// # Safety
/// - The given callback `f` must not call `access_locked_mutexes`.
/// - The return value of `f` must not borrow/reference the
///   `&mut Inner` provided to `f` in any way.
unsafe fn access_locked_mutexes<F, R>(f: F) -> R
where
    F: FnOnce(&mut Inner) -> R,
{
    // SAFETY REQUIREMENT:
    // We are in the function permitted to access `LOCKED_MUTEXES`.
    LOCKED_MUTEXES.with(|inner| {
        let locked_mutexes: *mut Inner = inner.get();

        // SAFETY:
        // The `UnsafeCell` ensures that the pointer it returns is
        // properly aligned, and at least initially points to a
        // valid value of type `Inner`. We never write to the
        // pointee with unsafe code (excluding whatever `std` does),
        // so that remains true.
        // The most important requirement we need to check is
        // aliasing. The caller promises that the passed callback
        // does not call `access_locked_mutexes`, and does not
        // return anything that borrows/references the provided
        // `&mut Inner` reference. Therefore, we are not being
        // called inside `access_locked_mutexes` or called while
        // some other reference to the pointee of `locked_mutexes`
        // is live. That is, aliasing is satisfied at the start of
        // this supposed-to-be-unique borrow. Additionally, this
        // borrow ends at the end of this function (and cannot
        // somehow be extended by the return value). Since `f` does
        // not call `access_locked_mutexes`, which is the only way
        // to get another pointer to `locked_mutexes`’ pointee, the
        // reference remains unique in the remainder of this
        // function. Thus, aliasing is satisfied.
        let locked_mutexes: &mut Inner = unsafe {
            &mut *locked_mutexes
        };

        f(locked_mutexes)
    })
}

#[must_use]
pub(crate) fn register_locked(mutex_id: MutexID) -> bool {
    // SAFETY:
    // - The callback does not call `access_locked_mutexes`, as the
    //   `locked_mutexes_inner` module does not import anything
    //   from this module.
    // - The return value, `bool`, does not reference anything.
    unsafe {
        access_locked_mutexes(|lm_inner| {
            lm_inner.register_locked(mutex_id)
        })
    }
}

#[must_use]
pub(crate) fn register_unlocked(mutex_id: MutexID) -> bool {
    // SAFETY:
    // - The callback does not call `access_locked_mutexes`, as the
    //   `locked_mutexes_inner` module does not import anything
    //   from this module.
    // - The return value, `bool`, does not reference anything.
    unsafe {
        access_locked_mutexes(|lm_inner| {
            lm_inner.register_unlocked(mutex_id)
        })
    }
}

#[inline]
#[must_use]
pub(crate) fn locked_by_current_thread(mutex_id: MutexID) -> bool {
    // SAFETY:
    // - The callback does not call `access_locked_mutexes`, as the
    //   `locked_mutexes_inner` module does not import anything
    //   from this module.
    // - The return value, `bool`, does not reference anything.
    unsafe {
        access_locked_mutexes(|lm_inner| {
            lm_inner.locked_by_current_thread(mutex_id)
        })
    }
}

