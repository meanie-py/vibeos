use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::substrate::SetLastError;

// ---- file-scope preamble (translated) ----

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }

#[no_mangle]
pub unsafe extern "system" fn GetVersion() -> DWORD {
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    // C promotes each field to int/DWORD before shifting; do the shifts in DWORD so
    // OSMinorVersion << 8 (and OSBuildNumber << 16) don't wrap in the narrow field type.
    return (((*Peb).OSPlatformId ^ 2) << 30)
        | (((*Peb).OSBuildNumber as DWORD) << 16)
        | (((*Peb).OSMinorVersion as DWORD) << 8)
        | ((*Peb).OSMajorVersion as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn GetVersionExW(
    lpVersionInformation: LPOSVERSIONINFOW,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    let mut lpVersionInformationEx: LPOSVERSIONINFOEXW = core::ptr::null_mut();

    if ((*lpVersionInformation).dwOSVersionInfoSize as usize
        != core::mem::size_of::<OSVERSIONINFOW>() as usize)
        && ((*lpVersionInformation).dwOSVersionInfoSize as usize
            != core::mem::size_of::<OSVERSIONINFOEXW>() as usize)
    {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    Status = RtlGetVersion(lpVersionInformation as PRTL_OSVERSIONINFOW);
    if Status == STATUS_SUCCESS {
        if (*lpVersionInformation).dwOSVersionInfoSize as usize
            == core::mem::size_of::<OSVERSIONINFOEXW>() as usize
        {
            lpVersionInformationEx = lpVersionInformation as LPOSVERSIONINFOEXW;
            (*lpVersionInformationEx).wReserved = 0;
        }
        return 1;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetVersionExA(
    lpVersionInformation: LPOSVERSIONINFOA,
) -> BOOL {
    let mut VersionInformation: OSVERSIONINFOEXW = core::mem::zeroed();
    let mut lpVersionInformationEx: LPOSVERSIONINFOEXA = core::ptr::null_mut();
    let mut CsdVersionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut CsdVersionA: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if ((*lpVersionInformation).dwOSVersionInfoSize as usize
        != core::mem::size_of::<OSVERSIONINFOA>() as usize)
        && ((*lpVersionInformation).dwOSVersionInfoSize as usize
            != core::mem::size_of::<OSVERSIONINFOEXA>() as usize)
    {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    VersionInformation.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD;

    if GetVersionExW(&mut VersionInformation as *mut OSVERSIONINFOEXW as LPOSVERSIONINFOW) == 0 {
        return 0;
    }

    (*lpVersionInformation).dwMajorVersion = VersionInformation.dwMajorVersion;
    (*lpVersionInformation).dwMinorVersion = VersionInformation.dwMinorVersion;
    (*lpVersionInformation).dwBuildNumber = VersionInformation.dwBuildNumber;
    (*lpVersionInformation).dwPlatformId = VersionInformation.dwPlatformId;

    if (*lpVersionInformation).dwOSVersionInfoSize as usize
        == core::mem::size_of::<OSVERSIONINFOEXA>() as usize
    {
        lpVersionInformationEx = lpVersionInformation as *mut OSVERSIONINFOEXA;
        (*lpVersionInformationEx).wServicePackMajor = VersionInformation.wServicePackMajor;
        (*lpVersionInformationEx).wServicePackMinor = VersionInformation.wServicePackMinor;
        (*lpVersionInformationEx).wSuiteMask = VersionInformation.wSuiteMask;
        (*lpVersionInformationEx).wProductType = VersionInformation.wProductType;
        (*lpVersionInformationEx).wReserved = VersionInformation.wReserved;
    }

    RtlInitEmptyAnsiString(
        &mut CsdVersionA,
        (*lpVersionInformation).szCSDVersion.as_mut_ptr(),
        core::mem::size_of::<[u8; 128]>() as u16,
    );
    RtlInitUnicodeString(
        &mut CsdVersionW,
        VersionInformation.szCSDVersion.as_ptr(),
    );
    Status = RtlUnicodeStringToAnsiString(
        &mut CsdVersionA,
        &CsdVersionW,
        0,
    );
    return (Status >= 0) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn VerifyVersionInfoW(
    lpVersionInformation: LPOSVERSIONINFOEXW,
    dwTypeMask: DWORD,
    dwlConditionMask: DWORDLONG,
) -> BOOL {
    let Status: NTSTATUS;
    Status = RtlVerifyVersionInfo(
        lpVersionInformation as PRTL_OSVERSIONINFOEXW,
        dwTypeMask,
        dwlConditionMask,
    );
    match Status {
        STATUS_INVALID_PARAMETER => {
            SetLastError(ERROR_BAD_ARGUMENTS as DWORD);
            return FALSE as BOOL;
        }
        STATUS_REVISION_MISMATCH => {
            if !lpVersionInformation.is_null() {
                // DPRINT removed
            } else {
                // DPRINT removed
            }
            SetLastError(ERROR_OLD_WIN_VERSION as DWORD);
            return FALSE as BOOL;
        }
        _ => {
            debug_assert!(NT_SUCCESS(Status));
            return TRUE as BOOL;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn VerifyVersionInfoA(
    lpVersionInformation: LPOSVERSIONINFOEXA,
    dwTypeMask: DWORD,
    dwlConditionMask: DWORDLONG,
) -> BOOL {
    let mut viex: OSVERSIONINFOEXW = core::mem::zeroed();
    viex.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD;
    viex.dwMajorVersion = (*lpVersionInformation).dwMajorVersion;
    viex.dwMinorVersion = (*lpVersionInformation).dwMinorVersion;
    viex.dwBuildNumber = (*lpVersionInformation).dwBuildNumber;
    viex.dwPlatformId = (*lpVersionInformation).dwPlatformId;
    viex.wServicePackMajor = (*lpVersionInformation).wServicePackMajor;
    viex.wServicePackMinor = (*lpVersionInformation).wServicePackMinor;
    viex.wSuiteMask = (*lpVersionInformation).wSuiteMask;
    viex.wProductType = (*lpVersionInformation).wProductType;
    viex.wReserved = (*lpVersionInformation).wReserved;
    return VerifyVersionInfoW(&raw mut viex, dwTypeMask, dwlConditionMask);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/version.c
//  * PURPOSE:         Version functions
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//                     Ged Murphy (gedmurphy@reactos.org)
//  */
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetVersion(VOID)
// {
//     PPEB Peb = NtCurrentPeb();
//
//     return (DWORD)( ((Peb->OSPlatformId ^ 2) << 30) |
//                      (Peb->OSBuildNumber     << 16) |
//                      (Peb->OSMinorVersion    << 8 ) |
//                       Peb->OSMajorVersion );
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVersionExW(IN LPOSVERSIONINFOW lpVersionInformation)
// {
//     NTSTATUS Status;
//     LPOSVERSIONINFOEXW lpVersionInformationEx;
//
//     if ((lpVersionInformation->dwOSVersionInfoSize != sizeof(OSVERSIONINFOW)) &&
//         (lpVersionInformation->dwOSVersionInfoSize != sizeof(OSVERSIONINFOEXW)))
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
//
//     Status = RtlGetVersion((PRTL_OSVERSIONINFOW)lpVersionInformation);
//     if (Status == STATUS_SUCCESS)
//     {
//         if (lpVersionInformation->dwOSVersionInfoSize == sizeof(OSVERSIONINFOEXW))
//         {
//             lpVersionInformationEx = (PVOID)lpVersionInformation;
//             lpVersionInformationEx->wReserved = 0;
//         }
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetVersionExA(IN LPOSVERSIONINFOA lpVersionInformation)
// {
//     OSVERSIONINFOEXW VersionInformation;
//     LPOSVERSIONINFOEXA lpVersionInformationEx;
//     UNICODE_STRING CsdVersionW;
//     NTSTATUS Status;
//     ANSI_STRING CsdVersionA;
//
//     if ((lpVersionInformation->dwOSVersionInfoSize != sizeof(OSVERSIONINFOA)) &&
//         (lpVersionInformation->dwOSVersionInfoSize != sizeof(OSVERSIONINFOEXA)))
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return FALSE;
//     }
//
//     VersionInformation.dwOSVersionInfoSize = sizeof(OSVERSIONINFOEXW);
//
//     if (!GetVersionExW((LPOSVERSIONINFOW)&VersionInformation)) return FALSE;
//
//     /* Copy back fields that match both supported structures */
//     lpVersionInformation->dwMajorVersion = VersionInformation.dwMajorVersion;
//     lpVersionInformation->dwMinorVersion = VersionInformation.dwMinorVersion;
//     lpVersionInformation->dwBuildNumber = VersionInformation.dwBuildNumber;
//     lpVersionInformation->dwPlatformId = VersionInformation.dwPlatformId;
//
//     if (lpVersionInformation->dwOSVersionInfoSize == sizeof(OSVERSIONINFOEXA))
//     {
//         lpVersionInformationEx = (PVOID)lpVersionInformation;
//         lpVersionInformationEx->wServicePackMajor = VersionInformation.wServicePackMajor;
//         lpVersionInformationEx->wServicePackMinor = VersionInformation.wServicePackMinor;
//         lpVersionInformationEx->wSuiteMask = VersionInformation.wSuiteMask;
//         lpVersionInformationEx->wProductType = VersionInformation.wProductType;
//         lpVersionInformationEx->wReserved = VersionInformation.wReserved;
//     }
//
//     /* Convert the CSD string */
//     RtlInitEmptyAnsiString(&CsdVersionA,
//                            lpVersionInformation->szCSDVersion,
//                            sizeof(lpVersionInformation->szCSDVersion));
//     RtlInitUnicodeString(&CsdVersionW, VersionInformation.szCSDVersion);
//     Status = RtlUnicodeStringToAnsiString(&CsdVersionA, &CsdVersionW, FALSE);
//     return (NT_SUCCESS(Status));
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// VerifyVersionInfoW(IN LPOSVERSIONINFOEXW lpVersionInformation,
//                    IN DWORD dwTypeMask,
//                    IN DWORDLONG dwlConditionMask)
// {
//     NTSTATUS Status;
//
//     Status = RtlVerifyVersionInfo((PRTL_OSVERSIONINFOEXW)lpVersionInformation,
//                                   dwTypeMask,
//                                   dwlConditionMask);
//     switch (Status)
//     {
//         case STATUS_INVALID_PARAMETER:
//             SetLastError(ERROR_BAD_ARGUMENTS);
//             return FALSE;
//
//         case STATUS_REVISION_MISMATCH:
//             if (lpVersionInformation)
//             {
//                 DPRINT1("VerifyVersionInfo -- Version mismatch(%d.%d.%d:%d)\n",
//                         (dwTypeMask & VER_MAJORVERSION) ? lpVersionInformation->dwMajorVersion : -1,
//                         (dwTypeMask & VER_MINORVERSION) ? lpVersionInformation->dwMinorVersion : -1,
//                         (dwTypeMask & VER_BUILDNUMBER) ? lpVersionInformation->dwBuildNumber : -1,
//                         (dwTypeMask & VER_PLATFORMID) ? lpVersionInformation->dwPlatformId : -1);
//             }
//             else
//             {
//                 DPRINT1("VerifyVersionInfo -- Version mismatch(NULL)\n");
//             }
//             SetLastError(ERROR_OLD_WIN_VERSION);
//             return FALSE;
//
//         default:
//             /* RtlVerifyVersionInfo shouldn't report any other failure code! */
//             ASSERT(NT_SUCCESS(Status));
//             return TRUE;
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// VerifyVersionInfoA(IN LPOSVERSIONINFOEXA lpVersionInformation,
//                    IN DWORD dwTypeMask,
//                    IN DWORDLONG dwlConditionMask)
// {
//     OSVERSIONINFOEXW viex;
//
//     /* NOTE: szCSDVersion is ignored, we don't need to convert it to Unicode */
//     viex.dwOSVersionInfoSize = sizeof(viex);
//     viex.dwMajorVersion = lpVersionInformation->dwMajorVersion;
//     viex.dwMinorVersion = lpVersionInformation->dwMinorVersion;
//     viex.dwBuildNumber = lpVersionInformation->dwBuildNumber;
//     viex.dwPlatformId = lpVersionInformation->dwPlatformId;
//     viex.wServicePackMajor = lpVersionInformation->wServicePackMajor;
//     viex.wServicePackMinor = lpVersionInformation->wServicePackMinor;
//     viex.wSuiteMask = lpVersionInformation->wSuiteMask;
//     viex.wProductType = lpVersionInformation->wProductType;
//     viex.wReserved = lpVersionInformation->wReserved;
//     return VerifyVersionInfoW(&viex, dwTypeMask, dwlConditionMask);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
