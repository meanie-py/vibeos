use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::handle::CloseHandle;
use crate::prelude::*;
use core::ffi::c_void;

use crate::string::nls_subst::NT_SUCCESS;
use crate::k32_externs::{BaseSetLastNTError, RtlGetProcessHeap, SetLastError, GetLastError, UlongToHandle, InitializeObjectAttributes};

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// SID_IDENTIFIER_AUTHORITY initializers (setypes.h).
const SECURITY_NT_AUTHORITY: [BYTE; 6] = [0, 0, 0, 0, 0, 5];
const SECURITY_WORLD_SID_AUTHORITY: [BYTE; 6] = [0, 0, 0, 0, 0, 1];

// MUTANT_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED|SYNCHRONIZE|MUTANT_QUERY_STATE.
const STANDARD_RIGHTS_REQUIRED: u32 = 0x000F_0000;
const MUTANT_ALL_ACCESS: u32 = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | MUTANT_QUERY_STATE;

// ntstatus.h codes not bindgen-evaluable.
const STATUS_USER_APC: NTSTATUS = 0x0000_00C0;
const DBG_PRINTEXCEPTION_C: DWORD = 0x4001_0006;
const DBG_UNABLE_TO_PROVIDE_HANDLE: NTSTATUS = 0x4001_0002;

// Local mirror of the anonymous DBWIN shared-buffer struct in debugger.c.
#[repr(C)]
struct DBWIN_BUFFER {
    ProcessId: DWORD,
    Buffer: [i8; 1],
}

// DbgSsReserved[0] holds the head of the per-thread debug data list (k32 macros).
#[inline]
unsafe fn DbgSsGetThreadData() -> PDBGSS_THREAD_DATA {
    (*NtCurrentTeb()).DbgSsReserved[0] as PDBGSS_THREAD_DATA
}
#[inline]
unsafe fn DbgSsSetThreadData(d: PDBGSS_THREAD_DATA) {
    (*NtCurrentTeb()).DbgSsReserved[0] = d as PVOID;
}

