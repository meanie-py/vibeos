use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn debugstr_a(text: *const c_char) -> *const c_char;
    fn cmsSetLogErrorHandler(handler: Option<unsafe extern "C" fn(ctx: *mut core::ffi::c_void, error: u32, text: *const i8)>);
    fn free_handle_tables();
}
extern "system" {
    fn DisableThreadLibraryCalls(hinst: HINSTANCE) -> i32;
}

// ---- file-scope preamble (translated) ----
static DEBUG_CHANNEL: &str = "mscms";

unsafe fn lcms_error_handler(ctx: cmsContext, error: cmsUInt32Number, text: *const c_char) {
    TRACE!(b"%u %s\n\0".as_ptr() as *const c_char, error, debugstr_a(text));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinst: HINSTANCE, reason: u32, reserved: LPVOID) -> i32 {
    TRACE!("({:p}, {}, {:p})\n", hinst, reason, reserved);
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinst);
            cmsSetLogErrorHandler(Some(lcms_error_handler));
        }
        DLL_PROCESS_DETACH => {
            if reserved.is_null() {
                free_handle_tables();
            }
        }
        _ => {}
    }
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * MSCMS - Color Management System for Wine
//  *
//  * Copyright 2004, 2005 Hans Leidekker
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
// #include "wine/port.h"
// #include "wine/debug.h"
// #include "wine/library.h"
//
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "icm.h"
//
// #include "mscms_priv.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(mscms);
//
// #ifdef HAVE_LCMS2
// static void lcms_error_handler(cmsContext ctx, cmsUInt32Number error, const char *text)
// {
//     TRACE("%u %s\n", error, debugstr_a(text));
// }
// #endif
//
// BOOL WINAPI DllMain( HINSTANCE hinst, DWORD reason, LPVOID reserved )
// {
//     TRACE( "(%p, %d, %p)\n", hinst, reason, reserved );
//
//     switch (reason)
//     {
//     case DLL_PROCESS_ATTACH:
//         DisableThreadLibraryCalls( hinst );
// #ifdef HAVE_LCMS2
//         cmsSetLogErrorHandler( lcms_error_handler );
// #else
//         ERR( "No support for liblcms2, expect problems\n" );
// #endif
//         break;
//     case DLL_PROCESS_DETACH:
//         if (reserved) break;
// #ifdef HAVE_LCMS2
//         free_handle_tables();
// #endif
//         break;
//     }
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
