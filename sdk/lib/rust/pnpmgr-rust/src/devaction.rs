use ntkrnl_sys::globals::*;

// ntstatus.h values NOT present in ntkrnl-sys. Without these consts the match
// arms below silently become irrefutable catch-all bindings (the WM_CTLCOLOR
// bug class — see field manual). Keep them defined.
const STATUS_FAILED_DRIVER_ENTRY: NTSTATUS = 0xC0000365_u32 as NTSTATUS;
const STATUS_ILL_FORMED_SERVICE_ENTRY: NTSTATUS = 0xC0000160_u32 as NTSTATUS;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
#[allow(unused_imports)]
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _ERESOURCE, _KEVENT, _LIST_ENTRY, _IO_STACK_LOCATION, _IO_STATUS_BLOCK,
    _DEVICE_CAPABILITIES, _UNICODE_STRING, _KEY_VALUE_FULL_INFORMATION, _DEVICE_OBJECT,
    _DEVICE_NODE, _DRIVER_OBJECT, _DEVICE_RELATIONS, _PNP_BUS_INFORMATION,
    PERESOURCE, PKEVENT, PLIST_ENTRY, PIO_STACK_LOCATION, PIO_STATUS_BLOCK,
    PDEVICE_CAPABILITIES, PUNICODE_STRING, PKEY_VALUE_FULL_INFORMATION, PDEVICE_OBJECT,
    PDEVICE_NODE, PDRIVER_OBJECT, PDEVICE_RELATIONS, PPNP_BUS_INFORMATION,
    PCM_RESOURCE_LIST, PIO_RESOURCE_REQUIREMENTS_LIST, LPGUID, LCID,
    PNP_DEVICE_STATE, HANDLE, POBJECT_TYPE, LPCGUID,
    PWORK_QUEUE_ITEM, _WORK_QUEUE_ITEM, WORK_QUEUE_ITEM,
    _RTL_QUERY_REGISTRY_TABLE, RTL_QUERY_REGISTRY_TABLE,
    PTARGET_DEVICE_CUSTOM_NOTIFICATION,
    _DEVICE_ACTION, KSPIN_LOCK_QUEUE_NUMBER, PCWSTR, PLCID,
    _BUS_QUERY_ID_TYPE, _DEVICE_RELATION_TYPE, _DEVICE_TEXT_TYPE, _PNP_DEVNODE_STATE,
    _CM_SERVICE_LOAD_TYPE, _WORK_QUEUE_TYPE, _EVENT_TYPE,
    ExAllocatePool, ExAllocatePoolZero, ExFreePool, ExQueueWorkItem,
    IoAcquireVpbSpinLock, IoGetAttachedDevice, IoGetAttachedDeviceReference, IoReleaseVpbSpinLock,
    IopAssignDeviceResources, IopCreateDeviceKeyPath, IopGetDeviceNode,
    IopGetDriverNames, IopInitiatePnpIrp, IopQueueDeviceInstallEvent,
    IopQueueTargetDeviceEvent, IopSynchronousCall, KeInitializeEvent, KeWaitForSingleObject,
    ObReferenceObjectByName, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString,
    RtlInitUnicodeString, RtlFreeUnicodeString, RtlCopyUnicodeString, RtlDuplicateUnicodeString,
    RtlQueryRegistryValues,
    ZwClose, ZwSetValueKey, ZwQueryValueKey, ZwQueryDefaultLocale,
    KeBugCheckEx, KeSetEvent, KeClearEvent, KeAcquireQueuedSpinLock, KeReleaseQueuedSpinLock,
};

// BUS_QUERY_ID_TYPE enum values
use ntkrnl_sys::ddk::{
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
};

// DEVICE_ACTION enum values
use ntkrnl_sys::ddk::{
    _DEVICE_ACTION_PiActionEnumDeviceTree as PiActionEnumDeviceTree,
    _DEVICE_ACTION_PiActionEnumRootDevices as PiActionEnumRootDevices,
    _DEVICE_ACTION_PiActionResetDevice as PiActionResetDevice,
    _DEVICE_ACTION_PiActionAddBootDevices as PiActionAddBootDevices,
    _DEVICE_ACTION_PiActionStartDevice as PiActionStartDevice,
    _DEVICE_ACTION_PiActionQueryState as PiActionQueryState,
};

// PNP_DEVNODE_STATE enum values
use ntkrnl_sys::ddk::{
    _PNP_DEVNODE_STATE_DeviceNodeUnspecified as DeviceNodeUnspecified,
    _PNP_DEVNODE_STATE_DeviceNodeUninitialized as DeviceNodeUninitialized,
    _PNP_DEVNODE_STATE_DeviceNodeInitialized as DeviceNodeInitialized,
    _PNP_DEVNODE_STATE_DeviceNodeDriversAdded as DeviceNodeDriversAdded,
    _PNP_DEVNODE_STATE_DeviceNodeResourcesAssigned as DeviceNodeResourcesAssigned,
    _PNP_DEVNODE_STATE_DeviceNodeStartPending as DeviceNodeStartPending,
    _PNP_DEVNODE_STATE_DeviceNodeStartCompletion as DeviceNodeStartCompletion,
    _PNP_DEVNODE_STATE_DeviceNodeStartPostWork as DeviceNodeStartPostWork,
    _PNP_DEVNODE_STATE_DeviceNodeStarted as DeviceNodeStarted,
    _PNP_DEVNODE_STATE_DeviceNodeQueryStopped as DeviceNodeQueryStopped,
    _PNP_DEVNODE_STATE_DeviceNodeStopped as DeviceNodeStopped,
    _PNP_DEVNODE_STATE_DeviceNodeRestartCompletion as DeviceNodeRestartCompletion,
    _PNP_DEVNODE_STATE_DeviceNodeEnumeratePending as DeviceNodeEnumeratePending,
    _PNP_DEVNODE_STATE_DeviceNodeEnumerateCompletion as DeviceNodeEnumerateCompletion,
    _PNP_DEVNODE_STATE_DeviceNodeAwaitingQueuedDeletion as DeviceNodeAwaitingQueuedDeletion,
    _PNP_DEVNODE_STATE_DeviceNodeAwaitingQueuedRemoval as DeviceNodeAwaitingQueuedRemoval,
    _PNP_DEVNODE_STATE_DeviceNodeQueryRemoved as DeviceNodeQueryRemoved,
    _PNP_DEVNODE_STATE_DeviceNodeRemovePendingCloses as DeviceNodeRemovePendingCloses,
    _PNP_DEVNODE_STATE_DeviceNodeRemoved as DeviceNodeRemoved,
    _PNP_DEVNODE_STATE_DeviceNodeDeletePendingCloses as DeviceNodeDeletePendingCloses,
    _PNP_DEVNODE_STATE_DeviceNodeDeleted as DeviceNodeDeleted,
};

// DEVICE_RELATION_TYPE enum values
use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_EjectionRelations as EjectionRelations,
    _DEVICE_RELATION_TYPE_RemovalRelations as RemovalRelations,
};

// DEVICE_TEXT_TYPE enum values
use ntkrnl_sys::ddk::{
    _DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription,
    _DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation,
};

// Other enum values
use ntkrnl_sys::ddk::{
    _CM_SERVICE_LOAD_TYPE_DisableLoad as DisableLoad,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _EVENT_TYPE_SynchronizationEvent,
    _KWAIT_REASON_Executive,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueBasicInformation as KeyValueBasicInformation,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
};
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


const LockQueueIoDatabaseLock: KSPIN_LOCK_QUEUE_NUMBER = 10;


// Type aliases for enum types used as parameter types
type BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE;
type DEVICE_ACTION = _DEVICE_ACTION;
type SERVICE_LOAD_TYPE = _CM_SERVICE_LOAD_TYPE;

// IRP major/minor function codes
const IRP_MJ_PNP: UCHAR = 0x1b;
const IRP_MN_START_DEVICE: UCHAR = 0x00;
const IRP_MN_QUERY_REMOVE_DEVICE: UCHAR = 0x01;
const IRP_MN_REMOVE_DEVICE: UCHAR = 0x02;
const IRP_MN_CANCEL_REMOVE_DEVICE: UCHAR = 0x03;
const IRP_MN_QUERY_DEVICE_RELATIONS: UCHAR = 0x07;
const IRP_MN_QUERY_CAPABILITIES: UCHAR = 0x09;
const IRP_MN_QUERY_RESOURCES: UCHAR = 0x0A;
const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: UCHAR = 0x0B;
const IRP_MN_EJECT: UCHAR = 0x11;
const IRP_MN_QUERY_ID: UCHAR = 0x13;
const IRP_MN_QUERY_BUS_INFORMATION: UCHAR = 0x15;
const IRP_MN_SURPRISE_REMOVAL: UCHAR = 0x17;

// PNP_DEVICE_STATE flags
const PNP_DEVICE_DISABLED: PNP_DEVICE_STATE = 0x00000001;
const PNP_DEVICE_DONT_DISPLAY_IN_UI: PNP_DEVICE_STATE = 0x00000002;
const PNP_DEVICE_FAILED: PNP_DEVICE_STATE = 0x00000004;
const PNP_DEVICE_REMOVED: PNP_DEVICE_STATE = 0x00000008;
const PNP_DEVICE_RESOURCE_REQUIREMENTS_CHANGED: PNP_DEVICE_STATE = 0x00000010;
const PNP_DEVICE_NOT_DISABLEABLE: PNP_DEVICE_STATE = 0x00000020;

// Device node flags (DNF_*)
const DNF_REENUMERATE: ULONG = 0x00000008;
const DNF_ENUMERATED: ULONG = 0x00000010;
const DNF_IDS_QUERIED: ULONG = 0x00000020;
const DNF_HAS_BOOT_CONFIG: ULONG = 0x00000040;
const DNF_NO_RESOURCE_REQUIRED: ULONG = 0x00000100;
const DNF_RESOURCE_REQUIREMENTS_CHANGED: ULONG = 0x00000400;
const DNF_NON_STOPPED_REBALANCE: ULONG = 0x00000800;
const DNF_LEGACY_DRIVER: ULONG = 0x00001000;
const DNF_HAS_PROBLEM: ULONG = 0x00002000;
const DNF_DEVICE_GONE: ULONG = 0x00010000;

// Device node user flags (DNUF_*)
const DNUF_DONT_SHOW_IN_UI: ULONG = 0x0002;
const DNUF_NOT_DISABLEABLE: ULONG = 0x0008;

// CM_PROB_* problem codes
const CM_PROB_OUT_OF_MEMORY: ULONG = 0x00000003;
const CM_PROB_FAILED_START: ULONG = 0x0000000A;
const CM_PROB_NEED_RESTART: ULONG = 0x0000000E;
const CM_PROB_REGISTRY: ULONG = 0x00000013;
const CM_PROB_DEVICE_NOT_THERE: ULONG = 0x00000018;
const CM_PROB_HARDWARE_DISABLED: ULONG = 0x0000001D;
const CM_PROB_FAILED_ADD: ULONG = 0x0000001F;
const CM_PROB_DISABLED_SERVICE: ULONG = 0x00000020;
const CM_PROB_FAILED_DRIVER_ENTRY: ULONG = 0x00000025;
const CM_PROB_DRIVER_FAILED_LOAD: ULONG = 0x00000027;
const CM_PROB_DRIVER_SERVICE_KEY_INVALID: ULONG = 0x00000028;
const CM_PROB_FAILED_POST_START: ULONG = 0x0000002B;
const CM_PROB_HELD_FOR_EJECT: ULONG = 0x0000002F;

// VPB flags
const VPB_MOUNTED: USHORT = 0x0001;
const VPB_REMOVE_PENDING: USHORT = 0x0008;

// Other constants
const DO_BUS_ENUMERATED_DEVICE: ULONG = 0x1000;
const FILE_DEVICE_ACPI: ULONG = 0x00000032;
const PNP_DETECTED_FATAL_ERROR: ULONG = 0x000000CA;
const STATUS_PLUGPLAY_NO_DEVICE: NTSTATUS = 0xC000025E_u32 as i32;
const STATUS_PNP_REBOOT_REQUIRED: NTSTATUS = 0xC00002D2_u32 as i32;
const UNICODE_STRING_MAX_BYTES: ULONG = 65534;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_OPENED_EXISTING_KEY: ULONG = 0x00000002;
const RTL_REGISTRY_HANDLE: ULONG = 0x40000000;
const RTL_QUERY_REGISTRY_REQUIRED: ULONG = 0x00000004;
const RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING: ULONG = 0x00000002;

const TAG_IO: u32 = 0x20206F49;
const TAG_PNP_DEVACTION: u32 = 0x61447050;

// ENUM_ROOT wide string: L"\\Registry\\Machine\\System\\CurrentControlSet\\Enum"
const ENUM_ROOT: &[u16] = &[
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16,
    b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16,
    b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16,
    b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
    b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16,
    b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, 0,
];

// REGSTR_VAL_CLASSGUID: L"ClassGUID"
const REGSTR_VAL_CLASSGUID: &[u16] = &[
    b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'G' as u16, b'U' as u16,
    b'I' as u16, b'D' as u16, 0,
];

// REGSTR_KEY_DEVICE_PROPERTIES: L"Properties"
const REGSTR_KEY_DEVICE_PROPERTIES: &[u16] = &[
    b'P' as u16, b'r' as u16, b'o' as u16, b'p' as u16, b'e' as u16, b'r' as u16, b't' as u16,
    b'i' as u16, b'e' as u16, b's' as u16, 0,
];

// GUID definitions
#[repr(C)]
struct GUID {
    Data1: u32,
    Data2: u16,
    Data3: u16,
    Data4: [u8; 8],
}

static GUID_DEVICE_ARRIVAL: GUID = GUID {
    Data1: 0xCB3A4009, Data2: 0x46F0, Data3: 0x11D0,
    Data4: [0xB0, 0x8F, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3F],
};
static GUID_DEVICE_ENUMERATED: GUID = GUID {
    Data1: 0xCB3A400A, Data2: 0x46F0, Data3: 0x11D0,
    Data4: [0xB0, 0x8F, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3F],
};
static GUID_DEVICE_REMOVE_PENDING: GUID = GUID {
    Data1: 0xCB3A400D, Data2: 0x46F0, Data3: 0x11D0,
    Data4: [0xB0, 0x8F, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3F],
};
static GUID_DEVICE_EJECT: GUID = GUID {
    Data1: 0xCB3A400F, Data2: 0x46F0, Data3: 0x11D0,
    Data4: [0xB0, 0x8F, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3F],
};
static GUID_DEVICE_KERNEL_INITIATED_EJECT: GUID = GUID {
    Data1: 0x14689B54, Data2: 0x0703, Data3: 0x11D3,
    Data4: [0x97, 0xD2, 0x00, 0xA0, 0xC9, 0x40, 0x52, 0x2E],
};
static GUID_DEVICE_REMOVAL_VETOED: GUID = GUID {
    Data1: 0x60DBD5FA, Data2: 0xDDD2, Data3: 0x11D2,
    Data4: [0x97, 0xB8, 0x00, 0xA0, 0xC9, 0x40, 0x52, 0x2E],
};
static GUID_DEVICE_SAFE_REMOVAL: GUID = GUID {
    Data1: 0x8FBEF967, Data2: 0xD6C5, Data3: 0x11D2,
    Data4: [0x97, 0xB5, 0x00, 0xA0, 0xC9, 0x40, 0x52, 0x2E],
};
static GUID_DEVICE_SURPRISE_REMOVAL: GUID = GUID {
    Data1: 0xCE5AF000, Data2: 0x80DD, Data3: 0x11D2,
    Data4: [0xA8, 0x8D, 0x00, 0xA0, 0xC9, 0x69, 0x6B, 0x4B],
};
static GUID_DEVICE_EJECT_VETOED: GUID = GUID {
    Data1: 0xCF7B71E8, Data2: 0xD8FD, Data3: 0x11D2,
    Data4: [0x97, 0xB5, 0x00, 0xA0, 0xC9, 0x40, 0x52, 0x2E],
};
static GUID_TARGET_DEVICE_QUERY_REMOVE: GUID = GUID {
    Data1: 0xcb3a4006, Data2: 0x46f0, Data3: 0x11d0,
    Data4: [0xb0, 0x8f, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3f],
};
static GUID_TARGET_DEVICE_REMOVE_CANCELLED: GUID = GUID {
    Data1: 0xcb3a4007, Data2: 0x46f0, Data3: 0x11d0,
    Data4: [0xb0, 0x8f, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3f],
};
static GUID_TARGET_DEVICE_REMOVE_COMPLETE: GUID = GUID {
    Data1: 0xcb3a4008, Data2: 0x46f0, Data3: 0x11d0,
    Data4: [0xb0, 0x8f, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3f],
};

extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn ExInitializeWorkItem(WorkItem: PWORK_QUEUE_ITEM, Routine: unsafe extern "system" fn(PVOID), Context: PVOID);
}

#[inline(always)]
unsafe fn CONTAINING_RECORD<T>(ptr: PVOID, _type_size: usize, field_offset: usize) -> *mut T {
    (ptr as usize - field_offset) as *mut T
}

#[inline(never)]
unsafe fn da_trace(_s: &[u8]) {}

#[inline(never)]
unsafe fn da_tracex(_tag: &[u8], _a: usize, _b: usize) {}
// `IoRequestDeviceEject_c` deleted 2026-05-06 — Rust impl below replaces it.
extern "system" {
    fn IopGetRegistryValue(Handle: HANDLE, ValueName: PWSTR, Information: *mut PKEY_VALUE_FULL_INFORMATION) -> NTSTATUS;
    fn IopOpenRegistryKeyEx(KeyHandle: *mut HANDLE, ParentKey: HANDLE, Name: PUNICODE_STRING, DesiredAccess: u32) -> NTSTATUS;
    fn IopCreateRegistryKeyEx(Handle: *mut HANDLE, RootHandle: HANDLE, KeyName: PUNICODE_STRING, DesiredAccess: u32, CreateOptions: ULONG, Disposition: *mut ULONG) -> NTSTATUS;
}
extern "C" {
    fn IopLoadDriver(ServiceHandle: HANDLE, DriverObject: *mut PDRIVER_OBJECT) -> NTSTATUS;
}
// `PiSetDevNodeText_c` deleted 2026-05-06 — Rust impl below replaces it.
// `PiQueueDeviceAction_c` and `PiPerformSyncDeviceAction_c` deleted 2026-05-06
// — Rust impls below replace them. Same captures(none)-ghost re-probe pattern
// as `IopGetBusTypeGuidIndex` (commit 280782e5).

extern "C" {
    static mut IopRootDeviceNode: PDEVICE_NODE;
    static mut PnpSystemInit: BOOLEAN;
}
extern "system" {
    static mut IopDriverLoadResource: _ERESOURCE;
    static mut PnPBootDriversLoaded: BOOLEAN;
    static mut PnPBootDriversInitialized: BOOLEAN;
    static mut IoDriverObjectType: POBJECT_TYPE;
    static mut PopSystemPowerDeviceNode: PDEVICE_NODE;
    static mut IopDeviceTreeLock: KSPIN_LOCK;
}

// Functions defined in sibling pnpmgr modules (linked via #[no_mangle])
// devnode.rs functions are cdecl (no NTAPI in C declarations)
extern "C" {
    fn PipAllocateDeviceNode(PhysicalDeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
    fn PiInsertDevNode(DeviceNode: PDEVICE_NODE, ParentNode: PDEVICE_NODE);
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, NewState: u32) -> u32;
    fn PiSetDevNodeProblem(DeviceNode: PDEVICE_NODE, Problem: ULONG);
}
// pnpirp.rs functions — these are also cdecl (no NTAPI in C)
extern "C" {
    fn PiIrpStartDevice(DeviceNode: PDEVICE_NODE) -> u32;
    fn PiIrpStopDevice(DeviceNode: PDEVICE_NODE) -> u32;
    fn PiIrpQueryStopDevice(DeviceNode: PDEVICE_NODE) -> u32;
    fn PiIrpCancelStopDevice(DeviceNode: PDEVICE_NODE) -> u32;
    fn PiIrpQueryDeviceRelations(DeviceNode: PDEVICE_NODE, Type: _DEVICE_RELATION_TYPE) -> NTSTATUS;
    fn PiIrpQueryResources(DeviceNode: PDEVICE_NODE, BootResources: *mut PCM_RESOURCE_LIST) -> u32;
    fn PiIrpQueryResourceRequirements(DeviceNode: PDEVICE_NODE, ResourceReqs: *mut PIO_RESOURCE_REQUIREMENTS_LIST) -> u32;
    fn PiIrpQueryDeviceText(DeviceNode: PDEVICE_NODE, LocaleId: LCID, TextType: _DEVICE_TEXT_TYPE, Text: *mut PWSTR) -> NTSTATUS;
    fn PiIrpQueryPnPDeviceState(DeviceNode: PDEVICE_NODE, PnPFlags: *mut PNP_DEVICE_STATE) -> NTSTATUS;
}
// pnpnotify.rs — PiNotifyTargetDeviceChange is non-NTAPI (extern "C")
extern "C" {
    fn PiNotifyTargetDeviceChange(Event: LPCGUID, DeviceObject: PDEVICE_OBJECT, CustomNotification: PTARGET_DEVICE_CUSTOM_NOTIFICATION);
}

// IopDeviceNodeSetFlag is a macro/inline in C - define as direct flag set
#[no_mangle]
pub unsafe extern "C" fn IopDeviceNodeSetFlag(DeviceNode: PDEVICE_NODE, Flag: ULONG) {
    (*DeviceNode).Flags |= Flag;
}

const MAX_DEVICE_ID_LEN: usize = 200;
const MAX_SEPARATORS_INSTANCEID: ULONG = 0;
const MAX_SEPARATORS_DEVICEID: ULONG = 1;

#[used]
#[no_mangle]
pub static mut IopDeviceActionRequestList: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[used]
#[no_mangle]
pub static mut IopDeviceActionWorkItem: _WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[used]
#[no_mangle]
pub static mut IopDeviceActionInProgress: BOOLEAN = 0;
#[used]
#[no_mangle]
pub static mut IopDeviceActionLock: KSPIN_LOCK = 0;
#[used]
#[no_mangle]
pub static mut PiEnumerationFinished: _KEVENT = unsafe { core::mem::zeroed() };

#[inline(always)]
unsafe fn device_tree_lock_ptr() -> *mut KSPIN_LOCK {
    core::ptr::addr_of_mut!(IopDeviceTreeLock)
}

#[inline(always)]
unsafe fn device_action_lock_ptr() -> *mut KSPIN_LOCK {
    core::ptr::addr_of_mut!(IopDeviceActionLock)
}

#[inline(always)]
unsafe fn device_action_request_list_ptr() -> PLIST_ENTRY {
    core::ptr::addr_of_mut!(IopDeviceActionRequestList) as PLIST_ENTRY
}

#[inline(always)]
unsafe fn enumeration_finished_ptr() -> PKEVENT {
    core::ptr::addr_of_mut!(PiEnumerationFinished) as PKEVENT
}

#[inline(always)]
unsafe fn device_relations_object_slot(
    device_relations: PDEVICE_RELATIONS,
    index: ULONG,
) -> *mut PDEVICE_OBJECT {
    // bindgen models the trailing flexible array as `[T; 1]`, so callers must
    // use raw pointer arithmetic instead of Rust indexing when Count > 1.
    core::ptr::addr_of_mut!((*device_relations).Objects[0]).add(index as usize)
}

#[inline(always)]
unsafe fn device_relations_object(
    device_relations: PDEVICE_RELATIONS,
    index: ULONG,
) -> PDEVICE_OBJECT {
    *device_relations_object_slot(device_relations, index)
}

// L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\"
static ServicesKeyName: [u16; 53] = [
    b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16,
    b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16,
    b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16,
    b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
    b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16,
    b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    b'\\' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
    b'e' as u16, b's' as u16, b'\\' as u16, 0,
];

#[repr(C)]
struct DEVICE_ACTION_REQUEST {
    RequestListEntry: _LIST_ENTRY,
    DeviceObject: PDEVICE_OBJECT,
    CompletionEvent: PKEVENT,
    CompletionStatus: *mut NTSTATUS,
    Action: DEVICE_ACTION,
}
type PDEVICE_ACTION_REQUEST = *mut DEVICE_ACTION_REQUEST;

#[repr(C)]
#[derive(Copy, Clone)]
enum ADD_DEV_DRIVER_TYPE {
    LowerFilter,
    LowerClassFilter,
    DeviceDriver,
    UpperFilter,
    UpperClassFilter,
}

#[repr(C)]
struct ADD_DEV_DRIVERS_LIST {
    ListEntry: _LIST_ENTRY,
    DriverObject: PDRIVER_OBJECT,
    DriverType: ADD_DEV_DRIVER_TYPE,
}
type PADD_DEV_DRIVERS_LIST = *mut ADD_DEV_DRIVERS_LIST;

#[repr(C)]
struct ATTACH_FILTER_DRIVERS_CONTEXT {
    DriverType: ADD_DEV_DRIVER_TYPE,
    DeviceNode: PDEVICE_NODE,
    DriversListHead: PLIST_ENTRY,
}
type PATTACH_FILTER_DRIVERS_CONTEXT = *mut ATTACH_FILTER_DRIVERS_CONTEXT;

extern "C" {
    fn IopGetDeviceObjectFromDeviceInstance(DeviceInstance: PUNICODE_STRING) -> PDEVICE_OBJECT;
    fn IopGetParentIdPrefix(DeviceNode: PDEVICE_NODE, ParentIdPrefix: PUNICODE_STRING) -> NTSTATUS;
    fn IopSetDeviceInstanceData(InstanceKey: HANDLE, DeviceNode: PDEVICE_NODE) -> NTSTATUS;
}
extern "system" {
    fn IopGetBusTypeGuidIndex(BusTypeGuid: LPGUID) -> USHORT;
    fn IopInstallCriticalDevice(DeviceNode: PDEVICE_NODE);
}

