use handler_trait::{Bearer, Headers, Method, Query, Request, Router};

fn hello() -> &'static str {
    "hello"
}

fn echo(body: String) -> String {
    body
}

fn greet(Query(q): Query) -> String {
    format!("hi {}", q.get("name").map_or("stranger", String::as_str))
}

/// A parts extractor in the last position goes through `FromRequest<ViaParts>`.
fn whoami(method: Method, Bearer(token): Bearer) -> String {
    format!("{method:?} as {token}")
}

fn create(Bearer(token): Bearer, Query(q): Query, body: String) -> (u16, String) {
    let kind = q.get("kind").map_or("thing", String::as_str);
    (201, format!("{token} created {kind}: {body}"))
}

fn four(_: Method, _: Query, Headers(h): Headers, body: String) -> String {
    format!("{} headers, body {body}", h.len())
}

fn divide(Query(q): Query) -> Result<String, (u16, &'static str)> {
    let a: i64 = q.get("a").and_then(|a| a.parse().ok()).ok_or((400, "bad a"))?;
    let b: i64 = q.get("b").and_then(|b| b.parse().ok()).ok_or((400, "bad b"))?;
    if b == 0 {
        return Err((422, "division by zero"));
    }
    Ok((a / b).to_string())
}

fn app() -> Router {
    let prefix = String::from(">> ");
    Router::new()
        .route(Method::Get, "/", hello)
        .route(Method::Post, "/echo", echo)
        .route(Method::Get, "/greet", greet)
        .route(Method::Get, "/whoami", whoami)
        .route(Method::Post, "/things", create)
        .route(Method::Put, "/four", four)
        .route(Method::Get, "/divide", divide)
        .route(Method::Post, "/prefix", move |body: String| {
            format!("{prefix}{body}")
        })
}

#[test]
fn zero_args() {
    let res = app().handle(Request::get("/"));
    assert_eq!((res.status, res.body.as_str()), (200, "hello"));
}

#[test]
fn body_extractor() {
    let res = app().handle(Request::post("/echo").with_body("ping"));
    assert_eq!(res.body, "ping");
}

#[test]
fn query_extractor() {
    let app = app();
    assert_eq!(app.handle(Request::get("/greet")).body, "hi stranger");
    let res = app.handle(Request::get("/greet").with_query("name", "ferris"));
    assert_eq!(res.body, "hi ferris");
}

#[test]
fn parts_extractor_in_last_position() {
    let req = Request::get("/whoami").with_header("Authorization", "Bearer abc");
    assert_eq!(app().handle(req).body, "Get as abc");
}

#[test]
fn rejection_short_circuits() {
    let res = app().handle(Request::get("/whoami"));
    assert_eq!((res.status, res.body.as_str()), (401, "missing bearer token"));

    let req = Request::get("/whoami").with_header("Authorization", "Basic abc");
    assert_eq!(app().handle(req).status, 401);
}

#[test]
fn three_args_and_status() {
    let req = Request::post("/things")
        .with_header("authorization", "Bearer abc")
        .with_query("kind", "widget")
        .with_body("blue");
    let res = app().handle(req);
    assert_eq!((res.status, res.body.as_str()), (201, "abc created widget: blue"));
}

#[test]
fn four_args() {
    let req = Request::new(Method::Put, "/four")
        .with_header("x-a", "1")
        .with_header("x-b", "2")
        .with_body("!");
    assert_eq!(app().handle(req).body, "2 headers, body !");
}

#[test]
fn result_responses() {
    let app = app();
    let ok = app.handle(Request::get("/divide").with_query("a", "9").with_query("b", "3"));
    assert_eq!((ok.status, ok.body.as_str()), (200, "3"));
    let zero = app.handle(Request::get("/divide").with_query("a", "9").with_query("b", "0"));
    assert_eq!(zero.status, 422);
    assert_eq!(app.handle(Request::get("/divide")).status, 400);
}

#[test]
fn closures_are_handlers_too() {
    let res = app().handle(Request::post("/prefix").with_body("hi"));
    assert_eq!(res.body, ">> hi");
}

#[test]
fn not_found_and_wrong_method() {
    let app = app();
    assert_eq!(app.handle(Request::get("/nope")).status, 404);
    assert_eq!(app.handle(Request::get("/echo")).status, 405);
}
