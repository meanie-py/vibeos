use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    ANSI_STRING,
    CONTROLLER_OBJECT,
    DEVICE_OBJECT,
    DRIVER_OBJECT,
    ERESOURCE,
    ExInitializeResourceLite,
    ExInitializeSystemLookasideList,
    FILE_OBJECT,
    _GENERIC_MAPPING,
    IOP_MINI_COMPLETION_PACKET,
    IO_STACK_LOCATION,
    _IO_STATUS_BLOCK,
    _UNICODE_STRING,
    IRP,
    IoAssignDriveLetters,
    IopCloseFile,
    IopCreateArcNames,
    IopDeleteDevice,
    IopDeleteDriver,
    IopDeleteFile,
    IopDeleteIoCompletion,
    IopGetSetSecurityObject,
    IopInitializeBootDrivers,
    IopInitializePlugPlayServices,
    IopInitializeReserveIrp,
    IopInitializeSystemDrivers,
    IopParseDevice,
    IopParseFile,
    IopQueryName,
    IopReassignSystemRoot,
    IopReinitializeBootDrivers,
    IopReinitializeDrivers,
    IopStartRamdisk,
    KQUEUE,
    KTIMER,
    KeInitializeDpc,
    KeInitializeSpinLock,
    KeInitializeTimerEx,
    KeSetTimerEx,
    LARGE_INTEGER,
    LIST_ENTRY,
    MDL,
    NtClose,
    NtCreateDirectoryObject,
    _OBJECT_ATTRIBUTES,
    OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType,
    ObReferenceObjectByHandle,
    RtlInitUnicodeString,
    ZwClose,
    ZwOpenFile,
    PDEVICE_OBJECT,
    PLOADER_PARAMETER_BLOCK,
    HANDLE,
    KPROCESSOR_MODE,
    _LOADER_PARAMETER_BLOCK,
    POBJECT_TYPE,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList,
};

// IO tag constants
const IOC_TAG1: ULONG = u32::from_le_bytes(*b" cpI");
const IO_LARGEIRP: ULONG = u32::from_le_bytes(*b"lprI");
const IO_SMALLIRP: ULONG = u32::from_le_bytes(*b"sprI");
const IO_LARGEIRP_CPU: ULONG = u32::from_le_bytes(*b"LprI");
const IO_SMALLIRP_CPU: ULONG = u32::from_le_bytes(*b"SprI");
const TAG_IO: ULONG = u32::from_le_bytes(*b"  oI");
const IO_COMPLETION_ALL_ACCESS: ULONG = 0x001F0003;
const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
const INACCESSIBLE_BOOT_DEVICE: ULONG = 0x0000007B;
const DO_SYSTEM_BOOT_PARTITION: ULONG = 0x00000100;
const FILE_READ_ACCESS: ULONG = 0x00000001;
const FILE_SHARE_READ: ULONG = 0x00000001;

#[cfg(target_arch = "aarch64")]
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
}

#[cfg(target_arch = "aarch64")]
macro_rules! ARM64_UART_TRACE {
    ($s:expr) => {
        unsafe {
            let uart = Arm64UartKernelBase as *mut ULONG;
            if !uart.is_null() {
                let bytes = $s.as_bytes();
                for &b in bytes {
                    *uart = b as ULONG;
                }
            }
        }
    };
}

#[cfg(not(target_arch = "aarch64"))]
macro_rules! ARM64_UART_TRACE {
    ($s:expr) => {};
}

#[no_mangle]
pub static mut IopTraceLevel: ULONG = 0;
#[no_mangle]
pub static mut PnpSystemInit: BOOLEAN = 0u8;
#[no_mangle]
pub static mut IoDeviceObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle]
pub static mut IoFileObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle]
pub static mut IopFileMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE,
    GenericWrite: STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES | FILE_EXECUTE | SYNCHRONIZE,
    GenericAll: FILE_ALL_ACCESS,
};
#[no_mangle]
pub static mut IoStatisticsLock: KSPIN_LOCK = 0;

extern "system" {
    fn IopTimerDispatch( Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID, );
}
extern "C" {
    fn WmiInitialize() -> BOOLEAN;
    fn PiInitializeNotifications();
    fn ApphelpCacheInitialize();
    #[link_name = "HalInitPnpDriver_shim"]
    fn HalInitPnpDriver();
    fn PiQueueDeviceAction( DeviceObject: PDEVICE_OBJECT, Action: ULONG, Callback: PVOID, Context: PVOID, );
    fn PsLocateSystemDll() -> NTSTATUS;
    fn RtlAnsiStringToUnicodeString( DestinationString: *mut _UNICODE_STRING, SourceString: *const ANSI_STRING, AllocateDestinationString: BOOLEAN, ) -> NTSTATUS;
    fn RtlUnicodeStringToAnsiString( DestinationString: *mut ANSI_STRING, SourceString: *const _UNICODE_STRING, AllocateDestinationString: BOOLEAN, ) -> NTSTATUS;
    fn RtlInitAnsiString(DestinationString: *mut ANSI_STRING, SourceString: *const i8);
    fn RtlInitEmptyAnsiString( AnsiString: *mut ANSI_STRING, Buffer: *mut i8, BufferSize: USHORT, );
    fn RtlFreeUnicodeString(UnicodeString: *mut _UNICODE_STRING);
    static mut IoDriverObjectType: POBJECT_TYPE;
    static mut IoCompletionType: POBJECT_TYPE;
    static mut IoControllerObjectType: POBJECT_TYPE;
    static mut IopCompletionMapping: _GENERIC_MAPPING;
    static mut ShutdownListHead: LIST_ENTRY;
    static mut LastChanceShutdownListHead: LIST_ENTRY;
    static mut ShutdownListLock: KSPIN_LOCK;
    static mut IoAdapterObjectType: POBJECT_TYPE;
    static mut IopDatabaseResource: ERESOURCE;
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
    static mut IoCompletionPacketLookaside: GENERAL_LOOKASIDE;
    static mut IopReserveIrpAllocator: PVOID;
    static mut ExSystemLookasideListHead: LIST_ENTRY;
    static mut KeNumberProcessors: ULONG;
    static KiProcessorBlock: [*mut _KPRCB; 0];
    static mut IopRootDeviceNode: *mut PVOID;
    static mut NtSystemRoot: _UNICODE_STRING;
    static mut InitIsWinPEMode: BOOLEAN;
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
}

