use crate::types::*;
use crate::glue::*;
use crate::glue::{IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_ID, IRP_MN_QUERY_DEVICE_TEXT, IRP_MN_QUERY_CAPABILITIES, IRP_MN_WRITE_CONFIG, IRP_MN_SET_LOCK, IRP_MN_READ_CONFIG, IRP_MN_QUERY_RESOURCE_REQUIREMENTS, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_BUS_INFORMATION, IRP_MN_FILTER_RESOURCE_REQUIREMENTS, IRP_MN_EJECT, IRP_MN_DEVICE_USAGE_NOTIFICATION}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
use crate::cb;
use crate::wstr;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, DEVICE_CAPABILITIES, DEVICE_DESCRIPTION, DEVICE_RELATIONS, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoAllocateIrp, IoBuildSynchronousFsdRequest, IoConnectInterrupt, IoCopyCurrentIrpStackLocationToNext, IoCreateDevice, IoCreateSymbolicLink, IoCsqInitialize, IoDeleteDevice, IoDetachDevice, IoDisconnectInterrupt, IoFreeIrp, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IoGetDmaAdapter, IoGetNextIrpStackLocation, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, IoSkipCurrentIrpStackLocation, KEVENT, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, MmMapIoSpace, _EVENT_TYPE_NotificationEvent, PBUS_INTERFACE_STANDARD, PCI_COMMON_CONFIG, PCI_DEVICE_PRESENCE_PARAMETERS, PCI_DEVICE_PRESENT_INTERFACE, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDMA_ADAPTER, PINTERFACE, PIO_STACK_LOCATION, PIRP, PKEVENT, PNP_BUS_INFORMATION, PPNP_BUS_INFORMATION, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, _KWAIT_REASON_Suspended, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;

