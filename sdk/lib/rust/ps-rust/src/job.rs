use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED, STATUS_INSUFFICIENT_RESOURCES,
    NT_SUCCESS, TRUE, FALSE,
    APC_LEVEL, PASSIVE_LEVEL, PAGE_SHIFT};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::mm::PsInitialSystemProcess;

use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use core::mem::align_of;

extern "C" {
    static PsProcessType: POBJECT_TYPE;
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

#[inline]
unsafe fn InterlockedExchangeUL(target: *mut ULONG, value: ULONG) -> ULONG {
    use core::sync::atomic::{AtomicU32, Ordering};
    let atomic = &*(target as *const AtomicU32);
    atomic.swap(value, Ordering::SeqCst)
}

const PSP_JOB_SCHEDULING_CLASSES: usize = 10;

#[no_mangle]
pub static mut PsJobType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut PsJobListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

static mut PsJobListLock: _FAST_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut PspUseJobSchedulingClasses: BOOLEAN = 0;

#[no_mangle]
pub static mut PspJobSchedulingClasses: [u8; PSP_JOB_SCHEDULING_CLASSES] = [
    1 * 6,
    2 * 6,
    3 * 6,
    4 * 6,
    5 * 6,
    6 * 6,
    7 * 6,
    8 * 6,
    9 * 6,
    10 * 6,
];

#[no_mangle]
pub static mut PspJobMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | JOB_OBJECT_QUERY,
    GenericWrite: STANDARD_RIGHTS_WRITE | JOB_OBJECT_ASSIGN_PROCESS | JOB_OBJECT_SET_ATTRIBUTES | JOB_OBJECT_TERMINATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: STANDARD_RIGHTS_ALL | THREAD_ALL_ACCESS,
};

#[no_mangle]
pub static mut PspJobInfoLengths: [ULONG; 11] = [
    0x0,
    core::mem::size_of::<_JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_BASIC_LIMIT_INFORMATION>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_BASIC_PROCESS_ID_LIST>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_BASIC_UI_RESTRICTIONS>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_SECURITY_LIMIT_INFORMATION>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_END_OF_JOB_TIME_INFORMATION>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as ULONG,
    core::mem::size_of::<JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION>() as ULONG,
    core::mem::size_of::<_JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as ULONG,
    0x4,
];

#[no_mangle]
pub static mut PspJobInfoAlign: [ULONG; 11] = [
    0x0,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
    core::mem::size_of::<ULONG>() as ULONG,
];

