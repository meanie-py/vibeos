use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut bCtrlBreak: BOOL = (FALSE as i32);
#[no_mangle]
pub unsafe extern "C" fn GetRootPath(mut InPath: LPCTSTR, mut OutPath: LPTSTR, mut size: INT) -> INT {
    if *InPath.offset(0 as isize) != 0
        && *InPath.offset(1 as isize) == (':' as u16)
    {
        let mut t: INT = 0 as INT;

        if (*InPath.offset(0 as isize) >= ('0' as u16))
            && (*InPath.offset(0 as isize) <= ('9' as u16))
        {
            t = ((*InPath.offset(0 as isize) as INT)
                - (('0' as u16) as INT))
                + (28 as INT);
        } else if (*InPath.offset(0 as isize) >= ('a' as u16))
            && (*InPath.offset(0 as isize) <= ('z' as u16))
        {
            t = ((*InPath.offset(0 as isize) as INT)
                - (('a' as u16) as INT))
                + (1 as INT);
        } else if (*InPath.offset(0 as isize) >= ('A' as u16))
            && (*InPath.offset(0 as isize) <= ('Z' as u16))
        {
            t = ((*InPath.offset(0 as isize) as INT)
                - (('A' as u16) as INT))
                + (1 as INT);
        }

        return (_tgetdcwd(t, OutPath, size) == core::ptr::null_mut()) as INT;
    }

    /* Get current directory */
    return (GetCurrentDirectory(size as DWORD, OutPath) == 0) as INT;
}
#[no_mangle]
pub unsafe extern "C" fn GetPathCase(Path: *mut TCHAR, OutPath: *mut TCHAR) {
    let mut i: UINT = 0;
    let mut TempPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FindFileData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();

    _tcscpy(TempPath.as_mut_ptr(), (&[0x0000u16]).as_ptr() /* _T "" */);
    _tcscpy(OutPath, (&[0x0000u16]).as_ptr() /* _T "" */);

    i = 0;
    while (i as usize) < _tcslen(Path) {
        'body: {
            if *Path.add(i as usize) != ('\\' as u16) {
                _tcsncat(TempPath.as_mut_ptr(), Path.add(i as usize), 1);
                if (i as usize) != _tcslen(Path) - 1 {
                    break 'body;
                }
            }
            /* Handle the base part of the path different.
               Because if you put it into findfirstfile, it will
               return your current folder */
            if _tcslen(TempPath.as_mut_ptr()) == 2
                && TempPath[1] == (':' as u16)
            {
                _tcscat(OutPath, TempPath.as_mut_ptr());
                _tcscat(OutPath, (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
                _tcscat(TempPath.as_mut_ptr(), (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
            } else {
                hFind = FindFirstFile(TempPath.as_mut_ptr(), core::ptr::addr_of_mut!(FindFileData));
                if hFind == INVALID_HANDLE_VALUE {
                    _tcscpy(OutPath, Path);
                    return;
                }
                _tcscat(TempPath.as_mut_ptr(), (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
                _tcscat(OutPath, FindFileData.cFileName.as_mut_ptr());
                _tcscat(OutPath, (&[0x005c, 0x0000u16]).as_ptr() /* _T "\\" */);
                FindClose(hFind);
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn IsExistingFile(pszPath: LPCTSTR) -> BOOL {
    let attr: DWORD = GetFileAttributes(pszPath);
    return (((attr != INVALID_FILE_ATTRIBUTES) && ((attr & FILE_ATTRIBUTE_DIRECTORY) == 0)) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn IsExistingDirectory(pszPath: LPCTSTR) -> BOOL {
    let attr: DWORD = GetFileAttributes(pszPath);
    return ((attr != INVALID_FILE_ATTRIBUTES) && ((attr & FILE_ATTRIBUTE_DIRECTORY) != 0)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn FilePromptYNA(resID: UINT) -> INT {
    let mut szMsg: [TCHAR; 4] = [0; 4];
//  TCHAR cKey = 0;
//  LPTSTR szKeys = (&[0x0079, 0x006e, 0x0061, 0x0000u16]).as_ptr() /* _T "yna" */;

    let mut szIn: [TCHAR; 10] = [0; 10];
    let mut p: LPTSTR;

    if resID != 0 as UINT {
        ConOutResPrintf!(resID);
    }

    /* preliminary fix */
    ConInString(szIn.as_mut_ptr(), 10 as DWORD);

    _tcsupr(szIn.as_mut_ptr());
    p = szIn.as_mut_ptr();
    while _istspace!(*p) != 0 {
        p = p.offset(1);
    }

    LoadStringW(
        core::ptr::null_mut(),
        STRING_COPY_OPTION as UINT,
        szMsg.as_mut_ptr(),
        ARRAYSIZE!(szMsg) as INT,
    );

    if _tcsncmp(
        p,
        szMsg.as_mut_ptr().add(0 as usize),
        1 as usize,
    ) == 0
    {
        return (PROMPT_YES) as INT;
    } else if _tcsncmp(
        p,
        szMsg.as_mut_ptr().add(1 as usize),
        1 as usize,
    ) == 0
    {
        return (PROMPT_NO) as INT;
    } else if _tcsncmp(
        p,
        szMsg.as_mut_ptr().add(2 as usize),
        1 as usize,
    ) == 0
    {
        return (PROMPT_ALL) as INT;
    }

    return (PROMPT_NO) as INT;

    /* unfinished solution */
}
#[no_mangle]
pub unsafe extern "C" fn ConInString(mut lpInput: LPTSTR, mut dwLength: DWORD) {
    let mut dwOldMode: DWORD = 0;
    let mut dwRead: DWORD = 0;
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut p: LPTSTR = core::ptr::null_mut();
    let mut pBuf: PCHAR = core::ptr::null_mut();

    pBuf = malloc(dwLength.wrapping_sub(1) as usize) as PCHAR;
    ZeroMemory!(
        lpInput as PVOID,
        dwLength.wrapping_mul(core::mem::size_of::<TCHAR>() as DWORD),
    );
    hFile = GetStdHandle(STD_INPUT_HANDLE);
    GetConsoleMode(hFile, core::ptr::addr_of_mut!(dwOldMode));

    SetConsoleMode(
        hFile,
        ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT,
    );

    ReadFile(
        hFile,
        pBuf as PVOID,
        dwLength.wrapping_sub(1),
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::null_mut(),
    );

    MultiByteToWideChar(
        GetConsoleCP(),
        0,
        pBuf as LPCSTR,
        dwRead as i32,
        lpInput as LPWSTR,
        dwLength.wrapping_sub(1) as i32,
    );
    free(pBuf as *mut core::ffi::c_void);

    p = lpInput;
    while *p != 0 {
        if *p == ('\r' as u16) {
            *p = ('\0' as u16);
            break;
        } else {
            p = p.offset(1);
        }
    }

    SetConsoleMode(hFile, dwOldMode);
}
#[no_mangle]
pub unsafe extern "C" fn msg_pause() {
    ConOutResPuts!(STRING_ERROR_D_PAUSEMSG);
}
#[no_mangle]
pub unsafe extern "C" fn ConFormatMessage_v(mut Stream: PCON_STREAM, mut MessageId: DWORD, ap_list: *mut ::core::ffi::c_void) {
    let mut Len: INT = 0;
    let mut arg_ptr: va_list = *(ap_list as *mut va_list);
    Len = ConMsgPrintfV(
        (Stream).cast(),
        FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null_mut(),
        MessageId,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(arg_ptr),
    );

    if Len <= 0 {
        ConResPrintf((Stream).cast(), STRING_CONSOLE_ERROR, MessageId);
    }
}


#[no_mangle]
pub unsafe extern "C" fn ConOutChar(mut c: TCHAR) {
    ConWrite(StdOut, (core::ptr::addr_of_mut!(c) as PCTCH).cast(), 1 as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn cgetchar() -> TCHAR {
    let hInput: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let mut irBuffer: INPUT_RECORD = core::mem::zeroed();
    let mut dwRead: DWORD = 0;

    loop {
        ReadConsoleInput(hInput, core::ptr::addr_of_mut!(irBuffer), 1, core::ptr::addr_of_mut!(dwRead));
        if (irBuffer.EventType as ::core::ffi::c_int == (KEY_EVENT as i32))
            && (irBuffer.Event.KeyEvent.bKeyDown != (FALSE as i32))
        {
            if (irBuffer.Event.KeyEvent.dwControlKeyState
                & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) as DWORD)
                != 0
            {
                if irBuffer.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int == 'C' as i32 {
                    bCtrlBreak = (TRUE as i32);
                    break;
                }
            } else if ((irBuffer.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int) as u32 == VK_SHIFT)
                || ((irBuffer.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int) as u32 == VK_MENU)
                || ((irBuffer.Event.KeyEvent.wVirtualKeyCode as ::core::ffi::c_int) as u32 == VK_CONTROL)
            {
                // Nothing to do
            } else {
                break;
            }
        }
        if (((TRUE)) == 0) {
            break;
        }
    }

    return irBuffer.Event.KeyEvent.uChar.UnicodeChar as TCHAR;
}
