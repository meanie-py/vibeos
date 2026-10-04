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


// ---- DsSetupBind ----
unsafe fn DsSetupBind(
    lpServerName: LPWSTR,
    hBinding: *mut handle_t,
) -> RPC_STATUS {
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    *hBinding = core::ptr::null_mut();

    // Wide string literals: "ncacn_np\" and "\\pipe\\lsarpc"
    let ncacn_np: [u16; 9] = [
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe_lsarpc: [u16; 14] = [
        0x005c, 0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x006c, 0x0073, 0x0061,
        0x0072, 0x0070, 0x0063, 0x0000,
    ];

    // RPC_WSTR is `*mut WCHAR`; the C passes wide string LITERALS here (an implicit
    // const-to-non-const the RPC runtime never writes through), so cast rather than copy.
    let mut status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        lpServerName,
        pipe_lsarpc.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return status;
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, hBinding);
    if status != 0 {
        // fall through
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {
        // fall through
    }

    status
}

// ---- DsSetupUnbind ----
unsafe fn DsSetupUnbind(
    mut hBinding: handle_t
) {
    // RpcBindingFree takes `RPC_BINDING_HANDLE*` and NULLs the caller's copy, so the local
    // must be `mut` to hand out a &mut — matching the C, which passes &hBinding by address.
    let status: RPC_STATUS = RpcBindingFree(&mut hBinding);
    if status != 0 {
        // nothing
    }
}

// ---- DsRoleFreeMemory ----
#[no_mangle]
pub unsafe extern "system" fn DsRoleFreeMemory(Buffer: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0u32, Buffer);
}

// ---- DsRoleGetPrimaryDomainInformation ----
#[no_mangle]
pub unsafe extern "system" fn DsRoleGetPrimaryDomainInformation(
    lpServer: LPCWSTR,
    InfoLevel: DSROLE_PRIMARY_DOMAIN_INFO_LEVEL,
    // C is `PBYTE* Buffer` (dssetup.c:95) — an OUT pointer the callee fills. The translation
    // dropped a level to `PBYTE`, which made `*Buffer = NULL` assign a u8 and the RPC out-param
    // cast bogus. Restored to the real pointer-to-pointer.
    Buffer: *mut PBYTE,
) -> DWORD {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut status: NET_API_STATUS;

    if Buffer.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if InfoLevel < DsRolePrimaryDomainInfoBasic || InfoLevel > DsRoleOperationState {
        return ERROR_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    // DsSetupBind returns RPC_STATUS (c_long); `status` is NET_API_STATUS (DWORD). The C
    // assigns across the same signed/unsigned boundary implicitly.
    status = DsSetupBind(lpServer as LPWSTR, &mut hBinding) as NET_API_STATUS;
    if status != 0 {
        return status;
    }

    // RpcTryExcept body (exception handler omitted)
    status = rpc_try(|| DsRolerGetPrimaryDomainInformation(
        hBinding,
        InfoLevel,
        Buffer as *mut PDSROLER_PRIMARY_DOMAIN_INFORMATION,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if !hBinding.is_null() {
        DsSetupUnbind(hBinding);
    }

    status
}
