use ntkrnl_sys::globals::*;
// Cross-crate ke/mm/ARM3 globals+recorders as link symbols (all #[no_mangle]
// in ntoskrnl-rust). crate-split Phase 1b.
extern "C" {
    static mut KeNodeBlock: [ntkrnl_sys::ddk::PKNODE; 1];
    static mut KeNumberNodes: ntkrnl_sys::base::UCHAR;
    static mut KeProcessorArchitecture: ntkrnl_sys::base::USHORT;
    static mut KeProcessorLevel: ntkrnl_sys::base::USHORT;
    static mut KeProcessorRevision: ntkrnl_sys::base::USHORT;
    static mut KeActiveProcessors: ntkrnl_sys::ddk::KAFFINITY;
    static mut KeBootTime: ntkrnl_sys::ddk::_LARGE_INTEGER;
    static mut KeFeatureBits: u64;
    static mut KeMaximumIncrement: ntkrnl_sys::base::ULONG;
    static mut KeNumberProcessors: core::ffi::c_char;
    static mut KeTimeAdjustment: ntkrnl_sys::base::ULONG;
    static mut KiAdjustDpcThreshold: ntkrnl_sys::base::ULONG;
    static mut KiIdealDpcRate: ntkrnl_sys::base::ULONG;
    static mut KiMaximumDpcQueueDepth: ntkrnl_sys::base::ULONG;
    static mut KiMinimumDpcRate: ntkrnl_sys::base::ULONG;
    static mut MiFreeSwapPages: ntkrnl_sys::ddk::PFN_COUNT;
    static mut MiUsedSwapPages: ntkrnl_sys::ddk::PFN_COUNT;
    static mut MmLoadedUserImageList: ntkrnl_sys::ddk::_LIST_ENTRY;
    static mut PsLoadedModuleList: ntkrnl_sys::ddk::_LIST_ENTRY;
    static mut PsLoadedModuleResource: ntkrnl_sys::ddk::_ERESOURCE;
}
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    ANSI_STRING,
    ARC_STATUS,
    DRIVER_OBJECT,
    ERESOURCE,
    _ETHREAD,
    EXHANDLE,
    EX_PUSH_LOCK,
    ExAcquirePushLockShared,
    ExAcquireResourceExclusiveLite,
    ExGetPoolTagInfo,
    ExReleasePushLockShared,
    ExReleaseResourceLite,
    ExSystemExceptionFilter,
    ExUnlockHandleTableEntry,
    ExpLockHandleTableEntry,
    ExpLookupHandleTableEntry,
    ExpSetTimeZoneInformation,
    FAST_MUTEX,
    _HANDLE_TABLE,
    HANDLE,
    HalSetEnvironmentVariable,
    _IMAGE_NT_HEADERS,
    PIMAGE_NT_HEADERS,
    IoAllocateMdl,
    IoGetConfigurationInformation,
    IoWMIOpenBlock,
    IoWMIQueryAllData,
    KAFFINITY,
    KPROCESSOR_MODE,
    KeEnterCriticalRegion,
    KeGetCurrentProcessorNumber,
    KeLeaveCriticalRegion,
    KeQueryRuntimeProcess,
    KeQueryTimeIncrement,
    LARGE_INTEGER,
    _LDR_DATA_TABLE_ENTRY,
    _LUID,
    LIST_ENTRY,
    LPGUID,
    IoFreeMdl,
    MmLoadSystemImage,
    MmUnloadSystemImage,
    MmUnlockPages,
    PEPROCESS,
    PETHREAD,
    POBJECT_HEADER,
    PHANDLE_TABLE_ENTRY,
    RtlEqualUnicodeString,
    RtlImageNtHeader,
    RTL_TIME_ZONE_INFORMATION,
    PRTL_TIME_ZONE_INFORMATION,
    _UNICODE_STRING,
    PUNICODE_STRING,
    ZwSetSystemInformation,
    // SYSTEM_*_INFORMATION structs
    _SYSTEM_BASIC_INFORMATION,
    _SYSTEM_PROCESSOR_INFORMATION,
    _SYSTEM_PERFORMANCE_INFORMATION,
    _SYSTEM_TIMEOFDAY_INFORMATION,
    _SYSTEM_PROCESS_INFORMATION,
    _SYSTEM_THREAD_INFORMATION,
    _SYSTEM_DEVICE_INFORMATION,
    _SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION,
    _SYSTEM_FLAGS_INFORMATION,
    _SYSTEM_HANDLE_INFORMATION,
    _SYSTEM_HANDLE_TABLE_ENTRY_INFO,
    _SYSTEM_PAGEFILE_INFORMATION,
    _SYSTEM_FILECACHE_INFORMATION,
    _SYSTEM_INTERRUPT_INFORMATION,
    _SYSTEM_DPC_BEHAVIOR_INFORMATION,
    _SYSTEM_GDI_DRIVER_INFORMATION,
    _SYSTEM_QUERY_TIME_ADJUST_INFORMATION,
    _SYSTEM_SET_TIME_ADJUST_INFORMATION,
    _SYSTEM_EXCEPTION_INFORMATION,
    _SYSTEM_KERNEL_DEBUGGER_INFORMATION,
    _SYSTEM_CONTEXT_SWITCH_INFORMATION,
    _SYSTEM_REGISTRY_QUOTA_INFORMATION,
    _SYSTEM_LOOKASIDE_INFORMATION,
    _SYSTEM_FIRMWARE_TABLE_INFORMATION,
    _SYSTEM_LOGICAL_PROCESSOR_INFORMATION,
    _RTL_PROCESS_MODULES,
    _RTL_PROCESS_MODULE_INFORMATION,
    _GENERAL_LOOKASIDE,
    _CONFIGURATION_INFORMATION,
    PCONFIGURATION_INFORMATION,
};

// Rust-defined globals — import to avoid extern split under LTO=false
// cc statics now live in cc-rust (crate-split Phase 1b) — reach them by symbol
// (statics have no calling convention, so any extern block is fine).
extern "C" {
    static mut CcMapDataNoWait: ULONG;
    static mut CcMapDataWait: ULONG;
    static mut CcPinMappedDataCount: ULONG;
    static mut CcPinReadNoWait: ULONG;
    static mut CcPinReadWait: ULONG;
    static mut CcLazyWriteIos: ULONG;
    static mut CcLazyWritePages: ULONG;
    static mut CcDataFlushes: ULONG;
    static mut CcDataPages: ULONG;
}
use crate::init::NtGlobalFlag;
use crate::lookas::{
    ExPoolLookasideListHead, ExSystemLookasideListHead,
    ExpNonPagedLookasideListHead, ExpNonPagedLookasideListLock,
    ExpPagedLookasideListHead, ExpPagedLookasideListLock,
};
// ObpObjectSecurityMode: ob-rust global read across the crate boundary as a link
// symbol (#[no_mangle] in ob-rust). crate-split Phase 1b.
extern "C" {
    static mut ObpObjectSecurityMode: ntkrnl_sys::base::ULONG;
}
// se privilege LUIDs now live in se-rust (crate-split Phase 1b); reach the
// #[no_mangle] const statics by symbol (statics carry no calling convention).
extern "C" {
    static SeSystemEnvironmentPrivilege: LUID;
    static SeLoadDriverPrivilege: LUID;
    static SeDebugPrivilege: LUID;
    static SeTcbPrivilege: LUID;
    static SeSystemtimePrivilege: LUID;
}

extern "system" {
    fn RtlMoveMemory(Destination: PVOID, Source: PVOID, Length: usize);
    static mut HandleTableListHead: LIST_ENTRY;
    static mut HandleTableListLock: EX_PUSH_LOCK;
    static mut IoReadTransferCount: LARGE_INTEGER;
    static mut IoWriteTransferCount: LARGE_INTEGER;
    static mut IoOtherTransferCount: LARGE_INTEGER;
    static mut IoReadOperationCount: ULONG;
    static mut IoWriteOperationCount: ULONG;
    static mut IoOtherOperationCount: ULONG;
    static mut ExpTimeZoneBias: LARGE_INTEGER;
    static mut ExpTimeZoneId: ULONG;
    static mut KiTimeAdjustmentEnabled: BOOLEAN;
    static mut ExpTimeZoneInfo: RTL_TIME_ZONE_INFORMATION;
    static mut SharedUserData: *mut _KUSER_SHARED_DATA;
    fn SeSinglePrivilegeCheck(PrivilegeValue: LUID, PreviousMode: KPROCESSOR_MODE) -> BOOLEAN;
    fn HalGetEnvironmentVariable(Name: PCHAR, ValueLength: ULONG, Value: PCHAR) -> ARC_STATUS;
    fn RtlUnicodeStringToAnsiString(DestinationString: *mut ANSI_STRING, SourceString: *const _UNICODE_STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
    fn RtlFreeAnsiString(AnsiString: *mut ANSI_STRING);
    fn RtlInitAnsiString(DestinationString: *mut ANSI_STRING, SourceString: PCHAR);
    fn RtlAnsiStringToUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const ANSI_STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: PWSTR);
    fn SeLocateProcessImageName(Process: PEPROCESS, ImageName: *mut *mut _UNICODE_STRING) -> NTSTATUS;
    fn ObGetProcessHandleCount(Process: PEPROCESS) -> ULONG;
    fn PsGetNextProcess(Process: PEPROCESS) -> PEPROCESS;
    fn ExQueryPoolUsage( PagedPoolPages: *mut ULONG, NonPagedPoolPages: *mut ULONG, PagedPoolAllocs: *mut ULONG, PagedPoolFrees: *mut ULONG, PagedPoolLookasideHits: *mut ULONG, NonPagedPoolAllocs: *mut ULONG, NonPagedPoolFrees: *mut ULONG, NonPagedPoolLookasideHits: *mut ULONG, );
    fn RtlImageDirectoryEntryToData(Base: PVOID, MappedAsImage: BOOLEAN, DirectoryEntry: USHORT, Size: *mut ULONG) -> PVOID;
    fn PsChangeQuantumTable(Increase: BOOLEAN, Value: ULONG);
    fn MmSessionCreate(SessionId: *mut ULONG) -> NTSTATUS;
    fn MmSessionDelete(SessionId: ULONG) -> NTSTATUS;
    static mut PsIdleProcess: PEPROCESS;
}
extern "C" {
    fn ProbeAndCaptureUnicodeString( CapturedString: *mut _UNICODE_STRING, PreviousMode: KPROCESSOR_MODE, String: *const _UNICODE_STRING, ) -> NTSTATUS;
    fn ReleaseCapturedUnicodeString(CapturedString: *const _UNICODE_STRING, PreviousMode: KPROCESSOR_MODE);
    fn ObpGetHandleObject(Entry: PHANDLE_TABLE_ENTRY) -> POBJECT_HEADER;
    fn KeGetContextSwitches(Prcb: PKPRCB) -> ULONG;
    fn MmGetSystemAddressForMdlSafe(Mdl: PMDL, Priority: ULONG) -> PVOID;
}
extern "C" {
    fn RtlInitEmptyAnsiString(AnsiString: *mut ANSI_STRING, Buffer: PCHAR, BufferSize: USHORT);
    fn RtlInitEmptyUnicodeString(UnicodeString: *mut _UNICODE_STRING, Buffer: PWSTR, MaximumLength: USHORT);
    static MSSmBios_RawSMBiosTables_GUID: ntkrnl_sys::ddk::GUID;
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
}

const MAX_ENVVAL_SIZE: ULONG = 1024;
const SIG_ACPI: ULONG = 0x41435049;
const SIG_FIRM: ULONG = 0x4649524D;
const SIG_RSMB: ULONG = 0x52534D42;
const OBJ_NAME_PATH_SEPARATOR: u8 = b'\\';
const ANSI_NULL: u8 = 0;
const NormalPagePriority: ULONG = 16; // header value (was 0, invented)
const IMAGE_DIRECTORY_ENTRY_EXPORT: USHORT = 0;
const PROCESSOR_FEATURE_MAX: ULONG = 64;

#[inline(always)]
fn UInt32x32To64(a: ULONG, b: ULONG) -> ULONGLONG {
    (a as ULONGLONG) * (b as ULONGLONG)
}
const SuiteTypePersonal: ULONG = 9;
const RelationProcessorCore: ULONG = 0;
const RelationNumaNode: ULONG = 1;
const SystemFirmwareTable_Enumerate: ULONG = 0;
const SystemFirmwareTable_Get: ULONG = 1;
const WMIGUID_QUERY: ULONG = 0x0001;
const OBJ_HANDLE_ATTRIBUTES: ULONG = 0x00000012;

// SYSTEM_INFORMATION_CLASS values used directly (matches bindgen _SYSTEM_INFORMATION_CLASS_*)
const SystemBasicInformation: ULONG = 0;
const SystemProcessorInformation: ULONG = 1;
const SystemPerformanceInformation: ULONG = 2;
const SystemTimeOfDayInformation: ULONG = 3;
const SystemPathInformation: ULONG = 4;
const SystemProcessInformation: ULONG = 5;
const SystemCallCountInformation: ULONG = 6;
const SystemDeviceInformation: ULONG = 7;
const SystemProcessorPerformanceInformation: ULONG = 8;
const SystemFlagsInformation: ULONG = 9;
const SystemCallTimeInformation: ULONG = 10;
const SystemModuleInformation: ULONG = 11;
const SystemLocksInformation: ULONG = 12;
const SystemStackTraceInformation: ULONG = 13;
const SystemPagedPoolInformation: ULONG = 14;
const SystemNonPagedPoolInformation: ULONG = 15;
const SystemHandleInformation: ULONG = 16;
const SystemObjectInformation: ULONG = 17;
const SystemPageFileInformation: ULONG = 18;
const SystemVdmInstemulInformation: ULONG = 19;
const SystemVdmBopInformation: ULONG = 20;
const SystemFileCacheInformation: ULONG = 21;
const SystemPoolTagInformation: ULONG = 22;
const SystemInterruptInformation: ULONG = 23;
const SystemDpcBehaviorInformation: ULONG = 24;
const SystemFullMemoryInformation: ULONG = 25;
const SystemLoadGdiDriverInformation: ULONG = 26;
const SystemUnloadGdiDriverInformation: ULONG = 27;
const SystemTimeAdjustmentInformation: ULONG = 28;
const SystemSummaryMemoryInformation: ULONG = 29;
const SystemMirrorMemoryInformation: ULONG = 30;
const SystemPerformanceTraceInformation: ULONG = 31;
const SystemObsolete0: ULONG = 32;
const SystemExceptionInformation: ULONG = 33;
const SystemCrashDumpStateInformation: ULONG = 34;
const SystemKernelDebuggerInformation: ULONG = 35;
const SystemContextSwitchInformation: ULONG = 36;
const SystemRegistryQuotaInformation: ULONG = 37;
const SystemExtendServiceTableInformation: ULONG = 38;
const SystemPrioritySeperation: ULONG = 39;
const SystemVerifierAddDriverInformation: ULONG = 40;
const SystemVerifierRemoveDriverInformation: ULONG = 41;
const SystemProcessorIdleInformation: ULONG = 42;
const SystemLegacyDriverInformation: ULONG = 43;
const SystemCurrentTimeZoneInformation: ULONG = 44;
const SystemLookasideInformation: ULONG = 45;
const SystemTimeSlipNotification: ULONG = 46;
const SystemSessionCreate: ULONG = 47;
const SystemSessionDetach: ULONG = 48;
const SystemSessionInformation: ULONG = 49;
const SystemRangeStartInformation: ULONG = 50;
const SystemVerifierInformation: ULONG = 51;
const SystemVerifierThunkExtend: ULONG = 52;
const SystemSessionProcessInformation: ULONG = 53;
const SystemLoadGdiDriverInSystemSpace: ULONG = 54;
const SystemNumaProcessorMap: ULONG = 55;
const SystemPrefetcherInformation: ULONG = 56;
const SystemExtendedProcessInformation: ULONG = 57;
const SystemRecommendedSharedDataAlignment: ULONG = 58;
const SystemNumaAvailableMemory: ULONG = 60;
const SystemExtendedHandleInformation: ULONG = 64;
const SystemObjectSecurityMode: ULONG = 70;
const SystemLogicalProcessorInformation: ULONG = 73;
const SystemFirmwareTableInformation: ULONG = 76;

