//! Receiving C varargs on stable Rust, ported ONCE.
//!
//! Stable Rust cannot DEFINE a C-variadic (`c_variadic` is nightly), so
//! the tree's printf-family entry points (freeldr's TuiPrintf/printf/DbgPrint,
//! ntoskrnl's KdIoPrintf, classpnp's TracePrint) used to print their format
//! string LITERALLY -- every `%s`/`%lu` came out as the two characters, when it
//! came out at all.
//!
//! We cannot *define* a variadic, but we can *receive* one. On all three of our
//! targets a variadic call places arguments in a flat, ascending slot sequence
//! (i386 cdecl: pushed on the stack, caller-pop; win64: RCX/RDX/R8/R9 then
//! stack; ARM64: x0-x7 then stack), and -- the fact that makes this work --
//! `va_list` is `*mut c_char` on all three (bindgen_{i386,amd64,arm64}.rs), a
//! plain pointer walked upward. Nothing about the SysV x86-64 register-save-area
//! va_list applies here.
//!
//! So: declare the trailing slots as ordinary fixed parameters, copy them into a
//! local array, and hand that array's address to libcntpr's `_vsnprintf` as the
//! va_list. The callee's declared arity does not affect a cdecl/win64 caller, so
//! the C headers stay variadic and not one call site changes.
//!
//! Two bounds, both measured rather than guessed (survey over every ERR/WARN/
//! TRACE/FIXME/DPRINT/printf format literal in boot/freeldr + freeldr-rust):
//!
//!   VA_SLOTS = 12   The widest live format needs 11 slots
//!                   ("VolumeLabel: %c%c%c%c%c%c%c%c%c%c%c", fat.c). Zero sites
//!                   need more. Counted in SLOTS, not specifiers, so an i386
//!                   `%I64x` is the 2 it really occupies. Capturing 12 reads at
//!                   most one slot past what a caller pushed; those loads land
//!                   in the caller's own frame (always mapped) and are never
//!                   consumed, because the format string is what decides how far
//!                   `_vsnprintf` walks.
//!
//!   VA_STAGE = 32   The array handed to `_vsnprintf` is deliberately LARGER
//!                   than what we capture, with the tail zeroed. If someone
//!                   later writes a 20-specifier format, it reads zeros --
//!                   printing `0` and `(null)` (streamout.c:537) -- instead of
//!                   walking off the end of the array into this function's own
//!                   frame. Degrades, never corrupts.

#![no_std]
#![allow(non_snake_case)]

extern "C" {
    fn _vsnprintf(buffer: *mut i8, count: usize, format: *const i8, ap: *mut core::ffi::c_void)
        -> i32;
}

pub const VA_SLOTS: usize = 12;
pub const VA_STAGE: usize = 32;

/// The captured variadic slot sequence, in call order.
pub type VaSlots = [usize; VA_SLOTS];

/// Format `fmt` into `buf` using `slots` as the variadic argument sequence, and
/// return `_vsnprintf`'s own value verbatim -- including `-1` for truncation, so
/// each caller can apply the C semantics of the function it is standing in for.
/// Does NOT NUL-terminate; see `format_slots_z` for the callers that need it.
pub unsafe fn format_slots(
    buf: *mut i8,
    count: usize,
    fmt: *const i8,
    slots: &VaSlots,
) -> i32 {
    let mut stage: [usize; VA_STAGE] = [0; VA_STAGE];
    stage[..VA_SLOTS].copy_from_slice(slots);
    _vsnprintf(buf, count, fmt, stage.as_mut_ptr() as *mut core::ffi::c_void)
}

/// The captured slots as an argument area a `va_list` can point at, for a consumer
/// that takes the `va_list` itself rather than formatting through `_vsnprintf`
/// (shell32's `ShellMessageBoxA` hands it to `FormatMessageA`, whose `Arguments`
/// is a `va_list *`). Staged exactly as `format_slots` does: the slots, then zeros.
/// The caller keeps the array alive for as long as the `va_list` is in use.
pub fn stage(slots: &VaSlots) -> [usize; VA_STAGE] {
    let mut stage: [usize; VA_STAGE] = [0; VA_STAGE];
    stage[..VA_SLOTS].copy_from_slice(slots);
    stage
}

/// `format_slots` for callers that hand the buffer to a string printer rather
/// than to a counted loop: always NUL-terminates, because `_vsnprintf` does not
/// terminate on truncation.
pub unsafe fn format_slots_z(buf: &mut [i8], fmt: *const i8, slots: &VaSlots) {
    let n = buf.len();
    format_slots(buf.as_mut_ptr(), n - 1, fmt, slots);
    buf[n - 1] = 0;
}
