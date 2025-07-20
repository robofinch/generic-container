use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, PoisonError, TryLockError as StdTryLockError},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{locked_mutexes, mutex_id};
use crate::mutex_id::MutexID;
use crate::error::{AccessResult, LockError, LockResult, TryLockError, TryLockResult};


/// A variant of [`std::sync::Mutex`] which gracefully returns an error when a thread attempts
/// to acquire a `ThreadCheckedMutex` that it already holds.
///
/// In such a situation, [`Mutex::lock`] is guaranteed to either lock or panic, while
/// [`Mutex::try_lock`] checks if *any* thread holds the lock (and cannot distinguish whether the
/// current thread holds the lock). As such, attempting to lock the same `Mutex` twice on a thread
/// is potentially a fatal error; `ThreadCheckedMutex` allows for recovery.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ThreadCheckedMutex<T: ?Sized> {
    mutex_id: MutexID,
    mutex:    Mutex<T>,
}

impl<T> ThreadCheckedMutex<T> {
    /// Creates a new mutex in an unlocked state.
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
    /// Helper function for creating a [`ThreadCheckedMutexGuard`] from a [`MutexGuard`].
    #[inline]
    const fn new_guard<'a>(&self, guard: MutexGuard<'a, T>) -> ThreadCheckedMutexGuard<'a, T> {
        ThreadCheckedMutexGuard {
            mutex_id: self.mutex_id,
            guard,
        }
    }

    /// Helper function for mapping the type inside a [`PoisonError`] from [`MutexGuard`] to
    /// [`ThreadCheckedMutexGuard`].
    #[inline]
    fn poisoned_guard<'a>(
        &self,
        poison: PoisonError<MutexGuard<'a, T>>,
    ) -> PoisonError<ThreadCheckedMutexGuard<'a, T>> {
        PoisonError::new(self.new_guard(poison.into_inner()))
    }
}

impl<T: ?Sized> ThreadCheckedMutex<T> {
    /// Attempts to acquire this mutex, blocking the current thread while the mutex is locked in
    /// other threads.
    ///
    /// If the mutex is acquired (either completely successfully or with a poison error), a
    /// [`ThreadCheckedMutexGuard`] is returned. Only one thread at a time can hold the lock; at
    /// most one [`ThreadCheckedMutexGuard`] can exist at a time (across any thread); and the mutex
    /// is unlocked when the returned guard is dropped.
    ///
    /// # Errors
    /// If the mutex was already held by the current thread when this call was made, then a
    /// [`LockedByCurrentThread`] error is returned.
    ///
    /// If another user of this mutex panicked while holding the mutex, then this call will still
    /// acquire the mutex but wrap the returned guard in a poison error. See the
    /// [`HandlePoisonResult`] trait for methods to ignore poison errors and treat them as
    /// successful, or to panic if a poison error was returned.
    ///
    /// [`HandlePoisonResult`]: crate::HandlePoisonResult
    /// [`LockedByCurrentThread`]: LockError::LockedByCurrentThread
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

    /// Attempts to acquire this mutex without blocking.
    ///
    /// If the mutex is acquired (either completely successfully or with a poison error), a
    /// [`ThreadCheckedMutexGuard`] is returned. Only one thread at a time can hold the lock; at
    /// most one [`ThreadCheckedMutexGuard`] can exist at a time (across any thread); and the mutex
    /// is unlocked when the returned guard is dropped.
    ///
    /// # Errors
    /// If the mutex was already held by the current thread when this call was made, then a
    /// [`LockedByCurrentThread`] error is returned. If the mutex was held by a different thread,
    /// then a [`WouldBlock`] error is returned.
    ///
    /// If another user of this mutex panicked while holding the mutex, then this call will still
    /// acquire the mutex but wrap the returned guard in a poison error. See the
    /// [`HandlePoisonResult`] trait for methods to ignore poison errors and treat them as
    /// successful, or to panic if a poison error was returned.
    ///
    /// [`HandlePoisonResult`]: crate::HandlePoisonResult
    /// [`LockedByCurrentThread`]: TryLockError::LockedByCurrentThread
    /// [`WouldBlock`]: TryLockError::WouldBlock
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

    /// Determines whether this mutex is currently held by the current thread.
    #[inline]
    #[must_use]
    pub fn locked_by_current_thread(&self) -> bool {
        locked_mutexes::locked_by_current_thread(self.mutex_id)
    }

