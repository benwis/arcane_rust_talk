---
title: "Arcane Rust"
sub_title: "Six spells the compiler will let you cast"
author: benwis
theme:
  name: catppuccin-mocha
---

What we'll cast
===

<!--
speaker_note: |
  ~2 min. Welcome. "Arcane" doesn't mean "don't use this": every technique
  today is load-bearing in a crate you've probably used this week.
  [click through the list] Briefly name each one. Don't explain yet.
  Suggested pacing for a ~60 min talk: autoderef 10, borrow 9, semver 6,
  tacit params 8, handlers 12, const 9, wrap-up 3. For a workshop, stop at
  each exercise slide for 10 to 20 minutes.
  [click] Everything is stable Rust, 1.85+ for edition 2024.
  Point at the two cargo commands: your attempt vs. the reference. 03 is
  its own workspace because it starts out broken on purpose.
-->

<!-- incremental_lists: true -->

1. **Autoderef & autoref specialization**: bending method resolution
2. **The complex key borrow trick**: `HashSet`/`BTreeMap` lookups with borrowed composite keys
3. **The semver trick**: making two major versions share types
4. **Tacit trait parameters**: sneaking past coherence
5. **The axum handler trick**: functions as plugins
6. **Const assertions**: turning bugs into compile errors

<!-- incremental_lists: false -->
<!-- pause -->

---

None of these need nightly. All of them are in crates you already use:
`anyhow`, `axum`, `bevy`, `hashbrown`, `static_assertions`, `libc`.

Every section has an exercise in `exercises/`:

```bash
cargo test -p <exercise>                      # your attempt
cargo test -p <exercise> --features solution  # the reference
```


<!-- end_slide -->

<!-- jump_to_middle -->

I. Autoderef
===

<!--
speaker_note: |
  Section I, ~10 min. Ask the room: "When you write x.foo(), who decides
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

Deref coercion: the other half
===

<!--
speaker_note: |
  Deref coercion is a separate mechanism from the method probe, and people
  conflate them. Coercion happens at "coercion sites": function arguments,
  annotated lets, returns, struct fields.
  It applies Deref as many times as needed: three hops here.
  [click] But coercion needs a known target type. With a generic parameter
  there's nothing to coerce to, so T just becomes Box<Rc<String>>.
  Practical upshot: if a generic API "doesn't accept" your smart pointer,
  pass &*b or b.as_ref() explicitly.
-->

Separate from the method probe. At **coercion sites** (function args, `let`
with a type, returns, struct fields), `&U` becomes `&T` when
`U: Deref<Target = T>`, repeated as needed:

```rust
fn shout(s: &str) { /* … */ }

let b: Box<Rc<String>> = Box::new(Rc::new("hi".into()));
shout(&b);   // &Box<Rc<String>> → &Rc<String> → &String → &str
```

<!-- pause -->

But **not** when the target type is generic, because there's nothing to
coerce *to*:

```rust
fn show<T: Debug + ?Sized>(t: &T) { /* … */ }

show(&b);    // T = Box<Rc<String>>, no coercion happens
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

🧪 Exercise 01: `exercises/01-autoderef`
===

<!--
speaker_note: |
  Exercise time (~10 to 15 min in a workshop).
  Part 1 is a quiz: insist people fill in predictions before running
  anything. Running cargo test reveals answers one at a time.
  Common sticking points in part 2:
  - Forgetting to bring all three traits into scope inside the macro.
  - Wrapping $e instead of &$e, which moves the value.
  - Using crate:: instead of $crate:: in the macro.
  Q3 (Smart(Foo).who()) is the one most people get wrong: the probe
  derefs to Foo and calls by value, which works because Foo is Copy.
-->

A six-question method-probe quiz (no running the code first!), then build
`describe!` yourself.

<!-- end_slide -->

<!-- jump_to_middle -->

II. The complex key borrow trick
===

<!--
speaker_note: |
  Section II, ~9 min. Credit up front: this walkthrough follows Rain's
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
  &str. Ask: how does that type-check? contains is not overloaded.
  [click] The signature is generic over Q, with T: Borrow<Q>. String
  implements Borrow<str>, so Q = str.
  [click] The two rules for implementing Borrow. The first is mechanical.
  The second, consistency, is the one that matters, and the compiler can't
  check it. String and str literally share their Hash and Eq code, so
  they're trivially consistent.
-->

```rust
let mut set: HashSet<String> = HashSet::new();
set.insert("example-string".to_string());

