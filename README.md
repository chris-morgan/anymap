# ``AnyMap``, a safe and convenient store for one value of each type

If you’re familiar with Go and Go web frameworks, you may have come across the common “environment” pattern for storing data related to the request. It’s typically something like ``map[string]interface{}`` and is accessed with arbitrary strings which may clash and type assertions which are a little unwieldy and must be used very carefully. (Personally I would consider that it is just *asking* for things to blow up in your face.) In a language like Go, lacking in generics, this is the best that can be done; such a thing cannot possibly be made safe without generics.

As another example of such an interface, JavaScript objects are exactly the same—a mapping of string keys to arbitrary values. (There it is actually *more* dangerous, because methods and fields/attributes/properties are on the same plane.)

Fortunately, we can do better than these things in Rust. Our type system is quite equal to easy, robust expression of such problems.

The ``AnyMap`` type is a friendly wrapper around a ``HashMap<TypeId, Box<dyn Any>>``, exposing a nice, easy typed interface, perfectly safe and absolutely robust.

What this means is that in an ``AnyMap`` you may store zero or one values for every type.

## Cargo features/dependencies/usage

Typical Cargo.toml usage:

```toml
[dependencies]
anymap = "1.0.0-beta.1"
```

No-std usage, using `alloc` and the [hashbrown](https://rust-lang.github.io/hashbrown) crate instead of `std::collections::HashMap`:

```toml
[dependencies]
anymap = { version = "1.0.0-beta.1", default-features = false, features = ["hashbrown"] }
```

The `std` feature is enabled by default. The `hashbrown` feature overrides it. At least one of the two must be enabled.

**On stability:** hashbrown is still pre-1.0.0 and experiencing breaking changes. Because it’s useful for a small fraction of users, I am retaining it, but with *different compatibility guarantees to the typical SemVer ones*. Where possible, I will just widen the range for new releases of hashbrown (e.g. change `0.12` to `>=0.12, <0.14` when they release 0.13.0), but if an incompatible change occurs, I may drop support for older versions of hashbrown with a bump to the *minor* part of the anymap version number (e.g. 1.1.0, 1.2.0). Iff you’re using this feature, this is cause to *consider* using a tilde requirement like `"~1.0"` (or spell it out as `>=1, <1.1`).

## Unsafe code in this library

This library uses a fair bit of unsafe code for several reasons:

- To support `CloneAny`, unsafe code is required (because the downcast methods are defined on `dyn Any` rather than being trait methods); if you wanted to ditch `Clone` support this unsafety could be removed.

- In the interests of performance, skipping various checks that are unnecessary because of the invariants of the data structure (no need to check the type ID when it’s been statically ensured by being used as the hash map key).

- In the `Clone` implementation of `dyn CloneAny` with `Send` and/or `Sync` auto traits added, an unsafe block is used where safe code used to be used in order to avoid a [spurious future-compatibility lint](https://github.com/rust-lang/rust/issues/51443#issuecomment-421988013).

It’s not possible to remove all unsafety from this library without also removing some of the functionality. Still, at the cost of the `CloneAny` functionality and the raw interface, you can definitely remove all unsafe code. Here’s how you could do it:

- Remove the genericness of it all (choose `dyn Any`, `dyn Any + Send` or `dyn Any + Send + Sync` and stick with it);
- Merge `anymap::raw` into the normal interface, flattening it;
- Change things like `.map(|any| unsafe { any.downcast_unchecked() })` to `.and_then(|any| any.downcast())` (performance cost: one extra superfluous type ID comparison, indirect).

Yeah, the performance costs of going safe are quite small. The more serious matter is the loss of `Clone`.

But frankly, if you wanted to do all this it’d be easier and faster to write it from scratch. The core of the library is actually really simple and perfectly safe, as can be seen in [`src/lib.rs` in the first commit](https://github.com/chris-morgan/anymap/tree/a294948f57dee47bb284d6a3ae1b8f61a902a03c/src/lib.rs) (note that that code won’t run without a few syntactic alterations; it was from well before Rust 1.0 and has things like `Any:'static` where now we have `Any + 'static`).

## Colophon

**Authorship:** [Chris Morgan](https://chrismorgan.info/) is the author and maintainer of this library.

**Licensing:** this library is distributed under the terms of the
[Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0), the
[MIT License](https://opensource.org/licenses/MIT) and the
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0), at your choice.
See [COPYING](COPYING) for details.
