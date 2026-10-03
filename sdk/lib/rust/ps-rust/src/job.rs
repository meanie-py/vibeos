//! ntoskrnl/ps/job.c — job objects (upstream 55563444377 + b272427baa3: create, open, assign,
//! remove, terminate, close/delete, limits, accounting, process-id list, completion ports and
//! the win32k UI-restriction callout).
//!
//! Translated 2026-10-03 (rustconv, sealed, DeepSeek; banked under tools/rustconv/.responses/
//! 20261003-104202-job.c-run-*) and integrated against this crate's conventions: every
//! `_SEH2_TRY` user access goes through `ntkrnl_sys::probe`, so a hostile pointer returns its
//! status instead of faulting the kernel. The model dropped the guard on the `*JobHandle = Handle`
//! write-backs in NtCreateJobObject/NtOpenJobObject, and the C has it, so it is restored.
//!
//! Divergences from the C, each because the C cannot be expressed without SEH:
//! - PspQueryJobProcessIdList fills the caller's buffer from inside the job lock. The C wraps
//!   the whole enumeration in one `_SEH2_TRY` and does read-modify-write on the user header.
//!   Here the counts are kept in the context and every user write is a fault-tolerant copy, with
//!   the same results and the same buffer contents on success.
//! - PspJobInfoAlign[JobObjectAssociateCompletionPortInformation] is sizeof(HANDLE) as the C says.
//!   The previous translation had sizeof(ULONG), which under-checked the alignment on 64-bit.

use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use ntkrnl_sys::base::{FALSE, NT_SUCCESS, PAGE_SHIFT, STATUS_NOT_IMPLEMENTED, STATUS_SUCCESS, TRUE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::interlocked::InterlockedCompareExchangePointer;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, ASSERT};

use crate::win32::PspW32JobCallout;

extern "C" {
    static PsProcessType: POBJECT_TYPE;
    static IoCompletionType: POBJECT_TYPE;
    static mut KeActiveProcessors: KAFFINITY;
    static SeIncreaseBasePriorityPrivilege: LUID;
}

extern "system" {
    fn KeGetCurrentThread() -> PKTHREAD;
    fn ObCreateObject(
        mode: KPROCESSOR_MODE,
        obj_type: POBJECT_TYPE,
        oa: POBJECT_ATTRIBUTES,
        mode2: KPROCESSOR_MODE,
        parse_context: PVOID,
        size: ULONG,
        paged: ULONG,
        nonpaged: ULONG,
        obj: *mut PVOID,
    ) -> NTSTATUS;
    fn ObInsertObject(
        object: PVOID,
        context: PVOID,
        access: ACCESS_MASK,
        count: ULONG,
        new_obj: *mut PVOID,
        handle: *mut HANDLE,
    ) -> NTSTATUS;
    fn ObOpenObjectByName(
        oa: POBJECT_ATTRIBUTES,
        obj_type: POBJECT_TYPE,
        mode: KPROCESSOR_MODE,
        access_state: PVOID,
        access: ACCESS_MASK,
        parse_context: PVOID,
        handle: *mut HANDLE,
    ) -> NTSTATUS;
    fn ExInitializeResourceLite(resource: PVOID) -> NTSTATUS;
    fn ExDeleteResourceLite(resource: PVOID) -> NTSTATUS;
    fn KeInitializeEvent(event: PVOID, event_type: u32, state: BOOLEAN);
}
extern "C" {
    fn KeEnterGuardedRegionThread(thread: PKTHREAD);
    fn KeLeaveGuardedRegionThread(thread: PKTHREAD);
}

// ---- internal/ps.h -------------------------------------------------------------------------

pub const PSP_JOB_SCHEDULING_CLASSES: usize = 10;
/// The default is 5 per Yosifovich, "Windows 10 System Programming, Part 1".
pub const PSP_JOB_SCHEDULING_CLASS_DEFAULT: ULONG = 5;

const PSP_JOB_BASIC_LIMIT_VALID_FLAGS: ULONG = JOB_OBJECT_LIMIT_WORKINGSET
    | JOB_OBJECT_LIMIT_PROCESS_TIME
    | JOB_OBJECT_LIMIT_JOB_TIME
    | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
    | JOB_OBJECT_LIMIT_AFFINITY
    | JOB_OBJECT_LIMIT_PRIORITY_CLASS
    | JOB_OBJECT_LIMIT_PRESERVE_JOB_TIME
    | JOB_OBJECT_LIMIT_SCHEDULING_CLASS;
const PSP_JOB_EXTENDED_LIMIT_VALID_FLAGS: ULONG = PSP_JOB_BASIC_LIMIT_VALID_FLAGS
    | JOB_OBJECT_LIMIT_PROCESS_MEMORY
    | JOB_OBJECT_LIMIT_JOB_MEMORY
    | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION
    | JOB_OBJECT_LIMIT_BREAKAWAY_OK
    | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK
    | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

/// EPROCESS.JobStatus (the Flags2 layout introduced in 6.0).
pub const PSP_JOB_NOT_REALLY_ACTIVE: ULONG = 0x0000_0001;
pub const PSP_JOB_ACCOUNTING_FOLDED: ULONG = 0x0000_0002;

/// EJOB.JobFlags.
pub const PSP_JOB_CLOSE_DONE: ULONG = 0x0000_0001;
pub const PSP_JOB_TERMINATING: ULONG = 0x0000_0080;

// ---- globals --------------------------------------------------------------------------------

#[no_mangle]
pub static mut PsJobType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut PsJobListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

static mut PsJobListLock: _FAST_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut PspUseJobSchedulingClasses: BOOLEAN = 0;

#[no_mangle]
pub static mut PspJobSchedulingClasses: [u8; PSP_JOB_SCHEDULING_CLASSES] =
    [1 * 6, 2 * 6, 3 * 6, 4 * 6, 5 * 6, 6 * 6, 7 * 6, 8 * 6, 9 * 6, 10 * 6];

#[no_mangle]
pub static mut PspJobMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | JOB_OBJECT_QUERY,
    GenericWrite: STANDARD_RIGHTS_WRITE
        | JOB_OBJECT_ASSIGN_PROCESS
        | JOB_OBJECT_SET_ATTRIBUTES
        | JOB_OBJECT_TERMINATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: STANDARD_RIGHTS_ALL | THREAD_ALL_ACCESS, // bug fixed only in vista
};

/// Indexed by JOBOBJECTINFOCLASS.
#[no_mangle]
pub static mut PspJobInfoLengths: [ULONG; 11] = [
    0x0,
    core::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as ULONG,
    core::mem::size_of::<JOBOBJECT_BASIC_LIMIT_INFORMATION>() as ULONG,
    core::mem::size_of::<JOBOBJECT_BASIC_PROCESS_ID_LIST>() as ULONG,
    core::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as ULONG,
    core::mem::size_of::<JOBOBJECT_SECURITY_LIMIT_INFORMATION>() as ULONG,
    core::mem::size_of::<JOBOBJECT_END_OF_JOB_TIME_INFORMATION>() as ULONG,
    core::mem::size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as ULONG,
    core::mem::size_of::<JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION>() as ULONG,
    core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as ULONG,
    0x4,
];

/// Indexed by JOBOBJECTINFOCLASS.
#[no_mangle]
pub static mut PspJobInfoAlign: [ULONG; 11] = [
    0x0,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<HANDLE>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
];

#[repr(C)]
struct PSP_TERMINATE_PROCESS_CONTEXT {
    Job: PEJOB,
    ExitStatus: NTSTATUS,
}

