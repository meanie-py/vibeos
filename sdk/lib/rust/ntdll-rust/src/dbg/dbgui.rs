//! Rust translation of `dll/ntdll/dbg/dbgui.c`.
//!
//! Native (`DbgUi*`) wrappers over the NT debug object API. All
//! `@implemented`.
//!
//! Translation notes (vs. the C):
//!   * `NtCurrentTeb`/`NtCurrentPeb` are FORCEINLINE (no exported symbol) —
//!     use the hand-written `ntdll_sys::nt` inlines.
//!   * `InitializeObjectAttributes` and `PtrToUlong` are C macros — inlined.
//!   * `DbgUiConvertStateChangeStructure`'s `switch (NewState)` is rendered as
//!     an `if`/`else if` chain: the bindgen `Dbg*StateChange` values are
//!     `const`s, and naming a bare `const` in a `match` arm binds a fresh
//!     variable (a silent catch-all) rather than comparing — so an explicit
//!     `==` chain is required to preserve the C semantics.
//!   * The output `DEBUG_EVENT` buffer is filled with direct field stores
//!     (C-faithful): unlike the captures(none) victims elsewhere, it is never
//!     passed to a callee here, so the stores cannot be elided. (ud2-scanned.)

use core::mem::{size_of, transmute, zeroed};
use core::ptr::null_mut;
use ntdll_sys::nt::*;

// ntstatus.h debug-exception codes (not in the ntdll-sys bindgen surface).
const DBG_PRINTEXCEPTION_C: u32 = 0x4001_0006;
const DBG_RIPEXCEPTION: u32 = 0x4001_0007;

/// `PtrToUlong` (winnt.h macro): truncate a pointer to ULONG.
#[inline(always)]
unsafe fn ptr_to_ulong(p: *mut core::ffi::c_void) -> ULONG {
    p as usize as ULONG
}

/// C: `NTSTATUS NTAPI DbgUiConnectToDbg(VOID)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiConnectToDbg() -> NTSTATUS {
    let teb = NtCurrentTeb();

    /* Don't connect twice */
    if !(*teb).DbgSsReserved[1].is_null() {
        return STATUS_SUCCESS;
    }

    /* Setup the Attributes — InitializeObjectAttributes(&oa, NULL, 0, NULL, 0) */
    let mut object_attributes: _OBJECT_ATTRIBUTES = zeroed();
    object_attributes.Length = size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    // RootDirectory / ObjectName / Attributes / SecurityDescriptor /
    // SecurityQualityOfService all NULL/0 (already zeroed).

    /* Create the object */
    ZwCreateDebugObject(
        &mut (*teb).DbgSsReserved[1],
        DEBUG_OBJECT_ALL_ACCESS,
        &mut object_attributes,
        DBGK_KILL_PROCESS_ON_EXIT,
    )
}

/// C: `NTSTATUS NTAPI DbgUiContinue(PCLIENT_ID, NTSTATUS)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiContinue(
    ClientId: PCLIENT_ID,
    ContinueStatus: NTSTATUS,
) -> NTSTATUS {
    ZwDebugContinue((*NtCurrentTeb()).DbgSsReserved[1], ClientId, ContinueStatus)
}

