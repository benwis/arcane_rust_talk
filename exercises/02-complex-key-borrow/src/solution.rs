use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

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

// Step 1 -------------------------------------------------------------------

pub trait Key {
    fn key<'k>(&'k self) -> BorrowedKey<'k>;
}

// Step 2 -------------------------------------------------------------------

impl Key for OwnedKey {
    fn key<'k>(&'k self) -> BorrowedKey<'k> {
        BorrowedKey {
            s: self.s.as_str(),
            bytes: self.bytes.as_slice(),
        }
    }
}

impl<'a> Key for BorrowedKey<'a> {
    fn key<'k>(&'k self) -> BorrowedKey<'k> {
        // A copy with the shorter lifetime 'k. Allowed because `BorrowedKey`
        // is covariant in 'a.
        *self
    }
}

// Step 3 -------------------------------------------------------------------

impl<'a> Borrow<dyn Key + 'a> for OwnedKey {
    fn borrow(&self) -> &(dyn Key + 'a) {
        self
    }
}

// Steps 4 to 6 ---------------------------------------------------------------

impl<'a> PartialEq for dyn Key + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.key().eq(&other.key())
    }
}

impl<'a> Eq for dyn Key + 'a {}

impl<'a> PartialOrd for dyn Key + 'a {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for dyn Key + 'a {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl<'a> Hash for dyn Key + 'a {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}
