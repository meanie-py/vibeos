// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8476B7D2BE8A: [u16; 16] = [
    0x0043u16, 0x0075, 0x0072, 0x0073, 0x006F, 0x0072, 0x0042, 0x006C, 0x0069, 0x006E,
    0x006B, 0x0052, 0x0061, 0x0074, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_C47486B2BD9A: [u16; 22] = [
    0x0043u16, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061,
    0x006E, 0x0065, 0x006C, 0x005C, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F,
    0x0070, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ID_BLINK_TIMER: i32 = 345;

#[repr(C)]
pub struct SPEED_DATA {
    pub nKeyboardDelay: i32,
    pub nOrigKeyboardDelay: i32,
    pub dwKeyboardSpeed: u32,
    pub dwOrigKeyboardSpeed: u32,
    pub uCaretBlinkTime: u32,
    pub uOrigCaretBlinkTime: u32,
    pub fShowCursor: i32,
    pub rcCursor: RECT,
}

pub type PSPEED_DATA = *mut SPEED_DATA;

unsafe extern "C" fn UpdateCaretBlinkTimeReg(uCaretBlinkTime: UINT) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szBuffer: [WCHAR; 12] = [0; 12];

    if RegOpenKeyExW(
        HKEY_CURRENT_USER as HKEY,
        RUSTCONV_WSTR_C47486B2BD9A.as_ptr() as LPCWSTR,
        0 as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    wsprintf(
        core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
        RUSTCONV_WSTR_A9EDBA2FC474.as_ptr() as LPCWSTR,
        uCaretBlinkTime,
    );

    RegSetValueExW(
        hKey,
        RUSTCONV_WSTR_8476B7D2BE8A.as_ptr() as LPCWSTR,
        0 as DWORD,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(szBuffer) as *const BYTE,
        (((wcslen(core::ptr::addr_of_mut!(szBuffer) as *const WCHAR) as usize)
            + (1 as usize))
            * core::mem::size_of::<WCHAR>()) as DWORD,
    );

    RegCloseKey(hKey);
}

unsafe extern "system" fn KeyboardSpeedProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSpeedData: PSPEED_DATA = core::ptr::null_mut();

    pSpeedData = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSPEED_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pSpeedData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<SPEED_DATA>()) as SIZE_T) as PSPEED_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pSpeedData as LONG_PTR);

            /* Get current keyboard delay */
            if !(SystemParametersInfo(SPI_GETKEYBOARDDELAY,
                                      core::mem::size_of::<INT>() as UINT,
                                      core::ptr::addr_of_mut!((*pSpeedData).nKeyboardDelay) as PVOID,
                                      0) != 0)
            {
                (*pSpeedData).nKeyboardDelay = 2;
            }

            (*pSpeedData).nOrigKeyboardDelay = (*pSpeedData).nKeyboardDelay;

            /* Get current keyboard delay */
            if !(SystemParametersInfo(SPI_GETKEYBOARDSPEED,
                                      core::mem::size_of::<DWORD>() as UINT,
                                      core::ptr::addr_of_mut!((*pSpeedData).dwKeyboardSpeed) as PVOID,
                                      0) != 0)
            {
                (*pSpeedData).dwKeyboardSpeed = 31;
            }

            (*pSpeedData).dwOrigKeyboardSpeed = (*pSpeedData).dwKeyboardSpeed;

            (*pSpeedData).fShowCursor = TRUE as BOOL;
            GetWindowRect(GetDlgItem(hwndDlg, (IDC_TEXT_CURSOR_BLINK as i32)), core::ptr::addr_of_mut!((*pSpeedData).rcCursor));
            ScreenToClient(hwndDlg, core::ptr::addr_of_mut!((*pSpeedData).rcCursor.left) as LPPOINT);
            ScreenToClient(hwndDlg, core::ptr::addr_of_mut!((*pSpeedData).rcCursor.right) as LPPOINT);

            /* Get the caret blink time and save its original value */
            (*pSpeedData).uOrigCaretBlinkTime = GetCaretBlinkTime();
            (*pSpeedData).uCaretBlinkTime = (*pSpeedData).uOrigCaretBlinkTime;

            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_DELAY as i32), TBM_SETRANGE, TRUE as WPARAM, MAKELONG!(0, 3) as LPARAM);
            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_DELAY as i32), TBM_SETPOS, TRUE as WPARAM, (3 - (*pSpeedData).nKeyboardDelay) as LPARAM);

            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_RATE as i32), TBM_SETRANGE, TRUE as WPARAM, MAKELONG!(0, 31) as LPARAM);
            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_RATE as i32), TBM_SETPOS, TRUE as WPARAM, (*pSpeedData).dwKeyboardSpeed as LPARAM);

            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_CURSOR_BLINK as i32), TBM_SETRANGE, TRUE as WPARAM, MAKELONG!(0, 10) as LPARAM);
            SendDlgItemMessage(hwndDlg, (IDC_SLIDER_CURSOR_BLINK as i32), TBM_SETPOS, TRUE as WPARAM, (12 - ((*pSpeedData).uCaretBlinkTime / (100 as UINT))) as LPARAM);

            /* Start the blink timer */
            (*pSpeedData).uCaretBlinkTime = if (*pSpeedData).uCaretBlinkTime >= 1200 as UINT {
                (-(1 as ::core::ffi::c_int)) as UINT
            } else {
                (*pSpeedData).uCaretBlinkTime
            };
            if ((*pSpeedData).uCaretBlinkTime as INT) > (0 as INT) {
                SetTimer(hwndDlg, ID_BLINK_TIMER as UINT_PTR, (*pSpeedData).uCaretBlinkTime, None);
            } else {
                PostMessage(hwndDlg, WM_TIMER, ID_BLINK_TIMER as WPARAM, 0);
            }
        }

        WM_HSCROLL => {
            if (lParam as HWND) == GetDlgItem(hwndDlg, (IDC_SLIDER_REPEAT_DELAY as i32)) {
                match (LOWORD!(wParam)) as u32 {
                    TB_LINEUP
                    | TB_LINEDOWN
                    | TB_PAGEUP
                    | TB_PAGEDOWN
                    | TB_TOP
                    | TB_BOTTOM
                    | TB_ENDTRACK => {
                        (*pSpeedData).nKeyboardDelay = 3 - (SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_DELAY as i32), TBM_GETPOS, 0, 0) as INT);
                        SystemParametersInfo(SPI_SETKEYBOARDDELAY,
                                             (*pSpeedData).nKeyboardDelay as UINT,
                                             core::ptr::null_mut(),
                                             0);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }

                    TB_THUMBTRACK => {
                        (*pSpeedData).nKeyboardDelay = 3 - (HIWORD!(wParam) as INT);
                        SystemParametersInfo(SPI_SETKEYBOARDDELAY,
                                             (*pSpeedData).nKeyboardDelay as UINT,
                                             core::ptr::null_mut(),
                                             0);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    _ => {}
                }
            } else if (lParam as HWND) == GetDlgItem(hwndDlg, (IDC_SLIDER_REPEAT_RATE as i32)) {
                match (LOWORD!(wParam)) as u32 {
                    TB_LINEUP
                    | TB_LINEDOWN
                    | TB_PAGEUP
                    | TB_PAGEDOWN
                    | TB_TOP
                    | TB_BOTTOM
                    | TB_ENDTRACK => {
                        (*pSpeedData).dwKeyboardSpeed = SendDlgItemMessage(hwndDlg, (IDC_SLIDER_REPEAT_RATE as i32), TBM_GETPOS, 0, 0) as DWORD;
                        SystemParametersInfo(SPI_SETKEYBOARDSPEED,
                                             (*pSpeedData).dwKeyboardSpeed as UINT,
                                             core::ptr::null_mut(),
                                             0);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }

                    TB_THUMBTRACK => {
                        (*pSpeedData).dwKeyboardSpeed = HIWORD!(wParam) as DWORD;
                        SystemParametersInfo(SPI_SETKEYBOARDSPEED,
                                             (*pSpeedData).dwKeyboardSpeed as UINT,
                                             core::ptr::null_mut(),
                                             0);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    _ => {}
                }
            } else if (lParam as HWND) == GetDlgItem(hwndDlg, (IDC_SLIDER_CURSOR_BLINK as i32)) {
                match (LOWORD!(wParam)) as u32 {
                    TB_LINEUP
                    | TB_LINEDOWN
                    | TB_PAGEUP
                    | TB_PAGEDOWN
                    | TB_TOP
                    | TB_BOTTOM
                    | TB_ENDTRACK => {
                        (*pSpeedData).uCaretBlinkTime = ((12 as UINT) - (SendDlgItemMessage(hwndDlg, (IDC_SLIDER_CURSOR_BLINK as i32), TBM_GETPOS, 0, 0) as UINT)) * (100 as UINT);
                        KillTimer(hwndDlg, (ID_BLINK_TIMER) as UINT_PTR);
                        (*pSpeedData).uCaretBlinkTime = if (*pSpeedData).uCaretBlinkTime >= 1200 as UINT {
                            (-(1 as ::core::ffi::c_int)) as UINT
                        } else {
                            (*pSpeedData).uCaretBlinkTime
                        };
                        if ((*pSpeedData).uCaretBlinkTime as INT) > (0 as INT) {
                            SetTimer(hwndDlg, ID_BLINK_TIMER as UINT_PTR, (*pSpeedData).uCaretBlinkTime, None);
                        } else if (*pSpeedData).fShowCursor != 0 {
                            SendMessage(hwndDlg, WM_TIMER, ID_BLINK_TIMER as WPARAM, 0);
                        }
                        SetCaretBlinkTime((*pSpeedData).uCaretBlinkTime);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }

                    TB_THUMBTRACK => {
                        (*pSpeedData).uCaretBlinkTime = ((12 as UINT) - (HIWORD!(wParam) as UINT)) * (100 as UINT);
                        KillTimer(hwndDlg, (ID_BLINK_TIMER) as UINT_PTR);
                        (*pSpeedData).uCaretBlinkTime = if (*pSpeedData).uCaretBlinkTime >= 1200 as UINT {
                            (-(1 as ::core::ffi::c_int)) as UINT
                        } else {
                            (*pSpeedData).uCaretBlinkTime
                        };
                        if ((*pSpeedData).uCaretBlinkTime as INT) > (0 as INT) {
                            SetTimer(hwndDlg, ID_BLINK_TIMER as UINT_PTR, (*pSpeedData).uCaretBlinkTime, None);
                        } else if (*pSpeedData).fShowCursor != 0 {
                            SendMessage(hwndDlg, WM_TIMER, ID_BLINK_TIMER as WPARAM, 0);
                        }
                        SetCaretBlinkTime((*pSpeedData).uCaretBlinkTime);
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                    _ => {}
                }
            }
        }

        WM_TIMER => {
            if wParam == ID_BLINK_TIMER as WPARAM {
                if (*pSpeedData).fShowCursor != 0 {
                    let mut hDC: HDC = GetDC(hwndDlg);
                    let mut hBrush: HBRUSH = GetSysColorBrush((COLOR_BTNTEXT) as i32);
                    FillRect(hDC, core::ptr::addr_of_mut!((*pSpeedData).rcCursor), hBrush);
                    ReleaseDC(hwndDlg, hDC);
                } else {
                    InvalidateRect(hwndDlg, core::ptr::addr_of_mut!((*pSpeedData).rcCursor), (TRUE) as BOOL);
                }

                (*pSpeedData).fShowCursor = ((*pSpeedData).fShowCursor == 0) as ::core::ffi::c_int as BOOL;
            }
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match (*lpnm).code as i32 {
                PSN_APPLY => {
                    /* Set the new keyboard settings */
                    if (*pSpeedData).uOrigCaretBlinkTime != (*pSpeedData).uCaretBlinkTime {
                        UpdateCaretBlinkTimeReg((*pSpeedData).uCaretBlinkTime);
                    }

                    SystemParametersInfo(SPI_SETKEYBOARDDELAY,
                                         (*pSpeedData).nKeyboardDelay as UINT,
                                         core::ptr::null_mut(),
                                         0);
                    SystemParametersInfo(SPI_SETKEYBOARDSPEED,
                                         (*pSpeedData).dwKeyboardSpeed as UINT,
                                         core::ptr::null_mut(),
                                         SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
                    return TRUE as INT_PTR;
                }

                PSN_RESET => {
                    /* Restore the original settings */
                    SetCaretBlinkTime((*pSpeedData).uOrigCaretBlinkTime);
                    SystemParametersInfo(SPI_SETKEYBOARDDELAY,
                                         (*pSpeedData).nOrigKeyboardDelay as UINT,
                                         core::ptr::null_mut(),
                                         0);
                    SystemParametersInfo(SPI_SETKEYBOARDSPEED,
                                         (*pSpeedData).dwOrigKeyboardSpeed as UINT,
                                         core::ptr::null_mut(),
                                         0);
                }

                _ => {}
            }
        }

        WM_DESTROY => {
            KillTimer(hwndDlg, (ID_BLINK_TIMER) as UINT_PTR);
            HeapFree(GetProcessHeap(), 0, pSpeedData as PVOID);
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn KeybHardwareProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut Guids: [GUID; 1] = core::mem::zeroed();
    Guids[0] = GUID_DEVCLASS_KEYBOARD;

    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    if uMsg == WM_INITDIALOG {
        /* Create the hardware page */
        DeviceCreateHardwarePageEx(
            hwndDlg,
            core::ptr::addr_of_mut!(Guids) as LPGUID,
            (core::mem::size_of::<[GUID; 1]>()
                / core::mem::size_of_val(&Guids[0])) as UINT,
            HWPD_STANDARDLIST,
        );
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON = core::ptr::null_mut();
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDC_CPLICON_2 as WORD) as LPCWSTR);
            SendMessageW(hwndDlg, WM_SETICON as UINT, ICON_BIG as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn KeyboardApplet(mut hwnd: HWND, mut uMsg: UINT, mut wParam: LPARAM, mut lParam: LPARAM) -> LONG {
    let mut hpsp: [HPROPSHEETPAGE; (MAX_CPL_PAGES as usize)] =
        [core::ptr::null_mut(); (MAX_CPL_PAGES as usize)];
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hpsxa: HPSXA = core::ptr::null_mut();
    let mut nPage: INT = 0 as INT;
    let mut ret: LONG;

    if uMsg == CPL_STARTWPARMSW as UINT && lParam != 0 as LPARAM {
        nPage = _wtoi((lParam as PWSTR) as *const WCHAR) as INT;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut ::core::ffi::c_void,
        (core::mem::size_of::<PROPSHEETHEADER>()) as usize,
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCE!(IDC_CPLICON_2) as LPCWSTR;
    psh.pszCaption = MAKEINTRESOURCE!(IDS_CPLNAME_2) as LPCWSTR;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    /* Load additional pages provided by shell extensions */
    let path: [u16; 67] = [
        'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16,
        'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16,
        's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16,
        'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16,
        'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16,
        'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
        'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        's' as u16, ' ' as u16, 'F' as u16, 'o' as u16, 'l' as u16, 'd' as u16, 'e' as u16,
        'r' as u16, '\\' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16,
        'a' as u16, 'r' as u16, 'd' as u16, 0,
    ];
    hpsxa = SHCreatePropSheetExtArray(
        HKEY_LOCAL_MACHINE as HKEY,
        path.as_ptr(),
        (MAX_CPL_PAGES as UINT).wrapping_sub(psh.nPages),
    );

    /* NOTE: The speed page (CPLPAGE_KEYBOARD_SPEED) cannot be replaced by
             shell extensions since Win2k! */
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psh),
        IDD_KEYBSPEED as WORD,
        Some(KeyboardSpeedProc),
    );
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psh),
        IDD_HARDWARE as WORD,
        Some(KeybHardwareProc),
    );

    if !hpsxa.is_null() {
        SHAddFromPropSheetExtArray(
            hpsxa,
            Some(PropSheetAddPage),
            core::ptr::addr_of_mut!(psh) as LPARAM,
        );
    }

    if nPage != 0 as INT && nPage as UINT <= psh.nPages {
        psh.__bindgen_anon_2.nStartPage = nPage as UINT;
    }

    ret = (PropertySheet(core::ptr::addr_of_mut!(psh) as LPCPROPSHEETHEADERW)
        != (-(1)) as INT_PTR) as LONG;

    if !hpsxa.is_null() {
        SHDestroyPropSheetExtArray(hpsxa);
    }

    return ret;
}
