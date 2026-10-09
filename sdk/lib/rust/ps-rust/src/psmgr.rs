use ntkrnl_sys::ddk::*;
// ke globals across the crate boundary as link symbols (#[no_mangle] in
// ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeFeatureBits: u64;
}
use ntkrnl_sys::base::{STATUS_SUCCESS, PAGE_READWRITE, PAGE_SHIFT};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::mm::{NonPagedPool, PagedPool};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

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


// PS-subsystem storage moved to ps/ps_globals.rs (was in
// kernel_c_helpers.c). Pull the new defs into scope so this file's
// callers don't need to qualify them.
pub use crate::ps_globals::{
    KeUserPopEntrySListEnd, KeUserPopEntrySListFault, KeUserPopEntrySListResume,
    PsInitialSystemProcess, PsIdleProcess, PsMaximumWorkingSet, PsMinimumWorkingSet,
    PsNtDllPathName, PspBootAccessToken, PspCidTable, PspDefaultNonPagedLimit,
    PspDefaultPagedLimit, PspDefaultPagefileLimit, PspDoingGiveBacks,
    PspInitialSystemProcessHandle, PspProcessMapping, PspSystemDllBase,
    PspSystemDllEntryPoint, PspSystemDllSection, PspThreadMapping,
};

extern "system" {
    static mut ExpInitializationPhase: ULONG;
    static mut PsActiveProcessHead: LIST_ENTRY;
    static mut PspActiveProcessMutex: KGUARDED_MUTEX;
    static mut PspReaperWorkItem: _WORK_QUEUE_ITEM;
    static mut PsProcessType: *mut _OBJECT_TYPE;
    static mut PsThreadType: *mut _OBJECT_TYPE;
    static mut PsJobType: *mut _OBJECT_TYPE;
    static mut PspJobMapping: _GENERIC_MAPPING;
    static mut PspThreadNotifyRoutine: [_EX_CALLBACK; 64];
    static mut PspProcessNotifyRoutine: [_EX_CALLBACK; 64];
    static mut PspLoadImageNotifyRoutine: [_EX_CALLBACK; 64];
    static mut PsRawPrioritySeparation: ULONG;
    static mut NtMajorVersion: ULONG;
    static mut NtMinorVersion: ULONG;
    static mut NtBuildNumber: ULONG;
    static mut CmCSDVersionString: _UNICODE_STRING;
    static mut KeUserApcDispatcher: PVOID;
    static mut KeUserExceptionDispatcher: PVOID;
    static mut KeUserCallbackDispatcher: PVOID;
    static mut KeRaiseUserExceptionDispatcher: PVOID;
    static mut SharedUserData: *mut _KUSER_SHARED_DATA;
    static mut PsNtosImageBase: ULONG_PTR;
    static mut MmSectionObjectType: *mut _OBJECT_TYPE;

    fn KdUpdateDataBlock();
    fn KeBugCheckEx(BugCheckCode: ULONG, P1: ULONG_PTR, P2: ULONG_PTR, P3: ULONG_PTR, P4: ULONG_PTR) -> !;
}

extern "C" {
    fn strcpy(dest: PCHAR, src: PCSTR) -> PCHAR;
    fn ARM64_UART_TRACE(msg: PCSTR);
    fn ARM64_UART_TRACE_HEX(prefix: PCSTR, val: ULONG_PTR);
}

extern "system" {
    fn Phase1Initialization(Context: PVOID);
}

static mut PspWorkingSetChangeHead: PspWorkingSetChangeHeadStruct = PspWorkingSetChangeHeadStruct {
    List: LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() },
    Lock: unsafe { core::mem::zeroed() },
};

#[repr(C)]
struct PspWorkingSetChangeHeadStruct {
    List: LIST_ENTRY,
    Lock: KGUARDED_MUTEX,
}

const PSP_MAX_CREATE_THREAD_NOTIFY: usize = 64;
const PSP_MAX_CREATE_PROCESS_NOTIFY: usize = 64;
const PSP_MAX_LOAD_IMAGE_NOTIFY: usize = 64;

const KF_FAST_SYSCALL: ULONG = 0x00001000;

const PROCESS1_INITIALIZATION_FAILED: ULONG = 0x6B;
const UNEXPECTED_INITIALIZATION_CALL: ULONG = 0x33;

const FILE_READ_ACCESS: ACCESS_MASK = 0x0001;
const FILE_SHARE_READ: ULONG = 0x00000001;
const PAGE_EXECUTE: ULONG = 0x10;
const SEC_IMAGE: ULONG = 0x01000000;
const SECTION_ALL_ACCESS: ACCESS_MASK = 0x000F001F;
const MAXULONG: ULONG = 0xFFFFFFFF;
const TAG_SEPA: ULONG = 0x61506553;
const ViewShare: ULONG = 1;

const MmMediumSystem: MM_SYSTEMSIZE = _MM_SYSTEM_SIZE_MmMediumSystem;
const MmLargeSystem: MM_SYSTEMSIZE = _MM_SYSTEM_SIZE_MmLargeSystem;

const OptionOk: ULONG = 0x01;
const STATUS_IMAGE_CHECKSUM_MISMATCH: NTSTATUS = 0xC0000221u32 as i32;
const STATUS_INVALID_IMAGE_PROTECT: NTSTATUS = 0xC0000130_u32 as i32;
const STATUS_PROCEDURE_NOT_FOUND: NTSTATUS = 0xC000007Au32 as i32;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018u32 as i32;

