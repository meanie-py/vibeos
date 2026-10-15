use ntkrnl_sys::globals::*;
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
use ntkrnl_sys::inline::common::ProbeAndCaptureUnicodeString;
use ntkrnl_sys::inline::common::ReleaseCapturedUnicodeString;

use ntkrnl_sys::ddk::{
    _BOOT_DRIVER_LIST_ENTRY,
    _DEVICE_NODE,
    _DRIVER_EXTENSION,
    _DRIVER_INFORMATION,
    _DRIVER_OBJECT,
    CSHORT,
    _DRIVER_REINIT_ITEM,
    _ERESOURCE,
    _EXTENDED_DEVOBJ_EXTENSION,
    _EXTENDED_DRIVER_EXTENSION,
    _IO_CLIENT_EXTENSION,
    _KEVENT,
    _KPRCB,
    _LDR_DATA_TABLE_ENTRY,
    _LIST_ENTRY,
    _KEY_BASIC_INFORMATION,
    _KEY_VALUE_FULL_INFORMATION,
    _UNICODE_STRING,
    _OBJECT_ATTRIBUTES,
    _LOADER_PARAMETER_BLOCK,
    _WORK_QUEUE_ITEM,
    BOOT_DRIVER_LIST_ENTRY,
    DEVICE_OBJECT,
    DRIVER_INFORMATION,
    DRIVER_OBJECT,
    DRIVER_REINIT_ITEM,
    ERESOURCE,
    EXTENDED_DEVOBJ_EXTENSION,
    EXTENDED_DRIVER_EXTENSION,
    IO_CLIENT_EXTENSION,
    KEVENT,
    LDR_DATA_TABLE_ENTRY,
    PBOOT_DRIVER_LIST_ENTRY,
    PDRIVER_INFORMATION,
    PDRIVER_OBJECT,
    PDRIVER_REINIT_ITEM,
    PERESOURCE,
    PEXTENDED_DEVOBJ_EXTENSION,
    PEXTENDED_DRIVER_EXTENSION,
    PIO_CLIENT_EXTENSION,
    PIRP,
    PKEY_NAME_INFORMATION,
    PLDR_DATA_TABLE_ENTRY,
    PLIST_ENTRY,
    POBJECT_ATTRIBUTES,
    POBJECT_TYPE,
    PUNICODE_STRING,
    PWORK_QUEUE_ITEM,
    ACCESS_MASK,
    HANDLE,
    KPROCESSOR_MODE,
    PEPROCESS,
    LUID,
    OB_OPEN_REASON,
    RTL_QUERY_REGISTRY_TABLE,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    _KWAIT_REASON_UserRequest,
    CmGetSystemDriverList,
    ExAcquireResourceExclusiveLite,
    ExFreePool,
    ExGetPreviousMode,
    ExQueueWorkItem,
    ExReleaseResourceLite,
    HalDisplayString,
    InbvIndicateProgress,
    IopGetRegistryValue,
    IopOpenRegistryKeyEx,
    KeEnterCriticalRegion,
    KeInitializeEvent,
    KeLeaveCriticalRegion,
    KeRaiseIrqlToDpcLevel,
    KeWaitForSingleObject,
    MmCallDllInitialize,
    MmFreeDriverInitialization,
    MmLoadSystemImage,
    MmUnloadSystemImage,
    ObCreateObject,
    ObInsertObject,
    ObMakeTemporaryObject,
    ObReferenceObjectByHandle,
    ObReferenceObjectByName,
    RtlAppendUnicodeStringToString,
    RtlAppendUnicodeToString,
    RtlCopyUnicodeString,
    RtlCreateUnicodeString,
    RtlEqualUnicodeString,
    RtlFindCharInUnicodeString,
    RtlFreeUnicodeString,
    RtlImageNtHeader,
    RtlInitUnicodeString,
    RtlQueryRegistryValues,
    RtlUpcaseUnicodeChar,
    SeSinglePrivilegeCheck,
    ZwClose,
    ZwLoadDriver,
    ZwQueryKey,
};

use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::inline::common::InitializeObjectAttributes;

// ExInterlockedInsertTailList / ExInterlockedRemoveHeadList: in bindgen on amd64/arm64 only
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::{ExInterlockedInsertTailList, ExInterlockedRemoveHeadList};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedInsertTailList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedInsertTailList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedInsertTailList(ListHead, ListEntry, Lock)
}

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedRemoveHeadList(
    ListHead: PLIST_ENTRY,
    Lock: *mut KSPIN_LOCK,
) -> PLIST_ENTRY {
    extern "fastcall" {
        fn ExfInterlockedRemoveHeadList(
            ListHead: PLIST_ENTRY,
            Lock: *mut KSPIN_LOCK,
        ) -> PLIST_ENTRY;
    }
    ExfInterlockedRemoveHeadList(ListHead, Lock)
}

extern "system" {
    fn PipGetDriverTagPriority(ServiceHandle: HANDLE) -> ULONG;
    fn PpInitGetGroupOrderIndex(ServiceHandle: HANDLE) -> USHORT;
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn MiResolveImageReferences(ImageBase: PVOID, ImageFileDirectory: PUNICODE_STRING, NamePrefix: PUNICODE_STRING, MissingApi: *mut PCHAR, MissingDriver: *mut PWCHAR, LoadImports: *mut PVOID) -> NTSTATUS;
}
extern "system" {
    fn KeLowerIrql(NewIrql: KIRQL);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}

unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: i8) {
    IofCompleteRequest(Irp, PriorityBoost);
}

extern "C" {
    fn IopGetDeviceObjectFromDeviceInstance(DeviceInstance: PUNICODE_STRING) -> *mut DEVICE_OBJECT;
    fn PiPerformSyncDeviceAction(DeviceObject: *mut DEVICE_OBJECT, Action: ULONG);
    fn PiQueueDeviceAction(DeviceObject: *mut DEVICE_OBJECT, Action: ULONG, Callback: PVOID, Context: PVOID);
    fn _snprintf(buf: PCHAR, count: usize, format: PCHAR, ...) -> i32;
    fn _snwprintf(buf: PWCHAR, count: usize, format: *const u16, ...) -> i32;
    fn ExInitializeWorkItem(Item: *mut _WORK_QUEUE_ITEM, Routine: Option<unsafe extern "system" fn(PVOID)>, Context: PVOID);
    fn CONTAINING_RECORD(ptr: PVOID, ty: PVOID, field: PVOID) -> PVOID;
}
extern "system" {
    // IopReadyDeviceObjects is defined in iomgr/device.rs as `extern "system"`
    // (NTAPI/stdcall on i386). Declaring it `extern "C"` linked silently on
    // amd64+arm64 (where NTAPI=cdecl) but produced a stdcall callee + cdecl
    // call-site mismatch on i386 — LLVM detected the ABI UB and folded the
    // post-call instructions to ud2.
    fn IopReadyDeviceObjects(DriverObject: PDRIVER_OBJECT);
}

extern "system" {
    fn RawFsDriverEntry(DriverObject: PDRIVER_OBJECT, RegistryPath: PUNICODE_STRING) -> NTSTATUS;
}

// Constants
const TAG_IO: ULONG = 0x20206F49;
const TAG_REINIT: ULONG = 0x69526F49;
const TAG_DRIVER_EXTENSION: ULONG = 0x45565244;
const TAG_LDR_WSTR: ULONG = u32::from_be_bytes(*b"swlk"); // C literal 'swlk' -- ntoskrnl/include/internal/tag.h
const TAG_RTLREGISTRY: ULONG = 0x76727152;
const UNICODE_STRING_MAX_BYTES: ULONG = 65534;
const STATUS_ILL_FORMED_SERVICE_ENTRY: NTSTATUS = 0xC0000160_u32 as i32;
const STATUS_FAILED_DRIVER_ENTRY: NTSTATUS = 0xC0000365_u32 as i32;
const SERVICE_RECOGNIZER_DRIVER: ULONG = 8;
const SERVICE_FILE_SYSTEM_DRIVER: ULONG = 2;
const DRVO_UNLOAD_INVOKED: ULONG = 0x00000001;
const DRVO_LEGACY_DRIVER: ULONG = 0x00000002;
const DRVO_BUILTIN_DRIVER: ULONG = 0x00000004;
const DRVO_REINIT_REGISTERED: ULONG = 0x00000008;
const DRVO_INITIALIZED: ULONG = 0x00000010;
const DRVO_BOOTREINIT_REGISTERED: ULONG = 0x00000020;
const DRVO_LEGACY_RESOURCES: ULONG = 0x00000040;
const DOE_UNLOAD_PENDING: ULONG = 0x00000001;
const IO_TYPE_DRIVER: USHORT = 4;
const LDRP_DRIVER_DEPENDENT_DLL: ULONG = 0x04000000;
const RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END: ULONG = 1;
const RTL_QUERY_REGISTRY_DIRECT: ULONG = 0x00000020;
const RTL_REGISTRY_ABSOLUTE: ULONG = 0;
const DelayedWorkQueue: u32 = 1;
const PiActionEnumDeviceTree: ULONG = 0;
const PiActionEnumRootDevices: ULONG = 1;
const PiActionAddBootDevices: ULONG = 3;
const MAXIMUM_FILENAME_LENGTH: usize = 256;
// mm.h: ((PVOID)(ULONG_PTR)-2). This was 0xFFFFFFFF -- a 32-bit -1, i.e. neither sentinel
// on 64-bit -- so an image whose imports were never resolved kept a LoadedImports the unload
// path does not recognise as a sentinel and would walk as an import list.
const MM_SYSLDR_NO_IMPORTS: usize = (-2isize) as usize;
const KeyBasicInformation: ULONG = 0;

static FILESYSTEM_ROOT_NAME: &[u16] = &[
    b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
    b'm' as u16, b'\\' as u16, 0,
];

static DRIVER_ROOT_NAME: &[u16] = &[
    b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16,
    b'e' as u16, b'r' as u16, b'\\' as u16, 0,
];

// UTF-16 string constants for registry value names and paths
// (replacing incorrect b"...".as_ptr() as *mut u16 casts)
static OBJECT_NAME_STR: [u16; 11] = [
    b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16,
    b't' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0,
];
static TYPE_STR: [u16; 5] = [
    b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, 0,
];
static DOT_SYS_UPPER_STR: [u16; 5] = [
    b'.' as u16, b'S' as u16, b'Y' as u16, b'S' as u16, 0,
];
static DOT_SYS_LOWER_STR: [u16; 5] = [
    b'.' as u16, b's' as u16, b'y' as u16, b's' as u16, 0,
];
static SYSTEM_ROOT_STR: [u16; 13] = [
    b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16,
    b'e' as u16, b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16,
    b't' as u16, b'\\' as u16, 0,
];
static DRIVERS_PATH_STR: [u16; 30] = [
    b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16,
    b'e' as u16, b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16,
    b't' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16,
    b't' as u16, b'e' as u16, b'm' as u16, b'3' as u16, b'2' as u16,
    b'\\' as u16, b'd' as u16, b'r' as u16, b'i' as u16, b'v' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'\\' as u16, 0,
];
static ENUM_STR: [u16; 5] = [
    b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, 0,
];
static COUNT_STR: [u16; 6] = [
    b'C' as u16, b'o' as u16, b'u' as u16, b'n' as u16, b't' as u16, 0,
];
static PERCENT_U_STR: [u16; 3] = [
    b'%' as u16, b'u' as u16, 0,
];
static BACKSLASH_STR: [u16; 2] = [
    b'\\' as u16, 0,
];
static IMAGE_PATH_STR: [u16; 10] = [
    b'I' as u16, b'm' as u16, b'a' as u16, b'g' as u16, b'e' as u16,
    b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0,
];
static DRIVER_FMT_STR: [u16; 13] = [
    b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16,
    b'e' as u16, b'r' as u16, b'\\' as u16, b'%' as u16, b'0' as u16,
    b'8' as u16, b'u' as u16, 0,
];

#[cfg(target_arch = "aarch64")]
extern "system" {
    static mut Arm64UartKernelBase: PVOID;
    fn Arm64UartPutChar(c: u8);
}

