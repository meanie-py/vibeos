use crate::prelude::*;
#[no_mangle]
pub unsafe extern "C" fn LsapLoadString(
    hInstance: HINSTANCE,
    uId: u32,
    lpBuffer: *mut u16,
    nBufferMax: i32,
) -> i32 {
    let resource_id = (((uId & 0xffff) >> 4) + 1) as u16;
    let hrsrc = FindResourceW(
        hInstance,
        resource_id as *mut u16,
        6u16 as *mut u16, // RT_STRING
    );
    if hrsrc.is_null() {
        return 0;
    }
    let hmem = LoadResource(hInstance, hrsrc);
    if hmem.is_null() {
        return 0;
    }
    let mut p = LockResource(hmem) as *mut u16;
    let string_num = (uId & 0x000f) as i32;
    let mut i = 0;
    while i < string_num {
        p = p.add((*p as usize) + 1);
        i += 1;
    }
    i = if (nBufferMax - 1) < *p as i32 {
        nBufferMax - 1
    } else {
        *p as i32
    };
    if i > 0 {
        core::ptr::copy_nonoverlapping(p.add(1), lpBuffer, i as usize);
        *lpBuffer.add(i as usize) = 0;
    } else if nBufferMax > 1 {
        *lpBuffer.add(0) = 0;
        return 0;
    }
    i
}
#[no_mangle]
pub unsafe extern "C" fn LsapGetResourceStringLengthEx(
    hInstance: HINSTANCE,
    uId: UINT,
    _usLanguage: USHORT,
) -> INT {
    let name = ((uId & 0xFFFF) >> 4) + 1;
    let hrsrc = FindResourceW(
        hInstance as HMODULE,
        name as *const u16,
        6u32 as *const u16,
    );
    if hrsrc.is_null() {
        return 0;
    }
    let hmem = LoadResource(hInstance, hrsrc);
    if hmem.is_null() {
        return 0;
    }
    let mut p = LockResource(hmem) as *mut WCHAR;
    let string_num = uId & 0xFu32;
    for _ in 0..string_num {
        p = p.offset((*p as isize) + 1);
    }
    (*p as INT) + 1
}
#[no_mangle]
pub unsafe extern "C" fn LsapLoadStringEx(
    hInstance: HINSTANCE,
    uId: UINT,
    _usLanguage: USHORT,
    lpBuffer: LPWSTR,
    nBufferMax: INT,
) -> INT {
    let hrsrc: HRSRC = FindResourceW(
        hInstance as HMODULE,
        (((uId as u16 & 0xFFFF) >> 4) + 1) as ULONG_PTR as LPWSTR as LPCWSTR,
        (6 as ULONG_PTR) as LPWSTR as LPCWSTR,
    );
    if hrsrc.is_null() {
        return 0;
    }

    let hmem: HGLOBAL = LoadResource(hInstance, hrsrc);
    if hmem.is_null() {
        return 0;
    }

    let mut p: *mut WCHAR = LockResource(hmem) as *mut WCHAR;
    let string_num: i32 = (uId & 0xF) as i32;

    let mut i: i32 = 0;
    while i < string_num {
        p = p.offset((*p as i32 + 1) as isize);
        i += 1;
    }

    i = if (nBufferMax - 1) < (*p as i32) {
        nBufferMax - 1
    } else {
        *p as i32
    };

    if i > 0 {
        core::ptr::copy_nonoverlapping(p.offset(1), lpBuffer, i as usize);
        *lpBuffer.offset(i as isize) = 0;
    } else if nBufferMax > 1 {
        *lpBuffer = 0;
        return 0;
    }

    i
}
#[no_mangle]
pub unsafe extern "C" fn LsapAppendRidToSid(SrcSid: PSID, Rid: ULONG) -> PSID {
    let mut Rids: [ULONG; 8] = [0; 8];
    let mut DstSid: PSID = core::ptr::null_mut();

    let mut RidCount: UCHAR = *RtlSubAuthorityCountSid(SrcSid);
    if RidCount >= 8 {
        return core::ptr::null_mut();
    }

    for i in 0..RidCount as ULONG {
        Rids[i as usize] = *RtlSubAuthoritySid(SrcSid, i);
    }

    Rids[RidCount as usize] = Rid;
    RidCount = RidCount.wrapping_add(1);

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
        core::ptr::addr_of_mut!(DstSid),
    );

    DstSid
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/lsasrv/utils.c
//  * PURPOSE:         Utility functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// #include <winuser.h>
//
// /* FUNCTIONS ***************************************************************/
//
// INT
// LsapLoadString(HINSTANCE hInstance,
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
// INT
// LsapGetResourceStringLengthEx(
//      HINSTANCE hInstance,
//      UINT uId,
//      USHORT usLanguage)
// {
//     HGLOBAL hmem;
//     HRSRC hrsrc;
//     WCHAR *p;
//     UINT i, string_num;
//
//     /* Use loword (incremented by 1) as resourceid */
// //    hrsrc = FindResourceExW(hInstance,
// //                            MAKEINTRESOURCEW((LOWORD(uId) >> 4) + 1),
// //                            (LPWSTR)RT_STRING,
// //                            usLanguage);
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
//     return *p + 1;
// }
//
//
// INT
// LsapLoadStringEx(
//      HINSTANCE hInstance,
//      UINT uId,
//      USHORT usLanguage,
//      LPWSTR lpBuffer,
//      INT nBufferMax)
// {
//     HGLOBAL hmem;
//     HRSRC hrsrc;
//     WCHAR *p;
//     int string_num;
//     int i;
//
//     /* Use loword (incremented by 1) as resourceid */
// //    hrsrc = FindResourceExW(hInstance,
// //                            MAKEINTRESOURCEW((LOWORD(uId) >> 4) + 1),
// //                            (LPWSTR)RT_STRING,
// //                            usLanguage);
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
// PSID
// LsapAppendRidToSid(
//     PSID SrcSid,
//     ULONG Rid)
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
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
