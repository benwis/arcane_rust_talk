use autoderef::describe;
use std::fmt::Debug;

struct NoTraits;

#[derive(Debug)]
#[allow(dead_code)]
struct OnlyDebug {
    x: i32,
}

#[test]
fn display_wins_over_debug() {
    assert_eq!(describe!(42), "Display: 42");
    assert_eq!(describe!("hi"), "Display: hi");
}

#[test]
fn falls_back_to_debug() {
    assert_eq!(describe!(vec![1, 2]), "Debug: [1, 2]");
    assert_eq!(describe!(Some(3)), "Debug: Some(3)");
    assert_eq!(describe!(OnlyDebug { x: 1 }), "Debug: OnlyDebug { x: 1 }");
}

#[test]
fn falls_back_to_opaque() {
    assert_eq!(describe!(NoTraits), "<opaque>");
}

/// The limitation: the method is picked while type-checking the generic
/// function, using only the bounds it can see, not at monomorphization.
fn generic_debug<T: Debug>(t: T) -> String {
    describe!(t)
}

#[test]
fn generic_code_only_sees_its_bounds() {
    // i32 implements Display, but inside `generic_debug` all we know is Debug.
    assert_eq!(generic_debug(42), "Debug: 42");
}