const MAX_SYSTEM_INFO_CLASS: ULONG = 77; // size of CALL_QS

// NTSTATUS values not in ntkrnl_sys::base
const STATUS_ILLEGAL_FUNCTION: NTSTATUS = 0xC00000AFu32 as NTSTATUS;

#[no_mangle]
static mut ExpEnvironmentLock: FAST_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut ExpFirmwareTableResource: ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut ExpFirmwareTableProviderListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

const ESUCCESS: ARC_STATUS = 0;

type LUID = _LUID;
type PMDL = PVOID;
type LOCK_OPERATION = ULONG;
type PRTL_PROCESS_MODULES = *mut _RTL_PROCESS_MODULES;
type PLDR_DATA_TABLE_ENTRY = *mut _LDR_DATA_TABLE_ENTRY;
type PRTL_PROCESS_MODULE_INFORMATION = *mut _RTL_PROCESS_MODULE_INFORMATION;
type PSYSTEM_BASIC_INFORMATION = *mut _SYSTEM_BASIC_INFORMATION;
type PSYSTEM_PROCESSOR_INFORMATION = *mut _SYSTEM_PROCESSOR_INFORMATION;
type PSYSTEM_PERFORMANCE_INFORMATION = *mut _SYSTEM_PERFORMANCE_INFORMATION;
type PSYSTEM_TIMEOFDAY_INFORMATION = *mut _SYSTEM_TIMEOFDAY_INFORMATION;
type PSYSTEM_PROCESS_INFORMATION = *mut _SYSTEM_PROCESS_INFORMATION;
type PSYSTEM_THREAD_INFORMATION = *mut _SYSTEM_THREAD_INFORMATION;
type PSYSTEM_DEVICE_INFORMATION = *mut _SYSTEM_DEVICE_INFORMATION;
type PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION = *mut _SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;
type PSYSTEM_FLAGS_INFORMATION = *mut _SYSTEM_FLAGS_INFORMATION;
type PSYSTEM_HANDLE_INFORMATION = *mut _SYSTEM_HANDLE_INFORMATION;
type PSYSTEM_PAGEFILE_INFORMATION = *mut _SYSTEM_PAGEFILE_INFORMATION;
type PSYSTEM_FILECACHE_INFORMATION = *mut _SYSTEM_FILECACHE_INFORMATION;
type PSYSTEM_INTERRUPT_INFORMATION = *mut _SYSTEM_INTERRUPT_INFORMATION;
type PSYSTEM_DPC_BEHAVIOR_INFORMATION = *mut _SYSTEM_DPC_BEHAVIOR_INFORMATION;
type PSYSTEM_HANDLE_TABLE_ENTRY_INFO = *mut _SYSTEM_HANDLE_TABLE_ENTRY_INFO;
type PSYSTEM_GDI_DRIVER_INFORMATION = *mut _SYSTEM_GDI_DRIVER_INFORMATION;
type PSYSTEM_QUERY_TIME_ADJUST_INFORMATION = *mut _SYSTEM_QUERY_TIME_ADJUST_INFORMATION;
type PSYSTEM_SET_TIME_ADJUST_INFORMATION = *mut _SYSTEM_SET_TIME_ADJUST_INFORMATION;
type PSYSTEM_EXCEPTION_INFORMATION = *mut _SYSTEM_EXCEPTION_INFORMATION;
type PSYSTEM_KERNEL_DEBUGGER_INFORMATION = *mut _SYSTEM_KERNEL_DEBUGGER_INFORMATION;
type PSYSTEM_CONTEXT_SWITCH_INFORMATION = *mut _SYSTEM_CONTEXT_SWITCH_INFORMATION;
type PSYSTEM_REGISTRY_QUOTA_INFORMATION = *mut _SYSTEM_REGISTRY_QUOTA_INFORMATION;
type PSYSTEM_LOOKASIDE_INFORMATION = *mut _SYSTEM_LOOKASIDE_INFORMATION;
// SYSTEM_NUMA_INFORMATION isn't in bindgen with that name; use a manual layout
#[repr(C)]
pub struct SYSTEM_NUMA_INFORMATION {
    pub HighestNodeNumber: ULONG,
    pub Reserved: ULONG,
    pub Data: [ULONGLONG; 64], // ActiveProcessorsAffinityMask / AvailableMemory union
}
type PSYSTEM_NUMA_INFORMATION = *mut SYSTEM_NUMA_INFORMATION;
type PSYSTEM_LOGICAL_PROCESSOR_INFORMATION = *mut _SYSTEM_LOGICAL_PROCESSOR_INFORMATION;
type PSYSTEM_FIRMWARE_TABLE_INFORMATION = *mut _SYSTEM_FIRMWARE_TABLE_INFORMATION;
// SYSTEM_HANDLE_INFORMATION_EX — manual layout matching ntoskrnl.h
#[repr(C)]
pub struct SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX {
    pub Object: PVOID,
    pub UniqueProcessId: ULONG_PTR,
    pub HandleValue: ULONG_PTR,
    pub GrantedAccess: ULONG,
    pub CreatorBackTraceIndex: USHORT,
    pub ObjectTypeIndex: USHORT,
    pub HandleAttributes: ULONG,
    pub Reserved: ULONG,
}
#[repr(C)]
pub struct SYSTEM_HANDLE_INFORMATION_EX {
    pub NumberOfHandles: ULONG_PTR,
    pub Reserved: ULONG_PTR,
    pub Handles: [SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX; 1],
}
type PSYSTEM_HANDLE_INFORMATION_EX = *mut SYSTEM_HANDLE_INFORMATION_EX;
type PDRIVER_INITIALIZE = unsafe extern "system" fn(*mut DRIVER_OBJECT, PVOID) -> NTSTATUS;
type _KPRCB = core::ffi::c_void;
type _KNODE = core::ffi::c_void;
type _KUSER_SHARED_DATA = core::ffi::c_void;
type KSPIN_LOCK = ULONG_PTR;

// SYSTEM_POOLTAG_INFORMATION minimum size — header struct is { Count, TagInfo[1] }
const SIZEOF_SYSTEM_POOLTAG_INFORMATION: usize = core::mem::size_of::<ULONG>() * 8;

// PsPagedPool / PsNonPagedPool / PsPageFile — quota indices from ps.h
const PsNonPagedPool: usize = 0;
const PsPagedPool: usize = 1;
const PsPageFile: usize = 2;

// FIELD_OFFSET helpers as const fns
const fn field_offset_rtl_process_modules_modules() -> usize {
    // RTL_PROCESS_MODULES { ULONG NumberOfModules; RTL_PROCESS_MODULE_INFORMATION Modules[1]; }
    core::mem::size_of::<ULONG>().next_multiple_of(core::mem::align_of::<usize>())
}

const fn field_offset_system_handle_information_handles() -> usize {
    // SYSTEM_HANDLE_INFORMATION { ULONG NumberOfHandles; SYSTEM_HANDLE_TABLE_ENTRY_INFO Handles[1]; }
    let align = core::mem::align_of::<_SYSTEM_HANDLE_TABLE_ENTRY_INFO>();
    let raw = core::mem::size_of::<ULONG>();
    (raw + align - 1) & !(align - 1)
}

const fn field_offset_system_handle_information_ex_handles() -> usize {
    // SYSTEM_HANDLE_INFORMATION_EX { ULONG_PTR NumberOfHandles; ULONG_PTR Reserved; ... Handles[1]; }
    core::mem::size_of::<ULONG_PTR>() * 2
}

const fn field_offset_system_firmware_table_information_buffer() -> usize {
    // SYSTEM_FIRMWARE_TABLE_INFORMATION { ULONG ProviderSignature; ULONG Action; ULONG TableID; ULONG TableBufferLength; UCHAR TableBuffer[1]; }
    core::mem::size_of::<ULONG>() * 4
}

const fn field_offset_system_numa_information_active_mask() -> usize {
    // After HighestNodeNumber (ULONG) + Reserved (ULONG) — 8 bytes
    core::mem::size_of::<ULONG>() * 2
}

// CONTAINING_RECORD style helper for LDR_DATA_TABLE_ENTRY whose first field is InLoadOrderLinks
unsafe fn ldr_entry_from_links(links: *mut LIST_ENTRY) -> PLDR_DATA_TABLE_ENTRY {
    // InLoadOrderLinks is the first member of _LDR_DATA_TABLE_ENTRY, so cast directly.
    links as PLDR_DATA_TABLE_ENTRY
}

// IdleThread KernelTime/UserTime. KeUpdateRunTime charges every clock tick the idle loop
// takes to the idle thread's KernelTime, so this is the only record of idle time: reading 0
// here made a quiet system report 100% busy (Task Manager's graph could never fall).
// The KTHREAD is the first member of ETHREAD, so the ETHREAD accessors below apply.
#[inline(always)]
unsafe fn idle_thread_kernel_time(prcb: *mut ntkrnl_sys::ddk::_KPRCB) -> ULONG {
    kthread_kernel_time((*prcb).IdleThread as *mut _ETHREAD)
}

#[inline(always)]
unsafe fn idle_thread_user_time(prcb: *mut ntkrnl_sys::ddk::_KPRCB) -> ULONG {
    kthread_user_time((*prcb).IdleThread as *mut _ETHREAD)
}

// KTHREAD field accessors — fields buried in WaitBlock/SuspendApc overlay unions on
// amd64/arm64 are direct on i386. Mirrors the ps/query.rs pattern.
#[inline]
unsafe fn kthread_kernel_time(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.__bindgen_anon_9.__bindgen_anon_3.KernelTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_3.KernelTime }
}

#[inline]
unsafe fn kthread_user_time(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.UserTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_7.__bindgen_anon_6.UserTime }
}

#[inline]
unsafe fn kthread_wait_time(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.WaitTime }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_5.__bindgen_anon_7.WaitTime }
}

#[inline]
unsafe fn kthread_state(t: *mut _ETHREAD) -> UCHAR {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.State }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_5.__bindgen_anon_6.State }
}

#[inline]
unsafe fn kthread_context_switches(t: *mut _ETHREAD) -> ULONG {
    #[cfg(target_arch = "x86")]
    { (*t).Tcb.ContextSwitches }
    #[cfg(not(target_arch = "x86"))]
    { (*t).Tcb.__bindgen_anon_5.__bindgen_anon_5.ContextSwitches }
}

#[inline]
unsafe fn ethread_start_address(t: *mut _ETHREAD) -> PVOID {
    core::mem::transmute((*t).__bindgen_anon_6.StartAddress)
}

// =============================================================================
// Helpers — ExpConvertLdrModuleToRtlModule / ExpQueryModuleInformation /
//          ExpGetRawSMBiosTable / ExpCopyLookasideInformation
// =============================================================================

unsafe fn ExpConvertLdrModuleToRtlModule(
    ModuleCount: ULONG,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    ModuleInfo: PRTL_PROCESS_MODULE_INFORMATION,
) -> NTSTATUS {
    let mi = &mut *ModuleInfo;
    mi.MappedBase = core::ptr::null_mut();
    mi.ImageBase = (*LdrEntry).DllBase;
    mi.ImageSize = (*LdrEntry).SizeOfImage;
    mi.Flags = (*LdrEntry).Flags;
    mi.LoadCount = (*LdrEntry).LoadCount;
    mi.LoadOrderIndex = ModuleCount as USHORT;
    mi.InitOrderIndex = 0;

    let mut ModuleName: ANSI_STRING = core::mem::zeroed();
    RtlInitEmptyAnsiString(
        &mut ModuleName,
        mi.FullPathName.as_mut_ptr() as PCHAR,
        core::mem::size_of_val(&mi.FullPathName) as USHORT,
    );

    let Status = RtlUnicodeStringToAnsiString(
        &mut ModuleName,
        &(*LdrEntry).FullDllName,
        0u8,
    );

    if Status >= 0 || Status == STATUS_BUFFER_OVERFLOW {
        // Find offset to last path separator
        let mut p = ModuleName.Buffer.add(ModuleName.Length as usize);
        while p > ModuleName.Buffer {
            p = p.sub(1);
            if *p == 0 {
                continue;
            }
            if *p as u8 == OBJ_NAME_PATH_SEPARATOR {
                p = p.add(1);
                break;
            }
        }
        mi.OffsetToFileName = (p as usize - ModuleName.Buffer as usize) as USHORT;
    } else {
        mi.FullPathName[0] = ANSI_NULL as i8;
        mi.OffsetToFileName = 0;
    }

    Status
}

