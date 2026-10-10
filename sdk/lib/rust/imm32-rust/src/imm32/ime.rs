use crate::imm32::ctf::CtfImmIsTextFrameServiceDisabled;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut gcsImeDpi: RTL_CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

#[no_mangle]
pub static mut gpImeDpiList: PIMEDPI = ::core::ptr::null::<IMEDPI>() as *mut IMEDPI;

// WINE_DEFAULT_DEBUG_CHANNEL(imm): debug-channel registration, no runtime effect here.
// DEFINE_IME_ENTRY: macro is #undef'd immediately, so nothing to emit.
#[no_mangle]
pub unsafe extern "C" fn Imm32FindImeDpi(hKL: HKL) -> PIMEDPI {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut::<IMEDPI>();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
    pImeDpi = gpImeDpiList;
    while !pImeDpi.is_null() {
        if (*pImeDpi).hKL == hKL {
            break;
        }
        pImeDpi = (*pImeDpi).pNext as PIMEDPI;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    return pImeDpi;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32FreeIME(pImeDpi: PIMEDPI, bDestroy: BOOL) {
    if (*pImeDpi).hInst.is_null() {
        return;
    }
    if bDestroy != 0 {
        (*pImeDpi).ImeDestroy.unwrap_unchecked()(0 as UINT);
    }
    FreeLibrary((*pImeDpi).hInst as HMODULE);
    (*pImeDpi).hInst = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn Imm32InquireIme(mut pImeDpi: PIMEDPI) -> BOOL {
    let mut szUIClass: [WCHAR; 64] = [0; 64];
    let mut wcW: WNDCLASSW = core::mem::zeroed();
    let mut dwSysInfoFlags: DWORD = 0 as DWORD;
    let mut pImeInfo: LPIMEINFO = &raw mut (*pImeDpi).ImeInfo;

    if NtUserGetThreadState(THREADSTATE_ISWINLOGON as DWORD) != 0 {
        dwSysInfoFlags |= IME_SYSINFO_WINLOGON as DWORD;
    }

    if ((*GetWin32ClientInfo!()).dwTIFlags & (TIF_16BIT as DWORD)) != 0 {
        dwSysInfoFlags |= IME_SYSINFO_WOW16 as DWORD;
    }

    if IS_IME_HKL!((*pImeDpi).hKL) != false {
        if (*pImeDpi).ImeInquire.unwrap_unchecked()(
            pImeInfo,
            &raw mut szUIClass as *mut WCHAR as LPVOID,
            dwSysInfoFlags,
        ) == 0
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"win32ss/user/imm32/ime.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                    62 as ::core::ffi::c_int,
                    b"\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
    } else if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == false {
        if (*pImeDpi).CtfImeInquireExW.unwrap_unchecked()(
            pImeInfo,
            &raw mut szUIClass as *mut WCHAR as LPVOID,
            dwSysInfoFlags,
            (*pImeDpi).hKL,
        ) != S_OK
        {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"win32ss/user/imm32/ime.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                    70 as ::core::ffi::c_int,
                    b"\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
    } else {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                76 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }

    szUIClass[(_countof!(szUIClass) as usize) - (1 as usize)] =
        UNICODE_NULL;

    if ((*pImeInfo).dwPrivateDataSize as usize) < core::mem::size_of::<DWORD>() {
        (*pImeInfo).dwPrivateDataSize = core::mem::size_of::<DWORD>() as DWORD;
    }

    if ((*pImeInfo).fdwProperty & !(VALID_IME_PROP as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                126 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    if ((*pImeInfo).fdwConversionCaps & !(VALID_CMODE_CAPS as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                131 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    if ((*pImeInfo).fdwSentenceCaps & !(VALID_SMODE_CAPS as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                136 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    if ((*pImeInfo).fdwUICaps & !(VALID_UI_CAPS as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                141 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    if ((*pImeInfo).fdwSCSCaps & !(VALID_SCS_CAPS as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                146 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }
    if ((*pImeInfo).fdwSelectCaps & !(VALID_SELECT_CAPS as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                151 as ::core::ffi::c_int,
                b"Bad flags\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }

    if ((*pImeInfo).fdwProperty & (IME_PROP_UNICODE as DWORD)) != 0 {
        StringCchCopyW(
            &raw mut (*pImeDpi).szUIClass as STRSAFE_LPWSTR,
            (_countof!((*pImeDpi).szUIClass) as size_t),
            &raw mut szUIClass as *mut WCHAR as STRSAFE_LPCWSTR,
        );
    } else {
        if (*pImeDpi).uCodePage != GetACP()
            && (*pImeDpi).uCodePage != (CP_ACP as UINT)
        {
            return (FALSE) as BOOL;
        }
        MultiByteToWideChar(
            CP_ACP as UINT,
            MB_PRECOMPOSED as DWORD,
            &raw mut szUIClass as *mut WCHAR as LPSTR as LPCSTR,
            -(1 as ::core::ffi::c_int),
            &raw mut (*pImeDpi).szUIClass as LPWSTR,
            _countof!((*pImeDpi).szUIClass) as ::core::ffi::c_int,
        );
        (*pImeDpi).szUIClass[(_countof!((*pImeDpi).szUIClass) as usize)
            - (1 as usize)] = UNICODE_NULL;
    }

    if GetClassInfoW(
        (*pImeDpi).hInst,
        &raw mut (*pImeDpi).szUIClass as *mut WCHAR as LPCWSTR,
        &raw mut wcW,
    ) == 0
    {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_ERR as ::core::ffi::c_int)))
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32InquireIme\0" as *const u8 as *const ::core::ffi::c_char,
                179 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn Imm32LoadIME(mut pImeInfoEx: PIMEINFOEX, mut pImeDpi: PIMEDPI) -> BOOL {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hIME: HINSTANCE = ::core::ptr::null_mut::<HINSTANCE__>();
    let mut fn_0: FARPROC = None;
    let mut ret: BOOL = (FALSE) as i32;
    let mut failed: bool = false;

    if Imm32GetSystemLibraryPath(
        &raw mut szPath as LPWSTR,
        _countof!(szPath) as DWORD,
        &raw mut (*pImeInfoEx).wszImeFile as *mut WCHAR as LPCWSTR,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    hIME = LoadLibraryW(&raw mut szPath as *mut WCHAR as LPCWSTR);
    (*pImeDpi).hInst = hIME;
    if hIME.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                209 as ::core::ffi::c_int,
                b"LoadLibraryW(%s) failed\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(&raw mut szPath as *mut WCHAR),
            );
        }
        return (FALSE) as BOOL;
    }

    /* Populate the table by stub IME functions */
    (*pImeDpi).ImeInquire = Some(StubImeInquire);
    (*pImeDpi).ImeConversionList = Some(StubImeConversionList);
    (*pImeDpi).ImeRegisterWord = Some(StubImeRegisterWord);
    (*pImeDpi).ImeUnregisterWord = Some(StubImeUnregisterWord);
    (*pImeDpi).ImeGetRegisterWordStyle = Some(StubImeGetRegisterWordStyle);
    (*pImeDpi).ImeEnumRegisterWord = Some(StubImeEnumRegisterWord);
    (*pImeDpi).ImeConfigure = Some(StubImeConfigure);
    (*pImeDpi).ImeDestroy = Some(StubImeDestroy);
    (*pImeDpi).ImeEscape = Some(StubImeEscape);
    (*pImeDpi).ImeProcessKey = Some(StubImeProcessKey);
    (*pImeDpi).ImeSelect = Some(StubImeSelect);
    (*pImeDpi).ImeSetActiveContext = Some(StubImeSetActiveContext);
    (*pImeDpi).ImeToAsciiEx = Some(StubImeToAsciiEx);
    (*pImeDpi).NotifyIME = Some(StubNotifyIME);
    (*pImeDpi).ImeSetCompositionString = Some(StubImeSetCompositionString);
    (*pImeDpi).ImeGetImeMenuItems = Some(StubImeGetImeMenuItems);
    (*pImeDpi).CtfImeInquireExW = Some(StubCtfImeInquireExW);
    (*pImeDpi).CtfImeSelectEx = Some(StubCtfImeSelectEx);
    (*pImeDpi).CtfImeEscapeEx = Some(StubCtfImeEscapeEx);
    (*pImeDpi).CtfImeGetGuidAtom = Some(StubCtfImeGetGuidAtom);
    (*pImeDpi).CtfImeIsGuidMapEnable = Some(StubCtfImeIsGuidMapEnable);

    /* Populate the table by real IME functions */
    'body: {
        fn_0 = GetProcAddress(hIME, b"ImeInquire\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeInquire = ::core::mem::transmute::<FARPROC, FN_ImeInquire>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeInquire\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeConversionList\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeConversionList = ::core::mem::transmute::<FARPROC, FN_ImeConversionList>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    3 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeConversionList\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeRegisterWord\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeRegisterWord = ::core::mem::transmute::<FARPROC, FN_ImeRegisterWord>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    4 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeRegisterWord\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeUnregisterWord\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeUnregisterWord = ::core::mem::transmute::<FARPROC, FN_ImeUnregisterWord>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    5 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeUnregisterWord\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeGetRegisterWordStyle\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeGetRegisterWordStyle = ::core::mem::transmute::<FARPROC, FN_ImeGetRegisterWordStyle>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    6 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeGetRegisterWordStyle\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeEnumRegisterWord\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeEnumRegisterWord = ::core::mem::transmute::<FARPROC, FN_ImeEnumRegisterWord>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    7 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeEnumRegisterWord\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeConfigure\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeConfigure = ::core::mem::transmute::<FARPROC, FN_ImeConfigure>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    8 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeConfigure\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeDestroy\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeDestroy = ::core::mem::transmute::<FARPROC, FN_ImeDestroy>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    9 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeDestroy\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeEscape\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeEscape = ::core::mem::transmute::<FARPROC, FN_ImeEscape>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    10 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeEscape\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeProcessKey\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeProcessKey = ::core::mem::transmute::<FARPROC, FN_ImeProcessKey>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    11 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeProcessKey\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeSelect\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeSelect = ::core::mem::transmute::<FARPROC, FN_ImeSelect>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    12 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeSelect\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeSetActiveContext\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeSetActiveContext = ::core::mem::transmute::<FARPROC, FN_ImeSetActiveContext>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    13 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeSetActiveContext\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeToAsciiEx\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeToAsciiEx = ::core::mem::transmute::<FARPROC, FN_ImeToAsciiEx>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    14 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeToAsciiEx\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"NotifyIME\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).NotifyIME = ::core::mem::transmute::<FARPROC, FN_NotifyIME>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    15 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"NotifyIME\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        fn_0 = GetProcAddress(hIME, b"ImeSetCompositionString\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeSetCompositionString = ::core::mem::transmute::<FARPROC, FN_ImeSetCompositionString>(fn_0);
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"sdk/include/reactos/imetable.h\0" as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                    16 as ::core::ffi::c_int,
                    b"'%s' not found in IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    b"ImeSetCompositionString\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w(&raw mut szPath as *mut WCHAR),
                );
            }
            failed = true;
            break 'body;
        }

        /* optional entries: no error when missing */
        fn_0 = GetProcAddress(hIME, b"ImeGetImeMenuItems\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).ImeGetImeMenuItems = ::core::mem::transmute::<FARPROC, FN_ImeGetImeMenuItems>(fn_0);
        }

        fn_0 = GetProcAddress(hIME, b"CtfImeInquireExW\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).CtfImeInquireExW = ::core::mem::transmute::<FARPROC, FN_CtfImeInquireExW>(fn_0);
        }

        fn_0 = GetProcAddress(hIME, b"CtfImeSelectEx\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).CtfImeSelectEx = ::core::mem::transmute::<FARPROC, FN_CtfImeSelectEx>(fn_0);
        }

        fn_0 = GetProcAddress(hIME, b"CtfImeEscapeEx\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).CtfImeEscapeEx = ::core::mem::transmute::<FARPROC, FN_CtfImeEscapeEx>(fn_0);
        }

        fn_0 = GetProcAddress(hIME, b"CtfImeGetGuidAtom\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).CtfImeGetGuidAtom = ::core::mem::transmute::<FARPROC, FN_CtfImeGetGuidAtom>(fn_0);
        }

        fn_0 = GetProcAddress(hIME, b"CtfImeIsGuidMapEnable\0" as *const u8 as LPCSTR);
        if fn_0.is_some() {
            (*pImeDpi).CtfImeIsGuidMapEnable = ::core::mem::transmute::<FARPROC, FN_CtfImeIsGuidMapEnable>(fn_0);
        }

        /* Check for Cicero IMEs */
        if ((IS_IME_HKL!((*pImeDpi).hKL)) == false)
            && IS_CICERO_MODE!()
            && ((IS_CICERO_COMPAT_DISABLED!()) == (((false) as u32) != 0))
        {
            /* CHECK_IME_FN(CtfImeInquireExW) */
            if (*pImeDpi).CtfImeInquireExW.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                        240 as ::core::ffi::c_int,
                        b"'%s' not found in Cicero IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CtfImeInquireExW\0" as *const u8 as *const ::core::ffi::c_char,
                        debugstr_w(&raw mut szPath as *mut WCHAR),
                    );
                }
                failed = true;
                break 'body;
            }
            /* CHECK_IME_FN(CtfImeSelectEx) */
            if (*pImeDpi).CtfImeSelectEx.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                        241 as ::core::ffi::c_int,
                        b"'%s' not found in Cicero IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CtfImeSelectEx\0" as *const u8 as *const ::core::ffi::c_char,
                        debugstr_w(&raw mut szPath as *mut WCHAR),
                    );
                }
                failed = true;
                break 'body;
            }
            /* CHECK_IME_FN(CtfImeEscapeEx) */
            if (*pImeDpi).CtfImeEscapeEx.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                        242 as ::core::ffi::c_int,
                        b"'%s' not found in Cicero IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CtfImeEscapeEx\0" as *const u8 as *const ::core::ffi::c_char,
                        debugstr_w(&raw mut szPath as *mut WCHAR),
                    );
                }
                failed = true;
                break 'body;
            }
            /* CHECK_IME_FN(CtfImeGetGuidAtom) */
            if (*pImeDpi).CtfImeGetGuidAtom.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                        243 as ::core::ffi::c_int,
                        b"'%s' not found in Cicero IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CtfImeGetGuidAtom\0" as *const u8 as *const ::core::ffi::c_char,
                        debugstr_w(&raw mut szPath as *mut WCHAR),
                    );
                }
                failed = true;
                break 'body;
            }
            /* CHECK_IME_FN(CtfImeIsGuidMapEnable) */
            if (*pImeDpi).CtfImeIsGuidMapEnable.is_none() {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"win32ss/user/imm32/ime.c\0" as *const u8 as *const ::core::ffi::c_char,
                        b"Imm32LoadIME\0" as *const u8 as *const ::core::ffi::c_char,
                        244 as ::core::ffi::c_int,
                        b"'%s' not found in Cicero IME module %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
                        b"CtfImeIsGuidMapEnable\0" as *const u8 as *const ::core::ffi::c_char,
                        debugstr_w(&raw mut szPath as *mut WCHAR),
                    );
                }
                failed = true;
                break 'body;
            }
        }
    }

    if !failed {
        if Imm32InquireIme(pImeDpi) != 0 {
            ret = (TRUE) as i32;
        } else {
            failed = true;
        }
    }
    if failed {
        ret = (FALSE) as i32;
        FreeLibrary((*pImeDpi).hInst as HMODULE);
        (*pImeDpi).hInst = ::core::ptr::null_mut::<HINSTANCE__>();
    }

    if (*pImeInfoEx).fLoadFlag == 0 {
        if ret != 0 {
            C_ASSERT!(
                ::core::mem::size_of::<[WCHAR; 16]>() == ::core::mem::size_of::<[WCHAR; 16]>(),
            );
            (*pImeInfoEx).ImeInfo = (*pImeDpi).ImeInfo;
            RtlCopyMemory!(
                &raw mut (*pImeInfoEx).wszUIClass as *mut WCHAR as *mut ::core::ffi::c_void,
                &raw mut (*pImeDpi).szUIClass as *mut WCHAR as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[WCHAR; 16]>() as usize,
            );
            (*pImeInfoEx).fLoadFlag = 2;
        } else {
            (*pImeInfoEx).fLoadFlag = 1;
        }

        NtUserSetImeInfoEx(pImeInfoEx);
    }

    return ret;
}

