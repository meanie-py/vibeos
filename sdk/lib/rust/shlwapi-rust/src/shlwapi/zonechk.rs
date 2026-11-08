use crate::shlwapi::string::SHAnsiToUnicode;
use crate::shlwapi::ordinal::SHPinDllOfCLSID;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut g_pZoneMgrCF: *mut IClassFactory =
    ::core::ptr::null::<IClassFactory>() as *mut IClassFactory;

#[no_mangle]
pub static mut g_csZoneMgrLock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

static mut g_hinstZoneMgr: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// NO_SHLWAPI_REG (flag-only #define, no value; no Rust constant emitted)


unsafe extern "C" fn SHLWAPI_GetCachedZonesManagerInner(
    riid: *const IID,
    mut ppv: *mut PVOID,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut pCF: *mut IClassFactory = ::core::ptr::null_mut::<IClassFactory>();
    if g_pZoneMgrCF.is_null() {
        hr = CoGetClassObject(
            &raw const CLSID_InternetSecurityManager,
            CLSCTX_INPROC_SERVER as DWORD,
            ::core::ptr::null_mut::<COSERVERINFO>(),
            &raw const IID_IClassFactory,
            &raw mut pCF as *mut LPVOID,
        );
        if hr < (0 as HRESULT) {
            *ppv = NULL as PVOID;
            return hr;
        }
        g_pZoneMgrCF = pCF;
        g_hinstZoneMgr = SHPinDllOfCLSID(&raw const CLSID_InternetSecurityManager) as HINSTANCE;
    }
    return (*(*g_pZoneMgrCF).lpVtbl)
        .CreateInstance
        .unwrap_unchecked()(
        g_pZoneMgrCF,
        ::core::ptr::null_mut::<IUnknown>(),
        riid,
        ppv as *mut *mut ::core::ffi::c_void,
    );
}

