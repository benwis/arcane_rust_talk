//! # Exercise 04: tacit trait parameters vs. coherence
//!
//! We want one function, `label`, that accepts:
//!
//! * anything `Display`                  → `x.to_string()`
//! * any closure `FnOnce() -> String`    → call it
//! * any `Iterator` of `Display` items   → items joined with `", "`
//! * `Option<T>` of any of the above     → the inner label, or `"none"`
//!
//! The obvious blanket impls overlap as far as coherence is concerned (nothing
//! stops some type from being both `Display` and `Iterator`), so rustc rejects
//! them with E0119. The fix is a type parameter that callers never write and
//! inference always fills in.
//!
//! The tests won't compile until you've written the impls. That compile error
//! is your first failing test.
//!
//! Run `cargo test -p tacit-params` to check your work.
//!
//! ```compile_fail
//! // Ambiguity is still an error, but it's reported at the *call site*, and
//! // only for the types that actually match more than one impl.
//! use tacit_params::label;
//! use std::fmt;
//!
//! struct Both;
//! impl fmt::Display for Both {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("both") }
//! }
//! impl Iterator for Both {
//!     type Item = u8;
//!     fn next(&mut self) -> Option<u8> { None }
//! }
//!
//! label(Both); // error[E0283]: type annotations needed
//! ```

#[cfg(not(feature = "solution"))]
mod exercise;
#[cfg(not(feature = "solution"))]
pub use exercise::*;

#[cfg(feature = "solution")]
mod solution;
#[cfg(feature = "solution")]
pub use solution::*;