use ntkrnl_sys::ddk::_KPRCB;

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
pub static mut IopAutoReboot: ULONG = 0;
#[no_mangle]
pub static mut IopNumTriageDumpDataBlocks: ULONG = 0;
#[no_mangle]
pub static mut IopTriageDumpDataBlocks: [PVOID; 64] = [core::ptr::null_mut(); 64];

#[no_mangle]
pub static mut IopSecurityResource: ERESOURCE = unsafe { core::mem::zeroed() };

static mut IoLargeIrpLookaside: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
static mut IoSmallIrpLookaside: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };
static mut IopMdlLookasideList: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IopLoaderBlock: PLOADER_PARAMETER_BLOCK = core::ptr::null_mut();

unsafe fn IopInitLookasideLists() {
    let LargeIrpSize: ULONG;
    let SmallIrpSize: ULONG;
    let MdlSize: ULONG;
    let mut CurrentList: PGENERAL_LOOKASIDE = core::ptr::null_mut();

    LargeIrpSize = (core::mem::size_of::<IRP>() + (8 * core::mem::size_of::<IO_STACK_LOCATION>())) as ULONG;
    SmallIrpSize = (core::mem::size_of::<IRP>() + core::mem::size_of::<IO_STACK_LOCATION>()) as ULONG;
    MdlSize = (core::mem::size_of::<MDL>() + (23 * core::mem::size_of::<PFN_NUMBER>())) as ULONG;

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _,
        NonPagedPool,
        core::mem::size_of::<IOP_MINI_COMPLETION_PACKET>() as ULONG,
        IOC_TAG1,
        32,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _,
        NonPagedPool,
        LargeIrpSize,
        IO_LARGEIRP,
        64,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _,
        NonPagedPool,
        SmallIrpSize,
        IO_SMALLIRP,
        32,
        core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
    );

    ExInitializeSystemLookasideList(
        core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _,
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

    for i in 0..KeNumberProcessors as isize {
        let Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const *mut _KPRCB).offset(i);

        (*Prcb).LookasideIrpFloat = (512 / KeNumberProcessors) as i32;

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].L = core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _,
                NonPagedPool,
                core::mem::size_of::<IOP_MINI_COMPLETION_PACKET>() as ULONG,
                IO_SMALLIRP_CPU,
                32,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].P = CurrentList as *mut _;
            CurrentList = CurrentList.offset(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize].P = core::ptr::addr_of_mut!(IoCompletionPacketLookaside) as *mut _;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].L = core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _,
                NonPagedPool,
                LargeIrpSize,
                IO_LARGEIRP_CPU,
                64,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].P = CurrentList as *mut _;
            CurrentList = CurrentList.offset(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideLargeIrpList as usize].P = core::ptr::addr_of_mut!(IoLargeIrpLookaside) as *mut _;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].L = core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _,
                NonPagedPool,
                SmallIrpSize,
                IO_SMALLIRP_CPU,
                32,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].P = CurrentList as *mut _;
            CurrentList = CurrentList.offset(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideSmallIrpList as usize].P = core::ptr::addr_of_mut!(IoSmallIrpLookaside) as *mut _;
        }

        (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].L = core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _;
        if !CurrentList.is_null() {
            ExInitializeSystemLookasideList(
                CurrentList as *mut _,
                NonPagedPool,
                SmallIrpSize,
                TAG_MDL,
                128,
                core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY,
            );
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].P = CurrentList as *mut _;
            CurrentList = CurrentList.offset(1);
        } else {
            (*Prcb).PPLookasideList[_PP_NPAGED_LOOKASIDE_NUMBER_LookasideMdlList as usize].P = core::ptr::addr_of_mut!(IopMdlLookasideList) as *mut _;
        }
    }
}

unsafe fn IopCreateObjectTypes() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as PVOID,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as USHORT;
    ObjectTypeInitializer.PoolType = NonPagedPool;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = 1u8;
    ObjectTypeInitializer.GenericMapping = IopFileMapping;

    RtlInitUnicodeString(&mut Name, b"A\0d\0a\0p\0t\0e\0r\0\0\0".as_ptr() as *const u16);
    if ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoAdapterObjectType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, b"C\0o\0n\0t\0r\0o\0l\0l\0e\0r\0\0\0".as_ptr() as *const u16);
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<CONTROLLER_OBJECT>() as ULONG;
    if ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoControllerObjectType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, b"D\0e\0v\0i\0c\0e\0\0\0".as_ptr() as *const u16);
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<DEVICE_OBJECT>() as ULONG;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteDevice);
    ObjectTypeInitializer.ParseProcedure = Some(IopParseDevice);
    ObjectTypeInitializer.SecurityProcedure = Some(IopGetSetSecurityObject);
    ObjectTypeInitializer.CaseInsensitive = 1u8;
    if ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoDeviceObjectType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, b"D\0r\0i\0v\0e\0r\0\0\0".as_ptr() as *const u16);
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<DRIVER_OBJECT>() as ULONG;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteDriver);
    ObjectTypeInitializer.ParseProcedure = None;
    ObjectTypeInitializer.SecurityProcedure = None;
    if ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoDriverObjectType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, b"I\0o\0C\0o\0m\0p\0l\0e\0t\0i\0o\0n\0\0\0".as_ptr() as *const u16);
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<KQUEUE>() as ULONG;
    ObjectTypeInitializer.ValidAccessMask = IO_COMPLETION_ALL_ACCESS;
    ObjectTypeInitializer.InvalidAttributes |= OBJ_PERMANENT;
    ObjectTypeInitializer.GenericMapping = IopCompletionMapping;
    ObjectTypeInitializer.DeleteProcedure = Some(IopDeleteIoCompletion);
    if ObCreateObjectType(
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoCompletionType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, b"F\0i\0l\0e\0\0\0".as_ptr() as *const u16);
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
        &mut Name,
        &mut ObjectTypeInitializer,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoFileObjectType) as *mut _,
    ) < 0
    {
        return 0u8;
    }

    1u8
}

