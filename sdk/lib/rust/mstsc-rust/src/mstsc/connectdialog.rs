use crate::mstsc::settings::GetIntegerFromSettings;
use crate::mstsc::settings::GetStringFromSettings;
use crate::mstsc::settings::LoadRdpSettingsFromFile;
use crate::mstsc::settings::SaveAllSettings;
use crate::mstsc::settings::SaveRdpSettingsToFile;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4D70F4A881C7: [u16; 2] = [0x003Au16, 0x0000];
pub static RUSTCONV_WSTR_69BE2E3F4A5C: [u16; 7] = [0x0054u16, 0x0061, 0x0068, 0x006F, 0x006D, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_76E86BF0A91D: [u16; 15] = [
    0x0073u16, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0020, 0x006D, 0x006F, 0x0064,
    0x0065, 0x0020, 0x0069, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_7E99F0C2EB7C: [u16; 10] = [0x0052u16, 0x0044, 0x0050, 0x0020, 0x0065, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_8D0582078CBE: [u16; 25] = [
    0x0053u16, 0x004D, 0x0042, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074,
    0x0020, 0x0079, 0x0065, 0x0074, 0x0020, 0x0073, 0x0075, 0x0070, 0x0070, 0x006F,
    0x0072, 0x0074, 0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_8F802E85B5A5: [u16; 13] = [
    0x0066u16, 0x0075, 0x006C, 0x006C, 0x0020, 0x0061, 0x0064, 0x0064, 0x0072, 0x0065,
    0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_97F575EF508B: [u16; 13] = [
    0x0064u16, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0077, 0x0069, 0x0064,
    0x0074, 0x0068, 0x0000,
];
pub static RUSTCONV_WSTR_993EF02FC0D2: [u16; 9] = [0x0075u16, 0x0073, 0x0065, 0x0072, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_A3B740B38441: [u16; 4] = [0x0052u16, 0x0044, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_A90688F55D61: [u16; 13] = [
    0x0055u16, 0x0073, 0x0065, 0x0072, 0x006E, 0x0061, 0x006D, 0x0065, 0x0048, 0x0069,
    0x006E, 0x0074, 0x0000,
];
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
#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub const MAX_KEY_NAME: i32 = 255;
unsafe extern "C" fn DoOpenFile(pInfo: PINFO) {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let wide_default: [u16; 12] = ['D' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, '.' as u16, 'r' as u16, 'd' as u16, 'p' as u16, 0];
    let mut i: usize = 0;
    while i < wide_default.len() {
        szFileName[i] = wide_default[i];
        i += 1;
    }
    static mut szFilter: [WCHAR; 36] = [
    0x0052u16, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0020, 0x0044, 0x0065, 0x0073,
    0x006B, 0x0074, 0x006F, 0x0070, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073,
    0x0020, 0x0028, 0x002A, 0x002E, 0x0052, 0x0044, 0x0050, 0x0029, 0x0000, 0x002A,
    0x002E, 0x0072, 0x0064, 0x0070, 0x0000, 0x0000,
];

    core::ptr::write_bytes(&raw mut ofn as *mut u8, 0, core::mem::size_of::<OPENFILENAMEW>());
    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.hwndOwner = (*pInfo).hGeneralPage;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.nMaxFileTitle = MAX_PATH as DWORD;
    ofn.lpstrDefExt = RUSTCONV_WSTR_A3B740B38441.as_ptr() as LPCWSTR;
    ofn.lpstrFilter = &raw mut szFilter as *mut WCHAR as LPCWSTR;
    ofn.lpstrFile = &raw mut szFileName as *mut WCHAR as LPWSTR;
    ofn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST) as DWORD;

    if GetOpenFileNameW(&raw mut ofn) != 0 {
        LoadRdpSettingsFromFile((*pInfo).pRdpSettings, szFileName.as_mut_ptr());
        ReLoadGeneralPage(pInfo);
        ReLoadDisplayPage(pInfo);
    }
}
unsafe extern "C" fn DoSaveAs(pInfo: PINFO) {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    szFileName[0] = 'D' as WCHAR;
    szFileName[1] = 'e' as WCHAR;
    szFileName[2] = 'f' as WCHAR;
    szFileName[3] = 'a' as WCHAR;
    szFileName[4] = 'u' as WCHAR;
    szFileName[5] = 'l' as WCHAR;
    szFileName[6] = 't' as WCHAR;
    szFileName[7] = '.' as WCHAR;
    szFileName[8] = 'r' as WCHAR;
    szFileName[9] = 'd' as WCHAR;
    szFileName[10] = 'p' as WCHAR;
    static mut szFilter: [WCHAR; 36] = [
    0x0052u16, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0020, 0x0044, 0x0065, 0x0073,
    0x006B, 0x0074, 0x006F, 0x0070, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073,
    0x0020, 0x0028, 0x002A, 0x002E, 0x0052, 0x0044, 0x0050, 0x0029, 0x0000, 0x002A,
    0x002E, 0x0072, 0x0064, 0x0070, 0x0000, 0x0000,
];

    ZeroMemory!(
        &raw mut ofn as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );
    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.hwndOwner = (*pInfo).hGeneralPage;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.nMaxFileTitle = MAX_PATH as DWORD;
    ofn.lpstrDefExt = RUSTCONV_WSTR_A3B740B38441.as_ptr() as LPCWSTR;
    ofn.lpstrFilter = &raw mut szFilter as *mut WCHAR as LPCWSTR;
    ofn.lpstrFile = (&raw mut szFileName) as *mut WCHAR as LPWSTR;
    ofn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT) as DWORD;

    if GetSaveFileNameW(&raw mut ofn) != 0 {
        SaveAllSettings(pInfo);
        SaveRdpSettingsToFile((&raw mut szFileName) as *mut WCHAR as LPWSTR, (*pInfo).pRdpSettings);
    }
}

unsafe extern "C" fn OnTabWndSelChange(pInfo: PINFO) {
    match TabCtrl_GetCurSel!((*pInfo).hTab) {
        0 => {
            ShowWindow((*pInfo).hGeneralPage, (SW_SHOW) as i32);
            ShowWindow((*pInfo).hDisplayPage, (SW_HIDE) as i32);
            BringWindowToTop((*pInfo).hGeneralPage);
        }
        1 => {
            ShowWindow((*pInfo).hGeneralPage, (SW_HIDE) as i32);
            ShowWindow((*pInfo).hDisplayPage, (SW_SHOW) as i32);
            BringWindowToTop((*pInfo).hDisplayPage);
        }
        _ => {}
    }
}

unsafe extern "C" fn LoadUsernameHint(mut hDlg: HWND, mut iCur: INT) {
    let mut szValue: [WCHAR; (MAXVALUE + 1000) as _] = [0; (MAXVALUE + 1000) as _];
    let mut szName: [WCHAR; (MAX_KEY_NAME as usize)] = [0; (MAX_KEY_NAME as usize)];
    let mut szKeyName: [WCHAR; 50] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0054,
    0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x006C, 0x0020, 0x0053, 0x0065,
    0x0072, 0x0076, 0x0065, 0x0072, 0x0020, 0x0043, 0x006C, 0x0069, 0x0065, 0x006E,
    0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0073, 0x0000,
];
    let mut lpAddress: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut hKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut hSubKey: HKEY = core::ptr::null_mut::<HKEY__>();
    let mut lRet: LONG = ERROR_SUCCESS as LONG;
    let mut iIndex: INT = 0 as INT;
    let mut dwSize: DWORD = MAX_KEY_NAME as DWORD;

    SendDlgItemMessageW(
        hDlg,
        (IDC_SERVERCOMBO as i32),
        CB_GETLBTEXT as UINT,
        iCur as WPARAM,
        &raw mut szValue as *mut WCHAR as LPARAM,
    );

    /* remove possible port number */
    lpAddress = wcstok(
        &raw mut szValue as *mut WCHAR,
        RUSTCONV_WSTR_4D70F4A881C7.as_ptr(),
    ) as PWCHAR;

    if lpAddress.is_null() {
        return;
    }

    if RegOpenKeyExW(
        HKEY_CURRENT_USER as HKEY,
        &raw mut szKeyName as *mut WCHAR as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        &raw mut hKey,
    ) == ERROR_SUCCESS as LSTATUS
    {
        while lRet == ERROR_SUCCESS as LONG {
            dwSize = MAX_KEY_NAME as DWORD;

            lRet = RegEnumKeyExW(
                hKey,
                iIndex as DWORD,
                &raw mut szName as LPWSTR,
                &raw mut dwSize,
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<WCHAR>(),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<_FILETIME>(),
            ) as LONG;

            if lRet == ERROR_SUCCESS as LONG
                && wcscmp(
                    &raw mut szName as *mut WCHAR,
                    lpAddress as *const WCHAR,
                ) == 0 as i32
            {
                if RegOpenKeyExW(
                    hKey,
                    &raw mut szName as *mut WCHAR as LPCWSTR,
                    0 as DWORD,
                    KEY_READ as REGSAM,
                    &raw mut hSubKey,
                ) != ERROR_SUCCESS as LSTATUS
                {
                    break;
                }

                dwSize = (MAXVALUE as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD;

                if RegQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_A90688F55D61.as_ptr(),
                    0 as LPDWORD,
                    core::ptr::null_mut::<DWORD>(),
                    &raw mut szValue as *mut WCHAR as LPBYTE,
                    &raw mut dwSize,
                ) == ERROR_SUCCESS as LSTATUS
                {
                    SetDlgItemTextW(
                        hDlg,
                        (IDC_NAMEEDIT as i32),
                        &raw mut szValue as *mut WCHAR as LPCWSTR,
                    );
                }

                RegCloseKey(hSubKey);
                break;
            }
            iIndex += 1;
        }
        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn FillServerAddressCombo(mut pInfo: PINFO) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut KeyName: [WCHAR; 50] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0054,
    0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x006C, 0x0020, 0x0053, 0x0065,
    0x0072, 0x0076, 0x0065, 0x0072, 0x0020, 0x0043, 0x006C, 0x0069, 0x0065, 0x006E,
    0x0074, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000,
];
    let mut Name: [WCHAR; (MAX_KEY_NAME as usize)] = [0; (MAX_KEY_NAME as usize)];
    let mut ret: LONG = ERROR_SUCCESS as LONG;
    let mut size: DWORD = 0;
    let mut i: INT = 0 as INT;
    let mut found: BOOL = FALSE as BOOL;

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) == ERROR_SUCCESS as LSTATUS
    {
        while ret == ERROR_SUCCESS as LONG {
            size = MAX_KEY_NAME as DWORD;
            ret = RegEnumValueW(
                hKey,
                i as DWORD,
                core::ptr::addr_of_mut!(Name) as *mut WCHAR as LPWSTR,
                core::ptr::addr_of_mut!(size),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<DWORD>(),
                core::ptr::null_mut::<BYTE>(),
                core::ptr::null_mut::<DWORD>(),
            ) as LONG;
            if ret == ERROR_SUCCESS as LONG {
                size = core::mem::size_of::<[WCHAR; (MAX_KEY_NAME as usize)]>() as DWORD;
                if RegQueryValueExW(
                    hKey,
                    core::ptr::addr_of_mut!(Name) as *mut WCHAR as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::null_mut::<BYTE>(),
                    core::ptr::addr_of_mut!(size),
                ) == ERROR_SUCCESS as LSTATUS
                {
                    let mut lpAddress: LPWSTR = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        size as SIZE_T,
                    ) as LPWSTR;
                    if !lpAddress.is_null() {
                        if RegQueryValueExW(
                            hKey,
                            core::ptr::addr_of_mut!(Name) as *mut WCHAR as LPCWSTR,
                            core::ptr::null_mut(),
                            core::ptr::null_mut::<DWORD>(),
                            lpAddress as LPBYTE,
                            core::ptr::addr_of_mut!(size),
                        ) == ERROR_SUCCESS as LSTATUS
                        {
                            SendDlgItemMessageW(
                                (*pInfo).hGeneralPage,
                                (IDC_SERVERCOMBO as i32),
                                CB_ADDSTRING as UINT,
                                0 as WPARAM,
                                lpAddress as LPARAM,
                            );
                            found = TRUE as BOOL;
                        }

                        HeapFree(
                            GetProcessHeap(),
                            0 as DWORD,
                            lpAddress as PVOID,
                        );
                    }
                }
            }

            i += 1;
        }
        RegCloseKey(hKey);
    }

    if LoadStringW(
        hInst,
        IDS_BROWSESERVER as UINT,
        core::ptr::addr_of_mut!(Name) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; (MAX_KEY_NAME as usize)]>()
            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    ) != 0
    {
        SendDlgItemMessageW(
            (*pInfo).hGeneralPage,
            (IDC_SERVERCOMBO as i32),
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(Name) as *mut WCHAR as LPARAM,
        );
    }

    if found != 0 {
        SendDlgItemMessageW(
            (*pInfo).hGeneralPage,
            (IDC_SERVERCOMBO as i32),
            CB_SETCURSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        LoadUsernameHint((*pInfo).hGeneralPage, 0);
    }
}

unsafe extern "C" fn ReLoadGeneralPage(mut pInfo: PINFO) {
    let mut lpText: LPWSTR = core::ptr::null_mut();

    /* add file address */
    lpText = GetStringFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_8F802E85B5A5.as_ptr() as LPWSTR);
    if !lpText.is_null() {
        SetDlgItemTextW((*pInfo).hGeneralPage, (IDC_SERVERCOMBO as i32), lpText as LPCWSTR);
    }

    /* set user name */
    lpText = GetStringFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_993EF02FC0D2.as_ptr() as LPWSTR);
    if !lpText.is_null() {
        SetDlgItemTextW((*pInfo).hGeneralPage, (IDC_NAMEEDIT as i32), lpText as LPCWSTR);
    }
}