const STANDARD_RIGHTS_READ: ACCESS_MASK = 0x00020000;
const STANDARD_RIGHTS_WRITE: ACCESS_MASK = 0x00020000;
const STANDARD_RIGHTS_EXECUTE: ACCESS_MASK = 0x00020000;
const SYNCHRONIZE: ACCESS_MASK = 0x00100000;
const PROCESS_QUERY_INFORMATION: ACCESS_MASK = 0x0400;
const PROCESS_VM_READ: ACCESS_MASK = 0x0010;
const PROCESS_CREATE_PROCESS: ACCESS_MASK = 0x0080;
const PROCESS_CREATE_THREAD: ACCESS_MASK = 0x0002;
const PROCESS_VM_OPERATION: ACCESS_MASK = 0x0008;
const PROCESS_VM_WRITE: ACCESS_MASK = 0x0020;
const PROCESS_DUP_HANDLE: ACCESS_MASK = 0x0040;
const PROCESS_TERMINATE: ACCESS_MASK = 0x0001;
const PROCESS_SET_QUOTA: ACCESS_MASK = 0x0100;
const PROCESS_SET_INFORMATION: ACCESS_MASK = 0x0200;
const PROCESS_SUSPEND_RESUME: ACCESS_MASK = 0x0800;
const PROCESS_ALL_ACCESS: ACCESS_MASK = 0x001F0FFF;

const THREAD_GET_CONTEXT: ACCESS_MASK = 0x0008;
const THREAD_QUERY_INFORMATION: ACCESS_MASK = 0x0040;
const THREAD_TERMINATE: ACCESS_MASK = 0x0001;
const THREAD_SUSPEND_RESUME: ACCESS_MASK = 0x0002;
const THREAD_ALERT: ACCESS_MASK = 0x0004;
const THREAD_SET_INFORMATION: ACCESS_MASK = 0x0020;
const THREAD_SET_CONTEXT: ACCESS_MASK = 0x0010;
const THREAD_ALL_ACCESS: ACCESS_MASK = 0x001F03FF;

const JOB_OBJECT_ALL_ACCESS: ACCESS_MASK = 0x001F001F;

// ReactOS uses 7 for ALL architectures (3 low bits), matching C #define.
const MAX_FAST_REFS: usize = 7;

unsafe fn PspLookupSystemDllEntryPoint(
    name: PCSTR,
    entry_point: *mut PVOID,
) -> NTSTATUS {
    RtlpFindExportedRoutineByName(
        PspSystemDllBase,
        name,
        entry_point,
        core::ptr::null_mut(),
        STATUS_PROCEDURE_NOT_FOUND,
    )
}

