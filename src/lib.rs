//! This crate provides the `AnyMap` type, a safe and convenient store for one value of each type.

#![feature(core, std_misc)]
#![cfg_attr(test, feature(test))]
#![warn(missing_docs, unused_results)]

#[cfg(test)]
extern crate test;

use std::any::TypeId;
use std::marker::PhantomData;

use raw::{RawAnyMap, Any};
use unchecked_any::UncheckedAnyExt;

macro_rules! impl_common_methods {
    (
        field: $t:ident.$field:ident;
        new() => $new:expr;
        with_capacity($with_capacity_arg:ident) => $with_capacity:expr;
    ) => {
        impl $t {
            /// Create an empty collection.
            #[inline]
            pub fn new() -> $t {
                $t {
                    $field: $new,
                }
            }

            /// Creates an empty collection with the given initial capacity.
            #[inline]
            pub fn with_capacity($with_capacity_arg: usize) -> $t {
                $t {
                    $field: $with_capacity,
                }
            }

            /// Returns the number of elements the collection can hold without reallocating.
            #[inline]
            pub fn capacity(&self) -> usize {
                self.$field.capacity()
            }

            /// Reserves capacity for at least `additional` more elements to be inserted
            /// in the collection. The collection may reserve more space to avoid
            /// frequent reallocations.
            ///
            /// # Panics
            ///
            /// Panics if the new allocation size overflows `usize`.
            #[inline]
            pub fn reserve(&mut self, additional: usize) {
                self.$field.reserve(additional)
            }

            /// Shrinks the capacity of the collection as much as possible. It will drop
            /// down as much as possible while maintaining the internal rules
            /// and possibly leaving some space in accordance with the resize policy.
            #[inline]
            pub fn shrink_to_fit(&mut self) {
                self.$field.shrink_to_fit()
            }

            /// Returns the number of items in the collection.
            #[inline]
            pub fn len(&self) -> usize {
                self.$field.len()
            }

            /// Returns true if there are no items in the collection.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.$field.is_empty()
            }

            /// Removes all items from the collection. Keeps the allocated memory for reuse.
            #[inline]
            pub fn clear(&mut self) {
                self.$field.clear()
            }
        }
    }
}

mod unchecked_any;
pub mod raw;
#[cfg(feature = "clone")]
mod with_clone;

/// A collection containing zero or one values for any given type and allowing convenient,
/// type-safe access to those values.
///
/// ```rust
/// # use anymap::AnyMap;
/// let mut data = AnyMap::new();
/// assert_eq!(data.get(), None::<&i32>);
/// data.insert(42i32);
/// assert_eq!(data.get(), Some(&42i32));
/// data.remove::<i32>();
/// assert_eq!(data.get::<i32>(), None);
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Foo {
///     str: String,
/// }
///
/// assert_eq!(data.get::<Foo>(), None);
/// data.insert(Foo { str: format!("foo") });
/// assert_eq!(data.get(), Some(&Foo { str: format!("foo") }));
/// data.get_mut::<Foo>().map(|foo| foo.str.push('t'));
/// assert_eq!(&*data.get::<Foo>().unwrap().str, "foot");
/// ```
///
/// Values containing non-static references are not permitted.
#[derive(Debug)]
#[cfg_attr(feature = "clone", derive(Clone))]
pub struct AnyMap {
    raw: RawAnyMap,
}

impl_common_methods! {
    field: AnyMap.raw;
    new() => RawAnyMap::new();
    with_capacity(capacity) => RawAnyMap::with_capacity(capacity);
}

impl AnyMap {
    /// Returns a reference to the value stored in the collection for the type `T`, if it exists.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.raw.get(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_ref_unchecked::<T>() })
    }

    /// Returns a mutable reference to the value stored in the collection for the type `T`,
    /// if it exists.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.raw.get_mut(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_mut_unchecked::<T>() })
    }

    /// Sets the value stored in the collection for the type `T`.
    /// If the collection already had a value of type `T`, that value is returned.
    /// Otherwise, `None` is returned.
    pub fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        unsafe {
            self.raw.insert(TypeId::of::<T>(), Box::new(value))
                .map(|any| *any.downcast_unchecked::<T>())
        }
    }

    /// Removes the `T` value from the collection,
    /// returning it if there was one or `None` if there was not.
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.raw.remove(&TypeId::of::<T>())
            .map(|any| *unsafe { any.downcast_unchecked::<T>() })
    }

    /// Returns true if the collection contains a value of type `T`.
    #[inline]
    pub fn contains<T: Any>(&self) -> bool {
        self.raw.contains_key(&TypeId::of::<T>())
    }

    /// Gets the entry for the given type in the collection for in-place manipulation
    pub fn entry<T: Any>(&mut self) -> Entry<T> {
        match self.raw.entry(TypeId::of::<T>()) {
            raw::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                inner: e,
                type_: PhantomData,
            }),
            raw::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                inner: e,
                type_: PhantomData,
            }),
        }
    }
}

impl AsRef<RawAnyMap> for AnyMap {
    fn as_ref(&self) -> &RawAnyMap {
        &self.raw
    }
}

impl AsMut<RawAnyMap> for AnyMap {
    fn as_mut(&mut self) -> &mut RawAnyMap {
        &mut self.raw
    }
}

impl Into<RawAnyMap> for AnyMap {
    fn into(self) -> RawAnyMap {
        self.raw
    }
}

/// A view into a single occupied location in an `AnyMap`.
pub struct OccupiedEntry<'a, V: 'a> {
    inner: raw::OccupiedEntry<'a>,
    type_: PhantomData<V>,
}

