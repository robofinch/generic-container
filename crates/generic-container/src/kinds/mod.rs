//! ## Container Kinds and Container Kind traits
//!
//! Currently, Rust doesn't allow bounds like `where C: for<T> Container<T> + Clone + Send + Sync`.
//! The solution is to define an extra trait with whatever you need for a GAT's bounds:
//!
//! ```
//! use generic_container::MutContainer;
//! use dupe::Dupe;
//!
//! pub trait NeededContainerStuff {
//!    type Container<T: ?Sized>: MutContainer<T> + Dupe + Send + Sync;
//! }
//! ```
//!
//! Here, such a trait is referred to as a "container kind trait" (with implementations being
//! "container kinds", just as implementations of the container traits are "containers").
//!
//! There are 32 combinations of trait bounds most likely to be of concern, from the eight
//! container traits, with or without [`Dupe`] bounds, and with or without `Send + Sync` bounds.
//! `*ContainerKind` traits are provided for these cases. (Half of them are gated behind the
//! `dupe` feature.) Note also that none of these cases are `?Sized`; although a container type
//! *could* be unsized, presumably a `Sized` container is usually needed in practice.
//!
//! TODO: what about debug bounds? Do I need `DebugContainerKind` and `DebugMutContainerKind`?
//! Is `#[derive(Debug)]` good enough?
//!
//!
//! It is the responsibility of any container implementation to provide a corresponding
//! container kind implementation. For implementors, it is worth noting that there are blanket
//! implementations from each non-fragile container kind to the corresponding fragile container
//! kind, as well as from [`MutContainerKind`] to [`ContainerKind`] and [`TryMutContainerKind`],
//! and from [`ContainerKind`] to [`TryContainerKind`].
//! Blanket implementations from threadsafe kinds to non-threadsafe kinds, or from dupable kinds
//! to non-dupable kinds, are not provided. Not every blanket implementation that one might wish
//! for are provided, as they would conflict with each other (even if any two conflicting
//! implementations would be the same).
//!
#![cfg_attr(
    not(feature = "dupe"),
    doc = " [`Dupe`]: https://docs.rs/dupe/0.9.1/dupe/trait.Dupe.html",
)]
#![cfg_attr(
    feature = "dupe",
    doc = " [`Dupe`]: dupe::Dupe",
)]

// TODO TODO TODO: there's a TODO in the module documentation above. Don't forget about it.


#[cfg(feature = "dupe")]
mod dupe_kind_traits;


#[cfg(feature = "dupe")]
pub use self::dupe_kind_traits::*;


// macro_rules! define_container_kind {
//     ($kind_name:ident, $container_trait:ident $(+ $extra_bound:ident)*) => {
//         pub trait $kind_name {
//             type $container_trait<T: ?Sized>: ?Sized + $container_trait<T>
//         }
//     };
// }


// /// The [container kind][self] corresponding to the [`FragileTryContainer`] container trait.
// pub trait FragileTryContainerKind {
//     /// A container which implements [`FragileTryContainer`].
//     type FragileTryContainer<T: ?Sized>: ?Sized + FragileTryContainer<T>;
// }

// pub trait TryContainerKind {

// }

