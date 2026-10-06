//! # Exercise 05: the axum handler trick
//!
//! A tiny, synchronous axum. The HTTP types, extractor traits and router are
//! provided. What's missing is the part that lets you write
//!
//! ```ignore
//! fn create(Bearer(token): Bearer, Query(q): Query, body: String) -> (u16, String) { .. }
//!
//! Router::new().route(Method::Post, "/things", create);
//! ```
//!
//! and have the router figure out how to build every argument from a request.
//!
//! Work through `src/exercise.rs`. The tests won't compile until `Handler` is
//! implemented for functions. That compile error is your first failing test.
//!
//! Run `cargo test -p handler-trait` to check your work.

pub mod extract;
pub mod http;
pub mod router;

pub use extract::{FromRequest, FromRequestParts, Headers, Query};
pub use http::{IntoResponse, Method, Parts, Request, Response};
pub use router::Router;

#[cfg(not(feature = "solution"))]
mod exercise;
#[cfg(not(feature = "solution"))]
pub use exercise::*;

#[cfg(feature = "solution")]
mod solution;
#[cfg(feature = "solution")]
pub use solution::*;
