use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::container_traits::FragileTryContainer;


// Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, and Hash are manually implemented.
#[derive(Default)]
pub struct Contained<T: ?Sized, C: ?Sized + FragileTryContainer<T>> {
    pub _marker:   PhantomData<T>,
    pub container: C,
}

impl<T: ?Sized, C: FragileTryContainer<T>> Contained<T, C> {
    #[inline]
    #[must_use]
    pub const fn new(container: C) -> Self {
        Self {
            _marker: PhantomData,
            container,
        }
    }
}

impl<T, C> Debug for Contained<T, C>
where
    T:             ?Sized,
    C:             ?Sized + FragileTryContainer<T>,
    for<'a> &'a C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Contained")
            .field("_marker", &self._marker)
            .field("container", &&self.container)
            .finish()
    }
}

impl<T: ?Sized, C: FragileTryContainer<T> + Copy> Copy for Contained<T, C> {}

impl<T: ?Sized, C: FragileTryContainer<T> + Clone> Clone for Contained<T, C> {
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

// impl<T, Rhs, K> PartialEq<Rhs> for Contained<T, C>
// where
//     T: ?Sized + PartialEq<Rhs>,
//     Rhs: ?Sized,
//     C: In,
// {
//     fn eq(&self, other: &Rhs) -> bool {
//         self.container.get_ref().eq(other)
//     }
// }

// impl<T: ?Sized, C: Container<T> + Eq> Eq for Contained<T, C> {}

