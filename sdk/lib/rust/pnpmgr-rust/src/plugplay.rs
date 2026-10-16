use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _DEVICE_CAPABILITIES,
    _DEVICE_NODE,
    _DEVICE_RELATIONS,
    _DEVICETREE_TRAVERSE_CONTEXT,
    DEVICETREE_TRAVERSE_ROUTINE,
    _IO_STACK_LOCATION,
    _IO_STATUS_BLOCK,
    _KEVENT,
    _PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA,
    _PLUGPLAY_CONTROL_DEPTH_DATA,
    _PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
    _PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA,
    _PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA,
    _PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA,
    _PLUGPLAY_CONTROL_PROPERTY_DATA,
    _PLUGPLAY_CONTROL_QUERY_REMOVE_DATA,
    _PLUGPLAY_CONTROL_RELATED_DEVICE_DATA,
    _PLUGPLAY_CONTOL_STATUS_DATA,
    _PLUGPLAY_CONTROL_USER_RESPONSE_DATA,
    _PLUGPLAY_EVENT_BLOCK,
    CM_Power_Data_s, _UNICODE_STRING,
    DEVICE_CAPABILITIES,
    DEVICE_REGISTRY_PROPERTY,
    GUID, LUID, KPROCESSOR_MODE,
    IO_STACK_LOCATION,
    KEVENT,
    PDEVICE_NODE,
    PDEVICE_OBJECT,
    PDEVICE_RELATIONS,
    PCM_POWER_DATA,
    PPLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA,
    PPLUGPLAY_CONTROL_DEPTH_DATA,
    PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
    PPLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA,
    PPLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA,
    PPLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA,
    PPLUGPLAY_CONTROL_PROPERTY_DATA,
    PPLUGPLAY_CONTROL_QUERY_REMOVE_DATA,
    PPLUGPLAY_CONTROL_RELATED_DEVICE_DATA,
    PPLUGPLAY_CONTROL_STATUS_DATA,
    PPLUGPLAY_CONTROL_USER_RESPONSE_DATA,
    PPLUGPLAY_EVENT_BLOCK,
    PUNICODE_STRING,
    PZZWSTR,
    ExAllocatePool,
    ExFreePool,
    IoDeleteDevice,
    IoGetDeviceInterfaceAlias,
    IoGetDeviceInterfaces,
    IoRegisterDeviceInterface,
    IopCreateDeviceKeyPath,
    IopFreeDeviceNode,
    IopGetDeviceNode,
    IopTraverseDeviceTree,
    KeInitializeEvent,
    KeWaitForSingleObject,
    RtlAppendUnicodeStringToString,
    _KWAIT_REASON_UserRequest,
    _EVENT_TYPE_SynchronizationEvent,
};

use ntkrnl_sys::probe::*;
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


extern "C" {
    static mut IopRootDeviceNode: PDEVICE_NODE;
}
// NTAPI (stdcall on i386) functions
extern "system" {
    static mut SeTcbPrivilege: LUID;
    fn SeSinglePrivilegeCheck(PrivilegeValue: LUID, PreviousMode: KPROCESSOR_MODE) -> BOOLEAN;
    fn RtlDuplicateUnicodeString(Flags: ULONG, SourceString: PUNICODE_STRING, DestinationString: PUNICODE_STRING) -> NTSTATUS;
    fn IopInitiatePnpIrp(DeviceObject: PDEVICE_OBJECT, IoStatusBlock: *mut _IO_STATUS_BLOCK, MinorFunction: UCHAR, Stack: *mut _IO_STACK_LOCATION) -> NTSTATUS;
    fn IoGetDeviceProperty(DeviceObject: PDEVICE_OBJECT, DeviceProperty: DEVICE_REGISTRY_PROPERTY, BufferLength: ULONG, PropertyBuffer: PVOID, ResultLength: *mut ULONG) -> NTSTATUS;
    fn RtlEqualUnicodeString(String1: PUNICODE_STRING, String2: PUNICODE_STRING, CaseInSensitive: BOOLEAN) -> BOOLEAN;
    fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: *const u16);
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn IopInitDeviceTreeTraverseContext(Context: *mut _DEVICETREE_TRAVERSE_CONTEXT, DeviceNode: PDEVICE_NODE, Callback: DEVICETREE_TRAVERSE_ROUTINE, CallbackContext: PVOID);
}
// cdecl functions (no NTAPI in C declaration, or CRT functions)
extern "C" {
    fn IopSetDeviceInstanceData(InstanceKey: HANDLE, DeviceNode: PDEVICE_NODE) -> NTSTATUS;
    fn PnpRootCreateDeviceObject(DeviceObject: *mut PDEVICE_OBJECT) -> NTSTATUS;
    fn PipAllocateDeviceNode(PhysicalDeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, NewState: ULONG) -> u32;
    fn PnpRootRegisterDevice(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS;
    fn PiInsertDevNode(DeviceNode: PDEVICE_NODE, ParentNode: PDEVICE_NODE);
    fn PiPerformSyncDeviceAction(DeviceObject: PDEVICE_OBJECT, Action: ULONG) -> NTSTATUS;
    fn wcslen(s: *const u16) -> usize;
    fn memcpy(dest: PVOID, src: *const core::ffi::c_void, n: usize) -> PVOID;
}

const RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE: ULONG = 1;
const REG_OPTION_NON_VOLATILE: ULONG = 0;

const DO_BUS_ENUMERATED_DEVICE: ULONG = 0x1000;
const DNF_MADEUP: ULONG = 0x00000001;
const DNF_IDS_QUERIED: ULONG = 0x20;
const DNF_ENUMERATED: ULONG = 0x10;
const DNF_HAS_PROBLEM: ULONG = 0x2000;
const DNF_HAS_PRIVATE_PROBLEM: ULONG = 0x4000;
const DNF_DRIVER_BLOCKED: ULONG = 0x00100000;
const DNF_CHILD_WITH_INVALID_ID: ULONG = 0x00200000;
const DNF_LEGACY_DRIVER: ULONG = 0x00001000;

const DNUF_WILL_BE_REMOVED: ULONG = 0x00000001;
const DNUF_DONT_SHOW_IN_UI: ULONG = 0x00000002;
const DNUF_NOT_DISABLEABLE: ULONG = 0x00000008;

const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as NTSTATUS;
const STATUS_USER_APC: NTSTATUS = 0x000000C0_u32 as NTSTATUS;

const DN_ROOT_ENUMERATED: ULONG = 0x00000001;
const DN_DRIVER_LOADED: ULONG = 0x00000002;
const DN_ENUM_LOADED: ULONG = 0x00000004;
const DN_STARTED: ULONG = 0x00000008;
const DN_MANUAL: ULONG = 0x00000010;
const DN_NEED_TO_ENUM: ULONG = 0x00000020;
const DN_NOT_FIRST_TIME: ULONG = 0x00000040;
const DN_HARDWARE_ENUM: ULONG = 0x00000080;
const DN_LIAR: ULONG = 0x00000100;
const DN_HAS_MARK: ULONG = 0x00000200;
const DN_HAS_PROBLEM: ULONG = 0x00000400;
const DN_FILTERED: ULONG = 0x00000800;
const DN_MOVED: ULONG = 0x00001000;
const DN_DISABLEABLE: ULONG = 0x00002000;
const DN_REMOVABLE: ULONG = 0x00004000;
const DN_PRIVATE_PROBLEM: ULONG = 0x00008000;
const DN_MF_PARENT: ULONG = 0x00010000;
const DN_MF_CHILD: ULONG = 0x00020000;
const DN_WILL_BE_REMOVED: ULONG = 0x00040000;
const DN_NOT_FIRST_TIMEE: ULONG = 0x00080000;
const DN_STOP_FREE_RES: ULONG = 0x00100000;
const DN_REBAL_CANDIDATE: ULONG = 0x00200000;
const DN_BAD_PARTIAL: ULONG = 0x00400000;
const DN_NT_ENUMERATOR: ULONG = 0x00800000;
const DN_NT_DRIVER: ULONG = 0x01000000;
const DN_NEEDS_LOCKING: ULONG = 0x02000000;
const DN_ARM_WAKEUP: ULONG = 0x04000000;
const DN_APM_ENUMERATOR: ULONG = 0x08000000;
const DN_APM_DRIVER: ULONG = 0x10000000;
const DN_SILENT_INSTALL: ULONG = 0x20000000;
const DN_NO_SHOW_IN_DM: ULONG = 0x40000000;
const DN_BOOT_LOG_PROB: ULONG = 0x80000000;
const DN_DRIVER_BLOCKED: ULONG = 0x00000040;
const DN_LEGACY_DRIVER: ULONG = DN_MOVED; // psdk/cfg.h:155 (NTDDI >= WINXP); 0x80 is DN_HARDWARE_ENUM
const DN_CHILD_WITH_INVALID_ID: ULONG = 0x00000200;

const DeviceNodeInitialized: ULONG = 0x00000302;
const DeviceNodeDriversAdded: ULONG = 0x303;
const DeviceNodeStartPending: ULONG = 0x305;
const DeviceNodeStartCompletion: ULONG = 0x306;
const DeviceNodeStartPostWork: ULONG = 0x307;
const DeviceNodeStarted: ULONG = 0x308;
const DeviceNodeQueryStopped: ULONG = 0x309;
const DeviceNodeEnumeratePending: ULONG = 0x0000030C;
const DeviceNodeEnumerateCompletion: ULONG = 0x0000030D;
const DeviceNodeStopped: ULONG = 0x0000030A;
const DeviceNodeRestartCompletion: ULONG = 0x0000030B;

const PNP_GET_PARENT_DEVICE: ULONG = 1;
const PNP_GET_CHILD_DEVICE: ULONG = 2;
const PNP_GET_SIBLING_DEVICE: ULONG = 3;

const PNP_GET_DEVICE_STATUS: ULONG = 0;
const PNP_SET_DEVICE_STATUS: ULONG = 1;
const PNP_CLEAR_DEVICE_STATUS: ULONG = 2;

const PNP_EJECT_RELATIONS: ULONG = 0;
const PNP_REMOVAL_RELATIONS: ULONG = 1;
const PNP_POWER_RELATIONS: ULONG = 2;
const PNP_BUS_RELATIONS: ULONG = 3;

const PNP_PROPERTY_POWER_DATA: ULONG = 0x5;
const PNP_PROPERTY_REMOVAL_POLICY_OVERRIDE: ULONG = 0x7;
const PNP_PROPERTY_REMOVAL_POLICY_HARDWARE_DEFAULT: ULONG = 0xa;
const PNP_PROPERTY_UI_NUMBER: ULONG = 0x0;
const PNP_PROPERTY_PHYSICAL_DEVICE_OBJECT_NAME: ULONG = 0x1;
const PNP_PROPERTY_BUSTYPEGUID: ULONG = 0x2;
const PNP_PROPERTY_LEGACYBUSTYPE: ULONG = 0x3;
const PNP_PROPERTY_BUSNUMBER: ULONG = 0x4;
const PNP_PROPERTY_REMOVAL_POLICY: ULONG = 0x6;
const PNP_PROPERTY_ADDRESS: ULONG = 0x8;
const PNP_PROPERTY_ENUMERATOR_NAME: ULONG = 0x9;
const PNP_PROPERTY_INSTALL_STATE: ULONG = 0xb;
const PNP_PROPERTY_LOCATION_PATHS: ULONG = 0xc;
const PNP_PROPERTY_CONTAINERID: ULONG = 0xd;

const DevicePropertyUINumber: DEVICE_REGISTRY_PROPERTY = 0x00000011;
const DevicePropertyPhysicalDeviceObjectName: DEVICE_REGISTRY_PROPERTY = 14;
const DevicePropertyBusTypeGuid: DEVICE_REGISTRY_PROPERTY = 12; // header value; the hand copy said 19
const DevicePropertyLegacyBusType: DEVICE_REGISTRY_PROPERTY = 0x0000000D;
const DevicePropertyBusNumber: DEVICE_REGISTRY_PROPERTY = 0x0000000E;
const DevicePropertyRemovalPolicy: DEVICE_REGISTRY_PROPERTY = 0x00000013;
const DevicePropertyAddress: DEVICE_REGISTRY_PROPERTY = 0x00000010;
const DevicePropertyEnumeratorName: DEVICE_REGISTRY_PROPERTY = 15; // header value; the hand copy said 22
const DevicePropertyInstallState: DEVICE_REGISTRY_PROPERTY = 0x00000012;
const DevicePropertyContainerID: DEVICE_REGISTRY_PROPERTY = 22; // header value; the hand copy said 28

const EjectionRelations: ULONG = 0x00000001;
const RemovalRelations: ULONG = 0x00000003;
const PowerRelations: ULONG = 2;
const BusRelations: ULONG = 0x00000000;

const IRP_MN_QUERY_CAPABILITIES: UCHAR = 0x09;
const IRP_MN_QUERY_DEVICE_RELATIONS: UCHAR = 0x07;

const PlugPlayControlEnumerateDevice: ULONG = 0;
const PlugPlayControlRegisterNewDevice: ULONG = 1;
const PlugPlayControlDeregisterDevice: ULONG = 2;
const PlugPlayControlInitializeDevice: ULONG = 3;
const PlugPlayControlStartDevice: ULONG = 4;
const PlugPlayControlUnlockDevice: ULONG = 5;
const PlugPlayControlQueryAndRemoveDevice: ULONG = 6;
const PlugPlayControlUserResponse: ULONG = 7;
const PlugPlayControlGenerateLegacyDevice: ULONG = 8;
const PlugPlayControlGetInterfaceDeviceList: ULONG = 9;
const PlugPlayControlProperty: ULONG = 10;
const PlugPlayControlDeviceClassAssociation: ULONG = 11;
const PlugPlayControlGetRelatedDevice: ULONG = 12;
const PlugPlayControlGetInterfaceDeviceAlias: ULONG = 13;
const PlugPlayControlDeviceStatus: ULONG = 14;
const PlugPlayControlGetDeviceDepth: ULONG = 15;
const PlugPlayControlQueryDeviceRelations: ULONG = 16;
const PlugPlayControlResetDevice: ULONG = 20;

const DeviceClassChangeEvent: ULONG = 0x02;
const DeviceInstallEvent: ULONG = 0x04;
const DevicePropertyChangeEvent: ULONG = ntkrnl_sys::ddk::_PLUGPLAY_EVENT_CATEGORY_DevicePropertyChangeEvent as ULONG;
const TargetDeviceChangeEvent: ULONG = 0x01;

const SynchronizationEvent: ULONG = 0x01;
const UserRequest: ULONG = 0x06;
const UserMode: KPROCESSOR_MODE = 1i8;

const PowerDeviceD0: ULONG = 1;

const PDCAP_D1_SUPPORTED: ULONG = 0x2;
const PDCAP_D2_SUPPORTED: ULONG = 0x4;
const PDCAP_WAKE_FROM_D0_SUPPORTED: ULONG = 0x10;
const PDCAP_WAKE_FROM_D1_SUPPORTED: ULONG = 0x20;
const PDCAP_WAKE_FROM_D2_SUPPORTED: ULONG = 0x40;
const PDCAP_WAKE_FROM_D3_SUPPORTED: ULONG = 0x80;
const PDCAP_WARM_EJECT_SUPPORTED: ULONG = 0x100;
const PDCAP_D0_SUPPORTED: ULONG = 0x1;
const PDCAP_D3_SUPPORTED: ULONG = 0x8;

const PiActionEnumDeviceTree: ULONG = 0;
const PiActionStartDevice: ULONG = 0x00000004;
const PiActionResetDevice: ULONG = 2;

const MUTANT_INCREMENT: LONG = 1;

extern "system" {
    static GUID_DEVICE_ENUMERATED: GUID;
}

#[repr(C)]
struct PNP_EVENT_ENTRY {
    ListEntry: LIST_ENTRY,
    Event: _PLUGPLAY_EVENT_BLOCK,
}

type PPNP_EVENT_ENTRY = *mut PNP_EVENT_ENTRY;

#[repr(C)]
struct IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT {
    InstancePath: PUNICODE_STRING,
    DeviceObject: PDEVICE_OBJECT,
}

type PIOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT = *mut IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT;

static mut IopPnpEventQueueHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
static mut IopPnpNotifyEvent: KEVENT = unsafe { core::mem::zeroed() };

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "C" fn IopInitPlugPlayEvents() -> NTSTATUS {
    InitializeListHead(&raw mut IopPnpEventQueueHead);

    KeInitializeEvent(
        &raw mut IopPnpNotifyEvent,
        _EVENT_TYPE_SynchronizationEvent as u32,
        0u8,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopQueueDeviceChangeEvent(
    EventGuid: *const GUID,
    InterfaceClassGuid: *const GUID,
    SymbolicLinkName: PUNICODE_STRING,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let mut Copy: _UNICODE_STRING = core::mem::zeroed();
    
    Copy.Length = 0;
    Copy.MaximumLength = (*SymbolicLinkName).Length + core::mem::size_of::<u16>() as u16;
    let TotalSize = core::mem::size_of::<_PLUGPLAY_EVENT_BLOCK>() + Copy.MaximumLength as usize;

    let EventEntry = ExAllocatePool(
        NonPagedPool,
        (TotalSize + core::mem::size_of::<LIST_ENTRY>()) as _,
    ) as PPNP_EVENT_ENTRY;
    
    if EventEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    
    core::ptr::write_bytes(EventEntry as *mut u8, 0, TotalSize + core::mem::size_of::<LIST_ENTRY>());

    core::ptr::copy_nonoverlapping(
        EventGuid as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.EventGuid) as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    (*EventEntry).Event.EventCategory = DeviceClassChangeEvent as u32;
    (*EventEntry).Event.TotalSize = TotalSize as ULONG;

    core::ptr::copy_nonoverlapping(
        InterfaceClassGuid as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.DeviceClass.ClassGuid) as *mut u8,
        core::mem::size_of::<GUID>(),
    );

    core::ptr::copy_nonoverlapping(
        (*SymbolicLinkName).Buffer as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.DeviceClass.SymbolicLinkName) as *mut u8,
        (*SymbolicLinkName).Length as usize,
    );
    
    let sym_link_ptr = core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.DeviceClass.SymbolicLinkName) as *mut u16;
    *sym_link_ptr.add(((*SymbolicLinkName).Length / core::mem::size_of::<u16>() as u16) as usize) = 0;

    InsertHeadList(
        &raw mut IopPnpEventQueueHead,
        core::ptr::addr_of_mut!((*EventEntry).ListEntry),
    );
    
    KeSetEvent(core::ptr::addr_of_mut!(IopPnpNotifyEvent) as PVOID, 0i32, 0u8);

    STATUS_SUCCESS
}

