use core::ops::{Deref, DerefMut};


// ================================================================
//  The four `{Fragile|}{Try|}Container` traits
// ================================================================

/// An abstraction over some container which owns a `T` and can provide immutable references to it,
/// or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// This is the base container trait, which places the fewest requirements on implementors.
///
/// # Fragility: Potential Panics or Deadlocks
///
/// Unless a `FragileTryContainer` is known to also implement [`TryContainer`], it should be treated
/// as [fragile].
///
/// # Errors
///
/// The [`into_inner`] and [`try_get_ref`] methods may be able to fail, depending on the container;
/// a container should clearly document the circumstances in which a `None` or `Err` variant may
/// be returned.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [fragile]: crate#fragility-potential-panics-or-deadlocks
pub trait FragileTryContainer<T: ?Sized> {
    /// An immutably borrowed value from the container.
    ///
    /// May have a nontrivial `Drop` implementatation, as with the [`Ref`] type corresponding
    /// to [`RefCell`].
    ///
    /// [`Ref`]: std::cell::Ref
    /// [`RefCell`]: std::cell::RefCell
    type Ref<'a>:  Deref<Target = T> where Self: 'a;
    /// An error that might be returned by [`try_get_ref`]. This type should implement
    /// [`std::error::Error`].
    ///
    /// The canonical error to use when [`try_get_ref`] can never return an error
    /// is [`Infallible`].
    ///
    /// [`try_get_ref`]: FragileTryContainer::try_get_ref
    /// [`Infallible`]: std::convert::Infallible
    type RefError;

    /// Create a new container that owns the provided `T`.
    #[must_use]
    fn new_container(t: T) -> Self where Self: Sized, T: Sized;

    /// Attempt to retrieve the inner `T` from the container.
    ///
    /// ### Note for implementors
    ///
    /// Given a collection of containers that refer to the same inner `T` (as with several cloned
    /// `Rc` or `Arc` containers, or the trivial case of a single container like `Box<T>`), if
    /// `into_inner` is called on each of those containers, then an implementation should return
    /// `Some(T)` for exactly one of them, unless there is some useful reason for the implementation
    /// to do otherwise.
    #[must_use]
    fn into_inner(self) -> Option<T> where Self: Sized, T: Sized;

    /// Attempt to immutably access the inner `T`.
    ///
    /// There are no particular constraints imposed on implementations. In particular, depending on
    /// the container implementation:
    /// - the function could be infallible,
    /// - the function could panic or deadlock (see below),
    /// - retrying the function in a loop might never succeed.
    ///
    /// However, if the container implements [`FragileContainer<T>`], then implementors should
    /// usually make `try_get_ref` infallible as well, unless there is some useful reason to not
    /// do so.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// Unless this [`FragileTryContainer`] is also a [`TryContainer`], implementations are
    /// permitted to panic or deadlock if this method is called from a thread which already has a
    /// reference to the inner `T` of this container.
    ///
    /// [Read more about fragility.](crate#fragility-potential-panics-or-deadlocks)
    ///
    /// # Errors
    ///
    /// Errors are implementation-defined, and should be documented by implementors.
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError>;
}

/// An abstraction over some container which owns a `T` and can infallibly provide immutable
/// references to it, or attempt to be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// # Fragility: Potential Panics or Deadlocks
///
/// This container should be assumed to be [fragile], unless it is known to implement
/// [`Container<T>`].
///
/// # `None` values
///
/// Note that [`into_inner`] is still permitted to return `None`, even though [`get_ref`] does not
/// fail, and most implementors should make [`try_get_ref`] infallible as well. A container should
/// clearly document when [`into_inner`] returns `None`.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`get_ref`]: FragileContainer::get_ref
/// [fragile]: crate#fragility-potential-panics-or-deadlocks
pub trait FragileContainer<T: ?Sized>: FragileTryContainer<T> {
    /// Immutably borrow the inner `T`.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// Unless this [`FragileContainer`] is also a [`Container`], implementations are
    /// permitted to panic or deadlock if this method is called from a thread which already has a
    /// reference to the inner `T` of this container.
    ///
    /// [Read more about fragility.](crate#fragility-potential-panics-or-deadlocks)
    #[must_use]
    fn get_ref(&self) -> Self::Ref<'_>;
}

/// An abstraction over some container which owns a `T` and can provide immutable references to it,
/// or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// # Errors
///
/// The [`into_inner`] and [`try_get_ref`] methods may be able to fail, depending on the container;
/// a container should clearly document the circumstances in which a `None` or `Err` variant may
/// be returned.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
pub trait TryContainer<T: ?Sized>: FragileTryContainer<T> {}

/// An abstraction over some container which owns a `T` and can infallibly provide immutable
/// references to it, or attempt to be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// # `None` values
///
/// Note that [`into_inner`] is still permitted to return `None`, even though [`get_ref`] does not
/// fail, and most implementors should make [`try_get_ref`] infallible as well. A container should
/// clearly document when [`into_inner`] returns `None`.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`get_ref`]: FragileContainer::get_ref
pub trait Container<T: ?Sized>: FragileContainer<T> + TryContainer<T> {}

// ================================================================
//  The four `{Fragile|}{Try|}MutContainer` traits
// ================================================================

