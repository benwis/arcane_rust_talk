use std::fmt::{Debug, Display};

// ---------------------------------------------------------------------------
// Part 1: the quiz.
// ---------------------------------------------------------------------------

/// Inherent and trait methods both match at the first step (autoref `&Foo`),
/// and inherent wins ties.
pub const Q1: &str = "Foo (inherent)";
/// `&&Foo` has no impl. Neither do `&&&Foo` or `&mut &&Foo`. One deref gives
/// `&Foo`, which has a by-value impl.
pub const Q2: &str = "&Foo";
/// `Smart`, `&Smart` and `&mut Smart` don't match. One deref gives `Foo`,
/// which matches by value (and `Foo: Copy`, so moving out of the deref is ok).
/// `&Foo` would only be tried after that.
pub const Q3: &str = "Foo";
/// `Named::name(&self)` for `Smart` matches at autoref `&Smart`, *before*
/// the probe ever derefs to `Foo`. The outer trait beats the inner inherent.
pub const Q4: &str = "Smart (trait)";
/// The receiver `x` has type `&T`. At the by-value step, `<T as Clone>::clone`
/// (receiver `&T`) would match, but `T: Clone` isn't known, so it's skipped.
/// At the autoref step (`&&T`), `<&T as Clone>::clone` matches, so you get a
/// copy of the reference.
pub const Q5: &str = "&T";
/// With the bound, `<T as Clone>::clone` matches at the very first step: by
/// value on `&T`, which is exactly the `&self` receiver `Clone::clone` wants.
pub const Q6: &str = "T";

// ---------------------------------------------------------------------------
// Part 2: autoref specialization.
// ---------------------------------------------------------------------------

pub struct Wrap<T>(pub T);

pub trait ViaDisplay {
    fn describe(&self) -> String;
}

impl<T: Display> ViaDisplay for &&Wrap<T> {
    fn describe(&self) -> String {
        format!("Display: {}", self.0)
    }
}

pub trait ViaDebug {
    fn describe(&self) -> String;
}

impl<T: Debug> ViaDebug for &Wrap<T> {
    fn describe(&self) -> String {
        format!("Debug: {:?}", self.0)
    }
}

pub trait ViaNothing {
    fn describe(&self) -> String;
}

impl<T> ViaNothing for Wrap<T> {
    fn describe(&self) -> String {
        String::from("<opaque>")
    }
}

#[macro_export]
macro_rules! describe {
    ($e:expr) => {{
        #[allow(unused_imports)]
        use $crate::{ViaDebug as _, ViaDisplay as _, ViaNothing as _};
        (&&&$crate::Wrap(&$e)).describe()
    }};
}
