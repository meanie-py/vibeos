// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12A6BF85117F: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_D929332961F9: [u16; 15] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x0070, 0x006C, 0x0075, 0x0067,
    0x0070, 0x006C, 0x0061, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_E4F2BEC36683: [u16; 4] = [0x0050u16, 0x004C, 0x0054, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut LocalBindingHandle: RPC_BINDING_HANDLE = NULL;

static mut LocalStringTable: HSTRING_TABLE = NULL;
#[no_mangle]
pub unsafe extern "C" fn PnpBindRpc(
    pszMachine: LPCWSTR,
    BindingHandle: *mut RPC_BINDING_HANDLE,
) -> RPC_STATUS {
    let mut pszStringBinding: PWSTR = core::ptr::null_mut();
    let mut Status: RPC_STATUS;

    Status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        RUSTCONV_WSTR_12A6BF85117F.as_ptr() as *mut u16,
        pszMachine as *mut u16,
        RUSTCONV_WSTR_D929332961F9.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(pszStringBinding),
    );
    if Status != RPC_S_OK {
        return Status;
    }

    Status = RpcBindingFromStringBindingW(pszStringBinding, BindingHandle);

    RpcStringFreeW(core::ptr::addr_of_mut!(pszStringBinding));

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn PnpUnbindRpc(BindingHandle: *mut RPC_BINDING_HANDLE) -> RPC_STATUS {
    if BindingHandle != core::ptr::null_mut() {
        RpcBindingFree((*BindingHandle).cast());
        *BindingHandle = core::ptr::null_mut();
    }

    return RPC_S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn PnpGetLocalHandles(
    BindingHandle: *mut RPC_BINDING_HANDLE,
    StringTable: *mut HSTRING_TABLE,
) -> BOOL {
    if !LocalBindingHandle.is_null() {
        if !BindingHandle.is_null() {
            *BindingHandle = LocalBindingHandle;
        }

        if !StringTable.is_null() {
            *StringTable = LocalStringTable;
        }

        return (TRUE) as BOOL;
    }

    LocalStringTable = pSetupStringTableInitialize();
    if LocalStringTable.is_null() {
        return (FALSE) as BOOL;
    }

    if PnpBindRpc(core::ptr::null(), core::ptr::addr_of_mut!(LocalBindingHandle)) != RPC_S_OK {
        pSetupStringTableDestroy(LocalStringTable);
        return (FALSE) as BOOL;
    }

    pSetupStringTableAddString(LocalStringTable, RUSTCONV_WSTR_E4F2BEC36683.as_ptr() as LPWSTR, 1);

    if !BindingHandle.is_null() {
        *BindingHandle = LocalBindingHandle;
    }

    if !StringTable.is_null() {
        *StringTable = LocalStringTable;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn PnpUnbindLocalBindingHandle() -> RPC_STATUS {
    pSetupStringTableDestroy(LocalStringTable);
    LocalStringTable = core::ptr::null_mut();
    return PnpUnbindRpc(core::ptr::addr_of_mut!(LocalBindingHandle));
}

#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as u32, len) as *mut core::ffi::c_void;
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 as DWORD, ptr);
}
