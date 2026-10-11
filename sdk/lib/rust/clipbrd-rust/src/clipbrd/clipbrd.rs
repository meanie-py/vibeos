use crate::clipbrd::winutils::BitBltFromClipboard;
use crate::clipbrd::winutils::BringWindowToFront;
use crate::clipbrd::cliputils::DeleteClipboardContent;
use crate::clipbrd::winutils::DrawTextFromClipboard;
use crate::clipbrd::winutils::DrawTextFromResource;
use crate::clipbrd::cliputils::GetAutomaticClipboardFormat;
use crate::clipbrd::cliputils::GetClipboardDataDimensions;
use crate::clipbrd::winutils::HDropFromClipboard;
use crate::clipbrd::cliputils::IsClipboardFormatSupported;
use crate::clipbrd::winutils::MessageBoxRes;
use crate::clipbrd::scrollutils::OnKeyScroll;
use crate::clipbrd::scrollutils::OnMouseScroll;
use crate::clipbrd::scrollutils::OnScroll;
use crate::clipbrd::winutils::PlayEnhMetaFileFromClipboard;
use crate::clipbrd::winutils::PlayMetaFileFromClipboard;
use crate::clipbrd::fileutils::ReadClipboardFile;
use crate::clipbrd::winutils::RealizeClipboardPalette;
use crate::clipbrd::cliputils::RetrieveClipboardFormatName;
use crate::clipbrd::cliputils::SendClipboardOwnerMessage;
use crate::clipbrd::winutils::SetDIBitsToDeviceFromClipboard;
use crate::clipbrd::winutils::ShowLastWin32Error;
use crate::clipbrd::scrollutils::UpdateLinesToScroll;
use crate::clipbrd::scrollutils::UpdateWindowScrollState;
use crate::clipbrd::fileutils::WriteClipboardFile;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0AB4BE745DBE: [u16; 4] = [0x0063u16, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_3D9B1397D845: [u16; 6] = [0x002Au16, 0x002E, 0x0063, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_96F36D1FBB6F: [u16; 12] = [
    0x0063u16, 0x006C, 0x0069, 0x0070, 0x0062, 0x0072, 0x0064, 0x002E, 0x0063, 0x0068,
    0x006D, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szClassName: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"C\0l\0i\0p\0B\0o\0o\0k\0W\0C\0l\0a\0s\0s\0\0\0",
    )
};

#[no_mangle]
pub static mut Globals: CLIPBOARD_GLOBALS = _CLIPBOARD_GLOBALS {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hMainWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hWndNext: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    uDisplayFormat: 0,
    uCheckedItem: 0,
    CharWidth: 0,
    CharHeight: 0,
};

#[no_mangle]
pub static mut Scrollstate: SCROLLSTATE = _SCROLLSTATE {
    uLinesToScroll: 0,
    iWheelCarryoverX: 0,
    iWheelCarryoverY: 0,
    nPageX: 0,
    nPageY: 0,
    CurrentX: 0,
    CurrentY: 0,
    MaxX: 0,
    MaxY: 0,
    nMaxWidth: 0,
    nMaxHeight: 0,
};

unsafe extern "C" fn SaveClipboardToFile() {
    let mut sfn: OPENFILENAMEW = core::mem::zeroed();
    let mut c: LPWSTR = core::ptr::null_mut();
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFilterMask: [WCHAR; (MAX_STRING_LEN + 10) as _] =
        [0; (MAX_STRING_LEN + 10) as _];

    ZeroMemory!(
        core::ptr::addr_of_mut!(szFilterMask) as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_STRING_LEN + 10) as _]>(),
    );
    c = (core::ptr::addr_of_mut!(szFilterMask) as *mut WCHAR)
        .offset(LoadStringW(
            Globals.hInstance,
            STRING_FORMAT_NT,
            core::ptr::addr_of_mut!(szFilterMask) as LPWSTR,
            (MAX_STRING_LEN) as INT,
        ) as isize)
        .offset(1 as isize) as LPWSTR;
    wcscpy(c, RUSTCONV_WSTR_3D9B1397D845.as_ptr());

    ZeroMemory!(
        core::ptr::addr_of_mut!(szFileName) as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(sfn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );
    sfn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    sfn.hwndOwner = Globals.hMainWnd;
    sfn.hInstance = Globals.hInstance;
    sfn.lpstrFilter = core::ptr::addr_of_mut!(szFilterMask) as *mut WCHAR as LPCWSTR;
    sfn.lpstrFile = core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPWSTR;
    sfn.nMaxFile = ARRAYSIZE!(szFileName) as DWORD;
    sfn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_OVERWRITEPROMPT) as DWORD;
    sfn.lpstrDefExt = RUSTCONV_WSTR_0AB4BE745DBE.as_ptr();

    if GetSaveFileNameW(core::ptr::addr_of_mut!(sfn)) == 0 {
        return;
    }

    if OpenClipboard(Globals.hMainWnd) == 0 {
        ShowLastWin32Error(Globals.hMainWnd);
        return;
    }

    WriteClipboardFile(
        core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPCWSTR,
        CLIP_FMT_NT as WORD,
    );

    CloseClipboard();
}

