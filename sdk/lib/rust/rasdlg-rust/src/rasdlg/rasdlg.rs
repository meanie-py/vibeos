use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
static mut hDllInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn RasDialDlgA(
    lpszPhonebook: LPSTR,
    lpszEntry: LPSTR,
    lpszPhoneNumber: LPSTR,
    lpInfo: LPRASDIALDLG,
) -> BOOL {
    // FIXME!("(%s,%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpszPhoneNumber,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDialDlgW(
    lpszPhonebook: LPWSTR,
    lpszEntry: LPWSTR,
    lpszPhoneNumber: LPWSTR,
    lpInfo: LPRASDIALDLG,
) -> BOOL {
    // FIXME!("(%s,%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpszPhoneNumber,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasMonitorDlgA(
    lpszDeviceName: LPSTR,
    lpInfo: LPRASMONITORDLG,
) -> BOOL {
    // FIXME!("(%s,%p),stub!\n",lpszDeviceName,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasMonitorDlgW(
    lpszDeviceName: LPWSTR,
    lpInfo: LPRASMONITORDLG,
) -> BOOL {
    // FIXME!("(%s,%p),stub!\n",lpszDeviceName,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEntryDlgA(
    lpszPhonebook: LPSTR,
    lpszEntry: LPSTR,
    lpInfo: LPRASENTRYDLGA,
) -> BOOL {
    // FIXME!("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEntryDlgW(
    lpszPhonebook: LPWSTR,
    lpszEntry: LPWSTR,
    lpInfo: LPRASENTRYDLGW,
) -> BOOL {
    // FIXME!("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasPhonebookDlgA(
    lpszPhonebook: LPSTR,
    lpszEntry: LPSTR,
    lpInfo: LPRASPBDLGA,
) -> BOOL {
    // FIXME!("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasPhonebookDlgW(
    lpszPhonebook: LPWSTR,
    lpszEntry: LPWSTR,
    lpInfo: LPRASPBDLGW,
) -> BOOL {
    // FIXME!("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
    0
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
//  * RASDLG
//  *
//  * Copyright 2007 Dmitry Chapyshev
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
// #include <rasdlg.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(ras);
//
// BOOL WINAPI
// RasDialDlgA(LPSTR lpszPhonebook, LPSTR lpszEntry, LPSTR lpszPhoneNumber, LPRASDIALDLG lpInfo)
// {
// 	FIXME("(%s,%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpszPhoneNumber,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasDialDlgW(LPWSTR lpszPhonebook, LPWSTR lpszEntry, LPWSTR lpszPhoneNumber, LPRASDIALDLG lpInfo)
// {
// 	FIXME("(%s,%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpszPhoneNumber,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasMonitorDlgA(LPSTR lpszDeviceName, LPRASMONITORDLG lpInfo)
// {
// 	FIXME("(%s,%p),stub!\n",lpszDeviceName,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasMonitorDlgW(LPWSTR lpszDeviceName, LPRASMONITORDLG lpInfo)
// {
// 	FIXME("(%s,%p),stub!\n",lpszDeviceName,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasEntryDlgA(LPSTR lpszPhonebook, LPSTR lpszEntry, LPRASENTRYDLGA lpInfo)
// {
// 	FIXME("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasEntryDlgW(LPWSTR lpszPhonebook, LPWSTR lpszEntry, LPRASENTRYDLGW lpInfo)
// {
// 	FIXME("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasPhonebookDlgA(LPSTR lpszPhonebook, LPSTR lpszEntry, LPRASPBDLGA lpInfo)
// {
// 	FIXME("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
// 	return 0;
// }
//
// BOOL WINAPI
// RasPhonebookDlgW(LPWSTR lpszPhonebook, LPWSTR lpszEntry, LPRASPBDLGW lpInfo)
// {
// 	FIXME("(%s,%s,%p),stub!\n",lpszPhonebook,lpszEntry,lpInfo);
// 	return 0;
// }
//
// static HINSTANCE hDllInstance;
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
