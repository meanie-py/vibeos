use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use crate::service_scm::ScmRpcStatusToWinError;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SERVICE_THREAD_PARAMSA {
    pub lpServiceMain: LPSERVICE_MAIN_FUNCTIONA,
    pub dwArgCount: DWORD,
    pub lpArgVector: *mut LPSTR,
    pub dwServiceTag: DWORD,
}
pub type SERVICE_THREAD_PARAMSA = _SERVICE_THREAD_PARAMSA;
pub type PSERVICE_THREAD_PARAMSA = *mut _SERVICE_THREAD_PARAMSA;

#[repr(C)]
pub struct _SERVICE_THREAD_PARAMSW {
    pub lpServiceMain: LPSERVICE_MAIN_FUNCTIONW,
    pub dwArgCount: DWORD,
    pub lpArgVector: *mut LPWSTR,
    pub dwServiceTag: DWORD,
}
pub type SERVICE_THREAD_PARAMSW = _SERVICE_THREAD_PARAMSW;
pub type PSERVICE_THREAD_PARAMSW = *mut _SERVICE_THREAD_PARAMSW;

#[repr(C)]
pub union _ACTIVE_SERVICE__bindgen_ty_1 {
    pub A: LPSERVICE_MAIN_FUNCTIONA,
    pub W: LPSERVICE_MAIN_FUNCTIONW,
}

#[repr(C)]
pub struct _ACTIVE_SERVICE {
    pub hServiceStatus: SERVICE_STATUS_HANDLE,
    pub ServiceName: _UNICODE_STRING,
    pub ServiceMain: _ACTIVE_SERVICE__bindgen_ty_1,
    pub HandlerFunction: LPHANDLER_FUNCTION,
    pub HandlerFunctionEx: LPHANDLER_FUNCTION_EX,
    pub HandlerContext: LPVOID,
    pub bUnicode: BOOL,
    pub bOwnProcess: BOOL,
    pub dwServiceTag: DWORD,
}
pub type ACTIVE_SERVICE = _ACTIVE_SERVICE;
pub type PACTIVE_SERVICE = *mut _ACTIVE_SERVICE;

pub static mut dwActiveServiceCount: DWORD = 0;
pub static mut lpActiveServices: PACTIVE_SERVICE = core::ptr::null_mut();
pub static mut hStatusBinding: handle_t = core::ptr::null_mut();
pub static mut bSecurityServiceProcess: BOOL = 0;

#[no_mangle]
pub unsafe extern "system" fn RPC_SERVICE_STATUS_HANDLE_bind(hServiceStatus: RPC_SERVICE_STATUS_HANDLE) -> handle_t {
    hStatusBinding
}
#[no_mangle]
pub unsafe extern "system" fn RPC_SERVICE_STATUS_HANDLE_unbind(
    hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
    hBinding: handle_t,
) {
}

unsafe fn ScCreateStatusBinding() -> RPC_STATUS {
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS;

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        w!("ncacn_np").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        w!("\\pipe\\ntsvcs").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != RPC_S_OK {
        return status;
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hStatusBinding);
    if status != RPC_S_OK {
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != RPC_S_OK {
    }

    status
}

unsafe fn ScDestroyStatusBinding() -> RPC_STATUS {
    let status: RPC_STATUS;

    if hStatusBinding.is_null() {
        return RPC_S_OK;
    }

    status = RpcBindingFree(core::ptr::addr_of_mut!(hStatusBinding));
    if status != RPC_S_OK {
    } else {
        hStatusBinding = core::ptr::null_mut();
    }

    status
}

unsafe fn ScLookupServiceByServiceName(lpServiceName: LPCWSTR) -> PACTIVE_SERVICE {
    let mut i: u32;

    if (*lpActiveServices.offset(0)).bOwnProcess != 0 {
        return lpActiveServices.offset(0);
    }

    i = 0;
    while i < dwActiveServiceCount {
        if _wcsicmp(
            (*lpActiveServices.offset(i as isize)).ServiceName.Buffer,
            lpServiceName,
        ) == 0
        {
            return lpActiveServices.offset(i as isize);
        }
        i += 1;
    }

    core::ptr::null_mut()
}
unsafe extern "system" fn ScServiceMainStubA(Context: LPVOID) -> DWORD {
    let ThreadParams = Context as PSERVICE_THREAD_PARAMSA;

    TRACE!(b"ScServiceMainStubA(%p)\n\0".as_ptr() as LPCSTR, Context);

    let Teb: PTEB = NtCurrentTeb();
    (*Teb).SubProcessTag = UlongToPtr((*ThreadParams).dwServiceTag);

    ((*ThreadParams).lpServiceMain.unwrap())(
        (*ThreadParams).dwArgCount,
        (*ThreadParams).lpArgVector,
    );

    (*Teb).SubProcessTag = core::ptr::null_mut();

    if !(*ThreadParams).lpArgVector.is_null() {
        HeapFree(GetProcessHeap(), 0, (*ThreadParams).lpArgVector as LPVOID);
    }
    HeapFree(GetProcessHeap(), 0, ThreadParams as LPVOID);

    ERROR_SUCCESS
}
unsafe extern "system" fn ScServiceMainStubW(Context: LPVOID) -> DWORD {
    let Teb: PTEB;
    let ThreadParams: PSERVICE_THREAD_PARAMSW = Context as PSERVICE_THREAD_PARAMSW;

    Teb = NtCurrentTeb();
    (*Teb).SubProcessTag = UlongToPtr((*ThreadParams).dwServiceTag);

    if let Some(lpServiceMain) = (*ThreadParams).lpServiceMain {
        lpServiceMain((*ThreadParams).dwArgCount, (*ThreadParams).lpArgVector);
    }

    (*Teb).SubProcessTag = core::ptr::null_mut();

    if !(*ThreadParams).lpArgVector.is_null() {
        HeapFree(GetProcessHeap(), 0, (*ThreadParams).lpArgVector as LPVOID);
    }
    HeapFree(GetProcessHeap(), 0, ThreadParams as LPVOID);

    ERROR_SUCCESS
}

unsafe fn ScConnectControlPipe(hPipe: *mut HANDLE) -> DWORD {
    let mut dwBytesWritten: DWORD = 0;
    let mut dwState: DWORD;
    let mut dwServiceCurrent: DWORD = 1;
    let Status: NTSTATUS;
    let mut NtControlPipeName: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let dwProcessId: DWORD;

    if bSecurityServiceProcess == 0 {
        RtlZeroMemory(
            QueryTable.as_mut_ptr() as *mut _,
            core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>() as _,
        );

        // C: `QueryTable[0].Name = L"";` (query the key's DEFAULT value).
        // Was `b"\0"` — ONE byte cast to PWSTR, which as WCHARs is not an
        // empty string (u16[0] = 0x00nn from the neighboring rodata byte), so
        // this REQUIRED query failed, ScConnectControlPipe bailed before its
        // own [SCTRL] trace, StartServiceCtrlDispatcherW returned FALSE, and
        // every svchost exited 0 ~0.4s after spawn — costing the SCM a full
        // 31s pipe-connect timeout per hosted service on EVERY boot
        // (DcomLaunch/RPCSS dead since the translation). An empty WIDE string
        // needs two zero bytes.
        QueryTable[0].Name = w!("").as_ptr() as PWSTR;
        QueryTable[0].Flags = (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED) as _;
        QueryTable[0].EntryContext = core::ptr::addr_of_mut!(dwServiceCurrent) as *mut _;

        Status = RtlQueryRegistryValues(
            RTL_REGISTRY_CONTROL as _,
            "S\0e\0r\0v\0i\0c\0e\0C\0u\0r\0r\0e\0n\0t\0\0\0".as_ptr() as PWSTR,
            QueryTable.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            return RtlNtStatusToDosError(Status);
        }
    } else {
        dwServiceCurrent = 0;
    }

    swprintf(
        NtControlPipeName.as_mut_ptr(),
        "\\\0\\\0.\0\\\0p\0i\0p\0e\0\\\0n\0e\0t\0\\\0N\0t\0C\0o\0n\0t\0r\0o\0l\0P\0i\0p\0e\0%\0u\0\0\0".as_ptr() as PWSTR,
        dwServiceCurrent,
    );

    // Temp control-pipe-race instrumentation (DcomLaunch wedge). DbgPrint reaches
    // serial (OutputDebugString's exception path does not under DUMMY_PSEH).
    extern "C" {
        fn DbgPrint(fmt: *const u8, ...) -> u32;
    }
    DbgPrint(b"[SCTRL] pid=%lu connecting pipe=%ws\n\0".as_ptr(), GetCurrentProcessId(), NtControlPipeName.as_ptr());
    // The security service (lsass) can reach here BEFORE the SCM has created its
    // control pipe (NtControlPipe0): the SCM creates it lazily as it loads the
    // SamSs image, which can be after lsass starts. WaitNamedPipeW returns FALSE
    // *immediately* (ERROR_FILE_NOT_FOUND) for a pipe that does not yet exist
    // rather than waiting, so for the security process POLL until it appears.
    // This is what makes the SamSs handshake reliable: lsasrv no longer gates on
    // the cross-process SECURITY_SERVICES_STARTED event (whose object sharing was
    // timing-fragile in this kernel — it intermittently left the lsasrv dispatcher
    // hung on an INFINITE wait, so SamSs timed out at 30s / connect result=1053 on
    // every boot, all arches). The pipe shares reliably across processes; the
    // event did not. Non-security services never need this — the SCM creates
    // their pipe before it spawns them, so a single 30s wait is right for them.
    let mut pipe_ready: BOOL = FALSE as BOOL;
    if bSecurityServiceProcess != 0 {
        for _ in 0..600u32 {
            if WaitNamedPipeW(NtControlPipeName.as_ptr(), 1000) != 0 {
                pipe_ready = TRUE as BOOL;
                break;
            }
            Sleep(50); // ~30s cap; the SCM creates NtControlPipe0 within the boot
        }
    } else if WaitNamedPipeW(NtControlPipeName.as_ptr(), 30000) != 0 {
        pipe_ready = TRUE as BOOL;
    }
    if pipe_ready == FALSE as BOOL {
        DbgPrint(b"[SCTRL] pid=%lu WaitNamedPipe FAILED err=%lu\n\0".as_ptr(), GetCurrentProcessId(), GetLastError());
        return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
    }
    DbgPrint(b"[SCTRL] pid=%lu WaitNamedPipe OK\n\0".as_ptr(), GetCurrentProcessId());

    *hPipe = CreateFileW(
        NtControlPipeName.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if *hPipe == INVALID_HANDLE_VALUE {
        return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
    }

    dwState = PIPE_READMODE_MESSAGE;
    if SetNamedPipeHandleState(
        *hPipe,
        core::ptr::addr_of_mut!(dwState),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        CloseHandle(*hPipe);
        *hPipe = INVALID_HANDLE_VALUE;
        return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
    }

    dwProcessId = GetCurrentProcessId();
    WriteFile(
        *hPipe,
        core::ptr::addr_of!(dwProcessId) as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        core::ptr::addr_of_mut!(dwBytesWritten),
        core::ptr::null_mut(),
    );

    ERROR_SUCCESS
}

unsafe fn ScBuildUnicodeArgsVector(
    ControlPacket: PSCM_CONTROL_PACKET,
    lpArgCount: LPDWORD,
    lpArgVector: *mut *mut LPWSTR,
) -> DWORD {
    let lpVector: *mut PWSTR;
    let pszServiceName: PWSTR;
    let cbServiceName: DWORD;
    let cbArguments: DWORD;
    let mut cbTotal: DWORD;
    let mut i: DWORD;

    if ControlPacket.is_null() || lpArgCount.is_null() || lpArgVector.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *lpArgCount = 0;
    *lpArgVector = core::ptr::null_mut();

    pszServiceName = (ControlPacket as usize + (*ControlPacket).dwServiceNameOffset as usize) as PWSTR;
    cbServiceName = lstrlenW(pszServiceName) as DWORD * core::mem::size_of::<u16>() as DWORD
        + core::mem::size_of::<u16>() as DWORD;

    cbTotal = core::mem::size_of::<PWSTR>() as DWORD + cbServiceName;
    if (*ControlPacket).dwArgumentsCount > 0 {
        cbArguments = (*ControlPacket).dwSize - (*ControlPacket).dwArgumentsOffset;
    } else {
        cbArguments = 0;
    }
    cbTotal += cbArguments;

    lpVector = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbTotal as SIZE_T) as *mut PWSTR;
    if lpVector.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    if (*ControlPacket).dwArgumentsCount > 0 {
        memcpy(
            lpVector.add(1) as *mut core::ffi::c_void,
            (ControlPacket as usize + (*ControlPacket).dwArgumentsOffset as usize) as *const core::ffi::c_void,
            cbArguments as usize,
        );

        i = 0;
        while i < (*ControlPacket).dwArgumentsCount {
            *lpVector.add((i + 1) as usize) =
                (lpVector.add(1) as usize + (*lpVector.add((i + 1) as usize)) as usize) as PWSTR;
            TRACE!(
                b"Unicode lpVector[%lu] = '%ls'\n\0".as_ptr() as *const i8,
                i + 1,
                *lpVector.add((i + 1) as usize),
            );
            i += 1;
        }
    }

    *lpVector.add(0) = (lpVector.add(1) as usize + cbArguments as usize) as PWSTR;
    memcpy(
        *lpVector.add(0) as *mut core::ffi::c_void,
        pszServiceName as *const core::ffi::c_void,
        cbServiceName as usize,
    );
    TRACE!(
        b"Unicode lpVector[%lu] = '%ls'\n\0".as_ptr() as *const i8,
        0,
        *lpVector.add(0),
    );

    *lpArgCount = (*ControlPacket).dwArgumentsCount + 1;
    *lpArgVector = lpVector as *mut LPWSTR;

    ERROR_SUCCESS
}

unsafe fn ScBuildAnsiArgsVector(
    ControlPacket: PSCM_CONTROL_PACKET,
    lpArgCount: LPDWORD,
    lpArgVector: *mut *mut LPSTR,
) -> DWORD {
    let mut dwError: DWORD;
    let mut Status: NTSTATUS;
    let mut ArgCount: DWORD = 0;
    let mut i: DWORD;
    let mut lpVectorW: *mut LPWSTR = core::ptr::null_mut();
    let lpVectorA: *mut LPSTR;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    if ControlPacket.is_null() || lpArgCount.is_null() || lpArgVector.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *lpArgCount = 0;
    *lpArgVector = core::ptr::null_mut();

    dwError = ScBuildUnicodeArgsVector(
        ControlPacket,
        core::ptr::addr_of_mut!(ArgCount),
        core::ptr::addr_of_mut!(lpVectorW),
    );
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    lpVectorA = lpVectorW as *mut LPSTR;
    i = 0;
    while i < ArgCount {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(UnicodeString),
            *lpVectorW.offset(i as isize),
        );
        RtlInitEmptyAnsiString(
            core::ptr::addr_of_mut!(AnsiString),
            *lpVectorA.offset(i as isize),
            UnicodeString.MaximumLength,
        );
        Status = RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(AnsiString),
            core::ptr::addr_of_mut!(UnicodeString),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            dwError = RtlNtStatusToDosError(Status);
            HeapFree(GetProcessHeap(), 0, lpVectorW as LPVOID);
            return dwError;
        }

        *AnsiString
            .Buffer
            .offset((AnsiString.Length as usize / core::mem::size_of::<CHAR>()) as isize) = 0;

        i += 1;
    }

    *lpArgCount = ArgCount;
    *lpArgVector = lpVectorA;

    ERROR_SUCCESS
}

