use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

pub const WINE_DEFAULT_DEBUG_CHANNEL: &[u8; 9] = b"netapi32\0";

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netapi32.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"DllMain\0".as_ptr() as *const core::ffi::c_char,
            25i32,
            b"%p,%x,%p\n\0".as_ptr() as *const core::ffi::c_char,
            hinstDLL,
            fdwReason,
            lpvReserved,
        );
    }
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            InitializeListHead(&raw mut g_EnumContextListHead);
            InitializeCriticalSection(&raw mut g_EnumContextListLock);
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            NetBIOSInit();
            NetBTInit();
        }
        DLL_PROCESS_DETACH => {
            if lpvReserved.is_null() {
                NetBIOSShutdown();
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
// /* Copyright 2001 Mike McCormack
//  * Copyright 2003 Juan Lang
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
// #include "netapi32.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// BOOL WINAPI DllMain (HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved)
// {
//     TRACE("%p,%x,%p\n", hinstDLL, fdwReason, lpvReserved);
//
//     switch (fdwReason) {
//         case DLL_PROCESS_ATTACH:
//             InitializeListHead(&g_EnumContextListHead);
//             InitializeCriticalSection(&g_EnumContextListLock);
//             DisableThreadLibraryCalls(hinstDLL);
//             NetBIOSInit();
//             NetBTInit();
//             break;
//         case DLL_PROCESS_DETACH:
//             if (lpvReserved) break;
//             NetBIOSShutdown();
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