#[repr(C)]
struct PSP_QUERY_JOB_PROCESS_ID_CONTEXT {
    /// The caller's buffer: written only through fault-tolerant copies.
    ProcessIdList: PJOBOBJECT_BASIC_PROCESS_ID_LIST,
    NextProcessId: *mut ULONG_PTR,
    /// SIZE_T in the C (pointer-sized).
    RemainingLength: usize,
    /// Kernel-side shadows of the two header counts (see the module doc).
    NumberOfAssignedProcesses: ULONG,
    NumberOfProcessIdsInList: ULONG,
}

// ---- helpers --------------------------------------------------------------------------------

#[inline(always)]
fn FlagOn(flags: ULONG, flag: ULONG) -> bool {
    flags & flag != 0
}

/// `InterlockedOr((PLONG)&x, v)`: returns the previous value.
#[inline(always)]
unsafe fn InterlockedOrUlong(target: *mut ULONG, value: ULONG) -> ULONG {
    (*(target as *const AtomicI32)).fetch_or(value as i32, Ordering::SeqCst) as ULONG
}

/// `InterlockedAnd((PLONG)&x, v)`: returns the previous value.
#[inline(always)]
unsafe fn InterlockedAndUlong(target: *mut ULONG, value: ULONG) -> ULONG {
    (*(target as *const AtomicI32)).fetch_and(value as i32, Ordering::SeqCst) as ULONG
}

#[inline(always)]
unsafe fn InterlockedExchangeUL(target: *mut ULONG, value: ULONG) -> ULONG {
    (*(target as *const AtomicU32)).swap(value, Ordering::SeqCst)
}

/// `CONTAINING_RECORD(Entry, EPROCESS, JobLinks)`.
#[inline(always)]
unsafe fn ProcessFromJobLinks(entry: PLIST_ENTRY) -> PEPROCESS {
    (entry as *mut u8).sub(core::mem::offset_of!(_EPROCESS, JobLinks)) as PEPROCESS
}

#[inline(always)]
unsafe fn JobLockHeld(job: PEJOB) -> bool {
    ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!((*job).JobLock)) != 0
        || ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*job).JobLock)) != 0
}

/// `DPRINT1(...)`, through the crate's Rust-side formatter (no variadic FFI hop).
macro_rules! DPRINT1 {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        crate::dbg_print!(concat!("(ntoskrnl/ps/job.c) ", $fmt, "\0").as_bytes() $(, $arg)*)
    };
}

// ---- implementation -------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn PspInitializeJobStructures() {
    InitializeListHead(core::ptr::addr_of_mut!(PsJobListHead));
    ExInitializeFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
}

/// The next process directly assigned to `job`, with the job lock held (shared or exclusive).
/// Neither `previous` nor the result is referenced.
unsafe fn PspGetNextProcessInJobLocked(job: PEJOB, previous: PEPROCESS) -> PEPROCESS {
    ASSERT!(JobLockHeld(job));

    let entry: PLIST_ENTRY = if !previous.is_null() {
        ASSERT!((*previous).Job == job);
        ASSERT!(!IsListEmpty(core::ptr::addr_of_mut!((*previous).JobLinks)));
        (*previous).JobLinks.Flink
    } else {
        (*job).ProcessListHead.Flink
    };

    if entry == core::ptr::addr_of_mut!((*job).ProcessListHead) {
        return core::ptr::null_mut();
    }

    ASSERT!(!(*entry).Flink.is_null());
    ASSERT!(!(*entry).Blink.is_null());
    ASSERT!((*(*entry).Flink).Blink == entry);
    ASSERT!((*(*entry).Blink).Flink == entry);

    let process = ProcessFromJobLinks(entry);
    ASSERT!((*process).Job == job);
    process
}

/// The next REFERENCED process in `job`. Consumes the reference on `previous` (if any). The job
/// lock is not held on return.
unsafe fn PspGetNextProcessInJob(job: PEJOB, previous: PEPROCESS) -> PEPROCESS {
    let mut next: PEPROCESS = core::ptr::null_mut();

    ExEnterCriticalRegionAndAcquireResourceShared(core::ptr::addr_of_mut!((*job).JobLock));

    let mut candidate = previous;
    loop {
        candidate = PspGetNextProcessInJobLocked(job, candidate);
        if candidate.is_null() {
            break;
        }
        // Skip processes whose deletion has begun; JobLinks stays valid under the lock.
        if ObReferenceObjectSafe(candidate as PVOID) != 0 {
            next = candidate;
            break;
        }
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));

    // After releasing the lock: the dereference can run the process delete procedure, which
    // removes JobLinks under the same lock.
    if !previous.is_null() {
        ObDereferenceObject(previous as PVOID);
    }

    next
}

/// Call `callback` for every process in `job`, WITHOUT the job lock held. Stops at the first
/// failure and returns it.
#[no_mangle]
pub unsafe extern "system" fn PspEnumerateProcessesInJob(
    job: PEJOB,
    callback: PJOB_ENUMERATOR_CALLBACK,
    context: PVOID,
) -> NTSTATUS {
    let job = opaque_ptr(job);
    let context = opaque_ptr(context);
    let Some(callback) = callback else {
        return STATUS_INVALID_PARAMETER;
    };
    let mut status = STATUS_SUCCESS;

    let mut process = PspGetNextProcessInJob(job, core::ptr::null_mut());
    while !process.is_null() {
        status = callback(process, context);
        if !NT_SUCCESS(status) {
            // On success PspGetNextProcessInJob consumes this reference; on failure release it.
            ObDereferenceObject(process as PVOID);
            break;
        }
        process = PspGetNextProcessInJob(job, process);
    }

    status
}

/// Same, with the job lock held by the caller. The callback must not touch the lock, change
/// membership, or keep the process pointer.
unsafe fn PspEnumerateProcessesInJobLocked(
    job: PEJOB,
    callback: unsafe extern "system" fn(PEPROCESS, PVOID) -> NTSTATUS,
    context: PVOID,
) -> NTSTATUS {
    ASSERT!(JobLockHeld(job));

    let mut status = STATUS_SUCCESS;
    let mut process = PspGetNextProcessInJobLocked(job, core::ptr::null_mut());
    while !process.is_null() {
        status = callback(process, context);
        if !NT_SUCCESS(status) {
            break;
        }
        process = PspGetNextProcessInJobLocked(job, process);
    }
    status
}