unsafe fn IopValidateID(
    Id: PWCHAR,
    QueryType: BUS_QUERY_ID_TYPE,
) -> BOOLEAN {
    let mut PtrChar: PWCHAR;
    let mut StringEnd: PWCHAR;
    let mut Char: WCHAR;
    let mut SeparatorsCount: ULONG = 0;
    let mut PtrPrevChar: PWCHAR = core::ptr::null_mut();
    let MaxSeparators: ULONG;
    let IsMultiSz: BOOLEAN;

    if QueryType == BusQueryDeviceID {
        MaxSeparators = MAX_SEPARATORS_DEVICEID;
        IsMultiSz = 0;
    } else if QueryType == BusQueryInstanceID {
        MaxSeparators = MAX_SEPARATORS_INSTANCEID;
        IsMultiSz = 0;
    } else if QueryType == BusQueryHardwareIDs || QueryType == BusQueryCompatibleIDs {
        MaxSeparators = MAX_SEPARATORS_DEVICEID;
        IsMultiSz = 1;
    } else {
        return 0;
    }

    StringEnd = Id.add(MAX_DEVICE_ID_LEN);

    PtrChar = Id;
    while PtrChar < StringEnd {
        Char = *PtrChar;

        if Char == 0 {
            if IsMultiSz == 0 || (!PtrPrevChar.is_null() && PtrChar == PtrPrevChar.add(1)) {
                if MaxSeparators == SeparatorsCount || IsMultiSz != 0 {
                    return 1;
                }
                return 0;
            }

            StringEnd = PtrChar.add(MAX_DEVICE_ID_LEN + 1);
            PtrPrevChar = PtrChar;
            SeparatorsCount = 0;
        } else if Char < b' ' as u16 || Char > 0x7F || Char == b',' as u16 {
            return 0;
        } else if Char == b' ' as u16 {
            *PtrChar = b'_' as u16;
        } else if Char == b'\\' as u16 {
            SeparatorsCount += 1;

            if SeparatorsCount > MaxSeparators {
                return 0;
            }
        }

        PtrChar = PtrChar.add(1);
    }

    0
}

unsafe fn IopCreateDeviceInstancePath(
    DeviceNode: PDEVICE_NODE,
    InstancePath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceId: _UNICODE_STRING = core::mem::zeroed();
    let mut InstanceId: _UNICODE_STRING = core::mem::zeroed();
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ParentIdPrefix: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceCapabilities: _DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut IsValidID: BOOLEAN = 0;

    Stack.Parameters.QueryId.IdType = BusQueryDeviceID;
    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_ID,
        &mut Stack as *mut _,
    );
    if Status < 0 {
        crate::dbg_print!(
            b"[DA]   CreateDeviceInstancePath: QUERY_ID(DeviceID) -> 0x%x\r\n\0",
            Status as u32
        );
        return Status;
    }

    IsValidID = IopValidateID(IoStatusBlock.Information as PWCHAR, BusQueryDeviceID);
    if IsValidID == 0 {
    }

    RtlInitUnicodeString(&mut DeviceId as *mut _, IoStatusBlock.Information as PWSTR);

    if DeviceId.Length == 0 {
        RtlFreeUnicodeString(&mut DeviceId as *mut _);
        return STATUS_PLUGPLAY_NO_DEVICE;
    }

    Status = IopQueryDeviceCapabilities(DeviceNode, &mut DeviceCapabilities as *mut _);
    if Status < 0 {
        crate::dbg_print!(
            b"[DA]   CreateDeviceInstancePath: QUERY_CAPABILITIES -> 0x%x\r\n\0",
            Status as u32
        );
        if Status != STATUS_NOT_SUPPORTED {
        }
        RtlFreeUnicodeString(&mut DeviceId as *mut _);
        return Status;
    }

    if DeviceCapabilities.HardwareDisabled() != 0 {
        RtlFreeUnicodeString(&mut DeviceId as *mut _);
        return STATUS_PLUGPLAY_NO_DEVICE;
    }

    if DeviceCapabilities.UniqueID() == 0 {
        Status = IopGetParentIdPrefix(DeviceNode, &mut ParentIdPrefix as *mut _);
        if Status < 0 {
            RtlFreeUnicodeString(&mut DeviceId as *mut _);
            return Status;
        }
    }

    Stack.Parameters.QueryId.IdType = BusQueryInstanceID;
    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_ID,
        &mut Stack as *mut _,
    );
    if Status < 0 {
    }

    if IoStatusBlock.Information != 0 {
        IsValidID = IopValidateID(IoStatusBlock.Information as PWCHAR, BusQueryInstanceID);
        if IsValidID == 0 {
        }
    }

    RtlInitUnicodeString(&mut InstanceId as *mut _, IoStatusBlock.Information as PWSTR);

    (*InstancePath).Length = 0;
    (*InstancePath).MaximumLength = DeviceId.Length
        + core::mem::size_of::<WCHAR>() as u16
        + ParentIdPrefix.Length
        + InstanceId.Length
        + core::mem::size_of::<u16>() as u16;
    if ParentIdPrefix.Length != 0 && InstanceId.Length != 0 {
        (*InstancePath).MaximumLength += core::mem::size_of::<WCHAR>() as u16;
    }

    (*InstancePath).Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*InstancePath).MaximumLength as _,
        TAG_IO,
    ) as PWSTR;
    if (*InstancePath).Buffer.is_null() {
        RtlFreeUnicodeString(&mut InstanceId as *mut _);
        RtlFreeUnicodeString(&mut ParentIdPrefix as *mut _);
        RtlFreeUnicodeString(&mut DeviceId as *mut _);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(InstancePath, &mut DeviceId as *mut _);
    // L"\\"
    static BACKSLASH: [u16; 2] = [b'\\' as u16, 0];
    RtlAppendUnicodeToString(InstancePath, BACKSLASH.as_ptr());

    RtlAppendUnicodeStringToString(InstancePath, &mut ParentIdPrefix as *mut _);
    if ParentIdPrefix.Length != 0 && InstanceId.Length != 0 {
        RtlAppendUnicodeToString(InstancePath, w!("&").as_ptr());
    }

    RtlAppendUnicodeStringToString(InstancePath, &mut InstanceId as *mut _);

    RtlFreeUnicodeString(&mut InstanceId as *mut _);
    RtlFreeUnicodeString(&mut DeviceId as *mut _);
    RtlFreeUnicodeString(&mut ParentIdPrefix as *mut _);

    STATUS_SUCCESS
}

unsafe extern "system" fn PiAttachFilterDriversCallback(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Ctx: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let context = Ctx as PATTACH_FILTER_DRIVERS_CONTEXT;
    let mut DriverObject: PDRIVER_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let loadDrivers = EntryContext as ULONG_PTR as BOOLEAN;

    if ValueType != REG_SZ {
        return STATUS_SUCCESS;
    }

    if ValueLength <= core::mem::size_of::<WCHAR>() as ULONG {
        return STATUS_OBJECT_NAME_NOT_FOUND;
    }

    let mut serviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut servicesKeyName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut serviceName as *mut _, ValueData as PWSTR);
    RtlInitUnicodeString(
        &mut servicesKeyName as *mut _,
        ServicesKeyName.as_ptr(),
    );

    let mut ccsServicesHandle: HANDLE = core::ptr::null_mut();
    let mut serviceHandle: HANDLE = core::ptr::null_mut();

    Status = IopOpenRegistryKeyEx(
        &mut ccsServicesHandle as *mut _,
        core::ptr::null_mut(),
        &mut servicesKeyName as *mut _,
        KEY_READ,
    );
    if Status < 0 {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        &mut serviceHandle as *mut _,
        ccsServicesHandle,
        &mut serviceName as *mut _,
        KEY_READ,
    );
    ZwClose(ccsServicesHandle);
    if Status < 0 {
        return Status;
    }

    let driverEntry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<ADD_DEV_DRIVERS_LIST>() as _,
        TAG_PNP_DEVACTION,
    ) as PADD_DEV_DRIVERS_LIST;

    if driverEntry.is_null() {
        ZwClose(serviceHandle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut kvInfo: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut startType: SERVICE_LOAD_TYPE = DisableLoad;

    Status = IopGetRegistryValue(serviceHandle, b"S\0t\0a\0r\0t\0\0\0" as *const u8 as PWSTR, &mut kvInfo as *mut _);
    if Status >= 0 {
        if (*kvInfo).Type == REG_DWORD {
            core::ptr::copy_nonoverlapping(
                (kvInfo as ULONG_PTR + (*kvInfo).DataOffset as ULONG_PTR) as *const u8,
                &mut startType as *mut _ as *mut u8,
                core::mem::size_of::<SERVICE_LOAD_TYPE>(),
            );
        }

        ExFreePool(kvInfo as PVOID);
    }

    if startType >= DisableLoad {
        if ((*(*context).DeviceNode).Flags & DNF_HAS_PROBLEM) == 0 {
            PiSetDevNodeProblem((*context).DeviceNode, CM_PROB_DISABLED_SERVICE);
        }

        Status = STATUS_UNSUCCESSFUL;
        ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
        if !serviceHandle.is_null() {
            ZwClose(serviceHandle);
        }
        return Status;
    }

    let mut driverName: _UNICODE_STRING = core::mem::zeroed();
    Status = IopGetDriverNames(serviceHandle, &mut driverName as *mut _, core::ptr::null_mut());
    if Status < 0 {
        ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
        if !serviceHandle.is_null() {
            ZwClose(serviceHandle);
        }
        return Status;
    }

    Status = ObReferenceObjectByName(
        &mut driverName as *mut _,
        OBJ_OPENIF | OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        0,
        IoDriverObjectType,
        0i32 as i8,
        core::ptr::null_mut(),
        &mut DriverObject as *mut _ as *mut PVOID,
    );
    RtlFreeUnicodeString(&mut driverName as *mut _);

    if Status < 0 {
        if loadDrivers != 0 {
            Status = IopLoadDriver(serviceHandle, &mut DriverObject as *mut _);
        } else {
            Status = STATUS_UNSUCCESSFUL;
            ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
            if !serviceHandle.is_null() {
                ZwClose(serviceHandle);
            }
            return Status;
        }
    }

    if Status >= 0 {
        (*driverEntry).DriverObject = DriverObject;
        (*driverEntry).DriverType = (*context).DriverType;
        InsertTailList((*context).DriversListHead, &mut (*driverEntry).ListEntry as *mut _);
        ZwClose(serviceHandle);
        return STATUS_SUCCESS;
    } else {
        if ((*(*context).DeviceNode).Flags & DNF_HAS_PROBLEM) == 0 {
            match Status {
                STATUS_INSUFFICIENT_RESOURCES => {
                    PiSetDevNodeProblem((*context).DeviceNode, CM_PROB_OUT_OF_MEMORY);
                }
                STATUS_FAILED_DRIVER_ENTRY => {
                    PiSetDevNodeProblem((*context).DeviceNode, CM_PROB_FAILED_DRIVER_ENTRY);
                }
                STATUS_ILL_FORMED_SERVICE_ENTRY => {
                    PiSetDevNodeProblem((*context).DeviceNode, CM_PROB_DRIVER_SERVICE_KEY_INVALID);
                }
                _ => {
                    PiSetDevNodeProblem((*context).DeviceNode, CM_PROB_DRIVER_FAILED_LOAD);
                }
            }
        }
    }

    ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
    if !serviceHandle.is_null() {
        ZwClose(serviceHandle);
    }
    Status
}

unsafe fn PiAttachFilterDrivers(
    DriversListHead: PLIST_ENTRY,
    DeviceNode: PDEVICE_NODE,
    EnumSubKey: HANDLE,
    ClassKey: HANDLE,
    Lower: BOOLEAN,
    LoadDrivers: BOOLEAN,
) -> NTSTATUS {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut routineContext: ATTACH_FILTER_DRIVERS_CONTEXT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    routineContext.DriversListHead = DriversListHead;
    routineContext.DeviceNode = DeviceNode;

    routineContext.DriverType = if Lower != 0 { ADD_DEV_DRIVER_TYPE::LowerFilter } else { ADD_DEV_DRIVER_TYPE::UpperFilter };
    QueryTable[0] = RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: Some(core::mem::transmute(PiAttachFilterDriversCallback as unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS)),
        Name: if Lower != 0 { w!("LowerFilters").as_ptr() } else { w!("UpperFilters").as_ptr() },
        DefaultType: REG_NONE,
        EntryContext: LoadDrivers as ULONG_PTR as PVOID,
        ..core::mem::zeroed()
    };

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_HANDLE,
        EnumSubKey as PWSTR,
        QueryTable.as_mut_ptr(),
        &mut routineContext as *mut _ as PVOID,
        core::ptr::null_mut(),
    );
    if ClassKey.is_null() {
        return Status;
    }

    routineContext.DriverType = if Lower != 0 { ADD_DEV_DRIVER_TYPE::LowerClassFilter } else { ADD_DEV_DRIVER_TYPE::UpperClassFilter };
    QueryTable[0] = RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: Some(core::mem::transmute(PiAttachFilterDriversCallback as unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS)),
        Name: if Lower != 0 { w!("LowerFilters").as_ptr() } else { w!("UpperFilters").as_ptr() },
        DefaultType: REG_NONE,
        EntryContext: LoadDrivers as ULONG_PTR as PVOID,
        ..core::mem::zeroed()
    };

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_HANDLE,
        ClassKey as PWSTR,
        QueryTable.as_mut_ptr(),
        &mut routineContext as *mut _ as PVOID,
        core::ptr::null_mut(),
    );
    Status
}

unsafe fn PiCallDriverAddDevice(
    DeviceNode: PDEVICE_NODE,
    LoadDrivers: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut EnumRootKey: HANDLE = core::ptr::null_mut();
    let mut SubKey: HANDLE = core::ptr::null_mut();
    let mut ClassKey: HANDLE = core::ptr::null_mut();
    let mut EnumRoot: _UNICODE_STRING = core::mem::zeroed();
    // L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Class"
    static CCS_CONTROL_CLASS: [u16; 57] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16,
        b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16,
        b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'y' as u16, b's' as u16,
        b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16,
        b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16,
        b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
        b'l' as u16, b'\\' as u16, b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16,
        0,
    ];
    let mut ccsControlClass: _UNICODE_STRING = _UNICODE_STRING {
        Length: (CCS_CONTROL_CLASS.len() * 2 - 2) as u16,
        MaximumLength: (CCS_CONTROL_CLASS.len() * 2) as u16,
        Buffer: CCS_CONTROL_CLASS.as_ptr() as PWSTR,
    };
    let mut kvInfo: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();

    RtlInitUnicodeString(&mut EnumRoot as *mut _, ENUM_ROOT.as_ptr());

    Status = IopOpenRegistryKeyEx(&mut EnumRootKey as *mut _, core::ptr::null_mut(), &mut EnumRoot as *mut _, KEY_READ);
    if Status < 0 {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(&mut SubKey as *mut _, EnumRootKey, &mut (*DeviceNode).InstancePath as *mut _, KEY_READ);
    ZwClose(EnumRootKey);
    if Status < 0 {
        return Status;
    }

    Status = IopGetRegistryValue(SubKey, REGSTR_VAL_CLASSGUID.as_ptr() as PWSTR, &mut kvInfo as *mut _);
    if Status >= 0 {
        if ((*kvInfo).Type == REG_SZ)
            && ((*kvInfo).DataLength > core::mem::size_of::<u16>() as ULONG)
            && ((*kvInfo).DataLength <= UNICODE_STRING_MAX_BYTES)
            && (((*kvInfo).DataLength % core::mem::size_of::<WCHAR>() as ULONG) == 0)
        {
            let classGUID = _UNICODE_STRING {
                MaximumLength: (*kvInfo).DataLength as u16,
                Length: ((*kvInfo).DataLength - core::mem::size_of::<u16>() as ULONG) as u16,
                Buffer: (kvInfo as ULONG_PTR + (*kvInfo).DataOffset as ULONG_PTR) as PWSTR,
            };
            let mut ccsControlHandle: HANDLE = core::ptr::null_mut();

            Status = IopOpenRegistryKeyEx(&mut ccsControlHandle as *mut _, core::ptr::null_mut(), &ccsControlClass as *const _ as *mut _, KEY_READ);
            if Status >= 0 {
                Status = IopOpenRegistryKeyEx(&mut ClassKey as *mut _, ccsControlHandle, &classGUID as *const _ as *mut _, KEY_READ);
                ZwClose(ccsControlHandle);
            }

            if !ClassKey.is_null() {
                let mut properties: _UNICODE_STRING = core::mem::zeroed();
                RtlInitUnicodeString(&mut properties as *mut _, REGSTR_KEY_DEVICE_PROPERTIES.as_ptr());
                let mut propertiesHandle: HANDLE = core::ptr::null_mut();

                Status = IopOpenRegistryKeyEx(&mut propertiesHandle as *mut _, ClassKey, &mut properties as *mut _, KEY_READ);
                if Status >= 0 {
                    ZwClose(propertiesHandle);
                }
            }
        }

        ExFreePool(kvInfo as PVOID);
    }

    let mut drvListHead: _LIST_ENTRY = core::mem::zeroed();
    InitializeListHead(&mut drvListHead as *mut _);

    Status = PiAttachFilterDrivers(&mut drvListHead as *mut _, DeviceNode, SubKey, ClassKey, 1, LoadDrivers);
    if Status < 0 {
        while !IsListEmpty(&mut drvListHead as *mut _) {
            let listEntry = RemoveHeadList(&mut drvListHead as *mut _);
            let driverEntry = CONTAINING_RECORD(listEntry as PVOID, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>(), 0) as PADD_DEV_DRIVERS_LIST;
            if !(*driverEntry).DriverObject.is_null() {
                ObDereferenceObject((*driverEntry).DriverObject as PVOID);
            }
            ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
        }
        ZwClose(SubKey);
        if !ClassKey.is_null() {
            ZwClose(ClassKey);
        }
        return Status;
    }

    let mut routineContext = ATTACH_FILTER_DRIVERS_CONTEXT {
        DriversListHead: &mut drvListHead as *mut _,
        DriverType: ADD_DEV_DRIVER_TYPE::DeviceDriver,
        DeviceNode,
    };

    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: Some(core::mem::transmute(PiAttachFilterDriversCallback as unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS)),
            Name: w!("Service").as_ptr(),
            Flags: RTL_QUERY_REGISTRY_REQUIRED,
            DefaultType: REG_SZ,
            DefaultData: b"\0\0\0\0" as *const u8 as PVOID,
            EntryContext: LoadDrivers as ULONG_PTR as PVOID,
            ..core::mem::zeroed()
        },
        core::mem::zeroed(),
    ];

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_HANDLE,
        SubKey as PWSTR,
        queryTable.as_mut_ptr(),
        &mut routineContext as *mut _ as PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
    } else if Status == STATUS_OBJECT_NAME_NOT_FOUND && ((*DeviceNode).CapabilityFlags & 0x00000040) != 0 {
        let driverEntry = ExAllocatePoolZero(PagedPool, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>() as _, TAG_PNP_DEVACTION) as PADD_DEV_DRIVERS_LIST;
        (*driverEntry).DriverType = ADD_DEV_DRIVER_TYPE::DeviceDriver;
        InsertTailList(&mut drvListHead as *mut _, &mut (*driverEntry).ListEntry as *mut _);
    } else {
        if Status == STATUS_OBJECT_TYPE_MISMATCH && ((*DeviceNode).Flags & DNF_HAS_PROBLEM) == 0 {
            PiSetDevNodeProblem(DeviceNode, CM_PROB_REGISTRY);
        }
        while !IsListEmpty(&mut drvListHead as *mut _) {
            let listEntry = RemoveHeadList(&mut drvListHead as *mut _);
            let driverEntry = CONTAINING_RECORD(listEntry as PVOID, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>(), 0) as PADD_DEV_DRIVERS_LIST;
            if !(*driverEntry).DriverObject.is_null() {
                ObDereferenceObject((*driverEntry).DriverObject as PVOID);
            }
            ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
        }
        ZwClose(SubKey);
        if !ClassKey.is_null() {
            ZwClose(ClassKey);
        }
        return Status;
    }

    Status = PiAttachFilterDrivers(&mut drvListHead as *mut _, DeviceNode, SubKey, ClassKey, 0, LoadDrivers);
    if Status < 0 {
        while !IsListEmpty(&mut drvListHead as *mut _) {
            let listEntry = RemoveHeadList(&mut drvListHead as *mut _);
            let driverEntry = CONTAINING_RECORD(listEntry as PVOID, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>(), 0) as PADD_DEV_DRIVERS_LIST;
            if !(*driverEntry).DriverObject.is_null() {
                ObDereferenceObject((*driverEntry).DriverObject as PVOID);
            }
            ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
        }
        ZwClose(SubKey);
        if !ClassKey.is_null() {
            ZwClose(ClassKey);
        }
        return Status;
    }

    let mut listEntry = drvListHead.Flink;
    while listEntry != &mut drvListHead as *mut _ {
        let driverEntry = CONTAINING_RECORD(listEntry as PVOID, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>(), 0) as PADD_DEV_DRIVERS_LIST;
        let driverObject = (*driverEntry).DriverObject;

        if !driverObject.is_null() && !(*driverObject).DriverExtension.is_null() && (*(*driverObject).DriverExtension).AddDevice.is_some() {
            Status = ((*(*driverObject).DriverExtension).AddDevice.unwrap())((*driverEntry).DriverObject, (*DeviceNode).PhysicalDeviceObject);
            crate::dbg_print!(
                b"[DA] AddDevice %wZ (drv %wZ) -> 0x%x\r\n\0",
                core::ptr::addr_of!((*DeviceNode).InstancePath),
                core::ptr::addr_of!((*driverObject).DriverName),
                Status as u32
            );
        } else if driverObject.is_null() {
            Status = STATUS_SUCCESS;
        } else {
            crate::dbg_print!(
                b"[DA] AddDevice %wZ: legacy driver (no AddDevice) -> UNSUCCESSFUL\r\n\0",
                core::ptr::addr_of!((*DeviceNode).InstancePath)
            );
            (*DeviceNode).Flags |= DNF_LEGACY_DRIVER;
            Status = STATUS_UNSUCCESSFUL;
        }

        if matches!((*driverEntry).DriverType, ADD_DEV_DRIVER_TYPE::DeviceDriver) {
            if Status >= 0 {
                let fdo = IoGetAttachedDeviceReference((*DeviceNode).PhysicalDeviceObject);

                if (*fdo).DeviceType == FILE_DEVICE_ACPI {
                    static mut SystemPowerDeviceNodeCreated: BOOLEAN = 0;

                    if SystemPowerDeviceNodeCreated == 0 {
                        PopSystemPowerDeviceNode = DeviceNode;
                        ObReferenceObject((*PopSystemPowerDeviceNode).PhysicalDeviceObject as PVOID);
                        SystemPowerDeviceNodeCreated = 1;
                    }
                }

                ObDereferenceObject(fdo as PVOID);
                PiSetDevNodeState(DeviceNode, DeviceNodeDriversAdded);
            } else {
                PiSetDevNodeProblem(DeviceNode, CM_PROB_FAILED_ADD);
                PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
                break;
            }
        }

        listEntry = (*listEntry).Flink;
    }

    while !IsListEmpty(&mut drvListHead as *mut _) {
        let listEntry = RemoveHeadList(&mut drvListHead as *mut _);
        let driverEntry = CONTAINING_RECORD(listEntry as PVOID, core::mem::size_of::<ADD_DEV_DRIVERS_LIST>(), 0) as PADD_DEV_DRIVERS_LIST;

        if !(*driverEntry).DriverObject.is_null() {
            ObDereferenceObject((*driverEntry).DriverObject as PVOID);
        }
        ExFreePoolWithTag(driverEntry as PVOID, TAG_PNP_DEVACTION);
    }

    ZwClose(SubKey);
    if !ClassKey.is_null() {
        ZwClose(ClassKey);
    }

    Status
}

