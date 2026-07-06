use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::{STATUS_CONFLICTING_ADDRESSES, NonPagedPool};
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _KEVENT, _OBJECT_ATTRIBUTES, _TARGET_DEVICE_CUSTOM_NOTIFICATION, _WORK_QUEUE_ITEM,
    _UNICODE_STRING, GUID,  _EVENT_TYPE_NotificationEvent, _KWAIT_REASON_Executive,
    PKEVENT, PTARGET_DEVICE_CUSTOM_NOTIFICATION,
    PWORK_QUEUE_ITEM, PDEVICE_OBJECT, PDRIVER_OBJECT, PCM_RESOURCE_LIST,
    PIO_RESOURCE_REQUIREMENTS_LIST, PDEVICE_CHANGE_COMPLETE_CALLBACK, PDEVICE_NODE,
    INTERFACE_TYPE, HANDLE, ACCESS_MASK, PNTSTATUS,
    ExQueueWorkItem, KeInitializeEvent, KeWaitForSingleObject, RtlInitUnicodeString,
    ZwClose, ZwCreateKey, ZwSetValueKey,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const TAG_PNP: u32 = u32::from_le_bytes(*b"Pp  ");

#[repr(C)]
struct INTERNAL_WORK_QUEUE_ITEM {
    WorkItem: _WORK_QUEUE_ITEM,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    Callback: PDEVICE_CHANGE_COMPLETE_CALLBACK,
    Context: PVOID,
    NotificationStructure: _TARGET_DEVICE_CUSTOM_NOTIFICATION,
}

type PINTERNAL_WORK_QUEUE_ITEM = *mut INTERNAL_WORK_QUEUE_ITEM;