/// Queue `message` to the job's completion port. Job lock held; the port must exist.
#[no_mangle]
pub unsafe extern "system" fn PspSendJobMessageLocked(
    job: PEJOB,
    message: ULONG,
    completion_value: PVOID,
    quota: BOOLEAN,
) -> NTSTATUS {
    let job = opaque_ptr(job);
    let completion_value = opaque_ptr(completion_value);
    ASSERT!(!(*job).CompletionPort.is_null());
    ASSERT!(JobLockHeld(job));

    IoSetIoCompletion(
        (*job).CompletionPort,
        (*job).CompletionKey,
        completion_value,
        STATUS_SUCCESS,
        message as ULONG_PTR,
        quota,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PspAssignProcessToJob(process: PEPROCESS, job: PEJOB) -> NTSTATUS {
    let process = opaque_ptr(process);
    let job = opaque_ptr(job);
    let mut status = STATUS_SUCCESS;
    let mut callout_status = STATUS_SUCCESS;

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) == 0 {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    'exit: {
        // "If the job or any of its parent jobs in the job chain is terminating when
        // AssignProcessToJob is called, the function fails"
        if FlagOn((*job).JobFlags, PSP_JOB_TERMINATING) {
            status = STATUS_INVALID_PARAMETER;
            break 'exit;
        }

        // A job flagged closed with kill-on-close accepts no more processes.
        if FlagOn((*job).LimitFlags, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
            && FlagOn((*job).JobFlags, PSP_JOB_CLOSE_DONE)
        {
            status = STATUS_INVALID_PARAMETER;
            break 'exit;
        }

        if FlagOn((*job).LimitFlags, JOB_OBJECT_LIMIT_ACTIVE_PROCESS)
            && (*job).ActiveProcesses >= (*job).ActiveProcessLimit
        {
            if !(*job).CompletionPort.is_null() {
                let _ = PspSendJobMessageLocked(
                    job,
                    JOB_OBJECT_MSG_ACTIVE_PROCESS_LIMIT,
                    core::ptr::null_mut(),
                    TRUE,
                );
            }
            status = STATUS_QUOTA_EXCEEDED;
            break 'exit;
        }

        // Take the reference Process->Job owns BEFORE publishing the pointer, so every
        // observable non-NULL Process->Job is backed by it; release it if another assignment wins.
        ObReferenceObject(job as PVOID);

        // JobLock guards this job, but a caller holding a different job's lock may be assigning
        // the same process concurrently.
        let previous_job = InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*process).Job) as *mut PVOID,
            job as PVOID,
            core::ptr::null_mut(),
        );
        if !previous_job.is_null() {
            ObDereferenceObject(job as PVOID);
            status = STATUS_ACCESS_DENIED;
            break 'exit;
        }

        // Committed: nothing below may fail. Readers of ProcessListHead are held off by JobLock.
        ASSERT!(IsListEmpty(core::ptr::addr_of_mut!((*process).JobLinks)));

        InsertTailList(
            core::ptr::addr_of_mut!((*job).ProcessListHead),
            core::ptr::addr_of_mut!((*process).JobLinks),
        );

        (*job).TotalProcesses += 1;
        (*job).ActiveProcesses += 1;

        if !(*job).CompletionPort.is_null() && !(*process).UniqueProcessId.is_null() {
            let _ = PspSendJobMessageLocked(
                job,
                JOB_OBJECT_MSG_NEW_PROCESS,
                (*process).UniqueProcessId,
                FALSE,
            );
        }

        // Hand the process to win32k, which enforces the UI restrictions. One that has not
        // connected to win32k yet is picked up when it does.
        if (*job).UIRestrictionsClass != 0 && !(*process).Win32Process.is_null() {
            callout_status = PspInvokeW32JobCallout(
                job,
                _PSW32JOBCALLOUTTYPE_PsW32JobCalloutAddProcess,
                (*process).Win32Process as PVOID,
            );
        }
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));

    // Committed, but win32k will not enforce the UI restrictions, so the process must not run.
    // The caller is told the assignment failed, since that is what it asked for.
    if !NT_SUCCESS(callout_status) {
        DPRINT1!(
            "Failed to apply the UI restrictions of job %p to process %p: 0x%lx\n",
            job,
            process,
            callout_status
        );
        let _ = PsTerminateProcess(process, callout_status);
        status = callout_status;
    }

    // TODO (as the C): ensure that job limits are respected.
    status
}

/// Mark `process` inactive in its job (job lock held exclusively). TRUE if this call made the
/// active-to-inactive transition AND left the job with no active processes.
unsafe fn PspDeactivateProcessFromJobLocked(job: PEJOB, process: PEPROCESS) -> bool {
    ASSERT!((*process).Job == job);
    ASSERT!(ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*job).JobLock)) != 0);

    if FlagOn((*process).JobStatus, PSP_JOB_NOT_REALLY_ACTIVE) {
        return false;
    }

    ASSERT!((*job).ActiveProcesses != 0);
    (*job).ActiveProcesses -= 1;

    InterlockedOrUlong(core::ptr::addr_of_mut!((*process).JobStatus), PSP_JOB_NOT_REALLY_ACTIVE);

    (*job).ActiveProcesses == 0
}

/// From PspDeleteProcess: unlink `process` from its job. It must still be linked.
#[no_mangle]
pub unsafe extern "system" fn PspRemoveProcessFromJob(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let job = (*process).Job;
    ASSERT!(!job.is_null());

    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    ASSERT!((*process).Job == job);
    ASSERT!(!(*process).JobLinks.Flink.is_null());
    ASSERT!(!(*process).JobLinks.Blink.is_null());
    ASSERT!(!IsListEmpty(core::ptr::addr_of_mut!((*process).JobLinks)));

    RemoveEntryList(core::ptr::addr_of_mut!((*process).JobLinks));
    InitializeListHead(core::ptr::addr_of_mut!((*process).JobLinks));

    let active_process_zero = PspDeactivateProcessFromJobLocked(job, process);

    // TODO (as the C): ensure that job limits are respected.

    if active_process_zero && !(*job).CompletionPort.is_null() {
        let _ = PspSendJobMessageLocked(
            job,
            JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO,
            core::ptr::null_mut(),
            FALSE,
        );
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
}

/// From PspExitThread, when the last thread exits. The process must be in a job.
#[no_mangle]
pub unsafe extern "system" fn PspExitProcessFromJob(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let job = (*process).Job;
    ASSERT!(!job.is_null());

    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    // Job membership is immutable in the current implementation.
    ASSERT!((*process).Job == job);

    let active_process_zero = PspDeactivateProcessFromJobLocked(job, process);

    if active_process_zero && !(*job).CompletionPort.is_null() {
        let _ = PspSendJobMessageLocked(
            job,
            JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO,
            core::ptr::null_mut(),
            FALSE,
        );
    }

    // TODO (as the C): ensure that job limits are respected.

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
}

/// Terminate one process of the job and update the active count. Runs WITHOUT the job lock;
/// `process` carries the enumerator's reference. Always STATUS_SUCCESS.
unsafe extern "system" fn PspTerminateProcessCallback(process: PEPROCESS, context: PVOID) -> NTSTATUS {
    let terminate_context = context as *mut PSP_TERMINATE_PROCESS_CONTEXT;
    let job = (*terminate_context).Job;
    let exit_status = (*terminate_context).ExitStatus;

    ASSERT!(!job.is_null());
    ASSERT!((*process).Job == job);

    // Do not enter termination for a process that already made its active transition.
    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    'exit: {
        if FlagOn((*process).JobStatus, PSP_JOB_NOT_REALLY_ACTIVE) {
            break 'exit;
        }

        // STATUS_NOTHING_TO_TERMINATE (no threads) leaves accounting to the ordinary exit.
        if !NT_SUCCESS(PsTerminateProcess(process, exit_status)) {
            break 'exit;
        }

        if PspDeactivateProcessFromJobLocked(job, process) {
            // Signalled only in the termination path.
            KeSetEvent(core::ptr::addr_of_mut!((*job).Event), IO_NO_INCREMENT as KPRIORITY, FALSE);

            if !(*job).CompletionPort.is_null() {
                let _ = PspSendJobMessageLocked(
                    job,
                    JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO,
                    core::ptr::null_mut(),
                    FALSE,
                );
            }
        }
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PspTerminateJobObject(job: PEJOB, exit_status: NTSTATUS) -> NTSTATUS {
    let job = opaque_ptr(job);

    let previous_flags = InterlockedOrUlong(core::ptr::addr_of_mut!((*job).JobFlags), PSP_JOB_TERMINATING);

    // Idempotent: another caller already owns the traversal.
    if previous_flags & PSP_JOB_TERMINATING != 0 {
        return STATUS_SUCCESS;
    }

    let mut context = PSP_TERMINATE_PROCESS_CONTEXT { Job: job, ExitStatus: exit_status };

    let status = PspEnumerateProcessesInJob(
        job,
        Some(PspTerminateProcessCallback),
        core::ptr::addr_of_mut!(context) as PVOID,
    );

    // The callback always succeeds; per-process failures are handled there.
    ASSERT!(NT_SUCCESS(status));

    InterlockedAndUlong(core::ptr::addr_of_mut!((*job).JobFlags), !PSP_JOB_TERMINATING);

    status
}

/// Close procedure of the job object type: terminates the job on the last close when
/// JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE is set, then drops the completion port.
#[no_mangle]
pub unsafe extern "system" fn PspCloseJob(
    _process: PEPROCESS,
    object_body: PVOID,
    _granted_access: ACCESS_MASK,
    _process_handle_count: ULONG_PTR,
    system_handle_count: ULONG_PTR,
) {
    let job = opaque_ptr(object_body) as PEJOB;

    // Proceed only when the last handle is left.
    if system_handle_count > 1 {
        DPRINT1!(
            "PspJobClose called with unexpected SystemHandleCount: %lu\n",
            system_handle_count as ULONG
        );
        return;
    }

    InterlockedOrUlong(core::ptr::addr_of_mut!((*job).JobFlags), PSP_JOB_CLOSE_DONE);

    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    if FlagOn((*job).LimitFlags, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE) {
        // Keep the completion port during termination so the final messages are delivered.
        ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));

        let status = PspTerminateJobObject(job, STATUS_SUCCESS);
        ASSERT!(NT_SUCCESS(status));

        ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));
    }

    let completion_port = (*job).CompletionPort;
    (*job).CompletionPort = core::ptr::null_mut();
    (*job).CompletionKey = core::ptr::null_mut();

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));

    if !completion_port.is_null() {
        ObDereferenceObject(completion_port);
    }
}

