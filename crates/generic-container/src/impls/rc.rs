use core::convert::Infallible;
use alloc::rc::Rc;

use crate::container_traits::{Container, FragileContainer, FragileTryContainer, TryContainer};


impl<T: ?Sized> FragileTryContainer<T> for Rc<T> {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(t)
    }

    /// Attempt to retrieve the inner `T` from the container.
    ///
    /// Uses [`Rc::into_inner`].
    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self)
    }

    /// Infallibly get immutable access to the inner `T`.
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T: ?Sized> TryContainer<T> for Rc<T> {}

impl<T: ?Sized> FragileContainer<T> for Rc<T> {
    /// Infallibly get immutable access to the inner `T`.
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T: ?Sized> Container<T> for Rc<T> {}
