<div align="center" class="rustdoc-hidden">
<h1> Thread-checked Lock </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-thread--checked--lock-08f?logo=github" height="20">](https://github.com/robofinch/generic-container/tree/main/crates/thread-checked-lock)
[![Latest version](https://img.shields.io/crates/v/thread-checked-lock.svg)](https://crates.io/crates/thread-checked-lock)
[![Documentation](https://img.shields.io/docsrs/thread-checked-lock)](https://docs.rs/thread-checked-lock)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

This crate slightly strengthens the information returned by a mutex's `try_lock` method, and
ensures that calling `lock` in a thread which has already acquired the mutex is not a fatal
error.

The provided [`ThreadCheckedMutex`] struct gracefully errors (instead of panicking or
deadlocking) when a thread attempts to acquire a [`ThreadCheckedMutex`] that it already holds.

## Motivation

The standard [`Mutex`] does provide [`Mutex::try_lock`], which doesn't panic or deadlock,
but it cannot distinguish between the current thread holding the lock and a different thread
holding the lock; it only indicates that attempting to acquire the lock would not immediately
succeed.

Comparing [`Mutex::try_lock`] with [`RefCell::try_borrow`], the return value of `try_borrow` is
less ambiguous, because there's only one way for it to fail: the current thread must have
mutably borrowed the `RefCell`. When implementing the [`generic-container`] crate's traits
for various types, this felt like a gap; the standard mutex type can easily be implemented as
a "fragile" container (which places greater responsibilities on the caller), but not even a
spinlock approach with `try_lock` could make it non-fragile. [`ThreadCheckedMutex`] fills out a
niche in the container traits that did not seem to be met by an existing crate.

## Example

```rust
use thread_checked_lock::{ThreadCheckedMutex, LockError};

let mutex = ThreadCheckedMutex::new(0_u8);

let guard = mutex.lock().expect("Locking a new mutex succeeds");

// An additional attempt to lock should fail.
assert!(matches!(
    mutex.lock(),
    Err(LockError::LockedByCurrentThread),
));

drop(guard);

// Now it should succeed. The mutex is unlocked, and not poisoned.
let _guard = mutex.lock().unwrap();
```

## Features
- `serde`: derives `Serialize` and `Deserialize` for `ThreadCheckedMutex`.

## Minimum supported Rust Version (MSRV)
Rust 1.85, the earliest version of the 2024 edition, is supported.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE])
 * MIT license ([LICENSE-MIT])

at your option.

[LICENSE-APACHE]: ../../LICENSE-APACHE
[LICENSE-MIT]: ../../LICENSE-MIT

[`generic-container`]: https://crates.io/crates/generic-container

[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
[`Mutex::try_lock`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.try_lock
[`RefCell::try_borrow`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.try_borrow

[`ThreadCheckedMutex`]: https://docs.rs/thread-checked-lock/0/thread_checked_lock/struct.ThreadCheckedMutex.html
