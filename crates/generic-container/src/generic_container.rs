#![warn(
    clippy::missing_inline_in_public_items,
    reason = "the wrapper type should mostly just delegate",
)]

use core::{cmp::Ordering, marker::PhantomData};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
};


// Default, Debug, Copy, Clone, PartialEq<Self>, Eq, PartialOrd<Self>, Ord, and Hash are all
// manually implemented and defer to the container.
/// A wrapper type intended for use in blanket implementations of `YourTrait` ranging over
/// containers `C` that hold a `T: YourTrait`.
///
/// This is necessary to avoid conflicting trait implementations.
///
/// ## Examples
/// Not needed when the `T: YourTrait` is fixed:
/// ```
/// use generic_container::Container;
///
/// trait Trait {
///     fn do_thing(&self);
/// }
///
/// impl<C: ?Sized + Container<dyn Trait>> Trait for C {
///     fn do_thing(&self) {
///         self.get_ref().do_thing();
///     }
/// }
/// ```
///
/// But when the inner type varies:
/// ```
/// use generic_container::{Container, GenericContainer};
///
/// trait Trait {
///     fn do_thing(&self);
/// }
///
/// impl<T: ?Sized + Trait, C: ?Sized + Container<T>> Trait for GenericContainer<T, C> {
///     fn do_thing(&self) {
///         self.container.get_ref().do_thing();
///     }
/// }
/// ```
///
/// Without a wrapper type, it does not compile:
/// ```compile_fail
/// use generic_container::{Container, GenericContainer};
///
/// trait Trait {
///     fn do_thing(&self);
/// }
///
/// impl<T: ?Sized + Trait, C: ?Sized + Container<T>> Trait for C {
///     fn do_thing(&self) {
///         self.get_ref().do_thing();
///     }
/// }
/// ```
#[repr(transparent)]
pub struct GenericContainer<T: ?Sized, C: ?Sized> {
    /// Distinguish which type is supposed to be contained.
    pub _marker:   PhantomData<T>,
    /// Should implement the base container trait, [`FragileTryContainer<T>`].
    ///
    /// [`FragileTryContainer<T>`]: crate::container_traits::FragileTryContainer
    pub container: C,
}

impl<T: ?Sized, C> GenericContainer<T, C> {
    /// Create a new `GenericContainer` struct wrapping the provided value, treated as a container
    /// around a specific type.
    #[inline]
    #[must_use]
    pub const fn new(container: C) -> Self {
        Self {
            _marker: PhantomData,
            container,
        }
    }
}

impl<T: ?Sized, C: Default> Default for GenericContainer<T, C> {
    #[inline]
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<T, C> Debug for GenericContainer<T, C>
where
    T: ?Sized,
    C: ?Sized + Debug,
{
    #[allow(clippy::missing_inline_in_public_items, reason = "not trivial or likely to be hot")]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("GenericContainer")
            .field("_marker", &self._marker)
            .field("container", &&self.container)
            .finish()
    }
}

impl<T: ?Sized, C: Copy> Copy for GenericContainer<T, C> {}

impl<T: ?Sized, C: Clone> Clone for GenericContainer<T, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _marker:   self._marker,
            container: self.container.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.container.clone_from(&source.container);
    }
}

impl<T: ?Sized, C: ?Sized + PartialEq<C>> PartialEq<Self> for GenericContainer<T, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.container.eq(&other.container)
    }
}

impl<T: ?Sized, C: ?Sized + Eq> Eq for GenericContainer<T, C> {}

impl<T: ?Sized, C: ?Sized + PartialOrd<C>> PartialOrd<Self> for GenericContainer<T, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.container.partial_cmp(&other.container)
    }
}

impl<T: ?Sized, C: ?Sized + Ord> Ord for GenericContainer<T, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.container.cmp(&other.container)
    }
}

impl<T: ?Sized, C: ?Sized + Hash> Hash for GenericContainer<T, C> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.container.hash(state);
    }
}
