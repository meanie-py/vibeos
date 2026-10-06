use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_CAPABILITIES, DEVICE_RELATIONS, DEVICE_POWER_STATE, DEVICE_TEXT_TYPE, ExFreePool, _KWAIT_REASON_Executive, GUID, IoAcquireCancelSpinLock, IoAllocateIrp, IoCancelIrp, IoCopyCurrentIrpStackLocationToNext, IoDeleteDevice, IoDetachDevice, IoFreeIrp, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoInvalidateDeviceRelations, IoMarkIrpPending, IoRegisterDeviceInterface, IoReleaseCancelSpinLock, IoSetCancelRoutine, IoSetCompletionRoutine, IoSetDeviceInterfaceState, KEVENT, KeDelayExecutionThread, KeInitializeEvent, KeInitializeSemaphore, KeInitializeSpinLock, KeReleaseSemaphore, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, _EVENT_TYPE_NotificationEvent, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PINTERFACE, PIO_STACK_LOCATION, PIRP, PNP_BUS_INFORMATION, PPNP_BUS_INFORMATION, PRKEVENT, _KWAIT_REASON_Suspended};
// PNP/Power/text-type enum members (bindgen names them `_ENUM_Member`)
use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations as BusRelations;
use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation;
use ntkrnl_sys::ddk::DEVICE_RELATION_TYPE;
use ntkrnl_sys::ddk::_DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0;
use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking;
use ntkrnl_sys::ddk::_DEVICE_TEXT_TYPE_DeviceTextDescription as DeviceTextDescription;
use ntkrnl_sys::ddk::_DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation;
// USB enum members from the crate's bindgen types
use crate::types::_USB_DEVICE_TYPE_Usb11Device as Usb11Device;
use crate::types::_USB_DEVICE_TYPE_Usb20Device as Usb20Device;
use crate::types::_USB_CONNECTION_STATUS_NoDeviceConnected as NoDeviceConnected;
use crate::types::_USB_CONNECTION_STATUS_DeviceHubNestedTooDeeply as DeviceHubNestedTooDeeply;
use core::ffi::c_void;
// Registry helpers used by the reconstructed USBH_SetPdoRegistryParameter (bindgen home: ddk).
use ntkrnl_sys::ddk::{RtlInitUnicodeString, IoOpenDeviceRegistryKey, ZwSetValueKey, ZwClose, HANDLE, UNICODE_STRING};

// sdk/include/psdk/usbiodef.h:76-77 — DEFINE_GUID device-interface class GUIDs.

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

static GUID_DEVINTERFACE_USB_HUB: GUID = GUID {
    Data1: 0xF18A0E88,
    Data2: 0xc30C,
    Data3: 0x11D0,
    Data4: [0x88, 0x15, 0x00, 0xA0, 0xC9, 0x06, 0xBE, 0xD8],
};
static GUID_DEVINTERFACE_USB_DEVICE: GUID = GUID {
    Data1: 0xA5DCBF10,
    Data2: 0x6530,
    Data3: 0x11D2,
    Data4: [0x90, 0x1F, 0x00, 0xC0, 0x4F, 0xB9, 0x51, 0xED],
};

#[inline]
unsafe fn LANGIDFROMLCID(lcid: u32) -> u16 {
    // sdk/include/xdk/ntbasedef.h: #define LANGIDFROMLCID(lcid) ((USHORT)(lcid))
    lcid as u16
}

// `#define` constants (verified against the C headers)
const STATUS_CONTINUE_COMPLETION: NTSTATUS = STATUS_SUCCESS; // xdk/iotypes.h: == STATUS_SUCCESS
// IRP_MN_* PnP minor function codes (xdk/iotypes.h:1920-1944)
const IRP_MN_START_DEVICE: u32 = 0x00; // iotypes.h:1920
const IRP_MN_QUERY_REMOVE_DEVICE: u32 = 0x01; // iotypes.h:1921
const IRP_MN_REMOVE_DEVICE: u32 = 0x02; // iotypes.h:1922
const IRP_MN_CANCEL_REMOVE_DEVICE: u32 = 0x03; // iotypes.h:1923
const IRP_MN_STOP_DEVICE: u32 = 0x04; // iotypes.h:1924
const IRP_MN_QUERY_STOP_DEVICE: u32 = 0x05; // iotypes.h:1925
const IRP_MN_CANCEL_STOP_DEVICE: u32 = 0x06; // iotypes.h:1926
const IRP_MN_QUERY_DEVICE_RELATIONS: u32 = 0x07; // iotypes.h:1928
const IRP_MN_QUERY_INTERFACE: u32 = 0x08; // xdk/iotypes.h:1929
const IRP_MN_QUERY_CAPABILITIES: u32 = 0x09; // xdk/iotypes.h:1930
const IRP_MN_QUERY_RESOURCES: u32 = 0x0A; // iotypes.h:1931
const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u32 = 0x0B; // iotypes.h:1932
const IRP_MN_QUERY_DEVICE_TEXT: u32 = 0x0C; // iotypes.h:1933
const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u32 = 0x0D; // iotypes.h:1934
const IRP_MN_READ_CONFIG: u32 = 0x0F; // iotypes.h:1936
const IRP_MN_WRITE_CONFIG: u32 = 0x10; // iotypes.h:1937
const IRP_MN_EJECT: u32 = 0x11; // iotypes.h:1938
const IRP_MN_SET_LOCK: u32 = 0x12; // iotypes.h:1939
const IRP_MN_QUERY_ID: u32 = 0x13; // iotypes.h:1940
const IRP_MN_QUERY_PNP_DEVICE_STATE: u32 = 0x14; // iotypes.h:1941
const IRP_MN_QUERY_BUS_INFORMATION: u32 = 0x15; // iotypes.h:1942
const IRP_MN_DEVICE_USAGE_NOTIFICATION: u32 = 0x16; // iotypes.h:1943
const IRP_MN_SURPRISE_REMOVAL: u32 = 0x17; // iotypes.h:1944
const POWER_SYSTEM_MAXIMUM: u32 = 7; // potypes.h: PowerSystemMaximum == 7

// USBD_ParseConfigurationDescriptorEx / USBD_CreateConfigurationRequestEx are NTAPI (usbdlib.h)
// → stdcall on i386; usbd-rust exports them `_Name@N`. MUST be extern "system", not the cdecl
// block below (which would emit undecorated `_Name`, undefined at link).
extern "system" {
    fn USBD_ParseConfigurationDescriptorEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, StartPosition: PVOID, InterfaceNumber: LONG, AlternateSetting: LONG, InterfaceClass: LONG, InterfaceSubClass: LONG, InterfaceProtocol: LONG, ) -> PUSB_INTERFACE_DESCRIPTOR;
    fn USBD_CreateConfigurationRequestEx( ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, InterfaceList: PUSBD_INTERFACE_LIST_ENTRY, ) -> PURB;
}
// CRT — cdecl on all arches
extern "C" {
    fn wcslen(str: *const u16) -> usize;
}

