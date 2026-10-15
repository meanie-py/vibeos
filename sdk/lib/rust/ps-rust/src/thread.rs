use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES,
    STATUS_NOT_IMPLEMENTED,
    APC_LEVEL, PASSIVE_LEVEL};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ke::KeGetCurrentPrcb;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// NT6 THREAD_ALL_ACCESS (the SDK name means the NT5 0x1F03FF at this tree's 0x502): the
// thread type's ValidAccessMask, and what the creator's pseudo-handle access holds.
const THREAD_ALL_ACCESS_NT6: ACCESS_MASK = 0x001FFFFF;

extern "system" {

    static mut CcPfEnablePrefetcher: BOOLEAN;
    static mut MmReadClusterSize: ULONG;
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
    static mut PspCidTable: PHANDLE_TABLE;
    static mut PsProcessType: POBJECT_TYPE;
    static mut SeDebugPrivilege: LUID;

    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ExGetPreviousMode() -> KPROCESSOR_MODE;
    // PspRunCreateThreadNotifyRoutines: imported via ddk::* (bindgen extern "C")
    fn PspTerminateThreadByPointer(thread: PETHREAD, exit_status: NTSTATUS, b_self: BOOLEAN,) -> NTSTATUS;
    fn PsLookupProcessThreadByCid(Cid: PCLIENT_ID, Process: *mut PEPROCESS, Thread: *mut PETHREAD) -> NTSTATUS;
    fn PsReferencePrimaryToken(Process: PEPROCESS) -> PACCESS_TOKEN;

    static mut PspSystemDllEntryPoint: PVOID;
    static mut PspSystemDllBase: PVOID;
}
extern "C" {

    fn PspSetCrossThreadFlag(Thread: PETHREAD, Flag: ULONG);
    fn KeGetExceptionFrame(Thread: *mut _KTHREAD) -> PKEXCEPTION_FRAME;
    fn KeGetTrapFrame(Thread: *mut _KTHREAD) -> PKTRAP_FRAME;
}
unsafe extern "system" {
    fn KeLowerIrql(NewIrql: KIRQL);
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
}
extern "C" {
    fn ObpValidateAttributes(Attributes: ULONG, PreviousMode: KPROCESSOR_MODE) -> ULONG;
}

// FORCEINLINE functions provided by ps_rust_shims.c as __cdecl
extern "C" {
    fn KeGetContextPc(Context: PCONTEXT) -> ULONG_PTR;
    fn KeGetContextReturnRegister(Context: PCONTEXT) -> ULONG_PTR;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(Destination: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedCompareExchange(Destination: *mut LONG, Exchange: LONG, Comparand: LONG) -> LONG;
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
}

#[cfg(target_arch = "aarch64")]
use crate::trace::{ARM64_UART_TRACE, ARM64_UART_TRACE_HEX};
#[cfg(target_arch = "aarch64")]
extern "C" {
}

type PACCESS_TOKEN = PVOID;

const CT_SYSTEM_THREAD_BIT: ULONG = 0x00000010;
const CT_DEAD_THREAD_BIT: ULONG = 0x00000002;
const ThreadObject: UCHAR = 6;

#[no_mangle]
pub static mut PsThreadType: POBJECT_TYPE = core::ptr::null_mut();

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn Arm64ReadTtbr0El1() -> ULONG_PTR {
    let mut value: u64;
    core::arch::asm!("mrs {}, ttbr0_el1", out(reg) value, options(nomem, nostack));
    value as ULONG_PTR
}

#[cfg(target_arch = "aarch64")]
unsafe fn Arm64TraceImageName(prefix: PCSTR, process: PEPROCESS) {
    let mut buf = [0u8; 32];
    let image_name = (*(process as *mut _EPROCESS)).ImageFileName.as_ptr();
    let len = core::mem::size_of_val(&(*(process as *mut _EPROCESS)).ImageFileName);
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

unsafe extern "system" fn PspUserThreadStartup(StartRoutine: PKSTART_ROUTINE, StartContext: PVOID) {
    let mut Thread: PETHREAD;
    let mut Teb: PTEB;
    let mut DeadThread: BOOLEAN = 0u8;
    let mut OldIrql: KIRQL = 0;

    #[cfg(target_arch = "aarch64")]
    {
        ARM64_UART_TRACE(b"[PS] PspUserThreadStartup enter\r\n\0".as_ptr() as PCSTR);
        ARM64_UART_TRACE_HEX(b"[PS] PspUserThreadStartup: StartRoutine=\0".as_ptr() as PCSTR, StartRoutine.map(|f| f as ULONG_PTR).unwrap_or(0));
        ARM64_UART_TRACE_HEX(b"[PS] PspUserThreadStartup: StartContext=\0".as_ptr() as PCSTR, StartContext as ULONG_PTR);
    }

    KeLowerIrql(PASSIVE_LEVEL);
    Thread = PsGetCurrentThread();

    #[cfg(target_arch = "aarch64")]
    {
        let current_process = PsGetCurrentProcess() as PEPROCESS;
        let thread_process = (*Thread).ThreadsProcess;
        ARM64_UART_TRACE_HEX(
            b"[PS] PspUserThreadStartup: CurrentProcess=\0".as_ptr() as PCSTR,
            current_process as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[PS] PspUserThreadStartup: ThreadProcess=\0".as_ptr() as PCSTR,
            thread_process as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[PS] PspUserThreadStartup: Current.DTB=\0".as_ptr() as PCSTR,
            (*(current_process as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[PS] PspUserThreadStartup: Thread.DTB=\0".as_ptr() as PCSTR,
            (*(thread_process as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[PS] PspUserThreadStartup: TTBR0=\0".as_ptr() as PCSTR,
            Arm64ReadTtbr0El1(),
        );
        Arm64TraceImageName(
            b"[PS] PspUserThreadStartup: Current.Image=\0".as_ptr() as PCSTR,
            current_process,
        );
        Arm64TraceImageName(
            b"[PS] PspUserThreadStartup: Thread.Image=\0".as_ptr() as PCSTR,
            thread_process,
        );
    }

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() != 0 {
        DeadThread = 1u8;
    } else {
        Teb = NtCurrentTeb();
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(b"[PS] PspUserThreadStartup: Teb=\0".as_ptr() as PCSTR, Teb as ULONG_PTR);
            ARM64_UART_TRACE_HEX(
                b"[PS] PspUserThreadStartup: Teb.StackBase=\0".as_ptr() as PCSTR,
                (*Teb).NtTib.StackBase as ULONG_PTR,
            );
            ARM64_UART_TRACE_HEX(
                b"[PS] PspUserThreadStartup: Teb.StackLimit=\0".as_ptr() as PCSTR,
                (*Teb).NtTib.StackLimit as ULONG_PTR,
            );
            ARM64_UART_TRACE_HEX(
                b"[PS] PspUserThreadStartup: Teb.Dealloc=\0".as_ptr() as PCSTR,
                (*Teb).DeallocationStack as ULONG_PTR,
            );
        }
        (*Teb).CurrentLocale = MmGetSessionLocaleId();
        (*Teb).IdealProcessor = (*Thread).Tcb.IdealProcessor;
    }

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() == 0
        && (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() == 0 {
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE(b"[PS] PspUserThreadStartup: DbgkCreateThread\r\n\0".as_ptr() as PCSTR);
        }
        DbgkCreateThread(Thread, StartContext);
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE(b"[PS] PspUserThreadStartup: DbgkCreateThread done\r\n\0".as_ptr() as PCSTR);
        }
    }

    if DeadThread == 0 {
        if CcPfEnablePrefetcher != 0 {
        }

        KeRaiseIrql(APC_LEVEL, &mut OldIrql);

        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(b"[PS] PspUserThreadStartup: PspSystemDllEntryPoint=\0".as_ptr() as PCSTR, PspSystemDllEntryPoint as ULONG_PTR);
            ARM64_UART_TRACE_HEX(b"[PS] PspUserThreadStartup: PspSystemDllBase=\0".as_ptr() as PCSTR, PspSystemDllBase as ULONG_PTR);
            ARM64_UART_TRACE(b"[PS] PspUserThreadStartup: KiInitializeUserApc\r\n\0".as_ptr() as PCSTR);
        }
        KiInitializeUserApc(
            KeGetExceptionFrame(&mut (*Thread).Tcb),
            KeGetTrapFrame(&mut (*Thread).Tcb),
            core::mem::transmute(PspSystemDllEntryPoint),
            core::ptr::null_mut(),
            PspSystemDllBase,
            core::ptr::null_mut(),
        );
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE(b"[PS] PspUserThreadStartup: KiInitializeUserApc done\r\n\0".as_ptr() as PCSTR);
        }

        KeLowerIrql(PASSIVE_LEVEL);
    } else {
        PspTerminateThreadByPointer(Thread, STATUS_THREAD_IS_TERMINATING, 1u8);
    }

    #[cfg(target_arch = "aarch64")]
    {
        crate::trace::milestone(b"[PS] PspUserThreadStartup: returning\r\n\0".as_ptr() as PCSTR);
    }

    while (*SharedUserData).Cookie == 0 {
        let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
        let mut NewCookie: ULONG;
        let Prcb: *mut _KPRCB;

        KeQuerySystemTime(&mut SystemTime);
        Prcb = KeGetCurrentPrcb() as *mut _KPRCB;
        NewCookie = ((*Prcb).MmPageFaultCount as ULONG) ^ (*Prcb).InterruptTime
            ^ SystemTime.u.LowPart ^ (SystemTime.u.HighPart as ULONG)
            ^ ((&SystemTime as *const _ as ULONG_PTR) as ULONG);

        InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*SharedUserData).Cookie) as *mut LONG,
            NewCookie as LONG,
            0,
        );
    }
}

