use crate::prelude::*;
use crate::sol::solundo::ClearUndo;
use crate::sol::solcreate::CreateSol;
use crate::sol::solgame::GameFinished;
use crate::sol::solgame::NewGame;
use crate::sol::solundo::Undo;
use crate::sol::solgame::UpdatePileDisplay;
macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// SCORE_STD, CARDBACK_RES_START and NUM_CARDBACKS are defined in the C header,
// which was not provided. The values below are placeholders and must be checked.

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut hInstance: HINSTANCE = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut hwndMain: HWND = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut hwndStatus: HWND = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut SolWnd: *mut CARDWINDOW = core::ptr::null_mut();

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut szAppName: [TCHAR; 128] = [0; 128];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgQuit: [TCHAR; 128] = [0; 128];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgAbout: [TCHAR; 128] = [0; 128];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgWin: [TCHAR; 128] = [0; 128];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgDeal: [TCHAR; 128] = [0; 128];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgScore: [TCHAR; 64] = [0; 64];
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut MsgTime: [TCHAR; 64] = [0; 64];

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut nDrawCards: i32 = 1;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut nScoreMode: i32 = (SCORE_STD as i32);
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut nCardBack: i32 = (CARDBACK_RES_START as i32);
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut fKeepScore: i32 = 0; // FALSE
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut fShowTime: i32 = 1; // TRUE
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut fShowStatus: i32 = 1; // TRUE

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut lScore: i32 = 0;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut dwElapsed: u32 = 0;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut fGameStarted: i32 = 0;
#[allow(non_upper_case_globals)]
static mut fTimerRunning: i32 = 0; // FALSE

// (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004f, 0x0053, 0x005c, 0x0053, 0x006f, 0x006c, 0x0069, 0x0074, 0x0061, 0x0069, 0x0072, 0x0065, 0x0000u16]).as_ptr() /* _T "Software\\ReactOS\\Solitaire" */ with NUL terminator (27 code units)
#[allow(non_upper_case_globals)]
pub static REGISTRY_KEY: [TCHAR; 27] = [
    0x53, 0x6F, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, // Software
    0x5C,                                           // \
    0x52, 0x65, 0x61, 0x63, 0x74, 0x4F, 0x53,       // ReactOS
    0x5C,                                           // \
    0x53, 0x6F, 0x6C, 0x69, 0x74, 0x61, 0x69, 0x72, 0x65, // Solitaire
    0x00,
];

#[repr(C)]
pub struct CARDBACK {
    pub hSelf: HWND,
    pub pfnOldProc: WNDPROC,
    pub nSlot: i32, /* slot in the card bitmap strip */
    pub bSelected: i32,
}
pub type PCARDBACK = *mut CARDBACK;

#[allow(non_upper_case_globals)]
pub static mut CardBacks: [CARDBACK; (NUM_CARDBACKS as usize)] = unsafe { core::mem::zeroed() };
unsafe extern "C" fn LoadDword(hKey: HKEY, pszName: LPCTSTR, pnValue: *mut INT) {
    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    if (RegQueryValueEx(hKey, pszName, core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), core::ptr::addr_of_mut!(dwValue) as LPBYTE, core::ptr::addr_of_mut!(cbData))) as u32 == ERROR_SUCCESS
        && dwType == REG_DWORD && cbData == core::mem::size_of::<DWORD>() as DWORD
    {
        *pnValue = dwValue as INT;
    }
}

