// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8A9EBF6F9AD6: [u16; 32] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0064, 0x0069, 0x0073, 0x0070,
    0x006C, 0x0061, 0x0079, 0x0069, 0x006E, 0x0067, 0x0020, 0x0065, 0x0072, 0x0072,
    0x006F, 0x0072, 0x0020, 0x006D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065,
    0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_95841B59047A: [u16; 31] = [
    0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0064, 0x0069, 0x0073, 0x0070,
    0x006C, 0x0061, 0x0079, 0x0069, 0x006E, 0x0067, 0x0020, 0x0069, 0x006E, 0x0066,
    0x006F, 0x0020, 0x006D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x002E,
    0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn ErrorMessageBox_v(
    mut hWnd: HWND,
    mut lpTitle: LPCWSTR,
    mut dwErrorCode: DWORD,
    ap_list: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut iRet: ::core::ffi::c_int = 0;
    let mut lpMsgBuf: LPWSTR = ::core::ptr::null_mut();
    let mut Status: DWORD = 0;


    Status = FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
        ::core::ptr::null::<::core::ffi::c_void>(),
        dwErrorCode,
        MAKELANGID!(LANG_NEUTRAL as WORD, SUBLANG_DEFAULT as WORD) as DWORD,
        &raw mut lpMsgBuf as LPWSTR,
        0 as DWORD,
        (ap_list as *mut va_list),
    );



    iRet = MessageBoxW(
        hWnd,
        if Status != 0 && !lpMsgBuf.is_null() {
            lpMsgBuf as LPCWSTR
        } else {
            RUSTCONV_WSTR_8A9EBF6F9AD6.as_ptr()
        },
        lpTitle,
        (MB_OK | MB_ICONERROR) as UINT,
    );

    if !lpMsgBuf.is_null() {
        LocalFree(lpMsgBuf as HLOCAL);
    }

    /* Return the MessageBoxW information */
    return iRet;
}


#[no_mangle]
pub unsafe extern "C" fn InfoMessageBox_v(
    hWnd: HWND,
    uType: UINT,
    lpTitle: LPCWSTR,
    lpMessage: LPCWSTR,
    ap_list: *mut ::core::ffi::c_void,
) -> core::ffi::c_int {
    let mut iRet: core::ffi::c_int = 0 as core::ffi::c_int;
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();
    let mut args: va_list = *(ap_list as *mut va_list);



    if !lpMessage.is_null() {
        let mut strLen: SIZE_T = _vscwprintf(lpMessage, (args).cast()) as SIZE_T;

        /* Create a buffer on the heap and zero it out (LPTR) */
        lpMsgBuf = LocalAlloc(
            LPTR as UINT,
            strLen
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpMsgBuf.is_null() {
            _vsnwprintf(lpMsgBuf, strLen as usize, lpMessage, args);
        }
    }



    iRet = MessageBoxW(
        hWnd,
        if !lpMessage.is_null() && !lpMsgBuf.is_null() {
            lpMsgBuf as LPCWSTR
        } else {
            RUSTCONV_WSTR_95841B59047A.as_ptr()
        },
        lpTitle,
        uType,
    );

    if !lpMsgBuf.is_null() {
        LocalFree(lpMsgBuf as *mut core::ffi::c_void);
    }

    /* Return the MessageBoxW information */
    return iRet;
}
