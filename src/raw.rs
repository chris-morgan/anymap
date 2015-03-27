//! The raw form of an AnyMap, allowing untyped access.
//!
//! All relevant details are in the `RawAnyMap` struct.

use std::any::TypeId;
use std::borrow::Borrow;
use std::collections::hash_map::{self, HashMap};
use std::collections::hash_state::HashState;
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::iter::IntoIterator;
use std::mem;
use std::ops::{Index, IndexMut};
use std::ptr;

#[cfg(not(feature = "clone"))]
pub use std::any::Any;
#[cfg(feature = "clone")]
pub use with_clone::Any;

struct TypeIdHasher {
    value: u64,
}

#[cfg_attr(feature = "clone", derive(Clone))]
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
            ptr::copy_nonoverlapping(&mut self.value, mem::transmute(&bytes[0]), 1)
        }
    }

    #[inline(always)]
    fn finish(&self) -> u64 { self.value }
}


/// The raw, underlying form of an AnyMap.
///
/// At its essence, this is a wrapper around `HashMap<TypeId, Box<Any>>`, with the portions that
/// would be memory-unsafe removed or marked unsafe. Normal people are expected to use the safe
/// `AnyMap` interface instead, but there is the occasional use for this such as iteration over the
/// contents of an `AnyMap`. However, because you will then be dealing with `Any` trait objects, it
/// doesn’t tend to be so very useful. Still, if you need it, it’s here.
#[derive(Debug)]
#[cfg_attr(feature = "clone", derive(Clone))]
pub struct RawAnyMap {
    inner: HashMap<TypeId, Box<Any>, TypeIdState>,
}

impl Default for RawAnyMap {
    fn default() -> RawAnyMap {
        RawAnyMap::new()
    }
}

impl_common_methods! {
    field: RawAnyMap.inner;
    new() => HashMap::with_hash_state(TypeIdState);
    with_capacity(capacity) => HashMap::with_capacity_and_hash_state(capacity, TypeIdState);
}

/// RawAnyMap iterator.
#[derive(Clone)]
pub struct Iter<'a> {
    inner: hash_map::Iter<'a, TypeId, Box<Any>>,
}
impl<'a> Iterator for Iter<'a> {
    type Item = &'a Any;
    #[inline] fn next(&mut self) -> Option<&'a Any> { self.inner.next().map(|x| &**x.1) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}
impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline] fn len(&self) -> usize { self.inner.len() }
}

/// RawAnyMap mutable iterator.
pub struct IterMut<'a> {
    inner: hash_map::IterMut<'a, TypeId, Box<Any>>,
}
impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Any;
    #[inline] fn next(&mut self) -> Option<&'a mut Any> { self.inner.next().map(|x| &mut **x.1) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}
impl<'a> ExactSizeIterator for IterMut<'a> {
    #[inline] fn len(&self) -> usize { self.inner.len() }
}

