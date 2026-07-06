//! Rust-native bridge to the kernel debug-print sinks (KdbPutsN,
//! KdbpPagerInternal, KdpPrintString) that bypasses the C variadic
//! trampolines. Rust callers use the [`kdb_printf!`], [`kdbp_print!`],
//! [`kdp_dprintf!`] macros — the same call shape they had with the FFI
//! variadic, just routed through `ntprintf` instead of `_vsnprintf`.
//!
//! ## Migration cookbook
//!
//! ```ignore
//! // Old (called the C trampoline via extern "C" { fn KdbpPrint(...); })
//! KdbpPrint(b"%p Thread %p\n\0".as_ptr() as PSTR, thread, current);
//!
//! // New (bypasses C entirely; trailing NUL in the literal is stripped)
//! crate::kdbp_print!(b"%p Thread %p\n", thread, current);
//! ```
//!
//! `IntoSlot` lets every primitive integer / pointer / `bool` flow into a
//! `u64` slot without explicit casts at the call site.

use core::ffi::c_void;

use ntprintf::{format_into, Writer};
use ntprintf_w::{format_into as format_into_w, WriterW};
use ntkrnl_sys::opaque_ptr;

// ---------------------------------------------------------------------------
// IntoSlot — pack any printf-shaped value into the 8-byte slot ntprintf wants.

pub trait IntoSlot {
    fn into_slot(self) -> u64;
}