unsafe extern "C" fn GeneralOnInit(hwnd: HWND, pInfo: PINFO) {
    SetWindowLongPtrW(hwnd,
                      GWLP_USERDATA,
                      pInfo as LONG_PTR);

    (*pInfo).hGeneralPage = hwnd;

    SetWindowPos((*pInfo).hGeneralPage,
                 core::ptr::null_mut(),
                 2,
                 22,
                 0,
                 0,
                 SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);

    (*pInfo).hLogon = LoadImageW(hInst,
                               MAKEINTRESOURCEW!(IDI_LOGON) as LPCWSTR,
                               IMAGE_ICON,
                               32,
                               32,
                               LR_DEFAULTCOLOR) as HICON;
    if !(*pInfo).hLogon.is_null() {
        SendDlgItemMessageW((*pInfo).hGeneralPage,
                            (IDC_LOGONICON as i32),
                            STM_SETICON,
                            (*pInfo).hLogon as WPARAM,
                            0);
    }

    (*pInfo).hConn = LoadImageW(hInst,
                              MAKEINTRESOURCEW!(IDI_CONN) as LPCWSTR,
                              IMAGE_ICON,
                              32,
                              32,
                              LR_DEFAULTCOLOR) as HICON;
    if !(*pInfo).hConn.is_null() {
        SendDlgItemMessageW((*pInfo).hGeneralPage,
                            (IDC_CONNICON as i32),
                            STM_SETICON,
                            (*pInfo).hConn as WPARAM,
                            0);
    }

    FillServerAddressCombo(pInfo);
    ReLoadGeneralPage(pInfo);
}

