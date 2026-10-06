use crate::extract::{FromRequest, FromRequestParts, ViaParts};
use crate::http::{IntoResponse, Parts, Request, Response};

// Task 1 -------------------------------------------------------------------

/// Every parts extractor is also a whole-request extractor. Without the
/// marker, this blanket impl would conflict with any type that implements
/// both traits, e.g. a generic wrapper like `Option<T>` given both a
/// `FromRequestParts` and a `FromRequest` impl. With it, the two are
/// `FromRequest<ViaParts>` and `FromRequest<ViaRequest>`: different traits,
/// as far as coherence cares.
impl<T: FromRequestParts> FromRequest<ViaParts> for T {
    type Rejection = T::Rejection;

    fn from_request(req: Request) -> Result<Self, Self::Rejection> {
        let (parts, _body) = req.into_parts();
        T::from_request_parts(&parts)
    }
}

// Task 2 -------------------------------------------------------------------

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid handler",
    label = "not a handler",
    note = "handlers take up to 4 arguments: every argument but the last must implement `FromRequestParts`, the last must implement `FromRequest`",
    note = "the return type must implement `IntoResponse`"
)]
pub trait Handler<T>: Clone + 'static {
    fn call(self, req: Request) -> Response;
}

impl<F, R> Handler<((),)> for F
where
    F: FnOnce() -> R + Clone + 'static,
    R: IntoResponse,
{
    fn call(self, _req: Request) -> Response {
        self().into_response()
    }
}

// Task 3 -------------------------------------------------------------------

macro_rules! impl_handler {
    ( $($ty:ident),* ; $last:ident ) => {
        #[allow(non_snake_case)]
        impl<F, R, M, $($ty,)* $last> Handler<(M, $($ty,)* $last,)> for F
        where
            F: FnOnce($($ty,)* $last) -> R + Clone + 'static,
            R: IntoResponse,
            $( $ty: FromRequestParts, )*
            $last: FromRequest<M>,
        {
            fn call(self, req: Request) -> Response {
                #[allow(unused_variables)]
                let (parts, body) = req.into_parts();
                $(
                    let $ty = match <$ty as FromRequestParts>::from_request_parts(&parts) {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };
                )*
                let req = Request::from_parts(parts, body);
                let $last = match <$last as FromRequest<M>>::from_request(req) {
                    Ok(value) => value,
                    Err(rejection) => return rejection.into_response(),
                };
                self($($ty,)* $last).into_response()
            }
        }
    };
}

impl_handler!(; T1);
impl_handler!(T1; T2);
impl_handler!(T1, T2; T3);
impl_handler!(T1, T2, T3; T4);

// Task 4 -------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Bearer(pub String);

impl FromRequestParts for Bearer {
    type Rejection = (u16, &'static str);

    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get("authorization")
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(|token| Bearer(token.to_owned()))
            .ok_or((401, "missing bearer token"))
    }
}
