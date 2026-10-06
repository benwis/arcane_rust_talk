use complex_key_borrow::{BorrowedKey, Key, OwnedKey};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

fn owned(s: &str, bytes: &[u8]) -> OwnedKey {
    OwnedKey {
        s: s.to_owned(),
        bytes: bytes.to_vec(),
    }
}

#[test]
fn the_simple_case() {
    // String: Borrow<str>, so a &str works for lookups.
    let mut set: HashSet<String> = HashSet::new();
    set.insert("example-string".to_owned());
    assert!(set.contains("example-string"));
}

#[test]
fn hash_set() {
    let mut set: HashSet<OwnedKey> = HashSet::new();
    set.insert(owned("foo", b"abc"));

    let hit = BorrowedKey { s: "foo", bytes: b"abc" };
    let miss = BorrowedKey { s: "foo", bytes: b"abd" };
    // On current compilers the coercion to `&dyn Key` has to be spelled out:
    // with `Q` still unknown, there's nothing to coerce *to*.
    assert!(set.contains(&hit as &dyn Key));
    assert!(!set.contains(&miss as &dyn Key));
}

#[test]
fn hash_map() {
    let mut map: HashMap<OwnedKey, u32> = HashMap::new();
    map.insert(owned("foo", b"abc"), 1);
    map.insert(owned("bar", b""), 2);

    let key = BorrowedKey { s: "bar", bytes: b"" };
    assert_eq!(map.get(&key as &dyn Key), Some(&2));
    assert_eq!(map.remove(&key as &dyn Key), Some(2));
    assert_eq!(map.len(), 1);
}

#[test]
fn btree_set_and_map() {
    let set: BTreeSet<OwnedKey> = [owned("a", b"1"), owned("b", b"2")].into();
    assert!(set.contains(&BorrowedKey { s: "b", bytes: b"2" } as &dyn Key));
    assert!(!set.contains(&BorrowedKey { s: "b", bytes: b"1" } as &dyn Key));

    let mut map: BTreeMap<OwnedKey, u32> = BTreeMap::new();
    map.insert(owned("a", b"1"), 10);
    let key = BorrowedKey { s: "a", bytes: b"1" };
    assert_eq!(map.get(&key as &dyn Key), Some(&10));
}

// --- The point of the exercise: lookups that don't allocate. ---------------

// Counts allocations per thread, so tests running in parallel don't see each
// other's allocations.
struct Counting;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn allocations_during<R>(f: impl FnOnce() -> R) -> (R, usize) {
    let before = ALLOCATIONS.with(Cell::get);
    let result = f();
    let after = ALLOCATIONS.with(Cell::get);
    (result, after - before)
}

#[test]
fn lookups_do_not_allocate() {
    let hashed: HashSet<OwnedKey> = [owned("foo", b"abc")].into();
    let ordered: BTreeSet<OwnedKey> = [owned("foo", b"abc")].into();
    let key = BorrowedKey { s: "foo", bytes: b"abc" };

    let (found, allocs) = allocations_during(|| hashed.contains(&key as &dyn Key));
    assert!(found);
    assert_eq!(allocs, 0, "HashSet lookup allocated");

    let (found, allocs) = allocations_during(|| ordered.contains(&key as &dyn Key));
    assert!(found);
    assert_eq!(allocs, 0, "BTreeSet lookup allocated");
}