/// C: `NTSTATUS NTAPI DbgUiConvertStateChangeStructure(PDBGUI_WAIT_STATE_CHANGE,
/// PVOID Win32DebugEvent)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiConvertStateChangeStructure(
    WaitStateChange: PDBGUI_WAIT_STATE_CHANGE,
    Win32DebugEvent: *mut core::ffi::c_void,
) -> NTSTATUS {
    let DebugEvent = Win32DebugEvent as LPDEBUG_EVENT;

    /* Write common data */
    (*DebugEvent).dwProcessId = ptr_to_ulong((*WaitStateChange).AppClientId.UniqueProcess);
    (*DebugEvent).dwThreadId = ptr_to_ulong((*WaitStateChange).AppClientId.UniqueThread);

    let new_state = (*WaitStateChange).NewState;

    if new_state == DbgCreateThreadStateChange {
        /* New thread */
        (*DebugEvent).dwDebugEventCode = CREATE_THREAD_DEBUG_EVENT;
        (*DebugEvent).u.CreateThread.hThread =
            (*WaitStateChange).StateInfo.CreateThread.HandleToThread;
        // C casts the PVOID StartAddress into the LPTHREAD_START_ROUTINE slot.
        (*DebugEvent).u.CreateThread.lpStartAddress =
            transmute((*WaitStateChange).StateInfo.CreateThread.NewThread.StartAddress);

        let mut thread_basic_info: THREAD_BASIC_INFORMATION = zeroed();
        let status = NtQueryInformationThread(
            (*WaitStateChange).StateInfo.CreateThread.HandleToThread,
            ThreadBasicInformation,
            &mut thread_basic_info as *mut _ as *mut core::ffi::c_void,
            size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
            null_mut(),
        );
        (*DebugEvent).u.CreateThread.lpThreadLocalBase = if status < 0 {
            null_mut()
        } else {
            thread_basic_info.TebBaseAddress
        };
    } else if new_state == DbgCreateProcessStateChange {
        /* New process */
        (*DebugEvent).dwDebugEventCode = CREATE_PROCESS_DEBUG_EVENT;
        (*DebugEvent).u.CreateProcessInfo.hProcess =
            (*WaitStateChange).StateInfo.CreateProcessInfo.HandleToProcess;
        (*DebugEvent).u.CreateProcessInfo.hThread =
            (*WaitStateChange).StateInfo.CreateProcessInfo.HandleToThread;
        (*DebugEvent).u.CreateProcessInfo.hFile =
            (*WaitStateChange).StateInfo.CreateProcessInfo.NewProcess.FileHandle;
        (*DebugEvent).u.CreateProcessInfo.lpBaseOfImage =
            (*WaitStateChange).StateInfo.CreateProcessInfo.NewProcess.BaseOfImage;
        (*DebugEvent).u.CreateProcessInfo.dwDebugInfoFileOffset =
            (*WaitStateChange).StateInfo.CreateProcessInfo.NewProcess.DebugInfoFileOffset;
        (*DebugEvent).u.CreateProcessInfo.nDebugInfoSize =
            (*WaitStateChange).StateInfo.CreateProcessInfo.NewProcess.DebugInfoSize;
        (*DebugEvent).u.CreateProcessInfo.lpStartAddress = transmute(
            (*WaitStateChange).StateInfo.CreateProcessInfo.NewProcess.InitialThread.StartAddress,
        );

        let mut thread_basic_info: THREAD_BASIC_INFORMATION = zeroed();
        let status = NtQueryInformationThread(
            (*WaitStateChange).StateInfo.CreateProcessInfo.HandleToThread,
            ThreadBasicInformation,
            &mut thread_basic_info as *mut _ as *mut core::ffi::c_void,
            size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
            null_mut(),
        );
        (*DebugEvent).u.CreateProcessInfo.lpThreadLocalBase = if status < 0 {
            null_mut()
        } else {
            thread_basic_info.TebBaseAddress
        };

        (*DebugEvent).u.CreateProcessInfo.lpImageName = null_mut();
        (*DebugEvent).u.CreateProcessInfo.fUnicode = 1;
    } else if new_state == DbgExitThreadStateChange {
        /* Thread exited */
        (*DebugEvent).dwDebugEventCode = EXIT_THREAD_DEBUG_EVENT;
        (*DebugEvent).u.ExitThread.dwExitCode =
            (*WaitStateChange).StateInfo.ExitThread.ExitStatus as DWORD;
    } else if new_state == DbgExitProcessStateChange {
        /* Process exited */
        (*DebugEvent).dwDebugEventCode = EXIT_PROCESS_DEBUG_EVENT;
        (*DebugEvent).u.ExitProcess.dwExitCode =
            (*WaitStateChange).StateInfo.ExitProcess.ExitStatus as DWORD;
    } else if new_state == DbgExceptionStateChange
        || new_state == DbgBreakpointStateChange
        || new_state == DbgSingleStepStateChange
    {
        /* Any sort of exception */
        let exception_code = (*WaitStateChange)
            .StateInfo
            .Exception
            .ExceptionRecord
            .ExceptionCode;
        if exception_code == DBG_PRINTEXCEPTION_C {
            /* This was a debug print */
            (*DebugEvent).dwDebugEventCode = OUTPUT_DEBUG_STRING_EVENT;
            (*DebugEvent).u.DebugString.lpDebugStringData = (*WaitStateChange)
                .StateInfo
                .Exception
                .ExceptionRecord
                .ExceptionInformation[1] as LPSTR;
            (*DebugEvent).u.DebugString.nDebugStringLength = (*WaitStateChange)
                .StateInfo
                .Exception
                .ExceptionRecord
                .ExceptionInformation[0] as USHORT;
            (*DebugEvent).u.DebugString.fUnicode = 0;
        } else if exception_code == DBG_RIPEXCEPTION {
            (*DebugEvent).dwDebugEventCode = RIP_EVENT;
            (*DebugEvent).u.RipInfo.dwType = (*WaitStateChange)
                .StateInfo
                .Exception
                .ExceptionRecord
                .ExceptionInformation[1] as DWORD;
            (*DebugEvent).u.RipInfo.dwError = (*WaitStateChange)
                .StateInfo
                .Exception
                .ExceptionRecord
                .ExceptionInformation[0] as DWORD;
        } else {
            /* Otherwise, this is a debug event, copy info over */
            (*DebugEvent).dwDebugEventCode = EXCEPTION_DEBUG_EVENT;
            (*DebugEvent).u.Exception.ExceptionRecord =
                (*WaitStateChange).StateInfo.Exception.ExceptionRecord;
            (*DebugEvent).u.Exception.dwFirstChance =
                (*WaitStateChange).StateInfo.Exception.FirstChance as DWORD;
        }
    } else if new_state == DbgLoadDllStateChange {
        /* DLL Load */
        (*DebugEvent).dwDebugEventCode = LOAD_DLL_DEBUG_EVENT;
        (*DebugEvent).u.LoadDll.hFile = (*WaitStateChange).StateInfo.LoadDll.FileHandle;
        (*DebugEvent).u.LoadDll.lpBaseOfDll = (*WaitStateChange).StateInfo.LoadDll.BaseOfDll;
        (*DebugEvent).u.LoadDll.dwDebugInfoFileOffset =
            (*WaitStateChange).StateInfo.LoadDll.DebugInfoFileOffset;
        (*DebugEvent).u.LoadDll.nDebugInfoSize =
            (*WaitStateChange).StateInfo.LoadDll.DebugInfoSize;
        (*DebugEvent).u.LoadDll.lpImageName = (*WaitStateChange).StateInfo.LoadDll.NamePointer;
        (*DebugEvent).u.LoadDll.fUnicode = 1;
    } else if new_state == DbgUnloadDllStateChange {
        /* DLL Unload */
        (*DebugEvent).dwDebugEventCode = UNLOAD_DLL_DEBUG_EVENT;
        (*DebugEvent).u.UnloadDll.lpBaseOfDll =
            (*WaitStateChange).StateInfo.UnloadDll.BaseAddress;
    } else {
        /* Anything else, fail */
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}

/// C: `NTSTATUS NTAPI DbgUiWaitStateChange(PDBGUI_WAIT_STATE_CHANGE,
/// PLARGE_INTEGER TimeOut OPTIONAL)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiWaitStateChange(
    WaitStateChange: PDBGUI_WAIT_STATE_CHANGE,
    TimeOut: PLARGE_INTEGER,
) -> NTSTATUS {
    NtWaitForDebugEvent(
        (*NtCurrentTeb()).DbgSsReserved[1],
        TRUE as BOOLEAN,
        TimeOut,
        WaitStateChange,
    )
}

