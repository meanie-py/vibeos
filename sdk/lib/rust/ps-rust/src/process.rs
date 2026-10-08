use ntkrnl_sys::ddk::*;
// ke globals across the crate boundary as link symbols (#[no_mangle] in
// ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeActiveProcessors: ntkrnl_sys::ddk::KAFFINITY;
}
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::mm::PsInitialSystemProcess;
use ntkrnl_sys::probe::{probe_write_user, probe_user_buffer};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Status codes not in ddk
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as i32;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as i32;
const STATUS_INVALID_PARAMETER_MIX: NTSTATUS = 0xC0000030_u32 as i32;
const STATUS_PENDING: NTSTATUS = 0x00000103;
const STATUS_PROCESS_IS_TERMINATING: NTSTATUS = 0xC000010A_u32 as i32;
const STATUS_INVALID_CID: NTSTATUS = 0xC000000B_u32 as i32;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001_u32 as i32;

/// ObReferenceObject -- forwarding to ObfReferenceObject.
#[inline]
unsafe fn ObReferenceObject(object: PVOID) {
    #[cfg(target_arch = "x86")]
    extern "fastcall" {
        fn ObfReferenceObject(Object: PVOID);
    }
    #[cfg(not(target_arch = "x86"))]
    extern "C" {
        fn ObfReferenceObject(Object: PVOID);
    }
    ObfReferenceObject(object);
}

/// ObDereferenceObject -- forwarding to ObfDereferenceObject.
#[inline]
unsafe fn ObDereferenceObject(object: PVOID) {
    #[cfg(target_arch = "x86")]
    extern "fastcall" {
        fn ObfDereferenceObject(Object: PVOID);
    }
    #[cfg(not(target_arch = "x86"))]
    extern "C" {
        fn ObfDereferenceObject(Object: PVOID);
    }
    ObfDereferenceObject(object);
}

/// Initialize a LIST_ENTRY as an empty list head (ddk-typed).
#[inline]
unsafe fn InitializeListHead(head: PLIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}

/// Insert entry at tail of list (ddk-typed).
#[inline]
unsafe fn InsertTailList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;
}

/// IoGetCurrentProcess wrapper returning ddk PEPROCESS.
#[inline]
unsafe fn PsGetCurrentProcess() -> PEPROCESS {
    extern "system" {
        #[link_name = "IoGetCurrentProcess"]
        fn IoGetCurrentProcess() -> PEPROCESS;
    }
    IoGetCurrentProcess()
}

/// Inline InterlockedOr -- atomic OR on LONG, returns old value.
#[inline]
unsafe fn InterlockedOr(target: *mut LONG, value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.fetch_or(value, Ordering::SeqCst)
}

/// ProcessObject dispatcher type value.
const ProcessObject: u8 = 3;
/// ThreadObject dispatcher type value.
const ThreadObject: u8 = 6;

extern "system" {
    static mut PsMinimumWorkingSet: ULONG;
    static mut PsMaximumWorkingSet: ULONG;
    static mut PspCidTable: PHANDLE_TABLE;
    static mut PspUseJobSchedulingClasses: BOOLEAN;
    static mut PspJobSchedulingClasses: [UCHAR; 10];
    static mut ProcessCount: ULONG;
    static mut SeDebugPrivilege: LUID;
    static DbgkDebugObjectType: POBJECT_TYPE;
    static MmSectionObjectType: POBJECT_TYPE;
    static LpcPortObjectType: POBJECT_TYPE;

    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn PspInheritQuota(process: PEPROCESS, parent: PEPROCESS);
    fn PspInitializeProcessSecurity(process: PEPROCESS, parent: PEPROCESS) -> NTSTATUS;
    fn ObInitProcess(parent: PEPROCESS, process: PEPROCESS) -> NTSTATUS;
    fn PspMapSystemDll(process: PEPROCESS, base: *mut PVOID, load: BOOLEAN) -> NTSTATUS;
}
extern "C" {
    fn PspPrioritySeparationFromMask(mask: ULONG) -> ULONG;
    fn PspQuantumTypeFromMask(mask: ULONG) -> ULONG;
    fn PspQuantumLengthFromMask(mask: ULONG) -> ULONG;
}
extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}
extern "C" {
    fn ObpValidateAttributes(attrs: ULONG, mode: KPROCESSOR_MODE) -> ULONG;
}

// FORCEINLINE functions provided by ps_rust_shims.c as __cdecl
extern "C" {
    fn Ki386PerfEnd();
    fn PspRunCreateProcessNotifyRoutines(Process: PEPROCESS, Create: BOOLEAN);
}
extern "C" {
    fn ARM64_UART_TRACE(msg: *const i8);
}
extern "C" {
    fn ARM64_UART_TRACE_HEX(msg: *const i8, val: ULONG_PTR);
}
extern "system" {
    fn KeQuerySystemTime(time: *mut LARGE_INTEGER);
}

const PSP_VARIABLE_QUANTUMS: ULONG = 0x4;
const PSP_FIXED_QUANTUMS: ULONG = 0x8;
const PSP_LONG_QUANTUMS: ULONG = 0x10;
const PSP_SHORT_QUANTUMS: ULONG = 0x20;

const MEMORY_PRIORITY_FOREGROUND: UCHAR = 0x02;
const MEMORY_PRIORITY_BACKGROUND: UCHAR = 0x0;

const PROCESS_CREATE_FLAGS_BREAKAWAY: ULONG = 0x00000001;
const PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT: ULONG = 0x00000002;
const PROCESS_CREATE_FLAGS_INHERIT_HANDLES: ULONG = 0x00000004;
const PROCESS_CREATE_FLAGS_LARGE_PAGES: ULONG = 0x00000010;
// ndk/pstypes.h:93: also includes PROCESS_CREATE_FLAGS_OVERRIDE_ADDRESS_SPACE (0x8).
const PROCESS_CREATE_FLAGS_LEGAL_MASK: ULONG = 0x0000001F;

const PSF_NO_DEBUG_INHERIT_BIT: LONG = 0x00000002;
const PSF_HAS_ADDRESS_SPACE_BIT: LONG = 0x00040000;
const PSF_PROCESS_DELETE_BIT: LONG = 0x00000008;

const PROCESS_PRIORITY_CLASS_IDLE: ULONG = 1;
const PROCESS_PRIORITY_CLASS_NORMAL: ULONG = 2;
const PROCESS_PRIORITY_CLASS_BELOW_NORMAL: ULONG = 5;
const PROCESS_PRIORITY_CLASS_ABOVE_NORMAL: ULONG = 6;

const SEM_FAILCRITICALERRORS: ULONG = 0x0001;
const SEM_NOALIGNMENTFAULTEXCEPT: ULONG = 0x0004;

const TAG_SEPA: ULONG = 0x61506553;

const PsProcessPriorityBackground: ULONG = 0x01;
const PsProcessPriorityForeground: ULONG = 0x00;
const PsProcessPrioritySpinning: ULONG = 2;

#[no_mangle]
pub static mut PsProcessType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut PsActiveProcessHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut PspActiveProcessMutex: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ShortPsLockDelay: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut PsRawPrioritySeparation: ULONG = 0;

// Registry-controlled values owned by ps/process. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static PROCESS_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("PriorityControl", "Win32PrioritySeparation",
            core::ptr::addr_of_mut!(PsRawPrioritySeparation)),
    ] });

#[no_mangle]
pub static mut PsPrioritySeparation: ULONG = 0;

#[no_mangle]
pub static mut PspForegroundQuantum: [i8; 3] = [0; 3];

#[no_mangle]
pub static mut PspFixedQuantums: [i8; 6] = [
    3 * 6,
    3 * 6,
    3 * 6,
    6 * 6,
    6 * 6,
    6 * 6,
];

#[no_mangle]
pub static mut PspVariableQuantums: [i8; 6] = [
    1 * 6,
    2 * 6,
    3 * 6,
    2 * 6,
    4 * 6,
    6 * 6,
];

#[no_mangle]
pub static mut PspPriorityTable: [KPRIORITY; 7] = [8, 4, 8, 13, 24, 6, 10];

#[no_mangle]
pub unsafe extern "system" fn PsGetNextProcessThread(
    process: PEPROCESS,
    thread: PETHREAD,
) -> PETHREAD {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let mut found_thread: PETHREAD = core::ptr::null_mut();
    let list_head: PLIST_ENTRY;
    let mut entry: PLIST_ENTRY;

    KeEnterCriticalRegion();
    ExAcquirePushLockShared(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ProcessLock));

    if !thread.is_null() {
        entry = (*(thread as *mut _ETHREAD)).ThreadListEntry.Flink;
    } else {
        entry = (*(process as *mut _EPROCESS)).ThreadListHead.Flink;
    }

    list_head = core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ThreadListHead) as PLIST_ENTRY;

    while list_head != entry {
        // CONTAINING_RECORD(Entry, ETHREAD, ThreadListEntry)
        found_thread = (entry as *mut u8).sub(
            core::mem::offset_of!(_ETHREAD, ThreadListEntry)
        ) as PETHREAD;

        if ObReferenceObjectSafe(found_thread as PVOID) != 0 {
            break;
        }

        found_thread = core::ptr::null_mut();
        entry = (*entry).Flink;
    }

    ExReleasePushLockShared(core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).ProcessLock));
    KeLeaveCriticalRegion();

    if !thread.is_null() {
        ObDereferenceObject(thread as PVOID);
    }

    found_thread
}

#[no_mangle]
pub unsafe extern "system" fn PsGetNextProcess(old_process: PEPROCESS) -> PEPROCESS {
    let old_process = opaque_ptr(old_process);
    let mut entry: PLIST_ENTRY;
    let mut found_process: PEPROCESS = core::ptr::null_mut();

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));

    if !old_process.is_null() {
        entry = (*(old_process as *mut _EPROCESS)).ActiveProcessLinks.Flink;
    } else {
        entry = PsActiveProcessHead.Flink;
    }

    while entry != core::ptr::addr_of_mut!(PsActiveProcessHead) as PLIST_ENTRY {
        // CONTAINING_RECORD(Entry, EPROCESS, ActiveProcessLinks)
        found_process = (entry as *mut u8).sub(
            core::mem::offset_of!(_EPROCESS, ActiveProcessLinks)
        ) as PEPROCESS;

        if ObReferenceObjectSafe(found_process as PVOID) != 0 {
            break;
        }

        found_process = core::ptr::null_mut();
        entry = (*entry).Flink;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));

    if !old_process.is_null() {
        ObDereferenceObject(old_process as PVOID);
    }

    found_process
}

unsafe fn PspComputeQuantumAndPriority(
    process: PEPROCESS,
    mode: PSPROCESSPRIORITYMODE,
    quantum: *mut UCHAR,
) -> KPRIORITY {
    let i: ULONG;
    let local_quantum: UCHAR;
    let memory_priority: UCHAR;
    let proc = process as *mut _EPROCESS;

    if matches!(mode, PsProcessPriorityForeground) {
        memory_priority = MEMORY_PRIORITY_FOREGROUND;
        i = PsPrioritySeparation;
    } else {
        memory_priority = MEMORY_PRIORITY_BACKGROUND;
        i = 0;
    }

    if !matches!(mode, PsProcessPrioritySpinning) {
        MmSetMemoryPriorityProcess(process, memory_priority);
    }

    if (*proc).PriorityClass != PROCESS_PRIORITY_CLASS_IDLE as UCHAR {
        if !(*proc).Job.is_null() && PspUseJobSchedulingClasses != 0 {
            local_quantum = PspJobSchedulingClasses[(*((*proc).Job)).SchedulingClass as usize];
        } else {
            local_quantum = PspForegroundQuantum[i as usize] as UCHAR;
        }
    } else {
        local_quantum = 6;
    }

    *quantum = local_quantum;

    PspPriorityTable[(*proc).PriorityClass as usize]
}

