// See https://linebender.org/blog/doc-include for this README inclusion strategy
//! [Arc]: std::sync::Arc
//! [`Clone`]: Clone
//! [`Copy`]: Copy
//! [`Drop`]: Drop
//! [`FnOnce`]: FnOnce
//! [`Fn`]: Fn
//! [`FnMut`]: FnMut
//! [`Send`]: Send
//! [`Sized`]: Sized
//! [`Sync`]: Sync
//!
//! [`Ref`]: FragileTryContainer::Ref
//! [`RefMut`]: FragileTryMutContainer::RefMut
//! [`GenericContainer<T, C>`]: GenericContainer
//! [`TryContainer`]: TryContainer
//! [`TryMutContainer<T>`]: TryMutContainer
//!
// File links are not supported by rustdoc
//! [LICENSE-APACHE]: https://github.com/robofinch/generic-container/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/generic-container/blob/main/LICENSE-MIT
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]


#![cfg_attr(docsrs, feature(doc_cfg))]

#![no_std]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

#[cfg(any(feature = "std", doc))]
extern crate std;


mod container_traits;
mod impls;
mod generic_container;
#[cfg(any(feature = "kinds", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "kinds")))]
pub mod kinds;


// `dupe` is only used in doctests, which still triggers the `unused_crate_dependencies` lint.
#[cfg(test)]
use dupe as _;


pub use self::generic_container::GenericContainer;
pub use self::container_traits::{
    // The core eight
    FragileTryContainer,    TryContainer,    FragileContainer,    Container,
    FragileTryMutContainer, TryMutContainer, FragileMutContainer, MutContainer,

    // Non-nightly "trait aliases"
    BaseContainer, BaseMutContainer,
};

#[cfg(any(feature = "alloc", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub use self::impls::CheckedRcRefCell;

#[cfg(feature = "thread-checked-lock")]
#[cfg_attr(docsrs, doc(cfg(feature = "thread-checked-lock")))]
pub use self::impls::ErasedLockError;
