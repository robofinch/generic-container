use std::convert::Infallible;

use crate::container_traits::{
    Container, FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
    MutContainer, TryContainer, TryMutContainer,
};


impl<T> FragileTryContainer<T> for T {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self {
        t
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Some(self)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T> TryContainer<T> for T {}

impl<T> FragileContainer<T> for T {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T> Container<T> for T {}

impl<T> FragileTryMutContainer<T> for T {
    type RefMut<'a>  = &'a mut T where T: 'a;
    type RefMutError = Infallible;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self)
    }
}

impl<T> TryMutContainer<T> for T {}

impl<T> FragileMutContainer<T> for T {
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self
    }
}

impl<T> MutContainer<T> for T {}
