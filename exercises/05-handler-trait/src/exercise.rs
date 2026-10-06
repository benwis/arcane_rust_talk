#[allow(unused_imports)]
use crate::extract::{FromRequest, FromRequestParts, ViaParts, ViaRequest};
#[allow(unused_imports)]
use crate::http::{IntoResponse, Parts, Request, Response};

// ---------------------------------------------------------------------------
// Task 1: let `FromRequestParts` extractors appear in the last position.
//
// Write a blanket impl:
//
//     impl<T: FromRequestParts> FromRequest<ViaParts> for T { .. }
//
// Split the request with `req.into_parts()` and delegate.
//
// Why the marker? Try it without: drop `M` and add a generic wrapper that
// implements both traits, like axum-core does for `Option<T>`/`Result<T, _>`:
//
//     impl<T: FromRequestParts> FromRequestParts for Option<T> { .. }
//     impl<T: FromRequest>      FromRequest      for Option<T> { .. }
//
// `Option<T>` now matches two `FromRequest` impls: E0119.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Task 2: the `Handler` trait (given), implemented for zero-argument functions.
//
// `T` is never used in the trait body. It's there because an impl like
//
//     impl<F, A> Handler for F where F: FnOnce(A) -> R
//
// is rejected: `A` isn't constrained by the trait or the self type (E0207).
// Moving the argument types into a trait parameter fixes that.
//
// Implement `Handler<((),)>` for any `F: FnOnce() -> R + Clone + 'static`
// where `R: IntoResponse`.
// ---------------------------------------------------------------------------

/// Bonus: decorate this with `#[diagnostic::on_unimplemented(...)]` so a bad
/// handler produces a helpful error. Try passing `fn bad(x: u32) {}` to
/// `route` before and after.
pub trait Handler<T>: Clone + 'static {
    fn call(self, req: Request) -> Response;
}

// ---------------------------------------------------------------------------
// Task 3: functions with 1 to 4 arguments, via `macro_rules!`.
//
// For arguments `T1, …, Tn-1, Tn`:
//   * `T1 … Tn-1` must be `FromRequestParts`
//   * `Tn` must be `FromRequest<M>`, for some marker `M`
//   * the impl is `Handler<(M, T1, …, Tn,)>`, because `M` has to be
//     constrained too
//
// Inside `call`: split the request, run each parts extractor (returning the
// rejection as a response on error), rebuild the request, run the last
// extractor, call `self`.
//
// Suggested macro shape:
//
//     macro_rules! impl_handler {
//         ( $($ty:ident),* ; $last:ident ) => { .. };
//     }
//     impl_handler!(; T1);
//     impl_handler!(T1; T2);
//     impl_handler!(T1, T2; T3);
//     impl_handler!(T1, T2, T3; T4);
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Task 4: a fallible extractor.
//
// `Bearer(String)` reads the `authorization` header, which must look like
// `Bearer <token>`. If it's missing or malformed, reject with
// `(401, "missing bearer token")`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Bearer(pub String);
