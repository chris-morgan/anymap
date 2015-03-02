//! This crate provides the `AnyMap` type, a safe and convenient store for one value of each type.

#![feature(core, std_misc, hash)]
#![cfg_attr(test, feature(test))]
#![warn(unused_qualifications, non_upper_case_globals,
        variant_size_differences, unused_typecasts,
        missing_docs, unused_results)]

#[cfg(test)]
extern crate test;

use std::any::{Any, TypeId};
use std::mem::forget;
use std::collections::HashMap;
use std::collections::hash_map;
use std::hash::Hasher;
use std::collections::hash_state::HashState;
use std::mem::transmute;
use std::raw::TraitObject;
use std::marker::PhantomData;

struct TypeIdHasher {
    value: u64,
}

struct TypeIdState;

impl HashState for TypeIdState {
    type Hasher = TypeIdHasher;

    fn hasher(&self) -> TypeIdHasher {
        TypeIdHasher { value: 0 }
    }
}

impl Hasher for TypeIdHasher {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive one and exactly one 64-bit value
        debug_assert!(bytes.len() == 8);
        unsafe {
            std::ptr::copy_nonoverlapping(&mut self.value, transmute(&bytes[0]), 1)
        }
    }

    #[inline(always)]
    fn finish(&self) -> u64 { self.value }
}

/// An extension of `AnyRefExt` allowing unchecked downcasting of trait objects to `&T`.
trait UncheckedAnyRefExt<'a> {
    /// Returns a reference to the boxed value, assuming that it is of type `T`. This should only be
    /// called if you are ABSOLUTELY CERTAIN of `T` as you will get really wacky output if it’s not.
    unsafe fn downcast_ref_unchecked<T: 'static>(self) -> &'a T;
}

impl<'a> UncheckedAnyRefExt<'a> for &'a Any {
    #[inline]
    unsafe fn downcast_ref_unchecked<T: 'static>(self) -> &'a T {
        // Get the raw representation of the trait object
        let to: TraitObject = transmute(self);

        // Extract the data pointer
        transmute(to.data)
    }
}

/// An extension of `AnyMutRefExt` allowing unchecked downcasting of trait objects to `&mut T`.
trait UncheckedAnyMutRefExt<'a> {
    /// Returns a reference to the boxed value, assuming that it is of type `T`. This should only be
    /// called if you are ABSOLUTELY CERTAIN of `T` as you will get really wacky output if it’s not.
    unsafe fn downcast_mut_unchecked<T: 'static>(self) -> &'a mut T;
}

impl<'a> UncheckedAnyMutRefExt<'a> for &'a mut Any {
    #[inline]
    unsafe fn downcast_mut_unchecked<T: 'static>(self) -> &'a mut T {
        // Get the raw representation of the trait object
        let to: TraitObject = transmute(self);

        // Extract the data pointer
        transmute(to.data)
    }
}

/// An extension of `BoxAny` allowing unchecked downcasting of trait objects to `Box<T>`.
trait UncheckedBoxAny {
    /// Returns the boxed value, assuming that it is of type `T`. This should only be called if you
    /// are ABSOLUTELY CERTAIN of `T` as you will get really wacky output if it’s not.
    unsafe fn downcast_unchecked<T: 'static>(self) -> Box<T>;
}

impl UncheckedBoxAny for Box<Any + 'static> {
    #[inline]
    unsafe fn downcast_unchecked<T: 'static>(self) -> Box<T> {
        // Get the raw representation of the trait object
        let to: TraitObject = *transmute::<&Box<Any>, &TraitObject>(&self);

        // Prevent destructor on self being run
        forget(self);

        // Extract the data pointer
        transmute(to.data)
    }
}

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
/// #[derive(PartialEq, Debug)]
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
#[stable]
pub struct AnyMap {
    data: HashMap<TypeId, Box<Any + 'static>, TypeIdState>,
}

impl AnyMap {
    /// Construct a new `AnyMap`.
    #[inline]
    #[stable]
    pub fn new() -> AnyMap {
        AnyMap {
            data: HashMap::with_hash_state(TypeIdState),
        }
    }

    /// Creates an empty AnyMap with the given initial capacity.
    #[inline]
    #[stable]
    pub fn with_capcity(capacity: usize) -> AnyMap {
        AnyMap {
            data: HashMap::with_capacity_and_hash_state(capacity, TypeIdState),
        }
    }

