use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn cmd_alloc(size: usize) -> *mut core::ffi::c_void;
    fn cmd_free(p: *mut core::ffi::c_void);
}
extern "system" {
    fn CmdTrace(level: INT, file: LPCSTR, line: INT, func: LPCSTR, fmt: LPCSTR);
    fn ZeroMemory(Destination: *mut core::ffi::c_void, Length: usize);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut InputCodePage: UINT = 0;

#[no_mangle]
pub static mut OutputCodePage: UINT = 0;

#[no_mangle]
pub static mut StdOutScreen: CON_SCREEN = unsafe {
    _CON_SCREEN {
        Stream: &raw const csStdOut as PCON_STREAM,
        csbi: _CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: _COORD { X: 0, Y: 0 },
            dwCursorPosition: _COORD { X: 0, Y: 0 },
            wAttributes: 0,
            srWindow: _SMALL_RECT {
                Left: 0,
                Top: 0,
                Right: 0,
                Bottom: 0,
            },
            dwMaximumWindowSize: _COORD { X: 0, Y: 0 },
        },
        cci: _CONSOLE_CURSOR_INFO {
            dwSize: 0,
            bVisible: 0,
        },
    }
};

#[no_mangle]
pub static mut StdOutPager: CON_PAGER = unsafe {
    _CON_PAGER {
        Screen: &raw const StdOutScreen as PCON_SCREEN,
        PageColumns: 0 as DWORD,
        PageRows: 0,
        PagerLine: None,
        dwFlags: 0,
        nTabWidth: 0,
        ScrollRows: 0,
        CachedLine: ::core::ptr::null::<WCHAR>(),
        cchCachedLine: 0,
        ich: 0,
        CurrentLine: ::core::ptr::null::<WCHAR>(),
        ichCurr: 0,
        iEndLine: 0,
        nSpacePending: 0,
        iColumn: 0,
        iLine: 0,
        lineno: 0,
    }
};

pub const OUTPUT_BUFFER_SIZE: i32 = 4096;

// INIT_CON_SCREEN(StdOut) expansion was not provided; zero-initialized placeholder.

