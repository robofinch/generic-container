use std::fmt::Debug;
use std::ops::{Deref, DerefMut};


// TODO: unify documentation about reentrancy, and standardize the wording of reference and borrow.
// I think "borrow" might be better.

// ================================================================
//  The four `{Fragile|}{Try|}Container` types
// ================================================================

/// An abstraction over some container which owns a `T` and can provide immutable references to it,
/// or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// Common examples include `Box<T>`, `Rc<T>`, `Rc<RefCell<T>>`, `Arc<T>`, `Arc<RwLock<T>>`,
/// `Arc<Mutex<T>>`, and a `T` itself.
///
/// TODO: this next sentence might need to be updated at some point.
///
/// Additionally, [`CheckedRcRefCell<T>`],
/// [`CheckedArcRwLock<T>`], and [`CheckedArcMutex<T>`] are provided (which perform additional
/// checks that should be unnecessary in bug-free code).
///
/// # Fragility: Potential Panics or Deadlocks
///
/// A single thread should not attempt to get multiple live references to the `T` in a
/// `FragileTryContainer`, whether from the same container struct or clones referencing the same
/// inner `T`. Doing so risks a panic or deadlock (such as in the case of `Arc<Mutex<T>>`), unless
/// this `FragileTryContainer` is also a [`TryContainer`]. In other words, `FragileTryContainer`
/// does not guarantee that the container handles [reentrancy] gracefully. A thread should drop any
/// existing borrow before a new borrow may be obtained from the fragile container.
///
/// # Errors
///
/// The [`into_inner`] and [`try_get_ref`] methods may be able to fail, depending on the container;
/// a container should clearly document the circumstances in which a `None` or `Err` variant may
/// be returned.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`CheckedRcRefCell<T>`]: TODO::TODO::TODO
/// [`CheckedArcRwLock<T>`]: TODO::TODO::TODO
/// [`CheckedArcMutex<T>`]: TODO::TODO::TODO
/// [reentrancy]: https://en.wikipedia.org/wiki/Reentrancy_(computing)
pub trait FragileTryContainer<T: ?Sized> {
    type Ref<'a>:  Deref<Target = T> where Self: 'a;
    type RefError: Debug;

    /// Create a new container that owns the provided `T`.
    #[must_use]
    fn new_container(t: T) -> Self;

    /// Attempt to retrieve the inner `T` from the container.
    ///
    /// If the container allows for multiple handles to the same `T` (as with `Rc` or `Arc`),
    /// and this method is called on each of those handles, then `Some(T)` should be returned
    /// for exactly one container.
    #[must_use]
    fn into_inner(self) -> Option<T> where T: Sized;

    /// Immutably borrow the wrapped `T`.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// Depending on the container implementation, the borrow may have a nontrivial `Drop` impl
    /// that interferes with other attempts to borrow from this container or other clones of
    /// the container.
    ///
    /// Unless this [`FragileTryContainer`] is also a [`TryContainer`], implementations are
    /// permitted to panic or deadlock if a single thread fails to drop previous references and
    /// attempts to obtain multiple live references.
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError>;
}

