//! # Exercise 06: const assertions
//!
//! Move checks from "the test suite noticed" to "it doesn't compile".
//! Work through `src/exercise.rs`. Several of the tests below are
//! `compile_fail` doctests: they pass only when the code in them is
//! *rejected*, so a no-op stub fails them.
//!
//! Run `cargo test -p const-assertions` to check your work.
//!
//! ## Task 1: `const_assert!`
//!
//! ```
//! use const_assertions::const_assert;
//! const_assert!(core::mem::size_of::<u64>() == 8);
//! const_assert!(u8::MAX as u32 + 1 == 256, "with a message");
//! ```
//!
//! ```compile_fail
//! use const_assertions::const_assert;
//! const_assert!(1 + 1 == 3);
//! ```
//!
//! ## Task 2: `assert_impl!`
//!
//! ```
//! use const_assertions::assert_impl;
//! assert_impl!(String: Send + Sync + Clone);
//! assert_impl!(str: Send);
//! ```
//!
//! ```compile_fail
//! use const_assertions::assert_impl;
//! assert_impl!(std::rc::Rc<u8>: Send);
//! ```
//!
//! ## Task 3: a post-monomorphization check on a const generic
//!
//! ```
//! use const_assertions::RingBuffer;
//! let mut ring = RingBuffer::<u8, 4>::new();
//! ring.push(1);
//! ```
//!
//! ```compile_fail
//! use const_assertions::RingBuffer;
//! let ring = RingBuffer::<u8, 3>::new();
//! ```
//!
//! ```compile_fail
//! use const_assertions::RingBuffer;
//! let ring = RingBuffer::<u8, 0>::new();
//! ```
//!
//! ## Task 4: validated constants
//!
//! ```
//! use const_assertions::hex_color;
//! const CORAL: u32 = hex_color("#ff7f50");
//! assert_eq!(CORAL, 0xff7f50);
//! ```
//!
//! ```compile_fail
//! use const_assertions::hex_color;
//! const TOO_SHORT: u32 = hex_color("#fff");
//! ```
//!
//! ```compile_fail
//! use const_assertions::hex_color;
//! const NOT_HEX: u32 = hex_color("#ff7g50");
//! ```

#[cfg(not(feature = "solution"))]
mod exercise;
#[cfg(not(feature = "solution"))]
pub use exercise::*;

#[cfg(feature = "solution")]
mod solution;
#[cfg(feature = "solution")]
pub use solution::*;
