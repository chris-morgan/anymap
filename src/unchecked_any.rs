use raw::Any;
use std::mem;
use std::raw::TraitObject;

#[allow(missing_docs)]  // Bogus warning (it’s not public outside the crate), ☹
pub trait UncheckedAnyExt {
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T;
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T;
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T>;
}

impl UncheckedAnyExt for Any {
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
        mem::transmute(mem::transmute::<_, TraitObject>(self).data)
    }

    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        mem::transmute(mem::transmute::<_, TraitObject>(self).data)
    }

    unsafe fn downcast_unchecked<T: 'static>(self: Box<Any>) -> Box<T> {
        mem::transmute(mem::transmute::<_, TraitObject>(self).data)
    }
}