#[cfg(target_arch = "aarch64")]
macro_rules! ARM64_UART_TRACE {
    ($s:expr) => {
        unsafe {
            let uart = Arm64UartKernelBase as *mut ULONG;
            if !uart.is_null() {
                let bytes = $s.as_bytes();
                for &b in bytes {
                    Arm64UartPutChar(b as u8);
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
pub static mut IopDriverLoadResource: ERESOURCE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut DriverReinitListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut DriverReinitListLock: KSPIN_LOCK = 0;
static mut DriverReinitTailEntry: PLIST_ENTRY = core::ptr::null_mut();

static mut DriverBootReinitTailEntry: PLIST_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut DriverBootReinitListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut DriverBootReinitListLock: KSPIN_LOCK = 0;

// L"\\REGISTRY\\MACHINE\\HARDWARE\\DESCRIPTION\\SYSTEM"
static mut IOP_HARDWARE_DATABASE_KEY_BUF: [u16; 46] = [
    b'\\' as u16, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16,
    b'T' as u16, b'R' as u16, b'Y' as u16, b'\\' as u16, b'M' as u16, b'A' as u16,
    b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16,
    b'H' as u16, b'A' as u16, b'R' as u16, b'D' as u16, b'W' as u16, b'A' as u16,
    b'R' as u16, b'E' as u16, b'\\' as u16, b'D' as u16, b'E' as u16, b'S' as u16,
    b'C' as u16, b'R' as u16, b'I' as u16, b'P' as u16, b'T' as u16, b'I' as u16,
    b'O' as u16, b'N' as u16, b'\\' as u16, b'S' as u16, b'Y' as u16, b'S' as u16,
    b'T' as u16, b'E' as u16, b'M' as u16, 0,
];
#[no_mangle]
pub static mut IopHardwareDatabaseKey: _UNICODE_STRING = unsafe {
    _UNICODE_STRING {
        Length: 90, // 45 chars * 2
        MaximumLength: 92, // 46 chars * 2
        Buffer: (*(&raw const IOP_HARDWARE_DATABASE_KEY_BUF)).as_ptr() as *mut u16,
    }
};

static SERVICES_KEY_NAME: &[u16] = &[
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
    b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
    b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
    b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
    b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
    b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
    b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16,
    b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16, 0,
];

#[no_mangle]
pub static mut IoDriverObjectType: POBJECT_TYPE = core::ptr::null_mut();

extern "system" {
    static mut PnpSystemInit: BOOLEAN;
    static mut PnPBootDriversLoaded: BOOLEAN;
    static mut PiEnumerationFinished: KEVENT;
    static mut SosEnabled: BOOLEAN;
    static mut KeLoaderBlock: *mut core::ffi::c_void;
    static mut PsLoadedModuleList: LIST_ENTRY;
    static mut IopRootDeviceNode: *mut core::ffi::c_void;
    static mut KiProcessorBlock: [*mut _KPRCB; 0];
    static SeLoadDriverPrivilege: LUID;
    static KeTickCount: ULONG;
}

static mut IopGroupIndex: USHORT = 0;
static mut IopGroupTable: PLIST_ENTRY = core::ptr::null_mut();

struct LOAD_UNLOAD_PARAMS {
    Status: NTSTATUS,
    RegistryPath: PUNICODE_STRING,
    WorkItem: _WORK_QUEUE_ITEM,
    Event: KEVENT,
    DriverObject: PDRIVER_OBJECT,
    SetEvent: BOOLEAN,
}

#[no_mangle]
pub unsafe extern "system" fn IopInvalidDeviceRequest(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_INVALID_DEVICE_REQUEST
}

#[no_mangle]
pub unsafe extern "system" fn IopDeleteDriver(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    let DriverObject = ObjectBody as PDRIVER_OBJECT;
    let mut DriverExtension: PIO_CLIENT_EXTENSION;
    let mut NextDriverExtension: PIO_CLIENT_EXTENSION;

    DriverExtension = (*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ClientDriverExtension;
    while !DriverExtension.is_null() {
        NextDriverExtension = (*DriverExtension).NextExtension;
        ExFreePoolWithTag(DriverExtension as PVOID, TAG_DRIVER_EXTENSION);
        DriverExtension = NextDriverExtension;
    }

    if !(*DriverObject).DriverSection.is_null() {
        MmUnloadSystemImage((*DriverObject).DriverSection as PVOID);
    }

    if !(*DriverObject).DriverName.Buffer.is_null() {
        ExFreePool((*DriverObject).DriverName.Buffer as PVOID);
    }

    if !(*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ServiceKeyName.Buffer.is_null() {
        ExFreePool((*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ServiceKeyName.Buffer as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IopGetDriverNames(
    ServiceHandle: HANDLE,
    DriverName: *mut _UNICODE_STRING,
    ServiceName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let ServiceHandle = opaque_ptr(ServiceHandle);
    let DriverName = opaque_ptr(DriverName);
    let ServiceName = opaque_ptr(ServiceName);
    let mut driverName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut serviceName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut kvInfo: *mut _KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut status: NTSTATUS;

    status = IopGetRegistryValue(ServiceHandle, OBJECT_NAME_STR.as_ptr() as *mut u16, &mut kvInfo);
    if status >= 0 {
        if (*kvInfo).Type != REG_SZ
            || (*kvInfo).DataLength < core::mem::size_of::<u16>() as ULONG
            || (*kvInfo).DataLength > UNICODE_STRING_MAX_BYTES as ULONG
            || ((*kvInfo).DataLength % core::mem::size_of::<u16>() as ULONG) != 0
        {
            ExFreePool(kvInfo as PVOID);
            return STATUS_ILL_FORMED_SERVICE_ENTRY;
        }

        driverName.Length = ((*kvInfo).DataLength - core::mem::size_of::<u16>() as ULONG) as USHORT;
        driverName.MaximumLength = (*kvInfo).DataLength as USHORT;
        driverName.Buffer = ExAllocatePoolWithTag(NonPagedPool, driverName.MaximumLength as usize, TAG_IO) as *mut u16;
        if driverName.Buffer.is_null() {
            ExFreePool(kvInfo as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            ((kvInfo as ULONG_PTR) + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
            driverName.Buffer as *mut u8,
            driverName.Length as usize,
        );
        *driverName.Buffer.add(driverName.Length as usize / core::mem::size_of::<u16>()) = 0;
        ExFreePool(kvInfo as PVOID);
    }

    let mut basicInfo: *mut _KEY_BASIC_INFORMATION = core::ptr::null_mut();
    if status < 0 || !ServiceName.is_null() {
        let mut infoLength: ULONG = 0;
        status = ZwQueryKey(ServiceHandle, KeyBasicInformation, core::ptr::null_mut(), 0, &mut infoLength);
        if status != STATUS_BUFFER_TOO_SMALL {
            status = if status >= 0 { STATUS_UNSUCCESSFUL } else { status };
            if !driverName.Buffer.is_null() {
                ExFreePoolWithTag(driverName.Buffer as PVOID, TAG_IO);
            }
            return status;
        }

        basicInfo = ExAllocatePoolWithTag(PagedPool, infoLength as usize, TAG_IO) as *mut _KEY_BASIC_INFORMATION;
        if basicInfo.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            if !driverName.Buffer.is_null() {
                ExFreePoolWithTag(driverName.Buffer as PVOID, TAG_IO);
            }
            return status;
        }

        status = ZwQueryKey(ServiceHandle, KeyBasicInformation, basicInfo as PVOID, infoLength, &mut infoLength);
        if status < 0 {
            ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
            if !driverName.Buffer.is_null() {
                ExFreePoolWithTag(driverName.Buffer as PVOID, TAG_IO);
            }
            return status;
        }

        serviceName.Length = (*basicInfo).NameLength as USHORT;
        serviceName.MaximumLength = (*basicInfo).NameLength as USHORT;
        serviceName.Buffer = (*basicInfo).Name.as_mut_ptr();
    }

    if driverName.Buffer.is_null() {
        let mut driverType: ULONG = 0;
        status = IopGetRegistryValue(ServiceHandle, TYPE_STR.as_ptr() as *mut u16, &mut kvInfo);
        if status < 0 {
            ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
            return status;
        }
        if (*kvInfo).Type != REG_DWORD || (*kvInfo).DataLength != core::mem::size_of::<ULONG>() as ULONG {
            ExFreePool(kvInfo as PVOID);
            ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
            return STATUS_ILL_FORMED_SERVICE_ENTRY;
        }

        core::ptr::copy_nonoverlapping(
            ((kvInfo as ULONG_PTR) + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
            &mut driverType as *mut ULONG as *mut u8,
            core::mem::size_of::<ULONG>(),
        );
        ExFreePool(kvInfo as PVOID);

        if driverType == SERVICE_RECOGNIZER_DRIVER || driverType == SERVICE_FILE_SYSTEM_DRIVER {
            driverName.MaximumLength = (FILESYSTEM_ROOT_NAME.len() * core::mem::size_of::<u16>() + serviceName.Length as usize) as USHORT;
        } else {
            driverName.MaximumLength = (DRIVER_ROOT_NAME.len() * core::mem::size_of::<u16>() + serviceName.Length as usize) as USHORT;
        }

        driverName.Length = 0;
        driverName.Buffer = ExAllocatePoolWithTag(NonPagedPool, driverName.MaximumLength as usize, TAG_IO) as *mut u16;
        if driverName.Buffer.is_null() {
            ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if driverType == SERVICE_RECOGNIZER_DRIVER || driverType == SERVICE_FILE_SYSTEM_DRIVER {
            RtlAppendUnicodeToString(&mut driverName as *mut _UNICODE_STRING, FILESYSTEM_ROOT_NAME.as_ptr());
        } else {
            RtlAppendUnicodeToString(&mut driverName as *mut _UNICODE_STRING, DRIVER_ROOT_NAME.as_ptr());
        }

        RtlAppendUnicodeStringToString(&mut driverName as *mut _UNICODE_STRING, &mut serviceName as *mut _UNICODE_STRING);
    }

    if !ServiceName.is_null() {
        let buf = ExAllocatePoolWithTag(PagedPool, serviceName.Length as usize, TAG_IO) as *mut u16;
        if buf.is_null() {
            ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
            if !driverName.Buffer.is_null() {
                ExFreePoolWithTag(driverName.Buffer as PVOID, TAG_IO);
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(serviceName.Buffer as *const u8, buf as *mut u8, serviceName.Length as usize);
        (*ServiceName).MaximumLength = serviceName.Length;
        (*ServiceName).Length = serviceName.Length;
        (*ServiceName).Buffer = buf;
    }

    *DriverName = driverName;
    status = STATUS_SUCCESS;

    if !basicInfo.is_null() {
        ExFreePoolWithTag(basicInfo as PVOID, TAG_IO);
    }

    status
}

unsafe fn IopSuffixUnicodeString(
    String1: *const _UNICODE_STRING,
    String2: *const _UNICODE_STRING,
    CaseInSensitive: BOOLEAN,
) -> BOOLEAN {
    if (*String2).Length < (*String1).Length {
        return 0u8;
    }

    let NumChars = (*String1).Length as usize / core::mem::size_of::<u16>();
    let pc1 = (*String1).Buffer;
    let pc2 = (*String2).Buffer.add((*String2).Length as usize / core::mem::size_of::<u16>() - NumChars);

    if !pc1.is_null() && !pc2.is_null() {
        if CaseInSensitive != 0 {
            for i in 0..NumChars {
                if RtlUpcaseUnicodeChar(*pc1.add(i)) != RtlUpcaseUnicodeChar(*pc2.add(i)) {
                    return 0u8;
                }
            }
        } else {
            for i in 0..NumChars {
                if *pc1.add(i) != *pc2.add(i) {
                    return 0u8;
                }
            }
        }
        return 1u8;
    }

    0u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IopDisplayLoadingMessage(ServiceName: *const _UNICODE_STRING) {
    static mut DOT_SYS: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOT_SYS_UPPER_STR.as_ptr() as *mut u16,
    };
    let mut TextBuffer: [u8; 256] = [0; 256];

    if SosEnabled == 0 {
        return;
    }
    if KeLoaderBlock.is_null() {
        return;
    }
    _snprintf(
        TextBuffer.as_mut_ptr() as PCHAR,
        core::mem::size_of_val(&TextBuffer),
        b"%s%sSystem32\\Drivers\\%wZ%s\r\n\0".as_ptr() as PCHAR,
        (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).ArcBootDeviceName,
        (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).NtBootPathName,
        ServiceName,
        if IopSuffixUnicodeString(&raw const DOT_SYS as *const _UNICODE_STRING, ServiceName, 1u8) != 0 {
            b"\0".as_ptr() as PCHAR
        } else {
            b".SYS\0".as_ptr() as PCHAR
        },
    );
    HalDisplayString(TextBuffer.as_ptr() as PCHAR);
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IopDisplayLoadingMessage(ServiceName: *const _UNICODE_STRING) {
    static mut DOT_SYS: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOT_SYS_UPPER_STR.as_ptr() as *mut u16,
    };
    let mut TextBuffer: [u8; 256] = [0; 256];

    if SosEnabled == 0 {
        return;
    }
    if KeLoaderBlock.is_null() {
        return;
    }
    _snprintf(
        TextBuffer.as_mut_ptr() as PCHAR,
        core::mem::size_of_val(&TextBuffer),
        b"%s%sSystem32\\Drivers\\%wZ%s\r\n\0".as_ptr() as PCHAR,
        (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).ArcBootDeviceName,
        (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).NtBootPathName,
        ServiceName,
        if IopSuffixUnicodeString(&raw const DOT_SYS as *const _UNICODE_STRING, ServiceName, 1u8) != 0 {
            b"\0".as_ptr() as PCHAR
        } else {
            b".SYS\0".as_ptr() as PCHAR
        },
    );
    HalDisplayString(TextBuffer.as_ptr() as PCHAR);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IopNormalizeImagePath(
    ImagePath: *mut _UNICODE_STRING,
    ServiceName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    static mut SYSTEM_ROOT_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: SYSTEM_ROOT_STR.as_ptr() as *mut u16,
    };
    static mut DRIVERS_PATH_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 58,
        MaximumLength: 60,
        Buffer: DRIVERS_PATH_STR.as_ptr() as *mut u16,
    };
    static mut DOT_SYS_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOT_SYS_LOWER_STR.as_ptr() as *mut u16,
    };

    let InputImagePath = *ImagePath;
    if InputImagePath.Length == 0 {
        (*ImagePath).Length = 0;
        (*ImagePath).MaximumLength = DRIVERS_PATH_STRING.Length
            + (*ServiceName).Length
            + DOT_SYS_STRING.Length
            + core::mem::size_of::<u16>() as USHORT;
        (*ImagePath).Buffer = ExAllocatePoolWithTag(NonPagedPool, (*ImagePath).MaximumLength as usize, TAG_IO) as *mut u16;
        if (*ImagePath).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyUnicodeString(ImagePath, &raw const DRIVERS_PATH_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
        RtlAppendUnicodeStringToString(ImagePath, ServiceName);
        RtlAppendUnicodeStringToString(ImagePath, &raw const DOT_SYS_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
    } else if *InputImagePath.Buffer != b'\\' as u16 {
        (*ImagePath).Length = 0;
        (*ImagePath).MaximumLength = SYSTEM_ROOT_STRING.Length + InputImagePath.Length + core::mem::size_of::<u16>() as USHORT;
        (*ImagePath).Buffer = ExAllocatePoolWithTag(NonPagedPool, (*ImagePath).MaximumLength as usize, TAG_IO) as *mut u16;
        if (*ImagePath).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyUnicodeString(ImagePath, &raw const SYSTEM_ROOT_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
        RtlAppendUnicodeStringToString(ImagePath, &InputImagePath as *const _UNICODE_STRING as *mut _UNICODE_STRING);

        ExFreePoolWithTag(InputImagePath.Buffer as PVOID, TAG_RTLREGISTRY);
    }

    STATUS_SUCCESS
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IopNormalizeImagePath(
    ImagePath: *mut _UNICODE_STRING,
    ServiceName: *mut _UNICODE_STRING,
) -> NTSTATUS {
    static mut SYSTEM_ROOT_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: SYSTEM_ROOT_STR.as_ptr() as *mut u16,
    };
    static mut DRIVERS_PATH_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 58,
        MaximumLength: 60,
        Buffer: DRIVERS_PATH_STR.as_ptr() as *mut u16,
    };
    static mut DOT_SYS_STRING: _UNICODE_STRING = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: DOT_SYS_LOWER_STR.as_ptr() as *mut u16,
    };

    let InputImagePath = *ImagePath;
    if InputImagePath.Length == 0 {
        (*ImagePath).Length = 0;
        (*ImagePath).MaximumLength = DRIVERS_PATH_STRING.Length
            + (*ServiceName).Length
            + DOT_SYS_STRING.Length
            + core::mem::size_of::<u16>() as USHORT;
        (*ImagePath).Buffer = ExAllocatePoolWithTag(NonPagedPool, (*ImagePath).MaximumLength as usize, TAG_IO) as *mut u16;
        if (*ImagePath).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyUnicodeString(ImagePath, &raw const DRIVERS_PATH_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
        RtlAppendUnicodeStringToString(ImagePath, ServiceName);
        RtlAppendUnicodeStringToString(ImagePath, &raw const DOT_SYS_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
    } else if *InputImagePath.Buffer != b'\\' as u16 {
        (*ImagePath).Length = 0;
        (*ImagePath).MaximumLength = SYSTEM_ROOT_STRING.Length + InputImagePath.Length + core::mem::size_of::<u16>() as USHORT;
        (*ImagePath).Buffer = ExAllocatePoolWithTag(NonPagedPool, (*ImagePath).MaximumLength as usize, TAG_IO) as *mut u16;
        if (*ImagePath).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyUnicodeString(ImagePath, &raw const SYSTEM_ROOT_STRING as *const _UNICODE_STRING as *mut _UNICODE_STRING);
        RtlAppendUnicodeStringToString(ImagePath, &InputImagePath as *const _UNICODE_STRING as *mut _UNICODE_STRING);

        ExFreePoolWithTag(InputImagePath.Buffer as PVOID, TAG_RTLREGISTRY);
    }

    STATUS_SUCCESS
}

unsafe extern "system" fn LdrProcessDriverModule(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    FileName: PUNICODE_STRING,
    ModuleObject: *mut PLDR_DATA_TABLE_ENTRY,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut BaseName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut BaseDirectory: _UNICODE_STRING;
    let mut LoadedImports: PVOID = MM_SYSLDR_NO_IMPORTS as PVOID;
    let mut MissingApiName: PCHAR;
    let mut MissingDriverName: PWCHAR = core::ptr::null_mut();
    let DriverBase = (*LdrEntry).DllBase;

    let Buffer = ExAllocatePoolWithTag(NonPagedPool, MAXIMUM_FILENAME_LENGTH, TAG_LDR_WSTR) as PCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if *(*FileName).Buffer == OBJ_NAME_PATH_SEPARATOR as u16 {
        let mut p = (*FileName).Buffer.add((*FileName).Length as usize / core::mem::size_of::<u16>());
        while *(p.sub(1)) != OBJ_NAME_PATH_SEPARATOR as u16 {
            p = p.sub(1);
        }

        let BaseLength = ((*FileName).Buffer.add((*FileName).Length as usize / core::mem::size_of::<u16>()).offset_from(p) as usize) * core::mem::size_of::<u16>();

        BaseName.Length = BaseLength as USHORT;
        BaseName.Buffer = p;
    } else {
        BaseName.Length = (*FileName).Length;
        BaseName.Buffer = (*FileName).Buffer;
    }

    BaseName.MaximumLength = BaseName.Length;

    BaseDirectory = *FileName;
    BaseDirectory.Length -= BaseName.Length;
    BaseDirectory.MaximumLength = BaseDirectory.Length;

    MissingApiName = Buffer;
    Status = MiResolveImageReferences(
        DriverBase,
        &mut BaseDirectory as *mut _UNICODE_STRING,
        core::ptr::null_mut(),
        &mut MissingApiName as *mut PCHAR,
        &mut MissingDriverName as *mut PWCHAR,
        &mut LoadedImports as *mut PVOID,
    );

    ExFreePoolWithTag(Buffer as PVOID, TAG_LDR_WSTR);

    if Status < 0 {
        return Status;
    }

    *ModuleObject = LdrEntry;
    STATUS_SUCCESS
}

unsafe fn IopInitializeBuiltinDriver(BootLdrEntry: PLDR_DATA_TABLE_ENTRY) -> BOOLEAN {
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PWCHAR;
    let mut FileNameWithoutPath: PWCHAR;
    let FileExtension: PWSTR;
    let ModuleName = &mut (*BootLdrEntry).BaseDllName as *mut _UNICODE_STRING;
    crate::dbg_print!(b"[BLT] IopInitializeBuiltinDriver: %wZ\r\n\0", ModuleName);
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut NextEntry: PLIST_ENTRY;
    let mut ServiceName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut Success: BOOLEAN;

    #[cfg(target_arch = "aarch64")]
    {
        let uart = Arm64UartKernelBase as *mut ULONG;
        if !uart.is_null() {
            let msg = b"[DRV] Loading ";
            for &b in msg {
                Arm64UartPutChar(b as u8);
            }
            for i in 0..((*ModuleName).Length / 2) as usize {
                Arm64UartPutChar((*(*ModuleName).Buffer.add(i) & 0x7F) as u8);
            }
            let nl = b"\r\n";
            for &b in nl {
                Arm64UartPutChar(b as u8);
            }
        }
    }

    IopDisplayLoadingMessage(ModuleName);
    InbvIndicateProgress();

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        ((*ModuleName).Length as usize + core::mem::size_of::<u16>()) as usize,
        TAG_IO,
    ) as PWCHAR;
    if Buffer.is_null() {
        return 0u8;
    }

    core::ptr::copy_nonoverlapping((*ModuleName).Buffer as *const u8, Buffer as *mut u8, (*ModuleName).Length as usize);
    *Buffer.add((*ModuleName).Length as usize / core::mem::size_of::<u16>()) = 0;

    FileNameWithoutPath = Buffer;
    let mut i = 0;
    while i < (*ModuleName).Length as usize / core::mem::size_of::<u16>() {
        if *Buffer.add(i) == b'\\' as u16 {
            FileNameWithoutPath = Buffer.add(i + 1);
        }
        i += 1;
    }

    Success = RtlCreateUnicodeString(&mut ServiceName as *mut _UNICODE_STRING, FileNameWithoutPath);
    ExFreePoolWithTag(Buffer as PVOID, TAG_IO);
    if Success == 0 {
        return 0u8;
    }

    let mut ext_pos = 0;
    for j in 0..(ServiceName.Length / 2) as usize {
        if *ServiceName.Buffer.add(j) == b'.' as u16 {
            ext_pos = j;
        }
    }
    if ext_pos > 0 {
        ServiceName.Length = (ext_pos * core::mem::size_of::<u16>()) as USHORT;
        *ServiceName.Buffer.add(ext_pos) = 0;
    }

    let mut RegistryPath = _UNICODE_STRING {
        Length: 0,
        // SERVICES_KEY_NAME is &[u16] — use .len() * 2 for byte size (not size_of_val which gives fat pointer size)
        MaximumLength: (SERVICES_KEY_NAME.len() * core::mem::size_of::<u16>() + ServiceName.Length as usize) as USHORT,
        Buffer: core::ptr::null_mut(),
    };
    RegistryPath.Buffer = ExAllocatePoolWithTag(PagedPool, RegistryPath.MaximumLength as usize, TAG_IO) as PWCHAR;
    if RegistryPath.Buffer.is_null() {
        RtlFreeUnicodeString(&mut ServiceName as *mut _UNICODE_STRING);
        return 0u8;
    }
    RtlAppendUnicodeToString(&mut RegistryPath as *mut _UNICODE_STRING, SERVICES_KEY_NAME.as_ptr());
    RtlAppendUnicodeStringToString(&mut RegistryPath as *mut _UNICODE_STRING, &mut ServiceName as *mut _UNICODE_STRING);
    RtlFreeUnicodeString(&mut ServiceName as *mut _UNICODE_STRING);

    let mut serviceHandle: HANDLE = core::ptr::null_mut();
    Status = IopOpenRegistryKeyEx(&mut serviceHandle as *mut HANDLE, core::ptr::null_mut(), &mut RegistryPath as *mut _UNICODE_STRING, KEY_READ);
    RtlFreeUnicodeString(&mut RegistryPath as *mut _UNICODE_STRING);
    if Status < 0 {
        return 0u8;
    }

    NextEntry = PsLoadedModuleList.Flink;
    while NextEntry != &raw mut PsLoadedModuleList as *mut LIST_ENTRY {
        LdrEntry = ((NextEntry as usize) - core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;
        if RtlEqualUnicodeString(ModuleName, &mut (*LdrEntry).BaseDllName as *mut _UNICODE_STRING, 1u8) != 0 {
            break;
        }
        NextEntry = (*NextEntry).Flink;
    }

    let mut driverEntryStatus: NTSTATUS = STATUS_SUCCESS;
    Status = IopInitializeDriverModule(LdrEntry, serviceHandle, &mut DriverObject as *mut PDRIVER_OBJECT, &mut driverEntryStatus as *mut NTSTATUS);

    if Status < 0 {
        ARM64_UART_TRACE!("[DRV] DriverEntry FAILED\r\n");
        ZwClose(serviceHandle);
        return 0u8;
    }
    ARM64_UART_TRACE!("[DRV] DriverEntry OK\r\n");

    let mut deviceAdded = if ((*DriverObject).Flags & DRVO_LEGACY_DRIVER) != 0 { 1u8 } else { 0u8 };

    let mut enumServiceHandle: HANDLE = core::ptr::null_mut();
    let enumName = _UNICODE_STRING {
        Length: 8,
        MaximumLength: 10,
        Buffer: ENUM_STR.as_ptr() as *mut u16,
    };

    Status = IopOpenRegistryKeyEx(&mut enumServiceHandle as *mut HANDLE, serviceHandle, &enumName as *const _UNICODE_STRING as *mut _UNICODE_STRING, KEY_READ);
    ZwClose(serviceHandle);

    if Status >= 0 {
        let mut instanceCount: ULONG = 0;
        let mut kvInfo: *mut _KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
        Status = IopGetRegistryValue(enumServiceHandle, COUNT_STR.as_ptr() as *mut u16, &mut kvInfo as *mut *mut _KEY_VALUE_FULL_INFORMATION);
        if Status < 0 {
            ZwClose(enumServiceHandle);
            ObDereferenceObject(DriverObject as PVOID);
            return deviceAdded;
        }
        if (*kvInfo).Type != REG_DWORD || (*kvInfo).DataLength != core::mem::size_of::<ULONG>() as ULONG {
            ExFreePool(kvInfo as PVOID);
            ZwClose(enumServiceHandle);
            ObDereferenceObject(DriverObject as PVOID);
            return deviceAdded;
        }

        core::ptr::copy_nonoverlapping(
            ((kvInfo as ULONG_PTR) + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
            &mut instanceCount as *mut ULONG as *mut u8,
            core::mem::size_of::<ULONG>(),
        );
        ExFreePool(kvInfo as PVOID);

        for i in 0..instanceCount {
            let mut num: [u16; 11] = [0; 11];
            let mut instancePath = _UNICODE_STRING {
                Buffer: core::ptr::null_mut(),
                Length: 0,
                MaximumLength: 0,
            };
            _snwprintf(num.as_mut_ptr(), num.len(), PERCENT_U_STR.as_ptr() as *const u16, i);

            Status = IopGetRegistryValue(enumServiceHandle, num.as_ptr() as *mut u16, &mut kvInfo as *mut *mut _KEY_VALUE_FULL_INFORMATION);
            if Status < 0 {
                continue;
            }
            if (*kvInfo).Type != REG_SZ
                || (*kvInfo).DataLength < core::mem::size_of::<u16>() as ULONG
                || (*kvInfo).DataLength > UNICODE_STRING_MAX_BYTES as ULONG
                || ((*kvInfo).DataLength % core::mem::size_of::<u16>() as ULONG) != 0
            {
                ExFreePool(kvInfo as PVOID);
                continue;
            }

            instancePath.Length = ((*kvInfo).DataLength - core::mem::size_of::<u16>() as ULONG) as USHORT;
            instancePath.MaximumLength = (*kvInfo).DataLength as USHORT;
            instancePath.Buffer = ExAllocatePoolWithTag(NonPagedPool, instancePath.MaximumLength as usize, TAG_IO) as *mut u16;
            if !instancePath.Buffer.is_null() {
                core::ptr::copy_nonoverlapping(
                    ((kvInfo as ULONG_PTR) + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
                    instancePath.Buffer as *mut u8,
                    instancePath.Length as usize,
                );
                *instancePath.Buffer.add(instancePath.Length as usize / core::mem::size_of::<u16>()) = 0;

                let pdo = IopGetDeviceObjectFromDeviceInstance(&mut instancePath as *mut _UNICODE_STRING);
                if !pdo.is_null() {
                    PiQueueDeviceAction(pdo, PiActionAddBootDevices, core::ptr::null_mut(), core::ptr::null_mut());
                    ObDereferenceObject(pdo as PVOID);
                    deviceAdded = 1u8;
                }
            }

            ExFreePool(kvInfo as PVOID);
        }

        ZwClose(enumServiceHandle);
    }

    ObDereferenceObject(DriverObject as PVOID);

    deviceAdded
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn IopInitializeBootDrivers() {
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut NextEntry2: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut DriverName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut i: ULONG;
    let mut Index: ULONG;
    let mut DriverInfo: PDRIVER_INFORMATION;
    let mut DriverInfoTag: PDRIVER_INFORMATION;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut BootEntry: PBOOT_DRIVER_LIST_ENTRY;

    // L"\\FileSystem\\RAW" — must be UTF-16, not ASCII bytes
    static RAW_DRIVER_NAME: [u16; 16] = [
        b'\\' as u16, b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
        b'm' as u16, b'\\' as u16, b'R' as u16, b'A' as u16, b'W' as u16, 0,
    ];
    RtlInitUnicodeString(&mut DriverName as *mut _UNICODE_STRING, RAW_DRIVER_NAME.as_ptr() as *mut u16);

    Status = IoCreateDriver(&mut DriverName as *mut _UNICODE_STRING, core::mem::transmute(Some(RawFsDriverEntry as unsafe extern "system" fn(PDRIVER_OBJECT, PUNICODE_STRING) -> NTSTATUS)));
    if Status < 0 {
        return;
    }

    IopGroupIndex = PpInitGetGroupOrderIndex(core::ptr::null_mut());
    if IopGroupIndex == 0xFFFF {
        return;
    }

    IopGroupTable = ExAllocatePoolWithTag(PagedPool, (IopGroupIndex as usize) * core::mem::size_of::<LIST_ENTRY>(), TAG_IO) as PLIST_ENTRY;
    if IopGroupTable.is_null() {
        return;
    }

    for j in 0..IopGroupIndex as usize {
        InitializeListHead(IopGroupTable.add(j));
    }

    ListHead = &mut (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).LoadOrderListHead as *mut LIST_ENTRY;
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        LdrEntry = ((NextEntry as usize) - core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;

        if ((*LdrEntry).Flags & LDRP_DRIVER_DEPENDENT_DLL) != 0 {
            MmCallDllInitialize(LdrEntry, core::ptr::null_mut());
        }

        NextEntry = (*NextEntry).Flink;
    }

    ListHead = &mut (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).BootDriverListHead as *mut LIST_ENTRY;
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        BootEntry = ((NextEntry as usize) - core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_LIST_ENTRY;

        DriverInfo = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<DRIVER_INFORMATION>(), TAG_IO) as PDRIVER_INFORMATION;
        if !DriverInfo.is_null() {
            core::ptr::write_bytes(DriverInfo as *mut u8, 0, core::mem::size_of::<DRIVER_INFORMATION>());
            InitializeListHead(&mut (*DriverInfo).Link as *mut LIST_ENTRY);
            (*DriverInfo).DataTableEntry = BootEntry;

            Status = IopOpenRegistryKeyEx(
                &mut KeyHandle as *mut HANDLE,
                core::ptr::null_mut(),
                &mut (*BootEntry).RegistryPath as *mut _UNICODE_STRING,
                KEY_READ,
            );

            if Status >= 0 || (!KeLoaderBlock.is_null() && !(*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).SetupLdrBlock.is_null()) {
                if Status < 0 {
                    KeyHandle = 1 as HANDLE;
                }
                (*DriverInfo).ServiceHandle = KeyHandle;

                Index = PpInitGetGroupOrderIndex(KeyHandle) as ULONG;

                (*DriverInfo).TagPosition = PipGetDriverTagPriority(KeyHandle) as USHORT;

                NextEntry2 = (*IopGroupTable.add(Index as usize)).Flink;
                while NextEntry2 != IopGroupTable.add(Index as usize) {
                    DriverInfoTag = ((NextEntry2 as usize) - core::mem::offset_of!(_DRIVER_INFORMATION, Link)) as PDRIVER_INFORMATION;

                    if (*DriverInfoTag).TagPosition > (*DriverInfo).TagPosition {
                        break;
                    }

                    NextEntry2 = (*NextEntry2).Flink;
                }

                NextEntry2 = (*NextEntry2).Blink;
                InsertHeadList(NextEntry2, &mut (*DriverInfo).Link as *mut LIST_ENTRY);
            }
        }

        NextEntry = (*NextEntry).Flink;
    }

    for j in 0..IopGroupIndex as usize {
        NextEntry = (*IopGroupTable.add(j)).Flink;
        let mut drv_count: u32 = 0;
        while NextEntry != IopGroupTable.add(j) {
            DriverInfo = ((NextEntry as usize) - core::mem::offset_of!(_DRIVER_INFORMATION, Link)) as PDRIVER_INFORMATION;

            LdrEntry = (*(*DriverInfo).DataTableEntry).LdrEntry;

            drv_count += 1;
            let ret = IopInitializeBuiltinDriver(LdrEntry);
            if ret != 0 {
                PiQueueDeviceAction(
                    (*(IopRootDeviceNode as *mut _DEVICE_NODE)).PhysicalDeviceObject,
                    PiActionEnumRootDevices,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }

            NextEntry = (*NextEntry).Flink;
        }
    }
    PnPBootDriversLoaded = 1u8;

    PiQueueDeviceAction(
        (*(IopRootDeviceNode as *mut _DEVICE_NODE)).PhysicalDeviceObject,
        PiActionEnumDeviceTree,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn IopInitializeSystemDrivers() {
    #[inline(never)]
    unsafe fn sd_trace(s: &[u8]) {
        crate::dbg_print!(b"%s\0", s.as_ptr() as *const i8);
    }
    let mut DriverList: *mut PUNICODE_STRING;
    let mut SavedList: *mut PUNICODE_STRING;

    sd_trace(b"[SYSDRV] PiPerformSyncDeviceAction\r\n\0");
    PiPerformSyncDeviceAction(
        (*(IopRootDeviceNode as *mut _DEVICE_NODE)).PhysicalDeviceObject,
        PiActionEnumDeviceTree,
    );
    sd_trace(b"[SYSDRV] PiPerformSyncDeviceAction done\r\n\0");

    let setup_ldr = (*(KeLoaderBlock as *mut _LOADER_PARAMETER_BLOCK)).SetupLdrBlock;
    if !setup_ldr.is_null() {
        return;
    }
    // arm64 used to skip this bulk load and ZwLoadDriver a fixed list (netstack, Afd,
    // Npfs) instead, because the hand-staged ramdisk lacked most Start=1 images and
    // loading an absent one hung. The ramdisk is the livecd file set now, so arm64
    // loads what the registry says, in its group order, like x86.
    sd_trace(b"[SYSDRV] CmGetSystemDriverList\r\n\0");
    SavedList = CmGetSystemDriverList();
    DriverList = SavedList;
    sd_trace(b"[SYSDRV] CmGetSystemDriverList done\r\n\0");

    ARM64_UART_TRACE!("[SYSDRV] Loading system drivers...\r\n");

    let mut sys_drv_count: u32 = 0;

    while !(*DriverList).is_null() {
        sys_drv_count += 1;
        crate::dbg_print!(b"[SYSDRV] loading #%d: %wZ\r\n\0", sys_drv_count, *DriverList);

        #[cfg(all(target_arch = "aarch64", feature = "arm64_uart_trace"))]
        {
            let uart = Arm64UartKernelBase as *mut ULONG;
            if !uart.is_null() {
                let msg = b"[SYSDRV] ZwLoadDriver: ";
                for &b in msg {
                    Arm64UartPutChar(b as u8);
                }
                if !(**DriverList).Buffer.is_null() {
                    for k in 0..((**DriverList).Length / 2) as usize {
                        Arm64UartPutChar((*(**DriverList).Buffer.add(k) as u8) as u8);
                    }
                }
                let nl = b"\r\n";
                for &b in nl {
                    Arm64UartPutChar(b as u8);
                }
            }
        }

        ZwLoadDriver(*DriverList);

        RtlFreeUnicodeString(*DriverList);
        ExFreePool(*DriverList as PVOID);

        InbvIndicateProgress();
        DriverList = DriverList.add(1);
    }

    ARM64_UART_TRACE!("[SYSDRV] All system drivers loaded\r\n");

    ExFreePool(SavedList as PVOID);

    PiQueueDeviceAction(
        (*(IopRootDeviceNode as *mut _DEVICE_NODE)).PhysicalDeviceObject,
        PiActionEnumDeviceTree,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ARM64_UART_TRACE!("[SYSDRV] IopInitializeSystemDrivers complete\r\n");
}
}

unsafe extern "system" fn IopUnloadDriver(
    DriverServiceName: PUNICODE_STRING,
    UnloadPnpDrivers: BOOLEAN,
) -> NTSTATUS {
    static mut BACKSLASH: _UNICODE_STRING = _UNICODE_STRING {
        Length: 2,
        MaximumLength: 4,
        Buffer: BACKSLASH_STR.as_ptr() as *mut u16,
    };
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = unsafe { core::mem::zeroed() };
    let mut ImagePath = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut ServiceName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut ObjectName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT;
    let mut DeviceExtension: PEXTENDED_DEVOBJ_EXTENSION;
    let mut Status: NTSTATUS = 0;
    let mut LastBackslash: USHORT = 0;
    let mut SafeToUnload = 1u8;
    let PreviousMode: KPROCESSOR_MODE;
    let mut CapturedServiceName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };

    PreviousMode = ExGetPreviousMode();

    if SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = ProbeAndCaptureUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode, DriverServiceName);
    if Status < 0 {
        return Status;
    }

    if CapturedServiceName.Length == 0 || CapturedServiceName.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return STATUS_INVALID_PARAMETER;
    }

    Status = RtlFindCharInUnicodeString(
        RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
        &mut CapturedServiceName as *mut _UNICODE_STRING,
        &raw const BACKSLASH as *const _UNICODE_STRING as *mut _UNICODE_STRING,
        &mut LastBackslash as *mut USHORT,
    );
    if Status >= 0 {
        ServiceName.Buffer = CapturedServiceName.Buffer.add((LastBackslash / core::mem::size_of::<u16>() as USHORT + 1) as usize);
        ServiceName.Length = CapturedServiceName.Length - LastBackslash - core::mem::size_of::<u16>() as USHORT;
        ServiceName.MaximumLength = CapturedServiceName.MaximumLength - LastBackslash - core::mem::size_of::<u16>() as USHORT;
    } else {
        ServiceName = CapturedServiceName;
    }

    {
        let a = (DRIVER_ROOT_NAME.len() * core::mem::size_of::<u16>()) as u32;
        let b = ServiceName.Length as u32;
        let sum = a + b;
        if sum > u16::MAX as u32 {
            Status = 0xC0000095_u32 as NTSTATUS; // STATUS_INTEGER_OVERFLOW
        } else {
            ObjectName.MaximumLength = sum as USHORT;
            Status = STATUS_SUCCESS;
        }
    }
    if Status < 0 {
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return Status;
    }
    ObjectName.Length = 0;
    ObjectName.Buffer = ExAllocatePoolWithTag(PagedPool, ObjectName.MaximumLength as usize, TAG_IO) as *mut u16;
    if ObjectName.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlAppendUnicodeToString(&mut ObjectName as *mut _UNICODE_STRING, DRIVER_ROOT_NAME.as_ptr());
    RtlAppendUnicodeStringToString(&mut ObjectName as *mut _UNICODE_STRING, &mut ServiceName as *mut _UNICODE_STRING);

    Status = ObReferenceObjectByName(
        &mut ObjectName as *mut _UNICODE_STRING,
        0,
        core::ptr::null_mut(),
        0,
        IoDriverObjectType,
        0i32 as i8,
        core::ptr::null_mut(),
        &mut DriverObject as *mut PDRIVER_OBJECT as *mut PVOID,
    );

    if Status < 0 {
        ExFreePoolWithTag(ObjectName.Buffer as PVOID, TAG_IO);
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return Status;
    }

    ExFreePoolWithTag(ObjectName.Buffer as PVOID, TAG_IO);

    if ((*DriverObject).Flags & DRVO_UNLOAD_INVOKED) != 0 {
        ObDereferenceObject(DriverObject as PVOID);
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return STATUS_DELETE_PENDING;
    }

    core::ptr::write_bytes(&mut QueryTable as *mut _ as *mut u8, 0, core::mem::size_of_val(&QueryTable));

    RtlInitUnicodeString(&mut ImagePath as *mut _UNICODE_STRING, core::ptr::null_mut());

    QueryTable[0].Name = IMAGE_PATH_STR.as_ptr() as *mut u16;
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    QueryTable[0].EntryContext = &mut ImagePath as *mut _UNICODE_STRING as PVOID;

    // Divergence from the C, which passes CapturedServiceName.Buffer straight through:
    // RTL_REGISTRY_ABSOLUTE takes a NUL-terminated PWSTR, but this is a COUNTED string.
    // A driver unloading itself from DriverEntry passes its RegistryPath, which points
    // into a KEY_NAME_INFORMATION block with no terminator, so the key name ran on into
    // whatever pool bytes followed and the open failed STATUS_OBJECT_NAME_NOT_FOUND on
    // about half of boots (kmtest IoDeviceObject_drv.c:300, caught by a gdb probe
    // reading the path at NtOpenKey). Query through a terminated copy.
    let TerminatedLength = CapturedServiceName.Length as usize + core::mem::size_of::<u16>();
    let TerminatedName = ExAllocatePoolWithTag(PagedPool, TerminatedLength, TAG_IO) as *mut u16;
    if TerminatedName.is_null() {
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        ObDereferenceObject(DriverObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        CapturedServiceName.Buffer,
        TerminatedName,
        CapturedServiceName.Length as usize / core::mem::size_of::<u16>(),
    );
    *TerminatedName.add(CapturedServiceName.Length as usize / core::mem::size_of::<u16>()) = 0;

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        TerminatedName,
        QueryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ExFreePoolWithTag(TerminatedName as PVOID, TAG_IO);
    ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);

    if Status < 0 {
        ObDereferenceObject(DriverObject as PVOID);
        return Status;
    }

    Status = IopNormalizeImagePath(&mut ImagePath as *mut _UNICODE_STRING, &mut ServiceName as *mut _UNICODE_STRING);

    if Status < 0 {
        ObDereferenceObject(DriverObject as PVOID);
        return Status;
    }

    ExFreePool(ImagePath.Buffer as PVOID);

    if !(*DriverObject).DriverUnload.is_none()
        && !(*DriverObject).DriverSection.is_null()
        && (UnloadPnpDrivers != 0 || ((*DriverObject).Flags & DRVO_LEGACY_DRIVER) != 0)
    {
        DeviceObject = (*DriverObject).DeviceObject;
        while !DeviceObject.is_null() {
            DeviceExtension = (*DeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION;
            (*DeviceExtension).ExtensionFlags |= DOE_UNLOAD_PENDING;

            if (*DeviceObject).ReferenceCount != 0 || !(*DeviceObject).AttachedDevice.is_null() {
                SafeToUnload = 0u8;
            }

            DeviceObject = (*DeviceObject).NextDevice;
        }

        if SafeToUnload == 0 {
            ObDereferenceObject(DriverObject as PVOID);
            return STATUS_SUCCESS;
        }

        (*DriverObject).Flags |= DRVO_UNLOAD_INVOKED;
        Status = IopDoLoadUnloadDriver(core::ptr::null_mut(), &mut DriverObject as *mut PDRIVER_OBJECT);

        ObMakeTemporaryObject(DriverObject as PVOID);

        ObDereferenceObject(DriverObject as PVOID);
        ObDereferenceObject(DriverObject as PVOID);

        return Status;
    } else {
        ObDereferenceObject(DriverObject as PVOID);

        return STATUS_INVALID_DEVICE_REQUEST;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopReinitializeDrivers() {
    let mut ReinitItem: PDRIVER_REINIT_ITEM;
    let mut Entry: PLIST_ENTRY;

    Entry = ExInterlockedRemoveHeadList(&raw mut DriverReinitListHead as *mut LIST_ENTRY, &raw mut DriverReinitListLock as *mut KSPIN_LOCK as *mut _);
    while !Entry.is_null() {
        ReinitItem = ((Entry as usize) - core::mem::offset_of!(_DRIVER_REINIT_ITEM, ItemEntry)) as PDRIVER_REINIT_ITEM;

        (*(*(*ReinitItem).DriverObject).DriverExtension).Count += 1;

        (*(*ReinitItem).DriverObject).Flags &= !DRVO_REINIT_REGISTERED;

        ((*ReinitItem).ReinitRoutine.unwrap())(
            (*ReinitItem).DriverObject,
            (*ReinitItem).Context,
            (*(*(*ReinitItem).DriverObject).DriverExtension).Count,
        );

        ExFreePool(Entry as PVOID);

        Entry = ExInterlockedRemoveHeadList(&raw mut DriverReinitListHead as *mut LIST_ENTRY, &raw mut DriverReinitListLock as *mut KSPIN_LOCK as *mut _);
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopReinitializeBootDrivers() {
    let mut ReinitItem: PDRIVER_REINIT_ITEM;
    let mut Entry: PLIST_ENTRY;

    Entry = ExInterlockedRemoveHeadList(&raw mut DriverBootReinitListHead as *mut LIST_ENTRY, &raw mut DriverBootReinitListLock as *mut KSPIN_LOCK as *mut _);
    while !Entry.is_null() {
        ReinitItem = ((Entry as usize) - core::mem::offset_of!(_DRIVER_REINIT_ITEM, ItemEntry)) as PDRIVER_REINIT_ITEM;

        (*(*(*ReinitItem).DriverObject).DriverExtension).Count += 1;

        (*(*ReinitItem).DriverObject).Flags &= !DRVO_BOOTREINIT_REGISTERED;

        ((*ReinitItem).ReinitRoutine.unwrap())(
            (*ReinitItem).DriverObject,
            (*ReinitItem).Context,
            (*(*(*ReinitItem).DriverObject).DriverExtension).Count,
        );

        ExFreePool(Entry as PVOID);

        Entry = ExInterlockedRemoveHeadList(&raw mut DriverBootReinitListHead as *mut LIST_ENTRY, &raw mut DriverBootReinitListLock as *mut KSPIN_LOCK as *mut _);
    }

    KeWaitForSingleObject(&raw mut PiEnumerationFinished as *mut KEVENT as PVOID, _KWAIT_REASON_Executive, 0i32 as i8, 0u8, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateDriver(
    DriverName: PUNICODE_STRING,
    InitializationFunction: ntkrnl_sys::ddk::PDRIVER_INITIALIZE,
) -> NTSTATUS {
    let DriverName = opaque_ptr(DriverName);
    let mut NameBuffer: [u16; 100] = [0; 100];
    let mut NameLength: USHORT;
    let mut LocalDriverName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut Status: NTSTATUS = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = unsafe { core::mem::zeroed() };
    let mut ObjectSize: ULONG = 0;
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    let mut ServiceKeyName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut hDriver: HANDLE = core::ptr::null_mut();
    let mut i: ULONG;
    let mut RetryCount: ULONG = 0;

    loop {
        if DriverName.is_null() {
            NameLength = _snwprintf(
                NameBuffer.as_mut_ptr(),
                NameBuffer.len(),
                DRIVER_FMT_STR.as_ptr() as *const u16,
                (*(core::ptr::addr_of!(KeTickCount) as *const LARGE_INTEGER)).s.LowPart,
            ) as USHORT;
            LocalDriverName.Length = NameLength * core::mem::size_of::<u16>() as USHORT;
            LocalDriverName.MaximumLength = LocalDriverName.Length + core::mem::size_of::<u16>() as USHORT;
            LocalDriverName.Buffer = NameBuffer.as_mut_ptr();
        } else {
            LocalDriverName = *DriverName;
        }

        ObjectSize = core::mem::size_of::<DRIVER_OBJECT>() as ULONG + core::mem::size_of::<EXTENDED_DRIVER_EXTENSION>() as ULONG;
        ObjectAttributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: &mut LocalDriverName as *mut _UNICODE_STRING,
            Attributes: OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        Status = ObCreateObject(
            0i32 as i8,
            IoDriverObjectType,
            &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES,
            0i32 as i8,
            core::ptr::null_mut(),
            ObjectSize,
            0,
            0,
            &mut DriverObject as *mut PDRIVER_OBJECT as *mut PVOID,
        );
        if Status < 0 {
            return Status;
        }

        core::ptr::write_bytes(DriverObject as *mut u8, 0, ObjectSize as usize);
        (*DriverObject).Type = IO_TYPE_DRIVER as i16;
        (*DriverObject).Size = core::mem::size_of::<DRIVER_OBJECT>() as CSHORT;
        (*DriverObject).Flags = DRVO_BUILTIN_DRIVER;
        (*DriverObject).DriverExtension = (DriverObject as usize + core::mem::size_of::<DRIVER_OBJECT>()) as *mut _DRIVER_EXTENSION;
        (*(*DriverObject).DriverExtension).DriverObject = DriverObject;
        (*DriverObject).DriverInit = InitializationFunction;

        for j in 0..=IRP_MJ_MAXIMUM_FUNCTION {
            (*DriverObject).MajorFunction[j as usize] = core::mem::transmute(Some(IopInvalidDeviceRequest as unsafe extern "system" fn(*mut DEVICE_OBJECT, PIRP) -> NTSTATUS));
        }

        ServiceKeyName.MaximumLength = LocalDriverName.Length + core::mem::size_of::<u16>() as USHORT;
        ServiceKeyName.Buffer = ExAllocatePoolWithTag(PagedPool, ServiceKeyName.MaximumLength as usize, TAG_IO) as *mut u16;
        if ServiceKeyName.Buffer.is_null() {
            ObMakeTemporaryObject(DriverObject as PVOID);
            ObDereferenceObject(DriverObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyUnicodeString(&mut ServiceKeyName as *mut _UNICODE_STRING, &mut LocalDriverName as *mut _UNICODE_STRING);
        *ServiceKeyName.Buffer.add(ServiceKeyName.Length as usize / core::mem::size_of::<u16>()) = 0;
        (*(*DriverObject).DriverExtension).ServiceKeyName = ServiceKeyName;

        (*DriverObject).DriverName.MaximumLength = LocalDriverName.Length;
        (*DriverObject).DriverName.Buffer = ExAllocatePoolWithTag(PagedPool, (*DriverObject).DriverName.MaximumLength as usize, TAG_IO) as *mut u16;
        if (*DriverObject).DriverName.Buffer.is_null() {
            ObMakeTemporaryObject(DriverObject as PVOID);
            ObDereferenceObject(DriverObject as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyUnicodeString(&mut (*DriverObject).DriverName as *mut _UNICODE_STRING, &mut LocalDriverName as *mut _UNICODE_STRING);

        Status = ObInsertObject(
            DriverObject as PVOID,
            core::ptr::null_mut(),
            FILE_READ_DATA,
            0,
            core::ptr::null_mut(),
            &mut hDriver as *mut HANDLE,
        );

        if DriverName.is_null() && Status == STATUS_OBJECT_NAME_COLLISION && RetryCount < 100 {
            RetryCount += 1;
            continue;
        }

        break;
    }

    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        hDriver,
        0,
        IoDriverObjectType,
        0i32 as i8,
        &mut DriverObject as *mut PDRIVER_OBJECT as *mut PVOID,
        core::ptr::null_mut(),
    );

    ZwClose(hDriver);

    if Status < 0 {
        ObMakeTemporaryObject(DriverObject as PVOID);
        ObDereferenceObject(DriverObject as PVOID);
        return Status;
    }

    Status = (InitializationFunction.unwrap())(DriverObject, core::ptr::null_mut());
    if Status < 0 {
        ObMakeTemporaryObject(DriverObject as PVOID);
        ObDereferenceObject(DriverObject as PVOID);
        return Status;
    }

    for j in 0..=IRP_MJ_MAXIMUM_FUNCTION {
        if (*DriverObject).MajorFunction[j as usize].is_none() {
            (*DriverObject).MajorFunction[j as usize] = core::mem::transmute(Some(IopInvalidDeviceRequest as unsafe extern "system" fn(*mut DEVICE_OBJECT, PIRP) -> NTSTATUS));
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoDeleteDriver(DriverObject: PDRIVER_OBJECT) {
    let DriverObject = opaque_ptr(DriverObject);
    ObDereferenceObject(DriverObject as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterBootDriverReinitialization(
    DriverObject: PDRIVER_OBJECT,
    ReinitRoutine: ntkrnl_sys::ddk::PDRIVER_REINITIALIZE,
    Context: PVOID,
) {
    let DriverObject = opaque_ptr(DriverObject);
    let Context = opaque_ptr(Context);
    let ReinitItem = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<DRIVER_REINIT_ITEM>(), TAG_REINIT) as PDRIVER_REINIT_ITEM;
    if ReinitItem.is_null() {
        return;
    }

    (*ReinitItem).DriverObject = DriverObject;
    (*ReinitItem).ReinitRoutine = ReinitRoutine;
    (*ReinitItem).Context = Context;

    (*DriverObject).Flags |= DRVO_BOOTREINIT_REGISTERED;
    ExInterlockedInsertTailList(
        &raw mut DriverBootReinitListHead as *mut LIST_ENTRY,
        &mut (*ReinitItem).ItemEntry as *mut LIST_ENTRY,
        &raw mut DriverBootReinitListLock as *mut KSPIN_LOCK as *mut _,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterDriverReinitialization(
    DriverObject: PDRIVER_OBJECT,
    ReinitRoutine: ntkrnl_sys::ddk::PDRIVER_REINITIALIZE,
    Context: PVOID,
) {
    let DriverObject = opaque_ptr(DriverObject);
    let Context = opaque_ptr(Context);
    let ReinitItem = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<DRIVER_REINIT_ITEM>(), TAG_REINIT) as PDRIVER_REINIT_ITEM;
    if ReinitItem.is_null() {
        return;
    }

    (*ReinitItem).DriverObject = DriverObject;
    (*ReinitItem).ReinitRoutine = ReinitRoutine;
    (*ReinitItem).Context = Context;

    (*DriverObject).Flags |= DRVO_REINIT_REGISTERED;
    ExInterlockedInsertTailList(
        &raw mut DriverReinitListHead as *mut LIST_ENTRY,
        &mut (*ReinitItem).ItemEntry as *mut LIST_ENTRY,
        &raw mut DriverReinitListLock as *mut KSPIN_LOCK as *mut _,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IoAllocateDriverObjectExtension(
    DriverObject: PDRIVER_OBJECT,
    ClientIdentificationAddress: PVOID,
    DriverObjectExtensionSize: ULONG,
    DriverObjectExtension: *mut PVOID,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let ClientIdentificationAddress = opaque_ptr(ClientIdentificationAddress);
    let DriverObjectExtension = opaque_ptr(DriverObjectExtension);
    let mut OldIrql: KIRQL;
    let mut DriverExtensions: PIO_CLIENT_EXTENSION;
    let NewDriverExtension: PIO_CLIENT_EXTENSION;
    let mut Inserted = 0u8;

    *DriverObjectExtension = core::ptr::null_mut();

    NewDriverExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<IO_CLIENT_EXTENSION>() + DriverObjectExtensionSize as usize,
        TAG_DRIVER_EXTENSION,
    ) as PIO_CLIENT_EXTENSION;
    if NewDriverExtension.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        NewDriverExtension as *mut u8,
        0,
        core::mem::size_of::<IO_CLIENT_EXTENSION>() + DriverObjectExtensionSize as usize,
    );

    OldIrql = KeRaiseIrqlToDpcLevel();

    (*NewDriverExtension).ClientIdentificationAddress = ClientIdentificationAddress;

    DriverExtensions = (*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ClientDriverExtension;
    while !DriverExtensions.is_null() {
        if (*DriverExtensions).ClientIdentificationAddress == ClientIdentificationAddress {
            break;
        }

        DriverExtensions = (*DriverExtensions).NextExtension;
    }

    if DriverExtensions.is_null() {
        (*NewDriverExtension).NextExtension = (*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ClientDriverExtension;
        (*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ClientDriverExtension = NewDriverExtension;
        Inserted = 1u8;
    }

    KeLowerIrql(OldIrql);

    if Inserted == 0 {
        ExFreePoolWithTag(NewDriverExtension as PVOID, TAG_DRIVER_EXTENSION);
        return STATUS_OBJECT_NAME_COLLISION;
    }

    *DriverObjectExtension = (NewDriverExtension as usize + core::mem::size_of::<IO_CLIENT_EXTENSION>()) as PVOID;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDriverObjectExtension(
    DriverObject: PDRIVER_OBJECT,
    ClientIdentificationAddress: PVOID,
) -> PVOID {
    let DriverObject = opaque_ptr(DriverObject);
    let ClientIdentificationAddress = opaque_ptr(ClientIdentificationAddress);
    let mut OldIrql: KIRQL = 0;
    let mut DriverExtensions: PIO_CLIENT_EXTENSION;

    OldIrql = KeRaiseIrqlToDpcLevel();

    DriverExtensions = (*((*DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION)).ClientDriverExtension;
    while !DriverExtensions.is_null() {
        if (*DriverExtensions).ClientIdentificationAddress == ClientIdentificationAddress {
            break;
        }

        DriverExtensions = (*DriverExtensions).NextExtension;
    }

    KeLowerIrql(OldIrql);

    if DriverExtensions.is_null() {
        return core::ptr::null_mut();
    }
    (DriverExtensions as usize + core::mem::size_of::<IO_CLIENT_EXTENSION>()) as PVOID
}

/// `IopInitializeDriverModule` — Direct port of the C body that lived in
/// `io_rust_shims.c` from 2024 onward. Was the last `_rust_shims.c` "complex
/// delegation" entry on the io ledger.
///
/// Creates the DRIVER_OBJECT for a loaded module, fills in its fields,
/// inserts it into the object namespace under the driver name, and calls
/// the module's DriverEntry. Returns the in-out DriverObject and the
/// driver-entry status separately (DriverEntry's NTSTATUS goes through
/// `*DriverEntryStatus`; the function's own status is the load+init
/// pipeline's status).
///
/// Mirrors the C cleanup paths bit-for-bit. The major-function fixup loop
/// is preserved (Windows compatibility), and the legacy-flag clear when no
/// device objects were created is preserved as the same HACK comment.
///
/// `#[inline(never)]` is load-bearing — without it, LLVM's IPO folds the
/// success-path tail (the `MmFreeDriverInitialization → IopReadyDeviceObjects
/// → IopReinitializeDrivers` chain) into ud2 because it deduces the call to
/// the indirect DriverInit function pointer can't return successfully (a
/// captures(none)-shaped propagation that even write_volatile + opaque_ptr
/// laundering at the field level can't defeat).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IopInitializeDriverModule(
    ModuleObject: PLDR_DATA_TABLE_ENTRY,
    ServiceHandle: HANDLE,
    OutDriverObject: *mut PDRIVER_OBJECT,
    DriverEntryStatus: *mut NTSTATUS,
) -> NTSTATUS {
    const KeyNameInformation: ULONG = 3;
    const IMAGE_DLLCHARACTERISTICS_WDM_DRIVER: USHORT = 0x2000;

    let ModuleObject = opaque_ptr(ModuleObject);
    let ServiceHandle = opaque_ptr(ServiceHandle);
    let OutDriverObject = opaque_ptr(OutDriverObject);
    let DriverEntryStatus = opaque_ptr(DriverEntryStatus);

    let mut DriverName: _UNICODE_STRING = core::mem::zeroed();
    let mut ServiceName: _UNICODE_STRING = core::mem::zeroed();
    let mut RegistryPath: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = IopGetDriverNames(ServiceHandle, &mut DriverName, &mut ServiceName);
    if Status < 0 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"IopInitializeDriverModule: IopGetDriverNames failed 0x%x\n\0".as_ptr() as *const i8,
            Status,
        );
        MmUnloadSystemImage(ModuleObject as PVOID);
        return Status;
    }

    let NtHeaders = RtlImageNtHeader((*ModuleObject).DllBase);

    // Obtain the registry path for the DriverInit routine
    let mut nameInfo: PKEY_NAME_INFORMATION = core::ptr::null_mut();
    let mut infoLength: ULONG = 0;
    Status = ZwQueryKey(
        ServiceHandle,
        KeyNameInformation,
        core::ptr::null_mut(),
        0,
        &mut infoLength,
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        // One WCHAR more than the key name needs: see the terminator below.
        nameInfo = ExAllocatePoolWithTag(
            NonPagedPool,
            infoLength as usize + core::mem::size_of::<u16>(),
            TAG_IO,
        ) as PKEY_NAME_INFORMATION;
        if !nameInfo.is_null() {
            Status = ZwQueryKey(
                ServiceHandle,
                KeyNameInformation,
                nameInfo as PVOID,
                infoLength,
                &mut infoLength,
            );
            if Status >= 0 {
                RegistryPath.Length = (*nameInfo).NameLength as USHORT;
                RegistryPath.MaximumLength = (*nameInfo).NameLength as USHORT;
                // Bindgen renders Name as `[WCHAR; 1]`. Take the address of the
                // base of the array (not an indexed slot) so we don't trip the
                // bounds-check that fires when index >= 1.
                RegistryPath.Buffer = core::ptr::addr_of_mut!((*nameInfo).Name) as *mut u16;
                // Divergence from the C: NUL-terminate, in the WCHAR allocated past the
                // name. KeyNameInformation is counted, but drivers hand
                // RegistryPath->Buffer to RTL_REGISTRY_ABSOLUTE queries as a PWSTR
                // (mountmgr, ramdisk, i8042prt here), and without this the key name ran
                // on into pool garbage -- the open then failed
                // STATUS_OBJECT_NAME_NOT_FOUND on about half of boots. The terminator
                // stays OUTSIDE MaximumLength: Windows reports MaximumLength == Length
                // (kmtest IoDeviceObject_drv.c:191).
                *RegistryPath
                    .Buffer
                    .add((*nameInfo).NameLength as usize / core::mem::size_of::<u16>()) = 0;
            } else {
                ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
                nameInfo = core::ptr::null_mut();
            }
        } else {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        Status = if Status >= 0 { STATUS_UNSUCCESSFUL } else { Status };
    }

    if Status < 0 {
        RtlFreeUnicodeString(&mut ServiceName);
        RtlFreeUnicodeString(&mut DriverName);
        MmUnloadSystemImage(ModuleObject as PVOID);
        return Status;
    }

    // Create the driver object
    let ObjectSize = (core::mem::size_of::<DRIVER_OBJECT>()
        + core::mem::size_of::<EXTENDED_DRIVER_EXTENSION>()) as ULONG;
    let mut objAttrs = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &mut DriverName,
        Attributes: OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut driverObject: PDRIVER_OBJECT = core::ptr::null_mut();

    Status = ObCreateObject(
        KernelMode,
        IoDriverObjectType,
        &mut objAttrs,
        KernelMode,
        core::ptr::null_mut(),
        ObjectSize,
        0,
        0,
        &mut driverObject as *mut PDRIVER_OBJECT as *mut PVOID,
    );
    if Status < 0 {
        // The C prints this after freeing DriverName; print it while the name is still live.
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"Error while creating driver object \"%wZ\" status %x\n\0".as_ptr() as *const i8,
            &mut DriverName as *mut _,
            Status,
        );
        ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
        RtlFreeUnicodeString(&mut ServiceName);
        RtlFreeUnicodeString(&mut DriverName);
        MmUnloadSystemImage(ModuleObject as PVOID);
        return Status;
    }
    // Launder through opaque_ptr so LLVM can't propagate captures(none) from
    // the bindgen ObCreateObject decl back to the &mut driverObject slot —
    // without this, later (*driverObject).field accesses fold to a NULL
    // base and LLVM emits ud2 in MmFreeDriverInitialization at function tail.
    let driverObject: PDRIVER_OBJECT = opaque_ptr(driverObject);

    core::ptr::write_bytes(driverObject as *mut u8, 0, ObjectSize as usize);
    // Use write_volatile for every field assignment so LLVM can't deduce
    // captures(none)-shaped "the field is still in its post-zero-init state"
    // — without this, the later `(*driverObject).DriverInit.unwrap()` folds
    // to the unwrap_failed (panic) branch + ud2.
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*driverObject).Type), IO_TYPE_DRIVER as i16);
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).Size),
        core::mem::size_of::<DRIVER_OBJECT>() as CSHORT,
    );

    // Set the legacy flag if this is not a WDM driver
    if ((*NtHeaders).OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_WDM_DRIVER) == 0 {
        let cur = core::ptr::read_volatile(core::ptr::addr_of!((*driverObject).Flags));
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*driverObject).Flags),
            cur | DRVO_LEGACY_DRIVER,
        );
    }

    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).DriverSection),
        ModuleObject as PVOID,
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).DriverStart),
        (*ModuleObject).DllBase,
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).DriverSize),
        (*ModuleObject).SizeOfImage,
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).DriverInit),
        core::mem::transmute((*ModuleObject).EntryPoint),
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).HardwareDatabase),
        core::ptr::addr_of_mut!(IopHardwareDatabaseKey),
    );
    let driverExt =
        (driverObject as usize + core::mem::size_of::<DRIVER_OBJECT>()) as *mut _DRIVER_EXTENSION;
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverObject).DriverExtension),
        driverExt,
    );
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*driverExt).DriverObject),
        driverObject,
    );

    // Loop all Major Functions
    let invalid_dispatch: ntkrnl_sys::ddk::PDRIVER_DISPATCH = core::mem::transmute(Some(
        IopInvalidDeviceRequest
            as unsafe extern "system" fn(*mut DEVICE_OBJECT, PIRP) -> NTSTATUS,
    ));
    for j in 0..=IRP_MJ_MAXIMUM_FUNCTION {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*driverObject).MajorFunction[j as usize]),
            invalid_dispatch,
        );
    }

    // Add the Object and get its handle
    let mut hDriver: HANDLE = core::ptr::null_mut();
    Status = ObInsertObject(
        driverObject as PVOID,
        core::ptr::null_mut(),
        FILE_READ_DATA,
        0,
        core::ptr::null_mut(),
        &mut hDriver,
    );
    if Status < 0 {
        ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
        RtlFreeUnicodeString(&mut ServiceName);
        RtlFreeUnicodeString(&mut DriverName);
        return Status;
    }

    // Now reference it. Use a fresh slot for the output so opaque_ptr can
    // launder its result without mutating the already-laundered driverObject
    // immutable shadow above.
    let mut refDriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    Status = ObReferenceObjectByHandle(
        hDriver,
        0,
        IoDriverObjectType,
        KernelMode,
        &mut refDriverObject as *mut PDRIVER_OBJECT as *mut PVOID,
        core::ptr::null_mut(),
    );

    // Close the extra handle
    ZwClose(hDriver);

    if Status < 0 {
        ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
        RtlFreeUnicodeString(&mut ServiceName);
        RtlFreeUnicodeString(&mut DriverName);
        return Status;
    }
    // Launder the freshly-referenced driverObject. (The C original re-uses
    // the same local; here we keep the post-ObCreate laundered shadow valid
    // while taking the new ObReferenceObjectByHandle result through its own
    // captures(none) firewall.)
    let driverObject: PDRIVER_OBJECT = opaque_ptr(refDriverObject);

    // Set up the service key name buffer
    let mut serviceKeyName: _UNICODE_STRING = core::mem::zeroed();
    serviceKeyName.Length = 0;
    serviceKeyName.MaximumLength = ServiceName.MaximumLength + core::mem::size_of::<u16>() as USHORT;
    serviceKeyName.Buffer =
        ExAllocatePoolWithTag(NonPagedPool, serviceKeyName.MaximumLength as usize, TAG_IO)
            as *mut u16;
    if serviceKeyName.Buffer.is_null() {
        ObMakeTemporaryObject(driverObject as PVOID);
        ObDereferenceObject(driverObject as PVOID);
        ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
        RtlFreeUnicodeString(&mut ServiceName);
        RtlFreeUnicodeString(&mut DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Copy the name and set it in the driver extension
    RtlCopyUnicodeString(&mut serviceKeyName, &mut ServiceName);
    RtlFreeUnicodeString(&mut ServiceName);
    (*(*driverObject).DriverExtension).ServiceKeyName = serviceKeyName;

    // Make a copy of the driver name to store in the driver object
    let mut driverNamePaged: _UNICODE_STRING = core::mem::zeroed();
    driverNamePaged.Length = 0;
    driverNamePaged.MaximumLength = DriverName.MaximumLength + core::mem::size_of::<u16>() as USHORT;
    driverNamePaged.Buffer =
        ExAllocatePoolWithTag(PagedPool, driverNamePaged.MaximumLength as usize, TAG_IO)
            as *mut u16;
    if driverNamePaged.Buffer.is_null() {
        ObMakeTemporaryObject(driverObject as PVOID);
        ObDereferenceObject(driverObject as PVOID);
        ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
        RtlFreeUnicodeString(&mut DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(&mut driverNamePaged, &mut DriverName);
    (*driverObject).DriverName = driverNamePaged;

    // Finally, call its init function. Read DriverInit through read_volatile
    // so LLVM doesn't fold the unwrap to the panic branch (captures(none)-driven
    // "field is still in its zero-init state, so .unwrap() panics" deduction).
    let driverInit: ntkrnl_sys::ddk::PDRIVER_INITIALIZE =
        core::ptr::read_volatile(core::ptr::addr_of!((*driverObject).DriverInit));
    Status = (driverInit.unwrap_unchecked())(driverObject, &mut RegistryPath);
    core::ptr::write_volatile(DriverEntryStatus, Status);
    if Status < 0 {
        // A vendor driver whose DriverEntry fails is otherwise invisible: PnP just never
        // calls its AddDevice.
        ntkrnl_sys::driver_ddk::DbgPrint(
            b"'%wZ' initialization failed, status (0x%08lx)\n\0".as_ptr() as *const i8,
            &mut DriverName as *mut _,
            Status,
        );
        Status = STATUS_FAILED_DRIVER_ENTRY;
    }

    // HACK: We're going to say if we don't have any DOs from DriverEntry, then we're not legacy.
    if (*driverObject).DeviceObject.is_null() {
        (*driverObject).Flags &= !DRVO_LEGACY_DRIVER;
    }

    // Windows does this fixup, keep it for compatibility
    for j in 0..=IRP_MJ_MAXIMUM_FUNCTION {
        if (*driverObject).MajorFunction[j as usize].is_none() {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"Driver <%wZ> set DriverObject->MajorFunction[%lu] to NULL!\n\0".as_ptr() as *const i8,
                core::ptr::addr_of_mut!((*driverObject).DriverName),
                j as u32,
            );
            (*driverObject).MajorFunction[j as usize] = core::mem::transmute(Some(
                IopInvalidDeviceRequest
                    as unsafe extern "system" fn(*mut DEVICE_OBJECT, PIRP) -> NTSTATUS,
            ));
        }
    }

    ExFreePoolWithTag(nameInfo as PVOID, TAG_IO);
    RtlFreeUnicodeString(&mut DriverName);

    if Status < 0 {
        ObMakeTemporaryObject(driverObject as PVOID);
        ObDereferenceObject(driverObject as PVOID);
        return Status;
    }

    core::ptr::write_volatile(OutDriverObject, driverObject);

    // The success-path tail calls (MmFreeDriverInitialization →
    // IopReadyDeviceObjects → IopReinitializeDrivers → return STATUS_SUCCESS)
    // get folded into ud2 by an LLVM IPO chain that even write_volatile +
    // opaque_ptr at every field site can't defeat. Pull the tail out into a
    // separate function so the call boundary breaks the analysis chain.
    iop_initialize_driver_module_tail(opaque_ptr(driverObject), ModuleObject)
}

#[inline(never)]
unsafe fn iop_initialize_driver_module_tail(
    driverObject: PDRIVER_OBJECT,
    moduleObject: PLDR_DATA_TABLE_ENTRY,
) -> NTSTATUS {
    let driverObject = opaque_ptr(driverObject);
    let moduleObject = opaque_ptr(moduleObject);
    MmFreeDriverInitialization(moduleObject);
    IopReadyDeviceObjects(driverObject);
    if PnpSystemInit != 0 {
        IopReinitializeDrivers();
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopLoadDriver(ServiceHandle: HANDLE, DriverObject: *mut PDRIVER_OBJECT) -> NTSTATUS {
    let ServiceHandle = opaque_ptr(ServiceHandle);
    let DriverObject = opaque_ptr(DriverObject);
    let mut ImagePath = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let mut Status: NTSTATUS = 0;
    let mut ModuleObject: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut BaseAddress: PVOID = core::ptr::null_mut();

    let mut kvInfo: *mut _KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    Status = IopGetRegistryValue(ServiceHandle, IMAGE_PATH_STR.as_ptr() as *mut u16, &mut kvInfo);
    if Status >= 0 {
        if ((*kvInfo).Type != REG_EXPAND_SZ && (*kvInfo).Type != REG_SZ)
            || (*kvInfo).DataLength < core::mem::size_of::<u16>() as ULONG
            || (*kvInfo).DataLength > UNICODE_STRING_MAX_BYTES as ULONG
            || ((*kvInfo).DataLength % core::mem::size_of::<u16>() as ULONG) != 0
        {
            ExFreePool(kvInfo as PVOID);
            return STATUS_ILL_FORMED_SERVICE_ENTRY;
        }

        ImagePath.Length = ((*kvInfo).DataLength - core::mem::size_of::<u16>() as ULONG) as USHORT;
        ImagePath.MaximumLength = (*kvInfo).DataLength as USHORT;
        ImagePath.Buffer = ExAllocatePoolWithTag(PagedPool, ImagePath.MaximumLength as usize, TAG_RTLREGISTRY) as *mut u16;
        if ImagePath.Buffer.is_null() {
            ExFreePool(kvInfo as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            ((kvInfo as ULONG_PTR) + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
            ImagePath.Buffer as *mut u8,
            ImagePath.Length as usize,
        );
        *ImagePath.Buffer.add(ImagePath.Length as usize / core::mem::size_of::<u16>()) = 0;
        ExFreePool(kvInfo as PVOID);
    } else {
        return Status;
    }

    Status = IopNormalizeImagePath(&mut ImagePath as *mut _UNICODE_STRING, core::ptr::null_mut());
    if Status < 0 {
        return Status;
    }

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(&raw mut IopDriverLoadResource as *mut ERESOURCE, 1u8);

    Status = MmLoadSystemImage(
        &mut ImagePath as *mut _UNICODE_STRING,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        &mut ModuleObject as *mut PLDR_DATA_TABLE_ENTRY as *mut PVOID,
        &mut BaseAddress as *mut PVOID,
    );
    RtlFreeUnicodeString(&mut ImagePath as *mut _UNICODE_STRING);

    if Status < 0 {
        ExReleaseResourceLite(&raw mut IopDriverLoadResource as *mut ERESOURCE);
        KeLeaveCriticalRegion();
        return Status;
    }

    let mut infoLength: ULONG = 0;
    Status = ZwQueryKey(ServiceHandle, KeyBasicInformation, core::ptr::null_mut(), 0, &mut infoLength);
    if Status == STATUS_BUFFER_TOO_SMALL {
        let servName = ExAllocatePoolWithTag(PagedPool, infoLength as usize, TAG_IO) as *mut _KEY_BASIC_INFORMATION;
        if !servName.is_null() {
            Status = ZwQueryKey(ServiceHandle, KeyBasicInformation, servName as PVOID, infoLength, &mut infoLength);
            if Status >= 0 {
                let serviceName = _UNICODE_STRING {
                    Length: (*servName).NameLength as USHORT,
                    MaximumLength: (*servName).NameLength as USHORT,
                    Buffer: (*servName).Name.as_mut_ptr(),
                };

                IopDisplayLoadingMessage(&serviceName as *const _UNICODE_STRING);
            }
            ExFreePoolWithTag(servName as PVOID, TAG_IO);
        }
    }

    let mut driverEntryStatus: NTSTATUS = STATUS_SUCCESS;
    Status = IopInitializeDriverModule(ModuleObject, ServiceHandle, DriverObject, &mut driverEntryStatus as *mut NTSTATUS);
    if Status < 0 {
    }

    ExReleaseResourceLite(&raw mut IopDriverLoadResource as *mut ERESOURCE);
    KeLeaveCriticalRegion();

    Status
}

unsafe extern "system" fn IopLoadUnloadDriverWorker(Parameter: PVOID) {
    let LoadParams = Parameter as *mut LOAD_UNLOAD_PARAMS;

    if !(*LoadParams).DriverObject.is_null() {
        ((*(*LoadParams).DriverObject).DriverUnload.unwrap())((*LoadParams).DriverObject);
        (*LoadParams).Status = STATUS_SUCCESS;
    } else {
        let mut serviceHandle: HANDLE = core::ptr::null_mut();
        let status: NTSTATUS;
        status = IopOpenRegistryKeyEx(&mut serviceHandle as *mut HANDLE, core::ptr::null_mut(), (*LoadParams).RegistryPath, KEY_READ);
        if status < 0 {
            (*LoadParams).Status = status;
        } else {
            (*LoadParams).Status = IopLoadDriver(serviceHandle, &mut (*LoadParams).DriverObject as *mut PDRIVER_OBJECT);
            ZwClose(serviceHandle);
        }
    }

    if (*LoadParams).SetEvent != 0 {
        KeSetEvent(&mut (*LoadParams).Event as *mut KEVENT as PVOID, 0, 0u8);
    }
}

unsafe fn IopDoLoadUnloadDriver(
    RegistryPath: PUNICODE_STRING,
    DriverObject: *mut PDRIVER_OBJECT,
) -> NTSTATUS {
    let mut LoadParams = LOAD_UNLOAD_PARAMS {
        Status: STATUS_SUCCESS,
        RegistryPath,
        WorkItem: unsafe { core::mem::zeroed() },
        Event: unsafe { core::mem::zeroed() },
        DriverObject: *DriverObject,
        SetEvent: 0u8,
    };

    if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess {
        LoadParams.SetEvent = 1u8;
        KeInitializeEvent(&mut LoadParams.Event as *mut KEVENT, _EVENT_TYPE_NotificationEvent, 0u8);

        ExInitializeWorkItem(&mut LoadParams.WorkItem as *mut _WORK_QUEUE_ITEM, Some(IopLoadUnloadDriverWorker), &mut LoadParams as *mut LOAD_UNLOAD_PARAMS as PVOID);
        ExQueueWorkItem(&mut LoadParams.WorkItem as *mut _WORK_QUEUE_ITEM, DelayedWorkQueue);

        KeWaitForSingleObject(&mut LoadParams.Event as *mut KEVENT as PVOID, _KWAIT_REASON_UserRequest, 0i32 as i8, 0u8, core::ptr::null_mut());
    } else {
        LoadParams.SetEvent = 0u8;
        IopLoadUnloadDriverWorker(&mut LoadParams as *mut LOAD_UNLOAD_PARAMS as PVOID);
    }

    LoadParams.Status
}

#[no_mangle]
pub unsafe extern "system" fn NtLoadDriver(DriverServiceName: PUNICODE_STRING) -> NTSTATUS {
    let DriverServiceName = opaque_ptr(DriverServiceName);
    let mut CapturedServiceName = _UNICODE_STRING {
        Buffer: core::ptr::null_mut(),
        Length: 0,
        MaximumLength: 0,
    };
    let PreviousMode: KPROCESSOR_MODE;
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PreviousMode = KeGetPreviousMode();

    if SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = ProbeAndCaptureUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode, DriverServiceName);
    if Status < 0 {
        return Status;
    }

    if CapturedServiceName.Length == 0 || CapturedServiceName.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
        return STATUS_INVALID_PARAMETER;
    }

    Status = IopDoLoadUnloadDriver(&mut CapturedServiceName as *mut _UNICODE_STRING, &mut DriverObject as *mut PDRIVER_OBJECT);

    ReleaseCapturedUnicodeString(&mut CapturedServiceName as *mut _UNICODE_STRING, PreviousMode);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtUnloadDriver(DriverServiceName: PUNICODE_STRING) -> NTSTATUS {
    let DriverServiceName = opaque_ptr(DriverServiceName);
    IopUnloadDriver(DriverServiceName, 0u8)
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/driver.c
//  * PURPOSE:         Driver Object Management
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Filip Navara (navaraf@reactos.org)
//  *                  Hervé Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
// #include <mm/ARM3/miarm.h>
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
// /* GLOBALS ********************************************************************/
//
// ERESOURCE IopDriverLoadResource;
//
// LIST_ENTRY DriverReinitListHead;
// KSPIN_LOCK DriverReinitListLock;
// PLIST_ENTRY DriverReinitTailEntry;
//
// PLIST_ENTRY DriverBootReinitTailEntry;
// LIST_ENTRY DriverBootReinitListHead;
// KSPIN_LOCK DriverBootReinitListLock;
//
// UNICODE_STRING IopHardwareDatabaseKey =
//    RTL_CONSTANT_STRING(L"\\REGISTRY\\MACHINE\\HARDWARE\\DESCRIPTION\\SYSTEM");
// static const WCHAR ServicesKeyName[] = L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\";
//
// POBJECT_TYPE IoDriverObjectType = NULL;
//
// extern BOOLEAN PnpSystemInit;
// extern BOOLEAN PnPBootDriversLoaded;
// extern KEVENT PiEnumerationFinished;
//
// USHORT IopGroupIndex;
// PLIST_ENTRY IopGroupTable;
//
// /* TYPES *********************************************************************/
//
// // Parameters packet for Load/Unload work item's context
// typedef struct _LOAD_UNLOAD_PARAMS
// {
//     NTSTATUS Status;
//     PUNICODE_STRING RegistryPath;
//     WORK_QUEUE_ITEM WorkItem;
//     KEVENT Event;
//     PDRIVER_OBJECT DriverObject;
//     BOOLEAN SetEvent;
// } LOAD_UNLOAD_PARAMS, *PLOAD_UNLOAD_PARAMS;
//
// NTSTATUS
// IopDoLoadUnloadDriver(
//     _In_opt_ PUNICODE_STRING RegistryPath,
//     _Inout_ PDRIVER_OBJECT *DriverObject);
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// IopInvalidDeviceRequest(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_INVALID_DEVICE_REQUEST;
// }
//
// VOID
// NTAPI
// IopDeleteDriver(IN PVOID ObjectBody)
// {
//     PDRIVER_OBJECT DriverObject = ObjectBody;
//     PIO_CLIENT_EXTENSION DriverExtension, NextDriverExtension;
//     PAGED_CODE();
//
//     DPRINT1("Deleting driver object '%wZ'\n", &DriverObject->DriverName);
//
//     /* There must be no device objects remaining at this point */
//     ASSERT(!DriverObject->DeviceObject);
//
//     /* Get the extension and loop them */
//     DriverExtension = IoGetDrvObjExtension(DriverObject)->ClientDriverExtension;
//     while (DriverExtension)
//     {
//         /* Get the next one */
//         NextDriverExtension = DriverExtension->NextExtension;
//         ExFreePoolWithTag(DriverExtension, TAG_DRIVER_EXTENSION);
//
//         /* Move on */
//         DriverExtension = NextDriverExtension;
//     }
//
//     /* Check if the driver image is still loaded */
//     if (DriverObject->DriverSection)
//     {
//         /* Unload it */
//         MmUnloadSystemImage(DriverObject->DriverSection);
//     }
//
//     /* Check if it has a name */
//     if (DriverObject->DriverName.Buffer)
//     {
//         /* Free it */
//         ExFreePool(DriverObject->DriverName.Buffer);
//     }
//
//     /* Check if it has a service key name */
//     if (DriverObject->DriverExtension->ServiceKeyName.Buffer)
//     {
//         /* Free it */
//         ExFreePool(DriverObject->DriverExtension->ServiceKeyName.Buffer);
//     }
// }
//
// NTSTATUS
// IopGetDriverNames(
//     _In_ HANDLE ServiceHandle,
//     _Out_ PUNICODE_STRING DriverName,
//     _Out_opt_ PUNICODE_STRING ServiceName)
// {
//     UNICODE_STRING driverName = {.Buffer = NULL}, serviceName;
//     PKEY_VALUE_FULL_INFORMATION kvInfo;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     /* 1. Check the "ObjectName" field in the driver's registry key (it has priority) */
//     status = IopGetRegistryValue(ServiceHandle, L"ObjectName", &kvInfo);
//     if (NT_SUCCESS(status))
//     {
//         /* We've got the ObjectName, use it as the driver name */
//         if ((kvInfo->Type != REG_SZ) ||
//             (kvInfo->DataLength < sizeof(UNICODE_NULL)) ||
//             (kvInfo->DataLength > UNICODE_STRING_MAX_BYTES) ||
//             ((kvInfo->DataLength % sizeof(WCHAR)) != 0))
//         {
//             DPRINT1("ObjectName invalid (Type = %lu, DataLength = %lu)\n",
//                     kvInfo->Type,
//                     kvInfo->DataLength);
//             ExFreePool(kvInfo);
//             return STATUS_ILL_FORMED_SERVICE_ENTRY;
//         }
//
//         driverName.Length = (USHORT)(kvInfo->DataLength - sizeof(UNICODE_NULL));
//         driverName.MaximumLength = kvInfo->DataLength;
//         driverName.Buffer = ExAllocatePoolWithTag(NonPagedPool, driverName.MaximumLength, TAG_IO);
//         if (!driverName.Buffer)
//         {
//             ExFreePool(kvInfo);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlMoveMemory(driverName.Buffer,
//                       (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                       driverName.Length);
//         driverName.Buffer[driverName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//         ExFreePool(kvInfo);
//     }
//
//     /* Check whether we need to get ServiceName as well, either to construct
//      * the driver name (because we could not use "ObjectName"), or because
//      * it is requested by the caller. */
//     PKEY_BASIC_INFORMATION basicInfo = NULL;
//     if (!NT_SUCCESS(status) || ServiceName != NULL)
//     {
//         /* Retrieve the necessary buffer size */
//         ULONG infoLength;
//         status = ZwQueryKey(ServiceHandle, KeyBasicInformation, NULL, 0, &infoLength);
//         if (status != STATUS_BUFFER_TOO_SMALL)
//         {
//             status = (NT_SUCCESS(status) ? STATUS_UNSUCCESSFUL : status);
//             goto Cleanup;
//         }
//
//         /* Allocate the buffer and retrieve the data */
//         basicInfo = ExAllocatePoolWithTag(PagedPool, infoLength, TAG_IO);
//         if (!basicInfo)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         status = ZwQueryKey(ServiceHandle, KeyBasicInformation, basicInfo, infoLength, &infoLength);
//         if (!NT_SUCCESS(status))
//         {
//             goto Cleanup;
//         }
//
//         serviceName.Length = basicInfo->NameLength;
//         serviceName.MaximumLength = basicInfo->NameLength;
//         serviceName.Buffer = basicInfo->Name;
//     }
//
//     /* 2. There is no "ObjectName" - construct it ourselves. Depending on the driver type,
//      * it will be either "\Driver\<ServiceName>" or "\FileSystem\<ServiceName>" */
//     if (driverName.Buffer == NULL)
//     {
//         ASSERT(basicInfo); // Container for serviceName
//
//         /* Retrieve the driver type */
//         ULONG driverType;
//         status = IopGetRegistryValue(ServiceHandle, L"Type", &kvInfo);
//         if (!NT_SUCCESS(status))
//         {
//             goto Cleanup;
//         }
//         if (kvInfo->Type != REG_DWORD || kvInfo->DataLength != sizeof(ULONG))
//         {
//             ExFreePool(kvInfo);
//             status = STATUS_ILL_FORMED_SERVICE_ENTRY;
//             goto Cleanup;
//         }
//
//         RtlMoveMemory(&driverType,
//                       (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                       sizeof(ULONG));
//         ExFreePool(kvInfo);
//
//         /* Compute the necessary driver name string size */
//         if (driverType == SERVICE_RECOGNIZER_DRIVER || driverType == SERVICE_FILE_SYSTEM_DRIVER)
//             driverName.MaximumLength = sizeof(FILESYSTEM_ROOT_NAME);
//         else
//             driverName.MaximumLength = sizeof(DRIVER_ROOT_NAME);
//
//         driverName.MaximumLength += serviceName.Length;
//         driverName.Length = 0;
//
//         /* Allocate and build it */
//         driverName.Buffer = ExAllocatePoolWithTag(NonPagedPool, driverName.MaximumLength, TAG_IO);
//         if (!driverName.Buffer)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         if (driverType == SERVICE_RECOGNIZER_DRIVER || driverType == SERVICE_FILE_SYSTEM_DRIVER)
//             RtlAppendUnicodeToString(&driverName, FILESYSTEM_ROOT_NAME);
//         else
//             RtlAppendUnicodeToString(&driverName, DRIVER_ROOT_NAME);
//
//         RtlAppendUnicodeStringToString(&driverName, &serviceName);
//     }
//
//     if (ServiceName != NULL)
//     {
//         ASSERT(basicInfo); // Container for serviceName
//
//         /* Allocate a copy for the caller */
//         PWCHAR buf = ExAllocatePoolWithTag(PagedPool, serviceName.Length, TAG_IO);
//         if (!buf)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//         RtlMoveMemory(buf, serviceName.Buffer, serviceName.Length);
//         ServiceName->MaximumLength = serviceName.Length;
//         ServiceName->Length = serviceName.Length;
//         ServiceName->Buffer = buf;
//     }
//
//     *DriverName = driverName;
//     status = STATUS_SUCCESS;
//
// Cleanup:
//     if (basicInfo)
//         ExFreePoolWithTag(basicInfo, TAG_IO);
//
//     if (!NT_SUCCESS(status) && driverName.Buffer)
//         ExFreePoolWithTag(driverName.Buffer, TAG_IO);
//
//     return status;
// }
//
// /**
//  * @brief   Determines whether String1 may be a suffix of String2.
//  * @return  TRUE if String2 contains String1 as a suffix.
//  **/
// static BOOLEAN
// IopSuffixUnicodeString(
//     _In_ PCUNICODE_STRING String1,
//     _In_ PCUNICODE_STRING String2,
//     _In_ BOOLEAN CaseInSensitive)
// {
//     PWCHAR pc1, pc2;
//     ULONG NumChars;
//
//     if (String2->Length < String1->Length)
//         return FALSE;
//
//     NumChars = String1->Length / sizeof(WCHAR);
//     pc1 = String1->Buffer;
//     pc2 = &String2->Buffer[String2->Length / sizeof(WCHAR) - NumChars];
//
//     if (pc1 && pc2)
//     {
//         if (CaseInSensitive)
//         {
//             while (NumChars--)
//             {
//                 if (RtlUpcaseUnicodeChar(*pc1++) !=
//                     RtlUpcaseUnicodeChar(*pc2++))
//                 {
//                     return FALSE;
//                 }
//             }
//         }
//         else
//         {
//             while (NumChars--)
//             {
//                 if (*pc1++ != *pc2++)
//                     return FALSE;
//             }
//         }
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// /**
//  * @brief   Displays a driver-loading message in SOS mode.
//  **/
// static VOID
// FASTCALL
// IopDisplayLoadingMessage(
//     _In_ PCUNICODE_STRING ServiceName)
// {
//     extern BOOLEAN SosEnabled; // See ex/init.c
//     static const UNICODE_STRING DotSys = RTL_CONSTANT_STRING(L".SYS");
//     CHAR TextBuffer[256];
//
//     if (!SosEnabled) return;
//     if (!KeLoaderBlock) return;
//     RtlStringCbPrintfA(TextBuffer, sizeof(TextBuffer),
//                        "%s%sSystem32\\Drivers\\%wZ%s\r\n",
//                        KeLoaderBlock->ArcBootDeviceName,
//                        KeLoaderBlock->NtBootPathName,
//                        ServiceName,
//                        IopSuffixUnicodeString(&DotSys, ServiceName, TRUE)
//                            ? "" : ".SYS");
//     HalDisplayString(TextBuffer);
// }
//
// /*
//  * IopNormalizeImagePath
//  *
//  * Normalize an image path to contain complete path.
//  *
//  * Parameters
//  *    ImagePath
//  *       The input path and on exit the result path. ImagePath.Buffer
//  *       must be allocated by ExAllocatePool on input. Caller is responsible
//  *       for freeing the buffer when it's no longer needed.
//  *
//  *    ServiceName
//  *       Name of the service that ImagePath belongs to.
//  *
//  * Return Value
//  *    Status
//  *
//  * Remarks
//  *    The input image path isn't freed on error.
//  */
// NTSTATUS
// FASTCALL
// IopNormalizeImagePath(
//     _Inout_ _When_(return>=0, _At_(ImagePath->Buffer, _Post_notnull_ __drv_allocatesMem(Mem)))
//          PUNICODE_STRING ImagePath,
//     _In_ PUNICODE_STRING ServiceName)
// {
//     UNICODE_STRING SystemRootString = RTL_CONSTANT_STRING(L"\\SystemRoot\\");
//     UNICODE_STRING DriversPathString = RTL_CONSTANT_STRING(L"\\SystemRoot\\System32\\drivers\\");
//     UNICODE_STRING DotSysString = RTL_CONSTANT_STRING(L".sys");
//     UNICODE_STRING InputImagePath;
//
//     DPRINT("Normalizing image path '%wZ' for service '%wZ'\n", ImagePath, ServiceName);
//
//     InputImagePath = *ImagePath;
//     if (InputImagePath.Length == 0)
//     {
//         ImagePath->Length = 0;
//         ImagePath->MaximumLength = DriversPathString.Length +
//                                    ServiceName->Length +
//                                    DotSysString.Length +
//                                    sizeof(UNICODE_NULL);
//         ImagePath->Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                   ImagePath->MaximumLength,
//                                                   TAG_IO);
//         if (ImagePath->Buffer == NULL)
//             return STATUS_NO_MEMORY;
//
//         RtlCopyUnicodeString(ImagePath, &DriversPathString);
//         RtlAppendUnicodeStringToString(ImagePath, ServiceName);
//         RtlAppendUnicodeStringToString(ImagePath, &DotSysString);
//     }
//     else if (InputImagePath.Buffer[0] != L'\\')
//     {
//         ImagePath->Length = 0;
//         ImagePath->MaximumLength = SystemRootString.Length +
//                                    InputImagePath.Length +
//                                    sizeof(UNICODE_NULL);
//         ImagePath->Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                   ImagePath->MaximumLength,
//                                                   TAG_IO);
//         if (ImagePath->Buffer == NULL)
//             return STATUS_NO_MEMORY;
//
//         RtlCopyUnicodeString(ImagePath, &SystemRootString);
//         RtlAppendUnicodeStringToString(ImagePath, &InputImagePath);
//
//         /* Free caller's string */
//         ExFreePoolWithTag(InputImagePath.Buffer, TAG_RTLREGISTRY);
//     }
//
//     DPRINT("Normalized image path is '%wZ' for service '%wZ'\n", ImagePath, ServiceName);
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief      Initialize a loaded driver
//  *
//  * @param[in]  ModuleObject
//  *     Module object representing the driver. It can be retrieved by IopLoadServiceModule.
//  *     Freed on failure, so in a such case this should not be accessed anymore
//  *
//  * @param[in]  ServiceHandle
//  *     Handle to a driver's CCS/Services/<ServiceName> key
//  *
//  * @param[out] DriverObject
//  *     This contains the driver object if it was created (even with unsuccessfull result)
//  *
//  * @param[out] DriverEntryStatus
//  *     This contains the status value returned by the driver's DriverEntry routine
//  *     (will not be valid of the return value is not STATUS_SUCCESS or STATUS_FAILED_DRIVER_ENTRY)
//  *
//  * @return     Status of the operation
//  */
// NTSTATUS
// IopInitializeDriverModule(
//     _In_ PLDR_DATA_TABLE_ENTRY ModuleObject,
//     _In_ HANDLE ServiceHandle,
//     _Out_ PDRIVER_OBJECT *OutDriverObject,
//     _Out_ NTSTATUS *DriverEntryStatus)
// {
//     UNICODE_STRING DriverName, RegistryPath, ServiceName;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     Status = IopGetDriverNames(ServiceHandle, &DriverName, &ServiceName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopInitializeDriverModule: IopGetDriverNames failed 0x%x\n", Status);
//         MmUnloadSystemImage(ModuleObject);
//         return Status;
//     }
//
//     DPRINT("Driver name: '%wZ'\n", &DriverName);
//
//     /*
//      * Retrieve the driver's PE image NT header and perform some sanity checks.
//      * NOTE: We suppose that since the driver has been successfully loaded,
//      * its NT and optional headers are all valid and have expected sizes.
//      */
//     PIMAGE_NT_HEADERS NtHeaders = RtlImageNtHeader(ModuleObject->DllBase);
//     ASSERT(NtHeaders);
//     // NOTE: ModuleObject->SizeOfImage is actually (number of PTEs)*PAGE_SIZE.
//     ASSERT(ModuleObject->SizeOfImage == ROUND_TO_PAGES(NtHeaders->OptionalHeader.SizeOfImage));
//     ASSERT(ModuleObject->EntryPoint == RVA(ModuleObject->DllBase, NtHeaders->OptionalHeader.AddressOfEntryPoint));
//
//     /* Obtain the registry path for the DriverInit routine */
//     PKEY_NAME_INFORMATION nameInfo;
//     ULONG infoLength;
//     Status = ZwQueryKey(ServiceHandle, KeyNameInformation, NULL, 0, &infoLength);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         nameInfo = ExAllocatePoolWithTag(NonPagedPool, infoLength, TAG_IO);
//         if (nameInfo)
//         {
//             Status = ZwQueryKey(ServiceHandle,
//                                 KeyNameInformation,
//                                 nameInfo,
//                                 infoLength,
//                                 &infoLength);
//             if (NT_SUCCESS(Status))
//             {
//                 RegistryPath.Length = nameInfo->NameLength;
//                 RegistryPath.MaximumLength = nameInfo->NameLength;
//                 RegistryPath.Buffer = nameInfo->Name;
//             }
//             else
//             {
//                 ExFreePoolWithTag(nameInfo, TAG_IO);
//             }
//         }
//         else
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//     else
//     {
//         Status = NT_SUCCESS(Status) ? STATUS_UNSUCCESSFUL : Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeUnicodeString(&ServiceName);
//         RtlFreeUnicodeString(&DriverName);
//         MmUnloadSystemImage(ModuleObject);
//         return Status;
//     }
//
//     /* Create the driver object */
//     ULONG ObjectSize = sizeof(DRIVER_OBJECT) + sizeof(EXTENDED_DRIVER_EXTENSION);
//     OBJECT_ATTRIBUTES objAttrs;
//     PDRIVER_OBJECT driverObject;
//     InitializeObjectAttributes(&objAttrs,
//                                &DriverName,
//                                OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     Status = ObCreateObject(KernelMode,
//                             IoDriverObjectType,
//                             &objAttrs,
//                             KernelMode,
//                             NULL,
//                             ObjectSize,
//                             0,
//                             0,
//                             (PVOID*)&driverObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(nameInfo, TAG_IO); // container for RegistryPath
//         RtlFreeUnicodeString(&ServiceName);
//         RtlFreeUnicodeString(&DriverName);
//         MmUnloadSystemImage(ModuleObject);
//         DPRINT1("Error while creating driver object \"%wZ\" status %x\n", &DriverName, Status);
//         return Status;
//     }
//
//     DPRINT("Created driver object 0x%p for \"%wZ\"\n", driverObject, &DriverName);
//
//     RtlZeroMemory(driverObject, ObjectSize);
//     driverObject->Type = IO_TYPE_DRIVER;
//     driverObject->Size = sizeof(DRIVER_OBJECT);
//
//     /* Set the legacy flag if this is not a WDM driver */
//     if (!(NtHeaders->OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_WDM_DRIVER))
//         driverObject->Flags |= DRVO_LEGACY_DRIVER;
//
//     driverObject->DriverSection = ModuleObject;
//     driverObject->DriverStart = ModuleObject->DllBase;
//     driverObject->DriverSize = ModuleObject->SizeOfImage;
//     driverObject->DriverInit = ModuleObject->EntryPoint;
//     driverObject->HardwareDatabase = &IopHardwareDatabaseKey;
//     driverObject->DriverExtension = (PDRIVER_EXTENSION)(driverObject + 1);
//     driverObject->DriverExtension->DriverObject = driverObject;
//
//     /* Loop all Major Functions */
//     for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
//     {
//         /* Invalidate each function */
//         driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
//     }
//
//     /* Add the Object and get its handle */
//     HANDLE hDriver;
//     Status = ObInsertObject(driverObject, NULL, FILE_READ_DATA, 0, NULL, &hDriver);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(nameInfo, TAG_IO);
//         RtlFreeUnicodeString(&ServiceName);
//         RtlFreeUnicodeString(&DriverName);
//         return Status;
//     }
//
//     /* Now reference it */
//     Status = ObReferenceObjectByHandle(hDriver,
//                                        0,
//                                        IoDriverObjectType,
//                                        KernelMode,
//                                        (PVOID*)&driverObject,
//                                        NULL);
//
//     /* Close the extra handle */
//     ZwClose(hDriver);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(nameInfo, TAG_IO); // container for RegistryPath
//         RtlFreeUnicodeString(&ServiceName);
//         RtlFreeUnicodeString(&DriverName);
//         return Status;
//     }
//
//     /* Set up the service key name buffer */
//     UNICODE_STRING serviceKeyName;
//     serviceKeyName.Length = 0;
//     // NULL-terminate for Windows compatibility
//     serviceKeyName.MaximumLength = ServiceName.MaximumLength + sizeof(UNICODE_NULL);
//     serviceKeyName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                   serviceKeyName.MaximumLength,
//                                                   TAG_IO);
//     if (!serviceKeyName.Buffer)
//     {
//         ObMakeTemporaryObject(driverObject);
//         ObDereferenceObject(driverObject);
//         ExFreePoolWithTag(nameInfo, TAG_IO); // container for RegistryPath
//         RtlFreeUnicodeString(&ServiceName);
//         RtlFreeUnicodeString(&DriverName);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy the name and set it in the driver extension */
//     RtlCopyUnicodeString(&serviceKeyName, &ServiceName);
//     RtlFreeUnicodeString(&ServiceName);
//     driverObject->DriverExtension->ServiceKeyName = serviceKeyName;
//
//     /* Make a copy of the driver name to store in the driver object */
//     UNICODE_STRING driverNamePaged;
//     driverNamePaged.Length = 0;
//     // NULL-terminate for Windows compatibility
//     driverNamePaged.MaximumLength = DriverName.MaximumLength + sizeof(UNICODE_NULL);
//     driverNamePaged.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                    driverNamePaged.MaximumLength,
//                                                    TAG_IO);
//     if (!driverNamePaged.Buffer)
//     {
//         ObMakeTemporaryObject(driverObject);
//         ObDereferenceObject(driverObject);
//         ExFreePoolWithTag(nameInfo, TAG_IO); // container for RegistryPath
//         RtlFreeUnicodeString(&DriverName);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyUnicodeString(&driverNamePaged, &DriverName);
//     driverObject->DriverName = driverNamePaged;
//
//     /* Finally, call its init function */
//     Status = driverObject->DriverInit(driverObject, &RegistryPath);
//     *DriverEntryStatus = Status;
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("'%wZ' initialization failed, status (0x%08lx)\n", &DriverName, Status);
//         // return a special status value in case of failure
//         Status = STATUS_FAILED_DRIVER_ENTRY;
//     }
//
//     /* HACK: We're going to say if we don't have any DOs from DriverEntry, then we're not legacy.
//      * Other parts of the I/O manager depend on this behavior */
//     if (!driverObject->DeviceObject)
//     {
//         driverObject->Flags &= ~DRVO_LEGACY_DRIVER;
//     }
//
//     /* Windows does this fixup, keep it for compatibility */
//     for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
//     {
//         /*
//          * Make sure the driver didn't set any dispatch entry point to NULL!
//          * Doing so is illegal; drivers shouldn't touch entry points they
//          * do not implement.
//          */
//
//         /* Check if it did so anyway */
//         if (!driverObject->MajorFunction[i])
//         {
//             /* Print a warning in the debug log */
//             DPRINT1("Driver <%wZ> set DriverObject->MajorFunction[%lu] to NULL!\n",
//                     &driverObject->DriverName, i);
//
//             /* Fix it up */
//             driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
//         }
//     }
//
//     // TODO: for legacy drivers, unload the driver if it didn't create any DO
//
//     ExFreePoolWithTag(nameInfo, TAG_IO); // container for RegistryPath
//     RtlFreeUnicodeString(&DriverName);
//
//     if (!NT_SUCCESS(Status))
//     {
//         // if the driver entry has been failed, clear the object
//         ObMakeTemporaryObject(driverObject);
//         ObDereferenceObject(driverObject);
//         return Status;
//     }
//
//     *OutDriverObject = driverObject;
//
//     MmFreeDriverInitialization((PLDR_DATA_TABLE_ENTRY)driverObject->DriverSection);
//
//     /* Set the driver as initialized */
//     IopReadyDeviceObjects(driverObject);
//
//     if (PnpSystemInit) IopReinitializeDrivers();
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MiResolveImageReferences(IN PVOID ImageBase,
//                          IN PUNICODE_STRING ImageFileDirectory,
//                          IN PUNICODE_STRING NamePrefix OPTIONAL,
//                          OUT PCHAR *MissingApi,
//                          OUT PWCHAR *MissingDriver,
//                          OUT PLOAD_IMPORTS *LoadImports);
//
// //
// // Used for images already loaded (boot drivers)
// //
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// LdrProcessDriverModule(PLDR_DATA_TABLE_ENTRY LdrEntry,
//                        PUNICODE_STRING FileName,
//                        PLDR_DATA_TABLE_ENTRY *ModuleObject)
// {
//     NTSTATUS Status;
//     UNICODE_STRING BaseName, BaseDirectory;
//     PLOAD_IMPORTS LoadedImports = MM_SYSLDR_NO_IMPORTS;
//     PCHAR MissingApiName, Buffer;
//     PWCHAR MissingDriverName;
//     PVOID DriverBase = LdrEntry->DllBase;
//
//     /* Allocate a buffer we'll use for names */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                    MAXIMUM_FILENAME_LENGTH,
//                                    TAG_LDR_WSTR);
//     if (!Buffer)
//     {
//         /* Fail */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Check for a separator */
//     if (FileName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//     {
//         PWCHAR p;
//         ULONG BaseLength;
//
//         /* Loop the path until we get to the base name */
//         p = &FileName->Buffer[FileName->Length / sizeof(WCHAR)];
//         while (*(p - 1) != OBJ_NAME_PATH_SEPARATOR) p--;
//
//         /* Get the length */
//         BaseLength = (ULONG)(&FileName->Buffer[FileName->Length / sizeof(WCHAR)] - p);
//         BaseLength *= sizeof(WCHAR);
//
//         /* Setup the string */
//         BaseName.Length = (USHORT)BaseLength;
//         BaseName.Buffer = p;
//     }
//     else
//     {
//         /* Otherwise, we already have a base name */
//         BaseName.Length = FileName->Length;
//         BaseName.Buffer = FileName->Buffer;
//     }
//
//     /* Setup the maximum length */
//     BaseName.MaximumLength = BaseName.Length;
//
//     /* Now compute the base directory */
//     BaseDirectory = *FileName;
//     BaseDirectory.Length -= BaseName.Length;
//     BaseDirectory.MaximumLength = BaseDirectory.Length;
//
//     /* Resolve imports */
//     MissingApiName = Buffer;
//     Status = MiResolveImageReferences(DriverBase,
//                                       &BaseDirectory,
//                                       NULL,
//                                       &MissingApiName,
//                                       &MissingDriverName,
//                                       &LoadedImports);
//
//     /* Free the temporary buffer */
//     ExFreePoolWithTag(Buffer, TAG_LDR_WSTR);
//
//     /* Check the result of the imports resolution */
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Return */
//     *ModuleObject = LdrEntry;
//     return STATUS_SUCCESS;
// }
//
// PDEVICE_OBJECT
// IopGetDeviceObjectFromDeviceInstance(PUNICODE_STRING DeviceInstance);
//
// /*
//  * IopInitializeBuiltinDriver
//  *
//  * Initialize a driver that is already loaded in memory.
//  */
// CODE_SEG("INIT")
// static
// BOOLEAN
// IopInitializeBuiltinDriver(IN PLDR_DATA_TABLE_ENTRY BootLdrEntry)
// {
//     PDRIVER_OBJECT DriverObject;
//     NTSTATUS Status;
//     PWCHAR Buffer, FileNameWithoutPath;
//     PWSTR FileExtension;
//     PUNICODE_STRING ModuleName = &BootLdrEntry->BaseDllName;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY NextEntry;
//     UNICODE_STRING ServiceName;
//     BOOLEAN Success;
//
//     /*
//      * Display 'Loading XXX...' message
//      */
// #ifdef _M_ARM64
//     /* Print driver name to UART for boot debugging */
//     {
//         volatile ULONG *_uart = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_p = "[DRV] Loading ";
//         USHORT _i;
//         while (*_p) *_uart = (ULONG)*_p++;
//         for (_i = 0; _i < ModuleName->Length / sizeof(WCHAR); _i++)
//             *_uart = (ULONG)(ModuleName->Buffer[_i] & 0x7F);
//         _p = "\r\n";
//         while (*_p) *_uart = (ULONG)*_p++;
//     }
// #endif
//     IopDisplayLoadingMessage(ModuleName);
//     InbvIndicateProgress();
//
//     Buffer = ExAllocatePoolWithTag(PagedPool,
//                                    ModuleName->Length + sizeof(UNICODE_NULL),
//                                    TAG_IO);
//     if (Buffer == NULL)
//     {
//         return FALSE;
//     }
//
//     RtlCopyMemory(Buffer, ModuleName->Buffer, ModuleName->Length);
//     Buffer[ModuleName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /*
//      * Generate filename without path (not needed by freeldr)
//      */
//     FileNameWithoutPath = wcsrchr(Buffer, L'\\');
//     if (FileNameWithoutPath == NULL)
//     {
//         FileNameWithoutPath = Buffer;
//     }
//     else
//     {
//         FileNameWithoutPath++;
//     }
//
//     /*
//      * Strip the file extension from ServiceName
//      */
//     Success = RtlCreateUnicodeString(&ServiceName, FileNameWithoutPath);
//     ExFreePoolWithTag(Buffer, TAG_IO);
//     if (!Success)
//     {
//         return FALSE;
//     }
//
//     FileExtension = wcsrchr(ServiceName.Buffer, L'.');
//     if (FileExtension != NULL)
//     {
//         ServiceName.Length -= (USHORT)wcslen(FileExtension) * sizeof(WCHAR);
//         FileExtension[0] = UNICODE_NULL;
//     }
//
//     UNICODE_STRING RegistryPath;
//
//     // Make the registry path for the driver
//     RegistryPath.Length = 0;
//     RegistryPath.MaximumLength = sizeof(ServicesKeyName) + ServiceName.Length;
//     RegistryPath.Buffer = ExAllocatePoolWithTag(PagedPool, RegistryPath.MaximumLength, TAG_IO);
//     if (RegistryPath.Buffer == NULL)
//     {
//         return FALSE;
//     }
//     RtlAppendUnicodeToString(&RegistryPath, ServicesKeyName);
//     RtlAppendUnicodeStringToString(&RegistryPath, &ServiceName);
//     RtlFreeUnicodeString(&ServiceName);
//
//     HANDLE serviceHandle;
//     Status = IopOpenRegistryKeyEx(&serviceHandle, NULL, &RegistryPath, KEY_READ);
//     RtlFreeUnicodeString(&RegistryPath);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     /* Lookup the new Ldr entry in PsLoadedModuleList */
//     for (NextEntry = PsLoadedModuleList.Flink;
//          NextEntry != &PsLoadedModuleList;
//          NextEntry = NextEntry->Flink)
//     {
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//         if (RtlEqualUnicodeString(ModuleName, &LdrEntry->BaseDllName, TRUE))
//         {
//             break;
//         }
//     }
//     ASSERT(NextEntry != &PsLoadedModuleList);
//
//     /*
//      * Initialize the driver
//      */
//     NTSTATUS driverEntryStatus;
//     Status = IopInitializeDriverModule(LdrEntry,
//                                        serviceHandle,
//                                        &DriverObject,
//                                        &driverEntryStatus);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ARM64_UART_TRACE!("[DRV] DriverEntry FAILED\r\n");
//         DPRINT1("Driver '%wZ' load failed, status (%x)\n", ModuleName, Status);
//         return FALSE;
//     }
//     ARM64_UART_TRACE!("[DRV] DriverEntry OK\r\n");
//
//     // The driver has been loaded, now check if where are any PDOs
//     // for that driver, and queue AddDevice call for them.
//     // The check is possible because HKLM/SYSTEM/CCS/Services/<ServiceName>/Enum directory
//     // is populated upon a new device arrival based on a (critical) device database
//
//     // Legacy drivers may add devices inside DriverEntry.
//     // We're lazy and always assume that they are doing so
//     BOOLEAN deviceAdded = !!(DriverObject->Flags & DRVO_LEGACY_DRIVER);
//
//     HANDLE enumServiceHandle;
//     UNICODE_STRING enumName = RTL_CONSTANT_STRING(L"Enum");
//
//     Status = IopOpenRegistryKeyEx(&enumServiceHandle, serviceHandle, &enumName, KEY_READ);
//     ZwClose(serviceHandle);
//
//     if (NT_SUCCESS(Status))
//     {
//         ULONG instanceCount = 0;
//         PKEY_VALUE_FULL_INFORMATION kvInfo;
//         Status = IopGetRegistryValue(enumServiceHandle, L"Count", &kvInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//         if (kvInfo->Type != REG_DWORD || kvInfo->DataLength != sizeof(ULONG))
//         {
//             ExFreePool(kvInfo);
//             goto Cleanup;
//         }
//
//         RtlMoveMemory(&instanceCount,
//                       (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                       sizeof(ULONG));
//         ExFreePool(kvInfo);
//
//         DPRINT("Processing %u instances for %wZ module\n", instanceCount, ModuleName);
//
//         for (ULONG i = 0; i < instanceCount; i++)
//         {
//             WCHAR num[11];
//             UNICODE_STRING instancePath;
//             RtlStringCbPrintfW(num, sizeof(num), L"%u", i);
//
//             Status = IopGetRegistryValue(enumServiceHandle, num, &kvInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 continue;
//             }
//             if ((kvInfo->Type != REG_SZ) ||
//                 (kvInfo->DataLength < sizeof(UNICODE_NULL)) ||
//                 (kvInfo->DataLength > UNICODE_STRING_MAX_BYTES) ||
//                 ((kvInfo->DataLength % sizeof(WCHAR)) != 0))
//             {
//                 DPRINT1("ObjectName invalid (Type = %lu, DataLength = %lu)\n",
//                         kvInfo->Type,
//                         kvInfo->DataLength);
//                 ExFreePool(kvInfo);
//                 continue;
//             }
//
//             instancePath.Length = (USHORT)(kvInfo->DataLength - sizeof(UNICODE_NULL));
//             instancePath.MaximumLength = kvInfo->DataLength;
//             instancePath.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                         instancePath.MaximumLength,
//                                                         TAG_IO);
//             if (instancePath.Buffer)
//             {
//                 RtlMoveMemory(instancePath.Buffer,
//                               (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                               instancePath.Length);
//                 instancePath.Buffer[instancePath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//                 PDEVICE_OBJECT pdo = IopGetDeviceObjectFromDeviceInstance(&instancePath);
//                 if (pdo != NULL)
//                 {
//                     PiQueueDeviceAction(pdo, PiActionAddBootDevices, NULL, NULL);
//                     ObDereferenceObject(pdo);
//                     deviceAdded = TRUE;
//                 }
//                 else
//                 {
//                     DPRINT1("No device node found matching instance path '%wZ'\n", &instancePath);
//                 }
//             }
//
//             ExFreePool(kvInfo);
//         }
//
//         ZwClose(enumServiceHandle);
//     }
// Cleanup:
//     /* Remove extra reference from IopInitializeDriverModule */
//     ObDereferenceObject(DriverObject);
//
//     return deviceAdded;
// }
//
// /*
//  * IopInitializeBootDrivers
//  *
//  * Initialize boot drivers and free memory for boot files.
//  *
//  * Parameters
//  *    None
//  *
//  * Return Value
//  *    None
//  */
// CODE_SEG("INIT")
// VOID
// FASTCALL
// IopInitializeBootDrivers(VOID)
// {
//     PLIST_ENTRY ListHead, NextEntry, NextEntry2;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     NTSTATUS Status;
//     UNICODE_STRING DriverName;
//     ULONG i, Index;
//     PDRIVER_INFORMATION DriverInfo, DriverInfoTag;
//     HANDLE KeyHandle;
//     PBOOT_DRIVER_LIST_ENTRY BootEntry;
//     DPRINT("IopInitializeBootDrivers()\n");
//
//     /* Create the RAW FS built-in driver */
//     RtlInitUnicodeString(&DriverName, L"\\FileSystem\\RAW");
//
//     Status = IoCreateDriver(&DriverName, RawFsDriverEntry);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         return;
//     }
//
//     /* Get highest group order index */
//     IopGroupIndex = PpInitGetGroupOrderIndex(NULL);
//     if (IopGroupIndex == 0xFFFF)
//     {
//         UNIMPLEMENTED_DBGBREAK();
//     }
//
//     /* Allocate the group table */
//     IopGroupTable = ExAllocatePoolWithTag(PagedPool,
//                                           IopGroupIndex * sizeof(LIST_ENTRY),
//                                           TAG_IO);
//     if (IopGroupTable == NULL)
//     {
//         UNIMPLEMENTED_DBGBREAK();
//     }
//
//     /* Initialize the group table lists */
//     for (i = 0; i < IopGroupIndex; i++) InitializeListHead(&IopGroupTable[i]);
//
//     /* Loop the boot modules */
//     ListHead = &KeLoaderBlock->LoadOrderListHead;
//     for (NextEntry = ListHead->Flink;
//          NextEntry != ListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Check if the DLL needs to be initialized */
//         if (LdrEntry->Flags & LDRP_DRIVER_DEPENDENT_DLL)
//         {
//             /* Call its entrypoint */
//             MmCallDllInitialize(LdrEntry, NULL);
//         }
//     }
//
//     /* Loop the boot drivers */
//     ListHead = &KeLoaderBlock->BootDriverListHead;
//     for (NextEntry = ListHead->Flink;
//          NextEntry != ListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the entry */
//         BootEntry = CONTAINING_RECORD(NextEntry,
//                                       BOOT_DRIVER_LIST_ENTRY,
//                                       Link);
//
//         // FIXME: TODO: This LdrEntry is to be used in a special handling
//         // for SETUPLDR (a similar procedure is done on Windows), where
//         // the loader would, under certain conditions, be loaded in the
//         // SETUPLDR-specific code block below...
// #if 0
//         /* Get the driver loader entry */
//         LdrEntry = BootEntry->LdrEntry;
// #endif
//
//         /* Allocate our internal accounting structure */
//         DriverInfo = ExAllocatePoolWithTag(PagedPool,
//                                            sizeof(DRIVER_INFORMATION),
//                                            TAG_IO);
//         if (DriverInfo)
//         {
//             /* Zero it and initialize it */
//             RtlZeroMemory(DriverInfo, sizeof(DRIVER_INFORMATION));
//             InitializeListHead(&DriverInfo->Link);
//             DriverInfo->DataTableEntry = BootEntry;
//
//             /* Open the registry key */
//             Status = IopOpenRegistryKeyEx(&KeyHandle,
//                                           NULL,
//                                           &BootEntry->RegistryPath,
//                                           KEY_READ);
//             DPRINT("IopOpenRegistryKeyEx(%wZ) returned 0x%08lx\n", &BootEntry->RegistryPath, Status);
// #if 0
//             if (NT_SUCCESS(Status))
// #else // Hack still needed...
//             if ((NT_SUCCESS(Status)) || /* ReactOS HACK for SETUPLDR */
//                 ((KeLoaderBlock->SetupLdrBlock) && ((KeyHandle = (PVOID)1)))) // yes, it's an assignment!
// #endif
//             {
//                 /* Save the handle */
//                 DriverInfo->ServiceHandle = KeyHandle;
//
//                 /* Get the group oder index */
//                 Index = PpInitGetGroupOrderIndex(KeyHandle);
//
//                 /* Get the tag position */
//                 DriverInfo->TagPosition = PipGetDriverTagPriority(KeyHandle);
//
//                 /* Insert it into the list, at the right place */
//                 ASSERT(Index < IopGroupIndex);
//                 NextEntry2 = IopGroupTable[Index].Flink;
//                 while (NextEntry2 != &IopGroupTable[Index])
//                 {
//                     /* Get the driver info */
//                     DriverInfoTag = CONTAINING_RECORD(NextEntry2,
//                                                       DRIVER_INFORMATION,
//                                                       Link);
//
//                     /* Check if we found the right tag position */
//                     if (DriverInfoTag->TagPosition > DriverInfo->TagPosition)
//                     {
//                         /* We're done */
//                         break;
//                     }
//
//                     /* Next entry */
//                     NextEntry2 = NextEntry2->Flink;
//                 }
//
//                 /* Insert us right before the next entry */
//                 NextEntry2 = NextEntry2->Blink;
//                 InsertHeadList(NextEntry2, &DriverInfo->Link);
//             }
//         }
//     }
//
//     /* Loop each group index */
//     for (i = 0; i < IopGroupIndex; i++)
//     {
//         /* Loop each group table */
//         for (NextEntry = IopGroupTable[i].Flink;
//              NextEntry != &IopGroupTable[i];
//              NextEntry = NextEntry->Flink)
//         {
//             /* Get the entry */
//             DriverInfo = CONTAINING_RECORD(NextEntry,
//                                            DRIVER_INFORMATION,
//                                            Link);
//
//             /* Get the driver loader entry */
//             LdrEntry = DriverInfo->DataTableEntry->LdrEntry;
//
//             /* Log boot driver init to serial (DbgPrint) */
//             {
//                 static ULONG _drv_count = 0;
//                 DbgPrint("[BOOT] Init driver %lu: %wZ\n", _drv_count, &LdrEntry->BaseDllName);
//                 _drv_count++;
//             }
//
//             /* Initialize it */
//             if (IopInitializeBuiltinDriver(LdrEntry))
//             {
//                 // it does not make sense to enumerate the tree if there are no new devices added
//                 PiQueueDeviceAction(IopRootDeviceNode->PhysicalDeviceObject,
//                                     PiActionEnumRootDevices,
//                                     NULL,
//                                     NULL);
//             }
//         }
//     }
//
//     /* HAL Root Bus is being initialized before loading the boot drivers so this may cause issues
//      * when some devices are not being initialized with their drivers. This flag is used to delay
//      * all actions with devices (except PnP root device) until boot drivers are loaded.
//      * See PiQueueDeviceAction function
//      */
//     PnPBootDriversLoaded = TRUE;
//
//     DbgPrint("BOOT DRIVERS LOADED\n");
//
//     PiQueueDeviceAction(IopRootDeviceNode->PhysicalDeviceObject,
//                         PiActionEnumDeviceTree,
//                         NULL,
//                         NULL);
// }
//
// CODE_SEG("INIT")
// VOID
// FASTCALL
// IopInitializeSystemDrivers(VOID)
// {
//     PUNICODE_STRING *DriverList, *SavedList;
//
//     PiPerformSyncDeviceAction(IopRootDeviceNode->PhysicalDeviceObject, PiActionEnumDeviceTree);
//
//     /* HACK: No system drivers on the BootCD */
//     if (KeLoaderBlock->SetupLdrBlock)
//     {
//         DPRINT1("IopInitializeSystemDrivers: SetupLdrBlock is set, skipping system drivers\n");
//         return;
//     }
//
// #ifdef _M_ARM64
//     /* ARM64: Skip loading system-start drivers from the filesystem.
//      * ZwLoadDriver hangs when trying to load drivers that don't exist
//      * on the ramdisk. All critical drivers are boot-start. */
//     ARM64_UART_TRACE!("[SYSDRV] ARM64: Skipping system-start drivers\r\n");
//     return;
// #endif
//
//     /* Get the driver list */
//     SavedList = DriverList = CmGetSystemDriverList();
//     ASSERT(DriverList);
//
//     DPRINT1("IopInitializeSystemDrivers: loading system-start drivers\n");
//
//     ARM64_UART_TRACE!("[SYSDRV] Loading system drivers...\r\n");
//
//     /* Loop it */
//     while (*DriverList)
//     {
//         DPRINT1("  SysDrv: %wZ\n", *DriverList);
// #if defined(_M_ARM64) && defined(ARM64_UART_TRACE_ENABLE)
//         {
//             volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//             const char *_m = "[SYSDRV] ZwLoadDriver: ";
//             while (*_m) { *_u = (ULONG)*_m++; }
//             /* Print the driver name (Unicode to ASCII) */
//             if (*DriverList && (*DriverList)->Buffer)
//             {
//                 USHORT i;
//                 for (i = 0; i < (*DriverList)->Length / sizeof(WCHAR); i++)
//                     *_u = (ULONG)(UCHAR)(*DriverList)->Buffer[i];
//             }
//             _m = "\r\n";
//             while (*_m) { *_u = (ULONG)*_m++; }
//         }
// #endif
//
//         /* Load the driver */
//         {
//             NTSTATUS _st = ZwLoadDriver(*DriverList);
//             DPRINT1("  SysDrv: %wZ => 0x%x\n", *DriverList, _st);
//         }
//
//         /* Free the entry */
//         RtlFreeUnicodeString(*DriverList);
//         ExFreePool(*DriverList);
//
//         /* Next entry */
//         InbvIndicateProgress();
//         DriverList++;
//     }
//
//     ARM64_UART_TRACE!("[SYSDRV] All system drivers loaded\r\n");
//
//     /* Free the list */
//     ExFreePool(SavedList);
//
//     PiQueueDeviceAction(IopRootDeviceNode->PhysicalDeviceObject,
//                         PiActionEnumDeviceTree,
//                         NULL,
//                         NULL);
//
//     ARM64_UART_TRACE!("[SYSDRV] IopInitializeSystemDrivers complete\r\n");
// }
//
// /*
//  * IopUnloadDriver
//  *
//  * Unloads a device driver.
//  *
//  * Parameters
//  *    DriverServiceName
//  *       Name of the service to unload (registry key).
//  *
//  *    UnloadPnpDrivers
//  *       Whether to unload Plug & Plug or only legacy drivers. If this
//  *       parameter is set to FALSE, the routine will unload only legacy
//  *       drivers.
//  *
//  * Return Value
//  *    Status
//  *
//  * To do
//  *    Guard the whole function by SEH.
//  */
//
// NTSTATUS NTAPI
// IopUnloadDriver(PUNICODE_STRING DriverServiceName, BOOLEAN UnloadPnpDrivers)
// {
//     UNICODE_STRING Backslash = RTL_CONSTANT_STRING(L"\\");
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2];
//     UNICODE_STRING ImagePath;
//     UNICODE_STRING ServiceName;
//     UNICODE_STRING ObjectName;
//     PDRIVER_OBJECT DriverObject;
//     PDEVICE_OBJECT DeviceObject;
//     PEXTENDED_DEVOBJ_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     USHORT LastBackslash;
//     BOOLEAN SafeToUnload = TRUE;
//     KPROCESSOR_MODE PreviousMode;
//     UNICODE_STRING CapturedServiceName;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* Need the appropriate priviliege */
//     if (!SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode))
//     {
//         DPRINT1("No unload privilege!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Capture the service name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedServiceName,
//                                           PreviousMode,
//                                           DriverServiceName);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DPRINT("IopUnloadDriver('%wZ', %u)\n", &CapturedServiceName, UnloadPnpDrivers);
//
//     /* We need a service name */
//     if (CapturedServiceName.Length == 0 || CapturedServiceName.Buffer == NULL)
//     {
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /*
//      * Get the service name from the registry key name
//      */
//     Status = RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
//                                         &CapturedServiceName,
//                                         &Backslash,
//                                         &LastBackslash);
//     if (NT_SUCCESS(Status))
//     {
//         NT_ASSERT(CapturedServiceName.Length >= LastBackslash + sizeof(WCHAR));
//         ServiceName.Buffer = &CapturedServiceName.Buffer[LastBackslash / sizeof(WCHAR) + 1];
//         ServiceName.Length = CapturedServiceName.Length - LastBackslash - sizeof(WCHAR);
//         ServiceName.MaximumLength = CapturedServiceName.MaximumLength - LastBackslash - sizeof(WCHAR);
//     }
//     else
//     {
//         ServiceName = CapturedServiceName;
//     }
//
//     /*
//      * Construct the driver object name
//      */
//     Status = RtlUShortAdd(sizeof(DRIVER_ROOT_NAME),
//                           ServiceName.Length,
//                           &ObjectName.MaximumLength);
//     if (!NT_SUCCESS(Status))
//     {
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return Status;
//     }
//     ObjectName.Length = 0;
//     ObjectName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                               ObjectName.MaximumLength,
//                                               TAG_IO);
//     if (!ObjectName.Buffer)
//     {
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     NT_VERIFY(NT_SUCCESS(RtlAppendUnicodeToString(&ObjectName, DRIVER_ROOT_NAME)));
//     NT_VERIFY(NT_SUCCESS(RtlAppendUnicodeStringToString(&ObjectName, &ServiceName)));
//
//     /*
//      * Find the driver object
//      */
//     Status = ObReferenceObjectByName(&ObjectName,
//                                      0,
//                                      0,
//                                      0,
//                                      IoDriverObjectType,
//                                      KernelMode,
//                                      0,
//                                      (PVOID*)&DriverObject);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Can't locate driver object for %wZ\n", &ObjectName);
//         ExFreePoolWithTag(ObjectName.Buffer, TAG_IO);
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return Status;
//     }
//
//     /* Free the buffer for driver object name */
//     ExFreePoolWithTag(ObjectName.Buffer, TAG_IO);
//
//     /* Check that driver is not already unloading */
//     if (DriverObject->Flags & DRVO_UNLOAD_INVOKED)
//     {
//         DPRINT1("Driver deletion pending\n");
//         ObDereferenceObject(DriverObject);
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return STATUS_DELETE_PENDING;
//     }
//
//     /*
//      * Get path of service...
//      */
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//
//     RtlInitUnicodeString(&ImagePath, NULL);
//
//     QueryTable[0].Name = L"ImagePath";
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
//     QueryTable[0].EntryContext = &ImagePath;
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                     CapturedServiceName.Buffer,
//                                     QueryTable,
//                                     NULL,
//                                     NULL);
//
//     /* We no longer need service name */
//     ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlQueryRegistryValues() failed (Status %x)\n", Status);
//         ObDereferenceObject(DriverObject);
//         return Status;
//     }
//
//     /*
//      * Normalize the image path for all later processing.
//      */
//     Status = IopNormalizeImagePath(&ImagePath, &ServiceName);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopNormalizeImagePath() failed (Status %x)\n", Status);
//         ObDereferenceObject(DriverObject);
//         return Status;
//     }
//
//     /* Free the service path */
//     ExFreePool(ImagePath.Buffer);
//
//     /*
//      * Unload the module and release the references to the device object
//      */
//
//     /* Call the load/unload routine, depending on current process */
//     if (DriverObject->DriverUnload && DriverObject->DriverSection &&
//         (UnloadPnpDrivers || (DriverObject->Flags & DRVO_LEGACY_DRIVER)))
//     {
//         /* Loop through each device object of the driver
//            and set DOE_UNLOAD_PENDING flag */
//         DeviceObject = DriverObject->DeviceObject;
//         while (DeviceObject)
//         {
//             /* Set the unload pending flag for the device */
//             DeviceExtension = IoGetDevObjExtension(DeviceObject);
//             DeviceExtension->ExtensionFlags |= DOE_UNLOAD_PENDING;
//
//             /* Make sure there are no attached devices or no reference counts */
//             if ((DeviceObject->ReferenceCount) || (DeviceObject->AttachedDevice))
//             {
//                 /* Not safe to unload */
//                 DPRINT1("Drivers device object is referenced or has attached devices\n");
//
//                 SafeToUnload = FALSE;
//             }
//
//             DeviceObject = DeviceObject->NextDevice;
//         }
//
//         /* If not safe to unload, then return success */
//         if (!SafeToUnload)
//         {
//             ObDereferenceObject(DriverObject);
//             return STATUS_SUCCESS;
//         }
//
//         DPRINT1("Unloading driver '%wZ' (manual)\n", &DriverObject->DriverName);
//
//         /* Set the unload invoked flag and call the unload routine */
//         DriverObject->Flags |= DRVO_UNLOAD_INVOKED;
//         Status = IopDoLoadUnloadDriver(NULL, &DriverObject);
//         ASSERT(Status == STATUS_SUCCESS);
//
//         /* Mark the driver object temporary, so it could be deleted later */
//         ObMakeTemporaryObject(DriverObject);
//
//         /* Dereference it 2 times */
//         ObDereferenceObject(DriverObject);
//         ObDereferenceObject(DriverObject);
//
//         return Status;
//     }
//     else
//     {
//         DPRINT1("No DriverUnload function! '%wZ' will not be unloaded!\n", &DriverObject->DriverName);
//
//         /* Dereference one time (refd inside this function) */
//         ObDereferenceObject(DriverObject);
//
//         /* Return unloading failure */
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
// }
//
// VOID
// NTAPI
// IopReinitializeDrivers(VOID)
// {
//     PDRIVER_REINIT_ITEM ReinitItem;
//     PLIST_ENTRY Entry;
//
//     /* Get the first entry and start looping */
//     Entry = ExInterlockedRemoveHeadList(&DriverReinitListHead,
//                                         &DriverReinitListLock);
//     while (Entry)
//     {
//         /* Get the item */
//         ReinitItem = CONTAINING_RECORD(Entry, DRIVER_REINIT_ITEM, ItemEntry);
//
//         /* Increment reinitialization counter */
//         ReinitItem->DriverObject->DriverExtension->Count++;
//
//         /* Remove the device object flag */
//         ReinitItem->DriverObject->Flags &= ~DRVO_REINIT_REGISTERED;
//
//         /* Call the routine */
//         ReinitItem->ReinitRoutine(ReinitItem->DriverObject,
//                                   ReinitItem->Context,
//                                   ReinitItem->DriverObject->
//                                   DriverExtension->Count);
//
//         /* Free the entry */
//         ExFreePool(Entry);
//
//         /* Move to the next one */
//         Entry = ExInterlockedRemoveHeadList(&DriverReinitListHead,
//                                             &DriverReinitListLock);
//     }
// }
//
// VOID
// NTAPI
// IopReinitializeBootDrivers(VOID)
// {
//     PDRIVER_REINIT_ITEM ReinitItem;
//     PLIST_ENTRY Entry;
//
//     /* Get the first entry and start looping */
//     Entry = ExInterlockedRemoveHeadList(&DriverBootReinitListHead,
//                                         &DriverBootReinitListLock);
//     while (Entry)
//     {
//         /* Get the item */
//         ReinitItem = CONTAINING_RECORD(Entry, DRIVER_REINIT_ITEM, ItemEntry);
//
//         /* Increment reinitialization counter */
//         ReinitItem->DriverObject->DriverExtension->Count++;
//
//         /* Remove the device object flag */
//         ReinitItem->DriverObject->Flags &= ~DRVO_BOOTREINIT_REGISTERED;
//
//         /* Call the routine */
//         ReinitItem->ReinitRoutine(ReinitItem->DriverObject,
//                                   ReinitItem->Context,
//                                   ReinitItem->DriverObject->
//                                   DriverExtension->Count);
//
//         /* Free the entry */
//         ExFreePool(Entry);
//
//         /* Move to the next one */
//         Entry = ExInterlockedRemoveHeadList(&DriverBootReinitListHead,
//                                             &DriverBootReinitListLock);
//     }
//
//     /* Wait for all device actions being finished*/
//     KeWaitForSingleObject(&PiEnumerationFinished, Executive, KernelMode, FALSE, NULL);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateDriver(
//     _In_opt_ PUNICODE_STRING DriverName,
//     _In_ PDRIVER_INITIALIZE InitializationFunction)
// {
//     WCHAR NameBuffer[100];
//     USHORT NameLength;
//     UNICODE_STRING LocalDriverName;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG ObjectSize;
//     PDRIVER_OBJECT DriverObject;
//     UNICODE_STRING ServiceKeyName;
//     HANDLE hDriver;
//     ULONG i, RetryCount = 0;
//
// try_again:
//     /* First, create a unique name for the driver if we don't have one */
//     if (!DriverName)
//     {
//         /* Create a random name and set up the string */
//         NameLength = (USHORT)swprintf(NameBuffer,
//                                       DRIVER_ROOT_NAME L"%08u",
//                                       KeTickCount.LowPart);
//         LocalDriverName.Length = NameLength * sizeof(WCHAR);
//         LocalDriverName.MaximumLength = LocalDriverName.Length + sizeof(UNICODE_NULL);
//         LocalDriverName.Buffer = NameBuffer;
//     }
//     else
//     {
//         /* So we can avoid another code path, use a local var */
//         LocalDriverName = *DriverName;
//     }
//
//     /* Initialize the Attributes */
//     ObjectSize = sizeof(DRIVER_OBJECT) + sizeof(EXTENDED_DRIVER_EXTENSION);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LocalDriverName,
//                                OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Create the Object */
//     Status = ObCreateObject(KernelMode,
//                             IoDriverObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             ObjectSize,
//                             0,
//                             0,
//                             (PVOID*)&DriverObject);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     DPRINT("IopCreateDriver(): created DO %p\n", DriverObject);
//
//     /* Set up the Object */
//     RtlZeroMemory(DriverObject, ObjectSize);
//     DriverObject->Type = IO_TYPE_DRIVER;
//     DriverObject->Size = sizeof(DRIVER_OBJECT);
//     DriverObject->Flags = DRVO_BUILTIN_DRIVER;
//     DriverObject->DriverExtension = (PDRIVER_EXTENSION)(DriverObject + 1);
//     DriverObject->DriverExtension->DriverObject = DriverObject;
//     DriverObject->DriverInit = InitializationFunction;
//     /* Loop all Major Functions */
//     for (i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
//     {
//         /* Invalidate each function */
//         DriverObject->MajorFunction[i] = IopInvalidDeviceRequest;
//     }
//
//     /* Set up the service key name buffer */
//     ServiceKeyName.MaximumLength = LocalDriverName.Length + sizeof(UNICODE_NULL);
//     ServiceKeyName.Buffer = ExAllocatePoolWithTag(PagedPool, LocalDriverName.MaximumLength, TAG_IO);
//     if (!ServiceKeyName.Buffer)
//     {
//         /* Fail */
//         ObMakeTemporaryObject(DriverObject);
//         ObDereferenceObject(DriverObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* For builtin drivers, the ServiceKeyName is equal to DriverName */
//     RtlCopyUnicodeString(&ServiceKeyName, &LocalDriverName);
//     ServiceKeyName.Buffer[ServiceKeyName.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     DriverObject->DriverExtension->ServiceKeyName = ServiceKeyName;
//
//     /* Make a copy of the driver name to store in the driver object */
//     DriverObject->DriverName.MaximumLength = LocalDriverName.Length;
//     DriverObject->DriverName.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                             DriverObject->DriverName.MaximumLength,
//                                                             TAG_IO);
//     if (!DriverObject->DriverName.Buffer)
//     {
//         /* Fail */
//         ObMakeTemporaryObject(DriverObject);
//         ObDereferenceObject(DriverObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlCopyUnicodeString(&DriverObject->DriverName, &LocalDriverName);
//
//     /* Add the Object and get its handle */
//     Status = ObInsertObject(DriverObject,
//                             NULL,
//                             FILE_READ_DATA,
//                             0,
//                             NULL,
//                             &hDriver);
//
//     /* Eliminate small possibility when this function is called more than
//        once in a row, and KeTickCount doesn't get enough time to change */
//     if (!DriverName && (Status == STATUS_OBJECT_NAME_COLLISION) && (RetryCount < 100))
//     {
//         RetryCount++;
//         goto try_again;
//     }
//
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Now reference it */
//     Status = ObReferenceObjectByHandle(hDriver,
//                                        0,
//                                        IoDriverObjectType,
//                                        KernelMode,
//                                        (PVOID*)&DriverObject,
//                                        NULL);
//
//     /* Close the extra handle */
//     ZwClose(hDriver);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         ObMakeTemporaryObject(DriverObject);
//         ObDereferenceObject(DriverObject);
//         return Status;
//     }
//
//     /* Finally, call its init function */
//     DPRINT("Calling driver entrypoint at %p\n", InitializationFunction);
//     Status = InitializationFunction(DriverObject, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If it didn't work, then kill the object */
//         DPRINT1("'%wZ' initialization failed, status (0x%08lx)\n", &LocalDriverName, Status);
//         ObMakeTemporaryObject(DriverObject);
//         ObDereferenceObject(DriverObject);
//         return Status;
//     }
//
//     /* Windows does this fixup, keep it for compatibility */
//     for (i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
//     {
//         /*
//          * Make sure the driver didn't set any dispatch entry point to NULL!
//          * Doing so is illegal; drivers shouldn't touch entry points they
//          * do not implement.
//          */
//
//         /* Check if it did so anyway */
//         if (!DriverObject->MajorFunction[i])
//         {
//             /* Print a warning in the debug log */
//             DPRINT1("Driver <%wZ> set DriverObject->MajorFunction[%lu] to NULL!\n",
//                     &DriverObject->DriverName, i);
//
//             /* Fix it up */
//             DriverObject->MajorFunction[i] = IopInvalidDeviceRequest;
//         }
//     }
//
//     /* Return the Status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoDeleteDriver(
//     _In_ PDRIVER_OBJECT DriverObject)
// {
//     /* Simply dereference the Object */
//     ObDereferenceObject(DriverObject);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRegisterBootDriverReinitialization(IN PDRIVER_OBJECT DriverObject,
//                                      IN PDRIVER_REINITIALIZE ReinitRoutine,
//                                      IN PVOID Context)
// {
//     PDRIVER_REINIT_ITEM ReinitItem;
//
//     /* Allocate the entry */
//     ReinitItem = ExAllocatePoolWithTag(NonPagedPool,
//                                        sizeof(DRIVER_REINIT_ITEM),
//                                        TAG_REINIT);
//     if (!ReinitItem) return;
//
//     /* Fill it out */
//     ReinitItem->DriverObject = DriverObject;
//     ReinitItem->ReinitRoutine = ReinitRoutine;
//     ReinitItem->Context = Context;
//
//     /* Set the Driver Object flag and insert the entry into the list */
//     DriverObject->Flags |= DRVO_BOOTREINIT_REGISTERED;
//     ExInterlockedInsertTailList(&DriverBootReinitListHead,
//                                 &ReinitItem->ItemEntry,
//                                 &DriverBootReinitListLock);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRegisterDriverReinitialization(IN PDRIVER_OBJECT DriverObject,
//                                  IN PDRIVER_REINITIALIZE ReinitRoutine,
//                                  IN PVOID Context)
// {
//     PDRIVER_REINIT_ITEM ReinitItem;
//
//     /* Allocate the entry */
//     ReinitItem = ExAllocatePoolWithTag(NonPagedPool,
//                                        sizeof(DRIVER_REINIT_ITEM),
//                                        TAG_REINIT);
//     if (!ReinitItem) return;
//
//     /* Fill it out */
//     ReinitItem->DriverObject = DriverObject;
//     ReinitItem->ReinitRoutine = ReinitRoutine;
//     ReinitItem->Context = Context;
//
//     /* Set the Driver Object flag and insert the entry into the list */
//     DriverObject->Flags |= DRVO_REINIT_REGISTERED;
//     ExInterlockedInsertTailList(&DriverReinitListHead,
//                                 &ReinitItem->ItemEntry,
//                                 &DriverReinitListLock);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoAllocateDriverObjectExtension(IN PDRIVER_OBJECT DriverObject,
//                                 IN PVOID ClientIdentificationAddress,
//                                 IN ULONG DriverObjectExtensionSize,
//                                 OUT PVOID *DriverObjectExtension)
// {
//     KIRQL OldIrql;
//     PIO_CLIENT_EXTENSION DriverExtensions, NewDriverExtension;
//     BOOLEAN Inserted = FALSE;
//
//     /* Assume failure */
//     *DriverObjectExtension = NULL;
//
//     /* Allocate the extension */
//     NewDriverExtension = ExAllocatePoolWithTag(NonPagedPool,
//                                                sizeof(IO_CLIENT_EXTENSION) +
//                                                DriverObjectExtensionSize,
//                                                TAG_DRIVER_EXTENSION);
//     if (!NewDriverExtension) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Clear the extension for teh caller */
//     RtlZeroMemory(NewDriverExtension,
//                   sizeof(IO_CLIENT_EXTENSION) + DriverObjectExtensionSize);
//
//     /* Acqure lock */
//     OldIrql = KeRaiseIrqlToDpcLevel();
//
//     /* Fill out the extension */
//     NewDriverExtension->ClientIdentificationAddress = ClientIdentificationAddress;
//
//     /* Loop the current extensions */
//     DriverExtensions = IoGetDrvObjExtension(DriverObject)->
//                        ClientDriverExtension;
//     while (DriverExtensions)
//     {
//         /* Check if the identifier matches */
//         if (DriverExtensions->ClientIdentificationAddress ==
//             ClientIdentificationAddress)
//         {
//             /* We have a collision, break out */
//             break;
//         }
//
//         /* Go to the next one */
//         DriverExtensions = DriverExtensions->NextExtension;
//     }
//
//     /* Check if we didn't collide */
//     if (!DriverExtensions)
//     {
//         /* Link this one in */
//         NewDriverExtension->NextExtension =
//             IoGetDrvObjExtension(DriverObject)->ClientDriverExtension;
//         IoGetDrvObjExtension(DriverObject)->ClientDriverExtension =
//             NewDriverExtension;
//         Inserted = TRUE;
//     }
//
//     /* Release the lock */
//     KeLowerIrql(OldIrql);
//
//     /* Check if insertion failed */
//     if (!Inserted)
//     {
//         /* Free the entry and fail */
//         ExFreePoolWithTag(NewDriverExtension, TAG_DRIVER_EXTENSION);
//         return STATUS_OBJECT_NAME_COLLISION;
//     }
//
//     /* Otherwise, return the pointer */
//     *DriverObjectExtension = NewDriverExtension + 1;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// IoGetDriverObjectExtension(IN PDRIVER_OBJECT DriverObject,
//                            IN PVOID ClientIdentificationAddress)
// {
//     KIRQL OldIrql;
//     PIO_CLIENT_EXTENSION DriverExtensions;
//
//     /* Acquire lock */
//     OldIrql = KeRaiseIrqlToDpcLevel();
//
//     /* Loop the list until we find the right one */
//     DriverExtensions = IoGetDrvObjExtension(DriverObject)->ClientDriverExtension;
//     while (DriverExtensions)
//     {
//         /* Check for a match */
//         if (DriverExtensions->ClientIdentificationAddress ==
//             ClientIdentificationAddress)
//         {
//             /* Break out */
//             break;
//         }
//
//         /* Keep looping */
//         DriverExtensions = DriverExtensions->NextExtension;
//     }
//
//     /* Release lock */
//     KeLowerIrql(OldIrql);
//
//     /* Return nothing or the extension */
//     if (!DriverExtensions) return NULL;
//     return DriverExtensions + 1;
// }
//
// NTSTATUS
// IopLoadDriver(
//     _In_ HANDLE ServiceHandle,
//     _Out_ PDRIVER_OBJECT *DriverObject)
// {
//     UNICODE_STRING ImagePath;
//     NTSTATUS Status;
//     PLDR_DATA_TABLE_ENTRY ModuleObject;
//     PVOID BaseAddress;
//
//     PKEY_VALUE_FULL_INFORMATION kvInfo;
//     Status = IopGetRegistryValue(ServiceHandle, L"ImagePath", &kvInfo);
//     if (NT_SUCCESS(Status))
//     {
//         if ((kvInfo->Type != REG_EXPAND_SZ && kvInfo->Type != REG_SZ) ||
//             (kvInfo->DataLength < sizeof(UNICODE_NULL)) ||
//             (kvInfo->DataLength > UNICODE_STRING_MAX_BYTES) ||
//             ((kvInfo->DataLength % sizeof(WCHAR)) != 0))
//         {
//             DPRINT1("ObjectName invalid (Type = %lu, DataLength = %lu)\n",
//                     kvInfo->Type,
//                     kvInfo->DataLength);
//             ExFreePool(kvInfo);
//             return STATUS_ILL_FORMED_SERVICE_ENTRY;
//         }
//
//         ImagePath.Length = (USHORT)(kvInfo->DataLength - sizeof(UNICODE_NULL));
//         ImagePath.MaximumLength = kvInfo->DataLength;
//         ImagePath.Buffer = ExAllocatePoolWithTag(PagedPool, ImagePath.MaximumLength, TAG_RTLREGISTRY);
//         if (!ImagePath.Buffer)
//         {
//             ExFreePool(kvInfo);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlMoveMemory(ImagePath.Buffer,
//                       (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                       ImagePath.Length);
//         ImagePath.Buffer[ImagePath.Length / sizeof(WCHAR)] = UNICODE_NULL;
//         ExFreePool(kvInfo);
//     }
//     else
//     {
//         return Status;
//     }
//
//     /*
//      * Normalize the image path for all later processing.
//      */
//     Status = IopNormalizeImagePath(&ImagePath, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopNormalizeImagePath() failed (Status %x)\n", Status);
//         return Status;
//     }
//
//     DPRINT("FullImagePath: '%wZ'\n", &ImagePath);
//
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&IopDriverLoadResource, TRUE);
//
//     /*
//      * Load the driver module
//      */
//     DPRINT("Loading module from %wZ\n", &ImagePath);
//     Status = MmLoadSystemImage(&ImagePath, NULL, NULL, 0, (PVOID)&ModuleObject, &BaseAddress);
//     RtlFreeUnicodeString(&ImagePath);
//
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("MmLoadSystemImage() failed (Status %lx)\n", Status);
//         ExReleaseResourceLite(&IopDriverLoadResource);
//         KeLeaveCriticalRegion();
//         return Status;
//     }
//
//     // Display the loading message
//     ULONG infoLength;
//     Status = ZwQueryKey(ServiceHandle, KeyBasicInformation, NULL, 0, &infoLength);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         PKEY_BASIC_INFORMATION servName = ExAllocatePoolWithTag(PagedPool, infoLength, TAG_IO);
//         if (servName)
//         {
//             Status = ZwQueryKey(ServiceHandle,
//                                 KeyBasicInformation,
//                                 servName,
//                                 infoLength,
//                                 &infoLength);
//             if (NT_SUCCESS(Status))
//             {
//                 UNICODE_STRING serviceName = {
//                     .Length = servName->NameLength,
//                     .MaximumLength = servName->NameLength,
//                     .Buffer = servName->Name
//                 };
//
//                 IopDisplayLoadingMessage(&serviceName);
//             }
//             ExFreePoolWithTag(servName, TAG_IO);
//         }
//     }
//
//     NTSTATUS driverEntryStatus;
//     Status = IopInitializeDriverModule(ModuleObject,
//                                        ServiceHandle,
//                                        DriverObject,
//                                        &driverEntryStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopInitializeDriverModule() failed (Status %lx)\n", Status);
//     }
//
//     ExReleaseResourceLite(&IopDriverLoadResource);
//     KeLeaveCriticalRegion();
//
//     return Status;
// }
//
// static
// VOID
// NTAPI
// IopLoadUnloadDriverWorker(
//     _Inout_ PVOID Parameter)
// {
//     PLOAD_UNLOAD_PARAMS LoadParams = Parameter;
//
//     ASSERT(PsGetCurrentProcess() == PsInitialSystemProcess);
//
//     if (LoadParams->DriverObject)
//     {
//         // unload request
//         LoadParams->DriverObject->DriverUnload(LoadParams->DriverObject);
//         LoadParams->Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         // load request
//         HANDLE serviceHandle;
//         NTSTATUS status;
//         status = IopOpenRegistryKeyEx(&serviceHandle, NULL, LoadParams->RegistryPath, KEY_READ);
//         if (!NT_SUCCESS(status))
//         {
//             LoadParams->Status = status;
//         }
//         else
//         {
//             LoadParams->Status = IopLoadDriver(serviceHandle, &LoadParams->DriverObject);
//             ZwClose(serviceHandle);
//         }
//     }
//
//     if (LoadParams->SetEvent)
//     {
//         KeSetEvent(&LoadParams->Event, 0, FALSE);
//     }
// }
//
// /**
//  * @brief      Process load and unload driver operations. This is mostly for NtLoadDriver
//  *             and NtUnloadDriver, because their code should run inside PsInitialSystemProcess
//  *
//  * @param[in]  RegistryPath  The registry path
//  * @param      DriverObject  The driver object
//  *
//  * @return     Status of the operation
//  */
// NTSTATUS
// IopDoLoadUnloadDriver(
//     _In_opt_ PUNICODE_STRING RegistryPath,
//     _Inout_ PDRIVER_OBJECT *DriverObject)
// {
//     LOAD_UNLOAD_PARAMS LoadParams;
//
//     /* Prepare parameters block */
//     LoadParams.RegistryPath = RegistryPath;
//     LoadParams.DriverObject = *DriverObject;
//
//
//     if (PsGetCurrentProcess() != PsInitialSystemProcess)
//     {
//         LoadParams.SetEvent = TRUE;
//         KeInitializeEvent(&LoadParams.Event, NotificationEvent, FALSE);
//
//         /* Initialize and queue a work item */
//         ExInitializeWorkItem(&LoadParams.WorkItem, IopLoadUnloadDriverWorker, &LoadParams);
//         ExQueueWorkItem(&LoadParams.WorkItem, DelayedWorkQueue);
//
//         /* And wait till it completes */
//         KeWaitForSingleObject(&LoadParams.Event, UserRequest, KernelMode, FALSE, NULL);
//     }
//     else
//     {
//         /* If we're already in a system process, call it right here */
//         LoadParams.SetEvent = FALSE;
//         IopLoadUnloadDriverWorker(&LoadParams);
//     }
//
//
//     return LoadParams.Status;
// }
//
// /*
//  * NtLoadDriver
//  *
//  * Loads a device driver.
//  *
//  * Parameters
//  *    DriverServiceName
//  *       Name of the service to load (registry key).
//  *
//  * Return Value
//  *    Status
//  *
//  * Status
//  *    implemented
//  */
// NTSTATUS NTAPI
// NtLoadDriver(IN PUNICODE_STRING DriverServiceName)
// {
//     UNICODE_STRING CapturedServiceName = { 0, 0, NULL };
//     KPROCESSOR_MODE PreviousMode;
//     PDRIVER_OBJECT DriverObject;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//
//     PreviousMode = KeGetPreviousMode();
//
//     /* Need the appropriate priviliege */
//     if (!SeSinglePrivilegeCheck(SeLoadDriverPrivilege, PreviousMode))
//     {
//         DPRINT1("No load privilege!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//
//     /* Capture the service name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedServiceName,
//                                           PreviousMode,
//                                           DriverServiceName);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DPRINT("NtLoadDriver('%wZ')\n", &CapturedServiceName);
//
//     /* We need a service name */
//     if (CapturedServiceName.Length == 0 || CapturedServiceName.Buffer == NULL)
//     {
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//
//     /* Load driver and call its entry point */
//     DriverObject = NULL;
//     Status = IopDoLoadUnloadDriver(&CapturedServiceName, &DriverObject);
//
//
//     ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//     return Status;
// }
//
// /*
//  * NtUnloadDriver
//  *
//  * Unloads a legacy device driver.
//  *
//  * Parameters
//  *    DriverServiceName
//  *       Name of the service to unload (registry key).
//  *
//  * Return Value
//  *    Status
//  *
//  * Status
//  *    implemented
//  */
//
// NTSTATUS NTAPI
// NtUnloadDriver(IN PUNICODE_STRING DriverServiceName)
// {
//     return IopUnloadDriver(DriverServiceName, FALSE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
