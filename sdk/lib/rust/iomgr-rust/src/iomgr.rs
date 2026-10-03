use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
#[cfg(not(target_arch = "aarch64"))]
#[inline(never)]
unsafe fn io_trace(s: &[u8]) {
    crate::dbg_print!(b"%s\0", s.as_ptr() as *const i8);
}

// aarch64 used to discard these entirely. All 40 call sites are one-shot BOOT
// milestones (IopInitializeBootDrivers, IopCreateArcNames, IopMarkBootPartition,
// …) plus their FAILED reports — none is per-IRP — so there is no hot path to
// protect, and dropping them cost real diagnostic ability: `PsLocateSystemDll
// FAILED` is reported through this function, and on arm64 IoInitSystem then
// CONTINUES BOOTING ("continuing on ARM64"). A failure to map the system DLL was
// therefore completely silent on the one arch that tolerates it, which is a
// candidate mechanism for the sweep-launches-nothing face.
//
// Route to ARM64_UART_TRACE — the mechanism proven to reach the serial (a soak
// log carries 7449 [PS] lines emitted this way). The nearby hand-rolled
// `for &ch in msg.iter() { *uart = ch }` loops are NOT equivalent: LLVM unrolls
// them into immediate stores, so their text does not even appear in the binary.
#[cfg(target_arch = "aarch64")]
#[inline(never)]
unsafe fn io_trace(s: &[u8]) {
    extern "C" {
        fn ARM64_UART_TRACE(msg: *const i8);
    }
    // Call sites all pass NUL-terminated byte strings; guard anyway so a future
    // caller that forgets cannot walk off the end.
    if s.last() == Some(&0) {
        ARM64_UART_TRACE(s.as_ptr() as *const i8);
    }
}

use crate::file::IopGetSetSecurityObject;
use crate::file::IopCloseFile;

use ntkrnl_sys::ddk::{
    ANSI_STRING,
    CONTROLLER_OBJECT,
    DEVICE_OBJECT,
    DRIVER_OBJECT,
    ERESOURCE,
    FILE_OBJECT,
    _GENERIC_MAPPING,
    IOP_MINI_COMPLETION_PACKET,
    IO_STACK_LOCATION,
    _IO_STATUS_BLOCK,
    _UNICODE_STRING,
    _KPRCB,
    IRP,
    KQUEUE,
    KTIMER,
    LARGE_INTEGER,
    LIST_ENTRY,
    MDL,
    _OBJECT_ATTRIBUTES,
    OBJECT_TYPE_INITIALIZER,
    PDEVICE_OBJECT,
    PLOADER_PARAMETER_BLOCK,
    _LOADER_PARAMETER_BLOCK,
    POBJECT_TYPE,
    PFILE_OBJECT,
    HANDLE,
    ExInitializeResourceLite,
    ExInitializeSystemLookasideList,
    IoAssignDriveLetters,
    IopCreateArcNames,
    IopDeleteDevice,
    IopDeleteDriver,
    IopDeleteFile,
    IopDeleteIoCompletion,
    IopInitializeBootDrivers,
    IopInitializeReserveIrp,
    IopInitializeSystemDrivers,
    IopParseDevice,
    IopParseFile,
    IopQueryName,
    IopReassignSystemRoot,
    IopReinitializeBootDrivers,
    IopReinitializeDrivers,
    IopStartRamdisk,
    KeInitializeDpc,
    KeInitializeSpinLock,
    KeInitializeTimerEx,
    KeSetTimerEx,
    NtClose,
    NtCreateDirectoryObject,
    ObCreateObjectType,
    ObReferenceObjectByHandle,
    RtlInitUnicodeString,
    ZwOpenFile,
    ZwClose,
    KeBugCheckEx,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList,
    _DEVICE_ACTION_PiActionEnumRootDevices,
    _GENERAL_LOOKASIDE,
    PLIST_ENTRY,
};