// Functions that are cdecl (no NTAPI in C declarations)
extern "C" {
    fn PnpRootCreateDevice(ServiceName: *mut _UNICODE_STRING, Pdo: *mut PDEVICE_OBJECT, InstancePath: *mut _UNICODE_STRING) -> NTSTATUS;
    fn PipAllocateDeviceNode(Pdo: PDEVICE_OBJECT) -> PDEVICE_NODE;
    fn PiInsertDevNode(DeviceNode: PDEVICE_NODE, Parent: PDEVICE_NODE);
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, State: u32);
    fn PnpRootRegisterDevice(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS;
    fn PiQueueDeviceAction(DeviceObject: PDEVICE_OBJECT, Action: u32, Request: PVOID, Callback: PVOID);
    fn ExInitializeWorkItem(WorkItem: PWORK_QUEUE_ITEM, Routine: unsafe extern "system" fn(PVOID), Context: PVOID);
    fn PiSetDevNodeText(DeviceNode: PDEVICE_NODE, InstanceKey: HANDLE);
    fn IopSetDeviceInstanceData(InstanceKey: HANDLE, DeviceNode: PDEVICE_NODE) -> NTSTATUS;
    fn IopDeviceNodeSetFlag(DeviceNode: PDEVICE_NODE, Flag: ULONG);
    fn PiNotifyTargetDeviceChange(Event: *const GUID, DeviceObject: PDEVICE_OBJECT, NotificationStructure: PTARGET_DEVICE_CUSTOM_NOTIFICATION);
}
// Functions that are NTAPI (stdcall on i386)
extern "system" {
    fn IopCreateDeviceKeyPath(InstancePath: *const _UNICODE_STRING, Options: ULONG, InstanceKey: *mut HANDLE) -> NTSTATUS;
    fn IopAssignDeviceResources(DeviceNode: PDEVICE_NODE) -> NTSTATUS;
    fn IopDetectResourceConflict(ResourceList: PCM_RESOURCE_LIST, Silent: BOOLEAN, ConflictingDevice: PDEVICE_OBJECT) -> NTSTATUS;
    fn IopIsValidPhysicalDeviceObject(PhysicalDeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    static mut IopRootDeviceNode: PDEVICE_NODE;
    static GUID_PNP_CUSTOM_NOTIFICATION: GUID;
    static GUID_TARGET_DEVICE_QUERY_REMOVE: GUID;
    static GUID_TARGET_DEVICE_REMOVE_CANCELLED: GUID;
    static GUID_TARGET_DEVICE_REMOVE_COMPLETE: GUID;
}

// Correct constant values from sdk/include/ndk/iotypes.h
const DNF_MADEUP: ULONG = 0x00000001;
const DNF_HAS_BOOT_CONFIG: ULONG = 0x00000040;
const DNF_NO_RESOURCE_REQUIRED: ULONG = 0x00000100;
const DNF_ENUMERATED: ULONG = 0x00000010;
const DeviceNodeStartPostWork: u32 = 0x307;
const PiActionEnumDeviceTree: u32 = 0; // first enum value in _DEVICE_ACTION
const DRVO_BUILTIN_DRIVER: u32 = 0x00000004;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const REG_MULTI_SZ: ULONG = 7;
const KEY_SET_VALUE: ACCESS_MASK = 0x0002;
const IO_NO_INCREMENT: i8 = 0;
const DelayedWorkQueue: ULONG = 1;
const PNP_DETECTED_FATAL_ERROR: ULONG = 0xCA;

const Internal: INTERFACE_TYPE = 0;
const Isa: INTERFACE_TYPE = 1;
const Eisa: INTERFACE_TYPE = 2;
const MicroChannel: INTERFACE_TYPE = 3;
const TurboChannel: INTERFACE_TYPE = 4;
const PCIBus: INTERFACE_TYPE = 5;
const VMEBus: INTERFACE_TYPE = 6;
const NuBus: INTERFACE_TYPE = 7;
const PCMCIABus: INTERFACE_TYPE = 8;
const CBus: INTERFACE_TYPE = 9;
const MPIBus: INTERFACE_TYPE = 10;
const MPSABus: INTERFACE_TYPE = 11;
const ProcessorInternal: INTERFACE_TYPE = 12;
const PNPISABus: INTERFACE_TYPE = 14;
const PNPBus: INTERFACE_TYPE = 15;
const Vmcs: INTERFACE_TYPE = 16;
const InterfaceTypeUndefined: INTERFACE_TYPE = -1i32 as INTERFACE_TYPE;

// Wide string literals for IopGetInterfaceTypeString
static IFACE_INTERNAL: [u16; 9] = [b'I' as u16, b'n' as u16, b't' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'a' as u16, b'l' as u16, 0];
static IFACE_ISA: [u16; 4] = [b'I' as u16, b's' as u16, b'a' as u16, 0];
static IFACE_EISA: [u16; 5] = [b'E' as u16, b'i' as u16, b's' as u16, b'a' as u16, 0];
static IFACE_MICROCHANNEL: [u16; 13] = [b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b'C' as u16, b'h' as u16, b'a' as u16, b'n' as u16, b'n' as u16, b'e' as u16, b'l' as u16, 0];
static IFACE_TURBOCHANNEL: [u16; 13] = [b'T' as u16, b'u' as u16, b'r' as u16, b'b' as u16, b'o' as u16, b'C' as u16, b'h' as u16, b'a' as u16, b'n' as u16, b'n' as u16, b'e' as u16, b'l' as u16, 0];
static IFACE_PCIBUS: [u16; 7] = [b'P' as u16, b'C' as u16, b'I' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_VMEBUS: [u16; 7] = [b'V' as u16, b'M' as u16, b'E' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_NUBUS: [u16; 6] = [b'N' as u16, b'u' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_PCMCIABUS: [u16; 10] = [b'P' as u16, b'C' as u16, b'M' as u16, b'C' as u16, b'I' as u16, b'A' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_CBUS: [u16; 5] = [b'C' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_MPIBUS: [u16; 7] = [b'M' as u16, b'P' as u16, b'I' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_MPSABUS: [u16; 8] = [b'M' as u16, b'P' as u16, b'S' as u16, b'A' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_PROCESSORINTERNAL: [u16; 18] = [b'P' as u16, b'r' as u16, b'o' as u16, b'c' as u16, b'e' as u16, b's' as u16, b's' as u16, b'o' as u16, b'r' as u16, b'I' as u16, b'n' as u16, b't' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'a' as u16, b'l' as u16, 0];
static IFACE_PNPISABUS: [u16; 10] = [b'P' as u16, b'N' as u16, b'P' as u16, b'I' as u16, b'S' as u16, b'A' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_PNPBUS: [u16; 7] = [b'P' as u16, b'N' as u16, b'P' as u16, b'B' as u16, b'u' as u16, b's' as u16, 0];
static IFACE_VMCS: [u16; 5] = [b'V' as u16, b'm' as u16, b'c' as u16, b's' as u16, 0];

// Wide string literals used with RtlInitUnicodeString (must be proper u16 arrays)
static W_SERVICE: [u16; 8] = [b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, 0];
static W_LEGACY: [u16; 7] = [b'L' as u16, b'e' as u16, b'g' as u16, b'a' as u16, b'c' as u16, b'y' as u16, 0];
static W_COMPATIBLE_IDS: [u16; 14] = [b'C' as u16, b'o' as u16, b'm' as u16, b'p' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'b' as u16, b'l' as u16, b'e' as u16, b'I' as u16, b'D' as u16, b's' as u16, 0];
// "DETECTED" as u16 array for hardware ID construction
static W_DETECTED: [u16; 9] = [b'D' as u16, b'E' as u16, b'T' as u16, b'E' as u16, b'C' as u16, b'T' as u16, b'E' as u16, b'D' as u16, 0];

/// Inline implementation of IsEqualGUID (C macro, no exported symbol)
#[inline]
unsafe fn IsEqualGUID(guid1: *const GUID, guid2: *const GUID) -> bool {
    (*guid1).Data1 == (*guid2).Data1 &&
    (*guid1).Data2 == (*guid2).Data2 &&
    (*guid1).Data3 == (*guid2).Data3 &&
    (*guid1).Data4 == (*guid2).Data4
}

// IopGetInterfaceTypeString is file-internal only (no NTAPI, not exported)
unsafe fn IopGetInterfaceTypeString(IfType: INTERFACE_TYPE) -> *mut u16 {
    match IfType {
        0 /* Internal */ => IFACE_INTERNAL.as_ptr() as *mut u16,
        1 /* Isa */ => IFACE_ISA.as_ptr() as *mut u16,
        2 /* Eisa */ => IFACE_EISA.as_ptr() as *mut u16,
        3 /* MicroChannel */ => IFACE_MICROCHANNEL.as_ptr() as *mut u16,
        4 /* TurboChannel */ => IFACE_TURBOCHANNEL.as_ptr() as *mut u16,
        5 /* PCIBus */ => IFACE_PCIBUS.as_ptr() as *mut u16,
        6 /* VMEBus */ => IFACE_VMEBUS.as_ptr() as *mut u16,
        7 /* NuBus */ => IFACE_NUBUS.as_ptr() as *mut u16,
        8 /* PCMCIABus */ => IFACE_PCMCIABUS.as_ptr() as *mut u16,
        9 /* CBus */ => IFACE_CBUS.as_ptr() as *mut u16,
        10 /* MPIBus */ => IFACE_MPIBUS.as_ptr() as *mut u16,
        11 /* MPSABus */ => IFACE_MPSABUS.as_ptr() as *mut u16,
        12 /* ProcessorInternal */ => IFACE_PROCESSORINTERNAL.as_ptr() as *mut u16,
        14 /* PNPISABus */ => IFACE_PNPISABUS.as_ptr() as *mut u16,
        15 /* PNPBus */ => IFACE_PNPBUS.as_ptr() as *mut u16,
        16 /* Vmcs */ => IFACE_VMCS.as_ptr() as *mut u16,
        _ => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn IopReportTargetDeviceChangeAsyncWorker(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let Item = Context as PINTERNAL_WORK_QUEUE_ITEM;
    PpSetCustomTargetEvent(
        (*Item).PhysicalDeviceObject,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (*Item).Callback,
        (*Item).Context,
        core::ptr::addr_of_mut!((*Item).NotificationStructure),
    );
    ObDereferenceObject((*Item).PhysicalDeviceObject as PVOID);
    ExFreePoolWithTag(Context, TAG_PNP);
}

unsafe fn PpSetCustomTargetEvent(
    DeviceObject: PDEVICE_OBJECT,
    SyncEvent: PKEVENT,
    SyncStatus: PNTSTATUS,
    _Callback: PDEVICE_CHANGE_COMPLETE_CALLBACK,
    _Context: PVOID,
    NotificationStructure: PTARGET_DEVICE_CUSTOM_NOTIFICATION,
) -> NTSTATUS {
    if !SyncEvent.is_null() {
        *SyncStatus = STATUS_PENDING;
    }

    PiNotifyTargetDeviceChange(
        core::ptr::addr_of!(GUID_PNP_CUSTOM_NOTIFICATION),
        DeviceObject,
        NotificationStructure,
    );

    if !SyncEvent.is_null() {
        KeSetEvent(SyncEvent as PVOID, 0i32, 0u8);
        *SyncStatus = STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoReportDetectedDevice(
    DriverObject: PDRIVER_OBJECT,
    LegacyBusType: INTERFACE_TYPE,
    _BusNumber: ULONG,
    _SlotNumber: ULONG,
    ResourceList: PCM_RESOURCE_LIST,
    ResourceRequirements: PIO_RESOURCE_REQUIREMENTS_LIST,
    ResourceAssigned: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let DriverObject = opaque_ptr(DriverObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    static CONTROL_STRING: [u16; 8] = [b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, 0];
    static DEVICE_REPORTED_STRING: [u16; 15] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b'p' as u16, b'o' as u16, b'r' as u16, b't' as u16, b'e' as u16, b'd' as u16, 0];

    let mut Control = _UNICODE_STRING {
        Length: 14,   // 7 chars * 2
        MaximumLength: 16, // 8 chars * 2
        Buffer: CONTROL_STRING.as_ptr() as *mut u16,
    };
    let mut DeviceReportedName = _UNICODE_STRING {
        Length: 28,   // 14 chars * 2
        MaximumLength: 30, // 15 chars * 2
        Buffer: DEVICE_REPORTED_STRING.as_ptr() as *mut u16,
    };

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LegacyBusType = LegacyBusType;
    let ServiceLongName = (*(*DriverObject).DriverExtension).ServiceKeyName;
    let mut ServiceName = ServiceLongName;

    if LegacyBusType == InterfaceTypeUndefined {
        LegacyBusType = Internal;
    }

    let IfString = IopGetInterfaceTypeString(LegacyBusType);

    if IfString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*DriverObject).Flags & DRVO_BUILTIN_DRIVER) != 0 {
        if !ServiceName.Buffer.is_null() && ServiceName.Length >= core::mem::size_of::<u16>() as u16 {
            let mut ValueName = _UNICODE_STRING {
                Length: 2,
                MaximumLength: 0,
                Buffer: ServiceName.Buffer.add((ServiceName.Length / 2) as usize - 1),
            };

            while ValueName.Buffer > ServiceName.Buffer && *ValueName.Buffer != b'\\' as u16 {
                ValueName.Buffer = ValueName.Buffer.sub(1);
                ValueName.Length += 2;
            }
            if *ValueName.Buffer == b'\\' as u16 {
                ValueName.Buffer = ValueName.Buffer.add(1);
                ValueName.Length -= 2;
            }

            ServiceName.MaximumLength = (ServiceName.MaximumLength as i32 - (ServiceName.Length as i32 - ValueName.Length as i32)) as u16;
            ServiceName.Length = ValueName.Length;
            ServiceName.Buffer = ValueName.Buffer;
        }
    }

    let mut instancePath: _UNICODE_STRING = core::mem::zeroed();
    let mut Pdo: PDEVICE_OBJECT = core::ptr::null_mut();
    if !DeviceObject.is_null() && !(*DeviceObject).is_null() {
        Pdo = *DeviceObject;
    } else {
        let Status = PnpRootCreateDevice(
            core::ptr::addr_of_mut!(ServiceName),
            core::ptr::addr_of_mut!(Pdo),
            core::ptr::addr_of_mut!(instancePath),
        );
        if Status < 0 {
            return Status;
        }
    }

    let DeviceNode = PipAllocateDeviceNode(Pdo);
    if DeviceNode.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceNode).InstancePath = instancePath;

    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut Status = IopCreateDeviceKeyPath(
        core::ptr::addr_of!((*DeviceNode).InstancePath),
        REG_OPTION_NON_VOLATILE,
        core::ptr::addr_of_mut!(InstanceKey),
    );
    if Status < 0 {
        return Status;
    }

    // "Service" as proper u16 wide string
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), W_SERVICE.as_ptr() as *mut u16);
    Status = ZwSetValueKey(
        InstanceKey,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_SZ,
        ServiceLongName.Buffer as PVOID,
        (ServiceLongName.Length as usize + core::mem::size_of::<u16>()) as ULONG,
    );

    // "Legacy" as proper u16 wide string
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), W_LEGACY.as_ptr() as *mut u16);
    let LegacyValue: ULONG = 0;
    Status = ZwSetValueKey(
        InstanceKey,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_DWORD,
        core::ptr::addr_of!(LegacyValue) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    let DeviceReported: ULONG = 1;
    Status = ZwSetValueKey(
        InstanceKey,
        core::ptr::addr_of_mut!(DeviceReportedName),
        0,
        REG_DWORD,
        core::ptr::addr_of!(DeviceReported) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = InstanceKey;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(Control);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut ControlKey: HANDLE = core::ptr::null_mut();
    Status = ZwCreateKey(
        core::ptr::addr_of_mut!(ControlKey),
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        Status = ZwSetValueKey(
            ControlKey,
            core::ptr::addr_of_mut!(DeviceReportedName),
            0,
            REG_DWORD,
            core::ptr::addr_of!(DeviceReported) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        ZwClose(ControlKey);
    }

    let mut HardwareId: [u16; 256] = [0; 256];
    let mut IdLength: usize = 0;

    let mut temp_buf: [u16; 128] = [0; 128];
    let len1 = swprintf_detected_interface(temp_buf.as_mut_ptr(), IfString, ServiceName.Buffer, ServiceName.Length);
    for i in 0..len1 {
        HardwareId[IdLength + i] = temp_buf[i];
    }
    IdLength += len1;
    HardwareId[IdLength] = 0;
    IdLength += 1;

    let len2 = swprintf_detected(temp_buf.as_mut_ptr(), ServiceName.Buffer, ServiceName.Length);
    for i in 0..len2 {
        HardwareId[IdLength + i] = temp_buf[i];
    }
    IdLength += len2;
    HardwareId[IdLength] = 0;
    IdLength += 1;

    HardwareId[IdLength] = 0;
    IdLength += 1;

    // "CompatibleIDs" as proper u16 wide string
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), W_COMPATIBLE_IDS.as_ptr() as *mut u16);
    Status = ZwSetValueKey(
        InstanceKey,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_MULTI_SZ,
        HardwareId.as_ptr() as PVOID,
        (IdLength * core::mem::size_of::<u16>()) as ULONG,
    );
    if Status < 0 {
        ZwClose(InstanceKey);
        return Status;
    }

    PiSetDevNodeText(DeviceNode, InstanceKey);

    (*DeviceNode).BootResources = ResourceList;
    (*DeviceNode).ResourceRequirements = ResourceRequirements;

    if !(*DeviceNode).BootResources.is_null() {
        IopDeviceNodeSetFlag(DeviceNode, DNF_HAS_BOOT_CONFIG);
    }

    if (*DeviceNode).ResourceRequirements.is_null() && (*DeviceNode).BootResources.is_null() {
        IopDeviceNodeSetFlag(DeviceNode, DNF_NO_RESOURCE_REQUIRED);
    }

    IopSetDeviceInstanceData(InstanceKey, DeviceNode);

    if ResourceAssigned == 0u8 {
        Status = IopAssignDeviceResources(DeviceNode);

        if Status < 0 {
            ZwClose(InstanceKey);
            return Status;
        }
    }

    ZwClose(InstanceKey);

    if !DeviceObject.is_null() && !(*DeviceObject).is_null() {
        PnpRootRegisterDevice(*DeviceObject);
    }

    PiInsertDevNode(DeviceNode, IopRootDeviceNode);
    (*DeviceNode).Flags |= DNF_MADEUP | DNF_ENUMERATED;

    PiSetDevNodeState(DeviceNode, DeviceNodeStartPostWork);

    PiQueueDeviceAction(Pdo, PiActionEnumDeviceTree, core::ptr::null_mut(), core::ptr::null_mut());

    if !DeviceObject.is_null() {
        *DeviceObject = Pdo;
    }

    STATUS_SUCCESS
}

/// Build "DETECTED<IfString>\<ServiceName>" into buf, using proper u16 wide chars.
unsafe fn swprintf_detected_interface(buf: *mut u16, ifstring: *const u16, service: *const u16, service_len: u16) -> usize {
    let mut idx = 0;
    // Copy "DETECTED" (8 u16 chars, no null)
    for i in 0..8 {
        *buf.add(idx) = W_DETECTED[i];
        idx += 1;
    }
    // Copy interface type string
    let mut i = 0;
    while *ifstring.add(i) != 0 {
        *buf.add(idx) = *ifstring.add(i);
        idx += 1;
        i += 1;
    }
    *buf.add(idx) = b'\\' as u16;
    idx += 1;
    // Copy service name
    let chars = service_len as usize / 2;
    for i in 0..chars {
        *buf.add(idx) = *service.add(i);
        idx += 1;
    }
    *buf.add(idx) = 0;
    idx
}

/// Build "DETECTED\<ServiceName>" into buf, using proper u16 wide chars.
unsafe fn swprintf_detected(buf: *mut u16, service: *const u16, service_len: u16) -> usize {
    let mut idx = 0;
    // Copy "DETECTED" (8 u16 chars, no null)
    for i in 0..8 {
        *buf.add(idx) = W_DETECTED[i];
        idx += 1;
    }
    *buf.add(idx) = b'\\' as u16;
    idx += 1;
    // Copy service name
    let chars = service_len as usize / 2;
    for i in 0..chars {
        *buf.add(idx) = *service.add(i);
        idx += 1;
    }
    *buf.add(idx) = 0;
    idx
}

#[no_mangle]
pub unsafe extern "system" fn IoReportResourceForDetection(
    _DriverObject: PDRIVER_OBJECT,
    DriverList: PCM_RESOURCE_LIST,
    _DriverListSize: ULONG,
    _DeviceObject: PDEVICE_OBJECT,
    DeviceList: PCM_RESOURCE_LIST,
    _DeviceListSize: ULONG,
    ConflictDetected: *mut BOOLEAN,
) -> NTSTATUS {
    let _DriverObject = opaque_ptr(_DriverObject);
    let _DeviceObject = opaque_ptr(_DeviceObject);
    let ConflictDetected = opaque_ptr(ConflictDetected);
    *ConflictDetected = 0u8;

    if DriverList.is_null() && DeviceList.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let ResourceList = if DriverList.is_null() { DeviceList } else { DriverList };

    let Status = IopDetectResourceConflict(ResourceList, 1u8, core::ptr::null_mut());
    if Status == STATUS_CONFLICTING_ADDRESSES {
        *ConflictDetected = 1u8;
    } else if Status >= 0 {
        // TODO: Claim the resources in the ResourceMap
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopSetEvent(Context: PVOID) {
    let Context = opaque_ptr(Context);
    KeSetEvent(Context, 0i32, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn IoReportTargetDeviceChange(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    NotificationStructure: PVOID,
) -> NTSTATUS {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let NotificationStructure = opaque_ptr(NotificationStructure);
    let notifyStruct = NotificationStructure as PTARGET_DEVICE_CUSTOM_NOTIFICATION;

    if IopIsValidPhysicalDeviceObject(PhysicalDeviceObject) == 0u8 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, PhysicalDeviceObject as usize, 0, 0);
    }

    if IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_QUERY_REMOVE)) ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_CANCELLED)) ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_COMPLETE)) {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (*notifyStruct).Version != 1 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let mut NotifyEvent: _KEVENT = core::mem::zeroed();
    KeInitializeEvent(core::ptr::addr_of_mut!(NotifyEvent), _EVENT_TYPE_NotificationEvent, 0u8);

    let mut NotifyStatus: NTSTATUS = STATUS_SUCCESS;
    let Status = PpSetCustomTargetEvent(
        PhysicalDeviceObject,
        core::ptr::addr_of_mut!(NotifyEvent),
        core::ptr::addr_of_mut!(NotifyStatus),
        None,
        core::ptr::null_mut(),
        notifyStruct,
    );

    if Status >= 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(NotifyEvent) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        return NotifyStatus;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoReportTargetDeviceChangeAsynchronous(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    NotificationStructure: PVOID,
    Callback: PDEVICE_CHANGE_COMPLETE_CALLBACK,
    Context: PVOID,
) -> NTSTATUS {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let NotificationStructure = opaque_ptr(NotificationStructure);
    let Context = opaque_ptr(Context);
    let notifyStruct = NotificationStructure as PTARGET_DEVICE_CUSTOM_NOTIFICATION;

    if IopIsValidPhysicalDeviceObject(PhysicalDeviceObject) == 0u8 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, PhysicalDeviceObject as usize, 0, 0);
    }

    if IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_QUERY_REMOVE)) ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_CANCELLED)) ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_COMPLETE)) {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if (*notifyStruct).Version != 1 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let WorkItemSize = core::mem::offset_of!(INTERNAL_WORK_QUEUE_ITEM, NotificationStructure) + (*notifyStruct).Size as usize;

    let Item = ExAllocatePoolWithTag(NonPagedPool, WorkItemSize, TAG_PNP);
    if Item.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Item = Item as PINTERNAL_WORK_QUEUE_ITEM;

    ObReferenceObject(PhysicalDeviceObject as PVOID);
    (*Item).PhysicalDeviceObject = PhysicalDeviceObject;
    (*Item).Callback = Callback;
    (*Item).Context = Context;
    core::ptr::copy_nonoverlapping(
        notifyStruct as *const u8,
        core::ptr::addr_of_mut!((*Item).NotificationStructure) as *mut u8,
        (*notifyStruct).Size as usize,
    );
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!((*Item).WorkItem),
        IopReportTargetDeviceChangeAsyncWorker,
        Item as PVOID,
    );

    ExQueueWorkItem(core::ptr::addr_of_mut!((*Item).WorkItem), DelayedWorkQueue);

    STATUS_PENDING
}
