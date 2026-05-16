//! Hand-written cabi shims that have no differential target (pure
//! side-effect / handler-dispatch — nothing for cnt-difftest to pin).
//! NOT generated, NOT in cnt-manifest.toml / symbols.json.

use core::sync::atomic::{AtomicUsize, Ordering};

/// `void (__cdecl *)(const wchar_t*, const wchar_t*, const wchar_t*,
/// unsigned int, uintptr_t)` — the MSVCRT invalid-parameter handler.
pub type InvalidParameterHandler =
    unsafe extern "C" fn(*const u16, *const u16, *const u16, u32, usize);

/// Faithful to `_invalid_parameter.c`'s `static ... = NULL;` (a relaxed
/// atomic is behaviorally equivalent to the C non-atomic global for
/// this contract and keeps the shim sound).
static HANDLER: AtomicUsize = AtomicUsize::new(0);

/// `void __cdecl _invalid_parameter(expr, func, file, line, arg)`.
/// libcntpr (`_LIBCNT_`, `_MSVCR_VER` unset): if a handler is
/// installed, invoke it; otherwise the C only emits a debug `ERR(...)`
/// and returns — it does NOT `RaiseException` (that path is
/// `#if _MSVCR_VER > 0`). So the faithful behavior with no handler is
/// a no-op return (the kernel's debug print has no observable contract
/// and isn't part of the translation surface).
#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter(
    expr: *const u16,
    func: *const u16,
    file: *const u16,
    line: u32,
    arg: usize,
) {
    let h = HANDLER.load(Ordering::Relaxed);
    if h != 0 {
        let f: InvalidParameterHandler = core::mem::transmute(h);
        f(expr, func, file, line, arg);
    }
}

/// `_invalid_parameter_handler __cdecl _get_invalid_parameter_handler(void)`
#[no_mangle]
pub unsafe extern "C" fn _get_invalid_parameter_handler() -> usize {
    HANDLER.load(Ordering::Relaxed)
}

/// `_invalid_parameter_handler __cdecl _set_invalid_parameter_handler(h)`
/// — returns the previous handler (faithful to the C).
#[no_mangle]
pub unsafe extern "C" fn _set_invalid_parameter_handler(handler: usize) -> usize {
    HANDLER.swap(handler, Ordering::Relaxed)
}

// ===================================================================
// scanf family — sscanf/swscanf/_snscanf/_snwscanf.
//
// The public symbols are irreducibly C-variadic and Rust cannot DEFINE
// a variadic fn on stable (the `c_variadic` feature is nightly; this
// project's discipline is "declare, never define" variadics). So a
// tiny C trampoline (src/cabi/scanf_trampoline.c) owns `va_start`/
// `va_end` and calls these non-variadic Rust entry points with a
// `va_list*` (opaque `*mut c_void` here — never dereferenced in Rust).
// Each assignment target is fetched via the C helper `cnt_va_next_ptr`
// (one `va_arg(*ap, void*)` — every scanf target is a pointer, so the
// ABI slot is uniform) at the EXACT per-directive point the genuine
// scanf.h does its `va_arg`. This mirrors genuine scanf.c, which is
// itself a 4-line `va_start; vsscanf_l(); va_end` shell over the
// engine. Phase 1: cnt_vsscanf/cnt_vswscanf (Phase 3 adds the _sn).
// ===================================================================

use crate::core_rs::scanf::{scanf_engine, IntWidth, Sink, TargetKind};

extern "C" {
    /// `void *cnt_va_next_ptr(va_list *ap)` — one `va_arg(*ap, void*)`.
    fn cnt_va_next_ptr(ap: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

struct VaSink {
    ap: *mut core::ffi::c_void,
    tgt: *mut u8,
    idx: usize,
    wide: bool,
}

impl Sink for VaSink {
    fn store_int(&mut self, w: IntWidth, value: u64) {
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe {
            match w {
                IntWidth::I64 => *(p as *mut i64) = value as i64,
                IntWidth::Long | IntWidth::Int => *(p as *mut i32) = value as i32,
                IntWidth::Short => *(p as *mut i16) = value as i16,
            }
        }
    }
    fn store_float(&mut self, negative: bool, exp: i32, d: u64, is_double: bool) {
        // Faithful to winesup.c::_internal_handle_float. Genuine uses
        // x87 `long double`; Rust has only f64 — Phase 1 does NOT fuzz
        // float (the long-double-vs-f64 parity question is owed,
        // tracked for Phase 2; this path is wired but unexercised).
        let mut curv: f64 = 1.0;
        let mut expcnt: f64 = 10.0;
        let mut e = exp;
        let negexp = e < 0;
        if negexp {
            e = -e;
        }
        while e != 0 {
            if e & 1 != 0 {
                curv *= expcnt;
            }
            e /= 2;
            expcnt *= expcnt;
        }
        let mut val = if negexp { d as f64 / curv } else { d as f64 * curv };
        if negative {
            val = -val;
        }
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe {
            if is_double {
                *(p as *mut f64) = val;
            } else {
                *(p as *mut f32) = val as f32;
            }
        }
    }
    fn store_n(&mut self, value: i32) {
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe { *(p as *mut i32) = value };
    }
    fn begin_target(&mut self, kind: TargetKind) {
        self.tgt = unsafe { cnt_va_next_ptr(self.ap) } as *mut u8;
        self.idx = 0;
        self.wide = kind == TargetKind::Wide;
    }
    fn push(&mut self, e: u32) {
        unsafe {
            if self.wide {
                *(self.tgt as *mut u16).add(self.idx) = e as u16;
            } else {
                *self.tgt.add(self.idx) = e as u8;
            }
        }
        self.idx += 1;
    }
    fn terminate(&mut self) {
        unsafe {
            if self.wide {
                *(self.tgt as *mut u16).add(self.idx) = 0;
            } else {
                *self.tgt.add(self.idx) = 0;
            }
        }
    }
}

/// `int sscanf(const char*, const char*, ...)` engine half — called by
/// the C trampoline with the live `va_list*`. NUL-bounded input.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsscanf(
    file: *const u8,
    fmt: *const u8,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input = super::cstr_slice(file);
    let format = super::cstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u8, _>(input, false, format, &mut sink)
}

/// `int swscanf(const wchar_t*, const wchar_t*, ...)` engine half.
#[no_mangle]
pub unsafe extern "C" fn cnt_vswscanf(
    file: *const u16,
    fmt: *const u16,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input = super::wcstr_slice(file);
    let format = super::wcstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u16, _>(input, false, format, &mut sink)
}

/// `int _snscanf(const char*, size_t, const char*, ...)` engine half.
/// Input is exactly `length` bytes (NOT NUL-bounded — `_snscanf` can
/// read embedded NULs up to `length`); the engine's `has_len_limit`
/// yields the genuine `consumed==length ? '\0'` (no advance) quirk.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsnscanf(
    file: *const u8,
    length: usize,
    fmt: *const u8,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input: &[u8] = if file.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(file, length)
    };
    let format = super::cstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u8, _>(input, true, format, &mut sink)
}

/// `int _snwscanf(const wchar_t*, size_t, const wchar_t*, ...)` half.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsnwscanf(
    file: *const u16,
    length: usize,
    fmt: *const u16,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input: &[u16] = if file.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(file, length)
    };
    let format = super::wcstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u16, _>(input, true, format, &mut sink)
}