unsafe fn IopCreateRootDirectories() -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirName: _UNICODE_STRING = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut DirName, b"\\\0D\0r\0i\0v\0e\0r\0\0\0".as_ptr() as *const u16);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DirName;
    ObjectAttributes.Attributes = OBJ_PERMANENT;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateDirectoryObject(
        &mut Handle,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    RtlInitUnicodeString(&mut DirName, b"\\\0F\0i\0l\0e\0S\0y\0s\0t\0e\0m\0\0\0".as_ptr() as *const u16);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DirName;
    ObjectAttributes.Attributes = OBJ_PERMANENT;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateDirectoryObject(
        &mut Handle,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    RtlInitUnicodeString(&mut DirName, b"\\\0F\0i\0l\0e\0S\0y\0s\0t\0e\0m\0\\\0F\0i\0l\0t\0e\0r\0s\0\0\0".as_ptr() as *const u16);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DirName;
    ObjectAttributes.Attributes = OBJ_PERMANENT;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateDirectoryObject(
        &mut Handle,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        return 0u8;
    }
    NtClose(Handle);

    1u8
}

unsafe fn IopMarkBootPartition(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DeviceString: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: [i8; 256] = [0; 256];
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();

    sprintf(
        Buffer.as_mut_ptr(),
        b"\\ArcName\\%s\0".as_ptr() as *const i8,
        (*LoaderBlock).ArcBootDeviceName,
    );
    RtlInitAnsiString(&mut DeviceString, Buffer.as_ptr());
    Status = RtlAnsiStringToUnicodeString(&mut DeviceName, &DeviceString, 1u8);
    if Status < 0 {
        return 0u8;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DeviceName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = ZwOpenFile(
        &mut FileHandle,
        FILE_READ_ATTRIBUTES,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        0,
        FILE_NON_DIRECTORY_FILE,
    );
    if Status < 0 {
        #[cfg(target_arch = "aarch64")]
        {
            RtlFreeUnicodeString(&mut DeviceName);
            return 0u8;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode != 0 {
                RtlFreeUnicodeString(&mut DeviceName);
                return 0u8;
            }
            KeBugCheckEx(
                INACCESSIBLE_BOOT_DEVICE,
                &mut DeviceName as *mut _UNICODE_STRING as ULONG_PTR,
                Status as ULONG_PTR,
                0,
                0,
            );
        }
    }

    Status = ObReferenceObjectByHandle(
        FileHandle,
        0,
        IoFileObjectType as *mut _,
        0i8,
        core::ptr::addr_of_mut!(FileObject) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        RtlFreeUnicodeString(&mut DeviceName);
        return 0u8;
    }

    (*(*FileObject).DeviceObject).Flags |= DO_SYSTEM_BOOT_PARTITION;

    ObReferenceObject((*FileObject).DeviceObject as PVOID);
    IopErrorLogObject = (*FileObject).DeviceObject;

    RtlFreeUnicodeString(&mut DeviceName);
    NtClose(FileHandle);
    ObDereferenceObject(FileObject as PVOID);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IoInitSystem(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> BOOLEAN {
    let mut ExpireTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: [i8; 256] = [0; 256];
    let mut NtBootPath: ANSI_STRING = core::mem::zeroed();
    let mut RootString: ANSI_STRING = core::mem::zeroed();

    RtlInitEmptyAnsiString(&mut NtBootPath, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as USHORT);

    ARM64_UART_TRACE!("[IO] IopInitLookasideLists\r\n");
    IopInitLookasideLists();

    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopDatabaseResource));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopSecurityResource));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopDriverLoadResource));
    InitializeListHead(core::ptr::addr_of_mut!(IopDiskFileSystemQueueHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(IopCdRomFileSystemQueueHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(IopTapeFileSystemQueueHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(IopNetworkFileSystemQueueHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(DriverBootReinitListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(DriverReinitListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(ShutdownListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(LastChanceShutdownListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(IopFsNotifyChangeQueueHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(IopErrorLogListHead) as PLIST_ENTRY);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(IoStatisticsLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(DriverReinitListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(DriverBootReinitListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ShutdownListLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopLogListLock) as *mut _);

    PiInitializeNotifications();

    if IopInitializeReserveIrp(core::ptr::addr_of_mut!(IopReserveIrpAllocator) as *mut _) == 0 {
        return 0u8;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopTimerLock) as *mut _);

    InitializeListHead(core::ptr::addr_of_mut!(IopTimerQueueHead) as PLIST_ENTRY);

    ExpireTime.QuadPart = -10000000i64;
    KeInitializeDpc(core::ptr::addr_of_mut!(IopTimerDpc) as *mut _, core::mem::transmute(Some(IopTimerDispatch as unsafe extern "system" fn(PKDPC, PVOID, PVOID, PVOID))), core::ptr::null_mut());
    KeInitializeTimerEx(core::ptr::addr_of_mut!(IopTimer) as *mut _, 1u32);
    KeSetTimerEx(
        core::ptr::addr_of_mut!(IopTimer) as *mut _,
        ExpireTime,
        1000,
        core::ptr::addr_of_mut!(IopTimerDpc) as *mut _,
    );

    ARM64_UART_TRACE!("[IO] IopCreateObjectTypes\r\n");
    if IopCreateObjectTypes() == 0 {
        return 0u8;
    }

    ARM64_UART_TRACE!("[IO] IopCreateRootDirectories\r\n");
    if IopCreateRootDirectories() == 0 {
        return 0u8;
    }

    ARM64_UART_TRACE!("[IO] IopInitializePlugPlayServices\r\n");
    IopInitializePlugPlayServices();

    ApphelpCacheInitialize();

    WmiInitialize();

    ARM64_UART_TRACE!("[IO] HalInitPnpDriver\r\n");
    HalInitPnpDriver();

    ARM64_UART_TRACE!("[IO] PiQueueDeviceAction\r\n");
    PiQueueDeviceAction(
        (*IopRootDeviceNode) as PDEVICE_OBJECT,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    IopLoaderBlock = LoaderBlock;

    ARM64_UART_TRACE!("[IO] IopInitializeBootDrivers\r\n");
    IopInitializeBootDrivers();

    ARM64_UART_TRACE!("[IO] IopReinitializeBootDrivers\r\n");
    IopReinitializeBootDrivers();

    ARM64_UART_TRACE!("[IO] Checking ArcBootDeviceName for ramdisk...\r\n");
    let arc_name = (*LoaderBlock).ArcBootDeviceName;
    let mut is_ramdisk = false;
    if !arc_name.is_null() {
        let slice = core::slice::from_raw_parts(arc_name as *const u8, 10);
        if slice.eq_ignore_ascii_case(b"ramdisk(0)") {
            is_ramdisk = true;
        }
    }

    if is_ramdisk {
        ARM64_UART_TRACE!("[IO] IopStartRamdisk\r\n");
        IopStartRamdisk(LoaderBlock);
        ARM64_UART_TRACE!("[IO] IopStartRamdisk done\r\n");

        #[cfg(target_arch = "aarch64")]
        {
            let mut TestOa: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut TestIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut TestHandle: HANDLE = core::ptr::null_mut();
            let mut TestPath: _UNICODE_STRING = core::mem::zeroed();

            ARM64_UART_TRACE!("[IO] Testing ramdisk filesystem access...\r\n");
            RtlInitUnicodeString(&mut TestPath, b"\\\0A\0r\0c\0N\0a\0m\0e\0\\\0r\0a\0m\0d\0i\0s\0k\0(\x000\0)\0\\\0r\0e\0a\0c\0t\0o\0s\0\\\0s\0y\0s\0t\0e\0m\x003\x002\0\\\0n\0t\0d\0l\0l\0.\0d\0l\0l\0\0\0".as_ptr() as *const u16);
            TestOa.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            TestOa.RootDirectory = core::ptr::null_mut();
            TestOa.ObjectName = &mut TestPath;
            TestOa.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
            TestOa.SecurityDescriptor = core::ptr::null_mut();
            TestOa.SecurityQualityOfService = core::ptr::null_mut();
            ARM64_UART_TRACE!("[IO] ZwOpenFile(ntdll.dll) start\r\n");
            let TestStatus = ZwOpenFile(
                &mut TestHandle,
                FILE_READ_ACCESS,
                &mut TestOa,
                &mut TestIosb,
                FILE_SHARE_READ,
                0,
            );
            if TestStatus >= 0 {
                ARM64_UART_TRACE!("[IO] ZwOpenFile(ntdll.dll) SUCCESS!\r\n");
                ZwClose(TestHandle);
            } else {
                let uart = Arm64UartKernelBase as *mut ULONG;
                let mut buf: [i8; 64] = [0; 64];
                let len = sprintf(
                    buf.as_mut_ptr(),
                    b"[IO] ZwOpenFile(ntdll.dll) failed: 0x%lx\r\n\0".as_ptr() as *const i8,
                    TestStatus as u32,
                );
                if !uart.is_null() {
                    for i in 0..len {
                        *uart = buf[i as usize] as ULONG;
                    }
                }
            }
        }
    } else {
        ARM64_UART_TRACE!("[IO] Not a ramdisk boot\r\n");
    }

    IopLoaderBlock = core::ptr::null_mut();

    ARM64_UART_TRACE!("[IO] IopCreateArcNames\r\n");
    Status = IopCreateArcNames(LoaderBlock);
    if Status < 0 {
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode == 0 {
                return 0u8;
            }
        }
    }

    ARM64_UART_TRACE!("[IO] IopMarkBootPartition\r\n");
    if IopMarkBootPartition(LoaderBlock) == 0 {
        ARM64_UART_TRACE!("[IO] IopMarkBootPartition failed (continuing on ARM64/MININT)\r\n");
        #[cfg(not(target_arch = "aarch64"))]
        {
            if InitIsWinPEMode == 0 {
                return 0u8;
            }
        }
    } else {
        ARM64_UART_TRACE!("[IO] IopMarkBootPartition done\r\n");
    }

    PnPBootDriversInitialized = 1u8;

    ARM64_UART_TRACE!("[IO] IopInitializeSystemDrivers\r\n");
    IopInitializeSystemDrivers();
    PnpSystemInit = 1u8;
    ARM64_UART_TRACE!("[IO] IopInitializeSystemDrivers done\r\n");

    IopReinitializeDrivers();

    ARM64_UART_TRACE!("[IO] IopReassignSystemRoot\r\n");
    Status = IopReassignSystemRoot(LoaderBlock, &mut NtBootPath);
    if Status < 0 {
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE!("[IO] IopReassignSystemRoot failed (continuing on ARM64)\r\n");
            return 0u8;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            ARM64_UART_TRACE!("[IO] IopReassignSystemRoot failed\r\n");
            return 0u8;
        }
    }
    ARM64_UART_TRACE!("[IO] IopReassignSystemRoot done\r\n");

    RootString.MaximumLength = (NtSystemRoot.MaximumLength / core::mem::size_of::<u16>() as u16) as USHORT;
    RootString.Length = 0;
    RootString.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        RootString.MaximumLength as SIZE_T,
        TAG_IO,
    ) as *mut i8;

    Status = RtlUnicodeStringToAnsiString(&mut RootString, &NtSystemRoot, 0u8);
    if Status < 0 {
        return 0u8;
    }

    ARM64_UART_TRACE!("[IO] IoAssignDriveLetters\r\n");
    IoAssignDriveLetters(
        LoaderBlock,
        &mut NtBootPath,
        RootString.Buffer as *mut u8,
        &mut RootString,
    );
    ARM64_UART_TRACE!("[IO] IoAssignDriveLetters done\r\n");

    Status = RtlAnsiStringToUnicodeString(&mut NtSystemRoot, &RootString, 0u8);
    if Status < 0 {
        return 0u8;
    }

    ARM64_UART_TRACE!("[IO] PsLocateSystemDll\r\n");
    Status = PsLocateSystemDll();
    if Status < 0 {
        #[cfg(target_arch = "aarch64")]
        {
            ARM64_UART_TRACE!("[IO] PsLocateSystemDll failed (continuing on ARM64)\r\n");
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            return 0u8;
        }
    }

    ARM64_UART_TRACE!("[IO] IoInitSystem returning TRUE\r\n");
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IoInitializeCrashDump(_PageFileHandle: HANDLE) -> BOOLEAN {
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel
//  * FILE:            ntoskrnl/io/iomgr/iomgr.c
//  * PURPOSE:         I/O Manager Initialization and Misc Utility Functions
//  *
//  * PROGRAMMERS:     David Welch (welch@mcmail.com)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* Direct UART trace for ARM64 boot debugging — bypasses PCH limitation */
// #ifdef _M_ARM64
// extern volatile PVOID Arm64UartKernelBase;
// #undef ARM64_UART_TRACE
// #define ARM64_UART_TRACE(s) do { \
//     volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase; \
//     const char *_p = (s); \
//     if (!_uart) break; \
//     while (*_p) *_uart = (ULONG)*_p++; \
// } while(0)
// #endif
//
// ULONG IopTraceLevel = 0;
// BOOLEAN PnpSystemInit = FALSE;
//
//
// VOID
// NTAPI
// IopTimerDispatch(
//     IN PKDPC Dpc,
//     IN PVOID DeferredContext,
//     IN PVOID SystemArgument1,
//     IN PVOID SystemArgument2
// );
//
// BOOLEAN
// NTAPI
// WmiInitialize(
//     VOID);
//
// /* DATA ********************************************************************/
//
// POBJECT_TYPE IoDeviceObjectType = NULL;
// POBJECT_TYPE IoFileObjectType = NULL;
// extern POBJECT_TYPE IoControllerObjectType;
// BOOLEAN IoCountOperations = TRUE;
// ULONG IoReadOperationCount = 0;
// LARGE_INTEGER IoReadTransferCount = {{0, 0}};
// ULONG IoWriteOperationCount = 0;
// LARGE_INTEGER IoWriteTransferCount = {{0, 0}};
// ULONG IoOtherOperationCount = 0;
// LARGE_INTEGER IoOtherTransferCount = {{0, 0}};
// KSPIN_LOCK IoStatisticsLock = 0;
// ULONG IopAutoReboot;
// ULONG IopNumTriageDumpDataBlocks;
// PVOID IopTriageDumpDataBlocks[64];
//
// GENERIC_MAPPING IopFileMapping = {
//     FILE_GENERIC_READ,
//     FILE_GENERIC_WRITE,
//     FILE_GENERIC_EXECUTE,
//     FILE_ALL_ACCESS};
//
// extern LIST_ENTRY ShutdownListHead;
// extern LIST_ENTRY LastChanceShutdownListHead;
// extern KSPIN_LOCK ShutdownListLock;
// extern POBJECT_TYPE IoAdapterObjectType;
// extern ERESOURCE IopDatabaseResource;
// ERESOURCE IopSecurityResource;
// extern ERESOURCE IopDriverLoadResource;
// extern LIST_ENTRY IopDiskFileSystemQueueHead;
// extern LIST_ENTRY IopCdRomFileSystemQueueHead;
// extern LIST_ENTRY IopTapeFileSystemQueueHead;
// extern LIST_ENTRY IopNetworkFileSystemQueueHead;
// extern LIST_ENTRY DriverBootReinitListHead;
// extern LIST_ENTRY DriverReinitListHead;
// extern LIST_ENTRY IopFsNotifyChangeQueueHead;
// extern LIST_ENTRY IopErrorLogListHead;
// extern LIST_ENTRY IopTimerQueueHead;
// extern KDPC IopTimerDpc;
// extern KTIMER IopTimer;
// extern KSPIN_LOCK IoStatisticsLock;
// extern KSPIN_LOCK DriverReinitListLock;
// extern KSPIN_LOCK DriverBootReinitListLock;
// extern KSPIN_LOCK IopLogListLock;
// extern KSPIN_LOCK IopTimerLock;
//
// extern PDEVICE_OBJECT IopErrorLogObject;
// extern BOOLEAN PnPBootDriversInitialized;
//
// GENERAL_LOOKASIDE IoLargeIrpLookaside;
// GENERAL_LOOKASIDE IoSmallIrpLookaside;
// GENERAL_LOOKASIDE IopMdlLookasideList;
// extern GENERAL_LOOKASIDE IoCompletionPacketLookaside;
//
// PLOADER_PARAMETER_BLOCK IopLoaderBlock;
//
// /* INIT FUNCTIONS ************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// IopInitLookasideLists(VOID)
// {
//     ULONG LargeIrpSize, SmallIrpSize, MdlSize;
//     LONG i;
//     PKPRCB Prcb;
//     PGENERAL_LOOKASIDE CurrentList = NULL;
//
//     /* Calculate the sizes */
//     LargeIrpSize = sizeof(IRP) + (8 * sizeof(IO_STACK_LOCATION));
//     SmallIrpSize = sizeof(IRP) + sizeof(IO_STACK_LOCATION);
//     MdlSize = sizeof(MDL) + (23 * sizeof(PFN_NUMBER));
//
//     /* Initialize the Lookaside List for I\O Completion */
//     ExInitializeSystemLookasideList(&IoCompletionPacketLookaside,
//                                     NonPagedPool,
//                                     sizeof(IOP_MINI_COMPLETION_PACKET),
//                                     IOC_TAG1,
//                                     32,
//                                     &ExSystemLookasideListHead);
//
//     /* Initialize the Lookaside List for Large IRPs */
//     ExInitializeSystemLookasideList(&IoLargeIrpLookaside,
//                                     NonPagedPool,
//                                     LargeIrpSize,
//                                     IO_LARGEIRP,
//                                     64,
//                                     &ExSystemLookasideListHead);
//
//
//     /* Initialize the Lookaside List for Small IRPs */
//     ExInitializeSystemLookasideList(&IoSmallIrpLookaside,
//                                     NonPagedPool,
//                                     SmallIrpSize,
//                                     IO_SMALLIRP,
//                                     32,
//                                     &ExSystemLookasideListHead);
//
//     /* Initialize the Lookaside List for MDLs */
//     ExInitializeSystemLookasideList(&IopMdlLookasideList,
//                                     NonPagedPool,
//                                     MdlSize,
//                                     TAG_MDL,
//                                     128,
//                                     &ExSystemLookasideListHead);
//
//     /* Allocate the global lookaside list buffer */
//     CurrentList = ExAllocatePoolWithTag(NonPagedPool,
//                                         4 * KeNumberProcessors *
//                                         sizeof(GENERAL_LOOKASIDE),
//                                         TAG_IO);
//
//     /* Loop all processors */
//     for (i = 0; i < KeNumberProcessors; i++)
//     {
//         /* Get the PRCB for this CPU */
//         Prcb = KiProcessorBlock[i];
//         DPRINT("Setting up lookaside for CPU: %x, PRCB: %p\n", i, Prcb);
//
//         /* Write IRP credit limit */
//         Prcb->LookasideIrpFloat = 512 / KeNumberProcessors;
//
//         /* Set the I/O Completion List */
//         Prcb->PPLookasideList[LookasideCompletionList].L = &IoCompletionPacketLookaside;
//         if (CurrentList)
//         {
//             /* Initialize the Lookaside List for mini-packets */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             NonPagedPool,
//                                             sizeof(IOP_MINI_COMPLETION_PACKET),
//                                             IO_SMALLIRP_CPU,
//                                             32,
//                                             &ExSystemLookasideListHead);
//             Prcb->PPLookasideList[LookasideCompletionList].P = CurrentList;
//             CurrentList++;
//
//         }
//         else
//         {
//             Prcb->PPLookasideList[LookasideCompletionList].P = &IoCompletionPacketLookaside;
//         }
//
//         /* Set the Large IRP List */
//         Prcb->PPLookasideList[LookasideLargeIrpList].L = &IoLargeIrpLookaside;
//         if (CurrentList)
//         {
//             /* Initialize the Lookaside List for Large IRPs */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             NonPagedPool,
//                                             LargeIrpSize,
//                                             IO_LARGEIRP_CPU,
//                                             64,
//                                             &ExSystemLookasideListHead);
//             Prcb->PPLookasideList[LookasideLargeIrpList].P = CurrentList;
//             CurrentList++;
//
//         }
//         else
//         {
//             Prcb->PPLookasideList[LookasideLargeIrpList].P = &IoLargeIrpLookaside;
//         }
//
//         /* Set the Small IRP List */
//         Prcb->PPLookasideList[LookasideSmallIrpList].L = &IoSmallIrpLookaside;
//         if (CurrentList)
//         {
//             /* Initialize the Lookaside List for Small IRPs */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             NonPagedPool,
//                                             SmallIrpSize,
//                                             IO_SMALLIRP_CPU,
//                                             32,
//                                             &ExSystemLookasideListHead);
//             Prcb->PPLookasideList[LookasideSmallIrpList].P = CurrentList;
//             CurrentList++;
//
//         }
//         else
//         {
//             Prcb->PPLookasideList[LookasideSmallIrpList].P = &IoSmallIrpLookaside;
//         }
//
//         /* Set the MDL Completion List */
//         Prcb->PPLookasideList[LookasideMdlList].L = &IopMdlLookasideList;
//         if (CurrentList)
//         {
//             /* Initialize the Lookaside List for MDLs */
//             ExInitializeSystemLookasideList(CurrentList,
//                                             NonPagedPool,
//                                             SmallIrpSize,
//                                             TAG_MDL,
//                                             128,
//                                             &ExSystemLookasideListHead);
//
//             Prcb->PPLookasideList[LookasideMdlList].P = CurrentList;
//             CurrentList++;
//
//         }
//         else
//         {
//             Prcb->PPLookasideList[LookasideMdlList].P = &IopMdlLookasideList;
//         }
//     }
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// IopCreateObjectTypes(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     UNICODE_STRING Name;
//
//     /* Initialize default settings */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//     ObjectTypeInitializer.GenericMapping = IopFileMapping;
//
//     /* Do the Adapter Type */
//     RtlInitUnicodeString(&Name, L"Adapter");
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoAdapterObjectType))) return FALSE;
//
//     /* Do the Controller Type */
//     RtlInitUnicodeString(&Name, L"Controller");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(CONTROLLER_OBJECT);
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoControllerObjectType))) return FALSE;
//
//     /* Do the Device Type */
//     RtlInitUnicodeString(&Name, L"Device");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(DEVICE_OBJECT);
//     ObjectTypeInitializer.DeleteProcedure = IopDeleteDevice;
//     ObjectTypeInitializer.ParseProcedure = IopParseDevice;
//     ObjectTypeInitializer.SecurityProcedure = IopGetSetSecurityObject;
//     ObjectTypeInitializer.CaseInsensitive = TRUE;
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoDeviceObjectType))) return FALSE;
//
//     /* Initialize the Driver object type */
//     RtlInitUnicodeString(&Name, L"Driver");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(DRIVER_OBJECT);
//     ObjectTypeInitializer.DeleteProcedure = IopDeleteDriver;
//     ObjectTypeInitializer.ParseProcedure = NULL;
//     ObjectTypeInitializer.SecurityProcedure = NULL;
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoDriverObjectType))) return FALSE;
//
//     /* Initialize the I/O Completion object type */
//     RtlInitUnicodeString(&Name, L"IoCompletion");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(KQUEUE);
//     ObjectTypeInitializer.ValidAccessMask = IO_COMPLETION_ALL_ACCESS;
//     ObjectTypeInitializer.InvalidAttributes |= OBJ_PERMANENT;
//     ObjectTypeInitializer.GenericMapping = IopCompletionMapping;
//     ObjectTypeInitializer.DeleteProcedure = IopDeleteIoCompletion;
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoCompletionType))) return FALSE;
//
//     /* Initialize the File object type  */
//     RtlInitUnicodeString(&Name, L"File");
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(FILE_OBJECT);
//     ObjectTypeInitializer.InvalidAttributes |= OBJ_EXCLUSIVE;
//     ObjectTypeInitializer.MaintainHandleCount = TRUE;
//     ObjectTypeInitializer.ValidAccessMask = FILE_ALL_ACCESS;
//     ObjectTypeInitializer.GenericMapping = IopFileMapping;
//     ObjectTypeInitializer.CloseProcedure = IopCloseFile;
//     ObjectTypeInitializer.DeleteProcedure = IopDeleteFile;
//     ObjectTypeInitializer.SecurityProcedure = IopGetSetSecurityObject;
//     ObjectTypeInitializer.QueryNameProcedure = IopQueryName;
//     ObjectTypeInitializer.ParseProcedure = IopParseFile;
//     ObjectTypeInitializer.UseDefaultObject = FALSE;
//     if (!NT_SUCCESS(ObCreateObjectType(&Name,
//                                        &ObjectTypeInitializer,
//                                        NULL,
//                                        &IoFileObjectType))) return FALSE;
//
//     /* Success */
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// IopCreateRootDirectories(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING DirName;
//     HANDLE Handle;
//     NTSTATUS Status;
//
//     /* Create the '\Driver' object directory */
//     RtlInitUnicodeString(&DirName, L"\\Driver");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirName,
//                                OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create \\Driver directory: 0x%lx\n", Status);
//         return FALSE;
//     }
//     NtClose(Handle);
//
//     /* Create the '\FileSystem' object directory */
//     RtlInitUnicodeString(&DirName, L"\\FileSystem");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirName,
//                                OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create \\FileSystem directory: 0x%lx\n", Status);
//         return FALSE;
//     }
//     NtClose(Handle);
//
//     /* Create the '\FileSystem' object directory */
//     RtlInitUnicodeString(&DirName, L"\\FileSystem\\Filters");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirName,
//                                OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create \\FileSystem\\Filters directory: 0x%lx\n", Status);
//         return FALSE;
//     }
//     NtClose(Handle);
//
//     /* Return success */
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// IopMarkBootPartition(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     STRING DeviceString;
//     CHAR Buffer[256];
//     UNICODE_STRING DeviceName;
//     NTSTATUS Status;
//     HANDLE FileHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PFILE_OBJECT FileObject;
//
//     /* Build the ARC device name */
//     sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
//     RtlInitAnsiString(&DeviceString, Buffer);
//     Status = RtlAnsiStringToUnicodeString(&DeviceName, &DeviceString, TRUE);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Open it */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DeviceName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenFile(&FileHandle,
//                         FILE_READ_ATTRIBUTES,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         0,
//                         FILE_NON_DIRECTORY_FILE);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         /* On ARM64 UEFI boot without disk drivers, boot device may not exist.
//          * Return FALSE instead of bugchecking to allow boot to continue. */
//         RtlFreeUnicodeString(&DeviceName);
//         return FALSE;
// #else
//         /* MININT/WinPE boots may lack a real boot device (e.g. LiveCD when the
//          * ATAPI driver cannot probe the CD-ROM).  Return FALSE to let the
//          * caller decide whether to continue. */
//         if (InitIsWinPEMode)
//         {
//             RtlFreeUnicodeString(&DeviceName);
//             return FALSE;
//         }
//         /* Fail */
//         KeBugCheckEx(INACCESSIBLE_BOOT_DEVICE,
//                      (ULONG_PTR)&DeviceName,
//                      Status,
//                      0,
//                      0);
// #endif
//     }
//
//     /* Get the DO */
//     Status = ObReferenceObjectByHandle(FileHandle,
//                                        0,
//                                        IoFileObjectType,
//                                        KernelMode,
//                                        (PVOID *)&FileObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         RtlFreeUnicodeString(&DeviceName);
//         return FALSE;
//     }
//
//     /* Mark it as the boot partition */
//     FileObject->DeviceObject->Flags |= DO_SYSTEM_BOOT_PARTITION;
//
//     /* Save a copy of the DO for the I/O Error Logger */
//     ObReferenceObject(FileObject->DeviceObject);
//     IopErrorLogObject = FileObject->DeviceObject;
//
//     /* Cleanup and return success */
//     RtlFreeUnicodeString(&DeviceName);
//     NtClose(FileHandle);
//     ObDereferenceObject(FileObject);
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// IoInitSystem(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     LARGE_INTEGER ExpireTime;
//     NTSTATUS Status;
//     CHAR Buffer[256];
//     ANSI_STRING NtBootPath, RootString;
//
//     /* Initialize empty NT Boot Path */
//     RtlInitEmptyAnsiString(&NtBootPath, Buffer, sizeof(Buffer));
//
//     /* Initialize the lookaside lists */
//     ARM64_UART_TRACE("[IO] IopInitLookasideLists\r\n");
//     IopInitLookasideLists();
//
//     /* Initialize all locks and lists */
//     ExInitializeResourceLite(&IopDatabaseResource);
//     ExInitializeResourceLite(&IopSecurityResource);
//     ExInitializeResourceLite(&IopDriverLoadResource);
//     InitializeListHead(&IopDiskFileSystemQueueHead);
//     InitializeListHead(&IopCdRomFileSystemQueueHead);
//     InitializeListHead(&IopTapeFileSystemQueueHead);
//     InitializeListHead(&IopNetworkFileSystemQueueHead);
//     InitializeListHead(&DriverBootReinitListHead);
//     InitializeListHead(&DriverReinitListHead);
//     InitializeListHead(&ShutdownListHead);
//     InitializeListHead(&LastChanceShutdownListHead);
//     InitializeListHead(&IopFsNotifyChangeQueueHead);
//     InitializeListHead(&IopErrorLogListHead);
//     KeInitializeSpinLock(&IoStatisticsLock);
//     KeInitializeSpinLock(&DriverReinitListLock);
//     KeInitializeSpinLock(&DriverBootReinitListLock);
//     KeInitializeSpinLock(&ShutdownListLock);
//     KeInitializeSpinLock(&IopLogListLock);
//
//     /* Initialize PnP notifications */
//     PiInitializeNotifications();
//
//     /* Initialize the reserve IRP */
//     if (!IopInitializeReserveIrp(&IopReserveIrpAllocator))
//     {
//         DPRINT1("IopInitializeReserveIrp failed!\n");
//         return FALSE;
//     }
//
//     /* Initialize Timer List Lock */
//     KeInitializeSpinLock(&IopTimerLock);
//
//     /* Initialize Timer List */
//     InitializeListHead(&IopTimerQueueHead);
//
//     /* Initialize the DPC/Timer which will call the other Timer Routines */
//     ExpireTime.QuadPart = -10000000;
//     KeInitializeDpc(&IopTimerDpc, IopTimerDispatch, NULL);
//     KeInitializeTimerEx(&IopTimer, SynchronizationTimer);
//     KeSetTimerEx(&IopTimer, ExpireTime, 1000, &IopTimerDpc);
//
//     /* Create Object Types */
//     ARM64_UART_TRACE("[IO] IopCreateObjectTypes\r\n");
//     if (!IopCreateObjectTypes())
//     {
//         DPRINT1("IopCreateObjectTypes failed!\n");
//         return FALSE;
//     }
//
//     /* Create Object Directories */
//     ARM64_UART_TRACE("[IO] IopCreateRootDirectories\r\n");
//     if (!IopCreateRootDirectories())
//     {
//         DPRINT1("IopCreateRootDirectories failed!\n");
//         return FALSE;
//     }
//
//     /* Initialize PnP manager */
//     ARM64_UART_TRACE("[IO] IopInitializePlugPlayServices\r\n");
//     IopInitializePlugPlayServices();
//
//     /* Initialize SHIM engine */
//     ApphelpCacheInitialize();
//
//     /* Initialize WMI */
//     WmiInitialize();
//
//     /* Initialize HAL Root Bus Driver */
//     ARM64_UART_TRACE("[IO] HalInitPnpDriver\r\n");
//     HalInitPnpDriver();
//
//     /* Reenumerate what HAL has added (synchronously)
//      * This function call should eventually become a 2nd stage of the PnP initialization */
//     ARM64_UART_TRACE("[IO] PiQueueDeviceAction\r\n");
//     PiQueueDeviceAction(IopRootDeviceNode->PhysicalDeviceObject,
//                         PiActionEnumRootDevices,
//                         NULL,
//                         NULL);
//
//     /* Make loader block available for the whole kernel */
//     IopLoaderBlock = LoaderBlock;
//
//     /* Load boot start drivers */
//     ARM64_UART_TRACE("[IO] IopInitializeBootDrivers\r\n");
//     IopInitializeBootDrivers();
//
//     /* Call back drivers that asked for */
//     ARM64_UART_TRACE("[IO] IopReinitializeBootDrivers\r\n");
//     IopReinitializeBootDrivers();
//
//     /* Check if this was a ramdisk boot */
//     ARM64_UART_TRACE("[IO] Checking ArcBootDeviceName for ramdisk...\r\n");
//     if (!_strnicmp(LoaderBlock->ArcBootDeviceName, "ramdisk(0)", 10))
//     {
//         /* Initialize the ramdisk driver */
//         ARM64_UART_TRACE("[IO] IopStartRamdisk\r\n");
//         IopStartRamdisk(LoaderBlock);
//         ARM64_UART_TRACE("[IO] IopStartRamdisk done\r\n");
//
// #ifdef _M_ARM64
//         /* Test: try to open a file on the ramdisk to verify filesystem access */
//         {
//             OBJECT_ATTRIBUTES TestOa;
//             IO_STATUS_BLOCK TestIosb;
//             HANDLE TestHandle;
//             UNICODE_STRING TestPath;
//             NTSTATUS TestStatus;
//
//             ARM64_UART_TRACE("[IO] Testing ramdisk filesystem access...\r\n");
//             RtlInitUnicodeString(&TestPath, L"\\ArcName\\ramdisk(0)\\reactos\\system32\\ntdll.dll");
//             InitializeObjectAttributes(&TestOa, &TestPath, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);
//             ARM64_UART_TRACE("[IO] ZwOpenFile(ntdll.dll) start\r\n");
//             TestStatus = ZwOpenFile(&TestHandle, FILE_READ_ACCESS, &TestOa, &TestIosb, FILE_SHARE_READ, 0);
//             if (NT_SUCCESS(TestStatus))
//             {
//                 ARM64_UART_TRACE("[IO] ZwOpenFile(ntdll.dll) SUCCESS!\r\n");
//                 ZwClose(TestHandle);
//             }
//             else
//             {
//                 volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//                 char _buf[64];
//                 const char *_p;
//                 int _len = sprintf(_buf, "[IO] ZwOpenFile(ntdll.dll) failed: 0x%lx\r\n", (unsigned long)TestStatus);
//                 _p = _buf;
//                 while (*_p) { *_u = (ULONG)*_p++; }
//             }
//         }
// #endif
//     }
//     else
//     {
//         ARM64_UART_TRACE("[IO] Not a ramdisk boot\r\n");
//     }
//
//     /* No one should need loader block any longer */
//     IopLoaderBlock = NULL;
//
//     /* Create ARC names for boot devices */
//     ARM64_UART_TRACE("[IO] IopCreateArcNames\r\n");
//     Status = IopCreateArcNames(LoaderBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopCreateArcNames failed: %lx\n", Status);
// #if !defined(_M_ARM64)
//         /* MININT/WinPE boots may lack a real boot disk (e.g. LiveCD when the
//          * ATAPI driver cannot probe the CD-ROM device).  Allow boot to continue
//          * so that ramdisk or other drivers can still provide the volume. */
//         if (!InitIsWinPEMode)
//             return FALSE;
// #endif
//     }
//
//     /* Mark the system boot partition */
//     ARM64_UART_TRACE("[IO] IopMarkBootPartition\r\n");
//     if (!IopMarkBootPartition(LoaderBlock))
//     {
//         ARM64_UART_TRACE("[IO] IopMarkBootPartition failed (continuing on ARM64/MININT)\r\n");
// #if !defined(_M_ARM64)
//         if (!InitIsWinPEMode)
//             return FALSE;
// #endif
//     }
//     else
//     {
//         ARM64_UART_TRACE("[IO] IopMarkBootPartition done\r\n");
//     }
//
//     /* The disk subsystem is initialized here and the SystemRoot is set too.
//      * We can finally load other drivers from the boot volume. */
//     PnPBootDriversInitialized = TRUE;
//
//     /* Load system start drivers */
//     ARM64_UART_TRACE("[IO] IopInitializeSystemDrivers\r\n");
//     IopInitializeSystemDrivers();
//     PnpSystemInit = TRUE;
//     ARM64_UART_TRACE("[IO] IopInitializeSystemDrivers done\r\n");
//
//     /* Reinitialize drivers that requested it */
//     IopReinitializeDrivers();
//
//     /* Convert SystemRoot from ARC to NT path */
//     ARM64_UART_TRACE("[IO] IopReassignSystemRoot\r\n");
//     Status = IopReassignSystemRoot(LoaderBlock, &NtBootPath);
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         ARM64_UART_TRACE("[IO] IopReassignSystemRoot failed (continuing on ARM64)\r\n");
//         return FALSE;
// #else
//         ARM64_UART_TRACE("[IO] IopReassignSystemRoot failed\r\n");
//         return FALSE;
// #endif
//     }
//     ARM64_UART_TRACE("[IO] IopReassignSystemRoot done\r\n");
//
//     /* Set the ANSI_STRING for the root path */
//     RootString.MaximumLength = NtSystemRoot.MaximumLength / sizeof(WCHAR);
//     RootString.Length = 0;
//     RootString.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                               RootString.MaximumLength,
//                                               TAG_IO);
//
//     /* Convert the path into the ANSI_STRING */
//     Status = RtlUnicodeStringToAnsiString(&RootString, &NtSystemRoot, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlUnicodeStringToAnsiString failed: %lx\n", Status);
//         return FALSE;
//     }
//
//     /* Assign drive letters */
//     ARM64_UART_TRACE("[IO] IoAssignDriveLetters\r\n");
//     IoAssignDriveLetters(LoaderBlock,
//                          &NtBootPath,
//                          (PUCHAR)RootString.Buffer,
//                          &RootString);
//     ARM64_UART_TRACE("[IO] IoAssignDriveLetters done\r\n");
//
//     /* Update system root */
//     Status = RtlAnsiStringToUnicodeString(&NtSystemRoot, &RootString, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlAnsiStringToUnicodeString failed: %lx\n", Status);
//         return FALSE;
//     }
//
//     /* Load the System DLL and its entrypoints */
//     ARM64_UART_TRACE("[IO] PsLocateSystemDll\r\n");
//     Status = PsLocateSystemDll();
//     if (!NT_SUCCESS(Status))
//     {
// #ifdef _M_ARM64
//         /* On ARM64, if ntdll.dll can't be loaded (no filesystem),
//          * continue without it — Phase 1 will enter idle instead of SMSS. */
//         ARM64_UART_TRACE("[IO] PsLocateSystemDll failed (continuing on ARM64)\r\n");
//         DPRINT1("PsLocateSystemDll failed: %lx (ARM64, continuing)\n", Status);
// #else
//         DPRINT1("PsLocateSystemDll failed: %lx\n", Status);
//         return FALSE;
// #endif
//     }
//
//     /* Return success */
//     ARM64_UART_TRACE("[IO] IoInitSystem returning TRUE\r\n");
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// IoInitializeCrashDump(IN HANDLE PageFileHandle)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
