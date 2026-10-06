---
title: "Arcane Rust"
sub_title: "Four spells the compiler will let you cast"
author: benwis
theme:
  name: catppuccin-mocha
---

What we'll cast
===

<!--
speaker_note: |
  ~1.5 min. Welcome. "Arcane" doesn't mean "don't use this": every technique
  today is load-bearing in a crate you've probably used this week.
  [click through the list] Name each one, don't explain yet.
  Pacing for 30 min: autoderef 7.5, borrow 7.5, semver 4.5, tacit params
  6.5, wrap-up 2.5. That leaves no slack, so keep questions to the end.
  [click] Stable Rust only. The exercises are homework: one slide at the
  end has the details, so don't stop for them now.
-->

<!-- incremental_lists: true -->

1. **Autoderef & autoref specialization**: bending method resolution
2. **The complex key borrow trick**: `HashSet`/`BTreeMap` lookups with borrowed composite keys
3. **The semver trick**: making two major versions share types
4. **Tacit trait parameters**: sneaking past coherence

<!-- incremental_lists: false -->
<!-- pause -->

---

None of these need nightly. All of them are in crates you already use:
`anyhow`, `bevy`, `axum`, `libc`.

Each one has an exercise in `exercises/` to try afterwards.

<!-- end_slide -->

<!-- jump_to_middle -->

I. Autoderef
===

<!--
speaker_note: |
  Section I, ~7.5 min. Ask the room: "When you write x.foo(), who decides
  which foo runs?" Most people's mental model is "the type of x". That's
  mostly right, and the gaps are where the fun is.
-->

_or: who actually gets called when you write `x.foo()`?_

<!-- end_slide -->

The method probe
===

<!--
speaker_note: |
  This slide is the rule. Everything else in the section follows from it.
  Step one: build the deref chain. Box<Rc<String>> gives Box, Rc, String,
  str. The final unsizing step is why [1, 2, 3].len() works: arrays unsize
  to slices.
  [click] For each candidate, try by value, then &, then &mut.
  [click] The first match wins. Emphasise the three bullets:
  - Inherent beats trait only at the same step. Coming up.
  - A failed where clause means "skip", not "error". That's the trick we
    exploit at the end of the section.
  - Method resolution happens before borrowck, so the probe can pick a
    method you then aren't allowed to call (e.g. moving out of a deref).
-->

For `receiver.method()`, rustc builds a list of candidate types by
dereferencing over and over:

```
T  →  *T  →  **T  →  …   (plus one final unsizing step: [T; N] → [T])
```

<!-- pause -->

For **each** candidate `U`, in order, it tries three receiver types:

```
U        then        &U        then        &mut U
```

<!-- pause -->

**The first receiver type with a matching method wins.**

<!-- incremental_lists: true -->

* Inherent methods beat trait methods, *but only at the same step*
* Where clauses that fail rule out a candidate *without an error*
* Then the borrow checker gets a say. Picking a method is not the same as compiling.

<!-- end_slide -->

Walking the probe
===

<!--
speaker_note: |
  Give people 10 seconds to guess, and get a show of hands for "Foo" vs "&Foo".
  [click] Walk the table: the receiver type is &&Foo. Try &&Foo by value:
  no impl. Autoref &&&Foo: no. &mut &&Foo: no. Deref once to &Foo, by
  value: there's an impl for &Foo. Done.
  [click] Note we never reached Foo, even though "Foo" is what most people
  guess. The probe stops at the first hit, not the "best" one.
-->

<!-- column_layout: [3, 2] -->

<!-- column: 0 -->

```rust +line_numbers
#[derive(Clone, Copy)]
struct Foo;

trait Who { fn who(self) -> &'static str; }

impl Who for Foo  { /* returns "Foo"  */ }
impl Who for &Foo { /* returns "&Foo" */ }

let foo = Foo;
let r = &&foo;
r.who() // ???
```

<!-- column: 1 -->

<!-- pause -->

| step | receiver    | impl? |
|------|-------------|-------|
| 1    | `&&Foo`     | ✗     |
| 2    | `&&&Foo`    | ✗     |
| 3    | `&mut &&Foo`| ✗     |
| 4    | `&Foo`      | **✓** |

<!-- pause -->

**`"&Foo"`**

We never got as far as `Foo`.

<!-- reset_layout -->

<!-- end_slide -->

Outer traits beat inner inherents
===

