use ntkrnl_sys::ddk::*;
// ke globals across the crate boundary as link symbols (#[no_mangle] in
// ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeMaximumIncrement: ntkrnl_sys::base::ULONG;
}
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY,
    MEM_RELEASE, PASSIVE_LEVEL, APC_LEVEL, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::PsInitialSystemProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
// The checked-in bindgen still declares the PRE-merge two-parameter forms of these
// (Process, Job) / (Job, Process); upstream simplified both to take only PEPROCESS
// (ntoskrnl/include/internal/ps.h). Import the crate-local definitions explicitly so
// the stale externs from the ddk glob cannot win name resolution here.
use crate::job::{PspExitProcessFromJob, PspRemoveProcessFromJob};

// Bugcheck codes
const CRITICAL_PROCESS_DIED: ULONG = 0xEF;
const CRITICAL_OBJECT_TERMINATION: ULONG = 0xF4;
const CID_HANDLE_DELETION: ULONG = 0x17;
const INVALID_PROCESS_ATTACH_ATTEMPT: ULONG = 0x05;
const ACTIVE_EX_WORKER_THREAD_TERMINATION: ULONG = 0xE9;
const KERNEL_APC_PENDING_DURING_EXIT: ULONG = 0x20;
const KERNEL_STACK_LOCKED_AT_EXIT: ULONG = 0x94;
const DBG_TERMINATE_PROCESS: NTSTATUS = 0x40010004_u32 as i32;

// Pool tags
const TAG_TERMINATE_APC: ULONG = 0x43504154;
const TAG_SEPA: ULONG = u32::from_le_bytes(*b"SePa");

// APC environment
#[allow(non_upper_case_globals)]
const OriginalApcEnvironment: KAPC_ENVIRONMENT = _KAPC_ENVIRONMENT_OriginalApcEnvironment;

// W32 thread callout type
#[allow(non_upper_case_globals)]
const PsW32ThreadCalloutExit: PSW32THREADCALLOUTTYPE = _PSW32THREADCALLOUTTYPE_PsW32ThreadCalloutExit;

#[inline]
unsafe fn RemoveEntryList(entry: *mut _LIST_ENTRY) -> bool {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
    flink == blink
}

#[cfg(target_arch = "aarch64")]
unsafe fn Arm64TraceImageName(prefix: PCSTR, image_name: *const i8, len: usize) {
    let mut buf = [0u8; 32];
    let mut i = 0usize;

    while i < len && i + 1 < buf.len() {
        let ch = *image_name.add(i) as u8;
        if ch == 0 {
            break;
        }
        buf[i] = ch;
        i += 1;
    }
    buf[i] = 0;

    ARM64_UART_TRACE(prefix);
    ARM64_UART_TRACE(buf.as_ptr() as PCSTR);
    ARM64_UART_TRACE(b"\r\n\0".as_ptr() as PCSTR);
}

extern "C" {
    static mut PspActiveProcessMutex: _KGUARDED_MUTEX;
    static mut PspCidTable: PHANDLE_TABLE;
    static mut PsActiveProcessHead: _LIST_ENTRY;
    static PsIdleProcess: PEPROCESS;
    static PsProcessType: POBJECT_TYPE;
    static PsThreadType: POBJECT_TYPE;
    static LpcPortObjectType: POBJECT_TYPE;
    static KdDebuggerEnabled: BOOLEAN;
    static KdDebuggerNotPresent: BOOLEAN;
}

extern "system" {
    fn DbgPrompt(prompt: PCHAR, response: PCHAR, length: ULONG) -> ULONG;
    fn KeBugCheck(code: ULONG);
    fn ExFreePool(ptr: PVOID);
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn DbgBreakPoint();
}
extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}
extern "system" {
    fn KeLowerIrql(irql: KIRQL);
}
extern "system" {
    #[link_name = "IoGetCurrentProcess"]
    fn PsGetCurrentProcess() -> PEPROCESS;
}
extern "C" {
    fn PspSetCrossThreadFlag(thread: PETHREAD, flag: ULONG);
    fn PspSetProcessFlag(process: PEPROCESS, flag: ULONG);
}

// PspW32ProcessCallout and PspW32ThreadCallout are function pointer VARIABLES
// (not functions). In C they're called with the same syntax, but in Rust we
// must access the variable and call through the pointer.
extern "C" {
    static mut PspW32ProcessCallout: PKWIN32_PROCESS_CALLOUT;
    static mut PspW32ThreadCallout: PKWIN32_THREAD_CALLOUT;
}

/// Rust-native InterlockedExchange using atomics (avoids fastcall intrinsic linkage issues on i386)
#[inline]
unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.swap(value, Ordering::SeqCst)
}

#[cfg(target_arch = "aarch64")]
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
}
extern "C" {
    fn ARM64_UART_TRACE(msg: PCSTR);
}
extern "C" {
    fn ARM64_UART_TRACE_HEX(prefix: PCSTR, val: ULONG_PTR);
}
extern "system" {
    fn KeQuerySystemTime(current_time: *mut LARGE_INTEGER);
}

// ============================================================================
// Helper accessors for _KTHREAD fields with different anonymous union numbering
// on i386 vs amd64/arm64.
// ============================================================================

/// Access Tcb.LargeStack (in WaitBlock overlay union)
#[inline]
unsafe fn kthread_large_stack(t: *mut _ETHREAD) -> UCHAR {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_4.LargeStack }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_5.__bindgen_anon_4.LargeStack }
}

/// Access Tcb.CombinedApcDisable
#[inline]
unsafe fn kthread_combined_apc_disable(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_4.CombinedApcDisable }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable }
}

/// Access &mut Tcb.PowerState (in SuspendApc overlay union)
#[inline]
unsafe fn kthread_power_state_ptr(t: *mut _ETHREAD) -> *mut UCHAR {
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*t).Tcb.__bindgen_anon_9.__bindgen_anon_6.PowerState) }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*t).Tcb.__bindgen_anon_7.__bindgen_anon_6.PowerState) }
}

/// Access Tcb.LegoData (in SuspendApc overlay union)
#[inline]
unsafe fn kthread_lego_data(t: *mut _ETHREAD) -> PVOID {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_9.__bindgen_anon_5.LegoData }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_5.LegoData }
}

#[no_mangle]
static mut PspReaperListHead: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
static mut PspReaperWorkItem: _WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
static mut ShortTime: LARGE_INTEGER = unsafe { core::mem::transmute(-10i64 * 100 * 1000) };

unsafe fn PspCatchCriticalBreak(
    message: PCHAR,
    process_or_thread: PVOID,
    image_name: PCHAR,
) {
    let mut action: [u8; 2] = [0; 2];
    let mut handled = 0u8;

    if KdDebuggerEnabled != 0 {
        crate::dbgprint::rust_dbg_print_cstr(message,
            &[crate::dbgprint::IntoSlot::into_slot(process_or_thread),
              crate::dbgprint::IntoSlot::into_slot(image_name)]);
        loop {
            if KdDebuggerNotPresent != 0 {
                break;
            }

            DbgPrompt(
                b"Break, or Ignore (bi)? \0".as_ptr() as PCHAR,
                action.as_mut_ptr() as PCHAR,
                core::mem::size_of_val(&action) as ULONG,
            );

            match action[0] {
                b'B' | b'b' => {
                    DbgBreakPoint();
                    handled = 1u8;
                }
                b'I' | b'i' => {
                    handled = 1u8;
                }
                _ => continue,
            }
            break;
        }
    }

    if handled == 0 {
        // Name the critical process before bugchecking. It was amd64-only, and an
        // i386 0xF4 then left nothing but an EPROCESS address in the log.
        crate::dbg_print!(b"[CRIT-TERM] Process=%p ImageName=%s Message=%s\n\0",
            process_or_thread,
            image_name,
            message,
        );
        KeBugCheckEx(
            CRITICAL_OBJECT_TERMINATION,
            (*(process_or_thread as *mut _EPROCESS)).Pcb.Header.__bindgen_anon_1.__bindgen_anon_1.Type as ULONG_PTR,
            process_or_thread as ULONG_PTR,
            image_name as ULONG_PTR,
            message as ULONG_PTR,
        );
    }
}

