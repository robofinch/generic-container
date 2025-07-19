# Generic Container (and Thread-Checked Lock)

Abstract over how a `T` is stored and accessed by using generic "containers", bounded by
the container traits here. A container owns a `T` and can provide references to it, or be
consumed to return the inner `T` (if `T` is [`Sized`]).

Some containers are fallible, or can only provide immutable references and not mutable
references. The container traits are meant to be specific enough that a generic container
can be bound more tightly by the interface it needs, to support more container implementations.

A [thread-checked mutex] type is provided because implementations of the "fragile" container
traits [permit their implementations to panic or deadlock] if [reentrancy] occurs;
[the standard libary's mutex may do this] if reentrancy occurs within a single thread. Non-fragile
container traits, however, require their implementations to not completely crash and burn in this
situation, and at least gracefully return an error. The thread-checked mutex does so: it checks
whether the current thread is trying to lock a thread-checked mutex that it already locked, and
returns an error in that case instead of panicking or deadlocking.

See the README of each crate for more details.

## Testing and Build Dependencies

Currently, there are no strictly necesary dependencies that aren't part of normal Rust toolchains,
though Rust 1.87 or above is needed to build. To use the `Justfile`, both `just` and `cargo-hack`
are necessary.

### Testing / Linting

Before pushing a commit, run `just clippy-all --no-cache` and `just test-all --no-cache`, which run
checks on supported combinations of features and several architectures. Initially,
`just add-targets` may need to be run. Occasionally, `just find-possible-missing-commas` should be
run and looked through.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.


[`Sized`]: https://doc.rust-lang.org/std/marker/trait.Sized.html
[thread-checked mutex]: https://docs.rs/thread-checked-lock/0/thread_checked_lock/struct.ThreadCheckedMutex.html
[permit their implementations to panic or deadlock]: crates/generic-container/README.md#fragility-potential-panics-or-deadlocks
[reentrancy]: https://en.wikipedia.org/wiki/Reentrancy_(computing)
[the standard libary's mutex may do this]: https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock
