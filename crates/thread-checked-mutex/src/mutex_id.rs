#![expect(
    clippy::redundant_pub_crate,
    reason = "reemphasize that these are all internals",
)]

use std::num::NonZeroU64;
#[cfg(target_has_atomic = "64")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_has_atomic = "64"))]
use std::sync::Mutex;


pub(crate) type MutexID = NonZeroU64;

/// 2^63, which is basically half of `u64::MAX`.
const MAX_MUTEXES_PER_PROCESS: u64 = 1 << 63;


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
    #[expect(
        clippy::unwrap_used,
        reason = "panics cannot occur here, only above",
    )]
    NonZeroU64::new(counter + 1).unwrap()
}

#[cfg(target_has_atomic = "64")]
#[inline]
fn next_counter() -> u64 {
    static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

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