unsafe fn ScStartService(
    lpService: PACTIVE_SERVICE,
    ControlPacket: PSCM_CONTROL_PACKET,
) -> DWORD {
    let ThreadHandle: HANDLE;
    let mut ThreadId: DWORD = 0;
    let dwError: DWORD;
    let ThreadParamsA: PSERVICE_THREAD_PARAMSA;
    let ThreadParamsW: PSERVICE_THREAD_PARAMSW;

    if lpService.is_null() || ControlPacket.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    (*lpService).hServiceStatus = (*ControlPacket).hServiceStatus;
    (*lpService).dwServiceTag = (*ControlPacket).dwServiceTag;

    if (*lpService).bUnicode != 0 {
        ThreadParamsW = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SERVICE_THREAD_PARAMSW>() as SIZE_T,
        ) as PSERVICE_THREAD_PARAMSW;
        if ThreadParamsW.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        dwError = ScBuildUnicodeArgsVector(
            ControlPacket,
            core::ptr::addr_of_mut!((*ThreadParamsW).dwArgCount),
            core::ptr::addr_of_mut!((*ThreadParamsW).lpArgVector),
        );
        if dwError != ERROR_SUCCESS {
            HeapFree(GetProcessHeap(), 0, ThreadParamsW as LPVOID);
            return dwError;
        }
        (*ThreadParamsW).lpServiceMain = (*lpService).ServiceMain.W;
        (*ThreadParamsW).dwServiceTag = (*ControlPacket).dwServiceTag;
        ThreadHandle = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(ScServiceMainStubW),
            ThreadParamsW as LPVOID,
            0,
            core::ptr::addr_of_mut!(ThreadId),
        );
        if ThreadHandle.is_null() {
            if !(*ThreadParamsW).lpArgVector.is_null() {
                HeapFree(GetProcessHeap(), 0, (*ThreadParamsW).lpArgVector as LPVOID);
            }
            HeapFree(GetProcessHeap(), 0, ThreadParamsW as LPVOID);

            return ERROR_SERVICE_NO_THREAD;
        }

        CloseHandle(ThreadHandle);
    } else {
        ThreadParamsA = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<SERVICE_THREAD_PARAMSA>() as SIZE_T,
        ) as PSERVICE_THREAD_PARAMSA;
        if ThreadParamsA.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        dwError = ScBuildAnsiArgsVector(
            ControlPacket,
            core::ptr::addr_of_mut!((*ThreadParamsA).dwArgCount),
            core::ptr::addr_of_mut!((*ThreadParamsA).lpArgVector),
        );
        if dwError != ERROR_SUCCESS {
            HeapFree(GetProcessHeap(), 0, ThreadParamsA as LPVOID);
            return dwError;
        }
        (*ThreadParamsA).lpServiceMain = (*lpService).ServiceMain.A;
        (*ThreadParamsA).dwServiceTag = (*ControlPacket).dwServiceTag;
        ThreadHandle = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(ScServiceMainStubA),
            ThreadParamsA as LPVOID,
            0,
            core::ptr::addr_of_mut!(ThreadId),
        );
        if ThreadHandle.is_null() {
            if !(*ThreadParamsA).lpArgVector.is_null() {
                HeapFree(GetProcessHeap(), 0, (*ThreadParamsA).lpArgVector as LPVOID);
            }
            HeapFree(GetProcessHeap(), 0, ThreadParamsA as LPVOID);

            return ERROR_SERVICE_NO_THREAD;
        }

        CloseHandle(ThreadHandle);
    }

    ERROR_SUCCESS
}

