use std::convert::Infallible;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::container_traits::{
    FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
};
use super::HandlePoisonedResult as _;


impl<T: ?Sized> FragileTryContainer<T> for Arc<Mutex<T>> {
    type Ref<'a>  = MutexGuard<'a, T> where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(Mutex::new(t))
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self)
            .map(Mutex::into_inner)
            .map(Result::ignore_poisoned)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self.lock().panic_if_poisoned())
    }
}

impl<T: ?Sized> FragileContainer<T> for Arc<Mutex<T>> {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.lock().panic_if_poisoned()
    }
}

impl<T: ?Sized> FragileTryMutContainer<T> for Arc<Mutex<T>> {
    type RefMut<'a>  = MutexGuard<'a, T> where T: 'a;
    type RefMutError = Infallible;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self.lock().panic_if_poisoned())
    }
}

impl<T: ?Sized> FragileMutContainer<T> for Arc<Mutex<T>> {
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self.lock().panic_if_poisoned()
    }
}
