//! environment.c — user environment routines (CreateUserEnvironment +
//! BuildVolatileEnvironment). On the autologon boot path: SAS calls
//! CreateUserEnvironment after a successful logon, which impersonates the
//! new user, opens HKCU, and populates the "Volatile Environment" key
//! from `Session->Profile->pszEnvironment` plus the APPDATA shell folder.
//!
//! Exercises the full WLSESSION layout at runtime (reads `UserToken` and
//! the `Profile` (PWLX_PROFILE_V2_0) pointer) — the substrate layout is
//! size-asserted ×3 by winlogon-sys.

use winlogon_sys::nt::*;
use crate::preamble::*;

/// Build an aligned `[u16; N]` wide-string buffer from an ASCII byte
/// literal. Avoids `b"..".as_ptr() as *const u16` (align-1, faults on
/// amd64 WCHAR reads — feedback_byte_literal_wchar_misalign). ASCII-only,
/// which covers every literal in this file.
macro_rules! wsz {
    ($s:literal) => {{
        const B: &[u8] = $s;
        const N: usize = B.len() + 1;
        let mut a = [0u16; N];
        let mut i = 0;
        while i < B.len() {
            a[i] = B[i] as u16;
            i += 1;
        }
        a
    }};
}

unsafe fn BuildVolatileEnvironment(Session: PWLSESSION, hKeyCurrentUser: HKEY) {
    let mut szPath: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut wstr: LPCWSTR;
    let mut size: usize;
    let mut szEnvKey: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szEnvValue: [u16; 1024] = [0; 1024];
    let mut length: usize;
    let mut eqptr: LPWSTR;
    let mut endptr: LPWSTR;
    let mut dwDisp: DWORD = 0;
    let mut lError: LONG;
    let mut hKeyVolatileEnv: HKEY = core::ptr::null_mut();
    let mut hKeyShellFolders: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;

    /* Create the 'Volatile Environment' key */
    let volatile_environment = wsz!(b"Volatile Environment");
    lError = RegCreateKeyExW(
        hKeyCurrentUser,
        volatile_environment.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKeyVolatileEnv,
        &mut dwDisp,
    );
    if lError != ERROR_SUCCESS as LONG {
        // WARN: failed to create the volatile environment key
        return;
    }

    /* Parse the environment variables and add them to the volatile environment key */
    if (*(*Session).Profile).dwType == WLX_PROFILE_TYPE_V2_0 as DWORD
        && !(*(*Session).Profile).pszEnvironment.is_null()
    {
        wstr = (*(*Session).Profile).pszEnvironment;
        while *wstr != 0 {
            size = wcslen(wstr) + 1;

            eqptr = wcschr(wstr, b'=' as u16);

            if !eqptr.is_null() {
                endptr = eqptr;

                endptr = endptr.sub(1);
                while iswspace(*endptr as i32) != 0 {
                    endptr = endptr.sub(1);
                }

                length = ((endptr as *const u16).offset_from(wstr) + 1) as usize;

                wcsncpy(szEnvKey.as_mut_ptr(), wstr, length);
                szEnvKey[length] = 0;

                eqptr = eqptr.add(1);
                while iswspace(*eqptr as i32) != 0 {
                    eqptr = eqptr.add(1);
                }
                wcscpy(szEnvValue.as_mut_ptr(), eqptr);

                RegSetValueExW(
                    hKeyVolatileEnv,
                    szEnvKey.as_ptr(),
                    0,
                    REG_SZ,
                    szEnvValue.as_ptr() as *const BYTE,
                    ((wcslen(szEnvValue.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
                );
            }

            wstr = wstr.add(size);
        }
    }

    /* Set the 'APPDATA' environment variable */
    let shell_folders =
        wsz!(b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders");
    lError = RegOpenKeyExW(
        hKeyCurrentUser,
        shell_folders.as_ptr(),
        0,
        KEY_READ,
        &mut hKeyShellFolders,
    );
    if lError == ERROR_SUCCESS as LONG {
        let appdata_value = wsz!(b"AppData");
        dwSize = (MAX_PATH + 1) * core::mem::size_of::<WCHAR>() as DWORD;
        lError = RegQueryValueExW(
            hKeyShellFolders,
            appdata_value.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            szPath.as_mut_ptr() as LPBYTE,
            &mut dwSize,
        );
        if lError == ERROR_SUCCESS as LONG {
            let appdata_name = wsz!(b"APPDATA");
            RegSetValueExW(
                hKeyVolatileEnv,
                appdata_name.as_ptr(),
                0,
                REG_SZ,
                szPath.as_ptr() as *const BYTE,
                ((wcslen(szPath.as_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
            );
        }

        RegCloseKey(hKeyShellFolders);
    }

    RegCloseKey(hKeyVolatileEnv);
}

#[no_mangle]
pub unsafe extern "C" fn CreateUserEnvironment(Session: PWLSESSION) -> BOOL {
    let hKeyCurrentUser: HKEY;
    let lError: LONG;

    /* Impersonate the new user */
    if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        // ERR: ImpersonateLoggedOnUser() failed
        return FALSE as BOOL;
    }

    /* Open the new user HKCU key */
    let mut hKey: HKEY = core::ptr::null_mut();
    lError = RegOpenCurrentUser(KEY_CREATE_SUB_KEY, &mut hKey);
    hKeyCurrentUser = hKey;
    if lError == ERROR_SUCCESS as LONG {
        BuildVolatileEnvironment(Session, hKeyCurrentUser);
        RegCloseKey(hKeyCurrentUser);
    }

    /* Revert the impersonation */
    RevertToSelf();

    TRUE as BOOL
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Winlogon
//  * FILE:            base/system/winlogon/environment.c
//  * PURPOSE:         User environment routines
//  */
//
// #include "winlogon.h"
//
// static
// VOID
// BuildVolatileEnvironment(
//     IN PWLSESSION Session,
//     IN HKEY hKeyCurrentUser)
// {
//     WCHAR szPath[MAX_PATH + 1];
//     LPCWSTR wstr;
//     SIZE_T size;
//     WCHAR szEnvKey[MAX_PATH];
//     WCHAR szEnvValue[1024];
//     SIZE_T length;
//     LPWSTR eqptr, endptr;
//     DWORD dwDisp;
//     LONG lError;
//     HKEY hKeyVolatileEnv;
//     HKEY hKeyShellFolders;
//     DWORD dwType;
//     DWORD dwSize;
//
//     /* Create the 'Volatile Environment' key */
//     lError = RegCreateKeyExW(hKeyCurrentUser,
//                              L"Volatile Environment",
//                              0, NULL, REG_OPTION_VOLATILE, KEY_WRITE,
//                              NULL, &hKeyVolatileEnv, &dwDisp);
//     if (lError != ERROR_SUCCESS)
//     {
//         WARN("WL: RegCreateKeyExW() failed to create the volatile environment key (Error: %ld)\n", lError);
//         return;
//     }
//
//     /* Parse the environment variables and add them to the volatile environment key */
//     if (Session->Profile->dwType == WLX_PROFILE_TYPE_V2_0 &&
//         Session->Profile->pszEnvironment != NULL)
//     {
//         wstr = Session->Profile->pszEnvironment;
//         while (*wstr != UNICODE_NULL)
//         {
//             size = wcslen(wstr) + 1;
//             eqptr = wcschr(wstr, L'=');
//             if (eqptr != NULL)
//             {
//                 endptr = eqptr;
//                 endptr--;
//                 while (iswspace(*endptr))
//                     endptr--;
//                 length = (SIZE_T)(endptr - wstr + 1);
//                 wcsncpy(szEnvKey, wstr, length);
//                 szEnvKey[length] = 0;
//                 eqptr++;
//                 while (iswspace(*eqptr))
//                     eqptr++;
//                 wcscpy(szEnvValue, eqptr);
//                 RegSetValueExW(hKeyVolatileEnv, szEnvKey, 0, REG_SZ,
//                                (LPBYTE)szEnvValue,
//                                (wcslen(szEnvValue) + 1) * sizeof(WCHAR));
//             }
//             wstr += size;
//         }
//     }
//
//     /* Set the 'APPDATA' environment variable */
//     lError = RegOpenKeyExW(hKeyCurrentUser,
//                            L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
//                            0, KEY_READ, &hKeyShellFolders);
//     if (lError == ERROR_SUCCESS)
//     {
//         dwSize = (MAX_PATH + 1) * sizeof(WCHAR);
//         lError = RegQueryValueExW(hKeyShellFolders, L"AppData", NULL,
//                                   &dwType, (LPBYTE)szPath, &dwSize);
//         if (lError == ERROR_SUCCESS)
//         {
//             TRACE("APPDATA path: %S\n", szPath);
//             RegSetValueExW(hKeyVolatileEnv, L"APPDATA", 0, REG_SZ,
//                            (LPBYTE)szPath,
//                            (wcslen(szPath) + 1) * sizeof(WCHAR));
//         }
//         RegCloseKey(hKeyShellFolders);
//     }
//
//     RegCloseKey(hKeyVolatileEnv);
// }
//
// BOOL
// CreateUserEnvironment(
//     IN PWLSESSION Session)
// {
//     HKEY hKeyCurrentUser;
//     LONG lError;
//
//     TRACE("WL: CreateUserEnvironment called\n");
//
//     /* Impersonate the new user */
//     if (!ImpersonateLoggedOnUser(Session->UserToken))
//     {
//         ERR("ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Open the new user HKCU key */
//     lError = RegOpenCurrentUser(KEY_CREATE_SUB_KEY, &hKeyCurrentUser);
//     if (lError == ERROR_SUCCESS)
//     {
//         BuildVolatileEnvironment(Session, hKeyCurrentUser);
//         RegCloseKey(hKeyCurrentUser);
//     }
//
//     /* Revert the impersonation */
//     RevertToSelf();
//
//     TRACE("WL: CreateUserEnvironment done\n");
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
