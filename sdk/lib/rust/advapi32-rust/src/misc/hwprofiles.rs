use crate::prelude::*;
use core::ffi::c_void;

// WINAPI imports (Reg*/UuidCreate/SetLastError) come from crate::prelude (extern
// "system" — correct i386 stdcall decoration). Only the CRT cdecl helpers stay
// local here.
extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
}

// ---- file-scope preamble (translated) ----
pub const WINE_DEFAULT_DEBUG_CHANNEL: &str = "advapi";

#[no_mangle]
pub unsafe extern "system" fn GetCurrentHwProfileA(lpHwProfileInfo: LPHW_PROFILE_INFOA) -> BOOL {
    let mut ProfileInfo: HW_PROFILE_INFOW = core::mem::zeroed();
    let mut StringU: _UNICODE_STRING = core::mem::zeroed();
    let mut StringA: ANSI_STRING = core::mem::zeroed();
    let bResult: BOOL;
    let mut Status: NTSTATUS;

    bResult = GetCurrentHwProfileW(&mut ProfileInfo);
    if bResult == 0 {
        return 0;
    }

    (*lpHwProfileInfo).dwDockInfo = ProfileInfo.dwDockInfo;

    StringU.Buffer = ProfileInfo.szHwProfileGuid.as_mut_ptr();
    StringU.Length = (wcslen(ProfileInfo.szHwProfileGuid.as_ptr()) as USHORT)
        .wrapping_mul(core::mem::size_of::<u16>() as USHORT);
    StringU.MaximumLength = (HW_PROFILE_GUIDLEN as USHORT)
        .wrapping_mul(core::mem::size_of::<u16>() as USHORT);
    StringA.Buffer = core::ptr::addr_of_mut!((*lpHwProfileInfo).szHwProfileGuid) as PCHAR;
    StringA.Length = 0;
    StringA.MaximumLength = HW_PROFILE_GUIDLEN as USHORT;
    Status = RtlUnicodeStringToAnsiString(&mut StringA, &mut StringU, 0);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    StringU.Buffer = ProfileInfo.szHwProfileName.as_mut_ptr();
    StringU.Length = (wcslen(ProfileInfo.szHwProfileName.as_ptr()) as USHORT)
        .wrapping_mul(core::mem::size_of::<u16>() as USHORT);
    StringU.MaximumLength = (MAX_PROFILE_LEN as USHORT)
        .wrapping_mul(core::mem::size_of::<u16>() as USHORT);
    StringA.Buffer = core::ptr::addr_of_mut!((*lpHwProfileInfo).szHwProfileName) as PCHAR;
    StringA.Length = 0;
    StringA.MaximumLength = MAX_PROFILE_LEN as USHORT;
    Status = RtlUnicodeStringToAnsiString(&mut StringA, &mut StringU, 0);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentHwProfileW(lpHwProfileInfo: LPHW_PROFILE_INFOW) -> BOOL {

    let mut szKeyName: [u16; 256] = [0; 256];
    let mut hDbKey: HKEY = core::ptr::null_mut();
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwLength: DWORD;
    let mut dwConfigId: DWORD = 0;
    let mut uuid: UUID = core::mem::zeroed();

    TRACE!(b"GetCurrentHwProfileW() called\n\0".as_ptr() as *const i8);

    if lpHwProfileInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let idconfig: &[u16] = &{
        let s = "System\\CurrentControlSet\\Control\\IDConfigDB\0";
        let mut v = [0u16; 64];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        idconfig.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hDbKey,
    ) != 0
    {
        SetLastError(ERROR_REGISTRY_CORRUPT);
        return 0;
    }

    let current_config: &[u16] = &{
        let s = "CurrentConfig\0";
        let mut v = [0u16; 16];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(
        hDbKey,
        current_config.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwConfigId as *mut DWORD as LPBYTE,
        &mut dwLength,
    ) != 0
    {
        RegCloseKey(hDbKey);
        SetLastError(ERROR_REGISTRY_CORRUPT);
        return 0;
    }

    let fmt_key: &[u16] = &{
        let s = "Hardware Profile\\%04lu\0";
        let mut v = [0u16; 32];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    swprintf(szKeyName.as_mut_ptr(), fmt_key.as_ptr(), dwConfigId);

    if RegOpenKeyExW(
        hDbKey,
        szKeyName.as_ptr(),
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hProfileKey,
    ) != 0
    {
        RegCloseKey(hDbKey);
        SetLastError(ERROR_REGISTRY_CORRUPT);
        return 0;
    }

    let dockstate: &[u16] = &{
        let s = "DockState\0";
        let mut v = [0u16; 16];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    dwLength = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(
        hProfileKey,
        dockstate.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*lpHwProfileInfo).dwDockInfo) as LPBYTE,
        &mut dwLength,
    ) != 0
    {
        (*lpHwProfileInfo).dwDockInfo =
            DOCKINFO_DOCKED | DOCKINFO_UNDOCKED | DOCKINFO_USER_SUPPLIED;
    }

    let hwprofileguid: &[u16] = &{
        let s = "HwProfileGuid\0";
        let mut v = [0u16; 16];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    dwLength = HW_PROFILE_GUIDLEN * core::mem::size_of::<u16>() as DWORD;
    if RegQueryValueExW(
        hProfileKey,
        hwprofileguid.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*lpHwProfileInfo).szHwProfileGuid) as LPBYTE,
        &mut dwLength,
    ) != 0
    {
        UuidCreate(&mut uuid);

        let fmt_guid: &[u16] = &{
            let s = "{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}\0";
            let mut v = [0u16; 64];
            let mut i = 0;
            for c in s.encode_utf16() {
                v[i] = c;
                i += 1;
            }
            v
        };

        swprintf(
            (*lpHwProfileInfo).szHwProfileGuid.as_mut_ptr(),
            fmt_guid.as_ptr(),
            uuid.Data1,
            uuid.Data2 as u32,
            uuid.Data3 as u32,
            uuid.Data4[0] as u32,
            uuid.Data4[1] as u32,
            uuid.Data4[2] as u32,
            uuid.Data4[3] as u32,
            uuid.Data4[4] as u32,
            uuid.Data4[5] as u32,
            uuid.Data4[6] as u32,
            uuid.Data4[7] as u32,
        );

        dwLength =
            (wcslen((*lpHwProfileInfo).szHwProfileGuid.as_ptr()) as DWORD + 1)
                * core::mem::size_of::<u16>() as DWORD;
        RegSetValueExW(
            hProfileKey,
            hwprofileguid.as_ptr(),
            0,
            REG_SZ,
            (*lpHwProfileInfo).szHwProfileGuid.as_mut_ptr() as LPBYTE,
            dwLength,
        );
    }

    let friendlyname: &[u16] = &{
        let s = "FriendlyName\0";
        let mut v = [0u16; 16];
        let mut i = 0;
        for c in s.encode_utf16() {
            v[i] = c;
            i += 1;
        }
        v
    };

    dwLength = MAX_PROFILE_LEN * core::mem::size_of::<u16>() as DWORD;
    if RegQueryValueExW(
        hProfileKey,
        friendlyname.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*lpHwProfileInfo).szHwProfileName) as LPBYTE,
        &mut dwLength,
    ) != 0
    {
        let noname: &[u16] = &{
            let s = "Noname Hardware Profile\0";
            let mut v = [0u16; 32];
            let mut i = 0;
            for c in s.encode_utf16() {
                v[i] = c;
                i += 1;
            }
            v
        };

        wcscpy((*lpHwProfileInfo).szHwProfileName.as_mut_ptr(), noname.as_ptr());
        dwLength =
            (wcslen((*lpHwProfileInfo).szHwProfileName.as_ptr()) as DWORD + 1)
                * core::mem::size_of::<u16>() as DWORD;
        RegSetValueExW(
            hProfileKey,
            friendlyname.as_ptr(),
            0,
            REG_SZ,
            (*lpHwProfileInfo).szHwProfileName.as_mut_ptr() as LPBYTE,
            dwLength,
        );
    }

    RegCloseKey(hProfileKey);
    RegCloseKey(hDbKey);

    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/advapi32/misc/hwprofiles.c
