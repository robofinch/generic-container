use std::convert::Infallible;

use crate::container_traits::{
    Container, FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
    MutContainer, TryContainer, TryMutContainer,
};


impl<T> FragileTryContainer<T> for Box<T> {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self {
        Self::new(t)
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Some(*self)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T> TryContainer<T> for Box<T> {}

impl<T> FragileContainer<T> for Box<T> {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T> Container<T> for Box<T> {}

impl<T> FragileTryMutContainer<T> for Box<T> {
    type RefMut<'a>  = &'a mut T where T: 'a;
    type RefMutError = Infallible;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self)
    }
}

impl<T> TryMutContainer<T> for Box<T> {}

impl<T> FragileMutContainer<T> for Box<T> {
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self
    }
}

impl<T> MutContainer<T> for Box<T> {}
