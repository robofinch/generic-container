use std::{convert::Infallible, rc::Rc};
use std::cell::{Ref, RefCell, RefMut};

use crate::container_traits::{
    FragileContainer, FragileMutContainer, FragileTryContainer, FragileTryMutContainer,
};


impl<T: ?Sized> FragileTryContainer<T> for Rc<RefCell<T>> {
    type Ref<'a>  = Ref<'a, T> where T: 'a;
    type RefError = Infallible;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self::new(RefCell::new(t))
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self).map(RefCell::into_inner)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self.borrow())
    }
}

impl<T: ?Sized> FragileContainer<T> for Rc<RefCell<T>> {
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.borrow()
    }
}

impl<T: ?Sized> FragileTryMutContainer<T> for Rc<RefCell<T>> {
    type RefMut<'a>  = RefMut<'a, T> where T: 'a;
    type RefMutError = Infallible;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self.borrow_mut())
    }
}

impl<T: ?Sized> FragileMutContainer<T> for Rc<RefCell<T>> {
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self.borrow_mut()
    }
}
