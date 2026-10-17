// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0BA7EF90615C: [u16; 10] = [0x0025u16, 0x0073, 0x003A, 0x0073, 0x003A, 0x0025, 0x0073, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_76E86BF0A91D: [u16; 15] = [
    0x0073u16, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0020, 0x006D, 0x006F, 0x0064,
    0x0065, 0x0020, 0x0069, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_811DD8479E61: [u16; 13] = [
    0x005Cu16, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x002E, 0x0072,
    0x0064, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_8F802E85B5A5: [u16; 13] = [
    0x0066u16, 0x0075, 0x006C, 0x006C, 0x0020, 0x0061, 0x0064, 0x0064, 0x0072, 0x0065,
    0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_9783FB7F6725: [u16; 10] = [0x0025u16, 0x0073, 0x003A, 0x0069, 0x003A, 0x0025, 0x0064, 0x000D, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_97F575EF508B: [u16; 13] = [
    0x0064u16, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0077, 0x0069, 0x0064,
    0x0074, 0x0068, 0x0000,
];
pub static RUSTCONV_WSTR_993EF02FC0D2: [u16; 9] = [0x0075u16, 0x0073, 0x0065, 0x0072, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_CDA991128999: [u16; 12] = [
    0x0073u16, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0062, 0x0070,
    0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_D8EF2D6777E6: [u16; 14] = [
    0x0064u16, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0068, 0x0065, 0x0069,
    0x0067, 0x0068, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_76E86BF0A91D: [u16; 15] = [
    0x0073u16, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0020, 0x006D, 0x006F, 0x0064,
    0x0065, 0x0020, 0x0069, 0x0064, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_8F802E85B5A5: [u16; 13] = [
    0x0066u16, 0x0075, 0x006C, 0x006C, 0x0020, 0x0061, 0x0064, 0x0064, 0x0072, 0x0065,
    0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_97F575EF508B: [u16; 13] = [
    0x0064u16, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0077, 0x0069, 0x0064,
    0x0074, 0x0068, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_993EF02FC0D2: [u16; 9] = [0x0075u16, 0x0073, 0x0065, 0x0072, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_C2R_WSTR_CDA991128999: [u16; 12] = [
    0x0073u16, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0020, 0x0062, 0x0070,
    0x0070, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D8EF2D6777E6: [u16; 14] = [
    0x0064u16, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0068, 0x0065, 0x0069,
    0x0067, 0x0068, 0x0074, 0x0000,
];

#[no_mangle]
pub static mut lpSettings: [LPWSTR; 6] = unsafe {
    [
        RUSTCONV_C2R_WSTR_97F575EF508B.as_ptr() as LPWSTR,
        RUSTCONV_C2R_WSTR_D8EF2D6777E6.as_ptr() as LPWSTR,
        RUSTCONV_C2R_WSTR_CDA991128999.as_ptr() as LPWSTR,
        RUSTCONV_C2R_WSTR_8F802E85B5A5.as_ptr() as LPWSTR,
        RUSTCONV_C2R_WSTR_993EF02FC0D2.as_ptr() as LPWSTR,
        RUSTCONV_C2R_WSTR_76E86BF0A91D.as_ptr() as LPWSTR,
    ]
};

static mut OPEN_RDP_FILE_UNUSED: i32 = 0;
#[no_mangle]
pub unsafe extern "C" fn SaveAllSettings(pInfo: PINFO) {
    let mut ret: INT;
    let mut szValue: [WCHAR; (MAXVALUE) as _] = [0; (MAXVALUE) as _];

    /* server */
    if GetDlgItemText(
        (*pInfo).hGeneralPage,
        (IDC_SERVERCOMBO as i32),
        szValue.as_mut_ptr(),
        (MAXVALUE as i32),
    ) != 0
    {
        SetStringToSettings(
            (*pInfo).pRdpSettings,
            RUSTCONV_WSTR_8F802E85B5A5.as_ptr().cast_mut(),
            szValue.as_mut_ptr(),
        );
    }

    /* resolution and fullscreen*/
    ret = SendDlgItemMessage(
        (*pInfo).hDisplayPage,
        (IDC_GEOSLIDER as i32),
        TBM_GETPOS,
        0,
        0,
    ) as INT;
    if ret != -1 {
        SetIntegerToSettings(
            (*pInfo).pRdpSettings,
            RUSTCONV_WSTR_76E86BF0A91D.as_ptr().cast_mut(),
            (if ret as LRESULT
                == SendDlgItemMessageW(
                    (*pInfo).hDisplayPage,
                    (IDC_GEOSLIDER as i32),
                    TBM_GETRANGEMAX,
                    0,
                    0,
                )
            {
                2
            } else {
                1
            }),
        );
        SetIntegerToSettings(
            (*pInfo).pRdpSettings,
            RUSTCONV_WSTR_97F575EF508B.as_ptr().cast_mut(),
            ((*(*(*pInfo).DisplayDeviceList).Resolutions.offset(ret as isize)).dmPelsWidth) as INT,
        );
        SetIntegerToSettings(
            (*pInfo).pRdpSettings,
            RUSTCONV_WSTR_D8EF2D6777E6.as_ptr().cast_mut(),
            ((*(*(*pInfo).DisplayDeviceList).Resolutions.offset(ret as isize)).dmPelsHeight) as INT,
        );
    }

    /* bpp */
    ret = SendDlgItemMessage(
        (*pInfo).hDisplayPage,
        (IDC_BPPCOMBO as i32),
        CB_GETCURSEL,
        0,
        0,
    ) as INT;
    if ret != CB_ERR {
        ret = SendDlgItemMessage(
            (*pInfo).hDisplayPage,
            (IDC_BPPCOMBO as i32),
            CB_GETITEMDATA,
            ret as WPARAM,
            0,
        ) as INT;
        if ret != CB_ERR {
            SetIntegerToSettings(
                (*pInfo).pRdpSettings,
                RUSTCONV_WSTR_CDA991128999.as_ptr().cast_mut(),
                ret,
            );
        }
    }

    /* user name */
    if GetDlgItemText(
        (*pInfo).hGeneralPage,
        (IDC_NAMEEDIT as i32),
        szValue.as_mut_ptr(),
        (MAXVALUE as i32),
    ) != 0
    {
        SetStringToSettings(
            (*pInfo).pRdpSettings,
            RUSTCONV_WSTR_993EF02FC0D2.as_ptr().cast_mut(),
            szValue.as_mut_ptr(),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetIntegerToSettings(
    pRdpSettings: PRDPSETTINGS,
    lpKey: LPWSTR,
    Value: INT,
) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;

    if !pRdpSettings.is_null() {
        let mut i: INT;

        i = 0;
        while i < (*pRdpSettings).NumSettings {
            if wcscmp(
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *const WCHAR,
                lpKey as *const WCHAR,
            ) == 0
            {
                if (*(*pRdpSettings).pSettings.offset(i as isize)).Type == 0 {
                    (*(*pRdpSettings).pSettings.offset(i as isize)).Type = 0x0069u16;
                }

                (*(*pRdpSettings).pSettings.offset(i as isize)).Value.i = Value;
                bRet = (TRUE) as i32;
                break;
            }
            i += 1;
        }
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn SetStringToSettings(
    pRdpSettings: PRDPSETTINGS,
    lpKey: LPWSTR,
    lpValue: LPWSTR,
) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;

    if !pRdpSettings.is_null() {
        let mut i: INT;

        i = 0;
        while i < (*pRdpSettings).NumSettings {
            if wcscmp(
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                lpKey as *const wchar_t,
            ) == 0
            {
                if (*(*pRdpSettings).pSettings.offset(i as isize)).Type == 0 {
                    (*(*pRdpSettings).pSettings.offset(i as isize)).Type = 0x0073u16;
                }

                // Bounded like the file path above: Value.s holds MAXVALUE WCHARs.
                StringCchCopyW(
                    core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Value.s) as *mut WCHAR,
                    MAXVALUE as usize,
                    lpValue as *const WCHAR,
                );
                bRet = (TRUE) as i32;
                break;
            }
            i += 1;
        }
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn GetIntegerFromSettings(mut pRdpSettings: PRDPSETTINGS, mut lpKey: LPWSTR) -> INT {
    let mut Value: INT = -(1 as INT);
    if !pRdpSettings.is_null() {
        let mut i: INT = 0;
        i = 0 as INT;
        while i < (*pRdpSettings).NumSettings {
            if wcscmp(
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                lpKey as *const WCHAR,
            ) == 0
            {
                if (*(*pRdpSettings).pSettings.offset(i as isize)).Type == 0x0069u16 {
                    Value = (*(*pRdpSettings).pSettings.offset(i as isize)).Value.i;
                    break;
                }
            }
            i += 1;
        }
    }
    return Value;
}
#[no_mangle]
pub unsafe extern "C" fn GetStringFromSettings(pRdpSettings: PRDPSETTINGS, lpKey: LPWSTR) -> LPWSTR {
    let mut lpValue: LPWSTR = core::ptr::null_mut();

    if !pRdpSettings.is_null() {
        let mut i: INT;

        i = 0;
        while i < (*pRdpSettings).NumSettings {
            if wcscmp(
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                lpKey as *const u16,
            ) == 0
            {
                if (*(*pRdpSettings).pSettings.offset(i as isize)).Type == 0x0073u16 {
                    lpValue = core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Value.s)
                        as *mut WCHAR as LPWSTR;
                    break;
                }
            }
            i += 1;
        }
    }

    return lpValue;
}

unsafe extern "C" fn WriteRdpFile(mut hFile: HANDLE, mut pRdpSettings: PRDPSETTINGS) -> BOOL {
    let mut line: [WCHAR; (MAXKEY + MAXVALUE + 4) as usize] =
        [0; (MAXKEY + MAXVALUE + 4) as usize];
    let mut BytesToWrite: SIZE_T = 0;
    let mut BytesWritten: ULONG = 0;
    let mut bRet: BOOL = 0;
    let mut i: INT = 0;
    let mut k: INT = 0;
    i = 0;
    while i < (*pRdpSettings).NumSettings {
        k = 0;
        while k < (NUM_SETTINGS as i32) {
            if wcscmp(
                lpSettings[k as usize] as *const WCHAR,
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
            ) == 0
            {
                if (*(*pRdpSettings).pSettings.offset(i as isize)).Type == 0x0069u16 {
                    _snwprintf(
                        core::ptr::addr_of_mut!(line) as *mut WCHAR,
                        (MAXKEY + MAXVALUE + 4) as usize,
                        RUSTCONV_WSTR_9783FB7F6725.as_ptr(),
                        core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                        (*(*pRdpSettings).pSettings.offset(i as isize)).Value.i,
                    );
                } else {
                    _snwprintf(
                        core::ptr::addr_of_mut!(line) as *mut WCHAR,
                        (MAXKEY + MAXVALUE + 4) as usize,
                        RUSTCONV_WSTR_0BA7EF90615C.as_ptr(),
                        core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                        core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Value.s) as *mut WCHAR,
                    );
                }

                BytesToWrite = (wcslen(core::ptr::addr_of_mut!(line) as *mut WCHAR) as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as SIZE_T;

                bRet = WriteFile(
                    hFile,
                    core::ptr::addr_of_mut!(line) as *mut WCHAR as LPCVOID,
                    BytesToWrite as DWORD,
                    core::ptr::addr_of_mut!(BytesWritten),
                    core::ptr::null_mut::<_OVERLAPPED>(),
                );
                if bRet == 0 || BytesWritten == 0 {
                    return (FALSE) as BOOL;
                }
            }
            k += 1;
        }
        i += 1;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ParseSettings(pRdpSettings: PRDPSETTINGS, lpBuffer: LPWSTR) {
    let mut lpStr: LPWSTR = lpBuffer;
    let mut szSeps: [WCHAR; 4] = [0x003Au16, 0x000D, 0x000A, 0x0000];
    let mut szNewline: [WCHAR; 3] = [0x000Du16, 0x000A, 0x0000];
    let mut lpToken: LPWSTR = core::ptr::null_mut();
    let mut bFound: BOOL = 0;
    let mut i: INT = 0;

    /* move past unicode byte order */
    if *lpStr.offset(0 as isize) == 0xFEFF || *lpStr.offset(0 as isize) == 0xFFFE {
        lpStr = lpStr.offset(1 as isize);
    }

    lpToken = wcstok(lpStr, core::ptr::addr_of_mut!(szSeps) as *mut WCHAR) as LPWSTR;
    while !lpToken.is_null() {
        bFound = FALSE as BOOL;

        i = 0 as INT;
        while i < (*pRdpSettings).NumSettings && bFound == 0 {
            if wcscmp(
                lpToken as *const WCHAR,
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
            ) == 0
            {
                lpToken = wcstok(core::ptr::null_mut(), core::ptr::addr_of_mut!(szSeps) as *mut WCHAR) as LPWSTR;
                if *lpToken.offset(0 as isize) == 0x0069u16 {
                    (*(*pRdpSettings).pSettings.offset(i as isize)).Type = *lpToken.offset(0 as isize);
                    lpToken = wcstok(core::ptr::null_mut(), core::ptr::addr_of_mut!(szSeps) as *mut WCHAR) as LPWSTR;
                    if !lpToken.is_null() {
                        (*(*pRdpSettings).pSettings.offset(i as isize)).Value.i = _wtoi(lpToken as *const WCHAR) as INT;
                    }
                } else if *lpToken.offset(0 as isize) == 0x0073u16 {
                    (*(*pRdpSettings).pSettings.offset(i as isize)).Type = *lpToken.offset(0 as isize);
                    if *lpToken.offset(2 as isize) == 13
                        || *lpToken.offset(2 as isize) == 10
                        || *lpToken.offset(2 as isize) == 0
                    {
                        // terminate string
                        *lpToken.offset(0 as isize) = 0;
                    } else {
                        lpToken = wcstok(core::ptr::null_mut(), core::ptr::addr_of_mut!(szNewline) as *mut WCHAR) as LPWSTR;
                    }
                    if !lpToken.is_null() {
                        // The C's wcscpy into Value.s (MAXVALUE WCHARs) takes the token from
                        // the .rdp file unbounded; a longer value overran the settings array.
                        // Truncated instead, always terminated.
                        StringCchCopyW(
                            core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Value.s) as *mut WCHAR,
                            MAXVALUE as usize,
                            lpToken as *const WCHAR,
                        );
                    }
                }
                bFound = TRUE as BOOL;
            }
            i += 1;
        }

        /* move past the type and value */
        if bFound == 0 {
            lpToken = wcstok(core::ptr::null_mut(), core::ptr::addr_of_mut!(szNewline) as *mut WCHAR) as LPWSTR;
        }

        /* move to next key */
        lpToken = wcstok(core::ptr::null_mut(), core::ptr::addr_of_mut!(szSeps) as *mut WCHAR) as LPWSTR;
    }
}

unsafe extern "C" fn ReadRdpFile(hFile: HANDLE) -> LPWSTR {
    let mut lpBuffer: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut BytesToRead: DWORD = 0;
    let mut BytesRead: DWORD = 0;
    let mut bRes: BOOL = 0;
    if !hFile.is_null() {
        BytesToRead = GetFileSize(hFile, core::ptr::null_mut::<DWORD>());
        if BytesToRead != 0 {
            lpBuffer = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                BytesToRead.wrapping_add(2 as DWORD) as SIZE_T,
            ) as LPWSTR;
            if !lpBuffer.is_null() {
                bRes = ReadFile(
                    hFile,
                    lpBuffer as PVOID,
                    BytesToRead,
                    core::ptr::addr_of_mut!(BytesRead),
                    core::ptr::null_mut::<_OVERLAPPED>(),
                );
                if bRes != 0 {
                    *lpBuffer.offset(BytesRead.wrapping_div(2 as DWORD) as isize) =
                        0 as WCHAR;
                } else {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        lpBuffer as PVOID,
                    );
                    lpBuffer = core::ptr::null_mut::<WCHAR>();
                }
            }
        }
    }
    return lpBuffer;
}

unsafe extern "C" fn OpenRdpFile(path: LPWSTR, bWrite: BOOL) -> HANDLE {
    let mut hFile: HANDLE = core::ptr::null_mut();

    if !path.is_null() {
        hFile = CreateFileW(
            path,
            if bWrite != 0 { GENERIC_WRITE } else { GENERIC_READ },
            0,
            core::ptr::null_mut(),
            if bWrite != 0 { CREATE_ALWAYS } else { OPEN_EXISTING },
            FILE_ATTRIBUTE_NORMAL | FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        );
    }

    return hFile;
}

unsafe extern "C" fn CloseRdpFile(hFile: HANDLE) {
    if !hFile.is_null() {
        CloseHandle(hFile);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SaveRdpSettingsToFile(mut lpFile: LPWSTR, mut pRdpSettings: PRDPSETTINGS) -> BOOL {
    let mut pszPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;

    /* use default file */
    if lpFile.is_null() {
        let mut hr: HRESULT;
        let mut lpidl: LPITEMIDLIST = core::ptr::null_mut();

        hr = SHGetFolderLocation(
            core::ptr::null_mut(),
            (CSIDL_PERSONAL as i32),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(lpidl),
        );
        if hr == S_OK {
            if SHGetPathFromIDListW(lpidl, core::ptr::addr_of_mut!(pszPath) as LPWSTR) != 0 {
                wcscat(
                    core::ptr::addr_of_mut!(pszPath) as *mut WCHAR,
                    RUSTCONV_WSTR_811DD8479E61.as_ptr(),
                );
                lpFile = core::ptr::addr_of_mut!(pszPath) as LPWSTR;
                CoTaskMemFree(lpidl as LPVOID);
            }
        }
    }

    if !lpFile.is_null() {
        hFile = OpenRdpFile(lpFile, (TRUE) as BOOL);
        if !hFile.is_null() {
            if WriteRdpFile(hFile, pRdpSettings) != 0 {
                bRet = (TRUE) as i32;
            }

            CloseRdpFile(hFile);
        }
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn LoadRdpSettingsFromFile(mut pRdpSettings: PRDPSETTINGS, mut lpFile: LPWSTR) -> BOOL {
    let mut pszPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;

    /* use default file */
    if lpFile.is_null() {
        let mut hr: HRESULT;
        let mut lpidl: LPITEMIDLIST = core::ptr::null_mut();

        hr = SHGetFolderLocation(
            core::ptr::null_mut(),
            (CSIDL_PERSONAL as i32),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(lpidl),
        );
        if hr == S_OK {
            if SHGetPathFromIDListW(lpidl, core::ptr::addr_of_mut!(pszPath) as LPWSTR) != 0 {
                wcscat(
                    core::ptr::addr_of_mut!(pszPath) as *mut WCHAR,
                    RUSTCONV_WSTR_811DD8479E61.as_ptr(),
                );
                lpFile = core::ptr::addr_of_mut!(pszPath) as *mut WCHAR as LPWSTR;
                CoTaskMemFree(lpidl as LPVOID);
            }
        }
    }

    if !lpFile.is_null() {
        let mut lpBuffer: LPWSTR = core::ptr::null_mut();

        hFile = OpenRdpFile(lpFile, (FALSE) as BOOL);
        if !hFile.is_null() {
            lpBuffer = ReadRdpFile(hFile);
            if !lpBuffer.is_null() {
                ParseSettings(pRdpSettings, lpBuffer);

                HeapFree(
                    GetProcessHeap(),
                    0,
                    lpBuffer as PVOID,
                );

                bRet = (TRUE) as i32;
            }

            CloseRdpFile(hFile);
        }
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn InitRdpSettings(pRdpSettings: PRDPSETTINGS) -> BOOL {
    let mut bRet: BOOL = FALSE as BOOL;

    (*pRdpSettings).pSettings = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (core::mem::size_of::<SETTINGS>() as SIZE_T).wrapping_mul(NUM_SETTINGS as SIZE_T),
    ) as PSETTINGS;
    if !(*pRdpSettings).pSettings.is_null() {
        let mut i: INT = 0;

        i = 0 as INT;
        while i < NUM_SETTINGS as INT {
            wcscpy(
                core::ptr::addr_of_mut!((*(*pRdpSettings).pSettings.offset(i as isize)).Key) as *mut WCHAR,
                lpSettings[i as usize] as *const WCHAR,
            );
            (*(*pRdpSettings).pSettings.offset(i as isize)).Type = 0 as WCHAR;
            (*(*pRdpSettings).pSettings.offset(i as isize)).Value.i = 0 as INT;
            i += 1;
        }

        (*pRdpSettings).NumSettings = NUM_SETTINGS as INT;

        bRet = TRUE as BOOL;
    }

    return bRet;
}