#[no_mangle]
pub unsafe extern "system" fn GeneralDlgProc(
    hDlg: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pInfo: PINFO = GetWindowLongPtrW(hDlg, GWLP_USERDATA) as PINFO;

    match message {
        WM_INITDIALOG => {
            GeneralOnInit(hDlg, lParam as PINFO);
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_SERVERCOMBO => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                        let mut last: INT = 0;
                        let mut cur: INT = 0;

                        cur = SendDlgItemMessageW(
                            hDlg,
                            (IDC_SERVERCOMBO as i32),
                            CB_GETCURSEL,
                            0,
                            0,
                        ) as INT;

                        last = SendDlgItemMessageW(
                            hDlg,
                            (IDC_SERVERCOMBO as i32),
                            CB_GETCOUNT,
                            0,
                            0,
                        ) as INT;

                        if (cur + 1) == last {
                            MessageBoxW(
                                hDlg,
                                RUSTCONV_WSTR_8D0582078CBE.as_ptr(),
                                RUSTCONV_WSTR_7E99F0C2EB7C.as_ptr(),
                                MB_ICONERROR,
                            );
                        } else {
                            LoadUsernameHint(hDlg, cur);
                        }
                    }
                }

                IDC_SAVE => {
                    SaveAllSettings(pInfo);
                    SaveRdpSettingsToFile(core::ptr::null_mut(), (*pInfo).pRdpSettings);
                }

                IDC_SAVEAS => {
                    DoSaveAs(pInfo);
                }

                IDC_OPEN => {
                    DoOpenFile(pInfo);
                }

                _ => {}
            }
        }

        WM_CLOSE => {
            if !(*pInfo).hLogon.is_null() {
                DestroyIcon((*pInfo).hLogon);
            }

            if !(*pInfo).hConn.is_null() {
                DestroyIcon((*pInfo).hConn);
            }
        }

        _ => {}
    }

    return 0 as INT_PTR;
}

unsafe extern "C" fn GetPossibleSettings(
    mut lpDeviceName: LPCWSTR,
    mut pSettingsCount: *mut DWORD,
    mut CurrentSettings: *mut PSETTINGS_ENTRY,
) -> PSETTINGS_ENTRY {
    let mut devmode: DEVMODEW = core::mem::zeroed();
    let mut NbSettings: DWORD = 0 as DWORD;
    let mut iMode: DWORD = 0 as DWORD;
    let mut dwFlags: DWORD = 0 as DWORD;
    let mut Settings: PSETTINGS_ENTRY = core::ptr::null_mut();
    let mut hDC: HDC = core::ptr::null_mut();
    let mut Current: PSETTINGS_ENTRY = core::ptr::null_mut();
    let mut bpp: DWORD = 0;
    let mut xres: DWORD = 0;
    let mut yres: DWORD = 0;
    let mut checkbpp: DWORD = 0;

    /* Get current settings */
    *CurrentSettings = core::ptr::null_mut();
    hDC = CreateICW(
        core::ptr::null(),
        lpDeviceName,
        core::ptr::null(),
        core::ptr::null(),
    );
    bpp = GetDeviceCaps(hDC, (PLANES as i32)) as DWORD;
    bpp = bpp.wrapping_mul(GetDeviceCaps(hDC, (BITSPIXEL as i32)) as DWORD);
    xres = GetDeviceCaps(hDC, (HORZRES as i32)) as DWORD;
    yres = GetDeviceCaps(hDC, (VERTRES as i32)) as DWORD;
    DeleteDC(hDC);

    /* List all settings */
    devmode.dmSize = core::mem::size_of::<DEVMODE>() as WORD;
    devmode.dmDriverExtra = 0 as WORD;

    if EnumDisplaySettingsExW(
        lpDeviceName,
        ENUM_CURRENT_SETTINGS,
        core::ptr::addr_of_mut!(devmode),
        dwFlags,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    while EnumDisplaySettingsExW(
        lpDeviceName,
        iMode,
        core::ptr::addr_of_mut!(devmode),
        dwFlags,
    ) != 0
    {
        if devmode.dmBitsPerPel == 8 as DWORD
            || devmode.dmBitsPerPel == 16 as DWORD
            || devmode.dmBitsPerPel == 24 as DWORD
            || devmode.dmBitsPerPel == 32 as DWORD
        {
            checkbpp = 1 as DWORD;
        } else {
            checkbpp = 0 as DWORD;
        }

        if devmode.dmPelsWidth < 640 as DWORD
            || devmode.dmPelsHeight < 480 as DWORD
            || checkbpp == 0 as DWORD
        {
            iMode = iMode.wrapping_add(1);
            continue;
        }

        Current = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            core::mem::size_of::<SETTINGS_ENTRY>() as SIZE_T,
        ) as PSETTINGS_ENTRY;
        if !Current.is_null() {
            /* Sort resolutions by increasing height, and BPP */
            let mut Previous: PSETTINGS_ENTRY = core::ptr::null_mut();
            let mut Next: PSETTINGS_ENTRY = Settings;
            (*Current).dmPelsWidth = devmode.dmPelsWidth;
            (*Current).dmPelsHeight = devmode.dmPelsHeight;
            (*Current).dmBitsPerPel = devmode.dmBitsPerPel;
            while !Next.is_null()
                && ((*Next).dmPelsWidth < (*Current).dmPelsWidth
                    || ((*Next).dmPelsWidth == (*Current).dmPelsWidth
                        && (*Next).dmPelsHeight < (*Current).dmPelsHeight)
                    || ((*Next).dmPelsHeight == (*Current).dmPelsHeight
                        && (*Next).dmPelsWidth == (*Current).dmPelsWidth
                        && (*Next).dmBitsPerPel < (*Current).dmBitsPerPel))
            {
                Previous = Next;
                Next = (*Next).Flink as PSETTINGS_ENTRY;
            }
            (*Current).Blink = Previous as *mut _SETTINGS_ENTRY;
            (*Current).Flink = Next as *mut _SETTINGS_ENTRY;
            if Previous.is_null() {
                Settings = Current;
            } else {
                (*Previous).Flink = Current as *mut _SETTINGS_ENTRY;
            }
            if !Next.is_null() {
                (*Next).Blink = Current as *mut _SETTINGS_ENTRY;
            }
            if devmode.dmPelsWidth == xres
                && devmode.dmPelsHeight == yres
                && devmode.dmBitsPerPel == bpp
            {
                *CurrentSettings = Current;
            }
            NbSettings = NbSettings.wrapping_add(1);
        }
        iMode = iMode.wrapping_add(1);
    }

    *pSettingsCount = NbSettings;
    return Settings;
}