// IO tag constants
const IOC_TAG1: ULONG = 0x20637049;
const IO_LARGEIRP: ULONG = 0x6C707249;
const IO_SMALLIRP: ULONG = 0x73707249;
const IO_LARGEIRP_CPU: ULONG = 0x4C707249;
const IO_SMALLIRP_CPU: ULONG = 0x53707249;
const TAG_IO: ULONG = 0x20206F49;
const IO_COMPLETION_ALL_ACCESS: ULONG = 0x001F0003;
const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
const INACCESSIBLE_BOOT_DEVICE: ULONG = 0x0000007B;
const DO_SYSTEM_BOOT_PARTITION: ULONG = 0x00000100;
const FILE_READ_ACCESS: ULONG = 0x00000001;
const FILE_SHARE_READ: ULONG = 0x00000001;

unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).ObjectName = n;
    (*p).Attributes = a;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

extern "system" {
    fn WmiInitialize() -> BOOLEAN;
    fn ApphelpCacheInitialize() -> NTSTATUS;
    fn PsLocateSystemDll() -> NTSTATUS;
}

extern "C" {
    fn PiInitializeNotifications();
    #[link_name = "HalInitPnpDriver_shim"]
    fn HalInitPnpDriver();
    fn PiQueueDeviceAction(DeviceObject: PDEVICE_OBJECT, Action: ULONG, Callback: PVOID, Context: PVOID);
    fn RtlInitEmptyAnsiString(AnsiString: *mut ANSI_STRING, Buffer: *mut i8, BufferSize: USHORT);
    static mut IoDriverObjectType: POBJECT_TYPE;
    static mut IoCompletionType: POBJECT_TYPE;
    static mut IopCompletionMapping: _GENERIC_MAPPING;
    static mut IopReserveIrpAllocator: PVOID;
}

// `IopInitializePlugPlayServices` is `extern "system"` in `pnpmgr/pnpinit.rs`
// (NTAPI = stdcall on i386, cdecl on amd64/arm64). Call directly — the
// historical C trampoline at `io_rust_shims.c::IopInitializePlugPlayServices_shim`
// existed because of an LLVM `ud2` after the call when routed through
// `core::hint::black_box`; the workspace `lto = false` + `codegen-units = 256`
// firewall (plus the call landing in a different CGU than the callee within
// the same crate, given the per-fn CGU split) should now be sufficient. If
// the ud2 reappears on any arch, route the call through a sibling rlib (the
// same pattern as `*-inlines` / `*-macros`) instead of resurrecting the C shim.
extern "system" {
    fn IopInitializePlugPlayServices() -> NTSTATUS;
}

// `extern "system"` maps to stdcall on i386 (with `@N` decoration) and
// cdecl on amd64/arm64.
extern "system" {
    fn RtlAnsiStringToUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *mut ANSI_STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
    fn RtlUnicodeStringToAnsiString(DestinationString: *mut ANSI_STRING, SourceString: *mut _UNICODE_STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
    fn RtlInitAnsiString(DestinationString: *mut ANSI_STRING, SourceString: *const i8);
    fn RtlFreeUnicodeString(UnicodeString: *mut _UNICODE_STRING);
}

extern "C" {
    static mut ExSystemLookasideListHead: LIST_ENTRY;
    static mut KeNumberProcessors: i8;
    static KiProcessorBlock: [*mut _KPRCB; 0];
    static mut IopRootDeviceNode: *mut PVOID;
    static mut NtSystemRoot: _UNICODE_STRING;
    static mut InitIsWinPEMode: BOOLEAN;
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
}

#[cfg(target_arch = "aarch64")]
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
    fn Arm64UartPutChar(c: u8);
}

#[no_mangle]
pub static mut IopTraceLevel: ULONG = 0;

#[no_mangle]
pub static mut PnpSystemInit: BOOLEAN = 0u8;

unsafe extern "system" fn IopTimerDispatch(
    _Dpc: *mut ntkrnl_sys::ddk::_KDPC,
    _DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
}

#[no_mangle]
pub static mut IoDeviceObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
#[used]
pub static mut IoFileObjectType: POBJECT_TYPE = core::ptr::null_mut();

extern "C" {
    static mut IoControllerObjectType: POBJECT_TYPE;
}

#[no_mangle]
pub static mut IoCountOperations: BOOLEAN = 1u8;

#[no_mangle]
pub static mut IoReadOperationCount: ULONG = 0;

#[no_mangle]
pub static mut IoReadTransferCount: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IoWriteOperationCount: ULONG = 0;