set.contains("example-string")   // a &str, not a &String!
```

<!-- pause -->

```rust
impl<T: Hash + Eq> HashSet<T> {
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
}
```

<!-- pause -->

An owned `O` may implement `Borrow<B>` if:

* you can write `fn borrow(&self) -> &B`, **and**
* `Eq`, `Ord` and `Hash` are **consistent** between `O` and `B`

`String: Borrow<str>` qualifies. They even share the same code underneath.

<!-- end_slide -->

What "consistent" means
===

<!--
speaker_note: |
  The formal version. For any two owned values, comparing or hashing them
  must give the same answer as comparing or hashing their borrowed forms.
  Ord matters for BTreeMap and BTreeSet. Hash matters for hash collections.
  [click] If you break this, nothing panics: lookups just silently miss,
  because the probe hashes to the wrong bucket or walks the wrong branch of
  the tree. "Hold that thought": we'll come back with a way to test it.
-->

For **all** owned values `owned1`, `owned2`, with
`borrowed1 = owned1.borrow()` and `borrowed2 = owned2.borrow()`:

| trait  | must always hold                                      |
|--------|-------------------------------------------------------|
| `Eq`   | `owned1 == owned2`  ⟺  `borrowed1 == borrowed2`       |
| `Ord`  | `owned1.cmp(owned2)` == `borrowed1.cmp(borrowed2)`    |
| `Hash` | `owned1` hashes the same as `borrowed1`, for any hasher |

<!-- pause -->

The compiler checks **none** of this. Break it, and lookups silently miss.

Hold that thought.

<!-- end_slide -->

Now make the key complex
===

<!--
speaker_note: |
  Two structs, the same shape, one owns its data and one borrows it. Very
  common for composite keys: a name plus some bytes, an (org, repo) pair,
  and so on.
  [click] We want to look up an owned-key set using the borrowed form,
  without allocating a String and a Vec just to compare.
-->

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OwnedKey {
    s: String,
    bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct BorrowedKey<'a> {
    s: &'a str,
    bytes: &'a [u8],
}
```

<!-- pause -->

The same type, apart from ownership. Can we do this, with no allocation?

```rust
let set: HashSet<OwnedKey> = /* … */;
set.contains(&BorrowedKey { s: "foo", bytes: b"abc" })
```

<!-- end_slide -->

The dead end
===

<!--
speaker_note: |
  The obvious attempt. Ask the room what goes in the body.
  [click] Nothing works: borrow must return a reference, and there is no
  BorrowedKey stored inside an OwnedKey for it to point at. You could build
  one, but you can't return a reference to a temporary.
  [click] So most people give up and allocate on every lookup. In a hot
  loop, that's two heap allocations per lookup.
-->

```rust
impl<'a> Borrow<BorrowedKey<'a>> for OwnedKey {
    fn borrow(&self) -> &BorrowedKey<'a> {
        // … uhh, what do we put here?
    }
}
```

<!-- pause -->

`borrow` returns a **reference**. Unlike `String`/`str`, there's no
`BorrowedKey` hiding *inside* an `OwnedKey` to point at.

<!-- pause -->

So everyone writes this and moves on:

```rust
set.contains(&OwnedKey { s: s.to_owned(), bytes: bytes.to_vec() })
//                       ^^^^^^^^^^^^^^           ^^^^^^^^^^^^^^  2 allocations
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
  Works with all four std collections. Zero allocations: our exercise
  proves this with a counting global allocator.
  [click] A gotcha: Rain's original writes contains(&key) and relies on a
  coercion. On current compilers that fails. Q is inferred from the
  argument as BorrowedKey, and there's no Borrow<BorrowedKey> impl. The
  repo's own comment says to add `as &dyn Key` if it doesn't work, and now
  you need it. Good reminder that inference details shift over time.
-->

```rust
let key = BorrowedKey { s: "foo", bytes: b"abc" };

hash_set.contains(&key as &dyn Key)    // HashSet<OwnedKey>
hash_map.get(&key as &dyn Key)         // HashMap<OwnedKey, V>
btree_set.contains(&key as &dyn Key)   // BTreeSet<OwnedKey>
btree_map.remove(&key as &dyn Key)     // BTreeMap<OwnedKey, V>
```

Zero allocations.

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

Alternatives
===

<!--
speaker_note: |
  Be honest about alternatives.
  [click] hashbrown and indexmap have an Equivalent trait designed for
  exactly this. It's simpler, with no trait object, vtable or cast. But
  it only works with their own map types. Rule of thumb: use it for
  internal hash maps on a hot path, or if you already depend on hashbrown
  or indexmap. Stick with the dyn trick when std's HashMap is in your
  public API (switching is a breaking change) or you need a BTreeMap.
  The consistency contract still applies, and so does the proptest. If
  keys come from untrusted input, note that hashbrown's default hasher
  favours speed over HashDoS resistance: pass std's RandomState.
  [click] raw_entry: never stabilised in std, and on its way out.
  hashbrown's HashTable is the low-level successor.
  [click] Interning sidesteps the problem entirely.
  [click] And sometimes two small allocations are fine. Profile first.
  [click] The dyn trick's selling point: std collections, BTreeMap
  included, with zero dependencies.
-->

<!-- incremental_lists: true -->

* **`hashbrown` / `indexmap`**: `impl Equivalent<OwnedKey> for BorrowedKey<'_>`.
  No trait object, no vtable, no `as &dyn` cast. But only for *their* map
  types: not std's, and no `BTreeMap`. (Same consistency contract!)
