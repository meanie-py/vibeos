use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// setupapi_private.h only DECLARES hInstance, OsVersionInfo (setupcab.c) and
// GlobalSetupFlags (misc.c, still C) `extern`; they are defined where the C defines them.

pub const ANYSIZE_ARRAY: usize = 1;






pub unsafe extern "C" fn strdupAtoW(str: *const core::ffi::c_char) -> *mut WCHAR {
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    if !str.is_null() {
        let mut len: DWORD = MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, str as LPCSTR, -(1 as core::ffi::c_int), core::ptr::null_mut::<WCHAR>(), 0 as core::ffi::c_int) as DWORD;
        ret = HeapAlloc(GetProcessHeap(), 0 as DWORD, (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
        if !ret.is_null() {
            MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, str as LPCSTR, -(1 as core::ffi::c_int), ret as LPWSTR, len as core::ffi::c_int);
        }
    }
    return ret;
}
