//! SCM service-configuration interface — Rust flip of
//! base/system/services/config.c (SCM wave 6).
//!
//! Registry read/write of a service's config (Open/CreateServiceKey,
//! dependencies, security descriptor, delete-flag), plus the LSA-backed
//! service-account password storage (ScmSetServicePassword) and the
//! SystemFunction005/028 password decrypt (ScmDecryptPassword). All exports are
//! cdecl (services.h internal — no WINAPI), consumed by database.c/rpcserver.c.

#![allow(non_snake_case)]

use crate::types::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::nt::{RtlGetProcessHeap, STATUS_BUFFER_TOO_SMALL};

// CRT wide-string helpers (msvcrt) — not in winsrv-sys bindgen.
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
}

// ntsecapi.h / winsvc.h constants absent from winsrv-sys.
const SC_GROUP_IDENTIFIERW: u16 = b'+' as u16; // L'+'
const POLICY_CREATE_SECRET: ACCESS_MASK = 0x0000_0020;

/// ustring — the raw-counted blob struct SystemFunction005/028 marshal.
#[repr(C)]
struct ustring {
    Length: DWORD,
    MaximumLength: DWORD,
    Buffer: *mut u8,
}

// advapi32 / kernel32 WINAPI imports not carried by the winsrv-sys surface.
// All stdcall on i386 (extern "system"); LSA + cng crypto + env-expand.
extern "system" {
    fn SystemFunction005(input: *const ustring, key: *const ustring, output: *mut ustring) -> NTSTATUS;
    fn SystemFunction028(ContextHandle: PVOID, SessionKey: *mut u8) -> NTSTATUS;
    fn LsaOpenPolicy(
        SystemName: *mut UNICODE_STRING,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: *mut HANDLE,
    ) -> NTSTATUS;
    fn LsaStorePrivateData(
        PolicyHandle: HANDLE,
        KeyName: *mut UNICODE_STRING,
        PrivateData: *mut UNICODE_STRING,
    ) -> NTSTATUS;
    fn LsaClose(ObjectHandle: HANDLE) -> NTSTATUS;
    fn ExpandEnvironmentStringsW(lpSrc: *const u16, lpDst: *mut u16, nSize: DWORD) -> DWORD;
}

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

// L"System\\CurrentControlSet\\Services" — NUL-terminated.
static SERVICES_KEY: [u16; 33] = [
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
    b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16,
    b'c' as u16, b'e' as u16, 0,
];
// L"DependOnService" / L"DependOnGroup"
static VAL_DEPEND_SERVICE: [u16; 16] = [
    b'D' as u16, b'e' as u16, b'p' as u16, b'e' as u16, b'n' as u16, b'd' as u16,
    b'O' as u16, b'n' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16,
    b'i' as u16, b'c' as u16, b'e' as u16, 0,
];
static VAL_DEPEND_GROUP: [u16; 14] = [
    b'D' as u16, b'e' as u16, b'p' as u16, b'e' as u16, b'n' as u16, b'd' as u16,
    b'O' as u16, b'n' as u16, b'G' as u16, b'r' as u16, b'o' as u16, b'u' as u16,
    b'p' as u16, 0,
];
// L"DeleteFlag"
static VAL_DELETE_FLAG: [u16; 11] = [
    b'D' as u16, b'e' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'e' as u16,
    b'F' as u16, b'l' as u16, b'a' as u16, b'g' as u16, 0,
];
// L"Security"
static VAL_SECURITY: [u16; 9] = [
    b'S' as u16, b'e' as u16, b'c' as u16, b'u' as u16, b'r' as u16, b'i' as u16,
    b't' as u16, b'y' as u16, 0,
];
// L"_SC_"
static PREFIX_SC: [u16; 5] = [b'_' as u16, b'S' as u16, b'C' as u16, b'_' as u16, 0];

