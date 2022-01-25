//! The different types of `Any` for use in a map.
//!
//! This stuff is all based on `std::any`, but goes a little further, with `CloneAny` being a
//! cloneable `Any` and with the `Send` and `Sync` bounds possible on both `Any` and `CloneAny`.

use std::fmt;
use std::any::Any as StdAny;

#[doc(hidden)]
pub trait CloneToAny {
    /// Clone `self` into a new `Box<dyn CloneAny>` object.
    fn clone_to_any(&self) -> Box<dyn CloneAny>;
}

impl<T: Any + Clone> CloneToAny for T {
    #[inline]
    fn clone_to_any(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}

macro_rules! impl_clone {
    ($t:ty) => {
        impl Clone for Box<$t> {
            #[inline]
            fn clone(&self) -> Box<$t> {
                // SAFETY: this dance is to reapply any Send/Sync marker. I’m not happy about this
                // approach, given that I used to do it in safe code, but then came a dodgy
                // future-compatibility warning where_clauses_object_safety, which is spurious for
                // auto traits but still super annoying (future-compatibility lints seem to mean
                // your bin crate needs a corresponding allow!). Although I explained my plight¹
                // and it was all explained and agreed upon, no action has been taken. So I finally
                // caved and worked around it by doing it this way, which matches what’s done for
                // std::any², so it’s probably not *too* bad.
                //
                // ¹ https://github.com/rust-lang/rust/issues/51443#issuecomment-421988013
                // ² https://github.com/rust-lang/rust/blob/e7825f2b690c9a0d21b6f6d84c404bb53b151b38/library/alloc/src/boxed.rs#L1613-L1616
                let clone: Box<dyn CloneAny> = (**self).clone_to_any();
                let raw: *mut dyn CloneAny = Box::into_raw(clone);
                unsafe { Box::from_raw(raw as *mut $t) }
            }
        }
    }
}

#[allow(missing_docs)]  // Bogus warning (it’s not public outside the crate), ☹
pub trait UncheckedAnyExt: Any {
    unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T;
    unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T;
    unsafe fn downcast_unchecked<T: Any>(self: Box<Self>) -> Box<T>;
}

#[doc(hidden)]
/// A trait for the conversion of an object into a boxed trait object.
pub trait IntoBox<A: ?Sized + UncheckedAnyExt>: Any {
    /// Convert self into the appropriate boxed form.
    fn into_box(self) -> Box<A>;
}

macro_rules! implement {
    ($base:ident, $(+ $bounds:ident)*) => {
        impl fmt::Debug for dyn $base $(+ $bounds)* {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(stringify!(dyn $base $(+ $bounds)*))
            }
        }

        impl UncheckedAnyExt for dyn $base $(+ $bounds)* {
            #[inline]
            unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
                &*(self as *const Self as *const T)
            }

            #[inline]
            unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
                &mut *(self as *mut Self as *mut T)
            }

            #[inline]
            unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T> {
                Box::from_raw(Box::into_raw(self) as *mut T)
            }
        }

        impl<T: $base $(+ $bounds)*> IntoBox<dyn $base $(+ $bounds)*> for T {
            #[inline]
            fn into_box(self) -> Box<dyn $base $(+ $bounds)*> {
                Box::new(self)
            }
        }
    }
}

/// A type to emulate dynamic typing.
///
/// Every type with no non-`'static` references implements `Any`.
/// See the [`std::any` documentation](https://doc.rust-lang.org/std/any/index.html) for
/// more details on `Any` in general.
///
/// This trait is not `std::any::Any` but rather a type extending that for this library’s
/// purposes so that it can be combined with marker traits like
/// <code><a class=trait title=core::marker::Send
/// href=http://doc.rust-lang.org/std/marker/trait.Send.html>Send</a></code> and
/// <code><a class=trait title=core::marker::Sync
/// href=http://doc.rust-lang.org/std/marker/trait.Sync.html>Sync</a></code>.
///
/// See also [`CloneAny`](trait.CloneAny.html) for a cloneable version of this trait.
pub trait Any: StdAny { }
impl<T: StdAny> Any for T { }
implement!(Any,);
implement!(Any, + Send);
implement!(Any, + Sync);
implement!(Any, + Send + Sync);

/// A type to emulate dynamic typing with cloning.
///
/// Every type with no non-`'static` references that implements `Clone` implements `Any`.
/// See the [`std::any` documentation](https://doc.rust-lang.org/std/any/index.html) for
/// more details on `Any` in general.
///
/// This trait is not `std::any::Any` but rather a type extending that for this library’s
/// purposes so that it can be combined with marker traits like
/// <code><a class=trait title=core::marker::Send
/// href=http://doc.rust-lang.org/std/marker/trait.Send.html>Send</a></code> and
/// <code><a class=trait title=core::marker::Sync
/// href=http://doc.rust-lang.org/std/marker/trait.Sync.html>Sync</a></code>.
///
/// See also [`Any`](trait.Any.html) for a version without the `Clone` requirement.
pub trait CloneAny: Any + CloneToAny { }
impl<T: StdAny + Clone> CloneAny for T { }
implement!(CloneAny,);
implement!(CloneAny, + Send);
implement!(CloneAny, + Sync);
implement!(CloneAny, + Send + Sync);
impl_clone!(dyn CloneAny);
impl_clone!(dyn CloneAny + Send);
impl_clone!(dyn CloneAny + Sync);
impl_clone!(dyn CloneAny + Send + Sync);
