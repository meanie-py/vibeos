// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_13B54BA03C00: [u16; 8] = [0x006Eu16, 0x0063, 0x0061, 0x006C, 0x0072, 0x0070, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_2C21497BC39D: [u16; 8] = [0x0073u16, 0x0070, 0x006F, 0x006F, 0x006C, 0x0073, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hinstWinSpool: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut rtlCritSec: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};
#[no_mangle]
pub unsafe extern "system" fn WINSPOOL_HANDLE_bind(wszName: WINSPOOL_HANDLE) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut wszStringBinding: PWSTR = core::ptr::null_mut();
    let mut Status: RPC_STATUS = 0;

    // Get us a string binding handle from the supplied connection information
    Status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        RUSTCONV_WSTR_13B54BA03C00.as_ptr() as RPC_WSTR,
        core::ptr::null_mut(),
        RUSTCONV_WSTR_2C21497BC39D.as_ptr() as RPC_WSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(wszStringBinding),
    );
    if Status != RPC_S_OK {
        return core::ptr::null_mut();
    }

    // Get a handle_t binding handle from the string binding handle
    Status = RpcBindingFromStringBindingW(
        wszStringBinding as RPC_WSTR,
        core::ptr::addr_of_mut!(hBinding),
    );
    if Status != RPC_S_OK {
        return core::ptr::null_mut();
    }

    // Free the string binding handle
    Status = RpcStringFreeW(core::ptr::addr_of_mut!(wszStringBinding));
    if Status != RPC_S_OK {
        return core::ptr::null_mut();
    }

    return hBinding;
}

#[no_mangle]
pub unsafe extern "system" fn WINSPOOL_HANDLE_unbind(mut wszName: WINSPOOL_HANDLE, mut hBinding: handle_t) {
    let _ = wszName;
    let mut Status: RPC_STATUS;
    Status = RpcBindingFree(core::ptr::addr_of_mut!(hBinding));
    if Status != RPC_S_OK {
    }
}

#[export_name = "MIDL_user_allocate"]
pub unsafe extern "system" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    return HeapAlloc(hProcessHeap, HEAP_ZERO_MEMORY as DWORD, len) as *mut core::ffi::c_void;
}

#[export_name = "MIDL_user_free"]
pub unsafe extern "system" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(hProcessHeap, 0, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hProcessHeap = GetProcessHeap();
            hinstWinSpool = hinstDLL;
            InitializeCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));
        }
        DLL_PROCESS_DETACH => {
            DeleteCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));
        }
        _ => {}
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SpoolerInit() -> BOOL {
    let mut bReturnValue: BOOL = FALSE as BOOL;

    // RpcTryExcept { .. } RpcExcept(EXCEPTION_EXECUTE_HANDLER) { ERR(..) }: a raise leaves
    // bReturnValue FALSE and the last error untouched.
    if let Ok(dwErrorCode) = ntdll_sys::seh::rpc_try(|| _RpcSpoolerInit()) {
        SetLastError(dwErrorCode);
        bReturnValue = (dwErrorCode == ERROR_SUCCESS) as BOOL;
    }

    return bReturnValue;
}