unsafe fn ScControlService(lpService: PACTIVE_SERVICE,
                            ControlPacket: PSCM_CONTROL_PACKET) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let dwControl: DWORD;
    let mut dwEventType: DWORD = 0;
    let mut pEventData: PVOID = core::ptr::null_mut();

    if lpService.is_null() || ControlPacket.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    /* Set service tag */
    (*NtCurrentTeb()).SubProcessTag = UlongToPtr((*lpService).dwServiceTag);

    dwControl = (*ControlPacket).dwControl;

    if let Some(handler) = (*lpService).HandlerFunction {
        if ((dwControl >= SERVICE_CONTROL_STOP) && (dwControl <= SERVICE_CONTROL_NETBINDDISABLE)) ||
           ((dwControl >= 128) && (dwControl <= 255)) {
            handler(dwControl);
        } else {
            dwError = ERROR_INVALID_SERVICE_CONTROL;
        }
    } else if let Some(handler_ex) = (*lpService).HandlerFunctionEx {
        if dwControl == SERVICE_CONTROL_DEVICEEVENT {
            dwEventType = *((ControlPacket as ULONG_PTR + core::mem::size_of::<SCM_CONTROL_PACKET>() as ULONG_PTR) as LPDWORD);
            pEventData = (ControlPacket as ULONG_PTR + core::mem::size_of::<SCM_CONTROL_PACKET>() as ULONG_PTR + core::mem::size_of::<DWORD>() as ULONG_PTR) as PVOID;
        }

        handler_ex(dwControl,
                   dwEventType,
                   pEventData,
                   (*lpService).HandlerContext);
    } else {
        dwError = ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
    }

    /* Reset service tag */
    (*NtCurrentTeb()).SubProcessTag = core::ptr::null_mut();

    dwError
}