<!--
speaker_note: |
  [highlight steps through the code] Smart derefs to Foo. Foo has an
  inherent name(). Smart has a trait name().
  Ask: who wins? Many people say inherent, because "inherent always wins".
  [click] The trait on Smart matches at step 2 (&Smart). Foo isn't a
  candidate until we deref, at step 4.
  [click] This is why Rc and Arc use associated functions for their own API:
  Rc::strong_count(&rc), Rc::get_mut(&mut rc). Methods on Rc would shadow
  methods on whatever's inside, because the outer type is probed first.
  The as_ptr example: Rc::as_ptr is an associated function, so
  rc.as_ptr() falls through to Vec::as_ptr. If it were a method, the probe
  would find it first (at &Rc) and the same call would return a different
  type pointing at a different address: the Vec struct, not the buffer.
  In unsafe/FFI code that compiles and reads the wrong memory. Box does
  the same (Box::into_raw, so Box<CString>.into_raw() still gives a
  *mut c_char), and the Rc docs say it's deliberate: it "avoids conflicts
  with methods of the inner type T".
  Same reason adding a trait impl to a smart pointer can silently change
  which method existing code calls.
-->

```rust {1-5|7-9|11|all} +line_numbers
struct Smart(Foo);

impl Deref for Smart {
    type Target = Foo;
    fn deref(&self) -> &Foo { &self.0 }
}

impl Foo                  { fn name(&self) -> &str { "Foo (inherent)" } }
trait Named               { fn name(&self) -> &str; }
impl Named for Smart      { fn name(&self) -> &str { "Smart (trait)" } }

Smart(Foo).name()   // ???
```

<!-- pause -->

**`"Smart (trait)"`**: `Named::name` matches at `&Smart` (step 2), and `Foo`
isn't even a candidate until step 4.

<!-- pause -->

This is why `Rc` uses `Rc::strong_count(&rc)`, not `rc.strong_count()`: a
method on the smart pointer would shadow one on whatever it points to.

```rust
let rc: Rc<Vec<u8>> = Rc::new(vec![1, 2, 3]);
rc.as_ptr()       // Vec::as_ptr → *const u8: the heap buffer
Rc::as_ptr(&rc)   // Rc::as_ptr  → *const Vec<u8>: the Vec inside the Rc
```


<!-- end_slide -->

The one that bites everyone
===

<!--
speaker_note: |
  This one has bitten everyone in the room, whether they know it or not.
  [click] The receiver x has type &T. Step 1, by value on &T: T's clone
  takes &T, but there's no T: Clone bound, so it's skipped. Step 2, autoref
  &&T: &T is always Clone (shared refs are Copy), so that matches. You get a
  &T back. The code compiles and does nothing useful.
  [click] With the bound, T's clone matches at step 1, by value on &T,
  before the probe ever gets to autoref. You get a T.
  [click] Rustc now warns about this (noop_method_call), but only in
  the obvious cases. In generic code it can still surprise you.
-->

<!-- column_layout: [1, 1] -->

<!-- column: 0 -->

```rust
fn dup<T>(x: &T) {
    let y = x.clone();
}
```

<!-- pause -->

`T` has no `Clone` bound, so `<T as Clone>::clone` (receiver `&T`) is
skipped…

…and autoref `&&T` finds `impl Clone for &T`.

`y: &T`. You cloned the **reference**.

<!-- column: 1 -->

<!-- pause -->

```rust
fn dup<T: Clone>(x: &T) {
    let y = x.clone();
}
```

`y: T`, found at the very first step: by value on `&T`, exactly the
`&self` receiver `Clone::clone` wants.

<!-- reset_layout -->

<!-- pause -->

```
warning: call to `.clone()` on a reference in this situation does nothing
  = note: `#[warn(noop_method_call)]` on by default
```

<!-- end_slide -->

Autoref specialization
===

<!--
speaker_note: |
  Now we weaponise the probe. dtolnay calls this "autoref-based stable
  specialization".
  [click] [highlight: Wrap] A wrapper so we control the types involved.
  [highlight: ViaDisplay] Highest priority, implemented for &&Wrap<T> with
  T: Display.
  [highlight: ViaDebug] Next tier, for &Wrap<T> with T: Debug.
  [highlight: ViaNothing] Fallback, for Wrap<T>, no bounds.
  All three traits have a method with the same name. Normally that's an
  ambiguity error. The probe never sees the ambiguity, because it stops at
  the first step where something matches.
-->

Specialization isn't stable. But the method probe *tries candidates in order*
and *silently skips* impls whose bounds don't hold…

<!-- pause -->

```rust {1|3-6|8-11|13-16|all} +line_numbers
struct Wrap<T>(T);

trait ViaDisplay { fn describe(&self) -> String; }
impl<T: Display> ViaDisplay for &&Wrap<T> {
    fn describe(&self) -> String { format!("Display: {}", self.0) }
}