/// {F455D791-057F-48D8-B075-D125C401A137} (umpnpmgr/sysguid.h).
static GUID_PNP_PROPERTY_UPDATE: GUID = GUID {
    Data1: 0xF455D791,
    Data2: 0x057F,
    Data3: 0x48D8,
    Data4: [0xB0, 0x75, 0xD1, 0x25, 0xC4, 0x01, 0xA1, 0x37],
};

/// Queue a DevicePropertyChangeEvent for umpnpmgr (upstream 3cc9aa99292): the property
/// store raises it when a device's property set changes.
#[no_mangle]
pub unsafe extern "C" fn IopQueueDevicePropertyChangeEvent(DeviceId: ntkrnl_sys::ddk::PCUNICODE_STRING) -> NTSTATUS {
    let DeviceId = opaque_ptr_const(DeviceId);
    // FIELD_OFFSET(PLUGPLAY_EVENT_BLOCK, TargetDevice.DeviceIds) + Length + sizeof(UNICODE_NULL)
    let probe = core::mem::MaybeUninit::<_PLUGPLAY_EVENT_BLOCK>::uninit();
    let base = probe.as_ptr();
    let ids_offset = core::ptr::addr_of!((*base).__bindgen_anon_1.TargetDevice.DeviceIds) as usize - base as usize;
    let TotalSize = ids_offset + (*DeviceId).Length as usize + core::mem::size_of::<u16>();
    let EntrySize = core::mem::offset_of!(PNP_EVENT_ENTRY, Event) + TotalSize;

    // ExAllocatePoolZero(NonPagedPool, EntrySize, TAG_IO)
    let EventEntry = ntkrnl_sys::ddk::ExAllocatePoolWithTag(NonPagedPool as _, EntrySize as _, u32::from_le_bytes(*b"Io  ")) as PPNP_EVENT_ENTRY;
    if EventEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(EventEntry as *mut u8, 0, EntrySize);

    (*EventEntry).Event.EventGuid = GUID_PNP_PROPERTY_UPDATE;
    (*EventEntry).Event.EventCategory = DevicePropertyChangeEvent as u32;
    (*EventEntry).Event.TotalSize = TotalSize as ULONG;

    core::ptr::copy_nonoverlapping(
        (*DeviceId).Buffer as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.TargetDevice.DeviceIds) as *mut u8,
        (*DeviceId).Length as usize,
    );

    InsertHeadList(
        &raw mut IopPnpEventQueueHead,
        core::ptr::addr_of_mut!((*EventEntry).ListEntry),
    );
    KeSetEvent(core::ptr::addr_of_mut!(IopPnpNotifyEvent) as PVOID, 0i32, 0u8);

    STATUS_SUCCESS
}


#[no_mangle]
pub unsafe extern "C" fn IopQueueDeviceInstallEvent(
    EventGuid: *const GUID,
    DeviceId: PUNICODE_STRING,
) -> NTSTATUS {
    let DeviceId = opaque_ptr(DeviceId);

    let mut Copy: _UNICODE_STRING = core::mem::zeroed();
    
    Copy.Length = 0;
    Copy.MaximumLength = (*DeviceId).Length + core::mem::size_of::<u16>() as u16;
    let TotalSize = core::mem::size_of::<_PLUGPLAY_EVENT_BLOCK>() + Copy.MaximumLength as usize;

    let EventEntry = ExAllocatePool(
        NonPagedPool,
        (TotalSize + core::mem::size_of::<LIST_ENTRY>()) as _,
    ) as PPNP_EVENT_ENTRY;
    
    if EventEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    
    core::ptr::write_bytes(EventEntry as *mut u8, 0, TotalSize + core::mem::size_of::<LIST_ENTRY>());

    core::ptr::copy_nonoverlapping(
        EventGuid as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.EventGuid) as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    (*EventEntry).Event.EventCategory = DeviceInstallEvent as u32;
    (*EventEntry).Event.TotalSize = TotalSize as ULONG;

    core::ptr::copy_nonoverlapping(
        (*DeviceId).Buffer as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.InstallDevice.DeviceId) as *mut u8,
        (*DeviceId).Length as usize,
    );
    
    let device_id_ptr = core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.InstallDevice.DeviceId) as *mut u16;
    *device_id_ptr.add(((*DeviceId).Length / core::mem::size_of::<u16>() as u16) as usize) = 0;

    InsertHeadList(
        &raw mut IopPnpEventQueueHead,
        core::ptr::addr_of_mut!((*EventEntry).ListEntry),
    );

    KeSetEvent(core::ptr::addr_of_mut!(IopPnpNotifyEvent) as PVOID, 0i32, 0u8);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopQueueTargetDeviceEvent(
    EventGuid: *const GUID,
    DeviceIds: PUNICODE_STRING,
) -> NTSTATUS {
    let DeviceIds = opaque_ptr(DeviceIds);
    let mut Copy: _UNICODE_STRING = core::mem::zeroed();
    
    Copy.Length = 0;
    Copy.MaximumLength = (*DeviceIds).Length + core::mem::size_of::<u16>() as u16;
    let TotalSize = core::mem::size_of::<_PLUGPLAY_EVENT_BLOCK>() + Copy.MaximumLength as usize;

    let EventEntry = ExAllocatePool(
        NonPagedPool,
        (TotalSize + core::mem::size_of::<LIST_ENTRY>()) as _,
    ) as PPNP_EVENT_ENTRY;
    
    if EventEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    
    core::ptr::write_bytes(EventEntry as *mut u8, 0, TotalSize + core::mem::size_of::<LIST_ENTRY>());

    core::ptr::copy_nonoverlapping(
        EventGuid as *const u8,
        core::ptr::addr_of_mut!((*EventEntry).Event.EventGuid) as *mut u8,
        core::mem::size_of::<GUID>(),
    );
    (*EventEntry).Event.EventCategory = TargetDeviceChangeEvent as u32;
    (*EventEntry).Event.TotalSize = TotalSize as ULONG;

    Copy.Buffer = core::ptr::addr_of_mut!((*EventEntry).Event.__bindgen_anon_1.TargetDevice.DeviceIds) as *mut u16;
    let Status = RtlAppendUnicodeStringToString(&mut Copy, DeviceIds);
    if Status < 0 {
        ExFreePool(EventEntry as PVOID);
        return Status;
    }

    InsertHeadList(
        &raw mut IopPnpEventQueueHead,
        core::ptr::addr_of_mut!((*EventEntry).ListEntry),
    );
    
    KeSetEvent(core::ptr::addr_of_mut!(IopPnpNotifyEvent) as PVOID, 0i32, 0u8);

    STATUS_SUCCESS
}