// variadic — cdecl on all arches (never stdcall)
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
    fn RtlStringCbPrintfExW( pszDest: *mut u16, cbDest: usize, ppszDestEnd: *mut *mut u16, pcbRemaining: *mut usize, dwFlags: u32, pszFormat: *const u16, ... ) -> NTSTATUS;
}
extern "C" {
}
unsafe extern "system" {
    fn USBH_FdoSyncSubmitUrb(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_SyncGetPortStatus( HubExtension: PUSBHUB_FDO_EXTENSION, Port: USHORT, PortStatus: PVOID, Length: ULONG, ) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_SyncResetPort(HubExtension: PUSBHUB_FDO_EXTENSION, Port: USHORT) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_SyncDisablePort(HubExtension: PUSBHUB_FDO_EXTENSION, Port: USHORT) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_CreateDevice( HubExtension: PUSBHUB_FDO_EXTENSION, Port: USHORT, UsbPortStatus: USB_PORT_STATUS, ix: ULONG, ) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_Wait(Milliseconds: ULONG) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBH_CompleteIrp(Irp: PIRP, Status: NTSTATUS);
}
unsafe extern "system" {
    fn USBH_PassIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
unsafe extern "system" {
    fn USBD_RemoveDeviceEx(HubExtension: PUSBHUB_FDO_EXTENSION, DeviceHandle: PUSB_DEVICE_HANDLE, Flags: ULONG,) -> NTSTATUS;
}
unsafe extern "system" {
    fn PdoExt(PdoDevice: PDEVICE_OBJECT) -> PUSBHUB_PORT_PDO_EXTENSION;
}
extern "system" {
    fn InterlockedIncrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long;
    fn InterlockedDecrement(Addend: *mut core::ffi::c_long) -> core::ffi::c_long;
    fn USBHUB_DumpingIDs(Buffer: *mut core::ffi::c_void);
    fn USBH_CheckDeviceLanguage(DeviceObject: PDEVICE_OBJECT, LanguageId: USHORT) -> NTSTATUS;
    fn USBH_SyncGetStringDescriptor( DeviceObject: PDEVICE_OBJECT, Index: UCHAR, LanguageId: USHORT, Descriptor: PUSB_STRING_DESCRIPTOR, DescriptorLength: ULONG, LengthReturned: *mut ULONG, FlagValue: BOOLEAN, ) -> NTSTATUS;
    // USBH_SetPdoRegistryParameter is NTAPI (stdcall) in usbhub.h/pnp.c — it is defined
    // below in this module as `extern "system"`. The old `extern "C"` decl here was an
    // ABI infidelity (produced a cdecl `_USBH_SetPdoRegistryParameter` undefined symbol);
    // removed so call sites bind to the real stdcall definition.
    fn USBH_ResetDevice( HubExtension: PUSBHUB_FDO_EXTENSION, PortNumber: USHORT, IsKeepDeviceData: BOOLEAN, IsWait: BOOLEAN, ) -> NTSTATUS;
    fn USBD_GetPdoRegistryParameter( PhysicalDeviceObject: PDEVICE_OBJECT, Parameter: PVOID, ParameterLength: ULONG, KeyName: PWSTR, KeyNameLength: ULONG, ) -> NTSTATUS;
    fn USBD_RegisterRootHubCallBack(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn USBH_GetConfigurationDescriptor( DeviceObject: PDEVICE_OBJECT, ConfigDescriptor: *mut PUSB_CONFIGURATION_DESCRIPTOR, ) -> NTSTATUS;
    fn USBH_GetDeviceDescriptor( DeviceObject: PDEVICE_OBJECT, DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR, ) -> NTSTATUS;
    fn USBH_GetDeviceType( HubExtension: PUSBHUB_FDO_EXTENSION, DeviceHandle: PUSB_DEVICE_HANDLE, DeviceType: *mut USB_DEVICE_TYPE, ) -> NTSTATUS;
    fn USBH_HubIsBusPowered( DeviceObject: PDEVICE_OBJECT, ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR, ) -> BOOLEAN;
    fn USBH_SubmitStatusChangeTransfer(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn USBH_SyncClearPortStatus( HubExtension: PUSBHUB_FDO_EXTENSION, Port: USHORT, FeatureSelector: USHORT, ) -> NTSTATUS;
    fn USBH_SyncGetDeviceHandle(DeviceObject: PDEVICE_OBJECT) -> PUSB_DEVICE_HANDLE;
    fn USBH_SyncGetHubCount(DeviceObject: PDEVICE_OBJECT, HubCount: *mut ULONG) -> NTSTATUS;
    fn USBH_SyncGetHubDescriptor(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn USBH_SyncGetRootHubPdo( DeviceObject: PDEVICE_OBJECT, RootHubPdo: *mut PDEVICE_OBJECT, RootHubPdo2: *mut PDEVICE_OBJECT, ) -> NTSTATUS;
    fn USBH_SyncPowerOnPorts(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn USBH_WriteFailReasonID(DeviceObject: PDEVICE_OBJECT, FailReason: ULONG,) -> NTSTATUS;
    fn USBD_UnRegisterRootHubCallBack(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS;
    fn USBH_HubCompletePortWakeIrps(HubExtension: PUSBHUB_FDO_EXTENSION, Status: NTSTATUS);
    fn USBH_HubCancelIdleIrp(HubExtension: PUSBHUB_FDO_EXTENSION, IdleIrp: PIRP);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_IrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut Event: PRKEVENT;

    // DPRINT("USBH_IrpCompletion: Irp - %p\n", Irp);

    Event = Context as PRKEVENT;
    KeSetEvent(Event as *mut c_void, EVENT_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_HubPnPIrpComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut HubExtension: PUSBHUB_FDO_EXTENSION;

    HubExtension = Context as PUSBHUB_FDO_EXTENSION;

    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED as u32;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_MORE_PROCESSING_REQUIRED;

    KeSetEvent(
        core::ptr::addr_of_mut!((*HubExtension).LowerDeviceEvent) as *mut c_void,
        EVENT_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_QueryCapsComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Capabilities: PDEVICE_CAPABILITIES;

    // DPRINT("USBH_QueryCapsComplete: ... \n");

    // ASSERT(NT_SUCCESS(Irp->IoStatus.Status));

    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Capabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;

    (*Capabilities).set_SurpriseRemovalOK(1);

    return STATUS_CONTINUE_COMPLETION;
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_GetBusInterface(
    DeviceObject: PDEVICE_OBJECT,
    BusInterface: PUSB_BUS_INTERFACE_HUB_V5,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(USBH_IrpCompletion as *const ()),
        core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_PNP as u8;
    (*IoStack).MinorFunction = IRP_MN_QUERY_INTERFACE as u8;

    (*IoStack).Parameters.QueryInterface.InterfaceType = core::ptr::addr_of!(USB_BUS_INTERFACE_HUB_GUID);
    (*IoStack).Parameters.QueryInterface.Size = core::mem::size_of::<USB_BUS_INTERFACE_HUB_V5>() as u16;
    (*IoStack).Parameters.QueryInterface.Version = USB_BUSIF_HUB_VERSION_5 as u16;
    (*IoStack).Parameters.QueryInterface.Interface = BusInterface as PINTERFACE;
    (*IoStack).Parameters.QueryInterface.InterfaceSpecificData = DeviceObject as *mut core::ffi::c_void;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i32 as i8,
            0,
            core::ptr::null_mut(),
        );

        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    IoFreeIrp(Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBHUB_GetBusInterfaceUSBDI(
    DeviceObject: PDEVICE_OBJECT,
    BusInterfaceUSBDI: PUSB_BUS_INTERFACE_USBDI_V2,
) -> NTSTATUS {

    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(USBH_IrpCompletion as *const ()),
        core::ptr::addr_of_mut!(Event) as PVOID,
        1,
        1,
        1,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*IoStack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;

    (*IoStack).Parameters.QueryInterface.InterfaceType = core::ptr::addr_of!(USB_BUS_INTERFACE_USBDI_GUID);
    (*IoStack).Parameters.QueryInterface.Size =
        core::mem::size_of::<USB_BUS_INTERFACE_USBDI_V2>() as USHORT;
    (*IoStack).Parameters.QueryInterface.Version = USB_BUSIF_USBDI_VERSION_2 as USHORT;
    (*IoStack).Parameters.QueryInterface.Interface = BusInterfaceUSBDI as PINTERFACE;
    (*IoStack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as i32 as KPROCESSOR_MODE,
            0,
            core::ptr::null_mut(),
        );

        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    IoFreeIrp(Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_QueryCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    DeviceCapabilities: PDEVICE_CAPABILITIES,
) {
    let mut Irp: PIRP;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();

    RtlZeroMemory(
        DeviceCapabilities as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);

    if Irp.is_null() {
        return;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(USBH_IrpCompletion as *const ()),
        core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_PNP as u8;
    (*IoStack).MinorFunction = IRP_MN_QUERY_CAPABILITIES as u8;

    (*IoStack).Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;
    (*(*IoStack).Parameters.DeviceCapabilities.Capabilities).Size =
        core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    (*(*IoStack).Parameters.DeviceCapabilities.Capabilities).Version = 1;
    (*(*IoStack).Parameters.DeviceCapabilities.Capabilities).Address = MAXULONG;
    (*(*IoStack).Parameters.DeviceCapabilities.Capabilities).UINumber = MAXULONG;

    if IoCallDriver(DeviceObject, Irp) == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    IoFreeIrp(Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_OpenConfiguration(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {

    const USBHUB_FDO_FLAG_USB20_HUB: u32 = 0x00008000; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_MULTIPLE_TTS: u32 = 0x00040000; // FIXED 2026-07-06: translation invented 0x8; real value is a (1<<N) define in usbhub.h
    const USB_DEVICE_CLASS_HUB: i32 = 0x09;

    let mut current_block: u64;
    let mut Pid: PUSB_INTERFACE_DESCRIPTOR;
    let Urb: PURB;
    let Status: NTSTATUS;
    let mut InterfaceList: [USBD_INTERFACE_LIST_ENTRY; 2] = [
        USBD_INTERFACE_LIST_ENTRY {
            InterfaceDescriptor: core::ptr::null_mut(),
            Interface: core::ptr::null_mut(),
        },
        USBD_INTERFACE_LIST_ENTRY {
            InterfaceDescriptor: core::ptr::null_mut(),
            Interface: core::ptr::null_mut(),
        },
    ];

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_USB20_HUB != 0
        && (*HubExtension).LowerPDO != (*HubExtension).RootHubPdo
    {
        Pid = USBD_ParseConfigurationDescriptorEx(
            (*HubExtension).HubConfigDescriptor,
            (*HubExtension).HubConfigDescriptor as PVOID,
            -1,
            -1,
            USB_DEVICE_CLASS_HUB,
            -1,
            2,
        );
        if !Pid.is_null() {
            (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_MULTIPLE_TTS;
            current_block = 17860125682698302841;
        } else {
            Pid = USBD_ParseConfigurationDescriptorEx(
                (*HubExtension).HubConfigDescriptor,
                (*HubExtension).HubConfigDescriptor as PVOID,
                -1,
                -1,
                USB_DEVICE_CLASS_HUB,
                -1,
                1,
            );
            if !Pid.is_null() {
                current_block = 4985031440501613709;
            } else {
                Pid = USBD_ParseConfigurationDescriptorEx(
                    (*HubExtension).HubConfigDescriptor,
                    (*HubExtension).HubConfigDescriptor as PVOID,
                    -1,
                    -1,
                    USB_DEVICE_CLASS_HUB,
                    -1,
                    0,
                );
                current_block = 17860125682698302841;
            }
        }
    } else {
        Pid = USBD_ParseConfigurationDescriptorEx(
            (*HubExtension).HubConfigDescriptor,
            (*HubExtension).HubConfigDescriptor as PVOID,
            -1,
            -1,
            USB_DEVICE_CLASS_HUB,
            -1,
            -1,
        );
        current_block = 17860125682698302841;
    }

    match current_block {
        17860125682698302841 => {
            if Pid.is_null() {
                return STATUS_UNSUCCESSFUL;
            }
        }
        _ => {}
    }

    if (*Pid).bInterfaceClass as i32 != USB_DEVICE_CLASS_HUB {
        return STATUS_UNSUCCESSFUL;
    }

    InterfaceList[0].InterfaceDescriptor = Pid;

    Urb = USBD_CreateConfigurationRequestEx(
        (*HubExtension).HubConfigDescriptor,
        core::ptr::addr_of_mut!(InterfaceList) as PUSBD_INTERFACE_LIST_ENTRY,
    );

    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = USBH_FdoSyncSubmitUrb((*HubExtension).Common.SelfDevice, Urb);

    if NT_SUCCESS(Status) {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*HubExtension).PipeInfo) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*InterfaceList[0].Interface).Pipes) as *mut core::ffi::c_void,
            core::mem::size_of::<USBD_PIPE_INFORMATION>(),
        );

        (*HubExtension).ConfigHandle =
            (*Urb).__bindgen_anon_1.UrbSelectConfiguration.as_ref().ConfigurationHandle;
    }

    ExFreePool(Urb as PVOID);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBD_Initialize20Hub(HubExtension: PUSBHUB_FDO_EXTENSION) -> NTSTATUS {
    let mut Initialize20Hub: PUSB_BUSIFFN_INITIALIZE_20HUB;
    let mut TtCount: ULONG;
    let DeviceHandle: PUSB_DEVICE_HANDLE;

    Initialize20Hub = (*HubExtension).BusInterface.Initialize20Hub;

    if Initialize20Hub.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    TtCount = 1 as ULONG;

    if (*HubExtension).HubFlags as ::core::ffi::c_ulong
        & USBHUB_FDO_FLAG_MULTIPLE_TTS as ::core::ffi::c_ulong
        != 0
    {
        TtCount = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;
    }

    DeviceHandle = USBH_SyncGetDeviceHandle((*HubExtension).LowerDevice);

    return Initialize20Hub.expect("non-null function pointer")(
        (*HubExtension).BusInterface.BusContext,
        DeviceHandle,
        TtCount,
    );
}
#[no_mangle]
pub unsafe extern "system" fn USBH_AbortInterruptPipe(
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let mut Urb: *mut _URB_PIPE_REQUEST;
    let Status: NTSTATUS;

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_PIPE_REQUEST>() as SIZE_T,
        USB_HUB_TAG as ULONG,
    ) as *mut _URB_PIPE_REQUEST;

    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        Urb as *mut core::ffi::c_void,
        core::mem::size_of::<_URB_PIPE_REQUEST>(),
    );

    (*Urb).Hdr.Length = core::mem::size_of::<_URB_PIPE_REQUEST>() as USHORT;
    (*Urb).Hdr.Function = URB_FUNCTION_ABORT_PIPE as USHORT;
    (*Urb).PipeHandle = (*HubExtension).PipeInfo.PipeHandle;

    Status = USBH_FdoSyncSubmitUrb((*HubExtension).Common.SelfDevice, Urb as PURB);

    if NT_SUCCESS(Status) {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
    }

    ExFreePoolWithTag(Urb as PVOID, USB_HUB_TAG as ULONG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoCleanup(HubExtension: PUSBHUB_FDO_EXTENSION) {

    const USBHUB_FDO_FLAG_DEVICE_STOPPING: u32 = 0x00000004; // USBHUB_FDO_FLAG_DEVICE_STOPPING
    const USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST: u32 = 0x00001000; // FIXED 2026-07-06: translation invented 0x80000; real value is a (1<<N) define in usbhub.h // USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST
    const USBHUB_PDO_FLAG_IDLE_NOTIFICATION: u32 = 0x00400000; // FIXED 2026-07-06: translation invented 0x8; real value is a (1<<N) define in usbhub.h // USBHUB_PDO_FLAG_IDLE_NOTIFICATION
    const USBHUB_PDO_FLAG_WAIT_WAKE: u32 = 0x00000020; // FIXED 2026-07-06: translation invented 0x10; real value is a (1<<N) define in usbhub.h // USBHUB_PDO_FLAG_WAIT_WAKE
    const USBHUB_PDO_FLAG_POWER_D3: u32 = 0x00000100; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h // USBHUB_PDO_FLAG_POWER_D3
    const USBHUB_FDO_FLAG_DEVICE_STARTED: u32 = 0x00000001; // USBHUB_FDO_FLAG_DEVICE_STARTED
    const USB_HUB_TAG: u32 = 0x42554855; // USB_HUB_TAG 'HSuB'
    const EVENT_INCREMENT: i32 = 1; // EVENT_INCREMENT

    let mut IdleIrp: PIRP = core::ptr::null_mut();
    let mut WakeIrp: PIRP = core::ptr::null_mut();
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut PortIdleIrp: PIRP = core::ptr::null_mut();
    let mut PortWakeIrp: PIRP = core::ptr::null_mut();
    let mut DeviceHandle: PVOID;
    let mut Status: NTSTATUS;
    let mut Port: USHORT;
    let NumberPorts: UCHAR;
    let mut Irql: KIRQL = 0;

    USBD_UnRegisterRootHubCallBack(HubExtension);

    (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_STOPPING;

    if (*HubExtension).ResetRequestCount != 0 {
        IoCancelIrp((*HubExtension).ResetPortIrp);

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    IoFreeIrp((*HubExtension).ResetPortIrp);

    (*HubExtension).ResetPortIrp = core::ptr::null_mut();

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST != 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).IdleEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

    if !(*HubExtension).PendingWakeIrp.is_null() {
        WakeIrp = (*HubExtension).PendingWakeIrp;
        (*HubExtension).PendingWakeIrp = core::ptr::null_mut();
    }

    if !(*HubExtension).PendingIdleIrp.is_null() {
        IdleIrp = (*HubExtension).PendingIdleIrp;
        (*HubExtension).PendingIdleIrp = core::ptr::null_mut();
    }

    IoReleaseCancelSpinLock(Irql);

    if !WakeIrp.is_null() {
        USBH_HubCancelWakeIrp(HubExtension, WakeIrp);
    }

    USBH_HubCompletePortWakeIrps(HubExtension, STATUS_DELETE_PENDING);

    if !IdleIrp.is_null() {
        USBH_HubCancelIdleIrp(HubExtension, IdleIrp);
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount)) > 0 {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
    }

    if !(*HubExtension).SCEIrp.is_null() {
        Status = USBH_AbortInterruptPipe(HubExtension);

        if !NT_SUCCESS(Status) && IoCancelIrp((*HubExtension).SCEIrp) != 0 {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent) as PVOID,
                _KWAIT_REASON_Suspended,
                0i32 as i8,
                0,
                core::ptr::null_mut(),
            );
        }

        IoFreeIrp((*HubExtension).SCEIrp);

        (*HubExtension).SCEIrp = core::ptr::null_mut();
    }

    'exit: loop {
        if (*HubExtension).PortData.is_null() || (*HubExtension).HubDescriptor.is_null() {
            break 'exit;
        }

        PortData = (*HubExtension).PortData;
        NumberPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts;

        Port = 0;
        while (Port as i32) < NumberPorts as i32 {
            if !(*PortData.offset(Port as isize)).DeviceObject.is_null() {
                PortExtension = (*(*PortData.offset(Port as isize)).DeviceObject).DeviceExtension
                    as PUSBHUB_PORT_PDO_EXTENSION;

                IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));

                PortIdleIrp = (*PortExtension).IdleNotificationIrp;

                if !PortIdleIrp.is_null() {
                    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_IDLE_NOTIFICATION;
                    (*PortExtension).IdleNotificationIrp = core::ptr::null_mut();

                    if (*PortIdleIrp).Cancel != 0 {
                        PortIdleIrp = core::ptr::null_mut();
                    }

                    if !PortIdleIrp.is_null() {
                        IoSetCancelRoutine(PortIdleIrp, None);
                    }
                }

                PortWakeIrp = (*PortExtension).PdoWaitWakeIrp;

                if !PortWakeIrp.is_null() {
                    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_WAIT_WAKE;
                    (*PortExtension).PdoWaitWakeIrp = core::ptr::null_mut();

                    if (*PortWakeIrp).Cancel != 0
                        || IoSetCancelRoutine(PortWakeIrp, None).is_none()
                    {
                        PortWakeIrp = core::ptr::null_mut();

                        if InterlockedDecrement(
                            core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount),
                        ) == 0
                        {
                            KeSetEvent(
                                core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void,
                                EVENT_INCREMENT,
                                0,
                            );
                        }
                    }
                }

                IoReleaseCancelSpinLock(Irql);

                if !PortIdleIrp.is_null() {
                    (*PortIdleIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
                    IoCompleteRequest(PortIdleIrp, IO_NO_INCREMENT as i8);
                }

                if !PortWakeIrp.is_null() {
                    USBH_CompletePowerIrp(HubExtension, PortWakeIrp, STATUS_CANCELLED);
                }

                if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3 == 0 {
                    DeviceHandle = InterlockedExchangePointer(
                        core::ptr::addr_of_mut!((*PortExtension).DeviceHandle),
                        core::ptr::null_mut(),
                    ) as PVOID;

                    if !DeviceHandle.is_null() {
                        USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
                    }

                    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_POWER_D3;
                }
            }

            USBH_SyncDisablePort(HubExtension, (Port as i32 + 1) as USHORT);

            Port = Port.wrapping_add(1);
        }

        break 'exit;
    }

    if !(*HubExtension).SCEBitmap.is_null() {
        ExFreePoolWithTag((*HubExtension).SCEBitmap, USB_HUB_TAG);
    }

    if !(*HubExtension).HubDescriptor.is_null() {
        ExFreePoolWithTag((*HubExtension).HubDescriptor as PVOID, USB_HUB_TAG);
    }

    if !(*HubExtension).HubConfigDescriptor.is_null() {
        ExFreePoolWithTag((*HubExtension).HubConfigDescriptor as PVOID, USB_HUB_TAG);
    }

    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_DEVICE_STARTED;

    (*HubExtension).HubDescriptor = core::ptr::null_mut();
    (*HubExtension).HubConfigDescriptor = core::ptr::null_mut();

    (*HubExtension).SCEIrp = core::ptr::null_mut();
    (*HubExtension).SCEBitmap = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBH_StartHubFdoDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let mut DisableRemoteWakeup: ULONG = 0;
    let mut HubCount: ULONG = 0;
    let DeviceHandle: PUSB_DEVICE_HANDLE;
    let mut DeviceType: USB_DEVICE_TYPE = Usb11Device;
    let mut DeviceCapabilities: DEVICE_CAPABILITIES = core::mem::zeroed();
    let IsBusPowered: BOOLEAN;
    static mut DisableWakeValueName: [WCHAR; 20] = unsafe {
        core::mem::transmute::<[u8; 40], [WCHAR; 20]>(
            *b"D\0i\0s\0a\0b\0l\0e\0R\0e\0m\0o\0t\0e\0W\0a\0k\0e\0u\0p\0\0\0",
        )
    };

    let mut error: bool = false;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).IdleEvent),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).ResetEvent),
        _EVENT_TYPE_NotificationEvent,
        1 as BOOLEAN,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).LowerDeviceEvent),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).StatusChangeEvent),
        _EVENT_TYPE_NotificationEvent,
        1 as BOOLEAN,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*HubExtension).RootHubNotificationEvent),
        _EVENT_TYPE_NotificationEvent,
        1 as BOOLEAN,
    );

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*HubExtension).CheckIdleSpinLock));

    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
        1 as LONG,
        1 as LONG,
    );
    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).HubPortSemaphore),
        1 as LONG,
        1 as LONG,
    );
    KeInitializeSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).HubSemaphore),
        1 as LONG,
        1 as LONG,
    );

    (*HubExtension).HubFlags = 0;
    (*HubExtension).HubConfigDescriptor = core::ptr::null_mut();
    (*HubExtension).HubDescriptor = core::ptr::null_mut();
    (*HubExtension).SCEIrp = core::ptr::null_mut();
    (*HubExtension).SCEBitmap = core::ptr::null_mut();
    (*HubExtension).SystemPowerState.SystemState = PowerSystemWorking;
    (*HubExtension).PendingRequestCount = 1 as LONG;
    (*HubExtension).ResetRequestCount = 0 as LONG;
    (*HubExtension).PendingIdleIrp = core::ptr::null_mut();
    (*HubExtension).PendingWakeIrp = core::ptr::null_mut();

    InitializeListHead(core::ptr::addr_of_mut!((*HubExtension).PdoList));

    (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_WITEM_INIT;
    InitializeListHead(core::ptr::addr_of_mut!((*HubExtension).WorkItemList));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*HubExtension).WorkItemSpinLock));

    IoCopyCurrentIrpStackLocationToNext(Irp);

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(USBH_HubPnPIrpComplete as *const ()),
        HubExtension as PVOID,
        1 as BOOLEAN,
        1 as BOOLEAN,
        1 as BOOLEAN,
    );

    if IoCallDriver((*HubExtension).LowerDevice, Irp) == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).LowerDeviceEvent) as PVOID,
            _KWAIT_REASON_Suspended,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
    }

    (*HubExtension).RootHubPdo = core::ptr::null_mut();

    Status = USBH_SyncGetRootHubPdo(
        (*HubExtension).LowerDevice,
        core::ptr::addr_of_mut!((*HubExtension).RootHubPdo),
        core::ptr::addr_of_mut!((*HubExtension).RootHubPdo2),
    );

    if !NT_SUCCESS(Status) {
        error = true;
    } else {
        USBH_WriteFailReasonID((*HubExtension).LowerPDO, USBHUB_FAIL_NO_FAIL as ULONG);

        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0 {
            Status = STATUS_UNSUCCESSFUL;
            error = true;
        } else {
            (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_REMOTE_WAKEUP;

            Status = USBD_GetPdoRegistryParameter(
                (*HubExtension).LowerPDO,
                core::ptr::addr_of_mut!(DisableRemoteWakeup) as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
                core::ptr::addr_of_mut!(DisableWakeValueName) as PWSTR,
                core::mem::size_of::<[WCHAR; 20]>() as ULONG,
            );

            if NT_SUCCESS(Status) && DisableRemoteWakeup != 0 {
                (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_REMOTE_WAKEUP;
            }

            (*HubExtension).CurrentPowerState.DeviceState = PowerDeviceD0;

            USBH_SyncGetHubCount((*HubExtension).LowerDevice, core::ptr::addr_of_mut!(HubCount));

            Status = USBHUB_GetBusInterface(
                (*HubExtension).RootHubPdo,
                core::ptr::addr_of_mut!((*HubExtension).BusInterface),
            );
            ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBHUB_GetBusInterface -> 0x%x\n\0".as_ptr() as *const i8, Status);

            if !NT_SUCCESS(Status) {
                error = true;
            } else {
                Status = USBHUB_GetBusInterfaceUSBDI(
                    (*HubExtension).LowerDevice,
                    core::ptr::addr_of_mut!((*HubExtension).BusInterfaceUSBDI),
                );
                ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBHUB_GetBusInterfaceUSBDI -> 0x%x\n\0".as_ptr() as *const i8, Status);

                if !NT_SUCCESS(Status) {
                    error = true;
                } else {
                    DeviceHandle = USBH_SyncGetDeviceHandle((*HubExtension).LowerDevice);

                    let mut handle_failed = false;
                    if !DeviceHandle.is_null() {
                        Status = USBH_GetDeviceType(
                            HubExtension,
                            DeviceHandle,
                            core::ptr::addr_of_mut!(DeviceType),
                        );
                        ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_GetDeviceType -> 0x%x\n\0".as_ptr() as *const i8, Status);

                        if !NT_SUCCESS(Status) {
                            error = true;
                            handle_failed = true;
                        } else {
                            if DeviceType as u32 == Usb20Device as u32 {
                                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_USB20_HUB;
                            }
                        }
                    }

                    if !handle_failed {
                        if HubCount > USBHUB_MAX_CASCADE_LEVELS as ULONG {
                            let ParentPdoExtension: PUSBHUB_PORT_PDO_EXTENSION;
                            let ParentHubExtension: PUSBHUB_FDO_EXTENSION;
                            let ParentPort: USHORT;
                            let PortData: PUSBHUB_PORT_DATA;

                            USBH_WriteFailReasonID(
                                (*HubExtension).LowerPDO,
                                USBHUB_FAIL_NESTED_TOO_DEEPLY as ULONG,
                            );

                            ParentPdoExtension =
                                (*(*HubExtension).LowerPDO).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                            ParentHubExtension = (*ParentPdoExtension).HubExtension;

                            ParentPort = ((*ParentPdoExtension).PortNumber as i32 - 1) as USHORT;
                            PortData = (*ParentHubExtension).PortData.offset(ParentPort as isize)
                                as PUSBHUB_PORT_DATA;
                            (*PortData).ConnectionStatus = DeviceHubNestedTooDeeply;

                            (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
                        }

                        USBH_QueryCapabilities(
                            (*HubExtension).LowerDevice,
                            core::ptr::addr_of_mut!(DeviceCapabilities),
                        );

                        (*HubExtension).SystemWake = DeviceCapabilities.SystemWake;
                        (*HubExtension).DeviceWake = DeviceCapabilities.DeviceWake;

                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of_mut!(DeviceCapabilities.DeviceState) as *const u8,
                            core::ptr::addr_of_mut!((*HubExtension).DeviceState) as *mut u8,
                            (POWER_SYSTEM_MAXIMUM as usize)
                                * core::mem::size_of::<DEVICE_POWER_STATE>(),
                        );

                        Status = USBH_GetDeviceDescriptor(
                            (*HubExtension).Common.SelfDevice,
                            core::ptr::addr_of_mut!((*HubExtension).HubDeviceDescriptor),
                        );
                        ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_GetDeviceDescriptor -> 0x%x\n\0".as_ptr() as *const i8, Status);

                        if !NT_SUCCESS(Status) {
                            error = true;
                        } else {
                            Status = USBH_GetConfigurationDescriptor(
                                (*HubExtension).Common.SelfDevice,
                                core::ptr::addr_of_mut!((*HubExtension).HubConfigDescriptor),
                            );
                            ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_GetConfigurationDescriptor -> 0x%x\n\0".as_ptr() as *const i8, Status);

                            if !NT_SUCCESS(Status) {
                                error = true;
                            } else {
                                Status = USBH_SyncGetHubDescriptor(HubExtension);
                                ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_SyncGetHubDescriptor -> 0x%x\n\0".as_ptr() as *const i8, Status);

                                if !NT_SUCCESS(Status) {
                                    error = true;
                                } else {
                                    IsBusPowered = USBH_HubIsBusPowered(
                                        (*HubExtension).Common.SelfDevice,
                                        (*HubExtension).HubConfigDescriptor,
                                    );

                                    if IsBusPowered != 0 {
                                        (*HubExtension).MaxPowerPerPort = 100 as ULONG;
                                        (*(*HubExtension).HubConfigDescriptor).MaxPower = 250 as UCHAR;
                                    } else {
                                        (*HubExtension).MaxPowerPerPort = 500 as ULONG;
                                    }

                                    Status = USBH_OpenConfiguration(HubExtension);
                                    ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_OpenConfiguration -> 0x%x\n\0".as_ptr() as *const i8, Status);

                                    if !NT_SUCCESS(Status) {
                                        error = true;
                                    } else {
                                        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_USB20_HUB != 0 {
                                            Status = USBD_Initialize20Hub(HubExtension);
                                            ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBD_Initialize20Hub -> 0x%x\n\0".as_ptr() as *const i8, Status);
                                        }

                                        if !NT_SUCCESS(Status) {
                                            error = true;
                                        } else {
                                            (*HubExtension).SCEIrp = IoAllocateIrp(
                                                (*(*HubExtension).Common.SelfDevice).StackSize,
                                                0 as BOOLEAN,
                                            );

                                            (*HubExtension).ResetPortIrp = IoAllocateIrp(
                                                (*(*HubExtension).Common.SelfDevice).StackSize,
                                                0 as BOOLEAN,
                                            );

                                            if (*HubExtension).SCEIrp.is_null()
                                                || (*HubExtension).ResetPortIrp.is_null()
                                            {
                                                Status = STATUS_INSUFFICIENT_RESOURCES;
                                                error = true;
                                            } else {
                                                (*HubExtension).SCEBitmapLength =
                                                    (*HubExtension).PipeInfo.MaximumPacketSize as ULONG;

                                                (*HubExtension).SCEBitmap = ExAllocatePoolWithTag(
                                                    NonPagedPool,
                                                    (*HubExtension).SCEBitmapLength as SIZE_T,
                                                    USB_HUB_TAG as ULONG,
                                                );

                                                if (*HubExtension).SCEBitmap.is_null() {
                                                    Status = STATUS_INSUFFICIENT_RESOURCES;
                                                    error = true;
                                                } else {
                                                    core::ptr::write_bytes(
                                                        (*HubExtension).SCEBitmap as *mut u8,
                                                        0,
                                                        (*HubExtension).SCEBitmapLength as usize,
                                                    );

                                                    Status = USBH_SyncPowerOnPorts(HubExtension);
                                                    ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_SyncPowerOnPorts -> 0x%x\n\0".as_ptr() as *const i8, Status);

                                                    if !NT_SUCCESS(Status) {
                                                        error = true;
                                                    } else {
                                                        let mut Port: USHORT;

                                                        (*HubExtension).HubFlags |=
                                                            USBHUB_FDO_FLAG_DEVICE_STARTED;

                                                        Port = 1 as USHORT;
                                                        while Port as i32
                                                            <= (*(*HubExtension).HubDescriptor)
                                                                .bNumberOfPorts
                                                                as i32
                                                        {
                                                            USBH_SyncClearPortStatus(
                                                                HubExtension,
                                                                Port,
                                                                USBHUB_FEATURE_C_PORT_CONNECTION
                                                                    as USHORT,
                                                            );
                                                            Port = Port.wrapping_add(1);
                                                        }

                                                        if (*HubExtension).LowerPDO
                                                            == (*HubExtension).RootHubPdo
                                                        {
                                                            USBD_RegisterRootHubCallBack(HubExtension);
                                                        } else {
                                                            (*HubExtension).HubFlags |=
                                                                USBHUB_FDO_FLAG_DO_ENUMERATION;
                                                            USBH_SubmitStatusChangeTransfer(
                                                                HubExtension,
                                                            );
                                                            ntkrnl_sys::driver_ddk::DbgPrint(b"[USBHUB] stage USBH_SubmitStatusChangeTransfer -> 0x%x\n\0".as_ptr() as *const i8, Status);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if error {
        if !(*HubExtension).HubDescriptor.is_null() {
            ExFreePoolWithTag((*HubExtension).HubDescriptor as PVOID, USB_HUB_TAG as ULONG);
            (*HubExtension).HubDescriptor = core::ptr::null_mut();
        }

        if !(*HubExtension).SCEIrp.is_null() {
            IoFreeIrp((*HubExtension).SCEIrp);
            (*HubExtension).SCEIrp = core::ptr::null_mut();
        }

        if !(*HubExtension).ResetPortIrp.is_null() {
            IoFreeIrp((*HubExtension).ResetPortIrp);
            (*HubExtension).ResetPortIrp = core::ptr::null_mut();
        }

        if !(*HubExtension).SCEBitmap.is_null() {
            ExFreePoolWithTag((*HubExtension).SCEBitmap, USB_HUB_TAG as ULONG);
            (*HubExtension).SCEBitmap = core::ptr::null_mut();
        }

        if !(*HubExtension).HubConfigDescriptor.is_null() {
            ExFreePoolWithTag((*HubExtension).HubConfigDescriptor as PVOID, USB_HUB_TAG as ULONG);
            (*HubExtension).HubConfigDescriptor = core::ptr::null_mut();
        }
    }

    USBH_CompleteIrp(Irp, Status);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoQueryBusRelations(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const USBHUB_FDO_FLAG_DEVICE_STARTED: u32 = 0x00000001; // FIXED 2026-07-06: translation invented 0x400; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_DO_ENUMERATION: u32 = 0x00100000; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_NOT_ENUMERATED: u32 = 0x00800000; // FIXED 2026-07-06: translation invented 0x800; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_ESD_RECOVERING: u32 = 0x00000200; // FIXED 2026-07-06: translation invented 0x400000; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_ENUM_POST_RECOVER: u32 = 0x00080000; // FIXED 2026-07-06: translation invented 0x800000; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_DEVICE_FAILED: u32 = 0x00000008; // FIXED 2026-07-06: translation invented 0x1000; real value is a (1<<N) define in usbhub.h
    const USBHUB_FDO_FLAG_USB20_HUB: u32 = 0x00008000; // FIXED 2026-07-06: translation invented 0x200; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_OVERCURRENT_PORT: u32 = 0x00004000; // FIXED 2026-07-06: translation invented 0x40; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_DELETE_PENDING: u32 = 0x00000080; // FIXED 2026-07-06: translation invented 0x4; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_POWER_D1_OR_D2: u32 = 0x00002000; // FIXED 2026-07-06: translation invented 0x80; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_PORT_LOW_SPEED: u32 = 0x00000008; // FIXED 2026-07-06: translation invented 0x1; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_PORT_HIGH_SPEED: u32 = 0x00800000; // FIXED 2026-07-06: translation invented 0x2; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_HS_USB1_DUALMODE: u32 = 0x00000400; // FIXED 2026-07-06: translation invented 0x800; real value is a (1<<N) define in usbhub.h
    const USBHUB_ENUM_FLAG_DEVICE_PRESENT: u32 = 0x00000001;
    const USBHUB_ENUM_FLAG_GHOST_DEVICE: u32 = 0x00000002;
    const STATUS_INVALID_DEVICE_STATE: NTSTATUS = 0xC0000184u32 as NTSTATUS;
    const USB_HUB_TAG: u32 = 0x42554855;
    const EVENT_INCREMENT: i32 = 1;
    const NoDeviceConnected: u32 = 0;
    const DeviceConnected: u32 = 1;
    const DeviceFailedEnumeration: u32 = 2;

    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut GhostPdoList: LIST_ENTRY = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;
    let mut PdoList: PLIST_ENTRY;
    let mut NumberPorts: UCHAR = 0;
    let mut Port: USHORT;
    let mut GhostPort: USHORT;
    let mut PortData: PUSBHUB_PORT_DATA;
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut pdoExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let mut NtStatus: NTSTATUS;
    let mut SerialNumber: PVOID;
    let mut DeviceHandle: PVOID;
    let mut UsbPortStatus: USB_PORT_STATUS;
    let mut Entry: PLIST_ENTRY;
    let mut Length: ULONG;

    let mut current_block: u64;

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED == 0 {
        Status = STATUS_INVALID_DEVICE_STATE;
    } else if (*HubExtension).HubDescriptor.is_null() {
        Status = STATUS_UNSUCCESSFUL;
    } else if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION == 0 {
        let mut Interval: LARGE_INTEGER = core::mem::zeroed();
        Status = STATUS_SUCCESS;
        IoInvalidateDeviceRelations((*HubExtension).LowerPDO, BusRelations);
        Interval.QuadPart = -10000i64 * 1000;
        KeDelayExecutionThread(0i32 as i8, 0, core::ptr::addr_of_mut!(Interval));
    } else {
        InterlockedIncrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long);
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
        NumberPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts;
        Length = (core::mem::offset_of!(DEVICE_RELATIONS, Objects)
            + NumberPorts as usize * core::mem::size_of::<PDEVICE_OBJECT>()) as ULONG;
        if (*Irp).IoStatus.Information != 0 {
            DbgPrint(b"FIXME: leaking old bus relations\n\0".as_ptr() as *const i8);
        }
        DeviceRelations = ExAllocatePoolWithTag(NonPagedPool, Length as usize, USB_HUB_TAG) as PDEVICE_RELATIONS;
        if DeviceRelations.is_null() {
            (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_NOT_ENUMERATED;
            KeReleaseSemaphore(
                core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
                LOW_REALTIME_PRIORITY as i32,
                1,
                0,
            );
            if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long) == 0 {
                KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void, EVENT_INCREMENT, 0);
            }
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            core::ptr::write_bytes(DeviceRelations as *mut u8, 0, Length as usize);
            (*DeviceRelations).Count = 0;
            '_EnumStart: loop {
                if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING != 0 {
                    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_NOT_ENUMERATED;
                    KeReleaseSemaphore(
                        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
                        LOW_REALTIME_PRIORITY as i32,
                        1,
                        0,
                    );
                    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long) == 0 {
                        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void, EVENT_INCREMENT, 0);
                    }
                    Status = STATUS_SUCCESS;
                    break;
                } else {
                    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_ENUM_POST_RECOVER;
                    Port = 1;
                    while Port as i32 <= NumberPorts as i32 {
                        PortData = (*HubExtension).PortData.offset((Port as i32 - 1) as isize);
                        if !((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0) {
                            Status = USBH_SyncGetPortStatus(
                                HubExtension,
                                Port,
                                core::ptr::addr_of_mut!((*PortData).PortStatus) as PVOID,
                                core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
                            );
                            if !(Status >= 0) {
                                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
                                (*DeviceRelations).Count = 0;
                                continue '_EnumStart;
                            } else {
                                PdoDevice = (*PortData).DeviceObject;
                                if !(*PortData).DeviceObject.is_null() {
                                    PdoExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                                    if (*PdoExtension).PortPdoFlags & USBHUB_PDO_FLAG_OVERCURRENT_PORT != 0 {
                                        (*PortData).PortStatus.__bindgen_anon_1.as_mut().PortStatus.Usb20PortStatus.as_mut().__bindgen_anon_1.as_mut().set_CurrentConnectStatus(1);
                                    }
                                }
                                if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0 {
                                    DbgPrint(b"USBH_FdoQueryBusRelations: DbgBreakPoint() \n\0".as_ptr() as *const i8);
                                    DbgBreakPoint();
                                }
                                if (*PortData).PortStatus.__bindgen_anon_1.as_ref().PortStatus.Usb20PortStatus.as_ref().__bindgen_anon_1.as_ref().CurrentConnectStatus() == 0 {
                                    if !PdoDevice.is_null() {
                                        PdoExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                                        (*PdoExtension).PortPdoFlags |= USBHUB_PDO_FLAG_DELETE_PENDING;
                                        (*PdoExtension).EnumFlags &= !USBHUB_ENUM_FLAG_DEVICE_PRESENT;
                                        SerialNumber = InterlockedExchangePointer(
                                            core::ptr::addr_of_mut!((*PdoExtension).SerialNumber) as *mut PVOID,
                                            core::ptr::null_mut(),
                                        );
                                        if !SerialNumber.is_null() {
                                            ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
                                        }
                                        DeviceHandle = InterlockedExchangePointer(
                                            core::ptr::addr_of_mut!((*PdoExtension).DeviceHandle) as *mut PVOID,
                                            core::ptr::null_mut(),
                                        );
                                        if !DeviceHandle.is_null() {
                                            USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
                                            USBH_SyncDisablePort(HubExtension, Port);
                                        }
                                    }
                                    (*PortData).DeviceObject = core::ptr::null_mut();
                                    (*PortData).ConnectionStatus = NoDeviceConnected;
                                } else if !PdoDevice.is_null() {
                                    ObReferenceObject(PdoDevice as PVOID);
                                    (*PdoDevice).Flags |= DO_POWER_PAGABLE;
                                    (*PdoDevice).Flags &= !DO_DEVICE_INITIALIZING;
                                    let fresh6 = (*DeviceRelations).Count;
                                    (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
                                    *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(fresh6 as isize) = PdoDevice;
                                    PdoExtension = (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                                    (*PdoExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_POWER_D1_OR_D2;
                                } else {
                                    USBH_Wait(100);
                                    NtStatus = USBH_SyncResetPort(HubExtension, Port);
                                    if !(NtStatus >= 0) {
                                        if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_USB20_HUB != 0 {
                                            (*PortData).DeviceObject = core::ptr::null_mut();
                                            (*PortData).ConnectionStatus = NoDeviceConnected;
                                            current_block = 5141539773904409130;
                                        } else {
                                            current_block = 7337917895049117968;
                                        }
                                    } else {
                                        NtStatus = USBH_SyncGetPortStatus(
                                            HubExtension,
                                            Port,
                                            core::ptr::addr_of_mut!((*PortData).PortStatus) as PVOID,
                                            core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
                                        );
                                        if NtStatus >= 0 {
                                            let mut ix: ULONG = 0;
                                            UsbPortStatus = core::ptr::read(core::ptr::addr_of!((*PortData).PortStatus.__bindgen_anon_1.as_ref().PortStatus));
                                            NtStatus = USBH_CreateDevice(HubExtension, Port, core::ptr::read(core::ptr::addr_of!(UsbPortStatus)), ix);
                                            while !(NtStatus >= 0) {
                                                USBH_Wait(500);
                                                if ix >= 2 {
                                                    break;
                                                }
                                                if !(*PortData).DeviceObject.is_null() {
                                                    IoDeleteDevice((*PortData).DeviceObject);
                                                    (*PortData).DeviceObject = core::ptr::null_mut();
                                                    (*PortData).ConnectionStatus = NoDeviceConnected;
                                                }
                                                USBH_SyncResetPort(HubExtension, Port);
                                                /* The device can come back at another speed after the reset */
                                                if USBH_SyncGetPortStatus(
                                                    HubExtension,
                                                    Port,
                                                    core::ptr::addr_of_mut!((*PortData).PortStatus) as PVOID,
                                                    core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
                                                ) >= 0
                                                {
                                                    UsbPortStatus = core::ptr::read(core::ptr::addr_of!((*PortData).PortStatus.__bindgen_anon_1.as_ref().PortStatus));
                                                }
                                                ix = ix.wrapping_add(1);
                                                NtStatus = USBH_CreateDevice(HubExtension, Port, core::ptr::read(core::ptr::addr_of!(UsbPortStatus)), ix);
                                            }
                                            if NtStatus >= 0 {
                                                PdoExtension = (*(*PortData).DeviceObject).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                                                if (*PdoExtension).PortPdoFlags & USBHUB_PDO_FLAG_PORT_LOW_SPEED == 0
                                                    && (*PdoExtension).PortPdoFlags & USBHUB_PDO_FLAG_PORT_HIGH_SPEED == 0
                                                    && (*HubExtension).HubFlags & USBHUB_FDO_FLAG_USB20_HUB == 0
                                                {
                                                    DbgPrint(b"USBH_FdoQueryBusRelations: FIXME USBH_DeviceIs2xDualMode()\n\0".as_ptr() as *const i8);
                                                    if false {
                                                        (*PdoExtension).PortPdoFlags |= USBHUB_PDO_FLAG_HS_USB1_DUALMODE;
                                                    }
                                                }
                                                ObReferenceObject((*PortData).DeviceObject as PVOID);
                                                *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset((*DeviceRelations).Count as isize) = (*PortData).DeviceObject;
                                                (*(*PortData).DeviceObject).Flags |= DO_POWER_PAGABLE;
                                                (*(*PortData).DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
                                                (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
                                                (*PortData).ConnectionStatus = DeviceConnected;
                                                current_block = 5141539773904409130;
                                            } else {
                                                current_block = 7337917895049117968;
                                            }
                                        } else {
                                            current_block = 7337917895049117968;
                                        }
                                    }
                                    match current_block {
                                        5141539773904409130 => {}
                                        _ => {
                                            (*PortData).ConnectionStatus = DeviceFailedEnumeration;
                                            if (USBH_SyncDisablePort(HubExtension, Port) as ULONG) >> 30 == 3 {
                                                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
                                            }
                                            if !(*PortData).DeviceObject.is_null() {
                                                ObReferenceObject((*PortData).DeviceObject as PVOID);
                                                (*(*PortData).DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
                                                let fresh9 = (*DeviceRelations).Count;
                                                (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
                                                *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(fresh9 as isize) = (*PortData).DeviceObject;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Port = Port.wrapping_add(1);
                    }
                    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_NOT_ENUMERATED;
                    KeReleaseSemaphore(
                        core::ptr::addr_of_mut!((*HubExtension).ResetDeviceSemaphore),
                        LOW_REALTIME_PRIORITY as i32,
                        1,
                        0,
                    );
                    if InterlockedDecrement(core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount) as *mut core::ffi::c_long) == 0 {
                        KeSetEvent(core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void, EVENT_INCREMENT, 0);
                    }
                    break;
                }
            }
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if !(Status >= 0) {
        if !DeviceRelations.is_null() {
            ExFreePoolWithTag(DeviceRelations as PVOID, USB_HUB_TAG);
        }
        USBH_CompleteIrp(Irp, Status);
        return Status;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock) as PKSPIN_LOCK, core::ptr::addr_of_mut!(OldIrql));

    if !DeviceRelations.is_null() && (*DeviceRelations).Count != 0 {
        Port = 0;
        while (Port as ULONG) < (*DeviceRelations).Count {
            let mut current_block_160: u64;
            PdoDevice = *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(Port as isize);
            Entry = (*HubExtension).PdoList.Flink;
            loop {
                if !(Entry != core::ptr::addr_of_mut!((*HubExtension).PdoList)) {
                    current_block_160 = 2544535129495155983;
                    break;
                }
                pdoExtension = ((Entry as usize)
                    .wrapping_sub(core::mem::offset_of!(USBHUB_PORT_PDO_EXTENSION, PortLink)))
                    as PUSBHUB_PORT_PDO_EXTENSION;
                if pdoExtension == (*PdoDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION {
                    (*PdoExt(PdoDevice)).EnumFlags |= USBHUB_ENUM_FLAG_GHOST_DEVICE;
                    current_block_160 = 13665239467142187023;
                    break;
                } else {
                    Entry = (*Entry).Flink;
                }
            }
            match current_block_160 {
                2544535129495155983 => {
                    (*PdoExt(PdoDevice)).EnumFlags |= USBHUB_ENUM_FLAG_DEVICE_PRESENT;
                }
                _ => {}
            }
            Port = Port.wrapping_add(1);
        }
        Port = 0;
        while (Port as ULONG) < (*DeviceRelations).Count {
            PdoDevice = *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(Port as isize);
            if (*PdoExt(PdoDevice)).EnumFlags & USBHUB_ENUM_FLAG_GHOST_DEVICE != 0 {
                GhostPort = Port;
                while (GhostPort as ULONG) < (*DeviceRelations).Count {
                    *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset(GhostPort as isize) =
                        *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT).offset((GhostPort as i32 + 1) as isize);
                    GhostPort = GhostPort.wrapping_add(1);
                }
                ObDereferenceObject(PdoDevice as PVOID);
                (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_sub(1);
                if (*PdoExt(PdoDevice)).EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT != 0 {
                    (*PdoExt(PdoDevice)).EnumFlags &= !USBHUB_ENUM_FLAG_GHOST_DEVICE;
                }
            }
            Port = Port.wrapping_add(1);
        }
    }

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR as _;

    InitializeListHead(core::ptr::addr_of_mut!(GhostPdoList));
    PdoList = core::ptr::addr_of_mut!((*HubExtension).PdoList);
    while !IsListEmpty(PdoList) {
        Entry = RemoveHeadList(PdoList);
        PdoExtension = ((Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBHUB_PORT_PDO_EXTENSION, PortLink)))
            as PUSBHUB_PORT_PDO_EXTENSION;
        (*PdoExtension).EnumFlags &= !USBHUB_ENUM_FLAG_DEVICE_PRESENT;
        if (*PdoExtension).EnumFlags & USBHUB_ENUM_FLAG_GHOST_DEVICE != 0 {
            InsertTailList(core::ptr::addr_of_mut!(GhostPdoList), core::ptr::addr_of_mut!((*PdoExtension).PortLink));
        }
    }
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*HubExtension).RelationsWorkerSpinLock) as PKSPIN_LOCK, OldIrql);

    while !IsListEmpty(core::ptr::addr_of_mut!(GhostPdoList)) {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!(GhostPdoList));
        PdoExtension = ((Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBHUB_PORT_PDO_EXTENSION, PortLink)))
            as PUSBHUB_PORT_PDO_EXTENSION;
        IoDeleteDevice((*PdoExtension).Common.SelfDevice);
    }

    return USBH_PassIrp((*HubExtension).LowerDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoStopDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBH_FdoStopDevice: UNIMPLEMENTED. FIXME\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/pnp.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1456 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/pnp.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1456 as ::core::ffi::c_int,
        );
    }
    DbgBreakPoint();
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoSurpriseRemoveDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) {
    let mut PortExtension: PUSBHUB_PORT_PDO_EXTENSION;
    let PortData: PUSBHUB_PORT_DATA;
    let NumberPorts: ULONG;
    let mut Port: ULONG;

    if (*HubExtension).PortData.is_null() || (*HubExtension).HubDescriptor.is_null() {
        return;
    }

    PortData = (*HubExtension).PortData;
    NumberPorts = (*(*HubExtension).HubDescriptor).bNumberOfPorts as ULONG;

    Port = 0 as ULONG;
    while Port < NumberPorts {
        if !(*PortData.offset(Port as isize)).DeviceObject.is_null() {
            PortExtension = PdoExt((*PortData.offset(Port as isize)).DeviceObject);
            (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_DELETE_PENDING as core::ffi::c_ulong;
            (*PortExtension).EnumFlags &= !USBHUB_ENUM_FLAG_DEVICE_PRESENT as core::ffi::c_ulong;
            (*PortData.offset(Port as isize)).DeviceObject = core::ptr::null_mut();
            (*PortData.offset(Port as isize)).ConnectionStatus = NoDeviceConnected;
        }
        Port = Port.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoQueryId(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let IdType: ULONG;
    let mut Buffer: [WCHAR; 200] = [0; 200];
    let mut EndBuffer: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut Remaining: usize = core::mem::size_of::<[WCHAR; 200]>();
    let mut Length: usize;
    let mut Id: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR;
    let InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;

    IdType = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.QueryId.IdType as ULONG;
    DeviceDescriptor =
        core::ptr::addr_of_mut!((*PortExtension).DeviceDescriptor) as PUSB_DEVICE_DESCRIPTOR;
    InterfaceDescriptor =
        core::ptr::addr_of_mut!((*PortExtension).InterfaceDescriptor) as PUSB_INTERFACE_DESCRIPTOR;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
        0,
        core::mem::size_of::<[WCHAR; 200]>() / core::mem::size_of::<WCHAR>(),
    );

    match IdType {
        0 => {
            // BusQueryDeviceID
            if (*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_INIT_PORT_FAILED as u32 != 0 {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 44], [u16; 22]>(
                        *b"U\0S\0B\0\\\0V\0i\0d\0_\x000\x000\x000\x000\0&\0P\0i\0d\0_\x000\x000\x000\x000\0\0\0",
                    ).as_ptr(),
                );
            } else {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 44], [u16; 22]>(
                        *b"U\0S\0B\0\\\0V\0i\0d\0_\0%\x000\x004\0x\0&\0P\0i\0d\0_\0%\x000\x004\0x\0\0\0",
                    ).as_ptr(),
                    (*DeviceDescriptor).idVendor as core::ffi::c_int,
                    (*DeviceDescriptor).idProduct as core::ffi::c_int,
                );
            }

            Length = (core::mem::size_of::<[WCHAR; 200]>())
                .wrapping_sub(Remaining.wrapping_sub(core::mem::size_of::<WCHAR>()));

            Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG as ULONG) as PWCHAR;

            if Id.is_null() {
                // break
            } else {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!(Buffer) as *const u8,
                    Id as *mut u8,
                    Length,
                );
            }
        }
        1 => {
            // BusQueryHardwareIDs
            if (*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_INIT_PORT_FAILED as u32 != 0 {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 24], [u16; 12]>(
                        *b"U\0S\0B\0\\\0U\0N\0K\0N\0O\0W\0N\0\0\0",
                    ).as_ptr(),
                );
            } else {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 62], [u16; 31]>(
                        *b"U\0S\0B\0\\\0V\0i\0d\0_\0%\x000\x004\0x\0&\0P\0i\0d\0_\0%\x000\x004\0x\0&\0R\0e\0v\0_\0%\x000\x004\0x\0\0\0",
                    ).as_ptr(),
                    (*DeviceDescriptor).idVendor as core::ffi::c_int,
                    (*DeviceDescriptor).idProduct as core::ffi::c_int,
                    (*DeviceDescriptor).bcdDevice as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 44], [u16; 22]>(
                        *b"U\0S\0B\0\\\0V\0i\0d\0_\0%\x000\x004\0x\0&\0P\0i\0d\0_\0%\x000\x004\0x\0\0\0",
                    ).as_ptr(),
                    (*DeviceDescriptor).idVendor as core::ffi::c_int,
                    (*DeviceDescriptor).idProduct as core::ffi::c_int,
                );
            }

            Length = (core::mem::size_of::<[WCHAR; 200]>()).wrapping_sub(
                Remaining.wrapping_sub((2usize).wrapping_mul(core::mem::size_of::<WCHAR>())),
            );

            Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG as ULONG) as PWCHAR;

            if Id.is_null() {
                // break
            } else {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!(Buffer) as *const u8,
                    Id as *mut u8,
                    Length,
                );

                if !((*PortExtension).PortPdoFlags as u32
                    & USBHUB_PDO_FLAG_INIT_PORT_FAILED as u32
                    != 0)
                {
                    USBHUB_DumpingIDs(Id as *mut core::ffi::c_void);
                }
            }
        }
        2 => {
            // BusQueryCompatibleIDs
            if (*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_INIT_PORT_FAILED as u32 != 0 {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 24], [u16; 12]>(
                        *b"U\0S\0B\0\\\0U\0N\0K\0N\0O\0W\0N\0\0\0",
                    ).as_ptr(),
                );
            } else if (*PortExtension).PortPdoFlags as u32
                & USBHUB_PDO_FLAG_MULTI_INTERFACE as u32
                != 0
            {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 84], [u16; 42]>(
                        *b"U\0S\0B\0\\\0D\0e\0v\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0S\0u\0b\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0P\0r\0o\0t\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceSubClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceProtocol as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 64], [u16; 32]>(
                        *b"U\0S\0B\0\\\0D\0e\0v\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0S\0u\0b\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceSubClass as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 36], [u16; 18]>(
                        *b"U\0S\0B\0\\\0D\0e\0v\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 28], [u16; 14]>(
                        *b"U\0S\0B\0\\\0C\0O\0M\0P\0O\0S\0I\0T\0E\0\0\0",
                    ).as_ptr(),
                );
            } else {
                RtlStringCbPrintfExW(
                    core::ptr::addr_of_mut!(Buffer) as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 78], [u16; 39]>(
                        *b"U\0S\0B\0\\\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0S\0u\0b\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0P\0r\0o\0t\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceSubClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceProtocol as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::addr_of_mut!(EndBuffer),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 58], [u16; 29]>(
                        *b"U\0S\0B\0\\\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0&\0S\0u\0b\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                    (*InterfaceDescriptor).bInterfaceSubClass as core::ffi::c_int,
                );

                EndBuffer = EndBuffer.offset(1);
                Remaining = Remaining.wrapping_sub(core::mem::size_of::<WCHAR>());

                RtlStringCbPrintfExW(
                    EndBuffer as *mut u16,
                    Remaining,
                    core::ptr::null_mut::<*mut u16>(),
                    core::ptr::addr_of_mut!(Remaining),
                    0,
                    core::mem::transmute::<[u8; 30], [u16; 15]>(
                        *b"U\0S\0B\0\\\0C\0l\0a\0s\0s\0_\0%\x000\x002\0x\0\0\0",
                    ).as_ptr(),
                    (*InterfaceDescriptor).bInterfaceClass as core::ffi::c_int,
                );
            }

            Length = (core::mem::size_of::<[WCHAR; 200]>()).wrapping_sub(
                Remaining.wrapping_sub((2usize).wrapping_mul(core::mem::size_of::<WCHAR>())),
            );

            Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG as ULONG) as PWCHAR;

            if Id.is_null() {
                // break
            } else {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!(Buffer) as *const u8,
                    Id as *mut u8,
                    Length,
                );

                if !((*PortExtension).PortPdoFlags as u32
                    & USBHUB_PDO_FLAG_INIT_PORT_FAILED as u32
                    != 0)
                {
                    USBHUB_DumpingIDs(Id as *mut core::ffi::c_void);
                }
            }
        }
        3 => {
            // BusQueryInstanceID
            if !(*PortExtension).SerialNumber.is_null() {
                Id = ExAllocatePoolWithTag(
                    PagedPool,
                    (*PortExtension).SN_DescriptorLength as usize,
                    USB_HUB_TAG as ULONG,
                ) as PWCHAR;

                if !Id.is_null() {
                    core::ptr::write_bytes(
                        Id as *mut u8,
                        0,
                        (*PortExtension).SN_DescriptorLength as usize,
                    );

                    core::ptr::copy_nonoverlapping(
                        (*PortExtension).SerialNumber as *const u8,
                        Id as *mut u8,
                        (*PortExtension).SN_DescriptorLength as usize,
                    );
                }
            } else {
                Length = core::mem::size_of_val(&(*PortExtension).InstanceID)
                    .wrapping_add(core::mem::size_of::<WCHAR>());

                Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG as ULONG) as PWCHAR;

                if !Id.is_null() {
                    core::ptr::write_bytes(Id as *mut u8, 0, Length);

                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*PortExtension).InstanceID) as *const u8,
                        Id as *mut u8,
                        core::mem::size_of_val(&(*PortExtension).InstanceID),
                    );
                }
            }
        }
        _ => {
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.Information = Id as ULONG_PTR as _;

    if Id.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoQueryDeviceText(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    const LANG_ENGLISH: u32 = 0x9;
    const SUBLANG_ENGLISH_US: u32 = 0x1;
    const MAXIMUM_USB_STRING_LENGTH: u32 = 255;
    const USB_HUB_TAG: u32 = 0x42554855; // 'BHSU'
    const USBHUB_PDO_FLAG_INIT_PORT_FAILED: u32 = 0x00000004; // FIXED 2026-07-06: translation invented 0x200; real value is a (1<<N) define in usbhub.h

    let mut DeviceObject: PDEVICE_OBJECT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut DeviceTextType: DEVICE_TEXT_TYPE;
    let mut LanguageId: USHORT;
    let mut DefaultId: USHORT;
    let mut Descriptor: PUSB_STRING_DESCRIPTOR;
    let mut DeviceText: PWCHAR;
    let mut iProduct: UCHAR = 0 as UCHAR;
    let mut Status: NTSTATUS;
    let mut NumSymbols: usize;
    let mut Length: usize;

    DeviceObject = (*PortExtension).Common.SelfDevice;
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceTextType = (*IoStack).Parameters.QueryDeviceText.DeviceTextType;

    if DeviceTextType as u32 != DeviceTextDescription as u32
        && DeviceTextType as u32 != DeviceTextLocationInformation as u32
    {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    LanguageId = LANGIDFROMLCID((*IoStack).Parameters.QueryDeviceText.LocaleId) as USHORT;
    DefaultId = (((SUBLANG_ENGLISH_US as USHORT as i32) << 10)
        | (LANG_ENGLISH as USHORT as i32)) as USHORT;

    if LanguageId == 0 {
        LanguageId = DefaultId;
    }

    iProduct = (*PortExtension).DeviceDescriptor.iProduct;

    if !(*PortExtension).DeviceHandle.is_null()
        && iProduct as i32 != 0
        && (*PortExtension).IgnoringHwSerial == 0
        && ((*PortExtension).PortPdoFlags as u32 & USBHUB_PDO_FLAG_INIT_PORT_FAILED) == 0
    {
        Descriptor = ExAllocatePoolWithTag(
            NonPagedPool,
            MAXIMUM_USB_STRING_LENGTH as usize,
            USB_HUB_TAG,
        ) as PUSB_STRING_DESCRIPTOR;

        if !Descriptor.is_null() {
            let current_block_44: u64;

            RtlZeroMemory(Descriptor as *mut core::ffi::c_void, MAXIMUM_USB_STRING_LENGTH as usize);

            Status = USBH_CheckDeviceLanguage(DeviceObject, LanguageId);
            loop {
                if Status >= 0 as NTSTATUS {
                    Status = USBH_SyncGetStringDescriptor(
                        DeviceObject,
                        iProduct,
                        LanguageId,
                        Descriptor,
                        MAXIMUM_USB_STRING_LENGTH,
                        core::ptr::null_mut(),
                        1 as BOOLEAN,
                    );

                    if Status >= 0 as NTSTATUS {
                        current_block_44 = 14763689060501151050;
                        break;
                    }
                }

                if LanguageId as i32 == DefaultId as i32 {
                    current_block_44 = 12122751017158166585;
                    break;
                }

                LanguageId = DefaultId;
                Status = USBH_CheckDeviceLanguage(DeviceObject, DefaultId);
            }

            match current_block_44 {
                14763689060501151050 => {
                    if (*Descriptor).bLength as usize
                        <= core::mem::size_of::<USB_COMMON_DESCRIPTOR>()
                    {
                        Status = STATUS_UNSUCCESSFUL;
                    }

                    if Status >= 0 as NTSTATUS {
                        Length = ((*Descriptor).bLength as i32
                            - core::mem::offset_of!(USB_STRING_DESCRIPTOR, bString) as i32)
                            as usize;

                        DeviceText = ExAllocatePoolWithTag(
                            PagedPool,
                            Length.wrapping_add(core::mem::size_of::<WCHAR>()),
                            USB_HUB_TAG,
                        ) as PWCHAR;

                        if !DeviceText.is_null() {
                            RtlZeroMemory(
                                DeviceText as *mut core::ffi::c_void,
                                Length.wrapping_add(core::mem::size_of::<WCHAR>()),
                            );

                            RtlCopyMemory(
                                DeviceText as *mut core::ffi::c_void,
                                core::ptr::addr_of_mut!((*Descriptor).bString) as *const core::ffi::c_void,
                                Length,
                            );

                            (*Irp).IoStatus.Information = DeviceText as ULONG_PTR as _;
                        } else {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        }
                    }
                }
                _ => {}
            }

            ExFreePoolWithTag(Descriptor as PVOID, USB_HUB_TAG);

            if Status >= 0 as NTSTATUS {
                return Status;
            }
        } else {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        Status = STATUS_NOT_SUPPORTED;
    }

    if GenericUSBDeviceString.is_null() {
        return Status;
    }

    NumSymbols = wcslen(GenericUSBDeviceString as *const u16);
    Length = NumSymbols.wrapping_add(1).wrapping_mul(core::mem::size_of::<WCHAR>());

    DeviceText = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG) as PWCHAR;

    if DeviceText.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(DeviceText as *mut core::ffi::c_void, Length);

    RtlCopyMemory(
        DeviceText as *mut core::ffi::c_void,
        GenericUSBDeviceString as *const core::ffi::c_void,
        NumSymbols.wrapping_mul(core::mem::size_of::<WCHAR>()),
    );

    (*Irp).IoStatus.Information = DeviceText as ULONG_PTR as _;

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_SymbolicLink(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    InterfaceClassGuid: *const GUID,
    IsEnable: BOOLEAN,
) -> NTSTATUS {
    const PLUGPLAY_REGKEY_DEVICE: u32 = 0x00000001;

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let NameBuffer: PVOID;

    if IsEnable != 0 {
        Status = IoRegisterDeviceInterface(
            (*PortExtension).Common.SelfDevice,
            InterfaceClassGuid,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*PortExtension).SymbolicLinkName),
        );

        if NT_SUCCESS(Status) {
            USBH_SetPdoRegistryParameter(
                (*PortExtension).Common.SelfDevice,
                w!("SymbolicName").as_ptr() as PCWSTR,
                (*PortExtension).SymbolicLinkName.Buffer as PVOID,
                (*PortExtension).SymbolicLinkName.Length as ULONG,
                REG_SZ as ULONG,
                PLUGPLAY_REGKEY_DEVICE as ULONG,
            );

            Status = IoSetDeviceInterfaceState(
                core::ptr::addr_of_mut!((*PortExtension).SymbolicLinkName),
                1 as BOOLEAN,
            );
        }
    } else {
        NameBuffer = (*PortExtension).SymbolicLinkName.Buffer as PVOID;

        if !NameBuffer.is_null() {
            Status = IoSetDeviceInterfaceState(
                core::ptr::addr_of_mut!((*PortExtension).SymbolicLinkName),
                0 as BOOLEAN,
            );

            ExFreePool((*PortExtension).SymbolicLinkName.Buffer as PVOID);

            (*PortExtension).SymbolicLinkName.Buffer = core::ptr::null_mut();
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_RestoreDevice(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    IsKeepDeviceData: BOOLEAN,
) -> NTSTATUS {

    const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as NTSTATUS;
    const USBHUB_PDO_FLAG_POWER_D3: u32 = 0x00000100; // FIXED 2026-07-06: translation invented 0x20; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_PORT_RESTORE_FAIL: u32 = 0x00001000; // FIXED 2026-07-06: translation invented 0x400; real value is a (1<<N) define in usbhub.h
    const USBHUB_PDO_FLAG_INIT_PORT_FAILED: u32 = 0x00000004; // FIXED 2026-07-06: translation invented 0x80; real value is a (1<<N) define in usbhub.h

    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let PortData: PUSBHUB_PORT_DATA;
    let mut Status: NTSTATUS;
    let mut ix: ULONG;

    // DPRINT("USBH_RestoreDevice ... \n");

    HubExtension = (*PortExtension).HubExtension;

    if HubExtension.is_null() {
        Status = STATUS_UNSUCCESSFUL;
        return Status;
    }

    // ASSERT(PortExtension->PortNumber > 0);
    PortData = (*HubExtension)
        .PortData
        .offset(((*PortExtension).PortNumber as i32 - 1) as isize) as PUSBHUB_PORT_DATA;

    if (*PortExtension).Common.SelfDevice != (*PortData).DeviceObject {
        Status = STATUS_UNSUCCESSFUL;
        return Status;
    }

    Status = USBH_SyncGetPortStatus(
        HubExtension,
        (*PortExtension).PortNumber,
        core::ptr::addr_of_mut!((*PortData).PortStatus) as PVOID,
        core::mem::size_of::<USB_PORT_STATUS_AND_CHANGE>() as ULONG,
    );

    if NT_SUCCESS(Status) {
        ix = 0;
        while ix < 3 {
            Status = USBH_ResetDevice(
                HubExtension as PUSBHUB_FDO_EXTENSION,
                (*PortExtension).PortNumber,
                IsKeepDeviceData,
                (ix == 0) as i32 as BOOLEAN,
            );

            if NT_SUCCESS(Status) || Status == STATUS_NO_SUCH_DEVICE {
                break;
            }

            USBH_Wait(1000);
            ix = ix.wrapping_add(1);
        }
    }

    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_POWER_D3;

    if NT_SUCCESS(Status) {
        (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_PORT_RESTORE_FAIL;
    } else {
        (*PortExtension).PortPdoFlags |=
            USBHUB_PDO_FLAG_INIT_PORT_FAILED | USBHUB_PDO_FLAG_PORT_RESTORE_FAIL;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoStopDevice(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBH_PdoStopDevice: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/pnp.c\0"
            as *const u8 as *const core::ffi::c_char,
        2355 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/usb/usbhub/pnp.c\0"
                as *const u8 as *const core::ffi::c_char,
            2355 as core::ffi::c_int,
        );
    }
    DbgBreakPoint();
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Hub Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBHub plug and play functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbhub.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBHUB_PNP
// #define NDEBUG_USBHUB_ENUM
// #include "dbg_uhub.h"
//
// NTSTATUS
// NTAPI
// USBH_IrpCompletion(IN PDEVICE_OBJECT DeviceObject,
//                    IN PIRP Irp,
//                    IN PVOID Context)
// {
//     PRKEVENT Event;
//
//     DPRINT("USBH_IrpCompletion: Irp - %p\n", Irp);
//
//     Event = Context;
//     KeSetEvent(Event, EVENT_INCREMENT, FALSE);
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_HubPnPIrpComplete(IN PDEVICE_OBJECT DeviceObject,
//                        IN PIRP Irp,
//                        IN PVOID Context)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//
//     DPRINT("USBH_HubPnPIrpComplete: Irp - %p\n", Irp);
//
//      HubExtension = Context;
//
//     if (!NT_SUCCESS(Irp->IoStatus.Status))
//     {
//         DPRINT1("USBH_HubPnPIrpComplete: Irp failed - %lX\n", Irp->IoStatus.Status);
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//     }
//
//     Irp->IoStatus.Status = STATUS_MORE_PROCESSING_REQUIRED;
//
//     KeSetEvent(&HubExtension->LowerDeviceEvent, EVENT_INCREMENT, FALSE);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBH_QueryCapsComplete(IN PDEVICE_OBJECT DeviceObject,
//                        IN PIRP Irp,
//                        IN PVOID Context)
// {
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_CAPABILITIES Capabilities;
//
//     DPRINT("USBH_QueryCapsComplete: ... \n");
//
//     ASSERT(NT_SUCCESS(Irp->IoStatus.Status));
//
//     if (Irp->PendingReturned)
//     {
//         IoMarkIrpPending(Irp);
//     }
//
//     IoStack= IoGetCurrentIrpStackLocation(Irp);
//     Capabilities = IoStack->Parameters.DeviceCapabilities.Capabilities;
//
//     Capabilities->SurpriseRemovalOK = 1;
//
//     return STATUS_CONTINUE_COMPLETION;
// }
//
// NTSTATUS
// NTAPI
// USBHUB_GetBusInterface(IN PDEVICE_OBJECT DeviceObject,
//                        OUT PUSB_BUS_INTERFACE_HUB_V5 BusInterface)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//
//     DPRINT("USBHUB_GetBusInterface: ... \n");
//
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//
//     if (!Irp)
//     {
//         DPRINT1("USBHUB_GetBusInterface: IoAllocateIrp() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_IrpCompletion,
//                            &Event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_PNP;
//     IoStack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//
//     IoStack->Parameters.QueryInterface.InterfaceType = &USB_BUS_INTERFACE_HUB_GUID;
//     IoStack->Parameters.QueryInterface.Size = sizeof(USB_BUS_INTERFACE_HUB_V5);
//     IoStack->Parameters.QueryInterface.Version = USB_BUSIF_HUB_VERSION_5;
//     IoStack->Parameters.QueryInterface.Interface = (PINTERFACE)BusInterface;
//     IoStack->Parameters.QueryInterface.InterfaceSpecificData = DeviceObject;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     IoFreeIrp(Irp);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBHUB_GetBusInterfaceUSBDI(IN PDEVICE_OBJECT DeviceObject,
//                             OUT PUSB_BUS_INTERFACE_USBDI_V2 BusInterfaceUSBDI)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//
//     DPRINT("USBHUB_GetBusInterfaceUSBDI: ... \n");
//
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//
//     if (!Irp)
//     {
//         DPRINT1("USBHUB_GetBusInterfaceUSBDI: IoAllocateIrp() failed\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_IrpCompletion,
//                            &Event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_PNP;
//     IoStack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//
//     IoStack->Parameters.QueryInterface.InterfaceType = &USB_BUS_INTERFACE_USBDI_GUID;
//     IoStack->Parameters.QueryInterface.Size = sizeof(USB_BUS_INTERFACE_USBDI_V2);
//     IoStack->Parameters.QueryInterface.Version = USB_BUSIF_USBDI_VERSION_2;
//     IoStack->Parameters.QueryInterface.Interface = (PINTERFACE)BusInterfaceUSBDI;
//     IoStack->Parameters.QueryInterface.InterfaceSpecificData = NULL;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     IoFreeIrp(Irp);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_QueryCapabilities(IN PDEVICE_OBJECT DeviceObject,
//                        IN PDEVICE_CAPABILITIES DeviceCapabilities)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//
//     DPRINT("USBH_QueryCapabilities: ... \n");
//
//     RtlZeroMemory(DeviceCapabilities, sizeof(DEVICE_CAPABILITIES));
//
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//
//     if (!Irp)
//     {
//         DPRINT1("USBH_QueryCapabilities: IoAllocateIrp() failed\n");
//         return;
//     }
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_IrpCompletion,
//                            &Event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     IoStack->MajorFunction = IRP_MJ_PNP;
//     IoStack->MinorFunction = IRP_MN_QUERY_CAPABILITIES;
//
//     IoStack->Parameters.DeviceCapabilities.Capabilities = DeviceCapabilities;
//     IoStack->Parameters.DeviceCapabilities.Capabilities->Size = sizeof(DEVICE_CAPABILITIES);
//     IoStack->Parameters.DeviceCapabilities.Capabilities->Version = 1;
//     IoStack->Parameters.DeviceCapabilities.Capabilities->Address = MAXULONG;
//     IoStack->Parameters.DeviceCapabilities.Capabilities->UINumber = MAXULONG;
//
//     if (IoCallDriver(DeviceObject, Irp) == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     IoFreeIrp(Irp);
// }
//
// NTSTATUS
// NTAPI
// USBH_OpenConfiguration(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_INTERFACE_DESCRIPTOR Pid;
//     PURB Urb;
//     NTSTATUS Status;
//     USBD_INTERFACE_LIST_ENTRY InterfaceList[2] = {{NULL, NULL}, {NULL, NULL}};
//
//     DPRINT("USBH_OpenConfiguration ... \n");
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB &&
//         HubExtension->LowerPDO != HubExtension->RootHubPdo)
//     {
//         Pid = USBD_ParseConfigurationDescriptorEx(HubExtension->HubConfigDescriptor,
//                                                   HubExtension->HubConfigDescriptor,
//                                                   -1,
//                                                   -1,
//                                                   USB_DEVICE_CLASS_HUB,
//                                                   -1,
//                                                   2);
//
//         if (Pid)
//         {
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_MULTIPLE_TTS;
//         }
//         else
//         {
//             Pid = USBD_ParseConfigurationDescriptorEx(HubExtension->HubConfigDescriptor,
//                                                       HubExtension->HubConfigDescriptor,
//                                                       -1,
//                                                       -1,
//                                                       USB_DEVICE_CLASS_HUB,
//                                                       -1,
//                                                       1);
//
//             if (Pid)
//             {
//                 goto Next;
//             }
//
//             Pid = USBD_ParseConfigurationDescriptorEx(HubExtension->HubConfigDescriptor,
//                                                       HubExtension->HubConfigDescriptor,
//                                                       -1,
//                                                       -1,
//                                                       USB_DEVICE_CLASS_HUB,
//                                                       -1,
//                                                       0);
//         }
//     }
//     else
//     {
//         Pid = USBD_ParseConfigurationDescriptorEx(HubExtension->HubConfigDescriptor,
//                                                   HubExtension->HubConfigDescriptor,
//                                                   -1,
//                                                   -1,
//                                                   USB_DEVICE_CLASS_HUB,
//                                                   -1,
//                                                   -1);
//     }
//
//     if (!Pid)
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//   Next:
//
//     if (Pid->bInterfaceClass != USB_DEVICE_CLASS_HUB)
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     InterfaceList[0].InterfaceDescriptor = Pid;
//
//     Urb = USBD_CreateConfigurationRequestEx(HubExtension->HubConfigDescriptor,
//                                             InterfaceList);
//
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = USBH_FdoSyncSubmitUrb(HubExtension->Common.SelfDevice, Urb);
//
//     if (NT_SUCCESS(Status))
//     {
//         RtlCopyMemory(&HubExtension->PipeInfo,
//                       InterfaceList[0].Interface->Pipes,
//                       sizeof(USBD_PIPE_INFORMATION));
//
//         HubExtension->ConfigHandle = Urb->UrbSelectConfiguration.ConfigurationHandle;
//     }
//
//     ExFreePool(Urb);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBD_Initialize20Hub(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PUSB_BUSIFFN_INITIALIZE_20HUB Initialize20Hub;
//     ULONG TtCount;
//     PUSB_DEVICE_HANDLE DeviceHandle;
//
//     DPRINT("USBD_InitUsb2Hub ... \n");
//
//     Initialize20Hub = HubExtension->BusInterface.Initialize20Hub;
//
//     if (!Initialize20Hub)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     TtCount = 1;
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_MULTIPLE_TTS)
//     {
//         TtCount = HubExtension->HubDescriptor->bNumberOfPorts;
//     }
//
//     DeviceHandle = USBH_SyncGetDeviceHandle(HubExtension->LowerDevice);
//
//     return Initialize20Hub(HubExtension->BusInterface.BusContext,
//                            DeviceHandle,
//                            TtCount);
// }
//
// NTSTATUS
// NTAPI
// USBH_AbortInterruptPipe(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     struct _URB_PIPE_REQUEST * Urb;
//     NTSTATUS Status;
//
//     DPRINT("USBH_AbortInterruptPipe: HubExtension - %p\n", HubExtension);
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_PIPE_REQUEST),
//                                 USB_HUB_TAG);
//
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Urb, sizeof(struct _URB_PIPE_REQUEST));
//
//     Urb->Hdr.Length = sizeof(struct _URB_PIPE_REQUEST);
//     Urb->Hdr.Function = URB_FUNCTION_ABORT_PIPE;
//     Urb->PipeHandle = HubExtension->PipeInfo.PipeHandle;
//
//     Status = USBH_FdoSyncSubmitUrb(HubExtension->Common.SelfDevice,
//                                    (PURB)Urb);
//
//     if (NT_SUCCESS(Status))
//     {
//         KeWaitForSingleObject(&HubExtension->StatusChangeEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     ExFreePoolWithTag(Urb, USB_HUB_TAG);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_FdoCleanup(IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     PIRP IdleIrp = NULL;
//     PIRP WakeIrp = NULL;
//     PUSBHUB_PORT_DATA PortData;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PIRP PortIdleIrp = NULL;
//     PIRP PortWakeIrp = NULL;
//     PVOID DeviceHandle;
//     NTSTATUS Status;
//     USHORT Port;
//     UCHAR NumberPorts;
//     KIRQL Irql;
//
//     DPRINT("USBH_FdoCleanup: HubExtension - %p\n", HubExtension);
//
//     USBD_UnRegisterRootHubCallBack(HubExtension);
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_STOPPING;
//
//     if (HubExtension->ResetRequestCount)
//     {
//         IoCancelIrp(HubExtension->ResetPortIrp);
//
//         KeWaitForSingleObject(&HubExtension->IdleEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     IoFreeIrp(HubExtension->ResetPortIrp);
//
//     HubExtension->ResetPortIrp = NULL;
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST)
//     {
//         KeWaitForSingleObject(&HubExtension->IdleEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     IoAcquireCancelSpinLock(&Irql);
//
//     if (HubExtension->PendingWakeIrp)
//     {
//         WakeIrp = HubExtension->PendingWakeIrp;
//         HubExtension->PendingWakeIrp = NULL;
//     }
//
//     if (HubExtension->PendingIdleIrp)
//     {
//         IdleIrp = HubExtension->PendingIdleIrp;
//         HubExtension->PendingIdleIrp = NULL;
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     if (WakeIrp)
//     {
//         USBH_HubCancelWakeIrp(HubExtension, WakeIrp);
//     }
//
//     USBH_HubCompletePortWakeIrps(HubExtension, STATUS_DELETE_PENDING);
//
//     if (IdleIrp)
//     {
//         USBH_HubCancelIdleIrp(HubExtension, IdleIrp);
//     }
//
//     if (InterlockedDecrement(&HubExtension->PendingRequestCount) > 0)
//     {
//         KeWaitForSingleObject(&HubExtension->PendingRequestEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     if (HubExtension->SCEIrp)
//     {
//         Status = USBH_AbortInterruptPipe(HubExtension);
//
//         if (!NT_SUCCESS(Status) && IoCancelIrp(HubExtension->SCEIrp))
//         {
//             KeWaitForSingleObject(&HubExtension->StatusChangeEvent,
//                                   Suspended,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//         }
//
//         IoFreeIrp(HubExtension->SCEIrp);
//
//         HubExtension->SCEIrp = NULL;
//     }
//
//     if (!HubExtension->PortData ||
//         !HubExtension->HubDescriptor)
//     {
//         goto Exit;
//     }
//
//     PortData = HubExtension->PortData;
//     NumberPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     for (Port = 0; Port < NumberPorts; Port++)
//     {
//         if (PortData[Port].DeviceObject)
//         {
//             PortExtension = PortData[Port].DeviceObject->DeviceExtension;
//
//             IoAcquireCancelSpinLock(&Irql);
//
//             PortIdleIrp = PortExtension->IdleNotificationIrp;
//
//             if (PortIdleIrp)
//             {
//                 PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_IDLE_NOTIFICATION;
//                 PortExtension->IdleNotificationIrp = NULL;
//
//                 if (PortIdleIrp->Cancel)
//                 {
//                     PortIdleIrp = NULL;
//                 }
//
//                 if (PortIdleIrp)
//                 {
//                     IoSetCancelRoutine(PortIdleIrp, NULL);
//                 }
//             }
//
//             PortWakeIrp = PortExtension->PdoWaitWakeIrp;
//
//             if (PortWakeIrp)
//             {
//                 PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_WAIT_WAKE;
//                 PortExtension->PdoWaitWakeIrp = NULL;
//
//                 if (PortWakeIrp->Cancel || !IoSetCancelRoutine(PortWakeIrp, NULL))
//                 {
//                     PortWakeIrp = NULL;
//
//                     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//                     {
//                         KeSetEvent(&HubExtension->PendingRequestEvent,
//                                    EVENT_INCREMENT,
//                                    FALSE);
//                     }
//                 }
//             }
//
//             IoReleaseCancelSpinLock(Irql);
//
//             if (PortIdleIrp)
//             {
//                 PortIdleIrp->IoStatus.Status = STATUS_CANCELLED;
//                 IoCompleteRequest(PortIdleIrp, IO_NO_INCREMENT);
//             }
//
//             if (PortWakeIrp)
//             {
//                 USBH_CompletePowerIrp(HubExtension,
//                                       PortWakeIrp,
//                                       STATUS_CANCELLED);
//             }
//
//             if (!(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3))
//             {
//                 DeviceHandle = InterlockedExchangePointer(&PortExtension->DeviceHandle,
//                                                           NULL);
//
//                 if (DeviceHandle)
//                 {
//                     USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
//                 }
//
//                 PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_POWER_D3;
//             }
//         }
//
//         USBH_SyncDisablePort(HubExtension, Port + 1);
//     }
//
// Exit:
//
//     if (HubExtension->SCEBitmap)
//     {
//         ExFreePoolWithTag(HubExtension->SCEBitmap, USB_HUB_TAG);
//     }
//
//     if (HubExtension->HubDescriptor)
//     {
//         ExFreePoolWithTag(HubExtension->HubDescriptor, USB_HUB_TAG);
//     }
//
//     if (HubExtension->HubConfigDescriptor)
//     {
//         ExFreePoolWithTag(HubExtension->HubConfigDescriptor, USB_HUB_TAG);
//     }
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_DEVICE_STARTED;
//
//     HubExtension->HubDescriptor = NULL;
//     HubExtension->HubConfigDescriptor = NULL;
//
//     HubExtension->SCEIrp = NULL;
//     HubExtension->SCEBitmap = NULL;
// }
//
// NTSTATUS
// NTAPI
// USBH_StartHubFdoDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                        IN PIRP Irp)
// {
//     NTSTATUS Status;
//     ULONG DisableRemoteWakeup = 0;
//     ULONG HubCount = 0;
//     PUSB_DEVICE_HANDLE DeviceHandle;
//     USB_DEVICE_TYPE DeviceType;
//     DEVICE_CAPABILITIES  DeviceCapabilities;
//     BOOLEAN IsBusPowered;
//     static WCHAR DisableWakeValueName[] = L"DisableRemoteWakeup";
//
//     DPRINT("USBH_StartHubFdoDevice: ... \n");
//
//     KeInitializeEvent(&HubExtension->IdleEvent, NotificationEvent, FALSE);
//     KeInitializeEvent(&HubExtension->ResetEvent, NotificationEvent, TRUE);
//     KeInitializeEvent(&HubExtension->PendingRequestEvent, NotificationEvent, FALSE);
//     KeInitializeEvent(&HubExtension->LowerDeviceEvent, NotificationEvent, FALSE);
//     KeInitializeEvent(&HubExtension->StatusChangeEvent, NotificationEvent, TRUE);
//     KeInitializeEvent(&HubExtension->RootHubNotificationEvent,
//                       NotificationEvent,
//                       TRUE);
//
//     KeInitializeSpinLock(&HubExtension->RelationsWorkerSpinLock);
//     KeInitializeSpinLock(&HubExtension->CheckIdleSpinLock);
//
//     KeInitializeSemaphore(&HubExtension->ResetDeviceSemaphore, 1, 1);
//     KeInitializeSemaphore(&HubExtension->HubPortSemaphore, 1, 1);
//     KeInitializeSemaphore(&HubExtension->HubSemaphore, 1, 1);
//
//     HubExtension->HubFlags = 0;
//     HubExtension->HubConfigDescriptor = NULL;
//     HubExtension->HubDescriptor = NULL;
//     HubExtension->SCEIrp = NULL;
//     HubExtension->SCEBitmap = NULL;
//     HubExtension->SystemPowerState.SystemState = PowerSystemWorking;
//     HubExtension->PendingRequestCount = 1;
//     HubExtension->ResetRequestCount = 0;
//     HubExtension->PendingIdleIrp = NULL;
//     HubExtension->PendingWakeIrp = NULL;
//
//     InitializeListHead(&HubExtension->PdoList);
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_WITEM_INIT;
//     InitializeListHead(&HubExtension->WorkItemList);
//     KeInitializeSpinLock(&HubExtension->WorkItemSpinLock);
//
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//     IoSetCompletionRoutine(Irp,
//                            USBH_HubPnPIrpComplete,
//                            HubExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     if (IoCallDriver(HubExtension->LowerDevice, Irp) == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&HubExtension->LowerDeviceEvent,
//                               Suspended,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//     }
//
//     HubExtension->RootHubPdo = NULL;
//
//     Status = USBH_SyncGetRootHubPdo(HubExtension->LowerDevice,
//                                     &HubExtension->RootHubPdo,
//                                     &HubExtension->RootHubPdo2);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_SyncGetRootHubPdo() failed - %lX\n", Status);
//         goto ErrorExit;
//     }
//
//     USBH_WriteFailReasonID(HubExtension->LowerPDO, USBHUB_FAIL_NO_FAIL);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED)
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBHUB_FDO_FLAG_DEVICE_FAILED - TRUE\n");
//         Status = STATUS_UNSUCCESSFUL;
//         goto ErrorExit;
//     }
//
//     HubExtension->HubFlags |= USBHUB_FDO_FLAG_REMOTE_WAKEUP;
//
//     Status = USBD_GetPdoRegistryParameter(HubExtension->LowerPDO,
//                                           &DisableRemoteWakeup,
//                                           sizeof(DisableRemoteWakeup),
//                                           DisableWakeValueName,
//                                           sizeof(DisableWakeValueName));
//
//     if (NT_SUCCESS(Status) && DisableRemoteWakeup)
//     {
//         DPRINT("USBH_StartHubFdoDevice: DisableRemoteWakeup - TRUE\n");
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_REMOTE_WAKEUP;
//     }
//
//     HubExtension->CurrentPowerState.DeviceState = PowerDeviceD0;
//
//     USBH_SyncGetHubCount(HubExtension->LowerDevice,
//                          &HubCount);
//
//     Status = USBHUB_GetBusInterface(HubExtension->RootHubPdo,
//                                     &HubExtension->BusInterface);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBHUB_GetBusInterface() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     Status = USBHUB_GetBusInterfaceUSBDI(HubExtension->LowerDevice,
//                                          &HubExtension->BusInterfaceUSBDI);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBHUB_GetBusInterfaceUSBDI() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     DeviceHandle = USBH_SyncGetDeviceHandle(HubExtension->LowerDevice);
//
//     if (DeviceHandle)
//     {
//         Status = USBH_GetDeviceType(HubExtension, DeviceHandle, &DeviceType);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("USBH_StartHubFdoDevice: USBH_GetDeviceType() failed - %lX\n",
//                     Status);
//
//             goto ErrorExit;
//         }
//
//         if (DeviceType == Usb20Device)
//         {
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_USB20_HUB;
//         }
//     }
//
//     if (HubCount > USBHUB_MAX_CASCADE_LEVELS)
//     {
//         PUSBHUB_PORT_PDO_EXTENSION ParentPdoExtension;
//         PUSBHUB_FDO_EXTENSION ParentHubExtension;
//         USHORT ParentPort;
//         PUSBHUB_PORT_DATA PortData;
//
//         DPRINT1("USBH_StartHubFdoDevice: HubCount > 6 - %x\n", HubCount);
//
//         USBH_WriteFailReasonID(HubExtension->LowerPDO,
//                                USBHUB_FAIL_NESTED_TOO_DEEPLY);
//
//         ParentPdoExtension = HubExtension->LowerPDO->DeviceExtension;
//         ParentHubExtension = ParentPdoExtension->HubExtension;
//
//         ParentPort = ParentPdoExtension->PortNumber - 1;
//         PortData = &ParentHubExtension->PortData[ParentPort];
//         PortData->ConnectionStatus = DeviceHubNestedTooDeeply;
//
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//     }
//
//     USBH_QueryCapabilities(HubExtension->LowerDevice, &DeviceCapabilities);
//
//     HubExtension->SystemWake = DeviceCapabilities.SystemWake;
//     HubExtension->DeviceWake = DeviceCapabilities.DeviceWake;
//
//     RtlCopyMemory(HubExtension->DeviceState,
//                   &DeviceCapabilities.DeviceState,
//                   POWER_SYSTEM_MAXIMUM * sizeof(DEVICE_POWER_STATE));
//
//     Status = USBH_GetDeviceDescriptor(HubExtension->Common.SelfDevice,
//                                       &HubExtension->HubDeviceDescriptor);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBH_GetDeviceDescriptor() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     Status = USBH_GetConfigurationDescriptor(HubExtension->Common.SelfDevice,
//                                              &HubExtension->HubConfigDescriptor);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBH_GetConfigurationDescriptor() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     Status = USBH_SyncGetHubDescriptor(HubExtension);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBH_SyncGetHubDescriptor() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     IsBusPowered = USBH_HubIsBusPowered(HubExtension->Common.SelfDevice,
//                                         HubExtension->HubConfigDescriptor);
//
//     if (IsBusPowered)
//     {
//         /* bus-powered hub is allowed a maximum of 100 mA only for each port */
//         HubExtension->MaxPowerPerPort = 100;
//
//         /* can have 4 ports (4 * 100 mA) and 100 mA remains for itself;
//            expressed in 2 mA units (i.e., 250 = 500 mA). */
//         HubExtension->HubConfigDescriptor->MaxPower = 250;
//     }
//     else
//     {
//         /* self-powered hub is allowed a maximum of 500 mA for each port */
//         HubExtension->MaxPowerPerPort = 500;
//     }
//
//     Status = USBH_OpenConfiguration(HubExtension);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBH_StartHubFdoDevice: USBH_OpenConfiguration() failed - %lX\n",
//                 Status);
//         goto ErrorExit;
//     }
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB)
//     {
//         Status = USBD_Initialize20Hub(HubExtension);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//
//     HubExtension->SCEIrp = IoAllocateIrp(HubExtension->Common.SelfDevice->StackSize,
//                                          FALSE);
//
//     HubExtension->ResetPortIrp = IoAllocateIrp(HubExtension->Common.SelfDevice->StackSize,
//                                                FALSE);
//
//     if (!HubExtension->SCEIrp || !HubExtension->ResetPortIrp)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ErrorExit;
//     }
//
//     HubExtension->SCEBitmapLength = HubExtension->PipeInfo.MaximumPacketSize;
//
//     HubExtension->SCEBitmap = ExAllocatePoolWithTag(NonPagedPool,
//                                                     HubExtension->SCEBitmapLength,
//                                                     USB_HUB_TAG);
//
//     if (!HubExtension->SCEBitmap)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ErrorExit;
//     }
//
//     RtlZeroMemory(HubExtension->SCEBitmap, HubExtension->SCEBitmapLength);
//
//     Status = USBH_SyncPowerOnPorts(HubExtension);
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto ErrorExit;
//     }
//     else
//     {
//         USHORT Port;
//
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_STARTED;
//
//         for (Port = 1;
//              Port <= HubExtension->HubDescriptor->bNumberOfPorts;
//              Port++)
//         {
//             USBH_SyncClearPortStatus(HubExtension,
//                                      Port,
//                                      USBHUB_FEATURE_C_PORT_CONNECTION);
//         }
//     }
//
//     if (HubExtension->LowerPDO == HubExtension->RootHubPdo)
//     {
//         USBD_RegisterRootHubCallBack(HubExtension);
//     }
//     else
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DO_ENUMERATION;
//         USBH_SubmitStatusChangeTransfer(HubExtension);
//     }
//
//     goto Exit;
//
//   ErrorExit:
//
//     if (HubExtension->HubDescriptor)
//     {
//         ExFreePoolWithTag(HubExtension->HubDescriptor, USB_HUB_TAG);
//         HubExtension->HubDescriptor = NULL;
//     }
//
//     if (HubExtension->SCEIrp)
//     {
//         IoFreeIrp(HubExtension->SCEIrp);
//         HubExtension->SCEIrp = NULL;
//     }
//
//     if (HubExtension->ResetPortIrp)
//     {
//         IoFreeIrp(HubExtension->ResetPortIrp);
//         HubExtension->ResetPortIrp = NULL;
//     }
//
//     if (HubExtension->SCEBitmap)
//     {
//         ExFreePoolWithTag(HubExtension->SCEBitmap, USB_HUB_TAG);
//         HubExtension->SCEBitmap = NULL;
//     }
//
//     if (HubExtension->HubConfigDescriptor)
//     {
//         ExFreePoolWithTag(HubExtension->HubConfigDescriptor, USB_HUB_TAG);
//         HubExtension->HubConfigDescriptor = NULL;
//     }
//
//   Exit:
//
//     USBH_CompleteIrp(Irp, Status);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoStartDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                     IN PIRP Irp)
// {
//     NTSTATUS Status;
//
//     DPRINT("USBH_FdoStartDevice: HubExtension - %p\n", HubExtension);
//
//     HubExtension->RootHubPdo = NULL;
//
//     Status = USBH_SyncGetRootHubPdo(HubExtension->LowerDevice,
//                                     &HubExtension->RootHubPdo,
//                                     &HubExtension->RootHubPdo2);
//
//     if (NT_SUCCESS(Status))
//     {
//         if (HubExtension->RootHubPdo)
//         {
//             Status = USBH_StartHubFdoDevice(HubExtension, Irp);
//         }
//         else
//         {
//             DPRINT1("USBH_FdoStartDevice: FIXME. start ParentDevice\n");
//             DbgBreakPoint();
//         }
//     }
//     else
//     {
//         DPRINT1("USBH_FdoStartDevice: FIXME. USBH_SyncGetRootHubPdo return - %lX\n",
//                 Status);
//
//         DbgBreakPoint();
//         USBH_CompleteIrp(Irp, Status);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoQueryBusRelations(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                           IN PIRP Irp)
// {
//     PDEVICE_RELATIONS DeviceRelations = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//     LIST_ENTRY GhostPdoList;
//     KIRQL OldIrql;
//     PLIST_ENTRY PdoList;
//     UCHAR NumberPorts;
//     USHORT Port;
//     USHORT GhostPort;
//     PUSBHUB_PORT_DATA PortData;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PdoExtension;
//     PUSBHUB_PORT_PDO_EXTENSION pdoExtension;
//     NTSTATUS NtStatus;
//     PVOID SerialNumber;
//     PVOID DeviceHandle;
//     USB_PORT_STATUS UsbPortStatus;
//     PLIST_ENTRY Entry;
//     ULONG Length;
//
//     DPRINT_ENUM("USBH_FdoQueryBusRelations: HubFlags - %lX\n",
//                 HubExtension->HubFlags);
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED))
//     {
//        Status = STATUS_INVALID_DEVICE_STATE;
//        goto RelationsWorker;
//     }
//
//     if (!HubExtension->HubDescriptor)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         goto RelationsWorker;
//     }
//
//     if (!(HubExtension->HubFlags & USBHUB_FDO_FLAG_DO_ENUMERATION))
//     {
//         // FIXME: this delay makes devices discovery during early boot more reliable
//         LARGE_INTEGER Interval;
//         Status = STATUS_SUCCESS;
//         IoInvalidateDeviceRelations(HubExtension->LowerPDO, BusRelations);
//         Interval.QuadPart = -10000LL * 1000; // 1 sec.
//         KeDelayExecutionThread(KernelMode, FALSE, &Interval);
//
//         DPRINT_ENUM("USBH_FdoQueryBusRelations: Skip enumeration\n");
//         goto RelationsWorker;
//     }
//
//     InterlockedIncrement(&HubExtension->PendingRequestCount);
//
//     KeWaitForSingleObject(&HubExtension->ResetDeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     NumberPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//     DPRINT_ENUM("USBH_FdoQueryBusRelations: NumberPorts - %x\n", NumberPorts);
//
//     Length = FIELD_OFFSET(DEVICE_RELATIONS, Objects) +
//              NumberPorts * sizeof(PDEVICE_OBJECT);
//
//     if (Irp->IoStatus.Information)
//     {
//         DPRINT1("FIXME: leaking old bus relations\n");
//     }
//
//     DeviceRelations = ExAllocatePoolWithTag(NonPagedPool, Length, USB_HUB_TAG);
//
//     if (!DeviceRelations)
//     {
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_NOT_ENUMERATED;
//
//         KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//         {
//             KeSetEvent(&HubExtension->PendingRequestEvent, EVENT_INCREMENT, FALSE);
//         }
//
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto RelationsWorker;
//     }
//
//     RtlZeroMemory(DeviceRelations, Length);
//
//     DeviceRelations->Count = 0;
//
// EnumStart:
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_ESD_RECOVERING)
//     {
//         HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_NOT_ENUMERATED;
//
//         KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//         {
//             KeSetEvent(&HubExtension->PendingRequestEvent, EVENT_INCREMENT, FALSE);
//         }
//
//         Status = STATUS_SUCCESS;
//         goto RelationsWorker;
//     }
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_ENUM_POST_RECOVER;
//
//     for (Port = 1; Port <= NumberPorts; Port++)
//     {
//         PortData = &HubExtension->PortData[Port - 1];
//
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED)
//         {
//             continue;
//         }
//
//         Status = USBH_SyncGetPortStatus(HubExtension,
//                                         Port,
//                                         &PortData->PortStatus,
//                                         sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT_ENUM("USBH_FdoQueryBusRelations: Status - %X\n", Status);
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//             DeviceRelations->Count = 0;
//             goto EnumStart;
//         }
//
//         DPRINT_ENUM("USBH_FdoQueryBusRelations: Port - %x, ConnectStatus - %x\n",
//                     Port,
//                     PortData->PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus);
//
//         PdoDevice = PortData->DeviceObject;
//
//         if (PortData->DeviceObject)
//         {
//             PdoExtension = PdoDevice->DeviceExtension;
//
//             if (PdoExtension->PortPdoFlags & USBHUB_PDO_FLAG_OVERCURRENT_PORT)
//             {
//                 PortData->PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus = 1;
//             }
//         }
//
//         if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED)
//         {
//             DPRINT1("USBH_FdoQueryBusRelations: DbgBreakPoint() \n");
//             DbgBreakPoint();
//         }
//
//         if (!PortData->PortStatus.PortStatus.Usb20PortStatus.CurrentConnectStatus)
//         {
//             if (PdoDevice)
//             {
//                 PdoExtension = PdoDevice->DeviceExtension;
//
//                 PdoExtension->PortPdoFlags |= USBHUB_PDO_FLAG_DELETE_PENDING;
//                 PdoExtension->EnumFlags &= ~USBHUB_ENUM_FLAG_DEVICE_PRESENT;
//
//                 SerialNumber = InterlockedExchangePointer((PVOID)&PdoExtension->SerialNumber,
//                                                           NULL);
//
//                 if (SerialNumber)
//                 {
//                     ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
//                 }
//
//                 DeviceHandle = InterlockedExchangePointer(&PdoExtension->DeviceHandle,
//                                                           NULL);
//
//                 if (DeviceHandle)
//                 {
//                     USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
//                     USBH_SyncDisablePort(HubExtension, Port);
//                 }
//             }
//
//             PortData->DeviceObject = NULL;
//             PortData->ConnectionStatus = NoDeviceConnected;
//             continue;
//         }
//
//         if (PdoDevice)
//         {
//             ObReferenceObject(PdoDevice);
//
//             PdoDevice->Flags |= DO_POWER_PAGABLE;
//             PdoDevice->Flags &= ~DO_DEVICE_INITIALIZING;
//
//             DeviceRelations->Objects[DeviceRelations->Count++] = PdoDevice;
//
//             PdoExtension = PdoDevice->DeviceExtension;
//             PdoExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_POWER_D1_OR_D2;
//
//             continue;
//         }
//
//         USBH_Wait(100);
//
//         NtStatus = USBH_SyncResetPort(HubExtension, Port);
//
//         if (!NT_SUCCESS(NtStatus))
//         {
//             if (HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB)
//             {
//                 PortData->DeviceObject = NULL;
//                 PortData->ConnectionStatus = NoDeviceConnected;
//                 continue;
//             }
//         }
//         else
//         {
//             NtStatus = USBH_SyncGetPortStatus(HubExtension,
//                                               Port,
//                                               &PortData->PortStatus,
//                                               sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//             UsbPortStatus = PortData->PortStatus.PortStatus;
//
//             if (NT_SUCCESS(NtStatus))
//             {
//                 ULONG ix = 0;
//
//                 for (NtStatus = USBH_CreateDevice(HubExtension, Port, UsbPortStatus, ix);
//                      !NT_SUCCESS(NtStatus);
//                      NtStatus = USBH_CreateDevice(HubExtension, Port, UsbPortStatus, ix))
//                 {
//                     USBH_Wait(500);
//
//                     if (ix >= 2)
//                     {
//                         break;
//                     }
//
//                     if (PortData->DeviceObject)
//                     {
//                         IoDeleteDevice(PortData->DeviceObject);
//                         PortData->DeviceObject = NULL;
//                         PortData->ConnectionStatus = NoDeviceConnected;
//                     }
//
//                     USBH_SyncResetPort(HubExtension, Port);
//
//                     ix++;
//                 }
//
//                 if (NT_SUCCESS(NtStatus))
//                 {
//                     PdoExtension = PortData->DeviceObject->DeviceExtension;
//
//                     if (!(PdoExtension->PortPdoFlags & USBHUB_PDO_FLAG_PORT_LOW_SPEED) &&
//                         !(PdoExtension->PortPdoFlags & USBHUB_PDO_FLAG_PORT_HIGH_SPEED) &&
//                         !(HubExtension->HubFlags & USBHUB_FDO_FLAG_USB20_HUB))
//                     {
//                         DPRINT1("USBH_FdoQueryBusRelations: FIXME USBH_DeviceIs2xDualMode()\n");
//
//                         if (0)//USBH_DeviceIs2xDualMode(PdoExtension))
//                         {
//                             PdoExtension->PortPdoFlags |= USBHUB_PDO_FLAG_HS_USB1_DUALMODE;
//                         }
//                     }
//
//                     ObReferenceObject(PortData->DeviceObject);
//
//                     DeviceRelations->Objects[DeviceRelations->Count] = PortData->DeviceObject;
//
//                     PortData->DeviceObject->Flags |= DO_POWER_PAGABLE;
//                     PortData->DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//                     DeviceRelations->Count++;
//
//                     PortData->ConnectionStatus = DeviceConnected;
//
//                     continue;
//                 }
//             }
//         }
//
//         PortData->ConnectionStatus = DeviceFailedEnumeration;
//
//         if (NT_ERROR(USBH_SyncDisablePort(HubExtension, Port)))
//         {
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_FAILED;
//         }
//
//         if (PortData->DeviceObject)
//         {
//             ObReferenceObject(PortData->DeviceObject);
//             PortData->DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//             DeviceRelations->Objects[DeviceRelations->Count++] = PortData->DeviceObject;
//         }
//     }
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_NOT_ENUMERATED;
//
//     KeReleaseSemaphore(&HubExtension->ResetDeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//     {
//         KeSetEvent(&HubExtension->PendingRequestEvent, EVENT_INCREMENT, FALSE);
//     }
//
// RelationsWorker:
//
//     Irp->IoStatus.Status = Status;
//
//     if (!NT_SUCCESS(Status))
//     {
//         //Irp->IoStatus.Information = 0;
//
//         if (DeviceRelations)
//         {
//             ExFreePoolWithTag(DeviceRelations, USB_HUB_TAG);
//         }
//
//         USBH_CompleteIrp(Irp, Status);
//
//         return Status;
//     }
//
//     KeAcquireSpinLock(&HubExtension->RelationsWorkerSpinLock, &OldIrql);
//
//     if (DeviceRelations && DeviceRelations->Count)
//     {
//         for (Port = 0; Port < DeviceRelations->Count; Port++)
//         {
//             PdoDevice = DeviceRelations->Objects[Port];
//             Entry = HubExtension->PdoList.Flink;
//
//             while (Entry != &HubExtension->PdoList)
//             {
//                 pdoExtension = CONTAINING_RECORD(Entry,
//                                                  USBHUB_PORT_PDO_EXTENSION,
//                                                  PortLink);
//
//                 if (pdoExtension == PdoDevice->DeviceExtension)
//                 {
//                     PdoExt(PdoDevice)->EnumFlags |= USBHUB_ENUM_FLAG_GHOST_DEVICE;
//                     goto PortNext;
//                 }
//
//                 Entry = Entry->Flink;
//             }
//
//             PdoExt(PdoDevice)->EnumFlags |= USBHUB_ENUM_FLAG_DEVICE_PRESENT;
//
//     PortNext:;
//         }
//
//         for (Port = 0; Port < DeviceRelations->Count; Port++)
//         {
//             PdoDevice = DeviceRelations->Objects[Port];
//
//             if (PdoExt(PdoDevice)->EnumFlags & USBHUB_ENUM_FLAG_GHOST_DEVICE)
//             {
//                 for (GhostPort = Port;
//                      GhostPort < DeviceRelations->Count;
//                      GhostPort++)
//                 {
//                     DeviceRelations->Objects[GhostPort] =
//                     DeviceRelations->Objects[GhostPort + 1];
//                 }
//
//                 ObDereferenceObject(PdoDevice);
//
//                 DeviceRelations->Count--;
//
//                 if (PdoExt(PdoDevice)->EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT)
//                 {
//                     PdoExt(PdoDevice)->EnumFlags &= ~USBHUB_ENUM_FLAG_GHOST_DEVICE;
//                 }
//             }
//         }
//     }
//
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//
//     InitializeListHead(&GhostPdoList);
//     PdoList = &HubExtension->PdoList;
//
//     while (!IsListEmpty(PdoList))
//     {
//         Entry = RemoveHeadList(PdoList);
//
//         PdoExtension = CONTAINING_RECORD(Entry,
//                                          USBHUB_PORT_PDO_EXTENSION,
//                                          PortLink);
//
//         PdoExtension->EnumFlags &= ~USBHUB_ENUM_FLAG_DEVICE_PRESENT;
//
//         if (PdoExtension->EnumFlags & USBHUB_ENUM_FLAG_GHOST_DEVICE)
//         {
//             InsertTailList(&GhostPdoList, &PdoExtension->PortLink);
//         }
//     }
//
//     KeReleaseSpinLock(&HubExtension->RelationsWorkerSpinLock, OldIrql);
//
//     while (!IsListEmpty(&GhostPdoList))
//     {
//         Entry = RemoveHeadList(&GhostPdoList);
//
//         PdoExtension = CONTAINING_RECORD(Entry,
//                                          USBHUB_PORT_PDO_EXTENSION,
//                                          PortLink);
//
//         IoDeleteDevice(PdoExtension->Common.SelfDevice);
//     }
//
//     return USBH_PassIrp(HubExtension->LowerDevice, Irp);
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoStopDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                    IN PIRP Irp)
// {
//     DPRINT1("USBH_FdoStopDevice: UNIMPLEMENTED. FIXME\n");
//     DbgBreakPoint();
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBH_FdoRemoveDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                      IN PIRP Irp)
// {
//     PUSB_HUB_DESCRIPTOR HubDescriptor;
//     PUSBHUB_PORT_DATA PortData;
//     USHORT NumPorts;
//     USHORT ix;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     NTSTATUS Status;
//
//     DPRINT("USBH_FdoRemoveDevice: HubExtension - %p\n", HubExtension);
//
//     HubDescriptor = HubExtension->HubDescriptor;
//
//     if (HubDescriptor && HubExtension->PortData)
//     {
//         NumPorts = HubDescriptor->bNumberOfPorts;
//
//         for (ix = 0; ix < NumPorts; ++ix)
//         {
//             PortData = HubExtension->PortData + ix;
//
//             PortDevice = PortData->DeviceObject;
//
//             if (PortDevice)
//             {
//                 PortData->PortStatus.AsUlong32 = 0;
//                 PortData->DeviceObject = NULL;
//
//                 PortExtension = PortDevice->DeviceExtension;
//                 PortExtension->EnumFlags &= ~USBHUB_ENUM_FLAG_DEVICE_PRESENT;
//
//                 USBH_PdoRemoveDevice(PortExtension, HubExtension);
//             }
//         }
//     }
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED)
//     {
//         USBH_FdoCleanup(HubExtension);
//     }
//
//     if (HubExtension->PortData)
//     {
//         ExFreePoolWithTag(HubExtension->PortData, USB_HUB_TAG);
//         HubExtension->PortData = NULL;
//     }
//
//     DPRINT1("USBH_FdoRemoveDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");
//
//     Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//
//     IoDetachDevice(HubExtension->LowerDevice);
//     IoDeleteDevice(HubExtension->Common.SelfDevice);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBH_FdoSurpriseRemoveDevice(IN PUSBHUB_FDO_EXTENSION HubExtension,
//                              IN PIRP Irp)
// {
//     PUSBHUB_PORT_PDO_EXTENSION PortExtension;
//     PUSBHUB_PORT_DATA PortData;
//     ULONG NumberPorts;
//     ULONG Port;
//
//     DPRINT("USBH_FdoSurpriseRemoveDevice: HubExtension - %p, Irp - %p\n",
//            HubExtension,
//            Irp);
//
//     if (!HubExtension->PortData ||
//         !HubExtension->HubDescriptor)
//     {
//         return;
//     }
//
//     PortData = HubExtension->PortData;
//     NumberPorts = HubExtension->HubDescriptor->bNumberOfPorts;
//
//     for (Port = 0; Port < NumberPorts; Port++)
//     {
//         if (PortData[Port].DeviceObject)
//         {
//             PortExtension = PdoExt(PortData[Port].DeviceObject);
//             PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_DELETE_PENDING;
//             PortExtension->EnumFlags &= ~USBHUB_ENUM_FLAG_DEVICE_PRESENT;
//
//             PortData[Port].DeviceObject = NULL;
//             PortData[Port].ConnectionStatus = NoDeviceConnected;
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoQueryId(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                 IN PIRP Irp)
// {
//     ULONG IdType;
//     WCHAR Buffer[200];
//     PWCHAR EndBuffer;
//     size_t Remaining = sizeof(Buffer);
//     size_t Length;
//     PWCHAR Id = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PUSB_DEVICE_DESCRIPTOR DeviceDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//
//     IdType = IoGetCurrentIrpStackLocation(Irp)->Parameters.QueryId.IdType;
//     DeviceDescriptor = &PortExtension->DeviceDescriptor;
//     InterfaceDescriptor = &PortExtension->InterfaceDescriptor;
//
//     RtlZeroMemory(Buffer, sizeof(Buffer));
//
//     switch (IdType)
//     {
//         case BusQueryDeviceID:
//             DPRINT("USBH_PdoQueryId: BusQueryDeviceID\n");
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED)
//             {
//                 DPRINT("USBH_PdoQueryId: USBHUB_PDO_FLAG_INIT_PORT_FAILED\n");
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Vid_0000&Pid_0000");
//             }
//             else
//             {
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Vid_%04x&Pid_%04x",
//                                      DeviceDescriptor->idVendor,
//                                      DeviceDescriptor->idProduct);
//             }
//
//             Length = sizeof(Buffer) - (Remaining - sizeof(UNICODE_NULL));
//
//             Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//             if (!Id)
//             {
//                 break;
//             }
//
//             RtlCopyMemory(Id, Buffer, Length);
//             DPRINT("USBH_PdoQueryId: BusQueryDeviceID - %S\n", Id);
//             break;
//
//         case BusQueryHardwareIDs:
//             DPRINT("USBH_PdoQueryId: BusQueryHardwareIDs\n");
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED)
//             {
//                 DPRINT("USBH_PdoQueryId: USBHUB_PDO_FLAG_INIT_PORT_FAILED\n");
//
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\UNKNOWN");
//             }
//             else
//             {
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Vid_%04x&Pid_%04x&Rev_%04x",
//                                      DeviceDescriptor->idVendor,
//                                      DeviceDescriptor->idProduct,
//                                      DeviceDescriptor->bcdDevice);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Vid_%04x&Pid_%04x",
//                                      DeviceDescriptor->idVendor,
//                                      DeviceDescriptor->idProduct);
//             }
//
//             Length = sizeof(Buffer) - (Remaining - 2 * sizeof(UNICODE_NULL));
//
//             Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//             if (!Id)
//             {
//                 break;
//             }
//
//             RtlCopyMemory(Id, Buffer, Length);
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED)
//             {
//                 DPRINT("USBH_PdoQueryId: BusQueryHardwareID - %S\n", Id);
//             }
//             else
//             {
//                 USBHUB_DumpingIDs(Id);
//             }
//
//             break;
//
//         case BusQueryCompatibleIDs:
//             DPRINT("USBH_PdoQueryId: BusQueryCompatibleIDs\n");
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED)
//             {
//                 DPRINT("USBH_PdoQueryId: USBHUB_PDO_FLAG_INIT_PORT_FAILED\n");
//
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\UNKNOWN");
//             }
//             else if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_MULTI_INTERFACE)
//             {
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\DevClass_%02x&SubClass_%02x&Prot_%02x",
//                                      InterfaceDescriptor->bInterfaceClass,
//                                      InterfaceDescriptor->bInterfaceSubClass,
//                                      InterfaceDescriptor->bInterfaceProtocol);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\DevClass_%02x&SubClass_%02x",
//                                      InterfaceDescriptor->bInterfaceClass,
//                                      InterfaceDescriptor->bInterfaceSubClass);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\DevClass_%02x",
//                                      InterfaceDescriptor->bInterfaceClass);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\COMPOSITE");
//             }
//             else
//             {
//                 RtlStringCbPrintfExW(Buffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Class_%02x&SubClass_%02x&Prot_%02x",
//                                      InterfaceDescriptor->bInterfaceClass,
//                                      InterfaceDescriptor->bInterfaceSubClass,
//                                      InterfaceDescriptor->bInterfaceProtocol);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      &EndBuffer,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Class_%02x&SubClass_%02x",
//                                      InterfaceDescriptor->bInterfaceClass,
//                                      InterfaceDescriptor->bInterfaceSubClass);
//
//                 EndBuffer++;
//                 Remaining -= sizeof(UNICODE_NULL);
//
//                 RtlStringCbPrintfExW(EndBuffer,
//                                      Remaining,
//                                      NULL,
//                                      &Remaining,
//                                      0,
//                                      L"USB\\Class_%02x",
//                                      InterfaceDescriptor->bInterfaceClass);
//             }
//
//             Length = sizeof(Buffer) - (Remaining - 2 * sizeof(UNICODE_NULL));
//
//             Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//             if (!Id)
//             {
//                 break;
//             }
//
//             RtlCopyMemory(Id, Buffer, Length);
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED)
//             {
//                 DPRINT("USBH_PdoQueryId: BusQueryCompatibleID - %S\n", Id);
//             }
//             else
//             {
//                 USBHUB_DumpingIDs(Id);
//             }
//
//             break;
//
//         case BusQueryInstanceID:
//             DPRINT("USBH_PdoQueryId: BusQueryInstanceID\n");
//
//             if (PortExtension->SerialNumber)
//             {
//                 Id = ExAllocatePoolWithTag(PagedPool,
//                                            PortExtension->SN_DescriptorLength,
//                                            USB_HUB_TAG);
//
//                 if (Id)
//                 {
//                     RtlZeroMemory(Id, PortExtension->SN_DescriptorLength);
//
//                     RtlCopyMemory(Id,
//                                   PortExtension->SerialNumber,
//                                   PortExtension->SN_DescriptorLength);
//                 }
//             }
//             else
//             {
//                  Length = sizeof(PortExtension->InstanceID) +
//                           sizeof(UNICODE_NULL);
//
//                  Id = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//                  if (Id)
//                  {
//                      RtlZeroMemory(Id, Length);
//
//                      RtlCopyMemory(Id,
//                                    PortExtension->InstanceID,
//                                    sizeof(PortExtension->InstanceID));
//                  }
//             }
//
//             DPRINT("USBH_PdoQueryId: BusQueryInstanceID - %S\n", Id);
//             break;
//
//         default:
//             DPRINT1("USBH_PdoQueryId: unknown query id type 0x%lx\n", IdType);
//             return Irp->IoStatus.Status;
//     }
//
//     Irp->IoStatus.Information = (ULONG_PTR)Id;
//
//     if (!Id)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoQueryDeviceText(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                         IN PIRP Irp)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PIO_STACK_LOCATION IoStack;
//     DEVICE_TEXT_TYPE DeviceTextType;
//     USHORT LanguageId;
//     USHORT DefaultId;
//     PUSB_STRING_DESCRIPTOR Descriptor;
//     PWCHAR DeviceText;
//     UCHAR iProduct = 0;
//     NTSTATUS Status;
//     size_t NumSymbols;
//     size_t Length;
//
//     DPRINT("USBH_PdoQueryDeviceText ... \n");
//
//     DeviceObject = PortExtension->Common.SelfDevice;
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     DeviceTextType = IoStack->Parameters.QueryDeviceText.DeviceTextType;
//
//     if (DeviceTextType != DeviceTextDescription &&
//         DeviceTextType != DeviceTextLocationInformation)
//     {
//         return Irp->IoStatus.Status;
//     }
//
//     LanguageId = LANGIDFROMLCID(IoStack->Parameters.QueryDeviceText.LocaleId);
//     DefaultId = MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US);
//
//     if (!LanguageId)
//     {
//         LanguageId = DefaultId;
//     }
//
//     iProduct = PortExtension->DeviceDescriptor.iProduct;
//
//     if (PortExtension->DeviceHandle && iProduct &&
//         !PortExtension->IgnoringHwSerial &&
//         !(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_INIT_PORT_FAILED))
//     {
//         Descriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                            MAXIMUM_USB_STRING_LENGTH,
//                                            USB_HUB_TAG);
//
//         if (Descriptor)
//         {
//             RtlZeroMemory(Descriptor, MAXIMUM_USB_STRING_LENGTH);
//
//             for (Status = USBH_CheckDeviceLanguage(DeviceObject, LanguageId);
//                  ;
//                  Status = USBH_CheckDeviceLanguage(DeviceObject, DefaultId))
//             {
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = USBH_SyncGetStringDescriptor(DeviceObject,
//                                                           iProduct,
//                                                           LanguageId,
//                                                           Descriptor,
//                                                           MAXIMUM_USB_STRING_LENGTH,
//                                                           NULL,
//                                                           TRUE);
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         break;
//                     }
//                 }
//
//                 if (LanguageId == DefaultId)
//                 {
//                     goto Exit;
//                 }
//
//                 LanguageId = DefaultId;
//             }
//
//             if (Descriptor->bLength <= sizeof(USB_COMMON_DESCRIPTOR))
//             {
//                 Status = STATUS_UNSUCCESSFUL;
//             }
//
//             if (NT_SUCCESS(Status))
//             {
//                 Length = Descriptor->bLength -
//                          FIELD_OFFSET(USB_STRING_DESCRIPTOR, bString);
//
//                 DeviceText = ExAllocatePoolWithTag(PagedPool,
//                                                    Length + sizeof(UNICODE_NULL),
//                                                    USB_HUB_TAG);
//
//                 if (DeviceText)
//                 {
//                     RtlZeroMemory(DeviceText, Length + sizeof(UNICODE_NULL));
//
//                     RtlCopyMemory(DeviceText, Descriptor->bString, Length);
//
//                     Irp->IoStatus.Information = (ULONG_PTR)DeviceText;
//
//                     DPRINT("USBH_PdoQueryDeviceText: Descriptor->bString - %S\n",
//                            DeviceText);
//                 }
//                 else
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//             }
//
//         Exit:
//
//             ExFreePoolWithTag(Descriptor, USB_HUB_TAG);
//
//             if (NT_SUCCESS(Status))
//             {
//                 return Status;
//             }
//         }
//         else
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//     else
//     {
//         Status = STATUS_NOT_SUPPORTED;
//     }
//
//     if (!GenericUSBDeviceString)
//     {
//         return Status;
//     }
//
//     NumSymbols = wcslen(GenericUSBDeviceString);
//     Length = (NumSymbols + 1) * sizeof(WCHAR);
//
//     DeviceText = ExAllocatePoolWithTag(PagedPool, Length, USB_HUB_TAG);
//
//     if (!DeviceText)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(DeviceText, Length);
//
//     RtlCopyMemory(DeviceText,
//                   GenericUSBDeviceString,
//                   NumSymbols * sizeof(WCHAR));
//
//     Irp->IoStatus.Information = (ULONG_PTR)DeviceText;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBH_SymbolicLink(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                   IN const GUID * InterfaceClassGuid,
//                   IN BOOLEAN IsEnable)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PVOID NameBuffer;
//
//     DPRINT("USBH_SymbolicLink ... \n");
//
//     if (IsEnable)
//     {
//         Status = IoRegisterDeviceInterface(PortExtension->Common.SelfDevice,
//                                            InterfaceClassGuid,
//                                            NULL,
//                                            &PortExtension->SymbolicLinkName);
//
//         if (NT_SUCCESS(Status))
//         {
//             USBH_SetPdoRegistryParameter(PortExtension->Common.SelfDevice,
//                                          L"SymbolicName",
//                                          PortExtension->SymbolicLinkName.Buffer,
//                                          PortExtension->SymbolicLinkName.Length,
//                                          REG_SZ,
//                                          PLUGPLAY_REGKEY_DEVICE);
//
//             Status = IoSetDeviceInterfaceState(&PortExtension->SymbolicLinkName,
//                                                TRUE);
//         }
//     }
//     else
//     {
//         NameBuffer = PortExtension->SymbolicLinkName.Buffer;
//
//         if (NameBuffer)
//         {
//             Status = IoSetDeviceInterfaceState(&PortExtension->SymbolicLinkName,
//                                                FALSE);
//
//             ExFreePool(PortExtension->SymbolicLinkName.Buffer);
//
//             PortExtension->SymbolicLinkName.Buffer = NULL;
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_RestoreDevice(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                    IN BOOLEAN IsKeepDeviceData)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PUSBHUB_PORT_DATA PortData;
//     NTSTATUS Status;
//     ULONG ix;
//
//     DPRINT("USBH_RestoreDevice ... \n");
//
//     HubExtension = PortExtension->HubExtension;
//
//     if (!HubExtension)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         return Status;
//     }
//
//     ASSERT(PortExtension->PortNumber > 0);
//     PortData = &HubExtension->PortData[PortExtension->PortNumber - 1];
//
//     if (PortExtension->Common.SelfDevice != PortData->DeviceObject)
//     {
//         Status = STATUS_UNSUCCESSFUL;
//         return Status;
//     }
//
//     Status = USBH_SyncGetPortStatus(HubExtension,
//                                     PortExtension->PortNumber,
//                                     &PortData->PortStatus,
//                                     sizeof(USB_PORT_STATUS_AND_CHANGE));
//
//     if (NT_SUCCESS(Status))
//     {
//         for (ix = 0; ix < 3; ix++)
//         {
//             Status = USBH_ResetDevice((PUSBHUB_FDO_EXTENSION)HubExtension,
//                                       PortExtension->PortNumber,
//                                       IsKeepDeviceData,
//                                       ix == 0);
//
//             if (NT_SUCCESS(Status) || Status == STATUS_NO_SUCH_DEVICE)
//             {
//                 break;
//             }
//
//             USBH_Wait(1000);
//         }
//     }
//
//     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_POWER_D3;
//
//     if (NT_SUCCESS(Status))
//     {
//         PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_PORT_RESTORE_FAIL;
//     }
//     else
//     {
//         PortExtension->PortPdoFlags |= (USBHUB_PDO_FLAG_INIT_PORT_FAILED |
//                                         USBHUB_PDO_FLAG_PORT_RESTORE_FAIL);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoStartDevice(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                     IN PIRP Irp)
// {
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     const GUID * Guid;
//     NTSTATUS Status;
//
//     DPRINT("USBH_PdoStartDevice: PortExtension - %p\n", PortExtension);
//
//     if (!PortExtension->HubExtension &&
//         PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3)
//     {
//         PortExtension->HubExtension = PortExtension->RootHubExtension;
//     }
//
//     HubExtension = PortExtension->HubExtension;
//
//     if (HubExtension)
//     {
//         USBHUB_SetDeviceHandleData(HubExtension,
//                                    PortExtension->Common.SelfDevice,
//                                    PortExtension->DeviceHandle);
//     }
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE)
//     {
//         Guid = &GUID_DEVINTERFACE_USB_HUB;
//     }
//     else
//     {
//         Guid = &GUID_DEVINTERFACE_USB_DEVICE;
//     }
//
//     Status = USBH_SymbolicLink(PortExtension, Guid, TRUE);
//
//     if (NT_SUCCESS(Status))
//     {
//         PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
//     }
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3)
//     {
//         Status = USBH_RestoreDevice(PortExtension, 0);
//     }
//
//     PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_DEVICE_STARTED;
//
//     PortExtension->CurrentPowerState.DeviceState = PowerDeviceD0;
//
//     DPRINT1("USBH_PdoStartDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoRemoveDevice(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                      IN PUSBHUB_FDO_EXTENSION HubExtension)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PDEVICE_OBJECT PortDevice;
//     PUSBHUB_PORT_PDO_EXTENSION PortExt;
//     PUSBHUB_PORT_DATA PortData;
//     PIRP IdleNotificationIrp;
//     PIRP WakeIrp;
//     PVOID DeviceHandle;
//     PDEVICE_OBJECT Pdo;
//     PVOID SerialNumber;
//     USHORT Port;
//     KIRQL Irql;
//
//     DPRINT("USBH_PdoRemoveDevice ... \n");
//
//     PortDevice = PortExtension->Common.SelfDevice;
//     PortExtension->HubExtension = NULL;
//
//     Port = PortExtension->PortNumber;
//     ASSERT(Port > 0);
//
//     if (HubExtension &&
//         HubExtension->CurrentPowerState.DeviceState != PowerDeviceD0 &&
//         (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED) != 0)
//     {
//         USBH_HubSetD0(HubExtension);
//     }
//
//     IoAcquireCancelSpinLock(&Irql);
//     IdleNotificationIrp = PortExtension->IdleNotificationIrp;
//
//     if (IdleNotificationIrp)
//     {
//         PortExtension->IdleNotificationIrp = NULL;
//         PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_IDLE_NOTIFICATION;
//
//         if (IdleNotificationIrp->Cancel)
//         {
//             IdleNotificationIrp = NULL;
//         }
//
//         if (IdleNotificationIrp)
//         {
//             IoSetCancelRoutine(IdleNotificationIrp, NULL);
//         }
//     }
//
//     WakeIrp = PortExtension->PdoWaitWakeIrp;
//
//     if (WakeIrp)
//     {
//         PortExtension->PdoWaitWakeIrp = NULL;
//         PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_WAIT_WAKE;
//
//         if (WakeIrp->Cancel || !IoSetCancelRoutine(WakeIrp, NULL))
//         {
//             WakeIrp = NULL;
//
//             ASSERT(HubExtension);
//             if (!InterlockedDecrement(&HubExtension->PendingRequestCount))
//             {
//                 KeSetEvent(&HubExtension->PendingRequestEvent,
//                            EVENT_INCREMENT,
//                            FALSE);
//             }
//         }
//     }
//
//     IoReleaseCancelSpinLock(Irql);
//
//     if (IdleNotificationIrp)
//     {
//         IdleNotificationIrp->IoStatus.Status = STATUS_CANCELLED;
//         IoCompleteRequest(IdleNotificationIrp, IO_NO_INCREMENT);
//     }
//
//     if (WakeIrp)
//     {
//         ASSERT(HubExtension);
//         USBH_CompletePowerIrp(HubExtension, WakeIrp, STATUS_CANCELLED);
//     }
//
//     PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_POWER_D3;
//
//     if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_REG_DEV_INTERFACE)
//     {
//         Status = USBH_SymbolicLink(PortExtension, NULL, FALSE);
//
//         if (NT_SUCCESS(Status))
//         {
//             PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
//         }
//     }
//
//     DeviceHandle = InterlockedExchangePointer(&PortExtension->DeviceHandle,
//                                               NULL);
//
//     if (DeviceHandle)
//     {
//         ASSERT(HubExtension);
//         Status = USBD_RemoveDeviceEx(HubExtension, DeviceHandle, 0);
//
//         if (HubExtension->PortData &&
//             HubExtension->PortData[Port - 1].DeviceObject == PortDevice)
//         {
//             USBH_SyncDisablePort(HubExtension, Port);
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_DEVICE_STARTED;
//
//         if (HubExtension && HubExtension->PortData)
//         {
//             PortData = &HubExtension->PortData[Port - 1];
//
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_DELETE_PENDING)
//             {
//                 Pdo = PortData->DeviceObject;
//
//                 if (Pdo)
//                 {
//                     PortData->DeviceObject = NULL;
//                     PortData->ConnectionStatus = NoDeviceConnected;
//
//                     if (PdoExt(Pdo)->EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT)
//                     {
//                         PortExt = PdoExt(Pdo);
//
//                         InsertTailList(&HubExtension->PdoList,
//                                        &PortExt->PortLink);
//                     }
//                 }
//             }
//         }
//
//         if (!(PortExtension->EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT) &&
//             !(PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_NOT_CONNECTED))
//         {
//             PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_NOT_CONNECTED;
//
//             SerialNumber = InterlockedExchangePointer((PVOID)&PortExtension->SerialNumber,
//                                                       NULL);
//
//             if (SerialNumber)
//             {
//                 ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
//             }
//
//             DPRINT1("USBH_PdoRemoveDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");
//
//             if (HubExtension)
//                 USBHUB_FlushAllTransfers(HubExtension);
//
//             IoDeleteDevice(PortDevice);
//         }
//     }
//
//     if (HubExtension)
//     {
//         DPRINT("USBH_PdoRemoveDevice: call USBH_CheckIdleDeferred()\n");
//         USBH_CheckIdleDeferred(HubExtension);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoStopDevice(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//                    IN PIRP Irp)
// {
//     DPRINT1("USBH_PdoStopDevice: UNIMPLEMENTED. FIXME\n");
//     DbgBreakPoint();
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// RECONSTRUCTED (pnprecon): USBH_FdoPnP / USBH_PdoPnP — faithful from pnp.c
// ============================================================================

