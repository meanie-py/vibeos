//! Width-generic support for the `SetFlag!` / `ClearFlag!` macros (lib.rs). C's
//! `x |= f` converts `f` to the type of `x`; `as _` cannot express that through a
//! compound assignment, so both sides go through `u64` and back, truncating like C.

pub(crate) trait Flag: Copy {
    fn bits(self) -> u64;
    fn from_bits(b: u64) -> Self;
}

macro_rules! flag_int {
    ($($t:ty),*) => {$(
        impl Flag for $t {
            #[inline(always)]
            fn bits(self) -> u64 {
                self as u64
            }
            #[inline(always)]
            fn from_bits(b: u64) -> Self {
                b as $t
            }
        }
    )*};
}
flag_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[inline(always)]
pub(crate) unsafe fn set<T: Flag, F: Flag>(x: *mut T, f: F) {
    *x = T::from_bits((*x).bits() | f.bits());
}

#[inline(always)]
pub(crate) unsafe fn clear<T: Flag, F: Flag>(x: *mut T, f: F) {
    *x = T::from_bits((*x).bits() & !f.bits());
}