extern "system" {
    fn OpenMutexW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn CreateMutexW(lpAttrs: LPSECURITY_ATTRIBUTES, bInitialOwner: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HANDLE;
    fn GlobalFree(hMem: HANDLE) -> HANDLE;
    fn GetProcessHeap() -> HANDLE;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn OpenFileMappingW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn MapViewOfFile(hFileMappingObject: HANDLE, dwDesiredAccess: DWORD, dwFileOffsetHigh: DWORD, dwFileOffsetLow: DWORD, dwNumberOfBytesToMap: SIZE_T) -> LPVOID;
    fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> BOOL;
    fn OpenEventW(dwDesiredAccess: DWORD, bInheritHandle: BOOL, lpName: LPCWSTR) -> HANDLE;
    fn SetEvent(hEvent: HANDLE) -> BOOL;
    fn GetCurrentProcessId() -> DWORD;
    fn RaiseException(dwExceptionCode: DWORD, dwExceptionFlags: DWORD, nNumberOfArguments: DWORD, lpArguments: *const ULONG_PTR);
    fn BaseFormatTimeOut(Timeout: PLARGE_INTEGER, dwMilliseconds: DWORD) -> PLARGE_INTEGER;
}

// HeapAlloc/HeapFree are pure ntdll forwarders in kernel32.spec (no link-time
// local symbol) — route to the crate's thin Rtl* wrappers instead of an extern.
use crate::string::nls_subst::{HeapAlloc, HeapFree};

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _DBGSS_THREAD_DATA {
    pub Next: *mut _DBGSS_THREAD_DATA,
    pub ThreadHandle: HANDLE,
    pub ProcessHandle: HANDLE,
    pub ProcessId: DWORD,
    pub ThreadId: DWORD,
    pub HandleMarked: BOOLEAN,
}
pub type DBGSS_THREAD_DATA = _DBGSS_THREAD_DATA;
pub type PDBGSS_THREAD_DATA = *mut _DBGSS_THREAD_DATA;
unsafe fn K32CreateDBMonMutex() -> HANDLE {
    static siaNTAuth: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    static siaWorldAuth: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_WORLD_SID_AUTHORITY,
    };
    let mut hMutex: HANDLE;
    let mut Status: NTSTATUS;

    let mut psidSystem: PSID = core::ptr::null_mut();
    let mut psidAdministrators: PSID = core::ptr::null_mut();
    let mut psidEveryone: PSID = core::ptr::null_mut();

    let mut pDaclBuf: PVOID = core::ptr::null_mut();

    let mut nDaclBufSize: SIZE_T =
        (core::mem::size_of::<ACL>() + 3 * core::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart)) as SIZE_T;

    let mut sdMutexSecurity: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut saMutexAttribs: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: core::ptr::addr_of_mut!(sdMutexSecurity) as *mut c_void,
        bInheritHandle: 1,
    };

    hMutex = OpenMutexW(
        SYNCHRONIZE | READ_CONTROL | MUTANT_QUERY_STATE,
        1,
        w!("DBWinMutex\0").as_ptr() as LPCWSTR,
    );
    if hMutex != core::ptr::null_mut() {
        return hMutex;
    } else if GetLastError() != ERROR_FILE_NOT_FOUND {
        return core::ptr::null_mut();
    }

    'cleanup: {
    Status = RtlAllocateAndInitializeSid(
        core::ptr::addr_of!(siaNTAuth) as PSID_IDENTIFIER_AUTHORITY,
        1,
        SECURITY_LOCAL_SYSTEM_RID,
        0, 0, 0, 0, 0, 0, 0,
        core::ptr::addr_of_mut!(psidSystem),
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAllocateAndInitializeSid(
        core::ptr::addr_of!(siaNTAuth) as PSID_IDENTIFIER_AUTHORITY,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0, 0, 0, 0, 0, 0,
        core::ptr::addr_of_mut!(psidAdministrators),
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAllocateAndInitializeSid(
        core::ptr::addr_of!(siaWorldAuth) as PSID_IDENTIFIER_AUTHORITY,
        1,
        SECURITY_WORLD_RID,
        0, 0, 0, 0, 0, 0, 0,
        core::ptr::addr_of_mut!(psidEveryone),
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    nDaclBufSize += RtlLengthSid(psidSystem) as SIZE_T;
    nDaclBufSize += RtlLengthSid(psidAdministrators) as SIZE_T;
    nDaclBufSize += RtlLengthSid(psidEveryone) as SIZE_T;

    pDaclBuf = GlobalAlloc(GMEM_FIXED, nDaclBufSize);
    if pDaclBuf == core::ptr::null_mut() {
        break 'cleanup;
    }

    Status = RtlCreateAcl(pDaclBuf as PACL, nDaclBufSize as u32, ACL_REVISION);
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAddAccessAllowedAce(
        pDaclBuf as PACL,
        ACL_REVISION,
        SYNCHRONIZE | READ_CONTROL | MUTANT_QUERY_STATE,
        psidEveryone,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAddAccessAllowedAce(
        pDaclBuf as PACL,
        ACL_REVISION,
        MUTANT_ALL_ACCESS,
        psidAdministrators,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlAddAccessAllowedAce(
        pDaclBuf as PACL,
        ACL_REVISION,
        MUTANT_ALL_ACCESS,
        psidSystem,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlCreateSecurityDescriptor(
        core::ptr::addr_of_mut!(sdMutexSecurity) as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    Status = RtlSetDaclSecurityDescriptor(
        core::ptr::addr_of_mut!(sdMutexSecurity) as PSECURITY_DESCRIPTOR,
        1,
        pDaclBuf as PACL,
        0,
    );
    if !NT_SUCCESS(Status) {
        break 'cleanup;
    }

    hMutex = CreateMutexW(
        core::ptr::addr_of_mut!(saMutexAttribs),
        0,
        w!("DBWinMutex\0").as_ptr() as LPCWSTR,
    );
    } // 'cleanup

    // Cleanup:
    if !pDaclBuf.is_null() {
        GlobalFree(pDaclBuf);
    }
    if !psidEveryone.is_null() {
        RtlFreeSid(psidEveryone);
    }
    if !psidAdministrators.is_null() {
        RtlFreeSid(psidAdministrators);
    }
    if !psidSystem.is_null() {
        RtlFreeSid(psidSystem);
    }

    return hMutex;
}

#[no_mangle]
pub unsafe extern "system" fn SaveThreadHandle(
    dwProcessId: DWORD,
    dwThreadId: DWORD,
    hThread: HANDLE,
) {
    let ThreadData: PDBGSS_THREAD_DATA;

    /* Allocate a thread structure */
    ThreadData = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<DBGSS_THREAD_DATA>() as SIZE_T,
    ) as PDBGSS_THREAD_DATA;
    if ThreadData.is_null() {
        return;
    }

    /* Fill it out */
    (*ThreadData).ThreadHandle = hThread;
    (*ThreadData).ProcessId = dwProcessId;
    (*ThreadData).ThreadId = dwThreadId;
    (*ThreadData).ProcessHandle = core::ptr::null_mut();
    (*ThreadData).HandleMarked = 0;

    /* Link it */
    (*ThreadData).Next = DbgSsGetThreadData();
    DbgSsSetThreadData(ThreadData);
}

#[no_mangle]
pub unsafe extern "system" fn SaveProcessHandle(
    dwProcessId: DWORD,
    hProcess: HANDLE,
) {
    let ThreadData: PDBGSS_THREAD_DATA = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<DBGSS_THREAD_DATA>() as SIZE_T,
    ) as PDBGSS_THREAD_DATA;
    if ThreadData.is_null() {
        return;
    }

    (*ThreadData).ProcessHandle = hProcess;
    (*ThreadData).ProcessId = dwProcessId;
    (*ThreadData).ThreadId = 0;
    (*ThreadData).ThreadHandle = core::ptr::null_mut();
    (*ThreadData).HandleMarked = 0;

    (*ThreadData).Next = DbgSsGetThreadData();
    DbgSsSetThreadData(ThreadData);
}

#[no_mangle]
pub unsafe extern "system" fn MarkThreadHandle(dwThreadId: DWORD) {
    let mut ThreadData: PDBGSS_THREAD_DATA;

    ThreadData = DbgSsGetThreadData();
    while !ThreadData.is_null() {
        if (*ThreadData).ThreadId == dwThreadId {
            (*ThreadData).HandleMarked = 1; // TRUE
            break;
        }
        ThreadData = (*ThreadData).Next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MarkProcessHandle(dwProcessId: DWORD) {
    let mut ThreadData: PDBGSS_THREAD_DATA;

    ThreadData = DbgSsGetThreadData();
    while !ThreadData.is_null() {
        if (ThreadData.as_ref().unwrap().ProcessId == dwProcessId) && (ThreadData.as_ref().unwrap().ThreadId == 0) {
            ThreadData.as_mut().unwrap().HandleMarked = 1;
            break;
        }
        ThreadData = ThreadData.as_ref().unwrap().Next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RemoveHandles(
    dwProcessId: DWORD,
    dwThreadId: DWORD,
) {
    let mut ThreadData: *mut PDBGSS_THREAD_DATA;
    let mut ThisData: PDBGSS_THREAD_DATA;

    /* Loop all thread data events */
    ThreadData = &mut (*NtCurrentTeb()).DbgSsReserved as *mut _ as *mut PDBGSS_THREAD_DATA;
    ThisData = *ThreadData;
    while ThisData != 0 as *mut _ {
        /* Check if this one matches */
        if ((*ThisData).HandleMarked != 0)
            && (((*ThisData).ProcessId == dwProcessId) || ((*ThisData).ThreadId == dwThreadId))
        {
            /* Close open handles */
            if (*ThisData).ThreadHandle != 0 as *mut _ {
                CloseHandle((*ThisData).ThreadHandle);
            }
            if (*ThisData).ProcessHandle != 0 as *mut _ {
                CloseHandle((*ThisData).ProcessHandle);
            }

            /* Unlink the thread data */
            *ThreadData = (*ThisData).Next;

            /* Free it*/
            RtlFreeHeap(RtlGetProcessHeap(), 0, ThisData as *mut core::ffi::c_void);
        } else {
            /* Move to the next one */
            ThreadData = &mut (*ThisData).Next as *mut _;
        }
        ThisData = *ThreadData;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CloseAllProcessHandles(dwProcessId: DWORD) {

    let mut ThreadData: *mut PDBGSS_THREAD_DATA;
    let mut ThisData: PDBGSS_THREAD_DATA;

    /* Loop all thread data events */
    ThreadData = (*NtCurrentTeb()).DbgSsReserved.as_mut_ptr() as *mut PDBGSS_THREAD_DATA;
    ThisData = *ThreadData;
    while ThisData != 0 as *mut _ {
        /* Check if this one matches */
        if (*ThisData).ProcessId == dwProcessId {
            /* Close open handles */
            if (*ThisData).ThreadHandle != 0 as *mut _ {
                CloseHandle((*ThisData).ThreadHandle);
            }
            if (*ThisData).ProcessHandle != 0 as *mut _ {
                CloseHandle((*ThisData).ProcessHandle);
            }

            /* Unlink the thread data */
            *ThreadData = (*ThisData).Next;

            /* Free it*/
            RtlFreeHeap(RtlGetProcessHeap(), 0, ThisData as *mut core::ffi::c_void);
        } else {
            /* Move to the next one */
            ThreadData = &mut (*ThisData).Next as *mut *mut _ as *mut PDBGSS_THREAD_DATA;
        }
        ThisData = *ThreadData;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ProcessIdToHandle(dwProcessId: DWORD) -> HANDLE {
    let Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();

    /* If we don't have a PID, look it up */
    let mut dwProcessId = dwProcessId;
    if dwProcessId == MAXDWORD {
        dwProcessId = CsrGetProcessId() as DWORD_PTR as DWORD;
    }

    /* Open a handle to the process */
    ClientId.UniqueThread = core::ptr::null_mut();
    ClientId.UniqueProcess = UlongToHandle(dwProcessId);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenProcess(
        &mut Handle,
        PROCESS_CREATE_THREAD
            | PROCESS_VM_OPERATION
            | PROCESS_VM_WRITE
            | PROCESS_VM_READ
            | PROCESS_SUSPEND_RESUME
            | PROCESS_QUERY_INFORMATION,
        &mut ObjectAttributes,
        &mut ClientId,
    );
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return 0 as HANDLE;
    }

    /* Return the handle */
    return Handle;
}

#[no_mangle]
pub unsafe extern "system" fn CheckRemoteDebuggerPresent(
    hProcess: HANDLE,
    pbDebuggerPresent: PBOOL,
) -> BOOL {
    let mut DebugPort: HANDLE = core::ptr::null_mut();
    let Status: NTSTATUS;

    /* Make sure we have an output and process*/
    if pbDebuggerPresent.is_null() || hProcess.is_null() {
        /* Fail */
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Check if the process has a debug object/port */
    Status = NtQueryInformationProcess(
        hProcess,
        ProcessDebugPort,
        &mut DebugPort as *mut HANDLE as *mut core::ffi::c_void,
        core::mem::size_of::<HANDLE>() as ULONG,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        /* Return the current state */
        *pbDebuggerPresent = if !DebugPort.is_null() { 1 } else { 0 };
        return 1;
    }

    /* Otherwise, fail */
    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ContinueDebugEvent(
    dwProcessId: DWORD,
    dwThreadId: DWORD,
    dwContinueStatus: DWORD,
) -> BOOL {
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();
    let Status: NTSTATUS;

    /* Set the Client ID */
    ClientId.UniqueProcess = UlongToHandle(dwProcessId);
    ClientId.UniqueThread = UlongToHandle(dwThreadId);

    /* Continue debugging */
    Status = DbgUiContinue(&mut ClientId, dwContinueStatus as NTSTATUS);
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return 0; /* FALSE */
    }

    /* Remove the process/thread handles */
    RemoveHandles(dwProcessId, dwThreadId);

    /* Success */
    return 1; /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn DebugActiveProcess(dwProcessId: DWORD) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Status1: NTSTATUS;
    let mut Handle: HANDLE;

    /* Connect to the debugger */
    Status = DbgUiConnectToDbg();
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0; /* FALSE */
    }

    /* Get the process handle */
    Handle = ProcessIdToHandle(dwProcessId);
    if Handle.is_null() {
        return 0; /* FALSE */
    }

    /* Now debug the process */
    Status = DbgUiDebugActiveProcess(Handle);

    /* Close the handle since we're done */
    Status1 = NtClose(Handle);
    // ASSERT(NT_SUCCESS(Status1)); /* NDEBUG defined, so assert is empty */

    /* Check if debugging worked */
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return 0; /* FALSE */
    }

    /* Success */
    return 1; /* TRUE */
}
#[no_mangle]
pub unsafe extern "system" fn DebugActiveProcessStop(dwProcessId: DWORD) -> BOOL {
    let Status: NTSTATUS;
    let Status1: NTSTATUS;
    let Handle: HANDLE;

    /* Get the process handle */
    Handle = ProcessIdToHandle(dwProcessId);
    if Handle.is_null() {
        return 0i32;
    }

    /* Close all the process handles */
    CloseAllProcessHandles(dwProcessId);

    /* Now stop debugging the process */
    Status = DbgUiStopDebugging(Handle);
    Status1 = NtClose(Handle);
    debug_assert!(NT_SUCCESS(Status1));

    /* Check for failure */
    if !NT_SUCCESS(Status) {
        /* Fail */
        SetLastError(ERROR_ACCESS_DENIED);
        return 0i32;
    }

    /* Success */
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn DebugBreakProcess(Process: HANDLE) -> BOOL {
    let Status: NTSTATUS;

    Status = DbgUiIssueRemoteBreakin(Process);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DebugSetProcessKillOnExit(KillOnExit: BOOL) -> BOOL {
    let Handle: HANDLE;
    let Status: NTSTATUS;
    let mut State: ULONG;

    /* Get the debug object */
    Handle = DbgUiGetThreadDebugObject();
    if Handle.is_null() {
        /* Fail */
        BaseSetLastNTError(STATUS_INVALID_HANDLE);
        return 0;
    }

    /* Now set the kill-on-exit state */
    State = if KillOnExit != 0 { 1 } else { 0 };
    Status = NtSetInformationDebugObject(
        Handle,
        DebugObjectKillProcessOnExitInformation,
        &mut State as *mut ULONG as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return 0;
    }

    /* Success */
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn IsDebuggerPresent() -> BOOL {
    (*NtCurrentPeb()).BeingDebugged as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WaitForDebugEvent(
    lpDebugEvent: LPDEBUG_EVENT,
    dwMilliseconds: DWORD,
) -> BOOL {
    let mut WaitTime: LARGE_INTEGER = core::mem::zeroed();
    let Timeout: PLARGE_INTEGER;
    let mut WaitStateChange: DBGUI_WAIT_STATE_CHANGE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    /* Convert to NT Timeout */
    Timeout = BaseFormatTimeOut(&mut WaitTime, dwMilliseconds);

    /* Loop while we keep getting interrupted */
    loop {
        /* Call the native API */
        Status = DbgUiWaitStateChange(&mut WaitStateChange, Timeout);
        if !((Status == STATUS_ALERTED) || (Status == STATUS_USER_APC)) {
            break;
        }
    }

    /* Check if the wait failed */
    if !(NT_SUCCESS(Status)) || (Status == DBG_UNABLE_TO_PROVIDE_HANDLE) {
        /* Set the error code and quit */
        BaseSetLastNTError(Status);
        return 0i32;
    }

    /* Check if we timed out */
    if Status == STATUS_TIMEOUT {
        /* Fail with a timeout error */
        SetLastError(ERROR_SEM_TIMEOUT);
        return 0i32;
    }

    /* Convert the structure */
    Status = DbgUiConvertStateChangeStructure(&mut WaitStateChange, lpDebugEvent as *mut c_void);
    if !NT_SUCCESS(Status) {
        /* Set the error code and quit */
        BaseSetLastNTError(Status);
        return 0i32;
    }

    /* Check what kind of event this was */
    match (*lpDebugEvent).dwDebugEventCode {
        /* New thread was created */
        CREATE_THREAD_DEBUG_EVENT => {
            /* Setup the thread data */
            SaveThreadHandle(
                (*lpDebugEvent).dwProcessId,
                (*lpDebugEvent).dwThreadId,
                (*lpDebugEvent).u.CreateThread.hThread,
            );
        }

        /* New process was created */
        CREATE_PROCESS_DEBUG_EVENT => {
            /* Setup the process data */
            SaveProcessHandle(
                (*lpDebugEvent).dwProcessId,
                (*lpDebugEvent).u.CreateProcessInfo.hProcess,
            );

            /* Setup the thread data */
            SaveThreadHandle(
                (*lpDebugEvent).dwProcessId,
                (*lpDebugEvent).dwThreadId,
                (*lpDebugEvent).u.CreateProcessInfo.hThread,
            );
        }

        /* Process was exited */
        EXIT_PROCESS_DEBUG_EVENT => {
            /* Mark the thread data as such and fall through */
            MarkProcessHandle((*lpDebugEvent).dwProcessId);
            /* Fall through to EXIT_THREAD_DEBUG_EVENT */
            MarkThreadHandle((*lpDebugEvent).dwThreadId);
        }

        /* Thread was exited */
        EXIT_THREAD_DEBUG_EVENT => {
            /* Mark the thread data */
            MarkThreadHandle((*lpDebugEvent).dwThreadId);
        }

        /* Nothing to do */
        EXCEPTION_DEBUG_EVENT | LOAD_DLL_DEBUG_EVENT | UNLOAD_DLL_DEBUG_EVENT
        | OUTPUT_DEBUG_STRING_EVENT | RIP_EVENT => {}

        /* Fail anything else */
        _ => return 0i32,
    }

    /* Return success */
    1i32
}

#[no_mangle]
pub unsafe extern "system" fn OutputDebugStringA(_OutputString: LPCSTR) {
    let mut _OutputString = _OutputString;
    if (*NtCurrentPeb()).BeingDebugged == 0 {
        DbgPrint("%s\0".as_ptr() as *const i8, _OutputString);
        return;
    }
    // _SEH2_TRY
    {
        let mut a_nArgs: [ULONG_PTR; 2] = [0; 2];
        a_nArgs[0] = (strlen(_OutputString) + 1) as ULONG_PTR;
        a_nArgs[1] = _OutputString as ULONG_PTR;
        RaiseException(DBG_PRINTEXCEPTION_C, 0, 2, a_nArgs.as_ptr());
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        static mut s_hDBMonMutex: HANDLE = core::ptr::null_mut();
        static mut s_bDBMonMutexTriedOpen: BOOL = 0;
        let mut hDBMonMutex: HANDLE = s_hDBMonMutex;
        let mut hDBMonBuffer: HANDLE = core::ptr::null_mut();
        let mut pDBMonBuffer: *mut DBWIN_BUFFER = core::ptr::null_mut();
        let mut hDBMonBufferReady: HANDLE = core::ptr::null_mut();
        let mut hDBMonDataReady: HANDLE = core::ptr::null_mut();

        if hDBMonMutex.is_null() && s_bDBMonMutexTriedOpen == 0 {
            hDBMonMutex = K32CreateDBMonMutex();
            s_hDBMonMutex = hDBMonMutex;
        }

        // _SEH2_TRY
        {
            let mut a_cBuffer: *mut i8 = core::ptr::null_mut();

            if hDBMonMutex.is_null() {
                s_bDBMonMutexTriedOpen = 1;
            } else {
                loop {
                    WaitForSingleObject(hDBMonMutex, INFINITE);
                    hDBMonBuffer = OpenFileMappingW(
                        SECTION_MAP_WRITE,
                        0,
                        w!("DBWIN_BUFFER\0").as_ptr() as LPCWSTR,
                    );
                    if hDBMonBuffer.is_null() {
                        break;
                    }
                    pDBMonBuffer = MapViewOfFile(
                        hDBMonBuffer,
                        SECTION_MAP_READ | SECTION_MAP_WRITE,
                        0,
                        0,
                        0,
                    ) as *mut DBWIN_BUFFER;
                    if pDBMonBuffer.is_null() {
                        break;
                    }
                    hDBMonBufferReady = OpenEventW(
                        SYNCHRONIZE,
                        0,
                        w!("DBWIN_BUFFER_READY\0").as_ptr() as LPCWSTR,
                    );
                    if hDBMonBufferReady.is_null() {
                        break;
                    }
                    hDBMonDataReady = OpenEventW(
                        EVENT_MODIFY_STATE,
                        0,
                        w!("DBWIN_DATA_READY\0").as_ptr() as LPCWSTR,
                    );
                    break;
                }
                if hDBMonDataReady.is_null() {
                    ReleaseMutex(hDBMonMutex);
                }
            }

            // _SEH2_TRY
            {
                let mut nRoundLen: SIZE_T;
                let mut nOutputStringLen: SIZE_T;
                nOutputStringLen = strlen(_OutputString) as SIZE_T;

                loop {
                    if !hDBMonDataReady.is_null() {
                        if WaitForSingleObject(hDBMonBufferReady, 10000) != 0 {
                            break;
                        }
                        (*pDBMonBuffer).ProcessId = GetCurrentProcessId();
                        if nOutputStringLen > (PAGE_SIZE as SIZE_T - core::mem::size_of::<DWORD>() as SIZE_T - 1) {
                            nRoundLen = PAGE_SIZE as SIZE_T - core::mem::size_of::<DWORD>() as SIZE_T - 1;
                        } else {
                            nRoundLen = nOutputStringLen;
                        }
                        core::ptr::copy_nonoverlapping(
                            _OutputString as *const u8,
                            (*pDBMonBuffer).Buffer.as_mut_ptr() as *mut u8,
                            nRoundLen as usize,
                        );
                        *((*pDBMonBuffer).Buffer.as_mut_ptr().add(nRoundLen as usize)) = 0;
                        SetEvent(hDBMonDataReady);
                    } else {
                        a_cBuffer = HeapAlloc(GetProcessHeap(), 0, 512) as *mut i8;
                        if a_cBuffer.is_null() {
                            DbgPrint(
                                "OutputDebugStringA: Failed\0".as_ptr() as *const i8,
                            );
                            break;
                        }
                        if nOutputStringLen > 510 {
                            nRoundLen = 510;
                        } else {
                            nRoundLen = nOutputStringLen;
                        }
                        core::ptr::copy_nonoverlapping(
                            _OutputString as *const u8,
                            a_cBuffer as *mut u8,
                            nRoundLen as usize,
                        );
                        *a_cBuffer.add(nRoundLen as usize) = 0;
                        DbgPrint("%s\0".as_ptr() as *const i8, a_cBuffer);
                        if !a_cBuffer.is_null() {
                            HeapFree(GetProcessHeap(), 0, a_cBuffer as *mut core::ffi::c_void);
                            a_cBuffer = core::ptr::null_mut();
                        }
                    }
                    _OutputString = _OutputString.add(nRoundLen as usize);
                    nOutputStringLen -= nRoundLen;
                    if nOutputStringLen == 0 {
                        break;
                    }
                }
            }
            // _SEH2_EXCEPT
            {
                if !a_cBuffer.is_null() {
                    HeapFree(GetProcessHeap(), 0, a_cBuffer as *mut core::ffi::c_void);
                }
                DbgPrint(
                    "\nOutputDebugString faulted during output\0".as_ptr() as *const i8,
                );
            }
            // _SEH2_END
        }
        // _SEH2_FINALLY
        {
            if !hDBMonBufferReady.is_null() {
                CloseHandle(hDBMonBufferReady);
            }
            if !pDBMonBuffer.is_null() {
                UnmapViewOfFile(pDBMonBuffer as *mut core::ffi::c_void);
            }
            if !hDBMonBuffer.is_null() {
                CloseHandle(hDBMonBuffer);
            }
            if !hDBMonDataReady.is_null() {
                CloseHandle(hDBMonDataReady);
            }
            if !hDBMonDataReady.is_null() {
                ReleaseMutex(hDBMonMutex);
            }
        }
        // _SEH2_END
    }
    // _SEH2_END
}

#[no_mangle]
pub unsafe extern "system" fn OutputDebugStringW(OutputString: LPCWSTR) {
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    /* convert the string in ANSI */
    RtlInitUnicodeString(&mut UnicodeString, OutputString);
    Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &UnicodeString, 1);

    /* OutputDebugStringW always prints something, even if conversion fails */
    if !NT_SUCCESS(Status) {
        AnsiString.Buffer = b"\0".as_ptr() as *mut i8;
    }

    /* Output the converted string */
    OutputDebugStringA(AnsiString.Buffer);

    /* free the converted string */
    if NT_SUCCESS(Status) {
        RtlFreeAnsiString(&mut AnsiString);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/debugger.c
//  * PURPOSE:         Wrappers for the NT Debug Implementation
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
//
// #include <ndk/dbgkfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _DBGSS_THREAD_DATA
// {
//     struct _DBGSS_THREAD_DATA *Next;
//     HANDLE ThreadHandle;
//     HANDLE ProcessHandle;
//     DWORD ProcessId;
//     DWORD ThreadId;
//     BOOLEAN HandleMarked;
// } DBGSS_THREAD_DATA, *PDBGSS_THREAD_DATA;
//
// #define DbgSsSetThreadData(d) \
//     NtCurrentTeb()->DbgSsReserved[0] = d
//
// #define DbgSsGetThreadData() \
//     ((PDBGSS_THREAD_DATA)NtCurrentTeb()->DbgSsReserved[0])
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static HANDLE
// K32CreateDBMonMutex(VOID)
// {
//     static SID_IDENTIFIER_AUTHORITY siaNTAuth = {SECURITY_NT_AUTHORITY};
//     static SID_IDENTIFIER_AUTHORITY siaWorldAuth = {SECURITY_WORLD_SID_AUTHORITY};
//     HANDLE hMutex;
//     NTSTATUS Status;
//
//     /* SIDs to be used in the DACL */
//     PSID psidSystem = NULL;
//     PSID psidAdministrators = NULL;
//     PSID psidEveryone = NULL;
//
//     /* Buffer for the DACL */
//     PVOID pDaclBuf = NULL;
//
//     /* Minimum size of the DACL: an ACL descriptor and three ACCESS_ALLOWED_ACE
//      * headers. We will add the size of SIDs when they are known. */
//     SIZE_T nDaclBufSize =
//         sizeof(ACL) + 3 * FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart);
//
//     /* Security descriptor and attributes of the mutex */
//     SECURITY_DESCRIPTOR sdMutexSecurity;
//     SECURITY_ATTRIBUTES saMutexAttribs = {sizeof(saMutexAttribs),
//                                           &sdMutexSecurity,
//                                           TRUE};
//
//     /* Try to open the mutex */
//     hMutex = OpenMutexW(SYNCHRONIZE | READ_CONTROL | MUTANT_QUERY_STATE,
//                         TRUE,
//                         L"DBWinMutex");
//     if (hMutex != NULL)
//     {
//         /* Success */
//         return hMutex;
//     }
//     /* Error other than the mutex not being found */
//     else if (GetLastError() != ERROR_FILE_NOT_FOUND)
//     {
//         /* Failure */
//         return NULL;
//     }
//
//     /* If the mutex does not exist, set up its security, then create it */
//
//     /* Allocate the NT AUTHORITY\SYSTEM SID */
//     Status = RtlAllocateAndInitializeSid(&siaNTAuth,
//                                          1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &psidSystem);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Allocate the BUILTIN\Administrators SID */
//     Status = RtlAllocateAndInitializeSid(&siaNTAuth,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0, 0, 0, 0, 0, 0,
//                                          &psidAdministrators);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Allocate the Everyone SID */
//     Status = RtlAllocateAndInitializeSid(&siaWorldAuth,
//                                          1,
//                                          SECURITY_WORLD_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &psidEveryone);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Allocate space for the SIDs too */
//     nDaclBufSize += RtlLengthSid(psidSystem);
//     nDaclBufSize += RtlLengthSid(psidAdministrators);
//     nDaclBufSize += RtlLengthSid(psidEveryone);
//
//     /* Allocate the buffer for the DACL */
//     pDaclBuf = GlobalAlloc(GMEM_FIXED, nDaclBufSize);
//     if (pDaclBuf == NULL)
//         goto Cleanup;
//
//     /* Create the DACL */
//     Status = RtlCreateAcl(pDaclBuf, nDaclBufSize, ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Grant the minimum required access to Everyone */
//     Status = RtlAddAccessAllowedAce(pDaclBuf,
//                                     ACL_REVISION,
//                                     SYNCHRONIZE |
//                                     READ_CONTROL |
//                                     MUTANT_QUERY_STATE,
//                                     psidEveryone);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Grant full access to BUILTIN\Administrators */
//     Status = RtlAddAccessAllowedAce(pDaclBuf,
//                                     ACL_REVISION,
//                                     MUTANT_ALL_ACCESS,
//                                     psidAdministrators);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Grant full access to NT AUTHORITY\SYSTEM */
//     Status = RtlAddAccessAllowedAce(pDaclBuf,
//                                     ACL_REVISION,
//                                     MUTANT_ALL_ACCESS,
//                                     psidSystem);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Create the security descriptor */
//     Status = RtlCreateSecurityDescriptor(&sdMutexSecurity,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Set the descriptor's DACL to the created ACL */
//     Status = RtlSetDaclSecurityDescriptor(&sdMutexSecurity,
//                                           TRUE,
//                                           pDaclBuf,
//                                           FALSE);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Create the mutex */
//     hMutex = CreateMutexW(&saMutexAttribs, FALSE, L"DBWinMutex");
//
// Cleanup:
//     /* Free the buffers */
//     if (pDaclBuf) GlobalFree(pDaclBuf);
//     if (psidEveryone) RtlFreeSid(psidEveryone);
//     if (psidAdministrators) RtlFreeSid(psidAdministrators);
//     if (psidSystem) RtlFreeSid(psidSystem);
//
//     return hMutex;
// }
//
// VOID
// WINAPI
// SaveThreadHandle(IN DWORD dwProcessId,
//                  IN DWORD dwThreadId,
//                  IN HANDLE hThread)
// {
//     PDBGSS_THREAD_DATA ThreadData;
//
//     /* Allocate a thread structure */
//     ThreadData = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  0,
//                                  sizeof(DBGSS_THREAD_DATA));
//     if (!ThreadData) return;
//
//     /* Fill it out */
//     ThreadData->ThreadHandle = hThread;
//     ThreadData->ProcessId = dwProcessId;
//     ThreadData->ThreadId = dwThreadId;
//     ThreadData->ProcessHandle = NULL;
//     ThreadData->HandleMarked = FALSE;
//
//     /* Link it */
//     ThreadData->Next = DbgSsGetThreadData();
//     DbgSsSetThreadData(ThreadData);
// }
//
// VOID
// WINAPI
// SaveProcessHandle(IN DWORD dwProcessId,
//                   IN HANDLE hProcess)
// {
//     PDBGSS_THREAD_DATA ThreadData;
//
//     /* Allocate a thread structure */
//     ThreadData = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  0,
//                                  sizeof(DBGSS_THREAD_DATA));
//     if (!ThreadData) return;
//
//     /* Fill it out */
//     ThreadData->ProcessHandle = hProcess;
//     ThreadData->ProcessId = dwProcessId;
//     ThreadData->ThreadId = 0;
//     ThreadData->ThreadHandle = NULL;
//     ThreadData->HandleMarked = FALSE;
//
//     /* Link it */
//     ThreadData->Next = DbgSsGetThreadData();
//     DbgSsSetThreadData(ThreadData);
// }
//
// VOID
// WINAPI
// MarkThreadHandle(IN DWORD dwThreadId)
// {
//     PDBGSS_THREAD_DATA ThreadData;
//
//     /* Loop all thread data events */
//     for (ThreadData = DbgSsGetThreadData(); ThreadData; ThreadData = ThreadData->Next)
//     {
//         /* Check if this one matches */
//         if (ThreadData->ThreadId == dwThreadId)
//         {
//             /* Mark the structure and break out */
//             ThreadData->HandleMarked = TRUE;
//             break;
//         }
//     }
// }
//
// VOID
// WINAPI
// MarkProcessHandle(IN DWORD dwProcessId)
// {
//     PDBGSS_THREAD_DATA ThreadData;
//
//     /* Loop all thread data events */
//     for (ThreadData = DbgSsGetThreadData(); ThreadData; ThreadData = ThreadData->Next)
//     {
//         /* Check if this one matches */
//         if ((ThreadData->ProcessId == dwProcessId) && !(ThreadData->ThreadId))
//         {
//             /* Mark the structure and break out */
//             ThreadData->HandleMarked = TRUE;
//             break;
//         }
//     }
// }
//
// VOID
// WINAPI
// RemoveHandles(IN DWORD dwProcessId,
//               IN DWORD dwThreadId)
// {
//     PDBGSS_THREAD_DATA *ThreadData;
//     PDBGSS_THREAD_DATA ThisData;
//
//     /* Loop all thread data events */
//     ThreadData = (PDBGSS_THREAD_DATA*)NtCurrentTeb()->DbgSsReserved;
//     ThisData = *ThreadData;
//     while(ThisData)
//     {
//         /* Check if this one matches */
//         if ((ThisData->HandleMarked) &&
//             ((ThisData->ProcessId == dwProcessId) || (ThisData->ThreadId == dwThreadId)))
//         {
//             /* Close open handles */
//             if (ThisData->ThreadHandle) CloseHandle(ThisData->ThreadHandle);
//             if (ThisData->ProcessHandle) CloseHandle(ThisData->ProcessHandle);
//
//             /* Unlink the thread data */
//             *ThreadData = ThisData->Next;
//
//             /* Free it*/
//             RtlFreeHeap(RtlGetProcessHeap(), 0, ThisData);
//         }
//         else
//         {
//             /* Move to the next one */
//             ThreadData = &ThisData->Next;
//         }
//         ThisData = *ThreadData;
//     }
// }
//
// VOID
// WINAPI
// CloseAllProcessHandles(IN DWORD dwProcessId)
// {
//     PDBGSS_THREAD_DATA *ThreadData;
//     PDBGSS_THREAD_DATA ThisData;
//
//     /* Loop all thread data events */
//     ThreadData = (PDBGSS_THREAD_DATA*)NtCurrentTeb()->DbgSsReserved;
//     ThisData = *ThreadData;
//     while(ThisData)
//     {
//         /* Check if this one matches */
//         if (ThisData->ProcessId == dwProcessId)
//         {
//             /* Close open handles */
//             if (ThisData->ThreadHandle) CloseHandle(ThisData->ThreadHandle);
//             if (ThisData->ProcessHandle) CloseHandle(ThisData->ProcessHandle);
//
//             /* Unlink the thread data */
//             *ThreadData = ThisData->Next;
//
//             /* Free it*/
//             RtlFreeHeap(RtlGetProcessHeap(), 0, ThisData);
//         }
//         else
//         {
//             /* Move to the next one */
//             ThreadData = &ThisData->Next;
//         }
//         ThisData = *ThreadData;
//     }
// }
//
// HANDLE
// WINAPI
// ProcessIdToHandle(IN DWORD dwProcessId)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     CLIENT_ID ClientId;
//
//     /* If we don't have a PID, look it up */
//     if (dwProcessId == MAXDWORD) dwProcessId = (DWORD_PTR)CsrGetProcessId();
//
//     /* Open a handle to the process */
//     ClientId.UniqueThread = NULL;
//     ClientId.UniqueProcess = UlongToHandle(dwProcessId);
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     Status = NtOpenProcess(&Handle,
//                            PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION |
//                            PROCESS_VM_WRITE | PROCESS_VM_READ |
//                            PROCESS_SUSPEND_RESUME | PROCESS_QUERY_INFORMATION,
//                            &ObjectAttributes,
//                            &ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     /* Return the handle */
//     return Handle;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CheckRemoteDebuggerPresent(IN HANDLE hProcess,
//                            OUT PBOOL pbDebuggerPresent)
// {
//     HANDLE DebugPort;
//     NTSTATUS Status;
//
//     /* Make sure we have an output and process*/
//     if (!(pbDebuggerPresent) || !(hProcess))
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Check if the process has a debug object/port */
//     Status = NtQueryInformationProcess(hProcess,
//                                        ProcessDebugPort,
//                                        &DebugPort,
//                                        sizeof(DebugPort),
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the current state */
//         *pbDebuggerPresent = DebugPort != NULL;
//         return TRUE;
//     }
//
//     /* Otherwise, fail */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ContinueDebugEvent(IN DWORD dwProcessId,
//                    IN DWORD dwThreadId,
//                    IN DWORD dwContinueStatus)
// {
//     CLIENT_ID ClientId;
//     NTSTATUS Status;
//
//     /* Set the Client ID */
//     ClientId.UniqueProcess = UlongToHandle(dwProcessId);
//     ClientId.UniqueThread = UlongToHandle(dwThreadId);
//
//     /* Continue debugging */
//     Status = DbgUiContinue(&ClientId, dwContinueStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Remove the process/thread handles */
//     RemoveHandles(dwProcessId, dwThreadId);
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DebugActiveProcess(IN DWORD dwProcessId)
// {
//     NTSTATUS Status, Status1;
//     HANDLE Handle;
//
//     /* Connect to the debugger */
//     Status = DbgUiConnectToDbg();
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Get the process handle */
//     Handle = ProcessIdToHandle(dwProcessId);
//     if (!Handle) return FALSE;
//
//     /* Now debug the process */
//     Status = DbgUiDebugActiveProcess(Handle);
//
//     /* Close the handle since we're done */
//     Status1 = NtClose(Handle);
//     ASSERT(NT_SUCCESS(Status1));
//
//     /* Check if debugging worked */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DebugActiveProcessStop(IN DWORD dwProcessId)
// {
//     NTSTATUS Status, Status1;
//     HANDLE Handle;
//
//     /* Get the process handle */
//     Handle = ProcessIdToHandle(dwProcessId);
//     if (!Handle) return FALSE;
//
//     /* Close all the process handles */
//     CloseAllProcessHandles(dwProcessId);
//
//     /* Now stop debugging the process */
//     Status = DbgUiStopDebugging(Handle);
//     Status1 = NtClose(Handle);
//     ASSERT(NT_SUCCESS(Status1));
//
//     /* Check for failure */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         SetLastError(ERROR_ACCESS_DENIED);
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DebugBreakProcess(IN HANDLE Process)
// {
//     NTSTATUS Status;
//
//     /* Send the breakin request */
//     Status = DbgUiIssueRemoteBreakin(Process);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DebugSetProcessKillOnExit(IN BOOL KillOnExit)
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     ULONG State;
//
//     /* Get the debug object */
//     Handle = DbgUiGetThreadDebugObject();
//     if (!Handle)
//     {
//         /* Fail */
//         BaseSetLastNTError(STATUS_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Now set the kill-on-exit state */
//     State = KillOnExit != 0;
//     Status = NtSetInformationDebugObject(Handle,
//                                          DebugObjectKillProcessOnExitInformation,
//                                          &State,
//                                          sizeof(State),
//                                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsDebuggerPresent(VOID)
// {
//     return (BOOL)NtCurrentPeb()->BeingDebugged;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WaitForDebugEvent(IN LPDEBUG_EVENT lpDebugEvent,
//                   IN DWORD dwMilliseconds)
// {
//     LARGE_INTEGER WaitTime;
//     PLARGE_INTEGER Timeout;
//     DBGUI_WAIT_STATE_CHANGE WaitStateChange;
//     NTSTATUS Status;
//
//     /* Convert to NT Timeout */
//     Timeout = BaseFormatTimeOut(&WaitTime, dwMilliseconds);
//
//     /* Loop while we keep getting interrupted */
//     do
//     {
//         /* Call the native API */
//         Status = DbgUiWaitStateChange(&WaitStateChange, Timeout);
//     } while ((Status == STATUS_ALERTED) || (Status == STATUS_USER_APC));
//
//     /* Check if the wait failed */
//     if (!(NT_SUCCESS(Status)) || (Status == DBG_UNABLE_TO_PROVIDE_HANDLE))
//     {
//         /* Set the error code and quit */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Check if we timed out */
//     if (Status == STATUS_TIMEOUT)
//     {
//         /* Fail with a timeout error */
//         SetLastError(ERROR_SEM_TIMEOUT);
//         return FALSE;
//     }
//
//     /* Convert the structure */
//     Status = DbgUiConvertStateChangeStructure(&WaitStateChange, lpDebugEvent);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set the error code and quit */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Check what kind of event this was */
//     switch (lpDebugEvent->dwDebugEventCode)
//     {
//         /* New thread was created */
//         case CREATE_THREAD_DEBUG_EVENT:
//
//             /* Setup the thread data */
//             SaveThreadHandle(lpDebugEvent->dwProcessId,
//                              lpDebugEvent->dwThreadId,
//                              lpDebugEvent->u.CreateThread.hThread);
//             break;
//
//         /* New process was created */
//         case CREATE_PROCESS_DEBUG_EVENT:
//
//             /* Setup the process data */
//             SaveProcessHandle(lpDebugEvent->dwProcessId,
//                               lpDebugEvent->u.CreateProcessInfo.hProcess);
//
//             /* Setup the thread data */
//             SaveThreadHandle(lpDebugEvent->dwProcessId,
//                              lpDebugEvent->dwThreadId,
//                              lpDebugEvent->u.CreateProcessInfo.hThread);
//             break;
//
//         /* Process was exited */
//         case EXIT_PROCESS_DEBUG_EVENT:
//
//             /* Mark the thread data as such and fall through */
//             MarkProcessHandle(lpDebugEvent->dwProcessId);
//
//         /* Thread was exited */
//         case EXIT_THREAD_DEBUG_EVENT:
//
//             /* Mark the thread data */
//             MarkThreadHandle(lpDebugEvent->dwThreadId);
//             break;
//
//         /* Nothing to do */
//         case EXCEPTION_DEBUG_EVENT:
//         case LOAD_DLL_DEBUG_EVENT:
//         case UNLOAD_DLL_DEBUG_EVENT:
//         case OUTPUT_DEBUG_STRING_EVENT:
//         case RIP_EVENT:
//             break;
//
//         /* Fail anything else */
//         default:
//             return FALSE;
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// OutputDebugStringA(IN LPCSTR _OutputString)
// {
//     /* On ARM64 (DUMMY_PSEH) and amd64 (broken unwind tables for SEH around
//      * RaiseException), the _SEH2_EXCEPT below doesn't catch DBG_PRINTEXCEPTION_C,
//      * causing the unhandled exception to reach csrss which kills the process.
//      * Bypass: if no user-mode debugger is attached, go straight to DbgPrint. */
//     if (!NtCurrentPeb()->BeingDebugged)
//     {
//         DbgPrint("%s", _OutputString);
//         return;
//     }
//     _SEH2_TRY
//     {
//         ULONG_PTR a_nArgs[2];
//
//         a_nArgs[0] = (ULONG_PTR)(strlen(_OutputString) + 1);
//         a_nArgs[1] = (ULONG_PTR)_OutputString;
//
//         /* send the string to the user-mode debugger */
//         RaiseException(DBG_PRINTEXCEPTION_C, 0, 2, a_nArgs);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* no user-mode debugger: try the systemwide debug message monitor, or the
//            kernel debugger as a last resort */
//
//         /* mutex used to synchronize invocations of OutputDebugString */
//         static HANDLE s_hDBMonMutex = NULL;
//         /* true if we already attempted to open/create the mutex */
//         static BOOL s_bDBMonMutexTriedOpen = FALSE;
//
//         /* local copy of the mutex handle */
//         volatile HANDLE hDBMonMutex = s_hDBMonMutex;
//         /* handle to the Section of the shared buffer */
//         volatile HANDLE hDBMonBuffer = NULL;
//
//         /* pointer to the mapped view of the shared buffer. It consist of the current
//            process id followed by the message string */
//         struct { DWORD ProcessId; CHAR Buffer[1]; } * pDBMonBuffer = NULL;
//
//         /* event: signaled by the debug message monitor when OutputDebugString can write
//            to the shared buffer */
//         volatile HANDLE hDBMonBufferReady = NULL;
//
//         /* event: to be signaled by OutputDebugString when it's done writing to the
//            shared buffer */
//         volatile HANDLE hDBMonDataReady = NULL;
//
//         /* mutex not opened, and no previous attempts to open/create it */
//         if (hDBMonMutex == NULL && !s_bDBMonMutexTriedOpen)
//         {
//             /* open/create the mutex */
//             hDBMonMutex = K32CreateDBMonMutex();
//             /* store the handle */
//             s_hDBMonMutex = hDBMonMutex;
//         }
//
//         _SEH2_TRY
//         {
//             volatile PCHAR a_cBuffer = NULL;
//
//             /* opening the mutex failed */
//             if (hDBMonMutex == NULL)
//             {
//                 /* remember next time */
//                 s_bDBMonMutexTriedOpen = TRUE;
//             }
//             /* opening the mutex succeeded */
//             else
//             {
//                 do
//                 {
//                     /* synchronize with other invocations of OutputDebugString */
//                     WaitForSingleObject(hDBMonMutex, INFINITE);
//
//                     /* buffer of the system-wide debug message monitor */
//                     hDBMonBuffer = OpenFileMappingW(SECTION_MAP_WRITE, FALSE, L"DBWIN_BUFFER");
//
//                     /* couldn't open the buffer: send the string to the kernel debugger */
//                     if (hDBMonBuffer == NULL) break;
//
//                     /* map the buffer */
//                     pDBMonBuffer = MapViewOfFile(hDBMonBuffer,
//                                                  SECTION_MAP_READ | SECTION_MAP_WRITE,
//                                                  0,
//                                                  0,
//                                                  0);
//
//                     /* couldn't map the buffer: send the string to the kernel debugger */
//                     if (pDBMonBuffer == NULL) break;
//
//                     /* open the event signaling that the buffer can be accessed */
//                     hDBMonBufferReady = OpenEventW(SYNCHRONIZE, FALSE, L"DBWIN_BUFFER_READY");
//
//                     /* couldn't open the event: send the string to the kernel debugger */
//                     if (hDBMonBufferReady == NULL) break;
//
//                     /* open the event to be signaled when the buffer has been filled */
//                     hDBMonDataReady = OpenEventW(EVENT_MODIFY_STATE, FALSE, L"DBWIN_DATA_READY");
//                 }
//                 while(0);
//
//                 /* we couldn't connect to the system-wide debug message monitor: send the
//                    string to the kernel debugger */
//                 if (hDBMonDataReady == NULL) ReleaseMutex(hDBMonMutex);
//             }
//
//             _SEH2_TRY
//             {
//                 /* size of the current output block */
//                 volatile SIZE_T nRoundLen;
//
//                 /* size of the remainder of the string */
//                 volatile SIZE_T nOutputStringLen;
//
//                 /* output the whole string */
//                 nOutputStringLen = strlen(_OutputString) as SIZE_T;
//
//                 do
//                 {
//                     /* we're connected to the debug monitor:
//                        write the current block to the shared buffer */
//                     if (hDBMonDataReady)
//                     {
//                         /* wait a maximum of 10 seconds for the debug monitor
//                            to finish processing the shared buffer */
//                         if (WaitForSingleObject(hDBMonBufferReady, 10000) != WAIT_OBJECT_0)
//                         {
//                             /* timeout or failure: give up */
//                             break;
//                         }
//
//                         /* write the process id into the buffer */
//                         pDBMonBuffer->ProcessId = GetCurrentProcessId();
//
//                         /* write only as many bytes as they fit in the buffer */
//                         if (nOutputStringLen > (PAGE_SIZE - sizeof(DWORD) - 1))
//                             nRoundLen = PAGE_SIZE - sizeof(DWORD) - 1;
//                         else
//                             nRoundLen = nOutputStringLen;
//
//                         /* copy the current block into the buffer */
//                         memcpy(pDBMonBuffer->Buffer, _OutputString, nRoundLen);
//
//                         /* null-terminate the current block */
//                         pDBMonBuffer->Buffer[nRoundLen] = 0;
//
//                         /* signal that the data contains meaningful data and can be read */
//                         SetEvent(hDBMonDataReady);
//                     }
//                     /* else, send the current block to the kernel debugger */
//                     else
//                     {
//                         /* output in blocks of 512 characters */
//                         a_cBuffer = (CHAR*)HeapAlloc(GetProcessHeap(), 0, 512);
//
//                         if (!a_cBuffer)
//                         {
//                             DbgPrint("OutputDebugStringA: Failed\n");
//                             break;
//                         }
//
//                         /* write a maximum of 511 bytes */
//                         if (nOutputStringLen > 510)
//                             nRoundLen = 510;
//                         else
//                             nRoundLen = nOutputStringLen;
//
//                         /* copy the current block */
//                         memcpy(a_cBuffer, _OutputString, nRoundLen);
//
//                         /* null-terminate the current block */
//                         a_cBuffer[nRoundLen] = 0;
//
//                         /* send the current block to the kernel debugger */
//                         DbgPrint("%s", a_cBuffer);
//
//                         if (a_cBuffer)
//                         {
//                             HeapFree(GetProcessHeap(), 0, a_cBuffer);
//                             a_cBuffer = NULL;
//                         }
//                     }
//
//                     /* move to the next block */
//                     _OutputString += nRoundLen;
//                     nOutputStringLen -= nRoundLen;
//                 }
//                 /* repeat until the string has been fully output */
//                 while (nOutputStringLen > 0);
//             }
//             /* ignore access violations and let other exceptions fall through */
//             _SEH2_EXCEPT((_SEH2_GetExceptionCode() == STATUS_ACCESS_VIOLATION) ? EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH)
//             {
//                 if (a_cBuffer)
//                     HeapFree(GetProcessHeap(), 0, a_cBuffer);
//
//                 /* string copied verbatim from Microsoft's kernel32.dll */
//                 DbgPrint("\nOutputDebugString faulted during output\n");
//             }
//             _SEH2_END;
//         }
//         _SEH2_FINALLY
//         {
//             /* close all the still open resources */
//             if (hDBMonBufferReady) CloseHandle(hDBMonBufferReady);
//             if (pDBMonBuffer) UnmapViewOfFile(pDBMonBuffer);
//             if (hDBMonBuffer) CloseHandle(hDBMonBuffer);
//             if (hDBMonDataReady) CloseHandle(hDBMonDataReady);
//
//             /* leave the critical section */
//             if (hDBMonDataReady != NULL)
//                 ReleaseMutex(hDBMonMutex);
//         }
//         _SEH2_END;
//     }
//     _SEH2_END;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// OutputDebugStringW(IN LPCWSTR OutputString)
// {
//     UNICODE_STRING UnicodeString;
//     ANSI_STRING AnsiString;
//     NTSTATUS Status;
//
//     /* convert the string in ANSI */
//     RtlInitUnicodeString(&UnicodeString, OutputString);
//     Status = RtlUnicodeStringToAnsiString(&AnsiString, &UnicodeString, TRUE);
//
//     /* OutputDebugStringW always prints something, even if conversion fails */
//     if (!NT_SUCCESS(Status)) AnsiString.Buffer = "";
//
//     /* Output the converted string */
//     OutputDebugStringA(AnsiString.Buffer);
//
//     /* free the converted string */
//     if (NT_SUCCESS(Status)) RtlFreeAnsiString(&AnsiString);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
