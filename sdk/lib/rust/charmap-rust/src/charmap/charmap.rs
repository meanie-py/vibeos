use crate::charmap::settings::LoadSettings;
use crate::charmap::map::RegisterMapClasses;
use crate::charmap::settings::SaveSettings;
use crate::charmap::about::ShowAboutDlg;
use crate::charmap::map::UnregisterMapClasses;
use crate::charmap::settings::hWnd;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_444BC7989119: [u16; 13] = [
    0x0052u16, 0x0049, 0x0043, 0x0048, 0x0045, 0x0044, 0x0032, 0x0030, 0x002E, 0x0044,
    0x004C, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_5C5E7EF322F0: [u16; 11] = [
    0x0055u16, 0x002B, 0x0025, 0x0030, 0x0034, 0x0058, 0x003A, 0x0020, 0x0025, 0x0073,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut codePages: [UINT; 31] = [
    864 as ::core::ffi::c_int as UINT,
    775 as ::core::ffi::c_int as UINT,
    863 as ::core::ffi::c_int as UINT,
    855 as ::core::ffi::c_int as UINT,
    737 as ::core::ffi::c_int as UINT,
    856 as ::core::ffi::c_int as UINT,
    862 as ::core::ffi::c_int as UINT,
    861 as ::core::ffi::c_int as UINT,
    852 as ::core::ffi::c_int as UINT,
    869 as ::core::ffi::c_int as UINT,
    850 as ::core::ffi::c_int as UINT,
    858 as ::core::ffi::c_int as UINT,
    865 as ::core::ffi::c_int as UINT,
    860 as ::core::ffi::c_int as UINT,
    866 as ::core::ffi::c_int as UINT,
    857 as ::core::ffi::c_int as UINT,
    437 as ::core::ffi::c_int as UINT,
    1256 as ::core::ffi::c_int as UINT,
    1257 as ::core::ffi::c_int as UINT,
    1250 as ::core::ffi::c_int as UINT,
    1251 as ::core::ffi::c_int as UINT,
    1253 as ::core::ffi::c_int as UINT,
    1255 as ::core::ffi::c_int as UINT,
    932 as ::core::ffi::c_int as UINT,
    949 as ::core::ffi::c_int as UINT,
    1252 as ::core::ffi::c_int as UINT,
    936 as ::core::ffi::c_int as UINT,
    874 as ::core::ffi::c_int as UINT,
    950 as ::core::ffi::c_int as UINT,
    1254 as ::core::ffi::c_int as UINT,
    1258 as ::core::ffi::c_int as UINT,
];

#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hAdvancedDlg: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hCharmapDlg: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hStatusWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hSmIcon: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

#[no_mangle]
pub static mut hBgIcon: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

#[no_mangle]
pub static mut Settings: SETTINGS = SETTINGS { IsAdvancedView: 0 };

pub const ID_ABOUT: i32 = 0x1;

const REMOVE_ADVANCED: bool = false;
unsafe extern "C" fn FillCharacterSetComboList(hwndCombo: HWND) {
    let mut szCharSetText: [WCHAR; 256] = [0; 256];
    let mut trimmedName: LPWSTR = core::ptr::null_mut();
    let mut cpInfo: CPINFOEXW = core::mem::zeroed();
    let mut i: INT;

    if LoadStringW(
        hInstance,
        IDS_UNICODE as UINT,
        core::ptr::addr_of_mut!(szCharSetText) as LPWSTR,
        SIZEOF!(szCharSetText) as ::core::ffi::c_int,
    ) != 0
    {
        SendMessageW(
            hwndCombo,
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szCharSetText) as *mut WCHAR as LPARAM,
        );
    }

    i = 0 as INT;
    while (i as usize) < SIZEOF!(codePages) as usize {
        if GetCPInfoExW(
            codePages[i as usize],
            0 as DWORD,
            core::ptr::addr_of_mut!(cpInfo),
        ) != 0
        {
            trimmedName = wcschr(
                core::ptr::addr_of_mut!(cpInfo.CodePageName) as *mut WCHAR,
                0x0028u16,
            ) as LPWSTR;
            if trimmedName.is_null() {
                trimmedName = core::ptr::addr_of_mut!(cpInfo.CodePageName) as *mut WCHAR as LPWSTR;
            }

            SendMessageW(
                hwndCombo,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                trimmedName as LPARAM,
            );
        }
        i += 1;
    }

    SendMessageW(
        hwndCombo,
        CB_SETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn FillGroupByComboList(hwndCombo: HWND) {
    let mut szAllText: [WCHAR; 256] = [0; 256];

    if LoadStringW(hInstance, IDS_ALL as UINT, core::ptr::addr_of_mut!(szAllText) as LPWSTR, SIZEOF!(szAllText) as ::core::ffi::c_int) != 0
    {
        SendMessageW(hwndCombo, CB_ADDSTRING as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(szAllText) as *mut WCHAR as LPARAM);
    }

    SendMessageW(hwndCombo, CB_SETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM);
}
unsafe extern "system" fn EnumFontNames(
    lpelfe: *mut ENUMLOGFONTEXW,
    lpntme: *mut NEWTEXTMETRICEXW,
    FontType: DWORD,
    lParam: LPARAM,
) -> ::core::ffi::c_int {
    let mut hwndCombo: HWND = lParam as HWND;
    let mut pszName: LPWSTR = core::ptr::addr_of_mut!((*lpelfe).elfLogFont.lfFaceName) as LPWSTR;

    /* Skip rotated font */
    if *pszName.add(0 as usize) == 0x0040u16 {
        return 1 as ::core::ffi::c_int;
    }

    /* make sure font doesn't already exist in our list */
    if SendMessageW(
        hwndCombo,
        CB_FINDSTRINGEXACT as UINT,
        0 as WPARAM,
        pszName as LPARAM,
    ) == (CB_ERR as LRESULT)
    {
        let mut idx: INT = 0;
        let mut fFixed: BOOL = 0;
        let mut fTrueType: BOOL = 0;

        /* add the font */
        idx = SendMessageW(
            hwndCombo,
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            pszName as LPARAM,
        ) as INT;

        /* record the font's attributes (Fixedwidth and Truetype) */
        fFixed = (if ((*lpelfe).elfLogFont.lfPitchAndFamily as ::core::ffi::c_int
            & (FIXED_PITCH as ::core::ffi::c_int))
            != 0
        {
            TRUE
        } else {
            FALSE
        }) as BOOL;
        fTrueType = (if (*lpelfe).elfLogFont.lfOutPrecision as ::core::ffi::c_int
            == (OUT_STROKE_PRECIS as ::core::ffi::c_int)
        {
            TRUE
        } else {
            FALSE
        }) as BOOL;

        /* store this information in the list-item's userdata area */
        SendMessageW(
            hwndCombo,
            CB_SETITEMDATA as UINT,
            idx as WPARAM,
            MAKEWPARAM!(fFixed as WORD, fTrueType as WORD) as LPARAM,
        );
    }

    return 1 as ::core::ffi::c_int;
}

unsafe extern "C" fn FillFontStyleComboList(hwndCombo: HWND) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut lf: LOGFONTW = core::mem::zeroed();

    /* FIXME: for fun, draw each font in its own style */
    let hFont: HFONT = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
    SendMessageW(hwndCombo,
                 WM_SETFONT,
                 hFont as WPARAM,
                 0);

    ZeroMemory!(core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
               core::mem::size_of::<LOGFONTW>());
    lf.lfCharSet = DEFAULT_CHARSET as u8;

    hdc = GetDC(hwndCombo);

    /* store the list of fonts in the combo */
    EnumFontFamiliesExW(hdc,
                        core::ptr::addr_of_mut!(lf),
                        core::mem::transmute::<
                            Option<
                                unsafe extern "system" fn(
                                    *mut ENUMLOGFONTEXW,
                                    *mut NEWTEXTMETRICEXW,
                                    DWORD,
                                    LPARAM,
                                ) -> core::ffi::c_int,
                            >,
                            FONTENUMPROCW,
                        >(Some(
                            EnumFontNames,
                        )),
                        hwndCombo as LPARAM,
                        0 as DWORD);

    ReleaseDC(hwndCombo,
              hdc);

    SendMessageW(hwndCombo,
                 CB_SETCURSEL as UINT,
                 0 as WPARAM,
                 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn ChangeMapFont(hDlg: HWND) {
    let mut hCombo: HWND = core::ptr::null_mut();
    let mut hMap: HWND = core::ptr::null_mut();
    let mut lpFontName: LPWSTR = core::ptr::null_mut();
    let mut Len: INT = 0;

    hCombo = GetDlgItem(hDlg, IDC_FONTCOMBO as INT);

    Len = GetWindowTextLengthW(hCombo);

    if Len != 0 as INT {
        lpFontName = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((Len + 1 as INT) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;

        if !lpFontName.is_null() {
            SendMessageW(
                hCombo,
                WM_GETTEXT as UINT,
                (Len + 1 as INT) as WPARAM,
                lpFontName as LPARAM,
            );

            hMap = GetDlgItem(hDlg, IDC_FONTMAP as INT);

            SendMessageW(
                hMap,
                FM_SETFONT as UINT,
                0 as WPARAM,
                lpFontName as LPARAM,
            );
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            lpFontName as PVOID,
        );
    }
}

unsafe extern "C" fn CopyCharacters(hDlg: HWND) {
    let hText: HWND = GetDlgItem(hDlg, IDC_TEXTBOX as i32);
    let mut dwStart: DWORD = 0;
    let mut dwEnd: DWORD = 0;

    // Acquire selection limits
    SendMessage(
        hText,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwStart) as WPARAM,
        core::ptr::addr_of_mut!(dwEnd) as LPARAM,
    );

    // Test if the whose text is unselected
    if dwStart == dwEnd {

        // Select the whole text
        SendMessageW(
            hText,
            EM_SETSEL as UINT,
            0 as WPARAM,
            (-(1 as isize)) as LPARAM,
        );

        // Copy text
        SendMessageW(
            hText,
            WM_COPY as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );

        // Restore previous values
        SendMessageW(
            hText,
            EM_SETSEL as UINT,
            dwStart as WPARAM,
            dwEnd as LPARAM,
        );

    } else {

        // Copy text
        SendMessageW(
            hText,
            WM_COPY as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn GetFontMetrics(hWndArg: HWND, hFont: HFONT) -> BYTE {
    unsafe {
        let mut tmFont: TEXTMETRIC = core::mem::zeroed();
        let mut hOldObj: HGDIOBJ = core::ptr::null_mut();
        let mut hDC: HDC = core::ptr::null_mut();

        hDC = GetDC(hWndArg);
        hOldObj = SelectObject(hDC, (hFont).cast());
        GetTextMetrics(hDC, core::ptr::addr_of_mut!(tmFont));
        SelectObject(hDC, hOldObj);
        ReleaseDC(hWndArg, hDC);

        return tmFont.tmCharSet;
    }
}

unsafe extern "C" fn AddCharToSelection(hDlg: HWND, mut ch: WCHAR) {
    let mut hMap: HWND = GetDlgItem(hDlg, (IDC_FONTMAP as i32));
    let mut hText: HWND = GetDlgItem(hDlg, (IDC_TEXTBOX as i32));
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut lFont: LOGFONT = core::mem::zeroed();
    let mut cf: CHARFORMAT = core::mem::zeroed();

    // Retrieve current character selected
    if ch as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        ch = SendMessageW(hMap, FM_GETCHAR as UINT, 0 as WPARAM, 0 as LPARAM) as WCHAR;
        if ch == 0 {
            return;
        }
    }

    // Retrieve current selected font
    hFont = SendMessage(hMap, FM_GETHFONT as UINT, 0 as WPARAM, 0 as LPARAM) as HFONT;

    // Recover LOGFONT structure from hFont
    if GetObject(hFont as HANDLE, core::mem::size_of::<LOGFONT>() as ::core::ffi::c_int, core::ptr::addr_of_mut!(lFont) as LPVOID) == 0 {
        return;
    }

    // Recover font properties of Richedit control
    ZeroMemory!(core::ptr::addr_of_mut!(cf) as *mut ::core::ffi::c_void, core::mem::size_of::<CHARFORMAT>());
    cf.cbSize = core::mem::size_of::<CHARFORMAT>() as UINT;
    SendMessage(hText, EM_GETCHARFORMAT as UINT, SCF_SELECTION as WPARAM, core::ptr::addr_of_mut!(cf) as LPARAM);

    // Apply properties of the new font
    cf.bCharSet = GetFontMetrics(hText, hFont);

    // Update font name
    wcscpy(core::ptr::addr_of_mut!(cf.szFaceName) as *mut WCHAR, core::ptr::addr_of_mut!(lFont.lfFaceName) as *mut WCHAR);

    // Update font properties
    SendMessage(hText, EM_SETCHARFORMAT as UINT, SCF_SELECTION as WPARAM, core::ptr::addr_of_mut!(cf) as LPARAM);

    // Send selected character to Richedit
    SendMessage(hText, WM_CHAR as UINT, ch as WPARAM, 0 as LPARAM);
}

unsafe extern "C" fn UpdateSettings(hDlg: HWND) {
    if hDlg == hCharmapDlg {
        Settings.IsAdvancedView = SendDlgItemMessage(
            hDlg,
            (IDC_CHECK_ADVANCED as i32),
            BM_GETCHECK,
            0,
            0,
        ) as BOOL;
    }

    if hDlg == hAdvancedDlg {
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateStatusBar(wch: WCHAR) {
    let mut buff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDesc: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    GetUName(wch, core::ptr::addr_of_mut!(szDesc) as *mut WCHAR);
    wsprintfW(
        core::ptr::addr_of_mut!(buff) as *mut WCHAR,
        RUSTCONV_WSTR_5C5E7EF322F0.as_ptr(),
        wch as core::ffi::c_uint,
        core::ptr::addr_of_mut!(szDesc) as *mut WCHAR,
    );
    SendMessageW(
        hStatusWnd,
        (SB_SETTEXT) as UINT,
        0,
        core::ptr::addr_of_mut!(buff) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn ChangeView(hwnd: HWND) {
    let mut rcCharmap: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcAdvanced: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcCopy: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcPanelExt: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcPanelInt: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcStatus: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut DeX: UINT = 0;
    let mut DeY: UINT = 0;
    let mut xPos: LONG = 0;
    let mut yPos: LONG = 0;
    let mut Width: UINT = 0;
    let mut Height: UINT = 0;
    let mut DeskTopWidth: UINT = 0;
    let mut DeskTopHeight: UINT = 0;
    let mut hCopy: HWND = core::ptr::null_mut();

    GetClientRect(hCharmapDlg, &raw mut rcCharmap);
    if !REMOVE_ADVANCED {
        GetClientRect(hAdvancedDlg, &raw mut rcAdvanced);
    } else {
        hCopy = GetDlgItem(hCharmapDlg, (IDC_COPY as i32));
        GetClientRect(hCopy, &raw mut rcCopy);
    }
    GetWindowRect(hwnd, &raw mut rcPanelExt);
    GetClientRect(hwnd, &raw mut rcPanelInt);
    GetClientRect(hStatusWnd, &raw mut rcStatus);

    DeskTopWidth = GetSystemMetrics((SM_CXFULLSCREEN) as i32) as UINT;
    DeskTopHeight = GetSystemMetrics((SM_CYFULLSCREEN) as i32) as UINT;

    DeX = (rcPanelExt.right.wrapping_sub(rcPanelExt.left)).wrapping_sub(rcPanelInt.right) as UINT;
    DeY = (rcPanelExt.bottom.wrapping_sub(rcPanelExt.top)).wrapping_sub(rcPanelInt.bottom) as UINT;

    MoveWindow(
        hCharmapDlg,
        0,
        0,
        rcCharmap.right,
        rcCharmap.bottom,
        (FALSE) as BOOL,
    );
    if !REMOVE_ADVANCED {
        MoveWindow(
            hAdvancedDlg,
            0,
            rcCharmap.bottom,
            rcAdvanced.right,
            rcAdvanced.bottom,
            (FALSE) as BOOL,
        );
        ShowWindow(
            hAdvancedDlg,
            if Settings.IsAdvancedView != 0 {
                (SW_SHOW) as i32
            } else {
                (SW_HIDE) as i32
            },
        );
    }
    xPos = rcPanelExt.left;
    yPos = rcPanelExt.top;

    Width = DeX.wrapping_add(rcCharmap.right as UINT);
    Height = DeY
        .wrapping_add(rcCharmap.bottom as UINT)
        .wrapping_add(rcStatus.bottom as UINT);
    if !REMOVE_ADVANCED {
        if Settings.IsAdvancedView != 0 {
            Height = Height.wrapping_add(rcAdvanced.bottom as UINT);
        }
    } else {
        Height = (rcCharmap.bottom as UINT)
            .wrapping_add(rcCopy.bottom as UINT)
            .wrapping_add(10);
    }
    // FIXME: This fails on multi monitor setups
    if (xPos as UINT).wrapping_add(Width) > DeskTopWidth {
        xPos = DeskTopWidth.wrapping_sub(Width) as LONG;
    }

    if (yPos as UINT).wrapping_add(Height) > DeskTopHeight {
        yPos = DeskTopHeight.wrapping_sub(Height) as LONG;
    }

    MoveWindow(
        hwnd,
        xPos,
        yPos,
        Width as i32,
        Height as i32,
        (TRUE) as BOOL,
    );
}

unsafe extern "system" fn CharMapDlgProc(
    mut hDlg: HWND,
    mut Message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            let mut evMask: DWORD;
            let hAdv: HWND;

            FillFontStyleComboList(GetDlgItem(hDlg, (IDC_FONTCOMBO as i32)));

            ChangeMapFont(hDlg);

            // Configure Richedit control for sending notification changes.
            evMask = SendDlgItemMessage(hDlg, (IDC_TEXTBOX as i32), EM_GETEVENTMASK, 0, 0) as DWORD;
            evMask |= ENM_CHANGE as DWORD;
            SendDlgItemMessage(hDlg, (IDC_TEXTBOX as i32), EM_SETEVENTMASK, 0, evMask as LPARAM);

            // REMOVE_ADVANCED branch (compiled out; kept as dead code)
            if false {
                hAdv = GetDlgItem(hDlg, (IDC_CHECK_ADVANCED as i32));
                ShowWindow(hAdv, (SW_HIDE) as i32);
            }

            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (((wParam & 0xffff) as WORD) as i32) as u32 {
                IDC_FONTMAP => {
                    match ((((wParam >> 16) & 0xffff) as WORD) as i32) as u32 {
                        FM_SETCHAR => {
                            AddCharToSelection(hDlg, (lParam & 0xffff) as WCHAR);
                        }
                        _ => {}
                    }
                }

                IDC_FONTCOMBO => {
                    if ((((wParam >> 16) & 0xffff) as WORD) as i32) as u32 == CBN_SELCHANGE {
                        ChangeMapFont(hDlg);
                    }
                }

                IDC_SELECT => {
                    AddCharToSelection(hDlg, 0);
                }

                IDC_TEXTBOX => {
                    match ((((wParam >> 16) & 0xffff) as WORD) as i32) as u32 {
                        EN_CHANGE => {
                            if GetWindowTextLength(GetDlgItem(hDlg, (IDC_TEXTBOX as i32))) == 0 {
                                EnableWindow(GetDlgItem(hDlg, (IDC_COPY as i32)), (FALSE) as BOOL);
                            } else {
                                EnableWindow(GetDlgItem(hDlg, (IDC_COPY as i32)), (TRUE) as BOOL);
                            }
                        }
                        _ => {}
                    }
                }

                IDC_COPY => {
                    CopyCharacters(hDlg);
                }

                IDC_CHECK_ADVANCED => {
                    UpdateSettings(hDlg);
                    ChangeView(GetParent(hDlg));
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
unsafe extern "system" fn AdvancedDlgProc(
    hDlg: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    if Message == WM_INITDIALOG {
        return TRUE as INT_PTR;
    }
    if Message == WM_COMMAND {
        if LOWORD!(wParam) as i32 == IDC_COMBO_CHARSET as i32 {
            if HIWORD!(wParam) as i32 == CBN_SELCHANGE as i32 {
                let idx: INT = SendMessageW(
                    lParam as HWND,
                    CB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as INT;
                SendMessageW(
                    GetDlgItem(hCharmapDlg, (IDC_FONTMAP as i32)),
                    FM_SETCHARMAP as UINT,
                    idx as WPARAM,
                    0 as LPARAM,
                );
                EnableWindow(
                    GetDlgItem(hAdvancedDlg, (IDC_EDIT_UNICODE as i32)),
                    (idx == 0 as INT) as i32,
                );
            }
        }
        // switch falls through to default
        return FALSE as INT_PTR;
    }
    return FALSE as INT_PTR;
    return FALSE as INT_PTR;
}

unsafe extern "C" fn PanelOnCreate(hWndParam: HWND, wParam: WPARAM, lParam: LPARAM) -> i32 {
    let mut hSysMenu: HMENU;
    let mut lpAboutText: [WCHAR; 256] = [0; 256];

    hCharmapDlg = CreateDialog!(
        hInstance,
        MAKEINTRESOURCE!(IDD_CHARMAP),
        hWndParam,
        Some(CharMapDlgProc),
    );

    // For now, the Help push button is disabled because of lacking of HTML Help support
    EnableWindow(GetDlgItem(hCharmapDlg, (IDC_CMHELP as i32)), (FALSE) as BOOL);

    hAdvancedDlg = CreateDialog!(
        hInstance,
        MAKEINTRESOURCE!(IDD_ADVANCED),
        hWndParam,
        Some(AdvancedDlgProc),
    );

    FillCharacterSetComboList(GetDlgItem(hAdvancedDlg, (IDC_COMBO_CHARSET as i32)));

    FillGroupByComboList(GetDlgItem(hAdvancedDlg, (IDC_COMBO_GROUPBY as i32)));
    EnableWindow(GetDlgItem(hAdvancedDlg, (IDC_COMBO_GROUPBY as i32)), (FALSE) as BOOL);
    EnableWindow(GetDlgItem(hAdvancedDlg, (IDC_BUTTON_SEARCH as i32)), (FALSE) as BOOL);

    hStatusWnd = CreateWindow!(
        STATUSCLASSNAME,
        core::ptr::null_mut(),
        WS_CHILD | WS_VISIBLE,
        0,
        0,
        0,
        0,
        hWndParam,
        IDD_STATUSBAR as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );

    // Set the status bar for multiple parts output
    SendMessage(hStatusWnd, SB_SIMPLE, FALSE as WPARAM, 0 as LPARAM);

    ChangeView(hWndParam);

    hSysMenu = GetSystemMenu(hWndParam, (FALSE) as BOOL);

    if hSysMenu != core::ptr::null_mut() {
        if LoadStringW(
            hInstance,
            IDS_ABOUT,
            core::ptr::addr_of_mut!(lpAboutText) as LPWSTR,
            (SIZEOF!(lpAboutText)) as INT,
        ) != 0
        {
            AppendMenuW(hSysMenu, MF_SEPARATOR, 0, core::ptr::null_mut());
            AppendMenuW(
                hSysMenu,
                MF_STRING,
                (ID_ABOUT) as UINT_PTR,
                core::ptr::addr_of_mut!(lpAboutText) as *mut WCHAR as LPCWSTR,
            );
        }
    }

    SetFocus(hCharmapDlg);

    return 0;
}

unsafe extern "system" fn PanelWndProc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            return (PanelOnCreate(hwnd, wParam, lParam)) as LRESULT;
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            return 0;
        }
        WM_SIZE => {
            SendMessage(hStatusWnd, msg, wParam, lParam);
        }
        WM_DESTROY => {
            SaveSettings();
            PostQuitMessage(0);
            return 0;
        }
        WM_SYSCOMMAND => {
            match (wParam) as i32 {
                ID_ABOUT => {
                    ShowAboutDlg(hwnd);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return DefWindowProc(hwnd, msg, wParam, lParam);
}

unsafe extern "C" fn InitInstance(hInst: HINSTANCE) -> HWND {
    let mut szClass: [WCHAR; 8] = [0x0043u16, 0x0068, 0x0061, 0x0072, 0x004D, 0x0061, 0x0070, 0x0000];
    let mut szTitle: [WCHAR; 256] = [0; 256];
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut hWndLocal: HWND = core::ptr::null_mut();

    LoadStringW(hInst, IDS_TITLE, core::ptr::addr_of_mut!(szTitle) as LPWSTR, (SIZEOF!(szTitle)) as INT);

    hSmIcon = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_ICON),
        IMAGE_ICON,
        16,
        16,
        0,
    ) as HICON;

    hBgIcon = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_ICON),
        IMAGE_ICON,
        32,
        32,
        0,
    ) as HICON;

    // Create workspace
    ZeroMemory!(core::ptr::addr_of_mut!(wc) as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSEXW>());

    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.lpfnWndProc = Some(PanelWndProc);
    wc.hInstance = hInst;
    wc.hIcon = hBgIcon;
    wc.hCursor = LoadCursor(core::ptr::null_mut(), MAKEINTRESOURCE!(IDC_ARROW)) as HCURSOR;
    wc.hbrBackground = GetStockObject((NULL_BRUSH as i32)) as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = core::ptr::addr_of_mut!(szClass) as *mut WCHAR as LPCWSTR;
    wc.hIconSm = hSmIcon;

    RegisterClassExW(core::ptr::addr_of_mut!(wc));

    hWndLocal = CreateWindowW!(
        core::ptr::addr_of_mut!(szClass) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX) as DWORD,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInst,
        core::ptr::null_mut(),
    );

    if hWndLocal != core::ptr::null_mut() {
        LoadSettings();
        ShowWindow(hWndLocal, (SW_SHOW) as i32);
        UpdateWindow(hWndLocal);
    }

    return hWndLocal;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrev: HINSTANCE,
    Cmd: LPWSTR,
    iCmd: core::ffi::c_int,
) -> INT {
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut Ret: INT = 1;
    let mut hRichEd20: HMODULE = core::ptr::null_mut();
    let mut Msg: MSG = core::mem::zeroed();

    hInstance = hInst;

    /* Mirroring code for the titlebar */
    let Lang: WORD = GetUserDefaultUILanguage() as WORD;
    if Lang == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) {
        SetProcessDefaultLayout(LAYOUT_RTL as DWORD);
    }

    iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccx.dwICC = ICC_TAB_CLASSES as DWORD;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

    if RegisterMapClasses(hInstance) != 0 {
        hRichEd20 = LoadLibraryW(RUSTCONV_WSTR_444BC7989119.as_ptr());

        if !hRichEd20.is_null() {
            InitInstance(hInst);

            loop {
                if GetMessage(
                    core::ptr::addr_of_mut!(Msg),
                    core::ptr::null_mut(),
                    0 as UINT,
                    0 as UINT,
                ) <= 0
                {
                    Ret = Msg.wParam as INT;
                    break;
                }

                /* NOTE: CreateDialog needs IsDialogMessage call in message loop */
                if !hCharmapDlg.is_null()
                    && IsDialogMessage(hCharmapDlg, core::ptr::addr_of_mut!(Msg)) != 0
                {
                    continue;
                }
                if !hAdvancedDlg.is_null()
                    && IsDialogMessage(hAdvancedDlg, core::ptr::addr_of_mut!(Msg)) != 0
                {
                    continue;
                }

                TranslateMessage(core::ptr::addr_of_mut!(Msg));
                DispatchMessage(core::ptr::addr_of_mut!(Msg));
            }

            FreeLibrary(hRichEd20);
        }
        UnregisterMapClasses(hInstance);
    }

    return Ret;
}
