//! # Container Kind Traits
//!
//! Currently, Rust doesn't allow bounds like
//! `where C: for<T: Send> Container<T> + Clone + Send + Sync`.
//! The solution is to define an extra trait with whatever you need as the bounds of a GAT
//! (generic associated type):
//! ```
//! use generic_container::FragileMutContainer;
//! use dupe::Dupe;
//!
//! pub trait NeededContainerStuff {
//!    // Implementors should use `T: ?Sized` when possible. But right now, the only way to create,
//!    // for example, `Arc<Mutex<[T]>>` is via unsizing coercion from `Arc<Mutex<[T; N]>>`;
//!    // as such, a `T: ?Sized` bound would be somewhat useless without also requiring the
//!    // container to support unsizing coercion, which currently requires nightly-only traits.
//!    type MutableContainer<T: Send>: FragileMutContainer<T> + Dupe + Send + Sync;
//! }
//! ```
//!
//! If some data needs thread-safe mutability, but you don't want to pay the cost of a lock for
//! read-only data, you can use multiple GATs:
//! ```
//! use generic_container::{FragileMutContainer, Container};
//! use dupe::Dupe;
//!
//! pub trait NeededContainerStuff {
//!    // E.g.: `Arc<Mutex<T>>`, or something `parking_lot`-based
//!    type MutableContainer<T: Send>: FragileMutContainer<T> + Dupe + Send + Sync;
//!    // E.g.: `Arc<T>`
//!    type ReadOnlyContainer<T: Send + Sync>: Container<T> + Dupe + Send + Sync;
//! }
//! ```
//!
//! Such a trait is called a "container kind trait" (with implementations being "container kinds",
//! just as implementations of the container traits are "containers"). Relevant characteristics for
//! a container kind's container include the eight container traits, `Send + Sync` bounds (possibly
//! only when `T` is `Send + Sync`, or just `Send`), [`Dupe`], whether the GAT allows `T` to be
//! unsized, and `Debug` bounds.
//!
//! Not every conceivably-useful container kind trait is provided, as there would be
//! exponentially-many such traits. Note also that creating simple container kind traits that can
//! be combined into more complicated bounds does *work*, but not well. A container kind should set
//! the GAT of each container kind trait it implements to the same container type; this *can* be
//! asserted or required with Rust's type system, but the trait solver doesn't understand it very
//! well. Such container kind traits would likely not be pleasant to use.
//!
//! When the `kinds` feature is enabled, container kinds for common sorts of containers are
//! provided. They do not use [`Dupe`] bounds, and do not mess with type-equality shenanigans that
//! confuse the trait solver.
//!
//! [`Dupe`]: https://docs.rs/dupe/0.9/dupe/trait.Dupe.html

use crate::container_traits::{
    Container, FragileContainer, FragileMutContainer, MutContainer, TryMutContainer,
};


// ================================
//  Container Kind Traits
// ================================

/// A [container kind trait](self) based on how a type `T` acts as a container for itself.
///
/// Has strictly looser requirements than [`FragileTLike`].
pub trait TLike {
    /// A `T`-like container type
    type Container<T>: MutContainer<T>;
}

/// A [container kind trait](self) based on how a type `T` acts as a container for itself.
///
/// `T`, however, is not a [fragile](crate#fragility-potential-panics-or-deadlocks) container;
/// this kind trait loosens that requirement.
pub trait FragileTLike {
    /// A `T`-like container type, but permitted to be
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    type Container<T>: FragileMutContainer<T>;
}

/// A [container kind trait](self) based on how `Box<T>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`TLike`], [`FragileTLike`], and [`FragileBoxLike`].
pub trait BoxLike {
    /// A `Box<T>`-like container type
    type Container<T: ?Sized>: MutContainer<T>;
}

/// A [container kind trait](self) based on how `Box<T>` acts as a container for `T`.
///
/// `Box<T>`, however, is not a [fragile](crate#fragility-potential-panics-or-deadlocks) container;
/// this kind trait loosens that requirement.
///
/// Has strictly looser requirements than [`FragileTLike`].
pub trait FragileBoxLike {
    /// A `Box<T>`-like container type, but permitted to be
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    type Container<T: ?Sized>: FragileMutContainer<T>;
}

/// A [container kind trait](self) based on how `Rc<T>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`FragileRcLike`].
pub trait RcLike {
    /// An `Rc<T>`-like container type
    type Container<T: ?Sized>: Container<T> + Clone;
}

/// A [container kind trait](self) based on how `Rc<T>` acts as a container for `T`.
///
/// `Rc<T>`, however, is not a [fragile](crate#fragility-potential-panics-or-deadlocks) container;
/// this kind trait loosens that requirement.
pub trait FragileRcLike {
    /// An `Rc<T>`-like container type, but permitted to be
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    type Container<T: ?Sized>: FragileContainer<T> + Clone;
}

