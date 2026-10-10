use crate::notepad::dialog::DIALOG_EditCopy;
use crate::notepad::dialog::DIALOG_EditCut;
use crate::notepad::dialog::DIALOG_EditDelete;
use crate::notepad::dialog::DIALOG_EditPaste;
use crate::notepad::dialog::DIALOG_EditSelectAll;
use crate::notepad::dialog::DIALOG_EditTimeDate;
use crate::notepad::dialog::DIALOG_EditUndo;
use crate::notepad::dialog::DIALOG_EditWrap;
use crate::notepad::dialog::DIALOG_FileExit;
use crate::notepad::dialog::DIALOG_FileNew;
use crate::notepad::dialog::DIALOG_FileNewWindow;
use crate::notepad::dialog::DIALOG_FileOpen;
use crate::notepad::dialog::DIALOG_FileSave;
use crate::notepad::dialog::DIALOG_FileSaveAs;
use crate::notepad::dialog::DIALOG_GoTo;
use crate::notepad::dialog::DIALOG_HelpAboutNotepad;
use crate::notepad::dialog::DIALOG_HelpContents;
use crate::notepad::dialog::DIALOG_Replace;
use crate::notepad::dialog::DIALOG_Search;
use crate::notepad::dialog::DIALOG_SearchNext;
use crate::notepad::dialog::DIALOG_SelectFont;
use crate::notepad::dialog::DIALOG_StatusBarAlignParts;
use crate::notepad::dialog::DIALOG_StatusBarUpdateCaretPos;
use crate::notepad::dialog::DIALOG_StringMsgBox;
use crate::notepad::dialog::DIALOG_ViewStatusBar;
use crate::notepad::dialog::DoCloseFile;
use crate::notepad::dialog::DoCreateEditWindow;
use crate::notepad::dialog::DoOpenFile;
use crate::notepad::dialog::DoShowHideStatusBar;
use crate::notepad::dialog::FileExists;
use crate::notepad::dialog::HasFileExtension;
use crate::notepad::dialog::ShowLastError;
use crate::notepad::dialog::UpdateWindowCaption;
use crate::notepad::dialog::WaitCursor;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

#[no_mangle]
pub static mut Globals: NOTEPAD_GLOBALS = NOTEPAD_GLOBALS {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hMainWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hFindReplaceDlg: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hEdit: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hStatusBar: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
    hMenu: ::core::ptr::null::<HMENU__>() as *mut HMENU__,
    hDevMode: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    hDevNames: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    lfFont: tagLOGFONTW {
        lfHeight: 0,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: 0,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: 0,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: 0,
        lfPitchAndFamily: 0,
        lfFaceName: [0; 32],
    },
    bWrapLongLines: 0,
    bShowStatusBar: 0,
    szFindText: [0; 260],
    szReplaceText: [0; 260],
    szFileName: [0; 260],
    szFileTitle: [0; 260],
    szFilter: [0; 512],
    lMargins: tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    },
    szHeader: [0; 260],
    szFooter: [0; 260],
    szStatusBarLineCol: [0; 260],
    encFile: ENCODING_ANSI,
    iEoln: EOLN_CRLF,
    find: FINDREPLACE {
        lStructSize: 0,
        hwndOwner: ::core::ptr::null::<HWND__>() as *mut HWND__,
        hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
        Flags: 0,
        lpstrFindWhat: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        lpstrReplaceWith: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        wFindWhatLen: 0,
        wReplaceWithLen: 0,
        lCustData: 0,
        lpfnHook: None,
        lpTemplateName: ::core::ptr::null::<WCHAR>(),
    },
    EditProc: None,
    bWasModified: 0,
};

