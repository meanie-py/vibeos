// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0B087FAD2DEC: [u16; 29] = [
    0x0044u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0020, 0x0054, 0x0068,
    0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0049, 0x006E, 0x0070, 0x0075, 0x0074,
    0x0020, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_13EF54168D9A: [u16; 36] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0054, 0x0046, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0053,
    0x0068, 0x0061, 0x0072, 0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_5BED95ADB00B: [u16; 11] = [
    0x006Du16, 0x0073, 0x006F, 0x006F, 0x0062, 0x0065, 0x002E, 0x0065, 0x0078, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_60A8639771FC: [u16; 5] = [0x0043u16, 0x0055, 0x0041, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_B5364BD7CA8B: [u16; 23] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0054, 0x0046, 0x0000,
];
pub static RUSTCONV_WSTR_EC63E0C750FF: [u16; 12] = [
    0x0061u16, 0x0070, 0x0070, 0x0068, 0x0065, 0x006C, 0x0070, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_hOle32: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_hMsctf: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_aimm_compat_flags: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut g_disable_CUAS_flag: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut g_hCtfIme: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_dwTLSIndex: DWORD = -(1 as ::core::ffi::c_int) as DWORD;

// WINE_DEFAULT_DEBUG_CHANNEL(imm); -- debug channel macro, no Rust declaration emitted

pub type FN_CoInitializeEx = Option<unsafe extern "system" fn(LPVOID, DWORD) -> HRESULT>;
pub type FN_CoUninitialize = Option<unsafe extern "system" fn()>;
pub type FN_CoRegisterInitializeSpy =
    Option<unsafe extern "system" fn(*mut IInitializeSpy, *mut ULARGE_INTEGER) -> HRESULT>;
pub type FN_CoRevokeInitializeSpy = Option<unsafe extern "system" fn(ULARGE_INTEGER) -> HRESULT>;

pub static mut g_pfnOLE32_CoInitializeEx: FN_CoInitializeEx = None;
pub static mut g_pfnOLE32_CoUninitialize: FN_CoUninitialize = None;
pub static mut g_pfnOLE32_CoRegisterInitializeSpy: FN_CoRegisterInitializeSpy = None;
pub static mut g_pfnOLE32_CoRevokeInitializeSpy: FN_CoRevokeInitializeSpy = None;

pub type FN_TF_CreateLangBarMgr =
    Option<unsafe extern "system" fn(*mut *mut ITfLangBarMgr) -> HRESULT>;
pub type FN_TF_InvalidAssemblyListCacheIfExist = Option<unsafe extern "system" fn()>;

pub static mut g_pfnMSCTF_TF_CreateLangBarMgr: FN_TF_CreateLangBarMgr = None;
pub static mut g_pfnMSCTF_TF_InvalidAssemblyListCacheIfExist:
    FN_TF_InvalidAssemblyListCacheIfExist = None;

pub type FN_ApphelpCheckIME = Option<unsafe extern "system" fn(LPCWSTR) -> BOOL>;

// g_dwTLSIndex = -1 (DWORD)

#[repr(C)]
pub struct IMMTLSDATA {
    pub pSpy: *mut IInitializeSpy,   /* CoInitialize Spy */
    pub dwUnknown1: DWORD,
    pub uliCookie: ULARGE_INTEGER,   /* Spy requires a cookie for revoking */
    pub bDoCount: BOOL,              /* Is it counting? */
    pub dwSkipCount: DWORD,          /* The skipped count */
    pub bUninitializing: BOOL,       /* Is it uninitializing? */
    pub dwUnknown2: DWORD,
}
pub type PIMMTLSDATA = *mut IMMTLSDATA;

#[repr(C)]
pub struct ISPY {
    pub m_pSpyVtbl: *const IInitializeSpyVtbl,
    pub m_cRefs: LONG,
}
pub type PISPY = *mut ISPY;

// ctf.c's `static const IInitializeSpyVtbl g_vtblISPY`, slot for slot.
static g_vtblISPY: IInitializeSpyVtbl = IInitializeSpyVtbl {
    QueryInterface: Some(ISPY_QueryInterface),
    AddRef: Some(ISPY_AddRef),
    Release: Some(ISPY_Release),
    PreInitialize: Some(ISPY_PreInitialize),
    PostInitialize: Some(ISPY_PostInitialize),
    PreUninitialize: Some(ISPY_PreUninitialize),
    PostUninitialize: Some(ISPY_PostUninitialize),
};

// Imm32DeleteIMMISPY(pSpy) -- macro, no declaration emitted

#[repr(C)]
pub struct IMM_DELAY_SET_LANG_BAND {
    pub hWnd: HWND,
    pub fSet: BOOL,
}
pub type PIMM_DELAY_SET_LANG_BAND = *mut IMM_DELAY_SET_LANG_BAND;
unsafe extern "C" fn Imm32InsideLoaderLock() -> BOOL {
    return ((*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).LoaderLock).OwningThread
        == (*NtCurrentTeb()).ClientId.UniqueThread) as BOOL;
}

unsafe extern "C" fn Imm32IsInteractiveUserLogon() -> BOOL {
    let mut bOK: BOOL = 0;
    let mut IsMember: BOOL = FALSE as BOOL;
    let mut pSid: PSID = ::core::ptr::null_mut::<::core::ffi::c_void>();
    // SECURITY_NT_AUTHORITY is the aggregate initializer `{0,0,0,0,0,5}`, not a scalar.
    let mut IdentAuth: SID_IDENTIFIER_AUTHORITY = _SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    if AllocateAndInitializeSid(
        &raw mut IdentAuth,
        1 as BYTE,
        SECURITY_INTERACTIVE_RID as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        &raw mut pSid,
    ) == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/ctf.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32IsInteractiveUserLogon\0" as *const u8
                    as *const ::core::ffi::c_char,
                32 as ::core::ffi::c_int,
                b"Error: %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
                GetLastError(),
            );
        }
        return FALSE as BOOL;
    }
    bOK = CheckTokenMembership(core::ptr::null_mut(), pSid, &raw mut IsMember);
    if !pSid.is_null() {
        FreeSid(pSid);
    }
    return (bOK != 0 && IsMember != 0) as BOOL;
}