trait ViaDebug { fn describe(&self) -> String; }
impl<T: Debug> ViaDebug for &Wrap<T> {
    fn describe(&self) -> String { format!("Debug: {:?}", self.0) }
}

trait ViaNothing { fn describe(&self) -> String; }
impl<T> ViaNothing for Wrap<T> {
    fn describe(&self) -> String { "<opaque>".into() }
}
```

<!-- end_slide -->

Autoref specialization
===

<!--
speaker_note: |
  The macro takes a reference to the expression (so we don't move it),
  wraps it, and adds three &s.
  [click] Walk the table. The method takes &self, so at step 1 the receiver
  &&&Wrap means Self = &&Wrap. That's the Display impl. If T isn't
  Display, the where clause fails, and the probe silently moves on: deref
  once, Self = &Wrap, the Debug impl. Then the fallback.
  [click] Three different behaviours from one macro, on stable, no
  specialization feature.
-->

```rust
macro_rules! describe {
    ($e:expr) => {{
        use $crate::{ViaDisplay as _, ViaDebug as _, ViaNothing as _};
        (&&&$crate::Wrap(&$e)).describe()
    }};
}
```

<!-- pause -->

| step | receiver `&Self` | `Self`       | needs       |
|------|------------------|--------------|-------------|
| 1    | `&&&Wrap<_>`     | `&&Wrap<_>`  | `Display`   |
| 2    | `&&Wrap<_>`      | `&Wrap<_>`   | `Debug`     |
| 3    | `&Wrap<_>`       | `Wrap<_>`    | nothing     |

<!-- pause -->

```rust
describe!(42)         // "Display: 42"
describe!(vec![1])    // "Debug: [1]"
describe!(NoTraits)   // "<opaque>"
```

<!-- end_slide -->

The catch
===

<!--
speaker_note: |
  The catch: resolution happens when the code is type-checked. Inside a
  generic function, the only facts available are the declared bounds.
  T: Debug is known, T: Display is not, so the Debug impl wins, even when
  T is later instantiated with i32.
  [click] So it only works in macros, where the expression has a concrete
  type at the expansion site.
  [click] Real-world example: anyhow!(...) uses exactly this to decide
  whether its argument is a std::error::Error (keep the source chain) or
  just Display + Debug (wrap it as an ad hoc message).
-->

The method is chosen when the code is **type-checked**, not when it's
monomorphized.

```rust
fn generic<T: Debug>(t: T) -> String {
    describe!(t)
}

generic(42)   // "Debug: 42", even though i32: Display
```

<!-- pause -->

So it's a **macro-only** technique, for concrete types at the call site.

<!-- pause -->

**In the wild:** `anyhow!($err)` uses it to tell apart "a `std::error::Error`"
from "just something `Display`", then picks the right constructor.

<!-- end_slide -->

<!-- jump_to_middle -->

II. The complex key borrow trick
===

<!--
speaker_note: |
  Section II, ~7.5 min. Credit up front: this walkthrough follows Rain's
  literate example, sunshowers-code/borrow-complex-key-example, which
  credits Ivan Dubrov's 2018 post "Tricking the HashMap". Their structure,
  our slides.
-->

_or: looking up a `HashSet<OwnedKey>` with a borrowed key_

<!-- alignment: center -->

From Rain's `sunshowers-code/borrow-complex-key-example`

<!-- end_slide -->

How `contains` takes a `&str`
===

<!--
speaker_note: |
  Start from something everyone's done: a HashSet<String> looked up with a
  &str. contains isn't overloaded, so how does that type-check?
  Answer: it's generic over Q with T: Borrow<Q>, and String: Borrow<str>.
  [click] Implementing Borrow has a mechanical part (write borrow) and a
  contract part: for any two owned values, comparing or hashing them must
  give the same answer as comparing or hashing their borrowed forms. Ord
  matters for BTrees, Hash for hash collections. String and str literally
  share their Hash and Eq code, so they're trivially consistent.
  [click] The compiler checks none of this. Break it and nothing panics:
  lookups just silently miss. "Hold that thought": we'll test it later.
-->

```rust
let set: HashSet<String> = /* … */;
set.contains("example-string")   // a &str, not a &String!

