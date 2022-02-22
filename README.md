# ``AnyMap``, a safe and convenient store for one value of each type

``AnyMap`` is a type-safe wrapper around ``HashMap<TypeId, Box<dyn Any>>`` that lets you not worry about ``TypeId`` or downcasting, but just get on with storing one each of a bag of diverse types, which is really useful for extensibility in some sorts of libraries.

## Background

If you’re familiar with Go and Go web frameworks, you may have come across the common “environment” pattern for storing data related to the request. It’s typically something like ``map[string]interface{}`` and is accessed with arbitrary strings which may clash and type assertions which are a little unwieldy and must be used very carefully. (Personally I would consider that it is just *asking* for things to blow up in your face.) In a language like Go, lacking in generics, this is the best that can be done; such a thing cannot possibly be made safe without generics.

As another example of such an interface, JavaScript objects are exactly the same—a mapping of string keys to arbitrary values. (There it is actually *more* dangerous, because methods and fields/attributes/properties are on the same plane—though it’s *possible* to use `Map` these days.)

Fortunately, we can do better than these things in Rust. Our type system is quite equal to easy, robust expression of such problems.

## Example

```rust
let mut data = anymap::AnyMap::new();
assert_eq!(data.get(), None::<&i32>);
data.insert(42i32);
assert_eq!(data.get(), Some(&42i32));
data.remove::<i32>();
assert_eq!(data.get::<i32>(), None);

#[derive(Clone, PartialEq, Debug)]
struct Foo {
    str: String,
}

assert_eq!(data.get::<Foo>(), None);
data.insert(Foo { str: format!("foo") });
assert_eq!(data.get(), Some(&Foo { str: format!("foo") }));
data.get_mut::<Foo>().map(|foo| foo.str.push('t'));
assert_eq!(&*data.get::<Foo>().unwrap().str, "foot");
```

## Features

- Store up to one value for each type in a bag.
- Add `Send` or `Send + Sync` bounds.
- You can opt into making the map `Clone`. (In theory you could add all kinds of other functionality, but you can’t readily make this work *generically*, and the bones of it are simple enough that it becomes better to make your own extension of `Any` and reimplement `AnyMap`.)
- no_std if you like.

## Cargo features/dependencies/usage

Typical Cargo.toml usage, providing `anymap::AnyMap` *et al.* backed by `std::collections::HashMap`:

```toml
[dependencies]
anymap = "1.0.0-beta.2"
```

No-std usage, providing `anymap::hashbrown::AnyMap` *et al.* (note the different path, required because Cargo features are additive) backed by `alloc` and the [hashbrown](https://rust-lang.github.io/hashbrown) crate:

```toml
[dependencies]
anymap = { version = "1.0.0-beta.2", default-features = false, features = ["hashbrown"] }
```

**On stability:** hashbrown is still pre-1.0.0 and experiencing breaking changes. Because it’s useful for a small fraction of users, I am retaining it, but with *different compatibility guarantees to the typical SemVer ones*. Where possible, I will just widen the range for new releases of hashbrown, but if an incompatible change occurs, I may drop support for older versions of hashbrown with a bump to the *minor* part of the anymap version number (e.g. 1.1.0, 1.2.0). Iff you’re using this feature, this is cause to *consider* using a tilde requirement like `"~1.0"` (or spell it out as `>=1, <1.1`).

## Unsafe code in this library

This library uses a fair bit of unsafe code for several reasons:

- To support `CloneAny`, unsafe code is currently required (because the downcast methods are defined on `dyn Any` rather than being trait methods, and upcasting is an incomplete feature in rustc); if you wanted to ditch `Clone` support this unsafety could be removed.

- For `dyn CloneAny + Send` and `dyn CloneAny + Send + Sync`’s `Clone` implementation, an unsafe block is used to attach the auto traits where safe code used to be used, in order to avoid a [spurious future-compatibility lint](https://github.com/rust-lang/rust/issues/51443#issuecomment-421988013).

- In the interests of performance, type ID checks are skipped as unnecessary because of the invariants of the data structure (though this does come at the cost of `Map::{as_raw_mut, into_raw}` being marked unsafe).

It is possible to remove all unsafe code at the cost of only `CloneAny` functionality and a little performance. The `safe` branch in the Git repository contains a couple of commits demonstrating the concept. It’s quite straightforward; the core of this library is very simple and perfectly safe.

## Colophon

**Authorship:** [Chris Morgan](https://chrismorgan.info/) is the author and maintainer of this library.

**Licensing:** this library is distributed under the terms of the
[Blue Oak Model License 1.0.0](https://blueoakcouncil.org/license/1.0.0), the
[MIT License](https://opensource.org/licenses/MIT) and the
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0), at your choice.
See [COPYING](COPYING) for details.