/// RawAnyMap move iterator.
pub struct IntoIter {
    inner: hash_map::IntoIter<TypeId, Box<Any>>,
}
impl Iterator for IntoIter {
    type Item = Box<Any>;
    #[inline] fn next(&mut self) -> Option<Box<Any>> { self.inner.next().map(|x| x.1) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}
impl ExactSizeIterator for IntoIter {
    #[inline] fn len(&self) -> usize { self.inner.len() }
}

/// RawAnyMap drain iterator.
pub struct Drain<'a> {
    inner: hash_map::Drain<'a, TypeId, Box<Any>>,
}
impl<'a> Iterator for Drain<'a> {
    type Item = Box<Any>;
    #[inline] fn next(&mut self) -> Option<Box<Any>> { self.inner.next().map(|x| x.1) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}
impl<'a> ExactSizeIterator for Drain<'a> {
    #[inline] fn len(&self) -> usize { self.inner.len() }
}

impl RawAnyMap {
    /// An iterator visiting all entries in arbitrary order.
    ///
    /// Iterator element type is `&Any`.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.inner.iter(),
        }
    }

    /// An iterator visiting all entries in arbitrary order.
    ///
    /// Iterator element type is `&mut Any`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Creates a consuming iterator, that is, one that moves each item
    /// out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    ///
    /// Iterator element type is `Box<Any>`.
    #[inline]
    pub fn into_iter(self) -> IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }

    /// Clears the map, returning all items as an iterator.
    ///
    /// Iterator element type is `Box<Any>`.
    ///
    /// Keeps the allocated memory for reuse.
    #[inline]
    pub fn drain(&mut self) -> Drain {
        Drain {
            inner: self.inner.drain(),
        }
    }

    /// Gets the entry for the given type in the collection for in-place manipulation.
    pub fn entry(&mut self, key: TypeId) -> Entry {
        match self.inner.entry(key) {
            hash_map::Entry::Occupied(e) => Entry::Occupied(OccupiedEntry {
                inner: e,
            }),
            hash_map::Entry::Vacant(e) => Entry::Vacant(VacantEntry {
                inner: e,
            }),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Any>
    where TypeId: Borrow<Q>, Q: Hash + Eq {
        self.inner.get(k).map(|x| &**x)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where TypeId: Borrow<Q>, Q: Hash + Eq {
        self.inner.contains_key(k)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Any>
    where TypeId: Borrow<Q>, Q: Hash + Eq {
        self.inner.get_mut(k).map(|x| &mut **x)
    }

    /// Inserts a key-value pair from the map. If the key already had a value present in the map,
    /// that value is returned. Otherwise, None is returned.
    ///
    /// It is the caller’s responsibility to ensure that the key corresponds with the type ID of
    /// the value. If they do not, memory safety may be violated.
    pub unsafe fn insert(&mut self, key: TypeId, value: Box<Any>) -> Option<Box<Any>> {
        self.inner.insert(key, value)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map.
    ///
    /// The key may be any borrowed form of the map's key type, but `Hash` and `Eq` on the borrowed
    /// form *must* match those for the key type.
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Box<Any>>
    where TypeId: Borrow<Q>, Q: Hash + Eq {
        self.inner.remove(k)
    }

}

impl<Q> Index<Q> for RawAnyMap where TypeId: Borrow<Q>, Q: Eq + Hash {
    type Output = Any;

    fn index<'a>(&'a self, index: Q) -> &'a Any {
        self.get(&index).expect("no entry found for key")
    }
}

impl<Q> IndexMut<Q> for RawAnyMap where TypeId: Borrow<Q>, Q: Eq + Hash {
    fn index_mut<'a>(&'a mut self, index: Q) -> &'a mut Any {
        self.get_mut(&index).expect("no entry found for key")
    }
}

impl IntoIterator for RawAnyMap {
    type Item = Box<Any>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        self.into_iter()
    }
}

/// A view into a single occupied location in a `RawAnyMap`.
pub struct OccupiedEntry<'a> {
    inner: hash_map::OccupiedEntry<'a, TypeId, Box<Any>>,
}

/// A view into a single empty location in a `RawAnyMap`.
pub struct VacantEntry<'a> {
    inner: hash_map::VacantEntry<'a, TypeId, Box<Any>>,
}

/// A view into a single location in an AnyMap, which may be vacant or occupied.
pub enum Entry<'a> {
    /// An occupied Entry
    Occupied(OccupiedEntry<'a>),
    /// A vacant Entry
    Vacant(VacantEntry<'a>),
}

impl<'a> Entry<'a> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// It is the caller’s responsibility to ensure that the key of the entry corresponds with
    /// the type ID of `value`. If they do not, memory safety may be violated.
    pub unsafe fn or_insert(self, default: Box<Any>) -> &'a mut Any {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// It is the caller’s responsibility to ensure that the key of the entry corresponds with
    /// the type ID of `value`. If they do not, memory safety may be violated.
    pub unsafe fn or_insert_with<F: FnOnce() -> Box<Any>>(self, default: F) -> &'a mut Any {
        match self {
            Entry::Occupied(inner) => inner.into_mut(),
            Entry::Vacant(inner) => inner.insert(default()),
        }
    }
}

impl<'a> OccupiedEntry<'a> {
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &Any {
        &**self.inner.get() 
    }

    /// Gets a mutable reference to the value in the entry.
    pub fn get_mut(&mut self) -> &mut Any {
        &mut **self.inner.get_mut()
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the collection itself.
    pub fn into_mut(self) -> &'a mut Any {
        &mut **self.inner.into_mut()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// It is the caller’s responsibility to ensure that the key of the entry corresponds with
    /// the type ID of `value`. If they do not, memory safety may be violated.
    pub unsafe fn insert(&mut self, value: Box<Any>) -> Box<Any> {
        self.inner.insert(value)
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> Box<Any> {
        self.inner.remove()
    }
}

impl<'a> VacantEntry<'a> {
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it
    ///
    /// It is the caller’s responsibility to ensure that the key of the entry corresponds with
    /// the type ID of `value`. If they do not, memory safety may be violated.
    pub unsafe fn insert(self, value: Box<Any>) -> &'a mut Any {
        &mut **self.inner.insert(value)
    }
}
