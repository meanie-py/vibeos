use ntkrnl_sys::ddk::*;
// ke globals across the crate boundary as link symbols (#[no_mangle] in
// ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeMaximumIncrement: ntkrnl_sys::base::ULONG;
    static mut KeActiveProcessors: ntkrnl_sys::ddk::KAFFINITY;
}
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED, TRUE, FALSE, PAGE_SHIFT, APC_LEVEL};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::macros::NtCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// PROCESSINFOCLASS / THREADINFOCLASS type aliases
type PROCESSINFOCLASS = _PROCESSINFOCLASS;
type THREADINFOCLASS = _THREADINFOCLASS;

// Process information class constants
const ProcessBasicInformation: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessBasicInformation;
const ProcessQuotaLimits: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessQuotaLimits;
const ProcessIoCounters: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessIoCounters;
const ProcessVmCounters: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessVmCounters;
const ProcessTimes: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessTimes;
const ProcessBasePriority: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessBasePriority;
const ProcessRaisePriority: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessRaisePriority;
const ProcessDebugPort: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessDebugPort;
const ProcessExceptionPort: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessExceptionPort;
const ProcessAccessToken: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessAccessToken;
const ProcessLdtInformation: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessLdtInformation;
const ProcessLdtSize: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessLdtSize;
const ProcessDefaultHardErrorMode: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessDefaultHardErrorMode;
const ProcessIoPortHandlers: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessIoPortHandlers;
const ProcessPooledUsageAndLimits: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessPooledUsageAndLimits;
const ProcessWorkingSetWatch: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessWorkingSetWatch;
const ProcessUserModeIOPL: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessUserModeIOPL;
const ProcessEnableAlignmentFaultFixup: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessEnableAlignmentFaultFixup;
const ProcessPriorityClass: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessPriorityClass;
const ProcessWx86Information: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessWx86Information;
const ProcessHandleCount: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessHandleCount;
const ProcessAffinityMask: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessAffinityMask;
const ProcessPriorityBoost: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessPriorityBoost;
const ProcessDeviceMap: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessDeviceMap;
const ProcessSessionInformation: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessSessionInformation;
const ProcessForegroundInformation: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessForegroundInformation;
const ProcessWow64Information: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessWow64Information;
const ProcessImageFileName: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessImageFileName;
const ProcessLUIDDeviceMapsEnabled: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessLUIDDeviceMapsEnabled;
const ProcessBreakOnTermination: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessBreakOnTermination;
const ProcessDebugObjectHandle: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessDebugObjectHandle;
const ProcessDebugFlags: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessDebugFlags;
const ProcessHandleTracing: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessHandleTracing;
const ProcessExecuteFlags: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessExecuteFlags;
const ProcessCookie: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessCookie;
const ProcessImageInformation: PROCESSINFOCLASS = _PROCESSINFOCLASS_ProcessImageInformation;
const ProcessImageFileNameWin32: PROCESSINFOCLASS = 43;

// Thread information class constants
const ThreadBasicInformation: THREADINFOCLASS = _THREADINFOCLASS_ThreadBasicInformation;
const ThreadTimes: THREADINFOCLASS = _THREADINFOCLASS_ThreadTimes;
const ThreadPriority: THREADINFOCLASS = _THREADINFOCLASS_ThreadPriority;
const ThreadBasePriority: THREADINFOCLASS = _THREADINFOCLASS_ThreadBasePriority;
const ThreadAffinityMask: THREADINFOCLASS = _THREADINFOCLASS_ThreadAffinityMask;
const ThreadImpersonationToken: THREADINFOCLASS = _THREADINFOCLASS_ThreadImpersonationToken;
const ThreadDescriptorTableEntry: THREADINFOCLASS = _THREADINFOCLASS_ThreadDescriptorTableEntry;
const ThreadQuerySetWin32StartAddress: THREADINFOCLASS = _THREADINFOCLASS_ThreadQuerySetWin32StartAddress;
const ThreadZeroTlsCell: THREADINFOCLASS = _THREADINFOCLASS_ThreadZeroTlsCell;
const ThreadPerformanceCount: THREADINFOCLASS = _THREADINFOCLASS_ThreadPerformanceCount;
const ThreadAmILastThread: THREADINFOCLASS = _THREADINFOCLASS_ThreadAmILastThread;
const ThreadIdealProcessor: THREADINFOCLASS = _THREADINFOCLASS_ThreadIdealProcessor;
const ThreadPriorityBoost: THREADINFOCLASS = _THREADINFOCLASS_ThreadPriorityBoost;
const ThreadIsIoPending: THREADINFOCLASS = _THREADINFOCLASS_ThreadIsIoPending;
const ThreadHideFromDebugger: THREADINFOCLASS = _THREADINFOCLASS_ThreadHideFromDebugger;
const ThreadBreakOnTermination: THREADINFOCLASS = _THREADINFOCLASS_ThreadBreakOnTermination;
const ThreadIsTerminated: THREADINFOCLASS = _THREADINFOCLASS_ThreadIsTerminated;

// Quota pool index constants
const PsPagedPool: usize = 0x01;
const PsNonPagedPool: usize = 0x00;
const PsPageFile: usize = 2;

// TLS constants
const TLS_MINIMUM_AVAILABLE: usize = 64;
const TLS_EXPANSION_SLOTS: usize = 1024;

// Compatibility version constant
const _WIN32_WINNT_WS03: ULONG = 0x0502;

// Process priority mode constants
const PsProcessPriorityForeground: PSPROCESSPRIORITYMODE = _PSPROCESSPRIORITYMODE_PsProcessPriorityForeground;
const PsProcessPriorityBackground: PSPROCESSPRIORITYMODE = _PSPROCESSPRIORITYMODE_PsProcessPriorityBackground;

// Kernel global variables (not in bindgen)
extern "C" {
    static PsProcessType: POBJECT_TYPE;
    static PsThreadType: POBJECT_TYPE;
    static LpcPortObjectType: POBJECT_TYPE;
    static SeDebugPrivilege: LUID;
    static SeTcbPrivilege: LUID;
    static SeIncreaseBasePriorityPrivilege: LUID;
    static SeAssignPrimaryTokenPrivilege: LUID;
    static mut PspDefaultQuotaBlock: _EPROCESS_QUOTA_BLOCK;
    static PsProcessInfoClass: INFORMATION_CLASS_INFO;
    static PsThreadInfoClass: INFORMATION_CLASS_INFO;
}

// Info class table sizes — must match the actual per-arch entry count in
// ntoskrnl/include/internal/ps_i.h (`PsProcessInfoClass[]` /
// `PsThreadInfoClass[]`). The arrays have 47 / 28 active entries for
// NT 5.x (last valid indices: ProcessMemoryAllocationMode=46,
// ThreadCSwitchMon=27).
//
// DO NOT use the bindgen `Max*InfoClass` enum values: those follow the
// Vista+ NTDDI definition (51 / 56) and used to be (incorrectly)
// transcribed here as 48 / 57. Passing those larger counts to
// `DefaultQueryInfoBufferCheck` skipped the bounds check for classes
// past the actual array end, reading garbage `AlignmentQUERY` bytes
// that tripped `ProbeForRead`'s misalign check and bugchecked (with
// DUMMY_PSEH the resulting STATUS_DATATYPE_MISALIGNMENT is unhandled).
// The C-side `kernel_c_helpers.c` re-include trick happens to leave
// "safe" trailing bytes after the array; the parallel Rust translation
// in `ps/info_class.rs` does not, which is why renaming the Rust
// supplier to the canonical symbol surfaced the latent bug.
const PS_PROCESS_INFO_CLASS_COUNT: ULONG = 47;
const PS_THREAD_INFO_CLASS_COUNT: ULONG = 28;

// Functions not in bindgen (FORCEINLINE/macros in C)
extern "system" {
    fn PspSetPrimaryToken(process: PEPROCESS, token_handle: HANDLE, previous_mode: *const KPROCESSOR_MODE) -> NTSTATUS;
    fn PspSetQuotaLimits(process: PEPROCESS, unused: ULONG, buffer: PVOID, length: ULONG, mode: KPROCESSOR_MODE) -> NTSTATUS;
    fn PspQueryDescriptorThread(thread: PETHREAD, info: PVOID, length: ULONG, return_length: PULONG) -> NTSTATUS;
}
extern "C" {
    fn DefaultQueryInfoBufferCheck(
        info_class: ULONG,
        info_class_table: *const INFORMATION_CLASS_INFO,
        table_entries: ULONG,
        flags: ULONG,
        buffer: PVOID,
        buffer_length: ULONG,
        return_length: PULONG,
        previous_mode_ptr: *const KPROCESSOR_MODE,
        previous_mode: KPROCESSOR_MODE,
    ) -> NTSTATUS;

    fn DefaultSetInfoBufferCheck(
        info_class: ULONG,
        info_class_table: *const INFORMATION_CLASS_INFO,
        table_entries: ULONG,
        buffer: PVOID,
        buffer_length: ULONG,
        previous_mode: KPROCESSOR_MODE,
    ) -> NTSTATUS;

    fn PspSetProcessFlag(process: *mut _EPROCESS, flag: ULONG);
    fn PspClearProcessFlag(process: *mut _EPROCESS, flag: ULONG);
    fn PspSetCrossThreadFlag(thread: *mut _ETHREAD, flag: ULONG);
    fn PspClearCrossThreadFlag(Thread: PETHREAD, Flag: ULONG) -> LONG;
}
unsafe extern "system" {
    fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL);
    fn KeLowerIrql(new_irql: KIRQL);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
extern "system" {
    fn KeQuerySystemTime(current_time: *mut LARGE_INTEGER);
}

// ============================================================================
// Helper accessors for _KTHREAD fields with different anonymous union numbering
// on i386 vs amd64/arm64.
// ============================================================================

/// Access Tcb.KernelTime (in SuspendApc overlay union)
#[inline]
unsafe fn kthread_kernel_time(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_9.__bindgen_anon_3.KernelTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_3.KernelTime }
}

/// Access Tcb.UserTime
#[inline]
unsafe fn kthread_user_time(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.UserTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_6.UserTime }
}

/// Access Tcb.DisableBoost bitfield
#[inline]
unsafe fn kthread_disable_boost(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_6.__bindgen_anon_1.DisableBoost() }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.DisableBoost() }
}

/// IsListEmpty - check if doubly-linked list is empty
#[inline]
unsafe fn IsListEmpty(head: *const LIST_ENTRY) -> BOOLEAN {
    if (*head).Flink == head as *mut LIST_ENTRY { TRUE } else { FALSE }
}

/// CONTAINING_RECORD: compute pointer to containing struct from a field pointer.
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $field:ident) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $field)) as *mut $type
    };
}