// INIT_CON_PAGER(&StdOutScreen) expansion was not provided; zero-initialized placeholder.
// The pager's screen link must be set up to point at StdOutScreen before use.
#[no_mangle]
pub unsafe extern "C" fn ConInDisable() {
    let hInput: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let mut dwMode: DWORD = 0;

    GetConsoleMode(hInput, core::ptr::addr_of_mut!(dwMode));
    dwMode &= !(ENABLE_PROCESSED_INPUT as DWORD);
    SetConsoleMode(hInput, dwMode);
}
#[no_mangle]
pub unsafe extern "C" fn ConInEnable() {
    let hInput: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let mut dwMode: DWORD = 0;

    GetConsoleMode(hInput, core::ptr::addr_of_mut!(dwMode));
    dwMode |= ENABLE_PROCESSED_INPUT as DWORD;
    SetConsoleMode(hInput, dwMode);
}
#[no_mangle]
pub unsafe extern "C" fn ConInFlush() {
    FlushConsoleInputBuffer(GetStdHandle(STD_INPUT_HANDLE));
}
#[no_mangle]
pub unsafe extern "C" fn ConInKey(lpBuffer: PINPUT_RECORD) {
    let hInput: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let mut dwRead: DWORD = 0;

    if hInput == INVALID_HANDLE_VALUE {
        CmdTrace(
            __WINE_DBCL_WARN as INT,
            b"base/shell/cmd/console.c\0".as_ptr() as LPCSTR,
            67 as INT,
            b"ConInKey\0".as_ptr() as LPCSTR,
            b"Invalid input handle!!!\n\0".as_ptr() as LPCSTR,
        );
    }

    loop {
        ReadConsoleInput(
            hInput,
            lpBuffer,
            1,
            core::ptr::addr_of_mut!(dwRead),
        );
        if (*lpBuffer).EventType == KEY_EVENT
            && (*lpBuffer).Event.KeyEvent.bKeyDown != 0
        {
            break;
        }
        if !(TRUE) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ConInString(lpInput: LPWSTR, dwLength: DWORD) {
    let mut dwOldMode: DWORD;
    let mut dwRead: DWORD = 0;
    let mut hFile: HANDLE;

    let mut p: LPWSTR;
    let mut pBuf: PCHAR;

    pBuf = cmd_alloc(dwLength.wrapping_sub(1) as usize) as PCHAR;

    ZeroMemory(lpInput as *mut core::ffi::c_void, dwLength as usize * core::mem::size_of::<WCHAR>());
    hFile = GetStdHandle(STD_INPUT_HANDLE);
    GetConsoleMode(hFile, core::ptr::addr_of_mut!(dwOldMode));

    SetConsoleMode(hFile, ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);

    ReadFile(
        hFile,
        pBuf as PVOID,
        dwLength.wrapping_sub(1),
        core::ptr::addr_of_mut!(dwRead),
        core::ptr::null_mut(),
    );

    MultiByteToWideChar(
        InputCodePage,
        0,
        pBuf as PCSTR,
        dwRead as i32,
        lpInput,
        dwLength.wrapping_sub(1) as i32,
    );
    cmd_free(pBuf as *mut core::ffi::c_void);

    p = lpInput;
    while *p != 0 {
        if *p == 0x000Du16 {
            // Terminate at the carriage-return.
            *p = 0x0000u16;
            break;
        }
        p = p.add(1);
    }

    SetConsoleMode(hFile, dwOldMode);
}
#[no_mangle]
pub unsafe extern "C" fn ConOutChar(mut c: WCHAR) {
    ConWrite(core::ptr::addr_of_mut!(csStdOut), core::ptr::addr_of_mut!(c) as PCTCH, 1 as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn ConErrChar(mut c: WCHAR) {
    ConWrite(StdErr, core::ptr::addr_of_mut!(c) as PCTCH, 1);
}
#[no_mangle]
pub unsafe extern "C" fn ConFormatMessage(mut Stream: PCON_STREAM, mut MessageId: DWORD, mut args: ...) {
    let mut Len: INT = 0;
    let mut arg_ptr: va_list;
    va_start(arg_ptr, MessageId);
    Len = ConMsgPrintfV(
        Stream,
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut(),
        MessageId,
        LANG_USER_DEFAULT as DWORD,
        &raw mut arg_ptr,
    );
    va_end(arg_ptr);

    if Len <= 0 as INT {
        ConResPrintf(Stream, STRING_CONSOLE_ERROR as UINT, MessageId);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ConPrintfVPaging(
    mut Pager: PCON_PAGER,
    mut StartPaging: BOOL,
    mut szFormat: LPWSTR,
    mut arg_ptr: va_list,
) -> BOOL {
    let mut szOut: [WCHAR; OUTPUT_BUFFER_SIZE] = [0; OUTPUT_BUFFER_SIZE];

    /* Return if no string has been given */
    if szFormat.is_null() {
        return TRUE;
    }

    /*len =*/
    _vswprintf(
        core::ptr::addr_of_mut!(szOut) as *mut WCHAR,
        szFormat as *const WCHAR,
        arg_ptr.as_va_list(),
    );

    // return ConPutsPaging(Pager, PagePrompt, StartPaging, szOut);
    return ConWritePaging(
        Pager,
        Some(PagePrompt as unsafe extern "C" fn(PCON_PAGER, DWORD, DWORD) -> BOOL),
        StartPaging,
        core::ptr::addr_of_mut!(szOut) as *mut WCHAR as PCTCH,
        wcslen(core::ptr::addr_of_mut!(szOut) as *mut WCHAR) as SIZE_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ConOutPrintfPaging(StartPaging: BOOL, szFormat: LPWSTR, mut args: ...) -> BOOL {
    let mut bRet: BOOL;
    let mut arg_ptr: va_list;

    va_start(arg_ptr, szFormat);
    bRet = ConPrintfVPaging(core::ptr::addr_of_mut!(StdOutPager), StartPaging, szFormat, arg_ptr);
    va_end(arg_ptr);
    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn ConOutResPaging(StartPaging: BOOL, resID: UINT) {
    ConResPaging(
        core::ptr::addr_of_mut!(StdOutPager),
        Some(PagePrompt as unsafe extern "C" fn(PCON_PAGER, DWORD, DWORD) -> BOOL),
        StartPaging,
        resID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SetCursorXY(mut x: SHORT, mut y: SHORT) {
    let mut coPos: COORD = COORD { X: 0, Y: 0 };

    coPos.X = x;
    coPos.Y = y;
    SetConsoleCursorPosition(GetStdHandle(STD_OUTPUT_HANDLE), coPos);
}
#[no_mangle]
pub unsafe extern "C" fn GetCursorXY(x: PSHORT, y: PSHORT) {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();

    GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(csbi));

    *x = csbi.dwCursorPosition.X;
    *y = csbi.dwCursorPosition.Y;
}
#[no_mangle]
pub unsafe extern "C" fn GetCursorX() -> SHORT {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();

    GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(csbi));
    return csbi.dwCursorPosition.X;
}
#[no_mangle]
pub unsafe extern "C" fn GetCursorY() -> SHORT {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(csbi));
    return csbi.dwCursorPosition.Y;
}
#[no_mangle]
pub unsafe extern "C" fn SetCursorType(bInsert: BOOL, bVisible: BOOL) {
    let mut cci: CONSOLE_CURSOR_INFO = core::mem::zeroed();

    cci.dwSize = if bInsert != 0 { 10 } else { 99 };
    cci.bVisible = bVisible;

    SetConsoleCursorInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(cci));
}
#[no_mangle]
pub unsafe extern "C" fn GetScreenSize(maxx: PSHORT, maxy: PSHORT) {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();

    if GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), core::ptr::addr_of_mut!(csbi)) == 0 {
        csbi.dwSize.X = 80 as SHORT;
        csbi.dwSize.Y = 25 as SHORT;
    }

    if !maxx.is_null() {
        *maxx = csbi.dwSize.X;
    }
    if !maxy.is_null() {
        *maxy = csbi.dwSize.Y;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ConGetDefaultAttributes(pwDefAttr: PWORD) -> BOOL {
    let mut Success: BOOL = 0;
    let mut hConsole: HANDLE = core::ptr::null_mut();
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let CONOUT: [u16; 8] = ['C' as u16, 'O' as u16, 'N' as u16, 'O' as u16, 'U' as u16, 'T' as u16, '$' as u16, 0];

    /* Do not modify *pwDefAttr if we fail, in which case use default attributes */

    hConsole = CreateFile(CONOUT.as_ptr() as LPCWSTR, GENERIC_READ | GENERIC_WRITE,
                          FILE_SHARE_READ | FILE_SHARE_WRITE, core::ptr::null_mut(),
                          OPEN_EXISTING, 0, core::ptr::null_mut());
    if hConsole == INVALID_HANDLE_VALUE {
        return FALSE; // No default console
    }

    Success = GetConsoleScreenBufferInfo(hConsole, &mut csbi);
    if Success != 0 {
        *pwDefAttr = csbi.wAttributes;
    }

    CloseHandle(hConsole);
    return Success;
}

#[no_mangle]
pub unsafe extern "C" fn ConSetTitle(lpConsoleTitle: LPCWSTR) -> BOOL {
    SetConsoleTitle(lpConsoleTitle)
}
#[no_mangle]
pub unsafe extern "C" fn ConRingBell(hOutput: HANDLE) {
    MessageBeep(-1 as UINT);
}
#[no_mangle]
pub unsafe extern "C" fn ConSetScreenColor(hOutput: HANDLE, wColor: WORD, bFill: BOOL) -> BOOL {
    let mut dwWritten: DWORD = 0;
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut coPos: COORD = core::mem::zeroed();

    /* Foreground and Background colors can't be the same */
    if (wColor as i32 & 0x0F) == ((wColor as i32 & 0xF0) >> 4) {
        return FALSE;
    }

    /* Fill the whole background if needed */
    if bFill != 0 {
        GetConsoleScreenBufferInfo(hOutput, core::ptr::addr_of_mut!(csbi));

        coPos.X = 0 as SHORT;
        coPos.Y = 0 as SHORT;
        FillConsoleOutputAttribute(
            hOutput,
            (wColor as i32 & 0x00FF) as WORD,
            (csbi.dwSize.X as i32 * csbi.dwSize.Y as i32) as DWORD,
            coPos,
            core::ptr::addr_of_mut!(dwWritten),
        );
    }

    /* Set the text attribute */
    SetConsoleTextAttribute(hOutput, (wColor as i32 & 0x00FF) as WORD);
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn ConGetTextWidthW(pszText: PCWSTR) -> SIZE_T {
    let mut ich: SIZE_T;
    let mut cxWidth: SIZE_T;

    if !IsCJKCodePage(OutputCodePage) {
        return _tcslen(pszText);
    }

    cxWidth = 0;
    ich = cxWidth;
    while *pszText.add(ich) != 0 {
        if IS_FULL_WIDTH(*pszText.add(ich)) {
            cxWidth += 2;
        } else {
            cxWidth += 1;
        }
        ich += 1;
    }

    return cxWidth;
}
#[no_mangle]
pub unsafe extern "C" fn ConGetTextWidthA(pszText: PCSTR) -> SIZE_T {
    let mut cchMax: ::core::ffi::c_int = 0;
    let mut pszWide: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut cxWidth: SIZE_T = 0;
    cchMax = MultiByteToWideChar(
        OutputCodePage,
        0 as DWORD,
        pszText as LPCSTR,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    );
    pszWide = cmd_alloc(
        (cchMax as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    MultiByteToWideChar(
        OutputCodePage,
        0 as DWORD,
        pszText as LPCSTR,
        -(1 as ::core::ffi::c_int),
        pszWide as LPWSTR,
        cchMax,
    );
    cxWidth = ConGetTextWidthW(pszWide as PCWSTR);
    cmd_free(pszWide as *mut ::core::ffi::c_void);
    return cxWidth;
}
