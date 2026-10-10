use crate::imm32::imm::ImmLockClientImc;
use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::imm::ImmUnlockClientImc;
use crate::imm32::ime::ImmUnlockImeDpi;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ghImmHeap: HANDLE = NULL;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Imm32CurrentPti() -> PTHREADINFO {
    if (*NtCurrentTeb()).Win32ThreadInfo.is_null() {
        NtUserGetThreadState(THREADSTATE_GETTHREADINFO as DWORD);
    }
    return (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn Imm32CurrentPti() -> PTHREADINFO {
    if (*NtCurrentTeb()).Win32ThreadInfo.is_null() {
        NtUserGetThreadState(THREADSTATE_GETTHREADINFO as DWORD);
    }
    return (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32IsCrossThreadAccess(mut hIMC: HIMC) -> BOOL {
    let mut dwImeThreadId: DWORD_PTR = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID as DWORD);
    let mut dwCurrentThreadId: DWORD_PTR = GetCurrentThreadId() as DWORD_PTR;
    return (dwImeThreadId != dwCurrentThreadId) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32IsCrossProcessAccess(hWnd: HWND) -> BOOL {
    let WndPID: DWORD_PTR = NtUserQueryWindow(hWnd, QUERY_WINDOW_UNIQUE_PROCESS_ID as DWORD);
    let CurrentPID: DWORD_PTR = (*NtCurrentTeb()).ClientId.UniqueProcess as DWORD_PTR;
    return (WndPID != CurrentPID) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32StrToUInt(
    pszText: PCWSTR,
    pdwValue: PDWORD,
    nBase: ULONG,
) -> HRESULT {
    let mut Status: NTSTATUS;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    *pdwValue = 0;
    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeString), pszText);
    Status = RtlUnicodeStringToInteger(
        core::ptr::addr_of_mut!(UnicodeString),
        nBase,
        pdwValue as PULONG,
    );
    if !NT_SUCCESS!(Status) {
        return E_FAIL;
    }
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32UIntToStr(
    dwValue: DWORD,
    nBase: ULONG,
    pszBuff: PWSTR,
    cchBuff: USHORT,
) -> HRESULT {
    let mut Status: NTSTATUS;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    UnicodeString.Buffer = pszBuff;
    UnicodeString.MaximumLength = ((cchBuff as usize) * core::mem::size_of::<WCHAR>()) as USHORT;
    Status = RtlIntegerToUnicodeString(dwValue, nBase, core::ptr::addr_of_mut!(UnicodeString));
    if !NT_SUCCESS!(Status) {
        return E_FAIL;
    }
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32IsSystemJapaneseOrKorean() -> BOOL {
    let lcid: LCID = GetSystemDefaultLCID();
    let LangID: LANGID = LANGIDFROMLCID!(lcid);
    let wPrimary: WORD = PRIMARYLANGID!(LangID);
    if (wPrimary as i32) as u32 != LANG_JAPANESE && (wPrimary as i32) as u32 != LANG_KOREAN {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32IsImcAnsi(hIMC: HIMC) -> BOOL {
    let mut ret: BOOL;
    let mut pClientImc: PCLIENTIMC = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return -1;
    }
    ret = (((*pClientImc).dwFlags & CLIENTIMC_WIDE) == 0) as BOOL;
    ImmUnlockClientImc(pClientImc);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32WideFromAnsi(uCodePage: UINT, pszA: LPCSTR) -> LPWSTR {
    let mut cch: INT = lstrlenA(pszA) as INT;
    let pszW: LPWSTR = ImmLocalAlloc(
        0,
        ((cch + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
    ) as LPWSTR;
    if IS_NULL_UNEXPECTEDLY!(pszW) != false {
        return core::ptr::null_mut();
    }
    cch = MultiByteToWideChar(
        uCodePage,
        MB_PRECOMPOSED,
        pszA,
        cch,
        pszW,
        cch + 1,
    ) as INT;
    *pszW.offset(cch as isize) = UNICODE_NULL;
    return pszW;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32AnsiFromWide(uCodePage: UINT, pszW: LPCWSTR) -> LPSTR {
    let mut cchW: INT = lstrlenW(pszW);
    let mut cchA: INT = ((cchW + 1) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as INT;
    let mut pszA: LPSTR = ImmLocalAlloc(0 as DWORD, cchA as DWORD) as LPSTR;
    if IS_NULL_UNEXPECTEDLY!(pszA) {
        return core::ptr::null_mut::<CHAR>();
    }
    cchA = WideCharToMultiByte(
        uCodePage,
        0 as DWORD,
        pszW,
        cchW as core::ffi::c_int,
        pszA,
        cchA as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as INT;
    *pszA.offset(cchA as isize) = ANSI_NULL;
    return pszA;
}
#[no_mangle]
pub unsafe extern "C" fn IchWideFromAnsi(mut cchAnsi: LONG, mut pchAnsi: LPCSTR, mut uCodePage: UINT) -> LONG {
    let mut cchWide: LONG;
    cchWide = 0;
    while cchAnsi > 0 {
        if IsDBCSLeadByteEx(uCodePage, *pchAnsi as BYTE) != 0 && *pchAnsi.offset(1 as isize) != 0 {
            cchAnsi -= 2;
            pchAnsi = pchAnsi.offset(2 as isize);
        } else {
            cchAnsi -= 1;
            pchAnsi = pchAnsi.offset(1);
        }
        cchWide += 1;
    }
    return cchWide;
}
#[no_mangle]
pub unsafe extern "C" fn IchAnsiFromWide(mut cchWide: LONG, mut pchWide: LPCWSTR, mut uCodePage: UINT) -> LONG {
    let mut cb: LONG = 0;
    let mut cchAnsi: LONG = 0;
    cchAnsi = 0;
    while cchWide > 0 {
        cb = WideCharToMultiByte(
            uCodePage,
            0,
            pchWide,
            1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as LONG;
        if cb > 1 {
            cchAnsi += 1;
        }
        cchAnsi += 1;
        pchWide = pchWide.offset(1);
        cchWide -= 1;
    }
    return cchAnsi;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32GetSystemLibraryPath(mut pszPath: LPWSTR, mut cchPath: DWORD, mut pszFileName: LPCWSTR) -> BOOL {
    if *pszFileName.add(0 as usize) == 0
        || GetSystemDirectoryW(pszPath, cchPath as UINT) == 0
    {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32GetSystemLibraryPath\0" as *const u8
                    as *const ::core::ffi::c_char,
                156 as ::core::ffi::c_int,
                b"Invalid filename\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    StringCchCatW(
        pszPath,
        cchPath as size_t,
        RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
    );
    StringCchCatW(
        pszPath,
        cchPath as size_t,
        pszFileName,
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LogFontAnsiToWide(plfA: *const LOGFONTA, plfW: LPLOGFONTW) {
    let mut cch: size_t = 0;
    RtlCopyMemory!(
        plfW as *mut ::core::ffi::c_void,
        plfA as *const ::core::ffi::c_void,
        core::mem::offset_of!(LOGFONTA, lfFaceName),
    );
    StringCchLengthA(
        core::ptr::addr_of!((*plfA).lfFaceName) as STRSAFE_LPCSTR,
        _countof!((*plfA).lfFaceName) as usize,
        core::ptr::addr_of_mut!(cch),
    );
    cch = MultiByteToWideChar(
        CP_ACP,
        MB_PRECOMPOSED,
        core::ptr::addr_of!((*plfA).lfFaceName) as LPCSTR,
        cch as INT,
        core::ptr::addr_of_mut!((*plfW).lfFaceName) as LPWSTR,
        _countof!((*plfW).lfFaceName) as INT,
    ) as size_t;
    if cch > (_countof!((*plfW).lfFaceName) - 1) as _ {
        cch = (_countof!((*plfW).lfFaceName) - 1) as _;
    }
    (*plfW).lfFaceName[cch as usize] = UNICODE_NULL;
}
#[no_mangle]
pub unsafe extern "C" fn LogFontWideToAnsi(plfW: *const LOGFONTW, plfA: LPLOGFONTA) {
    let mut cch: size_t = 0;
    RtlCopyMemory!(
        plfA as *mut core::ffi::c_void,
        plfW as *const core::ffi::c_void,
        core::mem::offset_of!(LOGFONTW, lfFaceName),
    );
    StringCchLengthW(
        &raw const (*plfW).lfFaceName as *const WCHAR,
        (_countof!((*plfW).lfFaceName)) as _,
        &raw mut cch,
    );
    cch = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw const (*plfW).lfFaceName as LPCWSTR,
        cch as INT,
        &raw mut (*plfA).lfFaceName as LPSTR,
        _countof!((*plfA).lfFaceName) as INT,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as size_t;
    if cch > (_countof!((*plfA).lfFaceName) as size_t) - (1 as size_t) {
        cch = (_countof!((*plfA).lfFaceName) as size_t) - (1 as size_t);
    }
    (*plfA).lfFaceName[cch as usize] = ANSI_NULL as CHAR;
}
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DesktopPtrToUser(mut ptr: PVOID) -> PVOID {
    let mut pci: PCLIENTINFO = GetWin32ClientInfo!();
    let mut pdi: PDESKTOPINFO = (*pci).pDeskInfo;
    if ptr.is_null() {
        RtlAssert(
            b"ptr != NULL\0".as_ptr() as *const i8 as PVOID,
            b"win32ss/user/imm32/utils.c\0".as_ptr() as *const i8 as PVOID,
            193 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if pdi.is_null() {
        RtlAssert(
            b"pdi != NULL\0".as_ptr() as *const i8 as PVOID,
            b"win32ss/user/imm32/utils.c\0".as_ptr() as *const i8 as PVOID,
            194 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pdi).pvDesktopBase <= ptr && ptr < (*pdi).pvDesktopLimit {
        return (ptr as ULONG_PTR).wrapping_sub((*pci).ulClientDelta) as PVOID;
    } else {
        return NtUserCallOneParam(
            ptr as DWORD_PTR,
            ONEPARAM_ROUTINE_GETDESKTOPMAPPING as DWORD,
        ) as PVOID;
    };
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DesktopPtrToUser(mut ptr: PVOID) -> PVOID {
    let mut pci: PCLIENTINFO = GetWin32ClientInfo!();
    let mut pdi: PDESKTOPINFO = (*pci).pDeskInfo;
    if ptr.is_null() {
        RtlAssert(
            b"ptr != NULL\0".as_ptr() as *const i8 as PVOID,
            b"win32ss/user/imm32/utils.c\0".as_ptr() as *const i8 as PVOID,
            193 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if pdi.is_null() {
        RtlAssert(
            b"pdi != NULL\0".as_ptr() as *const i8 as PVOID,
            b"win32ss/user/imm32/utils.c\0".as_ptr() as *const i8 as PVOID,
            194 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*pdi).pvDesktopBase <= ptr && ptr < (*pdi).pvDesktopLimit {
        return (ptr as ULONG_PTR).wrapping_sub((*pci).ulClientDelta) as PVOID;
    } else {
        return NtUserCallOneParam(
            ptr as DWORD_PTR,
            ONEPARAM_ROUTINE_GETDESKTOPMAPPING as DWORD,
        ) as PVOID;
    };
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateHandleNoErr(mut hObject: HANDLE, mut uType: UINT) -> LPVOID {
    let mut index: UINT = 0;
    let mut ht: PUSER_HANDLE_TABLE = core::ptr::null_mut();
    let mut he: PUSER_HANDLE_ENTRY = core::ptr::null_mut();
    let mut generation: WORD = 0;
    let mut ptr: LPVOID = core::ptr::null_mut();

    if NtUserValidateHandleSecure(hObject) == 0 {
        return core::ptr::null_mut();
    }

    ht = gSharedInfo.aheList as PUSER_HANDLE_TABLE;

    he = ((*ht).handles as ULONG_PTR).wrapping_sub(gSharedInfo.ulSharedDelta) as PUSER_HANDLE_ENTRY;

    index = ((LOWORD!(hObject) as INT - (FIRST_USER_HANDLE as i32)) >> 1) as UINT;
    if (index as INT) < 0
        || (*ht).nb_handles as UINT <= index
        || (*he.offset(index as isize)).type_ as UINT != uType
    {
        return core::ptr::null_mut();
    }

    if ((*he.offset(index as isize)).flags as INT & (HANDLEENTRY_DESTROY as i32)) != 0 {
        return core::ptr::null_mut();
    }

    generation = HIWORD!(hObject);
    if (generation as INT) != ((*he.offset(index as isize)).generation as INT)
        && (generation as INT) != 0
        && (generation as INT) != 0xFFFF
    {
        return core::ptr::null_mut();
    }

    ptr = (*he.offset(index as isize)).ptr as LPVOID;
    if !ptr.is_null() {
        ptr = DesktopPtrToUser(ptr as PVOID) as LPVOID;
    }

    return ptr;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateHandleNoErr(mut hObject: HANDLE, mut uType: UINT) -> LPVOID {
    let mut index: UINT = 0;
    let mut ht: PUSER_HANDLE_TABLE = core::ptr::null_mut();
    let mut he: PUSER_HANDLE_ENTRY = core::ptr::null_mut();
    let mut generation: WORD = 0;
    let mut ptr: LPVOID = core::ptr::null_mut();

    if NtUserValidateHandleSecure(hObject) == 0 {
        return core::ptr::null_mut();
    }

    ht = gSharedInfo.aheList as PUSER_HANDLE_TABLE;

    he = ((*ht).handles as ULONG_PTR).wrapping_sub(gSharedInfo.ulSharedDelta) as PUSER_HANDLE_ENTRY;

    index = ((LOWORD!(hObject) as INT - (FIRST_USER_HANDLE as i32)) >> 1) as UINT;
    if (index as INT) < 0
        || (*ht).nb_handles as UINT <= index
        || (*he.offset(index as isize)).type_ as UINT != uType
    {
        return core::ptr::null_mut();
    }

    if ((*he.offset(index as isize)).flags as INT & (HANDLEENTRY_DESTROY as i32)) != 0 {
        return core::ptr::null_mut();
    }

    generation = HIWORD!(hObject);
    if (generation as INT) != ((*he.offset(index as isize)).generation as INT)
        && (generation as INT) != 0
        && (generation as INT) != 0xFFFF
    {
        return core::ptr::null_mut();
    }

    ptr = (*he.offset(index as isize)).ptr as LPVOID;
    if !ptr.is_null() {
        ptr = DesktopPtrToUser(ptr as PVOID) as LPVOID;
    }

    return ptr;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateHandle(mut hObject: HANDLE, mut uType: UINT) -> LPVOID {
    let mut pvObj: LPVOID = ValidateHandleNoErr(hObject, uType);
    if !pvObj.is_null() {
        return pvObj;
    }
    if uType == TYPE_WINDOW as UINT {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE as DWORD);
    } else {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    }
    return core::ptr::null_mut();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateHandle(mut hObject: HANDLE, mut uType: UINT) -> LPVOID {
    let mut pvObj: LPVOID = ValidateHandleNoErr(hObject, uType);
    if !pvObj.is_null() {
        return pvObj;
    }
    if uType == TYPE_WINDOW as UINT {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE as DWORD);
    } else {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CheckImcProcess(pIMC: PIMC) -> BOOL {
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut dwPID1: DWORD_PTR;
    let mut dwPID2: DWORD_PTR;

    if IS_NULL_UNEXPECTEDLY!(pIMC) != false {
        return (FALSE) as BOOL;
    }

    if (*pIMC).head.__bindgen_anon_1.pti == Imm32CurrentPti() {
        return (TRUE) as BOOL;
    }

    hIMC = ((*pIMC).head.__bindgen_anon_1.__bindgen_anon_1.h).cast();
    dwPID1 = NtUserQueryInputContext(hIMC, QIC_INPUTPROCESSID) as DWORD as DWORD_PTR;
    dwPID2 = (*NtCurrentTeb()).ClientId.UniqueProcess as DWORD_PTR;
    if dwPID1 != dwPID2 {
        if (*__wine_dbch___default).flags as i32
            & ((1 as i32) << (__WINE_DBCL_WARN as i32))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const i8,
                b"Imm32CheckImcProcess\0" as *const u8 as *const i8,
                268 as i32,
                b"PID 0x%X != 0x%X\n\0" as *const u8 as *const i8,
                dwPID1,
                dwPID2,
            );
        }
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ImmLocalAlloc(dwFlags: DWORD, dwBytes: DWORD) -> LPVOID {
    if ghImmHeap.is_null() {
        ghImmHeap = RtlGetProcessHeap!();
        if IS_NULL_UNEXPECTEDLY!(ghImmHeap) != false {
            return core::ptr::null_mut();
        }
    }
    return HeapAlloc(ghImmHeap, dwFlags, dwBytes as SIZE_T) as LPVOID;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32MakeIMENotify(hIMC: HIMC, hwnd: HWND, dwAction: DWORD, dwIndex: DWORD, dwValue: DWORD_PTR, dwCommand: DWORD, dwData: DWORD_PTR) -> BOOL {
    let mut dwThreadId: DWORD;
    let mut hKL: HKL;
    let mut pImeDpi: PIMEDPI;

    if dwAction != 0 {
        dwThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
        if dwThreadId != 0 {
            /* find keyboard layout and lock it */
            hKL = GetKeyboardLayout(dwThreadId);
            pImeDpi = ImmLockImeDpi(hKL);
            if !pImeDpi.is_null() {
                /* do notify */
                if (*pImeDpi).NotifyIME.unwrap_unchecked()(
                    hIMC, dwAction, dwIndex, dwValue,
                ) == 0
                {
                    WARN!(c"NotifyIME(%p, %lu, %lu, %p) failed\n".as_ptr(), hIMC, dwAction, dwIndex, dwValue);
                }

                ImmUnlockImeDpi(pImeDpi); /* unlock */
            } else {
            }
        } else {
        }
    } else {
    }

    if !hwnd.is_null() && dwCommand != 0 {
        SendMessageW(
            hwnd,
            WM_IME_NOTIFY as UINT,
            dwCommand as WPARAM,
            dwData as LPARAM,
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32BuildHimcList(dwThreadId: DWORD, pphList: *mut *mut HIMC) -> DWORD {
    let mut Status: NTSTATUS;
    let mut dwCount: DWORD = INITIAL_COUNT;
    let mut cRetry: DWORD = 0;
    let mut phNewList: *mut HIMC;

    phNewList = ImmLocalAlloc(
        0,
        dwCount * core::mem::size_of::<HIMC>() as DWORD,
    ) as *mut HIMC;
    if IS_NULL_UNEXPECTEDLY!(phNewList.is_null()) != false {
        return 0;
    }

    Status = NtUserBuildHimcList(
        dwThreadId,
        dwCount,
        phNewList,
        core::ptr::addr_of_mut!(dwCount),
    );
    while Status == STATUS_BUFFER_TOO_SMALL {
        ImmLocalFree!(phNewList as LPVOID);
        let fresh_retry = cRetry;
        cRetry = cRetry.wrapping_add(1);
        if fresh_retry >= MAX_RETRY {
            return 0;
        }

        phNewList = ImmLocalAlloc(
            0,
            dwCount * core::mem::size_of::<HIMC>() as DWORD,
        ) as *mut HIMC;
        if IS_NULL_UNEXPECTEDLY!(phNewList.is_null()) != false {
            return 0;
        }

        Status = NtUserBuildHimcList(
            dwThreadId,
            dwCount,
            phNewList,
            core::ptr::addr_of_mut!(dwCount),
        );
    }

    if ((Status as ULONG) >> 30) == 3 || dwCount == 0 {
        ImmLocalFree!(phNewList as LPVOID);
        return 0;
    }

    *pphList = phNewList;
    dwCount
}
#[no_mangle]
pub unsafe extern "C" fn Imm32FetchImeState(pIC: LPINPUTCONTEXTDX, hKL: HKL) -> PIME_STATE {
    let mut pState: PIME_STATE = ::core::ptr::null_mut::<IME_STATE>();
    let Lang: WORD = PRIMARYLANGID!(LOWORD!(hKL as DWORD));
    pState = (*pIC).pState as PIME_STATE;
    while !pState.is_null() {
        if (*pState).wLang == Lang {
            break;
        }
        pState = (*pState).pNext as PIME_STATE;
    }
    if pState.is_null() {
        pState = ImmLocalAlloc(
            HEAP_ZERO_MEMORY as DWORD,
            core::mem::size_of::<IME_STATE>() as DWORD,
        ) as PIME_STATE;
        if !pState.is_null() {
            (*pState).wLang = Lang;
            (*pState).pNext = (*pIC).pState;
            (*pIC).pState = pState as *mut IME_STATE;
        }
    }
    return pState;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32FetchImeSubState(pState: PIME_STATE, hKL: HKL) -> PIME_SUBSTATE {
    let mut pSubState: PIME_SUBSTATE;
    pSubState = (*pState).pSubState;
    while !pSubState.is_null() {
        if (*pSubState).hKL == hKL {
            return pSubState;
        }
        pSubState = (*pSubState).pNext as PIME_SUBSTATE;
    }
    pSubState = ImmLocalAlloc(
        0 as DWORD,
        ::core::mem::size_of::<IME_SUBSTATE>() as DWORD,
    ) as PIME_SUBSTATE;
    if pSubState.is_null() {
        return core::ptr::null_mut();
    }
    (*pSubState).dwValue = 0 as DWORD;
    (*pSubState).hKL = hKL;
    (*pSubState).pNext = (*pState).pSubState as *mut IME_SUBSTATE;
    (*pState).pSubState = pSubState;
    return pSubState;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32LoadImeStateSentence(pIC: LPINPUTCONTEXTDX, pState: PIME_STATE, hKL: HKL) -> BOOL {
    let pSubState: PIME_SUBSTATE = Imm32FetchImeSubState(pState, hKL);
    if IS_NULL_UNEXPECTEDLY!(pSubState) != false {
        return (FALSE) as BOOL;
    }

    (*pIC).__bindgen_anon_1.fdwSentence |= (*pSubState).dwValue;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32SaveImeStateSentence(pIC: LPINPUTCONTEXTDX, pState: PIME_STATE, hKL: HKL) -> BOOL {
    let mut pSubState: PIME_SUBSTATE = Imm32FetchImeSubState(pState, hKL);
    if IS_NULL_UNEXPECTEDLY!(pSubState) != false {
        return (FALSE) as BOOL;
    }

    (*pSubState).dwValue = ((*pIC).__bindgen_anon_1.fdwSentence & 0xffff0000) as DWORD;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32ReconvertWideFromAnsi(mut pDest: LPRECONVERTSTRING, mut pSrc: *const RECONVERTSTRING, mut uCodePage: UINT) -> DWORD {
    let mut cch0: DWORD = 0;
    let mut cchDest: DWORD = 0;
    let mut cbDest: DWORD = 0;
    let mut pchSrc: LPCSTR = (pSrc as LPCSTR).offset((*pSrc).dwStrOffset as isize);
    let mut pchDest: LPWSTR = core::ptr::null_mut();

    if (*pSrc).dwVersion != 0 {
        if ((*__wine_dbch___default).flags as core::ffi::c_int
            & ((1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32ReconvertWideFromAnsi\0" as *const u8 as *const core::ffi::c_char,
                458 as core::ffi::c_int,
                b"\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return 0;
    }

    cchDest = MultiByteToWideChar(
        uCodePage,
        MB_PRECOMPOSED,
        pchSrc,
        (*pSrc).dwStrLen as core::ffi::c_int,
        core::ptr::null_mut(),
        0,
    ) as DWORD;
    cbDest = (core::mem::size_of::<RECONVERTSTRING>()
        + (cchDest.wrapping_add(1) as usize)
            * core::mem::size_of::<WCHAR>()) as DWORD;
    if pDest.is_null() {
        return cbDest;
    }

    if (*pDest).dwSize < cbDest {
        if ((*__wine_dbch___default).flags as core::ffi::c_int
            & ((1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32ReconvertWideFromAnsi\0" as *const u8 as *const core::ffi::c_char,
                469 as core::ffi::c_int,
                b"Too small\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return 0;
    }

    /* dwSize */
    (*pDest).dwSize = cbDest;

    /* dwVersion */
    (*pDest).dwVersion = 0;

    /* dwStrOffset */
    (*pDest).dwStrOffset = core::mem::size_of::<RECONVERTSTRING>() as DWORD;

    /* dwCompStrOffset */
    cch0 = IchWideFromAnsi((*pSrc).dwCompStrOffset as LONG, pchSrc, uCodePage) as DWORD;
    (*pDest).dwCompStrOffset = cch0.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);

    /* dwCompStrLen */
    cch0 = IchWideFromAnsi(
        (*pSrc).dwCompStrOffset.wrapping_add((*pSrc).dwCompStrLen) as LONG,
        pchSrc,
        uCodePage,
    ) as DWORD;
    (*pDest).dwCompStrLen = cch0
        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD)
        .wrapping_sub((*pDest).dwCompStrOffset)
        .wrapping_div(core::mem::size_of::<WCHAR>() as DWORD);

    /* dwTargetStrOffset */
    cch0 = IchWideFromAnsi((*pSrc).dwTargetStrOffset as LONG, pchSrc, uCodePage) as DWORD;
    (*pDest).dwTargetStrOffset = cch0.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);

    /* dwTargetStrLen */
    cch0 = IchWideFromAnsi(
        (*pSrc).dwTargetStrOffset.wrapping_add((*pSrc).dwTargetStrLen) as LONG,
        pchSrc,
        uCodePage,
    ) as DWORD;
    (*pDest).dwTargetStrLen = cch0
        .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD)
        .wrapping_sub((*pDest).dwTargetStrOffset)
        .wrapping_div(core::mem::size_of::<WCHAR>() as DWORD);

    /* dwStrLen */
    (*pDest).dwStrLen = cchDest;

    /* the string */
    pchDest = ((pDest as LPBYTE).offset((*pDest).dwStrOffset as isize)) as LPWSTR;
    cchDest = MultiByteToWideChar(
        uCodePage,
        MB_PRECOMPOSED,
        pchSrc,
        (*pSrc).dwStrLen as core::ffi::c_int,
        pchDest,
        cchDest as core::ffi::c_int,
    ) as DWORD;
    *pchDest.offset(cchDest as isize) = UNICODE_NULL;

    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"win32ss/user/imm32/utils.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"Imm32ReconvertWideFromAnsi\0" as *const u8 as *const core::ffi::c_char,
            507 as core::ffi::c_int,
            b"cbDest: 0x%X\n\0" as *const u8 as *const core::ffi::c_char,
            cbDest,
        );
    }

    return cbDest;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32ReconvertAnsiFromWide(mut pDest: LPRECONVERTSTRING, mut pSrc: *const RECONVERTSTRING, mut uCodePage: UINT) -> DWORD {
    let mut cch0: DWORD = 0;
    let mut cch1: DWORD = 0;
    let mut cchDest: DWORD = 0;
    let mut cbDest: DWORD = 0;
    let mut pchSrc: LPCWSTR = (pSrc as LPCSTR).offset((*pSrc).dwStrOffset as isize) as LPCWSTR;
    let mut pchDest: LPSTR = ::core::ptr::null_mut::<CHAR>();

    if (*pSrc).dwVersion != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32ReconvertAnsiFromWide\0" as *const u8
                    as *const ::core::ffi::c_char,
                520 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    cchDest = WideCharToMultiByte(
        uCodePage,
        0,
        pchSrc,
        (*pSrc).dwStrLen as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        0,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    cbDest = (core::mem::size_of::<RECONVERTSTRING>() as usize).wrapping_add(
        (cchDest.wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<CHAR>() as usize),
    ) as DWORD;
    if pDest.is_null() {
        return cbDest;
    }

    if (*pDest).dwSize < cbDest {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/utils.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32ReconvertAnsiFromWide\0" as *const u8
                    as *const ::core::ffi::c_char,
                532 as ::core::ffi::c_int,
                b"Too small\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    /* dwSize */
    (*pDest).dwSize = cbDest;

    /* dwVersion */
    (*pDest).dwVersion = 0;

    /* dwStrOffset */
    (*pDest).dwStrOffset = core::mem::size_of::<RECONVERTSTRING>() as DWORD;

    /* dwCompStrOffset */
    cch1 = ((*pSrc).dwCompStrOffset as usize).wrapping_div(core::mem::size_of::<WCHAR>() as usize)
        as DWORD;
    cch0 = IchAnsiFromWide(cch1 as LONG, pchSrc, uCodePage) as DWORD;
    (*pDest).dwCompStrOffset =
        (cch0 as usize).wrapping_mul(core::mem::size_of::<CHAR>() as usize) as DWORD;

    /* dwCompStrLen */
    cch0 = IchAnsiFromWide(
        cch1.wrapping_add((*pSrc).dwCompStrLen) as LONG,
        pchSrc,
        uCodePage,
    ) as DWORD;
    (*pDest).dwCompStrLen = (cch0 as usize)
        .wrapping_mul(core::mem::size_of::<CHAR>() as usize)
        .wrapping_sub((*pDest).dwCompStrOffset as usize) as DWORD;

    /* dwTargetStrOffset */
    cch1 = ((*pSrc).dwTargetStrOffset as usize)
        .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD;
    cch0 = IchAnsiFromWide(cch1 as LONG, pchSrc, uCodePage) as DWORD;
    (*pDest).dwTargetStrOffset =
        (cch0 as usize).wrapping_mul(core::mem::size_of::<CHAR>() as usize) as DWORD;

    /* dwTargetStrLen */
    cch0 = IchAnsiFromWide(
        cch1.wrapping_add((*pSrc).dwTargetStrLen) as LONG,
        pchSrc,
        uCodePage,
    ) as DWORD;
    (*pDest).dwTargetStrLen = (cch0 as usize)
        .wrapping_mul(core::mem::size_of::<CHAR>() as usize)
        .wrapping_sub((*pDest).dwTargetStrOffset as usize) as DWORD;

    /* dwStrLen */
    (*pDest).dwStrLen = cchDest;

    /* the string */
    pchDest = (pDest as LPSTR).offset((*pDest).dwStrOffset as isize);
    cchDest = WideCharToMultiByte(
        uCodePage,
        0,
        pchSrc,
        (*pSrc).dwStrLen as ::core::ffi::c_int,
        pchDest,
        cchDest as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    *pchDest.offset(cchDest as isize) = ANSI_NULL;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"win32ss/user/imm32/utils.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Imm32ReconvertAnsiFromWide\0" as *const u8 as *const ::core::ffi::c_char,
            572 as ::core::ffi::c_int,
            b"cchDest: 0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
            cchDest,
        );
    }

    return cbDest;
}

#[no_mangle]
pub unsafe extern "system" fn ImmCreateIMCC(mut size: DWORD) -> HIMCC {
    if size < core::mem::size_of::<DWORD>() as DWORD {
        size = core::mem::size_of::<DWORD>() as DWORD;
    }
    return LocalAlloc(LHND as UINT, size as SIZE_T) as HIMCC;
}

#[no_mangle]
pub unsafe extern "system" fn ImmDestroyIMCC(block: HIMCC) -> HIMCC {
    if !block.is_null() {
        return LocalFree(block as HLOCAL) as HIMCC;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ImmLockIMCC(imcc: HIMCC) -> LPVOID {
    if !imcc.is_null() {
        return LocalLock(imcc as HLOCAL) as LPVOID;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnlockIMCC(imcc: HIMCC) -> BOOL {
    if !imcc.is_null() {
        return LocalUnlock(imcc as HLOCAL);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetIMCCLockCount(imcc: HIMCC) -> DWORD {
    return (LocalFlags(imcc as HLOCAL) & LMEM_LOCKCOUNT as UINT) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn ImmReSizeIMCC(imcc: HIMCC, size: DWORD) -> HIMCC {
    if imcc.is_null() {
        return core::ptr::null_mut();
    }
    return LocalReAlloc(imcc as HLOCAL, size as SIZE_T, LHND as UINT) as HIMCC;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetIMCCSize(imcc: HIMCC) -> DWORD {
    if !imcc.is_null() {
        return LocalSize(imcc as HLOCAL) as DWORD;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetIMCLockCount(hIMC: HIMC) -> DWORD {
    let mut ret: DWORD;
    let mut hInputContext: HANDLE;
    let mut pClientImc: PCLIENTIMC;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return 0;
    }

    ret = 0;
    hInputContext = (*pClientImc).hInputContext;
    if !hInputContext.is_null() {
        ret = ((LocalFlags(hInputContext as HLOCAL) & (LMEM_LOCKCOUNT as UINT)) as DWORD);
    }

    ImmUnlockClientImc(pClientImc);
    return ret;
}