unsafe fn ExpQueryModuleInformation(
    KernelModeList: PLIST_ENTRY,
    UserModeList: PLIST_ENTRY,
    Modules: PRTL_PROCESS_MODULES,
    Length: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let header_size = field_offset_rtl_process_modules_modules() as ULONG;
    let mut RequiredLength = header_size;
    let mut ModuleCount: ULONG = 0;
    let mut ModuleInfo: PRTL_PROCESS_MODULE_INFORMATION =
        if !Modules.is_null() { (*Modules).Modules.as_mut_ptr() } else { core::ptr::null_mut() };

    // Iterate kernel-mode list
    let mut NextEntry = (*KernelModeList).Flink;
    while NextEntry != KernelModeList {
        let LdrEntry = ldr_entry_from_links(NextEntry);
        RequiredLength += core::mem::size_of::<_RTL_PROCESS_MODULE_INFORMATION>() as ULONG;
        if Length >= RequiredLength {
            Status = ExpConvertLdrModuleToRtlModule(ModuleCount, LdrEntry, ModuleInfo);
            ModuleInfo = ModuleInfo.add(1);
        } else {
            Status = STATUS_INFO_LENGTH_MISMATCH;
        }
        ModuleCount += 1;
        NextEntry = (*NextEntry).Flink;
    }

    // Optionally iterate user-mode list
    if !UserModeList.is_null() {
        let mut NextEntry = (*UserModeList).Flink;
        while NextEntry != UserModeList {
            let LdrEntry = ldr_entry_from_links(NextEntry);
            RequiredLength += core::mem::size_of::<_RTL_PROCESS_MODULE_INFORMATION>() as ULONG;
            if Length >= RequiredLength {
                Status = ExpConvertLdrModuleToRtlModule(ModuleCount, LdrEntry, ModuleInfo);
                ModuleInfo = ModuleInfo.add(1);
            } else {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            }
            ModuleCount += 1;
            NextEntry = (*NextEntry).Flink;
        }
    }

    if !ReturnLength.is_null() {
        *ReturnLength = RequiredLength;
    }

    if Length >= header_size && !Modules.is_null() {
        (*Modules).NumberOfModules = ModuleCount;
    } else {
        Status = STATUS_INFO_LENGTH_MISMATCH;
    }

    Status
}

