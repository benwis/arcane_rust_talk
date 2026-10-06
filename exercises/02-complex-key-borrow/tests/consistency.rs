//! `Borrow` comes with a contract the compiler can't check: `Eq`, `Ord` and
//! `Hash` must agree between the owned type and what it borrows as. Those are
//! *properties*, so check them with property-based tests.

use complex_key_borrow::{Key, OwnedKey};
use proptest::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_output(x: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

fn owned_key() -> impl Strategy<Value = OwnedKey> {
    // Short strings over a tiny alphabet, so that equal keys and shared
    // prefixes actually come up.
    ("[ab]{0,3}", prop::collection::vec(0u8..3, 0..3))
        .prop_map(|(s, bytes)| OwnedKey { s, bytes })
}

proptest! {
    #[test]
    fn consistent_borrow(owned1 in owned_key(), owned2 in owned_key()) {
        let borrowed1: &dyn Key = &owned1;
        let borrowed2: &dyn Key = &owned2;

        prop_assert_eq!(owned1 == owned2, borrowed1 == borrowed2, "consistent Eq");
        prop_assert_eq!(
            owned1.partial_cmp(&owned2),
            borrowed1.partial_cmp(borrowed2),
            "consistent PartialOrd"
        );
        prop_assert_eq!(owned1.cmp(&owned2), borrowed1.cmp(borrowed2), "consistent Ord");
        prop_assert_eq!(hash_output(&owned1), hash_output(borrowed1), "consistent Hash");
        prop_assert_eq!(hash_output(&owned2), hash_output(borrowed2), "consistent Hash");
    }
}