/// A [container kind trait](self) based on how `Rc<RefCell<T>>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`FragileRcLike`].
pub trait RcRefCellLike {
    /// An `Rc<RefCell<T>>`-like container type
    type Container<T: ?Sized>: FragileMutContainer<T> + Clone;
}

/// A [container kind trait](self) based on how `Arc<T>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`FragileArcLike`].
pub trait ArcLike {
    /// An `Arc<T>`-like container type
    type Container<T: ?Sized + Send + Sync>: Container<T> + Clone + Send + Sync;
}

/// A [container kind trait](self) based on how `Arc<T>` acts as a container for `T`.
///
/// `Arc<T>`, however, is not a [fragile](crate#fragility-potential-panics-or-deadlocks) container;
/// this kind trait loosens that requirement.
pub trait FragileArcLike {
    /// An `Arc<T>`-like container type, but permitted to be
    /// [fragile](crate#fragility-potential-panics-or-deadlocks).
    type Container<T: ?Sized + Send + Sync>: FragileContainer<T> + Clone + Send + Sync;
}

/// A [container kind trait](self) based on how `Arc<RwLock<T>>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`FragileArcLike`].
pub trait ArcRwLockLike {
    /// An `Arc<RwLock<T>>`-like container type
    type Container<T: ?Sized + Send + Sync>: FragileMutContainer<T> + Clone + Send + Sync;
}

/// A [container kind trait](self) based on how `Arc<Mutex<T>>` acts as a container for `T`.
///
/// Has strictly looser requirements than [`ArcRwLockLike`] and [`FragileArcLike`].
pub trait ArcMutexLike {
    /// An `Arc<Mutex<T>>`-like container type
    type Container<T: ?Sized + Send>: FragileMutContainer<T> + Clone + Send + Sync;
}

/// A [container kind trait](self) based on how [`CheckedRcRefCell<T>`] acts as a container for `T`.
///
#[cfg_attr(
    feature = "alloc",
    doc = "[`CheckedRcRefCell<T>`]: crate::CheckedRcRefCell",
)]
#[cfg_attr(
    not(feature = "alloc"),
    doc = "[`CheckedRcRefCell<T>`]: \
    https://docs.rs/generic-container/0/generic_container/struct.CheckedRcRefCell.html",
)]
pub trait CheckedRcRefCellLike {
    /// A [`CheckedRcRefCell<T>`]-like container type
    ///
    #[cfg_attr(
        feature = "alloc",
        doc = "[`CheckedRcRefCell<T>`]: crate::CheckedRcRefCell",
    )]
    #[cfg_attr(
        not(feature = "alloc"),
        doc = "[`CheckedRcRefCell<T>`]: \
        https://docs.rs/generic-container/0/generic_container/struct.CheckedRcRefCell.html",
    )]
    type Container<T: ?Sized>: TryMutContainer<T> + Clone;
}

/// A [container kind trait](self) based on how <code>Arc<[ThreadCheckedMutex]\<T\>></code> acts as
/// a container for `T`.
///
#[cfg_attr(
    feature = "thread-checked-lock",
    doc = "[ThreadCheckedMutex]: thread_checked_lock::ThreadCheckedMutex",
)]
#[cfg_attr(
    not(feature = "thread-checked-lock"),
    doc = "[ThreadCheckedMutex]: \
    https://docs.rs/thread-checked-lock/0/thread_checked_lock/struct.ThreadCheckedMutex.html",
)]
pub trait ArcThreadCheckedMutexLike {
    /// An <code>Arc<[ThreadCheckedMutex]\<T\>></code>-like container type
    ///
    #[cfg_attr(
        feature = "thread-checked-lock",
        doc = "[ThreadCheckedMutex]: thread_checked_lock::ThreadCheckedMutex",
    )]
    #[cfg_attr(
        not(feature = "thread-checked-lock"),
        doc = "[ThreadCheckedMutex]: \
        https://docs.rs/thread-checked-lock/0/thread_checked_lock/struct.ThreadCheckedMutex.html",
    )]
    type Container<T: ?Sized + Send>: TryMutContainer<T> + Clone + Send + Sync;
}

// ================================
//  Container Kinds
// ================================

/// The [container kind](crate::kinds) corresponding to `T` as a container for itself.
#[cfg_attr(docsrs, doc(cfg(feature = "kinds")))]
#[derive(Default, Debug, Clone, Copy)]
pub struct TKind;

impl TLike for TKind {
    type Container<T> = T;
}

impl FragileTLike for TKind {
    type Container<T> = T;
}

#[cfg(any(feature = "alloc", doc))]
mod alloc_kinds {
    use core::cell::RefCell;
    use alloc::{boxed::Box, rc::Rc, sync::Arc};

