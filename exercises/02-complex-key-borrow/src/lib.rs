//! # Exercise 02: implementing `Borrow` for complex keys
//!
//! Adapted from Rain's literate example,
//! <https://github.com/sunshowers-code/borrow-complex-key-example> (CC0),
//! which was in turn inspired by Ivan Dubrov's "Tricking the HashMap"
//! (<http://idubrov.name/rust/2018/06/01/tricking-the-hashmap.html>).
//!
//! Given an owned key and its borrowed twin:
//!
//! ```ignore
//! struct OwnedKey         { s: String,   bytes: Vec<u8>   }
//! struct BorrowedKey<'a>  { s: &'a str,  bytes: &'a [u8]  }
//! ```
//!
//! how do you look up a `HashSet<OwnedKey>` or `BTreeSet<OwnedKey>` with a
//! `BorrowedKey`, without allocating a whole new `OwnedKey` just to compare
//! against?
//!
//! Work through `src/exercise.rs`. The tests won't compile until the `Borrow`
//! impl and the trait-object impls exist. That compile error is your first
//! failing test.
//!
//! Run `cargo test -p complex-key-borrow` to check your work.

#[cfg(not(feature = "solution"))]
mod exercise;
#[cfg(not(feature = "solution"))]
pub use exercise::*;

#[cfg(feature = "solution")]
mod solution;
#[cfg(feature = "solution")]
pub use solution::*;