unsafe fn PspTerminateProcess(process: PEPROCESS, exit_status: NTSTATUS) -> NTSTATUS {
    let mut thread: PETHREAD;
    let mut status = STATUS_NOTHING_TO_TERMINATE;

    if (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.BreakOnTermination() != 0 {
        #[cfg(target_arch = "x86_64")]
        {
            crate::dbg_print!(b"[CRIT-TERM] PspTerminateProcess: process=%p image=%s exit_status=0x%lx\n\0",
                process as PVOID,
                (*(process as *mut _EPROCESS)).ImageFileName.as_ptr() as PCHAR,
                exit_status as ULONG_PTR,
            );
        }
        PspCatchCriticalBreak(
            b"Terminating critical process 0x%p (%s)\n\0".as_ptr() as PCHAR,
            process as PVOID,
            (*(process as *mut _EPROCESS)).ImageFileName.as_ptr() as PCHAR,
        );
    }

    core::sync::atomic::AtomicU32::from_ptr(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags) as *mut u32
    )
    .fetch_or(0x8u32, core::sync::atomic::Ordering::SeqCst);

    thread = PsGetNextProcessThread(process, core::ptr::null_mut());
    while !thread.is_null() {
        PspTerminateThreadByPointer(thread, exit_status, 0u8);
        thread = PsGetNextProcessThread(process, thread);
        status = STATUS_SUCCESS;
    }

    if status == STATUS_NOTHING_TO_TERMINATE || !(*(process as *mut _EPROCESS)).DebugPort.is_null() {
        ObClearProcessHandleTable(process);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsTerminateProcess(
    process: PEPROCESS,
    exit_status: NTSTATUS,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    PspTerminateProcess(process, exit_status)
}

unsafe fn PspShutdownProcessManager() {
    let mut process = PsGetNextProcess(core::ptr::null_mut());
    while !process.is_null() {
        if process != PsInitialSystemProcess as PEPROCESS && process != PsIdleProcess {
            PspTerminateProcess(process, STATUS_SYSTEM_SHUTDOWN);
        }
        process = PsGetNextProcess(process);
    }
}

unsafe extern "system" fn PspExitApcRundown(apc: PKAPC) {
    ExFreePool(apc as PVOID);
}

#[no_mangle]
unsafe extern "system" fn PspReapRoutine(context: PVOID) {
    let mut next_entry: PSINGLE_LIST_ENTRY;
    let mut thread: PETHREAD;

    loop {
        next_entry = InterlockedExchangePointer(
            core::ptr::addr_of_mut!(PspReaperListHead.Flink) as *mut PVOID,
            1 as PVOID,
        ) as PSINGLE_LIST_ENTRY;

        loop {
            let entry_addr = next_entry as usize;
            let offset = core::mem::offset_of!(_ETHREAD, __bindgen_anon_3) + core::mem::offset_of!(_ETHREAD__bindgen_ty_3, ReaperLink);
            thread = (entry_addr - offset) as PETHREAD;

            // SMP kernel-stack teardown race (field manual §9.88 Bug 2 /
            // §9.94): KeTerminateThread queues this reaper entry (and sets
            // SwapBusy) while the dying thread is STILL RUNNING on the stack
            // we're about to free — its final SwapContext clears SwapBusy only
            // once its CPU is off the stack.  Freeing before that leaves the
            // dying CPU executing on released system PTEs → 0x1E
            // STATUS_IN_PAGE_ERROR on a kernel-stack VA.  Spin until the
            // context swap completes (Acquire pairs with the Release clear in
            // KiSwapContextExit / SwapContext).  UP: SwapBusy is never set —
            // the load returns 0 immediately.
            {
                let busy = core::sync::atomic::AtomicU8::from_ptr(
                    core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb.SwapBusy),
                );
                while busy.load(core::sync::atomic::Ordering::Acquire) != 0 {
                    core::hint::spin_loop();
                }
            }

            MmDeleteKernelStack(
                (*(thread as *mut _ETHREAD)).Tcb.StackBase as PVOID,
                kthread_large_stack(thread as *mut _ETHREAD),
            );
            (*(thread as *mut _ETHREAD)).Tcb.InitialStack = core::ptr::null_mut();

            next_entry = (*next_entry).Next;

            ObDereferenceObject(thread as PVOID);

            if next_entry.is_null() || next_entry == 1 as PSINGLE_LIST_ENTRY {
                break;
            }
        }

        if InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(PspReaperListHead.Flink) as *mut PVOID,
            core::ptr::null_mut(),
            1 as PVOID,
        ) == 1 as PVOID
        {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PspDeleteProcess(object_body: PVOID) {
    let object_body = opaque_ptr(object_body);
    let process = object_body as PEPROCESS;
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();

    if !(*(process as *mut _EPROCESS)).ActiveProcessLinks.Flink.is_null() {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));
        RemoveEntryList(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ActiveProcessLinks));
        (*(process as *mut _EPROCESS)).ActiveProcessLinks.Flink = core::ptr::null_mut();
        (*(process as *mut _EPROCESS)).ActiveProcessLinks.Blink = core::ptr::null_mut();
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));
    }

    if !(*(process as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName.is_null() {
        ExFreePoolWithTag(
            (*(process as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName as PVOID,
            TAG_SEPA,
        );
        (*(process as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName = core::ptr::null_mut();
    }

    if !(*(process as *mut _EPROCESS)).Job.is_null() {
        PspRemoveProcessFromJob(process);
        ObDereferenceObject((*(process as *mut _EPROCESS)).Job as PVOID);
        (*(process as *mut _EPROCESS)).Job = core::ptr::null_mut();
    }

    (*(process as *mut _EPROCESS)).Pcb.StackCount += 1;

    if !(*(process as *mut _EPROCESS)).DebugPort.is_null() {
        ObDereferenceObject((*(process as *mut _EPROCESS)).DebugPort);
        (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();
    }

    if !(*(process as *mut _EPROCESS)).ExceptionPort.is_null() {
        ObDereferenceObject((*(process as *mut _EPROCESS)).ExceptionPort);
        (*(process as *mut _EPROCESS)).ExceptionPort = core::ptr::null_mut();
    }

    if !(*(process as *mut _EPROCESS)).SectionObject.is_null() {
        ObDereferenceObject((*(process as *mut _EPROCESS)).SectionObject);
        (*(process as *mut _EPROCESS)).SectionObject = core::ptr::null_mut();
    }

    if !(*(process as *mut _EPROCESS)).ObjectTable.is_null() {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
            core::ptr::addr_of_mut!(apc_state),
        );
        ObKillProcess(process);
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(apc_state));
    }

    if (*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags & 0x1 != 0 {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
            core::ptr::addr_of_mut!(apc_state),
        );
        PspExitProcess(0u8, process);
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(apc_state));
        MmDeleteProcessAddressSpace(process);
    }

    if !(*(process as *mut _EPROCESS)).UniqueProcessId.is_null() {
        if ExDestroyHandle(
            PspCidTable,
            (*(process as *mut _EPROCESS)).UniqueProcessId,
            core::ptr::null_mut(),
        ) == 0
        {
            KeBugCheck(CID_HANDLE_DELETION);
        }
    }

    PspDeleteProcessSecurity(process);

    if !(*(process as *mut _EPROCESS)).WorkingSetWatch.is_null() {
        ExFreePool((*(process as *mut _EPROCESS)).WorkingSetWatch as PVOID);
        PsReturnProcessNonPagedPoolQuota(process, 0x2000);
    }

    ObDereferenceDeviceMap(process);
    PspDereferenceQuotaBlock(process, (*(process as *mut _EPROCESS)).QuotaBlock);
}

#[no_mangle]
pub unsafe extern "system" fn PspDeleteThread(object_body: PVOID) {
    let object_body = opaque_ptr(object_body);
    let thread = object_body as PETHREAD;
    let process = (*(thread as *mut _ETHREAD)).ThreadsProcess;

    if !(*(thread as *mut _ETHREAD)).Tcb.InitialStack.is_null() {
        // Same context-swap guard as PspReapRoutine (§9.88 Bug 2): never free
        // a stack a CPU may still be walking off.  Threads that never ran
        // (failed creation) have SwapBusy 0 — the load passes immediately.
        {
            let busy = core::sync::atomic::AtomicU8::from_ptr(
                core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb.SwapBusy),
            );
            while busy.load(core::sync::atomic::Ordering::Acquire) != 0 {
                core::hint::spin_loop();
            }
        }

        MmDeleteKernelStack(
            (*(thread as *mut _ETHREAD)).Tcb.StackBase as PVOID,
            kthread_large_stack(thread as *mut _ETHREAD),
        );
    }

    if !(*(thread as *mut _ETHREAD)).Cid.UniqueThread.is_null() {
        if ExDestroyHandle(
            PspCidTable,
            (*(thread as *mut _ETHREAD)).Cid.UniqueThread,
            core::ptr::null_mut(),
        ) == 0
        {
            KeBugCheck(CID_HANDLE_DELETION);
        }
    }

    PspDeleteThreadSecurity(thread);

    if process.is_null() {
        return;
    }

    if !(*(thread as *mut _ETHREAD)).ThreadListEntry.Flink.is_null() {
        KeEnterCriticalRegion();
        ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ProcessLock));
        RemoveEntryList(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).ThreadListEntry));
        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ProcessLock));
        KeLeaveCriticalRegion();
    }

    ObDereferenceObject(process as PVOID);
}