pub fn contains<Q>(&self, value: &Q) -> bool
where T: Borrow<Q>, Q: Hash + Eq + ?Sized
```

<!-- pause -->

`O: Borrow<B>` means `fn borrow(&self) -> &B`, **plus** a contract. For all
owned `o1`, `o2` and their borrowed forms `b1`, `b2`:

| trait  | must always hold                              |
|--------|-----------------------------------------------|
| `Eq`   | `o1 == o2`  ⟺  `b1 == b2`                     |
| `Ord`  | `o1.cmp(o2)` == `b1.cmp(b2)`                  |
| `Hash` | `o1` hashes the same as `b1`, for any hasher  |

<!-- pause -->

The compiler checks **none** of this. Break it, and lookups silently miss.
Hold that thought.

<!-- end_slide -->

Now make the key complex
===

<!--
speaker_note: |
  Two structs, the same shape, one owns its data and one borrows it. Very
  common for composite keys: a name plus some bytes, an (org, repo) pair.
  We want to look up an owned-key set using the borrowed form.
  [click] The obvious attempt. Ask what goes in the body. Nothing works:
  borrow must return a reference, and there's no BorrowedKey stored inside
  an OwnedKey to point at. You can't return a reference to a temporary.
  [click] So most people give up and allocate twice per lookup.
-->

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OwnedKey { s: String, bytes: Vec<u8> }

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct BorrowedKey<'a> { s: &'a str, bytes: &'a [u8] }
```

<!-- pause -->

```rust
impl<'a> Borrow<BorrowedKey<'a>> for OwnedKey {
    fn borrow(&self) -> &BorrowedKey<'a> { /* … what goes here? */ }
}
```

`borrow` returns a **reference**, and there's no `BorrowedKey` hiding
*inside* an `OwnedKey` to point at.

<!-- pause -->

So everyone writes this and moves on:

```rust
set.contains(&OwnedKey { s: s.to_owned(), bytes: bytes.to_vec() })  // 2 allocations
```

<!-- end_slide -->

Steps 1 & 2: a trait for "has a borrowed form"
===

<!--
speaker_note: |
  The insight: we can't return a reference to a BorrowedKey, but we can
  return a reference to something that can *produce* one.
  [highlight: trait] Key produces the borrowed form on demand.
  [highlight: OwnedKey impl] The owned key builds one from its fields.
  [highlight: BorrowedKey impl] The borrowed key returns a copy of itself.
  Mention variance: BorrowedKey<'a> can be returned as BorrowedKey<'k>
  for a shorter 'k because it's covariant in 'a. Rain has a separate
  tutorial on lifetime variance if anyone wants to go deeper.
-->

```rust {1-3|5-9|11-17|all} +line_numbers
trait Key {
    fn key<'k>(&'k self) -> BorrowedKey<'k>;
}

impl Key for OwnedKey {
    fn key<'k>(&'k self) -> BorrowedKey<'k> {
        BorrowedKey { s: self.s.as_str(), bytes: self.bytes.as_slice() }
    }
}

impl<'a> Key for BorrowedKey<'a> {
    fn key<'k>(&'k self) -> BorrowedKey<'k> {
        // A copy with the shorter lifetime 'k. Fine because
        // BorrowedKey is *covariant* in 'a.
        *self
    }
}
```

<!-- end_slide -->

Step 3: borrow as a trait object
===

<!--
speaker_note: |
  And here's the magic line. The body is just `self`: an unsizing
  coercion from &OwnedKey to &dyn Key. No temporary, nothing to dangle.
  So Q = dyn Key, and any &dyn Key can now be used for lookups, including
  a &BorrowedKey coerced to one.
  [click] Only the stored type needs the Borrow impl, since contains
  requires T: Borrow<Q>, not Q: Borrow<anything>.
-->

There's no `BorrowedKey` inside an `OwnedKey`… but an `OwnedKey` **is** a
`dyn Key`:

```rust
impl<'a> Borrow<dyn Key + 'a> for OwnedKey {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self      // ✓ an unsizing coercion, not a temporary
    }
}
```

<!-- pause -->

Only the **stored** type needs the impl. `contains` requires `T: Borrow<Q>`,
so there's no need for `BorrowedKey: Borrow<dyn Key>`.

<!-- end_slide -->

Steps 4–6: give `dyn Key` the comparison traits
===

<!--
speaker_note: |
  Q must be Hash + Eq (+ Ord for BTrees), so we implement those for the
  trait object type itself. Many people don't know you can write
  impl SomeTrait for dyn OtherTrait. It's a perfectly normal unsized type.
  Each impl calls key() on both sides and delegates to BorrowedKey's
  derived impl.
  [click] This is the subtle part: it's consistent with OwnedKey's derived
  impls only because derives work field by field in declaration order, and
  String hashes and compares exactly like &str, Vec<u8> like &[u8].
  Reorder the fields in one struct and it silently breaks. Remember that.
-->

Yes, you can implement traits for a `dyn Trait` type:

