//! EnumSystemLocalesEx.c (from wine-locale.c).

use crate::common::*;

pub type LOCALE_ENUMPROCEX =
    Option<unsafe extern "system" fn(LPWSTR, DWORD, LPARAM) -> BOOL>;

struct enum_locale_ex_data {
    proc_: LOCALE_ENUMPROCEX,
    flags: DWORD,
    lparam: LPARAM,
}

unsafe extern "system" fn enum_locale_ex_proc(
    _module: HMODULE,
    _type_: LPCWSTR,
    _name: LPCWSTR,
    lang: WORD,
    lparam: LONG_PTR,
) -> BOOL {
    let data = &mut *(lparam as *mut enum_locale_ex_data);
    let mut buffer: [WCHAR; 256] = [0; 256];
    let mut neutral: DWORD = 0;

    GetLocaleInfoW(
        MAKELCID(lang, SORT_DEFAULT as u16),
        (LOCALE_SNAME | LOCALE_NOUSEROVERRIDE) as LCTYPE,
        buffer.as_mut_ptr(),
        (buffer.len()) as core::ffi::c_int, // sizeof(buffer)/sizeof(WCHAR)
    );
    if GetLocaleInfoW(
        MAKELCID(lang, SORT_DEFAULT as u16),
        (LOCALE_INEUTRAL | LOCALE_NOUSEROVERRIDE | LOCALE_RETURN_NUMBER) as LCTYPE,
        &mut neutral as *mut DWORD as LPWSTR,
        (core::mem::size_of::<DWORD>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    ) == 0
    {
        neutral = 0;
    }

    let mut flags: DWORD = LOCALE_WINDOWS;
    flags |= if neutral != 0 { LOCALE_NEUTRALDATA } else { LOCALE_SPECIFICDATA };
    if data.flags != 0 && (data.flags & flags) == 0 {
        return TRUE as BOOL;
    }
    (data.proc_.unwrap())(buffer.as_mut_ptr(), flags, data.lparam)
}

#[no_mangle]
pub unsafe extern "system" fn EnumSystemLocalesEx(
    proc_: LOCALE_ENUMPROCEX,
    flags: DWORD,
    lparam: LPARAM,
    reserved: LPVOID,
) -> BOOL {
    let kernel32_handle = GetModuleHandleW(w!("kernel32.dll").as_ptr());

    if !reserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let mut data = enum_locale_ex_data {
        proc_,
        flags,
        lparam,
    };

    // ENUMRESLANGPROCW lowers to the per-arch concrete ABI (stdcall on i386);
    // our `extern "system"` callback has the identical ABI — transmute the fn ptr
    // to the bindgen callback type (Option<fn>; the fn item is non-null → Some).
    let cb: ENUMRESLANGPROCW = core::mem::transmute(enum_locale_ex_proc as *const ());

    EnumResourceLanguagesW(
        kernel32_handle,
        RT_STRING_ID as usize as LPCWSTR,                          // (LPCWSTR)RT_STRING
        ((LOCALE_SNAME >> 4) + 1) as usize as LPCWSTR,             // MAKEINTRESOURCE((LOCALE_SNAME>>4)+1)
        cb,
        &mut data as *mut enum_locale_ex_data as LONG_PTR,
    );

    TRUE as BOOL
}