/// Delete procedure of the job object type.
#[no_mangle]
pub unsafe extern "system" fn PspDeleteJob(object_body: PVOID) {
    let job = opaque_ptr(object_body) as PEJOB;

    // Let win32k tear down any per-job UI restriction state.
    if (*job).UIRestrictionsClass != 0 {
        let _ = PspInvokeW32JobCallout(job, _PSW32JOBCALLOUTTYPE_PsW32JobCalloutTerminate, core::ptr::null_mut());
        (*job).UIRestrictionsClass = 0;
    }

    (*job).LimitFlags = 0;

    if !(*job).CompletionPort.is_null() {
        ObDereferenceObject((*job).CompletionPort);
        (*job).CompletionPort = core::ptr::null_mut();
    }

    // TODO (as the C): job sets.

    ExAcquireFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
    ASSERT!(!IsListEmpty(core::ptr::addr_of_mut!((*job).JobLinks)));
    RemoveEntryList(core::ptr::addr_of_mut!((*job).JobLinks));
    ExReleaseFastMutex(core::ptr::addr_of_mut!(PsJobListLock));

    // TODO (as the C): security information.

    ExDeleteResourceLite(core::ptr::addr_of_mut!((*job).JobLock) as PVOID);
}

/// Apply basic or extended limits. APCs must already be disabled by the caller.
unsafe fn PspSetJobLimitsBasicOrExtended(
    job: PEJOB,
    extended_limit: *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    is_extended_limit: bool,
) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;
    let previous_mode = ExGetPreviousMode();
    let basic = &(*extended_limit).BasicLimitInformation;

    ASSERT!(KeAreAllApcsDisabled() != 0);

    let allowed_flags = if is_extended_limit {
        PSP_JOB_EXTENDED_LIMIT_VALID_FLAGS
    } else {
        PSP_JOB_BASIC_LIMIT_VALID_FLAGS
    };

    if basic.LimitFlags & !allowed_flags != 0 {
        DPRINT1!("Invalid LimitFlags specified: 0x%08X\n", basic.LimitFlags & !allowed_flags);
        return STATUS_INVALID_PARAMETER;
    }

    if basic.LimitFlags & JOB_OBJECT_LIMIT_PRESERVE_JOB_TIME != 0
        && basic.LimitFlags & JOB_OBJECT_LIMIT_JOB_TIME != 0
    {
        DPRINT1!("Invalid LimitFlags combination specified (PRESERVE_JOB_TIME and JOB_TIME are mutually exclusive)\n");
        return STATUS_INVALID_PARAMETER;
    }

    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*job).JobLock), TRUE);

    'exit_from_basic_limits: {
        // ---- basic limits ----
        if basic.LimitFlags & JOB_OBJECT_LIMIT_WORKINGSET != 0 {
            // Max nonzero needs min nonzero and vice versa; min may not exceed max.
            if (basic.MaximumWorkingSetSize > 0 && basic.MinimumWorkingSetSize == 0)
                || (basic.MinimumWorkingSetSize > 0 && basic.MaximumWorkingSetSize == 0)
                || basic.MaximumWorkingSetSize < basic.MinimumWorkingSetSize
            {
                status = STATUS_INVALID_PARAMETER;
                break 'exit_from_basic_limits;
            }
            // EJOB keeps them as ULONG (implicit truncation in the C).
            (*job).MinimumWorkingSetSize = basic.MinimumWorkingSetSize as ULONG;
            (*job).MaximumWorkingSetSize = basic.MaximumWorkingSetSize as ULONG;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_PROCESS_TIME != 0 {
            (*job).PerProcessUserTimeLimit.QuadPart = basic.PerProcessUserTimeLimit.QuadPart;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_JOB_TIME != 0 {
            (*job).PerJobUserTimeLimit.QuadPart = basic.PerJobUserTimeLimit.QuadPart;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_ACTIVE_PROCESS != 0 {
            (*job).ActiveProcessLimit = basic.ActiveProcessLimit;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_AFFINITY != 0 {
            // Must be a subset of the system affinity (KeActiveProcessors).
            if basic.Affinity != (basic.Affinity & KeActiveProcessors as ULONG_PTR) {
                status = STATUS_INVALID_PARAMETER;
                break 'exit_from_basic_limits;
            }
            (*job).Affinity = basic.Affinity as ULONG;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_PRIORITY_CLASS != 0 {
            if basic.PriorityClass > PROCESS_PRIORITY_CLASS_ABOVE_NORMAL
                || basic.PriorityClass <= PROCESS_PRIORITY_CLASS_INVALID
            {
                status = STATUS_INVALID_PARAMETER;
                break 'exit_from_basic_limits;
            }
            // "The calling process must enable the SE_INC_BASE_PRIORITY_NAME privilege"
            if SeCheckPrivilegedObject(
                SeIncreaseBasePriorityPrivilege,
                job as HANDLE,
                JOB_OBJECT_SET_ATTRIBUTES,
                previous_mode,
            ) != 0
            {
                (*job).PriorityClass = basic.PriorityClass as UCHAR;
            } else {
                status = STATUS_PRIVILEGE_NOT_HELD;
                break 'exit_from_basic_limits;
            }
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_SCHEDULING_CLASS != 0 {
            if basic.SchedulingClass as usize >= PSP_JOB_SCHEDULING_CLASSES {
                status = STATUS_INVALID_PARAMETER;
                break 'exit_from_basic_limits;
            }
            // "To use a scheduling class greater than 5, the calling process must enable the
            // SE_INC_BASE_PRIORITY_NAME privilege"
            if basic.SchedulingClass > PSP_JOB_SCHEDULING_CLASS_DEFAULT {
                if SeCheckPrivilegedObject(
                    SeIncreaseBasePriorityPrivilege,
                    job as HANDLE,
                    JOB_OBJECT_SET_ATTRIBUTES,
                    previous_mode,
                ) != 0
                {
                    (*job).SchedulingClass = basic.SchedulingClass;
                } else {
                    status = STATUS_PRIVILEGE_NOT_HELD;
                    break 'exit_from_basic_limits;
                }
            } else {
                (*job).SchedulingClass = basic.SchedulingClass;
            }
        }

        // ---- extended memory limits ----
        KeAcquireGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));

        if basic.LimitFlags & JOB_OBJECT_LIMIT_PROCESS_MEMORY != 0 {
            (*job).ProcessMemoryLimit = ((*extended_limit).ProcessMemoryLimit >> PAGE_SHIFT) as ULONG;
        }

        if basic.LimitFlags & JOB_OBJECT_LIMIT_JOB_MEMORY != 0 {
            (*job).JobMemoryLimit = ((*extended_limit).JobMemoryLimit >> PAGE_SHIFT) as ULONG;
        }

        // Includes the extended limits that only set a flag.
        (*job).LimitFlags = basic.LimitFlags;

        KeReleaseGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
    status
}

/// Initial NEW_PROCESS notification for a process already in the job (job lock exclusive).
unsafe extern "system" fn PspAssociateCompletionPortCallback(process: PEPROCESS, context: PVOID) -> NTSTATUS {
    let job = context as PEJOB;

    ASSERT!((*process).Job == job);
    ASSERT!(!(*job).CompletionPort.is_null());
    ASSERT!(ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*job).JobLock)) != 0);

    if !FlagOn((*process).JobStatus, PSP_JOB_NOT_REALLY_ACTIVE) && !(*process).UniqueProcessId.is_null() {
        let _ = PspSendJobMessageLocked(job, JOB_OBJECT_MSG_NEW_PROCESS, (*process).UniqueProcessId, FALSE);
    }

    STATUS_SUCCESS
}

