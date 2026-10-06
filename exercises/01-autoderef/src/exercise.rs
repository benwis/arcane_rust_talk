// ---------------------------------------------------------------------------
// Part 1: the quiz. Replace each "???" with your prediction.
// ---------------------------------------------------------------------------

pub const Q1: &str = "???";
pub const Q2: &str = "???";
pub const Q3: &str = "???";
pub const Q4: &str = "???";
pub const Q5: &str = "???";
pub const Q6: &str = "???";

// ---------------------------------------------------------------------------
// Part 2: autoref specialization.
//
// Make `describe!(expr)` return:
//   * `"Display: {expr}"`   if the type implements `Display`
//   * `"Debug: {expr:?}"`   else if it implements `Debug`
//   * `"<opaque>"`          otherwise
//
// The recipe:
//   1. A wrapper struct, e.g. `pub struct Wrap<T>(pub T);`
//   2. One trait per "tier", each with the same method name, e.g.
//      `fn describe(&self) -> String`.
//   3. Implement the highest priority tier for `&&Wrap<T>`, the next for
//      `&Wrap<T>`, and the fallback for `Wrap<T>`.
//   4. In the macro, call the method on `(&&&Wrap(&$e))`, with all three
//      traits in scope. The method probe strips one `&` per step, and the
//      first impl whose bounds hold wins.
//
// Use `$crate::` paths in the macro so it works from other crates.
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! describe {
    ($e:expr) => {{
        let _ = &$e;
        ::std::string::String::from("TODO")
    }};
}
