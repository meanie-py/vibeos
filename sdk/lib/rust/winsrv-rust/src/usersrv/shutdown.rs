use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;

extern "system" {
    pub fn NtUserSetInformationThread( ThreadHandle: HANDLE, ThreadInformationClass: u32, ThreadInformation: *const core::ffi::c_void, ThreadInformationLength: ULONG, ) -> NTSTATUS;
    pub fn NtUserCallNoParam(Routine: u32) -> ULONG_PTR;
}

// win32ss/include/ntuser.h:3606 = 0xffff0001; routine 0 is NOPARAM_ROUTINE_CREATEMENU.
use winsrv_sys::winsrv::NOPARAM_ROUTINE_ISCONSOLEMODE;
// From the headers (ntuser.h USERTHREADINFOCLASS, csrsrv.h CSR_SHUTDOWN_FLAGS), not retyped.
// The hand copies here were 0/1/2/3 and 0/1, where the headers say 7/9/5/6 and 4/8: win32k
// answered every SrvExitWindowsEx with STATUS_NOT_IMPLEMENTED (an unknown info class), so
// ExitWindowsEx -- GUI setup's reboot, every shutdown and logoff -- failed with error 1.
const UserThreadUseActiveDesktop: u32 =
    winsrv_sys::winsrv::_USERTHREADINFOCLASS_UserThreadUseActiveDesktop as u32;
const UserThreadRestoreDesktop: u32 =
    winsrv_sys::winsrv::_USERTHREADINFOCLASS_UserThreadRestoreDesktop as u32;
const UserThreadInitiateShutdown: u32 =
    winsrv_sys::winsrv::_USERTHREADINFOCLASS_UserThreadInitiateShutdown as u32;
const UserThreadEndShutdown: u32 =
    winsrv_sys::winsrv::_USERTHREADINFOCLASS_UserThreadEndShutdown as u32;

const CsrShutdownSystem: u32 = 4; // csrsrv.h CSR_SHUTDOWN_FLAGS
const CsrShutdownOther: u32 = 8;
const SHUTDOWN_SYSTEMCONTEXT: u32 = CsrShutdownSystem;
const SHUTDOWN_OTHERCONTEXT: u32 = CsrShutdownOther;

#[repr(C)]
struct NOTIFY_CONTEXT {
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    StartTime: DWORD,
    QueryResult: DWORD,
    Dlg: HWND,
    EndNowResult: DWORD,
    ShowUI: BOOL,
    UIThread: HANDLE,
    WndClient: HWND,
    ShutdownSettings: *mut SHUTDOWN_SETTINGS,
}

const QUERY_RESULT_ABORT: DWORD = 0;
const QUERY_RESULT_CONTINUE: DWORD = 1;
const QUERY_RESULT_TIMEOUT: DWORD = 2;
const QUERY_RESULT_ERROR: DWORD = 3;
const QUERY_RESULT_FORCE: DWORD = 4;

type INITCOMMONCONTROLS_PROC = Option<unsafe extern "system" fn()>;

#[repr(C)]
struct MESSAGE_CONTEXT {
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    Timeout: DWORD,
}

static mut hComCtl32Lib: HMODULE = core::ptr::null_mut();