unsafe fn PspUnhandledExceptionInSystemThread(ExceptionPointers: *mut EXCEPTION_POINTERS) -> LONG {
    KeBugCheckEx(
        SYSTEM_THREAD_EXCEPTION_NOT_HANDLED,
        (*(*ExceptionPointers).ExceptionRecord).ExceptionCode as ULONG_PTR,
        (*(*ExceptionPointers).ExceptionRecord).ExceptionAddress as ULONG_PTR,
        (*ExceptionPointers).ExceptionRecord as ULONG_PTR,
        (*ExceptionPointers).ContextRecord as ULONG_PTR,
    )
}

unsafe extern "system" fn PspSystemThreadStartup(StartRoutine: PKSTART_ROUTINE, StartContext: PVOID) {
    let Thread: PETHREAD;

    KeLowerIrql(PASSIVE_LEVEL);
    Thread = PsGetCurrentThread();

    let terminated = (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.Terminated();
    let dead = (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread();

    if terminated == 0 && dead == 0 {
        (StartRoutine.unwrap())(StartContext);
    }

    PspTerminateThreadByPointer(Thread, STATUS_SUCCESS, 1u8);
}

// ARM64 early boot hit an optimizer-sensitive failure in this routine during
// Phase 1 system-thread creation. Keeping it uninlined avoids the bad codegen
// while preserving the Rust implementation on that architecture.
#[cfg_attr(target_arch = "aarch64", inline(never))]
unsafe fn PspCreateThread(
    ThreadHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessHandle: HANDLE,
    TargetProcess: PEPROCESS,
    ClientId: PCLIENT_ID,
    ThreadContext: PCONTEXT,
    InitialTeb: PINITIAL_TEB,
    CreateSuspended: BOOLEAN,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
) -> NTSTATUS {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut TebBase: PTEB = core::ptr::null_mut();
    let mut PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut AccessStatus: NTSTATUS = 0;
    let mut CidEntry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let mut LocalAccessState: _ACCESS_STATE = core::mem::zeroed();
    let AccessState: PACCESS_STATE = &mut LocalAccessState;
    let mut AuxData: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut Result: BOOLEAN;
    let mut SdAllocated: BOOLEAN = 0;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SubjectContext: _SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();

    if StartRoutine.is_some() {
        PreviousMode = 0i8;
    }

    if !ProcessHandle.is_null() {
        Status = ObReferenceObjectByHandle(
            ProcessHandle,
            PROCESS_CREATE_THREAD,
            PsProcessType,
            PreviousMode,
            &mut Process as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
    } else {
        if StartRoutine.is_some() {
            ObReferenceObject(TargetProcess as PVOID);
            Process = TargetProcess;
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_INVALID_HANDLE;
        }
    }

    if Status < 0 {
        return Status;
    }

    if PreviousMode != 0i8 && Process as PVOID == ntkrnl_sys::mm::PsInitialSystemProcess {
        ObDereferenceObject(Process as PVOID);
        return STATUS_INVALID_HANDLE;
    }

    Status = ObCreateObject(
        PreviousMode,
        PsThreadType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<_ETHREAD>() as ULONG,
        0,
        0,
        &mut Thread as *mut _ as *mut PVOID,
    );

    if Status < 0 {
        ObDereferenceObject(Process as PVOID);
        return Status;
    }

    core::ptr::write_bytes(Thread as *mut u8, 0, core::mem::size_of::<_ETHREAD>());

    ExInitializeRundownProtection(core::ptr::addr_of_mut!((*Thread).RundownProtect));

    (*(Thread as *mut _ETHREAD)).__bindgen_anon_2.ExitStatus = STATUS_PENDING;

    (*Thread).ThreadsProcess = Process;
    (*Thread).Cid.UniqueProcess = (*Process).UniqueProcessId;

    CidEntry.__bindgen_anon_1.Object = Thread as PVOID;
    CidEntry.__bindgen_anon_2.GrantedAccess = 0;
    (*Thread).Cid.UniqueThread = ExCreateHandle(PspCidTable, &mut CidEntry);
    if (*Thread).Cid.UniqueThread.is_null() {
        ObDereferenceObject(Thread as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Thread).ReadClusterSize = MmReadClusterSize;

    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).__bindgen_anon_4.LpcReplySemaphore),
        0,
        1,
    );

    InitializeListHead(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).__bindgen_anon_1.LpcReplyChain) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*Thread).IrpList) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*Thread).PostBlockList) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*Thread).ActiveTimerListHead) as PLIST_ENTRY);
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock));

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect)) == 0 {
        ObDereferenceObject(Thread as PVOID);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    if !ThreadContext.is_null() {
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE(b"[PS] PspCreateThread: MmCreateTeb\r\n\0".as_ptr() as PCSTR);
            ARM64_UART_TRACE_HEX(
                b"[PS] PspCreateThread: InitialTeb.Alloc=\0".as_ptr() as PCSTR,
                (*InitialTeb).AllocatedStackBase as ULONG_PTR,
            );
            ARM64_UART_TRACE_HEX(
                b"[PS] PspCreateThread: InitialTeb.Base=\0".as_ptr() as PCSTR,
                (*InitialTeb).StackBase as ULONG_PTR,
            );
            ARM64_UART_TRACE_HEX(
                b"[PS] PspCreateThread: InitialTeb.Limit=\0".as_ptr() as PCSTR,
                (*InitialTeb).StackLimit as ULONG_PTR,
            );
            ARM64_UART_TRACE_HEX(
                b"[PS] PspCreateThread: InitialCtx.Sp=\0".as_ptr() as PCSTR,
                (*ThreadContext).Sp as ULONG_PTR,
            );
        }
        Status = MmCreateTeb(Process, &mut (*Thread).Cid, InitialTeb, &mut TebBase);
        if Status < 0 {
            #[cfg(target_arch = "aarch64")]
            {
                ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: MmCreateTeb FAILED=\0".as_ptr() as PCSTR, Status as ULONG_PTR);
            }
            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));
            ObDereferenceObject(Thread as PVOID);
            return Status;
        }
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: TebBase=\0".as_ptr() as PCSTR, TebBase as ULONG_PTR);
        }

        (*(Thread as *mut _ETHREAD)).__bindgen_anon_6.StartAddress = core::mem::transmute(KeGetContextPc(ThreadContext));
        (*Thread).Win32StartAddress = KeGetContextReturnRegister(ThreadContext) as PVOID;
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(
                b"[PS] PspCreateThread: StartAddress=\0".as_ptr() as PCSTR,
                (*(Thread as *mut _ETHREAD))
                    .__bindgen_anon_6
                    .StartAddress
                    .map(|f| f as ULONG_PTR)
                    .unwrap_or(0),
            );
            ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: Win32StartAddress=\0".as_ptr() as PCSTR, (*Thread).Win32StartAddress as ULONG_PTR);
        }

        if Status >= 0 {
            #[cfg(target_arch = "aarch64")]
            {
                ARM64_UART_TRACE(b"[PS] PspCreateThread: KeInitThread\r\n\0".as_ptr() as PCSTR);
            }
            Status = KeInitThread(
                &mut (*Thread).Tcb,
                core::ptr::null_mut(),
                core::mem::transmute(PspUserThreadStartup as usize),
                None,
                core::mem::transmute((*(Thread as *mut _ETHREAD)).__bindgen_anon_6.StartAddress),
                ThreadContext,
                TebBase as PVOID,
                &mut (*Process).Pcb,
            );
            #[cfg(target_arch = "aarch64")]
            {
                ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: KeInitThread status=\0".as_ptr() as PCSTR, Status as ULONG_PTR);
            }
        }
    } else {

        (*(Thread as *mut _ETHREAD)).__bindgen_anon_6.StartAddress = core::mem::transmute(StartRoutine);
        PspSetCrossThreadFlag(Thread, CT_SYSTEM_THREAD_BIT);

        Status = KeInitThread(
            &mut (*Thread).Tcb,
            core::ptr::null_mut(),
            core::mem::transmute(PspSystemThreadStartup as usize),
            StartRoutine,
            StartContext,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut (*Process).Pcb,
        );
    }

    if Status < 0 {
        if !TebBase.is_null() {
            MmDeleteTeb(Process, TebBase);
        }

        ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));
        ObDereferenceObject(Thread as PVOID);
        return Status;
    }

    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*Process).ProcessLock));

    if (*(Process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessDelete() != 0 {
        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*Process).ProcessLock));
        KeLeaveCriticalRegion();
        KeUninitThread(&mut (*Thread).Tcb);
        if !TebBase.is_null() {
            MmDeleteTeb(Process, TebBase);
        }
        ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));
        ObDereferenceObject(Thread as PVOID);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.Terminated() != 0 && !ThreadContext.is_null() && (*Thread).ThreadsProcess == Process {
        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*Process).ProcessLock));
        KeLeaveCriticalRegion();
        KeUninitThread(&mut (*Thread).Tcb);
        if !TebBase.is_null() {
            MmDeleteTeb(Process, TebBase);
        }
        ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));
        ObDereferenceObject(Thread as PVOID);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    InsertTailList(
        core::ptr::addr_of_mut!((*Process).ThreadListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Thread).ThreadListEntry) as PLIST_ENTRY,
    );
    (*Process).ActiveThreads += 1;

    #[cfg(target_arch = "aarch64")]
    {
        ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: Thread=\0".as_ptr() as PCSTR, Thread as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[PS] PspCreateThread: Process=\0".as_ptr() as PCSTR, Process as ULONG_PTR);
        ARM64_UART_TRACE(b"[PS] PspCreateThread: KeStartThread\r\n\0".as_ptr() as PCSTR);
    }
    KeStartThread(&mut (*Thread).Tcb);
    #[cfg(target_arch = "aarch64")]
    {
        ARM64_UART_TRACE(b"[PS] PspCreateThread: KeStartThread done\r\n\0".as_ptr() as PCSTR);
    }

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*Process).ProcessLock));
    KeLeaveCriticalRegion();

    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));

    PspRunCreateThreadNotifyRoutines(Thread, 1u8);

    ObReferenceObjectEx(Thread as PVOID, 2);

    if CreateSuspended != 0 {
        KeSuspendThread(&mut (*Thread).Tcb);
    }

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.Terminated() != 0 {
        KeForceResumeThread(&mut (*Thread).Tcb);
    }

    Status = SeCreateAccessStateEx(
        core::ptr::null_mut(),
        if !ThreadContext.is_null() {
            ntkrnl_sys::ps::PsGetCurrentProcess() as PEPROCESS
        } else {
            Process
        },
        &mut LocalAccessState,
        &mut AuxData,
        DesiredAccess,
        core::ptr::addr_of_mut!((*PsThreadType).TypeInfo.GenericMapping),
    );

    if Status < 0 {
        PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
        if CreateSuspended != 0 {
            KeResumeThread(&mut (*Thread).Tcb);
        }
        KeReadyThread(&mut (*Thread).Tcb);
        ObDereferenceObjectEx(Thread as PVOID, 2);
        return Status;
    }

    Status = ObInsertObject(
        Thread as PVOID,
        AccessState,
        DesiredAccess,
        0,
        core::ptr::null_mut(),
        &mut hThread,
    );

    if !AccessState.is_null() {
        SeDeleteAccessState(AccessState);
    }

    if Status >= 0 {
        // C: both stores share one __try, and its handler kills the thread just
        // inserted (dead flag, resume, ready, dereference, close the handle) before
        // returning the code. Returning early left it suspended with a live handle.
        let mut fault = None;
        if !ClientId.is_null() {
            if let Err(s) = write_back(ClientId, (*Thread).Cid) {
                fault = Some(s);
            }
        }
        if fault.is_none() {
            if let Err(s) = probe_write_user(ThreadHandle, hThread, PreviousMode) {
                fault = Some(s);
            }
        }
        if let Some(s) = fault {
            PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
            if CreateSuspended != 0 {
                KeResumeThread(&mut (*Thread).Tcb);
            }
            KeReadyThread(&mut (*Thread).Tcb);
            ObDereferenceObject(Thread as PVOID);
            ObCloseHandle(hThread, PreviousMode);
            return s;
        }
    } else {
        PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
        if CreateSuspended != 0 {
            KeResumeThread(&mut (*Thread).Tcb);
        }
    }

    KeQuerySystemTime(core::ptr::addr_of_mut!((*Thread).CreateTime));

    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.DeadThread() == 0 {
        Status = ObGetObjectSecurity(
            Thread as PVOID,
            &mut SecurityDescriptor,
            &mut SdAllocated,
        );

        if Status < 0 {
            PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
            if CreateSuspended != 0 {
                KeResumeThread(&mut (*Thread).Tcb);
            }
            KeReadyThread(&mut (*Thread).Tcb);
            ObDereferenceObject(Thread as PVOID);
            ObCloseHandle(hThread, PreviousMode);
            return Status;
        }

        SubjectContext.ProcessAuditId = Process as PVOID;
        SubjectContext.PrimaryToken = PsReferencePrimaryToken(Process);
        SubjectContext.ClientToken = core::ptr::null_mut();

        Result = SeAccessCheck(
            SecurityDescriptor,
            &mut SubjectContext,
            0u8,
            MAXIMUM_ALLOWED,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*PsThreadType).TypeInfo.GenericMapping),
            PreviousMode,
            core::ptr::addr_of_mut!((*Thread).GrantedAccess),
            &mut AccessStatus,
        );

        ObFastDereferenceObject(
            core::ptr::addr_of_mut!((*Process).Token),
            SubjectContext.PrimaryToken,
        );
        ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);

        if Result == 0 {
            (*Process).GrantedAccess = 0;
        }

        (*Thread).GrantedAccess |= THREAD_TERMINATE
            | THREAD_SET_INFORMATION
            | THREAD_QUERY_INFORMATION
            | THREAD_SET_LIMITED_INFORMATION
            | THREAD_QUERY_LIMITED_INFORMATION;
    } else {
        (*Thread).GrantedAccess = THREAD_ALL_ACCESS_NT6;
    }

    KeReadyThread(&mut (*Thread).Tcb);

    ObDereferenceObject(Thread as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PsCreateSystemThread(
    ThreadHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessHandle: HANDLE,
    ClientId: PCLIENT_ID,
    StartRoutine: PKSTART_ROUTINE,
    StartContext: PVOID,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let ClientId = opaque_ptr(ClientId);
    let StartContext = opaque_ptr(StartContext);
    let mut TargetProcess: PEPROCESS = core::ptr::null_mut();
    let mut Handle: HANDLE = ProcessHandle;

    if ProcessHandle.is_null() {
        Handle = core::ptr::null_mut();
        TargetProcess = ntkrnl_sys::mm::PsInitialSystemProcess as PEPROCESS;
    }

    PspCreateThread(
        ThreadHandle,
        DesiredAccess,
        ObjectAttributes,
        Handle,
        TargetProcess,
        ClientId,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
        StartRoutine,
        StartContext,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PsLookupThreadByThreadId(
    ThreadId: HANDLE,
    Thread: *mut PETHREAD,
) -> NTSTATUS {
    let ThreadId = opaque_ptr(ThreadId);
    let Thread = opaque_ptr(Thread);
    let mut CidEntry: PHANDLE_TABLE_ENTRY;
    let mut FoundThread: PETHREAD;
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    KeEnterCriticalRegion();

    CidEntry = ExMapHandleToPointer(PspCidTable, ThreadId);
    if !CidEntry.is_null() {
        FoundThread = (*CidEntry).__bindgen_anon_1.Object as PETHREAD;

        if (*FoundThread).Tcb.Header.__bindgen_anon_1.__bindgen_anon_1.Type == ThreadObject {
            if ObReferenceObjectSafe(FoundThread as PVOID) != 0 {
                *Thread = FoundThread;
                Status = STATUS_SUCCESS;
            }
        }

        ExUnlockHandleTableEntry(PspCidTable, CidEntry);
    }

    KeLeaveCriticalRegion();
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadFreezeCount(Thread: PETHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    #[cfg(not(target_arch = "x86"))]
    { (*Thread).Tcb.__bindgen_anon_6.__bindgen_anon_1.FreezeCount as ULONG }
    #[cfg(target_arch = "x86")]
    { (*Thread).Tcb.__bindgen_anon_8.__bindgen_anon_1.FreezeCount as ULONG }
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadHardErrorsAreDisabled(Thread: PETHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HardErrorsAreDisabled() != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadId(Thread: PETHREAD) -> HANDLE {
    let Thread = opaque_ptr(Thread);
    (*Thread).Cid.UniqueThread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadId() -> HANDLE {
    (*PsGetCurrentThread()).Cid.UniqueThread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadProcess(Thread: PETHREAD) -> PEPROCESS {
    let Thread = opaque_ptr(Thread);
    (*Thread).ThreadsProcess
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadProcess() -> PEPROCESS {
    (*PsGetCurrentThread()).ThreadsProcess
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadProcessId(Thread: PETHREAD) -> HANDLE {
    let Thread = opaque_ptr(Thread);
    (*Thread).Cid.UniqueProcess
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadProcessId() -> HANDLE {
    (*PsGetCurrentThread()).Cid.UniqueProcess
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadSessionId(Thread: PETHREAD) -> ULONG {
    let Thread = opaque_ptr(Thread);
    MmGetSessionId((*Thread).ThreadsProcess)
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadTeb(Thread: PETHREAD) -> PTEB {
    let Thread = opaque_ptr(Thread);
    (*Thread).Tcb.Teb
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadTeb() -> PVOID {
    (*PsGetCurrentThread()).Tcb.Teb as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetThreadWin32Thread(Thread: PETHREAD) -> PVOID {
    let Thread = opaque_ptr(Thread);
    (*Thread).Tcb.Win32Thread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadWin32Thread() -> PVOID {
    (*PsGetCurrentThread()).Tcb.Win32Thread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadWin32ThreadAndEnterCriticalRegion(
    OutProcessId: *mut HANDLE,
) -> PVOID {
    let OutProcessId = opaque_ptr(OutProcessId);
    let CurrentThread: PETHREAD;

    CurrentThread = PsGetCurrentThread();

    *OutProcessId = (*CurrentThread).Cid.UniqueProcess;

    KeEnterCriticalRegion();

    (*CurrentThread).Tcb.Win32Thread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadPreviousMode() -> KPROCESSOR_MODE {
    #[cfg(not(target_arch = "x86"))]
    { (*PsGetCurrentThread()).Tcb.__bindgen_anon_5.__bindgen_anon_2.PreviousMode as KPROCESSOR_MODE }
    #[cfg(target_arch = "x86")]
    { (*PsGetCurrentThread()).Tcb.__bindgen_anon_7.__bindgen_anon_2.PreviousMode as KPROCESSOR_MODE }
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadStackBase() -> PVOID {
    (*PsGetCurrentThread()).Tcb.StackBase
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentThreadStackLimit() -> PVOID {
    (*PsGetCurrentThread()).Tcb.StackLimit as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsIsThreadTerminating(Thread: PETHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.Terminated() != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsIsSystemThread(Thread: PETHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsIsThreadImpersonating(Thread: PETHREAD) -> BOOLEAN {
    let Thread = opaque_ptr(Thread);
    if (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.ActiveImpersonationInfo() != 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsSetThreadHardErrorsAreDisabled(
    Thread: PETHREAD,
    HardErrorsAreDisabled: BOOLEAN,
) {
    let Thread = opaque_ptr(Thread);
    (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.set_HardErrorsAreDisabled(HardErrorsAreDisabled as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn PsSetThreadWin32Thread(
    Thread: PETHREAD,
    Win32Thread: PVOID,
    OldWin32Thread: PVOID,
) -> PVOID {
    let Thread = opaque_ptr(Thread);
    let Win32Thread = opaque_ptr(Win32Thread);
    let OldWin32Thread = opaque_ptr(OldWin32Thread);
    if !Win32Thread.is_null() {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*Thread).Tcb.Win32Thread),
            Win32Thread,
        )
    } else {
        InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*Thread).Tcb.Win32Thread),
            Win32Thread,
            OldWin32Thread,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsWrapApcWow64Thread(
    ApcContext: *mut PVOID,
    ApcRoutine: *mut PVOID,
) -> NTSTATUS {
    let ApcContext = opaque_ptr(ApcContext);
    let ApcRoutine = opaque_ptr(ApcRoutine);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateThread(
    ThreadHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessHandle: HANDLE,
    ClientId: PCLIENT_ID,
    ThreadContext: PCONTEXT,
    InitialTeb: PINITIAL_TEB,
    CreateSuspended: BOOLEAN,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let ClientId = opaque_ptr(ClientId);
    let mut SafeInitialTeb: _INITIAL_TEB = core::mem::zeroed();
    let mut SafeContext: _CONTEXT = core::mem::zeroed();
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();

    if PreviousMode != 0i8 {
        if ThreadContext.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        // Probe user-mode pointers (was inside _SEH2_TRY in C)
        if let Err(s) = probe_user_buffer_write(
            ThreadHandle as *const _,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return s;
        }

        if !ClientId.is_null() {
            if let Err(s) = probe_user_buffer_write(
                ClientId as *const _, core::mem::size_of::<_CLIENT_ID>(),
                core::mem::align_of::<ULONG>(),
                PreviousMode,
            ) {
                return s;
            }
        }

        if let Err(s) = probe_user_buffer(
            ThreadContext as *const _,
            core::mem::size_of::<_CONTEXT>(),
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return s;
        }

        if let Err(s) = probe_user_buffer(
            InitialTeb as *const _,
            core::mem::size_of::<_INITIAL_TEB>(),
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return s;
        }
        SafeInitialTeb = match ntkrnl_sys::probe::read_back(InitialTeb) { Ok(v) => v, Err(s) => return s };
        // The C passes the caller's CONTEXT on and reads it under KeInitThread's
        // __try. Capture it once instead: the same values, no fault to survive
        // later, and no second read that could differ from the first.
        SafeContext = match ntkrnl_sys::probe::read_back(ThreadContext as *const _CONTEXT) {
            Ok(v) => v,
            Err(s) => return s,
        };
    } else {
        SafeInitialTeb = match ntkrnl_sys::probe::read_back(InitialTeb) { Ok(v) => v, Err(s) => return s };
    }
    let ThreadContext: PCONTEXT = if PreviousMode != 0i8 { &mut SafeContext } else { ThreadContext };

    PspCreateThread(
        ThreadHandle,
        DesiredAccess,
        ObjectAttributes,
        ProcessHandle,
        core::ptr::null_mut(),
        ClientId,
        ThreadContext,
        &mut SafeInitialTeb,
        CreateSuspended,
        None,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenThread(
    ThreadHandle: *mut HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ClientId: PCLIENT_ID,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let ClientId = opaque_ptr(ClientId);
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut SafeClientId: _CLIENT_ID = core::mem::zeroed();
    let mut Attributes: ULONG = 0;
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut HasObjectName: BOOLEAN = 0u8;
    let mut AccessState: _ACCESS_STATE = core::mem::zeroed();
    let mut AuxData: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut ClientId = ClientId;

    if PreviousMode != 0i8 {
        // Probe user-mode pointers (was inside _SEH2_TRY in C)
        if let Err(s) = probe_user_buffer_write(
            ThreadHandle as *const _,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return s;
        }

        if !ClientId.is_null() {
            if let Err(s) = probe_user_buffer(
                ClientId as *const _, core::mem::size_of::<_CLIENT_ID>(),
                core::mem::align_of::<ULONG>(),
                PreviousMode,
            ) {
                return s;
            }
            SafeClientId = match ntkrnl_sys::probe::read_back(ClientId) { Ok(v) => v, Err(s) => return s };
            ClientId = &mut SafeClientId;
        }

        if let Err(s) = probe_user_buffer(
            ObjectAttributes as *const _,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
            core::mem::align_of::<ULONG>(),
            PreviousMode,
        ) {
            return s;
        }
        HasObjectName = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*ObjectAttributes).ObjectName)) { Ok(n) => (!n.is_null()) as u8, Err(s) => return s };

        Attributes = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*ObjectAttributes).Attributes)) { Ok(a) => ObpValidateAttributes(a, PreviousMode), Err(s) => return s };
    } else {
        HasObjectName = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*ObjectAttributes).ObjectName)) { Ok(n) => (!n.is_null()) as u8, Err(s) => return s };

        Attributes = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*ObjectAttributes).Attributes)) { Ok(a) => ObpValidateAttributes(a, PreviousMode), Err(s) => return s };
    }

    if HasObjectName != 0 && !ClientId.is_null() {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    Status = SeCreateAccessState(
        &mut AccessState,
        &mut AuxData,
        DesiredAccess,
        core::ptr::addr_of_mut!((*PsThreadType).TypeInfo.GenericMapping),
    );
    if Status < 0 {
        return Status;
    }

    if SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode) != 0 {
        if AccessState.RemainingDesiredAccess & MAXIMUM_ALLOWED != 0 {
            AccessState.PreviouslyGrantedAccess |= THREAD_ALL_ACCESS_NT6;
        } else {
            AccessState.PreviouslyGrantedAccess |= AccessState.RemainingDesiredAccess;
        }

        AccessState.RemainingDesiredAccess = 0;
    }

    if HasObjectName != 0 {
        Status = ObOpenObjectByName(
            ObjectAttributes,
            PsThreadType,
            PreviousMode,
            &mut AccessState,
            0,
            core::ptr::null_mut(),
            &mut hThread,
        );

        SeDeleteAccessState(&mut AccessState);
    } else if !ClientId.is_null() {
        if !(*ClientId).UniqueProcess.is_null() {
            Status = PsLookupProcessThreadByCid(ClientId, core::ptr::null_mut(), &mut Thread);
        } else {
            Status = PsLookupThreadByThreadId((*ClientId).UniqueThread, &mut Thread);
        }

        if Status < 0 {
            SeDeleteAccessState(&mut AccessState);
            return Status;
        }

        Status = ObOpenObjectByPointer(
            Thread as PVOID,
            Attributes,
            &mut AccessState,
            0,
            PsThreadType,
            PreviousMode,
            &mut hThread,
        );

        SeDeleteAccessState(&mut AccessState);
        ObDereferenceObject(Thread as PVOID);
    } else {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    if Status >= 0 {
        // Write-back to user-mode pointer (was inside _SEH2_TRY in C)
        if let Err(s) = probe_write_user(ThreadHandle, hThread, PreviousMode) {
            return s;
        }
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/thread.c
//  * PURPOSE:         Process Manager: Thread Management
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// extern BOOLEAN CcPfEnablePrefetcher;
// extern ULONG MmReadClusterSize;
// POBJECT_TYPE PsThreadType = NULL;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// PspUserThreadStartup(IN PKSTART_ROUTINE StartRoutine,
//                      IN PVOID StartContext)
// {
//     PETHREAD Thread;
//     PTEB Teb;
//     BOOLEAN DeadThread = FALSE;
//     KIRQL OldIrql;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG,
//             "StartRoutine: %p StartContext: %p\n", StartRoutine, StartContext);
//
//     /* Go to Passive Level */
//     KeLowerIrql(PASSIVE_LEVEL);
//     Thread = PsGetCurrentThread();
//
//     ARM64_UART_TRACE("[PS] PspUserThreadStartup enter\r\n");
//
//     /* Check if the thread is dead */
//     if (Thread->DeadThread)
//     {
//         /* Remember that we're dead */
//         DeadThread = TRUE;
//     }
//     else
//     {
//         /* Get the Locale ID and save Preferred Proc */
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: NtCurrentTeb\r\n");
//         Teb =  NtCurrentTeb();
//         ARM64_UART_TRACE_HEX("[PS] PspUserThreadStartup: Teb=", (ULONG_PTR)Teb);
//         Teb->CurrentLocale = MmGetSessionLocaleId();
//         Teb->IdealProcessor = Thread->Tcb.IdealProcessor;
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: Teb written\r\n");
//     }
//
//     /* Check if this is a dead thread, or if we're hiding */
//     if (!(Thread->DeadThread) && !(Thread->HideFromDebugger))
//     {
//         /* We're not, so notify the debugger */
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: DbgkCreateThread\r\n");
//         DbgkCreateThread(Thread, StartContext);
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: DbgkCreateThread done\r\n");
//     }
//
//     /* Make sure we're not already dead */
//     if (!DeadThread)
//     {
//         /* Check if the Prefetcher is enabled */
//         if (CcPfEnablePrefetcher)
//         {
//             /* FIXME: Prepare to prefetch this process */
//         }
//
//         /* Raise to APC */
//         KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//         /* Queue the User APC */
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: KiInitializeUserApc\r\n");
//         KiInitializeUserApc(KeGetExceptionFrame(&Thread->Tcb),
//                             KeGetTrapFrame(&Thread->Tcb),
//                             PspSystemDllEntryPoint,
//                             NULL,
//                             PspSystemDllBase,
//                             NULL);
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: KiInitializeUserApc done\r\n");
//
//         /* Lower it back to passive */
//         KeLowerIrql(PASSIVE_LEVEL);
//         ARM64_UART_TRACE("[PS] PspUserThreadStartup: returning\r\n");
//     }
//     else
//     {
//         /* We're dead, kill us now */
//         PspTerminateThreadByPointer(Thread,
//                                     STATUS_THREAD_IS_TERMINATING,
//                                     TRUE);
//     }
//
//     /* Do we have a cookie set yet? */
//     while (!SharedUserData->Cookie)
//     {
//         LARGE_INTEGER SystemTime;
//         ULONG NewCookie;
//         PKPRCB Prcb;
//
//         /* Generate a new cookie */
//         KeQuerySystemTime(&SystemTime);
//         Prcb = KeGetCurrentPrcb();
//         NewCookie = (Prcb->MmPageFaultCount ^ Prcb->InterruptTime ^
//                     SystemTime.u.LowPart ^ SystemTime.u.HighPart ^
//                     (ULONG)(ULONG_PTR)&SystemTime);
//
//         /* Set the new cookie*/
//         InterlockedCompareExchange((LONG*)&SharedUserData->Cookie,
//                                    NewCookie,
//                                    0);
//     }
// }
//
// LONG
// PspUnhandledExceptionInSystemThread(PEXCEPTION_POINTERS ExceptionPointers)
// {
//     /* Print debugging information */
//     DPRINT1("PS: Unhandled Kernel Mode Exception Pointers = 0x%p\n",
//             ExceptionPointers);
//     DPRINT1("Code %x Addr %p Info0 %p Info1 %p Info2 %p Info3 %p\n",
//             ExceptionPointers->ExceptionRecord->ExceptionCode,
//             ExceptionPointers->ExceptionRecord->ExceptionAddress,
//             ExceptionPointers->ExceptionRecord->ExceptionInformation[0],
//             ExceptionPointers->ExceptionRecord->ExceptionInformation[1],
//             ExceptionPointers->ExceptionRecord->ExceptionInformation[2],
//             ExceptionPointers->ExceptionRecord->ExceptionInformation[3]);
//
//     /* Bugcheck the system */
//     KeBugCheckEx(SYSTEM_THREAD_EXCEPTION_NOT_HANDLED,
//                  ExceptionPointers->ExceptionRecord->ExceptionCode,
//                  (ULONG_PTR)ExceptionPointers->ExceptionRecord->ExceptionAddress,
//                  (ULONG_PTR)ExceptionPointers->ExceptionRecord,
//                  (ULONG_PTR)ExceptionPointers->ContextRecord);
//     return 0;
// }
//
// VOID
// NTAPI
// PspSystemThreadStartup(IN PKSTART_ROUTINE StartRoutine,
//                        IN PVOID StartContext)
// {
//     PETHREAD Thread;
//     PSTRACE(PS_THREAD_DEBUG,
//             "StartRoutine: %p StartContext: %p\n", StartRoutine, StartContext);
//
//     /* Unlock the dispatcher Database */
//     KeLowerIrql(PASSIVE_LEVEL);
//     Thread = PsGetCurrentThread();
//
//     /* Make sure the thread isn't gone */
//     _SEH2_TRY
//     {
//         if (!(Thread->Terminated) && !(Thread->DeadThread))
//         {
//             /* Call the Start Routine */
//             StartRoutine(StartContext);
//         }
//     }
//     _SEH2_EXCEPT(PspUnhandledExceptionInSystemThread(_SEH2_GetExceptionInformation()))
//     {
//         /* Bugcheck if we got here */
//         KeBugCheck(KMODE_EXCEPTION_NOT_HANDLED);
//     }
//     _SEH2_END;
//
//     /* Exit the thread */
//     PspTerminateThreadByPointer(Thread, STATUS_SUCCESS, TRUE);
// }
//
// NTSTATUS
// NTAPI
// PspCreateThread(OUT PHANDLE ThreadHandle,
//                 IN ACCESS_MASK DesiredAccess,
//                 IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                 IN HANDLE ProcessHandle,
//                 IN PEPROCESS TargetProcess,
//                 OUT PCLIENT_ID ClientId,
//                 IN PCONTEXT ThreadContext,
//                 IN PINITIAL_TEB InitialTeb,
//                 IN BOOLEAN CreateSuspended,
//                 IN PKSTART_ROUTINE StartRoutine OPTIONAL,
//                 IN PVOID StartContext OPTIONAL)
// {
//     HANDLE hThread;
//     PEPROCESS Process;
//     PETHREAD Thread;
//     PTEB TebBase = NULL;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status, AccessStatus;
//     HANDLE_TABLE_ENTRY CidEntry;
//     ACCESS_STATE LocalAccessState;
//     PACCESS_STATE AccessState = &LocalAccessState;
//     AUX_ACCESS_DATA AuxData;
//     BOOLEAN Result, SdAllocated;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG,
//             "ThreadContext: %p TargetProcess: %p ProcessHandle: %p\n",
//             ThreadContext, TargetProcess, ProcessHandle);
//
//     /* If we were called from PsCreateSystemThread, then we're kernel mode */
//     if (StartRoutine) PreviousMode = KernelMode;
//
//     /* Reference the Process by handle or pointer, depending on what we got */
//     if (ProcessHandle)
//     {
//         /* Normal thread or System Thread */
//         Status = ObReferenceObjectByHandle(ProcessHandle,
//                                            PROCESS_CREATE_THREAD,
//                                            PsProcessType,
//                                            PreviousMode,
//                                            (PVOID*)&Process,
//                                            NULL);
//         PSREFTRACE(Process);
//     }
//     else
//     {
//         /* System thread inside System Process, or Normal Thread with a bug */
//         if (StartRoutine)
//         {
//             /* Reference the Process by Pointer */
//             ObReferenceObject(TargetProcess);
//             Process = TargetProcess;
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* Fake ObReference returning this */
//             Status = STATUS_INVALID_HANDLE;
//         }
//     }
//
//     /* Check for success */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Also make sure that User-Mode isn't trying to create a system thread */
//     if ((PreviousMode != KernelMode) && (Process == PsInitialSystemProcess))
//     {
//         /* Fail */
//         ObDereferenceObject(Process);
//         return STATUS_INVALID_HANDLE;
//     }
//
//     /* Create Thread Object */
//     Status = ObCreateObject(PreviousMode,
//                             PsThreadType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(ETHREAD),
//                             0,
//                             0,
//                             (PVOID*)&Thread);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed; dereference the process and exit */
//         ObDereferenceObject(Process);
//         return Status;
//     }
//
//     /* Zero the Object entirely */
//     RtlZeroMemory(Thread, sizeof(ETHREAD));
//
//     /* Initialize rundown protection */
//     ExInitializeRundownProtection(&Thread->RundownProtect);
//
//     /* Initialize exit code */
//     Thread->ExitStatus = STATUS_PENDING;
//
//     /* Set the Process CID */
//     Thread->ThreadsProcess = Process;
//     Thread->Cid.UniqueProcess = Process->UniqueProcessId;
//
//     /* Create Cid Handle */
//     CidEntry.Object = Thread;
//     CidEntry.GrantedAccess = 0;
//     Thread->Cid.UniqueThread = ExCreateHandle(PspCidTable, &CidEntry);
//     if (!Thread->Cid.UniqueThread)
//     {
//         /* We couldn't create the CID, dereference the thread and fail */
//         ObDereferenceObject(Thread);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Save the read cluster size */
//     Thread->ReadClusterSize = MmReadClusterSize;
//
//     /* Initialize the LPC Reply Semaphore */
//     KeInitializeSemaphore(&Thread->LpcReplySemaphore, 0, 1);
//
//     /* Initialize the list heads and locks */
//     InitializeListHead(&Thread->LpcReplyChain);
//     InitializeListHead(&Thread->IrpList);
//     InitializeListHead(&Thread->PostBlockList);
//     InitializeListHead(&Thread->ActiveTimerListHead);
//     KeInitializeSpinLock(&Thread->ActiveTimerListLock);
//
//     /* Acquire rundown protection */
//     if (!ExAcquireRundownProtection (&Process->RundownProtect))
//     {
//         /* Fail */
//         ObDereferenceObject(Thread);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Now let the kernel initialize the context */
//     if (ThreadContext)
//     {
//         /* User-mode Thread, create Teb */
//         ARM64_UART_TRACE("[PS] PspCreateThread: MmCreateTeb\r\n");
//         Status = MmCreateTeb(Process, &Thread->Cid, InitialTeb, &TebBase);
//         if (!NT_SUCCESS(Status))
//         {
//             ARM64_UART_TRACE("[PS] PspCreateThread: MmCreateTeb FAILED\r\n");
//             /* Failed to create the TEB. Release rundown and dereference */
//             ExReleaseRundownProtection(&Process->RundownProtect);
//             ObDereferenceObject(Thread);
//             return Status;
//         }
//         ARM64_UART_TRACE("[PS] PspCreateThread: MmCreateTeb OK\r\n");
//
//         /* Set the Start Addresses from the untrusted ThreadContext */
//         _SEH2_TRY
//         {
//             Thread->StartAddress = (PVOID)KeGetContextPc(ThreadContext);
//             Thread->Win32StartAddress = (PVOID)KeGetContextReturnRegister(ThreadContext);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Let the kernel intialize the Thread */
//         if (NT_SUCCESS(Status))
//         {
//             ARM64_UART_TRACE("[PS] PspCreateThread: KeInitThread\r\n");
//             Status = KeInitThread(&Thread->Tcb,
//                                   NULL,
//                                   PspUserThreadStartup,
//                                   NULL,
//                                   Thread->StartAddress,
//                                   ThreadContext,
//                                   TebBase,
//                                   &Process->Pcb);
//             ARM64_UART_TRACE("[PS] PspCreateThread: KeInitThread done\r\n");
//         }
//     }
//     else
//     {
//         /* System Thread */
//         Thread->StartAddress = StartRoutine;
//         PspSetCrossThreadFlag(Thread, CT_SYSTEM_THREAD_BIT);
//
//         /* Let the kernel intialize the Thread */
//         Status = KeInitThread(&Thread->Tcb,
//                               NULL,
//                               PspSystemThreadStartup,
//                               StartRoutine,
//                               StartContext,
//                               NULL,
//                               NULL,
//                               &Process->Pcb);
//     }
//
//     /* Check if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Delete the TEB if we had done */
//         if (TebBase) MmDeleteTeb(Process, TebBase);
//
//         /* Release rundown and dereference */
//         ExReleaseRundownProtection(&Process->RundownProtect);
//         ObDereferenceObject(Thread);
//         return Status;
//     }
//
//     /* Lock the process */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&Process->ProcessLock);
//
//     /* Make sure the process didn't just die on us */
//     if (Process->ProcessDelete) goto Quickie;
//
//     /* Check if the thread was ours, terminated and it was user mode */
//     if ((Thread->Terminated) &&
//         (ThreadContext) &&
//         (Thread->ThreadsProcess == Process))
//     {
//         /* Cleanup, we don't want to start it up and context switch */
//         goto Quickie;
//     }
//
//     /*
//      * Insert the Thread into the Process's Thread List
//      * Note, this is the ETHREAD Thread List. It is removed in
//      * ps/kill.c!PspExitThread.
//      */
//     InsertTailList(&Process->ThreadListHead, &Thread->ThreadListEntry);
//     Process->ActiveThreads++;
//
//     /* Start the thread */
//     ARM64_UART_TRACE("[PS] PspCreateThread: KeStartThread\r\n");
//     KeStartThread(&Thread->Tcb);
//     ARM64_UART_TRACE("[PS] PspCreateThread: KeStartThread done\r\n");
//
//     /* Release the process lock */
//     ExReleasePushLockExclusive(&Process->ProcessLock);
//     KeLeaveCriticalRegion();
//
//     /* Release rundown */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//
//     /* Notify WMI */
//     //WmiTraceProcess(Process, TRUE);
//     //WmiTraceThread(Thread, InitialTeb, TRUE);
//
//     /* Notify Thread Creation */
//     ARM64_UART_TRACE("[PS] PspCreateThread: PspRunCreateThreadNotify\r\n");
//     PspRunCreateThreadNotifyRoutines(Thread, TRUE);
//
//     /* Reference ourselves as a keep-alive */
//     ObReferenceObjectEx(Thread, 2);
//
//     /* Suspend the Thread if we have to */
//     if (CreateSuspended) KeSuspendThread(&Thread->Tcb);
//
//     /* Check if we were already terminated */
//     if (Thread->Terminated) KeForceResumeThread(&Thread->Tcb);
//
//     /* Create an access state */
//     Status = SeCreateAccessStateEx(NULL,
//                                    ThreadContext ?
//                                    PsGetCurrentProcess() : Process,
//                                    &LocalAccessState,
//                                    &AuxData,
//                                    DesiredAccess,
//                                    &PsThreadType->TypeInfo.GenericMapping);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Access state failed, thread is dead */
//         PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
//
//         /* If we were suspended, wake it up */
//         if (CreateSuspended) KeResumeThread(&Thread->Tcb);
//
//         /* Dispatch thread */
//         KeReadyThread(&Thread->Tcb);
//
//         /* Dereference completely to kill it */
//         ObDereferenceObjectEx(Thread, 2);
//         return Status;
//     }
//
//     /* Insert the Thread into the Object Manager */
//     Status = ObInsertObject(Thread,
//                             AccessState,
//                             DesiredAccess,
//                             0,
//                             NULL,
//                             &hThread);
//
//     /* Delete the access state if we had one */
//     if (AccessState) SeDeleteAccessState(AccessState);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Wrap in SEH to protect against bad user-mode pointers */
//         _SEH2_TRY
//         {
//             /* Return Cid and Handle */
//             if (ClientId) *ClientId = Thread->Cid;
//             *ThreadHandle = hThread;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Thread insertion failed, thread is dead */
//             PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
//
//             /* If we were suspended, wake it up */
//             if (CreateSuspended) KeResumeThread(&Thread->Tcb);
//
//             /* Dispatch thread */
//             KeReadyThread(&Thread->Tcb);
//
//             /* Dereference it, leaving only the keep-alive */
//             ObDereferenceObject(Thread);
//
//             /* Close its handle, killing it */
//             ObCloseHandle(hThread, PreviousMode);
//
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Thread insertion failed, thread is dead */
//         PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
//
//         /* If we were suspended, wake it up */
//         if (CreateSuspended) KeResumeThread(&Thread->Tcb);
//     }
//
//     /* Get the create time */
//     KeQuerySystemTime(&Thread->CreateTime);
//     ASSERT(!(Thread->CreateTime.HighPart & 0xF0000000));
//
//     /* Make sure the thread isn't dead */
//     if (!Thread->DeadThread)
//     {
//         /* Get the thread's SD */
//         Status = ObGetObjectSecurity(Thread,
//                                      &SecurityDescriptor,
//                                      &SdAllocated);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Thread insertion failed, thread is dead */
//             PspSetCrossThreadFlag(Thread, CT_DEAD_THREAD_BIT);
//
//             /* If we were suspended, wake it up */
//             if (CreateSuspended) KeResumeThread(&Thread->Tcb);
//
//             /* Dispatch thread */
//             KeReadyThread(&Thread->Tcb);
//
//             /* Dereference it, leaving only the keep-alive */
//             ObDereferenceObject(Thread);
//
//             /* Close its handle, killing it */
//             ObCloseHandle(hThread, PreviousMode);
//             return Status;
//         }
//
//         /* Create the subject context */
//         SubjectContext.ProcessAuditId = Process;
//         SubjectContext.PrimaryToken = PsReferencePrimaryToken(Process);
//         SubjectContext.ClientToken = NULL;
//
//         /* Do the access check */
//         Result = SeAccessCheck(SecurityDescriptor,
//                                &SubjectContext,
//                                FALSE,
//                                MAXIMUM_ALLOWED,
//                                0,
//                                NULL,
//                                &PsThreadType->TypeInfo.GenericMapping,
//                                PreviousMode,
//                                &Thread->GrantedAccess,
//                                &AccessStatus);
//
//         /* Dereference the token and let go the SD */
//         ObFastDereferenceObject(&Process->Token,
//                                 SubjectContext.PrimaryToken);
//         ObReleaseObjectSecurity(SecurityDescriptor, SdAllocated);
//
//         /* Remove access if it failed */
//         if (!Result) Process->GrantedAccess = 0;
//
//         /* Set least some minimum access */
//         Thread->GrantedAccess |= (THREAD_TERMINATE |
//                                   THREAD_SET_INFORMATION |
//                                   THREAD_QUERY_INFORMATION);
//     }
//     else
//     {
//         /* Set the thread access mask to maximum */
//         Thread->GrantedAccess = THREAD_ALL_ACCESS;
//     }
//
//     /* Dispatch thread */
//     KeReadyThread(&Thread->Tcb);
//
//     /* Dereference it, leaving only the keep-alive */
//     ObDereferenceObject(Thread);
//
//     /* Return */
//     return Status;
//
//     /* Most annoying failure case ever, where we undo almost all manually */
// Quickie:
//     /* When we get here, the process is locked, unlock it */
//     ExReleasePushLockExclusive(&Process->ProcessLock);
//     KeLeaveCriticalRegion();
//
//     /* Uninitailize it */
//     KeUninitThread(&Thread->Tcb);
//
//     /* If we had a TEB, delete it */
//     if (TebBase) MmDeleteTeb(Process, TebBase);
//
//     /* Release rundown protection, which we also hold */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//
//     /* Dereference the thread and return failure */
//     ObDereferenceObject(Thread);
//     return STATUS_PROCESS_IS_TERMINATING;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsCreateSystemThread(OUT PHANDLE ThreadHandle,
//                      IN ACCESS_MASK DesiredAccess,
//                      IN POBJECT_ATTRIBUTES ObjectAttributes,
//                      IN HANDLE ProcessHandle,
//                      IN PCLIENT_ID ClientId,
//                      IN PKSTART_ROUTINE StartRoutine,
//                      IN PVOID StartContext)
// {
//     PEPROCESS TargetProcess = NULL;
//     HANDLE Handle = ProcessHandle;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG,
//             "ProcessHandle: %p StartRoutine: %p StartContext: %p\n",
//             ProcessHandle, StartRoutine, StartContext);
//
//     /* Check if we have a handle. If not, use the System Process */
//     if (!ProcessHandle)
//     {
//         Handle = NULL;
//         TargetProcess = PsInitialSystemProcess;
//     }
//
//     /* Call the shared function */
//     return PspCreateThread(ThreadHandle,
//                            DesiredAccess,
//                            ObjectAttributes,
//                            Handle,
//                            TargetProcess,
//                            ClientId,
//                            NULL,
//                            NULL,
//                            FALSE,
//                            StartRoutine,
//                            StartContext);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsLookupThreadByThreadId(IN HANDLE ThreadId,
//                          OUT PETHREAD *Thread)
// {
//     PHANDLE_TABLE_ENTRY CidEntry;
//     PETHREAD FoundThread;
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG, "ThreadId: %p\n", ThreadId);
//     KeEnterCriticalRegion();
//
//     /* Get the CID Handle Entry */
//     CidEntry = ExMapHandleToPointer(PspCidTable, ThreadId);
//     if (CidEntry)
//     {
//         /* Get the Thread */
//         FoundThread = CidEntry->Object;
//
//         /* Make sure it's really a thread */
//         if (FoundThread->Tcb.Header.Type == ThreadObject)
//         {
//             /* Safe Reference and return it */
//             if (ObReferenceObjectSafe(FoundThread))
//             {
//                 *Thread = FoundThread;
//                 Status = STATUS_SUCCESS;
//             }
//         }
//
//         /* Unlock the Entry */
//         ExUnlockHandleTableEntry(PspCidTable, CidEntry);
//     }
//
//     /* Return to caller */
//     KeLeaveCriticalRegion();
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetThreadFreezeCount(IN PETHREAD Thread)
// {
//     return Thread->Tcb.FreezeCount;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsGetThreadHardErrorsAreDisabled(IN PETHREAD Thread)
// {
//     return Thread->HardErrorsAreDisabled ? TRUE : FALSE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetThreadId(IN PETHREAD Thread)
// {
//     return Thread->Cid.UniqueThread;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetCurrentThreadId(VOID)
// {
//     return PsGetCurrentThread()->Cid.UniqueThread;
// }
//
// /*
//  * @implemented
//  */
// PEPROCESS
// NTAPI
// PsGetThreadProcess(IN PETHREAD Thread)
// {
//     return Thread->ThreadsProcess;
// }
//
// /*
//  * @implemented
//  */
// PEPROCESS
// NTAPI
// PsGetCurrentThreadProcess(VOID)
// {
//     return PsGetCurrentThread()->ThreadsProcess;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetThreadProcessId(IN PETHREAD Thread)
// {
//     return Thread->Cid.UniqueProcess;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetCurrentThreadProcessId(VOID)
// {
//     return PsGetCurrentThread()->Cid.UniqueProcess;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetThreadSessionId(IN PETHREAD Thread)
// {
//     return MmGetSessionId(Thread->ThreadsProcess);
// }
//
// /*
//  * @implemented
//  */
// PTEB
// NTAPI
// PsGetThreadTeb(IN PETHREAD Thread)
// {
//     return Thread->Tcb.Teb;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentThreadTeb(VOID)
// {
//     return PsGetCurrentThread()->Tcb.Teb;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetThreadWin32Thread(IN PETHREAD Thread)
// {
//     return Thread->Tcb.Win32Thread;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentThreadWin32Thread(VOID)
// {
//     return PsGetCurrentThread()->Tcb.Win32Thread;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentThreadWin32ThreadAndEnterCriticalRegion(
//     _Out_ HANDLE* OutProcessId)
// {
//     PETHREAD CurrentThread;
//
//     /* Get the current thread */
//     CurrentThread = PsGetCurrentThread();
//
//     /* Return the process id */
//     *OutProcessId = CurrentThread->Cid.UniqueProcess;
//
//     /* Enter critical region */
//     KeEnterCriticalRegion();
//
//     /* Return the win32 thread */
//     return CurrentThread->Tcb.Win32Thread;
// }
//
// /*
//  * @implemented
//  */
// KPROCESSOR_MODE
// NTAPI
// PsGetCurrentThreadPreviousMode(VOID)
// {
//     return (KPROCESSOR_MODE)PsGetCurrentThread()->Tcb.PreviousMode;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentThreadStackBase(VOID)
// {
//     return PsGetCurrentThread()->Tcb.StackBase;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentThreadStackLimit(VOID)
// {
//     return (PVOID)PsGetCurrentThread()->Tcb.StackLimit;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsIsThreadTerminating(IN PETHREAD Thread)
// {
//     return Thread->Terminated ? TRUE : FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsIsSystemThread(IN PETHREAD Thread)
// {
//     return Thread->SystemThread ? TRUE: FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsIsThreadImpersonating(IN PETHREAD Thread)
// {
//     return Thread->ActiveImpersonationInfo ? TRUE : FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsSetThreadHardErrorsAreDisabled(IN PETHREAD Thread,
//                                  IN BOOLEAN HardErrorsAreDisabled)
// {
//     Thread->HardErrorsAreDisabled = HardErrorsAreDisabled;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsSetThreadWin32Thread(
//     _Inout_ PETHREAD Thread,
//     _In_ PVOID Win32Thread,
//     _In_ PVOID OldWin32Thread)
// {
//     /* Are we setting the win32 process? */
//     if (Win32Thread != NULL)
//     {
//         /* Just exchange it */
//         return InterlockedExchangePointer(&Thread->Tcb.Win32Thread,
//                                           Win32Thread);
//     }
//     else
//     {
//         /* We are resetting, only exchange when the old win32 thread matches */
//         return InterlockedCompareExchangePointer(&Thread->Tcb.Win32Thread,
//                                                  Win32Thread,
//                                                  OldWin32Thread);
//     }
// }
//
// NTSTATUS
// NTAPI
// PsWrapApcWow64Thread(IN OUT PVOID *ApcContext,
//                      IN OUT PVOID *ApcRoutine)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtCreateThread(OUT PHANDLE ThreadHandle,
//                IN ACCESS_MASK DesiredAccess,
//                IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                IN HANDLE ProcessHandle,
//                OUT PCLIENT_ID ClientId,
//                IN PCONTEXT ThreadContext,
//                IN PINITIAL_TEB InitialTeb,
//                IN BOOLEAN CreateSuspended)
// {
//     INITIAL_TEB SafeInitialTeb;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG,
//             "ProcessHandle: %p Context: %p\n", ProcessHandle, ThreadContext);
//
//     /* Check if this was from user-mode */
//     if (KeGetPreviousMode() != KernelMode)
//     {
//         /* Make sure that we got a context */
//         if (!ThreadContext) return STATUS_INVALID_PARAMETER;
//
//         /* Protect checks */
//         _SEH2_TRY
//         {
//             /* Make sure the handle pointer we got is valid */
//             ProbeForWriteHandle(ThreadHandle);
//
//             /* Check if the caller wants a client id */
//             if (ClientId)
//             {
//                 /* Make sure we can write to it */
//                 ProbeForWrite(ClientId, sizeof(CLIENT_ID), sizeof(ULONG));
//             }
//
//             /* Make sure that the entire context is readable */
//             ProbeForRead(ThreadContext, sizeof(CONTEXT), sizeof(ULONG));
//
//             /* Check the Initial TEB */
//             ProbeForRead(InitialTeb, sizeof(INITIAL_TEB), sizeof(ULONG));
//             SafeInitialTeb = *InitialTeb;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Use the Initial TEB as is */
//         SafeInitialTeb = *InitialTeb;
//     }
//
//     /* Call the shared function */
//     return PspCreateThread(ThreadHandle,
//                            DesiredAccess,
//                            ObjectAttributes,
//                            ProcessHandle,
//                            NULL,
//                            ClientId,
//                            ThreadContext,
//                            &SafeInitialTeb,
//                            CreateSuspended,
//                            NULL,
//                            NULL);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenThread(OUT PHANDLE ThreadHandle,
//              IN ACCESS_MASK DesiredAccess,
//              IN POBJECT_ATTRIBUTES ObjectAttributes,
//              IN PCLIENT_ID ClientId OPTIONAL)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     CLIENT_ID SafeClientId;
//     ULONG Attributes = 0;
//     HANDLE hThread = NULL;
//     NTSTATUS Status;
//     PETHREAD Thread;
//     BOOLEAN HasObjectName = FALSE;
//     ACCESS_STATE AccessState;
//     AUX_ACCESS_DATA AuxData;
//     PAGED_CODE();
//     PSTRACE(PS_THREAD_DEBUG,
//             "ClientId: %p ObjectAttributes: %p\n", ClientId, ObjectAttributes);
//
//     /* Check if we were called from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the thread handle */
//             ProbeForWriteHandle(ThreadHandle);
//
//             /* Check for a CID structure */
//             if (ClientId)
//             {
//                 /* Probe and capture it */
//                 ProbeForRead(ClientId, sizeof(CLIENT_ID), sizeof(ULONG));
//                 SafeClientId = *ClientId;
//                 ClientId = &SafeClientId;
//             }
//
//             /*
//              * Just probe the object attributes structure, don't capture it
//              * completely. This is done later if necessary
//              */
//             ProbeForRead(ObjectAttributes,
//                          sizeof(OBJECT_ATTRIBUTES),
//                          sizeof(ULONG));
//             HasObjectName = (ObjectAttributes->ObjectName != NULL);
//
//             /* Validate user attributes */
//             Attributes = ObpValidateAttributes(ObjectAttributes->Attributes, PreviousMode);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Otherwise just get the data directly */
//         HasObjectName = (ObjectAttributes->ObjectName != NULL);
//
//         /* Still have to sanitize attributes */
//         Attributes = ObpValidateAttributes(ObjectAttributes->Attributes, PreviousMode);
//     }
//
//     /* Can't pass both, fail */
//     if ((HasObjectName) && (ClientId)) return STATUS_INVALID_PARAMETER_MIX;
//
//     /* Create an access state */
//     Status = SeCreateAccessState(&AccessState,
//                                  &AuxData,
//                                  DesiredAccess,
//                                  &PsThreadType->TypeInfo.GenericMapping);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if this is a debugger */
//     if (SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode))
//     {
//         /* Did he want full access? */
//         if (AccessState.RemainingDesiredAccess & MAXIMUM_ALLOWED)
//         {
//             /* Give it to him */
//             AccessState.PreviouslyGrantedAccess |= THREAD_ALL_ACCESS;
//         }
//         else
//         {
//             /* Otherwise just give every other access he could want */
//             AccessState.PreviouslyGrantedAccess |=
//                 AccessState.RemainingDesiredAccess;
//         }
//
//         /* The caller desires nothing else now */
//         AccessState.RemainingDesiredAccess = 0;
//     }
//
//     /* Open by name if one was given */
//     if (HasObjectName)
//     {
//         /* Open it */
//         Status = ObOpenObjectByName(ObjectAttributes,
//                                     PsThreadType,
//                                     PreviousMode,
//                                     &AccessState,
//                                     0,
//                                     NULL,
//                                     &hThread);
//
//         /* Get rid of the access state */
//         SeDeleteAccessState(&AccessState);
//     }
//     else if (ClientId)
//     {
//         /* Open by Thread ID */
//         if (ClientId->UniqueProcess)
//         {
//             /* Get the Process */
//             Status = PsLookupProcessThreadByCid(ClientId, NULL, &Thread);
//         }
//         else
//         {
//             /* Get the Process */
//             Status = PsLookupThreadByThreadId(ClientId->UniqueThread, &Thread);
//         }
//
//         /* Check if we didn't find anything */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Get rid of the access state and return */
//             SeDeleteAccessState(&AccessState);
//             return Status;
//         }
//
//         /* Open the Thread Object */
//         Status = ObOpenObjectByPointer(Thread,
//                                        Attributes,
//                                        &AccessState,
//                                        0,
//                                        PsThreadType,
//                                        PreviousMode,
//                                        &hThread);
//
//         /* Delete the access state and dereference the thread */
//         SeDeleteAccessState(&AccessState);
//         ObDereferenceObject(Thread);
//     }
//     else
//     {
//         /* Neither an object name nor a client id was passed */
//         return STATUS_INVALID_PARAMETER_MIX;
//     }
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Protect against bad user-mode pointers */
//         _SEH2_TRY
//         {
//             /* Write back the handle */
//             *ThreadHandle = hThread;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
