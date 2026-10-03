use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn RasConnectionNotificationA(
    hrasconn: HRASCONN,
    hEvent: HANDLE,
    dwFlags: DWORD,
) -> DWORD {
    // FIXME!("(%p,%p,0x%08x),stub!\n",hrasconn,hEvent,dwFlags);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasConnectionNotificationW(
    hrasconn: HRASCONN,
    hEvent: HANDLE,
    dwFlags: DWORD,
) -> DWORD {
    // FIXME!("(%p,%p,0x%08x),stub!\n",hrasconn,hEvent,dwFlags);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasCreatePhonebookEntryA(
    hwnd: HWND,
    lpszPhonebook: LPCSTR,
) -> DWORD {
    // FIXME!("(%p,%s),stub!\n",hwnd,debugstr_a(lpszPhonebook));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasCreatePhonebookEntryW(
    hwnd: HWND,
    lpszPhonebook: LPCWSTR,
) -> DWORD {
    // FIXME!("(%p,%s),stub!\n",hwnd,debugstr_w(lpszPhonebook));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDeleteSubEntryA(
    lpszPhonebook: LPCSTR,
    lpszEntry: LPCSTR,
    dwSubEntryId: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,0x%08x),stub!\n",debugstr_a(lpszPhonebook), debugstr_a(lpszEntry),dwSubEntryId);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDeleteSubEntryW(
    lpszPhonebook: LPCWSTR,
    lpszEntry: LPCWSTR,
    dwSubEntryId: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,0x%08x),stub!\n",debugstr_w(lpszPhonebook), debugstr_w(lpszEntry),dwSubEntryId);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDialA(
    lpRasDialExtensions: LPRASDIALEXTENSIONS,
    lpszPhonebook: LPCSTR,
    lpRasDialParams: LPRASDIALPARAMSA,
    dwNotifierType: DWORD,
    lpvNotifier: LPVOID,
    lphRasConn: LPHRASCONN,
) -> DWORD {
    // FIXME!("(%p,%s,%p,0x%08x,%p,%p),stub!\n",lpRasDialExtensions,debugstr_a(lpszPhonebook), lpRasDialParams,dwNotifierType,lpvNotifier,lphRasConn);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasDialW(
    lpRasDialExtensions: LPRASDIALEXTENSIONS,
    lpszPhonebook: LPCWSTR,
    lpRasDialParams: LPRASDIALPARAMSW,
    dwNotifierType: DWORD,
    lpvNotifier: LPVOID,
    lphRasConn: LPHRASCONN,
) -> DWORD {
    // FIXME!("(%p,%s,%p,0x%08x,%p,%p),stub!\n",lpRasDialExtensions,debugstr_w(lpszPhonebook), lpRasDialParams,dwNotifierType,lpvNotifier,lphRasConn);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasEditPhonebookEntryA(
    hwnd: HWND,
    lpszPhonebook: LPCSTR,
    lpszEntryName: LPCSTR,
) -> DWORD {
    // FIXME!("(%p,%s,%s),stub!\n",hwnd,debugstr_a(lpszPhonebook),debugstr_a(lpszEntryName));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEditPhonebookEntryW(
    hwnd: HWND,
    lpszPhonebook: LPCWSTR,
    lpszEntryName: LPCWSTR,
) -> DWORD {
    // FIXME!("(%p,%s,%s),stub!\n",hwnd,debugstr_w(lpszPhonebook),debugstr_w(lpszEntryName));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumConnectionsA(
    rca: LPRASCONNA,
    lpcb: LPDWORD,
    lpcConnections: LPDWORD,
) -> DWORD {
    *lpcb = 0;
    *lpcConnections = 0;

    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumConnectionsW(
    rcw: LPRASCONNW,
    lpcb: LPDWORD,
    lpcConnections: LPDWORD,
) -> DWORD {
    *lpcb = 0;
    *lpcConnections = 0;

    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumEntriesA(
    Reserved: LPCSTR,
    lpszPhoneBook: LPCSTR,
    lpRasEntryName: LPRASENTRYNAMEA,
    lpcb: LPDWORD,
    lpcEntries: LPDWORD,
) -> DWORD {
    *lpcEntries = 0;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumEntriesW(
    Reserved: LPCWSTR,
    lpszPhoneBook: LPCWSTR,
    lpRasEntryName: LPRASENTRYNAMEW,
    lpcb: LPDWORD,
    lpcEntries: LPDWORD,
) -> DWORD {
    *lpcEntries = 0;
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetConnectStatusA(
    hrasconn: HRASCONN,
    lprasconnstatus: LPRASCONNSTATUSA,
) -> DWORD {
    // FIXME!("(%p,%p),stub!\n",hrasconn,lprasconnstatus);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetConnectStatusW(
    hrasconn: HRASCONN,
    lprasconnstatus: LPRASCONNSTATUSW,
) -> DWORD {
    // FIXME!("(%p,%p),stub!\n",hrasconn,lprasconnstatus);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetEntryDialParamsA(
    lpszPhoneBook: LPCSTR,
    lpRasDialParams: LPRASDIALPARAMSA,
    lpfPassword: LPBOOL,
) -> DWORD {
    // FIXME!("(%s,%p,%p),stub!\n",debugstr_a(lpszPhoneBook), lpRasDialParams,lpfPassword);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetEntryDialParamsW(
    lpszPhoneBook: LPCWSTR,
    lpRasDialParams: LPRASDIALPARAMSW,
    lpfPassword: LPBOOL,
) -> DWORD {
    // FIXME!("(%s,%p,%p),stub!\n",debugstr_w(lpszPhoneBook), lpRasDialParams,lpfPassword);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasHangUpA(hrasconn: HRASCONN) -> DWORD {
    // FIXME!("(%p),stub!\n",hrasconn);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasHangUpW(hrasconn: HRASCONN) -> DWORD {
    // FIXME!("(%p),stub!\n",hrasconn);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDeleteEntryA(a: LPCSTR, b: LPCSTR) -> DWORD {
    // FIXME!("(%s,%s),stub!\n",debugstr_a(a),debugstr_a(b));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasDeleteEntryW(a: LPCWSTR, b: LPCWSTR) -> DWORD {
    // FIXME!("(%s,%s),stub!\n",debugstr_w(a),debugstr_w(b));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumAutodialAddressesA(
    a: *mut LPSTR,
    b: LPDWORD,
    c: LPDWORD,
) -> DWORD {
    // FIXME!("(%p,%p,%p),stub!\n",a,b,c);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumAutodialAddressesW(
    a: *mut LPWSTR,
    b: LPDWORD,
    c: LPDWORD,
) -> DWORD {
    // FIXME!("(%p,%p,%p),stub!\n",a,b,c);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumDevicesA(
    lpRasDevinfo: LPRASDEVINFOA,
    lpcb: LPDWORD,
    lpcDevices: LPDWORD,
) -> DWORD {
    if lpcb.is_null() || lpcDevices.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if !lpRasDevinfo.is_null() && (*lpRasDevinfo).dwSize != core::mem::size_of::<RASDEVINFOA>() as u32 {
        return ERROR_INVALID_SIZE;
    }

    *lpcDevices = 1;

    if lpRasDevinfo.is_null() || (*lpcb < core::mem::size_of::<RASDEVINFOA>() as u32) {
        *lpcb = core::mem::size_of::<RASDEVINFOA>() as u32;
        return ERROR_BUFFER_TOO_SMALL;
    }

    strcpy((*lpRasDevinfo).szDeviceType.as_mut_ptr(), RASDT_Modem);
    strcpy((*lpRasDevinfo).szDeviceName.as_mut_ptr(), b"WINE virtmodem\0".as_ptr() as *const i8);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn RasEnumDevicesW(
    a: LPRASDEVINFOW,
    b: LPDWORD,
    c: LPDWORD,
) -> DWORD {
    // FIXME!("(%p,%p,%p),stub!\n",a,b,c);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialAddressA(
    a: LPCSTR,
    b: LPDWORD,
    c: LPRASAUTODIALENTRYA,
    d: LPDWORD,
    e: LPDWORD,
) -> DWORD {
    // FIXME!("(%s,%p,%p,%p,%p),stub!\n",debugstr_a(a),b,c,d,e);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialAddressW(
    a: LPCWSTR,
    b: LPDWORD,
    c: LPRASAUTODIALENTRYW,
    d: LPDWORD,
    e: LPDWORD,
) -> DWORD {
    // FIXME!("(%s,%p,%p,%p,%p),stub!\n",debugstr_w(a),b,c,d,e);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialEnableA(a: DWORD, b: LPBOOL) -> DWORD {
    // FIXME!("(%x,%p),stub!\n",a,b);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialEnableW(a: DWORD, b: LPBOOL) -> DWORD {
    // FIXME!("(%x,%p),stub!\n",a,b);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialParamA(
    dwKey: DWORD,
    lpvValue: LPVOID,
    lpdwcbValue: LPDWORD,
) -> DWORD {
    // FIXME!("(%x,%p,%p),stub!\n",dwKey,lpvValue,lpdwcbValue);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetAutodialParamW(
    dwKey: DWORD,
    lpvValue: LPVOID,
    lpdwcbValue: LPDWORD,
) -> DWORD {
    // FIXME!("(%x,%p,%p),stub!\n",dwKey,lpvValue,lpdwcbValue);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialAddressA(
    a: LPCSTR,
    b: DWORD,
    c: LPRASAUTODIALENTRYA,
    d: DWORD,
    e: DWORD,
) -> DWORD {
    // FIXME!("(%s,%x,%p,%x,%x),stub!\n",debugstr_a(a),b,c,d,e);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialAddressW(
    a: LPCWSTR,
    b: DWORD,
    c: LPRASAUTODIALENTRYW,
    d: DWORD,
    e: DWORD,
) -> DWORD {
    // FIXME!("(%s,%x,%p,%x,%x),stub!\n",debugstr_w(a),b,c,d,e);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialEnableA(
    dwDialingLocation: DWORD,
    fEnabled: BOOL,
) -> DWORD {
    // FIXME!("(%x,%x),stub!\n",dwDialingLocation,fEnabled);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialEnableW(
    dwDialingLocation: DWORD,
    fEnabled: BOOL,
) -> DWORD {
    // FIXME!("(%x,%x),stub!\n",dwDialingLocation,fEnabled);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialParamA(a: DWORD, b: LPVOID, c: DWORD) -> DWORD {
    // FIXME!("(%x,%p,%x),stub!\n",a,b,c);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetAutodialParamW(a: DWORD, b: LPVOID, c: DWORD) -> DWORD {
    // FIXME!("(%x,%p,%x),stub!\n",a,b,c);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetEntryPropertiesA(
    lpszPhonebook: LPCSTR,
    lpszEntry: LPCSTR,
    lpRasEntry: LPRASENTRYA,
    dwEntryInfoSize: DWORD,
    lpbDeviceInfo: LPBYTE,
    dwDeviceInfoSize: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,%d,%p,%d), stub!\n", debugstr_a(lpszPhonebook),debugstr_a(lpszEntry), lpRasEntry,dwEntryInfoSize,lpbDeviceInfo,dwDeviceInfoSize );
    // FIXME!("Rasentry:\n");
    // FIXME!("\tdwfOptions %x\n",lpRasEntry->dwfOptions);
    // FIXME!("\tszLocalPhoneNumber %s\n",debugstr_a(lpRasEntry->szLocalPhoneNumber));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetEntryPropertiesW(
    lpszPhonebook: LPCWSTR,
    lpszEntry: LPCWSTR,
    lpRasEntry: LPRASENTRYW,
    dwEntryInfoSize: DWORD,
    lpbDeviceInfo: LPBYTE,
    dwDeviceInfoSize: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,%d,%p,%d), stub!\n", debugstr_w(lpszPhonebook),debugstr_w(lpszEntry), lpRasEntry,dwEntryInfoSize,lpbDeviceInfo,dwDeviceInfoSize );
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasValidateEntryNameA(
    lpszPhonebook: LPCSTR,
    lpszEntry: LPCSTR,
) -> DWORD {
    // FIXME!("(%s,%s), stub!\n",debugstr_a(lpszPhonebook),debugstr_a(lpszEntry));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasValidateEntryNameW(
    lpszPhonebook: LPCWSTR,
    lpszEntry: LPCWSTR,
) -> DWORD {
    // FIXME!("(%s,%s), stub!\n",debugstr_w(lpszPhonebook),debugstr_w(lpszEntry));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetEntryPropertiesA(
    lpszPhonebook: LPCSTR,
    lpszEntry: LPCSTR,
    lpRasEntry: LPRASENTRYA,
    lpdwEntryInfoSize: LPDWORD,
    lpbDeviceInfo: LPBYTE,
    lpdwDeviceInfoSize: LPDWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,%p,%p,%p), stub!\n", debugstr_a(lpszPhonebook), debugstr_a(lpszEntry), lpRasEntry, lpdwEntryInfoSize, lpbDeviceInfo, lpdwDeviceInfoSize);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetEntryPropertiesW(
    lpszPhonebook: LPCWSTR,
    lpszEntry: LPCWSTR,
    lpRasEntry: LPRASENTRYW,
    lpdwEntryInfoSize: LPDWORD,
    lpbDeviceInfo: LPBYTE,
    lpdwDeviceInfoSize: LPDWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,%p,%p,%p), stub!\n", debugstr_w(lpszPhonebook), debugstr_w(lpszEntry), lpRasEntry, lpdwEntryInfoSize, lpbDeviceInfo, lpdwDeviceInfoSize);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetErrorStringA(
    uErrorValue: UINT,
    lpszErrorString: LPSTR,
    cBufSize: DWORD,
) -> DWORD {
    // FIXME!("(0x%08x,%p,0x%08x), stub!\n", uErrorValue, lpszErrorString, cBufSize);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasGetErrorStringW(
    uErrorValue: UINT,
    lpszErrorString: LPWSTR,
    cBufSize: DWORD,
) -> DWORD {
    // FIXME!("(0x%08x,%p,0x%08x), stub!\n", uErrorValue, lpszErrorString, cBufSize);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasGetProjectionInfoA(
    hrasconn: HRASCONN,
    rasprojection: RASPROJECTION,
    lpprojection: LPVOID,
    lpcb: LPDWORD,
) -> DWORD {
    // FIXME!("(%p,0x%08x,%p,%p), stub!\n", hrasconn, rasprojection, lpprojection, lpcb);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasGetProjectionInfoW(
    hrasconn: HRASCONN,
    rasprojection: RASPROJECTION,
    lpprojection: LPVOID,
    lpcb: LPDWORD,
) -> DWORD {
    // FIXME!("(%p,0x%08x,%p,%p), stub!\n", hrasconn, rasprojection, lpprojection, lpcb);
    1
}

#[no_mangle]
pub unsafe extern "system" fn RasRenameEntryA(
    lpszPhonebook: LPCSTR,
    lpszOldEntry: LPCSTR,
    lpszNewEntry: LPCSTR,
) -> DWORD {
    // FIXME!("(%s,%s,%s), stub!\n", debugstr_a(lpszPhonebook), debugstr_a(lpszOldEntry), debugstr_a(lpszNewEntry));
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasRenameEntryW(
    lpszPhonebook: LPCWSTR,
    lpszOldEntry: LPCWSTR,
    lpszNewEntry: LPCWSTR,
) -> DWORD {
    // FIXME!("(%s,%s,%s), stub!\n", debugstr_w(lpszPhonebook), debugstr_w(lpszOldEntry), debugstr_w(lpszNewEntry));
    0
}

// Upstream's Wine-10 sync turned these two spec entries from `stub` into stdcall exports
// with FIXME bodies; without a Rust definition rasapi32.dll stops linking.
#[no_mangle]
pub unsafe extern "system" fn RasSetCredentialsA(
    phonebook: *const i8,
    entry: *const i8,
    credentials: *mut core::ffi::c_void, // RASCREDENTIALSA*
    clear: BOOL,
) -> DWORD {
    // FIXME!("(%s,%s,%p,0x%x), stub!\n", debugstr_a(phonebook), debugstr_a(entry), credentials, clear);
    let _ = (phonebook, entry, credentials, clear);
    ERROR_UNKNOWN
}

#[no_mangle]
pub unsafe extern "system" fn RasSetCredentialsW(
    phonebook: *const WCHAR,
    entry: *const WCHAR,
    credentials: *mut core::ffi::c_void, // RASCREDENTIALSW*
    clear: BOOL,
) -> DWORD {
    // FIXME!("(%s,%s,%p,0x%x), stub!\n", debugstr_w(phonebook), debugstr_w(entry), credentials, clear);
    let _ = (phonebook, entry, credentials, clear);
    ERROR_UNKNOWN
}

#[no_mangle]
pub unsafe extern "system" fn RasSetCustomAuthDataA(
    phonebook: *const i8,
    entry: *const i8,
    authdata: *mut BYTE,
    size: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,0x%08x), stub!\n", debugstr_a(phonebook), debugstr_a(entry), authdata, size);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetCustomAuthDataW(
    phonebook: *const WCHAR,
    entry: *const WCHAR,
    authdata: *mut BYTE,
    size: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,%p,0x%08x), stub!\n", debugstr_w(phonebook), debugstr_w(entry), authdata, size);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetEntryDialParamsA(
    lpszPhonebook: LPCSTR,
    lprasdialparams: LPRASDIALPARAMSA,
    fRemovePassword: BOOL,
) -> DWORD {
    // FIXME!("(%s,%p,0x%x), stub!\n", debugstr_a(lpszPhonebook), lprasdialparams, fRemovePassword);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetEntryDialParamsW(
    lpszPhonebook: LPCWSTR,
    lprasdialparams: LPRASDIALPARAMSW,
    fRemovePassword: BOOL,
) -> DWORD {
    // FIXME!("(%s,%p,0x%x), stub!\n", debugstr_w(lpszPhonebook), lprasdialparams, fRemovePassword);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetSubEntryPropertiesA(
    lpszPhonebook: LPCSTR,
    lpszEntry: LPCSTR,
    dwSubEntry: DWORD,
    lpRasSubEntry: LPRASSUBENTRYA,
    dwcbRasSubEntry: DWORD,
    lpbDeviceConfig: LPBYTE,
    dwcbDeviceConfig: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,0x%08x,%p,0x%08x,%p,0x%08x), stub!\n", debugstr_a(lpszPhonebook), debugstr_a(lpszEntry), dwSubEntry, lpRasSubEntry, dwcbRasSubEntry, lpbDeviceConfig, dwcbDeviceConfig);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasSetSubEntryPropertiesW(
    lpszPhonebook: LPCWSTR,
    lpszEntry: LPCWSTR,
    dwSubEntry: DWORD,
    lpRasSubEntry: LPRASSUBENTRYW,
    dwcbRasSubEntry: DWORD,
    lpbDeviceConfig: LPBYTE,
    dwcbDeviceConfig: DWORD,
) -> DWORD {
    // FIXME!("(%s,%s,0x%08x,%p,0x%08x,%p,0x%08x), stub!\n", debugstr_w(lpszPhonebook), debugstr_w(lpszEntry), dwSubEntry, lpRasSubEntry, dwcbRasSubEntry, lpbDeviceConfig, dwcbDeviceConfig);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetLinkStatistics(
    connection: HRASCONN,
    entry: DWORD,
    statistics: *mut RAS_STATS,
) -> DWORD {
    // FIXME!("(%p,%u,%p), stub!\n", connection, entry, statistics);
    0
}