unsafe extern "C" fn Imm32LoadImeDpi(mut hKL: HKL, mut bLock: BOOL) -> PIMEDPI {
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let mut ci: CHARSETINFO = core::mem::zeroed();
    let mut pImeDpiNew: PIMEDPI = core::ptr::null_mut();
    let mut pImeDpiFound: PIMEDPI = core::ptr::null_mut();
    let mut uCodePage: UINT;
    let mut lcid: LCID;

    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(ImeInfoEx),
        ImeInfoExKeyboardLayout,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    if ImeInfoEx.fLoadFlag == 1 {
        return core::ptr::null_mut();
    }

    pImeDpiNew = ImmLocalAlloc(
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IMEDPI>() as DWORD,
    ) as PIMEDPI;
    if IS_NULL_UNEXPECTEDLY!(pImeDpiNew) != false {
        return core::ptr::null_mut();
    }

    (*pImeDpiNew).hKL = hKL;

    lcid = LOWORD!(hKL as DWORD) as LCID;
    if TranslateCharsetInfo(
        UlongToPtr!(lcid) as PDWORD,
        core::ptr::addr_of_mut!(ci),
        TCI_SRCLOCALE as DWORD,
    ) != 0
    {
        uCodePage = ci.ciACP;
    } else {
        uCodePage = CP_ACP as UINT;
    }
    (*pImeDpiNew).uCodePage = uCodePage;

    if Imm32LoadIME(core::ptr::addr_of_mut!(ImeInfoEx), pImeDpiNew) == 0 {
        ImmLocalFree!(pImeDpiNew as PVOID);
        return FALSE as PIMEDPI;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    pImeDpiFound = Imm32FindImeDpi(hKL);
    if !pImeDpiFound.is_null() {
        if bLock == 0 {
            (*pImeDpiFound).dwFlags &= !(IMEDPI_FLAG_LOCKED as DWORD);
        }

        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
        Imm32FreeIME(pImeDpiNew, (FALSE) as BOOL);
        ImmLocalFree!(pImeDpiNew as PVOID);
        return pImeDpiFound;
    } else {
        if bLock != 0 {
            (*pImeDpiNew).dwFlags |= IMEDPI_FLAG_LOCKED as DWORD;
            (*pImeDpiNew).cLockObj = 1 as DWORD;
        }

        (*pImeDpiNew).pNext = gpImeDpiList;
        gpImeDpiList = pImeDpiNew;

        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
        return pImeDpiNew;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Imm32FindOrLoadImeDpi(hKL: HKL) -> PIMEDPI {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();

    if ((IS_IME_HKL!(hKL)) == false) && (((IS_CICERO_MODE!()) == false) || IS_16BIT_MODE!()) {
        return core::ptr::null_mut();
    }

    pImeDpi = ImmLockImeDpi(hKL);
    if pImeDpi.is_null() {
        pImeDpi = Imm32LoadImeDpi(hKL, (TRUE) as BOOL);
    }
    return pImeDpi;
}

unsafe extern "C" fn ImeDpi_Escape(
    mut pImeDpi: PIMEDPI,
    mut hIMC: HIMC,
    mut uSubFunc: UINT,
    mut lpData: LPVOID,
    mut hKL: HKL,
) -> LRESULT {
    if IS_IME_HKL!(hKL) != false {
        return ((*pImeDpi).ImeEscape.unwrap_unchecked())(hIMC, uSubFunc, lpData);
    }
    if IS_CICERO_MODE!() != false && IS_16BIT_MODE!() == false {
        return ((*pImeDpi).CtfImeEscapeEx.unwrap_unchecked())(
            hIMC, uSubFunc, lpData, hKL,
        );
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32ReleaseIME(hKL: HKL) -> BOOL {
    let mut ret: BOOL = (TRUE) as i32;
    let mut pImeDpi0: PIMEDPI = core::ptr::null_mut();
    let mut pImeDpi1: PIMEDPI = core::ptr::null_mut();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    'quit: {
        pImeDpi0 = gpImeDpiList;
        while !pImeDpi0.is_null() {
            if (*pImeDpi0).hKL == hKL {
                break;
            }
            pImeDpi0 = (*pImeDpi0).pNext as PIMEDPI;
        }

        if pImeDpi0.is_null() {
            break 'quit;
        }

        if (*pImeDpi0).cLockObj != 0 {
            (*pImeDpi0).dwFlags |= IMEDPI_FLAG_UNLOADED as DWORD;
            ret = FALSE as BOOL;
            break 'quit;
        }

        if gpImeDpiList == pImeDpi0 {
            gpImeDpiList = (*pImeDpi0).pNext as PIMEDPI;
        } else if !gpImeDpiList.is_null() {
            pImeDpi1 = gpImeDpiList;
            while !pImeDpi1.is_null() {
                if (*pImeDpi1).pNext == pImeDpi0 {
                    (*pImeDpi1).pNext = (*pImeDpi0).pNext;
                    break;
                }
                pImeDpi1 = (*pImeDpi1).pNext as PIMEDPI;
            }
        }

        Imm32FreeIME(pImeDpi0, (TRUE) as BOOL);
        ImmLocalFree!((pImeDpi0).cast());
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmIsIME(mut hKL: HKL) -> BOOL {
    let mut info: IMEINFOEX = core::mem::zeroed();
    return (ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(info),
        ImeInfoExKeyboardLayoutTFS,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetDefaultIMEWnd(hWnd: HWND) -> HWND {
    if ((IS_IMM_MODE!()) == 0) {
        return core::ptr::null_mut();
    }

    if hWnd.is_null() {
        return NtUserGetThreadState(THREADSTATE_DEFAULTIMEWINDOW) as HWND;
    }

    return NtUserQueryWindow(hWnd, QUERY_WINDOW_DEFAULT_IME) as HWND;
}

#[no_mangle]
pub unsafe extern "system" fn ImmNotifyIME(
    hIMC: HIMC,
    dwAction: DWORD,
    dwIndex: DWORD,
    dwValue: DWORD_PTR,
) -> BOOL {
    let mut hKL: HKL = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/ime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmNotifyIME\0" as *const u8 as *const ::core::ffi::c_char,
            469 as ::core::ffi::c_int,
            b"(%p, %lu, %lu, %lu)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hIMC,
            dwAction,
            dwIndex,
            dwValue,
        );
    }
    if !hIMC.is_null() && IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }
    hKL = GetKeyboardLayout(0);
    pImeDpi = ImmLockImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/user/imm32/ime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmNotifyIME\0" as *const u8 as *const ::core::ffi::c_char,
            479 as ::core::ffi::c_int,
            b"NotifyIME(%p, %lu, %lu, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hIMC,
            dwAction,
            dwIndex,
            dwValue,
        );
    }
    ret = (*pImeDpi).NotifyIME.unwrap_unchecked()(
        hIMC, dwAction, dwIndex, dwValue,
    );
    if ret == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmNotifyIME\0" as *const u8 as *const ::core::ffi::c_char,
                482 as ::core::ffi::c_int,
                b"NotifyIME(%p, %lu, %lu, %p) failed\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                hIMC,
                dwAction,
                dwIndex,
                dwValue,
            );
        }
    }
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetImeInfoEx(
    mut pImeInfoEx: PIMEINFOEX,
    mut SearchType: IMEINFOEXCLASS,
    mut pvSearchKey: PVOID,
) -> BOOL {
    let mut bTextServiceDisabled: BOOL = (FALSE) as i32;

    if SearchType == ImeInfoExKeyboardLayoutTFS {
        SearchType = ImeInfoExKeyboardLayout;
        bTextServiceDisabled = CtfImmIsTextFrameServiceDisabled();
    }

    if SearchType == ImeInfoExKeyboardLayout {
        let mut hKL: HKL = *(pvSearchKey as *mut HKL);
        (*pImeInfoEx).hkl = hKL;

        if IS_IME_HKL!(hKL) == false
            && (IS_CICERO_MODE!() == false
                || IS_CICERO_COMPAT_DISABLED!() != (false)
                || bTextServiceDisabled != 0)
        {
            return (FALSE) as BOOL;
        }

        return NtUserGetImeInfoEx(pImeInfoEx, SearchType);
    }

    if SearchType == ImeInfoExImeFileName {
        StringCchCopyW(
            core::ptr::addr_of_mut!((*pImeInfoEx).wszImeFile) as *mut u16,
            (_countof!((*pImeInfoEx).wszImeFile)) as _,
            pvSearchKey as *const u16,
        );
        return NtUserGetImeInfoEx(pImeInfoEx, SearchType);
    }

    /* NOTE: ImeInfoExImeWindow is ignored */

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmLockImeDpi(hKL: HKL) -> PIMEDPI {
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut::<IMEDPI>();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    /* Find by hKL */
    pImeDpi = gpImeDpiList;
    while !pImeDpi.is_null() {
        if (*pImeDpi).hKL == hKL {
            /* found */
            /* lock if possible */
            if ((*pImeDpi).dwFlags & (IMEDPI_FLAG_UNLOADED as DWORD)) != 0 {
                pImeDpi = core::ptr::null_mut::<IMEDPI>();
            } else {
                (*pImeDpi).cLockObj = (*pImeDpi).cLockObj.wrapping_add(1);
            }
            break;
        } else {
            pImeDpi = (*pImeDpi).pNext as PIMEDPI;
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
    return pImeDpi;
}

#[no_mangle]
pub unsafe extern "system" fn ImmUnlockImeDpi(pImeDpi: PIMEDPI) {
    let mut ppEntry: *mut PIMEDPI = core::ptr::null_mut();

    if pImeDpi.is_null() {
        return;
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    /* unlock */
    (*pImeDpi).cLockObj = (*pImeDpi).cLockObj.wrapping_sub(1);
    if (*pImeDpi).cLockObj != 0 {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
        return;
    }

    if ((*pImeDpi).dwFlags & IMEDPI_FLAG_UNLOADED) == 0 {
        if ((*pImeDpi).dwFlags & IMEDPI_FLAG_LOCKED) == 0 ||
            ((*pImeDpi).ImeInfo.fdwProperty & IME_PROP_END_UNLOAD) == 0
        {
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
            return;
        }
    }

    /* Remove from list */
    ppEntry = core::ptr::addr_of_mut!(gpImeDpiList);
    while !(*ppEntry).is_null() {
        if *ppEntry == pImeDpi {
            /* found */
            *ppEntry = (*pImeDpi).pNext;
            break;
        }
        ppEntry = core::ptr::addr_of_mut!((**ppEntry).pNext) as *mut PIMEDPI;
    }

    Imm32FreeIME(pImeDpi, (TRUE) as BOOL);
    ImmLocalFree!((pImeDpi).cast());

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
}

#[no_mangle]
pub unsafe extern "system" fn ImmLoadIME(hKL: HKL) -> BOOL {
    let mut pImeDpi: PIMEDPI;

    if IS_IME_HKL!(hKL) == false && (IS_CICERO_MODE!() == false || IS_16BIT_MODE!() != false) {
        return (FALSE) as BOOL;
    }

    pImeDpi = Imm32FindImeDpi(hKL);
    if pImeDpi.is_null() {
        pImeDpi = Imm32LoadImeDpi(hKL, (FALSE) as BOOL);
    }
    return (!pImeDpi.is_null()) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmDisableIME(dwThreadId: DWORD) -> BOOL {
    NtUserDisableThreadIme(dwThreadId)
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetDescriptionA(
    mut hKL: HKL,
    mut lpszDescription: LPSTR,
    mut uBufLen: UINT,
) -> UINT {
    let mut info: IMEINFOEX = core::mem::zeroed();
    let mut cch: size_t = 0;

    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(info),
        ImeInfoExKeyboardLayout,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) == 0
    {
        return 0;
    }

    StringCchLengthW(
        core::ptr::addr_of_mut!(info.wszImeDescription) as *mut WCHAR as STRSAFE_LPCWSTR,
        (_countof!(info.wszImeDescription)) as _,
        core::ptr::addr_of_mut!(cch),
    );
    cch = WideCharToMultiByte(
        CP_ACP,
        0,
        core::ptr::addr_of_mut!(info.wszImeDescription) as *mut WCHAR as LPCWSTR,
        cch as ::core::ffi::c_int,
        lpszDescription,
        uBufLen as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as size_t;
    if uBufLen != 0 {
        *lpszDescription.add(cch) = ANSI_NULL;
    }
    return cch as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetDescriptionW(
    mut hKL: HKL,
    mut lpszDescription: LPWSTR,
    mut uBufLen: UINT,
) -> UINT {
    let mut info: IMEINFOEX = core::mem::zeroed();
    let mut cch: size_t = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/ime.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmGetDescriptionW\0" as *const u8 as *const ::core::ffi::c_char,
            677 as ::core::ffi::c_int,
            b"(%p, %p, %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hKL,
            lpszDescription,
            uBufLen,
        );
    }
    if IS_IME_HKL!(hKL) == false
        || ImmGetImeInfoEx(
            core::ptr::addr_of_mut!(info),
            ImeInfoExKeyboardLayout,
            core::ptr::addr_of_mut!(hKL) as PVOID,
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
                b"win32ss/user/imm32/ime.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetDescriptionW\0" as *const u8 as *const ::core::ffi::c_char,
                681 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }
    if uBufLen != 0 {
        StringCchCopyW(
            lpszDescription as STRSAFE_LPWSTR,
            uBufLen as size_t,
            core::ptr::addr_of_mut!(info.wszImeDescription) as *mut WCHAR as STRSAFE_LPCWSTR,
        );
    }
    StringCchLengthW(
        core::ptr::addr_of_mut!(info.wszImeDescription) as *mut WCHAR as STRSAFE_LPCWSTR,
        (_countof!(info.wszImeDescription)) as _,
        core::ptr::addr_of_mut!(cch),
    );
    return cch as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetIMEFileNameA(
    mut hKL: HKL,
    mut lpszFileName: LPSTR,
    mut uBufLen: UINT,
) -> UINT {
    let mut bDefUsed: BOOL = 0;
    let mut info: IMEINFOEX = core::mem::zeroed();
    let mut cch: size_t = 0;

    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(info),
        ImeInfoExKeyboardLayout,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) == 0
    {
        if uBufLen > 0 {
            *lpszFileName.offset(0 as isize) = ANSI_NULL;
        }
        return 0;
    }

    StringCchLengthW(
        core::ptr::addr_of_mut!(info.wszImeFile) as *mut WCHAR as STRSAFE_LPCWSTR,
        (_countof!(info.wszImeFile)) as _,
        core::ptr::addr_of_mut!(cch),
    );

    cch = WideCharToMultiByte(
        CP_ACP,
        0,
        core::ptr::addr_of_mut!(info.wszImeFile) as *mut WCHAR as LPCWSTR,
        cch as i32,
        lpszFileName,
        uBufLen as i32,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(bDefUsed),
    ) as size_t;

    if uBufLen == 0 {
        return cch as UINT;
    }

    if cch > (uBufLen - 1) as size_t {
        cch = (uBufLen - 1) as size_t;
    }

    *lpszFileName.add(cch) = ANSI_NULL;
    return cch as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetIMEFileNameW(
    mut hKL: HKL,
    mut lpszFileName: LPWSTR,
    mut uBufLen: UINT,
) -> UINT {
    let mut info: IMEINFOEX = core::mem::zeroed();
    let mut cch: size_t = 0;

    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(info),
        ImeInfoExKeyboardLayout,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) == 0
    {
        if uBufLen > 0 {
            *lpszFileName.add(0) = UNICODE_NULL;
        }
        return 0;
    }

    StringCchLengthW(
        core::ptr::addr_of_mut!(info.wszImeFile) as *mut WCHAR as STRSAFE_LPCWSTR,
        _countof!(info.wszImeFile) as size_t,
        core::ptr::addr_of_mut!(cch),
    );
    if uBufLen == 0 {
        return cch as UINT;
    }

    StringCchCopyNW(
        lpszFileName as STRSAFE_LPWSTR,
        uBufLen as size_t,
        core::ptr::addr_of_mut!(info.wszImeFile) as *mut WCHAR as STRSAFE_LPCWSTR,
        cch,
    );

    if cch > uBufLen.wrapping_sub(1) as size_t {
        cch = uBufLen.wrapping_sub(1) as size_t;
    }

    *lpszFileName.add(cch) = UNICODE_NULL;
    return cch as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetProperty(mut hKL: HKL, mut fdwIndex: DWORD) -> DWORD {
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let mut pImeInfo: LPIMEINFO = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();

    if ImmGetImeInfoEx(
        core::ptr::addr_of_mut!(ImeInfoEx),
        ImeInfoExKeyboardLayout,
        core::ptr::addr_of_mut!(hKL) as PVOID,
    ) == 0
    {
        return FALSE;
    }

    if fdwIndex == IGP_GETIMEVERSION {
        return ImeInfoEx.dwImeWinVersion;
    }

    if ImeInfoEx.fLoadFlag != 2 {
        pImeDpi = Imm32FindOrLoadImeDpi(hKL);
        if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
            return FALSE;
        }

        pImeInfo = core::ptr::addr_of_mut!((*pImeDpi).ImeInfo);
    } else {
        pImeInfo = core::ptr::addr_of_mut!(ImeInfoEx.ImeInfo);
    }

    match fdwIndex {
        IGP_PROPERTY => {
            dwValue = (*pImeInfo).fdwProperty;
        }
        IGP_CONVERSION => {
            dwValue = (*pImeInfo).fdwConversionCaps;
        }
        IGP_SENTENCE => {
            dwValue = (*pImeInfo).fdwSentenceCaps;
        }
        IGP_UI => {
            dwValue = (*pImeInfo).fdwUICaps;
        }
        IGP_SETCOMPSTR => {
            dwValue = (*pImeInfo).fdwSCSCaps;
        }
        IGP_SELECT => {
            dwValue = (*pImeInfo).fdwSelectCaps;
        }
        _ => {
            dwValue = 0;
        }
    }

    if !pImeDpi.is_null() {
        ImmUnlockImeDpi(pImeDpi);
    }
    return dwValue;
}

#[no_mangle]
pub unsafe extern "system" fn ImmEscapeA(
    hKL: HKL,
    hIMC: HIMC,
    uSubFunc: UINT,
    lpData: LPVOID,
) -> LRESULT {
    let mut ret: LRESULT;
    let mut pImeDpi: PIMEDPI;
    let mut cch: INT;
    let mut szA: [CHAR; (MAX_IMM_FILENAME as usize)] = [0; (MAX_IMM_FILENAME as usize)];
    let mut szW: [WCHAR; (MAX_IMM_FILENAME as usize)] = [0; (MAX_IMM_FILENAME as usize)];

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0;
    }

    if ImeDpi_IsUnicode!(pImeDpi) == false || lpData.is_null() {
        /* No conversion needed */
        ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    match uSubFunc {
        IME_ESC_SEQUENCE_TO_INTERNAL => {
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);

            cch = 0;
            if HIWORD!(ret) != 0 {
                szW[cch as usize] = HIWORD!(ret) as WCHAR;
                cch += 1;
            }
            if LOWORD!(ret) != 0 {
                szW[cch as usize] = LOWORD!(ret) as WCHAR;
                cch += 1;
            }

            cch = WideCharToMultiByte(
                (*pImeDpi).uCodePage,
                0,
                core::ptr::addr_of_mut!(szW) as *mut WCHAR as LPCWSTR,
                cch,
                core::ptr::addr_of_mut!(szA) as *mut CHAR as LPSTR,
                (_countof!(szA)) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as INT;
            match cch {
                1 => {
                    ret = MAKEWORD!(szA[0 as usize], 0 as CHAR) as LRESULT;
                }
                2 => {
                    ret = MAKEWORD!(szA[1 as usize], szA[0 as usize]) as LRESULT;
                }
                3 => {
                    ret = MAKELONG!(
                        MAKEWORD!(szA[2 as usize], szA[1 as usize]),
                        MAKEWORD!(szA[0 as usize], 0 as CHAR),
                    ) as LRESULT;
                }
                4 => {
                    ret = MAKELONG!(
                        MAKEWORD!(szA[3 as usize], szA[2 as usize]),
                        MAKEWORD!(szA[1 as usize], szA[0 as usize]),
                    ) as LRESULT;
                }
                _ => {
                    ret = 0;
                }
            }
        }

        IME_ESC_GET_EUDC_DICTIONARY | IME_ESC_IME_NAME | IME_ESC_GETHELPFILENAME => {
            ret = ImeDpi_Escape(
                pImeDpi,
                hIMC,
                uSubFunc,
                core::ptr::addr_of_mut!(szW) as *mut WCHAR as LPVOID,
                hKL,
            );
            if ret != 0 {
                szW[(_countof!(szW) - 1) as usize] = UNICODE_NULL as WCHAR; /* Avoid buffer overrun */
                WideCharToMultiByte(
                    (*pImeDpi).uCodePage,
                    0,
                    core::ptr::addr_of_mut!(szW) as *mut WCHAR as LPCWSTR,
                    -1,
                    lpData as LPSTR,
                    (MAX_IMM_FILENAME as i32),
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                *(lpData as LPSTR).offset((MAX_IMM_FILENAME - 1) as isize) =
                    ANSI_NULL as CHAR; // Avoid buffer overrun
            }
        }

        IME_ESC_SET_EUDC_DICTIONARY | IME_ESC_HANJA_MODE => {
            MultiByteToWideChar(
                (*pImeDpi).uCodePage,
                MB_PRECOMPOSED,
                lpData as LPCSTR,
                -1,
                core::ptr::addr_of_mut!(szW) as *mut WCHAR as LPWSTR,
                (_countof!(szW)) as i32,
            );
            szW[(_countof!(szW) - 1) as usize] = UNICODE_NULL as WCHAR; /* Avoid buffer overrun */
            ret = ImeDpi_Escape(
                pImeDpi,
                hIMC,
                uSubFunc,
                core::ptr::addr_of_mut!(szW) as *mut WCHAR as LPVOID,
                hKL,
            );
        }

        _ => {
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);
        }
    }

    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmEscapeW(
    hKL: HKL,
    hIMC: HIMC,
    uSubFunc: UINT,
    lpData: LPVOID,
) -> LRESULT {
    let mut ret: LRESULT;
    let mut pImeDpi: PIMEDPI;
    let mut cch: INT;
    let mut szA: [CHAR; (MAX_IMM_FILENAME as usize)] = [0; (MAX_IMM_FILENAME as usize)];
    let mut szW: [WCHAR; (MAX_IMM_FILENAME as usize)] = [0; (MAX_IMM_FILENAME as usize)];
    let mut word: WORD;

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return 0;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false || lpData.is_null() {
        /* No conversion needed */
        ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    match uSubFunc {
        IME_ESC_SEQUENCE_TO_INTERNAL => {
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);

            word = LOWORD!(ret);
            cch = 0;
            if HIBYTE!(word) != 0 {
                szA[cch as usize] = HIBYTE!(word) as CHAR;
                cch += 1;
            }
            if LOBYTE!(word) != 0 {
                szA[cch as usize] = LOBYTE!(word) as CHAR;
                cch += 1;
            }

            cch = MultiByteToWideChar(
                (*pImeDpi).uCodePage,
                MB_PRECOMPOSED,
                addr_of_mut!(szA) as LPCSTR,
                cch,
                addr_of_mut!(szW) as LPWSTR,
                (_countof!(szW)) as i32,
            );
            match cch {
                1 => {
                    ret = szW[0 as usize] as LRESULT;
                }
                2 => {
                    ret = MAKELONG!(
                        szW[1 as usize],
                        szW[0 as usize],
                    ) as LRESULT;
                }
                _ => {
                    ret = 0;
                }
            }
        }

        IME_ESC_GET_EUDC_DICTIONARY | IME_ESC_IME_NAME | IME_ESC_GETHELPFILENAME => {
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, addr_of_mut!(szA) as LPVOID, hKL);
            if ret != 0 {
                // Avoid buffer overrun
                szA[(_countof!(szA) - 1) as usize] = ANSI_NULL;
                MultiByteToWideChar(
                    (*pImeDpi).uCodePage,
                    MB_PRECOMPOSED,
                    addr_of_mut!(szA) as LPCSTR,
                    -1,
                    (lpData).cast(),
                    (MAX_IMM_FILENAME as i32),
                );
                /* Avoid buffer overrun */
                *(lpData as LPWSTR)
                    .offset((MAX_IMM_FILENAME - 1) as isize) =
                    UNICODE_NULL;
            }
        }

        IME_ESC_SET_EUDC_DICTIONARY | IME_ESC_HANJA_MODE => {
            WideCharToMultiByte(
                (*pImeDpi).uCodePage,
                0,
                lpData as LPCWSTR,
                -1,
                addr_of_mut!(szA) as LPSTR,
                (_countof!(szA)) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            // Avoid buffer overrun
            szA[(_countof!(szA) - 1) as usize] = ANSI_NULL;
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, addr_of_mut!(szA) as LPVOID, hKL);
        }

        _ => {
            ret = ImeDpi_Escape(pImeDpi, hIMC, uSubFunc, lpData, hKL);
        }
    }

    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetOpenStatus(mut hIMC: HIMC) -> BOOL {
    let mut ret: BOOL;
    let mut pIC: LPINPUTCONTEXT;

    if IS_NULL_UNEXPECTEDLY!(hIMC) {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        return (FALSE) as BOOL;
    }

    ret = (*pIC).fOpen;
    ImmUnlockIMC(hIMC);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetOpenStatus(hIMC: HIMC, fOpen: BOOL) -> BOOL {
    let mut dwConversion: DWORD = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut bHasChange: BOOL = (FALSE) as i32;

    if IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if (*pIC).fOpen != fOpen {
        (*pIC).fOpen = fOpen;
        hWnd = (*pIC).hWnd;
        dwConversion = (*pIC).fdwConversion;
        bHasChange = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);

    if bHasChange != 0 {
        Imm32MakeIMENotify(
            hIMC,
            hWnd,
            NI_CONTEXTUPDATED,
            0,
            (IMC_SETOPENSTATUS) as DWORD_PTR,
            IMN_SETOPENSTATUS,
            0,
        );
        NtUserNotifyIMEStatus(hWnd, fOpen, dwConversion);
    } else {
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetStatusWindowPos(hIMC: HIMC, lpptPos: LPPOINT) -> BOOL {
    let mut pIC: LPINPUTCONTEXT;
    let mut ret: BOOL;

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    ret = (((*pIC).fdwInit & (INIT_STATUSWNDPOS as DWORD)) != 0) as BOOL;
    if ret != 0 {
        *lpptPos = (*pIC).ptStatusWndPos;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetStatusWindowPos(hIMC: HIMC, lpptPos: LPPOINT) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut hWnd: HWND = core::ptr::null_mut();

    if IS_CROSS_THREAD_HIMC!(hIMC) {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        return (FALSE) as BOOL;
    }

    hWnd = (*pIC).hWnd;
    (*pIC).ptStatusWndPos = *lpptPos;
    (*pIC).fdwInit |= INIT_STATUSWNDPOS;

    ImmUnlockIMC(hIMC);

    Imm32MakeIMENotify(
        hIMC,
        hWnd,
        NI_CONTEXTUPDATED,
        0,
        (IMC_SETSTATUSWINDOWPOS) as DWORD_PTR,
        IMN_SETSTATUSWINDOWPOS,
        0,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCompositionWindow(
    hIMC: HIMC,
    lpCompForm: LPCOMPOSITIONFORM,
) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if (*pIC).fdwInit & INIT_COMPFORM != 0 {
        *lpCompForm = (*pIC).cfCompForm;
        ret = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCompositionWindow(
    hIMC: HIMC,
    lpCompForm: LPCOMPOSITIONFORM,
) -> BOOL {
    let mut pIC: LPINPUTCONTEXTDX;
    let mut hWnd: HWND;

    if Imm32IsCrossThreadAccess(hIMC) != 0 {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        return (FALSE) as BOOL;
    }

    (*pIC).__bindgen_anon_1.cfCompForm = *lpCompForm;
    (*pIC).__bindgen_anon_1.fdwInit |= INIT_COMPFORM as DWORD;

    if (*pIC).dwUIFlags & (_IME_UI_NO_COMPFORM as DWORD) != 0 {
        (*pIC).dwUIFlags &= !(_IME_UI_NO_COMPFORM as DWORD);
    } else {
        (*pIC).dwUIFlags &= !(_IME_UI_HIDDEN as DWORD);
    }

    hWnd = (*pIC).__bindgen_anon_1.hWnd;

    ImmUnlockIMC(hIMC);

    Imm32MakeIMENotify(
        hIMC,
        hWnd,
        NI_CONTEXTUPDATED,
        0,
        (IMC_SETCOMPOSITIONWINDOW) as DWORD_PTR,
        IMN_SETCOMPOSITIONWINDOW,
        0,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCompositionFontA(hIMC: HIMC, lplf: LPLOGFONTA) -> BOOL {
    let mut pClientImc: PCLIENTIMC;
    let mut ret: BOOL = (FALSE) as i32;
    let mut bWide: BOOL;
    let mut pIC: LPINPUTCONTEXT;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    bWide = ((*pClientImc).dwFlags & CLIENTIMC_WIDE) as BOOL;
    ImmUnlockClientImc(pClientImc);

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if ((*pIC).fdwInit & INIT_LOGFONT) != 0 {
        if bWide != 0 {
            LogFontWideToAnsi(core::ptr::addr_of_mut!((*pIC).lfFont.W), lplf);
        } else {
            *lplf = (*pIC).lfFont.A;
        }

        ret = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetCompositionFontW(mut hIMC: HIMC, mut lplf: LPLOGFONTW) -> BOOL {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut bWide: BOOL = 0;
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    bWide = ((*pClientImc).dwFlags & CLIENTIMC_WIDE) as BOOL;
    ImmUnlockClientImc(pClientImc);

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if ((*pIC).fdwInit & INIT_LOGFONT) != 0 {
        if bWide != 0 {
            *lplf = (*pIC).lfFont.W;
        } else {
            LogFontAnsiToWide(core::ptr::addr_of_mut!((*pIC).lfFont.A), lplf);
        }

        ret = (TRUE) as i32;
    }

    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCompositionFontA(hIMC: HIMC, lplf: LPLOGFONTA) -> BOOL {
    let mut lfW: LOGFONTW = core::mem::zeroed();
    let mut pClientImc: PCLIENTIMC;
    let mut bWide: BOOL;
    let mut pIC: LPINPUTCONTEXTDX;
    let mut LangID: LANGID;
    let mut hWnd: HWND;

    if IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    bWide = ((*pClientImc).dwFlags & CLIENTIMC_WIDE) as BOOL;
    ImmUnlockClientImc(pClientImc);

    if bWide != 0 {
        LogFontAnsiToWide(lplf, core::ptr::addr_of_mut!(lfW));
        return ImmSetCompositionFontW(hIMC, core::ptr::addr_of_mut!(lfW));
    }

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if (*GetWin32ClientInfo!()).dwExpWinVer < _WIN32_WINNT_NT4 {
        /* old version (3.x)? */
        LangID = LANGIDFROMLCID!(GetSystemDefaultLCID());
        if (PRIMARYLANGID!(LangID)) as u32 == LANG_JAPANESE
            && ((*pIC).dwUIFlags & _IME_UI_HIDDEN) == 0
            && (*pIC).__bindgen_anon_1.cfCompForm.dwStyle != CFS_DEFAULT
        {
            PostMessageA(
                (*pIC).__bindgen_anon_1.hWnd,
                WM_IME_REPORT,
                (IR_CHANGECONVERT) as WPARAM,
                0,
            );
        }
    }

    (*pIC).__bindgen_anon_1.lfFont.A = *lplf;
    (*pIC).__bindgen_anon_1.fdwInit |= INIT_LOGFONT;
    hWnd = (*pIC).__bindgen_anon_1.hWnd;

    ImmUnlockIMC(hIMC);

    Imm32MakeIMENotify(
        hIMC,
        hWnd,
        NI_CONTEXTUPDATED,
        0,
        (IMC_SETCOMPOSITIONFONT) as DWORD_PTR,
        IMN_SETCOMPOSITIONFONT,
        0,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetCompositionFontW(hIMC: HIMC, lplf: LPLOGFONTW) -> BOOL {
    let mut lfA: LOGFONTA = core::mem::zeroed();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut bWide: BOOL = 0;
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXTDX = core::ptr::null_mut();
    let mut LangID: LANGID = 0;

    if IS_CROSS_THREAD_HIMC!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    bWide = ((*pClientImc).dwFlags & CLIENTIMC_WIDE as DWORD) as BOOL;
    ImmUnlockClientImc(pClientImc);

    if bWide == 0 {
        LogFontWideToAnsi(lplf, core::ptr::addr_of_mut!(lfA));
        return ImmSetCompositionFontA(hIMC, core::ptr::addr_of_mut!(lfA));
    }

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXTDX;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if (*GetWin32ClientInfo!()).dwExpWinVer < _WIN32_WINNT_NT4 as DWORD {
        /* old version (3.x)? */
        LangID = LANGIDFROMLCID!(GetSystemDefaultLCID());
        if PRIMARYLANGID!(LangID) == LANG_JAPANESE as WORD
            && ((*pIC).dwUIFlags & _IME_UI_HIDDEN as DWORD) == 0
            && (*pIC).__bindgen_anon_1.cfCompForm.dwStyle != CFS_DEFAULT as DWORD
        {
            PostMessageW(
                (*pIC).__bindgen_anon_1.hWnd,
                WM_IME_REPORT as UINT,
                IR_CHANGECONVERT as WPARAM,
                0 as LPARAM,
            );
        }
    }

    (*pIC).__bindgen_anon_1.lfFont.W = *lplf;
    (*pIC).__bindgen_anon_1.fdwInit |= INIT_LOGFONT as DWORD;
    hWnd = (*pIC).__bindgen_anon_1.hWnd;

    ImmUnlockIMC(hIMC);

    Imm32MakeIMENotify(
        hIMC,
        hWnd,
        NI_CONTEXTUPDATED as DWORD,
        0 as DWORD,
        IMC_SETCOMPOSITIONFONT as DWORD_PTR,
        IMN_SETCOMPOSITIONFONT as DWORD,
        0 as DWORD_PTR,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetConversionListA(
    hKL: HKL,
    hIMC: HIMC,
    pSrc: LPCSTR,
    lpDst: LPCANDIDATELIST,
    dwBufLen: DWORD,
    uFlag: UINT,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut cb: UINT = 0;
    let mut pszSrcW: LPWSTR = core::ptr::null_mut();
    let mut pCL: LPCANDIDATELIST = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI;

    'quit: {
        pImeDpi = Imm32FindOrLoadImeDpi(hKL);
        if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
            return 0;
        }

        if ImeDpi_IsUnicode!(pImeDpi) == false {
            /* No conversion needed */
            ret = (*pImeDpi).ImeConversionList.unwrap_unchecked()(
                hIMC, pSrc as LPCVOID, lpDst, dwBufLen, uFlag,
            );
            ImmUnlockImeDpi(pImeDpi);
            return ret;
        }

        if !pSrc.is_null() {
            pszSrcW = Imm32WideFromAnsi((*pImeDpi).uCodePage, pSrc);
            if IS_NULL_UNEXPECTEDLY!(pszSrcW) != false {
                break 'quit;
            }
        }

        cb = (*pImeDpi).ImeConversionList.unwrap_unchecked()(
            hIMC,
            pszSrcW as LPCVOID,
            core::ptr::null_mut(),
            0,
            uFlag,
        ) as UINT;
        if IS_ZERO_UNEXPECTEDLY!(cb) != false {
            break 'quit;
        }

        pCL = ImmLocalAlloc(0, cb) as LPCANDIDATELIST;
        if IS_NULL_UNEXPECTEDLY!(pCL) != false {
            break 'quit;
        }

        cb = (*pImeDpi).ImeConversionList.unwrap_unchecked()(
            hIMC,
            pszSrcW as LPCVOID,
            pCL,
            cb as DWORD,
            uFlag,
        ) as UINT;
        if IS_ZERO_UNEXPECTEDLY!(cb) != false {
            break 'quit;
        }

        ret = CandidateListWideToAnsi(pCL, lpDst, dwBufLen, (*pImeDpi).uCodePage);
    }

    ImmLocalFree!((pszSrcW).cast());
    ImmLocalFree!((pCL).cast());
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetConversionListW(
    hKL: HKL,
    hIMC: HIMC,
    pSrc: LPCWSTR,
    lpDst: LPCANDIDATELIST,
    dwBufLen: DWORD,
    uFlag: UINT,
) -> DWORD {
    let mut ret: DWORD = 0;
    let mut cb: INT;
    let mut pImeDpi: PIMEDPI;
    let mut pCL: LPCANDIDATELIST = core::ptr::null_mut();
    let mut pszSrcA: LPSTR = core::ptr::null_mut();

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi as *const core::ffi::c_void) != false {
        return 0;
    }

    if ImeDpi_IsUnicode!(pImeDpi) != false {
        /* No conversion needed */
        ret = ((*pImeDpi).ImeConversionList.unwrap_unchecked())(
            hIMC, pSrc as LPCVOID, lpDst, dwBufLen, uFlag,
        );
        ImmUnlockImeDpi(pImeDpi);
        return ret;
    }

    'Quit: {
        if !pSrc.is_null() {
            pszSrcA = Imm32AnsiFromWide((*pImeDpi).uCodePage, pSrc);
            if IS_NULL_UNEXPECTEDLY!(pszSrcA as *const core::ffi::c_void) != false {
                break 'Quit;
            }
        }

        cb = ((*pImeDpi).ImeConversionList.unwrap_unchecked())(
            hIMC,
            pszSrcA as LPCVOID,
            core::ptr::null_mut(),
            0,
            uFlag,
        ) as INT;
        if IS_ZERO_UNEXPECTEDLY!(cb) != false {
            break 'Quit;
        }

        pCL = ImmLocalAlloc(0, cb as DWORD) as LPCANDIDATELIST;
        if IS_NULL_UNEXPECTEDLY!(pCL as *const core::ffi::c_void) != false {
            break 'Quit;
        }

        cb = ((*pImeDpi).ImeConversionList.unwrap_unchecked())(
            hIMC,
            pszSrcA as LPCVOID,
            pCL,
            cb as DWORD,
            uFlag,
        ) as INT;
        if IS_ZERO_UNEXPECTEDLY!(cb) != false {
            break 'Quit;
        }

        ret = CandidateListAnsiToWide(pCL, lpDst, dwBufLen, (*pImeDpi).uCodePage);
    }

    // Quit:
    ImmLocalFree!(pszSrcA as LPVOID);
    ImmLocalFree!(pCL as LPVOID);
    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetConversionStatus(
    hIMC: HIMC,
    lpfdwConversion: LPDWORD,
    lpfdwSentence: LPDWORD,
) -> BOOL {
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    if !lpfdwConversion.is_null() {
        *lpfdwConversion = (*pIC).fdwConversion;
    }

    if !lpfdwSentence.is_null() {
        *lpfdwSentence = (*pIC).fdwSentence;
    }

    ImmUnlockIMC(hIMC);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmSetConversionStatus(
    hIMC: HIMC,
    fdwConversion: DWORD,
    fdwSentence: DWORD,
) -> BOOL {
    let mut hKL: HKL;
    let mut pIC: LPINPUTCONTEXT;
    let mut dwOldConversion: DWORD = fdwConversion;
    let mut dwOldSentence: DWORD = fdwSentence;
    let mut fOpen: BOOL = (FALSE) as i32;
    let mut fConversionChange: BOOL = (FALSE) as i32;
    let mut fSentenceChange: BOOL = (FALSE) as i32;
    let mut fUseCicero: BOOL = (FALSE) as i32;
    let mut hWnd: HWND;

    hKL = GetKeyboardLayout(0);
    if ((IS_IME_HKL!(hKL)) == false) && IS_CICERO_MODE!() && ((IS_16BIT_MODE!()) == false) && ((IS_CICERO_COMPAT_DISABLED!()) == false) {
        fUseCicero = (TRUE) as i32;
    }

    if IS_CROSS_THREAD_HIMC!(hIMC) {
        return (FALSE) as BOOL;
    }

    pIC = ImmLockIMC(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pIC) {
        return (FALSE) as BOOL;
    }

    if (*pIC).fdwConversion != fdwConversion {
        dwOldConversion = (*pIC).fdwConversion;
        (*pIC).fdwConversion = fdwConversion;
        fConversionChange = (TRUE) as i32;
    }

    if (*pIC).fdwSentence != fdwSentence {
        dwOldSentence = (*pIC).fdwSentence;
        (*pIC).fdwSentence = fdwSentence;
        fSentenceChange = (TRUE) as i32;
    }

    hWnd = (*pIC).hWnd;
    fOpen = (*pIC).fOpen;
    ImmUnlockIMC(hIMC);

    if fConversionChange != 0 || fUseCicero != 0 {
        Imm32MakeIMENotify(
            hIMC,
            hWnd,
            NI_CONTEXTUPDATED,
            dwOldConversion,
            (IMC_SETCONVERSIONMODE) as DWORD_PTR,
            IMN_SETCONVERSIONMODE,
            0,
        );
        if fConversionChange != 0 {
            NtUserNotifyIMEStatus(hWnd, fOpen, fdwConversion);
        }
    }

    if fSentenceChange != 0 || fUseCicero != 0 {
        Imm32MakeIMENotify(
            hIMC,
            hWnd,
            NI_CONTEXTUPDATED,
            dwOldSentence,
            (IMC_SETSENTENCEMODE) as DWORD_PTR,
            IMN_SETSENTENCEMODE,
            0,
        );
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmConfigureIMEA(
    hKL: HKL,
    hWnd: HWND,
    dwMode: DWORD,
    mut lpData: LPVOID,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut RegWordW: REGISTERWORDW = core::mem::zeroed();
    let mut pRegWordA: LPREGISTERWORDA = core::ptr::null_mut();

    if IS_NULL_UNEXPECTEDLY!(ValidateHwnd!((hWnd).cast())) != false || IS_CROSS_PROCESS_HWND!(hWnd) != false {
        return FALSE as BOOL;
    }

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return FALSE as BOOL;
    }

    RtlZeroMemory!(
        core::ptr::addr_of_mut!(RegWordW) as *mut core::ffi::c_void,
        core::mem::size_of::<REGISTERWORDW>(),
    );

    'quit: {
        'doit: {
            if ImeDpi_IsUnicode!(pImeDpi) == false
                || lpData.is_null()
                || dwMode != IME_CONFIG_REGISTERWORD as DWORD
            {
                break 'doit;
            }

            pRegWordA = lpData as LPREGISTERWORDA;

            if !(*pRegWordA).lpReading.is_null() {
                RegWordW.lpReading =
                    Imm32WideFromAnsi((*pImeDpi).uCodePage, (*pRegWordA).lpReading);
                if IS_NULL_UNEXPECTEDLY!(RegWordW.lpReading) != false {
                    break 'quit;
                }
            }

            if !(*pRegWordA).lpWord.is_null() {
                RegWordW.lpWord =
                    Imm32WideFromAnsi((*pImeDpi).uCodePage, (*pRegWordA).lpWord);
                if IS_NULL_UNEXPECTEDLY!(RegWordW.lpWord) != false {
                    break 'quit;
                }
            }

            lpData = core::ptr::addr_of_mut!(RegWordW) as LPVOID;
        }

        // DoIt:
        SendMessageW(
            hWnd,
            WM_IME_SYSTEM as UINT,
            0x1B as WPARAM,
            0 as LPARAM,
        );
        ret = (*pImeDpi).ImeConfigure.unwrap_unchecked()(
            hKL, hWnd, dwMode, lpData,
        );
        SendMessageW(
            hWnd,
            WM_IME_SYSTEM as UINT,
            0x1A as WPARAM,
            0 as LPARAM,
        );
    }

    // Quit:
    ImmLocalFree!(RegWordW.lpReading as LPVOID);
    ImmLocalFree!(RegWordW.lpWord as LPVOID);
    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmConfigureIMEW(
    hKL: HKL,
    hWnd: HWND,
    dwMode: DWORD,
    mut lpData: LPVOID,
) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut RegWordA: REGISTERWORDA = core::mem::zeroed();
    let mut pRegWordW: LPREGISTERWORDW = core::ptr::null_mut();

    if IS_NULL_UNEXPECTEDLY!(ValidateHwnd!((hWnd).cast())) != false || IS_CROSS_PROCESS_HWND!(hWnd) != false {
        return (FALSE) as BOOL;
    }

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return (FALSE) as BOOL;
    }

    core::ptr::write_bytes(core::ptr::addr_of_mut!(RegWordA), 0u8, 1);

    'quit: {
        'doit: {
            if ImeDpi_IsUnicode!(pImeDpi) != false
                || lpData.is_null()
                || dwMode != IME_CONFIG_REGISTERWORD
            {
                break 'doit; /* No conversion needed */
            }

            pRegWordW = lpData as LPREGISTERWORDW;

            if !(*pRegWordW).lpReading.is_null() {
                RegWordA.lpReading =
                    Imm32AnsiFromWide((*pImeDpi).uCodePage, (*pRegWordW).lpReading as LPCWSTR);
                if IS_NULL_UNEXPECTEDLY!(RegWordA.lpReading) != false {
                    break 'quit;
                }
            }

            if !(*pRegWordW).lpWord.is_null() {
                RegWordA.lpWord =
                    Imm32AnsiFromWide((*pImeDpi).uCodePage, (*pRegWordW).lpWord as LPCWSTR);
                if IS_NULL_UNEXPECTEDLY!(RegWordA.lpWord) != false {
                    break 'quit;
                }
            }

            lpData = core::ptr::addr_of_mut!(RegWordA) as LPVOID;
        }

        // DoIt:
        SendMessageW(hWnd, WM_IME_SYSTEM, 0x1B, 0);
        ret = (*pImeDpi).ImeConfigure.unwrap_unchecked()(hKL, hWnd, dwMode, lpData);
        SendMessageW(hWnd, WM_IME_SYSTEM, 0x1A, 0);
    }

    // Quit:
    ImmLocalFree!(RegWordA.lpReading as LPVOID);
    ImmLocalFree!(RegWordA.lpWord as LPVOID);
    ImmUnlockImeDpi(pImeDpi);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmWINNLSEnableIME(mut hWnd: HWND, mut enable: BOOL) -> BOOL {
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut hImeWnd: HWND = core::ptr::null_mut();
    let mut bImeWnd: BOOL = 0;
    let mut ret: BOOL = 0;

    if Imm32IsSystemJapaneseOrKorean() == 0 {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return (FALSE) as BOOL;
    }

    hIMC = NtUserGetThreadState(THREADSTATE_DEFAULTINPUTCONTEXT) as HIMC;
    if IS_NULL_UNEXPECTEDLY!(hIMC) != false {
        return (FALSE) as BOOL;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    ret = ((((*pClientImc).dwFlags & CLIENTIMC_DISABLEIME) == 0) as BOOL);
    if ((enable != 0) as BOOL) == ret {
        ImmUnlockClientImc(pClientImc);
        return ret;
    }

    if IsWindow(hWnd) == 0 {
        hWnd = GetFocus();
    }

    hImeWnd = ImmGetDefaultIMEWnd(hWnd);
    bImeWnd = IsWindow(hImeWnd);
    if bImeWnd != 0 {
        ImmSetActiveContext(
            hWnd,
            if enable != 0 { core::ptr::null_mut() } else { hIMC },
            (FALSE) as BOOL,
        );
    }

    if enable != 0 {
        (*pClientImc).dwFlags &= !CLIENTIMC_DISABLEIME;
    } else {
        (*pClientImc).dwFlags |= CLIENTIMC_DISABLEIME;
    }

    ImmUnlockClientImc(pClientImc);

    if bImeWnd != 0 {
        ImmSetActiveContext(
            hWnd,
            if enable != 0 { hIMC } else { core::ptr::null_mut() },
            (TRUE) as BOOL,
        );
    }

    return ret;
}
