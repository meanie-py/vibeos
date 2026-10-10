use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn LengthOfStrResource(mut hInst: HINSTANCE, mut uID: UINT) -> INT {
    let mut hrSrc: HRSRC = core::ptr::null_mut();
    let mut hRes: HGLOBAL = core::ptr::null_mut();
    let mut lpName: LPWSTR = core::ptr::null_mut();
    let mut lpStr: LPWSTR = core::ptr::null_mut();

    if hInst.is_null() {
        return -1;
    }

    /* There are always blocks of 16 strings */
    lpName = MAKEINTRESOURCE!((((uID >> 4) + 1) as WORD)) as LPWSTR;

    /* Find the string table block */
    if {
        hrSrc = FindResourceW(hInst as HMODULE, lpName as LPCWSTR, (RT_STRING as WORD as ULONG_PTR as LPWSTR) as LPCWSTR);
        !hrSrc.is_null()
    } && {
        hRes = LoadResource(hInst, hrSrc);
        !hRes.is_null()
    } && {
        lpStr = LockResource(hRes) as LPWSTR;
        !lpStr.is_null()
    } {
        let mut x: UINT;

        /* Find the string we're looking for */
        uID &= 0xF as UINT; /* position in the block, same as % 16 */
        x = 0 as UINT;
        while x < uID {
            lpStr = lpStr.offset(((*lpStr) as isize) + (1 as isize));
            x = x.wrapping_add(1);
        }

        /* Found the string */
        return (*lpStr) as INT;
    }
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn AllocAndLoadString(lpTarget: *mut LPTSTR, hInst: HINSTANCE, uID: UINT) -> INT {
    let mut ln: INT;
    ln = LengthOfStrResource(hInst, uID);
    let fresh = ln;
    ln = ln + 1;
    if fresh > 0 {
        *lpTarget = LocalAlloc(
            LMEM_FIXED as UINT,
            (ln as SIZE_T).wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPWSTR as LPTSTR;
        if !(*lpTarget).is_null() {
            let mut Ret: INT;
            Ret = LoadString(hInst, uID, *lpTarget, ln);
            if Ret == 0 {
                LocalFree(*lpTarget as HLOCAL);
            }
            return Ret;
        }
    }
    return 0;
}
// The C-variadic `LoadAndFormatString` is a trampoline in rust_varargs_shim.c, which hands
// over `&ap` -- exactly the `va_list *` FormatMessage's Arguments wants, so it is forwarded
// as is. (The machine port passed the address of a Rust `VaArgs` there: the wrong object,
// and FormatMessage would have read garbage for the first %1.)
#[no_mangle]
pub unsafe extern "C" fn LoadAndFormatString_v(
    hInstance: HINSTANCE,
    uID: UINT,
    lpTarget: *mut LPTSTR,
    ap_list: *mut ::core::ffi::c_void,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let mut lpFormat: LPWSTR = core::ptr::null_mut();
    if AllocAndLoadString(core::ptr::addr_of_mut!(lpFormat), hInstance, uID) != 0 {
        /* let's use FormatMessage to format it because it has the ability to allocate
           memory automatically */
        Ret = FormatMessage(
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING) as DWORD,
            lpFormat as LPCVOID,
            0 as DWORD,
            0 as DWORD,
            lpTarget as LPTSTR,
            0 as DWORD,
            ap_list as *mut va_list,
        );

        LocalFree(lpFormat as HLOCAL);
    }

    return Ret;
}