static mut aFINDMSGSTRING: ATOM = 0;
#[no_mangle]
pub unsafe extern "C" fn NOTEPAD_EnableSearchMenu() {
    let bEmpty: BOOL = (GetWindowTextLengthW(Globals.hEdit) == 0) as BOOL;
    let uEnable: UINT = MF_BYCOMMAND | (if bEmpty != 0 { MF_GRAYED } else { MF_ENABLED }) as UINT;
    EnableMenuItem(Globals.hMenu, CMD_SEARCH as UINT, uEnable);
    EnableMenuItem(Globals.hMenu, CMD_SEARCH_NEXT as UINT, uEnable);
    EnableMenuItem(Globals.hMenu, CMD_SEARCH_PREV as UINT, uEnable);
}
#[no_mangle]
pub unsafe extern "C" fn SetFileName(szFileName: LPCTSTR) {
    StringCchCopy(
        core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR,
        (_countof!(Globals.szFileName)) as _,
        szFileName,
    );
    Globals.szFileTitle[0] = 0;
    GetFileTitle(
        szFileName,
        core::ptr::addr_of_mut!(Globals.szFileTitle) as *mut TCHAR,
        _countof!(Globals.szFileTitle) as WORD,
    );

    if !szFileName.is_null() && *szFileName.offset(0) != 0 {
        SHAddToRecentDocs(SHARD_PATHW as UINT, szFileName as LPCVOID);
    }
}

