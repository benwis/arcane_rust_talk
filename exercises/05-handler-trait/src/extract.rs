//! Provided: the extractor traits, and a few extractors. You don't need to
//! edit this. You *will* need to add one impl of `FromRequest` in
//! `exercise.rs`.

use crate::http::{IntoResponse, Method, Parts, Request};
use std::collections::HashMap;
use std::convert::Infallible;

/// An extractor that only needs the request's [`Parts`]. Can appear in any
/// argument position.
pub trait FromRequestParts: Sized {
    type Rejection: IntoResponse;

    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection>;
}

/// An extractor that consumes the whole request, body included. Only allowed
/// as the *last* argument.
///
/// The `M` parameter is a tacit marker. Implementors write plain
/// `impl FromRequest for X` and get `M = ViaRequest`. Exercise task 1 is to add
/// a second, non-overlapping blanket impl with `M = ViaParts`, so that every
/// `FromRequestParts` type is also usable in the last position.
pub trait FromRequest<M = ViaRequest>: Sized {
    type Rejection: IntoResponse;

    fn from_request(req: Request) -> Result<Self, Self::Rejection>;
}

// Markers. In axum these live in a private module so nobody can name them.
#[doc(hidden)]
pub enum ViaRequest {}
#[doc(hidden)]
pub enum ViaParts {}

impl FromRequestParts for Method {
    type Rejection = Infallible;

    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection> {
        Ok(parts.method)
    }
}

/// The query string as a map.
#[derive(Debug, Clone)]
pub struct Query(pub HashMap<String, String>);

impl FromRequestParts for Query {
    type Rejection = Infallible;

    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection> {
        Ok(Query(parts.query.clone()))
    }
}

/// All headers as a map, with lowercase keys.
#[derive(Debug, Clone)]
pub struct Headers(pub HashMap<String, String>);

impl FromRequestParts for Headers {
    type Rejection = Infallible;

    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection> {
        Ok(Headers(parts.headers.clone()))
    }
}

/// The body. Consumes the request, so it must come last.
impl FromRequest for String {
    type Rejection = Infallible;

    fn from_request(req: Request) -> Result<Self, Self::Rejection> {
        Ok(req.body)
    }
}
