use core::mem::{align_of, offset_of, size_of};

// Task 1 -------------------------------------------------------------------

#[macro_export]
macro_rules! const_assert {
    ($cond:expr $(,)?) => {
        const _: () = ::core::assert!($cond);
    };
    ($cond:expr, $msg:literal $(,)?) => {
        const _: () = ::core::assert!($cond, $msg);
    };
}

// Task 2 -------------------------------------------------------------------

#[macro_export]
macro_rules! assert_impl {
    ($ty:ty : $($bounds:tt)+) => {
        const _: fn() = || {
            fn check<T: ?Sized + $($bounds)+>() {}
            check::<$ty>();
        };
    };
}

// Task 3 -------------------------------------------------------------------

pub struct RingBuffer<T, const N: usize> {
    slots: [Option<T>; N],
    next: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        // Evaluated once per `N` this is instantiated with, during codegen.
        // Note: `cargo check` doesn't monomorphize, so it won't report this.
        // `cargo build` will.
        const { assert!(N.is_power_of_two(), "RingBuffer capacity must be a power of two") };
        Self {
            slots: [const { None }; N],
            next: 0,
        }
    }

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

// Task 4 -------------------------------------------------------------------

pub const fn hex_color(s: &str) -> u32 {
    let bytes = s.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        panic!("expected a color like \"#rrggbb\"");
    }
    let mut value = 0u32;
    let mut i = 1;
    while i < bytes.len() {
        let digit = match bytes[i] {
            b @ b'0'..=b'9' => b - b'0',
            b @ b'a'..=b'f' => b - b'a' + 10,
            b @ b'A'..=b'F' => b - b'A' + 10,
            _ => panic!("invalid hex digit in color"),
        };
        value = (value << 4) | digit as u32;
        i += 1;
    }
    value
}

// Task 5 -------------------------------------------------------------------

#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub version: u16,
    pub flags: u16,
}

crate::const_assert!(size_of::<Header>() == 8);
crate::const_assert!(align_of::<Header>() == 4);
crate::const_assert!(offset_of!(Header, flags) == 6, "flags moved: wire format broken");
crate::assert_impl!(Header: Send + Sync);
