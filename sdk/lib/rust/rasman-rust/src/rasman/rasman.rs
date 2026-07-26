use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
static mut hDllInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogComplete(pSecMsg: *mut SECURITY_MESSAGE) {
    // FIXME!("(%p),stub!\n",pSecMsg);
}

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogBegin(
    hPort: HPORT,
    pSendBuf: PBYTE,
    SendBufSize: DWORD,
    pRecvBuf: PBYTE,
    RecvBufSize: DWORD,
    RasSecurityDialogComplete: Option<unsafe extern "system" fn(*mut SECURITY_MESSAGE)>,
) -> DWORD {
    return NO_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogSend(
    hPort: HPORT,
    pBuffer: PBYTE,
    BufferLength: WORD,
) -> DWORD {
    // FIXME!("(%p,%p,%x),stub!\n",hPort, pBuffer, BufferLength);
    NO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogReceive(
    hPort: HPORT,
    pBuffer: PBYTE,
    pBufferLength: PWORD,
    Timeout: DWORD,
    hEvent: HANDLE,
) -> DWORD {
    // FIXME!("(%p,%p,%p,%x,%p),stub!\n",hPort, pBuffer, pBufferLength, Timeout, hEvent);
    NO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogEnd(hPort: HPORT) -> DWORD {
    // FIXME!("(%p),stub!\n",hPort);
    NO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn RasSecurityDialogGetInfo(
    hPort: HPORT,
    pBuffer: *mut RAS_SECURITY_INFO,
) -> DWORD {
    // FIXME!("(%p,%p),stub!\n",hPort,pBuffer);
    NO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;
            DisableThreadLibraryCalls(hinstDLL);
        }
        _ => {}
    }

    (TRUE) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * RASMAN
//  *
//  * Copyright 2007 ReactOS Team
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
// #define WIN32_NO_STATUS
//
// #include <stdarg.h>
//
// #include <windef.h>
// #include <winbase.h>
// //#include <ras.h>
// #include <wine/debug.h>
// #include <rasshost.h>
//
// static HINSTANCE hDllInstance;
//
// WINE_DEFAULT_DEBUG_CHANNEL(rasman);
//
// VOID WINAPI
// RasSecurityDialogComplete(SECURITY_MESSAGE* pSecMsg)
// {
//     FIXME("(%p),stub!\n",pSecMsg);
// }
//
// DWORD WINAPI
// RasSecurityDialogBegin(HPORT hPort, PBYTE pSendBuf,
//     DWORD SendBufSize, PBYTE pRecvBuf, DWORD RecvBufSize,
//     VOID (WINAPI *RasSecurityDialogComplete)(SECURITY_MESSAGE*))
// {
//     FIXME("(%p,%p,0x%08x,%p,0x%08x,%p),stub!\n",hPort,pSendBuf,SendBufSize,
//           pRecvBuf,RecvBufSize,*RasSecurityDialogComplete);
//     return NO_ERROR;
// }
//
// DWORD WINAPI
// RasSecurityDialogSend(HPORT hPort, PBYTE pBuffer, WORD BufferLength)
// {
//     FIXME("(%p,%p,%x),stub!\n",hPort, pBuffer, BufferLength);
//     return NO_ERROR;
// }
//
// DWORD WINAPI
// RasSecurityDialogReceive(HPORT hPort, PBYTE pBuffer, PWORD pBufferLength, DWORD Timeout, HANDLE hEvent)
// {
//     FIXME("(%p,%p,%p,%x,%p),stub!\n",hPort, pBuffer, pBufferLength, Timeout, hEvent);
//     return NO_ERROR;
// }
//
// DWORD WINAPI
// RasSecurityDialogEnd(HPORT hPort)
// {
//     FIXME("(%p),stub!\n",hPort);
//     return NO_ERROR;
// }
//
// DWORD WINAPI
// RasSecurityDialogGetInfo(HPORT hPort, RAS_SECURITY_INFO* pBuffer)
// {
//     FIXME("(%p,%p),stub!\n",hPort,pBuffer);
//     return NO_ERROR;
// }
//
// BOOL WINAPI
// DllMain( HINSTANCE hinstDLL,
//          DWORD dwReason,
//          LPVOID lpvReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             hDllInstance = hinstDLL;
//             DisableThreadLibraryCalls(hinstDLL);
//             break;
//     }
//
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