unsafe extern "C" fn AddDisplayDevice(
    mut pInfo: PINFO,
    mut DisplayDevice: PDISPLAY_DEVICEW,
) -> BOOL {
    let mut newEntry: PDISPLAY_DEVICE_ENTRY = core::ptr::null_mut();
    let mut description: LPWSTR = core::ptr::null_mut();
    let mut name: LPWSTR = core::ptr::null_mut();
    let mut key: LPWSTR = core::ptr::null_mut();
    let mut devid: LPWSTR = core::ptr::null_mut();
    let mut descriptionSize: SIZE_T = 0;
    let mut nameSize: SIZE_T = 0;
    let mut keySize: SIZE_T = 0;
    let mut devidSize: SIZE_T = 0;
    let mut Current: PSETTINGS_ENTRY = core::ptr::null_mut();
    let mut ResolutionsCount: DWORD = 1 as DWORD;
    let mut i: DWORD = 0;

    'ByeBye: {
        newEntry = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            core::mem::size_of::<DISPLAY_DEVICE_ENTRY>() as SIZE_T,
        ) as PDISPLAY_DEVICE_ENTRY;
        if newEntry.is_null() {
            break 'ByeBye;
        }
        ZeroMemory!(
            newEntry as *mut core::ffi::c_void,
            core::mem::size_of::<DISPLAY_DEVICE_ENTRY>() as SIZE_T,
        );

        (*newEntry).Settings = GetPossibleSettings(
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceName) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!((*newEntry).SettingsCount),
            core::ptr::addr_of_mut!((*newEntry).CurrentSettings),
        );
        if (*newEntry).Settings.is_null() {
            break 'ByeBye;
        }

        (*newEntry).InitialSettings.dmPelsWidth = (*(*newEntry).CurrentSettings).dmPelsWidth;
        (*newEntry).InitialSettings.dmPelsHeight = (*(*newEntry).CurrentSettings).dmPelsHeight;
        (*newEntry).InitialSettings.dmBitsPerPel = (*(*newEntry).CurrentSettings).dmBitsPerPel;

        /* Count different resolutions */
        Current = (*newEntry).Settings;
        while !Current.is_null() {
            if !(*Current).Flink.is_null()
                && ((*Current).dmPelsWidth != (*(*Current).Flink).dmPelsWidth
                    && (*Current).dmPelsHeight != (*(*Current).Flink).dmPelsHeight)
            {
                ResolutionsCount = ResolutionsCount.wrapping_add(1);
            }
            Current = (*Current).Flink as PSETTINGS_ENTRY;
        }

        (*newEntry).Resolutions = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            // The C allocates ResolutionsCount * (sizeof + 1) bytes, but the fill loop writes
            // ResolutionsCount entries and the fullscreen one goes at [ResolutionsCount] (the
            // geometry slider runs 0..=ResolutionsCount): one RESOLUTION_INFO past the buffer,
            // which the stray byte per entry only partly covered. Room for all of them.
            (ResolutionsCount as SIZE_T + 1)
                .wrapping_mul(core::mem::size_of::<RESOLUTION_INFO>() as SIZE_T),
        ) as PRESOLUTION_INFO;
        if (*newEntry).Resolutions.is_null() {
            break 'ByeBye;
        }

        (*newEntry).ResolutionsCount = ResolutionsCount;

        /* Fill resolutions infos */
        Current = (*newEntry).Settings;
        i = 0 as DWORD;
        while !Current.is_null() {
            if (*Current).Flink.is_null()
                || (!(*Current).Flink.is_null()
                    && ((*Current).dmPelsWidth != (*(*Current).Flink).dmPelsWidth
                        && (*Current).dmPelsHeight != (*(*Current).Flink).dmPelsHeight))
            {
                (*(*newEntry).Resolutions.offset(i as isize)).dmPelsWidth = (*Current).dmPelsWidth;
                (*(*newEntry).Resolutions.offset(i as isize)).dmPelsHeight = (*Current).dmPelsHeight;
                i = i.wrapping_add(1);
            }
            Current = (*Current).Flink as PSETTINGS_ENTRY;
        }

        /* fullscreen */
        (*(*newEntry).Resolutions.offset(i as isize)).dmPelsWidth =
            GetSystemMetrics(SM_CXSCREEN as core::ffi::c_int) as DWORD;
        (*(*newEntry).Resolutions.offset(i as isize)).dmPelsHeight =
            GetSystemMetrics(SM_CYSCREEN as core::ffi::c_int) as DWORD;

        descriptionSize = (wcslen(
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceString) as *mut WCHAR,
        ) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            as SIZE_T;
        description = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            descriptionSize,
        ) as LPWSTR;
        if description.is_null() {
            break 'ByeBye;
        }

        nameSize = (wcslen(
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceName) as *mut WCHAR,
        ) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            as SIZE_T;
        name = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            nameSize,
        ) as LPWSTR;
        if name.is_null() {
            break 'ByeBye;
        }

        keySize = (wcslen(
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceKey) as *mut WCHAR,
        ) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            as SIZE_T;
        key = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            keySize,
        ) as LPWSTR;
        if key.is_null() {
            break 'ByeBye;
        }

        devidSize = (wcslen(
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceID) as *mut WCHAR,
        ) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())
            as SIZE_T;
        devid = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            devidSize,
        ) as LPWSTR;
        if devid.is_null() {
            break 'ByeBye;
        }

        memcpy(
            description as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceString) as *mut WCHAR
                as *const core::ffi::c_void,
            descriptionSize as usize,
        );
        memcpy(
            name as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceName) as *mut WCHAR
                as *const core::ffi::c_void,
            nameSize as usize,
        );
        memcpy(
            key as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceKey) as *mut WCHAR
                as *const core::ffi::c_void,
            keySize as usize,
        );
        memcpy(
            devid as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*DisplayDevice).DeviceID) as *mut WCHAR
                as *const core::ffi::c_void,
            devidSize as usize,
        );
        (*newEntry).DeviceDescription = description;
        (*newEntry).DeviceName = name;
        (*newEntry).DeviceKey = key;
        (*newEntry).DeviceID = devid;
        (*newEntry).DeviceStateFlags = (*DisplayDevice).StateFlags;
        (*newEntry).Flink = (*pInfo).DisplayDeviceList as *mut _DISPLAY_DEVICE_ENTRY;
        (*pInfo).DisplayDeviceList = newEntry;
        return TRUE as BOOL;
    }

    // ByeBye:
    if !newEntry.is_null() {
        if !(*newEntry).Settings.is_null() {
            Current = (*newEntry).Settings;
            while !Current.is_null() {
                let Next: PSETTINGS_ENTRY = (*Current).Flink as PSETTINGS_ENTRY;
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    Current as PVOID,
                );
                Current = Next;
            }
        }
        if !(*newEntry).Resolutions.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*newEntry).Resolutions as PVOID,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            newEntry as PVOID,
        );
    }
    if !description.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            description as PVOID,
        );
    }
    if !name.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            name as PVOID,
        );
    }
    if !key.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            key as PVOID,
        );
    }
    if !devid.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            devid as PVOID,
        );
    }
    return FALSE as BOOL;
}