// ---------------------------------------------------------------------------
// Reconstructed PnP dispatch helpers (rustconv dropped these; faithful ports of
// drivers/usb/usbhub/pnp.c USBH_FdoStartDevice / USBH_FdoRemoveDevice /
// USBH_PdoStartDevice / USBH_PdoRemoveDevice and usbhub.c USBH_SetPdoRegistryParameter).
// All are NTAPI in the C headers → `extern "system"` (stdcall on i386).
// ---------------------------------------------------------------------------

// drivers/usb/usbhub/pnp.c:986
#[no_mangle]
pub unsafe extern "system" fn USBH_FdoStartDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBHUB] FdoStartDevice hub=%p\n\0".as_ptr() as *const i8,
        HubExtension,
    );

    (*HubExtension).RootHubPdo = core::ptr::null_mut();

    Status = USBH_SyncGetRootHubPdo(
        (*HubExtension).LowerDevice,
        core::ptr::addr_of_mut!((*HubExtension).RootHubPdo),
        core::ptr::addr_of_mut!((*HubExtension).RootHubPdo2),
    );

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBHUB] SyncGetRootHubPdo -> 0x%x pdo=%p\n\0".as_ptr() as *const i8,
        Status,
        (*HubExtension).RootHubPdo,
    );

    if NT_SUCCESS(Status) {
        if !(*HubExtension).RootHubPdo.is_null() {
            Status = USBH_StartHubFdoDevice(HubExtension, Irp);
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[USBHUB] StartHubFdoDevice -> 0x%x\n\0".as_ptr() as *const i8,
                Status,
            );
        } else {
            // DPRINT1("USBH_FdoStartDevice: FIXME. start ParentDevice\n");
            DbgBreakPoint();
        }
    } else {
        // DPRINT1("USBH_FdoStartDevice: FIXME. USBH_SyncGetRootHubPdo return - %lX\n", Status);
        DbgBreakPoint();
        USBH_CompleteIrp(Irp, Status);
    }

    return Status;
}