unsafe extern "C" fn NOTEPAD_MenuCommand(wParam: WPARAM) -> ::core::ffi::c_int {
    match wParam as u32 {
        CMD_NEW => {
            DIALOG_FileNew();
        }
        CMD_NEW_WINDOW => {
            DIALOG_FileNewWindow();
        }
        CMD_OPEN => {
            DIALOG_FileOpen();
        }
        CMD_SAVE => {
            DIALOG_FileSave();
        }
        CMD_SAVE_AS => {
            DIALOG_FileSaveAs();
        }
        CMD_PRINT => {
            DIALOG_FilePrint();
        }
        CMD_PAGE_SETUP => {
            DIALOG_FilePageSetup();
        }
        CMD_EXIT => {
            DIALOG_FileExit();
        }
        CMD_UNDO => {
            DIALOG_EditUndo();
        }
        CMD_CUT => {
            DIALOG_EditCut();
        }
        CMD_COPY => {
            DIALOG_EditCopy();
        }
        CMD_PASTE => {
            DIALOG_EditPaste();
        }
        CMD_DELETE => {
            DIALOG_EditDelete();
        }
        CMD_SELECT_ALL => {
            DIALOG_EditSelectAll();
        }
        CMD_TIME_DATE => {
            DIALOG_EditTimeDate();
        }
        CMD_SEARCH => {
            DIALOG_Search();
        }
        CMD_SEARCH_NEXT => {
            DIALOG_SearchNext((TRUE) as BOOL);
        }
        CMD_REPLACE => {
            DIALOG_Replace();
        }
        CMD_GOTO => {
            DIALOG_GoTo();
        }
        CMD_SEARCH_PREV => {
            DIALOG_SearchNext((FALSE) as BOOL);
        }
        CMD_WRAP => {
            DIALOG_EditWrap();
        }
        CMD_FONT => {
            DIALOG_SelectFont();
        }
        CMD_STATUSBAR => {
            DIALOG_ViewStatusBar();
        }
        CMD_HELP_CONTENTS => {
            DIALOG_HelpContents();
        }
        CMD_HELP_ABOUT_NOTEPAD => {
            DIALOG_HelpAboutNotepad();
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn NOTEPAD_FindTextAt(
    pFindReplace: *mut FINDREPLACE,
    pszText: LPCTSTR,
    iTextLength: INT,
    dwPosition: DWORD,
) -> BOOL {
    let mut bMatches: BOOL;
    let mut iTargetLength: size_t;
    let mut pchPosition: LPCTSTR;

    if pFindReplace.is_null() || pszText.is_null() {
        return (FALSE) as BOOL;
    }

    iTargetLength = _tcslen((*pFindReplace).lpstrFindWhat) as size_t;
    pchPosition = pszText.offset(dwPosition as isize);

    /* Make proper comparison */
    if ((*pFindReplace).Flags & FR_MATCHCASE) != 0 {
        bMatches = (_tcsncmp(pchPosition, (*pFindReplace).lpstrFindWhat, iTargetLength) == 0) as BOOL;
    } else {
        bMatches = (_tcsnicmp(pchPosition, (*pFindReplace).lpstrFindWhat, iTargetLength) == 0) as BOOL;
    }

    if bMatches != 0 && ((*pFindReplace).Flags & FR_WHOLEWORD) != 0 {
        if dwPosition > 0 {
            if _istalnum!(*pchPosition.offset(-(1 as isize))) != 0
                || *pchPosition.offset(-(1 as isize)) == ('_' as u16)
            {
                bMatches = (FALSE) as i32;
            }
        }
        if ((dwPosition as INT as usize).wrapping_add(iTargetLength)) < (iTextLength as usize) {
            if _istalnum!(*pchPosition.add(iTargetLength)) != 0
                || *pchPosition.add(iTargetLength) == ('_' as u16)
            {
                bMatches = (FALSE) as i32;
            }
        }
    }

    return bMatches;
}
#[no_mangle]
pub unsafe extern "C" fn NOTEPAD_FindNext(pFindReplace: *mut FINDREPLACE, bReplace: BOOL, bShowAlert: BOOL) -> BOOL {
    let mut iTextLength: ::core::ffi::c_int = 0;
    let mut iTargetLength: ::core::ffi::c_int = 0;
    let mut iAdjustment: size_t = 0;
    let mut pszText: LPTSTR = core::ptr::null_mut();
    let mut dwPosition: DWORD = 0;
    let mut dwBegin: DWORD = 0;
    let mut dwEnd: DWORD = 0;
    let mut bMatches: BOOL = (FALSE) as i32;
    let mut szResource: [TCHAR; 128] = [0 as TCHAR; 128];
    let mut szText: [TCHAR; 128] = [0 as TCHAR; 128];
    let mut bSuccess: BOOL = 0;

    iTargetLength = _tcslen((*pFindReplace).lpstrFindWhat) as ::core::ffi::c_int;

    /* Retrieve the window text */
    iTextLength = GetWindowTextLength(Globals.hEdit);
    if iTextLength > 0 {
        pszText = HeapAlloc(
            GetProcessHeap(),
            0,
            ((iTextLength + 1) as usize * core::mem::size_of::<TCHAR>()) as SIZE_T,
        ) as LPTSTR;
        if pszText.is_null() {
            return (FALSE) as BOOL;
        }

        GetWindowText(Globals.hEdit, pszText as LPWSTR, iTextLength + 1);
    }

    SendMessage(
        Globals.hEdit,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwBegin) as WPARAM,
        core::ptr::addr_of_mut!(dwEnd) as LPARAM,
    );
    if bReplace != 0 && (dwEnd.wrapping_sub(dwBegin) == iTargetLength as DWORD) {
        if NOTEPAD_FindTextAt(pFindReplace, pszText as LPCTSTR, iTextLength as INT, dwBegin) != 0 {
            SendMessage(
                Globals.hEdit,
                EM_REPLACESEL as UINT,
                TRUE as WPARAM,
                (*pFindReplace).lpstrReplaceWith as LPARAM,
            );
            iAdjustment = (_tcslen((*pFindReplace).lpstrReplaceWith) as size_t)
                .wrapping_sub(dwEnd.wrapping_sub(dwBegin) as size_t);
        }
    }

    if ((*pFindReplace).Flags & FR_DOWN) != 0 {
        /* Find Down */
        dwPosition = dwEnd;
        while dwPosition < iTextLength as DWORD {
            bMatches = NOTEPAD_FindTextAt(pFindReplace, pszText as LPCTSTR, iTextLength as INT, dwPosition);
            if bMatches != 0 {
                break;
            }
            dwPosition = dwPosition.wrapping_add(1);
        }
    } else {
        /* Find Up */
        dwPosition = dwBegin;
        while dwPosition > 0 {
            dwPosition = dwPosition.wrapping_sub(1);
            bMatches = NOTEPAD_FindTextAt(pFindReplace, pszText as LPCTSTR, iTextLength as INT, dwPosition);
            if bMatches != 0 {
                break;
            }
        }
    }

    if bMatches != 0 {
        /* Found target */
        if dwPosition > dwBegin {
            dwPosition = dwPosition.wrapping_add(iAdjustment as DWORD);
        }
        SendMessage(
            Globals.hEdit,
            EM_SETSEL as UINT,
            dwPosition as WPARAM,
            dwPosition.wrapping_add(iTargetLength as DWORD) as LPARAM,
        );
        SendMessage(
            Globals.hEdit,
            EM_SCROLLCARET as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        bSuccess = (TRUE) as i32;
    } else {
        /* Can't find target */
        if bShowAlert != 0 {
            LoadString(
                Globals.hInstance,
                STRING_CANNOTFIND as UINT,
                core::ptr::addr_of_mut!(szResource) as LPWSTR,
                _countof!(szResource) as ::core::ffi::c_int,
            );
            StringCchPrintf!(
                core::ptr::addr_of_mut!(szText) as STRSAFE_LPWSTR,
                _countof!(szText) as size_t,
                core::ptr::addr_of_mut!(szResource) as STRSAFE_LPCWSTR,
                (*pFindReplace).lpstrFindWhat,
            );
            LoadString(
                Globals.hInstance,
                STRING_NOTEPAD as UINT,
                core::ptr::addr_of_mut!(szResource) as LPWSTR,
                _countof!(szResource) as ::core::ffi::c_int,
            );
            MessageBox(
                Globals.hFindReplaceDlg,
                core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPCWSTR,
                core::ptr::addr_of_mut!(szResource) as *mut TCHAR as LPCWSTR,
                MB_OK as UINT,
            );
        }
        bSuccess = (FALSE) as i32;
    }

    if !pszText.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pszText as PVOID);
    }
    return bSuccess;
}

unsafe extern "C" fn NOTEPAD_ReplaceAll(pFindReplace: *mut FINDREPLACE) {
    let mut bShowAlert: BOOL = (TRUE) as i32;

    SendMessage(Globals.hEdit, EM_SETSEL, 0, 0);

    while NOTEPAD_FindNext(pFindReplace, (TRUE) as BOOL, bShowAlert) != 0 {
        bShowAlert = (FALSE) as i32;
    }
}

unsafe extern "C" fn NOTEPAD_FindTerm() {
    Globals.hFindReplaceDlg = core::ptr::null_mut();
}

unsafe extern "C" fn NOTEPAD_InitData(hInstance: HINSTANCE) {
    let mut p: LPTSTR = core::ptr::null_mut();
    static txt_files: [u16; 6] = ['*' as u16, '.' as u16, 't' as u16, 'x' as u16, 't' as u16, 0];
    static all_files: [u16; 4] = ['*' as u16, '.' as u16, '*' as u16, 0];

    ZeroMemory!(
        core::ptr::addr_of_mut!(Globals) as *mut core::ffi::c_void,
        core::mem::size_of::<NOTEPAD_GLOBALS>(),
    );
    Globals.hInstance = hInstance;
    Globals.encFile = ENCODING_DEFAULT;

    p = core::ptr::addr_of_mut!(Globals.szFilter) as *mut TCHAR as LPTSTR;
    p = p.offset(
        (LoadString(
            Globals.hInstance,
            STRING_TEXT_FILES_TXT,
            p,
            (MAX_STRING_LEN as i32),
        ) + 1) as isize,
    );
    _tcscpy(p, core::ptr::addr_of!(txt_files) as *const TCHAR);
    p = p.offset(_countof!(txt_files) as isize);

    p = p.offset(
        (LoadString(
            Globals.hInstance,
            STRING_ALL_FILES,
            p,
            (MAX_STRING_LEN as i32),
        ) + 1) as isize,
    );
    _tcscpy(p, core::ptr::addr_of!(all_files) as *const TCHAR);
    p = p.offset(_countof!(all_files) as isize);
    *p = '\0' as u16;
    Globals.find.lpstrFindWhat = core::ptr::null_mut();

    Globals.hDevMode = core::ptr::null_mut();
    Globals.hDevNames = core::ptr::null_mut();
}

unsafe extern "C" fn NOTEPAD_InitMenuPopup(menu: HMENU, index: LPARAM) {
    let mut dwStart: DWORD = 0;
    let mut dwEnd: DWORD = 0;
    let mut enable: ::core::ffi::c_int = 0;

    UNREFERENCED_PARAMETER!(index);

    CheckMenuItem(menu, CMD_WRAP as UINT, (if Globals.bWrapLongLines != 0 { MF_CHECKED as UINT } else { MF_UNCHECKED as UINT }));
    CheckMenuItem(menu, CMD_STATUSBAR as UINT, (if Globals.bShowStatusBar != 0 { MF_CHECKED as UINT } else { MF_UNCHECKED as UINT }));
    EnableMenuItem(menu, CMD_UNDO as UINT,
        (if SendMessage(Globals.hEdit, EM_CANUNDO as UINT, 0 as WPARAM, 0 as LPARAM) != 0 { MF_ENABLED as UINT } else { MF_GRAYED as UINT }));
    EnableMenuItem(menu, CMD_PASTE as UINT,
        (if IsClipboardFormatAvailable(CF_TEXT as UINT) != 0 { MF_ENABLED as UINT } else { MF_GRAYED as UINT }));
    SendMessage(Globals.hEdit, EM_GETSEL as UINT, core::ptr::addr_of_mut!(dwStart) as WPARAM, core::ptr::addr_of_mut!(dwEnd) as LPARAM);
    enable = (if dwStart == dwEnd { (MF_GRAYED) as i32 } else { (MF_ENABLED) as i32 });
    EnableMenuItem(menu, CMD_CUT as UINT, enable as UINT);
    EnableMenuItem(menu, CMD_COPY as UINT, enable as UINT);
    EnableMenuItem(menu, CMD_DELETE as UINT, enable as UINT);

    EnableMenuItem(menu, CMD_SELECT_ALL as UINT,
        (if GetWindowTextLength(Globals.hEdit) != 0 { MF_ENABLED as UINT } else { MF_GRAYED as UINT }));
}

#[no_mangle]
pub unsafe extern "system" fn EDIT_WndProc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if msg == WM_KEYDOWN || msg == WM_KEYUP {
        if wParam == (VK_UP) as _
            || wParam == (VK_DOWN) as _
            || wParam == (VK_LEFT) as _
            || wParam == (VK_RIGHT) as _
        {
            DIALOG_StatusBarUpdateCaretPos();
        } else {
            UpdateWindowCaption((FALSE) as BOOL);
        }
        // fall through into the msg == COMMENT_TOKEN_ECHO____ case body
        DIALOG_StatusBarUpdateCaretPos();
    } else if msg == WM_LBUTTONUP {
        DIALOG_StatusBarUpdateCaretPos();
    }
    return CallWindowProc(Globals.EditProc, hWnd, msg, wParam, lParam);
}

