use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hwldap32: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinst: HINSTANCE, reason: u32, reserved: LPVOID) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            hwldap32 = hinst;
            DisableThreadLibraryCalls(hinst);
        }
        _ => {}
    }
    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * WLDAP32 - LDAP support for Wine
//  *
//  * Copyright 2005 Hans Leidekker
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
// #include "config.h"
//
// #include "wine/debug.h"
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
//
// HINSTANCE hwldap32;
//
// WINE_DEFAULT_DEBUG_CHANNEL(wldap32);
//
// BOOL WINAPI DllMain( HINSTANCE hinst, DWORD reason, LPVOID reserved )
// {
//     TRACE( "(%p, %d, %p)\n", hinst, reason, reserved );
//
//     switch (reason)
//     {
//     case DLL_PROCESS_ATTACH:
//         hwldap32 = hinst;
//         DisableThreadLibraryCalls( hinst );
//         break;
//     }
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
