use crate::prelude_rpc::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (standard netapi32 wine-debug channel; TRACE/FIXME reference it) ----
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


// ---- ATSVC_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn ATSVC_HANDLE_bind(
    pszSystemName: ATSVC_HANDLE,
) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    // Wide string literals
    let ncacn_np: [u16; 9] = [
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe_atsvc: [u16; 12] = [
        0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x0061, 0x0074, 0x0073, 0x0076, 0x0063,
        0x0000,
    ];

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as RPC_WSTR,
        pszSystemName as RPC_WSTR,
        pipe_atsvc.as_ptr() as RPC_WSTR,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(
        pszStringBinding as RPC_WSTR,
        &mut hBinding,
    );
    // Ignore status; if failed hBinding remains NULL

    status = RpcStringFreeW(&mut pszStringBinding);
    // Ignore status

    hBinding
}

// ---- ATSVC_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn ATSVC_HANDLE_unbind(
    pszSystemName: *mut u16,
    mut hBinding: *mut core::ffi::c_void,
) {
    let status = RpcBindingFree(&mut hBinding);
    if status != 0 {
        // nothing
    }
}

// ---- NetScheduleJobAdd ----
#[no_mangle]
pub unsafe extern "system" fn NetScheduleJobAdd(
    ServerName: LPCWSTR,
    Buffer: LPBYTE,
    JobId: LPDWORD,
) -> NET_API_STATUS {
    rpc_try(|| NetrJobAdd(ServerName, Buffer as *mut _AT_INFO, JobId)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetScheduleJobDel ----
#[no_mangle]
pub unsafe extern "system" fn NetScheduleJobDel(
    ServerName: LPCWSTR,
    MinJobId: DWORD,
    MaxJobId: DWORD,
) -> NET_API_STATUS {
    rpc_try(|| NetrJobDel(ServerName, MinJobId, MaxJobId)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- NetScheduleJobEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetScheduleJobEnum(
    ServerName: LPCWSTR,
    PointerToBuffer: *mut LPBYTE,
    PreferredMaximumLength: DWORD,
    EntriesRead: LPDWORD,
    TotalEntries: LPDWORD,
    ResumeHandle: LPDWORD,
) -> NET_API_STATUS {
    // The translation hand-rolled a local `AT_ENUM_CONTAINER { EntriesRead, Buffer: *mut u8 }`
    // that SHADOWED the real wire type. atsvc_c.h declares Buffer as LPAT_ENUM, so the local
    // copy both diverged from the marshaller's layout and made the NetrJobEnum call untypeable.
    // Use the bindgen type — a wire struct's layout is never hand-written.
    let mut enum_container = AT_ENUM_CONTAINER {
        EntriesRead: 0,
        Buffer: core::ptr::null_mut(),
    };

    let status = rpc_try(|| NetrJobEnum(
        ServerName,
        &mut enum_container as *mut AT_ENUM_CONTAINER,
        PreferredMaximumLength,
        TotalEntries,
        ResumeHandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == NERR_Success || status == ERROR_MORE_DATA {
        *PointerToBuffer = enum_container.Buffer as LPBYTE;
        *EntriesRead = enum_container.EntriesRead;
    }

    status
}

// ---- NetScheduleJobGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetScheduleJobGetInfo(
    ServerName: LPCWSTR,
    JobId: DWORD,
    PointerToBuffer: *mut LPBYTE,
) -> NET_API_STATUS {
    // schedule.c:187 casts the OUT buffer through: `(LPAT_INFO *)PointerToBuffer`.
    rpc_try(|| NetrJobGetInfo(ServerName, JobId, PointerToBuffer as *mut LPAT_INFO)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}
