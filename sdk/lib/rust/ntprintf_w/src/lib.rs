//! UTF-16 sibling of `ntprintf`. Replaces `_vsnwprintf` calls inside the C
//! variadic shims `RtlStringCbPrintfW` and `swprintf`.
//!
//! Same two-call FFI shape as the ANSI crate, but format string and output
//! buffer are `*const u16` / `*mut u16`. Slot ABI is identical (8-byte
//! `u64` slots), so the C trampoline can reuse its `va_arg` pull loop.
//!
//! ## Specifier semantics differ from ANSI
//!
//! In a wide format string, the meaning of `%s`/`%S`/`%c`/`%C` *flips*:
//!
//! | Spec   | A fmt (`ntprintf`) | W fmt (`ntprintf_w`) |
//! |--------|--------------------|----------------------|
//! | `%s`   | PCSTR              | **PCWSTR**           |
//! | `%S`   | PCWSTR             | **PCSTR**            |
//! | `%hs`  | PCSTR              | PCSTR                |
//! | `%ls`  | PCWSTR             | PCWSTR               |
//! | `%c`   | char               | **wchar (16-bit)**   |
//! | `%C`   | wchar              | **char**             |
//! | `%hc`  | char               | char                 |
//! | `%lc`  | wchar              | wchar                |
//! | `%wZ`  | UNICODE_STRING     | UNICODE_STRING       |
//!
//! This is Microsoft's actual rule: the *unprefixed* `s`/`c` follows the
//! caller's "width", which is determined by the format-string char width.
//!
//! ## Output sizing
//!
//! The `cap` parameter is **wide-character count**, not byte count.
//! `RtlStringCbPrintfW`'s `cb` is bytes; the C trampoline divides by 2
//! before calling `ntprintf_w_format`.

#![cfg_attr(not(test), no_std)]

mod scanner;
mod formatter;

pub use scanner::{ArgType, scan_format};
pub use formatter::{WriterW, BoundedBufW, format_into};

/// FFI: scan a NUL-terminated wide format string and emit one tag per `%`
/// specifier. Tag bytes match the ANSI scanner's contract — the C trampoline
/// can use the same `va_arg` pull loop for both ANSI and wide format strings.
#[no_mangle]
pub unsafe extern "C" fn ntprintf_w_scan_arg_types(
    fmt: *const u16,
    out_types: *mut u8,
    max: usize,
) -> i32 {
    if fmt.is_null() || out_types.is_null() || max == 0 {
        return -1;
    }
    let fmt_slice = wcstr_to_slice(fmt);
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

/// FFI: format `fmt` (wide) using `slots[0..n]`, write into `dst[0..cap_chars]`
/// (also wide). Always NUL-terminates if `cap_chars > 0`. Returns the number of
/// wide characters that *would* have been written excluding the NUL —
/// matching `_vsnwprintf` semantics (truncated == returned >= cap).
#[no_mangle]
pub unsafe extern "C" fn ntprintf_w_format(
    dst: *mut u16,
    cap_chars: usize,
    fmt: *const u16,
    slots: *const u64,
    n: usize,
) -> usize {
    if fmt.is_null() {
        return 0;
    }
    let fmt_slice = wcstr_to_slice(fmt);
    let slot_slice = if slots.is_null() || n == 0 {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(slots, n) }
    };
    let mut writer = BoundedBufW::new(dst, cap_chars);
    let written = format_into(&mut writer, fmt_slice, slot_slice);
    writer.terminate();
    written
}

/// Walk a NUL-terminated UTF-16 string into a Rust slice (no allocation).
/// Capped at 64 Ki wide chars (128 KiB) to avoid unbounded scans on a
/// runaway pointer.
#[inline]
unsafe fn wcstr_to_slice<'a>(p: *const u16) -> &'a [u16] {
    let mut len = 0usize;
    while len < 0x10000 {
        if unsafe { *p.add(len) } == 0 {
            break;
        }
        len += 1;
    }
    unsafe { core::slice::from_raw_parts(p, len) }
}