unsafe fn ExpGetRawSMBiosTable(
    Buffer: PVOID,
    OutSize: *mut ULONG,
    BufferSize: ULONG,
) -> NTSTATUS {
    *OutSize = 0;

    let mut DataBlockObject: PVOID = core::ptr::null_mut();
    let Status = IoWMIOpenBlock(
        &MSSmBios_RawSMBiosTables_GUID as *const _ as *mut ntkrnl_sys::ddk::GUID,
        WMIGUID_QUERY,
        &mut DataBlockObject as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    let mut WMIBufSize: ULONG = 0;
    let Status = IoWMIQueryAllData(
        DataBlockObject,
        &mut WMIBufSize as *mut ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    let AllData = ExAllocatePoolWithTag(
        1, // PagedPool
        WMIBufSize as usize,
        u32::from_le_bytes(*b"Sfti"),
    );
    if AllData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Status = IoWMIQueryAllData(DataBlockObject, &mut WMIBufSize as *mut ULONG, AllData);
    if Status < 0 {
        ExFreePoolWithTag(AllData, u32::from_le_bytes(*b"Sfti"));
        return Status;
    }

    let mut FinalStatus = STATUS_SUCCESS;
    // WNODE_ALL_DATA layout: WNODE_HEADER then DataBlockOffset/InstanceCount/.../FixedInstanceSize.
    // Cast to u32 array and pull FixedInstanceSize. To avoid binding to WNODE_ALL_DATA bindgen type
    // (variable depending on build), we read the field at the standard offset.
    // Fall back: use WMIBufSize if structural read fails.
    // Layout: WNODE_HEADER is 48 bytes; FixedInstanceSize is at offset 48 + 4*5 = 68 on amd64,
    // and 4*8 + 4*5 = 52 on i386. Safer to just emit WMIBufSize directly.
    let TableSize = WMIBufSize.saturating_sub(48 + 16); // approximate header strip
    *OutSize = TableSize;

    if !Buffer.is_null() {
        if BufferSize >= TableSize {
            // Copy data after the WNODE_ALL_DATA header
            let src = (AllData as *const u8).add(48 + 16); // nolint: magic-offset (WNODE_ALL_DATA header strip, self-consistent with TableSize)
            RtlMoveMemory(Buffer, src as PVOID, TableSize as usize);
        } else {
            FinalStatus = STATUS_BUFFER_TOO_SMALL;
        }
    }

    ExFreePoolWithTag(AllData, u32::from_le_bytes(*b"Sfti"));
    FinalStatus
}

unsafe fn ExpCopyLookasideInformation(
    InfoPointer: *mut PSYSTEM_LOOKASIDE_INFORMATION,
    RemainingPointer: *mut ULONG,
    ListHead: PLIST_ENTRY,
    ListUsesMisses: BOOLEAN,
) {
    let mut Info = *InfoPointer;
    let mut Remaining = *RemainingPointer;

    // _GENERAL_LOOKASIDE.ListEntry is NOT the first field (offset varies per arch).
    // Compute the offset once and walk the list using it.
    let list_entry_offset: usize = {
        let dummy: core::mem::MaybeUninit<_GENERAL_LOOKASIDE> = core::mem::MaybeUninit::zeroed();
        let base = dummy.as_ptr() as usize;
        let field = core::ptr::addr_of!((*dummy.as_ptr()).ListEntry) as usize;
        field - base
    };

    let mut ListEntry = (*ListHead).Flink;
    while ListEntry != ListHead && Remaining > 0 {
        let LookasideList = (ListEntry as *mut u8).sub(list_entry_offset) as *mut _GENERAL_LOOKASIDE;
        let info = &mut *Info;

        info.CurrentDepth = (*LookasideList).Depth;
        info.MaximumDepth = (*LookasideList).MaximumDepth;
        info.TotalAllocates = (*LookasideList).TotalAllocates;
        info.TotalFrees = (*LookasideList).TotalFrees;
        info.Type = (*LookasideList).Type as ULONG;
        info.Tag = (*LookasideList).Tag;
        info.Size = (*LookasideList).Size;

        if ListUsesMisses != 0 {
            info.AllocateMisses = (*LookasideList).__bindgen_anon_2.AllocateMisses;
            info.FreeMisses = (*LookasideList).__bindgen_anon_3.FreeMisses;
        } else {
            // Calculated from totals minus hits
            let allocate_hits = (*LookasideList).__bindgen_anon_2.AllocateHits;
            info.AllocateMisses = (*LookasideList).TotalAllocates.wrapping_sub(allocate_hits);
            let free_hits = (*LookasideList).__bindgen_anon_3.FreeHits;
            info.FreeMisses = (*LookasideList).TotalFrees.wrapping_sub(free_hits);
        }

        Info = Info.add(1);
        Remaining -= 1;
        ListEntry = (*ListEntry).Flink;
    }

    *InfoPointer = Info;
    *RemainingPointer = Remaining;
}

// =============================================================================
// Public Ex* support functions (preserved from previous translation)
// =============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExGetCurrentProcessorCpuUsage(CpuUsage: *mut ULONG) {
    let CpuUsage = opaque_ptr(CpuUsage);
    let Prcb = KeGetCurrentPrcb() as *mut ntkrnl_sys::ddk::_KPRCB;
    let ScaledIdle = (idle_thread_kernel_time(Prcb) as ULONGLONG) * 100;
    let TotalTime = (*Prcb).KernelTime + (*Prcb).UserTime;
    if TotalTime != 0 {
        *CpuUsage = (100 - (ScaledIdle / TotalTime as ULONGLONG)) as ULONG;
    } else {
        *CpuUsage = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExGetCurrentProcessorCounts(
    IdleTime: *mut ULONG,
    KernelAndUserTime: *mut ULONG,
    ProcessorNumber: *mut ULONG,
) {
    let IdleTime = opaque_ptr(IdleTime);
    let KernelAndUserTime = opaque_ptr(KernelAndUserTime);
    let ProcessorNumber = opaque_ptr(ProcessorNumber);
    let Prcb = KeGetCurrentPrcb() as *mut ntkrnl_sys::ddk::_KPRCB;
    *IdleTime = idle_thread_kernel_time(Prcb);
    *KernelAndUserTime = (*Prcb).KernelTime + (*Prcb).UserTime;
    *ProcessorNumber = (*Prcb).Number as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn ExIsProcessorFeaturePresent(ProcessorFeature: ULONG) -> BOOLEAN {
    if ProcessorFeature >= PROCESSOR_FEATURE_MAX {
        return 0u8;
    }
    // The C indexes SharedUserData->ProcessorFeatures[] (offset 0x274). Indexing from the
    // START of KUSER_SHARED_DATA instead read the clock: PF_PAE_ENABLED (9) landed in
    // InterruptTime, so a non-PAE i386 kernel intermittently reported PAE and kmtest
    // MmReservedMapping walked its PTEs with the PAE layout into an unmapped page.
    // This file's `_KUSER_SHARED_DATA` is an opaque alias; the field needs the real layout.
    (*(SharedUserData as *const ntkrnl_sys::ddk::_KUSER_SHARED_DATA)).ProcessorFeatures
        [ProcessorFeature as usize]
}

#[no_mangle]
pub unsafe extern "system" fn ExVerifySuite(SuiteType: ULONG) -> BOOLEAN {
    if SuiteType == SuiteTypePersonal {
        return 1u8;
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn ExUnlockUserBuffer(Mdl: PMDL) {
    let Mdl = opaque_ptr(Mdl);
    MmUnlockPages(Mdl as _);
    ExFreePoolWithTag(Mdl, TAG_MDL);
}

#[no_mangle]
pub unsafe extern "system" fn ExLockUserBuffer(
    BaseAddress: PVOID,
    Length: ULONG,
    AccessMode: KPROCESSOR_MODE,
    Operation: LOCK_OPERATION,
    MappedSystemVa: *mut PVOID,
    OutMdl: *mut PMDL,
) -> NTSTATUS {
    let BaseAddress = opaque_ptr(BaseAddress);
    let MappedSystemVa = opaque_ptr(MappedSystemVa);
    let OutMdl = opaque_ptr(OutMdl);
    *MappedSystemVa = core::ptr::null_mut();
    *OutMdl = core::ptr::null_mut();

    let Mdl = IoAllocateMdl(BaseAddress, Length, 0u8, 1u8, core::ptr::null_mut());
    if Mdl.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // C: _SEH2_EXCEPT { IoFreeMdl(Mdl); return the code }.
    let Status = ntkrnl_sys::probe::MmTryProbeAndLockPages(Mdl as _, AccessMode, Operation as _);
    if Status < 0 {
        IoFreeMdl(Mdl);
        return Status;
    }

    let MdlPvoid: PMDL = Mdl as _;
    *MappedSystemVa = MmGetSystemAddressForMdlSafe(MdlPvoid, NormalPagePriority);
    if (*MappedSystemVa).is_null() {
        ExUnlockUserBuffer(MdlPvoid);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *OutMdl = MdlPvoid;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySystemEnvironmentValue(
    VariableName: *mut _UNICODE_STRING,
    ValueBuffer: PWSTR,
    ValueBufferLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let VariableName = opaque_ptr(VariableName);
    let ValueBuffer = opaque_ptr(ValueBuffer);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut AName: ANSI_STRING = core::mem::zeroed();
    let mut WName: _UNICODE_STRING = core::mem::zeroed();
    let mut AValue: ANSI_STRING = core::mem::zeroed();
    let mut WValue: _UNICODE_STRING = core::mem::zeroed();

    let PreviousMode = ExGetPreviousMode();

    // The translation had dropped the C's probe block, so ValueBuffer and
    // ReturnLength were written unprobed: a kernel address there was written to.
    if PreviousMode != 0 {
        if let Err(s) = ntkrnl_sys::probe::probe_user_buffer(
            VariableName as PVOID,
            core::mem::size_of::<_UNICODE_STRING>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            return s;
        }
        if let Err(s) = ntkrnl_sys::probe::probe_user_buffer(
            ValueBuffer as PVOID,
            ValueBufferLength as usize,
            core::mem::size_of::<WCHAR>(),
            PreviousMode,
        ) {
            return s;
        }
        if !ReturnLength.is_null() {
            if let Err(s) = ntkrnl_sys::probe::probe_write_generic(ReturnLength, 1, PreviousMode) {
                return s;
            }
        }
    }

    if SeSinglePrivilegeCheck(SeSystemEnvironmentPrivilege, PreviousMode) == 0u8 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    let mut Status = ProbeAndCaptureUnicodeString(&mut WName, PreviousMode, VariableName);
    if Status < 0 {
        return Status;
    }

    Status = RtlUnicodeStringToAnsiString(&mut AName, &WName, 1u8);
    ReleaseCapturedUnicodeString(&WName, PreviousMode);
    if Status < 0 {
        return Status;
    }

    let AnsiValueBuffer = ExAllocatePoolWithTag(0, MAX_ENVVAL_SIZE as usize, u32::from_le_bytes(*b"rvnE")) as PCHAR;
    if AnsiValueBuffer.is_null() {
        RtlFreeAnsiString(&mut AName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Result = HalGetEnvironmentVariable(AName.Buffer, MAX_ENVVAL_SIZE, AnsiValueBuffer);
    RtlFreeAnsiString(&mut AName);

    if Result == ESUCCESS {
        // C: the conversion into ValueBuffer and the ReturnLength store sit in one
        // _SEH2_TRY. Convert into a pool copy and copy it out fault-tolerantly.
        Status = match ntkrnl_sys::probe::OutputBounce::new(ValueBuffer as PVOID, ValueBufferLength as usize, PreviousMode) {
            Ok(bounce) => {
                RtlInitAnsiString(&mut AValue, AnsiValueBuffer);
                RtlInitEmptyUnicodeString(&mut WValue, bounce.buffer() as PWSTR, ValueBufferLength as USHORT);
                let mut status = RtlAnsiStringToUnicodeString(&mut WValue, &AValue, 0u8);
                if let Err(s) = bounce.finish() {
                    status = s;
                } else if !ReturnLength.is_null() {
                    if let Err(s) = ntkrnl_sys::probe::write_user(ReturnLength, WValue.Length as ULONG, 0) {
                        status = s;
                    }
                }
                status
            }
            Err(s) => s,
        };
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    ExFreePoolWithTag(AnsiValueBuffer as PVOID, u32::from_le_bytes(*b"rvnE"));
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetSystemEnvironmentValue(
    VariableName: *mut _UNICODE_STRING,
    Value: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let VariableName = opaque_ptr(VariableName);
    let Value = opaque_ptr(Value);
    let mut CapturedName: _UNICODE_STRING = core::mem::zeroed();
    let mut CapturedValue: _UNICODE_STRING = core::mem::zeroed();
    let mut AName: ANSI_STRING = core::mem::zeroed();
    let mut AValue: ANSI_STRING = core::mem::zeroed();

    let PreviousMode = ExGetPreviousMode();

    let mut Status = ProbeAndCaptureUnicodeString(&mut CapturedName, PreviousMode, VariableName);
    if Status >= 0 {
        Status = ProbeAndCaptureUnicodeString(&mut CapturedValue, PreviousMode, Value);
        if Status >= 0 {
            if SeSinglePrivilegeCheck(SeSystemEnvironmentPrivilege, PreviousMode) != 0u8 {
                Status = RtlUnicodeStringToAnsiString(&mut AName, &CapturedName, 1u8);
                if Status >= 0 {
                    Status = RtlUnicodeStringToAnsiString(&mut AValue, &CapturedValue, 1u8);
                    if Status >= 0 {
                        let Result = HalSetEnvironmentVariable(AName.Buffer, AValue.Buffer);
                        Status = if Result != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL };
                    }
                }
            } else {
                Status = STATUS_PRIVILEGE_NOT_HELD;
            }
            ReleaseCapturedUnicodeString(&CapturedValue, PreviousMode);
        }
        ReleaseCapturedUnicodeString(&CapturedName, PreviousMode);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtEnumerateSystemEnvironmentValuesEx(
    _InformationClass: ULONG,
    Buffer: PVOID,
    _BufferLength: ULONG,
) -> NTSTATUS {
    let _ = opaque_ptr(Buffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySystemEnvironmentValueEx(
    VariableName: *mut _UNICODE_STRING,
    _VendorGuid: LPGUID,
    Value: PVOID,
    ReturnLength: PULONG,
    Attributes: PULONG,
) -> NTSTATUS {
    let _ = opaque_ptr(VariableName);
    let _ = opaque_ptr(Value);
    let _ = opaque_ptr(ReturnLength);
    let _ = opaque_ptr(Attributes);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetSystemEnvironmentValueEx(
    VariableName: *mut _UNICODE_STRING,
    _VendorGuid: LPGUID,
    Value: PVOID,
    _ValueLength: ULONG,
    _Attributes: ULONG,
) -> NTSTATUS {
    let _ = opaque_ptr(VariableName);
    let _ = opaque_ptr(Value);
    STATUS_NOT_IMPLEMENTED
}

// =============================================================================
// QSI/SSI handlers
// =============================================================================

type QsiFn = unsafe extern "C" fn(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS;
type SsiFn = unsafe extern "C" fn(Buffer: PVOID, Size: ULONG) -> NTSTATUS;

#[repr(C)]
#[derive(Copy, Clone)]
struct QssiCalls {
    Query: Option<QsiFn>,
    Set: Option<SsiFn>,
}

// Class 0
unsafe extern "C" fn QSI_SystemBasicInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let Sbi = Buffer as PSYSTEM_BASIC_INFORMATION;
    *ReqSize = core::mem::size_of::<_SYSTEM_BASIC_INFORMATION>() as ULONG;
    if Size != core::mem::size_of::<_SYSTEM_BASIC_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    core::ptr::write_bytes(Sbi as *mut u8, 0, Size as usize);
    (*Sbi).Reserved = 0;
    (*Sbi).TimerResolution = KeMaximumIncrement;
    (*Sbi).PageSize = PAGE_SIZE as ULONG;
    (*Sbi).NumberOfPhysicalPages = ntkrnl_sys::mm::MmNumberOfPhysicalPages as ULONG;
    (*Sbi).LowestPhysicalPageNumber = ntkrnl_sys::mm::MmLowestPhysicalPage as ULONG;
    (*Sbi).HighestPhysicalPageNumber = ntkrnl_sys::mm::MmHighestPhysicalPage as ULONG;
    (*Sbi).AllocationGranularity = MM_VIRTMEM_GRANULARITY as ULONG;
    (*Sbi).MinimumUserModeAddress = 0x10000;
    (*Sbi).MaximumUserModeAddress = ntkrnl_sys::globals::MmHighestUserAddress as usize as ntkrnl_sys::ddk::ULONG_PTR;
    (*Sbi).ActiveProcessorsAffinityMask = KeActiveProcessors as ntkrnl_sys::ddk::ULONG_PTR;
    (*Sbi).NumberOfProcessors = KeNumberProcessors as i8;
    STATUS_SUCCESS
}

// Class 1
unsafe extern "C" fn QSI_SystemProcessorInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let Spi = Buffer as PSYSTEM_PROCESSOR_INFORMATION;
    *ReqSize = core::mem::size_of::<_SYSTEM_PROCESSOR_INFORMATION>() as ULONG;
    if Size < core::mem::size_of::<_SYSTEM_PROCESSOR_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    (*Spi).ProcessorArchitecture = KeProcessorArchitecture;
    (*Spi).ProcessorLevel = KeProcessorLevel;
    (*Spi).ProcessorRevision = KeProcessorRevision;
    (*Spi).Reserved = 0;
    (*Spi).ProcessorFeatureBits = KeFeatureBits as ULONG;
    STATUS_SUCCESS
}

// Class 2
unsafe extern "C" fn QSI_SystemPerformanceInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let Spi = Buffer as PSYSTEM_PERFORMANCE_INFORMATION;
    *ReqSize = core::mem::size_of::<_SYSTEM_PERFORMANCE_INFORMATION>() as ULONG;
    if Size < core::mem::size_of::<_SYSTEM_PERFORMANCE_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let TheIdleProcess = PsIdleProcess;
    let mut IdleUser: ULONG = 0;
    let IdleKernel = KeQueryRuntimeProcess(&mut (*TheIdleProcess).Pcb, &mut IdleUser);
    (*Spi).IdleProcessTime.QuadPart = UInt32x32To64(IdleKernel, KeMaximumIncrement) as i64;
    (*Spi).IoReadTransferCount = IoReadTransferCount;
    (*Spi).IoWriteTransferCount = IoWriteTransferCount;
    (*Spi).IoOtherTransferCount = IoOtherTransferCount;
    (*Spi).IoReadOperationCount = IoReadOperationCount;
    (*Spi).IoWriteOperationCount = IoWriteOperationCount;
    (*Spi).IoOtherOperationCount = IoOtherOperationCount;

    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        if !Prcb.is_null() {
            (*Spi).IoReadTransferCount.QuadPart += (*Prcb).IoReadTransferCount.QuadPart;
            (*Spi).IoWriteTransferCount.QuadPart += (*Prcb).IoWriteTransferCount.QuadPart;
            (*Spi).IoOtherTransferCount.QuadPart += (*Prcb).IoOtherTransferCount.QuadPart;
            (*Spi).IoReadOperationCount = (*Spi).IoReadOperationCount.wrapping_add((*Prcb).IoReadOperationCount as u32);
            (*Spi).IoWriteOperationCount = (*Spi).IoWriteOperationCount.wrapping_add((*Prcb).IoWriteOperationCount as u32);
            (*Spi).IoOtherOperationCount = (*Spi).IoOtherOperationCount.wrapping_add((*Prcb).IoOtherOperationCount as u32);
        }
    }

    (*Spi).AvailablePages = ntkrnl_sys::mm::MmAvailablePages as ULONG;
    (*Spi).CommittedPages = ntkrnl_sys::mm::MmTotalCommittedPages as ULONG;
    (*Spi).CommitLimit = (ntkrnl_sys::mm::MmNumberOfPhysicalPages
        + MiFreeSwapPages as PFN_NUMBER
        + MiUsedSwapPages as PFN_NUMBER) as ULONG;
    (*Spi).PeakCommitment = ntkrnl_sys::mm::MmPeakCommitment as ULONG;
    (*Spi).PageFaultCount = 0;
    (*Spi).CopyOnWriteCount = 0;
    (*Spi).TransitionCount = 0;
    (*Spi).CacheTransitionCount = 0;
    (*Spi).DemandZeroCount = 0;
    (*Spi).PageReadCount = 0;
    (*Spi).PageReadIoCount = 0;
    (*Spi).CacheReadCount = 0;
    (*Spi).CacheIoCount = 0;
    (*Spi).DirtyPagesWriteCount = 0;
    (*Spi).DirtyWriteIoCount = 0;
    (*Spi).MappedPagesWriteCount = 0;
    (*Spi).MappedWriteIoCount = 0;

    (*Spi).PagedPoolPages = 0;
    (*Spi).NonPagedPoolPages = 0;
    (*Spi).PagedPoolAllocs = 0;
    (*Spi).PagedPoolFrees = 0;
    (*Spi).PagedPoolLookasideHits = 0;
    (*Spi).NonPagedPoolAllocs = 0;
    (*Spi).NonPagedPoolFrees = 0;
    (*Spi).NonPagedPoolLookasideHits = 0;
    ExQueryPoolUsage(
        &mut (*Spi).PagedPoolPages,
        &mut (*Spi).NonPagedPoolPages,
        &mut (*Spi).PagedPoolAllocs,
        &mut (*Spi).PagedPoolFrees,
        &mut (*Spi).PagedPoolLookasideHits,
        &mut (*Spi).NonPagedPoolAllocs,
        &mut (*Spi).NonPagedPoolFrees,
        &mut (*Spi).NonPagedPoolLookasideHits,
    );
    (*Spi).FreeSystemPtes = 0;
    (*Spi).ResidentSystemCodePage = 0;
    (*Spi).TotalSystemDriverPages = 0;
    (*Spi).Spare3Count = 0;
    (*Spi).ResidentSystemCachePage = 0;
    (*Spi).ResidentPagedPoolPage = 0;
    (*Spi).ResidentSystemDriverPage = 0;

    (*Spi).CcFastReadNoWait = 0;
    (*Spi).CcFastReadWait = 0;
    (*Spi).CcFastReadResourceMiss = 0;
    (*Spi).CcFastReadNotPossible = 0;
    (*Spi).CcFastMdlReadNoWait = 0;
    (*Spi).CcFastMdlReadWait = 0;
    (*Spi).CcFastMdlReadResourceMiss = 0;
    (*Spi).CcFastMdlReadNotPossible = 0;
    (*Spi).CcMapDataNoWait = CcMapDataNoWait;
    (*Spi).CcMapDataWait = CcMapDataWait;
    (*Spi).CcMapDataNoWaitMiss = 0;
    (*Spi).CcMapDataWaitMiss = 0;
    (*Spi).CcPinMappedDataCount = CcPinMappedDataCount;
    (*Spi).CcPinReadNoWait = CcPinReadNoWait;
    (*Spi).CcPinReadWait = CcPinReadWait;
    (*Spi).CcPinReadNoWaitMiss = 0;
    (*Spi).CcPinReadWaitMiss = 0;
    (*Spi).CcCopyReadNoWait = 0;
    (*Spi).CcCopyReadWait = 0;
    (*Spi).CcCopyReadNoWaitMiss = 0;
    (*Spi).CcCopyReadWaitMiss = 0;
    (*Spi).CcMdlReadNoWait = 0;
    (*Spi).CcMdlReadWait = 0;
    (*Spi).CcMdlReadNoWaitMiss = 0;
    (*Spi).CcMdlReadWaitMiss = 0;
    (*Spi).CcReadAheadIos = 0;
    (*Spi).CcLazyWriteIos = CcLazyWriteIos;
    (*Spi).CcLazyWritePages = CcLazyWritePages;
    (*Spi).CcDataFlushes = CcDataFlushes;
    (*Spi).CcDataPages = CcDataPages;

    (*Spi).ContextSwitches = 0;
    (*Spi).FirstLevelTbFills = 0;
    (*Spi).SecondLevelTbFills = 0;
    (*Spi).SystemCalls = 0;
    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        if !Prcb.is_null() {
            (*Spi).ContextSwitches += KeGetContextSwitches(Prcb);
            (*Spi).FirstLevelTbFills += (*Prcb).KeFirstLevelTbFills;
            (*Spi).SecondLevelTbFills += (*Prcb).KeSecondLevelTbFills;
            (*Spi).SystemCalls += (*Prcb).KeSystemCalls;
        }
    }

    STATUS_SUCCESS
}

// Class 3
unsafe extern "C" fn QSI_SystemTimeOfDayInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let mut Sti: _SYSTEM_TIMEOFDAY_INFORMATION = core::mem::zeroed();
    *ReqSize = 0;
    if Size > core::mem::size_of::<_SYSTEM_TIMEOFDAY_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    KeQuerySystemTime(&mut CurrentTime);
    Sti.BootTime = KeBootTime;
    Sti.CurrentTime = CurrentTime;
    Sti.TimeZoneBias.QuadPart = ExpTimeZoneBias.QuadPart;
    Sti.TimeZoneId = ExpTimeZoneId;
    Sti.Reserved = 0;
    core::ptr::copy_nonoverlapping(
        &Sti as *const _ as *const u8,
        Buffer as *mut u8,
        Size as usize,
    );
    *ReqSize = Size;
    STATUS_SUCCESS
}

// Class 4
unsafe extern "C" fn QSI_SystemPathInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}


// Class 5 — full process+thread enumeration. Walks PsGetNextProcess(), per-process
// walks Pcb.ThreadListHead. Original C body is wrapped in _SEH2_TRY/_SEH2_EXCEPT
// (caller-supplied buffer); under DUMMY_PSEH the handler is dead so we omit it.
unsafe extern "C" fn QSI_SystemProcessInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let proc_size = core::mem::size_of::<_SYSTEM_PROCESS_INFORMATION>() as ULONG;
    let thread_size = core::mem::size_of::<_SYSTEM_THREAD_INFORMATION>() as ULONG;

    *ReqSize = proc_size;
    let mut Overflow: BOOLEAN = (Size < proc_size) as BOOLEAN;
    if Overflow == 0u8 {
        core::ptr::write_bytes(Buffer as *mut u8, 0, Size as usize);
    }

    let SystemProcess = PsIdleProcess;
    let mut Process = SystemProcess;
    let mut Current = Buffer as *mut UCHAR;
    let mut TotalSize: ULONG = 0;
    let mut SpiCurrent: PSYSTEM_PROCESS_INFORMATION;
    let mut CurrentSize: ULONG = 0;
    let mut ImageNameMaximumLength: USHORT = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    loop {
        SpiCurrent = Current as PSYSTEM_PROCESS_INFORMATION;

        KeEnterCriticalRegion();
        ExAcquirePushLockShared(core::ptr::addr_of_mut!((*Process).ProcessLock) as *mut _);

        let pcb_thread_list_head = core::ptr::addr_of_mut!((*Process).Pcb.ThreadListHead);
        let zombie = (*Process).__bindgen_anon_2.__bindgen_anon_1.ProcessExiting() != 0
            && (*Process).Pcb.Header.SignalState != 0
            && (*Process).ActiveThreads == 0
            && (*pcb_thread_list_head).Flink == pcb_thread_list_head;
        let mut ProcessImageName: PUNICODE_STRING = core::ptr::null_mut();

        if zombie {
            CurrentSize = 0;
            ImageNameMaximumLength = 0;
            ExReleasePushLockShared(core::ptr::addr_of_mut!((*Process).ProcessLock) as *mut _);
            KeLeaveCriticalRegion();
        } else {
            let mut ThreadsCount: ULONG = 0;
            let mut entry = (*pcb_thread_list_head).Flink;
            while entry != pcb_thread_list_head {
                ThreadsCount += 1;
                entry = (*entry).Flink;
            }

            CurrentSize = proc_size + thread_size * ThreadsCount;
            let mut ImageNameLength: USHORT = 0;
            let mut TempProcessImageName: PUNICODE_STRING = core::ptr::null_mut();
            let name_status = SeLocateProcessImageName(Process, &mut TempProcessImageName);
            ProcessImageName = TempProcessImageName;
            let mut szSrc: PWCHAR = core::ptr::null_mut();
            if NT_SUCCESS(name_status) && !ProcessImageName.is_null() && (*ProcessImageName).Length > 0 {
                szSrc = ((*ProcessImageName).Buffer as *mut u8)
                    .add((*ProcessImageName).Length as usize) as PWCHAR;
                while szSrc > (*ProcessImageName).Buffer {
                    szSrc = szSrc.sub(1);
                    if *szSrc == OBJ_NAME_PATH_SEPARATOR as u16 {
                        szSrc = szSrc.add(1);
                        break;
                    } else {
                        ImageNameLength += core::mem::size_of::<u16>() as USHORT;
                    }
                }
            }
            if ImageNameLength == 0 && Process != PsIdleProcess {
                let mut n: USHORT = 0;
                let name = (*Process).ImageFileName.as_ptr();
                while n < (*Process).ImageFileName.len() as USHORT && *name.add(n as usize) != 0 {
                    n += 1;
                }
                ImageNameLength = n * core::mem::size_of::<u16>() as USHORT;
            }

            ImageNameMaximumLength = if ImageNameLength > 0 {
                ((ImageNameLength + core::mem::size_of::<u16>() as USHORT + 7) & !7) as USHORT
            } else {
                0
            };

            TotalSize += CurrentSize + ImageNameMaximumLength as ULONG;
            if TotalSize > Size {
                Overflow = 1u8;
            }

            if Overflow == 0u8 {
                (*SpiCurrent).NextEntryOffset = CurrentSize + ImageNameMaximumLength as ULONG;
                (*SpiCurrent).NumberOfThreads = ThreadsCount;
                (*SpiCurrent).CreateTime = (*Process).CreateTime;
                (*SpiCurrent).ImageName.Length = ImageNameLength;
                (*SpiCurrent).ImageName.MaximumLength = ImageNameMaximumLength;
                (*SpiCurrent).ImageName.Buffer = Current.add(CurrentSize as usize) as *mut u16;

                if Process != PsIdleProcess {
                    if !szSrc.is_null() {
                        core::ptr::copy_nonoverlapping(
                            szSrc as *const u8,
                            (*SpiCurrent).ImageName.Buffer as *mut u8,
                            (*SpiCurrent).ImageName.Length as usize,
                        );
                    } else {
                        let mut ImageName: ANSI_STRING = core::mem::zeroed();
                        RtlInitAnsiString(&mut ImageName, (*Process).ImageFileName.as_ptr() as PCHAR);
                        let s = RtlAnsiStringToUnicodeString(&mut (*SpiCurrent).ImageName, &ImageName, 0u8);
                        if !NT_SUCCESS(s) {
                            (*SpiCurrent).ImageName.Length = 0;
                        }
                    }
                } else {
                    RtlInitUnicodeString(&mut (*SpiCurrent).ImageName, core::ptr::null_mut());
                }

                (*SpiCurrent).BasePriority = (*Process).Pcb.BasePriority as LONG;
                (*SpiCurrent).UniqueProcessId = (*Process).UniqueProcessId;
                (*SpiCurrent).InheritedFromUniqueProcessId =
                    (*Process).InheritedFromUniqueProcessId as HANDLE;
                (*SpiCurrent).HandleCount = if Process == PsIdleProcess {
                    0
                } else {
                    ObGetProcessHandleCount(Process)
                };
                (*SpiCurrent).PeakVirtualSize = (*Process).PeakVirtualSize;
                (*SpiCurrent).VirtualSize = (*Process).VirtualSize;
                (*SpiCurrent).PageFaultCount = (*Process).Vm.PageFaultCount;
                (*SpiCurrent).PeakWorkingSetSize = (*Process).Vm.PeakWorkingSetSize as _;
                (*SpiCurrent).WorkingSetSize = (*Process).Vm.WorkingSetSize as _;
                (*SpiCurrent).QuotaPeakPagedPoolUsage = (*Process).QuotaPeak[PsPagedPool];
                (*SpiCurrent).QuotaPagedPoolUsage = (*Process).QuotaUsage[PsPagedPool];
                (*SpiCurrent).QuotaPeakNonPagedPoolUsage = (*Process).QuotaPeak[PsNonPagedPool];
                (*SpiCurrent).QuotaNonPagedPoolUsage = (*Process).QuotaUsage[PsNonPagedPool];
                (*SpiCurrent).PagefileUsage = (*Process).QuotaUsage[PsPageFile];
                (*SpiCurrent).PeakPagefileUsage = (*Process).QuotaPeak[PsPageFile];
                (*SpiCurrent).PrivatePageCount = (*Process).CommitCharge;

                let mut ThreadInfo = SpiCurrent.add(1) as PSYSTEM_THREAD_INFORMATION;
                let mut entry = (*pcb_thread_list_head).Flink;
                while entry != pcb_thread_list_head {
                    let CurrentThread = (entry as *mut u8)
                        .sub(core::mem::offset_of!(_ETHREAD, Tcb)
                            + core::mem::offset_of!(ntkrnl_sys::ddk::_KTHREAD, ThreadListEntry))
                        as *mut _ETHREAD;

                    (*ThreadInfo).KernelTime.QuadPart =
                        UInt32x32To64(kthread_kernel_time(CurrentThread), KeMaximumIncrement) as i64;
                    (*ThreadInfo).UserTime.QuadPart =
                        UInt32x32To64(kthread_user_time(CurrentThread), KeMaximumIncrement) as i64;
                    (*ThreadInfo).CreateTime.QuadPart = (*CurrentThread).CreateTime.QuadPart;
                    (*ThreadInfo).WaitTime = kthread_wait_time(CurrentThread);
                    (*ThreadInfo).StartAddress = ethread_start_address(CurrentThread);
                    (*ThreadInfo).ClientId = (*CurrentThread).Cid;
                    (*ThreadInfo).Priority = (*CurrentThread).Tcb.Priority as LONG;
                    (*ThreadInfo).BasePriority = (*CurrentThread).Tcb.BasePriority as LONG;
                    (*ThreadInfo).ContextSwitches = kthread_context_switches(CurrentThread);
                    (*ThreadInfo).ThreadState = kthread_state(CurrentThread) as ULONG;
                    (*ThreadInfo).WaitReason = (*CurrentThread).Tcb.WaitReason as ULONG;

                    ThreadInfo = ThreadInfo.add(1);
                    entry = (*entry).Flink;
                }

                let mut TotalUser: ULONG = 0;
                let TotalKernel = KeQueryRuntimeProcess(&mut (*Process).Pcb, &mut TotalUser);
                (*SpiCurrent).UserTime.QuadPart =
                    UInt32x32To64(TotalUser, KeMaximumIncrement) as i64;
                (*SpiCurrent).KernelTime.QuadPart =
                    UInt32x32To64(TotalKernel, KeMaximumIncrement) as i64;
            }

            if !ProcessImageName.is_null() {
                ExFreePoolWithTag(ProcessImageName as PVOID, TAG_SEPA);
            }

            ExReleasePushLockShared(core::ptr::addr_of_mut!((*Process).ProcessLock) as *mut _);
            KeLeaveCriticalRegion();
        }

        // Skip:
        if Process == PsIdleProcess {
            Process = core::ptr::null_mut();
        }
        Process = PsGetNextProcess(Process);
        if Process == SystemProcess || Process.is_null() {
            if Overflow == 0u8 {
                (*SpiCurrent).NextEntryOffset = 0;
            }
            break;
        } else {
            Current = Current.add((CurrentSize + ImageNameMaximumLength as ULONG) as usize);
        }
    }

    if !Process.is_null() {
        ObDereferenceObject(Process as PVOID);
    }
    if Overflow != 0u8 {
        Status = STATUS_INFO_LENGTH_MISMATCH;
    }
    *ReqSize = TotalSize;
    Status
}


// Class 6
unsafe extern "C" fn QSI_SystemCallCountInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 7
unsafe extern "C" fn QSI_SystemDeviceInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let Sdi = Buffer as PSYSTEM_DEVICE_INFORMATION;
    *ReqSize = core::mem::size_of::<_SYSTEM_DEVICE_INFORMATION>() as ULONG;
    if Size < core::mem::size_of::<_SYSTEM_DEVICE_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let ConfigInfo = IoGetConfigurationInformation();
    (*Sdi).NumberOfDisks = (*ConfigInfo).DiskCount;
    (*Sdi).NumberOfFloppies = (*ConfigInfo).FloppyCount;
    (*Sdi).NumberOfCdRoms = (*ConfigInfo).CdRomCount;
    (*Sdi).NumberOfTapes = (*ConfigInfo).TapeCount;
    (*Sdi).NumberOfSerialPorts = (*ConfigInfo).SerialCount;
    (*Sdi).NumberOfParallelPorts = (*ConfigInfo).ParallelCount;
    STATUS_SUCCESS
}

// Class 8
unsafe extern "C" fn QSI_SystemProcessorPerformanceInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let mut Spi = Buffer as PSYSTEM_PROCESSOR_PERFORMANCE_INFORMATION;
    *ReqSize = (KeNumberProcessors as ULONG) * core::mem::size_of::<_SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as ULONG;
    if Size < *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        let total = idle_thread_kernel_time(Prcb) + idle_thread_user_time(Prcb);
        (*Spi).IdleTime.QuadPart = UInt32x32To64(total, KeMaximumIncrement) as i64;
        (*Spi).KernelTime.QuadPart = UInt32x32To64((*Prcb).KernelTime, KeMaximumIncrement) as i64;
        (*Spi).UserTime.QuadPart = UInt32x32To64((*Prcb).UserTime, KeMaximumIncrement) as i64;
        (*Spi).DpcTime.QuadPart = UInt32x32To64((*Prcb).DpcTime, KeMaximumIncrement) as i64;
        (*Spi).InterruptTime.QuadPart = UInt32x32To64((*Prcb).InterruptTime, KeMaximumIncrement) as i64;
        (*Spi).InterruptCount = (*Prcb).InterruptCount;
        Spi = Spi.add(1);
    }
    STATUS_SUCCESS
}

// Class 9 Q
unsafe extern "C" fn QSI_SystemFlagsInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if core::mem::size_of::<_SYSTEM_FLAGS_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    (*(Buffer as PSYSTEM_FLAGS_INFORMATION)).Flags = NtGlobalFlag;
    *ReqSize = core::mem::size_of::<_SYSTEM_FLAGS_INFORMATION>() as ULONG;
    STATUS_SUCCESS
}

// Class 9 S
unsafe extern "C" fn SSI_SystemFlagsInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if core::mem::size_of::<_SYSTEM_FLAGS_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if SeSinglePrivilegeCheck(SeDebugPrivilege, ExGetPreviousMode()) == 0u8 {
        return STATUS_ACCESS_DENIED;
    }
    NtGlobalFlag = (*(Buffer as PSYSTEM_FLAGS_INFORMATION)).Flags;
    STATUS_SUCCESS
}

// Class 10
unsafe extern "C" fn QSI_SystemCallTimeInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 11
unsafe extern "C" fn QSI_SystemModuleInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(&raw mut PsLoadedModuleResource, 1u8);
    let Status = ExpQueryModuleInformation(
        &raw mut PsLoadedModuleList,
        &raw mut MmLoadedUserImageList,
        Buffer as PRTL_PROCESS_MODULES,
        Size,
        ReqSize,
    );
    ExReleaseResourceLite(&raw mut PsLoadedModuleResource);
    KeLeaveCriticalRegion();
    Status
}

// Class 12
unsafe extern "C" fn QSI_SystemLocksInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 13
unsafe extern "C" fn QSI_SystemStackTraceInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 14
unsafe extern "C" fn QSI_SystemPagedPoolInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 15
unsafe extern "C" fn QSI_SystemNonPagedPoolInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 16 — full system handle table enumeration.
unsafe extern "C" fn QSI_SystemHandleInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let header_size = field_offset_system_handle_information_handles() as ULONG;
    *ReqSize = header_size;
    if Size < header_size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let mut HandleInformation: PSYSTEM_HANDLE_INFORMATION = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut Status = ExLockUserBuffer(
        Buffer,
        Size,
        ExGetPreviousMode(),
        1, // IoWriteAccess
        &mut HandleInformation as *mut _ as *mut PVOID,
        &mut Mdl,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*HandleInformation).NumberOfHandles = 0;
    let mut Index: ULONG = 0;

    KeEnterCriticalRegion();
    ExAcquirePushLockShared(core::ptr::addr_of_mut!(HandleTableListLock) as *mut _);

    let head = core::ptr::addr_of_mut!(HandleTableListHead);
    let mut next = (*head).Flink;
    while next != head {
        // CONTAINING_RECORD(next, HANDLE_TABLE, HandleTableList)
        let HandleTable = (next as *mut u8)
            .sub(core::mem::offset_of!(_HANDLE_TABLE, HandleTableList))
            as *mut _HANDLE_TABLE;

        let mut Handle: EXHANDLE = core::mem::zeroed();
        loop {
            let Entry = ExpLookupHandleTableEntry(HandleTable, Handle);
            if Entry.is_null() {
                break;
            }

            let Object = (*Entry).__bindgen_anon_1.Object;
            let NextFreeTableEntry = (*Entry).__bindgen_anon_2.NextFreeTableEntry;
            if !Object.is_null() && NextFreeTableEntry != -2 {
                (*HandleInformation).NumberOfHandles += 1;

                if ExpLockHandleTableEntry(HandleTable, Entry) != 0 {
                    *ReqSize += core::mem::size_of::<_SYSTEM_HANDLE_TABLE_ENTRY_INFO>() as ULONG;
                    if *ReqSize > Size {
                        Status = STATUS_INFO_LENGTH_MISMATCH;
                    } else {
                        let Hdr = ObpGetHandleObject(Entry);
                        let entry_ptr =
                            core::ptr::addr_of_mut!((*HandleInformation).Handles[0]).add(Index as usize);
                        (*entry_ptr).UniqueProcessId = (*HandleTable).UniqueProcessId as ULONG_PTR as USHORT;
                        (*entry_ptr).CreatorBackTraceIndex = 0;
                        // FIXME: ObjectHeader->Type->Index — Type field corrupted; mirrors C #if 0.
                        (*entry_ptr).ObjectTypeIndex = 0;
                        (*entry_ptr).HandleAttributes =
                            ((*Entry).__bindgen_anon_1.ObAttributes as ULONG_PTR & OBJ_HANDLE_ATTRIBUTES as ULONG_PTR) as UCHAR;
                        (*entry_ptr).HandleValue = Handle.GenericHandleOverlay as ULONG_PTR as USHORT;
                        (*entry_ptr).Object = core::ptr::addr_of_mut!((*Hdr).Body) as PVOID;
                        (*entry_ptr).GrantedAccess = (*Entry).__bindgen_anon_2.GrantedAccess;
                        Index += 1;
                    }

                    ExUnlockHandleTableEntry(HandleTable, Entry);
                }
            }

            Handle.Value = (Handle.Value as ULONG_PTR + core::mem::size_of::<HANDLE>()) as _;
        }

        next = (*next).Flink;
    }

    ExReleasePushLockShared(core::ptr::addr_of_mut!(HandleTableListLock) as *mut _);
    KeLeaveCriticalRegion();
    ExUnlockUserBuffer(Mdl);

    Status
}

// Class 17
unsafe extern "C" fn QSI_SystemObjectInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 18
unsafe extern "C" fn QSI_SystemPageFileInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<_SYSTEM_PAGEFILE_INFORMATION>() as ULONG {
        *ReqSize = core::mem::size_of::<_SYSTEM_PAGEFILE_INFORMATION>() as ULONG;
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let Spfi = Buffer as PSYSTEM_PAGEFILE_INFORMATION;
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut FileName, core::ptr::null_mut());
    (*Spfi).NextEntryOffset = 0;
    (*Spfi).TotalSize = (MiFreeSwapPages + MiUsedSwapPages) as ULONG;
    (*Spfi).TotalInUse = MiUsedSwapPages as ULONG;
    (*Spfi).PeakUsage = MiUsedSwapPages as ULONG;
    (*Spfi).PageFileName = FileName;
    STATUS_SUCCESS
}

// Class 19
unsafe extern "C" fn QSI_SystemVdmInstemulInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 20
unsafe extern "C" fn QSI_SystemVdmBopInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 21 Q
unsafe extern "C" fn QSI_SystemFileCacheInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let Sci = Buffer as PSYSTEM_FILECACHE_INFORMATION;
    *ReqSize = core::mem::size_of::<_SYSTEM_FILECACHE_INFORMATION>() as ULONG;
    if Size < *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    core::ptr::write_bytes(Sci as *mut u8, 0, core::mem::size_of::<_SYSTEM_FILECACHE_INFORMATION>());
    (*Sci).CurrentSize = 0;
    (*Sci).PeakSize = 0;
    (*Sci).CurrentSizeIncludingTransitionInPages = 0;
    (*Sci).PageFaultCount = 0;
    (*Sci).MinimumWorkingSet = 0;
    (*Sci).MaximumWorkingSet = 0;
    STATUS_SUCCESS
}