unsafe fn PspExitThread(exit_status: NTSTATUS) {
    ARM64_UART_TRACE(b"[PS] PspExitThread entered\r\n\0".as_ptr() as PCSTR);
    #[cfg(target_arch = "x86_64")]
    {
        let _proc = ntkrnl_sys::ps::PsGetCurrentProcess();
        let _img = (*((_proc) as *mut _EPROCESS)).ImageFileName.as_ptr();
        let _crit = (*(_proc as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.BreakOnTermination();
        crate::dbg_print!(b"[EXIT-THR] image=%s exit_status=0x%lx crit=%d\n\0",
            _img as PCHAR,
            exit_status as ULONG_PTR,
            _crit as u32,
        );
    }
    let mut termination_msg: CLIENT_DIED_MSG = core::mem::zeroed();
    let mut status: NTSTATUS;
    let teb: *mut u8;
    let current_process: PEPROCESS;
    let thread: PETHREAD;
    let mut other_thread: PETHREAD;
    let mut previous_thread: PETHREAD = core::ptr::null_mut();
    let mut deallocation_stack: PVOID;
    let mut dummy: usize = 0;
    let mut last = 0u8;
    let mut termination_port: PTERMINATION_PORT;
    let mut next_port: PTERMINATION_PORT;
    let mut first_entry: PLIST_ENTRY;
    let mut current_entry: PLIST_ENTRY;
    let mut apc: PKAPC;
    let primary_token: PVOID;

    thread = PsGetCurrentThread();
    current_process = (*(thread as *mut _ETHREAD)).ThreadsProcess;

    #[cfg(target_arch = "aarch64")]
    {
        ARM64_UART_TRACE_HEX(b"[PS] PspExitThread: Process=\0".as_ptr() as PCSTR, current_process as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[PS] PspExitThread: Thread=\0".as_ptr() as PCSTR, thread as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[PS] PspExitThread: ExitStatus=\0".as_ptr() as PCSTR, exit_status as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[PS] PspExitThread: Teb=\0".as_ptr() as PCSTR, (*(thread as *mut _ETHREAD)).Tcb.Teb as ULONG_PTR);
        ARM64_UART_TRACE_HEX(
            b"[PS] PspExitThread: StartAddress=\0".as_ptr() as PCSTR,
            (*(thread as *mut _ETHREAD))
                .__bindgen_anon_6
                .StartAddress
                .map(|f| f as ULONG_PTR)
                .unwrap_or(0),
        );
        Arm64TraceImageName(
            b"[PS] PspExitThread: Image=\0".as_ptr() as PCSTR,
            (*(current_process as *mut _EPROCESS)).ImageFileName.as_ptr(),
            core::mem::size_of_val(&(*(current_process as *mut _EPROCESS)).ImageFileName),
        );
    }

    if KeIsAttachedProcess() != 0 {
        KeBugCheckEx(
            INVALID_PROCESS_ATTACH_ATTEMPT,
            current_process as ULONG_PTR,
            (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.ApcState.Process as ULONG_PTR,
            (*(thread as *mut _ETHREAD)).Tcb.ApcStateIndex as ULONG_PTR,
            thread as ULONG_PTR,
        );
    }

    KeLowerIrql(PASSIVE_LEVEL as u8);

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_8.__bindgen_anon_1.ActiveExWorker() != 0 {
        KeBugCheckEx(
            ACTIVE_EX_WORKER_THREAD_TERMINATION,
            thread as ULONG_PTR,
            0,
            0,
            0,
        );
    }

    if kthread_combined_apc_disable(thread as *mut _ETHREAD) != 0 {
        KeBugCheckEx(
            KERNEL_APC_PENDING_DURING_EXIT,
            0,
            kthread_combined_apc_disable(thread as *mut _ETHREAD) as ULONG_PTR,
            0,
            1,
        );
    }

    ExWaitForRundownProtectionRelease(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).RundownProtect));
    PopCleanupPowerState(kthread_power_state_ptr(thread as *mut _ETHREAD) as PPOWER_STATE);
    PspRunCreateThreadNotifyRoutines(thread, 0u8);

    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).ProcessLock));

    let active_threads = &mut (*(current_process as *mut _EPROCESS)).ActiveThreads;
    *active_threads -= 1;
    if *active_threads == 0 {
        core::sync::atomic::AtomicU32::from_ptr(
            core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).__bindgen_anon_2.Flags) as *mut u32
        )
        .fetch_or(0x8u32, core::sync::atomic::Ordering::SeqCst);
        last = 1u8;

        if exit_status == STATUS_THREAD_IS_TERMINATING {
            if (*(current_process as *mut _EPROCESS)).ExitStatus == STATUS_PENDING {
                (*(current_process as *mut _EPROCESS)).ExitStatus =
                    (*(current_process as *mut _EPROCESS)).LastThreadExitStatus;
            }
        } else {
            (*(current_process as *mut _EPROCESS)).ExitStatus = exit_status;
        }

        first_entry = core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).ThreadListHead);
        current_entry = (*first_entry).Flink;
        while first_entry != current_entry {
            let entry_addr = current_entry as usize;
            let offset = core::mem::offset_of!(_ETHREAD, ThreadListEntry);
            other_thread = (entry_addr - offset) as PETHREAD;

            if other_thread != thread
                && KeReadStateThread(core::ptr::addr_of_mut!((*(other_thread as *mut _ETHREAD)).Tcb)) == 0
                && ObReferenceObjectSafe(other_thread as PVOID) != 0
            {
                ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).ProcessLock));
                KeLeaveCriticalRegion();

                KeWaitForSingleObject(
                    other_thread as PVOID,
                    _KWAIT_REASON_Executive,
                    KernelMode,
                    0u8,
                    core::ptr::null_mut(),
                );

                if !previous_thread.is_null() {
                    ObDereferenceObject(previous_thread as PVOID);
                }

                previous_thread = other_thread;
                KeEnterCriticalRegion();
                ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).ProcessLock));
            }

            current_entry = (*current_entry).Flink;
        }
    } else if exit_status != STATUS_THREAD_IS_TERMINATING {
        (*(current_process as *mut _EPROCESS)).LastThreadExitStatus = exit_status;
    }

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).ProcessLock));
    KeLeaveCriticalRegion();

    if !previous_thread.is_null() {
        ObDereferenceObject(previous_thread as PVOID);
    }

    if !(*(current_process as *mut _EPROCESS)).DebugPort.is_null()
        && (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0
    {
        if last != 0 {
            DbgkExitProcess((*(current_process as *mut _EPROCESS)).ExitStatus);
        } else {
            DbgkExitThread(exit_status);
        }
    }

    if KdDebuggerEnabled != 0 && (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.BreakOnTermination() != 0 {
        PspCatchCriticalBreak(
            b"Critical thread 0x%p (in %s) exited\n\0".as_ptr() as PCHAR,
            thread as PVOID,
            (*(current_process as *mut _EPROCESS)).ImageFileName.as_ptr() as PCHAR,
        );
    }

    if last != 0 && (*(current_process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.BreakOnTermination() != 0 {
        if KdDebuggerEnabled != 0 {
            PspCatchCriticalBreak(
                b"Critical process 0x%p (%s) exited\n\0".as_ptr() as PCHAR,
                current_process as PVOID,
                (*(current_process as *mut _EPROCESS)).ImageFileName.as_ptr() as PCHAR,
            );
        } else {
            KeBugCheckEx(CRITICAL_PROCESS_DIED, current_process as ULONG_PTR, 0, 0, 0);
        }
    }

    termination_port = (*(thread as *mut _ETHREAD)).__bindgen_anon_3.TerminationPort;
    if !termination_port.is_null() {
        termination_msg.h.u2.ZeroInit = 0;
        termination_msg.h.u2.s2.Type = 0x04;
        termination_msg.h.u1.s1.TotalLength = core::mem::size_of::<CLIENT_DIED_MSG>() as CSHORT;
        termination_msg.h.u1.s1.DataLength =
            (core::mem::size_of::<CLIENT_DIED_MSG>() - core::mem::size_of::<PORT_MESSAGE>()) as CSHORT;

        loop {
            termination_msg.CreateTime = (*(thread as *mut _ETHREAD)).CreateTime;

            loop {
                status = LpcRequestPort(
                    (*(termination_port as *mut _TERMINATION_PORT)).Port,
                    core::ptr::addr_of_mut!(termination_msg.h),
                );
                if status == STATUS_NO_MEMORY || status == STATUS_INSUFFICIENT_RESOURCES {
                    KeDelayExecutionThread(KernelMode, 0u8, core::ptr::addr_of_mut!(ShortTime));
                    continue;
                }
                break;
            }

            ObDereferenceObject((*(termination_port as *mut _TERMINATION_PORT)).Port);
            next_port = (*(termination_port as *mut _TERMINATION_PORT)).Next;
            ExFreePoolWithTag(termination_port as PVOID, 0x5054733D);
            termination_port = next_port;

            if termination_port.is_null() {
                break;
            }
        }
    } else if (exit_status == STATUS_THREAD_IS_TERMINATING && (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() != 0)
        || (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() == 0
    {
        termination_msg.h.u2.ZeroInit = 0;
        termination_msg.h.u2.s2.Type = 0x04;
        termination_msg.h.u1.s1.TotalLength = core::mem::size_of::<CLIENT_DIED_MSG>() as CSHORT;
        termination_msg.h.u1.s1.DataLength =
            (core::mem::size_of::<CLIENT_DIED_MSG>() - core::mem::size_of::<PORT_MESSAGE>()) as CSHORT;

        if !(*(current_process as *mut _EPROCESS)).ExceptionPort.is_null() {
            termination_msg.CreateTime = (*(thread as *mut _ETHREAD)).CreateTime;

            loop {
                status = LpcRequestPort(
                    (*(current_process as *mut _EPROCESS)).ExceptionPort,
                    core::ptr::addr_of_mut!(termination_msg.h),
                );
                if status == STATUS_NO_MEMORY || status == STATUS_INSUFFICIENT_RESOURCES {
                    KeDelayExecutionThread(KernelMode, 0u8, core::ptr::addr_of_mut!(ShortTime));
                    continue;
                }
                break;
            }
        }
    }

    if !(*(thread as *mut _ETHREAD)).Tcb.Win32Thread.is_null() {
        if let Some(callout) = PspW32ThreadCallout {
            callout(thread, PsW32ThreadCalloutExit);
        }
    }

    if last != 0 && !(*(current_process as *mut _EPROCESS)).Win32Process.is_null() {
        if let Some(callout) = PspW32ProcessCallout {
            callout(current_process, 0u8);
        }
    }

    if (*(thread as *mut _ETHREAD)).Tcb.EnableStackSwap == 0 {
        KeBugCheckEx(KERNEL_STACK_LOCKED_AT_EXIT, 0, 0, 0, 0);
    }

    IoCancelThreadIo(thread);
    ExTimerRundown();
    KeRundownThread();

    teb = (*(thread as *mut _ETHREAD)).Tcb.Teb as *mut u8;
    if !teb.is_null() {
        if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() == 0 {
            let teb_ptr = teb as *mut _TEB;
            if (*teb_ptr).FreeStackOnTermination != 0 {
                dummy = 0;
                deallocation_stack = (*teb_ptr).DeallocationStack;

                ZwFreeVirtualMemory(
                    NtCurrentProcess() as HANDLE,
                    core::ptr::addr_of_mut!(deallocation_stack),
                    core::ptr::addr_of_mut!(dummy) as PSIZE_T,
                    MEM_RELEASE,
                );
            }

            let dbg_handle = (*teb_ptr).DbgSsReserved[1];
            if !dbg_handle.is_null() {
                ObCloseHandle(dbg_handle, UserMode);
            }
        }

        MmDeleteTeb(current_process, teb as PTEB);
        (*(thread as *mut _ETHREAD)).Tcb.Teb = core::ptr::null_mut();
    }

    ARM64_UART_TRACE(b"[PS] PspExitThread: about to call LpcExitThread\r\n\0".as_ptr() as PCSTR);
    LpcExitThread(thread);
    (*(thread as *mut _ETHREAD)).__bindgen_anon_2.ExitStatus = exit_status;
    KeQuerySystemTime(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).__bindgen_anon_1.ExitTime));

    if last != 0 {
        (*(current_process as *mut _EPROCESS)).ExitTime = (*(thread as *mut _ETHREAD)).__bindgen_anon_1.ExitTime;
        PspExitProcess(1u8, current_process);

        primary_token = PsReferencePrimaryToken(current_process);
        if SeDetailedAuditingWithToken(primary_token as PTOKEN) != 0 {
            SeAuditProcessExit(current_process);
        }
        ObFastDereferenceObject(
            core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).Token),
            primary_token,
        );

        if !(*(current_process as *mut _EPROCESS)).VdmObjects.is_null() {}

        ObKillProcess(current_process);

        if !(*(current_process as *mut _EPROCESS)).SectionObject.is_null() {
            ObDereferenceObject((*(current_process as *mut _EPROCESS)).SectionObject);
            (*(current_process as *mut _EPROCESS)).SectionObject = core::ptr::null_mut();
        }

        if !(*(current_process as *mut _EPROCESS)).Job.is_null() {
            PspExitProcessFromJob(current_process);
        }
    }

    KeEnterCriticalRegion();
    (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.__bindgen_anon_1.ApcQueueable = 0u8;
    KeForceResumeThread(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb));
    KeLeaveCriticalRegion();

    first_entry = KeFlushQueueApc(
        core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb),
        UserMode,
    );
    if !first_entry.is_null() {
        current_entry = first_entry;
        loop {
            let entry_addr = current_entry as usize;
            let offset = core::mem::offset_of!(_KAPC, ApcListEntry);
            apc = (entry_addr - offset) as PKAPC;

            current_entry = (*current_entry).Flink;

            if let Some(rundown) = (*(apc as *mut _KAPC)).RundownRoutine {
                rundown(apc);
            } else {
                ExFreePool(apc as PVOID);
            }

            if current_entry == first_entry {
                break;
            }
        }
    }

    if last != 0 {
        MmCleanProcessAddressSpace(current_process);
    }

    if !kthread_lego_data(thread as *mut _ETHREAD).is_null() {
        PspRunLegoRoutine(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb));
    }

    first_entry = KeFlushQueueApc(
        core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb),
        KernelMode,
    );
    if !first_entry.is_null() || kthread_combined_apc_disable(thread as *mut _ETHREAD) != 0 {
        KeBugCheckEx(
            KERNEL_APC_PENDING_DURING_EXIT,
            first_entry as ULONG_PTR,
            kthread_combined_apc_disable(thread as *mut _ETHREAD) as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
        );
    }

    if last != 0 {
        KeSetProcess(
            core::ptr::addr_of_mut!((*(current_process as *mut _EPROCESS)).Pcb),
            0,
            0u8,
        );
    }

    KeTerminateThread(0);
}

#[no_mangle]
unsafe extern "system" fn PsExitSpecialApc(
    apc: PKAPC,
    _normal_routine: *mut PKNORMAL_ROUTINE,
    normal_context: *mut PVOID,
    _system_argument1: *mut PVOID,
    system_argument2: *mut PVOID,
) {
    PsExitSpecialApcInner(apc, normal_context, system_argument2);
}

#[inline(always)]
unsafe fn PsExitSpecialApcInner(
    apc: PKAPC,
    normal_context: *mut PVOID,
    system_argument2: *mut PVOID,
) {
    if !(*system_argument2).is_null() {
        let status = (*normal_context) as NTSTATUS;
        PspExitApcRundown(apc);
        PspExitThread(status);
    }
}

unsafe extern "system" fn PspExitNormalApc(
    normal_context: PVOID,
    system_argument1: PVOID,
    system_argument2: PVOID,
) {
    PspExitNormalApcInner(normal_context, system_argument1, system_argument2);
}

