// ---------------------------------------------------------------------------
// Task 1: `const_assert!(cond)` and `const_assert!(cond, "message")`.
//
// Must work at item level (outside any function). Hint: an unnamed constant,
// `const _: () = ...;`, is evaluated even though nothing ever uses it, and
// `assert!` has worked in const contexts since Rust 1.57.
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! const_assert {
    ($($tt:tt)*) => {};
}

// ---------------------------------------------------------------------------
// Task 2: `assert_impl!(Type: Trait + OtherTrait)`.
//
// Hint: declare a function with the bounds you want to check, and mention
// it applied to the type somewhere the compiler has to type-check but never
// runs. Remember `?Sized`, or `assert_impl!(str: Send)` won't work.
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! assert_impl {
    ($($tt:tt)*) => {};
}

// ---------------------------------------------------------------------------
// Task 3: `RingBuffer` indexes with `& (N - 1)` instead of `% N`, which is
// only correct when `N` is a power of two. Make `RingBuffer::<_, 3>::new()`
// a compile error.
//
// A `const _: () = ...` item can't see `N`, since items don't inherit
// generics. An inline `const { ... }` block inside `new` can.
// ---------------------------------------------------------------------------

pub struct RingBuffer<T, const N: usize> {
    slots: [Option<T>; N],
    next: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        // TODO: reject N that isn't a power of two.
        Self {
            // Inline const as an array repeat operand: works even though
            // `Option<T>` isn't `Copy`.
            slots: [const { None }; N],
            next: 0,
        }
    }

    /// Overwrites the oldest element once full.
    pub fn push(&mut self, value: T) {
        self.slots[self.next & (N - 1)] = Some(value);
        self.next += 1;
    }

    pub fn len(&self) -> usize {
        self.next.min(N)
    }

    pub fn is_empty(&self) -> bool {
        self.next == 0
    }

    /// `get(0)` is the oldest element still in the buffer.
    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len() {
            return None;
        }
        let oldest = self.next - self.len();
        self.slots[(oldest + i) & (N - 1)].as_ref()
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Task 4: `hex_color("#rrggbb") -> u32`, usable in `const` items, where a bad
// literal is a compile error.
//
// You can't use iterators, `?`, `str::parse` or most trait methods in a
// `const fn`. You *can* use `s.as_bytes()`, `while` loops, `match` on bytes
// and `panic!("literal message")`.
// ---------------------------------------------------------------------------

pub const fn hex_color(s: &str) -> u32 {
    let _ = s;
    todo!()
}

// ---------------------------------------------------------------------------
// Task 5 (no tests): pin down a wire format. Once Task 1 works, assert that
// `Header` is 8 bytes, 4-aligned, and `flags` is at offset 6
// (`core::mem::offset_of!`). Then try reordering the fields.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,
}
