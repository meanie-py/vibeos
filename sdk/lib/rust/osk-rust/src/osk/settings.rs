// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_05AEA91AC982: [u16; 11] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x004C, 0x0065, 0x0066, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_12A941F4C3F1: [u16; 19] = [
    0x0049u16, 0x0073, 0x0045, 0x006E, 0x0068, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064,
    0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_3F1E8D3C78FF: [u16; 11] = [
    0x0046u16, 0x006F, 0x006E, 0x0074, 0x0048, 0x0065, 0x0069, 0x0067, 0x0068, 0x0074,
    0x0000,
];
pub static RUSTCONV_WSTR_491507152B61: [u16; 13] = [
    0x004Du16, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044,
    0x006C, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_552B5C3D0AC2: [u16; 10] = [0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0054, 0x006F, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_73783670B323: [u16; 12] = [
    0x0053u16, 0x0068, 0x006F, 0x0077, 0x0057, 0x0061, 0x0072, 0x006E, 0x0069, 0x006E,
    0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_8063BF3535C4: [u16; 11] = [
    0x0043u16, 0x006C, 0x0069, 0x0063, 0x006B, 0x0053, 0x006F, 0x0075, 0x006E, 0x0064,
    0x0000,
];
pub static RUSTCONV_WSTR_84D79009C680: [u16; 13] = [
    0x0046u16, 0x006F, 0x006E, 0x0074, 0x0046, 0x0061, 0x0063, 0x0065, 0x004E, 0x0061,
    0x006D, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_D8C4AE18D547: [u16; 23] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x004F,
    0x0073, 0x006B, 0x0000,
];
pub static RUSTCONV_WSTR_EC430604E483: [u16; 12] = [
    0x0041u16, 0x006C, 0x0077, 0x0061, 0x0079, 0x0073, 0x004F, 0x006E, 0x0054, 0x006F,
    0x0070, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn LoadDWORDFromRegistry(
    lpValueDataName: LPCWSTR,
    pdwValueData: PDWORD,
) -> LONG {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG;
    let mut dwValue: DWORD = 0;
    let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    /* Initialize the pointer parameter to default */
    *pdwValueData = 0;

    /* Open our application's key in order to load its configuration data */
    lResult = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_D8C4AE18D547.as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    );

    if lResult != ERROR_SUCCESS as LONG {
        /* Bail out */
        DPRINT!("LoadDWORDFromRegistry(): Failed to open the application's key! (Error - %li)\n", lResult);
        return lResult;
    }

    /* Load the specific value based on the parameter caller, lpValueDataName */
    lResult = RegQueryValueExW(
        hKey,
        lpValueDataName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwValue) as *mut u8,
        &mut cbData,
    );

    if lResult != ERROR_SUCCESS as LONG {
        /* Bail out */
        DPRINT!("LoadDWORDFromRegistry(): Failed to load the following value - \"%S\". (Error - %li)\n", lpValueDataName, lResult);
        RegCloseKey(hKey);
        return lResult;
    }

    /* Is the buffer's size too small to query the required data? */
    if cbData as usize != core::mem::size_of::<DWORD>() {
        /* It is therefore bail out */
        DPRINT!("LoadDWORDFromRegistry(): The buffer is too small to hold the data!\n");
        RegCloseKey(hKey);
        return ERROR_MORE_DATA as LONG;
    }

    *pdwValueData = dwValue;
    RegCloseKey(hKey);
    return lResult;
}
#[no_mangle]
pub unsafe extern "C" fn LoadStringFromRegistry(
    lpValueDataName: LPCWSTR,
    lpValueData: LPWSTR,
    cchCount: LPUINT,
) -> LONG {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG;
    let mut cbCount: UINT;

    cbCount = (*cchCount).wrapping_mul(core::mem::size_of::<u16>() as UINT);

    /* Open our application's key in order to load its configuration data */
    lResult = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_D8C4AE18D547.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"LoadStringFromRegistry(): Failed to open the application's key! (Error - %li)\n\0".as_ptr() as *const i8,
            lResult,
        );
        return lResult;
    }

    /* Load the specific value based on the parameter caller, lpValueDataName */
    lResult = RegQueryValueExW(
        hKey,
        lpValueDataName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpValueData as *mut u8,
        core::ptr::addr_of_mut!(cbCount) as LPDWORD,
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"LoadStringFromRegistry(): Failed to load the following value - \"%S\". (Error - %li)\n\0".as_ptr() as *const i8,
            lpValueDataName,
            lResult,
        );
        RegCloseKey(hKey);
        return lResult;
    }

    *cchCount = cbCount / (core::mem::size_of::<u16>() as UINT);

    RegCloseKey(hKey);
    return lResult;
}
#[no_mangle]
pub unsafe extern "C" fn SaveDWORDToRegistry(lpValueDataName: LPCWSTR, mut dwValueData: DWORD) -> LONG {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG;

    /* Set up the application's key in case it has not been made yet */
    lResult = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_D8C4AE18D547.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"SaveDWORDToRegistry(): Failed to create the application's key! (Error - %li)\n\0".as_ptr() as *const i8,
            lResult,
        );
        return lResult;
    }

    /* Save the data into the registry value */
    lResult = RegSetValueExW(
        hKey,
        lpValueDataName,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(dwValueData) as *mut BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"SaveDWORDToRegistry(): Failed to save the following value - \"%S\". (Error - %li)\n\0".as_ptr() as *const i8,
            lpValueDataName,
            lResult,
        );
        RegCloseKey(hKey);
        return lResult;
    }

    RegCloseKey(hKey);
    return lResult;
}
#[no_mangle]
pub unsafe extern "C" fn SaveStringToRegistry(
    lpValueDataName: LPCWSTR,
    lpValueData: LPCWSTR,
    cchCount: UINT,
) -> LONG {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lResult: LONG;

    /* Set up the application's key in case it has not been made yet */
    lResult = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        RUSTCONV_WSTR_D8C4AE18D547.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"SaveStringToRegistry(): Failed to create the application's key! (Error - %li)\n\0".as_ptr() as *const i8,
            lResult,
        );
        return lResult;
    }

    /* Save the data into the registry value */
    lResult = RegSetValueExW(
        hKey,
        lpValueDataName,
        0,
        REG_SZ,
        lpValueData as *mut BYTE,
        (cchCount as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
    );

    if (lResult) as u32 != ERROR_SUCCESS {
        /* Bail out */
        DPRINT!(
            b"SaveStringToRegistry(): Failed to save the following value - \"%S\". (Error - %li)\n\0".as_ptr() as *const i8,
            lpValueDataName,
            lResult,
        );
        RegCloseKey(hKey);
        return lResult;
    }

    RegCloseKey(hKey);
    return lResult;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSettings() {
    let mut dwValue: DWORD = 0;
    let mut lResult: LONG = 0;

    /* Initialize the registry application settings */
    Globals.bShowWarning = (TRUE) as i32;
    Globals.bIsEnhancedKeyboard = (TRUE) as i32;
    Globals.bAlwaysOnTop = (TRUE) as i32;
    Globals.bSoundClick = (FALSE) as i32;

    /* Set the coordinate values to default */
    Globals.PosX = CW_USEDEFAULT;
    Globals.PosY = CW_USEDEFAULT;

    /* Set font value defaults */
    Globals.FontHeight = (DEFAULT_FONTSIZE as i32);

    /* Warning dialog registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_73783670B323.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.bShowWarning = (dwValue != 0) as BOOL;
    }

    /* Enhanced keyboard switch dialog registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_12A941F4C3F1.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.bIsEnhancedKeyboard = (dwValue != 0) as BOOL;
    }

    /* Sound on click event registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_8063BF3535C4.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.bSoundClick = (dwValue != 0) as BOOL;
    }

    /* X coordinate dialog placement registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_05AEA91AC982.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.PosX = dwValue as INT;
    }

    /* Y coordinate dialog placement registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_552B5C3D0AC2.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.PosY = dwValue as INT;
    }

    /* Top window state registry setting */
    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_EC430604E483.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.bAlwaysOnTop = (dwValue != 0) as BOOL;
    }

    /* Font information */
    let mut cchCount: UINT = _countof!(Globals.FontFaceName) as UINT;
    lResult = LoadStringFromRegistry(
        RUSTCONV_WSTR_84D79009C680.as_ptr(),
        core::ptr::addr_of_mut!(Globals.FontFaceName) as LPWSTR,
        core::ptr::addr_of_mut!(cchCount),
    );

    if (lResult) as u32 != NO_ERROR {
        /* Copy default on failure */
        StringCchCopyW(
            core::ptr::addr_of_mut!(Globals.FontFaceName) as LPWSTR,
            _countof!(Globals.FontFaceName) as usize,
            RUSTCONV_WSTR_491507152B61.as_ptr(),
        );
    }

    lResult = LoadDWORDFromRegistry(RUSTCONV_WSTR_3F1E8D3C78FF.as_ptr(), core::ptr::addr_of_mut!(dwValue));
    if (lResult) as u32 == NO_ERROR {
        Globals.FontHeight = dwValue as LONG;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() {
    let mut wp: WINDOWPLACEMENT = core::mem::zeroed();

    /* Initialize the window placement structure */
    wp.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    GetWindowPlacement(Globals.hMainWnd, core::ptr::addr_of_mut!(wp));

    /* Warning dialog registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_73783670B323.as_ptr() as LPCWSTR, Globals.bShowWarning as DWORD);

    /* Enhanced keyboard switch dialog registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_12A941F4C3F1.as_ptr() as LPCWSTR, Globals.bIsEnhancedKeyboard as DWORD);

    /* Sound on click event registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_8063BF3535C4.as_ptr() as LPCWSTR, Globals.bSoundClick as DWORD);

    /* X coordinate dialog placement registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_05AEA91AC982.as_ptr() as LPCWSTR, wp.rcNormalPosition.left as DWORD);

    /* Y coordinate dialog placement registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_552B5C3D0AC2.as_ptr() as LPCWSTR, wp.rcNormalPosition.top as DWORD);

    /* Top window state registry setting */
    SaveDWORDToRegistry(RUSTCONV_WSTR_EC430604E483.as_ptr() as LPCWSTR, Globals.bAlwaysOnTop as DWORD);

    /* Font information */
    SaveStringToRegistry(
        RUSTCONV_WSTR_84D79009C680.as_ptr() as LPCWSTR,
        core::ptr::addr_of_mut!(Globals.FontFaceName) as *mut WCHAR as LPCWSTR,
        _countof!(Globals.FontFaceName) as UINT,
    );
    SaveDWORDToRegistry(RUSTCONV_WSTR_3F1E8D3C78FF.as_ptr() as LPCWSTR, Globals.FontHeight as DWORD);
}