#[no_mangle]
pub static mut IoWriteTransferCount: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IoOtherOperationCount: ULONG = 0;

#[no_mangle]
pub static mut IoOtherTransferCount: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IoStatisticsLock: KSPIN_LOCK = 0;

// IopAutoReboot + its IOMGR_CONTROL_VECTOR registry slice were relocated to
// ntoskrnl-rust/src/config/cmdata.rs (crate-split Phase 1b): the kernel's
// CmControlVector is a *const*-concatenated table, and const-eval cannot reference
// an extern (severed-crate) static's address — so IopAutoReboot must live in the
// same crate as that table. iomgr had no runtime use of it beyond the table; its
// reader is ke/bug.rs (via extern decl).

#[no_mangle]
pub static mut IopNumTriageDumpDataBlocks: ULONG = 0;

#[no_mangle]
pub static mut IopTriageDumpDataBlocks: [PVOID; 64] = [core::ptr::null_mut(); 64];

#[no_mangle]
pub static mut IopFileMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE,
    GenericWrite: STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES | FILE_EXECUTE | SYNCHRONIZE,
    GenericAll: FILE_ALL_ACCESS,
};

extern "C" {
    static mut ShutdownListHead: LIST_ENTRY;
    static mut LastChanceShutdownListHead: LIST_ENTRY;
    static mut ShutdownListLock: KSPIN_LOCK;
    static mut IoAdapterObjectType: POBJECT_TYPE;
    static mut IopDatabaseResource: ERESOURCE;
}

#[no_mangle]
pub static mut IopSecurityResource: ERESOURCE = unsafe { core::mem::zeroed() };

extern "C" {
    static mut IopDriverLoadResource: ERESOURCE;
    static mut IopDiskFileSystemQueueHead: LIST_ENTRY;
    static mut IopCdRomFileSystemQueueHead: LIST_ENTRY;
    static mut IopTapeFileSystemQueueHead: LIST_ENTRY;
    static mut IopNetworkFileSystemQueueHead: LIST_ENTRY;
    static mut DriverBootReinitListHead: LIST_ENTRY;
    static mut DriverReinitListHead: LIST_ENTRY;
    static mut IopFsNotifyChangeQueueHead: LIST_ENTRY;
    static mut IopErrorLogListHead: LIST_ENTRY;
    static mut IopTimerQueueHead: LIST_ENTRY;
    static mut IopTimerDpc: KDPC;
    static mut IopTimer: KTIMER;
    static mut DriverReinitListLock: KSPIN_LOCK;
    static mut DriverBootReinitListLock: KSPIN_LOCK;
    static mut IopLogListLock: KSPIN_LOCK;
    static mut IopTimerLock: KSPIN_LOCK;
    static mut IopErrorLogObject: PDEVICE_OBJECT;
    static mut PnPBootDriversInitialized: BOOLEAN;
}

static mut IoLargeIrpLookaside: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
static mut IoSmallIrpLookaside: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
static mut IopMdlLookasideList: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };

extern "C" {
    static mut IoCompletionPacketLookaside: GENERAL_LOOKASIDE;
}

#[no_mangle]
pub static mut IopLoaderBlock: PLOADER_PARAMETER_BLOCK = core::ptr::null_mut();

