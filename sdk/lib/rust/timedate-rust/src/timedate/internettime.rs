// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_23E7B20AEA52: [u16; 53] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0057, 0x0033, 0x0032, 0x0054, 0x0069, 0x006D,
    0x0065, 0x005C, 0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065,
    0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_4F8320D91E97: [u16; 2] = [0x0030u16, 0x0000];
pub static RUSTCONV_WSTR_64B1335C67DE: [u16; 7] = [0x004Eu16, 0x006F, 0x0053, 0x0079, 0x006E, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_709779F01788: [u16; 59] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0044, 0x0061, 0x0074, 0x0065, 0x0054, 0x0069, 0x006D, 0x0065,
    0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B3EC63A2C2D1: [u16; 10] = [0x006Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0068, 0x006F, 0x0073, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_BC2CD8BBA88F: [u16; 4] = [0x004Eu16, 0x0054, 0x0050, 0x0000];
pub static RUSTCONV_WSTR_D47D43486FD2: [u16; 5] = [0x0054u16, 0x0079, 0x0070, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SyncStatus: SYNC_STATUS = SYNC_STATUS {
    szSyncSuc: [0; 1024],
    szSyncWait: [0; 1024],
    szSyncErr: [0; 1024],
    szSyncType: [0; 1024],
    szSyncInit: [0; 1024],
};

unsafe extern "C" fn CreateNTPServerList(hwnd: HWND) {
    let mut hList: HWND = core::ptr::null_mut();
    let mut szValName: [WCHAR; (MAX_VALUE_NAME) as _] = [0; (MAX_VALUE_NAME) as _];
    let mut szData: [WCHAR; 256] = [0; 256];
    let mut dwIndex: DWORD = 0;
    let mut dwValSize: DWORD = 0;
    let mut dwNameSize: DWORD = 0;
    let mut dwDefault: DWORD = 1;
    let mut lRet: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    hList = GetDlgItem(hwnd,
                       (IDC_SERVERLIST as i32));

    lRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                        RUSTCONV_WSTR_709779F01788.as_ptr(),
                        0,
                        KEY_QUERY_VALUE,
                        core::ptr::addr_of_mut!(hKey));
    if (lRet) as u32 != ERROR_SUCCESS {
        return;
    }

    loop {
        // MAX_VALUE_NAME == 16383
        dwValSize = (16383 as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        szValName[0] = 0x0000u16;
        lRet = RegEnumValueW(hKey,
                             dwIndex,
                             szValName.as_mut_ptr(),
                             core::ptr::addr_of_mut!(dwValSize),
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             szData.as_mut_ptr() as LPBYTE,
                             core::ptr::addr_of_mut!(dwNameSize));
        if (lRet) as u32 == ERROR_SUCCESS {
            /* Get date from default reg value */
            if wcscmp(szValName.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr()) == 0 { // if (Index == 0)
                dwDefault = (_wtoi(szData.as_mut_ptr())) as u32;
                dwIndex = dwIndex.wrapping_add(1);
            } else {
                SendMessageW(hList,
                             CB_ADDSTRING,
                             0,
                             szData.as_mut_ptr() as LPARAM);
                dwIndex = dwIndex.wrapping_add(1);
            }
        } else if (lRet) as u32 != ERROR_MORE_DATA {
            break;
        }
    }

    if dwDefault < 1 || dwDefault > dwIndex {
        dwDefault = 1;
    }

    /* Server reg entries count from 1,
     * Combo boxes count from 0 */
    dwDefault = dwDefault.wrapping_sub(1);

    SendMessageW(hList,
                 CB_SETCURSEL,
                 (dwDefault) as WPARAM,
                 0);

    RegCloseKey(hKey);
}

unsafe extern "C" fn SetNTPServer(hwnd: HWND, bBeginUpdate: BOOL) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hList: HWND = core::ptr::null_mut();
    let mut uSel: UINT = 0;
    let mut szSel: [WCHAR; 4] = [0; 4];
    let mut lRet: LONG = 0;
    let mut buffer: [WCHAR; 256] = [0; 256];
    let mut szFormat: [WCHAR; (BUFSIZE) as _] = [0; (BUFSIZE) as _];

    hList = GetDlgItem(hwnd, IDC_SERVERLIST as ::core::ffi::c_int);

    uSel = SendMessageW(
        hList,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;

    SendDlgItemMessageW(
        hwnd,
        IDC_SERVERLIST as ::core::ffi::c_int,
        WM_GETTEXT as UINT,
        _countof!(buffer) as WPARAM,
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPARAM,
    );

    /* If the condition is true that means the user wants to update (synchronize) the time */
    if bBeginUpdate != 0 {
        /* Inform the user that the synchronization is about to begin (depending on how reachable the NTP server is) */
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(szFormat) as *mut WCHAR,
            _countof!(szFormat),
            core::ptr::addr_of!(SyncStatus.szSyncWait) as *const WCHAR,
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        );
        SetDlgItemTextW(
            hwnd,
            IDC_SUCSYNC as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
        );
    }

    /* If there is new data entered then save it in the registry
       The same key name of "0" is used to store all user entered values
    */
    if uSel == (0 as UINT).wrapping_sub(1 as UINT) {
        lRet = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE as HKEY,
            RUSTCONV_WSTR_709779F01788.as_ptr(),
            0 as DWORD,
            KEY_SET_VALUE as REGSAM,
            core::ptr::addr_of_mut!(hKey),
        ) as LONG;
        if lRet != ERROR_SUCCESS as LONG {
            DisplayWin32Error!(lRet as DWORD);
            return;
        }
        lRet = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_4F8320D91E97.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE as *const BYTE,
            ((wcslen(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as usize
                + 1 as usize)
                * ::core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LONG;
        if lRet != ERROR_SUCCESS as LONG {
            DisplayWin32Error!(lRet as DWORD);
        }
    }

    /* Server reg entries count from 1,
     * Combo boxes count from 0 */
    uSel = uSel.wrapping_add(1);

    /* Convert to wide char */
    _itow(
        uSel as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(szSel) as *mut WCHAR,
        10 as ::core::ffi::c_int,
    );

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_709779F01788.as_ptr(),
        0 as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) as LONG;
    if lRet != ERROR_SUCCESS as LONG {
        DisplayWin32Error!(lRet as DWORD);
        return;
    }

    lRet = RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        0 as DWORD,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(szSel) as *mut WCHAR as LPBYTE as *const BYTE,
        ((wcslen(core::ptr::addr_of_mut!(szSel) as *mut WCHAR) as usize
            + 1 as usize)
            * ::core::mem::size_of::<WCHAR>()) as DWORD,
    ) as LONG;
    if lRet != ERROR_SUCCESS as LONG {
        DisplayWin32Error!(lRet as DWORD);
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn EnableDialogText(hwnd: HWND) {
    let mut bChecked: BOOL = 0;
    let mut uCheck: UINT = 0;

    uCheck = SendDlgItemMessageW(hwnd, (IDC_AUTOSYNC as i32), BM_GETCHECK, 0, 0) as UINT;
    bChecked = if uCheck == BST_CHECKED as UINT { (TRUE) as i32 } else { (FALSE) as i32 };

    EnableWindow(GetDlgItem(hwnd, (IDC_SERVERTEXT as i32)), bChecked);
    EnableWindow(GetDlgItem(hwnd, (IDC_SERVERLIST as i32)), bChecked);
    EnableWindow(GetDlgItem(hwnd, (IDC_UPDATEBUTTON as i32)), bChecked);
    EnableWindow(GetDlgItem(hwnd, (IDC_SUCSYNC as i32)), bChecked);
    EnableWindow(GetDlgItem(hwnd, (IDC_NEXTSYNC as i32)), bChecked);
}

unsafe extern "C" fn SyncNTPStatusInit(hwnd: HWND) {
    LoadStringW(
        hApplet,
        IDS_INETTIMEWELCOME as UINT,
        core::ptr::addr_of_mut!(SyncStatus.szSyncInit) as LPWSTR,
        _countof!(SyncStatus.szSyncInit) as core::ffi::c_int,
    );
    LoadStringW(
        hApplet,
        IDS_INETTIMESUCSYNC as UINT,
        core::ptr::addr_of_mut!(SyncStatus.szSyncSuc) as LPWSTR,
        _countof!(SyncStatus.szSyncSuc) as core::ffi::c_int,
    );
    LoadStringW(
        hApplet,
        IDS_INETTIMESYNCING as UINT,
        core::ptr::addr_of_mut!(SyncStatus.szSyncWait) as LPWSTR,
        _countof!(SyncStatus.szSyncWait) as core::ffi::c_int,
    );
    LoadStringW(
        hApplet,
        IDS_INETTIMEERROR as UINT,
        core::ptr::addr_of_mut!(SyncStatus.szSyncErr) as LPWSTR,
        _countof!(SyncStatus.szSyncErr) as core::ffi::c_int,
    );
    LoadStringW(
        hApplet,
        IDS_INETTIMESUCFILL as UINT,
        core::ptr::addr_of_mut!(SyncStatus.szSyncType) as LPWSTR,
        _countof!(SyncStatus.szSyncType) as core::ffi::c_int,
    );

    /*
     * TODO: XP's and Server 2003's timedate.cpl loads the last successful attempt of the NTP synchronization
     * displaying the last time and date of the said sync. I have no idea how does timedate.cpl remember its last
     * successful sync so for the time being, we will only load the initial remark string.
    */
    SetDlgItemTextW(
        hwnd,
        IDC_SUCSYNC as core::ffi::c_int,
        core::ptr::addr_of_mut!(SyncStatus.szSyncInit) as *mut WCHAR as LPCWSTR,
    );
}

unsafe extern "C" fn UpdateNTPStatus(hwnd: HWND, dwReason: DWORD) {
    let mut szFormat: [WCHAR; (BUFSIZE) as _] = [0; (BUFSIZE) as _];
    let mut szNtpServerName: [WCHAR; (MAX_VALUE_NAME) as _] = [0; (MAX_VALUE_NAME) as _];
    let mut szLocalDate: [WCHAR; (BUFSIZE) as _] = [0; (BUFSIZE) as _];
    let mut szLocalTime: [WCHAR; (BUFSIZE) as _] = [0; (BUFSIZE) as _];
    let mut hDlgComboList: HWND;

    /* Retrieve the server NTP name from the edit box */
    hDlgComboList = GetDlgItem(hwnd, (IDC_SERVERLIST as i32));
    SendMessageW(
        hDlgComboList,
        WM_GETTEXT,
        _countof!(szNtpServerName) as WPARAM,
        core::ptr::addr_of_mut!(szNtpServerName) as *mut WCHAR as LPARAM,
    );

    /* Iterate over the case reasons so we can compute the exact status of the NTP synchronization */
    match dwReason {
        /* The NTP time synchronization has completed successfully */
        ERROR_SUCCESS => {
            /* Get the current date based on the locale identifier */
            GetDateFormatW(
                LOCALE_USER_DEFAULT,
                DATE_SHORTDATE,
                core::ptr::null::<SYSTEMTIME>(),
                core::ptr::null::<WCHAR>(),
                core::ptr::addr_of_mut!(szLocalDate) as *mut WCHAR,
                _countof!(szLocalDate) as ::core::ffi::c_int,
            );

            /* Get the current time based on the locale identifier */
            GetTimeFormatW(
                LOCALE_USER_DEFAULT,
                TIME_NOSECONDS,
                core::ptr::null::<SYSTEMTIME>(),
                core::ptr::null::<WCHAR>(),
                core::ptr::addr_of_mut!(szLocalTime) as *mut WCHAR,
                _countof!(szLocalTime) as ::core::ffi::c_int,
            );

            /* Format the resource sting with the given NTP server name and the current time data */
            StringCchPrintfW!(
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR,
                _countof!(szFormat) as size_t,
                core::ptr::addr_of_mut!(SyncStatus.szSyncSuc) as *mut WCHAR,
                core::ptr::addr_of_mut!(szNtpServerName) as *mut WCHAR,
                core::ptr::addr_of_mut!(szLocalDate) as *mut WCHAR,
                core::ptr::addr_of_mut!(szLocalTime) as *mut WCHAR,
            );
            SetDlgItemTextW(
                hwnd,
                (IDC_SUCSYNC as i32),
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
            );
        }

        /* Empty field data has been caught -- simply tell the user to write the NTP name to continue */
        ERROR_INVALID_DATA => {
            SetDlgItemTextW(
                hwnd,
                (IDC_SUCSYNC as i32),
                core::ptr::addr_of_mut!(SyncStatus.szSyncType) as *mut WCHAR as LPCWSTR,
            );
            DPRINT!(
                b"UpdateNTPStatus(): The user didn't submit any NTP server name!\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }

        /* General failure -- the NTP synchronization has failed for whatever reason */
        _ => {
            StringCchPrintfW!(
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR,
                _countof!(szFormat) as size_t,
                core::ptr::addr_of_mut!(SyncStatus.szSyncErr) as *mut WCHAR,
                core::ptr::addr_of_mut!(szNtpServerName) as *mut WCHAR,
            );
            SetDlgItemTextW(
                hwnd,
                (IDC_SUCSYNC as i32),
                core::ptr::addr_of_mut!(szFormat) as *mut WCHAR as LPCWSTR,
            );
            DPRINT!(
                b"UpdateNTPStatus(): Failed to synchronize the time! (Error: %lu).\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                dwReason,
            );
        }
    }
}

unsafe extern "C" fn GetSyncSetting(hwnd: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szData: [WCHAR; 8] = [0; 8];
    let mut dwSize: DWORD = 0;

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE as HKEY,
        RUSTCONV_WSTR_23E7B20AEA52.as_ptr(),
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) == ERROR_SUCCESS as LSTATUS
    {
        dwSize = (8 as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        if RegQueryValueExW(
            hKey,
            RUSTCONV_WSTR_D47D43486FD2.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(szData) as *mut WCHAR as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) == ERROR_SUCCESS as LSTATUS
        {
            if wcscmp(
                core::ptr::addr_of_mut!(szData) as *mut WCHAR,
                RUSTCONV_WSTR_BC2CD8BBA88F.as_ptr(),
            ) == 0 as i32
            {
                SendDlgItemMessageW(
                    hwnd,
                    IDC_AUTOSYNC as i32,
                    BM_SETCHECK as UINT,
                    BST_CHECKED as WPARAM,
                    0 as LPARAM,
                );
            } else {
                SendDlgItemMessageW(
                    hwnd,
                    IDC_AUTOSYNC as i32,
                    BM_SETCHECK as UINT,
                    BST_UNCHECKED as WPARAM,
                    0 as LPARAM,
                );
            }
        }

        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn OnInitDialog(hwnd: HWND) {
    GetSyncSetting(hwnd);
    EnableDialogText(hwnd);
    CreateNTPServerList(hwnd);
    SyncNTPStatusInit(hwnd);
    SetWindowLongPtr(hwnd, DWLP_USER as ::core::ffi::c_int, FALSE as LONG_PTR);
}

unsafe extern "C" fn OnAutoSync(Sync: BOOL) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lRet: LONG = 0;
    let mut szAuto: LPCWSTR = core::ptr::null();

    if Sync != 0 {
        szAuto = RUSTCONV_WSTR_BC2CD8BBA88F.as_ptr();
    } else {
        szAuto = RUSTCONV_WSTR_64B1335C67DE.as_ptr();
    }

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_23E7B20AEA52.as_ptr(),
        0,
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(hKey),
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        DisplayWin32Error!(lRet as DWORD);
        return;
    }

    lRet = RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_D47D43486FD2.as_ptr(),
        0,
        REG_SZ,
        szAuto as LPBYTE,
        ((wcslen(szAuto) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
    );
    if (lRet) as u32 != ERROR_SUCCESS {
        DisplayWin32Error!(lRet as DWORD);
    }

    RegCloseKey(hKey);
}

unsafe extern "system" fn UpdateThread(lpParameter: LPVOID) -> DWORD {
    let mut hwndDlg: HWND;
    let mut dwError: DWORD;

    hwndDlg = lpParameter as HWND;

    SetNTPServer(hwndDlg, (TRUE) as BOOL);

    dwError = W32TimeSyncNow(RUSTCONV_WSTR_B3EC63A2C2D1.as_ptr(), 0, 0);
    UpdateNTPStatus(hwndDlg, dwError);

    SetWindowLongPtr(hwndDlg, DWLP_USER, FALSE as LONG_PTR);
    return 0;
}

unsafe extern "C" fn OnUpdate(hwndDlg: HWND) {
    if (GetWindowLongPtr(hwndDlg, DWLP_USER) as BOOL) as u32 == FALSE {
        SetWindowLongPtr(hwndDlg, DWLP_USER, TRUE as LONG_PTR);

        if CreateThread(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            0,
            Some(UpdateThread),
            hwndDlg as PVOID,
            0,
            core::ptr::null_mut::<DWORD>(),
        )
        .is_null()
        {
            UpdateNTPStatus(hwndDlg, GetLastError());
            SetWindowLongPtr(hwndDlg, DWLP_USER, FALSE as LONG_PTR);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn InetTimePageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            OnInitDialog(hwndDlg);
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_UPDATEBUTTON => {
                    OnUpdate(hwndDlg);
                }
                IDC_SERVERLIST => {
                    if ((HIWORD!(wParam) as ::core::ffi::c_int) as u32 == CBN_SELCHANGE)
                        || ((HIWORD!(wParam) as ::core::ffi::c_int) as u32 == CBN_EDITCHANGE)
                    {
                        /* Enable the 'Apply' button */
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                IDC_AUTOSYNC => {
                    if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == BN_CLICKED {
                        EnableDialogText(hwndDlg);

                        /* Enable the 'Apply' button */
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }
        WM_DESTROY => {}
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_APPLY => {
                    SetNTPServer(hwndDlg, FALSE as BOOL);

                    if SendDlgItemMessageW(
                        hwndDlg,
                        (IDC_AUTOSYNC as i32),
                        BM_GETCHECK as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED as LRESULT
                    {
                        OnAutoSync(TRUE as BOOL);
                    } else {
                        OnAutoSync(FALSE as BOOL);
                    }

                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
