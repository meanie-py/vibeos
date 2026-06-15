use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::string::nls::MultiByteToWideChar;
use crate::string::nls_subst::{HeapAlloc, HeapFree};
use crate::client::heapmem::GetProcessHeap;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 202 as ULONG;

#[repr(C)]
pub struct ACTCTXW {
    pub cbSize: ULONG,
    pub dwFlags: DWORD,
    pub lpSource: LPCWSTR,
    pub wProcessorArchitecture: USHORT,
    pub wLangId: LANGID,
    pub lpAssemblyDirectory: LPCWSTR,
    pub lpResourceName: LPCWSTR,
    pub lpApplicationName: LPCWSTR,
    pub hModule: HMODULE,
}

pub type PACTCTXW = *mut ACTCTXW;

#[repr(C)]
pub struct ACTCTXA {
    pub cbSize: ULONG,
    pub dwFlags: DWORD,
    pub lpSource: LPCSTR,
    pub wProcessorArchitecture: USHORT,
    pub wLangId: LANGID,
    pub lpAssemblyDirectory: LPCSTR,
    pub lpResourceName: LPCSTR,
    pub lpApplicationName: LPCSTR,
    pub hModule: HMODULE,
}

pub type PACTCTXA = *mut ACTCTXA;

pub const ACTCTX_FLAG_PROCESSOR_ARCHITECTURE_VALID: DWORD = 0x0001;
pub const ACTCTX_FLAG_LANGID_VALID: DWORD = 0x0002;
pub const ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID: DWORD = 0x0004;
pub const ACTCTX_FLAG_RESOURCE_NAME_VALID: DWORD = 0x0008;
pub const ACTCTX_FLAG_APPLICATION_NAME_VALID: DWORD = 0x0010;
pub const ACTCTX_FLAG_HMODULE_VALID: DWORD = 0x0020;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

pub const ERROR_INVALID_PARAMETER: DWORD = 87;

// GUID comes from ntdll_sys::nt (= _GUID); local shadow removed so the Rtl
// activation-context-section fns (which take *const _GUID) type-check.

#[repr(C)]
pub struct ACTCTX_SECTION_KEYED_DATA {
    pub cbSize: ULONG,
    pub ulDataFormatVersion: ULONG,
    pub lpData: PVOID,
    pub ulLength: ULONG,
    pub lpSectionGlobalData: PVOID,
    pub ulSectionGlobalDataLength: ULONG,
    pub lpSectionBase: PVOID,
    pub ulSectionTotalLength: ULONG,
    pub hActCtx: HANDLE,
    pub ulRoopCount: ULONG,
}

pub type PACTCTX_SECTION_KEYED_DATA = *mut ACTCTX_SECTION_KEYED_DATA;

