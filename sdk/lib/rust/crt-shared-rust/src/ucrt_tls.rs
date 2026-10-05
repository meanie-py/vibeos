//! dll/win32/msvcrt/reactos/ucrt_tls_sup.c: ucrtbase's per-thread `thread_data_t` slot for
//! the Wine (msvcrt_shared) code it carries. msvcrt.dll has its own (msvcrt-crt-rust's
//! thread.rs, which also seeds the locale); ucrtbase's is this smaller one, driven by
//! vcruntime's `__vcrt_initialize`/`__vcrt_uninitialize` and the `.CRT$XLD` TLS callback.

use crate::prelude::*;
use core::ffi::c_int;

/// winbase.h `TLS_OUT_OF_INDEXES` (not in the bindgen island).
const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF;

extern "C" {
    fn exit(code: c_int) -> !;
    fn free(mem: *mut core::ffi::c_void);
    fn _assert(message: *const i8, file: *const i8, line: u32);
}

#[no_mangle]
pub static mut msvcrt_tls_index: DWORD = 0;

#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_tls() -> BOOL {
    msvcrt_tls_index = TlsAlloc();
    if msvcrt_tls_index == TLS_OUT_OF_INDEXES {
        ERR!("TlsAlloc() failed!\n");
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_tls() -> BOOL {
    if TlsFree(msvcrt_tls_index) == 0 {
        ERR!("TlsFree() failed!\n");
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// The C's `assert()`s (live: ReactOS builds with assertions on).
macro_rules! assert_null {
    ($p:expr, $text:literal) => {
        if !$p.is_null() {
            _assert(concat!($text, "\0").as_ptr() as _, concat!(file!(), "\0").as_ptr() as _, line!());
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_tls_mem() {
    let tls = TlsGetValue(msvcrt_tls_index) as *mut thread_data_t;
    if !tls.is_null() {
        free((*tls).efcvt_buffer as _);
        assert_null!((*tls).asctime_buffer, "tls->asctime_buffer == NULL");
        assert_null!((*tls).wasctime_buffer, "tls->wasctime_buffer == NULL");
        assert_null!((*tls).strerror_buffer, "tls->strerror_buffer == NULL");
        assert_null!((*tls).wcserror_buffer, "tls->wcserror_buffer == NULL");
        assert_null!((*tls).time_buffer, "tls->time_buffer == NULL");
        assert_null!((*tls).tmpnam_buffer, "tls->tmpnam_buffer == NULL");
        assert_null!((*tls).wtmpnam_buffer, "tls->wtmpnam_buffer == NULL");
        assert_null!((*tls).locinfo, "tls->locinfo == NULL");
        assert_null!((*tls).mbcinfo, "tls->mbcinfo == NULL");
        HeapFree(GetProcessHeap(), 0, tls as _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn msvcrt_get_thread_data() -> *mut thread_data_t {
    let err = GetLastError(); // need to preserve last error
    let mut ptr = TlsGetValue(msvcrt_tls_index) as *mut thread_data_t;
    if ptr.is_null() {
        ptr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<thread_data_t>() as _)
            as *mut thread_data_t;
        if ptr.is_null() {
            exit(_RT_THREAD as c_int);
        }
        if TlsSetValue(msvcrt_tls_index, ptr as _) == 0 {
            exit(_RT_THREAD as c_int);
        }
        (*ptr).tid = GetCurrentThreadId();
        (*ptr).handle = INVALID_HANDLE_VALUE;
        (*ptr).random_seed = 1;
    }
    SetLastError(err);
    ptr
}
