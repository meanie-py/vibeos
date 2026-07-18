//! `VaArgs` — read C varargs from stable Rust.
//!
//! # Why this exists
//!
//! Rust cannot, on the **stable** channel, do exactly two things:
//!
//!   1. **DEFINE** a C-variadic function (`fn f(fmt: *const i8, mut args: ...)`), and
//!   2. **`va_arg`** — pull the next argument out of a `va_list`.
//!
//! Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a
//! merge disposition, so this crate is expected to be retired when it stabilizes;
//! see `docs/engineering/active-lanes/stable-toolchain-plan.md`). *Calling* a
//! variadic C function and *declaring* one are already stable, so only the two
//! primitives above need help.
//!
//! # The shape
//!
//! A tiny per-module C file (`rust_varargs_shim.c`) supplies only those two
//! primitives — the variadic entry point (which is the sole place `va_start` can
//! legally appear) and a set of `va_arg` pullers. It immediately hands control back
//! to Rust. **All real logic stays in Rust** — the printf formatters, the strsafe
//! length/`NTSTATUS` semantics, the driver notification dispatch. This is a shim for
//! a language gap, not a translation being reverted to C.
//!
//! The C entry passes a **pointer to** its `va_list` (`&ap`), never the `va_list`
//! itself. That deliberately avoids assuming `va_list` is a single pointer: it is
//! `char*` on all three of our Windows targets, but is an array-of-struct on SysV
//! x86-64, and the by-pointer form is correct on both. (It is also what lets the
//! host test below exercise the real code path natively.)
//!
//! `VaArgs::arg::<T>()` mirrors the signature of nightly's `VaListImpl::arg::<T>()`
//! on purpose, so a translated formatter body ports by changing only its parameter
//! *type* — not a single line of its logic.
//!
//! # Contract (the sharp edges)
//!
//! * A `VaArgs` borrows the C frame's `va_list`. It must not outlive the call from
//!   the C entry point — never store one.
//! * Reading a type that does not match what the caller actually passed is UB,
//!   exactly as with C `va_arg`. Respect the **default argument promotions**: C
//!   promotes anything narrower than `int` to `int`, and `f32` to `f64`. So a `%c`
//!   or `%hd` argument must be pulled as `i32`/`u32`, never as `u8`/`i16`; a float
//!   must be pulled as `f64`. The `VaArg` impls below are deliberately limited to
//!   the already-promoted types so the wrong pull is hard to write by accident.

#![cfg_attr(not(test), no_std)]

use core::ffi::c_void;

extern "C" {
    // Supplied by the per-module `rust_varargs_shim.c`. Each takes a POINTER to the
    // C `va_list` and advances it, so successive pulls walk the argument list.
    fn vibeos_va_arg_i32(ap: *mut c_void) -> i32;
    fn vibeos_va_arg_u32(ap: *mut c_void) -> u32;
    fn vibeos_va_arg_i64(ap: *mut c_void) -> i64;
    fn vibeos_va_arg_u64(ap: *mut c_void) -> u64;
    fn vibeos_va_arg_f64(ap: *mut c_void) -> f64;
    fn vibeos_va_arg_ptr(ap: *mut c_void) -> *mut c_void;
}

/// An opaque handle to a C `va_list`, owned by the C frame that called into Rust.
///
/// Construct it only from the `*mut c_void` a `rust_varargs_shim.c` entry point
/// hands you (which is `&ap`, a pointer to that frame's `va_list`).
#[repr(transparent)]
pub struct VaArgs(*mut c_void);

impl VaArgs {
    /// Wrap the `va_list` pointer handed over by the C variadic entry point.
    ///
    /// # Safety
    /// `ap` must be the `void*` the C shim passed (i.e. `&ap` of a live, `va_start`ed
    /// `va_list`), and the resulting `VaArgs` must not outlive that C frame.
    #[inline]
    pub const unsafe fn from_ptr(ap: *mut c_void) -> Self {
        VaArgs(ap)
    }

    /// Pull the next argument. Mirrors nightly `VaListImpl::arg::<T>()`.
    ///
    /// # Safety
    /// `T` must match the type the caller actually passed, **after** C's default
    /// argument promotions (see the module docs). Wrong `T` is UB.
    #[inline]
    pub unsafe fn arg<T: VaArg>(&mut self) -> T {
        T::pull(self)
    }
}

/// A type that can be pulled from a `va_list`.
///
/// Deliberately implemented only for types that survive C's default argument
/// promotions, so `arg::<u8>()` — a classic `va_arg` footgun — will not compile.
pub trait VaArg {
    /// # Safety
    /// See [`VaArgs::arg`].
    unsafe fn pull(v: &mut VaArgs) -> Self;
}

macro_rules! impl_va_arg {
    ($($t:ty => $f:ident),* $(,)?) => {$(
        impl VaArg for $t {
            #[inline]
            unsafe fn pull(v: &mut VaArgs) -> $t { $f(v.0) as $t }
        }
    )*};
}

// `int`-class (C promotes anything narrower to `int`), the 64-bit class, and `double`.
impl_va_arg! {
    i32 => vibeos_va_arg_i32,
    u32 => vibeos_va_arg_u32,
    i64 => vibeos_va_arg_i64,
    u64 => vibeos_va_arg_u64,
    f64 => vibeos_va_arg_f64,
}

impl<T> VaArg for *const T {
    #[inline]
    unsafe fn pull(v: &mut VaArgs) -> *const T {
        vibeos_va_arg_ptr(v.0) as *const T
    }
}

impl<T> VaArg for *mut T {
    #[inline]
    unsafe fn pull(v: &mut VaArgs) -> *mut T {
        vibeos_va_arg_ptr(v.0) as *mut T
    }
}

// `usize`/`isize` are pointer-width: pull them at the width of the target so an
// `ULONG_PTR` argument round-trips on both 32- and 64-bit.
impl VaArg for usize {
    #[inline]
    unsafe fn pull(v: &mut VaArgs) -> usize {
        #[cfg(target_pointer_width = "64")]
        {
            vibeos_va_arg_u64(v.0) as usize
        }
        #[cfg(target_pointer_width = "32")]
        {
            vibeos_va_arg_u32(v.0) as usize
        }
    }
}

impl VaArg for isize {
    #[inline]
    unsafe fn pull(v: &mut VaArgs) -> isize {
        #[cfg(target_pointer_width = "64")]
        {
            vibeos_va_arg_i64(v.0) as isize
        }
        #[cfg(target_pointer_width = "32")]
        {
            vibeos_va_arg_i32(v.0) as isize
        }
    }
}