#[no_mangle]
pub unsafe extern "system" fn PsChangeQuantumTable(
    immediate: BOOLEAN,
    priority_separation: ULONG,
) {
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut i: ULONG;
    let mut quantum: UCHAR;
    let mut quantum_table: *mut i8;

    PsPrioritySeparation = PspPrioritySeparationFromMask(priority_separation);

    if PsPrioritySeparation == 3 {
        PsPrioritySeparation = 2;
    }

    if PspQuantumTypeFromMask(priority_separation) == PSP_VARIABLE_QUANTUMS {
        quantum_table = (*(&raw mut PspVariableQuantums)).as_mut_ptr();
    } else if PspQuantumTypeFromMask(priority_separation) == PSP_FIXED_QUANTUMS {
        quantum_table = (*(&raw mut PspFixedQuantums)).as_mut_ptr();
    } else {
        quantum_table = if MmIsThisAnNtAsSystem() != 0 {
            (*(&raw mut PspFixedQuantums)).as_mut_ptr()
        } else {
            (*(&raw mut PspVariableQuantums)).as_mut_ptr()
        };
    }

    if PspQuantumLengthFromMask(priority_separation) == PSP_LONG_QUANTUMS {
        quantum_table = quantum_table.add(3);
    } else if PspQuantumLengthFromMask(priority_separation) == PSP_SHORT_QUANTUMS {
    } else {
        quantum_table = quantum_table.add(if MmIsThisAnNtAsSystem() != 0 { 3 } else { 0 });
    }

    if quantum_table == (*(&raw mut PspFixedQuantums)).as_mut_ptr().add(3) {
        PspUseJobSchedulingClasses = 1u8;
    } else {
        PspUseJobSchedulingClasses = 0u8;
    }

    core::ptr::copy_nonoverlapping(
        quantum_table as *const u8,
        (*(&raw mut PspForegroundQuantum)).as_mut_ptr() as *mut u8,
        core::mem::size_of_val(&*(&raw const PspForegroundQuantum)),
    );

    if immediate != 0 {
        process = PsGetNextProcess(process);
        while !process.is_null() {
            let proc = process as *mut _EPROCESS;
            i = if (*proc).Vm.Flags.MemoryPriority() == MEMORY_PRIORITY_BACKGROUND as u32 {
                0
            } else {
                PsPrioritySeparation
            };

            if (*proc).PriorityClass != PROCESS_PRIORITY_CLASS_IDLE as UCHAR {
                if !(*proc).Job.is_null() && PspUseJobSchedulingClasses != 0 {
                    quantum = PspJobSchedulingClasses[(*((*proc).Job)).SchedulingClass as usize];
                } else {
                    quantum = PspForegroundQuantum[i as usize] as UCHAR;
                }
            } else {
                quantum = 6;
            }

            KeSetQuantumProcess(
                core::ptr::addr_of_mut!((*proc).Pcb),
                quantum,
            );

            process = PsGetNextProcess(process);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PspCreateProcess(
    process_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    parent_process: HANDLE,
    mut flags: ULONG,
    section_handle: HANDLE,
    debug_port: HANDLE,
    exception_port: HANDLE,
    in_job: BOOLEAN,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let parent_process = opaque_ptr(parent_process);
    let section_handle = opaque_ptr(section_handle);
    let debug_port = opaque_ptr(debug_port);
    let exception_port = opaque_ptr(exception_port);
    let mut h_process: HANDLE = core::ptr::null_mut();
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut parent: PEPROCESS = core::ptr::null_mut();
    let mut exception_port_object: PVOID = core::ptr::null_mut();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let mut section_object: PSECTION = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut access_status: NTSTATUS = 0;
    let mut directory_table_base: [ULONG_PTR; 2] = [0, 0];
    let affinity: KAFFINITY;
    let mut cid_entry: _HANDLE_TABLE_ENTRY = core::mem::zeroed();
    let previous_mode: KPROCESSOR_MODE = KeGetPreviousMode();
    let current_process: PEPROCESS = PsGetCurrentProcess();
    let min_ws: ULONG;
    let max_ws: ULONG;
    let mut local_access_state: _ACCESS_STATE = core::mem::zeroed();
    let access_state: PACCESS_STATE = core::ptr::addr_of_mut!(local_access_state);
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();
    let mut quantum: UCHAR = 0;
    let result: BOOLEAN;
    let mut sd_allocated: BOOLEAN = 0u8;
    let mut security_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut subject_context: _SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut needs_peb: BOOLEAN = 0u8;
    let mut initial_peb: _INITIAL_PEB = core::mem::zeroed();

    ARM64_UART_TRACE(b"[PS] PspCreateProcess: enter\r\n\0".as_ptr() as *const i8);

    if (flags & !PROCESS_CREATE_FLAGS_LEGAL_MASK) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if !parent_process.is_null() {
        status = ObReferenceObjectByHandle(
            parent_process,
            PROCESS_CREATE_PROCESS,
            PsProcessType,
            previous_mode,
            core::ptr::addr_of_mut!(parent) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            return status;
        }

        if in_job != 0 && (*(parent as *mut _EPROCESS)).Job.is_null() {
            ObDereferenceObject(parent as PVOID);
            return STATUS_INVALID_PARAMETER;
        }

        affinity = (*(parent as *mut _EPROCESS)).Pcb.Affinity;
    } else {
        parent = core::ptr::null_mut();
        affinity = KeActiveProcessors;
    }

    min_ws = PsMinimumWorkingSet;
    max_ws = PsMaximumWorkingSet;

    status = ObCreateObject(
        previous_mode,
        PsProcessType,
        object_attributes,
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_EPROCESS>() as ULONG,
        0,
        0,
        core::ptr::addr_of_mut!(process) as *mut PVOID,
    );
    if status < 0 {
        if !parent.is_null() {
            ObDereferenceObject(parent as PVOID);
        }
        return status;
    }

    core::ptr::write_bytes(process as *mut u8, 0, core::mem::size_of::<_EPROCESS>());

    let proc = process as *mut _EPROCESS;

    ExInitializeRundownProtection(core::ptr::addr_of_mut!((*proc).RundownProtect));
    (*proc).ProcessLock.__bindgen_anon_1.Value = 0;

    InitializeListHead(core::ptr::addr_of_mut!((*proc).ThreadListHead) as PLIST_ENTRY);

    // The job process list entry (upstream 55563444377): PspAssignProcessToJob asserts it is
    // empty and PspRemoveProcessFromJob unlinks it.
    InitializeListHead(core::ptr::addr_of_mut!((*proc).JobLinks) as PLIST_ENTRY);

    PspInheritQuota(process, parent);
    ObInheritDeviceMap(parent, process);

    if !parent.is_null() {
        (*proc).InheritedFromUniqueProcessId = (*(parent as *mut _EPROCESS)).UniqueProcessId;
        (*proc).DefaultHardErrorProcessing = (*(parent as *mut _EPROCESS)).DefaultHardErrorProcessing;
    } else {
        (*proc).DefaultHardErrorProcessing = SEM_FAILCRITICALERRORS;
    }

    if !section_handle.is_null() {
        status = ObReferenceObjectByHandle(
            section_handle,
            SECTION_MAP_EXECUTE,
            MmSectionObjectType,
            previous_mode,
            core::ptr::addr_of_mut!(section_object) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }
    } else {
        section_object = core::ptr::null_mut();

        if parent != PsInitialSystemProcess as PEPROCESS && !parent.is_null() {
            if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*(parent as *mut _EPROCESS)).RundownProtect)) != 0 {
                section_object = (*(parent as *mut _EPROCESS)).SectionObject as PSECTION;
                if !section_object.is_null() {
                    ObReferenceObject(section_object as PVOID);
                }
                ExReleaseRundownProtection(core::ptr::addr_of_mut!((*(parent as *mut _EPROCESS)).RundownProtect));
            }

            if section_object.is_null() {
                status = STATUS_PROCESS_IS_TERMINATING;
                ObDereferenceObject(process as PVOID);
                if !parent.is_null() {
                    ObDereferenceObject(parent as PVOID);
                }
                return status;
            }
        }
    }

    (*proc).SectionObject = section_object as PVOID;

    if !debug_port.is_null() {
        status = ObReferenceObjectByHandle(
            debug_port,
            DEBUG_OBJECT_ADD_REMOVE_PROCESS,
            DbgkDebugObjectType,
            previous_mode,
            core::ptr::addr_of_mut!(debug_object) as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }

        (*proc).DebugPort = debug_object as PVOID;

        if (flags & PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT) != 0 {
            InterlockedOr(
                core::ptr::addr_of_mut!((*proc).__bindgen_anon_2.Flags) as *mut LONG,
                PSF_NO_DEBUG_INHERIT_BIT,
            );
        }
    } else {
        if !parent.is_null() {
            DbgkCopyProcessDebugPort(process, parent);
        }
    }

    if !exception_port.is_null() {
        status = ObReferenceObjectByHandle(
            exception_port,
            PORT_ALL_ACCESS,
            LpcPortObjectType,
            previous_mode,
            core::ptr::addr_of_mut!(exception_port_object),
            core::ptr::null_mut(),
        );
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }

        (*proc).ExceptionPort = exception_port_object;
    }

    (*proc).SectionObject = section_object as PVOID;
    (*proc).ExitStatus = STATUS_PENDING;

    if !parent.is_null() {
        ARM64_UART_TRACE(b"[PS] PspCreateProcess: MmCreateProcessAddressSpace\r\n\0".as_ptr() as *const i8);
        if MmCreateProcessAddressSpace(
            min_ws,
            process,
            directory_table_base.as_mut_ptr(),
        ) == 0 {
            status = STATUS_INSUFFICIENT_RESOURCES;
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }
    } else {
        (*proc).ObjectTable = (*(current_process as *mut _EPROCESS)).ObjectTable;
        status = MmInitializeHandBuiltProcess(process, directory_table_base.as_mut_ptr());
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }
    }

    InterlockedOr(
        core::ptr::addr_of_mut!((*proc).__bindgen_anon_2.Flags) as *mut LONG,
        PSF_HAS_ADDRESS_SPACE_BIT,
    );

    (*proc).Vm.MaximumWorkingSetSize = max_ws;

    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"PspCreateProcess-before-KeInitProc\0".as_ptr() as *const i8);
    }
    ARM64_UART_TRACE(b"[PS] PspCreateProcess: KeInitializeProcess\r\n\0".as_ptr() as *const i8);
    KeInitializeProcess(
        core::ptr::addr_of_mut!((*proc).Pcb),
        PROCESS_PRIORITY_NORMAL as KPRIORITY,
        affinity,
        directory_table_base.as_mut_ptr(),
        if ((*proc).DefaultHardErrorProcessing & SEM_NOALIGNMENTFAULTEXCEPT) != 0 { 1u8 } else { 0u8 },
    );

    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"PspCreateProcess-after-KeInitProc\0".as_ptr() as *const i8);
    }
    ARM64_UART_TRACE(b"[PS] PspCreateProcess: PspInitializeProcessSecurity\r\n\0".as_ptr() as *const i8);
    status = PspInitializeProcessSecurity(process, parent);
    if status < 0 {
        ObDereferenceObject(process as PVOID);
        if !parent.is_null() {
            ObDereferenceObject(parent as PVOID);
        }
        return status;
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"PspCreateProcess-after-InitSecurity\0".as_ptr() as *const i8);
    }
    ARM64_UART_TRACE(b"[PS] PspCreateProcess: PspInitializeProcessSecurity done\r\n\0".as_ptr() as *const i8);

    (*proc).PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL as UCHAR;

    if !parent.is_null() {
        if (*(parent as *mut _EPROCESS)).PriorityClass == PROCESS_PRIORITY_CLASS_IDLE as UCHAR ||
           (*(parent as *mut _EPROCESS)).PriorityClass == PROCESS_PRIORITY_CLASS_BELOW_NORMAL as UCHAR {
            (*proc).PriorityClass = (*(parent as *mut _EPROCESS)).PriorityClass;
        }

        #[cfg(target_arch = "aarch64")]
        {
            extern "C" { fn MmDbgCheckIrql(label: *const i8); }
            MmDbgCheckIrql(b"before-ObInitProcess\0".as_ptr() as *const i8);
        }
        ARM64_UART_TRACE(b"[PS] PspCreateProcess: ObInitProcess\r\n\0".as_ptr() as *const i8);
        status = ObInitProcess(
            if (flags & PROCESS_CREATE_FLAGS_INHERIT_HANDLES) != 0 { parent } else { core::ptr::null_mut() },
            process,
        );
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }
        #[cfg(target_arch = "aarch64")]
        {
            extern "C" { fn MmDbgCheckIrql(label: *const i8); }
            MmDbgCheckIrql(b"after-ObInitProcess\0".as_ptr() as *const i8);
        }
        ARM64_UART_TRACE(b"[PS] PspCreateProcess: ObInitProcess done\r\n\0".as_ptr() as *const i8);
    } else {
        status = MmInitializeHandBuiltProcess2(process);
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }
    }

    status = STATUS_SUCCESS;

    ARM64_UART_TRACE(b"[PS] PspCreateProcess: MmInitProcAS\r\n\0".as_ptr() as *const i8);
    if !section_handle.is_null() {
        status = MmInitializeProcessAddressSpace(
            process,
            core::ptr::null_mut(),
            section_object as PVOID,
            core::ptr::addr_of_mut!(flags) as *mut ULONG,
            core::ptr::addr_of_mut!((*proc).SeAuditProcessCreationInfo.ImageFileName),
        );
        if status < 0 {
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }

        needs_peb = 1u8;
    } else if !parent.is_null() {
        if parent != PsInitialSystemProcess as PEPROCESS {
            needs_peb = 1u8;
        } else {
            let mut flags_local = flags;
            flags_local &= !PROCESS_CREATE_FLAGS_LARGE_PAGES;
            status = MmInitializeProcessAddressSpace(
                process,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(flags_local),
                core::ptr::null_mut(),
            );
            if status < 0 {
                ObDereferenceObject(process as PVOID);
                if !parent.is_null() {
                    ObDereferenceObject(parent as PVOID);
                }
                return status;
            }

            (*proc).SeAuditProcessCreationInfo.ImageFileName = ExAllocatePoolWithTag(
                _POOL_TYPE_PagedPool,
                core::mem::size_of::<_OBJECT_NAME_INFORMATION>() as SIZE_T,
                TAG_SEPA,
            ) as POBJECT_NAME_INFORMATION;
            if (*proc).SeAuditProcessCreationInfo.ImageFileName.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                ObDereferenceObject(process as PVOID);
                if !parent.is_null() {
                    ObDereferenceObject(parent as PVOID);
                }
                return status;
            }

            core::ptr::write_bytes(
                (*proc).SeAuditProcessCreationInfo.ImageFileName as *mut u8,
                0,
                core::mem::size_of::<_OBJECT_NAME_INFORMATION>(),
            );
        }
    }

    #[cfg(feature = "mi_trace_pfns")]
    {
        let pfn_entry = MiGetPfnEntry((*proc).Pcb.DirectoryTableBase[0] >> PAGE_SHIFT);
        core::ptr::copy_nonoverlapping(
            (*proc).ImageFileName.as_ptr(),
            (*pfn_entry).ProcessName.as_mut_ptr(),
            16,
        );
        if (*proc).Pcb.DirectoryTableBase[1] != 0 {
            let pfn_entry = MiGetPfnEntry((*proc).Pcb.DirectoryTableBase[1] >> PAGE_SHIFT);
            core::ptr::copy_nonoverlapping(
                (*proc).ImageFileName.as_ptr(),
                (*pfn_entry).ProcessName.as_mut_ptr(),
                16,
            );
        }
        if (*proc).WorkingSetPage != 0 {
            let pfn_entry = MiGetPfnEntry((*proc).WorkingSetPage);
            core::ptr::copy_nonoverlapping(
                (*proc).ImageFileName.as_ptr(),
                (*pfn_entry).ProcessName.as_mut_ptr(),
                16,
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"before-MapSystemDll\0".as_ptr() as *const i8);
    }
    ARM64_UART_TRACE(b"[PS] PspCreateProcess: PspMapSystemDll\r\n\0".as_ptr() as *const i8);
    if !section_object.is_null() {
        PspMapSystemDll(process, core::ptr::null_mut(), 0u8);
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"after-MapSystemDll\0".as_ptr() as *const i8);
    }
    ARM64_UART_TRACE(b"[PS] PspCreateProcess: CreateHandle\r\n\0".as_ptr() as *const i8);
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn MmDbgCheckIrql(label: *const i8); }
        MmDbgCheckIrql(b"after-CreateHandle\0".as_ptr() as *const i8);
    }

    cid_entry.__bindgen_anon_1.Object = process as PVOID;
    cid_entry.__bindgen_anon_2.GrantedAccess = 0;
    (*proc).UniqueProcessId = ExCreateHandle(PspCidTable, core::ptr::addr_of_mut!(cid_entry));
    if (*proc).UniqueProcessId.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        ObDereferenceObject(process as PVOID);
        if !parent.is_null() {
            ObDereferenceObject(parent as PVOID);
        }
        return status;
    }

    (*(*proc).ObjectTable).UniqueProcessId = (*proc).UniqueProcessId;

    if SeDetailedAuditingWithToken(core::ptr::null_mut()) != 0 {
        SeAuditProcessCreate(process);
    }

    // Attach the process to the parent's job unless the job allows silent breakaway
    // (upstream 55563444377; the C here was a "Jobs not yet supported" DPRINT).
    if !parent.is_null()
        && !(*(parent as *mut _EPROCESS)).Job.is_null()
        && (*(*(parent as *mut _EPROCESS)).Job).LimitFlags & JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK == 0
    {
        let parent_job = (*(parent as *mut _EPROCESS)).Job;

        status = if flags & PROCESS_CREATE_FLAGS_BREAKAWAY != 0 {
            // An explicit breakaway is denied unless the job allows it.
            if (*parent_job).LimitFlags & JOB_OBJECT_LIMIT_BREAKAWAY_OK == 0 {
                STATUS_ACCESS_DENIED
            } else {
                STATUS_SUCCESS
            }
        } else {
            // Normally the child joins the parent's job.
            crate::job::PspAssignProcessToJob(process, parent_job)
        };
        if status < 0 {
            // goto CleanupWithRef
            ObDereferenceObject(process as PVOID);
            ObDereferenceObject(parent as PVOID);
            return status;
        }
    }

    if !parent.is_null() && needs_peb != 0 {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(initial_peb) as *mut u8,
            0,
            core::mem::size_of::<_INITIAL_PEB>(),
        );
        initial_peb.Mutant = -1isize as HANDLE;
        initial_peb.__bindgen_anon_1.__bindgen_anon_1.set_ImageUsesLargePages(0);

        if !section_handle.is_null() {
            status = MmCreatePeb(process, core::ptr::addr_of_mut!(initial_peb), core::ptr::addr_of_mut!((*proc).Peb));
            if status < 0 {
                ObDereferenceObject(process as PVOID);
                if !parent.is_null() {
                    ObDereferenceObject(parent as PVOID);
                }
                return status;
            }
        } else {
        }
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));
    InsertTailList(
        core::ptr::addr_of_mut!(PsActiveProcessHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*proc).ActiveProcessLinks) as PLIST_ENTRY,
    );
    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PspActiveProcessMutex));

    status = SeCreateAccessStateEx(
        PsGetCurrentThread() as *mut _ETHREAD,
        if !parent.is_null() && parent == PsInitialSystemProcess as PEPROCESS { parent } else { current_process },
        core::ptr::addr_of_mut!(local_access_state),
        core::ptr::addr_of_mut!(aux_data),
        desired_access,
        core::ptr::addr_of_mut!((*PsProcessType).TypeInfo.GenericMapping),
    );
    if status < 0 {
        ObDereferenceObject(process as PVOID);
        if !parent.is_null() {
            ObDereferenceObject(parent as PVOID);
        }
        return status;
    }

    status = ObInsertObject(
        process as PVOID,
        access_state,
        desired_access,
        1,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(h_process),
    );

    if !access_state.is_null() {
        SeDeleteAccessState(access_state);
    }

    if status < 0 {
        if !parent.is_null() {
            ObDereferenceObject(parent as PVOID);
        }
        return status;
    }

    (*proc).Pcb.BasePriority = PspComputeQuantumAndPriority(
        process,
        PsProcessPriorityBackground,
        core::ptr::addr_of_mut!(quantum),
    ) as i8;
    (*proc).Pcb.QuantumReset = quantum as i8;

    (*proc).GrantedAccess = PROCESS_TERMINATE;
    if !parent.is_null() && parent != PsInitialSystemProcess as PEPROCESS {
        status = ObGetObjectSecurity(
            process as PVOID,
            core::ptr::addr_of_mut!(security_descriptor),
            core::ptr::addr_of_mut!(sd_allocated),
        );
        if status < 0 {
            ObCloseHandle(h_process, previous_mode);
            ObDereferenceObject(process as PVOID);
            if !parent.is_null() {
                ObDereferenceObject(parent as PVOID);
            }
            return status;
        }

        subject_context.ProcessAuditId = process as PVOID;
        subject_context.PrimaryToken = PsReferencePrimaryToken(process);
        subject_context.ClientToken = core::ptr::null_mut();

        result = SeAccessCheck(
            security_descriptor,
            core::ptr::addr_of_mut!(subject_context),
            0u8,
            MAXIMUM_ALLOWED,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*PsProcessType).TypeInfo.GenericMapping),
            previous_mode,
            core::ptr::addr_of_mut!((*proc).GrantedAccess),
            core::ptr::addr_of_mut!(access_status) as *mut NTSTATUS,
        );

        ObFastDereferenceObject(
            core::ptr::addr_of_mut!((*proc).Token),
            subject_context.PrimaryToken,
        );
        ObReleaseObjectSecurity(security_descriptor, sd_allocated);

        if result == 0 {
            (*proc).GrantedAccess = 0;
        }

        (*proc).GrantedAccess |= PROCESS_VM_OPERATION |
                                  PROCESS_VM_READ |
                                  PROCESS_VM_WRITE |
                                  PROCESS_QUERY_INFORMATION |
                                  PROCESS_TERMINATE |
                                  PROCESS_CREATE_THREAD |
                                  PROCESS_DUP_HANDLE |
                                  PROCESS_CREATE_PROCESS |
                                  PROCESS_SET_INFORMATION |
                                  STANDARD_RIGHTS_ALL |
                                  PROCESS_SET_QUOTA;
    } else {
        (*proc).GrantedAccess = PROCESS_ALL_ACCESS;
    }

    KeQuerySystemTime(core::ptr::addr_of_mut!((*proc).CreateTime));

    // Hacky way of returning the PEB to the user-mode creator
    let current_thread = PsGetCurrentThread() as *mut _ETHREAD;
    if !(*proc).Peb.is_null() && !(*current_thread).Tcb.Teb.is_null() {
        (*((*current_thread).Tcb.Teb as *mut _TEB)).NtTib.ArbitraryUserPointer = (*proc).Peb as PVOID;
    }

    if let Err(s) = probe_write_user(process_handle, h_process, previous_mode) {
        return s;
    }

    PspRunCreateProcessNotifyRoutines(process, 1u8);

    ProcessCount += 1;
    if ProcessCount == 12 {
        Ki386PerfEnd();
    }

    ObDereferenceObject(process as PVOID);

    if !parent.is_null() {
        ObDereferenceObject(parent as PVOID);
    }

    ARM64_UART_TRACE(b"[PS] PspCreateProcess: returning\r\n\0".as_ptr() as *const i8);
    ARM64_UART_TRACE_HEX(b"[PS]   Status=\0".as_ptr() as *const i8, status as ULONG_PTR);
    status
}