/// Associate an I/O completion port with the job and announce its current processes.
unsafe fn PspAssociateCompletionPortWithJob(
    job: PEJOB,
    associate_cp_info: *const JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
) -> NTSTATUS {
    let previous_mode = ExGetPreviousMode();
    let mut io_completion: PVOID = core::ptr::null_mut();

    ASSERT!(KeAreAllApcsDisabled() != 0);

    if (*associate_cp_info).CompletionPort.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let status = ObReferenceObjectByHandle(
        (*associate_cp_info).CompletionPort,
        IO_COMPLETION_MODIFY_STATE,
        IoCompletionType,
        previous_mode,
        &mut io_completion,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return status;
    }

    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*job).JobLock), TRUE);

    // Already associated, or the job is in a final state.
    if !(*job).CompletionPort.is_null() || FlagOn((*job).JobFlags, PSP_JOB_CLOSE_DONE) {
        ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
        ObDereferenceObject(io_completion);
        return STATUS_INVALID_PARAMETER;
    }

    (*job).CompletionKey = (*associate_cp_info).CompletionKey;
    (*job).CompletionPort = io_completion;

    // Assignment is serialized by JobLock, so every process is covered either here or by the
    // normal assignment path.
    let status = PspEnumerateProcessesInJobLocked(job, PspAssociateCompletionPortCallback, job as PVOID);
    ASSERT!(NT_SUCCESS(status));

    ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));

    // Committed. Initial notifications are best effort.
    STATUS_SUCCESS
}

unsafe fn PspQueryJobBasicAccountingInfo(
    job: PEJOB,
    basic_and_io: *mut JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION,
) -> NTSTATUS {
    let mut values: PROCESS_VALUES = core::mem::zeroed();
    let b = &mut *basic_and_io;

    b.BasicInfo = core::mem::zeroed();

    ExEnterCriticalRegionAndAcquireResourceShared(core::ptr::addr_of_mut!((*job).JobLock));

    // The job's accumulated accounting.
    b.BasicInfo.TotalUserTime.QuadPart = (*job).TotalUserTime.QuadPart;
    b.BasicInfo.TotalKernelTime.QuadPart = (*job).TotalKernelTime.QuadPart;
    b.BasicInfo.ThisPeriodTotalUserTime.QuadPart = (*job).ThisPeriodTotalUserTime.QuadPart;
    b.BasicInfo.ThisPeriodTotalKernelTime.QuadPart = (*job).ThisPeriodTotalKernelTime.QuadPart;
    b.BasicInfo.TotalPageFaultCount = (*job).TotalPageFaultCount;
    b.BasicInfo.TotalProcesses = (*job).TotalProcesses;
    b.BasicInfo.ActiveProcesses = (*job).ActiveProcesses;
    b.BasicInfo.TotalTerminatedProcesses = (*job).TotalTerminatedProcesses;

    // I/O info (even though some classes do not return it).
    b.IoInfo.ReadOperationCount = (*job).ReadOperationCount;
    b.IoInfo.WriteOperationCount = (*job).WriteOperationCount;
    b.IoInfo.OtherOperationCount = (*job).OtherOperationCount;
    b.IoInfo.ReadTransferCount = (*job).ReadTransferCount;
    b.IoInfo.WriteTransferCount = (*job).WriteTransferCount;
    b.IoInfo.OtherTransferCount = (*job).OtherTransferCount;

    // Plus every process whose accounting has not been folded into the job yet.
    let head = core::ptr::addr_of_mut!((*job).ProcessListHead);
    let mut next_entry = (*head).Flink;
    while next_entry != head {
        let process = ProcessFromJobLinks(next_entry);
        if !FlagOn((*process).JobStatus, PSP_JOB_ACCOUNTING_FOLDED) {
            KeQueryValuesProcess(core::ptr::addr_of_mut!((*process).Pcb), &mut values);

            b.BasicInfo.TotalUserTime.QuadPart += values.TotalUserTime.QuadPart;
            b.BasicInfo.TotalKernelTime.QuadPart += values.TotalKernelTime.QuadPart;
            b.IoInfo.ReadOperationCount += values.IoInfo.ReadOperationCount;
            b.IoInfo.WriteOperationCount += values.IoInfo.WriteOperationCount;
            b.IoInfo.OtherOperationCount += values.IoInfo.OtherOperationCount;
            b.IoInfo.ReadTransferCount += values.IoInfo.ReadTransferCount;
            b.IoInfo.WriteTransferCount += values.IoInfo.WriteTransferCount;
            b.IoInfo.OtherTransferCount += values.IoInfo.OtherTransferCount;
        }
        next_entry = (*next_entry).Flink;
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
    STATUS_SUCCESS
}

unsafe fn PspQueryJobLimitInformation(
    job: PEJOB,
    extended: bool,
    extended_limit: *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
) -> NTSTATUS {
    let current_thread = KeGetCurrentThread();
    let e = &mut *extended_limit;

    KeEnterGuardedRegionThread(current_thread);
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*job).JobLock), TRUE);

    e.BasicLimitInformation.LimitFlags = (*job).LimitFlags;
    e.BasicLimitInformation.MinimumWorkingSetSize = (*job).MinimumWorkingSetSize as SIZE_T;
    e.BasicLimitInformation.MaximumWorkingSetSize = (*job).MaximumWorkingSetSize as SIZE_T;
    e.BasicLimitInformation.ActiveProcessLimit = (*job).ActiveProcessLimit;
    e.BasicLimitInformation.PriorityClass = (*job).PriorityClass as ULONG;
    e.BasicLimitInformation.SchedulingClass = (*job).SchedulingClass;
    e.BasicLimitInformation.Affinity = (*job).Affinity as ULONG_PTR;
    e.BasicLimitInformation.PerProcessUserTimeLimit.QuadPart = (*job).PerProcessUserTimeLimit.QuadPart;
    e.BasicLimitInformation.PerJobUserTimeLimit.QuadPart = (*job).PerJobUserTimeLimit.QuadPart;

    if extended {
        KeAcquireGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));

        e.ProcessMemoryLimit = ((*job).ProcessMemoryLimit as SIZE_T) << PAGE_SHIFT;
        e.JobMemoryLimit = ((*job).JobMemoryLimit as SIZE_T) << PAGE_SHIFT;
        e.PeakProcessMemoryUsed = ((*job).PeakProcessMemoryUsed as SIZE_T) << PAGE_SHIFT;
        e.PeakJobMemoryUsed = ((*job).PeakJobMemoryUsed as SIZE_T) << PAGE_SHIFT;

        KeReleaseGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));

        // Zero IoInfo so no kernel memory leaks out.
        e.IoInfo = core::mem::zeroed();
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
    KeLeaveGuardedRegionThread(current_thread);
    STATUS_SUCCESS
}