/// An abstraction over some container which owns a `T` and can infallibly provide immutable
/// references to it, or attempt to be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// Common examples include `Box<T>`, `Rc<T>`, `Rc<RefCell<T>>`, `Arc<T>`, `Arc<RwLock<T>>`,
/// `Arc<Mutex<T>>`, and a `T` itself.
///
/// Note that the implementations for `Arc<RwLock<T>>` and `Arc<Mutex<T>>` will panic on poison
/// errors, as other threads should not panic unless a bug was encountered somewhere. Similarly,
/// the implementation for `Rc<RefCell<T>>` may panic if borrowing from the `RefCell` would violate
/// aliasing rules; however, if the below rules about handling a [`FragileTryContainer`] or
/// `FragileContainer` are followed, the implementation will not panic.
///
/// TODO: this next sentence might need to be updated at some point.
///
/// [`CheckedRcRefCell<T>`], [`CheckedArcRwLock<T>`], and [`CheckedArcMutex<T>`] return errors
/// instead of panicking when such bugs are encountered (and are thus not infallible).
///
/// # Fragility: Potential Panics or Deadlocks
///
/// A single thread should not attempt to get multiple live references to the `T` in a
/// `FragileContainer`, whether from the same container struct or clones referencing the same
/// inner `T`. Doing so risks a panic or deadlock (such as in the case of `Arc<Mutex<T>>`), unless
/// this `FragileContainer` is also a [`Container`]. In other words, `FragileContainer` does not
/// guarantee that the container handles [reentrancy] gracefully. A thread should drop any existing
/// borrow before a new borrow may be obtained from the fragile container.
///
/// # `None` values
///
/// Note that `into_inner` is still permitted to return `None`, even though `get_ref` and
/// `try_get_ref` do not fail. A container should clearly document when `into_inner` returns `None`.
///
/// [`CheckedRcRefCell<T>`]: TODO::TODO::TODO
/// [`CheckedArcRwLock<T>`]: TODO::TODO::TODO
/// [`CheckedArcMutex<T>`]: TODO::TODO::TODO
/// [reentrancy]: https://en.wikipedia.org/wiki/Reentrancy_(computing)
pub trait FragileContainer<T: ?Sized>: FragileTryContainer<T> {
    /// Immutably borrow the wrapped `T`.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// A single thread should not attempt to get multiple live references to the `T` in a
    /// `FragileContainer`, whether from the same container struct or clones referencing the same
    /// inner `T`. Doing so risks a panic or deadlock (such as in the case of `Arc<Mutex<T>>`),
    /// unless this `FragileContainer` is also a [`Container`]. In other words, `FragileContainer`
    /// does not guarantee that the container handles [reentrancy] gracefully. A thread must drop
    /// any existing borrow before a new borrow may be obtained from the fragile container.
    #[must_use]
    fn get_ref(&self) -> Self::Ref<'_>;
}

/// An abstraction over some container which owns a `T` and can provide immutable references to it,
/// or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// Common examples include `Box<T>`, `Rc<T>`, `Arc<T>`, and a `T` itself.
///
/// Notably, `Rc<RefCell<T>>` and `Arc<Mutex<T>>` only implement [`FragileContainer`] and not
/// [`Container`] or [`TryContainer`], as attempting to get multiple mutable borrows in a single
/// thread from the same `RefCell` or `Mutex` would cause a panic or deadlock. A `TryContainer`
/// implementation must not panic or deadlock due to existing live borrows in the same thread.
///
/// TODO: note alternatives which are fine. Also, mention `Arc<RwLock<T>>`.
///
/// # Errors
///
/// The [`into_inner`] and [`try_get_ref`] methods may be able to fail, depending on the container;
/// a container should clearly document the circumstances in which a `None` or `Err` variant may
/// be returned.
///
/// [`try_borrow`]: std::cell::RefCell::try_borrow
/// [`into_inner`]: FragileContainer::into_inner
/// [`try_get_ref`]: FragileContainer::try_get_ref
pub trait TryContainer<T: ?Sized>: FragileTryContainer<T> {}

/// An abstraction over some container which owns a `T` and can infallibly provide immutable
/// references to it, or attempt to be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// Common examples include `Box<T>`, `Rc<T>`, `Arc<T>`, and a `T` itself.
///
/// TODO: mention more container types.
///
/// Note that `into_inner` is still permitted to return `None`, even though `get_ref` and
/// `try_get_ref` do not fail. A container should clearly document when `into_inner` returns `None`.
pub trait Container<T: ?Sized>: FragileContainer<T> + TryContainer<T> {}

// ================================================================
//  The four `{Fragile|}{Try|}MutContainer` types
// ================================================================

// TODO: documentation

pub trait FragileTryMutContainer<T: ?Sized>: FragileTryContainer<T> {
    type RefMut<'a>:  DerefMut<Target = T> where Self: 'a;
    type RefMutError: Debug;

    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError>;
}

pub trait FragileMutContainer<T: ?Sized>: FragileTryMutContainer<T> + FragileContainer<T> {
    #[must_use]
    fn get_mut(&mut self) -> Self::RefMut<'_>;
}

pub trait TryMutContainer<T: ?Sized>: FragileTryMutContainer<T> + TryContainer<T> {}

pub trait MutContainer<T: ?Sized>: FragileMutContainer<T> + TryMutContainer<T> + Container<T> {}