* **`raw_entry` API**: unstable in std, and on its way out. `hashbrown` has a
  successor (`HashTable`).
* **Interning**: turn keys into IDs up front, then key by the ID.
* **Just allocate.** Measure first!

<!-- incremental_lists: false -->
<!-- pause -->

The `dyn` trick works on **std's `HashMap`, `HashSet`, `BTreeMap` and `BTreeSet`**,
with **no dependencies**.

<!-- end_slide -->

<!-- jump_to_middle -->

🧪 Exercise 02: `exercises/02-complex-key-borrow`
===

<!--
speaker_note: |
  Exercise (~10 to 15 min). The tests won't compile until the impls exist,
  so the compile error is the first failing test.
  Sticking points:
  - Writing impls for dyn Key without the lifetime: use dyn Key + 'a or
    dyn Key + '_.
  - Forgetting `as &dyn Key` at the lookup site (see the earlier slide).
  - Implementing PartialOrd separately from Ord: write it as
    Some(self.cmp(other)).
  For fast finishers: the bonus is to swap the field order and watch the
  proptest catch it, and to think about enums.
-->

Rain's walkthrough as an exercise: write the `Key` impls, the `Borrow` impl
and the trait-object impls. A counting allocator proves the lookups don't
allocate, and proptest proves `Borrow`'s contract holds.

<!-- end_slide -->

<!-- jump_to_middle -->

III. The semver trick
===

<!--
speaker_note: |
  Section III, ~6 min. This one's about ecosystems, not code. Ask: who has
  seen an error saying a type isn't the same as itself?
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
    foreign to 0.1.1.
  - 0.1.1 can still implement its own traits for the re-exported types,
    and add From conversions between the forked types.
  - Timing: do it right after releasing 0.2, before the split spreads.
  - Credit: dtolnay/semver-trick, which uses libc as the motivating case.
-->

<!-- incremental_lists: true -->

* Only types whose public API is **unchanged** (or only grew) can be
  re-exported. Everything else stays forked.
* **Trait impls come with the type.** If 0.1's `Point` implemented
  `serde::Serialize`, 0.2's `Point` had better too, because 0.1.1 can't add
  a foreign trait impl to a foreign type (orphan rule).
* 0.1.1 *can* still implement its **own** traits for 0.2's types, and add
  `From` impls between the forked types.
* Do it **before** the ecosystem fragments: release 0.2, then 0.1.1 right
  away.
* Written up by dtolnay as `dtolnay/semver-trick`, with `libc` as the
  motivating example.

<!-- end_slide -->

<!-- jump_to_middle -->

🧪 Exercise 03: `exercises/03-semver-trick`
===

<!--
speaker_note: |
  Exercise (~10 min). It's its own workspace: cd into
  exercises/03-semver-trick first.
  Hints if people stall:
  - The dependency needs a rename: geom02 = { package = "geom", ... }.
  - Bump the version to 0.1.1.
  - Keep Color as the 0.1 tuple struct, because app prints it.
  Afterwards, have them run cargo tree -i geom@0.2.0 to see who depends
  on 0.2 now, then discuss the serde question in the README.
-->

Its own workspace: `cd` in and `cargo run`. It doesn't compile.
Publish a `geom 0.1.1` that fixes it without touching anyone else's code.

<!-- end_slide -->

<!-- jump_to_middle -->

IV. Tacit trait parameters
===

