// See https://linebender.org/blog/doc-include for this README inclusion strategy
//! [`Mutex`]: std::sync::Mutex
//! [`Mutex::try_lock`]: std::sync::Mutex::try_lock
//! [`RefCell::try_borrow`]: std::cell::RefCell::try_borrow
//!
//! [`ThreadCheckedMutex`]: ThreadCheckedMutex
//!
// File links are not supported by rustdoc
//! [LICENSE-APACHE]: https://github.com/robofinch/generic-container/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/generic-container/blob/main/LICENSE-MIT
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

mod mutex;
mod error;

mod locked_mutexes;
mod locked_mutexes_inner;
mod mutex_id;


pub use self::{
    error::{
        AccessError, AccessResult, HandlePoisonResult, LockError, LockResult,
        PoisonlessAccessResult, PoisonlessLockResult, PoisonlessTryLockResult,
        TryLockError, TryLockResult,
    },
    mutex::{ThreadCheckedMutex, ThreadCheckedMutexGuard},
};