    /// Returns the number of elements the collection can hold without reallocating.
    #[inline]
    #[stable]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `AnyMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    #[inline]
    #[stable]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    /// Shrinks the capacity of the collection as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    #[inline]
    #[stable]
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    /// An iterator visiting all items in the collection in arbitrary order.
    /// Iterator element type is `&Any`.
    ///
    /// This is probably not a great deal of use.
    #[inline]
    #[stable]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.data.iter(),
        }
    }

    /// An iterator visiting all items in the collection in arbitrary order.
    /// Iterator element type is `&mut Any`.
    ///
    /// This is probably not a great deal of use.
    #[inline]
    #[stable]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            inner: self.data.iter_mut(),
        }
    }

    /// An iterator visiting all items in the collection in arbitrary order.
    /// Creates a consuming iterator, that is, one that moves each item
    /// out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    ///
    /// Iterator element type is `Box<Any>`.
    #[inline]
    #[stable]
    pub fn into_iter(self) -> IntoIter {
        IntoIter {
            inner: self.data.into_iter(),
        }
    }

    /// Returns a reference to the value stored in the collection for the type `T`, if it exists.
    #[stable]
    pub fn get<T: Any + 'static>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_ref_unchecked::<T>() })
    }

    /// Returns a mutable reference to the value stored in the collection for the type `T`,
    /// if it exists.
    #[stable]
    pub fn get_mut<T: Any + 'static>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())
            .map(|any| unsafe { any.downcast_mut_unchecked::<T>() })
    }

    /// Sets the value stored in the collection for the type `T`.
    /// If the collection already had a value of type `T`, that value is returned.
    /// Otherwise, `None` is returned.
    #[stable]
    pub fn insert<T: Any + 'static>(&mut self, value: T) -> Option<T> {
        self.data.insert(TypeId::of::<T>(), Box::new(value) as Box<Any>)
            .map(|any| *unsafe { any.downcast_unchecked::<T>() })
    }

    /// Removes the `T` value from the collection,
    /// returning it if there was one or `None` if there was not.
    #[stable]
    pub fn remove<T: Any + 'static>(&mut self) -> Option<T> {
        self.data.remove(&TypeId::of::<T>())
            .map(|any| *unsafe { any.downcast_unchecked::<T>() })
    }

    /// Returns true if the collection contains a value of type `T`.
    #[stable]
    pub fn contains<T: Any + 'static>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    /// Gets the entry for the given type in the collection for in-place manipulation
    #[stable]
    pub fn entry<T: Any + 'static>(&mut self) -> Entry<T> {
        match self.data.entry(TypeId::of::<T>()) {
            hash_map::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                entry: e,
                type_: PhantomData,
            }),
            hash_map::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                entry: e,
                type_: PhantomData,
            }),
        }
    }

    /// Returns the number of items in the collection.
    #[inline]
    #[stable]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if there are no items in the collection.
    #[inline]
    #[stable]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears the map, returning all items as an iterator.
    ///
    /// Iterator element type is `Box<Any>`.
    ///
    /// Keeps the allocated memory for reuse.
    #[inline]
    #[unstable = "matches collection reform specification, waiting for dust to settle"]
    pub fn drain(&mut self) -> Drain {
        Drain {
            inner: self.data.drain(),
        }
    }

    /// Removes all items from the collection. Keeps the allocated memory for reuse.
    #[inline]
    #[stable]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// A view into a single occupied location in an AnyMap
#[stable]
pub struct OccupiedEntry<'a, V: 'a> {
    entry: hash_map::OccupiedEntry<'a, TypeId, Box<Any + 'static>>,
    type_: PhantomData<V>,
}

/// A view into a single empty location in an AnyMap
#[stable]
pub struct VacantEntry<'a, V: 'a> {
    entry: hash_map::VacantEntry<'a, TypeId, Box<Any + 'static>>,
    type_: PhantomData<V>,
}

/// A view into a single location in an AnyMap, which may be vacant or occupied
#[stable]
pub enum Entry<'a, V: 'a> {
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, V>),
    /// A vacant Entry
    Vacant(VacantEntry<'a, V>),
}

