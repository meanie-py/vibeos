//! startup/atonexit.c — the dll-side atexit table (`_onexit`/`atexit`/
//! `__call_atexit`) and the `__onexitbegin`/`__onexitend` globals.
//!
//! Built without `CRTDLL` (only crtdll.c/tlssup #undef it), so the
//! `__onexitbegin == -1` fast-path is live: when the exe startup set that
//! sentinel (crtexe pre_c_init), `_onexit` delegates to msvcrt's own
//! `_onexit` through its IAT slot (`__imp__onexit`) to avoid recursing
//! into this definition.

use crate::misc::{_decode_pointer, _encode_pointer};
use crate::winapi::*;
use core::ffi::c_void;

const _EXIT_LOCK1: i32 = 8; // nolint: const-contradiction (startup/atonexit.c:18 `#define _EXIT_LOCK1 8`, mingw value; msvcrt mtdll.h has 13)

#[no_mangle]
pub static mut __onexitbegin: *mut Pvfv = core::ptr::null_mut();
#[no_mangle]
pub static mut __onexitend: *mut Pvfv = core::ptr::null_mut();

// msvcrt's _onexit via the import address table — NOT a direct `_onexit`
// extern (that would resolve to our own #[no_mangle] below → recursion).
// i386 decorates `_onexit` as `__onexit`, so its IAT slot is
// `__imp___onexit`; amd64/arm64 keep `__imp__onexit`.
type ImpOnexit = unsafe extern "C" fn(onexit_t) -> onexit_t;
#[cfg(target_arch = "x86")]
extern "C" {
    // i686-windows-gnu prepends `_` to link_name → final `__imp___onexit`.
    #[link_name = "_imp___onexit"]
    static IMP_ONEXIT: ImpOnexit;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    #[link_name = "__imp__onexit"]
    static IMP_ONEXIT: ImpOnexit;
}

const MINUS_ONE: *mut Pvfv = usize::MAX as *mut Pvfv;

/// call atexit functions — only with the exit lock held.
#[no_mangle]
pub unsafe extern "C" fn __call_atexit() {
    if __onexitbegin.is_null() || __onexitbegin == MINUS_ONE {
        return;
    }
    let first = _decode_pointer(__onexitbegin as *mut c_void) as *mut Pvfv;
    let mut last = _decode_pointer(__onexitend as *mut c_void) as *mut Pvfv;

    loop {
        last = last.sub(1);
        if last < first {
            break;
        }
        if let Some(f) = *last {
            f();
        }
    }
    free(first as *mut c_void);
    __onexitbegin = core::ptr::null_mut();
    __onexitend = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn _onexit(func: onexit_t) -> onexit_t {
    // exe path: defer to the CRT's own table.
    if __onexitbegin == MINUS_ONE {
        return (IMP_ONEXIT)(func);
    }

    _lock(_EXIT_LOCK1);

    let mut onexitbegin: *mut Pvfv;
    let mut onexitend: *mut Pvfv;
    if __onexitbegin.is_null() {
        // first call — allocate the one-slot array.
        onexitbegin = calloc(1, core::mem::size_of::<Pvfv>()) as *mut Pvfv;
        if onexitbegin.is_null() {
            _unlock(_EXIT_LOCK1);
            return None;
        }
        onexitend = onexitbegin;
    } else {
        onexitbegin = _decode_pointer(__onexitbegin as *mut c_void) as *mut Pvfv;
        onexitend = _decode_pointer(__onexitend as *mut c_void) as *mut Pvfv;
    }

    let retval = __dllonexit(func, &mut onexitbegin, &mut onexitend);

    if retval.is_some() {
        __onexitbegin = _encode_pointer(onexitbegin as *mut c_void) as *mut Pvfv;
        __onexitend = _encode_pointer(onexitend as *mut c_void) as *mut Pvfv;
    }

    _unlock(_EXIT_LOCK1);
    retval
}

#[no_mangle]
pub unsafe extern "C" fn atexit(func: Pvfv) -> i32 {
    // reinterpret the void(*)(void) as the int(*)(void) onexit takes.
    let f: onexit_t = core::mem::transmute(func);
    if _onexit(f).is_none() {
        -1
    } else {
        0
    }
}