unsafe extern "C" fn IopFindDeviceInstanceTraverse(
    DeviceNode: PDEVICE_NODE,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceInstanceContext = Context as PIOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT;

    if RtlEqualUnicodeString(
        core::ptr::addr_of!((*DeviceNode).InstancePath) as PUNICODE_STRING,
        (*DeviceInstanceContext).InstancePath,
        1u8,
    ) != 0
    {
        ObReferenceObject((*DeviceNode).PhysicalDeviceObject as PVOID);
        (*DeviceInstanceContext).DeviceObject = (*DeviceNode).PhysicalDeviceObject;
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopGetDeviceObjectFromDeviceInstance(
    DeviceInstance: PUNICODE_STRING,
) -> PDEVICE_OBJECT {
    let DeviceInstance = opaque_ptr(DeviceInstance);
    let mut Context: _DEVICETREE_TRAVERSE_CONTEXT = core::mem::zeroed();
    let mut DeviceInstanceContext: IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT = core::mem::zeroed();

    if IopRootDeviceNode.is_null() {
        return core::ptr::null_mut();
    }

    if DeviceInstance.is_null() || (*DeviceInstance).Length == 0 {
        if !(*IopRootDeviceNode).PhysicalDeviceObject.is_null() {
            ObReferenceObject((*IopRootDeviceNode).PhysicalDeviceObject as PVOID);
            return (*IopRootDeviceNode).PhysicalDeviceObject;
        } else {
            return core::ptr::null_mut();
        }
    }

    DeviceInstanceContext.InstancePath = DeviceInstance;
    DeviceInstanceContext.DeviceObject = core::ptr::null_mut();
    
    IopInitDeviceTreeTraverseContext(
        &mut Context as *mut _,
        IopRootDeviceNode,
        Some(IopFindDeviceInstanceTraverse),
        &mut DeviceInstanceContext as *mut _ as PVOID,
    );

    
    let _ = IopTraverseDeviceTree(&mut Context);

    DeviceInstanceContext.DeviceObject
}

unsafe fn IopCaptureUnicodeString(
    DstName: PUNICODE_STRING,
    SrcName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    Name.Buffer = core::ptr::null_mut();
    
    let result = probe_read_user::<_UNICODE_STRING>(SrcName, KeGetPreviousMode());
    let src_name = match result {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    Name.Length = src_name.Length;
    Name.MaximumLength = src_name.MaximumLength;
    
    if Name.Length > Name.MaximumLength {
        return STATUS_INVALID_PARAMETER;
    }

    if Name.MaximumLength != 0 {
        if let Err(e) = probe_user_buffer(
            src_name.Buffer as *const core::ffi::c_void,
            Name.MaximumLength as usize,
            core::mem::size_of::<u16>(),
            KeGetPreviousMode(),
        ) {
            return e;
        }
        
        Name.Buffer = ExAllocatePool(NonPagedPool, Name.MaximumLength as _) as *mut u16;
        if Name.Buffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        let copy_result = safe_copy_from_user(
            Name.Buffer as *mut core::ffi::c_void,
            src_name.Buffer as *const core::ffi::c_void,
            Name.MaximumLength as usize,
            1,
            KeGetPreviousMode(),
        );
        
        if let Err(e) = copy_result {
            ExFreePool(Name.Buffer as PVOID);
            return e;
        }
    }

    *DstName = Name;
    STATUS_SUCCESS
}

unsafe fn PiControlInitializeDevice(
    ControlData: PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
) -> NTSTATUS {
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DeviceNode: PDEVICE_NODE;
    let mut Status = STATUS_SUCCESS;
    let mut InstanceKey: HANDLE = core::ptr::null_mut();

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*ControlData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceObject.is_null() {
        ObDereferenceObject(DeviceObject as PVOID);
        Status = STATUS_SUCCESS;
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    Status = PnpRootCreateDeviceObject(&mut DeviceObject);
    if Status < 0 {
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    DeviceNode = PipAllocateDeviceNode(DeviceObject);
    if DeviceNode.is_null() {
        IoDeleteDevice(DeviceObject);
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    Status = RtlDuplicateUnicodeString(
        RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath) as PUNICODE_STRING,
    );
    
    if Status < 0 {
        IopFreeDeviceNode(DeviceNode);
        IoDeleteDevice(DeviceObject);
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    (*DeviceObject).Flags |= DO_BUS_ENUMERATED_DEVICE;
    (*DeviceNode).Flags |= DNF_MADEUP | DNF_IDS_QUERIED | DNF_ENUMERATED;
    PiSetDevNodeState(DeviceNode, DeviceNodeInitialized);

    Status = IopCreateDeviceKeyPath(
        &mut DeviceInstance,
        REG_OPTION_NON_VOLATILE,
        &mut InstanceKey,
    );
    
    if Status < 0 {
        IopFreeDeviceNode(DeviceNode);
        IoDeleteDevice(DeviceObject);
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    IopSetDeviceInstanceData(InstanceKey, DeviceNode);

    PnpRootRegisterDevice(DeviceObject);

    PiInsertDevNode(DeviceNode, IopRootDeviceNode);

    let _ = IopQueueDeviceInstallEvent(
        &GUID_DEVICE_ENUMERATED,
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath) as PUNICODE_STRING,
    );

    ZwClose(InstanceKey);
    
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }

    Status
}

unsafe fn IopRemovePlugPlayEvent(
    ResponseData: PPLUGPLAY_CONTROL_USER_RESPONSE_DATA,
) -> NTSTATUS {
    if !IsListEmpty(&raw mut IopPnpEventQueueHead) {
        let entry = RemoveTailList(&raw mut IopPnpEventQueueHead);
        let event_entry = (entry as usize - core::mem::offset_of!(PNP_EVENT_ENTRY, ListEntry)) as PPNP_EVENT_ENTRY;
        ExFreePool(event_entry as PVOID);
    }

    if !IsListEmpty(&raw mut IopPnpEventQueueHead) {
        KeSetEvent(core::ptr::addr_of_mut!(IopPnpNotifyEvent) as PVOID, 0i32, 0u8);
    }

    STATUS_SUCCESS
}

unsafe fn IopGetInterfaceDeviceList(
    DeviceList: PPLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut StackList: _PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA = core::mem::zeroed();
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut FilterGuid: GUID = core::mem::zeroed();
    let mut SymbolicLinkList: PZZWSTR = core::ptr::null_mut();
    let mut LinkList: PZZWSTR;
    let mut TotalLength: usize;

    let result = probe_read_user::<_PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA>(
        DeviceList as *const _,
        KeGetPreviousMode(),
    );
    StackList = match result {
        Ok(s) => s,
        Err(e) => return e,
    };

    if let Err(e) = probe_user_buffer(
        StackList.FilterGuid as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
        1,
        KeGetPreviousMode(),
    ) {
        return e;
    }
    
    if let Err(e) = safe_copy_from_user(
        &mut FilterGuid as *mut GUID as *mut core::ffi::c_void,
        StackList.FilterGuid as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
        1,
        KeGetPreviousMode(),
    ) {
        return e;
    }

    if !StackList.Buffer.is_null() && StackList.BufferSize != 0 {
        if let Err(e) = probe_user_buffer_write(
            StackList.Buffer as *const core::ffi::c_void,
            StackList.BufferSize as usize,
            1,
            KeGetPreviousMode(),
        ) {
            return e;
        }
    }

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!(StackList.DeviceInstance) as PUNICODE_STRING,
    );
    
    if Status >= 0 {
        DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
    }

    Status = IoGetDeviceInterfaces(
        &FilterGuid,
        DeviceObject,
        StackList.Flags,
        &mut SymbolicLinkList,
    );
    
    if !DeviceObject.is_null() {
        ObDereferenceObject(DeviceObject as PVOID);
    }

    if Status < 0 {
        return Status;
    }

    LinkList = SymbolicLinkList;
    while *SymbolicLinkList != 0 {
        SymbolicLinkList = SymbolicLinkList.add(wcslen(SymbolicLinkList) + 1);
    }
    TotalLength = ((SymbolicLinkList as usize - LinkList as usize) / core::mem::size_of::<u16>() + 1) * core::mem::size_of::<u16>();

    if !StackList.Buffer.is_null() && StackList.BufferSize as usize >= TotalLength {
        if let Err(e) = safe_copy_to_user(
            StackList.Buffer as *mut core::ffi::c_void,
            LinkList as *const core::ffi::c_void,
            TotalLength,
            1,
            KeGetPreviousMode(),
        ) {
            ExFreePool(LinkList as PVOID);
            return e;
        }
    }

    if let Err(e) = write_user(
        core::ptr::addr_of_mut!((*DeviceList).BufferSize),
        TotalLength as ULONG,
        KeGetPreviousMode(),
    ) {
        ExFreePool(LinkList as PVOID);
        return e;
    }

    ExFreePool(LinkList as PVOID);
    STATUS_SUCCESS
}

unsafe fn IopGetDeviceProperty(
    PropertyData: PPLUGPLAY_CONTROL_PROPERTY_DATA,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceNode: PDEVICE_NODE;
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut BufferSize: ULONG;
    let mut Property: ULONG;
    let mut DeviceProperty: DEVICE_REGISTRY_PROPERTY = 0;
    let mut Buffer: PVOID;
    let mut Status: NTSTATUS;

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*PropertyData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    let prop_data = match probe_read_user::<_PLUGPLAY_CONTROL_PROPERTY_DATA>(
        PropertyData,
        KeGetPreviousMode(),
    ) {
        Ok(p) => p,
        Err(e) => {
            if !DeviceInstance.Buffer.is_null() {
                ExFreePool(DeviceInstance.Buffer as PVOID);
            }
            return e;
        }
    };
    
    Property = prop_data.Property;
    BufferSize = prop_data.BufferSize;
    
    if let Err(e) = probe_user_buffer_write(
        prop_data.Buffer as *const core::ffi::c_void,
        BufferSize as usize,
        1,
        KeGetPreviousMode(),
    ) {
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return e;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Buffer = ExAllocatePool(NonPagedPool, BufferSize as _);
    if Buffer.is_null() {
        ObDereferenceObject(DeviceObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DeviceNode = IopGetDeviceNode(DeviceObject);

    if Property == PNP_PROPERTY_POWER_DATA {
        if BufferSize < core::mem::size_of::<CM_Power_Data_s>() as ULONG {
            BufferSize = 0;
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            let mut DeviceCapabilities: _DEVICE_CAPABILITIES = core::mem::zeroed();
            let PowerData: PCM_POWER_DATA = Buffer as PCM_POWER_DATA;
            let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
            let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

            core::ptr::write_bytes(PowerData as *mut u8, 0, core::mem::size_of::<CM_Power_Data_s>());
            (*PowerData).PD_Size = core::mem::size_of::<CM_Power_Data_s>() as ULONG;

            core::ptr::write_bytes(
                &mut DeviceCapabilities as *mut _ as *mut u8,
                0,
                core::mem::size_of::<_DEVICE_CAPABILITIES>(),
            );
            DeviceCapabilities.Size = core::mem::size_of::<_DEVICE_CAPABILITIES>() as USHORT;
            DeviceCapabilities.Version = 1;
            DeviceCapabilities.Address = -1i32 as ULONG;
            DeviceCapabilities.UINumber = -1i32 as ULONG;

            Stack.Parameters.DeviceCapabilities.Capabilities = &mut DeviceCapabilities;

            Status = IopInitiatePnpIrp(
                DeviceObject,
                &mut IoStatusBlock,
                IRP_MN_QUERY_CAPABILITIES,
                &mut Stack,
            );
            
            if Status >= 0 {
                (*PowerData).PD_MostRecentPowerState = PowerDeviceD0;
                // Named getters: the raw `_bitfield_1.get(n, 1)` calls read bits 2-5 (Lock/Eject/
                // Removable/Dock) as WakeFromD0-D3 and bit 12 (WakeFromD2) as WarmEjectSupported.
                if DeviceCapabilities.DeviceD1() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_D1_SUPPORTED;
                }
                if DeviceCapabilities.DeviceD2() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_D2_SUPPORTED;
                }
                if DeviceCapabilities.WakeFromD0() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_WAKE_FROM_D0_SUPPORTED;
                }
                if DeviceCapabilities.WakeFromD1() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_WAKE_FROM_D1_SUPPORTED;
                }
                if DeviceCapabilities.WakeFromD2() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_WAKE_FROM_D2_SUPPORTED;
                }
                if DeviceCapabilities.WakeFromD3() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_WAKE_FROM_D3_SUPPORTED;
                }
                if DeviceCapabilities.WarmEjectSupported() != 0 {
                    (*PowerData).PD_Capabilities |= PDCAP_WARM_EJECT_SUPPORTED;
                }
                (*PowerData).PD_D1Latency = DeviceCapabilities.D1Latency;
                (*PowerData).PD_D2Latency = DeviceCapabilities.D2Latency;
                (*PowerData).PD_D3Latency = DeviceCapabilities.D3Latency;
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!(DeviceCapabilities.DeviceState) as *const u8,
                    core::ptr::addr_of_mut!((*PowerData).PD_PowerStateMapping) as *mut u8,
                    core::mem::size_of_val(&DeviceCapabilities.DeviceState),
                );
                (*PowerData).PD_DeepestSystemWake = DeviceCapabilities.SystemWake;
            } else {
                (*PowerData).PD_Capabilities = PDCAP_D0_SUPPORTED | PDCAP_D3_SUPPORTED;
                (*PowerData).PD_MostRecentPowerState = PowerDeviceD0;
            }
        }
    } else if Property == PNP_PROPERTY_REMOVAL_POLICY_OVERRIDE {
        BufferSize = 0;
        Status = STATUS_NOT_IMPLEMENTED;
    } else if Property == PNP_PROPERTY_REMOVAL_POLICY_HARDWARE_DEFAULT {
        if BufferSize < core::mem::size_of_val(&(*DeviceNode).HardwareRemovalPolicy) as ULONG {
            BufferSize = 0;
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            BufferSize = core::mem::size_of_val(&(*DeviceNode).HardwareRemovalPolicy) as ULONG;
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*DeviceNode).HardwareRemovalPolicy) as *const u8,
                Buffer as *mut u8,
                BufferSize as usize,
            );
        }
    } else if Property == PNP_PROPERTY_LOCATION_PATHS {
        BufferSize = 0;
        Status = STATUS_NOT_IMPLEMENTED;
    } else {
        match Property {
            PNP_PROPERTY_UI_NUMBER => DeviceProperty = DevicePropertyUINumber,
            PNP_PROPERTY_PHYSICAL_DEVICE_OBJECT_NAME => DeviceProperty = DevicePropertyPhysicalDeviceObjectName,
            PNP_PROPERTY_BUSTYPEGUID => DeviceProperty = DevicePropertyBusTypeGuid,
            PNP_PROPERTY_LEGACYBUSTYPE => DeviceProperty = DevicePropertyLegacyBusType,
            PNP_PROPERTY_BUSNUMBER => DeviceProperty = DevicePropertyBusNumber,
            PNP_PROPERTY_REMOVAL_POLICY => DeviceProperty = DevicePropertyRemovalPolicy,
            PNP_PROPERTY_ADDRESS => DeviceProperty = DevicePropertyAddress,
            PNP_PROPERTY_ENUMERATOR_NAME => DeviceProperty = DevicePropertyEnumeratorName,
            PNP_PROPERTY_INSTALL_STATE => DeviceProperty = DevicePropertyInstallState,
            PNP_PROPERTY_CONTAINERID => DeviceProperty = DevicePropertyContainerID,
            _ => {
                BufferSize = 0;
                Status = STATUS_INVALID_PARAMETER;
            }
        }

        if Status == STATUS_SUCCESS {
            Status = IoGetDeviceProperty(
                DeviceObject,
                DeviceProperty,
                BufferSize,
                Buffer,
                &mut BufferSize,
            );
        }
    }

    ObDereferenceObject(DeviceObject as PVOID);

    if Status >= 0 {
        if let Err(e) = safe_copy_to_user(
            prop_data.Buffer as *mut core::ffi::c_void, // the captured copy, not a raw re-read
            Buffer,
            BufferSize as usize,
            1,
            KeGetPreviousMode(),
        ) {
            Status = e;
        } else if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*PropertyData).BufferSize),
            BufferSize,
            KeGetPreviousMode(),
        ) {
            Status = e;
        }
    }

    ExFreePool(Buffer);
    Status
}