//  * PURPOSE:         advapi32.dll Hardware Functions
//  * PROGRAMMER:      Steven Edwards
//  *                  Eric Kohl
//  */
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /******************************************************************************
//  * GetCurrentHwProfileA [ADVAPI32.@]
//  *
//  * Get the current hardware profile.
//  *
//  * PARAMS
//  *  lpHwProfileInfo [O] Destination for hardware profile information.
//  *
//  * RETURNS
//  *  Success: TRUE. lpHwProfileInfo is updated with the hardware profile details.
//  *  Failure: FALSE.
//  *
//  * @implemented
//  */
// BOOL WINAPI
// GetCurrentHwProfileA(LPHW_PROFILE_INFOA lpHwProfileInfo)
// {
//     HW_PROFILE_INFOW ProfileInfo;
//     UNICODE_STRING StringU;
//     ANSI_STRING StringA;
//     BOOL bResult;
//     NTSTATUS Status;
//
//     TRACE("GetCurrentHwProfileA() called\n");
//
//     bResult = GetCurrentHwProfileW(&ProfileInfo);
//     if (bResult == FALSE)
//         return FALSE;
//
//     lpHwProfileInfo->dwDockInfo = ProfileInfo.dwDockInfo;
//
//     /* Convert the profile GUID to ANSI */
//     StringU.Buffer = ProfileInfo.szHwProfileGuid;
//     StringU.Length = (USHORT)wcslen(ProfileInfo.szHwProfileGuid) * sizeof(WCHAR);
//     StringU.MaximumLength = HW_PROFILE_GUIDLEN * sizeof(WCHAR);
//     StringA.Buffer = (PCHAR)&lpHwProfileInfo->szHwProfileGuid;
//     StringA.Length = 0;
//     StringA.MaximumLength = HW_PROFILE_GUIDLEN;
//     Status = RtlUnicodeStringToAnsiString(&StringA,
//                                           &StringU,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     /* Convert the profile name to ANSI */
//     StringU.Buffer = ProfileInfo.szHwProfileName;
//     StringU.Length = (USHORT)wcslen(ProfileInfo.szHwProfileName) * sizeof(WCHAR);
//     StringU.MaximumLength = MAX_PROFILE_LEN * sizeof(WCHAR);
//     StringA.Buffer = (PCHAR)&lpHwProfileInfo->szHwProfileName;
//     StringA.Length = 0;
//     StringA.MaximumLength = MAX_PROFILE_LEN;
//     Status = RtlUnicodeStringToAnsiString(&StringA,
//                                           &StringU,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// GetCurrentHwProfileW(LPHW_PROFILE_INFOW lpHwProfileInfo)
// {
//     WCHAR szKeyName[256];
//     HKEY hDbKey;
//     HKEY hProfileKey;
//     DWORD dwLength;
//     DWORD dwConfigId;
//     UUID uuid;
//
//     TRACE("GetCurrentHwProfileW() called\n");
//
//     if (lpHwProfileInfo == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"System\\CurrentControlSet\\Control\\IDConfigDB",
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hDbKey))
//     {
//         SetLastError(ERROR_REGISTRY_CORRUPT);
//         return FALSE;
//     }
//
//     dwLength = sizeof(DWORD);
//     if (RegQueryValueExW(hDbKey,
//                          L"CurrentConfig",
//                          0,
//                          NULL,
//                          (LPBYTE)&dwConfigId,
//                          &dwLength))
//     {
//         RegCloseKey(hDbKey);
//         SetLastError(ERROR_REGISTRY_CORRUPT);
//         return FALSE;
//     }
//
//     swprintf(szKeyName,
//              L"Hardware Profile\\%04lu",
//              dwConfigId);
//
//     if (RegOpenKeyExW(hDbKey,
//                       szKeyName,
//                       0,
//                       KEY_QUERY_VALUE | KEY_SET_VALUE,
//                       &hProfileKey))
//     {
//         RegCloseKey(hDbKey);
//         SetLastError(ERROR_REGISTRY_CORRUPT);
//         return FALSE;
//     }
//
//     dwLength = sizeof(DWORD);
//     if (RegQueryValueExW(hProfileKey,
//                          L"DockState",
//                          0,
//                          NULL,
//                          (LPBYTE)&lpHwProfileInfo->dwDockInfo,
//                          &dwLength))
//     {
//         lpHwProfileInfo->dwDockInfo =
//             DOCKINFO_DOCKED | DOCKINFO_UNDOCKED | DOCKINFO_USER_SUPPLIED;
//     }
//
//     dwLength = HW_PROFILE_GUIDLEN * sizeof(WCHAR);
//     if (RegQueryValueExW(hProfileKey,
//                          L"HwProfileGuid",
//                          0,
//                          NULL,
//                          (LPBYTE)&lpHwProfileInfo->szHwProfileGuid,
//                          &dwLength))
//     {
//         /* Create a new GUID */
//         UuidCreate(&uuid);
//         swprintf(
//             lpHwProfileInfo->szHwProfileGuid,
//             L"{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
//             uuid.Data1,
//             uuid.Data2,
//             uuid.Data3,
//             uuid.Data4[0], uuid.Data4[1],
//             uuid.Data4[2], uuid.Data4[3], uuid.Data4[4], uuid.Data4[5],
//             uuid.Data4[6], uuid.Data4[7]);
//
//         dwLength = (wcslen(lpHwProfileInfo->szHwProfileGuid) + 1) * sizeof(WCHAR);
//         RegSetValueExW(hProfileKey,
//                        L"HwProfileGuid",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)lpHwProfileInfo->szHwProfileGuid,
//                        dwLength);
//     }
//
//     dwLength = MAX_PROFILE_LEN * sizeof(WCHAR);
//     if (RegQueryValueExW(hProfileKey,
//                          L"FriendlyName",
//                          0,
//                          NULL,
//                          (LPBYTE)&lpHwProfileInfo->szHwProfileName,
//                          &dwLength))
//     {
//         wcscpy(lpHwProfileInfo->szHwProfileName,
//                L"Noname Hardware Profile");
//         dwLength = (wcslen(lpHwProfileInfo->szHwProfileName) + 1) * sizeof(WCHAR);
//         RegSetValueExW(hProfileKey,
//                        L"FriendlyName",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)lpHwProfileInfo->szHwProfileName,
//                        dwLength);
//     }
//
//     RegCloseKey(hProfileKey);
//     RegCloseKey(hDbKey);
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
