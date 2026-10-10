// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0015E866C5D4: [u16; 15] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_1CDEBCAEE168: [u16; 12] = [
    0x0050u16, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_24B5C33A15AA: [u16; 11] = [
    0x0043u16, 0x0053, 0x0044, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E,
    0x0000,
];
pub static RUSTCONV_WSTR_FDA0860BBFB4: [u16; 19] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0042, 0x0075, 0x0069,
    0x006C, 0x0064, 0x004E, 0x0075, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const OSINFO_KEY: [::core::ffi::c_ushort; 45] = unsafe {
    ::core::mem::transmute::<
        [u8; 90],
        [::core::ffi::c_ushort; 45],
    >(
        *b"S\0O\0F\0T\0W\0A\0R\0E\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\0\0",
    )
};

unsafe extern "C" fn Winver_GetRegValueString(
    mut hKey: HKEY,
    mut pValue: LPCWSTR,
    mut pBuffer: LPWSTR,
    mut cchSize: DWORD,
) {
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut lError: LSTATUS = 0;

    /* NOTE: Reserved space for a NULL terminator */
    dwSize = (cchSize.wrapping_sub(1 as DWORD) as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    lError = RegQueryValueExW(
        hKey,
        pValue,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        pBuffer as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );
    if lError != ERROR_SUCCESS as LSTATUS || dwType != REG_SZ as DWORD
    {
        /* Return empty string on failure */
        *pBuffer.offset(0 as isize) = UNICODE_NULL as WCHAR;
        return;
    }

    /* Ensure the returned string is NULL terminated */
    *pBuffer.offset(cchSize.wrapping_sub(1 as DWORD) as isize) =
        UNICODE_NULL as WCHAR;
}

unsafe extern "C" fn Winver_GetFormattedSpkInfo(
    mut hKey: HKEY,
    mut pBuffer: LPWSTR,
    mut cchSize: DWORD,
) {
    let mut szRegValue: [WCHAR; 48] = [0; 48];
    let mut szFormat: [WCHAR; 16] = [0; 16];

    Winver_GetRegValueString(
        hKey,
        RUSTCONV_WSTR_24B5C33A15AA.as_ptr(),
        core::ptr::addr_of_mut!(szRegValue) as LPWSTR,
        _countof!(szRegValue),
    );
    if szRegValue[0] == 0 {
        /* Return empty string on failure */
        *pBuffer.offset(0 as isize) = UNICODE_NULL;
        return;
    }

    LoadStringW(
        Winver_hInstance,
        IDS_OSINFO_SPK_FORMAT as UINT,
        core::ptr::addr_of_mut!(szFormat) as LPWSTR,
        _countof!(szFormat) as core::ffi::c_int,
    );

    StringCchPrintfW!(
        pBuffer,
        cchSize,
        core::ptr::addr_of_mut!(szFormat) as LPCWSTR,
        core::ptr::addr_of_mut!(szRegValue) as LPCWSTR,
    );
}

unsafe extern "C" fn Winver_FormatCompatInfo(
    mut hKey: HKEY,
    mut pBuffer: LPWSTR,
    mut cchSize: DWORD,
) {
    let mut szNtVersion: [WCHAR; 16] = [0; 16];
    let mut szNtBuild: [WCHAR; 16] = [0; 16];
    let mut szNtSpk: [WCHAR; 64] = [0; 64];
    let mut szFormat: [WCHAR; 64] = [0; 64];

    /* NOTE: Required info must be valid */
    Winver_GetRegValueString(
        hKey,
        RUSTCONV_WSTR_0015E866C5D4.as_ptr(),
        &raw mut szNtVersion as LPWSTR,
        _countof!(szNtVersion) as DWORD,
    );
    Winver_GetRegValueString(
        hKey,
        RUSTCONV_WSTR_FDA0860BBFB4.as_ptr(),
        &raw mut szNtBuild as LPWSTR,
        _countof!(szNtBuild) as DWORD,
    );
    if szNtVersion[0] == 0 || szNtBuild[0] == 0
    {
        /* Return empty string on failure */
        *pBuffer.offset(0 as isize) = UNICODE_NULL;
        return;
    }

    /* NOTE: Service pack info is optional */
    Winver_GetFormattedSpkInfo(
        hKey,
        &raw mut szNtSpk as LPWSTR,
        _countof!(szNtSpk) as DWORD,
    );

    LoadStringW(
        Winver_hInstance,
        IDS_OSINFO_COMPAT_FORMAT as UINT,
        &raw mut szFormat as LPWSTR,
        _countof!(szFormat) as ::core::ffi::c_int,
    );

    StringCchPrintfW!(
        pBuffer,
        cchSize as size_t,
        &raw mut szFormat as *mut WCHAR as STRSAFE_LPCWSTR,
        &raw mut szNtVersion as *mut WCHAR,
        &raw mut szNtBuild as *mut WCHAR,
        &raw mut szNtSpk as *mut WCHAR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Winver_GetOSInfo(OSInfo: PWINVER_OS_INFO) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let lError: LSTATUS;
    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           (OSINFO_KEY).as_ptr(),
                           0,
                           KEY_QUERY_VALUE,
                           core::ptr::addr_of_mut!(hKey));
    if (lError) as u32 != ERROR_SUCCESS {
        return (FALSE) as BOOL;
    }

    /* OS name */
    Winver_GetRegValueString(hKey, RUSTCONV_WSTR_1CDEBCAEE168.as_ptr(), core::ptr::addr_of_mut!((*OSInfo).szName) as LPWSTR, _countof!((*OSInfo).szName));
    if (*OSInfo).szName[0 as usize] == 0 {
        /* This info must be valid */
        RegCloseKey(hKey);
        return (FALSE) as BOOL;
    }

    /* Compatibility information */
    Winver_FormatCompatInfo(hKey, core::ptr::addr_of_mut!((*OSInfo).szCompatInfo) as LPWSTR, _countof!((*OSInfo).szCompatInfo));

    RegCloseKey(hKey);

    return (TRUE) as BOOL;
}
