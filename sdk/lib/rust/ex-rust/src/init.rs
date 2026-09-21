use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
// Cross-crate ke/mm/ARM3 globals+recorders as link symbols (all #[no_mangle]
// in ntoskrnl-rust). crate-split Phase 1b.
extern "C" {
    static mut KeBootTime: ntkrnl_sys::ddk::_LARGE_INTEGER;
    static mut KeBootTimeBias: u64;
    static mut KeLoaderBlock: ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
    static mut KeMaximumIncrement: ntkrnl_sys::base::ULONG;
    static mut KeNumberProcessors: core::ffi::c_char;
    static mut KeServiceDescriptorTable: [ntkrnl_sys::ddk::_KSERVICE_TABLE_DESCRIPTOR; 2];
    static mut KeServiceDescriptorTableShadow: [ntkrnl_sys::ddk::_KSERVICE_TABLE_DESCRIPTOR; 2];
    static mut KiServiceLimit: ntkrnl_sys::base::ULONG;
}
extern "system" {
    // KiStartSyncCensus is NTAPI (stdcall on i386, @0); extern "system" so the i386
    // symbol decoration matches its #[no_mangle] def in ke/census.rs.
    fn KiStartSyncCensus();
}
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;

// Use actual definitions, not extern declarations (lto=false splits them)
use crate::time::{ExpLastTimeZoneBias, ExpAltTimeZoneBias, ExpTimeZoneBias};

// Types only available in bindgen (not in hand-written modules)
use ntkrnl_sys::ddk::{


    _UNICODE_STRING, _OBJECT_ATTRIBUTES, _EPROCESS, _KSERVICE_TABLE_DESCRIPTOR, _KUSER_SHARED_DATA,
    PLOADER_PARAMETER_BLOCK, LOADER_PARAMETER_EXTENSION,
    _MEMORY_ALLOCATION_DESCRIPTOR, PMEMORY_ALLOCATION_DESCRIPTOR,
    NLS_DATA_BLOCK, NLSTABLEINFO, RTL_USER_PROCESS_PARAMETERS, _CM_SYSTEM_CONTROL_VECTOR,
    RTL_USER_PROCESS_INFORMATION, LDR_DATA_TABLE_ENTRY, MESSAGE_RESOURCE_ENTRY,
    ANSI_STRING, KEY_VALUE_PARTIAL_INFORMATION, TIME_FIELDS,
    FAST_MUTEX, PFAST_MUTEX, ERESOURCE, PERESOURCE,
    _LARGE_INTEGER, LARGE_INTEGER, KPROCESSOR_MODE,
    _MEMORY_CACHING_TYPE_MmNonCached as MmNonCached,
    // Functions from bindgen
    ObReferenceObjectByHandle, ObCloseHandle,
    NtCreateDirectoryObject, NtCreateSymbolicLinkObject, NtClose,
    ZwCreateSection, ZwWaitForSingleObject, ZwAllocateVirtualMemory,
    ZwFreeVirtualMemory, ZwDisplayString, ZwResumeThread, ZwClose,
    RtlInitUnicodeString, RtlFreeUnicodeString, RtlAnsiStringToUnicodeString,
    RtlCopyUnicodeString, RtlAppendUnicodeToString, RtlAppendUnicodeStringToString,
    RtlInitAnsiString, RtlInitString, RtlInitEmptyUnicodeString,
    RtlInitNlsTables, RtlResetRtlTranslations, RtlCreateUnicodeStringFromAsciiz,
    RtlCreateUserProcess, RtlDeNormalizeProcessParams, RtlFindMessage,
    RtlInitializeRangeListPackage,
    HalInitSystem, HalReportResourceUsage,
    HalQueryRealTimeClock, RtlTimeFieldsToTime, KeSetSystemTime,
    ExLocalTimeToSystemTime, ZwSetSystemTime,
    CmGetSystemControlValues, CmInitSystem1,
    CcInitializeCacheManager, CcPfInitializePrefetcher,
    IoInitSystem, LpcInitSystem, DbgkInitialize,
    SeInitSystem, SeRmInitPhase1,
    ExRefreshTimeZoneInformation,
    ExInitializeFastMutex, ExInitializeResourceLite,
    ExInitPoolLookasidePointers,
    ExpInitializePushLocks, ExpInitializeWorkerThreads,
    ExpInitializeEventImplementation, ExpInitializeEventPairImplementation,
    ExpInitializeMutantImplementation, ExpInitializeCallbacks,
    ExpInitializeSemaphoreImplementation, ExpInitializeTimerImplementation,
    ExpInitializeProfileImplementation, ExpUuidInitialization,
    ExpInitializeKeyedEventImplementation, ExpWin32kInit,
    ExpResourceInitialization, ExpInitLookasideLists,
    ExpInitializeHandleTables,
    ObInitSystem,
    MmArmInitSystem, MmZeroPageThread,
    MmMapViewInSystemSpace, MmMapViewOfSection,
    KeInitSystem, PsGetCurrentThread, KeSetPriorityThread, PKTHREAD,
    KeBugCheck, KiInitializeBugCheck,
    InbvDriverInitialize, InbvEnableBootDriver, InbvEnableDisplayString,
    InbvNotifyDisplayOwnershipLost, InbvSetProgressBarSubset, InbvUpdateProgressBar,
    PsInitSystem,
};

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


// Symbols not in bindgen — declare as extern
extern "system" {
    static InbvBootDriverInstalled: BOOLEAN;
    fn MmFreeLoaderBlock(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn MmVerifyImageIsOkForMpUse(Address: PVOID) -> BOOLEAN;
    fn PpInitSystem() -> BOOLEAN;
    fn DbgLoadImageSymbols(FileName: *mut ANSI_STRING, Base: PVOID, ProcessId: ULONG_PTR);
    fn FsRtlInitSystem() -> BOOLEAN;
    fn HeadlessInit(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    static KdBreakAfterSymbolLoad: BOOLEAN;
    fn KdDebuggerInitialize1(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn KdInitSystem(Phase: ULONG, LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN;
    static mut KdpTimeSlipPending: LONG;
    fn KeI386VdmInitialize();
    fn PoInitSystem(Phase: ULONG) -> BOOLEAN;
    fn PoNotifySystemTimeSet();
    fn XIPInit(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn DbgBreakPointWithStatus(Status: ULONG);
    fn MmInitSystem(Phase: ULONG, LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN;

    static SePublicDefaultUnrestrictedSd: PVOID;
    static MmSectionObjectType: PVOID;
    static mut CmNtGlobalFlag: ULONG;
    static mut SharedUserData: *mut _KUSER_SHARED_DATA;
    static mut ExpTimeRefreshLock: ERESOURCE;
    static mut ExpEnvironmentLock: FAST_MUTEX;
    static mut ExpFirmwareTableProviderListHead: LIST_ENTRY;
    static mut ExpFirmwareTableResource: ERESOURCE;
    // CmControlVector is now defined in crate::config::cmdata as
    // [_CM_SYSTEM_CONTROL_VECTOR; 211]. Imported below via path.
}
extern "C" {
    fn RtlStringCbCopyA(pszDest: *mut i8, cbDest: usize, pszSrc: *const i8) -> NTSTATUS;
    fn _strupr(s: *mut i8) -> *mut i8;
    fn strstr(haystack: *const i8, needle: *const i8) -> *const i8;
}

// NtCurrentProcess is a macro: ((HANDLE)(LONG_PTR)-1)
#[allow(non_snake_case)]
#[inline(always)]
unsafe fn NtCurrentProcess() -> HANDLE {
    -1isize as HANDLE
}

// ViewShare — section inherit enum value
const ViewShare: i32 = 1;

// RT_MESSAGETABLE — resource type constant
const RT_MESSAGETABLE: ULONG = 11;

const LOADER_PARAMETER_EXTENSION_MIN_SIZE: usize = core::mem::size_of::<LOADER_PARAMETER_EXTENSION>();

#[repr(C)]
struct INIT_BUFFER {
    DebugBuffer: [u16; 256],
    VersionBuffer: [u8; 256],
    BootlogHeader: [u8; 256],
    VersionNumber: [u8; 24],
    ProcessInfo: RTL_USER_PROCESS_INFORMATION,
    RegistryBuffer: [u16; 256],
}

#[no_mangle]
pub static mut NtMajorVersion: ULONG = 5;
#[no_mangle]
pub static mut NtMinorVersion: ULONG = 2;
#[cfg(debug_assertions)]
#[no_mangle]
pub static mut NtBuildNumber: ULONG = 3790 | 0xC0000000;
#[cfg(not(debug_assertions))]
#[no_mangle]
pub static mut NtBuildNumber: ULONG = 3790;

#[no_mangle]
pub static mut NtGlobalFlag: ULONG = 0;
#[no_mangle]
pub static mut ExSuiteMask: ULONG = 0;
#[no_mangle]
pub static mut CmNtSpBuildNumber: ULONG = 0;
#[no_mangle]
pub static mut CmNtCSDVersion: ULONG = 0;
#[no_mangle]
pub static mut CmNtCSDReleaseType: ULONG = 0;

// Registry-controlled values owned by ex/init. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static INIT_CONTROL_VECTOR: ntkrnl_sys::SubVector<2> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Windows", "CSDVersion",
            core::ptr::addr_of_mut!(CmNtCSDVersion)),
        ntkrnl_sys::cv!("Windows", "CSDReleaseType",
            core::ptr::addr_of_mut!(CmNtCSDReleaseType)),
    ] });

#[no_mangle]
pub static mut CmVersionString: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CmCSDVersionString: _UNICODE_STRING = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static NtBuildLab: [u8; 22] = *b"3790.reactos.0.x86fre\0";

#[no_mangle]
pub static mut ExpInitializationPhase: ULONG = 0;
#[no_mangle]
pub static mut ExpInTextModeSetup: BOOLEAN = 0;
#[no_mangle]
pub static mut IoRemoteBootClient: BOOLEAN = 0;
#[no_mangle]
pub static mut InitSafeBootMode: ULONG = 0;
#[no_mangle]
pub static mut InitIsWinPEMode: BOOLEAN = 0;
#[no_mangle]
pub static mut InitWinPEModeType: BOOLEAN = 0;
#[no_mangle]
pub static mut SosEnabled: BOOLEAN = 0;

#[no_mangle]
pub static mut NtSystemRoot: _UNICODE_STRING = unsafe { core::mem::zeroed() };

// L"\\SystemRoot\\System32\\smss.exe"
static mut NtInitialUserProcessBuffer: [u16; 128] = {
    let mut buf = [0u16; 128];
    // \SystemRoot
    buf[0] = b'\\' as u16;
    buf[1] = b'S' as u16; buf[2] = b'y' as u16; buf[3] = b's' as u16;
    buf[4] = b't' as u16; buf[5] = b'e' as u16; buf[6] = b'm' as u16;
    buf[7] = b'R' as u16; buf[8] = b'o' as u16; buf[9] = b'o' as u16; buf[10] = b't' as u16;
    // \System32
    buf[11] = b'\\' as u16;
    buf[12] = b'S' as u16; buf[13] = b'y' as u16; buf[14] = b's' as u16;
    buf[15] = b't' as u16; buf[16] = b'e' as u16; buf[17] = b'm' as u16;
    buf[18] = b'3' as u16; buf[19] = b'2' as u16;
    // \smss.exe
    buf[20] = b'\\' as u16;
    buf[21] = b's' as u16; buf[22] = b'm' as u16; buf[23] = b's' as u16;
    buf[24] = b's' as u16; buf[25] = b'.' as u16; buf[26] = b'e' as u16;
    buf[27] = b'x' as u16; buf[28] = b'e' as u16;
    // buf[29..] = 0 (null terminator + padding already zero)
    buf
};
#[no_mangle]
pub static mut NtInitialUserProcessBufferLength: ULONG = (core::mem::size_of::<[u16; 128]>() - core::mem::size_of::<u16>()) as ULONG;
#[no_mangle]
pub static mut NtInitialUserProcessBufferType: ULONG = 1;

#[no_mangle]
pub static mut ExpNlsTableBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut ExpAnsiCodePageDataOffset: ULONG = 0;
#[no_mangle]
pub static mut ExpOemCodePageDataOffset: ULONG = 0;
#[no_mangle]
pub static mut ExpUnicodeCaseTableDataOffset: ULONG = 0;
#[no_mangle]
pub static mut ExpNlsTableInfo: NLSTABLEINFO = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ExpNlsTableSize: SIZE_T = 0;
#[no_mangle]
pub static mut ExpNlsSectionPointer: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut ExCmosClockIsSane: BOOLEAN = 1;
#[no_mangle]
pub static mut ExpRealTimeIsUniversal: BOOLEAN = 0;

extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
}

unsafe fn ExpCreateSystemRootLink(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let mut LinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LinkHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: [u8; 256] = [0; 256];
    let mut TargetString: ANSI_STRING = core::mem::zeroed();
    let mut TargetName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut LinkName, w!("\\ArcName").as_ptr() as *mut u16);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut LinkName,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        SePublicDefaultUnrestrictedSd as PVOID,
    );

    Status = NtCreateDirectoryObject(
        &mut LinkHandle,
        0x000F000F,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 1, 0, 0);
    }

    NtClose(LinkHandle);

    RtlInitUnicodeString(&mut LinkName, w!("\\Device").as_ptr() as *mut u16);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut LinkName,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        SePublicDefaultUnrestrictedSd as PVOID,
    );

    Status = NtCreateDirectoryObject(
        &mut LinkHandle,
        0x000F000F,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 2, 0, 0);
    }

    ObCloseHandle(LinkHandle, 0i8);

    RtlInitAnsiString(&mut AnsiName, b"\\SystemRoot\0".as_ptr() as *mut i8);
    Status = RtlAnsiStringToUnicodeString(&mut LinkName, &mut AnsiName, 1u8);
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 3, 0, 0);
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut LinkName,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        SePublicDefaultUnrestrictedSd as PVOID,
    );

    let arc_boot = (*LoaderBlock).ArcBootDeviceName;
    let nt_boot = (*LoaderBlock).NtBootPathName;
    
    let mut buf_ptr = Buffer.as_mut_ptr();
    core::ptr::copy_nonoverlapping(b"\\ArcName\\".as_ptr(), buf_ptr, 9);
    buf_ptr = buf_ptr.add(9);
    
    let mut src = arc_boot as *const u8;
    while *src != 0 {
        *buf_ptr = *src;
        buf_ptr = buf_ptr.add(1);
        src = src.add(1);
    }
    
    src = nt_boot as *const u8;
    while *src != 0 {
        *buf_ptr = *src;
        buf_ptr = buf_ptr.add(1);
        src = src.add(1);
    }
    buf_ptr = buf_ptr.sub(1);
    *buf_ptr = 0;

    RtlInitString(&mut TargetString, Buffer.as_ptr() as *mut i8);
    Status = RtlAnsiStringToUnicodeString(&mut TargetName, &mut TargetString, 1u8);
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 4, 0, 0);
    }

    Status = NtCreateSymbolicLinkObject(
        &mut LinkHandle,
        0x0000001F,
        &mut ObjectAttributes,
        &mut TargetName,
    );

    RtlFreeUnicodeString(&mut LinkName);
    RtlFreeUnicodeString(&mut TargetName);

    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 5, 0, 0);
    }

    ObCloseHandle(LinkHandle, 0i8);
    STATUS_SUCCESS
}

unsafe fn ExpInitNls(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut NlsSection: HANDLE = core::ptr::null_mut();
    let mut SectionBase: PVOID = core::ptr::null_mut();
    let mut ViewSize: ntkrnl_sys::ddk::ULONG_PTR = 0;
    let mut SectionOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut MdBlock: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut NlsTablesEncountered: ULONG = 0;
    let mut NlsTableSizes: [SIZE_T; 3] = [0; 3];

    if ExpInitializationPhase == 0 {
        ListHead = core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead);
        NextEntry = (*ListHead).Flink;
        while NextEntry != ListHead {
            MdBlock = ((NextEntry as usize) - core::mem::offset_of!(_MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)) as PMEMORY_ALLOCATION_DESCRIPTOR;

            if (*MdBlock).MemoryType == 21 { // LoaderNlsData
                ExpNlsTableSize += (*MdBlock).PageCount as usize * PAGE_SIZE;
                NlsTableSizes[NlsTablesEncountered as usize] = (*MdBlock).PageCount as usize * PAGE_SIZE;
                NlsTablesEncountered += 1;
            }

            NextEntry = (*MdBlock).ListEntry.Flink;
        }

        ExpNlsTableBase = ExAllocatePoolWithTag(NonPagedPool, ExpNlsTableSize, 0x696C7452);
        if ExpNlsTableBase.is_null() {
            KeBugCheck(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED);
        }

        if NlsTablesEncountered == 1 {
            let nls_data = (*LoaderBlock).NlsData as *mut NLS_DATA_BLOCK;
            core::ptr::copy_nonoverlapping(
                (*nls_data).AnsiCodePageData as *const u8,
                ExpNlsTableBase as *mut u8,
                ExpNlsTableSize,
            );
        } else {
            let nls_data = (*LoaderBlock).NlsData as *mut NLS_DATA_BLOCK;
            core::ptr::copy_nonoverlapping(
                (*nls_data).AnsiCodePageData as *const u8,
                ExpNlsTableBase as *mut u8,
                NlsTableSizes[0],
            );
            core::ptr::copy_nonoverlapping(
                (*nls_data).OemCodePageData as *const u8,
                (ExpNlsTableBase as usize + NlsTableSizes[0]) as *mut u8,
                NlsTableSizes[1],
            );
            core::ptr::copy_nonoverlapping(
                (*nls_data).UnicodeCodePageData as *const u8,
                (ExpNlsTableBase as usize + NlsTableSizes[0] + NlsTableSizes[1]) as *mut u8,
                NlsTableSizes[2],
            );
        }

        RtlInitNlsTables(
            (ExpNlsTableBase as usize + ExpAnsiCodePageDataOffset as usize) as *mut u16,
            (ExpNlsTableBase as usize + ExpOemCodePageDataOffset as usize) as *mut u16,
            (ExpNlsTableBase as usize + ExpUnicodeCaseTableDataOffset as usize) as *mut u16,
            &raw mut ExpNlsTableInfo,
        );
        RtlResetRtlTranslations(&raw mut ExpNlsTableInfo);
        return;
    }

    SectionSize.QuadPart = ExpNlsTableSize as i64;

    Status = ZwCreateSection(
        &mut NlsSection,
        0x000F0001,
        core::ptr::null_mut(),
        &mut SectionSize,
        PAGE_READWRITE,
        0x08000001,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 1, 0, 0);
    }

    Status = ObReferenceObjectByHandle(
        NlsSection,
        0x000F0001,
        MmSectionObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
        0i8,
        &raw mut ExpNlsSectionPointer,
        core::ptr::null_mut(),
    );
    ObCloseHandle(NlsSection, 0i8);
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 2, 0, 0);
    }

    let mut NlsTableSizeForMap: ntkrnl_sys::ddk::ULONG_PTR = ExpNlsTableSize as ntkrnl_sys::ddk::ULONG_PTR;
    Status = MmMapViewInSystemSpace(
        ExpNlsSectionPointer,
        &mut SectionBase,
        &mut NlsTableSizeForMap,
    );
    ExpNlsTableSize = NlsTableSizeForMap as SIZE_T;
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 3, 0, 0);
    }

    core::ptr::copy_nonoverlapping(
        ExpNlsTableBase as *const u8,
        SectionBase as *mut u8,
        ExpNlsTableSize,
    );

    ExFreePoolWithTag(ExpNlsTableBase, 0x696C7452);

    ExpNlsTableBase = SectionBase;

    RtlInitNlsTables(
        (ExpNlsTableBase as usize + ExpAnsiCodePageDataOffset as usize) as *mut u16,
        (ExpNlsTableBase as usize + ExpOemCodePageDataOffset as usize) as *mut u16,
        (ExpNlsTableBase as usize + ExpUnicodeCaseTableDataOffset as usize) as *mut u16,
        &raw mut ExpNlsTableInfo,
    );
    RtlResetRtlTranslations(&raw mut ExpNlsTableInfo);

    SectionBase = core::ptr::null_mut();

    Status = MmMapViewOfSection(
        ExpNlsSectionPointer,
        PsGetCurrentProcess() as *mut _EPROCESS,
        &mut SectionBase,
        0,
        0,
        &mut SectionOffset,
        &mut ViewSize,
        ViewShare as u32,
        0,
        PAGE_READWRITE,
    );
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 5, 0, 0);
    }

    core::ptr::copy_nonoverlapping(
        ExpNlsTableBase as *const u8,
        SectionBase as *mut u8,
        ExpNlsTableSize,
    );
    ExpNlsTableBase = SectionBase;
}

