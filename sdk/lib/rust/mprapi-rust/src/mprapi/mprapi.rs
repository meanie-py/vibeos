use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: u32,
    reserved: LPVOID,
) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinst);
        }
        _ => {}
    }
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn MprAdminGetErrorString(mprerror: DWORD, localstr: *mut LPWSTR) -> DWORD {
    *localstr = core::ptr::null_mut();
    return ERROR_MR_MID_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn MprAdminIsServiceRunning(server: LPWSTR) -> BOOL {
    // FIXME!("(%s): stub!\n", debugstr_w(server));
    (FALSE) as i32
}

// Added by the 2026-09-20 upstream sync: upstream implemented MprConfigServerConnect
// (mprapi.c) and added it to mprapi.spec after our fork. The spec drives the .def, so the
// export went live in this tree the moment the merge landed and the link failed here —
// the oracle arriving as a link error rather than as a diff. Still a stub upstream, but
// the NULL check is the real behaviour and is worth having.
#[no_mangle]
pub unsafe extern "system" fn MprConfigServerConnect(
    _lpws_server_name: LPWSTR,
    phmpr_config: *mut HANDLE,
) -> DWORD {
    if phmpr_config.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    ERROR_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright (C) 2006 Dmitry Timoshkov
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "mprapi.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(mprapi);
//
// /*****************************************************
//  *      DllMain
//  */
// BOOL WINAPI DllMain(HINSTANCE hinst, DWORD reason, LPVOID reserved)
// {
//     TRACE("(%p, %d, %p)\n", hinst, reason, reserved);
//
//     switch(reason)
//     {
// #ifndef __REACTOS__
//     case DLL_WINE_PREATTACH:
//         return FALSE;  /* prefer native version */
// #endif
//
//     case DLL_PROCESS_ATTACH:
//         DisableThreadLibraryCalls( hinst );
//         break;
//     }
//     return TRUE;
// }
//
// /***********************************************************************
//  * MprAdminGetErrorString (MPRAPI.@)
//  *
//  * Return a unicode string for the given mpr errorcode
//  *
//  * PARAMS
//  *  mprerror [i] errorcode, for which a description is requested
//  *  localstr [o] pointer, where a buffer with the error description is returned
//  *
//  * RETURNS
//  *  Failure: ERROR_MR_MID_NOT_FOUND, when mprerror is not known
//  *  Success: ERROR_SUCCESS, and in localstr a pointer to a buffer from LocalAlloc,
//  *           which contains the error description.
//  *
//  * NOTES
//  *  The caller must free the returned buffer with LocalFree
//  *
//  */
// DWORD __stdcall MprAdminGetErrorString(DWORD mprerror, LPWSTR *localstr)
// {
//     FIXME("(0x%x/%u, %p): stub!\n", mprerror, mprerror, localstr);
//
//     *localstr = NULL;
//     return ERROR_MR_MID_NOT_FOUND;
// }
//
// /***********************************************************************
//  *      MprAdminIsServiceRunning (MPRAPI.@)
//  */
// BOOL __stdcall MprAdminIsServiceRunning(LPWSTR server)
// {
//     FIXME("(%s): stub!\n", debugstr_w(server));
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
