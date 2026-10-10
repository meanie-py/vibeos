// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut helpfile: [TCHAR; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [TCHAR; 12]>(*b"n\0o\0t\0e\0p\0a\0d\0.\0h\0l\0p\0\0\0")
};

static mut empty_str: [TCHAR; 1] =
    unsafe { ::core::mem::transmute::<[u8; 2], [TCHAR; 1]>(*b"\0\0") };

static mut szDefaultExt: [TCHAR; 4] =
    unsafe { ::core::mem::transmute::<[u8; 8], [TCHAR; 4]>(*b"t\0x\0t\0\0\0") };

static mut txt_files: [TCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [TCHAR; 6]>(*b"*\0.\0t\0x\0t\0\0\0") };

static mut EolnToStrId: [UINT; 3] = [STRING_CRLF as UINT, STRING_LF as UINT, STRING_CR as UINT];

static mut EncToStrId: [UINT; 5] = [
    STRING_ANSI as UINT,
    STRING_UNICODE as UINT,
    STRING_UNICODE_BE as UINT,
    STRING_UTF8 as UINT,
    STRING_UTF8_BOM as UINT,
];

#[repr(C)]
pub struct tagGOTO_DATA {
    pub iLine: UINT,
    pub cLines: UINT,
}
pub type GOTO_DATA = tagGOTO_DATA;
pub type PGOTO_DATA = *mut GOTO_DATA;

pub type FINDPROC = unsafe extern "system" fn(lpfr: LPFINDREPLACE) -> HWND;