// drivers/usb/usbhub/pnp.c:1461
#[no_mangle]
pub unsafe extern "system" fn USBH_FdoRemoveDevice(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let HubDescriptor: PUSB_HUB_DESCRIPTOR;
    let mut PortData: PUSBHUB_PORT_DATA;
    let NumPorts: USHORT;
    let mut ix: USHORT;
    let Status: NTSTATUS;

    // DPRINT("USBH_FdoRemoveDevice: HubExtension - %p\n", HubExtension);

    HubDescriptor = (*HubExtension).HubDescriptor;

    if !HubDescriptor.is_null() && !(*HubExtension).PortData.is_null() {
        NumPorts = (*HubDescriptor).bNumberOfPorts as USHORT;

        ix = 0;
        while (ix as i32) < NumPorts as i32 {
            PortData = (*HubExtension).PortData.offset(ix as isize);

            let PortDevice = (*PortData).DeviceObject;

            if !PortDevice.is_null() {
                *(*PortData).PortStatus.AsUlong32.as_mut() = 0;
                (*PortData).DeviceObject = core::ptr::null_mut();

                let PortExtension = (*PortDevice).DeviceExtension as PUSBHUB_PORT_PDO_EXTENSION;
                (*PortExtension).EnumFlags &= !USBHUB_ENUM_FLAG_DEVICE_PRESENT;

                USBH_PdoRemoveDevice(PortExtension, HubExtension);
            }

            ix = ix.wrapping_add(1);
        }
    }

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED != 0 {
        USBH_FdoCleanup(HubExtension);
    }

    if !(*HubExtension).PortData.is_null() {
        ExFreePoolWithTag((*HubExtension).PortData as PVOID, USB_HUB_TAG);
        (*HubExtension).PortData = core::ptr::null_mut();
    }

    // DPRINT1("USBH_FdoRemoveDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");

    Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);

    IoDetachDevice((*HubExtension).LowerDevice);
    IoDeleteDevice((*HubExtension).Common.SelfDevice);

    let _ = PortData;
    return Status;
}

