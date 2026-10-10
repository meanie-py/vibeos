// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_13B54BA03C00: [u16; 8] = [0x006Eu16, 0x0063, 0x0061, 0x006C, 0x0072, 0x0070, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_2C21497BC39D: [u16; 8] = [0x0073u16, 0x0070, 0x006F, 0x006F, 0x006C, 0x0073, 0x0073, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn LrpcThreadProc(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS;

    Status = RpcServerUseProtseqEpW(RUSTCONV_WSTR_13B54BA03C00.as_ptr() as RPC_WSTR, 20, RUSTCONV_WSTR_2C21497BC39D.as_ptr() as RPC_WSTR, core::ptr::null_mut());
    if Status != RPC_S_OK {
        return 0;
    }

    Status = RpcServerRegisterIf(winspool_v1_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
    if Status != RPC_S_OK {
        return 0;
    }

    Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0);
    if Status != RPC_S_OK {
    }

    return 0;
}

#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut c_void {
    return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, len);
}
#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: LPVOID) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn WINSPOOL_GDI_HANDLE_rundown(hGdiHandle: WINSPOOL_GDI_HANDLE) {}

#[no_mangle]
pub unsafe extern "system" fn WINSPOOL_PRINTER_HANDLE_rundown(
    hPrinter: WINSPOOL_PRINTER_HANDLE,
) {}