/// C: `VOID NTAPI DbgUiRemoteBreakin(VOID)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiRemoteBreakin() {
    /* Make sure a debugger is enabled; if so, breakpoint */
    if (*NtCurrentPeb()).BeingDebugged != 0 {
        DbgBreakPoint();
    }

    /* Exit the thread */
    RtlExitUserThread(STATUS_SUCCESS);
}

/// C: `NTSTATUS NTAPI DbgUiIssueRemoteBreakin(HANDLE Process)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiIssueRemoteBreakin(Process: HANDLE) -> NTSTATUS {
    let mut h_thread: HANDLE = null_mut();
    let mut client_id: _CLIENT_ID = zeroed();

    // C casts (PVOID)DbgUiRemoteBreakin into the PTHREAD_START_ROUTINE slot;
    // the start routine's parameter is ignored (the thread breakpoints then
    // RtlExitUserThread's out). Both are stdcall on i386, so transmute the
    // (non-null) fn pointer into the niche-Some of the Option<fn> alias.
    let start: PTHREAD_START_ROUTINE =
        transmute(DbgUiRemoteBreakin as unsafe extern "system" fn());

    /* Create the thread that will do the breakin */
    let status = RtlCreateUserThread(
        Process,
        null_mut(),
        FALSE as BOOLEAN,
        0,
        0,
        PAGE_SIZE as SIZE_T,
        start,
        null_mut(),
        &mut h_thread,
        &mut client_id,
    );

    /* Close the handle on success */
    if status >= 0 {
        NtClose(h_thread);
    }

    status
}

/// C: `HANDLE NTAPI DbgUiGetThreadDebugObject(VOID)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiGetThreadDebugObject() -> HANDLE {
    /* Just return the handle from the TEB */
    (*NtCurrentTeb()).DbgSsReserved[1]
}

/// C: `VOID NTAPI DbgUiSetThreadDebugObject(HANDLE DebugObject)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiSetThreadDebugObject(DebugObject: HANDLE) {
    /* Just set the handle in the TEB */
    (*NtCurrentTeb()).DbgSsReserved[1] = DebugObject;
}

/// C: `NTSTATUS NTAPI DbgUiDebugActiveProcess(HANDLE Process)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiDebugActiveProcess(Process: HANDLE) -> NTSTATUS {
    /* Tell the kernel to start debugging */
    let mut status = NtDebugActiveProcess(Process, (*NtCurrentTeb()).DbgSsReserved[1]);
    if status >= 0 {
        /* Now break-in the process */
        status = DbgUiIssueRemoteBreakin(Process);
        if status < 0 {
            /* We couldn't break-in, cancel debugging */
            DbgUiStopDebugging(Process);
        }
    }

    status
}

/// C: `NTSTATUS NTAPI DbgUiStopDebugging(HANDLE Process)`. @implemented.
#[no_mangle]
pub unsafe extern "system" fn DbgUiStopDebugging(Process: HANDLE) -> NTSTATUS {
    /* Call the kernel to remove the debug object */
    NtRemoveProcessDebug(Process, (*NtCurrentTeb()).DbgSsReserved[1])
}
