use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;

// GetUserSid is a stdcall WINAPI spec export (userenv.spec:142 `stdcall GetUserSid`, sid.c
// `PSID WINAPI GetUserSid`) → extern "system". Its internal callers (profile.rs/environment.rs)
// declare it extern "system" to match.
#[no_mangle]
pub unsafe extern "system" fn GetUserSid(mut hToken: HANDLE) -> PSID {
    let mut UserBuffer: PTOKEN_USER = core::ptr::null_mut::<_TOKEN_USER>();
    let mut TempBuffer: PTOKEN_USER = core::ptr::null_mut::<_TOKEN_USER>();
    let mut Status: NTSTATUS = 0;
    let mut Length: ULONG = 0;
    let mut pSid: PSID = core::ptr::null_mut::<core::ffi::c_void>();
    Length = 256 as ULONG;
    UserBuffer = LocalAlloc(LPTR as UINT, Length as usize) as PTOKEN_USER;
    if UserBuffer.is_null() {
        return core::ptr::null_mut();
    }
    Status = NtQueryInformationToken(
        hToken,
        TokenUser,
        UserBuffer as PVOID,
        Length,
        &raw mut Length,
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        TempBuffer = LocalReAlloc(
            UserBuffer as HLOCAL,
            Length as usize,
            LMEM_MOVEABLE as UINT,
        ) as PTOKEN_USER;
        if TempBuffer.is_null() {
            LocalFree(UserBuffer as HLOCAL);
            return core::ptr::null_mut();
        }
        UserBuffer = TempBuffer;
        Status = NtQueryInformationToken(
            hToken,
            TokenUser,
            UserBuffer as PVOID,
            Length,
            &raw mut Length,
        );
    }
    if !NT_SUCCESS(Status) {
        LocalFree(UserBuffer as HLOCAL);
        return core::ptr::null_mut();
    }
    Length = RtlLengthSid((*UserBuffer).User.Sid);
    pSid = LocalAlloc(LPTR as UINT, Length as usize) as PSID;
    if pSid.is_null() {
        LocalFree(UserBuffer as HLOCAL);
        return core::ptr::null_mut();
    }
    Status = RtlCopySid(Length, pSid, (*UserBuffer).User.Sid);
    LocalFree(UserBuffer as HLOCAL);
    if !NT_SUCCESS(Status) {
        LocalFree(pSid as HLOCAL);
        return core::ptr::null_mut();
    }
    pSid
}
#[no_mangle]
pub unsafe extern "C" fn GetUserSidStringFromToken(
    hToken: *mut core::ffi::c_void,
    SidString: PUNICODE_STRING,
) -> u32 {
    let mut UserBuffer: PTOKEN_USER = core::ptr::null_mut();
    let mut nsb: PTOKEN_USER;
    let mut Length: u32 = 256;
    let mut Status: i32;

    UserBuffer = LocalAlloc(LPTR as u32, Length as usize) as PTOKEN_USER;
    if UserBuffer.is_null() {
        return 0u32;
    }

    Status = NtQueryInformationToken(
        hToken,
        TokenUser,
        UserBuffer as *mut core::ffi::c_void,
        Length,
        &mut Length,
    );

    if Status == STATUS_BUFFER_TOO_SMALL {
        nsb = LocalReAlloc(
            UserBuffer as *mut core::ffi::c_void,
            Length as usize,
            LMEM_MOVEABLE as u32,
        ) as PTOKEN_USER;
        if nsb.is_null() {
            LocalFree(UserBuffer as *mut core::ffi::c_void);
            return 0u32;
        }

        UserBuffer = nsb;
        Status = NtQueryInformationToken(
            hToken,
            TokenUser,
            UserBuffer as *mut core::ffi::c_void,
            Length,
            &mut Length,
        );
    }

    if !NT_SUCCESS(Status) {
        LocalFree(UserBuffer as *mut core::ffi::c_void);
        SetLastError(RtlNtStatusToDosError(Status));
        return 0u32;
    }

    Status = RtlConvertSidToUnicodeString(SidString, (*UserBuffer).User.Sid, 1u8);

    LocalFree(UserBuffer as *mut core::ffi::c_void);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0u32;
    }

    return 1u32;
}
// enforce_import_abi: moved 4 import decl(s) into the extern block their
// exporting DLL's .spec requires: LocalAlloc->system, LocalFree->system, LocalReAlloc->system, SetLastError->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "system" {
    fn LocalAlloc(uFlags: u32, uBytes: usize) -> *mut core::ffi::c_void;
    fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn LocalReAlloc(hMem: *mut core::ffi::c_void, uBytes: usize, uFlags: u32) -> *mut core::ffi::c_void;
    fn SetLastError(dwErrCode: u32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/sid.c
//  * PURPOSE:         User profile code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #include <ndk/sefuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// PSID
// WINAPI
// GetUserSid(HANDLE hToken)
// {
//     PTOKEN_USER UserBuffer;
//     PTOKEN_USER TempBuffer;
//     NTSTATUS Status;
//     ULONG Length;
//     PSID pSid;
//
//     Length = 256;
//     UserBuffer = LocalAlloc(LPTR, Length);
//     if (UserBuffer == NULL)
//     {
//         return NULL;
//     }
//
//     Status = NtQueryInformationToken(hToken,
//                                      TokenUser,
//                                      (PVOID)UserBuffer,
//                                      Length,
//                                      &Length);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         TempBuffer = LocalReAlloc(UserBuffer, Length, LMEM_MOVEABLE);
//         if (TempBuffer == NULL)
//         {
//             LocalFree(UserBuffer);
//             return NULL;
//         }
//
//         UserBuffer = TempBuffer;
//         Status = NtQueryInformationToken(hToken,
//                                          TokenUser,
//                                          (PVOID)UserBuffer,
//                                          Length,
//                                          &Length);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         LocalFree(UserBuffer);
//         return NULL;
//     }
//
//         Length = RtlLengthSid(UserBuffer->User.Sid);
//
//     pSid = LocalAlloc(LPTR, Length);
//     if (pSid == NULL)
//     {
//         LocalFree(UserBuffer);
//         return NULL;
//     }
//
//     Status = RtlCopySid(Length, pSid, UserBuffer->User.Sid);
//
//     LocalFree(UserBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         LocalFree(pSid);
//         return NULL;
//     }
//
//     return pSid;
// }
//
// BOOL
// GetUserSidStringFromToken(HANDLE hToken,
//                           PUNICODE_STRING SidString)
// {
//     PTOKEN_USER UserBuffer, nsb;
//     ULONG Length;
//     NTSTATUS Status;
//
//     Length = 256;
//     UserBuffer = LocalAlloc(LPTR, Length);
//     if (UserBuffer == NULL)
//         return FALSE;
//
//     Status = NtQueryInformationToken(hToken,
//                                      TokenUser,
//                                      (PVOID)UserBuffer,
//                                      Length,
//                                      &Length);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         nsb = LocalReAlloc(UserBuffer,
//                            Length,
//                            LMEM_MOVEABLE);
//         if (nsb == NULL)
//         {
//             LocalFree(UserBuffer);
//             return FALSE;
//         }
//
//         UserBuffer = nsb;
//         Status = NtQueryInformationToken(hToken,
//                                          TokenUser,
//                                          (PVOID)UserBuffer,
//                                          Length,
//                                          &Length);
//     }
//
//     if (!NT_SUCCESS (Status))
//     {
//         LocalFree(UserBuffer);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     DPRINT("SidLength: %lu\n", RtlLengthSid (UserBuffer->User.Sid));
//
//     Status = RtlConvertSidToUnicodeString(SidString,
//                                           UserBuffer->User.Sid,
//                                           TRUE);
//
//     LocalFree(UserBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     DPRINT("SidString.Length: %lu\n", SidString->Length);
//     DPRINT("SidString.MaximumLength: %lu\n", SidString->MaximumLength);
//     DPRINT("SidString: '%wZ'\n", SidString);
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// DeriveAppContainerSidFromAppContainerName maps a profile name to its S-1-15-2-... SID.
// That SID is a fixed hash of the lowercased name, and this tree has no reference for the exact
// digest-to-subauthority layout to check against, so no SID is fabricated: arguments are validated
// as Windows does, then E_NOTIMPL is returned with the output cleared.
#[no_mangle]
pub unsafe extern "system" fn DeriveAppContainerSidFromAppContainerName(
    pszAppContainerName: LPCWSTR,
    ppsidAppContainerSid: *mut *mut c_void,
) -> i32 {
    if pszAppContainerName.is_null() || ppsidAppContainerSid.is_null() {
        return E_INVALIDARG_HR;
    }
    *ppsidAppContainerSid = core::ptr::null_mut();
    E_NOTIMPL_HR
}

const E_INVALIDARG_HR: i32 = 0x8007_0057u32 as i32;
const E_NOTIMPL_HR: i32 = 0x8000_4001u32 as i32;
