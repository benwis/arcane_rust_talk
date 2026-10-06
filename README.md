# Arcane Rust

A talk plus hands-on exercises covering six techniques that work on stable
Rust and show up all over the ecosystem:

| # | Topic | Exercise |
|---|-------|----------|
| 1 | Autoderef, the method probe & autoref specialization | `exercises/01-autoderef` |
| 2 | The complex key borrow trick (`Borrow<dyn Key>`), after [Rain's example](https://github.com/sunshowers-code/borrow-complex-key-example) | `exercises/02-complex-key-borrow` |
| 3 | The semver trick | `exercises/03-semver-trick` (separate workspace) |
| 4 | Tacit trait parameters vs. coherence | `exercises/04-tacit-params` |
| 5 | The axum handler trick | `exercises/05-handler-trait` |
| 6 | Const assertions | `exercises/06-const-assertions` |

## The slides

Two decks, both with speaker notes:

| deck | length | covers |
|------|--------|--------|
| `slides/arcane-rust.md` | ~30 min | spells 1–4; exercises 01–04 as homework |
| `slides/arcane-rust-full.md` | ~60 min (+ exercise time) | all six spells |

```bash
presenterm slides/arcane-rust.md        # add -p for presentation mode

# speaker notes: run these in two terminals
presenterm -P slides/arcane-rust.md     # presenter
presenterm -l slides/arcane-rust.md     # notes view
```

Checked against 120×35 and larger terminals with `--validate-overflows`.

## The exercises

Each exercise crate has a `src/exercise.rs` to edit and a `src/solution.rs`
behind a `solution` feature. The tests are the spec.

```bash
cargo test -p autoderef                        # check your attempt
cargo test -p autoderef --features solution    # run against the reference solution
cargo test --workspace --all-features          # every reference solution
```

Notes:

- **01-autoderef**: a method-resolution quiz (fill in your predictions
  *before* running anything), then build `describe!` with autoref
  specialization.
- **02-complex-key-borrow**: Rain's walkthrough as an exercise: look up
  `HashSet`/`BTreeSet<OwnedKey>` with a `BorrowedKey`. A counting global
  allocator checks the lookups don't allocate, and proptest checks `Borrow`'s
  `Eq`/`Ord`/`Hash` consistency contract.
- **03-semver-trick** is its own workspace, because its starting state
  deliberately doesn't compile: `cd exercises/03-semver-trick && cargo run`.
  See its README.
- **02**, **04** and **05**: the tests don't compile until the trait impls
  exist. That compile error is your first failing test.
- **06-const-assertions**: several tests are `compile_fail` doctests in
  `src/lib.rs`, which pass only when your code *refuses* to compile the bad
  input.

Requires Rust 1.85+ (edition 2024).