unsafe extern "C" fn LoadClipboardDataFromFile(mut lpszFileName: LPWSTR) {
    if (MessageBoxRes(
        Globals.hMainWnd,
        Globals.hInstance,
        STRING_DELETE_MSG as UINT,
        STRING_DELETE_TITLE as UINT,
        (MB_ICONWARNING | MB_YESNO) as UINT,
    )) as u32 != IDYES
    {
        return;
    }

    if OpenClipboard(Globals.hMainWnd) == 0 {
        ShowLastWin32Error(Globals.hMainWnd);
        return;
    }

    EmptyClipboard();
    ReadClipboardFile(lpszFileName as LPCWSTR);

    CloseClipboard();
}

unsafe extern "C" fn LoadClipboardFromFile() {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut c: LPWSTR;
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFilterMask: [WCHAR; (MAX_STRING_LEN + 10) as _] =
        [0; (MAX_STRING_LEN + 10) as _];

    ZeroMemory!(
        core::ptr::addr_of_mut!(szFilterMask) as *mut ::core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_STRING_LEN + 10) as _]>(),
    );
    c = (core::ptr::addr_of_mut!(szFilterMask) as *mut WCHAR)
        .offset(LoadStringW(
            Globals.hInstance,
            STRING_FORMAT_GEN,
            core::ptr::addr_of_mut!(szFilterMask) as LPWSTR,
            (MAX_STRING_LEN) as INT,
        ) as isize)
        .offset(1 as isize) as LPWSTR;
    wcscpy(c as *mut WCHAR, RUSTCONV_WSTR_3D9B1397D845.as_ptr() as *const WCHAR);

    ZeroMemory!(
        core::ptr::addr_of_mut!(szFileName) as *mut ::core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(ofn) as *mut ::core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );
    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.hwndOwner = Globals.hMainWnd;
    ofn.hInstance = Globals.hInstance;
    ofn.lpstrFilter = core::ptr::addr_of_mut!(szFilterMask) as *mut WCHAR as *const WCHAR;
    ofn.lpstrFile = core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPWSTR;
    ofn.nMaxFile = ARRAYSIZE!(szFileName) as DWORD;
    ofn.Flags = (OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_FILEMUSTEXIST) as DWORD;

    if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn)) == 0 {
        return;
    }

    LoadClipboardDataFromFile(core::ptr::addr_of_mut!(szFileName) as LPWSTR);
}

unsafe extern "C" fn LoadClipboardFromDrop(hDrop: HDROP) {
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    DragQueryFileW(
        hDrop,
        0,
        core::ptr::addr_of_mut!(szFileName) as LPWSTR,
        ARRAYSIZE!(szFileName) as UINT,
    );
    DragFinish(hDrop);

    LoadClipboardDataFromFile(core::ptr::addr_of_mut!(szFileName) as LPWSTR);
}

