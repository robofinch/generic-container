use core::convert::Infallible;
use alloc::boxed::Box;

use crate::container_traits::{
    Container, FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
    MutContainer, TryContainer, TryMutContainer,
};


impl<T: ?Sized> FragileTryContainer<T> for Box<T> {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(t)
    }

    /// Infallibly get the inner `T` of this box.
    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Some(*self)
    }

    /// Infallibly get immutable access to the inner `T`.
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T: ?Sized> TryContainer<T> for Box<T> {}

impl<T: ?Sized> FragileContainer<T> for Box<T> {
    /// Infallibly get immutable access to the inner `T`.
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T: ?Sized> Container<T> for Box<T> {}

impl<T: ?Sized> FragileTryMutContainer<T> for Box<T> {
    type RefMut<'a>  = &'a mut T where T: 'a;
    type RefMutError = Infallible;

    /// Infallibly get mutable access to the inner `T`.
    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self)
    }
}

impl<T: ?Sized> TryMutContainer<T> for Box<T> {}

impl<T: ?Sized> FragileMutContainer<T> for Box<T> {
    /// Infallibly get mutable access to the inner `T`.
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self
    }
}

impl<T: ?Sized> MutContainer<T> for Box<T> {}