unsafe fn PspLookupKernelUserEntryPoints() -> NTSTATUS {
    let mut status: NTSTATUS;

    status = PspLookupSystemDllEntryPoint(
        b"KiUserApcDispatcher\0".as_ptr() as PCSTR,
        &raw mut KeUserApcDispatcher,
    );
    if status < 0 {
        return status;
    }

    status = PspLookupSystemDllEntryPoint(
        b"KiUserExceptionDispatcher\0".as_ptr() as PCSTR,
        &raw mut KeUserExceptionDispatcher,
    );
    if status < 0 {
        return status;
    }

    status = PspLookupSystemDllEntryPoint(
        b"KiUserCallbackDispatcher\0".as_ptr() as PCSTR,
        &raw mut KeUserCallbackDispatcher,
    );
    if status < 0 {
        return status;
    }

    status = PspLookupSystemDllEntryPoint(
        b"KiRaiseUserExceptionDispatcher\0".as_ptr() as PCSTR,
        &raw mut KeRaiseUserExceptionDispatcher,
    );
    if status < 0 {
        return status;
    }

    // x86-ONLY, per ntdll.spec itself:
    //     @ stdcall -arch=i386   ExpInterlockedPopEntrySListEnd()
    //     @ stdcall -arch=x86_64 ExpInterlockedPopEntrySListEnd()
    // There is no arm64 row, and the built arm64 ntdll.dll exports none of the
    // three (measured: 0 occurrences, vs 3 each for the four dispatchers above).
    // They exist to let the kernel fix up a fault *inside* the x86 user-mode
    // SLIST pop, whose read-Next-then-CAS window arm64's LL/SC/LSE form does not
    // have; the only consumer in the tree is ke/i386/traphdlr.rs.
    //
    // Looking them up unconditionally therefore failed on EVERY arm64 boot with
    // STATUS_PROCEDURE_NOT_FOUND, and because this function returns early, that
    // propagated out of PsLocateSystemDll as a whole-system-DLL-init failure.
    // x86 would have bugchecked PROCESS1_INITIALIZATION_FAILED/8; arm64 swallowed
    // it and booted on ("continuing on ARM64"), so the escape hatch was
    // PERMANENTLY exercised and could never signal a real failure. Gating the
    // lookups to the arches that export them makes arm64 return SUCCESS here, and
    // restores that hatch to meaning something.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        status = PspLookupSystemDllEntryPoint(
            b"ExpInterlockedPopEntrySListEnd\0".as_ptr() as PCSTR,
            &raw mut KeUserPopEntrySListEnd,
        );
        if status < 0 {
            return status;
        }

        status = PspLookupSystemDllEntryPoint(
            b"ExpInterlockedPopEntrySListFault\0".as_ptr() as PCSTR,
            &raw mut KeUserPopEntrySListFault,
        );
        if status < 0 {
            return status;
        }

        status = PspLookupSystemDllEntryPoint(
            b"ExpInterlockedPopEntrySListResume\0".as_ptr() as PCSTR,
            &raw mut KeUserPopEntrySListResume,
        );
        if status < 0 {
            return status;
        }
    }

    #[cfg(target_arch = "x86")]
    {
        if (KeFeatureBits & KF_FAST_SYSCALL as u64) != 0 {
            (*SharedUserData).SystemCall = PsNtosImageBase >> (PAGE_SHIFT + 1);
            status = PspLookupSystemDllEntryPoint(
                b"KiFastSystemCall\0".as_ptr() as PCSTR,
                core::ptr::addr_of_mut!((*SharedUserData).SystemCall) as *mut PVOID,
            );
            if status < 0 {
                return status;
            }

            status = PspLookupSystemDllEntryPoint(
                b"KiFastSystemCallRet\0".as_ptr() as PCSTR,
                core::ptr::addr_of_mut!((*SharedUserData).SystemCallReturn) as *mut PVOID,
            );
            if status < 0 {
                return status;
            }
        } else {
            status = PspLookupSystemDllEntryPoint(
                b"KiIntSystemCall\0".as_ptr() as PCSTR,
                core::ptr::addr_of_mut!((*SharedUserData).SystemCall) as *mut PVOID,
            );
            if status < 0 {
                return status;
            }
        }

        (*SharedUserData).TestRetInstruction = 0xC3;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PspMapSystemDll(
    process: PEPROCESS,
    dll_base: *mut PVOID,
    use_large_pages: BOOLEAN,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let dll_base = opaque_ptr(dll_base);
    let mut offset = LARGE_INTEGER { QuadPart: 0 };
    let mut view_size: SIZE_T = 0;
    let mut image_base: PVOID = core::ptr::null_mut();

    let mut status = MmMapViewOfSection(
        PspSystemDllSection,
        process,
        &mut image_base,
        0,
        0,
        &mut offset,
        &mut view_size,
        ViewShare,
        0,
        PAGE_READWRITE,
    );
    if status != STATUS_SUCCESS {
        status = STATUS_CONFLICTING_ADDRESSES;
    }

    if !dll_base.is_null() {
        *dll_base = image_base;
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn PsLocateSystemDll() -> NTSTATUS {
    let mut object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &raw mut PsNtDllPathName as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut file_handle: HANDLE = core::ptr::null_mut();
    let mut section_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut hard_error_parameters: ULONG_PTR;
    let mut hard_error_response: ULONG = 0;

    status = ZwOpenFile(
        &mut file_handle,
        FILE_READ_ACCESS,
        &mut object_attributes,
        &mut io_status_block,
        FILE_SHARE_READ,
        0,
    );
    if status < 0 {
        // psmgr.c bugchecks here on every arch. arm64 used to return the status and
        // boot on without a system DLL; the C's only arm64 leniency is in
        // PspInitializeSystemDll (subcodes 7/8), not here. The UART line stays: the
        // bugcheck banner alone is easy to miss on arm64's console.
        #[cfg(target_arch = "aarch64")]
        ARM64_UART_TRACE_HEX(
            b"[PS] PsLocateSystemDll FAILED subcode=2 status=\0".as_ptr() as PCSTR,
            status as ULONG_PTR,
        );
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 2, 0, 0);
    }

    status = MmCheckSystemImage(file_handle);
    if status == STATUS_IMAGE_CHECKSUM_MISMATCH || status == STATUS_INVALID_IMAGE_PROTECT {
        hard_error_parameters = &raw mut PsNtDllPathName as *mut _UNICODE_STRING as ULONG_PTR;
        NtRaiseHardError(
            status,
            1,
            1,
            &mut hard_error_parameters,
            OptionOk,
            &mut hard_error_response,
        );
        return status;
    }

    status = ZwCreateSection(
        &mut section_handle,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_EXECUTE,
        SEC_IMAGE,
        file_handle,
    );
    ZwClose(file_handle);
    if status < 0 {
        // psmgr.c bugchecks here on every arch. arm64 used to return the status and
        // boot on without a system DLL; the C's only arm64 leniency is in
        // PspInitializeSystemDll (subcodes 7/8), not here. The UART line stays: the
        // bugcheck banner alone is easy to miss on arm64's console.
        #[cfg(target_arch = "aarch64")]
        ARM64_UART_TRACE_HEX(
            b"[PS] PsLocateSystemDll FAILED subcode=3 status=\0".as_ptr() as PCSTR,
            status as ULONG_PTR,
        );
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 3, 0, 0);
    }

    status = ObReferenceObjectByHandle(
        section_handle,
        SECTION_ALL_ACCESS,
        MmSectionObjectType,
        KernelMode as i8,
        &raw mut PspSystemDllSection,
        core::ptr::null_mut(),
    );
    ZwClose(section_handle);
    if status < 0 {
        // psmgr.c bugchecks here on every arch. arm64 used to return the status and
        // boot on without a system DLL; the C's only arm64 leniency is in
        // PspInitializeSystemDll (subcodes 7/8), not here. The UART line stays: the
        // bugcheck banner alone is easy to miss on arm64's console.
        #[cfg(target_arch = "aarch64")]
        ARM64_UART_TRACE_HEX(
            b"[PS] PsLocateSystemDll FAILED subcode=4 status=\0".as_ptr() as PCSTR,
            status as ULONG_PTR,
        );
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 4, 0, 0);
    }

    status = PspMapSystemDll(PsGetCurrentProcess() as PEPROCESS, &raw mut PspSystemDllBase, 0u8);
    if status < 0 {
        // psmgr.c bugchecks here on every arch. arm64 used to return the status and
        // boot on without a system DLL; the C's only arm64 leniency is in
        // PspInitializeSystemDll (subcodes 7/8), not here. The UART line stays: the
        // bugcheck banner alone is easy to miss on arm64's console.
        #[cfg(target_arch = "aarch64")]
        ARM64_UART_TRACE_HEX(
            b"[PS] PsLocateSystemDll FAILED subcode=5 status=\0".as_ptr() as PCSTR,
            status as ULONG_PTR,
        );
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 5, 0, 0);
    }

    status
}

unsafe fn PspInitializeSystemDll() -> NTSTATUS {
    let mut status: NTSTATUS;

    status = PspLookupSystemDllEntryPoint(
        b"LdrInitializeThunk\0".as_ptr() as PCSTR,
        &raw mut PspSystemDllEntryPoint,
    );
    if status < 0 {
        // psmgr.c's own `#ifdef _M_ARM64 return Status;` — the one arm64 leniency the C
        // has in this file. Traced, since the return is otherwise silent.
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(
                b"[PS] PspInitializeSystemDll FAILED subcode=7 status=\0".as_ptr() as PCSTR,
                status as ULONG_PTR,
            );
            return status;
        }
        #[cfg(not(target_arch = "aarch64"))]
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 7, 0, 0);
    }

    status = PspLookupKernelUserEntryPoints();
    if status < 0 {
        // psmgr.c's own `#ifdef _M_ARM64 return Status;` — the one arm64 leniency the C
        // has in this file. Traced, since the return is otherwise silent.
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE_HEX(
                b"[PS] PspInitializeSystemDll FAILED subcode=8 status=\0".as_ptr() as PCSTR,
                status as ULONG_PTR,
            );
            return status;
        }
        #[cfg(not(target_arch = "aarch64"))]
        KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, status as ULONG_PTR, 8, 0, 0);
    }

    KdUpdateDataBlock();

    status
}