unsafe extern "C" fn SetDisplayFormat(uFormat: UINT) {
    let mut rc: RECT = core::mem::zeroed();

    CheckMenuItem(Globals.hMenu, Globals.uCheckedItem, MF_BYCOMMAND | MF_UNCHECKED);
    Globals.uCheckedItem = uFormat + CMD_AUTOMATIC;
    CheckMenuItem(Globals.hMenu, Globals.uCheckedItem, MF_BYCOMMAND | MF_CHECKED);

    if uFormat == 0 {
        Globals.uDisplayFormat = GetAutomaticClipboardFormat();
    } else {
        Globals.uDisplayFormat = uFormat;
    }

    GetClipboardDataDimensions(Globals.uDisplayFormat, &raw mut rc);
    Scrollstate.CurrentY = 0;
    Scrollstate.CurrentX = Scrollstate.CurrentY;
    Scrollstate.iWheelCarryoverY = 0;
    Scrollstate.iWheelCarryoverX = Scrollstate.iWheelCarryoverY;
    UpdateWindowScrollState(Globals.hMainWnd, rc.right as INT, rc.bottom as INT, &raw mut Scrollstate);

    InvalidateRect(Globals.hMainWnd, core::ptr::null_mut(), (TRUE) as BOOL);
}

unsafe extern "C" fn InitMenuPopup(hMenu: HMENU, index: LPARAM) {
    if (GetMenuItemID(hMenu, 0) == CMD_DELETE) || (GetMenuItemID(hMenu, 1) == CMD_SAVE_AS) {
        if CountClipboardFormats() == 0 {
            EnableMenuItem(hMenu, CMD_DELETE, MF_GRAYED);
            EnableMenuItem(hMenu, CMD_SAVE_AS, MF_GRAYED);
        } else {
            EnableMenuItem(hMenu, CMD_DELETE, MF_ENABLED);
            EnableMenuItem(hMenu, CMD_SAVE_AS, MF_ENABLED);
        }
    }

    DrawMenuBar(Globals.hMainWnd);
}

unsafe extern "C" fn UpdateDisplayMenu() {
    let mut uFormat: UINT = 0;
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut szFormatName: [WCHAR; (MAX_FMT_NAME_LEN + 1) as usize] =
        [0; (MAX_FMT_NAME_LEN + 1) as usize];

    hMenu = GetSubMenu(Globals.hMenu, (DISPLAY_MENU_POS as i32));

    while GetMenuItemCount(hMenu) > 1 {
        DeleteMenu(hMenu, 1, MF_BYPOSITION);
    }

    if CountClipboardFormats() == 0 {
        return;
    }

    if OpenClipboard(Globals.hMainWnd) == 0 {
        return;
    }

    AppendMenuW(hMenu, MF_SEPARATOR, 0, core::ptr::null_mut());

    /* Display the supported clipboard formats first */
    uFormat = EnumClipboardFormats(0);
    while uFormat != 0 {
        if IsClipboardFormatSupported(uFormat) != 0 {
            RetrieveClipboardFormatName(Globals.hInstance, uFormat, (TRUE) as BOOL,
                                        (szFormatName.as_mut_ptr()).cast(), ARRAYSIZE!(szFormatName));
            AppendMenuW(hMenu, MF_STRING, (CMD_AUTOMATIC + uFormat) as UINT_PTR, szFormatName.as_mut_ptr());
        }
        uFormat = EnumClipboardFormats(uFormat);
    }

    /* Now display the unsupported clipboard formats */
    uFormat = EnumClipboardFormats(0);
    while uFormat != 0 {
        if IsClipboardFormatSupported(uFormat) == 0 {
            RetrieveClipboardFormatName(Globals.hInstance, uFormat, (TRUE) as BOOL,
                                        (szFormatName.as_mut_ptr()).cast(), ARRAYSIZE!(szFormatName));
            AppendMenuW(hMenu, MF_STRING | MF_GRAYED, 0, szFormatName.as_mut_ptr());
        }
        uFormat = EnumClipboardFormats(uFormat);
    }

    CloseClipboard();
}