unsafe fn ExpLoadInitialProcess(
    InitBuffer: *mut INIT_BUFFER,
    ProcessParameters: *mut *mut RTL_USER_PROCESS_PARAMETERS,
    ProcessEnvironment: *mut *mut i8,
) {
    let mut Status: NTSTATUS;
    let mut Size: ntkrnl_sys::ddk::ULONG_PTR;
    let mut p: *mut u16;
    let mut NullString: _UNICODE_STRING = core::mem::zeroed();
    NullString.Buffer = w!("").as_ptr() as *mut u16;
    NullString.Length = 2;
    NullString.MaximumLength = 2;
    let mut SmssName: _UNICODE_STRING = core::mem::zeroed();
    let mut Environment: _UNICODE_STRING = core::mem::zeroed();
    let mut SystemDriveString: _UNICODE_STRING = core::mem::zeroed();
    let mut DebugString: _UNICODE_STRING = core::mem::zeroed();
    let mut EnvironmentPtr: PVOID = core::ptr::null_mut();
    let ProcessInformation = core::ptr::addr_of_mut!((*InitBuffer).ProcessInfo);
    let mut ProcessParams: *mut RTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();

    // Use kernel pool instead of ZwAllocateVirtualMemory to avoid ROSMM/ARM3
    // conflicts on amd64 where user-mode VAs in the System process overlap
    // with section views mapped by ROSMM during driver loading.
    // The caller's ZwFreeVirtualMemory calls will fail silently (pool != VM);
    // the pool allocation is freed via ExFreePoolWithTag below.
    let params_size = core::mem::size_of::<RTL_USER_PROCESS_PARAMETERS>() + (520 * 6 * 2);
    let env_size = PAGE_SIZE;
    Size = (params_size + env_size) as ntkrnl_sys::ddk::ULONG_PTR;
    ProcessParams = ExAllocatePoolWithTag(PagedPool, Size as usize, 0x50707345) as *mut RTL_USER_PROCESS_PARAMETERS; // 'EspP'
    if ProcessParams.is_null() {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SESSION1_INITIALIZATION_FAILED, 0xC0000017u32 as ULONG_PTR, 0, 0, 0);
    }
    core::ptr::write_bytes(ProcessParams as *mut u8, 0, Size as usize);

    (*ProcessParams).Length = params_size as ULONG;
    (*ProcessParams).MaximumLength = params_size as ULONG;
    (*ProcessParams).Flags = 0x00000001 | 0x00000008;

    EnvironmentPtr = (ProcessParams as usize + params_size) as PVOID;

    (*ProcessParams).Environment = EnvironmentPtr as *mut u16;

    p = (ProcessParams as usize + core::mem::size_of::<RTL_USER_PROCESS_PARAMETERS>()) as *mut u16;
    (*ProcessParams).CurrentDirectory.DosPath.Buffer = p;
    (*ProcessParams).CurrentDirectory.DosPath.MaximumLength = (520 * 2) as USHORT;

    RtlCopyUnicodeString(
        &mut (*ProcessParams).CurrentDirectory.DosPath,
        &raw const NtSystemRoot,
    );

    p = ((*ProcessParams).CurrentDirectory.DosPath.Buffer as usize + (*ProcessParams).CurrentDirectory.DosPath.MaximumLength as usize) as *mut u16;
    (*ProcessParams).DllPath.Buffer = p;
    (*ProcessParams).DllPath.MaximumLength = (520 * 2) as USHORT;

    RtlCopyUnicodeString(&mut (*ProcessParams).DllPath, &(*ProcessParams).CurrentDirectory.DosPath);
    RtlAppendUnicodeToString(&mut (*ProcessParams).DllPath, w!("\\System32").as_ptr() as *mut u16);

    p = ((*ProcessParams).DllPath.Buffer as usize + (*ProcessParams).DllPath.MaximumLength as usize) as *mut u16;
    (*ProcessParams).ImagePathName.Buffer = p;
    (*ProcessParams).ImagePathName.MaximumLength = (520 * 2) as USHORT;

    if (NtInitialUserProcessBufferType != 1) ||
       ((NtInitialUserProcessBufferLength != 0xFFFFFFFF) &&
        ((NtInitialUserProcessBufferLength < 2) ||
         (NtInitialUserProcessBufferLength > (core::mem::size_of_val(&*(&raw const NtInitialUserProcessBuffer)) - 2) as ULONG)))
    {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SESSION2_INITIALIZATION_FAILED, 0xC000000D, NtInitialUserProcessBufferType as ULONG_PTR, NtInitialUserProcessBufferLength as ULONG_PTR, core::mem::size_of_val(&*(&raw const NtInitialUserProcessBuffer)));
    }

    p = (*(&raw mut NtInitialUserProcessBuffer)).as_mut_ptr();
    while (*p != 0) && (*p != b' ' as u16) {
        p = p.add(1);
    }

    (*ProcessParams).ImagePathName.Length = (p as usize - (*(&raw const NtInitialUserProcessBuffer)).as_ptr() as usize) as USHORT;

    core::ptr::copy_nonoverlapping(
        (*(&raw const NtInitialUserProcessBuffer)).as_ptr(),
        (*ProcessParams).ImagePathName.Buffer,
        (*ProcessParams).ImagePathName.Length as usize / 2,
    );

    *(*ProcessParams).ImagePathName.Buffer.add((*ProcessParams).ImagePathName.Length as usize / 2) = 0;

    p = ((*ProcessParams).ImagePathName.Buffer as usize + (*ProcessParams).ImagePathName.MaximumLength as usize) as *mut u16;
    (*ProcessParams).CommandLine.Buffer = p;
    (*ProcessParams).CommandLine.MaximumLength = (520 * 2) as USHORT;

    RtlAppendUnicodeToString(&mut (*ProcessParams).CommandLine, (*(&raw const NtInitialUserProcessBuffer)).as_ptr());

    RtlInitEmptyUnicodeString(
        &mut Environment,
        (*ProcessParams).Environment as *mut u16,
        env_size as USHORT,
    );

    RtlAppendUnicodeToString(&mut Environment, w!("Path=").as_ptr() as *mut u16);
    RtlAppendUnicodeStringToString(&mut Environment, &(*ProcessParams).DllPath);
    RtlAppendUnicodeStringToString(&mut Environment, &NullString);

    SystemDriveString = NtSystemRoot;
    SystemDriveString.Length = 4;

    RtlAppendUnicodeToString(&mut Environment, w!("SystemDrive=").as_ptr() as *mut u16);
    RtlAppendUnicodeStringToString(&mut Environment, &SystemDriveString);
    RtlAppendUnicodeStringToString(&mut Environment, &NullString);

    RtlAppendUnicodeToString(&mut Environment, w!("SystemRoot=").as_ptr() as *mut u16);
    RtlAppendUnicodeStringToString(&mut Environment, &raw const NtSystemRoot);
    RtlAppendUnicodeStringToString(&mut Environment, &NullString);

    SmssName = (*ProcessParams).ImagePathName;
    uart_trace!("[SMSS] RtlCreateUserProcess...");
    Status = RtlCreateUserProcess(
        &mut SmssName,
        OBJ_CASE_INSENSITIVE,
        RtlDeNormalizeProcessParams(ProcessParams),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ProcessInformation,
    );
    uart_trace!("[SMSS] RtlCreateUserProcess done");
    uart_trace_hex!("[SMSS] Status=", Status as usize);
    if Status < 0 {
        RtlInitUnicodeString(&mut DebugString, w!("INIT: Unable to create Session Manager.").as_ptr() as *mut u16);
        ZwDisplayString(&mut DebugString);
        KeBugCheckEx(ntkrnl_sys::bugcodes::SESSION3_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    Status = ZwResumeThread((*ProcessInformation).ThreadHandle, core::ptr::null_mut());
    if Status < 0 {
        RtlInitUnicodeString(&mut DebugString, w!("INIT: Unable to resume Session Manager.").as_ptr() as *mut u16);
        ZwDisplayString(&mut DebugString);
        KeBugCheckEx(ntkrnl_sys::bugcodes::SESSION4_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    *ProcessParameters = ProcessParams;
    *ProcessEnvironment = EnvironmentPtr as *mut i8;
}

unsafe fn ExComputeTickCountMultiplier(ClockIncrement: ULONG) -> ULONG {
    let mut MsRemainder: ULONG = 0;
    let mut MsIncrement: ULONG;
    let mut IncrementRemainder: ULONG;

    MsIncrement = ClockIncrement / 10000;
    IncrementRemainder = ClockIncrement - (MsIncrement * 10000);

    for _ in 0..24 {
        MsRemainder <<= 1;
        IncrementRemainder <<= 1;

        if IncrementRemainder >= 10000 {
            IncrementRemainder -= 10000;
            MsRemainder |= 1;
        }
    }

    (MsIncrement << 24) | MsRemainder
}

unsafe fn ExpInitSystemPhase0() -> BOOLEAN {
    ExpResourceInitialization();
    ExInitializeFastMutex(core::ptr::addr_of_mut!(ExpEnvironmentLock));
    ExpInitLookasideLists();
    InitializeListHead(core::ptr::addr_of_mut!(ExpFirmwareTableProviderListHead));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(ExpFirmwareTableResource));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(ExpTimeRefreshLock));
    ExSuiteMask = 0xFFFFFFFF;
    1u8
}

unsafe fn ExpInitSystemPhase1() -> BOOLEAN {
    ExpInitializeWorkerThreads();
    ExpInitializePushLocks();

    // sync-instrumentation §2/§3: stamp the synctrace table header + start the
    // wedge self-census watchdog (silent when healthy; on a wedge it prints
    // the stuck threads + lost-wakeup/holder-stuck verdict to serial).
    KiStartSyncCensus();

    if ExpInitializeEventImplementation() == 0 {
        return 0;
    }
    if ExpInitializeEventPairImplementation() == 0 {
        return 0;
    }
    if ExpInitializeMutantImplementation() == 0 {
        return 0;
    }
    if ExpInitializeCallbacks() == 0 {
        return 0;
    }
    if ExpInitializeSemaphoreImplementation() == 0 {
        return 0;
    }
    if ExpInitializeTimerImplementation() == 0 {
        return 0;
    }
    if ExpInitializeProfileImplementation() == 0 {
        return 0;
    }
    if ExpUuidInitialization() == 0 {
        return 0;
    }
    if ExpInitializeKeyedEventImplementation() == 0 {
        return 0;
    }
    if ExpWin32kInit() == 0 {
        return 0;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExInitSystem() -> BOOLEAN {
    match ExpInitializationPhase {
        0 => ExpInitSystemPhase0(),
        1 => ExpInitSystemPhase1(),
        _ => {
            KeBugCheck(ntkrnl_sys::bugcodes::UNEXPECTED_INITIALIZATION_CALL);
            0
        }
    }
}

unsafe fn ExpIsLoaderValid(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let Extension = (*LoaderBlock).Extension;
    if (*Extension).Size < LOADER_PARAMETER_EXTENSION_MIN_SIZE as ULONG {
        return 0;
    }
    if (*Extension).MajorVersion > 5 {
        return 1;
    }
    if (*Extension).MajorVersion < 5 {
        return 0;
    }
    if (*Extension).MinorVersion < 2 {
        return 0;
    }
    1u8
}

unsafe fn ExpLoadBootSymbols(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let mut NextEntry: PLIST_ENTRY;
    let mut LdrEntry: *mut LDR_DATA_TABLE_ENTRY;
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut Count: ULONG;
    let mut Length: ULONG;
    let mut Name: *mut u16;
    let mut ImageName: ANSI_STRING = core::mem::zeroed();
    let mut NameBuffer: [u8; 256] = [0; 256];

    NextEntry = (*LoaderBlock).LoadOrderListHead.Flink;
    i = 0;
    while NextEntry != core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead) {
        if i >= 2 {
            LdrEntry = ((NextEntry as usize) - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as *mut LDR_DATA_TABLE_ENTRY;
            
            if *(*LdrEntry).FullDllName.Buffer == b'\\' as u16 {
                Name = (*LdrEntry).FullDllName.Buffer;
                Length = (*LdrEntry).FullDllName.Length as ULONG / 2;

                if NameBuffer.len() < (Length as usize + 1) {
                    Status = 0x80000005u32 as NTSTATUS;
                } else {
                    Count = 0;
                    while Count < Length {
                        NameBuffer[Count as usize] = *Name.add(Count as usize) as u8;
                        Count += 1;
                    }
                    NameBuffer[Count as usize] = 0;
                    Status = STATUS_SUCCESS;
                }
            } else {
                Status = STATUS_SUCCESS;
            }

            if Status >= 0 {
                RtlInitString(&mut ImageName, NameBuffer.as_ptr() as *mut i8);
                DbgLoadImageSymbols(
                    &mut ImageName,
                    (*LdrEntry).DllBase,
                    0xFFFFFFFF,
                );
            }
        }

        NextEntry = (*NextEntry).Flink;
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExBurnMemory(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    PagesToDestroy: ULONG_PTR,
    MemoryType: ULONG,
) {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut ListEntry: PLIST_ENTRY;
    let mut MemDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut PagesToDestroy = PagesToDestroy as ULONG_PTR;

    ListEntry = (*LoaderBlock).MemoryDescriptorListHead.Blink;
    while ListEntry != core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead) {
        MemDescriptor = ((ListEntry as usize) - core::mem::offset_of!(_MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)) as PMEMORY_ALLOCATION_DESCRIPTOR;

        if (*MemDescriptor).MemoryType == 2 || (*MemDescriptor).MemoryType == 5 { // LoaderFree || LoaderFirmwareTemporary
            if (*MemDescriptor).PageCount as ULONG_PTR > PagesToDestroy {
                (*MemDescriptor).PageCount -= PagesToDestroy as ntkrnl_sys::ddk::PFN_NUMBER;
                break;
            } else {
                (*MemDescriptor).MemoryType = MemoryType;
                PagesToDestroy -= (*MemDescriptor).PageCount as ULONG_PTR;
                if PagesToDestroy == 0 {
                    break;
                }
            }
        }

        ListEntry = (*ListEntry).Blink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExpInitializeExecutive(
    Cpu: ULONG,
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut NlsData: *mut NLS_DATA_BLOCK;
    let mut Buffer: [u8; 256] = [0; 256];
    let mut AnsiPath: ANSI_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut CommandLine: *mut i8;
    let mut PerfMem: *mut i8;
    let mut PerfMemUsed: ULONG;
    let mut NtosEntry: *mut LDR_DATA_TABLE_ENTRY;
    let mut MsgEntry: *mut MESSAGE_RESOURCE_ENTRY = core::ptr::null_mut();
    let mut CSDString: ANSI_STRING = core::mem::zeroed();
    let mut Remaining: usize = 0;
    let mut RcEnd: *mut i8 = core::ptr::null_mut();
    let mut VersionBuffer: [u8; 65] = [0; 65];

    uart_trace!("[EXP] ExpInitializeExecutive entered");

    if ExpIsLoaderValid(LoaderBlock) == 0 {
        uart_trace!("[EXP] !!! ExpIsLoaderValid FAILED");
        KeBugCheckEx(
            ntkrnl_sys::bugcodes::MISMATCHED_HAL,
            3,
            (*(*LoaderBlock).Extension).Size as ULONG_PTR,
            (*(*LoaderBlock).Extension).MajorVersion as ULONG_PTR,
            (*(*LoaderBlock).Extension).MinorVersion as ULONG_PTR,
        );
    }

    ExInitPoolLookasidePointers();

    if Cpu != 0 {
        // i386 SMP (M5 Phase C): the AP must NOT call HalInitSystem. On i386 that
        // re-runs GLOBAL Phase-1 HAL init (HalpInitBusHandlers → contiguous-memory
        // alloc → MmMapIoSpace → KeFlushEntireTb), which broadcasts a TLB-flush
        // IPI to P0 — but P0 is spin-waiting in KeStartAllProcessors' handshake
        // (LoaderBlock->Prcb != 0) with IPIs unserviced → hard deadlock (GDB:
        // AP@KiIpiSendPacket ⇄ P0@YieldProcessor). The AP's per-CPU HAL is already
        // done by HalInitializeProcessor (KiSystemStartup). amd64/arm64 APs never
        // reach ExpInitializeExecutive (ReactOS's i386 AP path was an unimplemented
        // FIXME), so skipping HalInitSystem here is a no-op there. Gate on the
        // explicit Cpu arg — reliable, unlike a PRCB field read across the
        // hal_ddk/ddk bindgen boundary (that read returns a stale 0 on the AP).
        return;
    }

    ExpInTextModeSetup = 0;
    IoRemoteBootClient = 0;

    if !(*LoaderBlock).SetupLdrBlock.is_null() {
        if ((*(*LoaderBlock).SetupLdrBlock).Flags & 1) != 0 {
            ExpInTextModeSetup = 1;
        }
        if ((*(*LoaderBlock).SetupLdrBlock).Flags & 2) != 0 {
            IoRemoteBootClient = 1;
        }
    }

    ExpInitializationPhase = 0;
    CommandLine = (*LoaderBlock).LoadOptions;

    NlsData = (*LoaderBlock).NlsData as *mut NLS_DATA_BLOCK;
    ExpNlsTableBase = (*NlsData).AnsiCodePageData;
    ExpAnsiCodePageDataOffset = 0;
    ExpOemCodePageDataOffset = ((*NlsData).OemCodePageData as usize - (*NlsData).AnsiCodePageData as usize) as ULONG;
    ExpUnicodeCaseTableDataOffset = ((*NlsData).UnicodeCodePageData as usize - (*NlsData).AnsiCodePageData as usize) as ULONG;

    RtlInitNlsTables(
        (ExpNlsTableBase as usize + ExpAnsiCodePageDataOffset as usize) as *mut u16,
        (ExpNlsTableBase as usize + ExpOemCodePageDataOffset as usize) as *mut u16,
        (ExpNlsTableBase as usize + ExpUnicodeCaseTableDataOffset as usize) as *mut u16,
        &raw mut ExpNlsTableInfo,
    );
    RtlResetRtlTranslations(&raw mut ExpNlsTableInfo);


    if HalInitSystem(ExpInitializationPhase, LoaderBlock) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::HAL_INITIALIZATION_FAILED);
    }


    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("sti");
    #[cfg(target_arch = "x86")]
    core::arch::asm!("sti");

    // amd64 freeldr leaves TimeZoneBias with garbage where High1Time !=
    // High2Time (e.g. High1=0x01010101, High2=0x101). That makes user-mode
    // GetLocalTime spin forever in its do-while consistency check, which
    // hangs CTrayClockWnd::CalculateDueTime and blocks the explorer
    // taskbar. Zero TimeZoneBias here so the page is consistent before
    // ExRefreshTimeZoneInformation runs in Phase1Initialization.
    // See docs/engineering/amd64-taskbar-regression.md.
    (*SharedUserData).TimeZoneBias = core::mem::zeroed();

    (*SharedUserData).CryptoExponent = 0;

    #[cfg(debug_assertions)]
    {
        NtGlobalFlag |= 0x00000020 | 0x00080000;
    }

    let boot_path = (*LoaderBlock).NtBootPathName;
    Buffer[0] = b'C';
    Buffer[1] = b':';
    let mut i = 2;
    let mut src = boot_path as *const u8;
    while *src != 0 && i < 255 {
        Buffer[i] = *src;
        i += 1;
        src = src.add(1);
    }
    Buffer[i] = 0;

    RtlInitString(&mut AnsiPath, Buffer.as_ptr() as *mut i8);
    AnsiPath.Length -= 1;

    RtlInitEmptyUnicodeString(
        &raw mut NtSystemRoot,
        (*SharedUserData).NtSystemRoot.as_mut_ptr(),
        (core::mem::size_of_val(&(*SharedUserData).NtSystemRoot)) as USHORT,
    );

    Status = RtlAnsiStringToUnicodeString(&raw mut NtSystemRoot, &mut AnsiPath, 0u8);
    if Status < 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::SESSION3_INITIALIZATION_FAILED);
    }

    KiInitializeBugCheck();
    // CmControlVector + its runtime assembler now live in the severed config-rust
    // crate (config crate-split); reference them as link symbols. It's assembled at
    // runtime (not a compile-time const) so config can source per-subsystem slices
    // across crate boundaries. Build it immediately before its sole consumer.
    // CmControlVector is really [_CM_SYSTEM_CONTROL_VECTOR; N]; declaring element 0
    // gives the base pointer CmGetSystemControlValues walks (until the NULL entry).
    extern "C" {
        fn cm_build_control_vector();
        static mut CmControlVector: _CM_SYSTEM_CONTROL_VECTOR;
    }
    cm_build_control_vector();
    CmGetSystemControlValues(
        (*LoaderBlock).RegistryBase,
        core::ptr::addr_of_mut!(CmControlVector),
    );

    CmNtSpBuildNumber = 0;
    if ((CmNtCSDVersion & 0xFFFF0000) == 0) && (CmNtCSDReleaseType == 1) {
        CmNtCSDVersion |= 0 << 16;
    }

    NtGlobalFlag |= CmNtGlobalFlag;


    if ExInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED);
    }


    if MmArmInitSystem(0, LoaderBlock) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED);
    }
    uart_trace!("[EXP] MmArmInitSystem(0) done");

    ExpLoadBootSymbols(LoaderBlock);

    if KdBreakAfterSymbolLoad != 0 {
        DbgBreakPointWithStatus(0x40010008);
    }

    if (*(*LoaderBlock).Extension).Size >= core::mem::size_of::<LOADER_PARAMETER_EXTENSION>() as ULONG {
        HeadlessInit(LoaderBlock);
    }

    #[cfg(target_arch = "x86_64")]
    {
        (*SharedUserData).Reserved1 = 0x000007FFFFFFFFFFu64 as ULONG;
        (*SharedUserData).Reserved3 = 0xFFFF080000000000u64 as ULONG;
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        (*SharedUserData).Reserved1 = MmSystemRangeStart as ULONG;
        (*SharedUserData).Reserved3 = MmSystemRangeStart as ULONG;
    }

    ExpInitNls(LoaderBlock);

    NtosEntry = ((*LoaderBlock).LoadOrderListHead.Flink as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as *mut LDR_DATA_TABLE_ENTRY;

    if (CmNtCSDVersion & 0xFFFF) != 0 {
        Status = RtlFindMessage(
            (*NtosEntry).DllBase,
            RT_MESSAGETABLE,
            0,
            0x1000000A,
            &mut MsgEntry,
        );
        if Status >= 0 {
            RtlInitAnsiString(&mut CSDString, (*MsgEntry).Text.as_ptr() as *mut i8);
            while (CSDString.Length > 0) &&
                  ((*(CSDString.Buffer.add((CSDString.Length - 1) as usize)) == b'\r' as i8) ||
                   (*(CSDString.Buffer.add((CSDString.Length - 1) as usize)) == b'\n' as i8))
            {
                CSDString.Length -= 1;
            }
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_SUCCESS;
        }
    } else {
        Status = RtlStringCbCopyA(
            Buffer.as_mut_ptr() as *mut i8,
            Buffer.len(),
            b"Beta\0".as_ptr() as *mut i8,
        );
    }

    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    RtlInitAnsiString(&mut CSDString, Buffer.as_ptr() as *mut i8);
    Status = RtlAnsiStringToUnicodeString(&raw mut CmCSDVersionString, &mut CSDString, 1u8);
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    Status = crate::rtl_string_cb_printf_a!(
        VersionBuffer.as_mut_ptr(),
        VersionBuffer.len(),
        b"5.2",
    );
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE0_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    RtlCreateUnicodeStringFromAsciiz(&raw mut CmVersionString, VersionBuffer.as_ptr() as *mut i8);

    // Initialize Power Subsystem in Phase 0
    if PoInitSystem(0) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::INTERNAL_POWER_ERROR); // INTERNAL_POWER_ERROR
    }

    ExpInitializeHandleTables();

    // On checked builds, allocate the system-call count table (ex/init.c's
    // `#if DBG` block).
    //
    // This assigns ONLY `.Count`. Two `[0] = core::mem::zeroed()` lines used to
    // stand here — invented by the translation, present in neither the C nor
    // any Windows source — and they wiped `.Base`, `.Limit` and `.Number`, i.e.
    // the syscall dispatch table itself, which `ke/krnlinit.c:104-108` fills in
    // earlier. Every system call after Phase 0 would have dispatched through a
    // zeroed descriptor.
    //
    // It never fired because the whole block is `cfg(debug_assertions)`, which
    // the release profile never sets, so it had never been compiled OR run. It
    // became reachable the moment ex-rust's assertions were armed. A cfg'd-out
    // branch is not tested code — it is code with no evidence either way, and
    // this one was wrong.
    #[cfg(debug_assertions)]
    {
        let service_count = ExAllocatePoolWithTag(NonPagedPool, KiServiceLimit as usize * 4, 0x6C6C6143);
        (*core::ptr::addr_of_mut!(KeServiceDescriptorTable))[0].Count = service_count as *mut ULONG;
        (*core::ptr::addr_of_mut!(KeServiceDescriptorTableShadow))[0].Count = service_count as *mut ULONG;
        if !service_count.is_null() {
            core::ptr::write_bytes(service_count, 0, KiServiceLimit as usize * 4);
        }
    }

    uart_trace!("[EXP] ObInitSystem");
    if ObInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::OBJECT_INITIALIZATION_FAILED);
    }

    uart_trace!("[EXP] SeInitSystem");
    if SeInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::SECURITY_INITIALIZATION_FAILED);
    }

    uart_trace!("[EXP] PsInitSystem");
    if PsInitSystem(LoaderBlock) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::PROCESS_INITIALIZATION_FAILED);
    }

    uart_trace!("[EXP] PpInitSystem");
    if PpInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::PP0_INITIALIZATION_FAILED);
    }

    DbgkInitialize();

    let tick_mult = ExComputeTickCountMultiplier(KeMaximumIncrement);
    (*SharedUserData).TickCountMultiplier = tick_mult;

    (*SharedUserData).NtMajorVersion = NtMajorVersion;
    (*SharedUserData).NtMinorVersion = NtMinorVersion;

    #[cfg(target_arch = "x86_64")]
    {
        // WoW64 (Phase E): admit i386 images — kernel32's CreateProcessInternalW
        // gates on ImageNumberLow <= Machine <= ImageNumberHigh. Real amd64
        // Windows uses the same [I386, AMD64] range.
        (*SharedUserData).ImageNumberLow = 0x014c;
        (*SharedUserData).ImageNumberHigh = 0x8664;
    }
    #[cfg(target_arch = "x86")]
    {
        (*SharedUserData).ImageNumberLow = 0x014c;
        (*SharedUserData).ImageNumberHigh = 0x014c;
    }
    #[cfg(target_arch = "aarch64")]
    {
        // WoW64 (Phase C): admit i386 images — kernel32's CreateProcessInternalW
        // gates on ImageNumberLow <= Machine <= ImageNumberHigh. Real arm64
        // Windows uses the same [I386, ARM64] range.
        (*SharedUserData).ImageNumberLow = 0x014c;
        (*SharedUserData).ImageNumberHigh = 0xAA64;
    }

    *(((KI_USER_SHARED_DATA as usize) + PAGE_SIZE - 4) as *mut ULONG) = 0x8eac705;

    // DBG: ExpInitializeExecutive Phase 0 COMPLETE
}

