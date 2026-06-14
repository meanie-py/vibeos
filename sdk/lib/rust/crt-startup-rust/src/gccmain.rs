//! startup/gccmain.c — the GCC global ctor/dtor walk + `__main` hook.
//!
//! `__CTOR_LIST__`/`__DTOR_LIST__` are provided by the toolchain's
//! crtbegin (same as for the C msvcrtex, which links today). `__main` is
//! the ctor hook gcc/clang insert at the top of `main`.

use core::ptr::addr_of;

extern "C" {
    static __CTOR_LIST__: usize;
    static __DTOR_LIST__: usize;
}

// dtor cursor: C `static func_ptr *p = __DTOR_LIST__ + 1;`. Null = not yet
// initialized (a real `&__DTOR_LIST__[1]` is never null); it persists +
// advances across calls, exactly like the C function-local static.
static mut DTOR_P: *const usize = core::ptr::null();

#[no_mangle]
pub unsafe extern "C" fn __do_global_dtors() {
    if DTOR_P.is_null() {
        DTOR_P = addr_of!(__DTOR_LIST__).add(1);
    }
    while *DTOR_P != 0 {
        let f: extern "C" fn() = core::mem::transmute(*DTOR_P);
        f();
        DTOR_P = DTOR_P.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __do_global_ctors() {
    let list = addr_of!(__CTOR_LIST__);
    let mut nptrs = *list;
    if nptrs == usize::MAX {
        // count up to the NULL terminator.
        nptrs = 0;
        while *list.add(nptrs + 1) != 0 {
            nptrs += 1;
        }
    }
    let mut i = nptrs;
    while i >= 1 {
        let f: extern "C" fn() = core::mem::transmute(*list.add(i));
        f();
        i -= 1;
    }
}

static mut INITIALIZED: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn __main() {
    if INITIALIZED == 0 {
        INITIALIZED = 1;
        __do_global_ctors();
    }
}