/// `IopQueryDeviceCapabilities` — issue an `IRP_MN_QUERY_CAPABILITIES` to the
/// PDO, copy a curated subset of the result into `DEVICE_NODE::CapabilityFlags`,
/// and persist `Capabilities` + `UINumber` into the device's registry instance
/// key. Direct port of `IopQueryDeviceCapabilities_c` from `io_rust_shims.c` —
/// the historical "amd64-specific hang" was a captures(none) ghost (same family
/// as `Ob*_shim` / `IopGetBusTypeGuidIndex` / `Pi*DeviceAction`).
#[no_mangle]
pub unsafe extern "system" fn IopQueryDeviceCapabilities(
    DeviceNode: PDEVICE_NODE,
    DeviceCaps: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let DeviceNode = opaque_ptr(DeviceNode);
    let DeviceCaps = opaque_ptr(DeviceCaps);

    let mut StatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();

    core::ptr::write_bytes(
        DeviceCaps as *mut u8,
        0,
        core::mem::size_of::<_DEVICE_CAPABILITIES>(),
    );
    (*DeviceCaps).Size = core::mem::size_of::<_DEVICE_CAPABILITIES>() as USHORT;
    (*DeviceCaps).Version = 1;
    (*DeviceCaps).Address = 0xFFFFFFFF;
    (*DeviceCaps).UINumber = 0xFFFFFFFF;

    Stack.Parameters.DeviceCapabilities.Capabilities = DeviceCaps;

    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut StatusBlock as PIO_STATUS_BLOCK,
        IRP_MN_QUERY_CAPABILITIES,
        &mut Stack as PIO_STACK_LOCATION,
    );
    if Status < 0 {
        return Status;
    }

    (*DeviceNode).CapabilityFlags = 0;
    if (*DeviceCaps).LockSupported() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000001;
    }
    if (*DeviceCaps).EjectSupported() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000002;
    }
    if (*DeviceCaps).Removable() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000004;
    }
    if (*DeviceCaps).DockDevice() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000008;
    }
    if (*DeviceCaps).UniqueID() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000010;
    }
    if (*DeviceCaps).SilentInstall() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000020;
    }
    if (*DeviceCaps).RawDeviceOK() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000040;
    }
    if (*DeviceCaps).SurpriseRemovalOK() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000080;
    }
    if (*DeviceCaps).HardwareDisabled() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000100;
    }
    if (*DeviceCaps).NonDynamic() != 0 {
        (*DeviceNode).CapabilityFlags |= 0x00000200;
    }

    if (*DeviceCaps).NoDisplayInUI() != 0 {
        (*DeviceNode).UserFlags |= DNUF_DONT_SHOW_IN_UI;
    } else {
        (*DeviceNode).UserFlags &= !DNUF_DONT_SHOW_IN_UI;
    }

    Status = IopCreateDeviceKeyPath(
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
        REG_OPTION_NON_VOLATILE,
        &mut InstanceKey,
    );
    if Status >= 0 {
        // L"Capabilities"
        static CAPABILITIES_NAME: [u16; 13] = [
            b'C' as u16, b'a' as u16, b'p' as u16, b'a' as u16, b'b' as u16,
            b'i' as u16, b'l' as u16, b'i' as u16, b't' as u16, b'i' as u16,
            b'e' as u16, b's' as u16, 0,
        ];
        // L"UINumber"
        static UINUMBER_NAME: [u16; 9] = [
            b'U' as u16, b'I' as u16, b'N' as u16, b'u' as u16, b'm' as u16,
            b'b' as u16, b'e' as u16, b'r' as u16, 0,
        ];

        RtlInitUnicodeString(&mut ValueName, CAPABILITIES_NAME.as_ptr());
        Status = ZwSetValueKey(
            InstanceKey,
            &mut ValueName,
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!((*DeviceNode).CapabilityFlags) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if (*DeviceCaps).UINumber != 0xFFFFFFFF {
            RtlInitUnicodeString(&mut ValueName, UINUMBER_NAME.as_ptr());
            Status = ZwSetValueKey(
                InstanceKey,
                &mut ValueName,
                0,
                REG_DWORD,
                core::ptr::addr_of_mut!((*DeviceCaps).UINumber) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
            );
        }
        ZwClose(InstanceKey);
    }

    Status
}

unsafe fn IopQueryHardwareIds(
    DeviceNode: PDEVICE_NODE,
    InstanceKey: HANDLE,
) -> NTSTATUS {
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Ptr: PWSTR;
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Length: ULONG;
    let mut TotalLength: ULONG;
    let mut IsValidID: BOOLEAN = 0;

    core::ptr::write_bytes(&mut Stack as *mut _ as *mut u8, 0, core::mem::size_of::<_IO_STACK_LOCATION>());
    Stack.Parameters.QueryId.IdType = BusQueryHardwareIDs;
    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_ID,
        &mut Stack as *mut _,
    );
    if Status >= 0 {
        IsValidID = IopValidateID(IoStatusBlock.Information as PWCHAR, BusQueryHardwareIDs);

        if IsValidID == 0 {
        }

        TotalLength = 0;

        Ptr = IoStatusBlock.Information as PWSTR;
        while *Ptr != 0 {
            Length = wcslen(Ptr) as ULONG + 1;

            Ptr = Ptr.add(Length as usize);
            TotalLength += Length;
        }

        RtlInitUnicodeString(&mut ValueName as *mut _, w!("HardwareID").as_ptr());
        Status = ZwSetValueKey(
            InstanceKey,
            &mut ValueName as *mut _,
            0,
            REG_MULTI_SZ,
            IoStatusBlock.Information as PVOID,
            (TotalLength + 1) * core::mem::size_of::<WCHAR>() as ULONG,
        );
        if Status < 0 {
        }
    } else {
    }

    Status
}

unsafe fn IopQueryCompatibleIds(
    DeviceNode: PDEVICE_NODE,
    InstanceKey: HANDLE,
) -> NTSTATUS {
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Ptr: PWSTR;
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Length: ULONG;
    let mut TotalLength: ULONG;
    let mut IsValidID: BOOLEAN = 0;

    core::ptr::write_bytes(&mut Stack as *mut _ as *mut u8, 0, core::mem::size_of::<_IO_STACK_LOCATION>());
    Stack.Parameters.QueryId.IdType = BusQueryCompatibleIDs;
    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_ID,
        &mut Stack as *mut _,
    );
    if Status >= 0 && IoStatusBlock.Information != 0 {
        IsValidID = IopValidateID(IoStatusBlock.Information as PWCHAR, BusQueryCompatibleIDs);

        if IsValidID == 0 {
        }

        TotalLength = 0;

        Ptr = IoStatusBlock.Information as PWSTR;
        while *Ptr != 0 {
            Length = wcslen(Ptr) as ULONG + 1;

            Ptr = Ptr.add(Length as usize);
            TotalLength += Length;
        }

        RtlInitUnicodeString(&mut ValueName as *mut _, w!("CompatibleIDs").as_ptr());
        Status = ZwSetValueKey(
            InstanceKey,
            &mut ValueName as *mut _,
            0,
            REG_MULTI_SZ,
            IoStatusBlock.Information as PVOID,
            (TotalLength + 1) * core::mem::size_of::<WCHAR>() as ULONG,
        );
        if Status < 0 {
        }
    } else {
    }

    Status
}

/// `PiSetDevNodeText` — populate `DeviceDesc` and `LocationInformation`
/// registry values for a device's instance key. Direct port of the C body
/// `PiSetDevNodeText_c` (another captures(none) ghost).
///
/// `DeviceDesc` is only written if the value doesn't already exist (so user
/// edits are preserved across reboots); `LocationInformation` is overwritten
/// unconditionally on every call. If the driver doesn't return a description
/// the value gets the literal "Unknown device".
#[no_mangle]
pub unsafe extern "C" fn PiSetDevNodeText(
    DeviceNode: PDEVICE_NODE,
    InstanceKey: HANDLE,
) {
    let DeviceNode = opaque_ptr(DeviceNode);
    let InstanceKey = opaque_ptr(InstanceKey);

    let mut localeId: LCID = 0;
    let mut status: NTSTATUS = ZwQueryDefaultLocale(0, &mut localeId);
    if status < 0 {
        return;
    }

    // L"DeviceDesc"
    static DEVICE_DESC_NAME: [u16; 11] = [
        b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, 0,
    ];
    // L"LocationInformation"
    static LOCATION_INFO_NAME: [u16; 20] = [
        b'L' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b't' as u16,
        b'i' as u16, b'o' as u16, b'n' as u16, b'I' as u16, b'n' as u16,
        b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16, b'a' as u16,
        b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
    ];
    // L"Unknown device"
    static UNKNOWN_DEVICE: [u16; 15] = [
        b'U' as u16, b'n' as u16, b'k' as u16, b'n' as u16, b'o' as u16,
        b'w' as u16, b'n' as u16, b' ' as u16, b'd' as u16, b'e' as u16,
        b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, 0,
    ];

    let mut valDeviceDesc = _UNICODE_STRING {
        Length: ((DEVICE_DESC_NAME.len() - 1) * 2) as USHORT,
        MaximumLength: (DEVICE_DESC_NAME.len() * 2) as USHORT,
        Buffer: DEVICE_DESC_NAME.as_ptr() as *mut u16,
    };

    // Step 1: Write DeviceDesc only if it doesn't already exist.
    let mut len: ULONG = 0;
    status = ZwQueryValueKey(
        InstanceKey,
        &mut valDeviceDesc,
        KeyValueBasicInformation,
        core::ptr::null_mut(),
        0,
        &mut len,
    );
    if status == STATUS_OBJECT_NAME_NOT_FOUND {
        let mut deviceDesc: PWSTR = core::ptr::null_mut();
        status = PiIrpQueryDeviceText(DeviceNode, localeId, DeviceTextDescription, &mut deviceDesc);

        if !deviceDesc.is_null() && *deviceDesc != 0 {
            let bytes = ((wcslen(deviceDesc) + 1) * 2) as ULONG;
            ZwSetValueKey(
                InstanceKey,
                &mut valDeviceDesc,
                0,
                REG_SZ,
                deviceDesc as PVOID,
                bytes,
            );
        } else {
            ZwSetValueKey(
                InstanceKey,
                &mut valDeviceDesc,
                0,
                REG_SZ,
                UNKNOWN_DEVICE.as_ptr() as PVOID,
                (UNKNOWN_DEVICE.len() * 2) as ULONG,
            );
        }

        if !deviceDesc.is_null() {
            ExFreePoolWithTag(deviceDesc as PVOID, 0);
        }
    }

    // Step 2: LocationInformation — overwrite unconditionally.
    let mut deviceLocationInfo: PWSTR = core::ptr::null_mut();
    status = PiIrpQueryDeviceText(
        DeviceNode,
        localeId,
        DeviceTextLocationInformation,
        &mut deviceLocationInfo,
    );
    let _ = status; // C original ignores the return value here too

    if !deviceLocationInfo.is_null() && *deviceLocationInfo != 0 {
        let mut valLocationInfo = _UNICODE_STRING {
            Length: ((LOCATION_INFO_NAME.len() - 1) * 2) as USHORT,
            MaximumLength: (LOCATION_INFO_NAME.len() * 2) as USHORT,
            Buffer: LOCATION_INFO_NAME.as_ptr() as *mut u16,
        };
        let bytes = ((wcslen(deviceLocationInfo) + 1) * 2) as ULONG;
        ZwSetValueKey(
            InstanceKey,
            &mut valLocationInfo,
            0,
            REG_SZ,
            deviceLocationInfo as PVOID,
            bytes,
        );
    }

    if !deviceLocationInfo.is_null() {
        ExFreePoolWithTag(deviceLocationInfo as PVOID, 0);
    }
}

unsafe fn PiInitializeDevNode(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut InstancePathU: _UNICODE_STRING = core::mem::zeroed();
    let OldDeviceObject: PDEVICE_OBJECT;

    Status = IopCreateDeviceInstancePath(DeviceNode, &mut InstancePathU as *mut _);
    if Status < 0 {
        crate::dbg_print!(
            b"[DA] PiInitializeDevNode pdo=%p: CreateDeviceInstancePath FAILED 0x%x\r\n\0",
            (*DeviceNode).PhysicalDeviceObject as u64,
            Status as u32
        );
        if Status != STATUS_PLUGPLAY_NO_DEVICE {
        }
        return Status;
    }

    OldDeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut InstancePathU as *mut _);
    if !OldDeviceObject.is_null() {
        let OldDeviceNode = IopGetDeviceNode(OldDeviceObject);

        KeBugCheckEx(
            PNP_DETECTED_FATAL_ERROR,
            0x01 as _,
            (*DeviceNode).PhysicalDeviceObject as _,
            OldDeviceObject as _,
            0 as _,
        );
    }

    (*DeviceNode).InstancePath = InstancePathU;

    Status = IopCreateDeviceKeyPath(&mut (*DeviceNode).InstancePath as *mut _, REG_OPTION_NON_VOLATILE, &mut InstanceKey as *mut _);
    if Status < 0 {
        return STATUS_SUCCESS;
    }

    IopQueryHardwareIds(DeviceNode, InstanceKey);

    IopQueryCompatibleIds(DeviceNode, InstanceKey);

    (*DeviceNode).Flags |= DNF_IDS_QUERIED;

    PiSetDevNodeText(DeviceNode, InstanceKey);

    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_BUS_INFORMATION,
        core::ptr::null_mut(),
    );
    if Status >= 0 && IoStatusBlock.Information != 0 {
        let BusInformation = IoStatusBlock.Information as PPNP_BUS_INFORMATION;

        (*DeviceNode).ChildBusNumber = (*BusInformation).BusNumber;
        (*DeviceNode).ChildInterfaceType = (*BusInformation).LegacyBusType;
        (*DeviceNode).ChildBusTypeIndex = IopGetBusTypeGuidIndex(&(*BusInformation).BusTypeGuid as *const _ as LPGUID);
        ExFreePoolWithTag(BusInformation as PVOID, 0);
    } else {
        (*DeviceNode).ChildBusNumber = 0xFFFFFFF0;
        (*DeviceNode).ChildInterfaceType = InterfaceTypeUndefined;
        (*DeviceNode).ChildBusTypeIndex = -1i16 as USHORT;
    }

    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_RESOURCES,
        core::ptr::null_mut(),
    );
    if Status >= 0 && IoStatusBlock.Information != 0 {
        (*DeviceNode).BootResources = IoStatusBlock.Information as PCM_RESOURCE_LIST;
        IopDeviceNodeSetFlag(DeviceNode, DNF_HAS_BOOT_CONFIG);
    } else {
        (*DeviceNode).BootResources = core::ptr::null_mut();
    }

    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_RESOURCE_REQUIREMENTS,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        (*DeviceNode).ResourceRequirements = IoStatusBlock.Information as PIO_RESOURCE_REQUIREMENTS_LIST;
    } else {
        (*DeviceNode).ResourceRequirements = core::ptr::null_mut();
    }

    if !InstanceKey.is_null() {
        IopSetDeviceInstanceData(InstanceKey, DeviceNode);
    }

    IopInstallCriticalDevice(DeviceNode);
    IopSetServiceEnumData(DeviceNode, InstanceKey);

    ZwClose(InstanceKey);

    PiSetDevNodeState(DeviceNode, DeviceNodeInitialized);

    if ((*DeviceNode).Flags & DNF_LEGACY_DRIVER) == 0 {
        IopQueueDeviceInstallEvent(
            &GUID_DEVICE_ENUMERATED as *const _ as *mut _,
            &mut (*DeviceNode).InstancePath as *mut _,
        );
    }

    STATUS_SUCCESS
}

unsafe fn IopSetServiceEnumData(
    DeviceNode: PDEVICE_NODE,
    InstanceHandle: HANDLE,
) -> NTSTATUS {
    // Uses the module-level ServicesKeyName array (53 u16s including null)
    let ServicesKeyPath = _UNICODE_STRING {
        Length: (ServicesKeyName.len() as u16 - 1) * 2,
        MaximumLength: ServicesKeyName.len() as u16 * 2,
        Buffer: ServicesKeyName.as_ptr() as PWSTR,
    };
    let mut ServiceKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut EnumKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ServiceName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyValueInformation: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut kvInfo2: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut ServiceKey: HANDLE = core::ptr::null_mut();
    let mut ServiceEnumKey: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0;
    let mut Count: ULONG = 0;
    let mut NextInstance: ULONG = 0;
    let mut ValueBuffer: [WCHAR; 6] = [0; 6];
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = IopGetRegistryValue(InstanceHandle, b"S\0e\0r\0v\0i\0c\0e\0\0\0" as *const u8 as PWSTR, &mut kvInfo2 as *mut _);
    if Status < 0 {
        return Status;
    }

    if ((*kvInfo2).Type != REG_SZ)
        || ((*kvInfo2).DataLength <= core::mem::size_of::<u16>() as ULONG)
        || ((*kvInfo2).DataLength > UNICODE_STRING_MAX_BYTES)
        || (((*kvInfo2).DataLength % core::mem::size_of::<WCHAR>() as ULONG) != 0)
    {
        ExFreePool(kvInfo2 as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    ServiceName.MaximumLength = (*kvInfo2).DataLength as u16;
    ServiceName.Length = ((*kvInfo2).DataLength - core::mem::size_of::<u16>() as ULONG) as u16;
    ServiceName.Buffer = (kvInfo2 as ULONG_PTR + (*kvInfo2).DataOffset as ULONG_PTR) as PWSTR;

    ServiceKeyName.MaximumLength = ServicesKeyPath.Length + ServiceName.Length + core::mem::size_of::<u16>() as u16;
    ServiceKeyName.Length = 0;
    ServiceKeyName.Buffer = ExAllocatePool(PagedPool, ServiceKeyName.MaximumLength as _) as PWSTR;
    if ServiceKeyName.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeStringToString(&mut ServiceKeyName as *mut _, &ServicesKeyPath as *const _ as *mut _);
    RtlAppendUnicodeStringToString(&mut ServiceKeyName as *mut _, &mut ServiceName as *mut _);

    Status = IopOpenRegistryKeyEx(&mut ServiceKey as *mut _, core::ptr::null_mut(), &mut ServiceKeyName as *mut _, KEY_CREATE_SUB_KEY);
    if Status < 0 {
        ExFreePool(ServiceKeyName.Buffer as PVOID);
        ExFreePool(kvInfo2 as PVOID);
        return Status;
    }

    Status = RtlDuplicateUnicodeString(
        RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING,
        &mut ServiceName as *mut _,
        &mut (*DeviceNode).ServiceName as *mut _,
    );
    if Status < 0 {
        if !ServiceEnumKey.is_null() {
            ZwClose(ServiceEnumKey);
        }
        if !ServiceKey.is_null() {
            ZwClose(ServiceKey);
        }
        ExFreePool(ServiceKeyName.Buffer as PVOID);
        ExFreePool(kvInfo2 as PVOID);
        return Status;
    }

    RtlInitUnicodeString(&mut EnumKeyName as *mut _, w!("Enum").as_ptr());
    Status = IopCreateRegistryKeyEx(
        &mut ServiceEnumKey as *mut _,
        ServiceKey,
        &mut EnumKeyName as *mut _,
        KEY_SET_VALUE,
        REG_OPTION_VOLATILE,
        &mut Disposition as *mut _,
    );
    if Status >= 0 {
        if Disposition == REG_OPENED_EXISTING_KEY {
            Status = IopGetRegistryValue(ServiceEnumKey, b"C\0o\0u\0n\0t\0\0\0" as *const u8 as PWSTR, &mut KeyValueInformation as *mut _);
            if Status < 0 {
                if !ServiceEnumKey.is_null() {
                    ZwClose(ServiceEnumKey);
                }
                if !ServiceKey.is_null() {
                    ZwClose(ServiceKey);
                }
                ExFreePool(ServiceKeyName.Buffer as PVOID);
                ExFreePool(kvInfo2 as PVOID);
                return Status;
            }

            if ((*KeyValueInformation).Type == REG_DWORD) && ((*KeyValueInformation).DataLength != 0) {
                Count = *(((KeyValueInformation as ULONG_PTR) + (*KeyValueInformation).DataOffset as ULONG_PTR) as *const ULONG);
            }

            ExFreePool(KeyValueInformation as PVOID);
            KeyValueInformation = core::ptr::null_mut();

            Status = IopGetRegistryValue(ServiceEnumKey, b"N\0e\0x\0t\0I\0n\0s\0t\0a\0n\0c\0e\0\0\0" as *const u8 as PWSTR, &mut KeyValueInformation as *mut _);
            if Status < 0 {
                if !ServiceEnumKey.is_null() {
                    ZwClose(ServiceEnumKey);
                }
                if !ServiceKey.is_null() {
                    ZwClose(ServiceKey);
                }
                ExFreePool(ServiceKeyName.Buffer as PVOID);
                ExFreePool(kvInfo2 as PVOID);
                return Status;
            }

            if ((*KeyValueInformation).Type == REG_DWORD) && ((*KeyValueInformation).DataLength != 0) {
                NextInstance = *(((KeyValueInformation as ULONG_PTR) + (*KeyValueInformation).DataOffset as ULONG_PTR) as *const ULONG);
            }

            ExFreePool(KeyValueInformation as PVOID);
            KeyValueInformation = core::ptr::null_mut();
        }

        crate::swprintf!(ValueBuffer.as_mut_ptr(), ValueBuffer.len(), crate::wstr!("%lu"), NextInstance);
        RtlInitUnicodeString(&mut ValueName as *mut _, ValueBuffer.as_ptr());
        Status = ZwSetValueKey(
            ServiceEnumKey,
            &mut ValueName as *mut _,
            0,
            REG_SZ,
            (*DeviceNode).InstancePath.Buffer as PVOID,
            (*DeviceNode).InstancePath.MaximumLength as ULONG,
        );
        if Status < 0 {
            if !ServiceEnumKey.is_null() {
                ZwClose(ServiceEnumKey);
            }
            if !ServiceKey.is_null() {
                ZwClose(ServiceKey);
            }
            ExFreePool(ServiceKeyName.Buffer as PVOID);
            ExFreePool(kvInfo2 as PVOID);
            return Status;
        }

        Count += 1;
        NextInstance += 1;

        RtlInitUnicodeString(&mut ValueName as *mut _, w!("Count").as_ptr());
        Status = ZwSetValueKey(
            ServiceEnumKey,
            &mut ValueName as *mut _,
            0,
            REG_DWORD,
            &Count as *const _ as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if Status < 0 {
            if !ServiceEnumKey.is_null() {
                ZwClose(ServiceEnumKey);
            }
            if !ServiceKey.is_null() {
                ZwClose(ServiceKey);
            }
            ExFreePool(ServiceKeyName.Buffer as PVOID);
            ExFreePool(kvInfo2 as PVOID);
            return Status;
        }

        RtlInitUnicodeString(&mut ValueName as *mut _, w!("NextInstance").as_ptr());
        Status = ZwSetValueKey(
            ServiceEnumKey,
            &mut ValueName as *mut _,
            0,
            REG_DWORD,
            &NextInstance as *const _ as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    if !ServiceEnumKey.is_null() {
        ZwClose(ServiceEnumKey);
    }

    if !ServiceKey.is_null() {
        ZwClose(ServiceKey);
    }

    ExFreePool(ServiceKeyName.Buffer as PVOID);
    ExFreePool(kvInfo2 as PVOID);

    Status
}

unsafe fn PiUpdateDeviceState(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut PnPFlags: PNP_DEVICE_STATE = 0;
    let mut Status: NTSTATUS;

    Status = PiIrpQueryPnPDeviceState(DeviceNode, &mut PnPFlags as *mut _);
    if Status < 0 {
        return Status;
    }

    if (PnPFlags & PNP_DEVICE_NOT_DISABLEABLE) != 0 {
        (*DeviceNode).UserFlags |= DNUF_NOT_DISABLEABLE;
    } else {
        (*DeviceNode).UserFlags &= !DNUF_NOT_DISABLEABLE;
    }

    if (PnPFlags & PNP_DEVICE_DONT_DISPLAY_IN_UI) != 0 {
        (*DeviceNode).UserFlags |= DNUF_DONT_SHOW_IN_UI;
    } else {
        (*DeviceNode).UserFlags &= !DNUF_DONT_SHOW_IN_UI;
    }

    if (PnPFlags & PNP_DEVICE_REMOVED) != 0 || (PnPFlags & PNP_DEVICE_DISABLED) != 0 {
        PiSetDevNodeProblem(
            DeviceNode,
            if (PnPFlags & PNP_DEVICE_DISABLED) != 0 {
                CM_PROB_HARDWARE_DISABLED
            } else {
                CM_PROB_DEVICE_NOT_THERE
            },
        );

        PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
    } else if (PnPFlags & PNP_DEVICE_RESOURCE_REQUIREMENTS_CHANGED) != 0 {
        if (PnPFlags & PNP_DEVICE_FAILED) != 0 {
            // `&= !`, not `&=` (upstream 1b460291e26): the typo wiped every other devnode flag.
            (*DeviceNode).Flags &= !DNF_NON_STOPPED_REBALANCE;
        } else {
            (*DeviceNode).Flags |= DNF_NON_STOPPED_REBALANCE;
        }

        (*DeviceNode).Flags &= !DNF_NO_RESOURCE_REQUIRED;

        (*DeviceNode).Flags |= DNF_RESOURCE_REQUIREMENTS_CHANGED;
    } else if (PnPFlags & PNP_DEVICE_FAILED) != 0 {
        PiSetDevNodeProblem(DeviceNode, CM_PROB_FAILED_POST_START);
        PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
    }

    STATUS_SUCCESS
}

unsafe fn PiStartDeviceFinal(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut DeviceCapabilities: _DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if ((*DeviceNode).Flags & DNF_IDS_QUERIED) == 0 {
        let mut enumRoot = _UNICODE_STRING {
            Length: (ENUM_ROOT.len() as u16 - 1) * 2,
            MaximumLength: ENUM_ROOT.len() as u16 * 2,
            Buffer: ENUM_ROOT.as_ptr() as PWSTR,
        };
        let mut enumRootHandle: HANDLE = core::ptr::null_mut();
        let mut instanceHandle: HANDLE = core::ptr::null_mut();

        Status = IopOpenRegistryKeyEx(&mut enumRootHandle as *mut _, core::ptr::null_mut(), &mut enumRoot as *mut _, KEY_READ);
        if Status < 0 {
            return Status;
        }

        Status = IopOpenRegistryKeyEx(&mut instanceHandle as *mut _, enumRootHandle, &mut (*DeviceNode).InstancePath as *mut _, KEY_READ);
        ZwClose(enumRootHandle);
        if Status < 0 {
            return Status;
        }

        IopQueryHardwareIds(DeviceNode, instanceHandle);
        IopQueryCompatibleIds(DeviceNode, instanceHandle);

        (*DeviceNode).Flags |= DNF_IDS_QUERIED;
        ZwClose(instanceHandle);
    }

    (*DeviceNode).Flags |= DNF_REENUMERATE;

    Status = IopQueryDeviceCapabilities(DeviceNode, &mut DeviceCapabilities as *mut _);
    if Status < 0 {
    }

    PiUpdateDeviceState(DeviceNode);

    IopQueueTargetDeviceEvent(&GUID_DEVICE_ARRIVAL as *const _ as *mut _, &mut (*DeviceNode).InstancePath as *mut _);

    PiSetDevNodeState(DeviceNode, DeviceNodeStarted);

    STATUS_SUCCESS
}

unsafe fn PiIrpSendRemoveCheckVpb(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: UCHAR,
) -> NTSTATUS {
    let mut oldIrql: KIRQL = 0;

    let mut vpbDevObj = DeviceObject;
    let mut targetDevice = DeviceObject;

    loop {
        if !(*vpbDevObj).Vpb.is_null() {
            KeWaitForSingleObject(&(*vpbDevObj).DeviceLock as *const _ as PVOID, _KWAIT_REASON_Executive, 0i32 as i8, 0, core::ptr::null_mut());
            IoAcquireVpbSpinLock(&mut oldIrql as *mut _);

            if MinorFunction == IRP_MN_CANCEL_REMOVE_DEVICE {
                (*(*vpbDevObj).Vpb).Flags &= !VPB_REMOVE_PENDING;
            } else {
                (*(*vpbDevObj).Vpb).Flags |= VPB_REMOVE_PENDING;
            }

            let isMounted = ((*(*vpbDevObj).Vpb).Flags & VPB_MOUNTED) != 0;

            if isMounted {
                targetDevice = (*(*vpbDevObj).Vpb).DeviceObject;
            }

            IoReleaseVpbSpinLock(oldIrql);
            KeSetEvent(&(*vpbDevObj).DeviceLock as *const _ as PKEVENT, IO_NO_INCREMENT as i32, 0);

            if isMounted {
                break;
            }
        }

        oldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
        vpbDevObj = (*vpbDevObj).AttachedDevice;
        KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, oldIrql);

        if vpbDevObj.is_null() {
            break;
        }
    }

    let mut info: PVOID = core::ptr::null_mut();
    let stack = _IO_STACK_LOCATION {
        MajorFunction: IRP_MJ_PNP,
        MinorFunction,
        ..core::mem::zeroed()
    };

    IopSynchronousCall(targetDevice, &stack as *const _ as *mut _, &mut info as *mut _)
}

extern "C" {
    fn IopUpdateResourceMapForPnPDevice(DeviceNode: PDEVICE_NODE) -> NTSTATUS;
}

unsafe extern "system" fn IopSendRemoveDevice(DeviceObject: PDEVICE_OBJECT) {
    let DeviceNode = IopGetDeviceNode(DeviceObject);

    PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_REMOVE_DEVICE);

    if !(*DeviceNode).ResourceList.is_null() {
        (*(*DeviceNode).ResourceList).Count = 0;
        (*(*DeviceNode).ResourceListTranslated).Count = 0;
        IopUpdateResourceMapForPnPDevice(DeviceNode);
    }

    PiSetDevNodeState(DeviceNode, DeviceNodeRemoved);
    PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_REMOVE_COMPLETE as *const _ as *mut _, DeviceObject, core::ptr::null_mut());
    ObDereferenceObject(DeviceObject as PVOID);
}

#[no_mangle]
pub unsafe extern "C" fn IopSendRemoveDeviceRelations(DeviceRelations: PDEVICE_RELATIONS) {
    let mut i: ULONG;

    for i in 0..(*DeviceRelations).Count {
        let object = device_relations_object(DeviceRelations, i);
        IopSendRemoveDevice(object);
        *device_relations_object_slot(DeviceRelations, i) = core::ptr::null_mut();
    }

    ExFreePool(DeviceRelations as PVOID);
}

#[no_mangle]
pub unsafe extern "C" fn IopSendRemoveChildDevices(ParentDeviceNode: PDEVICE_NODE) {
    let mut ChildDeviceNode: PDEVICE_NODE;
    let mut NextDeviceNode: PDEVICE_NODE;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
    ChildDeviceNode = (*ParentDeviceNode).Child;
    while !ChildDeviceNode.is_null() {
        NextDeviceNode = (*ChildDeviceNode).Sibling;
        KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);

        IopSendRemoveDevice((*ChildDeviceNode).PhysicalDeviceObject);

        ChildDeviceNode = NextDeviceNode;

        KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
    }
    KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);
}