#[no_mangle]
pub unsafe extern "system" fn RasGetConnectionStatistics(
    connection: HRASCONN,
    statistics: *mut RAS_STATS,
) -> DWORD {
    // FIXME!("(%p,%p), stub!\n", connection, statistics);
    ERROR_UNKNOWN
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * RASAPI32
//  *
//  * Copyright 1998,2001 Marcus Meissner
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
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "ras.h"
// #include "raserror.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(ras);
//
// DWORD WINAPI RasConnectionNotificationA(HRASCONN hrasconn, HANDLE hEvent, DWORD dwFlags)
// {
//     FIXME("(%p,%p,0x%08x),stub!\n",hrasconn,hEvent,dwFlags);
//     return 1;
// }
//
// DWORD WINAPI RasConnectionNotificationW(HRASCONN hrasconn, HANDLE hEvent, DWORD dwFlags)
// {
//     FIXME("(%p,%p,0x%08x),stub!\n",hrasconn,hEvent,dwFlags);
//     return 1;
// }
//
// DWORD WINAPI RasCreatePhonebookEntryA(HWND hwnd, LPCSTR lpszPhonebook)
// {
//     FIXME("(%p,%s),stub!\n",hwnd,debugstr_a(lpszPhonebook));
//     return 0;
// }
//
// DWORD WINAPI RasCreatePhonebookEntryW(HWND hwnd, LPCWSTR lpszPhonebook)
// {
//     FIXME("(%p,%s),stub!\n",hwnd,debugstr_w(lpszPhonebook));
//     return 0;
// }
//
// DWORD WINAPI RasDeleteSubEntryA(LPCSTR lpszPhonebook, LPCSTR lpszEntry, DWORD dwSubEntryId)
// {
//     FIXME("(%s,%s,0x%08x),stub!\n",debugstr_a(lpszPhonebook),
//           debugstr_a(lpszEntry),dwSubEntryId);
//     return 0;
// }
//
// DWORD WINAPI RasDeleteSubEntryW(LPCWSTR lpszPhonebook, LPCWSTR lpszEntry, DWORD dwSubEntryId)
// {
//     FIXME("(%s,%s,0x%08x),stub!\n",debugstr_w(lpszPhonebook),
//           debugstr_w(lpszEntry),dwSubEntryId);
//     return 0;
// }
//
// DWORD WINAPI RasDialA(LPRASDIALEXTENSIONS lpRasDialExtensions, LPCSTR lpszPhonebook,
//                       LPRASDIALPARAMSA lpRasDialParams, DWORD dwNotifierType,
//                       LPVOID lpvNotifier, LPHRASCONN lphRasConn)
// {
//     FIXME("(%p,%s,%p,0x%08x,%p,%p),stub!\n",lpRasDialExtensions,debugstr_a(lpszPhonebook),
//           lpRasDialParams,dwNotifierType,lpvNotifier,lphRasConn);
//     return 1;
// }
//
// DWORD WINAPI RasDialW(LPRASDIALEXTENSIONS lpRasDialExtensions, LPCWSTR lpszPhonebook,
//                       LPRASDIALPARAMSW lpRasDialParams, DWORD dwNotifierType,
//                       LPVOID lpvNotifier, LPHRASCONN lphRasConn)
// {
//     FIXME("(%p,%s,%p,0x%08x,%p,%p),stub!\n",lpRasDialExtensions,debugstr_w(lpszPhonebook),
//           lpRasDialParams,dwNotifierType,lpvNotifier,lphRasConn);
//     return 1;
// }
//
// DWORD WINAPI RasEditPhonebookEntryA(HWND hwnd, LPCSTR lpszPhonebook, LPCSTR lpszEntryName)
// {
//     FIXME("(%p,%s,%s),stub!\n",hwnd,debugstr_a(lpszPhonebook),debugstr_a(lpszEntryName));
//     return 0;
// }
//
// DWORD WINAPI RasEditPhonebookEntryW(HWND hwnd, LPCWSTR lpszPhonebook, LPCWSTR lpszEntryName)
// {
//     FIXME("(%p,%s,%s),stub!\n",hwnd,debugstr_w(lpszPhonebook),debugstr_w(lpszEntryName));
//     return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumConnectionsA			[RASAPI32.544]
//  */
// DWORD WINAPI RasEnumConnectionsA( LPRASCONNA rca, LPDWORD lpcb, LPDWORD lpcConnections) {
// 	/* Remote Access Service stuff is done by underlying OS anyway */
// 	FIXME("(%p,%p,%p),stub!\n",rca,lpcb,lpcConnections);
// 	FIXME("RAS support is not implemented! Configure program to use LAN connection/winsock instead!\n");
// 	*lpcb = 0; /* size of buffer needed to enumerate connections */
// 	*lpcConnections = 0; /* no RAS connections available */
//
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumConnectionsW			[RASAPI32.545]
//  */
// DWORD WINAPI RasEnumConnectionsW( LPRASCONNW rcw, LPDWORD lpcb, LPDWORD lpcConnections) {
// 	/* Remote Access Service stuff is done by underlying OS anyway */
// 	FIXME("(%p,%p,%p),stub!\n",rcw,lpcb,lpcConnections);
// 	FIXME("RAS support is not implemented! Configure program to use LAN connection/winsock instead!\n");
// 	*lpcb = 0; /* size of buffer needed to enumerate connections */
// 	*lpcConnections = 0; /* no RAS connections available */
//
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumEntriesA		        	[RASAPI32.546]
//  */
// DWORD WINAPI RasEnumEntriesA( LPCSTR Reserved, LPCSTR lpszPhoneBook,
//         LPRASENTRYNAMEA lpRasEntryName,
//         LPDWORD lpcb, LPDWORD lpcEntries)
// {
// 	FIXME("(%p,%s,%p,%p,%p),stub!\n",Reserved,debugstr_a(lpszPhoneBook),
//             lpRasEntryName,lpcb,lpcEntries);
//         *lpcEntries = 0;
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumEntriesW		        	[RASAPI32.547]
//  */
// DWORD WINAPI RasEnumEntriesW( LPCWSTR Reserved, LPCWSTR lpszPhoneBook,
//         LPRASENTRYNAMEW lpRasEntryName,
//         LPDWORD lpcb, LPDWORD lpcEntries)
// {
// 	FIXME("(%p,%s,%p,%p,%p),stub!\n",Reserved,debugstr_w(lpszPhoneBook),
//             lpRasEntryName,lpcb,lpcEntries);
//         *lpcEntries = 0;
// 	return 0;
// }
//
// DWORD WINAPI RasGetConnectStatusA(HRASCONN hrasconn, LPRASCONNSTATUSA lprasconnstatus)
// {
//     FIXME("(%p,%p),stub!\n",hrasconn,lprasconnstatus);
//     return 0;
// }
//
// DWORD WINAPI RasGetConnectStatusW(HRASCONN hrasconn, LPRASCONNSTATUSW lprasconnstatus)
// {
//     FIXME("(%p,%p),stub!\n",hrasconn,lprasconnstatus);
//     return 0;
// }
//
// /**************************************************************************
//  *                 RasGetEntryDialParamsA			[RASAPI32.550]
//  */
// DWORD WINAPI RasGetEntryDialParamsA(
// 	LPCSTR lpszPhoneBook, LPRASDIALPARAMSA lpRasDialParams,
// 	LPBOOL lpfPassword)
// {
// 	FIXME("(%s,%p,%p),stub!\n",debugstr_a(lpszPhoneBook),
//             lpRasDialParams,lpfPassword);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetEntryDialParamsW           [RASAPI32.551]
//  */
// DWORD WINAPI RasGetEntryDialParamsW(
//     LPCWSTR lpszPhoneBook, LPRASDIALPARAMSW lpRasDialParams,
//     LPBOOL lpfPassword)
// {
//     FIXME("(%s,%p,%p),stub!\n",debugstr_w(lpszPhoneBook),
//             lpRasDialParams,lpfPassword);
//     return 0;
// }
//
// /**************************************************************************
//  *                 RasHangUpA			[RASAPI32.556]
//  */
// DWORD WINAPI RasHangUpA( HRASCONN hrasconn)
// {
// 	FIXME("(%p),stub!\n",hrasconn);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasHangUpW           [RASAPI32.557]
//  */
// DWORD WINAPI RasHangUpW(HRASCONN hrasconn)
// {
//     FIXME("(%p),stub!\n",hrasconn);
//     return 0;
// }
//
// /**************************************************************************
//  *                 RasDeleteEntryA		[RASAPI32.7]
//  */
// DWORD WINAPI RasDeleteEntryA(LPCSTR a, LPCSTR b)
// {
// 	FIXME("(%s,%s),stub!\n",debugstr_a(a),debugstr_a(b));
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasDeleteEntryW		[RASAPI32.8]
//  */
// DWORD WINAPI RasDeleteEntryW(LPCWSTR a, LPCWSTR b)
// {
// 	FIXME("(%s,%s),stub!\n",debugstr_w(a),debugstr_w(b));
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumAutodialAddressesA	[RASAPI32.14]
//  */
// DWORD WINAPI RasEnumAutodialAddressesA(LPSTR *a, LPDWORD b, LPDWORD c)
// {
// 	FIXME("(%p,%p,%p),stub!\n",a,b,c);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumAutodialAddressesW	[RASAPI32.15]
//  */
// DWORD WINAPI RasEnumAutodialAddressesW(LPWSTR *a, LPDWORD b, LPDWORD c)
// {
// 	FIXME("(%p,%p,%p),stub!\n",a,b,c);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumDevicesA		[RASAPI32.19]
//  *
//  * Just return a virtual modem to see what other APIs programs will
//  * call with it.
//  */
// DWORD WINAPI RasEnumDevicesA(LPRASDEVINFOA lpRasDevinfo, LPDWORD lpcb, LPDWORD lpcDevices)
// {
// 	if (!lpcb || !lpcDevices)
//             return ERROR_INVALID_PARAMETER;
//
// 	FIXME("(%p,%p,%p),stub!\n",lpRasDevinfo,lpcb,lpcDevices);
//
// 	if(lpRasDevinfo && lpRasDevinfo->dwSize != sizeof(RASDEVINFOA))
// 		return ERROR_INVALID_SIZE;
//
// 	*lpcDevices = 1;
//
// 	if (!lpRasDevinfo || (*lpcb < sizeof(RASDEVINFOA))) {
// 		*lpcb = sizeof(RASDEVINFOA);
// 		return ERROR_BUFFER_TOO_SMALL;
// 	}
// 	/* honor dwSize ? */
// 	strcpy(lpRasDevinfo->szDeviceType, RASDT_Modem);
// 	strcpy(lpRasDevinfo->szDeviceName, "WINE virtmodem");
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasEnumDevicesW		[RASAPI32.20]
//  */
// DWORD WINAPI RasEnumDevicesW(LPRASDEVINFOW a, LPDWORD b, LPDWORD c)
// {
// 	FIXME("(%p,%p,%p),stub!\n",a,b,c);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialAddressA	[RASAPI32.24]
//  */
// DWORD WINAPI RasGetAutodialAddressA(LPCSTR a, LPDWORD b, LPRASAUTODIALENTRYA c,
// 					LPDWORD d, LPDWORD e)
// {
// 	FIXME("(%s,%p,%p,%p,%p),stub!\n",debugstr_a(a),b,c,d,e);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialAddressW	[RASAPI32.25]
//  */
// DWORD WINAPI RasGetAutodialAddressW(LPCWSTR a, LPDWORD b, LPRASAUTODIALENTRYW c,
// 					LPDWORD d, LPDWORD e)
// {
// 	FIXME("(%s,%p,%p,%p,%p),stub!\n",debugstr_w(a),b,c,d,e);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialEnableA	[RASAPI32.26]
//  */
// DWORD WINAPI RasGetAutodialEnableA(DWORD a, LPBOOL b)
// {
// 	FIXME("(%x,%p),stub!\n",a,b);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialEnableW	[RASAPI32.27]
//  */
// DWORD WINAPI RasGetAutodialEnableW(DWORD a, LPBOOL b)
// {
// 	FIXME("(%x,%p),stub!\n",a,b);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialParamA		[RASAPI32.28]
//  */
// DWORD WINAPI RasGetAutodialParamA(DWORD dwKey, LPVOID lpvValue, LPDWORD lpdwcbValue)
// {
// 	FIXME("(%x,%p,%p),stub!\n",dwKey,lpvValue,lpdwcbValue);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetAutodialParamW		[RASAPI32.29]
//  */
// DWORD WINAPI RasGetAutodialParamW(DWORD dwKey, LPVOID lpvValue, LPDWORD lpdwcbValue)
// {
// 	FIXME("(%x,%p,%p),stub!\n",dwKey,lpvValue,lpdwcbValue);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialAddressA	[RASAPI32.57]
//  */
// DWORD WINAPI RasSetAutodialAddressA(LPCSTR a, DWORD b, LPRASAUTODIALENTRYA c,
// 					DWORD d, DWORD e)
// {
// 	FIXME("(%s,%x,%p,%x,%x),stub!\n",debugstr_a(a),b,c,d,e);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialAddressW	[RASAPI32.58]
//  */
// DWORD WINAPI RasSetAutodialAddressW(LPCWSTR a, DWORD b, LPRASAUTODIALENTRYW c,
// 					DWORD d, DWORD e)
// {
// 	FIXME("(%s,%x,%p,%x,%x),stub!\n",debugstr_w(a),b,c,d,e);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialEnableA	[RASAPI32.59]
//  */
// DWORD WINAPI RasSetAutodialEnableA(DWORD dwDialingLocation, BOOL fEnabled)
// {
// 	FIXME("(%x,%x),stub!\n",dwDialingLocation,fEnabled);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialEnableW	[RASAPI32.60]
//  */
// DWORD WINAPI RasSetAutodialEnableW(DWORD dwDialingLocation, BOOL fEnabled)
// {
// 	FIXME("(%x,%x),stub!\n",dwDialingLocation,fEnabled);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialParamA	[RASAPI32.61]
//  */
// DWORD WINAPI RasSetAutodialParamA(DWORD a, LPVOID b, DWORD c)
// {
// 	FIXME("(%x,%p,%x),stub!\n",a,b,c);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetAutodialParamW	[RASAPI32.62]
//  */
// DWORD WINAPI RasSetAutodialParamW(DWORD a, LPVOID b, DWORD c)
// {
// 	FIXME("(%x,%p,%x),stub!\n",a,b,c);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetEntryPropertiesA	[RASAPI32.67]
//  */
// DWORD WINAPI RasSetEntryPropertiesA(LPCSTR lpszPhonebook, LPCSTR lpszEntry,
// 	LPRASENTRYA lpRasEntry, DWORD dwEntryInfoSize, LPBYTE lpbDeviceInfo,
// 	DWORD dwDeviceInfoSize
// ) {
// 	FIXME("(%s,%s,%p,%d,%p,%d), stub!\n",
// 		debugstr_a(lpszPhonebook),debugstr_a(lpszEntry),
// 		lpRasEntry,dwEntryInfoSize,lpbDeviceInfo,dwDeviceInfoSize
// 	);
// 	FIXME("Rasentry:\n");
// 	FIXME("\tdwfOptions %x\n",lpRasEntry->dwfOptions);
// 	FIXME("\tszLocalPhoneNumber %s\n",debugstr_a(lpRasEntry->szLocalPhoneNumber));
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasSetEntryPropertiesW	[RASAPI32.68]
//  */
// DWORD WINAPI RasSetEntryPropertiesW(LPCWSTR lpszPhonebook, LPCWSTR lpszEntry,
// 	LPRASENTRYW lpRasEntry, DWORD dwEntryInfoSize, LPBYTE lpbDeviceInfo,
// 	DWORD dwDeviceInfoSize
// ) {
// 	FIXME("(%s,%s,%p,%d,%p,%d), stub!\n",
// 		debugstr_w(lpszPhonebook),debugstr_w(lpszEntry),
// 		lpRasEntry,dwEntryInfoSize,lpbDeviceInfo,dwDeviceInfoSize
// 	);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasValidateEntryNameA	[RASAPI32.72]
//  */
// DWORD WINAPI RasValidateEntryNameA(LPCSTR lpszPhonebook, LPCSTR lpszEntry) {
// 	FIXME("(%s,%s), stub!\n",debugstr_a(lpszPhonebook),debugstr_a(lpszEntry));
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasValidateEntryNameW	[RASAPI32.73]
//  */
// DWORD WINAPI RasValidateEntryNameW(LPCWSTR lpszPhonebook, LPCWSTR lpszEntry) {
// 	FIXME("(%s,%s), stub!\n",debugstr_w(lpszPhonebook),debugstr_w(lpszEntry));
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetEntryPropertiesA	[RASAPI32.@]
//  */
// DWORD WINAPI RasGetEntryPropertiesA(LPCSTR lpszPhonebook, LPCSTR lpszEntry, LPRASENTRYA lpRasEntry,
// 	LPDWORD lpdwEntryInfoSize, LPBYTE lpbDeviceInfo, LPDWORD lpdwDeviceInfoSize
// ) {
// 	FIXME("(%s,%s,%p,%p,%p,%p), stub!\n", debugstr_a(lpszPhonebook), debugstr_a(lpszEntry), lpRasEntry,
//           lpdwEntryInfoSize, lpbDeviceInfo, lpdwDeviceInfoSize);
// 	return 0;
// }
//
// /**************************************************************************
//  *                 RasGetEntryPropertiesW   [RASAPI32.@]
//  */
// DWORD WINAPI RasGetEntryPropertiesW(LPCWSTR lpszPhonebook, LPCWSTR lpszEntry, LPRASENTRYW lpRasEntry,
//     LPDWORD lpdwEntryInfoSize, LPBYTE lpbDeviceInfo, LPDWORD lpdwDeviceInfoSize
// ) {
//     FIXME("(%s,%s,%p,%p,%p,%p), stub!\n", debugstr_w(lpszPhonebook), debugstr_w(lpszEntry), lpRasEntry,
//           lpdwEntryInfoSize, lpbDeviceInfo, lpdwDeviceInfoSize);
//     return 0;
// }
//
// DWORD WINAPI RasGetErrorStringA(UINT uErrorValue, LPSTR lpszErrorString, DWORD cBufSize)
// {
//     FIXME("(0x%08x,%p,0x%08x), stub!\n", uErrorValue, lpszErrorString, cBufSize);
//     return 1;
// }
//
// DWORD WINAPI RasGetErrorStringW(UINT uErrorValue, LPWSTR lpszErrorString, DWORD cBufSize)
// {
//     FIXME("(0x%08x,%p,0x%08x), stub!\n", uErrorValue, lpszErrorString, cBufSize);
//     return 1;
// }
//
// DWORD WINAPI RasGetProjectionInfoA(HRASCONN hrasconn, RASPROJECTION rasprojection,
//                                    LPVOID lpprojection, LPDWORD lpcb)
// {
//     FIXME("(%p,0x%08x,%p,%p), stub!\n", hrasconn, rasprojection, lpprojection, lpcb);
//     return 1;
// }
//
// DWORD WINAPI RasGetProjectionInfoW(HRASCONN hrasconn, RASPROJECTION rasprojection,
//                                    LPVOID lpprojection, LPDWORD lpcb)
// {
//     FIXME("(%p,0x%08x,%p,%p), stub!\n", hrasconn, rasprojection, lpprojection, lpcb);
//     return 1;
// }
//
// DWORD WINAPI RasRenameEntryA(LPCSTR lpszPhonebook, LPCSTR lpszOldEntry, LPCSTR lpszNewEntry)
// {
//     FIXME("(%s,%s,%s), stub!\n", debugstr_a(lpszPhonebook), debugstr_a(lpszOldEntry),
//           debugstr_a(lpszNewEntry));
//     return 0;
// }
//
// DWORD WINAPI RasRenameEntryW(LPCWSTR lpszPhonebook, LPCWSTR lpszOldEntry, LPCWSTR lpszNewEntry)
// {
//     FIXME("(%s,%s,%s), stub!\n", debugstr_w(lpszPhonebook), debugstr_w(lpszOldEntry),
//           debugstr_w(lpszNewEntry));
//     return 0;
// }
//
// DWORD WINAPI RasSetCustomAuthDataA(const char *phonebook, const char *entry, BYTE *authdata, DWORD size)
// {
//     FIXME("(%s,%s,%p,0x%08x), stub!\n", debugstr_a(phonebook), debugstr_a(entry), authdata, size);
//     return 0;
// }
//
// DWORD WINAPI RasSetCustomAuthDataW(const WCHAR *phonebook, const WCHAR *entry, BYTE *authdata, DWORD size)
// {
//     FIXME("(%s,%s,%p,0x%08x), stub!\n", debugstr_w(phonebook), debugstr_w(entry), authdata, size);
//     return 0;
// }
//
// DWORD WINAPI RasSetEntryDialParamsA(LPCSTR lpszPhonebook, LPRASDIALPARAMSA lprasdialparams,
//                                     BOOL fRemovePassword)
// {
//     FIXME("(%s,%p,0x%x), stub!\n", debugstr_a(lpszPhonebook), lprasdialparams, fRemovePassword);
//     return 0;
// }
//
// DWORD WINAPI RasSetEntryDialParamsW(LPCWSTR lpszPhonebook, LPRASDIALPARAMSW lprasdialparams,
//                                     BOOL fRemovePassword)
// {
//     FIXME("(%s,%p,0x%x), stub!\n", debugstr_w(lpszPhonebook), lprasdialparams, fRemovePassword);
//     return 0;
// }
//
// DWORD WINAPI RasSetSubEntryPropertiesA(LPCSTR lpszPhonebook, LPCSTR lpszEntry, DWORD dwSubEntry,
//                                        LPRASSUBENTRYA lpRasSubEntry, DWORD dwcbRasSubEntry,
//                                        LPBYTE lpbDeviceConfig, DWORD dwcbDeviceConfig)
// {
//     FIXME("(%s,%s,0x%08x,%p,0x%08x,%p,0x%08x), stub!\n", debugstr_a(lpszPhonebook),
//           debugstr_a(lpszEntry), dwSubEntry, lpRasSubEntry, dwcbRasSubEntry, lpbDeviceConfig,
//           dwcbDeviceConfig);
//     return 0;
// }
//
// DWORD WINAPI RasSetSubEntryPropertiesW(LPCWSTR lpszPhonebook, LPCWSTR lpszEntry, DWORD dwSubEntry,
//                                        LPRASSUBENTRYW lpRasSubEntry, DWORD dwcbRasSubEntry,
//                                        LPBYTE lpbDeviceConfig, DWORD dwcbDeviceConfig)
// {
//     FIXME("(%s,%s,0x%08x,%p,0x%08x,%p,0x%08x), stub!\n", debugstr_w(lpszPhonebook),
//           debugstr_w(lpszEntry), dwSubEntry, lpRasSubEntry, dwcbRasSubEntry, lpbDeviceConfig,
//           dwcbDeviceConfig);
//     return 0;
// }
//
// DWORD WINAPI RasGetLinkStatistics(HRASCONN connection, DWORD entry, RAS_STATS *statistics)
// {
//     FIXME("(%p,%u,%p), stub!\n", connection, entry, statistics);
//     return 0;
// }
//
// DWORD WINAPI RasGetConnectionStatistics(HRASCONN connection, RAS_STATS *statistics)
// {
//     FIXME("(%p,%p), stub!\n", connection, statistics);
//     return ERROR_UNKNOWN;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