unsafe extern "C" fn OnResolutionChanged(mut pInfo: PINFO, mut position: INT) {
    let mut Buffer: [WCHAR; 64] = [0; 64];
    let mut MaxSlider: INT = 0;
    MaxSlider = SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        (IDC_GEOSLIDER as i32),
        TBM_GETRANGEMAX as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if position == MaxSlider {
        LoadStringW(
            hInst,
            IDS_FULLSCREEN as UINT,
            core::ptr::addr_of_mut!(Buffer) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 64]>() as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                as core::ffi::c_int,
        );
    } else {
        let mut Pixel: [WCHAR; 64] = [0; 64];
        if LoadStringW(
            hInst,
            IDS_PIXEL as UINT,
            core::ptr::addr_of_mut!(Pixel) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 64]>() as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                as core::ffi::c_int,
        ) != 0
        {
            swprintf(
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
                core::ptr::addr_of_mut!(Pixel) as *mut WCHAR,
                (*(*(*pInfo).DisplayDeviceList)
                    .Resolutions
                    .offset(position as isize))
                .dmPelsWidth,
                (*(*(*pInfo).DisplayDeviceList)
                    .Resolutions
                    .offset(position as isize))
                .dmPelsHeight,
                core::ptr::addr_of_mut!(Pixel) as *mut WCHAR,
            );
        }
    }
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        (IDC_SETTINGS_RESOLUTION_TEXT as i32),
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn FillResolutionsAndColors(mut pInfo: PINFO) {
    let mut Current: PSETTINGS_ENTRY = ::core::ptr::null_mut::<_SETTINGS_ENTRY>();
    let mut index: DWORD = 0;
    let mut i: DWORD = 0;
    let mut num: DWORD = 0;
    let mut MaxBpp: DWORD = 0 as DWORD;
    let mut types: [UINT; 5] = [0; 5];
    (*pInfo).CurrentDisplayDevice = (*pInfo).DisplayDeviceList;
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        IDC_BPPCOMBO as ::core::ffi::c_int,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    Current = (*(*pInfo).DisplayDeviceList).Settings;
    while !Current.is_null() {
        if (*Current).dmBitsPerPel > MaxBpp {
            MaxBpp = (*Current).dmBitsPerPel;
        }
        Current = (*Current).Flink as PSETTINGS_ENTRY;
    }
    match MaxBpp {
        32 => {
            num = 4 as DWORD;
        }
        24 => {
            num = 3 as DWORD;
        }
        16 => {
            num = 2 as DWORD;
        }
        15 => {
            num = 1 as DWORD;
        }
        8 => {
            num = 0 as DWORD;
        }
        _ => {
            num = 0 as DWORD;
        }
    }
    types[0 as usize] = IDS_256COLORS as UINT;
    types[1 as usize] = IDS_HIGHCOLOR15 as UINT;
    types[2 as usize] = IDS_HIGHCOLOR16 as UINT;
    types[3 as usize] = IDS_HIGHCOLOR24 as UINT;
    types[4 as usize] = IDS_HIGHCOLOR32 as UINT;
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        IDC_BPPCOMBO as ::core::ffi::c_int,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    i = 0 as DWORD;
    Current = (*(*pInfo).DisplayDeviceList).Settings;
    while i <= num && !Current.is_null() {
        let mut Buffer: [WCHAR; 64] = [0; 64];
        if LoadStringW(
            hInst,
            types[i as usize],
            core::ptr::addr_of_mut!(Buffer) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 64]>() as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                as ::core::ffi::c_int,
        ) != 0
        {
            index = SendDlgItemMessageW(
                (*pInfo).hDisplayPage,
                IDC_BPPCOMBO as ::core::ffi::c_int,
                CB_FINDSTRINGEXACT as UINT,
                (-(1 as ::core::ffi::c_int)) as WPARAM,
                core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPARAM,
            ) as DWORD;
            if index == CB_ERR as DWORD {
                index = SendDlgItemMessageW(
                    (*pInfo).hDisplayPage,
                    IDC_BPPCOMBO as ::core::ffi::c_int,
                    CB_ADDSTRING as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPARAM,
                ) as DWORD;
                SendDlgItemMessageW(
                    (*pInfo).hDisplayPage,
                    IDC_BPPCOMBO as ::core::ffi::c_int,
                    CB_SETITEMDATA as UINT,
                    index as WPARAM,
                    types[i as usize] as LPARAM,
                );
            }
        }
        i = i.wrapping_add(1);
        Current = (*Current).Flink as PSETTINGS_ENTRY;
    }
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        IDC_GEOSLIDER as ::core::ffi::c_int,
        TBM_CLEARTICS as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        IDC_GEOSLIDER as ::core::ffi::c_int,
        TBM_SETRANGE as UINT,
        TRUE as WPARAM,
        MAKELONG!(
            0 as WORD,
            (*(*pInfo).DisplayDeviceList).ResolutionsCount as WORD,
        ) as LPARAM,
    );
}