#[no_mangle]
pub unsafe extern "system" fn CreateActCtxA(pActCtx: PCACTCTXA) -> HANDLE {
    let mut actw: ACTCTXW = core::mem::zeroed();
    let mut len: SIZE_T = 0;
    let mut ret: HANDLE = INVALID_HANDLE_VALUE;
    let mut src: LPWSTR = core::ptr::null_mut();
    let mut assdir: LPWSTR = core::ptr::null_mut();
    let mut resname: LPWSTR = core::ptr::null_mut();
    let mut appname: LPWSTR = core::ptr::null_mut();

    if pActCtx.is_null() || (*pActCtx).cbSize != core::mem::size_of::<ACTCTXA>() as ULONG {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    actw.cbSize = core::mem::size_of::<ACTCTXW>() as ULONG;
    actw.dwFlags = (*pActCtx).dwFlags;

    if !(*pActCtx).lpSource.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpSource, -1, core::ptr::null_mut(), 0) as SIZE_T;
        src = HeapAlloc(GetProcessHeap(), 0, len as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if src.is_null() {
            return INVALID_HANDLE_VALUE;
        }
        MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpSource, -1, src, len as i32);
    }
    actw.lpSource = src as LPCWSTR;

    if actw.dwFlags & ACTCTX_FLAG_PROCESSOR_ARCHITECTURE_VALID != 0 {
        actw.wProcessorArchitecture = (*pActCtx).wProcessorArchitecture;
    }
    if actw.dwFlags & ACTCTX_FLAG_LANGID_VALID != 0 {
        actw.wLangId = (*pActCtx).wLangId;
    }
    if actw.dwFlags & ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID != 0 {
        len = MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpAssemblyDirectory, -1, core::ptr::null_mut(), 0) as SIZE_T;
        assdir = HeapAlloc(GetProcessHeap(), 0, len as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if !assdir.is_null() {
            MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpAssemblyDirectory, -1, assdir, len as i32);
            actw.lpAssemblyDirectory = assdir as LPCWSTR;
        }
    }
    if actw.dwFlags & ACTCTX_FLAG_RESOURCE_NAME_VALID != 0 {
        if ((*pActCtx).lpResourceName as ULONG_PTR) >> 16 != 0 {
            len = MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpResourceName, -1, core::ptr::null_mut(), 0) as SIZE_T;
            resname = HeapAlloc(GetProcessHeap(), 0, len as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;
            if !resname.is_null() {
                MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpResourceName, -1, resname, len as i32);
                actw.lpResourceName = resname as LPCWSTR;
            }
        } else {
            actw.lpResourceName = (*pActCtx).lpResourceName as LPCWSTR;
        }
    }
    if actw.dwFlags & ACTCTX_FLAG_APPLICATION_NAME_VALID != 0 {
        len = MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpApplicationName, -1, core::ptr::null_mut(), 0) as SIZE_T;
        appname = HeapAlloc(GetProcessHeap(), 0, len as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if !appname.is_null() {
            MultiByteToWideChar(CP_ACP, 0, (*pActCtx).lpApplicationName, -1, appname, len as i32);
            actw.lpApplicationName = appname as LPCWSTR;
        }
    }
    if actw.dwFlags & ACTCTX_FLAG_HMODULE_VALID != 0 {
        actw.hModule = (*pActCtx).hModule;
    }

    ret = CreateActCtxW(&raw mut actw as PCACTCTXW);

    HeapFree(GetProcessHeap(), 0, src as PVOID);
    HeapFree(GetProcessHeap(), 0, assdir as PVOID);
    HeapFree(GetProcessHeap(), 0, resname as PVOID);
    HeapFree(GetProcessHeap(), 0, appname as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateActCtxW(pActCtx: PCACTCTXW) -> HANDLE {
    let mut status: NTSTATUS = 0;
    let mut hActCtx: HANDLE = core::ptr::null_mut();
    let _ = status;
    let _ = hActCtx;

    // TRACE("%p %08x\n", pActCtx, pActCtx ? pActCtx->dwFlags : 0) — debug print,
    // no-op under NDEBUG. Elided; keep a read of pActCtx to mirror the C arg eval.
    let _ = pActCtx;

    status = RtlCreateActivationContext(
        0,
        pActCtx as PACTIVATION_CONTEXT_DATA,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut hActCtx as *mut HANDLE,
    );
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
        return INVALID_HANDLE_VALUE;
    }
    hActCtx
}