    /// Determines whether this mutex is currently poisoned.
    ///
    /// If another thread is active, the mutex could become poisoned or have its poison cleared
    /// at any time; as such, the return value of this function should generally not be depended on
    /// for program correctness.
    ///
    /// [Read more about poison.](crate::HandlePoisonResult#about-poison)
    #[inline]
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.mutex.is_poisoned()
    }

    /// Clear any poison from this mutex.
    ///
    /// When a [`ThreadCheckedMutexGuard`] is dropped in a thread which is panicking, its associated
    /// mutex becomes poisoned, and remains poisoned until this function is called (by any thread).
    ///
    /// [Read more about poison.](crate::HandlePoisonResult#about-poison)
    #[inline]
    pub fn clear_poison(&self) {
        self.mutex.clear_poison();
    }

    /// Consumes this mutex and returns the underlying data.
    ///
    /// # Errors
    /// If another user of this mutex panicked while holding the mutex, then the inner data is
    /// still returned, but wrapped in a poison error.
    ///
    /// [Read more about poison.](crate::HandlePoisonResult#about-poison)
    #[inline]
    pub fn into_inner(self) -> AccessResult<T>
    where
        T: Sized,
    {
        self.mutex.into_inner().map_err(Into::into)
    }

    /// Returns a mutable reference to the underlying data, without locking.
    ///
    /// # Errors
    /// If another user of this mutex panicked while holding the mutex, then a mutable reference is
    /// still returned, but wrapped in a poison error.
    ///
    /// [Read more about poison.](crate::HandlePoisonResult#about-poison)
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

/// A RAII scoped lock for a [`ThreadCheckedMutex`], analogous to [`MutexGuard`] for [`Mutex`].
///
/// When this guard is dropped, the corresponding [`ThreadCheckedMutex`] is unlocked. The guard
/// provides access to the mutex's protected data via [`Deref`] and [`DerefMut`].
///
/// This structure can be created via the [`lock`] and [`try_lock`] methods of
/// [`ThreadCheckedMutex`].
///
/// [`lock`]: ThreadCheckedMutex::lock
/// [`try_lock`]: ThreadCheckedMutex::try_lock
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


#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "these are tests")]

    use std::{sync::mpsc, thread};
    use std::{sync::Arc, time::Duration};

    use crate::mutex_id::run_this_before_each_test_that_creates_a_mutex_id;
    use super::*;


    #[test]
    fn lock_then_is_locked() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = ThreadCheckedMutex::new(0_u8);

        assert!(!mutex.locked_by_current_thread());

        let _guard = mutex.lock().unwrap();

        assert!(mutex.locked_by_current_thread());
    }

    #[test]
    fn lock_unlock_isnt_locked() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = ThreadCheckedMutex::new(0_u8);

        let guard = mutex.lock().unwrap();

        assert!(mutex.locked_by_current_thread());

        drop(guard);

        assert!(!mutex.locked_by_current_thread());
    }

    #[test]
    fn lock_unlock_lock() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = ThreadCheckedMutex::new(0_u8);

        {
            let _guard = mutex.lock().unwrap();
        }

        assert!(!mutex.locked_by_current_thread());

        let _guard = mutex.lock().unwrap();

        assert!(mutex.locked_by_current_thread());
    }

    #[test]
    fn lock_lock_unlock_lock() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = ThreadCheckedMutex::new(0_u8);

        let guard = mutex.lock().unwrap();

        // An additional attempt to lock should fail.
        assert!(matches!(
            mutex.lock(),
            Err(LockError::LockedByCurrentThread),
        ));

        drop(guard);

        // Now it should succeed.
        let _guard = mutex.lock().unwrap();
    }

    #[test]
    fn locked_by_current_thread() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = Arc::new(ThreadCheckedMutex::new(()));
        let (sender, receiver) = mpsc::channel();

        let mutex_clone = Arc::clone(&mutex);

        thread::spawn(move || {
            let guard = mutex_clone.try_lock().unwrap();
            drop(guard);
            sender.send(()).unwrap();
        });

        // Wait to receive something.
        receiver.recv().unwrap();

        // The mutex should have been unlocked before we received anything.
        let _guard = mutex.try_lock().unwrap();

        // An additional attempt to lock should fail.
        assert!(matches!(
            mutex.try_lock(),
            Err(TryLockError::LockedByCurrentThread),
        ));
    }

    #[test]
    fn would_block() {
        run_this_before_each_test_that_creates_a_mutex_id();

        let mutex = Arc::new(ThreadCheckedMutex::new(()));
        let (locking_sender, locking_receiver) = mpsc::channel();
        let (unlocking_sender, unlocking_receiver) = mpsc::channel();

        let mutex_clone = Arc::clone(&mutex);

        thread::spawn(move || {
            let guard = mutex_clone.try_lock().unwrap();

            locking_sender.send(()).unwrap();

            // Wait to receive something.
            unlocking_receiver.recv().unwrap();

            // Block for a bit, to try to ensure that `lock` is capable of waiting.
            thread::sleep(Duration::from_millis(50));

            drop(guard);
        });

        // Wait to receive something.
        locking_receiver.recv().unwrap();

        // The mutex should have been locked before we received anything, and since we haven't
        // sent anything, it should still be locked.

        assert!(matches!(
            mutex.try_lock(),
            Err(TryLockError::WouldBlock),
        ));

        unlocking_sender.send(()).unwrap();

        // Now `lock` should work, though `try_lock` might not.
        let _guard = mutex.lock().unwrap();
    }
}
