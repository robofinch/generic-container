#![warn(
    clippy::missing_inline_in_public_items,
    reason = "the wrapper type should mostly just delegate",
)]

use std::{cmp::Ordering, marker::PhantomData};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
};


// Default, Debug, Copy, Clone, PartialEq<Self>, Eq, PartialOrd<Self>, Ord, and Hash are all
// manually implemented and defer to the container.
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
    #[allow(clippy::missing_inline_in_public_items, reason = "nontrivial and unlikely to be hot")]
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
