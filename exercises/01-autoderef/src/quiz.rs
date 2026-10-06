//! Don't edit this file. Read it, predict what each `qN` returns, then write
//! your answers into `src/exercise.rs`.
//!
//! The rule to keep in mind: for `receiver.method()`, the compiler builds a
//! list of candidate receiver types by repeatedly dereferencing:
//! `T`, `*T`, `**T`, … (with one final unsizing step for arrays). For **each**
//! candidate `U` in order, it tries `U`, then `&U`, then `&mut U`. The first
//! receiver type with a matching method wins. Inherent methods beat trait
//! methods only when both match at the *same* step.

use std::any::type_name_of_val;
use std::ops::Deref;

#[derive(Clone, Copy)]
pub struct Foo;

impl Foo {
    pub fn name(&self) -> &'static str {
        "Foo (inherent)"
    }
}

pub trait Named {
    fn name(&self) -> &'static str;
}

impl Named for Foo {
    fn name(&self) -> &'static str {
        "Foo (trait)"
    }
}

/// Note: `who` takes `self` **by value**.
pub trait Who {
    fn who(self) -> &'static str;
}

impl Who for Foo {
    fn who(self) -> &'static str {
        "Foo"
    }
}

impl Who for &Foo {
    fn who(self) -> &'static str {
        "&Foo"
    }
}

pub struct Smart(pub Foo);

impl Deref for Smart {
    type Target = Foo;
    fn deref(&self) -> &Foo {
        &self.0
    }
}

impl Named for Smart {
    fn name(&self) -> &'static str {
        "Smart (trait)"
    }
}

/// Inherent or trait? Options: `"Foo (inherent)"`, `"Foo (trait)"`
pub fn q1() -> &'static str {
    Foo.name()
}

/// Options: `"Foo"`, `"&Foo"`
pub fn q2() -> &'static str {
    let x = Foo;
    let r = &&x;
    r.who()
}

/// `Smart` doesn't implement `Who` at all. Options: `"Foo"`, `"&Foo"`
pub fn q3() -> &'static str {
    Smart(Foo).who()
}

/// `Smart` derefs to `Foo`, which has an inherent `name`.
/// Options: `"Foo (inherent)"`, `"Foo (trait)"`, `"Smart (trait)"`
pub fn q4() -> &'static str {
    Smart(Foo).name()
}

/// What is the type of `y`? Options: `"T"`, `"&T"`
#[allow(noop_method_call)]
pub fn q5<T>(x: &T) -> &'static str {
    let y = x.clone();
    if type_name_of_val(&y).starts_with('&') { "&T" } else { "T" }
}

/// Same as `q5`, but now `T: Clone`. Options: `"T"`, `"&T"`
pub fn q6<T: Clone>(x: &T) -> &'static str {
    let y = x.clone();
    if type_name_of_val(&y).starts_with('&') { "&T" } else { "T" }
}
