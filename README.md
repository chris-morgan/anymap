``AnyMap``, a safe and convenient store for one value of each type
==================================================================

[![Build Status](https://travis-ci.org/chris-morgan/anymap.svg?branch=master)](https://travis-ci.org/chris-morgan/anymap)

If you’re familiar with Go and Go web frameworks, you may have come across the common “environment” pattern for storing data related to the request. It’s typically something like ``map[string]interface{}`` and is accessed with arbitrary strings which may clash and type assertions which are a little unwieldy and must be used very carefully. (Personally I would consider that it is just *asking* for things to blow up in your face.) In a language like Go, lacking in generics, this is the best that can be done; such a thing cannot possibly be made safe without generics.

As another example of such an interface, JavaScript objects are exactly the same—a mapping of string keys to arbitrary values. (There it is actually *more* dangerous, because methods and fields/attributes/properties are on the same plane.)

Fortunately, we can do better than these things in Rust. Our type system is quite equal to easy, robust expression of such problems.

The ``AnyMap`` type is a friendly wrapper around a ``HashMap<TypeId, Box<Any>>``, exposing a nice, easy typed interface, perfectly safe and absolutely robust.

What this means is that in an ``AnyMap`` you may store zero or one values for every type.

Instructions
------------

Cargo all the way: it is `anymap` on crates.io.

Unsafe code in this library
---------------------------

This library uses a fair bit of unsafe code for several reasons:

- To support Any and CloneAny, unsafe code is required (because of how the `downcast` methods are defined in `impl Any` rather than being trait methods; I think this is kind of a historical detail of the structure of `std::any::Any`); if you wanted to ditch `Clone` support this unsafety could be removed.

- In the interests of performance, skipping various checks that are unnecessary because of the invariants of the data structure (no need to check the type ID when it’s been statically ensured by being used as the hash map key) and simplifying hashing (type IDs are already good hashes, no need to mangle them through SipHash).

It’s not possible to remove all unsafety from this library without also removing some of the functionality. Still, at the cost of the `CloneAny` functionality, the raw interface and maybe the concurrency support, you can definitely remove all unsafe code. Here’s how you could do it:

- Remove the genericness of it all;
- Merge `anymap::raw` into the normal interface, flattening it;
- Change things like `.map(|any| unsafe { any.downcast_unchecked() })` to `.and_then(|any| any.downcast())` (performance cost: one extra superfluous type ID comparison, indirect);
- Ditch the `TypeIdHasher` since transmuting a `TypeId` is right out (cost: SIP mangling of a u64 on every access).

Yeah, the performance costs of going safe are quite small. The more serious matters are the loss of `Clone` and maybe `Send + Sync`.

But frankly, if you wanted to do all this it’d be easier and faster to write it from scratch. The core of the library is actually really simple and perfectly safe, as can be seen in [`src/lib.rs` in the first commit](https://github.com/chris-morgan/anymap/tree/a294948f57dee47bb284d6a3ae1b8f61a902a03c/src/lib.rs) (note that that code won’t run without a few syntactic alterations; it was from well before Rust 1.0 and has things like `Any:'static` where now we have `Any + 'static`).

## Colophon

**Authorship:** [Chris Morgan](https://chrismorgan.info/) is the author and maintainer of this library.

**Licensing:** this library is distributed under the terms of the
[Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0), the
[MIT License](https://opensource.org/licenses/MIT) and the
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0), at your choice.
See [COPYING](COPYING) for details.