unsafe extern "system" fn IopSendSurpriseRemoval(DeviceObject: PDEVICE_OBJECT) {
    PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_SURPRISE_REMOVAL);
}

unsafe extern "system" fn IopCancelRemoveDevice(DeviceObject: PDEVICE_OBJECT) {
    PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_CANCEL_REMOVE_DEVICE);

    PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_REMOVE_CANCELLED as *const _ as *mut _, DeviceObject, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "C" fn IopCancelRemoveChildDevices(ParentDeviceNode: PDEVICE_NODE) {
    let mut ChildDeviceNode: PDEVICE_NODE;
    let mut NextDeviceNode: PDEVICE_NODE;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
    ChildDeviceNode = (*ParentDeviceNode).Child;
    while !ChildDeviceNode.is_null() {
        NextDeviceNode = (*ChildDeviceNode).Sibling;
        KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);

        IopCancelPrepareDeviceForRemoval((*ChildDeviceNode).PhysicalDeviceObject);

        ChildDeviceNode = NextDeviceNode;

        KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
    }
    KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);
}

#[no_mangle]
pub unsafe extern "C" fn IopCancelRemoveDeviceRelations(DeviceRelations: PDEVICE_RELATIONS) {
    let mut i: ULONG;

    for i in 0..(*DeviceRelations).Count {
        let object = device_relations_object(DeviceRelations, i);
        IopCancelPrepareDeviceForRemoval(object);
        ObDereferenceObject(object as PVOID);
        *device_relations_object_slot(DeviceRelations, i) = core::ptr::null_mut();
    }

    ExFreePool(DeviceRelations as PVOID);
}

unsafe fn IopCancelPrepareDeviceForRemoval(DeviceObject: PDEVICE_OBJECT) {
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let mut Status: NTSTATUS;

    IopCancelRemoveDevice(DeviceObject);

    Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations;

    Status = IopInitiatePnpIrp(
        DeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_DEVICE_RELATIONS,
        &mut Stack as *mut _,
    );
    if Status < 0 {
        DeviceRelations = core::ptr::null_mut();
    } else {
        DeviceRelations = IoStatusBlock.Information as PDEVICE_RELATIONS;
    }

    if !DeviceRelations.is_null() {
        IopCancelRemoveDeviceRelations(DeviceRelations);
    }
}

unsafe extern "system" fn IopQueryRemoveDevice(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceNode = IopGetDeviceNode(DeviceObject);
    let mut Status: NTSTATUS;

    IopQueueTargetDeviceEvent(
        &GUID_DEVICE_REMOVE_PENDING as *const _ as *mut _,
        &mut (*DeviceNode).InstancePath as *mut _,
    );

    Status = PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_QUERY_REMOVE_DEVICE);

    PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_QUERY_REMOVE as *const _ as *mut _, DeviceObject, core::ptr::null_mut());

    if Status < 0 {
        IopQueueTargetDeviceEvent(
            &GUID_DEVICE_REMOVAL_VETOED as *const _ as *mut _,
            &mut (*DeviceNode).InstancePath as *mut _,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn IopQueryRemoveChildDevices(ParentDeviceNode: PDEVICE_NODE, Force: BOOLEAN) -> NTSTATUS {
    let mut ChildDeviceNode: PDEVICE_NODE;
    let mut NextDeviceNode: PDEVICE_NODE;
    let mut FailedRemoveDevice: PDEVICE_NODE;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
    ChildDeviceNode = (*ParentDeviceNode).Child;
    while !ChildDeviceNode.is_null() {
        NextDeviceNode = (*ChildDeviceNode).Sibling;
        KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);
        PiSetDevNodeState(ChildDeviceNode, DeviceNodeAwaitingQueuedRemoval);

        Status = IopPrepareDeviceForRemoval((*ChildDeviceNode).PhysicalDeviceObject, Force);
        if Status < 0 {
            FailedRemoveDevice = ChildDeviceNode;
            KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
            ChildDeviceNode = (*ParentDeviceNode).Child;
            while !ChildDeviceNode.is_null() {
                NextDeviceNode = (*ChildDeviceNode).Sibling;
                KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);

                IopCancelPrepareDeviceForRemoval((*ChildDeviceNode).PhysicalDeviceObject);

                if ChildDeviceNode == FailedRemoveDevice {
                    return Status;
                }

                ChildDeviceNode = NextDeviceNode;

                KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
            }
            KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);

            return Status;
        }

        KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
        ChildDeviceNode = NextDeviceNode;
    }
    KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopQueryRemoveDeviceRelations(DeviceRelations: PDEVICE_RELATIONS, Force: BOOLEAN) -> NTSTATUS {
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Status: NTSTATUS;

    for i in 0..(*DeviceRelations).Count {
        Status = IopPrepareDeviceForRemoval(device_relations_object(DeviceRelations, i), Force);
        if Status < 0 {
            j = i;
            for i in 0..=j {
                let object = device_relations_object(DeviceRelations, i);
                IopCancelPrepareDeviceForRemoval(object);
                ObDereferenceObject(object as PVOID);
                *device_relations_object_slot(DeviceRelations, i) = core::ptr::null_mut();
            }
            for i in (j + 1)..(*DeviceRelations).Count {
                let object = device_relations_object(DeviceRelations, i);
                ObDereferenceObject(object as PVOID);
                *device_relations_object_slot(DeviceRelations, i) = core::ptr::null_mut();
            }
            ExFreePool(DeviceRelations as PVOID);

            return Status;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopPrepareDeviceForRemoval(DeviceObject: PDEVICE_OBJECT, Force: BOOLEAN) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let DeviceNode = IopGetDeviceNode(DeviceObject);
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let mut Status: NTSTATUS;

    if ((*DeviceNode).UserFlags & DNUF_NOT_DISABLEABLE) != 0 && Force == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    if Force == 0 && IopQueryRemoveDevice(DeviceObject) != STATUS_SUCCESS {
        IopCancelRemoveDevice(DeviceObject);

        return STATUS_UNSUCCESSFUL;
    }

    Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations;

    Status = IopInitiatePnpIrp(
        DeviceObject,
        &mut IoStatusBlock as *mut _,
        IRP_MN_QUERY_DEVICE_RELATIONS,
        &mut Stack as *mut _,
    );
    if Status < 0 {
        DeviceRelations = core::ptr::null_mut();
    } else {
        DeviceRelations = IoStatusBlock.Information as PDEVICE_RELATIONS;
    }

    if !DeviceRelations.is_null() {
        Status = IopQueryRemoveDeviceRelations(DeviceRelations, Force);
        if Status < 0 {
            return Status;
        }
    }

    Status = IopQueryRemoveChildDevices(DeviceNode, Force);
    if Status < 0 {
        if !DeviceRelations.is_null() {
            IopCancelRemoveDeviceRelations(DeviceRelations);
        }
        return Status;
    }

    if !DeviceRelations.is_null() {
        IopSendRemoveDeviceRelations(DeviceRelations);
    }
    IopSendRemoveChildDevices(DeviceNode);

    STATUS_SUCCESS
}

unsafe fn IopRemoveDevice(DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    let surpriseRemoval: BOOLEAN = if ((*DeviceNode).Flags & DNF_DEVICE_GONE) != 0 { 1 } else { 0 };

    Status = IopPrepareDeviceForRemoval((*DeviceNode).PhysicalDeviceObject, surpriseRemoval);

    if surpriseRemoval != 0 {
        IopSendSurpriseRemoval((*DeviceNode).PhysicalDeviceObject);
        IopQueueTargetDeviceEvent(&GUID_DEVICE_SURPRISE_REMOVAL as *const _ as *mut _, &mut (*DeviceNode).InstancePath as *mut _);
    }

    if Status >= 0 {
        IopSendRemoveDevice((*DeviceNode).PhysicalDeviceObject);
        if surpriseRemoval != 0 {
            IopQueueTargetDeviceEvent(&GUID_DEVICE_SAFE_REMOVAL as *const _ as *mut _, &mut (*DeviceNode).InstancePath as *mut _);
        }
        return STATUS_SUCCESS;
    }

    Status
}

unsafe fn PiEnumerateDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut ChildDeviceObject: PDEVICE_OBJECT;
    let mut ChildDeviceNode: PDEVICE_NODE;
    let mut i: ULONG;

    if (*DeviceNode).CompletionStatus < 0 {
    }

    let DeviceRelations = (*DeviceNode).OverUsed1.PendingDeviceRelations;
    (*DeviceNode).OverUsed1.PendingDeviceRelations = core::ptr::null_mut();

    if DeviceRelations.is_null() {
        crate::dbg_print!(
            b"[DA] PiEnumerateDevice %wZ: no PDOs (cs=0x%x)\r\n\0",
            core::ptr::addr_of!((*DeviceNode).InstancePath),
            (*DeviceNode).CompletionStatus as u32
        );
        PiSetDevNodeState(DeviceNode, DeviceNodeStarted);
        return STATUS_SUCCESS;
    }

    crate::dbg_print!(
        b"[DA] PiEnumerateDevice %wZ: %u children (cs=0x%x)\r\n\0",
        core::ptr::addr_of!((*DeviceNode).InstancePath),
        (*DeviceRelations).Count,
        (*DeviceNode).CompletionStatus as u32
    );

    let mut child = (*DeviceNode).Child;
    while !child.is_null() {
        (*child).Flags &= !DNF_ENUMERATED;
        child = (*child).Sibling;
    }

    for i in 0..(*DeviceRelations).Count {
        ChildDeviceObject = device_relations_object(DeviceRelations, i);
        ChildDeviceNode = IopGetDeviceNode(ChildDeviceObject);
        if ChildDeviceNode.is_null() {
            ChildDeviceNode = PipAllocateDeviceNode(ChildDeviceObject);
            if !ChildDeviceNode.is_null() {
                PiInsertDevNode(ChildDeviceNode, DeviceNode);
                (*ChildDeviceNode).Flags |= DNF_ENUMERATED;
                (*ChildDeviceObject).Flags |= DO_BUS_ENUMERATED_DEVICE;
            } else {
                ObDereferenceObject(ChildDeviceObject as PVOID);
            }
        } else {
            (*ChildDeviceNode).Flags |= DNF_ENUMERATED;
            ObDereferenceObject(ChildDeviceObject as PVOID);
        }
    }
    ExFreePool(DeviceRelations as PVOID);

    child = (*DeviceNode).Child;
    while !child.is_null() {
        if ((*child).Flags & (DNF_ENUMERATED | DNF_DEVICE_GONE)) == 0 {
            (*child).Flags |= DNF_DEVICE_GONE;
            PiSetDevNodeState(child, DeviceNodeAwaitingQueuedRemoval);
        }
        child = (*child).Sibling;
    }

    PiSetDevNodeState(DeviceNode, DeviceNodeStarted);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IopSendEject(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut Dummy: PVOID = core::ptr::null_mut();

    core::ptr::write_bytes(&mut Stack as *mut _ as *mut u8, 0, core::mem::size_of::<_IO_STACK_LOCATION>());
    Stack.MajorFunction = IRP_MJ_PNP;
    Stack.MinorFunction = IRP_MN_EJECT;

    IopSynchronousCall(DeviceObject, &mut Stack as *mut _, &mut Dummy as *mut _)
}

/// `IoRequestDeviceEject` — request safe removal/ejection of a PDO.
///
/// Direct port of `IoRequestDeviceEject_c` from `io_rust_shims.c` (another
/// captures(none) ghost). The flow:
///   1. Notify subscribers via GUID_DEVICE_KERNEL_INITIATED_EJECT.
///   2. Query device capabilities.
///   3. Issue IRP_MN_QUERY_DEVICE_RELATIONS / EjectionRelations to gather
///      siblings that must be removed together.
///   4. Walk the relations + child devices and run query-remove on each;
///      bail to the veto path on any failure (cancelling already-prepared
///      devices in reverse order).
///   5. Send remove IRPs to all the devices, mark the PDO as
///      CM_PROB_HELD_FOR_EJECT, and (if `EjectSupported`) issue the
///      hardware-eject IRP.
///   6. On success: GUID_DEVICE_EJECT. On any failure: GUID_DEVICE_EJECT_VETOED.
#[no_mangle]
pub unsafe extern "system" fn IoRequestDeviceEject(PhysicalDeviceObject: PDEVICE_OBJECT) {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);

    let DeviceNode = IopGetDeviceNode(PhysicalDeviceObject);
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut Capabilities: _DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut Status: NTSTATUS;

    IopQueueTargetDeviceEvent(
        &GUID_DEVICE_KERNEL_INITIATED_EJECT as *const _ as *mut _,
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
    );

    // GOTO target: cleanup
    'cleanup: {
        if IopQueryDeviceCapabilities(DeviceNode, &mut Capabilities) != 0 {
            break 'cleanup;
        }

        Stack.Parameters.QueryDeviceRelations.Type = EjectionRelations;

        Status = IopInitiatePnpIrp(
            PhysicalDeviceObject,
            &mut IoStatusBlock as PIO_STATUS_BLOCK,
            IRP_MN_QUERY_DEVICE_RELATIONS,
            &mut Stack as PIO_STACK_LOCATION,
        );
        if Status < 0 {
            DeviceRelations = core::ptr::null_mut();
        } else {
            DeviceRelations = IoStatusBlock.Information as PDEVICE_RELATIONS;
        }

        if !DeviceRelations.is_null() {
            Status = IopQueryRemoveDeviceRelations(DeviceRelations, 0);
            if Status < 0 {
                break 'cleanup;
            }
        }

        Status = IopQueryRemoveChildDevices(DeviceNode, 0);
        if Status < 0 {
            if !DeviceRelations.is_null() {
                IopCancelRemoveDeviceRelations(DeviceRelations);
            }
            break 'cleanup;
        }

        if IopPrepareDeviceForRemoval(PhysicalDeviceObject, 0) != 0 {
            if !DeviceRelations.is_null() {
                IopCancelRemoveDeviceRelations(DeviceRelations);
            }
            IopCancelRemoveChildDevices(DeviceNode);
            break 'cleanup;
        }

        if !DeviceRelations.is_null() {
            IopSendRemoveDeviceRelations(DeviceRelations);
        }
        IopSendRemoveChildDevices(DeviceNode);

        (*DeviceNode).Problem = CM_PROB_HELD_FOR_EJECT;
        if Capabilities.EjectSupported() != 0 {
            if IopSendEject(PhysicalDeviceObject) != 0 {
                break 'cleanup;
            }
        } else {
            // (*DeviceNode).Flags |= DNF_DISABLED;  // intentionally commented in C
        }

        IopQueueTargetDeviceEvent(
            &GUID_DEVICE_EJECT as *const _ as *mut _,
            core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
        );

        return;
    }

    // cleanup:
    IopQueueTargetDeviceEvent(
        &GUID_DEVICE_EJECT_VETOED as *const _ as *mut _,
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
    );
}

unsafe fn PiFakeResourceRebalance(
    DeviceNode: PDEVICE_NODE,
) {
    let mut bootConfig: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut resourceRequirements: PIO_RESOURCE_REQUIREMENTS_LIST = core::ptr::null_mut();

    PiIrpQueryResources(DeviceNode, &mut bootConfig as *mut _);
    PiIrpQueryResourceRequirements(DeviceNode, &mut resourceRequirements as *mut _);

    (*DeviceNode).BootResources = bootConfig;
    (*DeviceNode).ResourceRequirements = resourceRequirements;

    if !bootConfig.is_null() {
        (*DeviceNode).Flags |= DNF_HAS_BOOT_CONFIG;
    }

    (*DeviceNode).Flags &= !DNF_RESOURCE_REQUIREMENTS_CHANGED;
}

