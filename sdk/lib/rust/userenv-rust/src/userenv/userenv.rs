use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// userenv.c:32 `HINSTANCE hInstance = NULL;`. `#[no_mangle]` so the bindgen `extern` import every
// other module reads resolves to THIS storage (a c2rust-default private static would be a twin
// DllMain writes and nobody else sees). It was C storage in userenv_rust_shims.c until 2026-09-25.
#[no_mangle]
pub static mut hInstance: HINSTANCE = core::ptr::null_mut();

extern {
    fn InitializeGPNotifications();
    fn UninitializeGPNotifications();
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    _lpvReserved: LPVOID,
) -> BOOL {
    if fdwReason == DLL_PROCESS_ATTACH {
        hInstance = hinstDLL;
        InitializeGPNotifications();
    } else if fdwReason == DLL_PROCESS_DETACH {
        UninitializeGPNotifications();
    }
    1
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
//  * FILE:            dll/win32/userenv/userenv.c
//  * PURPOSE:         DLL initialization code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// HINSTANCE hInstance = NULL;
//
// BOOL
// WINAPI
// DllMain(HINSTANCE hinstDLL,
//         DWORD fdwReason,
//         LPVOID lpvReserved)
// {
//     if (fdwReason == DLL_PROCESS_ATTACH)
//     {
//         hInstance = hinstDLL;
//         InitializeGPNotifications();
//     }
//     else if (fdwReason == DLL_PROCESS_DETACH)
//     {
//         UninitializeGPNotifications();
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
