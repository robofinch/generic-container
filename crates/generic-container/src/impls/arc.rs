use std::{convert::Infallible, sync::Arc};

use crate::container_traits::{Container, FragileContainer, FragileTryContainer, TryContainer};


impl<T> FragileTryContainer<T> for Arc<T> {
    type Ref<'a>  = &'a T where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self {
        Self::new(t)
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self)
    }
}

impl<T> TryContainer<T> for Arc<T> {}

impl<T> FragileContainer<T> for Arc<T> {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self
    }
}

impl<T> Container<T> for Arc<T> {}
