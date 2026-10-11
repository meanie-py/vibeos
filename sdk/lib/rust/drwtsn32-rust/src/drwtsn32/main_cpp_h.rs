use crate::prelude::*;
use crate::drwtsn32::drwtsn32_cpp_h::UpdateFromEvent;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0406C89F1409: [WCHAR; 3] = [0x002D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_06EC47F61DB3: [WCHAR; 32] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005C, 0x0043, 0x0072, 0x0061, 0x0073, 0x0068, 0x0020, 0x0052, 0x0065, 0x0070, 0x006F, 0x0072, 0x0074, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_1C468647B3E9: [WCHAR; 15] = [0x0044, 0x0075, 0x006D, 0x0070, 0x0020, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000];
pub static CPP_COM_WSTR_1EA9E69A696A: [WCHAR; 3] = [0x0077, 0x0062, 0x0000];
pub static CPP_COM_WSTR_5462AED96D21: [WCHAR; 3] = [0x002F, 0x003F, 0x0000];
pub static CPP_COM_WSTR_5CBD29E18072: [WCHAR; 3] = [0x002D, 0x0067, 0x0000];
pub static CPP_COM_WSTR_795916488012: [WCHAR; 3] = [0x002D, 0x003F, 0x0000];
pub static CPP_COM_WSTR_8CC343C4CE5F: [WCHAR; 3] = [0x002D, 0x0069, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_B6F67A34D720: [WCHAR; 5] = [0x002E, 0x0064, 0x006D, 0x0070, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_B922C861BCDA: [WCHAR; 9] = [0x004D, 0x0069, 0x006E, 0x0069, 0x0064, 0x0075, 0x006D, 0x0070, 0x0000];
pub static CPP_COM_WSTR_E6430EF23A83: [WCHAR; 41] = [0x0041, 0x0070, 0x0070, 0x0063, 0x0072, 0x0061, 0x0073, 0x0068, 0x005F, 0x0025, 0x0064, 0x002D, 0x0025, 0x0030, 0x0032, 0x0064, 0x002D, 0x0025, 0x0030, 0x0032, 0x0064, 0x005F, 0x0025, 0x0030, 0x0032, 0x0064, 0x002D, 0x0025, 0x0030, 0x0032, 0x0064, 0x002D, 0x0025, 0x0030, 0x0032, 0x0064, 0x002E, 0x0074, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_FFAFD82EDE59: [WCHAR; 3] = [0x002D, 0x0070, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szUsage: [u8; 261] = *b"Usage: DrWtsn32 [-i] [-g] [-p dddd] [-e dddd] [-?]\n    -i: Install DrWtsn32 as the postmortem debugger\n    -g: Ignored, Provided for compatibility with WinDbg and CDB.\n    -p dddd: Attach to process dddd.\n    -e dddd: Signal the event dddd.\n    -?: This help.\n\0";

#[no_mangle]
pub unsafe extern "C" fn xfprintf_v(stream: *mut FILE, fmt: *const i8, ap_list: *mut ::core::ffi::c_void) {
    let mut ap: va_list = *(ap_list as *mut va_list);


    vfprintf(stream, fmt, ap);
    vDbgPrintEx((1 as u32).wrapping_neg(), DPFLTR_ERROR_LEVEL, fmt, ap);

}



unsafe extern "C" fn SortModules(left: &ModuleData, right: &ModuleData) -> bool {
    return left.BaseAddress < right.BaseAddress;
}

unsafe extern "C" fn PrintThread(output: *mut FILE, data: &mut DumpData, tid: DWORD, thread: &mut ThreadData) {
    thread.Update();

    xfprintf(output, b"\r\nState Dump for Thread Id 0x%x%s\r\n\r\n\0".as_ptr() as *const i8, tid,
             if tid == data.ThreadID { b" (CRASH)\0".as_ptr() as *const i8 } else { b"\0".as_ptr() as *const i8 });

    // The C++ is one `#if defined(_M_IX86) / _M_AMD64 / _M_ARM64` chain per register group;
    // each arch prints its own CONTEXT, so each gets its own cfg branch here.
    let ctx: &CONTEXT = &thread.Context;
    if (ctx.ContextFlags & CONTEXT_INTEGER) == CONTEXT_INTEGER
    {
        #[cfg(target_arch = "x86")]
        xfprintf(output, b"eax:%p ebx:%p ecx:%p edx:%p esi:%p edi:%p\r\n\0".as_ptr() as *const i8,
                 ctx.Eax, ctx.Ebx, ctx.Ecx, ctx.Edx, ctx.Esi, ctx.Edi);
        #[cfg(target_arch = "x86_64")]
        {
            xfprintf(output, b"rax:%p rbx:%p rcx:%p rdx:%p rsi:%p rdi:%p rbp:%p rsp:%p\r\n\0".as_ptr() as *const i8,
                     ctx.Rax, ctx.Rbx, ctx.Rcx, ctx.Rdx, ctx.Rsi, ctx.Rdi, ctx.Rbp, ctx.Rsp);
            xfprintf(output, b"r8:%p r9:%p r10:%p r11:%p r12:%p r13:%p r14:%p r15:%p\r\n\0".as_ptr() as *const i8,
                     ctx.R8, ctx.R9, ctx.R10, ctx.R11, ctx.R12, ctx.R13, ctx.R14, ctx.R15);
        }
        #[cfg(target_arch = "aarch64")]
        {
            let x = &ctx.__bindgen_anon_1.X;
            xfprintf(output, b"x0:%p x1:%p x2:%p x3:%p x4:%p x5:%p x6:%p x7:%p\r\n\0".as_ptr() as *const i8,
                     x[0] as ULONG_PTR, x[1] as ULONG_PTR, x[2] as ULONG_PTR, x[3] as ULONG_PTR,
                     x[4] as ULONG_PTR, x[5] as ULONG_PTR, x[6] as ULONG_PTR, x[7] as ULONG_PTR);
            xfprintf(output, b"x8:%p x9:%p x10:%p x11:%p x12:%p x13:%p x14:%p x15:%p\r\n\0".as_ptr() as *const i8,
                     x[8] as ULONG_PTR, x[9] as ULONG_PTR, x[10] as ULONG_PTR, x[11] as ULONG_PTR,
                     x[12] as ULONG_PTR, x[13] as ULONG_PTR, x[14] as ULONG_PTR, x[15] as ULONG_PTR);
            xfprintf(output, b"x16:%p x17:%p x18:%p\r\n\0".as_ptr() as *const i8,
                     x[16] as ULONG_PTR, x[17] as ULONG_PTR, x[18] as ULONG_PTR);
        }
    }

    if (ctx.ContextFlags & CONTEXT_CONTROL) == CONTEXT_CONTROL
    {
        #[cfg(target_arch = "x86")]
        xfprintf(output, b"eip:%p esp:%p ebp:%p\r\n\0".as_ptr() as *const i8,
                 ctx.Eip, ctx.Esp, ctx.Ebp);
        #[cfg(target_arch = "x86_64")]
        xfprintf(output, b"rip:%p rsp:%p rbp:%p\r\n\0".as_ptr() as *const i8,
                 ctx.Rip, ctx.Rsp, ctx.Rbp);
        #[cfg(target_arch = "aarch64")]
        {
            // Fp and Lr are X29 and X30 of the same union.
            let x = &ctx.__bindgen_anon_1.X;
            xfprintf(output, b"sp:%p lr:%p pc:%p fp:%p cpsr:%p\r\n\0".as_ptr() as *const i8,
                     ctx.Sp as ULONG_PTR, x[30] as ULONG_PTR, ctx.Pc as ULONG_PTR,
                     x[29] as ULONG_PTR, ctx.Cpsr as ULONG_PTR);
        }
    }

    if (ctx.ContextFlags & CONTEXT_DEBUG_REGISTERS) == CONTEXT_DEBUG_REGISTERS
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        xfprintf(output, b"dr0:%p dr1:%p dr2:%p dr3:%p dr6:%p dr7:%p\r\n\0".as_ptr() as *const i8,
                 ctx.Dr0, ctx.Dr1, ctx.Dr2, ctx.Dr3, ctx.Dr6, ctx.Dr7);
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64_MAX_BREAKPOINTS / ARM64_MAX_WATCHPOINTS are the CONTEXT array lengths.
            let sep = |n: usize, max: usize| -> *const i8 {
                if n + 1 == max { b"\r\n\0".as_ptr() as *const i8 } else { b" \0".as_ptr() as *const i8 }
            };
            let (nb, nw) = (ctx.Bvr.len(), ctx.Wvr.len());
            for n in 0..nb {
                xfprintf(output, b"Bvr%d:%p%s\0".as_ptr() as *const i8, n as c_int, ctx.Bvr[n] as ULONG_PTR, sep(n, nb));
            }
            for n in 0..nb {
                xfprintf(output, b"Bcr%d:%p%s\0".as_ptr() as *const i8, n as c_int, ctx.Bcr[n], sep(n, nb));
            }
            for n in 0..nw {
                xfprintf(output, b"Wvr%d:%p%s\0".as_ptr() as *const i8, n as c_int, ctx.Wvr[n] as ULONG_PTR, sep(n, nw));
            }
            for n in 0..nw {
                xfprintf(output, b"Wcr%d:%p%s\0".as_ptr() as *const i8, n as c_int, ctx.Wcr[n], sep(n, nw));
            }
        }
    }

    PrintStackBacktrace(output, data, thread);
}
#[no_mangle]
pub unsafe extern "C" fn PrintBugreport(output: *mut FILE, data: &mut DumpData) {
    PrintSystemInfo(output, data);
    xfprintf(output, b"\r\n*----> Task List <----*\r\n\r\n\0".as_ptr() as *const i8);
    let hSnap: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if hSnap != INVALID_HANDLE_VALUE {
        let mut pe: PROCESSENTRY32 = core::mem::zeroed();
        pe.dwSize = core::mem::size_of::<PROCESSENTRY32>() as DWORD;
        if Process32First(hSnap, core::ptr::addr_of_mut!(pe)) != 0 {
            loop {
                xfprintf(
                    output,
                    b"%5d: %ls\r\n\0".as_ptr() as *const i8,
                    pe.th32ProcessID,
                    pe.szExeFile.as_ptr(),
                );
                if Process32Next(hSnap, core::ptr::addr_of_mut!(pe)) == 0 {
                    break;
                }
            }
        }
        CloseHandle(hSnap);
    }

    xfprintf(output, b"\r\n*----> Module List <----*\r\n\r\n\0".as_ptr() as *const i8);
    data.Modules.sort_by_less(|l, r| SortModules(l, r));

    let mut mainModule: ModuleData = ModuleData__make_0(core::ptr::null_mut());
    mainModule.Update(data.ProcessHandle);
    xfprintf(
        output,
        b"(%p - %p) %ls\r\n\0".as_ptr() as *const i8,
        mainModule.BaseAddress,
        (mainModule.BaseAddress as PBYTE).add(mainModule.Size as usize),
        data.ProcessPath.c_str(),
    );

    let mut n: usize = 0;
    while n < data.Modules.size() {
        let r#mod: &mut ModuleData = &mut *data.Modules.at(n);
        if !r#mod.Unloaded {
            r#mod.Update(data.ProcessHandle);
            xfprintf(
                output,
                b"(%p - %p) %s\r\n\0".as_ptr() as *const i8,
                r#mod.BaseAddress,
                (r#mod.BaseAddress as PBYTE).add(r#mod.Size as usize),
                r#mod.ModuleName.c_str(),
            );
        }
        n += 1;
    }

    BeginStackBacktrace(data);

    // First print the thread that crashed, then the others, in key order (std::map's).
    // PrintThread takes the map's element by reference while `data` is borrowed too, as the
    // C++ does; it reads `data`, never the map, so the element stays put.
    let dp: *mut DumpData = data;
    let crash = (*dp).Threads.find(&(*dp).ThreadID);
    if !crash.is_null() {
        PrintThread(output, &mut *dp, (*dp).ThreadID, &mut *crash);
    }
    let count = (*dp).Threads.size();
    for i in 0..count {
        let it: *mut StdPair<DWORD, ThreadData> = (*dp).Threads.as_mut_slice().as_mut_ptr().add(i);
        if (*it).first != (*dp).ThreadID {
            PrintThread(output, &mut *dp, (*it).first, &mut (*it).second);
        }
    }
    EndStackBacktrace(data);
}
// A C++ overload of the CRT's name: mangled in C++, so not exported here.
pub unsafe extern "C" fn abort(output: *mut FILE, err: i32) -> i32 {
    if output != (__iob_func().add(1)) {
        fclose(output);
    } else {
        _getch();
    }

    return err;
}
#[no_mangle]
pub unsafe extern "C" fn Settings_GetOutputPath() -> StdWString {
    let mut Buffer: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();
    let mut BufferSize: ULONG = _countof!(Buffer);
    let mut UseDefaultPath: BOOL = (TRUE as i32);

    let mut key: CRegKey = CRegKey__make_0();
    if key.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_06EC47F61DB3.as_ptr(), KEY_READ) == (ERROR_SUCCESS as i32) &&
        key.QueryStringValue(CPP_COM_WSTR_1C468647B3E9.as_ptr(), Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(BufferSize)) == (ERROR_SUCCESS as i32)
    {
        UseDefaultPath = (FALSE as i32);
    }

    if UseDefaultPath != 0 {
        if (SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_DESKTOP as i32), core::ptr::null_mut(), SHGFP_TYPE_CURRENT, Buffer.as_mut_ptr())) < 0
        {
            return StdWString::new();
        }
    }

    return StdWString::from_ptr(Buffer.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn Settings_GetShouldWriteDump() -> BOOL {
    let mut key = CRegKey__make_0();
    if key.Open(HKEY_CURRENT_USER, CPP_COM_WSTR_06EC47F61DB3.as_ptr(), KEY_READ) != (ERROR_SUCCESS as i32) {
        return (FALSE) as BOOL;
    }

    let mut Value: DWORD = 0;
    if key.QueryDWORDValue(CPP_COM_WSTR_B922C861BCDA.as_ptr(), &mut Value) != (ERROR_SUCCESS as i32) {
        return (FALSE) as BOOL;
    }

    return (Value != 0) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn WriteMinidump(LogFilePath: LPCWSTR, data: &mut DumpData) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    let mut DumpFilePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize]; // = L""
    StringCchCopyW(DumpFilePath.as_mut_ptr(), (_countof!(DumpFilePath)) as _, LogFilePath);
    PathRemoveExtensionW(DumpFilePath.as_mut_ptr());
    PathAddExtensionW(DumpFilePath.as_mut_ptr(), CPP_COM_WSTR_B6F67A34D720.as_ptr());

    let hDumpFile: HANDLE = CreateFileW(
        DumpFilePath.as_mut_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hDumpFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    // `data.Threads[data.ThreadID]`: std::map's operator[] default-constructs a missing entry
    // (ThreadData(NULL)) and returns it.
    let mut tp = data.Threads.find(&data.ThreadID);
    if tp.is_null() {
        data.Threads.insert(data.ThreadID, ThreadData__make_0(core::ptr::null_mut()));
        tp = data.Threads.find(&data.ThreadID);
    }
    let Thread: &mut ThreadData = &mut *tp;
    Thread.Update();
    let ContextPointer: PCONTEXT = core::ptr::addr_of_mut!(Thread.Context);

    let mut DumpExceptionInfo: MINIDUMP_EXCEPTION_INFORMATION = core::mem::zeroed();
    let mut ExceptionPointers: EXCEPTION_POINTERS = core::mem::zeroed();
    ExceptionPointers.ExceptionRecord = core::ptr::addr_of_mut!(data.ExceptionInfo.ExceptionRecord);
    ExceptionPointers.ContextRecord = ContextPointer;

    DumpExceptionInfo.ThreadId = data.ThreadID;
    DumpExceptionInfo.ExceptionPointers = core::ptr::addr_of_mut!(ExceptionPointers);
    DumpExceptionInfo.ClientPointers = (FALSE as i32);

    let DumpSucceeded: BOOL = MiniDumpWriteDump(
        data.ProcessHandle,
        data.ProcessID,
        hDumpFile,
        MiniDumpNormal,
        core::ptr::addr_of_mut!(DumpExceptionInfo),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if DumpSucceeded == 0 {
        // According to MSDN, this value is already an HRESULT, so don't convert it again.
        hr = (GetLastError()) as i32;
    }

    CloseHandle(hDumpFile);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, _hPrevInstance: HINSTANCE, cmdLine: LPWSTR, _nShowCmd: INT) -> i32 {
    let mut argc: i32 = 0;
    let argv: *mut *mut WCHAR = CommandLineToArgvW(GetCommandLineW(), core::ptr::addr_of_mut!(argc));

    let mut pid: DWORD = 0;
    let mut Filename: [WCHAR; 50] = [0 as WCHAR; 50];
    let mut output: *mut FILE = core::ptr::null_mut();
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut data: DumpData = DumpData__make_0();

    let mut n: i32 = 0;
    while n < argc {
        let arg: *mut WCHAR = *argv.offset(n as isize);

        if wcscmp(arg, CPP_COM_WSTR_8CC343C4CE5F.as_ptr()) == 0 {
            /* FIXME: Installs as the postmortem debugger. */
        }
        else if wcscmp(arg, CPP_COM_WSTR_5CBD29E18072.as_ptr()) == 0 {
        }
        else if wcscmp(arg, CPP_COM_WSTR_FFAFD82EDE59.as_ptr()) == 0 {
            if n + 1 < argc {
                pid = wcstoul(*argv.offset((n + 1) as isize), core::ptr::null_mut(), 10) as DWORD;
                n += 1;
            }
        }
        else if wcscmp(arg, CPP_COM_WSTR_0406C89F1409.as_ptr()) == 0 {
            if n + 1 < argc {
                data.Event = _wcstoui64(*argv.offset((n + 1) as isize), core::ptr::null_mut(), 10) as usize as HANDLE;
                n += 1;
            }
        }
        else if wcscmp(arg, CPP_COM_WSTR_795916488012.as_ptr()) == 0 {
            MessageBoxA(core::ptr::null_mut(), szUsage.as_ptr() as *const i8, b"ReactOS Crash Reporter\0".as_ptr() as *const i8, MB_OK);
            return abort(output, 0);
        }
        else if wcscmp(arg, CPP_COM_WSTR_5462AED96D21.as_ptr()) == 0 {
            xfprintf(__iob_func().add(1), b"%s\n\0".as_ptr() as *const i8, szUsage.as_ptr() as *const i8);
            return abort(__iob_func().add(1), 0);
        }
        n += 1;
    }

    if pid == 0 {
        MessageBoxA(core::ptr::null_mut(), szUsage.as_ptr() as *const i8, b"ReactOS Crash Reporter\0".as_ptr() as *const i8, MB_OK);
        return abort(__iob_func().add(1), 0);
    }

    GetLocalTime(core::ptr::addr_of_mut!(st));

    let mut OutputPath: StdWString = Settings_GetOutputPath();
    let HasPath: BOOL = (OutputPath.size() != 0) as BOOL;

    if PathIsDirectoryW(OutputPath.c_str()) == 0 {
        let res: i32 = SHCreateDirectoryExW(core::ptr::null_mut(), OutputPath.c_str(), core::ptr::null_mut());
        if res != (ERROR_SUCCESS as i32) && res != (ERROR_ALREADY_EXISTS as i32) {
            xfprintf(__iob_func().add(1), b"Could not create output directory, not writing dump\n\0".as_ptr() as *const i8);
            MessageBoxA(core::ptr::null_mut(), b"Could not create directory to write crash report.\0".as_ptr() as *const i8, b"ReactOS Crash Reporter\0".as_ptr() as *const i8, MB_ICONERROR | MB_OK);
            return abort(__iob_func().add(1), 0);
        }
    }

    if HasPath != 0 &&
        (StringCchPrintfW!(Filename.as_mut_ptr(), _countof!(Filename), CPP_COM_WSTR_E6430EF23A83.as_ptr(),
                                st.wYear as i32, st.wMonth as i32, st.wDay as i32, st.wHour as i32, st.wMinute as i32, st.wSecond as i32) >= 0)
    {
        OutputPath += CPP_COM_WSTR_B7FC02907B1F.as_ptr();
        OutputPath += Filename.as_ptr();
        output = (_wfopen(OutputPath.c_str(), CPP_COM_WSTR_1EA9E69A696A.as_ptr())).cast();
    }
    if output.is_null() {
        output = __iob_func().add(1);
    }

    if DebugActiveProcess(pid) == 0 {
        return abort(output, -2);
    }

    /* We should not kill it? */
    DebugSetProcessKillOnExit((FALSE) as BOOL);

    let mut evt: DEBUG_EVENT = core::mem::zeroed();
    if WaitForDebugEvent(core::ptr::addr_of_mut!(evt), 30000) == 0 {
        return abort(output, -3);
    }

    assert!(evt.dwDebugEventCode == CREATE_PROCESS_DEBUG_EVENT);

    while UpdateFromEvent(&mut evt, &mut data) != false {
        ContinueDebugEvent(evt.dwProcessId, evt.dwThreadId, (DBG_CONTINUE) as DWORD);

        if WaitForDebugEvent(core::ptr::addr_of_mut!(evt), 30000) == 0 {
            return abort(output, -4);
        }
    }

    PrintBugreport(output, &mut data);
    if Settings_GetShouldWriteDump() != 0 && HasPath != 0 {
        WriteMinidump(OutputPath.c_str(), &mut data);
    }

    TerminateProcess(data.ProcessHandle, data.ExceptionInfo.ExceptionRecord.ExceptionCode);

    let mut FormattedMessage: CStringW = CStringW::new();
    atl_format_id!(FormattedMessage, IDS_USER_ALERT_MESSAGE, data.ProcessName.c_str(), OutputPath.c_str());
    let mut DialogTitle: CStringW = CStringW::new();
    DialogTitle.LoadStringW(IDS_APP_TITLE);

    MessageBoxW(core::ptr::null_mut(), FormattedMessage.GetString(), DialogTitle.GetString(), MB_OK);

    return abort(output, 0);
}