unsafe extern "C" fn ReLoadDisplayPage(pInfo: PINFO) {
    let mut index: DWORD = 0;
    let mut width: INT = 0;
    let mut height: INT = 0;
    let mut pos: INT = 0;
    let mut bpp: INT = 0;
    let mut num: INT = 0;
    let mut i: INT = 0;
    let mut screenmode: INT = 0;
    let mut bSet: BOOL = (FALSE) as i32;

    /* get fullscreen info */
    screenmode = GetIntegerFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_76E86BF0A91D.as_ptr().cast_mut()) as INT;

    /* set trackbar position */
    width = GetIntegerFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_97F575EF508B.as_ptr().cast_mut()) as INT;
    height = GetIntegerFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_D8EF2D6777E6.as_ptr().cast_mut()) as INT;

    if width != -1 && height != -1 {
        if screenmode == 2 {
            pos = SendDlgItemMessageW(
                (*pInfo).hDisplayPage,
                (IDC_GEOSLIDER as i32),
                TBM_GETRANGEMAX,
                0,
                0,
            ) as INT;
        } else {
            index = 0 as DWORD;
            while index < (*(*pInfo).CurrentDisplayDevice).ResolutionsCount {
                if (*(*(*pInfo).CurrentDisplayDevice)
                    .Resolutions
                    .offset(index as isize))
                .dmPelsWidth
                    == width as DWORD
                    && (*(*(*pInfo).CurrentDisplayDevice)
                        .Resolutions
                        .offset(index as isize))
                    .dmPelsHeight
                        == height as DWORD
                {
                    pos = index as INT;
                    break;
                } else {
                    index = index.wrapping_add(1);
                }
            }
        }
    }

    /* set slider position */
    SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        (IDC_GEOSLIDER as i32),
        TBM_SETPOS,
        (TRUE) as WPARAM,
        pos as LPARAM,
    );

    OnResolutionChanged(pInfo, pos);

    /* set color combo */
    bpp = GetIntegerFromSettings((*pInfo).pRdpSettings, RUSTCONV_WSTR_CDA991128999.as_ptr().cast_mut()) as INT;

    num = SendDlgItemMessageW(
        (*pInfo).hDisplayPage,
        (IDC_BPPCOMBO as i32),
        CB_GETCOUNT,
        0,
        0,
    ) as INT;
    i = 0 as INT;
    while i < num {
        let mut data: INT = SendDlgItemMessageW(
            (*pInfo).hDisplayPage,
            (IDC_BPPCOMBO as i32),
            CB_GETITEMDATA,
            i as WPARAM,
            0,
        ) as INT;
        if data == bpp {
            SendDlgItemMessageW(
                (*pInfo).hDisplayPage,
                (IDC_BPPCOMBO as i32),
                CB_SETCURSEL,
                i as WPARAM,
                0,
            );
            bSet = (TRUE) as i32;
            break;
        } else {
            i += 1;
        }
    }

    if bSet == 0 {
        SendDlgItemMessageW(
            (*pInfo).hDisplayPage,
            (IDC_BPPCOMBO as i32),
            CB_SETCURSEL,
            (num - 1) as WPARAM,
            0,
        );
    }
}

