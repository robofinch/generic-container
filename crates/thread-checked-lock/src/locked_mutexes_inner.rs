#![expect(
    clippy::redundant_pub_crate,
    reason = "reemphasize that these are all internals",
)]

use std::collections::HashSet;

use crate::mutex_id::MutexID;


#[derive(Debug)]
pub(crate) struct LockedMutexesInner<const INLINE: usize> {
    inline_ids: [Option<MutexID>; INLINE],
    id_set:     HashSet<MutexID>,
}

impl<const INLINE: usize> LockedMutexesInner<INLINE> {
    #[inline]
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            inline_ids: [None; INLINE],
            id_set:     HashSet::new(),
        }
    }

    /// Returns `true` iff `mutex_id` was not previously locked. In either case, `mutex_id` is
    /// registered as locked when this function returns.
    ///
    /// Equivalently, returns `true` iff internal state semantically changed.
    pub(crate) fn register_locked(&mut self, mutex_id: MutexID) -> bool {
        if self.inline_ids.contains(&Some(mutex_id)) {
            return false;
        }

        // We know we can insert the unique ID, then.
        for id in &mut self.inline_ids {
            if id.is_none() {
                // Not sure how expensive this check is, but figured
                // I'd only do it when necessary.
                #[expect(clippy::redundant_else, reason = "clarity")]
                if self.id_set.contains(&mutex_id) {
                    return false;
                } else {
                    *id = Some(mutex_id);
                    return true;
                }
            }
        }

        // If we get here, then `inline_ids` is full.
        // `insert` returns `true` if the mutex_id isn't in there.
        self.id_set.insert(mutex_id)
    }

    /// Returns `true` iff `mutex_id` was locked. In either case, `mutex_id` is not registered
    /// as locked when this function returns.
    ///
    /// Equivalently, returns `true` iff internal state semantically changed.
    pub(crate) fn register_unlocked(&mut self, mutex_id: MutexID) -> bool {
        for id in &mut self.inline_ids {
            if *id == Some(mutex_id) {
                *id = None;
                return true;
            }
        }

        self.id_set.remove(&mutex_id)
    }

    /// Returns `true` iff `mutex_id` was locked.
    #[inline]
    pub(crate) fn locked_by_current_thread(&self, mutex_id: MutexID) -> bool {
        self.inline_ids.contains(&Some(mutex_id))
            || self.id_set.contains(&mutex_id)
    }
}

impl<const INLINE: usize> Default for LockedMutexesInner<INLINE> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
