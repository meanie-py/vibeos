//! Shell Registry Access — translation of dll/win32/shell32/wine/shellreg.c.
//!
//! Thin SHReg* wrappers over advapi32 Reg* + shlwapi SHQueryValueEx*, plus
//! SHCreateSessionKey (builds a per-logon volatile HKLM key). All exports are WINAPI
//! (stdcall on i386 → `_SHRegOpenKeyA@12` etc., matched by `#[no_mangle] extern "system"`).

use crate::types::*;
// One declaration per external symbol: island.rs.
use crate::island::CloseHandle;
use crate::island::GetCurrentProcess;
use crate::island::GetTokenInformation;
use crate::island::OpenProcessToken;
use crate::island::RegCloseKey;
use crate::island::RegCreateKeyExW;
use crate::island::RegOpenKeyA;
use crate::island::RegOpenKeyW;
use crate::island::RegQueryValueA;
use crate::island::RegQueryValueW;
use crate::island::SHQueryValueExA;
use crate::island::SHQueryValueExW;
use core::mem::size_of;
use core::ptr;

// ── imports (resolved via shell32's existing import libs at link) ──────────────────────
extern "system" {
    // advapi32
    // kernel32
    fn GetLastError() -> DWORD;
    // shlwapi
}

const TOKEN_READ: DWORD = 0x00020008; // STANDARD_RIGHTS_READ(0x20000) | TOKEN_QUERY(0x8)
const TokenStatistics: DWORD = 10; // TOKEN_INFORMATION_CLASS
const REG_OPTION_VOLATILE: DWORD = 1;

/// *************************************************************************
/// SHRegOpenKeyA                [SHELL32.506]
#[no_mangle]
pub unsafe extern "system" fn SHRegOpenKeyA(hKey: HKEY, lpSubKey: LPSTR, phkResult: PHKEY) -> HRESULT {
    RegOpenKeyA(hKey, lpSubKey as LPCSTR, phkResult) as HRESULT
}

/// *************************************************************************
/// SHRegOpenKeyW                [SHELL32.507] NT 4.0
#[no_mangle]
pub unsafe extern "system" fn SHRegOpenKeyW(hkey: HKEY, lpszSubKey: LPCWSTR, retkey: PHKEY) -> HRESULT {
    RegOpenKeyW(hkey, lpszSubKey, retkey) as HRESULT
}

/// *************************************************************************
/// SHRegQueryValueA   [SHELL32.508]
#[no_mangle]
pub unsafe extern "system" fn SHRegQueryValueA(
    hkey: HKEY,
    lpSubKey: LPSTR,
    lpValue: LPSTR,
    lpcbValue: LPDWORD,
) -> HRESULT {
    RegQueryValueA(hkey, lpSubKey as LPCSTR, lpValue, lpcbValue as *mut LONG) as HRESULT
}

/// *************************************************************************
/// SHRegQueryValueExA   [SHELL32.509]
#[no_mangle]
pub unsafe extern "system" fn SHRegQueryValueExA(
    hkey: HKEY,
    lpValueName: LPCSTR,
    lpReserved: LPDWORD,
    lpType: LPDWORD,
    lpData: LPBYTE,
    lpcbData: LPDWORD,
) -> LONG {
    (SHQueryValueExA(hkey, lpValueName, lpReserved, lpType, (lpData).cast(), lpcbData)) as i32
}

/// *************************************************************************
/// SHRegQueryValueW             [SHELL32.510] NT4.0
#[no_mangle]
pub unsafe extern "system" fn SHRegQueryValueW(
    hkey: HKEY,
    lpszSubKey: LPWSTR,
    lpszData: LPWSTR,
    lpcbData: LPDWORD,
) -> HRESULT {
    RegQueryValueW(hkey, lpszSubKey as LPCWSTR, lpszData, lpcbData as *mut LONG) as HRESULT
}