unsafe extern "C" fn DisplayOnInit(hwnd: HWND, pInfo: PINFO) {
    let mut displayDevice: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut iDevNum: DWORD = 0;
    let mut GotDev: BOOL = (FALSE) as i32;

    SetWindowLongPtrW(hwnd,
                      GWLP_USERDATA,
                      pInfo as LONG_PTR);

    (*pInfo).hDisplayPage = hwnd;

    SetWindowPos((*pInfo).hDisplayPage,
                 core::ptr::null_mut(),
                 2,
                 22,
                 0,
                 0,
                 (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT);

    (*pInfo).hRemote = LoadImageW(hInst,
                                MAKEINTRESOURCEW!(IDI_REMOTE),
                                IMAGE_ICON,
                                32,
                                32,
                                LR_DEFAULTCOLOR) as HICON;
    if !(*pInfo).hRemote.is_null() {
        SendDlgItemMessageW((*pInfo).hDisplayPage,
                            (IDC_REMICON as i32),
                            STM_SETICON,
                            (*pInfo).hRemote as WPARAM,
                            0);
    }

    (*pInfo).hColor = LoadImageW(hInst,
                               MAKEINTRESOURCEW!(IDI_COLORS),
                               IMAGE_ICON,
                               32,
                               32,
                               LR_DEFAULTCOLOR) as HICON;
    if !(*pInfo).hColor.is_null() {
        SendDlgItemMessageW((*pInfo).hDisplayPage,
                            (IDC_COLORSICON as i32),
                            STM_SETICON,
                            (*pInfo).hColor as WPARAM,
                            0);
    }

    (*pInfo).hSpectrum = LoadImageW(hInst,
                                  MAKEINTRESOURCEW!(IDB_SPECT),
                                  IMAGE_BITMAP,
                                  0,
                                  0,
                                  LR_DEFAULTCOLOR) as HBITMAP;
    if !(*pInfo).hSpectrum.is_null() {
        GetObjectW((*pInfo).hSpectrum as HANDLE,
                   core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                   core::ptr::addr_of_mut!((*pInfo).bitmap) as LPVOID);
    }

    /* Get video cards list */
    displayDevice.cb = core::mem::size_of::<DISPLAY_DEVICE>() as DWORD;
    while EnumDisplayDevicesW(core::ptr::null(),
                                 iDevNum,
                                 core::ptr::addr_of_mut!(displayDevice),
                                 0x1) != 0
    {
        if (displayDevice.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0 {
            if AddDisplayDevice(pInfo, core::ptr::addr_of_mut!(displayDevice)) != 0 {
                GotDev = (TRUE) as i32;
            }
        }
        iDevNum = iDevNum.wrapping_add(1);
    }

    if GotDev != 0 {
        FillResolutionsAndColors(pInfo);
        ReLoadDisplayPage(pInfo);
    }
}
#[no_mangle]
pub unsafe extern "system" fn DisplayDlgProc(
    mut hDlg: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut pInfo: PINFO = GetWindowLongPtrW(hDlg, GWLP_USERDATA) as PINFO;

    match message {
        WM_INITDIALOG => {
            DisplayOnInit(hDlg, lParam as PINFO);
            return TRUE as INT_PTR;
        }
        WM_DRAWITEM => {
            let mut lpDrawItem: LPDRAWITEMSTRUCT = ::core::ptr::null_mut::<tagDRAWITEMSTRUCT>();
            lpDrawItem = lParam as LPDRAWITEMSTRUCT;
            if (*lpDrawItem).CtlID == IDC_COLORIMAGE as UINT {
                let mut hdcMem: HDC = ::core::ptr::null_mut::<HDC__>();
                let mut hSpecOld: HBITMAP = ::core::ptr::null_mut::<HBITMAP__>();
                hdcMem = CreateCompatibleDC((*lpDrawItem).hDC);
                if !hdcMem.is_null() {
                    hSpecOld = SelectObject(hdcMem, (*pInfo).hSpectrum as HGDIOBJ) as HBITMAP;
                    StretchBlt(
                        (*lpDrawItem).hDC,
                        (*lpDrawItem).rcItem.left as ::core::ffi::c_int,
                        (*lpDrawItem).rcItem.top as ::core::ffi::c_int,
                        ((*lpDrawItem).rcItem.right - (*lpDrawItem).rcItem.left) as ::core::ffi::c_int,
                        ((*lpDrawItem).rcItem.bottom - (*lpDrawItem).rcItem.top) as ::core::ffi::c_int,
                        hdcMem,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        (*pInfo).bitmap.bmWidth as ::core::ffi::c_int,
                        (*pInfo).bitmap.bmHeight as ::core::ffi::c_int,
                        SRCCOPY as DWORD,
                    );
                    SelectObject(hdcMem, hSpecOld as HGDIOBJ);
                    DeleteDC(hdcMem);
                }
            }
        }
        WM_HSCROLL => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                TB_LINEUP
                | TB_LINEDOWN
                | TB_PAGEUP
                | TB_PAGEDOWN
                | TB_TOP
                | TB_BOTTOM
                | TB_ENDTRACK => {
                    let mut newPosition: INT = SendDlgItemMessageW(
                        hDlg,
                        (IDC_GEOSLIDER as UINT) as i32,
                        TBM_GETPOS as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as DWORD as INT;
                    OnResolutionChanged(pInfo, newPosition);
                }
                TB_THUMBTRACK => {
                    OnResolutionChanged(pInfo, HIWORD!(wParam) as INT);
                }
                _ => {}
            }
        }
        WM_CLOSE => {
            if !(*pInfo).hRemote.is_null() {
                DestroyIcon((*pInfo).hRemote);
            }
            if !(*pInfo).hColor.is_null() {
                DestroyIcon((*pInfo).hColor);
            }
            if !(*pInfo).hSpectrum.is_null() {
                DeleteObject((*pInfo).hSpectrum as HGDIOBJ);
            }
        }
        _ => {}
    }
    return 0 as INT_PTR;
}

unsafe extern "C" fn OnMainCreate(hwnd: HWND, pRdpSettings: PRDPSETTINGS) -> BOOL {
    let mut pInfo: PINFO = core::ptr::null_mut();
    let mut item: TCITEMW = core::mem::zeroed();
    let mut bRet: BOOL = FALSE as BOOL;

    pInfo = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<INFO>() as SIZE_T,
    ) as PINFO;
    if !pInfo.is_null() {
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA as i32,
            pInfo as LONG_PTR,
        );

        (*pInfo).hSelf = hwnd;

        /* add main settings pointer */
        (*pInfo).pRdpSettings = pRdpSettings;

        /* set the dialog icons */
        (*pInfo).hMstscSm = LoadImageW(
            hInst,
            MAKEINTRESOURCEW!(IDI_MSTSC),
            IMAGE_ICON as UINT,
            16 as i32,
            16 as i32,
            LR_DEFAULTCOLOR as UINT,
        ) as HICON;
        if !(*pInfo).hMstscSm.is_null() {
            SendMessageW(
                hwnd,
                WM_SETICON as UINT,
                ICON_SMALL as WPARAM,
                (*pInfo).hMstscSm as WPARAM as LPARAM,
            );
        }
        (*pInfo).hMstscLg = LoadImageW(
            hInst,
            MAKEINTRESOURCEW!(IDI_MSTSC),
            IMAGE_ICON as UINT,
            32 as i32,
            32 as i32,
            LR_DEFAULTCOLOR as UINT,
        ) as HICON;
        if !(*pInfo).hMstscLg.is_null() {
            SendMessageW(
                hwnd,
                WM_SETICON as UINT,
                ICON_BIG as WPARAM,
                (*pInfo).hMstscLg as WPARAM as LPARAM,
            );
        }

        (*pInfo).hHeader = LoadImageW(
            hInst,
            MAKEINTRESOURCEW!(IDB_HEADER),
            IMAGE_BITMAP as UINT,
            0 as i32,
            0 as i32,
            LR_DEFAULTCOLOR as UINT,
        ) as HBITMAP;
        if !(*pInfo).hHeader.is_null() {
            GetObjectW(
                (*pInfo).hHeader as HANDLE,
                core::mem::size_of::<BITMAP>() as i32,
                core::ptr::addr_of_mut!((*pInfo).headerbitmap) as LPVOID,
            );
        }

        /* setup the tabs */
        (*pInfo).hTab = GetDlgItem(hwnd, (IDC_TAB as i32));
        if !(*pInfo).hTab.is_null() {
            if CreateDialogParamW(
                hInst,
                MAKEINTRESOURCEW!(IDD_GENERAL),
                (*pInfo).hTab,
                Some(GeneralDlgProc),
                pInfo as LPARAM,
            )
            .is_null()
                == false
            {
                let mut str: [WCHAR; 256] = [0; 256];
                ZeroMemory!(
                    core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                    core::mem::size_of::<TCITEM>(),
                );
                item.mask = TCIF_TEXT as UINT;
                if LoadStringW(
                    hInst,
                    IDS_TAB_GENERAL as UINT,
                    str.as_mut_ptr() as LPWSTR,
                    256 as i32,
                ) != 0
                {
                    item.pszText = str.as_mut_ptr();
                }
                item.cchTextMax = 256 as i32;
                let _ = TabCtrl_InsertItem!(
                    (*pInfo).hTab,
                    0 as i32,
                    core::ptr::addr_of_mut!(item),
                );
            }

            if CreateDialogParamW(
                hInst,
                MAKEINTRESOURCEW!(IDD_DISPLAY),
                (*pInfo).hTab,
                Some(DisplayDlgProc),
                pInfo as LPARAM,
            )
            .is_null()
                == false
            {
                let mut str: [WCHAR; 256] = [0; 256];
                ZeroMemory!(
                    core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                    core::mem::size_of::<TCITEM>(),
                );
                item.mask = TCIF_TEXT as UINT;
                if LoadStringW(
                    hInst,
                    IDS_TAB_DISPLAY as UINT,
                    str.as_mut_ptr() as LPWSTR,
                    256 as i32,
                ) != 0
                {
                    item.pszText = str.as_mut_ptr();
                }
                item.cchTextMax = 256 as i32;
                let _ = TabCtrl_InsertItem!(
                    (*pInfo).hTab,
                    1 as i32,
                    core::ptr::addr_of_mut!(item),
                );
            }

            OnTabWndSelChange(pInfo);
        }
    }

    return bRet;
}

unsafe extern "C" fn Cleanup(pInfo: PINFO) {
    if !pInfo.is_null() {
        if !(*pInfo).hMstscSm.is_null() {
            DestroyIcon((*pInfo).hMstscSm);
        }
        if !(*pInfo).hMstscLg.is_null() {
            DestroyIcon((*pInfo).hMstscLg);
        }
        if !(*pInfo).hHeader.is_null() {
            DeleteObject((*pInfo).hHeader as HGDIOBJ);
        }
        if !(*pInfo).hSpectrum.is_null() {
            DeleteObject((*pInfo).hSpectrum as HGDIOBJ);
        }
        if !(*pInfo).hRemote.is_null() {
            DestroyIcon((*pInfo).hRemote);
        }
        if !(*pInfo).hLogon.is_null() {
            DestroyIcon((*pInfo).hLogon);
        }
        if !(*pInfo).hConn.is_null() {
            DestroyIcon((*pInfo).hConn);
        }
        if !(*pInfo).hColor.is_null() {
            DestroyIcon((*pInfo).hColor);
        }
        HeapFree(GetProcessHeap(), 0 as DWORD, pInfo as PVOID);
    }
}