unsafe fn PspInitPhase1() -> BOOLEAN {
    if PspInitializeSystemDll() < 0 {
        return 0u8;
    }
    1u8
}

unsafe fn PspInitPhase0(loader_block: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let mut status: NTSTATUS;
    let mut object_attributes: _OBJECT_ATTRIBUTES;
    let mut sys_thread_handle: HANDLE = core::ptr::null_mut();
    let mut sys_thread: PETHREAD = core::ptr::null_mut();
    let system_size: MM_SYSTEMSIZE;
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_type_initializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut i: ULONG;

    ARM64_UART_TRACE(b"[PS] PspInitPhase0 enter\r\n\0".as_ptr() as PCSTR);

    system_size = MmQuerySystemSize();

    PspDefaultPagefileLimit = MAXULONG;
    match system_size {
        MmMediumSystem => {
            PsMinimumWorkingSet += 10;
            PsMaximumWorkingSet += 100;
            PsMinimumWorkingSet += 30;
            PsMaximumWorkingSet += 300;
        }
        MmLargeSystem => {
            PsMinimumWorkingSet += 30;
            PsMaximumWorkingSet += 300;
        }
        _ => {}
    }

    for i in 0..PSP_MAX_CREATE_THREAD_NOTIFY {
        ExInitializeCallBack(core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i]));
    }
    for i in 0..PSP_MAX_CREATE_PROCESS_NOTIFY {
        ExInitializeCallBack(core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i]));
    }
    for i in 0..PSP_MAX_LOAD_IMAGE_NOTIFY {
        ExInitializeCallBack(core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i]));
    }

    PsChangeQuantumTable(0u8, PsRawPrioritySeparation);

    if PspDefaultPagedLimit == 0 {
        PspDefaultPagedLimit = 0;
    }
    if PspDefaultNonPagedLimit == 0 {
        PspDefaultNonPagedLimit = 0;
    }
    if PspDefaultNonPagedLimit == 0 && PspDefaultPagedLimit == 0 {
        PspDoingGiveBacks = 1u8;
    } else {
        PspDoingGiveBacks = 0u8;
    }

    PspDefaultPagedLimit <<= 20;
    PspDefaultNonPagedLimit <<= 20;
    if PspDefaultPagefileLimit != MAXULONG {
        PspDefaultPagefileLimit <<= 20;
    }

    InitializeListHead(&raw mut PsActiveProcessHead);
    KeInitializeGuardedMutex(&raw mut PspActiveProcessMutex);
    ARM64_UART_TRACE_HEX(
        b"[PS] PspActiveProcessMutex addr=\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(PspActiveProcessMutex) as ULONG_PTR,
    );

    PsIdleProcess = PsGetCurrentProcess() as PEPROCESS;

    (*(PsIdleProcess as *mut _EPROCESS)).ProcessLock.__bindgen_anon_1.Value = 0;
    ExInitializeRundownProtection(
        core::ptr::addr_of_mut!((*(PsIdleProcess as *mut _EPROCESS)).RundownProtect)
    );

    InitializeListHead(core::ptr::addr_of_mut!((*(PsIdleProcess as *mut _EPROCESS)).ThreadListHead) as PLIST_ENTRY);

    (*(PsIdleProcess as *mut _EPROCESS)).Pcb.KernelTime = 0;

    core::ptr::write_bytes(&mut object_type_initializer as *mut _ as *mut u8, 0, core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>());
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT;
    object_type_initializer.InvalidAttributes = OBJ_PERMANENT | OBJ_EXCLUSIVE | OBJ_OPENIF;
    object_type_initializer.PoolType = NonPagedPool as u32;
    object_type_initializer.SecurityRequired = 1u8;

    RtlInitUnicodeString(&mut name, w!("Process").as_ptr() as *const u16);
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_EPROCESS>() as ULONG;
    object_type_initializer.GenericMapping = PspProcessMapping;
    object_type_initializer.ValidAccessMask = PROCESS_ALL_ACCESS;
    object_type_initializer.DeleteProcedure = Some(PspDeleteProcess);
    ObCreateObjectType(&mut name, &mut object_type_initializer, core::ptr::null_mut(), &raw mut PsProcessType);

    RtlInitUnicodeString(&mut name, w!("Thread").as_ptr() as *const u16);
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT;
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_ETHREAD>() as ULONG;
    object_type_initializer.GenericMapping = PspThreadMapping;
    object_type_initializer.ValidAccessMask = THREAD_ALL_ACCESS;
    object_type_initializer.DeleteProcedure = Some(PspDeleteThread);
    ObCreateObjectType(&mut name, &mut object_type_initializer, core::ptr::null_mut(), &raw mut PsThreadType);

    RtlInitUnicodeString(&mut name, w!("Job").as_ptr() as *const u16);
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as USHORT;
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<_EJOB>() as ULONG;
    object_type_initializer.GenericMapping = PspJobMapping;
    object_type_initializer.InvalidAttributes = 0;
    object_type_initializer.ValidAccessMask = JOB_OBJECT_ALL_ACCESS;
    object_type_initializer.CloseProcedure = Some(crate::job::PspCloseJob);
    object_type_initializer.DeleteProcedure = Some(PspDeleteJob);
    ObCreateObjectType(&mut name, &mut object_type_initializer, core::ptr::null_mut(), &raw mut PsJobType);

    PspInitializeJobStructures();

    InitializeListHead(&raw mut PspWorkingSetChangeHead.List);
    KeInitializeGuardedMutex(&raw mut PspWorkingSetChangeHead.Lock);

    PspCidTable = ExCreateHandleTable(core::ptr::null_mut());
    if PspCidTable.is_null() {
        ARM64_UART_TRACE(b"[PS] FAIL: PspCidTable null\r\n\0".as_ptr() as PCSTR);
        return 0u8;
    }

    PspReaperWorkItem.WorkerRoutine = Some(PspReapRoutine);
    PspReaperWorkItem.Parameter = core::ptr::null_mut();
    PspReaperWorkItem.List.Flink = core::ptr::null_mut();

    PspBootAccessToken = ((*(PsIdleProcess as *mut _EPROCESS)).Token.__bindgen_anon_1.Value & !(MAX_FAST_REFS as ULONG_PTR)) as PTOKEN;

    object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    ARM64_UART_TRACE(b"[PS] Creating initial system process\r\n\0".as_ptr() as PCSTR);

    status = PspCreateProcess(
        &raw mut PspInitialSystemProcessHandle,
        PROCESS_ALL_ACCESS,
        &mut object_attributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
    );
    if status < 0 {
        ARM64_UART_TRACE_HEX(b"[PS] FAIL: PspCreateProcess status=\0".as_ptr() as PCSTR, status as ULONG_PTR);
        return 0u8;
    }

    ObReferenceObjectByHandle(
        PspInitialSystemProcessHandle,
        0,
        PsProcessType,
        KernelMode as i8,
        &raw mut PsInitialSystemProcess as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );

    ARM64_UART_TRACE(b"[PS] System process created\r\n\0".as_ptr() as PCSTR);

    strcpy(
        (*(PsIdleProcess as *mut _EPROCESS)).ImageFileName.as_mut_ptr(),
        b"Idle\0".as_ptr() as PCSTR,
    );
    strcpy(
        (*(PsInitialSystemProcess as *mut _EPROCESS)).ImageFileName.as_mut_ptr(),
        b"System\0".as_ptr() as PCSTR,
    );

    (*(PsInitialSystemProcess as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName =
        ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<OBJECT_NAME_INFORMATION>() as SIZE_T,
            TAG_SEPA,
        ) as *mut OBJECT_NAME_INFORMATION;
    if (*(PsInitialSystemProcess as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName.is_null() {
        ARM64_UART_TRACE(b"[PS] FAIL: SeAuditProcessCreationInfo null\r\n\0".as_ptr() as PCSTR);
        return 0u8;
    }

    core::ptr::write_bytes(
        (*(PsInitialSystemProcess as *mut _EPROCESS)).SeAuditProcessCreationInfo.ImageFileName as *mut u8,
        0,
        core::mem::size_of::<OBJECT_NAME_INFORMATION>(),
    );

    ARM64_UART_TRACE(b"[PS] Creating Phase1 thread\r\n\0".as_ptr() as PCSTR);

    status = PsCreateSystemThread(
        &mut sys_thread_handle,
        THREAD_ALL_ACCESS,
        &mut object_attributes,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::mem::transmute(Phase1Initialization as usize),
        loader_block as PVOID,
    );

    if status < 0 {
        ARM64_UART_TRACE_HEX(b"[PS] FAIL: PsCreateSystemThread status=\0".as_ptr() as PCSTR, status as ULONG_PTR);
        return 0u8;
    }

    ObReferenceObjectByHandle(
        sys_thread_handle,
        0,
        PsThreadType,
        KernelMode as i8,
        &mut sys_thread as *mut PETHREAD as *mut PVOID,
        core::ptr::null_mut(),
    );
    ObCloseHandle(sys_thread_handle, KernelMode as i8);

    ARM64_UART_TRACE(b"[PS] PspInitPhase0 done\r\n\0".as_ptr() as PCSTR);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn PsInitSystem(loader_block: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let loader_block = opaque_ptr(loader_block);
    match ExpInitializationPhase {
        0 => {
            crate::trace::init(loader_block);
            PspInitPhase0(loader_block)
        }
        1 => PspInitPhase1(),
        _ => {
            KeBugCheckEx(
                UNEXPECTED_INITIALIZATION_CALL,
                1,
                ExpInitializationPhase as ULONG_PTR,
                0,
                0,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsGetVersion(
    major_version: *mut ULONG,
    minor_version: *mut ULONG,
    build_number: *mut ULONG,
    csd_version: PUNICODE_STRING,
) -> BOOLEAN {
    let major_version = opaque_ptr(major_version);
    let minor_version = opaque_ptr(minor_version);
    let build_number = opaque_ptr(build_number);
    let csd_version = opaque_ptr(csd_version);
    if !major_version.is_null() {
        *major_version = NtMajorVersion;
    }
    if !minor_version.is_null() {
        *minor_version = NtMinorVersion;
    }
    if !build_number.is_null() {
        *build_number = NtBuildNumber & 0x3FFF;
    }

    if !csd_version.is_null() {
        (*csd_version).Length = CmCSDVersionString.Length;
        (*csd_version).MaximumLength = CmCSDVersionString.MaximumLength;
        (*csd_version).Buffer = CmCSDVersionString.Buffer;
    }

    if (NtBuildNumber >> 28) == 0xC { 1u8 } else { 0u8 }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ps/psmgr.c
//  * PURPOSE:         Process Manager: Initialization Code
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// extern ULONG ExpInitializationPhase;
//
// PVOID KeUserPopEntrySListEnd;
// PVOID KeUserPopEntrySListFault;
// PVOID KeUserPopEntrySListResume;
//
// GENERIC_MAPPING PspProcessMapping =
// {
//     STANDARD_RIGHTS_READ    | PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
//     STANDARD_RIGHTS_WRITE   | PROCESS_CREATE_PROCESS    | PROCESS_CREATE_THREAD   |
//     PROCESS_VM_OPERATION    | PROCESS_VM_WRITE          | PROCESS_DUP_HANDLE      |
//     PROCESS_TERMINATE       | PROCESS_SET_QUOTA         | PROCESS_SET_INFORMATION |
//     PROCESS_SUSPEND_RESUME,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     PROCESS_ALL_ACCESS
// };
//
// GENERIC_MAPPING PspThreadMapping =
// {
//     STANDARD_RIGHTS_READ    | THREAD_GET_CONTEXT      | THREAD_QUERY_INFORMATION,
//     STANDARD_RIGHTS_WRITE   | THREAD_TERMINATE        | THREAD_SUSPEND_RESUME    |
//     THREAD_ALERT            | THREAD_SET_INFORMATION  | THREAD_SET_CONTEXT,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     THREAD_ALL_ACCESS
// };
//
// PVOID PspSystemDllBase;
// PVOID PspSystemDllSection;
// PVOID PspSystemDllEntryPoint;
//
// UNICODE_STRING PsNtDllPathName =
//     RTL_CONSTANT_STRING(L"\\SystemRoot\\System32\\ntdll.dll");
//
// PHANDLE_TABLE PspCidTable;
//
// PEPROCESS PsInitialSystemProcess = NULL;
// PEPROCESS PsIdleProcess = NULL;
// HANDLE PspInitialSystemProcessHandle = NULL;
//
// ULONG PsMinimumWorkingSet, PsMaximumWorkingSet;
// struct
// {
//     LIST_ENTRY List;
//     KGUARDED_MUTEX Lock;
// } PspWorkingSetChangeHead;
// ULONG PspDefaultPagedLimit, PspDefaultNonPagedLimit, PspDefaultPagefileLimit;
// BOOLEAN PspDoingGiveBacks;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static CODE_SEG("INIT")
// NTSTATUS
// PspLookupSystemDllEntryPoint(
//     _In_ PCSTR Name,
//     _Out_ PVOID* EntryPoint)
// {
//     /* Call the internal API */
//     return RtlpFindExportedRoutineByName(PspSystemDllBase,
//                                          Name,
//                                          EntryPoint,
//                                          NULL,
//                                          STATUS_PROCEDURE_NOT_FOUND);
// }
//
// static CODE_SEG("INIT")
// NTSTATUS
// PspLookupKernelUserEntryPoints(VOID)
// {
//     NTSTATUS Status;
//
//     /* Get user-mode APC trampoline */
//     Status = PspLookupSystemDllEntryPoint("KiUserApcDispatcher",
//                                           &KeUserApcDispatcher);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get user-mode exception dispatcher */
//     Status = PspLookupSystemDllEntryPoint("KiUserExceptionDispatcher",
//                                           &KeUserExceptionDispatcher);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get user-mode callback dispatcher */
//     Status = PspLookupSystemDllEntryPoint("KiUserCallbackDispatcher",
//                                           &KeUserCallbackDispatcher);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get user-mode exception raise trampoline */
//     Status = PspLookupSystemDllEntryPoint("KiRaiseUserExceptionDispatcher",
//                                           &KeRaiseUserExceptionDispatcher);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get user-mode SLIST exception functions for page fault rollback race hack */
//     Status = PspLookupSystemDllEntryPoint("ExpInterlockedPopEntrySListEnd",
//                                           &KeUserPopEntrySListEnd);
//     if (!NT_SUCCESS(Status)) { DPRINT1("this not found\n"); return Status; }
//     Status = PspLookupSystemDllEntryPoint("ExpInterlockedPopEntrySListFault",
//                                           &KeUserPopEntrySListFault);
//     if (!NT_SUCCESS(Status)) { DPRINT1("this not found\n"); return Status; }
//     Status = PspLookupSystemDllEntryPoint("ExpInterlockedPopEntrySListResume",
//                                           &KeUserPopEntrySListResume);
//     if (!NT_SUCCESS(Status)) { DPRINT1("this not found\n"); return Status; }
//
//     /* On x86, there are multiple ways to do a system call, find the right stubs */
// #if defined(_X86_)
//     /* Check if this is a machine that supports SYSENTER */
//     if (KeFeatureBits & KF_FAST_SYSCALL)
//     {
//         /* Get user-mode sysenter stub */
//         SharedUserData->SystemCall = (PsNtosImageBase >> (PAGE_SHIFT + 1));
//         Status = PspLookupSystemDllEntryPoint("KiFastSystemCall",
//                                               (PVOID)&SharedUserData->
//                                               SystemCall);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Get user-mode sysenter return stub */
//         Status = PspLookupSystemDllEntryPoint("KiFastSystemCallRet",
//                                               (PVOID)&SharedUserData->
//                                               SystemCallReturn);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//     else
//     {
//         /* Get the user-mode interrupt stub */
//         Status = PspLookupSystemDllEntryPoint("KiIntSystemCall",
//                                               (PVOID)&SharedUserData->
//                                               SystemCall);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Set the test instruction */
//     SharedUserData->TestRetInstruction = 0xC3;
// #endif
//
//     /* Return the status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// PspMapSystemDll(IN PEPROCESS Process,
//                 IN PVOID *DllBase,
//                 IN BOOLEAN UseLargePages)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER Offset = {{0, 0}};
//     SIZE_T ViewSize = 0;
//     PVOID ImageBase = 0;
//
//     /* Map the System DLL */
//     Status = MmMapViewOfSection(PspSystemDllSection,
//                                 Process,
//                                 (PVOID*)&ImageBase,
//                                 0,
//                                 0,
//                                 &Offset,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0,
//                                 PAGE_READWRITE);
//     if (Status != STATUS_SUCCESS)
//     {
//         /* Normalize status code */
//         Status = STATUS_CONFLICTING_ADDRESSES;
//     }
//
//     /* Write the image base and return status */
//     if (DllBase) *DllBase = ImageBase;
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// PsLocateSystemDll(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle, SectionHandle;
//     NTSTATUS Status;
//     ULONG_PTR HardErrorParameters;
//     ULONG HardErrorResponse;
//
//     /* Locate and open NTDLL to determine ImageBase and LdrStartup */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &PsNtDllPathName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenFile(&FileHandle,
//                         FILE_READ_ACCESS,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ,
//                         0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 2, 0, 0);
//     }
//
//     /* Check if the image is valid */
//     Status = MmCheckSystemImage(FileHandle);
//     if (Status == STATUS_IMAGE_CHECKSUM_MISMATCH || Status == STATUS_INVALID_IMAGE_PROTECT)
//     {
//         /* Raise a hard error */
//         HardErrorParameters = (ULONG_PTR)&PsNtDllPathName;
//         NtRaiseHardError(Status,
//                          1,
//                          1,
//                          &HardErrorParameters,
//                          OptionOk,
//                          &HardErrorResponse);
//         return Status;
//     }
//
//     /* Create a section for NTDLL */
//     Status = ZwCreateSection(&SectionHandle,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              NULL,
//                              PAGE_EXECUTE,
//                              SEC_IMAGE,
//                              FileHandle);
//     ZwClose(FileHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 3, 0, 0);
//     }
//
//     /* Reference the Section */
//     Status = ObReferenceObjectByHandle(SectionHandle,
//                                        SECTION_ALL_ACCESS,
//                                        MmSectionObjectType,
//                                        KernelMode,
//                                        (PVOID*)&PspSystemDllSection,
//                                        NULL);
//     ZwClose(SectionHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 4, 0, 0);
//     }
//
//     /* Map it */
//     Status = PspMapSystemDll(PsGetCurrentProcess(), &PspSystemDllBase, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 5, 0, 0);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// PspInitializeSystemDll(VOID)
// {
//     NTSTATUS Status;
//
//     /* Get user-mode startup thunk */
//     Status = PspLookupSystemDllEntryPoint("LdrInitializeThunk",
//                                           &PspSystemDllEntryPoint);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         /* On ARM64 without ntdll.dll, this is expected */
//         return Status;
// #else
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 7, 0, 0);
// #endif
//     }
//
//     /* Get all the other entrypoints */
//     Status = PspLookupKernelUserEntryPoints();
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         return Status;
// #else
//         /* Failed, bugcheck */
//         KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, Status, 8, 0, 0);
// #endif
//     }
//
//     /* Let KD know we are done */
//     KdUpdateDataBlock();
//
//     /* Return status */
//     return Status;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// PspInitPhase1(VOID)
// {
//     /* Initialize the System DLL and return status of operation */
//     if (!NT_SUCCESS(PspInitializeSystemDll())) return FALSE;
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// PspInitPhase0(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE SysThreadHandle;
//     PETHREAD SysThread;
//     MM_SYSTEMSIZE SystemSize;
//     UNICODE_STRING Name;
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     ULONG i;
//
//     ARM64_UART_TRACE("[PS] PspInitPhase0 enter\r\n");
//
//     /* Get the system size */
//     SystemSize = MmQuerySystemSize();
//
//     /* Setup some memory options */
//     PspDefaultPagefileLimit = -1;
//     switch (SystemSize)
//     {
//         /* Medimum systems */
//         case MmMediumSystem:
//
//             /* Increase the WS sizes a bit */
//             PsMinimumWorkingSet += 10;
//             PsMaximumWorkingSet += 100;
//
//         /* Large systems */
//         case MmLargeSystem:
//
//             /* Increase the WS sizes a bit more */
//             PsMinimumWorkingSet += 30;
//             PsMaximumWorkingSet += 300;
//
//         /* Small and other systems */
//         default:
//             break;
//     }
//
//     /* Setup callbacks */
//     for (i = 0; i < PSP_MAX_CREATE_THREAD_NOTIFY; i++)
//     {
//         ExInitializeCallBack(&PspThreadNotifyRoutine[i]);
//     }
//     for (i = 0; i < PSP_MAX_CREATE_PROCESS_NOTIFY; i++)
//     {
//         ExInitializeCallBack(&PspProcessNotifyRoutine[i]);
//     }
//     for (i = 0; i < PSP_MAX_LOAD_IMAGE_NOTIFY; i++)
//     {
//         ExInitializeCallBack(&PspLoadImageNotifyRoutine[i]);
//     }
//
//     /* Setup the quantum table */
//     PsChangeQuantumTable(FALSE, PsRawPrioritySeparation);
//
//     /* Set quota settings */
//     if (!PspDefaultPagedLimit) PspDefaultPagedLimit = 0;
//     if (!PspDefaultNonPagedLimit) PspDefaultNonPagedLimit = 0;
//     if (!(PspDefaultNonPagedLimit) && !(PspDefaultPagedLimit))
//     {
//         /* Enable give-backs */
//         PspDoingGiveBacks = TRUE;
//     }
//     else
//     {
//         /* Disable them */
//         PspDoingGiveBacks = FALSE;
//     }
//
//     /* Now multiply limits by 1MB */
//     PspDefaultPagedLimit <<= 20;
//     PspDefaultNonPagedLimit <<= 20;
//     if (PspDefaultPagefileLimit != MAXULONG) PspDefaultPagefileLimit <<= 20;
//
//     /* Initialize the Active Process List */
//     InitializeListHead(&PsActiveProcessHead);
//     KeInitializeGuardedMutex(&PspActiveProcessMutex);
//
//     /* Get the idle process */
//     PsIdleProcess = PsGetCurrentProcess();
//
//     /* Setup the locks */
//     PsIdleProcess->ProcessLock.Value = 0;
//     ExInitializeRundownProtection(&PsIdleProcess->RundownProtect);
//
//     /* Initialize the thread list */
//     InitializeListHead(&PsIdleProcess->ThreadListHead);
//
//     /* Clear kernel time */
//     PsIdleProcess->Pcb.KernelTime = 0;
//
//     /* Initialize Object Initializer */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_PERMANENT |
//                                               OBJ_EXCLUSIVE |
//                                               OBJ_OPENIF;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//
//     /* Initialize the Process type */
//     RtlInitUnicodeString(&Name, L"Process");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(EPROCESS);
//     ObjectTypeInitializer.GenericMapping = PspProcessMapping;
//     ObjectTypeInitializer.ValidAccessMask = PROCESS_ALL_ACCESS;
//     ObjectTypeInitializer.DeleteProcedure = PspDeleteProcess;
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &PsProcessType);
//
//     /*  Initialize the Thread type  */
//     RtlInitUnicodeString(&Name, L"Thread");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(ETHREAD);
//     ObjectTypeInitializer.GenericMapping = PspThreadMapping;
//     ObjectTypeInitializer.ValidAccessMask = THREAD_ALL_ACCESS;
//     ObjectTypeInitializer.DeleteProcedure = PspDeleteThread;
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &PsThreadType);
//
//     /*  Initialize the Job type  */
//     RtlInitUnicodeString(&Name, L"Job");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(EJOB);
//     ObjectTypeInitializer.GenericMapping = PspJobMapping;
//     ObjectTypeInitializer.InvalidAttributes = 0;
//     ObjectTypeInitializer.ValidAccessMask = JOB_OBJECT_ALL_ACCESS;
//     ObjectTypeInitializer.DeleteProcedure = PspDeleteJob;
//     ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &PsJobType);
//
//     /* Initialize job structures external to this file */
//     PspInitializeJobStructures();
//
//     /* Initialize the Working Set data */
//     InitializeListHead(&PspWorkingSetChangeHead.List);
//     KeInitializeGuardedMutex(&PspWorkingSetChangeHead.Lock);
//
//     /* Create the CID Handle table */
//     PspCidTable = ExCreateHandleTable(NULL);
//     if (!PspCidTable) return FALSE;
//
//     /* FIXME: Initialize LDT/VDM support */
//
//     /* Setup the reaper */
//     ExInitializeWorkItem(&PspReaperWorkItem, PspReapRoutine, NULL);
//
//     /* Set the boot access token */
//     PspBootAccessToken = (PTOKEN)(PsIdleProcess->Token.Value & ~MAX_FAST_REFS);
//
//     /* Setup default object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL);
//
//     ARM64_UART_TRACE("[PS] Creating initial system process\r\n");
//
//     /* Create the Initial System Process */
//     Status = PspCreateProcess(&PspInitialSystemProcessHandle,
//                               PROCESS_ALL_ACCESS,
//                               &ObjectAttributes,
//                               0,
//                               FALSE,
//                               0,
//                               0,
//                               0,
//                               FALSE);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Get a reference to it */
//     ObReferenceObjectByHandle(PspInitialSystemProcessHandle,
//                               0,
//                               PsProcessType,
//                               KernelMode,
//                               (PVOID*)&PsInitialSystemProcess,
//                               NULL);
//
//     ARM64_UART_TRACE("[PS] System process created\r\n");
//
//     /* Copy the process names */
//     strcpy(PsIdleProcess->ImageFileName, "Idle");
//     strcpy(PsInitialSystemProcess->ImageFileName, "System");
//
//     /* Allocate a structure for the audit name */
//     PsInitialSystemProcess->SeAuditProcessCreationInfo.ImageFileName =
//         ExAllocatePoolWithTag(PagedPool,
//                               sizeof(OBJECT_NAME_INFORMATION),
//                               TAG_SEPA);
//     if (!PsInitialSystemProcess->SeAuditProcessCreationInfo.ImageFileName)
//     {
//         /* Allocation failed */
//         return FALSE;
//     }
//
//     /* Zero it */
//     RtlZeroMemory(PsInitialSystemProcess->
//                   SeAuditProcessCreationInfo.ImageFileName,
//                   sizeof(OBJECT_NAME_INFORMATION));
//
//     ARM64_UART_TRACE("[PS] Creating Phase1 thread\r\n");
//
//     /* Setup the system initialization thread */
//     Status = PsCreateSystemThread(&SysThreadHandle,
//                                   THREAD_ALL_ACCESS,
//                                   &ObjectAttributes,
//                                   0,
//                                   NULL,
//                                   Phase1Initialization,
//                                   LoaderBlock);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create a handle to it */
//     ObReferenceObjectByHandle(SysThreadHandle,
//                               0,
//                               PsThreadType,
//                               KernelMode,
//                               (PVOID*)&SysThread,
//                               NULL);
//     ObCloseHandle(SysThreadHandle, KernelMode);
//
//     ARM64_UART_TRACE("[PS] PspInitPhase0 done\r\n");
//
//     /* Return success */
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// PsInitSystem(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     /* Check the initialization phase */
//     switch (ExpInitializationPhase)
//     {
//     case 0:
//
//         /* Do Phase 0 */
//         return PspInitPhase0(LoaderBlock);
//
//     case 1:
//
//         /* Do Phase 1 */
//         return PspInitPhase1();
//
//     default:
//
//         /* Don't know any other phase! Bugcheck! */
//         KeBugCheckEx(UNEXPECTED_INITIALIZATION_CALL,
//                      1,
//                      ExpInitializationPhase,
//                      0,
//                      0);
//         return FALSE;
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// PsGetVersion(OUT PULONG MajorVersion OPTIONAL,
//              OUT PULONG MinorVersion OPTIONAL,
//              OUT PULONG BuildNumber  OPTIONAL,
//              OUT PUNICODE_STRING CSDVersion OPTIONAL)
// {
//     if (MajorVersion) *MajorVersion = NtMajorVersion;
//     if (MinorVersion) *MinorVersion = NtMinorVersion;
//     if (BuildNumber ) *BuildNumber  = NtBuildNumber & 0x3FFF;
//
//     if (CSDVersion)
//     {
//         CSDVersion->Length = CmCSDVersionString.Length;
//         CSDVersion->MaximumLength = CmCSDVersionString.MaximumLength;
//         CSDVersion->Buffer = CmCSDVersionString.Buffer;
//     }
//
//     /* Return TRUE if this is a Checked Build */
//     return (NtBuildNumber >> 28) == 0xC;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