unsafe extern "C" fn OnCommand(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> core::ffi::c_int {
    match (LOWORD!(wParam)) as u32 {
        CMD_OPEN => {
            LoadClipboardFromFile();
        }
        CMD_SAVE_AS => {
            SaveClipboardToFile();
        }
        CMD_EXIT => {
            PostMessageW(Globals.hMainWnd, WM_CLOSE, 0, 0);
        }
        CMD_DELETE => {
            if (MessageBoxRes(Globals.hMainWnd, Globals.hInstance,
                STRING_DELETE_MSG, STRING_DELETE_TITLE,
                MB_ICONWARNING | MB_YESNO)) as u32 == IDYES
            {
                DeleteClipboardContent();
            }
        }
        CMD_AUTOMATIC => {
            SetDisplayFormat(0);
        }
        CMD_HELP => {
            HtmlHelpW(Globals.hMainWnd, RUSTCONV_WSTR_96F36D1FBB6F.as_ptr(), 0, 0);
        }
        CMD_ABOUT => {
            let mut szTitle: [WCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];

            LoadStringW(Globals.hInstance, STRING_CLIPBOARD, core::ptr::addr_of_mut!(szTitle) as LPWSTR, (ARRAYSIZE!(szTitle)) as INT);
            ShellAboutW(Globals.hMainWnd, core::ptr::addr_of_mut!(szTitle) as LPWSTR, core::ptr::null_mut(),
                LoadIconW(Globals.hInstance, MAKEINTRESOURCEW!(CLIPBRD_ICON)));
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn OnPaint(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) {
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut ps: PAINTSTRUCT = tagPAINTSTRUCT {
        hdc: ::core::ptr::null_mut::<HDC__>(),
        fErase: 0,
        rcPaint: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        fRestore: 0,
        fIncUpdate: 0,
        rgbReserved: [0; 32],
    };
    let mut crOldBkColor: COLORREF = 0;
    let mut crOldTextColor: COLORREF = 0;
    let mut rc: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if OpenClipboard(Globals.hMainWnd) == 0 {
        return;
    }
    hdc = BeginPaint(hWnd, &raw mut ps);
    if ps.fErase != 0 {
        FillRect(
            ps.hdc,
            &raw mut ps.rcPaint,
            ((COLOR_WINDOW + 1) as HBRUSH),
        );
    }
    crOldBkColor = SetBkColor(
        ps.hdc,
        GetSysColor((COLOR_WINDOW) as i32) as COLORREF,
    );
    crOldTextColor = SetTextColor(
        ps.hdc,
        GetSysColor((COLOR_WINDOWTEXT) as i32) as COLORREF,
    );
    RealizeClipboardPalette(ps.hdc);
    match Globals.uDisplayFormat {
        CF_NONE => {}
        CF_DSPTEXT | CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT => {
            DrawTextFromClipboard(Globals.uDisplayFormat, ps, Scrollstate);
        }
        CF_DSPBITMAP | CF_BITMAP => {
            BitBltFromClipboard(ps, Scrollstate, SRCCOPY);
        }
        CF_DIB | CF_DIBV5 => {
            SetDIBitsToDeviceFromClipboard(Globals.uDisplayFormat, ps, Scrollstate, DIB_RGB_COLORS);
        }
        CF_DSPMETAFILEPICT | CF_METAFILEPICT => {
            GetClientRect(hWnd, &raw mut rc);
            PlayMetaFileFromClipboard(hdc, &raw mut rc);
        }
        CF_DSPENHMETAFILE | CF_ENHMETAFILE => {
            GetClientRect(hWnd, &raw mut rc);
            PlayEnhMetaFileFromClipboard(hdc, &raw mut rc);
        }
        CF_OWNERDISPLAY => {
            let mut hglb: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
            let mut pps: PPAINTSTRUCT = ::core::ptr::null_mut::<tagPAINTSTRUCT>();
            hglb = GlobalAlloc(
                GMEM_MOVEABLE as UINT,
                ::core::mem::size_of::<PAINTSTRUCT>() as SIZE_T,
            );
            if !hglb.is_null() {
                pps = GlobalLock(hglb) as PPAINTSTRUCT;
                CopyMemory!(
                    pps as *mut ::core::ffi::c_void,
                    &raw mut ps as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<PAINTSTRUCT>() as size_t,
                );
                GlobalUnlock(hglb);
                SendClipboardOwnerMessage(
                    (TRUE as UINT) as BOOL,
                    (WM_PAINTCLIPBOARD as WPARAM) as UINT,
                    hWnd as WPARAM,
                    hglb as LPARAM,
                );
                GlobalFree(hglb);
            }
        }
        CF_HDROP => {
            GetClientRect(hWnd, &raw mut rc);
            HDropFromClipboard(hdc, &raw mut rc);
        }
        _ => {
            GetClientRect(hWnd, &raw mut rc);
            DrawTextFromResource(
                Globals.hInstance,
                ERROR_UNSUPPORTED_FORMAT as UINT,
                hdc,
                &raw mut rc,
                (DT_CENTER | DT_WORDBREAK | DT_NOPREFIX) as UINT,
            );
        }
    }
    SetTextColor(ps.hdc, crOldTextColor);
    SetBkColor(ps.hdc, crOldBkColor);
    EndPaint(hWnd, &raw mut ps);
    CloseClipboard();
}
unsafe extern "system" fn MainWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut palette: bool = false;
    'sw: {
        match uMsg {
            WM_CREATE => {
                let mut tm: TEXTMETRICW = core::mem::zeroed();
                let mut hDC: HDC = GetDC(hWnd);

                /*
                 * Note that the method with GetObjectW just returns
                 * the original parameters with which the font was created.
                 */
                if GetTextMetricsW(hDC, core::ptr::addr_of_mut!(tm)) != 0 {
                    Globals.CharWidth = tm.tmMaxCharWidth; // tm.tmAveCharWidth;
                    Globals.CharHeight = tm.tmHeight + tm.tmExternalLeading;
                }
                ReleaseDC(hWnd, hDC);

                Globals.hMenu = GetMenu(hWnd);
                Globals.hWndNext = SetClipboardViewer(hWnd);

                // For now, the Help dialog item is disabled because of lacking of HTML support
                EnableMenuItem(Globals.hMenu, CMD_HELP, MF_BYCOMMAND | MF_GRAYED);

                UpdateLinesToScroll(core::ptr::addr_of_mut!(Scrollstate));

                UpdateDisplayMenu();
                SetDisplayFormat(0);

                DragAcceptFiles(hWnd, (TRUE) as BOOL);
            }

            WM_CLOSE => {
                DestroyWindow(hWnd);
            }

            WM_DESTROY => {
                ChangeClipboardChain(hWnd, Globals.hWndNext);

                if Globals.uDisplayFormat == CF_OWNERDISPLAY {
                    let mut hglb: HGLOBAL = core::ptr::null_mut();
                    let mut prc: PRECT = core::ptr::null_mut();

                    hglb = GlobalAlloc(GMEM_MOVEABLE, (core::mem::size_of::<RECT>()) as SIZE_T);
                    if !hglb.is_null() {
                        prc = GlobalLock(hglb) as PRECT;
                        SetRectEmpty(prc);
                        GlobalUnlock(hglb);

                        SendClipboardOwnerMessage((TRUE) as BOOL, WM_SIZECLIPBOARD,
                                                  hWnd as WPARAM, hglb as LPARAM);

                        GlobalFree(hglb);
                    }
                }

                PostQuitMessage(0);
            }

            WM_PAINT => {
                OnPaint(hWnd, wParam, lParam);
            }

            WM_KEYDOWN => {
                OnKeyScroll(hWnd, wParam, lParam, core::ptr::addr_of_mut!(Scrollstate));
            }

            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                OnMouseScroll(hWnd, uMsg, wParam, lParam, core::ptr::addr_of_mut!(Scrollstate));
            }

            WM_HSCROLL => {
                // NOTE: Windows uses an offset of 16 pixels
                OnScroll(hWnd, (SB_HORZ) as INT, wParam, 5, core::ptr::addr_of_mut!(Scrollstate));
            }

            WM_VSCROLL => {
                // NOTE: Windows uses an offset of 16 pixels
                OnScroll(hWnd, (SB_VERT) as INT, wParam, 5, core::ptr::addr_of_mut!(Scrollstate));
            }

            WM_SIZE => {
                let mut rc: RECT = core::mem::zeroed();

                if Globals.uDisplayFormat == CF_OWNERDISPLAY {
                    let mut hglb: HGLOBAL = core::ptr::null_mut();
                    let mut prc: PRECT = core::ptr::null_mut();

                    hglb = GlobalAlloc(GMEM_MOVEABLE, (core::mem::size_of::<RECT>()) as SIZE_T);
                    if !hglb.is_null() {
                        prc = GlobalLock(hglb) as PRECT;
                        if wParam == (SIZE_MINIMIZED) as _ {
                            SetRectEmpty(prc);
                        } else {
                            GetClientRect(hWnd, prc);
                        }
                        GlobalUnlock(hglb);

                        SendClipboardOwnerMessage((TRUE) as BOOL, WM_SIZECLIPBOARD,
                                                  hWnd as WPARAM, hglb as LPARAM);

                        GlobalFree(hglb);
                    }
                    break 'sw;
                }

                GetClipboardDataDimensions(Globals.uDisplayFormat, core::ptr::addr_of_mut!(rc));
                UpdateWindowScrollState(hWnd, rc.right, rc.bottom, core::ptr::addr_of_mut!(Scrollstate));

                // NOTE: There still are little problems drawing
                // the background when displaying clipboard text.
                if IsClipboardFormatSupported(Globals.uDisplayFormat) == 0 ||
                    Globals.uDisplayFormat == CF_DSPTEXT ||
                    Globals.uDisplayFormat == CF_TEXT ||
                    Globals.uDisplayFormat == CF_OEMTEXT ||
                    Globals.uDisplayFormat == CF_UNICODETEXT
                {
                    InvalidateRect(Globals.hMainWnd, core::ptr::null::<RECT>(), (TRUE) as BOOL);
                } else {
                    InvalidateRect(Globals.hMainWnd, core::ptr::null::<RECT>(), (FALSE) as BOOL);
                }
            }

            WM_CHANGECBCHAIN => {
                /* Transmit through the clipboard viewer chain */
                if (wParam as HWND) == Globals.hWndNext {
                    Globals.hWndNext = lParam as HWND;
                } else if !Globals.hWndNext.is_null() {
                    SendMessageW(Globals.hWndNext, uMsg, wParam, lParam);
                }
            }

            WM_DESTROYCLIPBOARD => {}

            WM_RENDERALLFORMATS => {
                /*
                 * When the user has cleared the clipboard via the DELETE command,
                 * we (clipboard viewer) become the clipboard owner. When we are
                 * subsequently closed, this message is then sent to us so that
                 * we get a chance to render everything we can. Since we don't have
                 * anything to render, just empty the clipboard.
                 */
                DeleteClipboardContent();
            }

            WM_RENDERFORMAT => {
                // TODO!
            }

            WM_DRAWCLIPBOARD => {
                UpdateDisplayMenu();
                SetDisplayFormat(0);

                /* Pass the message to the next window in clipboard viewer chain */
                SendMessageW(Globals.hWndNext, uMsg, wParam, lParam);
            }

            WM_COMMAND => {
                if LOWORD!(wParam) > (CMD_AUTOMATIC as u16) {
                    SetDisplayFormat((LOWORD!(wParam) - (CMD_AUTOMATIC as u16)) as UINT);
                } else {
                    OnCommand(hWnd, uMsg, wParam, lParam);
                }
            }

            WM_INITMENUPOPUP => {
                InitMenuPopup(wParam as HMENU, lParam);
            }

            WM_DROPFILES => {
                LoadClipboardFromDrop(wParam as HDROP);
            }

            WM_PALETTECHANGED => {
                /* Ignore if this comes from ourselves */
                if (wParam as HWND) == hWnd {
                    // break;
                } else {
                    /* Fall back to WM_QUERYNEWPALETTE */
                    palette = true;
                }
            }

            WM_QUERYNEWPALETTE => {
                palette = true;
            }

            WM_SYSCOLORCHANGE => {
                SetDisplayFormat(Globals.uDisplayFormat);
            }

            WM_SETTINGCHANGE => {
                if wParam == (SPI_SETWHEELSCROLLLINES) as _ {
                    UpdateLinesToScroll(core::ptr::addr_of_mut!(Scrollstate));
                }
            }

            _ => {
                return DefWindowProc(hWnd, uMsg, wParam, lParam);
            }
        }
    }

    if palette {
        let mut Success: BOOL = 0;
        let mut hDC: HDC;

        if OpenClipboard(Globals.hMainWnd) == 0 {
            return FALSE as LRESULT;
        }

        hDC = GetDC(hWnd);
        if hDC.is_null() {
            CloseClipboard();
            return FALSE as LRESULT;
        }

        Success = RealizeClipboardPalette(hDC);

        ReleaseDC(hWnd, hDC);
        CloseClipboard();

        if Success != 0 {
            InvalidateRect(hWnd, core::ptr::null::<RECT>(), (TRUE) as BOOL);
            UpdateWindow(hWnd);
            return TRUE as LRESULT;
        }
        return FALSE as LRESULT;
    }

    return 0 as LRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPWSTR, nCmdShow: i32) -> i32 {
    let mut msg: MSG = core::mem::zeroed();
    let mut hAccel: HACCEL = core::ptr::null_mut();
    let mut hPrevWindow: HWND;
    let mut wndclass: WNDCLASSEXW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];

    hPrevWindow = FindWindowW(core::ptr::addr_of!(szClassName) as LPCWSTR, core::ptr::null_mut());
    if !hPrevWindow.is_null() {
        BringWindowToFront(hPrevWindow);
        return 0;
    }

    let __sw = GetUserDefaultUILanguage();
    if __sw == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) {
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    ZeroMemory!(core::ptr::addr_of_mut!(Globals) as *mut core::ffi::c_void, core::mem::size_of::<CLIPBOARD_GLOBALS>());
    Globals.hInstance = hInstance;

    ZeroMemory!(core::ptr::addr_of_mut!(wndclass) as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSEXW>());
    wndclass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wndclass.lpfnWndProc = Some(MainWndProc);
    wndclass.hInstance = hInstance;
    wndclass.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW!(CLIPBRD_ICON));
    wndclass.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
    wndclass.hbrBackground = (COLOR_WINDOW + 1) as usize as HBRUSH;
    wndclass.lpszMenuName = MAKEINTRESOURCEW!(MAIN_MENU);
    wndclass.lpszClassName = core::ptr::addr_of!(szClassName) as LPCWSTR;

    if RegisterClassExW(core::ptr::addr_of_mut!(wndclass)) == 0 {
        ShowLastWin32Error(core::ptr::null_mut());
        return 0;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(Scrollstate) as *mut core::ffi::c_void, core::mem::size_of::<SCROLLSTATE>());

    LoadStringW(hInstance, STRING_CLIPBOARD, szBuffer.as_mut_ptr(), (ARRAYSIZE!(szBuffer)) as INT);
    Globals.hMainWnd = CreateWindowExW(
        WS_EX_CLIENTEDGE | WS_EX_ACCEPTFILES,
        core::ptr::addr_of!(szClassName) as LPCWSTR,
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
        WS_OVERLAPPEDWINDOW | WS_HSCROLL | WS_VSCROLL,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Globals.hInstance,
        core::ptr::null_mut(),
    );
    if Globals.hMainWnd.is_null() {
        ShowLastWin32Error(core::ptr::null_mut());
        return 0;
    }

    ShowWindow(Globals.hMainWnd, nCmdShow);
    UpdateWindow(Globals.hMainWnd);

    hAccel = LoadAcceleratorsW(Globals.hInstance, MAKEINTRESOURCEW!(ID_ACCEL));
    if hAccel.is_null() {
        ShowLastWin32Error(Globals.hMainWnd);
    }

    /* If the user provided a path to a clipboard data file, try to open it */
    if (*__p___argc()) >= 2 {
        LoadClipboardDataFromFile(*(*__p___wargv()).offset(1 as isize));
    }

    while GetMessageW(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        if TranslateAcceleratorW(Globals.hMainWnd, hAccel, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    return msg.wParam as i32;
}