/// Append one active process id to the caller's list (job lock held).
unsafe extern "system" fn PspQueryJobProcessIdListCallback(process: PEPROCESS, context: PVOID) -> NTSTATUS {
    let query_context = &mut *(context as *mut PSP_QUERY_JOB_PROCESS_ID_CONTEXT);

    if FlagOn((*process).JobStatus, PSP_JOB_NOT_REALLY_ACTIVE) {
        return STATUS_SUCCESS;
    }

    // Linked before its process id was assigned: not representable in this class.
    if (*process).UniqueProcessId.is_null() {
        ASSERT!(query_context.NumberOfAssignedProcesses != 0);
        query_context.NumberOfAssignedProcesses -= 1;
        return STATUS_SUCCESS;
    }

    if query_context.RemainingLength < core::mem::size_of::<ULONG_PTR>() {
        return STATUS_BUFFER_OVERFLOW;
    }

    let id = (*process).UniqueProcessId as ULONG_PTR;
    if let Err(status) = safe_copy_raw(
        query_context.NextProcessId as PVOID,
        &id as *const ULONG_PTR as PVOID,
        core::mem::size_of::<ULONG_PTR>(),
    ) {
        return status;
    }
    query_context.NextProcessId = query_context.NextProcessId.add(1);
    query_context.RemainingLength -= core::mem::size_of::<ULONG_PTR>();
    query_context.NumberOfProcessIdsInList += 1;

    STATUS_SUCCESS
}