unsafe fn PiDevNodeStateMachine(
    RootNode: PDEVICE_NODE,
) {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut doProcessAgain: BOOLEAN;
    let mut currentNode = RootNode;
    let mut referencedObject: PDEVICE_OBJECT;

    loop {
        doProcessAgain = 0;

        referencedObject = (*currentNode).PhysicalDeviceObject;
        ObReferenceObject(referencedObject as PVOID);

        if ((*currentNode).Flags & DNF_HAS_PROBLEM) != 0
            && (*currentNode).State != DeviceNodeAwaitingQueuedRemoval
        {
            let mut OldIrql: KIRQL = 0;
            KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
            if (*currentNode).State != DeviceNodeRemoved && !(*currentNode).Child.is_null() {
                currentNode = (*currentNode).Child;
            } else {
                while currentNode != RootNode {
                    if !(*currentNode).Sibling.is_null() {
                        currentNode = (*currentNode).Sibling;
                        break;
                    } else {
                        currentNode = (*currentNode).Parent;
                    }
                }
            }
            KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);
            ObDereferenceObject(referencedObject as PVOID);
            if currentNode == RootNode {
                break;
            }
            continue;
        }

        match (*currentNode).State {
            DeviceNodeUnspecified => {}
            DeviceNodeUninitialized => {
                status = PiInitializeDevNode(currentNode);
                doProcessAgain = if status >= 0 { 1 } else { 0 };
            }
            DeviceNodeInitialized => {
                status = PiCallDriverAddDevice(currentNode, PnPBootDriversInitialized);
                doProcessAgain = if status >= 0 { 1 } else { 0 };
            }
            DeviceNodeDriversAdded => {
                status = IopAssignDeviceResources(currentNode);
                doProcessAgain = if status >= 0 { 1 } else { 0 };
            }
            DeviceNodeResourcesAssigned => {
                PiIrpStartDevice(currentNode);

                PiSetDevNodeState(currentNode, DeviceNodeStartCompletion);
                doProcessAgain = 1;
            }
            DeviceNodeStartPending => {}
            DeviceNodeStartCompletion => {
                status = (*currentNode).CompletionStatus;
                doProcessAgain = 1;
                crate::dbg_print!(
                    b"[DA] START %wZ -> 0x%x\r\n\0",
                    core::ptr::addr_of!((*currentNode).InstancePath),
                    status as u32
                );
                if status < 0 {
                    let problem = if status == STATUS_PNP_REBOOT_REQUIRED {
                        CM_PROB_NEED_RESTART
                    } else {
                        CM_PROB_FAILED_START
                    };

                    PiSetDevNodeProblem(currentNode, problem);
                    PiSetDevNodeState(currentNode, DeviceNodeAwaitingQueuedRemoval);
                } else {
                    PiSetDevNodeState(currentNode, DeviceNodeStartPostWork);
                }
            }
            DeviceNodeStartPostWork => {
                status = PiStartDeviceFinal(currentNode);
                doProcessAgain = 1;
            }
            DeviceNodeStarted => {
                if ((*currentNode).Flags & DNF_REENUMERATE) != 0 {
                    (*currentNode).Flags &= !DNF_REENUMERATE;
                    status = PiIrpQueryDeviceRelations(currentNode, BusRelations);

                    crate::dbg_print!(
                        b"[DA] QDR(bus) %wZ -> 0x%x rel=%p\r\n\0",
                        core::ptr::addr_of!((*currentNode).InstancePath),
                        status as u32,
                        (*currentNode).OverUsed1.PendingDeviceRelations as u64
                    );

                    PiSetDevNodeState(currentNode, DeviceNodeEnumerateCompletion);
                    doProcessAgain = 1;
                } else if ((*currentNode).Flags & DNF_RESOURCE_REQUIREMENTS_CHANGED) != 0 {
                    if ((*currentNode).Flags & DNF_NON_STOPPED_REBALANCE) != 0 {
                        PiFakeResourceRebalance(currentNode);
                        (*currentNode).Flags &= !DNF_NON_STOPPED_REBALANCE;
                    } else {
                        PiIrpQueryStopDevice(currentNode);
                        PiSetDevNodeState(currentNode, DeviceNodeQueryStopped);
                    }

                    doProcessAgain = 1;
                }
            }
            DeviceNodeQueryStopped => {
                status = (*currentNode).CompletionStatus;
                if status >= 0 {
                    PiIrpStopDevice(currentNode);
                    PiSetDevNodeState(currentNode, DeviceNodeStopped);
                } else {
                    PiIrpCancelStopDevice(currentNode);
                    PiSetDevNodeState(currentNode, DeviceNodeStarted);
                }
                doProcessAgain = 1;
            }
            DeviceNodeStopped => {
                PiFakeResourceRebalance(currentNode);

                PiSetDevNodeState(currentNode, DeviceNodeDriversAdded);
                doProcessAgain = 1;
            }
            DeviceNodeRestartCompletion => {}
            DeviceNodeEnumeratePending => {}
            DeviceNodeEnumerateCompletion => {
                status = PiEnumerateDevice(currentNode);
                doProcessAgain = 1;
            }
            DeviceNodeAwaitingQueuedDeletion => {}
            DeviceNodeAwaitingQueuedRemoval => {
                status = IopRemoveDevice(currentNode);
            }
            DeviceNodeQueryRemoved => {}
            DeviceNodeRemovePendingCloses => {}
            DeviceNodeRemoved => {}
            DeviceNodeDeletePendingCloses => {}
            DeviceNodeDeleted => {}
            _ => {}
        }

        if doProcessAgain == 0 {
            let mut OldIrql: KIRQL = 0;
            KeAcquireSpinLock(device_tree_lock_ptr(), &mut OldIrql as *mut _);
            if (*currentNode).State != DeviceNodeRemoved && !(*currentNode).Child.is_null() {
                currentNode = (*currentNode).Child;
            } else {
                while currentNode != RootNode {
                    if !(*currentNode).Sibling.is_null() {
                        currentNode = (*currentNode).Sibling;
                        break;
                    } else {
                        currentNode = (*currentNode).Parent;
                    }
                }
            }
            KeReleaseSpinLock(device_tree_lock_ptr(), OldIrql);
        }
        ObDereferenceObject(referencedObject as PVOID);

        if currentNode == RootNode && doProcessAgain == 0 {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PipDeviceActionWorker(
    Context: PVOID,
) {
    let Context = opaque_ptr(Context);
    let mut ListEntry: PLIST_ENTRY;
    let mut Request: PDEVICE_ACTION_REQUEST;
    let mut OldIrql: KIRQL = 0;
    let mut deviceNode: PDEVICE_NODE;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    KeAcquireSpinLock(device_action_lock_ptr(), &mut OldIrql as *mut _);
    while !IsListEmpty(device_action_request_list_ptr()) {
        ListEntry = RemoveHeadList(device_action_request_list_ptr());
        KeReleaseSpinLock(device_action_lock_ptr(), OldIrql);
        Request = CONTAINING_RECORD(ListEntry as PVOID, core::mem::size_of::<DEVICE_ACTION_REQUEST>(), 0) as PDEVICE_ACTION_REQUEST;

        deviceNode = IopGetDeviceNode((*Request).DeviceObject);

        status = STATUS_SUCCESS;

        match (*Request).Action {
            PiActionAddBootDevices => {
                if (*deviceNode).State == DeviceNodeInitialized
                    && ((*deviceNode).Flags & DNF_HAS_PROBLEM) == 0
                {
                    status = PiCallDriverAddDevice(deviceNode, PnPBootDriversInitialized);
                }
            }
            PiActionEnumRootDevices | PiActionEnumDeviceTree => {
                (*deviceNode).Flags |= DNF_REENUMERATE;
                PiDevNodeStateMachine(deviceNode);
            }
            PiActionResetDevice => {
                if (*deviceNode).State == DeviceNodeRemoved {
                    (*deviceNode).State = DeviceNodeUninitialized;
                }
                status = STATUS_SUCCESS;
            }
            PiActionStartDevice => {
                if (*deviceNode).State == DeviceNodeInitialized
                    && ((*deviceNode).Flags & DNF_HAS_PROBLEM) == 0
                {
                    PiDevNodeStateMachine(deviceNode);
                } else {
                    status = STATUS_UNSUCCESSFUL;
                }
            }
            PiActionQueryState => {
                if (*deviceNode).State == DeviceNodeStarted {
                    status = PiUpdateDeviceState(deviceNode);
                    if status >= 0 {
                        PiDevNodeStateMachine(deviceNode);
                    }
                }
                status = STATUS_SUCCESS;
            }
            _ => {
                status = STATUS_NOT_IMPLEMENTED;
            }
        }

        if !(*Request).CompletionStatus.is_null() {
            *(*Request).CompletionStatus = status;
        }

        if !(*Request).CompletionEvent.is_null() {
            KeSetEvent((*Request).CompletionEvent, IO_NO_INCREMENT as i32, 0);
        }

        ObDereferenceObject((*Request).DeviceObject as PVOID);
        ExFreePoolWithTag(Request as PVOID, TAG_IO);
        KeAcquireSpinLock(device_action_lock_ptr(), &mut OldIrql as *mut _);
    }
    IopDeviceActionInProgress = 0;
    KeSetEvent(enumeration_finished_ptr(), IO_NO_INCREMENT as i32, 0);
    KeReleaseSpinLock(device_action_lock_ptr(), OldIrql);
}

/// `PiQueueDeviceAction` — enqueue a device action onto `IopDeviceActionRequestList`,
/// kicking off a worker if not already running. For `PiActionEnumRootDevices` and
/// `PiActionAddBootDevices` the worker runs synchronously on the caller's thread;
/// for everything else we queue an `ExWorkItem` to fire `PipDeviceActionWorker`
/// in the delayed work queue.
///
/// Direct port of the C body that lived in `io_rust_shims.c::PiQueueDeviceAction_c`.
/// The historical "Delegate to C implementation" comment was a captures(none)
/// ghost (same family as `Ob*_shim` and `IopGetBusTypeGuidIndex`); the workspace
/// `lto = false` + `codegen-units = 256` firewall keeps the callees in
/// different CGUs.
#[no_mangle]
pub unsafe extern "C" fn PiQueueDeviceAction(
    DeviceObject: PDEVICE_OBJECT,
    Action: DEVICE_ACTION,
    CompletionEvent: PKEVENT,
    CompletionStatus: *mut NTSTATUS,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let CompletionEvent = opaque_ptr(CompletionEvent);
    let CompletionStatus = opaque_ptr(CompletionStatus);

    let Request = ExAllocatePoolWithTag(
        NonPagedPoolMustSucceed,
        core::mem::size_of::<DEVICE_ACTION_REQUEST>() as ULONG_PTR,
        TAG_IO,
    ) as PDEVICE_ACTION_REQUEST;

    ObReferenceObject(DeviceObject as PVOID);

    (*Request).DeviceObject = DeviceObject;
    (*Request).Action = Action;
    (*Request).CompletionEvent = CompletionEvent;
    (*Request).CompletionStatus = CompletionStatus;

    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock(device_action_lock_ptr(), &mut OldIrql);
    InsertTailList(
        device_action_request_list_ptr(),
        core::ptr::addr_of_mut!((*Request).RequestListEntry) as PLIST_ENTRY,
    );

    if Action == PiActionEnumRootDevices || Action == PiActionAddBootDevices {
        IopDeviceActionInProgress = 1;
        KeClearEvent(enumeration_finished_ptr());
        KeReleaseSpinLock(device_action_lock_ptr(), OldIrql);

        PipDeviceActionWorker(core::ptr::null_mut());
        return;
    }

    if IopDeviceActionInProgress != 0 || PnPBootDriversLoaded == 0 {
        KeReleaseSpinLock(device_action_lock_ptr(), OldIrql);
        return;
    }
    IopDeviceActionInProgress = 1;
    KeClearEvent(enumeration_finished_ptr());
    KeReleaseSpinLock(device_action_lock_ptr(), OldIrql);

    ExInitializeWorkItem(
        core::ptr::addr_of_mut!(IopDeviceActionWorkItem) as PWORK_QUEUE_ITEM,
        PipDeviceActionWorker,
        core::ptr::null_mut(),
    );
    ExQueueWorkItem(
        core::ptr::addr_of_mut!(IopDeviceActionWorkItem) as PWORK_QUEUE_ITEM,
        DelayedWorkQueue,
    );
}

/// `PiPerformSyncDeviceAction` — synchronous variant: enqueue + wait on a
/// stack-local KEVENT. Direct port of `PiPerformSyncDeviceAction_c`.
#[no_mangle]
pub unsafe extern "C" fn PiPerformSyncDeviceAction(
    DeviceObject: PDEVICE_OBJECT,
    Action: DEVICE_ACTION,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);

    let mut opFinished: _KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS = 0;

    KeInitializeEvent(
        &mut opFinished as PKEVENT,
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );
    PiQueueDeviceAction(
        DeviceObject,
        Action,
        &mut opFinished as PKEVENT,
        &mut status,
    );
    KeWaitForSingleObject(
        &mut opFinished as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0,
        core::ptr::null_mut(),
    );

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     PnP manager device manipulation functions
//  * COPYRIGHT:   Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *              2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              2014-2017 Thomas Faber (thomas.faber@reactos.org)
//  *              2020 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// /* Device tree is a resource shared among all system services: hal, kernel, drivers etc.
//  * Thus all code which interacts with the tree needs to be synchronized.
//  * Here it's done via a list of DEVICE_ACTION_REQUEST structures, which represents
//  * the device action queue. It is being processed exclusively by the PipDeviceActionWorker.
//  *
//  * Operation queuing can be done with the PiQueueDeviceAction function or with
//  * the PiPerfomSyncDeviceAction for synchronous operations.
//  * All device manipulation like starting, removing, enumeration (see DEVICE_ACTION enum)
//  * have to be done with the PiQueueDeviceAction in order to avoid race conditions.
//  *
//  * Note: there is one special operation here - PiActionEnumRootDevices. It is meant to be done
//  * during initialization process (and be the first device tree operation executed) and
//  * is always executed synchronously.
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
//
// /* ARM64 verbose device action traces removed — PnP confirmed working */
// #define _DA_TRACE(s) ((void)0)
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern ERESOURCE IopDriverLoadResource;
// extern BOOLEAN PnpSystemInit;
// extern PDEVICE_NODE IopRootDeviceNode;
// extern BOOLEAN PnPBootDriversLoaded;
// extern BOOLEAN PnPBootDriversInitialized;
//
// #define MAX_DEVICE_ID_LEN          200
// #define MAX_SEPARATORS_INSTANCEID  0
// #define MAX_SEPARATORS_DEVICEID    1
//
// /* DATA **********************************************************************/
//
// LIST_ENTRY IopDeviceActionRequestList;
// WORK_QUEUE_ITEM IopDeviceActionWorkItem;
// BOOLEAN IopDeviceActionInProgress;
// KSPIN_LOCK IopDeviceActionLock;
// KEVENT PiEnumerationFinished;
// static const WCHAR ServicesKeyName[] = L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\";
//
// /* TYPES *********************************************************************/
//
// typedef struct _DEVICE_ACTION_REQUEST
// {
//     LIST_ENTRY RequestListEntry;
//     PDEVICE_OBJECT DeviceObject;
//     PKEVENT CompletionEvent;
//     NTSTATUS *CompletionStatus;
//     DEVICE_ACTION Action;
// } DEVICE_ACTION_REQUEST, *PDEVICE_ACTION_REQUEST;
//
// typedef enum _ADD_DEV_DRIVER_TYPE
// {
//     LowerFilter,
//     LowerClassFilter,
//     DeviceDriver,
//     UpperFilter,
//     UpperClassFilter
// } ADD_DEV_DRIVER_TYPE;
//
// typedef struct _ADD_DEV_DRIVERS_LIST
// {
//     LIST_ENTRY ListEntry;
//     PDRIVER_OBJECT DriverObject;
//     ADD_DEV_DRIVER_TYPE DriverType;
// } ADD_DEV_DRIVERS_LIST, *PADD_DEV_DRIVERS_LIST;
//
// typedef struct _ATTACH_FILTER_DRIVERS_CONTEXT
// {
//     ADD_DEV_DRIVER_TYPE DriverType;
//     PDEVICE_NODE DeviceNode;
//     PLIST_ENTRY DriversListHead;
// } ATTACH_FILTER_DRIVERS_CONTEXT, *PATTACH_FILTER_DRIVERS_CONTEXT;
//
// /* FUNCTIONS *****************************************************************/
//
// PDEVICE_OBJECT
// IopGetDeviceObjectFromDeviceInstance(PUNICODE_STRING DeviceInstance);
//
// NTSTATUS
// IopGetParentIdPrefix(PDEVICE_NODE DeviceNode, PUNICODE_STRING ParentIdPrefix);
//
// USHORT
// NTAPI
// IopGetBusTypeGuidIndex(LPGUID BusTypeGuid);
//
// NTSTATUS
// IopSetDeviceInstanceData(HANDLE InstanceKey, PDEVICE_NODE DeviceNode);
//
// VOID
// NTAPI
// IopInstallCriticalDevice(PDEVICE_NODE DeviceNode);
//
// static
// VOID
// IopCancelPrepareDeviceForRemoval(PDEVICE_OBJECT DeviceObject);
//
// static
// NTSTATUS
// IopPrepareDeviceForRemoval(PDEVICE_OBJECT DeviceObject, BOOLEAN Force);
//
// static
// NTSTATUS
// IopSetServiceEnumData(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ HANDLE InstanceHandle);
//
// static
// BOOLEAN
// IopValidateID(
//     _In_ PWCHAR Id,
//     _In_ BUS_QUERY_ID_TYPE QueryType)
// {
//     PWCHAR PtrChar;
//     PWCHAR StringEnd;
//     WCHAR Char;
//     ULONG SeparatorsCount = 0;
//     PWCHAR PtrPrevChar = NULL;
//     ULONG MaxSeparators;
//     BOOLEAN IsMultiSz;
//
//     PAGED_CODE();
//
//     switch (QueryType)
//     {
//         case BusQueryDeviceID:
//             MaxSeparators = MAX_SEPARATORS_DEVICEID;
//             IsMultiSz = FALSE;
//             break;
//         case BusQueryInstanceID:
//             MaxSeparators = MAX_SEPARATORS_INSTANCEID;
//             IsMultiSz = FALSE;
//             break;
//
//         case BusQueryHardwareIDs:
//         case BusQueryCompatibleIDs:
//             MaxSeparators = MAX_SEPARATORS_DEVICEID;
//             IsMultiSz = TRUE;
//             break;
//
//         default:
//             DPRINT1("IopValidateID: Not handled QueryType - %x\n", QueryType);
//             return FALSE;
//     }
//
//     StringEnd = Id + MAX_DEVICE_ID_LEN;
//
//     for (PtrChar = Id; PtrChar < StringEnd; PtrChar++)
//     {
//         Char = *PtrChar;
//
//         if (Char == UNICODE_NULL)
//         {
//             if (!IsMultiSz || (PtrPrevChar && PtrChar == PtrPrevChar + 1))
//             {
//                 if (MaxSeparators == SeparatorsCount || IsMultiSz)
//                 {
//                     return TRUE;
//                 }
//
//                 DPRINT1("IopValidateID: SeparatorsCount - %lu, MaxSeparators - %lu\n",
//                         SeparatorsCount, MaxSeparators);
//                 goto ErrorExit;
//             }
//
//             StringEnd = PtrChar + MAX_DEVICE_ID_LEN + 1;
//             PtrPrevChar = PtrChar;
//             SeparatorsCount = 0;
//         }
//         else if (Char < ' ' || Char > 0x7F || Char == ',')
//         {
//             DPRINT1("IopValidateID: Invalid character - %04X\n", Char);
//             goto ErrorExit;
//         }
//         else if (Char == ' ')
//         {
//             *PtrChar = '_';
//         }
//         else if (Char == '\\')
//         {
//             SeparatorsCount++;
//
//             if (SeparatorsCount > MaxSeparators)
//             {
//                 DPRINT1("IopValidateID: SeparatorsCount - %lu, MaxSeparators - %lu\n",
//                         SeparatorsCount, MaxSeparators);
//                 goto ErrorExit;
//             }
//         }
//     }
//
//     DPRINT1("IopValidateID: Not terminated ID\n");
//
// ErrorExit:
//     // FIXME logging
//     return FALSE;
// }
//
// static
// NTSTATUS
// IopCreateDeviceInstancePath(
//     _In_ PDEVICE_NODE DeviceNode,
//     _Out_ PUNICODE_STRING InstancePath)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING DeviceId;
//     UNICODE_STRING InstanceId;
//     IO_STACK_LOCATION Stack;
//     NTSTATUS Status;
//     UNICODE_STRING ParentIdPrefix = { 0, 0, NULL };
//     DEVICE_CAPABILITIES DeviceCapabilities;
//     BOOLEAN IsValidID;
//
//     DPRINT("Sending IRP_MN_QUERY_ID.BusQueryDeviceID to device stack\n");
//
//     Stack.Parameters.QueryId.IdType = BusQueryDeviceID;
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_ID,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopInitiatePnpIrp(BusQueryDeviceID) failed (Status %x)\n", Status);
//         return Status;
//     }
//
//     IsValidID = IopValidateID((PWCHAR)IoStatusBlock.Information, BusQueryDeviceID);
//
//     if (!IsValidID)
//     {
//         DPRINT1("Invalid DeviceID. DeviceNode - %p\n", DeviceNode);
//     }
//
//     /* Save the device id string */
//     RtlInitUnicodeString(&DeviceId, (PWSTR)IoStatusBlock.Information);
//
//     DPRINT("Sending IRP_MN_QUERY_CAPABILITIES to device stack (after enumeration)\n");
//
//     Status = IopQueryDeviceCapabilities(DeviceNode, &DeviceCapabilities);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status != STATUS_NOT_SUPPORTED)
//         {
//             DPRINT1("IopQueryDeviceCapabilities() failed (Status 0x%08lx)\n", Status);
//         }
//         RtlFreeUnicodeString(&DeviceId);
//         return Status;
//     }
//
//     /* This bit is only check after enumeration */
//     if (DeviceCapabilities.HardwareDisabled)
//     {
//         /* FIXME: Cleanup device */
//         RtlFreeUnicodeString(&DeviceId);
//         return STATUS_PLUGPLAY_NO_DEVICE;
//     }
//
//     if (!DeviceCapabilities.UniqueID)
//     {
//         /* Device has not a unique ID. We need to prepend parent bus unique identifier */
//         DPRINT("Instance ID is not unique\n");
//         Status = IopGetParentIdPrefix(DeviceNode, &ParentIdPrefix);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IopGetParentIdPrefix() failed (Status 0x%08lx)\n", Status);
//             RtlFreeUnicodeString(&DeviceId);
//             return Status;
//         }
//     }
//
//     DPRINT("Sending IRP_MN_QUERY_ID.BusQueryInstanceID to device stack\n");
//
//     Stack.Parameters.QueryId.IdType = BusQueryInstanceID;
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_ID,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopInitiatePnpIrp(BusQueryInstanceID) failed (Status %lx)\n", Status);
//         ASSERT(IoStatusBlock.Information == 0);
//     }
//
//     if (IoStatusBlock.Information)
//     {
//         IsValidID = IopValidateID((PWCHAR)IoStatusBlock.Information, BusQueryInstanceID);
//
//         if (!IsValidID)
//         {
//             DPRINT1("Invalid InstanceID. DeviceNode - %p\n", DeviceNode);
//         }
//     }
//
//     RtlInitUnicodeString(&InstanceId,
//                          (PWSTR)IoStatusBlock.Information);
//
//     InstancePath->Length = 0;
//     InstancePath->MaximumLength = DeviceId.Length + sizeof(WCHAR) +
//                                   ParentIdPrefix.Length +
//                                   InstanceId.Length +
//                                   sizeof(UNICODE_NULL);
//     if (ParentIdPrefix.Length && InstanceId.Length)
//     {
//         InstancePath->MaximumLength += sizeof(WCHAR);
//     }
//
//     InstancePath->Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                  InstancePath->MaximumLength,
//                                                  TAG_IO);
//     if (!InstancePath->Buffer)
//     {
//         RtlFreeUnicodeString(&InstanceId);
//         RtlFreeUnicodeString(&ParentIdPrefix);
//         RtlFreeUnicodeString(&DeviceId);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Start with the device id */
//     RtlCopyUnicodeString(InstancePath, &DeviceId);
//     RtlAppendUnicodeToString(InstancePath, L"\\");
//
//     /* Add information from parent bus device to InstancePath */
//     RtlAppendUnicodeStringToString(InstancePath, &ParentIdPrefix);
//     if (ParentIdPrefix.Length && InstanceId.Length)
//     {
//         RtlAppendUnicodeToString(InstancePath, L"&");
//     }
//
//     /* Finally, add the id returned by the driver stack */
//     RtlAppendUnicodeStringToString(InstancePath, &InstanceId);
//
//     /*
//      * FIXME: Check for valid characters, if there is invalid characters
//      * then bugcheck
//      */
//
//     RtlFreeUnicodeString(&InstanceId);
//     RtlFreeUnicodeString(&DeviceId);
//     RtlFreeUnicodeString(&ParentIdPrefix);
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief      Loads and/or returns the driver associated with the registry entry if the driver
//  *             is enabled. In case of an error, sets up a corresponding Problem to the DeviceNode
//  */
// static
// NTSTATUS
// NTAPI
// PiAttachFilterDriversCallback(
//     PWSTR ValueName,
//     ULONG ValueType,
//     PVOID ValueData,
//     ULONG ValueLength,
//     PVOID Ctx,
//     PVOID EntryContext)
// {
//     PATTACH_FILTER_DRIVERS_CONTEXT context = Ctx;
//     PDRIVER_OBJECT DriverObject;
//     NTSTATUS Status;
//     BOOLEAN loadDrivers = (BOOLEAN)(ULONG_PTR)EntryContext;
//
//     PAGED_CODE();
//
//     // No filter value present
//     if (ValueType != REG_SZ)
//         return STATUS_SUCCESS;
//
//     if (ValueLength <= sizeof(WCHAR))
//         return STATUS_OBJECT_NAME_NOT_FOUND;
//
//     // open the service registry key
//     UNICODE_STRING serviceName = { .Length = 0 }, servicesKeyName;
//     RtlInitUnicodeString(&serviceName, ValueData);
//     RtlInitUnicodeString(&servicesKeyName, ServicesKeyName);
//
//     HANDLE ccsServicesHandle, serviceHandle = NULL;
//
//     Status = IopOpenRegistryKeyEx(&ccsServicesHandle, NULL, &servicesKeyName, KEY_READ);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open a registry key for \"%wZ\" (status %x)\n", &serviceName, Status);
//         return Status;
//     }
//
//     Status = IopOpenRegistryKeyEx(&serviceHandle, ccsServicesHandle, &serviceName, KEY_READ);
//     ZwClose(ccsServicesHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open a registry key for \"%wZ\" (status %x)\n", &serviceName, Status);
//         return Status;
//     }
//
//     PADD_DEV_DRIVERS_LIST driverEntry = ExAllocatePoolWithTag(PagedPool,
//                                                               sizeof(*driverEntry),
//                                                               TAG_PNP_DEVACTION);
//
//     if (!driverEntry)
//     {
//         DPRINT1("Failed to allocate driverEntry for \"%wZ\"\n", &serviceName);
//         ZwClose(serviceHandle);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // check if the driver is disabled
//     PKEY_VALUE_FULL_INFORMATION kvInfo;
//     SERVICE_LOAD_TYPE startType = DisableLoad;
//
//     Status = IopGetRegistryValue(serviceHandle, L"Start", &kvInfo);
//     if (NT_SUCCESS(Status))
//     {
//         if (kvInfo->Type == REG_DWORD)
//         {
//             RtlMoveMemory(&startType,
//                           (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset),
//                           sizeof(startType));
//         }
//
//         ExFreePool(kvInfo);
//     }
//
//     // TODO: take into account other start types (like SERVICE_DEMAND_START)
//     if (startType >= DisableLoad)
//     {
//         if (!(context->DeviceNode->Flags & DNF_HAS_PROBLEM))
//         {
//             PiSetDevNodeProblem(context->DeviceNode, CM_PROB_DISABLED_SERVICE);
//         }
//
//         DPRINT("Service \"%wZ\" is disabled (start type %u)\n", &serviceName, startType);
//         Status = STATUS_UNSUCCESSFUL;
//         goto Cleanup;
//     }
//
//     // check if the driver is already loaded
//     UNICODE_STRING driverName;
//     Status = IopGetDriverNames(serviceHandle, &driverName, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Unable to obtain the driver name for \"%wZ\"\n", &serviceName);
//         goto Cleanup;
//     }
//
//     // try to open it
//     Status = ObReferenceObjectByName(&driverName,
//                                      OBJ_OPENIF | OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                      NULL, /* PassedAccessState */
//                                      0, /* DesiredAccess */
//                                      IoDriverObjectType,
//                                      KernelMode,
//                                      NULL, /* ParseContext */
//                                      (PVOID*)&DriverObject);
//     RtlFreeUnicodeString(&driverName);
//
//     // the driver was not probably loaded, try to load
//     if (!NT_SUCCESS(Status))
//     {
//         if (loadDrivers)
//         {
//             Status = IopLoadDriver(serviceHandle, &DriverObject);
//         }
//         else
//         {
//             DPRINT("Service \"%wZ\" will not be loaded now\n", &serviceName);
//             // return failure, the driver will be loaded later (in a subsequent call)
//             Status = STATUS_UNSUCCESSFUL;
//             goto Cleanup;
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         driverEntry->DriverObject = DriverObject;
//         driverEntry->DriverType = context->DriverType;
//         InsertTailList(context->DriversListHead, &driverEntry->ListEntry);
//         ZwClose(serviceHandle);
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         if (!(context->DeviceNode->Flags & DNF_HAS_PROBLEM))
//         {
//             switch (Status)
//             {
//                 case STATUS_INSUFFICIENT_RESOURCES:
//                     PiSetDevNodeProblem(context->DeviceNode, CM_PROB_OUT_OF_MEMORY);
//                     break;
//                 case STATUS_FAILED_DRIVER_ENTRY:
//                     PiSetDevNodeProblem(context->DeviceNode, CM_PROB_FAILED_DRIVER_ENTRY);
//                     break;
//                 case STATUS_ILL_FORMED_SERVICE_ENTRY:
//                     PiSetDevNodeProblem(context->DeviceNode, CM_PROB_DRIVER_SERVICE_KEY_INVALID);
//                     break;
//                 default:
//                     PiSetDevNodeProblem(context->DeviceNode, CM_PROB_DRIVER_FAILED_LOAD);
//                     break;
//             }
//         }
//
//         DPRINT1("Failed to load driver \"%wZ\" for %wZ (status %x)\n",
//             &serviceName, &context->DeviceNode->InstancePath, Status);
//     }
//
// Cleanup:
//     ExFreePoolWithTag(driverEntry, TAG_PNP_DEVACTION);
//     if (serviceHandle)
//     {
//         ZwClose(serviceHandle);
//     }
//     return Status;
// }
//
//
// /**
//  * @brief      Calls PiAttachFilterDriversCallback for filter drivers (if any)
//  */
// static
// NTSTATUS
// PiAttachFilterDrivers(
//     PLIST_ENTRY DriversListHead,
//     PDEVICE_NODE DeviceNode,
//     HANDLE EnumSubKey,
//     HANDLE ClassKey,
//     BOOLEAN Lower,
//     BOOLEAN LoadDrivers)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[2] = { { NULL, 0, NULL, NULL, 0, NULL, 0 }, };
//     ATTACH_FILTER_DRIVERS_CONTEXT routineContext;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     routineContext.DriversListHead = DriversListHead;
//     routineContext.DeviceNode = DeviceNode;
//
//     // First add device filters
//     routineContext.DriverType = Lower ? LowerFilter : UpperFilter;
//     QueryTable[0] = (RTL_QUERY_REGISTRY_TABLE){
//         .QueryRoutine = PiAttachFilterDriversCallback,
//         .Name = Lower ? L"LowerFilters" : L"UpperFilters",
//         .DefaultType = REG_NONE,
//         .EntryContext = (PVOID)(ULONG_PTR)LoadDrivers
//     };
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                     (PWSTR)EnumSubKey,
//                                     QueryTable,
//                                     &routineContext,
//                                     NULL);
//     if (ClassKey == NULL)
//     {
//         return Status;
//     }
//
//     // Then add device class filters
//     routineContext.DriverType = Lower ? LowerClassFilter : UpperClassFilter;
//     QueryTable[0] = (RTL_QUERY_REGISTRY_TABLE){
//         .QueryRoutine = PiAttachFilterDriversCallback,
//         .Name = Lower ? L"LowerFilters" : L"UpperFilters",
//         .DefaultType = REG_NONE,
//         .EntryContext = (PVOID)(ULONG_PTR)LoadDrivers
//     };
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                     (PWSTR)ClassKey,
//                                     QueryTable,
//                                     &routineContext,
//                                     NULL);
//     return Status;
// }
//
// /**
//  * @brief      Loads all drivers for a device node (actual service and filters)
//  *             and calls their AddDevice routine
//  *
//  * @param[in]  DeviceNode   The device node
//  * @param[in]  LoadDrivers  Whether to load drivers if they are not loaded yet
//  *                          (used when storage subsystem is not yet initialized)
//  */
// static
// NTSTATUS
// PiCallDriverAddDevice(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ BOOLEAN LoadDrivers)
// {
//     NTSTATUS Status;
//     HANDLE EnumRootKey, SubKey;
//     HANDLE ClassKey = NULL;
//     UNICODE_STRING EnumRoot = RTL_CONSTANT_STRING(ENUM_ROOT);
//     static UNICODE_STRING ccsControlClass =
//     RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Class");
//     PKEY_VALUE_FULL_INFORMATION kvInfo = NULL;
//
//     PAGED_CODE();
//
//     // open the enumeration root key
//     Status = IopOpenRegistryKeyEx(&EnumRootKey, NULL, &EnumRoot, KEY_READ);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopOpenRegistryKeyEx() failed for \"%wZ\" (status %x)\n", &EnumRoot, Status);
//         return Status;
//     }
//
//     // open an instance subkey
//     Status = IopOpenRegistryKeyEx(&SubKey, EnumRootKey, &DeviceNode->InstancePath, KEY_READ);
//     ZwClose(EnumRootKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open a devnode instance key for \"%wZ\" (status %x)\n",
//                 &DeviceNode->InstancePath, Status);
//         return Status;
//     }
//
//     // try to get the class GUID of an instance and its registry key
//     Status = IopGetRegistryValue(SubKey, REGSTR_VAL_CLASSGUID, &kvInfo);
//     if (NT_SUCCESS(Status))
//     {
//         if ((kvInfo->Type == REG_SZ) &&
//             (kvInfo->DataLength > sizeof(UNICODE_NULL)) &&
//             (kvInfo->DataLength <= UNICODE_STRING_MAX_BYTES) &&
//             ((kvInfo->DataLength % sizeof(WCHAR)) == 0))
//         {
//             UNICODE_STRING classGUID = {
//                 .MaximumLength = kvInfo->DataLength,
//                 .Length = (USHORT)(kvInfo->DataLength - sizeof(UNICODE_NULL)),
//                 .Buffer = (PVOID)((ULONG_PTR)kvInfo + kvInfo->DataOffset)
//             };
//             HANDLE ccsControlHandle;
//
//             Status = IopOpenRegistryKeyEx(&ccsControlHandle, NULL, &ccsControlClass, KEY_READ);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("IopOpenRegistryKeyEx() failed for \"%wZ\" (status %x)\n",
//                         &ccsControlClass, Status);
//             }
//             else
//             {
//                 // open the CCS\Control\Class\<ClassGUID> key
//                 Status = IopOpenRegistryKeyEx(&ClassKey, ccsControlHandle, &classGUID, KEY_READ);
//                 ZwClose(ccsControlHandle);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("Failed to open class key \"%wZ\" (status %x)\n", &classGUID, Status);
//                 }
//             }
//
//             if (ClassKey)
//             {
//                 // Check the Properties key of a class too
//                 // Windows fills some device properties from this key (which is protected)
//                 // TODO: add the device properties from this key
//
//                 UNICODE_STRING properties = RTL_CONSTANT_STRING(REGSTR_KEY_DEVICE_PROPERTIES);
//                 HANDLE propertiesHandle;
//
//                 Status = IopOpenRegistryKeyEx(&propertiesHandle, ClassKey, &properties, KEY_READ);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT("Properties key failed to open for \"%wZ\" (status %x)\n",
//                            &classGUID, Status);
//                 }
//                 else
//                 {
//                     ZwClose(propertiesHandle);
//                 }
//             }
//         }
//
//         ExFreePool(kvInfo);
//     }
//
//     // the driver loading order:
//     // 1. LowerFilters
//     // 2. LowerClassFilters
//     // 3. Device driver (only one service!)
//     // 4. UpperFilters
//     // 5. UpperClassFilters
//
//     LIST_ENTRY drvListHead;
//     InitializeListHead(&drvListHead);
//
//     // lower (class) filters
//     Status = PiAttachFilterDrivers(&drvListHead, DeviceNode, SubKey, ClassKey, TRUE, LoadDrivers);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     ATTACH_FILTER_DRIVERS_CONTEXT routineContext = {
//         .DriversListHead = &drvListHead,
//         .DriverType = DeviceDriver,
//         .DeviceNode = DeviceNode
//     };
//
//     RTL_QUERY_REGISTRY_TABLE queryTable[2] = {{
//         .QueryRoutine = PiAttachFilterDriversCallback,
//         .Name = L"Service",
//         .Flags = RTL_QUERY_REGISTRY_REQUIRED,
//         .DefaultType = REG_SZ, // REG_MULTI_SZ is not allowed here
//         .DefaultData = L"",
//         .EntryContext = (PVOID)(ULONG_PTR)LoadDrivers
//     },};
//
//     // device driver
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                     (PWSTR)SubKey,
//                                     queryTable,
//                                     &routineContext,
//                                     NULL);
//     if (NT_SUCCESS(Status))
//     {
//         // do nothing
//     }
//     // if a driver is not found, but a device allows raw access -> proceed
//     else if (Status == STATUS_OBJECT_NAME_NOT_FOUND &&
//              (DeviceNode->CapabilityFlags & 0x00000040)) // CM_DEVCAP_RAWDEVICEOK
//     {
//         // add a dummy entry to the drivers list (need for later processing)
//         PADD_DEV_DRIVERS_LIST driverEntry = ExAllocatePoolZero(PagedPool,
//                                                                sizeof(*driverEntry),
//                                                                TAG_PNP_DEVACTION);
//         driverEntry->DriverType = DeviceDriver;
//         InsertTailList(&drvListHead, &driverEntry->ListEntry);
//         DPRINT("No service for \"%wZ\" (RawDeviceOK)\n", &DeviceNode->InstancePath);
//     }
//     else
//     {
//         if (Status == STATUS_OBJECT_TYPE_MISMATCH && !(DeviceNode->Flags & DNF_HAS_PROBLEM))
//         {
//             PiSetDevNodeProblem(DeviceNode, CM_PROB_REGISTRY);
//         }
//         DPRINT("No service for \"%wZ\" (loadDrv: %u)\n", &DeviceNode->InstancePath, LoadDrivers);
//         goto Cleanup;
//     }
//
//     // upper (class) filters
//     Status = PiAttachFilterDrivers(&drvListHead, DeviceNode, SubKey, ClassKey, FALSE, LoadDrivers);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     // finally loop through the stack and call AddDevice for every driver
//     for (PLIST_ENTRY listEntry = drvListHead.Flink;
//          listEntry != &drvListHead;
//          listEntry = listEntry->Flink)
//     {
//         PADD_DEV_DRIVERS_LIST driverEntry;
//         driverEntry = CONTAINING_RECORD(listEntry, ADD_DEV_DRIVERS_LIST, ListEntry);
//         PDRIVER_OBJECT driverObject = driverEntry->DriverObject;
//
//         // FIXME: ReactOS is not quite ready for this assert
//         // (legacy drivers should not have AddDevice routine)
//         // ASSERT(!(DriverObject->Flags & DRVO_LEGACY_DRIVER));
//
//         if (driverObject && driverObject->DriverExtension->AddDevice)
//         {
//             Status = driverObject->DriverExtension->AddDevice(driverEntry->DriverObject,
//                                                               DeviceNode->PhysicalDeviceObject);
//         }
//         else if (driverObject == NULL)
//         {
//             // valid only for DeviceDriver
//             ASSERT(driverEntry->DriverType == DeviceDriver);
//             ASSERT(DeviceNode->CapabilityFlags & 0x00000040); // CM_DEVCAP_RAWDEVICEOK
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             // HACK: the driver doesn't have a AddDevice routine. We shouldn't be here,
//             // but ReactOS' PnP stack is not that correct yet
//             DeviceNode->Flags |= DNF_LEGACY_DRIVER;
//             Status = STATUS_UNSUCCESSFUL;
//         }
//
//         // for filter drivers we don't care about the AddDevice result
//         if (driverEntry->DriverType == DeviceDriver)
//         {
//             if (NT_SUCCESS(Status))
//             {
//                 PDEVICE_OBJECT fdo = IoGetAttachedDeviceReference(DeviceNode->PhysicalDeviceObject);
//
//                 // HACK: Check if we have a ACPI device (needed for power management)
//                 if (fdo->DeviceType == FILE_DEVICE_ACPI)
//                 {
//                     static BOOLEAN SystemPowerDeviceNodeCreated = FALSE;
//
//                     // There can be only one system power device
//                     if (!SystemPowerDeviceNodeCreated)
//                     {
//                         PopSystemPowerDeviceNode = DeviceNode;
//                         ObReferenceObject(PopSystemPowerDeviceNode->PhysicalDeviceObject);
//                         SystemPowerDeviceNodeCreated = TRUE;
//                     }
//                 }
//
//                 ObDereferenceObject(fdo);
//                 PiSetDevNodeState(DeviceNode, DeviceNodeDriversAdded);
//             }
//             else
//             {
//                 // lower filters (if already started) will be removed upon this request
//                 PiSetDevNodeProblem(DeviceNode, CM_PROB_FAILED_ADD);
//                 PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
//                 break;
//             }
//         }
//
// #if DBG
//         PDEVICE_OBJECT attachedDO = IoGetAttachedDevice(DeviceNode->PhysicalDeviceObject);
//         if (attachedDO->Flags & DO_DEVICE_INITIALIZING)
//         {
//             DPRINT1("DO_DEVICE_INITIALIZING is not cleared on a device 0x%p!\n", attachedDO);
//         }
// #endif
//     }
//
// Cleanup:
//     while (!IsListEmpty(&drvListHead))
//     {
//         PLIST_ENTRY listEntry = RemoveHeadList(&drvListHead);
//         PADD_DEV_DRIVERS_LIST driverEntry;
//         driverEntry = CONTAINING_RECORD(listEntry, ADD_DEV_DRIVERS_LIST, ListEntry);
//
//         // drivers which don't have any devices (in case of failure) will be cleaned up
//         if (driverEntry->DriverObject)
//         {
//             ObDereferenceObject(driverEntry->DriverObject);
//         }
//         ExFreePoolWithTag(driverEntry, TAG_PNP_DEVACTION);
//     }
//
//     ZwClose(SubKey);
//     if (ClassKey != NULL)
//     {
//         ZwClose(ClassKey);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IopQueryDeviceCapabilities(PDEVICE_NODE DeviceNode,
//                            PDEVICE_CAPABILITIES DeviceCaps)
// {
//     IO_STATUS_BLOCK StatusBlock;
//     IO_STACK_LOCATION Stack;
//     NTSTATUS Status;
//     HANDLE InstanceKey;
//     UNICODE_STRING ValueName;
//
//     /* Set up the Header */
//     RtlZeroMemory(DeviceCaps, sizeof(DEVICE_CAPABILITIES));
//     DeviceCaps->Size = sizeof(DEVICE_CAPABILITIES);
//     DeviceCaps->Version = 1;
//     DeviceCaps->Address = -1;
//     DeviceCaps->UINumber = -1;
//
//     /* Set up the Stack */
//     RtlZeroMemory(&Stack, sizeof(IO_STACK_LOCATION));
//     Stack.Parameters.DeviceCapabilities.Capabilities = DeviceCaps;
//
//     /* Send the IRP */
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &StatusBlock,
//                                IRP_MN_QUERY_CAPABILITIES,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status != STATUS_NOT_SUPPORTED)
//         {
//             DPRINT1("IRP_MN_QUERY_CAPABILITIES failed with status 0x%lx\n", Status);
//         }
//         return Status;
//     }
//
//     /* Map device capabilities to capability flags */
//     DeviceNode->CapabilityFlags = 0;
//     if (DeviceCaps->LockSupported)
//         DeviceNode->CapabilityFlags |= 0x00000001;    // CM_DEVCAP_LOCKSUPPORTED
//
//     if (DeviceCaps->EjectSupported)
//         DeviceNode->CapabilityFlags |= 0x00000002;    // CM_DEVCAP_EJECTSUPPORTED
//
//     if (DeviceCaps->Removable)
//         DeviceNode->CapabilityFlags |= 0x00000004;    // CM_DEVCAP_REMOVABLE
//
//     if (DeviceCaps->DockDevice)
//         DeviceNode->CapabilityFlags |= 0x00000008;    // CM_DEVCAP_DOCKDEVICE
//
//     if (DeviceCaps->UniqueID)
//         DeviceNode->CapabilityFlags |= 0x00000010;    // CM_DEVCAP_UNIQUEID
//
//     if (DeviceCaps->SilentInstall)
//         DeviceNode->CapabilityFlags |= 0x00000020;    // CM_DEVCAP_SILENTINSTALL
//
//     if (DeviceCaps->RawDeviceOK)
//         DeviceNode->CapabilityFlags |= 0x00000040;    // CM_DEVCAP_RAWDEVICEOK
//
//     if (DeviceCaps->SurpriseRemovalOK)
//         DeviceNode->CapabilityFlags |= 0x00000080;    // CM_DEVCAP_SURPRISEREMOVALOK
//
//     if (DeviceCaps->HardwareDisabled)
//         DeviceNode->CapabilityFlags |= 0x00000100;    // CM_DEVCAP_HARDWAREDISABLED
//
//     if (DeviceCaps->NonDynamic)
//         DeviceNode->CapabilityFlags |= 0x00000200;    // CM_DEVCAP_NONDYNAMIC
//
//     if (DeviceCaps->NoDisplayInUI)
//         DeviceNode->UserFlags |= DNUF_DONT_SHOW_IN_UI;
//     else
//         DeviceNode->UserFlags &= ~DNUF_DONT_SHOW_IN_UI;
//
//     Status = IopCreateDeviceKeyPath(&DeviceNode->InstancePath, REG_OPTION_NON_VOLATILE, &InstanceKey);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set 'Capabilities' value */
//         RtlInitUnicodeString(&ValueName, L"Capabilities");
//         Status = ZwSetValueKey(InstanceKey,
//                                &ValueName,
//                                0,
//                                REG_DWORD,
//                                &DeviceNode->CapabilityFlags,
//                                sizeof(ULONG));
//
//         /* Set 'UINumber' value */
//         if (DeviceCaps->UINumber != MAXULONG)
//         {
//             RtlInitUnicodeString(&ValueName, L"UINumber");
//             Status = ZwSetValueKey(InstanceKey,
//                                    &ValueName,
//                                    0,
//                                    REG_DWORD,
//                                    &DeviceCaps->UINumber,
//                                    sizeof(ULONG));
//         }
//
//         ZwClose(InstanceKey);
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopQueryHardwareIds(PDEVICE_NODE DeviceNode,
//                     HANDLE InstanceKey)
// {
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PWSTR Ptr;
//     UNICODE_STRING ValueName;
//     NTSTATUS Status;
//     ULONG Length, TotalLength;
//     BOOLEAN IsValidID;
//
//     DPRINT("Sending IRP_MN_QUERY_ID.BusQueryHardwareIDs to device stack\n");
//
//     RtlZeroMemory(&Stack, sizeof(Stack));
//     Stack.Parameters.QueryId.IdType = BusQueryHardwareIDs;
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_ID,
//                                &Stack);
//     if (NT_SUCCESS(Status))
//     {
//         IsValidID = IopValidateID((PWCHAR)IoStatusBlock.Information, BusQueryHardwareIDs);
//
//         if (!IsValidID)
//         {
//             DPRINT1("Invalid HardwareIDs. DeviceNode - %p\n", DeviceNode);
//         }
//
//         TotalLength = 0;
//
//         Ptr = (PWSTR)IoStatusBlock.Information;
//         DPRINT("Hardware IDs:\n");
//         while (*Ptr)
//         {
//             DPRINT("  %S\n", Ptr);
//             Length = (ULONG)wcslen(Ptr) + 1;
//
//             Ptr += Length;
//             TotalLength += Length;
//         }
//         DPRINT("TotalLength: %hu\n", TotalLength);
//         DPRINT("\n");
//
//         RtlInitUnicodeString(&ValueName, L"HardwareID");
//         Status = ZwSetValueKey(InstanceKey,
//                                &ValueName,
//                                0,
//                                REG_MULTI_SZ,
//                                (PVOID)IoStatusBlock.Information,
//                                (TotalLength + 1) * sizeof(WCHAR));
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwSetValueKey() failed (Status %lx)\n", Status);
//         }
//     }
//     else
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status %x)\n", Status);
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopQueryCompatibleIds(PDEVICE_NODE DeviceNode,
//                       HANDLE InstanceKey)
// {
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PWSTR Ptr;
//     UNICODE_STRING ValueName;
//     NTSTATUS Status;
//     ULONG Length, TotalLength;
//     BOOLEAN IsValidID;
//
//     DPRINT("Sending IRP_MN_QUERY_ID.BusQueryCompatibleIDs to device stack\n");
//
//     RtlZeroMemory(&Stack, sizeof(Stack));
//     Stack.Parameters.QueryId.IdType = BusQueryCompatibleIDs;
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_ID,
//                                &Stack);
//     if (NT_SUCCESS(Status) && IoStatusBlock.Information)
//     {
//         IsValidID = IopValidateID((PWCHAR)IoStatusBlock.Information, BusQueryCompatibleIDs);
//
//         if (!IsValidID)
//         {
//             DPRINT1("Invalid CompatibleIDs. DeviceNode - %p\n", DeviceNode);
//         }
//
//         TotalLength = 0;
//
//         Ptr = (PWSTR)IoStatusBlock.Information;
//         DPRINT("Compatible IDs:\n");
//         while (*Ptr)
//         {
//             DPRINT("  %S\n", Ptr);
//             Length = (ULONG)wcslen(Ptr) + 1;
//
//             Ptr += Length;
//             TotalLength += Length;
//         }
//         DPRINT("TotalLength: %hu\n", TotalLength);
//         DPRINT("\n");
//
//         RtlInitUnicodeString(&ValueName, L"CompatibleIDs");
//         Status = ZwSetValueKey(InstanceKey,
//                                &ValueName,
//                                0,
//                                REG_MULTI_SZ,
//                                (PVOID)IoStatusBlock.Information,
//                                (TotalLength + 1) * sizeof(WCHAR));
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwSetValueKey() failed (Status %lx) or no Compatible ID returned\n", Status);
//         }
//     }
//     else
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status %x)\n", Status);
//     }
//
//     return Status;
// }
//
// /**
//  * @brief      Sets the DeviceNode's DeviceDesc and LocationInformation registry values
//  */
// VOID
// PiSetDevNodeText(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ HANDLE InstanceKey)
// {
//     PAGED_CODE();
//
//     LCID localeId;
//
//     // Get the Locale ID
//     NTSTATUS status = ZwQueryDefaultLocale(FALSE, &localeId);
//     if (!NT_SUCCESS(status))
//     {
//         DPRINT1("ZwQueryDefaultLocale() failed with status %x\n", status);
//         return;
//     }
//
//     // Step 1: Write the DeviceDesc value if does not exist
//
//     UNICODE_STRING valDeviceDesc = RTL_CONSTANT_STRING(L"DeviceDesc");
//     ULONG len;
//
//     status = ZwQueryValueKey(InstanceKey, &valDeviceDesc, KeyValueBasicInformation, NULL, 0, &len);
//     if (status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         PWSTR deviceDesc = NULL;
//         status = PiIrpQueryDeviceText(DeviceNode, localeId, DeviceTextDescription, &deviceDesc);
//
//         if (deviceDesc && deviceDesc[0] != UNICODE_NULL)
//         {
//             status = ZwSetValueKey(InstanceKey,
//                                    &valDeviceDesc,
//                                    0,
//                                    REG_SZ,
//                                    deviceDesc,
//                                    ((ULONG)wcslen(deviceDesc) + 1) * sizeof(WCHAR));
//
//             if (!NT_SUCCESS(status))
//             {
//                 DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
//             }
//         }
//         else
//         {
//             // This key is mandatory, so even if the Irp fails, we still write it
//             UNICODE_STRING unknownDeviceDesc = RTL_CONSTANT_STRING(L"Unknown device");
//             DPRINT("Driver didn't return DeviceDesc (status %x)\n", status);
//
//             status = ZwSetValueKey(InstanceKey,
//                                    &valDeviceDesc,
//                                    0,
//                                    REG_SZ,
//                                    unknownDeviceDesc.Buffer,
//                                    unknownDeviceDesc.MaximumLength);
//             if (!NT_SUCCESS(status))
//             {
//                 DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
//             }
//         }
//
//         if (deviceDesc)
//         {
//             ExFreePoolWithTag(deviceDesc, 0);
//         }
//     }
//
//     // Step 2: LocaltionInformation is overwritten unconditionally
//
//     PWSTR deviceLocationInfo = NULL;
//     status = PiIrpQueryDeviceText(DeviceNode,
//                                   localeId,
//                                   DeviceTextLocationInformation,
//                                   &deviceLocationInfo);
//
//     if (deviceLocationInfo && deviceLocationInfo[0] != UNICODE_NULL)
//     {
//         UNICODE_STRING valLocationInfo = RTL_CONSTANT_STRING(L"LocationInformation");
//
//         status = ZwSetValueKey(InstanceKey,
//                                &valLocationInfo,
//                                0,
//                                REG_SZ,
//                                deviceLocationInfo,
//                                ((ULONG)wcslen(deviceLocationInfo) + 1) * sizeof(WCHAR));
//         if (!NT_SUCCESS(status))
//         {
//             DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
//         }
//     }
//
//     if (deviceLocationInfo)
//     {
//         ExFreePoolWithTag(deviceLocationInfo, 0);
//     }
//     else
//     {
//         DPRINT("Driver didn't return LocationInformation (status %x)\n", status);
//     }
// }
//
// static
// NTSTATUS
// PiInitializeDevNode(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//     HANDLE InstanceKey = NULL;
//     UNICODE_STRING InstancePathU;
//     PDEVICE_OBJECT OldDeviceObject;
//
//     DPRINT("PiProcessNewDevNode(%p)\n", DeviceNode);
//     DPRINT("PDO 0x%p\n", DeviceNode->PhysicalDeviceObject);
//
//     /*
//      * FIXME: For critical errors, cleanup and disable device, but always
//      * return STATUS_SUCCESS.
//      */
//
//     Status = IopCreateDeviceInstancePath(DeviceNode, &InstancePathU);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status != STATUS_PLUGPLAY_NO_DEVICE)
//         {
//             DPRINT1("IopCreateDeviceInstancePath() failed with status 0x%lx\n", Status);
//         }
//         return Status;
//     }
//
//     /* Verify that this is not a duplicate */
//     OldDeviceObject = IopGetDeviceObjectFromDeviceInstance(&InstancePathU);
//     if (OldDeviceObject != NULL)
//     {
//         PDEVICE_NODE OldDeviceNode = IopGetDeviceNode(OldDeviceObject);
//
//         DPRINT1("Duplicate device instance '%wZ'\n", &InstancePathU);
//         DPRINT1("Current instance parent: '%wZ'\n", &DeviceNode->Parent->InstancePath);
//         DPRINT1("Old instance parent: '%wZ'\n", &OldDeviceNode->Parent->InstancePath);
//
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR,
//                      0x01,
//                      (ULONG_PTR)DeviceNode->PhysicalDeviceObject,
//                      (ULONG_PTR)OldDeviceObject,
//                      0);
//     }
//
//     DeviceNode->InstancePath = InstancePathU;
//
//     DPRINT("InstancePath is %S\n", DeviceNode->InstancePath.Buffer);
//
//     /*
//      * Create registry key for the instance id, if it doesn't exist yet
//      */
//     Status = IopCreateDeviceKeyPath(&DeviceNode->InstancePath, REG_OPTION_NON_VOLATILE, &InstanceKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create the instance key! (Status %lx)\n", Status);
//
//         /* We have to return success otherwise we abort the traverse operation */
//         return STATUS_SUCCESS;
//     }
//
//     IopQueryHardwareIds(DeviceNode, InstanceKey);
//
//     IopQueryCompatibleIds(DeviceNode, InstanceKey);
//
//     DeviceNode->Flags |= DNF_IDS_QUERIED;
//
//     // Set the device's DeviceDesc and LocationInformation fields
//     PiSetDevNodeText(DeviceNode, InstanceKey);
//
//     DPRINT("Sending IRP_MN_QUERY_BUS_INFORMATION to device stack\n");
//
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_BUS_INFORMATION,
//                                NULL);
//     if (NT_SUCCESS(Status) && IoStatusBlock.Information)
//     {
//         PPNP_BUS_INFORMATION BusInformation = (PPNP_BUS_INFORMATION)IoStatusBlock.Information;
//
//         DeviceNode->ChildBusNumber = BusInformation->BusNumber;
//         DeviceNode->ChildInterfaceType = BusInformation->LegacyBusType;
//         DeviceNode->ChildBusTypeIndex = IopGetBusTypeGuidIndex(&BusInformation->BusTypeGuid);
//         ExFreePoolWithTag(BusInformation, 0);
//     }
//     else
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status %x) or IoStatusBlock.Information=NULL\n", Status);
//
//         DeviceNode->ChildBusNumber = 0xFFFFFFF0;
//         DeviceNode->ChildInterfaceType = InterfaceTypeUndefined;
//         DeviceNode->ChildBusTypeIndex = -1;
//     }
//
//     DPRINT("Sending IRP_MN_QUERY_RESOURCES to device stack\n");
//
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_RESOURCES,
//                                NULL);
//     if (NT_SUCCESS(Status) && IoStatusBlock.Information)
//     {
//         DeviceNode->BootResources = (PCM_RESOURCE_LIST)IoStatusBlock.Information;
//         IopDeviceNodeSetFlag(DeviceNode, DNF_HAS_BOOT_CONFIG);
//     }
//     else
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status %x) or IoStatusBlock.Information=NULL\n", Status);
//         DeviceNode->BootResources = NULL;
//     }
//
//     DPRINT("Sending IRP_MN_QUERY_RESOURCE_REQUIREMENTS to device stack\n");
//
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_RESOURCE_REQUIREMENTS,
//                                NULL);
//     if (NT_SUCCESS(Status))
//     {
//         DeviceNode->ResourceRequirements = (PIO_RESOURCE_REQUIREMENTS_LIST)IoStatusBlock.Information;
//     }
//     else
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status %08lx)\n", Status);
//         DeviceNode->ResourceRequirements = NULL;
//     }
//
//     if (InstanceKey != NULL)
//     {
//         IopSetDeviceInstanceData(InstanceKey, DeviceNode);
//     }
//
//     // Try installing a critical device, so its Service key is populated
//     // then call IopSetServiceEnumData to populate service's Enum key.
//     // That allows us to start devices during an early boot
//     IopInstallCriticalDevice(DeviceNode);
//     IopSetServiceEnumData(DeviceNode, InstanceKey);
//
//     ZwClose(InstanceKey);
//
//     PiSetDevNodeState(DeviceNode, DeviceNodeInitialized);
//
//     if (!IopDeviceNodeHasFlag(DeviceNode, DNF_LEGACY_DRIVER))
//     {
//         /* Report the device to the user-mode pnp manager */
//         IopQueueDeviceInstallEvent(&GUID_DEVICE_ENUMERATED,
//                                    &DeviceNode->InstancePath);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// IopSetServiceEnumData(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ HANDLE InstanceHandle)
// {
//     UNICODE_STRING ServicesKeyPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
//     UNICODE_STRING ServiceKeyName;
//     UNICODE_STRING EnumKeyName;
//     UNICODE_STRING ValueName;
//     UNICODE_STRING ServiceName;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation, kvInfo2;
//     HANDLE ServiceKey = NULL, ServiceEnumKey = NULL;
//     ULONG Disposition;
//     ULONG Count = 0, NextInstance = 0;
//     WCHAR ValueBuffer[6];
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     // obtain the device node's ServiceName
//     Status = IopGetRegistryValue(InstanceHandle, L"Service", &kvInfo2);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if ((kvInfo2->Type != REG_SZ) ||
//         (kvInfo2->DataLength <= sizeof(UNICODE_NULL)) ||
//         (kvInfo2->DataLength > UNICODE_STRING_MAX_BYTES) ||
//         ((kvInfo2->DataLength % sizeof(WCHAR)) != 0))
//     {
//         DPRINT1("ObjectName invalid (Type = %lu, DataLength = %lu)\n",
//                 kvInfo2->Type,
//                 kvInfo2->DataLength);
//         ExFreePool(kvInfo2);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     ServiceName.MaximumLength = kvInfo2->DataLength;
//     ServiceName.Length = (USHORT)(kvInfo2->DataLength - sizeof(UNICODE_NULL));
//     ServiceName.Buffer = (PVOID)((ULONG_PTR)kvInfo2 + kvInfo2->DataOffset);
//
//     DPRINT("IopSetServiceEnumData(%p)\n", DeviceNode);
//     DPRINT("Instance: %wZ\n", &DeviceNode->InstancePath);
//     DPRINT("Service: %wZ\n", &ServiceName);
//
//     ServiceKeyName.MaximumLength = ServicesKeyPath.Length + ServiceName.Length + sizeof(UNICODE_NULL);
//     ServiceKeyName.Length = 0;
//     ServiceKeyName.Buffer = ExAllocatePool(PagedPool, ServiceKeyName.MaximumLength);
//     if (ServiceKeyName.Buffer == NULL)
//     {
//         DPRINT1("No ServiceKeyName.Buffer!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlAppendUnicodeStringToString(&ServiceKeyName, &ServicesKeyPath);
//     RtlAppendUnicodeStringToString(&ServiceKeyName, &ServiceName);
//
//     DPRINT("ServiceKeyName: %wZ\n", &ServiceKeyName);
//
//     Status = IopOpenRegistryKeyEx(&ServiceKey, NULL, &ServiceKeyName, KEY_CREATE_SUB_KEY);
//     if (!NT_SUCCESS(Status))
//     {
//         goto done;
//     }
//
//     Status = RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING,
//                                        &ServiceName,
//                                        &DeviceNode->ServiceName);
//     if (!NT_SUCCESS(Status))
//     {
//         goto done;
//     }
//
//     RtlInitUnicodeString(&EnumKeyName, L"Enum");
//     Status = IopCreateRegistryKeyEx(&ServiceEnumKey,
//                                     ServiceKey,
//                                     &EnumKeyName,
//                                     KEY_SET_VALUE,
//                                     REG_OPTION_VOLATILE,
//                                     &Disposition);
//     if (NT_SUCCESS(Status))
//     {
//         if (Disposition == REG_OPENED_EXISTING_KEY)
//         {
//             /* Read the NextInstance value */
//             Status = IopGetRegistryValue(ServiceEnumKey,
//                                          L"Count",
//                                          &KeyValueInformation);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             if ((KeyValueInformation->Type == REG_DWORD) &&
//                 (KeyValueInformation->DataLength))
//             {
//                 /* Read it */
//                 Count = *(PULONG)((ULONG_PTR)KeyValueInformation +
//                                   KeyValueInformation->DataOffset);
//             }
//
//             ExFreePool(KeyValueInformation);
//             KeyValueInformation = NULL;
//
//             /* Read the NextInstance value */
//             Status = IopGetRegistryValue(ServiceEnumKey,
//                                          L"NextInstance",
//                                          &KeyValueInformation);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             if ((KeyValueInformation->Type == REG_DWORD) &&
//                 (KeyValueInformation->DataLength))
//             {
//                 NextInstance = *(PULONG)((ULONG_PTR)KeyValueInformation +
//                                          KeyValueInformation->DataOffset);
//             }
//
//             ExFreePool(KeyValueInformation);
//             KeyValueInformation = NULL;
//         }
//
//         /* Set the instance path */
//         swprintf(ValueBuffer, L"%lu", NextInstance);
//         RtlInitUnicodeString(&ValueName, ValueBuffer);
//         Status = ZwSetValueKey(ServiceEnumKey,
//                                &ValueName,
//                                0,
//                                REG_SZ,
//                                DeviceNode->InstancePath.Buffer,
//                                DeviceNode->InstancePath.MaximumLength);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Increment Count and NextInstance */
//         Count++;
//         NextInstance++;
//
//         /* Set the new Count value */
//         RtlInitUnicodeString(&ValueName, L"Count");
//         Status = ZwSetValueKey(ServiceEnumKey,
//                                &ValueName,
//                                0,
//                                REG_DWORD,
//                                &Count,
//                                sizeof(Count));
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* Set the new NextInstance value */
//         RtlInitUnicodeString(&ValueName, L"NextInstance");
//         Status = ZwSetValueKey(ServiceEnumKey,
//                                &ValueName,
//                                0,
//                                REG_DWORD,
//                                &NextInstance,
//                                sizeof(NextInstance));
//     }
//
// done:
//     if (ServiceEnumKey != NULL)
//         ZwClose(ServiceEnumKey);
//
//     if (ServiceKey != NULL)
//         ZwClose(ServiceKey);
//
//     ExFreePool(ServiceKeyName.Buffer);
//     ExFreePool(kvInfo2);
//
//     return Status;
// }
//
// /**
//  * @brief      Processes the IoInvalidateDeviceState request
//  *
//  * Sends IRP_MN_QUERY_PNP_DEVICE_STATE request and sets device node's flags
//  * according to the result.
//  * Tree reenumeration should be started upon a successful return of the function.
//  *
//  * @todo       Do not return STATUS_SUCCESS if nothing is changed.
//  */
// static
// NTSTATUS
// PiUpdateDeviceState(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PNP_DEVICE_STATE PnPFlags;
//     NTSTATUS Status;
//
//     Status = PiIrpQueryPnPDeviceState(DeviceNode, &PnPFlags);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (PnPFlags & PNP_DEVICE_NOT_DISABLEABLE)
//         DeviceNode->UserFlags |= DNUF_NOT_DISABLEABLE;
//     else
//         DeviceNode->UserFlags &= ~DNUF_NOT_DISABLEABLE;
//
//     if (PnPFlags & PNP_DEVICE_DONT_DISPLAY_IN_UI)
//         DeviceNode->UserFlags |= DNUF_DONT_SHOW_IN_UI;
//     else
//         DeviceNode->UserFlags &= ~DNUF_DONT_SHOW_IN_UI;
//
//     if (PnPFlags & PNP_DEVICE_REMOVED || PnPFlags & PNP_DEVICE_DISABLED)
//     {
//         PiSetDevNodeProblem(DeviceNode,
//                             PnPFlags & PNP_DEVICE_DISABLED
//                             ? CM_PROB_HARDWARE_DISABLED
//                             : CM_PROB_DEVICE_NOT_THERE);
//
//         PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
//     }
//     else if (PnPFlags & PNP_DEVICE_RESOURCE_REQUIREMENTS_CHANGED)
//     {
//         // Query resource rebalance
//
//         if (PnPFlags & PNP_DEVICE_FAILED)
//             DeviceNode->Flags &= DNF_NON_STOPPED_REBALANCE;
//         else
//             DeviceNode->Flags |= DNF_NON_STOPPED_REBALANCE;
//
//         // Clear DNF_NO_RESOURCE_REQUIRED just in case (will be set back if needed)
//         DeviceNode->Flags &= ~DNF_NO_RESOURCE_REQUIRED;
//
//         // This will be caught up later by enumeration
//         DeviceNode->Flags |= DNF_RESOURCE_REQUIREMENTS_CHANGED;
//     }
//     else if (PnPFlags & PNP_DEVICE_FAILED)
//     {
//         PiSetDevNodeProblem(DeviceNode, CM_PROB_FAILED_POST_START);
//         PiSetDevNodeState(DeviceNode, DeviceNodeAwaitingQueuedRemoval);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// PiStartDeviceFinal(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     DEVICE_CAPABILITIES DeviceCapabilities;
//     NTSTATUS Status;
//
//     if (!(DeviceNode->Flags & DNF_IDS_QUERIED))
//     {
//         // query ids (for reported devices)
//         UNICODE_STRING enumRoot = RTL_CONSTANT_STRING(ENUM_ROOT);
//         HANDLE enumRootHandle, instanceHandle;
//
//         // open the enumeration root key
//         Status = IopOpenRegistryKeyEx(&enumRootHandle, NULL, &enumRoot, KEY_READ);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IopOpenRegistryKeyEx() failed for \"%wZ\" (status %x)\n", &enumRoot, Status);
//             return Status;
//         }
//
//         // open an instance subkey
//         Status = IopOpenRegistryKeyEx(&instanceHandle, enumRootHandle, &DeviceNode->InstancePath, KEY_READ);
//         ZwClose(enumRootHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to open a devnode instance key for \"%wZ\" (status %x)\n",
//                     &DeviceNode->InstancePath, Status);
//             return Status;
//         }
//
//         IopQueryHardwareIds(DeviceNode, instanceHandle);
//         IopQueryCompatibleIds(DeviceNode, instanceHandle);
//
//         DeviceNode->Flags |= DNF_IDS_QUERIED;
//         ZwClose(instanceHandle);
//     }
//
//     // we're about to start - needs enumeration
//     DeviceNode->Flags |= DNF_REENUMERATE;
//
//     DPRINT("Sending IRP_MN_QUERY_CAPABILITIES to device stack (after start)\n");
//
//     Status = IopQueryDeviceCapabilities(DeviceNode, &DeviceCapabilities);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopInitiatePnpIrp() failed (Status 0x%08lx)\n", Status);
//     }
//
//     // Query the device state (IRP_MN_QUERY_PNP_DEVICE_STATE)
//     PiUpdateDeviceState(DeviceNode);
//
//     DPRINT("Sending GUID_DEVICE_ARRIVAL %wZ\n", &DeviceNode->InstancePath);
//     IopQueueTargetDeviceEvent(&GUID_DEVICE_ARRIVAL, &DeviceNode->InstancePath);
//
//     PiSetDevNodeState(DeviceNode, DeviceNodeStarted);
//
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /**
//  * @brief      Sends one of the remove IRPs to the device stack
//  *
//  * If there is a mounted VPB attached to a one of the stack devices, the IRP
//  * should be send to a VPB's DeviceObject first (which belongs to a FS driver).
//  * FS driver will then forward it down to the volume device.
//  * While walking the device stack, the function sets (or unsets) VPB_REMOVE_PENDING flag
//  * thus blocking all further mounts on a soon-to-be-removed devices
//  */
// static
// NTSTATUS
// PiIrpSendRemoveCheckVpb(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ UCHAR MinorFunction)
// {
//     KIRQL oldIrql;
//
//     ASSERT(MinorFunction == IRP_MN_QUERY_REMOVE_DEVICE ||
//            MinorFunction == IRP_MN_CANCEL_REMOVE_DEVICE ||
//            MinorFunction == IRP_MN_SURPRISE_REMOVAL ||
//            MinorFunction == IRP_MN_REMOVE_DEVICE);
//
//     PDEVICE_OBJECT vpbDevObj = DeviceObject, targetDevice = DeviceObject;
//
//     // walk the device stack down, stop on a first mounted device
//     do
//     {
//         if (vpbDevObj->Vpb)
//         {
//             // two locks are needed here
//             KeWaitForSingleObject(&vpbDevObj->DeviceLock, Executive, KernelMode, FALSE, NULL);
//             IoAcquireVpbSpinLock(&oldIrql);
//
//             if (MinorFunction == IRP_MN_CANCEL_REMOVE_DEVICE)
//             {
//                 vpbDevObj->Vpb->Flags &= ~VPB_REMOVE_PENDING;
//             }
//             else
//             {
//                 vpbDevObj->Vpb->Flags |= VPB_REMOVE_PENDING;
//             }
//
//             BOOLEAN isMounted = (_Bool)(vpbDevObj->Vpb->Flags & VPB_MOUNTED);
//
//             if (isMounted)
//             {
//                 targetDevice = vpbDevObj->Vpb->DeviceObject;
//             }
//
//             IoReleaseVpbSpinLock(oldIrql);
//             KeSetEvent(&vpbDevObj->DeviceLock, IO_NO_INCREMENT, FALSE);
//
//             if (isMounted)
//             {
//                 break;
//             }
//         }
//
//         oldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
//         vpbDevObj = vpbDevObj->AttachedDevice;
//         KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, oldIrql);
//     } while (vpbDevObj);
//
//     ASSERT(targetDevice);
//
//     PVOID info;
//     IO_STACK_LOCATION stack = {.MajorFunction = IRP_MJ_PNP, .MinorFunction = MinorFunction};
//
//     return IopSynchronousCall(targetDevice, &stack, &info);
// }
//
// NTSTATUS
// IopUpdateResourceMapForPnPDevice(
//     IN PDEVICE_NODE DeviceNode);
//
// static
// VOID
// NTAPI
// IopSendRemoveDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(DeviceObject);
//
//     ASSERT(DeviceNode->State == DeviceNodeAwaitingQueuedRemoval);
//
//     /* Drivers should never fail a IRP_MN_REMOVE_DEVICE request */
//     PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_REMOVE_DEVICE);
//
//     /* Start of HACK: update resources stored in registry, so IopDetectResourceConflict works */
//     if (DeviceNode->ResourceList)
//     {
//         ASSERT(DeviceNode->ResourceListTranslated);
//         DeviceNode->ResourceList->Count = 0;
//         DeviceNode->ResourceListTranslated->Count = 0;
//         IopUpdateResourceMapForPnPDevice(DeviceNode);
//     }
//     /* End of HACK */
//
//     PiSetDevNodeState(DeviceNode, DeviceNodeRemoved);
//     PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_REMOVE_COMPLETE, DeviceObject, NULL);
//     LONG_PTR refCount = ObDereferenceObject(DeviceObject);
//     if (refCount != 0)
//     {
//         DPRINT1("Leaking device %wZ, refCount = %d\n", &DeviceNode->InstancePath, (INT32)refCount);
//     }
// }
//
// static
// VOID
// IopSendRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations)
// {
//     /* This function DOES dereference the device objects in all cases */
//
//     ULONG i;
//
//     for (i = 0; i < DeviceRelations->Count; i++)
//     {
//         IopSendRemoveDevice(DeviceRelations->Objects[i]);
//         DeviceRelations->Objects[i] = NULL;
//     }
//
//     ExFreePool(DeviceRelations);
// }
//
// static
// VOID
// IopSendRemoveChildDevices(PDEVICE_NODE ParentDeviceNode)
// {
//     PDEVICE_NODE ChildDeviceNode, NextDeviceNode;
//     KIRQL OldIrql;
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     ChildDeviceNode = ParentDeviceNode->Child;
//     while (ChildDeviceNode != NULL)
//     {
//         NextDeviceNode = ChildDeviceNode->Sibling;
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//         IopSendRemoveDevice(ChildDeviceNode->PhysicalDeviceObject);
//
//         ChildDeviceNode = NextDeviceNode;
//
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     }
//     KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
// }
//
// static
// VOID
// NTAPI
// IopSendSurpriseRemoval(IN PDEVICE_OBJECT DeviceObject)
// {
//     ASSERT(IopGetDeviceNode(DeviceObject)->State == DeviceNodeAwaitingQueuedRemoval);
//     /* Drivers should never fail a IRP_MN_SURPRISE_REMOVAL request */
//     PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_SURPRISE_REMOVAL);
// }
//
// static
// VOID
// NTAPI
// IopCancelRemoveDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Drivers should never fail a IRP_MN_CANCEL_REMOVE_DEVICE request */
//     PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_CANCEL_REMOVE_DEVICE);
//
//     PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_REMOVE_CANCELLED, DeviceObject, NULL);
// }
//
// static
// VOID
// IopCancelRemoveChildDevices(PDEVICE_NODE ParentDeviceNode)
// {
//     PDEVICE_NODE ChildDeviceNode, NextDeviceNode;
//     KIRQL OldIrql;
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     ChildDeviceNode = ParentDeviceNode->Child;
//     while (ChildDeviceNode != NULL)
//     {
//         NextDeviceNode = ChildDeviceNode->Sibling;
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//         IopCancelPrepareDeviceForRemoval(ChildDeviceNode->PhysicalDeviceObject);
//
//         ChildDeviceNode = NextDeviceNode;
//
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     }
//     KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
// }
//
// static
// VOID
// IopCancelRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations)
// {
//     /* This function DOES dereference the device objects in all cases */
//
//     ULONG i;
//
//     for (i = 0; i < DeviceRelations->Count; i++)
//     {
//         IopCancelPrepareDeviceForRemoval(DeviceRelations->Objects[i]);
//         ObDereferenceObject(DeviceRelations->Objects[i]);
//         DeviceRelations->Objects[i] = NULL;
//     }
//
//     ExFreePool(DeviceRelations);
// }
//
// static
// VOID
// IopCancelPrepareDeviceForRemoval(PDEVICE_OBJECT DeviceObject)
// {
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_RELATIONS DeviceRelations;
//     NTSTATUS Status;
//
//     IopCancelRemoveDevice(DeviceObject);
//
//     Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations;
//
//     Status = IopInitiatePnpIrp(DeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_DEVICE_RELATIONS,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopInitiatePnpIrp() failed with status 0x%08lx\n", Status);
//         DeviceRelations = NULL;
//     }
//     else
//     {
//         DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
//     }
//
//     if (DeviceRelations)
//         IopCancelRemoveDeviceRelations(DeviceRelations);
// }
//
// static
// NTSTATUS
// NTAPI
// IopQueryRemoveDevice(IN PDEVICE_OBJECT DeviceObject)
// {
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(DeviceObject);
//     NTSTATUS Status;
//
//     ASSERT(DeviceNode);
//
//     IopQueueTargetDeviceEvent(&GUID_DEVICE_REMOVE_PENDING,
//                               &DeviceNode->InstancePath);
//
//     Status = PiIrpSendRemoveCheckVpb(DeviceObject, IRP_MN_QUERY_REMOVE_DEVICE);
//
//     PiNotifyTargetDeviceChange(&GUID_TARGET_DEVICE_QUERY_REMOVE, DeviceObject, NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Removal vetoed by %wZ\n", &DeviceNode->InstancePath);
//         IopQueueTargetDeviceEvent(&GUID_DEVICE_REMOVAL_VETOED,
//                                   &DeviceNode->InstancePath);
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopQueryRemoveChildDevices(PDEVICE_NODE ParentDeviceNode, BOOLEAN Force)
// {
//     PDEVICE_NODE ChildDeviceNode, NextDeviceNode, FailedRemoveDevice;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     ChildDeviceNode = ParentDeviceNode->Child;
//     while (ChildDeviceNode != NULL)
//     {
//         NextDeviceNode = ChildDeviceNode->Sibling;
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//         PiSetDevNodeState(ChildDeviceNode, DeviceNodeAwaitingQueuedRemoval);
//
//         Status = IopPrepareDeviceForRemoval(ChildDeviceNode->PhysicalDeviceObject, Force);
//         if (!NT_SUCCESS(Status))
//         {
//             FailedRemoveDevice = ChildDeviceNode;
//             goto cleanup;
//         }
//
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//         ChildDeviceNode = NextDeviceNode;
//     }
//     KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//     return STATUS_SUCCESS;
//
// cleanup:
//     KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     ChildDeviceNode = ParentDeviceNode->Child;
//     while (ChildDeviceNode != NULL)
//     {
//         NextDeviceNode = ChildDeviceNode->Sibling;
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//         IopCancelPrepareDeviceForRemoval(ChildDeviceNode->PhysicalDeviceObject);
//
//         /* IRP_MN_CANCEL_REMOVE_DEVICE is also sent to the device
//          * that failed the IRP_MN_QUERY_REMOVE_DEVICE request */
//         if (ChildDeviceNode == FailedRemoveDevice)
//             return Status;
//
//         ChildDeviceNode = NextDeviceNode;
//
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//     }
//     KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopQueryRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations, BOOLEAN Force)
// {
//     /* This function DOES NOT dereference the device objects on SUCCESS
//      * but it DOES dereference device objects on FAILURE */
//
//     ULONG i, j;
//     NTSTATUS Status;
//
//     for (i = 0; i < DeviceRelations->Count; i++)
//     {
//         Status = IopPrepareDeviceForRemoval(DeviceRelations->Objects[i], Force);
//         if (!NT_SUCCESS(Status))
//         {
//             j = i;
//             goto cleanup;
//         }
//     }
//
//     return STATUS_SUCCESS;
//
// cleanup:
//     /* IRP_MN_CANCEL_REMOVE_DEVICE is also sent to the device
//      * that failed the IRP_MN_QUERY_REMOVE_DEVICE request */
//     for (i = 0; i <= j; i++)
//     {
//         IopCancelPrepareDeviceForRemoval(DeviceRelations->Objects[i]);
//         ObDereferenceObject(DeviceRelations->Objects[i]);
//         DeviceRelations->Objects[i] = NULL;
//     }
//     for (; i < DeviceRelations->Count; i++)
//     {
//         ObDereferenceObject(DeviceRelations->Objects[i]);
//         DeviceRelations->Objects[i] = NULL;
//     }
//     ExFreePool(DeviceRelations);
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopPrepareDeviceForRemoval(IN PDEVICE_OBJECT DeviceObject, BOOLEAN Force)
// {
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(DeviceObject);
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_RELATIONS DeviceRelations;
//     NTSTATUS Status;
//
//     if ((DeviceNode->UserFlags & DNUF_NOT_DISABLEABLE) && !Force)
//     {
//         DPRINT1("Removal not allowed for %wZ\n", &DeviceNode->InstancePath);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (!Force && IopQueryRemoveDevice(DeviceObject) != STATUS_SUCCESS)
//     {
//         DPRINT1("Removal vetoed by failing the query remove request\n");
//
//         IopCancelRemoveDevice(DeviceObject);
//
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations;
//
//     Status = IopInitiatePnpIrp(DeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_DEVICE_RELATIONS,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopInitiatePnpIrp() failed with status 0x%08lx\n", Status);
//         DeviceRelations = NULL;
//     }
//     else
//     {
//         DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
//     }
//
//     if (DeviceRelations)
//     {
//         Status = IopQueryRemoveDeviceRelations(DeviceRelations, Force);
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     Status = IopQueryRemoveChildDevices(DeviceNode, Force);
//     if (!NT_SUCCESS(Status))
//     {
//         if (DeviceRelations)
//             IopCancelRemoveDeviceRelations(DeviceRelations);
//         return Status;
//     }
//
//     if (DeviceRelations)
//         IopSendRemoveDeviceRelations(DeviceRelations);
//     IopSendRemoveChildDevices(DeviceNode);
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// IopRemoveDevice(PDEVICE_NODE DeviceNode)
// {
//     NTSTATUS Status;
//
//     // This function removes the device subtree, with the root in DeviceNode
//     // atm everyting is in fact done inside this function, which is completely wrong.
//     // The right implementation should have a separate removal worker thread and
//     // properly do device node state transitions
//
//     DPRINT("Removing device: %wZ\n", &DeviceNode->InstancePath);
//
//     BOOLEAN surpriseRemoval = (_Bool)(DeviceNode->Flags & DNF_DEVICE_GONE);
//
//     Status = IopPrepareDeviceForRemoval(DeviceNode->PhysicalDeviceObject, surpriseRemoval);
//
//     if (surpriseRemoval)
//     {
//         IopSendSurpriseRemoval(DeviceNode->PhysicalDeviceObject);
//         IopQueueTargetDeviceEvent(&GUID_DEVICE_SURPRISE_REMOVAL, &DeviceNode->InstancePath);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         IopSendRemoveDevice(DeviceNode->PhysicalDeviceObject);
//         if (surpriseRemoval)
//         {
//             IopQueueTargetDeviceEvent(&GUID_DEVICE_SAFE_REMOVAL, &DeviceNode->InstancePath);
//         }
//         return STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiEnumerateDevice(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     PDEVICE_OBJECT ChildDeviceObject;
//     PDEVICE_NODE ChildDeviceNode;
//     ULONG i;
//
//     // bus relations are already obtained for this device node
//
//     if (!NT_SUCCESS(DeviceNode->CompletionStatus))
//     {
//         DPRINT("QDR request failed for %wZ, status %x\n",
//             &DeviceNode->InstancePath, DeviceNode->CompletionStatus);
//         // treat as if there are no child objects
//     }
//
//     PDEVICE_RELATIONS DeviceRelations = DeviceNode->OverUsed1.PendingDeviceRelations;
//     DeviceNode->OverUsed1.PendingDeviceRelations = NULL;
//
//     // it's acceptable not to have PDOs
//     if (!DeviceRelations)
//     {
//         PiSetDevNodeState(DeviceNode, DeviceNodeStarted);
//         DPRINT("No PDOs\n");
//         return STATUS_SUCCESS;
//     }
//
//     // mark children nodes as non-present (those not returned in DR request will be removed)
//     for (PDEVICE_NODE child = DeviceNode->Child; child != NULL; child = child->Sibling)
//     {
//         child->Flags &= ~DNF_ENUMERATED;
//     }
//
//     DPRINT("PiEnumerateDevice: enumerating %u children\n", DeviceRelations->Count);
//
//     // create device nodes for all new children and set DNF_ENUMERATED back for old ones
//     for (i = 0; i < DeviceRelations->Count; i++)
//     {
//         ChildDeviceObject = DeviceRelations->Objects[i];
//         ASSERT((ChildDeviceObject->Flags & DO_DEVICE_INITIALIZING) == 0);
//
//         ChildDeviceNode = IopGetDeviceNode(ChildDeviceObject);
//         if (!ChildDeviceNode)
//         {
//             /* One doesn't exist, create it */
//             ChildDeviceNode = PipAllocateDeviceNode(ChildDeviceObject);
//             if (ChildDeviceNode)
//             {
//                 PiInsertDevNode(ChildDeviceNode, DeviceNode);
//
//                 /* Mark the node as enumerated */
//                 ChildDeviceNode->Flags |= DNF_ENUMERATED;
//
//                 /* Mark the DO as bus enumerated */
//                 ChildDeviceObject->Flags |= DO_BUS_ENUMERATED_DEVICE;
//             }
//             else
//             {
//                 /* Ignore this DO */
//                 DPRINT1("PipAllocateDeviceNode() failed. Skipping PDO %u\n", i);
//                 ObDereferenceObject(ChildDeviceObject);
//             }
//         }
//         else
//         {
//             /* Mark it as enumerated */
//             ChildDeviceNode->Flags |= DNF_ENUMERATED;
//             ObDereferenceObject(ChildDeviceObject);
//         }
//     }
//     ExFreePool(DeviceRelations);
//
//     // time to remove non-reported devices
//     for (PDEVICE_NODE child = DeviceNode->Child; child != NULL; child = child->Sibling)
//     {
//         if (!(child->Flags & (DNF_ENUMERATED|DNF_DEVICE_GONE)))
//         {
//             // this flag indicates that this is a surprise removal
//             child->Flags |= DNF_DEVICE_GONE;
//             PiSetDevNodeState(child, DeviceNodeAwaitingQueuedRemoval);
//         }
//     }
//
//     PiSetDevNodeState(DeviceNode, DeviceNodeStarted);
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// NTAPI
// IopSendEject(IN PDEVICE_OBJECT DeviceObject)
// {
//     IO_STACK_LOCATION Stack;
//     PVOID Dummy;
//
//     RtlZeroMemory(&Stack, sizeof(IO_STACK_LOCATION));
//     Stack.MajorFunction = IRP_MJ_PNP;
//     Stack.MinorFunction = IRP_MN_EJECT;
//
//     return IopSynchronousCall(DeviceObject, &Stack, &Dummy);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// IoRequestDeviceEject(IN PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     PDEVICE_NODE DeviceNode = IopGetDeviceNode(PhysicalDeviceObject);
//     PDEVICE_RELATIONS DeviceRelations;
//     IO_STATUS_BLOCK IoStatusBlock;
//     IO_STACK_LOCATION Stack;
//     DEVICE_CAPABILITIES Capabilities;
//     NTSTATUS Status;
//
//     IopQueueTargetDeviceEvent(&GUID_DEVICE_KERNEL_INITIATED_EJECT,
//                               &DeviceNode->InstancePath);
//
//     if (IopQueryDeviceCapabilities(DeviceNode, &Capabilities) != STATUS_SUCCESS)
//     {
//         goto cleanup;
//     }
//
//     Stack.Parameters.QueryDeviceRelations.Type = EjectionRelations;
//
//     Status = IopInitiatePnpIrp(PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_DEVICE_RELATIONS,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("IopInitiatePnpIrp() failed with status 0x%08lx\n", Status);
//         DeviceRelations = NULL;
//     }
//     else
//     {
//         DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
//     }
//
//     if (DeviceRelations)
//     {
//         Status = IopQueryRemoveDeviceRelations(DeviceRelations, FALSE);
//         if (!NT_SUCCESS(Status))
//             goto cleanup;
//     }
//
//     Status = IopQueryRemoveChildDevices(DeviceNode, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         if (DeviceRelations)
//             IopCancelRemoveDeviceRelations(DeviceRelations);
//         goto cleanup;
//     }
//
//     if (IopPrepareDeviceForRemoval(PhysicalDeviceObject, FALSE) != STATUS_SUCCESS)
//     {
//         if (DeviceRelations)
//             IopCancelRemoveDeviceRelations(DeviceRelations);
//         IopCancelRemoveChildDevices(DeviceNode);
//         goto cleanup;
//     }
//
//     if (DeviceRelations)
//         IopSendRemoveDeviceRelations(DeviceRelations);
//     IopSendRemoveChildDevices(DeviceNode);
//
//     DeviceNode->Problem = CM_PROB_HELD_FOR_EJECT;
//     if (Capabilities.EjectSupported)
//     {
//         if (IopSendEject(PhysicalDeviceObject) != STATUS_SUCCESS)
//         {
//             goto cleanup;
//         }
//     }
//     else
//     {
//         // DeviceNode->Flags |= DNF_DISABLED;
//     }
//
//     IopQueueTargetDeviceEvent(&GUID_DEVICE_EJECT,
//                               &DeviceNode->InstancePath);
//
//     return;
//
// cleanup:
//     IopQueueTargetDeviceEvent(&GUID_DEVICE_EJECT_VETOED,
//                               &DeviceNode->InstancePath);
// }
//
// static
// VOID
// PiFakeResourceRebalance(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     ASSERT(DeviceNode->Flags & DNF_RESOURCE_REQUIREMENTS_CHANGED);
//
//     PCM_RESOURCE_LIST bootConfig = NULL;
//     PIO_RESOURCE_REQUIREMENTS_LIST resourceRequirements = NULL;
//
//     PiIrpQueryResources(DeviceNode, &bootConfig);
//     PiIrpQueryResourceRequirements(DeviceNode, &resourceRequirements);
//
//     DeviceNode->BootResources = bootConfig;
//     DeviceNode->ResourceRequirements = resourceRequirements;
//
//     if (bootConfig)
//     {
//         DeviceNode->Flags |= DNF_HAS_BOOT_CONFIG;
//     }
//
//     DeviceNode->Flags &= ~DNF_RESOURCE_REQUIREMENTS_CHANGED;
// }
//
// static
// VOID
// PiDevNodeStateMachine(
//     _In_ PDEVICE_NODE RootNode)
// {
//     NTSTATUS status;
//     BOOLEAN doProcessAgain;
//     PDEVICE_NODE currentNode = RootNode;
//     PDEVICE_OBJECT referencedObject;
//
//     do
//     {
//         doProcessAgain = FALSE;
//
//         // The device can be removed during processing, but we still need its Parent and Sibling
//         // links to continue the tree traversal. So keep the link till the and of a cycle
//         referencedObject = currentNode->PhysicalDeviceObject;
//         ObReferenceObject(referencedObject);
//
//         // Devices with problems are skipped (unless they are not being removed)
//         if (currentNode->Flags & DNF_HAS_PROBLEM &&
//             currentNode->State != DeviceNodeAwaitingQueuedRemoval)
//         {
//             goto skipEnum;
//         }
//
//         switch (currentNode->State)
//         {
//             case DeviceNodeUnspecified: // this state is not used
//                 break;
//             case DeviceNodeUninitialized:
//                 DPRINT("DeviceNodeUninitialized %wZ\n", &currentNode->InstancePath);
//                 status = PiInitializeDevNode(currentNode);
//                 doProcessAgain = NT_SUCCESS(status);
//                 break;
//             case DeviceNodeInitialized:
//                 DPRINT("DeviceNodeInitialized %wZ\n", &currentNode->InstancePath);
//                 status = PiCallDriverAddDevice(currentNode, PnPBootDriversInitialized);
//                 doProcessAgain = NT_SUCCESS(status);
//                 break;
//             case DeviceNodeDriversAdded:
//                 DPRINT("DeviceNodeDriversAdded %wZ\n", &currentNode->InstancePath);
//                 status = IopAssignDeviceResources(currentNode);
//                 doProcessAgain = NT_SUCCESS(status);
//                 break;
//             case DeviceNodeResourcesAssigned:
//                 DPRINT("DeviceNodeResourcesAssigned %wZ\n", &currentNode->InstancePath);
//                 // send IRP_MN_START_DEVICE
//                 PiIrpStartDevice(currentNode);
//
//                 // skip DeviceNodeStartPending, it is probably used for an async IRP_MN_START_DEVICE
//                 PiSetDevNodeState(currentNode, DeviceNodeStartCompletion);
//                 doProcessAgain = TRUE;
//                 break;
//             case DeviceNodeStartPending: // skipped on XP/2003
//                 break;
//             case DeviceNodeStartCompletion:
//                 DPRINT("DeviceNodeStartCompletion %wZ\n", &currentNode->InstancePath);
//                 status = currentNode->CompletionStatus;
//                 doProcessAgain = TRUE;
//                 if (!NT_SUCCESS(status))
//                 {
//                     UINT32 problem = (status == STATUS_PNP_REBOOT_REQUIRED)
//                                      ? CM_PROB_NEED_RESTART
//                                      : CM_PROB_FAILED_START;
//
//                     PiSetDevNodeProblem(currentNode, problem);
//                     PiSetDevNodeState(currentNode, DeviceNodeAwaitingQueuedRemoval);
//                 }
//                 else
//                 {
//                     // TODO: IopDoDeferredSetInterfaceState and IopAllocateLegacyBootResources
//                     // are called here too
//
//                     PiSetDevNodeState(currentNode, DeviceNodeStartPostWork);
//                 }
//                 break;
//             case DeviceNodeStartPostWork:
//                 DPRINT("DeviceNodeStartPostWork %wZ\n", &currentNode->InstancePath);
//                 // TODO: inspect the status
//                 status = PiStartDeviceFinal(currentNode);
//                 doProcessAgain = TRUE;
//                 break;
//             case DeviceNodeStarted:
//                 if (currentNode->Flags & DNF_REENUMERATE)
//                 {
//                     DPRINT("DeviceNodeStarted REENUMERATE %wZ\n", &currentNode->InstancePath);
//                     _DA_TRACE("[DA] DeviceNodeStarted REENUMERATE\r\n");
//                     currentNode->Flags &= ~DNF_REENUMERATE;
//                     _DA_TRACE("[DA] PiIrpQueryDeviceRelations enter\r\n");
//                     status = PiIrpQueryDeviceRelations(currentNode, BusRelations);
//                     _DA_TRACE("[DA] PiIrpQueryDeviceRelations done\r\n");
//
//                     // again, skip DeviceNodeEnumeratePending as with the starting sequence
//                     PiSetDevNodeState(currentNode, DeviceNodeEnumerateCompletion);
//                     doProcessAgain = TRUE;
//                 }
//                 else if (currentNode->Flags & DNF_RESOURCE_REQUIREMENTS_CHANGED)
//                 {
//                     if (currentNode->Flags & DNF_NON_STOPPED_REBALANCE)
//                     {
//                         PiFakeResourceRebalance(currentNode);
//                         currentNode->Flags &= ~DNF_NON_STOPPED_REBALANCE;
//                     }
//                     else
//                     {
//                         PiIrpQueryStopDevice(currentNode);
//                         PiSetDevNodeState(currentNode, DeviceNodeQueryStopped);
//                     }
//
//                     doProcessAgain = TRUE;
//                 }
//                 break;
//             case DeviceNodeQueryStopped:
//                 // we're here after sending IRP_MN_QUERY_STOP_DEVICE
//                 status = currentNode->CompletionStatus;
//                 if (NT_SUCCESS(status))
//                 {
//                     PiIrpStopDevice(currentNode);
//                     PiSetDevNodeState(currentNode, DeviceNodeStopped);
//                 }
//                 else
//                 {
//                     PiIrpCancelStopDevice(currentNode);
//                     PiSetDevNodeState(currentNode, DeviceNodeStarted);
//                 }
//                 doProcessAgain = TRUE;
//                 break;
//             case DeviceNodeStopped:
//                 // TODO: do resource rebalance (not implemented)
//                 PiFakeResourceRebalance(currentNode);
//
//                 PiSetDevNodeState(currentNode, DeviceNodeDriversAdded);
//                 doProcessAgain = TRUE;
//                 break;
//             case DeviceNodeRestartCompletion:
//                 break;
//             case DeviceNodeEnumeratePending: // skipped on XP/2003
//                 break;
//             case DeviceNodeEnumerateCompletion:
//                 DPRINT("DeviceNodeEnumerateCompletion %wZ\n", &currentNode->InstancePath);
//                 _DA_TRACE("[DA] PiEnumerateDevice enter\r\n");
//                 status = PiEnumerateDevice(currentNode);
//                 _DA_TRACE("[DA] PiEnumerateDevice done\r\n");
//                 doProcessAgain = TRUE;
//                 break;
//             case DeviceNodeAwaitingQueuedDeletion:
//                 break;
//             case DeviceNodeAwaitingQueuedRemoval:
//                 DPRINT("DeviceNodeAwaitingQueuedRemoval %wZ\n", &currentNode->InstancePath);
//                 status = IopRemoveDevice(currentNode);
//                 break;
//             case DeviceNodeQueryRemoved:
//                 break;
//             case DeviceNodeRemovePendingCloses:
//                 break;
//             case DeviceNodeRemoved:
//                 break;
//             case DeviceNodeDeletePendingCloses:
//                 break;
//             case DeviceNodeDeleted:
//                 break;
//             default:
//                 break;
//         }
//
// skipEnum:
//         if (!doProcessAgain)
//         {
//             KIRQL OldIrql;
//             KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//             /* If we have a child, simply go down the tree */
//             if (currentNode->State != DeviceNodeRemoved && currentNode->Child != NULL)
//             {
//                 ASSERT(currentNode->Child->Parent == currentNode);
//                 currentNode = currentNode->Child;
//             }
//             else
//             {
//                 while (currentNode != RootNode)
//                 {
//                     /* All children processed -- go sideways */
//                     if (currentNode->Sibling != NULL)
//                     {
//                         ASSERT(currentNode->Sibling->Parent == currentNode->Parent);
//                         currentNode = currentNode->Sibling;
//                         break;
//                     }
//                     else
//                     {
//                         /* We're the last sibling -- go back up */
//                         ASSERT(currentNode->Parent->LastChild == currentNode);
//                         currentNode = currentNode->Parent;
//                     }
//                     /* We already visited the parent and all its children, so keep looking */
//                 }
//             }
//             KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//         }
//         ObDereferenceObject(referencedObject);
//     } while (doProcessAgain || currentNode != RootNode);
// }
//
// #ifdef DBG
// static
// PCSTR
// ActionToStr(
//     _In_ DEVICE_ACTION Action)
// {
//     switch (Action)
//     {
//         case PiActionEnumDeviceTree:
//             return "PiActionEnumDeviceTree";
//         case PiActionEnumRootDevices:
//             return "PiActionEnumRootDevices";
//         case PiActionResetDevice:
//             return "PiActionResetDevice";
//         case PiActionAddBootDevices:
//             return "PiActionAddBootDevices";
//         case PiActionStartDevice:
//             return "PiActionStartDevice";
//         case PiActionQueryState:
//             return "PiActionQueryState";
//         default:
//             return "(request unknown)";
//     }
// }
// #endif
//
// static
// VOID
// NTAPI
// PipDeviceActionWorker(
//     _In_opt_ PVOID Context)
// {
//     PLIST_ENTRY ListEntry;
//     PDEVICE_ACTION_REQUEST Request;
//     KIRQL OldIrql;
//     PDEVICE_NODE deviceNode;
//     NTSTATUS status;
//
//     KeAcquireSpinLock(&IopDeviceActionLock, &OldIrql);
//     while (!IsListEmpty(&IopDeviceActionRequestList))
//     {
//         ListEntry = RemoveHeadList(&IopDeviceActionRequestList);
//         KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
//         Request = CONTAINING_RECORD(ListEntry, DEVICE_ACTION_REQUEST, RequestListEntry);
//
//         ASSERT(Request->DeviceObject);
//
//         deviceNode = IopGetDeviceNode(Request->DeviceObject);
//         ASSERT(deviceNode);
//
//         status = STATUS_SUCCESS;
//
//         DPRINT("Processing PnP request %p: DeviceObject - %p, Action - %s\n",
//                Request, Request->DeviceObject, ActionToStr(Request->Action));
//
//         _DA_TRACE("[DA] PipDeviceActionWorker processing\r\n");
//         switch (Request->Action)
//         {
//             case PiActionAddBootDevices:
//             {
//                 if (deviceNode->State == DeviceNodeInitialized &&
//                     !(deviceNode->Flags & DNF_HAS_PROBLEM))
//                 {
//                     status = PiCallDriverAddDevice(deviceNode, PnPBootDriversInitialized);
//                 }
//                 break;
//             }
//             case PiActionEnumRootDevices:
//             case PiActionEnumDeviceTree:
//                 _DA_TRACE("[DA] PiDevNodeStateMachine enter\r\n");
//                 deviceNode->Flags |= DNF_REENUMERATE;
//                 PiDevNodeStateMachine(deviceNode);
//                 _DA_TRACE("[DA] PiDevNodeStateMachine done\r\n");
//                 break;
//
//             case PiActionResetDevice:
//                 // TODO: the operation is a no-op for everything except removed nodes
//                 // for removed nodes, it returns them back to DeviceNodeUninitialized
//                 if (deviceNode->State == DeviceNodeRemoved)
//                 {
//                     deviceNode->State = DeviceNodeUninitialized;
//                 }
//                 status = STATUS_SUCCESS;
//                 break;
//
//             case PiActionStartDevice:
//                 // This action is triggered from usermode, when a driver is installed
//                 // for a non-critical PDO
//                 if (deviceNode->State == DeviceNodeInitialized &&
//                     !(deviceNode->Flags & DNF_HAS_PROBLEM))
//                 {
//                     PiDevNodeStateMachine(deviceNode);
//                 }
//                 else
//                 {
//                     DPRINT1("NOTE: attempt to start an already started/uninitialized device %wZ\n",
//                             &deviceNode->InstancePath);
//                     status = STATUS_UNSUCCESSFUL;
//                 }
//                 break;
//
//             case PiActionQueryState:
//                 // This action is only valid for started devices. If the device is not yet
//                 // started, the PnP manager issues IRP_MN_QUERY_PNP_DEVICE_STATE by itself.
//                 if (deviceNode->State == DeviceNodeStarted)
//                 {
//                     // Issue a IRP_MN_QUERY_PNP_DEVICE_STATE request: it will update node's flags
//                     // and then do enumeration if something has changed
//                     status = PiUpdateDeviceState(deviceNode);
//                     if (NT_SUCCESS(status))
//                     {
//                         PiDevNodeStateMachine(deviceNode);
//                     }
//                 }
//                 // TODO: Windows may return STATUS_DELETE_PENDING here
//                 status = STATUS_SUCCESS;
//                 break;
//
//             default:
//                 DPRINT1("Unimplemented device action %u\n", Request->Action);
//                 status = STATUS_NOT_IMPLEMENTED;
//                 break;
//         }
//
//         if (Request->CompletionStatus)
//         {
//             *Request->CompletionStatus = status;
//         }
//
//         if (Request->CompletionEvent)
//         {
//             KeSetEvent(Request->CompletionEvent, IO_NO_INCREMENT, FALSE);
//         }
//
//         DPRINT("Finished processing PnP request %p\n", Request);
//         ObDereferenceObject(Request->DeviceObject);
//         ExFreePoolWithTag(Request, TAG_IO);
//         KeAcquireSpinLock(&IopDeviceActionLock, &OldIrql);
//     }
//     IopDeviceActionInProgress = FALSE;
//     KeSetEvent(&PiEnumerationFinished, IO_NO_INCREMENT, FALSE);
//     KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
// }
//
// /**
//  * @brief      Queue a device operation to a worker thread.
//  *
//  * @param[in]  DeviceObject      The device object
//  * @param[in]  Action            The action
//  * @param[in]  CompletionEvent   The completion event object (optional)
//  * @param[out] CompletionStatus  Status returned be the action will be written here
//  */
//
// VOID
// PiQueueDeviceAction(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ DEVICE_ACTION Action,
//     _In_opt_ PKEVENT CompletionEvent,
//     _Out_opt_ NTSTATUS *CompletionStatus)
// {
//     PDEVICE_ACTION_REQUEST Request;
//     KIRQL OldIrql;
//
//     Request = ExAllocatePoolWithTag(NonPagedPoolMustSucceed, sizeof(*Request), TAG_IO);
//
//     DPRINT("PiQueueDeviceAction: DeviceObject - %p, Request - %p, Action - %s\n",
//         DeviceObject, Request, ActionToStr(Action));
//
//     ObReferenceObject(DeviceObject);
//
//     Request->DeviceObject = DeviceObject;
//     Request->Action = Action;
//     Request->CompletionEvent = CompletionEvent;
//     Request->CompletionStatus = CompletionStatus;
//
//     KeAcquireSpinLock(&IopDeviceActionLock, &OldIrql);
//     InsertTailList(&IopDeviceActionRequestList, &Request->RequestListEntry);
//
//     if (Action == PiActionEnumRootDevices || Action == PiActionAddBootDevices)
//     {
//         ASSERT(!IopDeviceActionInProgress);
//
//         IopDeviceActionInProgress = TRUE;
//         KeClearEvent(&PiEnumerationFinished);
//         KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
//
//         PipDeviceActionWorker(NULL);
//         return;
//     }
//
//     if (IopDeviceActionInProgress || !PnPBootDriversLoaded)
//     {
//         KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
//         return;
//     }
//     IopDeviceActionInProgress = TRUE;
//     KeClearEvent(&PiEnumerationFinished);
//     KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
//
//     ExInitializeWorkItem(&IopDeviceActionWorkItem, PipDeviceActionWorker, NULL);
//     ExQueueWorkItem(&IopDeviceActionWorkItem, DelayedWorkQueue);
// }
//
// /**
//  * @brief      Perfom a device operation synchronously via PiQueueDeviceAction
//  *
//  * @param[in]  DeviceObject  The device object
//  * @param[in]  Action        The action
//  *
//  * @return     Status of the operation
//  */
//
// NTSTATUS
// PiPerformSyncDeviceAction(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ DEVICE_ACTION Action)
// {
//     KEVENT opFinished;
//     NTSTATUS status;
//
//     KeInitializeEvent(&opFinished, SynchronizationEvent, FALSE);
//     PiQueueDeviceAction(DeviceObject, Action, &opFinished, &status);
//     KeWaitForSingleObject(&opFinished, Executive, KernelMode, FALSE, NULL);
//
//     return status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
