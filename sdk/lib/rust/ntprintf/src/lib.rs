//! Rust implementation of Windows-flavored printf for kernel use.
//!
//! Replaces the `_vsnprintf` calls inside the C variadic shims (KdbPrintf,
//! KdbpPrint, KdpDprintf, RtlStringCbPrintfA/W). The C trampoline collects
//! varargs into a typed slot array, then this crate does all interpretation.
//!
//! Two-call FFI:
//!  1. [`ntprintf_scan_arg_types`] — given a format string, write one byte per
//!     `%` specifier into `out_types`, return the count. The C trampoline uses
//!     this to know which `va_arg(ap, T)` width to pull for each slot.
//!  2. [`ntprintf_format`] — given the same format string + a `[u64; N]` slot
//!     array (filled by the trampoline) + an output writer callback, emit the
//!     formatted bytes. Returns the number of bytes emitted.
//!
//! No stdlib at runtime. `libc` is only used in dev-dependencies to diff
//! against `vsnprintf` for validation.

#![cfg_attr(not(test), no_std)]

mod scanner;
mod formatter;

pub use scanner::{ArgType, scan_format};
pub use formatter::{Writer, BoundedBuf, format_into};

/// FFI: scan a NUL-terminated format string and emit one [`ArgType`] tag per
/// `%` specifier. `out_types` must point at a buffer of at least `max` bytes.
/// Returns the number of tags written, or -1 if `max` was exceeded.
///
/// Tag bytes (one per `%` specifier, in order):
///   `d` = i32       — pull 4 bytes (int-promoted).
///   `u` = u32       — pull 4 bytes (int-promoted).
///   `D` = i64       — pull 8 bytes (long long / __int64).
///   `U` = u64       — pull 8 bytes (long long / __int64).
///   `p` = pointer   — pull native pointer width (4 or 8).
///   `*` = width-or-precision prefix int (always 4 bytes), followed by the
///         tag for the actual specifier.
///
/// Width/precision `*` slots are tagged `*` so the C trampoline pulls them
/// in order alongside the value slots.
#[no_mangle]
pub unsafe extern "C" fn ntprintf_scan_arg_types(
    fmt: *const u8,
    out_types: *mut u8,
    max: usize,
) -> i32 {
    if fmt.is_null() || out_types.is_null() || max == 0 {
        return -1;
    }
    let fmt_slice = cstr_to_slice(fmt);
    let mut written = 0usize;
    let result = scan_format(fmt_slice, |t| {
        if written >= max {
            return false;
        }
        unsafe { *out_types.add(written) = t.tag(); }
        written += 1;
        true
    });
    if result.is_err() {
        return -1;
    }
    written as i32
}

/// FFI: format `fmt` using `slots[0..n]` as the typed-arg array, write bytes
/// into `dst[0..cap]`. Always NUL-terminates if `cap > 0`. Returns the number
/// of bytes that *would* have been written excluding the NUL — matching
/// `_vsnprintf` semantics (truncated == returned >= cap).
///
/// Slot layout: each slot is 8 bytes raw. The trampoline sign/zero-extends
/// 32-bit pulls into the low 32 bits of the slot. The Rust formatter
/// re-narrows based on the tag from a parallel scan.
#[no_mangle]
pub unsafe extern "C" fn ntprintf_format_a(
    dst: *mut u8,
    cap: usize,
    fmt: *const u8,
    slots: *const u64,
    n: usize,
) -> usize {
    if fmt.is_null() {
        return 0;
    }
    let fmt_slice = cstr_to_slice(fmt);
    let slot_slice = if slots.is_null() || n == 0 {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(slots, n) }
    };
    let mut writer = BoundedBuf::new(dst, cap);
    let written = format_into(&mut writer, fmt_slice, slot_slice);
    writer.terminate();
    written
}

/// Walk a NUL-terminated C string into a Rust slice (no allocation).
/// Capped at 64 KiB to avoid unbounded scans on a runaway pointer.
#[inline]
unsafe fn cstr_to_slice<'a>(p: *const u8) -> &'a [u8] {
    let mut len = 0usize;
    while len < 0x10000 {
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }
    unsafe { core::slice::from_raw_parts(p, len) }
}
