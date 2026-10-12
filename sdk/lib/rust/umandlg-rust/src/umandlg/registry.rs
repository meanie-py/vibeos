use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut RegData: REGISTRY_DATA = _REGISTRY_DATA {
    lpszOskPath: ::core::ptr::null::<WCHAR>(),
    lpszOskDisplayName: ::core::ptr::null::<WCHAR>(),
    lpszMagnifyPath: ::core::ptr::null::<WCHAR>(),
    lpszMagnifyDisplayName: ::core::ptr::null::<WCHAR>(),
};

#[no_mangle]
pub static mut Settings: REGISTRY_SETTINGS = _REGISTRY_SETTINGS {
    lpszAppPath: ::core::ptr::null::<WCHAR>(),
    dwAppType: 0,
    dwClientControlCode: 0,
    lpszAppName: ::core::ptr::null::<WCHAR>(),
    lpszErrorOnLaunch: ::core::ptr::null::<WCHAR>(),
    bHideClient: 0,
    bStartWithUtilman: 0,
    bStartWithROS: 0,
    lpszHungRespondAction: ::core::ptr::null::<WCHAR>(),
    dwHungTimeOut: 0,
    bShowWarning: 0,
};

pub const ACCESS_UTILMAN_KEY: LPCWSTR = {
    const S: &[u16] = &[
        'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
        '\\' as u16,
        'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16,
        '\\' as u16,
        'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, ' ' as u16,
        'N' as u16, 'T' as u16,
        '\\' as u16,
        'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
        'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        '\\' as u16,
        'A' as u16, 'c' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, 'i' as u16, 'b' as u16,
        'i' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16,
        '\\' as u16,
        'U' as u16, 't' as u16, 'i' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, ' ' as u16,
        'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16,
        0,
    ];
    S.as_ptr()
};

pub const UTILMAN_KEY: LPCWSTR = {
    const S: &[u16] = &[
        'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
        '\\' as u16,
        'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16,
        '\\' as u16,
        'U' as u16, 't' as u16, 'i' as u16, 'l' as u16, 'i' as u16, 't' as u16, 'y' as u16, ' ' as u16,
        'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16,
        0,
    ];
    S.as_ptr()
};

pub const OSK_KEY: LPCWSTR = {
    const S: &[u16] = &[
        'O' as u16, 'n' as u16, '-' as u16, 'S' as u16, 'c' as u16, 'r' as u16, 'e' as u16, 'e' as u16, 'n' as u16, ' ' as u16,
        'K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16,
        0,
    ];
    S.as_ptr()
};

pub const MAGNIFIER_KEY: LPCWSTR = {
    const S: &[u16] = &[
        'M' as u16, 'a' as u16, 'g' as u16, 'n' as u16, 'i' as u16, 'f' as u16, 'i' as u16, 'e' as u16, 'r' as u16,
        0,
    ];
    S.as_ptr()
};
#[no_mangle]
pub unsafe extern "C" fn InitAppRegKey(
    mut hPredefinedKey: HKEY,
    mut lpszSubKey: LPCWSTR,
    mut phKey: PHKEY,
    mut lpdwDisposition: LPDWORD,
) -> BOOL {
    let mut lResult: LONG;

    lResult = RegCreateKeyExW(
        hPredefinedKey,
        lpszSubKey,
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        phKey,
        lpdwDisposition,
    );
    if lResult != (ERROR_SUCCESS as i32) {
        DPRINT!(
            b"InitAppRegKey(): Failed to create the following key (or open the key) of path \"%S\". The error code is \"%li\".\n\0".as_ptr() as *const i8,
            lpszSubKey,
            lResult
        );
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn QueryAppSettings(
    hKey: HKEY,
    lpszSubKey: LPCWSTR,
    lpszRegValue: LPCWSTR,
    mut ReturnedData: PVOID,
    lpdwSizeData: LPDWORD,
) -> BOOL {
    let mut lResult: LONG;
    let mut hKeyQueryValue: HKEY = core::ptr::null_mut();

    lResult = RegOpenKeyExW(
        hKey,
        lpszSubKey,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKeyQueryValue),
    );
    if lResult != (ERROR_SUCCESS as i32) {
        DPRINT!(
            "QueryAppSettings(): Failed to open the key of path \"%S\". The error code is \"%li\".\n",
            lpszSubKey,
            lResult
        );
        return (FALSE) as BOOL;
    }

    lResult = RegQueryValueExW(
        hKeyQueryValue,
        lpszRegValue,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ReturnedData) as LPBYTE,
        lpdwSizeData,
    );
    if lResult != (ERROR_SUCCESS as i32) {
        DPRINT!(
            "QueryAppSettings(): Failed to query the data from value \"%S\". The error code is \"%li\".\n",
            lpszRegValue,
            lResult
        );
        RegCloseKey(hKeyQueryValue);
        return (FALSE) as BOOL;
    }

    RegCloseKey(hKeyQueryValue);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SaveAppSettings(
    hKey: HKEY,
    lpszRegValue: LPCWSTR,
    dwRegType: DWORD,
    mut Data: PVOID,
    cbSize: DWORD,
) -> BOOL {
    let mut lResult: LONG;
    let mut hKeySetValue: HKEY = core::ptr::null_mut();

    lResult = RegOpenKeyExW(
        hKey,
        core::ptr::null(),
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hKeySetValue),
    ) as LONG;
    if lResult != (ERROR_SUCCESS as i32) {
        DPRINT!(
            "SaveAppSettings(): Failed to open the key, the error code is \"%li\"!\n",
            lResult
        );
        return (FALSE) as BOOL;
    }

    lResult = RegSetValueExW(
        hKeySetValue,
        lpszRegValue,
        0,
        dwRegType,
        core::ptr::addr_of_mut!(Data) as LPBYTE,
        cbSize,
    ) as LONG;
    if lResult != (ERROR_SUCCESS as i32) {
        DPRINT!(
            "SaveAppSettings(): Failed to set the \"%S\" value with data, the error code is \"%li\"!\n",
            lpszRegValue,
            lResult
        );
        RegCloseKey(hKeySetValue);
        return (FALSE) as BOOL;
    }

    RegCloseKey(hKeySetValue);
    return (TRUE) as BOOL;
}
