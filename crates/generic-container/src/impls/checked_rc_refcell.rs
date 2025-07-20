use alloc::rc::Rc;
use core::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::container_traits::{
    FragileTryContainer, FragileTryMutContainer, TryContainer, TryMutContainer,
};


/// A thin wrapper around `Rc<RefCell<T>>` which implements the container traits differently:
/// `CheckedRcRefCell<T>` is fallible, but not [fragile].
///
/// Note that `Rc<RefCell<T>>` is an infallible but [fragile] container, essentially interpreting
/// any circumstance which *would* return an error in `CheckedRcRefCell<T>` as instead being a bug
/// and panicking. The user of a fragile container must uphold greater restrictions, in exchange for
/// infallibility. Those restrictions still effectively apply here, but violating them results in
/// errors being returned instead of being fatal.
///
/// [fragile]: crate#fragility-potential-panics-or-deadlocks
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckedRcRefCell<T: ?Sized>(pub Rc<RefCell<T>>);

impl<T: ?Sized> FragileTryContainer<T> for CheckedRcRefCell<T> {
    type Ref<'a>  = Ref<'a, T> where T: 'a;
    type RefError = BorrowError;

    #[inline]
    fn new_container(t: T) -> Self where T: Sized {
        Self(Rc::new(RefCell::new(t)))
    }

    /// Attempt to retrieve the inner `T` from the container.
    /// Behaves identically to [`Rc::into_inner`].
    #[inline]
    fn into_inner(self) -> Option<T> where T: Sized {
        Rc::into_inner(self.0).map(RefCell::into_inner)
    }

    /// Immutably borrows the inner `T`, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// Behaves identically to [`RefCell::try_borrow`].
    #[inline]
    fn try_get_ref(&self) -> Result<Self::Ref<'_>, Self::RefError> {
        self.0.try_borrow()
    }
}

impl<T: ?Sized> TryContainer<T> for Rc<RefCell<T>> {}

impl<T: ?Sized> FragileTryMutContainer<T> for CheckedRcRefCell<T> {
    type RefMut<'a>  = RefMut<'a, T> where T: 'a;
    type RefMutError = BorrowMutError;

    /// Mutably borrows the inner `T`, returning an error if the value is currently borrowed.
    ///
    /// Behaves identically to [`RefCell::try_borrow_mut`].
    #[inline]
    fn try_get_mut(&mut self) -> Result<Self::RefMut<'_>, Self::RefMutError> {
        self.0.try_borrow_mut()
    }
}

impl<T: ?Sized> TryMutContainer<T> for Rc<RefCell<T>> {}