// Class 21 S
unsafe extern "C" fn SSI_SystemFileCacheInformation(_Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<_SYSTEM_FILECACHE_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    STATUS_NOT_IMPLEMENTED
}

// Class 22
unsafe extern "C" fn QSI_SystemPoolTagInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if (Size as usize) < SIZEOF_SYSTEM_POOLTAG_INFORMATION {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    ExGetPoolTagInfo(Buffer as *mut ntkrnl_sys::ddk::_SYSTEM_POOLTAG_INFORMATION, Size, ReqSize)
}

// Class 23
unsafe extern "C" fn QSI_SystemInterruptInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    // A caller that sizes its buffer from the first call's ReqSize got zero back and
    // could never succeed (upstream feb0f56da7).
    *ReqSize = (KeNumberProcessors as ULONG)
        * core::mem::size_of::<_SYSTEM_INTERRUPT_INFORMATION>() as ULONG;
    if Size < *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let ti = KeQueryTimeIncrement();
    let mut sii = Buffer as PSYSTEM_INTERRUPT_INFORMATION;
    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        (*sii).ContextSwitches = KeGetContextSwitches(Prcb);
        (*sii).DpcCount = (*Prcb).DpcData[0 /* DPC_NORMAL */].DpcCount as ULONG;
        (*sii).DpcRate = (*Prcb).DpcRequestRate;
        (*sii).TimeIncrement = ti;
        (*sii).DpcBypassCount = 0;
        (*sii).ApcBypassCount = 0;
        sii = sii.add(1);
    }
    STATUS_SUCCESS
}

