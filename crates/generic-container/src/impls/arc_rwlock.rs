use std::convert::Infallible;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::container_traits::{
    FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
};
use super::HandlePoisonedResult as _;


impl<T: ?Sized> FragileTryContainer<T> for Arc<RwLock<T>> {
    type Ref<'a>  = RwLockReadGuard<'a, T> where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(RwLock::new(t))
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self)
            .map(RwLock::into_inner)
            .map(Result::ignore_poisoned)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self.read().panic_if_poisoned())
    }
}

impl<T: ?Sized> FragileContainer<T> for Arc<RwLock<T>> {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.read().panic_if_poisoned()
    }
}

impl<T: ?Sized> FragileTryMutContainer<T> for Arc<RwLock<T>> {
    type RefMut<'a>  = RwLockWriteGuard<'a, T> where T: 'a;
    type RefMutError = Infallible;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self.write().panic_if_poisoned())
    }
}

impl<T: ?Sized> FragileMutContainer<T> for Arc<RwLock<T>> {
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self.write().panic_if_poisoned()
    }
}