macro_rules! impl_intoslot_uint {
    ($($t:ty),*) => { $(
        impl IntoSlot for $t { #[inline(always)] fn into_slot(self) -> u64 { self as u64 } }
    )* };
}
macro_rules! impl_intoslot_iint {
    ($($t:ty),*) => { $(
        impl IntoSlot for $t {
            #[inline(always)]
            fn into_slot(self) -> u64 {
                // Sign-extend through i64 then reinterpret. For i32/-1 this
                // produces 0xFFFFFFFF in the low 32 bits — what the C va_arg
                // path also produces (i32 promoted via int → low 32, high
                // bits ignored by the formatter when scanner.tag == 'd').
                self as i64 as u64
            }
        }
    )* };
}

impl_intoslot_uint!(u8, u16, u32, u64, usize, char, bool);
impl_intoslot_iint!(i8, i16, i32, i64, isize);

impl<T> IntoSlot for *const T { #[inline(always)] fn into_slot(self) -> u64 { self as usize as u64 } }
impl<T> IntoSlot for *mut T   { #[inline(always)] fn into_slot(self) -> u64 { self as usize as u64 } }
impl<T> IntoSlot for &T       { #[inline(always)] fn into_slot(self) -> u64 { self as *const T as usize as u64 } }
impl<T> IntoSlot for &mut T   { #[inline(always)] fn into_slot(self) -> u64 { self as *const T as usize as u64 } }

// ---------------------------------------------------------------------------
// FFI to the kernel sinks.

// KDBG sinks live in `ntoskrnl-rust::kdbg` (i386/amd64 only) and the
// supporting C shim (`kdbg_rust_shims.c`, also i386/amd64 only — arm64
// disables KDBG, see ntos.cmake `if(KDBG AND NOT ARCH STREQUAL "arm64")`).
// Don't reference the symbols on arm64 — the rust_kdb_*/rust_kdbp_*
// entry points become no-ops there (see below).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe extern "C" {
    /// Defined in Rust kdbg/kdb_print.rs (extern "C").
    fn KdbPutsN(string: *const u8, length: u16);
    /// Defined in Rust kdbg/kdb_cli.rs (extern "C").
    fn KdbpPagerInternal(buffer: *mut u8, length: u32, do_page: u8);
    /// Defined in C (kdbg_rust_shims.c). When non-zero, KDBG suppresses
    /// pager output (set when the user hits Ctrl-C mid-listing).
    static mut KdbOutputAborted: u8;
}

unsafe extern "system" {
    /// Defined in Rust kd64/kdprint.rs as `extern "system"` (NTAPI = stdcall
    /// on i386). Calling it through `extern "C"` corrupts the stack on i386
    /// and the function looked like it returned normally, but LLVM proved
    /// the post-call code path was UB → ud2. (#1 i386 crash cause per
    /// feedback_extern_c_abi.md.)
    fn KdpPrintString(output: *mut StringSlim) -> u8;
    /// Defined in C (sdk/lib/rtl/debug.c, `_NTOSKRNL_`-only). Takes a
    /// pre-rendered ANSI body and routes it through the kernel debug
    /// filter + DebugPrint sink. Replaces the Rust path that would
    /// otherwise re-enter the variadic `DbgPrint` trampoline.
    fn DbgPrintEmitPreformatted(body: *const u8, body_length: u16,
                                component_id: u32, level: u32) -> u32;
}

/// Mirror of the kernel STRING (ANSI) layout used by KdpPrintString.
#[repr(C)]
struct StringSlim {
    length: u16,
    maximum_length: u16,
    buffer: *mut u8,
}

// ---------------------------------------------------------------------------
// Per-target Rust shims. Each owns its stack buffer and emits to one sink.

/// Raw-pointer sink — avoids holding a `&mut [u8]` across the format_into
/// call. The slice variant trips the i386 LLVM captures(none) deduction:
/// rust_kdp_dprintf miscompiled to `ud2` immediately after format_into when
/// the sink held `&'a mut [u8]` and the caller later took `buf.as_mut_ptr()`
/// for the FFI hand-off. Raw `*mut u8` doesn't carry noalias, so the chain
/// breaks. Same pattern as ntprintf::BoundedBuf.
struct RawSink {
    dst: *mut u8,
    cap: usize,
    pos: usize,
    total: usize,
}
impl RawSink {
    /// `dst` is laundered through `opaque_ptr` so LLVM can't follow the chain
    /// from `&mut buf` → `as_mut_ptr` → sink-write back to a `&mut [u8; N]`
    /// stored elsewhere. Without this, format_into gets `captures(none)` and
    /// the caller miscompiles to `ud2` after the call (i386, see the
    /// captures cascade pattern documented in feedback_cgu1.md).
    fn new(dst: *mut u8, cap: usize) -> Self {
        let dst = unsafe { opaque_ptr(dst) };
        Self { dst, cap, pos: 0, total: 0 }
    }
}
impl Writer for RawSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.total += bytes.len();
        let mut i = 0;
        while i < bytes.len() && self.pos < self.cap {
            unsafe { *self.dst.add(self.pos) = bytes[i]; }
            self.pos += 1;
            i += 1;
        }
    }
    fn written(&self) -> usize { self.total }
}

/// Strip a single trailing NUL byte from a format-string literal so old
/// `b"...\0"` call sites and new `b"..."` call sites both render correctly.
#[inline]
fn strip_trailing_nul(fmt: &[u8]) -> &[u8] {
    match fmt.split_last() {
        Some((&0, rest)) => rest,
        _ => fmt,
    }
}

/// Format and send to KdbPutsN — replacement for `KdbPrintf(fmt, ...)`.
/// arm64: no-op stub (KDBG is disabled on arm64 — see ntos.cmake).
pub fn rust_kdb_printf(fmt: &[u8], slots: &[u64]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let fmt = strip_trailing_nul(fmt);
        let mut buf = [0u8; 1024];
        let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
        format_into(&mut sink, fmt, slots);
        let len = sink.pos.min(u16::MAX as usize - 1);
        unsafe { KdbPutsN(buf.as_ptr(), len as u16); }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    { let _ = (fmt, slots); }
}

/// Send a NUL-terminated C string straight through KdbpPagerInternal with
/// **no** format-string interpretation. For call sites that pull the format
/// pointer out of an array (e.g. `EflagsBits[i]`) where the contents are
/// pre-baked literal text and there are never any `%` specifiers.
///
/// `cstr` must point at a NUL-terminated byte sequence. Null pointers are
/// silently ignored (matches C printf-of-null-pointer-as-fmt being UB —
/// callers already check `.is_null()` before calling).
///
/// arm64: no-op stub.
pub fn rust_kdbp_print_cstr(cstr: *const u8) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if cstr.is_null() { return; }
        if unsafe { KdbOutputAborted } != 0 { return; }
        // Walk to the NUL (cap at 4 KiB to match the buffered emit path).
        let mut len = 0usize;
        while len < 4096 && unsafe { *cstr.add(len) } != 0 { len += 1; }
        let mut buf = [0u8; 4096];
        unsafe { core::ptr::copy_nonoverlapping(cstr, buf.as_mut_ptr(), len); }
        buf[len] = 0;
        unsafe { KdbpPagerInternal(buf.as_mut_ptr(), len as u32, 0); }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    { let _ = cstr; }
}

/// Format and send to KdbpPagerInternal — replacement for `KdbpPrint(fmt, ...)`.
/// arm64: no-op stub.
pub fn rust_kdbp_print(fmt: &[u8], slots: &[u64]) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        // Mirror the C version's KdbOutputAborted check so paged-output
        // suppression still works for Rust callers.
        if unsafe { KdbOutputAborted } != 0 {
            return;
        }
        let fmt = strip_trailing_nul(fmt);
        // 4 KiB matches the original C buffer; anything bigger gets clipped (and
        // the pager handles its own newline-by-newline emission anyway).
        let mut buf = [0u8; 4096];
        let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
        format_into(&mut sink, fmt, slots);
        let len = sink.pos.min(buf.len() - 1);
        unsafe {
            *buf.as_mut_ptr().add(len) = 0;
            KdbpPagerInternal(buf.as_mut_ptr(), len as u32, 0);
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    { let _ = (fmt, slots); }
}

/// Format and send through DbgPrintEmitPreformatted — replacement for
/// `DbgPrint(fmt, ...)`. Routes through the kernel debug filter (component =
/// MAXULONG = bypass filter) at error level (DPFLTR_ERROR_LEVEL = 3).
pub fn rust_dbg_print(fmt: &[u8], slots: &[u64]) -> u32 {
    let fmt = strip_trailing_nul(fmt);
    let mut buf = [0u8; 512];
    let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
    format_into(&mut sink, fmt, slots);
    let len = sink.pos.min(u16::MAX as usize);
    unsafe { DbgPrintEmitPreformatted(buf.as_ptr(), len as u16, u32::MAX, 3) }
}

/// `DbgPrint`-style emit for call sites that already hold the format-string
/// pointer at runtime (e.g. `DbgPrint(message, …)` where `message: *const i8`).
/// Walks the NUL-terminated `cstr` to learn its length, then runs ntprintf
/// the same as `rust_dbg_print`. Caps the format string at 4 KiB.
/// Null pointers are silently ignored.
pub fn rust_dbg_print_cstr(cstr: *const i8, slots: &[u64]) -> u32 {
    if cstr.is_null() { return 0; }
    let cstr = cstr as *const u8;
    let mut len = 0usize;
    while len < 4096 && unsafe { *cstr.add(len) } != 0 { len += 1; }
    let fmt = unsafe { core::slice::from_raw_parts(cstr, len) };
    let mut buf = [0u8; 512];
    let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
    format_into(&mut sink, fmt, slots);
    let len = sink.pos.min(u16::MAX as usize);
    unsafe { DbgPrintEmitPreformatted(buf.as_ptr(), len as u16, u32::MAX, 3) }
}

/// Format and send through DbgPrintEmitPreformatted with a caller-supplied
/// component_id + level — replacement for `DbgPrintEx(comp, level, fmt, ...)`.
pub fn rust_dbg_print_ex(component_id: u32, level: u32,
                         fmt: &[u8], slots: &[u64]) -> u32 {
    let fmt = strip_trailing_nul(fmt);
    let mut buf = [0u8; 512];
    let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
    format_into(&mut sink, fmt, slots);
    let len = sink.pos.min(u16::MAX as usize);
    unsafe { DbgPrintEmitPreformatted(buf.as_ptr(), len as u16, component_id, level) }
}

/// Format and send to KdpPrintString — replacement for `KdpDprintf(fmt, ...)`.
pub fn rust_kdp_dprintf(fmt: &[u8], slots: &[u64]) {
    let fmt = strip_trailing_nul(fmt);
    let mut buf = [0u8; 512];
    let mut sink = RawSink::new(buf.as_mut_ptr(), buf.len());
    format_into(&mut sink, fmt, slots);
    let len = sink.pos.min(u16::MAX as usize);
    let mut s = StringSlim {
        length: len as u16,
        maximum_length: len as u16,
        buffer: buf.as_mut_ptr(),
    };
    unsafe { KdpPrintString(&mut s as *mut StringSlim); }
}

/// Format into a caller-supplied buffer — replacement for
/// `RtlStringCbPrintfA(dst, cb, fmt, ...)`. Returns NTSTATUS:
/// `STATUS_SUCCESS` when the formatted string fit, `STATUS_BUFFER_OVERFLOW`
/// when truncated, `STATUS_INVALID_PARAMETER` when `cb == 0`.
pub fn rust_rtl_string_cb_printf_a(dst: *mut u8, cb: usize, fmt: &[u8], slots: &[u64]) -> i32 {
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005u32 as i32;
    const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;

    if dst.is_null() || cb == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    let fmt = strip_trailing_nul(fmt);
    let mut sink = RawSink::new(dst, cb);
    let total = format_into(&mut sink, fmt, slots);
    let term = sink.pos.min(cb - 1);
    unsafe { *dst.add(term) = 0; }
    if total + 1 > cb { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

// ---------------------------------------------------------------------------
// Wide (UTF-16) sinks + entry points. Mirrors the ANSI side via ntprintf_w.

/// Wide raw-pointer sink — mirrors `RawSink` for `&mut [u16]`.
struct RawSinkW {
    dst: *mut u16,
    cap_chars: usize,
    pos: usize,
    total: usize,
}
impl RawSinkW {
    fn new(dst: *mut u16, cap_chars: usize) -> Self {
        let dst = unsafe { opaque_ptr(dst as *mut u8) } as *mut u16;
        Self { dst, cap_chars, pos: 0, total: 0 }
    }
}
impl WriterW for RawSinkW {
    fn write_chars(&mut self, chars: &[u16]) {
        self.total += chars.len();
        let mut i = 0;
        while i < chars.len() && self.pos < self.cap_chars {
            unsafe { *self.dst.add(self.pos) = chars[i]; }
            self.pos += 1;
            i += 1;
        }
    }
    fn written(&self) -> usize { self.total }
}

/// Strip a single trailing NUL `u16` from a wide format-string literal, so
/// both `wstr!("...")` (no NUL) and the older `w!("...")` (with NUL) shapes
/// work as input.
#[inline]
fn strip_trailing_wnul(fmt: &[u16]) -> &[u16] {
    match fmt.split_last() {
        Some((&0, rest)) => rest,
        _ => fmt,
    }
}

/// Wide-format equivalent of `rust_rtl_string_cb_printf_a`. `cb` is **bytes**
/// (matches the `RtlStringCbPrintfW` signature); divided by `sizeof(WCHAR)`
/// internally for the wchar-counted formatter.
pub fn rust_rtl_string_cb_printf_w(dst: *mut u16, cb: usize, fmt: &[u16], slots: &[u64]) -> i32 {
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005u32 as i32;
    const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;

    if dst.is_null() || cb < 2 {
        return STATUS_INVALID_PARAMETER;
    }
    let cap_chars = cb / 2;
    let fmt = strip_trailing_wnul(fmt);
    let mut sink = RawSinkW::new(dst, cap_chars);
    let total = format_into_w(&mut sink, fmt, slots);
    let term = sink.pos.min(cap_chars - 1);
    unsafe { *dst.add(term) = 0; }
    if total + 1 > cap_chars { STATUS_BUFFER_OVERFLOW } else { STATUS_SUCCESS }
}

/// Replacement for the CRT `swprintf(dst, fmt, ...)`. Caller supplies
/// `cap_chars` (wchar count); we always NUL-terminate at the truncation
/// boundary. Returns wchar count written (excluding NUL), or `-1` on
/// truncation, matching CRT swprintf's quirky overflow semantics.
pub fn rust_swprintf(dst: *mut u16, cap_chars: usize, fmt: &[u16], slots: &[u64]) -> i32 {
    if dst.is_null() || cap_chars == 0 {
        return -1;
    }
    let fmt = strip_trailing_wnul(fmt);
    let mut sink = RawSinkW::new(dst, cap_chars);
    let total = format_into_w(&mut sink, fmt, slots);
    let term = sink.pos.min(cap_chars - 1);
    unsafe { *dst.add(term) = 0; }
    if total + 1 > cap_chars { -1 } else { total as i32 }
}

// Silence unused-warning when KdbOutputAborted_get isn't linked in some
// build configurations.
#[allow(dead_code)]
fn _force_link() -> usize { rust_kdb_printf as usize }

// We need a placeholder so `c_void` import isn't dead. Real callers use it
// transitively through pointer slots.
#[allow(dead_code)]
fn _hold_c_void(_: *mut c_void) {}