unsafe fn ScServiceDispatcher(
    hPipe: HANDLE,
    ControlPacket: PSCM_CONTROL_PACKET,
    dwBufferSize: DWORD,
) -> BOOL {
    let mut Count: DWORD = 0;
    let mut bResult: BOOL;
    let mut bRunning: BOOL = TRUE;
    let mut lpServiceName: LPWSTR;
    let mut lpService: PACTIVE_SERVICE;
    let mut ReplyPacket: SCM_REPLY_PACKET = core::mem::zeroed();
    let mut dwError: DWORD;

    if ControlPacket.is_null()
        || (dwBufferSize as usize) < core::mem::size_of::<SCM_CONTROL_PACKET>()
    {
        return FALSE;
    }

    while bRunning != FALSE {
        bResult = ReadFile(
            hPipe,
            ControlPacket as LPVOID,
            dwBufferSize,
            &mut Count,
            core::ptr::null_mut(),
        );
        if bResult == FALSE {
            return FALSE;
        }

        lpServiceName = (ControlPacket as PBYTE)
            .add((*ControlPacket).dwServiceNameOffset as usize)
            as LPWSTR;

        if (*ControlPacket).dwControl == SERVICE_CONTROL_STOP
            && *lpServiceName == UNICODE_NULL as u16
        {
            bRunning = FALSE;
            dwError = ERROR_SUCCESS;
        } else {
            if (*ControlPacket).dwControl == SERVICE_CONTROL_START_OWN {
                (*lpActiveServices.add(0)).bOwnProcess = TRUE;
            }

            lpService = ScLookupServiceByServiceName(lpServiceName);
            if !lpService.is_null() {
                match (*ControlPacket).dwControl {
                    SERVICE_CONTROL_START_SHARE | SERVICE_CONTROL_START_OWN => {
                        dwError = ScStartService(lpService, ControlPacket);
                    }
                    SERVICE_CONTROL_STOP => {
                        dwError = ScControlService(lpService, ControlPacket);
                    }
                    _ => {
                        dwError = ScControlService(lpService, ControlPacket);
                    }
                }
            } else {
                dwError = ERROR_SERVICE_NOT_IN_EXE;
            }
        }

        ReplyPacket.dwError = dwError;

        bResult = WriteFile(
            hPipe,
            &mut ReplyPacket as *mut _ as LPVOID,
            core::mem::size_of::<SCM_REPLY_PACKET>() as DWORD,
            &mut Count,
            core::ptr::null_mut(),
        );
        if bResult == FALSE {
            return FALSE;
        }
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn RegisterServiceCtrlHandlerA(
    lpServiceName: LPCSTR,
    lpHandlerProc: LPHANDLER_FUNCTION,
) -> SERVICE_STATUS_HANDLE {
    let mut ServiceNameA: ANSI_STRING = core::mem::zeroed();
    let mut ServiceNameU: _UNICODE_STRING = core::mem::zeroed();
    let hServiceStatus: SERVICE_STATUS_HANDLE;

    RtlInitAnsiString(&mut ServiceNameA, lpServiceName);
    if !NT_SUCCESS(RtlAnsiStringToUnicodeString(&mut ServiceNameU, &mut ServiceNameA, 1u8)) {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    hServiceStatus = RegisterServiceCtrlHandlerW(ServiceNameU.Buffer, lpHandlerProc);

    RtlFreeUnicodeString(&mut ServiceNameU);

    hServiceStatus
}

#[no_mangle]
pub unsafe extern "system" fn RegisterServiceCtrlHandlerW(
    lpServiceName: LPCWSTR,
    lpHandlerProc: LPHANDLER_FUNCTION,
) -> SERVICE_STATUS_HANDLE {
    let Service: PACTIVE_SERVICE = ScLookupServiceByServiceName(lpServiceName);
    if Service.is_null() {
        SetLastError(ERROR_SERVICE_NOT_IN_EXE);
        return core::ptr::null_mut();
    }

    if lpHandlerProc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    (*Service).HandlerFunction = lpHandlerProc;
    (*Service).HandlerFunctionEx = None;

    (*Service).hServiceStatus
}

#[no_mangle]
pub unsafe extern "system" fn RegisterServiceCtrlHandlerExA(
    lpServiceName: LPCSTR,
    lpHandlerProc: LPHANDLER_FUNCTION_EX,
    lpContext: LPVOID,
) -> SERVICE_STATUS_HANDLE {
    let mut ServiceNameA: ANSI_STRING = core::mem::zeroed();
    let mut ServiceNameU: _UNICODE_STRING = core::mem::zeroed();
    let hServiceStatus: SERVICE_STATUS_HANDLE;

    RtlInitAnsiString(&mut ServiceNameA, lpServiceName as PCSZ);
    if !NT_SUCCESS(RtlAnsiStringToUnicodeString(
        &mut ServiceNameU,
        &mut ServiceNameA,
        1u8,
    )) {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    hServiceStatus = RegisterServiceCtrlHandlerExW(
        ServiceNameU.Buffer,
        lpHandlerProc,
        lpContext,
    );

    RtlFreeUnicodeString(&mut ServiceNameU);

    hServiceStatus
}

#[no_mangle]
pub unsafe extern "system" fn RegisterServiceCtrlHandlerExW(
    lpServiceName: LPCWSTR,
    lpHandlerProc: LPHANDLER_FUNCTION_EX,
    lpContext: LPVOID,
) -> SERVICE_STATUS_HANDLE {
    let Service: PACTIVE_SERVICE = ScLookupServiceByServiceName(lpServiceName);
    if Service.is_null() {
        SetLastError(ERROR_SERVICE_NOT_IN_EXE);
        return core::ptr::null_mut();
    }

    if lpHandlerProc.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    (*Service).HandlerFunction = None;
    (*Service).HandlerFunctionEx = lpHandlerProc;
    (*Service).HandlerContext = lpContext;

    (*Service).hServiceStatus
}

#[no_mangle]
pub unsafe extern "system" fn I_ScIsSecurityProcess() {
    bSecurityServiceProcess = 1;
}

#[no_mangle]
pub unsafe extern "system" fn I_ScPnPGetServiceName(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    lpServiceName: LPWSTR,
    cchServiceName: DWORD,
) -> DWORD {
    let mut i: DWORD = 0;

    while i < dwActiveServiceCount {
        if (*lpActiveServices.offset(i as isize)).hServiceStatus == hServiceStatus {
            wcscpy(
                lpServiceName,
                (*lpActiveServices.offset(i as isize)).ServiceName.Buffer,
            );
            return ERROR_SUCCESS;
        }
        i += 1;
    }

    ERROR_SERVICE_NOT_IN_EXE
}

#[no_mangle]
pub unsafe extern "system" fn I_ScSetServiceBitsA(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    dwServiceBits: DWORD,
    bSetBitsOn: BOOL,
    bUpdateImmediately: BOOL,
    lpString: LPSTR,
) -> BOOL {
    let bResult: BOOL;

    bResult = rpc_try(|| RI_ScSetServiceBitsA(
        hServiceStatus as RPC_SERVICE_STATUS_HANDLE,
        dwServiceBits,
        bSetBitsOn,
        bUpdateImmediately,
        lpString,
    )).unwrap_or_else(|code| { SetLastError(ScmRpcStatusToWinError(code as _) as _); 0 as _ }) as BOOL;

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn I_ScSetServiceBitsW(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    dwServiceBits: DWORD,
    bSetBitsOn: BOOL,
    bUpdateImmediately: BOOL,
    lpString: LPWSTR,
) -> BOOL {
    let bResult: BOOL;

    TRACE!(
        b"I_ScSetServiceBitsW(%lu %lx %u %u %s)\n\0".as_ptr() as *const i8,
        hServiceStatus,
        dwServiceBits,
        bSetBitsOn,
        bUpdateImmediately,
        debugstr_w(lpString),
    );

    bResult = rpc_try(|| RI_ScSetServiceBitsW(
        hServiceStatus as RPC_SERVICE_STATUS_HANDLE,
        dwServiceBits,
        bSetBitsOn,
        bUpdateImmediately,
        lpString,
    )).unwrap_or_else(|code| { SetLastError(ScmRpcStatusToWinError(code as _) as _); 0 as _ }) as BOOL;

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn SetServiceBits(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    dwServiceBits: DWORD,
    bSetBitsOn: BOOL,
    bUpdateImmediately: BOOL,
) -> BOOL {
    I_ScSetServiceBitsW(
        hServiceStatus,
        dwServiceBits,
        bSetBitsOn,
        bUpdateImmediately,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn SetServiceStatus(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    lpServiceStatus: LPSERVICE_STATUS,
) -> BOOL {
    let dwError: DWORD;

    dwError = rpc_try(|| RSetServiceStatus(
        hServiceStatus as RPC_SERVICE_STATUS_HANDLE,
        lpServiceStatus,
    )).unwrap_or_else(|code| ScmRpcStatusToWinError(code as _) as _);

    if dwError != ERROR_SUCCESS {
        SetLastError(dwError);
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn StartServiceCtrlDispatcherA(
    lpServiceStartTable: *const SERVICE_TABLE_ENTRYA,
) -> BOOL {
    let mut i: ULONG;
    let mut hPipe: HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;
    let ControlPacket: PSCM_CONTROL_PACKET;
    let dwBufSize: DWORD;
    let mut bRet: BOOL = 1;

    TRACE!(b"StartServiceCtrlDispatcherA(%p)\n\0".as_ptr(), lpServiceStartTable);

    i = 0;
    while (*lpServiceStartTable.offset(i as isize)).lpServiceProc.is_some() {
        i += 1;
    }

    dwActiveServiceCount = i;

    lpActiveServices = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((dwActiveServiceCount as usize) * core::mem::size_of::<ACTIVE_SERVICE>()) as SIZE_T,
    ) as PACTIVE_SERVICE;
    if lpActiveServices.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    i = 0;
    while i < dwActiveServiceCount {
        RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!((*lpActiveServices.offset(i as isize)).ServiceName),
            (*lpServiceStartTable.offset(i as isize)).lpServiceName,
        );
        (*lpActiveServices.offset(i as isize)).ServiceMain.A =
            (*lpServiceStartTable.offset(i as isize)).lpServiceProc;
        (*lpActiveServices.offset(i as isize)).hServiceStatus = core::ptr::null_mut();
        (*lpActiveServices.offset(i as isize)).bUnicode = 0;
        (*lpActiveServices.offset(i as isize)).bOwnProcess = 0;
        i += 1;
    }

    'done: {
        dwError = ScConnectControlPipe(&mut hPipe);
        if dwError != ERROR_SUCCESS {
            bRet = 0;
            break 'done;
        }

        dwBufSize = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD
            + (MAX_SERVICE_NAME_LENGTH + 1) * core::mem::size_of::<WCHAR>() as DWORD;

        ControlPacket = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwBufSize as SIZE_T,
        ) as PSCM_CONTROL_PACKET;
        if ControlPacket.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            bRet = 0;
            break 'done;
        }

        ScCreateStatusBinding();

        ScServiceDispatcher(hPipe, ControlPacket, dwBufSize);

        ScDestroyStatusBinding();

        CloseHandle(hPipe);

        RtlFreeHeap(RtlGetProcessHeap(), 0, ControlPacket as PVOID);
    }

    i = 0;
    while i < dwActiveServiceCount {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(
            (*lpActiveServices.offset(i as isize)).ServiceName
        ));
        i += 1;
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, lpActiveServices as PVOID);
    lpActiveServices = core::ptr::null_mut();
    dwActiveServiceCount = 0;

    if bRet == 0 {
        SetLastError(dwError);
    }

    bRet
}

#[no_mangle]
pub unsafe extern "system" fn StartServiceCtrlDispatcherW(
    lpServiceStartTable: *const SERVICE_TABLE_ENTRYW,
) -> BOOL {
    let mut i: ULONG;
    let mut hPipe: HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;
    let ControlPacket: PSCM_CONTROL_PACKET;
    let dwBufSize: DWORD;
    let mut bRet: BOOL = 1;

    TRACE!(b"StartServiceCtrlDispatcherW(%p)\n\0".as_ptr(), lpServiceStartTable);

    i = 0;
    while (*lpServiceStartTable.offset(i as isize)).lpServiceProc.is_some() {
        i += 1;
    }

    dwActiveServiceCount = i;

    lpActiveServices = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwActiveServiceCount as usize * core::mem::size_of::<ACTIVE_SERVICE>()) as SIZE_T,
    ) as PACTIVE_SERVICE;
    if lpActiveServices.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    i = 0;
    while i < dwActiveServiceCount {
        RtlCreateUnicodeString(
            core::ptr::addr_of_mut!((*lpActiveServices.offset(i as isize)).ServiceName),
            (*lpServiceStartTable.offset(i as isize)).lpServiceName,
        );
        (*lpActiveServices.offset(i as isize)).ServiceMain.W =
            (*lpServiceStartTable.offset(i as isize)).lpServiceProc;
        (*lpActiveServices.offset(i as isize)).hServiceStatus = core::ptr::null_mut();
        (*lpActiveServices.offset(i as isize)).bUnicode = 1;
        (*lpActiveServices.offset(i as isize)).bOwnProcess = 0;
        i += 1;
    }

    'done: {
        dwError = ScConnectControlPipe(&mut hPipe);
        if dwError != ERROR_SUCCESS {
            bRet = 0;
            break 'done;
        }

        dwBufSize = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD
            + (MAX_SERVICE_NAME_LENGTH + 1) * core::mem::size_of::<WCHAR>() as DWORD;

        ControlPacket = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwBufSize as SIZE_T,
        ) as PSCM_CONTROL_PACKET;
        if ControlPacket.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            bRet = 0;
            break 'done;
        }

        ScCreateStatusBinding();

        ScServiceDispatcher(hPipe, ControlPacket, dwBufSize);

        ScDestroyStatusBinding();

        CloseHandle(hPipe);

        RtlFreeHeap(RtlGetProcessHeap(), 0, ControlPacket as LPVOID);
    }

    i = 0;
    while i < dwActiveServiceCount {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(
            (*lpActiveServices.offset(i as isize)).ServiceName
        ));
        i += 1;
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, lpActiveServices as LPVOID);
    lpActiveServices = core::ptr::null_mut();
    dwActiveServiceCount = 0;

    if bRet == 0 {
        SetLastError(dwError);
    }

    bRet
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS advapi32
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/advapi32/service/sctrl.c
//  * PURPOSE:     Service control manager functions
//  * COPYRIGHT:   Copyright 1999 Emanuele Aliberti
//  *              Copyright 2007 Ged Murphy <gedmurphy@reactos.org>
//  *                             Gregor Brunmar <gregor.brunmar@home.se>
//  */
//
//
// /* INCLUDES ******************************************************************/
//
// #include <advapi32.h>
// #include <pseh/pseh2.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
//
// /* TYPES *********************************************************************/
//
// typedef struct _SERVICE_THREAD_PARAMSA
// {
//     LPSERVICE_MAIN_FUNCTIONA lpServiceMain;
//     DWORD dwArgCount;
//     LPSTR *lpArgVector;
//     DWORD dwServiceTag;
// } SERVICE_THREAD_PARAMSA, *PSERVICE_THREAD_PARAMSA;
//
//
// typedef struct _SERVICE_THREAD_PARAMSW
// {
//     LPSERVICE_MAIN_FUNCTIONW lpServiceMain;
//     DWORD dwArgCount;
//     LPWSTR *lpArgVector;
//     DWORD dwServiceTag;
// } SERVICE_THREAD_PARAMSW, *PSERVICE_THREAD_PARAMSW;
//
//
// typedef struct _ACTIVE_SERVICE
// {
//     SERVICE_STATUS_HANDLE hServiceStatus;
//     UNICODE_STRING ServiceName;
//     union
//     {
//         LPSERVICE_MAIN_FUNCTIONA A;
//         LPSERVICE_MAIN_FUNCTIONW W;
//     } ServiceMain;
//     LPHANDLER_FUNCTION HandlerFunction;
//     LPHANDLER_FUNCTION_EX HandlerFunctionEx;
//     LPVOID HandlerContext;
//     BOOL bUnicode;
//     BOOL bOwnProcess;
//     DWORD dwServiceTag;
// } ACTIVE_SERVICE, *PACTIVE_SERVICE;
//
//
// /* GLOBALS *******************************************************************/
//
// static DWORD dwActiveServiceCount = 0;
// static PACTIVE_SERVICE lpActiveServices = NULL;
// static handle_t hStatusBinding = NULL;
// static BOOL bSecurityServiceProcess = FALSE;
//
//
// /* FUNCTIONS *****************************************************************/
//
// handle_t __RPC_USER
// RPC_SERVICE_STATUS_HANDLE_bind(RPC_SERVICE_STATUS_HANDLE hServiceStatus)
// {
//     return hStatusBinding;
// }
//
//
// void __RPC_USER
// RPC_SERVICE_STATUS_HANDLE_unbind(RPC_SERVICE_STATUS_HANDLE hServiceStatus,
//                                  handle_t hBinding)
// {
// }
//
//
// static RPC_STATUS
// ScCreateStatusBinding(VOID)
// {
//     LPWSTR pszStringBinding;
//     RPC_STATUS status;
//
//     TRACE("ScCreateStatusBinding()\n");
//
//     status = RpcStringBindingComposeW(NULL,
//                                       L"ncacn_np",
//                                       NULL,
//                                       L"\\pipe\\ntsvcs",
//                                       NULL,
//                                       &pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringBindingCompose returned 0x%x\n", status);
//         return status;
//     }
//
//     /* Set the binding handle that will be used to bind to the server. */
//     status = RpcBindingFromStringBindingW(pszStringBinding,
//                                           &hStatusBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFromStringBinding returned 0x%x\n", status);
//     }
//
//     status = RpcStringFreeW(&pszStringBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcStringFree returned 0x%x\n", status);
//     }
//
//     return status;
// }
//
//
// static RPC_STATUS
// ScDestroyStatusBinding(VOID)
// {
//     RPC_STATUS status;
//
//     TRACE("ScDestroyStatusBinding()\n");
//
//     if (hStatusBinding == NULL)
//         return RPC_S_OK;
//
//     status = RpcBindingFree(&hStatusBinding);
//     if (status != RPC_S_OK)
//     {
//         ERR("RpcBindingFree returned 0x%x\n", status);
//     }
//     else
//     {
//         hStatusBinding = NULL;
//     }
//
//     return status;
// }
//
//
// static PACTIVE_SERVICE
// ScLookupServiceByServiceName(LPCWSTR lpServiceName)
// {
//     DWORD i;
//
//     TRACE("ScLookupServiceByServiceName(%S)\n",
//           lpServiceName);
//
//     if (lpActiveServices[0].bOwnProcess)
//         return &lpActiveServices[0];
//
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         TRACE("Checking %S\n", lpActiveServices[i].ServiceName.Buffer);
//         if (_wcsicmp(lpActiveServices[i].ServiceName.Buffer, lpServiceName) == 0)
//         {
//             TRACE("Found!\n");
//             return &lpActiveServices[i];
//         }
//     }
//
//     TRACE("No service found!\n");
//     return NULL;
// }
//
//
// static DWORD WINAPI
// ScServiceMainStubA(LPVOID Context)
// {
//     PTEB Teb;
//     PSERVICE_THREAD_PARAMSA ThreadParams = Context;
//
//     TRACE("ScServiceMainStubA(%p)\n", Context);
//
//     /* Set service tag */
//     Teb = NtCurrentTeb();
//     Teb->SubProcessTag = UlongToPtr(ThreadParams->dwServiceTag);
//
//     /* Call the main service routine and free the arguments vector */
//     (ThreadParams->lpServiceMain)(ThreadParams->dwArgCount,
//                                   ThreadParams->lpArgVector);
//
//     /* Reset service tag */
//     Teb->SubProcessTag = 0;
//
//     if (ThreadParams->lpArgVector != NULL)
//     {
//         HeapFree(GetProcessHeap(), 0, ThreadParams->lpArgVector);
//     }
//     HeapFree(GetProcessHeap(), 0, ThreadParams);
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD WINAPI
// ScServiceMainStubW(LPVOID Context)
// {
//     PTEB Teb;
//     PSERVICE_THREAD_PARAMSW ThreadParams = Context;
//
//     TRACE("ScServiceMainStubW(%p)\n", Context);
//
//     /* Set service tag */
//     Teb = NtCurrentTeb();
//     Teb->SubProcessTag = UlongToPtr(ThreadParams->dwServiceTag);
//
//     /* Call the main service routine and free the arguments vector */
//     (ThreadParams->lpServiceMain)(ThreadParams->dwArgCount,
//                                   ThreadParams->lpArgVector);
//
//     /* Reset service tag */
//     Teb->SubProcessTag = 0;
//
//     if (ThreadParams->lpArgVector != NULL)
//     {
//         HeapFree(GetProcessHeap(), 0, ThreadParams->lpArgVector);
//     }
//     HeapFree(GetProcessHeap(), 0, ThreadParams);
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScConnectControlPipe(HANDLE *hPipe)
// {
//     DWORD dwBytesWritten;
//     DWORD dwState;
//     DWORD dwServiceCurrent = 1;
//     NTSTATUS Status;
//     WCHAR NtControlPipeName[MAX_PATH + 1];
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//     DWORD dwProcessId;
//
//     TRACE("ScConnectControlPipe(%p)\n",
//           hPipe);
//
//     /* Get the service number and create the named pipe */
//     if (bSecurityServiceProcess == FALSE)
//     {
//         RtlZeroMemory(&QueryTable, sizeof(QueryTable));
//
//         QueryTable[0].Name = L"";
//         QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED;
//         QueryTable[0].EntryContext = &dwServiceCurrent;
//
//         Status = RtlQueryRegistryValues(RTL_REGISTRY_CONTROL,
//                                         L"ServiceCurrent",
//                                         QueryTable,
//                                         NULL,
//                                         NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("RtlQueryRegistryValues() failed (Status %lx)\n", Status);
//             return RtlNtStatusToDosError(Status);
//         }
//     }
//     else
//     {
//         dwServiceCurrent = 0;
//     }
//
//     swprintf(NtControlPipeName, L"\\\\.\\pipe\\net\\NtControlPipe%u", dwServiceCurrent);
//     TRACE("PipeName: %S\n", NtControlPipeName);
//
//     if (!WaitNamedPipeW(NtControlPipeName, 30000))
//     {
//         ERR("WaitNamedPipe(%S) failed (Error %lu)\n", NtControlPipeName, GetLastError());
//         return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
//     }
//
//     *hPipe = CreateFileW(NtControlPipeName,
//                          GENERIC_READ | GENERIC_WRITE,
//                          FILE_SHARE_READ | FILE_SHARE_WRITE,
//                          NULL,
//                          OPEN_EXISTING,
//                          FILE_ATTRIBUTE_NORMAL,
//                          NULL);
//     if (*hPipe == INVALID_HANDLE_VALUE)
//     {
//         ERR("CreateFileW() failed for pipe %S (Error %lu)\n", NtControlPipeName, GetLastError());
//         return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
//     }
//
//     dwState = PIPE_READMODE_MESSAGE;
//     if (!SetNamedPipeHandleState(*hPipe, &dwState, NULL, NULL))
//     {
//         CloseHandle(*hPipe);
//         *hPipe = INVALID_HANDLE_VALUE;
//         return ERROR_FAILED_SERVICE_CONTROLLER_CONNECT;
//     }
//
//     /* Pass the ProcessId to the SCM */
//     dwProcessId = GetCurrentProcessId();
//     WriteFile(*hPipe,
//               &dwProcessId,
//               sizeof(dwProcessId),
//               &dwBytesWritten,
//               NULL);
//
//     TRACE("Sent Process ID %lu\n", dwProcessId);
//
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * Ansi/Unicode argument layout of the vector passed to a service at startup,
//  * depending on the different versions of Windows considered:
//  *
//  * - XP/2003:
//  *   [argv array of pointers][parameter 1][parameter 2]...[service name]
//  *
//  * - Vista:
//  *   [argv array of pointers][align to 8 bytes]
//  *   [parameter 1][parameter 2]...[service name]
//  *
//  * - Win7/8:
//  *   [argv array of pointers][service name]
//  *   [parameter 1][align to 4 bytes][parameter 2][align to 4 bytes]...
//  *
//  * Space for parameters and service name is always enough to store
//  * both the Ansi and the Unicode versions including NULL terminator.
//  */
//
// static DWORD
// ScBuildUnicodeArgsVector(PSCM_CONTROL_PACKET ControlPacket,
//                          LPDWORD lpArgCount,
//                          LPWSTR **lpArgVector)
// {
//     PWSTR *lpVector;
//     PWSTR pszServiceName;
//     DWORD cbServiceName;
//     DWORD cbArguments;
//     DWORD cbTotal;
//     DWORD i;
//
//     if (ControlPacket == NULL || lpArgCount == NULL || lpArgVector == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     *lpArgCount  = 0;
//     *lpArgVector = NULL;
//
//     /* Retrieve and count the start command line (NULL-terminated) */
//     pszServiceName = (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwServiceNameOffset);
//     cbServiceName  = lstrlenW(pszServiceName) * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//
//     /*
//      * The total size of the argument vector is equal to the entry for
//      * the service name, plus the size of the original argument vector.
//      */
//     cbTotal = sizeof(PWSTR) + cbServiceName;
//     if (ControlPacket->dwArgumentsCount > 0)
//         cbArguments = ControlPacket->dwSize - ControlPacket->dwArgumentsOffset;
//     else
//         cbArguments = 0;
//     cbTotal += cbArguments;
//
//     /* Allocate the new argument vector */
//     lpVector = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbTotal);
//     if (lpVector == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     /*
//      * The first argument is reserved for the service name, which
//      * will be appended to the end of the argument string list.
//      */
//
//     /* Copy the remaining arguments */
//     if (ControlPacket->dwArgumentsCount > 0)
//     {
//         memcpy(&lpVector[1],
//                (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwArgumentsOffset),
//                cbArguments);
//
//         for (i = 0; i < ControlPacket->dwArgumentsCount; i++)
//         {
//             lpVector[i + 1] = (PWSTR)((ULONG_PTR)&lpVector[1] + (ULONG_PTR)lpVector[i + 1]);
//             TRACE("Unicode lpVector[%lu] = '%ls'\n", i + 1, lpVector[i + 1]);
//         }
//     }
//
//     /* Now copy the service name */
//     lpVector[0] = (PWSTR)((ULONG_PTR)&lpVector[1] + cbArguments);
//     memcpy(lpVector[0], pszServiceName, cbServiceName);
//     TRACE("Unicode lpVector[%lu] = '%ls'\n", 0, lpVector[0]);
//
//     *lpArgCount  = ControlPacket->dwArgumentsCount + 1;
//     *lpArgVector = lpVector;
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScBuildAnsiArgsVector(PSCM_CONTROL_PACKET ControlPacket,
//                       LPDWORD lpArgCount,
//                       LPSTR **lpArgVector)
// {
//     DWORD dwError;
//     NTSTATUS Status;
//     DWORD ArgCount, i;
//     PWSTR *lpVectorW;
//     PSTR  *lpVectorA;
//     UNICODE_STRING UnicodeString;
//     ANSI_STRING AnsiString;
//
//     if (ControlPacket == NULL || lpArgCount == NULL || lpArgVector == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     *lpArgCount  = 0;
//     *lpArgVector = NULL;
//
//     /* Build the UNICODE arguments vector */
//     dwError = ScBuildUnicodeArgsVector(ControlPacket, &ArgCount, &lpVectorW);
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     /* Convert the vector to ANSI in place */
//     lpVectorA = (PSTR*)lpVectorW;
//     for (i = 0; i < ArgCount; i++)
//     {
//         RtlInitUnicodeString(&UnicodeString, lpVectorW[i]);
//         RtlInitEmptyAnsiString(&AnsiString, lpVectorA[i], UnicodeString.MaximumLength);
//         Status = RtlUnicodeStringToAnsiString(&AnsiString, &UnicodeString, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Failed to convert to ANSI; free the allocated vector and return */
//             dwError = RtlNtStatusToDosError(Status);
//             HeapFree(GetProcessHeap(), 0, lpVectorW);
//             return dwError;
//         }
//
//         /* NULL-terminate the string */
//         AnsiString.Buffer[AnsiString.Length / sizeof(CHAR)] = ANSI_NULL;
//
//         TRACE("Ansi lpVector[%lu] = '%s'\n", i, lpVectorA[i]);
//     }
//
//     *lpArgCount  = ArgCount;
//     *lpArgVector = lpVectorA;
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScStartService(PACTIVE_SERVICE lpService,
//                PSCM_CONTROL_PACKET ControlPacket)
// {
//     HANDLE ThreadHandle;
//     DWORD ThreadId;
//     DWORD dwError;
//     PSERVICE_THREAD_PARAMSA ThreadParamsA;
//     PSERVICE_THREAD_PARAMSW ThreadParamsW;
//
//     TRACE("ScStartService(%p %p)\n",
//           lpService, ControlPacket);
//
//     if (lpService == NULL || ControlPacket == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     TRACE("Size: %lu\n", ControlPacket->dwSize);
//     TRACE("Service: %S\n", (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwServiceNameOffset));
//
//     /* Set the service status handle */
//     lpService->hServiceStatus = ControlPacket->hServiceStatus;
//     /* Set the service tag */
//     lpService->dwServiceTag = ControlPacket->dwServiceTag;
//
//     /* Build the arguments vector */
//     if (lpService->bUnicode != FALSE)
//     {
//         ThreadParamsW = HeapAlloc(GetProcessHeap(), 0, sizeof(*ThreadParamsW));
//         if (ThreadParamsW == NULL)
//             return ERROR_NOT_ENOUGH_MEMORY;
//         dwError = ScBuildUnicodeArgsVector(ControlPacket,
//                                            &ThreadParamsW->dwArgCount,
//                                            &ThreadParamsW->lpArgVector);
//         if (dwError != ERROR_SUCCESS)
//         {
//             HeapFree(GetProcessHeap(), 0, ThreadParamsW);
//             return dwError;
//         }
//         ThreadParamsW->lpServiceMain = lpService->ServiceMain.W;
//         ThreadParamsW->dwServiceTag = ControlPacket->dwServiceTag;
//         ThreadHandle = CreateThread(NULL,
//                                     0,
//                                     ScServiceMainStubW,
//                                     ThreadParamsW,
//                                     0,
//                                     &ThreadId);
//         if (ThreadHandle == NULL)
//         {
//             if (ThreadParamsW->lpArgVector != NULL)
//             {
//                 HeapFree(GetProcessHeap(), 0, ThreadParamsW->lpArgVector);
//             }
//             HeapFree(GetProcessHeap(), 0, ThreadParamsW);
//
//             return ERROR_SERVICE_NO_THREAD;
//         }
//
//         CloseHandle(ThreadHandle);
//     }
//     else
//     {
//         ThreadParamsA = HeapAlloc(GetProcessHeap(), 0, sizeof(*ThreadParamsA));
//         if (ThreadParamsA == NULL)
//             return ERROR_NOT_ENOUGH_MEMORY;
//         dwError = ScBuildAnsiArgsVector(ControlPacket,
//                                         &ThreadParamsA->dwArgCount,
//                                         &ThreadParamsA->lpArgVector);
//         if (dwError != ERROR_SUCCESS)
//         {
//             HeapFree(GetProcessHeap(), 0, ThreadParamsA);
//             return dwError;
//         }
//         ThreadParamsA->lpServiceMain = lpService->ServiceMain.A;
//         ThreadParamsA->dwServiceTag = ControlPacket->dwServiceTag;
//         ThreadHandle = CreateThread(NULL,
//                                     0,
//                                     ScServiceMainStubA,
//                                     ThreadParamsA,
//                                     0,
//                                     &ThreadId);
//         if (ThreadHandle == NULL)
//         {
//             if (ThreadParamsA->lpArgVector != NULL)
//             {
//                 HeapFree(GetProcessHeap(), 0, ThreadParamsA->lpArgVector);
//             }
//             HeapFree(GetProcessHeap(), 0, ThreadParamsA);
//
//             return ERROR_SERVICE_NO_THREAD;
//         }
//
//         CloseHandle(ThreadHandle);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScControlService(PACTIVE_SERVICE lpService,
//                  PSCM_CONTROL_PACKET ControlPacket)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     DWORD dwControl;
//     DWORD dwEventType = 0;
//     PVOID pEventData = NULL;
//
//     TRACE("ScControlService(%p %p)\n",
//           lpService, ControlPacket);
//
//     if (lpService == NULL || ControlPacket == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     TRACE("Size: %lu\n", ControlPacket->dwSize);
//     TRACE("Service: %S\n", (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwServiceNameOffset));
//
//     /* Set service tag */
//     NtCurrentTeb()->SubProcessTag = UlongToPtr(lpService->dwServiceTag);
//
//     dwControl = ControlPacket->dwControl;
//
//     if (lpService->HandlerFunction)
//     {
//         if (((dwControl >= SERVICE_CONTROL_STOP) && (dwControl <= SERVICE_CONTROL_NETBINDDISABLE)) ||
//             ((dwControl >= 128) && (dwControl <= 255)))
//         {
//             _SEH2_TRY
//             {
//                 (lpService->HandlerFunction)(dwControl);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 dwError = ERROR_EXCEPTION_IN_SERVICE;
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             dwError = ERROR_INVALID_SERVICE_CONTROL;
//         }
//     }
//     else if (lpService->HandlerFunctionEx)
//     {
//         if (dwControl == SERVICE_CONTROL_DEVICEEVENT)
//         {
//             dwEventType = *(LPDWORD)((ULONG_PTR)ControlPacket + sizeof(SCM_CONTROL_PACKET));
//             pEventData = (PVOID)((ULONG_PTR)ControlPacket + sizeof(SCM_CONTROL_PACKET) + sizeof(DWORD));
//         }
//
//         _SEH2_TRY
//         {
//             (lpService->HandlerFunctionEx)(dwControl,
//                                            dwEventType,
//                                            pEventData,
//                                            lpService->HandlerContext);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             dwError = ERROR_EXCEPTION_IN_SERVICE;
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         dwError = ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
//     }
//
//     /* Reset service tag */
//     NtCurrentTeb()->SubProcessTag = 0;
//
//     TRACE("ScControlService() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// static BOOL
// ScServiceDispatcher(HANDLE hPipe,
//                     PSCM_CONTROL_PACKET ControlPacket,
//                     DWORD dwBufferSize)
// {
//     DWORD Count;
//     BOOL bResult;
//     BOOL bRunning = TRUE;
//     LPWSTR lpServiceName;
//     PACTIVE_SERVICE lpService;
//     SCM_REPLY_PACKET ReplyPacket;
//     DWORD dwError;
//
//     TRACE("ScServiceDispatcher(%p %p %lu)\n",
//           hPipe, ControlPacket, dwBufferSize);
//
//     if (ControlPacket == NULL || dwBufferSize < sizeof(SCM_CONTROL_PACKET))
//         return FALSE;
//
//     while (bRunning)
//     {
//         /* Read command from the control pipe */
//         bResult = ReadFile(hPipe,
//                            ControlPacket,
//                            dwBufferSize,
//                            &Count,
//                            NULL);
//         if (bResult == FALSE)
//         {
//             ERR("Pipe read failed (Error: %lu)\n", GetLastError());
//             return FALSE;
//         }
//
//         lpServiceName = (LPWSTR)((PBYTE)ControlPacket + ControlPacket->dwServiceNameOffset);
//         TRACE("Service: %S\n", lpServiceName);
//
//         if ((ControlPacket->dwControl == SERVICE_CONTROL_STOP) &&
//             (lpServiceName[0] == UNICODE_NULL))
//         {
//             TRACE("Stop dispatcher thread\n");
//             bRunning = FALSE;
//             dwError = ERROR_SUCCESS;
//         }
//         else
//         {
//             if (ControlPacket->dwControl == SERVICE_CONTROL_START_OWN)
//                 lpActiveServices[0].bOwnProcess = TRUE;
//
//             lpService = ScLookupServiceByServiceName(lpServiceName);
//             if (lpService != NULL)
//             {
//                 /* Execute command */
//                 switch (ControlPacket->dwControl)
//                 {
//                     case SERVICE_CONTROL_START_SHARE:
//                     case SERVICE_CONTROL_START_OWN:
//                         TRACE("Start command - received SERVICE_CONTROL_START\n");
//                         dwError = ScStartService(lpService, ControlPacket);
//                         break;
//
//                     case SERVICE_CONTROL_STOP:
//                         TRACE("Stop command - received SERVICE_CONTROL_STOP\n");
//                         dwError = ScControlService(lpService, ControlPacket);
//                         break;
//
//                     default:
//                         TRACE("Command %lu received\n", ControlPacket->dwControl);
//                         dwError = ScControlService(lpService, ControlPacket);
//                         break;
//                 }
//             }
//             else
//             {
//                 dwError = ERROR_SERVICE_NOT_IN_EXE;
//             }
//         }
//
//         ReplyPacket.dwError = dwError;
//
//         /* Send the reply packet */
//         bResult = WriteFile(hPipe,
//                             &ReplyPacket,
//                             sizeof(ReplyPacket),
//                             &Count,
//                             NULL);
//         if (bResult == FALSE)
//         {
//             ERR("Pipe write failed (Error: %lu)\n", GetLastError());
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *  RegisterServiceCtrlHandlerA
//  *
//  * @implemented
//  */
// SERVICE_STATUS_HANDLE WINAPI
// RegisterServiceCtrlHandlerA(LPCSTR lpServiceName,
//                             LPHANDLER_FUNCTION lpHandlerProc)
// {
//     ANSI_STRING ServiceNameA;
//     UNICODE_STRING ServiceNameU;
//     SERVICE_STATUS_HANDLE hServiceStatus;
//
//     TRACE("RegisterServiceCtrlHandlerA(%s %p)\n",
//           debugstr_a(lpServiceName), lpHandlerProc);
//
//     RtlInitAnsiString(&ServiceNameA, lpServiceName);
//     if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&ServiceNameU, &ServiceNameA, TRUE)))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     hServiceStatus = RegisterServiceCtrlHandlerW(ServiceNameU.Buffer,
//                                                  lpHandlerProc);
//
//     RtlFreeUnicodeString(&ServiceNameU);
//
//     return hServiceStatus;
// }
//
//
// /**********************************************************************
//  *  RegisterServiceCtrlHandlerW
//  *
//  * @implemented
//  */
// SERVICE_STATUS_HANDLE WINAPI
// RegisterServiceCtrlHandlerW(LPCWSTR lpServiceName,
//                             LPHANDLER_FUNCTION lpHandlerProc)
// {
//     PACTIVE_SERVICE Service;
//
//     TRACE("RegisterServiceCtrlHandlerW(%s %p)\n",
//           debugstr_w(lpServiceName), lpHandlerProc);
//
//     Service = ScLookupServiceByServiceName(lpServiceName);
//     if (Service == NULL)
//     {
//         SetLastError(ERROR_SERVICE_NOT_IN_EXE);
//         return NULL;
//     }
//
//     if (!lpHandlerProc)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     Service->HandlerFunction = lpHandlerProc;
//     Service->HandlerFunctionEx = NULL;
//
//     TRACE("RegisterServiceCtrlHandler returning %p\n", Service->hServiceStatus);
//
//     return Service->hServiceStatus;
// }
//
//
// /**********************************************************************
//  *  RegisterServiceCtrlHandlerExA
//  *
//  * @implemented
//  */
// SERVICE_STATUS_HANDLE WINAPI
// RegisterServiceCtrlHandlerExA(LPCSTR lpServiceName,
//                               LPHANDLER_FUNCTION_EX lpHandlerProc,
//                               LPVOID lpContext)
// {
//     ANSI_STRING ServiceNameA;
//     UNICODE_STRING ServiceNameU;
//     SERVICE_STATUS_HANDLE hServiceStatus;
//
//     TRACE("RegisterServiceCtrlHandlerExA(%s %p %p)\n",
//           debugstr_a(lpServiceName), lpHandlerProc, lpContext);
//
//     RtlInitAnsiString(&ServiceNameA, lpServiceName);
//     if (!NT_SUCCESS(RtlAnsiStringToUnicodeString(&ServiceNameU, &ServiceNameA, TRUE)))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return NULL;
//     }
//
//     hServiceStatus = RegisterServiceCtrlHandlerExW(ServiceNameU.Buffer,
//                                                    lpHandlerProc,
//                                                    lpContext);
//
//     RtlFreeUnicodeString(&ServiceNameU);
//
//     return hServiceStatus;
// }
//
//
// /**********************************************************************
//  *  RegisterServiceCtrlHandlerExW
//  *
//  * @implemented
//  */
// SERVICE_STATUS_HANDLE WINAPI
// RegisterServiceCtrlHandlerExW(LPCWSTR lpServiceName,
//                               LPHANDLER_FUNCTION_EX lpHandlerProc,
//                               LPVOID lpContext)
// {
//     PACTIVE_SERVICE Service;
//
//     TRACE("RegisterServiceCtrlHandlerExW(%s %p %p)\n",
//           debugstr_w(lpServiceName), lpHandlerProc, lpContext);
//
//     Service = ScLookupServiceByServiceName(lpServiceName);
//     if (Service == NULL)
//     {
//         SetLastError(ERROR_SERVICE_NOT_IN_EXE);
//         return NULL;
//     }
//
//     if (!lpHandlerProc)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     Service->HandlerFunction = NULL;
//     Service->HandlerFunctionEx = lpHandlerProc;
//     Service->HandlerContext = lpContext;
//
//     TRACE("RegisterServiceCtrlHandlerEx returning %p\n", Service->hServiceStatus);
//
//     return Service->hServiceStatus;
// }
//
//
// /**********************************************************************
//  *  I_ScIsSecurityProcess
//  *
//  * Undocumented
//  *
//  * @implemented
//  */
// VOID
// WINAPI
// I_ScIsSecurityProcess(VOID)
// {
//     TRACE("I_ScIsSecurityProcess()\n");
//     bSecurityServiceProcess = TRUE;
// }
//
//
// /**********************************************************************
//  *  I_ScPnPGetServiceName
//  *
//  * Undocumented
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// I_ScPnPGetServiceName(IN SERVICE_STATUS_HANDLE hServiceStatus,
//                       OUT LPWSTR lpServiceName,
//                       IN DWORD cchServiceName)
// {
//     DWORD i;
//
//     TRACE("I_ScPnPGetServiceName(%lu %p %lu)\n",
//           hServiceStatus, lpServiceName, cchServiceName);
//
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         if (lpActiveServices[i].hServiceStatus == hServiceStatus)
//         {
//             wcscpy(lpServiceName, lpActiveServices[i].ServiceName.Buffer);
//             return ERROR_SUCCESS;
//         }
//     }
//
//     return ERROR_SERVICE_NOT_IN_EXE;
// }
//
//
// /**********************************************************************
//  *  I_ScSetServiceBitsA
//  *
//  * Undocumented
//  *
//  * @implemented
//  */
// BOOL WINAPI
// I_ScSetServiceBitsA(SERVICE_STATUS_HANDLE hServiceStatus,
//                     DWORD dwServiceBits,
//                     BOOL bSetBitsOn,
//                     BOOL bUpdateImmediately,
//                     LPSTR lpString)
// {
//     BOOL bResult;
//
//     TRACE("I_ScSetServiceBitsA(%lu %lx %u %u %s)\n",
//           hServiceStatus, dwServiceBits, bSetBitsOn, bUpdateImmediately,
//           debugstr_a(lpString));
//
//     RpcTryExcept
//     {
//         bResult = RI_ScSetServiceBitsA((RPC_SERVICE_STATUS_HANDLE)hServiceStatus,
//                                        dwServiceBits,
//                                        bSetBitsOn,
//                                        bUpdateImmediately,
//                                        lpString);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastError(ScmRpcStatusToWinError(RpcExceptionCode()));
//         bResult = FALSE;
//     }
//     RpcEndExcept;
//
//     return bResult;
// }
//
//
// /**********************************************************************
//  *  I_ScSetServiceBitsW
//  *
//  * Undocumented
//  *
//  * @implemented
//  */
// BOOL WINAPI
// I_ScSetServiceBitsW(SERVICE_STATUS_HANDLE hServiceStatus,
//                     DWORD dwServiceBits,
//                     BOOL bSetBitsOn,
//                     BOOL bUpdateImmediately,
//                     LPWSTR lpString)
// {
//     BOOL bResult;
//
//     TRACE("I_ScSetServiceBitsW(%lu %lx %u %u %s)\n",
//           hServiceStatus, dwServiceBits, bSetBitsOn, bUpdateImmediately,
//           debugstr_w(lpString));
//
//     RpcTryExcept
//     {
//         bResult = RI_ScSetServiceBitsW((RPC_SERVICE_STATUS_HANDLE)hServiceStatus,
//                                        dwServiceBits,
//                                        bSetBitsOn,
//                                        bUpdateImmediately,
//                                        lpString);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastError(ScmRpcStatusToWinError(RpcExceptionCode()));
//         bResult = FALSE;
//     }
//     RpcEndExcept;
//
//     return bResult;
// }
//
//
// /**********************************************************************
//  *  SetServiceBits
//  *
//  * @implemented
//  */
// BOOL WINAPI
// SetServiceBits(SERVICE_STATUS_HANDLE hServiceStatus,
//                DWORD dwServiceBits,
//                BOOL bSetBitsOn,
//                BOOL bUpdateImmediately)
// {
//     TRACE("SetServiceBits(%lu %lx %u %u)\n",
//           hServiceStatus, dwServiceBits, bSetBitsOn, bUpdateImmediately);
//
//     return I_ScSetServiceBitsW(hServiceStatus,
//                                dwServiceBits,
//                                bSetBitsOn,
//                                bUpdateImmediately,
//                                NULL);
// }
//
//
// /**********************************************************************
//  *  SetServiceStatus
//  *
//  * @implemented
//  */
// BOOL WINAPI
// SetServiceStatus(SERVICE_STATUS_HANDLE hServiceStatus,
//                  LPSERVICE_STATUS lpServiceStatus)
// {
//     DWORD dwError;
//
//     TRACE("SetServiceStatus(%lu %p)\n",
//           hServiceStatus, lpServiceStatus);
//
//     RpcTryExcept
//     {
//         dwError = RSetServiceStatus((RPC_SERVICE_STATUS_HANDLE)hServiceStatus,
//                                     lpServiceStatus);
//     }
//     RpcExcept(EXCEPTION_EXECUTE_HANDLER)
//     {
//         dwError = ScmRpcStatusToWinError(RpcExceptionCode());
//     }
//     RpcEndExcept;
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("RSetServiceStatus() failed (Error %lu)\n", dwError);
//         SetLastError(dwError);
//         return FALSE;
//     }
//
//     TRACE("SetServiceStatus() done\n");
//
//     return TRUE;
// }
//
//
// /**********************************************************************
//  *  StartServiceCtrlDispatcherA
//  *
//  * @implemented
//  */
// BOOL WINAPI
// StartServiceCtrlDispatcherA(const SERVICE_TABLE_ENTRYA *lpServiceStartTable)
// {
//     ULONG i;
//     HANDLE hPipe;
//     DWORD dwError;
//     PSCM_CONTROL_PACKET ControlPacket;
//     DWORD dwBufSize;
//     BOOL bRet = TRUE;
//
//     TRACE("StartServiceCtrlDispatcherA(%p)\n",
//           lpServiceStartTable);
//
//     i = 0;
//     while (lpServiceStartTable[i].lpServiceProc != NULL)
//     {
//         i++;
//     }
//
//     dwActiveServiceCount = i;
//
//     /* Allocate the service table */
//     lpActiveServices = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        HEAP_ZERO_MEMORY,
//                                        dwActiveServiceCount * sizeof(ACTIVE_SERVICE));
//     if (lpActiveServices == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Copy service names and start procedure */
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         RtlCreateUnicodeStringFromAsciiz(&lpActiveServices[i].ServiceName,
//                                          lpServiceStartTable[i].lpServiceName);
//         lpActiveServices[i].ServiceMain.A = lpServiceStartTable[i].lpServiceProc;
//         lpActiveServices[i].hServiceStatus = NULL;
//         lpActiveServices[i].bUnicode = FALSE;
//         lpActiveServices[i].bOwnProcess = FALSE;
//     }
//
//     /* Connect to the SCM */
//     dwError = ScConnectControlPipe(&hPipe);
//     if (dwError != ERROR_SUCCESS)
//     {
//         bRet = FALSE;
//         goto done;
//     }
//
//     dwBufSize = sizeof(SCM_CONTROL_PACKET) +
//                 (MAX_SERVICE_NAME_LENGTH + 1) * sizeof(WCHAR);
//
//     ControlPacket = RtlAllocateHeap(RtlGetProcessHeap(),
//                                     HEAP_ZERO_MEMORY,
//                                     dwBufSize);
//     if (ControlPacket == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         bRet = FALSE;
//         goto done;
//     }
//
//     ScCreateStatusBinding();
//
//     /* Call the dispatcher loop */
//     ScServiceDispatcher(hPipe, ControlPacket, dwBufSize);
//
//
//     ScDestroyStatusBinding();
//
//     /* Close the connection */
//     CloseHandle(hPipe);
//
//     /* Free the control packet */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ControlPacket);
//
// done:
//     /* Free the service table */
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         RtlFreeUnicodeString(&lpActiveServices[i].ServiceName);
//     }
//     RtlFreeHeap(RtlGetProcessHeap(), 0, lpActiveServices);
//     lpActiveServices = NULL;
//     dwActiveServiceCount = 0;
//
//     if (!bRet)
//         SetLastError(dwError);
//
//     return bRet;
// }
//
//
// /**********************************************************************
//  *  StartServiceCtrlDispatcherW
//  *
//  * @implemented
//  */
// BOOL WINAPI
// StartServiceCtrlDispatcherW(const SERVICE_TABLE_ENTRYW *lpServiceStartTable)
// {
//     ULONG i;
//     HANDLE hPipe;
//     DWORD dwError;
//     PSCM_CONTROL_PACKET ControlPacket;
//     DWORD dwBufSize;
//     BOOL bRet = TRUE;
//
//     TRACE("StartServiceCtrlDispatcherW(%p)\n",
//           lpServiceStartTable);
//
//     i = 0;
//     while (lpServiceStartTable[i].lpServiceProc != NULL)
//     {
//         i++;
//     }
//
//     dwActiveServiceCount = i;
//
//     /* Allocate the service table */
//     lpActiveServices = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        HEAP_ZERO_MEMORY,
//                                        dwActiveServiceCount * sizeof(ACTIVE_SERVICE));
//     if (lpActiveServices == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     /* Copy service names and start procedure */
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         RtlCreateUnicodeString(&lpActiveServices[i].ServiceName,
//                                lpServiceStartTable[i].lpServiceName);
//         lpActiveServices[i].ServiceMain.W = lpServiceStartTable[i].lpServiceProc;
//         lpActiveServices[i].hServiceStatus = NULL;
//         lpActiveServices[i].bUnicode = TRUE;
//         lpActiveServices[i].bOwnProcess = FALSE;
//     }
//
//     /* Connect to the SCM */
//     dwError = ScConnectControlPipe(&hPipe);
//     if (dwError != ERROR_SUCCESS)
//     {
//         bRet = FALSE;
//         goto done;
//     }
//
//     dwBufSize = sizeof(SCM_CONTROL_PACKET) +
//                 (MAX_SERVICE_NAME_LENGTH + 1) * sizeof(WCHAR);
//
//     ControlPacket = RtlAllocateHeap(RtlGetProcessHeap(),
//                                     HEAP_ZERO_MEMORY,
//                                     dwBufSize);
//     if (ControlPacket == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         bRet = FALSE;
//         goto done;
//     }
//
//     ScCreateStatusBinding();
//
//     /* Call the dispatcher loop */
//     ScServiceDispatcher(hPipe, ControlPacket, dwBufSize);
//
//     ScDestroyStatusBinding();
//
//     /* Close the connection */
//     CloseHandle(hPipe);
//
//     /* Free the control packet */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ControlPacket);
//
// done:
//     /* Free the service table */
//     for (i = 0; i < dwActiveServiceCount; i++)
//     {
//         RtlFreeUnicodeString(&lpActiveServices[i].ServiceName);
//     }
//     RtlFreeHeap(RtlGetProcessHeap(), 0, lpActiveServices);
//     lpActiveServices = NULL;
//     dwActiveServiceCount = 0;
//
//     if (!bRet)
//         SetLastError(dwError);
//
//     return bRet;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
