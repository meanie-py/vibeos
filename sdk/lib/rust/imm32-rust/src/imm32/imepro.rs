use crate::imm32::ime::ImmGetImeInfoEx;
use crate::imm32::ime::ImmIsIME;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Imm32ConvertImeProWideToAnsi(pProW: *const IMEPROW, pProA: PIMEPROA) {
    (*pProA).hWnd = (*pProW).hWnd;
    (*pProA).InstDate = (*pProW).InstDate;
    (*pProA).wVersion = (*pProW).wVersion;

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw const (*pProW).szDescription as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut (*pProA).szDescription as *mut BYTE as LPSTR,
        _countof!((*pProA).szDescription) as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    (*pProA).szDescription[(_countof!((*pProA).szDescription) as usize)
        .wrapping_sub(1 as usize)] = ANSI_NULL as BYTE;

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw const (*pProW).szName as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut (*pProA).szName as *mut BYTE as LPSTR,
        _countof!((*pProA).szName) as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    (*pProA).szName[(_countof!((*pProA).szName) as usize)
        .wrapping_sub(1 as usize)] = ANSI_NULL as BYTE;

    (*pProA).szOptions[0 as usize] = ANSI_NULL as BYTE;
}

unsafe extern "C" fn Imm32IMPGetIME(mut hKL: HKL, mut pProW: PIMEPROW) -> BOOL {
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    if ImmGetImeInfoEx(
        &raw mut ImeInfoEx,
        ImeInfoExKeyboardLayout,
        &raw mut hKL as PVOID,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    (*pProW).hWnd = core::ptr::null_mut::<HWND__>();
    ZeroMemory!(
        &raw mut (*pProW).InstDate as PVOID,
        core::mem::size_of::<DATETIME>() as size_t,
    );
    (*pProW).wVersion = ImeInfoEx.dwImeWinVersion as UINT;

    StringCchCopyNW(
        &raw mut (*pProW).szDescription as STRSAFE_LPWSTR,
        _countof!((*pProW).szDescription) as size_t,
        &raw mut ImeInfoEx.wszImeDescription as *mut WCHAR as STRSAFE_LPCWSTR,
        _countof!(ImeInfoEx.wszImeDescription) as size_t,
    );
    StringCchCopyNW(
        &raw mut (*pProW).szName as STRSAFE_LPWSTR,
        _countof!((*pProW).szName) as size_t,
        &raw mut ImeInfoEx.wszImeFile as *mut WCHAR as STRSAFE_LPCWSTR,
        _countof!(ImeInfoEx.wszImeFile) as size_t,
    );
    (*pProW).szOptions[0] = UNICODE_NULL;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPGetIMEA(hWnd: HWND, pImePro: LPIMEPROA) -> BOOL {
    UNREFERENCED_PARAMETER!(hWnd);

    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return (FALSE) as BOOL;
    }

    let mut ImeProW: IMEPROW = core::mem::zeroed();
    let hKL: HKL = GetKeyboardLayout(0);
    if Imm32IMPGetIME(hKL, core::ptr::addr_of_mut!(ImeProW)) == 0 {
        return (FALSE) as BOOL;
    }

    Imm32ConvertImeProWideToAnsi(core::ptr::addr_of_mut!(ImeProW), pImePro);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPGetIMEW(hWnd: HWND, pImePro: LPIMEPROW) -> BOOL {
    UNREFERENCED_PARAMETER!(hWnd);

    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    }

    let hKL: HKL = GetKeyboardLayout(0 as DWORD);
    return Imm32IMPGetIME(hKL, pImePro);
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPQueryIMEA(pImePro: LPIMEPROA) -> BOOL {
    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    }

    let mut ProW: IMEPROW = core::mem::zeroed();
    if (*pImePro).szName[0 as usize] != 0 {
        if MultiByteToWideChar(
            CP_ACP as UINT,
            MB_PRECOMPOSED as DWORD,
            core::ptr::addr_of_mut!((*pImePro).szName) as PSTR as LPCSTR,
            -(1 as i32),
            core::ptr::addr_of_mut!(ProW.szName) as LPWSTR,
            _countof!(ProW.szName) as i32,
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        ProW.szName[_countof!(ProW.szName) as usize - 1] =
            UNICODE_NULL;
    } else {
        ProW.szName[0 as usize] = UNICODE_NULL;
    }

    if ImmIMPQueryIMEW(core::ptr::addr_of_mut!(ProW)) == 0 {
        return (FALSE) as BOOL;
    }

    Imm32ConvertImeProWideToAnsi(core::ptr::addr_of_mut!(ProW), pImePro);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPQueryIMEW(pImePro: LPIMEPROW) -> BOOL {
    if ((Imm32IsSystemJapaneseOrKorean()) == 0) {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return (FALSE) as BOOL;
    }

    let mut nLayouts: INT = GetKeyboardLayoutList(0, core::ptr::null_mut()) as INT;
    if nLayouts <= 0 {
        return (FALSE) as BOOL;
    }

    let mut phKLs: *mut HKL = ImmLocalAlloc(
        0,
        (nLayouts as usize).wrapping_mul(core::mem::size_of::<HKL>()) as DWORD,
    ) as *mut HKL;
    if phKLs.is_null() {
        return (FALSE) as BOOL;
    }

    if GetKeyboardLayoutList(nLayouts, phKLs) as INT != nLayouts {
        ImmLocalFree!(phKLs as PVOID);
        return (FALSE) as BOOL;
    }

    let mut result: BOOL = (FALSE) as i32;
    if (*pImePro).szName[0 as usize] != 0 {
        let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
        if ImmGetImeInfoEx(
            core::ptr::addr_of_mut!(ImeInfoEx),
            ImeInfoExImeFileName,
            core::ptr::addr_of_mut!((*pImePro).szName) as PVOID,
        ) != 0
        {
            let mut iKL: INT = 0;
            while iKL < nLayouts {
                if *phKLs.offset(iKL as isize) == ImeInfoEx.hkl {
                    result = Imm32IMPGetIME(*phKLs.offset(iKL as isize), pImePro);
                    break;
                }
                iKL += 1;
            }
        }
    } else {
        let mut iKL: INT = 0;
        while iKL < nLayouts {
            result = Imm32IMPGetIME(*phKLs.offset(iKL as isize), pImePro);
            if result != 0 {
                break;
            }
            iKL += 1;
        }
    }

    ImmLocalFree!(phKLs as PVOID);
    return result;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPSetIMEA(hWnd: HWND, pImePro: LPIMEPROA) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/imepro.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmIMPSetIMEA\0" as *const u8 as *const ::core::ffi::c_char,
            232 as ::core::ffi::c_int,
            b"(%p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hWnd,
            pImePro,
        );
    }
    if pImePro.is_null() {
        RtlAssert(
            b"pImePro\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"win32ss/user/imm32/imepro.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            234 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    let mut ProW: IMEPROW = core::mem::zeroed();

    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    }

    if (*pImePro).szName[0 as usize] != 0 {
        /* pImePro->szName is BYTE[], so we need type cast */
        if MultiByteToWideChar(
            CP_ACP as UINT,
            MB_PRECOMPOSED as DWORD,
            &raw mut (*pImePro).szName as *mut BYTE as PSTR as LPCSTR,
            -1 as ::core::ffi::c_int,
            &raw mut ProW.szName as LPWSTR,
            _countof!(ProW.szName) as ::core::ffi::c_int,
        ) == 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/user/imm32/imepro.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ImmIMPSetIMEA\0" as *const u8 as *const ::core::ffi::c_char,
                    249 as ::core::ffi::c_int,
                    b"szName: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_a(
                        &raw mut (*pImePro).szName as *mut BYTE as PSTR
                            as *const ::core::ffi::c_char,
                    ),
                );
            }
            return (FALSE) as BOOL;
        }
        ProW.szName[(_countof!(ProW.szName) as usize)
            - 1 as usize] = UNICODE_NULL as WCHAR; /* Avoid buffer overrun */
    } else {
        ProW.szName[0 as usize] = UNICODE_NULL as WCHAR;
    }

    return ImmIMPSetIMEW(hWnd, &raw mut ProW);
}