unsafe extern "C" fn SHLWAPI_GetCachedZonesManager(
    riid: REFIID,
    ppv: *mut PVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    EnterCriticalSection(core::ptr::addr_of_mut!(g_csZoneMgrLock));
    hr = SHLWAPI_GetCachedZonesManagerInner(riid, ppv);
    LeaveCriticalSection(core::ptr::addr_of_mut!(g_csZoneMgrLock));
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn SHLWAPI_DeleteCachedZonesManager() {
    EnterCriticalSection(core::ptr::addr_of_mut!(g_csZoneMgrLock));
    if !g_pZoneMgrCF.is_null() {
        (((*(*g_pZoneMgrCF).lpVtbl).Release).unwrap_unchecked())(g_pZoneMgrCF);
        g_pZoneMgrCF = core::ptr::null_mut();
    }
    if !g_hinstZoneMgr.is_null() {
        FreeLibrary(g_hinstZoneMgr);
        g_hinstZoneMgr = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(g_csZoneMgrLock));
}
#[no_mangle]
pub unsafe extern "C" fn SuperPrivate_ZoneCheckPath(pszPath: PCWSTR, dwExpectedZone: DWORD) -> HRESULT {
    let mut pISM: *mut IInternetSecurityManager = core::ptr::null_mut::<IInternetSecurityManager>();
    let mut hr: HRESULT = SHLWAPI_GetCachedZonesManager(
        core::ptr::addr_of!(IID_IInternetSecurityManager),
        core::ptr::addr_of_mut!(pISM) as *mut PVOID,
    );
    if hr < 0 {
        return E_ACCESSDENIED;
    }

    let mut dwRealZone: DWORD = (URLZONE_UNTRUSTED) as u32;
    hr = (((*(*pISM).lpVtbl).MapUrlToZone).unwrap_unchecked())(
        pISM,
        pszPath as LPCWSTR,
        core::ptr::addr_of_mut!(dwRealZone),
        0,
    );
    if hr >= 0 && dwRealZone == dwExpectedZone {
        hr = S_OK;
    } else {
        hr = E_ACCESSDENIED;
    }

    (((*(*pISM).lpVtbl).Release).unwrap_unchecked())(pISM);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlExCacheA(
    pszUrl: PCSTR,
    pbPolicy: PBYTE,
    cbPolicy: DWORD,
    pbContext: PBYTE,
    cbContext: DWORD,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
    pISM: *mut IInternetSecurityManager,
) -> HRESULT {
    let mut szUrl: [WCHAR; 2048] = [0; 2048];
    if pszUrl.is_null() {
        ERR!("pszUrl was NULL\n");
        return E_INVALIDARG;
    }
    SHAnsiToUnicode(pszUrl as LPCSTR, szUrl.as_mut_ptr() as LPWSTR, _countof!(szUrl) as INT);
    return ZoneCheckUrlExCacheW(
        szUrl.as_mut_ptr() as PCWSTR,
        pbPolicy,
        cbPolicy,
        pbContext,
        cbContext,
        dwAction,
        dwFlags,
        pSecuritySite,
        pISM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlExCacheW(
    pszUrl: PCWSTR,
    mut pbPolicy: PBYTE,
    mut cbPolicy: DWORD,
    mut pbContext: PBYTE,
    mut cbContext: DWORD,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
    pISM: *mut IInternetSecurityManager,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pWorkISM: *mut IInternetSecurityManager = core::ptr::null_mut();
    let mut dwPolicyBuf: DWORD;
    let mut dwContextBuf: DWORD;

    if pszUrl.is_null() {
        ERR!("pszUrl was NULL\n");
        return E_INVALIDARG;
    }

    if !pISM.is_null() && !(*pISM).lpVtbl.is_null() {
        hr = (*(*pISM).lpVtbl)
            .QueryInterface
            .unwrap_unchecked()(
            pISM,
            core::ptr::addr_of!(IID_IInternetSecurityManager),
            core::ptr::addr_of_mut!(pWorkISM) as *mut *mut core::ffi::c_void,
        );
    } else {
        hr = SHLWAPI_GetCachedZonesManager(
            core::ptr::addr_of!(IID_IInternetSecurityManager),
            core::ptr::addr_of_mut!(pWorkISM) as *mut PVOID,
        );
    }

    if hr < 0 {
        ERR!("hr: 0x%lX\n", hr);
        return hr;
    }

    if !pSecuritySite.is_null() {
        (*(*pWorkISM).lpVtbl)
            .SetSecuritySite
            .unwrap_unchecked()(pWorkISM, pSecuritySite);
    }

    if pbContext.is_null() {
        dwContextBuf = 0;
        pbContext = core::ptr::addr_of_mut!(dwContextBuf) as PBYTE;
        cbContext = core::mem::size_of::<DWORD>() as DWORD;
    }

    if pbPolicy.is_null() {
        dwPolicyBuf = 0;
        pbPolicy = core::ptr::addr_of_mut!(dwPolicyBuf) as PBYTE;
        cbPolicy = core::mem::size_of::<DWORD>() as DWORD;
    }

    hr = (*(*pWorkISM).lpVtbl)
        .ProcessUrlAction
        .unwrap_unchecked()(
        pWorkISM,
        pszUrl,
        dwAction,
        pbPolicy,
        cbPolicy,
        pbContext,
        cbContext,
        dwFlags,
        0,
    );

    if !pSecuritySite.is_null() {
        (*(*pWorkISM).lpVtbl)
            .SetSecuritySite
            .unwrap_unchecked()(
            pWorkISM,
            core::ptr::null_mut(),
        );
    }

    (*(*pWorkISM).lpVtbl)
        .Release
        .unwrap_unchecked()(pWorkISM);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckPathA(
    pszPath: PCSTR,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    let mut szPath: [WCHAR; 2048] = [0; 2048];
    if pszPath.is_null() {
        ERR!("pszPath was NULL\n");
        return E_INVALIDARG;
    }
    SHAnsiToUnicode(
        pszPath as LPCSTR,
        &raw mut szPath as LPWSTR,
        _countof!(szPath) as INT,
    );
    return ZoneCheckPathW(
        &raw mut szPath as *mut WCHAR as PCWSTR,
        dwAction,
        dwFlags,
        pSecuritySite,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckPathW(
    pszPath: PCWSTR,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    return ZoneCheckUrlW(pszPath, dwAction, dwFlags | (PUAF_ISFILE as DWORD), pSecuritySite);
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlA(
    pszUrl: PCSTR,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    let mut szUrl: [WCHAR; 2048] = [0; 2048];
    if pszUrl.is_null() {
        ERR!("pszUrl was NULL\n");
        return E_INVALIDARG;
    }
    SHAnsiToUnicode(pszUrl, core::ptr::addr_of_mut!(szUrl) as LPWSTR, (_countof!(szUrl)) as i32);
    return ZoneCheckUrlW(
        core::ptr::addr_of_mut!(szUrl) as PCWSTR,
        dwAction,
        dwFlags,
        pSecuritySite,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlW(
    pszUrl: PCWSTR,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    return ZoneCheckUrlExW(pszUrl, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, dwAction, dwFlags, pSecuritySite);
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlExA(
    pszUrl: PCSTR,
    pbPolicy: PBYTE,
    cbPolicy: DWORD,
    pbContext: PBYTE,
    cbContext: DWORD,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    let mut szUrl: [WCHAR; 2048] = [0; 2048];
    if pszUrl.is_null() {
        ERR!("pszUrl was NULL\n");
        return E_INVALIDARG;
    }
    SHAnsiToUnicode(
        pszUrl as LPCSTR,
        core::ptr::addr_of_mut!(szUrl) as LPWSTR,
        _countof!(szUrl) as INT,
    );
    return ZoneCheckUrlExW(
        core::ptr::addr_of_mut!(szUrl) as *mut WCHAR as PCWSTR,
        pbPolicy,
        cbPolicy,
        pbContext,
        cbContext,
        dwAction,
        dwFlags,
        pSecuritySite,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckUrlExW(
    pszUrl: PCWSTR,
    pbPolicy: PBYTE,
    cbPolicy: DWORD,
    pbContext: PBYTE,
    cbContext: DWORD,
    dwAction: DWORD,
    dwFlags: DWORD,
    pSecuritySite: *mut IInternetSecurityMgrSite,
) -> HRESULT {
    ZoneCheckUrlExCacheW(pszUrl, pbPolicy, cbPolicy, pbContext, cbContext, dwAction, dwFlags, pSecuritySite, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckHost(
    mut pISM: *mut IInternetSecurityManager,
    mut pszUrl: PCWSTR,
    mut dwAction: DWORD,
) -> HRESULT {
    return ZoneCheckHostEx(
        pISM,
        core::ptr::null_mut::<BYTE>(),
        0 as DWORD,
        core::ptr::null_mut::<BYTE>(),
        0 as DWORD,
        pszUrl,
        dwAction,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ZoneCheckHostEx(
    pISM: *mut IInternetSecurityManager,
    mut pbPolicy: PBYTE,
    mut cbPolicy: DWORD,
    mut pbContext: PBYTE,
    mut cbContext: DWORD,
    pszUrl: PCWSTR,
    dwAction: DWORD,
) -> HRESULT {
    let mut dwPolicyBuf: DWORD = 0;
    let mut dwContextBuf: DWORD = 0;

    if pISM.is_null() || pszUrl.is_null() {
        return E_INVALIDARG;
    }

    if pbPolicy.is_null() {
        dwPolicyBuf = 0;
        pbPolicy = core::ptr::addr_of_mut!(dwPolicyBuf) as PBYTE;
        cbPolicy = core::mem::size_of::<DWORD>() as DWORD;
    }

    if pbContext.is_null() {
        dwContextBuf = 0;
        pbContext = core::ptr::addr_of_mut!(dwContextBuf) as PBYTE;
        cbContext = core::mem::size_of::<DWORD>() as DWORD;
    }

    return (*(*pISM).lpVtbl)
        .ProcessUrlAction
        .unwrap_unchecked()(
        pISM,
        pszUrl,
        dwAction,
        pbPolicy,
        cbPolicy,
        pbContext,
        cbContext,
        0,
        0,
    );
}