// CODE_SEG("INIT")
unsafe fn IopInitLookasideLists() {
    let LargeIrpSize: ULONG;
    let SmallIrpSize: ULONG;
    let MdlSize: ULONG;
    let mut i: LONG;
    let mut Prcb: *mut _KPRCB;
    let mut CurrentList: PGENERAL_LOOKASIDE = core::ptr::null_mut();

    LargeIrpSize = (core::mem::size_of::<IRP>() + (8 * core::mem::size_of::<IO_STACK_LOCATION>())) as ULONG;
    SmallIrpSize = (core::mem::size_of::<IRP>() + core::mem::size_of::<IO_STACK_LOCATION>()) as ULONG;
    MdlSize = (core::mem::size_of::<MDL>() + (23 * core::mem::size_of::<PFN_NUMBER>())) as ULONG;

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _GENERAL_LOOKASIDE,
        NonPagedPool,
        core::mem::size_of::<IOP_MINI_COMPLETION_PACKET>() as ULONG,
        IOC_TAG1,
        32,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _GENERAL_LOOKASIDE,
        NonPagedPool,
        LargeIrpSize,
        IO_LARGEIRP,
        64,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _GENERAL_LOOKASIDE,
        NonPagedPool,
        SmallIrpSize,
        IO_SMALLIRP,
        32,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _GENERAL_LOOKASIDE,
        NonPagedPool,
        MdlSize,
        TAG_MDL,
        128,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    CurrentList = ExAllocatePoolWithTag(
        NonPagedPool,
        (4 * KeNumberProcessors as usize * core::mem::size_of::<GENERAL_LOOKASIDE>()) as SIZE_T,
        TAG_IO,
    ) as PGENERAL_LOOKASIDE;

    i = 0;
    while i < KeNumberProcessors as LONG {
        Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).add(i as usize);

        (*Prcb).LookasideIrpFloat = 512 / KeNumberProcessors as LONG;

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].L = core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _GENERAL_LOOKASIDE;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _GENERAL_LOOKASIDE,
                NonPagedPool,
                core::mem::size_of::<IOP_MINI_COMPLETION_PACKET>() as ULONG,
                IO_SMALLIRP_CPU,
                32,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].P = CurrentList as *mut _GENERAL_LOOKASIDE;
            CurrentList = CurrentList.add(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].P = core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _GENERAL_LOOKASIDE;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].L = core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _GENERAL_LOOKASIDE;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _GENERAL_LOOKASIDE,
                NonPagedPool,
                LargeIrpSize,
                IO_LARGEIRP_CPU,
                64,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].P = CurrentList as *mut _GENERAL_LOOKASIDE;
            CurrentList = CurrentList.add(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].P = core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _GENERAL_LOOKASIDE;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].L = core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _GENERAL_LOOKASIDE;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _GENERAL_LOOKASIDE,
                NonPagedPool,
                SmallIrpSize,
                IO_SMALLIRP_CPU,
                32,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].P = CurrentList as *mut _GENERAL_LOOKASIDE;
            CurrentList = CurrentList.add(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].P = core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _GENERAL_LOOKASIDE;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].L = core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _GENERAL_LOOKASIDE;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _GENERAL_LOOKASIDE,
                NonPagedPool,
                SmallIrpSize,
                TAG_MDL,
                128,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].P = CurrentList as *mut _GENERAL_LOOKASIDE;
            CurrentList = CurrentList.add(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].P = core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _GENERAL_LOOKASIDE;
        }

        i += 1;
    }
}

// CODE_SEG("INIT")
unsafe fn IopCreateObjectTypes() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as *mut core::ffi::c_void,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as USHORT;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = 1u8;
    ObjectTypeInitializer.GenericMapping = IopFileMapping;

    static ADAPTER_NAME: [u16; 8] = [b'A' as u16, b'd' as u16, b'a' as u16, b'p' as u16, b't' as u16, b'e' as u16, b'r' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), ADAPTER_NAME.as_ptr());
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoAdapterObjectType),
    ) < 0
    {
        return 0u8;
    }

    static CONTROLLER_NAME: [u16; 11] = [b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'l' as u16, b'e' as u16, b'r' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), CONTROLLER_NAME.as_ptr());
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<CONTROLLER_OBJECT>() as ULONG;
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoControllerObjectType),
    ) < 0
    {
        return 0u8;
    }

    static DEVICE_NAME: [u16; 7] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), DEVICE_NAME.as_ptr());
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<DEVICE_OBJECT>() as ULONG;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteDevice);
    ObjectTypeInitializer.ParseProcedure = Some(IopParseDevice);
    ObjectTypeInitializer.SecurityProcedure = Some(IopGetSetSecurityObject);
    ObjectTypeInitializer.CaseInsensitive = 1u8;
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoDeviceObjectType),
    ) < 0
    {
        return 0u8;
    }

    static DRIVER_NAME: [u16; 7] = [b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), DRIVER_NAME.as_ptr());
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<DRIVER_OBJECT>() as ULONG;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteDriver);
    ObjectTypeInitializer.ParseProcedure = None;
    ObjectTypeInitializer.SecurityProcedure = None;
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoDriverObjectType),
    ) < 0
    {
        return 0u8;
    }

    static IOCOMPLETION_NAME: [u16; 13] = [b'I' as u16, b'o' as u16, b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'l' as u16, b'e' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), IOCOMPLETION_NAME.as_ptr());
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KQUEUE>() as ULONG;
    ObjectTypeInitializer.ValidAccessMask = IO_COMPLETION_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes |= OBJ_PERMANENT;
    ObjectTypeInitializer.GenericMapping = IopCompletionMapping;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteIoCompletion);
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoCompletionType),
    ) < 0
    {
        return 0u8;
    }

    static FILE_NAME: [u16; 5] = [b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), FILE_NAME.as_ptr());
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<FILE_OBJECT>() as ULONG;
    ObjectTypeInitializer.InvalidAttributes |= OBJ_EXCLUSIVE;
    ObjectTypeInitializer.MaintainHandleCount = 1u8;
    ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
    ObjectTypeInitializer.GenericMapping = IopFileMapping;
    ObjectTypeInitializer.CloseProcedure = Some(IopCloseFile);
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteFile);
    ObjectTypeInitializer.SecurityProcedure = Some(IopGetSetSecurityObject);
    ObjectTypeInitializer.QueryNameProcedure = Some(IopQueryName);
    ObjectTypeInitializer.ParseProcedure = Some(IopParseFile);
    ObjectTypeInitializer.UseDefaultObject = 0u8;
    if ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoFileObjectType),
    ) < 0
    {
        return 0u8;
    }

    1u8
}