static mut PspTraceLevel: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn PsReferenceProcessFilePointer(
    Process: PEPROCESS,
    FileObject: *mut PFILE_OBJECT,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let FileObject = opaque_ptr(FileObject);
    let process = Process as *mut _EPROCESS;
    
    if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) == 0 {
        return STATUS_PROCESS_IS_TERMINATING;
    }

    let section = (*process).SectionObject as PSECTION;
    if !section.is_null() {
        *FileObject = MmGetFileObjectForSection(section as PVOID);
        ObReferenceObject(*FileObject as PVOID);
    }

    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));

    if !section.is_null() { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL }
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationProcess(
    ProcessHandle: HANDLE,
    ProcessInformationClass: PROCESSINFOCLASS,
    ProcessInformation: PVOID,
    ProcessInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let ProcessInformation = opaque_ptr(ProcessInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut Length: ULONG = 0;

    Status = DefaultQueryInfoBufferCheck(
        ProcessInformationClass,
        core::ptr::addr_of!(PsProcessInfoClass),
        PS_PROCESS_INFO_CLASS_COUNT,
        ICIF_PROBE_READ,
        ProcessInformation,
        ProcessInformationLength,
        ReturnLength,
        core::ptr::null_mut(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    if ((ProcessInformationClass == ProcessCookie) ||
        (ProcessInformationClass == ProcessImageInformation)) &&
       (ProcessHandle != NtCurrentProcess())
    {
        return STATUS_INVALID_PARAMETER;
    }

    match ProcessInformationClass {
        ProcessBasicInformation => {
            let ProcessBasicInfo = ProcessInformation as PPROCESS_BASIC_INFORMATION;

            if ProcessInformationLength != core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    (*ProcessBasicInfo).ExitStatus = (*process).ExitStatus;
                    (*ProcessBasicInfo).PebBaseAddress = (*process).Peb;
                    (*ProcessBasicInfo).AffinityMask = (*process).Pcb.Affinity;
                    (*ProcessBasicInfo).UniqueProcessId = (*process).UniqueProcessId as ULONG_PTR;
                    (*ProcessBasicInfo).InheritedFromUniqueProcessId = (*process).InheritedFromUniqueProcessId as ULONG_PTR;
                    (*ProcessBasicInfo).BasePriority = (*process).Pcb.BasePriority as LONG;

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessQuotaLimits => {
            let mut QuotaLimits: QUOTA_LIMITS_EX = core::mem::zeroed();
            let Extended: BOOLEAN;

            if ProcessInformationLength != core::mem::size_of::<QUOTA_LIMITS>() as ULONG &&
               ProcessInformationLength != core::mem::size_of::<QUOTA_LIMITS_EX>() as ULONG
            {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = ProcessInformationLength;
                Extended = if Length == core::mem::size_of::<QUOTA_LIMITS_EX>() as ULONG { 1u8 } else { 0u8 };

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    Status = STATUS_SUCCESS;

                    RtlZeroMemory(&mut QuotaLimits as *mut _ as PVOID, core::mem::size_of::<QUOTA_LIMITS_EX>());

                    QuotaLimits.MaximumWorkingSetSize = ((*process).Vm.MaximumWorkingSetSize as SIZE_T) << PAGE_SHIFT;
                    QuotaLimits.MinimumWorkingSetSize = ((*process).Vm.MinimumWorkingSetSize as SIZE_T) << PAGE_SHIFT;

                    QuotaLimits.TimeLimit.QuadPart = -1i64;

                    if (*process).QuotaBlock == &raw const PspDefaultQuotaBlock as *const _ as *mut _ {
                        QuotaLimits.PagedPoolLimit = SIZE_T::MAX;
                        QuotaLimits.NonPagedPoolLimit = SIZE_T::MAX;
                        QuotaLimits.PagefileLimit = SIZE_T::MAX;
                    } else {
                        QuotaLimits.PagedPoolLimit = (*(*process).QuotaBlock).QuotaEntry[PsPagedPool as usize].Limit;
                        QuotaLimits.NonPagedPoolLimit = (*(*process).QuotaBlock).QuotaEntry[PsNonPagedPool as usize].Limit;
                        QuotaLimits.PagefileLimit = (*(*process).QuotaBlock).QuotaEntry[PsPageFile as usize].Limit;
                    }

                    if Extended != 0 {
                        QuotaLimits.Flags |= if (*process).Vm.Flags.MaximumWorkingSetHard() != 0 {
                            QUOTA_LIMITS_HARDWS_MAX_ENABLE
                        } else {
                            QUOTA_LIMITS_HARDWS_MAX_DISABLE
                        };
                        QuotaLimits.Flags |= if (*process).Vm.Flags.MinimumWorkingSetHard() != 0 {
                            QUOTA_LIMITS_HARDWS_MIN_ENABLE
                        } else {
                            QUOTA_LIMITS_HARDWS_MIN_DISABLE
                        };

                        QuotaLimits.CpuRateLimit.RateData = 0;
                    }

                    if let Err(s) = safe_copy_to_user(ProcessInformation, &QuotaLimits as *const _ as PVOID, Length as usize, core::mem::align_of::<ULONG>(), PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessIoCounters => {
            let IoCounters = ProcessInformation as PIO_COUNTERS;
            let mut ProcessValues: PROCESS_VALUES = core::mem::zeroed();

            if ProcessInformationLength != core::mem::size_of::<IO_COUNTERS>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<IO_COUNTERS>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    KeQueryValuesProcess(
                        core::ptr::addr_of_mut!((*process).Pcb),
                        &mut ProcessValues,
                    );

                    if let Err(s) = safe_copy_to_user(
                        IoCounters as PVOID,
                        &ProcessValues.IoInfo as *const _ as PVOID,
                        core::mem::size_of::<IO_COUNTERS>(),
                        core::mem::align_of::<ULONG>(),
                        PreviousMode,
                    ) {
                        return s;
                    }

                    Status = STATUS_SUCCESS;

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessTimes => {
            let ProcessTime = ProcessInformation as PKERNEL_USER_TIMES;
            let mut UserTime: ULONG = 0;
            let mut KernelTime: ULONG;

            if ProcessInformationLength != core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    KernelTime = KeQueryRuntimeProcess(core::ptr::addr_of_mut!((*process).Pcb), &mut UserTime);
                    (*ProcessTime).CreateTime = (*process).CreateTime;
                    (*ProcessTime).UserTime.QuadPart = (UserTime as LONGLONG) * (KeMaximumIncrement as LONGLONG);
                    (*ProcessTime).KernelTime.QuadPart = (KernelTime as LONGLONG) * (KeMaximumIncrement as LONGLONG);
                    (*ProcessTime).ExitTime = (*process).ExitTime;

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessDebugPort => {
            if ProcessInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<HANDLE>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    let debug_handle = if !(*process).DebugPort.is_null() {
                        -1isize as HANDLE
                    } else {
                        core::ptr::null_mut()
                    };
                    if let Err(s) = probe_write_user::<PVOID>(ProcessInformation as *mut PVOID, debug_handle as PVOID, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessHandleCount => {
            let mut HandleCount: ULONG;

            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    HandleCount = ObGetProcessHandleCount(Process);

                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, HandleCount, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessSessionInformation => {
            let SessionInfo = ProcessInformation as PPROCESS_SESSION_INFORMATION;

            if ProcessInformationLength != core::mem::size_of::<PROCESS_SESSION_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<PROCESS_SESSION_INFORMATION>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    (*SessionInfo).SessionId = PsGetProcessSessionId(Process);

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessVmCounters => {
            let VmCounters = ProcessInformation as PVM_COUNTERS;

            if (ProcessInformationLength != core::mem::size_of::<VM_COUNTERS>() as ULONG) &&
               (ProcessInformationLength != core::mem::size_of::<VM_COUNTERS_EX>() as ULONG)
            {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    (*VmCounters).PeakVirtualSize = (*process).PeakVirtualSize;
                    (*VmCounters).VirtualSize = (*process).VirtualSize;
                    (*VmCounters).PageFaultCount = (*process).Vm.PageFaultCount;
                    (*VmCounters).PeakWorkingSetSize = (*process).Vm.PeakWorkingSetSize as SIZE_T;
                    (*VmCounters).WorkingSetSize = (*process).Vm.WorkingSetSize as SIZE_T;
                    (*VmCounters).QuotaPeakPagedPoolUsage = (*process).QuotaPeak[PsPagedPool as usize];
                    (*VmCounters).QuotaPagedPoolUsage = (*process).QuotaUsage[PsPagedPool as usize];
                    (*VmCounters).QuotaPeakNonPagedPoolUsage = (*process).QuotaPeak[PsNonPagedPool as usize];
                    (*VmCounters).QuotaNonPagedPoolUsage = (*process).QuotaUsage[PsNonPagedPool as usize];
                    (*VmCounters).PagefileUsage = (*process).QuotaUsage[PsPageFile as usize] << PAGE_SHIFT;
                    (*VmCounters).PeakPagefileUsage = (*process).QuotaPeak[PsPageFile as usize] << PAGE_SHIFT;

                    Length = ProcessInformationLength;

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessDefaultHardErrorMode => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, (*process).DefaultHardErrorProcessing, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessPriorityBoost => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    let boost_val = if (*process).Pcb.__bindgen_anon_1.__bindgen_anon_1.DisableBoost() != 0 { 1u32 } else { 0u32 };
                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, boost_val, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessDeviceMap => {
            let mut Flags: ULONG = 0;

            if ProcessInformationLength == core::mem::size_of::<PROCESS_DEVICEMAP_INFORMATION_EX>() as ULONG {
                let DeviceMapEx = ProcessInformation as PPROCESS_DEVICEMAP_INFORMATION_EX;
                Flags = (*DeviceMapEx).Flags;

                if ((Flags & !PROCESS_LUID_DOSDEVICES_ONLY) != 0) ||
                   (ObIsLUIDDeviceMapsEnabled() == 0)
                {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    Length = ProcessInformationLength;

                    Status = ObReferenceObjectByHandle(
                        ProcessHandle,
                        PROCESS_QUERY_INFORMATION,
                        PsProcessType,
                        PreviousMode,
                        &mut Process as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        Status = ObQueryDeviceMapInformation(Process, ProcessInformation as PPROCESS_DEVICEMAP_INFORMATION, Flags);
                        ObDereferenceObject(Process as PVOID);
                    }
                }
            } else {
                if ProcessInformationLength != core::mem::size_of::<_PROCESS_DEVICEMAP_INFORMATION__bindgen_ty_1__bindgen_ty_2>() as ULONG {
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    Flags = 0;
                    Length = ProcessInformationLength;

                    Status = ObReferenceObjectByHandle(
                        ProcessHandle,
                        PROCESS_QUERY_INFORMATION,
                        PsProcessType,
                        PreviousMode,
                        &mut Process as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        Status = ObQueryDeviceMapInformation(Process, ProcessInformation as PPROCESS_DEVICEMAP_INFORMATION, Flags);
                        ObDereferenceObject(Process as PVOID);
                    }
                }
            }
        }

        ProcessPriorityClass => {
            let PsPriorityClass = ProcessInformation as PPROCESS_PRIORITY_CLASS;

            if ProcessInformationLength != core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    (*PsPriorityClass).PriorityClass = (*process).PriorityClass;
                    (*PsPriorityClass).Foreground = 0u8;

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessImageFileName => {
            let mut ImageName: PUNICODE_STRING = core::ptr::null_mut();

            Status = ObReferenceObjectByHandle(
                ProcessHandle,
                PROCESS_QUERY_INFORMATION,
                PsProcessType,
                PreviousMode,
                &mut Process as *mut _ as *mut PVOID,
                core::ptr::null_mut(),
            );
            if Status >= 0 {
                Status = SeLocateProcessImageName(Process, &mut ImageName);
                if Status >= 0 {
                    Length = (*ImageName).MaximumLength as ULONG + core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG;

                    if Length <= ProcessInformationLength {
                        if let Err(s) = safe_copy_to_user(ProcessInformation, ImageName as PVOID, Length as usize, core::mem::align_of::<ULONG>(), PreviousMode) {
                            return s;
                        }

                        (*(ProcessInformation as PUNICODE_STRING)).Buffer =
                            (ProcessInformation as PUNICODE_STRING).add(1) as *mut u16;
                    } else {
                        Status = STATUS_INFO_LENGTH_MISMATCH;
                    }

                    ExFreePoolWithTag(ImageName as PVOID, TAG_SEPA);
                }
                ObDereferenceObject(Process as PVOID);
            }
        }

        ProcessImageFileNameWin32 => {
            let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
            let mut ObjectNameInfo: POBJECT_NAME_INFORMATION = core::ptr::null_mut();

            Status = ObReferenceObjectByHandle(
                ProcessHandle,
                PROCESS_QUERY_INFORMATION,
                PsProcessType,
                PreviousMode,
                &mut Process as *mut _ as *mut PVOID,
                core::ptr::null_mut(),
            );
            if Status >= 0 {
                Status = PsReferenceProcessFilePointer(Process, &mut FileObject);
                ObDereferenceObject(Process as PVOID);
                if Status >= 0 {
                    Status = IoQueryFileDosDeviceName(FileObject, &mut ObjectNameInfo);
                    ObDereferenceObject(FileObject as PVOID);
                    if Status >= 0 {
                        Length = core::mem::size_of::<_UNICODE_STRING>() as ULONG + (*ObjectNameInfo).Name.MaximumLength as ULONG;
                        if Length <= ProcessInformationLength {
                            let ImageName = ProcessInformation as PUNICODE_STRING;
                            (*ImageName).Length = (*ObjectNameInfo).Name.Length;
                            (*ImageName).MaximumLength = (*ObjectNameInfo).Name.MaximumLength;
                            if (*ObjectNameInfo).Name.MaximumLength != 0 {
                                (*ImageName).Buffer = (ImageName as *mut _UNICODE_STRING).add(1) as *mut u16;
                                if let Err(s) = safe_copy_to_user(
                                    (*ImageName).Buffer as PVOID,
                                    (*ObjectNameInfo).Name.Buffer as PVOID,
                                    (*ObjectNameInfo).Name.MaximumLength as usize,
                                    core::mem::align_of::<ULONG>(),
                                    PreviousMode,
                                ) {
                                    return s;
                                }
                            } else {
                                (*ImageName).Buffer = core::ptr::null_mut();
                            }
                        } else {
                            Status = STATUS_INFO_LENGTH_MISMATCH;
                        }
                        ExFreePool(ObjectNameInfo as PVOID);
                    }
                }
            }
        }

        ProcessDebugFlags => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    let debug_val = if (*process).__bindgen_anon_2.__bindgen_anon_1.NoDebugInherit() != 0 { 0u32 } else { 1u32 };
                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, debug_val, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessBreakOnTermination => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, (*process).__bindgen_anon_2.__bindgen_anon_1.BreakOnTermination(), PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessCookie => {
            let mut Cookie: ULONG;

            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Process = PsGetCurrentProcess() as PEPROCESS;
                let process = Process as *mut _EPROCESS;
                Cookie = (*process).Cookie;
                if Cookie == 0 {
                    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
                    let NewCookie: ULONG;
                    let Prcb: *mut _KPRCB;

                    KeQuerySystemTime(&mut SystemTime);
                    Prcb = KeGetCurrentPrcb();
                    NewCookie = (*Prcb).KeSystemCalls ^ (*Prcb).InterruptTime ^
                                SystemTime.QuadPart as ULONG ^ (SystemTime.QuadPart >> 32) as ULONG;

                    Cookie = InterlockedCompareExchange(
                        core::ptr::addr_of_mut!((*process).Cookie) as *mut LONG,
                        NewCookie as LONG,
                        Cookie as LONG,
                    ) as ULONG;
                    if Cookie == 0 {
                        Cookie = NewCookie;
                    }

                    Length = core::mem::size_of::<ULONG>() as ULONG;
                }

                Status = STATUS_SUCCESS;

                if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, Cookie, PreviousMode) {
                    return s;
                }
            }
        }

        ProcessImageInformation => {
            if ProcessInformationLength != core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as ULONG;

                Status = STATUS_SUCCESS;

                MmGetImageInformation(ProcessInformation as PSECTION_IMAGE_INFORMATION);
            }
        }

        ProcessDebugObjectHandle => {
            let mut DebugPort: HANDLE = core::ptr::null_mut();

            if ProcessInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<HANDLE>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    Status = DbgkOpenProcessDebugPort(Process, PreviousMode, &mut DebugPort);

                    ObDereferenceObject(Process as PVOID);

                    if Status >= 0 {
                        if let Err(s) = probe_write_user::<PVOID>(ProcessInformation as *mut PVOID, DebugPort as PVOID, PreviousMode) {
                            return s;
                        }
                    } else if !DebugPort.is_null() {
                        ObCloseHandle(DebugPort, PreviousMode);
                    }
                }
            }
        }

        ProcessHandleTracing => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        ProcessLUIDDeviceMapsEnabled => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = STATUS_SUCCESS;

                if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, ObIsLUIDDeviceMapsEnabled(), PreviousMode) {
                    return s;
                }
            }
        }

        ProcessWx86Information => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;
                    if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, (*process).__bindgen_anon_2.__bindgen_anon_1.VdmAllowed(), PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Process as PVOID);
                }
            }
        }

        ProcessWow64Information => {
            let mut Wow64: ULONG_PTR = 0;

            if ProcessInformationLength != core::mem::size_of::<ULONG_PTR>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG_PTR>() as ULONG;

                Status = ObReferenceObjectByHandle(
                    ProcessHandle,
                    PROCESS_QUERY_INFORMATION,
                    PsProcessType,
                    PreviousMode,
                    &mut Process as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let process = Process as *mut _EPROCESS;

                    #[cfg(target_pointer_width = "64")]
                    {
                        if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) != 0 {
                            Wow64 = (*process).Wow64Process as ULONG_PTR;
                            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));
                        }
                    }

                    ObDereferenceObject(Process as PVOID);

                    if let Err(s) = probe_write_user::<ULONG_PTR>(ProcessInformation as *mut ULONG_PTR, Wow64, PreviousMode) {
                        return s;
                    }
                }
            }
        }

        ProcessExecuteFlags => {
            let mut ExecuteOptions: ULONG = 0;

            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Length = core::mem::size_of::<ULONG>() as ULONG;

                if ProcessHandle != NtCurrentProcess() {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    Status = MmGetExecuteOptions(&mut ExecuteOptions);
                    if Status >= 0 {
                        if let Err(s) = probe_write_user::<ULONG>(ProcessInformation as PULONG, ExecuteOptions, PreviousMode) {
                            return s;
                        }
                    }
                }
            }
        }

        ProcessLdtInformation | ProcessWorkingSetWatch | ProcessPooledUsageAndLimits => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        _ => {
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    if !ReturnLength.is_null() && Length != 0 {
        if let Err(s) = probe_write_user::<ULONG>(ReturnLength as PULONG, Length, PreviousMode) {
            return s;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationProcess(
    ProcessHandle: HANDLE,
    ProcessInformationClass: PROCESSINFOCLASS,
    ProcessInformation: PVOID,
    ProcessInformationLength: ULONG,
) -> NTSTATUS {
    let ProcessHandle = opaque_ptr(ProcessHandle);
    let ProcessInformation = opaque_ptr(ProcessInformation);
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Access: ACCESS_MASK;
    let mut Status: NTSTATUS;
    let mut PortHandle: HANDLE = core::ptr::null_mut();
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut SessionInfo: PROCESS_SESSION_INFORMATION = core::mem::zeroed();
    let mut PriorityClass: PROCESS_PRIORITY_CLASS = core::mem::zeroed();
    let mut Foreground: PROCESS_FOREGROUND_BACKGROUND = core::mem::zeroed();
    let mut ExceptionPort: PVOID = core::ptr::null_mut();
    let mut Break: ULONG;
    let mut ValidAffinity: KAFFINITY;
    let mut Affinity: KAFFINITY = 0;
    let mut BasePriority: KPRIORITY = 0;
    let mut MemoryPriority: UCHAR = 0;
    let mut DisableBoost: BOOLEAN = 0;
    let mut DefaultHardErrorMode: ULONG = 0;
    let mut DebugFlags: ULONG = 0;
    let mut EnableFixup: ULONG = 0;
    let mut Boost: ULONG = 0;
    let mut NoExecute: ULONG = 0;
    let mut VdmPower: ULONG = 0;
    let mut HasPrivilege: BOOLEAN;
    let mut Next: PLIST_ENTRY;
    let mut Thread: PETHREAD;

    Status = DefaultSetInfoBufferCheck(
        ProcessInformationClass,
        core::ptr::addr_of!(PsProcessInfoClass),
        PS_PROCESS_INFO_CLASS_COUNT,
        ProcessInformation,
        ProcessInformationLength,
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    Access = PROCESS_SET_INFORMATION;
    if ProcessInformationClass == ProcessSessionInformation {
        Access |= PROCESS_SET_SESSIONID;
    } else if ProcessInformationClass == ProcessExceptionPort {
        Access |= PROCESS_SUSPEND_RESUME;
    }

    Status = ObReferenceObjectByHandle(
        ProcessHandle,
        Access,
        PsProcessType,
        PreviousMode,
        &mut Process as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    let process = Process as *mut _EPROCESS;

    match ProcessInformationClass {
        ProcessWx86Information => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                VdmPower = *(ProcessInformation as PULONG);

                if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
                    Status = STATUS_PRIVILEGE_NOT_HELD;
                } else {
                    if VdmPower != 0 {
                        PspSetProcessFlag(process, PSF_VDM_ALLOWED_BIT);
                    } else {
                        PspClearProcessFlag(process, PSF_VDM_ALLOWED_BIT);
                    }
                }
            }
        }

        ProcessExceptionPort => {
            if ProcessInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                PortHandle = *(ProcessInformation as PHANDLE);

                if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
                    Status = STATUS_PRIVILEGE_NOT_HELD;
                } else {
                    Status = ObReferenceObjectByHandle(
                        PortHandle,
                        0,
                        LpcPortObjectType,
                        PreviousMode,
                        &mut ExceptionPort,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        if !InterlockedCompareExchangePointer(
                            core::ptr::addr_of_mut!((*process).ExceptionPort),
                            ExceptionPort,
                            core::ptr::null_mut(),
                        ).is_null() {
                            ObDereferenceObject(ExceptionPort);
                            Status = STATUS_PORT_ALREADY_SET;
                        }
                    }
                }
            }
        }

        ProcessAccessToken => {
            if ProcessInformationLength != core::mem::size_of::<PROCESS_ACCESS_TOKEN>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                TokenHandle = (*(ProcessInformation as *const PROCESS_ACCESS_TOKEN)).Token;

                Status = PspSetPrimaryToken(Process, TokenHandle, core::ptr::null_mut());
            }
        }

        ProcessDefaultHardErrorMode => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                DefaultHardErrorMode = *(ProcessInformation as PULONG);

                (*process).DefaultHardErrorProcessing = DefaultHardErrorMode;

                if (DefaultHardErrorMode & SEM_NOALIGNMENTFAULTEXCEPT) != 0 {
                    KeSetAutoAlignmentProcess(core::ptr::addr_of_mut!((*process).Pcb), 1u8);
                } else {
                    KeSetAutoAlignmentProcess(core::ptr::addr_of_mut!((*process).Pcb), 0u8);
                }
                Status = STATUS_SUCCESS;
            }
        }

        ProcessSessionInformation => {
            if ProcessInformationLength != core::mem::size_of::<PROCESS_SESSION_INFORMATION>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                SessionInfo = *(ProcessInformation as PPROCESS_SESSION_INFORMATION);

                if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
                    Status = STATUS_PRIVILEGE_NOT_HELD;
                } else {
                    if SessionInfo.SessionId == PsGetProcessSessionId(Process) {
                        Status = STATUS_SUCCESS;
                    } else {
                        Status = STATUS_ACCESS_DENIED;
                    }
                }
            }
        }

        ProcessPriorityClass => {
            if ProcessInformationLength != core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                PriorityClass = *(ProcessInformation as PPROCESS_PRIORITY_CLASS);

                if PriorityClass.PriorityClass > PROCESS_PRIORITY_CLASS_ABOVE_NORMAL as u8 {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    if (PriorityClass.PriorityClass != (*process).PriorityClass) &&
                       (PriorityClass.PriorityClass == PROCESS_PRIORITY_CLASS_REALTIME as u8)
                    {
                        HasPrivilege = SeCheckPrivilegedObject(
                            SeIncreaseBasePriorityPrivilege,
                            ProcessHandle,
                            PROCESS_SET_INFORMATION,
                            PreviousMode,
                        );
                        if HasPrivilege == 0 {
                            ObDereferenceObject(Process as PVOID);
                            return STATUS_PRIVILEGE_NOT_HELD;
                        }
                    }

                    if !(*process).Job.is_null() {
                    }

                    (*process).PriorityClass = PriorityClass.PriorityClass;

                    PsSetProcessPriorityByClass(
                        Process,
                        if PriorityClass.Foreground != 0 {
                            PsProcessPriorityForeground
                        } else {
                            PsProcessPriorityBackground
                        },
                    );
                    Status = STATUS_SUCCESS;
                }
            }
        }

        ProcessForegroundInformation => {
            if ProcessInformationLength != core::mem::size_of::<PROCESS_FOREGROUND_BACKGROUND>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Foreground = *(ProcessInformation as PPROCESS_FOREGROUND_BACKGROUND);

                PsSetProcessPriorityByClass(
                    Process,
                    if Foreground.Foreground != 0 {
                        PsProcessPriorityForeground
                    } else {
                        PsProcessPriorityBackground
                    },
                );
                Status = STATUS_SUCCESS;
            }
        }

        ProcessBasePriority => {
            if ProcessInformationLength != core::mem::size_of::<KPRIORITY>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                BasePriority = *(ProcessInformation as *const KPRIORITY);

                if (BasePriority & 0x80000000u32 as KPRIORITY) != 0 {
                    MemoryPriority = MEMORY_PRIORITY_FOREGROUND as UCHAR;
                    BasePriority &= !0x80000000u32 as KPRIORITY;
                } else {
                    MemoryPriority = MEMORY_PRIORITY_BACKGROUND as UCHAR;
                }

                if (BasePriority > HIGH_PRIORITY as KPRIORITY) || (BasePriority <= LOW_PRIORITY as KPRIORITY) {
                    ObDereferenceObject(Process as PVOID);
                    return STATUS_INVALID_PARAMETER;
                }

                if BasePriority > (*process).Pcb.BasePriority as KPRIORITY {
                    HasPrivilege = SeCheckPrivilegedObject(
                        SeIncreaseBasePriorityPrivilege,
                        ProcessHandle,
                        PROCESS_SET_INFORMATION,
                        PreviousMode,
                    );
                    if HasPrivilege == 0 {
                        ObDereferenceObject(Process as PVOID);
                        return STATUS_PRIVILEGE_NOT_HELD;
                    }
                }

                KeSetPriorityAndQuantumProcess(core::ptr::addr_of_mut!((*process).Pcb), BasePriority, 0);

                MmSetMemoryPriorityProcess(Process, MemoryPriority);
                Status = STATUS_SUCCESS;
            }
        }

        ProcessRaisePriority => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Boost = *(ProcessInformation as PULONG);

                if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) != 0 {
                    KeEnterCriticalRegion();
                    ExAcquirePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));

                    Next = (*process).ThreadListHead.Flink;
                    while Next != core::ptr::addr_of_mut!((*process).ThreadListHead) {
                        Thread = CONTAINING_RECORD!(Next, _ETHREAD, ThreadListEntry) as PETHREAD;
                        KeBoostPriorityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), Boost as KPRIORITY);
                        Next = (*Next).Flink;
                    }

                    ExReleasePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));
                    KeLeaveCriticalRegion();
                    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));

                    Status = STATUS_SUCCESS;
                } else {
                    Status = STATUS_PROCESS_IS_TERMINATING;
                }
            }
        }

        ProcessBreakOnTermination => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Break = *(ProcessInformation as PULONG);

                if SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode) == 0 {
                    Status = STATUS_PRIVILEGE_NOT_HELD;
                } else {
                    if Break != 0 {
                        PspSetProcessFlag(process, PSF_BREAK_ON_TERMINATION_BIT);
                    } else {
                        PspClearProcessFlag(process, PSF_BREAK_ON_TERMINATION_BIT);
                    }
                }
            }
        }

        ProcessAffinityMask => {
            if ProcessInformationLength != core::mem::size_of::<KAFFINITY>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Affinity = *(ProcessInformation as *const KAFFINITY);

                ValidAffinity = Affinity & KeActiveProcessors;
                if (Affinity == 0) || (ValidAffinity != Affinity) {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    if !(*process).Job.is_null() {
                        Status = STATUS_NOT_IMPLEMENTED;
                    } else if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) != 0 {
                        KeEnterCriticalRegion();
                        ExAcquirePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));

                        KeSetAffinityProcess(core::ptr::addr_of_mut!((*process).Pcb), ValidAffinity);

                        ExReleasePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));
                        KeLeaveCriticalRegion();
                        ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));

                        Status = STATUS_SUCCESS;
                    } else {
                        Status = STATUS_PROCESS_IS_TERMINATING;
                    }
                }
            }
        }

        ProcessPriorityBoost => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                DisableBoost = *(ProcessInformation as *const BOOLEAN);

                if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect)) != 0 {
                    KeEnterCriticalRegion();
                    ExAcquirePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));

                    KeSetDisableBoostProcess(core::ptr::addr_of_mut!((*process).Pcb), DisableBoost);

                    Next = (*process).ThreadListHead.Flink;
                    while Next != core::ptr::addr_of_mut!((*process).ThreadListHead) {
                        Thread = CONTAINING_RECORD!(Next, _ETHREAD, ThreadListEntry) as PETHREAD;
                        KeSetDisableBoostThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), DisableBoost);
                        Next = (*Next).Flink;
                    }

                    ExReleasePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));
                    KeLeaveCriticalRegion();
                    ExReleaseRundownProtection(core::ptr::addr_of_mut!((*process).RundownProtect));

                    Status = STATUS_SUCCESS;
                } else {
                    Status = STATUS_PROCESS_IS_TERMINATING;
                }
            }
        }

        ProcessDebugFlags => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                DebugFlags = *(ProcessInformation as PULONG);

                if (DebugFlags & !1) != 0 {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    if (DebugFlags & 1) != 0 {
                        PspClearProcessFlag(process, PSF_NO_DEBUG_INHERIT_BIT);
                    } else {
                        PspSetProcessFlag(process, PSF_NO_DEBUG_INHERIT_BIT);
                    }
                    Status = STATUS_SUCCESS;
                }
            }
        }

        ProcessEnableAlignmentFaultFixup => {
            if ProcessInformationLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                EnableFixup = *(ProcessInformation as PULONG);

                if EnableFixup != 0 {
                    (*process).DefaultHardErrorProcessing |= SEM_NOALIGNMENTFAULTEXCEPT;
                } else {
                    (*process).DefaultHardErrorProcessing &= !SEM_NOALIGNMENTFAULTEXCEPT;
                }

                KeSetAutoAlignmentProcess(core::ptr::addr_of_mut!((*process).Pcb), 0u8);
                Status = STATUS_SUCCESS;
            }
        }

        ProcessUserModeIOPL => {
            if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
                Status = STATUS_PRIVILEGE_NOT_HELD;
            } else {
                #[cfg(target_arch = "x86")]
                {
                    Ke386SetIOPL();
                }
                #[cfg(target_arch = "x86_64")]
                {
                    if (ExGetPreviousMode() != 0i8) &&
                       (RtlRosGetAppcompatVersion() > _WIN32_WINNT_WS03)
                    {
                        Status = STATUS_NOT_IMPLEMENTED;
                    }
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    Status = STATUS_NOT_IMPLEMENTED;
                }
            }
        }

        ProcessExecuteFlags => {
            if ProcessInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else if ProcessHandle != NtCurrentProcess() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                NoExecute = *(ProcessInformation as PULONG);

                Status = MmSetExecuteOptions(NoExecute);
            }
        }

        ProcessDeviceMap => {
            if ProcessInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                DirectoryHandle = *(ProcessInformation as PHANDLE);

                Status = ObSetDeviceMap(Process, DirectoryHandle);
            }
        }

        ProcessLdtInformation | ProcessLdtSize | ProcessIoPortHandlers => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        ProcessQuotaLimits => {
            Status = PspSetQuotaLimits(
                Process,
                1,
                ProcessInformation,
                ProcessInformationLength,
                PreviousMode,
            );
        }

        ProcessWorkingSetWatch | ProcessHandleTracing => {
            Status = STATUS_NOT_IMPLEMENTED;
        }

        _ => {
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    ObDereferenceObject(Process as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: THREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: ULONG,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ThreadInformation = opaque_ptr(ThreadInformation);
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let mut Priority: KPRIORITY = 0;
    let mut Affinity: KAFFINITY = 0;
    let mut CombinedAffinity: KAFFINITY;
    let mut Address: PVOID = core::ptr::null_mut();
    let mut Process: PEPROCESS;
    let mut DisableBoost: ULONG_PTR = 0;
    let mut IdealProcessor: ULONG_PTR = 0;
    let mut Break: ULONG_PTR = 0;
    let mut Teb: PTEB;
    let mut TlsIndex: ULONG_PTR = 0;
    let mut ExpansionSlots: *mut PVOID;
    let mut ProcThread: PETHREAD;
    let mut HasPrivilege: BOOLEAN;

    Status = DefaultSetInfoBufferCheck(
        ThreadInformationClass,
        core::ptr::addr_of!(PsThreadInfoClass),
        PS_THREAD_INFO_CLASS_COUNT,
        ThreadInformation,
        ThreadInformationLength,
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    match ThreadInformationClass {
        ThreadPriority => {
            if ThreadInformationLength != core::mem::size_of::<KPRIORITY>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Priority = *(ThreadInformation as *const LONG);

                if (Priority > HIGH_PRIORITY as LONG) || (Priority <= LOW_PRIORITY as LONG) {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    if Priority >= LOW_REALTIME_PRIORITY as LONG {
                        HasPrivilege = SeCheckPrivilegedObject(
                            SeIncreaseBasePriorityPrivilege,
                            ThreadHandle,
                            THREAD_SET_INFORMATION,
                            PreviousMode,
                        );
                        if HasPrivilege == 0 {
                            return STATUS_PRIVILEGE_NOT_HELD;
                        }
                    }

                    Status = ObReferenceObjectByHandle(
                        ThreadHandle,
                        THREAD_SET_INFORMATION,
                        PsThreadType,
                        PreviousMode,
                        &mut Thread as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        KeSetPriorityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), Priority);

                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadBasePriority => {
            if ThreadInformationLength != core::mem::size_of::<LONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Priority = *(ThreadInformation as *const LONG);

                if (Priority > THREAD_BASE_PRIORITY_MAX) || (Priority < THREAD_BASE_PRIORITY_MIN) {
                    if (Priority != THREAD_BASE_PRIORITY_LOWRT + 1) &&
                       (Priority != THREAD_BASE_PRIORITY_IDLE - 1)
                    {
                        if (*(PsGetCurrentProcess() as *mut _EPROCESS)).PriorityClass !=
                           PROCESS_PRIORITY_CLASS_REALTIME as UCHAR
                        {
                            Status = STATUS_INVALID_PARAMETER;
                        } else {
                            Status = ObReferenceObjectByHandle(
                                ThreadHandle,
                                THREAD_SET_INFORMATION,
                                PsThreadType,
                                PreviousMode,
                                &mut Thread as *mut _ as *mut PVOID,
                                core::ptr::null_mut(),
                            );
                            if Status >= 0 {
                                KeSetBasePriorityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), Priority);
                                ObDereferenceObject(Thread as PVOID);
                            }
                        }
                    } else {
                        Status = ObReferenceObjectByHandle(
                            ThreadHandle,
                            THREAD_SET_INFORMATION,
                            PsThreadType,
                            PreviousMode,
                            &mut Thread as *mut _ as *mut PVOID,
                            core::ptr::null_mut(),
                        );
                        if Status >= 0 {
                            KeSetBasePriorityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), Priority);
                            ObDereferenceObject(Thread as PVOID);
                        }
                    }
                } else {
                    Status = ObReferenceObjectByHandle(
                        ThreadHandle,
                        THREAD_SET_INFORMATION,
                        PsThreadType,
                        PreviousMode,
                        &mut Thread as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        KeSetBasePriorityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), Priority);
                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadAffinityMask => {
            if ThreadInformationLength != core::mem::size_of::<ULONG_PTR>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Affinity = *(ThreadInformation as *const ULONG_PTR);

                if Affinity == 0 {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    Status = ObReferenceObjectByHandle(
                        ThreadHandle,
                        THREAD_SET_INFORMATION,
                        PsThreadType,
                        PreviousMode,
                        &mut Thread as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        Process = (*(Thread as *mut _ETHREAD)).ThreadsProcess as PEPROCESS;

                        if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).RundownProtect)) != 0 {
                            KeEnterCriticalRegion();
                            ExAcquirePushLockShared(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).ProcessLock));

                            CombinedAffinity = Affinity & (*(Process as *mut _EPROCESS)).Pcb.Affinity;
                            if CombinedAffinity != Affinity {
                                Status = STATUS_INVALID_PARAMETER;
                            } else {
                                KeSetAffinityThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), CombinedAffinity);
                            }

                            ExReleasePushLockShared(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).ProcessLock));
                            KeLeaveCriticalRegion();
                            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).RundownProtect));
                        } else {
                            Status = STATUS_PROCESS_IS_TERMINATING;
                        }

                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadImpersonationToken => {
            if ThreadInformationLength != core::mem::size_of::<HANDLE>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                TokenHandle = *(ThreadInformation as PHANDLE);

                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    THREAD_SET_THREAD_TOKEN,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    Status = PsAssignImpersonationToken(Thread, TokenHandle);

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadQuerySetWin32StartAddress => {
            if ThreadInformationLength != core::mem::size_of::<ULONG_PTR>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Address = *(ThreadInformation as *mut PVOID);

                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    THREAD_SET_INFORMATION,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    (*(Thread as *mut _ETHREAD)).Win32StartAddress = Address;

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadIdealProcessor => {
            if ThreadInformationLength != core::mem::size_of::<ULONG_PTR>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                IdealProcessor = *(ThreadInformation as *const ULONG_PTR);

                if IdealProcessor > MAXIMUM_PROCESSORS as ULONG_PTR {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    Status = ObReferenceObjectByHandle(
                        ThreadHandle,
                        THREAD_SET_INFORMATION,
                        PsThreadType,
                        PreviousMode,
                        &mut Thread as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        KeSetIdealProcessorThread(
                            core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb),
                            IdealProcessor as UCHAR,
                        );

                        Teb = (*(Thread as *mut _ETHREAD)).Tcb.Teb;
                        if !Teb.is_null() && (ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).RundownProtect)) != 0) {
                            (*Teb).IdealProcessor = (*(Thread as *mut _ETHREAD)).Tcb.IdealProcessor;

                            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).RundownProtect));
                        }

                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadPriorityBoost => {
            if ThreadInformationLength != core::mem::size_of::<ULONG_PTR>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                DisableBoost = *(ThreadInformation as *const ULONG_PTR);

                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    THREAD_SET_INFORMATION,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    KeSetDisableBoostThread(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).Tcb), DisableBoost as BOOLEAN);

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadZeroTlsCell => {
            if ThreadInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                TlsIndex = *(ThreadInformation as PULONG) as ULONG_PTR;

                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    THREAD_SET_INFORMATION,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    if Thread != PsGetCurrentThread() {
                        Status = STATUS_INVALID_PARAMETER;
                        ObDereferenceObject(Thread as PVOID);
                    } else {
                        Process = (*(Thread as *mut _ETHREAD)).ThreadsProcess as PEPROCESS;

                        // A 32-bit guest's TLS cells live in TEB32, not TEB64.
                        // TEB64's low slots belong to WoW64 itself (slot 1 is
                        // WOW64_TLS_CPURESERVED, wow64.dll's per-thread guest
                        // register file), so zeroing TEB64 here let a guest
                        // TlsFree() wipe that pointer for EVERY thread in the
                        // process — the guest then took a breakpoint with no CPU
                        // area and wow64.dll faulted inside its own VEH.
                        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                        let IsWow64 = !(*(Process as *mut _EPROCESS)).Wow64Process.is_null();
                        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
                        let IsWow64 = false;

                        ProcThread = PsGetNextProcessThread(Process, core::ptr::null_mut());
                        while !ProcThread.is_null() {
                            if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(ProcThread as *mut _ETHREAD)).RundownProtect)) != 0 {
                                Teb = (*(ProcThread as *mut _ETHREAD)).Tcb.Teb;
                                if !Teb.is_null() && IsWow64 {
                                    // TEB32 shares TEB64's combined region
                                    // (MmCreateTeb); its TlsSlots are ULONG.
                                    let Teb32 =
                                        (Teb as *mut u8).add(ntkrnl_sys::wow64::WOW64_TEB32_OFFSET);
                                    if TlsIndex > (TLS_MINIMUM_AVAILABLE - 1) as ULONG_PTR {
                                        if TlsIndex < ((TLS_MINIMUM_AVAILABLE + TLS_EXPANSION_SLOTS) - 1) as ULONG_PTR {
                                            let Exp32 = *(Teb32.add(0xF94) as *mut u32); // nolint: magic-offset (TEB32.TlsExpansionSlots)
                                            if Exp32 != 0 {
                                                *((Exp32 as ULONG_PTR as *mut u32)
                                                    .add((TlsIndex - TLS_MINIMUM_AVAILABLE as ULONG_PTR) as usize)) = 0;
                                            }
                                        }
                                    } else {
                                        *(Teb32.add(0xE10 + TlsIndex as usize * 4) as *mut u32) = 0; // nolint: magic-offset (TEB32.TlsSlots)
                                    }
                                } else if !Teb.is_null() {
                                    if TlsIndex > (TLS_MINIMUM_AVAILABLE - 1) as ULONG_PTR {
                                        if TlsIndex < ((TLS_MINIMUM_AVAILABLE + TLS_EXPANSION_SLOTS) - 1) as ULONG_PTR {
                                            ExpansionSlots = (*Teb).TlsExpansionSlots;
                                            if !ExpansionSlots.is_null() {
                                                *ExpansionSlots.offset((TlsIndex - TLS_MINIMUM_AVAILABLE as ULONG_PTR) as isize) = core::ptr::null_mut();
                                            }
                                        }
                                    } else {
                                        (*Teb).TlsSlots[TlsIndex as usize] = core::ptr::null_mut();
                                    }
                                }

                                ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(ProcThread as *mut _ETHREAD)).RundownProtect));
                            }

                            ProcThread = PsGetNextProcessThread(Process, ProcThread);
                        }

                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadBreakOnTermination => {
            if ThreadInformationLength != core::mem::size_of::<ULONG>() as ULONG {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Break = *(ThreadInformation as PULONG) as ULONG_PTR;

                if SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode) == 0 {
                    Status = STATUS_PRIVILEGE_NOT_HELD;
                } else {
                    Status = ObReferenceObjectByHandle(
                        ThreadHandle,
                        THREAD_SET_INFORMATION,
                        PsThreadType,
                        PreviousMode,
                        &mut Thread as *mut _ as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if Status >= 0 {
                        if Break != 0 {
                            PspSetCrossThreadFlag(Thread as *mut _ETHREAD, CT_BREAK_ON_TERMINATION_BIT);
                        } else {
                            PspClearCrossThreadFlag(Thread as *mut _ETHREAD, CT_BREAK_ON_TERMINATION_BIT);
                        }

                        ObDereferenceObject(Thread as PVOID);
                    }
                }
            }
        }

        ThreadHideFromDebugger => {
            if ThreadInformationLength != 0 {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    THREAD_SET_INFORMATION,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    PspSetCrossThreadFlag(Thread as *mut _ETHREAD, CT_HIDE_FROM_DEBUGGER_BIT);

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: THREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let ThreadHandle = opaque_ptr(ThreadHandle);
    let ThreadInformation = opaque_ptr(ThreadInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let Access: ULONG;
    let mut Length: ULONG = 0;
    let ThreadBasicInfo = ThreadInformation as PTHREAD_BASIC_INFORMATION;
    let ThreadTime = ThreadInformation as PKERNEL_USER_TIMES;
    let mut OldIrql: KIRQL = 0;
    let mut ThreadTerminated: ULONG;

    Status = DefaultQueryInfoBufferCheck(
        ThreadInformationClass,
        core::ptr::addr_of!(PsThreadInfoClass),
        PS_THREAD_INFO_CLASS_COUNT,
        ICIF_PROBE_READ,
        ThreadInformation,
        ThreadInformationLength,
        ReturnLength,
        core::ptr::null_mut(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    Access = THREAD_QUERY_INFORMATION;

    match ThreadInformationClass {
        ThreadBasicInformation => {
            Length = core::mem::size_of::<THREAD_BASIC_INFORMATION>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let thread = Thread as *mut _ETHREAD;
                    (*ThreadBasicInfo).ExitStatus = (*thread).__bindgen_anon_2.ExitStatus;
                    (*ThreadBasicInfo).TebBaseAddress = (*thread).Tcb.Teb as PVOID;
                    (*ThreadBasicInfo).ClientId = (*thread).Cid;
                    (*ThreadBasicInfo).AffinityMask = (*thread).Tcb.Affinity;
                    (*ThreadBasicInfo).Priority = (*thread).Tcb.Priority as LONG;
                    (*ThreadBasicInfo).BasePriority = KeQueryBasePriorityThread(core::ptr::addr_of_mut!((*thread).Tcb));

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadTimes => {
            Length = core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let thread = Thread as *mut _ETHREAD;
                    (*ThreadTime).KernelTime.QuadPart = (kthread_kernel_time(thread) as LONGLONG) * (KeMaximumIncrement as LONGLONG);
                    (*ThreadTime).UserTime.QuadPart = (kthread_user_time(thread) as LONGLONG) * (KeMaximumIncrement as LONGLONG);
                    (*ThreadTime).CreateTime = (*thread).CreateTime;

                    if KeReadStateThread(core::ptr::addr_of_mut!((*thread).Tcb)) != 0 {
                        (*ThreadTime).ExitTime = (*thread).__bindgen_anon_1.ExitTime;
                    } else {
                        (*ThreadTime).ExitTime.QuadPart = 0;
                    }

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadQuerySetWin32StartAddress => {
            Length = core::mem::size_of::<PVOID>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    *(ThreadInformation as *mut PVOID) = (*(Thread as *mut _ETHREAD)).Win32StartAddress;

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadPerformanceCount => {
            Length = core::mem::size_of::<LARGE_INTEGER>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    (*(ThreadInformation as *mut LARGE_INTEGER)).QuadPart = 0;

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadAmILastThread => {
            Length = core::mem::size_of::<ULONG>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let thread = Thread as *mut _ETHREAD;
                    let process = (*thread).ThreadsProcess as *mut _EPROCESS;
                    *(ThreadInformation as PULONG) = if (*(*process).ThreadListHead.Flink).Flink ==
                        core::ptr::addr_of_mut!((*process).ThreadListHead)
                    {
                        1
                    } else {
                        0
                    };

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadIsIoPending => {
            Length = core::mem::size_of::<ULONG>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    // [guard] KeRaiseIrql(APC_LEVEL as KIRQL, &mut OldIrql);
                    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL as KIRQL) };

                    *(ThreadInformation as PULONG) = if IsListEmpty(core::ptr::addr_of_mut!((*(Thread as *mut _ETHREAD)).IrpList)) == 0 {
                        1
                    } else {
                        0
                    };

                    drop(irql_guard);

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadDescriptorTableEntry => {
            #[cfg(target_arch = "x86")]
            {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    Status = PspQueryDescriptorThread(
                        Thread,
                        ThreadInformation,
                        ThreadInformationLength,
                        ReturnLength,
                    );

                    ObDereferenceObject(Thread as PVOID);
                }
            }
            #[cfg(not(target_arch = "x86"))]
            {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }

        ThreadPriorityBoost => {
            Length = core::mem::size_of::<ULONG>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    let boost_val = if kthread_disable_boost(Thread as *mut _ETHREAD) != 0 { 1u32 } else { 0u32 };
                    if let Err(s) = probe_write_user::<ULONG>(ThreadInformation as PULONG, boost_val, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadBreakOnTermination => {
            Length = core::mem::size_of::<ULONG>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    if let Err(s) = probe_write_user::<ULONG>(ThreadInformation as PULONG, (*(Thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.BreakOnTermination(), PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        ThreadIsTerminated => {
            Length = core::mem::size_of::<ULONG>() as ULONG;

            if ThreadInformationLength != Length {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                Status = ObReferenceObjectByHandle(
                    ThreadHandle,
                    Access,
                    PsThreadType,
                    PreviousMode,
                    &mut Thread as *mut _ as *mut PVOID,
                    core::ptr::null_mut(),
                );
                if Status >= 0 {
                    ThreadTerminated = PsIsThreadTerminating(Thread) as ULONG;

                    let term_val = if ThreadTerminated != 0 { 1u32 } else { 0u32 };
                    if let Err(s) = probe_write_user::<ULONG>(ThreadInformation as PULONG, term_val, PreviousMode) {
                        return s;
                    }

                    ObDereferenceObject(Thread as PVOID);
                }
            }
        }

        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    if !ReturnLength.is_null() {
        if let Err(s) = probe_write_user::<ULONG>(ReturnLength as PULONG, Length, PreviousMode) {
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
//  * FILE:            ntoskrnl/ps/query.c
//  * PURPOSE:         Process Manager: Thread/Process Query/Set Information
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Debugging Level */
// ULONG PspTraceLevel = 0;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// PsReferenceProcessFilePointer(
//     _In_ PEPROCESS Process,
//     _Outptr_ PFILE_OBJECT *FileObject)
// {
//     PSECTION Section;
//     PAGED_CODE();
//
//     /* Lock the process */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Get the section */
//     Section = Process->SectionObject;
//     if (Section)
//     {
//         /* Get the file object and reference it */
//         *FileObject = MmGetFileObjectForSection((PVOID)Section);
//         ObReferenceObject(*FileObject);
//     }
//
//     /* Release the protection */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//
//     /* Return status */
//     return Section ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
// }
//
// #if DBG
// static
// PCSTR
// PspDumpProcessInfoClassName(
//     _In_ PROCESSINFOCLASS ProcessInformationClass)
// {
//     static CHAR UnknownClassName[11];
//
// #define DBG_PROCESS_INFO_CLASS(InfoClass)   [InfoClass] = #InfoClass
//     static const PCSTR ProcessInfoClasses[] =
//     {
//         DBG_PROCESS_INFO_CLASS(ProcessBasicInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessQuotaLimits),
//         DBG_PROCESS_INFO_CLASS(ProcessVmCounters),
//         DBG_PROCESS_INFO_CLASS(ProcessTimes),
//         DBG_PROCESS_INFO_CLASS(ProcessBasePriority),
//         DBG_PROCESS_INFO_CLASS(ProcessRaisePriority),
//         DBG_PROCESS_INFO_CLASS(ProcessDebugPort),
//         DBG_PROCESS_INFO_CLASS(ProcessExceptionPort),
//         DBG_PROCESS_INFO_CLASS(ProcessAccessToken),
//         DBG_PROCESS_INFO_CLASS(ProcessLdtInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessLdtSize),
//         DBG_PROCESS_INFO_CLASS(ProcessDefaultHardErrorMode),
//         DBG_PROCESS_INFO_CLASS(ProcessIoPortHandlers),
//         DBG_PROCESS_INFO_CLASS(ProcessPooledUsageAndLimits),
//         DBG_PROCESS_INFO_CLASS(ProcessWorkingSetWatch),
//         DBG_PROCESS_INFO_CLASS(ProcessUserModeIOPL),
//         DBG_PROCESS_INFO_CLASS(ProcessEnableAlignmentFaultFixup),
//         DBG_PROCESS_INFO_CLASS(ProcessPriorityClass),
//         DBG_PROCESS_INFO_CLASS(ProcessWx86Information),
//         DBG_PROCESS_INFO_CLASS(ProcessHandleCount),
//         DBG_PROCESS_INFO_CLASS(ProcessAffinityMask),
//         DBG_PROCESS_INFO_CLASS(ProcessPriorityBoost),
//         DBG_PROCESS_INFO_CLASS(ProcessDeviceMap),
//         DBG_PROCESS_INFO_CLASS(ProcessSessionInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessForegroundInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessWow64Information),
//         DBG_PROCESS_INFO_CLASS(ProcessImageFileName),
//         DBG_PROCESS_INFO_CLASS(ProcessLUIDDeviceMapsEnabled),
//         DBG_PROCESS_INFO_CLASS(ProcessBreakOnTermination),
//         DBG_PROCESS_INFO_CLASS(ProcessDebugObjectHandle),
//         DBG_PROCESS_INFO_CLASS(ProcessDebugFlags),
//         DBG_PROCESS_INFO_CLASS(ProcessHandleTracing),
//         DBG_PROCESS_INFO_CLASS(ProcessIoPriority),
//         DBG_PROCESS_INFO_CLASS(ProcessExecuteFlags),
//         DBG_PROCESS_INFO_CLASS(ProcessTlsInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessCookie),
//         DBG_PROCESS_INFO_CLASS(ProcessImageInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessCycleTime),
//         DBG_PROCESS_INFO_CLASS(ProcessPagePriority),
//         DBG_PROCESS_INFO_CLASS(ProcessInstrumentationCallback),
//         DBG_PROCESS_INFO_CLASS(ProcessThreadStackAllocation),
//         DBG_PROCESS_INFO_CLASS(ProcessWorkingSetWatchEx),
//         DBG_PROCESS_INFO_CLASS(ProcessImageFileNameWin32),
//         DBG_PROCESS_INFO_CLASS(ProcessImageFileMapping),
//         DBG_PROCESS_INFO_CLASS(ProcessAffinityUpdateMode),
//         DBG_PROCESS_INFO_CLASS(ProcessMemoryAllocationMode),
//         DBG_PROCESS_INFO_CLASS(ProcessGroupInformation),
//         DBG_PROCESS_INFO_CLASS(ProcessConsoleHostProcess),
//         DBG_PROCESS_INFO_CLASS(ProcessWindowInformation),
//     };
// #undef DBG_PROCESS_INFO_CLASS
//
//     if (ProcessInformationClass < RTL_NUMBER_OF(ProcessInfoClasses))
//     {
//         return ProcessInfoClasses[ProcessInformationClass];
//     }
//
//     sprintf(UnknownClassName, "%lu", ProcessInformationClass);
//     return UnknownClassName;
// }
//
// static
// PCSTR
// PspDumpThreadInfoClassName(
//     _In_ THREADINFOCLASS ThreadInformationClass)
// {
//     static CHAR UnknownClassName[11];
//
// #define DBG_THREAD_INFO_CLASS(InfoClass)   [InfoClass] = #InfoClass
//     static const PCSTR ThreadInfoClasses[] =
//     {
//         DBG_THREAD_INFO_CLASS(ThreadBasicInformation),
//         DBG_THREAD_INFO_CLASS(ThreadTimes),
//         DBG_THREAD_INFO_CLASS(ThreadPriority),
//         DBG_THREAD_INFO_CLASS(ThreadBasePriority),
//         DBG_THREAD_INFO_CLASS(ThreadAffinityMask),
//         DBG_THREAD_INFO_CLASS(ThreadImpersonationToken),
//         DBG_THREAD_INFO_CLASS(ThreadDescriptorTableEntry),
//         DBG_THREAD_INFO_CLASS(ThreadEnableAlignmentFaultFixup),
//         DBG_THREAD_INFO_CLASS(ThreadEventPair_Reusable),
//         DBG_THREAD_INFO_CLASS(ThreadQuerySetWin32StartAddress),
//         DBG_THREAD_INFO_CLASS(ThreadZeroTlsCell),
//         DBG_THREAD_INFO_CLASS(ThreadPerformanceCount),
//         DBG_THREAD_INFO_CLASS(ThreadAmILastThread),
//         DBG_THREAD_INFO_CLASS(ThreadIdealProcessor),
//         DBG_THREAD_INFO_CLASS(ThreadPriorityBoost),
//         DBG_THREAD_INFO_CLASS(ThreadSetTlsArrayAddress),
//         DBG_THREAD_INFO_CLASS(ThreadIsIoPending),
//         DBG_THREAD_INFO_CLASS(ThreadHideFromDebugger),
//         DBG_THREAD_INFO_CLASS(ThreadBreakOnTermination),
//         DBG_THREAD_INFO_CLASS(ThreadSwitchLegacyState),
//         DBG_THREAD_INFO_CLASS(ThreadIsTerminated),
//         DBG_THREAD_INFO_CLASS(ThreadLastSystemCall),
//         DBG_THREAD_INFO_CLASS(ThreadIoPriority),
//         DBG_THREAD_INFO_CLASS(ThreadCycleTime),
//         DBG_THREAD_INFO_CLASS(ThreadPagePriority),
//         DBG_THREAD_INFO_CLASS(ThreadActualBasePriority),
//         DBG_THREAD_INFO_CLASS(ThreadTebInformation),
//         DBG_THREAD_INFO_CLASS(ThreadCSwitchMon),
//         DBG_THREAD_INFO_CLASS(ThreadCSwitchPmu),
//         DBG_THREAD_INFO_CLASS(ThreadWow64Context),
//         DBG_THREAD_INFO_CLASS(ThreadGroupInformation),
//         DBG_THREAD_INFO_CLASS(ThreadUmsInformation),
//         DBG_THREAD_INFO_CLASS(ThreadCounterProfiling),
//         DBG_THREAD_INFO_CLASS(ThreadIdealProcessorEx),
//         DBG_THREAD_INFO_CLASS(ThreadCpuAccountingInformation),
//         DBG_THREAD_INFO_CLASS(ThreadSuspendCount),
//         DBG_THREAD_INFO_CLASS(ThreadHeterogeneousCpuPolicy),
//         DBG_THREAD_INFO_CLASS(ThreadContainerId),
//         DBG_THREAD_INFO_CLASS(ThreadNameInformation),
//         DBG_THREAD_INFO_CLASS(ThreadSelectedCpuSets),
//         DBG_THREAD_INFO_CLASS(ThreadSystemThreadInformation),
//         DBG_THREAD_INFO_CLASS(ThreadActualGroupAffinity),
//         DBG_THREAD_INFO_CLASS(ThreadDynamicCodePolicyInfo),
//         DBG_THREAD_INFO_CLASS(ThreadExplicitCaseSensitivity),
//         DBG_THREAD_INFO_CLASS(ThreadWorkOnBehalfTicket),
//         DBG_THREAD_INFO_CLASS(ThreadSubsystemInformation),
//         DBG_THREAD_INFO_CLASS(ThreadDbgkWerReportActive),
//         DBG_THREAD_INFO_CLASS(ThreadAttachContainer),
//         DBG_THREAD_INFO_CLASS(ThreadManageWritesToExecutableMemory),
//         DBG_THREAD_INFO_CLASS(ThreadPowerThrottlingState),
//         DBG_THREAD_INFO_CLASS(ThreadWorkloadClass),
//         DBG_THREAD_INFO_CLASS(ThreadCreateStateChange),
//         DBG_THREAD_INFO_CLASS(ThreadApplyStateChange),
//         DBG_THREAD_INFO_CLASS(ThreadStrongerBadHandleChecks),
//         DBG_THREAD_INFO_CLASS(ThreadEffectiveIoPriority),
//         DBG_THREAD_INFO_CLASS(ThreadEffectivePagePriority),
//     };
// #undef DBG_THREAD_INFO_CLASS
//
//     if (ThreadInformationClass < RTL_NUMBER_OF(ThreadInfoClasses))
//     {
//         return ThreadInfoClasses[ThreadInformationClass];
//     }
//
//     sprintf(UnknownClassName, "%lu", ThreadInformationClass);
//     return UnknownClassName;
// }
// #endif // #if DBG
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryInformationProcess(
//     _In_ HANDLE ProcessHandle,
//     _In_ PROCESSINFOCLASS ProcessInformationClass,
//     _Out_writes_bytes_to_opt_(ProcessInformationLength, *ReturnLength)
//         PVOID ProcessInformation,
//     _In_ ULONG ProcessInformationLength,
//     _Out_opt_ PULONG ReturnLength)
// {
//     PEPROCESS Process;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     ULONG Length = 0;
//
//     PAGED_CODE();
//
//     /* Validate the information class */
//     Status = DefaultQueryInfoBufferCheck(ProcessInformationClass,
//                                          PsProcessInfoClass,
//                                          RTL_NUMBER_OF(PsProcessInfoClass),
//                                          ICIF_PROBE_READ,
//                                          ProcessInformation,
//                                          ProcessInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
// #if DBG
//         DPRINT1("NtQueryInformationProcess(ProcessInformationClass: %s): Class validation failed! (Status: 0x%lx)\n",
//                 PspDumpProcessInfoClassName(ProcessInformationClass), Status);
// #endif
//         return Status;
//     }
//
//     if (((ProcessInformationClass == ProcessCookie) ||
//          (ProcessInformationClass == ProcessImageInformation)) &&
//         (ProcessHandle != NtCurrentProcess()))
//     {
//         /*
//          * Retrieving the process cookie is only allowed for the calling process
//          * itself! XP only allows NtCurrentProcess() as process handles even if
//          * a real handle actually represents the current process.
//          */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check the information class */
//     switch (ProcessInformationClass)
//     {
//         /* Basic process information */
//         case ProcessBasicInformation:
//         {
//             PPROCESS_BASIC_INFORMATION ProcessBasicInfo = (PPROCESS_BASIC_INFORMATION)ProcessInformation;
//
//             if (ProcessInformationLength != sizeof(PROCESS_BASIC_INFORMATION))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(PROCESS_BASIC_INFORMATION);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Protect writes with SEH */
//             _SEH2_TRY
//             {
//                 /* Write all the information from the EPROCESS/KPROCESS */
//                 ProcessBasicInfo->ExitStatus = Process->ExitStatus;
//                 ProcessBasicInfo->PebBaseAddress = Process->Peb;
//                 ProcessBasicInfo->AffinityMask = Process->Pcb.Affinity;
//                 ProcessBasicInfo->UniqueProcessId = (ULONG_PTR)Process->
//                                                     UniqueProcessId;
//                 ProcessBasicInfo->InheritedFromUniqueProcessId =
//                     (ULONG_PTR)Process->InheritedFromUniqueProcessId;
//                 ProcessBasicInfo->BasePriority = Process->Pcb.BasePriority;
//
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Process quota limits */
//         case ProcessQuotaLimits:
//         {
//             QUOTA_LIMITS_EX QuotaLimits;
//             BOOLEAN Extended;
//
//             if (ProcessInformationLength != sizeof(QUOTA_LIMITS) &&
//                 ProcessInformationLength != sizeof(QUOTA_LIMITS_EX))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = ProcessInformationLength;
//             Extended = (Length == sizeof(QUOTA_LIMITS_EX));
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Indicate success */
//             Status = STATUS_SUCCESS;
//
//             RtlZeroMemory(&QuotaLimits, sizeof(QuotaLimits));
//
//             /* Get max/min working set sizes */
//             QuotaLimits.MaximumWorkingSetSize =
//                 Process->Vm.MaximumWorkingSetSize << PAGE_SHIFT;
//             QuotaLimits.MinimumWorkingSetSize =
//                 Process->Vm.MinimumWorkingSetSize << PAGE_SHIFT;
//
//             /* Get default time limits */
//             QuotaLimits.TimeLimit.QuadPart = -1LL;
//
//             /* Is quota block a default one? */
//             if (Process->QuotaBlock == &PspDefaultQuotaBlock)
//             {
//                 /* Get default pools and pagefile limits */
//                 QuotaLimits.PagedPoolLimit = (SIZE_T)-1;
//                 QuotaLimits.NonPagedPoolLimit = (SIZE_T)-1;
//                 QuotaLimits.PagefileLimit = (SIZE_T)-1;
//             }
//             else
//             {
//                 /* Get limits from non-default quota block */
//                 QuotaLimits.PagedPoolLimit =
//                     Process->QuotaBlock->QuotaEntry[PsPagedPool].Limit;
//                 QuotaLimits.NonPagedPoolLimit =
//                     Process->QuotaBlock->QuotaEntry[PsNonPagedPool].Limit;
//                 QuotaLimits.PagefileLimit =
//                     Process->QuotaBlock->QuotaEntry[PsPageFile].Limit;
//             }
//
//             /* Get additional information, if needed */
//             if (Extended)
//             {
//                 QuotaLimits.Flags |= (Process->Vm.Flags.MaximumWorkingSetHard ?
//                     QUOTA_LIMITS_HARDWS_MAX_ENABLE : QUOTA_LIMITS_HARDWS_MAX_DISABLE);
//                 QuotaLimits.Flags |= (Process->Vm.Flags.MinimumWorkingSetHard ?
//                     QUOTA_LIMITS_HARDWS_MIN_ENABLE : QUOTA_LIMITS_HARDWS_MIN_DISABLE);
//
//                 /* FIXME: Get the correct information */
//                 //QuotaLimits.WorkingSetLimit = (SIZE_T)-1; // Not used on Win2k3, it is set to 0
//                 QuotaLimits.CpuRateLimit.RateData = 0;
//             }
//
//             /* Protect writes with SEH */
//             _SEH2_TRY
//             {
//                 RtlCopyMemory(ProcessInformation, &QuotaLimits, Length);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         case ProcessIoCounters:
//         {
//             PIO_COUNTERS IoCounters = (PIO_COUNTERS)ProcessInformation;
//             PROCESS_VALUES ProcessValues;
//
//             if (ProcessInformationLength != sizeof(IO_COUNTERS))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             Length = sizeof(IO_COUNTERS);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Query IO counters from the process */
//             KeQueryValuesProcess(&Process->Pcb, &ProcessValues);
//
//             _SEH2_TRY
//             {
//                 RtlCopyMemory(IoCounters, &ProcessValues.IoInfo, sizeof(IO_COUNTERS));
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Ignore exception */
//             }
//             _SEH2_END;
//
//             /* Set status to success in any case */
//             Status = STATUS_SUCCESS;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Timing */
//         case ProcessTimes:
//         {
//             PKERNEL_USER_TIMES ProcessTime = (PKERNEL_USER_TIMES)ProcessInformation;
//             ULONG UserTime, KernelTime;
//
//             /* Set the return length */
//             if (ProcessInformationLength != sizeof(KERNEL_USER_TIMES))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             Length = sizeof(KERNEL_USER_TIMES);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Protect writes with SEH */
//             _SEH2_TRY
//             {
//                 /* Copy time information from EPROCESS/KPROCESS */
//                 KernelTime = KeQueryRuntimeProcess(&Process->Pcb, &UserTime);
//                 ProcessTime->CreateTime = Process->CreateTime;
//                 ProcessTime->UserTime.QuadPart = (LONGLONG)UserTime * KeMaximumIncrement;
//                 ProcessTime->KernelTime.QuadPart = (LONGLONG)KernelTime * KeMaximumIncrement;
//                 ProcessTime->ExitTime = Process->ExitTime;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Process Debug Port */
//         case ProcessDebugPort:
//
//             if (ProcessInformationLength != sizeof(HANDLE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(HANDLE);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* Return whether or not we have a debug port */
//                 *(PHANDLE)ProcessInformation = (Process->DebugPort ?
//                                                 (HANDLE)-1 : NULL);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         case ProcessHandleCount:
//         {
//             ULONG HandleCount;
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Count the number of handles this process has */
//             HandleCount = ObGetProcessHandleCount(Process);
//
//             /* Protect write in SEH */
//             _SEH2_TRY
//             {
//                 /* Return the count of handles */
//                 *(PULONG)ProcessInformation = HandleCount;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Session ID for the process */
//         case ProcessSessionInformation:
//         {
//             PPROCESS_SESSION_INFORMATION SessionInfo = (PPROCESS_SESSION_INFORMATION)ProcessInformation;
//
//             if (ProcessInformationLength != sizeof(PROCESS_SESSION_INFORMATION))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length*/
//             Length = sizeof(PROCESS_SESSION_INFORMATION);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for write safety */
//             _SEH2_TRY
//             {
//                 /* Write back the Session ID */
//                 SessionInfo->SessionId = PsGetProcessSessionId(Process);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Virtual Memory Statistics */
//         case ProcessVmCounters:
//         {
//             PVM_COUNTERS VmCounters = (PVM_COUNTERS)ProcessInformation;
//
//             /* Validate the input length */
//             if ((ProcessInformationLength != sizeof(VM_COUNTERS)) &&
//                 (ProcessInformationLength != sizeof(VM_COUNTERS_EX)))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for write safety */
//             _SEH2_TRY
//             {
//                 /* Return data from EPROCESS */
//                 VmCounters->PeakVirtualSize = Process->PeakVirtualSize;
//                 VmCounters->VirtualSize = Process->VirtualSize;
//                 VmCounters->PageFaultCount = Process->Vm.PageFaultCount;
//                 VmCounters->PeakWorkingSetSize = Process->Vm.PeakWorkingSetSize;
//                 VmCounters->WorkingSetSize = Process->Vm.WorkingSetSize;
//                 VmCounters->QuotaPeakPagedPoolUsage = Process->QuotaPeak[PsPagedPool];
//                 VmCounters->QuotaPagedPoolUsage = Process->QuotaUsage[PsPagedPool];
//                 VmCounters->QuotaPeakNonPagedPoolUsage = Process->QuotaPeak[PsNonPagedPool];
//                 VmCounters->QuotaNonPagedPoolUsage = Process->QuotaUsage[PsNonPagedPool];
//                 VmCounters->PagefileUsage = Process->QuotaUsage[PsPageFile] << PAGE_SHIFT;
//                 VmCounters->PeakPagefileUsage = Process->QuotaPeak[PsPageFile] << PAGE_SHIFT;
//                 //VmCounters->PrivateUsage = Process->CommitCharge << PAGE_SHIFT;
//                 //
//
//                 /* Set the return length */
//                 Length = ProcessInformationLength;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Hard Error Processing Mode */
//         case ProcessDefaultHardErrorMode:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for writing back data */
//             _SEH2_TRY
//             {
//                 /* Write the current processing mode */
//                 *(PULONG)ProcessInformation = Process->
//                                               DefaultHardErrorProcessing;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         /* Priority Boosting status */
//         case ProcessPriorityBoost:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for writing back data */
//             _SEH2_TRY
//             {
//                 /* Return boost status */
//                 *(PULONG)ProcessInformation = Process->Pcb.DisableBoost ?
//                                               TRUE : FALSE;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         /* DOS Device Map */
//         case ProcessDeviceMap:
//         {
//             ULONG Flags;
//
//             if (ProcessInformationLength == sizeof(PROCESS_DEVICEMAP_INFORMATION_EX))
//             {
//                 /* Protect read in SEH */
//                 _SEH2_TRY
//                 {
//                     PPROCESS_DEVICEMAP_INFORMATION_EX DeviceMapEx = ProcessInformation;
//
//                     Flags = DeviceMapEx->Flags;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Get the exception code */
//                     Status = _SEH2_GetExceptionCode();
//                     _SEH2_YIELD(break);
//                 }
//                 _SEH2_END;
//
//                 /* Only one flag is supported and it needs LUID mappings */
//                 if ((Flags & ~PROCESS_LUID_DOSDEVICES_ONLY) != 0 ||
//                     !ObIsLUIDDeviceMapsEnabled())
//                 {
//                     Status = STATUS_INVALID_PARAMETER;
//                     break;
//                 }
//             }
//             else
//             {
//                 /* This has to be the size of the Query union field for x64 compatibility! */
//                 if (ProcessInformationLength != RTL_FIELD_SIZE(PROCESS_DEVICEMAP_INFORMATION, Query))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                     break;
//                 }
//
//                 /* No flags for standard call */
//                 Flags = 0;
//             }
//
//             /* Set the return length */
//             Length = ProcessInformationLength;
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Query the device map information */
//             Status = ObQueryDeviceMapInformation(Process,
//                                                  ProcessInformation,
//                                                  Flags);
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         /* Priority class */
//         case ProcessPriorityClass:
//         {
//             PPROCESS_PRIORITY_CLASS PsPriorityClass = (PPROCESS_PRIORITY_CLASS)ProcessInformation;
//
//             if (ProcessInformationLength != sizeof(PROCESS_PRIORITY_CLASS))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length*/
//             Length = sizeof(PROCESS_PRIORITY_CLASS);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for writing back data */
//             _SEH2_TRY
//             {
//                 /* Return current priority class */
//                 PsPriorityClass->PriorityClass = Process->PriorityClass;
//                 PsPriorityClass->Foreground = FALSE;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
//         case ProcessImageFileName:
//         {
//             PUNICODE_STRING ImageName;
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Get the image path */
//             Status = SeLocateProcessImageName(Process, &ImageName);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Set the return length */
//                 Length = ImageName->MaximumLength +
//                          sizeof(OBJECT_NAME_INFORMATION);
//
//                 /* Make sure it's large enough */
//                 if (Length <= ProcessInformationLength)
//                 {
//                     /* Enter SEH to protect write */
//                     _SEH2_TRY
//                     {
//                         /* Copy it */
//                         RtlCopyMemory(ProcessInformation,
//                                       ImageName,
//                                       Length);
//
//                         /* Update pointer */
//                         ((PUNICODE_STRING)ProcessInformation)->Buffer =
//                             (PWSTR)((PUNICODE_STRING)ProcessInformation + 1);
//                    }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         /* Get the exception code */
//                         Status = _SEH2_GetExceptionCode();
//                     }
//                     _SEH2_END;
//                 }
//                 else
//                 {
//                     /* Buffer too small */
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//
//                 /* Free the image path */
//                 ExFreePoolWithTag(ImageName, TAG_SEPA);
//             }
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//         }
//
// #if (NTDDI_VERSION >= NTDDI_VISTA) || (DLL_EXPORT_VERSION >= _WIN32_WINNT_VISTA)
//         case ProcessImageFileNameWin32:
//         {
//             PFILE_OBJECT FileObject;
//             POBJECT_NAME_INFORMATION ObjectNameInformation;
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION, // FIXME: Use PROCESS_QUERY_LIMITED_INFORMATION if implemented
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* Get the image path */
//             Status = PsReferenceProcessFilePointer(Process, &FileObject);
//             ObDereferenceObject(Process);
//             if (!NT_SUCCESS(Status))
//             {
//                 break;
//             }
//             Status = IoQueryFileDosDeviceName(FileObject, &ObjectNameInformation);
//             ObDereferenceObject(FileObject);
//             if (!NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* Determine return length and output */
//             Length = sizeof(UNICODE_STRING) + ObjectNameInformation->Name.MaximumLength;
//             if (Length <= ProcessInformationLength)
//             {
//                 _SEH2_TRY
//                 {
//                     PUNICODE_STRING ImageName = (PUNICODE_STRING)ProcessInformation;
//                     ImageName->Length = ObjectNameInformation->Name.Length;
//                     ImageName->MaximumLength = ObjectNameInformation->Name.MaximumLength;
//                     if (ObjectNameInformation->Name.MaximumLength)
//                     {
//                         ImageName->Buffer = (PWSTR)(ImageName + 1);
//                         RtlCopyMemory(ImageName->Buffer,
//                                       ObjectNameInformation->Name.Buffer,
//                                       ObjectNameInformation->Name.MaximumLength);
//                     }
//                     else
//                     {
//                         ASSERT(ImageName->Length == 0);
//                         ImageName->Buffer = NULL;
//                     }
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//             }
//             else
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             ExFreePool(ObjectNameInformation);
//
//             break;
//         }
// #endif /* (NTDDI_VERSION >= NTDDI_VISTA) || (DLL_EXPORT_VERSION >= _WIN32_WINNT_VISTA) */
//
//         case ProcessDebugFlags:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for writing back data */
//             _SEH2_TRY
//             {
//                 /* Return the debug flag state */
//                 *(PULONG)ProcessInformation = Process->NoDebugInherit ? 0 : 1;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         case ProcessBreakOnTermination:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Enter SEH for writing back data */
//             _SEH2_TRY
//             {
//                 /* Return the BreakOnTermination state */
//                 *(PULONG)ProcessInformation = Process->BreakOnTermination;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         /* Per-process security cookie */
//         case ProcessCookie:
//         {
//             ULONG Cookie;
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 /* Length size wrong, bail out */
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Get the current process and cookie */
//             Process = PsGetCurrentProcess();
//             Cookie = Process->Cookie;
//             if (!Cookie)
//             {
//                 LARGE_INTEGER SystemTime;
//                 ULONG NewCookie;
//                 PKPRCB Prcb;
//
//                 /* Generate a new cookie */
//                 KeQuerySystemTime(&SystemTime);
//                 Prcb = KeGetCurrentPrcb();
//                 NewCookie = Prcb->KeSystemCalls ^ Prcb->InterruptTime ^
//                             SystemTime.u.LowPart ^ SystemTime.u.HighPart;
//
//                 /* Set the new cookie or return the current one */
//                 Cookie = InterlockedCompareExchange((LONG*)&Process->Cookie,
//                                                     NewCookie,
//                                                     Cookie);
//                 if (!Cookie) Cookie = NewCookie;
//
//                 /* Set the return length */
//                 Length = sizeof(ULONG);
//             }
//
//             /* Indicate success */
//             Status = STATUS_SUCCESS;
//
//             /* Enter SEH to protect write */
//             _SEH2_TRY
//             {
//                 /* Write back the cookie */
//                 *(PULONG)ProcessInformation = Cookie;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//             break;
//         }
//
//         case ProcessImageInformation:
//
//             if (ProcessInformationLength != sizeof(SECTION_IMAGE_INFORMATION))
//             {
//                 /* Break out */
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the length required and validate it */
//             Length = sizeof(SECTION_IMAGE_INFORMATION);
//
//             /* Indicate success */
//             Status = STATUS_SUCCESS;
//
//             /* Enter SEH to protect write */
//             _SEH2_TRY
//             {
//                 MmGetImageInformation((PSECTION_IMAGE_INFORMATION)ProcessInformation);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//             break;
//
//         case ProcessDebugObjectHandle:
//         {
//             HANDLE DebugPort = NULL;
//
//             if (ProcessInformationLength != sizeof(HANDLE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(HANDLE);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Get the debug port. Continue even if this fails. */
//             Status = DbgkOpenProcessDebugPort(Process, PreviousMode, &DebugPort);
//
//             /* Let go of the process */
//             ObDereferenceObject(Process);
//
//             /* Protect write in SEH */
//             _SEH2_TRY
//             {
//                 /* Return debug port's handle */
//                 *(PHANDLE)ProcessInformation = DebugPort;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 if (DebugPort)
//                     ObCloseHandle(DebugPort, PreviousMode);
//
//                 /* Get the exception code.
//                  * Note: This overwrites any previous failure status. */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//             break;
//         }
//
//         case ProcessHandleTracing:
//             DPRINT1("Handle tracing not implemented: %lu\n", ProcessInformationClass);
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         case ProcessLUIDDeviceMapsEnabled:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             /* Indicate success */
//             Status = STATUS_SUCCESS;
//
//             /* Protect write in SEH */
//             _SEH2_TRY
//             {
//                 /* Query Ob */
//                 *(PULONG)ProcessInformation = ObIsLUIDDeviceMapsEnabled();
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//             break;
//
//         case ProcessWx86Information:
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Protect write in SEH */
//             _SEH2_TRY
//             {
//                 /* Return if the flag is set */
//                 *(PULONG)ProcessInformation = (ULONG)Process->VdmAllowed;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//             break;
//
//         case ProcessWow64Information:
//         {
//             ULONG_PTR Wow64 = 0;
//
//             if (ProcessInformationLength != sizeof(ULONG_PTR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG_PTR);
//
//             /* Reference the process */
//             Status = ObReferenceObjectByHandle(ProcessHandle,
//                                                PROCESS_QUERY_INFORMATION,
//                                                PsProcessType,
//                                                PreviousMode,
//                                                (PVOID*)&Process,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
// #ifdef _WIN64
//             /* Make sure the process isn't dying */
//             if (ExAcquireRundownProtection(&Process->RundownProtect))
//             {
//                 /* Get the WOW64 process structure */
//                 Wow64 = (ULONG_PTR)Process->Wow64Process;
//                 /* Release the lock */
//                 ExReleaseRundownProtection(&Process->RundownProtect);
//             }
// #endif
//
//             /* Dereference the process */
//             ObDereferenceObject(Process);
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* Return the Wow64 process information */
//                 *(PULONG_PTR)ProcessInformation = Wow64;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//             break;
//         }
//
//         case ProcessExecuteFlags:
//         {
//             ULONG ExecuteOptions = 0;
//
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             if (ProcessHandle != NtCurrentProcess())
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Get the options */
//             Status = MmGetExecuteOptions(&ExecuteOptions);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Protect write with SEH */
//                 _SEH2_TRY
//                 {
//                     /* Return them */
//                     *(PULONG)ProcessInformation = ExecuteOptions;
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     /* Get exception code */
//                     Status = _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//             }
//             break;
//         }
//
//         case ProcessLdtInformation:
//             DPRINT1("VDM/16-bit not implemented: %lu\n", ProcessInformationClass);
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         case ProcessWorkingSetWatch:
//             DPRINT1("WS Watch not implemented: %lu\n", ProcessInformationClass);
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         case ProcessPooledUsageAndLimits:
//             DPRINT1("Pool limits not implemented: %lu\n", ProcessInformationClass);
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         /* Not supported by Server 2003 */
//         default:
// #if DBG
//             DPRINT1("Unsupported info class: %s\n", PspDumpProcessInfoClassName(ProcessInformationClass));
// #endif
//             Status = STATUS_INVALID_INFO_CLASS;
//     }
//
//     /* Check if caller wants the return length and if there is one */
//     if (ReturnLength != NULL && Length != 0)
//     {
//         /* Protect write with SEH */
//         _SEH2_TRY
//         {
//             *ReturnLength = Length;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get exception code.
//              * Note: This overwrites any previous failure status. */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetInformationProcess(
//     _In_ HANDLE ProcessHandle,
//     _In_ PROCESSINFOCLASS ProcessInformationClass,
//     _In_reads_bytes_(ProcessInformationLength) PVOID ProcessInformation,
//     _In_ ULONG ProcessInformationLength)
// {
//     PEPROCESS Process;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     ACCESS_MASK Access;
//     NTSTATUS Status;
//     HANDLE PortHandle = NULL;
//     HANDLE TokenHandle = NULL;
//     HANDLE DirectoryHandle = NULL;
//     PROCESS_SESSION_INFORMATION SessionInfo = {0};
//     PROCESS_PRIORITY_CLASS PriorityClass = {0};
//     PROCESS_FOREGROUND_BACKGROUND Foreground = {0};
//     PVOID ExceptionPort;
//     ULONG Break;
//     KAFFINITY ValidAffinity, Affinity = 0;
//     KPRIORITY BasePriority = 0;
//     UCHAR MemoryPriority = 0;
//     BOOLEAN DisableBoost = 0;
//     ULONG DefaultHardErrorMode = 0;
//     ULONG DebugFlags = 0, EnableFixup = 0, Boost = 0;
//     ULONG NoExecute = 0, VdmPower = 0;
//     BOOLEAN HasPrivilege;
//     PLIST_ENTRY Next;
//     PETHREAD Thread;
//     PAGED_CODE();
//
//     /* Validate the information class */
//     Status = DefaultSetInfoBufferCheck(ProcessInformationClass,
//                                        PsProcessInfoClass,
//                                        RTL_NUMBER_OF(PsProcessInfoClass),
//                                        ProcessInformation,
//                                        ProcessInformationLength,
//                                        PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
// #if DBG
//         DPRINT1("NtSetInformationProcess(ProcessInformationClass: %s): Class validation failed! (Status: 0x%lx)\n",
//                 PspDumpProcessInfoClassName(ProcessInformationClass), Status);
// #endif
//         return Status;
//     }
//
//     /* Check what class this is */
//     Access = PROCESS_SET_INFORMATION;
//     if (ProcessInformationClass == ProcessSessionInformation)
//     {
//         /* Setting the Session ID needs a special mask */
//         Access |= PROCESS_SET_SESSIONID;
//     }
//     else if (ProcessInformationClass == ProcessExceptionPort)
//     {
//         /* Setting the exception port needs a special mask */
//         Access |= PROCESS_SUSPEND_RESUME;
//     }
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        Access,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check what kind of information class this is */
//     switch (ProcessInformationClass)
//     {
//         case ProcessWx86Information:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the boolean */
//                 VdmPower = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Getting VDM powers requires the SeTcbPrivilege */
//             if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 DPRINT1("Need TCB privilege\n");
//                 break;
//             }
//
//             /* Set or clear the flag */
//             if (VdmPower)
//             {
//                 PspSetProcessFlag(Process, PSF_VDM_ALLOWED_BIT);
//             }
//             else
//             {
//                 PspClearProcessFlag(Process, PSF_VDM_ALLOWED_BIT);
//             }
//             break;
//
//         /* Error/Exception Port */
//         case ProcessExceptionPort:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(HANDLE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the handle */
//                 PortHandle = *(PHANDLE)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Setting the error port requires the SeTcbPrivilege */
//             if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 break;
//             }
//
//             /* Get the LPC Port */
//             Status = ObReferenceObjectByHandle(PortHandle,
//                                                0,
//                                                LpcPortObjectType,
//                                                PreviousMode,
//                                                (PVOID)&ExceptionPort,
//                                                NULL);
//             if (!NT_SUCCESS(Status)) break;
//
//             /* Change the pointer */
//             if (InterlockedCompareExchangePointer(&Process->ExceptionPort,
//                                                   ExceptionPort,
//                                                   NULL))
//             {
//                 /* We already had one, fail */
//                 ObDereferenceObject(ExceptionPort);
//                 Status = STATUS_PORT_ALREADY_SET;
//             }
//             break;
//
//         /* Security Token */
//         case ProcessAccessToken:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(PROCESS_ACCESS_TOKEN))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Save the token handle */
//                 TokenHandle = ((PPROCESS_ACCESS_TOKEN)ProcessInformation)->
//                                Token;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Assign the actual token */
//             Status = PspSetPrimaryToken(Process, TokenHandle, NULL);
//             break;
//
//         /* Hard error processing */
//         case ProcessDefaultHardErrorMode:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 DefaultHardErrorMode = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Set the mode */
//             Process->DefaultHardErrorProcessing = DefaultHardErrorMode;
//
//             /* Call Ke for the update */
//             if (DefaultHardErrorMode & SEM_NOALIGNMENTFAULTEXCEPT)
//             {
//                 KeSetAutoAlignmentProcess(&Process->Pcb, TRUE);
//             }
//             else
//             {
//                 KeSetAutoAlignmentProcess(&Process->Pcb, FALSE);
//             }
//             Status = STATUS_SUCCESS;
//             break;
//
//         /* Session ID */
//         case ProcessSessionInformation:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(PROCESS_SESSION_INFORMATION))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the caller's buffer */
//                 SessionInfo = *(PPROCESS_SESSION_INFORMATION)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Setting the session id requires the SeTcbPrivilege */
//             if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 break;
//             }
//
//             /*
//              * Since we cannot change the session ID of the given
//              * process anymore because it is set once and for all
//              * at process creation time and because it is stored
//              * inside the Process->Session structure managed by MM,
//              * we fake changing it: we just return success if the
//              * user-defined value is the same as the session ID of
//              * the process, and otherwise we fail.
//              */
//             if (SessionInfo.SessionId == PsGetProcessSessionId(Process))
//             {
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 Status = STATUS_ACCESS_DENIED;
//             }
//
//             break;
//
//         case ProcessPriorityClass:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(PROCESS_PRIORITY_CLASS))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the caller's buffer */
//                 PriorityClass = *(PPROCESS_PRIORITY_CLASS)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Check for invalid PriorityClass value */
//             if (PriorityClass.PriorityClass > PROCESS_PRIORITY_CLASS_ABOVE_NORMAL)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             if ((PriorityClass.PriorityClass != Process->PriorityClass) &&
//                 (PriorityClass.PriorityClass == PROCESS_PRIORITY_CLASS_REALTIME))
//             {
//                 /* Check the privilege */
//                 HasPrivilege = SeCheckPrivilegedObject(SeIncreaseBasePriorityPrivilege,
//                                                        ProcessHandle,
//                                                        PROCESS_SET_INFORMATION,
//                                                        PreviousMode);
//                 if (!HasPrivilege)
//                 {
//                     ObDereferenceObject(Process);
//                     DPRINT1("Privilege to change priority to realtime lacking\n");
//                     return STATUS_PRIVILEGE_NOT_HELD;
//                 }
//             }
//
//             /* Check if we have a job */
//             if (Process->Job)
//             {
//                 DPRINT1("Jobs not yet supported\n");
//             }
//
//             /* Set process priority class */
//             Process->PriorityClass = PriorityClass.PriorityClass;
//
//             /* Set process priority mode (foreground or background) */
//             PsSetProcessPriorityByClass(Process,
//                                         PriorityClass.Foreground ?
//                                         PsProcessPriorityForeground :
//                                         PsProcessPriorityBackground);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case ProcessForegroundInformation:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(PROCESS_FOREGROUND_BACKGROUND))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the caller's buffer */
//                 Foreground = *(PPROCESS_FOREGROUND_BACKGROUND)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Set process priority mode (foreground or background) */
//             PsSetProcessPriorityByClass(Process,
//                                         Foreground.Foreground ?
//                                         PsProcessPriorityForeground :
//                                         PsProcessPriorityBackground);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case ProcessBasePriority:
//
//             /* Validate input length */
//             if (ProcessInformationLength != sizeof(KPRIORITY))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 BasePriority = *(KPRIORITY*)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Extract the memory priority out of there */
//             if (BasePriority & 0x80000000)
//             {
//                 MemoryPriority = MEMORY_PRIORITY_FOREGROUND;
//                 BasePriority &= ~0x80000000;
//             }
//             else
//             {
//                 MemoryPriority = MEMORY_PRIORITY_BACKGROUND;
//             }
//
//             /* Validate the number */
//             if ((BasePriority > HIGH_PRIORITY) || (BasePriority <= LOW_PRIORITY))
//             {
//                 ObDereferenceObject(Process);
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Check if the new base is higher */
//             if (BasePriority > Process->Pcb.BasePriority)
//             {
//                 HasPrivilege = SeCheckPrivilegedObject(SeIncreaseBasePriorityPrivilege,
//                                                        ProcessHandle,
//                                                        PROCESS_SET_INFORMATION,
//                                                        PreviousMode);
//                 if (!HasPrivilege)
//                 {
//                     ObDereferenceObject(Process);
//                     DPRINT1("Privilege to change priority from %lx to %lx lacking\n", Process->Pcb.BasePriority, BasePriority);
//                     return STATUS_PRIVILEGE_NOT_HELD;
//                 }
//             }
//
//             /* Call Ke */
//             KeSetPriorityAndQuantumProcess(&Process->Pcb, BasePriority, 0);
//
//             /* Now set the memory priority */
//             MmSetMemoryPriorityProcess(Process, MemoryPriority);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case ProcessRaisePriority:
//
//             /* Validate input length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 Boost = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Make sure the process isn't dying */
//             if (ExAcquireRundownProtection(&Process->RundownProtect))
//             {
//                 /* Lock it */
//                 KeEnterCriticalRegion();
//                 ExAcquirePushLockShared(&Process->ProcessLock);
//
//                 /* Loop the threads */
//                 for (Next = Process->ThreadListHead.Flink;
//                      Next != &Process->ThreadListHead;
//                      Next = Next->Flink)
//                 {
//                     /* Call Ke for the thread */
//                     Thread = CONTAINING_RECORD(Next, ETHREAD, ThreadListEntry);
//                     KeBoostPriorityThread(&Thread->Tcb, Boost);
//                 }
//
//                 /* Release the lock and rundown */
//                 ExReleasePushLockShared(&Process->ProcessLock);
//                 KeLeaveCriticalRegion();
//                 ExReleaseRundownProtection(&Process->RundownProtect);
//
//                 /* Set success code */
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* Avoid race conditions */
//                 Status = STATUS_PROCESS_IS_TERMINATING;
//             }
//             break;
//
//         case ProcessBreakOnTermination:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 Break = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Setting 'break on termination' requires the SeDebugPrivilege */
//             if (!SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 break;
//             }
//
//             /* Set or clear the flag */
//             if (Break)
//             {
//                 PspSetProcessFlag(Process, PSF_BREAK_ON_TERMINATION_BIT);
//             }
//             else
//             {
//                 PspClearProcessFlag(Process, PSF_BREAK_ON_TERMINATION_BIT);
//             }
//
//             break;
//
//         case ProcessAffinityMask:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(KAFFINITY))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 Affinity = *(PKAFFINITY)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Make sure it's valid for the CPUs present */
//             ValidAffinity = Affinity & KeActiveProcessors;
//             if (!Affinity || (ValidAffinity != Affinity))
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Check if it's within job affinity limits */
//             if (Process->Job)
//             {
//                 /* Not yet implemented */
//                 UNIMPLEMENTED;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//             }
//
//             /* Make sure the process isn't dying */
//             if (ExAcquireRundownProtection(&Process->RundownProtect))
//             {
//                 /* Lock it */
//                 KeEnterCriticalRegion();
//                 ExAcquirePushLockShared(&Process->ProcessLock);
//
//                 /* Call Ke to do the work */
//                 KeSetAffinityProcess(&Process->Pcb, ValidAffinity);
//
//                 /* Release the lock and rundown */
//                 ExReleasePushLockShared(&Process->ProcessLock);
//                 KeLeaveCriticalRegion();
//                 ExReleaseRundownProtection(&Process->RundownProtect);
//
//                 /* Set success code */
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* Avoid race conditions */
//                 Status = STATUS_PROCESS_IS_TERMINATING;
//             }
//             break;
//
//         /* Priority Boosting status */
//         case ProcessPriorityBoost:
//
//             /* Validate input length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 DisableBoost = *(PBOOLEAN)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Make sure the process isn't dying */
//             if (ExAcquireRundownProtection(&Process->RundownProtect))
//             {
//                 /* Lock it */
//                 KeEnterCriticalRegion();
//                 ExAcquirePushLockShared(&Process->ProcessLock);
//
//                 /* Call Ke to do the work */
//                 KeSetDisableBoostProcess(&Process->Pcb, DisableBoost);
//
//                 /* Loop the threads too */
//                 for (Next = Process->ThreadListHead.Flink;
//                      Next != &Process->ThreadListHead;
//                      Next = Next->Flink)
//                 {
//                     /* Call Ke for the thread */
//                     Thread = CONTAINING_RECORD(Next, ETHREAD, ThreadListEntry);
//                     KeSetDisableBoostThread(&Thread->Tcb, DisableBoost);
//                 }
//
//                 /* Release the lock and rundown */
//                 ExReleasePushLockShared(&Process->ProcessLock);
//                 KeLeaveCriticalRegion();
//                 ExReleaseRundownProtection(&Process->RundownProtect);
//
//                 /* Set success code */
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* Avoid race conditions */
//                 Status = STATUS_PROCESS_IS_TERMINATING;
//             }
//             break;
//
//         case ProcessDebugFlags:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 DebugFlags = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Set the mode */
//             if (DebugFlags & ~1)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//             else
//             {
//                 if (DebugFlags & 1)
//                 {
//                     PspClearProcessFlag(Process, PSF_NO_DEBUG_INHERIT_BIT);
//                 }
//                 else
//                 {
//                     PspSetProcessFlag(Process, PSF_NO_DEBUG_INHERIT_BIT);
//                 }
//             }
//
//             /* Done */
//             Status = STATUS_SUCCESS;
//             break;
//
//         case ProcessEnableAlignmentFaultFixup:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(BOOLEAN))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 EnableFixup = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Set the mode */
//             if (EnableFixup)
//             {
//                 Process->DefaultHardErrorProcessing |= SEM_NOALIGNMENTFAULTEXCEPT;
//             }
//             else
//             {
//                 Process->DefaultHardErrorProcessing &= ~SEM_NOALIGNMENTFAULTEXCEPT;
//             }
//
//             /* Call Ke for the update */
//             KeSetAutoAlignmentProcess(&Process->Pcb, FALSE);
//             Status = STATUS_SUCCESS;
//             break;
//
//         case ProcessUserModeIOPL:
//
//             /* Only TCB can do this */
//             if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 DPRINT1("Need TCB to set IOPL\n");
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 break;
//             }
//
//             /* Only supported on x86 */
// #if defined (_X86_)
//             Ke386SetIOPL();
// #elif defined(_M_AMD64)
//             /* On x64 this function isn't implemented.
//                On Windows 2003 it returns success.
//                On Vista+ it returns STATUS_NOT_IMPLEMENTED. */
//             if ((ExGetPreviousMode() != KernelMode) &&
//                 (RtlRosGetAppcompatVersion() > _WIN32_WINNT_WS03))
//             {
//                 Status = STATUS_NOT_IMPLEMENTED;
//             }
// #else
//             Status = STATUS_NOT_IMPLEMENTED;
// #endif
//             /* Done */
//             break;
//
//         case ProcessExecuteFlags:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             if (ProcessHandle != NtCurrentProcess())
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 NoExecute = *(PULONG)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Call Mm for the update */
//             Status = MmSetExecuteOptions(NoExecute);
//             break;
//
//         case ProcessDeviceMap:
//
//             /* Check buffer length */
//             if (ProcessInformationLength != sizeof(HANDLE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Capture the handle */
//                 DirectoryHandle = *(PHANDLE)ProcessInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Call Ob to set the device map */
//             Status = ObSetDeviceMap(Process, DirectoryHandle);
//             break;
//
//
//         /* We currently don't implement any of these */
//         case ProcessLdtInformation:
//         case ProcessLdtSize:
//         case ProcessIoPortHandlers:
//              DPRINT1("VDM/16-bit Request not implemented: %lu\n", ProcessInformationClass);
//              Status = STATUS_NOT_IMPLEMENTED;
//              break;
//
//         case ProcessQuotaLimits:
//
//             Status = PspSetQuotaLimits(Process,
//                                      1,
//                                      ProcessInformation,
//                                      ProcessInformationLength,
//                                      PreviousMode);
//             break;
//
//         case ProcessWorkingSetWatch:
//             DPRINT1("WS watch not implemented\n");
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         case ProcessHandleTracing:
//             DPRINT1("Handle tracing not implemented\n");
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         /* Anything else is invalid */
//         default:
// #if DBG
//             DPRINT1("Invalid Server 2003 Info Class: %s\n", PspDumpProcessInfoClassName(ProcessInformationClass));
// #endif
//             Status = STATUS_INVALID_INFO_CLASS;
//     }
//
//     /* Dereference and return status */
//     ObDereferenceObject(Process);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetInformationThread(
//     _In_ HANDLE ThreadHandle,
//     _In_ THREADINFOCLASS ThreadInformationClass,
//     _In_reads_bytes_(ThreadInformationLength) PVOID ThreadInformation,
//     _In_ ULONG ThreadInformationLength)
// {
//     PETHREAD Thread;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     HANDLE TokenHandle = NULL;
//     KPRIORITY Priority = 0;
//     KAFFINITY Affinity = 0, CombinedAffinity;
//     PVOID Address = NULL;
//     PEPROCESS Process;
//     ULONG_PTR DisableBoost = 0;
//     ULONG_PTR IdealProcessor = 0;
//     ULONG_PTR Break = 0;
//     PTEB Teb;
//     ULONG_PTR TlsIndex = 0;
//     PVOID *ExpansionSlots;
//     PETHREAD ProcThread;
//     BOOLEAN HasPrivilege;
//     PAGED_CODE();
//
//     /* Validate the information class */
//     Status = DefaultSetInfoBufferCheck(ThreadInformationClass,
//                                        PsThreadInfoClass,
//                                        RTL_NUMBER_OF(PsThreadInfoClass),
//                                        ThreadInformation,
//                                        ThreadInformationLength,
//                                        PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
// #if DBG
//         DPRINT1("NtSetInformationThread(ThreadInformationClass: %s): Class validation failed! (Status: 0x%lx)\n",
//                 PspDumpThreadInfoClassName(ThreadInformationClass), Status);
// #endif
//         return Status;
//     }
//
//     /* Check what kind of information class this is */
//     switch (ThreadInformationClass)
//     {
//         /* Thread priority */
//         case ThreadPriority:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(KPRIORITY))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 Priority = *(PLONG)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Validate it */
//             if ((Priority > HIGH_PRIORITY) ||
//                 (Priority <= LOW_PRIORITY))
//             {
//                 /* Fail */
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Check for the required privilege */
//             if (Priority >= LOW_REALTIME_PRIORITY)
//             {
//                 HasPrivilege = SeCheckPrivilegedObject(SeIncreaseBasePriorityPrivilege,
//                                                        ThreadHandle,
//                                                        THREAD_SET_INFORMATION,
//                                                        PreviousMode);
//                 if (!HasPrivilege)
//                 {
//                     DPRINT1("Privilege to change priority to %lx lacking\n", Priority);
//                     return STATUS_PRIVILEGE_NOT_HELD;
//                 }
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set the priority */
//             KeSetPriorityThread(&Thread->Tcb, Priority);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadBasePriority:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(LONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 Priority = *(PLONG)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Validate it */
//             if ((Priority > THREAD_BASE_PRIORITY_MAX) ||
//                 (Priority < THREAD_BASE_PRIORITY_MIN))
//             {
//                 /* These ones are OK */
//                 if ((Priority != THREAD_BASE_PRIORITY_LOWRT + 1) &&
//                     (Priority != THREAD_BASE_PRIORITY_IDLE - 1))
//                 {
//                     /* Check if the process is real time */
//                     if (PsGetCurrentProcess()->PriorityClass !=
//                         PROCESS_PRIORITY_CLASS_REALTIME)
//                     {
//                         /* It isn't, fail */
//                         Status = STATUS_INVALID_PARAMETER;
//                         break;
//                     }
//                 }
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set the base priority */
//             KeSetBasePriorityThread(&Thread->Tcb, Priority);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadAffinityMask:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG_PTR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 Affinity = *(PULONG_PTR)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Validate it */
//             if (!Affinity)
//             {
//                 /* Fail */
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Get the process */
//             Process = Thread->ThreadsProcess;
//
//             /* Try to acquire rundown */
//             if (ExAcquireRundownProtection(&Process->RundownProtect))
//             {
//                 /* Lock it */
//                 KeEnterCriticalRegion();
//                 ExAcquirePushLockShared(&Process->ProcessLock);
//
//                 /* Combine masks */
//                 CombinedAffinity = Affinity & Process->Pcb.Affinity;
//                 if (CombinedAffinity != Affinity)
//                 {
//                     /* Fail */
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//                 else
//                 {
//                     /* Set the affinity */
//                     KeSetAffinityThread(&Thread->Tcb, CombinedAffinity);
//                 }
//
//                 /* Release the lock and rundown */
//                 ExReleasePushLockShared(&Process->ProcessLock);
//                 KeLeaveCriticalRegion();
//                 ExReleaseRundownProtection(&Process->RundownProtect);
//             }
//             else
//             {
//                 /* Too late */
//                 Status = STATUS_PROCESS_IS_TERMINATING;
//             }
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadImpersonationToken:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(HANDLE))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Save the token handle */
//                 TokenHandle = *(PHANDLE)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_THREAD_TOKEN,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Assign the actual token */
//             Status = PsAssignImpersonationToken(Thread, TokenHandle);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadQuerySetWin32StartAddress:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG_PTR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 Address = *(PVOID*)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set the address */
//             Thread->Win32StartAddress = Address;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadIdealProcessor:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG_PTR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 IdealProcessor = *(PULONG_PTR)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Validate it */
//             if (IdealProcessor > MAXIMUM_PROCESSORS)
//             {
//                 /* Fail */
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set the ideal */
//             Status = KeSetIdealProcessorThread(&Thread->Tcb,
//                                                (CCHAR)IdealProcessor);
//
//             /* Get the TEB and protect the thread */
//             Teb = Thread->Tcb.Teb;
//             if ((Teb) && (ExAcquireRundownProtection(&Thread->RundownProtect)))
//             {
//                 /* Save the ideal processor */
//                 Teb->IdealProcessor = Thread->Tcb.IdealProcessor;
//
//                 /* Release rundown protection */
//                 ExReleaseRundownProtection(&Thread->RundownProtect);
//             }
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadPriorityBoost:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG_PTR))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 DisableBoost = *(PULONG_PTR)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Call the kernel */
//             KeSetDisableBoostThread(&Thread->Tcb, (BOOLEAN)DisableBoost);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadZeroTlsCell:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Use SEH for capture */
//             _SEH2_TRY
//             {
//                 /* Get the priority */
//                 TlsIndex = *(PULONG)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* This is only valid for the current thread */
//             if (Thread != PsGetCurrentThread())
//             {
//                 /* Fail */
//                 Status = STATUS_INVALID_PARAMETER;
//                 ObDereferenceObject(Thread);
//                 break;
//             }
//
//             /* Get the process */
//             Process = Thread->ThreadsProcess;
//
//             /* Loop the threads */
//             ProcThread = PsGetNextProcessThread(Process, NULL);
//             while (ProcThread)
//             {
//                 /* Acquire rundown */
//                 if (ExAcquireRundownProtection(&ProcThread->RundownProtect))
//                 {
//                     /* Get the TEB */
//                     Teb = ProcThread->Tcb.Teb;
//                     if (Teb)
//                     {
//                         /* Check if we're in the expansion range */
//                         if (TlsIndex > TLS_MINIMUM_AVAILABLE - 1)
//                         {
//                             if (TlsIndex < (TLS_MINIMUM_AVAILABLE +
//                                             TLS_EXPANSION_SLOTS) - 1)
//                             {
//                                 /* Check if we have expansion slots */
//                                 ExpansionSlots = Teb->TlsExpansionSlots;
//                                 if (ExpansionSlots)
//                                 {
//                                     /* Clear the index */
//                                     ExpansionSlots[TlsIndex - TLS_MINIMUM_AVAILABLE] = 0;
//                                 }
//                             }
//                         }
//                         else
//                         {
//                             /* Clear the index */
//                             Teb->TlsSlots[TlsIndex] = NULL;
//                         }
//                     }
//
//                     /* Release rundown */
//                     ExReleaseRundownProtection(&ProcThread->RundownProtect);
//                 }
//
//                 /* Go to the next thread */
//                 ProcThread = PsGetNextProcessThread(Process, ProcThread);
//             }
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadBreakOnTermination:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != sizeof(ULONG))
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Enter SEH for direct buffer read */
//             _SEH2_TRY
//             {
//                 Break = *(PULONG)ThreadInformation;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Break = 0;
//                 Status = _SEH2_GetExceptionCode();
//                 _SEH2_YIELD(break);
//             }
//             _SEH2_END;
//
//             /* Setting 'break on termination' requires the SeDebugPrivilege */
//             if (!SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode))
//             {
//                 /* We don't hold the privilege, bail out */
//                 Status = STATUS_PRIVILEGE_NOT_HELD;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set or clear the flag */
//             if (Break)
//             {
//                 PspSetCrossThreadFlag(Thread, CT_BREAK_ON_TERMINATION_BIT);
//             }
//             else
//             {
//                 PspClearCrossThreadFlag(Thread, CT_BREAK_ON_TERMINATION_BIT);
//             }
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadHideFromDebugger:
//
//             /* Check buffer length */
//             if (ThreadInformationLength != 0)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                THREAD_SET_INFORMATION,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Set the flag */
//             PspSetCrossThreadFlag(Thread, CT_HIDE_FROM_DEBUGGER_BIT);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         /* Anything else */
//         default:
//             /* Not yet implemented */
// #if DBG
//             DPRINT1("Not implemented: %s\n", PspDumpThreadInfoClassName(ThreadInformationClass));
// #endif
//             Status = STATUS_NOT_IMPLEMENTED;
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryInformationThread(
//     _In_ HANDLE ThreadHandle,
//     _In_ THREADINFOCLASS ThreadInformationClass,
//     _Out_writes_bytes_to_opt_(ThreadInformationLength, *ReturnLength)
//         PVOID ThreadInformation,
//     _In_ ULONG ThreadInformationLength,
//     _Out_opt_ PULONG ReturnLength)
// {
//     PETHREAD Thread;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     ULONG Access;
//     ULONG Length = 0;
//     PTHREAD_BASIC_INFORMATION ThreadBasicInfo =
//         (PTHREAD_BASIC_INFORMATION)ThreadInformation;
//     PKERNEL_USER_TIMES ThreadTime = (PKERNEL_USER_TIMES)ThreadInformation;
//     KIRQL OldIrql;
//     ULONG ThreadTerminated;
//     PAGED_CODE();
//
//     /* Validate the information class */
//     Status = DefaultQueryInfoBufferCheck(ThreadInformationClass,
//                                          PsThreadInfoClass,
//                                          RTL_NUMBER_OF(PsThreadInfoClass),
//                                          ICIF_PROBE_READ,
//                                          ThreadInformation,
//                                          ThreadInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status))
//     {
// #if DBG
//         DPRINT1("NtQueryInformationThread(ThreadInformationClass: %s): Class validation failed! (Status: 0x%lx)\n",
//                 PspDumpThreadInfoClassName(ThreadInformationClass), Status);
// #endif
//         return Status;
//     }
//
//     /* Check what class this is */
//     Access = THREAD_QUERY_INFORMATION;
//
//     /* Check what kind of information class this is */
//     switch (ThreadInformationClass)
//     {
//         /* Basic thread information */
//         case ThreadBasicInformation:
//
//             /* Set the return length */
//             Length = sizeof(THREAD_BASIC_INFORMATION);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Protect writes with SEH */
//             _SEH2_TRY
//             {
//                 /* Write all the information from the ETHREAD/KTHREAD */
//                 ThreadBasicInfo->ExitStatus = Thread->ExitStatus;
//                 ThreadBasicInfo->TebBaseAddress = (PVOID)Thread->Tcb.Teb;
//                 ThreadBasicInfo->ClientId = Thread->Cid;
//                 ThreadBasicInfo->AffinityMask = Thread->Tcb.Affinity;
//                 ThreadBasicInfo->Priority = Thread->Tcb.Priority;
//                 ThreadBasicInfo->BasePriority = KeQueryBasePriorityThread(&Thread->Tcb);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         /* Thread time information */
//         case ThreadTimes:
//
//             /* Set the return length */
//             Length = sizeof(KERNEL_USER_TIMES);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Protect writes with SEH */
//             _SEH2_TRY
//             {
//                 /* Copy time information from ETHREAD/KTHREAD */
//                 ThreadTime->KernelTime.QuadPart = Thread->Tcb.KernelTime * KeMaximumIncrement;
//                 ThreadTime->UserTime.QuadPart = Thread->Tcb.UserTime * KeMaximumIncrement;
//                 ThreadTime->CreateTime = Thread->CreateTime;
//
//                 /* Exit time is in a union and only valid on actual exit! */
//                 if (KeReadStateThread(&Thread->Tcb))
//                 {
//                     ThreadTime->ExitTime = Thread->ExitTime;
//                 }
//                 else
//                 {
//                     ThreadTime->ExitTime.QuadPart = 0;
//                 }
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadQuerySetWin32StartAddress:
//
//             /* Set the return length*/
//             Length = sizeof(PVOID);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* Return the Win32 Start Address */
//                 *(PVOID*)ThreadInformation = Thread->Win32StartAddress;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadPerformanceCount:
//
//             /* Set the return length*/
//             Length = sizeof(LARGE_INTEGER);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* FIXME */
//                 (*(PLARGE_INTEGER)ThreadInformation).QuadPart = 0;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadAmILastThread:
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* Return whether or not we are the last thread */
//                 *(PULONG)ThreadInformation = ((Thread->ThreadsProcess->
//                                                ThreadListHead.Flink->Flink ==
//                                                &Thread->ThreadsProcess->
//                                                ThreadListHead) ?
//                                               TRUE : FALSE);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadIsIoPending:
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Raise the IRQL to protect the IRP list */
//             KeRaiseIrql(APC_LEVEL, &OldIrql);
//
//             /* Protect write with SEH */
//             _SEH2_TRY
//             {
//                 /* Check if the IRP list is empty or not */
//                 *(PULONG)ThreadInformation = !IsListEmpty(&Thread->IrpList);
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Lower IRQL back */
//             KeLowerIrql(OldIrql);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         /* LDT and GDT information */
//         case ThreadDescriptorTableEntry:
//
// #if defined(_X86_)
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             /* Call the worker routine */
//             Status = PspQueryDescriptorThread(Thread,
//                                               ThreadInformation,
//                                               ThreadInformationLength,
//                                               ReturnLength);
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
// #else
//             /* Only implemented on x86 */
//             Status = STATUS_NOT_IMPLEMENTED;
// #endif
//             break;
//
//         case ThreadPriorityBoost:
//
//             /* Set the return length*/
//             Length = sizeof(ULONG);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             _SEH2_TRY
//             {
//                 *(PULONG)ThreadInformation = Thread->Tcb.DisableBoost ? 1 : 0;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadBreakOnTermination:
//
//             /* Set the return length */
//             Length = sizeof(ULONG);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             _SEH2_TRY
//             {
//                 *(PULONG)ThreadInformation = Thread->BreakOnTermination;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         case ThreadIsTerminated:
//
//             /* Set the return length*/
//             Length = sizeof(ThreadTerminated);
//
//             if (ThreadInformationLength != Length)
//             {
//                 Status = STATUS_INFO_LENGTH_MISMATCH;
//                 break;
//             }
//
//             /* Reference the thread */
//             Status = ObReferenceObjectByHandle(ThreadHandle,
//                                                Access,
//                                                PsThreadType,
//                                                PreviousMode,
//                                                (PVOID*)&Thread,
//                                                NULL);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             ThreadTerminated = PsIsThreadTerminating(Thread);
//
//             _SEH2_TRY
//             {
//                 *(PULONG)ThreadInformation = ThreadTerminated ? 1 : 0;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             /* Dereference the thread */
//             ObDereferenceObject(Thread);
//             break;
//
//         /* Anything else */
//         default:
//             /* Not yet implemented */
// #if DBG
//             DPRINT1("Not implemented: %s\n", PspDumpThreadInfoClassName(ThreadInformationClass));
// #endif
//             Status = STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* Protect write with SEH */
//     _SEH2_TRY
//     {
//         /* Check if caller wanted return length */
//         if (ReturnLength) *ReturnLength = Length;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