<!--
speaker_note: |
  Section IV, ~8 min. This is the conceptual core for section V too. If
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
  trait's parameters anyway, or it's unconstrained (E0207, which we'll meet
  again in a minute).
  [click] Bevy: any fn whose parameters are system params becomes a
  system via IntoSystem's Marker parameter. Axum's FromRequest has an M for
  the same reason. That's our segue.
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
Res<..>)` be a system. Axum's `FromRequest<S, M>`… next section.

<!-- end_slide -->

<!-- jump_to_middle -->

🧪 Exercise 04: `exercises/04-tacit-params`
===

<!--
speaker_note: |
  Exercise (~10 min). Step 0 is to uncomment the naive version and see
  E0119 for real.
  The tests use the marker names (ViaDisplay, ViaIter) for turbofish, so
  people must use those names.
  Most common stumble: the Option impl without the nested marker gives
  E0207 or E0119. Hint: "where does M come from?"
  A lib.rs doctest checks the ambiguous case really is rejected.
-->

See E0119 for yourself, then build `label` for `Display`, closures,
iterators and nested `Option`s.

<!-- end_slide -->

<!-- jump_to_middle -->

V. The axum handler trick
===

<!--
speaker_note: |
  Section V, ~12 min, the longest. Everything so far comes together here:
  tacit parameters, coherence, macros, and a bit of type erasure.
-->

_or: how does a router call a function it's never seen?_

<!-- end_slide -->

The magic
===

<!--
speaker_note: |
  Real axum. Three arguments of three different extractor types, an
  opaque return type, and the router just accepts it.
  [click] Read the bullets. The "(almost)" is because the body-consuming
  extractor has to be last.
  The last bullet is the impressive part: no attribute macro on the
  handler, no registration, no runtime reflection. It's all trait
  resolution at compile time.
  [click] Our version drops async so we can focus on the type tricks.
  Async adds a Future return type to the bounds but the same structure.
-->

```rust
async fn create(
    State(db): State<Db>,
    Path(id): Path<u64>,
    Json(body): Json<NewThing>,
) -> impl IntoResponse { … }

let app = Router::new().route("/things/{id}", post(create));
```

<!-- pause -->

* Any number of arguments, any order (almost)
* Each argument type knows how to build itself from a request
* The return type knows how to become a response
* **No macros on the handler**, no registration, no reflection

<!-- pause -->

Five ingredients. Our version is synchronous so we can focus on the types.

<!-- end_slide -->

Ingredient 1: extractors
===

<!--
speaker_note: |
  Two extractor traits. Parts extractors only need headers, method, URI,
  etc., and borrow them, so any number can run. Request extractors consume
  the body, so only one can run, and it must be last.
  Each has a Rejection type that can turn into a response, so a failed
  extraction short-circuits with, say, a 401.
  [click] Examples from our mini framework. Bearer reads the Authorization
  header and rejects with 401.
  Say explicitly: "ignore the M on FromRequest for now". It's ingredient 4.
-->

```rust
/// Only needs the method, URI, headers… Any position.
trait FromRequestParts: Sized {
    type Rejection: IntoResponse;
    fn from_request_parts(parts: &Parts) -> Result<Self, Self::Rejection>;
}

/// Consumes the request, body included. Last position only.
trait FromRequest<M = ViaRequest>: Sized {
    type Rejection: IntoResponse;
    fn from_request(req: Request) -> Result<Self, Self::Rejection>;
}
```

<!-- pause -->

```rust
impl FromRequestParts for Method { … }      // never fails
impl FromRequestParts for Bearer { … }      // Rejection = (u16, &'static str)
impl FromRequest      for String { … }      // the body
```

Ignore the `M` for now.

<!-- end_slide -->

Ingredient 2: a trait for "callable with extractors"
===

<!--
speaker_note: |
  We want: "any function whose arguments are extractors is a handler".
  The obvious impl:
  [click] E0207. A is not constrained: nothing in "Handler for F" mentions
  A. And a single closure type could, in principle, implement FnOnce for
  several argument types, so rustc can't pick A from F.
  [click] The fix is the section IV trick again: move the argument types
  into a trait parameter. Handler<T> where T is never used in the body.
  It exists to constrain the impl, and callers never write it.
-->

What we want to write:

```rust
impl<F, A, R> Handler for F
where
    F: FnOnce(A) -> R,
    A: FromRequest,
    R: IntoResponse,
```

<!-- pause -->

```
error[E0207]: the type parameter `A` is not constrained by the impl trait,
              self type, or predicates
  |
6 | impl<F, A> Handler for F where F: FnOnce(A) { … }
  |         ^ unconstrained type parameter
