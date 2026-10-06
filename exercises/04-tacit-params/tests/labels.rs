use std::fmt;
use tacit_params::{ViaDisplay, ViaIter, label};

#[test]
fn display_values() {
    assert_eq!(label(42), "42");
    assert_eq!(label("hi"), "hi");
    assert_eq!(label(String::from("owned")), "owned");
}

#[test]
fn closures() {
    let name = String::from("lazy");
    assert_eq!(label(move || format!("{name}!")), "lazy!");
}

#[test]
fn iterators() {
    assert_eq!(label([1, 2, 3].iter()), "1, 2, 3");
    assert_eq!(label((1..=3).map(|x| x * 10)), "10, 20, 30");
}

#[test]
fn options_of_anything() {
    assert_eq!(label(Some(5)), "5");
    assert_eq!(label(None::<i32>), "none");
    assert_eq!(label(Some(|| String::from("from a closure"))), "from a closure");
    assert_eq!(label(Some(["a", "b"].iter())), "a, b");
    assert_eq!(label(Some(Some(1))), "1");
}

/// A type that is both `Display` and `Iterator` matches two impls. Plain
/// `label(Both)` is ambiguous, but naming the marker picks one.
struct Both(Option<u8>);

impl fmt::Display for Both {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("both")
    }
}

impl Iterator for Both {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.0.take()
    }
}

#[test]
fn disambiguate_with_turbofish() {
    assert_eq!(label::<ViaDisplay>(Both(Some(7))), "both");
    assert_eq!(label::<ViaIter>(Both(Some(7))), "7");
}
