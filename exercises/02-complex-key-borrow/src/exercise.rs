// ---------------------------------------------------------------------------
// Background: `HashSet<String>::contains` accepts a `&str` because
// `String: Borrow<str>`. For an owned type `O` and borrowed type `B`, `O` may
// implement `Borrow<B>` if:
//
//   * you can write `fn borrow(&self) -> &B`, and
//   * `Eq`, `Ord` and `Hash` are *consistent* between `O` and `B`: for all
//     `owned1`, `owned2`, comparing/hashing the owned values gives the same
//     results as comparing/hashing their borrowed forms.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OwnedKey {
    pub s: String,
    pub bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BorrowedKey<'a> {
    pub s: &'a str,
    pub bytes: &'a [u8],
}

// The first instinct doesn't work:
//
//     impl<'a> Borrow<BorrowedKey<'a>> for OwnedKey {
//         fn borrow(&self) -> &BorrowedKey<'a> {
//             // We need to return a *reference* to a BorrowedKey, but unlike
//             // String/str, there's no BorrowedKey hiding inside an OwnedKey.
//         }
//     }
//
// Instead, we use a trait object.

// ---------------------------------------------------------------------------
// Step 1 (given, so the tests can name it): a trait that produces the
// borrowed form of a key.
// ---------------------------------------------------------------------------

pub trait Key {
    fn key<'k>(&'k self) -> BorrowedKey<'k>;
}

// ---------------------------------------------------------------------------
// Step 2: implement `Key` for `OwnedKey` *and* for `BorrowedKey<'a>`.
//
// For `BorrowedKey<'a>`, `*self` is enough. Why can a `BorrowedKey<'a>`
// be returned as a `BorrowedKey<'k>`? (Hint: variance.)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Step 3: `impl<'a> Borrow<dyn Key + 'a> for OwnedKey`. The body is a
// coercion: `self`.
//
// Only the type *stored* in the collection needs the `Borrow` impl.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Steps 4 to 6: `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` for
// `dyn Key + 'a`, each delegating to `self.key()`.
//
// This is consistent with `OwnedKey`'s derives only because derived impls
// work field by field, in declaration order, and both structs declare their
// fields in the same order. `tests/consistency.rs` checks this with proptest.
// `Ord` is only needed for BTree collections, `Hash` only for hash ones.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Bonus, once everything passes:
//   * Swap the field order in `BorrowedKey` and rerun the tests. Which
//     properties break, and why does `Eq` survive?
//   * Does the trick work if the keys are enums?
// ---------------------------------------------------------------------------
