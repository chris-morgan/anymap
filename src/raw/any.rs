use std::fmt;
use std::any::Any as StdAny;

#[cfg(feature = "clone")]
#[doc(hidden)]
pub trait CloneToAny {
    /// Clone `self` into a new `Box<Any>` object.
    fn clone_to_any(&self) -> Box<Any>;
}

#[cfg(feature = "clone")]
impl<T: Any + Clone> CloneToAny for T {
    fn clone_to_any(&self) -> Box<Any> {
        Box::new(self.clone())
    }
}

macro_rules! define_any {
    (#[$m:meta] $t:item $i:item) => {
        /// A type to emulate dynamic typing.
        ///
        /// Every suitable type with no non-`'static` references implements `Any`. See the
        /// [`std::any` documentation](https://doc.rust-lang.org/std/any/index.html) for more
        /// details on `Any` in general.
        ///
        /// This trait is not `std::any::Any` but rather a type extending that for this library’s
        /// purposes; most specifically, there are a couple of Cargo features that can be enabled
        /// which will alter the constraints of what comprises a suitable type:
        ///
        /// <table>
        ///     <thead>
        ///         <tr>
        ///             <th title="The name of the Cargo feature to enable">Feature name</th>
        ///             <th title="If a type doesn’t satisfy these bounds, it won’t implement Any">Additional bounds</th>
        ///             <th title="Were these docs built with this feature enabled?">Enabled in these docs?</th>
        ///         </tr>
        ///     </thead>
        ///     <tbody>
        ///         <tr>
        ///             <th><code>clone</code></th>
        ///             <td><code><a class=trait title=core::clone::Clone
        ///             href=http://doc.rust-lang.org/std/clone/trait.Clone.html
        ///             >Clone</a></code></td>
        #[cfg_attr(feature = "clone", doc = "             <td>Yes</td>")]
        #[cfg_attr(not(feature = "clone"), doc = "             <td>No</td>")]
        ///         </tr>
        ///         <tr>
        ///             <th><code>concurrent</code></th>
        ///             <td><code><a class=trait title=core::marker::Send
        ///             href=http://doc.rust-lang.org/std/marker/trait.Send.html
        ///             >Send</a> + <a class=trait title=core::marker::Sync
        ///             href=http://doc.rust-lang.org/std/marker/trait.Sync.html
        ///             >Sync</a></code></td>
        #[cfg_attr(feature = "concurrent", doc = "             <td>Yes</td>")]
        #[cfg_attr(not(feature = "concurrent"), doc = "             <td>No</td>")]
        ///         </tr>
        ///     </tbody>
        /// </table>
        #[$m] $t
        #[$m] $i
    }
}

define_any! {
    #[cfg(all(not(feature = "clone"), not(feature = "concurrent")))]
    pub trait Any: StdAny { }
    impl<T: StdAny> Any for T { }
}

define_any! {
    #[cfg(all(feature = "clone", not(feature = "concurrent")))]
    pub trait Any: StdAny + CloneToAny { }
    impl<T: StdAny + Clone> Any for T { }
}

define_any! {
    #[cfg(all(not(feature = "clone"), feature = "concurrent"))]
    pub trait Any: StdAny + Send + Sync { }
    impl<T: StdAny + Send + Sync> Any for T { }
}

define_any! {
    #[cfg(all(feature = "clone", feature = "concurrent"))]
    pub trait Any: StdAny + CloneToAny + Send + Sync { }
    impl<T: StdAny + Clone + Send + Sync> Any for T { }
}

#[cfg(feature = "clone")]
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
