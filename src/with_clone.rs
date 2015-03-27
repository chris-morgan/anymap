use std::fmt;

#[doc(hidden)]
pub trait CloneToAny {
    /// Clone `self` into a new `Box<Any>` object.
    fn clone_to_any(&self) -> Box<Any>;
}

impl<T: 'static + Clone> CloneToAny for T {
    fn clone_to_any(&self) -> Box<Any> {
        Box::new(self.clone())
    }
}

#[doc(hidden)]
/// Pretty much just `std::any::Any + Clone`.
pub trait Any: ::std::any::Any + CloneToAny { }

impl<T: 'static + Clone> Any for T { }

impl Clone for Box<Any> {
    fn clone(&self) -> Box<Any> {
        (**self).clone_to_any()
    }
}

impl<'a> fmt::Debug for &'a Any {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("&Any")
    }
}

impl<'a> fmt::Debug for Box<Any> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("Box<Any>")
    }
}