// CODE_SEG("INIT")
unsafe fn IopCreateRootDirectories() -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirName: _UNICODE_STRING = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    static DRIVER_DIR: [u16; 8] = [b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DirName), DRIVER_DIR.as_ptr());
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(DirName),
        OBJ_PERMANENT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtCreateDirectoryObject(
        core::ptr::addr_of_mut!(Handle),
        DIRECTORY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    static FILESYSTEM_DIR: [u16; 12] = [b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DirName), FILESYSTEM_DIR.as_ptr());
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(DirName),
        OBJ_PERMANENT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtCreateDirectoryObject(
        core::ptr::addr_of_mut!(Handle),
        DIRECTORY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    static FILTERS_DIR: [u16; 21] = [b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b't' as u16, b'e' as u16, b'r' as u16, b's' as u16, 0, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DirName), FILTERS_DIR.as_ptr());
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(DirName),
        OBJ_PERMANENT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtCreateDirectoryObject(
        core::ptr::addr_of_mut!(Handle),
        DIRECTORY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    1u8
}

// CODE_SEG("INIT")
unsafe fn IopMarkBootPartition(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceString: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: [u8; 256] = [0; 256];
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

    sprintf(
        Buffer.as_mut_ptr() as *mut i8,
        b"\\ArcName\\%s\0".as_ptr() as *const i8,
        (*LoaderBlock).ArcBootDeviceName,
    );
    RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceString), Buffer.as_ptr() as *const i8);
    Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceName), core::ptr::addr_of_mut!(DeviceString), 1u8);
    if Status < 0 {
        return 0u8;
    }

    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(DeviceName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        0,
        FILE_NON_DIRECTORY_FILE,
    );
    if Status < 0 {
        #[cfg(target_arch = "aarch64")]
        {
            // x86 bugchecks INACCESSIBLE_BOOT_DEVICE unless in WinPE mode; arm64
            // just returns failure. Name it — a boot device that could not be
            // opened cascades into failures nowhere near here.
            io_trace(b"[IO] IopMarkBootPartition: ZwOpenFile FAILED\r\n\0");
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));
            return 0u8;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode != 0 {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));
                return 0u8;
            }
            KeBugCheckEx(
                INACCESSIBLE_BOOT_DEVICE,
                core::ptr::addr_of_mut!(DeviceName) as _,
                Status as _,
                0,
                0,
            );
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType,
        0i8,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));
        return 0u8;
    }

    (*(*FileObject).DeviceObject).Flags |= DO_SYSTEM_BOOT_PARTITION;

    ObReferenceObject((*FileObject).DeviceObject as PVOID);
    IopErrorLogObject = (*FileObject).DeviceObject;

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceName));
    NtClose(FileHandle);
    ObDereferenceObject(FileObject as PVOID);
    1u8
}

