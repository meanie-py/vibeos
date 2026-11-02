//! `ASSERT` / `ASSERTMSG` for translated crates: the kernel's convention, shared.
//!
//! ntoskrnl's own `ASSERT!` is `debug_assert!` -- a real check in a debug build,
//! compiled out of the release build the gate ships. Thirteen other crates spelled
//! it as an EMPTY FUNCTION instead (`fn ASSERT(_cond: i32) {}`), which is wrong both
//! ways: it never checks, yet its argument is still EVALUATED on every call, where the
//! C's `((VOID)0)` evaluates nothing. That eager evaluation is the class that read a
//! NULL `TransferBuffer` at DISPATCH_LEVEL in usbport (via an empty `DPRINT_URB`).
//!
//! `debug_assert!` type-checks its expression in every build and evaluates it only
//! when debug assertions are on, so it is the faithful shape for both builds.
//! C asserts take a `BOOLEAN`, an integer, a pointer or a real `bool`; `Truthy` gives
//! them the C meaning of "non-zero / non-null" without a cast at each call site.

pub trait Truthy {
    fn truthy(self) -> bool;
}

impl Truthy for bool {
    #[inline(always)]
    fn truthy(self) -> bool {
        self
    }
}

macro_rules! truthy_int {
    ($($t:ty),*) => { $(
        impl Truthy for $t {
            #[inline(always)]
            fn truthy(self) -> bool { self != 0 }
        }
    )* };
}
truthy_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl<T: ?Sized> Truthy for *const T {
    #[inline(always)]
    fn truthy(self) -> bool {
        !self.is_null()
    }
}

impl<T: ?Sized> Truthy for *mut T {
    #[inline(always)]
    fn truthy(self) -> bool {
        !self.is_null()
    }
}

#[inline(always)]
pub fn truthy<T: Truthy>(v: T) -> bool {
    v.truthy()
}

/// `ASSERT(x)`. Define it in a crate as
/// `macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }`.
///
/// Armed (`debug-assertions` on for the CALLING crate -- `cfg!` expands at the call
/// site), a failure reports through `RtlAssert` and CONTINUES, as the C's DBG build does.
/// It used to be `debug_assert!`, which panics: harmless while every user was compiled
/// out, but arming a crate would have turned each C diagnostic into a halt.
#[macro_export]
macro_rules! nt_assert {
    ($e:expr $(,)?) => {
        if cfg!(debug_assertions) && !$crate::dbgassert::truthy($e) {
            #[allow(unused_unsafe)]
            unsafe {
                $crate::ddk::RtlAssert(
                    concat!(stringify!($e), "\0").as_ptr() as *mut ::core::ffi::c_void,
                    concat!(file!(), "\0").as_ptr() as *mut ::core::ffi::c_void,
                    line!(),
                    ::core::ptr::null_mut::<i8>(),
                );
            }
        }
    };
}

/// `ASSERTMSG(msg, x)`: the message is the C's, the check is `nt_assert!`'s.
#[macro_export]
macro_rules! nt_assertmsg {
    ($msg:expr, $e:expr $(,)?) => {
        if cfg!(debug_assertions) && !$crate::dbgassert::truthy($e) {
            #[allow(unused_unsafe)]
            unsafe {
                $crate::ddk::RtlAssert(
                    concat!(stringify!($e), "\0").as_ptr() as *mut ::core::ffi::c_void,
                    concat!(file!(), "\0").as_ptr() as *mut ::core::ffi::c_void,
                    line!(),
                    ($msg) as *const i8 as *mut i8,
                );
            }
        }
    };
}

/// SAL and WPP annotations that expand to NOTHING in this build: no_sal2.h /
/// concurrencysal.h / specstrings.h (no `_PREFAST_`), storswtr.h (no WPP). Their
/// operands are never evaluated, so these discard their tokens -- an empty `fn` would
/// evaluate every one of them, out of line.
#[macro_export]
macro_rules! _Analysis_assume_ { ($($t:tt)*) => { () }; }
#[macro_export]
macro_rules! _Analysis_assume_lock_held_ { ($($t:tt)*) => { () }; }
#[macro_export]
macro_rules! _Analysis_assume_lock_not_held_ { ($($t:tt)*) => { () }; }
#[macro_export]
macro_rules! __analysis_assert { ($($t:tt)*) => { () }; }
#[macro_export]
macro_rules! WPP_INIT_TRACING { ($($t:tt)*) => { () }; }
#[macro_export]
macro_rules! WPP_CLEANUP { ($($t:tt)*) => { () }; }
