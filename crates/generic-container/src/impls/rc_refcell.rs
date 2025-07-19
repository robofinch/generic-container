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

    /// Attempt to retrieve the inner `T` from the container.
    /// Behaves identically to [`Rc::into_inner`].
    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Self::into_inner(self).map(RefCell::into_inner)
    }

    /// Get immutable access to the inner `T`.
    ///
    /// Uses [`RefCell::borrow`], so this container is
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    ///
    /// ## Panics
    /// Panics if the contract of a fragile container is broken.
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        Ok(self.borrow())
    }
}

impl<T: ?Sized> FragileContainer<T> for Rc<RefCell<T>> {
    /// Get immutable access to the inner `T`.
    ///
    /// Uses [`RefCell::borrow`], so this container is
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    ///
    /// ## Panics
    /// Panics if the contract of a fragile container is broken.
    #[inline]
    fn get_ref(&self) -> Self::Ref<'_> {
        self.borrow()
    }
}

impl<T: ?Sized> FragileTryMutContainer<T> for Rc<RefCell<T>> {
    type RefMut<'a>  = RefMut<'a, T> where T: 'a;
    type RefMutError = Infallible;

    /// Get mutable access to the inner `T`.
    ///
    /// Uses [`RefCell::borrow_mut`], so this container is
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    ///
    /// ## Panics
    /// Panics if the contract of a fragile container is broken.
    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        Ok(self.borrow_mut())
    }
}

impl<T: ?Sized> FragileMutContainer<T> for Rc<RefCell<T>> {
    /// Get mutable access to the inner `T`.
    ///
    /// Uses [`RefCell::borrow_mut`], so this container is
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    ///
    /// ## Panics
    /// Panics if the contract of a fragile container is broken.
    #[inline]
    fn get_mut(&mut self) -> Self::RefMut<'_> {
        self.borrow_mut()
    }
}