#[inline(always)]
unsafe fn PspExitNormalApcInner(
    normal_context: PVOID,
    system_argument1: PVOID,
    system_argument2: PVOID,
) {
    let apc = system_argument1 as PKAPC;
    let thread = PsGetCurrentThread();

    KeInitializeApc(
        apc,
        core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb),
        OriginalApcEnvironment,
        Some(PsExitSpecialApc),
        Some(PspExitApcRundown),
        Some(PspExitNormalApc),
        UserMode,
        normal_context,
    );

    if KeInsertQueueApc(apc, apc as PVOID, (system_argument2 as usize | 1) as PVOID, 2) == 0 {
        PspExitApcRundown(apc);
    }

    (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.ApcState.UserApcPending = 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn PspTerminateThreadByPointer(
    thread: PETHREAD,
    exit_status: NTSTATUS,
    b_self: BOOLEAN,
) -> NTSTATUS {
    let thread = opaque_ptr(thread);
    let apc: PKAPC;
    let mut status = STATUS_SUCCESS;
    let flags: ULONG;

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.BreakOnTermination() != 0 {
        PspCatchCriticalBreak(
            b"Terminating critical thread 0x%p (in %s)\n\0".as_ptr() as PCHAR,
            thread as PVOID,
            (*(*(thread as *mut _ETHREAD)).ThreadsProcess).ImageFileName.as_ptr() as PCHAR,
        );
    }

    if b_self != 0 || PsGetCurrentThread() == thread {
        PspSetCrossThreadFlag(thread, 0x1u32);
        PspExitThread(exit_status);
    }

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 {
        return STATUS_ACCESS_DENIED;
    }

    apc = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        core::mem::size_of::<_KAPC>() as SIZE_T,
        TAG_TERMINATE_APC,
    ) as PKAPC;
    if apc.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    flags = (*(thread as *mut _ETHREAD)).__bindgen_anon_7.CrossThreadFlags | 0x1u32;

    if InterlockedExchange(
        core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).__bindgen_anon_7.CrossThreadFlags) as *mut LONG,
        flags as LONG,
    ) as u32 & 0x1u32
        == 0
    {
        KeInitializeApc(
            apc,
            core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb),
            OriginalApcEnvironment,
            Some(PsExitSpecialApc),
            Some(PspExitApcRundown),
            Some(PspExitNormalApc),
            KernelMode,
            exit_status as PVOID,
        );

        if KeInsertQueueApc(apc, apc as PVOID, core::ptr::null_mut(), 2) == 0 {
            status = STATUS_UNSUCCESSFUL;
        } else {
            KeForceResumeThread(core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb));
            return status;
        }
    }

    ExFreePoolWithTag(apc as PVOID, TAG_TERMINATE_APC);
    status
}

