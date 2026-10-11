use crate::msconfig::msconfig::DisableAllExcept;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_01AC0DF2395A: [u16; 12] = [
    0x0066u16, 0x0072, 0x0065, 0x0065, 0x006C, 0x0064, 0x0072, 0x002E, 0x0069, 0x006E,
    0x0069, 0x0000,
];
pub static RUSTCONV_WSTR_0783B6D69C02: [u16; 9] = [0x0062u16, 0x006F, 0x006F, 0x0074, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static RUSTCONV_WSTR_31BB95C6B5D6: [u16; 9] = [0x0064u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_789ABEADDBB7: [u16; 9] = [0x0074u16, 0x0069, 0x006D, 0x0065, 0x006F, 0x0075, 0x0074, 0x003D, 0x0000];
pub static RUSTCONV_WSTR_B85BE71CE3BC: [u16; 3] = [0x0072u16, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hFreeLdrPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hFreeLdrDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut Settings: FREELDR_SETTINGS = FREELDR_SETTINGS {
    TimeOut: 0 as ULONG,
    szDefaultOS: [
        0 as ::core::ffi::c_int as WCHAR,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ],
    szDefaultPos: 0 as ULONG,
    OSConfigurationCount: 0 as ULONG,
    UseBootIni: (FALSE) as i32,
};

#[repr(C)]
pub struct FREELDR_SETTINGS {
    pub TimeOut: ULONG,
    pub szDefaultOS: [WCHAR; 512],
    pub szDefaultPos: ULONG,
    pub OSConfigurationCount: ULONG,
    pub UseBootIni: BOOL,
}

pub type PFREELDR_SETTINGS = *mut FREELDR_SETTINGS;

pub const BUFFER_SIZE: usize = 512;

unsafe extern "C" fn LoadBootIni(szDrive: *mut WCHAR, hDlg: HWND) -> BOOL {
    let mut szBuffer: [WCHAR; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut file: *mut FILE = core::ptr::null_mut();
    let mut length: UINT = 0;
    let mut pos: LRESULT = 0;
    let mut hr: HRESULT = 0;

    hr = StringCbCopyW(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; BUFFER_SIZE]>() as size_t,
        szDrive,
    );
    if hr < 0 {
        return (FALSE) as BOOL;
    }

    hr = StringCbCatW(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; BUFFER_SIZE]>() as size_t,
        RUSTCONV_WSTR_01AC0DF2395A.as_ptr(),
    );
    if hr < 0 {
        return (FALSE) as BOOL;
    }

    file = (_wfopen(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        RUSTCONV_WSTR_B85BE71CE3BC.as_ptr(),
    )).cast();
    if file.is_null() {
        hr = StringCbCopyW(
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
            core::mem::size_of::<[WCHAR; BUFFER_SIZE]>() as size_t,
            szDrive,
        );
        if hr < 0 {
            return (FALSE) as BOOL;
        }

        hr = StringCbCatW(
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
            core::mem::size_of::<[WCHAR; BUFFER_SIZE]>() as size_t,
            RUSTCONV_WSTR_0783B6D69C02.as_ptr(),
        );
        if hr < 0 {
            return (FALSE) as BOOL;
        }

        file = (_wfopen(
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
            RUSTCONV_WSTR_B85BE71CE3BC.as_ptr(),
        )).cast();
        if file.is_null() {
            return (FALSE) as BOOL;
        }
    }

    hDlgCtrl = GetDlgItem(hDlg, (IDC_LIST_BOX as i32));

    while feof(file) == 0 {
        if !(fgetws(
            core::ptr::addr_of_mut!(szBuffer) as *mut wchar_t,
            (BUFFER_SIZE as i32),
            file,
        ))
        .is_null()
        {
            length = wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR) as UINT;
            if length > 1 {
                szBuffer[length as usize] = 0x0000u16;
                szBuffer[(length - 1) as usize] = 0x0000u16;

                pos = SendMessageW(
                    hDlgCtrl,
                    LB_ADDSTRING,
                    0,
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPARAM,
                );

                if szBuffer[0 as usize] == 0x005Bu16 {
                    continue;
                }

                if _wcsnicmp(
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                    RUSTCONV_WSTR_789ABEADDBB7.as_ptr(),
                    8,
                ) == 0
                {
                    Settings.TimeOut = _wtoi(
                        (core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR)
                            .offset(8 as isize),
                    ) as ULONG;
                    continue;
                }

                if _wcsnicmp(
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                    RUSTCONV_WSTR_31BB95C6B5D6.as_ptr(),
                    8,
                ) == 0
                {
                    wcscpy(
                        core::ptr::addr_of_mut!(Settings.szDefaultOS) as *mut WCHAR,
                        (core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR)
                            .offset(8 as isize),
                    );
                    continue;
                }
                if pos != (LB_ERR) as _ {
                    SendMessage(hDlgCtrl, LB_SETITEMDATA, pos as WPARAM, 1); // indicate that this item is an boot entry
                }
                Settings.OSConfigurationCount = Settings.OSConfigurationCount.wrapping_add(1);
            }
        }
    }

    fclose(file);
    Settings.UseBootIni = (TRUE) as i32;

    pos = SendMessageW(
        hDlgCtrl,
        LB_FINDSTRING,
        3,
        core::ptr::addr_of_mut!(Settings.szDefaultOS) as *mut WCHAR as LPARAM,
    );
    if pos != (LB_ERR) as _ {
        Settings.szDefaultPos = pos as ULONG;
        SendMessage(hDlgCtrl, LB_SETCURSEL, pos as WPARAM, 0);
    }

    SetDlgItemInt(hDlg, (IDC_TXT_BOOT_TIMEOUT as i32), Settings.TimeOut as UINT, (FALSE) as BOOL);
    if Settings.OSConfigurationCount < 2 {
        EnableWindow(GetDlgItem(hDlg, (IDC_BTN_SET_DEFAULT_BOOT as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hDlg, (IDC_BTN_MOVE_UP_BOOT_OPTION as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hDlg, (IDC_BTN_MOVE_DOWN_BOOT_OPTION as i32)), (FALSE) as BOOL);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitializeFreeLDRDialog(hDlg: HWND) -> BOOL {
    let mut winDir: [WCHAR; (PATH_MAX) as _] = [0; (PATH_MAX) as _];
    let mut ptr: *mut WCHAR = core::ptr::null_mut();

    GetWindowsDirectoryW(winDir.as_mut_ptr(), PATH_MAX);
    ptr = wcschr(winDir.as_mut_ptr(), 0x005Cu16) as *mut WCHAR;
    if ptr.is_null() {
        return (FALSE) as BOOL;
    }
    *ptr.offset(1 as isize) = 0x0000u16;
    return LoadBootIni(winDir.as_mut_ptr(), hDlg);
}
#[no_mangle]
pub unsafe extern "system" fn FreeLdrPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pos: LRESULT = 0;
    match message {
        WM_INITDIALOG => {
            hFreeLdrDialog = hDlg;
            SetWindowPos(
                hDlg,
                core::ptr::null_mut(),
                10 as ::core::ffi::c_int,
                32 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );
            InitializeFreeLDRDialog(hDlg);
            DisableAllExcept(hDlg, IDC_LIST_BOX as UINT); // FIXME: Implement saving
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (HIWORD!(wParam) as ::core::ffi::c_int) as u32 {
                LBN_SELCHANGE => {
                    pos = SendMessage(
                        lParam as HWND,
                        LB_GETCURSEL as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    );
                    if pos != LB_ERR as LRESULT {
                        let mut res: LPARAM = SendMessage(
                            lParam as HWND,
                            LB_GETITEMDATA as UINT,
                            pos as WPARAM,
                            0 as LPARAM,
                        ) as LPARAM;
                        if res == 0 {
                            SendMessage(
                                lParam as HWND,
                                LB_SETCURSEL as UINT,
                                Settings.szDefaultPos as WPARAM,
                                0 as LPARAM,
                            );
                        } else {
                            Settings.szDefaultPos = pos as ULONG;
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    return 0 as INT_PTR;
}