#[no_mangle]
pub unsafe extern "system" fn PsCreateSystemProcess(
    process_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let object_attributes = opaque_ptr(object_attributes);
    PspCreateProcess(
        process_handle,
        desired_access,
        object_attributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PsLookupProcessByProcessId(
    process_id: HANDLE,
    process: *mut PEPROCESS,
) -> NTSTATUS {
    let process_id = opaque_ptr(process_id);
    let process = opaque_ptr(process);
    let mut cid_entry: PHANDLE_TABLE_ENTRY;
    let mut found_process: PEPROCESS;
    let mut status: NTSTATUS = STATUS_INVALID_PARAMETER;

    KeEnterCriticalRegion();

    cid_entry = ExMapHandleToPointer(PspCidTable, process_id);
    if !cid_entry.is_null() {
        found_process = (*cid_entry).__bindgen_anon_1.Object as PEPROCESS;

        if (*(found_process as *mut _EPROCESS)).Pcb.Header.__bindgen_anon_1.__bindgen_anon_1.Type == ProcessObject as UCHAR {
            if ObReferenceObjectSafe(found_process as PVOID) != 0 {
                *process = found_process;
                status = STATUS_SUCCESS;
            }
        }

        ExUnlockHandleTableEntry(PspCidTable, cid_entry);
    }

    KeLeaveCriticalRegion();
    status
}

#[no_mangle]
pub unsafe extern "system" fn PsLookupProcessThreadByCid(
    cid: PCLIENT_ID,
    process: *mut PEPROCESS,
    thread: *mut PETHREAD,
) -> NTSTATUS {
    let cid = opaque_ptr(cid);
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let mut cid_entry: PHANDLE_TABLE_ENTRY;
    let mut found_thread: PETHREAD;
    let mut status: NTSTATUS = STATUS_INVALID_CID;

    KeEnterCriticalRegion();

    cid_entry = ExMapHandleToPointer(PspCidTable, (*cid).UniqueThread);
    if !cid_entry.is_null() {
        found_thread = (*cid_entry).__bindgen_anon_1.Object as PETHREAD;

        if (*(found_thread as *mut _ETHREAD)).Tcb.Header.__bindgen_anon_1.__bindgen_anon_1.Type == ThreadObject as UCHAR &&
           (*(found_thread as *mut _ETHREAD)).Cid.UniqueProcess == (*cid).UniqueProcess {
            if ObReferenceObjectSafe(found_thread as PVOID) != 0 {
                *thread = found_thread;
                status = STATUS_SUCCESS;

                if !process.is_null() {
                    *process = (*(found_thread as *mut _ETHREAD)).ThreadsProcess;
                    ObReferenceObject(*process as PVOID);
                }
            }
        }

        ExUnlockHandleTableEntry(PspCidTable, cid_entry);
    }

    KeLeaveCriticalRegion();
    status
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessExitTime() -> LARGE_INTEGER {
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).ExitTime
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessCreateTimeQuadPart(process: PEPROCESS) -> i64 {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).CreateTime.QuadPart
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessDebugPort(process: PEPROCESS) -> PVOID {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).DebugPort
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessExitProcessCalled(process: PEPROCESS) -> BOOLEAN {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessExiting() as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessExitStatus(process: PEPROCESS) -> NTSTATUS {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).ExitStatus
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessId(process: PEPROCESS) -> HANDLE {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).UniqueProcessId
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessImageFileName(process: PEPROCESS) -> LPSTR {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).ImageFileName.as_ptr() as LPSTR
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessInheritedFromUniqueProcessId(process: PEPROCESS) -> HANDLE {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).InheritedFromUniqueProcessId
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessJob(process: PEPROCESS) -> PEJOB {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).Job
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessPeb(process: PEPROCESS) -> PPEB {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).Peb
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessPriorityClass(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).PriorityClass as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentProcessId() -> HANDLE {
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).UniqueProcessId
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentProcessSessionId() -> ULONG {
    MmGetSessionId(PsGetCurrentProcess())
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessSectionBaseAddress(process: PEPROCESS) -> PVOID {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).SectionBaseAddress
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessSecurityPort(process: PEPROCESS) -> PVOID {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).SecurityPort
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessSessionId(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    MmGetSessionId(process)
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessSessionIdEx(process: PEPROCESS) -> ULONG {
    let process = opaque_ptr(process);
    MmGetSessionIdEx(process)
}

#[no_mangle]
pub unsafe extern "system" fn PsGetCurrentProcessWin32Process() -> PVOID {
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Win32Process as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessWin32Process(process: PEPROCESS) -> PVOID {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).Win32Process as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetProcessWin32WindowStation(process: PEPROCESS) -> PVOID {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).Win32WindowStation
}

#[no_mangle]
pub unsafe extern "system" fn PsIsProcessBeingDebugged(process: PEPROCESS) -> BOOLEAN {
    let process = opaque_ptr(process);
    if (*(process as *mut _EPROCESS)).DebugPort.is_null() { 0u8 } else { 1u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsIsSystemProcess(process: PEPROCESS) -> BOOLEAN {
    let process = opaque_ptr(process);
    if process == PsInitialSystemProcess as PEPROCESS { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn PsSetProcessPriorityClass(
    process: PEPROCESS,
    priority_class: ULONG,
) {
    let process = opaque_ptr(process);
    (*(process as *mut _EPROCESS)).PriorityClass = priority_class as UCHAR;
}

#[no_mangle]
pub unsafe extern "system" fn PsSetProcessSecurityPort(
    process: PEPROCESS,
    security_port: PVOID,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let security_port = opaque_ptr(security_port);
    (*(process as *mut _EPROCESS)).SecurityPort = security_port;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PsSetProcessWin32Process(
    process: PEPROCESS,
    win32_process: PVOID,
    old_win32_process: PVOID,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let win32_process = opaque_ptr(win32_process);
    let old_win32_process = opaque_ptr(old_win32_process);
    let mut status: NTSTATUS;
    let proc = process as *mut _EPROCESS;

    status = STATUS_SUCCESS;

    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*proc).ProcessLock));

    if !win32_process.is_null() {
        if ((*proc).__bindgen_anon_2.Flags & PSF_PROCESS_DELETE_BIT as u32) == 0 &&
           (*proc).Win32Process.is_null() {
            (*proc).Win32Process = win32_process as *mut PVOID;
        } else {
            status = STATUS_PROCESS_IS_TERMINATING;
        }
    } else {
        if (*proc).Win32Process == old_win32_process as *mut PVOID {
            (*proc).Win32Process = core::ptr::null_mut();
        } else {
            status = STATUS_UNSUCCESSFUL;
        }
    }

    ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*proc).ProcessLock));
    KeLeaveCriticalRegion();

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsSetProcessWindowStation(
    process: PEPROCESS,
    window_station: PVOID,
) {
    let process = opaque_ptr(process);
    let window_station = opaque_ptr(window_station);
    (*(process as *mut _EPROCESS)).Win32WindowStation = window_station;
}

#[no_mangle]
pub unsafe extern "system" fn PsSetProcessPriorityByClass(
    process: PEPROCESS,
    ty: PSPROCESSPRIORITYMODE,
) {
    let process = opaque_ptr(process);
    let mut quantum: UCHAR = 0;
    let priority: ULONG;

    priority = PspComputeQuantumAndPriority(process, ty, core::ptr::addr_of_mut!(quantum)) as ULONG;

    KeSetPriorityAndQuantumProcess(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
        priority as KPRIORITY,
        quantum,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateProcessEx(
    process_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    parent_process: HANDLE,
    flags: ULONG,
    section_handle: HANDLE,
    debug_port: HANDLE,
    exception_port: HANDLE,
    in_job: BOOLEAN,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let parent_process = opaque_ptr(parent_process);
    let section_handle = opaque_ptr(section_handle);
    let debug_port = opaque_ptr(debug_port);
    let exception_port = opaque_ptr(exception_port);
    let previous_mode: KPROCESSOR_MODE = KeGetPreviousMode();
    let status: NTSTATUS;

    if previous_mode != 0 {
        if let Err(s) = probe_user_buffer(process_handle as _, core::mem::size_of::<HANDLE>(), core::mem::size_of::<HANDLE>(), previous_mode) {
            return s;
        }
    }

    if parent_process.is_null() {
        status = STATUS_INVALID_PARAMETER;
    } else {
        status = PspCreateProcess(
            process_handle, desired_access,
            object_attributes,
            parent_process,
            flags,
            section_handle,
            debug_port,
            exception_port,
            in_job,
        );
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateProcess(
    process_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    parent_process: HANDLE,
    inherit_object_table: BOOLEAN,
    section_handle: HANDLE,
    debug_port: HANDLE,
    exception_port: HANDLE,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let parent_process = opaque_ptr(parent_process);
    let section_handle = opaque_ptr(section_handle);
    let debug_port = opaque_ptr(debug_port);
    let exception_port = opaque_ptr(exception_port);
    let mut flags: ULONG = 0;

    if (section_handle as ULONG_PTR) & 1 != 0 {
        flags |= PROCESS_CREATE_FLAGS_BREAKAWAY;
    }
    if (debug_port as ULONG_PTR) & 1 != 0 {
        flags |= PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT;
    }
    if inherit_object_table != 0 {
        flags |= PROCESS_CREATE_FLAGS_INHERIT_HANDLES;
    }

    NtCreateProcessEx(
        process_handle,
        desired_access,
        object_attributes,
        parent_process,
        flags,
        section_handle,
        debug_port,
        exception_port,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenProcess(
    process_handle: PHANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    client_id: PCLIENT_ID,
) -> NTSTATUS {
    let mut process_handle = opaque_ptr(process_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let client_id = opaque_ptr(client_id);
    let previous_mode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut safe_client_id: _CLIENT_ID = core::mem::zeroed();
    let mut attributes: ULONG = 0;
    let mut h_process: HANDLE = core::ptr::null_mut();
    let mut has_object_name: BOOLEAN = 0u8;
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut access_state: _ACCESS_STATE = core::mem::zeroed();
    let mut aux_data: _AUX_ACCESS_DATA = core::mem::zeroed();
    let cid_ptr: PCLIENT_ID;

    if previous_mode != 0 {
        if let Err(s) = probe_user_buffer(process_handle as _, core::mem::size_of::<HANDLE>(), core::mem::size_of::<HANDLE>(), previous_mode) {
            return s;
        }

        if !client_id.is_null() {
            if let Err(s) = probe_user_buffer(client_id as _, core::mem::size_of::<_CLIENT_ID>(), core::mem::size_of::<ULONG>(), previous_mode) {
                return s;
            }
            safe_client_id = match ntkrnl_sys::probe::read_back(client_id) { Ok(v) => v, Err(s) => return s };
            cid_ptr = core::ptr::addr_of_mut!(safe_client_id);
        } else {
            cid_ptr = core::ptr::null_mut();
        }

        if let Err(s) = probe_user_buffer(
            object_attributes as _,
            core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }
        has_object_name = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*object_attributes).ObjectName)) { Ok(n) => (!n.is_null()) as u8, Err(s) => return s };

        attributes = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*object_attributes).Attributes)) { Ok(a) => ObpValidateAttributes(a, previous_mode), Err(s) => return s };
    } else {
        has_object_name = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*object_attributes).ObjectName)) { Ok(n) => (!n.is_null()) as u8, Err(s) => return s };
        attributes = match ntkrnl_sys::probe::read_back(core::ptr::addr_of!((*object_attributes).Attributes)) { Ok(a) => ObpValidateAttributes(a, previous_mode), Err(s) => return s };
        cid_ptr = client_id;
    }

    if has_object_name != 0 && !cid_ptr.is_null() {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    status = SeCreateAccessState(
        core::ptr::addr_of_mut!(access_state),
        core::ptr::addr_of_mut!(aux_data),
        desired_access,
        core::ptr::addr_of_mut!((*PsProcessType).TypeInfo.GenericMapping),
    );
    if status < 0 {
        return status;
    }

    if SeSinglePrivilegeCheck(SeDebugPrivilege, previous_mode) != 0 {
        if (access_state.RemainingDesiredAccess & MAXIMUM_ALLOWED) != 0 {
            access_state.PreviouslyGrantedAccess |= PROCESS_ALL_ACCESS;
        } else {
            access_state.PreviouslyGrantedAccess |= access_state.RemainingDesiredAccess;
        }

        access_state.RemainingDesiredAccess = 0;
    }

    if has_object_name != 0 {
        status = ObOpenObjectByName(
            object_attributes,
            PsProcessType,
            previous_mode,
            core::ptr::addr_of_mut!(access_state),
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(h_process),
        );

        SeDeleteAccessState(core::ptr::addr_of_mut!(access_state));
    } else if !cid_ptr.is_null() {
        if !(*cid_ptr).UniqueThread.is_null() {
            status = PsLookupProcessThreadByCid(cid_ptr, core::ptr::addr_of_mut!(process), core::ptr::addr_of_mut!(thread));
        } else {
            status = PsLookupProcessByProcessId((*cid_ptr).UniqueProcess, core::ptr::addr_of_mut!(process));
        }

        if status < 0 {
            SeDeleteAccessState(core::ptr::addr_of_mut!(access_state));
            return status;
        }

        status = ObOpenObjectByPointer(
            process as PVOID,
            attributes,
            core::ptr::addr_of_mut!(access_state),
            0,
            PsProcessType,
            previous_mode,
            core::ptr::addr_of_mut!(h_process),
        );

        SeDeleteAccessState(core::ptr::addr_of_mut!(access_state));

        if !thread.is_null() {
            ObDereferenceObject(thread as PVOID);
        }

        ObDereferenceObject(process as PVOID);
    } else {
        return STATUS_INVALID_PARAMETER_MIX;
    }

    if status >= 0 {
        if let Err(s) = probe_write_user(process_handle, h_process, previous_mode) {
            return s;
        }
    }

    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/process.c
//  * PURPOSE:         Process Manager: Process Management
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern ULONG PsMinimumWorkingSet, PsMaximumWorkingSet;
//
// POBJECT_TYPE PsProcessType = NULL;
//
// LIST_ENTRY PsActiveProcessHead;
// KGUARDED_MUTEX PspActiveProcessMutex;
//
// LARGE_INTEGER ShortPsLockDelay;
//
// ULONG PsRawPrioritySeparation;
// ULONG PsPrioritySeparation;
// CHAR PspForegroundQuantum[3];
//
// /* Fixed quantum table */
// CHAR PspFixedQuantums[6] =
// {
//     /* Short quantums */
//     3 * 6, /* Level 1 */
//     3 * 6, /* Level 2 */
//     3 * 6, /* Level 3 */
//
//     /* Long quantums */
//     6 * 6, /* Level 1 */
//     6 * 6, /* Level 2 */
//     6 * 6  /* Level 3 */
// };
//
// /* Variable quantum table */
// CHAR PspVariableQuantums[6] =
// {
//     /* Short quantums */
//     1 * 6, /* Level 1 */
//     2 * 6, /* Level 2 */
//     3 * 6, /* Level 3 */
//
//     /* Long quantums */
//     2 * 6, /* Level 1 */
//     4 * 6, /* Level 2 */
//     6 * 6  /* Level 3 */
// };
//
// /* Priority table */
// KPRIORITY PspPriorityTable[PROCESS_PRIORITY_CLASS_ABOVE_NORMAL + 1] =
// {
//     8,
//     4,
//     8,
//     13,
//     24,
//     6,
//     10
// };
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PETHREAD
// NTAPI
// PsGetNextProcessThread(IN PEPROCESS Process,
//                        IN PETHREAD Thread OPTIONAL)
// {
//     PETHREAD FoundThread = NULL;
//     PLIST_ENTRY ListHead, Entry;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG,
//             "Process: %p Thread: %p\n", Process, Thread);
//
//     /* Lock the process */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockShared(&Process->ProcessLock);
//
//     /* Check if we're already starting somewhere */
//     if (Thread)
//     {
//         /* Start where we left off */
//         Entry = Thread->ThreadListEntry.Flink;
//     }
//     else
//     {
//         /* Start at the beginning */
//         Entry = Process->ThreadListHead.Flink;
//     }
//
//     /* Set the list head and start looping */
//     ListHead = &Process->ThreadListHead;
//     while (ListHead != Entry)
//     {
//         /* Get the Thread */
//         FoundThread = CONTAINING_RECORD(Entry, ETHREAD, ThreadListEntry);
//
//         /* Safe reference the thread */
//         if (ObReferenceObjectSafe(FoundThread)) break;
//
//         /* Nothing found, keep looping */
//         FoundThread = NULL;
//         Entry = Entry->Flink;
//     }
//
//     /* Unlock the process */
//     ExReleasePushLockShared(&Process->ProcessLock);
//     KeLeaveCriticalRegion();
//
//     /* Check if we had a starting thread, and dereference it */
//     if (Thread) ObDereferenceObject(Thread);
//
//     /* Return what we found */
//     return FoundThread;
// }
//
// PEPROCESS
// NTAPI
// PsGetNextProcess(IN PEPROCESS OldProcess)
// {
//     PLIST_ENTRY Entry;
//     PEPROCESS FoundProcess = NULL;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG, "Process: %p\n", OldProcess);
//
//     /* Acquire the Active Process Lock */
//     KeAcquireGuardedMutex(&PspActiveProcessMutex);
//
//     /* Check if we're already starting somewhere */
//     if (OldProcess)
//     {
//         /* Start where we left off */
//         Entry = OldProcess->ActiveProcessLinks.Flink;
//     }
//     else
//     {
//         /* Start at the beginning */
//         Entry = PsActiveProcessHead.Flink;
//     }
//
//     /* Loop the process list */
//     while (Entry != &PsActiveProcessHead)
//     {
//         /* Get the process */
//         FoundProcess = CONTAINING_RECORD(Entry, EPROCESS, ActiveProcessLinks);
//
//         /* Reference the process */
//         if (ObReferenceObjectSafe(FoundProcess)) break;
//
//         /* Nothing found, keep trying */
//         FoundProcess = NULL;
//         Entry = Entry->Flink;
//     }
//
//     /* Release the lock */
//     KeReleaseGuardedMutex(&PspActiveProcessMutex);
//
//     /* Dereference the Process we had referenced earlier */
//     if (OldProcess) ObDereferenceObject(OldProcess);
//     return FoundProcess;
// }
//
// KPRIORITY
// NTAPI
// PspComputeQuantumAndPriority(IN PEPROCESS Process,
//                              IN PSPROCESSPRIORITYMODE Mode,
//                              OUT PUCHAR Quantum)
// {
//     ULONG i;
//     UCHAR LocalQuantum, MemoryPriority;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG, "Process: %p Mode: %lx\n", Process, Mode);
//
//     /* Check if this is a foreground process */
//     if (Mode == PsProcessPriorityForeground)
//     {
//         /* Set the memory priority and use priority separation */
//         MemoryPriority = MEMORY_PRIORITY_FOREGROUND;
//         i = PsPrioritySeparation;
//     }
//     else
//     {
//         /* Set the background memory priority and no separation */
//         MemoryPriority = MEMORY_PRIORITY_BACKGROUND;
//         i = 0;
//     }
//
//     /* Make sure that the process mode isn't spinning */
//     if (Mode != PsProcessPrioritySpinning)
//     {
//         /* Set the priority */
//         MmSetMemoryPriorityProcess(Process, MemoryPriority);
//     }
//
//     /* Make sure that the process isn't idle */
//     if (Process->PriorityClass != PROCESS_PRIORITY_CLASS_IDLE)
//     {
//         /* Does the process have a job? */
//         if ((Process->Job) && (PspUseJobSchedulingClasses))
//         {
//             /* Use job quantum */
//             LocalQuantum = PspJobSchedulingClasses[Process->Job->
//                                                    SchedulingClass];
//         }
//         else
//         {
//             /* Use calculated quantum */
//             LocalQuantum = PspForegroundQuantum[i];
//         }
//     }
//     else
//     {
//         /* Process is idle, use default quantum */
//         LocalQuantum = 6;
//     }
//
//     /* Return quantum to caller */
//     *Quantum = LocalQuantum;
//
//     /* Return priority */
//     return PspPriorityTable[Process->PriorityClass];
// }
//
// VOID
// NTAPI
// PsChangeQuantumTable(IN BOOLEAN Immediate,
//                      IN ULONG PrioritySeparation)
// {
//     PEPROCESS Process = NULL;
//     ULONG i;
//     UCHAR Quantum;
//     PCHAR QuantumTable;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG,
//             "%lx PrioritySeparation: %lx\n", Immediate, PrioritySeparation);
//
//     /* Write the current priority separation */
//     PsPrioritySeparation = PspPrioritySeparationFromMask(PrioritySeparation);
//
//     /* Normalize it if it was too high */
//     if (PsPrioritySeparation == 3) PsPrioritySeparation = 2;
//
//     /* Get the quantum table to use */
//     if (PspQuantumTypeFromMask(PrioritySeparation) == PSP_VARIABLE_QUANTUMS)
//     {
//         /* Use a variable table */
//         QuantumTable = PspVariableQuantums;
//     }
//     else if (PspQuantumTypeFromMask(PrioritySeparation) == PSP_FIXED_QUANTUMS)
//     {
//         /* Use fixed table */
//         QuantumTable = PspFixedQuantums;
//     }
//     else
//     {
//         /* Use default for the type of system we're on */
//         QuantumTable = MmIsThisAnNtAsSystem() ? PspFixedQuantums : PspVariableQuantums;
//     }
//
//     /* Now check if we should use long or short */
//     if (PspQuantumLengthFromMask(PrioritySeparation) == PSP_LONG_QUANTUMS)
//     {
//         /* Use long quantums */
//         QuantumTable += 3;
//     }
//     else if (PspQuantumLengthFromMask(PrioritySeparation) == PSP_SHORT_QUANTUMS)
//     {
//         /* Keep existing table */
//         NOTHING;
//     }
//     else
//     {
//         /* Use default for the type of system we're on */
//         QuantumTable += MmIsThisAnNtAsSystem() ? 3 : 0;
//     }
//
//     /* Check if we're using long fixed quantums */
//     if (QuantumTable == &PspFixedQuantums[3])
//     {
//         /* Use Job scheduling classes */
//          PspUseJobSchedulingClasses = TRUE;
//     }
//     else
//     {
//         /* Otherwise, we don't */
//         PspUseJobSchedulingClasses = FALSE;
//     }
//
//     /* Copy the selected table into the Foreground Quantum table */
//     RtlCopyMemory(PspForegroundQuantum,
//                   QuantumTable,
//                   sizeof(PspForegroundQuantum));
//
//     /* Check if we should apply these changes real-time */
//     if (Immediate)
//     {
//         /* We are...loop every process */
//         Process = PsGetNextProcess(Process);
//         while (Process)
//         {
//             /* Use the priority separation if this is a foreground process */
//             i = (Process->Vm.Flags.MemoryPriority ==
//                  MEMORY_PRIORITY_BACKGROUND) ?
//                  0: PsPrioritySeparation;
//
//             /* Make sure that the process isn't idle */
//             if (Process->PriorityClass != PROCESS_PRIORITY_CLASS_IDLE)
//             {
//                 /* Does the process have a job? */
//                 if ((Process->Job) && (PspUseJobSchedulingClasses))
//                 {
//                     /* Use job quantum */
//                     Quantum = PspJobSchedulingClasses[Process->Job->SchedulingClass];
//                 }
//                 else
//                 {
//                     /* Use calculated quantum */
//                     Quantum = PspForegroundQuantum[i];
//                 }
//             }
//             else
//             {
//                 /* Process is idle, use default quantum */
//                 Quantum = 6;
//             }
//
//             /* Now set the quantum */
//             KeSetQuantumProcess(&Process->Pcb, Quantum);
//
//             /* Get the next process */
//             Process = PsGetNextProcess(Process);
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// PspCreateProcess(OUT PHANDLE ProcessHandle,
//                  IN ACCESS_MASK DesiredAccess,
//                  IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                  IN HANDLE ParentProcess OPTIONAL,
//                  IN ULONG Flags,
//                  IN HANDLE SectionHandle OPTIONAL,
//                  IN HANDLE DebugPort OPTIONAL,
//                  IN HANDLE ExceptionPort OPTIONAL,
//                  IN BOOLEAN InJob)
// {
//     HANDLE hProcess;
//     PEPROCESS Process, Parent;
//     PVOID ExceptionPortObject;
//     PDEBUG_OBJECT DebugObject;
//     PSECTION SectionObject;
//     NTSTATUS Status, AccessStatus;
//     ULONG_PTR DirectoryTableBase[2] = {0,0};
//     KAFFINITY Affinity;
//     HANDLE_TABLE_ENTRY CidEntry;
//     PETHREAD CurrentThread = PsGetCurrentThread();
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PEPROCESS CurrentProcess = PsGetCurrentProcess();
//     ULONG MinWs, MaxWs;
//     ACCESS_STATE LocalAccessState;
//     PACCESS_STATE AccessState = &LocalAccessState;
//     AUX_ACCESS_DATA AuxData;
//     UCHAR Quantum;
//     BOOLEAN Result, SdAllocated;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     BOOLEAN NeedsPeb = FALSE;
//     INITIAL_PEB InitialPeb;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG,
//             "ProcessHandle: %p Parent: %p\n", ProcessHandle, ParentProcess);
//
//     /* Validate flags */
//     ARM64_UART_TRACE("[PS] PspCreateProcess: enter\r\n");
//     if (Flags & ~PROCESS_CREATE_FLAGS_LEGAL_MASK) return STATUS_INVALID_PARAMETER;
//
//     /* Check for parent */
//     if (ParentProcess)
//     {
//         /* Reference it */
//         Status = ObReferenceObjectByHandle(ParentProcess,
//                                            PROCESS_CREATE_PROCESS,
//                                            PsProcessType,
//                                            PreviousMode,
//                                            (PVOID*)&Parent,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* If this process should be in a job but the parent isn't */
//         if ((InJob) && (!Parent->Job))
//         {
//             /* This is illegal. Dereference the parent and fail */
//             ObDereferenceObject(Parent);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Inherit Parent process's Affinity. */
//         Affinity = Parent->Pcb.Affinity;
//     }
//     else
//     {
//         /* We have no parent */
//         Parent = NULL;
//         Affinity = KeActiveProcessors;
//     }
//
//     /* Save working set data */
//     MinWs = PsMinimumWorkingSet;
//     MaxWs = PsMaximumWorkingSet;
//
//     /* Create the Object */
//     Status = ObCreateObject(PreviousMode,
//                             PsProcessType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(EPROCESS),
//                             0,
//                             0,
//                             (PVOID*)&Process);
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Clean up the Object */
//     RtlZeroMemory(Process, sizeof(EPROCESS));
//
//     /* Initialize pushlock and rundown protection */
//     ExInitializeRundownProtection(&Process->RundownProtect);
//     Process->ProcessLock.Value = 0;
//
//     /* Setup the Thread List Head */
//     InitializeListHead(&Process->ThreadListHead);
//
//     /* Set up the Quota Block from the Parent */
//     PspInheritQuota(Process, Parent);
//
//     /* Set up Dos Device Map from the Parent */
//     ObInheritDeviceMap(Parent, Process);
//
//     /* Check if we have a parent */
//     if (Parent)
//     {
//         /* Inherit PID and hard-error processing */
//         Process->InheritedFromUniqueProcessId = Parent->UniqueProcessId;
//         Process->DefaultHardErrorProcessing = Parent->DefaultHardErrorProcessing;
//     }
//     else
//     {
//         /* Use default hard-error processing */
//         Process->DefaultHardErrorProcessing = SEM_FAILCRITICALERRORS;
//     }
//
//     /* Check for a section handle */
//     if (SectionHandle)
//     {
//         /* Get a pointer to it */
//         Status = ObReferenceObjectByHandle(SectionHandle,
//                                            SECTION_MAP_EXECUTE,
//                                            MmSectionObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&SectionObject,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//     }
//     else
//     {
//         /* Assume no section object */
//         SectionObject = NULL;
//
//         /* Is the parent the initial process?
//          * Check for NULL also, as at initialization PsInitialSystemProcess is NULL */
//         if (Parent != PsInitialSystemProcess && (Parent != NULL))
//         {
//             /* It's not, so acquire the process rundown */
//             if (ExAcquireRundownProtection(&Parent->RundownProtect))
//             {
//                 /* If the parent has a section, use it */
//                 SectionObject = Parent->SectionObject;
//                 if (SectionObject) ObReferenceObject(SectionObject);
//
//                 /* Release process rundown */
//                 ExReleaseRundownProtection(&Parent->RundownProtect);
//             }
//
//             /* If we don't have a section object */
//             if (!SectionObject)
//             {
//                 /* Then the process is in termination, so fail */
//                 Status = STATUS_PROCESS_IS_TERMINATING;
//                 goto CleanupWithRef;
//             }
//         }
//     }
//
//     /* Save the pointer to the section object */
//     Process->SectionObject = SectionObject;
//
//     /* Check for the debug port */
//     if (DebugPort)
//     {
//         /* Reference it */
//         Status = ObReferenceObjectByHandle(DebugPort,
//                                            DEBUG_OBJECT_ADD_REMOVE_PROCESS,
//                                            DbgkDebugObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&DebugObject,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//
//         /* Save the debug object */
//         Process->DebugPort = DebugObject;
//
//         /* Check if the caller doesn't want the debug stuff inherited */
//         if (Flags & PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT)
//         {
//             /* Set the process flag */
//             InterlockedOr((PLONG)&Process->Flags, PSF_NO_DEBUG_INHERIT_BIT);
//         }
//     }
//     else
//     {
//         /* Do we have a parent? Copy his debug port */
//         if (Parent) DbgkCopyProcessDebugPort(Process, Parent);
//     }
//
//     /* Now check for an exception port */
//     if (ExceptionPort)
//     {
//         /* Reference it */
//         Status = ObReferenceObjectByHandle(ExceptionPort,
//                                            PORT_ALL_ACCESS,
//                                            LpcPortObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&ExceptionPortObject,
//                                            NULL);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//
//         /* Save the exception port */
//         Process->ExceptionPort = ExceptionPortObject;
//     }
//
//     /* Save the pointer to the section object */
//     Process->SectionObject = SectionObject;
//
//     /* Set default exit code */
//     Process->ExitStatus = STATUS_PENDING;
//
//     /* Check if this is the initial process being built */
//     if (Parent)
//     {
//         /* Create the address space for the child */
//         ARM64_UART_TRACE("[PS] PspCreateProcess: MmCreateProcessAddressSpace\r\n");
//         if (!MmCreateProcessAddressSpace(MinWs,
//                                          Process,
//                                          DirectoryTableBase))
//         {
//             /* Failed */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto CleanupWithRef;
//         }
//     }
//     else
//     {
//         /* Otherwise, we are the boot process, we're already semi-initialized */
//         Process->ObjectTable = CurrentProcess->ObjectTable;
//         Status = MmInitializeHandBuiltProcess(Process, DirectoryTableBase);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//     }
//
//     /* We now have an address space */
//     InterlockedOr((PLONG)&Process->Flags, PSF_HAS_ADDRESS_SPACE_BIT);
//
//     /* Set the maximum WS */
//     Process->Vm.MaximumWorkingSetSize = MaxWs;
//
//     /* Now initialize the Kernel Process */
//     ARM64_UART_TRACE("[PS] PspCreateProcess: KeInitializeProcess\r\n");
//     KeInitializeProcess(&Process->Pcb,
//                         PROCESS_PRIORITY_NORMAL,
//                         Affinity,
//                         DirectoryTableBase,
//                         BooleanFlagOn(Process->DefaultHardErrorProcessing,
//                                       SEM_NOALIGNMENTFAULTEXCEPT));
//
//     /* Duplicate Parent Token */
//     ARM64_UART_TRACE("[PS] PspCreateProcess: PspInitializeProcessSecurity\r\n");
//     Status = PspInitializeProcessSecurity(Process, Parent);
//     if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//     ARM64_UART_TRACE("[PS] PspCreateProcess: PspInitializeProcessSecurity done\r\n");
//
//     /* Set default priority class */
//     Process->PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL;
//
//     /* Check if we have a parent */
//     if (Parent)
//     {
//         /* Check our priority class */
//         if (Parent->PriorityClass == PROCESS_PRIORITY_CLASS_IDLE ||
//             Parent->PriorityClass == PROCESS_PRIORITY_CLASS_BELOW_NORMAL)
//         {
//             /* Normalize it */
//             Process->PriorityClass = Parent->PriorityClass;
//         }
//
//         /* Initialize object manager for the process */
//         ARM64_UART_TRACE("[PS] PspCreateProcess: ObInitProcess\r\n");
//         Status = ObInitProcess(Flags & PROCESS_CREATE_FLAGS_INHERIT_HANDLES ?
//                                Parent : NULL,
//                                Process);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//         ARM64_UART_TRACE("[PS] PspCreateProcess: ObInitProcess done\r\n");
//     }
//     else
//     {
//         /* Do the second part of the boot process memory setup */
//         Status = MmInitializeHandBuiltProcess2(Process);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//     }
//
//     /* Set success for now */
//     Status = STATUS_SUCCESS;
//
//     /* Check if this is a real user-mode process */
//     ARM64_UART_TRACE("[PS] PspCreateProcess: MmInitProcAS\r\n");
//     if (SectionHandle)
//     {
//         /* Initialize the address space */
//         Status = MmInitializeProcessAddressSpace(Process,
//                                                  NULL,
//                                                  SectionObject,
//                                                  &Flags,
//                                                  &Process->
//                                                  SeAuditProcessCreationInfo.
//                                                  ImageFileName);
//         if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//
//         //
//         // We need a PEB
//         //
//         NeedsPeb = TRUE;
//     }
//     else if (Parent)
//     {
//         /* Check if this is a child of the system process */
//         if (Parent != PsInitialSystemProcess)
//         {
//             //
//             // We need a PEB
//             //
//             NeedsPeb = TRUE;
//
//             /* This is a clone! */
//             ASSERTMSG("No support for cloning yet\n", FALSE);
//         }
//         else
//         {
//             /* This is the initial system process */
//             Flags &= ~PROCESS_CREATE_FLAGS_LARGE_PAGES;
//             Status = MmInitializeProcessAddressSpace(Process,
//                                                      NULL,
//                                                      NULL,
//                                                      &Flags,
//                                                      NULL);
//             if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//
//             /* Create a dummy image file name */
//             Process->SeAuditProcessCreationInfo.ImageFileName =
//                 ExAllocatePoolWithTag(PagedPool,
//                                       sizeof(OBJECT_NAME_INFORMATION),
//                                       TAG_SEPA);
//             if (!Process->SeAuditProcessCreationInfo.ImageFileName)
//             {
//                 /* Fail */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto CleanupWithRef;
//             }
//
//             /* Zero it out */
//             RtlZeroMemory(Process->SeAuditProcessCreationInfo.ImageFileName,
//                           sizeof(OBJECT_NAME_INFORMATION));
//         }
//     }
//
// #if MI_TRACE_PFNS
//     /* Copy the process name now that we have it */
//     memcpy(MiGetPfnEntry(Process->Pcb.DirectoryTableBase[0] >> PAGE_SHIFT)->ProcessName, Process->ImageFileName, 16);
//     if (Process->Pcb.DirectoryTableBase[1]) memcpy(MiGetPfnEntry(Process->Pcb.DirectoryTableBase[1] >> PAGE_SHIFT)->ProcessName, Process->ImageFileName, 16);
//     if (Process->WorkingSetPage) memcpy(MiGetPfnEntry(Process->WorkingSetPage)->ProcessName, Process->ImageFileName, 16);
// #endif
//
//     /* Check if we have a section object and map the system DLL */
//     ARM64_UART_TRACE("[PS] PspCreateProcess: PspMapSystemDll\r\n");
//     if (SectionObject) PspMapSystemDll(Process, NULL, FALSE);
//     ARM64_UART_TRACE("[PS] PspCreateProcess: CreateHandle\r\n");
//
//     /* Create a handle for the Process */
//     CidEntry.Object = Process;
//     CidEntry.GrantedAccess = 0;
//     Process->UniqueProcessId = ExCreateHandle(PspCidTable, &CidEntry);
//     if (!Process->UniqueProcessId)
//     {
//         /* Fail */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto CleanupWithRef;
//     }
//
//     /* Set the handle table PID */
//     Process->ObjectTable->UniqueProcessId = Process->UniqueProcessId;
//
//     /* Check if we need to audit */
//     if (SeDetailedAuditingWithToken(NULL)) SeAuditProcessCreate(Process);
//
//     /* Check if the parent had a job */
//     if ((Parent) && (Parent->Job))
//     {
//         /* FIXME: We need to insert this process */
//         DPRINT1("Jobs not yet supported\n");
//     }
//
//     /* Create PEB only for User-Mode Processes */
//     if ((Parent) && (NeedsPeb))
//     {
//         //
//         // Set up the initial PEB
//         //
//         RtlZeroMemory(&InitialPeb, sizeof(INITIAL_PEB));
//         InitialPeb.Mutant = (HANDLE)-1;
//         InitialPeb.ImageUsesLargePages = 0; // FIXME: Not yet supported
//
//         //
//         // Create it only if we have an image section
//         //
//         if (SectionHandle)
//         {
//             //
//             // Create it
//             //
//             Status = MmCreatePeb(Process, &InitialPeb, &Process->Peb);
//             if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//         }
//         else
//         {
//             //
//             // We have to clone it
//             //
//             ASSERTMSG("No support for cloning yet\n", FALSE);
//         }
//
//     }
//
//     /* The process can now be activated */
//     KeAcquireGuardedMutex(&PspActiveProcessMutex);
//     InsertTailList(&PsActiveProcessHead, &Process->ActiveProcessLinks);
//     KeReleaseGuardedMutex(&PspActiveProcessMutex);
//
//     /* Create an access state */
//     Status = SeCreateAccessStateEx(CurrentThread,
//                                    ((Parent) &&
//                                    (Parent == PsInitialSystemProcess)) ?
//                                     Parent : CurrentProcess,
//                                    &LocalAccessState,
//                                    &AuxData,
//                                    DesiredAccess,
//                                    &PsProcessType->TypeInfo.GenericMapping);
//     if (!NT_SUCCESS(Status)) goto CleanupWithRef;
//
//     /* Insert the Process into the Object Directory */
//     Status = ObInsertObject(Process,
//                             AccessState,
//                             DesiredAccess,
//                             1,
//                             NULL,
//                             &hProcess);
//
//     /* Free the access state */
//     if (AccessState) SeDeleteAccessState(AccessState);
//
//     /* Cleanup on failure */
//     if (!NT_SUCCESS(Status)) goto Cleanup;
//
//     /* Compute Quantum and Priority */
//     ASSERT(IsListEmpty(&Process->ThreadListHead) == TRUE);
//     Process->Pcb.BasePriority =
//         (SCHAR)PspComputeQuantumAndPriority(Process,
//                                             PsProcessPriorityBackground,
//                                             &Quantum);
//     Process->Pcb.QuantumReset = Quantum;
//
//     /* Check if we have a parent other then the initial system process */
//     Process->GrantedAccess = PROCESS_TERMINATE;
//     if ((Parent) && (Parent != PsInitialSystemProcess))
//     {
//         /* Get the process's SD */
//         Status = ObGetObjectSecurity(Process,
//                                      &SecurityDescriptor,
//                                      &SdAllocated);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, close the handle and clean up */
//             ObCloseHandle(hProcess, PreviousMode);
//             goto CleanupWithRef;
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
//                                &PsProcessType->TypeInfo.GenericMapping,
//                                PreviousMode,
//                                &Process->GrantedAccess,
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
//         /* Give the process some basic access */
//         Process->GrantedAccess |= (PROCESS_VM_OPERATION |
//                                    PROCESS_VM_READ |
//                                    PROCESS_VM_WRITE |
//                                    PROCESS_QUERY_INFORMATION |
//                                    PROCESS_TERMINATE |
//                                    PROCESS_CREATE_THREAD |
//                                    PROCESS_DUP_HANDLE |
//                                    PROCESS_CREATE_PROCESS |
//                                    PROCESS_SET_INFORMATION |
//                                    STANDARD_RIGHTS_ALL |
//                                    PROCESS_SET_QUOTA);
//     }
//     else
//     {
//         /* Set full granted access */
//         Process->GrantedAccess = PROCESS_ALL_ACCESS;
//     }
//
//     /* Set the Creation Time */
//     KeQuerySystemTime(&Process->CreateTime);
//
//     /* Protect against bad user-mode pointer */
//     _SEH2_TRY
//     {
//         /* Hacky way of returning the PEB to the user-mode creator */
//         if ((Process->Peb) && (CurrentThread->Tcb.Teb))
//         {
//             CurrentThread->Tcb.Teb->NtTib.ArbitraryUserPointer = Process->Peb;
//         }
//
//         /* Save the process handle */
//        *ProcessHandle = hProcess;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the exception code */
//        Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Run the Notification Routines */
//     PspRunCreateProcessNotifyRoutines(Process, TRUE);
//
//     /* If 12 processes have been created, enough of user-mode is ready */
//     if (++ProcessCount == 12) Ki386PerfEnd();
//
// CleanupWithRef:
//     /*
//      * Dereference the process. For failures, kills the process and does
//      * cleanup present in PspDeleteProcess. For success, kills the extra
//      * reference added by ObInsertObject.
//      */
//     ObDereferenceObject(Process);
//
// Cleanup:
//     /* Dereference the parent */
//     if (Parent) ObDereferenceObject(Parent);
//
//     ARM64_UART_TRACE("[PS] PspCreateProcess: returning\r\n");
//     ARM64_UART_TRACE_HEX("[PS]   Status=", (ULONG_PTR)Status);
//     /* Return status to caller */
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsCreateSystemProcess(OUT PHANDLE ProcessHandle,
//                       IN ACCESS_MASK DesiredAccess,
//                       IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     /* Call the internal API */
//     return PspCreateProcess(ProcessHandle,
//                             DesiredAccess,
//                             ObjectAttributes,
//                             NULL,
//                             0,
//                             NULL,
//                             NULL,
//                             NULL,
//                             FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsLookupProcessByProcessId(IN HANDLE ProcessId,
//                            OUT PEPROCESS *Process)
// {
//     PHANDLE_TABLE_ENTRY CidEntry;
//     PEPROCESS FoundProcess;
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG, "ProcessId: %p\n", ProcessId);
//     KeEnterCriticalRegion();
//
//     /* Get the CID Handle Entry */
//     CidEntry = ExMapHandleToPointer(PspCidTable, ProcessId);
//     if (CidEntry)
//     {
//         /* Get the Process */
//         FoundProcess = CidEntry->Object;
//
//         /* Make sure it's really a process */
//         if (FoundProcess->Pcb.Header.Type == ProcessObject)
//         {
//             /* Safe Reference and return it */
//             if (ObReferenceObjectSafe(FoundProcess))
//             {
//                 *Process = FoundProcess;
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
// NTSTATUS
// NTAPI
// PsLookupProcessThreadByCid(IN PCLIENT_ID Cid,
//                            OUT PEPROCESS *Process OPTIONAL,
//                            OUT PETHREAD *Thread)
// {
//     PHANDLE_TABLE_ENTRY CidEntry;
//     PETHREAD FoundThread;
//     NTSTATUS Status = STATUS_INVALID_CID;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG, "Cid: %p\n", Cid);
//     KeEnterCriticalRegion();
//
//     /* Get the CID Handle Entry */
//     CidEntry = ExMapHandleToPointer(PspCidTable, Cid->UniqueThread);
//     if (CidEntry)
//     {
//         /* Get the Process */
//         FoundThread = CidEntry->Object;
//
//         /* Make sure it's really a thread and this process' */
//         if ((FoundThread->Tcb.Header.Type == ThreadObject) &&
//             (FoundThread->Cid.UniqueProcess == Cid->UniqueProcess))
//         {
//             /* Safe Reference and return it */
//             if (ObReferenceObjectSafe(FoundThread))
//             {
//                 *Thread = FoundThread;
//                 Status = STATUS_SUCCESS;
//
//                 /* Check if we should return the Process too */
//                 if (Process)
//                 {
//                     /* Return it and reference it */
//                     *Process = FoundThread->ThreadsProcess;
//                     ObReferenceObject(*Process);
//                 }
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
// LARGE_INTEGER
// NTAPI
// PsGetProcessExitTime(VOID)
// {
//     return PsGetCurrentProcess()->ExitTime;
// }
//
// /*
//  * @implemented
//  */
// LONGLONG
// NTAPI
// PsGetProcessCreateTimeQuadPart(PEPROCESS Process)
// {
//     return Process->CreateTime.QuadPart;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetProcessDebugPort(PEPROCESS Process)
// {
//     return Process->DebugPort;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsGetProcessExitProcessCalled(PEPROCESS Process)
// {
//     return (BOOLEAN)Process->ProcessExiting;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsGetProcessExitStatus(PEPROCESS Process)
// {
//     return Process->ExitStatus;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetProcessId(PEPROCESS Process)
// {
//     return (HANDLE)Process->UniqueProcessId;
// }
//
// /*
//  * @implemented
//  */
// LPSTR
// NTAPI
// PsGetProcessImageFileName(PEPROCESS Process)
// {
//     return (LPSTR)Process->ImageFileName;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetProcessInheritedFromUniqueProcessId(PEPROCESS Process)
// {
//     return Process->InheritedFromUniqueProcessId;
// }
//
// /*
//  * @implemented
//  */
// PEJOB
// NTAPI
// PsGetProcessJob(PEPROCESS Process)
// {
//     return Process->Job;
// }
//
// /*
//  * @implemented
//  */
// PPEB
// NTAPI
// PsGetProcessPeb(PEPROCESS Process)
// {
//     return Process->Peb;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetProcessPriorityClass(PEPROCESS Process)
// {
//     return Process->PriorityClass;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// NTAPI
// PsGetCurrentProcessId(VOID)
// {
//     return (HANDLE)PsGetCurrentProcess()->UniqueProcessId;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetCurrentProcessSessionId(VOID)
// {
//     return MmGetSessionId(PsGetCurrentProcess());
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetProcessSectionBaseAddress(PEPROCESS Process)
// {
//     return Process->SectionBaseAddress;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetProcessSecurityPort(PEPROCESS Process)
// {
//     return Process->SecurityPort;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetProcessSessionId(IN PEPROCESS Process)
// {
//     return MmGetSessionId(Process);
// }
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// PsGetProcessSessionIdEx(IN PEPROCESS Process)
// {
//     return MmGetSessionIdEx(Process);
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetCurrentProcessWin32Process(VOID)
// {
//     return PsGetCurrentProcess()->Win32Process;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetProcessWin32Process(PEPROCESS Process)
// {
//     return Process->Win32Process;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// PsGetProcessWin32WindowStation(PEPROCESS Process)
// {
//     return Process->Win32WindowStation;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsIsProcessBeingDebugged(PEPROCESS Process)
// {
//     return Process->DebugPort != NULL;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsIsSystemProcess(IN PEPROCESS Process)
// {
//     /* Return if this is the System Process */
//     return Process == PsInitialSystemProcess;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsSetProcessPriorityClass(PEPROCESS Process,
//                           ULONG PriorityClass)
// {
//     Process->PriorityClass = (UCHAR)PriorityClass;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsSetProcessSecurityPort(PEPROCESS Process,
//                          PVOID SecurityPort)
// {
//     Process->SecurityPort = SecurityPort;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PsSetProcessWin32Process(
//     _Inout_ PEPROCESS Process,
//     _In_opt_ PVOID Win32Process,
//     _In_opt_ PVOID OldWin32Process)
// {
//     NTSTATUS Status;
//
//     /* Assume success */
//     Status = STATUS_SUCCESS;
//
//     /* Lock the process */
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&Process->ProcessLock);
//
//     /* Check if we set a new win32 process */
//     if (Win32Process != NULL)
//     {
//         /* Check if the process is in the right state */
//         if (((Process->Flags & PSF_PROCESS_DELETE_BIT) == 0) &&
//             (Process->Win32Process == NULL))
//         {
//             /* Set the new win32 process */
//             Process->Win32Process = Win32Process;
//         }
//         else
//         {
//             /* Otherwise fail */
//             Status = STATUS_PROCESS_IS_TERMINATING;
//         }
//     }
//     else
//     {
//         /* Reset the win32 process, did the caller specify the correct old value? */
//         if (Process->Win32Process == OldWin32Process)
//         {
//             /* Yes, so reset the win32 process to NULL */
//             Process->Win32Process = NULL;
//         }
//         else
//         {
//             /* Otherwise fail */
//             Status = STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     /* Unlock the process */
//     ExReleasePushLockExclusive(&Process->ProcessLock);
//     KeLeaveCriticalRegion();
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsSetProcessWindowStation(PEPROCESS Process,
//                           PVOID WindowStation)
// {
//     Process->Win32WindowStation = WindowStation;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// PsSetProcessPriorityByClass(IN PEPROCESS Process,
//                             IN PSPROCESSPRIORITYMODE Type)
// {
//     UCHAR Quantum;
//     ULONG Priority;
//     PSTRACE(PS_PROCESS_DEBUG, "Process: %p Type: %lx\n", Process, Type);
//
//     /* Compute quantum and priority */
//     Priority = PspComputeQuantumAndPriority(Process, Type, &Quantum);
//
//     /* Set them */
//     KeSetPriorityAndQuantumProcess(&Process->Pcb, Priority, Quantum);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateProcessEx(OUT PHANDLE ProcessHandle,
//                   IN ACCESS_MASK DesiredAccess,
//                   IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                   IN HANDLE ParentProcess,
//                   IN ULONG Flags,
//                   IN HANDLE SectionHandle OPTIONAL,
//                   IN HANDLE DebugPort OPTIONAL,
//                   IN HANDLE ExceptionPort OPTIONAL,
//                   IN BOOLEAN InJob)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG,
//             "ParentProcess: %p Flags: %lx\n", ParentProcess, Flags);
//
//     /* Check if we came from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe process handle */
//             ProbeForWriteHandle(ProcessHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Make sure there's a parent process */
//     if (!ParentProcess)
//     {
//         /* Can't create System Processes like this */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//     else
//     {
//         /* Create a user Process */
//         Status = PspCreateProcess(ProcessHandle,
//                                   DesiredAccess,
//                                   ObjectAttributes,
//                                   ParentProcess,
//                                   Flags,
//                                   SectionHandle,
//                                   DebugPort,
//                                   ExceptionPort,
//                                   InJob);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateProcess(OUT PHANDLE ProcessHandle,
//                 IN ACCESS_MASK DesiredAccess,
//                 IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                 IN HANDLE ParentProcess,
//                 IN BOOLEAN InheritObjectTable,
//                 IN HANDLE SectionHandle OPTIONAL,
//                 IN HANDLE DebugPort OPTIONAL,
//                 IN HANDLE ExceptionPort OPTIONAL)
// {
//     ULONG Flags = 0;
//     PSTRACE(PS_PROCESS_DEBUG,
//             "Parent: %p Attributes: %p\n", ParentProcess, ObjectAttributes);
//
//     /* Set new-style flags */
//     if ((ULONG_PTR)SectionHandle & 1) Flags |= PROCESS_CREATE_FLAGS_BREAKAWAY;
//     if ((ULONG_PTR)DebugPort & 1) Flags |= PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT;
//     if (InheritObjectTable) Flags |= PROCESS_CREATE_FLAGS_INHERIT_HANDLES;
//
//     /* Call the new API */
//     return NtCreateProcessEx(ProcessHandle,
//                              DesiredAccess,
//                              ObjectAttributes,
//                              ParentProcess,
//                              Flags,
//                              SectionHandle,
//                              DebugPort,
//                              ExceptionPort,
//                              FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtOpenProcess(OUT PHANDLE ProcessHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN POBJECT_ATTRIBUTES ObjectAttributes,
//               IN PCLIENT_ID ClientId)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     CLIENT_ID SafeClientId;
//     ULONG Attributes = 0;
//     HANDLE hProcess;
//     BOOLEAN HasObjectName = FALSE;
//     PETHREAD Thread = NULL;
//     PEPROCESS Process = NULL;
//     NTSTATUS Status;
//     ACCESS_STATE AccessState;
//     AUX_ACCESS_DATA AuxData;
//     PAGED_CODE();
//     PSTRACE(PS_PROCESS_DEBUG,
//             "ClientId: %p Attributes: %p\n", ClientId, ObjectAttributes);
//
//     /* Check if we were called from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the thread handle */
//             ProbeForWriteHandle(ProcessHandle);
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
//                                  &PsProcessType->TypeInfo.GenericMapping);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if this is a debugger */
//     if (SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode))
//     {
//         /* Did he want full access? */
//         if (AccessState.RemainingDesiredAccess & MAXIMUM_ALLOWED)
//         {
//             /* Give it to him */
//             AccessState.PreviouslyGrantedAccess |= PROCESS_ALL_ACCESS;
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
//                                     PsProcessType,
//                                     PreviousMode,
//                                     &AccessState,
//                                     0,
//                                     NULL,
//                                     &hProcess);
//
//         /* Get rid of the access state */
//         SeDeleteAccessState(&AccessState);
//     }
//     else if (ClientId)
//     {
//         /* Open by Thread ID */
//         if (ClientId->UniqueThread)
//         {
//             /* Get the Process */
//             Status = PsLookupProcessThreadByCid(ClientId, &Process, &Thread);
//         }
//         else
//         {
//             /* Get the Process */
//             Status = PsLookupProcessByProcessId(ClientId->UniqueProcess,
//                                                 &Process);
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
//         /* Open the Process Object */
//         Status = ObOpenObjectByPointer(Process,
//                                        Attributes,
//                                        &AccessState,
//                                        0,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        &hProcess);
//
//         /* Delete the access state */
//         SeDeleteAccessState(&AccessState);
//
//         /* Dereference the thread if we used it */
//         if (Thread) ObDereferenceObject(Thread);
//
//         /* Dereference the Process */
//         ObDereferenceObject(Process);
//     }
//     else
//     {
//         /* neither an object name nor a client id was passed */
//         return STATUS_INVALID_PARAMETER_MIX;
//     }
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Use SEH for write back */
//         _SEH2_TRY
//         {
//             /* Write back the handle */
//             *ProcessHandle = hProcess;
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
