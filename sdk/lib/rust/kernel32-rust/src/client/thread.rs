use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

use crate::k32_externs::{
    BaseCreateStack, BaseInitializeContext, BaseFormatObjectAttributes,
    HandleToUlong, UlongToHandle, CSR_CREATE_API_NUMBER, BaseRunningInServerProcess,
    GetProcAddress,
};
// ExitProcess (process.c, still C) / GetModuleHandleA / ERROR_FATAL helpers:
extern "system" {
    fn ExitProcess(uExitCode: DWORD) -> !;
    fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE;
    fn BaseFreeThreadStack(hProcess: HANDLE, InitialTeb: PINITIAL_TEB);
}

type PCSR_CREATE_REMOTE_THREAD =
    Option<unsafe extern "system" fn(HANDLE, PCLIENT_ID) -> NTSTATUS>;

// The NT6 THREAD_ALL_ACCESS (upstream a6bdabdaf62's THREAD_ALL_ACCESS_NT6): the thread
// type's valid mask and GENERIC_ALL are the NT6 ones since 4b4063a01d6, so the handle
// CreateRemoteThread returns carries the Vista rights (the *_LIMITED_INFORMATION pair).
const THREAD_ALL_ACCESS: ACCESS_MASK = 0x1FFFFF;
const STATUS_NO_YIELD_PERFORMED: NTSTATUS = 0x40000024u32 as NTSTATUS;
const INVALID_ACTIVATION_CONTEXT: PACTIVATION_CONTEXT = -1isize as PACTIVATION_CONTEXT;

#[inline]
fn LANGIDFROMLCID(lcid: LCID) -> DWORD {
    (lcid & 0xFFFF) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn BasepNotifyCsrOfThread(
    ThreadHandle: HANDLE,
    ClientId: PCLIENT_ID,
) -> NTSTATUS {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let CreateThreadRequest: PBASE_CREATE_THREAD = &mut ApiMessage.Data.CreateThreadRequest;

    /* Fill out the request */
    (*CreateThreadRequest).ClientId = *ClientId;
    (*CreateThreadRequest).ThreadHandle = ThreadHandle;

    /* Call CSR */
    CsrClientCallServer(
        &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX as ULONG, BasepCreateThread as ULONG),
        core::mem::size_of::<BASE_CREATE_THREAD>() as u32, // sizeof(*CreateThreadRequest)
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        /* DPRINT1("Failed to tell CSRSS about new thread: %lx\n", ApiMessage.Status); */
        return ApiMessage.Status;
    }

    /* Return Success */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn BaseThreadStartup(
    lpStartAddress: LPTHREAD_START_ROUTINE,
    lpParameter: LPVOID,
) {
    crate::client::except::run_under_top_level_handler(move || {
        // Legacy check which is still used today for Win32 threads
        if (*NtCurrentTeb()).NtTib.__bindgen_anon_1.Version == (30 << 8) {
            // This registers the termination port with CSRSS
            if BaseRunningInServerProcess == 0 {
                CsrNewThread();
            }
        }

        // Get the exit code from the Thread Start
        ExitThread(lpStartAddress.expect("non-null function pointer")(lpParameter))
    })
}

#[no_mangle]
pub unsafe extern "system" fn BaseDispatchApc(
    ApcRoutine: PAPCFUNC,
    Data: PVOID,
    ActivationContext: PACTIVATION_CONTEXT,
) {
    let mut ActivationFrame: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME = core::mem::zeroed();

    /* Setup the activation context */
    ActivationFrame.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as SIZE_T;
    ActivationFrame.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;

    /* Check if caller wanted one */
    if ActivationContext == INVALID_ACTIVATION_CONTEXT
    {
        /* Do the APC directly */
        ApcRoutine.expect("non-null function pointer")(Data as ULONG_PTR);
        return;
    }

    /* Then activate it */
    RtlActivateActivationContextUnsafeFast(&mut ActivationFrame, ActivationContext);

    /* Call the routine under SEH */
    // _SEH2_TRY
    {
        ApcRoutine.expect("non-null function pointer")(Data as ULONG_PTR);
    }
    // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        // Exception handler - empty
    }
    // _SEH2_END;

    /* Now de-activate and release the activation context */
    RtlDeactivateActivationContextUnsafeFast(&mut ActivationFrame);
    RtlReleaseActivationContext(ActivationContext);
}