/// An abstraction over some container which owns a `T` and can provide mutable or immutable
/// references to it, or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// This is the base mutable container trait, which places the fewest requirements on container
/// implementations that can provide mutable access to the inner `T`.
///
/// # Fragility: Potential Panics or Deadlocks
///
/// Unless a `FragileTryMutContainer` is known to also implement [`TryMutContainer`], it should be
/// treated as [fragile].
///
/// # Errors
///
/// The [`into_inner`], [`try_get_ref`], and [`try_get_mut`] methods may be able to fail, depending
/// on the container; a container should clearly document the circumstances in which a `None` or
/// `Err` variant may be returned.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`try_get_mut`]: FragileTryMutContainer::try_get_mut
/// [fragile]: crate#fragility-potential-panics-or-deadlocks
pub trait FragileTryMutContainer<T: ?Sized>: FragileTryContainer<T> {
    /// A mutably borrowed value from the container.
    ///
    /// May have a nontrivial `Drop` implementatation, as with the [`RefMut`] type corresponding
    /// to [`RefCell`].
    ///
    /// [`RefMut`]: std::cell::RefMut
    /// [`RefCell`]: std::cell::RefCell
    type RefMut<'a>:  DerefMut<Target = T> where Self: 'a;
    /// An error that might be returned by [`try_get_mut`]. This type should implement
    /// [`std::error::Error`].
    ///
    /// The canonical error to use when [`try_get_mut`] can never return an error
    /// is [`Infallible`].
    ///
    /// [`try_get_mut`]: FragileTryMutContainer::try_get_mut
    /// [`Infallible`]: std::convert::Infallible
    type RefMutError;

    /// Attempt to mutably access the inner `T`.
    ///
    /// There are no particular constraints imposed on implementations. In particular, depending on
    /// the container implementation:
    /// - the function could be infallible,
    /// - the function could panic or deadlock (see below),
    /// - retrying the function in a loop might never succeed.
    ///
    /// However, if the container implements [`FragileMutContainer<T>`], then implementors should
    /// usually make `try_get_mut` infallible as well, unless there is some useful reason to not
    /// do so.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// Unless this [`FragileTryMutContainer`] is also a [`TryMutContainer`], implementations are
    /// permitted to panic or deadlock if this method is called from a thread which already has a
    /// reference to the inner `T` of this container.
    ///
    /// [Read more about fragility.](crate#fragility-potential-panics-or-deadlocks)
    ///
    /// # Errors
    /// Errors are implementation-defined, and should be documented by implementors.
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError>;
}

/// An abstraction over some container which owns a `T` and can infallibly provide mutable or
/// immutable references to it, or attempt to be consumed to return the inner `T`
/// (if `T` is [`Sized`]).
///
/// # Fragility: Potential Panics or Deadlocks
///
/// This container should be assumed to be [fragile], unless it is known to implement
/// [`MutContainer<T>`].
///
/// # `None` values
///
/// Note that [`into_inner`] is still permitted to return `None`, even though [`get_ref`] and
/// [`get_mut`] do not fail, and most implementors should make [`try_get_ref`] and [`try_get_mut`]
/// infallible as well. A container should clearly document when [`into_inner`] returns `None`.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`get_ref`]: FragileContainer::get_ref
/// [`try_get_mut`]: FragileTryMutContainer::try_get_mut
/// [`get_mut`]: FragileMutContainer::get_mut
/// [fragile]: crate#fragility-potential-panics-or-deadlocks
pub trait FragileMutContainer<T: ?Sized>: FragileTryMutContainer<T> + FragileContainer<T> {
    /// Mutably borrow the inner `T`.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    ///
    /// Unless this [`FragileMutContainer`] is also a [`MutContainer`], implementations are
    /// permitted to panic or deadlock if this method is called from a thread which already has a
    /// reference to the inner `T` of this container.
    ///
    /// [Read more about fragility.](crate#fragility-potential-panics-or-deadlocks)
    #[must_use]
    fn get_mut(&mut self) -> Self::RefMut<'_>;
}

/// An abstraction over some container which owns a `T` and can provide mutable or immutable
/// references to it, or be consumed to return the inner `T` (if `T` is [`Sized`]).
///
/// # Errors
///
/// The [`into_inner`], [`try_get_ref`], and [`try_get_mut`] methods may be able to fail, depending
/// on the container; a container should clearly document the circumstances in which a `None` or
/// `Err` variant may be returned.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`try_get_mut`]: FragileTryMutContainer::try_get_mut
pub trait TryMutContainer<T: ?Sized>: FragileTryMutContainer<T> + TryContainer<T> {}

/// An abstraction over some container which owns a `T` and can infallibly provide mutable or
/// immutable references to it, or attempt to be consumed to return the inner `T`
/// (if `T` is [`Sized`]).
///
/// # `None` values
///
/// Note that [`into_inner`] is still permitted to return `None`, even though [`get_ref`] and
/// [`get_mut`] do not fail, and most implementors should make [`try_get_ref`] and [`try_get_mut`]
/// infallible as well. A container should clearly document when [`into_inner`] returns `None`.
///
/// [`into_inner`]: FragileTryContainer::into_inner
/// [`try_get_ref`]: FragileTryContainer::try_get_ref
/// [`get_ref`]: FragileContainer::get_ref
/// [`try_get_mut`]: FragileTryMutContainer::try_get_mut
/// [`get_mut`]: FragileMutContainer::get_mut
pub trait MutContainer<T: ?Sized>: FragileMutContainer<T> + TryMutContainer<T> + Container<T> {}

// ================================================================
//  The two `*Base*Container` traits intended as aliases
// ================================================================

/// The trait for containers which places the fewest constraints on implementations.
///
/// A user-made trait alias for [`FragileTryContainer`].
pub trait BaseContainer<T: ?Sized>: FragileTryContainer<T> {}

impl<T: ?Sized, C: ?Sized + FragileTryContainer<T>> BaseContainer<T> for C {}

/// The trait for mutable containers which places the fewest constraints on implementations.
///
/// A user-made trait alias for [`FragileTryMutContainer`].
pub trait BaseMutContainer<T: ?Sized>: FragileTryMutContainer<T> {}

impl<T: ?Sized, C: ?Sized + FragileTryMutContainer<T>> BaseMutContainer<T> for C {}
