# Generic Container
Abstract over how a `T` is stored and accessed by using generic "containers", bounded by
the container traits here. A container owns a `T` and can provide references to it, or be
consumed to return the inner `T` (if `T` is [`Sized`]).

Some containers are fallible, or can only provide immutable references and not mutable
references. The container traits are meant to be specific enough that a generic container
can be bound more tightly by the interface it needs, to support more container implementations.

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
`dyn Trait` or a `T: Trait` [^blanket-container-t].

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
in particular), `Send`, and `Sync`.

Additionally, trait authors may wish to allow any generics bound by your traits to optionally
use `dyn` instead of monomorphizing everything; a standard way to allow for this is to
implement `YourTrait` for `Box<dyn YourTrait>` and perhaps a few other smart pointers, but even
better is providing a blanket implementation for `YourTrait` for any
`C: ?Sized + Container<dyn YourTrait>` [^container-dyn-trait].

## Example

```rust
// Note: "Fragile" means that, depending on the implementation, attempting to get multiple live
// borrows from the container in the same thread might panic or deadlock. In other words,
// fragile containers must not be assumed to be reentrant.
use generic_container::FragileContainer;

trait MyTrait {
    fn calculate_something(&self) -> u32;
}

// Wwhenever something is generic over `MyTrait`, thanks to this blanket impl, we could opt into
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
        // Note: "fragile"
        self.get_ref().calculate_something()
    }
}

struct NeedsToCalculateSomething<T: MyTrait>(T);

// This is effectively the same thing you'd get if you used a `dyn` object instead of a generic
// above; the user of your struct loses nothing.
type BoxedNeedsToCalculateSomething = NeedsToCalculateSomething<Box<dyn MyTrait>>;
```

# Container Traits
There are eight container traits provided here, with every combination of the following aspects:
- Mutability: whether a container can provide mutable references to the value stored in the
  container, in addition to immutable references.
- Fallibility: a container is fallible if `try_get_ref` can fail.
- Fragility: a fragile container is not guaranteed to be reentrant, and may panic or deadlock
  if `try_get_ref` (or similar) is called by a thread that already has a live reference to the
  value stored in the container.

Mutability is indicated by the prefix `Mut`, fallibility by the prefix `Try`, and fragility
by the prefix `Fragile`. They are combined in the order "`FragileTryMutContainer`".

Implementors of the traits must manually implement each of them; there are no blanket
`Container` implementations for `TryContainer` types whose error types are uninhabited,
for instance.

When bounding a generic by a container trait, you should generally bound by the minimum
container interface you need, plus any other marker trait needed, like [`Clone`] (or a cheap
clone like [`dupe::Dupe`]), `Send`, and `Sync`.

## Container Kind traits
Currently, Rust doesn't allow bounds like `where C: for<T> Container<T> + Clone + Send + Sync`.
The solution is to define an extra trait with whatever you need for a GAT's bounds:

```rust
use generic_container::MutContainer;
use dupe::Dupe;

pub trait NeededContainerStuff {
   type Container<T: ?Sized>: MutContainer<T> + Dupe + Send + Sync;
}
```

Here, such a trait is referred to as a "container kind trait" (with implementations being
"container kinds", just as implementations of the container traits are "containers").

There are 32 combinations of trait bounds most likely to be of concern, from the eight
container traits, with or without `Dupe` bounds, and with or without `Send + Sync` bounds.
`*ContainerKind` traits are provided for these cases. (Half of them are gated behind the
`dupe` feature.) Note also that none of these cases are `?Sized`; although a container type
*could* be unsized, presumably a `Sized` container is usually needed in practice. See the
[`kinds`] module for more.

TODO: what about debug bounds? Do I need `DebugContainerKind` and `DebugMutContainerKind`?
Is `#[derive(Debug)]` good enough?


## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.


TODO: add links to my types in the below footnotes

[^blanket-container-t]: To satisfy the trait solver and avoid conflicting trait implementations,
  a [`Contained<T, C>`] struct is provided. It is not necessary for blanket implementations
  over containers holding `dyn Trait`, but should be used for blanket implementations of `Trait`
  for containers holding an arbitrary `T: Trait`.
[^container-dyn-trait]: `Container<dyn Trait>` (where `Container` is the trait in this crate) might
  not be the best choice; [`FragileContainer`] is preferred if possible. Just as functions are
  encouraged to take [`FnOnce`] or [`FnMut`] callbacks rather than [`Fn`] (if possible), it would
  be best to accept a fragile container, if the `Trait`'s methods don't expose some potential for
  reentrancy with the container holding the `dyn Trait`. For example, if any of a
  `ReentrantTrait`'s methods take a `&self` parameter and take inputs that could, potentially, have
  some way of getting a reference to the wrapping [`FragileContainer`], then implementing such a
  method of `ReentrantTrait` for `FragileContainer<dyn ReentrantTrait>` would likely begin by
  calling `get_ref` on the container. Then, the other inputs of the method could call `get_ref` on
  their reference to the container. Containers which are actually fragile (and don't implement
  `TryContainer`) are probably refcounted and cloneable, so changing `&self` to `&mut self`
  doesn't help: if `ReentrantTrait` provides a user an opportunity to run arbitrary
  code inside one of its `&self` or `&mut self` methods, there's a potential problem.
  As such, while `FragileContainer<dyn ReentrantTrait>` could be used if you are careful,
  it would not work in every situation that the `ReentrantTrait` interface would normally
  require; therefore, a blanket implementation of `ReentrantTrait` for anything implementing
  `Container<dyn ReentrantTrait>` could be provided, but doing so for
  `FragileContainer<dyn ReentrantTrait>` would make it easy to hand a fragile container to
  something expecting a normal `ReentrantTrait`, leading to potential panics or deadlocks.

[`Sized`]: https://doc.rust-lang.org/std/marker/trait.Sized.html
[`Copy`]: https://doc.rust-lang.org/std/marker/trait.Copy.html
[`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
[`dupe::Dupe`]: https://docs.rs/dupe/0.9.1/dupe/trait.Dupe.html"
[`kinds`]: TODO
