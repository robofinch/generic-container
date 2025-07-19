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


mod container_traits;
mod impls;
mod generic_container;


// `dupe` is only used in doctests, which still triggers the `unused_crate_dependencies` lint.
#[cfg(test)]
use dupe as _;

pub use self::{generic_container::GenericContainer, impls::CheckedRcRefCell};
pub use self::container_traits::{
    // The core eight
    FragileTryContainer,    TryContainer,    FragileContainer,    Container,
    FragileTryMutContainer, TryMutContainer, FragileMutContainer, MutContainer,

    // Non-nightly "trait aliases"
    BaseContainer, BaseMutContainer,
};

#[cfg(feature = "thread-checked-lock")]
pub use self::impls::ErasedLockError;