unsafe extern "system" fn NOTEPAD_WndProc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match msg {

        WM_CREATE => {
            Globals.hMainWnd = hWnd;
            Globals.hMenu = GetMenu(hWnd);

            DragAcceptFiles(hWnd, (TRUE) as BOOL); /* Accept Drag & Drop */

            /* Create controls */
            DoCreateEditWindow();
            DoShowHideStatusBar();

            DIALOG_FileNew(); /* Initialize file info */

            // For now, the "Help" dialog is disabled due to the lack of HTML Help support
            EnableMenuItem(Globals.hMenu, CMD_HELP_CONTENTS, MF_BYCOMMAND | MF_GRAYED);
        }

        WM_COMMAND => {
            if (HIWORD!(wParam)) as u32 == EN_CHANGE || (HIWORD!(wParam)) as u32 == EN_HSCROLL || (HIWORD!(wParam)) as u32 == EN_VSCROLL {
                DIALOG_StatusBarUpdateCaretPos();
            }
            if (HIWORD!(wParam)) as u32 == EN_CHANGE {
                NOTEPAD_EnableSearchMenu();
            }
            NOTEPAD_MenuCommand((LOWORD!(wParam)) as WPARAM);
        }

        WM_CLOSE => {
            if DoCloseFile() != 0 {
                DestroyWindow(hWnd);
            }
        }

        WM_QUERYENDSESSION => {
            if DoCloseFile() != 0 {
                return 1;
            }
        }

        WM_DESTROY => {
            if !Globals.hFont.is_null() {
                DeleteObject((Globals.hFont).cast());
            }
            if !Globals.hDevMode.is_null() {
                GlobalFree(Globals.hDevMode);
            }
            if !Globals.hDevNames.is_null() {
                GlobalFree(Globals.hDevNames);
            }
            // C passes `(LONG_PTR)Globals.EditProc`, NULL when DoCreateEditWindow failed before
            // subclassing; `unwrap_unchecked` on that None was undefined behaviour.
            SetWindowLongPtr(
                Globals.hEdit,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, LONG_PTR>(Globals.EditProc),
            );
            NOTEPAD_SaveSettingsToRegistry();
            PostQuitMessage(0);
        }

        WM_SIZE => {
            let mut rc: RECT = core::mem::zeroed();
            GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));

            if Globals.bShowStatusBar != 0 {
                let mut rcStatus: RECT = core::mem::zeroed();
                SendMessageW(Globals.hStatusBar, WM_SIZE, 0, 0);
                GetWindowRect(Globals.hStatusBar, core::ptr::addr_of_mut!(rcStatus));
                rc.bottom -= rcStatus.bottom - rcStatus.top;
            }

            MoveWindow(Globals.hEdit, 0, 0, rc.right, rc.bottom, (TRUE) as BOOL);

            if Globals.bShowStatusBar != 0 {
                /* Align status bar parts, only if the status bar resize operation succeeds */
                DIALOG_StatusBarAlignParts();
            }
        }

        /* The entire client area is covered by edit control and by
         * the status bar. So there is no need to erase main background.
         * This resolves the horrible flicker effect during windows resizes. */
        WM_ERASEBKGND => {
            return 1;
        }

        WM_SETFOCUS => {
            SetFocus(Globals.hEdit);
        }

        WM_DROPFILES => {
            let mut szFileName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut hDrop: HDROP = wParam as HDROP;

            DragQueryFile(hDrop, 0, core::ptr::addr_of_mut!(szFileName) as *mut TCHAR, _countof!(szFileName));
            DragFinish(hDrop);
            DoOpenFile(core::ptr::addr_of_mut!(szFileName) as *mut TCHAR);
        }

        WM_INITMENUPOPUP => {
            NOTEPAD_InitMenuPopup(wParam as HMENU, lParam);
        }

        _ => {
            if msg == (aFINDMSGSTRING) as _ {
                let mut pFindReplace: *mut FINDREPLACE = lParam as *mut FINDREPLACE;
                Globals.find = *(lParam as *mut FINDREPLACE);

                WaitCursor((TRUE) as BOOL);

                if ((*pFindReplace).Flags & FR_FINDNEXT) != 0 {
                    NOTEPAD_FindNext(pFindReplace, (FALSE) as BOOL, (TRUE) as BOOL);
                } else if ((*pFindReplace).Flags & FR_REPLACE) != 0 {
                    NOTEPAD_FindNext(pFindReplace, (TRUE) as BOOL, (TRUE) as BOOL);
                } else if ((*pFindReplace).Flags & FR_REPLACEALL) != 0 {
                    NOTEPAD_ReplaceAll(pFindReplace);
                } else if ((*pFindReplace).Flags & FR_DIALOGTERM) != 0 {
                    NOTEPAD_FindTerm();
                }

                WaitCursor((FALSE) as BOOL);
            } else {
                return DefWindowProc(hWnd, msg, wParam, lParam);
            }
        }
    }
    return 0;
}