unsafe fn PspIsProcessExiting(process: PEPROCESS) -> BOOLEAN {
    if (*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags & 0x4u32 != 0 { 1u8 } else { 0u8 }
}

unsafe fn PspExitProcess(last_thread: BOOLEAN, process: PEPROCESS) {
    let mut actual: ULONG = 0;

    core::sync::atomic::AtomicU32::from_ptr(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).__bindgen_anon_2.Flags) as *mut u32
    )
    .fetch_or(0x4u32, core::sync::atomic::Ordering::SeqCst);

    if last_thread != 0 {
        PspRunCreateProcessNotifyRoutines(process, 0u8);
    }

    PopCleanupPowerState(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb.PowerState) as PPOWER_STATE);

    if (*(process as *mut _EPROCESS)).SecurityPort.is_null() {
        (*(process as *mut _EPROCESS)).SecurityPort = 1 as PVOID;
    } else if (*(process as *mut _EPROCESS)).SecurityPort != 1 as PVOID {
        ObDereferenceObject((*(process as *mut _EPROCESS)).SecurityPort);
        (*(process as *mut _EPROCESS)).SecurityPort = 1 as PVOID;
    }

    if last_thread != 0 {
        if (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.SetTimerResolution() != 0 {
            ZwSetTimerResolution(KeMaximumIncrement, 0, core::ptr::addr_of_mut!(actual));
        }

        if !(*(process as *mut _EPROCESS)).Job.is_null() {
            let job_ptr = (*(process as *mut _EPROCESS)).Job as *mut u8;
            let completion_port_offset = 0x100usize;
            let completion_port = *(job_ptr.add(completion_port_offset) as *mut PVOID);
            if !completion_port.is_null() {}
        }
    } else {
        MmCleanProcessAddressSpace(process);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsTerminateSystemThread(exit_status: NTSTATUS) -> NTSTATUS {
    let thread = PsGetCurrentThread();

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    PspTerminateThreadByPointer(thread, exit_status, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn NtTerminateProcess(
    mut process_handle: HANDLE,
    exit_status: NTSTATUS,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let mut status: NTSTATUS;
    let mut process: PEPROCESS = core::ptr::null_mut();
    let current_process = PsGetCurrentProcess();
    let mut thread: PETHREAD;
    let current_thread = PsGetCurrentThread();
    let kill_by_handle: BOOLEAN;

    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut ULONG;
        if !u.is_null() && KeGetPreviousMode() == UserMode {
            let m = b"[PROC-TERM] PID=\0";
            for &c in m.iter().take_while(|&&x| x != 0) {
                *u = c as ULONG;
            }
            let mut pid = PsGetProcessId(current_process) as ULONG;
            let mut buf = [0u8; 12];
            let mut pos = 0;
            if pid == 0 {
                buf[pos] = b'0';
                pos += 1;
            } else {
                let mut t = [0u8; 12];
                let mut tp = 0;
                while pid > 0 {
                    t[tp] = b'0' + (pid % 10) as u8;
                    tp += 1;
                    pid /= 10;
                }
                while tp > 0 {
                    tp -= 1;
                    buf[pos] = t[tp];
                    pos += 1;
                }
            }
            for i in 0..pos {
                *u = buf[i] as ULONG;
            }
            let m2 = b" ExitStatus=0x\0";
            for &c in m2.iter().take_while(|&&x| x != 0) {
                *u = c as ULONG;
            }
            let s = exit_status as ULONG;
            for hi in (0..8).rev() {
                let digit = (s >> (hi * 4)) & 0xF;
                let ch = if digit < 10 { b'0' + digit as u8 } else { b'A' + (digit - 10) as u8 };
                *u = ch as ULONG;
            }
            *u = b'\r' as ULONG;
            *u = b'\n' as ULONG;
        }
    }

    if process_handle.is_null() {
        kill_by_handle = 0u8;
        process_handle = NtCurrentProcess() as HANDLE;
    } else {
        kill_by_handle = 1u8;
    }

    status = ObReferenceObjectByHandle(
        process_handle,
        0x1u32,
        PsProcessType,
        KeGetPreviousMode(),
        core::ptr::addr_of_mut!(process) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.BreakOnTermination() != 0 {
        PspCatchCriticalBreak(
            b"Terminating critical process 0x%p (%s)\n\0".as_ptr() as PCHAR,
            process as PVOID,
            (*(process as *mut _EPROCESS)).ImageFileName.as_ptr() as PCHAR,
        );
    }

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect)) == 0 {
        ObDereferenceObject(process as PVOID);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    if kill_by_handle != 0 {
        PspSetProcessFlag(process, 0x8u32);
    }

    status = STATUS_NOTHING_TO_TERMINATE;
    thread = PsGetNextProcessThread(process, core::ptr::null_mut());
    if !thread.is_null() {
        status = STATUS_SUCCESS;

        loop {
            if thread != current_thread {
                PspTerminateThreadByPointer(thread, exit_status, 0u8);
            }

            thread = PsGetNextProcessThread(process, thread);
            if thread.is_null() {
                break;
            }
        }
    }

    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect));

    if process == current_process {
        if kill_by_handle != 0 {
            ObDereferenceObject(process as PVOID);
            PspTerminateThreadByPointer(current_thread, exit_status, 1u8);
        }
    } else if exit_status == DBG_TERMINATE_PROCESS {
        DbgkClearProcessDebugObject(process, core::ptr::null_mut());
    }

    if status == STATUS_NOTHING_TO_TERMINATE
        || (!(*(process as *mut _EPROCESS)).DebugPort.is_null() && kill_by_handle != 0)
    {
        ObClearProcessHandleTable(process);
        status = STATUS_SUCCESS;
    }

    ObDereferenceObject(process as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtTerminateThread(
    thread_handle: HANDLE,
    exit_status: NTSTATUS,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let current_thread = PsGetCurrentThread();
    let mut status: NTSTATUS;

    if thread_handle.is_null() {
        let proc_ptr = (*(current_thread as *mut _ETHREAD)).ThreadsProcess;
        if (*(proc_ptr as *mut _EPROCESS)).ActiveThreads == 1 {
            return STATUS_CANT_TERMINATE_SELF;
        }

        return PspTerminateThreadByPointer(current_thread, exit_status, 1u8);
    } else if thread_handle == NtCurrentThread() as HANDLE {
        return PspTerminateThreadByPointer(current_thread, exit_status, 1u8);
    }

    status = ObReferenceObjectByHandle(
        thread_handle,
        0x1u32,
        PsThreadType,
        KeGetPreviousMode(),
        core::ptr::addr_of_mut!(thread) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if thread != current_thread {
        status = PspTerminateThreadByPointer(thread, exit_status, 0u8);
        ObDereferenceObject(thread as PVOID);
    } else {
        ObDereferenceObject(thread as PVOID);
        return PspTerminateThreadByPointer(current_thread, exit_status, 1u8);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtRegisterThreadTerminatePort(port_handle: HANDLE) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let mut status: NTSTATUS;
    let termination_port: PTERMINATION_PORT;
    let mut termination_lpc_port: PVOID = core::ptr::null_mut();
    let thread: PETHREAD;

    status = ObReferenceObjectByHandle(
        port_handle,
        0xF0001u32,
        LpcPortObjectType,
        KeGetPreviousMode(),
        core::ptr::addr_of_mut!(termination_lpc_port),
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    termination_port = ExAllocatePoolWithTag(
        _POOL_TYPE_NonPagedPool,
        core::mem::size_of::<_TERMINATION_PORT>() as SIZE_T,
        0x5054733D,
    ) as PTERMINATION_PORT;
    if !termination_port.is_null() {
        thread = PsGetCurrentThread();
        (*(termination_port as *mut _TERMINATION_PORT)).Port = termination_lpc_port;
        (*(termination_port as *mut _TERMINATION_PORT)).Next =
            (*(thread as *mut _ETHREAD)).__bindgen_anon_3.TerminationPort;
        (*(thread as *mut _ETHREAD)).__bindgen_anon_3.TerminationPort = termination_port;
        return STATUS_SUCCESS;
    }

    ObDereferenceObject(termination_lpc_port);
    STATUS_INSUFFICIENT_RESOURCES
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/kill.c
//  * PURPOSE:         Process Manager: Process and Thread Termination
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (xnavara@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY PspReaperListHead = { NULL, NULL };
// WORK_QUEUE_ITEM PspReaperWorkItem;
// LARGE_INTEGER ShortTime = {{-10 * 100 * 1000, -1}};
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// PspCatchCriticalBreak(IN PCHAR Message,
//                       IN PVOID ProcessOrThread,
//                       IN PCHAR ImageName)
// {
//     CHAR Action[2];
//     BOOLEAN Handled = FALSE;
//     PAGED_CODE();
//
//     /* Check if a debugger is enabled */
//     if (KdDebuggerEnabled)
//     {
//         /* Print out the message */
//         DbgPrint(Message, ProcessOrThread, ImageName);
//         do
//         {
//             /* If a debugger isn't present, don't prompt */
//             if (KdDebuggerNotPresent) break;
//
//             /* A debugger is active, prompt for action */
//             DbgPrompt("Break, or Ignore (bi)? ", Action, sizeof(Action));
//             switch (Action[0])
//             {
//                 /* Break */
//                 case 'B': case 'b':
//                     DbgBreakPoint();
//                     /* Fall through */
//
//                 /* Ignore: Handle it */
//                 case 'I': case 'i':
//                     Handled = TRUE;
//
//                 /* Unrecognized: Prompt again */
//                 default:
//                     break;
//             }
//         } while (!Handled);
//     }
//
//     /* Did we ultimately handle this? */
//     if (!Handled)
//     {
//         /* We didn't, bugcheck */
//         KeBugCheckEx(CRITICAL_OBJECT_TERMINATION,
//                      ((PKPROCESS)ProcessOrThread)->Header.Type,
//                      (ULONG_PTR)ProcessOrThread,
//                      (ULONG_PTR)ImageName,
//                      (ULONG_PTR)Message);
//     }
// }
//
// NTSTATUS
// NTAPI
// PspTerminateProcess(IN PEPROCESS Process,
//                     IN NTSTATUS ExitStatus)
// {
//     PETHREAD Thread;
//     NTSTATUS Status = STATUS_NOTHING_TO_TERMINATE;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG,
//             "Process: %p ExitStatus: %d\n", Process, ExitStatus);
//     PSREFTRACE(Process);
//
//     /* Check if this is a Critical Process */
//     if (Process->BreakOnTermination)
//     {
//         /* Break to debugger */
//         PspCatchCriticalBreak("Terminating critical process 0x%p (%s)\n",
//                               Process,
//                               Process->ImageFileName);
//     }
//
//     /* Set the delete flag */
//     InterlockedOr((PLONG)&Process->Flags, PSF_PROCESS_DELETE_BIT);
//
//     /* Get the first thread */
//     Thread = PsGetNextProcessThread(Process, NULL);
//     while (Thread)
//     {
//         /* Kill it */
//         PspTerminateThreadByPointer(Thread, ExitStatus, FALSE);
//         Thread = PsGetNextProcessThread(Process, Thread);
//
//         /* We had at least one thread, so termination is OK */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Check if there was nothing to terminate or if we have a debug port */
//     if ((Status == STATUS_NOTHING_TO_TERMINATE) || (Process->DebugPort))
//     {
//         /* Clear the handle table anyway */
//         ObClearProcessHandleTable(Process);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PsTerminateProcess(IN PEPROCESS Process,
//                    IN NTSTATUS ExitStatus)
// {
//     /* Call the internal API */
//     return PspTerminateProcess(Process, ExitStatus);
// }
//
// VOID
// NTAPI
// PspShutdownProcessManager(VOID)
// {
//     PEPROCESS Process = NULL;
//
//     /* Loop every process */
//     Process = PsGetNextProcess(Process);
//     while (Process)
//     {
//         /* Make sure this isn't the idle or initial process */
//         if ((Process != PsInitialSystemProcess) && (Process != PsIdleProcess))
//         {
//             /* Kill it */
//             PspTerminateProcess(Process, STATUS_SYSTEM_SHUTDOWN);
//         }
//
//         /* Get the next process */
//         Process = PsGetNextProcess(Process);
//     }
// }
//
// VOID
// NTAPI
// PspExitApcRundown(IN PKAPC Apc)
// {
//     PAGED_CODE();
//
//     /* Free the APC */
//     ExFreePool(Apc);
// }
//
// VOID
// NTAPI
// PspReapRoutine(IN PVOID Context)
// {
//     PSINGLE_LIST_ENTRY NextEntry;
//     PETHREAD Thread;
//     PSTRACE(PS_KILL_DEBUG, "Context: %p\n", Context);
//
//     /* Start main loop */
//     do
//     {
//         /* Write magic value and return the next entry to process */
//         NextEntry = InterlockedExchangePointer((PVOID*)&PspReaperListHead.Flink,
//                                                (PVOID)1);
//         ASSERT((NextEntry != NULL) && (NextEntry != (PVOID)1));
//
//         /* Start inner loop */
//         do
//         {
//             /* Get the first Thread Entry */
//             Thread = CONTAINING_RECORD(NextEntry, ETHREAD, ReaperLink);
//
//             /* Delete this entry's kernel stack */
//             MmDeleteKernelStack((PVOID)Thread->Tcb.StackBase,
//                                 Thread->Tcb.LargeStack);
//             Thread->Tcb.InitialStack = NULL;
//
//             /* Move to the next entry */
//             NextEntry = NextEntry->Next;
//
//             /* Dereference this thread */
//             ObDereferenceObject(Thread);
//         } while ((NextEntry != NULL) && (NextEntry != (PVOID)1));
//
//         /* Remove magic value, keep looping if it got changed */
//     } while (InterlockedCompareExchangePointer((PVOID*)&PspReaperListHead.Flink,
//                                                NULL,
//                                                (PVOID)1) != (PVOID)1);
// }
//
// #if DBG
// VOID
// NTAPI
// PspCheckProcessList(VOID)
// {
//     PLIST_ENTRY Entry;
//
//     KeAcquireGuardedMutex(&PspActiveProcessMutex);
//     DbgPrint("# checking PsActiveProcessHead @ %p\n", &PsActiveProcessHead);
//     for (Entry = PsActiveProcessHead.Flink;
//          Entry != &PsActiveProcessHead;
//          Entry = Entry->Flink)
//     {
//         PEPROCESS Process = CONTAINING_RECORD(Entry, EPROCESS, ActiveProcessLinks);
//         POBJECT_HEADER Header;
//         PVOID Info, HeaderLocation;
//
//         /* Get the header and assume this is what we'll free */
//         Header = OBJECT_TO_OBJECT_HEADER(Process);
//         HeaderLocation = Header;
//
//         /* To find the header, walk backwards from how we allocated */
//         if ((Info = OBJECT_HEADER_TO_CREATOR_INFO(Header)))
//         {
//             HeaderLocation = Info;
//         }
//         if ((Info = OBJECT_HEADER_TO_NAME_INFO(Header)))
//         {
//             HeaderLocation = Info;
//         }
//         if ((Info = OBJECT_HEADER_TO_HANDLE_INFO(Header)))
//         {
//             HeaderLocation = Info;
//         }
//         if ((Info = OBJECT_HEADER_TO_QUOTA_INFO(Header)))
//         {
//             HeaderLocation = Info;
//         }
//
//         ExpCheckPoolAllocation(HeaderLocation, NonPagedPool, 'corP');
//     }
//
//     KeReleaseGuardedMutex(&PspActiveProcessMutex);
// }
// #endif
//
// VOID
// NTAPI
// PspDeleteProcess(IN PVOID ObjectBody)
// {
//     PEPROCESS Process = (PEPROCESS)ObjectBody;
//     KAPC_STATE ApcState;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "ObjectBody: %p\n", ObjectBody);
//     PSREFTRACE(Process);
//
//     /* Check if it has an Active Process Link */
//     if (Process->ActiveProcessLinks.Flink)
//     {
//         /* Remove it from the Active List */
//         KeAcquireGuardedMutex(&PspActiveProcessMutex);
//         RemoveEntryList(&Process->ActiveProcessLinks);
//         Process->ActiveProcessLinks.Flink = NULL;
//         Process->ActiveProcessLinks.Blink = NULL;
//         KeReleaseGuardedMutex(&PspActiveProcessMutex);
//     }
//
//     /* Check for Auditing information */
//     if (Process->SeAuditProcessCreationInfo.ImageFileName)
//     {
//         /* Free it */
//         ExFreePoolWithTag(Process->SeAuditProcessCreationInfo.ImageFileName,
//                           TAG_SEPA);
//         Process->SeAuditProcessCreationInfo.ImageFileName = NULL;
//     }
//
//     /* Check if we have a job */
//     if (Process->Job)
//     {
//         /* Remove the process from the job */
//         PspRemoveProcessFromJob(Process, Process->Job);
//
//         /* Dereference it */
//         ObDereferenceObject(Process->Job);
//         Process->Job = NULL;
//     }
//
//     /* Increase the stack count */
//     Process->Pcb.StackCount++;
//
//     /* Check if we have a debug port */
//     if (Process->DebugPort)
//     {
//         /* Deference the Debug Port */
//         ObDereferenceObject(Process->DebugPort);
//         Process->DebugPort = NULL;
//     }
//
//     /* Check if we have an exception port */
//     if (Process->ExceptionPort)
//     {
//         /* Deference the Exception Port */
//         ObDereferenceObject(Process->ExceptionPort);
//         Process->ExceptionPort = NULL;
//     }
//
//     /* Check if we have a section object */
//     if (Process->SectionObject)
//     {
//         /* Deference the Section Object */
//         ObDereferenceObject(Process->SectionObject);
//         Process->SectionObject = NULL;
//     }
//
// #if defined(_X86_)
//     /* Clean Ldt and Vdm objects */
//     PspDeleteLdt(Process);
//     PspDeleteVdmObjects(Process);
// #endif
//
//     /* Delete the Object Table */
//     if (Process->ObjectTable)
//     {
//         /* Attach to the process */
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//
//         /* Kill the Object Info */
//         ObKillProcess(Process);
//
//         /* Detach */
//         KeUnstackDetachProcess(&ApcState);
//     }
//
//     /* Check if we have an address space, and clean it */
//     if (Process->HasAddressSpace)
//     {
//         /* Attach to the process */
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//
//         /* Clean the Address Space */
//         PspExitProcess(FALSE, Process);
//
//         /* Detach */
//         KeUnstackDetachProcess(&ApcState);
//
//         /* Completely delete the Address Space */
//         MmDeleteProcessAddressSpace(Process);
//     }
//
//     /* See if we have a PID */
//     if (Process->UniqueProcessId)
//     {
//         /* Delete the PID */
//         if (!(ExDestroyHandle(PspCidTable, Process->UniqueProcessId, NULL)))
//         {
//             /* Something wrong happened, bugcheck */
//             KeBugCheck(CID_HANDLE_DELETION);
//         }
//     }
//
//     /* Cleanup security information */
//     PspDeleteProcessSecurity(Process);
//
//     /* Check if we have kept information on the Working Set */
//     if (Process->WorkingSetWatch)
//     {
//         /* Free it */
//         ExFreePool(Process->WorkingSetWatch);
//
//         /* And return the quota it was taking up */
//         PsReturnProcessNonPagedPoolQuota(Process, 0x2000);
//     }
//
//     /* Dereference the Device Map */
//     ObDereferenceDeviceMap(Process);
//
//     /*
//      * Dereference the quota block, the function
//      * will invoke a quota block cleanup if the
//      * block itself is no longer used by anybody.
//      */
//     PspDereferenceQuotaBlock(Process, Process->QuotaBlock);
// }
//
// VOID
// NTAPI
// PspDeleteThread(IN PVOID ObjectBody)
// {
//     PETHREAD Thread = (PETHREAD)ObjectBody;
//     PEPROCESS Process = Thread->ThreadsProcess;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "ObjectBody: %p\n", ObjectBody);
//     PSREFTRACE(Thread);
//     ASSERT(Thread->Tcb.Win32Thread == NULL);
//
//     /* Check if we have a stack */
//     if (Thread->Tcb.InitialStack)
//     {
//         /* Release it */
//         MmDeleteKernelStack((PVOID)Thread->Tcb.StackBase,
//                             Thread->Tcb.LargeStack);
//     }
//
//     /* Check if we have a CID Handle */
//     if (Thread->Cid.UniqueThread)
//     {
//         /* Delete the CID Handle */
//         if (!(ExDestroyHandle(PspCidTable, Thread->Cid.UniqueThread, NULL)))
//         {
//             /* Something wrong happened, bugcheck */
//             KeBugCheck(CID_HANDLE_DELETION);
//         }
//     }
//
//     /* Cleanup impersionation information */
//     PspDeleteThreadSecurity(Thread);
//
//     /* Make sure the thread was inserted, before continuing */
//     if (!Process) return;
//
//     /* Check if the thread list is valid */
//     if (Thread->ThreadListEntry.Flink)
//     {
//         /* Lock the thread's process */
//         KeEnterCriticalRegion();
//         ExAcquirePushLockExclusive(&Process->ProcessLock);
//
//         /* Remove us from the list */
//         RemoveEntryList(&Thread->ThreadListEntry);
//
//         /* Release the lock */
//         ExReleasePushLockExclusive(&Process->ProcessLock);
//         KeLeaveCriticalRegion();
//     }
//
//     /* Dereference the Process */
//     ObDereferenceObject(Process);
// }
//
// /*
//  * FUNCTION: Terminates the current thread
//  * See "Windows Internals" - Chapter 13, Page 50-53
//  */
// VOID
// NTAPI
// PspExitThread(IN NTSTATUS ExitStatus)
// {
//     CLIENT_DIED_MSG TerminationMsg;
//     NTSTATUS Status;
//     PTEB Teb;
//     PEPROCESS CurrentProcess;
//     PETHREAD Thread, OtherThread, PreviousThread = NULL;
//     PVOID DeallocationStack;
//     SIZE_T Dummy;
//     BOOLEAN Last = FALSE;
//     PTERMINATION_PORT TerminationPort, NextPort;
//     PLIST_ENTRY FirstEntry, CurrentEntry;
//     PKAPC Apc;
//     PTOKEN PrimaryToken;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "ExitStatus: %d\n", ExitStatus);
//
//     /* Get the Current Thread and Process */
//     Thread = PsGetCurrentThread();
//     CurrentProcess = Thread->ThreadsProcess;
//     ASSERT((Thread) == PsGetCurrentThread());
//
//     /* Can't terminate a thread if it attached another process */
//     if (KeIsAttachedProcess())
//     {
//         /* Bugcheck */
//         KeBugCheckEx(INVALID_PROCESS_ATTACH_ATTEMPT,
//                      (ULONG_PTR)CurrentProcess,
//                      (ULONG_PTR)Thread->Tcb.ApcState.Process,
//                      (ULONG_PTR)Thread->Tcb.ApcStateIndex,
//                      (ULONG_PTR)Thread);
//     }
//
//     /* Lower to Passive Level */
//     KeLowerIrql(PASSIVE_LEVEL);
//
//     /* Can't be a worker thread */
//     if (Thread->ActiveExWorker)
//     {
//         /* Bugcheck */
//         KeBugCheckEx(ACTIVE_EX_WORKER_THREAD_TERMINATION,
//                      (ULONG_PTR)Thread,
//                      0,
//                      0,
//                      0);
//     }
//
//     /* Can't have pending APCs */
//     if (Thread->Tcb.CombinedApcDisable != 0)
//     {
//         /* Bugcheck */
//         KeBugCheckEx(KERNEL_APC_PENDING_DURING_EXIT,
//                      0,
//                      Thread->Tcb.CombinedApcDisable,
//                      0,
//                      1);
//     }
//
//     /* Lock the thread */
//     ExWaitForRundownProtectionRelease(&Thread->RundownProtect);
//
//     /* Cleanup the power state */
//     PopCleanupPowerState((PPOWER_STATE)&Thread->Tcb.PowerState);
//
//     /* Call the WMI Callback for Threads */
//     //WmiTraceThread(Thread, NULL, FALSE);
//
//     /* Run Thread Notify Routines before we desintegrate the thread */
//     PspRunCreateThreadNotifyRoutines(Thread, FALSE);
//
//     /* Lock the Process before we modify its thread entries */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&CurrentProcess->ProcessLock);
//
//     /* Decrease the active thread count, and check if it's 0 */
//     if (!(--CurrentProcess->ActiveThreads))
//     {
//         /* Set the delete flag */
//         InterlockedOr((PLONG)&CurrentProcess->Flags, PSF_PROCESS_DELETE_BIT);
//
//         /* Remember we are last */
//         Last = TRUE;
//
//         /* Check if this termination is due to the thread dying */
//         if (ExitStatus == STATUS_THREAD_IS_TERMINATING)
//         {
//             /* Check if the last thread was pending */
//             if (CurrentProcess->ExitStatus == STATUS_PENDING)
//             {
//                 /* Use the last exit status */
//                 CurrentProcess->ExitStatus = CurrentProcess->
//                                              LastThreadExitStatus;
//             }
//         }
//         else
//         {
//             /* Just a normal exit, write the code */
//             CurrentProcess->ExitStatus = ExitStatus;
//         }
//
//         /* Loop all the current threads */
//         FirstEntry = &CurrentProcess->ThreadListHead;
//         CurrentEntry = FirstEntry->Flink;
//         while (FirstEntry != CurrentEntry)
//         {
//             /* Get the thread on the list */
//             OtherThread = CONTAINING_RECORD(CurrentEntry,
//                                             ETHREAD,
//                                             ThreadListEntry);
//
//             /* Check if it's a thread that's still alive */
//             if ((OtherThread != Thread) &&
//                 !(KeReadStateThread(&OtherThread->Tcb)) &&
//                 (ObReferenceObjectSafe(OtherThread)))
//             {
//                 /* It's a live thread and we referenced it, unlock process */
//                 ExReleasePushLockExclusive(&CurrentProcess->ProcessLock);
//                 KeLeaveCriticalRegion();
//
//                 /* Wait on the thread */
//                 KeWaitForSingleObject(OtherThread,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//
//                 /* Check if we had a previous thread to dereference */
//                 if (PreviousThread) ObDereferenceObject(PreviousThread);
//
//                 /* Remember the thread and re-lock the process */
//                 PreviousThread = OtherThread;
//                 KeEnterCriticalRegion();
//                 ExAcquirePushLockExclusive(&CurrentProcess->ProcessLock);
//             }
//
//             /* Go to the next thread */
//             CurrentEntry = CurrentEntry->Flink;
//         }
//     }
//     else if (ExitStatus != STATUS_THREAD_IS_TERMINATING)
//     {
//         /* Write down the exit status of the last thread to get killed */
//         CurrentProcess->LastThreadExitStatus = ExitStatus;
//     }
//
//     /* Unlock the Process */
//     ExReleasePushLockExclusive(&CurrentProcess->ProcessLock);
//     KeLeaveCriticalRegion();
//
//     /* Check if we had a previous thread to dereference */
//     if (PreviousThread) ObDereferenceObject(PreviousThread);
//
//     /* Check if the process has a debug port and if this is a user thread */
//     if ((CurrentProcess->DebugPort) && !(Thread->SystemThread))
//     {
//         /* Notify the Debug API. */
//         Last ? DbgkExitProcess(CurrentProcess->ExitStatus) :
//                DbgkExitThread(ExitStatus);
//     }
//
//     /* Check if this is a Critical Thread */
//     if ((KdDebuggerEnabled) && (Thread->BreakOnTermination))
//     {
//         /* Break to debugger */
//         PspCatchCriticalBreak("Critical thread 0x%p (in %s) exited\n",
//                               Thread,
//                               CurrentProcess->ImageFileName);
//     }
//
//     /* Check if it's the last thread and this is a Critical Process */
//     if ((Last) && (CurrentProcess->BreakOnTermination))
//     {
//         /* Check if a debugger is here to handle this */
//         if (KdDebuggerEnabled)
//         {
//             /* Break to debugger */
//             PspCatchCriticalBreak("Critical process 0x%p (%s) exited\n",
//                                   CurrentProcess,
//                                   CurrentProcess->ImageFileName);
//         }
//         else
//         {
//             /* Bugcheck, we can't allow this */
//             KeBugCheckEx(CRITICAL_PROCESS_DIED,
//                          (ULONG_PTR)CurrentProcess,
//                          0,
//                          0,
//                          0);
//         }
//     }
//
//     /* Sanity check */
//     ASSERT(Thread->Tcb.CombinedApcDisable == 0);
//
//     /* Process the Termination Ports */
//     TerminationPort = Thread->TerminationPort;
//     if (TerminationPort)
//     {
//         /* Setup the message header */
//         TerminationMsg.h.u2.ZeroInit = 0;
//         TerminationMsg.h.u2.s2.Type = LPC_CLIENT_DIED;
//         TerminationMsg.h.u1.s1.TotalLength = sizeof(TerminationMsg);
//         TerminationMsg.h.u1.s1.DataLength = sizeof(TerminationMsg) -
//                                             sizeof(PORT_MESSAGE);
//
//         /* Loop each port */
//         do
//         {
//             /* Save the Create Time */
//             TerminationMsg.CreateTime = Thread->CreateTime;
//
//             /* Loop trying to send message */
//             while (TRUE)
//             {
//                 /* Send the LPC Message */
//                 Status = LpcRequestPort(TerminationPort->Port,
//                                         &TerminationMsg.h);
//                 if ((Status == STATUS_NO_MEMORY) ||
//                     (Status == STATUS_INSUFFICIENT_RESOURCES))
//                 {
//                     /* Wait a bit and try again */
//                     KeDelayExecutionThread(KernelMode, FALSE, &ShortTime);
//                     continue;
//                 }
//                 break;
//             }
//
//             /* Dereference this LPC Port */
//             ObDereferenceObject(TerminationPort->Port);
//
//             /* Move to the next one */
//             NextPort = TerminationPort->Next;
//
//             /* Free the Termination Port Object */
//             ExFreePoolWithTag(TerminationPort, '=TsP');
//
//             /* Keep looping as long as there is a port */
//             TerminationPort = NextPort;
//         } while (TerminationPort);
//     }
//     else if (((ExitStatus == STATUS_THREAD_IS_TERMINATING) &&
//               (Thread->DeadThread)) ||
//              !(Thread->DeadThread))
//     {
//         /*
//          * This case is special and deserves some extra comments. What
//          * basically happens here is that this thread doesn't have a termination
//          * port, which means that it died before being fully created. Since we
//          * still have to notify an LPC Server, we'll use the exception port,
//          * which we know exists. However, we need to know how far the thread
//          * actually got created. We have three possibilities:
//          *
//          *  - NtCreateThread returned an error really early: DeadThread is set.
//          *  - NtCreateThread managed to create the thread: DeadThread is off.
//          *  - NtCreateThread was creating the thread (with DeadThread set,
//          *    but the thread got killed prematurely: STATUS_THREAD_IS_TERMINATING
//          *    is our exit code.)
//          *
//          * For the 2 & 3rd scenarios, the thread has been created far enough to
//          * warrant notification to the LPC Server.
//          */
//
//         /* Setup the message header */
//         TerminationMsg.h.u2.ZeroInit = 0;
//         TerminationMsg.h.u2.s2.Type = LPC_CLIENT_DIED;
//         TerminationMsg.h.u1.s1.TotalLength = sizeof(TerminationMsg);
//         TerminationMsg.h.u1.s1.DataLength = sizeof(TerminationMsg) -
//                                             sizeof(PORT_MESSAGE);
//
//         /* Make sure the process has an exception port */
//         if (CurrentProcess->ExceptionPort)
//         {
//             /* Save the Create Time */
//             TerminationMsg.CreateTime = Thread->CreateTime;
//
//             /* Loop trying to send message */
//             while (TRUE)
//             {
//                 /* Send the LPC Message */
//                 Status = LpcRequestPort(CurrentProcess->ExceptionPort,
//                                         &TerminationMsg.h);
//                 if ((Status == STATUS_NO_MEMORY) ||
//                     (Status == STATUS_INSUFFICIENT_RESOURCES))
//                 {
//                     /* Wait a bit and try again */
//                     KeDelayExecutionThread(KernelMode, FALSE, &ShortTime);
//                     continue;
//                 }
//                 break;
//             }
//         }
//     }
//
//     /* Rundown Win32 Thread if there is one */
//     if (Thread->Tcb.Win32Thread) PspW32ThreadCallout(Thread,
//                                                      PsW32ThreadCalloutExit);
//
//     /* If we are the last thread and have a W32 Process */
//     if ((Last) && (CurrentProcess->Win32Process))
//     {
//         /* Run it down too */
//         PspW32ProcessCallout(CurrentProcess, FALSE);
//     }
//
//     /* Make sure Stack Swap is enabled */
//     if (!Thread->Tcb.EnableStackSwap)
//     {
//         /* Stack swap really shouldn't be disabled during exit! */
//         KeBugCheckEx(KERNEL_STACK_LOCKED_AT_EXIT, 0, 0, 0, 0);
//     }
//
//     /* Cancel I/O for the thread. */
//     IoCancelThreadIo(Thread);
//
//     /* Rundown Timers */
//     ExTimerRundown();
//
//     /* FIXME: Rundown Registry Notifications (NtChangeNotify)
//     CmNotifyRunDown(Thread); */
//
//     /* Rundown Mutexes */
//     KeRundownThread();
//
//     /* Check if we have a TEB */
//     Teb = Thread->Tcb.Teb;
//     if (Teb)
//     {
//         /* Check if the thread is still alive */
//         if (!Thread->DeadThread)
//         {
//             /* Check if we need to free its stack */
//             if (Teb->FreeStackOnTermination)
//             {
//                 /* Set the TEB's Deallocation Stack as the Base Address */
//                 Dummy = 0;
//                 DeallocationStack = Teb->DeallocationStack;
//
//                 /* Free the Thread's Stack */
//                 ZwFreeVirtualMemory(NtCurrentProcess(),
//                                     &DeallocationStack,
//                                     &Dummy,
//                                     MEM_RELEASE);
//             }
//
//             /* Free the debug handle */
//             if (Teb->DbgSsReserved[1]) ObCloseHandle(Teb->DbgSsReserved[1],
//                                                      UserMode);
//         }
//
//         /* Decommit the TEB */
//         MmDeleteTeb(CurrentProcess, Teb);
//         Thread->Tcb.Teb = NULL;
//     }
//
//     /* Free LPC Data */
//     LpcExitThread(Thread);
//
//     /* Save the exit status and exit time */
//     Thread->ExitStatus = ExitStatus;
//     KeQuerySystemTime(&Thread->ExitTime);
//
//     /* Sanity check */
//     ASSERT(Thread->Tcb.CombinedApcDisable == 0);
//
//     /* Check if this is the final thread or not */
//     if (Last)
//     {
//         /* Set the process exit time */
//         CurrentProcess->ExitTime = Thread->ExitTime;
//
//         /* Exit the process */
//         PspExitProcess(TRUE, CurrentProcess);
//
//         /* Get the process token and check if we need to audit */
//         PrimaryToken = PsReferencePrimaryToken(CurrentProcess);
//         if (SeDetailedAuditingWithToken(PrimaryToken))
//         {
//             /* Audit the exit */
//             SeAuditProcessExit(CurrentProcess);
//         }
//
//         /* Dereference the process token */
//         ObFastDereferenceObject(&CurrentProcess->Token, PrimaryToken);
//
//         /* Check if this is a VDM Process and rundown the VDM DPCs if so */
//         if (CurrentProcess->VdmObjects) { /* VdmRundownDpcs(CurrentProcess); */ }
//
//         /* Kill the process in the Object Manager */
//         ObKillProcess(CurrentProcess);
//
//         /* Check if we have a section object */
//         if (CurrentProcess->SectionObject)
//         {
//             /* Dereference and clear the Section Object */
//             ObDereferenceObject(CurrentProcess->SectionObject);
//             CurrentProcess->SectionObject = NULL;
//         }
//
//         /* Check if the process is part of a job */
//         if (CurrentProcess->Job)
//         {
//             /* Remove the process from the job */
//             PspExitProcessFromJob(CurrentProcess->Job, CurrentProcess);
//         }
//     }
//
//     /* Disable APCs */
//     KeEnterCriticalRegion();
//
//     /* Disable APC queueing, force a resumption */
//     Thread->Tcb.ApcQueueable = FALSE;
//     KeForceResumeThread(&Thread->Tcb);
//
//     /* Re-enable APCs */
//     KeLeaveCriticalRegion();
//
//     /* Flush the User APCs */
//     FirstEntry = KeFlushQueueApc(&Thread->Tcb, UserMode);
//     if (FirstEntry)
//     {
//         /* Start with the first entry */
//         CurrentEntry = FirstEntry;
//         do
//         {
//            /* Get the APC */
//            Apc = CONTAINING_RECORD(CurrentEntry, KAPC, ApcListEntry);
//
//            /* Move to the next one */
//            CurrentEntry = CurrentEntry->Flink;
//
//            /* Rundown the APC or de-allocate it */
//            if (Apc->RundownRoutine)
//            {
//               /* Call its own routine */
//               Apc->RundownRoutine(Apc);
//            }
//            else
//            {
//               /* Do it ourselves */
//               ExFreePool(Apc);
//            }
//         }
//         while (CurrentEntry != FirstEntry);
//     }
//
//     /* Clean address space if this was the last thread */
//     if (Last) MmCleanProcessAddressSpace(CurrentProcess);
//
//     /* Call the Lego routine */
//     if (Thread->Tcb.LegoData) PspRunLegoRoutine(&Thread->Tcb);
//
//     /* Flush the APC queue, which should be empty */
//     FirstEntry = KeFlushQueueApc(&Thread->Tcb, KernelMode);
//     if ((FirstEntry) || (Thread->Tcb.CombinedApcDisable != 0))
//     {
//         /* Bugcheck time */
//         KeBugCheckEx(KERNEL_APC_PENDING_DURING_EXIT,
//                      (ULONG_PTR)FirstEntry,
//                      Thread->Tcb.CombinedApcDisable,
//                      KeGetCurrentIrql(),
//                      0);
//     }
//
//     /* Signal the process if this was the last thread */
//     if (Last) KeSetProcess(&CurrentProcess->Pcb, 0, FALSE);
//
//     /* Terminate the Thread from the Scheduler */
//     KeTerminateThread(0);
// }
//
// VOID
// NTAPI
// PsExitSpecialApc(IN PKAPC Apc,
//                  IN OUT PKNORMAL_ROUTINE* NormalRoutine,
//                  IN OUT PVOID* NormalContext,
//                  IN OUT PVOID* SystemArgument1,
//                  IN OUT PVOID* SystemArgument2)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG,
//             "Apc: %p SystemArgument2: %p\n", Apc, SystemArgument2);
//
//     /* Don't do anything unless we are in User-Mode */
//     if (Apc->SystemArgument2)
//     {
//         /* Free the APC */
//         Status = PtrToUlong(Apc->NormalContext);
//         PspExitApcRundown(Apc);
//
//         /* Terminate the Thread */
//         PspExitThread(Status);
//     }
// }
//
// VOID
// NTAPI
// PspExitNormalApc(IN PVOID NormalContext,
//                  IN PVOID SystemArgument1,
//                  IN PVOID SystemArgument2)
// {
//     PKAPC Apc = (PKAPC)SystemArgument1;
//     PETHREAD Thread = PsGetCurrentThread();
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "SystemArgument2: %p\n", SystemArgument2);
//
//     /* This should never happen */
//     ASSERT(!(((ULONG_PTR)SystemArgument2) & 1));
//
//     /* If we're here, this is not a System Thread, so kill it from User-Mode */
//     KeInitializeApc(Apc,
//                     &Thread->Tcb,
//                     OriginalApcEnvironment,
//                     PsExitSpecialApc,
//                     PspExitApcRundown,
//                     PspExitNormalApc,
//                     UserMode,
//                     NormalContext);
//
//     /* Now insert the APC with the User-Mode Flag */
//     if (!(KeInsertQueueApc(Apc,
//                            Apc,
//                            (PVOID)((ULONG_PTR)SystemArgument2 | 1),
//                            2)))
//     {
//         /* Failed to insert, free the APC */
//         PspExitApcRundown(Apc);
//     }
//
//     /* Set the APC Pending flag */
//     Thread->Tcb.ApcState.UserApcPending = TRUE;
// }
//
// /*
//  * See "Windows Internals" - Chapter 13, Page 49
//  */
// NTSTATUS
// NTAPI
// PspTerminateThreadByPointer(IN PETHREAD Thread,
//                             IN NTSTATUS ExitStatus,
//                             IN BOOLEAN bSelf)
// {
//     PKAPC Apc;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Flags;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "Thread: %p ExitStatus: %d\n", Thread, ExitStatus);
//     PSREFTRACE(Thread);
//
//     /* Check if this is a Critical Thread, and Bugcheck */
//     if (Thread->BreakOnTermination)
//     {
//         /* Break to debugger */
//         PspCatchCriticalBreak("Terminating critical thread 0x%p (in %s)\n",
//                               Thread,
//                               Thread->ThreadsProcess->ImageFileName);
//     }
//
//     /* Check if we are already inside the thread */
//     if ((bSelf) || (PsGetCurrentThread() == Thread))
//     {
//         /* This should only happen at passive */
//         ASSERT_IRQL_EQUAL(PASSIVE_LEVEL);
//
//         /* Mark it as terminated */
//         PspSetCrossThreadFlag(Thread, CT_TERMINATED_BIT);
//
//         /* Directly terminate the thread */
//         PspExitThread(ExitStatus);
//     }
//
//     /* This shouldn't be a system thread */
//     if (Thread->SystemThread) return STATUS_ACCESS_DENIED;
//
//     /* Allocate the APC */
//     Apc = ExAllocatePoolWithTag(NonPagedPool, sizeof(KAPC), TAG_TERMINATE_APC);
//     if (!Apc) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set the Terminated Flag */
//     Flags = Thread->CrossThreadFlags | CT_TERMINATED_BIT;
//
//     /* Set it, and check if it was already set while we were running */
//     if (!(InterlockedExchange((PLONG)&Thread->CrossThreadFlags, Flags) &
//           CT_TERMINATED_BIT))
//     {
//         /* Initialize a Kernel Mode APC to Kill the Thread */
//         KeInitializeApc(Apc,
//                         &Thread->Tcb,
//                         OriginalApcEnvironment,
//                         PsExitSpecialApc,
//                         PspExitApcRundown,
//                         PspExitNormalApc,
//                         KernelMode,
//                         UlongToPtr(ExitStatus));
//
//         /* Insert it into the APC Queue */
//         if (!KeInsertQueueApc(Apc, Apc, NULL, 2))
//         {
//             /* The APC was already in the queue, fail */
//             Status = STATUS_UNSUCCESSFUL;
//         }
//         else
//         {
//             /* Forcefully resume the thread and return */
//             KeForceResumeThread(&Thread->Tcb);
//             return Status;
//         }
//     }
//
//     /* We failed, free the APC */
//     ExFreePoolWithTag(Apc, TAG_TERMINATE_APC);
//
//     /* Return Status */
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// PspIsProcessExiting(IN PEPROCESS Process)
// {
//     return Process->Flags & PSF_PROCESS_EXITING_BIT;
// }
//
// VOID
// NTAPI
// PspExitProcess(IN BOOLEAN LastThread,
//                IN PEPROCESS Process)
// {
//     ULONG Actual;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG,
//             "LastThread: %u Process: %p\n", LastThread, Process);
//     PSREFTRACE(Process);
//
//     /* Set Process Exit flag */
//     InterlockedOr((PLONG)&Process->Flags, PSF_PROCESS_EXITING_BIT);
//
//     /* Check if we are the last thread */
//     if (LastThread)
//     {
//         /* Notify the WMI Process Callback */
//         //WmiTraceProcess(Process, FALSE);
//
//         /* Run the Notification Routines */
//         PspRunCreateProcessNotifyRoutines(Process, FALSE);
//     }
//
//     /* Cleanup the power state */
//     PopCleanupPowerState((PPOWER_STATE)&Process->Pcb.PowerState);
//
//     /* Clear the security port */
//     if (!Process->SecurityPort)
//     {
//         /* So we don't double-dereference */
//         Process->SecurityPort = (PVOID)1;
//     }
//     else if (Process->SecurityPort != (PVOID)1)
//     {
//         /* Dereference it */
//         ObDereferenceObject(Process->SecurityPort);
//         Process->SecurityPort = (PVOID)1;
//     }
//
//     /* Check if we are the last thread */
//     if (LastThread)
//     {
//         /* Check if we have to set the Timer Resolution */
//         if (Process->SetTimerResolution)
//         {
//             /* Set it to default */
//             ZwSetTimerResolution(KeMaximumIncrement, 0, &Actual);
//         }
//
//         /* Check if we are part of a Job that has a completion port */
//         if ((Process->Job) && (Process->Job->CompletionPort))
//         {
//             /* FIXME: Check job status code and do I/O completion if needed */
//         }
//
//         /* FIXME: Notify the Prefetcher */
//     }
//     else
//     {
//         /* Clear process' address space here */
//         MmCleanProcessAddressSpace(Process);
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsTerminateSystemThread(IN NTSTATUS ExitStatus)
// {
//     PETHREAD Thread = PsGetCurrentThread();
//
//     /* Make sure this is a system thread */
//     if (!Thread->SystemThread) return STATUS_INVALID_PARAMETER;
//
//     /* Terminate it for real */
//     return PspTerminateThreadByPointer(Thread, ExitStatus, TRUE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtTerminateProcess(IN HANDLE ProcessHandle OPTIONAL,
//                    IN NTSTATUS ExitStatus)
// {
//     NTSTATUS Status;
//     PEPROCESS Process, CurrentProcess = PsGetCurrentProcess();
//     PETHREAD Thread, CurrentThread = PsGetCurrentThread();
//     BOOLEAN KillByHandle;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG,
//             "ProcessHandle: %p ExitStatus: %d\n", ProcessHandle, ExitStatus);
//
//     /* Enhanced diagnostic: log caller PID */
//     DbgPrint("[PROC-TERM] CallerPID=%lu ExitStatus=0x%lX Handle=%p\n",
//              (ULONG)(ULONG_PTR)PsGetCurrentProcessId(),
//              (ULONG)ExitStatus,
//              ProcessHandle);
//
// #ifdef _M_ARM64
//     /* ARM64 trace: log process termination */
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
//         if (u && KeGetPreviousMode() == UserMode)
//         {
//             const char *m = "[PROC-TERM] PID=";
//             while (*m) *u = (ULONG)*m++;
//             {
//                 ULONG pid = (ULONG)(ULONG_PTR)PsGetCurrentProcessId();
//                 char buf[12]; int pos = 0;
//                 if (pid == 0) buf[pos++] = '0';
//                 else { char t[12]; int tp=0; while(pid){t[tp++]='0'+pid%10;pid/=10;} while(tp>0)buf[pos++]=t[--tp]; }
//                 buf[pos] = 0;
//                 for(int bi=0;bi<pos;bi++) *u = (ULONG)buf[bi];
//             }
//             m = " ExitStatus=0x";
//             while (*m) *u = (ULONG)*m++;
//             {
//                 ULONG s = (ULONG)ExitStatus;
//                 char hx[9]; int hi;
//                 for(hi=7;hi>=0;hi--){hx[hi]="0123456789ABCDEF"[s&0xF];s>>=4;}
//                 hx[8]=0;
//                 for(hi=0;hi<8;hi++) *u = (ULONG)hx[hi];
//             }
//             *u = '\r'; *u = '\n';
//         }
//     }
// #endif
//
//     /* Were we passed a process handle? */
//     if (ProcessHandle)
//     {
//         /* Yes we were, use it */
//         KillByHandle = TRUE;
//     }
//     else
//     {
//         /* We weren't... we assume this is suicide */
//         KillByHandle = FALSE;
//         ProcessHandle = NtCurrentProcess();
//     }
//
//     /* Get the Process Object */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_TERMINATE,
//                                        PsProcessType,
//                                        KeGetPreviousMode(),
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return(Status);
//
//     /* Enhanced diagnostic: log target process info */
//     DbgPrint("[PROC-TERM] TargetPID=%lu Image=%s Critical=%d\n",
//              (ULONG)(ULONG_PTR)PsGetProcessId(Process),
//              Process->ImageFileName,
//              Process->BreakOnTermination);
//
//     /* Check if this is a Critical Process, and Bugcheck */
//     if (Process->BreakOnTermination)
//     {
//         /* Break to debugger */
//         PspCatchCriticalBreak("Terminating critical process 0x%p (%s)\n",
//                               Process,
//                               Process->ImageFileName);
//     }
//
//     /* Lock the Process */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         /* Failed to lock, fail */
//         ObDereferenceObject(Process);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Set the delete flag, unless the process is comitting suicide */
//     if (KillByHandle) PspSetProcessFlag(Process, PSF_PROCESS_DELETE_BIT);
//
//     /* Get the first thread */
//     Status = STATUS_NOTHING_TO_TERMINATE;
//     Thread = PsGetNextProcessThread(Process, NULL);
//     if (Thread)
//     {
//         /* We know we have at least a thread */
//         Status = STATUS_SUCCESS;
//
//         /* Loop and kill the others */
//         do
//         {
//             /* Ensure it's not ours*/
//             if (Thread != CurrentThread)
//             {
//                 /* Kill it */
//                 PspTerminateThreadByPointer(Thread, ExitStatus, FALSE);
//             }
//
//             /* Move to the next thread */
//             Thread = PsGetNextProcessThread(Process, Thread);
//         } while (Thread);
//     }
//
//     /* Unlock the process */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//
//     /* Check if we are killing ourselves */
//     if (Process == CurrentProcess)
//     {
//         /* Also make sure the caller gave us our handle */
//         if (KillByHandle)
//         {
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//
//             /* Terminate ourselves */
//             PspTerminateThreadByPointer(CurrentThread, ExitStatus, TRUE);
//         }
//     }
//     else if (ExitStatus == DBG_TERMINATE_PROCESS)
//     {
//         /* Disable debugging on this process */
//         DbgkClearProcessDebugObject(Process, NULL);
//     }
//
//     /* Check if there was nothing to terminate, or if we have a Debug Port */
//     if ((Status == STATUS_NOTHING_TO_TERMINATE) ||
//         ((Process->DebugPort) && (KillByHandle)))
//     {
//         /* Clear the handle table */
//         ObClearProcessHandleTable(Process);
//
//         /* Return status now */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Decrease the reference count we added */
//     ObDereferenceObject(Process);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtTerminateThread(IN HANDLE ThreadHandle,
//                   IN NTSTATUS ExitStatus)
// {
//     PETHREAD Thread;
//     PETHREAD CurrentThread = PsGetCurrentThread();
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG,
//             "ThreadHandle: %p ExitStatus: %d\n", ThreadHandle, ExitStatus);
//
//     /* Handle the special NULL case */
//     if (!ThreadHandle)
//     {
//         /* Check if we're the only thread left */
//         if (PsGetCurrentProcess()->ActiveThreads == 1)
//         {
//             /* This is invalid */
//             return STATUS_CANT_TERMINATE_SELF;
//         }
//
//         /* Terminate us directly */
//         goto TerminateSelf;
//     }
//     else if (ThreadHandle == NtCurrentThread())
//     {
// TerminateSelf:
//         /* Terminate this thread */
//         return PspTerminateThreadByPointer(CurrentThread,
//                                            ExitStatus,
//                                            TRUE);
//     }
//
//     /* We are terminating another thread, get the Thread Object */
//     Status = ObReferenceObjectByHandle(ThreadHandle,
//                                        THREAD_TERMINATE,
//                                        PsThreadType,
//                                        KeGetPreviousMode(),
//                                        (PVOID*)&Thread,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check to see if we're running in the same thread */
//     if (Thread != CurrentThread)
//     {
//         /* Terminate it */
//         Status = PspTerminateThreadByPointer(Thread, ExitStatus, FALSE);
//
//         /* Dereference the Thread and return */
//         ObDereferenceObject(Thread);
//     }
//     else
//     {
//         /* Dereference the thread and terminate ourselves */
//         ObDereferenceObject(Thread);
//         goto TerminateSelf;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtRegisterThreadTerminatePort(IN HANDLE PortHandle)
// {
//     NTSTATUS Status;
//     PTERMINATION_PORT TerminationPort;
//     PVOID TerminationLpcPort;
//     PETHREAD Thread;
//     PAGED_CODE();
//     PSTRACE(PS_KILL_DEBUG, "PortHandle: %p\n", PortHandle);
//
//     /* Get the Port */
//     Status = ObReferenceObjectByHandle(PortHandle,
//                                        PORT_ALL_ACCESS,
//                                        LpcPortObjectType,
//                                        KeGetPreviousMode(),
//                                        &TerminationLpcPort,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return(Status);
//
//     /* Allocate the Port and make sure it suceeded */
//     TerminationPort = ExAllocatePoolWithTag(NonPagedPool,
//                                             sizeof(TERMINATION_PORT),
//                                             '=TsP');
//     if(TerminationPort)
//     {
//         /* Associate the Port */
//         Thread = PsGetCurrentThread();
//         TerminationPort->Port = TerminationLpcPort;
//         TerminationPort->Next = Thread->TerminationPort;
//         Thread->TerminationPort = TerminationPort;
//
//         /* Return success */
//         return STATUS_SUCCESS;
//     }
//
//     /* Dereference and Fail */
//     ObDereferenceObject(TerminationLpcPort);
//     return STATUS_INSUFFICIENT_RESOURCES;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