    use crate::impls::CheckedRcRefCell;
    use super::{
        ArcLike, BoxLike, CheckedRcRefCellLike,
        FragileArcLike, FragileBoxLike, FragileTLike, FragileRcLike,
        RcLike, RcRefCellLike, TLike,
    };


    /// The [container kind](crate::kinds) corresponding to `Box<T>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct BoxKind;

    impl BoxLike for BoxKind {
        type Container<T: ?Sized> = Box<T>;
    }

    impl FragileBoxLike for BoxKind {
        type Container<T: ?Sized> = Box<T>;
    }

    impl TLike for BoxKind {
        type Container<T> = Box<T>;
    }

    impl FragileTLike for BoxKind {
        type Container<T> = Box<T>;
    }

    /// The [container kind](crate::kinds) corresponding to `Rc<T>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct RcKind;

    impl RcLike for RcKind {
        type Container<T: ?Sized> = Rc<T>;
    }

    impl FragileRcLike for RcKind {
        type Container<T: ?Sized> = Rc<T>;
    }

    /// The [container kind](crate::kinds) corresponding to `Arc<T>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct ArcKind;

    impl ArcLike for ArcKind {
        type Container<T: ?Sized + Send + Sync> = Arc<T>;
    }

    impl FragileArcLike for ArcKind {
        type Container<T: ?Sized + Send + Sync> = Arc<T>;
    }

    impl RcLike for ArcKind {
        type Container<T: ?Sized> = Arc<T>;
    }

    impl FragileRcLike for ArcKind {
        type Container<T: ?Sized> = Arc<T>;
    }

    /// The [container kind](crate::kinds) corresponding to `Rc<RefCell<T>>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct RcRefCellKind;

    impl RcRefCellLike for RcRefCellKind {
        type Container<T: ?Sized> = Rc<RefCell<T>>;
    }

    impl FragileRcLike for RcRefCellKind {
        type Container<T: ?Sized> = Rc<RefCell<T>>;
    }

    /// The [container kind](crate::kinds) corresponding to [`CheckedRcRefCell<T>`] as a container
    /// for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "alloc", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct CheckedRcRefCellKind;

    impl CheckedRcRefCellLike for CheckedRcRefCellKind {
        type Container<T: ?Sized> = CheckedRcRefCell<T>;
    }
}

#[cfg(any(feature = "alloc", doc))]
pub use self::alloc_kinds::{ArcKind, BoxKind, CheckedRcRefCellKind, RcKind, RcRefCellKind};

#[cfg(any(feature = "std", doc))]
mod std_kinds {
    use alloc::sync::Arc;
    use std::sync::{Mutex, RwLock};

    use super::{ArcMutexLike, ArcRwLockLike, FragileArcLike};


    /// The [container kind](crate::kinds) corresponding to `Arc<RwLock<T>>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct ArcRwLockKind;

    impl ArcRwLockLike for ArcRwLockKind {
        type Container<T: ?Sized + Send + Sync> = Arc<RwLock<T>>;
    }

    impl FragileArcLike for ArcRwLockKind {
        type Container<T: ?Sized + Send + Sync> = Arc<RwLock<T>>;
    }

    /// The [container kind](crate::kinds) corresponding to `Arc<Mutex<T>>` as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct ArcMutexKind;

    impl ArcMutexLike for ArcMutexKind {
        type Container<T: ?Sized + Send> = Arc<Mutex<T>>;
    }

    impl ArcRwLockLike for ArcMutexKind {
        type Container<T: ?Sized + Send + Sync> = Arc<Mutex<T>>;
    }

    impl FragileArcLike for ArcMutexKind {
        type Container<T: ?Sized + Send + Sync> = Arc<Mutex<T>>;
    }
}

#[cfg(any(feature = "std", doc))]
pub use self::std_kinds::{ArcMutexKind, ArcRwLockKind};

#[cfg(feature = "thread-checked-lock")]
mod thread_checked_lock_kinds {
    use alloc::sync::Arc;

    use thread_checked_lock::ThreadCheckedMutex;

    use super::{ArcThreadCheckedMutexLike, CheckedRcRefCellLike};


    /// The [container kind](crate::kinds) corresponding to
    /// <code>[Arc]<[ThreadCheckedMutex]\<T\>></code> as a container for `T`.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "thread-checked-lock", feature = "kinds"))))]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct ArcThreadCheckedMutexKind;

    impl ArcThreadCheckedMutexLike for ArcThreadCheckedMutexKind {
        type Container<T: ?Sized + Send> = Arc<ThreadCheckedMutex<T>>;
    }

    impl CheckedRcRefCellLike for ArcThreadCheckedMutexKind {
        type Container<T: ?Sized> = Arc<ThreadCheckedMutex<T>>;
    }
}

#[cfg(feature = "thread-checked-lock")]
pub use self::thread_checked_lock_kinds::ArcThreadCheckedMutexKind;
