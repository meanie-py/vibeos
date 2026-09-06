use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn UsrSpInitUserModeContext(
    ContextHandle: LSA_SEC_HANDLE,
    PackedContext: PSecBuffer,
) -> NTSTATUS {
    // TRACE!("UsrSpInitUserModeContext(%p %p)\n", ContextHandle, PackedContext);
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Manage user mode contexts (create, destroy, reference)
//  * COPYRIGHT:   Copyright 2019-2020 Andreas Maier (staubim@quantentunnel.de)
//  */
//
// #include "precomp.h"
//
// #include "wine/debug.h"
// WINE_DEFAULT_DEBUG_CHANNEL(msv1_0);
//
// NTSTATUS
// NTAPI
// UsrSpInitUserModeContext(
//      LSA_SEC_HANDLE ContextHandle,
//      PSecBuffer PackedContext)
// {
//     TRACE("UsrSpInitUserModeContext(%p %p)\n",
//           ContextHandle, PackedContext);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
