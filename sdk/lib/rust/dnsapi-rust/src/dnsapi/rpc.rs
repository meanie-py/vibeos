use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// (C preamble is only preprocessor macros — no Rust declarations needed.)

// ---- DNSRSLVR_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn DNSRSLVR_HANDLE_bind(pszMachineName: *mut u16) -> *mut core::ffi::c_void {
    let mut hBinding: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pszStringBinding: *mut u16 = core::ptr::null_mut();
    let mut Status: i32;

    // DPRINT("DNSRSLVR_HANDLE_bind(%S)\n", pszMachineName);

    let ncalrpc: [u16; 8] = [0x006E, 0x0063, 0x0061, 0x006C, 0x0072, 0x0070, 0x0063, 0x0000];
    let dnsresolver: [u16; 12] = [
        0x0044, 0x004E, 0x0053, 0x0052, 0x0065, 0x0073, 0x006F, 0x006C, 0x0076, 0x0065, 0x0072, 0x0000,
    ];

    Status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncalrpc.as_ptr() as *mut u16,
        pszMachineName as *mut u16,
        dnsresolver.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(pszStringBinding),
    );
    if Status != 0 {
        // DPRINT1("RpcStringBindingCompose returned 0x%x\n", Status);
        return core::ptr::null_mut();
    }

    Status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if Status != 0 {
        // DPRINT1("RpcBindingFromStringBinding returned 0x%x\n", Status);
    }

    Status = RpcStringFreeW(core::ptr::addr_of_mut!(pszStringBinding));
    if Status != 0 {
        // DPRINT1("RpcStringFree returned 0x%x\n", Status);
    }

    hBinding
}

// ---- DNSRSLVR_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn DNSRSLVR_HANDLE_unbind(
    pszMachineName: DNSRSLVR_HANDLE,
    hBinding: handle_t,
) {
    let mut binding = hBinding;
    let Status = RpcBindingFree(&mut binding as *mut handle_t);
    if Status != 0 {
    }
}

// ---- midl_user_allocate ----
#[no_mangle]
pub unsafe extern "system" fn midl_user_allocate(len: usize) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}

// ---- midl_user_free ----
#[no_mangle]
pub unsafe extern "system" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

// dnsrslvr_c.c (the MIDL client stub) references the UPPERCASE MIDL_user_allocate/free@4 — the
// midl_user_allocate→MIDL_user_allocate #define isn't applied when the generated stub is compiled
// in this build. Provide the uppercase symbols the stub links against, forwarding to the
// translated lowercase definitions above (which mirror rpc.c's midl_user_allocate/free).
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: usize) -> *mut core::ffi::c_void {
    midl_user_allocate(len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    midl_user_free(ptr)
}
