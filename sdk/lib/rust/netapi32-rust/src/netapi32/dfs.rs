use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

#[no_mangle]
pub unsafe extern "system" fn NetDfsAdd(
    DfsEntryPath: LPWSTR,
    ServerName: LPWSTR,
    PathName: LPWSTR,
    Comment: LPWSTR,
    Flags: u32,
) -> u32 {
    // UNIMPLEMENTED macro expansion
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsAdd\0".as_ptr() as *const core::ffi::c_char,
            60,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsAdd\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsAddFtRoot(
    ServerName: LPWSTR,
    RootShare: LPWSTR,
    FtDfsName: LPWSTR,
    Comment: LPWSTR,
    Flags: DWORD,
) -> DWORD {
    if ((*__wine_dbch___default).flags as i32)
        & (1i32 << (__WINE_DBCL_FIXME as i32))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsAddFtRoot\0".as_ptr() as *const i8,
            74i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsAddFtRoot\0".as_ptr() as *const i8,
        );
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsAddStdRoot(
    server_name: LPWSTR,
    root_share: LPWSTR,
    comment: LPWSTR,
    flags: DWORD,
) -> DWORD {
    let dbch = __wine_dbch___default;
    if (*dbch).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            dbch as *mut _,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsAddStdRoot\0".as_ptr() as *const core::ffi::c_char,
            87i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsAddStdRoot\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsAddStdRootForced(
    _ServerName: LPWSTR,
    _RootShare: LPWSTR,
    _Comment: LPWSTR,
    _Store: LPWSTR,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dfs.c\0".as_ptr() as *const i8,
            b"NetDfsAddStdRootForced\0".as_ptr() as *const i8,
            100i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsAddStdRootForced\0".as_ptr() as *const i8,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsEnum(
    mut DfsName: LPWSTR,
    mut Level: DWORD,
    mut PrefMaxLen: DWORD,
    mut Buffer: *mut LPBYTE,
    mut EntriesRead: LPDWORD,
    mut ResumeHandle: LPDWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/dfs.c\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsEnum\0".as_ptr() as *const ::core::ffi::c_char,
            115i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsEnum\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsGetClientInfo(
    mut DfsEntryPath: LPWSTR,
    mut ServerName: LPWSTR,
    mut ShareName: LPWSTR,
    mut Level: DWORD,
    mut Buffer: *mut LPBYTE,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsGetClientInfo\0".as_ptr() as *const ::core::ffi::c_char,
            129i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsGetClientInfo\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsGetDcAddress(
    ServerName: LPWSTR,
    DcIpAddress: *mut LPWSTR,
    IsRoot: PBOOLEAN,
    Timeout: PULONG,
) -> DWORD {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1 << (__WINE_DBCL_FIXME as core::ffi::c_int)))
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsGetDcAddress\0".as_ptr() as *const core::ffi::c_char,
            142i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsGetDcAddress\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn NetDfsGetFtContainerSecurity(
    DomainName: LPWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    lpcbSecurityDescriptor: LPDWORD,
) -> u32 {
    if ((*__wine_dbch___default).flags as i32
        & (1 << __WINE_DBCL_FIXME as i32))
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            unsafe { __wine_dbch___default },
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsGetFtContainerSecurity\0".as_ptr() as *const i8,
            155i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsGetFtContainerSecurity\0".as_ptr() as *const i8,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsGetInfo(
    DfsEntryPath: LPWSTR,
    ServerName: LPWSTR,
    ShareName: LPWSTR,
    Level: DWORD,
    Buffer: *mut LPBYTE,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const c_char,
            b"NetDfsGetInfo\0".as_ptr() as *const c_char,
            169i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const c_char,
            b"NetDfsGetInfo\0".as_ptr() as *const c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsGetSecurity(
    _DfsEntryPath: LPWSTR,
    _SecurityInformation: SECURITY_INFORMATION,
    _ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    _lpcbSecurityDescriptor: LPDWORD,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsGetSecurity\0".as_ptr() as *const i8,
            182i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsGetSecurity\0".as_ptr() as *const i8,
        );
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsGetStdContainerSecurity(
    MachineName: LPWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    lpcbSecurityDescriptor: LPDWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            dbcl,
            dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsGetStdContainerSecurity\0".as_ptr() as *const ::core::ffi::c_char,
            195 as ::core::ffi::c_int,
            b"%s is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsGetStdContainerSecurity\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsManagerInitialize(
    ServerName: LPWSTR,
    Flags: DWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsManagerInitialize\0".as_ptr() as *const i8,
            209i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsManagerInitialize\0".as_ptr() as *const i8,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsMove(
    Path: LPWSTR,
    NewPath: LPWSTR,
    Flags: ULONG,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsMove\0".as_ptr() as *const core::ffi::c_char,
            224i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsMove\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsRemove(
    DfsEntryPath: LPWSTR,
    ServerName: LPWSTR,
    ShareName: LPWSTR,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsRemove\0".as_ptr() as *const ::core::ffi::c_char,
            236i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsRemove\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsRemoveFtRoot(
    ServerName: LPWSTR,
    RootShare: LPWSTR,
    FtDfsName: LPWSTR,
    Flags: DWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsRemoveFtRoot\0".as_ptr() as *const ::core::ffi::c_char,
            249i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"NetDfsRemoveFtRoot\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsRemoveFtRootForced(
    DomainName: LPWSTR,
    ServerName: LPWSTR,
    RootShare: LPWSTR,
    FtDfsName: LPWSTR,
    Flags: u32,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0".as_ptr() as *const i8,
            b"NetDfsRemoveFtRootForced\0".as_ptr() as *const i8,
            263i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsRemoveFtRootForced\0".as_ptr() as *const i8,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsRemoveStdRoot(
    ServerName: *mut u16,
    RootShare: *mut u16,
    Flags: u32,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsRemoveStdRoot\0".as_ptr() as *const i8,
            275i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsRemoveStdRoot\0".as_ptr() as *const i8,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsRename(Path: *mut u16, NewPath: *mut u16) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsRename\0".as_ptr() as *const core::ffi::c_char,
            286i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsRename\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsSetClientInfo(
    DfsEntryPath: LPWSTR,
    ServerName: LPWSTR,
    ShareName: LPWSTR,
    Level: u32,
    Buffer: LPBYTE,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetClientInfo\0".as_ptr() as *const core::ffi::c_char,
            300i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetClientInfo\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsSetFtContainerSecurity(
    DomainName: LPWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> u32 {
    let dbch = __wine_dbch___default;
    if ((*dbch).flags as i32) & (1i32 << (__WINE_DBCL_FIXME as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetFtContainerSecurity\0".as_ptr() as *const core::ffi::c_char,
            312i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetFtContainerSecurity\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsSetInfo(
    DfsEntryPath: LPWSTR,
    ServerName: LPWSTR,
    ShareName: LPWSTR,
    Level: u32,
    Buffer: LPBYTE,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetInfo\0".as_ptr() as *const core::ffi::c_char,
            326i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetInfo\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsSetSecurity(
    DfsEntryPath: LPWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const i8,
            b"NetDfsSetSecurity\0".as_ptr() as *const i8,
            338i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const i8,
            b"NetDfsSetSecurity\0".as_ptr() as *const i8,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn NetDfsSetStdContainerSecurity(
    MachineName: LPWSTR,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"dll/win32/netapi32/dfs.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetStdContainerSecurity\0".as_ptr() as *const core::ffi::c_char,
            350i32,
            b"%s is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
            b"NetDfsSetStdContainerSecurity\0".as_ptr() as *const core::ffi::c_char,
        );
    }
    0u32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         NetAPI DLL
//  * FILE:            reactos/dll/win32/netapi32/dfs.c
//  * PURPOSE:         Distributed File System Service interface code
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "netapi32.h"
//
// #include <rpc.h>
// #include "netdfs_c.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// /* FUNCTIONS *****************************************************************/
//
// NET_API_STATUS
// WINAPI
// NetDfsAdd(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR PathName,
//      LPWSTR Comment,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsAddFtRoot(
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      LPWSTR FtDfsName,
//      LPWSTR Comment,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsAddStdRoot(
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      LPWSTR Comment,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsAddStdRootForced(
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      LPWSTR Comment,
//      LPWSTR Store)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsEnum(
//      LPWSTR DfsName,
//      DWORD Level,
//      DWORD PrefMaxLen,
//      LPBYTE *Buffer,
//      LPDWORD EntriesRead,
//      LPDWORD ResumeHandle)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetClientInfo(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR ShareName,
//      DWORD Level,
//      LPBYTE *Buffer)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetDcAddress(
//      LPWSTR ServerName,
//      LPWSTR *DcIpAddress,
//      PBOOLEAN IsRoot,
//      PULONG Timeout)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetFtContainerSecurity(
//      LPWSTR DomainName,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR *ppSecurityDescriptor,
//      LPDWORD lpcbSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetInfo(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR ShareName,
//      DWORD Level,
//      LPBYTE *Buffer)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetSecurity(
//      LPWSTR DfsEntryPath,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR *ppSecurityDescriptor,
//      LPDWORD lpcbSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsGetStdContainerSecurity(
//      LPWSTR MachineName,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR *ppSecurityDescriptor,
//      LPDWORD lpcbSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* NetDfsManagerGetConfigInfo */
//
//
// NET_API_STATUS
// WINAPI
// NetDfsManagerInitialize(
//      LPWSTR ServerName,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* NetDfsManagerSendSiteInfo */
//
//
// NET_API_STATUS
// WINAPI
// NetDfsMove(
//      LPWSTR Path,
//      LPWSTR NewPath,
//      ULONG Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsRemove(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR ShareName)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsRemoveFtRoot(
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      LPWSTR FtDfsName,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsRemoveFtRootForced(
//      LPWSTR DomainName,
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      LPWSTR FtDfsName,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsRemoveStdRoot(
//      LPWSTR ServerName,
//      LPWSTR RootShare,
//      DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsRename(
//      LPWSTR Path,
//      LPWSTR NewPath)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsSetClientInfo(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR ShareName,
//      DWORD Level,
//      LPBYTE Buffer)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsSetFtContainerSecurity(
//      LPWSTR DomainName,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsSetInfo(
//      LPWSTR DfsEntryPath,
//      LPWSTR ServerName,
//      LPWSTR ShareName,
//      DWORD Level,
//      LPBYTE Buffer)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsSetSecurity(
//      LPWSTR DfsEntryPath,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetDfsSetStdContainerSecurity(
//      LPWSTR MachineName,
//      SECURITY_INFORMATION SecurityInformation,
//      PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