unsafe extern "C" fn Imm32IsRunningInMsoobe() -> BOOL {
    let mut pchFilePart: LPWSTR = core::ptr::null_mut();
    let mut Buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if GetModuleFileNameW(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FileName) as LPWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    GetFullPathNameW(
        core::ptr::addr_of_mut!(FileName) as *mut WCHAR as LPCWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
        core::ptr::addr_of_mut!(Buffer) as LPWSTR,
        core::ptr::addr_of_mut!(pchFilePart),
    );
    if pchFilePart.is_null() {
        return (FALSE) as BOOL;
    }

    return (lstrcmpiW(pchFilePart as LPCWSTR, RUSTCONV_WSTR_5BED95ADB00B.as_ptr()) == 0) as BOOL;
}

unsafe extern "C" fn Imm32IsCUASEnabledInRegistry() -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LSTATUS;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut dwData: DWORD;
    let mut cbData: DWORD;

    error = RegOpenKeyW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_13EF54168D9A.as_ptr(), &mut hKey);
    if (error) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    dwData = 0;
    cbData = core::mem::size_of::<DWORD>() as DWORD;
    error = RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_60A8639771FC.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::addr_of_mut!(dwData) as LPBYTE,
        &mut cbData,
    );
    RegCloseKey(hKey);

    if (error) as u32 != ERROR_SUCCESS || dwType != REG_DWORD {
        return (FALSE) as BOOL;
    }

    (dwData != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn Imm32GetFn(
    mut ppfn: *mut FARPROC,
    mut phinstDLL: *mut HINSTANCE,
    mut pszDllName: LPCWSTR,
    mut pszFuncName: LPCSTR,
) -> BOOL {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if (*ppfn).is_some() {
        return (TRUE) as BOOL;
    }

    if (*phinstDLL).is_null() {
        Imm32GetSystemLibraryPath(
            core::ptr::addr_of_mut!(szPath) as LPWSTR,
            _countof!(szPath) as DWORD,
            pszDllName,
        );
        *phinstDLL = LoadLibraryExW(
            core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
            core::ptr::null_mut(),
            0,
        );
        if (*phinstDLL).is_null() {
            return (FALSE) as BOOL;
        }
    }

    *ppfn = GetProcAddress(*phinstDLL, pszFuncName);
    return (*ppfn).is_some() as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CoInitializeEx() -> HRESULT {
    let dll_name: [u16; 10] = [
        'o' as u16, 'l' as u16, 'e' as u16, '3' as u16, '2' as u16,
        '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
    ];
    if Imm32GetFn(
        &raw mut g_pfnOLE32_CoInitializeEx as *mut FARPROC,
        &raw mut g_hOle32,
        dll_name.as_ptr() as LPCWSTR,
        b"CoInitializeEx\0".as_ptr() as LPCSTR,
    ) == 0
    {
        return E_FAIL;
    }

    return g_pfnOLE32_CoInitializeEx.unwrap_unchecked()(
        NULL,
        COINIT_APARTMENTTHREADED as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CoUninitialize() {
    if Imm32GetFn(
        core::ptr::addr_of_mut!(g_pfnOLE32_CoUninitialize) as *mut FARPROC,
        core::ptr::addr_of_mut!(g_hOle32),
        ['o' as u16, 'l' as u16, 'e' as u16, '3' as u16, '2' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr() as LPCWSTR,
        b"CoUninitialize\0".as_ptr() as LPCSTR,
    ) == 0
    {
        return;
    }
    g_pfnOLE32_CoUninitialize.unwrap_unchecked()();
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CoRegisterInitializeSpy(spy: *mut IInitializeSpy, cookie: *mut ULARGE_INTEGER) -> HRESULT {
    if Imm32GetFn(
        &raw mut g_pfnOLE32_CoRegisterInitializeSpy as *mut FARPROC,
        &raw mut g_hOle32,
        ['o' as u16, 'l' as u16, 'e' as u16, '3' as u16, '2' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr() as LPCWSTR,
        b"CoRegisterInitializeSpy\0".as_ptr() as LPCSTR,
    ) == 0
    {
        return E_FAIL;
    }

    return g_pfnOLE32_CoRegisterInitializeSpy.unwrap_unchecked()(spy, cookie);
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CoRevokeInitializeSpy(cookie: ULARGE_INTEGER) -> HRESULT {
    static OLE32_DLL_NAME: [u16; 10] = ['o' as u16, 'l' as u16, 'e' as u16, '3' as u16, '2' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
    if Imm32GetFn(
        &raw mut g_pfnOLE32_CoRevokeInitializeSpy as *mut FARPROC,
        &raw mut g_hOle32,
        OLE32_DLL_NAME.as_ptr() as LPCWSTR,
        b"CoRevokeInitializeSpy\0".as_ptr() as LPCSTR,
    ) == 0
    {
        return E_FAIL;
    }
    return g_pfnOLE32_CoRevokeInitializeSpy.unwrap_unchecked()(cookie);
}
#[no_mangle]
pub unsafe extern "C" fn Imm32TF_CreateLangBarMgr(ppBarMgr: *mut *mut ITfLangBarMgr) -> HRESULT {
    if Imm32GetMsctfFn!(TF_CreateLangBarMgr) == 0 {
        return E_FAIL;
    }

    return MSCTF_FN!(TF_CreateLangBarMgr).unwrap_unchecked()(ppBarMgr);
}
#[no_mangle]
pub unsafe extern "C" fn Imm32TF_InvalidAssemblyListCacheIfExist() {
    let dll_name: [u16; 10] = ['m' as u16, 's' as u16, 'c' as u16, 't' as u16, 'f' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];

    if Imm32GetFn(
        core::ptr::addr_of_mut!(g_pfnMSCTF_TF_InvalidAssemblyListCacheIfExist) as *mut FARPROC,
        core::ptr::addr_of_mut!(g_hMsctf),
        dll_name.as_ptr() as LPCWSTR,
        b"TF_InvalidAssemblyListCacheIfExist\0".as_ptr() as LPCSTR,
    ) == 0
    {
        return;
    }

    g_pfnMSCTF_TF_InvalidAssemblyListCacheIfExist.unwrap_unchecked()();
}
#[no_mangle]
pub unsafe extern "C" fn Imm32CheckAndApplyAppCompat(
    mut dwReason: ULONG,
    mut pszAppName: LPCWSTR,
) -> BOOL {
    let mut hinstApphelp: HINSTANCE = core::ptr::null_mut();
    let mut pApphelpCheckIME: FN_ApphelpCheckIME = None;
    if BaseCheckAppcompatCache(
        pszAppName,
        INVALID_HANDLE_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwReason),
    ) != 0
    {
        return (TRUE) as BOOL;
    }
    hinstApphelp = GetModuleHandleW(RUSTCONV_WSTR_EC63E0C750FF.as_ptr()) as HINSTANCE;
    if hinstApphelp.is_null() {
        hinstApphelp = LoadLibraryW(RUSTCONV_WSTR_EC63E0C750FF.as_ptr()) as HINSTANCE;
        if hinstApphelp.is_null() {
            return (TRUE) as BOOL;
        }
    }
    pApphelpCheckIME = core::mem::transmute::<FARPROC, FN_ApphelpCheckIME>(
        GetProcAddress(hinstApphelp, b"ApphelpCheckIME\0".as_ptr() as LPCSTR),
    );
    if pApphelpCheckIME.is_none() {
        return (TRUE) as BOOL;
    }
    let pApphelpCheckIME: unsafe extern "system" fn(LPCWSTR) -> BOOL =
        (pApphelpCheckIME.unwrap_unchecked());
    return pApphelpCheckIME(pszAppName);
}

unsafe extern "C" fn Imm32InitTLS() {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    if g_dwTLSIndex == (-(1 as i32)) as DWORD {
        g_dwTLSIndex = TlsAlloc();
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));
}

unsafe extern "C" fn Imm32AllocateTLS() -> *mut IMMTLSDATA {
    let mut pData: *mut IMMTLSDATA = core::ptr::null_mut::<IMMTLSDATA>();
    if g_dwTLSIndex == (-(1 as i32)) as DWORD {
        return core::ptr::null_mut::<IMMTLSDATA>();
    }
    pData = TlsGetValue(g_dwTLSIndex) as *mut IMMTLSDATA;
    if !pData.is_null() {
        return pData;
    }
    pData = ImmLocalAlloc(
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<IMMTLSDATA>() as DWORD,
    ) as *mut IMMTLSDATA;
    if IS_NULL_UNEXPECTEDLY!(pData) != false {
        return core::ptr::null_mut::<IMMTLSDATA>();
    }
    if IS_ZERO_UNEXPECTEDLY!(TlsSetValue(g_dwTLSIndex, pData as LPVOID)) != false {
        ImmLocalFree!(pData as LPVOID);
        return core::ptr::null_mut::<IMMTLSDATA>();
    }
    return pData;
}

unsafe extern "C" fn Imm32GetTLS() -> *mut IMMTLSDATA {
    if g_dwTLSIndex == -(1 as ::core::ffi::c_int) as DWORD {
        return ::core::ptr::null_mut::<IMMTLSDATA>();
    }

    return TlsGetValue(g_dwTLSIndex) as *mut IMMTLSDATA;
}

unsafe extern "C" fn Imm32GetCoInitCountSkip() -> DWORD {
    let mut pData: *mut IMMTLSDATA = Imm32GetTLS();
    if pData.is_null() {
        return 0;
    }
    return (*pData).dwSkipCount;
}

unsafe extern "C" fn Imm32IncCoInitCountSkip() -> DWORD {
    let mut pData: *mut IMMTLSDATA;
    let mut dwOldSkipCount: DWORD;
    pData = Imm32GetTLS();
    if pData.is_null() {
        return 0;
    }
    dwOldSkipCount = (*pData).dwSkipCount;
    if (*pData).bDoCount != 0 {
        (*pData).dwSkipCount = dwOldSkipCount.wrapping_add(1);
    }
    return dwOldSkipCount;
}

unsafe extern "C" fn Imm32DecCoInitCountSkip() -> DWORD {
    let mut dwSkipCount: DWORD;
    let mut pData: *mut IMMTLSDATA;

    pData = Imm32GetTLS();
    if pData.is_null() {
        return 0;
    }

    dwSkipCount = (*pData).dwSkipCount;
    if (*pData).bDoCount != 0 {
        if dwSkipCount != 0 {
            (*pData).dwSkipCount = dwSkipCount - 1;
        }
    }

    return dwSkipCount;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmEnterCoInitCountSkipMode() {
    let mut pData: *mut IMMTLSDATA = core::ptr::null_mut();

    pData = Imm32GetTLS();
    if !pData.is_null() {
        (*pData).bDoCount += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmLeaveCoInitCountSkipMode() -> BOOL {
    let mut pData: *mut IMMTLSDATA;

    pData = Imm32GetTLS();
    if pData.is_null() || (*pData).bDoCount == 0 {
        return (FALSE) as BOOL;
    }

    (*pData).bDoCount -= 1;
    return (TRUE) as BOOL;
}

unsafe extern "system" fn ISPY_QueryInterface(
    pThis: *mut IInitializeSpy,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let pSpy: *mut ISPY = pThis as *mut ISPY;

    if ppvObj.is_null() {
        return E_INVALIDARG;
    }

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) == 0
        && IsEqualIID!(riid, core::ptr::addr_of!(IID_IInitializeSpy)) == 0
    {
        return E_NOINTERFACE;
    }

    (*pSpy).m_cRefs += 1;
    *ppvObj = pSpy as LPVOID;
    return S_OK;
}
unsafe extern "system" fn ISPY_AddRef(pThis: *mut IInitializeSpy) -> ULONG {
    let pSpy: *mut ISPY = pThis as *mut ISPY;
    (*pSpy).m_cRefs += 1;
    return (*pSpy).m_cRefs as ULONG;
}

unsafe extern "system" fn ISPY_Release(pThis: *mut IInitializeSpy) -> ULONG {
    let pSpy: *mut ISPY = pThis as *mut ISPY;
    (*pSpy).m_cRefs -= 1;
    if (*pSpy).m_cRefs == 0 {
        ImmLocalFree!(pSpy as LPVOID);
        return 0;
    }
    return (*pSpy).m_cRefs as ULONG;
}
unsafe extern "system" fn ISPY_PreInitialize(
    mut pThis: *mut IInitializeSpy,
    mut dwCoInit: DWORD,
    mut dwCurThreadAptRefs: DWORD,
) -> HRESULT {
    let mut cCount: DWORD;

    UNREFERENCED_PARAMETER!(pThis);

    cCount = Imm32IncCoInitCountSkip();
    if dwCoInit == 0
        && (dwCurThreadAptRefs == cCount.wrapping_add(1))
        && ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR) != 0)
    {
        Imm32ActivateOrDeactivateTIM((FALSE) as BOOL);
        CtfImmCoUninitialize();
    }

    return S_OK;
}

unsafe extern "system" fn ISPY_PostInitialize(
    pThis: *mut IInitializeSpy,
    hrCoInit: HRESULT,
    dwCoInit: DWORD,
    dwNewThreadAptRefs: DWORD,
) -> HRESULT {
    let mut CoInitCountSkip: DWORD;

    UNREFERENCED_PARAMETER!(pThis);
    UNREFERENCED_PARAMETER!(dwCoInit);

    CoInitCountSkip = Imm32GetCoInitCountSkip();

    if (hrCoInit != S_FALSE)
        || (dwNewThreadAptRefs != CoInitCountSkip.wrapping_add(2))
        || !((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR) != 0)
    {
        return hrCoInit;
    }

    return S_OK;
}

unsafe extern "system" fn ISPY_PreUninitialize(
    pThis: *mut IInitializeSpy,
    dwCurThreadAptRefs: DWORD,
) -> HRESULT {
    UNREFERENCED_PARAMETER!(pThis);

    if dwCurThreadAptRefs == 1
        && RtlDllShutdownInProgress() == 0
        && Imm32InsideLoaderLock() == 0
        && ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR)) != 0
    {
        let mut pData: *mut IMMTLSDATA = Imm32GetTLS();
        if !pData.is_null() && (*pData).bUninitializing == 0 {
            Imm32CoInitializeEx();
        }
    }

    return S_OK;
}
unsafe extern "system" fn ISPY_PostUninitialize(
    pThis: *mut IInitializeSpy,
    dwNewThreadAptRefs: DWORD,
) -> HRESULT {
    let _ = pThis;
    let _ = dwNewThreadAptRefs;
    Imm32DecCoInitCountSkip();
    return S_OK;
}

unsafe extern "C" fn Imm32AllocIMMISPY() -> *mut ISPY {
    let mut pSpy: *mut ISPY = ImmLocalAlloc(
        0 as DWORD,
        ::core::mem::size_of::<ISPY>() as DWORD,
    ) as *mut ISPY;
    if pSpy.is_null() {
        return ::core::ptr::null_mut::<ISPY>();
    }
    (*pSpy).m_pSpyVtbl = core::ptr::addr_of!(g_vtblISPY);
    (*pSpy).m_cRefs = 1 as LONG;
    return pSpy;
}
#[no_mangle]
pub unsafe extern "C" fn CtfImmCoInitialize() -> HRESULT {
    let mut hr: HRESULT;
    let mut pData: *mut IMMTLSDATA;
    let mut pSpy: *mut ISPY;

    if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR)) != 0 {
        return S_OK; /* Already initialized */
    }

    hr = Imm32CoInitializeEx();
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return hr; /* CoInitializeEx failed */
    }

    (*GetWin32ClientInfo!()).CI_flags |= (CI_CTFCOINIT as ULONG_PTR);
    Imm32InitTLS();

    pData = Imm32AllocateTLS();
    if pData.is_null() || !(*pData).pSpy.is_null() {
        return S_OK; /* Cannot allocate or already it has a spy */
    }

    pSpy = Imm32AllocIMMISPY();
    (*pData).pSpy = pSpy as *mut IInitializeSpy;
    if IS_NULL_UNEXPECTEDLY!(pSpy as PVOID) != false {
        return S_OK; /* Cannot allocate a spy */
    }

    if FAILED_UNEXPECTEDLY!(Imm32CoRegisterInitializeSpy(
        (*pData).pSpy,
        core::ptr::addr_of_mut!((*pData).uliCookie),
    )) != false
    {
        /* Failed to register the spy */
        Imm32DeleteIMMISPY!((*pData).pSpy as PVOID);
        (*pData).pSpy = core::ptr::null_mut();
        (*pData).uliCookie.QuadPart = 0;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmCoUninitialize() {
    let mut pData: *mut IMMTLSDATA;

    if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR)) == 0 {
        return; /* Not CoInitialize'd */
    }

    pData = Imm32GetTLS();
    if !pData.is_null() {
        (*pData).bUninitializing = (TRUE) as i32;
        Imm32CoUninitialize(); /* Do CoUninitialize */
        (*pData).bUninitializing = (FALSE) as i32;

        (*GetWin32ClientInfo!()).CI_flags &= !(CI_CTFCOINIT as ULONG_PTR);
    }

    pData = Imm32AllocateTLS();
    if pData.is_null() || (*pData).pSpy.is_null() {
        return; /* There were no spy */
    }

    /* Our work is done. We don't need spies like you anymore. */
    Imm32CoRevokeInitializeSpy((*pData).uliCookie);
    ISPY_Release((*pData).pSpy);
    (*pData).pSpy = core::ptr::null_mut();
    (*pData).uliCookie.QuadPart = 0;
}
#[no_mangle]
pub unsafe extern "C" fn Imm32LoadCtfIme() -> HINSTANCE {
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let mut szImeFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Lock the IME interface */
    RtlEnterCriticalSection(&raw mut gcsImeDpi);

    loop {
        if !g_hCtfIme.is_null() {
            /* Already loaded? */
            bSuccess = (TRUE) as i32;
            break;
        }

        if ImmLoadLayout(ULongToHandle!(0x04090409) as HKL, &raw mut ImeInfoEx) == 0 {
            break;
        }

        /* Build a path string in system32. The installed IME file must be in system32. */
        Imm32GetSystemLibraryPath(
            &raw mut szImeFile as LPWSTR,
            _countof!(szImeFile) as DWORD,
            &raw mut ImeInfoEx.wszImeFile as *mut WCHAR as LPCWSTR,
        );

        /* Is the CTF IME disabled by app compatibility patcher? */
        if Imm32CheckAndApplyAppCompat(
            0 as ULONG,
            &raw mut szImeFile as *mut WCHAR as LPCWSTR,
        ) == 0
        {
            break; /* This IME is disabled */
        }

        /* Load a CTF IME file */
        g_hCtfIme = LoadLibraryW(&raw mut szImeFile as *mut WCHAR as LPCWSTR);
        if g_hCtfIme.is_null() {
            break;
        }

        /* Assume success */
        bSuccess = (TRUE) as i32;

        /* Retrieve the CTF IME functions */
        g_pfnCtfImeCreateThreadMgr = ::core::mem::transmute::<FARPROC, FN_CtfImeCreateThreadMgr>(
            GetProcAddress(g_hCtfIme, b"CtfImeCreateThreadMgr\0".as_ptr() as LPCSTR),
        );
        if g_pfnCtfImeCreateThreadMgr.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeDestroyThreadMgr = ::core::mem::transmute::<FARPROC, FN_CtfImeDestroyThreadMgr>(
            GetProcAddress(g_hCtfIme, b"CtfImeDestroyThreadMgr\0".as_ptr() as LPCSTR),
        );
        if g_pfnCtfImeDestroyThreadMgr.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeCreateInputContext =
            ::core::mem::transmute::<FARPROC, FN_CtfImeCreateInputContext>(GetProcAddress(
                g_hCtfIme,
                b"CtfImeCreateInputContext\0".as_ptr() as LPCSTR,
            ));
        if g_pfnCtfImeCreateInputContext.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeDestroyInputContext =
            ::core::mem::transmute::<FARPROC, FN_CtfImeDestroyInputContext>(GetProcAddress(
                g_hCtfIme,
                b"CtfImeDestroyInputContext\0".as_ptr() as LPCSTR,
            ));
        if g_pfnCtfImeDestroyInputContext.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeSetActiveContextAlways =
            ::core::mem::transmute::<FARPROC, FN_CtfImeSetActiveContextAlways>(GetProcAddress(
                g_hCtfIme,
                b"CtfImeSetActiveContextAlways\0".as_ptr() as LPCSTR,
            ));
        if g_pfnCtfImeSetActiveContextAlways.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeProcessCicHotkey =
            ::core::mem::transmute::<FARPROC, FN_CtfImeProcessCicHotkey>(GetProcAddress(
                g_hCtfIme,
                b"CtfImeProcessCicHotkey\0".as_ptr() as LPCSTR,
            ));
        if g_pfnCtfImeProcessCicHotkey.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeDispatchDefImeMessage =
            ::core::mem::transmute::<FARPROC, FN_CtfImeDispatchDefImeMessage>(GetProcAddress(
                g_hCtfIme,
                b"CtfImeDispatchDefImeMessage\0".as_ptr() as LPCSTR,
            ));
        if g_pfnCtfImeDispatchDefImeMessage.is_none() {
            bSuccess = 0;
            break;
        }

        g_pfnCtfImeIsIME = ::core::mem::transmute::<FARPROC, FN_CtfImeIsIME>(GetProcAddress(
            g_hCtfIme,
            b"CtfImeIsIME\0".as_ptr() as LPCSTR,
        ));
        if g_pfnCtfImeIsIME.is_none() {
            bSuccess = 0;
            break;
        }

        if (0) == 0 {
            break;
        }
    }

    /* Unload the CTF IME if failed */
    if bSuccess == 0 {
        /* Set NULL to the function pointers */
        g_pfnCtfImeCreateThreadMgr = None;
        g_pfnCtfImeDestroyThreadMgr = None;
        g_pfnCtfImeCreateInputContext = None;
        g_pfnCtfImeDestroyInputContext = None;
        g_pfnCtfImeSetActiveContextAlways = None;
        g_pfnCtfImeProcessCicHotkey = None;
        g_pfnCtfImeDispatchDefImeMessage = None;
        g_pfnCtfImeIsIME = None;

        if !g_hCtfIme.is_null() {
            FreeLibrary(g_hCtfIme as HMODULE);
            g_hCtfIme = ::core::ptr::null_mut::<HINSTANCE__>();
        }
    }

    /* Unlock the IME interface */
    RtlLeaveCriticalSection(&raw mut gcsImeDpi);

    return g_hCtfIme;
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeCreateThreadMgr() -> HRESULT {
    if Imm32LoadCtfIme().is_null() {
        return E_FAIL;
    }

    return g_pfnCtfImeCreateThreadMgr.unwrap_unchecked()();
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeProcessCicHotkey(hIMC: HIMC, vKey: UINT, lParam: LPARAM) -> BOOL {
    if Imm32LoadCtfIme().is_null() {
        return (FALSE) as BOOL;
    }
    return g_pfnCtfImeProcessCicHotkey.unwrap_unchecked()(hIMC, vKey, lParam) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeDestroyThreadMgr() -> HRESULT {
    if Imm32LoadCtfIme().is_null() {
        return E_FAIL;
    }
    return g_pfnCtfImeDestroyThreadMgr.unwrap_unchecked()();
}

#[no_mangle]
pub unsafe extern "system" fn CtfAImmIsIME(hKL: HKL) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/ctf.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CtfAImmIsIME\0" as *const u8 as *const ::core::ffi::c_char,
            769 as ::core::ffi::c_int,
            b"(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hKL,
        );
    }
    if Imm32LoadCtfIme().is_null() {
        return ImmIsIME(hKL);
    }
    return g_pfnCtfImeIsIME.unwrap_unchecked()(hKL);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmIsCiceroStartedInThread() -> BOOL {
    return (((*GetWin32ClientInfo!()).CI_flags & (CI_CICERO_STARTED as ULONG_PTR)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmSetCiceroStartInThread(bStarted: BOOL) {
    if bStarted != 0 {
        (*GetWin32ClientInfo!()).CI_flags |= (CI_CICERO_STARTED as ULONG_PTR);
    } else {
        (*GetWin32ClientInfo!()).CI_flags &= !(CI_CICERO_STARTED as ULONG_PTR);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmSetAppCompatFlags(dwFlags: DWORD) {
    if (dwFlags & (0xF0FFFFFF as DWORD)) == 0 {
        g_aimm_compat_flags = dwFlags;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeCreateInputContext(hIMC: HIMC) -> HRESULT {
    if Imm32LoadCtfIme().is_null() {
        return E_FAIL;
    }

    return (g_pfnCtfImeCreateInputContext.unwrap_unchecked())(hIMC);
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeDestroyInputContext(hIMC: HIMC) -> HRESULT {
    if Imm32LoadCtfIme().is_null() {
        return E_FAIL;
    }

    return (g_pfnCtfImeDestroyInputContext.unwrap_unchecked())(hIMC);
}
#[no_mangle]
pub unsafe extern "C" fn CtfImeSetActiveContextAlways(mut hIMC: HIMC, mut fActive: BOOL, mut hWnd: HWND, mut hKL: HKL) -> HRESULT {
    if !Imm32LoadCtfIme().is_null() {
    } else {
        return E_FAIL;
    }
    return g_pfnCtfImeSetActiveContextAlways.unwrap_unchecked()(
        hIMC, fActive, hWnd, hKL,
    );
}

unsafe extern "system" fn Imm32EnumCreateCtfICProc(hIMC: HIMC, lParam: LPARAM) -> BOOL {
    let _ = lParam;
    CtfImeCreateInputContext(hIMC);
    return (TRUE) as BOOL; /* Continue */
}

unsafe extern "C" fn Imm32IsTIMDisabledInRegistry() -> BOOL {
    let mut dwData: DWORD;
    let mut cbData: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LSTATUS;

    error = RegOpenKeyW(HKEY_CURRENT_USER, RUSTCONV_WSTR_B5364BD7CA8B.as_ptr() as *const u16, &mut hKey);
    if (error) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    dwData = 0;
    cbData = core::mem::size_of::<DWORD>() as DWORD;
    RegQueryValueExW(hKey, RUSTCONV_WSTR_0B087FAD2DEC.as_ptr() as *const u16, core::ptr::null_mut(), core::ptr::null_mut(), &mut dwData as *mut DWORD as LPBYTE, &mut cbData);
    RegCloseKey(hKey);
    (dwData != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn Imm32ActivateOrDeactivateTIM(bCreate: BOOL) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if IS_CICERO_MODE!() == false
        || IS_16BIT_MODE!()
        || ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR)) == 0
    {
        return S_OK; /* No need to activate/de-activate TIM */
    }

    if bCreate != 0 {
        if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFTIM as ULONG_PTR)) == 0 {
            hr = CtfImeCreateThreadMgr();
            if hr >= 0 {
                (*GetWin32ClientInfo!()).CI_flags |= (CI_CTFTIM as ULONG_PTR);
            }
        }
    } else {
        /* Destroy */
        if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFTIM as ULONG_PTR)) != 0 {
            hr = CtfImeDestroyThreadMgr();
            if hr >= 0 {
                (*GetWin32ClientInfo!()).CI_flags &= !(CI_CTFTIM as ULONG_PTR);
            }
        }
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn CtfImmTIMDestroyInputContext(hIMC: HIMC) -> HRESULT {
    if ((IS_CICERO_MODE!()) == false) || IS_CICERO_COMPAT_DISABLED!() {
        return E_NOINTERFACE;
    }

    return CtfImeDestroyInputContext(hIMC);
}
#[no_mangle]
pub unsafe extern "C" fn CtfImmTIMCreateInputContext(hIMC: HIMC) -> HRESULT {
    let mut pClientImc: PCLIENTIMC = core::ptr::null_mut();
    let mut dwImeThreadId: DWORD_PTR = 0;
    let mut dwCurrentThreadId: DWORD_PTR = 0;
    let mut hr: HRESULT = S_FALSE;

    pClientImc = ImmLockClientImc(hIMC);
    if pClientImc.is_null() {
        return E_FAIL;
    }

    if ((*GetWin32ClientInfo!()).CI_flags & (CI_AIMMACTIVATED as ULONG_PTR)) != 0 {
        if (*pClientImc).bCtfIme == 0 {
            dwImeThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID);
            dwCurrentThreadId = (GetCurrentThreadId()) as _;
            if dwImeThreadId == dwCurrentThreadId {
                (*pClientImc).bCtfIme = (TRUE) as i32;
                hr = CtfImeCreateInputContext(hIMC);
                if FAILED_UNEXPECTEDLY!(hr) {
                    (*pClientImc).bCtfIme = (FALSE) as i32;
                }
            }
        }
    } else {
        if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFTIM as ULONG_PTR)) == 0 {
            return S_OK;
        }

        if (*pClientImc).bCtfIme == 0 {
            dwImeThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID);
            dwCurrentThreadId = (GetCurrentThreadId()) as _;
            if (dwImeThreadId == dwCurrentThreadId) && IS_CICERO_MODE!() && ((IS_16BIT_MODE!()) == (false)) {
                (*pClientImc).bCtfIme = (TRUE) as i32;
                hr = CtfImeCreateInputContext(hIMC);
                if FAILED_UNEXPECTEDLY!(hr) {
                    (*pClientImc).bCtfIme = (FALSE) as i32;
                }
            }
        }
    }

    ImmUnlockClientImc(pClientImc);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmLastEnabledWndDestroy(bCreate: BOOL) -> HRESULT {
    // TRACE("(%d)\n", bCreate);
    Imm32ActivateOrDeactivateTIM(bCreate)
}

#[no_mangle]
pub unsafe extern "system" fn CtfAImmActivate(phinstCtfIme: *mut HINSTANCE) -> HRESULT {
    let mut hr: HRESULT;
    let mut hinstCtfIme: HINSTANCE;

    /* Load a CTF IME file if necessary */
    hinstCtfIme = Imm32LoadCtfIme();

    /* Create a thread manager of the CTF IME */
    hr = CtfImeCreateThreadMgr();
    if hr == S_OK {
        /* Update CI_... flags of the thread client info */
        (*GetWin32ClientInfo!()).CI_flags |= CI_AIMMACTIVATED as ULONG_PTR; /* Activate AIMM */
        (*GetWin32ClientInfo!()).CI_flags &= !(CI_TSFDISABLED as ULONG_PTR); /* Enable TSF */

        /* Create the CTF input contexts */
        ImmEnumInputContext(
            0,
            Some(Imm32EnumCreateCtfICProc),
            0,
        );
    }

    if !phinstCtfIme.is_null() {
        *phinstCtfIme = hinstCtfIme;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CtfAImmDeactivate(bDestroy: BOOL) -> HRESULT {
    let mut hr: HRESULT;

    if bDestroy == 0 {
        return E_FAIL;
    }

    hr = CtfImeDestroyThreadMgr();
    if hr == S_OK {
        (*GetWin32ClientInfo!()).CI_flags &= !(CI_AIMMACTIVATED as ULONG_PTR); /* Deactivate AIMM */
        (*GetWin32ClientInfo!()).CI_flags |= (CI_TSFDISABLED as ULONG_PTR); /* Disable TSF */
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmIsCiceroEnabled() -> BOOL {
    (IS_CICERO_MODE!()) as i32
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmIsTextFrameServiceDisabled() -> BOOL {
    return (((*GetWin32ClientInfo!()).CI_flags & (CI_TSFDISABLED as ULONG_PTR)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmDisableTextFrameService(dwThreadId: DWORD) -> BOOL {
    let mut hr: HRESULT = S_OK;

    if dwThreadId == (-(1 as i32)) as DWORD {
        g_disable_CUAS_flag = (TRUE) as i32;
    }

    if (dwThreadId != 0 && g_disable_CUAS_flag == 0)
        || ((*GetWin32ClientInfo!()).CI_flags & CI_TSFDISABLED as ULONG_PTR) != 0
    {
        return (TRUE) as BOOL;
    }

    (*GetWin32ClientInfo!()).CI_flags |= CI_TSFDISABLED as ULONG_PTR;

    if IS_CICERO_MODE!() != false
        && IS_16BIT_MODE!() == false
        && ((*GetWin32ClientInfo!()).CI_flags & CI_CTFCOINIT as ULONG_PTR) != 0
        && ((*GetWin32ClientInfo!()).CI_flags & CI_CTFTIM as ULONG_PTR) != 0
    {
        hr = CtfImeDestroyThreadMgr();
        if hr >= 0 {
            (*GetWin32ClientInfo!()).CI_flags &= !(CI_CTFTIM as ULONG_PTR);
            CtfImmCoUninitialize();
        }
    }

    (hr == S_OK) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmTIMActivate(mut hKL: HKL) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if g_disable_CUAS_flag != 0 {
        (*GetWin32ClientInfo!()).CI_flags |= (CI_TSFDISABLED as ULONG_PTR);
        return S_OK;
    }

    if ((*GetWin32ClientInfo!()).CI_flags & (CI_TSFDISABLED as ULONG_PTR)) != 0 {
        return S_OK;
    }

    if Imm32IsTIMDisabledInRegistry() != 0 {
        (*GetWin32ClientInfo!()).CI_flags |= (CI_TSFDISABLED as ULONG_PTR);
        return S_OK;
    }

    if Imm32IsInteractiveUserLogon() == 0 || Imm32IsRunningInMsoobe() != 0 {
        return S_OK;
    }

    if Imm32IsCUASEnabledInRegistry() == 0 {
        (*GetWin32ClientInfo!()).CI_flags |= (CI_TSFDISABLED as ULONG_PTR);
        return S_OK;
    }

    if ((*(*NtCurrentTeb()).ProcessEnvironmentBlock)
        .AppCompatFlags
        .__bindgen_anon_1.LowPart
        & 0x100)
        != 0
    {
        (*GetWin32ClientInfo!()).CI_flags |= (CI_TSFDISABLED as ULONG_PTR);
        return S_OK;
    }

    if RtlIsThreadWithinLoaderCallout() != 0 || Imm32InsideLoaderLock() != 0 {
        return S_OK;
    }

    if IS_CICERO_MODE!() == false || IS_16BIT_MODE!() != false {
        return S_OK;
    }

    if IS_IME_HKL!(hKL) != false {
        hKL = UlongToHandle!(MAKELONG!(LOWORD!(hKL), LOWORD!(hKL))) as HKL;
    }

    if ImmLoadIME(hKL) == 0 {
        Imm32TF_InvalidAssemblyListCacheIfExist();
    }

    CtfImmCoInitialize();

    if ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFCOINIT as ULONG_PTR)) != 0
        && ((*GetWin32ClientInfo!()).CI_flags & (CI_CTFTIM as ULONG_PTR)) == 0
    {
        hr = CtfImeCreateThreadMgr();
        if hr >= 0 {
            (*GetWin32ClientInfo!()).CI_flags |= (CI_CTFTIM as ULONG_PTR);
        }
    }

    return hr;
}
unsafe extern "system" fn Imm32DelaySetLangBandProc(arg: LPVOID) -> DWORD {
    let mut hwndDefIME: HWND;
    let mut wParam: WPARAM;
    let mut lResult: DWORD_PTR = core::mem::zeroed();
    let pSetBand: PIMM_DELAY_SET_LANG_BAND = arg as PIMM_DELAY_SET_LANG_BAND;

    Sleep(3000 as DWORD); /* Delay 3 seconds! */

    hwndDefIME = ImmGetDefaultIMEWnd((*pSetBand).hWnd);
    if !hwndDefIME.is_null() {
        wParam = (if (*pSetBand).fSet != 0 {
            IMS_SETLANGBAND
        } else {
            IMS_UNSETLANGBAND
        }) as WPARAM;
        SendMessageTimeoutW(
            hwndDefIME,
            WM_IME_SYSTEM as UINT,
            wParam,
            (*pSetBand).hWnd as LPARAM,
            (SMTO_BLOCK | SMTO_ABORTIFHUNG) as UINT,
            5000 as UINT,
            core::ptr::addr_of_mut!(lResult),
        );
    }
    ImmLocalFree!(pSetBand as LPVOID);
    return FALSE as DWORD;
}

#[no_mangle]
pub unsafe extern "C" fn CtfImmSetLangBand(hWnd: HWND, fSet: BOOL) -> LRESULT {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut pSetBand: PIMM_DELAY_SET_LANG_BAND = core::ptr::null_mut();
    let mut lResult: DWORD_PTR = 0 as DWORD_PTR;

    if !hWnd.is_null() && !gpsi.is_null() {
        pWnd = ValidateHwndNoErr!(hWnd as HANDLE).cast();
    }

    if IS_NULL_UNEXPECTEDLY!(pWnd) {
        return 0 as LRESULT;
    }

    if ((*pWnd).state2 & WNDS2_WMCREATEMSGPROCESSED) != 0 {
        SendMessageTimeoutW(
            hWnd,
            (WM_USER + 0x105) as UINT,
            0 as WPARAM,
            fSet as LPARAM,
            (SMTO_BLOCK | SMTO_ABORTIFHUNG) as UINT,
            5000 as UINT,
            &mut lResult as *mut DWORD_PTR,
        );
        return lResult as LRESULT;
    }

    pSetBand = ImmLocalAlloc(
        0 as DWORD,
        core::mem::size_of::<IMM_DELAY_SET_LANG_BAND>() as DWORD,
    ) as PIMM_DELAY_SET_LANG_BAND;
    if IS_NULL_UNEXPECTEDLY!(pSetBand) {
        return 0 as LRESULT;
    }

    (*pSetBand).hWnd = hWnd;
    (*pSetBand).fSet = fSet;

    hThread = CreateThread(
        core::ptr::null_mut(),
        0 as DWORD,
        Some(Imm32DelaySetLangBandProc),
        pSetBand as LPVOID,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    if !hThread.is_null() {
        CloseHandle(hThread);
    }
    return 0 as LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmGenerateMessage(hIMC: HIMC, bSend: BOOL) -> BOOL {
    let mut dwImeThreadId: DWORD_PTR;
    let mut dwCurrentThreadId: DWORD_PTR;
    let mut pClientImc: PCLIENTIMC;
    let mut bUnicode: BOOL;
    let mut pIC: LPINPUTCONTEXT;
    let mut iMsg: DWORD;
    let mut dwNumMsgBuf: DWORD;
    let mut pOldTransMsg: LPTRANSMSG;
    let mut pNewTransMsg: LPTRANSMSG;
    let mut cbTransMsg: SIZE_T;

    dwImeThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD_PTR;
    dwCurrentThreadId = GetCurrentThreadId() as DWORD_PTR;
    if dwImeThreadId != dwCurrentThreadId {
        return (FALSE) as BOOL;
    }

    pClientImc = ImmLockClientImc(hIMC);
    if IS_NULL_UNEXPECTEDLY!(pClientImc) != false {
        return (FALSE) as BOOL;
    }

    bUnicode = (((*pClientImc).dwFlags & CLIENTIMC_WIDE) != 0) as BOOL;
    ImmUnlockClientImc(pClientImc);

    pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXT;
    if IS_NULL_UNEXPECTEDLY!(pIC) != false {
        return (FALSE) as BOOL;
    }

    dwNumMsgBuf = (*pIC).dwNumMsgBuf;
    pOldTransMsg = ImmLockIMCC((*pIC).hMsgBuf) as LPTRANSMSG;
    if IS_NULL_UNEXPECTEDLY!(pOldTransMsg) != false {
        (*pIC).dwNumMsgBuf = 0;
        ImmUnlockIMC(hIMC);
        return (TRUE) as BOOL;
    }

    cbTransMsg = (core::mem::size_of::<TRANSMSG>()).wrapping_mul(dwNumMsgBuf as usize) as SIZE_T;
    pNewTransMsg = ImmLocalAlloc(0, cbTransMsg as DWORD) as PTRANSMSG as LPTRANSMSG;
    if IS_NULL_UNEXPECTEDLY!(pNewTransMsg) != false {
        ImmUnlockIMCC((*pIC).hMsgBuf);
        (*pIC).dwNumMsgBuf = 0;
        ImmUnlockIMC(hIMC);
        return (TRUE) as BOOL;
    }

    RtlCopyMemory!(pNewTransMsg as *mut core::ffi::c_void, pOldTransMsg as *const core::ffi::c_void, cbTransMsg as usize);

    iMsg = 0;
    while iMsg < dwNumMsgBuf {
        let hWnd: HWND = (*pIC).hWnd;
        let uMsg: UINT = (*pNewTransMsg.offset(iMsg as isize)).message;
        let wParam: WPARAM = (*pNewTransMsg.offset(iMsg as isize)).wParam;
        let lParam: LPARAM = (*pNewTransMsg.offset(iMsg as isize)).lParam;
        if bSend != 0 {
            if bUnicode != 0 {
                SendMessageW(hWnd, uMsg, wParam, lParam);
            } else {
                SendMessageA(hWnd, uMsg, wParam, lParam);
            }
        } else if bUnicode != 0 {
            PostMessageW(hWnd, uMsg, wParam, lParam);
        } else {
            PostMessageA(hWnd, uMsg, wParam, lParam);
        }
        iMsg = iMsg.wrapping_add(1);
    }

    ImmLocalFree!(pNewTransMsg as *mut core::ffi::c_void);
    ImmUnlockIMCC((*pIC).hMsgBuf);
    (*pIC).dwNumMsgBuf = 0; /* Processed */
    ImmUnlockIMC(hIMC);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmHideToolbarWnd() -> DWORD {
    let mut pBarMgr: *mut ITfLangBarMgr = core::ptr::null_mut::<ITfLangBarMgr>();
    let mut dwShowFlags: DWORD = 0 as DWORD;
    let mut bShown: BOOL = 0;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/ctf.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CtfImmHideToolbarWnd\0" as *const u8 as *const ::core::ffi::c_char,
            1369 as ::core::ffi::c_int,
            b"()\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if Imm32TF_CreateLangBarMgr(&raw mut pBarMgr) < (0 as HRESULT) {
        return dwShowFlags;
    }
    if (*(*pBarMgr).lpVtbl)
        .GetShowFloatingStatus
        .unwrap_unchecked()(pBarMgr, &raw mut dwShowFlags)
        >= (0 as HRESULT)
    {
        bShown = ((dwShowFlags & (0x800 as DWORD)) == 0) as ::core::ffi::c_int as BOOL;
        dwShowFlags &= 0xF as DWORD;
        if bShown != 0 {
            (*(*pBarMgr).lpVtbl)
                .ShowFloating
                .unwrap_unchecked()(pBarMgr, 8 as DWORD);
        }
    }
    (*(*pBarMgr).lpVtbl)
        .Release
        .unwrap_unchecked()(pBarMgr);
    return dwShowFlags;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmRestoreToolbarWnd(pUnused: LPVOID, dwShowFlags: DWORD) {
    let mut hr: HRESULT;
    let mut pBarMgr: *mut ITfLangBarMgr = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(pUnused);

    hr = Imm32TF_CreateLangBarMgr(&mut pBarMgr);
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return;
    }

    if dwShowFlags != 0 {
        (*(*pBarMgr).lpVtbl)
            .ShowFloating
            .unwrap_unchecked()(pBarMgr, dwShowFlags);
    }

    (*(*pBarMgr).lpVtbl)
        .Release
        .unwrap_unchecked()(pBarMgr);
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmDispatchDefImeMessage(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if RtlDllShutdownInProgress() as i32 != 0
        || Imm32InsideLoaderLock() != 0
        || Imm32LoadCtfIme().is_null()
    {
        return 0;
    }

    return g_pfnCtfImeDispatchDefImeMessage.unwrap_unchecked()(
        hWnd, uMsg, wParam, lParam,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmIsGuidMapEnable(hIMC: HIMC) -> BOOL {
    let mut dwThreadId: DWORD = 0;
    let mut hKL: HKL = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    if IS_CICERO_MODE!() == false || IS_16BIT_MODE!() != false {
        return ret;
    }

    dwThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
    hKL = GetKeyboardLayout(dwThreadId);

    if IS_IME_HKL!(hKL) != false {
        return ret;
    }

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return ret;
    }

    ret = (*pImeDpi).CtfImeIsGuidMapEnable.unwrap_unchecked()(hIMC);

    ImmUnlockImeDpi(pImeDpi);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CtfImmGetGuidAtom(
    hIMC: HIMC,
    dwUnknown: DWORD,
    pdwGuidAtom: LPDWORD,
) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    let mut pImeDpi: PIMEDPI;
    let mut dwThreadId: DWORD;
    let mut hKL: HKL;

    *pdwGuidAtom = 0;

    if ((IS_CICERO_MODE!()) == false) || IS_16BIT_MODE!() {
        return hr;
    }

    dwThreadId = NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
    hKL = GetKeyboardLayout(dwThreadId);
    if IS_IME_HKL!(hKL) {
        return S_OK;
    }

    pImeDpi = Imm32FindOrLoadImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) {
        return hr;
    }

    hr = (*pImeDpi).CtfImeGetGuidAtom.unwrap_unchecked()(hIMC, dwUnknown, pdwGuidAtom);

    ImmUnlockImeDpi(pImeDpi);
    return hr;
}
