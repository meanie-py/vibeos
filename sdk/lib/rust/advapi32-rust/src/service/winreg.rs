use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut __wine_dbch_advapi: () = ();
#[no_mangle]
pub unsafe extern "system" fn PREGISTRY_SERVER_NAME_bind(pszServerName: PREGISTRY_SERVER_NAME) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        b"n\0c\0a\0c\0n\0_\0n\0p\0\0\0".as_ptr() as LPWSTR,
        pszServerName,
        b"\\\0p\0i\0p\0e\0\\\0w\0i\0n\0r\0e\0g\0\0\0".as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != RPC_S_OK as RPC_STATUS {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != RPC_S_OK as RPC_STATUS {}

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != RPC_S_OK as RPC_STATUS {}

    hBinding
}
#[no_mangle]
pub unsafe extern "system" fn PREGISTRY_SERVER_NAME_unbind(
    pszServerName: PREGISTRY_SERVER_NAME,
    mut hBinding: handle_t,
) {
    let status: RPC_STATUS;

    status = RpcBindingFree(core::ptr::addr_of_mut!(hBinding));
    if status != RPC_S_OK {
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/service/winreg.c
//  * PURPOSE:         Remote registry functions
//  * PROGRAMMER:      Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
//
// /* FUNCTIONS *****************************************************************/
//
//
// handle_t __RPC_USER
// PREGISTRY_SERVER_NAME_bind(PREGISTRY_SERVER_NAME pszServerName)
// {
//     handle_t hBinding = NULL;
//     LPWSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("PREGISTRY_SERVER_NAME_bind() called\n");
//
//     status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       pszServerName,
//                                       L"\\pipe\\winreg",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringBindingCompose returned 0x%x\n", status);
//         return NULL;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeW(&pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return hBinding;
// }
//
//
// void __RPC_USER
// PREGISTRY_SERVER_NAME_unbind(PREGISTRY_SERVER_NAME pszServerName,
//                              handle_t hBinding)
// {
//     RPC_STATUS status;
//
//     TRACE("PREGISTRY_SERVER_NAME_unbind() called\n");
//
//     status = RpcBindingFree(&hBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFree returned 0x%x\n", status);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
