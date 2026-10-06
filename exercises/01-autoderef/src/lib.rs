//! # Exercise 01: autoderef, autoref and the method probe
//!
//! Two parts:
//!
//! 1. **The quiz.** Read `src/quiz.rs`, predict what each `qN()` returns, and
//!    write your predictions into `src/exercise.rs`. No running the code first.
//! 2. **Autoref specialization.** Build a `describe!(expr)` macro that picks
//!    the "best" formatting for a value: `Display` if it has one, otherwise
//!    `Debug`, otherwise a fallback, all on stable Rust with no specialization.
//!
//! Run `cargo test -p autoderef` to check your work.

pub mod quiz;

#[cfg(not(feature = "solution"))]
mod exercise;
#[cfg(not(feature = "solution"))]
pub use exercise::*;

#[cfg(feature = "solution")]
mod solution;
#[cfg(feature = "solution")]
pub use solution::*;
