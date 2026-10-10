use crate::cmd::batch::Batch;
use crate::cmd::batch::FindArg;
use crate::cmd::batch::ReadBatchLine;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut bExit: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bCanExit: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bCtrlBreak: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bIgnoreEcho: BOOL = (FALSE) as i32;

static mut fSingleCommand: BOOL = 0 as BOOL;

static mut bAlwaysStrip: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut nErrorLevel: INT = 0 as INT;

#[no_mangle]
pub static mut ChildProcessRunningLock: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

#[no_mangle]
pub static mut bDisableBatchEcho: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bEnableExtensions: BOOL = (TRUE) as i32;

#[no_mangle]
pub static mut bDelayedExpansion: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut dwChildProcessId: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut lpOriginalEnvironment: LPTSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut CMD_ModuleHandle: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut bTitleSet: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut szCurTitle: [TCHAR; 260] = [0; 260];

static mut NtQueryInformationProcessPtr: NtQueryInformationProcessProc = None;

static mut NtReadVirtualMemoryPtr: NtReadVirtualMemoryProc = None;

#[no_mangle]
pub static mut OutputStreamMode: CON_STREAM_MODE = UTF8Text;

#[no_mangle]
pub static mut wDefColor: WORD = 0 as WORD;

pub const SHELLEXECUTETEXT: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"ShellExecuteExW\0") };

// C's function-pointer typedef: nullable, and checked for NULL before each call.
pub type NtQueryInformationProcessProc = Option<unsafe extern "system" fn(
    HANDLE,
    PROCESSINFOCLASS,
    PVOID,
    ULONG,
    PULONG,
) -> NTSTATUS>;

// C's function-pointer typedef: nullable, and checked for NULL before each call.
pub type NtReadVirtualMemoryProc = Option<unsafe extern "system" fn(
    HANDLE,
    PVOID,
    PVOID,
    SIZE_T,
    PSIZE_T,
) -> NTSTATUS>;

// INCLUDE_CMD_COLOR block (wDefColor) emitted unconditionally

// SHELLEXECUTETEXT (_UNICODE branch)

pub type MYEX = unsafe extern "system" fn(LPSHELLEXECUTEINFO) -> BOOL;
#[no_mangle]
pub unsafe extern "C" fn ConvertULargeInteger(mut num: ULONGLONG, mut des: LPTSTR, mut len: UINT, mut bPutSeparator: BOOL) -> INT {
    let mut temp: [TCHAR; 39] = [0; 39];
    let mut n: UINT = 0;
    let mut iTarget: UINT = 0;

    if len <= (1 as UINT) {
        return 0 as INT;
    }

    n = 0 as UINT;
    iTarget = nNumberGroups as UINT;
    if nNumberGroups == 0 {
        bPutSeparator = FALSE as BOOL;
    }

    loop {
        if iTarget == n && bPutSeparator != 0 {
            iTarget = iTarget.wrapping_add(
                (nNumberGroups as UINT).wrapping_add(1 as UINT),
            );
            let fresh31 = n;
            n = n.wrapping_add(1);
            temp[((38 as UINT).wrapping_sub(fresh31)) as usize] =
                cThousandSeparator;
        }
        let fresh32 = n;
        n = n.wrapping_add(1);
        temp[((38 as UINT).wrapping_sub(fresh32)) as usize] =
            ((((num % (10 as ULONGLONG)) as TCHAR)) + ('0' as u16)) as TCHAR;
        num = num / (10 as ULONGLONG);
        if !(num > (0 as ULONGLONG)) {
            break;
        }
    }

    if n > len.wrapping_sub(1 as UINT) {
        n = len.wrapping_sub(1 as UINT);
    }

    memcpy(
        des as *mut ::core::ffi::c_void,
        ((temp.as_mut_ptr() as *const TCHAR).add(
            (39 as usize).wrapping_sub(n as usize),
        )) as *const ::core::ffi::c_void,
        (n as size_t).wrapping_mul(::core::mem::size_of::<TCHAR>() as size_t),
    );
    *des.add(n as usize) = ('\0' as u16);

    return n as INT;
}

