use std::rc::Rc;
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};

use crate::container_traits::{
    FragileTryContainer, FragileTryMutContainer, TryContainer, TryMutContainer,
};


/// A thin wrapper around `Rc<RefCell<T>>` which implements the container traits differently:
/// `CheckedRcRefCell<T>` is fallible, but not [fragile].
///
/// Note that `Rc<RefCell<T>>` is an infallible but [fragile] container, essentially interpreting
/// any circumstance which *would* return an error in `CheckedRcRefCell<T>` as instead being a bug
/// and panicking. The user of a fragile container must uphold greater restrictions, in exchange for
/// infallibility.
///
/// [fragile]: TODO
/// [`FragileContainer`]: crate::container_traits::FragileContainer
/// [`FragileMutContainer`]: crate::container_traits::FragileMutContainer
#[derive(Debug, Clone)]
pub struct CheckedRcRefCell<T: ?Sized>(pub Rc<RefCell<T>>);

impl<T: ?Sized> FragileTryContainer<T> for CheckedRcRefCell<T> {
    type Ref<'a>  = Ref<'a, T> where T: 'a;
    type RefError = BorrowError;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self(Rc::new(RefCell::new(t)))
    }

    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Rc::into_inner(self.0).map(RefCell::into_inner)
    }

    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        self.0.try_borrow()
    }
}

impl<T: ?Sized> TryContainer<T> for Rc<RefCell<T>> {}

impl<T: ?Sized> FragileTryMutContainer<T> for CheckedRcRefCell<T> {
    type RefMut<'a>  = RefMut<'a, T> where T: 'a;
    type RefMutError = BorrowMutError;

    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        self.0.try_borrow_mut()
    }
}

impl<T: ?Sized> TryMutContainer<T> for Rc<RefCell<T>> {}
