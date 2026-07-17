use crate::prelude::*;
#[no_mangle]
pub unsafe extern "C" fn SampLoadString(
    hInstance: HINSTANCE,
    uId: UINT,
    lpBuffer: LPWSTR,
    nBufferMax: INT,
) -> INT {
    let hmem: HGLOBAL;
    let hrsrc: HRSRC;
    let mut p: *mut WCHAR;
    let string_num: i32;
    let mut i: i32;

    hrsrc = FindResourceW(
        hInstance,
        MAKEINTRESOURCEW((LOWORD(uId) >> 4) + 1),
        RT_STRING as LPWSTR,
    );
    if hrsrc.is_null() {
        return 0;
    }

    hmem = LoadResource(hInstance, hrsrc);
    if hmem.is_null() {
        return 0;
    }

    p = LockResource(hmem) as *mut WCHAR;
    string_num = (uId & 0x000f) as i32;
    i = 0;
    while i < string_num {
        p = p.offset((*p + 1) as isize);
        i += 1;
    }

    i = min(nBufferMax - 1, *p as i32);
    if i > 0 {
        memcpy(
            lpBuffer as *mut core::ffi::c_void,
            p.offset(1) as *const core::ffi::c_void,
            (i as usize) * core::mem::size_of::<WCHAR>(),
        );
        *lpBuffer.offset(i as isize) = 0;
    } else {
        if nBufferMax > 1 {
            *lpBuffer.offset(0) = 0;
            return 0;
        }
    }

    i
}
#[no_mangle]
pub unsafe extern "C" fn SampIsSetupRunning() -> BOOL {
    let mut dwError: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwSetupType: DWORD = 0;

    let setup_path: [u16; 13] = [
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16,
        b'M' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'u' as u16, b'p' as u16, 0,
    ];
    let setup_type_name: [u16; 10] = [
        b'S' as u16, b'e' as u16, b't' as u16, b'u' as u16, b'p' as u16,
        b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, 0,
    ];

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        setup_path.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return FALSE as BOOL;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hKey,
        setup_type_name.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwSetupType as *mut DWORD as LPBYTE,
        &mut dwSize,
    ) as DWORD;

    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS
        || dwType != REG_DWORD
        || dwSize != core::mem::size_of::<DWORD>() as DWORD
    {
        return FALSE as BOOL;
    }

    (dwSetupType != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn AppendRidToSid(SrcSid: PSID, Rid: ULONG) -> PSID {
    let mut Rids: [ULONG; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    let mut RidCount: UCHAR;
    let mut DstSid: PSID = core::ptr::null_mut();
    let mut i: ULONG;

    RidCount = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8 {
        return core::ptr::null_mut();
    }

    i = 0;
    while i < RidCount as ULONG {
        Rids[i as usize] = *RtlSubAuthoritySid(SrcSid, i);
        i += 1;
    }

    Rids[RidCount as usize] = Rid;
    RidCount += 1;

    RtlAllocateAndInitializeSid(
        RtlIdentifierAuthoritySid(SrcSid),
        RidCount,
        Rids[0],
        Rids[1],
        Rids[2],
        Rids[3],
        Rids[4],
        Rids[5],
        Rids[6],
        Rids[7],
        &mut DstSid,
    );

    DstSid
}
#[no_mangle]
pub unsafe extern "C" fn SampGetRidFromSid(Sid: PSID, Rid: *mut ULONG) -> NTSTATUS {
    let RidCount: UCHAR = *RtlSubAuthorityCountSid(Sid);
    if RidCount < 1 {
        return STATUS_INVALID_SID;
    }

    *Rid = *RtlSubAuthoritySid(Sid, (RidCount - 1) as ULONG);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn SampCheckAccountName(
    AccountName: PRPC_UNICODE_STRING,
    MaxLength: USHORT,
) -> NTSTATUS {
    if (*AccountName).Length as usize > MaxLength as usize * core::mem::size_of::<WCHAR>() {
        return STATUS_INVALID_ACCOUNT_NAME;
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/utils.c
//  * PURPOSE:         Utility functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// #include <winuser.h>
//
// /* FUNCTIONS ***************************************************************/
//
// INT
// SampLoadString(HINSTANCE hInstance,
//                UINT uId,
//                LPWSTR lpBuffer,
//                INT nBufferMax)
// {
//     HGLOBAL hmem;
//     HRSRC hrsrc;
//     WCHAR *p;
//     int string_num;
//     int i;
//
//     /* Use loword (incremented by 1) as resourceid */
//     hrsrc = FindResourceW(hInstance,
//                           MAKEINTRESOURCEW((LOWORD(uId) >> 4) + 1),
//                           (LPWSTR)RT_STRING);
//     if (!hrsrc)
//         return 0;
//
//     hmem = LoadResource(hInstance, hrsrc);
//     if (!hmem)
//         return 0;
//
//     p = LockResource(hmem);
//     string_num = uId & 0x000f;
//     for (i = 0; i < string_num; i++)
//         p += *p + 1;
//
//     i = min(nBufferMax - 1, *p);
//     if (i > 0)
//     {
//         memcpy(lpBuffer, p + 1, i * sizeof(WCHAR));
//         lpBuffer[i] = 0;
//     }
//     else
//     {
//         if (nBufferMax > 1)
//         {
//             lpBuffer[0] = 0;
//             return 0;
//         }
//     }
//
//     return i;
// }
//
//
// BOOL
// SampIsSetupRunning(VOID)
// {
//     DWORD dwError;
//     HKEY hKey;
//     DWORD dwType;
//     DWORD dwSize;
//     DWORD dwSetupType;
//
//     TRACE("SampIsSetupRunning()\n");
//
//     /* Open key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\Setup",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hKey);
//     if (dwError != ERROR_SUCCESS)
//         return FALSE;
//
//     /* Read key */
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hKey,
//                                L"SetupType",
//                                NULL,
//                                &dwType,
//                                (LPBYTE)&dwSetupType,
//                                &dwSize);
//
//     /* Close key, and check if returned values are correct */
//     RegCloseKey(hKey);
//     if (dwError != ERROR_SUCCESS || dwType != REG_DWORD || dwSize != sizeof(DWORD))
//         return FALSE;
//
//     TRACE("SampIsSetupRunning() returns %s\n", (dwSetupType != 0) ? "TRUE" : "FALSE");
//     return (dwSetupType != 0);
// }
//
//
// PSID
// AppendRidToSid(PSID SrcSid,
//                ULONG Rid)
// {
//     ULONG Rids[8] = {0, 0, 0, 0, 0, 0, 0, 0};
//     UCHAR RidCount;
//     PSID DstSid;
//     ULONG i;
//
//     RidCount = *RtlSubAuthorityCountSid(SrcSid);
//     if (RidCount >= 8)
//         return NULL;
//
//     for (i = 0; i < RidCount; i++)
//         Rids[i] = *RtlSubAuthoritySid(SrcSid, i);
//
//     Rids[RidCount] = Rid;
//     RidCount++;
//
//     RtlAllocateAndInitializeSid(RtlIdentifierAuthoritySid(SrcSid),
//                                 RidCount,
//                                 Rids[0],
//                                 Rids[1],
//                                 Rids[2],
//                                 Rids[3],
//                                 Rids[4],
//                                 Rids[5],
//                                 Rids[6],
//                                 Rids[7],
//                                 &DstSid);
//
//     return DstSid;
// }
//
//
// NTSTATUS
// SampGetRidFromSid(IN PSID Sid,
//                   OUT PULONG Rid)
// {
//     UCHAR RidCount;
//
//     RidCount = *RtlSubAuthorityCountSid(Sid);
//     if (RidCount < 1)
//         return STATUS_INVALID_SID;
//
//     *Rid = *RtlSubAuthoritySid(Sid, RidCount - 1);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SampCheckAccountName(IN PRPC_UNICODE_STRING AccountName,
//                      IN USHORT MaxLength)
// {
//     if (AccountName->Length > MaxLength * sizeof(WCHAR))
//         return STATUS_INVALID_ACCOUNT_NAME;
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