unsafe extern "C" fn AlertFileDoesNotExist(szFileName: LPCTSTR) -> core::ffi::c_int {
    return DIALOG_StringMsgBox(
        Globals.hMainWnd,
        (STRING_DOESNOTEXIST as i32),
        szFileName,
        (MB_ICONEXCLAMATION | MB_YESNO) as DWORD,
    );
}

unsafe extern "C" fn HandleCommandLine(mut cmdline: LPTSTR) -> BOOL {
    let mut opt_print: BOOL = (FALSE) as i32;
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    while *cmdline == (' ' as u16) || *cmdline == ('-' as u16) || *cmdline == ('/' as u16)
    {
        let mut option: TCHAR;

        let fresh_cmd = cmdline;
        cmdline = cmdline.offset(1);
        if *fresh_cmd == (' ' as u16) {
            continue;
        }

        option = *cmdline;
        if option != 0 {
            cmdline = cmdline.offset(1);
        }
        while *cmdline == (' ' as u16) {
            cmdline = cmdline.offset(1);
        }

        match option as i32 {
            112 | 80 => {
                // case 'p': case 'P':
                opt_print = (TRUE) as i32;
            }
            _ => {}
        }
    }

    if *cmdline != 0 {
        /* file name is passed in the command line */
        let mut file_name: LPCTSTR = core::ptr::null();
        let mut file_exists: BOOL = (FALSE) as i32;
        let mut buf: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if *cmdline.offset(0 as isize) == ('"' as u16) {
            cmdline = cmdline.offset(1);
            *cmdline.offset((lstrlen(cmdline) - 1) as isize) =
                0;
        }

        file_name = cmdline as LPCTSTR;
        if FileExists(file_name) != 0 {
            file_exists = (TRUE) as i32;
        } else if HasFileExtension(cmdline as LPCTSTR) == 0 {
            static txt: [TCHAR; 5] = ['.' as TCHAR, 't' as TCHAR, 'x' as TCHAR, 't' as TCHAR, 0];
            let txt_p: LPCTSTR = core::ptr::addr_of!(txt) as LPCTSTR;

            /* try to find file with ".txt" extension */
            // C compares `cmdline + len - 4` unguarded, pointing BEFORE the string for a
            // name shorter than ".txt"; such a name cannot end in ".txt".
            let cch = _tcslen(cmdline as LPCTSTR) as isize;
            let cch_txt = _tcslen(txt_p) as isize;
            if cch >= cch_txt && _tcscmp(txt_p, cmdline.offset(cch - cch_txt)) == 0
            {
                file_exists = (FALSE) as i32;
            } else {
                _tcsncpy(
                    core::ptr::addr_of_mut!(buf) as LPTSTR,
                    cmdline as LPCTSTR,
                    (MAX_PATH - (_tcslen(txt_p)) as u32 - 1)
                        as usize,
                );
                _tcscat(core::ptr::addr_of_mut!(buf) as LPTSTR, txt_p);
                file_name = core::ptr::addr_of_mut!(buf) as LPTSTR as LPCTSTR;
                file_exists = FileExists(file_name);
            }
        }

        GetFullPathName(
            file_name,
            _countof!(szPath),
            core::ptr::addr_of_mut!(szPath) as LPTSTR,
            core::ptr::null_mut(),
        );

        if file_exists != 0 {
            DoOpenFile(core::ptr::addr_of_mut!(szPath) as LPCTSTR);
            InvalidateRect(
                Globals.hMainWnd,
                core::ptr::null_mut(),
                (FALSE) as BOOL,
            );
            if opt_print != 0 {
                DIALOG_FilePrint();
                return (FALSE) as BOOL;
            }
        } else {
            match (AlertFileDoesNotExist(file_name)) as u32 {
                IDYES => {
                    DoOpenFile(core::ptr::addr_of_mut!(szPath) as LPCTSTR);
                }
                IDNO => {}
                _ => {}
            }
        }
    }

    return (TRUE) as BOOL;
}

