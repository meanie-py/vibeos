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

use ntkrnl_sys::ddk::{
    _EPROCESS, _KEVENT, _OBJECT_ATTRIBUTES, _TARGET_DEVICE_CUSTOM_NOTIFICATION, _WORK_QUEUE_ITEM,
    _UNICODE_STRING, GUID, _EVENT_TYPE_NotificationEvent, _KWAIT_REASON_Executive,
    PEPROCESS, PKEVENT, POBJECT_ATTRIBUTES, PTARGET_DEVICE_CUSTOM_NOTIFICATION,
    PWORK_QUEUE_ITEM, PDEVICE_OBJECT, PDRIVER_OBJECT, PCM_RESOURCE_LIST,
    PIO_RESOURCE_REQUIREMENTS_LIST, PDEVICE_CHANGE_COMPLETE_CALLBACK, PDEVICE_NODE,
    INTERFACE_TYPE, HANDLE, KPROCESSOR_MODE, ACCESS_MASK, PNTSTATUS,
    ExQueueWorkItem, KeInitializeEvent, KeWaitForSingleObject, RtlInitUnicodeString,
    ZwClose, ZwCreateKey, ZwSetValueKey,
};

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

extern "system" {
    fn IopSetDeviceInstanceData(InstanceKey: HANDLE, DeviceNode: PDEVICE_NODE) -> NTSTATUS;
    fn IopGetInterfaceTypeString(IfType: INTERFACE_TYPE) -> *mut u16;
    fn PnpRootCreateDevice(ServiceName: *mut _UNICODE_STRING, Pdo: *mut PDEVICE_OBJECT, InstancePath: *mut _UNICODE_STRING) -> NTSTATUS;
    fn PipAllocateDeviceNode(Pdo: PDEVICE_OBJECT) -> PDEVICE_NODE;
    fn IopCreateDeviceKeyPath(InstancePath: *const _UNICODE_STRING, Options: ULONG, InstanceKey: *mut HANDLE) -> NTSTATUS;
    fn PiSetDevNodeText(DeviceNode: PDEVICE_NODE, InstanceKey: HANDLE);
    fn IopDeviceNodeSetFlag(DeviceNode: PDEVICE_NODE, Flag: ULONG);
    fn IopAssignDeviceResources(DeviceNode: PDEVICE_NODE) -> NTSTATUS;
    fn PnpRootRegisterDevice(DeviceObject: PDEVICE_OBJECT);
    fn PiInsertDevNode(DeviceNode: PDEVICE_NODE, Parent: PDEVICE_NODE);
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, State: ULONG);
    fn PiQueueDeviceAction(DeviceObject: PDEVICE_OBJECT, Action: ULONG, Request: PVOID, Callback: PVOID);
    fn IopDetectResourceConflict(ResourceList: PCM_RESOURCE_LIST, Silent: BOOLEAN, ConflictingDevice: PDEVICE_OBJECT) -> NTSTATUS;
    fn IopIsValidPhysicalDeviceObject(PhysicalDeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    fn PiNotifyTargetDeviceChange(Event: *const GUID, DeviceObject: PDEVICE_OBJECT, NotificationStructure: PTARGET_DEVICE_CUSTOM_NOTIFICATION);
    fn IsEqualGUID(guid1: *const GUID, guid2: *const GUID) -> BOOLEAN;
    fn ExInitializeWorkItem(WorkItem: PWORK_QUEUE_ITEM, Routine: unsafe extern "system" fn(PVOID), Context: PVOID);
    static mut IopRootDeviceNode: PDEVICE_NODE;
    static GUID_PNP_CUSTOM_NOTIFICATION: GUID;
    static GUID_TARGET_DEVICE_QUERY_REMOVE: GUID;
    static GUID_TARGET_DEVICE_REMOVE_CANCELLED: GUID;
    static GUID_TARGET_DEVICE_REMOVE_COMPLETE: GUID;
}