/// Fill the caller's JOBOBJECT_BASIC_PROCESS_ID_LIST directly. `return_required_length` is a
/// kernel variable.
unsafe fn PspQueryJobProcessIdList(
    job: PEJOB,
    proc_id_list: PJOBOBJECT_BASIC_PROCESS_ID_LIST,
    job_information_length: ULONG,
    return_required_length: *mut ULONG,
) -> NTSTATUS {
    if (job_information_length as usize) < core::mem::size_of::<JOBOBJECT_BASIC_PROCESS_ID_LIST>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let id_list_offset = core::mem::offset_of!(JOBOBJECT_BASIC_PROCESS_ID_LIST, ProcessIdList);
    let mut query_context = PSP_QUERY_JOB_PROCESS_ID_CONTEXT {
        ProcessIdList: proc_id_list,
        NextProcessId: (proc_id_list as *mut u8).add(id_list_offset) as *mut ULONG_PTR,
        RemainingLength: job_information_length as usize - id_list_offset,
        NumberOfAssignedProcesses: 0,
        NumberOfProcessIdsInList: 0,
    };

    ExEnterCriticalRegionAndAcquireResourceShared(core::ptr::addr_of_mut!((*job).JobLock));

    query_context.NumberOfAssignedProcesses = (*job).ActiveProcesses;
    let mut status = PspEnumerateProcessesInJobLocked(
        job,
        PspQueryJobProcessIdListCallback,
        core::ptr::addr_of_mut!(query_context) as PVOID,
    );

    // The header the C wrote in place (before and during the walk), written once here.
    let header = [query_context.NumberOfAssignedProcesses, query_context.NumberOfProcessIdsInList];
    if let Err(fault) = safe_copy_raw(
        query_context.ProcessIdList as PVOID,
        header.as_ptr() as PVOID,
        core::mem::size_of_val(&header),
    ) {
        status = fault;
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));

    if NT_SUCCESS(status) || status == STATUS_BUFFER_OVERFLOW {
        // The bytes actually written.
        *return_required_length = (job_information_length as usize - query_context.RemainingLength) as ULONG;
    } else {
        *return_required_length = 0;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobLock(job: PEJOB) -> PVOID {
    let job = opaque_ptr(job);
    ASSERT!(!job.is_null());
    core::ptr::addr_of_mut!((*job).JobLock) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobSessionId(job: PEJOB) -> ULONG {
    let job = opaque_ptr(job);
    ASSERT!(!job.is_null());
    (*job).SessionId
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobUIRestrictionsClass(job: PEJOB) -> ULONG {
    let job = opaque_ptr(job);
    ASSERT!(!job.is_null());
    (*job).UIRestrictionsClass
}

#[no_mangle]
pub unsafe extern "system" fn PsSetJobUIRestrictionsClass(job: PEJOB, ui_restrictions_class: ULONG) {
    let job = opaque_ptr(job);
    ASSERT!(!job.is_null());
    InterlockedExchangeUL(core::ptr::addr_of_mut!((*job).UIRestrictionsClass), ui_restrictions_class);
}

/// Invoke win32k's job callout. STATUS_SUCCESS when none is registered (win32k not loaded yet).
/// FIXME (as the C): does not attach to the job's session; win32k is only ever in one.
#[no_mangle]
pub unsafe extern "system" fn PspInvokeW32JobCallout(
    job: PEJOB,
    callout_type: PSW32JOBCALLOUTTYPE,
    data: PVOID,
) -> NTSTATUS {
    let job = opaque_ptr(job);
    let data = opaque_ptr(data);
    let Some(callout) = *core::ptr::addr_of!(PspW32JobCallout) else {
        return STATUS_SUCCESS;
    };

    let mut parameters = WIN32_JOBCALLOUT_PARAMETERS {
        Job: job as PVOID,
        CalloutType: callout_type,
        Data: data,
    };
    callout(&mut parameters)
}

/// Apply a new UI restrictions class. Stored only once win32k accepts it, since win32k may fail
/// to allocate the per-job state it needs to enforce them.
unsafe fn PspSetJobUIRestrictions(job: PEJOB, ui_restrictions_class: ULONG) -> NTSTATUS {
    let mut status = STATUS_SUCCESS;

    if ui_restrictions_class & !JOB_OBJECT_UILIMIT_ALL != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    ExEnterCriticalRegionAndAcquireResourceExclusive(core::ptr::addr_of_mut!((*job).JobLock));

    // Only bother win32k if something actually changes.
    if (*job).UIRestrictionsClass != ui_restrictions_class {
        status = PspInvokeW32JobCallout(
            job,
            _PSW32JOBCALLOUTTYPE_PsW32JobCalloutSetInformation,
            ui_restrictions_class as ULONG_PTR as PVOID,
        );
        if NT_SUCCESS(status) {
            (*job).UIRestrictionsClass = ui_restrictions_class;
        }
    }

    ExReleaseResourceAndLeaveCriticalRegion(core::ptr::addr_of_mut!((*job).JobLock));
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateJobObject(
    job_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut job: PEJOB = core::ptr::null_mut();

    let previous_mode = ExGetPreviousMode();
    let current_process = PsGetCurrentProcess() as PEPROCESS;

    if previous_mode != 0 {
        if let Err(status) = probe_write_handle(job_handle as *mut PVOID, previous_mode) {
            return status;
        }
    }

    let mut status = ObCreateObject(
        previous_mode,
        PsJobType,
        object_attributes,
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<EJOB>() as ULONG,
        0,
        0,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
    );
    if !NT_SUCCESS(status) {
        DPRINT1!("Failed to create job object, Status 0x%08lx\n", status);
        return status;
    }

    core::ptr::write_bytes(job as *mut u8, 0, core::mem::size_of::<EJOB>());

    InitializeListHead(core::ptr::addr_of_mut!((*job).JobSetLinks));
    InitializeListHead(core::ptr::addr_of_mut!((*job).ProcessListHead));

    // So an early destruction does not unlink an object that was never linked.
    InitializeListHead(core::ptr::addr_of_mut!((*job).JobLinks));

    (*job).SessionId = PsGetProcessSessionId(current_process);

    KeInitializeGuardedMutex(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));
    let _ = ExInitializeResourceLite(core::ptr::addr_of_mut!((*job).JobLock) as PVOID);
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*job).Event) as PVOID,
        _EVENT_TYPE_NotificationEvent as u32,
        FALSE,
    );

    (*job).SchedulingClass = PSP_JOB_SCHEDULING_CLASS_DEFAULT;

    ExAcquireFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
    InsertTailList(core::ptr::addr_of_mut!(PsJobListHead), core::ptr::addr_of_mut!((*job).JobLinks));
    ExReleaseFastMutex(core::ptr::addr_of_mut!(PsJobListLock));

    status = ObInsertObject(
        job as PVOID,
        core::ptr::null_mut(),
        desired_access,
        0,
        core::ptr::null_mut(),
        &mut handle,
    );
    if NT_SUCCESS(status) {
        // _SEH2_TRY { *JobHandle = Handle; }
        if let Err(fault) = probe_write_user(job_handle, handle, 0) {
            status = fault;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateJobSet(
    _num_job: ULONG,
    _user_job_set: PJOB_SET_ARRAY,
    _flags: ULONG,
) -> NTSTATUS {
    DPRINT1!("NtCreateJobSet is UNIMPLEMENTED\n");
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenJobObject(
    job_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let mut handle: HANDLE = core::ptr::null_mut();

    let previous_mode = ExGetPreviousMode();

    if previous_mode != 0 {
        if let Err(status) = probe_write_handle(job_handle as *mut PVOID, previous_mode) {
            return status;
        }
    }

    let mut status = ObOpenObjectByName(
        object_attributes,
        PsJobType,
        previous_mode,
        core::ptr::null_mut(),
        desired_access,
        core::ptr::null_mut(),
        &mut handle,
    );
    if NT_SUCCESS(status) {
        // _SEH2_TRY { *JobHandle = Handle; }
        if let Err(fault) = probe_write_user(job_handle, handle, 0) {
            status = fault;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtAssignProcessToJobObject(job_handle: HANDLE, process_handle: HANDLE) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let process_handle = opaque_ptr(process_handle);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut job: PEJOB = core::ptr::null_mut();

    let previous_mode = ExGetPreviousMode();

    // JOB_OBJECT_ASSIGN_PROCESS rights are required for assignment.
    let mut status = ObReferenceObjectByHandle(
        job_handle,
        JOB_OBJECT_ASSIGN_PROCESS,
        PsJobType,
        previous_mode,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return status;
    }

    // The process handle needs PROCESS_SET_QUOTA and PROCESS_TERMINATE.
    status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_SET_QUOTA | PROCESS_TERMINATE,
        PsProcessType,
        previous_mode,
        core::ptr::addr_of_mut!(process) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        ObDereferenceObject(job as PVOID);
        return status;
    }

    // The session must match the job creator's, and the process may not already be in a job.
    let session_id = PsGetProcessSessionId(process);
    if !(*process).Job.is_null() || session_id != (*job).SessionId {
        ObDereferenceObject(job as PVOID);
        ObDereferenceObject(process as PVOID);
        return STATUS_ACCESS_DENIED;
    }

    status = PspAssignProcessToJob(process, job);

    if status == STATUS_QUOTA_EXCEEDED {
        // Preserve the assignment failure.
        let _ = PsTerminateProcess(process, STATUS_QUOTA_EXCEEDED);
    }

    ObDereferenceObject(job as PVOID);
    ObDereferenceObject(process as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtIsProcessInJob(process_handle: HANDLE, job_handle: HANDLE) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let job_handle = opaque_ptr(job_handle);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let previous_mode = ExGetPreviousMode();
    let current = process_handle == NtCurrentProcess();

    if current {
        process = PsGetCurrentProcess() as PEPROCESS;
    } else {
        status = ObReferenceObjectByHandle(
            process_handle,
            PROCESS_QUERY_INFORMATION,
            PsProcessType,
            previous_mode,
            core::ptr::addr_of_mut!(process) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    let process_job = (*process).Job;
    if !process_job.is_null() {
        if job_handle.is_null() {
            // No specific job asked about: it is in one.
            status = STATUS_PROCESS_IN_JOB;
        } else {
            let mut job_from_handle: PEJOB = core::ptr::null_mut();
            status = ObReferenceObjectByHandle(
                job_handle,
                JOB_OBJECT_QUERY,
                PsJobType,
                previous_mode,
                core::ptr::addr_of_mut!(job_from_handle) as *mut PVOID,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(status) {
                status = if process_job == job_from_handle {
                    STATUS_PROCESS_IN_JOB
                } else {
                    STATUS_PROCESS_NOT_IN_JOB
                };
                ObDereferenceObject(job_from_handle as PVOID);
            }
        }
    } else {
        status = STATUS_PROCESS_NOT_IN_JOB;
    }

    if !current {
        ObDereferenceObject(process as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtTerminateJobObject(job_handle: HANDLE, exit_status: NTSTATUS) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let mut job: PEJOB = core::ptr::null_mut();

    let previous_mode = ExGetPreviousMode();

    let mut status = ObReferenceObjectByHandle(
        job_handle,
        JOB_OBJECT_TERMINATE,
        PsJobType,
        previous_mode,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(status) {
        status = PspTerminateJobObject(job, exit_status);
        ObDereferenceObject(job as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationJobObject(
    job_handle: HANDLE,
    job_information_class: JOBOBJECTINFOCLASS,
    job_information: PVOID,
    job_information_length: ULONG,
    return_length: PULONG,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let job_information = opaque_ptr(job_information);
    let return_length = opaque_ptr(return_length);
    let mut job: PEJOB = core::ptr::null_mut();
    let status: NTSTATUS;
    let mut no_output = false;
    let mut job_info_buffer: *const core::ffi::c_void = core::ptr::null();
    let mut extended_limit: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
    let mut basic_and_io: JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION = core::mem::zeroed();
    let mut ui_restrictions: JOBOBJECT_BASIC_UI_RESTRICTIONS = core::mem::zeroed();

    let current_thread = KeGetCurrentThread();

    if job_information_class > _JOBOBJECTINFOCLASS_JobObjectJobSetInformation
        || job_information_class < _JOBOBJECTINFOCLASS_JobObjectBasicAccountingInformation
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    let mut required_length = PspJobInfoLengths[job_information_class as usize];
    let required_align = PspJobInfoAlign[job_information_class as usize];
    let mut return_required_length = required_length;

    if job_information_length != required_length {
        // Only the variable-length classes may differ, and only upwards.
        if (job_information_class != _JOBOBJECTINFOCLASS_JobObjectBasicProcessIdList
            && job_information_class != _JOBOBJECTINFOCLASS_JobObjectSecurityLimitInformation)
            || job_information_length < required_length
        {
            return STATUS_INFO_LENGTH_MISMATCH;
        }
        required_length = job_information_length;
    }

    let previous_mode = ExGetPreviousMode();

    if previous_mode != 0 {
        ASSERT!(matches!(required_align, 1 | 2 | 4 | 8 | 16));

        if !job_information.is_null() {
            if let Err(status) = probe_user_buffer(
                job_information,
                job_information_length as usize,
                required_align as usize,
                previous_mode,
            ) {
                return status;
            }
        }
        if !return_length.is_null() {
            if let Err(status) = probe_write_ulong(return_length, previous_mode) {
                return status;
            }
        }
    }

    if !job_handle.is_null() {
        let st = ObReferenceObjectByHandle(
            job_handle,
            JOB_OBJECT_QUERY,
            PsJobType,
            previous_mode,
            core::ptr::addr_of_mut!(job) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(st) {
            return st;
        }
    } else {
        // Our current process' job, if any.
        let current_process = (*current_thread).__bindgen_anon_1.ApcState.Process as PEPROCESS;
        job = (*current_process).Job;
        if job.is_null() {
            return STATUS_ACCESS_DENIED;
        }
        ObReferenceObject(job as PVOID);
    }

    match job_information_class {
        _JOBOBJECTINFOCLASS_JobObjectBasicAccountingInformation
        | _JOBOBJECTINFOCLASS_JobObjectBasicAndIoAccountingInformation => {
            status = PspQueryJobBasicAccountingInfo(job, &mut basic_and_io);
            job_info_buffer = core::ptr::addr_of!(basic_and_io) as *const _;
        }
        _JOBOBJECTINFOCLASS_JobObjectBasicLimitInformation
        | _JOBOBJECTINFOCLASS_JobObjectExtendedLimitInformation => {
            status = PspQueryJobLimitInformation(
                job,
                job_information_class == _JOBOBJECTINFOCLASS_JobObjectExtendedLimitInformation,
                &mut extended_limit,
            );
            job_info_buffer = core::ptr::addr_of!(extended_limit) as *const _;
        }
        _JOBOBJECTINFOCLASS_JobObjectBasicProcessIdList => {
            // Filled in place by the helper.
            status = PspQueryJobProcessIdList(
                job,
                job_information as PJOBOBJECT_BASIC_PROCESS_ID_LIST,
                job_information_length,
                &mut return_required_length,
            );
            no_output = true;
        }
        _JOBOBJECTINFOCLASS_JobObjectBasicUIRestrictions => {
            ui_restrictions.UIRestrictionsClass = (*job).UIRestrictionsClass;
            job_info_buffer = core::ptr::addr_of!(ui_restrictions) as *const _;
            status = STATUS_SUCCESS;
        }
        _JOBOBJECTINFOCLASS_JobObjectSecurityLimitInformation
        | _JOBOBJECTINFOCLASS_JobObjectEndOfJobTimeInformation
        | _JOBOBJECTINFOCLASS_JobObjectAssociateCompletionPortInformation
        | _JOBOBJECTINFOCLASS_JobObjectJobSetInformation => {
            DPRINT1!("Class %d not implemented\n", job_information_class as i32);
            status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            DPRINT1!("Invalid class %d\n", job_information_class as i32);
            status = STATUS_NOT_IMPLEMENTED;
        }
    }

    ObDereferenceObject(job as PVOID);

    if NT_SUCCESS(status) {
        // _SEH2_TRY { RtlCopyMemory(...); *ReturnLength = ...; }
        if !no_output {
            if let Err(fault) = safe_copy_raw(job_information, job_info_buffer, required_length as usize) {
                return fault;
            }
        }
        if !return_length.is_null() {
            if let Err(fault) = probe_write_user(return_length, return_required_length, 0) {
                return fault;
            }
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationJobObject(
    job_handle: HANDLE,
    job_information_class: JOBOBJECTINFOCLASS,
    job_information: PVOID,
    job_information_length: ULONG,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let job_information = opaque_ptr(job_information);
    let mut job: PEJOB = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let current_thread = KeGetCurrentThread();

    if job_information_class > _JOBOBJECTINFOCLASS_JobObjectJobSetInformation
        || job_information_class < _JOBOBJECTINFOCLASS_JobObjectBasicAccountingInformation
    {
        return STATUS_INVALID_INFO_CLASS;
    }

    let required_length = PspJobInfoLengths[job_information_class as usize];
    let required_align = PspJobInfoAlign[job_information_class as usize];

    let previous_mode = ExGetPreviousMode();

    if previous_mode != 0 {
        ASSERT!(matches!(required_align, 1 | 2 | 4 | 8 | 16));

        if job_information_length != 0 {
            if let Err(st) = probe_user_buffer(
                job_information,
                job_information_length as usize,
                required_align as usize,
                previous_mode,
            ) {
                return st;
            }
        }
    }

    if job_information_length != required_length {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let mut desired_access = JOB_OBJECT_SET_ATTRIBUTES;
    if job_information_class == _JOBOBJECTINFOCLASS_JobObjectSecurityLimitInformation {
        desired_access |= JOB_OBJECT_SET_SECURITY_ATTRIBUTES;
    }

    status = ObReferenceObjectByHandle(
        job_handle,
        desired_access,
        PsJobType,
        previous_mode,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return status;
    }

    // APC delivery off for the set: the handlers take locks with the unsafe variants, which
    // expect the caller to have established this state.
    KeEnterGuardedRegionThread(current_thread);

    match job_information_class {
        _JOBOBJECTINFOCLASS_JobObjectBasicLimitInformation
        | _JOBOBJECTINFOCLASS_JobObjectExtendedLimitInformation => {
            let is_extended_limit =
                job_information_class == _JOBOBJECTINFOCLASS_JobObjectExtendedLimitInformation;
            let mut extended_limit: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
            let captured = if is_extended_limit {
                safe_copy_raw(
                    core::ptr::addr_of_mut!(extended_limit) as PVOID,
                    job_information,
                    core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>(),
                )
            } else {
                safe_copy_raw(
                    core::ptr::addr_of_mut!(extended_limit.BasicLimitInformation) as PVOID,
                    job_information,
                    core::mem::size_of::<JOBOBJECT_BASIC_LIMIT_INFORMATION>(),
                )
            };
            status = match captured {
                Ok(()) => PspSetJobLimitsBasicOrExtended(job, &extended_limit, is_extended_limit),
                Err(fault) => fault,
            };
        }
        _JOBOBJECTINFOCLASS_JobObjectAssociateCompletionPortInformation => {
            let mut associate_cp_info: JOBOBJECT_ASSOCIATE_COMPLETION_PORT = core::mem::zeroed();
            status = match safe_copy_raw(
                core::ptr::addr_of_mut!(associate_cp_info) as PVOID,
                job_information,
                core::mem::size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>(),
            ) {
                Ok(()) => PspAssociateCompletionPortWithJob(job, &associate_cp_info),
                Err(fault) => fault,
            };
        }
        _JOBOBJECTINFOCLASS_JobObjectBasicUIRestrictions => {
            let mut ui_restrictions: JOBOBJECT_BASIC_UI_RESTRICTIONS = core::mem::zeroed();
            status = match safe_copy_raw(
                core::ptr::addr_of_mut!(ui_restrictions) as PVOID,
                job_information,
                core::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>(),
            ) {
                Ok(()) => PspSetJobUIRestrictions(job, ui_restrictions.UIRestrictionsClass),
                Err(fault) => fault,
            };
        }
        _JOBOBJECTINFOCLASS_JobObjectBasicAccountingInformation
        | _JOBOBJECTINFOCLASS_JobObjectBasicAndIoAccountingInformation
        | _JOBOBJECTINFOCLASS_JobObjectBasicProcessIdList
        | _JOBOBJECTINFOCLASS_JobObjectEndOfJobTimeInformation
        | _JOBOBJECTINFOCLASS_JobObjectJobSetInformation
        | _JOBOBJECTINFOCLASS_JobObjectSecurityLimitInformation => {
            DPRINT1!("Class %d not implemented\n", job_information_class as i32);
            status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            DPRINT1!("Invalid class %d\n", job_information_class as i32);
            status = STATUS_INVALID_PARAMETER;
        }
    }

    KeLeaveGuardedRegionThread(current_thread);
    ObDereferenceObject(job as PVOID);
    status
}