#[export_name = "wWinMain"]
pub unsafe extern "system" fn _tWinMain(
    hInstance: HINSTANCE,
    prev: HINSTANCE,
    cmdline: LPTSTR,
    show: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut msg: MSG = core::mem::zeroed();
    let mut hAccel: HACCEL = core::ptr::null_mut();
    let mut wndclass: WNDCLASSEXW = core::mem::zeroed();
    let mut wp: WINDOWPLACEMENT = core::mem::zeroed();
    static className: [TCHAR; 8] = [
        'N' as TCHAR,
        'o' as TCHAR,
        't' as TCHAR,
        'e' as TCHAR,
        'p' as TCHAR,
        'a' as TCHAR,
        'd' as TCHAR,
        0 as TCHAR,
    ];
    static winName: [TCHAR; 8] = [
        'N' as TCHAR,
        'o' as TCHAR,
        't' as TCHAR,
        'e' as TCHAR,
        'p' as TCHAR,
        'a' as TCHAR,
        'd' as TCHAR,
        0 as TCHAR,
    ];

    if GetUserDefaultUILanguage() == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) {
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    UNREFERENCED_PARAMETER!(prev);

    aFINDMSGSTRING = RegisterWindowMessage(FINDMSGSTRING.as_ptr().cast()) as ATOM;

    NOTEPAD_InitData(hInstance);
    NOTEPAD_LoadSettingsFromRegistry(core::ptr::addr_of_mut!(wp));

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndclass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSEX>(),
    );
    wndclass.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wndclass.lpfnWndProc = Some(NOTEPAD_WndProc);
    wndclass.hInstance = Globals.hInstance;
    wndclass.hIcon = LoadIcon(
        hInstance,
        MAKEINTRESOURCE!(IDI_NPICON),
    ) as HICON;
    wndclass.hCursor = LoadCursor(
        core::ptr::null_mut(),
        IDC_ARROW,
    ) as HICON;
    wndclass.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wndclass.lpszMenuName = MAKEINTRESOURCE!(MAIN_MENU);
    wndclass.lpszClassName = (core::ptr::addr_of!(className) as LPCTSTR).cast();
    wndclass.hIconSm = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_NPICON),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        0,
    ) as HICON;
    if RegisterClassEx(core::ptr::addr_of_mut!(wndclass)) == 0 {
        ShowLastError();
        return 1;
    }

    /* Globals.hMainWnd will be set in WM_CREATE handling */
    CreateWindow!(
        core::ptr::addr_of!(className) as LPCTSTR,
        core::ptr::addr_of!(winName) as LPCTSTR,
        WS_OVERLAPPEDWINDOW,
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
        ShowLastError();
        return 1;
    }

    /* Use the result of CW_USEDEFAULT if the data in the registry is not valid */
    if wp.rcNormalPosition.right == wp.rcNormalPosition.left {
        GetWindowPlacement(Globals.hMainWnd, core::ptr::addr_of_mut!(wp));
    }
    /* Does the parent process want to force a show action? */
    if (show) as u32 != SW_SHOWDEFAULT {
        wp.showCmd = show as UINT;
    }
    SetWindowPlacement(Globals.hMainWnd, core::ptr::addr_of_mut!(wp));
    UpdateWindow(Globals.hMainWnd);
    SetForegroundWindow(Globals.hMainWnd);
    SetFocus(Globals.hEdit);

    if HandleCommandLine(cmdline) == 0 {
        return 0;
    }

    hAccel = LoadAccelerators(hInstance, MAKEINTRESOURCE!(ID_ACCEL));

    while GetMessage(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        if (Globals.hFindReplaceDlg.is_null()
            || IsDialogMessage(Globals.hFindReplaceDlg, core::ptr::addr_of_mut!(msg)) == 0)
            && TranslateAccelerator(Globals.hMainWnd, hAccel, core::ptr::addr_of_mut!(msg)) == 0
        {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    DestroyAcceleratorTable(hAccel);

    return msg.wParam as ::core::ffi::c_int;
}