impl<'a, V: 'static + Clone> Entry<'a, V> {
    #[unstable = "matches collection reform v2 specification, waiting for dust to settle"]
    /// Returns a mutable reference to the entry if occupied, or the VacantEntry if vacant
    pub fn get(self) -> Result<&'a mut V, VacantEntry<'a, V>> {
        match self {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => Err(entry),
        }
    }
}

impl<'a, V: 'static> OccupiedEntry<'a, V> {
    #[stable]
    /// Gets a reference to the value in the entry
    pub fn get(&self) -> &V {
        unsafe { self.entry.get().downcast_ref_unchecked() }
    }

    #[stable]
    /// Gets a mutable reference to the value in the entry
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.entry.get_mut().downcast_mut_unchecked() }
    }

    #[stable]
    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the collection itself
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.entry.into_mut().downcast_mut_unchecked() }
    }

    #[stable]
    /// Sets the value of the entry, and returns the entry's old value
    pub fn insert(&mut self, value: V) -> V {
        unsafe { *self.entry.insert(Box::new(value) as Box<Any + 'static>).downcast_unchecked() }
    }

    #[stable]
    /// Takes the value out of the entry, and returns it
    pub fn remove(self) -> V {
        unsafe { *self.entry.remove().downcast_unchecked() }
    }
}

impl<'a, V: 'static> VacantEntry<'a, V> {
    #[stable]
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it
    pub fn insert(self, value: V) -> &'a mut V {
        unsafe { self.entry.insert(Box::new(value) as Box<Any + 'static>).downcast_mut_unchecked() }
    }
}

/// `AnyMap` iterator.
#[stable]
#[derive(Clone)]
pub struct Iter<'a> {
    inner: hash_map::Iter<'a, TypeId, Box<Any + 'static>>,
}

/// `AnyMap` mutable references iterator.
#[stable]
pub struct IterMut<'a> {
    inner: hash_map::IterMut<'a, TypeId, Box<Any + 'static>>,
}

/// `AnyMap` draining iterator.
#[unstable = "matches collection reform specification, waiting for dust to settle"]
pub struct Drain<'a> {
    inner: hash_map::Drain<'a, TypeId, Box<Any + 'static>>,
}

/// `AnyMap` move iterator.
#[stable]
pub struct IntoIter {
    inner: hash_map::IntoIter<TypeId, Box<Any + 'static>>,
}

#[stable]
impl<'a> Iterator for Iter<'a> {
    type Item = &'a Any;

    #[inline]
    fn next(&mut self) -> Option<&'a Any> {
        self.inner.next().map(|item| &**item.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[stable]
impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Any;

    #[inline]
    fn next(&mut self) -> Option<&'a mut Any> {
        self.inner.next().map(|item| &mut **item.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[stable]
impl<'a> Iterator for Drain<'a> {
    type Item = Box<Any + 'static>;

    #[inline]
    fn next(&mut self) -> Option<Box<Any + 'static>> {
        self.inner.next().map(|item| item.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[stable]
impl Iterator for IntoIter {
    type Item = Box<Any + 'static>;

    #[inline]
    fn next(&mut self) -> Option<Box<Any + 'static>> {
        self.inner.next().map(|item| item.1)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[bench]
fn bench_insertion(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        for _ in range(0, 100) {
            let _ = data.insert(42i32);
        }
    })
}

#[bench]
fn bench_get_missing(b: &mut ::test::Bencher) {
    b.iter(|| {
        let data = AnyMap::new();
        for _ in range(0, 100) {
            assert_eq!(data.get(), None::<&i32>);
        }
    })
}

#[bench]
fn bench_get_present(b: &mut ::test::Bencher) {
    b.iter(|| {
        let mut data = AnyMap::new();
        let _ = data.insert(42i32);
        // These inner loops are a feeble attempt to drown the other factors.
        for _ in range(0, 100) {
            assert_eq!(data.get(), Some(&42i32));
        }
    })
}

#[test]
fn test_entry() {
    #[derive(Debug, PartialEq)] struct A(i32);
    #[derive(Debug, PartialEq)] struct B(i32);
    #[derive(Debug, PartialEq)] struct C(i32);
    #[derive(Debug, PartialEq)] struct D(i32);
    #[derive(Debug, PartialEq)] struct E(i32);
    #[derive(Debug, PartialEq)] struct F(i32);
    #[derive(Debug, PartialEq)] struct J(i32);

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
    assert_eq!(map.get().unwrap(), &B(200));
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
}
