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

// ============================================================================
// Windows: defining a C `...` entry point WITHOUT a C file.
//
// The two gaps above are only gaps in GENERAL. On the three Windows ABIs this tree builds,
// a `va_list` is a `char*` into the argument list itself, and a variadic function's
// arguments can be made contiguous in memory by one fixed spill that does not depend on the
// function's arity:
//
// * i386 (cdecl): every argument is already on the stack, from [esp + 4].
// * x86_64 (Microsoft x64): the first four travel in rcx/rdx/r8/r9 and the caller reserves
//   their 32-byte home area right below the stack arguments; storing all four into it makes
//   the whole list contiguous from [rsp + 8]. (Variadic doubles are passed in the integer
//   registers too, so this covers them.)
// * aarch64 (Windows ARM64, variadic convention): x0-x7 then the stack, floating point in
//   the integer registers too; x0-x7 stored in the 64 bytes just below the caller's stack
//   arguments make the list contiguous.
//
// So a naked thunk that performs that spill and calls `helper(&first_argument)` serves every
// `...` export, and the helper reads its fixed arguments from the slots ([`VaSlots::get`])
// and passes [`VaSlots::va`] wherever the C passed its `va_list`. [`va_entry!`] writes both.
// ============================================================================

/// The argument list of a C `...` function, laid out contiguously by [`va_entry!`]'s thunk:
/// one pointer-width slot per fixed argument, then the variadic ones.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VaSlots(pub *const usize);

#[cfg(target_os = "windows")]
impl VaSlots {
    /// Fixed argument `i`, as its whole slot. Every fixed parameter of the CRT's `...`
    /// exports is a pointer, an `int` or a `size_t`, each of which occupies exactly one
    /// pointer-width slot (an `int` in the low half on 64-bit); convert with `as _` at the
    /// call, which truncates an `int` to its 32 bits as the callee would.
    ///
    /// # Safety
    /// `i` must be below the function's fixed-argument count.
    #[inline(always)]
    pub unsafe fn get(self, i: usize) -> usize {
        *self.0.add(i)
    }

    /// The `va_list` for the variadic arguments that follow `n` fixed ones (`va_start` on
    /// the last fixed parameter): on these ABIs a plain pointer to them.
    #[inline(always)]
    pub fn va(self, n: usize) -> *mut c_void {
        self.0.wrapping_add(n) as *mut c_void
    }
}

/// `va_entry!(export => |a| -> Ret { body })` defines the C-variadic export `export`: a naked
/// thunk (see the section comment) that hands `body` the argument list as `a: VaSlots`.
/// `body` runs in a private module named after the export, which glob-imports the caller's
/// module, so it sees the same items the caller does.
///
/// ```ignore
/// // int sprintf(char *str, const char *format, ...)
/// va_entry!(sprintf => |a| -> i32 { _vsnprintf(a.get(0) as _, i32::MAX as _, a.get(1) as _, a.va(2) as _) });
/// ```
#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! va_entry {
    ($export:ident => |$a:ident| -> $ret:ty $body:block) => {
        #[doc(hidden)]
        #[allow(non_snake_case, unused_imports)]
        mod $export {
            use super::*;
            #[inline(never)]
            pub(super) unsafe extern "C" fn body($a: $crate::VaSlots) -> $ret $body
        }
        $crate::va_entry_thunk!($export, $export::body);
    };
}

#[cfg(all(target_os = "windows", target_arch = "x86"))]
#[doc(hidden)]
#[macro_export]
macro_rules! va_entry_thunk {
    ($export:ident, $helper:path) => {
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            core::arch::naked_asm!(
                "lea eax, [esp + 4]",
                "push eax",
                "call {h}",
                "add esp, 4",
                "ret",
                h = sym $helper,
            );
        }
    };
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
#[doc(hidden)]
#[macro_export]
macro_rules! va_entry_thunk {
    ($export:ident, $helper:path) => {
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            // The calls out carry SEH unwind directives: rustc emits none for a naked
            // function, and an exception unwinding through a frame without .pdata is lost.
            core::arch::naked_asm!(
                concat!(".seh_proc ", stringify!($export)),
                "mov [rsp + 8], rcx",
                "mov [rsp + 16], rdx",
                "mov [rsp + 24], r8",
                "mov [rsp + 32], r9",
                "sub rsp, 40",
                ".seh_stackalloc 40",
                ".seh_endprologue",
                "lea rcx, [rsp + 48]",
                "call {h}",
                "add rsp, 40",
                "ret",
                ".seh_endproc",
                h = sym $helper,
            );
        }
    };
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
#[doc(hidden)]
#[macro_export]
macro_rules! va_entry_thunk {
    ($export:ident, $helper:path) => {
        #[unsafe(naked)]
        #[no_mangle]
        pub unsafe extern "C" fn $export() {
            core::arch::naked_asm!(
                concat!(".seh_proc ", stringify!($export)),
                // 16 bytes fp/lr + x0..x7 (64), ending at the caller's stack arguments.
                "stp x29, x30, [sp, #-80]!",
                ".seh_save_fplr_x 80",
                "mov x29, sp",
                ".seh_set_fp",
                ".seh_endprologue",
                "stp x0, x1, [sp, #16]",
                "stp x2, x3, [sp, #32]",
                "stp x4, x5, [sp, #48]",
                "stp x6, x7, [sp, #64]",
                "add x0, sp, #16",
                "bl {h}",
                ".seh_startepilogue",
                "ldp x29, x30, [sp], #80",
                ".seh_save_fplr_x 80",
                ".seh_endepilogue",
                "ret",
                ".seh_endproc",
                h = sym $helper,
            );
        }
    };
}

/// The six `vibeos_va_arg_*` pullers in Rust, for a module that defines its `...` entries with
/// [`va_entry!`] and so has no `rust_varargs_shim.c` to supply them. Windows `va_arg`: read the
/// value at the pointer, advance by its size rounded up to the pointer width (i386: 4 for
/// `int`/pointers, 8 for `long long`/`double`, unaligned; 64-bit: 8 for every type here).
#[cfg(all(feature = "rust-pullers", target_os = "windows"))]
mod pullers {
    use core::ffi::c_void;
    use core::mem::size_of;

    #[inline(always)]
    unsafe fn pull<T: Copy>(app: *mut c_void) -> T {
        let p = app as *mut *mut u8;
        let v = (*p as *const T).read_unaligned();
        let w = size_of::<usize>();
        *p = (*p).add((size_of::<T>() + w - 1) & !(w - 1));
        v
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_i32(app: *mut c_void) -> i32 {
        pull(app)
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_u32(app: *mut c_void) -> u32 {
        pull(app)
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_i64(app: *mut c_void) -> i64 {
        pull(app)
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_u64(app: *mut c_void) -> u64 {
        pull(app)
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_f64(app: *mut c_void) -> f64 {
        pull(app)
    }
    #[no_mangle]
    pub unsafe extern "C" fn vibeos_va_arg_ptr(app: *mut c_void) -> *mut c_void {
        pull(app)
    }
}