// Class 24 Q
unsafe extern "C" fn QSI_SystemDpcBehaviorInformation(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<_SYSTEM_DPC_BEHAVIOR_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let sdbi = Buffer as PSYSTEM_DPC_BEHAVIOR_INFORMATION;
    (*sdbi).DpcQueueDepth = KiMaximumDpcQueueDepth as ULONG;
    (*sdbi).MinimumDpcRate = KiMinimumDpcRate;
    (*sdbi).AdjustDpcThreshold = KiAdjustDpcThreshold;
    (*sdbi).IdealDpcRate = KiIdealDpcRate;
    STATUS_SUCCESS
}

// Class 24 S
unsafe extern "C" fn SSI_SystemDpcBehaviorInformation(_Buffer: PVOID, _Size: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 25
unsafe extern "C" fn QSI_SystemFullMemoryInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    *ReqSize = core::mem::size_of::<ULONG>() as ULONG;
    if Size != core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    *(Buffer as *mut ULONG) = 0; // FIXME: was MiMemoryConsumers[MC_USER].PagesUsed
    STATUS_SUCCESS
}

// Class 26 S — Load GDI driver image
unsafe extern "C" fn SSI_SystemLoadGdiDriverInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<_SYSTEM_GDI_DRIVER_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if ExGetPreviousMode() != 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }
    let DriverInfo = Buffer as PSYSTEM_GDI_DRIVER_INFORMATION;
    let mut SectionPointer: PVOID = core::ptr::null_mut();
    let mut ImageBase: PVOID = core::ptr::null_mut();
    let Status = MmLoadSystemImage(
        &mut (*DriverInfo).DriverName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        &mut SectionPointer as *mut PVOID,
        &mut ImageBase as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }
    let mut DirSize: ULONG = 0;
    (*DriverInfo).ExportSectionPointer = RtlImageDirectoryEntryToData(
        ImageBase,
        1,
        IMAGE_DIRECTORY_ENTRY_EXPORT,
        &mut DirSize,
    ) as ntkrnl_sys::ddk::PIMAGE_EXPORT_DIRECTORY;
    let NtHeader = RtlImageNtHeader(ImageBase);
    let EntryPoint = (*NtHeader).OptionalHeader.AddressOfEntryPoint as ULONG_PTR
        + ImageBase as ULONG_PTR;
    (*DriverInfo).ImageAddress = ImageBase;
    (*DriverInfo).SectionPointer = SectionPointer;
    (*DriverInfo).EntryPoint = EntryPoint as PVOID;
    (*DriverInfo).ImageLength = (*NtHeader).OptionalHeader.SizeOfImage;
    STATUS_SUCCESS
}

// Class 27 S — Unload GDI driver image
unsafe extern "C" fn SSI_SystemUnloadGdiDriverInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<PVOID>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if ExGetPreviousMode() != 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }
    MmUnloadSystemImage(*(Buffer as *mut PVOID));
    STATUS_SUCCESS
}

// Class 28 Q
unsafe extern "C" fn QSI_SystemTimeAdjustmentInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    // ReqSize must be reported on SUCCESS too, and this class takes an exact size
    // (fixed struct, no trailing array) -- upstream c6410fb0c2.
    *ReqSize = core::mem::size_of::<_SYSTEM_QUERY_TIME_ADJUST_INFORMATION>() as ULONG;
    if Size != *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let TimeInfo = Buffer as PSYSTEM_QUERY_TIME_ADJUST_INFORMATION;
    (*TimeInfo).TimeIncrement = KeMaximumIncrement;
    (*TimeInfo).TimeAdjustment = KeTimeAdjustment;
    (*TimeInfo).Enable = (KiTimeAdjustmentEnabled == 0) as BOOLEAN;
    STATUS_SUCCESS
}

// Class 28 S
unsafe extern "C" fn SSI_SystemTimeAdjustmentInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    let PreviousMode = KeGetPreviousMode();
    if core::mem::size_of::<_SYSTEM_SET_TIME_ADJUST_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if PreviousMode != 0 {
        if SeSinglePrivilegeCheck(SeSystemtimePrivilege, PreviousMode) == 0u8 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }
    let TimeInfo = Buffer as PSYSTEM_SET_TIME_ADJUST_INFORMATION;
    if (*TimeInfo).Enable != 0 {
        KiTimeAdjustmentEnabled = 0;
        KeTimeAdjustment = KeMaximumIncrement;
    } else {
        if (*TimeInfo).TimeAdjustment == 0 {
            return STATUS_INVALID_PARAMETER_2;
        }
        KiTimeAdjustmentEnabled = 1;
        KeTimeAdjustment = (*TimeInfo).TimeAdjustment;
    }
    STATUS_SUCCESS
}

// Class 29
unsafe extern "C" fn QSI_SystemSummaryMemoryInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 30
unsafe extern "C" fn QSI_SystemMirrorMemoryInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 31
unsafe extern "C" fn QSI_SystemPerformanceTraceInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 32
unsafe extern "C" fn QSI_SystemObsolete0(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 33
unsafe extern "C" fn QSI_SystemExceptionInformation(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    if core::mem::size_of::<_SYSTEM_EXCEPTION_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let mut AlignmentFixupCount: ULONG = 0;
    let mut ExceptionDispatchCount: ULONG = 0;
    let mut FloatingEmulationCount: ULONG = 0;
    let ByteWordEmulationCount: ULONG = 0;
    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        if !Prcb.is_null() {
            AlignmentFixupCount += (*Prcb).KeAlignmentFixupCount;
            ExceptionDispatchCount += (*Prcb).KeExceptionDispatchCount;
            // sysinfo.c: `#ifndef _M_ARM` (32-bit ARM) — arm64 counts it too.
            FloatingEmulationCount += (*Prcb).KeFloatingEmulationCount;
        }
    }
    let ei = Buffer as PSYSTEM_EXCEPTION_INFORMATION;
    (*ei).AlignmentFixupCount = AlignmentFixupCount;
    (*ei).ExceptionDispatchCount = ExceptionDispatchCount;
    (*ei).FloatingEmulationCount = FloatingEmulationCount;
    (*ei).ByteWordEmulationCount = ByteWordEmulationCount;
    STATUS_SUCCESS
}

// Class 34
unsafe extern "C" fn QSI_SystemCrashDumpStateInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 35
unsafe extern "C" fn QSI_SystemKernelDebuggerInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    *ReqSize = core::mem::size_of::<_SYSTEM_KERNEL_DEBUGGER_INFORMATION>() as ULONG;
    if Size < core::mem::size_of::<_SYSTEM_KERNEL_DEBUGGER_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let skdi = Buffer as PSYSTEM_KERNEL_DEBUGGER_INFORMATION;
    extern "C" {
        static KdDebuggerEnabled: BOOLEAN;
        static KdDebuggerNotPresent: BOOLEAN;
    }
    (*skdi).KernelDebuggerEnabled = KdDebuggerEnabled;
    (*skdi).KernelDebuggerNotPresent = KdDebuggerNotPresent;
    STATUS_SUCCESS
}

// Class 36
unsafe extern "C" fn QSI_SystemContextSwitchInformation(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    if core::mem::size_of::<_SYSTEM_CONTEXT_SWITCH_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let mut ContextSwitches: ULONG = 0;
    for i in 0..KeNumberProcessors as usize {
        let Prcb = KiProcessorBlock[i];
        if !Prcb.is_null() {
            ContextSwitches += KeGetContextSwitches(Prcb);
        }
    }
    let csi = Buffer as PSYSTEM_CONTEXT_SWITCH_INFORMATION;
    (*csi).ContextSwitches = ContextSwitches;
    (*csi).FindAny = 0;
    (*csi).FindLast = 0;
    (*csi).FindIdeal = 0;
    (*csi).IdleAny = 0;
    (*csi).IdleCurrent = 0;
    (*csi).IdleLast = 0;
    (*csi).IdleIdeal = 0;
    (*csi).PreemptAny = 0;
    (*csi).PreemptCurrent = 0;
    (*csi).PreemptLast = 0;
    (*csi).SwitchToIdle = 0;
    STATUS_SUCCESS
}

// Class 37 Q
unsafe extern "C" fn QSI_SystemRegistryQuotaInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    *ReqSize = core::mem::size_of::<_SYSTEM_REGISTRY_QUOTA_INFORMATION>() as ULONG;
    if Size < *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let srqi = Buffer as PSYSTEM_REGISTRY_QUOTA_INFORMATION;
    (*srqi).RegistryQuotaAllowed = 0x2000000;
    (*srqi).RegistryQuotaUsed = 0x200000;
    (*srqi).PagedPoolSize = 0x200000;
    STATUS_SUCCESS
}

// Class 37 S
unsafe extern "C" fn SSI_SystemRegistryQuotaInformation(_Buffer: PVOID, _Size: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 38 S — Load and call image (win32k.sys)
unsafe extern "C" fn SSI_SystemExtendServiceTableInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<_UNICODE_STRING>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let PreviousMode = KeGetPreviousMode();
    if PreviousMode != 0 {
        // Hardcoded win32k.sys path: "\SystemRoot\System32\win32k.sys" = 31 wide chars
        static WIN32K_NAME_BUF: [u16; 31] = {
            let mut buf = [0u16; 31];
            let s: &[u8] = b"\\SystemRoot\\System32\\win32k.sys";
            let mut i = 0;
            while i < 31 { buf[i] = s[i] as u16; i += 1; }
            buf
        };
        let Win32kName = _UNICODE_STRING {
            Length: 62,    // 31 chars * 2 bytes
            MaximumLength: 62,
            Buffer: WIN32K_NAME_BUF.as_ptr() as *mut u16,
        };
        if SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode) == 0u8 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
        // C: probe + capture + probe the string buffer under _SEH2_TRY, whose
        // handler returns the exception code.
        let ImageName: _UNICODE_STRING =
            match ntkrnl_sys::probe::probe_read_generic(Buffer as *const _UNICODE_STRING, 1, 1) {
                Ok(name) => name,
                Err(status) => return status,
            };
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
            ImageName.Buffer as *const core::ffi::c_void,
            ImageName.Length as usize,
            core::mem::size_of::<u16>(),
            1,
        ) {
            return status;
        }
        if RtlEqualUnicodeString(
            &ImageName as *const _UNICODE_STRING,
            &Win32kName as *const _UNICODE_STRING,
            0,
        ) == 0 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
        return ZwSetSystemInformation(
            SystemExtendServiceTableInformation,
            &Win32kName as *const _UNICODE_STRING as PVOID,
            core::mem::size_of::<_UNICODE_STRING>() as ULONG,
        );
    }

    // Kernel-mode path
    let mut ModuleObject: PVOID = core::ptr::null_mut();
    let mut ImageBase: PVOID = core::ptr::null_mut();
    let Status = MmLoadSystemImage(
        Buffer as *mut _UNICODE_STRING,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        &mut ModuleObject as *mut PVOID,
        &mut ImageBase as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }
    let NtHeader = RtlImageNtHeader(ImageBase);
    if NtHeader.is_null() {
        MmUnloadSystemImage(ModuleObject);
        return 0xC000007Bu32 as NTSTATUS;
    }
    let EntryPoint = (*NtHeader).OptionalHeader.AddressOfEntryPoint as ULONG_PTR
        + ImageBase as ULONG_PTR;
    let DriverInit: PDRIVER_INITIALIZE = core::mem::transmute(EntryPoint);
    let mut Win32k: DRIVER_OBJECT = core::mem::zeroed();
    Win32k.DriverStart = ImageBase;
    let Status = DriverInit(&mut Win32k, core::ptr::null_mut());
    if Status < 0 {
        MmUnloadSystemImage(ModuleObject);
    }
    Status
}