unsafe fn CallInitCommonControls() {
    static mut Initialized: BOOL = 0;

    if Initialized != 0 {
        return;
    }

    static COMCTL32_DLL: [u16; 13] = [
        b'C' as u16, b'O' as u16, b'M' as u16, b'C' as u16,
        b'T' as u16, b'L' as u16, b'3' as u16, b'2' as u16,
        b'.' as u16, b'D' as u16, b'L' as u16, b'L' as u16, 0,
    ];

    hComCtl32Lib = LoadLibraryW(COMCTL32_DLL.as_ptr());
    if hComCtl32Lib.is_null() {
        return;
    }

    let InitProc: INITCOMMONCONTROLS_PROC = core::mem::transmute(
        GetProcAddress(hComCtl32Lib, b"InitCommonControls\0".as_ptr() as *const i8)
    );
    if InitProc.is_none() {
        return;
    }

    (InitProc.unwrap())();

    Initialized = 1;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UpdateProgressBar(ProgressBar: HWND, NotifyContext: *mut NOTIFY_CONTEXT) {
    UpdateProgressBarImpl(ProgressBar, NotifyContext)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UpdateProgressBar(ProgressBar: HWND, NotifyContext: *mut NOTIFY_CONTEXT) {
    UpdateProgressBarImpl(ProgressBar, NotifyContext)
}

unsafe fn UpdateProgressBarImpl(ProgressBar: HWND, NotifyContext: *mut NOTIFY_CONTEXT) {
    let mut Passed = GetTickCount().wrapping_sub((*NotifyContext).StartTime);
    Passed = Passed.wrapping_sub((*(*NotifyContext).ShutdownSettings).HungAppTimeout);
    if (*(*NotifyContext).ShutdownSettings).WaitToKillAppTimeout < Passed {
        Passed = (*(*NotifyContext).ShutdownSettings).WaitToKillAppTimeout;
    }
    SendMessageW(ProgressBar, PBM_SETPOS, (Passed / 2) as WPARAM, 0);
}

unsafe extern "system" fn EndNowDlgProc(
    Dlg: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let Result: INT_PTR;
    let NotifyContext: *mut NOTIFY_CONTEXT;
    let ProgressBar: HWND;
    let TitleLength: DWORD;
    let Len: i32;
    let Title: LPWSTR;

    match Msg {
        WM_INITDIALOG => {
            NotifyContext = lParam as *mut NOTIFY_CONTEXT;
            (*NotifyContext).EndNowResult = QUERY_RESULT_ABORT;
            SetWindowLongPtrW(Dlg, DWLP_USER, lParam as LONG_PTR);
            TitleLength = SendMessageW((*NotifyContext).WndClient, WM_GETTEXTLENGTH, 0, 0) as DWORD
                + GetWindowTextLengthW(Dlg) as DWORD;
            Title = HeapAlloc(
                UserServerHeap,
                0,
                ((TitleLength + 1) * core::mem::size_of::<u16>() as DWORD) as SIZE_T,
            ) as LPWSTR;
            if !Title.is_null() {
                Len = GetWindowTextW(Dlg, Title, (TitleLength + 1) as i32);
                SendMessageW(
                    (*NotifyContext).WndClient,
                    WM_GETTEXT,
                    (TitleLength + 1 - Len as DWORD) as WPARAM,
                    (Title as usize + Len as usize * core::mem::size_of::<u16>()) as LPARAM,
                );
                SetWindowTextW(Dlg, Title);
                HeapFree(UserServerHeap, 0, Title as LPVOID);
            }
            ProgressBar = GetDlgItem(Dlg, IDC_PROGRESS as i32);
            SendMessageW(
                ProgressBar,
                PBM_SETRANGE32,
                0,
                ((*(*NotifyContext).ShutdownSettings).WaitToKillAppTimeout / 2) as LPARAM,
            );
            UpdateProgressBar(ProgressBar, NotifyContext);
            SetTimer(Dlg, 0, 200, None);
            Result = 0;
        }

        WM_TIMER => {
            NotifyContext = GetWindowLongPtrW(Dlg, DWLP_USER) as *mut NOTIFY_CONTEXT;
            ProgressBar = GetDlgItem(Dlg, IDC_PROGRESS as i32);
            UpdateProgressBar(ProgressBar, NotifyContext);
            Result = 1;
        }

        WM_COMMAND => {
            if ((wParam >> 16) as UINT) == BN_CLICKED && (wParam & 0xFFFF) as UINT == IDC_END_NOW {
                NotifyContext = GetWindowLongPtrW(Dlg, DWLP_USER) as *mut NOTIFY_CONTEXT;
                (*NotifyContext).EndNowResult = QUERY_RESULT_FORCE;
                SendMessageW(Dlg, WM_CLOSE, 0, 0);
                Result = 1;
            } else {
                Result = 0;
            }
        }

        WM_CLOSE => {
            DestroyWindow(Dlg);
            Result = 1;
        }

        WM_DESTROY => {
            NotifyContext = GetWindowLongPtrW(Dlg, DWLP_USER) as *mut NOTIFY_CONTEXT;
            (*NotifyContext).Dlg = core::ptr::null_mut();
            KillTimer(Dlg, 0);
            PostQuitMessage((*NotifyContext).EndNowResult as i32);
            Result = 1;
        }

        _ => {
            Result = 0;
        }
    }

    Result
}

unsafe extern "system" fn EndNowThreadProc(Parameter: LPVOID) -> DWORD {
    let NotifyContext = Parameter as *mut NOTIFY_CONTEXT;
    let mut Msg: MSG = core::mem::zeroed();

    NtUserSetInformationThread(
        NtCurrentThread(),
        UserThreadUseActiveDesktop,
        core::ptr::null(),
        0,
    );

    CallInitCommonControls();
    (*NotifyContext).Dlg = CreateDialogParamW(
        UserServerDllInstance,
        IDD_END_NOW as usize as LPWSTR,
        core::ptr::null_mut(),
        // extern "system" → extern "stdcall" — same ABI on i386,
        // single ABI on amd64/arm64; transmute is sound.
        Some(core::mem::transmute(EndNowDlgProc as *const ())),
        NotifyContext as LPARAM,
    );
    if (*NotifyContext).Dlg.is_null() {
        return 0;
    }

    ShowWindow((*NotifyContext).Dlg, SW_SHOWNORMAL as i32);

    while GetMessageW(&mut Msg, core::ptr::null_mut(), 0, 0) != 0 {
        if IsDialogMessageW((*NotifyContext).Dlg, &mut Msg) == 0 {
            TranslateMessage(&Msg);
            DispatchMessageW(&Msg);
        }
    }

    Msg.wParam as DWORD
}

unsafe extern "system" fn SendClientShutdown(Parameter: LPVOID) -> DWORD {
    let Context = Parameter as *mut MESSAGE_CONTEXT;
    let mut Result: DWORD_PTR = 0;

    if ((*Context).wParam & (MCS_QUERYENDSESSION | MCS_ENDSESSION) as WPARAM) == 0 {
        SendNotifyMessageW(
            (*Context).Wnd,
            WM_CLIENTSHUTDOWN,
            (*Context).wParam,
            (*Context).lParam,
        );
        return QUERY_RESULT_CONTINUE;
    }

    if SendMessageTimeoutW(
        (*Context).Wnd,
        WM_CLIENTSHUTDOWN,
        (*Context).wParam,
        (*Context).lParam,
        SMTO_NORMAL,
        (*Context).Timeout,
        &mut Result,
    ) != 0
    {
        let Ret: DWORD;

        if ((*Context).wParam & MCS_QUERYENDSESSION as WPARAM) != 0 {
            match Result as DWORD {
                MCSR_DONOTSHUTDOWN => {
                    Ret = QUERY_RESULT_ABORT;
                }
                MCSR_GOODFORSHUTDOWN | MCSR_SHUTDOWNFINISHED | _ => {
                    Ret = QUERY_RESULT_CONTINUE;
                }
            }
        } else {
            Ret = QUERY_RESULT_CONTINUE;
        }

        return Ret;
    }

    if GetLastError() == 0 {
        QUERY_RESULT_TIMEOUT
    } else {
        QUERY_RESULT_ERROR
    }
}

unsafe fn NotifyTopLevelWindow(Wnd: HWND, NotifyContext: *mut NOTIFY_CONTEXT) -> BOOL {
    let mut MessageContext: MESSAGE_CONTEXT = core::mem::zeroed();
    let Now: DWORD;
    let Passed: DWORD;
    let mut Timeout: DWORD;
    let MessageThread: HANDLE;
    let mut Threads: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let WaitStatus: DWORD;

    SetForegroundWindow(Wnd);

    Now = GetTickCount();
    if (*NotifyContext).StartTime == 0 {
        (*NotifyContext).StartTime = Now;
    }

    Passed = Now.wrapping_sub((*NotifyContext).StartTime);
    MessageContext.Wnd = Wnd;
    MessageContext.Msg = (*NotifyContext).Msg;
    MessageContext.wParam = (*NotifyContext).wParam;
    MessageContext.lParam = (*NotifyContext).lParam;
    MessageContext.Timeout = (*(*NotifyContext).ShutdownSettings).HungAppTimeout;
    if (*(*NotifyContext).ShutdownSettings).AutoEndTasks == 0 {
        MessageContext.Timeout += (*(*NotifyContext).ShutdownSettings).WaitToKillAppTimeout;
    }
    if Passed < MessageContext.Timeout {
        MessageContext.Timeout -= Passed;
        MessageThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(core::mem::transmute(SendClientShutdown as *const ())),
            &mut MessageContext as *mut _ as LPVOID,
            0,
            core::ptr::null_mut(),
        );
        if MessageThread.is_null() {
            (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
            return 0;
        }
        Timeout = (*(*NotifyContext).ShutdownSettings).HungAppTimeout;
        if Passed < Timeout {
            Timeout -= Passed;
            WaitStatus = WaitForSingleObjectEx(MessageThread, Timeout, 0);
        } else {
            WaitStatus = WAIT_TIMEOUT;
        }
        if WaitStatus == WAIT_TIMEOUT {
            (*NotifyContext).WndClient = Wnd;
            if (*NotifyContext).UIThread.is_null() && (*NotifyContext).ShowUI != 0 {
                (*NotifyContext).UIThread = CreateThread(
                    core::ptr::null_mut(),
                    0,
                    Some(core::mem::transmute(EndNowThreadProc as *const ())),
                    NotifyContext as LPVOID,
                    0,
                    core::ptr::null_mut(),
                );
            }
            Threads[0] = MessageThread;
            Threads[1] = (*NotifyContext).UIThread;
            let num_threads = if (*NotifyContext).UIThread.is_null() { 1 } else { 2 };
            let WaitStatus = WaitForMultipleObjectsEx(
                num_threads,
                Threads.as_ptr(),
                0,
                INFINITE,
                0,
            );
            if WaitStatus == WAIT_OBJECT_0 {
                if GetExitCodeThread(MessageThread, &mut (*NotifyContext).QueryResult) == 0 {
                    (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
                }
            } else if WaitStatus == WAIT_OBJECT_0 + 1 {
                if GetExitCodeThread((*NotifyContext).UIThread, &mut (*NotifyContext).QueryResult) == 0 {
                    (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
                }
            } else {
                (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
            }
            if WaitStatus != WAIT_OBJECT_0 {
                TerminateThread(MessageThread, QUERY_RESULT_TIMEOUT);
            }
        } else if WaitStatus == WAIT_OBJECT_0 {
            if GetExitCodeThread(MessageThread, &mut (*NotifyContext).QueryResult) == 0 {
                (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
            }
        } else {
            (*NotifyContext).QueryResult = QUERY_RESULT_ERROR;
        }
        CloseHandle(MessageThread);
    } else {
        (*NotifyContext).QueryResult = QUERY_RESULT_TIMEOUT;
    }

    ((*NotifyContext).QueryResult == QUERY_RESULT_CONTINUE) as BOOL
}

unsafe fn IsConsoleMode() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_ISCONSOLEMODE) as BOOL
}

unsafe fn ThreadShutdownNotify(
    CsrThread: *mut CSR_THREAD,
    Flags: ULONG,
    Flags2: ULONG,
    Context: *mut NOTIFY_CONTEXT,
) -> BOOL {
    let mut TopWnd: HWND = core::ptr::null_mut();

    EnumThreadWindows(
        (*CsrThread).ClientId.UniqueThread as DWORD,
        Some(FindTopLevelWnd),
        &mut TopWnd as *mut _ as LPARAM,
    );
    if !TopWnd.is_null() {
        let mut hWndOwner: HWND;

        loop {
            hWndOwner = GetWindow(TopWnd, GW_OWNER);
            if hWndOwner.is_null() {
                break;
            }
            TopWnd = hWndOwner;
        }
    } else {
        return 0;
    }

    (*Context).wParam = Flags2 as WPARAM;
    (*Context).lParam = (if (Flags & EWX_CALLER_WINLOGON_LOGOFF) != 0 {
        ENDSESSION_LOGOFF
    } else {
        0
    }) as LPARAM;

    (*Context).StartTime = 0;
    (*Context).UIThread = core::ptr::null_mut();
    (*Context).ShowUI = (IsConsoleMode() == 0 && (Flags2 & (MCS_QUERYENDSESSION | MCS_ENDSESSION)) != 0) as BOOL;
    (*Context).Dlg = core::ptr::null_mut();

    NotifyTopLevelWindow(TopWnd, Context);

    if !(*Context).UIThread.is_null() {
        if !(*Context).Dlg.is_null() {
            SendMessageW((*Context).Dlg, WM_CLOSE, 0, 0);
        } else {
            TerminateThread((*Context).UIThread, QUERY_RESULT_ERROR);
        }
        CloseHandle((*Context).UIThread);
        (*Context).UIThread = core::ptr::null_mut();
        (*Context).Dlg = core::ptr::null_mut();
    }

    1
}

unsafe fn NotifyUserProcessForShutdown(
    CsrProcess: *mut CSR_PROCESS,
    ShutdownSettingsParam: *mut SHUTDOWN_SETTINGS,
    Flags: UINT,
) -> ULONG {
    let mut QueryResult = QUERY_RESULT_CONTINUE;
    let mut Process: *mut CSR_PROCESS = core::ptr::null_mut();
    let mut Thread: *mut CSR_THREAD;
    let mut NextEntry: *mut LIST_ENTRY;
    let mut Context: NOTIFY_CONTEXT = core::mem::zeroed();
    let mut FoundWindows = 0u32 as BOOL;

    if (Flags & EWX_FORCE) != 0 {
        return CsrShutdownCsrProcess;
    }

    Context.ShutdownSettings = ShutdownSettingsParam;
    Context.QueryResult = QUERY_RESULT_CONTINUE;

    CsrLockProcessByClientId((*CsrProcess).ClientId.UniqueProcess, &mut Process);

    NextEntry = (*CsrProcess).ThreadList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*CsrProcess).ThreadList) as *mut LIST_ENTRY {
        Thread = ((NextEntry as usize) - core::mem::offset_of!(CSR_THREAD, Link)) as *mut CSR_THREAD;

        NextEntry = (*NextEntry).Flink;

        if ((*Thread).Flags & CsrThreadTerminated) != 0 {
            continue;
        }

        CsrReferenceThread(Thread);
        CsrUnlockProcess(Process);

        Context.QueryResult = QUERY_RESULT_CONTINUE;
        if ThreadShutdownNotify(Thread, Flags, MCS_QUERYENDSESSION, &mut Context) != 0 {
            FoundWindows = 1;
        }

        CsrLockProcessByClientId((*CsrProcess).ClientId.UniqueProcess, &mut Process);
        CsrDereferenceThread(Thread);

        if Context.QueryResult == QUERY_RESULT_ABORT {
            CsrUnlockProcess(Process);
            return CsrShutdownCancelled;
        }
    }

    if FoundWindows == 0 {
        CsrUnlockProcess(Process);
        return CsrShutdownNonCsrProcess;
    }

    QueryResult = Context.QueryResult;

    NextEntry = (*CsrProcess).ThreadList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*CsrProcess).ThreadList) as *mut LIST_ENTRY {
        Thread = ((NextEntry as usize) - core::mem::offset_of!(CSR_THREAD, Link)) as *mut CSR_THREAD;

        NextEntry = (*NextEntry).Flink;

        if ((*Thread).Flags & CsrThreadTerminated) != 0 {
            continue;
        }

        CsrReferenceThread(Thread);
        CsrUnlockProcess(Process);

        Context.QueryResult = QUERY_RESULT_CONTINUE;
        ThreadShutdownNotify(
            Thread,
            Flags,
            if QueryResult != QUERY_RESULT_ABORT { MCS_ENDSESSION } else { 0 },
            &mut Context,
        );

        CsrLockProcessByClientId((*CsrProcess).ClientId.UniqueProcess, &mut Process);
        CsrDereferenceThread(Thread);
    }

    CsrUnlockProcess(Process);

    if QueryResult == QUERY_RESULT_ABORT {
        CsrShutdownCancelled
    } else {
        CsrShutdownCsrProcess
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserExitReactOS(CsrThread: *mut CSR_THREAD, Flags: UINT) -> NTSTATUS {
    UserExitReactOSImpl(CsrThread, Flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserExitReactOS(CsrThread: *mut CSR_THREAD, Flags: UINT) -> NTSTATUS {
    UserExitReactOSImpl(CsrThread, Flags)
}

extern "C" {
    // ntdll, cdecl.
    fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
}

unsafe fn UserExitReactOSImpl(CsrThread: *mut CSR_THREAD, Flags: UINT) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut CallerLuid: LUID = core::mem::zeroed();

    let ProcessId = (*CsrThread).ClientId.UniqueProcess as ULONG_PTR;
    let _ThreadId = (*CsrThread).ClientId.UniqueThread as ULONG_PTR;

    let mut Flags = Flags;

    // The C's DPRINT1s on this path, restored: the request, and what win32k answered (which
    // decides whether winlogon hears LN_LOGOFF at all).
    DbgPrint(
        c"[WINSRV] SrvExitWindowsEx(ClientId: %lx.%lx, Flags: 0x%x)\n".as_ptr(),
        ProcessId as u32,
        _ThreadId as u32,
        Flags,
    );
    if (Flags & EWX_CALLER_WINLOGON) != 0 {
        if ProcessId != LogonProcessId {
            return STATUS_ACCESS_DENIED;
        }
    }

    if (Flags & (EWX_POWEROFF | EWX_REBOOT)) != 0 {
        Flags |= EWX_SHUTDOWN;
    }

    if CsrImpersonateClient(core::ptr::null_mut()) == 0 {
        return STATUS_BAD_IMPERSONATION_LEVEL;
    }

    Status = CsrGetProcessLuid(core::ptr::null_mut(), &mut CallerLuid);
    if Status < 0 {
        CsrRevertToSelf();
        return Status;
    }

    loop {
        Status = NtUserSetInformationThread(
            (*CsrThread).ThreadHandle,
            UserThreadInitiateShutdown,
            &Flags as *const _ as *const core::ffi::c_void,
            core::mem::size_of::<UINT>() as ULONG,
        );
        DbgPrint(c"[WINSRV] UserThreadInitiateShutdown -> 0x%08lx\n".as_ptr(), Status);
        match Status {
            STATUS_CANT_WAIT => {
                CsrRevertToSelf();
                return Status;
            }

            STATUS_PENDING => {
                CsrRevertToSelf();
                return Status;
            }

            STATUS_RETRY => {
                continue;
            }

            _ => {
                if Status < 0 {
                    SetLastError(RtlNtStatusToDosError(Status));
                    CsrRevertToSelf();
                    return Status;
                }
            }
        }

        break;
    }

    Status = CsrShutdownProcesses(&mut CallerLuid, Flags);
    if Status < 0 {
        // Continue anyway
    }

    NtUserSetInformationThread(
        (*CsrThread).ThreadHandle,
        UserThreadEndShutdown,
        &Status as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<NTSTATUS>() as ULONG,
    );

    CsrRevertToSelf();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn UserClientShutdown(
    CsrProcess: *mut CSR_PROCESS,
    Flags: ULONG,
    _FirstPhase: BOOLEAN,
) -> ULONG {
    let result: ULONG;

    if (Flags & EWX_SHUTDOWN) == EWX_LOGOFF
        && ((*CsrProcess).ShutdownFlags & (SHUTDOWN_OTHERCONTEXT | SHUTDOWN_SYSTEMCONTEXT)) != 0
    {
        return CsrShutdownNonCsrProcess;
    }

    if (*CsrProcess).ClientId.UniqueProcess == LogonProcessId as HANDLE {
        CsrDereferenceProcess(CsrProcess);
        return CsrShutdownCsrProcess;
    }

    result = NotifyUserProcessForShutdown(CsrProcess, &raw mut ShutdownSettings, Flags);
    if result == CsrShutdownCancelled || result == CsrShutdownNonCsrProcess {
        return result;
    }

    NtTerminateProcess((*CsrProcess).ProcessHandle, 0);
    WaitForSingleObject((*CsrProcess).ProcessHandle, ShutdownSettings.ProcessTerminateTimeout);

    CsrDereferenceProcess(CsrProcess);
    CsrShutdownCsrProcess
}

#[no_mangle]
pub unsafe extern "system" fn SrvExitWindowsEx(
    ApiMessage: *mut CSR_API_MESSAGE,
    _ReplyCode: *mut u32,
) -> NTSTATUS {
    let ExitReactOSRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.ExitReactOSRequest
    );

    let mut Status = NtUserSetInformationThread(
        NtCurrentThread(),
        UserThreadUseActiveDesktop,
        core::ptr::null(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    Status = UserExitReactOS(CsrGetClientThread(), (*ExitReactOSRequest).Flags);
    (*ExitReactOSRequest).Success = (Status >= 0) as BOOL;
    (*ExitReactOSRequest).LastError = GetLastError();

    NtUserSetInformationThread(NtCurrentThread(), UserThreadRestoreDesktop, core::ptr::null(), 0);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvEndTask(
    ApiMessage: *mut CSR_API_MESSAGE,
    _ReplyCode: *mut u32,
) -> NTSTATUS {
    let EndTaskRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.EndTaskRequest
    );

    let Status = NtUserSetInformationThread(
        NtCurrentThread(),
        UserThreadUseActiveDesktop,
        core::ptr::null(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    SendMessageW((*EndTaskRequest).WndHandle, WM_CLOSE, 0, 0);

    if IsWindow((*EndTaskRequest).WndHandle) != 0 {
        if (*EndTaskRequest).Force != 0 {
            (*EndTaskRequest).Success = DestroyWindow((*EndTaskRequest).WndHandle) as BOOL;
            (*EndTaskRequest).LastError = GetLastError();
        } else {
            (*EndTaskRequest).Success = 0;
        }
    } else {
        (*EndTaskRequest).Success = 1;
        (*EndTaskRequest).LastError = ERROR_SUCCESS;
    }

    NtUserSetInformationThread(NtCurrentThread(), UserThreadRestoreDesktop, core::ptr::null(), 0);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvRecordShutdownReason(
    _ApiMessage: *mut CSR_API_MESSAGE,
    _ReplyCode: *mut u32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}