#[no_mangle]
pub unsafe extern "system" fn PspDeleteJob(object_body: PVOID) {
    let object_body = opaque_ptr(object_body);
    let job = object_body as PEJOB;

    if (*job).CompletionPort != core::ptr::null_mut() {
        ObDereferenceObject((*job).CompletionPort as PVOID);
    }

    if (*job).JobLinks.Flink != core::ptr::null_mut() {
        ExAcquireFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
        RemoveEntryList(core::ptr::addr_of_mut!((*job).JobLinks) as PLIST_ENTRY);
        ExReleaseFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
    }

    ExDeleteResourceLite(core::ptr::addr_of_mut!((*job).JobLock) as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn PspInitializeJobStructures() {
    InitializeListHead(core::ptr::addr_of_mut!(PsJobListHead) as PLIST_ENTRY);
    ExInitializeFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
}

unsafe fn PspAssignProcessToJob(process: PEPROCESS, job: PEJOB) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe fn PspTerminateJobObject(job: PEJOB, access_mode: KPROCESSOR_MODE, exit_status: NTSTATUS) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn PspRemoveProcessFromJob(process: PEPROCESS) {
    let process = opaque_ptr(process);
}

#[no_mangle]
pub unsafe extern "system" fn PspExitProcessFromJob(process: PEPROCESS) {
    let process = opaque_ptr(process);
}

#[no_mangle]
pub unsafe extern "system" fn NtAssignProcessToJobObject(job_handle: HANDLE, process_handle: HANDLE) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let process_handle = opaque_ptr(process_handle);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let previous_mode: KPROCESSOR_MODE;
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();

    status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_TERMINATE,
        PsProcessType,
        previous_mode,
        core::ptr::addr_of_mut!(process) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let process_eproc = process as *mut _EPROCESS;
        if (*process_eproc).Job == core::ptr::null_mut() {
            let mut job: PEJOB = core::ptr::null_mut();

            status = ObReferenceObjectByHandle(
                job_handle,
                JOB_OBJECT_ASSIGN_PROCESS,
                PsJobType,
                previous_mode,
                core::ptr::addr_of_mut!(job) as *mut PVOID,
                core::ptr::null_mut(),
            );

            if status >= 0 {
                if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process_eproc).RundownProtect)) != 0 {
                    if (*process_eproc).Job == core::ptr::null_mut() && PsGetProcessSessionId(process) == (*job).SessionId {
                        (*process_eproc).Job = job;
                        ObReferenceObject(job as PVOID);
                    } else {
                        status = STATUS_ACCESS_DENIED;
                    }
                    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process_eproc).RundownProtect));

                    if status >= 0 {
                        status = PspAssignProcessToJob(process, job);
                    }
                }

                ObDereferenceObject(job as PVOID);
            }
        } else {
            status = STATUS_ACCESS_DENIED;
        }

        ObDereferenceObject(process as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateJobSet(
    num_job: ULONG,
    user_job_set: PJOB_SET_ARRAY,
    flags: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateJobObject(
    job_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let mut h_job: HANDLE = core::ptr::null_mut();
    let mut job: PEJOB = core::ptr::null_mut();
    let previous_mode: KPROCESSOR_MODE;
    let current_process: PEPROCESS;
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();
    current_process = PsGetCurrentProcess() as PEPROCESS;

    if previous_mode != 0 {
        if let Err(s) = probe_write_user::<PVOID>(job_handle as _, core::ptr::null_mut(), previous_mode) {
            return s;
        }
    }

    status = ObCreateObject(
        previous_mode, PsJobType,
        object_attributes,
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_EJOB>() as ULONG,
        0,
        0,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
    );

    if status >= 0 {
        core::ptr::write_bytes(job as *mut u8, 0, core::mem::size_of::<_EJOB>());

        (*job).JobLinks.Flink = core::ptr::null_mut();

        InitializeListHead(core::ptr::addr_of_mut!((*job).JobSetLinks) as PLIST_ENTRY);
        InitializeListHead(core::ptr::addr_of_mut!((*job).ProcessListHead) as PLIST_ENTRY);

        (*job).SessionId = PsGetProcessSessionId(current_process);

        KeInitializeGuardedMutex(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));

        status = ExInitializeResourceLite(core::ptr::addr_of_mut!((*job).JobLock) as PVOID);
        if status < 0 {
            ObDereferenceObject(job as PVOID);
            return status;
        }
        KeInitializeEvent(
            core::ptr::addr_of_mut!((*job).Event) as PVOID,
            0,
            0,
        );

        ExAcquireFastMutex(core::ptr::addr_of_mut!(PsJobListLock));
        InsertTailList(
            core::ptr::addr_of_mut!(PsJobListHead) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*job).JobLinks) as PLIST_ENTRY,
        );
        ExReleaseFastMutex(core::ptr::addr_of_mut!(PsJobListLock));

        status = ObInsertObject(
            job as PVOID,
            core::ptr::null_mut(),
            desired_access,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(h_job),
        );
        if status >= 0 {
            if let Err(s) = probe_write_user(job_handle as _, h_job, previous_mode) {
                return s;
            }
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtIsProcessInJob(
    process_handle: HANDLE,
    job_handle: HANDLE,
) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let job_handle = opaque_ptr(job_handle);
    let previous_mode: KPROCESSOR_MODE;
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();

    status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_QUERY_INFORMATION,
        PsProcessType,
        previous_mode,
        core::ptr::addr_of_mut!(process) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let process_eproc = process as *mut _EPROCESS;
        let process_job = (*process_eproc).Job;

        if process_job != core::ptr::null_mut() {
            if job_handle == core::ptr::null_mut() {
                status = STATUS_PROCESS_IN_JOB;
            } else {
                let mut job_object: PEJOB = core::ptr::null_mut();

                status = ObReferenceObjectByHandle(
                    job_handle,
                    JOB_OBJECT_QUERY,
                    PsJobType,
                    previous_mode,
                    core::ptr::addr_of_mut!(job_object) as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if status >= 0 {
                    status = if process_job == job_object {
                        STATUS_PROCESS_IN_JOB
                    } else {
                        STATUS_PROCESS_NOT_IN_JOB
                    };
                    ObDereferenceObject(job_object as PVOID);
                }
            }
        } else {
            status = STATUS_PROCESS_NOT_IN_JOB;
        }
        ObDereferenceObject(process as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenJobObject(
    job_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let previous_mode: KPROCESSOR_MODE;
    let mut h_job: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();

    if previous_mode != 0 {
        if let Err(s) = probe_write_user::<PVOID>(job_handle as _, core::ptr::null_mut(), previous_mode) {
            return s;
        }
    }

    status = ObOpenObjectByName(
        object_attributes, PsJobType,
        previous_mode,
        core::ptr::null_mut(),
        desired_access,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(h_job),
    );
    if status >= 0 {
        if let Err(s) = probe_write_user(job_handle as _, h_job, previous_mode) {
            return s;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationJobObject(
    job_handle: HANDLE,
    job_information_class: JOBOBJECTINFOCLASS,
    job_information: PVOID,
    job_information_length: ULONG,
    return_length: *mut ULONG,
) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let job_information = opaque_ptr(job_information);
    let return_length = opaque_ptr(return_length);
    let mut job: PEJOB = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut no_output: BOOLEAN;
    let mut generic_copy: PVOID = core::ptr::null_mut();
    let mut next_entry: PLIST_ENTRY;
    let current_thread: PKTHREAD;
    let previous_mode: KPROCESSOR_MODE;
    let mut extended_limit: _JOBOBJECT_EXTENDED_LIMIT_INFORMATION = core::mem::zeroed();
    let mut basic_and_io: JOBOBJECT_BASIC_AND_IO_ACCOUNTING_INFORMATION = core::mem::zeroed();
    let required_length: ULONG;
    let required_align: ULONG;
    let mut size_to_copy: ULONG;
    let needed_size: ULONG;

    current_thread = KeGetCurrentThread();

    if job_information_class > 10 || job_information_class < 1 {
        return STATUS_INVALID_INFO_CLASS;
    }

    required_length = PspJobInfoLengths[job_information_class as usize];
    required_align = PspJobInfoAlign[job_information_class as usize];
    size_to_copy = required_length;
    needed_size = required_length;

    if job_information_length != required_length {
        if (job_information_class != 3 && job_information_class != 5) || job_information_length < required_length {
            return STATUS_INFO_LENGTH_MISMATCH;
        }

        size_to_copy = job_information_length;
    }

    previous_mode = ExGetPreviousMode();
    if previous_mode != 0 {
        if job_information != core::ptr::null_mut() {
            if let Err(s) = probe_user_buffer(job_information as _, job_information_length as usize, required_align as usize, previous_mode) {
                return s;
            }
        }

        if return_length != core::ptr::null_mut() {
            if let Err(s) = probe_write_user::<ULONG>(return_length, 0, previous_mode) {
                return s;
            }
        }
    }

    if job_handle != core::ptr::null_mut() {
        status = ObReferenceObjectByHandle(
            job_handle, JOB_OBJECT_QUERY,
            PsJobType,
            previous_mode,
            core::ptr::addr_of_mut!(job) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            return status;
        }
    } else {
        let current_process: PEPROCESS;

        current_process = (*(current_thread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.ApcState.Process as PEPROCESS;
        let current_eproc = current_process as *mut _EPROCESS;
        job = (*current_eproc).Job;
        if job == core::ptr::null_mut() {
            return STATUS_ACCESS_DENIED;
        }

        ObReferenceObject(job as PVOID);
    }

    no_output = 0;
    match job_information_class {
        1 | 8 => {
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(basic_and_io.BasicInfo) as *mut u8,
                0,
                core::mem::size_of::<_JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>(),
            );

            KeEnterGuardedRegionThread(current_thread);
            ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*job).JobLock), 1);

            basic_and_io.BasicInfo.TotalUserTime.QuadPart = (*job).TotalUserTime.QuadPart;
            basic_and_io.BasicInfo.TotalKernelTime.QuadPart = (*job).TotalKernelTime.QuadPart;
            basic_and_io.BasicInfo.ThisPeriodTotalUserTime.QuadPart = (*job).ThisPeriodTotalUserTime.QuadPart;
            basic_and_io.BasicInfo.ThisPeriodTotalKernelTime.QuadPart = (*job).ThisPeriodTotalKernelTime.QuadPart;
            basic_and_io.BasicInfo.TotalPageFaultCount = (*job).TotalPageFaultCount;
            basic_and_io.BasicInfo.TotalProcesses = (*job).TotalProcesses;
            basic_and_io.BasicInfo.ActiveProcesses = (*job).ActiveProcesses;
            basic_and_io.BasicInfo.TotalTerminatedProcesses = (*job).TotalTerminatedProcesses;

            basic_and_io.IoInfo.ReadOperationCount = (*job).ReadOperationCount;
            basic_and_io.IoInfo.WriteOperationCount = (*job).WriteOperationCount;
            basic_and_io.IoInfo.OtherOperationCount = (*job).OtherOperationCount;
            basic_and_io.IoInfo.ReadTransferCount = (*job).ReadTransferCount;
            basic_and_io.IoInfo.WriteTransferCount = (*job).WriteTransferCount;
            basic_and_io.IoInfo.OtherTransferCount = (*job).OtherTransferCount;

            next_entry = (*job).ProcessListHead.Flink;
            while next_entry != core::ptr::addr_of_mut!((*job).ProcessListHead) {
                let process = ((next_entry as usize) - core::mem::offset_of!(_EPROCESS, JobLinks)) as PEPROCESS;
                let process_eproc = process as *mut _EPROCESS;
                if ((*process_eproc).JobStatus & 2) == 0 {
                    let mut values: _PROCESS_VALUES = core::mem::zeroed();

                    KeQueryValuesProcess(core::ptr::addr_of_mut!((*process_eproc).Pcb), core::ptr::addr_of_mut!(values));
                    basic_and_io.BasicInfo.TotalUserTime.QuadPart += values.TotalUserTime.QuadPart;
                    basic_and_io.BasicInfo.TotalKernelTime.QuadPart += values.TotalKernelTime.QuadPart;
                    basic_and_io.IoInfo.ReadOperationCount += values.IoInfo.ReadOperationCount;
                    basic_and_io.IoInfo.WriteOperationCount += values.IoInfo.WriteOperationCount;
                    basic_and_io.IoInfo.OtherOperationCount += values.IoInfo.OtherOperationCount;
                    basic_and_io.IoInfo.ReadTransferCount += values.IoInfo.ReadTransferCount;
                    basic_and_io.IoInfo.WriteTransferCount += values.IoInfo.WriteTransferCount;
                    basic_and_io.IoInfo.OtherTransferCount += values.IoInfo.OtherTransferCount;
                }
                next_entry = (*next_entry).Flink;
            }

            ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
            KeLeaveGuardedRegionThread(current_thread);

            generic_copy = core::ptr::addr_of_mut!(basic_and_io) as PVOID;
            status = STATUS_SUCCESS;
        }
        2 | 9 => {
            KeEnterGuardedRegionThread(current_thread);
            ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*job).JobLock), 1);

            extended_limit.BasicLimitInformation.LimitFlags = (*job).LimitFlags;
            extended_limit.BasicLimitInformation.MinimumWorkingSetSize = (*job).MinimumWorkingSetSize as SIZE_T;
            extended_limit.BasicLimitInformation.MaximumWorkingSetSize = (*job).MaximumWorkingSetSize as SIZE_T;
            extended_limit.BasicLimitInformation.ActiveProcessLimit = (*job).ActiveProcessLimit;
            extended_limit.BasicLimitInformation.PriorityClass = (*job).PriorityClass as ULONG;
            extended_limit.BasicLimitInformation.SchedulingClass = (*job).SchedulingClass;
            extended_limit.BasicLimitInformation.Affinity = (*job).Affinity as ULONG_PTR;
            extended_limit.BasicLimitInformation.PerProcessUserTimeLimit.QuadPart = (*job).PerProcessUserTimeLimit.QuadPart;
            extended_limit.BasicLimitInformation.PerJobUserTimeLimit.QuadPart = (*job).PerJobUserTimeLimit.QuadPart;

            if job_information_class == 9 {
                KeAcquireGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));
                extended_limit.ProcessMemoryLimit = (((*job).ProcessMemoryLimit as u64) << PAGE_SHIFT) as SIZE_T;
                extended_limit.JobMemoryLimit = (((*job).JobMemoryLimit as u64) << PAGE_SHIFT) as SIZE_T;
                extended_limit.PeakProcessMemoryUsed = (((*job).PeakProcessMemoryUsed as u64) << PAGE_SHIFT) as SIZE_T;
                extended_limit.PeakJobMemoryUsed = (((*job).PeakJobMemoryUsed as u64) << PAGE_SHIFT) as SIZE_T;
                KeReleaseGuardedMutexUnsafe(core::ptr::addr_of_mut!((*job).MemoryLimitsLock));

                ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
                KeLeaveGuardedRegionThread(current_thread);

                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!(extended_limit.IoInfo) as *mut u8,
                    0,
                    core::mem::size_of::<_IO_COUNTERS>(),
                );
            } else {
                ExReleaseResourceLite(core::ptr::addr_of_mut!((*job).JobLock));
                KeLeaveGuardedRegionThread(current_thread);
            }

            generic_copy = core::ptr::addr_of_mut!(extended_limit) as PVOID;
            status = STATUS_SUCCESS;
        }
        _ => {
            status = STATUS_NOT_IMPLEMENTED;
        }
    }

    ObDereferenceObject(job as PVOID);

    if status >= 0 {
        if no_output == 0 {
            if let Err(s) = safe_copy_to_user(job_information as _, generic_copy as _, size_to_copy as usize, align_of::<ULONG>(), previous_mode) {
                return s;
            }
        }

        if return_length != core::ptr::null_mut() {
            if let Err(s) = probe_write_user(return_length as _, needed_size, previous_mode) {
                return s;
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
    let current_thread: PKTHREAD;
    let mut desired_access: ACCESS_MASK;
    let previous_mode: KPROCESSOR_MODE;
    let required_length: ULONG;
    let required_align: ULONG;

    current_thread = KeGetCurrentThread();

    if job_information_class > 10 || job_information_class < 1 {
        return STATUS_INVALID_INFO_CLASS;
    }

    required_length = PspJobInfoLengths[job_information_class as usize];
    required_align = PspJobInfoAlign[job_information_class as usize];

    previous_mode = ExGetPreviousMode();
    if previous_mode != 0 {
        if job_information_length != 0 {
            if let Err(s) = probe_user_buffer(job_information as _, job_information_length as usize, required_align as usize, previous_mode) {
                return s;
            }
        }
    }

    if job_information_length != required_length {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    desired_access = JOB_OBJECT_SET_ATTRIBUTES;
    if job_information_class == 5 {
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
    if status < 0 {
        return status;
    }

    KeEnterGuardedRegionThread(current_thread);
    match job_information_class {
        9 => {
            status = STATUS_SUCCESS;
        }
        _ => {
            status = STATUS_NOT_IMPLEMENTED;
        }
    }
    KeLeaveGuardedRegionThread(current_thread);

    ObDereferenceObject(job as PVOID);

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtTerminateJobObject(job_handle: HANDLE, exit_status: NTSTATUS) -> NTSTATUS {
    let job_handle = opaque_ptr(job_handle);
    let previous_mode: KPROCESSOR_MODE;
    let mut job: PEJOB = core::ptr::null_mut();
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();

    status = ObReferenceObjectByHandle(
        job_handle,
        JOB_OBJECT_TERMINATE,
        PsJobType,
        previous_mode,
        core::ptr::addr_of_mut!(job) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status >= 0 {
        status = PspTerminateJobObject(job, previous_mode, exit_status);
        ObDereferenceObject(job as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobLock(job: PEJOB) -> PVOID {
    core::ptr::addr_of_mut!((*job).JobLock) as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobSessionId(job: PEJOB) -> ULONG {
    (*job).SessionId
}

#[no_mangle]
pub unsafe extern "system" fn PsGetJobUIRestrictionsClass(job: PEJOB) -> ULONG {
    (*job).UIRestrictionsClass
}

#[no_mangle]
pub unsafe extern "system" fn PsSetJobUIRestrictionsClass(job: PEJOB, ui_restrictions_class: ULONG) {
    InterlockedExchangeUL(
        core::ptr::addr_of_mut!((*job).UIRestrictionsClass) as *mut ULONG,
        ui_restrictions_class,
    );
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ps/job.c
//  * PURPOSE:         Job Native Functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net) (stubs)
//  *                  Thomas Weidenmueller <w3seek@reactos.com>
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS *******************************************************************/
//
// POBJECT_TYPE PsJobType = NULL;
//
// LIST_ENTRY PsJobListHead;
// static FAST_MUTEX PsJobListLock;
//
// BOOLEAN PspUseJobSchedulingClasses;
//
// CHAR PspJobSchedulingClasses[PSP_JOB_SCHEDULING_CLASSES] =
// {
//     1 * 6,
//     2 * 6,
//     3 * 6,
//     4 * 6,
//     5 * 6,
//     6 * 6,
//     7 * 6,
//     8 * 6,
//     9 * 6,
//     10 * 6
// };
//
// GENERIC_MAPPING PspJobMapping =
// {
//     STANDARD_RIGHTS_READ | JOB_OBJECT_QUERY,
//
//     STANDARD_RIGHTS_WRITE | JOB_OBJECT_ASSIGN_PROCESS |
//     JOB_OBJECT_SET_ATTRIBUTES | JOB_OBJECT_TERMINATE,
//
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//
//     STANDARD_RIGHTS_ALL | THREAD_ALL_ACCESS // bug fixed only in vista
// };
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
