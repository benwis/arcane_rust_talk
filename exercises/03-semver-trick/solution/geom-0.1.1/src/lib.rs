//! geom 0.1.1: the semver trick.
//!
//! A semver-compatible patch release of 0.1 that depends on 0.2 and
//! re-exports every type that *didn't* change. Anyone on `geom = "0.1"` picks
//! it up with a plain `cargo update`, and from then on `geom 0.1::Point` and
//! `geom 0.2::Point` are the same type.

pub use geom02::Point;

/// `Color` changed incompatibly in 0.2, so 0.1 keeps its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

/// Optional: bridge the types that did change.
impl From<Color> for geom02::Color {
    fn from(Color(r, g, b): Color) -> Self {
        geom02::Color { r, g, b, a: 0xff }
    }
}
