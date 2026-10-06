//! Provided: just enough HTTP to be dangerous. You don't need to edit this.

use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Delete,
}

/// Everything about a request except the body. Extractors that only need
/// these can run in any argument position, because they don't consume anything.
#[derive(Debug, Default, Clone)]
pub struct Parts {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    /// Keys are lowercase.
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct Request {
    pub parts: Parts,
    pub body: String,
}

impl Request {
    pub fn new(method: Method, path: &str) -> Self {
        let parts = Parts {
            method,
            path: path.to_owned(),
            ..Parts::default()
        };
        Request {
            parts,
            body: String::new(),
        }
    }

    pub fn get(path: &str) -> Self {
        Self::new(Method::Get, path)
    }

    pub fn post(path: &str) -> Self {
        Self::new(Method::Post, path)
    }

    pub fn with_query(mut self, key: &str, value: &str) -> Self {
        self.parts.query.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.parts
            .headers
            .insert(key.to_ascii_lowercase(), value.to_owned());
        self
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body = body.to_owned();
        self
    }

    pub fn into_parts(self) -> (Parts, String) {
        (self.parts, self.body)
    }

    pub fn from_parts(parts: Parts, body: String) -> Self {
        Request { parts, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Anything a handler can return.
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response {
            status: 200,
            body: self,
        }
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        self.to_owned().into_response()
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        String::new().into_response()
    }
}

/// `(status, body)`
impl<T: IntoResponse> IntoResponse for (u16, T) {
    fn into_response(self) -> Response {
        let mut res = self.1.into_response();
        res.status = self.0;
        res
    }
}

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

/// For extractors that can't fail.
impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        match self {}
    }
}
