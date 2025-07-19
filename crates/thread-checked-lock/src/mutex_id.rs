#![expect(
    clippy::redundant_pub_crate,
    reason = "reemphasize that these are all internals",
)]

use std::num::NonZeroU64;
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_has_atomic = "64"))]
use std::sync::Mutex;


/// A unique `MutexId` should be assigned to each `ThreadCheckedMutex` so that each thread
/// can track which mutexes they have acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MutexID(NonZeroU64);

/// 2^63, which is basically half of [`u64::MAX`].
const MAX_MUTEXES_PER_PROCESS: u64 = 1 << 63;


/// Returns a unique `MutexID` that was not returned on any previous call in the program to this
/// function.
pub(crate) fn next_id() -> MutexID {
    let counter = next_counter();

    // There are `max` counter values in `0..max`.
    assert!(
        counter < MAX_MUTEXES_PER_PROCESS,
        "Only 2^63 thread-checked mutexes may be created in one process",
    );

    // We unwrap below, because a panic would only occur if
    // `counter + 1`, a sum of `u64`s, is zero. This only occurs if
    // `counter` is `u64::MAX`, which would trigger a panic above.
    #[expect(clippy::unwrap_used, reason = "panics cannot occur here, only above")]
    let id = NonZeroU64::new(counter + 1).unwrap();

    MutexID(id)
}

/// Sequentially return the next `u64`, starting at `0` when first called in the program.
///
/// Could theoretically wrap back to `0`.
#[cfg(target_has_atomic = "64")]
#[inline]
fn next_counter() -> u64 {
    static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Sequentially return the next `u64`, starting at `0` when first called in the program.
///
/// Could theoretically wrap back to `0`.
#[cfg(not(target_has_atomic = "64"))]
#[inline]
fn next_counter() -> u64 {
    static ID_COUNTER: Mutex<u64> = Mutex::new(0);

    // On no thread will `wrapping_add` or loads and stores panic.
    #[expect(
        clippy::unwrap_used,
        reason = "Mutex can only be poisoned if the following three lines can panic",
    )]
    let mut counter_guard = ID_COUNTER.lock().unwrap();
    let counter: u64 = *counter_guard;
    *counter_guard = counter.wrapping_add(1);
    counter
}


#[cfg(test)]
pub(crate) use self::tests::run_this_before_each_test_that_creates_a_mutex_id;

#[cfg(test)]
mod tests {
    use std::sync::Once;
    use super::*;


    const fn first_id() -> MutexID {
        #[expect(clippy::unwrap_used, reason = "1 is nonzero")]
        MutexID(NonZeroU64::new(1).unwrap())
    }

    pub(crate) fn run_this_before_each_test_that_creates_a_mutex_id() {
        // Add `inline(never)` just in case; it could make backtraces better if it were
        // to fail.
        #[inline(never)]
        fn test_first_mutex_id() {
            assert_eq!(next_id(), first_id());
        }

        static ONCE: Once = Once::new();

        ONCE.call_once(test_first_mutex_id);
    }

    #[test]
    fn check_start_and_uniqueness() {
        run_this_before_each_test_that_creates_a_mutex_id();

        assert_ne!(next_id(), first_id());
    }
}