/// A view into a single empty location in an `AnyMap`.
pub struct VacantEntry<'a, V: 'a> {
    inner: raw::VacantEntry<'a>,
    type_: PhantomData<V>,
}

/// A view into a single location in an `AnyMap`, which may be vacant or occupied.
pub enum Entry<'a, V: 'a> {
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, V>),
    /// A vacant Entry
    Vacant(VacantEntry<'a, V>),
}

impl<'a, V: Any + Clone> Entry<'a, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default()),
        }
    }
}

impl<'a, V: Any> OccupiedEntry<'a, V> {
    /// Gets a reference to the value in the entry
    pub fn get(&self) -> &V {
        unsafe { self.inner.get().downcast_ref_unchecked() }
    }

    /// Gets a mutable reference to the value in the entry
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.inner.get_mut().downcast_mut_unchecked() }
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the collection itself
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.inner.into_mut().downcast_mut_unchecked() }
    }

    /// Sets the value of the entry, and returns the entry's old value
    pub fn insert(&mut self, value: V) -> V {
        unsafe { *self.inner.insert(Box::new(value)).downcast_unchecked() }
    }

    /// Takes the value out of the entry, and returns it
    pub fn remove(self) -> V {
        unsafe { *self.inner.remove().downcast_unchecked() }
    }
}

impl<'a, V: Any> VacantEntry<'a, V> {
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self.inner.insert(Box::new(value)).downcast_mut_unchecked() }
    }
}

#[bench]
fn bench_insertion(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        for _ in 0..100 {
            let _ = data.insert(42);
        }
    })
}

#[bench]
fn bench_get_missing(b: &mut ::test::Bencher) {
    b.iter(|| {
        let data = AnyMap::new();
        for _ in 0..100 {
            assert_eq!(data.get(), None::<&i32>);
        }
    })
}

#[bench]
fn bench_get_present(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        let _ = data.insert(42);
        // These inner loops are a feeble attempt to drown the other factors.
        for _ in 0..100 {
            assert_eq!(data.get(), Some(&42));
        }
    })
}

#[cfg(test)]
mod tests {
    use {AnyMap, Entry};

    #[derive(Clone, Debug, PartialEq)] struct A(i32);
    #[derive(Clone, Debug, PartialEq)] struct B(i32);
    #[derive(Clone, Debug, PartialEq)] struct C(i32);
    #[derive(Clone, Debug, PartialEq)] struct D(i32);
    #[derive(Clone, Debug, PartialEq)] struct E(i32);
    #[derive(Clone, Debug, PartialEq)] struct F(i32);
    #[derive(Clone, Debug, PartialEq)] struct J(i32);

    #[test]
    fn test_entry() {
        let mut map: AnyMap = AnyMap::new();
        assert_eq!(map.insert(A(10)), None);
        assert_eq!(map.insert(B(20)), None);
        assert_eq!(map.insert(C(30)), None);
        assert_eq!(map.insert(D(40)), None);
        assert_eq!(map.insert(E(50)), None);
        assert_eq!(map.insert(F(60)), None);

        // Existing key (insert)
        match map.entry::<A>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                assert_eq!(view.get(), &A(10));
                assert_eq!(view.insert(A(100)), A(10));
            }
        }
        assert_eq!(map.get::<A>().unwrap(), &A(100));
        assert_eq!(map.len(), 6);


        // Existing key (update)
        match map.entry::<B>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut view) => {
                let v = view.get_mut();
                let new_v = B(v.0 * 10);
                *v = new_v;
            }
        }

        {
            let b: &B = map.get().unwrap();
            assert_eq!(b, &B(200));
        }

        assert_eq!(map.len(), 6);

        // Existing key (remove)
        match map.entry::<C>() {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(view) => {
                assert_eq!(view.remove(), C(30));
            }
        }
        assert_eq!(map.get::<C>(), None);
        assert_eq!(map.len(), 5);


        // Inexistent key (insert)
        match map.entry::<J>() {
            Entry::Occupied(_) => unreachable!(),
            Entry::Vacant(view) => {
                assert_eq!(*view.insert(J(1000)), J(1000));
            }
        }
        assert_eq!(map.get::<J>().unwrap(), &J(1000));
        assert_eq!(map.len(), 6);

        // Entry.or_insert on existing key
        map.entry::<B>().or_insert(B(71)).0 += 1;
        assert_eq!(map.get::<B>().unwrap(), &B(201));
        assert_eq!(map.len(), 6);

        // Entry.or_insert on nonexisting key
        map.entry::<C>().or_insert(C(300)).0 += 1;
        assert_eq!(map.get::<C>().unwrap(), &C(301));
        assert_eq!(map.len(), 7);
    }

    #[cfg(feature = "clone")]
    #[test]
    fn test_clone() {
        let mut map = AnyMap::new();
        let _ = map.insert(A(1));
        let _ = map.insert(B(2));
        let _ = map.insert(D(3));
        let _ = map.insert(E(4));
        let _ = map.insert(F(5));
        let _ = map.insert(J(6));
        let map2 = map.clone();
        assert_eq!(map2.len(), 6);
        assert_eq!(map2.get::<A>(), Some(&A(1)));
        assert_eq!(map2.get::<B>(), Some(&B(2)));
        assert_eq!(map2.get::<C>(), None);
        assert_eq!(map2.get::<D>(), Some(&D(3)));
        assert_eq!(map2.get::<E>(), Some(&E(4)));
        assert_eq!(map2.get::<F>(), Some(&F(5)));
        assert_eq!(map2.get::<J>(), Some(&J(6)));
    }
}
