//! Provided: a router that stores handlers of *different* types side by side.
//! You don't need to edit this, but read `route`: it's where the generic
//! `Handler<T>` gets erased into a plain boxed closure.

use crate::Handler;
use crate::http::{IntoResponse, Method, Request, Response};
use std::collections::HashMap;

type BoxedHandler = Box<dyn Fn(Request) -> Response>;

#[derive(Default)]
pub struct Router {
    routes: HashMap<String, HashMap<Method, BoxedHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    /// `T` is whatever marker tuple the matching `Handler` impl picked. The
    /// caller never sees it.
    pub fn route<H, T>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
        T: 'static,
    {
        let erased: BoxedHandler = Box::new(move |req| handler.clone().call(req));
        self.routes
            .entry(path.to_owned())
            .or_default()
            .insert(method, erased);
        self
    }

    pub fn handle(&self, req: Request) -> Response {
        let Some(methods) = self.routes.get(&req.parts.path) else {
            return (404, "not found").into_response();
        };
        let Some(handler) = methods.get(&req.parts.method) else {
            return (405, "method not allowed").into_response();
        };
        handler(req)
    }
}
