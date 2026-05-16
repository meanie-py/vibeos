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
