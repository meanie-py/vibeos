//! startup/tlssup.c + tlsthrd.c + tlsmthread.c — TLS template/directory,
//! the dynamic TLS init/dtor callbacks, and the thread-key dtor registry.
//!
//! DISABLE_MS_TLS is set (as in the C), so the in-callback dtor-node list
//! is compiled out; __tlregdtor is a no-op and __dyn_tls_dtor only drives
//! __mingw_TLScallback.

use crate::globals::_tls_index;
use crate::winapi::*;
use core::ffi::c_void;
use core::ptr::addr_of;

// ---- TLS raw template markers ---------------------------------------
#[no_mangle]
#[link_section = ".tls"]
#[used]
pub static mut _tls_start: *mut u8 = core::ptr::null_mut();
#[no_mangle]
#[link_section = ".tls$ZZZ"]
#[used]
pub static mut _tls_end: *mut u8 = core::ptr::null_mut();

// ---- TLS callback array bounds (.CRT$XL*) ---------------------------
#[no_mangle]
#[link_section = ".CRT$XLA"]
#[used]
pub static __xl_a: PIMAGE_TLS_CALLBACK = None;
#[no_mangle]
#[link_section = ".CRT$XLZ"]
#[used]
pub static __xl_z: PIMAGE_TLS_CALLBACK = None;

// dynamic TLS init/dtor callbacks placed between the bounds.
#[no_mangle]
#[link_section = ".CRT$XLC"]
#[used]
pub static __xl_c: PIMAGE_TLS_CALLBACK = Some(dyn_tls_init_cb);
#[no_mangle]
#[link_section = ".CRT$XLD"]
#[used]
pub static __xl_d: PIMAGE_TLS_CALLBACK = Some(dyn_tls_dtor_cb);

// ---- C-runtime dynamic-init dtor bounds (.CRT$XD*) ------------------
#[no_mangle]
#[link_section = ".CRT$XDA"]
#[used]
pub static __xd_a: Pvfv = None;
#[no_mangle]
#[link_section = ".CRT$XDZ"]
#[used]
pub static __xd_z: Pvfv = None;

// ---- IMAGE_TLS_DIRECTORY (`_tls_used`) ------------------------------
// Pointer-typed fields (same layout as IMAGE_TLS_DIRECTORY) so the
// address initializers don't need a const ptr→int cast. AddressOfCallBacks
// = &__xl_a + 1 (the first real callback slot), expressed with the const
// `wrapping_add`.
#[repr(C)]
struct TlsDir {
    start: *const u8,
    end: *const u8,
    index: *const u32,
    callbacks: *const PIMAGE_TLS_CALLBACK,
    size_of_zero_fill: u32,
    characteristics: u32,
}
unsafe impl Sync for TlsDir {}

#[no_mangle]
#[used]
pub static _tls_used: TlsDir = TlsDir {
    start: addr_of!(_tls_start) as *const u8,
    end: addr_of!(_tls_end) as *const u8,
    index: addr_of!(_tls_index),
    callbacks: addr_of!(__xl_a).wrapping_add(1),
    size_of_zero_fill: 0,
    characteristics: 0,
};

// ---- the dynamic TLS callbacks --------------------------------------
// (__mingw_TLScallback is defined below, in the tlsthrd.c section.)

#[no_mangle]
pub unsafe extern "system" fn __dyn_tls_init(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    lpreserved: LPVOID,
) -> WINBOOL {
    if crate::globals::_CRT_MT != 2 {
        crate::globals::_CRT_MT = 2;
    }

    if dwReason != DLL_THREAD_ATTACH {
        if dwReason == DLL_PROCESS_ATTACH {
            __mingw_TLScallback(hDllHandle, dwReason, lpreserved);
        }
        return TRUE;
    }

    // walk the .CRT$XD* C-runtime TLS init list (XDA+1 .. XDZ).
    let mut ps = addr_of!(__xd_a) as usize + core::mem::size_of::<usize>();
    let end = addr_of!(__xd_z) as usize;
    while ps != end {
        let pfunc = *(ps as *const Pvfv);
        if let Some(f) = pfunc {
            f();
        }
        ps += core::mem::size_of::<usize>();
    }
    TRUE
}

// PIMAGE_TLS_CALLBACK-typed trampoline (the C casts __dyn_tls_init, whose
// BOOL return the loader ignores).
unsafe extern "system" fn dyn_tls_init_cb(h: HANDLE, r: DWORD, l: LPVOID) {
    __dyn_tls_init(h, r, l);
}

#[no_mangle]
pub static __dyn_tls_init_callback: PIMAGE_TLS_CALLBACK = Some(dyn_tls_init_cb);