#[no_mangle]
pub unsafe extern "system" fn ImmIMPSetIMEW(hWnd: HWND, pImePro: LPIMEPROW) -> BOOL {
    let _ = hWnd;

    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
        return (FALSE) as BOOL;
    }

    let mut hTargetKL: HKL = core::ptr::null_mut();
    if (*pImePro).szName[0 as usize] != 0 {
        let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
        if ImmGetImeInfoEx(
            core::ptr::addr_of_mut!(ImeInfoEx),
            ImeInfoExImeFileName,
            core::ptr::addr_of_mut!((*pImePro).szName) as PVOID,
        ) == 0
        {
            return (FALSE) as BOOL;
        }

        hTargetKL = ImeInfoEx.hkl;
    } else {
        let mut nLayouts: INT = GetKeyboardLayoutList(0 as INT, core::ptr::null_mut()) as INT;
        if nLayouts <= 0 {
            return (FALSE) as BOOL;
        }

        let phKLs: *mut HKL = ImmLocalAlloc(
            0 as DWORD,
            (nLayouts as usize).wrapping_mul(core::mem::size_of::<HKL>()) as DWORD,
        ) as *mut HKL;
        if phKLs.is_null() {
            return (FALSE) as BOOL;
        }

        if GetKeyboardLayoutList(nLayouts as INT, phKLs) == nLayouts as UINT {
            let mut iKL: INT = 0 as INT;
            while iKL < nLayouts {
                if ImmIsIME(*phKLs.offset(iKL as isize)) == 0 {
                    hTargetKL = *phKLs.offset(iKL as isize);
                    break;
                }
                iKL += 1;
            }
        } else {
        }

        ImmLocalFree!(phKLs as PVOID);
    }

    if !hTargetKL.is_null() && GetKeyboardLayout(0 as DWORD) != hTargetKL {
        let hwndFocus: HWND = GetFocus();
        if !hwndFocus.is_null() {
            PostMessageW(
                hwndFocus,
                WM_INPUTLANGCHANGEREQUEST as UINT,
                INPUTLANGCHANGE_SYSCHARSET as WPARAM,
                hTargetKL as LPARAM,
            );
            return (TRUE) as BOOL;
        }
    }

    (FALSE) as i32
}