```rust
impl<'a> PartialEq for dyn Key + 'a {
    fn eq(&self, other: &Self) -> bool { self.key().eq(&other.key()) }
}
impl<'a> Eq for dyn Key + 'a {}

impl<'a> Ord for dyn Key + 'a {          // only for BTree collections
    fn cmp(&self, other: &Self) -> Ordering { self.key().cmp(&other.key()) }
}

impl<'a> Hash for dyn Key + 'a {         // only for hash collections
    fn hash<H: Hasher>(&self, state: &mut H) { self.key().hash(state) }
}
// (+ PartialOrd)
```

<!-- pause -->

Each one delegates to `BorrowedKey`'s **derived** impl. That's consistent with
`OwnedKey`'s derives only because derives go field by field, **in declaration
order**, and both structs list `s` then `bytes`.

<!-- end_slide -->

Using it
===

<!--
speaker_note: |
  Works with all four std collections, BTreeMap included, with zero
  allocations and no dependencies. hashbrown and indexmap have an
  Equivalent trait that does this more directly, but only for their own
  map types. Use it for internal hash maps on a hot path. Keep the dyn
  trick when std's HashMap is in your public API (switching is a breaking
  change) or you need a BTreeMap. The consistency contract applies either
  way. (Details are in the Q&A note on the last slide.)
  [click] A gotcha: Rain's original writes contains(&key) and relies on a
  coercion. On current compilers that fails. Q is inferred from the
  argument as BorrowedKey, and there's no Borrow<BorrowedKey> impl. The
  repo's own comment says to add `as &dyn Key` if it doesn't work, and now
  you need it.
-->

```rust
let key = BorrowedKey { s: "foo", bytes: b"abc" };

hash_set.contains(&key as &dyn Key)    // HashSet<OwnedKey>
hash_map.get(&key as &dyn Key)         // HashMap<OwnedKey, V>
btree_set.contains(&key as &dyn Key)   // BTreeSet<OwnedKey>
btree_map.remove(&key as &dyn Key)     // BTreeMap<OwnedKey, V>
```