#[no_mangle]
pub unsafe extern "system" fn CreateThread(
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    dwStackSize: DWORD,
    lpStartAddress: LPTHREAD_START_ROUTINE,
    lpParameter: LPVOID,
    dwCreationFlags: DWORD,
    lpThreadId: LPDWORD,
) -> HANDLE {
    CreateRemoteThread(
        NtCurrentProcess(),
        lpThreadAttributes,
        dwStackSize,
        lpStartAddress,
        lpParameter,
        dwCreationFlags,
        lpThreadId,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateRemoteThread(
    hProcess: HANDLE,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    dwStackSize: DWORD,
    lpStartAddress: LPTHREAD_START_ROUTINE,
    lpParameter: LPVOID,
    dwCreationFlags: DWORD,
    lpThreadId: LPDWORD,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut InitialTeb: INITIAL_TEB = core::mem::zeroed();
    let mut Context: CONTEXT = core::mem::zeroed();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let ObjectAttributes: POBJECT_ATTRIBUTES;
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut Dummy: ULONG = 0;
    let mut Teb: PTEB;
    let mut ThreadBasicInfo: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let mut ActivationContextStack: PACTIVATION_CONTEXT_STACK = core::ptr::null_mut();
    let mut ActCtxInfo: ACTIVATION_CONTEXT_BASIC_INFORMATION = core::mem::zeroed();
    let mut Cookie: ULONG_PTR = 0;
    let mut ReturnLength: ULONG = 0;
    let mut ReturnSize: SIZE_T = 0;

    /* Clear the Context */
    core::ptr::write_bytes(&mut Context as *mut CONTEXT as *mut u8, 0, core::mem::size_of::<CONTEXT>());

    /* Write PID */
    ClientId.UniqueProcess = hProcess;

    /* Create the Stack */
    Status = BaseCreateStack(
        hProcess,
        if (dwCreationFlags & STACK_SIZE_PARAM_IS_A_RESERVATION) != 0 {
            0
        } else {
            dwStackSize as SIZE_T
        },
        if (dwCreationFlags & STACK_SIZE_PARAM_IS_A_RESERVATION) != 0 {
            dwStackSize as SIZE_T
        } else {
            0
        },
        &mut InitialTeb,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Create the Initial Context */
    BaseInitializeContext(
        &mut Context,
        lpParameter,
        core::mem::transmute::<LPTHREAD_START_ROUTINE, PVOID>(lpStartAddress),
        InitialTeb.StackBase,
        1,
    );

    /* Initialize the attributes for the thread object */
    ObjectAttributes = BaseFormatObjectAttributes(
        &mut LocalObjectAttributes,
        lpThreadAttributes,
        core::ptr::null_mut(),
    );

    /* Create the Kernel Thread Object */
    Status = NtCreateThread(
        &mut hThread,
        THREAD_ALL_ACCESS,
        ObjectAttributes,
        hProcess,
        &mut ClientId,
        &mut Context,
        &mut InitialTeb,
        1u8,
    );
    if !NT_SUCCESS(Status) {
        /* Fail the kernel create */
        BaseFreeThreadStack(hProcess, &mut InitialTeb);
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* The body down to `Quit:` lives in 'Quit; `goto Quit` =>
     * `break 'Quit` (the cleanup tail follows the block). */
    'Quit: {
    /* Are we in the same process? */
    if hProcess == NtCurrentProcess() {
        /* Get the TEB */
        Status = NtQueryInformationThread(
            hThread,
            ThreadBasicInformation,
            &mut ThreadBasicInfo as *mut THREAD_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
            &mut ReturnLength,
        );
        if !NT_SUCCESS(Status) {
            /* Fail */
            break 'Quit;
        }

        /* Allocate the Activation Context Stack */
        Status = RtlAllocateActivationContextStack(&mut ActivationContextStack);
        if !NT_SUCCESS(Status) {
            /* Fail */
            break 'Quit;
        }

        /* Save it */
        Teb = ThreadBasicInfo.TebBaseAddress as PTEB;
        (*Teb).ActivationContextStackPointer = ActivationContextStack;

        /* Query the Context */
        Status = RtlQueryInformationActivationContext(
            RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            ActivationContextBasicInformation as ULONG,
            &mut ActCtxInfo as *mut ACTIVATION_CONTEXT_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>() as SIZE_T,
            &mut ReturnSize,
        );
        if !NT_SUCCESS(Status) {
            /* Fail */
            break 'Quit;
        }

        /* Does it need to be activated? */
        if (ActCtxInfo.hActCtx != core::ptr::null_mut()) && (ActCtxInfo.dwFlags & 1) == 0 {
            /* Activate it */
            Status = RtlActivateActivationContextEx(
                RTL_ACTIVATE_ACTIVATION_CONTEXT_EX_FLAG_RELEASE_ON_STACK_DEALLOCATION,
                Teb,
                ActCtxInfo.hActCtx,
                &mut Cookie,
            );
            if !NT_SUCCESS(Status) {
                /* Fail */
                break 'Quit;
            }
        }

        /* Sync the service tag with the parent thread's one */
        (*Teb).SubProcessTag = (*NtCurrentTeb()).SubProcessTag;
    }

    /* Notify CSR */
    if BaseRunningInServerProcess == 0 {
        Status = BasepNotifyCsrOfThread(hThread, &mut ClientId);

        /* A WoW64 guest runs with NO CSRSS connection at all: the 64-bit
         * csrss refuses the 32-bit CSR_API_CONNECTINFO until the CSR-LPC-32
         * thunk layer exists, and DllMain deliberately continues anyway
         * ("WOW64: Continuing without CSRSS connection"). The notify above
         * therefore hands NtRequestWaitReplyPort a port that was never
         * connected, the kernel rejects it with STATUS_INVALID_PARAMETER, and
         * the cleanup path below tears down a thread that is otherwise
         * perfectly built — which is why EVERY CreateThread in EVERY 32-bit
         * guest failed with ERROR_INVALID_PARAMETER (task #64).
         *
         * Tolerate it for exactly that case and no other. What is lost is
         * csrss's bookkeeping entry for the thread, which nothing in a
         * CSR-less process consults; a guest that ever does get a CSR
         * connection takes the strict path again. */
        #[cfg(target_arch = "x86")]
        if !NT_SUCCESS(Status) && crate::client::dllmain::BaseWow64NoCsrConnection {
            static mut Reported: bool = false;
            if !Reported {
                Reported = true;
                DbgPrint(
                    b"WOW64: CreateThread continuing without CSRSS thread notify (0x%lx)\n\0"
                        .as_ptr() as *const i8,
                    Status,
                );
            }
            Status = STATUS_SUCCESS;
        }
    } else {
        if hProcess != NtCurrentProcess() {
            let CsrCreateRemoteThread: PCSR_CREATE_REMOTE_THREAD;

            /* Get the direct CSRSRV export */
            CsrCreateRemoteThread = core::mem::transmute::<FARPROC, PCSR_CREATE_REMOTE_THREAD>(
                GetProcAddress(
                    GetModuleHandleA(b"csrsrv\0".as_ptr() as *const i8),
                    b"CsrCreateRemoteThread\0".as_ptr() as *const i8,
                ),
            );
            if CsrCreateRemoteThread.is_some() {
                /* Call it instead of going through LPC */
                Status = CsrCreateRemoteThread.unwrap()(hThread, &mut ClientId);
            }
        }
    }
    } // 'Quit

    // Quit:
    {
        if !NT_SUCCESS(Status) {
            /* Failed to create the thread */

            /* Free the activation context stack */
            if ActivationContextStack != core::ptr::null_mut() {
                RtlFreeActivationContextStack(ActivationContextStack);
            }

            NtTerminateThread(hThread, Status);
            // FIXME: Wait for the thread to terminate?
            BaseFreeThreadStack(hProcess, &mut InitialTeb);
            NtClose(hThread);

            BaseSetLastNTError(Status);
            return core::ptr::null_mut();
        }
    }

    /* Success */
    if lpThreadId != core::ptr::null_mut() {
        *lpThreadId = HandleToUlong(ClientId.UniqueThread);
    }

    /* Resume the thread if asked */
    if (dwCreationFlags & CREATE_SUSPENDED) == 0 {
        NtResumeThread(hThread, &mut Dummy);
    }

    /* Return handle to thread */
    hThread
}

#[no_mangle]
pub unsafe extern "system" fn ExitThread(uExitCode: DWORD) {
    let mut Status: NTSTATUS;
    let mut LastThread: ULONG = 0;
    let LoaderLock: PRTL_CRITICAL_SECTION;

    /* Make sure loader lock isn't held */
    LoaderLock = (*NtCurrentPeb()).LoaderLock;
    if !LoaderLock.is_null() {
        debug_assert!((*NtCurrentTeb()).ClientId.UniqueThread != (*LoaderLock).OwningThread);
    }

    /*
     * Terminate process if this is the last thread
     * of the current process
     */
    Status = NtQueryInformationThread(
        NtCurrentThread(),
        ThreadAmILastThread,
        &mut LastThread as *mut ULONG as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as u32,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) && (LastThread != 0) {
        ExitProcess(uExitCode);
    }

    /* Notify DLLs and TLS Callbacks of termination */
    LdrShutdownThread();

    /* Tell the Kernel to free the Stack */
    (*NtCurrentTeb()).FreeStackOnTermination = 1u8;
    NtTerminateThread(core::ptr::null_mut(), uExitCode as NTSTATUS);

    /* Divergence from the C, which spins here ("should never be reached"). The kernel
     * refuses to terminate the current thread only when it is the last one
     * (STATUS_CANT_TERMINATE_SELF), so finish the job the way the last thread does: a
     * silent busy loop at normal priority starved the arm64 guest of a CPU forever. */
    ExitProcess(uExitCode);
}

#[no_mangle]
pub unsafe extern "system" fn OpenThread(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    dwThreadId: DWORD,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();

    ClientId.UniqueProcess = core::ptr::null_mut();
    ClientId.UniqueThread = UlongToHandle(dwThreadId);

    // InitializeObjectAttributes inline expansion
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.Attributes = if bInheritHandle != 0 { OBJ_INHERIT } else { 0 };
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenThread(
        &mut ThreadHandle,
        dwDesiredAccess,
        &mut ObjectAttributes,
        &mut ClientId,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    ThreadHandle
}
#[no_mangle]
pub unsafe extern "C" fn GetTeb() -> PTEB {
    return NtCurrentTeb();
}

#[no_mangle]
pub unsafe extern "system" fn SwitchToThread() -> BOOL {
    (NtYieldExecution() != STATUS_NO_YIELD_PERFORMED) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentThreadId() -> DWORD {
    return HandleToUlong((*NtCurrentTeb()).ClientId.UniqueThread);
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadTimes(
    hThread: HANDLE,
    lpCreationTime: LPFILETIME,
    lpExitTime: LPFILETIME,
    lpKernelTime: LPFILETIME,
    lpUserTime: LPFILETIME,
) -> BOOL {
    let mut KernelUserTimes: KERNEL_USER_TIMES = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        hThread,
        ThreadTimes,
        &mut KernelUserTimes as *mut KERNEL_USER_TIMES as *mut core::ffi::c_void,
        core::mem::size_of::<KERNEL_USER_TIMES>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *lpCreationTime = *(&KernelUserTimes.CreateTime as *const _ as *const FILETIME);
    *lpExitTime = *(&KernelUserTimes.ExitTime as *const _ as *const FILETIME);
    *lpKernelTime = *(&KernelUserTimes.KernelTime as *const _ as *const FILETIME);
    *lpUserTime = *(&KernelUserTimes.UserTime as *const _ as *const FILETIME);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadContext(
    hThread: HANDLE,
    lpContext: LPCONTEXT,
) -> BOOL {
    let Status: NTSTATUS;

    Status = NtGetContextThread(hThread, lpContext);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadContext(
    hThread: HANDLE,
    lpContext: *const CONTEXT,
) -> BOOL {
    let Status: NTSTATUS;
    Status = NtSetContextThread(hThread, lpContext as PCONTEXT);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn GetExitCodeThread(
    hThread: HANDLE,
    lpExitCode: LPDWORD,
) -> BOOL {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        hThread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *lpExitCode = ThreadBasic.ExitStatus as DWORD;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ResumeThread(hThread: HANDLE) -> DWORD {
    let mut PreviousResumeCount: ULONG = 0;
    let Status: NTSTATUS;

    Status = NtResumeThread(hThread, &mut PreviousResumeCount);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return !0u32; // -1 as DWORD
    }

    return PreviousResumeCount;
}

#[no_mangle]
pub unsafe extern "system" fn TerminateThread(
    hThread: HANDLE,
    dwExitCode: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let LoaderLock: PRTL_CRITICAL_SECTION;
    let mut ThreadInfo: THREAD_BASIC_INFORMATION = core::mem::zeroed();

    /* Check for invalid thread handle */
    if hThread.is_null() {
        /* Fail if one was passed */
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Get the loader lock */
    LoaderLock = (*NtCurrentPeb()).LoaderLock;
    if !LoaderLock.is_null() {
        /* Get our TID */
        Status = NtQueryInformationThread(
            hThread,
            ThreadBasicInformation,
            &mut ThreadInfo as *mut THREAD_BASIC_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            /* If terminating the current thread, we must not hold the loader lock */
            if (*NtCurrentTeb()).ClientId.UniqueThread == ThreadInfo.ClientId.UniqueThread {
                // ASSERT(NtCurrentTeb()->ClientId.UniqueThread != LoaderLock->OwningThread);
            }
        }
    }

    /* Now terminate the thread */
    Status = NtTerminateThread(hThread, dwExitCode as NTSTATUS);
    if !NT_SUCCESS(Status) {
        /* Fail */
        BaseSetLastNTError(Status);
        return 0;
    }

    /* All done */
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SuspendThread(hThread: HANDLE) -> DWORD {
    let mut PreviousSuspendCount: ULONG = 0;
    let Status: NTSTATUS;

    Status = NtSuspendThread(hThread, &mut PreviousSuspendCount);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0xFFFFFFFF; // -1 as DWORD
    }

    return PreviousSuspendCount;
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadAffinityMask(
    hThread: HANDLE,
    dwThreadAffinityMask: DWORD_PTR,
) -> DWORD_PTR {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let mut AffinityMask: KAFFINITY;
    let mut Status: NTSTATUS;

    AffinityMask = dwThreadAffinityMask as KAFFINITY;

    Status = NtQueryInformationThread(
        hThread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    Status = NtSetInformationThread(
        hThread,
        ThreadAffinityMask,
        &mut AffinityMask as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<KAFFINITY>() as u32,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        ThreadBasic.AffinityMask = 0;
    }

    return ThreadBasic.AffinityMask;
}

// Upstream-sync 2026-09: GetThreadGroupAffinity/SetThreadGroupAffinity are genuinely NEW
// upstream code (thread.c, added in the 1,269-commit merge; no pre-merge C or Rust form
// existed). Both are UP-only fakes in the C original too -- `STUB; // FIXME: Real group
// support` -- so this is a faithful, not a lesser, port: there is only ever one group
// (THEONLYGROUP == 0) until real processor-group support lands.
const THEONLYGROUP: WORD = 0;

#[no_mangle]
pub unsafe extern "system" fn GetThreadGroupAffinity(
    hThread: HANDLE,
    GroupAffinity: PGROUP_AFFINITY,
) -> BOOL {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();

    let Status = NtQueryInformationThread(
        hThread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *GroupAffinity = core::mem::zeroed(); // For Reserved
    (*GroupAffinity).Mask = ThreadBasic.AffinityMask as KAFFINITY;
    (*GroupAffinity).Group = THEONLYGROUP;
    1
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadGroupAffinity(
    hThread: HANDLE,
    GroupAffinity: *const GROUP_AFFINITY,
    PreviousGroupAffinity: PGROUP_AFFINITY,
) -> BOOL {
    if GroupAffinity.is_null() || (*GroupAffinity).Group != THEONLYGROUP {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let OldMask = SetThreadAffinityMask(hThread, (*GroupAffinity).Mask as DWORD_PTR);
    if OldMask != 0 && !PreviousGroupAffinity.is_null() {
        *PreviousGroupAffinity = core::mem::zeroed(); // For Reserved
        (*PreviousGroupAffinity).Mask = OldMask as KAFFINITY;
        (*PreviousGroupAffinity).Group = THEONLYGROUP;
    }
    (OldMask != 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadPriority(
    hThread: HANDLE,
    nPriority: i32,
) -> BOOL {
    let mut Prio: LONG = nPriority;
    let Status: NTSTATUS;

    if Prio == THREAD_PRIORITY_TIME_CRITICAL as i32 {
        Prio = ((HIGH_PRIORITY as i32) + 1) / 2;
    } else if Prio == THREAD_PRIORITY_IDLE {
        Prio = -(((HIGH_PRIORITY as i32) + 1) / 2);
    }

    Status = NtSetInformationThread(
        hThread,
        ThreadBasePriority,
        &Prio as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<LONG>() as u32,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadPriority(hThread: HANDLE) -> i32 {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        hThread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return THREAD_PRIORITY_ERROR_RETURN as i32;
    }

    if ThreadBasic.BasePriority == ((HIGH_PRIORITY as i32) + 1) / 2 {
        ThreadBasic.BasePriority = THREAD_PRIORITY_TIME_CRITICAL as i32;
    } else if ThreadBasic.BasePriority == -(((HIGH_PRIORITY as i32) + 1) / 2) {
        ThreadBasic.BasePriority = THREAD_PRIORITY_IDLE;
    }

    return ThreadBasic.BasePriority;
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadPriorityBoost(
    hThread: HANDLE,
    pDisablePriorityBoost: PBOOL,
) -> BOOL {
    let mut PriorityBoost: ULONG = 0;
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        hThread,
        ThreadPriorityBoost,
        &mut PriorityBoost as *mut ULONG as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    *pDisablePriorityBoost = PriorityBoost as BOOL;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadPriorityBoost(
    hThread: HANDLE,
    bDisablePriorityBoost: BOOL,
) -> BOOL {
    let mut PriorityBoost: ULONG;
    let Status: NTSTATUS;

    PriorityBoost = if bDisablePriorityBoost != 0 { 1 } else { 0 };

    Status = NtSetInformationThread(
        hThread,
        ThreadPriorityBoost,
        &mut PriorityBoost as *mut ULONG as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as u32,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadSelectorEntry(
    hThread: HANDLE,
    dwSelector: DWORD,
    lpSelectorEntry: LPLDT_ENTRY,
) -> BOOL {
    #[cfg(target_arch = "x86")]
    {
        let mut DescriptionTableEntry: DESCRIPTOR_TABLE_ENTRY = core::mem::zeroed();
        let Status: NTSTATUS;

        DescriptionTableEntry.Selector = dwSelector;
        Status = NtQueryInformationThread(
            hThread,
            ThreadDescriptorTableEntry,
            &mut DescriptionTableEntry as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<DESCRIPTOR_TABLE_ENTRY>() as u32,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return 0;
        }

        *lpSelectorEntry = DescriptionTableEntry.Descriptor;
        return 1;
    }

    #[cfg(not(target_arch = "x86"))]
    {
        // DPRINT removed
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadIdealProcessor(
    hThread: HANDLE,
    dwIdealProcessor: DWORD,
) -> DWORD {
    let Status: NTSTATUS;
    Status = NtSetInformationThread(
        hThread,
        ThreadIdealProcessor,
        &dwIdealProcessor as *const DWORD as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as u32,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return !0u32; // -1 as DWORD
    }
    return Status as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessIdOfThread(Thread: HANDLE) -> DWORD {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        Thread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut THREAD_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return HandleToUlong(ThreadBasic.ClientId.UniqueProcess);
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadId(Thread: HANDLE) -> DWORD {
    let mut ThreadBasic: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        Thread,
        ThreadBasicInformation,
        &mut ThreadBasic as *mut THREAD_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return HandleToUlong(ThreadBasic.ClientId.UniqueThread);
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadUILanguage(LangId: u32) -> u32 {
    // We only support LangId == 0, for selecting a language
    // identifier that best supports the NT Console.
    if LangId != 0 {
        BaseSetLastNTError(STATUS_NOT_SUPPORTED);
        return 0;
    }

    // UNIMPLEMENTED - placeholder for future implementation
    // This is a mechanical transliteration; the original C had UNIMPLEMENTED here

    return LANGIDFROMLCID((*NtCurrentTeb()).CurrentLocale);
}

#[no_mangle]
pub unsafe extern "system" fn QueueUserAPC(
    pfnAPC: PAPCFUNC,
    hThread: HANDLE,
    dwData: ULONG_PTR,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut ActCtxInfo: ACTIVATION_CONTEXT_BASIC_INFORMATION = core::mem::zeroed();

    /* Zero the activation context and query information on it */
    core::ptr::write_bytes(&mut ActCtxInfo as *mut _ as *mut u8, 0, core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>());
    Status = RtlQueryInformationActivationContext(
        RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ActivationContextBasicInformation as ULONG,
        &mut ActCtxInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        /* Fail due to SxS */
        DbgPrint(
            b"SXS: %s failing because RtlQueryInformationActivationContext()\
             returned status %08lx\n\0".as_ptr() as *const i8,
            b"QueueUserAPC\0".as_ptr() as *const i8,
            Status,
        );
        BaseSetLastNTError(Status);
        return 0i32 as DWORD;
    }

    /* Queue the APC */
    Status = NtQueueApcThread(
        hThread,
        core::mem::transmute::<usize, PKNORMAL_ROUTINE>(BaseDispatchApc as usize),
        core::mem::transmute::<PAPCFUNC, PVOID>(pfnAPC),
        dwData as PVOID,
        if (ActCtxInfo.dwFlags & 1) != 0 {
            INVALID_ACTIVATION_CONTEXT
        } else {
            ActCtxInfo.hActCtx
        },
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32 as DWORD;
    }

    /* All good */
    return 1i32 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadStackGuarantee(
    StackSizeInBytes: *mut u32,
) -> i32 {
    let Teb: PTEB = NtCurrentTeb();
    let GuaranteedStackBytes: u32;
    let AllocationSize: u32;

    if StackSizeInBytes.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    AllocationSize = *StackSizeInBytes;

    /* Retrieve the current stack size */
    GuaranteedStackBytes = (*Teb).GuaranteedStackBytes;

    /* Return the size of the previous stack */
    *StackSizeInBytes = GuaranteedStackBytes;

    /*
     * If the new stack size is either zero or is less than the current size,
     * the previous stack size is returned and we return success.
     */
    if (AllocationSize == 0) || (AllocationSize < GuaranteedStackBytes) {
        return 1;
    }

    // FIXME: Unimplemented!
    // UNIMPLEMENTED_ONCE;

    // Temporary HACK for supporting applications!
    return 1; // FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadIOPendingFlag(
    hThread: HANDLE,
    lpIOIsPending: PBOOL,
) -> BOOL {
    let mut IoPending: ULONG = 0;
    let Status: NTSTATUS;

    Status = NtQueryInformationThread(
        hThread,
        ThreadIsIoPending,
        &mut IoPending as *mut ULONG as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as u32,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        *lpIOIsPending = if IoPending != 0 { 1 } else { 0 };
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn QueueUserWorkItem(
    Function: LPTHREAD_START_ROUTINE,
    Context: PVOID,
    Flags: ULONG,
) -> BOOL {
    let Status: NTSTATUS;

    /* NOTE: Rtl needs to safely call the function using a trampoline */
    Status = RtlQueueWorkItem(
        core::mem::transmute::<LPTHREAD_START_ROUTINE, WORKERCALLBACKFUNC>(Function),
        Context,
        Flags,
    );
    if !NT_SUCCESS(Status) {
        /* Failed */
        BaseSetLastNTError(Status);
        return 0; // FALSE
    }

    /* All good */
    return 1; // TRUE
}
#[no_mangle]
pub unsafe extern "system" fn TlsAlloc() -> DWORD {
    let mut Index: ULONG;
    let Teb: PTEB;
    let Peb: PPEB;

    /* Get the PEB and TEB, lock the PEB */
    Teb = NtCurrentTeb();
    Peb = (*Teb).ProcessEnvironmentBlock;
    RtlAcquirePebLock();

    /* Try to get regular TEB slot */
    Index = RtlFindClearBitsAndSet((*Peb).TlsBitmap as PRTL_BITMAP, 1, 0);
    if Index != 0xFFFFFFFF
    {
        /* Clear the value. */
        (*Teb).TlsSlots[Index as usize] = core::ptr::null_mut();
        RtlReleasePebLock();
        return Index;
    }

    /* If it fails, try to find expansion TEB slot. */
    Index = RtlFindClearBitsAndSet((*Peb).TlsExpansionBitmap as PRTL_BITMAP, 1, 0);
    if Index != 0xFFFFFFFF
    {
        /* Is there no expansion slot yet? */
        if (*Teb).TlsExpansionSlots.is_null()
        {
            /* Allocate an array */
            (*Teb).TlsExpansionSlots = RtlAllocateHeap(RtlGetProcessHeap(),
                                                       HEAP_ZERO_MEMORY,
                                                       (TLS_EXPANSION_SLOTS as SIZE_T) *
                                                       core::mem::size_of::<PVOID>() as SIZE_T) as *mut PVOID;
        }

        /* Did we get an array? */
        if (*Teb).TlsExpansionSlots.is_null()
        {
            /* Fail */
            RtlClearBits((*Peb).TlsExpansionBitmap as PRTL_BITMAP, Index, 1);
            Index = 0xFFFFFFFF;
            BaseSetLastNTError(STATUS_NO_MEMORY);
        }
        else
        {
            /* Clear the value. */
            (*Teb).TlsExpansionSlots.offset(Index as isize).write(core::ptr::null_mut());
            Index += TLS_MINIMUM_AVAILABLE;
        }
    }
    else
    {
        /* Fail */
        BaseSetLastNTError(STATUS_NO_MEMORY);
    }

    /* Release the lock and return */
    RtlReleasePebLock();
    return Index;
}

#[no_mangle]
pub unsafe extern "system" fn TlsFree(mut Index: DWORD) -> BOOL {
    let mut BitSet: BOOL;
    let Peb: PPEB;
    let mut TlsIndex: ULONG;
    let mut TlsBitmap: PVOID;
    let mut Status: NTSTATUS;

    /* Acquire the PEB lock and grab the PEB */
    Peb = NtCurrentPeb();
    RtlAcquirePebLock();

    /* Check if the index is too high */
    if Index >= TLS_MINIMUM_AVAILABLE {
        /* Check if it can fit in the expansion slots */
        TlsIndex = Index - TLS_MINIMUM_AVAILABLE;
        if TlsIndex >= TLS_EXPANSION_SLOTS {
            /* It's invalid */
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
            RtlReleasePebLock();
            return 0;
        } else {
            /* Use the expansion bitmap */
            TlsBitmap = (*Peb).TlsExpansionBitmap;
            Index = TlsIndex;
        }
    } else {
        /* Use the normal bitmap */
        TlsBitmap = (*Peb).TlsBitmap;
    }

    /* Check if the index was set */
    BitSet = RtlAreBitsSet(TlsBitmap as PRTL_BITMAP, Index, 1) as BOOL;
    if BitSet != 0 {
        /* Tell the kernel to free the TLS cells */
        Status = NtSetInformationThread(
            NtCurrentThread(),
            ThreadZeroTlsCell,
            &Index as *const _ as *mut core::ffi::c_void,
            core::mem::size_of::<DWORD>() as u32,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
            RtlReleasePebLock();
            return 0;
        }

        /* Clear the bit */
        RtlClearBits(TlsBitmap as PRTL_BITMAP, Index, 1);
    } else {
        /* Fail */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        RtlReleasePebLock();
        return 0;
    }

    /* Done! */
    RtlReleasePebLock();
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn TlsGetValue(Index: DWORD) -> LPVOID {
    let Teb: PTEB;

    /* Get the TEB and clear the last error */
    Teb = NtCurrentTeb();
    (*Teb).LastErrorValue = 0;

    /* Check for simple TLS index */
    if Index < TLS_MINIMUM_AVAILABLE {
        /* Return it */
        return (*Teb).TlsSlots[Index as usize];
    }

    /* Check for valid index */
    if Index >= TLS_EXPANSION_SLOTS + TLS_MINIMUM_AVAILABLE {
        /* Fail */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    /* The expansion slots are allocated on demand, so check for it. */
    (*Teb).LastErrorValue = 0;
    if (*Teb).TlsExpansionSlots.is_null() {
        return core::ptr::null_mut();
    }

    /* Return the value from the expansion slots */
    return *(*Teb).TlsExpansionSlots.add((Index - TLS_MINIMUM_AVAILABLE) as usize);
}

#[no_mangle]
pub unsafe extern "system" fn TlsSetValue(
    Index: DWORD,
    Value: LPVOID,
) -> BOOL {
    let TlsIndex: DWORD;
    let Teb: PTEB = NtCurrentTeb();

    /* Check for simple TLS index */
    if Index < TLS_MINIMUM_AVAILABLE {
        /* Return it */
        (*Teb).TlsSlots[Index as usize] = Value;
        return 1;
    }

    /* Check if this is an expansion slot */
    TlsIndex = Index - TLS_MINIMUM_AVAILABLE;
    if TlsIndex >= TLS_EXPANSION_SLOTS {
        /* Fail */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    /* Do we not have expansion slots? */
    if (*Teb).TlsExpansionSlots.is_null() {
        /* Get the PEB lock to see if we still need them */
        RtlAcquirePebLock();
        if (*Teb).TlsExpansionSlots.is_null() {
            /* Allocate them */
            (*Teb).TlsExpansionSlots = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (TLS_EXPANSION_SLOTS as usize * core::mem::size_of::<PVOID>()) as SIZE_T,
            ) as *mut PVOID;
            if (*Teb).TlsExpansionSlots.is_null() {
                /* Fail */
                RtlReleasePebLock();
                BaseSetLastNTError(STATUS_NO_MEMORY);
                return 0;
            }
        }

        /* Release the lock */
        RtlReleasePebLock();
    }

    /* Write the value */
    *(*Teb).TlsExpansionSlots.add(TlsIndex as usize) = Value;

    /* Success */
    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/thread.c
//  * PURPOSE:         Thread functions
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Ariadne (ariadne@xs4all.nl)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define SXS_SUPPORT_FIXME
//
// typedef NTSTATUS (NTAPI *PCSR_CREATE_REMOTE_THREAD)(IN HANDLE ThreadHandle, IN PCLIENT_ID ClientId);
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// WINAPI
// BasepNotifyCsrOfThread(IN HANDLE ThreadHandle,
//                        IN PCLIENT_ID ClientId)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_CREATE_THREAD CreateThreadRequest = &ApiMessage.Data.CreateThreadRequest;
//
//     DPRINT("BasepNotifyCsrOfThread: Thread: %p, Handle %p\n",
//             ClientId->UniqueThread, ThreadHandle);
//
//     /* Fill out the request */
//     CreateThreadRequest->ClientId = *ClientId;
//     CreateThreadRequest->ThreadHandle = ThreadHandle;
//
//     /* Call CSR */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepCreateThread),
//                         sizeof(*CreateThreadRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         DPRINT1("Failed to tell CSRSS about new thread: %lx\n", ApiMessage.Status);
//         return ApiMessage.Status;
//     }
//
//     /* Return Success */
//     return STATUS_SUCCESS;
// }
//
// DECLSPEC_NORETURN
// VOID
// WINAPI
// BaseThreadStartup(
//      LPTHREAD_START_ROUTINE lpStartAddress,
//      LPVOID lpParameter)
// {
//     /* Attempt to call the Thread Start Address */
//     _SEH2_TRY
//     {
//         /* Legacy check which is still used today for Win32 threads */
//         if (NtCurrentTeb()->NtTib.Version == (30 << 8)) // OS/2 V3.0 ("Cruiser")
//         {
//             /* This registers the termination port with CSRSS */
//             if (!BaseRunningInServerProcess) CsrNewThread();
//         }
//
//         /* Get the exit code from the Thread Start */
//         ExitThread(lpStartAddress(lpParameter));
//     }
//     _SEH2_EXCEPT(UnhandledExceptionFilter(_SEH2_GetExceptionInformation()))
//     {
//         /* Get the Exit code from the SEH Handler */
//         if (!BaseRunningInServerProcess)
//         {
//             /* Kill the whole process, usually */
//             ExitProcess(_SEH2_GetExceptionCode());
//         }
//         else
//         {
//             /* If running inside CSRSS, kill just this thread */
//             ExitThread(_SEH2_GetExceptionCode());
//         }
//     }
//     _SEH2_END;
// }
//
// VOID
// NTAPI
// BaseDispatchApc(IN PAPCFUNC ApcRoutine,
//                 IN PVOID Data,
//                 IN PACTIVATION_CONTEXT ActivationContext)
// {
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActivationFrame;
//
//     /* Setup the activation context */
//     ActivationFrame.Size = sizeof(ActivationFrame);
//     ActivationFrame.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//
//     /* Check if caller wanted one */
//     if (ActivationContext == INVALID_ACTIVATION_CONTEXT)
//     {
//         /* Do the APC directly */
//         ApcRoutine((ULONG_PTR)Data);
//         return;
//     }
//
//     /* Then activate it */
//     RtlActivateActivationContextUnsafeFast(&ActivationFrame, ActivationContext);
//
//     /* Call the routine under SEH */
//     _SEH2_TRY
//     {
//         ApcRoutine((ULONG_PTR)Data);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//
//     }
//     _SEH2_END;
//
//     /* Now de-activate and release the activation context */
//     RtlDeactivateActivationContextUnsafeFast(&ActivationFrame);
//     RtlReleaseActivationContext(ActivationContext);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateThread(IN LPSECURITY_ATTRIBUTES lpThreadAttributes,
//              IN DWORD dwStackSize,
//              IN LPTHREAD_START_ROUTINE lpStartAddress,
//              IN LPVOID lpParameter,
//              IN DWORD dwCreationFlags,
//              OUT LPDWORD lpThreadId)
// {
//     /* Act as if we're going to create a remote thread in ourselves */
//     return CreateRemoteThread(NtCurrentProcess(),
//                               lpThreadAttributes,
//                               dwStackSize,
//                               lpStartAddress,
//                               lpParameter,
//                               dwCreationFlags,
//                               lpThreadId);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateRemoteThread(IN HANDLE hProcess,
//                    IN LPSECURITY_ATTRIBUTES lpThreadAttributes,
//                    IN DWORD dwStackSize,
//                    IN LPTHREAD_START_ROUTINE lpStartAddress,
//                    IN LPVOID lpParameter,
//                    IN DWORD dwCreationFlags,
//                    OUT LPDWORD lpThreadId)
// {
//     NTSTATUS Status;
//     INITIAL_TEB InitialTeb;
//     CONTEXT Context;
//     CLIENT_ID ClientId;
//     OBJECT_ATTRIBUTES LocalObjectAttributes;
//     POBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE hThread;
//     ULONG Dummy;
//     PTEB Teb;
//     THREAD_BASIC_INFORMATION ThreadBasicInfo;
//     PACTIVATION_CONTEXT_STACK ActivationContextStack = NULL;
//     ACTIVATION_CONTEXT_BASIC_INFORMATION ActCtxInfo;
//     ULONG_PTR Cookie;
//     ULONG ReturnLength;
//     SIZE_T ReturnSize;
//
//     DPRINT("CreateRemoteThread: hProcess: %p dwStackSize: %lu lpStartAddress"
//            ": %p lpParameter: %p, dwCreationFlags: %lx\n", hProcess,
//            dwStackSize, lpStartAddress, lpParameter, dwCreationFlags);
//
//     /* Clear the Context */
//     RtlZeroMemory(&Context, sizeof(Context));
//
//     /* Write PID */
//     ClientId.UniqueProcess = hProcess;
//
//     /* Create the Stack */
//     Status = BaseCreateStack(hProcess,
//                              (dwCreationFlags & STACK_SIZE_PARAM_IS_A_RESERVATION) ?
//                                 0 : dwStackSize,
//                              (dwCreationFlags & STACK_SIZE_PARAM_IS_A_RESERVATION) ?
//                                 dwStackSize : 0,
//                              &InitialTeb);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Create the Initial Context */
//     BaseInitializeContext(&Context,
//                           lpParameter,
//                           lpStartAddress,
//                           InitialTeb.StackBase,
//                           1);
//
//     /* Initialize the attributes for the thread object */
//     ObjectAttributes = BaseFormatObjectAttributes(&LocalObjectAttributes,
//                                                   lpThreadAttributes,
//                                                   NULL);
//
//     /* Create the Kernel Thread Object */
//     Status = NtCreateThread(&hThread,
//                             THREAD_ALL_ACCESS,
//                             ObjectAttributes,
//                             hProcess,
//                             &ClientId,
//                             &Context,
//                             &InitialTeb,
//                             TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail the kernel create */
//         BaseFreeThreadStack(hProcess, &InitialTeb);
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Are we in the same process? */
//     if (hProcess == NtCurrentProcess())
//     {
//         /* Get the TEB */
//         Status = NtQueryInformationThread(hThread,
//                                           ThreadBasicInformation,
//                                           &ThreadBasicInfo,
//                                           sizeof(ThreadBasicInfo),
//                                           &ReturnLength);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             DPRINT1("SXS: %s - Failing thread create because "
//                     "NtQueryInformationThread() failed with status %08lx\n",
//                     __FUNCTION__, Status);
//             break 'Quit;
//         }
//
//         /* Allocate the Activation Context Stack */
//         Status = RtlAllocateActivationContextStack(&ActivationContextStack);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             DPRINT1("SXS: %s - Failing thread create because "
//                     "RtlAllocateActivationContextStack() failed with status %08lx\n",
//                     __FUNCTION__, Status);
//             break 'Quit;
//         }
//
//         /* Save it */
//         Teb = ThreadBasicInfo.TebBaseAddress;
//         Teb->ActivationContextStackPointer = ActivationContextStack;
//
//         /* Query the Context */
//         Status = RtlQueryInformationActivationContext(RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT,
//                                                       NULL,
//                                                       0,
//                                                       ActivationContextBasicInformation,
//                                                       &ActCtxInfo,
//                                                       sizeof(ActCtxInfo),
//                                                       &ReturnSize);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             DPRINT1("SXS: %s - Failing thread create because "
//                     "RtlQueryInformationActivationContext() failed with status %08lx\n",
//                     __FUNCTION__, Status);
//             break 'Quit;
//         }
//
//         /* Does it need to be activated? */
//         if ((ActCtxInfo.hActCtx) && !(ActCtxInfo.dwFlags & 1))
//         {
//             /* Activate it */
//             Status = RtlActivateActivationContextEx(RTL_ACTIVATE_ACTIVATION_CONTEXT_EX_FLAG_RELEASE_ON_STACK_DEALLOCATION,
//                                                     Teb,
//                                                     ActCtxInfo.hActCtx,
//                                                     &Cookie);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Fail */
//                 DPRINT1("SXS: %s - Failing thread create because "
//                         "RtlActivateActivationContextEx() failed with status %08lx\n",
//                         __FUNCTION__, Status);
//                 break 'Quit;
//             }
//         }
//
//         /* Sync the service tag with the parent thread's one */
//         Teb->SubProcessTag = NtCurrentTeb()->SubProcessTag;
//     }
//
//     /* Notify CSR */
//     if (!BaseRunningInServerProcess)
//     {
//         Status = BasepNotifyCsrOfThread(hThread, &ClientId);
//     }
//     else
//     {
//         if (hProcess != NtCurrentProcess())
//         {
//             PCSR_CREATE_REMOTE_THREAD CsrCreateRemoteThread;
//
//             /* Get the direct CSRSRV export */
//             CsrCreateRemoteThread = (PCSR_CREATE_REMOTE_THREAD)
//                                     GetProcAddress(GetModuleHandleA("csrsrv"),
//                                                    "CsrCreateRemoteThread");
//             if (CsrCreateRemoteThread)
//             {
//                 /* Call it instead of going through LPC */
//                 Status = CsrCreateRemoteThread(hThread, &ClientId);
//             }
//         }
//     }
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to create the thread */
//
//         /* Free the activation context stack */
//         if (ActivationContextStack)
//             RtlFreeActivationContextStack(ActivationContextStack);
//
//         NtTerminateThread(hThread, Status);
//         // FIXME: Wait for the thread to terminate?
//         BaseFreeThreadStack(hProcess, &InitialTeb);
//         NtClose(hThread);
//
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Success */
//     if (lpThreadId)
//         *lpThreadId = HandleToUlong(ClientId.UniqueThread);
//
//     /* Resume the thread if asked */
//     if (!(dwCreationFlags & CREATE_SUSPENDED))
//         NtResumeThread(hThread, &Dummy);
//
//     /* Return handle to thread */
//     return hThread;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// ExitThread(IN DWORD uExitCode)
// {
//     NTSTATUS Status;
//     ULONG LastThread;
//     PRTL_CRITICAL_SECTION LoaderLock;
//
//     /* Make sure loader lock isn't held */
//     LoaderLock = NtCurrentPeb()->LoaderLock;
//     if (LoaderLock) ASSERT(NtCurrentTeb()->ClientId.UniqueThread != LoaderLock->OwningThread);
//
//     /*
//      * Terminate process if this is the last thread
//      * of the current process
//      */
//     Status = NtQueryInformationThread(NtCurrentThread(),
//                                       ThreadAmILastThread,
//                                       &LastThread,
//                                       sizeof(LastThread),
//                                       NULL);
//     if ((NT_SUCCESS(Status)) && (LastThread)) ExitProcess(uExitCode);
//
//     /* Notify DLLs and TLS Callbacks of termination */
//     LdrShutdownThread();
//
//     /* Tell the Kernel to free the Stack */
//     NtCurrentTeb()->FreeStackOnTermination = TRUE;
//     NtTerminateThread(NULL, uExitCode);
//
//     /* We should never reach this place */
//     ERROR_FATAL("It should not happen\n");
//     while (TRUE); /* 'noreturn' function */
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// OpenThread(IN DWORD dwDesiredAccess,
//            IN BOOL bInheritHandle,
//            IN DWORD dwThreadId)
// {
//     NTSTATUS Status;
//     HANDLE ThreadHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     CLIENT_ID ClientId;
//
//     ClientId.UniqueProcess = 0;
//     ClientId.UniqueThread = ULongToHandle(dwThreadId);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                (bInheritHandle ? OBJ_INHERIT : 0),
//                                NULL,
//                                NULL);
//
//     Status = NtOpenThread(&ThreadHandle,
//                           dwDesiredAccess,
//                           &ObjectAttributes,
//                           &ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     return ThreadHandle;
// }
//
// /*
//  * @implemented
//  */
// PTEB
// GetTeb(VOID)
// {
//     return NtCurrentTeb();
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SwitchToThread(VOID)
// {
//     return NtYieldExecution() != STATUS_NO_YIELD_PERFORMED;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetCurrentThreadId(VOID)
// {
//     return HandleToUlong(NtCurrentTeb()->ClientId.UniqueThread);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// GetThreadTimes(IN HANDLE hThread,
//                OUT LPFILETIME lpCreationTime,
//                OUT LPFILETIME lpExitTime,
//                OUT LPFILETIME lpKernelTime,
//                OUT LPFILETIME lpUserTime)
// {
//     KERNEL_USER_TIMES KernelUserTimes;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadTimes,
//                                       &KernelUserTimes,
//                                       sizeof(KERNEL_USER_TIMES),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     *lpCreationTime = *(LPFILETIME)&KernelUserTimes.CreateTime;
//     *lpExitTime = *(LPFILETIME)&KernelUserTimes.ExitTime;
//     *lpKernelTime = *(LPFILETIME)&KernelUserTimes.KernelTime;
//     *lpUserTime = *(LPFILETIME)&KernelUserTimes.UserTime;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetThreadContext(IN HANDLE hThread,
//                  OUT LPCONTEXT lpContext)
// {
//     NTSTATUS Status;
//
//     Status = NtGetContextThread(hThread, lpContext);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetThreadContext(IN HANDLE hThread,
//                  IN CONST CONTEXT *lpContext)
// {
//     NTSTATUS Status;
//
//     Status = NtSetContextThread(hThread, (PCONTEXT)lpContext);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetExitCodeThread(IN HANDLE hThread,
//                   OUT LPDWORD lpExitCode)
// {
//     THREAD_BASIC_INFORMATION ThreadBasic;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadBasicInformation,
//                                       &ThreadBasic,
//                                       sizeof(THREAD_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     *lpExitCode = ThreadBasic.ExitStatus;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// ResumeThread(IN HANDLE hThread)
// {
//     ULONG PreviousResumeCount;
//     NTSTATUS Status;
//
//     Status = NtResumeThread(hThread, &PreviousResumeCount);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return -1;
//     }
//
//     return PreviousResumeCount;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TerminateThread(IN HANDLE hThread,
//                 IN DWORD dwExitCode)
// {
//     NTSTATUS Status;
//     PRTL_CRITICAL_SECTION LoaderLock;
//     THREAD_BASIC_INFORMATION ThreadInfo;
//
//     /* Check for invalid thread handle */
//     if (!hThread)
//     {
//         /* Fail if one was passed */
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Get the loader lock */
//     LoaderLock = NtCurrentPeb()->LoaderLock;
//     if (LoaderLock)
//     {
//         /* Get our TID */
//         Status = NtQueryInformationThread(hThread,
//                                           ThreadBasicInformation,
//                                           &ThreadInfo,
//                                           sizeof(ThreadInfo),
//                                           NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* If terminating the current thread, we must not hold the loader lock */
//             if (NtCurrentTeb()->ClientId.UniqueThread == ThreadInfo.ClientId.UniqueThread)
//                 ASSERT(NtCurrentTeb()->ClientId.UniqueThread != LoaderLock->OwningThread);
//         }
//     }
//
//     /* Now terminate the thread */
//     Status = NtTerminateThread(hThread, dwExitCode);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All done */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SuspendThread(IN HANDLE hThread)
// {
//     ULONG PreviousSuspendCount;
//     NTSTATUS Status;
//
//     Status = NtSuspendThread(hThread, &PreviousSuspendCount);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return -1;
//     }
//
//     return PreviousSuspendCount;
// }
//
// /*
//  * @implemented
//  */
// DWORD_PTR
// WINAPI
// SetThreadAffinityMask(IN HANDLE hThread,
//                       IN DWORD_PTR dwThreadAffinityMask)
// {
//     THREAD_BASIC_INFORMATION ThreadBasic;
//     KAFFINITY AffinityMask;
//     NTSTATUS Status;
//
//     AffinityMask = (KAFFINITY)dwThreadAffinityMask;
//
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadBasicInformation,
//                                       &ThreadBasic,
//                                       sizeof(THREAD_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     Status = NtSetInformationThread(hThread,
//                                     ThreadAffinityMask,
//                                     &AffinityMask,
//                                     sizeof(KAFFINITY));
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         ThreadBasic.AffinityMask = 0;
//     }
//
//     return ThreadBasic.AffinityMask;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetThreadPriority(IN HANDLE hThread,
//                   IN int nPriority)
// {
//     LONG Prio = nPriority;
//     NTSTATUS Status;
//
//     /* Check if values forcing saturation should be used */
//     if (Prio == THREAD_PRIORITY_TIME_CRITICAL)
//     {
//         /* This is 16 */
//         Prio = (HIGH_PRIORITY + 1) / 2;
//     }
//     else if (Prio == THREAD_PRIORITY_IDLE)
//     {
//         /* This is -16 */
//         Prio = -((HIGH_PRIORITY + 1) / 2);
//     }
//
//     /* Set the Base Priority */
//     Status = NtSetInformationThread(hThread,
//                                     ThreadBasePriority,
//                                     &Prio,
//                                     sizeof(LONG));
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Return */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// int
// WINAPI
// GetThreadPriority(IN HANDLE hThread)
// {
//     THREAD_BASIC_INFORMATION ThreadBasic;
//     NTSTATUS Status;
//
//     /* Query the Base Priority Increment */
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadBasicInformation,
//                                       &ThreadBasic,
//                                       sizeof(THREAD_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failure */
//         BaseSetLastNTError(Status);
//         return THREAD_PRIORITY_ERROR_RETURN;
//     }
//
//     /* Do some conversions for saturation values */
//     if (ThreadBasic.BasePriority == ((HIGH_PRIORITY + 1) / 2))
//     {
//         /* Win32 calls this "time critical" */
//         ThreadBasic.BasePriority = THREAD_PRIORITY_TIME_CRITICAL;
//     }
//     else if (ThreadBasic.BasePriority == -((HIGH_PRIORITY + 1) / 2))
//     {
//         /* Win32 calls this "idle" */
//         ThreadBasic.BasePriority = THREAD_PRIORITY_IDLE;
//     }
//
//     /* Return the final result */
//     return ThreadBasic.BasePriority;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetThreadPriorityBoost(IN HANDLE hThread,
//                        OUT PBOOL pDisablePriorityBoost)
// {
//     ULONG PriorityBoost;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadPriorityBoost,
//                                       &PriorityBoost,
//                                       sizeof(ULONG),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     *pDisablePriorityBoost = PriorityBoost;
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// SetThreadPriorityBoost(IN HANDLE hThread,
//                        IN BOOL bDisablePriorityBoost)
// {
//     ULONG PriorityBoost;
//     NTSTATUS Status;
//
//     PriorityBoost = bDisablePriorityBoost != FALSE;
//
//     Status = NtSetInformationThread(hThread,
//                                     ThreadPriorityBoost,
//                                     &PriorityBoost,
//                                     sizeof(ULONG));
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetThreadSelectorEntry(IN HANDLE hThread,
//                        IN DWORD dwSelector,
//                        OUT LPLDT_ENTRY lpSelectorEntry)
// {
// #ifdef _M_IX86
//     DESCRIPTOR_TABLE_ENTRY DescriptionTableEntry;
//     NTSTATUS Status;
//
//     /* Set the selector and do the query */
//     DescriptionTableEntry.Selector = dwSelector;
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadDescriptorTableEntry,
//                                       &DescriptionTableEntry,
//                                       sizeof(DESCRIPTOR_TABLE_ENTRY),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success, return the selector */
//     *lpSelectorEntry = DescriptionTableEntry.Descriptor;
//     return TRUE;
// #else
//     DPRINT1("Calling GetThreadSelectorEntry!\n");
//     return FALSE;
// #endif
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetThreadIdealProcessor(IN HANDLE hThread,
//                         IN DWORD dwIdealProcessor)
// {
//     NTSTATUS Status;
//
//     Status = NtSetInformationThread(hThread,
//                                     ThreadIdealProcessor,
//                                     &dwIdealProcessor,
//                                     sizeof(ULONG));
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return -1;
//     }
//
//     return (DWORD)Status;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetProcessIdOfThread(IN HANDLE Thread)
// {
//     THREAD_BASIC_INFORMATION ThreadBasic;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationThread(Thread,
//                                       ThreadBasicInformation,
//                                       &ThreadBasic,
//                                       sizeof(THREAD_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     return HandleToUlong(ThreadBasic.ClientId.UniqueProcess);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetThreadId(IN HANDLE Thread)
// {
//     THREAD_BASIC_INFORMATION ThreadBasic;
//     NTSTATUS Status;
//
//     Status = NtQueryInformationThread(Thread,
//                                       ThreadBasicInformation,
//                                       &ThreadBasic,
//                                       sizeof(THREAD_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     return HandleToUlong(ThreadBasic.ClientId.UniqueThread);
// }
//
// /*
//  * @unimplemented
//  */
// LANGID
// WINAPI
// SetThreadUILanguage(IN LANGID LangId)
// {
// #if (NTDDI_VERSION < NTDDI_LONGHORN)
//     /* We only support LangId == 0, for selecting a language
//      * identifier that best supports the NT Console. */
//     if (LangId != 0)
//     {
//         BaseSetLastNTError(STATUS_NOT_SUPPORTED);
//         return 0;
//     }
// #endif
//
//     UNIMPLEMENTED;
//
//     return LANGIDFROMLCID(NtCurrentTeb()->CurrentLocale);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// QueueUserAPC(IN PAPCFUNC pfnAPC,
//              IN HANDLE hThread,
//              IN ULONG_PTR dwData)
// {
//     NTSTATUS Status;
//     ACTIVATION_CONTEXT_BASIC_INFORMATION ActCtxInfo;
//
//     /* Zero the activation context and query information on it */
//     RtlZeroMemory(&ActCtxInfo, sizeof(ActCtxInfo));
//     Status = RtlQueryInformationActivationContext(RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT,
//                                                   NULL,
//                                                   0,
//                                                   ActivationContextBasicInformation,
//                                                   &ActCtxInfo,
//                                                   sizeof(ActCtxInfo),
//                                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail due to SxS */
//         DbgPrint("SXS: %s failing because RtlQueryInformationActivationContext()"
//                  "returned status %08lx\n", __FUNCTION__, Status);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Queue the APC */
//     Status = NtQueueApcThread(hThread,
//                               (PKNORMAL_ROUTINE)BaseDispatchApc,
//                               pfnAPC,
//                               (PVOID)dwData,
//                               (ActCtxInfo.dwFlags & 1) ?
//                               INVALID_ACTIVATION_CONTEXT : ActCtxInfo.hActCtx);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// SetThreadStackGuarantee(IN OUT PULONG StackSizeInBytes)
// {
//     PTEB Teb = NtCurrentTeb();
//     ULONG GuaranteedStackBytes;
//     ULONG AllocationSize;
//
//     if (!StackSizeInBytes)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     AllocationSize = *StackSizeInBytes;
//
//     /* Retrieve the current stack size */
//     GuaranteedStackBytes = Teb->GuaranteedStackBytes;
//
//     /* Return the size of the previous stack */
//     *StackSizeInBytes = GuaranteedStackBytes;
//
//     /*
//      * If the new stack size is either zero or is less than the current size,
//      * the previous stack size is returned and we return success.
//      */
//     if ((AllocationSize == 0) || (AllocationSize < GuaranteedStackBytes))
//     {
//         return TRUE;
//     }
//
//     // FIXME: Unimplemented!
//     UNIMPLEMENTED_ONCE;
//
//     // Temporary HACK for supporting applications!
//     return TRUE; // FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetThreadIOPendingFlag(IN HANDLE hThread,
//                        OUT PBOOL lpIOIsPending)
// {
//     ULONG IoPending;
//     NTSTATUS Status;
//
//     /* Query the flag */
//     Status = NtQueryInformationThread(hThread,
//                                       ThreadIsIoPending,
//                                       &IoPending,
//                                       sizeof(IoPending),
//                                       NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the flag */
//         *lpIOIsPending = IoPending ? TRUE : FALSE;
//         return TRUE;
//     }
//
//     /* Fail */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueueUserWorkItem(IN LPTHREAD_START_ROUTINE Function,
//                   IN PVOID Context,
//                   IN ULONG Flags)
// {
//     NTSTATUS Status;
//
//     /* NOTE: Rtl needs to safely call the function using a trampoline */
//     Status = RtlQueueWorkItem((WORKERCALLBACKFUNC)Function, Context, Flags);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// TlsAlloc(VOID)
// {
//     ULONG Index;
//     PTEB Teb;
//     PPEB Peb;
//
//     /* Get the PEB and TEB, lock the PEB */
//     Teb = NtCurrentTeb();
//     Peb = Teb->ProcessEnvironmentBlock;
//     RtlAcquirePebLock();
//
//     /* Try to get regular TEB slot */
//     Index = RtlFindClearBitsAndSet(Peb->TlsBitmap, 1, 0);
//     if (Index != 0xFFFFFFFF)
//     {
//         /* Clear the value. */
//         Teb->TlsSlots[Index] = 0;
//         RtlReleasePebLock();
//         return Index;
//     }
//
//     /* If it fails, try to find expansion TEB slot. */
//     Index = RtlFindClearBitsAndSet(Peb->TlsExpansionBitmap, 1, 0);
//     if (Index != 0xFFFFFFFF)
//     {
//         /* Is there no expansion slot yet? */
//         if (!Teb->TlsExpansionSlots)
//         {
//             /* Allocate an array */
//             Teb->TlsExpansionSlots = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                      HEAP_ZERO_MEMORY,
//                                                      TLS_EXPANSION_SLOTS *
//                                                      sizeof(PVOID));
//         }
//
//         /* Did we get an array? */
//         if (!Teb->TlsExpansionSlots)
//         {
//             /* Fail */
//             RtlClearBits(Peb->TlsExpansionBitmap, Index, 1);
//             Index = 0xFFFFFFFF;
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//         }
//         else
//         {
//             /* Clear the value. */
//             Teb->TlsExpansionSlots[Index] = 0;
//             Index += TLS_MINIMUM_AVAILABLE;
//         }
//     }
//     else
//     {
//         /* Fail */
//         BaseSetLastNTError(STATUS_NO_MEMORY);
//     }
//
//     /* Release the lock and return */
//     RtlReleasePebLock();
//     return Index;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TlsFree(IN DWORD Index)
// {
//     BOOL BitSet;
//     PPEB Peb;
//     ULONG TlsIndex;
//     PVOID TlsBitmap;
//     NTSTATUS Status;
//
//     /* Acquire the PEB lock and grab the PEB */
//     Peb = NtCurrentPeb();
//     RtlAcquirePebLock();
//
//     /* Check if the index is too high */
//     if (Index >= TLS_MINIMUM_AVAILABLE)
//     {
//         /* Check if it can fit in the expansion slots */
//         TlsIndex = Index - TLS_MINIMUM_AVAILABLE;
//         if (TlsIndex >= TLS_EXPANSION_SLOTS)
//         {
//             /* It's invalid */
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             RtlReleasePebLock();
//             return FALSE;
//         }
//         else
//         {
//             /* Use the expansion bitmap */
//             TlsBitmap = Peb->TlsExpansionBitmap;
//             Index = TlsIndex;
//         }
//     }
//     else
//     {
//         /* Use the normal bitmap */
//         TlsBitmap = Peb->TlsBitmap;
//     }
//
//     /* Check if the index was set */
//     BitSet = RtlAreBitsSet(TlsBitmap, Index, 1);
//     if (BitSet)
//     {
//         /* Tell the kernel to free the TLS cells */
//         Status = NtSetInformationThread(NtCurrentThread(),
//                                         ThreadZeroTlsCell,
//                                         &Index,
//                                         sizeof(DWORD));
//         if (!NT_SUCCESS(Status))
//         {
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             RtlReleasePebLock();
//             return FALSE;
//         }
//
//         /* Clear the bit */
//         RtlClearBits(TlsBitmap, Index, 1);
//     }
//     else
//     {
//         /* Fail */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         RtlReleasePebLock();
//         return FALSE;
//     }
//
//     /* Done! */
//     RtlReleasePebLock();
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// LPVOID
// WINAPI
// TlsGetValue(IN DWORD Index)
// {
//     PTEB Teb;
//
//     /* Get the TEB and clear the last error */
//     Teb = NtCurrentTeb();
//     Teb->LastErrorValue = 0;
//
//     /* Check for simple TLS index */
//     if (Index < TLS_MINIMUM_AVAILABLE)
//     {
//         /* Return it */
//         return Teb->TlsSlots[Index];
//     }
//
//     /* Check for valid index */
//     if (Index >= TLS_EXPANSION_SLOTS + TLS_MINIMUM_AVAILABLE)
//     {
//         /* Fail */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return NULL;
//     }
//
//     /* The expansion slots are allocated on demand, so check for it. */
//     Teb->LastErrorValue = 0;
//     if (!Teb->TlsExpansionSlots) return NULL;
//
//     /* Return the value from the expansion slots */
//     return Teb->TlsExpansionSlots[Index - TLS_MINIMUM_AVAILABLE];
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TlsSetValue(IN DWORD Index,
//             IN LPVOID Value)
// {
//     DWORD TlsIndex;
//     PTEB Teb = NtCurrentTeb();
//
//     /* Check for simple TLS index */
//     if (Index < TLS_MINIMUM_AVAILABLE)
//     {
//         /* Return it */
//         Teb->TlsSlots[Index] = Value;
//         return TRUE;
//     }
//
//     /* Check if this is an expansion slot */
//     TlsIndex = Index - TLS_MINIMUM_AVAILABLE;
//     if (TlsIndex >= TLS_EXPANSION_SLOTS)
//     {
//         /* Fail */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Do we not have expansion slots? */
//     if (!Teb->TlsExpansionSlots)
//     {
//         /* Get the PEB lock to see if we still need them */
//         RtlAcquirePebLock();
//         if (!Teb->TlsExpansionSlots)
//         {
//             /* Allocate them */
//             Teb->TlsExpansionSlots = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                      HEAP_ZERO_MEMORY,
//                                                      TLS_EXPANSION_SLOTS *
//                                                      sizeof(PVOID));
//             if (!Teb->TlsExpansionSlots)
//             {
//                 /* Fail */
//                 RtlReleasePebLock();
//                 BaseSetLastNTError(STATUS_NO_MEMORY);
//                 return FALSE;
//             }
//         }
//
//         /* Release the lock */
//         RtlReleasePebLock();
//     }
//
//     /* Write the value */
//     Teb->TlsExpansionSlots[TlsIndex] = Value;
//
//     /* Success */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