unsafe extern "system" fn DlgProc(
    hDlg: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pInfo: PINFO = core::ptr::null_mut();

    /* Get the window context */
    pInfo = GetWindowLongPtrW(hDlg, GWLP_USERDATA) as PINFO;
    if pInfo.is_null() && Message != WM_INITDIALOG {
        return (FALSE) as INT_PTR;
    }

    match Message {
        WM_INITDIALOG => {
            OnMainCreate(hDlg, lParam as PRDPSETTINGS);
        }

        WM_COMMAND => {
            if (LOWORD!(wParam)) as u32 == IDOK || (LOWORD!(wParam)) as u32 == IDCANCEL {
                if (LOWORD!(wParam)) as u32 == IDOK {
                    SaveAllSettings(pInfo);
                    SaveRdpSettingsToFile(core::ptr::null_mut(), (*pInfo).pRdpSettings);
                }
                Cleanup(pInfo);
                EndDialog(hDlg, (LOWORD!(wParam)) as INT_PTR);
            }
        }

        WM_NOTIFY => {
            let mut pnmh: LPNMHDR = core::ptr::null_mut();
            pnmh = lParam as LPNMHDR;
            if (*pnmh).idFrom == (IDC_TAB as UINT_PTR)
                && (*pnmh).code == (TCN_SELCHANGE) as u32
            {
                OnTabWndSelChange(pInfo);
            }
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hdc: HDC = core::ptr::null_mut();

            hdc = BeginPaint(hDlg, core::ptr::addr_of_mut!(ps));
            if !hdc.is_null() {
                let mut hdcMem: HDC = CreateCompatibleDC(hdc);
                if !hdcMem.is_null() {
                    let mut szBuffer: [WCHAR; 32] = core::mem::zeroed();
                    let mut bmpRc: RECT = core::mem::zeroed();
                    let mut txtRc: RECT = core::mem::zeroed();
                    let mut lf: LOGFONTW = core::mem::zeroed();
                    let mut hFont: HFONT = core::ptr::null_mut();
                    let mut hFontOld: HFONT = core::ptr::null_mut();
                    let mut hBmpOld: HBITMAP = core::ptr::null_mut();

                    GetClientRect((*pInfo).hSelf, core::ptr::addr_of_mut!(bmpRc));

                    hBmpOld = SelectObject(hdcMem, (*pInfo).hHeader as HGDIOBJ) as HBITMAP;
                    StretchBlt(
                        hdc,
                        0,
                        0,
                        bmpRc.right,
                        (*pInfo).headerbitmap.bmHeight,
                        hdcMem,
                        0,
                        0,
                        (*pInfo).headerbitmap.bmWidth,
                        (*pInfo).headerbitmap.bmHeight,
                        SRCCOPY,
                    );

                    SelectObject(hdcMem, hBmpOld as HGDIOBJ);
                    txtRc.left = bmpRc.right / 4;
                    txtRc.top = 10;
                    txtRc.right = bmpRc.right * 3 / 4;
                    txtRc.bottom = (*pInfo).headerbitmap.bmHeight / 2;

                    ZeroMemory!(
                        core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
                        core::mem::size_of::<LOGFONTW>(),
                    );

                    if LoadStringW(
                        hInst,
                        IDS_HEADERTEXT1,
                        szBuffer.as_mut_ptr(),
                        (core::mem::size_of::<[WCHAR; 32]>()
                            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
                    ) != 0
                    {
                        lf.lfHeight = 20;
                        lf.lfCharSet = (OEM_CHARSET as u8);
                        lf.lfQuality = (DEFAULT_QUALITY as u8);
                        lf.lfWeight = (FW_MEDIUM as i32);
                        wcscpy(lf.lfFaceName.as_mut_ptr(), RUSTCONV_WSTR_69BE2E3F4A5C.as_ptr());

                        hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
                        if !hFont.is_null() {
                            hFontOld = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;

                            DPtoLP(hdc, core::ptr::addr_of_mut!(txtRc) as PPOINT, 2);
                            SetTextColor(hdc, RGB!(255, 255, 255));
                            SetBkMode(hdc, (TRANSPARENT as i32));
                            DrawTextW(
                                hdc,
                                szBuffer.as_mut_ptr(),
                                -1,
                                core::ptr::addr_of_mut!(txtRc),
                                DT_BOTTOM | DT_SINGLELINE | DT_NOCLIP | DT_CENTER,
                            ); //DT_CENTER makes the text visible in RTL layouts...
                            SelectObject(hdc, hFontOld as HGDIOBJ);
                            DeleteObject(hFont as HGDIOBJ);
                        }
                    }

                    txtRc.left = bmpRc.right / 4;
                    txtRc.top = txtRc.bottom - 5;
                    if true /* C: #ifdef __REACTOS__ -- defined in this build */ {
                        txtRc.right = bmpRc.right * 4 / 5;
                    } else {
                        txtRc.right = bmpRc.right * 3 / 4;
                    }
                    txtRc.bottom = (*pInfo).headerbitmap.bmHeight * 9 / 10;

                    if LoadStringW(
                        hInst,
                        IDS_HEADERTEXT2,
                        szBuffer.as_mut_ptr(),
                        (core::mem::size_of::<[WCHAR; 32]>()
                            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
                    ) != 0
                    {
                        lf.lfHeight = 24;
                        lf.lfCharSet = (OEM_CHARSET as u8);
                        lf.lfQuality = (DEFAULT_QUALITY as u8);
                        lf.lfWeight = (FW_EXTRABOLD as i32);
                        wcscpy(lf.lfFaceName.as_mut_ptr(), RUSTCONV_WSTR_69BE2E3F4A5C.as_ptr());

                        hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
                        if !hFont.is_null() {
                            hFontOld = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;

                            DPtoLP(hdc, core::ptr::addr_of_mut!(txtRc) as PPOINT, 2);
                            SetTextColor(hdc, RGB!(255, 255, 255));
                            SetBkMode(hdc, (TRANSPARENT as i32));
                            DrawTextW(
                                hdc,
                                szBuffer.as_mut_ptr(),
                                -1,
                                core::ptr::addr_of_mut!(txtRc),
                                DT_TOP | DT_SINGLELINE,
                            );
                            SelectObject(hdc, hFontOld as HGDIOBJ);
                            DeleteObject(hFont as HGDIOBJ);
                        }
                    }

                    DeleteDC(hdcMem);
                }

                EndPaint(hDlg, core::ptr::addr_of_mut!(ps));
            }
        }

        WM_CLOSE => {
            Cleanup(pInfo);
            EndDialog(hDlg, 0);
        }

        _ => {
            return (FALSE) as INT_PTR;
        }
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn OpenRDPConnectDialog(
    hInstance: HINSTANCE,
    pRdpSettings: PRDPSETTINGS,
) -> BOOL {
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();

    hInst = hInstance;

    iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccx.dwICC = ICC_TAB_CLASSES as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

    return (DialogBoxParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_CONNECTDIALOG as WORD) as LPCWSTR,
        core::ptr::null_mut(),
        Some(DlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
        pRdpSettings as LPARAM,
    ) == IDOK as INT_PTR) as BOOL;
}