```

<!-- pause -->

A closure type could implement `FnOnce(A)` for *many* `A`s, so `A` has to
come from somewhere. Put it in the trait. **A tacit parameter again:**

```rust
trait Handler<T>: Clone + 'static {
    fn call(self, req: Request) -> Response;
}
```

<!-- end_slide -->

Ingredient 3: one impl per arity
===

<!--
speaker_note: |
  Rust has no variadic generics, so we write one impl per arity, with a
  macro.
  [highlight: signature] The parts extractors are $ty, and the last
  argument is separate because it gets the FromRequest<M> bound.
  [highlight: bounds] T for the impl is the tuple (M, T1, ..., Tn). M rides
  along so it's constrained.
  [highlight: body] Split the request, run each parts extractor and
  return the rejection as a response if one fails, reassemble the request,
  run the last extractor, call the function.
  Fun detail: the macro reuses the type parameter names as variable names
  (let T1 = ...). It's legal because types and values live in different
  namespaces. Axum does the same.
-->

```rust {1-2|3-8|9-21|all} +line_numbers
macro_rules! impl_handler {
    ( $($ty:ident),* ; $last:ident ) => {
        impl<F, R, M, $($ty,)* $last> Handler<(M, $($ty,)* $last,)> for F
        where
            F: FnOnce($($ty,)* $last) -> R + Clone + 'static,
            R: IntoResponse,
            $( $ty: FromRequestParts, )*
            $last: FromRequest<M>,
        {
            fn call(self, req: Request) -> Response {
                let (parts, body) = req.into_parts();
                $( let $ty = match $ty::from_request_parts(&parts) {
                    Ok(v) => v,
                    Err(rejection) => return rejection.into_response(),
                }; )*
                let req = Request::from_parts(parts, body);
                let $last = match $last::from_request(req) { /* same */ };
                self($($ty,)* $last).into_response()
            }
        }
    };
}
```

<!-- end_slide -->

Ingredient 3: one impl per arity
===

<!--
speaker_note: |
  Four invocations give 1 to 4 arguments. Axum goes to 16 using a helper
  macro called all_the_tuples!, which is why handlers with 17 extractors
  mysteriously don't compile.
  [click] Zero arguments gets its own impl. T is ((),), a 1-tuple, so it
  can't collide with the others.
  [click] Each arity's T is a different tuple length, so no two impls can
  overlap. Read the three examples: the first element of the tuple is the
  marker of the last extractor.
-->

```rust
impl_handler!(; T1);
impl_handler!(T1; T2);
impl_handler!(T1, T2; T3);
impl_handler!(T1, T2, T3; T4);
// axum goes up to 16, generated by `all_the_tuples!`
```

<!-- pause -->

Plus zero arguments, by hand:

```rust
impl<F, R> Handler<((),)> for F
where F: FnOnce() -> R + Clone + 'static, R: IntoResponse
```

<!-- pause -->

Each arity's `T` is a different tuple length, so no two impls overlap.

```
fn()                 : Handler<((),)>
fn(String)           : Handler<(ViaRequest, String)>
fn(Method, Bearer)   : Handler<(ViaParts, Method, Bearer)>
```

<!-- end_slide -->

Ingredient 4: that `M`
===

<!--
speaker_note: |
  Now the M. We'd like parts extractors to be allowed last as well, so
  whoami(Method, Bearer) works.
  [click] Bridge: every FromRequestParts type is also a FromRequest by
  ignoring the body.
  [click] Careful with the "why": it does NOT conflict with
  impl FromRequest for String. Rustc can prove String will never implement
  our local FromRequestParts trait. The conflict appears as soon as some
  type could implement both traits, e.g. a generic wrapper:
  impl<T: FromRequestParts> FromRequestParts for Option<T> plus
  impl<T: FromRequest> FromRequest for Option<T>. Then Option<T> matches
  two FromRequest impls: E0119 (verified). axum-core has exactly such
  wrapper impls. With the marker they're FromRequest<ViaParts> and
  FromRequest<ViaRequest>, different traits.
  In real axum these markers live in a private module so users can never
  name or turbofish them.
-->

We want *any* extractor to be allowed last, `FromRequestParts` ones included:

```rust
fn whoami(method: Method, Bearer(token): Bearer) -> String
//                        ^^^^^^ FromRequestParts, in last position
```

<!-- pause -->

So: every `FromRequestParts` is a `FromRequest` too.

```rust
impl<T: FromRequestParts> FromRequest<ViaParts> for T {
    type Rejection = T::Rejection;
    fn from_request(req: Request) -> Result<Self, Self::Rejection> {
        let (parts, _) = req.into_parts();
        T::from_request_parts(&parts)
    }
}
```

<!-- pause -->

Without the marker, it conflicts with any type that's *both*. axum-core
gives wrappers like `Option<T>` impls of both traits → E0119. With it:
`FromRequest<ViaParts>` vs `FromRequest<ViaRequest>`, section IV again.
`M` rides along in `Handler<(M, …)>` so it's constrained.

<!-- end_slide -->

Ingredient 5: erase it
===

<!--
speaker_note: |
  Last ingredient: a router needs a single collection of handlers of
  different types. Erase them into Box<dyn Fn(Request) -> Response>.
  [click] T is inferred at the route() call, picks the impl, and is gone
  after this function. The boxed closure doesn't mention it.
  The clone() is why Handler requires Clone: each call consumes the handler
  (FnOnce), so we clone a fresh copy per request.
  Real axum erases into a tower Service, with futures and state, but the
  shape is the same.
-->

A router has to store handlers of **different types** side by side.

```rust
type BoxedHandler = Box<dyn Fn(Request) -> Response>;

