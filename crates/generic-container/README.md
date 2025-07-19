<div align="center" class="rustdoc-hidden">
<h1> Generic Container </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-generic--container-08f?logo=github" height="20">](https://github.com/robofinch/generic-container/tree/main/crates/generic-container)
[![Latest version](https://img.shields.io/crates/v/generic-container.svg)](https://crates.io/crates/generic-container)
[![Documentation](https://img.shields.io/docsrs/generic-container)](https://docs.rs/generic-container)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

Abstract over how a `T` is stored and accessed by using generic "containers", bounded by
the container traits here. A container owns a `T` and can provide references to it, or be
consumed to return the inner `T` (if `T` is [`Sized`]).

Some containers are fallible, or can only provide immutable references and not mutable
references. The container traits are meant to be specific enough that a generic container
can be bound more tightly by the interface it needs, to support more container implementations.

Skip to [here](#provided-container-implementations) for a list of the provided container
implementations.

## Motivation

Someone who only needs to use your type from a single thread, will never clone it, and will only
use a single concrete type for a generic or `dyn` will likely have very different preferences
from someone who wants to use your type from multiple threads and instantiate a bulky generic
type with many combinations of generic parameters.

This concern is likely not even applicable for `Clone + Send + Sync` types that have no
generics, and in the other direction, the two use cases may simply be far too different to
cleanly unify.

But when "how will this `T` be stored?" is a major blocker against flexibly supporting both
sorts of uses in a performant way, this crate can help. The traits here allow a type
to abstract over how something is held and accessed by using a generic "container", and
blanket implementations for `Trait` can implement `Trait` for any container wrapping
`dyn Trait`, or effectively `T: Trait` in general with the help of a container wrapper type
[^blanket-container-t].

Choices like these can be deferred to the consumer of your types (or traits):
- Is the tradeoff between atomic refcounts and `Send` or `Sync` worth it?
- Will the consumer never clone your type, or is it necessary to be able to cheaply clone
  something with a copyable or refcounted container (which could be signaled by the container
  implementing [`Copy`] or [`dupe::Dupe`], or perhaps by the container implementing [`Clone`]
  even when the contained type is not [`Clone`])?
- Will the impact of monomorphized generics on the binary size outweigh their better
  optimization compared to `dyn`, or are some of the concrete types involved not known until
  runtime? If so, a user may prefer or need a `dyn` trait object, and from the first two
  bullets, maybe a user would need `Rc<dyn Trait>` rather than `Box<dyn Trait>`, and in general
  may want something like `Container<dyn Trait>`[^container-dyn-trait].

Particular traits worth abstracting over are [`Clone`] (or a cheap clone like [`dupe::Dupe`]
in particular), [`Send`], and [`Sync`].

Additionally, trait authors may wish to allow any generics bound by your traits to optionally
use `dyn` instead of monomorphizing everything; a standard way to allow for this is to
implement `YourTrait` for `Box<dyn YourTrait>` and perhaps a few other smart pointers, but even
better is providing a blanket implementation for `YourTrait` for any
`C: ?Sized + Container<dyn YourTrait>` [^container-dyn-trait].

## Example

```rust
use generic_container::FragileContainer;

trait MyTrait {
    fn calculate_something(&self) -> u32;
}

// Whenever something is generic over `MyTrait`, thanks to this blanket impl, we could opt into
// using `Box<dyn MyTrait>`, `Arc<dyn MyTrait>`, and so on. This way, being generic over
// `MyTrait` gives the user a strict superset of the abilities they'd have if we only used
// `dyn MyTrait` internally.
impl<C: ?Sized + FragileContainer<dyn MyTrait>> MyTrait for C {
    // You should warn users like this when depending on `FragileContainer`.
    // Note that someone solely using the `MyTrait` interface could not encounter this problem:
    // any `FragileContainer<dyn MyTrait>` is a perfect drop-in for a `MyTrait`.
    //
    /// Calculate something using the internal `MyTrait` implementation.
    ///
    /// # Fragility: Potential Panics or Deadlocks
    /// If `C` does not also implement `Container` and there is an existing borrow from `C`
    /// at the time `calculate_something` is called, a panic or deadlock may occur.
    ///
    /// See [`FragileContainer::get_ref`].
    #[inline]
    fn calculate_something(&self) -> u32 {
        self.get_ref().calculate_something()
    }
}

struct NeedsToCalculateSomething<T: MyTrait>(T);

// This is effectively the same thing you'd get if you used a `dyn` object instead of a generic
// above; the user of your struct loses nothing.
type NeedsToCalculateSomethingDyn = NeedsToCalculateSomething<Box<dyn MyTrait>>;
```

# Container Traits
Eight main container traits are provided here, with every combination of the following aspects:
- Mutability: whether a container can provide mutable references to the value stored in the
  container, in addition to immutable references.
- Fallibility: a container is fallible if `try_get_ref` or `try_get_mut` can fail, and
  `get_ref` or `get_mut` cannot be provided.
- Fragility: a fragile container is not guaranteed to support reentrancy, and may panic or
  deadlock if `try_get_ref` (or similar) is called by a thread that already has a live reference
  to the value stored in the container.

Mutability is indicated by the prefix `Mut`, fallibility by the prefix `Try`, and fragility
by the prefix `Fragile`. They are combined in the order "`FragileTryMutContainer`".

There are also two de facto trait aliases which replace "`FragileTry`" with "`Base`" in the two
`FragileTry*Container` traits.

Except for the `Base*Container` traits intended as aliases, implementors of the traits
must manually implement each of them; there are no blanket [`Container`] implementations for
[`TryContainer`] types whose error types are uninhabited, for instance.

When bounding a generic by a container trait, you should generally bound by the minimum
container interface you need, plus any other marker trait needed, like [`Clone`] (or a cheap
clone like [`dupe::Dupe`]), [`Send`], and [`Sync`]. Conversely, you should bound the `T` which
the container is required to store as tightly as you can, especially by [`Sized`], [`Send`],
and [`Sync`].

## Fragility: Potential Panics or Deadlocks

The [`Ref`] associated types of some container implementations (and [`RefMut`] for mutable
containers) may have nontrivial [`Drop`] impls that interfere with other attempts to borrow from
the container, and possibly from other clones of the container referencing the same inner data.
Such a container might be "fragile", unless it implements additional container traits
indicating that it is not.

Being "fragile" means that a single thread should not attempt to get multiple live references
to the `T` in the container, whether from the same container instance or clones referencing the
same inner `T`. Doing so risks a panic or deadlock (such as in the case of `Arc<Mutex<T>>`).
In other words, the fragile container traits do not guarantee that the container handles
[reentrancy] gracefully. A thread should drop any borrow obtained from the fragile container's
methods accessing its inner `T` before a new reference to the inner `T` can be obtained without
any risk of panic or deadlock.

## Containers
Common examples of containers include `T` itself, `Box<T>`, `Rc<T>`, `Rc<RefCell<T>>`, `Arc<T>`,
`Arc<RwLock<T>>`, and `Arc<Mutex<T>>`.

Additionally, container traits are implemented for [`CheckedRcRefCell<T>`] (from this crate) and
<code>[Arc]<[ThreadCheckedMutex]\<T\>></code> (when the `thread-checked-lock` feature is enabled).

Note that `CheckedRcRefCell<T>` and `Arc<ThreadCheckedMutex<T>>` essentially shift how the
runtime invariants of a `RefCell` or `Mutex` are enforced; with a fragile implementation, the
user is required to enforce them (on pain of panics or deadlocks), while with a fallible
implementation, such usage will still fail, but not fatally. Defaulting to the fragile
implementations is likely the best choice.

Some container implementations choose to panic if a poison error is encountered, as a poison
error can only occur if another thread has already panicked.

Other crates may implement container traits for their own types.

## Provided Container Implementations
This crate provides the following container implementations:

- For `MutContainer<T>` (and its supertraits):
  - `T` itself
  - `Box<T>`

- For `Container<T>` (and its supertraits):
  - `Rc<T>`
  - `Arc<T>`

- For `FragileMutContainer<T>` (and its supertraits):
  - `Rc<RefCell<T>>`
  - `Arc<RwLock<T>>` (implementation may panic on poison)
  - `Arc<Mutex<T>>` (implementation may panic on poison)

- For `TryMutContainer<T>` (and its supertraits):
  - `CheckedRcRefCell<T>`
  - `Arc<ThreadCheckedMutex<T>>` (only if the `thread-checked-lock` feature is enabled)

## Container Kind traits

Currently, Rust doesn't allow bounds like
`where C: for<T: Send> Container<T> + Clone + Send + Sync`.
The solution is to define an extra trait with whatever you need as the bounds of a GAT
(generic associated type):
```rust
use generic_container::FragileMutContainer;
use dupe::Dupe;

pub trait NeededContainerStuff {
   // Implementors should use `T: ?Sized` when possible. But right now, the only way to create,
   // for example, `Arc<Mutex<[T]>>` is via unsizing coercion from `Arc<Mutex<[T; N]>>`;
   // as such, a `T: ?Sized` bound would be somewhat useless without also requiring the
   // container to support unsizing coercion, which currently requires nightly-only traits.
   type MutableContainer<T: Send>: FragileMutContainer<T> + Dupe + Send + Sync;
}
```

If some data needs thread-safe mutability, but you don't want to pay the cost of a lock for
read-only data, you can use multiple GATs:
```rust
use generic_container::{FragileMutContainer, Container};
use dupe::Dupe;

pub trait NeededContainerStuff {
   // E.g.: `Arc<Mutex<T>>`, or something `parking_lot`-based
   type MutableContainer<T: Send>: FragileMutContainer<T> + Dupe + Send + Sync;
   // E.g.: `Arc<T>`
   type ReadOnlyContainer<T: Send + Sync>: Container<T> + Dupe + Send + Sync;
}
```

Such a trait could be referred to as a "container kind trait" (with implementations being
"container kinds", just as implementations of the container traits are "containers").
Relevant characteristics for a container kind's container include the eight container traits,
`Send + Sync` bounds (possibly only when `T` is `Send + Sync`, or just `Send`), [`Dupe`],
whether the GAT allows `T` to be unsized, and `Debug` bounds.

Unfortunately, creating one container kind trait for each combination of bounds requires
exponentially many traits, and creating simple container kind traits that can be combined into
more complicated bounds does *work*, but not well. A container kind should set the GAT of each
container kind trait it implements to the same container type; this *can* be asserted or
required with Rust's type system, but the trait solver doesn't understand it very well. Such
container kind traits would likely not be pleasant to use.

As such, container kind traits are not provided here; you should create traits with GATs
as needed.

# Features

- `thread-checked-lock`: if enabled, [`TryMutContainer<T>`] is implemented for
  <code>[Arc]<[ThreadCheckedMutex]\<T\>></code>.

# MSRV

Rust 1.85, the earliest version of the 2024 edition, is supported.

# License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE][])
* MIT license ([LICENSE-MIT][])

at your option.


[^blanket-container-t]: To satisfy the trait solver and avoid conflicting trait implementations,
  a [`GenericContainer<T, C>`] struct is provided. It is not necessary for blanket
  implementations over containers holding `dyn Trait`, but should be used for blanket
  implementations of `Trait` for containers holding an arbitrary `T: Trait` or
  `T: ?Sized + Trait`. (Note that `dyn Trait: !Sized + Trait`, so an implementation for
  `dyn Trait` does not conflict with a blanket implementation for `T: Trait`, but a blanket
  implementation for `T: ?Sized + Trait` includes the `dyn Trait` case.)
[^container-dyn-trait]: `Container<dyn Trait>` might not be the best choice;
  [`FragileContainer`] is preferred if possible. Just as functions are encouraged to take
  [`FnOnce`] or [`FnMut`] callbacks rather than [`Fn`] (if possible), it would be best to accept
  a fragile container, if the `Trait`'s methods don't expose some potential for reentrancy with
  the container holding the `dyn Trait`. For example, if any of a `ReentrantTrait`'s methods
  take a `&self` parameter and take inputs that could, potentially, have some way of getting a
  reference to the wrapping [`FragileContainer`], then implementing such a method of
  `ReentrantTrait` for `FragileContainer<dyn ReentrantTrait>` would likely begin by calling
  `get_ref` on the container. Then, the other inputs of the method could call `get_ref` on
  their reference to the container. Containers which are actually fragile (and don't implement
  [`TryContainer`]) are probably refcounted and cloneable, so changing `&self` to `&mut self`
  doesn't help: if `ReentrantTrait` provides a user an opportunity to run arbitrary
  code inside one of its `&self` or `&mut self` methods, there's a potential problem.
  As such, while `FragileContainer<dyn ReentrantTrait>` could be used if you are careful,
  it would not work in every situation that the `ReentrantTrait` interface would normally
  require; therefore, a blanket implementation of `ReentrantTrait` for anything implementing
  `Container<dyn ReentrantTrait>` could be provided, but doing so for
  `FragileContainer<dyn ReentrantTrait>` would make it easy to hand a fragile container to
  something expecting a normal `ReentrantTrait`, leading to potential panics or deadlocks.

[`dupe::Dupe`]: https://docs.rs/dupe/0.9/dupe/trait.Dupe.html
[`Dupe`]: https://docs.rs/dupe/0.9/dupe/trait.Dupe.html
[reentrancy]: https://en.wikipedia.org/wiki/Reentrancy_(computing)
[ThreadCheckedMutex]:
  https://docs.rs/thread-checked-lock/0/thread_checked_lock/struct.ThreadCheckedMutex.html

[Arc]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
[`Copy`]: https://doc.rust-lang.org/std/marker/trait.Copy.html
[`Drop`]: https://doc.rust-lang.org/std/ops/trait.Drop.html
[`FnOnce`]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
[`FnMut`]: https://doc.rust-lang.org/std/ops/trait.FnMut.html
[`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`Sized`]: https://doc.rust-lang.org/std/marker/trait.Sized.html
[`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html

[`Ref`]: https://docs.rs/generic-container/0/generic_container/trait.FragileTryContainer.html#associatedtype.Ref
[`RefMut`]: https://docs.rs/generic-container/0/generic_container/trait.FragileTryMutContainer.html#associatedtype.RefMut
[`GenericContainer<T, C>`]: https://docs.rs/generic-container/0/generic_container/struct.GenericContainer.html
[`TryContainer`]: https://docs.rs/generic-container/0/generic_container/trait.TryContainer.html
[`TryMutContainer<T>`]: https://docs.rs/generic-container/0/generic_container/trait.TryMutContainer.html

[LICENSE-APACHE]: ../../LICENSE-APACHE
[LICENSE-MIT]: ../../LICENSE-MIT