#[no_mangle]
pub unsafe extern "system" fn Phase1InitializationDiscard(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let LoaderBlock = Context as PLOADER_PARAMETER_BLOCK;
    let mut Status: NTSTATUS;
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut SystemBootTime: LARGE_INTEGER = core::mem::zeroed();
    let mut UniversalBootTime: LARGE_INTEGER = core::mem::zeroed();
    let mut OldTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut NoGuiBoot: BOOLEAN;
    let mut ResetBias: BOOLEAN = 0;
    let mut AlternateShell: BOOLEAN = 0;
    let mut NtosEntry: *mut LDR_DATA_TABLE_ENTRY;
    let mut MsgEntry: *mut MESSAGE_RESOURCE_ENTRY = core::ptr::null_mut();
    let mut CommandLine: *mut i8;
    let mut Environment: *mut i8 = core::ptr::null_mut();
    let mut InitBuffer: *mut INIT_BUFFER;
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut LastTzBias: ULONG;
    let mut Length: ULONG;
    let mut MessageCode: ULONG = 0;
    let mut Size: SIZE_T;
    let ProcessInfo: *mut RTL_USER_PROCESS_INFORMATION;
    let mut KeyPartialInfo: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut OptionHandle: HANDLE = core::ptr::null_mut();
    let mut ProcessParameters: *mut RTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();
    let IoInitSucceeded: BOOLEAN;

    uart_trace!("[PHASE1] Phase1InitializationDiscard entered");

    InitBuffer = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<INIT_BUFFER>(), 0x74696E49) as *mut INIT_BUFFER;
    if InitBuffer.is_null() {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, 0xC0000017, 8, 0, 0);
    }

    ExpInitializationPhase = 1;

    KeSetPriorityThread(KeGetCurrentThread() as ntkrnl_sys::ddk::PKTHREAD, 31);

    uart_trace!("[PHASE1] HalInitSystem(1)");
    if HalInitSystem(1, LoaderBlock) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::HAL1_INITIALIZATION_FAILED);
    }

    // Get the command line and upcase it
    CommandLine = if !(*LoaderBlock).LoadOptions.is_null() {
        _strupr((*LoaderBlock).LoadOptions)
    } else {
        core::ptr::null_mut()
    };

    // Check if GUI Boot is enabled
    NoGuiBoot = if !CommandLine.is_null() && !strstr(CommandLine, b"NOGUIBOOT\0".as_ptr() as *const i8).is_null() { 1 } else { 0 };

    // Get the SOS setting
    SosEnabled = if !CommandLine.is_null() && !strstr(CommandLine, b"SOS\0".as_ptr() as *const i8).is_null() { 1 } else { 0 };

    // Setup the boot video driver
    InbvEnableBootDriver(if NoGuiBoot == 0 { 1 } else { 0 });
    InbvDriverInitialize(LoaderBlock, 5);

    if NoGuiBoot == 0 {
        InbvEnableDisplayString(SosEnabled);
    } else {
        InbvNotifyDisplayOwnershipLost(None);
        InbvEnableDisplayString(0);
    }

    // Check if this is LiveCD (WinPE) mode
    if !CommandLine.is_null() && !strstr(CommandLine, b"MININT\0".as_ptr() as *const i8).is_null() {
        InitIsWinPEMode = 1; // TRUE
        if !strstr(CommandLine, b"INRAM\0".as_ptr() as *const i8).is_null() {
            InitWinPEModeType |= 0x80u8;
        } else {
            InitWinPEModeType |= 0x01u8;
        }
    }

    NtosEntry = ((*LoaderBlock).LoadOrderListHead.Flink as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as *mut LDR_DATA_TABLE_ENTRY;

    InbvUpdateProgressBar(5);

    // CONFIG_SMP (C-reference order): start the application processors
    // BEFORE the Phase-1 Ob/Ex/Io inits — their per-processor loops iterate
    // KeNumberProcessors, so every AP must exist first. Wiring this after
    // ObInitSystem left CPU1's PRCB.PPLookasideList unset; the first
    // ObCreateObject scheduled onto the AP AV'd in
    // ObpAllocateObjectCreateInfoBuffer (M3 bring-up, 2026-07-11).
    // NUMPROC/BOOTPROC/MAXPROC command-line caps still unplumbed.
    #[cfg(feature = "smp")]
    {
        unsafe extern "system" {
            fn KeStartAllProcessors();
            fn HalAllProcessorsStarted() -> u8;
        }
        KeStartAllProcessors();
        if HalAllProcessorsStarted() == 0 {
            KeBugCheck(ntkrnl_sys::bugcodes::HAL1_INITIALIZATION_FAILED); // HAL1_INITIALIZATION_FAILED
        }

        // M3 self-test: prove the KeIpiGenericCall rendezvous actually runs
        // the worker on EVERY active processor (the amd64 implementation is
        // new — nothing else on the boot path exercises it). Each CPU ORs
        // its SetMember into the shared mask; a mismatch vs the active set
        // is a hard protocol failure — bugcheck loud, never limp on.
        {
            use core::sync::atomic::{AtomicU64, Ordering};
            // KeGetCurrentProcessorNumber via bindgen — hand-declaring it here
            // decorates wrong on i386 (stdcall `_..@0` vs the real export) and
            // broke the ntkrnlmp i386 link (gate 2026-07-11).
            use ntkrnl_sys::ddk::KeGetCurrentProcessorNumber;
            unsafe extern "system" {
                fn KeIpiGenericCall(
                    // PKIPI_BROADCAST_WORKER is NTAPI (stdcall on i386): the
                    // worker MUST be `extern "system"`, not `extern "C"`. A cdecl
                    // worker leaves KeIpiGenericCall's `(Function)(Argument)` call
                    // 4 bytes off (callee doesn't clean the stdcall arg) → the
                    // rendezvous frame skews → bad ret → EIP executes on the stack
                    // → deterministic 0x50. Invisible on amd64/arm64 (extern "C"
                    // == extern "system" there); i386-fatal. (field manual §9.91
                    // ABI family; the i386 M5 -smp 1 blocker, 2026-07-16.)
                    Function: Option<unsafe extern "system" fn(usize) -> usize>,
                    Argument: usize,
                ) -> usize;
                static KeActiveProcessors: usize;
                static KeNumberProcessors: i8;
            }
            unsafe extern "system" fn gc_selftest_worker(arg: usize) -> usize {
                let mask = &*(arg as *const AtomicU64);
                mask.fetch_or(1u64 << KeGetCurrentProcessorNumber(), Ordering::SeqCst);
                0
            }
            // The AP publishes its KeActiveProcessors bit at idle-loop entry,
            // which can trail KeStartAllProcessors returning — bounded wait.
            let want = KeNumberProcessors as u32;
            let mut spins: u64 = 0;
            while (core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) as u64)
                .count_ones() < want
            {
                core::hint::spin_loop();
                spins += 1;
                if spins > 500_000_000 {
                    break; // fall through — the mismatch print below reports it
                }
            }
            let active = core::ptr::read_volatile(core::ptr::addr_of!(KeActiveProcessors)) as u64;
            let mask = AtomicU64::new(0);
            KeIpiGenericCall(
                Some(gc_selftest_worker),
                core::ptr::addr_of!(mask) as usize,
            );
            let ran = mask.load(Ordering::SeqCst);
            if ran == active {
                crate::dbg_print!(
                    b"[SMP] KeIpiGenericCall self-test: worker ran on 0x%llx == active set OK\n\0",
                    ran
                );
            } else {
                crate::dbg_print!(
                    b"[SMP] KeIpiGenericCall self-test FAILED: ran 0x%llx vs active 0x%llx\n\0",
                    ran, active
                );
                KeBugCheckEx(ntkrnl_sys::bugcodes::HAL_INITIALIZATION_FAILED, 0x4743, ran as usize, active as usize, 0); // 0x4743 = 'GC'
            }
        }
    }

    uart_trace!("[PHASE1] ObInitSystem");
    if ObInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::OBJECT1_INITIALIZATION_FAILED);
    }
    uart_trace!("[PHASE1] ExInitSystem");
    if ExInitSystem() == 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, 0, 0, 1, 0);
    }
    uart_trace!("[PHASE1] KeInitSystem");
    if KeInitSystem() == 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, 0, 0, 2, 0);
    }
    #[cfg(target_arch = "x86")]
    {
    }
    uart_trace!("[PHASE1] KdInitSystem");
    if KdInitSystem(ExpInitializationPhase, KeLoaderBlock) == 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::PHASE1_INITIALIZATION_FAILED, 0, 0, 3, 0);
    }
    #[cfg(target_arch = "x86")]
    {
    }
    uart_trace!("[PHASE1] SeInitSystem");
    if SeInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::SECURITY1_INITIALIZATION_FAILED);
    }
    #[cfg(target_arch = "x86")]
    {
    }
    InbvUpdateProgressBar(10);
    #[cfg(target_arch = "x86")]
    {
    }
    uart_trace!("[PHASE1] ExpCreateSystemRootLink");
    Status = ExpCreateSystemRootLink(LoaderBlock);
    if Status < 0 {
        KeBugCheckEx(ntkrnl_sys::bugcodes::SYMBOLIC_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }
    #[cfg(target_arch = "x86")]
    {
    }

    uart_trace!("[PHASE1] MmInitSystem(1)");
    if MmInitSystem(1, LoaderBlock) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::MEMORY1_INITIALIZATION_FAILED);
    }
    // DBG: after MmInitSystem(1)
    #[cfg(target_arch = "x86")]
    {
    }

    #[cfg(target_arch = "aarch64")]
    {
        use ntkrnl_sys::ddk::MmMapIoSpace;
        extern "C" {
    static mut Arm64UartKernelBase: PVOID;
}
        // Remap the SAME physical UART the loader discovered and the kernel has
        // been writing to since KiSystemStartup — re-deriving it from a constant
        // here would point Phase 1 at a different device than Phase 0 used.
        let UartPhysical = (*LoaderBlock).u.Arm.Arm64UartBase;
        if UartPhysical != 0 {
            let mut UartPhys: LARGE_INTEGER = core::mem::zeroed();
            UartPhys.QuadPart = UartPhysical as i64;
            let KernelVa = MmMapIoSpace(UartPhys, PAGE_SIZE as ntkrnl_sys::ddk::ULONG_PTR, MmNonCached);
            if !KernelVa.is_null() {
                uart_trace!("[PHASE1] UART remapped to kernel VA");
                Arm64UartKernelBase = KernelVa;
                uart_trace!("[PHASE1] UART now using kernel VA");
            }
        }
    }

    // Map the GICv2 (QEMU virt: GICD @ 0x08000000, GICC @ 0x08010000) to kernel
    // VA and start the periodic ARM generic-timer clock interrupt.  Must run
    // after MmInitSystem(1) (so MmMapIoSpace works) and before the first process
    // switch (SMSS) — the clock ISR runs in arbitrary process context and user
    // PML4s lack the early-boot PXE[0] identity map, same constraint as the UART
    // remap above.  Without this the arm64 system clock is frozen:
    // InterruptTime/KeTickCount never advance, so every relative timer and
    // KeDelayExecutionThread is broken (netstack's smoltcp clock froze on this).
    #[cfg(target_arch = "aarch64")]
    {
        use ntkrnl_sys::ddk::MmMapIoSpace;
        extern "system" {
            fn HalInitializeClockArm64(GicDistVa: PVOID, GicCpuVa: PVOID);
        }
        // Both bases come from the ACPI MADT via the loader block; a board whose
        // firmware describes no GICv2 gets no clock rather than a clock pointed
        // at whatever happens to live at QEMU virt's addresses.
        let GicdPhysical = (*LoaderBlock).u.Arm.Arm64GicDistributorBase;
        let GiccPhysical = (*LoaderBlock).u.Arm.Arm64GicCpuInterfaceBase;
        if GicdPhysical != 0 && GiccPhysical != 0 {
            let mut Phys: LARGE_INTEGER = core::mem::zeroed();
            Phys.QuadPart = GicdPhysical as i64;
            let GicDistVa = MmMapIoSpace(Phys, PAGE_SIZE as ntkrnl_sys::ddk::ULONG_PTR, MmNonCached);
            Phys.QuadPart = GiccPhysical as i64;
            let GicCpuVa = MmMapIoSpace(Phys, PAGE_SIZE as ntkrnl_sys::ddk::ULONG_PTR, MmNonCached);
            if !GicDistVa.is_null() && !GicCpuVa.is_null() {
                HalInitializeClockArm64(GicDistVa, GicCpuVa);
            }
        } else {
            uart_trace!("[PHASE1] no GICv2 in ACPI MADT - system clock not started");
        }
    }

    uart_trace!("[PHASE1] ExpInitNls");
    ExpInitNls(LoaderBlock);

    uart_trace!("[PHASE1] CcInitializeCacheManager");
    if CcInitializeCacheManager() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::CACHE_INITIALIZATION_FAILED);
    }

    uart_trace!("[PHASE1] CmInitSystem1");
    if CmInitSystem1() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::CONFIG_INITIALIZATION_FAILED);
    }

    uart_trace!("[PHASE1] CcPfInitializePrefetcher");
    CcPfInitializePrefetcher();

    InbvUpdateProgressBar(15);

    // Query the RTC and seed the system time — the C original's "Query the
    // clock" block (ntoskrnl/ex/init.c: `if ((ExCmosClockIsSane) &&
    // (HalQueryRealTimeClock(&TimeFields)))`). This was dropped in
    // translation, so SystemTime accumulated from 0 == 1601-01-01 on every
    // boot (found via the arm64 tray clock stuck at "12:00 AM", 2026-07-06).
    // The Y2K "YEAR=" command-line hack is intentionally not carried over.
    if ExCmosClockIsSane != 0 && HalQueryRealTimeClock(core::ptr::addr_of_mut!(TimeFields)) != 0 {
        RtlTimeFieldsToTime(
            core::ptr::addr_of_mut!(TimeFields),
            core::ptr::addr_of_mut!(SystemBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
        );
        UniversalBootTime = SystemBootTime;

        // Check if real time is GMT
        if ExpRealTimeIsUniversal == 0 {
            // Check if we don't have a valid bias
            if ExpLastTimeZoneBias == 0xFFFFFFFF {
                ResetBias = 1;
                ExpLastTimeZoneBias = ExpAltTimeZoneBias;
            }

            // Calculate the bias in seconds
            ExpTimeZoneBias.QuadPart = (ExpLastTimeZoneBias as i32 as i64 * 60) * 10000000;

            // Set the boot time-zone bias
            (*SharedUserData).TimeZoneBias.High2Time = ExpTimeZoneBias.u.HighPart;
            (*SharedUserData).TimeZoneBias.LowPart = ExpTimeZoneBias.u.LowPart;
            (*SharedUserData).TimeZoneBias.High1Time = ExpTimeZoneBias.u.HighPart;

            // Convert the boot time to local time, and set it
            UniversalBootTime.QuadPart = SystemBootTime.QuadPart + ExpTimeZoneBias.QuadPart;
        }

        // Update the system time and notify the system
        KeSetSystemTime(
            core::ptr::addr_of_mut!(UniversalBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
            core::ptr::addr_of_mut!(OldTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
            0u8,
            core::ptr::null_mut(),
        );
        PoNotifySystemTimeSet();

        // Remember this as the boot time
        KeBootTime = UniversalBootTime;
        KeBootTimeBias = 0;
    }

    LastTzBias = ExpLastTimeZoneBias;
    ExRefreshTimeZoneInformation(core::ptr::addr_of!(SystemBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);

    // Post-refresh bias handling — also from the C original: a reset bias
    // re-derives UTC boot time from local time; a changed timezone re-syncs
    // via ZwSetSystemTime.
    if ResetBias != 0 {
        ExLocalTimeToSystemTime(
            core::ptr::addr_of_mut!(SystemBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
            core::ptr::addr_of_mut!(UniversalBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
        );
        KeBootTime = UniversalBootTime;
        KeBootTimeBias = 0;

        KeSetSystemTime(
            core::ptr::addr_of_mut!(UniversalBootTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
            core::ptr::addr_of_mut!(OldTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER,
            0u8,
            core::ptr::null_mut(),
        );
    } else if LastTzBias != ExpLastTimeZoneBias {
        ZwSetSystemTime(core::ptr::null_mut(), core::ptr::null_mut());
    }


    uart_trace!("[PHASE1] FsRtlInitSystem");
    if FsRtlInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::FILE_INITIALIZATION_FAILED);
    }

    RtlInitializeRangeListPackage();

    HalReportResourceUsage();

    uart_trace!("[PHASE1] KdDebuggerInitialize1");
    KdDebuggerInitialize1(LoaderBlock);

    uart_trace!("[PHASE1] PpInitSystem");
    if PpInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::PP1_INITIALIZATION_FAILED);
    }

    InbvUpdateProgressBar(20);

    uart_trace!("[PHASE1] LpcInitSystem");
    if LpcInitSystem() == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::LPC_INITIALIZATION_FAILED);
    }

    InbvUpdateProgressBar(25);

    KdpTimeSlipPending = 0;

    XIPInit(LoaderBlock);

    InbvSetProgressBarSubset(25, 75);

    uart_trace!("[PHASE1] IoInitSystem");
    IoInitSucceeded = IoInitSystem(LoaderBlock);
    if IoInitSucceeded == 0 {
        // arm64 deliberately carries on where every other arch bugchecks. That
        // escape hatch is only defensible if it is LOUD: report it, or a genuine
        // I/O-init failure is indistinguishable from a healthy boot.
        uart_trace!("[PHASE1] IoInitSystem failed (continuing on ARM64)");
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(ntkrnl_sys::bugcodes::IO1_INITIALIZATION_FAILED);
        }
    } else {
        uart_trace!("[PHASE1] IoInitSystem succeeded!");
    }

    InbvSetProgressBarSubset(0, 100);

    InbvUpdateProgressBar(80);

    #[cfg(target_arch = "x86")]
    {
        KeI386VdmInitialize();
    }

    uart_trace!("[PHASE1] PoInitSystem");
    if PoInitSystem(1) == 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::INTERNAL_POWER_ERROR);
    }

    InbvUpdateProgressBar(90);

    uart_trace!("[PHASE1] PsInitSystem");
    if PsInitSystem(LoaderBlock) == 0 {
        // Same arm64 escape hatch as IoInitSystem above, and it was firing on
        // EVERY boot until the PspLookupKernelUserEntryPoints fix (three x86-only
        // ntdll exports looked up unconditionally). Silent then; loud now.
        uart_trace!("[PHASE1] PsInitSystem failed (continuing on ARM64)");
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(ntkrnl_sys::bugcodes::PROCESS1_INITIALIZATION_FAILED);
        }
    }

    if LoaderBlock == KeLoaderBlock {
        KeLoaderBlock = core::ptr::null_mut();
    }
    MmFreeLoaderBlock(LoaderBlock);

    uart_trace!("[PHASE1] SeRmInitPhase1");
    if SeRmInitPhase1() == 0 {
        uart_trace!("[PHASE1] SeRmInitPhase1 failed (continuing on ARM64)");
        #[cfg(not(target_arch = "aarch64"))]
        {
            KeBugCheck(ntkrnl_sys::bugcodes::PROCESS1_INITIALIZATION_FAILED);
        }
    }

    InbvUpdateProgressBar(100);

    if InbvBootDriverInstalled != 0 {
    }

    InbvEnableDisplayString(1);

    // DUMMY_PSEH verdict PoC (dummy-pseh-question lane): prove native clang
    // SEH dispatches end-to-end in kernel mode. Requires requires-uwtable=true
    // in the x86_64-kernel target json (without it the image has no .pdata
    // for Rust frames and the walk double-faults — field manual §9.42). See
    // ntoskrnl/seh_poc.c + docs/engineering/dummy-pseh-verdict.md.
    // arm64: this is the boot-time oracle for KERNEL-mode software raise + native
    // __try/__except dispatch — the exact shape the kmtest exception-raising cluster
    // (ExHardError/ExResource/KeMutex/…) exercises. It had never run on arm64 (this
    // block was x86_64-only), which is why the arm64 kernel-SEH bug (§9.69 / task #10)
    // only surfaced under conformance. Now it fails loud at boot instead of hiding.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        extern "C" {
            fn SehPocRun();
        }
        SehPocRun();
    }

    // Per-boot tripwire for the FAULT-TOLERANT COPY (ntoskrnl/kmsafecopy.c).
    // SehPocRun above only proves a *software* raise dispatches; safe_copy_* depends
    // on a real HARDWARE page fault being caught (KiPageFault -> MmAccessFault ->
    // KiDispatchException -> __except). Nothing was checking that end-to-end. Prints
    // `[KMSAFECOPY] ... (PASS)`; a FAIL means the copy is silently unprotected.
    // i386 included since 2026-07-24: its KmProtectedCopyMemory is the extable
    // fault-fixup primitive (ntoskrnl-rust ke/i386/safecopy.rs), not the C
    // __try shim, and it exports its own KmSafeCopySelfTest with the same
    // contract — so the same boot oracle covers all three arches now.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
    {
        extern "C" {
            fn KmSafeCopySelfTest() -> i32;
        }
        KmSafeCopySelfTest();
    }

    // arm64 gives up on smss here rather than bugchecking. Boot simply STOPS,
    // with no desktop and — until now — not one byte on the serial saying why, so
    // the run presented as a bare HANG_TIMEOUT with a healthy-looking log.
    #[cfg(target_arch = "aarch64")]
    if IoInitSucceeded == 0 {
        uart_trace!("[PHASE1] *** ARM64 Phase 1 complete (no I/O). Entering idle. ***");
        ExpInitializationPhase += 1;
        ExFreePoolWithTag(InitBuffer as PVOID, 0x74696E49);
        return;
    }
    uart_trace!("[PHASE1] ARM64: I/O init succeeded, proceeding to SMSS launch.");

    uart_trace!("[PHASE1] ExpLoadInitialProcess (SMSS)");
    ProcessInfo = core::ptr::addr_of_mut!((*InitBuffer).ProcessInfo);
    ExpLoadInitialProcess(InitBuffer, &mut ProcessParameters, &mut Environment);

    // Wait 5s for the initial process (smss) to prove it doesn't immediately
    // die. A SUCCESS return means smss TERMINATED — fatal: bugcheck 0x6A
    // (SESSION5_INITIALIZATION_FAILED). Matches ntoskrnl/ex/init.c:2350-2360,
    // which does this on ALL arches unconditionally. The wait runs on the
    // Phase-1 thread after smss is spawned, so on a healthy boot it just hits
    // STATUS_TIMEOUT and doesn't gate the desktop marker.
    //
    // The old amd64 carve-out (#[cfg(not(x86_64))], added 5802b15 during March
    // bring-up) masked an early-smss-death rooted in the pre-fix Mm/pushlock
    // wedges (§9.13-9.17, resolved 2026-07-05). With those fixed and amd64
    // green ×3, the guard is restored here to match C.
    Timeout.QuadPart = -50000000i64;
    Status = ZwWaitForSingleObject((*ProcessInfo).ProcessHandle, 0u8, &mut Timeout);
    if Status == STATUS_SUCCESS {
        KeBugCheck(ntkrnl_sys::bugcodes::SESSION5_INITIALIZATION_FAILED);
    }

    ZwClose((*ProcessInfo).ThreadHandle);
    ZwClose((*ProcessInfo).ProcessHandle);

    // ProcessParameters was allocated from kernel pool (ExAllocatePoolWithTag)
    // in ExpLoadInitialProcess, not ZwAllocateVirtualMemory. Environment points
    // inside the same allocation, so no separate free is needed.
    ExFreePoolWithTag(ProcessParameters as PVOID, 0x50707345); // 'EspP'

    ExpInitializationPhase += 1;

    ExFreePoolWithTag(InitBuffer as PVOID, 0x74696E49);

}