const DNF_HAS_BOOT_CONFIG: ULONG = 0x00000001;
const DNF_NO_RESOURCE_REQUIRED: ULONG = 0x00000002;
const DNF_MADEUP: ULONG = 0x00000004;
const DNF_ENUMERATED: ULONG = 0x00000008;
const DeviceNodeStartPostWork: ULONG = 0x00000010;
const PiActionEnumDeviceTree: ULONG = 0x00000001;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const REG_MULTI_SZ: ULONG = 7;
const KEY_SET_VALUE: ACCESS_MASK = 0x0002;
const IO_NO_INCREMENT: i8 = 0;
const DelayedWorkQueue: ULONG = 1;
const NotificationEvent: ULONG = 1;
const Executive: ULONG = 0;
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

#[no_mangle]
pub unsafe extern "system" fn IopReportTargetDeviceChangeAsyncWorker(Context: PVOID) {
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
    Callback: PDEVICE_CHANGE_COMPLETE_CALLBACK,
    Context: PVOID,
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
    BusNumber: ULONG,
    SlotNumber: ULONG,
    ResourceList: PCM_RESOURCE_LIST,
    ResourceRequirements: PIO_RESOURCE_REQUIREMENTS_LIST,
    ResourceAssigned: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    static mut CONTROL_STRING: [u16; 8] = [b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, 0];
    static mut DEVICE_REPORTED_STRING: [u16; 15] = [b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'R' as u16, b'e' as u16, b'p' as u16, b'o' as u16, b'r' as u16, b't' as u16, b'e' as u16, b'd' as u16, 0];

    let mut Control = _UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: CONTROL_STRING.as_mut_ptr(),
    };
    let mut DeviceReportedName = _UNICODE_STRING {
        Length: 28,
        MaximumLength: 30,
        Buffer: DEVICE_REPORTED_STRING.as_mut_ptr(),
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

    if ((*DriverObject).Flags & 0x00000001) != 0 {
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

    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), b"S\0e\0r\0v\0i\0c\0e\0\0\0".as_ptr() as *mut u16);
    Status = ZwSetValueKey(
        InstanceKey,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_SZ,
        ServiceLongName.Buffer as PVOID,
        (ServiceLongName.Length as usize + core::mem::size_of::<u16>()) as ULONG,
    );

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), b"L\0e\0g\0a\0c\0y\0\0\0".as_ptr() as *mut u16);
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

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), b"C\0o\0m\0p\0a\0t\0i\0b\0l\0e\0I\0D\0s\0\0\0".as_ptr() as *mut u16);
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

unsafe fn swprintf_detected_interface(buf: *mut u16, ifstring: *const u16, service: *const u16, service_len: u16) -> usize {
    let detected = b"D\0E\0T\0E\0C\0T\0E\0D\0".as_ptr() as *const u16;
    let mut idx = 0;
    for i in 0..8 {
        *buf.add(idx) = *detected.add(i);
        idx += 1;
    }
    let mut i = 0;
    while *ifstring.add(i) != 0 {
        *buf.add(idx) = *ifstring.add(i);
        idx += 1;
        i += 1;
    }
    *buf.add(idx) = b'\\' as u16;
    idx += 1;
    let chars = service_len as usize / 2;
    for i in 0..chars {
        *buf.add(idx) = *service.add(i);
        idx += 1;
    }
    *buf.add(idx) = 0;
    idx
}