// Class 39 S
unsafe extern "C" fn SSI_SystemPrioritySeperation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    if SeSinglePrivilegeCheck(SeTcbPrivilege, ExGetPreviousMode()) == 0u8 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }
    PsChangeQuantumTable(1u8, *(Buffer as *mut ULONG));
    STATUS_SUCCESS
}

// Class 40
unsafe extern "C" fn QSI_SystemVerifierAddDriverInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 41
unsafe extern "C" fn QSI_SystemVerifierRemoveDriverInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 42
unsafe extern "C" fn QSI_SystemProcessorIdleInformation(_Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let need = core::mem::size_of::<ntkrnl_sys::ddk::PROCESSOR_POWER_INFORMATION>() * KeNumberProcessors as usize;
    *ReqSize = need as ULONG;
    if (need as ULONG) > Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    STATUS_NOT_IMPLEMENTED
}

// Class 43
unsafe extern "C" fn QSI_SystemLegacyDriverInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 44 Q
unsafe extern "C" fn QSI_SystemCurrentTimeZoneInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    *ReqSize = core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>() as ULONG;
    if core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>() as ULONG != Size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    core::ptr::copy_nonoverlapping(
        &raw const ExpTimeZoneInfo as *const RTL_TIME_ZONE_INFORMATION as *const u8,
        Buffer as *mut u8,
        core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>(),
    );
    STATUS_SUCCESS
}

// Class 44 S
unsafe extern "C" fn SSI_SystemCurrentTimeZoneInformation(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<RTL_TIME_ZONE_INFORMATION>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    ExpSetTimeZoneInformation(Buffer as PRTL_TIME_ZONE_INFORMATION)
}

// Class 45
unsafe extern "C" fn QSI_SystemLookasideInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let MaxCount = Size / core::mem::size_of::<_SYSTEM_LOOKASIDE_INFORMATION>() as ULONG;
    let mut Remaining = MaxCount;
    if Remaining == 0 {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let PreviousMode = ExGetPreviousMode();
    let mut Info: PSYSTEM_LOOKASIDE_INFORMATION = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();
    let Status = ExLockUserBuffer(
        Buffer,
        Size,
        PreviousMode,
        1, // IoWriteAccess
        &mut Info as *mut _ as *mut PVOID,
        &mut Mdl,
    );
    if Status < 0 {
        return Status;
    }

    ExpCopyLookasideInformation(&mut Info, &mut Remaining, &raw mut ExPoolLookasideListHead, 0u8);
    if Remaining > 0 {
        ExpCopyLookasideInformation(&mut Info, &mut Remaining, &raw mut ExSystemLookasideListHead, 1u8);
    }
    if Remaining > 0 {
        // RAII: guard drops (releases) at block end — release was already the
        // block's last statement, so this is a behavior-identical 1:1 swap.
        let _lk = ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!(ExpNonPagedLookasideListLock) as ntkrnl_sys::ke::PKSPIN_LOCK,
        );
        ExpCopyLookasideInformation(&mut Info, &mut Remaining, &raw mut ExpNonPagedLookasideListHead, 1u8);
    }
    if Remaining > 0 {
        let _lk = ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!(ExpPagedLookasideListLock) as ntkrnl_sys::ke::PKSPIN_LOCK,
        );
        ExpCopyLookasideInformation(&mut Info, &mut Remaining, &raw mut ExpPagedLookasideListHead, 1u8);
    }

    ExUnlockUserBuffer(Mdl);
    *ReqSize = (MaxCount - Remaining) * core::mem::size_of::<_SYSTEM_LOOKASIDE_INFORMATION>() as ULONG;
    STATUS_SUCCESS
}

// Class 46 S
unsafe extern "C" fn SSI_SystemTimeSlipNotification(_Buffer: PVOID, _Size: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 47 S
unsafe extern "C" fn SSI_SystemSessionCreate(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let PreviousMode = KeGetPreviousMode();
    if PreviousMode != 0 {
        if SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode) == 0u8 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }
    let mut SessionId: ULONG = 0;
    let Status = MmSessionCreate(&mut SessionId);
    if Status >= 0 {
        *(Buffer as *mut ULONG) = SessionId;
    }
    Status
}

// Class 48 S
unsafe extern "C" fn SSI_SystemSessionDetach(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let PreviousMode = KeGetPreviousMode();
    if PreviousMode != 0 {
        if SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode) == 0u8 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }
    }
    let SessionId = *(Buffer as *const ULONG);
    MmSessionDelete(SessionId)
}

// Class 49
unsafe extern "C" fn QSI_SystemSessionInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 50
unsafe extern "C" fn QSI_SystemRangeStartInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<ULONG_PTR>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    *(Buffer as *mut ULONG_PTR) = ntkrnl_sys::mm::MmSystemRangeStart as ULONG_PTR;
    if !ReqSize.is_null() {
        *ReqSize = core::mem::size_of::<ULONG_PTR>() as ULONG;
    }
    STATUS_SUCCESS
}

