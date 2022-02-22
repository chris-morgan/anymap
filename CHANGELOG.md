# 1.0.0 (unreleased)

Planned once the dust of 1.0.0-beta.1 settles, since 1.0.0-beta.1 ended up
being bigger than I’d earlier intended.

# 1.0.0-beta.2 (unreleased)

- Fixed the broken `Extend` implementation added in 1.0.0-beta.1.

- Split the hashbrown implementation into a new module, `hashbrown`:
  std and hashbrown can now coexist completely peacefully,
  with `anymap::Map` being powered by `std::collections::hash_map`,
  and `anymap::hashbrown::Map` being powered by `hashbrown::hash_map`.
  The `raw_hash_map` alias, provided in 1.0.0-beta.1 because of the ambiguity
  of what backed `anymap::Map`, is removed as superfluous and useless.
  `RawMap` remains, despite not being *required*, as an ergonomic improvement.
  With this, we’re back to proper completely additive Cargo features.

# 1.0.0-beta.1 (2022-01-25)

- Removed `anymap::any::Any` in favour of just plain `core::any::Any`, since its
  `Send`/`Sync` story is now long stable.

  - This loses `Any + Sync`. `CloneAny + Sync` is also removed for consistency.
    (So `Any + Sync` is gone, but `Any`, `Any + Send` and `Any + Send + Sync`
    remain, plus the same set for `CloneAny`.)

- `anymap::any::CloneAny` moved to `anymap::CloneAny`.
  With nothing public left in `anymap::any`, it is removed.

- Relicensed from MIT/Apache-2.0 to BlueOak-1.0.0/MIT/Apache-2.0.

- Increased the minimum supported version of Rust from 1.7.0 to 1.36.0.

- no_std is now possible in the usual way (default Cargo feature 'std'),
  depending on alloc and hashbrown.

- Removed the `bench` Cargo feature which was mostly to work around historical
  Cargo limitations, but was solved by moving benchmarks from `src/lib.rs` to
  `benches/bench.rs` even before those limitations were lifted. The benchmarks
  still won’t run on anything but nightly, but that don’t signify.

- Implemented `Default` on `Map` (not just on `RawMap`).

- Added `Entry::{or_default, and_modify}` (std::collections::hash_map parity).

- Removed the `anymap::raw` wrapper layer around `std::collections::hash_map`,
  in favour of exposing the raw `HashMap` directly. I think there was a reason
  I did it that seven years ago, but I think that reason may have dissolved by
  now, and I can’t think of it and I don’t like the particular safe
  `as_mut`/unsafe insert approach that I used. Because of the hashbrown stuff,
  I have retained `anymap::RawMap` is an alias, and `anymap::raw_hash_map` too.
  The end result of this is that raw access can finally access things that have
  stabilised since Rust 1.7.0, and we’ll no longer need to play catch-up.

- Worked around the spurious `where_clauses_object_safety` future-compatibility lint that has been raised since mid-2018.
  If you put `#![allow(where_clauses_object_safety)]` on your binary crates for this reason, you can remove it.

# 0.12.1 (2017-01-20)

- Remove superfluous Clone bound on Entry methods (#26)
- Consistent application of `#[inline]` where it should be
- Fix bad performance (see 724f94758def9f71ad27ff49e47e908a431c2728 for details)

# 0.12.0 (2016-03-05)

- Ungate `drain` iterator (stable from Rust 1.6.0)
- Ungate efficient hashing (stable from Rust 1.7.0)
- Remove `unstable` Cargo feature (in favour of a `bench` feature for benchmarking)

# 0.11.2 (2016-01-22)

- Rust warning updates only

# 0.11.1 (2015-06-24)

- Unstable Rust compatibility updates

# 0.11.0 (2015-06-10)

- Support concurrent maps (`Send + Sync` bound)
- Rename `nightly` feature to `unstable`
- Implement `Debug` for `Map` and `RawMap`
- Replace `clone` Cargo feature with arcane DST magicks

# Older releases (from the initial code on 2014-06-12 to 0.10.3 on 2015-04-18)

I’m not giving a changelog for these artefacts of ancient history.
If you really care you can look through the Git history easily enough.
Most of the releases were just compensating for changes to the language
(that being before Rust 1.0; yes, this crate has been around for a while).

I do think that [`src/lib.rs` in the first commit] is a work of art,
a thing of great beauty worth looking at; its simplicity is delightful,
and it doesn’t even need to contain any unsafe code.

[`src/lib.rs` in the first commit]: https://github.com/chris-morgan/anymap/tree/a294948f57dee47bb284d6a3ae1b8f61a902a03c/src/lib.rs
