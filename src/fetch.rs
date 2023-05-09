use core::any::{Any, TypeId};
use std::collections::HashSet;

use crate::{
    any::{Downcast, IntoBox},
    Map,
};

/// A container for a [`Map`] that allows for multiple mutable access of unique values.
pub struct Fetcher<'a, A: ?Sized + Downcast = dyn Any> {
    mask: HashSet<TypeId>,
    map: &'a mut Map<A>,
}

impl<'a, A: ?Sized + Downcast> Fetcher<'a, A> {
    /// Returns a new Fetcher that retrieves values from `map`.
    pub fn new(map: &'a mut Map<A>) -> Self {
        Self {
            mask: Default::default(),
            map,
        }
    }

    /// Fetches and returns a mutable reference to the value
    /// stored in the collection for the type `T`, if it exists.
    ///
    /// This method may be used multiple times while the reference is still alive.
    /// However, another request to the same type will result in `None`.
    pub fn fetch<T: IntoBox<A>>(&mut self) -> Option<&'a mut T> {
        // first insert id into mask, and check if it was there already
        if !self.mask.insert(TypeId::of::<T>()) {
            return None;
        }

        // then retrieve the value
        let value = self.map.get_mut::<T>()?;

        // SAFETY: Transmuting here only changes the lifetime of the value.
        // While this would normally be unsafe, the mask ensures that the user
        // is never able to mutably access the same data twice.
        Some(unsafe { std::mem::transmute(value) })
    }
}
