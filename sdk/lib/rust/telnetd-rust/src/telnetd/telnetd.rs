use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `static int sock` in the C. Renamed `g_sock` here only because two functions take an `int sock`
// PARAMETER that shadows it (legal in C, but E0530 in Rust — a static cannot be shadowed by a
// binding pattern). Storage class and file-local linkage are unchanged.
static mut bShutdown: BOOLEAN = 0;
static mut bSocketInterfaceInitialised: BOOLEAN = 0;
static mut g_sock: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn kickoff_telnetd() -> c_int {
    printf(b"Attempting to start Simple TelnetD\n\0".as_ptr() as *const c_char);
    SetConsoleCtrlHandler(Some(Cleanup), 1);
    if StartSocketInterface() == 0 {
        ErrorExit(b"Unable to start socket interface\n\0".as_ptr() as *const c_char);
    }
    CreateSocket();
    while bShutdown == 0 {
        WaitForConnect();
    }
    WSACleanup();
    0
}

// WINAPI. The x86 (`extern "stdcall"`) and non-x86 (`extern "C"`) copies of this and the four
// thread routines below had byte-identical bodies, so each pair is collapsed into one
// `extern "system"` definition — stdcall on i386, cdecl elsewhere, one body, and no fn-pointer
// transmute at the CreateThread / SetConsoleCtrlHandler call sites.
unsafe extern "system" fn Cleanup(dwControlType: DWORD) -> BOOL {
    if bSocketInterfaceInitialised != 0 {
        telnetd_printf(b"Cleanup...\n\0".as_ptr() as *const c_char);
        WSACleanup();
    }
    0
}
unsafe extern "C" fn StartSocketInterface() -> BOOLEAN {
    let wVersionRequested: WORD = MAKEWORD(2, 0);
    let mut wsaData: WSADATA = core::mem::zeroed();
    let err: c_int = WSAStartup(wVersionRequested, &mut wsaData);
    if err != 0 {
        telnetd_printf(
            b"requested winsock version not supported\n\0".as_ptr() as *const c_char,
        );
        return 0;
    }
    bSocketInterfaceInitialised = 1;
    if wsaData.wVersion != wVersionRequested {
        ErrorExit(
            b"requested winsock version not supported\n\0".as_ptr() as *const c_char,
        );
    }
    telnetd_printf(
        b"TelnetD, using %s\n\0".as_ptr() as *const c_char,
        wsaData.szDescription.as_ptr(),
    );
    1
}
unsafe extern "C" fn CreateSocket() {
    let mut sa: sockaddr_in = core::mem::zeroed();

    // The C keeps sockets in `int`; on Win64 SOCKET is 64-bit, so this narrows exactly as the C
    // compiler's implicit conversion does. Preserved rather than "fixed" — see the report.
    g_sock = socket(AF_INET as c_int, SOCK_STREAM as c_int, IPPROTO_TCP as c_int) as c_int;
    if g_sock < 0 {
        ErrorExit(b"Cannot create socket\0".as_ptr() as *const c_char);
    }

    sa.sin_family = AF_INET as c_short;
    sa.sin_addr.S_un.S_addr = INADDR_ANY; // `s_addr` is a winsock macro for `S_un.S_addr`
    sa.sin_port = htons(TELNET_PORT as u_short);

    if bind(
        g_sock as SOCKET,
        &sa as *const sockaddr_in as *const sockaddr,
        core::mem::size_of::<sockaddr_in>() as c_int,
    ) != 0
    {
        ErrorExit(b"Cannot bind address to socket\0".as_ptr() as *const c_char);
    }
}
unsafe extern "C" fn WaitForConnect() {
    let mut sa: sockaddr_in = core::mem::zeroed();
    let new_sock: c_int;

    if listen(g_sock as SOCKET, 1) < 0 {
        ErrorExit(b"Cannot listen on socket\0".as_ptr() as *const c_char);
    }

    new_sock = accept(
        g_sock as SOCKET,
        &mut sa as *mut sockaddr_in as *mut sockaddr,
        core::ptr::null_mut(),
    ) as c_int;

    if new_sock < 0 {
        fprintf(
            stderr(),
            b"Failed to accept incoming call\n\0".as_ptr() as *const c_char,
        );
    } else {
        telnetd_printf(
            b"user connected on socket %d, port %d, address %lx\n\0".as_ptr() as *const c_char,
            new_sock,
            htons(sa.sin_port) as c_int, // C default argument promotion: u_short -> int
            sa.sin_addr.S_un.S_addr,
        );
        UserLogin(new_sock);
    }
}
unsafe extern "C" fn UserLogin(client_socket: c_int) {
    let client: *mut client_t = malloc(core::mem::size_of::<client_t>()) as *mut client_t;
    if client.is_null() {
        ErrorExit(b"failed to allocate memory for client\0".as_ptr() as *const c_char);
    }
    (*client).socket = client_socket;
    let thread_handle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(UserLoginThread),
        client as LPVOID,
        0,
        core::ptr::null_mut(),
    );
    if thread_handle.is_null() {
        free(client as *mut c_void);
    } else {
        CloseHandle(thread_handle);
    }
}
unsafe extern "system" fn UserLoginThread(data: LPVOID) -> DWORD {
    let client = data as *mut client_t;
    let mut hostname: [c_char; 64] = [0; 64];
    // default "Unknown"
    core::ptr::copy_nonoverlapping(
        b"Unknown\0".as_ptr() as *const c_char,
        hostname.as_mut_ptr(),
        8,
    );
    let mut welcome: [c_char; 256] = [0; 256];
    let pwd_prompt = b"\r\npass:\0".as_ptr() as *const c_char;
    let mut user_id: [c_char; USERID_SIZE as usize] = [0; USERID_SIZE as usize];
    let mut password: [c_char; USERID_SIZE as usize] = [0; USERID_SIZE as usize];

    if DoTelnetHandshake((*client).socket) != 0 {
        closesocket((*client).socket as SOCKET);
        free(client as *mut c_void);
        return 0;
    }

    gethostname(
        hostname.as_mut_ptr(),
        core::mem::size_of::<[c_char; 64]>() as c_int,
    );

    sprintf(
        welcome.as_mut_ptr(),
        b"\r\nWelcome to %s, please identify yourself\r\n\r\nuser:\0".as_ptr() as *const c_char,
        hostname.as_ptr(),
    );

    if send(
        (*client).socket as SOCKET,
        welcome.as_ptr(),
        strlen(welcome.as_ptr()) as c_int,
        0,
    ) < 0
    {
        closesocket((*client).socket as SOCKET);
        free(client as *mut c_void);
        return 0;
    }

    let received = ReceiveLine(
        (*client).socket,
        user_id.as_mut_ptr(),
        core::mem::size_of::<[c_char; USERID_SIZE as usize]>() as c_int,
        EchoMode_Echo,
    );

    if received < 0 {
        closesocket((*client).socket as SOCKET);
        free(client as *mut c_void);
        return 0;
    } else if received != 0 {
        let terminator =
            strchr(user_id.as_mut_ptr() as *const c_char, CR as c_int) as *mut c_char;
        if !terminator.is_null() {
            *terminator = 0;
        }
    }

    if send(
        (*client).socket as SOCKET,
        pwd_prompt,
        strlen(pwd_prompt) as c_int,
        0,
    ) < 0
    {
        closesocket((*client).socket as SOCKET);
        free(client as *mut c_void);
        return 0;
    }

    let _ = ReceiveLine(
        (*client).socket,
        password.as_mut_ptr(),
        core::mem::size_of::<[c_char; USERID_SIZE as usize]>() as c_int,
        EchoMode_Password,
    );

    telnetd_printf(
        b"User '%p' logged on\n\0".as_ptr() as *const c_char,
        user_id.as_ptr(),
    );

    RunShell(client);
    0
}
unsafe extern "C" fn DoTelnetHandshake(sock: c_int) -> c_int {
    let mut set: fd_set = core::mem::zeroed();
    let mut timeout = timeval {
        tv_sec: HANDSHAKE_TIMEOUT as LONG,
        tv_usec: 0,
    };

    // telnetd.h spells these as one-char string literals, which bindgen emits as `&[u8; 2]`; the C
    // concatenates them into a `char[]`, so index 0 is the byte that goes on the wire. The array
    // is 33 negotiation bytes PLUS the string literal's NUL, because the C sends `sizeof()`.
    let will_echo: [u8; 34] = [
        IAC[0], DONT[0], ECHO[0],
        IAC[0], WILL[0], ECHO[0],
        IAC[0], WILL[0], NAWS[0],
        IAC[0], WILL[0], SUPPRESS_GO_AHEAD[0],
        IAC[0], DO[0], SUPPRESS_GO_AHEAD[0],
        IAC[0], DONT[0], NEWENVIRON[0],
        IAC[0], WONT[0], NEWENVIRON[0],
        IAC[0], WONT[0], LINEMODE[0],
        IAC[0], DO[0], NAWS[0],
        IAC[0], SB[0], TERMINAL_TYPE[0], 0x01, IAC[0], SE[0],
        0,
    ];

    let mut client_reply = [0u8; 256];
    let mut retval: c_int;
    let mut received: c_int;

    if send(
        sock as SOCKET,
        will_echo.as_ptr() as *const c_char,
        will_echo.len() as c_int,
        0,
    ) < 0
    {
        return -1;
    }

    FD_ZERO(&mut set);
    FD_SET(sock as SOCKET, &mut set);

    loop {
        retval = select(
            0,
            &mut set,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &timeout,
        );
        if retval < 0 {
            return -1;
        }
        if retval == 0 {
            return 0;
        }

        received = recv(
            sock as SOCKET,
            client_reply.as_mut_ptr() as *mut c_char,
            client_reply.len() as c_int,
            0,
        );
        if received <= 0 {
            return -1;
        }
    }
}
unsafe extern "C" fn ReceiveLine(sock: c_int, buffer: *mut c_char, len: c_int, echo: EchoMode) -> c_int {
    let mut i: c_int = 0;
    let mut set: fd_set = core::mem::zeroed();
    let timeout = timeval { tv_sec: 0, tv_usec: 100000 };
    const BS: u8 = 8;
    const DEL: u8 = 0x7F;
    let del: [u8; 3] = [BS, b' ', BS];
    let asterisk: [u8; 1] = [b'*'];

    FD_ZERO(&mut set as *mut fd_set);
    FD_SET(sock as SOCKET, &mut set as *mut fd_set);

    core::ptr::write_bytes(buffer as *mut u8, 0, len as usize);

    while i < len {
        // C: `(echo ? NULL : &timeout)` — ANY non-zero mode (Echo *and* Password) blocks with no
        // timeout. Testing truthiness, not `== Echo`, is what the C does.
        let timeout_ptr = if echo != 0 {
            core::ptr::null()
        } else {
            &timeout as *const timeval
        };
        let retval = select(
            0,
            &mut set as *mut fd_set,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            timeout_ptr,
        );
        if retval < 0 {
            return -1;
        } else if retval == 0 {
            return i;
        }

        if recv(sock as SOCKET, buffer.add(i as usize), 1, 0) <= 0 {
            return -1;
        }
        let c = *buffer.add(i as usize) as u8;

        if c == 0 || c == b'\n' {
            *buffer.add(i as usize) = 0;
        } else if c == DEL || c == BS {
            *buffer.add(i as usize) = 0;
            if echo != 0 {
                if i > 0 {
                    i -= 1;
                    *buffer.add(i as usize) = 0;
                    if send(sock as SOCKET, del.as_ptr() as *const c_char, del.len() as c_int, 0) < 0 {
                        return -1;
                    }
                }
            } else {
                *buffer.add(i as usize) = BS as c_char;
                i += 1;
            }
        } else {
            if echo == EchoMode_Echo {
                if send(sock as SOCKET, buffer.add(i as usize), 1, 0) < 0 {
                    return -1;
                }
            } else if echo == EchoMode_Password {
                if send(sock as SOCKET, asterisk.as_ptr() as *const c_char, asterisk.len() as c_int, 0) < 0 {
                    return -1;
                }
            }
            if c == CR as u8 {
                i += 1;
                *buffer.add(i as usize) = LF as c_char;
                i += 1;
                return i;
            }
            i += 1;
        }
    }

    i
}
unsafe extern "C" fn RunShell(client: *mut client_t) {
    let mut threadHandle: HANDLE;
    let mut hChildStdinRd: HANDLE = core::ptr::null_mut();
    let mut hChildStdinWr: HANDLE = core::ptr::null_mut();
    let mut hChildStdoutRd: HANDLE = core::ptr::null_mut();
    let mut hChildStdoutWr: HANDLE = core::ptr::null_mut();
    let mut si: STARTUPINFO = core::mem::zeroed();
    let mut piProcInfo: PROCESS_INFORMATION = core::mem::zeroed();
    let mut saAttr: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut cmd_path: [c_char; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if GetEnvironmentVariableA(
        b"COMSPEC\0".as_ptr() as *const c_char,
        cmd_path.as_mut_ptr(),
        cmd_path.len() as DWORD,
    ) == 0
    {
        if GetSystemDirectoryA(
            cmd_path.as_mut_ptr(),
            cmd_path.len() as UINT,
        ) != 0
        {
            StringCchCatA(
                cmd_path.as_mut_ptr(),
                cmd_path.len(),
                b"\\cmd.exe\0".as_ptr() as *const c_char,
            );
        } else {
            ErrorExit(b"GetSystemDirectoryA failed\n\0".as_ptr() as *const c_char);
        }
    }

    saAttr.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    saAttr.bInheritHandle = TRUE as BOOL;
    saAttr.lpSecurityDescriptor = core::ptr::null_mut();

    if CreatePipe(
        &mut hChildStdoutRd,
        &mut hChildStdoutWr,
        &mut saAttr,
        0,
    ) == 0
    {
        ErrorExit(b"Stdout pipe creation failed\n\0".as_ptr() as *const c_char);
    }

    if CreatePipe(
        &mut hChildStdinRd,
        &mut hChildStdinWr,
        &mut saAttr,
        0,
    ) == 0
    {
        ErrorExit(b"Stdin pipe creation failed\n\0".as_ptr() as *const c_char);
    }

    (*client).bTerminate = FALSE as BOOLEAN;
    (*client).bWriteToPipe = TRUE as BOOLEAN;
    (*client).bReadFromPipe = TRUE as BOOLEAN;
    (*client).hChildStdinWr = hChildStdinWr;
    (*client).hChildStdoutRd = hChildStdoutRd;

    telnetd_printf(b"Creating child process...\n\0".as_ptr() as *const c_char);

    // ZeroMemory(&si, sizeof(STARTUPINFO)): write_bytes' count is in ELEMENTS, not bytes — one
    // STARTUPINFO. (Passing size_of() here would zero size_of()^2 bytes and smash the stack.)
    core::ptr::write_bytes(&mut si as *mut STARTUPINFO, 0, 1);
    si.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;

    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = hChildStdinRd;
    si.hStdOutput = hChildStdoutWr;
    si.hStdError = hChildStdoutWr;

    if CreateProcessA(
        cmd_path.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TRUE as BOOL,
        DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
        core::ptr::null_mut(),
        core::ptr::null(),
        &mut si,
        &mut piProcInfo,
    ) == 0
    {
        ErrorExit(b"Create process failed\0".as_ptr() as *const c_char);
    }

    (*client).hProcess = piProcInfo.hProcess;
    (*client).dwProcessId = piProcInfo.dwProcessId;

    telnetd_printf(
        b"New child created (groupid=%lu)\n\0".as_ptr() as *const c_char,
        (*client).dwProcessId,
    );

    if CloseHandle(hChildStdoutWr) == 0 {
        ErrorExit(b"Closing handle failed\0".as_ptr() as *const c_char);
    }

    if CloseHandle(hChildStdinRd) == 0 {
        ErrorExit(b"Closing handle failed\0".as_ptr() as *const c_char);
    }

    threadHandle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(WriteToPipeThread),
        client as *mut c_void,
        0,
        core::ptr::null_mut(),
    );
    if !threadHandle.is_null() {
        CloseHandle(threadHandle);
    }

    threadHandle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(ReadFromPipeThread),
        client as *mut c_void,
        0,
        core::ptr::null_mut(),
    );
    if !threadHandle.is_null() {
        CloseHandle(threadHandle);
    }

    threadHandle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(MonitorChildThread),
        client as *mut c_void,
        0,
        core::ptr::null_mut(),
    );
    if !threadHandle.is_null() {
        CloseHandle(threadHandle);
    }
}
unsafe extern "system" fn MonitorChildThread(data: LPVOID) -> DWORD {
    let client = data as *mut client_t;
    let mut exitCode: DWORD = 0;

    telnetd_printf(b"Monitor thread running...\n\0".as_ptr() as *const c_char);

    WaitForSingleObject((*client).hProcess, INFINITE);

    GetExitCodeProcess((*client).hProcess, &mut exitCode);
    telnetd_printf(b"Child process terminated with code %lx\n\0".as_ptr() as *const c_char, exitCode);

    (*client).bTerminate = TRUE as BOOLEAN;

    Sleep(500);

    CloseHandle((*client).hChildStdoutRd);
    CloseHandle((*client).hChildStdinWr);
    CloseHandle((*client).hProcess);

    closesocket((*client).socket as SOCKET);

    telnetd_printf(b"Waiting for all threads to give up..\n\0".as_ptr() as *const c_char);

    while (*client).bWriteToPipe != 0 || (*client).bReadFromPipe != 0 {
        telnetd_printf(b".\0".as_ptr() as *const c_char);
        fflush(stdout());
        Sleep(1000);
    }

    telnetd_printf(
        b"Cleanup for user '%s'\n\0".as_ptr() as *const c_char,
        (*client).userID.as_ptr(),
    );
    free(client as *mut c_void);
    0
}
unsafe extern "system" fn WriteToPipeThread(data: LPVOID) -> DWORD {
    let client = data as *mut client_t;
    let mut iRead: c_int;
    let mut dwWritten: DWORD = 0;
    let mut chBuf: [c_char; BUFSIZE as usize] = [0; BUFSIZE as usize];

    while (*client).bTerminate == 0 {
        iRead = ReceiveLine((*client).socket, chBuf.as_mut_ptr(), BUFSIZE as c_int, FALSE as EchoMode);
        if iRead < 0 {
            telnetd_printf(b"Client disconnect\n\0".as_ptr() as *const c_char);
            break;
        } else if iRead > 0 {
            if !strchr(chBuf.as_ptr() as *const c_char, CTRLC as c_int).is_null() {
                GenerateConsoleCtrlEvent(CTRL_C_EVENT, (*client).dwProcessId);
            }
            if send((*client).socket as SOCKET, chBuf.as_ptr(), iRead as c_int, 0) < 0 {
                telnetd_printf(b"error writing to socket\n\0".as_ptr() as *const c_char);
                break;
            }
            if WriteFile((*client).hChildStdinWr, chBuf.as_ptr() as *const c_void, iRead as DWORD, &mut dwWritten, core::ptr::null_mut()) == 0 {
                telnetd_printf(b"Error writing to pipe\n\0".as_ptr() as *const c_char);
                break;
            }
        }
    }

    if (*client).bTerminate == 0 {
        TerminateShell(client);
    }

    telnetd_printf(b"WriteToPipeThread terminated\n\0".as_ptr() as *const c_char);

    (*client).bWriteToPipe = FALSE as BOOLEAN;
    0
}
unsafe extern "system" fn ReadFromPipeThread(data: LPVOID) -> DWORD {
    let client = data as *mut client_t;
    let mut dwAvail: DWORD;
    let mut dwRead: DWORD = 0;
    let mut chBuf: [u8; BUFSIZE as usize] = [0u8; BUFSIZE as usize];
    let mut txBuf: [u8; BUFSIZE as usize * 2] = [0u8; BUFSIZE as usize * 2];
    let mut from: usize;
    let mut to: usize;

    while (*client).bTerminate == 0 && (*client).bWriteToPipe != 0 {
        dwAvail = 0;
        if PeekNamedPipe(
            (*client).hChildStdoutRd,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            &mut dwAvail,
            core::ptr::null_mut(),
        ) == 0
        {
            telnetd_printf(b"Failed to peek in pipe\n\0".as_ptr() as *const c_char);
            break;
        }
        if dwAvail != 0 {
            if ReadFile(
                (*client).hChildStdoutRd,
                chBuf.as_mut_ptr() as *mut c_void,
                BUFSIZE as DWORD,
                &mut dwRead,
                core::ptr::null_mut(),
            ) == 0
                || dwRead == 0
            {
                telnetd_printf(b"Failed to read from pipe\n\0".as_ptr() as *const c_char);
                break;
            }

            from = 0;
            to = 0;
            while from < dwRead as usize {
                txBuf[to] = chBuf[from];
                if txBuf[to] == b'\n' {
                    txBuf[to] = b'\r';
                    to += 1;
                    txBuf[to] = b'\n';
                }
                from += 1;
                to += 1;
            }
            if send((*client).socket as SOCKET, txBuf.as_ptr() as *const c_char, to as c_int, 0) < 0 {
                telnetd_printf(b"error writing to socket\n\0".as_ptr() as *const c_char);
                break;
            }
        }
        Sleep(100);
    }

    if (*client).bTerminate == 0 {
        TerminateShell(client);
    }

    telnetd_printf(b"ReadFromPipeThread terminated\n\0".as_ptr() as *const c_char);

    (*client).bReadFromPipe = FALSE as BOOLEAN;
    0
}
unsafe extern "C" fn TerminateShell(client: *mut client_t) {
    const STILL_ACTIVE: DWORD = 259;
    const CTRL_BREAK_EVENT: DWORD = 1;
    const CTRL_C_EVENT: DWORD = 0;
    const WAIT_FAILED: DWORD = 0xFFFFFFFF;

    let mut exitCode: DWORD = 0;
    let mut dwWritten: DWORD = 0;
    let stop = b"\x03\r\nexit\r\n\0";

    GetExitCodeProcess((*client).hProcess, &raw mut exitCode);

    if exitCode == STILL_ACTIVE {
        let mut hEvent: HANDLE = core::ptr::null_mut();
        let dwWaitResult: DWORD;

        telnetd_printf(
            b"user shell still active, send Ctrl-Break to group-id %lu\n\0".as_ptr() as *const c_char,
            (*client).dwProcessId,
        );

        // ANSI module (no UNICODE), so CreateEvent resolves to CreateEventA.
        hEvent = CreateEventA(core::ptr::null_mut(), TRUE as BOOL, FALSE as BOOL, core::ptr::null());

        if hEvent.is_null() {
            printf(b"CreateEvent error\n\0".as_ptr() as *const c_char);
        }

        if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, (*client).dwProcessId) == 0 {
            telnetd_printf(b"Failed to send Ctrl_break\n\0".as_ptr() as *const c_char);
        }

        if GenerateConsoleCtrlEvent(CTRL_C_EVENT, (*client).dwProcessId) == 0 {
            telnetd_printf(b"Failed to send Ctrl_C\n\0".as_ptr() as *const c_char);
        }

        if WriteFile(
            (*client).hChildStdinWr,
            stop.as_ptr() as *const c_void,
            stop.len() as DWORD,
            &raw mut dwWritten,
            core::ptr::null_mut(),
        ) == 0
        {
            telnetd_printf(b"Error writing to pipe\n\0".as_ptr() as *const c_char);
        }

        dwWaitResult = WaitForSingleObject(hEvent, 500);

        if dwWaitResult == WAIT_FAILED {
            telnetd_printf(b"WaitForSingleObject failed\n\0".as_ptr() as *const c_char);
        }

        GetExitCodeProcess((*client).hProcess, &raw mut exitCode);
        if exitCode == STILL_ACTIVE {
            telnetd_printf(b"user shell still active, attempt to terminate it now...\n\0".as_ptr() as *const c_char);

            if !hEvent.is_null() {
                if CloseHandle(hEvent) == 0 {
                    telnetd_printf(b"CloseHandle\0".as_ptr() as *const c_char);
                }
            }
            TerminateProcess((*client).hProcess, 0);
        }
        TerminateProcess((*client).hProcess, 0);
    }
    TerminateProcess((*client).hProcess, 0);
}
unsafe extern "C" fn ErrorExit(lpszMessage: *const c_char) {
    fprintf(stderr(), b"%s\n\0".as_ptr() as *const c_char, lpszMessage);
    if bSocketInterfaceInitialised != 0 {
        telnetd_printf(b"WSAGetLastError=%d\n\0".as_ptr() as *const c_char, WSAGetLastError());
        WSACleanup();
    }
    ExitProcess(0);
}




// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Abstract: a simple telnet 'daemon' for Windows hosts.
//  *
//  * Compiled & run successfully using MSVC 5.0 under Windows95 (requires 
//  * Winsock2 update) and Windows98 and MSVC 6.0 under WindowsNT4
//  *
//  * Compiler options : no special options needed
//  * Linker options   : add wsock32.lib or ws2_32.lib
//  *
//  * Written by fred.van.lieshout 'at' zonnet.nl
//  * Use freely, no copyrights.
//  * Use Linux.
//  *
//  * Parts Copyright Steven Edwards
//  * Public Domain
//  *
//  * TODO: 
//  * - access control
//  * - will/won't handshake
//  * - Unify Debugging output and return StatusCodes
//  */
//
// #include "telnetd.h"
//
// #define telnetd_printf printf
//
// /* Local data */
//
// static BOOLEAN bShutdown = 0;
// static BOOLEAN bSocketInterfaceInitialised = 0;
// static int sock;
//
// /* In the future, some options might be passed here to handle
//  * authentication options in the registry or command line
//  * options passed to the service
//  *
//  * Once you are ready to turn on the service
//  * rename this function
//  * int kickoff_telnetd(void)
//  */
// int kickoff_telnetd(void)
// {
//   printf("Attempting to start Simple TelnetD\n");
//
// //  DetectPlatform();
//   SetConsoleCtrlHandler(Cleanup, 1);
//
//   if (!StartSocketInterface())
//     ErrorExit("Unable to start socket interface\n");
//
//   CreateSocket();
//
//   while(!bShutdown) {
//     WaitForConnect();
//   }
//
//   WSACleanup();
//   return 0;
// }
//
// /* Cleanup */
// static BOOL WINAPI Cleanup(DWORD dwControlType)
// {
//   if (bSocketInterfaceInitialised) {
//     telnetd_printf("Cleanup...\n");
//     WSACleanup();
//   }
//   return 0;
// }
//
// /* StartSocketInterface */
// static BOOLEAN StartSocketInterface(void)
// {
//   WORD    wVersionRequested;
//   WSADATA wsaData;
//   int     err; 
//
//   wVersionRequested = MAKEWORD( 2, 0 ); 
//   err = WSAStartup(wVersionRequested, &wsaData);
//   if (err != 0) {
//     telnetd_printf("requested winsock version not supported\n");
//     return 0;
//   } 
//
//   bSocketInterfaceInitialised = 1; /* for ErrorExit function */
//
//   if ( wsaData.wVersion != wVersionRequested)
//     ErrorExit("requested winsock version not supported\n");
//
//   telnetd_printf("TelnetD, using %s\n", wsaData.szDescription);
//   return 1;
// }
//
// /* CreateSocket */
// static void CreateSocket(void)
// {
//    struct sockaddr_in sa;
//
//    sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
//    if (sock < 0)
//      ErrorExit("Cannot create socket");
//
//    memset(&sa, 0, sizeof(sa));
//    sa.sin_family = AF_INET;
//    sa.sin_addr.s_addr = INADDR_ANY;
//    sa.sin_port = htons(TELNET_PORT);
//
//    if (bind(sock, (struct sockaddr*) &sa, sizeof(sa)) != 0)
//       ErrorExit("Cannot bind address to socket");
// }
//
// /* WaitForConnect */
// static void WaitForConnect(void)
// {
//   struct sockaddr_in sa;
//   int new_sock;
//
//   if (listen(sock, 1) < 0)
//      ErrorExit("Cannot listen on socket");
//
//   if ((new_sock = accept(sock, (struct sockaddr*) &sa, NULL)) < 0) {
//     fprintf(stderr, "Failed to accept incoming call\n");
//   } else {
//     telnetd_printf("user connected on socket %d, port %d, address %lx\n", new_sock,
//                                        htons(sa.sin_port), sa.sin_addr.s_addr);
//     UserLogin(new_sock);
//   }
// }
//
// /* Function: UserLogin */
// static void UserLogin(int client_socket)
// {
//   HANDLE     threadHandle;
//   client_t  *client = malloc(sizeof(client_t));
//
//   if (client == NULL)
//     ErrorExit("failed to allocate memory for client");
//
//   client->socket = client_socket;
//   threadHandle = CreateThread(NULL, 0, UserLoginThread, client, 0, NULL);
//   if (threadHandle == NULL)
//     free(client);
//   else
//     CloseHandle(threadHandle);
// }
//
// /* Function: UserLoginThread */
// static DWORD WINAPI UserLoginThread(LPVOID data)
// {
//   client_t  *client = (client_t *) data;
//   char       welcome[256];
//   char       hostname[64] = "Unknown";
//   char      *pwdPrompt = "\r\npass:";
//   //char      *logonPrompt = "\r\nLogin OK, please wait...";
//   //char      *byebye = "\r\nWrong! bye bye...\r\n";
//   char       userID[USERID_SIZE];
//   char       password[USERID_SIZE];
//   int        received;
//   char      *terminator;
//
//   if (DoTelnetHandshake(client->socket)) {
//     closesocket(client->socket);
//     free(client);
//     return 0;
//   }
//
//   gethostname(hostname, sizeof(hostname));
//   sprintf(welcome, "\r\nWelcome to %s, please identify yourself\r\n\r\nuser:", hostname);
//
//   if (send(client->socket, welcome, strlen(welcome), 0) < 0) {   
//     closesocket(client->socket);
//     free(client);
//     return 0;
//   }
//   received = ReceiveLine(client->socket, userID, sizeof(userID), Echo );
//   if (received < 0) {
//     closesocket(client->socket);
//     free(client);
//     return 0;
//   } else if (received) {
//     if ((terminator = strchr(userID, CR)) != NULL) {
//       *terminator = '\0';
//     }
//   }
//
//   if (send(client->socket, pwdPrompt, strlen(pwdPrompt), 0) < 0) {   
//     closesocket(client->socket);
//     free(client);
//     return 0;
//   }
//   received = ReceiveLine(client->socket, password, sizeof(password), Password );
//
//
//   /* TODO: do authentication here */
//
//   
//   telnetd_printf("User '%p' logged on\n", userID);
//   RunShell(client);
//   return 0;
// }
//
// /* Function: DoTelnetHandshake */
// static int DoTelnetHandshake(int sock)
// {
//   int retval;
//   int received;
//   fd_set set;
//   struct timeval timeout = { HANDSHAKE_TIMEOUT, 0 };
//
//   char will_echo[]=
//       IAC DONT ECHO
//       IAC WILL ECHO
//       IAC WILL NAWS
//       IAC WILL SUPPRESS_GO_AHEAD
//       IAC DO SUPPRESS_GO_AHEAD
//       IAC DONT NEWENVIRON
//       IAC WONT NEWENVIRON
//       IAC WONT LINEMODE
//       IAC DO NAWS
//       IAC SB TERMINAL_TYPE "\x01" IAC SE
//       ;
//
//   unsigned char client_reply[256];
//
//   if (send(sock, will_echo, sizeof(will_echo), 0) < 0) {   
//     return -1;
//   }
//
//   /* Now wait for client response (and ignore it) */
//   FD_ZERO(&set);
//   FD_SET(sock, &set);
//
//   do {
//     retval = select(0, &set, NULL, NULL, &timeout);
//     /* check for error */
//     if (retval < 0) {
//       return -1;
//       /* check for timeout */
//     } else if (retval == 0) {
//       return 0;
//     }
//     /* no error and no timeout, we have data in our sock */
//     received = recv(sock, (char *) client_reply, sizeof(client_reply), 0);
//     if (received <= 0) {
//      return -1;
//     }
//   } while (retval);
//
//   return 0;
// }
//
// /*
// ** Function: ReceiveLine
// **
// ** Abstract: receive until timeout or CR
// ** In      : sock, len
// ** Out     : buffer
// ** Result  : int
// ** Pre     : 'sock' must be valid socket
// ** Post    : (result = the number of bytes read into 'buffer')
// **           OR (result = -1 and error)
// */
// static int ReceiveLine(int sock, char *buffer, int len, EchoMode echo)
// {
//   int            i = 0;
//   int            retval;
//   fd_set         set;
//   struct timeval timeout = { 0, 100000 };
//   char           del[3] = { BS, ' ', BS };
//   char           asterisk[1] = { '*' };
//
//   FD_ZERO(&set);
//   FD_SET(sock, &set);
//
//   memset(buffer, '\0', len);
//
//   do {
//     /* When we're in echo mode, we do not need a timeout */
//     retval = select(0, &set, NULL, NULL, (echo ? NULL : &timeout) );
//     /* check for error */
//     if (retval < 0) {
//       return -1;
//       /* check for timeout */
//     } else if (retval == 0) {
//       /* return number of characters received so far */
//       return i;
//     }
//     /* no error and no timeout, we have data in our sock */
//     if (recv(sock, &buffer[i], 1, 0) <= 0) {
//       return -1;
//     }
//     if ((buffer[i] == '\0') || (buffer[i] == LF)) {
//       /* ignore null characters and linefeeds from DOS telnet clients */
//       buffer[i] = '\0';
//     } else if ((buffer[i] == DEL) || (buffer[i] == BS)) {
//       /* handle delete and backspace */
//       buffer[i] = '\0';
//       if (echo) {
// 	      if (i > 0) {
//           i--;
//           buffer[i] = '\0';
//           if (send(sock, del, sizeof(del), 0) < 0) {
//             return -1;
//           }
//         }
//       } else {
//         buffer[i] = BS;  /* Let shell process handle it */
// 	      i++;
//       }
//     } else {
//       /* echo typed characters */
//       if (echo == Echo && send(sock, &buffer[i], 1, 0) < 0) {
//         return -1;
//       } else if (echo == Password && send(sock, asterisk, sizeof(asterisk), 0) < 0) {
//         return -1;
//       }
//       if (buffer[i] == CR) {
//         i++;
//         buffer[i] = LF; /* append LF for DOS command processor */
//         i++;
//         return i;
//       }
//
//       i++;
//     }    
//   } while (i < len);
//
//   return i;
// }
//
// /*
// ** Function: RunShell
// */
// static void RunShell(client_t *client) 
// { 
//    HANDLE                threadHandle;
//    HANDLE                hChildStdinRd;
//    HANDLE                hChildStdinWr;
//    HANDLE                hChildStdoutRd;
//    HANDLE                hChildStdoutWr;
//    STARTUPINFO           si;
//    PROCESS_INFORMATION   piProcInfo;
//    SECURITY_ATTRIBUTES   saAttr;
//    char cmd_path[MAX_PATH];
//
//    if (!GetEnvironmentVariableA("COMSPEC", cmd_path, ARRAYSIZE(cmd_path)))
//    {
//       if (GetSystemDirectoryA(cmd_path, ARRAYSIZE(cmd_path)))
//       {
//          StringCchCatA(cmd_path, ARRAYSIZE(cmd_path), "\\cmd.exe");
//       }
//       else
//       {
//          ErrorExit("GetSystemDirectoryA failed\n");
//       }
//    }
//
//    saAttr.nLength = sizeof(SECURITY_ATTRIBUTES); 
//    saAttr.bInheritHandle = TRUE; 
//    saAttr.lpSecurityDescriptor = NULL; 
//    
//    // Create a pipe for the child process's STDOUT.  
//    if (! CreatePipe(&hChildStdoutRd, &hChildStdoutWr, &saAttr, 0)) 
//       ErrorExit("Stdout pipe creation failed\n");  
//
//    if (! CreatePipe(&hChildStdinRd, &hChildStdinWr, &saAttr, 0)) 
//       ErrorExit("Stdin pipe creation failed\n");  
//
//
//    client->bTerminate = FALSE;
//    client->bWriteToPipe = TRUE;
//    client->bReadFromPipe = TRUE;
//    client->hChildStdinWr = hChildStdinWr;   
//    client->hChildStdoutRd = hChildStdoutRd;
//
//
//    // Create the child process (the shell)
//    telnetd_printf("Creating child process...\n");
//
//    ZeroMemory( &si, sizeof(STARTUPINFO) );
//    si.cb = sizeof(STARTUPINFO);  
//
//    si.dwFlags = STARTF_USESTDHANDLES;
//    si.hStdInput = hChildStdinRd;
//    si.hStdOutput = hChildStdoutWr;
//    si.hStdError = hChildStdoutWr;
//
//    //si.dwFlags |= STARTF_USESHOWWINDOW;
//    //si.wShowWindow = SW_SHOW;
//
//    if (!CreateProcess(cmd_path,                  // executable module
//                       NULL,                      // command line 
//                       NULL,                      // process security attributes 
//                       NULL,                      // primary thread security attributes 
//                       TRUE,                      // handles are inherited 
//                       DETACHED_PROCESS +         // creation flags 
//                       CREATE_NEW_PROCESS_GROUP,
//                       NULL,                      // use parent's environment 
//                       NULL,                      // use parent's current directory 
//                       &si,                       // startup info
//                       &piProcInfo)) {
//      ErrorExit("Create process failed");
//    }
//
//    client->hProcess = piProcInfo.hProcess;
//    client->dwProcessId = piProcInfo.dwProcessId;
//
//    telnetd_printf("New child created (groupid=%lu)\n", client->dwProcessId);
//
//    // No longer need these in the parent...
//    if (!CloseHandle(hChildStdoutWr)) 
//      ErrorExit("Closing handle failed");  
//
//    if (!CloseHandle(hChildStdinRd)) 
//      ErrorExit("Closing handle failed");  
//
//    threadHandle = CreateThread(NULL, 0, WriteToPipeThread, client, 0, NULL);
//    if (threadHandle != NULL)
//      CloseHandle(threadHandle);
//
//    threadHandle = CreateThread(NULL, 0, ReadFromPipeThread, client, 0, NULL);
//    if (threadHandle != NULL)
//      CloseHandle(threadHandle);
//
//    threadHandle = CreateThread(NULL, 0, MonitorChildThread, client, 0, NULL);
//    if (threadHandle != NULL)
//      CloseHandle(threadHandle);
// } 
//
// /*
//  * Function: MonitorChildThread
//  *
//  * Abstract: Monitor the child (shell) process
//  */
// static DWORD WINAPI MonitorChildThread(LPVOID data)
// {
//   DWORD exitCode;
//   client_t *client = (client_t *) data;
//
//   telnetd_printf("Monitor thread running...\n");
//
//   WaitForSingleObject(client->hProcess, INFINITE);
//
//   GetExitCodeProcess(client->hProcess, &exitCode);
//   telnetd_printf("Child process terminated with code %lx\n", exitCode);
//
//   /* signal the other threads to give up */
//   client->bTerminate = TRUE;
//
//   Sleep(500);
//
//   CloseHandle(client->hChildStdoutRd);
//   CloseHandle(client->hChildStdinWr);       
//   CloseHandle(client->hProcess);
//
//   closesocket(client->socket);
//
//   telnetd_printf("Waiting for all threads to give up..\n");
//
//   while (client->bWriteToPipe || client->bReadFromPipe) {
//     telnetd_printf(".");
//     fflush(stdout);
//     Sleep(1000);
//   }
//
//   telnetd_printf("Cleanup for user '%s'\n", client->userID);
//   free(client);
//   return 0;
// }
//
// /*
//  * Function: WriteToPipeThread
//  * 
//  * Abstract: read data from the telnet client socket
//  *           and pass it on to the shell process.
//  */
// static DWORD WINAPI WriteToPipeThread(LPVOID data)
// {
//   int       iRead;
//   DWORD     dwWritten;
//   CHAR      chBuf[BUFSIZE];
//   client_t *client = (client_t *) data;
//
//   while (!client->bTerminate) {
//     iRead = ReceiveLine(client->socket, chBuf, BUFSIZE, FALSE);
//     if (iRead < 0) {
//       telnetd_printf("Client disconnect\n");
//       break;
//     } else if (iRead > 0) {
//       if (strchr(chBuf, CTRLC)) {
//         GenerateConsoleCtrlEvent(CTRL_C_EVENT, client->dwProcessId);
//       }
//       if (send(client->socket, chBuf, iRead, 0) < 0) {
// 		 telnetd_printf("error writing to socket\n");
//          break;    
// 	  }
//       if (! WriteFile(client->hChildStdinWr, chBuf, (DWORD) iRead, &dwWritten, NULL)) {
//         telnetd_printf("Error writing to pipe\n");
//         break;
//       }
//     }
//   }
//
//   if (!client->bTerminate)
//     TerminateShell(client);
//
//   telnetd_printf("WriteToPipeThread terminated\n");
//
//   client->bWriteToPipe = FALSE;
//   return 0;
// }
//
// /*
//  * Function: ReadFromPipeThread
//  *
//  * Abstract: Read data from the shell's stdout handle and
//  *           pass it on to the telnet client socket.
//  */
// static DWORD WINAPI ReadFromPipeThread(LPVOID data) 
// {    
//   DWORD dwRead;
//   DWORD dwAvail;
//   CHAR chBuf[BUFSIZE];
//   CHAR txBuf[BUFSIZE*2];
//   DWORD from,to;
//   //char warning[] = "warning: rl_prep_terminal: cannot get terminal settings";
//
//   client_t *client = (client_t *) data;
//
//   while (!client->bTerminate && client->bWriteToPipe) {
//     // Since we do not want to block, first peek...
//     if (PeekNamedPipe(client->hChildStdoutRd, NULL, 0, NULL, &dwAvail, NULL) == 0) {
//       telnetd_printf("Failed to peek in pipe\n");
//       break;
//     }
//     if (dwAvail) {
//       if( ! ReadFile( client->hChildStdoutRd, chBuf, BUFSIZE, &dwRead, NULL) ||
//            dwRead == 0) {
//         telnetd_printf("Failed to read from pipe\n");
//         break;
//       }
// 	  for (from=0, to=0; from<dwRead; from++, to++) {
//         txBuf[to] = chBuf[from];
// 		if (txBuf[to] == '\n') {
// 			txBuf[to] = '\r';
// 			to++;
// 			txBuf[to] = '\n';
// 		}
// 	  }
//       if (send(client->socket, txBuf, to, 0) < 0) {
// 		 telnetd_printf("error writing to socket\n");
//          break;    
// 	  }
// 	}
//     Sleep(100); /* Hmmm, oh well... what the heck! */
//   }
//
//   if (!client->bTerminate)
//     TerminateShell(client);
//
//   telnetd_printf("ReadFromPipeThread terminated\n");
//
//   client->bReadFromPipe = FALSE;
//   return 0;
// }
//
// /* TerminateShell */ 
// static void TerminateShell(client_t *client)
// {
//     DWORD exitCode;
//     DWORD dwWritten;
//     char stop[] = "\003\r\nexit\r\n"; /* Ctrl-C + exit */
//
//     GetExitCodeProcess(client->hProcess, &exitCode);
//
//     if (exitCode == STILL_ACTIVE)
//     {
//         HANDLE hEvent = NULL;
//         DWORD dwWaitResult;
//
//         telnetd_printf("user shell still active, send Ctrl-Break to group-id %lu\n", client->dwProcessId );
//
//         hEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
//
//         if (hEvent == NULL)
//             printf("CreateEvent error\n");
//
//         if (!GenerateConsoleCtrlEvent( CTRL_BREAK_EVENT, client->dwProcessId ))
//             telnetd_printf("Failed to send Ctrl_break\n");
//
//         if (!GenerateConsoleCtrlEvent( CTRL_C_EVENT, client->dwProcessId ))
//             telnetd_printf("Failed to send Ctrl_C\n");
//
//         if (!WriteFile(client->hChildStdinWr, stop, sizeof(stop), &dwWritten, NULL))
//             telnetd_printf("Error writing to pipe\n");
//
//         /* wait for our handler to be called */
//         dwWaitResult=WaitForSingleObject(hEvent, 500);
//
//         if (WAIT_FAILED==dwWaitResult)
//             telnetd_printf("WaitForSingleObject failed\n");
//
//         GetExitCodeProcess(client->hProcess, &exitCode);
//         if (exitCode == STILL_ACTIVE) 
//         {
//             telnetd_printf("user shell still active, attempt to terminate it now...\n");
//         
//             if (hEvent != NULL) 
//             {
//                 if (!CloseHandle(hEvent)) 
//                    telnetd_printf("CloseHandle");
//             }
//             TerminateProcess(client->hProcess, 0);
//         }
//         TerminateProcess(client->hProcess, 0);
//     }
//     TerminateProcess(client->hProcess, 0);
// }
//
// /* ErrorExit */
// static VOID ErrorExit (LPTSTR lpszMessage) 
// { 
//    fprintf(stderr, "%s\n", lpszMessage);
//    if (bSocketInterfaceInitialised) {
//      telnetd_printf("WSAGetLastError=%d\n", WSAGetLastError());
//      WSACleanup();
//    }
//    ExitProcess(0); 
// } 
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