#[no_mangle]
pub unsafe extern "C" fn ScmOpenServiceKey(
    lpServiceName: LPWSTR,
    samDesired: REGSAM,
    phKey: PHKEY,
) -> DWORD {
    let mut hServicesKey: HKEY = core::ptr::null_mut();

    core::ptr::write_volatile(phKey, core::ptr::null_mut());

    let mut dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        SERVICES_KEY.as_ptr(),
        0,
        KEY_READ,
        &mut hServicesKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwError = RegOpenKeyExW(hServicesKey, lpServiceName, 0, samDesired, phKey) as DWORD;

    RegCloseKey(hServicesKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmCreateServiceKey(
    lpServiceName: LPCWSTR,
    samDesired: REGSAM,
    phKey: PHKEY,
) -> DWORD {
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;

    core::ptr::write_volatile(phKey, core::ptr::null_mut());

    let mut dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        SERVICES_KEY.as_ptr(),
        0,
        KEY_READ | KEY_CREATE_SUB_KEY,
        &mut hServicesKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwError = RegCreateKeyExW(
        hServicesKey,
        lpServiceName,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        samDesired,
        core::ptr::null_mut(),
        phKey,
        &mut dwDisposition,
    ) as DWORD;

    RegCloseKey(hServicesKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmWriteDependencies(
    hServiceKey: HKEY,
    lpDependencies: LPCWSTR,
    dwDependenciesLength: DWORD,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut cchGroupLength: usize = 0;
    let mut cchServiceLength: usize = 0;
    let mut cchLength: usize;

    if *lpDependencies == 0 {
        RegDeleteValueW(hServiceKey, VAL_DEPEND_SERVICE.as_ptr());
        RegDeleteValueW(hServiceKey, VAL_DEPEND_GROUP.as_ptr());
    } else {
        let lpGroupDeps = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((dwDependenciesLength as usize + 2) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut u16;
        if lpGroupDeps.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        let mut lpSrc: *const u16 = lpDependencies;
        let mut lpDst: *mut u16 = lpGroupDeps;
        while *lpSrc != 0 {
            cchLength = wcslen(lpSrc) + 1;
            if *lpSrc == SC_GROUP_IDENTIFIERW {
                lpSrc = lpSrc.add(1);
                cchLength -= 1;
                cchGroupLength += cchLength;
                wcscpy(lpDst, lpSrc);
                lpDst = lpDst.add(cchLength);
            }
            lpSrc = lpSrc.add(cchLength);
        }
        *lpDst = 0;
        lpDst = lpDst.add(1);
        cchGroupLength += 1;

        lpSrc = lpDependencies;
        let lpServiceDeps: *mut u16 = lpDst;
        while *lpSrc != 0 {
            cchLength = wcslen(lpSrc) + 1;
            if *lpSrc != SC_GROUP_IDENTIFIERW {
                cchServiceLength += cchLength;
                wcscpy(lpDst, lpSrc);
                lpDst = lpDst.add(cchLength);
            }
            lpSrc = lpSrc.add(cchLength);
        }
        *lpDst = 0;
        cchServiceLength += 1;

        if cchGroupLength > 1 {
            dwError = RegSetValueExW(
                hServiceKey,
                VAL_DEPEND_GROUP.as_ptr(),
                0,
                REG_MULTI_SZ,
                lpGroupDeps as *const u8,
                (cchGroupLength * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
        } else {
            RegDeleteValueW(hServiceKey, VAL_DEPEND_GROUP.as_ptr());
        }

        if dwError == ERROR_SUCCESS {
            if cchServiceLength > 1 {
                dwError = RegSetValueExW(
                    hServiceKey,
                    VAL_DEPEND_SERVICE.as_ptr(),
                    0,
                    REG_MULTI_SZ,
                    lpServiceDeps as *const u8,
                    (cchServiceLength * core::mem::size_of::<u16>()) as DWORD,
                ) as DWORD;
            } else {
                RegDeleteValueW(hServiceKey, VAL_DEPEND_SERVICE.as_ptr());
            }
        }

        HeapFree(GetProcessHeap(), 0, lpGroupDeps as PVOID);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmMarkServiceForDelete(pService: PSERVICE) -> DWORD {
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let dwValue: DWORD = 1;

    let dwError = ScmOpenServiceKey((*pService).lpServiceName, KEY_WRITE, &mut hServiceKey);
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    let dwError = RegSetValueExW(
        hServiceKey,
        VAL_DELETE_FLAG.as_ptr(),
        0,
        REG_DWORD,
        &dwValue as *const DWORD as *const u8,
        core::mem::size_of::<DWORD>() as DWORD,
    ) as DWORD;

    RegCloseKey(hServiceKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmIsDeleteFlagSet(hServiceKey: HKEY) -> BOOL {
    let mut dwType: DWORD = 0;
    let mut dwFlag: DWORD = 0;
    let mut dwSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    let dwError = RegQueryValueExW(
        hServiceKey,
        VAL_DELETE_FLAG.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwFlag as *mut DWORD as LPBYTE,
        &mut dwSize,
    ) as DWORD;

    (dwError == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn ScmReadString(
    hServiceKey: HKEY,
    lpValueName: LPCWSTR,
    lpValue: *mut LPWSTR,
) -> DWORD {
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;

    core::ptr::write_volatile(lpValue, core::ptr::null_mut());

    let mut dwError = RegQueryValueExW(
        hServiceKey,
        lpValueName,
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    let ptr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize as SIZE_T) as *mut u16;
    if ptr.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    dwError = RegQueryValueExW(
        hServiceKey,
        lpValueName,
        core::ptr::null_mut(),
        &mut dwType,
        ptr as LPBYTE,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        HeapFree(GetProcessHeap(), 0, ptr as PVOID);
        return dwError;
    }

    if dwType == REG_EXPAND_SZ {
        /* Expand the value... */
        dwSize = ExpandEnvironmentStringsW(ptr, core::ptr::null_mut(), 0);
        if dwSize > 0 {
            let expanded = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (dwSize as usize * core::mem::size_of::<u16>()) as SIZE_T,
            ) as *mut u16;
            if !expanded.is_null() {
                if dwSize == ExpandEnvironmentStringsW(ptr, expanded, dwSize) {
                    core::ptr::write_volatile(lpValue, expanded);
                    dwError = ERROR_SUCCESS;
                } else {
                    dwError = GetLastError();
                    HeapFree(GetProcessHeap(), 0, expanded as PVOID);
                }
            } else {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
            }
        } else {
            dwError = GetLastError();
        }

        HeapFree(GetProcessHeap(), 0, ptr as PVOID);
    } else {
        core::ptr::write_volatile(lpValue, ptr);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmReadDependencies(
    hServiceKey: HKEY,
    lpDependencies: *mut LPWSTR,
    lpdwDependenciesLength: *mut DWORD,
) -> DWORD {
    let mut lpGroups: LPWSTR = core::ptr::null_mut();
    let mut lpServices: LPWSTR = core::ptr::null_mut();
    let mut cchGroupsLength: usize = 0;
    let mut cchServicesLength: usize = 0;
    let mut lpSrc: *const u16;
    let mut lpDest: *mut u16;
    let mut cchLength: usize;

    core::ptr::write_volatile(lpDependencies, core::ptr::null_mut());
    core::ptr::write_volatile(lpdwDependenciesLength, 0);

    /* Read the dependency values */
    ScmReadString(hServiceKey, VAL_DEPEND_GROUP.as_ptr(), &mut lpGroups);
    ScmReadString(hServiceKey, VAL_DEPEND_SERVICE.as_ptr(), &mut lpServices);

    /* Leave, if there are no dependencies */
    if lpGroups.is_null() && lpServices.is_null() {
        return ERROR_SUCCESS;
    }

    /* Determine the total buffer size for the dependencies */
    if !lpGroups.is_null() {
        lpSrc = lpGroups;
        while *lpSrc != 0 {
            cchLength = wcslen(lpSrc) + 1;
            cchGroupsLength += cchLength + 1;
            lpSrc = lpSrc.add(cchLength);
        }
    }

    if !lpServices.is_null() {
        lpSrc = lpServices;
        while *lpSrc != 0 {
            cchLength = wcslen(lpSrc) + 1;
            cchServicesLength += cchLength;
            lpSrc = lpSrc.add(cchLength);
        }
    }

    let cchTotalLength = cchGroupsLength + cchServicesLength + 1;

    /* Allocate the common buffer for the dependencies */
    let deps = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (cchTotalLength * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    core::ptr::write_volatile(lpDependencies, deps);
    if deps.is_null() {
        if !lpGroups.is_null() {
            HeapFree(GetProcessHeap(), 0, lpGroups as PVOID);
        }
        if !lpServices.is_null() {
            HeapFree(GetProcessHeap(), 0, lpServices as PVOID);
        }
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    /* Return the allocated buffer length in characters */
    core::ptr::write_volatile(lpdwDependenciesLength, cchTotalLength as DWORD);

    /* Copy the service dependencies into the common buffer */
    lpDest = deps;
    if !lpServices.is_null() {
        core::ptr::copy_nonoverlapping(
            lpServices as *const u8,
            lpDest as *mut u8,
            cchServicesLength * core::mem::size_of::<u16>(),
        );
        lpDest = lpDest.add(cchServicesLength);
    }

    /* Copy the group dependencies into the common buffer */
    if !lpGroups.is_null() {
        lpSrc = lpGroups;
        while *lpSrc != 0 {
            cchLength = wcslen(lpSrc) + 1;

            *lpDest = SC_GROUP_IDENTIFIERW;
            lpDest = lpDest.add(1);

            wcscpy(lpDest, lpSrc);

            lpDest = lpDest.add(cchLength);
            lpSrc = lpSrc.add(cchLength);
        }
    }

    /* Free the temporary buffers */
    if !lpGroups.is_null() {
        HeapFree(GetProcessHeap(), 0, lpGroups as PVOID);
    }
    if !lpServices.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServices as PVOID);
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ScmSetServicePassword(
    pszServiceName: PCWSTR,
    pszPassword: PCWSTR,
) -> DWORD {
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: HANDLE = core::ptr::null_mut();
    let mut ServiceName: UNICODE_STRING = core::mem::zeroed();
    let mut Password: UNICODE_STRING = core::mem::zeroed();
    let mut dwError: DWORD = ERROR_SUCCESS;

    let ServiceNameLength = wcslen(pszServiceName);
    if ServiceNameLength > (UNICODE_STRING_MAX_CHARS as usize - 4) {
        return ERROR_INVALID_PARAMETER;
    }

    let Status = LsaOpenPolicy(
        core::ptr::null_mut(),
        &mut ObjectAttributes,
        POLICY_CREATE_SECRET,
        &mut PolicyHandle,
    );
    if !nt_success(Status) {
        return RtlNtStatusToDosError(Status);
    }

    ServiceName.Length = ((ServiceNameLength as USHORT + 4) * core::mem::size_of::<u16>() as USHORT) as USHORT;
    ServiceName.MaximumLength = ServiceName.Length + core::mem::size_of::<u16>() as USHORT;
    ServiceName.Buffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ServiceName.MaximumLength as SIZE_T,
    ) as *mut u16;
    if ServiceName.Buffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy(ServiceName.Buffer, PREFIX_SC.as_ptr());
    wcscat(ServiceName.Buffer, pszServiceName);

    RtlInitUnicodeString(&mut Password, pszPassword);

    let Status = LsaStorePrivateData(
        PolicyHandle,
        &mut ServiceName,
        if !pszPassword.is_null() {
            &mut Password
        } else {
            core::ptr::null_mut()
        },
    );
    if !nt_success(Status) {
        dwError = RtlNtStatusToDosError(Status);
    }

    // done:
    if !ServiceName.Buffer.is_null() {
        HeapFree(GetProcessHeap(), 0, ServiceName.Buffer as PVOID);
    }
    if !PolicyHandle.is_null() {
        LsaClose(PolicyHandle);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmWriteSecurityDescriptor(
    hServiceKey: HKEY,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut hSecurityKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;

    let dwError = RegCreateKeyExW(
        hServiceKey,
        VAL_SECURITY.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        &mut hSecurityKey,
        &mut dwDisposition,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    let dwError = RegSetValueExW(
        hSecurityKey,
        VAL_SECURITY.as_ptr(),
        0,
        REG_BINARY,
        pSecurityDescriptor as *const u8,
        RtlLengthSecurityDescriptor(pSecurityDescriptor),
    ) as DWORD;

    RegCloseKey(hSecurityKey);

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmReadSecurityDescriptor(
    hServiceKey: HKEY,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut pRelativeSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut hSecurityKey: HKEY = core::ptr::null_mut();
    let mut dwBufferLength: DWORD = 0;
    let mut dwType: DWORD = 0;

    core::ptr::write_volatile(ppSecurityDescriptor, core::ptr::null_mut());

    let mut dwError = RegOpenKeyExW(
        hServiceKey,
        VAL_SECURITY.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hSecurityKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        /* Do not fail if the Security key does not exist */
        if dwError == ERROR_FILE_NOT_FOUND {
            dwError = ERROR_SUCCESS;
        }
        // goto done
        if hSecurityKey.is_null() {
            return dwError;
        }
        RegCloseKey(hSecurityKey);
        return dwError;
    }

    dwError = RegQueryValueExW(
        hSecurityKey,
        VAL_SECURITY.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut dwBufferLength,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        if dwError == ERROR_FILE_NOT_FOUND {
            dwError = ERROR_SUCCESS;
        }
        RegCloseKey(hSecurityKey);
        return dwError;
    }

    pRelativeSD = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, dwBufferLength as SIZE_T);
    if pRelativeSD.is_null() {
        // C returns directly here (leaks hSecurityKey) — preserve behaviour.
        return ERROR_OUTOFMEMORY;
    }

    dwError = RegQueryValueExW(
        hSecurityKey,
        VAL_SECURITY.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        pRelativeSD as LPBYTE,
        &mut dwBufferLength,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        // goto done
        if !pRelativeSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pRelativeSD);
        }
        RegCloseKey(hSecurityKey);
        return dwError;
    }

    core::ptr::write_volatile(ppSecurityDescriptor, pRelativeSD);

    // done:
    if dwError != ERROR_SUCCESS && !pRelativeSD.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, pRelativeSD);
    }
    if !hSecurityKey.is_null() {
        RegCloseKey(hSecurityKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmDeleteRegKey(hKey: HKEY, pszSubKey: PCWSTR) -> DWORD {
    let mut dwMaxSubkeyLen: DWORD = 0;
    let mut dwMaxValueLen: DWORD = 0;
    let mut dwSize: DWORD;
    let mut pszName: PWSTR = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();

    let mut dwError = RegOpenKeyExW(hKey, pszSubKey, 0, KEY_READ, &mut hSubKey) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Get maximum length of key and value names */
    dwError = RegQueryInfoKeyW(
        hSubKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwMaxSubkeyLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwMaxValueLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        // goto done
        RegCloseKey(hSubKey);
        if dwError == ERROR_SUCCESS {
            dwError = RegDeleteKeyW(hKey, pszSubKey) as DWORD;
        }
        return dwError;
    }

    dwMaxSubkeyLen += 1;
    dwMaxValueLen += 1;
    let dwMaxLen = core::cmp::max(dwMaxSubkeyLen, dwMaxValueLen);

    /* Allocate the name buffer */
    pszName = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwMaxLen as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if pszName.is_null() {
        dwError = ERROR_NOT_ENOUGH_MEMORY;
        // goto done
        RegCloseKey(hSubKey);
        if dwError == ERROR_SUCCESS {
            dwError = RegDeleteKeyW(hKey, pszSubKey) as DWORD;
        }
        return dwError;
    }

    /* Recursively delete all the subkeys */
    loop {
        dwSize = dwMaxLen;
        if RegEnumKeyExW(
            hSubKey,
            0,
            pszName,
            &mut dwSize,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD
            != ERROR_SUCCESS
        {
            break;
        }

        dwError = ScmDeleteRegKey(hSubKey, pszName);
        if dwError != ERROR_SUCCESS {
            // goto done
            if !pszName.is_null() {
                HeapFree(GetProcessHeap(), 0, pszName as PVOID);
            }
            RegCloseKey(hSubKey);
            if dwError == ERROR_SUCCESS {
                dwError = RegDeleteKeyW(hKey, pszSubKey) as DWORD;
            }
            return dwError;
        }
    }

    // done:
    if !pszName.is_null() {
        HeapFree(GetProcessHeap(), 0, pszName as PVOID);
    }
    RegCloseKey(hSubKey);

    /* Finally delete the key */
    if dwError == ERROR_SUCCESS {
        dwError = RegDeleteKeyW(hKey, pszSubKey) as DWORD;
    }

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmDecryptPassword(
    ContextHandle: PVOID,
    pPassword: PBYTE,
    dwPasswordSize: DWORD,
    pClearTextPassword: *mut PWSTR,
) -> DWORD {
    let mut inData: ustring = core::mem::zeroed();
    let mut keyData: ustring = core::mem::zeroed();
    let mut outData: ustring = core::mem::zeroed();
    let mut SessionKey: [u8; 16] = [0; 16];

    /* Get the session key */
    let Status = SystemFunction028(ContextHandle, SessionKey.as_mut_ptr());
    if !nt_success(Status) {
        return RtlNtStatusToDosError(Status);
    }

    inData.Length = dwPasswordSize;
    inData.MaximumLength = inData.Length;
    inData.Buffer = pPassword;

    keyData.Length = core::mem::size_of_val(&SessionKey) as DWORD;
    keyData.MaximumLength = keyData.Length;
    keyData.Buffer = SessionKey.as_mut_ptr();

    outData.Length = 0;
    outData.MaximumLength = 0;
    outData.Buffer = core::ptr::null_mut();

    /* Get the required buffer size */
    let Status = SystemFunction005(&inData, &keyData, &mut outData);
    if Status != STATUS_BUFFER_TOO_SMALL {
        return RtlNtStatusToDosError(Status);
    }

    /* Allocate a buffer for the clear text password */
    let pBuffer = HeapAlloc(GetProcessHeap(), 0, outData.Length as SIZE_T) as *mut u16;
    if pBuffer.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    outData.MaximumLength = outData.Length;
    outData.Buffer = pBuffer as *mut u8;

    /* Decrypt the password */
    let Status = SystemFunction005(&inData, &keyData, &mut outData);
    if !nt_success(Status) {
        HeapFree(GetProcessHeap(), 0, pBuffer as PVOID);
        return RtlNtStatusToDosError(Status);
    }

    core::ptr::write_volatile(pClearTextPassword, pBuffer);

    ERROR_SUCCESS
}