extern "system" {
    fn USBPORT_SetRegistryKeyValue(DeviceObject: PDEVICE_OBJECT, UseDeviceParameter: u32, Type: u32, ValueName: PWSTR, Data: *mut core::ffi::c_void, DataSize: u32) -> NTSTATUS;
    fn RtlFreeUnicodeString(UnicodeString: *mut _UNICODE_STRING);
    fn RtlQueryRegistryValues( RelativeTo: u32, Path: PWSTR, QueryTable: *mut RTL_QUERY_REGISTRY_TABLE, Context: *mut core::ffi::c_void, Environment: *mut core::ffi::c_void, ) -> NTSTATUS;
    fn USBPORT_DumpingCapabilities(Capabilities: PDEVICE_CAPABILITIES);
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: PWSTR);
    fn USBPORT_AdjustDeviceCapabilities( FdoDevice: PDEVICE_OBJECT, PdoDevice: PDEVICE_OBJECT, );
    fn USBPORT_DumpingIDs(Buffer: PVOID);
    fn USBPORT_AllocateCommonBuffer( FdoDevice: PDEVICE_OBJECT, BufferLength: SIZE_T, ) -> PUSBPORT_COMMON_BUFFER_HEADER;
    fn USBPORT_CreateWorkerThread(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS;
    fn USBPORT_FreeCommonBuffer( FdoDevice: PDEVICE_OBJECT, HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER, );
    fn USBPORT_GetRegistryKeyValueFullInfo(FdoDevice: PDEVICE_OBJECT, PdoDevice: PDEVICE_OBJECT, UseHardwareKey: u32, KeyNameString: PWSTR, KeyNameStringLength: ULONG, Data: PVOID, DataLength: ULONG) -> NTSTATUS;
    fn USBPORT_MiniportInterrupts(FdoDevice: PDEVICE_OBJECT, IsEnable: BOOLEAN);
    pub fn USBPORT_StartTimer(FdoDevice: PDEVICE_OBJECT, Time: ULONG) -> BOOLEAN;
    fn USBPORT_StopWorkerThread(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_PdoQueryInterface( FdoDevice: PDEVICE_OBJECT, PdoDevice: PDEVICE_OBJECT, Irp: PIRP, ) -> NTSTATUS;
    fn USBPORT_RootHubCreateDevice( FdoDevice: PDEVICE_OBJECT, PdoDevice: PDEVICE_OBJECT, ) -> NTSTATUS;
    fn USBPORT_AddUSB1Fdo(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_AddUSB2Fdo(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_InvalidateControllerHandler( FdoDevice: PDEVICE_OBJECT, Type: ULONG, ) -> ();
}
unsafe extern "C" {
    fn RtlStringCbCopyW(pszDest: PWSTR, cbDest: usize, pszSrc: *const u16) -> NTSTATUS;
}
extern "C" {
    fn RtlStringCbPrintfW(pszDest: PWSTR, cbDest: usize, pszFormat: PWSTR, ...) -> NTSTATUS;
    fn RtlStringCbPrintfExW( pszDest: PWSTR, cbDest: usize, ppszDestEnd: *mut PWSTR, pcbRemaining: *mut usize, dwFlags: u32, pszFormat: *const u16, ... ) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;

pub const NDEBUG_USBPORT_CORE: u32 = 1;

const USBPORT_FLAG_DOS_SYMBOLIC_NAME: u32 = 0x00010000; // FIXED 2026-07-05: translation invented 0x40; verified vs usbport.h/usbmport.h

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FdoStartCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    const EVENT_INCREMENT: i32 = 1;
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
    KeSetEvent((Context as *mut KEVENT as *mut c_void), EVENT_INCREMENT as i32, 0);
    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RegisterDeviceInterface(
    PdoDevice: PDEVICE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    InterfaceClassGuid: *const GUID,
    Enable: BOOLEAN,
) -> NTSTATUS {

    let DeviceExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let SymbolicLinkName: *mut _UNICODE_STRING;
    let mut Status: NTSTATUS;

    // DPRINT("USBPORT_RegisterDeviceInterface: Enable - %x\n", Enable);

    DeviceExtension = (*DeviceObject).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    SymbolicLinkName = core::ptr::addr_of_mut!((*DeviceExtension).CommonExtension.SymbolicLinkName);

    if Enable != 0 {
        Status = IoRegisterDeviceInterface(
            PdoDevice,
            InterfaceClassGuid,
            core::ptr::null_mut(),
            SymbolicLinkName,
        );

        if NT_SUCCESS(Status) {
            (*DeviceExtension).CommonExtension.IsInterfaceEnabled = 1;

            Status = USBPORT_SetRegistryKeyValue(
                PdoDevice,
                0,
                REG_SZ,
                wstr!("SymbolicName\0") as PWSTR,
                (*SymbolicLinkName).Buffer as *mut core::ffi::c_void,
                (*SymbolicLinkName).Length as u32,
            );

            if NT_SUCCESS(Status) {
                // DPRINT("USBPORT_RegisterDeviceInterface: LinkName  - %wZ\n",
                //        &DeviceExtension->CommonExtension.SymbolicLinkName);

                Status = IoSetDeviceInterfaceState(SymbolicLinkName, 1);
            }
        }
    } else {
        /* Disable device interface */
        Status = IoSetDeviceInterfaceState(SymbolicLinkName, 0);

        if NT_SUCCESS(Status) {
            RtlFreeUnicodeString(SymbolicLinkName);
            (*DeviceExtension).CommonExtension.IsInterfaceEnabled = 0; // Disabled interface
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IsSelectiveSuspendEnabled(
    FdoDevice: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Disabled: ULONG = 0;

    // DPRINT("USBPORT_IsSelectiveSuspendEnabled: ... \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    USBPORT_GetRegistryKeyValueFullInfo(
        FdoDevice,
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        wstr!("HcDisableSelectiveSuspend\0") as PWSTR,
        core::mem::size_of::<[u16; 26]>() as ULONG,
        core::ptr::addr_of_mut!(Disabled) as *mut core::ffi::c_void,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    (Disabled == 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetConfigValue(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if ValueType == REG_DWORD {
        *(EntryContext as PULONG) = *(ValueData as PULONG);
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetDefaultBIOSx(
    FdoDevice: PDEVICE_OBJECT,
    UsbBIOSx: PULONG,
    DisableSelectiveSuspend: PULONG,
    DisableCcDetect: PULONG,
    IdleEpSupport: PULONG,
    IdleEpSupportEx: PULONG,
    SoftRetry: PULONG,
) -> NTSTATUS {

    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 7] = core::mem::zeroed();

    // DPRINT("USBPORT_GetDefaultBIOS_X: ... \n");

    RtlZeroMemory(
        QueryTable.as_mut_ptr() as *mut core::ffi::c_void,
        (core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 7]>() as u32 as usize),
    );

    *UsbBIOSx = 2;

    QueryTable[0].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[0].Flags = 0;
    QueryTable[0].Name = wstr!("UsbBIOSx\0") as PWSTR;
    QueryTable[0].EntryContext = UsbBIOSx as *mut core::ffi::c_void;
    QueryTable[0].DefaultType = REG_DWORD;
    QueryTable[0].DefaultData = UsbBIOSx as *mut core::ffi::c_void;
    QueryTable[0].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    QueryTable[1].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[1].Flags = 0;
    QueryTable[1].Name = wstr!("DisableSelectiveSuspend\0") as PWSTR;
    QueryTable[1].EntryContext = DisableSelectiveSuspend as *mut core::ffi::c_void;
    QueryTable[1].DefaultType = REG_DWORD;
    QueryTable[1].DefaultData = DisableSelectiveSuspend as *mut core::ffi::c_void;
    QueryTable[1].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    QueryTable[2].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[2].Flags = 0;
    QueryTable[2].Name = wstr!("DisableCcDetect\0") as PWSTR;
    QueryTable[2].EntryContext = DisableCcDetect as *mut core::ffi::c_void;
    QueryTable[2].DefaultType = REG_DWORD;
    QueryTable[2].DefaultData = DisableCcDetect as *mut core::ffi::c_void;
    QueryTable[2].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    QueryTable[3].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[3].Flags = 0;
    QueryTable[3].Name = wstr!("EnIdleEndpointSupport\0") as PWSTR;
    QueryTable[3].EntryContext = IdleEpSupport as *mut core::ffi::c_void;
    QueryTable[3].DefaultType = REG_DWORD;
    QueryTable[3].DefaultData = IdleEpSupport as *mut core::ffi::c_void;
    QueryTable[3].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    QueryTable[4].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[4].Flags = 0;
    QueryTable[4].Name = wstr!("EnIdleEndpointSupportEx\0") as PWSTR;
    QueryTable[4].EntryContext = IdleEpSupportEx as *mut core::ffi::c_void;
    QueryTable[4].DefaultType = REG_DWORD;
    QueryTable[4].DefaultData = IdleEpSupportEx as *mut core::ffi::c_void;
    QueryTable[4].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    QueryTable[5].QueryRoutine = cb!(USBPORT_GetConfigValue);
    QueryTable[5].Flags = 0;
    QueryTable[5].Name = wstr!("EnSoftRetry\0") as PWSTR;
    QueryTable[5].EntryContext = SoftRetry as *mut core::ffi::c_void;
    QueryTable[5].DefaultType = REG_DWORD;
    QueryTable[5].DefaultData = SoftRetry as *mut core::ffi::c_void;
    QueryTable[5].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    return RtlQueryRegistryValues(
        RTL_REGISTRY_SERVICES,
        wstr!("usb\0") as PWSTR,
        QueryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_IsCompanionController(
    DeviceObject: PDEVICE_OBJECT,
    IsCompanion: *mut BOOLEAN,
) -> NTSTATUS {
    let HighestDevice: PDEVICE_OBJECT;
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let IoStack: PIO_STACK_LOCATION;
    let mut PciInterface: PCI_DEVICE_PRESENT_INTERFACE = core::mem::zeroed();
    let mut Parameters: PCI_DEVICE_PRESENCE_PARAMETERS = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let IsPresent: BOOLEAN;

    *IsCompanion = 0;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0);

    HighestDevice = IoGetAttachedDeviceReference(DeviceObject);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        HighestDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        ObDereferenceObject(HighestDevice as *mut _);
        return Status;
    }

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;

    (*IoStack).MinorFunction = (IRP_MN_QUERY_INTERFACE as u8);

    (*IoStack).Parameters.QueryInterface.InterfaceType =
        core::ptr::addr_of!(GUID_PCI_DEVICE_PRESENT_INTERFACE);
    (*IoStack).Parameters.QueryInterface.Size =
        core::mem::size_of::<PCI_DEVICE_PRESENT_INTERFACE>() as u16;
    (*IoStack).Parameters.QueryInterface.Version = 1;
    (*IoStack).Parameters.QueryInterface.Interface =
        core::ptr::addr_of_mut!(PciInterface) as PINTERFACE;
    (*IoStack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

    Status = IoCallDriver(HighestDevice, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut _,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    if !NT_SUCCESS(Status) {
        ObDereferenceObject(HighestDevice as *mut _);
        return Status;
    }

    if (PciInterface.Size as usize) < core::mem::size_of::<PCI_DEVICE_PRESENT_INTERFACE>() {
        ObDereferenceObject(HighestDevice as *mut _);
        return Status;
    }

    Parameters.Size = (core::mem::size_of::<PCI_DEVICE_PRESENT_INTERFACE>() as u16 as u32);

    Parameters.BaseClass = (PCI_CLASS_SERIAL_BUS_CTLR as u8);
    Parameters.SubClass = (PCI_SUBCLASS_SB_USB as u8);
    Parameters.ProgIf = (PCI_INTERFACE_USB_ID_EHCI as u8);

    Parameters.Flags = PCI_USE_LOCAL_BUS
        | PCI_USE_LOCAL_DEVICE
        | PCI_USE_CLASS_SUBCLASS
        | PCI_USE_PROGIF;

    IsPresent = (PciInterface.IsDevicePresentEx).unwrap()(
        PciInterface.Context,
        core::ptr::addr_of_mut!(Parameters),
    );

    if IsPresent != 0 {
    } else {
    }

    *IsCompanion = IsPresent;

    (PciInterface.InterfaceDereference).unwrap()(PciInterface.Context);

    ObDereferenceObject(HighestDevice as *mut _);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueryPciBusInterface(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let BusInterface: PBUS_INTERFACE_STANDARD;
    let IoStack: PIO_STACK_LOCATION;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let HighestDevice: PDEVICE_OBJECT;
    let mut Event: KEVENT = core::mem::zeroed();
    let Irp: PIRP;
    let mut Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    BusInterface = core::ptr::addr_of_mut!((*FdoExtension).BusInterface);

    RtlZeroMemory(BusInterface as *mut _, core::mem::size_of::<BUS_INTERFACE_STANDARD>());
    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_SynchronizationEvent, 0);
    HighestDevice = IoGetAttachedDeviceReference(FdoDevice);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        HighestDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );

    if !Irp.is_null() {
        IoStack = IoGetNextIrpStackLocation(Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;

        (*IoStack).MinorFunction = IRP_MN_QUERY_INTERFACE as u8;

        (*IoStack).Parameters.QueryInterface.InterfaceType =
            core::ptr::addr_of!(GUID_BUS_INTERFACE_STANDARD);
        (*IoStack).Parameters.QueryInterface.Size =
            core::mem::size_of::<BUS_INTERFACE_STANDARD>() as u16;
        (*IoStack).Parameters.QueryInterface.Version = 1;
        (*IoStack).Parameters.QueryInterface.Interface = BusInterface as PINTERFACE;
        (*IoStack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

        Status = IoCallDriver(HighestDevice, Irp);

        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as *mut _,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    ObDereferenceObject(HighestDevice as *mut _);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueryCapabilities(
    FdoDevice: PDEVICE_OBJECT,
    Capabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {

    let FdoExtention: PUSBPORT_DEVICE_EXTENSION;
    let Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();

    FdoExtention = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    RtlZeroMemory(
        Capabilities as *mut _,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );

    (*Capabilities).Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    (*Capabilities).Version = 1;
    (*Capabilities).Address = MAXULONG;
    (*Capabilities).UINumber = MAXULONG;

    Irp = IoAllocateIrp(
        (*(*FdoExtention).CommonExtension.LowerDevice).StackSize,
        0,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).MajorFunction = IRP_MJ_PNP as u8;
    (*IoStack).MinorFunction = IRP_MN_QUERY_CAPABILITIES as u8;

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    IoSetCompletionRoutine(
        Irp,
        cb!(USBPORT_FdoStartCompletion),
        core::ptr::addr_of_mut!(Event) as *mut _,
        1,
        1,
        1,
    );

    (*IoStack).Parameters.DeviceCapabilities.Capabilities = Capabilities;

    Status = IoCallDriver((*FdoExtention).CommonExtension.LowerDevice, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut _,
            _KWAIT_REASON_Suspended,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if NT_SUCCESS(Status) && !Capabilities.is_null() {
        USBPORT_DumpingCapabilities(Capabilities);
    }

    IoFreeIrp(Irp);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CreateLegacySymbolicLink(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut CharName: [WCHAR; 255] = [0; 255];
    let mut CharDosName: [WCHAR; 255] = [0; 255];
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    RtlStringCbPrintfW(
        CharName.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; 255]>(),
        ['\\' as u16, 'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '\\' as u16, 'U' as u16, 'S' as u16, 'B' as u16, 'F' as u16, 'D' as u16, 'O' as u16, '-' as u16, '%' as u16, 'd' as u16, 0].as_ptr() as PWSTR,
        (*FdoExtension).FdoNameNumber,
    );

    RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceName), CharName.as_mut_ptr());

    RtlStringCbPrintfW(
        CharDosName.as_mut_ptr(),
        core::mem::size_of::<[WCHAR; 255]>(),
        ['\\' as u16, 'D' as u16, 'o' as u16, 's' as u16, 'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16, '\\' as u16, 'H' as u16, 'C' as u16, 'D' as u16, '%' as u16, 'd' as u16, 0].as_ptr() as PWSTR,
        (*FdoExtension).FdoNameNumber,
    );

    RtlInitUnicodeString(core::ptr::addr_of_mut!((*FdoExtension).DosDeviceSymbolicName), CharDosName.as_mut_ptr());

    Status = IoCreateSymbolicLink(
        core::ptr::addr_of_mut!((*FdoExtension).DosDeviceSymbolicName),
        core::ptr::addr_of_mut!(DeviceName),
    );

    if NT_SUCCESS(Status) {
        (*FdoExtension).Flags |= USBPORT_FLAG_DOS_SYMBOLIC_NAME;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_StopDevice(FdoDevice: PDEVICE_OBJECT) -> NTSTATUS {
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_StartDevice(
    FdoDevice: PDEVICE_OBJECT,
    UsbPortResources: PUSBPORT_RESOURCES,
) -> NTSTATUS {

    const PCI_WHICHSPACE_CONFIG: ULONG = 0;
    const PCI_COMMON_HDR_LENGTH: ULONG = 64; // FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific)
    const DEVICE_DESCRIPTION_VERSION: ULONG = 0;
    const MAXIMUM_LENGTH_MAXULONG: ULONG = MAXULONG;
    const USBPORT_FLAG_BIOS_DISABLE_SS: ULONG = 0x08000000; // FIXED 2026-07-05: translation invented 0x80; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_SELECTIVE_SUSPEND: ULONG = 0x00000800; // FIXED 2026-07-05: translation invented 0x40; verified vs usbport.h/usbmport.h
    const USB_MINIPORT_FLAGS_POLLING: ULONG = 0x00000080; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const USB_MINIPORT_FLAGS_WAKE_SUPPORT: ULONG = 0x00000200; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USB_MINIPORT_FLAGS_DISABLE_SS: ULONG = 0x00000020;
    const USBPORT_FLAG_HC_POLLING: ULONG = 0x00000004; // FIXED 2026-07-05: translation invented 0x800; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_HC_WAKE_SUPPORT: ULONG = 0x00200000; // FIXED 2026-07-05: translation invented 0x1000; verified vs usbport.h/usbmport.h
    const PCI_CLASS_SERIAL_BUS_CTLR: UCHAR = 0x0C;
    const PCI_SUBCLASS_SB_USB: UCHAR = 0x03;
    const PCI_INTERFACE_USB_ID_EHCI: UCHAR = 0x20;
    const USBPORT_FLAG_COMPANION_HC: ULONG = 0x01000000; // FIXED 2026-07-05: translation invented 0x100; verified vs usbport.h/usbmport.h
    const USB2_FRAMES: ULONG = 32;
    const USB_PORT_TAG: ULONG = 0x70627375; // 'Ussp'
    const USBPORT_FLAG_INT_CONNECTED: ULONG = 0x00000001; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_LEGACY_SUPPORT: ULONG = 0x00080000; // FIXED 2026-07-05: translation invented 0x40000; verified vs usbport.h/usbmport.h
    const USBPORT_MPFLAG_INTERRUPTS_ENABLED: ULONG = 0x00000001;
    const USBPORT_FLAG_HC_STARTED: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x20; verified vs usbport.h/usbmport.h
    const Width32Bits: i32 = 0x02;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Status: NTSTATUS;
    let mut PciConfig: PCI_COMMON_CONFIG = core::mem::zeroed();
    let BytesRead: ULONG;
    let mut DeviceDescription: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut DmaAdapter: PDMA_ADAPTER = core::ptr::null_mut();
    let MiniPortStatus: ULONG;
    let HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER;
    let mut ResultLength: ULONG = 0;
    let mut DisableSelectiveSuspend: ULONG = 0;
    let mut DisableCcDetect: ULONG = 0;
    let mut IdleEpSupport: ULONG = 0;
    let mut IdleEpSupportEx: ULONG = 0;
    let mut SoftRetry: ULONG = 0;
    let mut Limit2GB: ULONG = 0;
    let mut TotalBusBandwidth: ULONG = 0;
    let mut IsCompanion: BOOLEAN = 0;
    let LegacyBIOS: ULONG;
    let MiniportFlags: ULONG;
    let mut ix: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    Status = USBPORT_QueryPciBusInterface(FdoDevice);
    if !NT_SUCCESS(Status) {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    BytesRead = ((*FdoExtension).BusInterface.GetBusData).unwrap()(
        (*FdoExtension).BusInterface.Context,
        PCI_WHICHSPACE_CONFIG,
        core::ptr::addr_of_mut!(PciConfig) as PVOID,
        0,
        PCI_COMMON_HDR_LENGTH,
    );

    if BytesRead != PCI_COMMON_HDR_LENGTH {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    let _pci = core::ptr::addr_of!(PciConfig) as *const u8;
    (*FdoExtension).VendorID = *(_pci as *const USHORT);
    (*FdoExtension).DeviceID = *(_pci.add(2) as *const USHORT);
    (*FdoExtension).RevisionID = *_pci.add(8);
    (*FdoExtension).ProgIf = *_pci.add(9);
    (*FdoExtension).SubClass = *_pci.add(10);
    (*FdoExtension).BaseClass = *_pci.add(11);

    RtlZeroMemory(
        core::ptr::addr_of_mut!(DeviceDescription) as PVOID,
        core::mem::size_of::<DEVICE_DESCRIPTION>(),
    );

    DeviceDescription.Version = DEVICE_DESCRIPTION_VERSION;
    DeviceDescription.Master = 1;
    DeviceDescription.ScatterGather = 1;
    DeviceDescription.Dma32BitAddresses = 1;
    DeviceDescription.InterfaceType = PCIBus;
    DeviceDescription.DmaWidth = (Width32Bits as u32);
    DeviceDescription.DmaSpeed = Compatible;
    DeviceDescription.MaximumLength = MAXIMUM_LENGTH_MAXULONG;

    DmaAdapter = IoGetDmaAdapter(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        core::ptr::addr_of_mut!(DeviceDescription),
        core::ptr::addr_of_mut!((*FdoExtension).NumberMapRegs),
    );

    (*FdoExtension).DmaAdapter = DmaAdapter;

    if DmaAdapter.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    Status = USBPORT_CreateWorkerThread(FdoDevice);
    if !NT_SUCCESS(Status) {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    Status = USBPORT_QueryCapabilities(
        FdoDevice,
        core::ptr::addr_of_mut!((*FdoExtension).Capabilities),
    );
    if !NT_SUCCESS(Status) {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    (*FdoExtension).PciDeviceNumber = (*FdoExtension).Capabilities.Address >> 16;
    (*FdoExtension).PciFunctionNumber = (*FdoExtension).Capabilities.Address & 0xFFFF;

    Status = IoGetDeviceProperty(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        DevicePropertyBusNumber,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!((*FdoExtension).BusNumber) as PVOID,
        core::ptr::addr_of_mut!(ResultLength),
    );

    if !NT_SUCCESS(Status) {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsqSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsqSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).MapTransferSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).DoneTransferSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).WorkerThreadEventSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).TimerFlagsSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).PowerWakeSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).SetPowerD0SpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).RootHubCallbackSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock));

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).IsrDpc),
        cb!(USBPORT_IsrDpc),
        FdoDevice as PVOID,
    );

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).TransferFlushDpc),
        cb!(USBPORT_TransferFlushDpc),
        FdoDevice as PVOID,
    );

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).WorkerRequestDpc),
        cb!(USBPORT_WorkerRequestDpc),
        FdoDevice as PVOID,
    );

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*FdoExtension).HcWakeDpc),
        cb!(USBPORT_HcWakeDpc),
        FdoDevice as PVOID,
    );

    IoCsqInitialize(
        core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsq),
        cb!(USBPORT_InsertIdleIrp),
        cb!(USBPORT_RemoveIdleIrp),
        cb!(USBPORT_PeekNextIdleIrp),
        cb!(USBPORT_AcquireIdleLock),
        cb!(USBPORT_ReleaseIdleLock),
        cb!(USBPORT_CompleteCanceledIdleIrp),
    );

    IoCsqInitialize(
        core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsq),
        cb!(USBPORT_InsertBadRequest),
        cb!(USBPORT_RemoveBadRequest),
        cb!(USBPORT_PeekNextBadRequest),
        cb!(USBPORT_AcquireBadRequestLock),
        cb!(USBPORT_ReleaseBadRequestLock),
        cb!(USBPORT_CompleteCanceledBadRequest),
    );

    (*FdoExtension).IsrDpcCounter = -1;
    (*FdoExtension).IsrDpcHandlerCounter = -1;
    (*FdoExtension).IdleLockCounter = -1;
    (*FdoExtension).BadRequestLockCounter = -1;
    (*FdoExtension).ChirpRootPortLock = -1;

    (*FdoExtension).RHInitCallBackLock = 0;

    (*FdoExtension).UsbAddressBitMap[0] = 1;
    (*FdoExtension).UsbAddressBitMap[1] = 0;
    (*FdoExtension).UsbAddressBitMap[2] = 0;
    (*FdoExtension).UsbAddressBitMap[3] = 0;

    USBPORT_GetDefaultBIOSx(
        FdoDevice,
        core::ptr::addr_of_mut!((*FdoExtension).UsbBIOSx),
        core::ptr::addr_of_mut!(DisableSelectiveSuspend),
        core::ptr::addr_of_mut!(DisableCcDetect),
        core::ptr::addr_of_mut!(IdleEpSupport),
        core::ptr::addr_of_mut!(IdleEpSupportEx),
        core::ptr::addr_of_mut!(SoftRetry),
    );

    if DisableSelectiveSuspend != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_BIOS_DISABLE_SS;
    }

    if DisableSelectiveSuspend == 0 && USBPORT_IsSelectiveSuspendEnabled(FdoDevice) != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
    }

    MiniportFlags = (*Packet).MiniPortFlags;

    if MiniportFlags & USB_MINIPORT_FLAGS_POLLING != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_HC_POLLING;
    }

    if MiniportFlags & USB_MINIPORT_FLAGS_WAKE_SUPPORT != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_HC_WAKE_SUPPORT;
    }

    if MiniportFlags & USB_MINIPORT_FLAGS_DISABLE_SS != 0 {
        (*FdoExtension).Flags = ((*FdoExtension).Flags & !USBPORT_FLAG_SELECTIVE_SUSPEND)
            | USBPORT_FLAG_BIOS_DISABLE_SS;
    }

    USBPORT_SetRegistryKeyValue(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        REG_DWORD,
        wstr!("EnIdleEndpointSupport\0") as PWSTR,
        core::ptr::addr_of_mut!(IdleEpSupport) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    USBPORT_SetRegistryKeyValue(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        REG_DWORD,
        wstr!("EnIdleEndpointSupportEx\0") as PWSTR,
        core::ptr::addr_of_mut!(IdleEpSupportEx) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    USBPORT_SetRegistryKeyValue(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        REG_DWORD,
        wstr!("EnSoftRetry\0") as PWSTR,
        core::ptr::addr_of_mut!(SoftRetry) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    USBPORT_GetRegistryKeyValueFullInfo(
        FdoDevice,
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        wstr!("CommonBuffer2GBLimit\0") as PWSTR,
        (core::mem::size_of::<u16>() * "CommonBuffer2GBLimit\0".chars().count()) as ULONG,
        core::ptr::addr_of_mut!(Limit2GB) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    (*FdoExtension).CommonBufferLimit = ((Limit2GB != 0) as BOOLEAN as u32);

    if (*FdoExtension).BaseClass == PCI_CLASS_SERIAL_BUS_CTLR
        && (*FdoExtension).SubClass == PCI_SUBCLASS_SB_USB
        && (*FdoExtension).ProgIf < PCI_INTERFACE_USB_ID_EHCI
    {
        Status = USBPORT_IsCompanionController(FdoDevice, core::ptr::addr_of_mut!(IsCompanion));

        if !NT_SUCCESS(Status) {
            if IsCompanion != 0 {
                (*FdoExtension).Flags |= USBPORT_FLAG_COMPANION_HC;
            } else {
                (*FdoExtension).Flags &= !USBPORT_FLAG_COMPANION_HC;
            }
        }
    }

    if DisableCcDetect != 0 {
        (*FdoExtension).Flags &= !USBPORT_FLAG_COMPANION_HC;
    }

    TotalBusBandwidth = (*Packet).MiniPortBusBandwidth;
    (*FdoExtension).TotalBusBandwidth = TotalBusBandwidth;

    USBPORT_GetRegistryKeyValueFullInfo(
        FdoDevice,
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        wstr!("TotalBusBandwidth\0") as PWSTR,
        (core::mem::size_of::<u16>() * "TotalBusBandwidth\0".chars().count()) as ULONG,
        core::ptr::addr_of_mut!(TotalBusBandwidth) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    if TotalBusBandwidth != (*FdoExtension).TotalBusBandwidth {
        (*FdoExtension).TotalBusBandwidth = TotalBusBandwidth;
    }

    ix = 0;
    while ix < USB2_FRAMES {
        (*FdoExtension).Bandwidth[ix as usize] =
            (*FdoExtension).TotalBusBandwidth - (*FdoExtension).TotalBusBandwidth / 10;
        ix += 1;
    }

    (*FdoExtension).ActiveIrpTable = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBPORT_IRP_TABLE>(),
        USB_PORT_TAG,
    ) as *mut USBPORT_IRP_TABLE;

    if (*FdoExtension).ActiveIrpTable.is_null() {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    RtlZeroMemory(
        (*FdoExtension).ActiveIrpTable as PVOID,
        core::mem::size_of::<USBPORT_IRP_TABLE>(),
    );

    (*FdoExtension).PendingIrpTable = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBPORT_IRP_TABLE>(),
        USB_PORT_TAG,
    ) as *mut USBPORT_IRP_TABLE;

    if (*FdoExtension).PendingIrpTable.is_null() {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    RtlZeroMemory(
        (*FdoExtension).PendingIrpTable as PVOID,
        core::mem::size_of::<USBPORT_IRP_TABLE>(),
    );

    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*FdoExtension).InterruptObject),
        cb!(USBPORT_InterruptService),
        FdoDevice as PVOID,
        core::ptr::null_mut(),
        (*UsbPortResources).InterruptVector,
        (*UsbPortResources).InterruptLevel,
        (*UsbPortResources).InterruptLevel,
        (*UsbPortResources).InterruptMode,
        (*UsbPortResources).ShareVector,
        (*UsbPortResources).InterruptAffinity,
        0,
    );

    if !NT_SUCCESS(Status) {
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    // The C clears the flag here, so nothing ever disconnects the interrupt: a
    // controller whose START fails below leaves its KINTERRUPT in the IDT after
    // the HAL has given the vector back, and the next device handed that vector
    // cannot connect to it (seen as scsiport's IDE line failing, then 0xA).
    (*FdoExtension).Flags |= USBPORT_FLAG_INT_CONNECTED;

    if (*Packet).MiniPortExtensionSize != 0 {
        RtlZeroMemory(
            (*FdoExtension).MiniPortExt as PVOID,
            (*Packet).MiniPortExtensionSize as usize,
        );
    }

    if (*Packet).MiniPortResourcesSize != 0 {
        HeaderBuffer = USBPORT_AllocateCommonBuffer(FdoDevice, (*Packet).MiniPortResourcesSize as _);

        if HeaderBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            USBPORT_StopWorkerThread(FdoDevice);
            USBPORT_StopDevice(FdoDevice);
            return Status;
        }

        (*UsbPortResources).StartVA = (*HeaderBuffer).VirtualAddress;
        (*UsbPortResources).StartPA = (*HeaderBuffer).PhysicalAddress;

        (*FdoExtension).MiniPortCommonBuffer = HeaderBuffer;
    } else {
        (*FdoExtension).MiniPortCommonBuffer = core::ptr::null_mut();
    }

    MiniPortStatus = ((*Packet).StartController).unwrap()(
        (*FdoExtension).MiniPortExt,
        UsbPortResources,
    );

    if (*UsbPortResources).LegacySupport != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_LEGACY_SUPPORT;
        LegacyBIOS = 1;
    } else {
        LegacyBIOS = 0;
    }

    USBPORT_SetRegistryKeyValue(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        0,
        REG_DWORD,
        wstr!("DetectedLegacyBIOS\0") as PWSTR,
        core::ptr::addr_of!(LegacyBIOS) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    if MiniPortStatus != 0 {
        if (*FdoExtension).Flags & USBPORT_FLAG_INT_CONNECTED != 0 {
            IoDisconnectInterrupt((*FdoExtension).InterruptObject);
            (*FdoExtension).Flags &= !USBPORT_FLAG_INT_CONNECTED;
        }

        if !(*FdoExtension).MiniPortCommonBuffer.is_null() {
            USBPORT_FreeCommonBuffer(FdoDevice, (*FdoExtension).MiniPortCommonBuffer);
            (*FdoExtension).MiniPortCommonBuffer = core::ptr::null_mut();
        }

        // The C returns Status here, which is still STATUS_SUCCESS, so PnP
        // treats a controller whose miniport refused to start as started, and
        // usbhub then waits forever for a root hub that never comes up. Fail
        // the START instead.
        DbgPrint(
            b"[USBPORT] FDO start: miniport StartController failed (%x)\n\0".as_ptr() as PCSTR,
            MiniPortStatus,
        );
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return STATUS_UNSUCCESSFUL;
    } else {
        (*FdoExtension).MiniPortFlags |= USBPORT_MPFLAG_INTERRUPTS_ENABLED;
        USBPORT_MiniportInterrupts(FdoDevice, 1);
    }

    (*FdoExtension).TimerValue = 500;
    USBPORT_StartTimer((FdoDevice as PVOID) as _, 500);

    Status = USBPORT_RegisterDeviceInterface(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        FdoDevice,
        core::ptr::addr_of!(GUID_DEVINTERFACE_USB_HOST_CONTROLLER),
        1,
    );

    if !NT_SUCCESS(Status) {
        // The timer and the interrupt both point into this FDO, which PnP
        // deletes once the START fails.
        ntkrnl_sys::ddk::KeCancelTimer(core::ptr::addr_of_mut!((*FdoExtension).TimerObject) as _);
        USBPORT_MiniportInterrupts(FdoDevice, 0);
        (*FdoExtension).MiniPortFlags &= !USBPORT_MPFLAG_INTERRUPTS_ENABLED;
        IoDisconnectInterrupt((*FdoExtension).InterruptObject);
        (*FdoExtension).Flags &= !USBPORT_FLAG_INT_CONNECTED;
        USBPORT_StopWorkerThread(FdoDevice);
        USBPORT_StopDevice(FdoDevice);
        return Status;
    }

    USBPORT_CreateLegacySymbolicLink(FdoDevice);

    (*FdoExtension).Flags |= USBPORT_FLAG_HC_STARTED;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ParseResources(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    UsbPortResources: PUSBPORT_RESOURCES,
) -> NTSTATUS {
    const CmResourceTypePort: u8 = 1;
    const CmResourceTypeInterrupt: u8 = 2;
    const CmResourceTypeMemory: u8 = 3;
    const CM_RESOURCE_PORT_IO: u16 = 0x0001;
    const CM_RESOURCE_INTERRUPT_LATCHED: u16 = 0x0001;
    const CM_RESOURCE_INTERRUPT_MESSAGE: u16 = 0x0002;
    const USBPORT_RESOURCES_PORT: u32 = 1;
    const USBPORT_RESOURCES_MEMORY: u32 = 0x00000004; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const USBPORT_RESOURCES_INTERRUPT: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const STATUS_NONE_MAPPED: NTSTATUS = 0xC0000073u32 as NTSTATUS;

    let AllocatedResourcesTranslated: PCM_RESOURCE_LIST;
    let ResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut PortDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut MemoryDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut InterruptDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let mut ix: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    AllocatedResourcesTranslated =
        (*IoStack).Parameters.StartDevice.AllocatedResourcesTranslated;

    if !AllocatedResourcesTranslated.is_null() {
        RtlZeroMemory(
            UsbPortResources as *mut core::ffi::c_void,
            (core::mem::size_of::<USBPORT_RESOURCES>() as ULONG as usize),
        );

        ResourceList = core::ptr::addr_of_mut!(
            (*(*AllocatedResourcesTranslated).List.as_mut_ptr().offset(0)).PartialResourceList
        );

        PartialDescriptor =
            core::ptr::addr_of_mut!(*(*ResourceList).PartialDescriptors.as_mut_ptr().offset(0));

        ix = 0;
        while ix < (*ResourceList).Count {
            if (*PartialDescriptor).Type == CmResourceTypePort {
                if PortDescriptor.is_null() {
                    PortDescriptor = PartialDescriptor;
                }
            } else if (*PartialDescriptor).Type == CmResourceTypeInterrupt {
                // A message, when pci.sys assigned any (msi.rs asked for them), wins
                // over the line: the device raises messages once they are enabled.
                let message = (*PartialDescriptor).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0;
                if InterruptDescriptor.is_null()
                    || (message && (*InterruptDescriptor).Flags & CM_RESOURCE_INTERRUPT_MESSAGE == 0)
                {
                    InterruptDescriptor = PartialDescriptor;
                }
            } else if (*PartialDescriptor).Type == CmResourceTypeMemory {
                if MemoryDescriptor.is_null() {
                    MemoryDescriptor = PartialDescriptor;
                }
            }

            PartialDescriptor = PartialDescriptor.offset(1);

            ix += 1;
        }

        if !PortDescriptor.is_null() {
            if ((*PortDescriptor).Flags & CM_RESOURCE_PORT_IO) != 0 {
                (*UsbPortResources).ResourceBase =
                    (*PortDescriptor).u.Port.Start.QuadPart as ULONG_PTR as PVOID;
            } else {
                (*UsbPortResources).ResourceBase = MmMapIoSpace(
                    (*PortDescriptor).u.Port.Start,
                    ((*PortDescriptor).u.Port.Length as _),
                    0,
                );
            }

            (*UsbPortResources).IoSpaceLength = (*PortDescriptor).u.Port.Length;

            if !(*UsbPortResources).ResourceBase.is_null() {
                (*UsbPortResources).ResourcesTypes |= USBPORT_RESOURCES_PORT;
            } else {
                Status = STATUS_NONE_MAPPED;
            }
        }

        if !MemoryDescriptor.is_null() && NT_SUCCESS(Status) {
            (*UsbPortResources).IoSpaceLength = (*MemoryDescriptor).u.Memory.Length;

            (*UsbPortResources).ResourceBase = MmMapIoSpace(
                (*MemoryDescriptor).u.Memory.Start,
                ((*MemoryDescriptor).u.Memory.Length as _),
                0,
            );

            if !(*UsbPortResources).ResourceBase.is_null() {
                (*UsbPortResources).ResourcesTypes |= USBPORT_RESOURCES_MEMORY;
            } else {
                Status = STATUS_NONE_MAPPED;
            }
        }

        if !InterruptDescriptor.is_null() && NT_SUCCESS(Status) {
            (*UsbPortResources).ResourcesTypes |= USBPORT_RESOURCES_INTERRUPT;

            (*UsbPortResources).InterruptVector = (*InterruptDescriptor).u.Interrupt.Vector;
            (*UsbPortResources).InterruptLevel = ((*InterruptDescriptor).u.Interrupt.Level as u8);
            (*UsbPortResources).InterruptAffinity = (*InterruptDescriptor).u.Interrupt.Affinity;

            (*UsbPortResources).ShareVector =
                (((*InterruptDescriptor).ShareDisposition == (CmResourceShareShared as u8)) as u32 as u8);

            // A test of the flag, not of the whole field: the C compares Flags with
            // CM_RESOURCE_INTERRUPT_LATCHED, which reads a latched message (LATCHED |
            // MESSAGE) as level-sensitive.
            (*UsbPortResources).InterruptMode =
                ((*InterruptDescriptor).Flags & CM_RESOURCE_INTERRUPT_LATCHED != 0) as u32;
            if (*InterruptDescriptor).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0 {
                ntkrnl_sys::driver_ddk::DbgPrint(
                    b"[USBPORT] interrupt: message vector %u at IRQL %u\n\0".as_ptr() as _,
                    (*InterruptDescriptor).u.Interrupt.Vector,
                    (*InterruptDescriptor).u.Interrupt.Level,
                );
            } else {
                ntkrnl_sys::driver_ddk::DbgPrint(
                    b"[USBPORT] interrupt: line vector %u at IRQL %u, %s\n\0".as_ptr() as _,
                    (*InterruptDescriptor).u.Interrupt.Vector,
                    (*InterruptDescriptor).u.Interrupt.Level,
                    if (*UsbPortResources).InterruptMode != 0 { b"edge\0".as_ptr() } else { b"level\0".as_ptr() },
                );
            }
        }
    } else {
        Status = STATUS_NONE_MAPPED;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CreatePdo(
    FdoDevice: PDEVICE_OBJECT,
    RootHubPdo: *mut PDEVICE_OBJECT,
) -> NTSTATUS {

    const FILE_DEVICE_BUS_EXTENDER: u32 = 0x0000002A;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut DeviceNumber: ULONG = 0;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut CharDeviceName: [u16; 64] = [0; 64];
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    loop {
        RtlStringCbPrintfW(
            CharDeviceName.as_mut_ptr(),
            core::mem::size_of::<[u16; 64]>(),
            wstr!("\\Device\\USBPDO-%d\0"),
            DeviceNumber,
        );

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(DeviceName),
            CharDeviceName.as_ptr() as PWSTR,
        );

        Status = IoCreateDevice(
            (*(*FdoExtension).MiniPortInterface).DriverObject,
            core::mem::size_of::<USBPORT_RHDEVICE_EXTENSION>() as u32,
            core::ptr::addr_of_mut!(DeviceName),
            FILE_DEVICE_BUS_EXTENDER,
            0,
            0,
            core::ptr::addr_of_mut!(DeviceObject),
        );

        DeviceNumber += 1;

        if Status != STATUS_OBJECT_NAME_COLLISION {
            break;
        }
    }

    if !NT_SUCCESS(Status) {
        *RootHubPdo = core::ptr::null_mut();
        return Status;
    }

    if !DeviceObject.is_null() {
        PdoExtension = (*DeviceObject).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

        RtlZeroMemory(
            PdoExtension as *mut _,
            core::mem::size_of::<USBPORT_RHDEVICE_EXTENSION>(),
        );

        (*PdoExtension).CommonExtension.SelfDevice = DeviceObject;
        (*PdoExtension).CommonExtension.IsPDO = 1;

        (*PdoExtension).FdoDevice = FdoDevice;
        (*PdoExtension).PdoNameNumber = DeviceNumber;

        USBPORT_AdjustDeviceCapabilities(FdoDevice, DeviceObject);

        (*DeviceObject).StackSize = (*FdoDevice).StackSize;

        (*DeviceObject).Flags |= DO_POWER_PAGABLE;
        (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    if !NT_SUCCESS(Status) {
        *RootHubPdo = core::ptr::null_mut();
    } else {
        *RootHubPdo = DeviceObject;
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FdoPnP(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    const USBPORT_PNP_STATE_STOPPED: ULONG = 0x00000008; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const USBPORT_PNP_STATE_NOT_INIT: ULONG = 0x00000001;
    const USBPORT_PNP_STATE_STARTED: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_PNP_STATE_FAILED: ULONG = 0x00000004; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h
    const USB_MINIPORT_FLAGS_USB2: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const USB_PORT_TAG: ULONG = 0x70627375;
    const USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x3; verified vs usbport.h/usbmport.h

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION =
        (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    let FdoCommonExtension: PUSBPORT_COMMON_DEVICE_EXTENSION =
        core::ptr::addr_of_mut!((*FdoExtension).CommonExtension);
    let UsbPortResources: PUSBPORT_RESOURCES =
        core::ptr::addr_of_mut!((*FdoExtension).UsbPortResources);
    let Packet: PUSBPORT_REGISTRATION_PACKET =
        core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Minor: UCHAR = (*IoStack).MinorFunction;

    let RelationType: DEVICE_RELATION_TYPE =
        (*IoStack).Parameters.QueryDeviceRelations.Type;

    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut DeviceRelations: PDEVICE_RELATIONS;
    let RootHubPdo: PDEVICE_OBJECT;

    match Minor {
        IRP_MN_START_DEVICE => {
            KeInitializeEvent(
                core::ptr::addr_of_mut!(Event),
                _EVENT_TYPE_NotificationEvent,
                0,
            );

            IoCopyCurrentIrpStackLocationToNext(Irp);

            IoSetCompletionRoutine(
                Irp,
                cb!(USBPORT_FdoStartCompletion),
                core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                1,
                1,
                1,
            );

            Status = IoCallDriver((*FdoCommonExtension).LowerDevice, Irp);

            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                    _KWAIT_REASON_Suspended,
                    0i32 as i8,
                    0,
                    core::ptr::null_mut(),
                );

                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }

            'exit: loop {
                if !NT_SUCCESS(Status) {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"[USBPORT] FDO start: lower driver failed 0x%x\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                    break 'exit;
                }

                Status = USBPORT_ParseResources(FdoDevice, Irp, UsbPortResources);

                if !NT_SUCCESS(Status) {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"[USBPORT] FDO start: ParseResources failed 0x%x\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                    (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_STOPPED;
                    break 'exit;
                }

                Status = USBPORT_StartDevice(FdoDevice, UsbPortResources);

                ntkrnl_sys::driver_ddk::DbgPrint(
                    b"[USBPORT] FDO start: USBPORT_StartDevice -> 0x%x\n\0".as_ptr()
                        as *const i8,
                    Status,
                );

                if !NT_SUCCESS(Status) {
                    (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_STOPPED;
                    break 'exit;
                }

                (*FdoCommonExtension).PnpStateFlags &= !USBPORT_PNP_STATE_NOT_INIT;
                (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_STARTED;

                (*FdoCommonExtension).DevicePowerState = PowerDeviceD0;

                if (*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
                    USBPORT_AddUSB2Fdo(FdoDevice);
                } else {
                    USBPORT_AddUSB1Fdo(FdoDevice);
                }

                break 'exit;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return Status;
        }

        IRP_MN_QUERY_REMOVE_DEVICE => {
            if (*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // goto ForwardIrp;
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_REMOVE_DEVICE => {
            (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_FAILED;

            if (*FdoCommonExtension).PnpStateFlags & USBPORT_PNP_STATE_STARTED != 0
                && ((*FdoCommonExtension).PnpStateFlags & USBPORT_PNP_STATE_NOT_INIT) == 0
            {
                (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_NOT_INIT;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*FdoCommonExtension).LowerDevice, Irp);

            IoDetachDevice((*FdoCommonExtension).LowerDevice);

            RootHubPdo = (*FdoExtension).RootHubPdo;

            IoDeleteDevice(FdoDevice);

            if !RootHubPdo.is_null() {
                IoDeleteDevice(RootHubPdo);
            }

            return Status;
        }

        IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // goto ForwardIrp;
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_STOP_DEVICE => {
            if (*FdoCommonExtension).PnpStateFlags & USBPORT_PNP_STATE_STARTED != 0 {
                (*FdoCommonExtension).PnpStateFlags &= !USBPORT_PNP_STATE_STARTED;
                (*FdoCommonExtension).PnpStateFlags |= USBPORT_PNP_STATE_NOT_INIT;
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // goto ForwardIrp;
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // goto ForwardIrp;
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // goto ForwardIrp;
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            'forward: loop {
                if RelationType == BusRelations {
                    DeviceRelations = ExAllocatePoolWithTag(
                        PagedPool,
                        core::mem::size_of::<DEVICE_RELATIONS>(),
                        USB_PORT_TAG,
                    ) as PDEVICE_RELATIONS;

                    if DeviceRelations.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                        return Status;
                    }

                    (*DeviceRelations).Count = 0;
                    (*DeviceRelations).Objects[0] = core::ptr::null_mut();

                    if (*FdoExtension).RootHubPdo.is_null() {
                        Status = USBPORT_CreatePdo(
                            FdoDevice,
                            core::ptr::addr_of_mut!((*FdoExtension).RootHubPdo),
                        );

                        ntkrnl_sys::driver_ddk::DbgPrint(
                            b"[USBPORT] QDR BusRelations: CreatePdo(roothub) -> 0x%x\n\0"
                                .as_ptr() as *const i8,
                            Status,
                        );

                        if !NT_SUCCESS(Status) {
                            ExFreePoolWithTag(
                                DeviceRelations as *mut core::ffi::c_void,
                                USB_PORT_TAG,
                            );
                            break 'forward;
                        }
                    } else {
                        Status = STATUS_SUCCESS;
                    }

                    (*DeviceRelations).Count = 1;
                    (*DeviceRelations).Objects[0] = (*FdoExtension).RootHubPdo;

                    ObReferenceObject((*FdoExtension).RootHubPdo as *mut core::ffi::c_void);
                    (*Irp).IoStatus.Information = (DeviceRelations as _);
                } else {
                    if RelationType == RemovalRelations {
                    }

                    break 'forward;
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                break 'forward;
            }
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_INTERFACE => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_CAPABILITIES => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_RESOURCES => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_DEVICE_TEXT => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_READ_CONFIG => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_WRITE_CONFIG => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_EJECT => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_SET_LOCK => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_ID => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_QUERY_BUS_INFORMATION => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            IoSkipCurrentIrpStackLocation(Irp);
        }

        IRP_MN_SURPRISE_REMOVAL => {
            if ((*FdoCommonExtension).PnpStateFlags & USBPORT_PNP_STATE_FAILED) == 0 {
                USBPORT_InvalidateControllerHandler(
                    FdoDevice,
                    USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE,
                );
            }
            IoSkipCurrentIrpStackLocation(Irp);
        }

        _ => {
            // ForwardIrp:
            IoSkipCurrentIrpStackLocation(Irp);
        }
    }

    return IoCallDriver((*FdoCommonExtension).LowerDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetDeviceHwIds(
    FdoDevice: PDEVICE_OBJECT,
    VendorID: USHORT,
    DeviceID: USHORT,
    RevisionID: USHORT,
) -> PVOID {

    const USB_MINIPORT_FLAGS_USB2: u32 = 0x00000010; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USB_PORT_TAG: u32 = crate::glue::USB_PORT_TAG; // 'usbp' (translation had zeroed this)
    const UNICODE_NULL: u16 = 0;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let Id: PVOID;
    let mut Buffer: [WCHAR; 300] = [0; 300];
    let mut Length: SIZE_T = 0;
    let mut Remaining: usize = core::mem::size_of::<[WCHAR; 300]>();
    let mut EndBuffer: PWCHAR = core::ptr::null_mut();

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBPORT] GetDeviceHwIds vid=%x pid=%x\n\0".as_ptr() as *const i8,
        VendorID as u32,
        DeviceID as u32,
    );

    if ((*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2) != 0 {
        RtlStringCbPrintfExW(
            Buffer.as_mut_ptr(),
            Remaining,
            core::ptr::addr_of_mut!(EndBuffer),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB20&VID%04x&PID%04x&REV%04x\0"),
            VendorID as u32,
            DeviceID as u32,
            RevisionID as u32,
        );

        ntkrnl_sys::driver_ddk::DbgPrint(
            b"[USBPORT] GetDeviceHwIds fmt1 done rem=%u\n\0".as_ptr() as *const i8,
            Remaining as u32,
        );

        EndBuffer = EndBuffer.offset(1);
        Remaining -= core::mem::size_of::<u16>();

        RtlStringCbPrintfExW(
            EndBuffer,
            Remaining,
            core::ptr::addr_of_mut!(EndBuffer),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB20&VID%04x&PID%04x\0"),
            VendorID as u32,
            DeviceID as u32,
        );

        EndBuffer = EndBuffer.offset(1);
        Remaining -= core::mem::size_of::<u16>();

        RtlStringCbPrintfExW(
            EndBuffer,
            Remaining,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB20\0"),
        );
    } else {
        RtlStringCbPrintfExW(
            Buffer.as_mut_ptr(),
            Remaining,
            core::ptr::addr_of_mut!(EndBuffer),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB&VID%04x&PID%04x&REV%04x\0"),
            VendorID as u32,
            DeviceID as u32,
            RevisionID as u32,
        );

        EndBuffer = EndBuffer.offset(1);
        Remaining -= core::mem::size_of::<u16>();

        RtlStringCbPrintfExW(
            EndBuffer,
            Remaining,
            core::ptr::addr_of_mut!(EndBuffer),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB&VID%04x&PID%04x\0"),
            VendorID as u32,
            DeviceID as u32,
        );

        EndBuffer = EndBuffer.offset(1);
        Remaining -= core::mem::size_of::<u16>();

        RtlStringCbPrintfExW(
            EndBuffer,
            Remaining,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Remaining),
            0,
            wstr!("USB\\ROOT_HUB\0"),
        );
    }

    Length = (core::mem::size_of::<[WCHAR; 300]>() - Remaining + 2 * core::mem::size_of::<u16>()) as SIZE_T;

    /* for debug only */
    if false {
        USBPORT_DumpingIDs(Buffer.as_mut_ptr() as PVOID);
    }

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBPORT] GetDeviceHwIds alloc len=%u\n\0".as_ptr() as *const i8,
        Length as u32,
    );

    Id = ExAllocatePoolWithTag(PagedPool, Length, USB_PORT_TAG);

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBPORT] GetDeviceHwIds alloc done id=%p\n\0".as_ptr() as *const i8,
        Id,
    );

    if Id.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::copy(Buffer.as_ptr() as *const u8, Id as *mut u8, Length as usize);

    return Id;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoPnP(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    let PdoCommonExtension: PUSBPORT_COMMON_DEVICE_EXTENSION = core::ptr::addr_of_mut!((*PdoExtension).CommonExtension);

    let FdoDevice: PDEVICE_OBJECT = (*PdoExtension).FdoDevice;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Minor: UCHAR = (*IoStack).MinorFunction;

    let mut Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBPORT] PdoPnP minor=0x%x\n\0".as_ptr() as *const i8,
        Minor as u32,
    );

    let DeviceCapabilities: PDEVICE_CAPABILITIES;
    let BusInformation: PPNP_BUS_INFORMATION;

    match Minor {
        IRP_MN_START_DEVICE => {
            // DPRINT("IRP_MN_START_DEVICE\n");

            Status = USBPORT_RootHubCreateDevice(FdoDevice, PdoDevice);

            if NT_SUCCESS(Status) {
                Status = USBPORT_RegisterDeviceInterface(
                    PdoDevice,
                    PdoDevice,
                    core::ptr::addr_of!(GUID_DEVINTERFACE_USB_HUB),
                    1,
                );

                if NT_SUCCESS(Status) {
                    (*PdoCommonExtension).DevicePowerState = PowerDeviceD0;
                    (*PdoCommonExtension).PnpStateFlags = USBPORT_PNP_STATE_STARTED;
                }
            }
        }

        IRP_MN_QUERY_REMOVE_DEVICE => {
            // DPRINT("USBPORT_PdoPnP: IRP_MN_QUERY_REMOVE_DEVICE\n");
            Status = STATUS_SUCCESS;
        }

        IRP_MN_REMOVE_DEVICE => {
            // DPRINT1("USBPORT_PdoPnP: IRP_MN_REMOVE_DEVICE UNIMPLEMENTED. FIXME. \n");
            //USBPORT_StopRootHub();
            Status = STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_REMOVE_DEVICE => {
            // DPRINT("IRP_MN_CANCEL_REMOVE_DEVICE\n");
            Status = STATUS_SUCCESS;
        }

        IRP_MN_STOP_DEVICE => {
            // DPRINT1("USBPORT_PdoPnP: IRP_MN_STOP_DEVICE UNIMPLEMENTED. FIXME. \n");
            //USBPORT_StopRootHub();
            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_STOP_DEVICE => {
            // DPRINT("IRP_MN_QUERY_STOP_DEVICE\n");
            Status = STATUS_SUCCESS;
        }

        IRP_MN_CANCEL_STOP_DEVICE => {
            // DPRINT("IRP_MN_CANCEL_STOP_DEVICE\n");
            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            let DeviceRelations: PDEVICE_RELATIONS;

            // DPRINT("IRP_MN_QUERY_DEVICE_RELATIONS\n");
            if (*IoStack).Parameters.QueryDeviceRelations.Type != TargetDeviceRelation {
                // break;
            } else {
                DeviceRelations = ExAllocatePoolWithTag(
                    PagedPool,
                    core::mem::size_of::<DEVICE_RELATIONS>(),
                    USB_PORT_TAG,
                ) as PDEVICE_RELATIONS;

                if DeviceRelations.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    (*Irp).IoStatus.Information = 0;
                } else {
                    (*DeviceRelations).Count = 1;
                    (*DeviceRelations).Objects[0] = PdoDevice;

                    ObReferenceObject(PdoDevice as PVOID);

                    Status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = (DeviceRelations as _);
                }
            }
        }

        IRP_MN_QUERY_INTERFACE => {
            // DPRINT("IRP_MN_QUERY_INTERFACE\n");
            Status = USBPORT_PdoQueryInterface(FdoDevice, PdoDevice, Irp);
        }

        IRP_MN_QUERY_CAPABILITIES => {
            // DPRINT("IRP_MN_QUERY_CAPABILITIES\n");

            DeviceCapabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;

            RtlCopyMemory(
                DeviceCapabilities as PVOID,
                core::ptr::addr_of!((*PdoExtension).Capabilities) as *const _ as PVOID,
                core::mem::size_of::<DEVICE_CAPABILITIES>(),
            );

            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_RESOURCES => {
            // DPRINT("USBPORT_PdoPnP: IRP_MN_QUERY_RESOURCES\n");
        }

        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            // DPRINT("IRP_MN_QUERY_RESOURCE_REQUIREMENTS\n");
        }

        IRP_MN_QUERY_DEVICE_TEXT => {
            // DPRINT("IRP_MN_QUERY_DEVICE_TEXT\n");
        }

        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            // DPRINT("IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
        }

        IRP_MN_READ_CONFIG => {
            // DPRINT("IRP_MN_READ_CONFIG\n");
            // ASSERT(FALSE);
        }

        IRP_MN_WRITE_CONFIG => {
            // DPRINT("IRP_MN_WRITE_CONFIG\n");
            // ASSERT(FALSE);
        }

        IRP_MN_EJECT => {
            // DPRINT("IRP_MN_EJECT\n");
            // ASSERT(FALSE);
        }

        IRP_MN_SET_LOCK => {
            // DPRINT("IRP_MN_SET_LOCK\n");
            // ASSERT(FALSE);
        }

        IRP_MN_QUERY_ID => {
            let IdType: ULONG;
            let Length: LONG;
            let mut Buffer: [WCHAR; 64] = [0; 64];
            let Id: PVOID;

            Status = STATUS_SUCCESS;
            IdType = (*IoStack).Parameters.QueryId.IdType;

            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[USBPORT] PdoPnP QUERY_ID IdType=%u\n\0".as_ptr() as *const i8,
                IdType as u32,
            );

            if IdType == BusQueryDeviceID {
                let Packet: PUSBPORT_REGISTRATION_PACKET;
                Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

                if (*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
                    RtlStringCbPrintfW(
                        Buffer.as_mut_ptr(),
                        core::mem::size_of::<[WCHAR; 64]>(),
                        wstr!("USB\\ROOT_HUB20\0"),
                    );
                } else {
                    RtlStringCbPrintfW(
                        Buffer.as_mut_ptr(),
                        core::mem::size_of::<[WCHAR; 64]>(),
                        wstr!("USB\\ROOT_HUB\0"),
                    );
                }

                Length = (crate::glue::wcslen(Buffer.as_ptr()) + 1) as LONG;

                Id = ExAllocatePoolWithTag(
                    PagedPool,
                    (Length as usize) * core::mem::size_of::<WCHAR>(),
                    USB_PORT_TAG,
                );

                if !Id.is_null() {
                    RtlZeroMemory(Id, (Length as usize) * core::mem::size_of::<WCHAR>());
                    RtlStringCbCopyW(
                        Id as PWSTR,
                        (Length as usize) * core::mem::size_of::<WCHAR>(),
                        Buffer.as_ptr(),
                    );

                    // DPRINT("BusQueryDeviceID - %S, TotalLength - %hu\n", Id, Length);
                }

                (*Irp).IoStatus.Information = (Id as _);
            } else if IdType == BusQueryHardwareIDs {
                Id = USBPORT_GetDeviceHwIds(
                    FdoDevice,
                    (*FdoExtension).VendorID,
                    (*FdoExtension).DeviceID,
                    ((*FdoExtension).RevisionID as u16),
                );

                (*Irp).IoStatus.Information = (Id as _);
            } else if IdType == BusQueryCompatibleIDs || IdType == BusQueryInstanceID {
                (*Irp).IoStatus.Information = 0;
            } else {
                /* IdType == BusQueryDeviceSerialNumber */
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            // DPRINT("IRP_MN_QUERY_PNP_DEVICE_STATE\n");
            Status = STATUS_SUCCESS;
        }

        IRP_MN_QUERY_BUS_INFORMATION => {
            // DPRINT("IRP_MN_QUERY_BUS_INFORMATION\n");

            /* Allocate buffer for bus information */
            BusInformation = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<PNP_BUS_INFORMATION>(),
                USB_PORT_TAG,
            ) as PPNP_BUS_INFORMATION;

            if BusInformation.is_null() {
                /* No memory */
                Status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                RtlZeroMemory(BusInformation as PVOID, core::mem::size_of::<PNP_BUS_INFORMATION>());

                /* Copy BUS GUID */
                core::ptr::copy(
                    core::ptr::addr_of!(GUID_BUS_TYPE_USB) as *const u8,
                    core::ptr::addr_of_mut!((*BusInformation).BusTypeGuid) as *mut u8,
                    core::mem::size_of::<GUID>(),
                );

                /* Set bus type */
                (*BusInformation).LegacyBusType = PNPBus;
                (*BusInformation).BusNumber = 0;

                Status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = (BusInformation as _);
            }
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            // DPRINT("IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
        }

        IRP_MN_SURPRISE_REMOVAL => {
            // DPRINT("USBPORT_PdoPnP: IRP_MN_SURPRISE_REMOVAL\n");
            Status = STATUS_SUCCESS;
        }

        _ => {
            // DPRINT("unknown IRP_MN_???\n");
        }
    }

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[USBPORT] PdoPnP done minor=0x%x -> 0x%x info=%p\n\0".as_ptr() as *const i8,
        Minor as u32,
        Status,
        (*Irp).IoStatus.Information,
    );
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort plug and play functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_CORE
// #include "usbdebug.h"
//
// IO_COMPLETION_ROUTINE USBPORT_FdoStartCompletion;
//
// NTSTATUS
// NTAPI
// USBPORT_FdoStartCompletion(IN PDEVICE_OBJECT DeviceObject,
//                            IN PIRP Irp,
//                            IN PVOID Context)
// {
//     KeSetEvent((PKEVENT)Context, EVENT_INCREMENT, FALSE);
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_RegisterDeviceInterface(IN PDEVICE_OBJECT PdoDevice,
//                                 IN PDEVICE_OBJECT DeviceObject,
//                                 IN CONST GUID *InterfaceClassGuid,
//                                 IN BOOLEAN Enable)
// {
//     PUSBPORT_RHDEVICE_EXTENSION DeviceExtension;
//     PUNICODE_STRING SymbolicLinkName;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_RegisterDeviceInterface: Enable - %x\n", Enable);
//
//     DeviceExtension = DeviceObject->DeviceExtension;
//     SymbolicLinkName = &DeviceExtension->CommonExtension.SymbolicLinkName;
//
//     if (Enable)
//     {
//         Status = IoRegisterDeviceInterface(PdoDevice,
//                                            InterfaceClassGuid,
//                                            NULL,
//                                            SymbolicLinkName);
//
//         if (NT_SUCCESS(Status))
//         {
//             DeviceExtension->CommonExtension.IsInterfaceEnabled = 1;
//
//             Status = USBPORT_SetRegistryKeyValue(PdoDevice,
//                                                  FALSE,
//                                                  REG_SZ,
//                                                  L"SymbolicName",
//                                                  SymbolicLinkName->Buffer,
//                                                  SymbolicLinkName->Length);
//
//             if (NT_SUCCESS(Status))
//             {
//                 DPRINT("USBPORT_RegisterDeviceInterface: LinkName  - %wZ\n",
//                        &DeviceExtension->CommonExtension.SymbolicLinkName);
//
//                 Status = IoSetDeviceInterfaceState(SymbolicLinkName, TRUE);
//             }
//         }
//     }
//     else
//     {
//         /* Disable device interface */
//         Status = IoSetDeviceInterfaceState(SymbolicLinkName, FALSE);
//
//         if (NT_SUCCESS(Status))
//         {
//             RtlFreeUnicodeString(SymbolicLinkName);
//             DeviceExtension->CommonExtension.IsInterfaceEnabled = 0; // Disabled interface
//         }
//     }
//
//     return Status;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_IsSelectiveSuspendEnabled(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG Disabled = 0;
//
//     DPRINT("USBPORT_IsSelectiveSuspendEnabled: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     USBPORT_GetRegistryKeyValueFullInfo(FdoDevice,
//                                         FdoExtension->CommonExtension.LowerPdoDevice,
//                                         TRUE,
//                                         L"HcDisableSelectiveSuspend",
//                                         sizeof(L"HcDisableSelectiveSuspend"),
//                                         &Disabled,
//                                         sizeof(Disabled));
//
//     return (Disabled == 0);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetConfigValue(IN PWSTR ValueName,
//                        IN ULONG ValueType,
//                        IN PVOID ValueData,
//                        IN ULONG ValueLength,
//                        IN PVOID Context,
//                        IN PVOID EntryContext)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("USBPORT_GetConfigValue \n");
//
//     if (ValueType == REG_DWORD)
//     {
//         *(PULONG)EntryContext = *(PULONG)ValueData;
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetDefaultBIOSx(IN PDEVICE_OBJECT FdoDevice,
//                          IN PULONG UsbBIOSx,
//                          IN PULONG DisableSelectiveSuspend,
//                          IN PULONG DisableCcDetect,
//                          IN PULONG IdleEpSupport,
//                          IN PULONG IdleEpSupportEx,
//                          IN PULONG SoftRetry)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[7];
//
//     DPRINT("USBPORT_GetDefaultBIOS_X: ... \n");
//
//     RtlZeroMemory(QueryTable, sizeof(QueryTable));
//
//     *UsbBIOSx = 2;
//
//     QueryTable[0].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[0].Flags = 0;
//     QueryTable[0].Name = L"UsbBIOSx";
//     QueryTable[0].EntryContext = UsbBIOSx;
//     QueryTable[0].DefaultType = REG_DWORD;
//     QueryTable[0].DefaultData = UsbBIOSx;
//     QueryTable[0].DefaultLength = sizeof(ULONG);
//
//     QueryTable[1].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[1].Flags = 0;
//     QueryTable[1].Name = L"DisableSelectiveSuspend";
//     QueryTable[1].EntryContext = DisableSelectiveSuspend;
//     QueryTable[1].DefaultType = REG_DWORD;
//     QueryTable[1].DefaultData = DisableSelectiveSuspend;
//     QueryTable[1].DefaultLength = sizeof(ULONG);
//
//     QueryTable[2].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[2].Flags = 0;
//     QueryTable[2].Name = L"DisableCcDetect";
//     QueryTable[2].EntryContext = DisableCcDetect;
//     QueryTable[2].DefaultType = REG_DWORD;
//     QueryTable[2].DefaultData = DisableCcDetect;
//     QueryTable[2].DefaultLength = sizeof(ULONG);
//
//     QueryTable[3].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[3].Flags = 0;
//     QueryTable[3].Name = L"EnIdleEndpointSupport";
//     QueryTable[3].EntryContext = IdleEpSupport;
//     QueryTable[3].DefaultType = REG_DWORD;
//     QueryTable[3].DefaultData = IdleEpSupport;
//     QueryTable[3].DefaultLength = sizeof(ULONG);
//
//     QueryTable[4].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[4].Flags = 0;
//     QueryTable[4].Name = L"EnIdleEndpointSupportEx";
//     QueryTable[4].EntryContext = IdleEpSupportEx;
//     QueryTable[4].DefaultType = REG_DWORD;
//     QueryTable[4].DefaultData = IdleEpSupportEx;
//     QueryTable[4].DefaultLength = sizeof(ULONG);
//
//     QueryTable[5].QueryRoutine = USBPORT_GetConfigValue;
//     QueryTable[5].Flags = 0;
//     QueryTable[5].Name = L"EnSoftRetry";
//     QueryTable[5].EntryContext = SoftRetry;
//     QueryTable[5].DefaultType = REG_DWORD;
//     QueryTable[5].DefaultData = SoftRetry;
//     QueryTable[5].DefaultLength = sizeof(ULONG);
//
//     return RtlQueryRegistryValues(RTL_REGISTRY_SERVICES,
//                                   L"usb",
//                                   QueryTable,
//                                   NULL,
//                                   NULL);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_IsCompanionController(IN PDEVICE_OBJECT DeviceObject,
//                               IN BOOLEAN *IsCompanion)
// {
//     PDEVICE_OBJECT HighestDevice;
//     PIRP Irp;
//     KEVENT Event;
//     PIO_STACK_LOCATION IoStack;
//     PCI_DEVICE_PRESENT_INTERFACE PciInterface = {0};
//     PCI_DEVICE_PRESENCE_PARAMETERS Parameters   = {0};
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//     BOOLEAN IsPresent;
//
//     DPRINT("USBPORT_IsCompanionController: ... \n");
//
//     *IsCompanion = FALSE;
//
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//
//     HighestDevice = IoGetAttachedDeviceReference(DeviceObject);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        HighestDevice,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatusBlock);
//
//     if (!Irp)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         ObDereferenceObject(HighestDevice);
//         return Status;
//     }
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = 0;
//
//     IoStack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//
//     IoStack->Parameters.QueryInterface.InterfaceType = &GUID_PCI_DEVICE_PRESENT_INTERFACE;
//     IoStack->Parameters.QueryInterface.Size = sizeof(PCI_DEVICE_PRESENT_INTERFACE);
//     IoStack->Parameters.QueryInterface.Version = 1;
//     IoStack->Parameters.QueryInterface.Interface = (PINTERFACE)&PciInterface;
//     IoStack->Parameters.QueryInterface.InterfaceSpecificData = 0;
//
//     Status = IoCallDriver(HighestDevice, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBPORT_IsCompanionController: query interface failed\\n");
//         ObDereferenceObject(HighestDevice);
//         return Status;
//     }
//
//     DPRINT("USBPORT_IsCompanionController: query interface succeeded\n");
//
//     if (PciInterface.Size < sizeof(PCI_DEVICE_PRESENT_INTERFACE))
//     {
//         DPRINT1("USBPORT_IsCompanionController: old version\n");
//         ObDereferenceObject(HighestDevice);
//         return Status;
//     }
//
//     Parameters.Size = sizeof(PCI_DEVICE_PRESENT_INTERFACE);
//
//     Parameters.BaseClass = PCI_CLASS_SERIAL_BUS_CTLR;
//     Parameters.SubClass = PCI_SUBCLASS_SB_USB;
//     Parameters.ProgIf = PCI_INTERFACE_USB_ID_EHCI;
//
//     Parameters.Flags = PCI_USE_LOCAL_BUS |
//                        PCI_USE_LOCAL_DEVICE |
//                        PCI_USE_CLASS_SUBCLASS |
//                        PCI_USE_PROGIF;
//
//     IsPresent = (PciInterface.IsDevicePresentEx)(PciInterface.Context,
//                                                  &Parameters);
//
//     if (IsPresent)
//     {
//         DPRINT("USBPORT_IsCompanionController: Present EHCI controller for FDO - %p\n",
//                DeviceObject);
//     }
//     else
//     {
//         DPRINT("USBPORT_IsCompanionController: No EHCI controller for FDO - %p\n",
//                DeviceObject);
//     }
//
//     *IsCompanion = IsPresent;
//
//     (PciInterface.InterfaceDereference)(PciInterface.Context);
//
//     ObDereferenceObject(HighestDevice);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_QueryPciBusInterface(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PBUS_INTERFACE_STANDARD BusInterface;
//     PIO_STACK_LOCATION IoStack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_OBJECT HighestDevice;
//     KEVENT Event;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_QueryPciBusInterface: ...  \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     BusInterface = &FdoExtension->BusInterface;
//
//     RtlZeroMemory(BusInterface, sizeof(BUS_INTERFACE_STANDARD));
//     KeInitializeEvent(&Event, SynchronizationEvent, FALSE);
//     HighestDevice = IoGetAttachedDeviceReference(FdoDevice);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        HighestDevice,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatusBlock);
//
//     if (Irp)
//     {
//         IoStack = IoGetNextIrpStackLocation(Irp);
//
//         Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//         Irp->IoStatus.Information = 0;
//
//         IoStack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//
//         IoStack->Parameters.QueryInterface.InterfaceType = &GUID_BUS_INTERFACE_STANDARD;
//         IoStack->Parameters.QueryInterface.Size = sizeof(BUS_INTERFACE_STANDARD);
//         IoStack->Parameters.QueryInterface.Version = 1;
//         IoStack->Parameters.QueryInterface.Interface = (PINTERFACE)BusInterface;
//         IoStack->Parameters.QueryInterface.InterfaceSpecificData = 0;
//
//         Status = IoCallDriver(HighestDevice, Irp);
//
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//     }
//     else
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     ObDereferenceObject(HighestDevice);
//
//     DPRINT("USBPORT_QueryPciBusInterface: return Status - %x\n", Status);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_QueryCapabilities(IN PDEVICE_OBJECT FdoDevice,
//                           IN PDEVICE_CAPABILITIES Capabilities)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtention;
//     PIRP Irp;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//
//     DPRINT("USBPORT_QueryCapabilities: ... \n");
//
//     FdoExtention = FdoDevice->DeviceExtension;
//
//     RtlZeroMemory(Capabilities, sizeof(DEVICE_CAPABILITIES));
//
//     Capabilities->Size     = sizeof(DEVICE_CAPABILITIES);
//     Capabilities->Version  = 1;
//     Capabilities->Address  = MAXULONG;
//     Capabilities->UINumber = MAXULONG;
//
//     Irp = IoAllocateIrp(FdoExtention->CommonExtension.LowerDevice->StackSize,
//                         FALSE);
//
//     if (!Irp)
//     {
//         DPRINT1("USBPORT_QueryCapabilities: No resources - IoAllocateIrp!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->MajorFunction = IRP_MJ_PNP;
//     IoStack->MinorFunction = IRP_MN_QUERY_CAPABILITIES;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     IoSetCompletionRoutine(Irp,
//                           USBPORT_FdoStartCompletion,
//                           &Event,
//                           TRUE,
//                           TRUE,
//                           TRUE);
//
//     IoStack->Parameters.DeviceCapabilities.Capabilities = Capabilities;
//
//     Status = IoCallDriver(FdoExtention->CommonExtension.LowerDevice, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Suspended, KernelMode, FALSE, NULL);
//         Status = Irp->IoStatus.Status;
//     }
//
//     if (NT_SUCCESS(Status) && Capabilities)
//     {
//         USBPORT_DumpingCapabilities(Capabilities);
//     }
//
//     IoFreeIrp(Irp);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_CreateLegacySymbolicLink(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     WCHAR CharName[255] = {0};
//     WCHAR CharDosName[255] = {0};
//     UNICODE_STRING DeviceName;
//     NTSTATUS Status;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     RtlStringCbPrintfW(CharName,
//                        sizeof(CharName),
//                        L"\\Device\\USBFDO-%d",
//                        FdoExtension->FdoNameNumber);
//
//     RtlInitUnicodeString(&DeviceName, CharName);
//
//     RtlStringCbPrintfW(CharDosName,
//                        sizeof(CharDosName),
//                        L"\\DosDevices\\HCD%d",
//                        FdoExtension->FdoNameNumber);
//
//     RtlInitUnicodeString(&FdoExtension->DosDeviceSymbolicName, CharDosName);
//
//     DPRINT("USBPORT_CreateLegacySymbolicLink: DeviceName - %wZ, DosSymbolicName - %wZ\n",
//            &DeviceName,
//            &FdoExtension->DosDeviceSymbolicName);
//
//     Status = IoCreateSymbolicLink(&FdoExtension->DosDeviceSymbolicName,
//                                   &DeviceName);
//
//     if (NT_SUCCESS(Status))
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_DOS_SYMBOLIC_NAME;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_StopDevice(IN PDEVICE_OBJECT FdoDevice)
// {
//     DPRINT1("USBPORT_StopDevice: UNIMPLEMENTED. FIXME\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_StartDevice(IN PDEVICE_OBJECT FdoDevice,
//                     IN PUSBPORT_RESOURCES UsbPortResources)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     NTSTATUS Status;
//     PCI_COMMON_CONFIG PciConfig;
//     ULONG BytesRead;
//     DEVICE_DESCRIPTION DeviceDescription;
//     PDMA_ADAPTER DmaAdapter = NULL;
//     ULONG MiniPortStatus;
//     PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer;
//     ULONG ResultLength;
//     ULONG DisableSelectiveSuspend = 0;
//     ULONG DisableCcDetect = 0;
//     ULONG IdleEpSupport = 0;
//     ULONG IdleEpSupportEx = 0;
//     ULONG SoftRetry = 0;
//     ULONG Limit2GB = 0;
//     ULONG TotalBusBandwidth = 0;
//     BOOLEAN IsCompanion = FALSE;
//     ULONG LegacyBIOS;
//     ULONG MiniportFlags;
//     ULONG ix;
//
//     DPRINT("USBPORT_StartDevice: FdoDevice - %p, UsbPortResources - %p\n",
//            FdoDevice,
//            UsbPortResources);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     Status = USBPORT_QueryPciBusInterface(FdoDevice);
//     if (!NT_SUCCESS(Status))
//         goto ExitWithError;
//
//     BytesRead = (*FdoExtension->BusInterface.GetBusData)(FdoExtension->BusInterface.Context,
//                                                          PCI_WHICHSPACE_CONFIG,
//                                                          &PciConfig,
//                                                          0,
//                                                          PCI_COMMON_HDR_LENGTH);
//
//     if (BytesRead != PCI_COMMON_HDR_LENGTH)
//     {
//         DPRINT1("USBPORT_StartDevice: Failed to get pci config information!\n");
//         goto ExitWithError;
//     }
//
//     FdoExtension->VendorID = PciConfig.VendorID;
//     FdoExtension->DeviceID = PciConfig.DeviceID;
//     FdoExtension->RevisionID = PciConfig.RevisionID;
//     FdoExtension->ProgIf = PciConfig.ProgIf;
//     FdoExtension->SubClass = PciConfig.SubClass;
//     FdoExtension->BaseClass = PciConfig.BaseClass;
//
//     RtlZeroMemory(&DeviceDescription, sizeof(DeviceDescription));
//
//     DeviceDescription.Version = DEVICE_DESCRIPTION_VERSION;
//     DeviceDescription.Master = TRUE;
//     DeviceDescription.ScatterGather = TRUE;
//     DeviceDescription.Dma32BitAddresses = TRUE;
//     DeviceDescription.InterfaceType = PCIBus;
//     DeviceDescription.DmaWidth = Width32Bits;
//     DeviceDescription.DmaSpeed = Compatible;
//     DeviceDescription.MaximumLength = MAXULONG;
//
//     DmaAdapter = IoGetDmaAdapter(FdoExtension->CommonExtension.LowerPdoDevice,
//                                  &DeviceDescription,
//                                  &FdoExtension->NumberMapRegs);
//
//     FdoExtension->DmaAdapter = DmaAdapter;
//
//     if (!DmaAdapter)
//     {
//         DPRINT1("USBPORT_StartDevice: Failed to get DmaAdapter!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto ExitWithError;
//     }
//
//     Status = USBPORT_CreateWorkerThread(FdoDevice);
//     if (!NT_SUCCESS(Status))
//         goto ExitWithError;
//
//     Status = USBPORT_QueryCapabilities(FdoDevice, &FdoExtension->Capabilities);
//     if (!NT_SUCCESS(Status))
//         goto ExitWithError;
//
//     FdoExtension->PciDeviceNumber = FdoExtension->Capabilities.Address >> 16;
//     FdoExtension->PciFunctionNumber = FdoExtension->Capabilities.Address & 0xFFFF;
//
//     Status = IoGetDeviceProperty(FdoExtension->CommonExtension.LowerPdoDevice,
//                                  DevicePropertyBusNumber,
//                                  sizeof(ULONG),
//                                  &FdoExtension->BusNumber,
//                                  &ResultLength);
//
//     if (!NT_SUCCESS(Status))
//         goto ExitWithError;
//
//     KeInitializeSpinLock(&FdoExtension->EndpointListSpinLock);
//     KeInitializeSpinLock(&FdoExtension->EpStateChangeSpinLock);
//     KeInitializeSpinLock(&FdoExtension->EndpointClosedSpinLock);
//     KeInitializeSpinLock(&FdoExtension->DeviceHandleSpinLock);
//     KeInitializeSpinLock(&FdoExtension->IdleIoCsqSpinLock);
//     KeInitializeSpinLock(&FdoExtension->BadRequestIoCsqSpinLock);
//     KeInitializeSpinLock(&FdoExtension->MapTransferSpinLock);
//     KeInitializeSpinLock(&FdoExtension->FlushTransferSpinLock);
//     KeInitializeSpinLock(&FdoExtension->FlushPendingTransferSpinLock);
//     KeInitializeSpinLock(&FdoExtension->DoneTransferSpinLock);
//     KeInitializeSpinLock(&FdoExtension->WorkerThreadEventSpinLock);
//     KeInitializeSpinLock(&FdoExtension->MiniportSpinLock);
//     KeInitializeSpinLock(&FdoExtension->TimerFlagsSpinLock);
//     KeInitializeSpinLock(&FdoExtension->PowerWakeSpinLock);
//     KeInitializeSpinLock(&FdoExtension->SetPowerD0SpinLock);
//     KeInitializeSpinLock(&FdoExtension->RootHubCallbackSpinLock);
//     KeInitializeSpinLock(&FdoExtension->TtSpinLock);
//
//     KeInitializeDpc(&FdoExtension->IsrDpc, USBPORT_IsrDpc, FdoDevice);
//
//     KeInitializeDpc(&FdoExtension->TransferFlushDpc,
//                     USBPORT_TransferFlushDpc,
//                     FdoDevice);
//
//     KeInitializeDpc(&FdoExtension->WorkerRequestDpc,
//                     USBPORT_WorkerRequestDpc,
//                     FdoDevice);
//
//     KeInitializeDpc(&FdoExtension->HcWakeDpc,
//                     USBPORT_HcWakeDpc,
//                     FdoDevice);
//
//     IoCsqInitialize(&FdoExtension->IdleIoCsq,
//                     USBPORT_InsertIdleIrp,
//                     USBPORT_RemoveIdleIrp,
//                     USBPORT_PeekNextIdleIrp,
//                     USBPORT_AcquireIdleLock,
//                     USBPORT_ReleaseIdleLock,
//                     USBPORT_CompleteCanceledIdleIrp);
//
//     IoCsqInitialize(&FdoExtension->BadRequestIoCsq,
//                     USBPORT_InsertBadRequest,
//                     USBPORT_RemoveBadRequest,
//                     USBPORT_PeekNextBadRequest,
//                     USBPORT_AcquireBadRequestLock,
//                     USBPORT_ReleaseBadRequestLock,
//                     USBPORT_CompleteCanceledBadRequest);
//
//     FdoExtension->IsrDpcCounter = -1;
//     FdoExtension->IsrDpcHandlerCounter = -1;
//     FdoExtension->IdleLockCounter = -1;
//     FdoExtension->BadRequestLockCounter = -1;
//     FdoExtension->ChirpRootPortLock = -1;
//
//     FdoExtension->RHInitCallBackLock = 0;
//
//     FdoExtension->UsbAddressBitMap[0] = 1;
//     FdoExtension->UsbAddressBitMap[1] = 0;
//     FdoExtension->UsbAddressBitMap[2] = 0;
//     FdoExtension->UsbAddressBitMap[3] = 0;
//
//     USBPORT_GetDefaultBIOSx(FdoDevice,
//                             &FdoExtension->UsbBIOSx,
//                             &DisableSelectiveSuspend,
//                             &DisableCcDetect,
//                             &IdleEpSupport,
//                             &IdleEpSupportEx,
//                             &SoftRetry);
//
//     if (DisableSelectiveSuspend)
//         FdoExtension->Flags |= USBPORT_FLAG_BIOS_DISABLE_SS;
//
//     if (!DisableSelectiveSuspend &&
//         USBPORT_IsSelectiveSuspendEnabled(FdoDevice))
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
//     }
//
//     MiniportFlags = Packet->MiniPortFlags;
//
//     if (MiniportFlags & USB_MINIPORT_FLAGS_POLLING)
//         FdoExtension->Flags |= USBPORT_FLAG_HC_POLLING;
//
//     if (MiniportFlags & USB_MINIPORT_FLAGS_WAKE_SUPPORT)
//         FdoExtension->Flags |= USBPORT_FLAG_HC_WAKE_SUPPORT;
//
//     if (MiniportFlags & USB_MINIPORT_FLAGS_DISABLE_SS)
//         FdoExtension->Flags = (FdoExtension->Flags & ~USBPORT_FLAG_SELECTIVE_SUSPEND) |
//                               USBPORT_FLAG_BIOS_DISABLE_SS;
//
//     USBPORT_SetRegistryKeyValue(FdoExtension->CommonExtension.LowerPdoDevice,
//                                 TRUE,
//                                 REG_DWORD,
//                                 L"EnIdleEndpointSupport",
//                                 &IdleEpSupport,
//                                 sizeof(IdleEpSupport));
//
//     USBPORT_SetRegistryKeyValue(FdoExtension->CommonExtension.LowerPdoDevice,
//                                 TRUE,
//                                 REG_DWORD,
//                                 L"EnIdleEndpointSupportEx",
//                                 &IdleEpSupportEx,
//                                 sizeof(IdleEpSupportEx));
//
//     USBPORT_SetRegistryKeyValue(FdoExtension->CommonExtension.LowerPdoDevice,
//                                 TRUE,
//                                 REG_DWORD,
//                                 L"EnSoftRetry",
//                                 &SoftRetry,
//                                 sizeof(SoftRetry));
//
//     USBPORT_GetRegistryKeyValueFullInfo(FdoDevice,
//                                         FdoExtension->CommonExtension.LowerPdoDevice,
//                                         TRUE,
//                                         L"CommonBuffer2GBLimit",
//                                         sizeof(L"CommonBuffer2GBLimit"),
//                                         &Limit2GB,
//                                         sizeof(Limit2GB));
//
//     FdoExtension->CommonBufferLimit = (Limit2GB != 0);
//
//     if (FdoExtension->BaseClass == PCI_CLASS_SERIAL_BUS_CTLR &&
//         FdoExtension->SubClass == PCI_SUBCLASS_SB_USB  &&
//         FdoExtension->ProgIf < PCI_INTERFACE_USB_ID_EHCI)
//     {
//         Status = USBPORT_IsCompanionController(FdoDevice, &IsCompanion);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (IsCompanion)
//             {
//                 FdoExtension->Flags |= USBPORT_FLAG_COMPANION_HC;
//             }
//             else
//             {
//                 FdoExtension->Flags &= ~USBPORT_FLAG_COMPANION_HC;
//             }
//         }
//     }
//
//     if (DisableCcDetect)
//     {
//         FdoExtension->Flags &= ~USBPORT_FLAG_COMPANION_HC;
//     }
//
//     TotalBusBandwidth = Packet->MiniPortBusBandwidth;
//     FdoExtension->TotalBusBandwidth = TotalBusBandwidth;
//
//     USBPORT_GetRegistryKeyValueFullInfo(FdoDevice,
//                                         FdoExtension->CommonExtension.LowerPdoDevice,
//                                         TRUE,
//                                         L"TotalBusBandwidth",
//                                         sizeof(L"TotalBusBandwidth"),
//                                         &TotalBusBandwidth,
//                                         sizeof(TotalBusBandwidth));
//
//     if (TotalBusBandwidth != FdoExtension->TotalBusBandwidth)
//     {
//         FdoExtension->TotalBusBandwidth = TotalBusBandwidth;
//     }
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] = FdoExtension->TotalBusBandwidth -
//                                       FdoExtension->TotalBusBandwidth / 10;
//     }
//
//     FdoExtension->ActiveIrpTable = ExAllocatePoolWithTag(NonPagedPool,
//                                                          sizeof(USBPORT_IRP_TABLE),
//                                                          USB_PORT_TAG);
//
//     if (!FdoExtension->ActiveIrpTable)
//     {
//         DPRINT1("USBPORT_StartDevice: Allocate ActiveIrpTable failed!\n");
//         goto ExitWithError;
//     }
//
//     RtlZeroMemory(FdoExtension->ActiveIrpTable, sizeof(USBPORT_IRP_TABLE));
//
//     FdoExtension->PendingIrpTable = ExAllocatePoolWithTag(NonPagedPool,
//                                                           sizeof(USBPORT_IRP_TABLE),
//                                                           USB_PORT_TAG);
//
//     if (!FdoExtension->PendingIrpTable)
//     {
//         DPRINT1("USBPORT_StartDevice: Allocate PendingIrpTable failed!\n");
//         goto ExitWithError;
//     }
//
//     RtlZeroMemory(FdoExtension->PendingIrpTable, sizeof(USBPORT_IRP_TABLE));
//
//     Status = IoConnectInterrupt(&FdoExtension->InterruptObject,
//                                 USBPORT_InterruptService,
//                                 (PVOID)FdoDevice,
//                                 0,
//                                 UsbPortResources->InterruptVector,
//                                 UsbPortResources->InterruptLevel,
//                                 UsbPortResources->InterruptLevel,
//                                 UsbPortResources->InterruptMode,
//                                 UsbPortResources->ShareVector,
//                                 UsbPortResources->InterruptAffinity,
//                                 0);
//
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBPORT_StartDevice: IoConnectInterrupt failed!\n");
//         goto ExitWithError;
//     }
//
//     FdoExtension->Flags &= ~USBPORT_FLAG_INT_CONNECTED;
//
//     if (Packet->MiniPortExtensionSize)
//     {
//         RtlZeroMemory(FdoExtension->MiniPortExt, Packet->MiniPortExtensionSize);
//     }
//
//     if (Packet->MiniPortResourcesSize)
//     {
//         HeaderBuffer = USBPORT_AllocateCommonBuffer(FdoDevice,
//                                                     Packet->MiniPortResourcesSize);
//
//         if (!HeaderBuffer)
//         {
//             DPRINT1("USBPORT_StartDevice: Failed to AllocateCommonBuffer!\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto ExitWithError;
//         }
//
//         UsbPortResources->StartVA = HeaderBuffer->VirtualAddress;
//         UsbPortResources->StartPA = HeaderBuffer->PhysicalAddress;
//
//         FdoExtension->MiniPortCommonBuffer = HeaderBuffer;
//     }
//     else
//     {
//         FdoExtension->MiniPortCommonBuffer = NULL;
//     }
//
//     MiniPortStatus = Packet->StartController(FdoExtension->MiniPortExt,
//                                              UsbPortResources);
//
//     if (UsbPortResources->LegacySupport)
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_LEGACY_SUPPORT;
//         LegacyBIOS = 1;
//     }
//     else
//     {
//         LegacyBIOS = 0;
//     }
//
//     USBPORT_SetRegistryKeyValue(FdoExtension->CommonExtension.LowerPdoDevice,
//                                 FALSE,
//                                 REG_DWORD,
//                                 L"DetectedLegacyBIOS",
//                                 &LegacyBIOS,
//                                 sizeof(LegacyBIOS));
//
//     if (MiniPortStatus)
//     {
//         DPRINT1("USBPORT_StartDevice: Failed to Start MiniPort. MiniPortStatus - %x\n",
//                 MiniPortStatus);
//
//         if (FdoExtension->Flags & USBPORT_FLAG_INT_CONNECTED)
//         {
//             IoDisconnectInterrupt(FdoExtension->InterruptObject);
//             FdoExtension->Flags &= ~USBPORT_FLAG_INT_CONNECTED;
//         }
//
//         if (FdoExtension->MiniPortCommonBuffer)
//         {
//             USBPORT_FreeCommonBuffer(FdoDevice, FdoExtension->MiniPortCommonBuffer);
//             FdoExtension->MiniPortCommonBuffer = NULL;
//         }
//
//         goto ExitWithError;
//     }
//     else
//     {
//         FdoExtension->MiniPortFlags |= USBPORT_MPFLAG_INTERRUPTS_ENABLED;
//         USBPORT_MiniportInterrupts(FdoDevice, TRUE);
//     }
//
//     FdoExtension->TimerValue = 500;
//     USBPORT_StartTimer((PVOID)FdoDevice, 500);
//
//     Status = USBPORT_RegisterDeviceInterface(FdoExtension->CommonExtension.LowerPdoDevice,
//                                              FdoDevice,
//                                              &GUID_DEVINTERFACE_USB_HOST_CONTROLLER,
//                                              TRUE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBPORT_StartDevice: RegisterDeviceInterface failed!\n");
//         goto ExitWithError;
//     }
//
//     USBPORT_CreateLegacySymbolicLink(FdoDevice);
//
//     FdoExtension->Flags |= USBPORT_FLAG_HC_STARTED;
//
//     DPRINT("USBPORT_StartDevice: Exit Status - %p\n", Status);
//     return Status;
//
// ExitWithError:
//     USBPORT_StopWorkerThread(FdoDevice);
//     USBPORT_StopDevice(FdoDevice);
//
//     DPRINT1("USBPORT_StartDevice: ExitWithError Status - %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ParseResources(IN PDEVICE_OBJECT FdoDevice,
//                        IN PIRP Irp,
//                        IN PUSBPORT_RESOURCES UsbPortResources)
// {
//     PCM_RESOURCE_LIST AllocatedResourcesTranslated;
//     PCM_PARTIAL_RESOURCE_LIST ResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PortDescriptor = NULL;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR MemoryDescriptor = NULL;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR InterruptDescriptor = NULL;
//     PIO_STACK_LOCATION IoStack;
//     ULONG ix;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("USBPORT_ParseResources: ... \n");
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     AllocatedResourcesTranslated = IoStack->Parameters.StartDevice.AllocatedResourcesTranslated;
//
//     if (AllocatedResourcesTranslated)
//     {
//         RtlZeroMemory(UsbPortResources, sizeof(USBPORT_RESOURCES));
//
//         ResourceList = &AllocatedResourcesTranslated->List[0].PartialResourceList;
//
//         PartialDescriptor = &ResourceList->PartialDescriptors[0];
//
//         for (ix = 0; ix < ResourceList->Count; ++ix)
//         {
//             if (PartialDescriptor->Type == CmResourceTypePort)
//             {
//                 if (!PortDescriptor)
//                     PortDescriptor = PartialDescriptor;
//             }
//             else if (PartialDescriptor->Type == CmResourceTypeInterrupt)
//             {
//                 if (!InterruptDescriptor)
//                     InterruptDescriptor = PartialDescriptor;
//             }
//             else if (PartialDescriptor->Type == CmResourceTypeMemory)
//             {
//                 if (!MemoryDescriptor)
//                     MemoryDescriptor = PartialDescriptor;
//             }
//
//             PartialDescriptor += 1;
//         }
//
//         if (PortDescriptor)
//         {
//             if (PortDescriptor->Flags & CM_RESOURCE_PORT_IO)
//             {
//                 UsbPortResources->ResourceBase = (PVOID)(ULONG_PTR)PortDescriptor->u.Port.Start.QuadPart;
//             }
//             else
//             {
//                 UsbPortResources->ResourceBase = MmMapIoSpace(PortDescriptor->u.Port.Start,
//                                                               PortDescriptor->u.Port.Length,
//                                                               0);
//             }
//
//             UsbPortResources->IoSpaceLength = PortDescriptor->u.Port.Length;
//
//             if (UsbPortResources->ResourceBase)
//             {
//                 UsbPortResources->ResourcesTypes |= USBPORT_RESOURCES_PORT;
//             }
//             else
//             {
//                 Status = STATUS_NONE_MAPPED;
//             }
//         }
//
//         if (MemoryDescriptor && NT_SUCCESS(Status))
//         {
//             UsbPortResources->IoSpaceLength = MemoryDescriptor->u.Memory.Length;
//
//             UsbPortResources->ResourceBase = MmMapIoSpace(MemoryDescriptor->u.Memory.Start,
//                                                           MemoryDescriptor->u.Memory.Length,
//                                                           0);
//
//             if (UsbPortResources->ResourceBase)
//             {
//                 UsbPortResources->ResourcesTypes |= USBPORT_RESOURCES_MEMORY;
//             }
//             else
//             {
//                 Status = STATUS_NONE_MAPPED;
//             }
//         }
//
//         if (InterruptDescriptor && NT_SUCCESS(Status))
//         {
//             UsbPortResources->ResourcesTypes |= USBPORT_RESOURCES_INTERRUPT;
//
//             UsbPortResources->InterruptVector = InterruptDescriptor->u.Interrupt.Vector;
//             UsbPortResources->InterruptLevel = InterruptDescriptor->u.Interrupt.Level;
//             UsbPortResources->InterruptAffinity = InterruptDescriptor->u.Interrupt.Affinity;
//
//             UsbPortResources->ShareVector = InterruptDescriptor->ShareDisposition ==
//                                             CmResourceShareShared;
//
//             UsbPortResources->InterruptMode = InterruptDescriptor->Flags ==
//                                               CM_RESOURCE_INTERRUPT_LATCHED;
//         }
//     }
//     else
//     {
//         Status = STATUS_NONE_MAPPED;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_CreatePdo(IN PDEVICE_OBJECT FdoDevice,
//                   OUT PDEVICE_OBJECT *RootHubPdo)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     UNICODE_STRING DeviceName;
//     ULONG DeviceNumber = 0;
//     PDEVICE_OBJECT DeviceObject = NULL;
//     WCHAR CharDeviceName[64];
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("USBPORT_CreatePdo: FdoDevice - %p, RootHubPdo - %p\n",
//            FdoDevice,
//            RootHubPdo);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     do
//     {
//         RtlStringCbPrintfW(CharDeviceName,
//                            sizeof(CharDeviceName),
//                            L"\\Device\\USBPDO-%d",
//                            DeviceNumber);
//
//         RtlInitUnicodeString(&DeviceName, CharDeviceName);
//
//         DPRINT("USBPORT_CreatePdo: DeviceName - %wZ\n", &DeviceName);
//
//         Status = IoCreateDevice(FdoExtension->MiniPortInterface->DriverObject,
//                                 sizeof(USBPORT_RHDEVICE_EXTENSION),
//                                 &DeviceName,
//                                 FILE_DEVICE_BUS_EXTENDER,
//                                 0,
//                                 FALSE,
//                                 &DeviceObject);
//
//         ++DeviceNumber;
//     }
//     while (Status == STATUS_OBJECT_NAME_COLLISION);
//
//     if (!NT_SUCCESS(Status))
//     {
//         *RootHubPdo = NULL;
//         DPRINT1("USBPORT_CreatePdo: Filed create HubPdo!\n");
//         return Status;
//     }
//
//     if (DeviceObject)
//     {
//         PdoExtension = DeviceObject->DeviceExtension;
//
//         RtlZeroMemory(PdoExtension, sizeof(USBPORT_RHDEVICE_EXTENSION));
//
//         PdoExtension->CommonExtension.SelfDevice = DeviceObject;
//         PdoExtension->CommonExtension.IsPDO = TRUE;
//
//         PdoExtension->FdoDevice = FdoDevice;
//         PdoExtension->PdoNameNumber = DeviceNumber;
//
//         USBPORT_AdjustDeviceCapabilities(FdoDevice, DeviceObject);
//
//         DeviceObject->StackSize = FdoDevice->StackSize;
//
//         DeviceObject->Flags |= DO_POWER_PAGABLE;
//         DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//     }
//     else
//     {
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     if (!NT_SUCCESS(Status))
//         *RootHubPdo = NULL;
//     else
//         *RootHubPdo = DeviceObject;
//
//     DPRINT("USBPORT_CreatePdo: HubPdo - %p\n", DeviceObject);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_FdoPnP(IN PDEVICE_OBJECT FdoDevice,
//                IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_COMMON_DEVICE_EXTENSION FdoCommonExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_RESOURCES UsbPortResources;
//     PIO_STACK_LOCATION IoStack;
//     UCHAR Minor;
//     KEVENT Event;
//     NTSTATUS Status;
//     DEVICE_RELATION_TYPE RelationType;
//     PDEVICE_RELATIONS DeviceRelations;
//     PDEVICE_OBJECT RootHubPdo;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     FdoCommonExtension = &FdoExtension->CommonExtension;
//     UsbPortResources = &FdoExtension->UsbPortResources;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Minor = IoStack->MinorFunction;
//
//     DPRINT("USBPORT_FdoPnP: FdoDevice - %p, Minor - %x\n", FdoDevice, Minor);
//
//     RelationType = IoStack->Parameters.QueryDeviceRelations.Type;
//
//     switch (Minor)
//     {
//         case IRP_MN_START_DEVICE:
//             DPRINT("IRP_MN_START_DEVICE\n");
//
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//             IoCopyCurrentIrpStackLocationToNext(Irp);
//
//             IoSetCompletionRoutine(Irp,
//                                    USBPORT_FdoStartCompletion,
//                                    &Event,
//                                    TRUE,
//                                    TRUE,
//                                    TRUE);
//
//             Status = IoCallDriver(FdoCommonExtension->LowerDevice,
//                                   Irp);
//
//             if (Status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&Event,
//                                       Suspended,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//
//                 Status = Irp->IoStatus.Status;
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Exit;
//             }
//
//             Status = USBPORT_ParseResources(FdoDevice,
//                                             Irp,
//                                             UsbPortResources);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_STOPPED;
//                 goto Exit;
//             }
//
//             Status = USBPORT_StartDevice(FdoDevice, UsbPortResources);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_STOPPED;
//                 goto Exit;
//             }
//
//             FdoCommonExtension->PnpStateFlags &= ~USBPORT_PNP_STATE_NOT_INIT;
//             FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_STARTED;
//
//             FdoCommonExtension->DevicePowerState = PowerDeviceD0;
//
//             if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//             {
//                 USBPORT_AddUSB2Fdo(FdoDevice);
//             }
//             else
//             {
//                 USBPORT_AddUSB1Fdo(FdoDevice);
//             }
//
// Exit:
//             Irp->IoStatus.Status = Status;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//             DPRINT("IRP_MN_QUERY_REMOVE_DEVICE\n");
//             if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//             {
//                 DPRINT1("USBPORT_FdoPnP: Haction registry write FIXME\n");
//             }
//
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             goto ForwardIrp;
//
//         case IRP_MN_REMOVE_DEVICE:
//             DPRINT("USBPORT_FdoPnP: IRP_MN_REMOVE_DEVICE\n");
//             FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_FAILED;
//
//             if (FdoCommonExtension->PnpStateFlags & USBPORT_PNP_STATE_STARTED &&
//                !(FdoCommonExtension->PnpStateFlags & USBPORT_PNP_STATE_NOT_INIT))
//             {
//                 DPRINT1("USBPORT_FdoPnP: stop fdo FIXME\n");
//                 FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_NOT_INIT;
//             }
//
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             IoSkipCurrentIrpStackLocation(Irp);
//             Status = IoCallDriver(FdoCommonExtension->LowerDevice, Irp);
//
//             IoDetachDevice(FdoCommonExtension->LowerDevice);
//
//             RootHubPdo = FdoExtension->RootHubPdo;
//
//             IoDeleteDevice(FdoDevice);
//
//             if (RootHubPdo)
//             {
//                 IoDeleteDevice(RootHubPdo);
//             }
//
//             return Status;
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//             DPRINT("IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             goto ForwardIrp;
//
//         case IRP_MN_STOP_DEVICE:
//             DPRINT("IRP_MN_STOP_DEVICE\n");
//             if (FdoCommonExtension->PnpStateFlags & USBPORT_PNP_STATE_STARTED)
//             {
//                 DPRINT1("USBPORT_FdoPnP: stop fdo FIXME\n");
//
//                 FdoCommonExtension->PnpStateFlags &= ~USBPORT_PNP_STATE_STARTED;
//                 FdoCommonExtension->PnpStateFlags |= USBPORT_PNP_STATE_NOT_INIT;
//             }
//
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//             DPRINT("IRP_MN_QUERY_STOP_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             goto ForwardIrp;
//
//         case IRP_MN_CANCEL_STOP_DEVICE:
//             DPRINT("IRP_MN_CANCEL_STOP_DEVICE\n");
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//             DPRINT("IRP_MN_QUERY_DEVICE_RELATIONS\n");
//             if (RelationType == BusRelations)
//             {
//                 DeviceRelations = ExAllocatePoolWithTag(PagedPool,
//                                                         sizeof(DEVICE_RELATIONS),
//                                                         USB_PORT_TAG);
//
//                 if (!DeviceRelations)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     Irp->IoStatus.Status = Status;
//                     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//                     return Status;
//                 }
//
//                 DeviceRelations->Count = 0;
//                 DeviceRelations->Objects[0] = NULL;
//
//                 if (!FdoExtension->RootHubPdo)
//                 {
//                     Status = USBPORT_CreatePdo(FdoDevice,
//                                                &FdoExtension->RootHubPdo);
//
//                     if (!NT_SUCCESS(Status))
//                     {
//                         ExFreePoolWithTag(DeviceRelations, USB_PORT_TAG);
//                         goto ForwardIrp;
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_SUCCESS;
//                 }
//
//                 DeviceRelations->Count = 1;
//                 DeviceRelations->Objects[0] = FdoExtension->RootHubPdo;
//
//                 ObReferenceObject(FdoExtension->RootHubPdo);
//                 Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//             }
//             else
//             {
//                 if (RelationType == RemovalRelations)
//                 {
//                     DPRINT1("USBPORT_FdoPnP: FIXME IRP_MN_QUERY_DEVICE_RELATIONS/RemovalRelations\n");
//                 }
//
//                 goto ForwardIrp;
//             }
//
//             Irp->IoStatus.Status = Status;
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_INTERFACE:
//             DPRINT("IRP_MN_QUERY_INTERFACE\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_CAPABILITIES:
//             DPRINT("IRP_MN_QUERY_CAPABILITIES\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_RESOURCES:
//             DPRINT("IRP_MN_QUERY_RESOURCES\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//             DPRINT("IRP_MN_QUERY_RESOURCE_REQUIREMENTS\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_DEVICE_TEXT:
//             DPRINT("IRP_MN_QUERY_DEVICE_TEXT\n");
//             goto ForwardIrp;
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//             DPRINT("IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             goto ForwardIrp;
//
//         case IRP_MN_READ_CONFIG:
//             DPRINT("IRP_MN_READ_CONFIG\n");
//             goto ForwardIrp;
//
//         case IRP_MN_WRITE_CONFIG:
//             DPRINT("IRP_MN_WRITE_CONFIG\n");
//             goto ForwardIrp;
//
//         case IRP_MN_EJECT:
//             DPRINT("IRP_MN_EJECT\n");
//             goto ForwardIrp;
//
//         case IRP_MN_SET_LOCK:
//             DPRINT("IRP_MN_SET_LOCK\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_ID:
//             DPRINT("IRP_MN_QUERY_ID\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_PNP_DEVICE_STATE:
//             DPRINT("IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//             goto ForwardIrp;
//
//         case IRP_MN_QUERY_BUS_INFORMATION:
//             DPRINT("IRP_MN_QUERY_BUS_INFORMATION\n");
//             goto ForwardIrp;
//
//         case IRP_MN_DEVICE_USAGE_NOTIFICATION:
//             DPRINT("IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
//             goto ForwardIrp;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             DPRINT1("IRP_MN_SURPRISE_REMOVAL\n");
//             if (!(FdoCommonExtension->PnpStateFlags & USBPORT_PNP_STATE_FAILED))
//             {
//                 USBPORT_InvalidateControllerHandler(FdoDevice,
//                                                     USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE);
//             }
//             goto ForwardIrp;
//
//         default:
//             DPRINT("unknown IRP_MN_???\n");
// ForwardIrp:
//             /* forward irp to next device object */
//             IoSkipCurrentIrpStackLocation(Irp);
//             break;
//     }
//
//     return IoCallDriver(FdoCommonExtension->LowerDevice, Irp);
// }
//
// PVOID
// NTAPI
// USBPORT_GetDeviceHwIds(IN PDEVICE_OBJECT FdoDevice,
//                        IN USHORT VendorID,
//                        IN USHORT DeviceID,
//                        IN USHORT RevisionID)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PVOID Id;
//     WCHAR Buffer[300] = {0};
//     SIZE_T Length = 0;
//     size_t Remaining = sizeof(Buffer);
//     PWCHAR EndBuffer;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     DPRINT("USBPORT_GetDeviceHwIds: FdoDevice - %p, Packet->MiniPortFlags - %p\n",
//            FdoDevice,
//            Packet->MiniPortFlags);
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         RtlStringCbPrintfExW(Buffer,
//                              Remaining,
//                              &EndBuffer,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB20&VID%04x&PID%04x&REV%04x",
//                              VendorID,
//                              DeviceID,
//                              RevisionID);
//
//         EndBuffer++;
//         Remaining -= sizeof(UNICODE_NULL);
//
//         RtlStringCbPrintfExW(EndBuffer,
//                              Remaining,
//                              &EndBuffer,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB20&VID%04x&PID%04x",
//                              VendorID,
//                              DeviceID);
//
//         EndBuffer++;
//         Remaining -= sizeof(UNICODE_NULL);
//
//         RtlStringCbPrintfExW(EndBuffer,
//                              Remaining,
//                              NULL,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB20");
//     }
//     else
//     {
//         RtlStringCbPrintfExW(Buffer,
//                              Remaining,
//                              &EndBuffer,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB&VID%04x&PID%04x&REV%04x",
//                              VendorID,
//                              DeviceID,
//                              RevisionID);
//
//         EndBuffer++;
//         Remaining -= sizeof(UNICODE_NULL);
//
//         RtlStringCbPrintfExW(EndBuffer,
//                              Remaining,
//                              &EndBuffer,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB&VID%04x&PID%04x",
//                              VendorID,
//                              DeviceID);
//
//         EndBuffer++;
//         Remaining -= sizeof(UNICODE_NULL);
//
//         RtlStringCbPrintfExW(EndBuffer,
//                              Remaining,
//                              NULL,
//                              &Remaining,
//                              0,
//                              L"USB\\ROOT_HUB");
//     }
//
//     Length = (sizeof(Buffer) - Remaining + 2 * sizeof(UNICODE_NULL));
//
//      /* for debug only */
//     if (FALSE)
//     {
//         DPRINT("Hardware IDs:\n");
//         USBPORT_DumpingIDs(Buffer);
//     }
//
//     Id = ExAllocatePoolWithTag(PagedPool, Length, USB_PORT_TAG);
//
//     if (!Id)
//         return NULL;
//
//     RtlMoveMemory(Id, Buffer, Length);
//
//     return Id;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoPnP(IN PDEVICE_OBJECT PdoDevice,
//                IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_COMMON_DEVICE_EXTENSION PdoCommonExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIO_STACK_LOCATION IoStack;
//     UCHAR Minor;
//     NTSTATUS Status;
//     PPNP_BUS_INFORMATION BusInformation;
//     PDEVICE_CAPABILITIES DeviceCapabilities;
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     PdoCommonExtension = &PdoExtension->CommonExtension;
//
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     Minor = IoStack->MinorFunction;
//
//     Status = Irp->IoStatus.Status;
//
//     DPRINT("USBPORT_PdoPnP: PdoDevice - %p, Minor - %x\n", PdoDevice, Minor);
//
//     switch (Minor)
//     {
//         case IRP_MN_START_DEVICE:
//             DPRINT("IRP_MN_START_DEVICE\n");
//
//             Status = USBPORT_RootHubCreateDevice(FdoDevice, PdoDevice);
//
//             if (NT_SUCCESS(Status))
//             {
//                 Status = USBPORT_RegisterDeviceInterface(PdoDevice,
//                                                          PdoDevice,
//                                                          &GUID_DEVINTERFACE_USB_HUB,
//                                                          TRUE);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     PdoCommonExtension->DevicePowerState = PowerDeviceD0;
//                     PdoCommonExtension->PnpStateFlags = USBPORT_PNP_STATE_STARTED;
//                 }
//             }
//
//             break;
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//             DPRINT("USBPORT_PdoPnP: IRP_MN_QUERY_REMOVE_DEVICE\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_REMOVE_DEVICE:
//             DPRINT1("USBPORT_PdoPnP: IRP_MN_REMOVE_DEVICE UNIMPLEMENTED. FIXME. \n");
//             //USBPORT_StopRootHub();
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//             DPRINT("IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_STOP_DEVICE:
//             DPRINT1("USBPORT_PdoPnP: IRP_MN_STOP_DEVICE UNIMPLEMENTED. FIXME. \n");
//             //USBPORT_StopRootHub();
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//             DPRINT("IRP_MN_QUERY_STOP_DEVICE\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_CANCEL_STOP_DEVICE:
//             DPRINT("IRP_MN_CANCEL_STOP_DEVICE\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             PDEVICE_RELATIONS DeviceRelations;
//
//             DPRINT("IRP_MN_QUERY_DEVICE_RELATIONS\n");
//             if (IoStack->Parameters.QueryDeviceRelations.Type != TargetDeviceRelation)
//             {
//                 break;
//             }
//
//             DeviceRelations = ExAllocatePoolWithTag(PagedPool,
//                                                     sizeof(DEVICE_RELATIONS),
//                                                     USB_PORT_TAG);
//
//             if (!DeviceRelations)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//
//             DeviceRelations->Count = 1;
//             DeviceRelations->Objects[0] = PdoDevice;
//
//             ObReferenceObject(PdoDevice);
//
//             Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//             break;
//         }
//
//         case IRP_MN_QUERY_INTERFACE:
//             DPRINT("IRP_MN_QUERY_INTERFACE\n");
//             Status = USBPORT_PdoQueryInterface(FdoDevice, PdoDevice, Irp);
//             break;
//
//         case IRP_MN_QUERY_CAPABILITIES:
//             DPRINT("IRP_MN_QUERY_CAPABILITIES\n");
//
//             DeviceCapabilities = IoStack->Parameters.DeviceCapabilities.Capabilities;
//
//             RtlCopyMemory(DeviceCapabilities,
//                           &PdoExtension->Capabilities,
//                           sizeof(DEVICE_CAPABILITIES));
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_RESOURCES:
//             DPRINT("USBPORT_PdoPnP: IRP_MN_QUERY_RESOURCES\n");
//             break;
//
//         case IRP_MN_QUERY_RESOURCE_REQUIREMENTS:
//             DPRINT("IRP_MN_QUERY_RESOURCE_REQUIREMENTS\n");
//             break;
//
//         case IRP_MN_QUERY_DEVICE_TEXT:
//             DPRINT("IRP_MN_QUERY_DEVICE_TEXT\n");
//             break;
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS:
//             DPRINT("IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             break;
//
//         case IRP_MN_READ_CONFIG:
//             DPRINT("IRP_MN_READ_CONFIG\n");
//             ASSERT(FALSE);
//             break;
//
//         case IRP_MN_WRITE_CONFIG:
//             DPRINT("IRP_MN_WRITE_CONFIG\n");
//             ASSERT(FALSE);
//             break;
//
//         case IRP_MN_EJECT:
//             DPRINT("IRP_MN_EJECT\n");
//             ASSERT(FALSE);
//             break;
//
//         case IRP_MN_SET_LOCK:
//             DPRINT("IRP_MN_SET_LOCK\n");
//             ASSERT(FALSE);
//             break;
//
//         case IRP_MN_QUERY_ID:
//         {
//             ULONG IdType;
//             LONG Length;
//             WCHAR Buffer[64] = {0};
//             PVOID Id;
//
//             Status = STATUS_SUCCESS;
//             IdType = IoStack->Parameters.QueryId.IdType;
//
//             DPRINT("IRP_MN_QUERY_ID/Type %x\n", IdType);
//
//             if (IdType == BusQueryDeviceID)
//             {
//                 PUSBPORT_REGISTRATION_PACKET Packet;
//                 Packet = &FdoExtension->MiniPortInterface->Packet;
//
//                 if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//                 {
//                     RtlStringCbPrintfW(Buffer,
//                                        sizeof(Buffer),
//                                        L"USB\\ROOT_HUB20");
//                 }
//                 else
//                 {
//                     RtlStringCbPrintfW(Buffer,
//                                        sizeof(Buffer),
//                                        L"USB\\ROOT_HUB");
//                 }
//
//                 Length = (LONG)(crate::glue::wcslen(Buffer) + 1);
//
//                 Id = ExAllocatePoolWithTag(PagedPool,
//                                            Length * sizeof(WCHAR),
//                                            USB_PORT_TAG);
//
//                 if (Id)
//                 {
//                     RtlZeroMemory(Id, Length * sizeof(WCHAR));
//                     RtlStringCbCopyW(Id, Length * sizeof(WCHAR), Buffer);
//
//                     DPRINT("BusQueryDeviceID - %S, TotalLength - %hu\n",
//                            Id,
//                            Length);
//                 }
//
//                 Irp->IoStatus.Information = (ULONG_PTR)Id;
//                 break;
//             }
//
//             if (IdType == BusQueryHardwareIDs)
//             {
//                 Id = USBPORT_GetDeviceHwIds(FdoDevice,
//                                             FdoExtension->VendorID,
//                                             FdoExtension->DeviceID,
//                                             FdoExtension->RevisionID);
//
//                 Irp->IoStatus.Information = (ULONG_PTR)Id;
//                 break;
//             }
//
//             if (IdType == BusQueryCompatibleIDs ||
//                 IdType == BusQueryInstanceID)
//             {
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//
//             /* IdType == BusQueryDeviceSerialNumber */
//             Status = Irp->IoStatus.Status;
//             break;
//         }
//
//         case IRP_MN_QUERY_PNP_DEVICE_STATE:
//             DPRINT("IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IRP_MN_QUERY_BUS_INFORMATION:
//             DPRINT("IRP_MN_QUERY_BUS_INFORMATION\n");
//
//             /* Allocate buffer for bus information */
//             BusInformation = ExAllocatePoolWithTag(PagedPool,
//                                                    sizeof(PNP_BUS_INFORMATION),
//                                                    USB_PORT_TAG);
//
//             if (!BusInformation)
//             {
//                 /* No memory */
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//
//             RtlZeroMemory(BusInformation, sizeof(PNP_BUS_INFORMATION));
//
//             /* Copy BUS GUID */
//             RtlMoveMemory(&BusInformation->BusTypeGuid,
//                           &GUID_BUS_TYPE_USB,
//                           sizeof(GUID));
//
//             /* Set bus type */
//             BusInformation->LegacyBusType = PNPBus;
//             BusInformation->BusNumber = 0;
//
//             Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = (ULONG_PTR)BusInformation;
//             break;
//
//         case IRP_MN_DEVICE_USAGE_NOTIFICATION:
//             DPRINT("IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             DPRINT("USBPORT_PdoPnP: IRP_MN_SURPRISE_REMOVAL\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         default:
//             DPRINT("unknown IRP_MN_???\n");
//             break;
//     }
//
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