// Class 51 Q
unsafe extern "C" fn QSI_SystemVerifierInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 51 S
unsafe extern "C" fn SSI_SystemVerifierInformation(_Buffer: PVOID, _Size: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 52 S
unsafe extern "C" fn SSI_SystemVerifierThunkExtend(_Buffer: PVOID, _Size: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 53
unsafe extern "C" fn QSI_SystemSessionProcessInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 54 S — alias for SystemLoadGdiDriverInformation
unsafe extern "C" fn SSI_SystemLoadGdiDriverInSystemSpace(Buffer: PVOID, Size: ULONG) -> NTSTATUS {
    SSI_SystemLoadGdiDriverInformation(Buffer, Size)
}

// Class 55
unsafe extern "C" fn QSI_SystemNumaProcessorMap(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let NumaInformation = Buffer as PSYSTEM_NUMA_INFORMATION;
    (*NumaInformation).HighestNodeNumber = (KeNumberNodes - 1) as ULONG;
    let header = field_offset_system_numa_information_active_mask() as ULONG;
    let mut MaxEntries = if Size > header {
        ((Size - header) as usize) / core::mem::size_of::<ULONGLONG>()
    } else {
        0
    };
    if MaxEntries > KeNumberNodes as usize {
        MaxEntries = KeNumberNodes as usize;
    }
    if Size >= header && MaxEntries != 0 {
        *ReqSize = header + (MaxEntries * core::mem::size_of::<ULONGLONG>()) as ULONG;
        // Data is a bindgen [ULONGLONG; 1] flexible array — indexing past [0] via the
        // array is UB (the optimizer knows the bound). Walk it by raw pointer instead.
        let data = core::ptr::addr_of_mut!((*NumaInformation).Data) as *mut ULONGLONG;
        for Node in 0..MaxEntries {
            *data.add(Node) = (*KeNodeBlock[Node]).ProcessorMask as ULONGLONG;
        }
    } else {
        *ReqSize = core::mem::size_of::<ULONG>() as ULONG;
    }
    STATUS_SUCCESS
}

// Class 56
unsafe extern "C" fn QSI_SystemPrefetcherInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 57
unsafe extern "C" fn QSI_SystemExtendedProcessInformation(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 58
unsafe extern "C" fn QSI_SystemRecommendedSharedDataAlignment(_Buffer: PVOID, _Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// Class 60
unsafe extern "C" fn QSI_SystemNumaAvailableMemory(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    if Size < core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let NumaInformation = Buffer as PSYSTEM_NUMA_INFORMATION;
    (*NumaInformation).HighestNodeNumber = (KeNumberNodes - 1) as ULONG;
    let header = field_offset_system_numa_information_active_mask() as ULONG;
    let mut MaxEntries = if Size > header {
        ((Size - header) as usize) / core::mem::size_of::<ULONGLONG>()
    } else {
        0
    };
    if MaxEntries > KeNumberNodes as usize {
        MaxEntries = KeNumberNodes as usize;
    }
    if Size >= header && MaxEntries != 0 {
        *ReqSize = header + (MaxEntries * core::mem::size_of::<ULONGLONG>()) as ULONG;
        if MaxEntries == 1 {
            (*NumaInformation).Data[0] = (ntkrnl_sys::mm::MmAvailablePages as ULONGLONG) << PAGE_SHIFT;
        } else {
            // Flexible-array walk — see the SystemNumaProcessorMap note above ([T; 1] UB).
            let data = core::ptr::addr_of_mut!((*NumaInformation).Data) as *mut ULONGLONG;
            for Node in 0..MaxEntries {
                let n = KeNodeBlock[Node];
                *data.add(Node) =
                    (((*n).FreeCount[0] + (*n).FreeCount[1]) as ULONGLONG) << PAGE_SHIFT;
            }
        }
    } else {
        *ReqSize = core::mem::size_of::<ULONG>() as ULONG;
    }
    STATUS_SUCCESS
}

// Class 64 — extended system handle table enumeration. Same walk as Class 16,
// fills SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX records (wider Object/HandleValue, adds Reserved).
unsafe extern "C" fn QSI_SystemExtendedHandleInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let header_size = field_offset_system_handle_information_ex_handles() as ULONG;
    *ReqSize = header_size;
    if Size < header_size {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let mut HandleInformation: PSYSTEM_HANDLE_INFORMATION_EX = core::ptr::null_mut();
    let mut Mdl: PMDL = core::ptr::null_mut();
    let mut Status = ExLockUserBuffer(
        Buffer,
        Size,
        ExGetPreviousMode(),
        1, // IoWriteAccess
        &mut HandleInformation as *mut _ as *mut PVOID,
        &mut Mdl,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*HandleInformation).NumberOfHandles = 0;
    let mut Index: ULONG = 0;

    KeEnterCriticalRegion();
    ExAcquirePushLockShared(core::ptr::addr_of_mut!(HandleTableListLock) as *mut _);

    let head = core::ptr::addr_of_mut!(HandleTableListHead);
    let mut next = (*head).Flink;
    while next != head {
        let HandleTable = (next as *mut u8)
            .sub(core::mem::offset_of!(_HANDLE_TABLE, HandleTableList))
            as *mut _HANDLE_TABLE;

        let mut Handle: EXHANDLE = core::mem::zeroed();
        loop {
            let Entry = ExpLookupHandleTableEntry(HandleTable, Handle);
            if Entry.is_null() {
                break;
            }

            let Object = (*Entry).__bindgen_anon_1.Object;
            let NextFreeTableEntry = (*Entry).__bindgen_anon_2.NextFreeTableEntry;
            if !Object.is_null() && NextFreeTableEntry != -2 {
                (*HandleInformation).NumberOfHandles += 1;

                if ExpLockHandleTableEntry(HandleTable, Entry) != 0 {
                    *ReqSize += core::mem::size_of::<SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX>() as ULONG;
                    if *ReqSize > Size {
                        Status = STATUS_INFO_LENGTH_MISMATCH;
                    } else {
                        let Hdr = ObpGetHandleObject(Entry);
                        let entry_ptr =
                            core::ptr::addr_of_mut!((*HandleInformation).Handles[0]).add(Index as usize);
                        (*entry_ptr).UniqueProcessId = (*HandleTable).UniqueProcessId as ULONG_PTR;
                        (*entry_ptr).CreatorBackTraceIndex = 0;
                        // FIXME: ObjectHeader->Type->Index — Type field corrupted; mirrors C #if 0.
                        (*entry_ptr).ObjectTypeIndex = 0;
                        (*entry_ptr).HandleAttributes =
                            ((*Entry).__bindgen_anon_1.ObAttributes as ULONG_PTR & OBJ_HANDLE_ATTRIBUTES as ULONG_PTR) as ULONG;
                        (*entry_ptr).HandleValue = Handle.GenericHandleOverlay as ULONG_PTR;
                        (*entry_ptr).Object = core::ptr::addr_of_mut!((*Hdr).Body) as PVOID;
                        (*entry_ptr).GrantedAccess = (*Entry).__bindgen_anon_2.GrantedAccess;
                        (*entry_ptr).Reserved = 0;
                        Index += 1;
                    }

                    ExUnlockHandleTableEntry(HandleTable, Entry);
                }
            }

            Handle.Value = (Handle.Value as ULONG_PTR + core::mem::size_of::<HANDLE>()) as _;
        }

        next = (*next).Flink;
    }

    ExReleasePushLockShared(core::ptr::addr_of_mut!(HandleTableListLock) as *mut _);
    KeLeaveCriticalRegion();
    ExUnlockUserBuffer(Mdl);

    Status
}

// Class 70
unsafe extern "C" fn QSI_SystemObjectSecurityMode(Buffer: PVOID, Size: ULONG, _ReqSize: *mut ULONG) -> NTSTATUS {
    if Size != core::mem::size_of::<ULONG>() as ULONG {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    *(Buffer as *mut ULONG) = ObpObjectSecurityMode;
    STATUS_SUCCESS
}

// Class 73 — full enumeration. One entry per main processor core (HT/MC master),
// plus one entry per NUMA node.
unsafe extern "C" fn QSI_SystemLogicalProcessorInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let entry_size = core::mem::size_of::<_SYSTEM_LOGICAL_PROCESSOR_INFORMATION>() as ULONG;
    let mut Status = STATUS_SUCCESS;
    let mut DataSize: ULONG = 0;
    let mut CurrentInfo = Buffer as PSYSTEM_LOGICAL_PROCESSOR_INFORMATION;

    // Walk active processors, emit core entries for each HT/MC master.
    let mut CurrentProc = KeActiveProcessors;
    let mut i: usize = 0;
    loop {
        let Prcb = KiProcessorBlock[i];
        if (CurrentProc & 1) != 0 && !Prcb.is_null() && Prcb == (*Prcb).MultiThreadSetMaster {
            // Single-core if PRCB.SetMember matches its own multithread set.
            let ProcessorFlags: u8 =
                if (*Prcb).SetMember as u64 == (*Prcb).MultiThreadProcessorSet as u64 {
                    0
                } else {
                    1
                };

            DataSize += entry_size;
            if DataSize > Size {
                Status = STATUS_INFO_LENGTH_MISMATCH;
            } else {
                core::ptr::write_bytes(CurrentInfo as *mut u8, 0, entry_size as usize);
                (*CurrentInfo).ProcessorMask =
                    (*Prcb).MultiThreadProcessorSet as ntkrnl_sys::ddk::ULONG_PTR;
                (*CurrentInfo).Relationship = RelationProcessorCore;
                (*CurrentInfo).__bindgen_anon_1.ProcessorCore.Flags = ProcessorFlags;
                CurrentInfo = CurrentInfo.add(1);
            }
        }

        CurrentProc >>= 1;
        i += 1;
        if CurrentProc == 0 {
            break;
        }
    }

    // Then one entry per NUMA node.
    for node in 0..KeNumberNodes as ULONG {
        DataSize += entry_size;
        if DataSize > Size {
            Status = STATUS_INFO_LENGTH_MISMATCH;
        } else {
            core::ptr::write_bytes(CurrentInfo as *mut u8, 0, entry_size as usize);
            (*CurrentInfo).ProcessorMask = KeActiveProcessors as ntkrnl_sys::ddk::ULONG_PTR;
            (*CurrentInfo).Relationship = RelationNumaNode;
            (*CurrentInfo).__bindgen_anon_1.NumaNode.NodeNumber = node;
            CurrentInfo = CurrentInfo.add(1);
        }
    }

    *ReqSize = DataSize;
    Status
}

// Class 76
unsafe extern "C" fn QSI_SystemFirmwareTableInformation(Buffer: PVOID, Size: ULONG, ReqSize: *mut ULONG) -> NTSTATUS {
    let SysFirmwareInfo = Buffer as PSYSTEM_FIRMWARE_TABLE_INFORMATION;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut DataSize: ULONG = 0;
    let mut TableCount: ULONG = 0;
    *ReqSize = field_offset_system_firmware_table_information_buffer() as ULONG;
    if Size < *ReqSize {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let DataBufSize = Size - *ReqSize;
    match (*SysFirmwareInfo).ProviderSignature {
        SIG_ACPI => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        SIG_FIRM => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        SIG_RSMB => {
            Status = ExpGetRawSMBiosTable(core::ptr::null_mut(), &mut DataSize, 0);
            if DataSize > 0 {
                TableCount = 1;
                if (*SysFirmwareInfo).Action == SystemFirmwareTable_Enumerate {
                    DataSize = TableCount * core::mem::size_of::<ULONG>() as ULONG;
                    if DataSize <= DataBufSize {
                        *((*SysFirmwareInfo).TableBuffer.as_mut_ptr() as *mut ULONG) = 0;
                    }
                } else if (*SysFirmwareInfo).Action == SystemFirmwareTable_Get && DataSize <= DataBufSize {
                    Status = ExpGetRawSMBiosTable(
                        (*SysFirmwareInfo).TableBuffer.as_mut_ptr() as PVOID,
                        &mut DataSize,
                        DataBufSize,
                    );
                }
                (*SysFirmwareInfo).TableBufferLength = DataSize;
                *ReqSize += DataSize;
            }
        }
        _ => {
            *ReqSize = 0;
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    if Status >= 0 {
        match (*SysFirmwareInfo).Action {
            x if x == SystemFirmwareTable_Enumerate || x == SystemFirmwareTable_Get => {
                if (*SysFirmwareInfo).TableBufferLength > DataBufSize {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
            }
            _ => {
                Status = STATUS_ILLEGAL_FUNCTION;
            }
        }
    } else {
        (*SysFirmwareInfo).TableBufferLength = 0;
    }
    Status
}

// =============================================================================
// CallQS dispatch table
// =============================================================================

const _SI_QS: QssiCalls = QssiCalls { Query: None, Set: None };

const fn si_qx(q: QsiFn) -> QssiCalls { QssiCalls { Query: Some(q), Set: None } }
const fn si_xs(s: SsiFn) -> QssiCalls { QssiCalls { Query: None, Set: Some(s) } }
const fn si_qs(q: QsiFn, s: SsiFn) -> QssiCalls { QssiCalls { Query: Some(q), Set: Some(s) } }
const fn si_xx() -> QssiCalls { QssiCalls { Query: None, Set: None } }

static CALL_QS: [QssiCalls; MAX_SYSTEM_INFO_CLASS as usize] = {
    let mut t: [QssiCalls; MAX_SYSTEM_INFO_CLASS as usize] = [si_xx(); MAX_SYSTEM_INFO_CLASS as usize];
    t[0] = si_qx(QSI_SystemBasicInformation);
    t[1] = si_qx(QSI_SystemProcessorInformation);
    t[2] = si_qx(QSI_SystemPerformanceInformation);
    t[3] = si_qx(QSI_SystemTimeOfDayInformation);
    t[4] = si_qx(QSI_SystemPathInformation);
    t[5] = si_qx(QSI_SystemProcessInformation);
    t[6] = si_qx(QSI_SystemCallCountInformation);
    t[7] = si_qx(QSI_SystemDeviceInformation);
    t[8] = si_qx(QSI_SystemProcessorPerformanceInformation);
    t[9] = si_qs(QSI_SystemFlagsInformation, SSI_SystemFlagsInformation);
    t[10] = si_qx(QSI_SystemCallTimeInformation);
    t[11] = si_qx(QSI_SystemModuleInformation);
    t[12] = si_qx(QSI_SystemLocksInformation);
    t[13] = si_qx(QSI_SystemStackTraceInformation);
    t[14] = si_qx(QSI_SystemPagedPoolInformation);
    t[15] = si_qx(QSI_SystemNonPagedPoolInformation);
    t[16] = si_qx(QSI_SystemHandleInformation);
    t[17] = si_qx(QSI_SystemObjectInformation);
    t[18] = si_qx(QSI_SystemPageFileInformation);
    t[19] = si_qx(QSI_SystemVdmInstemulInformation);
    t[20] = si_qx(QSI_SystemVdmBopInformation);
    t[21] = si_qs(QSI_SystemFileCacheInformation, SSI_SystemFileCacheInformation);
    t[22] = si_qx(QSI_SystemPoolTagInformation);
    t[23] = si_qx(QSI_SystemInterruptInformation);
    t[24] = si_qs(QSI_SystemDpcBehaviorInformation, SSI_SystemDpcBehaviorInformation);
    t[25] = si_qx(QSI_SystemFullMemoryInformation);
    t[26] = si_xs(SSI_SystemLoadGdiDriverInformation);
    t[27] = si_xs(SSI_SystemUnloadGdiDriverInformation);
    t[28] = si_qs(QSI_SystemTimeAdjustmentInformation, SSI_SystemTimeAdjustmentInformation);
    t[29] = si_qx(QSI_SystemSummaryMemoryInformation);
    t[30] = si_qx(QSI_SystemMirrorMemoryInformation);
    t[31] = si_qx(QSI_SystemPerformanceTraceInformation);
    t[32] = si_qx(QSI_SystemObsolete0);
    t[33] = si_qx(QSI_SystemExceptionInformation);
    t[34] = si_qx(QSI_SystemCrashDumpStateInformation);
    t[35] = si_qx(QSI_SystemKernelDebuggerInformation);
    t[36] = si_qx(QSI_SystemContextSwitchInformation);
    t[37] = si_qs(QSI_SystemRegistryQuotaInformation, SSI_SystemRegistryQuotaInformation);
    t[38] = si_xs(SSI_SystemExtendServiceTableInformation);
    t[39] = si_xs(SSI_SystemPrioritySeperation);
    t[40] = si_qx(QSI_SystemVerifierAddDriverInformation);
    t[41] = si_qx(QSI_SystemVerifierRemoveDriverInformation);
    t[42] = si_qx(QSI_SystemProcessorIdleInformation);
    t[43] = si_qx(QSI_SystemLegacyDriverInformation);
    t[44] = si_qs(QSI_SystemCurrentTimeZoneInformation, SSI_SystemCurrentTimeZoneInformation);
    t[45] = si_qx(QSI_SystemLookasideInformation);
    t[46] = si_xs(SSI_SystemTimeSlipNotification);
    t[47] = si_xs(SSI_SystemSessionCreate);
    t[48] = si_xs(SSI_SystemSessionDetach);
    t[49] = si_qx(QSI_SystemSessionInformation);
    t[50] = si_qx(QSI_SystemRangeStartInformation);
    t[51] = si_qs(QSI_SystemVerifierInformation, SSI_SystemVerifierInformation);
    t[52] = si_xs(SSI_SystemVerifierThunkExtend);
    t[53] = si_qx(QSI_SystemSessionProcessInformation);
    t[54] = si_xs(SSI_SystemLoadGdiDriverInSystemSpace);
    t[55] = si_qx(QSI_SystemNumaProcessorMap);
    t[56] = si_qx(QSI_SystemPrefetcherInformation);
    t[57] = si_qx(QSI_SystemExtendedProcessInformation);
    t[58] = si_qx(QSI_SystemRecommendedSharedDataAlignment);
    t[60] = si_qx(QSI_SystemNumaAvailableMemory);
    t[64] = si_qx(QSI_SystemExtendedHandleInformation);
    t[70] = si_qx(QSI_SystemObjectSecurityMode);
    t[73] = si_qx(QSI_SystemLogicalProcessorInformation);
    t[76] = si_qx(QSI_SystemFirmwareTableInformation);
    t
};

unsafe impl Sync for QssiCalls {}

// =============================================================================
// Public NtQuerySystemInformation / NtSetSystemInformation
// =============================================================================

const TAG_QSI_BOUNCE: ULONG = u32::from_le_bytes(*b"ExQs");

/// Runs a user-mode query against a pool copy of the caller's buffer, then copies the
/// result back with a fault-tolerant copy.
///
/// The C runs every handler in place under one `_SEH2_TRY`, so a user page that goes
/// bad mid-query becomes `STATUS_ACCESS_VIOLATION`. Without SEH the same fault in a
/// handler bugchecks, and the handlers write the buffer with plain stores, some of them
/// under a push lock. Bouncing covers every class at once instead of re-plumbing ~40
/// handlers. The result is the same: the buffer is copied in first, so bytes a handler
/// leaves alone come back unchanged, and the input header the firmware-table class reads
/// is there.
///
/// Returns `Err` only for a fault on the user buffer, which is the C handler's status.
unsafe fn query_through_bounce(
    query: QsiFn,
    class: ULONG,
    user_buffer: PVOID,
    length: ULONG,
    req_size: *mut ULONG,
) -> Result<NTSTATUS, NTSTATUS> {
    if length == 0 {
        // Nothing to bounce, and no handler writes past a zero size.
        return Ok(query(user_buffer, 0, req_size));
    }
    let kbuf = ExAllocatePoolWithTag(1 /* PagedPool */, length as usize, TAG_QSI_BOUNCE);
    if kbuf.is_null() {
        return Err(STATUS_INSUFFICIENT_RESOURCES);
    }
    let _guard = ntkrnl_sys::probe::PoolAllocGuard::new(kbuf, TAG_QSI_BOUNCE);
    ntkrnl_sys::probe::safe_copy_raw(kbuf, user_buffer, length as usize)?;
    let status = query(kbuf, length, req_size);
    if class == SystemProcessInformation {
        rebase_process_image_names(kbuf, user_buffer, length);
    }
    ntkrnl_sys::probe::safe_copy_raw(user_buffer, kbuf, length as usize)?;
    Ok(status)
}

/// SystemProcessInformation is the one class that stores a pointer INTO the buffer
/// (`ImageName.Buffer` points at the name it appends after each entry), so a bounced
/// query has to move those pointers from the pool copy to the caller's buffer.
unsafe fn rebase_process_image_names(kbuf: PVOID, user_buffer: PVOID, length: ULONG) {
    let base = kbuf as usize;
    let end = base + length as usize;
    let delta = (user_buffer as usize).wrapping_sub(base);
    let entry_size = core::mem::size_of::<_SYSTEM_PROCESS_INFORMATION>();
    let mut offset = 0usize;
    while offset + entry_size <= length as usize {
        let entry = (base + offset) as PSYSTEM_PROCESS_INFORMATION;
        let name = (*entry).ImageName.Buffer as usize;
        if name >= base && name < end {
            (*entry).ImageName.Buffer = name.wrapping_add(delta) as *mut u16;
        }
        let next = (*entry).NextEntryOffset as usize;
        if next == 0 {
            break;
        }
        offset += next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySystemInformation(
    SystemInformationClass: ULONG,
    SystemInformation: PVOID,
    SystemInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    let SystemInformation = opaque_ptr(SystemInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;
    let mut CapturedResultLength: ULONG = 0;
    let mut Alignment = core::mem::align_of::<ULONG>() as ULONG;
    let PreviousMode = ExGetPreviousMode();

    if SystemInformationClass >= MAX_SYSTEM_INFO_CLASS {
        return STATUS_INVALID_INFO_CLASS;
    }

    if PreviousMode != 0 {
        if SystemInformationClass == SystemKernelDebuggerInformation {
            Alignment = core::mem::align_of::<BOOLEAN>() as ULONG;
        }
        // C: the whole body is one _SEH2_TRY whose handler sets Status to the
        // exception code; nothing after a fault runs, so returning it is the same.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
            SystemInformation,
            SystemInformationLength as usize,
            Alignment as usize,
            1,
        ) {
            return status;
        }
        if !ReturnLength.is_null() {
            // ProbeForWriteUlong: range-only, like the other typed probe.h macros.
            if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                ReturnLength as PVOID,
                core::mem::size_of::<ULONG>(),
                1,
                1,
            ) {
                return status;
            }
        }
    }

    if !ReturnLength.is_null() {
        if let Err(status) = ntkrnl_sys::probe::write_user(ReturnLength, 0, 0) {
            return status;
        }
    }

    if let Some(query) = CALL_QS[SystemInformationClass as usize].Query {
        Status = if PreviousMode != 0 {
            match query_through_bounce(
                query,
                SystemInformationClass,
                SystemInformation,
                SystemInformationLength,
                &mut CapturedResultLength,
            ) {
                Ok(status) => status,
                Err(status) => return status,
            }
        } else {
            query(SystemInformation, SystemInformationLength, &mut CapturedResultLength)
        };
        if !ReturnLength.is_null() {
            if let Err(status) = ntkrnl_sys::probe::write_user(ReturnLength, CapturedResultLength, 0) {
                return status;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetSystemInformation(
    SystemInformationClass: ULONG,
    SystemInformation: PVOID,
    SystemInformationLength: ULONG,
) -> NTSTATUS {
    let SystemInformation = opaque_ptr(SystemInformation);
    let mut Status: NTSTATUS = STATUS_INVALID_INFO_CLASS;
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0 {
        // C: under _SEH2_TRY, whose handler returns the exception code as Status.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
            SystemInformation,
            SystemInformationLength as usize,
            core::mem::size_of::<ULONG>(),
            1,
        ) {
            return status;
        }
    }

    if SystemInformationClass < MAX_SYSTEM_INFO_CLASS {
        if let Some(set) = CALL_QS[SystemInformationClass as usize].Set {
            Status = set(SystemInformation, SystemInformationLength);
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtGetCurrentProcessorNumber() -> ULONG {
    KeGetCurrentProcessorNumber()
}

#[no_mangle]
pub unsafe extern "system" fn ExGetPreviousMode() -> KPROCESSOR_MODE {
    KeGetPreviousMode()
}