Zero allocations, on std collections, no dependencies.
(`hashbrown`/`indexmap` have an `Equivalent<K>` trait for this, but only
for their own map types: not std's, and no `BTreeMap`.)

<!-- pause -->

The original example writes plain `contains(&key)`. On today's compiler:

```
error[E0277]: the trait bound `OwnedKey: Borrow<BorrowedKey<'_>>` is not satisfied
```

`Q` is inferred from the argument, `BorrowedKey`, before any coercion gets a
chance. Spell out the `as &dyn Key`.

<!-- end_slide -->

Don't trust it, test it
===

<!--
speaker_note: |
  Back to "hold that thought". The consistency rules are properties:
  "for all owned1, owned2". That's exactly what property-based testing is
  for. proptest generates random pairs and checks Eq, Ord and Hash agree
  between the owned and borrowed forms.
  [click] Demo point: swap the field order in BorrowedKey only. Eq still
  passes, since field-by-field equality doesn't care about order. But Ord is
  lexicographic in field order and Hash feeds fields in order, so both
  diverge. proptest finds it and shrinks to a minimal counterexample. Without
  the test, your lookups would just silently miss.
-->

Consistency is a **property**, so check it with property-based testing:

```rust
proptest! {
    #[test]
    fn consistent_borrow(owned1 in owned_key(), owned2 in owned_key()) {
        let borrowed1: &dyn Key = &owned1;
        let borrowed2: &dyn Key = &owned2;

        prop_assert_eq!(owned1 == owned2, borrowed1 == borrowed2);
        prop_assert_eq!(owned1.cmp(&owned2), borrowed1.cmp(borrowed2));
        prop_assert_eq!(hash_output(&owned1), hash_output(borrowed1));
        prop_assert_eq!(hash_output(&owned2), hash_output(borrowed2));
    }
}
```

<!-- pause -->

Now swap the field order in `BorrowedKey`:

```
minimal failing input: owned1 = OwnedKey { … }
  left: `…`, right: `…`: consistent Hash
```

`Eq` still passes. `Hash` and `Ord` don't. Your lookups would have silently
missed.

<!-- end_slide -->

<!-- jump_to_middle -->

III. The semver trick
===

<!--
speaker_note: |
  Section III, ~4.5 min, the shortest. This one's about ecosystems, not
  code. Ask: who has seen an error saying a type isn't the same as itself?
-->

_or: how to stop a breaking release from splitting the ecosystem in two_

<!-- end_slide -->

The split
===

<!--
speaker_note: |
  Setup: your app depends on two crates you don't control. One is still on
  geom 0.1, the other has moved to 0.2.
  [click] You pass a Point from one to the other.
  [click] The real error, and it's remarkably honest: expected Point, found a
  different Point, and the note tells you why.
  [click] To Cargo, 0.1 and 0.2 are semver incompatible, so it builds both,
  and to rustc they're two unrelated crates that happen to share a name.
  Every type is distinct, even ones that didn't change between versions.
  Fixing it normally means waiting for every crate in the ecosystem to
  upgrade.
-->

```
app
├── shapes ── geom 0.1
└── render ── geom 0.2
```

<!-- pause -->

```rust
render::polyline(&shapes::unit_square())
```

<!-- pause -->

```
error[E0308]: mismatched types
  |
  |     render::polyline(&shapes::unit_square())
  |                      ^^^^^^^^^^^^^^^^^^^^^^ expected `geom::Point`,
  |                                             found a different `geom::Point`
  |
note: there are multiple different versions of crate `geom` in the dependency graph
```

<!-- pause -->

Cargo treats `0.1` and `0.2` as **unrelated crates**. Every type is
distinct, even ones that didn't change at all.

<!-- end_slide -->

The trick
===

<!--
speaker_note: |
  The trick, from dtolnay: publish a new *0.1 patch release* that
  depends on 0.2. A crate can depend on a different major version of
  itself. You just rename the dependency.
  [click] Re-export every type that didn't change. Keep the old definitions
  of the types that did.
  [click] Now anyone on geom = "0.1" gets 0.1.1 from a plain cargo update,
  and 0.1's Point *is* 0.2's Point. Nobody downstream had to change a line.
  Show the cargo tree from the exercise if there's time.
-->

Publish **geom 0.1.1**, a semver-compatible patch release of 0.1 that
**depends on geom 0.2**:

```toml
[package]
name = "geom"
version = "0.1.1"

[dependencies]
geom02 = { package = "geom", version = "0.2" }   # yes, itself
```

<!-- pause -->

```rust
// geom 0.1.1: src/lib.rs
pub use geom02::Point;          // unchanged in 0.2: re-export it

pub struct Color(pub u8, pub u8, pub u8);   // changed in 0.2: keep the old one
```

<!-- pause -->

```
app
├── shapes ── geom 0.1.1 ──┐
└── render ────────────────┴── geom 0.2        one `Point` 🎉
```

`cargo update` is all the downstream crates need.

<!-- end_slide -->

The fine print
===

<!--
speaker_note: |
  [click through each bullet]
  - Only re-export types whose public API didn't break. Additions are fine.
  - The subtle one: trait impls. If 0.1's Point implemented a foreign trait
    like Serialize, 0.2's Point must too. 0.1.1 can't add it, because the
    orphan rule forbids a foreign trait on a foreign type, and Point is now
    foreign to 0.1.1. (It can still add its own traits, and From impls
    between the forked types.)
  - Timing: do it right after releasing 0.2, before the split spreads.
    dtolnay's semver-trick repo uses libc as the motivating case.
-->

<!-- incremental_lists: true -->

* Only types whose public API is **unchanged** (or only grew) can be
  re-exported. Everything else stays forked.
* **Trait impls come with the type.** If 0.1's `Point` implemented
  `serde::Serialize`, 0.2's `Point` had better too, because 0.1.1 can't add
  a foreign trait impl to a foreign type (orphan rule).
* Do it **right after** releasing 0.2, before the ecosystem fragments.
  (dtolnay wrote it up as `dtolnay/semver-trick`.)

<!-- end_slide -->

<!-- jump_to_middle -->

IV. Tacit trait parameters
===

<!--
speaker_note: |
  Section IV, ~6.5 min. Last spell, and the most generally useful one. If
  people only remember one technique from today, this is a good one.
-->

_or: a type parameter nobody writes, that makes coherence go away_

<!-- end_slide -->

The goal
===

<!--
speaker_note: |
  One function, four very different kinds of argument. A Python
  programmer would just do this. Can Rust?
  [click] The natural approach: one trait, a blanket impl per category.
  [click] E0119. Pause on the error and ask why, since closures aren't
  Display. Take an answer or two before the next slide.
-->

```rust
label(42)                          // "42"         Display
label(|| expensive())              // "…"          FnOnce() -> String
label([1, 2, 3].iter())            // "1, 2, 3"    Iterator<Item: Display>
label(Some(5))                     // "5"          Option<anything above>
```

<!-- pause -->

```rust
trait IntoLabel { fn into_label(self) -> String; }

impl<T: Display>            IntoLabel for T { /* to_string */ }
impl<F: FnOnce() -> String> IntoLabel for F { /* call it   */ }
```

<!-- pause -->

```
error[E0119]: conflicting implementations of trait `IntoLabel`
  |
2 | impl<T: std::fmt::Display> IntoLabel for T { … }
  | ------------------------------------------ first implementation here
3 | impl<F: FnOnce() -> String> IntoLabel for F { … }
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation
```

<!-- end_slide -->

Why coherence says no
===

<!--
speaker_note: |
  Coherence guarantees at most one impl for any type, both now and after
  upstream crates add impls in future minor releases.
  [click] Rust can't reason negatively: it can't conclude "no type will ever
  be both Display and FnOnce() -> String". So two blanket impls over
  unrelated bounds *might* overlap, and that's enough to reject them.
  [click] The trick: give the trait a type parameter. IntoLabel<ViaDisplay>
  and IntoLabel<ViaFn> are different traits as far as coherence is
  concerned, so they can't overlap.
  [highlight: label] And the caller never writes the marker: it's a generic
  parameter of label, inferred. That's what "tacit" means.
-->

Coherence must guarantee **at most one impl** of a trait for any type, now and
after any future (non-breaking) change upstream.

<!-- pause -->

Nothing stops a type from being both `Display` **and** `FnOnce() -> String`.
Rust has no negative reasoning ("closures are never `Display`"), so the two
blanket impls *might* overlap, and that's enough.

<!-- pause -->

But `IntoLabel<A>` and `IntoLabel<B>` are **different traits**:

```rust {1|3-6|8|all} +line_numbers
trait IntoLabel<Marker> { fn into_label(self) -> String; }

enum ViaDisplay {}
enum ViaFn {}
impl<T: Display>            IntoLabel<ViaDisplay> for T { … }
impl<F: FnOnce() -> String> IntoLabel<ViaFn>      for F { … }

fn label<M>(x: impl IntoLabel<M>) -> String { x.into_label() }
```

<!-- end_slide -->

How `label(42)` works
===

<!--
speaker_note: |
  How does inference pick the marker?
  [click] The solver needs i32: IntoLabel<?M> with ?M unknown. It looks at
  every impl that could unify and checks the where clauses.
  Only the Display impl survives.
  [click] One candidate means ?M = ViaDisplay. Zero means a "trait not
  implemented" error. Two or more is the next slide.
  [click] The key sentence: coherence is checked against impls, ambiguity
  against uses. We moved the error from where the impls are defined to the
  rare call site that's genuinely ambiguous.
-->

```rust
label(42)       // needs: i32: IntoLabel<?M>
```

<!-- pause -->

The trait solver looks at every impl that *could* produce `i32: IntoLabel<?M>`:

| impl                          | where clause            | |
|-------------------------------|-------------------------|-|
| `IntoLabel<ViaDisplay> for T` | `i32: Display`          | ✓ |
| `IntoLabel<ViaFn> for F`      | `i32: FnOnce() -> String` | ✗ |
| `IntoLabel<ViaIter> for I`    | `i32: Iterator`         | ✗ |

<!-- pause -->

Exactly one survives, so `?M = ViaDisplay`. The caller never wrote a marker.

<!-- pause -->

**Coherence is checked against impls. Ambiguity is checked against uses.**
We traded a definition-site error for a (rare) call-site one.

<!-- end_slide -->

When it *is* ambiguous
===

<!--
speaker_note: |
  A type that's both Display and an Iterator matches two impls, so
  label(Both) gives E0283, "type annotations needed".
  [click] Since the marker is just a type parameter, the caller can pick:
  label::<ViaDisplay>. Turbofish alongside impl Trait arguments has been
  allowed since 1.63. If your marker types are private (as in axum),
  callers can't do this, which may be what you want.
-->

```rust
struct Both;   // Display + Iterator

label(Both)
```

```
error[E0283]: type annotations needed
```

<!-- pause -->

The marker is still a type parameter, so you can name it:

```rust
label::<ViaDisplay>(Both)   // "both"
label::<ViaIter>(Both)      // joins whatever it yields
```

(Explicit generics alongside `impl Trait` arguments: stable since 1.63.)

<!-- end_slide -->

Markers compose
===

<!--
speaker_note: |
  We want Option<T> for any labelable T, so the obvious impl just forwards
  the marker.
  [click] That conflicts again: for M = ViaDisplay, both this impl and the
  Display blanket impl could apply to Option<T>, since std could add a
  Display for Option in the future. So make the outer marker unique and
  carry the inner one in a tuple: (ViaOption, M). M must appear in the
  trait's parameters anyway, or it's unconstrained (E0207).
  [click] Bevy: any fn whose parameters are system params becomes a system
  via IntoSystem's Marker. Axum uses the same family of tricks for its
  extractors and handlers. If people want the axum version, the full deck
  and exercise 05 cover it: mention that at the end if asked.
-->

`Option<T>` for *any* labelable `T`:

```rust
impl<T, M> IntoLabel<M> for Option<T> where T: IntoLabel<M> { … }
```

<!-- pause -->

That would conflict with the others again: `Option<T>` *could* be
`IntoLabel<ViaDisplay>` both ways. Make the outer marker unique and carry the
inner one along:

```rust
enum ViaOption {}

impl<T, M> IntoLabel<(ViaOption, M)> for Option<T>
where
    T: IntoLabel<M>,
{ … }

label(Some(|| "lazy".into()))   // M = (ViaOption, ViaFn)
```

<!-- pause -->

**In the wild:** Bevy's `IntoSystem<In, Out, Marker>` lets any `fn(Query<..>,
Res<..>)` be a system. Axum's `FromRequest<S, M>` and `Handler<T>` are why
any `async fn(State<_>, Json<_>)` can be a route handler.

<!-- end_slide -->

🧪 Homework
===

<!--
speaker_note: |
  ~45 s. Don't walk through these, just point at them. Each exercise has
  an exercise.rs to edit, tests as the spec, and a reference solution
  behind --features solution so people can check their work later.
  Mention that 03 is its own workspace because it starts out broken on
  purpose, and that for 02 and 04 the tests don't compile until the impls
  exist: that compile error is the first failing test.
-->

Each spell has an exercise in `exercises/`:

| exercise | you'll… |
|----------|---------|
| `01-autoderef` | predict six method calls, then build `describe!` |
| `02-complex-key-borrow` | make lookups allocation free, proptest the contract |
| `03-semver-trick` | fix "expected `Point`, found a different `Point`" |
| `04-tacit-params` | one `label()` for `Display`, closures, iterators, `Option` |

```bash
cargo test -p <exercise>                      # your attempt
cargo test -p <exercise> --features solution  # the reference solution

cd exercises/03-semver-trick && cargo run     # 03 is its own workspace
```

<!-- end_slide -->

The spellbook
===

<!--
speaker_note: |
  ~1 min. One sentence per row: what the trick is, and where people have
  already met it without knowing.
  [click] The closing point: each of these costs readability and makes
  errors weirder. They pay off in library code, where one maintainer
  absorbs the complexity so every user gets a nicer API. In application
  code, reach for the boring solution first.
-->

| spell | the trick | seen in |
|-------|-----------|---------|
| autoref specialization | probe order + silently failing bounds | `anyhow!` |
| complex key borrow | `Borrow<dyn Key>` + proptest | std collections, composite keys |
| semver trick | old major re-exports new major | `libc`-style ecosystems |
| tacit parameters | marker types split blanket impls | `bevy`, `axum` |

<!-- pause -->

**With great power:** each of these costs you some readability. Reach for them
in library code, where one maintainer pays so a thousand users don't.

<!-- end_slide -->

Further reading
===

<!--
speaker_note: |
  ~30 s, then questions. Point at dtolnay's case studies and Rain's repo,
  which is a great model of literate Rust.
  Likely questions:
  - "Is autoref specialization going away?" No, it relies on documented
    method resolution, though real specialization would make it unnecessary.
  - "Should I always use hashbrown's Equivalent?" Not always. Yes for
    internal hash maps on a hot path, or if you already use hashbrown or
    indexmap. No when std's HashMap is in your public API (switching
    map types is a breaking change) or you need a BTreeMap, which has no
    equivalent mechanism. Not hot? Just allocate. Either way the
    consistency contract still applies: the proptest works unchanged.
    Bonus caveat: hashbrown's default hasher favours speed over HashDoS
    resistance, so pass std's RandomState if keys come from untrusted input.
  - "How does axum do it?" Tacit parameters plus a macro per arity. The
    full deck and exercise 05 walk through it.
-->

* dtolnay, _Autoref-based stable specialization_: `github.com/dtolnay/case-studies`
* Rain, _Implementing Borrow for complex keys_:
  `github.com/sunshowers-code/borrow-complex-key-example`
* Ivan Dubrov, _Tricking the HashMap_: `idubrov.name/rust/2018/06/01/tricking-the-hashmap.html`
* dtolnay, _The semver trick_: `github.com/dtolnay/semver-trick`
* The Rust Reference: _Method call expressions_
* `bevy_ecs/src/system/function_system.rs`, `axum/src/handler/mod.rs`

<!-- new_lines: 2 -->

<!-- alignment: center -->

**Thanks! 🦀✨**