impl Router {
    pub fn route<H, T>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
        T: 'static,
    {
        let erased: BoxedHandler = Box::new(move |req| handler.clone().call(req));
        self.routes.entry(path.into()).or_default().insert(method, erased);
        self
    }
}
```

<!-- pause -->

`T` is inferred, used once to pick the impl, and then gone.

(Real axum erases to a `tower::Service` instead, with `Clone` and futures
along for the ride.)

<!-- end_slide -->

The price: error messages
===

<!--
speaker_note: |
  The downside of all this cleverness. Pass a function with a non-extractor
  argument (u32)...
  [click] ...and you get "Handler<_> is not satisfied", followed by a long
  list of every impl the compiler considered. Not helpful for beginners.
  [click] Axum's solution is #[debug_handler], a proc macro that checks
  each argument and the return type individually, with targeted errors.
  The cheap solution, stable since 1.78: #[diagnostic::on_unimplemented],
  which lets the trait author write the error message.
-->

```rust
fn bad(x: u32) -> String { … }

Router::new().route(Method::Get, "/", bad)
```

<!-- pause -->

```
error[E0277]: the trait bound `fn(u32) -> String {bad}: Handler<_>` is not satisfied
```

…and then a list of every `Handler` impl. 😐

<!-- pause -->

Axum's answer is `#[axum::debug_handler]`, which re-checks each argument
separately. The cheap answer, since 1.78:

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid handler",
    label = "not a handler",
    note = "every argument but the last must implement `FromRequestParts`, \
            the last must implement `FromRequest`",
)]
pub trait Handler<T>: Clone + 'static { … }
```

<!-- end_slide -->

The price: error messages
===

<!--
speaker_note: |
  Real compiler output from our exercise. The headline, the label under
  the bad argument, and both notes come from the attribute. One attribute,
  and the error goes from baffling to actionable. If you maintain a trait
  with a lot of blanket impls, add this today.
-->

```
error[E0277]: `fn(u32) -> String {bad}` is not a valid handler
  |
3 | Router::new().route(Method::Get, "/", bad)
  |               -----                   ^^^ not a handler
  |               |
  |               required by a bound introduced by this call
  |
  = help: the trait `Handler<_>` is not implemented for
          fn item `fn(u32) -> String {bad}`
  = note: handlers take up to 4 arguments: every argument but the
          last must implement `FromRequestParts`, the last must
          implement `FromRequest`
  = note: the return type must implement `IntoResponse`
```

<!-- end_slide -->

<!-- jump_to_middle -->

🧪 Exercise 05: `exercises/05-handler-trait`
===

<!--
speaker_note: |
  Exercise (~15 to 20 min, the biggest one). Suggested order: task 1 (the
  ViaParts bridge), then task 2 (zero arguments), then the macro, then
  Bearer.
  Sticking points:
  - Macro repetition syntax: $( ... )* and where the commas go.
  - Calling methods on a type parameter inside the macro: use
    <$ty as FromRequestParts>::from_request_parts to avoid ambiguity.
  - Forgetting Clone + 'static on F.
  The bonus: add on_unimplemented, then pass fn bad(x: u32) to route()
  before and after to compare the errors.
-->

A tiny synchronous axum. Write the `Handler` impls, the macro, the `ViaParts`
bridge, and a fallible `Bearer` extractor.

<!-- end_slide -->

<!-- jump_to_middle -->

VI. Const assertions
===

<!--
speaker_note: |
  Section VI, ~9 min. Shift of gear: less type system trickery, more
  "move checks earlier". Tests catch bugs when you run them. Const
  assertions catch them whenever anyone compiles.
-->

_or: making the test suite unnecessary for a whole class of bugs_

<!-- end_slide -->

Archaeology
===

<!--
speaker_note: |
  A bit of history, because you'll still see this in older code.
  [click] An array length of 0 - (condition as usize). If the condition is
  true, 0 - 0 = 0. If false, 0 - 1 overflows usize in a const, which is a
  compile error. Hacky, but it worked on Rust 1.0.
  [click] Since 1.57 panic works in const evaluation, so assert works.
  The macro is one line.
  `const _` is the key: an unnamed constant can't be referenced, but it's
  still evaluated, so the assertion always runs at compile time.
  Use it to pin down layout: size_of, align_of, and offset_of (1.77).
-->

Before `assert!` worked in `const`, `static_assertions` did this:

```rust
const _: [(); 0 - !{ const ASSERT: bool = $cond; ASSERT } as usize] = [];
```

<!-- pause -->

True → `0 - 0` → `[(); 0]`. Fine.
False → `0 - 1` → **overflow in a constant** → compile error.

<!-- pause -->

Since **1.57**, `panic!` (and so `assert!`) works in const contexts:

```rust
macro_rules! const_assert {
    ($cond:expr $(, $msg:literal)?) => {
        const _: () = assert!($cond $(, $msg)?);
    };
}