unsafe extern "system" fn dyn_tls_dtor_cb(h: HANDLE, r: DWORD, l: LPVOID) {
    __dyn_tls_dtor(h, r, l);
}

unsafe extern "system" fn __dyn_tls_dtor(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    lpreserved: LPVOID,
) -> WINBOOL {
    if dwReason != DLL_THREAD_DETACH && dwReason != DLL_PROCESS_DETACH {
        return TRUE;
    }
    __mingw_TLScallback(hDllHandle, dwReason, lpreserved);
    TRUE
}

#[no_mangle]
pub unsafe extern "C" fn __tlregdtor(func: Pvfv) -> i32 {
    if func.is_none() {
        return 0;
    }
    // DISABLE_MS_TLS: nothing to register.
    0
}

// ---- tlsthrd.c: thread-key dtor registry ----------------------------
static mut __mingwthr_cs: CRITICAL_SECTION = CRITICAL_SECTION::zeroed();
static mut __mingwthr_cs_init: i32 = 0;

type KeyDtor = Option<unsafe extern "C" fn(*mut c_void)>;

#[repr(C)]
struct MingwthrKey {
    key: DWORD,
    dtor: KeyDtor,
    next: *mut MingwthrKey,
}

static mut key_dtor_list: *mut MingwthrKey = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn ___w64_mingwthr_add_key_dtor(key: DWORD, dtor: KeyDtor) -> i32 {
    if __mingwthr_cs_init == 0 {
        return 0;
    }
    let new_key = calloc(1, core::mem::size_of::<MingwthrKey>()) as *mut MingwthrKey;
    if new_key.is_null() {
        return -1;
    }
    (*new_key).key = key;
    (*new_key).dtor = dtor;

    EnterCriticalSection(addr_of_mut_cs());
    (*new_key).next = key_dtor_list;
    key_dtor_list = new_key;
    LeaveCriticalSection(addr_of_mut_cs());
    0
}

#[no_mangle]
pub unsafe extern "C" fn ___w64_mingwthr_remove_key_dtor(key: DWORD) -> i32 {
    if __mingwthr_cs_init == 0 {
        return 0;
    }
    EnterCriticalSection(addr_of_mut_cs());

    let mut prev: *mut MingwthrKey = core::ptr::null_mut();
    let mut cur = key_dtor_list;
    while !cur.is_null() {
        if (*cur).key == key {
            if prev.is_null() {
                key_dtor_list = (*cur).next;
            } else {
                (*prev).next = (*cur).next;
            }
            free(cur as *mut c_void);
            break;
        }
        prev = cur;
        cur = (*cur).next;
    }
    LeaveCriticalSection(addr_of_mut_cs());
    0
}

unsafe fn __mingwthr_run_key_dtors() {
    if __mingwthr_cs_init == 0 {
        return;
    }
    EnterCriticalSection(addr_of_mut_cs());
    let mut keyp = key_dtor_list;
    while !keyp.is_null() {
        let value = TlsGetValue((*keyp).key);
        if GetLastError() == ERROR_SUCCESS {
            if !value.is_null() {
                if let Some(d) = (*keyp).dtor {
                    d(value);
                }
            }
        }
        keyp = (*keyp).next;
    }
    LeaveCriticalSection(addr_of_mut_cs());
}

#[no_mangle]
pub unsafe extern "C" fn __mingw_TLScallback(
    _hDllHandle: HANDLE,
    reason: DWORD,
    _reserved: LPVOID,
) -> WINBOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            if __mingwthr_cs_init == 0 {
                InitializeCriticalSection(addr_of_mut_cs());
            }
            __mingwthr_cs_init = 1;
        }
        DLL_PROCESS_DETACH => {
            __mingwthr_run_key_dtors();
            if __mingwthr_cs_init == 1 {
                __mingwthr_cs_init = 0;
                DeleteCriticalSection(addr_of_mut_cs());
            }
        }
        DLL_THREAD_DETACH => {
            __mingwthr_run_key_dtors();
        }
        _ => {}
    }
    TRUE
}

#[inline]
unsafe fn addr_of_mut_cs() -> *mut CRITICAL_SECTION {
    core::ptr::addr_of_mut!(__mingwthr_cs)
}

// ---- tlsmthread.c: public wrappers ----------------------------------
#[no_mangle]
pub unsafe extern "C" fn __mingwthr_remove_key_dtor(key: DWORD) -> i32 {
    ___w64_mingwthr_remove_key_dtor(key)
}

#[no_mangle]
pub unsafe extern "C" fn __mingwthr_key_dtor(key: DWORD, dtor: KeyDtor) -> i32 {
    if dtor.is_some() {
        return ___w64_mingwthr_add_key_dtor(key, dtor);
    }
    0
}