#[no_mangle]
pub unsafe extern "system" fn Phase1Initialization(Context: PVOID) {
    let Context = opaque_ptr(Context);
    Phase1InitializationDiscard(Context);
    MmZeroPageThread();
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/init.c
//  * PURPOSE:         Executive Initialization Code
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <reactos/buildno.h>
// #include "inbv/logo.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* This is the size that we can expect from the win 2003 loader */
// #define LOADER_PARAMETER_EXTENSION_MIN_SIZE \
//     RTL_SIZEOF_THROUGH_FIELD(LOADER_PARAMETER_EXTENSION, AcpiTableSize)
//
// /* Temporary hack */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MmArmInitSystem(
//     IN ULONG Phase,
//     IN PLOADER_PARAMETER_BLOCK LoaderBlock
// );
//
// typedef struct _INIT_BUFFER
// {
//     WCHAR DebugBuffer[256];
//     CHAR VersionBuffer[256];
//     CHAR BootlogHeader[256];
//     CHAR VersionNumber[24];
//     RTL_USER_PROCESS_INFORMATION ProcessInfo;
//     WCHAR RegistryBuffer[256];
// } INIT_BUFFER, *PINIT_BUFFER;
//
// /* DATA **********************************************************************/
//
// /* NT Version Info */
// ULONG NtMajorVersion = VER_PRODUCTMAJORVERSION;
// ULONG NtMinorVersion = VER_PRODUCTMINORVERSION;
// #if DBG /* Checked Build */
// ULONG NtBuildNumber = VER_PRODUCTBUILD | 0xC0000000;
// #else   /* Free Build */
// ULONG NtBuildNumber = VER_PRODUCTBUILD;
// #endif
//
// /* NT System Info */
// ULONG NtGlobalFlag = 0;
// ULONG ExSuiteMask;
//
// /* Cm Version Info */
// ULONG CmNtSpBuildNumber;
// ULONG CmNtCSDVersion;
// ULONG CmNtCSDReleaseType;
// _UNICODE_STRING CmVersionString;
// _UNICODE_STRING CmCSDVersionString;
//
// CHAR NtBuildLab[] = KERNEL_VERSION_BUILD_STR "."
//                     REACTOS_COMPILER_NAME "_" REACTOS_COMPILER_VERSION;
//
// /* Init flags and settings */
// ULONG ExpInitializationPhase;
// BOOLEAN ExpInTextModeSetup;
// BOOLEAN IoRemoteBootClient;
// ULONG InitSafeBootMode;
// BOOLEAN InitIsWinPEMode, InitWinPEModeType;
// BOOLEAN SosEnabled; // Used by driver.c!IopDisplayLoadingMessage()
//
// /* NT Boot Path */
// _UNICODE_STRING NtSystemRoot;
//
// /* NT Initial User Application */
// WCHAR NtInitialUserProcessBuffer[128] = L"\\SystemRoot\\System32\\smss.exe";
// ULONG NtInitialUserProcessBufferLength = sizeof(NtInitialUserProcessBuffer) -
//                                          sizeof(WCHAR);
// ULONG NtInitialUserProcessBufferType = REG_SZ;
//
// /* Boot NLS information */
// PVOID ExpNlsTableBase;
// ULONG ExpAnsiCodePageDataOffset, ExpOemCodePageDataOffset;
// ULONG ExpUnicodeCaseTableDataOffset;
// NLSTABLEINFO ExpNlsTableInfo;
// SIZE_T ExpNlsTableSize;
// PVOID ExpNlsSectionPointer;
//
// /* CMOS Timer Sanity */
// BOOLEAN ExCmosClockIsSane = TRUE;
// BOOLEAN ExpRealTimeIsUniversal;
//
// /* FUNCTIONS ****************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// ExpCreateSystemRootLink(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     _UNICODE_STRING LinkName;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE LinkHandle;
//     NTSTATUS Status;
//     ANSI_STRING AnsiName;
//     CHAR Buffer[256];
//     ANSI_STRING TargetString;
//     _UNICODE_STRING TargetName;
//
//     /* Initialize the ArcName tree */
//     RtlInitUnicodeString(&LinkName, L"\\ArcName");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                SePublicDefaultUnrestrictedSd);
//
//     /* Create it */
//     Status = NtCreateDirectoryObject(&LinkHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 1, 0, 0);
//     }
//
//     /* Close the LinkHandle */
//     NtClose(LinkHandle);
//
//     /* Initialize the Device tree */
//     RtlInitUnicodeString(&LinkName, L"\\Device");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                SePublicDefaultUnrestrictedSd);
//
//     /* Create it */
//     Status = NtCreateDirectoryObject(&LinkHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 2, 0, 0);
//     }
//
//     /* Close the LinkHandle */
//     ObCloseHandle(LinkHandle, KernelMode);
//
//     /* Create the system root symlink name */
//     RtlInitAnsiString(&AnsiName, "\\SystemRoot");
//     Status = RtlAnsiStringToUnicodeString(&LinkName, &AnsiName, TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 3, 0, 0);
//     }
//
//     /* Initialize the attributes for the link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                SePublicDefaultUnrestrictedSd);
//
//     /* Build the ARC name */
//     sprintf(Buffer,
//             "\\ArcName\\%s%s",
//             LoaderBlock->ArcBootDeviceName,
//             LoaderBlock->NtBootPathName);
//     Buffer[strlen(Buffer) - 1] = ANSI_NULL;
//
//     /* Convert it to Unicode */
//     RtlInitString(&TargetString, Buffer);
//     Status = RtlAnsiStringToUnicodeString(&TargetName,
//                                           &TargetString,
//                                           TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, bugcheck */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 4, 0, 0);
//     }
//
//     /* Create it */
//     Status = NtCreateSymbolicLinkObject(&LinkHandle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         &TargetName);
//
//     /* Free the strings */
//     RtlFreeUnicodeString(&LinkName);
//     RtlFreeUnicodeString(&TargetName);
//
//     /* Check if creating the link failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 5, 0, 0);
//     }
//
//     /* Close the handle and return success */
//     ObCloseHandle(LinkHandle, KernelMode);
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitNls(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     LARGE_INTEGER SectionSize;
//     NTSTATUS Status;
//     HANDLE NlsSection;
//     PVOID SectionBase = NULL;
//     SIZE_T ViewSize = 0;
//     LARGE_INTEGER SectionOffset = {{0, 0}};
//     PLIST_ENTRY ListHead, NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//     ULONG NlsTablesEncountered = 0;
//     SIZE_T NlsTableSizes[3] = {0, 0, 0}; /* 3 NLS tables */
//
//     /* Check if this is boot-time phase 0 initialization */
//     if (!ExpInitializationPhase)
//     {
//         /* Loop the memory descriptors */
//         ListHead = &LoaderBlock->MemoryDescriptorListHead;
//         NextEntry = ListHead->Flink;
//         while (NextEntry != ListHead)
//         {
//             /* Get the current block */
//             MdBlock = CONTAINING_RECORD(NextEntry,
//                                         MEMORY_ALLOCATION_DESCRIPTOR,
//                                         ListEntry);
//
//             /* Check if this is an NLS block */
//             if (MdBlock->MemoryType == LoaderNlsData)
//             {
//                 /* Increase the table size */
//                 ExpNlsTableSize += MdBlock->PageCount * PAGE_SIZE;
//
//                 /* FreeLdr-specific */
//                 NlsTableSizes[NlsTablesEncountered] = MdBlock->PageCount * PAGE_SIZE;
//                 NlsTablesEncountered++;
//                 ASSERT(NlsTablesEncountered < 4);
//             }
//
//             /* Go to the next block */
//             NextEntry = MdBlock->ListEntry.Flink;
//         }
//
//         /* Allocate the a new buffer since loader memory will be freed */
//         ExpNlsTableBase = ExAllocatePoolWithTag(NonPagedPool,
//                                                 ExpNlsTableSize,
//                                                 TAG_RTLI);
//         if (!ExpNlsTableBase) KeBugCheck(PHASE0_INITIALIZATION_FAILED);
//
//         /* Copy the codepage data in its new location. */
//         if (NlsTablesEncountered == 1)
//         {
//             /* Ntldr-way boot process */
//             RtlCopyMemory(ExpNlsTableBase,
//                           LoaderBlock->NlsData->AnsiCodePageData,
//                           ExpNlsTableSize);
//         }
//         else
//         {
//             /*
//             * In NT, the memory blocks are contiguous, but in ReactOS they aren't,
//             * so unless someone fixes FreeLdr, we'll have to use this icky hack.
//             */
//             RtlCopyMemory(ExpNlsTableBase,
//                           LoaderBlock->NlsData->AnsiCodePageData,
//                           NlsTableSizes[0]);
//
//             RtlCopyMemory((PVOID)((ULONG_PTR)ExpNlsTableBase + NlsTableSizes[0]),
//                           LoaderBlock->NlsData->OemCodePageData,
//                           NlsTableSizes[1]);
//
//             RtlCopyMemory((PVOID)((ULONG_PTR)ExpNlsTableBase + NlsTableSizes[0] +
//                           NlsTableSizes[1]),
//                           LoaderBlock->NlsData->UnicodeCodePageData,
//                           NlsTableSizes[2]);
//             /* End of Hack */
//         }
//
//         /* Initialize and reset the NLS TAbles */
//         RtlInitNlsTables((PVOID)((ULONG_PTR)ExpNlsTableBase +
//                                  ExpAnsiCodePageDataOffset),
//                          (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                                  ExpOemCodePageDataOffset),
//                          (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                                  ExpUnicodeCaseTableDataOffset),
//                          &ExpNlsTableInfo);
//         RtlResetRtlTranslations(&ExpNlsTableInfo);
//         return;
//     }
//
//     /* Set the section size */
//     SectionSize.QuadPart = ExpNlsTableSize;
//
//     /* Create the NLS Section */
//     Status = ZwCreateSection(&NlsSection,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &SectionSize,
//                              PAGE_READWRITE,
//                              SEC_COMMIT | 0x1,
//                              NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 1, 0, 0);
//     }
//
//     /* Get a pointer to the section */
//     Status = ObReferenceObjectByHandle(NlsSection,
//                                        SECTION_ALL_ACCESS,
//                                        MmSectionObjectType,
//                                        KernelMode,
//                                        &ExpNlsSectionPointer,
//                                        NULL);
//     ObCloseHandle(NlsSection, KernelMode);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 2, 0, 0);
//     }
//
//     /* Map the NLS Section in system space */
//     Status = MmMapViewInSystemSpace(ExpNlsSectionPointer,
//                                     &SectionBase,
//                                     &ExpNlsTableSize);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 3, 0, 0);
//     }
//
//     /* Copy the codepage data in its new location. */
//     ASSERT(SectionBase >= MmSystemRangeStart);
//     RtlCopyMemory(SectionBase, ExpNlsTableBase, ExpNlsTableSize);
//
//     /* Free the previously allocated buffer and set the new location */
//     ExFreePoolWithTag(ExpNlsTableBase, TAG_RTLI);
//     ExpNlsTableBase = SectionBase;
//
//     /* Initialize the NLS Tables */
//     RtlInitNlsTables((PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpAnsiCodePageDataOffset),
//                      (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpOemCodePageDataOffset),
//                      (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpUnicodeCaseTableDataOffset),
//                      &ExpNlsTableInfo);
//     RtlResetRtlTranslations(&ExpNlsTableInfo);
//
//     /* Reset the base to 0 */
//     SectionBase = NULL;
//
//     /* Map the section in the system process */
//     Status = MmMapViewOfSection(ExpNlsSectionPointer,
//                                 PsGetCurrentProcess(),
//                                 &SectionBase,
//                                 0L,
//                                 0L,
//                                 &SectionOffset,
//                                 &ViewSize,
//                                 ViewShare,
//                                 0L,
//                                 PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 5, 0, 0);
//     }
//
//     /* Copy the table into the system process and set this as the base */
//     RtlCopyMemory(SectionBase, ExpNlsTableBase, ExpNlsTableSize);
//     ExpNlsTableBase = SectionBase;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpLoadInitialProcess(IN PINIT_BUFFER InitBuffer,
//                       OUT PRTL_USER_PROCESS_PARAMETERS *ProcessParameters,
//                       OUT PCHAR *ProcessEnvironment)
// {
//     NTSTATUS Status;
//     SIZE_T Size;
//     PWSTR p;
//     _UNICODE_STRING NullString = RTL_CONSTANT_STRING(L"");
//     _UNICODE_STRING SmssName, Environment, SystemDriveString, DebugString;
//     PVOID EnvironmentPtr = NULL;
//     PRTL_USER_PROCESS_INFORMATION ProcessInformation;
//     PRTL_USER_PROCESS_PARAMETERS ProcessParams = NULL;
//
//     NullString.Length = sizeof(WCHAR);
//
//     /* Use the initial buffer, after the strings */
//     ProcessInformation = &InitBuffer->ProcessInfo;
//
//     /* Allocate memory for the process parameters */
//     Size = sizeof(*ProcessParams) + ((MAX_WIN32_PATH * 6) * sizeof(WCHAR));
//     Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                      (PVOID*)&ProcessParams,
//                                      0,
//                                      &Size,
//                                      MEM_RESERVE | MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, display error */
//         _snwprintf(InitBuffer->DebugBuffer,
//                    sizeof(InitBuffer->DebugBuffer)/sizeof(WCHAR),
//                    L"INIT: Unable to allocate Process Parameters. 0x%lx",
//                    Status);
//         RtlInitUnicodeString(&DebugString, InitBuffer->DebugBuffer);
//         ZwDisplayString(&DebugString);
//
//         /* Bugcheck the system */
//         KeBugCheckEx(SESSION1_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     ARM64_UART_TRACE("[SMSS] ZwAllocVM OK\r\n");
//     ARM64_UART_TRACE_HEX("[SMSS] ProcessParams=", (ULONG_PTR)ProcessParams);
//     ARM64_UART_TRACE_HEX("[SMSS] Size=", (ULONG_PTR)Size);
//
//     /* Setup the basic header, and give the process the low 1MB to itself */
//     ProcessParams->Length = (ULONG)Size;
//     ProcessParams->MaximumLength = (ULONG)Size;
//     ProcessParams->Flags = RTL_USER_PROCESS_PARAMETERS_NORMALIZED |
//                            RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB;
//
//     /* Allocate a page for the environment */
//     Size = PAGE_SIZE;
//     Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                      &EnvironmentPtr,
//                                      0,
//                                      &Size,
//                                      MEM_RESERVE | MEM_COMMIT,
//                                      PAGE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, display error */
//         _snwprintf(InitBuffer->DebugBuffer,
//                    sizeof(InitBuffer->DebugBuffer)/sizeof(WCHAR),
//                    L"INIT: Unable to allocate Process Environment. 0x%lx",
//                    Status);
//         RtlInitUnicodeString(&DebugString, InitBuffer->DebugBuffer);
//         ZwDisplayString(&DebugString);
//
//         /* Bugcheck the system */
//         KeBugCheckEx(SESSION2_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Write the pointer */
//     ProcessParams->Environment = EnvironmentPtr;
//
//     /* Make a buffer for the DOS path */
//     p = (PWSTR)(ProcessParams + 1);
//     ProcessParams->CurrentDirectory.DosPath.Buffer = p;
//     ProcessParams->CurrentDirectory.DosPath.MaximumLength = MAX_WIN32_PATH *
//                                                             sizeof(WCHAR);
//
//     /* Debug: verify store to user page works */
//     {
//         volatile ULONG_PTR *test_addr = (volatile ULONG_PTR *)&ProcessParams->CurrentDirectory.DosPath.Buffer;
//         ARM64_UART_TRACE_HEX("[SMSS] &Buf=", (ULONG_PTR)test_addr);
//         ARM64_UART_TRACE_HEX("[SMSS] pre-store=", *test_addr);
//         *test_addr = (ULONG_PTR)p;
//         ARM64_UART_TRACE_HEX("[SMSS] post-store=", *test_addr);
//
//         /* Also test raw write at ProcessParams+0 (Length was written earlier) */
//         ARM64_UART_TRACE_HEX("[SMSS] Length=", (ULONG_PTR)ProcessParams->Length);
//         ARM64_UART_TRACE_HEX("[SMSS] MaxLen=", (ULONG_PTR)ProcessParams->MaximumLength);
//         ARM64_UART_TRACE_HEX("[SMSS] Flags=", (ULONG_PTR)ProcessParams->Flags);
//
//         /* Read word at offset 0x10 (ConsoleHandle - should be zero if page zeroed) */
//         ARM64_UART_TRACE_HEX("[SMSS] @+0x10=", *(volatile ULONG_PTR *)((PCHAR)ProcessParams + 0x10));
//         ARM64_UART_TRACE_HEX("[SMSS] @+0x20=", *(volatile ULONG_PTR *)((PCHAR)ProcessParams + 0x20));
//         ARM64_UART_TRACE_HEX("[SMSS] @+0x30=", *(volatile ULONG_PTR *)((PCHAR)ProcessParams + 0x30));
//     }
//     ARM64_UART_TRACE_HEX("[SMSS] NtSystemRoot.Buf=", (ULONG_PTR)NtSystemRoot.Buffer);
//     ARM64_UART_TRACE_HEX("[SMSS] NtSystemRoot.Len=", (ULONG_PTR)NtSystemRoot.Length);
//
//     /* Copy the DOS path */
//     RtlCopyUnicodeString(&ProcessParams->CurrentDirectory.DosPath,
//                          &NtSystemRoot);
//
//     /* Make a buffer for the DLL Path */
//     p = (PWSTR)((PCHAR)ProcessParams->CurrentDirectory.DosPath.Buffer +
//                 ProcessParams->CurrentDirectory.DosPath.MaximumLength);
//     ProcessParams->DllPath.Buffer = p;
//     ProcessParams->DllPath.MaximumLength = MAX_WIN32_PATH * sizeof(WCHAR);
//
//     /* Copy the DLL path and append the system32 directory */
//     RtlCopyUnicodeString(&ProcessParams->DllPath,
//                          &ProcessParams->CurrentDirectory.DosPath);
//     RtlAppendUnicodeToString(&ProcessParams->DllPath, L"\\System32");
//
//     /* Make a buffer for the image name */
//     p = (PWSTR)((PCHAR)ProcessParams->DllPath.Buffer +
//                 ProcessParams->DllPath.MaximumLength);
//     ProcessParams->ImagePathName.Buffer = p;
//     ProcessParams->ImagePathName.MaximumLength = MAX_WIN32_PATH * sizeof(WCHAR);
//
//     /* Make sure the buffer is a valid string which within the given length */
//     if ((NtInitialUserProcessBufferType != REG_SZ) ||
//         ((NtInitialUserProcessBufferLength != MAXULONG) &&
//          ((NtInitialUserProcessBufferLength < sizeof(WCHAR)) ||
//           (NtInitialUserProcessBufferLength >
//            sizeof(NtInitialUserProcessBuffer) - sizeof(WCHAR)))))
//     {
//         /* Invalid initial process string, bugcheck */
//         KeBugCheckEx(SESSION2_INITIALIZATION_FAILED,
//                      STATUS_INVALID_PARAMETER,
//                      NtInitialUserProcessBufferType,
//                      NtInitialUserProcessBufferLength,
//                      sizeof(NtInitialUserProcessBuffer));
//     }
//
//     /* Cut out anything after a space */
//     p = NtInitialUserProcessBuffer;
//     while ((*p) && (*p != L' ')) p++;
//
//     /* Set the image path length */
//     ProcessParams->ImagePathName.Length =
//         (USHORT)((PCHAR)p - (PCHAR)NtInitialUserProcessBuffer);
//
//     /* Copy the actual buffer */
//     RtlCopyMemory(ProcessParams->ImagePathName.Buffer,
//                   NtInitialUserProcessBuffer,
//                   ProcessParams->ImagePathName.Length);
//
//     /* Null-terminate it */
//     ProcessParams->ImagePathName.Buffer[ProcessParams->ImagePathName.Length /
//                                         sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Make a buffer for the command line */
//     p = (PWSTR)((PCHAR)ProcessParams->ImagePathName.Buffer +
//                 ProcessParams->ImagePathName.MaximumLength);
//     ProcessParams->CommandLine.Buffer = p;
//     ProcessParams->CommandLine.MaximumLength = MAX_WIN32_PATH * sizeof(WCHAR);
//
//     /* Add the image name to the command line */
//     RtlAppendUnicodeToString(&ProcessParams->CommandLine,
//                              NtInitialUserProcessBuffer);
//
//     /* Create the environment string */
//     RtlInitEmptyUnicodeString(&Environment,
//                               ProcessParams->Environment,
//                               (USHORT)Size);
//
//     /* Append the DLL path to it */
//     RtlAppendUnicodeToString(&Environment, L"Path=");
//     RtlAppendUnicodeStringToString(&Environment, &ProcessParams->DllPath);
//     RtlAppendUnicodeStringToString(&Environment, &NullString);
//
//     /* Create the system drive string */
//     SystemDriveString = NtSystemRoot;
//     SystemDriveString.Length = 2 * sizeof(WCHAR);
//
//     /* Append it to the environment */
//     RtlAppendUnicodeToString(&Environment, L"SystemDrive=");
//     RtlAppendUnicodeStringToString(&Environment, &SystemDriveString);
//     RtlAppendUnicodeStringToString(&Environment, &NullString);
//
//     /* Append the system root to the environment */
//     RtlAppendUnicodeToString(&Environment, L"SystemRoot=");
//     RtlAppendUnicodeStringToString(&Environment, &NtSystemRoot);
//     RtlAppendUnicodeStringToString(&Environment, &NullString);
//
//     /* Prepare the prefetcher */
//     //CcPfBeginBootPhase(150);
//
//     /* Create SMSS process */
//     SmssName = ProcessParams->ImagePathName;
//     ARM64_UART_TRACE("[SMSS] RtlCreateUserProcess...\r\n");
//     Status = RtlCreateUserProcess(&SmssName,
//                                   OBJ_CASE_INSENSITIVE,
//                                   RtlDeNormalizeProcessParams(ProcessParams),
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   FALSE,
//                                   NULL,
//                                   NULL,
//                                   ProcessInformation);
//     ARM64_UART_TRACE("[SMSS] RtlCreateUserProcess done\r\n");
//     ARM64_UART_TRACE_HEX("[SMSS] Status=", (ULONG_PTR)Status);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, display error */
//         _snwprintf(InitBuffer->DebugBuffer,
//                    sizeof(InitBuffer->DebugBuffer)/sizeof(WCHAR),
//                    L"INIT: Unable to create Session Manager. 0x%lx",
//                    Status);
//         RtlInitUnicodeString(&DebugString, InitBuffer->DebugBuffer);
//         ZwDisplayString(&DebugString);
//
//         /* Bugcheck the system */
//         KeBugCheckEx(SESSION3_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Resume the thread */
//     Status = ZwResumeThread(ProcessInformation->ThreadHandle, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed, display error */
//         _snwprintf(InitBuffer->DebugBuffer,
//                    sizeof(InitBuffer->DebugBuffer)/sizeof(WCHAR),
//                    L"INIT: Unable to resume Session Manager. 0x%lx",
//                    Status);
//         RtlInitUnicodeString(&DebugString, InitBuffer->DebugBuffer);
//         ZwDisplayString(&DebugString);
//
//         /* Bugcheck the system */
//         KeBugCheckEx(SESSION4_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Return success */
//     *ProcessParameters = ProcessParams;
//     *ProcessEnvironment = EnvironmentPtr;
// }
//
// CODE_SEG("INIT")
// ULONG
// NTAPI
// ExComputeTickCountMultiplier(IN ULONG ClockIncrement)
// {
//     ULONG MsRemainder = 0, MsIncrement;
//     ULONG IncrementRemainder;
//     ULONG i;
//
//     /* Count the number of milliseconds for each clock interrupt */
//     MsIncrement = ClockIncrement / (10 * 1000);
//
//     /* Count the remainder from the division above, with 24-bit precision */
//     IncrementRemainder = ClockIncrement - (MsIncrement * (10 * 1000));
//     for (i= 0; i < 24; i++)
//     {
//         /* Shift the remainders */
//         MsRemainder <<= 1;
//         IncrementRemainder <<= 1;
//
//         /* Check if we've went past 1 ms */
//         if (IncrementRemainder >= (10 * 1000))
//         {
//             /* Increase the remainder by one, and substract from increment */
//             IncrementRemainder -= (10 * 1000);
//             MsRemainder |= 1;
//         }
//     }
//
//     /* Return the increment */
//     return (MsIncrement << 24) | MsRemainder;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitSystemPhase0(VOID)
// {
//     /* Initialize EXRESOURCE Support */
//     ExpResourceInitialization();
//
//     /* Initialize the environment lock */
//     ExInitializeFastMutex(&ExpEnvironmentLock);
//
//     /* Initialize the lookaside lists and locks */
//     ExpInitLookasideLists();
//
//     /* Initialize the Firmware Table resource and listhead */
//     InitializeListHead(&ExpFirmwareTableProviderListHead);
//     ExInitializeResourceLite(&ExpFirmwareTableResource);
//     ExInitializeResourceLite(&ExpTimeRefreshLock);
//
//     /* Set the suite mask to maximum and return */
//     ExSuiteMask = 0xFFFFFFFF;
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitSystemPhase1(VOID)
// {
//     /* Initialize worker threads */
//     ExpInitializeWorkerThreads();
//
//     /* Initialize pushlocks */
//     ExpInitializePushLocks();
//
//     /* Initialize events and event pairs */
//     if (ExpInitializeEventImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Event initialization failed\n");
//         return FALSE;
//     }
//     if (ExpInitializeEventPairImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Event Pair initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize mutants */
//     if (ExpInitializeMutantImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Mutant initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize callbacks */
//     if (ExpInitializeCallbacks() == FALSE)
//     {
//         DPRINT1("Executive: Callback initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize semaphores */
//     if (ExpInitializeSemaphoreImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Semaphore initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize timers */
//     if (ExpInitializeTimerImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Timer initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize profiling */
//     if (ExpInitializeProfileImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Profile initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize UUIDs */
//     if (ExpUuidInitialization() == FALSE)
//     {
//         DPRINT1("Executive: Uuid initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize keyed events */
//     if (ExpInitializeKeyedEventImplementation() == FALSE)
//     {
//         DPRINT1("Executive: Keyed event initialization failed\n");
//         return FALSE;
//     }
//
//     /* Initialize Win32K */
//     if (ExpWin32kInit() == FALSE)
//     {
//         DPRINT1("Executive: Win32 initialization failed\n");
//         return FALSE;
//     }
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExInitSystem(VOID)
// {
//     /* Check the initialization phase */
//     switch (ExpInitializationPhase)
//     {
//         case 0:
//
//             /* Do Phase 0 */
//             return ExpInitSystemPhase0();
//
//         case 1:
//
//             /* Do Phase 1 */
//             return ExpInitSystemPhase1();
//
//         default:
//
//             /* Don't know any other phase! Bugcheck! */
//             KeBugCheck(UNEXPECTED_INITIALIZATION_CALL);
//             return FALSE;
//     }
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpIsLoaderValid(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLOADER_PARAMETER_EXTENSION Extension;
//
//     /* Get the loader extension */
//     Extension = LoaderBlock->Extension;
//
//     /* Validate the size (Windows 2003 loader doesn't provide more) */
//     if (Extension->Size < LOADER_PARAMETER_EXTENSION_MIN_SIZE) return FALSE;
//
//     /* Don't validate upper versions */
//     if (Extension->MajorVersion > VER_PRODUCTMAJORVERSION) return TRUE;
//
//     /* Fail if this is NT 4 */
//     if (Extension->MajorVersion < VER_PRODUCTMAJORVERSION) return FALSE;
//
//     /* Fail if this is XP */
//     if (Extension->MinorVersion < VER_PRODUCTMINORVERSION) return FALSE;
//
//     /* This is 2003 or newer, approve it */
//     return TRUE;
// }
//
// static CODE_SEG("INIT")
// VOID
// ExpLoadBootSymbols(
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextEntry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     NTSTATUS Status;
//     ULONG i;
//     ULONG Count, Length;
//     PWCHAR Name;
//     STRING ImageName;
//     CHAR NameBuffer[256];
//
//     /* Loop over the boot modules list */
//     for (NextEntry = LoaderBlock->LoadOrderListHead.Flink, i = 0;
//          NextEntry != &LoaderBlock->LoadOrderListHead;
//          NextEntry = NextEntry->Flink, ++i)
//     {
//         /* Skip the first two images: HAL and kernel */
//         if (i < 2)
//             continue;
//
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         if (LdrEntry->FullDllName.Buffer[0] == L'\\')
//         {
//             /* We have a name, read its data */
//             Name = LdrEntry->FullDllName.Buffer;
//             Length = LdrEntry->FullDllName.Length / sizeof(WCHAR);
//
//             /* Check if our buffer can hold it */
//             if (sizeof(NameBuffer) < Length + sizeof(ANSI_NULL))
//             {
//                 /* It's too long */
//                 Status = STATUS_BUFFER_OVERFLOW;
//             }
//             else
//             {
//                 /* Copy the name */
//                 Count = 0;
//                 do
//                 {
//                     /* Do cheap Unicode to ANSI conversion */
//                     NameBuffer[Count++] = (CHAR)*Name++;
//                 } while (Count < Length);
//
//                 /* Null-terminate */
//                 NameBuffer[Count] = ANSI_NULL;
//                 Status = STATUS_SUCCESS;
//             }
//         }
//         else
//         {
//             /* Safely print the string into our buffer */
//             Status = RtlStringCbPrintfA(NameBuffer,
//                                         sizeof(NameBuffer),
//                                         "%S\\System32\\Drivers\\%wZ",
//                                         &SharedUserData->NtSystemRoot[2],
//                                         &LdrEntry->BaseDllName);
//         }
//
//         /* Check if the buffer is OK */
//         if (NT_SUCCESS(Status))
//         {
//             /* Load the symbols */
//             RtlInitString(&ImageName, NameBuffer);
//             DbgLoadImageSymbols(&ImageName,
//                                 LdrEntry->DllBase,
//                                 (ULONG_PTR)PsGetCurrentProcessId());
//         }
//
// #ifdef CONFIG_SMP
//         /* Check that the image is safe to use if we have more than one CPU */
//         if (!MmVerifyImageIsOkForMpUse(LdrEntry->DllBase))
//         {
//             KeBugCheckEx(UP_DRIVER_ON_MP_SYSTEM,
//                          (ULONG_PTR)LdrEntry->DllBase,
//                          0, 0, 0);
//         }
// #endif // CONFIG_SMP
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExBurnMemory(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//              IN ULONG_PTR PagesToDestroy,
//              IN TYPE_OF_MEMORY MemoryType)
// {
//     PLIST_ENTRY ListEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR MemDescriptor;
//
//     DPRINT1("Burn RAM amount: %lu pages\n", PagesToDestroy);
//
//     /* Loop the memory descriptors, beginning at the end */
//     for (ListEntry = LoaderBlock->MemoryDescriptorListHead.Blink;
//          ListEntry != &LoaderBlock->MemoryDescriptorListHead;
//          ListEntry = ListEntry->Blink)
//     {
//         /* Get the memory descriptor structure */
//         MemDescriptor = CONTAINING_RECORD(ListEntry,
//                                           MEMORY_ALLOCATION_DESCRIPTOR,
//                                           ListEntry);
//
//         /* Is memory free there or is it temporary? */
//         if (MemDescriptor->MemoryType == LoaderFree ||
//             MemDescriptor->MemoryType == LoaderFirmwareTemporary)
//         {
//             /* Check if the descriptor has more pages than we want */
//             if (MemDescriptor->PageCount > PagesToDestroy)
//             {
//                 /* Change block's page count, ntoskrnl doesn't care much */
//                 MemDescriptor->PageCount -= PagesToDestroy;
//                 break;
//             }
//             else
//             {
//                 /* Change block type */
//                 MemDescriptor->MemoryType = MemoryType;
//                 PagesToDestroy -= MemDescriptor->PageCount;
//
//                 /* Check if we are done */
//                 if (PagesToDestroy == 0) break;
//             }
//         }
//     }
// }
//
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitializeExecutive(IN ULONG Cpu,
//                        IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PNLS_DATA_BLOCK NlsData;
//     CHAR Buffer[256];
//     ANSI_STRING AnsiPath;
//     NTSTATUS Status;
//     PCHAR CommandLine, PerfMem;
//     ULONG PerfMemUsed;
//     PLDR_DATA_TABLE_ENTRY NtosEntry;
//     PMESSAGE_RESOURCE_ENTRY MsgEntry;
//     ANSI_STRING CSDString;
//     size_t Remaining = 0;
//     PCHAR RcEnd = NULL;
//     CHAR VersionBuffer[65];
//
//     ARM64_UART_TRACE("[EXP] ExpInitializeExecutive entered\r\n");
//
//     /* Validate Loader */
//     if (!ExpIsLoaderValid(LoaderBlock))
//     {
//         ARM64_UART_TRACE("[EXP] !!! ExpIsLoaderValid FAILED\r\n");
//         /* Invalid loader version */
//         KeBugCheckEx(MISMATCHED_HAL,
//                      3,
//                      LoaderBlock->Extension->Size,
//                      LoaderBlock->Extension->MajorVersion,
//                      LoaderBlock->Extension->MinorVersion);
//     }
//     /* Initialize PRCB pool lookaside pointers */
//     ExInitPoolLookasidePointers();
//
//     /* Check if this is an application CPU */
//     if (Cpu)
//     {
//         /* Then simply initialize it with HAL */
//         if (!HalInitSystem(ExpInitializationPhase, LoaderBlock))
//         {
//             /* Initialization failed */
//             KeBugCheck(HAL_INITIALIZATION_FAILED);
//         }
//
//         /* We're done */
//         return;
//     }
//
//     /* Assume no text-mode or remote boot */
//     ExpInTextModeSetup = FALSE;
//     IoRemoteBootClient = FALSE;
//
//     /* Check if we have a setup loader block */
//     if (LoaderBlock->SetupLdrBlock)
//     {
//         /* Check if this is text-mode setup */
//         if (LoaderBlock->SetupLdrBlock->Flags & SETUPLDR_TEXT_MODE)
//             ExpInTextModeSetup = TRUE;
//
//         /* Check if this is network boot */
//         if (LoaderBlock->SetupLdrBlock->Flags & SETUPLDR_REMOTE_BOOT)
//         {
//             /* Set variable */
//             IoRemoteBootClient = TRUE;
//
//             /* Make sure we're actually booting off the network */
//             ASSERT(!_memicmp(LoaderBlock->ArcBootDeviceName, "net(0)", 6));
//         }
//     }
//
//     /* Set phase to 0 */
//     ExpInitializationPhase = 0;
//     /* Get boot command line */
//     CommandLine = LoaderBlock->LoadOptions;
//     if (CommandLine)
//     {
//         /* Upcase it for comparison and check if we're in performance mode */
//         _strupr(CommandLine);
//         PerfMem = strstr(CommandLine, "PERFMEM");
//         if (PerfMem)
//         {
//             /* Check if the user gave a number of bytes to use */
//             PerfMem = strstr(PerfMem, "=");
//             if (PerfMem)
//             {
//                 /* Read the number of pages we'll use */
//                 PerfMemUsed = atol(PerfMem + 1) * (1024 * 1024 / PAGE_SIZE);
//                 if (PerfMemUsed)
//                 {
//                     /* FIXME: TODO */
//                     DPRINT1("BBT performance mode not yet supported."
//                             "/PERFMEM option ignored.\n");
//                 }
//             }
//         }
//
//         /* Check if we're burning memory */
//         PerfMem = strstr(CommandLine, "BURNMEMORY");
//         if (PerfMem)
//         {
//             /* Check if the user gave a number of bytes to use */
//             PerfMem = strstr(PerfMem, "=");
//             if (PerfMem)
//             {
//                 /* Read the number of pages we'll use */
//                 PerfMemUsed = atol(PerfMem + 1) * (1024 * 1024 / PAGE_SIZE);
//                 if (PerfMemUsed) ExBurnMemory(LoaderBlock, PerfMemUsed, LoaderBad);
//             }
//         }
//     }
//
//     /* Setup NLS Base and offsets */
//     NlsData = LoaderBlock->NlsData;
//     ExpNlsTableBase = NlsData->AnsiCodePageData;
//     ExpAnsiCodePageDataOffset = 0;
//     ExpOemCodePageDataOffset = (ULONG)((ULONG_PTR)NlsData->OemCodePageData -
//                                        (ULONG_PTR)NlsData->AnsiCodePageData);
//     ExpUnicodeCaseTableDataOffset = (ULONG)((ULONG_PTR)NlsData->UnicodeCodePageData -
//                                             (ULONG_PTR)NlsData->AnsiCodePageData);
//
//     /* Initialize the NLS Tables */
//     RtlInitNlsTables((PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpAnsiCodePageDataOffset),
//                      (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpOemCodePageDataOffset),
//                      (PVOID)((ULONG_PTR)ExpNlsTableBase +
//                              ExpUnicodeCaseTableDataOffset),
//                      &ExpNlsTableInfo);
//     RtlResetRtlTranslations(&ExpNlsTableInfo);
//
//     /* Now initialize the HAL */
//     if (!HalInitSystem(ExpInitializationPhase, LoaderBlock))
//     {
//         /* HAL failed to initialize, bugcheck */
//         KeBugCheck(HAL_INITIALIZATION_FAILED);
//     }
//
//     /* Make sure interrupts are active now */
//     _enable();
//
//     /* Clear the crypto exponent */
//     SharedUserData->CryptoExponent = 0;
//
//     /* Set global flags for the checked build */
// #if DBG
//     NtGlobalFlag |= FLG_ENABLE_CLOSE_EXCEPTIONS |
//                     FLG_ENABLE_KDEBUG_SYMBOL_LOAD;
// #endif
//
//     /* Setup NT System Root Path */
//     sprintf(Buffer, "C:%s", LoaderBlock->NtBootPathName);
//
//     /* Convert to ANSI_STRING and null-terminate it */
//     RtlInitString(&AnsiPath, Buffer);
//     Buffer[--AnsiPath.Length] = ANSI_NULL;
//
//     /* Get the string from KUSER_SHARED_DATA's buffer */
//     RtlInitEmptyUnicodeString(&NtSystemRoot,
//                               SharedUserData->NtSystemRoot,
//                               sizeof(SharedUserData->NtSystemRoot));
//
//     /* Now fill it in */
//     Status = RtlAnsiStringToUnicodeString(&NtSystemRoot, &AnsiPath, FALSE);
//     if (!NT_SUCCESS(Status)) KeBugCheck(SESSION3_INITIALIZATION_FAILED);
//
//     /* Setup bugcheck messages */
//     KiInitializeBugCheck();
//
//     /* Setup initial system settings */
//     CmGetSystemControlValues(LoaderBlock->RegistryBase, CmControlVector);
//
//     /* Set the Service Pack Number and add it to the CSD Version number if needed */
//     CmNtSpBuildNumber = VER_PRODUCTBUILD_QFE;
//     if (((CmNtCSDVersion & 0xFFFF0000) == 0) && (CmNtCSDReleaseType == 1))
//     {
//         CmNtCSDVersion |= (VER_PRODUCTBUILD_QFE << 16);
//     }
//
//     /* Add loaded CmNtGlobalFlag value */
//     NtGlobalFlag |= CmNtGlobalFlag;
//
//     /* Initialize the executive at phase 0 */
//     if (!ExInitSystem()) KeBugCheck(PHASE0_INITIALIZATION_FAILED);
//
//     /* Initialize the memory manager at phase 0 */
//     if (!MmArmInitSystem(0, LoaderBlock)) KeBugCheck(PHASE0_INITIALIZATION_FAILED);
//     ARM64_UART_TRACE("[EXP] MmArmInitSystem(0) done\r\n");
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after MmArmInitSystem(0)\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Load boot symbols */
//     ExpLoadBootSymbols(LoaderBlock);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after ExpLoadBootSymbols\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Check if we should break after symbol load */
//     if (KdBreakAfterSymbolLoad)
//         DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
//
//     /* Check if this loader is compatible with NT 5.2 */
//     if (LoaderBlock->Extension->Size >= sizeof(LOADER_PARAMETER_EXTENSION))
//     {
//         /* Setup headless terminal settings */
//         HeadlessInit(LoaderBlock);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after HeadlessInit\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     }
//
//     /* Set system ranges */
// #ifdef _M_AMD64
//     SharedUserData->Reserved1 = MM_HIGHEST_USER_ADDRESS_WOW64;
//     SharedUserData->Reserved3 = MM_SYSTEM_RANGE_START_WOW64;
// #else
//     SharedUserData->Reserved1 = (ULONG_PTR)MmHighestUserAddress;
//     SharedUserData->Reserved3 = (ULONG_PTR)MmSystemRangeStart;
// #endif
//
//     /* Make a copy of the NLS Tables */
//     ExpInitNls(LoaderBlock);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after ExpInitNls\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Get the kernel's load entry */
//     NtosEntry = CONTAINING_RECORD(LoaderBlock->LoadOrderListHead.Flink,
//                                   LDR_DATA_TABLE_ENTRY,
//                                   InLoadOrderLinks);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] got NtosEntry\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] before CSD check\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     /* Check if this is a service pack */
//     if (CmNtCSDVersion & 0xFFFF)
//     {
//         /* Get the service pack string */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] calling RtlFindMessage\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//         Status = RtlFindMessage(NtosEntry->DllBase,
//                                 RT_MESSAGETABLE,
//                                 MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
//                                 WINDOWS_NT_CSD_STRING,
//                                 &MsgEntry);
//         if (NT_SUCCESS(Status))
//         {
//             /* Setup the string */
//             RtlInitAnsiString(&CSDString, (PCHAR)MsgEntry->Text);
//
//             /* Remove trailing newline */
//             while ((CSDString.Length > 0) &&
//                    ((CSDString.Buffer[CSDString.Length - 1] == '\r') ||
//                     (CSDString.Buffer[CSDString.Length - 1] == '\n')))
//             {
//                 /* Skip the trailing character */
//                 CSDString.Length--;
//             }
//
//             /* Fill the buffer with version information */
//             Status = RtlStringCbPrintfA(Buffer,
//                                         sizeof(Buffer),
//                                         "%Z %u%c",
//                                         &CSDString,
//                                         (CmNtCSDVersion & 0xFF00) >> 8,
//                                         (CmNtCSDVersion & 0xFF) ?
//                                         'A' + (CmNtCSDVersion & 0xFF) - 1 :
//                                         ANSI_NULL);
//         }
//         else
//         {
//             /* Build default string */
//             Status = RtlStringCbPrintfA(Buffer,
//                                         sizeof(Buffer),
//                                         "CSD %04x",
//                                         CmNtCSDVersion);
//         }
//
//         /* Check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//         }
//     }
//     else
//     {
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] beta path\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//         /* Then this is a beta */
//         Status = RtlStringCbCopyExA(Buffer,
//                                     sizeof(Buffer),
//                                     VER_PRODUCTBETA_STR,
//                                     NULL,
//                                     &Remaining,
//                                     0);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//         }
//
//         /* Update length */
//         CmCSDVersionString.MaximumLength = sizeof(Buffer) - (USHORT)Remaining;
//     }
//
//     /* Check if we have an RC number */
//     if ((CmNtCSDVersion & 0xFFFF0000) && (CmNtCSDReleaseType == 1))
//     {
//         /* Check if we have no version data yet */
//         if (!(*Buffer))
//         {
//             /* Set defaults */
//             Remaining = sizeof(Buffer);
//             RcEnd = Buffer;
//         }
//         else
//         {
//             /* Add comma and space */
//             Status = RtlStringCbCatExA(Buffer,
//                                        sizeof(Buffer),
//                                        ", ",
//                                        &RcEnd,
//                                        &Remaining,
//                                        0);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Fail */
//                 KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//             }
//         }
//
//         /* Add the version format string */
//         Status = RtlStringCbPrintfA(RcEnd,
//                                     Remaining,
//                                     "v.%u",
//                                     (CmNtCSDVersion & 0xFFFF0000) >> 16);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//         }
//     }
//
//     /* Now setup the final string */
//     RtlInitAnsiString(&CSDString, Buffer);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] before CmCSDVersionString\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     Status = RtlAnsiStringToUnicodeString(&CmCSDVersionString,
//                                           &CSDString,
//                                           TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Add our version */
//     Status = RtlStringCbPrintfA(VersionBuffer,
//                                 sizeof(VersionBuffer),
//                                 "%u.%u",
//                                 VER_PRODUCTMAJORVERSION,
//                                 VER_PRODUCTMINORVERSION);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         KeBugCheckEx(PHASE0_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Build the final version string */
//     RtlCreateUnicodeStringFromAsciiz(&CmVersionString, VersionBuffer);
//
//     /* Check if the user wants a kernel stack trace database */
//     if (NtGlobalFlag & FLG_KERNEL_STACK_TRACE_DB)
//     {
//         /* FIXME: TODO */
//         DPRINT1("Kernel-mode stack trace support not yet present."
//                 "FLG_KERNEL_STACK_TRACE_DB flag ignored.\n");
//     }
//
//     /* Check if he wanted exception logging */
//     if (NtGlobalFlag & FLG_ENABLE_EXCEPTION_LOGGING)
//     {
//         /* FIXME: TODO */
//         DPRINT1("Kernel-mode exception logging support not yet present."
//                 "FLG_ENABLE_EXCEPTION_LOGGING flag ignored.\n");
//     }
//
//     /* Initialize the Handle Table */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] before ExpInitializeHandleTables\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     ExpInitializeHandleTables();
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after ExpInitializeHandleTables\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
// #if DBG
//     /* On checked builds, allocate the system call count table */
//     KeServiceDescriptorTable[0].Count =
//         ExAllocatePoolWithTag(NonPagedPool,
//                               KiServiceLimit * sizeof(ULONG),
//                               'llaC');
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after DBG pool alloc\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Use it for the shadow table too */
//     KeServiceDescriptorTableShadow[0].Count = KeServiceDescriptorTable[0].Count;
//
//     /* Make sure allocation succeeded */
//     if (KeServiceDescriptorTable[0].Count)
//     {
//         /* Zero the call counts to 0 */
//         RtlZeroMemory(KeServiceDescriptorTable[0].Count,
//                       KiServiceLimit * sizeof(ULONG));
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after DBG RtlZeroMemory\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     }
// #endif
//
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] before ObInitSystem\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     /* Create the Basic Object Manager Types to allow new Object Types */
//     ARM64_UART_TRACE("[EXP] ObInitSystem\r\n");
//     if (!ObInitSystem()) KeBugCheck(OBJECT_INITIALIZATION_FAILED);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after ObInitSystem\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Load basic Security for other Managers */
//     ARM64_UART_TRACE("[EXP] SeInitSystem\r\n");
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] before SeInitSystem\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     if (!SeInitSystem()) KeBugCheck(SECURITY_INITIALIZATION_FAILED);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after SeInitSystem, before PsInitSystem\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Initialize the Process Manager */
//     ARM64_UART_TRACE("[EXP] PsInitSystem\r\n");
//     if (!PsInitSystem(LoaderBlock)) KeBugCheck(PROCESS_INITIALIZATION_FAILED);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after PsInitSystem\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Initialize the PnP Manager */
//     ARM64_UART_TRACE("[EXP] PpInitSystem\r\n");
//     if (!PpInitSystem()) KeBugCheck(PP0_INITIALIZATION_FAILED);
//
//     /* Initialize the User-Mode Debugging Subsystem */
//     DbgkInitialize();
//
//     /* Calculate the tick count multiplier */
//     ExpTickCountMultiplier = ExComputeTickCountMultiplier(KeMaximumIncrement);
//     SharedUserData->TickCountMultiplier = ExpTickCountMultiplier;
//
//     /* Set the OS Version */
//     SharedUserData->NtMajorVersion = NtMajorVersion;
//     SharedUserData->NtMinorVersion = NtMinorVersion;
//
//     /* Set the machine type */
//     SharedUserData->ImageNumberLow = IMAGE_FILE_MACHINE_NATIVE;
//     SharedUserData->ImageNumberHigh = IMAGE_FILE_MACHINE_NATIVE;
//
//     /* ReactOS magic */
//     *(PULONG)(KI_USER_SHARED_DATA + PAGE_SIZE - sizeof(ULONG)) = 0x8eac705;
// }
//
// VOID
// NTAPI
// MmFreeLoaderBlock(IN PLOADER_PARAMETER_BLOCK LoaderBlock);
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// Phase1InitializationDiscard(IN PVOID Context)
// {
//     PLOADER_PARAMETER_BLOCK LoaderBlock = Context;
//     NTSTATUS Status, MsgStatus;
//     TIME_FIELDS TimeFields;
//     LARGE_INTEGER SystemBootTime, UniversalBootTime, OldTime, Timeout;
//     BOOLEAN NoGuiBoot, ResetBias = FALSE, AlternateShell = FALSE;
//     PLDR_DATA_TABLE_ENTRY NtosEntry;
//     PMESSAGE_RESOURCE_ENTRY MsgEntry;
//     PCHAR CommandLine, Y2KHackRequired, SafeBoot, Environment;
//     PCHAR StringBuffer, EndBuffer, BeginBuffer, MpString = "";
//     PINIT_BUFFER InitBuffer;
//     ANSI_STRING TempString;
//     ULONG LastTzBias, Length, YearHack = 0, Disposition, MessageCode = 0;
//     SIZE_T Size;
//     size_t Remaining;
//     PRTL_USER_PROCESS_INFORMATION ProcessInfo;
//     KEY_VALUE_PARTIAL_INFORMATION KeyPartialInfo;
//     _UNICODE_STRING KeyName;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle, OptionHandle;
//     PRTL_USER_PROCESS_PARAMETERS ProcessParameters = NULL;
// #ifdef _M_ARM64
//     BOOLEAN IoInitSucceeded = FALSE;
// #endif
//
//     /* Allocate the initialization buffer */
//     InitBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                        sizeof(INIT_BUFFER),
//                                        TAG_INIT);
//     if (!InitBuffer)
//     {
//         /* Bugcheck */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, STATUS_NO_MEMORY, 8, 0, 0);
//     }
//
//     /* Set to phase 1 */
//     ExpInitializationPhase = 1;
//
//     ARM64_UART_TRACE("[PHASE1] Phase1InitializationDiscard entered\r\n");
//
//     /* Set us at maximum priority */
//     KeSetPriorityThread(KeGetCurrentThread(), HIGH_PRIORITY);
//
//
//     /* Do Phase 1 HAL Initialization */
//     ARM64_UART_TRACE("[PHASE1] HalInitSystem(1)\r\n");
//     if (!HalInitSystem(1, LoaderBlock)) KeBugCheck(HAL1_INITIALIZATION_FAILED);
//
//     /* Get the command line and upcase it */
//     CommandLine = (LoaderBlock->LoadOptions ? _strupr(LoaderBlock->LoadOptions) : NULL);
//
//     /* Check if GUI Boot is enabled */
//     NoGuiBoot = (CommandLine && strstr(CommandLine, "NOGUIBOOT") != NULL);
//
//     /* Get the SOS setting */
//     SosEnabled = (CommandLine && strstr(CommandLine, "SOS") != NULL);
//
//     /* Setup the boot video driver */
//     InbvEnableBootDriver(!NoGuiBoot);
//     InbvDriverInitialize(LoaderBlock, IDB_MAX_RESOURCES);
//
//     /* Check if GUI boot is enabled */
//     if (!NoGuiBoot)
//     {
//         /* It is, display the boot logo and enable printing strings */
//         InbvEnableDisplayString(SosEnabled);
//         DisplayBootBitmap(SosEnabled);
//     }
//     else
//     {
//         /* Release display ownership if not using GUI boot */
//         InbvNotifyDisplayOwnershipLost(NULL);
//
//         /* Don't allow boot-time strings */
//         InbvEnableDisplayString(FALSE);
//     }
//
//     /* Check if this is LiveCD (WinPE) mode */
//     if (CommandLine && strstr(CommandLine, "MININT") != NULL)
//     {
//         /* Setup WinPE Settings */
//         InitIsWinPEMode = TRUE;
//         InitWinPEModeType |= (strstr(CommandLine, "INRAM") != NULL) ? 0x80000000 : 0x00000001;
//     }
//
//     /* Get the kernel's load entry */
//     NtosEntry = CONTAINING_RECORD(LoaderBlock->LoadOrderListHead.Flink,
//                                   LDR_DATA_TABLE_ENTRY,
//                                   InLoadOrderLinks);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] got NtosEntry\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Find the banner message */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] calling RtlFindMessage\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     MsgStatus = RtlFindMessage(NtosEntry->DllBase,
//                                RT_MESSAGETABLE,
//                                MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
//                                WINDOWS_NT_BANNER,
//                                &MsgEntry);
//
//     /* Setup defaults and check if we have a version string */
//     StringBuffer = InitBuffer->VersionBuffer;
//     BeginBuffer = StringBuffer;
//     EndBuffer = StringBuffer;
//     Remaining = sizeof(InitBuffer->VersionBuffer);
//     if (CmCSDVersionString.Length)
//     {
//         /* Print the version string */
//         Status = RtlStringCbPrintfExA(StringBuffer,
//                                       Remaining,
//                                       &EndBuffer,
//                                       &Remaining,
//                                       0,
//                                       ": %wZ",
//                                       &CmCSDVersionString);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Bugcheck */
//             KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 7, 0, 0);
//         }
//     }
//     else
//     {
//         /* No version */
//         *EndBuffer = ANSI_NULL; /* Null-terminate the string */
//     }
//
//     /* Skip over the null-terminator to start a new string */
//     ++EndBuffer;
//     --Remaining;
//
//     /* Build the version number */
//     StringBuffer = InitBuffer->VersionNumber;
//     Status = RtlStringCbPrintfA(StringBuffer,
//                                 sizeof(InitBuffer->VersionNumber),
//                                 "%u.%u",
//                                 VER_PRODUCTMAJORVERSION,
//                                 VER_PRODUCTMINORVERSION);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 7, 0, 0);
//     }
//
//     /* Check if we had found a banner message */
//     if (NT_SUCCESS(MsgStatus))
//     {
//         /* Create the banner message */
//         /* ReactOS specific: Report ReactOS version, NtBuildLab information and reported NT kernel version */
//         Status = RtlStringCbPrintfA(EndBuffer,
//                                     Remaining,
//                                     (PCHAR)MsgEntry->Text,
//                                     KERNEL_VERSION_STR,
//                                     NtBuildLab,
//                                     StringBuffer,
//                                     NtBuildNumber & 0xFFFF,
//                                     BeginBuffer);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Bugcheck */
//             KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 7, 0, 0);
//         }
//     }
//     else
//     {
//         /* Use hard-coded banner message */
//         Status = RtlStringCbCopyA(EndBuffer, Remaining, "REACTOS (R)\r\n");
//         if (!NT_SUCCESS(Status))
//         {
//             /* Bugcheck */
//             KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 7, 0, 0);
//         }
//     }
//
//     /* Display the version string on-screen */
//     InbvDisplayString(EndBuffer);
//
//     /* Initialize Power Subsystem in Phase 0 */
//     if (!PoInitSystem(0)) KeBugCheck(INTERNAL_POWER_ERROR);
//
//     /* Check for Y2K hack */
//     Y2KHackRequired = CommandLine ? strstr(CommandLine, "YEAR") : NULL;
//     if (Y2KHackRequired) Y2KHackRequired = strstr(Y2KHackRequired, "=");
//     if (Y2KHackRequired) YearHack = atol(Y2KHackRequired + 1);
//
//     /* Query the clock */
//     if ((ExCmosClockIsSane) && (HalQueryRealTimeClock(&TimeFields)))
//     {
//         /* Check if we're using the Y2K hack */
//         if (Y2KHackRequired) TimeFields.Year = (CSHORT)YearHack;
//
//         /* Convert to time fields */
//         RtlTimeFieldsToTime(&TimeFields, &SystemBootTime);
//         UniversalBootTime = SystemBootTime;
//
//         /* Check if real time is GMT */
//         if (!ExpRealTimeIsUniversal)
//         {
//             /* Check if we don't have a valid bias */
//             if (ExpLastTimeZoneBias == MAXULONG)
//             {
//                 /* Reset */
//                 ResetBias = TRUE;
//                 ExpLastTimeZoneBias = ExpAltTimeZoneBias;
//             }
//
//             /* Calculate the bias in seconds */
//             ExpTimeZoneBias.QuadPart = Int32x32To64(ExpLastTimeZoneBias * 60,
//                                                     10000000);
//
//             /* Set the boot time-zone bias */
//             SharedUserData->TimeZoneBias.High2Time = ExpTimeZoneBias.HighPart;
//             SharedUserData->TimeZoneBias.LowPart = ExpTimeZoneBias.LowPart;
//             SharedUserData->TimeZoneBias.High1Time = ExpTimeZoneBias.HighPart;
//
//             /* Convert the boot time to local time, and set it */
//             UniversalBootTime.QuadPart = SystemBootTime.QuadPart +
//                                          ExpTimeZoneBias.QuadPart;
//         }
//
//         /* Update the system time and notify the system */
//         KeSetSystemTime(&UniversalBootTime, &OldTime, FALSE, NULL);
//         PoNotifySystemTimeSet();
//
//         /* Remember this as the boot time */
//         KeBootTime = UniversalBootTime;
//         KeBootTimeBias = 0;
//     }
//
// #ifdef CONFIG_SMP
//     /*
//      * IMPORTANT NOTE:
//      * Because ReactOS is a "nice" OS, we do not care _at all_
//      * about any number of registered/licensed processors:
//      * no usage of KeRegisteredProcessors nor KeLicensedProcessors.
//      */
//     if (CommandLine)
//     {
//         PSTR Option;
//
//         /* Check for NUMPROC: maximum number of logical processors
//          * that can be started (including dynamically) at run-time */
//         Option = strstr(CommandLine, "NUMPROC");
//         if (Option) Option = strstr(Option, "=");
//         if (Option) KeNumprocSpecified = atol(Option + 1);
//
//         /* Check for BOOTPROC (NT6+ and ReactOS): maximum number
//          * of logical processors that can be started at boot-time */
//         Option = strstr(CommandLine, "BOOTPROC");
//         if (Option) Option = strstr(Option, "=");
//         if (Option) KeBootprocSpecified = atol(Option + 1);
//
//         /* Check for MAXPROC (NT6+ and ReactOS): forces the kernel to report
//          * as existing the maximum number of processors that can be handled */
//         if (strstr(CommandLine, "MAXPROC"))
//             KeMaximumProcessors = MAXIMUM_PROCESSORS;
//     }
//
//     /* Start Application Processors */
//     KeStartAllProcessors();
// #endif
//
//     /* Initialize all processors */
//     if (!HalAllProcessorsStarted()) KeBugCheck(HAL1_INITIALIZATION_FAILED);
//
// #ifdef CONFIG_SMP
//     /* HACK: We should use RtlFindMessage and not only fallback to this */
//     MpString = "MultiProcessor Kernel\r\n";
// #endif
//
//     /* Setup the "MP" String */
//     RtlInitAnsiString(&TempString, MpString);
//
//     /* Make sure to remove the \r\n if we actually have a string */
//     while ((TempString.Length > 0) &&
//            ((TempString.Buffer[TempString.Length - 1] == '\r') ||
//             (TempString.Buffer[TempString.Length - 1] == '\n')))
//     {
//         /* Skip the trailing character */
//         TempString.Length--;
//     }
//
//     /* Get the information string from our resource file */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] calling RtlFindMessage\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//     MsgStatus = RtlFindMessage(NtosEntry->DllBase,
//                                RT_MESSAGETABLE,
//                                MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
//                                KeNumberProcessors > 1 ?
//                                WINDOWS_NT_INFO_STRING_PLURAL :
//                                WINDOWS_NT_INFO_STRING,
//                                &MsgEntry);
//
//     /* Get total RAM size, in MiB */
//     /* Round size up. Assumed to better match actual physical RAM size */
//     Size = ALIGN_UP_BY(MmNumberOfPhysicalPages * PAGE_SIZE, 1024 * 1024) / (1024 * 1024);
//
//     /* Create the string */
//     StringBuffer = InitBuffer->VersionBuffer;
//     Status = RtlStringCbPrintfA(StringBuffer,
//                                 sizeof(InitBuffer->VersionBuffer),
//                                 NT_SUCCESS(MsgStatus) ?
//                                 (PCHAR)MsgEntry->Text :
//                                 "%u System Processor [%Iu MB Memory] %Z\r\n",
//                                 KeNumberProcessors,
//                                 Size,
//                                 &TempString);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 4, 0, 0);
//     }
//
//     /* Display RAM and CPU count */
//     InbvDisplayString(StringBuffer);
//
//     /* Update the progress bar */
//     InbvUpdateProgressBar(5);
//
//     /* Call OB initialization again */
//     ARM64_UART_TRACE("[PHASE1] ObInitSystem\r\n");
//     if (!ObInitSystem()) KeBugCheck(OBJECT1_INITIALIZATION_FAILED);
//
//     /* Initialize Basic System Objects and Worker Threads */
//     ARM64_UART_TRACE("[PHASE1] ExInitSystem\r\n");
//     if (!ExInitSystem()) KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, 0, 0, 1, 0);
//
//     /* Initialize the later stages of the kernel */
//     ARM64_UART_TRACE("[PHASE1] KeInitSystem\r\n");
//     if (!KeInitSystem()) KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, 0, 0, 2, 0);
//
//     /* Call KD Providers at Phase 1 */
//     ARM64_UART_TRACE("[PHASE1] KdInitSystem\r\n");
//     if (!KdInitSystem(ExpInitializationPhase, KeLoaderBlock))
//     {
//         /* Failed, bugcheck */
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, 0, 0, 3, 0);
//     }
//
//     /* Initialize the SRM in Phase 1 */
//     ARM64_UART_TRACE("[PHASE1] SeInitSystem\r\n");
//     if (!SeInitSystem()) KeBugCheck(SECURITY1_INITIALIZATION_FAILED);
//
//     /* Update the progress bar */
//     InbvUpdateProgressBar(10);
//
//     /* Create SystemRoot Link */
//     ARM64_UART_TRACE("[PHASE1] ExpCreateSystemRootLink\r\n");
//     Status = ExpCreateSystemRootLink(LoaderBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to create the system root link */
//         KeBugCheckEx(SYMBOLIC_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Set up Region Maps, Sections and the Paging File */
//     ARM64_UART_TRACE("[PHASE1] MmInitSystem(1)\r\n");
//     if (!MmInitSystem(1, LoaderBlock)) KeBugCheck(MEMORY1_INITIALIZATION_FAILED);
//
// #ifdef _M_ARM64
//     /* Remap PL011 UART from identity-mapped VA (0x09000000) to kernel VA.
//      * MmMapIoSpace is available now. This must happen before the first process
//      * switch (SMSS) since user process PML4s don't have PXE[0] identity mapping. */
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         PHYSICAL_ADDRESS UartPhys;
//         PVOID KernelVa;
//
//         UartPhys.QuadPart = 0x09000000;
//         KernelVa = MmMapIoSpace(UartPhys, PAGE_SIZE, MmNonCached);
//         if (KernelVa)
//         {
//             ARM64_UART_TRACE("[PHASE1] UART remapped to kernel VA\r\n");
//             Arm64UartKernelBase = KernelVa;
//             ARM64_UART_TRACE("[PHASE1] UART now using kernel VA\r\n");
//         }
//     }
// #endif
//
//     /* Create NLS section */
//     ARM64_UART_TRACE("[PHASE1] ExpInitNls\r\n");
//     ExpInitNls(LoaderBlock);
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] after ExpInitNls\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//
//     /* Initialize Cache Views */
//     ARM64_UART_TRACE("[PHASE1] CcInitializeCacheManager\r\n");
//     if (!CcInitializeCacheManager()) KeBugCheck(CACHE_INITIALIZATION_FAILED);
//
//     /* Initialize the Registry */
//     ARM64_UART_TRACE("[PHASE1] CmInitSystem1\r\n");
//     if (!CmInitSystem1()) KeBugCheck(CONFIG_INITIALIZATION_FAILED);
//
//     /* Initialize Prefetcher */
//     ARM64_UART_TRACE("[PHASE1] CcPfInitializePrefetcher\r\n");
//     CcPfInitializePrefetcher();
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(15);
//
//     /* Update timezone information */
//     LastTzBias = ExpLastTimeZoneBias;
//     ExRefreshTimeZoneInformation(&SystemBootTime);
//
//     /* Check if we're resetting timezone data */
//     if (ResetBias)
//     {
//         /* Convert the local time to system time */
//         ExLocalTimeToSystemTime(&SystemBootTime, &UniversalBootTime);
//         KeBootTime = UniversalBootTime;
//         KeBootTimeBias = 0;
//
//         /* Set the new time */
//         KeSetSystemTime(&UniversalBootTime, &OldTime, FALSE, NULL);
//     }
//     else
//     {
//         /* Check if the timezone switched and update the time */
//         if (LastTzBias != ExpLastTimeZoneBias)
//             ZwSetSystemTime(NULL, NULL);
//     }
//
//     /* Initialize the File System Runtime Library */
//     ARM64_UART_TRACE("[PHASE1] FsRtlInitSystem\r\n");
//     if (!FsRtlInitSystem()) KeBugCheck(FILE_INITIALIZATION_FAILED);
//
//     /* Initialize range lists */
//     RtlInitializeRangeListPackage();
//
//     /* Report all resources used by HAL */
//     HalReportResourceUsage();
//
//     /* Call the debugger DLL */
//     ARM64_UART_TRACE("[PHASE1] KdDebuggerInitialize1\r\n");
//     KdDebuggerInitialize1(LoaderBlock);
//
//     /* Setup PnP Manager in phase 1 */
//     ARM64_UART_TRACE("[PHASE1] PpInitSystem\r\n");
//     if (!PpInitSystem()) KeBugCheck(PP1_INITIALIZATION_FAILED);
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(20);
//
//     /* Initialize LPC */
//     ARM64_UART_TRACE("[PHASE1] LpcInitSystem\r\n");
//     if (!LpcInitSystem()) KeBugCheck(LPC_INITIALIZATION_FAILED);
//
//     /* Make sure we have a command line */
//     if (CommandLine)
//     {
//         /* Check if this is a safe mode boot */
//         SafeBoot = strstr(CommandLine, "SAFEBOOT:");
//         if (SafeBoot)
//         {
//             /* Check what kind of boot this is */
//             SafeBoot += 9;
//             if (!strncmp(SafeBoot, "MINIMAL", 7))
//             {
//                 /* Minimal mode */
//                 InitSafeBootMode = 1;
//                 SafeBoot += 7;
//                 MessageCode = BOOTING_IN_SAFEMODE_MINIMAL;
//             }
//             else if (!strncmp(SafeBoot, "NETWORK", 7))
//             {
//                 /* With Networking */
//                 InitSafeBootMode = 2;
//                 SafeBoot += 7;
//                 MessageCode = BOOTING_IN_SAFEMODE_NETWORK;
//             }
//             else if (!strncmp(SafeBoot, "DSREPAIR", 8))
//             {
//                 /* Domain Server Repair */
//                 InitSafeBootMode = 3;
//                 SafeBoot += 8;
//                 MessageCode = BOOTING_IN_SAFEMODE_DSREPAIR;
//
//             }
//             else
//             {
//                 /* Invalid */
//                 InitSafeBootMode = 0;
//             }
//
//             /* Check if there's any settings left */
//             if (*SafeBoot)
//             {
//                 /* Check if an alternate shell was requested */
//                 if (!strncmp(SafeBoot, "(ALTERNATESHELL)", 16))
//                 {
//                     /* Remember this for later */
//                     AlternateShell = TRUE;
//                 }
//             }
//
//             /* Find the message to print out */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] calling RtlFindMessage\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//             Status = RtlFindMessage(NtosEntry->DllBase,
//                                     RT_MESSAGETABLE,
//                                     MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
//                                     MessageCode,
//                                     &MsgEntry);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Display it */
//                 InbvDisplayString((PCSTR)MsgEntry->Text);
//             }
//         }
//     }
//
//     /* Make sure we have a command line */
//     if (CommandLine)
//     {
//         /* Check if bootlogging is enabled */
//         if (strstr(CommandLine, "BOOTLOG"))
//         {
//             /* Find the message to print out */
// #ifdef _M_ARM64
//     {
//         extern volatile PVOID Arm64UartKernelBase;
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "[EXP] calling RtlFindMessage\r\n";
//         if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//     }
// #endif
//             Status = RtlFindMessage(NtosEntry->DllBase,
//                                     RT_MESSAGETABLE,
//                                     MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
//                                     BOOTLOG_ENABLED,
//                                     &MsgEntry);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Display it */
//                 InbvDisplayString((PCSTR)MsgEntry->Text);
//             }
//
//             /* Setup boot logging */
//             //IopInitializeBootLogging(LoaderBlock, InitBuffer->BootlogHeader);
//         }
//     }
//
//     /* Setup the Executive in Phase 2 */
//     //ExInitSystemPhase2();
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(25);
//
//     /* No KD Time Slip is pending */
//     KdpTimeSlipPending = 0;
//
//     /* Initialize in-place execution support */
//     XIPInit(LoaderBlock);
//
//     /* Set maximum update to 75% */
//     InbvSetProgressBarSubset(25, 75);
//
//     /* Initialize the I/O Subsystem */
//     ARM64_UART_TRACE("[PHASE1] IoInitSystem\r\n");
// #ifdef _M_ARM64
//     IoInitSucceeded = IoInitSystem(LoaderBlock);
//     if (!IoInitSucceeded)
//     {
//         ARM64_UART_TRACE("[PHASE1] IoInitSystem failed (continuing on ARM64)\r\n");
//     }
//     else
//     {
//         ARM64_UART_TRACE("[PHASE1] IoInitSystem succeeded!\r\n");
//     }
// #else
//     if (!IoInitSystem(LoaderBlock))
//     {
//         KeBugCheck(IO1_INITIALIZATION_FAILED);
//     }
// #endif
//
//     /* Set maximum update to 100% */
//     InbvSetProgressBarSubset(0, 100);
//
//     /* Are we in safe mode? */
//     if (InitSafeBootMode)
//     {
//         /* Open the safe boot key */
//         RtlInitUnicodeString(&KeyName,
//                              L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET"
//                              L"\\CONTROL\\SAFEBOOT");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = ZwOpenKey(&KeyHandle, KEY_ALL_ACCESS, &ObjectAttributes);
//         if (NT_SUCCESS(Status))
//         {
//             /* First check if we have an alternate shell */
//             if (AlternateShell)
//             {
//                 /* Make sure that the registry has one setup */
//                 RtlInitUnicodeString(&KeyName, L"AlternateShell");
//                 Status = NtQueryValueKey(KeyHandle,
//                                          &KeyName,
//                                          KeyValuePartialInformation,
//                                          &KeyPartialInfo,
//                                          sizeof(KeyPartialInfo),
//                                          &Length);
//                 if (!(NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW))
//                 {
//                     AlternateShell = FALSE;
//                 }
//             }
//
//             /* Create the option key */
//             RtlInitUnicodeString(&KeyName, L"Option");
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &KeyName,
//                                        OBJ_CASE_INSENSITIVE,
//                                        KeyHandle,
//                                        NULL);
//             Status = ZwCreateKey(&OptionHandle,
//                                  KEY_ALL_ACCESS,
//                                  &ObjectAttributes,
//                                  0,
//                                  NULL,
//                                  REG_OPTION_VOLATILE,
//                                  &Disposition);
//             NtClose(KeyHandle);
//
//             /* Check if the key create worked */
//             if (NT_SUCCESS(Status))
//             {
//                 /* Write the safe boot type */
//                 RtlInitUnicodeString(&KeyName, L"OptionValue");
//                 NtSetValueKey(OptionHandle,
//                               &KeyName,
//                               0,
//                               REG_DWORD,
//                               &InitSafeBootMode,
//                               sizeof(InitSafeBootMode));
//
//                 /* Check if we have to use an alternate shell */
//                 if (AlternateShell)
//                 {
//                     /* Remember this for later */
//                     Disposition = TRUE;
//                     RtlInitUnicodeString(&KeyName, L"UseAlternateShell");
//                     NtSetValueKey(OptionHandle,
//                                   &KeyName,
//                                   0,
//                                   REG_DWORD,
//                                   &Disposition,
//                                   sizeof(Disposition));
//                 }
//
//                 /* Close the options key handle */
//                 NtClose(OptionHandle);
//             }
//         }
//     }
//
//     /* Are we in Win PE mode? */
//     if (InitIsWinPEMode)
//     {
//         /* Open the safe control key */
//         RtlInitUnicodeString(&KeyName,
//                              L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET"
//                              L"\\CONTROL");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = ZwOpenKey(&KeyHandle, KEY_ALL_ACCESS, &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Bugcheck */
//             KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 6, 0, 0);
//         }
//
//         /* Create the MiniNT key */
//         RtlInitUnicodeString(&KeyName, L"MiniNT");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    KeyHandle,
//                                    NULL);
//         Status = ZwCreateKey(&OptionHandle,
//                              KEY_ALL_ACCESS,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              REG_OPTION_VOLATILE,
//                              &Disposition);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Bugcheck */
//             KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 6, 0, 0);
//         }
//
//         /* Close the handles */
//         NtClose(KeyHandle);
//         NtClose(OptionHandle);
//     }
//
//     /* FIXME: This doesn't do anything for now */
//     MmArmInitSystem(2, LoaderBlock);
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(80);
//
//     /* Initialize VDM support */
// #if defined(_M_IX86)
//     KeI386VdmInitialize();
// #endif
//
//     /* Initialize Power Subsystem in Phase 1*/
//     ARM64_UART_TRACE("[PHASE1] PoInitSystem\r\n");
//     if (!PoInitSystem(1)) KeBugCheck(INTERNAL_POWER_ERROR);
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(90);
//
//     /* Initialize the Process Manager at Phase 1 */
//     ARM64_UART_TRACE("[PHASE1] PsInitSystem\r\n");
//     if (!PsInitSystem(LoaderBlock))
//     {
// #ifdef _M_ARM64
//         ARM64_UART_TRACE("[PHASE1] PsInitSystem failed (continuing on ARM64)\r\n");
// #else
//         DPRINT1("PsInitSystem(Phase1) failed!\n");
//         KeBugCheck(PROCESS1_INITIALIZATION_FAILED);
// #endif
//     }
//
//     /* Make sure nobody touches the loader block again */
//     if (LoaderBlock == KeLoaderBlock) KeLoaderBlock = NULL;
//     MmFreeLoaderBlock(LoaderBlock);
//     LoaderBlock = Context = NULL;
//
//     /* Initialize the SRM in phase 1 */
//     ARM64_UART_TRACE("[PHASE1] SeRmInitPhase1\r\n");
//     if (!SeRmInitPhase1())
//     {
// #ifdef _M_ARM64
//         ARM64_UART_TRACE("[PHASE1] SeRmInitPhase1 failed (continuing on ARM64)\r\n");
// #else
//         DPRINT1("SeRmInitPhase1 failed!\n");
//         KeBugCheck(PROCESS1_INITIALIZATION_FAILED);
// #endif
//     }
//
//     /* Update progress bar */
//     InbvUpdateProgressBar(100);
//
//     /* Clear the screen */
//     if (InbvBootDriverInstalled) FinalizeBootLogo();
//
//     /* Allow strings to be displayed */
//     InbvEnableDisplayString(TRUE);
//
// #ifdef _M_ARM64
//     if (!IoInitSucceeded)
//     {
//         /* I/O subsystem failed (no filesystem) — cannot launch smss.exe.
//          * Phase 1 initialization is complete — enter the zero page thread. */
//         ARM64_UART_TRACE("[PHASE1] *** ARM64 Phase 1 complete (no I/O). Entering idle. ***\r\n");
//         ExpInitializationPhase++;
//         ExFreePoolWithTag(InitBuffer, TAG_INIT);
//         return;
//     }
//     ARM64_UART_TRACE("[PHASE1] ARM64: I/O init succeeded, proceeding to SMSS launch.\r\n");
// #endif
//
// #ifdef REACTOS_TEST_KERNEL
//     if (CommandLine && strstr(CommandLine, "TESTKERNEL"))
//     {
//         extern VOID NTAPI NtTestKernelDispatchLoop(VOID);
//         NtTestKernelDispatchLoop();
//         ExpInitializationPhase++;
//         ExFreePoolWithTag(InitBuffer, TAG_INIT);
//         return;
//     }
// #endif
//
//     /* Launch initial process */
//     ARM64_UART_TRACE("[PHASE1] ExpLoadInitialProcess (SMSS)\r\n");
//     ProcessInfo = &InitBuffer->ProcessInfo;
//     ExpLoadInitialProcess(InitBuffer, &ProcessParameters, &Environment);
//
//     /* Wait 5 seconds for initial process to initialize */
//     Timeout.QuadPart = Int32x32To64(5, -10000000);
//     Status = ZwWaitForSingleObject(ProcessInfo->ProcessHandle, FALSE, &Timeout);
//     if (Status == STATUS_SUCCESS)
//     {
//         /* Failed, display error */
//         DPRINT1("INIT: Session Manager terminated.\n");
//
//         /* Bugcheck the system if SMSS couldn't initialize */
//         KeBugCheck(SESSION5_INITIALIZATION_FAILED);
//     }
//
//     /* Close process handles */
//     ZwClose(ProcessInfo->ThreadHandle);
//     ZwClose(ProcessInfo->ProcessHandle);
//
//     /* Free the initial process environment */
//     Size = 0;
//     ZwFreeVirtualMemory(NtCurrentProcess(),
//                         (PVOID*)&Environment,
//                         &Size,
//                         MEM_RELEASE);
//
//     /* Free the initial process parameters */
//     Size = 0;
//     ZwFreeVirtualMemory(NtCurrentProcess(),
//                         (PVOID*)&ProcessParameters,
//                         &Size,
//                         MEM_RELEASE);
//
//     /* Increase init phase */
//     ExpInitializationPhase++;
//
//     /* Free the boot buffer */
//     ExFreePoolWithTag(InitBuffer, TAG_INIT);
// }
//
// VOID
// NTAPI
// Phase1Initialization(IN PVOID Context)
// {
//     /* Do the .INIT part of Phase 1 which we can free later */
//     Phase1InitializationDiscard(Context);
//
//     /* Jump into zero page thread */
//     MmZeroPageThread();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
