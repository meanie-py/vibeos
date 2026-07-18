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

#[no_mangle]
pub unsafe extern "system" fn NetApiBufferAllocate(
    ByteCount: u32,
    Buffer: *mut LPVOID,
) -> u32 {
    if Buffer.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *Buffer = HeapAlloc(GetProcessHeap(), 0, ByteCount as SIZE_T);
    if !(*Buffer).is_null() {
        NERR_Success
    } else {
        GetLastError()
    }
}

#[no_mangle]
pub unsafe extern "system" fn NetApiBufferFree(Buffer: LPVOID) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/apibuf.c\0".as_ptr() as *const i8,
            b"NetApiBufferFree\0".as_ptr() as *const i8,
            45,
            b"(%p)\n\0".as_ptr() as *const i8,
            Buffer,
        );
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, Buffer as *mut c_void);
    NERR_Success as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetApiBufferReallocate(
    OldBuffer: LPVOID,
    NewByteCount: DWORD,
    NewBuffer: *mut LPVOID,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch = __wine_dbch___default;
        let __dbcl = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/apibuf.c\0"
                .as_ptr() as *const i8,
            b"NetApiBufferReallocate\0".as_ptr() as *const i8,
            56i32,
            b"(%p, %d, %p)\n\0".as_ptr() as *const i8,
            OldBuffer,
            NewByteCount,
            NewBuffer,
        );
    }
    if NewByteCount != 0 {
        if !OldBuffer.is_null() {
            *NewBuffer = HeapReAlloc(
                GetProcessHeap(),
                0u32 as DWORD,
                OldBuffer as PVOID,
                NewByteCount as SIZE_T,
            ) as LPVOID;
        } else {
            *NewBuffer = HeapAlloc(
                GetProcessHeap(),
                0u32 as DWORD,
                NewByteCount as SIZE_T,
            ) as LPVOID;
        }
        return if !(*NewBuffer).is_null() {
            NERR_Success as DWORD
        } else {
            GetLastError()
        };
    } else {
        if HeapFree(GetProcessHeap(), 0u32 as DWORD, OldBuffer as PVOID) == 0 {
            return GetLastError();
        }
        *NewBuffer = core::ptr::null_mut::<c_void>();
        return NERR_Success as DWORD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NetApiBufferSize(mut Buffer: LPVOID, mut ByteCount: LPDWORD) -> DWORD {
    let mut dw: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/apibuf.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetApiBufferSize\0" as *const u8 as *const ::core::ffi::c_char,
            81 as ::core::ffi::c_int,
            b"(%p, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            Buffer,
            ByteCount,
        );
    }
    if Buffer.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    dw = HeapSize(GetProcessHeap(), 0 as DWORD, Buffer as LPCVOID) as DWORD;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/netapi32/apibuf.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"NetApiBufferSize\0" as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
            b"size: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            dw,
        );
    }
    if dw != 0xffffffff as ::core::ffi::c_ulong {
        *ByteCount = dw;
    } else {
        *ByteCount = 0 as DWORD;
    }
    return NERR_Success as DWORD;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2002 Andriy Palamarchuk
//  *
//  * Net API buffer calls
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
// /************************************************************
//  *                NetApiBufferAllocate  (NETAPI32.@)
//  */
// NET_API_STATUS WINAPI NetApiBufferAllocate(DWORD ByteCount, LPVOID* Buffer)
// {
//     TRACE("(%d, %p)\n", ByteCount, Buffer);
//
//     if (Buffer == NULL) return ERROR_INVALID_PARAMETER;
//     *Buffer = HeapAlloc(GetProcessHeap(), 0, ByteCount);
//     if (*Buffer)
//         return NERR_Success;
//     else
//         return GetLastError();
// }
//
// /************************************************************
//  *                NetApiBufferFree  (NETAPI32.@)
//  */
// NET_API_STATUS WINAPI NetApiBufferFree(LPVOID Buffer)
// {
//     TRACE("(%p)\n", Buffer);
//     HeapFree(GetProcessHeap(), 0, Buffer);
//     return NERR_Success;
// }
//
// /************************************************************
//  *                NetApiBufferReallocate  (NETAPI32.@)
//  */
// NET_API_STATUS WINAPI NetApiBufferReallocate(LPVOID OldBuffer, DWORD NewByteCount,
//                                              LPVOID* NewBuffer)
// {
//     TRACE("(%p, %d, %p)\n", OldBuffer, NewByteCount, NewBuffer);
//     if (NewByteCount)
//     {
//         if (OldBuffer)
//             *NewBuffer = HeapReAlloc(GetProcessHeap(), 0, OldBuffer, NewByteCount);
//         else
//             *NewBuffer = HeapAlloc(GetProcessHeap(), 0, NewByteCount);
//         return *NewBuffer ? NERR_Success : GetLastError();
//     }
//     else
//     {
//         if (!HeapFree(GetProcessHeap(), 0, OldBuffer))
//             return GetLastError();
//         *NewBuffer = 0;
//         return NERR_Success;
//     }
// }
//
// /************************************************************
//  *                NetApiBufferSize  (NETAPI32.@)
//  */
// NET_API_STATUS WINAPI NetApiBufferSize(LPVOID Buffer, LPDWORD ByteCount)
// {
//     DWORD dw;
//
//     TRACE("(%p, %p)\n", Buffer, ByteCount);
//     if (Buffer == NULL)
//         return ERROR_INVALID_PARAMETER;
//     dw = HeapSize(GetProcessHeap(), 0, Buffer);
//     TRACE("size: %d\n", dw);
//     if (dw != 0xFFFFFFFF)
//         *ByteCount = dw;
//     else
//         *ByteCount = 0;
//
//     return NERR_Success;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