#[no_mangle]
pub unsafe extern "system" fn ActivateActCtx(
    hActCtx: HANDLE,
    ulCookie: *mut ULONG_PTR,
) -> BOOL {
    let status: NTSTATUS;
    status = RtlActivateActivationContext(0, hActCtx, ulCookie);
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeactivateActCtx(
    dwFlags: DWORD,
    ulCookie: ULONG_PTR,
) -> BOOL {
    RtlDeactivateActivationContext(dwFlags, ulCookie);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentActCtx(phActCtx: *mut HANDLE) -> BOOL {
    let status: NTSTATUS;
    status = RtlGetActiveActivationContext(phActCtx);
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn AddRefActCtx(hActCtx: HANDLE) {
    RtlAddRefActivationContext(hActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseActCtx(hActCtx: HANDLE) {
    RtlReleaseActivationContext(hActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn ZombifyActCtx(hActCtx: HANDLE) -> BOOL {
    let status: NTSTATUS;
    status = RtlZombifyActivationContext(hActCtx);
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn FindActCtxSectionStringA(
    dwFlags: DWORD,
    lpExtGuid: *const GUID,
    ulId: ULONG,
    lpSearchStr: LPCSTR,
    pInfo: PACTCTX_SECTION_KEYED_DATA,
) -> BOOL {
    let mut search_str: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut ret: BOOL = 0;

    if lpSearchStr.is_null() || pInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = MultiByteToWideChar(CP_ACP, 0, lpSearchStr, -1, core::ptr::null_mut(), 0) as DWORD;
    search_str = HeapAlloc(GetProcessHeap(), 0, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
    MultiByteToWideChar(CP_ACP, 0, lpSearchStr, -1, search_str, len as i32);

    ret = FindActCtxSectionStringW(dwFlags, lpExtGuid, ulId, search_str as LPCWSTR, pInfo);

    HeapFree(GetProcessHeap(), 0, search_str as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn FindActCtxSectionStringW(
    dwFlags: DWORD,
    lpExtGuid: *const GUID,
    ulId: ULONG,
    lpSearchStr: LPCWSTR,
    pInfo: PACTCTX_SECTION_KEYED_DATA,
) -> BOOL {
    let mut us: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut status: NTSTATUS = 0;

    if pInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    RtlInitUnicodeString(&raw mut us, lpSearchStr);
    status = RtlFindActivationContextSectionString(
        dwFlags as ULONG,
        lpExtGuid,
        ulId,
        &raw mut us,
        pInfo as PVOID,
    );
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status) as DWORD);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FindActCtxSectionGuid(
    dwFlags: DWORD,
    lpExtGuid: *const GUID,
    ulId: ULONG,
    lpSearchGuid: *const GUID,
    pInfo: PACTCTX_SECTION_KEYED_DATA,
) -> BOOL {
    let mut status: NTSTATUS = 0;
    status = RtlFindActivationContextSectionGuid(
        dwFlags as ULONG,
        lpExtGuid,
        ulId,
        lpSearchGuid,
        pInfo as *mut ::core::ffi::c_void,
    );
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status) as DWORD);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn QueryActCtxW(
    dwFlags: DWORD,
    hActCtx: HANDLE,
    pvSubInst: PVOID,
    ulClass: ULONG,
    pvBuff: PVOID,
    cbBuff: SIZE_T,
    pcbLen: *mut SIZE_T,
) -> BOOL {
    let status: NTSTATUS;

    status = RtlQueryInformationActivationContext(dwFlags, hActCtx, pvSubInst, ulClass, pvBuff, cbBuff, pcbLen);
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
        return 0; // FALSE
    }
    return 1; // TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/wine/actctx.c
//  * PURPOSE:         Activation contexts
//  * PROGRAMMERS:     Jacek Caban for CodeWeavers
//  *                  Eric Pouech
//  *                  Jon Griffiths
//  *                  Dmitry Chapyshev (dmitry@reactos.org)
//  *                  Samuel Serapi�n
//  */
//
// /* Synched with Wine 1.7.55 */
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(actctx);
//
// /***********************************************************************
//  * CreateActCtxA (KERNEL32.@)
//  *
//  * Create an activation context.
//  */
// HANDLE WINAPI CreateActCtxA(PCACTCTXA pActCtx)
// {
//     ACTCTXW     actw;
//     SIZE_T      len;
//     HANDLE      ret = INVALID_HANDLE_VALUE;
//     LPWSTR      src = NULL, assdir = NULL, resname = NULL, appname = NULL;
//
//     TRACE("%p %08x\n", pActCtx, pActCtx ? pActCtx->dwFlags : 0);
//
//     if (!pActCtx || pActCtx->cbSize != sizeof(*pActCtx))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     actw.cbSize = sizeof(actw);
//     actw.dwFlags = pActCtx->dwFlags;
//     if (pActCtx->lpSource)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, pActCtx->lpSource, -1, NULL, 0);
//         src = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!src) return INVALID_HANDLE_VALUE;
//         MultiByteToWideChar(CP_ACP, 0, pActCtx->lpSource, -1, src, len);
//     }
//     actw.lpSource = src;
//
//     if (actw.dwFlags & ACTCTX_FLAG_PROCESSOR_ARCHITECTURE_VALID)
//         actw.wProcessorArchitecture = pActCtx->wProcessorArchitecture;
//     if (actw.dwFlags & ACTCTX_FLAG_LANGID_VALID)
//         actw.wLangId = pActCtx->wLangId;
//     if (actw.dwFlags & ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, pActCtx->lpAssemblyDirectory, -1, NULL, 0);
//         assdir = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!assdir) goto done;
//         MultiByteToWideChar(CP_ACP, 0, pActCtx->lpAssemblyDirectory, -1, assdir, len);
//         actw.lpAssemblyDirectory = assdir;
//     }
//     if (actw.dwFlags & ACTCTX_FLAG_RESOURCE_NAME_VALID)
//     {
//         if ((ULONG_PTR)pActCtx->lpResourceName >> 16)
//         {
//             len = MultiByteToWideChar(CP_ACP, 0, pActCtx->lpResourceName, -1, NULL, 0);
//             resname = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//             if (!resname) goto done;
//             MultiByteToWideChar(CP_ACP, 0, pActCtx->lpResourceName, -1, resname, len);
//             actw.lpResourceName = resname;
//         }
//         else actw.lpResourceName = (LPCWSTR)pActCtx->lpResourceName;
//     }
//     if (actw.dwFlags & ACTCTX_FLAG_APPLICATION_NAME_VALID)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, pActCtx->lpApplicationName, -1, NULL, 0);
//         appname = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!appname) goto done;
//         MultiByteToWideChar(CP_ACP, 0, pActCtx->lpApplicationName, -1, appname, len);
//         actw.lpApplicationName = appname;
//     }
//     if (actw.dwFlags & ACTCTX_FLAG_HMODULE_VALID)
//         actw.hModule = pActCtx->hModule;
//
//     ret = CreateActCtxW(&actw);
//
// done:
//     HeapFree(GetProcessHeap(), 0, src);
//     HeapFree(GetProcessHeap(), 0, assdir);
//     HeapFree(GetProcessHeap(), 0, resname);
//     HeapFree(GetProcessHeap(), 0, appname);
//     return ret;
// }
//
// /***********************************************************************
//  * CreateActCtxW (KERNEL32.@)
//  *
//  * Create an activation context.
//  */
// HANDLE WINAPI CreateActCtxW(PCACTCTXW pActCtx)
// {
//     NTSTATUS    status;
//     HANDLE      hActCtx;
//
//     TRACE("%p %08x\n", pActCtx, pActCtx ? pActCtx->dwFlags : 0);
//
//     if ((status = RtlCreateActivationContext(0, (PVOID)pActCtx, 0, NULL, NULL, &hActCtx)))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return INVALID_HANDLE_VALUE;
//     }
//     return hActCtx;
// }
//
// #ifndef __REACTOS__
// /***********************************************************************
//  * ActivateActCtx (KERNEL32.@)
//  *
//  * Activate an activation context.
//  */
// BOOL WINAPI ActivateActCtx(HANDLE hActCtx, ULONG_PTR *ulCookie)
// {
//     NTSTATUS status;
//
//     if ((status = RtlActivateActivationContext( 0, hActCtx, ulCookie )))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /***********************************************************************
//  * DeactivateActCtx (KERNEL32.@)
//  *
//  * Deactivate an activation context.
//  */
// BOOL WINAPI DeactivateActCtx(DWORD dwFlags, ULONG_PTR ulCookie)
// {
//     RtlDeactivateActivationContext( dwFlags, ulCookie );
//     return TRUE;
// }
//
// /***********************************************************************
//  * GetCurrentActCtx (KERNEL32.@)
//  *
//  * Get the current activation context.
//  */
// BOOL WINAPI GetCurrentActCtx(HANDLE* phActCtx)
// {
//     NTSTATUS status;
//
//     if ((status = RtlGetActiveActivationContext(phActCtx)))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /***********************************************************************
//  * AddRefActCtx (KERNEL32.@)
//  *
//  * Add a reference to an activation context.
//  */
// void WINAPI AddRefActCtx(HANDLE hActCtx)
// {
//     RtlAddRefActivationContext(hActCtx);
// }
//
// /***********************************************************************
//  * ReleaseActCtx (KERNEL32.@)
//  *
//  * Release a reference to an activation context.
//  */
// void WINAPI ReleaseActCtx(HANDLE hActCtx)
// {
//     RtlReleaseActivationContext(hActCtx);
// }
//
// /***********************************************************************
//  * ZombifyActCtx (KERNEL32.@)
//  *
//  * Deactivate context without releasing it.
//  */
// BOOL WINAPI ZombifyActCtx(HANDLE hActCtx)
// {
//     NTSTATUS status;
//
//     if ((status = RtlZombifyActivationContext(hActCtx)))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//     return TRUE;
// }
// #endif // !__REACTOS__
//
// /***********************************************************************
//  * FindActCtxSectionStringA (KERNEL32.@)
//  *
//  * Find information about a string in an activation context.
//  */
// BOOL WINAPI FindActCtxSectionStringA(DWORD dwFlags, const GUID* lpExtGuid,
//                                     ULONG ulId, LPCSTR lpSearchStr,
//                                     PACTCTX_SECTION_KEYED_DATA pInfo)
// {
//     LPWSTR  search_str;
//     DWORD   len;
//     BOOL    ret;
//
//     TRACE("%08x %s %u %s %p\n", dwFlags, debugstr_guid(lpExtGuid),
//           ulId, debugstr_a(lpSearchStr), pInfo);
//
//     if (!lpSearchStr || !pInfo)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     len = MultiByteToWideChar(CP_ACP, 0, lpSearchStr, -1, NULL, 0);
//     search_str = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     MultiByteToWideChar(CP_ACP, 0, lpSearchStr, -1, search_str, len);
//
//     ret = FindActCtxSectionStringW(dwFlags, lpExtGuid, ulId, search_str, pInfo);
//
//     HeapFree(GetProcessHeap(), 0, search_str);
//     return ret;
// }
//
// /***********************************************************************
//  * FindActCtxSectionStringW (KERNEL32.@)
//  *
//  * Find information about a string in an activation context.
//  */
// BOOL WINAPI FindActCtxSectionStringW(DWORD dwFlags, const GUID* lpExtGuid,
//                                     ULONG ulId, LPCWSTR lpSearchStr,
//                                     PACTCTX_SECTION_KEYED_DATA pInfo)
// {
//     UNICODE_STRING us;
//     NTSTATUS status;
//
//     if (!pInfo)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&us, lpSearchStr);
//     if ((status = RtlFindActivationContextSectionString(dwFlags, lpExtGuid, ulId, &us, pInfo)))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /***********************************************************************
//  * FindActCtxSectionGuid (KERNEL32.@)
//  *
//  * Find information about a GUID in an activation context.
//  */
// BOOL WINAPI FindActCtxSectionGuid(DWORD dwFlags, const GUID* lpExtGuid,
//                                   ULONG ulId, const GUID* lpSearchGuid,
//                                   PACTCTX_SECTION_KEYED_DATA pInfo)
// {
//     NTSTATUS status;
//
//     if ((status = RtlFindActivationContextSectionGuid(dwFlags, lpExtGuid, ulId, lpSearchGuid, pInfo)))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// #ifndef __REACTOS__
// /***********************************************************************
//  * QueryActCtxW (KERNEL32.@)
//  *
//  * Get information about an activation context.
//  */
// BOOL WINAPI QueryActCtxW(DWORD dwFlags, HANDLE hActCtx, PVOID pvSubInst,
//                          ULONG ulClass, PVOID pvBuff, SIZE_T cbBuff,
//                          SIZE_T *pcbLen)
// {
//     NTSTATUS status;
//
//     if ((status = RtlQueryInformationActivationContext( dwFlags, hActCtx, pvSubInst, ulClass,
//                                                         pvBuff, cbBuff, pcbLen )))
//     {
//         SetLastError(RtlNtStatusToDosError(status));
//         return FALSE;
//     }
//     return TRUE;
// }
// #endif // !__REACTOS__
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