/// *************************************************************************
/// SHRegQueryValueExW   [SHELL32.511] NT4.0
#[no_mangle]
pub unsafe extern "system" fn SHRegQueryValueExW(
    hkey: HKEY,
    pszValue: LPCWSTR,
    pdwReserved: LPDWORD,
    pdwType: LPDWORD,
    pvData: LPVOID,
    pcbData: LPDWORD,
) -> LONG {
    (SHQueryValueExW(hkey, pszValue, pdwReserved, pdwType, pvData, pcbData)) as i32
}

/// *************************************************************************
/// SHRegDeleteKeyW   [SHELL32.512]
#[no_mangle]
pub unsafe extern "system" fn SHRegDeleteKeyW(hkey: HKEY, pszSubKey: LPCWSTR) -> HRESULT {
    // C body is a FIXME stub returning 0.
    0
}

/// *************************************************************************
/// SHRegCloseKey            [SHELL32.505] NT 4.0
#[no_mangle]
pub unsafe extern "system" fn SHRegCloseKey(hkey: HKEY) -> HRESULT {
    RegCloseKey(hkey) as HRESULT
}

// Per-process cached session-key path (C: function-static WCHAR[256], no critical section).
static mut WSZ_SESSION_KEY: [WCHAR; 256] = [0; 256];

#[inline]
fn hex_digit(n: u32) -> WCHAR {
    (if n < 10 { b'0' + n as u8 } else { b'a' + (n - 10) as u8 }) as WCHAR
}

/// *************************************************************************
/// SHCreateSessionKey            [SHELL32.723]
#[no_mangle]
pub unsafe extern "system" fn SHCreateSessionKey(samDesired: REGSAM, phKey: PHKEY) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if WSZ_SESSION_KEY[0] == 0 {
        // FIXME: Critical Section
        let mut hToken: HANDLE = ptr::null_mut();

        if OpenProcessToken(GetCurrentProcess(), TOKEN_READ, &mut hToken) != FALSE {
            let mut stats: TOKEN_STATISTICS = core::mem::zeroed();
            let mut return_length: DWORD = 0;

            if GetTokenInformation(
                hToken,
                TokenStatistics,
                &mut stats as *mut _ as LPVOID,
                size_of::<TOKEN_STATISTICS>() as DWORD,
                &mut return_length,
            ) != FALSE
            {
                // swprintf(wsz, L"SOFTWARE\\...\\SessionInfo\\%08x%08x",
                //          AuthenticationId.HighPart, AuthenticationId.LowPart)
                let prefix: &[u8] =
                    b"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\SessionInfo\\";
                let buf = &mut WSZ_SESSION_KEY;
                let mut idx = 0usize;
                for &b in prefix {
                    buf[idx] = b as WCHAR;
                    idx += 1;
                }
                let hi = stats.AuthenticationId.HighPart as u32;
                let lo = stats.AuthenticationId.LowPart;
                for shift in (0..8u32).rev() {
                    buf[idx] = hex_digit((hi >> (shift * 4)) & 0xF);
                    idx += 1;
                }
                for shift in (0..8u32).rev() {
                    buf[idx] = hex_digit((lo >> (shift * 4)) & 0xF);
                    idx += 1;
                }
                buf[idx] = 0;
            } else {
                hr = HRESULT_FROM_WIN32(GetLastError() as LONG);
            }

            CloseHandle(hToken);
        } else {
            hr = HRESULT_FROM_WIN32(GetLastError() as LONG);
        }
    }

    if hr >= 0 {
        // SUCCEEDED(hr)
        let error = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            WSZ_SESSION_KEY.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_VOLATILE,
            samDesired,
            ptr::null_mut(),
            phKey,
            ptr::null_mut(),
        );
        if error != ERROR_SUCCESS {
            hr = HRESULT_FROM_WIN32(error);
        }
    }

    hr
}

// HKEY_LOCAL_MACHINE = (HKEY)0x80000002
const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
