# Exercise 03: the semver trick

You depend on two crates you don't control:

- `shapes` uses `geom = "0.1"`
- `render` uses `geom = "0.2"`

`geom 0.2` was a breaking release, so Cargo builds both versions, and as far
as rustc is concerned they're unrelated crates. Every type in them is
different:

```console
$ cargo run
error[E0308]: mismatched types
  expected `geom::Point`, found a different `geom::Point`
note: there are multiple different versions of crate `geom` in the dependency graph
```

But `Point` didn't actually change between 0.1 and 0.2. Only `Color` did.

## Your task

You maintain `geom`. Without touching `shapes`, `render` or `app`'s code, make
`cargo run` work by publishing a new release of **0.1** (edit `geom-0.1/`).

Rules:

1. The release must be semver compatible with 0.1.0, so people on `geom =
   "0.1"` pick it up automatically. Bump the version to `0.1.1`.
2. `shapes::favorite_color()` must still return the 0.1 `Color` (a tuple
   struct), because `Color` really did change.

Hints:

- A crate can depend on a different major version of *itself*. You'll need
  to rename the dependency (`package = "geom"`).
- `pub use` makes two paths name the same type.

## Checking the reference solution

The solution is in `solution/geom-0.1.1/`. To try it, point `shapes` at it:

```toml
# shapes/Cargo.toml
geom = { version = "0.1", path = "../solution/geom-0.1.1" }
```

## Things to notice afterwards

- `cargo tree -i geom@0.2.0`: who depends on 0.2 now?
- What would go wrong if 0.1 had a trait that 0.2 removed, and 0.1's `Point`
  implemented it? What about a foreign trait like `serde::Serialize`?
