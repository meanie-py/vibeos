//! dbgprint-rust: the kernel's Rust-native debug-print infrastructure, hoisted
//! out of ntoskrnl-rust (crate-split Phase 1b). Holds the format-and-emit macros
//! plus the `dbgprint` runtime module. See Cargo.toml for the rationale.
//!
//! The macros are `#[macro_export]`, so `$crate` inside them resolves to THIS
//! crate — hence the runtime lives under `pub mod dbgprint` here, matching the
//! `$crate::dbgprint::…` paths the macros expand to. ntoskrnl-rust re-exports
//! both the macros and the module at its crate root (`pub use dbgprint_rust::…`)
//! so its ~530 `crate::dbg_print!(…)` / `crate::dbgprint::rust_*` call sites keep
//! working unchanged.

#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

/// Format-and-emit macros that bypass the C variadic trampolines and call
/// directly into the Rust ntprintf crate. See `dbgprint.rs` for the rationale
/// and migration cookbook.
///
/// `kdb_printf!`  → KdbPutsN          (replacement for `KdbPrintf(fmt, …)`)
/// `kdbp_print!`  → KdbpPagerInternal (replacement for `KdbpPrint(fmt, …)`)
/// `kdp_dprintf!` → KdpPrintString    (replacement for `KdpDprintf(fmt, …)`)
///
/// Each accepts a byte-slice format string (trailing `\0` is stripped if
/// present, so old `b"...\0"` and new `b"..."` shapes both work).
#[macro_export]
macro_rules! kdb_printf {
    ($fmt:expr $(,)?) => { $crate::dbgprint::rust_kdb_printf($fmt, &[]) };
    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_kdb_printf($fmt, slots)
    }};
}
#[macro_export]
macro_rules! kdbp_print {
    ($fmt:expr $(,)?) => { $crate::dbgprint::rust_kdbp_print($fmt, &[]) };
    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_kdbp_print($fmt, slots)
    }};
}
#[macro_export]
macro_rules! kdp_dprintf {
    ($fmt:expr $(,)?) => { $crate::dbgprint::rust_kdp_dprintf($fmt, &[]) };
    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_kdp_dprintf($fmt, slots)
    }};
}

/// `DbgPrint(fmt, …)` — routes through the kernel debug filter via
/// `DbgPrintEmitPreformatted` (no variadic FFI hop). Component bypasses the
/// filter (`MAXULONG`) at DPFLTR_ERROR_LEVEL.
#[macro_export]
macro_rules! dbg_print {
    ($fmt:expr $(,)?) => { $crate::dbgprint::rust_dbg_print($fmt, &[]) };
    ($fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_dbg_print($fmt, slots)
    }};
}

/// `DbgPrintEx(component, level, fmt, …)` — caller picks the component / level.
#[macro_export]
macro_rules! dbg_print_ex {
    ($comp:expr, $level:expr, $fmt:expr $(,)?) => {
        $crate::dbgprint::rust_dbg_print_ex($comp, $level, $fmt, &[])
    };
    ($comp:expr, $level:expr, $fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_dbg_print_ex($comp, $level, $fmt, slots)
    }};
}

/// Compile-time `&[u16]` literal — widens an ASCII string literal to UTF-16
/// without a trailing NUL. Use as `wstr!("Hello %d")`. Pair with
/// `rtl_string_cb_printf_w!` / `swprintf!` macros which `strip_trailing_wnul`
/// the format anyway, so a NUL-suffixed legacy `w!(...)` slice also works.
///
/// For NUL-terminated wide literals (e.g. `RtlInitUnicodeString`), keep
/// using the per-file `w!(...)` macro that retains the trailing zero.
#[macro_export]
macro_rules! wstr {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; BYTES.len()];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

/// Format into a caller-supplied buffer — `RtlStringCbPrintfA(dst, cb, fmt, …)`.
/// Returns NTSTATUS (`STATUS_SUCCESS`, `STATUS_BUFFER_OVERFLOW`,
/// `STATUS_INVALID_PARAMETER`). `$dst` is freely castable between `*mut u8`
/// and `*mut i8`; `$fmt` is a byte-slice (`b"…"` literal or `&[u8]` static).
#[macro_export]
macro_rules! rtl_string_cb_printf_a {
    ($dst:expr, $cb:expr, $fmt:expr $(,)?) => {
        $crate::dbgprint::rust_rtl_string_cb_printf_a($dst as *mut u8, $cb, $fmt, &[])
    };
    ($dst:expr, $cb:expr, $fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_rtl_string_cb_printf_a($dst as *mut u8, $cb, $fmt, slots)
    }};
}

/// Wide format into a caller-supplied buffer — `RtlStringCbPrintfW(dst, cb, fmt, …)`.
/// `$dst: *mut u16`, `$cb: usize` is **byte count** (not wchar count).
/// `$fmt: &[u16]` (use `wstr!("…")` literal helper).
#[macro_export]
macro_rules! rtl_string_cb_printf_w {
    ($dst:expr, $cb:expr, $fmt:expr $(,)?) => {
        $crate::dbgprint::rust_rtl_string_cb_printf_w($dst as *mut u16, $cb, $fmt, &[])
    };
    ($dst:expr, $cb:expr, $fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_rtl_string_cb_printf_w($dst as *mut u16, $cb, $fmt, slots)
    }};
}

/// CRT `swprintf(dst, fmt, …)` replacement. `$dst: *mut u16`, `$cap_chars: usize`
/// is **wide-character count** (not bytes). `$fmt: &[u16]` (use `wstr!`).
/// Returns wchar count written (excluding NUL), or `-1` on truncation.
#[macro_export]
macro_rules! swprintf {
    ($dst:expr, $cap_chars:expr, $fmt:expr $(,)?) => {
        $crate::dbgprint::rust_swprintf($dst as *mut u16, $cap_chars, $fmt, &[])
    };
    ($dst:expr, $cap_chars:expr, $fmt:expr, $($arg:expr),+ $(,)?) => {{
        let slots: &[u64] = &[$($crate::dbgprint::IntoSlot::into_slot($arg)),+];
        $crate::dbgprint::rust_swprintf($dst as *mut u16, $cap_chars, $fmt, slots)
    }};
}

pub mod dbgprint;
