use core::convert::Infallible;

use crate::container_traits::{
    Container, FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
    MutContainer, TryContainer, TryMutContainer,
};


impl<T: ?Sized> FragileTryContainer<T> for T {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where Self: Sized {
        t
    }

    /// Infallibly get the `T`.
    #[inline]
    fn into_inner(self) -> Option<T> where Self: Sized {
        Some(self)
    }

    /// Infallibly get immutable access to the `T`.
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T: ?Sized> TryContainer<T> for T {}

impl<T: ?Sized> FragileContainer<T> for T {
    /// Infallibly get immutable access to the `T`.
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T: ?Sized> Container<T> for T {}

impl<T: ?Sized> FragileTryMutContainer<T> for T {
    type RefMut<'a>  = &'a mut T where T: 'a;
    type RefMutError = Infallible;

    /// Infallibly get mutable access to the `T`.
    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self)
    }
}

impl<T: ?Sized> TryMutContainer<T> for T {}

impl<T: ?Sized> FragileMutContainer<T> for T {
    /// Infallibly get mutable access to the `T`.
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self
    }
}

impl<T: ?Sized> MutContainer<T> for T {}