pub const SBPART_EOLN: i32 = 1;
pub const SBPART_ENCODING: i32 = 2;
#[no_mangle]
pub unsafe extern "C" fn ShowLastError() {
    let error: DWORD = GetLastError();
    if error != NO_ERROR {
        let mut lpMsgBuf: LPTSTR = core::ptr::null_mut();
        let mut szTitle: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
        let mut szFallback: [TCHAR; 42] = [0; 42];
        let mut pszMessage: LPTSTR = core::ptr::addr_of_mut!(szFallback) as *mut TCHAR;

        LoadString(
            Globals.hInstance,
            STRING_ERROR as UINT,
            core::ptr::addr_of_mut!(szTitle) as *mut TCHAR,
            _countof!(szTitle) as i32,
        );

        FormatMessage(
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD,
            core::ptr::null_mut(),
            error,
            0 as DWORD,
            core::ptr::addr_of_mut!(lpMsgBuf) as *mut TCHAR,
            0 as DWORD,
            core::ptr::null_mut(),
        );

        if !lpMsgBuf.is_null() {
            pszMessage = lpMsgBuf;
        } else {
            wsprintfW(
                core::ptr::addr_of_mut!(szFallback) as *mut TCHAR,
                RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                error,
            );
        }

        MessageBox(
            Globals.hMainWnd,
            pszMessage as *const TCHAR,
            core::ptr::addr_of_mut!(szTitle) as *const TCHAR,
            (MB_OK | MB_ICONERROR) as UINT,
        );
        LocalFree(lpMsgBuf as HLOCAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateWindowCaption(clearModifyAlert: BOOL) {
    let mut szCaption: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut szNotepad: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut szFilename: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut isModified: BOOL;

    if clearModifyAlert != 0 {
        /* When a file is being opened or created, there is no need to have
         * the edited flag shown when the file has not been edited yet. */
        isModified = (FALSE) as i32;
    } else {
        /* Check whether the user has modified the file or not. If we are
         * in the same state as before, don't change the caption. */
        isModified = (SendMessage(Globals.hEdit, EM_GETMODIFY, 0, 0) != 0) as BOOL;
        if isModified == Globals.bWasModified {
            return;
        }
    }

    /* Remember the state for later calls */
    Globals.bWasModified = isModified;

    /* Load the name of the application */
    LoadString(Globals.hInstance, STRING_NOTEPAD, core::ptr::addr_of_mut!(szNotepad) as *mut TCHAR, (_countof!(szNotepad)) as i32);

    /* Determine if the file has been saved or if this is a new file */
    if Globals.szFileTitle[0] != 0 {
        StringCchCopy(core::ptr::addr_of_mut!(szFilename) as *mut TCHAR, (_countof!(szFilename)) as _, core::ptr::addr_of_mut!(Globals.szFileTitle) as *mut TCHAR);
    } else {
        LoadString(Globals.hInstance, STRING_UNTITLED, core::ptr::addr_of_mut!(szFilename) as *mut TCHAR, (_countof!(szFilename)) as i32);
    }

    let fmt: [u16; 10] = ['%' as u16, 's' as u16, '%' as u16, 's' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, 0];
    let star: [u16; 2] = ['*' as u16, 0];
    let empty: [u16; 1] = [0];

    /* Update the window caption based upon whether the user has modified the file or not */
    StringCbPrintf!(
        core::ptr::addr_of_mut!(szCaption) as *mut TCHAR,
        core::mem::size_of::<[TCHAR; (MAX_STRING_LEN as usize)]>(),
        fmt.as_ptr() as *const TCHAR,
        (if isModified != 0 { star.as_ptr() } else { empty.as_ptr() }) as *const TCHAR,
        core::ptr::addr_of_mut!(szFilename) as *mut TCHAR,
        core::ptr::addr_of_mut!(szNotepad) as *mut TCHAR,
    );

    SetWindowText(Globals.hMainWnd, core::ptr::addr_of_mut!(szCaption) as *mut TCHAR);
}
#[no_mangle]
pub unsafe extern "C" fn WaitCursor(bBegin: BOOL) {
    static mut s_hWaitCursor: HCURSOR = core::ptr::null_mut();
    static mut s_hOldCursor: HCURSOR = core::ptr::null_mut();
    static mut s_nLock: INT = 0;

    if bBegin != 0 {
        let fresh15 = s_nLock;
        s_nLock = s_nLock + 1;
        if fresh15 == 0 {
            if s_hWaitCursor.is_null() {
                s_hWaitCursor = LoadCursor(
                    core::ptr::null_mut(),
                    IDC_WAIT as usize as *const u16,
                );
            }
            s_hOldCursor = SetCursor(s_hWaitCursor);
        } else {
            SetCursor(s_hWaitCursor);
        }
    } else {
        s_nLock -= 1;
        if s_nLock == 0 {
            SetCursor(s_hOldCursor);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_StatusBarAlignParts() {
    static defaultWidths: [::core::ffi::c_int; 3] = [
        120 as ::core::ffi::c_int,
        120 as ::core::ffi::c_int,
        120 as ::core::ffi::c_int,
    ];
    let mut rcStatusBar: RECT = core::mem::zeroed();
    let mut parts: [::core::ffi::c_int; 3] = [0 as ::core::ffi::c_int; 3 as usize];

    GetClientRect(Globals.hStatusBar, core::ptr::addr_of_mut!(rcStatusBar));

    parts[0 as usize] = rcStatusBar.right
        - (defaultWidths[1 as usize] + defaultWidths[2 as usize]);
    parts[1 as usize] = rcStatusBar.right - defaultWidths[2 as usize];
    parts[2 as usize] = -(1);

    parts[0 as usize] = max(
        parts[0 as usize],
        defaultWidths[0 as usize],
    );
    parts[1 as usize] = max(
        parts[1 as usize],
        defaultWidths[0 as usize] + defaultWidths[1 as usize],
    );

    SendMessageW(
        Globals.hStatusBar,
        SB_SETPARTS as UINT,
        (_countof!(parts)) as WPARAM,
        core::ptr::addr_of_mut!(parts) as *mut ::core::ffi::c_int as LPARAM,
    );
}

unsafe extern "C" fn DIALOG_StatusBarUpdateLineEndings() {
    let mut szText: [WCHAR; 128] = [0; 128];

    LoadStringW(
        Globals.hInstance,
        // C indexes unchecked; an out-of-range EOLN (CB_ERR from the save hook) reads past
        // the table. Rust would panic: load string id 0 (none) instead.
        EolnToStrId.get(Globals.iEoln as usize).copied().unwrap_or(0),
        core::ptr::addr_of_mut!(szText) as LPWSTR,
        (_countof!(szText)) as INT,
    );

    SendMessageW(
        Globals.hStatusBar,
        SB_SETTEXTW as UINT,
        SBPART_EOLN as WPARAM,
        core::ptr::addr_of_mut!(szText) as LPARAM,
    );
}

unsafe extern "C" fn DIALOG_StatusBarUpdateEncoding() {
    let mut szText: [WCHAR; 128] = [0 as WCHAR; 128];

    if Globals.encFile as ::core::ffi::c_int != ENCODING_AUTO as ::core::ffi::c_int {
        LoadStringW(
            Globals.hInstance,
            EncToStrId.get(Globals.encFile as usize).copied().unwrap_or(0),
            &raw mut szText as LPWSTR,
            _countof!(szText) as ::core::ffi::c_int,
        );
    }

    SendMessageW(
        Globals.hStatusBar,
        SB_SETTEXTW as UINT,
        SBPART_ENCODING as WPARAM,
        &raw mut szText as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn DIALOG_StatusBarUpdateAll() {
    DIALOG_StatusBarUpdateCaretPos();
    DIALOG_StatusBarUpdateLineEndings();
    DIALOG_StatusBarUpdateEncoding();
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_StringMsgBox(hParent: HWND, formatId: ::core::ffi::c_int, szString: LPCTSTR, dwFlags: DWORD) -> ::core::ffi::c_int {
    let mut szMessage: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut szResource: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];

    /* Load and format szMessage */
    LoadString(Globals.hInstance, formatId as UINT, szResource.as_mut_ptr() as LPWSTR, _countof!(szResource) as c_int);
    StringCchPrintf!(szMessage.as_mut_ptr() as STRSAFE_LPWSTR, _countof!(szMessage) as size_t, szResource.as_mut_ptr() as STRSAFE_LPCWSTR, szString);

    /* Load szCaption */
    if (dwFlags & MB_ICONMASK) == MB_ICONEXCLAMATION {
        LoadString(Globals.hInstance, STRING_ERROR as UINT, szResource.as_mut_ptr() as LPWSTR, _countof!(szResource) as c_int);
    } else {
        LoadString(Globals.hInstance, STRING_NOTEPAD as UINT, szResource.as_mut_ptr() as LPWSTR, _countof!(szResource) as c_int);
    }

    /* Display Modal Dialog */
    // if (hParent == NULL)
        // hParent = Globals.hMainWnd;
    return MessageBox(hParent, szMessage.as_mut_ptr() as LPCWSTR, szResource.as_mut_ptr() as LPCWSTR, dwFlags as UINT);
}

unsafe extern "C" fn AlertFileNotFound(szFileName: LPCTSTR) {
    DIALOG_StringMsgBox(Globals.hMainWnd, (STRING_NOTFOUND as i32), szFileName, (MB_ICONEXCLAMATION | MB_OK) as DWORD);
}

unsafe extern "C" fn AlertFileNotSaved(mut szFileName: LPCTSTR) -> ::core::ffi::c_int {
    let mut szUntitled: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];

    LoadString(
        Globals.hInstance,
        STRING_UNTITLED as UINT,
        szUntitled.as_mut_ptr() as LPTSTR,
        _countof!(szUntitled) as ::core::ffi::c_int,
    );

    return DIALOG_StringMsgBox(
        Globals.hMainWnd,
        (STRING_NOTSAVED as UINT) as i32,
        if *szFileName.offset(0 as isize) != 0 {
            szFileName
        } else {
            szUntitled.as_mut_ptr() as LPCTSTR
        },
        (MB_ICONQUESTION | MB_YESNOCANCEL) as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn FileExists(mut szFilename: LPCTSTR) -> BOOL {
    return (GetFileAttributes(szFilename as LPCWSTR) != INVALID_FILE_ATTRIBUTES) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn HasFileExtension(mut szFilename: LPCTSTR) -> BOOL {
    let mut s: LPCTSTR;
    s = _tcsrchr(szFilename, ('\\' as u16));
    if !s.is_null() {
        szFilename = s;
    }
    return (_tcsrchr(szFilename, ('.' as u16)) != core::ptr::null_mut()) as BOOL;
}

unsafe extern "C" fn DoSaveFile() -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;
    let mut hFile: HANDLE;
    let mut cchText: DWORD;

    WaitCursor((TRUE) as BOOL);

    /* Use OPEN_ALWAYS instead of CREATE_ALWAYS in order to succeed
     * even if the file has HIDDEN or SYSTEM attributes */
    hFile = CreateFileW(
        core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR as LPCWSTR,
        GENERIC_WRITE,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        ShowLastError();
        WaitCursor((FALSE) as BOOL);
        return (FALSE) as BOOL;
    }

    cchText = GetWindowTextLengthW(Globals.hEdit) as DWORD;
    if cchText <= 0 {
        bRet = (TRUE) as i32;
    } else {
        let mut hLocal: HLOCAL = SendMessageW(
            Globals.hEdit,
            EM_GETHANDLE,
            0,
            0,
        ) as HLOCAL;
        let mut pszText: LPWSTR = LocalLock(hLocal) as LPWSTR;
        if !pszText.is_null() {
            bRet = WriteText(
                hFile,
                pszText as LPCWSTR,
                cchText,
                Globals.encFile,
                Globals.iEoln,
            );
            if bRet == 0 {
                ShowLastError();
            }

            LocalUnlock(hLocal);
        } else {
            ShowLastError();
        }
    }

    /* Truncate the file and close it */
    SetEndOfFile(hFile);
    CloseHandle(hFile);

    if bRet != 0 {
        SendMessage(
            Globals.hEdit,
            EM_SETMODIFY,
            (FALSE) as _,
            0,
        );
        SetFileName(core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR as LPCTSTR);
    }

    WaitCursor((FALSE) as BOOL);
    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn DoCloseFile() -> BOOL {
    let mut nResult: ::core::ffi::c_int = 0;

    if SendMessage(
        Globals.hEdit,
        EM_GETMODIFY as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) != 0
    {
        /* prompt user to save changes */
        nResult = AlertFileNotSaved(core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR as LPCTSTR);
        match (nResult) as u32 {
            IDYES => {
                if DIALOG_FileSave() == 0 {
                    return (FALSE) as BOOL;
                }
            }
            IDNO => {}
            IDCANCEL | _ => return (FALSE) as i32,
        }
    }

    SetFileName(core::ptr::addr_of!(empty_str) as LPCTSTR);
    UpdateWindowCaption((TRUE) as BOOL);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DoOpenFile(szFileName: LPCTSTR) {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut log: [TCHAR; 5] = [0; 5];
    let mut hOldLocal: HLOCAL = core::ptr::null_mut();
    let mut hNewLocal: HLOCAL = core::ptr::null_mut();

    /* Close any files and prompt to save changes */
    if DoCloseFile() == 0 {
        return;
    }

    WaitCursor((TRUE) as BOOL);
    SetWindowText(Globals.hEdit, core::ptr::null_mut());

    hFile = CreateFile(
        szFileName,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    'done: {
        if hFile == INVALID_HANDLE_VALUE {
            ShowLastError();
            break 'done;
        }

        /* To make loading file quicker, we use the internal handle of EDIT control */
        hOldLocal = SendMessageW(
            Globals.hEdit,
            EM_GETHANDLE,
            0,
            0,
        ) as HLOCAL;
        hNewLocal = ReadText(
            hFile,
            core::ptr::addr_of_mut!(Globals.encFile),
            core::ptr::addr_of_mut!(Globals.iEoln),
        );
        if hNewLocal.is_null() {
            ShowLastError();
            break 'done;
        }
        SendMessageW(
            Globals.hEdit,
            EM_SETHANDLE,
            hNewLocal as WPARAM,
            0,
        );
        LocalFree(hOldLocal);
        /* No need of EM_SETMODIFY and EM_EMPTYUNDOBUFFER here. EM_SETHANDLE does instead. */

        SetFocus(Globals.hEdit);

        /*  If the file starts with .LOG, add a time/date at the end and set cursor after
         *  See http://web.archive.org/web/20090627165105/http://support.microsoft.com/kb/260563
         */
        if GetWindowText(Globals.hEdit, log.as_mut_ptr(), _countof!(log) as core::ffi::c_int) != 0
            && _tcscmp(log.as_ptr(), (&[0x002e, 0x004c, 0x004f, 0x0047, 0x0000u16]).as_ptr() /* _T ".LOG" */) == 0
        {
            let lf: LPCTSTR = (&[0x000d, 0x000a, 0x0000u16]).as_ptr() /* _T "\r\n" */;
            SendMessage(
                Globals.hEdit,
                EM_SETSEL,
                GetWindowTextLength(Globals.hEdit) as WPARAM,
                -1,
            );
            SendMessage(
                Globals.hEdit,
                EM_REPLACESEL,
                (TRUE) as _,
                lf as LPARAM,
            );
            DIALOG_EditTimeDate();
            SendMessage(
                Globals.hEdit,
                EM_REPLACESEL,
                (TRUE) as _,
                lf as LPARAM,
            );
        }

        SetFileName(szFileName);
        UpdateWindowCaption((TRUE) as BOOL);
        NOTEPAD_EnableSearchMenu();
        DIALOG_StatusBarUpdateAll();
    }

    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    WaitCursor((FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileNew() {
    /* Close any files and prompt to save changes */
    if DoCloseFile() == 0 {
        return;
    }

    WaitCursor((TRUE) as BOOL);

    SetWindowText(Globals.hEdit, core::ptr::null_mut());
    SendMessage(Globals.hEdit, EM_EMPTYUNDOBUFFER, 0, 0);
    Globals.iEoln = EOLN_CRLF;
    Globals.encFile = ENCODING_DEFAULT;

    NOTEPAD_EnableSearchMenu();
    DIALOG_StatusBarUpdateAll();

    WaitCursor((FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileNewWindow() {
    let mut pszNotepadExe: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    WaitCursor((TRUE) as BOOL);

    GetModuleFileName(core::ptr::null_mut(), pszNotepadExe.as_mut_ptr(), _countof!(pszNotepadExe));
    ShellExecute(core::ptr::null_mut(), core::ptr::null_mut(), pszNotepadExe.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (SW_SHOWNORMAL) as i32);

    WaitCursor((FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileOpen() {
    let mut openfilename: OPENFILENAME = core::mem::zeroed();
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    ZeroMemory!(
        core::ptr::addr_of_mut!(openfilename) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAME>(),
    );

    if Globals.szFileName[0] == 0 {
        _tcscpy(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::ptr::addr_of!(txt_files) as *const TCHAR,
        );
    } else {
        _tcscpy(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR,
        );
    }

    openfilename.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    openfilename.hwndOwner = Globals.hMainWnd;
    openfilename.hInstance = Globals.hInstance;
    openfilename.lpstrFilter = core::ptr::addr_of_mut!(Globals.szFilter) as *mut TCHAR as LPCWSTR;
    openfilename.lpstrFile = core::ptr::addr_of_mut!(szPath) as *mut TCHAR as LPWSTR;
    openfilename.nMaxFile = _countof!(szPath) as DWORD;
    openfilename.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
    openfilename.lpstrDefExt = core::ptr::addr_of!(szDefaultExt) as *const TCHAR as LPCWSTR;

    if GetOpenFileName(core::ptr::addr_of_mut!(openfilename)) != 0 {
        if FileExists(openfilename.lpstrFile as LPCTSTR) != 0 {
            DoOpenFile(openfilename.lpstrFile as LPCTSTR);
        } else {
            AlertFileNotFound(openfilename.lpstrFile as LPCTSTR);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileSave() -> BOOL {
    if Globals.szFileName[0] == 0 {
        return DIALOG_FileSaveAs();
    } else if DoSaveFile() != 0 {
        UpdateWindowCaption((TRUE) as BOOL);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "system" fn DIALOG_FileSaveAs_Hook(
    hDlg: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT_PTR {
    let mut szText: [TCHAR; 128] = [0; 128];
    let mut hCombo: HWND;

    UNREFERENCED_PARAMETER!(wParam);

    match msg {
        WM_INITDIALOG => {
            hCombo = GetDlgItem(hDlg, (ID_ENCODING as i32));

            LoadString(Globals.hInstance, STRING_ANSI, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_UNICODE, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_UNICODE_BE, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_UTF8, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_UTF8_BOM, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            SendMessage(hCombo, CB_SETCURSEL, (Globals.encFile) as _, 0);

            hCombo = GetDlgItem(hDlg, (ID_EOLN as i32));

            LoadString(Globals.hInstance, STRING_CRLF, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_LF, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            LoadString(Globals.hInstance, STRING_CR, addr_of_mut!(szText) as *mut TCHAR as LPTSTR, (_countof!(szText)) as i32);
            SendMessage(hCombo, CB_ADDSTRING, 0, addr_of_mut!(szText) as *mut TCHAR as LPARAM);

            SendMessage(hCombo, CB_SETCURSEL, (Globals.iEoln) as _, 0);
        }

        WM_NOTIFY => {
            if (*(lParam as *mut NMHDR)).code == CDN_FILEOK {
                hCombo = GetDlgItem(hDlg, (ID_ENCODING as i32));
                if !hCombo.is_null() {
                    Globals.encFile = SendMessage(hCombo, CB_GETCURSEL, 0, 0) as ENCODING;
                }

                hCombo = GetDlgItem(hDlg, (ID_EOLN as i32));
                if !hCombo.is_null() {
                    Globals.iEoln = SendMessage(hCombo, CB_GETCURSEL, 0, 0) as EOLN;
                }
            }
        }

        _ => {}
    }

    return 0 as UINT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileSaveAs() -> BOOL {
    let mut saveas: OPENFILENAME = core::mem::zeroed();
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    ZeroMemory!(
        core::ptr::addr_of_mut!(saveas) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAME>(),
    );

    if Globals.szFileName[0] as core::ffi::c_int == 0 as core::ffi::c_int {
        _tcscpy(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::ptr::addr_of!(txt_files) as *const TCHAR,
        );
    } else {
        _tcscpy(
            core::ptr::addr_of_mut!(szPath) as *mut TCHAR,
            core::ptr::addr_of_mut!(Globals.szFileName) as *mut TCHAR,
        );
    }

    saveas.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    saveas.hwndOwner = Globals.hMainWnd;
    saveas.hInstance = Globals.hInstance;
    saveas.lpstrFilter = core::ptr::addr_of_mut!(Globals.szFilter) as *mut TCHAR as LPCWSTR;
    saveas.lpstrFile = core::ptr::addr_of_mut!(szPath) as *mut TCHAR as LPWSTR;
    saveas.nMaxFile = _countof!(szPath) as DWORD;
    saveas.Flags = (OFN_PATHMUSTEXIST
        | OFN_OVERWRITEPROMPT
        | OFN_HIDEREADONLY
        | OFN_EXPLORER
        | OFN_ENABLETEMPLATE
        | OFN_ENABLEHOOK) as DWORD;
    saveas.lpstrDefExt = core::ptr::addr_of!(szDefaultExt) as *const TCHAR as LPCWSTR;
    saveas.lpTemplateName = MAKEINTRESOURCE!(DIALOG_ENCODING) as LPCWSTR;
    saveas.lpfnHook = Some(
        DIALOG_FileSaveAs_Hook,
    );

    if GetSaveFileName(core::ptr::addr_of_mut!(saveas)) != 0 {
        /* HACK: Because in ROS, Save-As boxes don't check the validity
         * of file names and thus, here, szPath can be invalid !! We only
         * see its validity when we call DoSaveFile()... */
        SetFileName(core::ptr::addr_of_mut!(szPath) as *mut TCHAR as LPCTSTR);
        if DoSaveFile() != 0 {
            UpdateWindowCaption((TRUE) as BOOL);
            DIALOG_StatusBarUpdateAll();
            return (TRUE) as BOOL;
        } else {
            SetFileName((&[0x0000u16]).as_ptr() /* _T "" */ as LPCTSTR);
            return (FALSE) as BOOL;
        }
    } else {
        return (FALSE) as BOOL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_FileExit() {
    PostMessage(Globals.hMainWnd, WM_CLOSE, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditUndo() {
    SendMessage(Globals.hEdit, EM_UNDO as UINT, 0 as WPARAM, 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditCut() {
    SendMessage(Globals.hEdit, WM_CUT, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditCopy() {
    SendMessage(Globals.hEdit, WM_COPY, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditPaste() {
    SendMessage(Globals.hEdit, WM_PASTE, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditDelete() {
    SendMessage(Globals.hEdit, WM_CLEAR as UINT, 0 as WPARAM, 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditSelectAll() {
    SendMessage(Globals.hEdit, EM_SETSEL as UINT, 0 as WPARAM, -(1 as LPARAM));
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditTimeDate() {
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut szDate: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut szText: [TCHAR; (MAX_STRING_LEN * 2 + 2) as _] =
        [0; (MAX_STRING_LEN * 2 + 2) as _];

    GetLocalTime(core::ptr::addr_of_mut!(st));

    GetTimeFormat(
        LOCALE_USER_DEFAULT,
        0,
        core::ptr::addr_of_mut!(st),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szDate) as *mut TCHAR,
        (MAX_STRING_LEN as i32),
    );
    _tcscpy(
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        core::ptr::addr_of_mut!(szDate) as *mut TCHAR,
    );
    _tcscat(
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        (&[0x0020, 0x0000u16]).as_ptr() /* _T " " */,
    );
    GetDateFormat(
        LOCALE_USER_DEFAULT,
        DATE_LONGDATE,
        core::ptr::addr_of_mut!(st),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szDate) as *mut TCHAR,
        (MAX_STRING_LEN as i32),
    );
    _tcscat(
        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
        core::ptr::addr_of_mut!(szDate) as *mut TCHAR,
    );
    SendMessage(
        Globals.hEdit,
        EM_REPLACESEL,
        (TRUE) as _,
        core::ptr::addr_of_mut!(szText) as *mut TCHAR as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DoShowHideStatusBar() {
    /* Check if status bar object already exists. */
    if Globals.bShowStatusBar != 0 && Globals.hStatusBar.is_null() {
        /* Try to create the status bar */
        Globals.hStatusBar = CreateStatusWindow(
            (WS_CHILD | CCS_BOTTOM | SBARS_SIZEGRIP) as LONG,
            ::core::ptr::null::<WCHAR>(),
            Globals.hMainWnd,
            CMD_STATUSBAR_WND_ID as UINT,
        );

        if Globals.hStatusBar.is_null() {
            ShowLastError();
            return;
        }

        /* Load the string for formatting column/row text output */
        LoadString(
            Globals.hInstance,
            STRING_LINE_COLUMN as UINT,
            core::ptr::addr_of_mut!(Globals.szStatusBarLineCol) as LPWSTR,
            (MAX_PATH - 1) as ::core::ffi::c_int,
        );
    }

    /* Update layout of controls */
    SendMessageW(
        Globals.hMainWnd,
        WM_SIZE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    if Globals.hStatusBar.is_null() {
        return;
    }

    /* Update visibility of status bar */
    ShowWindow(
        Globals.hStatusBar,
        if Globals.bShowStatusBar != 0 {
            (SW_SHOWNOACTIVATE) as i32
        } else {
            (SW_HIDE) as i32
        },
    );

    /* Update status bar contents */
    DIALOG_StatusBarUpdateAll();
}
#[no_mangle]
pub unsafe extern "C" fn DoCreateEditWindow() {
    let mut dwStyle: DWORD;
    let mut iSize: ::core::ffi::c_int;
    let mut pTemp: LPTSTR = core::ptr::null_mut();
    let mut bModified: BOOL = (FALSE) as i32;

    iSize = 0;

    /* If the edit control already exists, try to save its content */
    if !Globals.hEdit.is_null() {
        /* number of chars currently written into the editor. */
        iSize = GetWindowTextLength(Globals.hEdit);
        if iSize != 0 {
            /* Allocates temporary buffer. */
            pTemp = HeapAlloc(
                GetProcessHeap(),
                0,
                ((iSize + 1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
            ) as LPTSTR;
            if pTemp.is_null() {
                ShowLastError();
                return;
            }

            /* Recover the text into the control. */
            GetWindowText(Globals.hEdit, pTemp, iSize + 1);

            if SendMessage(Globals.hEdit, EM_GETMODIFY, 0, 0) != 0 {
                bModified = (TRUE) as i32;
            }
        }

        /* Restore original window procedure */
        SetWindowLongPtr(
            Globals.hEdit,
            GWLP_WNDPROC,
            ::core::mem::transmute::<WNDPROC, LONG_PTR>(Globals.EditProc),
        );

        /* Destroy the edit control */
        DestroyWindow(Globals.hEdit);
    }

    /* Update wrap status into the main menu and recover style flags */
    dwStyle = if Globals.bWrapLongLines != 0 {
        EDIT_STYLE_WRAP
    } else {
        EDIT_STYLE
    };

    /* Create the new edit control */
    Globals.hEdit = CreateWindowEx(
        WS_EX_CLIENTEDGE,
        EDIT_CLASS,
        core::ptr::null(),
        dwStyle,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        Globals.hMainWnd,
        core::ptr::null_mut(),
        Globals.hInstance,
        core::ptr::null_mut(),
    );
    if Globals.hEdit.is_null() {
        if !pTemp.is_null() {
            HeapFree(GetProcessHeap(), 0, pTemp as PVOID);
        }

        ShowLastError();
        return;
    }

    SendMessage(
        Globals.hEdit,
        WM_SETFONT,
        Globals.hFont as WPARAM,
        (FALSE) as _,
    );
    SendMessage(
        Globals.hEdit,
        EM_LIMITTEXT,
        0,
        0,
    );

    /* If some text was previously saved, restore it. */
    if iSize != 0 {
        SetWindowText(Globals.hEdit, pTemp);
        HeapFree(GetProcessHeap(), 0, pTemp as PVOID);

        if bModified != 0 {
            SendMessage(
                Globals.hEdit,
                EM_SETMODIFY,
                (TRUE) as _,
                0,
            );
        }
    }

    /* Sub-class a new window callback for row/column detection. */
    Globals.EditProc = ::core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtr(
        Globals.hEdit,
        GWLP_WNDPROC,
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
            LONG_PTR,
        >(Some(
            EDIT_WndProc,
        )),
    ));

    /* Finally shows new edit control and set focus into it. */
    ShowWindow(Globals.hEdit, (SW_SHOW) as i32);
    SetFocus(Globals.hEdit);

    /* Re-arrange controls */
    PostMessageW(
        Globals.hMainWnd,
        WM_SIZE,
        0,
        0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_EditWrap() {
    Globals.bWrapLongLines = (Globals.bWrapLongLines == 0) as BOOL;

    EnableMenuItem(
        Globals.hMenu,
        CMD_GOTO as UINT,
        (if Globals.bWrapLongLines != 0 {
            MF_GRAYED
        } else {
            MF_ENABLED
        }) as UINT,
    );

    DoCreateEditWindow();
    DoShowHideStatusBar();
}

#[no_mangle]
pub unsafe extern "C" fn DIALOG_SelectFont() {
    let mut cf: CHOOSEFONT = core::mem::zeroed();
    let mut lf: LOGFONT = Globals.lfFont;

    ZeroMemory!(
        core::ptr::addr_of_mut!(cf) as *mut core::ffi::c_void,
        core::mem::size_of::<CHOOSEFONT>(),
    );
    cf.lStructSize = core::mem::size_of::<CHOOSEFONT>() as DWORD;
    cf.hwndOwner = Globals.hMainWnd;
    cf.lpLogFont = core::ptr::addr_of_mut!(lf);
    cf.Flags = (CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT | CF_NOVERTFONTS) as DWORD;

    if ChooseFont(core::ptr::addr_of_mut!(cf)) != 0 {
        let mut currfont: HFONT = Globals.hFont;

        Globals.hFont = CreateFontIndirect(core::ptr::addr_of_mut!(lf));
        Globals.lfFont = lf;
        SendMessage(
            Globals.hEdit,
            WM_SETFONT as UINT,
            Globals.hFont as WPARAM,
            TRUE as LPARAM,
        );
        if !currfont.is_null() {
            DeleteObject(currfont as HGDIOBJ);
        }
    }
}

unsafe extern "C" fn DIALOG_SearchDialog(pfnProc: FINDPROC) {
    if !Globals.hFindReplaceDlg.is_null() {
        SetFocus(Globals.hFindReplaceDlg);
        return;
    }

    if Globals.find.lpstrFindWhat.is_null() {
        ZeroMemory!(
            core::ptr::addr_of_mut!(Globals.find) as *mut core::ffi::c_void,
            core::mem::size_of::<FINDREPLACE>(),
        );
        Globals.find.lStructSize = core::mem::size_of::<FINDREPLACE>() as DWORD;
        Globals.find.hwndOwner = Globals.hMainWnd;
        Globals.find.lpstrFindWhat = core::ptr::addr_of_mut!(Globals.szFindText) as *mut TCHAR as LPWSTR;
        Globals.find.wFindWhatLen = _countof!(Globals.szFindText) as WORD;
        Globals.find.lpstrReplaceWith = core::ptr::addr_of_mut!(Globals.szReplaceText) as *mut TCHAR as LPWSTR;
        Globals.find.wReplaceWithLen = _countof!(Globals.szReplaceText) as WORD;
        Globals.find.Flags = FR_DOWN as DWORD;
    }

    /* We only need to create the modal FindReplace dialog which will */
    /* notify us of incoming events using hMainWnd Window Messages    */

    // C `assert`s the dialog was created; a failed FindText/ReplaceText (out of memory)
    // leaves no dialog, which every caller already tolerates -- not a reason to abort.
    Globals.hFindReplaceDlg = pfnProc(core::ptr::addr_of_mut!(Globals.find));
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_Search() {
    DIALOG_SearchDialog(FindTextW);
}

#[no_mangle]
pub unsafe extern "C" fn DIALOG_SearchNext(bDown: BOOL) {
    if bDown != 0 {
        Globals.find.Flags |= FR_DOWN as DWORD;
    } else {
        Globals.find.Flags &= !(FR_DOWN as DWORD);
    }

    if !Globals.find.lpstrFindWhat.is_null() && *Globals.find.lpstrFindWhat != 0 {
        NOTEPAD_FindNext(core::ptr::addr_of_mut!(Globals.find), (FALSE) as BOOL, (TRUE) as BOOL);
    } else {
        DIALOG_Search();
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIALOG_Replace() {
    DIALOG_SearchDialog(ReplaceTextW);
}
unsafe extern "system" fn DIALOG_GoTo_DialogProc(hwndDialog: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut s_pGotoData: PGOTO_DATA = core::ptr::null_mut();

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                s_pGotoData = lParam as PGOTO_DATA;
                SetDlgItemInt(hwndDialog, (ID_LINENUMBER as i32), (*s_pGotoData).iLine, (FALSE) as BOOL);
                return TRUE as INT_PTR; /* Set focus */
            }

            WM_COMMAND => {
                if (LOWORD!(wParam)) as u32 == IDOK {
                    let mut iLine: UINT = GetDlgItemInt(hwndDialog, (ID_LINENUMBER as i32), core::ptr::null_mut(), (FALSE) as BOOL);
                    if iLine <= 0 || (*s_pGotoData).cLines < iLine { /* Out of range */
                        /* Show error message */
                        let mut title: [WCHAR; 128] = [0; 128];
                        let mut text: [WCHAR; 256] = [0; 256];
                        LoadStringW(Globals.hInstance, STRING_NOTEPAD, core::ptr::addr_of_mut!(title) as LPWSTR, _countof!(title) as ::core::ffi::c_int);
                        LoadStringW(Globals.hInstance, STRING_LINE_NUMBER_OUT_OF_RANGE, core::ptr::addr_of_mut!(text) as LPWSTR, _countof!(text) as ::core::ffi::c_int);
                        MessageBoxW(hwndDialog, core::ptr::addr_of_mut!(text) as LPCWSTR, core::ptr::addr_of_mut!(title) as LPCWSTR, MB_OK);

                        SendDlgItemMessageW(hwndDialog, (ID_LINENUMBER as i32), EM_SETSEL, 0, -1);
                        SetFocus(GetDlgItem(hwndDialog, (ID_LINENUMBER as i32)));
                        break 'sw;
                    }
                    (*s_pGotoData).iLine = iLine;
                    EndDialog(hwndDialog, (IDOK) as INT_PTR);
                } else if (LOWORD!(wParam)) as u32 == IDCANCEL {
                    EndDialog(hwndDialog, (IDCANCEL) as INT_PTR);
                }
                break 'sw;
            }

            _ => {}
        }
    }

    return 0 as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_GoTo() {
    let mut GotoData: GOTO_DATA = core::mem::zeroed();
    let mut dwStart: DWORD = 0 as DWORD;
    let mut dwEnd: DWORD = 0 as DWORD;
    let mut ich: INT = 0;
    let mut cch: INT = GetWindowTextLength(Globals.hEdit) as INT;

    /* Get the current line number and the total line number */
    SendMessage(
        Globals.hEdit,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwStart) as WPARAM,
        core::ptr::addr_of_mut!(dwEnd) as LPARAM,
    );
    GotoData.iLine = (SendMessage(
        Globals.hEdit,
        EM_LINEFROMCHAR as UINT,
        dwStart as WPARAM,
        0 as LPARAM,
    ) as UINT)
        .wrapping_add(1 as UINT);
    GotoData.cLines = SendMessage(
        Globals.hEdit,
        EM_GETLINECOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;

    /* Ask the user for line number */
    if DialogBoxParam(
        Globals.hInstance,
        MAKEINTRESOURCE!(DIALOG_GOTO) as LPCWSTR,
        Globals.hMainWnd,
        Some(DIALOG_GoTo_DialogProc),
        core::ptr::addr_of_mut!(GotoData) as LPARAM,
    ) != IDOK as INT_PTR
    {
        return; /* Canceled */
    }

    GotoData.iLine = GotoData.iLine.wrapping_sub(1); /* Make it zero-based */

    /* Get ich (the target character index) from line number */
    if GotoData.iLine <= 0 as UINT {
        ich = 0 as INT;
    } else if GotoData.iLine >= GotoData.cLines {
        ich = cch;
    } else {
        ich = SendMessage(
            Globals.hEdit,
            EM_LINEINDEX as UINT,
            GotoData.iLine as WPARAM,
            0 as LPARAM,
        ) as INT;
    }

    /* EM_LINEINDEX can return -1 on failure */
    if ich < 0 as INT {
        ich = 0 as INT;
    }

    /* Move the caret */
    SendMessage(
        Globals.hEdit,
        EM_SETSEL as UINT,
        ich as WPARAM,
        ich as LPARAM,
    );
    SendMessage(
        Globals.hEdit,
        EM_SCROLLCARET as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_StatusBarUpdateCaretPos() {
    let mut line: ::core::ffi::c_int;
    let mut ich: ::core::ffi::c_int;
    let mut col: ::core::ffi::c_int;
    let mut buff: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwStart: DWORD = 0;
    let mut dwSize: DWORD = 0;

    SendMessage(
        Globals.hEdit,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(dwStart) as WPARAM,
        core::ptr::addr_of_mut!(dwSize) as LPARAM,
    );
    line = SendMessage(
        Globals.hEdit,
        EM_LINEFROMCHAR as UINT,
        dwStart as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    ich = SendMessage(
        Globals.hEdit,
        EM_LINEINDEX as UINT,
        line as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;

    /* EM_LINEINDEX can return -1 on failure */
    col = (if ich < 0 {
        0 as DWORD
    } else {
        dwStart.wrapping_sub(ich as DWORD)
    }) as ::core::ffi::c_int;

    StringCchPrintf!(
        core::ptr::addr_of_mut!(buff) as *mut TCHAR as LPTSTR,
        _countof!(buff),
        core::ptr::addr_of!(Globals.szStatusBarLineCol) as *const TCHAR as LPCTSTR,
        line + 1,
        col + 1,
    );
    SendMessage(
        Globals.hStatusBar,
        SB_SETTEXT as UINT,
        SBPART_CURPOS as WPARAM,
        core::ptr::addr_of_mut!(buff) as *mut TCHAR as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_ViewStatusBar() {
    Globals.bShowStatusBar = (Globals.bShowStatusBar == 0) as i32;
    DoShowHideStatusBar();
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_HelpContents() {
    WinHelp(Globals.hMainWnd, core::ptr::addr_of!(helpfile) as LPCTSTR, HELP_INDEX, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_HelpAboutNotepad() {
    let mut szNotepad: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];
    let mut szNotepadAuthors: [TCHAR; (MAX_STRING_LEN as usize)] = [0; (MAX_STRING_LEN as usize)];

    LoadString(Globals.hInstance, STRING_NOTEPAD as UINT, core::ptr::addr_of_mut!(szNotepad) as LPTSTR, _countof!(szNotepad) as ::core::ffi::c_int);
    LoadString(Globals.hInstance, STRING_NOTEPAD_AUTHORS as UINT, core::ptr::addr_of_mut!(szNotepadAuthors) as LPTSTR, _countof!(szNotepadAuthors) as ::core::ffi::c_int);

    ShellAbout(
        Globals.hMainWnd,
        core::ptr::addr_of_mut!(szNotepad) as *mut TCHAR as LPCTSTR,
        core::ptr::addr_of_mut!(szNotepadAuthors) as *mut TCHAR as LPCTSTR,
        LoadIcon(Globals.hInstance, MAKEINTRESOURCE!(IDI_NPICON) as LPCTSTR),
    );
}