// drivers/usb/usbhub/pnp.c:2122
#[no_mangle]
pub unsafe extern "system" fn USBH_PdoStartDevice(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let HubExtension: PUSBHUB_FDO_EXTENSION;
    let Guid: *const GUID;
    let mut Status: NTSTATUS;

    let _ = Irp;

    // DPRINT("USBH_PdoStartDevice: PortExtension - %p\n", PortExtension);

    if (*PortExtension).HubExtension.is_null()
        && (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3 != 0
    {
        (*PortExtension).HubExtension = (*PortExtension).RootHubExtension;
    }

    HubExtension = (*PortExtension).HubExtension;

    if !HubExtension.is_null() {
        USBHUB_SetDeviceHandleData(
            HubExtension,
            (*PortExtension).Common.SelfDevice,
            (*PortExtension).DeviceHandle as PVOID,
        );
    }

    if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_HUB_DEVICE != 0 {
        Guid = core::ptr::addr_of!(GUID_DEVINTERFACE_USB_HUB);
    } else {
        Guid = core::ptr::addr_of!(GUID_DEVINTERFACE_USB_DEVICE);
    }

    Status = USBH_SymbolicLink(PortExtension, Guid, TRUE as BOOLEAN);

    if NT_SUCCESS(Status) {
        (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
    }

    if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_POWER_D3 != 0 {
        Status = USBH_RestoreDevice(PortExtension, 0 as BOOLEAN);
    }

    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_DEVICE_STARTED;

    (*PortExtension).CurrentPowerState.DeviceState = PowerDeviceD0;

    // DPRINT1("USBH_PdoStartDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");

    return Status;
}

// drivers/usb/usbhub/pnp.c:2178
#[no_mangle]
pub unsafe extern "system" fn USBH_PdoRemoveDevice(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    HubExtension: PUSBHUB_FDO_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let PortDevice: PDEVICE_OBJECT;
    let mut IdleNotificationIrp: PIRP;
    let mut WakeIrp: PIRP;
    let DeviceHandle: PVOID;
    let Port: USHORT;
    let mut Irql: KIRQL = 0;

    // DPRINT("USBH_PdoRemoveDevice ... \n");

    PortDevice = (*PortExtension).Common.SelfDevice;
    (*PortExtension).HubExtension = core::ptr::null_mut();

    Port = (*PortExtension).PortNumber;
    // ASSERT(Port > 0);

    if !HubExtension.is_null()
        && (*HubExtension).CurrentPowerState.DeviceState != PowerDeviceD0
        && ((*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_STARTED) != 0
    {
        USBH_HubSetD0(HubExtension);
    }

    IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(Irql));
    IdleNotificationIrp = (*PortExtension).IdleNotificationIrp;

    if !IdleNotificationIrp.is_null() {
        (*PortExtension).IdleNotificationIrp = core::ptr::null_mut();
        (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_IDLE_NOTIFICATION;

        if (*IdleNotificationIrp).Cancel != 0 {
            IdleNotificationIrp = core::ptr::null_mut();
        }

        if !IdleNotificationIrp.is_null() {
            IoSetCancelRoutine(IdleNotificationIrp, None);
        }
    }

    WakeIrp = (*PortExtension).PdoWaitWakeIrp;

    if !WakeIrp.is_null() {
        (*PortExtension).PdoWaitWakeIrp = core::ptr::null_mut();
        (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_WAIT_WAKE;

        if (*WakeIrp).Cancel != 0 || IoSetCancelRoutine(WakeIrp, None).is_none() {
            WakeIrp = core::ptr::null_mut();

            // ASSERT(HubExtension);
            if InterlockedDecrement(
                core::ptr::addr_of_mut!((*HubExtension).PendingRequestCount),
            ) == 0
            {
                KeSetEvent(
                    core::ptr::addr_of_mut!((*HubExtension).PendingRequestEvent) as *mut c_void,
                    EVENT_INCREMENT as LONG,
                    0,
                );
            }
        }
    }

    IoReleaseCancelSpinLock(Irql);

    if !IdleNotificationIrp.is_null() {
        (*IdleNotificationIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        IoCompleteRequest(IdleNotificationIrp, IO_NO_INCREMENT as CCHAR);
    }

    if !WakeIrp.is_null() {
        // ASSERT(HubExtension);
        USBH_CompletePowerIrp(HubExtension, WakeIrp, STATUS_CANCELLED);
    }

    (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_POWER_D3;

    if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_REG_DEV_INTERFACE != 0 {
        Status = USBH_SymbolicLink(PortExtension, core::ptr::null(), FALSE as BOOLEAN);

        if NT_SUCCESS(Status) {
            (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
        }
    }

    DeviceHandle = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*PortExtension).DeviceHandle) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if !DeviceHandle.is_null() {
        // ASSERT(HubExtension);
        Status = crate::types::USBD_RemoveDeviceEx(HubExtension, DeviceHandle as PUSB_DEVICE_HANDLE, 0);

        if !(*HubExtension).PortData.is_null()
            && (*(*HubExtension).PortData.offset((Port as i32 - 1) as isize)).DeviceObject
                == PortDevice
        {
            USBH_SyncDisablePort(HubExtension, Port);
        }
    }

    if NT_SUCCESS(Status) {
        (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_DEVICE_STARTED;

        if !HubExtension.is_null() && !(*HubExtension).PortData.is_null() {
            let PortData = (*HubExtension).PortData.offset((Port as i32 - 1) as isize);

            if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_DELETE_PENDING != 0 {
                let Pdo = (*PortData).DeviceObject;

                if !Pdo.is_null() {
                    (*PortData).DeviceObject = core::ptr::null_mut();
                    (*PortData).ConnectionStatus = NoDeviceConnected;

                    if (*PdoExt(Pdo)).EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT != 0 {
                        let PortExt = PdoExt(Pdo);

                        InsertTailList(
                            core::ptr::addr_of_mut!((*HubExtension).PdoList),
                            core::ptr::addr_of_mut!((*PortExt).PortLink),
                        );
                    }
                }
            }
        }

        if (*PortExtension).EnumFlags & USBHUB_ENUM_FLAG_DEVICE_PRESENT == 0
            && (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_NOT_CONNECTED == 0
        {
            (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_NOT_CONNECTED;

            let SerialNumber = InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*PortExtension).SerialNumber) as *mut PVOID,
                core::ptr::null_mut(),
            );

            if !SerialNumber.is_null() {
                ExFreePoolWithTag(SerialNumber, USB_HUB_TAG);
            }

            // DPRINT1("USBH_PdoRemoveDevice: call IoWMIRegistrationControl UNIMPLEMENTED. FIXME\n");

            if !HubExtension.is_null() {
                USBHUB_FlushAllTransfers(HubExtension);
            }

            IoDeleteDevice(PortDevice);
        }
    }

    if !HubExtension.is_null() {
        // DPRINT("USBH_PdoRemoveDevice: call USBH_CheckIdleDeferred()\n");
        USBH_CheckIdleDeferred(HubExtension);
    }

    return Status;
}

// drivers/usb/usbhub/usbhub.c:211 (USBH_SetPdoRegistryParameter — referenced LIVE by
// USBH_SymbolicLink in this module; its usbhub.rs home was dropped by rustconv).
#[no_mangle]
pub unsafe extern "system" fn USBH_SetPdoRegistryParameter(
    DeviceObject: PDEVICE_OBJECT,
    ValueName: PCWSTR,
    Data: PVOID,
    DataSize: ULONG,
    Type: ULONG,
    DevInstKeyType: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut ValueNameString: UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    // DPRINT("USBH_SetPdoRegistryParameter ... \n");

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueNameString), ValueName);

    Status = IoOpenDeviceRegistryKey(
        DeviceObject,
        DevInstKeyType,
        STANDARD_RIGHTS_ALL,
        core::ptr::addr_of_mut!(KeyHandle),
    );

    if NT_SUCCESS(Status) {
        ZwSetValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(ValueNameString),
            0,
            Type,
            Data,
            DataSize,
        );

        ZwClose(KeyHandle);
    }

    return Status;
}

// _INTERFACE_TYPE_PNPBus (bindgen prefixed enum member; iotypes.h: PNPBus == 15)
use ntkrnl_sys::ddk::_INTERFACE_TYPE_PNPBus as PNPBus;



// iotypes.h:1003  #define PNP_DEVICE_FAILED  0x00000004
const PNP_DEVICE_FAILED: u32 = 0x00000004;

// wdmguid.h:102  DEFINE_GUID(GUID_BUS_TYPE_USB, 0x9d7debbc, 0xc85d, 0x11d1,
//                            0x9e, 0xb4, 0x00, 0x60, 0x08, 0xc3, 0xa1, 0x9a);
static GUID_BUS_TYPE_USB: GUID = GUID {
    Data1: 0x9d7debbc,
    Data2: 0xc85d,
    Data3: 0x11d1,
    Data4: [0x9e, 0xb4, 0x00, 0x60, 0x08, 0xc3, 0xa1, 0x9a],
};

// wdm.template.h:235  IsEqualGUIDAligned(g1, g2) — compares the two 8-byte halves.
#[inline]
unsafe fn IsEqualGUIDAligned(guid1: *const GUID, guid2: *const GUID) -> bool {
    let g1 = guid1 as *const i64;
    let g2 = guid2 as *const i64;
    *g1 == *g2 && *g1.add(1) == *g2.add(1)
}

#[no_mangle]
pub unsafe extern "system" fn USBH_FdoPnP(
    HubExtension: PUSBHUB_FDO_EXTENSION,
    Irp: PIRP,
    Minor: UCHAR,
) -> NTSTATUS {
    // usbhub.h flag bits (verified against drivers/usb/usbhub/usbhub.h)
    const USBHUB_FDO_FLAG_DEVICE_STARTED: ULONG = 1 << 0; // usbhub.h:33
    const USBHUB_FDO_FLAG_DEVICE_FAILED: ULONG = 1 << 3; // usbhub.h:35
    const USBHUB_FDO_FLAG_DEVICE_STOPPED: ULONG = 1 << 5; // usbhub.h:37
    const USBHUB_FDO_FLAG_HUB_BUSY: ULONG = 1 << 6; // usbhub.h:38
    const USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST: ULONG = 1 << 12; // usbhub.h:44
    const USBHUB_FDO_FLAG_STATE_CHANGING: ULONG = 1 << 13; // usbhub.h:45
    const USBHUB_FDO_FLAG_DEVICE_REMOVED: ULONG = 1 << 14; // usbhub.h:46
    const USBHUB_FDO_FLAG_GOING_IDLE: ULONG = 1 << 25; // usbhub.h:57
    const USBHUB_FDO_FLAG_DEVICE_SUSPENDED: ULONG = 1 << 26; // usbhub.h:58

    let Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;
    let RelationsType: DEVICE_RELATION_TYPE;
    let mut IsCheckIdle: BOOLEAN;

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST != 0
        && (Minor as u32 == IRP_MN_REMOVE_DEVICE || Minor as u32 == IRP_MN_STOP_DEVICE)
    {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_SUSPENDED;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE, // KernelMode == 0
        0 as BOOLEAN,
        core::ptr::null_mut(),
    );

    if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_GOING_IDLE != 0 {
        (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_SUSPENDED;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    RelationsType = (*IoStack).Parameters.QueryDeviceRelations.Type;

    if (*HubExtension).CurrentPowerState.DeviceState == PowerDeviceD0
        || (*HubExtension).HubFlags
            & (USBHUB_FDO_FLAG_DEVICE_STOPPED | USBHUB_FDO_FLAG_DEVICE_STARTED)
            == 0
        || (Minor as u32 == IRP_MN_QUERY_DEVICE_RELATIONS
            && RelationsType == TargetDeviceRelation)
    {
        IsCheckIdle = 0 as BOOLEAN;
    } else {
        IsCheckIdle = 1 as BOOLEAN;
        USBH_HubSetD0(HubExtension);
    }

    match Minor as u32 {
        IRP_MN_START_DEVICE => {
            IsCheckIdle = 0 as BOOLEAN;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_FdoStartDevice(HubExtension, Irp);
        }

        IRP_MN_QUERY_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_REMOVE_DEVICE => {
            IsCheckIdle = 0 as BOOLEAN;
            (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_DEVICE_REMOVED;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_FdoRemoveDevice(HubExtension, Irp);
        }

        IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_STOP_DEVICE => {
            IsCheckIdle = 0 as BOOLEAN;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_FdoStopDevice(HubExtension, Irp);
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if RelationsType != BusRelations {
                Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
            } else {
                (*HubExtension).HubFlags |= USBHUB_FDO_FLAG_HUB_BUSY;

                IsCheckIdle = 1 as BOOLEAN;

                Status = USBH_FdoQueryBusRelations(HubExtension, Irp);

                (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_HUB_BUSY;
            }
        }

        IRP_MN_QUERY_INTERFACE => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_CAPABILITIES => {
            IoCopyCurrentIrpStackLocationToNext(Irp);

            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(
                    USBH_QueryCapsComplete as *const (),
                ),
                HubExtension as *mut core::ffi::c_void,
                1,
                0,
                0,
            );

            Status = IoCallDriver((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_RESOURCES => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_DEVICE_TEXT => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_READ_CONFIG => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_WRITE_CONFIG => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_EJECT => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_SET_LOCK => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_ID => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            if (*HubExtension).HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED != 0 {
                (*Irp).IoStatus.Information = (*Irp).IoStatus.Information | PNP_DEVICE_FAILED as ntkrnl_sys::ddk::ULONG_PTR;
            }

            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_QUERY_BUS_INFORMATION => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        IRP_MN_SURPRISE_REMOVAL => {
            USBH_FdoSurpriseRemoveDevice(HubExtension, Irp);
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }

        _ => {
            Status = USBH_PassIrp((*HubExtension).LowerDevice, Irp);
        }
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*HubExtension).IdleSemaphore),
        LOW_REALTIME_PRIORITY as KPRIORITY,
        1,
        0 as BOOLEAN,
    );

    if IsCheckIdle != 0 {
        USBH_CheckIdleDeferred(HubExtension);
    }

    (*HubExtension).HubFlags &= !USBHUB_FDO_FLAG_STATE_CHANGING;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBH_PdoPnP(
    PortExtension: PUSBHUB_PORT_PDO_EXTENSION,
    Irp: PIRP,
    Minor: UCHAR,
    IsCompleteIrp: *mut BOOLEAN,
) -> NTSTATUS {
    // usbhub.h PDO flag bits (verified against drivers/usb/usbhub/usbhub.h)
    const USBHUB_PDO_FLAG_INIT_PORT_FAILED: ULONG = 1 << 2; // usbhub.h:63
    const USBHUB_PDO_FLAG_REG_DEV_INTERFACE: ULONG = 1 << 11; // usbhub.h:72
    const USBHUB_PDO_FLAG_PORT_RESTORE_FAIL: ULONG = 1 << 12; // usbhub.h:73
    const USBHUB_PDO_FLAG_OVERCURRENT_PORT: ULONG = 1 << 14; // usbhub.h:75
    const USBHUB_PDO_FLAG_INSUFFICIENT_PWR: ULONG = 1 << 16; // usbhub.h:77
    const USBHUB_PDO_FLAG_ENUMERATED: ULONG = 1 << 26; // usbhub.h:82

    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;
    let BusInfo: PPNP_BUS_INFORMATION;
    let DeviceCapabilities: PDEVICE_CAPABILITIES;
    let Size: USHORT;
    let Version: USHORT;
    let mut HubExtension: PUSBHUB_FDO_EXTENSION;
    let DeviceRelation: PDEVICE_RELATIONS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    *IsCompleteIrp = 1 as BOOLEAN;

    match Minor as u32 {
        IRP_MN_START_DEVICE => {
            return USBH_PdoStartDevice(PortExtension, Irp);
        }

        IRP_MN_QUERY_REMOVE_DEVICE => {
            return STATUS_SUCCESS;
        }

        IRP_MN_REMOVE_DEVICE => {
            return USBH_PdoRemoveDevice(PortExtension, (*PortExtension).HubExtension);
        }

        IRP_MN_CANCEL_REMOVE_DEVICE => {
            return STATUS_SUCCESS;
        }

        IRP_MN_STOP_DEVICE => {
            return USBH_PdoStopDevice(PortExtension, Irp);
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            return STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            return STATUS_SUCCESS;
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type != TargetDeviceRelation {
                return (*Irp).IoStatus.__bindgen_anon_1.Status;
            }

            DeviceRelation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<DEVICE_RELATIONS>() as SIZE_T,
                USB_HUB_TAG as ULONG,
            ) as PDEVICE_RELATIONS;

            if !DeviceRelation.is_null() {
                RtlZeroMemory(
                    DeviceRelation as *mut core::ffi::c_void,
                    core::mem::size_of::<DEVICE_RELATIONS>(),
                );

                (*DeviceRelation).Count = 1;
                (*DeviceRelation).Objects[0] = (*PortExtension).Common.SelfDevice;

                ObReferenceObject((*DeviceRelation).Objects[0] as PVOID);

                Status = STATUS_SUCCESS;
            } else {
                Status = STATUS_INSUFFICIENT_RESOURCES;
            }

            (*Irp).IoStatus.Information = DeviceRelation as _;
        }

        IRP_MN_QUERY_INTERFACE => {
            *IsCompleteIrp = 0 as BOOLEAN;

            if IsEqualGUIDAligned(
                (*IoStack).Parameters.QueryInterface.InterfaceType,
                core::ptr::addr_of!(USB_BUS_INTERFACE_USBDI_GUID),
            ) {
                (*IoStack).Parameters.QueryInterface.InterfaceSpecificData =
                    (*PortExtension).DeviceHandle;
            }

            HubExtension = (*PortExtension).HubExtension;

            if HubExtension.is_null() {
                HubExtension = (*PortExtension).RootHubExtension;
            }

            Status = USBH_PassIrp((*HubExtension).RootHubPdo, Irp);
        }

        IRP_MN_QUERY_CAPABILITIES => {
            DeviceCapabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;

            Size = (*DeviceCapabilities).Size;
            Version = (*DeviceCapabilities).Version;

            RtlCopyMemory(
                DeviceCapabilities as *mut core::ffi::c_void,
                core::ptr::addr_of!((*PortExtension).Capabilities) as *const core::ffi::c_void,
                core::mem::size_of::<DEVICE_CAPABILITIES>(),
            );

            (*DeviceCapabilities).Size = Size;
            (*DeviceCapabilities).Version = Version;

            /* All devices connected to a hub are removable */
            (*DeviceCapabilities).set_Removable(1);

            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_RESOURCES => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            (*PortExtension).PortPdoFlags |= USBHUB_PDO_FLAG_ENUMERATED;

            /* FIXME HKEY_LOCAL_MACHINE\SYSTEM\ControlSetXXX\Enum\USB\
               Vid_????&Pid_????\????????????\Device Parameters\
               if (ExtPropDescSemaphore)
            */

            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_DEVICE_TEXT => {
            return USBH_PdoQueryDeviceText(PortExtension, Irp);
        }

        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_READ_CONFIG => {
            DbgBreakPoint();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_WRITE_CONFIG => {
            DbgBreakPoint();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_EJECT => {
            DbgBreakPoint();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_SET_LOCK => {
            DbgBreakPoint();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_QUERY_ID => {
            return USBH_PdoQueryId(PortExtension, Irp);
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            if (*PortExtension).PortPdoFlags
                & (USBHUB_PDO_FLAG_INSUFFICIENT_PWR
                    | USBHUB_PDO_FLAG_OVERCURRENT_PORT
                    | USBHUB_PDO_FLAG_PORT_RESTORE_FAIL
                    | USBHUB_PDO_FLAG_INIT_PORT_FAILED)
                != 0
            {
                (*Irp).IoStatus.Information = (*Irp).IoStatus.Information | PNP_DEVICE_FAILED as ntkrnl_sys::ddk::ULONG_PTR;
            }

            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_BUS_INFORMATION => {
            BusInfo = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<PNP_BUS_INFORMATION>() as SIZE_T,
                USB_HUB_TAG as ULONG,
            ) as PPNP_BUS_INFORMATION;

            if BusInfo.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            RtlZeroMemory(
                BusInfo as *mut core::ffi::c_void,
                core::mem::size_of::<PNP_BUS_INFORMATION>(),
            );

            RtlCopyMemory(
                core::ptr::addr_of_mut!((*BusInfo).BusTypeGuid) as *mut core::ffi::c_void,
                core::ptr::addr_of!(GUID_BUS_TYPE_USB) as *const core::ffi::c_void,
                core::mem::size_of::<GUID>(),
            );

            (*BusInfo).LegacyBusType = PNPBus;
            (*BusInfo).BusNumber = 0;

            (*Irp).IoStatus.Information = BusInfo as _;
            Status = STATUS_SUCCESS;
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            DbgBreakPoint();
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        IRP_MN_SURPRISE_REMOVAL => {
            if (*PortExtension).PortPdoFlags & USBHUB_PDO_FLAG_REG_DEV_INTERFACE != 0 {
                Status = USBH_SymbolicLink(PortExtension, core::ptr::null(), 0 as BOOLEAN);

                if NT_SUCCESS(Status) {
                    (*PortExtension).PortPdoFlags &= !USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
                }
            }

            Status = STATUS_SUCCESS;
        }

        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    return Status;
}

// ============================================================================
// ORIGINAL C SOURCE (USBH_FdoPnP / USBH_PdoPnP) — kept for reference
// ============================================================================
//
// NTSTATUS
// NTAPI
// USBH_FdoPnP(IN PUSBHUB_FDO_EXTENSION HubExtension,
//             IN PIRP Irp,
//             IN UCHAR Minor)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     DEVICE_RELATION_TYPE RelationsType;
//     BOOLEAN IsCheckIdle;
//
//     DPRINT_PNP("USBH_FdoPnP: HubExtension - %p, Irp - %p, Minor - %X\n",
//                HubExtension,
//                Irp,
//                Minor);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_WAIT_IDLE_REQUEST &&
//         (Minor == IRP_MN_REMOVE_DEVICE || Minor == IRP_MN_STOP_DEVICE))
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_SUSPENDED;
//     }
//
//     KeWaitForSingleObject(&HubExtension->IdleSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     DPRINT_PNP("USBH_FdoPnP: HubFlags - %lX\n", HubExtension->HubFlags);
//
//     if (HubExtension->HubFlags & USBHUB_FDO_FLAG_GOING_IDLE)
//     {
//         HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_SUSPENDED;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     RelationsType = IoStack->Parameters.QueryDeviceRelations.Type;
//
//     if ((HubExtension->CurrentPowerState.DeviceState == PowerDeviceD0) ||
//         !(HubExtension->HubFlags & (USBHUB_FDO_FLAG_DEVICE_STOPPED | USBHUB_FDO_FLAG_DEVICE_STARTED)) ||
//         (Minor == IRP_MN_QUERY_DEVICE_RELATIONS && RelationsType == TargetDeviceRelation))
//     {
//         IsCheckIdle = FALSE;
//     }
//     else
//     {
//         DPRINT_PNP("USBH_FdoPnP: IsCheckIdle - TRUE\n");
//         IsCheckIdle = TRUE;
//         USBH_HubSetD0(HubExtension);
//     }
//
//     switch (Minor)
//     {
//         case IRP_MN_START_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_START_DEVICE\n");
//             IsCheckIdle = FALSE;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_FdoStartDevice(HubExtension, Irp);
//             break;
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_QUERY_REMOVE_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_REMOVE_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_REMOVE_DEVICE\n");
//             IsCheckIdle = FALSE;
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_DEVICE_REMOVED;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_FdoRemoveDevice(HubExtension, Irp);
//             break;
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_STOP_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_STOP_DEVICE\n");
//             IsCheckIdle = FALSE;
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_FdoStopDevice(HubExtension, Irp);
//             break;
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_QUERY_STOP_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_CANCEL_STOP_DEVICE:
//             DPRINT_PNP("FDO IRP_MN_CANCEL_STOP_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//             DPRINT_PNP("FDO IRP_MN_QUERY_DEVICE_RELATIONS\n");
//
//             if (RelationsType != BusRelations)
//             {
//                 Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//                 break;
//             }
//
//             HubExtension->HubFlags |= USBHUB_FDO_FLAG_HUB_BUSY;
//
//             IsCheckIdle = TRUE;
//             DPRINT_PNP("USBH_FdoPnP: IsCheckIdle - TRUE\n");
//
//             Status = USBH_FdoQueryBusRelations(HubExtension, Irp);
//
//             HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_HUB_BUSY;
//             break;
//
//         case IRP_MN_QUERY_INTERFACE:
//             DPRINT_PNP("FDO IRP_MN_QUERY_INTERFACE\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_CAPABILITIES:
//             DPRINT_PNP("FDO IRP_MN_QUERY_CAPABILITIES\n");
//             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//             IoSetCompletionRoutine(Irp,
//                                    USBH_QueryCapsComplete,
//                                    HubExtension,
//                                    TRUE,
//                                    FALSE,
//                                    FALSE);
//
//             Status = IoCallDriver(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_RESOURCES:
//             DPRINT_PNP("FDO IRP_MN_QUERY_RESOURCES\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//             DPRINT_PNP("FDO IRP_MN_QUERY_RESOURCE_REQUIREMENTS\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_DEVICE_TEXT:
//             DPRINT_PNP("FDO IRP_MN_QUERY_DEVICE_TEXT\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//             DPRINT_PNP("FDO IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_READ_CONFIG:
//             DPRINT_PNP("FDO IRP_MN_READ_CONFIG\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_WRITE_CONFIG:
//             DPRINT_PNP("FDO IRP_MN_WRITE_CONFIG\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_EJECT:
//             DPRINT_PNP("FDO IRP_MN_EJECT\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_SET_LOCK:
//             DPRINT_PNP("FDO IRP_MN_SET_LOCK\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_ID:
//             DPRINT_PNP("FDO IRP_MN_QUERY_ID\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_PNP_DEVICE_STATE:
//             DPRINT_PNP("FDO IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//
//             if (HubExtension->HubFlags & USBHUB_FDO_FLAG_DEVICE_FAILED)
//             {
//                 Irp->IoStatus.Information |= PNP_DEVICE_FAILED;
//             }
//
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_BUS_INFORMATION:
//             DPRINT_PNP("FDO IRP_MN_QUERY_BUS_INFORMATION\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_DEVICE_USAGE_NOTIFICATION:
//             DPRINT_PNP("FDO IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             DPRINT_PNP("FDO IRP_MN_SURPRISE_REMOVAL\n");
//             USBH_FdoSurpriseRemoveDevice(HubExtension, Irp);
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//
//         default:
//             DPRINT_PNP("FDO unknown IRP_MN_???\n");
//             Status = USBH_PassIrp(HubExtension->LowerDevice, Irp);
//             break;
//     }
//
//     KeReleaseSemaphore(&HubExtension->IdleSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (IsCheckIdle)
//     {
//         DPRINT_PNP("USBH_FdoPnP: call USBH_CheckIdleDeferred()\n");
//         USBH_CheckIdleDeferred(HubExtension);
//     }
//
//     HubExtension->HubFlags &= ~USBHUB_FDO_FLAG_STATE_CHANGING;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBH_PdoPnP(IN PUSBHUB_PORT_PDO_EXTENSION PortExtension,
//             IN PIRP Irp,
//             IN UCHAR Minor,
//             OUT BOOLEAN * IsCompleteIrp)
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     PPNP_BUS_INFORMATION BusInfo;
//     PDEVICE_CAPABILITIES DeviceCapabilities;
//     USHORT Size;
//     USHORT Version;
//     PUSBHUB_FDO_EXTENSION HubExtension;
//     PDEVICE_RELATIONS DeviceRelation;
//
//     DPRINT_PNP("USBH_PdoPnP: PortExtension - %p, Irp - %p, Minor - %X\n",
//                PortExtension,
//                Irp,
//                Minor);
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     *IsCompleteIrp = TRUE;
//
//     switch (Minor)
//     {
//         case IRP_MN_START_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_START_DEVICE\n");
//             return USBH_PdoStartDevice(PortExtension, Irp);
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_QUERY_REMOVE_DEVICE\n");
//             return STATUS_SUCCESS;
//
//         case IRP_MN_REMOVE_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_REMOVE_DEVICE\n");
//             return USBH_PdoRemoveDevice(PortExtension, PortExtension->HubExtension);
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             return STATUS_SUCCESS;
//
//         case IRP_MN_STOP_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_STOP_DEVICE\n");
//             return USBH_PdoStopDevice(PortExtension, Irp);
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_QUERY_STOP_DEVICE\n");
//             return STATUS_SUCCESS;
//
//         case IRP_MN_CANCEL_STOP_DEVICE:
//             DPRINT_PNP("PDO IRP_MN_CANCEL_STOP_DEVICE\n");
//             return STATUS_SUCCESS;
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//             DPRINT_PNP("PDO IRP_MN_QUERY_DEVICE_RELATIONS\n");
//
//             if (IoStack->Parameters.QueryDeviceRelations.Type != TargetDeviceRelation)
//             {
//                 return Irp->IoStatus.Status;
//             }
//
//             DeviceRelation = ExAllocatePoolWithTag(PagedPool,
//                                                    sizeof(DEVICE_RELATIONS),
//                                                    USB_HUB_TAG);
//
//             if (DeviceRelation)
//             {
//                 RtlZeroMemory(DeviceRelation, sizeof(DEVICE_RELATIONS));
//
//                 DeviceRelation->Count = 1;
//                 DeviceRelation->Objects[0] = PortExtension->Common.SelfDevice;
//
//                 ObReferenceObject(DeviceRelation->Objects[0]);
//
//                 Status = STATUS_SUCCESS;
//             }
//             else
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             Irp->IoStatus.Information = (ULONG_PTR)DeviceRelation;
//             break;
//
//         case IRP_MN_QUERY_INTERFACE:
//             DPRINT_PNP("PDO IRP_MN_QUERY_INTERFACE\n");
//
//             *IsCompleteIrp = 0;
//
//             if (IsEqualGUIDAligned(IoStack->Parameters.QueryInterface.InterfaceType,
//                                    &USB_BUS_INTERFACE_USBDI_GUID))
//             {
//                 IoStack->Parameters.QueryInterface.InterfaceSpecificData = PortExtension->DeviceHandle;
//             }
//
//             HubExtension = PortExtension->HubExtension;
//
//             if (!HubExtension)
//             {
//                 HubExtension = PortExtension->RootHubExtension;
//             }
//
//             Status = USBH_PassIrp(HubExtension->RootHubPdo, Irp);
//             break;
//
//         case IRP_MN_QUERY_CAPABILITIES:
//             DPRINT_PNP("PDO IRP_MN_QUERY_CAPABILITIES\n");
//
//             DeviceCapabilities = IoStack->Parameters.DeviceCapabilities.Capabilities;
//
//             Size = DeviceCapabilities->Size;
//             Version = DeviceCapabilities->Version;
//
//             RtlCopyMemory(DeviceCapabilities,
//                           &PortExtension->Capabilities,
//                           sizeof(DEVICE_CAPABILITIES));
//
//             DeviceCapabilities->Size = Size;
//             DeviceCapabilities->Version = Version;
//
//             /* All devices connected to a hub are removable */
//             DeviceCapabilities->Removable = 1;
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_RESOURCES:
//             DPRINT_PNP("PDO IRP_MN_QUERY_RESOURCES\n");
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//             DPRINT_PNP("PDO IRP_MN_QUERY_RESOURCE_REQUIREMENTS\n");
//             PortExtension->PortPdoFlags |= USBHUB_PDO_FLAG_ENUMERATED;
//
//             /* FIXME HKEY_LOCAL_MACHINE\SYSTEM\ControlSetXXX\Enum\USB\
//                Vid_????&Pid_????\????????????\Device Parameters\
//                if (ExtPropDescSemaphore)
//             */
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_DEVICE_TEXT:
//             DPRINT_PNP("PDO IRP_MN_QUERY_DEVICE_TEXT\n");
//             return USBH_PdoQueryDeviceText(PortExtension, Irp);
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//             DPRINT_PNP("PDO IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_READ_CONFIG:
//             DPRINT_PNP("PDO IRP_MN_READ_CONFIG\n");
//             DbgBreakPoint();
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_WRITE_CONFIG:
//             DPRINT_PNP("PDO IRP_MN_WRITE_CONFIG\n");
//             DbgBreakPoint();
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_EJECT:
//             DPRINT_PNP("PDO IRP_MN_EJECT\n");
//             DbgBreakPoint();
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_SET_LOCK:
//             DPRINT_PNP("PDO IRP_MN_SET_LOCK\n");
//             DbgBreakPoint();
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_QUERY_ID:
//             DPRINT_PNP("PDO IRP_MN_QUERY_ID\n");
//             return USBH_PdoQueryId(PortExtension, Irp);
//
//         case IRP_MN_QUERY_PNP_DEVICE_STATE:
//             DPRINT_PNP("PDO IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//             if (PortExtension->PortPdoFlags & (USBHUB_PDO_FLAG_INSUFFICIENT_PWR |
//                                                USBHUB_PDO_FLAG_OVERCURRENT_PORT |
//                                                USBHUB_PDO_FLAG_PORT_RESTORE_FAIL |
//                                                USBHUB_PDO_FLAG_INIT_PORT_FAILED))
//             {
//                 Irp->IoStatus.Information |= PNP_DEVICE_FAILED;
//             }
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_BUS_INFORMATION:
//             DPRINT_PNP("PDO IRP_MN_QUERY_BUS_INFORMATION\n");
//
//             BusInfo = ExAllocatePoolWithTag(PagedPool,
//                                             sizeof(PNP_BUS_INFORMATION),
//                                             USB_HUB_TAG);
//
//             if (!BusInfo)
//             {
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             RtlZeroMemory(BusInfo, sizeof(PNP_BUS_INFORMATION));
//
//             RtlCopyMemory(&BusInfo->BusTypeGuid,
//                           &GUID_BUS_TYPE_USB,
//                           sizeof(BusInfo->BusTypeGuid));
//
//             BusInfo->LegacyBusType = PNPBus;
//             BusInfo->BusNumber = 0;
//
//             Irp->IoStatus.Information = (ULONG_PTR)BusInfo;
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_DEVICE_USAGE_NOTIFICATION:
//             DPRINT_PNP("PDO IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
//             DbgBreakPoint();
//             Status = Irp->IoStatus.Status;
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             DPRINT_PNP("PDO IRP_MN_SURPRISE_REMOVAL\n");
//             if (PortExtension->PortPdoFlags & USBHUB_PDO_FLAG_REG_DEV_INTERFACE)
//             {
//                 Status = USBH_SymbolicLink(PortExtension, NULL, FALSE);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     PortExtension->PortPdoFlags &= ~USBHUB_PDO_FLAG_REG_DEV_INTERFACE;
//                 }
//             }
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         default:
//             DPRINT_PNP("PDO unknown IRP_MN_???\n");
//             Status = Irp->IoStatus.Status;
//             break;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