unsafe extern "C" fn SaveDword(hKey: HKEY, pszName: LPCTSTR, nValue: INT) {
    let dwValue: DWORD = nValue as DWORD;
    RegSetValueEx(hKey, pszName, 0, REG_DWORD, core::ptr::addr_of!(dwValue) as *const BYTE, core::mem::size_of::<DWORD>() as DWORD);
}
unsafe extern "C" fn LoadSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegOpenKeyEx(HKEY_CURRENT_USER, (REGISTRY_KEY).as_ptr(), 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
        LoadDword(hKey, ['D' as u16, 'r' as u16, 'a' as u16, 'w' as u16, 'C' as u16, 'a' as u16, 'r' as u16, 'd' as u16, 's' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(nDrawCards));
        LoadDword(hKey, ['S' as u16, 'c' as u16, 'o' as u16, 'r' as u16, 'e' as u16, 'M' as u16, 'o' as u16, 'd' as u16, 'e' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(nScoreMode));
        LoadDword(hKey, ['C' as u16, 'a' as u16, 'r' as u16, 'd' as u16, 'B' as u16, 'a' as u16, 'c' as u16, 'k' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(nCardBack));
        LoadDword(hKey, ['K' as u16, 'e' as u16, 'e' as u16, 'p' as u16, 'S' as u16, 'c' as u16, 'o' as u16, 'r' as u16, 'e' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(fKeepScore));
        LoadDword(hKey, ['S' as u16, 'h' as u16, 'o' as u16, 'w' as u16, 'T' as u16, 'i' as u16, 'm' as u16, 'e' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(fShowTime));
        LoadDword(hKey, ['S' as u16, 't' as u16, 'a' as u16, 't' as u16, 'u' as u16, 's' as u16, 'B' as u16, 'a' as u16, 'r' as u16, 0u16].as_ptr(), core::ptr::addr_of_mut!(fShowStatus));

        RegCloseKey(hKey);
    }

    /* Anything out of range is a hand-edited or newer registry.  Fall back
       rather than, say, indexing the bitmap strip off its end. */
    if nDrawCards != 1 && nDrawCards != 3 {
        nDrawCards = 1;
    }
    if nScoreMode < (SCORE_NONE as i32) || nScoreMode > (SCORE_VEGAS as i32) {
        nScoreMode = (SCORE_STD as i32);
    }
    if nCardBack < (CARDBACK_RES_START as i32) || (nCardBack) as u32 >= CARDBACK_RES_START + NUM_CARDBACKS {
        nCardBack = (CARDBACK_RES_START as i32);
    }

    fKeepScore = if fKeepScore != 0 { (TRUE) as i32 } else { (FALSE) as i32 };
    fShowTime = if fShowTime != 0 { (TRUE) as i32 } else { (FALSE) as i32 };
    fShowStatus = if fShowStatus != 0 { (TRUE) as i32 } else { (FALSE) as i32 };
}

unsafe extern "C" fn SaveSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;

    if RegCreateKeyEx(HKEY_CURRENT_USER, (REGISTRY_KEY).as_ptr(), 0, core::ptr::null_mut(), REG_OPTION_NON_VOLATILE,
                       KEY_SET_VALUE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::addr_of_mut!(dwDisposition)) != ERROR_SUCCESS as i32
    {
        return;
    }

    SaveDword(hKey, (&[0x0044, 0x0072, 0x0061, 0x0077, 0x0043, 0x0061, 0x0072, 0x0064, 0x0073, 0x0000u16]).as_ptr() /* _T "DrawCards" */, nDrawCards);
    SaveDword(hKey, (&[0x0053, 0x0063, 0x006f, 0x0072, 0x0065, 0x004d, 0x006f, 0x0064, 0x0065, 0x0000u16]).as_ptr() /* _T "ScoreMode" */, nScoreMode);
    SaveDword(hKey, (&[0x0043, 0x0061, 0x0072, 0x0064, 0x0042, 0x0061, 0x0063, 0x006b, 0x0000u16]).as_ptr() /* _T "CardBack" */,  nCardBack);
    SaveDword(hKey, (&[0x004b, 0x0065, 0x0065, 0x0070, 0x0053, 0x0063, 0x006f, 0x0072, 0x0065, 0x0000u16]).as_ptr() /* _T "KeepScore" */, fKeepScore);
    SaveDword(hKey, (&[0x0053, 0x0068, 0x006f, 0x0077, 0x0054, 0x0069, 0x006d, 0x0065, 0x0000u16]).as_ptr() /* _T "ShowTime" */,  fShowTime);
    SaveDword(hKey, (&[0x0053, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0042, 0x0061, 0x0072, 0x0000u16]).as_ptr() /* _T "StatusBar" */, fShowStatus);

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn GetScoreMode() -> INT {
    return nScoreMode;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateStatusBar() {
    let mut szText: [TCHAR; 128] = [0 as TCHAR; 128];

    if hwndStatus.is_null() {
        return;
    }

    if GetScoreMode() == (SCORE_NONE as i32) {
        szText[0] = ('\0' as u16);
    } else {
        wsprintf(szText.as_mut_ptr(), addr_of!(MsgScore) as *const TCHAR, lScore);
    }

    SendMessage(hwndStatus, (SB_SETTEXT as u32), 0, szText.as_ptr() as LPARAM);

    if fShowTime != 0 {
        wsprintf(szText.as_mut_ptr(), addr_of!(MsgTime) as *const TCHAR, dwElapsed as INT);
    } else {
        szText[0] = ('\0' as u16);
    }

    SendMessage(hwndStatus, (SB_SETTEXT as u32), 1, szText.as_ptr() as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn SetUndoMenuState(bEnable: BOOL) {
    let hMenu: HMENU = GetMenu(hwndMain);

    if hMenu != core::ptr::null_mut() {
        EnableMenuItem(hMenu, IDM_GAME_UNDO as UINT,
                       (MF_BYCOMMAND as UINT) | (if bEnable != 0 { MF_ENABLED as UINT } else { MF_GRAYED as UINT }));
    }
}
#[no_mangle]
pub unsafe extern "C" fn StartPlayTimer() {
    if ((fTimerRunning) == 0) {
        SetTimer(hwndMain, (IDT_PLAYTIMER) as UINT_PTR, 1000, None);
        fTimerRunning = (TRUE) as i32;
    }
}
#[no_mangle]
pub unsafe extern "C" fn StopPlayTimer() {
    if fTimerRunning != 0 {
        KillTimer(hwndMain, (IDT_PLAYTIMER) as UINT_PTR);
        fTimerRunning = (FALSE) as i32;
    }
}

unsafe extern "C" fn StatusBarHeight() -> INT {
    let mut rc: RECT = core::mem::zeroed();

    if hwndStatus.is_null() || IsWindowVisible(hwndStatus) == 0 {
        return 0;
    }

    GetWindowRect(hwndStatus, core::ptr::addr_of_mut!(rc));
    return rc.bottom - rc.top;
}

unsafe extern "C" fn SizeStatusBar() {
    let mut rc: RECT = core::mem::zeroed();
    let mut parts: [i32; 2] = [0; 2];

    if hwndStatus.is_null() {
        return;
    }

    GetClientRect(hwndMain, core::ptr::addr_of_mut!(rc));
    parts[0 as usize] = rc.right / 2;
    parts[1 as usize] = -1;

    SendMessage(hwndStatus, SB_SETPARTS, 2, parts.as_mut_ptr() as LPARAM);
}

unsafe extern "C" fn LayoutChildren() {
    let mut rc: RECT = core::mem::zeroed();

    if hwndMain.is_null() {
        return;
    }

    GetClientRect(hwndMain, core::ptr::addr_of_mut!(rc));

    if !hwndStatus.is_null() {
        SendMessage(hwndStatus, WM_SIZE, 0, 0);
        SizeStatusBar();
    }

    if !SolWnd.is_null() {
        MoveWindow(CardWindow_GetHwnd(SolWnd), 0, 0, rc.right, rc.bottom - StatusBarHeight(), (TRUE) as BOOL);
    }

    UpdateStatusBar();
}

unsafe extern "C" fn ApplyStatusBarVisibility() {
    if hwndStatus.is_null() {
        return;
    }
    ShowWindow(hwndStatus, if fShowStatus != 0 { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
    LayoutChildren();
}

unsafe extern "C" fn ScoreModeToId(nMode: INT) -> INT {
    if nMode == (SCORE_VEGAS as i32) {
        return (IDC_OPT_VEGAS) as INT;
    } else if nMode == (SCORE_NONE as i32) {
        return (IDC_OPT_NOSCORE) as INT;
    } else {
        return (IDC_OPT_STANDARD) as INT;
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn OptionsDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            CheckRadioButton(hDlg, (IDC_OPT_DRAWONE as i32), (IDC_OPT_DRAWTHREE as i32),
                if nDrawCards == 3 { (IDC_OPT_DRAWTHREE as i32) } else { (IDC_OPT_DRAWONE as i32) });
            CheckRadioButton(hDlg, (IDC_OPT_STANDARD as i32), (IDC_OPT_NOSCORE as i32), ScoreModeToId(nScoreMode));
            CheckDlgButton(hDlg, (IDC_OPT_KEEPSCORE as i32), if fKeepScore != 0 { BST_CHECKED } else { BST_UNCHECKED });
            CheckDlgButton(hDlg, (IDC_OPT_SHOWTIME as i32), if fShowTime != 0 { BST_CHECKED } else { BST_UNCHECKED });
            CheckDlgButton(hDlg, (IDC_OPT_STATUSBAR as i32), if fShowStatus != 0 { BST_CHECKED } else { BST_UNCHECKED });

            /* A running bank is a Vegas idea; it means nothing to the others. */
            EnableWindow(GetDlgItem(hDlg, (IDC_OPT_KEEPSCORE as i32)), (nScoreMode == (SCORE_VEGAS as i32)) as i32);
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDC_OPT_STANDARD | IDC_OPT_VEGAS | IDC_OPT_NOSCORE => {
                    EnableWindow(GetDlgItem(hDlg, (IDC_OPT_KEEPSCORE as i32)),
                                 (IsDlgButtonChecked(hDlg, (IDC_OPT_VEGAS as i32)) == BST_CHECKED) as i32);
                    return (TRUE) as INT_PTR;
                }

                IDOK => {
                    let nOldMode: INT = nScoreMode;

                    nDrawCards = if IsDlgButtonChecked(hDlg, (IDC_OPT_DRAWTHREE as i32)) == BST_CHECKED { 3 } else { 1 };

                    if IsDlgButtonChecked(hDlg, (IDC_OPT_VEGAS as i32)) == BST_CHECKED {
                        nScoreMode = (SCORE_VEGAS as i32);
                    } else if IsDlgButtonChecked(hDlg, (IDC_OPT_NOSCORE as i32)) == BST_CHECKED {
                        nScoreMode = (SCORE_NONE as i32);
                    } else {
                        nScoreMode = (SCORE_STD as i32);
                    }

                    fKeepScore  = (IsDlgButtonChecked(hDlg, (IDC_OPT_KEEPSCORE as i32)) == BST_CHECKED) as i32;
                    fShowTime   = (IsDlgButtonChecked(hDlg, (IDC_OPT_SHOWTIME as i32)) == BST_CHECKED) as i32;
                    fShowStatus = (IsDlgButtonChecked(hDlg, (IDC_OPT_STATUSBAR as i32)) == BST_CHECKED) as i32;

                    /* Switching scoring systems mid-deal: the running total
                       means something else now, and so does the score the
                       undo module has banked.  Start both over rather than
                       mixing two currencies.  The deal itself is left alone -
                       the player keeps the game they are in. */
                    if nScoreMode != nOldMode {
                        lScore = if nScoreMode == (SCORE_VEGAS as i32) { -(SCORE_VEGAS_STAKE as i32) } else { 0 };
                        ClearUndo();
                    }

                    SaveSettings();
                    ApplyStatusBarVisibility();
                    UpdateStatusBar();

                    EndDialog(hDlg, (TRUE) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, (FALSE) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn OptionsDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match uMsg {
        WM_INITDIALOG => {
            CheckRadioButton(hDlg, (IDC_OPT_DRAWONE as i32), (IDC_OPT_DRAWTHREE as i32),
                if nDrawCards == 3 { (IDC_OPT_DRAWTHREE as i32) } else { (IDC_OPT_DRAWONE as i32) });
            CheckRadioButton(hDlg, (IDC_OPT_STANDARD as i32), (IDC_OPT_NOSCORE as i32), ScoreModeToId(nScoreMode));
            CheckDlgButton(hDlg, (IDC_OPT_KEEPSCORE as i32), if fKeepScore != 0 { BST_CHECKED } else { BST_UNCHECKED });
            CheckDlgButton(hDlg, (IDC_OPT_SHOWTIME as i32), if fShowTime != 0 { BST_CHECKED } else { BST_UNCHECKED });
            CheckDlgButton(hDlg, (IDC_OPT_STATUSBAR as i32), if fShowStatus != 0 { BST_CHECKED } else { BST_UNCHECKED });

            /* A running bank is a Vegas idea; it means nothing to the others. */
            EnableWindow(GetDlgItem(hDlg, (IDC_OPT_KEEPSCORE as i32)), (nScoreMode == (SCORE_VEGAS as i32)) as i32);
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDC_OPT_STANDARD | IDC_OPT_VEGAS | IDC_OPT_NOSCORE => {
                    EnableWindow(GetDlgItem(hDlg, (IDC_OPT_KEEPSCORE as i32)),
                                 (IsDlgButtonChecked(hDlg, (IDC_OPT_VEGAS as i32)) == BST_CHECKED) as i32);
                    return (TRUE) as INT_PTR;
                }

                IDOK => {
                    let nOldMode: INT = nScoreMode;

                    nDrawCards = if IsDlgButtonChecked(hDlg, (IDC_OPT_DRAWTHREE as i32)) == BST_CHECKED { 3 } else { 1 };

                    if IsDlgButtonChecked(hDlg, (IDC_OPT_VEGAS as i32)) == BST_CHECKED {
                        nScoreMode = (SCORE_VEGAS as i32);
                    } else if IsDlgButtonChecked(hDlg, (IDC_OPT_NOSCORE as i32)) == BST_CHECKED {
                        nScoreMode = (SCORE_NONE as i32);
                    } else {
                        nScoreMode = (SCORE_STD as i32);
                    }

                    fKeepScore  = (IsDlgButtonChecked(hDlg, (IDC_OPT_KEEPSCORE as i32)) == BST_CHECKED) as i32;
                    fShowTime   = (IsDlgButtonChecked(hDlg, (IDC_OPT_SHOWTIME as i32)) == BST_CHECKED) as i32;
                    fShowStatus = (IsDlgButtonChecked(hDlg, (IDC_OPT_STATUSBAR as i32)) == BST_CHECKED) as i32;

                    /* Switching scoring systems mid-deal: the running total
                       means something else now, and so does the score the
                       undo module has banked.  Start both over rather than
                       mixing two currencies.  The deal itself is left alone -
                       the player keeps the game they are in. */
                    if nScoreMode != nOldMode {
                        lScore = if nScoreMode == (SCORE_VEGAS as i32) { -(SCORE_VEGAS_STAKE as i32) } else { 0 };
                        ClearUndo();
                    }

                    SaveSettings();
                    ApplyStatusBarVisibility();
                    UpdateStatusBar();

                    EndDialog(hDlg, (TRUE) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, (FALSE) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn CardImageWndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pBack: *mut CARDBACK = GetWindowLongPtr(hwnd, GWLP_USERDATA) as *mut CARDBACK;

    if pBack.is_null() {
        return DefWindowProc(hwnd, uMsg, wParam, lParam);
    }

    if uMsg == WM_PAINT {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let mut rc: RECT = core::mem::zeroed();
        let hdc: HDC;
        let hPen: HPEN;
        let hOldPen: HPEN;
        let hOldBrush: HBRUSH;

        hdc = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));
        GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));

        hPen = CreatePen((PS_SOLID as i32), 2,
                         if (*pBack).bSelected != 0 {
                             RGB!(0, 0, 0)
                         } else {
                             GetSysColor((COLOR_3DFACE) as i32) as COLORREF
                         });
        hOldPen = SelectObject(hdc, hPen as _) as HPEN;
        hOldBrush = SelectObject(hdc, GetStockObject((NULL_BRUSH as i32)) as _) as HBRUSH;

        Rectangle(hdc, rc.left + 1, rc.top + 1, rc.right, rc.bottom);

        StretchBlt(hdc, 3, 3, rc.right - 6, rc.bottom - 6,
                   CardLib_CardBitmapsDC(), (*pBack).nSlot * CardLib_CardWidth(), 0,
                   CardLib_CardWidth(), CardLib_CardHeight(), SRCCOPY);

        SelectObject(hdc, hOldBrush as _);
        SelectObject(hdc, hOldPen as _);
        DeleteObject((hPen).cast());

        EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
        return 0;
    }

    return CallWindowProc((*pBack).pfnOldProc, hwnd, uMsg, wParam, lParam);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn CardImageWndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pBack: *mut CARDBACK = GetWindowLongPtr(hwnd, GWLP_USERDATA) as *mut CARDBACK;

    if pBack.is_null() {
        return DefWindowProc(hwnd, uMsg, wParam, lParam);
    }

    if uMsg == WM_PAINT {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let mut rc: RECT = core::mem::zeroed();
        let hdc: HDC;
        let hPen: HPEN;
        let hOldPen: HPEN;
        let hOldBrush: HBRUSH;

        hdc = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));
        GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));

        hPen = CreatePen((PS_SOLID as i32), 2,
                         if (*pBack).bSelected != 0 {
                             RGB!(0, 0, 0)
                         } else {
                             GetSysColor((COLOR_3DFACE) as i32) as COLORREF
                         });
        hOldPen = SelectObject(hdc, hPen as _) as HPEN;
        hOldBrush = SelectObject(hdc, GetStockObject((NULL_BRUSH as i32)) as _) as HBRUSH;

        Rectangle(hdc, rc.left + 1, rc.top + 1, rc.right, rc.bottom);

        StretchBlt(hdc, 3, 3, rc.right - 6, rc.bottom - 6,
                   CardLib_CardBitmapsDC(), (*pBack).nSlot * CardLib_CardWidth(), 0,
                   CardLib_CardWidth(), CardLib_CardHeight(), SRCCOPY);

        SelectObject(hdc, hOldBrush as _);
        SelectObject(hdc, hOldPen as _);
        DeleteObject((hPen).cast());

        EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
        return 0;
    }

    return CallWindowProc((*pBack).pfnOldProc, hwnd, uMsg, wParam, lParam);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn CardBackDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut i: INT;

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                i = 0;
                while i < (NUM_CARDBACKS as i32) {
                    CardBacks[i as usize].hSelf = GetDlgItem(hDlg, (CARDBACK_START + (i as u32)) as i32);
                    CardBacks[i as usize].nSlot = (CARDBACK_RES_START + (i as u32)) as i32;
                    CardBacks[i as usize].bSelected = (CARDBACK_RES_START + (i as u32) == (nCardBack as u32)) as BOOL;

                    /* Hang the state off the control BEFORE subclassing it, so the
                       new proc can never run against a control that has none. */
                    SetWindowLongPtr(CardBacks[i as usize].hSelf, GWLP_USERDATA, core::ptr::addr_of_mut!(CardBacks[i as usize]) as LONG_PTR);
                    CardBacks[i as usize].pfnOldProc = core::mem::transmute::<LONG_PTR, WNDPROC>(
                        SetWindowLongPtr(CardBacks[i as usize].hSelf,
                                               GWLP_WNDPROC,
                                               (CardImageWndProc as usize as LONG_PTR)));
                    i += 1;
                }
                return (TRUE) as INT_PTR;
            }

            WM_COMMAND => {
                let nId: INT = LOWORD!(wParam) as INT;

                /* IDOK first: STN_CLICKED and BN_CLICKED are both 0, so testing
                   the notification code before the control id would swallow the
                   buttons. */
                if (nId) as u32 == IDOK {
                    let mut nChosen: INT = nCardBack;

                    i = 0;
                    while i < (NUM_CARDBACKS as i32) {
                        if CardBacks[i as usize].bSelected != 0 {
                            nChosen = CardBacks[i as usize].nSlot;
                        }
                        i += 1;
                    }

                    EndDialog(hDlg, nChosen);
                    return (TRUE) as INT_PTR;
                }

                if (nId) as u32 == IDCANCEL {
                    EndDialog(hDlg, 0);
                    return (TRUE) as INT_PTR;
                }

                if nId >= (CARDBACK_START as i32) && nId <= (CARDBACK_END as i32) && (HIWORD!(wParam) as INT) as u32 == STN_CLICKED {
                    i = 0;
                    while i < (NUM_CARDBACKS as i32) {
                        CardBacks[i as usize].bSelected = (CardBacks[i as usize].hSelf == lParam as HWND) as BOOL;
                        InvalidateRect(CardBacks[i as usize].hSelf, core::ptr::null_mut(), (TRUE) as BOOL);
                        i += 1;
                    }
                    return (TRUE) as INT_PTR;
                }
                break 'sw;
            }

            _ => {}
        }
    }

    return (FALSE) as INT_PTR;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn CardBackDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut i: INT;

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                i = 0;
                while i < (NUM_CARDBACKS as i32) {
                    CardBacks[i as usize].hSelf = GetDlgItem(hDlg, (CARDBACK_START + (i) as u32) as i32);
                    CardBacks[i as usize].nSlot = (CARDBACK_RES_START + (i) as u32) as i32;
                    CardBacks[i as usize].bSelected = (CARDBACK_RES_START + (i) as u32 == (nCardBack) as u32) as BOOL;

                    /* Hang the state off the control BEFORE subclassing it, so the
                       new proc can never run against a control that has none. */
                    SetWindowLongPtr(CardBacks[i as usize].hSelf, GWLP_USERDATA, core::ptr::addr_of_mut!(CardBacks[i as usize]) as LONG_PTR);
                    CardBacks[i as usize].pfnOldProc = core::mem::transmute::<LONG_PTR, WNDPROC>(
                        SetWindowLongPtr(CardBacks[i as usize].hSelf,
                                               GWLP_WNDPROC,
                                               (CardImageWndProc as usize as LONG_PTR)));
                    i += 1;
                }
                return (TRUE) as INT_PTR;
            }

            WM_COMMAND => {
                let nId: INT = LOWORD!(wParam) as INT;

                /* IDOK first: STN_CLICKED and BN_CLICKED are both 0, so testing
                   the notification code before the control id would swallow the
                   buttons. */
                if (nId) as u32 == IDOK {
                    let mut nChosen: INT = nCardBack;

                    i = 0;
                    while i < (NUM_CARDBACKS as i32) {
                        if CardBacks[i as usize].bSelected != 0 {
                            nChosen = CardBacks[i as usize].nSlot;
                        }
                        i += 1;
                    }

                    EndDialog(hDlg, (nChosen) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                if (nId) as u32 == IDCANCEL {
                    EndDialog(hDlg, 0);
                    return (TRUE) as INT_PTR;
                }

                if nId >= (CARDBACK_START as i32) && nId <= (CARDBACK_END as i32) && (HIWORD!(wParam) as INT) as u32 == STN_CLICKED {
                    i = 0;
                    while i < (NUM_CARDBACKS as i32) {
                        CardBacks[i as usize].bSelected = (CardBacks[i as usize].hSelf == lParam as HWND) as BOOL;
                        InvalidateRect(CardBacks[i as usize].hSelf, core::ptr::null_mut(), (TRUE) as BOOL);
                        i += 1;
                    }
                    return (TRUE) as INT_PTR;
                }
                break 'sw;
            }

            _ => {}
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "C" fn ShowDeckOptionsDlg(hwnd: HWND) {
    let nSlot: INT_PTR = DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_CARDBACK), hwnd, Some(CardBackDlgProc));

    if (nSlot as u32) >= CARDBACK_RES_START && (nSlot as u32) < CARDBACK_RES_START + NUM_CARDBACKS {
        nCardBack = nSlot as i32;
        CardWindow_SetBackCardIdx(SolWnd, nCardBack as u32);
        CardWindow_Redraw(SolWnd);
        SaveSettings();
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn WndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            /* Everything below needs this, and WM_CREATE runs before
               CreateWindow has returned it to WinMain. */
            hwndMain = hwnd;

            /* No HTML Help to point it at. */
            EnableMenuItem(GetMenu(hwnd), IDM_HELP_CONTENTS, MF_BYCOMMAND | MF_GRAYED);

            hwndStatus = CreateWindowEx(0, STATUSCLASSNAME, core::ptr::null_mut(),
                                        WS_CHILD | SBARS_SIZEGRIP,
                                        0, 0, 0, 0,
                                        hwnd, (IDC_STATUSBAR as UINT_PTR) as HMENU, hInstance, core::ptr::null_mut());
            if !hwndStatus.is_null() && fShowStatus != 0 {
                ShowWindow(hwndStatus, (SW_SHOW) as i32);
            }

            SolWnd = CardWindow_Create(hwnd, 0, WS_CHILD | WS_VISIBLE, 0, 0, 100, 100);
            if SolWnd.is_null() {
                return -(1 as LRESULT);
            }

            CardWindow_SetBackCardIdx(SolWnd, nCardBack as UINT);

            CreateSol();
            SetUndoMenuState((FALSE) as BOOL);
            NewGame();

            /* A zero-sized resize, which WM_GETMINMAXINFO clamps up to a
               table that actually fits seven columns. */
            SetWindowPos(hwnd, core::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOZORDER);
            return 0 as LRESULT;
        }

        WM_SIZE => {
            LayoutChildren();
            return 0 as LRESULT;
        }

        WM_GETMINMAXINFO => {
            let mmi: *mut MINMAXINFO = lParam as *mut MINMAXINFO;

            (*mmi).ptMinTrackSize.x = ((NUM_ROW_STACKS as i32) * (CardLib_CardWidth() as i32)
                                  + ((NUM_ROW_STACKS + 1) as i32) * (X_BORDER as i32)
                                  + 2 * GetSystemMetrics((SM_CXFRAME) as i32)) as i32;

            /* Room for the top row, a tableau column of thirteen fanned
               cards, and the chrome around them. */
            (*mmi).ptMinTrackSize.y = GetSystemMetrics((SM_CYCAPTION) as i32)
                                  + GetSystemMetrics((SM_CYMENU) as i32)
                                  + 2 * GetSystemMetrics((SM_CYFRAME) as i32)
                                  + (3 as i32) * (Y_BORDER as i32)
                                  + (2 as i32) * (CardLib_CardHeight() as i32)
                                  + 12 * yRowStackCardOffset
                                  + StatusBarHeight();
            return 0 as LRESULT;
        }

        WM_TIMER => {
            if wParam == (IDT_PLAYTIMER as WPARAM) {
                dwElapsed += 1;
                UpdateStatusBar();
            }
            return 0 as LRESULT;
        }

        WM_SOL_GAMEWON => {
            GameFinished();
            return 0 as LRESULT;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDM_GAME_NEW => {
                    NewGame();
                    return 0 as LRESULT;
                }

                IDM_GAME_UNDO => {
                    Undo();
                    /* Undo rebuilds the waste behind our back, and knows
                       nothing about the fan's centring or the stock's empty
                       image.  Rebuilding the view from the restored
                       activepile reproduces exactly what Undo just put there
                       and fixes up both. */
                    UpdatePileDisplay(VisiblePileCards);
                    CardWindow_Redraw(SolWnd);
                    return 0 as LRESULT;
                }

                IDM_GAME_DECK => {
                    ShowDeckOptionsDlg(hwnd);
                    return 0 as LRESULT;
                }

                IDM_GAME_OPTIONS => {
                    DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_OPTIONS), hwnd, Some(OptionsDlgProc));
                    return 0 as LRESULT;
                }

                IDM_GAME_EXIT => {
                    PostMessage(hwnd, WM_CLOSE, 0, 0);
                    return 0 as LRESULT;
                }

                IDM_HELP_CONTENTS => {
                    /* Greyed out above; there is no HTML Help to show. */
                    return 0 as LRESULT;
                }

                IDM_HELP_ABOUT => {
                    ShellAbout(hwnd, addr_of_mut!(szAppName) as *mut TCHAR, addr_of_mut!(MsgAbout) as *mut TCHAR,
                               LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SOLITAIRE)));
                    return 0 as LRESULT;
                }

                _ => {}
            }
            return 0 as LRESULT;
        }

        WM_CLOSE => {
            if fGameStarted == 0 ||
                (MessageBox(hwnd, addr_of_mut!(MsgQuit) as *mut TCHAR, addr_of_mut!(szAppName) as *mut TCHAR, MB_YESNO | MB_ICONQUESTION)) as u32 == IDYES
            {
                SaveSettings();
                DestroyWindow(hwnd);
            }
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            StopPlayTimer();
            if !SolWnd.is_null() {
                CardWindow_Destroy(SolWnd);
                SolWnd = core::ptr::null_mut();
            }
            PostQuitMessage(0);
            return 0 as LRESULT;
        }

        _ => {}
    }

    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn WndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            /* Everything below needs this, and WM_CREATE runs before
               CreateWindow has returned it to WinMain. */
            hwndMain = hwnd;

            /* No HTML Help to point it at. */
            EnableMenuItem(GetMenu(hwnd), IDM_HELP_CONTENTS, MF_BYCOMMAND | MF_GRAYED);

            hwndStatus = CreateWindowEx(0, STATUSCLASSNAME, core::ptr::null_mut(),
                                        WS_CHILD | SBARS_SIZEGRIP,
                                        0, 0, 0, 0,
                                        hwnd, (IDC_STATUSBAR as UINT_PTR) as HMENU, hInstance, core::ptr::null_mut());
            if !hwndStatus.is_null() && fShowStatus != 0 {
                ShowWindow(hwndStatus, (SW_SHOW) as i32);
            }

            SolWnd = CardWindow_Create(hwnd, 0, WS_CHILD | WS_VISIBLE, 0, 0, 100, 100);
            if SolWnd.is_null() {
                return -(1 as LRESULT);
            }

            CardWindow_SetBackCardIdx(SolWnd, nCardBack as UINT);

            CreateSol();
            SetUndoMenuState((FALSE) as BOOL);
            NewGame();

            /* A zero-sized resize, which WM_GETMINMAXINFO clamps up to a
               table that actually fits seven columns. */
            SetWindowPos(hwnd, core::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOZORDER);
            return 0 as LRESULT;
        }

        WM_SIZE => {
            LayoutChildren();
            return 0 as LRESULT;
        }

        WM_GETMINMAXINFO => {
            let mmi: *mut MINMAXINFO = lParam as *mut MINMAXINFO;

            (*mmi).ptMinTrackSize.x = ((NUM_ROW_STACKS as i32) * (CardLib_CardWidth() as i32)
                                  + ((NUM_ROW_STACKS + 1) as i32) * (X_BORDER as i32)
                                  + 2 * GetSystemMetrics((SM_CXFRAME) as i32)) as i32;

            /* Room for the top row, a tableau column of thirteen fanned
               cards, and the chrome around them. */
            (*mmi).ptMinTrackSize.y = GetSystemMetrics((SM_CYCAPTION) as i32)
                                  + GetSystemMetrics((SM_CYMENU) as i32)
                                  + 2 * GetSystemMetrics((SM_CYFRAME) as i32)
                                  + (3 as i32) * (Y_BORDER as i32)
                                  + (2 as i32) * (CardLib_CardHeight() as i32)
                                  + 12 * yRowStackCardOffset
                                  + StatusBarHeight();
            return 0 as LRESULT;
        }

        WM_TIMER => {
            if wParam == (IDT_PLAYTIMER as WPARAM) {
                dwElapsed += 1;
                UpdateStatusBar();
            }
            return 0 as LRESULT;
        }

        WM_SOL_GAMEWON => {
            GameFinished();
            return 0 as LRESULT;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDM_GAME_NEW => {
                    NewGame();
                    return 0 as LRESULT;
                }

                IDM_GAME_UNDO => {
                    Undo();
                    /* Undo rebuilds the waste behind our back, and knows
                       nothing about the fan's centring or the stock's empty
                       image.  Rebuilding the view from the restored
                       activepile reproduces exactly what Undo just put there
                       and fixes up both. */
                    UpdatePileDisplay(VisiblePileCards);
                    CardWindow_Redraw(SolWnd);
                    return 0 as LRESULT;
                }

                IDM_GAME_DECK => {
                    ShowDeckOptionsDlg(hwnd);
                    return 0 as LRESULT;
                }

                IDM_GAME_OPTIONS => {
                    DialogBox!(hInstance, MAKEINTRESOURCE!(IDD_OPTIONS), hwnd, Some(OptionsDlgProc));
                    return 0 as LRESULT;
                }

                IDM_GAME_EXIT => {
                    PostMessage(hwnd, WM_CLOSE, 0, 0);
                    return 0 as LRESULT;
                }

                IDM_HELP_CONTENTS => {
                    /* Greyed out above; there is no HTML Help to show. */
                    return 0 as LRESULT;
                }

                IDM_HELP_ABOUT => {
                    ShellAbout(hwnd, addr_of_mut!(szAppName) as *mut TCHAR, addr_of_mut!(MsgAbout) as *mut TCHAR,
                               LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SOLITAIRE)));
                    return 0 as LRESULT;
                }

                _ => {}
            }
            return 0 as LRESULT;
        }

        WM_CLOSE => {
            if fGameStarted == 0 ||
                (MessageBox(hwnd, addr_of_mut!(MsgQuit) as *mut TCHAR, addr_of_mut!(szAppName) as *mut TCHAR, MB_YESNO | MB_ICONQUESTION)) as u32 == IDYES
            {
                SaveSettings();
                DestroyWindow(hwnd);
            }
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            StopPlayTimer();
            if !SolWnd.is_null() {
                CardWindow_Destroy(SolWnd);
                SolWnd = core::ptr::null_mut();
            }
            PostQuitMessage(0);
            return 0 as LRESULT;
        }

        _ => {}
    }

    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInst: HINSTANCE, hPrev: HINSTANCE, szCmdLine: LPTSTR, iCmdShow: i32) -> i32 {
    let mut hwnd: HWND = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut wndclass: WNDCLASSW = core::mem::zeroed();
    let mut ice: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut hAccelTable: HACCEL = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(hPrev);
    UNREFERENCED_PARAMETER!(szCmdLine);

    hInstance = hInst;

    /* The title doubles as the window class name, so it has to exist. */
    LoadString(hInst, IDS_SOL_NAME as _, core::ptr::addr_of_mut!(szAppName) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(szAppName)))  / core::mem::size_of_val(&(*core::ptr::addr_of!(szAppName))[0 as usize])) as i32);
    if (*core::ptr::addr_of!(szAppName))[0 as usize] == ('\0' as u16) {
        lstrcpy(core::ptr::addr_of_mut!(szAppName) as LPTSTR, (&[0x0053, 0x006f, 0x006c, 0x0069, 0x0074, 0x0061, 0x0069, 0x0072, 0x0065, 0x0000u16]).as_ptr() /* _T "Solitaire" */);
    }

    /* Loaded once here rather than every time a message box goes up. */
    LoadString(hInst, IDS_SOL_ABOUT as _, core::ptr::addr_of_mut!(MsgAbout) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgAbout))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgAbout))[0 as usize])) as i32);
    LoadString(hInst, IDS_SOL_QUIT as _, core::ptr::addr_of_mut!(MsgQuit) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgQuit))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgQuit))[0 as usize])) as i32);
    LoadString(hInst, IDS_SOL_WIN as _, core::ptr::addr_of_mut!(MsgWin) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgWin))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgWin))[0 as usize])) as i32);
    LoadString(hInst, IDS_SOL_DEAL as _, core::ptr::addr_of_mut!(MsgDeal) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgDeal))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgDeal))[0 as usize])) as i32);
    LoadString(hInst, IDS_SOL_SCORE as _, core::ptr::addr_of_mut!(MsgScore) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgScore))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgScore))[0 as usize])) as i32);
    LoadString(hInst, IDS_SOL_TIME as _, core::ptr::addr_of_mut!(MsgTime) as LPTSTR, (core::mem::size_of_val(&(*core::ptr::addr_of!(MsgTime))) / core::mem::size_of_val(&(*core::ptr::addr_of!(MsgTime))[0 as usize])) as i32);

    LoadSettings();

    ice.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as u32;
    ice.dwICC = ICC_BAR_CLASSES as _;
    InitCommonControlsEx(core::ptr::addr_of_mut!(ice));

    /* Nothing below here can draw a card without the artwork, so a failed
       load is fatal rather than an empty green table. */
    if CardLib_Init() == 0 {
        MessageBox(core::ptr::null_mut(), (&[0x0046, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x0020, 0x0074, 0x006f, 0x0020, 0x006c, 0x006f, 0x0061, 0x0064, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0063, 0x0061, 0x0072, 0x0064, 0x0020, 0x0069, 0x006d, 0x0061, 0x0067, 0x0065, 0x0073, 0x0020, 0x0066, 0x0072, 0x006f, 0x006d, 0x0020, 0x0063, 0x0061, 0x0072, 0x0064, 0x0073, 0x002e, 0x0064, 0x006c, 0x006c, 0x002e, 0x0000u16]).as_ptr() /* _T "Failed to load the card images from cards.dll." */,
                   core::ptr::addr_of_mut!(szAppName) as LPTSTR, (MB_OK | MB_ICONERROR) as _);
        return 1 as i32;
    }

    wndclass.style         = 0 as _;
    wndclass.lpfnWndProc   = Some(WndProc);
    wndclass.cbClsExtra    = 0 as _;
    wndclass.cbWndExtra    = 0 as _;
    wndclass.hInstance     = hInst;
    wndclass.hIcon         = LoadIcon(hInst, MAKEINTRESOURCE!(IDI_SOLITAIRE as usize) as _);
    wndclass.hCursor       = LoadCursor(core::ptr::null_mut(), IDC_ARROW as usize as _);
    wndclass.hbrBackground = ((COLOR_3DFACE + 1) as usize) as HBRUSH;
    wndclass.lpszMenuName  = MAKEINTRESOURCE!(IDR_MENU1 as usize) as _;
    wndclass.lpszClassName = core::ptr::addr_of_mut!(szAppName) as _;

    if RegisterClass(core::ptr::addr_of_mut!(wndclass)) == 0 {
        CardLib_Term();
        return 1 as i32;
    }

    hwnd = CreateWindow!(core::ptr::addr_of_mut!(szAppName) as LPTSTR,
                        core::ptr::addr_of_mut!(szAppName) as LPTSTR,
                        (WS_OVERLAPPEDWINDOW) as DWORD,
                        CW_USEDEFAULT as c_int,
                        CW_USEDEFAULT as c_int,
                        0 as c_int,  /* the real size comes from WM_GETMINMAXINFO */
                        0 as c_int,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        hInst,
                        core::ptr::null_mut());
    if hwnd.is_null() {
        CardLib_Term();
        return 1 as i32;
    }

    ShowWindow(hwnd, iCmdShow);
    UpdateWindow(hwnd);

    hAccelTable = LoadAccelerators(hInst, MAKEINTRESOURCE!(IDR_ACCELERATOR1 as usize) as _);

    while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0 as _, 0 as _) != 0 {
        if TranslateAccelerator(hwnd, hAccelTable, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    CardLib_Term();

    return msg.wParam as i32;
}