unsafe extern "C" fn IsConsoleProcess(mut Process: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Info: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut ProcessPeb: PEB = core::mem::zeroed();
    let mut BytesRead: SIZE_T = 0;

    if NtQueryInformationProcessPtr.is_none() || NtReadVirtualMemoryPtr.is_none() {
        return (TRUE) as BOOL;
    }

    Status = NtQueryInformationProcessPtr.unwrap_unchecked()(
        Process,
        ProcessBasicInformation,
        core::ptr::addr_of_mut!(Info) as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if !(NT_SUCCESS!(Status)) {
        return (TRUE) as BOOL;
    }
    Status = NtReadVirtualMemoryPtr.unwrap_unchecked()(
        Process,
        Info.PebBaseAddress as PVOID,
        core::ptr::addr_of_mut!(ProcessPeb) as PVOID,
        core::mem::size_of::<PEB>() as SIZE_T,
        core::ptr::addr_of_mut!(BytesRead),
    );
    if !(NT_SUCCESS!(Status)) || core::mem::size_of::<PEB>() != BytesRead as usize {
        return (TRUE) as BOOL;
    }

    return (IMAGE_SUBSYSTEM_WINDOWS_CUI as ULONG == ProcessPeb.ImageSubsystem as ULONG) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RunFile(mut flags: DWORD, mut filename: LPTSTR, mut params: LPTSTR, mut directory: LPTSTR, mut show: INT) -> HANDLE {
    let mut sei: SHELLEXECUTEINFO = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFO>() as DWORD;
    sei.fMask = flags | SEE_MASK_FLAG_DDEWAIT;
    let mut hShell32: HMODULE;
    let mut hShExt: MYEX;
    let mut err: UINT;
    let SHELL32_NAME: [u16; 12] = ['S' as u16, 'H' as u16, 'E' as u16, 'L' as u16, 'L' as u16, '3' as u16, '2' as u16, '.' as u16, 'D' as u16, 'L' as u16, 'L' as u16, 0];

    hShell32 = LoadLibrary(SHELL32_NAME.as_ptr()) as HMODULE;
    if hShell32.is_null() {
        return core::ptr::null_mut();
    }

    hShExt = ::core::mem::transmute::<FARPROC, MYEX>(GetProcAddress(hShell32 as _, (SHELLEXECUTETEXT).as_ptr()));
    if (hShExt as usize) == 0 {
        FreeLibrary(hShell32);
        return core::ptr::null_mut();
    }

    sei.lpFile = filename;
    sei.lpParameters = params;
    sei.lpDirectory = directory;
    sei.nShow = show;
    err = (if hShExt(&raw mut sei) != 0 { ERROR_SUCCESS } else { GetLastError() }) as UINT;

    FreeLibrary(hShell32);

    SetLastError(err as DWORD);
    return if err != 0 { core::ptr::null_mut() } else { sei.hProcess };
}

unsafe extern "C" fn SetConTitle(pszTitle: LPCTSTR) {
    let mut szNewTitle: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    if pszTitle.is_null() {
        return;
    }
    /* Don't do anything if we run inside a batch file, or we are just running a single command */
    if !bc.is_null() || fSingleCommand == 1 {
        return;
    }
    /* Save the original console title and build a new one */
    GetConsoleTitle(
        core::ptr::addr_of_mut!(szCurTitle) as LPTSTR,
        ARRAYSIZE!(*core::ptr::addr_of!(szCurTitle)) as DWORD,
    );
    let FMT: [u16; 8] = [
        '%' as u16, 's' as u16, ' ' as u16, '-' as u16,
        ' ' as u16, '%' as u16, 's' as u16, 0,
    ];
    StringCchPrintf!(
        core::ptr::addr_of_mut!(szNewTitle) as LPTSTR,
        ARRAYSIZE!(*core::ptr::addr_of!(szNewTitle)) as size_t,
        (FMT.as_ptr() as LPCTSTR),
        core::ptr::addr_of!(szCurTitle) as LPCTSTR,
        pszTitle,
    );
    bTitleSet = (TRUE) as i32;
    ConSetTitle(core::ptr::addr_of!(szNewTitle) as LPCTSTR);
}

unsafe extern "C" fn ResetConTitle() {
    if bc.is_null() && bTitleSet != 0 {
        ConSetTitle(core::ptr::addr_of_mut!(szCurTitle) as *mut TCHAR as LPCTSTR);
        bTitleSet = (FALSE) as i32;
    }
}

unsafe extern "C" fn Execute(Full: LPTSTR, First: LPTSTR, Rest: LPTSTR, Cmd: *mut PARSED_COMMAND) -> INT {
    let mut first: *mut TCHAR = core::ptr::null_mut();
    let mut rest: *mut TCHAR = core::ptr::null_mut();
    let mut dot: *mut TCHAR = core::ptr::null_mut();
    let mut dwExitCode: DWORD = 0;
    let mut FirstEnd: *mut TCHAR = core::ptr::null_mut();
    let mut szFullName: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szFullCmdLine: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut comma_set: [TCHAR; 5] = [',' as TCHAR, ';' as TCHAR, '=' as TCHAR, '/' as TCHAR, 0];
    let mut colon_s: [TCHAR; 2] = [':' as TCHAR, 0];
    let mut dot_bat: [TCHAR; 5] = ['.' as TCHAR, 'b' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 0];
    let mut dot_cmd: [TCHAR; 5] = ['.' as TCHAR, 'c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, 0];
    let mut quote_s: [TCHAR; 2] = ['"' as TCHAR, 0];
    let mut empty_s: [TCHAR; 1] = [0];
    let mut space_s: [TCHAR; 2] = [' ' as TCHAR, 0];

    /* Though it was already parsed once, we have a different set of rules
       for parsing before we pass to CreateProcess */
    if *First.offset(0 as isize) == '/' as TCHAR
        || (*First.offset(0 as isize) != 0
            && *First.offset(1 as isize) == ':' as TCHAR)
    {
        /* Use the entire first word as the program name (no change) */
        FirstEnd = First.offset(_tcslen(First) as isize);
    } else {
        /* If present in the first word, spaces and ,;=/ end the program
         * name and become the beginning of its parameters. */
        let mut bInside: BOOL = (FALSE) as i32;
        FirstEnd = First;
        while *FirstEnd != 0 {
            if bInside == 0
                && (_istspace!(*FirstEnd) != 0
                    || !_tcschr(
                        core::ptr::addr_of_mut!(comma_set) as *mut TCHAR,
                        *FirstEnd,
                    )
                    .is_null())
            {
                break;
            }
            bInside ^= (*FirstEnd == '"' as TCHAR) as BOOL;
            FirstEnd = FirstEnd.offset(1);
        }
    }

    /* Copy the new first/rest into the buffer */
    rest = Full.offset(FirstEnd.offset_from(First) + 1);
    _tcscpy(rest, FirstEnd);
    _tcscat(rest, Rest);
    first = Full;
    *FirstEnd = 0 as TCHAR;
    _tcscpy(first, First);

    /* check for a drive change */
    if _istalpha!(*first.offset(0 as isize)) != 0
        && (_tcscmp(
            first.offset(1 as isize),
            core::ptr::addr_of_mut!(colon_s) as *mut TCHAR,
        ) == 0)
    {
        let mut working: BOOL = (TRUE) as i32;
        if SetCurrentDirectory(first) == 0 {
            /* Guess they changed disc or something, handle that gracefully and get to root */
            let mut str: [TCHAR; 4] = [0; 4];
            str[0 as usize] = *first.offset(0 as isize);
            str[1 as usize] = ':' as TCHAR;
            str[2 as usize] = '\\' as TCHAR;
            str[3 as usize] = 0;
            working = SetCurrentDirectory(core::ptr::addr_of_mut!(str) as *mut TCHAR);
        }

        if working == 0 {
            ConErrResPuts!(STRING_FREE_ERROR1);
        }
        return (working == 0) as INT;
    }

    /* get the PATH environment variable and parse it */
    /* search the PATH environment variable for the binary */
    StripQuotes(First);
    if SearchForExecutable(First, core::ptr::addr_of_mut!(szFullName) as *mut TCHAR) == 0 {
        error_bad_command(first);
        return 1;
    }

    /* Set the new console title */
    FirstEnd = first.offset(FirstEnd.offset_from(First)); /* Point to the separating NULL in the full built string */
    *FirstEnd = ' ' as TCHAR;
    SetConTitle(Full);

    /* check if this is a .BAT or .CMD file */
    dot = _tcsrchr(core::ptr::addr_of_mut!(szFullName) as *mut TCHAR, '.' as TCHAR);
    if !dot.is_null()
        && (_tcsicmp(dot, core::ptr::addr_of_mut!(dot_bat) as *mut TCHAR) == 0
            || _tcsicmp(dot, core::ptr::addr_of_mut!(dot_cmd) as *mut TCHAR) == 0)
    {
        while *rest == ' ' as TCHAR {
            rest = rest.offset(1);
        }

        *FirstEnd = 0 as TCHAR;
        dwExitCode = Batch(
            core::ptr::addr_of_mut!(szFullName) as *mut TCHAR,
            first,
            rest,
            Cmd,
        ) as DWORD;
    } else {
        /* exec the program */
        let mut prci: PROCESS_INFORMATION = core::mem::zeroed();
        let mut stui: STARTUPINFO = core::mem::zeroed();
        let mut execerror: UINT = ERROR_FILE_NOT_FOUND;

        /* build command line for CreateProcess(): FullName + " " + rest */
        let mut quoted: BOOL = (!_tcschr(First, ' ' as TCHAR).is_null()) as BOOL;
        _tcscpy(
            core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
            if quoted != 0 {
                core::ptr::addr_of_mut!(quote_s) as *mut TCHAR
            } else {
                core::ptr::addr_of_mut!(empty_s) as *mut TCHAR
            },
        );
        _tcsncat(
            core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
            First,
            (CMDLINE_LENGTH as usize)
                .wrapping_sub(_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR) as usize)
                .wrapping_sub(1 as usize),
        );
        _tcsncat(
            core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
            if quoted != 0 {
                core::ptr::addr_of_mut!(quote_s) as *mut TCHAR
            } else {
                core::ptr::addr_of_mut!(empty_s) as *mut TCHAR
            },
            (CMDLINE_LENGTH as usize)
                .wrapping_sub(_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR) as usize)
                .wrapping_sub(1 as usize),
        );

        if *rest != 0 {
            _tcsncat(
                core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
                core::ptr::addr_of_mut!(space_s) as *mut TCHAR,
                (CMDLINE_LENGTH as usize)
                    .wrapping_sub(_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR) as usize)
                    .wrapping_sub(1 as usize),
            );
            _tcsncat(
                core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
                rest,
                (CMDLINE_LENGTH as usize)
                    .wrapping_sub(_tcslen(core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR) as usize)
                    .wrapping_sub(1 as usize),
            );
        }

        /* fill startup info */
        memset(
            core::ptr::addr_of_mut!(stui) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<STARTUPINFO>(),
        );
        stui.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
        stui.lpTitle = Full;
        stui.dwFlags = 0;
        stui.wShowWindow = (SW_SHOWNORMAL) as u16;

        /* Set the console to standard mode */
        SetConsoleMode(
            ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdIn)),
            ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT,
        );

        if CreateProcess(
            core::ptr::addr_of_mut!(szFullName) as *mut TCHAR,
            core::ptr::addr_of_mut!(szFullCmdLine) as *mut TCHAR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (TRUE) as BOOL,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(stui),
            core::ptr::addr_of_mut!(prci),
        ) != 0
        {
            CloseHandle(prci.hThread);
        } else if GetLastError() == ERROR_BAD_EXE_FORMAT {
            // See if we can run this with ShellExecute() ie myfile.xls
            let mut hProcess: HANDLE = RunFile(
                SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NO_CONSOLE,
                core::ptr::addr_of_mut!(szFullName) as *mut TCHAR,
                rest,
                core::ptr::null_mut(),
                (SW_SHOWNORMAL) as INT,
            );
            execerror = (if !hProcess.is_null() {
                ERROR_SUCCESS
            } else {
                GetLastError()
            }) as UINT;
        }

        *FirstEnd = 0 as TCHAR;

        if !prci.hProcess.is_null() {
            if !bc.is_null()
                || fSingleCommand != 0
                || IsConsoleProcess(prci.hProcess) != 0
            {
                /* when processing a batch file or starting console processes: execute synchronously */
                EnterCriticalSection(core::ptr::addr_of_mut!(ChildProcessRunningLock));
                dwChildProcessId = prci.dwProcessId;

                WaitForSingleObject(prci.hProcess, INFINITE);

                LeaveCriticalSection(core::ptr::addr_of_mut!(ChildProcessRunningLock));

                GetExitCodeProcess(prci.hProcess, core::ptr::addr_of_mut!(dwExitCode));
                nErrorLevel = dwExitCode as INT;
            }
            CloseHandle(prci.hProcess);
        } else if execerror != 0 {
            /* ShellExecute failed: report the error */
            if execerror == ERROR_NO_ASSOCIATION {
                error_cant_exec_program();
            } else {
                error_bad_command(first);
            }
            dwExitCode = 1;
        }

        /* Restore the default console mode */
        SetConsoleMode(
            ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdIn)),
            ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT,
        );
        SetConsoleMode(
            ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdOut)),
            ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT,
        );
    }

    /* Update the local code page cache */
    {
        let mut uNewInputCodePage: UINT = GetConsoleCP();
        let mut uNewOutputCodePage: UINT = GetConsoleOutputCP();

        if (InputCodePage != uNewInputCodePage) || (OutputCodePage != uNewOutputCodePage) {
            InputCodePage = uNewInputCodePage;
            OutputCodePage = uNewOutputCodePage;

            /* Reset the current thread UI language */
            if IsConsoleHandle(ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdOut))) != 0
                || IsConsoleHandle(ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdErr))) != 0
            {
                ConSetThreadUILanguage(0);
            }
            /* Update the streams cached code page */
            ConStdStreamsSetCacheCodePage(InputCodePage, OutputCodePage);

            /* Update the locale as well */
            InitLocale();
        }
    }

    /* Restore the original console title */
    ResetConTitle();

    return dwExitCode as INT;
}
#[no_mangle]
pub unsafe extern "C" fn DoCommand(first: LPTSTR, rest: LPTSTR, Cmd: *mut PARSED_COMMAND) -> INT {
    let mut com: *mut TCHAR = core::ptr::null_mut();
    let mut cp: *mut TCHAR = core::ptr::null_mut();
    let mut param: LPTSTR = core::ptr::null_mut();
    let mut cl: INT = 0;
    let mut cmdptr: LPCOMMAND = core::ptr::null_mut();
    let mut nointernal: BOOL = (FALSE) as i32;
    let mut ret: INT = 0;

    /* Full command line */
    com = cmd_alloc!(
        ((_tcslen(first) + _tcslen(rest) + 2)
            * core::mem::size_of::<TCHAR>()) as usize,
    ) as *mut TCHAR;
    if com.is_null() {
        error_out_of_memory();
        return 1;
    }

    /* If present in the first word, these characters end the name of an
     * internal command and become the beginning of its parameters. */
    let delims = [
        '\t' as u16,
        ' ' as u16,
        '+' as u16,
        ',' as u16,
        '/' as u16,
        ';' as u16,
        '=' as u16,
        '[' as u16,
        ']' as u16,
        0,
    ];
    cp = first.offset(_tcscspn(first, delims.as_ptr()) as isize) as *mut TCHAR;

    cl = 0;
    while (cl as isize) < cp.offset_from(first) {
        /* These characters do it too, but if one of them is present,
         * then we check to see if the word is a file name and skip
         * checking for internal commands if so.
         * This allows running programs with names like "echo.exe" */
        let dots = ['.' as u16, ':' as u16, '\\' as u16, 0];
        if !_tcschr(dots.as_ptr(), *first.offset(cl as isize)).is_null() {
            let mut tmp: TCHAR = *cp;
            *cp = 0 as TCHAR;
            nointernal = (IsExistingFile(first) != 0) as i32;
            *cp = tmp;
            break;
        }
        cl += 1;
    }

    /* Scan internal command table */
    cmdptr = core::ptr::addr_of_mut!(cmds) as *mut COMMAND as LPCOMMAND;
    while nointernal == 0 && !(*cmdptr).name.is_null() {
        if _tcsnicmp(first, (*cmdptr).name, cl as usize) == 0
            && *(*cmdptr).name.offset(cl as isize) == 0 as TCHAR
        {
            _tcscpy(com, first);
            _tcscat(com, rest);
            param = com.offset(cl as isize);

            /* Skip over whitespace to rest of line, exclude 'echo' command */
            let echo = ['e' as u16, 'c' as u16, 'h' as u16, 'o' as u16, 0];
            if _tcsicmp((*cmdptr).name, echo.as_ptr()) != 0 {
                while _istspace!(*param) != 0 {
                    param = param.offset(1);
                }
            }

            /* Set the new console title */
            SetConTitle(com);

            ret = (((*cmdptr).func).unwrap_unchecked())(param);

            /* Restore the original console title */
            ResetConTitle();

            cmd_free!(com);
            return ret;
        }
        cmdptr = cmdptr.offset(1);
    }

    ret = Execute(com, first, rest, Cmd);
    cmd_free!(com);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ParseCommandLine(cmd: LPTSTR) -> INT {
    let mut Ret: INT = 0;
    let mut Cmd: *mut PARSED_COMMAND;

    Cmd = ParseCommand(cmd as PCTSTR);
    if Cmd.is_null() {
        /* Return an adequate error code */
        return if bParseError == 0 { 0 } else { 1 };
    }

    Ret = ExecuteCommand(Cmd);
    FreeCommand(Cmd);
    return Ret;
}

unsafe extern "C" fn ExecuteAsync(mut Cmd: *mut PARSED_COMMAND) -> HANDLE {
    let mut CmdPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut CmdParams: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut ParamsEnd: *mut TCHAR = core::ptr::null_mut();
    let mut stui: STARTUPINFO = core::mem::zeroed();
    let mut prci: PROCESS_INFORMATION = core::mem::zeroed();

    /* Get the path to cmd.exe */
    GetModuleFileName(
        core::ptr::null_mut(),
        &raw mut CmdPath as *mut TCHAR,
        ARRAYSIZE!(CmdPath),
    );

    /* Build the parameter string to pass to cmd.exe */
    let prefix: [u16; 8] = [
        '/' as u16, 'S' as u16, '/' as u16, 'D' as u16,
        '/' as u16, 'C' as u16, '"' as u16, 0,
    ];
    ParamsEnd = _stpcpy(
        &raw mut CmdParams as *mut TCHAR,
        &raw const prefix as *const TCHAR,
    );
    ParamsEnd = UnparseCommand(
        Cmd,
        ParamsEnd,
        (&raw mut CmdParams as *mut TCHAR)
            .offset((CMDLINE_LENGTH - 2) as isize),
    );
    if ParamsEnd.is_null() {
        error_out_of_memory();
        return core::ptr::null_mut();
    }
    let quote: [u16; 2] = ['"' as u16, 0];
    _tcscpy(ParamsEnd, &raw const quote as *const TCHAR);

    memset(
        &raw mut stui as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<STARTUPINFO>(),
    );
    stui.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
    if CreateProcess(
        &raw mut CmdPath as *mut TCHAR as LPCWSTR,
        &raw mut CmdParams as *mut TCHAR as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TRUE as BOOL,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut stui,
        &raw mut prci,
    ) == 0
    {
        ErrorMessage(GetLastError(), core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    CloseHandle(prci.hThread);
    return prci.hProcess;
}

unsafe extern "C" fn ExecutePipeline(mut Cmd: *mut PARSED_COMMAND) -> INT {
    let mut hInput: HANDLE = core::ptr::null_mut();
    let hOldConIn: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
    let hOldConOut: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
    let mut hProcess: [HANDLE; (MAXIMUM_WAIT_OBJECTS) as _] =
        [core::ptr::null_mut(); (MAXIMUM_WAIT_OBJECTS) as _];
    let mut nProcesses: INT = 0;
    let mut dwExitCode: DWORD = 0;
    let pipe_str: [u16; 2] = ['|' as u16, 0];

    'failed: {
        /* Do all but the last pipe command */
        loop {
            let mut hPipeRead: HANDLE = core::ptr::null_mut();
            let mut hPipeWrite: HANDLE = core::ptr::null_mut();
            if (nProcesses) as u32 > (MAXIMUM_WAIT_OBJECTS - 2) {
                error_too_many_parameters(pipe_str.as_ptr() as PCTSTR);
                break 'failed;
            }

            /* Create the pipe that this process will write into.
             * Make the handles non-inheritable initially, because this
             * process shouldn't inherit the reading handle. */
            if CreatePipe(
                &raw mut hPipeRead,
                &raw mut hPipeWrite,
                core::ptr::null_mut(),
                0,
            ) == 0
            {
                error_no_pipe();
                break 'failed;
            }

            /* The writing side of the pipe is STDOUT for this process */
            SetHandleInformation(hPipeWrite, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
            SetStdHandle(STD_OUTPUT_HANDLE, hPipeWrite);

            /* Execute it (error check is done later for easier cleanup) */
            hProcess[nProcesses as usize] =
                ExecuteAsync((*Cmd).Subcommands as *mut PARSED_COMMAND);
            CloseHandle(hPipeWrite);
            if !hInput.is_null() {
                CloseHandle(hInput);
            }

            /* The reading side of the pipe will be STDIN for the next process */
            SetHandleInformation(hPipeRead, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
            SetStdHandle(STD_INPUT_HANDLE, hPipeRead);
            hInput = hPipeRead;

            if hProcess[nProcesses as usize].is_null() {
                break 'failed;
            }
            nProcesses += 1;

            Cmd = (*(*Cmd).Subcommands).Next as *mut PARSED_COMMAND;

            if !((*Cmd).Type == C_PIPE) {
                break;
            }
        }

        /* The last process uses the original STDOUT */
        SetStdHandle(STD_OUTPUT_HANDLE, hOldConOut);
        hProcess[nProcesses as usize] = ExecuteAsync(Cmd);
        if hProcess[nProcesses as usize].is_null() {
            break 'failed;
        }
        nProcesses += 1;
        CloseHandle(hInput);
        SetStdHandle(STD_INPUT_HANDLE, hOldConIn);

        /* Wait for all processes to complete */
        EnterCriticalSection(&raw mut ChildProcessRunningLock);
        WaitForMultipleObjects(
            nProcesses as DWORD,
            &raw mut hProcess as *mut HANDLE,
            (TRUE) as BOOL,
            INFINITE,
        );
        LeaveCriticalSection(&raw mut ChildProcessRunningLock);

        /* Use the exit code of the last process in the pipeline */
        GetExitCodeProcess(
            hProcess[(nProcesses - 1) as usize],
            &raw mut dwExitCode,
        );
        nErrorLevel = dwExitCode as INT;

        loop {
            nProcesses -= 1;
            if !(nProcesses >= 0) {
                break;
            }
            CloseHandle(hProcess[nProcesses as usize]);
        }
        return nErrorLevel;
    }

    /* failed: */
    if !hInput.is_null() {
        CloseHandle(hInput);
    }
    loop {
        nProcesses -= 1;
        if !(nProcesses >= 0) {
            break;
        }
        TerminateProcess(hProcess[nProcesses as usize], 0);
        CloseHandle(hProcess[nProcesses as usize]);
    }
    SetStdHandle(STD_INPUT_HANDLE, hOldConIn);
    SetStdHandle(STD_OUTPUT_HANDLE, hOldConOut);

    return nErrorLevel;
}
#[no_mangle]
pub unsafe extern "C" fn ExecuteCommand(Cmd: *mut _PARSED_COMMAND) -> INT {
    let mut Sub: *mut PARSED_COMMAND = core::ptr::null_mut::<PARSED_COMMAND>();
    let mut First: LPTSTR = core::ptr::null_mut::<WCHAR>();
    let mut Rest: LPTSTR = core::ptr::null_mut::<WCHAR>();
    let mut Ret: INT = 0;

    /* If we don't have any command, or if this is REM, ignore it */
    if Cmd.is_null() || (*Cmd).Type as ::core::ffi::c_uint == C_REM {
        return 0;
    }
    /*
     * Do not execute any command if we are about to exit CMD, or about to
     * change batch execution context, e.g. in case of a CALL / GOTO / EXIT.
     */
    if bExit != 0 || (!bc.is_null() && (*bc).current.is_null()) {
        return 0;
    }

    if PerformRedirection((*Cmd).Redirections as *mut REDIRECTION) == 0 {
        return 1;
    }

    match (*Cmd).Type as ::core::ffi::c_uint {
        C_COMMAND => {
            Ret = 1;
            First = DoDelayedExpansion(
                core::ptr::addr_of_mut!((*Cmd).__bindgen_anon_1.Command.First) as *mut TCHAR as PCTSTR,
            ) as LPTSTR;
            if !First.is_null() {
                Rest = DoDelayedExpansion((*Cmd).__bindgen_anon_1.Command.Rest as PCTSTR) as LPTSTR;
                if !Rest.is_null() {
                    Ret = DoCommand(First, Rest, Cmd);
                    cmd_free!(Rest as *mut ::core::ffi::c_void);
                }
                cmd_free!(First as *mut ::core::ffi::c_void);
            }
            /* Fall through */
            // falls through to the break case below
        }
        C_REM => {
            // break
        }

        C_QUIET | C_BLOCK | C_MULTI => {
            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
            while !Sub.is_null() && !(!bc.is_null() && (*bc).current.is_null()) {
                Ret = ExecuteCommand(Sub);
                Sub = (*Sub).Next as *mut PARSED_COMMAND;
            }
        }

        C_OR => {
            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
            Ret = ExecuteCommand(Sub);
            if (Ret != 0) && !(!bc.is_null() && (*bc).current.is_null()) {
                nErrorLevel = Ret;
                Ret = ExecuteCommand((*Sub).Next as *mut PARSED_COMMAND);
            }
        }

        C_AND => {
            Sub = (*Cmd).Subcommands as *mut PARSED_COMMAND;
            Ret = ExecuteCommand(Sub);
            if (Ret == 0) && !(!bc.is_null() && (*bc).current.is_null()) {
                Ret = ExecuteCommand((*Sub).Next as *mut PARSED_COMMAND);
            }
        }

        C_PIPE => {
            Ret = ExecutePipeline(Cmd);
        }

        C_FOR => {
            Ret = ExecuteFor(Cmd);
        }

        C_IF => {
            Ret = ExecuteIf(Cmd);
        }

        _ => {}
    }

    UndoRedirection(
        (*Cmd).Redirections as *mut REDIRECTION,
        core::ptr::null_mut::<REDIRECTION>(),
    );
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn ExecuteCommandWithEcho(Cmd: *mut PARSED_COMMAND) -> INT {
    /* Echo the reconstructed command line */
    if bEcho != 0 && bDisableBatchEcho == 0 && !Cmd.is_null() && ((*Cmd).Type != C_QUIET) {
        if bIgnoreEcho == 0 {
            ConOutChar(('\n' as TCHAR));
        }
        PrintPrompt();
        EchoCommand(Cmd);
        ConOutChar(('\n' as TCHAR));
    }

    /* Run the command */
    return ExecuteCommand(Cmd);
}
#[no_mangle]
pub unsafe extern "C" fn GetEnvVar(varName: LPCTSTR) -> LPTSTR {
    static mut ret: LPTSTR = core::ptr::null_mut();
    let mut size: UINT;

    cmd_free!(ret as *mut core::ffi::c_void);
    ret = core::ptr::null_mut();
    size = GetEnvironmentVariable(varName as LPCWSTR, core::ptr::null_mut(), 0) as UINT;
    if size > 0 {
        ret = cmd_alloc!((size as usize) * core::mem::size_of::<TCHAR>()) as LPTSTR;
        if !ret.is_null() {
            GetEnvironmentVariable(varName as LPCWSTR, ret as LPWSTR, (size + 1) as DWORD);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn GetEnvVarOrSpecial(varName: LPCTSTR) -> LPCTSTR {
    static mut ret: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    let var: LPTSTR = GetEnvVar(varName);
    if !var.is_null() {
        return var as LPCTSTR;
    }

    /* The environment variable doesn't exist, look for
     * a "special" one only if extensions are enabled. */
    if bEnableExtensions == 0 {
        return core::ptr::null();
    }

    /* %CD% */
    if _tcsicmp(varName, (&[0x0043, 0x0044, 0x0000u16]).as_ptr() /* _T "CD" */) == 0 {
        GetCurrentDirectory(ARRAYSIZE!(ret), addr_of_mut!(ret) as *mut TCHAR);
        return addr_of_mut!(ret) as *mut TCHAR as LPCTSTR;
    }
    /* %DATE% */
    else if _tcsicmp(varName, (&[0x0044, 0x0041, 0x0054, 0x0045, 0x0000u16]).as_ptr() /* _T "DATE" */) == 0 {
        return GetDateString() as LPCTSTR;
    }
    /* %TIME% */
    else if _tcsicmp(varName, (&[0x0054, 0x0049, 0x004d, 0x0045, 0x0000u16]).as_ptr() /* _T "TIME" */) == 0 {
        return GetTimeString() as LPCTSTR;
    }
    /* %RANDOM% */
    else if _tcsicmp(varName, (&[0x0052, 0x0041, 0x004e, 0x0044, 0x004f, 0x004d, 0x0000u16]).as_ptr() /* _T "RANDOM" */) == 0 {
        /* Get random number */
        _itot(
            rand(),
            addr_of_mut!(ret) as *mut TCHAR,
            10,
        );
        return addr_of_mut!(ret) as *mut TCHAR as LPCTSTR;
    }
    /* %CMDCMDLINE% */
    else if _tcsicmp(varName, (&[0x0043, 0x004d, 0x0044, 0x0043, 0x004d, 0x0044, 0x004c, 0x0049, 0x004e, 0x0045, 0x0000u16]).as_ptr() /* _T "CMDCMDLINE" */) == 0 {
        return GetCommandLine() as LPCTSTR;
    }
    /* %CMDEXTVERSION% */
    else if _tcsicmp(varName, (&[0x0043, 0x004d, 0x0044, 0x0045, 0x0058, 0x0054, 0x0056, 0x0045, 0x0052, 0x0053, 0x0049, 0x004f, 0x004e, 0x0000u16]).as_ptr() /* _T "CMDEXTVERSION" */) == 0 {
        /* Set Command Extensions version number to CMDEXTVERSION */
        _itot(
            (CMDEXTVERSION as i32),
            addr_of_mut!(ret) as *mut TCHAR,
            10,
        );
        return addr_of_mut!(ret) as *mut TCHAR as LPCTSTR;
    }
    /* %ERRORLEVEL% */
    else if _tcsicmp(varName, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x004c, 0x0045, 0x0056, 0x0045, 0x004c, 0x0000u16]).as_ptr() /* _T "ERRORLEVEL" */) == 0 {
        _itot(
            nErrorLevel,
            addr_of_mut!(ret) as *mut TCHAR,
            10,
        );
        return addr_of_mut!(ret) as *mut TCHAR as LPCTSTR;
    }
    /* Available in Win7+, even if the underlying API is available in Win2003+ */
    /* %HIGHESTNUMANODENUMBER% */
    else if _tcsicmp(varName, (&[0x0048, 0x0049, 0x0047, 0x0048, 0x0045, 0x0053, 0x0054, 0x004e, 0x0055, 0x004d, 0x0041, 0x004e, 0x004f, 0x0044, 0x0045, 0x004e, 0x0055, 0x004d, 0x0042, 0x0045, 0x0052, 0x0000u16]).as_ptr() /* _T "HIGHESTNUMANODENUMBER" */) == 0 {
        let mut NumaNodeNumber: ULONG = 0;
        GetNumaHighestNodeNumber(addr_of_mut!(NumaNodeNumber));
        _itot(
            (NumaNodeNumber) as i32,
            addr_of_mut!(ret) as *mut TCHAR,
            10,
        );
        return addr_of_mut!(ret) as *mut TCHAR as LPCTSTR;
    }

    return core::ptr::null();
}

unsafe extern "C" fn GetEnhancedVar(
    pFormat: *mut PCTSTR,
    GetVar: unsafe extern "C" fn(TCHAR, *mut PCTSTR, *mut BOOL) -> BOOL,
) -> PCTSTR {
    const M_DRIVE: u32 = 1; // D: drive letter
    const M_PATH: u32 = 2; // P: path
    const M_NAME: u32 = 4; // N: filename
    const M_EXT: u32 = 8; // X: extension
    const M_FULL: u32 = 16; // F: full path
    const M_SHORT: u32 = 32; // S: short names
    const M_ATTR: u32 = 64; // A: attributes
    const M_TIME: u32 = 128; // T: modification time
    const M_SIZE: u32 = 256; // Z: file size

    static ModifierTable: [TCHAR; 10] = [
        'd' as TCHAR, 'p' as TCHAR, 'n' as TCHAR, 'x' as TCHAR, 'f' as TCHAR,
        's' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 'z' as TCHAR, 0,
    ];
    static EMPTY_STR: [TCHAR; 1] = [0];
    static FMT_SIZE_STR: [TCHAR; 7] = [
        '%' as TCHAR, 'I' as TCHAR, '6' as TCHAR, '4' as TCHAR, 'u' as TCHAR, ' ' as TCHAR, 0,
    ];
    static mut Result: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];

    let mut Modifiers: u32 = 0;

    let mut Format: PCTSTR = core::ptr::null();
    let mut FormatEnd: PCTSTR = core::ptr::null();
    let mut PathVarName: PCTSTR = core::ptr::null();
    let mut Variable: PCTSTR = core::ptr::null();
    let mut VarEnd: PCTSTR = core::ptr::null();
    let mut VariableIsParam0: BOOL = 0;
    let mut FullPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FixedPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Filename: PTCHAR = core::ptr::null_mut();
    let mut Extension: PTCHAR = core::ptr::null_mut();
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut w32fd: WIN32_FIND_DATA = core::mem::zeroed();
    let mut In: PTCHAR = core::ptr::null_mut();
    let mut Out: PTCHAR = core::ptr::null_mut();

    /* First, go through as many modifier characters as possible */
    FormatEnd = *pFormat;
    Format = FormatEnd;
    while *FormatEnd != 0
        && !(_tcschr(
            &raw const ModifierTable as PCTSTR,
            _totlower(*FormatEnd),
        ) as PCTSTR)
            .is_null()
    {
        FormatEnd = FormatEnd.offset(1);
    }

    if *FormatEnd == ('$' as u16) {
        /* $PATH: syntax */
        PathVarName = FormatEnd.offset(1 as isize);
        FormatEnd = _tcschr(PathVarName, (':' as u16)) as PCTSTR;
        if FormatEnd.is_null() {
            return core::ptr::null();
        }

        /* Must be immediately followed by the variable */
        FormatEnd = FormatEnd.offset(1);
        if GetVar(*FormatEnd, &raw mut Variable, &raw mut VariableIsParam0) == 0 {
            return core::ptr::null();
        }
    } else {
        /* Backtrack if necessary to get a variable name match */
        while GetVar(*FormatEnd, &raw mut Variable, &raw mut VariableIsParam0) == 0 {
            if FormatEnd == Format {
                return core::ptr::null();
            }
            FormatEnd = FormatEnd.offset(-1);
        }
    }

    *pFormat = FormatEnd.offset(1 as isize);

    /* If the variable is empty, return an empty string */
    if Variable.is_null() || *Variable == 0 {
        return &raw const EMPTY_STR as PCTSTR;
    }

    /* Exclude the leading and trailing quotes */
    VarEnd = Variable.offset(_tcslen(Variable) as isize);
    if *Variable == ('"' as u16) {
        Variable = Variable.offset(1);
        if VarEnd > Variable && *VarEnd.offset(-(1 as isize)) == ('"' as u16) {
            VarEnd = VarEnd.offset(-1);
        }
    }

    if (VarEnd as usize).wrapping_sub(Variable as usize)
        >= core::mem::size_of::<[TCHAR; (CMDLINE_LENGTH) as _]>()
    {
        return &raw const EMPTY_STR as PCTSTR;
    }
    memcpy(
        &raw mut Result as *mut TCHAR as *mut core::ffi::c_void,
        Variable as *const core::ffi::c_void,
        (VarEnd as usize).wrapping_sub(Variable as usize),
    );
    *(&raw mut Result as *mut TCHAR).offset(VarEnd.offset_from(Variable) as isize) = ('\0' as u16);

    /* Now determine the actual modifiers */
    while Format < FormatEnd && *Format != ('$' as u16) {
        Modifiers |= 1
            << (_tcschr(
                &raw const ModifierTable as PCTSTR,
                _totlower(*Format),
            ) as PCTSTR)
                .offset_from(&raw const ModifierTable as PCTSTR) as u32;
        Format = Format.offset(1);
    }

    if !PathVarName.is_null() {
        /* $PATH: syntax - search the directories listed in the
         * specified environment variable for the file */
        let mut PathVar: PTSTR = core::ptr::null_mut();
        *(FormatEnd as PTSTR).offset(-(1 as isize)) = ('\0' as u16); // FIXME: HACK!
        PathVar = GetEnvVar(PathVarName) as PTSTR;
        *(FormatEnd as PTSTR).offset(-(1 as isize)) = (':' as u16);
        if PathVar.is_null()
            || SearchPath(
                PathVar as LPCTSTR,
                &raw mut Result as *mut TCHAR as LPCTSTR,
                core::ptr::null(),
                ARRAYSIZE!(FullPath) as DWORD,
                &raw mut FullPath as *mut TCHAR as LPTSTR,
                core::ptr::null_mut(),
            ) == 0
        {
            return &raw const EMPTY_STR as PCTSTR;
        }
    } else if Modifiers == 0 {
        /* For plain %~var with no modifiers, just return the variable without quotes */
        return &raw mut Result as *mut TCHAR as PCTSTR;
    } else if VariableIsParam0 != 0 {
        /* Special case: %0 with modifiers uses the batch file's path */
        if bc.is_null() {
            RtlAssert(
                b"bc\0" as *const u8 as PVOID,
                b"base/shell/cmd/cmd.c\0"
                    as *const u8 as PVOID,
                1129 as ULONG,
                core::ptr::null_mut(),
            );
        }
        _tcscpy(
            &raw mut FullPath as *mut TCHAR,
            &raw mut (*bc).BatchFilePath as *mut TCHAR,
        );
    } else {
        /* Convert the variable, now without quotes, to a full path */
        if GetFullPathName(
            &raw mut Result as *mut TCHAR as LPCTSTR,
            ARRAYSIZE!(FullPath) as DWORD,
            &raw mut FullPath as *mut TCHAR as LPTSTR,
            core::ptr::null_mut(),
        ) == 0
        {
            return &raw const EMPTY_STR as PCTSTR;
        }
    }

    /* Next step is to change the path to fix letter case and, if requested
     * with the S modifier, replace long filenames with short. */

    In = &raw mut FullPath as *mut TCHAR;
    Out = &raw mut FixedPath as *mut TCHAR;

    /* Copy drive letter */
    *Out = *In;
    Out = Out.offset(1);
    In = In.offset(1);
    *Out = *In;
    Out = Out.offset(1);
    In = In.offset(1);
    *Out = *In;
    Out = Out.offset(1);
    In = In.offset(1);

    /* Loop over each \-separated component in the path */
    loop {
        let mut Next: PTCHAR = _tcschr(In as PCTSTR, ('\\' as u16)) as PTCHAR;
        if !Next.is_null() {
            *Next = ('\0' as u16);
            Next = Next.offset(1);
        }
        /* Use FindFirstFile to get the correct name */
        if Out
            .offset(_tcslen(In as PCTSTR) as isize)
            .offset(1 as isize)
            >= (&raw mut FixedPath as *mut TCHAR).offset(ARRAYSIZE!(FixedPath) as isize)
        {
            return &raw const EMPTY_STR as PCTSTR;
        }
        _tcscpy(Out, In as PCTSTR);
        hFind = FindFirstFile(
            &raw mut FixedPath as *mut TCHAR as LPCTSTR,
            &raw mut w32fd,
        );
        /* If it doesn't exist, just leave the name as it was given */
        if hFind != INVALID_HANDLE_VALUE {
            let mut FixedComponent: PTSTR = &raw mut w32fd.cFileName as *mut TCHAR;
            if (Modifiers & M_SHORT) != 0
                && *(&raw mut w32fd.cAlternateFileName as *mut TCHAR) != 0
            {
                FixedComponent = &raw mut w32fd.cAlternateFileName as *mut TCHAR;
            }

            FindClose(hFind);

            if Out
                .offset(_tcslen(FixedComponent as PCTSTR) as isize)
                .offset(1 as isize)
                >= (&raw mut FixedPath as *mut TCHAR).offset(ARRAYSIZE!(FixedPath) as isize)
            {
                return &raw const EMPTY_STR as PCTSTR;
            }
            _tcscpy(Out, FixedComponent as PCTSTR);
        }
        Filename = Out;
        Out = Out.offset(_tcslen(Out as PCTSTR) as isize);
        *Out = ('\\' as u16);
        Out = Out.offset(1);

        In = Next;
        if In.is_null() {
            break;
        }
    }
    *Out.offset(-(1 as isize)) = ('\0' as u16);

    /* Build the result string. Start with attributes, modification time, and
     * file size. If the file didn't exist, these fields will all be empty. */
    Out = &raw mut Result as *mut TCHAR as PTCHAR;
    if hFind != INVALID_HANDLE_VALUE {
        if (Modifiers & M_ATTR) != 0 {
            // cmd.c's 'v'/'x' rows are `#if (NTDDI_VERSION >= NTDDI_WIN8)`, off in this build.
            let Table: [(TCHAR, WORD); 9] = [
                (('d' as u16), FILE_ATTRIBUTE_DIRECTORY as WORD),
                (('r' as u16), FILE_ATTRIBUTE_READONLY as WORD),
                (('a' as u16), FILE_ATTRIBUTE_ARCHIVE as WORD),
                (('h' as u16), FILE_ATTRIBUTE_HIDDEN as WORD),
                (('s' as u16), FILE_ATTRIBUTE_SYSTEM as WORD),
                (('c' as u16), FILE_ATTRIBUTE_COMPRESSED as WORD),
                (('o' as u16), FILE_ATTRIBUTE_OFFLINE as WORD),
                (('t' as u16), FILE_ATTRIBUTE_TEMPORARY as WORD),
                (('l' as u16), FILE_ATTRIBUTE_REPARSE_POINT as WORD),
            ];
            let TableLen: usize = (ARRAYSIZE!(Table)) as usize;
            let mut Attrib: *const (TCHAR, WORD) = &raw const Table as *const (TCHAR, WORD);
            loop {
                if !(Attrib
                    != (&raw const Table as *const (TCHAR, WORD)).offset(TableLen as isize))
                {
                    break;
                }
                *Out = (if (w32fd.dwFileAttributes & ((*Attrib).1 as DWORD)) != 0 {
                    (*Attrib).0
                } else {
                    ('-' as u16)
                });
                Out = Out.offset(1);
                Attrib = Attrib.offset(1);
            }
            *Out = (' ' as u16);
            Out = Out.offset(1);
        }
        if (Modifiers & M_TIME) != 0 {
            let mut ft: FILETIME = core::mem::zeroed();
            let mut st: SYSTEMTIME = core::mem::zeroed();
            FileTimeToLocalFileTime(&raw mut w32fd.ftLastWriteTime, &raw mut ft);
            FileTimeToSystemTime(&raw mut ft, &raw mut st);

            Out = Out.offset(
                FormatDate(Out as *mut TCHAR, &raw mut st, (TRUE) as BOOL)
                    as isize,
            );
            *Out = (' ' as u16);
            Out = Out.offset(1);
            Out = Out.offset(FormatTime(Out as *mut TCHAR, &raw mut st) as isize);
            *Out = (' ' as u16);
            Out = Out.offset(1);
        }
        if (Modifiers & M_SIZE) != 0 {
            let mut Size: ULARGE_INTEGER = core::mem::zeroed();
            Size.__bindgen_anon_1.LowPart = w32fd.nFileSizeLow;
            Size.__bindgen_anon_1.HighPart = w32fd.nFileSizeHigh;
            Out = Out.offset(_stprintf(
                Out as *mut TCHAR,
                &raw const FMT_SIZE_STR as PCTSTR,
                Size.QuadPart,
            ) as isize);
        }
    }

    /* When using the path-searching syntax or the S modifier,
     * at least part of the file path is always included.
     * If none of the DPNX modifiers are present, include the full path */
    if !PathVarName.is_null() || (Modifiers & M_SHORT) != 0 {
        if (Modifiers
            & (M_DRIVE | M_PATH | M_NAME | M_EXT))
            == 0
        {
            Modifiers |= M_FULL;
        }
    }

    /* Now add the requested parts of the name.
     * With the F modifier, add all parts to form the full path. */
    Extension = _tcsrchr(Filename as PCTSTR, ('.' as u16)) as PTCHAR;
    if (Modifiers & (M_DRIVE | M_FULL)) != 0 {
        *Out = FixedPath[0 as usize];
        Out = Out.offset(1);
        *Out = FixedPath[1 as usize];
        Out = Out.offset(1);
    }
    if (Modifiers & (M_PATH | M_FULL)) != 0 {
        memcpy(
            Out as *mut core::ffi::c_void,
            (&raw mut FixedPath as *mut TCHAR).offset(2 as isize)
                as *const core::ffi::c_void,
            (Filename as usize).wrapping_sub(
                (&raw mut FixedPath as *mut TCHAR).offset(2 as isize) as usize,
            ),
        );
        Out = Out.offset(
            Filename.offset_from((&raw mut FixedPath as *mut TCHAR).offset(2 as isize))
                as isize,
        );
    }
    if (Modifiers & (M_NAME | M_FULL)) != 0 {
        while *Filename != 0 && Filename != Extension {
            *Out = *Filename;
            Out = Out.offset(1);
            Filename = Filename.offset(1);
        }
    }
    if (Modifiers & (M_EXT | M_FULL)) != 0 {
        if !Extension.is_null() {
            Out = _stpcpy(Out as LPTSTR, Extension as LPCTSTR) as PTCHAR;
        }
    }

    /* Trim trailing space which otherwise would appear as a
     * result of using the A/T/Z modifiers but no others. */
    while Out != (&raw mut Result as *mut TCHAR).offset(0 as isize)
        && *Out.offset(-(1 as isize)) == (' ' as u16)
    {
        Out = Out.offset(-1);
    }
    *Out = ('\0' as u16);

    return &raw mut Result as *mut TCHAR as PCTSTR;
}

unsafe extern "C" fn GetBatchVar(mut varName: PCTSTR, mut varNameLen: PUINT) -> PCTSTR {
    let mut ret: PCTSTR = ::core::ptr::null::<WCHAR>();
    let mut varNameEnd: PCTSTR = ::core::ptr::null::<WCHAR>();
    *varNameLen = 1;
    match *varName as ::core::ffi::c_int {
        126 => {
            varNameEnd = varName.offset(1 as isize);
            ret = GetEnhancedVar(
                &raw mut varNameEnd,
                FindArg as unsafe extern "C" fn(TCHAR, *mut PCTSTR, *mut BOOL) -> BOOL,
            );
            if ret.is_null() {
                ParseErrorEx(varName);
                return ::core::ptr::null::<WCHAR>();
            }
            *varNameLen =
                varNameEnd.offset_from(varName) as ::core::ffi::c_longlong as ::core::ffi::c_uint;
            return ret;
        }
        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
            let mut dummy: BOOL = 0;
            if FindArg(*varName, &raw mut ret, &raw mut dummy) == 0 {
                return ::core::ptr::null::<WCHAR>();
            } else {
                return ret;
            }
        }
        42 => {
            /* Copy over the raw params (not including the batch file name) */
            return (*bc).raw_params as PCTSTR;
        }
        37 => {
            static PERCENT: [u16; 2] = ['%' as u16, 0];
            return PERCENT.as_ptr();
        }
        _ => {}
    }
    return ::core::ptr::null::<WCHAR>();
}
#[no_mangle]
pub unsafe extern "C" fn SubstituteVar(
    mut Src: PCTSTR,
    SrcIncLen: *mut size_t,
    mut Dest: PTCHAR,
    DestEnd: PTCHAR,
    DestIncLen: *mut size_t,
    Delim: TCHAR,
) -> BOOL {
    let mut Var: PCTSTR = core::ptr::null();
    let mut Start: PCTSTR = core::ptr::null();
    let mut End: PCTSTR = core::ptr::null();
    let mut SubstStart: PCTSTR = core::ptr::null();
    let mut EndChr: TCHAR = 0;
    let mut VarLength: size_t = 0;

    Start = Src;
    End = Dest as PCTSTR;
    *SrcIncLen = 0 as size_t;
    *DestIncLen = 0 as size_t;

    if Delim == 0 {
        return (FALSE) as BOOL;
    }
    if *Src != Delim {
        return (FALSE) as BOOL;
    }

    Src = Src.add(1);

    /* If we are already at the end of the string, fail the substitution */
    SubstStart = Src;

    let outcome: u8 = 'body: {
        if *Src == 0 || *Src == ('\r' as u16) || *Src == ('\n' as u16) {
            break 'body 1;
        }

        if !bc.is_null() && Delim == ('%' as u16) {
            let mut NameLen: UINT = 0;
            Var = GetBatchVar(Src, core::ptr::addr_of_mut!(NameLen));
            if Var.is_null() && bParseError != 0 {
                /* Return the partially-parsed command to be
                 * echoed for error diagnostics purposes. */
                // APPEND1(Delim)
                if Dest >= DestEnd {
                    break 'body 2;
                }
                *Dest = Delim;
                Dest = Dest.add(1);
                // APPEND(Src, strlen(Src) + FACT_0004)
                let AppendLen: size_t =
                    (_tcslen(Src) + 1) as size_t;
                if Dest.add(AppendLen) > DestEnd {
                    break 'body 2;
                }
                core::ptr::copy_nonoverlapping(Src, Dest, AppendLen);
                Dest = Dest.add(AppendLen);
                return (FALSE) as BOOL;
            }
            if Var != core::ptr::null() {
                VarLength = _tcslen(Var) as size_t;
                // APPEND(Var, VarLength)
                if Dest.add(VarLength) > DestEnd {
                    break 'body 2;
                }
                core::ptr::copy_nonoverlapping(Var, Dest, VarLength);
                Dest = Dest.add(VarLength);
                Src = Src.add(NameLen as usize);
                break 'body 0;
            }
        }

        /* Find the end of the variable name. A colon (:) will usually
         * end the name and begin the optional modifier, but not if it
         * is immediately followed by the delimiter (%VAR:%). */
        SubstStart = Src;
        while *Src != 0
            && *Src != Delim
            && !(*Src == (':' as u16) && *Src.offset(1 as isize) != Delim)
        {
            Src = Src.add(1);
        }
        /* If we are either at the end of the string, or the delimiter
         * has been repeated more than once, fail the substitution. */
        if *Src == 0 || Src == SubstStart {
            break 'body 1;
        }

        EndChr = *Src;
        *(Src as PTSTR) = ('\0' as u16); // FIXME: HACK!
        Var = GetEnvVarOrSpecial(SubstStart);
        *(Src as PTSTR) = EndChr;
        Src = Src.add(1);
        if Var.is_null() {
            /* In a batch context, %NONEXISTENT% "expands" to an
             * empty string, otherwise fail the substitution. */
            if !bc.is_null() {
                break 'body 0;
            }
            break 'body 1;
        }
        VarLength = _tcslen(Var) as size_t;

        if EndChr == Delim {
            /* %VAR% - use as-is */
            // APPEND(Var, VarLength)
            if Dest.add(VarLength) > DestEnd {
                break 'body 2;
            }
            core::ptr::copy_nonoverlapping(Var, Dest, VarLength);
            Dest = Dest.add(VarLength);
        } else if *Src == ('~' as u16) {
            /* %VAR:~[start][,length]% - Substring.
             * Negative values are offsets from the end.
             */
            let mut Start: SSIZE_T = _tcstol(
                Src.offset(1 as isize),
                core::ptr::addr_of_mut!(Src) as *mut PTSTR,
                0,
            ) as SSIZE_T;
            let mut End: SSIZE_T = VarLength as SSIZE_T;
            if Start < 0 as SSIZE_T {
                Start = Start.wrapping_add(VarLength as SSIZE_T);
            }
            Start = min!(
                max!(Start, 0 as SSIZE_T) as size_t,
                VarLength,
            ) as SSIZE_T;
            if *Src == (',' as u16) {
                End = _tcstol(
                    Src.offset(1 as isize),
                    core::ptr::addr_of_mut!(Src) as *mut PTSTR,
                    0,
                ) as SSIZE_T;
                End = (End as size_t).wrapping_add(
                    if End < 0 as SSIZE_T {
                        VarLength
                    } else {
                        Start as size_t
                    },
                ) as SSIZE_T;
                End = min!(max!(End, Start) as size_t, VarLength) as SSIZE_T;
            }
            let c: TCHAR = *Src;
            Src = Src.add(1);
            if c != Delim {
                break 'body 1;
            }
            // APPEND(&Var[Start], End - Start)
            let AppendLen: size_t = (End - Start) as size_t;
            if Dest.add(AppendLen) > DestEnd {
                break 'body 2;
            }
            core::ptr::copy_nonoverlapping(Var.offset(Start as isize), Dest, AppendLen);
            Dest = Dest.add(AppendLen);
        } else {
            /* %VAR:old=new%  - Replace all occurrences of old with new.
             * %VAR:*old=new% - Replace first occurrence only,
             *                  and remove everything before it.
             */
            let mut Old: PCTSTR = core::ptr::null();
            let mut New: PCTSTR = core::ptr::null();
            let mut OldLength: size_t = 0;
            let mut NewLength: size_t = 0;
            let mut Star: BOOL = (FALSE) as i32;
            let mut LastMatch: size_t = 0 as size_t;
            let mut i: size_t = 0 as size_t;

            if *Src == ('*' as u16) {
                Star = (TRUE) as i32;
                Src = Src.add(1);
            }

            /* The string to replace may contain the delimiter */
            Old = Src;
            Src = _tcschr(Old, ('=' as u16)) as PCTSTR;
            if Src.is_null() {
                break 'body 1;
            }
            let fresh41 = Src;
            Src = Src.add(1);
            OldLength = fresh41.offset_from(Old) as size_t;
            if OldLength == 0 as size_t {
                break 'body 1;
            }

            New = Src;
            Src = _tcschr(New, Delim) as PCTSTR;
            if Src.is_null() {
                break 'body 1;
            }
            let fresh42 = Src;
            Src = Src.add(1);
            NewLength = fresh42.offset_from(New) as size_t;

            while i < VarLength {
                if _tcsnicmp(Var.add(i), Old, OldLength)
                    == 0
                {
                    if Star == 0 {
                        // APPEND(&Var[LastMatch], i - LastMatch)
                        let AppendLen: size_t = i - LastMatch;
                        if Dest.add(AppendLen) > DestEnd {
                            break 'body 2;
                        }
                        core::ptr::copy_nonoverlapping(Var.add(LastMatch), Dest, AppendLen);
                        Dest = Dest.add(AppendLen);
                    }
                    // APPEND(New, NewLength)
                    if Dest.add(NewLength) > DestEnd {
                        break 'body 2;
                    }
                    core::ptr::copy_nonoverlapping(New, Dest, NewLength);
                    Dest = Dest.add(NewLength);
                    i += OldLength;
                    LastMatch = i;
                    if Star != 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            // APPEND(&Var[LastMatch], VarLength - LastMatch)
            let AppendLen: size_t = VarLength - LastMatch;
            if Dest.add(AppendLen) > DestEnd {
                break 'body 2;
            }
            core::ptr::copy_nonoverlapping(Var.add(LastMatch), Dest, AppendLen);
            Dest = Dest.add(AppendLen);
        }

        break 'body 0;
    };

    let mut outcome = outcome;

    if outcome == 1 {
        /* bad_subst: */
        Src = SubstStart;
        /* Only if no batch context active do we echo the delimiter */
        if bc.is_null() {
            // APPEND1(Delim)
            if Dest >= DestEnd {
                outcome = 2;
            } else {
                *Dest = Delim;
                Dest = Dest.add(1);
            }
        }
    }

    if outcome == 2 {
        /* too_long: */
        ConOutResPrintf!(STRING_ALIAS_ERROR);
        nErrorLevel = 9023;
        return (FALSE) as BOOL;
    }

    /* success: */
    *SrcIncLen = Src.offset_from(Start) as size_t;
    *DestIncLen = Dest.offset_from(End) as size_t;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SubstituteVars(mut Src: PCTSTR, mut Dest: PTSTR, mut Delim: TCHAR) -> BOOL {
    let mut DestEnd: PTCHAR = Dest.offset((CMDLINE_LENGTH - 1) as isize);
    let mut End: PCTSTR;
    let mut SrcIncLen: size_t = 0;
    let mut DestIncLen: size_t = 0;

    'too_long: {
        while *Src != 0 {
            if *Src != Delim {
                End = _tcschr(Src, Delim) as PCTSTR;
                if End.is_null() {
                    End = Src.offset(_tcslen(Src) as isize);
                }
                // APPEND(Src, End - Src)
                if Dest.offset(End.offset_from(Src) as isize) > DestEnd {
                    break 'too_long;
                }
                core::ptr::copy_nonoverlapping(
                    Src as *const TCHAR,
                    Dest as *mut TCHAR,
                    End.offset_from(Src) as usize,
                );
                Dest = Dest.offset(End.offset_from(Src) as isize);
                Src = End;
                continue;
            }

            if SubstituteVar(
                Src,
                core::ptr::addr_of_mut!(SrcIncLen),
                Dest as PTCHAR,
                DestEnd,
                core::ptr::addr_of_mut!(DestIncLen),
                Delim,
            ) == 0
            {
                return (FALSE) as BOOL;
            } else {
                Src = Src.offset(SrcIncLen as isize);
                Dest = Dest.offset(DestIncLen as isize);
            }
        }
        // APPEND1(('\0' as u16))
        if Dest >= DestEnd {
            break 'too_long;
        }
        *Dest = ('\0' as u16);
        Dest = Dest.offset(1);
        return (TRUE) as BOOL;
    }

    // too_long:
    ConOutResPrintf!(STRING_ALIAS_ERROR);
    nErrorLevel = 9023;
    return (FALSE) as BOOL;
}

unsafe extern "C" fn FindForVar(
    mut Var: TCHAR,
    mut VarPtr: *mut PCTSTR,
    mut IsParam0: *mut BOOL,
) -> BOOL {
    let mut Ctx: PFOR_CONTEXT = core::ptr::null_mut::<_FOR_CONTEXT>();
    *VarPtr = core::ptr::null_mut();
    *IsParam0 = FALSE as BOOL;
    Ctx = fc;
    while !Ctx.is_null() {
        if ((Var as core::ffi::c_int - (*Ctx).firstvar as core::ffi::c_int) as UINT)
            < (*Ctx).varcount
        {
            *VarPtr = *(*Ctx).values.offset(
                (Var as core::ffi::c_int - (*Ctx).firstvar as core::ffi::c_int) as isize,
            );
            return TRUE as BOOL;
        }
        Ctx = (*Ctx).prev;
    }
    return FALSE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SubstituteForVars(mut Src: PCTSTR, mut Dest: PTSTR) -> BOOL {
    let empty_str: [u16; 1] = [0u16];
    let mut DestEnd: PTCHAR = Dest.offset((CMDLINE_LENGTH - 1) as isize) as PTCHAR;
    while *Src != 0 {
        if *Src.offset(0 as isize) == ('%' as u16) {
            let mut Dummy: BOOL = 0;
            let mut End: PCTSTR = Src.offset(2 as isize);
            let mut Value: PCTSTR = core::ptr::null();
            if *Src.offset(1 as isize) == ('~' as u16) {
                Value = GetEnhancedVar(
                    core::ptr::addr_of_mut!(End),
                    FindForVar as unsafe extern "C" fn(TCHAR, *mut PCTSTR, *mut BOOL) -> BOOL,
                );
            }
            if Value.is_null() && *Src.offset(1 as isize) != 0 {
                if FindForVar(
                    *Src.offset(1 as isize),
                    core::ptr::addr_of_mut!(Value),
                    core::ptr::addr_of_mut!(Dummy),
                ) != 0
                    && Value.is_null()
                {
                    /* The variable is empty, return an empty string */
                    Value = empty_str.as_ptr() as PCTSTR;
                }
            }
            if !Value.is_null() {
                if Dest.add(_tcslen(Value) as usize) > DestEnd {
                    return (FALSE) as BOOL;
                }
                Dest = _stpcpy(Dest as LPTSTR, Value as LPCTSTR) as PTSTR;
                Src = End;
                continue;
            }
        }
        /* Not a variable; just copy the character */
        if Dest >= DestEnd {
            return (FALSE) as BOOL;
        }
        *Dest = *Src;
        Dest = Dest.add(1);
        Src = Src.add(1);
    }
    *Dest = ('\0' as u16);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DoDelayedExpansion(Line: PCTSTR) -> PTSTR {
    let mut Buf1: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut Buf2: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut Src: PTCHAR;
    let mut Dst: PTCHAR;
    let DestEnd: PTCHAR = (&raw mut Buf2 as PTCHAR)
        .offset(CMDLINE_LENGTH as isize)
        .offset(-(1 as isize));
    let mut SrcIncLen: size_t = 0;
    let mut DestIncLen: size_t = 0;

    /* First, substitute FOR variables */
    if SubstituteForVars(Line, &raw mut Buf1 as PTSTR) == 0 {
        return core::ptr::null_mut();
    }

    if bDelayedExpansion == 0
        || _tcschr(&raw mut Buf1 as *mut TCHAR, ('!' as u16)).is_null()
    {
        return cmd_dup!(&raw mut Buf1 as *mut TCHAR);
    }

    /*
     * Perform delayed expansion: expand variables around '!',
     * and reparse escape carets.
     */

    Src = &raw mut Buf1 as PTCHAR;
    Dst = &raw mut Buf2 as PTCHAR;
    'body: {
        'scan: while *Src != 0
            && Src < (&raw mut Buf1 as PTCHAR).offset(CMDLINE_LENGTH as isize)
        {
            if *Src == ('^' as u16) {
                Src = Src.offset(1);
                if *Src == 0
                    || !(Src < (&raw mut Buf1 as PTCHAR).offset(CMDLINE_LENGTH as isize))
                {
                    break 'scan;
                }
                // APPEND1(*Src++)
                if Dst >= DestEnd {
                    break 'body;
                }
                let ch = *Src;
                Src = Src.offset(1);
                *Dst = ch;
                Dst = Dst.offset(1);
            } else if *Src == ('!' as u16) {
                if SubstituteVar(
                    Src as PCTSTR,
                    &raw mut SrcIncLen,
                    Dst,
                    DestEnd,
                    &raw mut DestIncLen,
                    ('!' as u16),
                ) == 0
                {
                    return core::ptr::null_mut();
                } else {
                    Src = Src.offset(SrcIncLen as isize);
                    Dst = Dst.offset(DestIncLen as isize);
                }
            } else {
                // APPEND1(*Src++)
                if Dst >= DestEnd {
                    break 'body;
                }
                let ch = *Src;
                Src = Src.offset(1);
                *Dst = ch;
                Dst = Dst.offset(1);
            }
            continue;
        }
        // APPEND1(('\0' as u16))
        if Dst >= DestEnd {
            break 'body;
        }
        *Dst = ('\0' as u16);
        Dst = Dst.offset(1);

        return cmd_dup!(&raw mut Buf2 as *mut TCHAR);
    }

    // too_long:
    ConOutResPrintf!(STRING_ALIAS_ERROR);
    nErrorLevel = 9023;
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn ReadLine(commandline: *mut TCHAR, bMore: BOOL) -> BOOL {
    let mut readline: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];
    let mut ip: LPTSTR = core::ptr::null_mut();

    /* if no batch input then... */
    if bc.is_null() {
        if bMore != 0 {
            ConOutResPrintf!(STRING_MORE);
        } else {
            /* JPP 19980807 - if echo off, don't print prompt */
            if bEcho != 0 {
                if bIgnoreEcho == 0 {
                    ConOutChar(('\n' as u16));
                }
                PrintPrompt();
            }
        }

        if ReadCommand(readline.as_mut_ptr(), (CMDLINE_LENGTH - 1) as INT) == 0 {
            bExit = (TRUE) as i32;
            return (FALSE) as BOOL;
        }

        if readline[0] == ('\0' as u16) {
            ConOutChar(('\n' as u16));
        }

        if CheckCtrlBreak((BREAK_INPUT) as INT) != 0 {
            return (FALSE) as BOOL;
        }

        if readline[0] == ('\0' as u16) {
            return (FALSE) as BOOL;
        }

        ip = readline.as_mut_ptr();
    } else {
        ip = ReadBatchLine();
        if ip.is_null() {
            return (FALSE) as BOOL;
        }
    }

    return SubstituteVars(ip as PCTSTR, commandline as PTSTR, ('%' as u16));
}

unsafe extern "C" fn ProcessInput() -> INT {
    let mut Ret: INT = 0;
    let mut Cmd: *mut PARSED_COMMAND = core::ptr::null_mut::<PARSED_COMMAND>();

    while bCanExit == 0 || bExit == 0 {
        /* Reset the Ctrl-Break / Ctrl-C state */
        bCtrlBreak = (FALSE) as i32;

        Cmd = ParseCommand(core::ptr::null_mut());
        if Cmd.is_null() {
            continue;
        }

        Ret = ExecuteCommand(Cmd);
        FreeCommand(Cmd);
    }

    return Ret;
}

unsafe extern "system" fn BreakHandler(dwCtrlType: DWORD) -> BOOL {
    let mut dwWritten: DWORD = 0;
    let mut rec: INPUT_RECORD = core::mem::zeroed();

    if (dwCtrlType != CTRL_C_EVENT) &&
        (dwCtrlType != CTRL_BREAK_EVENT)
    {
        return (FALSE) as BOOL;
    }

    if TryEnterCriticalSection(core::ptr::addr_of_mut!(ChildProcessRunningLock)) == 0
    {
        /* Child process is running and will have received the control event */
        return (TRUE) as BOOL;
    }
    else
    {
        LeaveCriticalSection(core::ptr::addr_of_mut!(ChildProcessRunningLock));
    }

    bCtrlBreak = (TRUE) as i32;

    rec.EventType = (KEY_EVENT as u16);
    rec.Event.KeyEvent.bKeyDown = (TRUE) as i32;
    rec.Event.KeyEvent.wRepeatCount = 1;
    rec.Event.KeyEvent.wVirtualKeyCode = ('C' as u16) as WORD;
    rec.Event.KeyEvent.wVirtualScanCode = (('C' as u16) - 35) as WORD;
    rec.Event.KeyEvent.uChar.AsciiChar = ('C' as u16) as CHAR;
    rec.Event.KeyEvent.uChar.UnicodeChar = ('C' as u16) as WCHAR;
    rec.Event.KeyEvent.dwControlKeyState = RIGHT_CTRL_PRESSED;

    WriteConsoleInput(
        ConStreamGetOSHandle(StdIn),
        core::ptr::addr_of_mut!(rec),
        1,
        core::ptr::addr_of_mut!(dwWritten),
    );

    /* FIXME: Handle batch files */

    // ConOutPrintf((&[0x005e, 0x0043, 0x0000u16]).as_ptr() /* _T "^C" */);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddBreakHandler() {
    SetConsoleCtrlHandler(
        Some(BreakHandler),
        (TRUE) as BOOL,
    );
}
#[no_mangle]
pub unsafe extern "C" fn RemoveBreakHandler() {
    SetConsoleCtrlHandler(Some(BreakHandler), (FALSE) as BOOL);
}

unsafe extern "C" fn LoadRegistrySettings(hKeyRoot: HKEY) {
    let mut lRet: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = core::mem::zeroed();
    let mut len: DWORD;
    /*
     * Buffer big enough to hold the string L"4294967295",
     * corresponding to the literal 0xFFFFFFFF (MAXULONG) in decimal.
     */
    let mut Buffer: [DWORD; 6] = [0; 6];

    let SubKeyPath: [u16; 37] = ['S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'P' as u16, 'r' as u16, 'o' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, 'o' as u16, 'r' as u16, 0];
    lRet = RegOpenKeyEx(hKeyRoot,
                        core::ptr::addr_of!(SubKeyPath) as LPCTSTR,
                        0,
                        KEY_QUERY_VALUE,
                        core::ptr::addr_of_mut!(hKey));
    if lRet != ERROR_SUCCESS as LONG {
        return;
    }

    len = core::mem::size_of::<[DWORD; 6]>() as DWORD;
    let ValDefaultColor: [u16; 13] = ['D' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16, 't' as u16, 'C' as u16, 'o' as u16, 'l' as u16, 'o' as u16, 'r' as u16, 0];
    lRet = RegQueryValueEx(hKey,
                           core::ptr::addr_of!(ValDefaultColor) as LPCTSTR,
                           core::ptr::null_mut(),
                           core::ptr::addr_of_mut!(dwType),
                           core::ptr::addr_of_mut!(Buffer) as LPBYTE,
                           core::ptr::addr_of_mut!(len));
    if lRet == ERROR_SUCCESS as LONG {
        /* Overwrite the default attributes */
        if dwType == REG_DWORD as DWORD {
            wDefColor = *(core::ptr::addr_of_mut!(Buffer) as *mut DWORD) as WORD;
        } else if dwType == REG_SZ as DWORD {
            wDefColor = _tcstol(addr_of_mut!(Buffer) as PTSTR, core::ptr::null_mut(), 0 as c_int) as WORD;
        }
    }
    // else, use the default attributes retrieved before.

    len = core::mem::size_of::<[DWORD; 6]>() as DWORD;
    let ValDelayedExpansion: [u16; 17] = ['D' as u16, 'e' as u16, 'l' as u16, 'a' as u16, 'y' as u16, 'e' as u16, 'd' as u16, 'E' as u16, 'x' as u16, 'p' as u16, 'a' as u16, 'n' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 0];
    lRet = RegQueryValueEx(hKey,
                           core::ptr::addr_of!(ValDelayedExpansion) as LPCTSTR,
                           core::ptr::null_mut(),
                           core::ptr::addr_of_mut!(dwType),
                           core::ptr::addr_of_mut!(Buffer) as LPBYTE,
                           core::ptr::addr_of_mut!(len));
    if lRet == ERROR_SUCCESS as LONG {
        /* Overwrite the default setting */
        if dwType == REG_DWORD as DWORD {
            bDelayedExpansion = (*(core::ptr::addr_of_mut!(Buffer) as *mut DWORD) != 0) as BOOL;
        } else if dwType == REG_SZ as DWORD {
            bDelayedExpansion = (_ttol(addr_of_mut!(Buffer) as PTSTR) == 1 as LONG) as BOOL;
        }
    }
    // else, use the default setting set globally.

    len = core::mem::size_of::<[DWORD; 6]>() as DWORD;
    let ValEnableExtensions: [u16; 17] = ['E' as u16, 'n' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, 'E' as u16, 'x' as u16, 't' as u16, 'e' as u16, 'n' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, 0];
    lRet = RegQueryValueEx(hKey,
                           core::ptr::addr_of!(ValEnableExtensions) as LPCTSTR,
                           core::ptr::null_mut(),
                           core::ptr::addr_of_mut!(dwType),
                           core::ptr::addr_of_mut!(Buffer) as LPBYTE,
                           core::ptr::addr_of_mut!(len));
    if lRet == ERROR_SUCCESS as LONG {
        /* Overwrite the default setting */
        if dwType == REG_DWORD as DWORD {
            bEnableExtensions = (*(core::ptr::addr_of_mut!(Buffer) as *mut DWORD) != 0) as BOOL;
        } else if dwType == REG_SZ as DWORD {
            bEnableExtensions = (_ttol(addr_of_mut!(Buffer) as PTSTR) == 1 as LONG) as BOOL;
        }
    }
    // else, use the default setting set globally.

    len = core::mem::size_of::<[DWORD; 6]>() as DWORD;
    let ValCompletionChar: [u16; 15] = ['C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 0];
    lRet = RegQueryValueEx(hKey,
                           core::ptr::addr_of!(ValCompletionChar) as LPCTSTR,
                           core::ptr::null_mut(),
                           core::ptr::addr_of_mut!(dwType),
                           core::ptr::addr_of_mut!(Buffer) as LPBYTE,
                           core::ptr::addr_of_mut!(len));
    if lRet == ERROR_SUCCESS as LONG {
        /* Overwrite the default setting */
        if dwType == REG_DWORD as DWORD {
            AutoCompletionChar = *(core::ptr::addr_of_mut!(Buffer) as *mut DWORD) as TCHAR;
        } else if dwType == REG_SZ as DWORD {
            AutoCompletionChar = _tcstol(addr_of_mut!(Buffer) as PTSTR, core::ptr::null_mut(), 0 as c_int) as TCHAR;
        }
    }
    // else, use the default setting set globally.

    /* Validity check */
    if (AutoCompletionChar as c_int == 0 || AutoCompletionChar as c_int == 0xd || AutoCompletionChar as c_int >= 0x20) {
        /* Disable autocompletion */
        AutoCompletionChar = 0x20 as TCHAR;
    }

    len = core::mem::size_of::<[DWORD; 6]>() as DWORD;
    let ValPathCompletionChar: [u16; 19] = ['P' as u16, 'a' as u16, 't' as u16, 'h' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 'C' as u16, 'h' as u16, 'a' as u16, 'r' as u16, 0];
    lRet = RegQueryValueEx(hKey,
                           core::ptr::addr_of!(ValPathCompletionChar) as LPCTSTR,
                           core::ptr::null_mut(),
                           core::ptr::addr_of_mut!(dwType),
                           core::ptr::addr_of_mut!(Buffer) as LPBYTE,
                           core::ptr::addr_of_mut!(len));
    if lRet == ERROR_SUCCESS as LONG {
        /* Overwrite the default setting */
        if dwType == REG_DWORD as DWORD {
            PathCompletionChar = *(core::ptr::addr_of_mut!(Buffer) as *mut DWORD) as TCHAR;
        } else if dwType == REG_SZ as DWORD {
            PathCompletionChar = _tcstol(addr_of_mut!(Buffer) as PTSTR, core::ptr::null_mut(), 0 as c_int) as TCHAR;
        }
    }
    // else, use the default setting set globally.

    /* Validity check */
    if (PathCompletionChar as c_int == 0 || PathCompletionChar as c_int == 0xd || PathCompletionChar as c_int >= 0x20) {
        /* Disable autocompletion */
        PathCompletionChar = 0x20 as TCHAR;
    }

    /* Adjust completion chars */
    if PathCompletionChar as c_int >= 0x20 as c_int && (AutoCompletionChar as c_int) < 0x20 as c_int {
        PathCompletionChar = AutoCompletionChar;
    } else if AutoCompletionChar as c_int >= 0x20 as c_int && (PathCompletionChar as c_int) < 0x20 as c_int {
        AutoCompletionChar = PathCompletionChar;
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn ExecuteAutoRunFile(hKeyRoot: HKEY) {
    let mut lRet: LONG = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut len: DWORD = 0;
    let mut AutoRun: [TCHAR; 2048] = [0; 2048];
    let SubKeyName: [u16; 37] = ['S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'a' as u16, 'n' as u16, 'd' as u16, ' ' as u16, 'P' as u16, 'r' as u16, 'o' as u16, 'c' as u16, 'e' as u16, 's' as u16, 's' as u16, 'o' as u16, 'r' as u16, 0];
    let ValueName: [u16; 8] = ['A' as u16, 'u' as u16, 't' as u16, 'o' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 0];

    lRet = RegOpenKeyEx(
        hKeyRoot,
        SubKeyName.as_ptr() as LPCTSTR,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hKey),
    );
    if lRet != ERROR_SUCCESS as LONG {
        return;
    }

    len = core::mem::size_of::<[TCHAR; 2048]>() as DWORD;
    lRet = RegQueryValueEx(
        hKey,
        ValueName.as_ptr() as LPCTSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(AutoRun) as LPBYTE,
        core::ptr::addr_of_mut!(len),
    ) as LONG;
    if (lRet == ERROR_SUCCESS as LONG)
        && (dwType == REG_EXPAND_SZ as DWORD || dwType == REG_SZ as DWORD)
    {
        if *(core::ptr::addr_of_mut!(AutoRun) as *mut TCHAR) != 0 {
            ParseCommandLine(core::ptr::addr_of_mut!(AutoRun) as LPTSTR);
        }
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn GetCmdLineCommand(
    mut commandline: LPTSTR,
    mut ptr: LPCTSTR,
    mut AlwaysStrip: BOOL,
) {
    let mut LastQuote: *mut TCHAR = core::ptr::null_mut();
    while ((_istspace!(*ptr)) != 0) {
        ptr = ptr.add(1);
    }

    /* Remove leading quote, find final quote */
    if *ptr == ('"' as u16) && {
        ptr = ptr.add(1);
        LastQuote = _tcsrchr(ptr, ('"' as u16)) as *mut TCHAR;
        !LastQuote.is_null()
    } {
        let mut Space: *const TCHAR;
        /* Under certain circumstances, all quotes are preserved.
         * CMD /? documents these conditions as follows:
         *  1. No /S switch
         *  2. Exactly two quotes
         *  3. No "special characters" between the quotes
         *     (CMD /? says &<>()@^| but parentheses did not
         *     trigger this rule when I tested them.)
         *  4. Whitespace exists between the quotes
         *  5. Enclosed string is an executable filename
         */
        *LastQuote = ('\0' as u16);
        Space = ptr.add(1 as usize);
        while Space < LastQuote as *const TCHAR {
            if ((_istspace!(*Space)) != 0) {
                /* Rule 4 */
                let ExcludeChars: [u16; 8] = [
                    '"' as u16,
                    '&' as u16,
                    '<' as u16,
                    '>' as u16,
                    '@' as u16,
                    '^' as u16,
                    '|' as u16,
                    0,
                ];
                if AlwaysStrip == 0 /* Rule 1 */
                    && _tcspbrk(ptr, ExcludeChars.as_ptr()).is_null() /* Rules 2, 3 */
                    && SearchForExecutable(ptr, commandline) != 0
                /* Rule 5 */
                {
                    /* All conditions met: preserve both the quotes */
                    *LastQuote = ('"' as u16);
                    _tcscpy(commandline, ptr.sub(1 as usize));
                    return;
                }
                break;
            }
            Space = Space.add(1);
        }

        /* The conditions were not met: remove both the
         * leading quote and the last quote */
        _tcscpy(commandline, ptr);
        _tcscpy(
            commandline.add(LastQuote.offset_from(ptr) as usize),
            LastQuote.add(1 as usize),
        );
        return;
    }

    /* No quotes; just copy */
    _tcscpy(commandline, ptr);
}

unsafe extern "C" fn Initialize() -> LPCTSTR {
    let mut NtDllModule: HMODULE = core::ptr::null_mut();
    let mut hIn: HANDLE = core::ptr::null_mut();
    let mut hOut: HANDLE = core::ptr::null_mut();
    let mut ptr: LPTSTR = core::ptr::null_mut();
    let mut cmdLine: LPTSTR = core::ptr::null_mut();
    let mut option: TCHAR = 0;
    let mut AutoRun: BOOL = (TRUE) as i32;
    let mut ModuleName: [TCHAR; (MAX_PATH + 1) as _] =
        [0; (MAX_PATH + 1) as _];

    let NTDLL_NAME: [TCHAR; 10] = [
        'n' as TCHAR, 't' as TCHAR, 'd' as TCHAR, 'l' as TCHAR, 'l' as TCHAR,
        '.' as TCHAR, 'd' as TCHAR, 'l' as TCHAR, 'l' as TCHAR, 0,
    ];
    let COMSPEC_NAME: [TCHAR; 8] = [
        'C' as TCHAR, 'O' as TCHAR, 'M' as TCHAR, 'S' as TCHAR, 'P' as TCHAR,
        'E' as TCHAR, 'C' as TCHAR, 0,
    ];
    let AUTOEXEC_NAME: [TCHAR; 14] = [
        '\\' as TCHAR, 'a' as TCHAR, 'u' as TCHAR, 't' as TCHAR, 'o' as TCHAR,
        'e' as TCHAR, 'x' as TCHAR, 'e' as TCHAR, 'c' as TCHAR, '.' as TCHAR,
        'b' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 0,
    ];
    let OFF_NAME: [TCHAR; 5] = [
        ':' as TCHAR, 'O' as TCHAR, 'F' as TCHAR, 'F' as TCHAR, 0,
    ];
    let SLASH_T_NAME: [TCHAR; 4] = [
        '/' as TCHAR, 'T' as TCHAR, ':' as TCHAR, 0,
    ];
    let EMPTY_NAME: [TCHAR; 1] = [0];

    /* Get version information */
    InitOSVersion();

    /* Some people like to run ReactOS cmd.exe on Win98, it helps in the
     * build process. So don't link implicitly against ntdll.dll, load it
     * dynamically instead */
    NtDllModule = GetModuleHandle(NTDLL_NAME.as_ptr());
    if !NtDllModule.is_null() {
        NtQueryInformationProcessPtr = core::mem::transmute::<FARPROC, NtQueryInformationProcessProc>(
            GetProcAddress(NtDllModule as HINSTANCE, b"NtQueryInformationProcess\0".as_ptr() as LPCSTR),
        );
        NtReadVirtualMemoryPtr = core::mem::transmute::<FARPROC, NtReadVirtualMemoryProc>(
            GetProcAddress(NtDllModule as HINSTANCE, b"NtReadVirtualMemory\0".as_ptr() as LPCSTR),
        );
    }

    /* Load the registry settings */
    LoadRegistrySettings(HKEY_LOCAL_MACHINE);
    LoadRegistrySettings(HKEY_CURRENT_USER);

    /* Initialize our locale */
    InitLocale();

    /* Initialize prompt support */
    InitPrompt();

    /* Initialize directory stack */
    InitDirectoryStack();

    /* Initialize history */
    InitHistory();

    /* Set COMSPEC environment variable */
    if GetModuleFileName(
        core::ptr::null_mut(),
        ModuleName.as_mut_ptr(),
        ARRAYSIZE!(ModuleName) as DWORD,
    ) != 0
    {
        ModuleName[MAX_PATH as usize] = '\0' as TCHAR;
        SetEnvironmentVariable(COMSPEC_NAME.as_ptr(), ModuleName.as_ptr());
    }

    /* Add ctrl break handler */
    AddBreakHandler();

    /* Set the default console mode */
    hOut = ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdOut));
    hIn = ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdIn));
    SetConsoleMode(hOut, 0 as DWORD); // Reinitialize the console output mode
    SetConsoleMode(
        hOut,
        (ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT) as DWORD,
    );
    SetConsoleMode(
        hIn,
        (ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT) as DWORD,
    );

    cmdLine = GetCommandLine() as LPTSTR;

    ptr = cmdLine;
    while *ptr != 0 {
        if *ptr == '/' as TCHAR {
            option = _totupper(*ptr.add(1 as usize));
            if option == '?' as TCHAR {
                ConOutResPaging((TRUE) as BOOL, STRING_CMD_HELP8);
                nErrorLevel = 1;
                bExit = (TRUE) as i32;
                return core::ptr::null();
            } else if option == 'P' as TCHAR {
                if IsExistingFile(AUTOEXEC_NAME.as_ptr()) == 0 {
                    cmd_date(EMPTY_NAME.as_ptr() as LPTSTR);
                    cmd_time(EMPTY_NAME.as_ptr() as LPTSTR);
                } else {
                    ParseCommandLine(AUTOEXEC_NAME.as_ptr() as LPTSTR);
                }
                bCanExit = (FALSE) as i32;
            } else if option == 'A' as TCHAR {
                OutputStreamMode = AnsiText;
            } else if option == 'C' as TCHAR || option == 'K' as TCHAR || option == 'R' as TCHAR {
                /* Remainder of command line is a command to be run */
                fSingleCommand = (((option == 'K' as TCHAR) as BOOL) << 1)
                    | 1;
                break;
            } else if option == 'D' as TCHAR {
                AutoRun = (FALSE) as i32;
            } else if option == 'Q' as TCHAR {
                bDisableBatchEcho = (TRUE) as i32;
            } else if option == 'S' as TCHAR {
                bAlwaysStrip = (TRUE) as i32;
            } else if _tcsnicmp(ptr as LPCTSTR, SLASH_T_NAME.as_ptr(), 3 as usize) == 0 {
                /* Process /T (color) argument; overwrite any previous settings */
                wDefColor = _tcstoul(
                    ptr.add(3 as usize),
                    core::ptr::addr_of_mut!(ptr),
                    16,
                ) as WORD;
            } else if option == 'U' as TCHAR {
                OutputStreamMode = UTF16Text;
            } else if option == 'F' as TCHAR {
                if _tcsnicmp(
                    ptr.add(2 as usize) as LPCTSTR,
                    OFF_NAME.as_ptr(),
                    4 as usize,
                ) == 0
                {
                    /* Disable file and path completion */
                    AutoCompletionChar = 0x20;
                    PathCompletionChar = 0x20;
                } else {
                    /* Enable (and replace) file and path completion
                     * characters with Ctrl-F and Ctrl-D respectively */
                    AutoCompletionChar = 0x06; // Ctrl-F
                    PathCompletionChar = 0x04; // Ctrl-D
                }
            } else if option == 'V' as TCHAR {
                // FIXME: Check validity of the parameter given to V !
                bDelayedExpansion = _tcsnicmp(
                    ptr.add(2 as usize) as LPCTSTR,
                    OFF_NAME.as_ptr(),
                    4 as usize,
                ) as BOOL;
            } else if option == 'E' as TCHAR {
                // FIXME: Check validity of the parameter given to E !
                bEnableExtensions = _tcsnicmp(
                    ptr.add(2 as usize) as LPCTSTR,
                    OFF_NAME.as_ptr(),
                    4 as usize,
                ) as BOOL;
            } else if option == 'X' as TCHAR {
                /* '/X' is identical to '/E:ON' */
                bEnableExtensions = (TRUE) as i32;
            } else if option == 'Y' as TCHAR {
                /* '/Y' is identical to '/E:OFF' */
                bEnableExtensions = (FALSE) as i32;
            }
        }
        ptr = ptr.add(1);
    }

    if wDefColor == 0 {
        /*
         * If we still do not have the console colour attribute set,
         * retrieve the default one.
         */
        ConGetDefaultAttributes(core::ptr::addr_of_mut!(wDefColor));
    }

    if wDefColor != 0 {
        ConSetScreenColor(
            ConStreamGetOSHandle(core::ptr::addr_of_mut!(csStdOut)),
            wDefColor,
            (TRUE) as BOOL,
        );
    }

    /* Reset the output Standard Streams translation modes and code page caches */
    ConStreamSetMode(core::ptr::addr_of_mut!(csStdOut), OutputStreamMode, OutputCodePage);
    ConStreamSetMode(core::ptr::addr_of_mut!(csStdErr), OutputStreamMode, OutputCodePage);

    if *ptr == 0 {
        /* If neither /C or /K was given, display a simple version string */

        /* Insert a new line above the copyright notice if we are drawing the information line. */
        if HasInfoLine() != 0 {
            ConOutChar('\n' as TCHAR);
        }

        ConOutResPrintf!(
            STRING_REACTOS_VERSION,
            KERNEL_VERSION_STR,
            KERNEL_VERSION_BUILD_STR,
        );
        ConOutResPrintf!(STRING_CMD_COPYRIGHT, COPYRIGHT_YEAR);
    }

    if AutoRun != 0 {
        ExecuteAutoRunFile(HKEY_LOCAL_MACHINE);
        ExecuteAutoRunFile(HKEY_CURRENT_USER);
    }

    /* Returns the rest of the command line */
    return ptr as LPCTSTR;
}

unsafe extern "C" fn Cleanup() {
    /* Run cmdexit.bat */
    if IsExistingFile(
        ['c' as u16, 'm' as u16, 'd' as u16, 'e' as u16, 'x' as u16, 'i' as u16, 't' as u16, '.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR,
    ) != 0
    {
        ConErrResPuts!(STRING_CMD_ERROR5);
        ParseCommandLine(
            ['c' as u16, 'm' as u16, 'd' as u16, 'e' as u16, 'x' as u16, 'i' as u16, 't' as u16, '.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0u16].as_ptr() as *const u16 as LPTSTR,
        );
    } else if IsExistingFile(
        ['\\' as u16, 'c' as u16, 'm' as u16, 'd' as u16, 'e' as u16, 'x' as u16, 'i' as u16, 't' as u16, '.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR,
    ) != 0
    {
        ConErrResPuts!(STRING_CMD_ERROR5);
        ParseCommandLine(
            ['\\' as u16, 'c' as u16, 'm' as u16, 'd' as u16, 'e' as u16, 'x' as u16, 'i' as u16, 't' as u16, '.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0u16].as_ptr() as *const u16 as LPTSTR,
        );
    }

    /* Remove ctrl break handler */
    RemoveBreakHandler();

    /* Restore the default console mode */
    SetConsoleMode(
        ConStreamGetOSHandle(&raw mut csStdIn),
        (ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT) as DWORD,
    );
    SetConsoleMode(
        ConStreamGetOSHandle(&raw mut csStdOut),
        (ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT) as DWORD,
    );

    #[cfg(feature = "debug_mem")]
    {
        #[cfg(feature = "directory_stack")]
        {
            /* Destroy directory stack */
            DestroyDirectoryStack();
        }

        #[cfg(feature = "history")]
        {
            CleanHistory();
        }

        /* Free GetEnvVar's buffer */
        GetEnvVar(core::ptr::null_mut());
    }

    DeleteCriticalSection(&raw mut ChildProcessRunningLock);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *const TCHAR) -> INT {
    let mut nExitCode: INT;
    let mut pCmdLine: LPCTSTR;
    let mut startPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    DbgPrint(b"[CMD] _tmain entered, argc=%d\n\0".as_ptr() as PCSTR, argc);

    InitializeCriticalSection(core::ptr::addr_of_mut!(ChildProcessRunningLock));
    lpOriginalEnvironment = DuplicateEnvironment();

    GetCurrentDirectory(ARRAYSIZE!(startPath) as DWORD, startPath.as_mut_ptr());
    _tchdir(startPath.as_ptr());

    SetFileApisToOEM();
    InputCodePage = GetConsoleCP();
    OutputCodePage = GetConsoleOutputCP();

    /* Initialize the Console Standard Streams */
    ConStreamInit(StdIn, GetStdHandle(STD_INPUT_HANDLE), /*OutputStreamMode*/ AnsiText, InputCodePage);
    ConStreamInit(StdOut, GetStdHandle(STD_OUTPUT_HANDLE), OutputStreamMode, OutputCodePage);
    ConStreamInit(StdErr, GetStdHandle(STD_ERROR_HANDLE), OutputStreamMode, OutputCodePage);
    /* Reset the current thread UI language */
    if IsConsoleHandle(ConStreamGetOSHandle(StdOut)) != 0
        || IsConsoleHandle(ConStreamGetOSHandle(StdErr)) != 0
    {
        ConSetThreadUILanguage(0);
    }

    CMD_ModuleHandle = GetModuleHandle(core::ptr::null_mut()) as HANDLE;

    /*
     * Perform general initialization, parse switches on command-line.
     * Initialize the exit code with the errorlevel as Initialize() can set it.
     */
    DbgPrint(b"[CMD] Calling Initialize()\n\0".as_ptr() as PCSTR);
    pCmdLine = Initialize();
    nExitCode = nErrorLevel;
    DbgPrint(
        b"[CMD] Initialize() done, pCmdLine=%p nErrorLevel=%d\n\0".as_ptr() as PCSTR,
        pCmdLine,
        nErrorLevel,
    );

    if !pCmdLine.is_null() && *pCmdLine != 0 {
        let mut commandline: [TCHAR; (CMDLINE_LENGTH) as _] = [0; (CMDLINE_LENGTH) as _];

        /* Do the /C or /K command */
        DbgPrint(b"[CMD] Executing /C command\n\0".as_ptr() as PCSTR);
        GetCmdLineCommand(
            commandline.as_mut_ptr(),
            pCmdLine.offset(2 as isize),
            bAlwaysStrip,
        );
        nExitCode = ParseCommandLine(commandline.as_mut_ptr());
        DbgPrint(
            b"[CMD] ParseCommandLine returned %d\n\0".as_ptr() as PCSTR,
            nExitCode,
        );
        if fSingleCommand == 1 {
            // nErrorLevel = nExitCode;
            bExit = (TRUE) as i32;
        }
        fSingleCommand = 0;
    }
    if bExit == 0 {
        /* Call prompt routine */
        nExitCode = ProcessInput();
    }

    /* Do the cleanup */
    DbgPrint(b"[CMD] Cleanup starting\n\0".as_ptr() as PCSTR);
    Cleanup();
    cmd_free!(lpOriginalEnvironment as *mut core::ffi::c_void);

    DbgPrint(b"[CMD] Exiting with code %d\n\0".as_ptr() as PCSTR, nExitCode);
    /* ARM64: bypass ExitProcess DLL cleanup which crashes in RtlDestroyHeap
     * due to stack page being unmapped during process teardown. Just terminate. */
    #[cfg(target_arch = "aarch64")]
    NtTerminateProcess(NtCurrentProcess(), nExitCode);
    cmd_exit!((nExitCode) as u32);
    return nExitCode;
}