#[inline(never)]
unsafe fn IopFinishIoInitSystem(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut Buffer: [u8; 256] = [0; 256];
    let mut NtBootPath: ANSI_STRING = core::mem::zeroed();
    let mut RootString: ANSI_STRING = core::mem::zeroed();

    RtlInitEmptyAnsiString(
        core::ptr::addr_of_mut!(NtBootPath),
        Buffer.as_mut_ptr() as *mut i8,
        core::mem::size_of_val(&Buffer) as USHORT,
    );

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopInitializePlugPlayServices\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopInitializePlugPlayServices\r\n\0");
    Status = IopInitializePlugPlayServices();
    if Status < 0 {
        io_trace(b"[IO] IopInitializePlugPlayServices FAILED\r\n\0");
        return 0u8;
    }
    io_trace(b"[IO] ApphelpCacheInitialize\r\n\0");
    ApphelpCacheInitialize();
    io_trace(b"[IO] WmiInitialize\r\n\0");
    WmiInitialize();

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] HalInitPnpDriver\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] HalInitPnpDriver\r\n\0");
    HalInitPnpDriver();
    io_trace(b"[IO] HalInitPnpDriver done\r\n\0");

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] PiQueueDeviceAction\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] PiQueueDeviceAction\r\n\0");
    PiQueueDeviceAction(
        (*(IopRootDeviceNode as *mut ntkrnl_sys::ddk::_DEVICE_NODE)).PhysicalDeviceObject,
        _DEVICE_ACTION_PiActionEnumRootDevices,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    io_trace(b"[IO] PiQueueDeviceAction done\r\n\0");

    IopLoaderBlock = LoaderBlock;

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopInitializeBootDrivers\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopInitializeBootDrivers\r\n\0");
    IopInitializeBootDrivers();
    io_trace(b"[IO] IopInitializeBootDrivers done\r\n\0");

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopReinitializeBootDrivers\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopReinitializeBootDrivers\r\n\0");
    IopReinitializeBootDrivers();
    io_trace(b"[IO] IopReinitializeBootDrivers done\r\n\0");

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] Checking ArcBootDeviceName for ramdisk...\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    if _strnicmp((*LoaderBlock).ArcBootDeviceName, b"ramdisk(0)\0".as_ptr() as *const i8, 10) == 0 {
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] IopStartRamdisk\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
        }

        IopStartRamdisk(LoaderBlock);

        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] IopStartRamdisk done\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }

            {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] Testing ramdisk filesystem access...\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }

                let mut TestOa: _OBJECT_ATTRIBUTES = core::mem::zeroed();
                let mut TestIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
                let mut TestHandle: HANDLE = core::ptr::null_mut();
                let mut TestPath: _UNICODE_STRING = core::mem::zeroed();

                static TEST_PATH: [u16; 47] = [
                    b'\\' as u16, b'A' as u16, b'r' as u16, b'c' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b'\\' as u16,
                    b'r' as u16, b'a' as u16, b'm' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16, b'(' as u16, b'0' as u16, b')' as u16, b'\\' as u16,
                    b'r' as u16, b'e' as u16, b'a' as u16, b'c' as u16, b't' as u16, b'o' as u16, b's' as u16, b'\\' as u16,
                    b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'3' as u16, b'2' as u16, b'\\' as u16,
                    b'n' as u16, b't' as u16, b'd' as u16, b'l' as u16, b'l' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
                ];
                RtlInitUnicodeString(core::ptr::addr_of_mut!(TestPath), TEST_PATH.as_ptr());
                InitializeObjectAttributes(
                    core::ptr::addr_of_mut!(TestOa),
                    core::ptr::addr_of_mut!(TestPath),
                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                let uart2 = Arm64UartKernelBase as *mut ULONG;
                let msg2 = b"[IO] ZwOpenFile(ntdll.dll) start\r\n";
                for &ch in msg2.iter() {
                    Arm64UartPutChar(ch as u8);
                }

                let TestStatus = ZwOpenFile(
                    core::ptr::addr_of_mut!(TestHandle),
                    FILE_READ_ACCESS,
                    core::ptr::addr_of_mut!(TestOa),
                    core::ptr::addr_of_mut!(TestIosb),
                    FILE_SHARE_READ,
                    0,
                );
                if TestStatus >= 0 {
                    let uart3 = Arm64UartKernelBase as *mut ULONG;
                    let msg3 = b"[IO] ZwOpenFile(ntdll.dll) SUCCESS!\r\n";
                    for &ch in msg3.iter() {
                        Arm64UartPutChar(ch as u8);
                    }
                    ZwClose(TestHandle);
                } else {
                    let uart4 = Arm64UartKernelBase as *mut ULONG;
                    let mut buf: [u8; 64] = [0; 64];
                    let len = sprintf(buf.as_mut_ptr() as *mut i8, b"[IO] ZwOpenFile(ntdll.dll) failed: 0x%lx\r\n\0".as_ptr() as *const i8, TestStatus as ULONG);
                    let mut p = buf.as_ptr();
                    let mut i = 0;
                    while i < len && *p != 0 {
                        Arm64UartPutChar(*p as u8);
                        p = p.add(1);
                        i += 1;
                    }
                }
            }
        }
    } else {
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] Not a ramdisk boot\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
        }
    }

    IopLoaderBlock = core::ptr::null_mut();

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopCreateArcNames\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopCreateArcNames\r\n\0");
    Status = IopCreateArcNames(LoaderBlock);
    if Status < 0 {
        io_trace(b"[IO] IopCreateArcNames FAILED\r\n\0");
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode == 0 {
                io_trace(b"[IO] returning 0 from IopCreateArcNames fail\r\n\0");
                return 0u8;
            }
        }
    } else {
        io_trace(b"[IO] IopCreateArcNames done\r\n\0");
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopMarkBootPartition\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopMarkBootPartition\r\n\0");
    if IopMarkBootPartition(LoaderBlock) == 0 {
        io_trace(b"[IO] IopMarkBootPartition FAILED\r\n\0");
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] IopMarkBootPartition failed (continuing on ARM64/MININT)\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode == 0 {
                io_trace(b"[IO] returning 0 from IopMarkBootPartition fail\r\n\0");
                return 0u8;
            }
        }
    } else {
        io_trace(b"[IO] IopMarkBootPartition done\r\n\0");
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] IopMarkBootPartition done\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
        }
    }

    PnPBootDriversInitialized = 1u8;

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopInitializeSystemDrivers\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopInitializeSystemDrivers\r\n\0");
    IopInitializeSystemDrivers();
    io_trace(b"[IO] IopInitializeSystemDrivers done\r\n\0");
    PnpSystemInit = 1u8;

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopInitializeSystemDrivers done\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    IopReinitializeDrivers();

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopReassignSystemRoot\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopReassignSystemRoot\r\n\0");
    Status = IopReassignSystemRoot(LoaderBlock, core::ptr::addr_of_mut!(NtBootPath));
    if Status < 0 {
        io_trace(b"[IO] IopReassignSystemRoot FAILED\r\n\0");
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] IopReassignSystemRoot failed (continuing on ARM64)\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
            return 0u8;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 0u8;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopReassignSystemRoot done\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    RootString.MaximumLength = NtSystemRoot.MaximumLength / core::mem::size_of::<u16>() as USHORT;
    RootString.Length = 0;
    RootString.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        RootString.MaximumLength as SIZE_T,
        TAG_IO,
    ) as *mut i8;

    io_trace(b"[IO] RtlUnicodeStringToAnsiString\r\n\0");
    Status = RtlUnicodeStringToAnsiString(core::ptr::addr_of_mut!(RootString), core::ptr::addr_of_mut!(NtSystemRoot), 0u8);
    if Status < 0 {
        io_trace(b"[IO] RtlUnicodeStringToAnsiString FAILED\r\n\0");
        return 0u8;
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IoAssignDriveLetters\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    IoAssignDriveLetters(
        LoaderBlock,
        core::ptr::addr_of_mut!(NtBootPath),
        RootString.Buffer as *mut u8,
        core::ptr::addr_of_mut!(RootString),
    );

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IoAssignDriveLetters done\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] RtlAnsiStringToUnicodeString\r\n\0");
    Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(NtSystemRoot), core::ptr::addr_of_mut!(RootString), 0u8);
    if Status < 0 {
        io_trace(b"[IO] RtlAnsiStringToUnicodeString FAILED\r\n\0");
        return 0u8;
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] PsLocateSystemDll\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] PsLocateSystemDll\r\n\0");
    Status = PsLocateSystemDll();
    if Status < 0 {
        io_trace(b"[IO] PsLocateSystemDll FAILED\r\n\0");
        #[cfg(target_arch = "aarch64")]
        {
            if !Arm64UartKernelBase.is_null() {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let msg = b"[IO] PsLocateSystemDll failed (continuing on ARM64)\r\n";
                for &ch in msg.iter() {
                    Arm64UartPutChar(ch as u8);
                }
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 0u8;
        }
    }
    io_trace(b"[IO] IoInitSystem returning TRUE\r\n\0");

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IoInitSystem returning TRUE\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    1u8
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IoInitSystem(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut ExpireTime: LARGE_INTEGER = core::mem::zeroed();

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopInitLookasideLists\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    IopInitLookasideLists();

    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopDriverLoadResource));
    InitializeListHead(core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead));
    InitializeListHead(core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead));
    InitializeListHead(core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead));
    InitializeListHead(core::ptr::addr_of_mut!(IopNetworkFileSystemQueueHead));
    InitializeListHead(core::ptr::addr_of_mut!(DriverBootReinitListHead));
    InitializeListHead(core::ptr::addr_of_mut!(DriverReinitListHead));
    InitializeListHead(core::ptr::addr_of_mut!(ShutdownListHead));
    InitializeListHead(core::ptr::addr_of_mut!(LastChanceShutdownListHead));
    InitializeListHead(core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead));
    InitializeListHead(core::ptr::addr_of_mut!(IopErrorLogListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(IoStatisticsLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(DriverReinitListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(DriverBootReinitListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ShutdownListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopLogListLock) as *mut _);

    PiInitializeNotifications();

    io_trace(b"[IO] IopInitializeReserveIrp\r\n\0");
    if IopInitializeReserveIrp(core::ptr::addr_of_mut!(IopReserveIrpAllocator) as _) == 0 {
        io_trace(b"[IO] IopInitializeReserveIrp FAILED\r\n\0");
        return 0u8;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopTimerLock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!(IopTimerQueueHead));

    ExpireTime.QuadPart = -10000000;
    KeInitializeDpc(core::ptr::addr_of_mut!(IopTimerDpc) as *mut _, core::mem::transmute(Some(IopTimerDispatch as unsafe extern "system" fn(_, _, _, _))), core::ptr::null_mut());
    KeInitializeTimerEx(core::ptr::addr_of_mut!(IopTimer) as *mut _, 1);
    KeSetTimerEx(core::ptr::addr_of_mut!(IopTimer) as *mut _, ExpireTime, 1000, core::ptr::addr_of_mut!(IopTimerDpc) as *mut _);

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopCreateObjectTypes\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopCreateObjectTypes\r\n\0");
    if IopCreateObjectTypes() == 0 {
        io_trace(b"[IO] IopCreateObjectTypes FAILED\r\n\0");
        return 0u8;
    }

    #[cfg(target_arch = "aarch64")]
    {
        if !Arm64UartKernelBase.is_null() {
            let uart = Arm64UartKernelBase as *mut ULONG;
            let msg = b"[IO] IopCreateRootDirectories\r\n";
            for &ch in msg.iter() {
                Arm64UartPutChar(ch as u8);
            }
        }
    }

    io_trace(b"[IO] IopCreateRootDirectories\r\n\0");
    if IopCreateRootDirectories() == 0 {
        io_trace(b"[IO] IopCreateRootDirectories FAILED\r\n\0");
        return 0u8;
    }
    io_trace(b"[IO] entering IopFinishIoInitSystem\r\n\0");

    // Keep the post-PnP phase out of the large entry function on i386; LLVM has
    // previously collapsed the continuation after PnP init into a `ud2`.
    IopFinishIoInitSystem(LoaderBlock)
}

#[no_mangle]
pub unsafe extern "system" fn IoInitializeCrashDump(_PageFileHandle: HANDLE) -> BOOLEAN {
    let _PageFileHandle = opaque_ptr(_PageFileHandle);
    0u8
}