const_assert!(size_of::<Header>() == 8);
const_assert!(offset_of!(Header, flags) == 6, "wire format changed!");
```

`const _` is unnamed and never used, but **always evaluated**.

<!-- end_slide -->

Asserting trait impls
===

<!--
speaker_note: |
  No const fn can ask "does T implement Send?". But the type checker can
  answer it.
  A closure containing a generic function with the bounds we want, called
  with our type. If the bound doesn't hold, it fails to type-check.
  ?Sized lets it work with str and dyn types.
  Real use case: assert your futures or error types are Send + Sync, so
  nobody silently breaks that by adding an Rc field.
  [click] The closure is never called, and costs nothing at runtime.
-->

There's no `const fn implements<T, Trait>()`. But type checking is a kind of
assertion:

```rust
macro_rules! assert_impl {
    ($ty:ty : $($bounds:tt)+) => {
        const _: fn() = || {
            fn check<T: ?Sized + $($bounds)+>() {}
            check::<$ty>();
        };
    };
}

assert_impl!(Header: Send + Sync);
assert_impl!(MyFuture: Send);   // catch the `Rc` someone held across an `.await`
```

<!-- pause -->

The closure is never called. It just has to **type-check**.

<!-- end_slide -->

Generics: where it gets interesting
===

<!--
speaker_note: |
  Assertions about generic parameters are where it gets interesting.
  A const item inside a generic fn can't see the fn's generics: E0401.
  Items are independent of their surrounding scope.
  [click] The pre-1.79 workaround: an associated const on the generic impl,
  which can see N. Gotcha: associated consts are lazily evaluated, so you
  must reference it (let () = Self::CHECK) or the check never runs.
-->

```rust
impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        const _: () = assert!(N.is_power_of_two());   // ❌
```

```
error[E0401]: can't use generic parameters from outer item
```

<!-- pause -->

Items don't inherit generics. The old workaround is an associated const:

```rust
impl<T, const N: usize> RingBuffer<T, N> {
    const CHECK: () = assert!(N.is_power_of_two());

    pub fn new() -> Self {
        let () = Self::CHECK;   // mention it, or it's never evaluated
        // …
```

<!-- end_slide -->

Inline const (1.79)
===

<!--
speaker_note: |
  Since 1.79: inline const blocks. They can use the enclosing generics,
  and they're evaluated whenever the containing function is used. One line.
  Bonus pointed out in the code: [const { None }; N] builds an array of
  non-Copy values. Before inline const you needed unsafe, or a Copy bound.
  [click] The error: note "while instantiating". It fires when
  RingBuffer<u8, 3> is actually used, because that's when N is known. Which
  leads to an important caveat...
-->

```rust
impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        const { assert!(N.is_power_of_two(), "capacity must be a power of two") };
        Self { slots: [const { None }; N], next: 0 }
        //             ^^^^^^^^^^^^^^ bonus: array repeat of a non-Copy type
    }
}
```

<!-- pause -->

```
error[E0080]: evaluation panicked: capacity must be a power of two
   |
   = note: evaluation of `RingBuffer::<u8, 3>::new::{constant#0}` failed here
   |
note: the above error was encountered while instantiating `fn RingBuffer::<u8, 3>::new`
  |
1 |     let ring = RingBuffer::<u8, 3>::new();
  |                ^^^^^^^^^^^^^^^^^^^^^^^^^^
```

<!-- end_slide -->

⚠️ Post-monomorphization
===

<!--
speaker_note: |
  Post-monomorphization errors appear when generic code is instantiated
  during codegen, and cargo check doesn't do codegen.
  This is real output: check says fine, build fails. I verified it on the
  current toolchain.
  [click through the bullets]
  - rust-analyzer runs check, so no red squiggle in your editor.
  - A library's own tests only instantiate what they use, so misuse
    surfaces in the downstream crate.
  - Uninstantiated generic code is never checked at all.
  - Still much better than a runtime panic.
-->

That error only exists once `RingBuffer::<u8, 3>::new` is **instantiated**,
which happens during codegen.

```bash
$ cargo check
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s

