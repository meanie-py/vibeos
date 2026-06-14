//! argv setup stubs: dllargv.c/wdllargv.c (`_setargv`/`_wsetargv` —
//! no-ops, globbing off) and dll_argv.c (`__setargv`/`__wsetargv` — the
//! globbing-enable alternates that set `_dowildcard`). Lazy-pulled: the
//! exe entry calls `_setargv`/`_wsetargv`; the `__`-variants are only
//! referenced when the image opts into wildcard expansion.

use crate::globals;

// startup/dllargv.c
#[no_mangle]
pub unsafe extern "C" fn _setargv() -> i32 {
    0
}
// startup/wdllargv.c
#[no_mangle]
pub unsafe extern "C" fn _wsetargv() -> i32 {
    0
}

// startup/dll_argv.c (non-W + WPRFLAG)
#[no_mangle]
pub unsafe extern "C" fn __setargv() -> i32 {
    globals::_dowildcard = 1;
    0
}
#[no_mangle]
pub unsafe extern "C" fn __wsetargv() -> i32 {
    globals::_dowildcard = 1;
    0
}
