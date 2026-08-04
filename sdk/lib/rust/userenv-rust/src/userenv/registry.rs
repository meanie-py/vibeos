use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;
// L".Default"
static DOT_DEFAULT_KEY: [u16; 9] = [
    b'.' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16,
    b't' as u16, 0,
];
unsafe extern "C" fn CopyKey(mut hDstKey: HKEY, mut hSrcKey: HKEY) -> BOOL {
    let mut Error: LONG;
    Error = RegCopyTreeW(hSrcKey, core::ptr::null(), hDstKey) as LONG;
    if Error != ERROR_SUCCESS as LONG {
        SetLastError(Error as DWORD);
        return 0;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn CreateUserHive(mut lpKeyName: LPCWSTR, mut lpProfilePath: LPCWSTR) -> BOOL {
    let mut hDefaultKey: HKEY = core::ptr::null_mut();
    let mut hUserKey: HKEY = core::ptr::null_mut();
    let mut Error: LONG = 0;
    let mut Ret: BOOL = FALSE;

    'cleanup: {
    Error = RegOpenKeyExW(
        HKEY_USERS,
        DOT_DEFAULT_KEY.as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        &mut hDefaultKey,
    );
    if Error != (ERROR_SUCCESS as i32) {
        SetLastError(Error as DWORD);
        break 'cleanup;
    }

    Error = RegOpenKeyExW(
        HKEY_USERS,
        lpKeyName,
        0,
        KEY_ALL_ACCESS,
        &mut hUserKey,
    );
    if Error != (ERROR_SUCCESS as i32) {
        SetLastError(Error as DWORD);
        break 'cleanup;
    }

    if CopyKey(hUserKey, hDefaultKey) == 0 {
        break 'cleanup;
    }

    if UpdateUsersShellFolderSettings(lpProfilePath, hUserKey) == 0 {
        break 'cleanup;
    }

    RegFlushKey(hUserKey);
    Ret = TRUE;

} // cleanup:
    if !hUserKey.is_null() {
        RegCloseKey(hUserKey);
    }

    if !hDefaultKey.is_null() {
        RegCloseKey(hDefaultKey);
    }

    Ret
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
//  * FILE:            dll/win32/userenv/registry.c
//  * PURPOSE:         User profile code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// LSTATUS WINAPI RegCopyTreeW( HKEY,  LPCWSTR,  HKEY);
//
// /* FUNCTIONS ***************************************************************/
//
// static
// BOOL
// CopyKey(HKEY hDstKey,
//         HKEY hSrcKey)
// {
//     LONG Error;
//
//     Error = RegCopyTreeW(hSrcKey,
//                          NULL,
//                          hDstKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         SetLastError((DWORD)Error);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// BOOL
// CreateUserHive(LPCWSTR lpKeyName,
//                LPCWSTR lpProfilePath)
// {
//     HKEY hDefaultKey = NULL;
//     HKEY hUserKey = NULL;
//     LONG Error;
//     BOOL Ret = FALSE;
//
//     DPRINT("CreateUserHive(%S) called\n", lpKeyName);
//
//     Error = RegOpenKeyExW(HKEY_USERS,
//                           L".Default",
//                           0,
//                           KEY_READ,
//                           &hDefaultKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         SetLastError((DWORD)Error);
//         goto Cleanup;
//     }
//
//     Error = RegOpenKeyExW(HKEY_USERS,
//                           lpKeyName,
//                           0,
//                           KEY_ALL_ACCESS,
//                           &hUserKey);
//     if (Error != ERROR_SUCCESS)
//     {
//         SetLastError((DWORD)Error);
//         goto Cleanup;
//     }
//
//     if (!CopyKey(hUserKey, hDefaultKey))
//     {
//         goto Cleanup;
//     }
//
//     if (!UpdateUsersShellFolderSettings(lpProfilePath,
//                                         hUserKey))
//     {
//         goto Cleanup;
//     }
//
//     RegFlushKey(hUserKey);
//     Ret = TRUE;
//
// Cleanup:
//     if (hUserKey != NULL)
//         RegCloseKey (hUserKey);
//
//     if (hDefaultKey != NULL)
//         RegCloseKey (hDefaultKey);
//
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