$ cargo build
error[E0080]: evaluation panicked: N must be a power of two
```

<!-- pause -->

<!-- incremental_lists: true -->

* Your editor (rust-analyzer runs `check`) won't show it
* A library's own `cargo test` won't catch misuse by its downstream crates
* Generic code that's never called is never checked
* Still beats a runtime panic in prod

<!-- end_slide -->

Validated constants
===

<!--
speaker_note: |
  const fn plus panic gives you a parser that runs in the compiler. Inside
  const fn: while loops, match on bytes, panic with a literal message. No
  iterators, no ?, no trait methods (yet).
  The payoff is the last two lines. A valid literal becomes a u32 at compile
  time with zero runtime cost. A typo is a compile error pointing at the
  constant. The same function still works at runtime with dynamic input,
  panicking there instead. Think of it as compile-time validation for
  config, colors, IP addresses, regex-ish formats.
-->

`const fn` + `panic!` = a parser that runs in the compiler:

```rust
pub const fn hex_color(s: &str) -> u32 {
    let bytes = s.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        panic!("expected a color like \"#rrggbb\"");
    }
    let (mut value, mut i) = (0u32, 1);
    while i < bytes.len() {
        let digit = match bytes[i] {
            b @ b'0'..=b'9' => b - b'0',
            b @ b'a'..=b'f' => b - b'a' + 10,
            _ => panic!("invalid hex digit in color"),
        };
        value = (value << 4) | digit as u32;
        i += 1;
    }
    value
}

const BRAND: u32 = hex_color("#c0ffee");   // ✓
const OOPS:  u32 = hex_color("#c0ffeg");   // compile error
```

<!-- end_slide -->

<!-- jump_to_middle -->

🧪 Exercise 06: `exercises/06-const-assertions`
===

<!--
speaker_note: |
  Exercise (~10 to 15 min). Explain compile_fail doctests: they pass only
  if the snippet fails to compile. A no-op stub makes them fail, which is
  the point.
  Sticking points:
  - assert_impl! without ?Sized fails on str.
  - The ring buffer check must be an inline const inside new(), not a
    const item, which can't see N.
  - Task 5 has no tests: assert the Header layout, then reorder the fields
    and watch it fail.
-->

`const_assert!`, `assert_impl!`, a power-of-two `RingBuffer`, and `hex_color`.
Several tests are `compile_fail` doctests: they pass when your code
*refuses to build*.

<!-- end_slide -->

The spellbook
===

<!--
speaker_note: |
  ~3 min wrap-up. Go row by row, one sentence each: what the trick is,
  and where people have already met it without knowing.
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
| handler trait | tuples + macros + markers + erasure | `axum`, `bevy`, `actix` |
| const assertions | `const _` / `const {}` panics | `static_assertions`, everywhere |

<!-- pause -->

**With great power:** each of these costs you some readability. Reach for them
in library code, where one maintainer pays so a thousand users don't.

<!-- end_slide -->

Further reading
===

<!--
speaker_note: |
  Point people to the sources, especially dtolnay's case studies and
  Rain's repo, which is a great model of literate Rust.
  Mention the exercises stay in the repo: the solution feature lets
  anyone check their work later.
  Take questions. Likely ones:
  - "Is autoref specialization going away?" No, it relies on documented
    method resolution. But real specialization would make it unnecessary.
  - "Does axum still work this way?" Yes, axum 0.8's Handler<T, S> has the
    same shape, plus async and state.
  - "Should I always use hashbrown's Equivalent?" Not always. Yes for
    internal hash maps on a hot path, or if you already use hashbrown or
    indexmap. No when std's HashMap is in your public API (switching
    map types is a breaking change) or you need a BTreeMap, which has no
    equivalent mechanism. Not hot? Just allocate. Either way the
    consistency contract still applies: the proptest works unchanged.
    Bonus caveat: hashbrown's default hasher favours speed over HashDoS
    resistance, so pass std's RandomState if keys come from untrusted input.
-->

* dtolnay, _Autoref-based stable specialization_: `github.com/dtolnay/case-studies`
* dtolnay, _The semver trick_: `github.com/dtolnay/semver-trick`
* The Rust Reference: _Method call expressions_, _Type coercions_
* `axum/src/handler/mod.rs` and `axum-core/src/extract/mod.rs`
* `bevy_ecs/src/system/function_system.rs`
* Rain, _Implementing Borrow for complex keys_:
  `github.com/sunshowers-code/borrow-complex-key-example`
* Ivan Dubrov, _Tricking the HashMap_: `idubrov.name/rust/2018/06/01/tricking-the-hashmap.html`
* `std::borrow::Borrow` docs (the `Hash`/`Eq`/`Ord` contract)
* `hashbrown::Equivalent`

<!-- new_lines: 2 -->

<!-- alignment: center -->

**Thanks! 🦀✨**