unsafe fn IopGetRelatedDevice(
    RelatedDeviceData: PPLUGPLAY_CONTROL_RELATED_DEVICE_DATA,
) -> NTSTATUS {
    let mut RootDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceNode: PDEVICE_NODE = core::ptr::null_mut();
    let mut RelatedDeviceNode: PDEVICE_NODE;
    let mut TargetDeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut Status = STATUS_SUCCESS;
    let mut Relation: ULONG = 0;
    let mut MaximumLength: ULONG = 0;

    Status = IopCaptureUnicodeString(
        &mut TargetDeviceInstance,
        core::ptr::addr_of_mut!((*RelatedDeviceData).TargetDeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    let rel_data = match probe_read_user::<_PLUGPLAY_CONTROL_RELATED_DEVICE_DATA>(
        RelatedDeviceData,
        KeGetPreviousMode(),
    ) {
        Ok(r) => r,
        Err(e) => {
            if !TargetDeviceInstance.Buffer.is_null() {
                ExFreePool(TargetDeviceInstance.Buffer as PVOID);
            }
            return e;
        }
    };
    
    Relation = rel_data.Relation;
    MaximumLength = rel_data.RelatedDeviceInstanceLength;
    
    if let Err(e) = probe_user_buffer_write(
        rel_data.RelatedDeviceInstance as *const core::ffi::c_void,
        MaximumLength as usize,
        core::mem::size_of::<u16>(),
        KeGetPreviousMode(),
    ) {
        if !TargetDeviceInstance.Buffer.is_null() {
            ExFreePool(TargetDeviceInstance.Buffer as PVOID);
        }
        return e;
    }

    RtlInitUnicodeString(&mut RootDeviceName, w!("HTREE\\ROOT\\0").as_ptr() as *const u16);
    
    if RtlEqualUnicodeString(&mut TargetDeviceInstance, &mut RootDeviceName, 1u8) != 0 {
        DeviceNode = IopRootDeviceNode;
        if !TargetDeviceInstance.Buffer.is_null() {
            ExFreePool(TargetDeviceInstance.Buffer as PVOID);
        }
    } else {
        DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut TargetDeviceInstance);
        if !TargetDeviceInstance.Buffer.is_null() {
            ExFreePool(TargetDeviceInstance.Buffer as PVOID);
        }
        if DeviceObject.is_null() {
            return STATUS_NO_SUCH_DEVICE;
        }

        DeviceNode = IopGetDeviceNode(DeviceObject);
    }

    match Relation {
        PNP_GET_PARENT_DEVICE => RelatedDeviceNode = (*DeviceNode).Parent,
        PNP_GET_CHILD_DEVICE => RelatedDeviceNode = (*DeviceNode).Child,
        PNP_GET_SIBLING_DEVICE => RelatedDeviceNode = (*DeviceNode).Sibling,
        _ => {
            if !DeviceObject.is_null() {
                ObDereferenceObject(DeviceObject as PVOID);
            }
            return STATUS_INVALID_PARAMETER;
        }
    }

    if RelatedDeviceNode.is_null() {
        if !DeviceObject.is_null() {
            ObDereferenceObject(DeviceObject as PVOID);
        }
        return STATUS_NO_SUCH_DEVICE;
    }

    if (*RelatedDeviceNode).InstancePath.Length as ULONG > MaximumLength {
        if !DeviceObject.is_null() {
            ObDereferenceObject(DeviceObject as PVOID);
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    if let Err(e) = safe_copy_to_user(
        rel_data.RelatedDeviceInstance as *mut core::ffi::c_void, // the captured copy, not a raw re-read
        (*RelatedDeviceNode).InstancePath.Buffer as *const core::ffi::c_void,
        (*RelatedDeviceNode).InstancePath.Length as usize,
        1,
        KeGetPreviousMode(),
    ) {
        Status = e;
    } else if let Err(e) = write_user(
        core::ptr::addr_of_mut!((*RelatedDeviceData).RelatedDeviceInstanceLength),
        (*RelatedDeviceNode).InstancePath.Length as ULONG,
        KeGetPreviousMode(),
    ) {
        Status = e;
    }

    if !DeviceObject.is_null() {
        ObDereferenceObject(DeviceObject as PVOID);
    }

    Status
}

unsafe fn PiIsDevNodeStarted(DeviceNode: PDEVICE_NODE) -> BOOLEAN {
    if (*DeviceNode).State == DeviceNodeStartPending
        || (*DeviceNode).State == DeviceNodeStartCompletion
        || (*DeviceNode).State == DeviceNodeStartPostWork
        || (*DeviceNode).State == DeviceNodeStarted
        || (*DeviceNode).State == DeviceNodeQueryStopped
        || (*DeviceNode).State == DeviceNodeEnumeratePending
        || (*DeviceNode).State == DeviceNodeEnumerateCompletion
        || (*DeviceNode).State == DeviceNodeStopped
        || (*DeviceNode).State == DeviceNodeRestartCompletion
    {
        1u8
    } else {
        0u8
    }
}

// Not static since upstream 3cc9aa99292: the property store (pnpprop) reports DEVPKEY_Device_DevNodeStatus.
#[no_mangle]
pub unsafe extern "C" fn IopGetDeviceNodeStatus(DeviceNode: PDEVICE_NODE) -> ULONG {
    let mut Output = DN_NT_ENUMERATOR | DN_NT_DRIVER;

    if (*DeviceNode).Parent == IopRootDeviceNode {
        Output |= DN_ROOT_ENUMERATED;
    }

    if (*DeviceNode).State >= DeviceNodeDriversAdded {
        Output |= DN_DRIVER_LOADED;
    }

    if PiIsDevNodeStarted(DeviceNode) != 0 {
        Output |= DN_STARTED;
    }

    if (*DeviceNode).UserFlags & DNUF_WILL_BE_REMOVED != 0 {
        Output |= DN_WILL_BE_REMOVED;
    }

    if (*DeviceNode).Flags & DNF_HAS_PROBLEM != 0 {
        Output |= DN_HAS_PROBLEM;
    }

    if (*DeviceNode).Flags & DNF_HAS_PRIVATE_PROBLEM != 0 {
        Output |= DN_PRIVATE_PROBLEM;
    }

    if (*DeviceNode).Flags & DNF_DRIVER_BLOCKED != 0 {
        Output |= DN_DRIVER_BLOCKED;
    }

    if (*DeviceNode).Flags & DNF_CHILD_WITH_INVALID_ID != 0 {
        Output |= DN_CHILD_WITH_INVALID_ID;
    }

    if (*DeviceNode).Flags & DNF_HAS_PRIVATE_PROBLEM != 0 {
        Output |= DN_PRIVATE_PROBLEM;
    }

    if (*DeviceNode).Flags & DNF_LEGACY_DRIVER != 0 {
        Output |= DN_LEGACY_DRIVER;
    }

    if (*DeviceNode).UserFlags & DNUF_DONT_SHOW_IN_UI != 0 {
        Output |= DN_NO_SHOW_IN_DM;
    }

    if (*DeviceNode).UserFlags & DNUF_NOT_DISABLEABLE == 0 {
        Output |= DN_DISABLEABLE;
    }

    Output
}

unsafe fn IopDeviceStatus(
    StatusData: PPLUGPLAY_CONTROL_STATUS_DATA,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DeviceNode: PDEVICE_NODE;
    let mut Operation: ULONG = 0;
    let mut DeviceStatus: ULONG = 0;
    let mut DeviceProblem: ULONG = 0;
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*StatusData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    let stat_data = match probe_read_user::<_PLUGPLAY_CONTOL_STATUS_DATA>(
        StatusData,
        KeGetPreviousMode(),
    ) {
        Ok(s) => s,
        Err(e) => {
            if !DeviceInstance.Buffer.is_null() {
                ExFreePool(DeviceInstance.Buffer as PVOID);
            }
            return e;
        }
    };
    
    Operation = stat_data.Operation;
    if Operation == PNP_SET_DEVICE_STATUS {
        DeviceStatus = stat_data.DeviceStatus;
        DeviceProblem = stat_data.DeviceProblem;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    DeviceNode = IopGetDeviceNode(DeviceObject);

    match Operation {
        PNP_GET_DEVICE_STATUS => {
            DeviceStatus = IopGetDeviceNodeStatus(DeviceNode);
            DeviceProblem = (*DeviceNode).Problem;
        }
        PNP_SET_DEVICE_STATUS => {}
        PNP_CLEAR_DEVICE_STATUS => {}
        _ => {}
    }

    ObDereferenceObject(DeviceObject as PVOID);

    if Operation == PNP_GET_DEVICE_STATUS {
        if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*StatusData).DeviceStatus),
            DeviceStatus,
            KeGetPreviousMode(),
        ) {
            Status = e;
        } else if let Err(e) = write_user(
            core::ptr::addr_of_mut!((*StatusData).DeviceProblem),
            DeviceProblem,
            KeGetPreviousMode(),
        ) {
            Status = e;
        }
    }

    Status
}

unsafe fn IopGetDeviceRelations(
    RelationsData: PPLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA,
) -> NTSTATUS {
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Stack: _IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut ChildDeviceObject: PDEVICE_OBJECT;
    let mut ChildDeviceNode: PDEVICE_NODE;
    let mut i: ULONG;
    let mut Relations: ULONG;
    let mut BufferSize: ULONG;
    let mut RequiredSize: ULONG;
    let mut BufferLeft: ULONG;
    let mut Buffer: *mut u16;
    let mut Ptr: *mut u16;
    let mut Status = STATUS_SUCCESS;

    let rel_data = match probe_read_user::<_PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA>(
        RelationsData,
        KeGetPreviousMode(),
    ) {
        Ok(r) => r,
        Err(e) => return e,
    };
    
    Relations = rel_data.Relations;
    BufferSize = rel_data.BufferSize;
    Buffer = rel_data.Buffer;

    if let Err(e) = probe_user_buffer_write(
        Buffer as *const core::ffi::c_void,
        BufferSize as usize,
        1,
        KeGetPreviousMode(),
    ) {
        return e;
    }

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*RelationsData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if DeviceObject.is_null() {
        Status = STATUS_NO_SUCH_DEVICE;
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    match Relations {
        PNP_EJECT_RELATIONS => Stack.Parameters.QueryDeviceRelations.Type = EjectionRelations as u32,
        PNP_REMOVAL_RELATIONS => Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations as u32,
        PNP_POWER_RELATIONS => Stack.Parameters.QueryDeviceRelations.Type = PowerRelations as u32,
        PNP_BUS_RELATIONS => Stack.Parameters.QueryDeviceRelations.Type = BusRelations as u32,
        _ => {
            Status = STATUS_INVALID_PARAMETER;
            if !DeviceRelations.is_null() {
                ExFreePool(DeviceRelations as PVOID);
            }
            if !DeviceObject.is_null() {
                ObDereferenceObject(DeviceObject as PVOID);
            }
            if !DeviceInstance.Buffer.is_null() {
                ExFreePool(DeviceInstance.Buffer as PVOID);
            }
            return Status;
        }
    }

    Status = IopInitiatePnpIrp(
        DeviceObject,
        &mut IoStatusBlock,
        IRP_MN_QUERY_DEVICE_RELATIONS,
        &mut Stack,
    );
    
    if Status < 0 {
        if !DeviceRelations.is_null() {
            ExFreePool(DeviceRelations as PVOID);
        }
        if !DeviceObject.is_null() {
            ObDereferenceObject(DeviceObject as PVOID);
        }
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        return Status;
    }

    DeviceRelations = IoStatusBlock.Information as PDEVICE_RELATIONS;

    RequiredSize = 0;
    BufferLeft = BufferSize;
    Ptr = Buffer;

    i = 0;
    while i < (*DeviceRelations).Count {
        ChildDeviceObject = *(*DeviceRelations).Objects.as_ptr().add(i as usize);

        ChildDeviceNode = IopGetDeviceNode(ChildDeviceObject);
        if !ChildDeviceNode.is_null() {
            if !Ptr.is_null() {
                if BufferLeft < ((*ChildDeviceNode).InstancePath.Length as ULONG + 2 * core::mem::size_of::<u16>() as ULONG) {
                    Status = STATUS_BUFFER_TOO_SMALL;
                    break;
                }

                if let Err(e) = safe_copy_to_user(
                    Ptr as *mut core::ffi::c_void,
                    (*ChildDeviceNode).InstancePath.Buffer as *const core::ffi::c_void,
                    (*ChildDeviceNode).InstancePath.Length as usize,
                    1,
                    KeGetPreviousMode(),
                ) {
                    Status = e;
                    break;
                }
                
                Ptr = Ptr.add(((*ChildDeviceNode).InstancePath.Length / core::mem::size_of::<u16>() as u16) as usize);
                if let Err(e) = write_user(Ptr, 0u16, KeGetPreviousMode()) {
                    Status = e;
                    break;
                }
                Ptr = Ptr.add(1);

                BufferLeft -= (*ChildDeviceNode).InstancePath.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
            }

            RequiredSize += (*ChildDeviceNode).InstancePath.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
        }
        
        i += 1;
    }

    if !Ptr.is_null() && BufferLeft >= core::mem::size_of::<u16>() as ULONG {
        let _ = write_user(Ptr, 0u16, KeGetPreviousMode());
    }

    if RequiredSize > 0 {
        RequiredSize += core::mem::size_of::<u16>() as ULONG;
    }

    if let Err(e) = write_user(
        core::ptr::addr_of_mut!((*RelationsData).BufferSize),
        RequiredSize,
        KeGetPreviousMode(),
    ) {
        Status = e;
    }

    if !DeviceRelations.is_null() {
        ExFreePool(DeviceRelations as PVOID);
    }
    if !DeviceObject.is_null() {
        ObDereferenceObject(DeviceObject as PVOID);
    }
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }

    Status
}

unsafe fn IopGetDeviceDepth(
    DepthData: PPLUGPLAY_CONTROL_DEPTH_DATA,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DeviceNode: PDEVICE_NODE;
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
    let mut Status = STATUS_SUCCESS;

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*DepthData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    DeviceNode = IopGetDeviceNode(DeviceObject);

    if let Err(e) = write_user(
        core::ptr::addr_of_mut!((*DepthData).Depth),
        (*DeviceNode).Level,
        KeGetPreviousMode(),
    ) {
        Status = e;
    }

    ObDereferenceObject(DeviceObject as PVOID);

    Status
}

unsafe fn PiControlSyncDeviceAction(
    DeviceData: PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
    ControlClass: ULONG,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Status: NTSTATUS;
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*DeviceData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    let Action = match ControlClass {
        PlugPlayControlEnumerateDevice => PiActionEnumDeviceTree,
        PlugPlayControlStartDevice => PiActionStartDevice,
        PlugPlayControlResetDevice => PiActionResetDevice,
        _ => 0,
    };

    Status = PiPerformSyncDeviceAction(DeviceObject, Action);

    ObDereferenceObject(DeviceObject as PVOID);

    Status
}

unsafe fn PiControlQueryRemoveDevice(
    ControlData: PPLUGPLAY_CONTROL_QUERY_REMOVE_DATA,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT;
    let mut Status: NTSTATUS;
    let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();

    Status = IopCaptureUnicodeString(
        &mut DeviceInstance,
        core::ptr::addr_of_mut!((*ControlData).DeviceInstance) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
    if !DeviceInstance.Buffer.is_null() {
        ExFreePool(DeviceInstance.Buffer as PVOID);
    }
    if DeviceObject.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Status = STATUS_NOT_IMPLEMENTED;

    ObDereferenceObject(DeviceObject as PVOID);

    Status
}

unsafe fn PiControlGetInterfaceDeviceAlias(
    ControlData: PPLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA,
) -> NTSTATUS {
    // ControlData is the caller's buffer and the C hands its string and GUID
    // straight to IoGetDeviceInterfaceAlias, which reads both with no __try.
    // Capture them, as the other PiControl helpers here do.
    let mode = KeGetPreviousMode();
    let cd = match probe_read_user::<_PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA>(ControlData, mode) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut guid: GUID = core::mem::zeroed();
    if !cd.AliasInterfaceClassGuid.is_null() {
        guid = match probe_read_user::<GUID>(cd.AliasInterfaceClassGuid, mode) {
            Ok(v) => v,
            Err(e) => return e,
        };
    }
    let mut SymbolicLinkName: _UNICODE_STRING = core::mem::zeroed();
    let Status = IopCaptureUnicodeString(
        &mut SymbolicLinkName,
        core::ptr::addr_of_mut!((*ControlData).SymbolicLinkName) as PUNICODE_STRING,
    );
    if Status < 0 {
        return Status;
    }

    // IoGetDeviceInterfaceAlias ignores the caller's buffer entirely and overwrites
    // the whole UNICODE_STRING with a fresh pool copy of the alias (deviface.rs,
    // `*AliasSymbolicLinkName = AliasSymbolicLink`), so pass it empty.
    let mut AliasSymbolicLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status = IoGetDeviceInterfaceAlias(
        &mut SymbolicLinkName as *mut _ as PUNICODE_STRING,
        if cd.AliasInterfaceClassGuid.is_null() { core::ptr::null() } else { &guid },
        &mut AliasSymbolicLinkName,
    );
    // The alias comes back in IoGetDeviceInterfaceAlias's own pool copy, NOT in the
    // buffer the caller passed. Copy it back and report the length — the step the
    // upstream C omits, which is why it allocates the alias and then leaks it without
    // ever returning it. AliasSymbolicLinkNameLength is in WCHARs (umpnpmgr's
    // PNP_GetInterfaceDeviceAlias sets size_is()-counted *pulLength from it and the
    // transfer length to that + 1 for the terminator): in = caller capacity, out =
    // alias length EXCLUDING the NUL. The user stores go through the fault-tolerant
    // primitives, as every other PiControl write-back in this file does.
    if Status >= 0 {
        let wchar = core::mem::size_of::<u16>();
        let alias_chars = AliasSymbolicLinkName.Length as usize / wchar;
        let cap_bytes = cd.AliasSymbolicLinkNameLength as usize * wchar;
        if AliasSymbolicLinkName.Length as usize + wchar <= cap_bytes {
            let out = cd.AliasSymbolicLinkName;
            if let Err(e) = safe_copy_to_user(
                out as *mut core::ffi::c_void,
                AliasSymbolicLinkName.Buffer as *const core::ffi::c_void,
                AliasSymbolicLinkName.Length as usize,
                1,
                mode,
            ) {
                Status = e;
            } else if let Err(e) = write_user(out.add(alias_chars), 0u16, mode) {
                Status = e;
            } else if let Err(e) = write_user(
                core::ptr::addr_of_mut!((*ControlData).AliasSymbolicLinkNameLength),
                alias_chars as ULONG,
                mode,
            ) {
                Status = e;
            }
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }
    }
    // The caller's buffer was never handed to IoGetDeviceInterfaceAlias, so its pool
    // copy never aliases it: free it unconditionally.
    if !AliasSymbolicLinkName.Buffer.is_null() {
        ExFreePool(AliasSymbolicLinkName.Buffer as PVOID);
    }
    if !SymbolicLinkName.Buffer.is_null() {
        ExFreePool(SymbolicLinkName.Buffer as PVOID);
    }
    Status
}

unsafe fn PiControlDeviceClassAssociation(
    ControlData: PPLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut SymbolicLink: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    // ControlData is the caller's buffer, and the C reads and writes it in place
    // with no __try: its strings and GUID go straight into IoRegisterDeviceInterface.
    // Work from captured copies and store results back fault-tolerantly.
    let mode = KeGetPreviousMode();
    let cd = match probe_read_user::<_PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA>(ControlData, mode) {
        Ok(v) => v,
        Err(e) => return e,
    };

    if cd.Register != 0 {
        if cd.DeviceInstance.Buffer.is_null() || cd.DeviceInstance.Length == 0 {
            return STATUS_INVALID_PARAMETER;
        }

        if cd.InterfaceGuid.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        let guid = match probe_read_user::<GUID>(cd.InterfaceGuid, mode) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let mut DeviceInstance: _UNICODE_STRING = core::mem::zeroed();
        Status = IopCaptureUnicodeString(
            &mut DeviceInstance,
            core::ptr::addr_of_mut!((*ControlData).DeviceInstance) as PUNICODE_STRING,
        );
        if Status < 0 {
            return Status;
        }
        let mut Reference: _UNICODE_STRING = core::mem::zeroed();
        Status = IopCaptureUnicodeString(
            &mut Reference,
            core::ptr::addr_of_mut!((*ControlData).Reference) as PUNICODE_STRING,
        );
        if Status < 0 {
            ExFreePool(DeviceInstance.Buffer as PVOID);
            return Status;
        }

        DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance as *mut _ as PUNICODE_STRING);
        if DeviceObject.is_null() {
            Status = STATUS_NO_SUCH_DEVICE;
        } else {
            Status = IoRegisterDeviceInterface(
                DeviceObject,
                &guid,
                &mut Reference as *mut _ as PUNICODE_STRING,
                &mut SymbolicLink,
            );

            if Status >= 0 {
                let needed = SymbolicLink.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
                if needed <= cd.SymbolicLinkNameLength {
                    let out = cd.SymbolicLinkName;
                    if let Err(e) = safe_copy_to_user(
                        out as *mut core::ffi::c_void,
                        SymbolicLink.Buffer as *const core::ffi::c_void,
                        SymbolicLink.Length as usize,
                        1,
                        mode,
                    ) {
                        Status = e;
                    } else if let Err(e) = write_user(
                        out.add((SymbolicLink.Length / core::mem::size_of::<u16>() as u16) as usize),
                        0u16,
                        mode,
                    ) {
                        Status = e;
                    }
                } else {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }
                // Both branches report the needed size, as the C does.
                if let Err(e) = write_user(core::ptr::addr_of_mut!((*ControlData).SymbolicLinkNameLength), needed, mode) {
                    Status = e;
                }

                ExFreePool(SymbolicLink.Buffer as PVOID);
            }

            ObDereferenceObject(DeviceObject as PVOID);
        }
        if !DeviceInstance.Buffer.is_null() {
            ExFreePool(DeviceInstance.Buffer as PVOID);
        }
        if !Reference.Buffer.is_null() {
            ExFreePool(Reference.Buffer as PVOID);
        }
    } else {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtGetPlugPlayEvent(
    Reserved1: ULONG,
    Reserved2: ULONG,
    Buffer: PPLUGPLAY_EVENT_BLOCK,
    BufferSize: ULONG,
) -> NTSTATUS {
    let mut Entry: PPNP_EVENT_ENTRY;
    let mut Status: NTSTATUS;

    if KeGetPreviousMode() == 0i8 {
        return STATUS_ACCESS_DENIED;
    }

    if SeSinglePrivilegeCheck(SeTcbPrivilege, 1i8) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = KeWaitForSingleObject(
        &raw mut IopPnpNotifyEvent as *mut _ as PVOID,
        _KWAIT_REASON_UserRequest,
        1i8,
        0u8,
        core::ptr::null_mut(),
    );
    
    if Status < 0 || Status == STATUS_USER_APC {
        return Status;
    }

    Entry = (IopPnpEventQueueHead.Blink as usize - core::mem::offset_of!(PNP_EVENT_ENTRY, ListEntry)) as PPNP_EVENT_ENTRY;

    if BufferSize < (*Entry).Event.TotalSize {
        return STATUS_BUFFER_TOO_SMALL;
    }

    if let Err(e) = safe_copy_to_user(
        Buffer as *mut core::ffi::c_void,
        core::ptr::addr_of!((*Entry).Event) as *const core::ffi::c_void,
        (*Entry).Event.TotalSize as usize,
        1,
        KeGetPreviousMode(),
    ) {
        return e;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtPlugPlayControl(
    PlugPlayControlClass: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    if KeGetPreviousMode() == 0i8 {
        return STATUS_ACCESS_DENIED;
    }

    if SeSinglePrivilegeCheck(SeTcbPrivilege, 1i8) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    if let Err(e) = probe_user_buffer_write(
        Buffer,
        BufferLength as usize,
        core::mem::size_of::<ULONG>(),
        KeGetPreviousMode(),
    ) {
        return e;
    }

    match PlugPlayControlClass {
        PlugPlayControlEnumerateDevice => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlSyncDeviceAction(
                Buffer as PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
                PlugPlayControlClass,
            )
        }
        PlugPlayControlInitializeDevice => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlInitializeDevice(Buffer as PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA)
        }
        PlugPlayControlStartDevice | PlugPlayControlResetDevice => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlSyncDeviceAction(
                Buffer as PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA,
                PlugPlayControlClass,
            )
        }
        PlugPlayControlUnlockDevice => STATUS_NOT_IMPLEMENTED,
        PlugPlayControlQueryAndRemoveDevice => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_QUERY_REMOVE_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlQueryRemoveDevice(Buffer as PPLUGPLAY_CONTROL_QUERY_REMOVE_DATA)
        }
        PlugPlayControlUserResponse => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_USER_RESPONSE_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopRemovePlugPlayEvent(Buffer as PPLUGPLAY_CONTROL_USER_RESPONSE_DATA)
        }
        PlugPlayControlGetInterfaceDeviceList => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopGetInterfaceDeviceList(Buffer as PPLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA)
        }
        PlugPlayControlProperty => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_PROPERTY_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopGetDeviceProperty(Buffer as PPLUGPLAY_CONTROL_PROPERTY_DATA)
        }
        PlugPlayControlDeviceClassAssociation => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlDeviceClassAssociation(Buffer as PPLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA)
        }
        PlugPlayControlGetRelatedDevice => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_RELATED_DEVICE_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopGetRelatedDevice(Buffer as PPLUGPLAY_CONTROL_RELATED_DEVICE_DATA)
        }
        PlugPlayControlGetInterfaceDeviceAlias => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            PiControlGetInterfaceDeviceAlias(Buffer as PPLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA)
        }
        PlugPlayControlDeviceStatus => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTOL_STATUS_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopDeviceStatus(Buffer as PPLUGPLAY_CONTROL_STATUS_DATA)
        }
        PlugPlayControlGetDeviceDepth => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_DEPTH_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopGetDeviceDepth(Buffer as PPLUGPLAY_CONTROL_DEPTH_DATA)
        }
        PlugPlayControlQueryDeviceRelations => {
            if Buffer.is_null() || BufferLength < core::mem::size_of::<_PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            IopGetDeviceRelations(Buffer as PPLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA)
        }
        _ => STATUS_NOT_IMPLEMENTED,
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/plugplay.c
//  * PURPOSE:         Plug-and-play interface routines
//  * PROGRAMMERS:     Eric Kohl <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _PNP_EVENT_ENTRY
// {
//     LIST_ENTRY ListEntry;
//     PLUGPLAY_EVENT_BLOCK Event;
// } PNP_EVENT_ENTRY, *PPNP_EVENT_ENTRY;
//
// typedef struct _IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT
// {
//     PCUNICODE_STRING InstancePath;
//     PDEVICE_OBJECT DeviceObject;
// } IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT, *PIOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT;
//
//
// /* GLOBALS *******************************************************************/
//
// static LIST_ENTRY IopPnpEventQueueHead;
// static KEVENT IopPnpNotifyEvent;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// IopSetDeviceInstanceData(HANDLE InstanceKey, PDEVICE_NODE DeviceNode);
//
// CODE_SEG("INIT")
// NTSTATUS
// IopInitPlugPlayEvents(VOID)
// {
//     InitializeListHead(&IopPnpEventQueueHead);
//
//     KeInitializeEvent(&IopPnpNotifyEvent,
//                       SynchronizationEvent,
//                       FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IopQueueDeviceChangeEvent(
//     _In_ const GUID *EventGuid,
//     _In_ const GUID *InterfaceClassGuid,
//     _In_ PUNICODE_STRING SymbolicLinkName)
// {
//     PPNP_EVENT_ENTRY EventEntry;
//     UNICODE_STRING Copy;
//     ULONG TotalSize;
//
//     /* Allocate a big enough buffer */
//     Copy.Length = 0;
//     Copy.MaximumLength = SymbolicLinkName->Length + sizeof(UNICODE_NULL);
//     TotalSize =
//         FIELD_OFFSET(PLUGPLAY_EVENT_BLOCK, DeviceClass.SymbolicLinkName) +
//         Copy.MaximumLength;
//
//     EventEntry = ExAllocatePool(NonPagedPool,
//                                 TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//     if (!EventEntry)
//         return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(EventEntry, TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//
//     /* Fill the buffer with the event GUID */
//     RtlCopyMemory(&EventEntry->Event.EventGuid, EventGuid, sizeof(GUID));
//     EventEntry->Event.EventCategory = DeviceClassChangeEvent;
//     EventEntry->Event.TotalSize = TotalSize;
//
//     /* Fill the interface class GUID */
//     RtlCopyMemory(&EventEntry->Event.DeviceClass.ClassGuid, InterfaceClassGuid, sizeof(GUID));
//
//     /* Fill the symbolic link name */
//     RtlCopyMemory(&EventEntry->Event.DeviceClass.SymbolicLinkName,
//                   SymbolicLinkName->Buffer, SymbolicLinkName->Length);
//     EventEntry->Event.DeviceClass.SymbolicLinkName[SymbolicLinkName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     InsertHeadList(&IopPnpEventQueueHead,
//                    &EventEntry->ListEntry);
//     KeSetEvent(&IopPnpNotifyEvent,
//                0,
//                FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IopQueueDeviceInstallEvent(
//     _In_ const GUID *EventGuid,
//     _In_ PUNICODE_STRING DeviceId)
// {
//     PPNP_EVENT_ENTRY EventEntry;
//     UNICODE_STRING Copy;
//     ULONG TotalSize;
//
//     /* Allocate a big enough buffer */
//     Copy.Length = 0;
//     Copy.MaximumLength = DeviceId->Length + sizeof(UNICODE_NULL);
//     TotalSize =
//         FIELD_OFFSET(PLUGPLAY_EVENT_BLOCK, InstallDevice.DeviceId) +
//         Copy.MaximumLength;
//
//     EventEntry = ExAllocatePool(NonPagedPool,
//                                 TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//     if (!EventEntry)
//         return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(EventEntry, TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//
//     /* Fill the buffer with the event GUID */
//     RtlCopyMemory(&EventEntry->Event.EventGuid, EventGuid, sizeof(GUID));
//     EventEntry->Event.EventCategory = DeviceInstallEvent;
//     EventEntry->Event.TotalSize = TotalSize;
//
//     /* Fill the symbolic link name */
//     RtlCopyMemory(&EventEntry->Event.InstallDevice.DeviceId,
//                   DeviceId->Buffer, DeviceId->Length);
//     EventEntry->Event.InstallDevice.DeviceId[DeviceId->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     InsertHeadList(&IopPnpEventQueueHead, &EventEntry->ListEntry);
//
//     KeSetEvent(&IopPnpNotifyEvent, 0, FALSE);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// IopQueueTargetDeviceEvent(const GUID *Guid,
//                           PUNICODE_STRING DeviceIds)
// {
//     PPNP_EVENT_ENTRY EventEntry;
//     UNICODE_STRING Copy;
//     ULONG TotalSize;
//     NTSTATUS Status;
//
//     ASSERT(DeviceIds);
//
//     /* Allocate a big enough buffer */
//     Copy.Length = 0;
//     Copy.MaximumLength = DeviceIds->Length + sizeof(UNICODE_NULL);
//     TotalSize =
//         FIELD_OFFSET(PLUGPLAY_EVENT_BLOCK, TargetDevice.DeviceIds) +
//         Copy.MaximumLength;
//
//     EventEntry = ExAllocatePool(NonPagedPool,
//                                 TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//     if (!EventEntry)
//         return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(EventEntry, TotalSize + FIELD_OFFSET(PNP_EVENT_ENTRY, Event));
//
//     /* Fill the buffer with the event GUID */
//     RtlCopyMemory(&EventEntry->Event.EventGuid,
//                   Guid,
//                   sizeof(GUID));
//     EventEntry->Event.EventCategory = TargetDeviceChangeEvent;
//     EventEntry->Event.TotalSize = TotalSize;
//
//     /* Fill the device id */
//     Copy.Buffer = EventEntry->Event.TargetDevice.DeviceIds;
//     Status = RtlAppendUnicodeStringToString(&Copy, DeviceIds);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePool(EventEntry);
//         return Status;
//     }
//
//     InsertHeadList(&IopPnpEventQueueHead,
//                    &EventEntry->ListEntry);
//     KeSetEvent(&IopPnpNotifyEvent,
//                0,
//                FALSE);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IopFindDeviceInstanceTraverse(
//     _In_ PDEVICE_NODE DeviceNode,
//     _Inout_ PVOID Context)
// {
//     PIOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT DeviceInstanceContext = Context;
//
//     if (RtlEqualUnicodeString(&DeviceNode->InstancePath,
//                               DeviceInstanceContext->InstancePath, TRUE))
//     {
//         ObReferenceObject(DeviceNode->PhysicalDeviceObject);
//         DeviceInstanceContext->DeviceObject = DeviceNode->PhysicalDeviceObject;
//
//         /* Stop enumeration */
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// PDEVICE_OBJECT
// IopGetDeviceObjectFromDeviceInstance(PUNICODE_STRING DeviceInstance)
// {
//     DEVICETREE_TRAVERSE_CONTEXT Context;
//     IOP_FIND_DEVICE_INSTANCE_TRAVERSE_CONTEXT DeviceInstanceContext;
//
//     if (IopRootDeviceNode == NULL)
//         return NULL;
//
//     if (DeviceInstance == NULL ||
//         DeviceInstance->Length == 0)
//     {
//         if (IopRootDeviceNode->PhysicalDeviceObject)
//         {
//             ObReferenceObject(IopRootDeviceNode->PhysicalDeviceObject);
//             return IopRootDeviceNode->PhysicalDeviceObject;
//         }
//         else
//             return NULL;
//     }
//
//     /* Traverse the device tree to find the matching device node */
//     DeviceInstanceContext.InstancePath = DeviceInstance;
//     DeviceInstanceContext.DeviceObject = NULL;
//     IopInitDeviceTreeTraverseContext(&Context,
//                                      IopRootDeviceNode,
//                                      IopFindDeviceInstanceTraverse,
//                                      &DeviceInstanceContext);
//     (void)IopTraverseDeviceTree(&Context);
//
//     /* In case of error or instance not found, this will still be NULL from above. */
//     return DeviceInstanceContext.DeviceObject;
// }
//
// static NTSTATUS
// IopCaptureUnicodeString(PUNICODE_STRING DstName, PUNICODE_STRING SrcName)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     volatile UNICODE_STRING Name;
//
//     Name.Buffer = NULL;
//     _SEH2_TRY
//     {
//         Name.Length = SrcName->Length;
//         Name.MaximumLength = SrcName->MaximumLength;
//         if (Name.Length > Name.MaximumLength)
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             _SEH2_LEAVE;
//         }
//
//         if (Name.MaximumLength)
//         {
//             ProbeForRead(SrcName->Buffer,
//                          Name.MaximumLength,
//                          sizeof(WCHAR));
//             Name.Buffer = ExAllocatePool(NonPagedPool, Name.MaximumLength);
//             if (Name.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 _SEH2_LEAVE;
//             }
//
//             memcpy(Name.Buffer, SrcName->Buffer, Name.MaximumLength);
//         }
//
//         *DstName = Name;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (Name.Buffer)
//         {
//             ExFreePool(Name.Buffer);
//         }
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// PiControlInitializeDevice(
//     _In_ PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA ControlData)
// {
//     UNICODE_STRING DeviceInstance;
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_NODE DeviceNode;
//     NTSTATUS Status = STATUS_SUCCESS;
//     HANDLE InstanceKey;
//
//     DPRINT("PiControlInitializeDevice(%p)\n", ControlData);
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &ControlData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DPRINT("Device: %wZ\n", &DeviceInstance);
//
//     /* Leave, if the device already exists */
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceObject != NULL)
//     {
//         DPRINT1("Device %wZ already exists!\n", &DeviceInstance);
//         ObDereferenceObject(DeviceObject);
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     DPRINT("Device %wZ does not exist!\n", &DeviceInstance);
//
//     /* Create a device node for the device instance */
//     Status = PnpRootCreateDeviceObject(&DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateDevice() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Allocate a new device node */
//     DeviceNode = PipAllocateDeviceNode(DeviceObject);
//     if (DeviceNode == NULL)
//     {
//         DPRINT1("Failed to allocate a device node!\n");
//         IoDeleteDevice(DeviceObject);
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     // Set the device instance of the device node
//     // NOTE: a NULL-terminated string is required for PnpRootRegisterDevice
//     Status = RtlDuplicateUnicodeString(
//                 RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
//                 &DeviceInstance,
//                 &DeviceNode->InstancePath);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlDuplicateUnicodeString() failed (Status 0x%08lx)\n", Status);
//         IopFreeDeviceNode(DeviceNode);
//         IoDeleteDevice(DeviceObject);
//         goto done;
//     }
//
//     DeviceObject->Flags |= DO_BUS_ENUMERATED_DEVICE;
//     DeviceNode->Flags |= DNF_MADEUP | DNF_IDS_QUERIED | DNF_ENUMERATED;
//     PiSetDevNodeState(DeviceNode, DeviceNodeInitialized);
//
//     Status = IopCreateDeviceKeyPath(&DeviceInstance, REG_OPTION_NON_VOLATILE, &InstanceKey);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create the instance key! (Status %lx)\n", Status);
//         IopFreeDeviceNode(DeviceNode);
//         IoDeleteDevice(DeviceObject);
//         goto done;
//     }
//
//     /* Write the resource information to the registry */
//     IopSetDeviceInstanceData(InstanceKey, DeviceNode);
//
//     // Finish the root device registration
//     PnpRootRegisterDevice(DeviceObject);
//
//     /* Insert as a root enumerated device node */
//     PiInsertDevNode(DeviceNode, IopRootDeviceNode);
//
//     /* Report the device to the user-mode pnp manager */
//     IopQueueDeviceInstallEvent(&GUID_DEVICE_ENUMERATED, &DeviceNode->InstancePath);
//
//     ZwClose(InstanceKey);
// done:
//     ExFreePool(DeviceInstance.Buffer);
//
//     return Status;
// }
//
//
// /*
//  * Remove the current PnP event from the tail of the event queue
//  * and signal IopPnpNotifyEvent if there is yet another event in the queue.
//  */
// static
// NTSTATUS
// IopRemovePlugPlayEvent(
//     _In_ PPLUGPLAY_CONTROL_USER_RESPONSE_DATA ResponseData)
// {
//     /* Remove a pnp event entry from the tail of the queue */
//     if (!IsListEmpty(&IopPnpEventQueueHead))
//     {
//         ExFreePool(CONTAINING_RECORD(RemoveTailList(&IopPnpEventQueueHead), PNP_EVENT_ENTRY, ListEntry));
//     }
//
//     /* Signal the next pnp event in the queue */
//     if (!IsListEmpty(&IopPnpEventQueueHead))
//     {
//         KeSetEvent(&IopPnpNotifyEvent,
//                    0,
//                    FALSE);
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static NTSTATUS
// IopGetInterfaceDeviceList(PPLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA DeviceList)
// {
//     NTSTATUS Status;
//     PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA StackList;
//     UNICODE_STRING DeviceInstance;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     GUID FilterGuid;
//     PZZWSTR SymbolicLinkList = NULL, LinkList;
//     SIZE_T TotalLength;
//
//     _SEH2_TRY
//     {
//         RtlCopyMemory(&StackList, DeviceList, sizeof(PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA));
//
//         ProbeForRead(StackList.FilterGuid, sizeof(GUID), sizeof(UCHAR));
//         RtlCopyMemory(&FilterGuid, StackList.FilterGuid, sizeof(GUID));
//
//         if (StackList.Buffer != NULL && StackList.BufferSize != 0)
//         {
//             ProbeForWrite(StackList.Buffer, StackList.BufferSize, sizeof(UCHAR));
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &StackList.DeviceInstance);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the device object */
//         DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//         if (DeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(DeviceInstance.Buffer);
//         }
//     }
//
//     Status = IoGetDeviceInterfaces(&FilterGuid, DeviceObject, StackList.Flags, &SymbolicLinkList);
//     ObDereferenceObject(DeviceObject);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         return Status;
//     }
//
//     LinkList = SymbolicLinkList;
//     while (*SymbolicLinkList != UNICODE_NULL)
//     {
//         SymbolicLinkList += wcslen(SymbolicLinkList) + (sizeof(UNICODE_NULL) / sizeof(WCHAR));
//     }
//     TotalLength = ((SymbolicLinkList - LinkList + 1) * sizeof(WCHAR));
//
//     _SEH2_TRY
//     {
//         if (StackList.Buffer != NULL &&
//             StackList.BufferSize >= TotalLength)
//         {
//             // We've already probed the buffer for writing above.
//             RtlCopyMemory(StackList.Buffer, LinkList, TotalLength);
//         }
//
//         DeviceList->BufferSize = TotalLength;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ExFreePool(LinkList);
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     ExFreePool(LinkList);
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// IopGetDeviceProperty(PPLUGPLAY_CONTROL_PROPERTY_DATA PropertyData)
// {
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PDEVICE_NODE DeviceNode;
//     UNICODE_STRING DeviceInstance;
//     ULONG BufferSize;
//     ULONG Property;
//     DEVICE_REGISTRY_PROPERTY DeviceProperty;
//     PVOID Buffer;
//     NTSTATUS Status;
//
//     DPRINT("IopGetDeviceProperty() called\n");
//     DPRINT("Device name: %wZ\n", &PropertyData->DeviceInstance);
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &PropertyData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     _SEH2_TRY
//     {
//         Property = PropertyData->Property;
//         BufferSize = PropertyData->BufferSize;
//         ProbeForWrite(PropertyData->Buffer,
//                       BufferSize,
//                       sizeof(UCHAR));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (DeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(DeviceInstance.Buffer);
//         }
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Get the device object */
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceInstance.Buffer != NULL)
//     {
//         ExFreePool(DeviceInstance.Buffer);
//     }
//     if (DeviceObject == NULL)
//     {
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     Buffer = ExAllocatePool(NonPagedPool, BufferSize);
//     if (Buffer == NULL)
//     {
//         ObDereferenceObject(DeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//
//     DeviceNode = ((PEXTENDED_DEVOBJ_EXTENSION)DeviceObject->DeviceObjectExtension)->DeviceNode;
//
//     if (Property == PNP_PROPERTY_POWER_DATA)
//     {
//         if (BufferSize < sizeof(CM_POWER_DATA))
//         {
//             BufferSize = 0;
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//         else
//         {
//             DEVICE_CAPABILITIES DeviceCapabilities;
//             PCM_POWER_DATA PowerData;
//             IO_STACK_LOCATION Stack;
//             IO_STATUS_BLOCK IoStatusBlock;
//
//             PowerData = (PCM_POWER_DATA)Buffer;
//             RtlZeroMemory(PowerData, sizeof(CM_POWER_DATA));
//             PowerData->PD_Size = sizeof(CM_POWER_DATA);
//
//             RtlZeroMemory(&DeviceCapabilities, sizeof(DEVICE_CAPABILITIES));
//             DeviceCapabilities.Size = sizeof(DEVICE_CAPABILITIES);
//             DeviceCapabilities.Version = 1;
//             DeviceCapabilities.Address = -1;
//             DeviceCapabilities.UINumber = -1;
//
//             Stack.Parameters.DeviceCapabilities.Capabilities = &DeviceCapabilities;
//
//             Status = IopInitiatePnpIrp(DeviceObject,
//                                        &IoStatusBlock,
//                                        IRP_MN_QUERY_CAPABILITIES,
//                                        &Stack);
//             if (NT_SUCCESS(Status))
//             {
//                 DPRINT("Got device capabiliities\n");
//
//                 PowerData->PD_MostRecentPowerState = PowerDeviceD0; // FIXME
//                 if (DeviceCapabilities.DeviceD1)
//                     PowerData->PD_Capabilities |= PDCAP_D1_SUPPORTED;
//                 if (DeviceCapabilities.DeviceD2)
//                     PowerData->PD_Capabilities |= PDCAP_D2_SUPPORTED;
//                 if (DeviceCapabilities.WakeFromD0)
//                     PowerData->PD_Capabilities |= PDCAP_WAKE_FROM_D0_SUPPORTED;
//                 if (DeviceCapabilities.WakeFromD1)
//                     PowerData->PD_Capabilities |= PDCAP_WAKE_FROM_D1_SUPPORTED;
//                 if (DeviceCapabilities.WakeFromD2)
//                     PowerData->PD_Capabilities |= PDCAP_WAKE_FROM_D2_SUPPORTED;
//                 if (DeviceCapabilities.WakeFromD3)
//                     PowerData->PD_Capabilities |= PDCAP_WAKE_FROM_D3_SUPPORTED;
//                 if (DeviceCapabilities.WarmEjectSupported)
//                     PowerData->PD_Capabilities |= PDCAP_WARM_EJECT_SUPPORTED;
//                 PowerData->PD_D1Latency = DeviceCapabilities.D1Latency;
//                 PowerData->PD_D2Latency = DeviceCapabilities.D2Latency;
//                 PowerData->PD_D3Latency = DeviceCapabilities.D3Latency;
//                 RtlCopyMemory(&PowerData->PD_PowerStateMapping,
//                               &DeviceCapabilities.DeviceState,
//                               sizeof(DeviceCapabilities.DeviceState));
//                 PowerData->PD_DeepestSystemWake = DeviceCapabilities.SystemWake;
//             }
//             else
//             {
//                 DPRINT("IRP_MN_QUERY_CAPABILITIES failed (Status 0x%08lx)\n", Status);
//
//                 PowerData->PD_Capabilities = PDCAP_D0_SUPPORTED | PDCAP_D3_SUPPORTED;
//                 PowerData->PD_MostRecentPowerState = PowerDeviceD0;
//             }
//         }
//     }
//     else if (Property == PNP_PROPERTY_REMOVAL_POLICY_OVERRIDE)
//     {
//         UNIMPLEMENTED;
//         BufferSize = 0;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//     else if (Property == PNP_PROPERTY_REMOVAL_POLICY_HARDWARE_DEFAULT)
//     {
//         if (BufferSize < sizeof(DeviceNode->HardwareRemovalPolicy))
//         {
//             BufferSize = 0;
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//         else
//         {
//             BufferSize = sizeof(DeviceNode->HardwareRemovalPolicy);
//             RtlCopyMemory(Buffer,
//                           &DeviceNode->HardwareRemovalPolicy,
//                           BufferSize);
//         }
//     }
// #if (WINVER >= _WIN32_WINNT_WS03)
//     else if (Property == PNP_PROPERTY_LOCATION_PATHS)
//     {
//         UNIMPLEMENTED;
//         BufferSize = 0;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
// #endif
//     else
//     {
//         switch (Property)
//         {
//             case PNP_PROPERTY_UI_NUMBER:
//                 DeviceProperty = DevicePropertyUINumber;
//                 break;
//
//             case PNP_PROPERTY_PHYSICAL_DEVICE_OBJECT_NAME:
//                 DeviceProperty = DevicePropertyPhysicalDeviceObjectName;
//                 break;
//
//             case PNP_PROPERTY_BUSTYPEGUID:
//                 DeviceProperty = DevicePropertyBusTypeGuid;
//                 break;
//
//             case PNP_PROPERTY_LEGACYBUSTYPE:
//                 DeviceProperty = DevicePropertyLegacyBusType;
//                 break;
//
//             case PNP_PROPERTY_BUSNUMBER:
//                 DeviceProperty = DevicePropertyBusNumber;
//                 break;
//
//             case PNP_PROPERTY_REMOVAL_POLICY:
//                 DeviceProperty = DevicePropertyRemovalPolicy;
//                 break;
//
//             case PNP_PROPERTY_ADDRESS:
//                 DeviceProperty = DevicePropertyAddress;
//                 break;
//
//             case PNP_PROPERTY_ENUMERATOR_NAME:
//                 DeviceProperty = DevicePropertyEnumeratorName;
//                 break;
//
//             case PNP_PROPERTY_INSTALL_STATE:
//                 DeviceProperty = DevicePropertyInstallState;
//                 break;
//
// #if (WINVER >= _WIN32_WINNT_WIN7)
//             case PNP_PROPERTY_CONTAINERID:
//                 DeviceProperty = DevicePropertyContainerID;
//                 break;
// #endif
//
//             default:
//                 BufferSize = 0;
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//         }
//
//         if (Status == STATUS_SUCCESS)
//         {
//             Status = IoGetDeviceProperty(DeviceObject,
//                                          DeviceProperty,
//                                          BufferSize,
//                                          Buffer,
//                                          &BufferSize);
//         }
//     }
//
//     ObDereferenceObject(DeviceObject);
//
//     if (NT_SUCCESS(Status))
//     {
//         _SEH2_TRY
//         {
//             RtlCopyMemory(PropertyData->Buffer, Buffer, BufferSize);
//             PropertyData->BufferSize = BufferSize;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     ExFreePool(Buffer);
//     return Status;
// }
//
//
// static NTSTATUS
// IopGetRelatedDevice(PPLUGPLAY_CONTROL_RELATED_DEVICE_DATA RelatedDeviceData)
// {
//     UNICODE_STRING RootDeviceName;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PDEVICE_NODE DeviceNode = NULL;
//     PDEVICE_NODE RelatedDeviceNode;
//     UNICODE_STRING TargetDeviceInstance;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Relation = 0;
//     ULONG MaximumLength = 0;
//
//     DPRINT("IopGetRelatedDevice() called\n");
//     DPRINT("Device name: %wZ\n", &RelatedDeviceData->TargetDeviceInstance);
//
//     Status = IopCaptureUnicodeString(&TargetDeviceInstance, &RelatedDeviceData->TargetDeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     _SEH2_TRY
//     {
//         Relation = RelatedDeviceData->Relation;
//         MaximumLength = RelatedDeviceData->RelatedDeviceInstanceLength;
//         ProbeForWrite(RelatedDeviceData->RelatedDeviceInstance,
//                       MaximumLength,
//                       sizeof(WCHAR));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (TargetDeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(TargetDeviceInstance.Buffer);
//         }
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     RtlInitUnicodeString(&RootDeviceName,
//                          L"HTREE\\ROOT\\0");
//     if (RtlEqualUnicodeString(&TargetDeviceInstance,
//                               &RootDeviceName,
//                               TRUE))
//     {
//         DeviceNode = IopRootDeviceNode;
//         if (TargetDeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(TargetDeviceInstance.Buffer);
//         }
//     }
//     else
//     {
//         /* Get the device object */
//         DeviceObject = IopGetDeviceObjectFromDeviceInstance(&TargetDeviceInstance);
//         if (TargetDeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(TargetDeviceInstance.Buffer);
//         }
//         if (DeviceObject == NULL)
//             return STATUS_NO_SUCH_DEVICE;
//
//         DeviceNode = ((PEXTENDED_DEVOBJ_EXTENSION)DeviceObject->DeviceObjectExtension)->DeviceNode;
//     }
//
//     switch (Relation)
//     {
//         case PNP_GET_PARENT_DEVICE:
//             RelatedDeviceNode = DeviceNode->Parent;
//             break;
//
//         case PNP_GET_CHILD_DEVICE:
//             RelatedDeviceNode = DeviceNode->Child;
//             break;
//
//         case PNP_GET_SIBLING_DEVICE:
//             RelatedDeviceNode = DeviceNode->Sibling;
//             break;
//
//         default:
//             if (DeviceObject != NULL)
//             {
//                 ObDereferenceObject(DeviceObject);
//             }
//
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     if (RelatedDeviceNode == NULL)
//     {
//         if (DeviceObject)
//         {
//             ObDereferenceObject(DeviceObject);
//         }
//
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     if (RelatedDeviceNode->InstancePath.Length > MaximumLength)
//     {
//         if (DeviceObject)
//         {
//             ObDereferenceObject(DeviceObject);
//         }
//
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Copy related device instance name */
//     _SEH2_TRY
//     {
//         RtlCopyMemory(RelatedDeviceData->RelatedDeviceInstance,
//                       RelatedDeviceNode->InstancePath.Buffer,
//                       RelatedDeviceNode->InstancePath.Length);
//         RelatedDeviceData->RelatedDeviceInstanceLength = RelatedDeviceNode->InstancePath.Length;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (DeviceObject != NULL)
//     {
//         ObDereferenceObject(DeviceObject);
//     }
//
//     DPRINT("IopGetRelatedDevice() done\n");
//
//     return Status;
// }
//
// static
// BOOLEAN
// PiIsDevNodeStarted(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     return (DeviceNode->State == DeviceNodeStartPending ||
//             DeviceNode->State == DeviceNodeStartCompletion ||
//             DeviceNode->State == DeviceNodeStartPostWork ||
//             DeviceNode->State == DeviceNodeStarted ||
//             DeviceNode->State == DeviceNodeQueryStopped ||
//             DeviceNode->State == DeviceNodeEnumeratePending ||
//             DeviceNode->State == DeviceNodeEnumerateCompletion ||
//             DeviceNode->State == DeviceNodeStopped ||
//             DeviceNode->State == DeviceNodeRestartCompletion);
// }
//
// static ULONG
// IopGetDeviceNodeStatus(PDEVICE_NODE DeviceNode)
// {
//     ULONG Output = DN_NT_ENUMERATOR | DN_NT_DRIVER;
//
//     if (DeviceNode->Parent == IopRootDeviceNode)
//         Output |= DN_ROOT_ENUMERATED;
//
//     // FIXME: review for deleted and removed states
//     if (DeviceNode->State >= DeviceNodeDriversAdded)
//         Output |= DN_DRIVER_LOADED;
//
//     if (PiIsDevNodeStarted(DeviceNode))
//         Output |= DN_STARTED;
//
//     if (DeviceNode->UserFlags & DNUF_WILL_BE_REMOVED)
//         Output |= DN_WILL_BE_REMOVED;
//
//     if (DeviceNode->Flags & DNF_HAS_PROBLEM)
//         Output |= DN_HAS_PROBLEM;
//
//     if (DeviceNode->Flags & DNF_HAS_PRIVATE_PROBLEM)
//         Output |= DN_PRIVATE_PROBLEM;
//
//     if (DeviceNode->Flags & DNF_DRIVER_BLOCKED)
//         Output |= DN_DRIVER_BLOCKED;
//
//     if (DeviceNode->Flags & DNF_CHILD_WITH_INVALID_ID)
//         Output |= DN_CHILD_WITH_INVALID_ID;
//
//     if (DeviceNode->Flags & DNF_HAS_PRIVATE_PROBLEM)
//         Output |= DN_PRIVATE_PROBLEM;
//
//     if (DeviceNode->Flags & DNF_LEGACY_DRIVER)
//         Output |= DN_LEGACY_DRIVER;
//
//     if (DeviceNode->UserFlags & DNUF_DONT_SHOW_IN_UI)
//         Output |= DN_NO_SHOW_IN_DM;
//
//     if (!(DeviceNode->UserFlags & DNUF_NOT_DISABLEABLE))
//         Output |= DN_DISABLEABLE;
//
//     return Output;
// }
//
// static NTSTATUS
// IopDeviceStatus(PPLUGPLAY_CONTROL_STATUS_DATA StatusData)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_NODE DeviceNode;
//     ULONG Operation = 0;
//     ULONG DeviceStatus = 0;
//     ULONG DeviceProblem = 0;
//     UNICODE_STRING DeviceInstance;
//     NTSTATUS Status;
//
//     DPRINT("IopDeviceStatus() called\n");
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &StatusData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DPRINT("Device name: '%wZ'\n", &DeviceInstance);
//
//     _SEH2_TRY
//     {
//         Operation = StatusData->Operation;
//         if (Operation == PNP_SET_DEVICE_STATUS)
//         {
//             DeviceStatus = StatusData->DeviceStatus;
//             DeviceProblem = StatusData->DeviceProblem;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         if (DeviceInstance.Buffer != NULL)
//         {
//             ExFreePool(DeviceInstance.Buffer);
//         }
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Get the device object */
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceInstance.Buffer != NULL)
//     {
//         ExFreePool(DeviceInstance.Buffer);
//     }
//     if (DeviceObject == NULL)
//     {
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     DeviceNode = IopGetDeviceNode(DeviceObject);
//
//     switch (Operation)
//     {
//         case PNP_GET_DEVICE_STATUS:
//             DPRINT("Get status data\n");
//             DeviceStatus = IopGetDeviceNodeStatus(DeviceNode);
//             DeviceProblem = DeviceNode->Problem;
//             break;
//
//         case PNP_SET_DEVICE_STATUS:
//             DPRINT1("Set status data is NOT SUPPORTED\n");
//             break;
//
//         case PNP_CLEAR_DEVICE_STATUS:
//             DPRINT1("FIXME: Clear status data!\n");
//             break;
//     }
//
//     ObDereferenceObject(DeviceObject);
//
//     if (Operation == PNP_GET_DEVICE_STATUS)
//     {
//         _SEH2_TRY
//         {
//             StatusData->DeviceStatus = DeviceStatus;
//             StatusData->DeviceProblem = DeviceProblem;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// IopGetDeviceRelations(PPLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA RelationsData)
// {
//     UNICODE_STRING DeviceInstance;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_RELATIONS DeviceRelations = NULL;
//     PDEVICE_OBJECT ChildDeviceObject;
//     PDEVICE_NODE ChildDeviceNode;
//     ULONG i;
//     ULONG Relations;
//     ULONG BufferSize, RequiredSize;
//     ULONG BufferLeft;
//     PWCHAR Buffer, Ptr;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT1("IopGetDeviceRelations() called\n");
//     DPRINT1("Device name: %wZ\n", &RelationsData->DeviceInstance);
//     DPRINT("Relations: %lu\n", RelationsData->Relations);
//     DPRINT("BufferSize: %lu\n", RelationsData->BufferSize);
//     DPRINT("Buffer: %p\n", RelationsData->Buffer);
//
//     _SEH2_TRY
//     {
//         Relations = RelationsData->Relations;
//         BufferSize = RelationsData->BufferSize;
//         Buffer = RelationsData->Buffer;
//
//         ProbeForWrite(Buffer, BufferSize, sizeof(CHAR));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &RelationsData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopCaptureUnicodeString() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Get the device object */
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceObject == NULL)
//     {
//         DPRINT1("IopGetDeviceObjectFromDeviceInstance() returned NULL\n");
//         Status = STATUS_NO_SUCH_DEVICE;
//         goto done;
//     }
//
//     switch (Relations)
//     {
//         case PNP_EJECT_RELATIONS:
//             Stack.Parameters.QueryDeviceRelations.Type = EjectionRelations;
//             break;
//
//         case PNP_REMOVAL_RELATIONS:
//             Stack.Parameters.QueryDeviceRelations.Type = RemovalRelations;
//             break;
//
//         case PNP_POWER_RELATIONS:
//             Stack.Parameters.QueryDeviceRelations.Type = PowerRelations;
//             break;
//
//         case PNP_BUS_RELATIONS:
//             Stack.Parameters.QueryDeviceRelations.Type = BusRelations;
//             break;
//
//         default:
//             Status = STATUS_INVALID_PARAMETER;
//             goto done;
//     }
//
//     Status = IopInitiatePnpIrp(DeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_QUERY_DEVICE_RELATIONS,
//                                &Stack);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopInitiatePnpIrp() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
//
//     DPRINT1("Found %d device relations\n", DeviceRelations->Count);
//
//     _SEH2_TRY
//     {
//         RequiredSize = 0;
//         BufferLeft = BufferSize;
//         Ptr = Buffer;
//
//         for (i = 0; i < DeviceRelations->Count; i++)
//         {
//             ChildDeviceObject = DeviceRelations->Objects[i];
//
//             ChildDeviceNode = IopGetDeviceNode(ChildDeviceObject);
//             if (ChildDeviceNode)
//             {
//                 DPRINT("Device instance: %wZ\n", &ChildDeviceNode->InstancePath);
//                 DPRINT("RequiredSize: %hu\n", ChildDeviceNode->InstancePath.Length + sizeof(WCHAR));
//
//                 if (Ptr != NULL)
//                 {
//                     if (BufferLeft < ChildDeviceNode->InstancePath.Length + 2 * sizeof(WCHAR))
//                     {
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                         break;
//                     }
//
//                     RtlCopyMemory(Ptr,
//                                   ChildDeviceNode->InstancePath.Buffer,
//                                   ChildDeviceNode->InstancePath.Length);
//                     Ptr = (PWCHAR)((ULONG_PTR)Ptr + ChildDeviceNode->InstancePath.Length);
//                     *Ptr = UNICODE_NULL;
//                     Ptr = (PWCHAR)((ULONG_PTR)Ptr + sizeof(WCHAR));
//
//                     BufferLeft -= (ChildDeviceNode->InstancePath.Length + sizeof(WCHAR));
//                 }
//
//                 RequiredSize += (ChildDeviceNode->InstancePath.Length + sizeof(WCHAR));
//             }
//         }
//
//         if (Ptr != NULL && BufferLeft >= sizeof(WCHAR))
//             *Ptr = UNICODE_NULL;
//
//         if (RequiredSize > 0)
//             RequiredSize += sizeof(WCHAR);
//
//         DPRINT("BufferSize: %lu  RequiredSize: %lu\n", RelationsData->BufferSize, RequiredSize);
//
//         RelationsData->BufferSize = RequiredSize;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
// done:
//     if (DeviceRelations != NULL)
//         ExFreePool(DeviceRelations);
//
//     if (DeviceObject != NULL)
//         ObDereferenceObject(DeviceObject);
//
//     if (DeviceInstance.Buffer != NULL)
//         ExFreePool(DeviceInstance.Buffer);
//
//     return Status;
// }
//
// static NTSTATUS
// IopGetDeviceDepth(PPLUGPLAY_CONTROL_DEPTH_DATA DepthData)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_NODE DeviceNode;
//     UNICODE_STRING DeviceInstance;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("IopGetDeviceDepth() called\n");
//     DPRINT("Device name: %wZ\n", &DepthData->DeviceInstance);
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &DepthData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get the device object */
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceInstance.Buffer != NULL)
//     {
//         ExFreePool(DeviceInstance.Buffer);
//     }
//     if (DeviceObject == NULL)
//     {
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     DeviceNode = IopGetDeviceNode(DeviceObject);
//
//     _SEH2_TRY
//     {
//         DepthData->Depth = DeviceNode->Level;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     ObDereferenceObject(DeviceObject);
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiControlSyncDeviceAction(
//     _In_ PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA DeviceData,
//     _In_ PLUGPLAY_CONTROL_CLASS ControlClass)
// {
//     PDEVICE_OBJECT DeviceObject;
//     NTSTATUS Status;
//     UNICODE_STRING DeviceInstance;
//
//     ASSERT(ControlClass == PlugPlayControlEnumerateDevice ||
//            ControlClass == PlugPlayControlStartDevice ||
//            ControlClass == PlugPlayControlResetDevice);
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &DeviceData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceInstance.Buffer != NULL)
//     {
//         ExFreePool(DeviceInstance.Buffer);
//     }
//     if (DeviceObject == NULL)
//     {
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     DEVICE_ACTION Action;
//
//     switch (ControlClass)
//     {
//         case PlugPlayControlEnumerateDevice:
//             Action = PiActionEnumDeviceTree;
//             break;
//         case PlugPlayControlStartDevice:
//             Action = PiActionStartDevice;
//             break;
//         case PlugPlayControlResetDevice:
//             Action = PiActionResetDevice;
//             break;
//         default:
//             UNREACHABLE;
//             break;
//     }
//
//     Status = PiPerformSyncDeviceAction(DeviceObject, Action);
//
//     ObDereferenceObject(DeviceObject);
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiControlQueryRemoveDevice(
//     _In_ PPLUGPLAY_CONTROL_QUERY_REMOVE_DATA ControlData)
// {
//     PDEVICE_OBJECT DeviceObject;
//     NTSTATUS Status;
//     UNICODE_STRING DeviceInstance;
//
//     Status = IopCaptureUnicodeString(&DeviceInstance, &ControlData->DeviceInstance);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DeviceObject = IopGetDeviceObjectFromDeviceInstance(&mut DeviceInstance);
//     if (DeviceInstance.Buffer != NULL)
//     {
//         ExFreePool(DeviceInstance.Buffer);
//     }
//     if (DeviceObject == NULL)
//     {
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     UNIMPLEMENTED;
//     Status = STATUS_NOT_IMPLEMENTED;
//
//     ObDereferenceObject(DeviceObject);
//
//     return Status;
// }
//
// static
// NTSTATUS
// PiControlGetInterfaceDeviceAlias(
//     _In_ PPLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA ControlData)
// {
//     UNICODE_STRING AliasSymbolicLinkName;
//
//     DPRINT("PiControlGetInterfaceDeviceAlias()\n");
//     DPRINT("SymbolicLinkName: %wZ\n", &ControlData->SymbolicLinkName);
//     DPRINT("AliasInterfaceClassGuid: %p\n", ControlData->AliasInterfaceClassGuid);
//     DPRINT("AliasSymbolicLinkName: %p\n", ControlData->AliasSymbolicLinkName);
//     DPRINT("AliasSymbolicLinkNameLength: %lu\n", ControlData->AliasSymbolicLinkNameLength);
//
//     AliasSymbolicLinkName.Length = 0;
//     AliasSymbolicLinkName.MaximumLength = ControlData->AliasSymbolicLinkNameLength;
//     AliasSymbolicLinkName.Buffer = ControlData->AliasSymbolicLinkName;
//     return IoGetDeviceInterfaceAlias(&ControlData->SymbolicLinkName,
//                                      ControlData->AliasInterfaceClassGuid,
//                                      &AliasSymbolicLinkName);
// }
//
// static
// NTSTATUS
// PiControlDeviceClassAssociation(
//     _In_ PPLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA ControlData)
// {
//     PDEVICE_OBJECT DeviceObject = NULL;
//     UNICODE_STRING SymbolicLink;
//     NTSTATUS Status;
//
//     DPRINT("PiControlDeviceClassAssociation()\n");
//     DPRINT("DeviceInstance: %wZ\n", &ControlData->DeviceInstance);
//
//     if (ControlData->Register)
//     {
//         if ((ControlData->DeviceInstance.Buffer == NULL) ||
//             (ControlData->DeviceInstance.Length == 0))
//             return STATUS_INVALID_PARAMETER;
//
//         if (ControlData->InterfaceGuid == NULL)
//             return STATUS_INVALID_PARAMETER;
//
//         DeviceObject = IopGetDeviceObjectFromDeviceInstance(&ControlData->DeviceInstance);
//         if (DeviceObject == NULL)
//             return STATUS_NO_SUCH_DEVICE;
//
//         Status = IoRegisterDeviceInterface(DeviceObject,
//                                            ControlData->InterfaceGuid,
//                                            &ControlData->Reference,
//                                            &SymbolicLink);
//         if (NT_SUCCESS(Status))
//         {
//             if (SymbolicLink.Length + sizeof(WCHAR) <= ControlData->SymbolicLinkNameLength)
//             {
//                 RtlCopyMemory(ControlData->SymbolicLinkName, SymbolicLink.Buffer, SymbolicLink.Length);
//                 ControlData->SymbolicLinkName[SymbolicLink.Length / sizeof(WCHAR)] = UNICODE_NULL;
//                 ControlData->SymbolicLinkNameLength = SymbolicLink.Length + sizeof(WCHAR);
//             }
//             else
//             {
//                 ControlData->SymbolicLinkNameLength = SymbolicLink.Length + sizeof(WCHAR);
//                 Status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             ExFreePool(SymbolicLink.Buffer);
//         }
//
//         ObDereferenceObject(DeviceObject);
//     }
//     else
//     {
//         UNIMPLEMENTED;
//         Status = STATUS_NOT_IMPLEMENTED;
//     }
//
//     return Status;
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * Plug and Play event structure used by NtGetPlugPlayEvent.
//  *
//  * EventGuid
//  *    Can be one of the following values:
//  *       GUID_HWPROFILE_QUERY_CHANGE
//  *       GUID_HWPROFILE_CHANGE_CANCELLED
//  *       GUID_HWPROFILE_CHANGE_COMPLETE
//  *       GUID_TARGET_DEVICE_QUERY_REMOVE
//  *       GUID_TARGET_DEVICE_REMOVE_CANCELLED
//  *       GUID_TARGET_DEVICE_REMOVE_COMPLETE
//  *       GUID_PNP_CUSTOM_NOTIFICATION
//  *       GUID_PNP_POWER_NOTIFICATION
//  *       GUID_DEVICE_* (see above)
//  *
//  * EventCategory
//  *    Type of the event that happened.
//  *
//  * Result
//  *    ?
//  *
//  * Flags
//  *    ?
//  *
//  * TotalSize
//  *    Size of the event block including the device IDs and other
//  *    per category specific fields.
//  */
//
// /*
//  * NtGetPlugPlayEvent
//  *
//  * Returns one Plug & Play event from a global queue.
//  *
//  * Parameters
//  *    Reserved1
//  *    Reserved2
//  *       Always set to zero.
//  *
//  *    Buffer
//  *       The buffer that will be filled with the event information on
//  *       successful return from the function.
//  *
//  *    BufferSize
//  *       Size of the buffer pointed by the Buffer parameter. If the
//  *       buffer size is not large enough to hold the whole event
//  *       information, error STATUS_BUFFER_TOO_SMALL is returned and
//  *       the buffer remains untouched.
//  *
//  * Return Values
//  *    STATUS_PRIVILEGE_NOT_HELD
//  *    STATUS_BUFFER_TOO_SMALL
//  *    STATUS_SUCCESS
//  *
//  * Remarks
//  *    This function isn't multi-thread safe!
//  *
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtGetPlugPlayEvent(IN ULONG Reserved1,
//                    IN ULONG Reserved2,
//                    OUT PPLUGPLAY_EVENT_BLOCK Buffer,
//                    IN ULONG BufferSize)
// {
//     PPNP_EVENT_ENTRY Entry;
//     NTSTATUS Status;
//
//     DPRINT("NtGetPlugPlayEvent() called\n");
//
//     /* Function can only be called from user-mode */
//     if (KeGetPreviousMode() == KernelMode)
//     {
//         DPRINT1("NtGetPlugPlayEvent cannot be called from kernel mode!\n");
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Check for Tcb privilege */
//     if (!SeSinglePrivilegeCheck(SeTcbPrivilege,
//                                 UserMode))
//     {
//         DPRINT1("NtGetPlugPlayEvent: Caller does not hold the SeTcbPrivilege privilege!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Wait for a PnP event */
//     DPRINT("Waiting for pnp notification event\n");
//     Status = KeWaitForSingleObject(&IopPnpNotifyEvent,
//                                    UserRequest,
//                                    UserMode,
//                                    FALSE,
//                                    NULL);
//     if (!NT_SUCCESS(Status) || Status == STATUS_USER_APC)
//     {
//         DPRINT("KeWaitForSingleObject() failed (Status %lx)\n", Status);
//         ASSERT(Status == STATUS_USER_APC);
//         return Status;
//     }
//
//     /* Get entry from the tail of the queue */
//     Entry = CONTAINING_RECORD(IopPnpEventQueueHead.Blink,
//                               PNP_EVENT_ENTRY,
//                               ListEntry);
//
//     /* Check the buffer size */
//     if (BufferSize < Entry->Event.TotalSize)
//     {
//         DPRINT1("Buffer is too small for the pnp-event\n");
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Copy event data to the user buffer */
//     _SEH2_TRY
//     {
//         ProbeForWrite(Buffer,
//                       Entry->Event.TotalSize,
//                       sizeof(UCHAR));
//         RtlCopyMemory(Buffer,
//                       &Entry->Event,
//                       Entry->Event.TotalSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     DPRINT("NtGetPlugPlayEvent() done\n");
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * NtPlugPlayControl
//  *
//  * A function for doing various Plug & Play operations from user mode.
//  *
//  * Parameters
//  *    PlugPlayControlClass
//  *       0x00   Reenumerate device tree
//  *
//  *              Buffer points to UNICODE_STRING decribing the instance
//  *              path (like "HTREE\ROOT\0" or "Root\ACPI_HAL\0000"). For
//  *              more information about instance paths see !devnode command
//  *              in kernel debugger or look at "Inside Windows 2000" book,
//  *              chapter "Driver Loading, Initialization, and Installation".
//  *
//  *       0x01   Register new device
//  *       0x02   Deregister device
//  *       0x03   Initialize device
//  *       0x04   Start device
//  *       0x06   Query and remove device
//  *       0x07   User response
//  *
//  *              Called after processing the message from NtGetPlugPlayEvent.
//  *
//  *       0x08   Generate legacy device
//  *       0x09   Get interface device list
//  *       0x0A   Get property data
//  *       0x0B   Device class association (Registration)
//  *       0x0C   Get related device
//  *       0x0D   Get device interface alias
//  *       0x0E   Get/set/clear device status
//  *       0x0F   Get device depth
//  *       0x10   Query device relations
//  *       0x11   Query target device relation
//  *       0x12   Query conflict list
//  *       0x13   Retrieve dock data
//  *       0x14   Reset device
//  *       0x15   Halt device
//  *       0x16   Get blocked driver data
//  *
//  *    Buffer
//  *       The buffer contains information that is specific to each control
//  *       code. The buffer is read-only.
//  *
//  *    BufferSize
//  *       Size of the buffer pointed by the Buffer parameter. If the
//  *       buffer size specifies incorrect value for specified control
//  *       code, error ??? is returned.
//  *
//  * Return Values
//  *    STATUS_PRIVILEGE_NOT_HELD
//  *    STATUS_SUCCESS
//  *    ...
//  *
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// NtPlugPlayControl(IN PLUGPLAY_CONTROL_CLASS PlugPlayControlClass,
//                   IN OUT PVOID Buffer,
//                   IN ULONG BufferLength)
// {
//     DPRINT("NtPlugPlayControl(%d %p %lu) called\n",
//            PlugPlayControlClass, Buffer, BufferLength);
//
//     /* Function can only be called from user-mode */
//     if (KeGetPreviousMode() == KernelMode)
//     {
//         DPRINT1("NtGetPlugPlayEvent cannot be called from kernel mode!\n");
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Check for Tcb privilege */
//     if (!SeSinglePrivilegeCheck(SeTcbPrivilege,
//                                 UserMode))
//     {
//         DPRINT1("NtGetPlugPlayEvent: Caller does not hold the SeTcbPrivilege privilege!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Probe the buffer */
//     _SEH2_TRY
//     {
//         ProbeForWrite(Buffer,
//                       BufferLength,
//                       sizeof(ULONG));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     switch (PlugPlayControlClass)
//     {
//         case PlugPlayControlEnumerateDevice:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_ENUMERATE_DEVICE_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             // the Flags field is not used anyway
//             return PiControlSyncDeviceAction((PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA)Buffer,
//                                              PlugPlayControlClass);
//
// //        case PlugPlayControlRegisterNewDevice:
// //        case PlugPlayControlDeregisterDevice:
//
//         case PlugPlayControlInitializeDevice:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return PiControlInitializeDevice((PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA)Buffer);
//
//         case PlugPlayControlStartDevice:
//         case PlugPlayControlResetDevice:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return PiControlSyncDeviceAction((PPLUGPLAY_CONTROL_DEVICE_CONTROL_DATA)Buffer,
//                                              PlugPlayControlClass);
//
//         case PlugPlayControlUnlockDevice:
//             return STATUS_NOT_IMPLEMENTED;
//
//         case PlugPlayControlQueryAndRemoveDevice:
//               if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_QUERY_REMOVE_DATA))
//                   return STATUS_INVALID_PARAMETER;
//               return PiControlQueryRemoveDevice((PPLUGPLAY_CONTROL_QUERY_REMOVE_DATA)Buffer);
//
//         case PlugPlayControlUserResponse:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_USER_RESPONSE_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopRemovePlugPlayEvent((PPLUGPLAY_CONTROL_USER_RESPONSE_DATA)Buffer);
//
// //        case PlugPlayControlGenerateLegacyDevice:
//
//         case PlugPlayControlGetInterfaceDeviceList:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopGetInterfaceDeviceList((PPLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA)Buffer);
//
//         case PlugPlayControlProperty:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_PROPERTY_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopGetDeviceProperty((PPLUGPLAY_CONTROL_PROPERTY_DATA)Buffer);
//
//         case PlugPlayControlDeviceClassAssociation:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return PiControlDeviceClassAssociation((PPLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA)Buffer);
//
//         case PlugPlayControlGetRelatedDevice:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_RELATED_DEVICE_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopGetRelatedDevice((PPLUGPLAY_CONTROL_RELATED_DEVICE_DATA)Buffer);
//
//         case PlugPlayControlGetInterfaceDeviceAlias:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return PiControlGetInterfaceDeviceAlias((PPLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA)Buffer);
//
//         case PlugPlayControlDeviceStatus:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_STATUS_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopDeviceStatus((PPLUGPLAY_CONTROL_STATUS_DATA)Buffer);
//
//         case PlugPlayControlGetDeviceDepth:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_DEPTH_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopGetDeviceDepth((PPLUGPLAY_CONTROL_DEPTH_DATA)Buffer);
//
//         case PlugPlayControlQueryDeviceRelations:
//             if (!Buffer || BufferLength < sizeof(PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA))
//                 return STATUS_INVALID_PARAMETER;
//             return IopGetDeviceRelations((PPLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA)Buffer);
//
// //        case PlugPlayControlTargetDeviceRelation:
// //        case PlugPlayControlQueryConflictList:
// //        case PlugPlayControlRetrieveDock:
// //        case PlugPlayControlHaltDevice:
// //        case PlugPlayControlGetBlockedDriverList:
//
//         default:
//             return STATUS_NOT_IMPLEMENTED;
//     }
//
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