unsafe fn swprintf_detected(buf: *mut u16, service: *const u16, service_len: u16) -> usize {
    let detected = b"D\0E\0T\0E\0C\0T\0E\0D\0".as_ptr() as *const u16;
    let mut idx = 0;
    for i in 0..8 {
        *buf.add(idx) = *detected.add(i);
        idx += 1;
    }
    *buf.add(idx) = b'\\' as u16;
    idx += 1;
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
    DriverObject: PDRIVER_OBJECT,
    DriverList: PCM_RESOURCE_LIST,
    DriverListSize: ULONG,
    DeviceObject: PDEVICE_OBJECT,
    DeviceList: PCM_RESOURCE_LIST,
    DeviceListSize: ULONG,
    ConflictDetected: *mut BOOLEAN,
) -> NTSTATUS {
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
    KeSetEvent(Context, 0i32, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn IoReportTargetDeviceChange(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    NotificationStructure: PVOID,
) -> NTSTATUS {
    let notifyStruct = NotificationStructure as PTARGET_DEVICE_CUSTOM_NOTIFICATION;

    if IopIsValidPhysicalDeviceObject(PhysicalDeviceObject) == 0u8 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, PhysicalDeviceObject as usize, 0, 0);
    }

    if IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_QUERY_REMOVE)) != 0u8 ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_CANCELLED)) != 0u8 ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_COMPLETE)) != 0u8 {
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
    let notifyStruct = NotificationStructure as PTARGET_DEVICE_CUSTOM_NOTIFICATION;

    if IopIsValidPhysicalDeviceObject(PhysicalDeviceObject) == 0u8 {
        KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, PhysicalDeviceObject as usize, 0, 0);
    }

    if IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_QUERY_REMOVE)) != 0u8 ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_CANCELLED)) != 0u8 ||
       IsEqualGUID(core::ptr::addr_of!((*notifyStruct).Event), core::ptr::addr_of!(GUID_TARGET_DEVICE_REMOVE_COMPLETE)) != 0u8 {
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


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnpreport.c
//  * PURPOSE:         Device Changes Reporting Functions
//  * PROGRAMMERS:     Cameron Gutman (cameron.gutman@reactos.org)
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *******************************************************************/
//
// typedef struct _INTERNAL_WORK_QUEUE_ITEM
// {
//     WORK_QUEUE_ITEM WorkItem;
//     PDEVICE_OBJECT PhysicalDeviceObject;
//     PDEVICE_CHANGE_COMPLETE_CALLBACK Callback;
//     PVOID Context;
//     TARGET_DEVICE_CUSTOM_NOTIFICATION NotificationStructure;
// } INTERNAL_WORK_QUEUE_ITEM, *PINTERNAL_WORK_QUEUE_ITEM;
//
// NTSTATUS
// IopSetDeviceInstanceData(HANDLE InstanceKey,
//                          PDEVICE_NODE DeviceNode);
//
// NTSTATUS
// PpSetCustomTargetEvent(IN PDEVICE_OBJECT DeviceObject,
//                        IN OUT PKEVENT SyncEvent OPTIONAL,
//                        IN OUT PNTSTATUS SyncStatus OPTIONAL,
//                        IN PDEVICE_CHANGE_COMPLETE_CALLBACK Callback OPTIONAL,
//                        IN PVOID Context OPTIONAL,
//                        IN PTARGET_DEVICE_CUSTOM_NOTIFICATION NotificationStructure);
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PWCHAR
// IopGetInterfaceTypeString(INTERFACE_TYPE IfType)
// {
//     switch (IfType)
//     {
//         case Internal:
//             return L"Internal";
//
//         case Isa:
//             return L"Isa";
//
//         case Eisa:
//             return L"Eisa";
//
//         case MicroChannel:
//             return L"MicroChannel";
//
//         case TurboChannel:
//             return L"TurboChannel";
//
//         case PCIBus:
//             return L"PCIBus";
//
//         case VMEBus:
//             return L"VMEBus";
//
//         case NuBus:
//             return L"NuBus";
//
//         case PCMCIABus:
//             return L"PCMCIABus";
//
//         case CBus:
//             return L"CBus";
//
//         case MPIBus:
//             return L"MPIBus";
//
//         case MPSABus:
//             return L"MPSABus";
//
//         case ProcessorInternal:
//             return L"ProcessorInternal";
//
//         case PNPISABus:
//             return L"PNPISABus";
//
//         case PNPBus:
//             return L"PNPBus";
//
//         case Vmcs:
//             return L"Vmcs";
//
//         default:
//             DPRINT1("Invalid bus type: %d\n", IfType);
//             return NULL;
//     }
// }
//
// VOID
// NTAPI
// IopReportTargetDeviceChangeAsyncWorker(PVOID Context)
// {
//     PINTERNAL_WORK_QUEUE_ITEM Item;
//
//     Item = (PINTERNAL_WORK_QUEUE_ITEM)Context;
//     PpSetCustomTargetEvent(Item->PhysicalDeviceObject, NULL, NULL, Item->Callback, Item->Context, &Item->NotificationStructure);
//     ObDereferenceObject(Item->PhysicalDeviceObject);
//     ExFreePoolWithTag(Context, '  pP');
// }
//
// NTSTATUS
// PpSetCustomTargetEvent(IN PDEVICE_OBJECT DeviceObject,
//                        IN OUT PKEVENT SyncEvent OPTIONAL,
//                        IN OUT PNTSTATUS SyncStatus OPTIONAL,
//                        IN PDEVICE_CHANGE_COMPLETE_CALLBACK Callback OPTIONAL,
//                        IN PVOID Context OPTIONAL,
//                        IN PTARGET_DEVICE_CUSTOM_NOTIFICATION NotificationStructure)
// {
//     ASSERT(NotificationStructure != NULL);
//     ASSERT(DeviceObject != NULL);
//
//     if (SyncEvent)
//     {
//         ASSERT(SyncStatus);
//         *SyncStatus = STATUS_PENDING;
//     }
//
//     /* That call is totally wrong but notifications handler must be fixed first */
//     PiNotifyTargetDeviceChange(&GUID_PNP_CUSTOM_NOTIFICATION, DeviceObject, NotificationStructure);
//
//     if (SyncEvent)
//     {
//         KeSetEvent(SyncEvent, IO_NO_INCREMENT, FALSE);
//         *SyncStatus = STATUS_SUCCESS;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoReportDetectedDevice(
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_ INTERFACE_TYPE LegacyBusType,
//     _In_ ULONG BusNumber,
//     _In_ ULONG SlotNumber,
//     _In_opt_ PCM_RESOURCE_LIST ResourceList,
//     _In_opt_ PIO_RESOURCE_REQUIREMENTS_LIST ResourceRequirements,
//     _In_ BOOLEAN ResourceAssigned,
//     _Inout_ PDEVICE_OBJECT *DeviceObject)
// {
//     UNICODE_STRING Control = RTL_CONSTANT_STRING(L"Control");
//     UNICODE_STRING DeviceReportedName = RTL_CONSTANT_STRING(L"DeviceReported");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PDEVICE_NODE DeviceNode;
//     PDEVICE_OBJECT Pdo;
//     NTSTATUS Status;
//     HANDLE InstanceKey, ControlKey;
//     UNICODE_STRING ValueName, ServiceLongName, ServiceName;
//     WCHAR HardwareId[256];
//     PWCHAR IfString;
//     ULONG IdLength;
//     ULONG LegacyValue;
//     ULONG DeviceReported = 1;
//
//     DPRINT("IoReportDetectedDevice (DeviceObject %p, *DeviceObject %p)\n",
//            DeviceObject, DeviceObject ? *DeviceObject : NULL);
//
//     ServiceLongName = DriverObject->DriverExtension->ServiceKeyName;
//     ServiceName = ServiceLongName;
//
//     /* If the interface type is unknown, treat it as internal */
//     if (LegacyBusType == InterfaceTypeUndefined)
//         LegacyBusType = Internal;
//
//     /* Get the string equivalent of the interface type */
//     IfString = IopGetInterfaceTypeString(LegacyBusType);
//
//     /* If NULL is returned then it's a bad type */
//     if (!IfString)
//         return STATUS_INVALID_PARAMETER;
//
//     /*
//      * Drivers that have been created via a direct IoCreateDriver() call
//      * have their ServiceKeyName set to \Driver\DriverName. We need to
//      * strip everything up to the last path separator and keep what remains.
//      */
//     if (DriverObject->Flags & DRVO_BUILTIN_DRIVER)
//     {
//         /*
//          * Find the last path separator.
//          * NOTE: Since ServiceName is not necessarily NULL-terminated,
//          * we cannot use wcsrchr().
//          */
//         if (ServiceName.Buffer && ServiceName.Length >= sizeof(WCHAR))
//         {
//             ValueName.Length = 1;
//             ValueName.Buffer = ServiceName.Buffer + (ServiceName.Length / sizeof(WCHAR)) - 1;
//
//             while ((ValueName.Buffer > ServiceName.Buffer) && (*ValueName.Buffer != L'\\'))
//             {
//                 --ValueName.Buffer;
//                 ++ValueName.Length;
//             }
//             if (*ValueName.Buffer == L'\\')
//             {
//                 ++ValueName.Buffer;
//                 --ValueName.Length;
//             }
//             ValueName.Length *= sizeof(WCHAR);
//
//             /* Shorten the string */
//             ServiceName.MaximumLength -= (ServiceName.Length - ValueName.Length);
//             ServiceName.Length = ValueName.Length;
//             ServiceName.Buffer = ValueName.Buffer;
//         }
//     }
//
//     /* We use the caller's PDO if they supplied one */
//     UNICODE_STRING instancePath = {0};
//     if (DeviceObject && *DeviceObject)
//     {
//         Pdo = *DeviceObject;
//     }
//     else
//     {
//         /* Create the PDO */
//         Status = PnpRootCreateDevice(&ServiceName, &Pdo, &instancePath);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("PnpRootCreateDevice() failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     /* Create the device node for the new PDO */
//     DeviceNode = PipAllocateDeviceNode(Pdo);
//     if (!DeviceNode)
//     {
//         DPRINT("PipAllocateDeviceNode() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // The string comes from PnpRootCreateDevice, so it can be used right away
//     DeviceNode->InstancePath = instancePath;
//
//     /* Open a handle to the instance path key */
//     Status = IopCreateDeviceKeyPath(&DeviceNode->InstancePath, REG_OPTION_NON_VOLATILE, &InstanceKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Save the driver name */
//     RtlInitUnicodeString(&ValueName, L"Service");
//     Status = ZwSetValueKey(InstanceKey, &ValueName, 0, REG_SZ, ServiceLongName.Buffer, ServiceLongName.Length + sizeof(UNICODE_NULL));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to write the Service name value: 0x%x\n", Status);
//     }
//
//     /* Report as non-legacy driver */
//     RtlInitUnicodeString(&ValueName, L"Legacy");
//     LegacyValue = 0;
//     Status = ZwSetValueKey(InstanceKey, &ValueName, 0, REG_DWORD, &LegacyValue, sizeof(LegacyValue));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to write the Legacy value: 0x%x\n", Status);
//     }
//     Status = ZwSetValueKey(InstanceKey, &DeviceReportedName, 0, REG_DWORD, &DeviceReported, sizeof(DeviceReported));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to write the DeviceReported value: 0x%x\n", Status);
//     }
//
//     /* Set DeviceReported=1 in Control subkey */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Control,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                InstanceKey,
//                                NULL);
//     Status = ZwCreateKey(&ControlKey,
//                          KEY_SET_VALUE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     if (NT_SUCCESS(Status))
//     {
//         Status = ZwSetValueKey(ControlKey,
//                                &DeviceReportedName,
//                                0,
//                                REG_DWORD,
//                                &DeviceReported,
//                                sizeof(DeviceReported));
//         ZwClose(ControlKey);
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to set ReportedDevice=1 for device %wZ (status 0x%08lx)\n", &instancePath, Status);
//     }
//
//     /* Add DETECTEDInterfaceType\DriverName */
//     IdLength = 0;
//     IdLength += swprintf(&HardwareId[IdLength],
//                          L"DETECTED%ls\\%wZ",
//                          IfString,
//                          &ServiceName);
//     IdLength++;
//
//     /* Add DETECTED\DriverName */
//     IdLength += swprintf(&HardwareId[IdLength],
//                          L"DETECTED\\%wZ",
//                          &ServiceName);
//     IdLength++;
//
//     /* Terminate the string with another null */
//     HardwareId[IdLength++] = UNICODE_NULL;
//
//     /* Store the value for CompatibleIDs */
//     RtlInitUnicodeString(&ValueName, L"CompatibleIDs");
//     Status = ZwSetValueKey(InstanceKey, &ValueName, 0, REG_MULTI_SZ, HardwareId, IdLength * sizeof(WCHAR));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("Failed to write the compatible IDs: 0x%x\n", Status);
//         ZwClose(InstanceKey);
//         return Status;
//     }
//
//     // Set the device's DeviceDesc and LocationInformation fields
//     PiSetDevNodeText(DeviceNode, InstanceKey);
//
//     /* Assign the resources to the device node */
//     DeviceNode->BootResources = ResourceList;
//     DeviceNode->ResourceRequirements = ResourceRequirements;
//
//     /* Set appropriate flags */
//     if (DeviceNode->BootResources)
//         IopDeviceNodeSetFlag(DeviceNode, DNF_HAS_BOOT_CONFIG);
//
//     if (!DeviceNode->ResourceRequirements && !DeviceNode->BootResources)
//         IopDeviceNodeSetFlag(DeviceNode, DNF_NO_RESOURCE_REQUIRED);
//
//     /* Write the resource information to the registry */
//     IopSetDeviceInstanceData(InstanceKey, DeviceNode);
//
//     /* If the caller didn't get the resources assigned for us, do it now */
//     if (!ResourceAssigned)
//     {
//         Status = IopAssignDeviceResources(DeviceNode);
//
//         /* See if we failed */
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("Assigning resources failed: 0x%x\n", Status);
//             ZwClose(InstanceKey);
//             return Status;
//         }
//     }
//
//     /* Close the instance key handle */
//     ZwClose(InstanceKey);
//
//     /* Register the given DO with PnP root if required */
//     if (DeviceObject && *DeviceObject)
//         PnpRootRegisterDevice(*DeviceObject);
//
//     PiInsertDevNode(DeviceNode, IopRootDeviceNode);
//     DeviceNode->Flags |= DNF_MADEUP | DNF_ENUMERATED;
//
//     // we still need to query IDs, send events and reenumerate this node
//     PiSetDevNodeState(DeviceNode, DeviceNodeStartPostWork);
//
//     DPRINT("Reported device: %S (%wZ)\n", HardwareId, &DeviceNode->InstancePath);
//
//     PiQueueDeviceAction(Pdo, PiActionEnumDeviceTree, NULL, NULL);
//
//     /* Return the PDO */
//     if (DeviceObject) *DeviceObject = Pdo;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @halfplemented
//  */
// NTSTATUS
// NTAPI
// IoReportResourceForDetection(IN PDRIVER_OBJECT DriverObject,
//                              IN PCM_RESOURCE_LIST DriverList OPTIONAL,
//                              IN ULONG DriverListSize OPTIONAL,
//                              IN PDEVICE_OBJECT DeviceObject OPTIONAL,
//                              IN PCM_RESOURCE_LIST DeviceList OPTIONAL,
//                              IN ULONG DeviceListSize OPTIONAL,
//                              OUT PBOOLEAN ConflictDetected)
// {
//     PCM_RESOURCE_LIST ResourceList;
//     NTSTATUS Status;
//
//     *ConflictDetected = FALSE;
//
//     if (!DriverList && !DeviceList)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Find the real list */
//     if (!DriverList)
//         ResourceList = DeviceList;
//     else
//         ResourceList = DriverList;
//
//     /* Look for a resource conflict */
//     Status = IopDetectResourceConflict(ResourceList, TRUE, NULL);
//     if (Status == STATUS_CONFLICTING_ADDRESSES)
//     {
//         /* Oh noes */
//         *ConflictDetected = TRUE;
//     }
//     else if (NT_SUCCESS(Status))
//     {
//         /* Looks like we're good to go */
//
//         /* TODO: Claim the resources in the ResourceMap */
//     }
//
//     return Status;
// }
//
// VOID
// NTAPI
// IopSetEvent(IN PVOID Context)
// {
//     PKEVENT Event = Context;
//
//     /* Set the event */
//     KeSetEvent(Event, IO_NO_INCREMENT, FALSE);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoReportTargetDeviceChange(IN PDEVICE_OBJECT PhysicalDeviceObject,
//                            IN PVOID NotificationStructure)
// {
//     KEVENT NotifyEvent;
//     NTSTATUS Status, NotifyStatus;
//     PTARGET_DEVICE_CUSTOM_NOTIFICATION notifyStruct = (PTARGET_DEVICE_CUSTOM_NOTIFICATION)NotificationStructure;
//
//     ASSERT(notifyStruct);
//
//     /* Check for valid PDO */
//     if (!IopIsValidPhysicalDeviceObject(PhysicalDeviceObject))
//     {
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)PhysicalDeviceObject, 0, 0);
//     }
//
//     /* FileObject must be null. PnP will fill in it */
//     ASSERT(notifyStruct->FileObject == NULL);
//
//     /* Do not handle system PnP events */
//     if (IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_QUERY_REMOVE) ||
//         IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_REMOVE_CANCELLED) ||
//         IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_REMOVE_COMPLETE))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     if (notifyStruct->Version != 1)
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Initialize even that will let us know when PnP will have finished notify */
//     KeInitializeEvent(&NotifyEvent, NotificationEvent, FALSE);
//
//     Status = PpSetCustomTargetEvent(PhysicalDeviceObject, &NotifyEvent, &NotifyStatus, NULL, NULL, notifyStruct);
//     /* If no error, wait for the notify to end and return the status of the notify and not of the event */
//     if (NT_SUCCESS(Status))
//     {
//         KeWaitForSingleObject(&NotifyEvent, Executive, KernelMode, FALSE, NULL);
//         Status = NotifyStatus;
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoReportTargetDeviceChangeAsynchronous(IN PDEVICE_OBJECT PhysicalDeviceObject,
//                                        IN PVOID NotificationStructure,
//                                        IN PDEVICE_CHANGE_COMPLETE_CALLBACK Callback OPTIONAL,
//                                        IN PVOID Context OPTIONAL)
// {
//     PINTERNAL_WORK_QUEUE_ITEM Item = NULL;
//     PTARGET_DEVICE_CUSTOM_NOTIFICATION notifyStruct = (PTARGET_DEVICE_CUSTOM_NOTIFICATION)NotificationStructure;
//     SIZE_T WorkItemSize;
//
//     ASSERT(notifyStruct);
//
//     /* Check for valid PDO */
//     if (!IopIsValidPhysicalDeviceObject(PhysicalDeviceObject))
//     {
//         KeBugCheckEx(PNP_DETECTED_FATAL_ERROR, 0x2, (ULONG_PTR)PhysicalDeviceObject, 0, 0);
//     }
//
//     /* FileObject must be null. PnP will fill in it */
//     ASSERT(notifyStruct->FileObject == NULL);
//
//     /* Do not handle system PnP events */
//     if (IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_QUERY_REMOVE) ||
//         IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_REMOVE_CANCELLED) ||
//         IsEqualGUID(&(notifyStruct->Event), &GUID_TARGET_DEVICE_REMOVE_COMPLETE))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     if (notifyStruct->Version != 1)
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Calculate the required size for the work item and notification structure */
//     WorkItemSize = FIELD_OFFSET(INTERNAL_WORK_QUEUE_ITEM, NotificationStructure)
//                    + notifyStruct->Size;
//
//     /* We need to store all the data given by the caller with the WorkItem, so use our own struct */
//     Item = ExAllocatePoolWithTag(NonPagedPool, WorkItemSize, '  pP');
//     if (!Item) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Initialize all stuff */
//     ObReferenceObject(PhysicalDeviceObject);
//     Item->PhysicalDeviceObject = PhysicalDeviceObject;
//     Item->Callback = Callback;
//     Item->Context = Context;
//     RtlCopyMemory(&Item->NotificationStructure, notifyStruct, notifyStruct->Size);
//     ExInitializeWorkItem(&(Item->WorkItem), IopReportTargetDeviceChangeAsyncWorker, Item);
//
//     /* Finally, queue the item, our work here is done */
//     ExQueueWorkItem(&(Item->WorkItem), DelayedWorkQueue);
//
//     return STATUS_PENDING;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
