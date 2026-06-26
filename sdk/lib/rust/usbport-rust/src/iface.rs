use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, RtlFreeUnicodeString, RtlStringFromGUID};
use core::ffi::c_void;

extern "C" {
    fn USBPORT_BadRequestFlush(FdoDevice: PDEVICE_OBJECT);
}
extern "C" {
    fn USBPORT_CreateDevice( UsbdDeviceHandle: *mut PUSBPORT_DEVICE_HANDLE, FdoDevice: PDEVICE_OBJECT, HubDeviceHandle: PUSBPORT_DEVICE_HANDLE, PortStatus: USHORT, PortNumber: USHORT, ) -> NTSTATUS;
    fn USBPORT_GetUsbDescriptor( DeviceHandle: PUSBPORT_DEVICE_HANDLE, FdoDevice: PDEVICE_OBJECT, Type: UCHAR, ConfigDescBuffer: PUCHAR, ConfigDescBufferLen: PULONG, ) -> NTSTATUS;
    fn USBPORT_DumpingDeviceDescriptor(Descriptor: PUSB_DEVICE_DESCRIPTOR);
    fn USBPORT_RemoveDevice( FdoDevice: PDEVICE_OBJECT, DeviceHandle: PUSBPORT_DEVICE_HANDLE, Flags: ULONG, ) -> NTSTATUS;
    fn USBPORT_RestoreDevice( FdoDevice: PDEVICE_OBJECT, OldUsbdDeviceHandle: PUSBPORT_DEVICE_HANDLE, NewUsbdDeviceHandle: PUSBPORT_DEVICE_HANDLE, ) -> NTSTATUS;
    fn USBPORT_SetRegistryKeyValue( DeviceObject: PDEVICE_OBJECT, UseDriverKey: BOOLEAN, Type: ULONG, ValueName: PCWSTR, Data: PVOID, DataSize: ULONG, ) -> NTSTATUS;
    fn USBPORT_FindUSB2Controller(FdoDevice: PDEVICE_OBJECT) -> PDEVICE_OBJECT;
    fn USBPORT_GetRegistryKeyValueFullInfo( FdoDevice: PDEVICE_OBJECT, LowerPdoDevice: PDEVICE_OBJECT, UseDefault: BOOLEAN, KeyNameString: PCWSTR, KeyNameStringLength: ULONG, Data: PVOID, DataLength: ULONG, ) -> NTSTATUS;
    fn USBPORT_GetSymbolicName( PdoDevice: PDEVICE_OBJECT, HubName: PUNICODE_STRING, ) -> NTSTATUS;
    fn DbgPrint(Format: *const i8, ...) -> u32;
    fn USBPORT_Initialize20Hub( FdoDevice: PDEVICE_OBJECT, UsbdHubDeviceHandle: PUSBPORT_DEVICE_HANDLE, TtCount: ULONG, ) -> NTSTATUS;
    fn IsEqualGUIDAligned(Guid1: *const GUID, Guid2: *const GUID) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG

const USBPORT_FLAG_BIOS_DISABLE_SS: ULONG = 0x00000080;
const USBPORT_FLAG_SELECTIVE_SUSPEND: ULONG = 0x00000200;

const USB_MINIPORT_FLAGS_USB2: u32 = 0x00000400;
const USB_PORTATTR_SHARED_USB2: u32 = 0x00000001;
const USB_PORTATTR_OWNED_BY_CC: u32 = 0x00000002;
const USB_PORTATTR_NO_OVERCURRENT_UI: u32 = 0x00000004;
const USBPORT_FLAG_COMPANION_HC: u32 = 0x01000000;

#[no_mangle]
pub unsafe extern "C" fn USBI_InterfaceReference(BusContext: PVOID) {
}
#[no_mangle]
pub unsafe extern "C" fn USBI_InterfaceDereference(BusContext: PVOID) {
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_CreateUsbDevice(
    BusContext: PVOID,
    UsbdDeviceHandle: *mut PUSB_DEVICE_HANDLE,
    UsbdHubDeviceHandle: PUSB_DEVICE_HANDLE,
    PortStatus: USHORT,
    PortNumber: USHORT,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut deviceHandle: PUSB_DEVICE_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    Status = USBPORT_CreateDevice(
        core::ptr::addr_of_mut!(deviceHandle) as *mut PUSBPORT_DEVICE_HANDLE,
        (*PdoExtension).FdoDevice,
        UsbdHubDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        PortStatus,
        PortNumber,
    );

    *UsbdDeviceHandle = deviceHandle;

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_InitializeUsbDevice(
    mut BusContext: PVOID,
    mut UsbdDeviceHandle: PUSB_DEVICE_HANDLE,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    return USBPORT_InitializeDevice(
        UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        (*PdoExtension).FdoDevice,
    );
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_GetUsbDescriptors(
    BusContext: PVOID,
    UsbdDeviceHandle: PUSB_DEVICE_HANDLE,
    DeviceDescBuffer: PUCHAR,
    DeviceDescBufferLen: PULONG,
    ConfigDescBuffer: PUCHAR,
    ConfigDescBufferLen: PULONG,
) -> NTSTATUS {

    const USB_CONFIGURATION_DESCRIPTOR_TYPE: u32 = 0x02;

    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;

    let Status: NTSTATUS;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    DeviceHandle = UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;

    if !DeviceDescBuffer.is_null() && *DeviceDescBufferLen != 0 {
        if *DeviceDescBufferLen as usize > core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() {
            *DeviceDescBufferLen = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG;
        }

        RtlCopyMemory(
            DeviceDescBuffer as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*DeviceHandle).DeviceDescriptor) as *const core::ffi::c_void,
            *DeviceDescBufferLen as usize,
        );
    }

    Status = USBPORT_GetUsbDescriptor(
        DeviceHandle,
        (*PdoExtension).FdoDevice,
        USB_CONFIGURATION_DESCRIPTOR_TYPE as UCHAR,
        ConfigDescBuffer,
        ConfigDescBufferLen,
    );

    USBPORT_DumpingDeviceDescriptor(DeviceDescBuffer as PUSB_DEVICE_DESCRIPTOR);

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_RemoveUsbDevice(
    mut BusContext: PVOID,
    mut UsbdDeviceHandle: PUSB_DEVICE_HANDLE,
    mut Flags: ULONG,
) -> NTSTATUS {

    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    return USBPORT_RemoveDevice(
        (*PdoExtension).FdoDevice,
        UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        Flags,
    );
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_RestoreUsbDevice(
    mut BusContext: PVOID,
    mut OldUsbdDeviceHandle: PUSB_DEVICE_HANDLE,
    mut NewUsbdDeviceHandle: PUSB_DEVICE_HANDLE,
) -> NTSTATUS {

    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    return USBPORT_RestoreDevice(
        (*PdoExtension).FdoDevice,
        OldUsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        NewUsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE,
    );
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_QueryDeviceInformation(
    mut BusContext: PVOID,
    mut UsbdDeviceHandle: PUSB_DEVICE_HANDLE,
    mut DeviceInfoBuffer: PVOID,
    mut DeviceInfoBufferLen: ULONG,
    mut LenDataReturned: PULONG,
) -> NTSTATUS {
    let mut DeviceInfo: PUSB_DEVICE_INFORMATION_0;
    let mut ConfigHandle: PUSBPORT_CONFIGURATION_HANDLE;
    let mut InterfaceEntry: PLIST_ENTRY;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut NumberOfOpenPipes: ULONG = 0;
    let mut PipeInfo: PUSB_PIPE_INFORMATION_0;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut InterfaceHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut ActualLength: ULONG;
    let mut ix: ULONG;

    *LenDataReturned = 0;

    if (DeviceInfoBufferLen as usize) < core::mem::size_of::<USB_LEVEL_INFORMATION>() {
        return STATUS_BUFFER_TOO_SMALL;
    }

    DeviceInfo = DeviceInfoBuffer as PUSB_DEVICE_INFORMATION_0;

    if (*DeviceInfo).InformationLevel > 0 {
        return STATUS_NOT_SUPPORTED;
    }

    DeviceHandle = UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    ConfigHandle = (*DeviceHandle).ConfigHandle;

    if !ConfigHandle.is_null() {
        InterfaceEntry = (*ConfigHandle).InterfaceHandleList.Flink;

        while !InterfaceEntry.is_null()
            && InterfaceEntry != core::ptr::addr_of_mut!((*ConfigHandle).InterfaceHandleList)
        {
            InterfaceHandle = (InterfaceEntry as usize).wrapping_sub(
                core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, InterfaceLink),
            ) as PUSBPORT_INTERFACE_HANDLE;

            NumberOfOpenPipes = NumberOfOpenPipes
                .wrapping_add((*InterfaceHandle).InterfaceDescriptor.bNumEndpoints as ULONG);

            InterfaceEntry = (*InterfaceEntry).Flink;
        }
    }

    ActualLength = (core::mem::offset_of!(USB_DEVICE_INFORMATION_0, PipeList))
        .wrapping_add((NumberOfOpenPipes as usize)
            .wrapping_mul(core::mem::size_of::<USB_PIPE_INFORMATION_0>())) as ULONG;

    if DeviceInfoBufferLen < ActualLength {
        (*DeviceInfo).ActualLength = ActualLength;
        *LenDataReturned = core::mem::size_of::<USB_LEVEL_INFORMATION>() as ULONG;

        return STATUS_BUFFER_TOO_SMALL;
    }

    RtlZeroMemory(DeviceInfo as PVOID, ActualLength as usize);

    (*DeviceInfo).InformationLevel = 0;
    (*DeviceInfo).ActualLength = ActualLength;
    (*DeviceInfo).DeviceAddress = (*DeviceHandle).DeviceAddress;
    (*DeviceInfo).NumberOfOpenPipes = NumberOfOpenPipes;
    (*DeviceInfo).DeviceSpeed = (*DeviceHandle).DeviceSpeed;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*DeviceInfo).DeviceDescriptor) as PVOID,
        core::ptr::addr_of_mut!((*DeviceHandle).DeviceDescriptor) as PVOID,
        core::mem::size_of::<USB_DEVICE_DESCRIPTOR>(),
    );

    USBPORT_DumpingDeviceDescriptor(core::ptr::addr_of_mut!((*DeviceInfo).DeviceDescriptor));

    if (*DeviceHandle).DeviceSpeed == UsbFullSpeed as USB_DEVICE_SPEED
        || (*DeviceHandle).DeviceSpeed == UsbLowSpeed as USB_DEVICE_SPEED
    {
        (*DeviceInfo).DeviceType = Usb11Device;
    } else if (*DeviceHandle).DeviceSpeed == UsbHighSpeed as USB_DEVICE_SPEED {
        (*DeviceInfo).DeviceType = Usb20Device;
    }

    (*DeviceInfo).CurrentConfigurationValue = 0;

    if ConfigHandle.is_null() {
        *LenDataReturned = ActualLength;
        return STATUS_SUCCESS;
    }

    (*DeviceInfo).CurrentConfigurationValue =
        (*(*ConfigHandle).ConfigurationDescriptor).bConfigurationValue;

    InterfaceEntry = (*ConfigHandle).InterfaceHandleList.Flink;

    while !InterfaceEntry.is_null()
        && InterfaceEntry != core::ptr::addr_of_mut!((*ConfigHandle).InterfaceHandleList)
    {
        InterfaceHandle = (InterfaceEntry as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, InterfaceLink),
        ) as PUSBPORT_INTERFACE_HANDLE;

        if (*InterfaceHandle).InterfaceDescriptor.bNumEndpoints as i32 > 0 {
            PipeInfo = core::ptr::addr_of_mut!((*DeviceInfo).PipeList) as *mut USB_PIPE_INFORMATION_0;
            PipeHandle =
                core::ptr::addr_of_mut!((*InterfaceHandle).PipeHandle) as *mut USBPORT_PIPE_HANDLE;

            ix = 0;
            while ix < (*InterfaceHandle).InterfaceDescriptor.bNumEndpoints as ULONG {
                if (*PipeHandle).Flags as ULONG & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE as ULONG != 0 {
                    (*PipeInfo).ScheduleOffset = 1;
                } else {
                    (*PipeInfo).ScheduleOffset =
                        (*(*PipeHandle).Endpoint).EndpointProperties.ScheduleOffset;
                }

                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*PipeInfo).EndpointDescriptor) as PVOID,
                    core::ptr::addr_of_mut!((*PipeHandle).EndpointDescriptor) as PVOID,
                    core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>(),
                );

                PipeInfo = PipeInfo.offset(1);
                PipeHandle = PipeHandle.offset(1);
                ix = ix.wrapping_add(1);
            }
        }

        InterfaceEntry = (*InterfaceEntry).Flink;
    }

    *LenDataReturned = ActualLength;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_GetControllerInformation(
    BusContext: PVOID,
    ControllerInfoBuffer: PVOID,
    ControllerInfoBufferLen: ULONG,
    LenDataReturned: PULONG,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut InfoBuffer: PUSB_CONTROLLER_INFORMATION_0;
    let mut Status: NTSTATUS;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    InfoBuffer = ControllerInfoBuffer as PUSB_CONTROLLER_INFORMATION_0;

    *LenDataReturned = 0;

    if (ControllerInfoBufferLen as usize) < core::mem::size_of::<USB_LEVEL_INFORMATION>() {
        Status = STATUS_BUFFER_TOO_SMALL;
        return Status;
    }

    *LenDataReturned = core::mem::size_of::<USB_LEVEL_INFORMATION>() as ULONG;

    if (*InfoBuffer).InformationLevel > 0 {
        Status = STATUS_NOT_SUPPORTED;
        return Status;
    }

    (*InfoBuffer).ActualLength = core::mem::size_of::<USB_CONTROLLER_INFORMATION_0>() as ULONG;

    if ControllerInfoBufferLen as usize >= core::mem::size_of::<USB_CONTROLLER_INFORMATION_0>() {
        (*InfoBuffer).SelectiveSuspendEnabled = (((*FdoExtension).Flags
            & USBPORT_FLAG_SELECTIVE_SUSPEND)
            == USBPORT_FLAG_SELECTIVE_SUSPEND) as i32 as BOOLEAN;
    }

    *LenDataReturned = core::mem::size_of::<USB_CONTROLLER_INFORMATION_0>() as ULONG;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_ControllerSelectiveSuspend(
    BusContext: PVOID,
    Enable: BOOLEAN,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Flags: ULONG;
    let mut HcDisable: ULONG;
    let mut Status: NTSTATUS;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Flags = (*FdoExtension).Flags;

    if Flags & USBPORT_FLAG_BIOS_DISABLE_SS != 0 {
        return STATUS_SUCCESS;
    }

    if Enable != 0 {
        (*FdoExtension).Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
        HcDisable = 0;
    } else {
        (*FdoExtension).Flags &= !USBPORT_FLAG_SELECTIVE_SUSPEND;
        HcDisable = 1;
    }

    Status = USBPORT_SetRegistryKeyValue(
        (*FdoExtension).CommonExtension.LowerPdoDevice,
        1,
        REG_DWORD as ULONG,
        ::core::mem::transmute::<&[u16; 26], PCWSTR>(&[
            0x48, 0x63, 0x44, 0x69, 0x73, 0x61, 0x62, 0x6c, 0x65, 0x53, 0x65, 0x6c, 0x65, 0x63,
            0x74, 0x69, 0x76, 0x65, 0x53, 0x75, 0x73, 0x70, 0x65, 0x6e, 0x64, 0x00,
        ]),
        core::ptr::addr_of_mut!(HcDisable) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );

    if NT_SUCCESS(Status) {
        if Enable != 0 {
            (*FdoExtension).Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
        } else {
            (*FdoExtension).Flags &= !USBPORT_FLAG_SELECTIVE_SUSPEND;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_GetExtendedHubInformation(
    BusContext: PVOID,
    HubPhysicalDeviceObject: PDEVICE_OBJECT,
    HubInformationBuffer: PVOID,
    HubInfoLen: ULONG,
    LenDataReturned: PULONG,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut NumPorts: ULONG;
    let mut ix: ULONG;
    let mut HubInfoBuffer: PUSB_EXTHUB_INFORMATION_0;
    let mut PortStatus: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();
    let mut PortAttrX: ULONG;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    HubInfoBuffer = HubInformationBuffer as PUSB_EXTHUB_INFORMATION_0;
    PortStatus.AsUlong32 = 0;

    if HubPhysicalDeviceObject != PdoDevice {
        *LenDataReturned = 0;
        return STATUS_NOT_SUPPORTED;
    }

    if (HubInfoLen as usize) < core::mem::size_of::<USB_EXTHUB_INFORMATION_0>() {
        *LenDataReturned = 0;
        return STATUS_BUFFER_TOO_SMALL;
    }

    NumPorts = (*(*PdoExtension).RootHubDescriptors).Descriptor.bNumberOfPorts as ULONG;
    (*HubInfoBuffer).NumberOfPorts = NumPorts;

    if NumPorts == 0 {
        *LenDataReturned = core::mem::size_of::<USB_EXTHUB_INFORMATION_0>() as ULONG;
        return STATUS_SUCCESS;
    }

    ix = 0;
    while ix < (*HubInfoBuffer).NumberOfPorts {
        (*HubInfoBuffer).Port[ix as usize].PhysicalPortNumber = ix.wrapping_add(1);
        (*HubInfoBuffer).Port[ix as usize].PortLabelNumber = ix;
        (*HubInfoBuffer).Port[ix as usize].VidOverride = 0;
        (*HubInfoBuffer).Port[ix as usize].PidOverride = 0;
        (*HubInfoBuffer).Port[ix as usize].PortAttributes = 0;

        if (*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_USB2 != 0 {
            (*HubInfoBuffer).Port[ix as usize].PortAttributes = USB_PORTATTR_SHARED_USB2 as ULONG;

            ((*Packet).RH_GetPortStatus.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
                ix.wrapping_add(1) as USHORT,
                core::ptr::addr_of_mut!(PortStatus),
            );

            if PortStatus.PortStatus.Usb20PortStatus.AsUshort16 as i32 & 0x8000 != 0 {
                (*HubInfoBuffer).Port[ix as usize].PortAttributes |= USB_PORTATTR_OWNED_BY_CC;
            }
        } else {
            if (*FdoExtension).Flags as u32 & USBPORT_FLAG_COMPANION_HC == 0 {
                ix = ix.wrapping_add(1);
                continue;
            }

            if !USBPORT_FindUSB2Controller(FdoDevice).is_null() {
                (*HubInfoBuffer).Port[ix as usize].PortAttributes |= USB_PORTATTR_NO_OVERCURRENT_UI;
            }
        }
        ix = ix.wrapping_add(1);
    }

    ix = 0;
    while ix < (*HubInfoBuffer).NumberOfPorts {
        PortAttrX = 0;

        USBPORT_GetRegistryKeyValueFullInfo(
            FdoDevice,
            (*FdoExtension).CommonExtension.LowerPdoDevice,
            0,
            core::ptr::addr_of!(*(b"P\0o\0r\0t\0A\0t\0t\0r\0X\0\0\0".as_ptr()
                as *const [u16; 10])) as PCWSTR,
            core::mem::size_of::<[u16; 10]>() as ULONG,
            core::ptr::addr_of_mut!(PortAttrX) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );

        (*HubInfoBuffer).Port[ix as usize].PortAttributes |= PortAttrX;
        ix = ix.wrapping_add(1);
    }

    *LenDataReturned = core::mem::size_of::<USB_EXTHUB_INFORMATION_0>() as ULONG;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_GetRootHubSymbolicName(
    BusContext: PVOID,
    HubInfoBuffer: PVOID,
    HubInfoBufferLen: ULONG,
    HubNameActualLen: PULONG,
) -> NTSTATUS {

    let mut PdoDevice: PDEVICE_OBJECT;
    let mut HubName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut InfoBuffer: PUNICODE_STRING;
    let mut Status: NTSTATUS;

    PdoDevice = BusContext as PDEVICE_OBJECT;

    Status = USBPORT_GetSymbolicName(PdoDevice, core::ptr::addr_of_mut!(HubName));

    if HubInfoBufferLen < HubName.Length as ULONG {
        InfoBuffer = HubInfoBuffer as PUNICODE_STRING;
        (*InfoBuffer).Length = 0;
    } else {
        RtlCopyMemory(
            HubInfoBuffer,
            HubName.Buffer as *const core::ffi::c_void,
            HubName.Length as usize,
        );
    }

    *HubNameActualLen = HubName.Length as ULONG;

    if NT_SUCCESS(Status) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(HubName));
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_GetDeviceBusContext(
    mut BusContext: PVOID,
    mut DeviceHandle: PVOID,
) -> PVOID {
    if DbgPrint(
        b"(%s:%d) USBHI_GetDeviceBusContext: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as *const i8,
        b"drivers/usb/usbport/iface.c\0"
            as *const u8 as *const i8,
        562 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"drivers/usb/usbport/iface.c\0"
                as *const u8 as *const i8,
            562 as i32,
        );
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_Initialize20Hub(
    BusContext: PVOID,
    UsbdHubDeviceHandle: PUSB_DEVICE_HANDLE,
    TtCount: ULONG,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    return USBPORT_Initialize20Hub(
        (*PdoExtension).FdoDevice,
        UsbdHubDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        TtCount,
    );
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_RootHubInitNotification(
    mut BusContext: PVOID,
    mut CallbackContext: PVOID,
    mut CallbackFunction: PRH_INIT_CALLBACK,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).RootHubCallbackSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    (*PdoExtension).RootHubInitContext = CallbackContext;
    (*PdoExtension).RootHubInitCallback = CallbackFunction;
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).RootHubCallbackSpinLock) as *mut usize),
        OldIrql,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBHI_FlushTransfers(
    BusContext: PVOID,
    UsbdDeviceHandle: PUSB_DEVICE_HANDLE,
) {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

    USBPORT_BadRequestFlush((*PdoExtension).FdoDevice);
}

#[no_mangle]
pub unsafe extern "C" fn USBHI_SetDeviceHandleData(
    mut BusContext: PVOID,
    mut DeviceHandle: PVOID,
    mut UsbDevicePdo: PDEVICE_OBJECT,
) {
    if DbgPrint(
        b"(%s:%d) USBHI_SetDeviceHandleData: UNIMPLEMENTED. FIXME.\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/iface.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        636 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/iface.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            636 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn USBDI_GetUSBDIVersion(
    mut BusContext: PVOID,
    mut VersionInfo: PUSBD_VERSION_INFORMATION,
    mut HcdCapabilities: PULONG,
) {
    if DbgPrint(
        b"(%s:%d) USBDI_GetUSBDIVersion: UNIMPLEMENTED. FIXME.\n\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/iface.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        647 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/iface.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            647 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn USBDI_QueryBusTime(
    mut BusContext: PVOID,
    mut CurrentFrame: PULONG,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBDI_QueryBusTime: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/usb/usbport/iface.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        655 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbport/iface.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            655 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBDI_SubmitIsoOutUrb(
    mut BusContext: PVOID,
    mut Urb: PURB,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBDI_SubmitIsoOutUrb: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/iface.c\0"
            as *const u8 as *const core::ffi::c_char,
        664 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/iface.c\0"
                as *const u8 as *const core::ffi::c_char,
            664 as core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBDI_QueryBusInformation(
    BusContext: PVOID,
    Level: ULONG,
    BusInfoBuffer: PVOID,
    BusInfoBufferLen: PULONG,
    BusInfoActualLen: PULONG,
) -> NTSTATUS {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Length: SIZE_T;
    let mut Buffer1: PUSB_BUS_INFORMATION_LEVEL_1;

    if Level != 0 && Level != 1 {
        return STATUS_NOT_SUPPORTED;
    }

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    if Level == 0 {
        if !BusInfoActualLen.is_null() {
            *BusInfoActualLen = core::mem::size_of::<USB_BUS_INFORMATION_LEVEL_0>() as ULONG;
        }

        if (*BusInfoBufferLen as usize) < core::mem::size_of::<USB_BUS_INFORMATION_LEVEL_0>() {
            return STATUS_BUFFER_TOO_SMALL;
        }

        *BusInfoBufferLen = core::mem::size_of::<USB_BUS_INFORMATION_LEVEL_0>() as ULONG;

        return STATUS_SUCCESS;
    }

    if Level == 1 {
        Length = (core::mem::size_of::<USB_BUS_INFORMATION_LEVEL_1>())
            .wrapping_add((*FdoExtension).CommonExtension.SymbolicLinkName.Length as usize)
            as SIZE_T;

        if !BusInfoActualLen.is_null() {
            *BusInfoActualLen = Length as ULONG;
        }

        if (*BusInfoBufferLen as u64) < Length as u64 {
            return STATUS_BUFFER_TOO_SMALL;
        }

        *BusInfoBufferLen = Length as ULONG;

        Buffer1 = BusInfoBuffer as PUSB_BUS_INFORMATION_LEVEL_1;

        (*Buffer1).ControllerNameLength =
            (*FdoExtension).CommonExtension.SymbolicLinkName.Length as ULONG;

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Buffer1).ControllerNameUnicodeString) as *mut core::ffi::c_void,
            (*FdoExtension).CommonExtension.SymbolicLinkName.Buffer as *const core::ffi::c_void,
            (*FdoExtension).CommonExtension.SymbolicLinkName.Length as usize,
        );

        return STATUS_SUCCESS;
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn USBDI_IsDeviceHighSpeed(mut BusContext: PVOID) -> BOOLEAN {
    let mut PdoDevice: PDEVICE_OBJECT;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;

    PdoDevice = BusContext as PDEVICE_OBJECT;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    return (((*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2) != 0) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn USBDI_EnumLogEntry(
    mut BusContext: PVOID,
    mut DriverTag: ULONG,
    mut EnumTag: ULONG,
    mut P1: ULONG,
    mut P2: ULONG,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) USBDI_EnumLogEntry: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/iface.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        776 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/iface.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            776 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PdoQueryInterface(
    FdoDevice: PDEVICE_OBJECT,
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut InterfaceHub: PUSB_BUS_INTERFACE_HUB_V5 =
        core::ptr::null_mut::<_USB_BUS_INTERFACE_HUB_V5>();
    let mut InterfaceDI: PUSB_BUS_INTERFACE_USBDI_V2 =
        core::ptr::null_mut::<_USB_BUS_INTERFACE_USBDI_V2>();
    let mut GuidBuffer: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut Status: NTSTATUS;

    if IsEqualGUIDAligned(
        (*IoStack).Parameters.QueryInterface.InterfaceType,
        core::ptr::addr_of!(USB_BUS_INTERFACE_HUB_GUID),
    ) != 0
    {
        InterfaceHub = (*IoStack).Parameters.QueryInterface.Interface as PUSB_BUS_INTERFACE_HUB_V5;
        (*InterfaceHub).Version = (*IoStack).Parameters.QueryInterface.Version;

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 6 {
            return (*Irp).IoStatus.c2rust_unnamed.__bindgen_anon_1.Status;
        }

        (*InterfaceHub).Size = (*IoStack).Parameters.QueryInterface.Size;
        (*InterfaceHub).BusContext = PdoDevice as PVOID;

        (*InterfaceHub).InterfaceReference = Some(USBI_InterfaceReference);
        (*InterfaceHub).InterfaceDereference = Some(USBI_InterfaceDereference);

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 1 {
            (*InterfaceHub).CreateUsbDevice = Some(USBHI_CreateUsbDevice);
            (*InterfaceHub).InitializeUsbDevice = Some(USBHI_InitializeUsbDevice);
            (*InterfaceHub).GetUsbDescriptors = Some(USBHI_GetUsbDescriptors);
            (*InterfaceHub).RemoveUsbDevice = Some(USBHI_RemoveUsbDevice);
            (*InterfaceHub).RestoreUsbDevice = Some(USBHI_RestoreUsbDevice);
            (*InterfaceHub).QueryDeviceInformation = Some(USBHI_QueryDeviceInformation);
        }

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 2 {
            (*InterfaceHub).GetControllerInformation = Some(USBHI_GetControllerInformation);
            (*InterfaceHub).ControllerSelectiveSuspend = Some(USBHI_ControllerSelectiveSuspend);
            (*InterfaceHub).GetExtendedHubInformation = Some(USBHI_GetExtendedHubInformation);
            (*InterfaceHub).GetRootHubSymbolicName = Some(USBHI_GetRootHubSymbolicName);
            (*InterfaceHub).GetDeviceBusContext = Some(USBHI_GetDeviceBusContext);
            (*InterfaceHub).Initialize20Hub = Some(USBHI_Initialize20Hub);
        }

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 3 {
            (*InterfaceHub).RootHubInitNotification = Some(USBHI_RootHubInitNotification);
        }

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 4 {
            (*InterfaceHub).FlushTransfers = Some(USBHI_FlushTransfers);
        }

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 5 {
            (*InterfaceHub).SetDeviceHandleData = Some(USBHI_SetDeviceHandleData);
        }

        return STATUS_SUCCESS;
    } else if IsEqualGUIDAligned(
        (*IoStack).Parameters.QueryInterface.InterfaceType,
        core::ptr::addr_of!(USB_BUS_INTERFACE_USBDI_GUID),
    ) != 0
    {
        InterfaceDI = (*IoStack).Parameters.QueryInterface.Interface as PUSB_BUS_INTERFACE_USBDI_V2;
        (*InterfaceDI).Version = (*IoStack).Parameters.QueryInterface.Version;

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 3 {
            return (*Irp).IoStatus.c2rust_unnamed.__bindgen_anon_1.Status;
        }

        (*InterfaceDI).Size = (*IoStack).Parameters.QueryInterface.Size;
        (*InterfaceDI).BusContext = PdoDevice as PVOID;
        (*InterfaceDI).InterfaceReference = Some(USBI_InterfaceReference);
        (*InterfaceDI).InterfaceDereference = Some(USBI_InterfaceDereference);
        (*InterfaceDI).GetUSBDIVersion = Some(USBDI_GetUSBDIVersion);
        (*InterfaceDI).QueryBusTime = Some(USBDI_QueryBusTime);
        (*InterfaceDI).SubmitIsoOutUrb = Some(USBDI_SubmitIsoOutUrb);
        (*InterfaceDI).QueryBusInformation = Some(USBDI_QueryBusInformation);

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 1 {
            (*InterfaceDI).IsDeviceHighSpeed = Some(USBDI_IsDeviceHighSpeed);
        }

        if (*IoStack).Parameters.QueryInterface.Version as i32 >= 2 {
            (*InterfaceDI).EnumLogEntry = Some(USBDI_EnumLogEntry);
        }

        return STATUS_SUCCESS;
    } else {
        Status = RtlStringFromGUID(
            (*IoStack).Parameters.QueryInterface.InterfaceType,
            core::ptr::addr_of_mut!(GuidBuffer),
        );

        if NT_SUCCESS(Status) {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(GuidBuffer));
        }
    }

    return (*Irp).IoStatus.c2rust_unnamed.__bindgen_anon_1.Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort interface functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// USB_BUSIFFN
// USBI_InterfaceReference(IN PVOID BusContext)
// {
//     DPRINT("USBI_InterfaceReference\n");
// }
//
// VOID
// USB_BUSIFFN
// USBI_InterfaceDereference(IN PVOID BusContext)
// {
//     DPRINT("USBI_InterfaceDereference\n");
// }
//
// /* USB port driver Interface functions */
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_CreateUsbDevice(IN PVOID BusContext,
//                       IN OUT PUSB_DEVICE_HANDLE *UsbdDeviceHandle,
//                       IN PUSB_DEVICE_HANDLE UsbdHubDeviceHandle,
//                       IN USHORT PortStatus,
//                       IN USHORT PortNumber)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSB_DEVICE_HANDLE deviceHandle = NULL;
//     NTSTATUS Status;
//
//     DPRINT("USBHI_CreateUsbDevice: ...\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     Status = USBPORT_CreateDevice(&deviceHandle,
//                                   PdoExtension->FdoDevice,
//                                   (PUSBPORT_DEVICE_HANDLE)UsbdHubDeviceHandle,
//                                   PortStatus,
//                                   PortNumber);
//
//     *UsbdDeviceHandle = deviceHandle;
//
//     return Status;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_InitializeUsbDevice(IN PVOID BusContext,
//                           OUT PUSB_DEVICE_HANDLE UsbdDeviceHandle)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBHI_InitializeUsbDevice\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     return USBPORT_InitializeDevice((PUSBPORT_DEVICE_HANDLE)UsbdDeviceHandle,
//                                     PdoExtension->FdoDevice);
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_GetUsbDescriptors(IN PVOID BusContext,
//                         IN PUSB_DEVICE_HANDLE UsbdDeviceHandle,
//                         IN PUCHAR DeviceDescBuffer,
//                         IN PULONG DeviceDescBufferLen,
//                         IN PUCHAR ConfigDescBuffer,
//                         IN PULONG ConfigDescBufferLen)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//
//     NTSTATUS Status;
//
//     DPRINT("USBHI_GetUsbDescriptors ...\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     DeviceHandle = (PUSBPORT_DEVICE_HANDLE)UsbdDeviceHandle;
//
//     if (DeviceDescBuffer && *DeviceDescBufferLen)
//     {
//         if (*DeviceDescBufferLen > sizeof(USB_DEVICE_DESCRIPTOR))
//             *DeviceDescBufferLen = sizeof(USB_DEVICE_DESCRIPTOR);
//
//         RtlCopyMemory(DeviceDescBuffer,
//                       &DeviceHandle->DeviceDescriptor,
//                       *DeviceDescBufferLen);
//     }
//
//     Status = USBPORT_GetUsbDescriptor(DeviceHandle,
//                                       PdoExtension->FdoDevice,
//                                       USB_CONFIGURATION_DESCRIPTOR_TYPE,
//                                       ConfigDescBuffer,
//                                       ConfigDescBufferLen);
//
//     USBPORT_DumpingDeviceDescriptor((PUSB_DEVICE_DESCRIPTOR)DeviceDescBuffer);
//
//     return Status;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_RemoveUsbDevice(IN PVOID BusContext,
//                       IN OUT PUSB_DEVICE_HANDLE UsbdDeviceHandle,
//                       IN ULONG Flags)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBHI_RemoveUsbDevice: UsbdDeviceHandle - %p, Flags - %x\n",
//            UsbdDeviceHandle,
//            Flags);
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     return USBPORT_RemoveDevice(PdoExtension->FdoDevice,
//                                 (PUSBPORT_DEVICE_HANDLE)UsbdDeviceHandle,
//                                 Flags);
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_RestoreUsbDevice(IN PVOID BusContext,
//                        OUT PUSB_DEVICE_HANDLE OldUsbdDeviceHandle,
//                        OUT PUSB_DEVICE_HANDLE NewUsbdDeviceHandle)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBHI_RestoreUsbDevice: OldUsbdDeviceHandle - %p, NewUsbdDeviceHandle - %x\n",
//            OldUsbdDeviceHandle,
//            NewUsbdDeviceHandle);
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     return USBPORT_RestoreDevice(PdoExtension->FdoDevice,
//                                 (PUSBPORT_DEVICE_HANDLE)OldUsbdDeviceHandle,
//                                 (PUSBPORT_DEVICE_HANDLE)NewUsbdDeviceHandle);
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_QueryDeviceInformation(IN PVOID BusContext,
//                              IN PUSB_DEVICE_HANDLE UsbdDeviceHandle,
//                              OUT PVOID DeviceInfoBuffer,
//                              IN ULONG DeviceInfoBufferLen,
//                              OUT PULONG LenDataReturned)
// {
//     PUSB_DEVICE_INFORMATION_0 DeviceInfo;
//     PUSBPORT_CONFIGURATION_HANDLE ConfigHandle;
//     PLIST_ENTRY InterfaceEntry;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     ULONG NumberOfOpenPipes = 0;
//     PUSB_PIPE_INFORMATION_0 PipeInfo;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle;
//     ULONG ActualLength;
//     ULONG ix;
//
//     DPRINT("USBHI_QueryDeviceInformation: ...\n");
//
//     *LenDataReturned = 0;
//
//     if (DeviceInfoBufferLen < sizeof(USB_LEVEL_INFORMATION))
//     {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     DeviceInfo = DeviceInfoBuffer;
//
//     if (DeviceInfo->InformationLevel > 0)
//     {
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     DeviceHandle = UsbdDeviceHandle;
//     ConfigHandle = DeviceHandle->ConfigHandle;
//
//     if (ConfigHandle)
//     {
//         InterfaceEntry = ConfigHandle->InterfaceHandleList.Flink;
//
//         while (InterfaceEntry &&
//                InterfaceEntry != &ConfigHandle->InterfaceHandleList)
//         {
//             InterfaceHandle = CONTAINING_RECORD(InterfaceEntry,
//                                                 USBPORT_INTERFACE_HANDLE,
//                                                 InterfaceLink);
//
//             NumberOfOpenPipes += InterfaceHandle->InterfaceDescriptor.bNumEndpoints;
//
//             InterfaceEntry = InterfaceEntry->Flink;
//         }
//     }
//
//     ActualLength = FIELD_OFFSET(USB_DEVICE_INFORMATION_0, PipeList) +
//                    NumberOfOpenPipes * sizeof(USB_PIPE_INFORMATION_0);
//
//     if (DeviceInfoBufferLen < ActualLength)
//     {
//         DeviceInfo->ActualLength = ActualLength;
//         *LenDataReturned = sizeof(USB_LEVEL_INFORMATION);
//
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     RtlZeroMemory(DeviceInfo, ActualLength);
//
//     DeviceInfo->InformationLevel = 0;
//     DeviceInfo->ActualLength = ActualLength;
//     DeviceInfo->DeviceAddress = DeviceHandle->DeviceAddress;
//     DeviceInfo->NumberOfOpenPipes = NumberOfOpenPipes;
//     DeviceInfo->DeviceSpeed = DeviceHandle->DeviceSpeed;
//
//     RtlCopyMemory(&DeviceInfo->DeviceDescriptor,
//                   &DeviceHandle->DeviceDescriptor,
//                   sizeof(USB_DEVICE_DESCRIPTOR));
//
//     USBPORT_DumpingDeviceDescriptor(&DeviceInfo->DeviceDescriptor);
//
//     if (DeviceHandle->DeviceSpeed == UsbFullSpeed ||
//         DeviceHandle->DeviceSpeed == UsbLowSpeed)
//     {
//         DeviceInfo->DeviceType = Usb11Device;
//     }
//     else if (DeviceHandle->DeviceSpeed == UsbHighSpeed)
//     {
//         DeviceInfo->DeviceType = Usb20Device;
//     }
//
//     DeviceInfo->CurrentConfigurationValue = 0;
//
//     if (!ConfigHandle)
//     {
//         *LenDataReturned = ActualLength;
//         return STATUS_SUCCESS;
//     }
//
//     DeviceInfo->CurrentConfigurationValue =
//         ConfigHandle->ConfigurationDescriptor->bConfigurationValue;
//
//     InterfaceEntry = ConfigHandle->InterfaceHandleList.Flink;
//
//     while (InterfaceEntry &&
//            InterfaceEntry != &ConfigHandle->InterfaceHandleList)
//     {
//         InterfaceHandle = CONTAINING_RECORD(InterfaceEntry,
//                                             USBPORT_INTERFACE_HANDLE,
//                                             InterfaceLink);
//
//         if (InterfaceHandle->InterfaceDescriptor.bNumEndpoints > 0)
//         {
//             PipeInfo = &DeviceInfo->PipeList[0];
//             PipeHandle = &InterfaceHandle->PipeHandle[0];
//
//             for (ix = 0;
//                  ix < InterfaceHandle->InterfaceDescriptor.bNumEndpoints;
//                  ix++)
//             {
//                 if (PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE)
//                 {
//                     PipeInfo->ScheduleOffset = 1;
//                 }
//                 else
//                 {
//                     PipeInfo->ScheduleOffset =
//                         PipeHandle->Endpoint->EndpointProperties.ScheduleOffset;
//                 }
//
//                 RtlCopyMemory(&PipeInfo->EndpointDescriptor,
//                               &PipeHandle->EndpointDescriptor,
//                               sizeof(USB_ENDPOINT_DESCRIPTOR));
//
//                 PipeInfo += 1;
//                 PipeHandle += 1;
//             }
//         }
//
//         InterfaceEntry = InterfaceEntry->Flink;
//     }
//
//     *LenDataReturned = ActualLength;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_GetControllerInformation(IN PVOID BusContext,
//                                OUT PVOID ControllerInfoBuffer,
//                                IN ULONG ControllerInfoBufferLen,
//                                OUT PULONG LenDataReturned)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSB_CONTROLLER_INFORMATION_0 InfoBuffer;
//     NTSTATUS Status;
//
//     DPRINT("USBHI_GetControllerInformation: ControllerInfoBufferLen - %x\n",
//            ControllerInfoBufferLen);
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     InfoBuffer = ControllerInfoBuffer;
//
//     *LenDataReturned = 0;
//
//     if (ControllerInfoBufferLen < sizeof(USB_LEVEL_INFORMATION))
//     {
//         Status = STATUS_BUFFER_TOO_SMALL;
//         return Status;
//     }
//
//     *LenDataReturned = sizeof(USB_LEVEL_INFORMATION);
//
//     if (InfoBuffer->InformationLevel > 0)
//     {
//         Status = STATUS_NOT_SUPPORTED;
//         return Status;
//     }
//
//     InfoBuffer->ActualLength = sizeof(USB_CONTROLLER_INFORMATION_0);
//
//     if (ControllerInfoBufferLen >= sizeof(USB_CONTROLLER_INFORMATION_0))
//     {
//         InfoBuffer->SelectiveSuspendEnabled =
//             (FdoExtension->Flags & USBPORT_FLAG_SELECTIVE_SUSPEND) ==
//             USBPORT_FLAG_SELECTIVE_SUSPEND;
//     }
//
//     *LenDataReturned = sizeof(USB_CONTROLLER_INFORMATION_0);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_ControllerSelectiveSuspend(IN PVOID BusContext,
//                                  IN BOOLEAN Enable)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG Flags;
//     ULONG HcDisable;
//     NTSTATUS Status;
//
//     DPRINT("USBHI_ControllerSelectiveSuspend: Enable - %x\n", Enable);
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Flags = FdoExtension->Flags;
//
//     if (Flags & USBPORT_FLAG_BIOS_DISABLE_SS)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (Enable)
//     {
//         FdoExtension->Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
//         HcDisable = 0;
//     }
//     else
//     {
//         FdoExtension->Flags &= ~USBPORT_FLAG_SELECTIVE_SUSPEND;
//         HcDisable = 1;
//     }
//
//     Status = USBPORT_SetRegistryKeyValue(FdoExtension->CommonExtension.LowerPdoDevice,
//                                          TRUE,
//                                          REG_DWORD,
//                                          L"HcDisableSelectiveSuspend",
//                                          &HcDisable,
//                                          sizeof(HcDisable));
//
//     if (NT_SUCCESS(Status))
//     {
//         if (Enable)
//             FdoExtension->Flags |= USBPORT_FLAG_SELECTIVE_SUSPEND;
//         else
//             FdoExtension->Flags &= ~USBPORT_FLAG_SELECTIVE_SUSPEND;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_GetExtendedHubInformation(IN PVOID BusContext,
//                                 IN PDEVICE_OBJECT HubPhysicalDeviceObject,
//                                 IN OUT PVOID HubInformationBuffer,
//                                 IN ULONG HubInfoLen,
//                                 IN OUT PULONG LenDataReturned)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     ULONG NumPorts;
//     ULONG ix;
//     PUSB_EXTHUB_INFORMATION_0 HubInfoBuffer;
//     USB_PORT_STATUS_AND_CHANGE PortStatus;
//     ULONG PortAttrX;
//
//     DPRINT("USBHI_GetExtendedHubInformation: ...\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     HubInfoBuffer = HubInformationBuffer;
//     PortStatus.AsUlong32 = 0;
//
//     if (HubPhysicalDeviceObject != PdoDevice)
//     {
//         *LenDataReturned = 0;
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     if (HubInfoLen < sizeof(USB_EXTHUB_INFORMATION_0))
//     {
//         *LenDataReturned = 0;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     NumPorts = PdoExtension->RootHubDescriptors->Descriptor.bNumberOfPorts;
//     HubInfoBuffer->NumberOfPorts = NumPorts;
//
//     if (NumPorts == 0)
//     {
//         *LenDataReturned = sizeof(USB_EXTHUB_INFORMATION_0);
//         return STATUS_SUCCESS;
//     }
//
//     for (ix = 0; ix < HubInfoBuffer->NumberOfPorts; ++ix)
//     {
//         HubInfoBuffer->Port[ix].PhysicalPortNumber = ix + 1;
//         HubInfoBuffer->Port[ix].PortLabelNumber = ix;
//         HubInfoBuffer->Port[ix].VidOverride = 0;
//         HubInfoBuffer->Port[ix].PidOverride = 0;
//         HubInfoBuffer->Port[ix].PortAttributes = 0;
//
//         if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//         {
//             HubInfoBuffer->Port[ix].PortAttributes = USB_PORTATTR_SHARED_USB2;
//
//             Packet->RH_GetPortStatus(FdoExtension->MiniPortExt,
//                                      ix + 1,
//                                      &PortStatus);
//
//             if (PortStatus.PortStatus.Usb20PortStatus.AsUshort16 & 0x8000)
//             {
//                 HubInfoBuffer->Port[ix].PortAttributes |= USB_PORTATTR_OWNED_BY_CC;
//             }
//         }
//         else
//         {
//             if (!(FdoExtension->Flags & USBPORT_FLAG_COMPANION_HC))
//             {
//                 continue;
//             }
//
//             if (USBPORT_FindUSB2Controller(FdoDevice))
//             {
//                 HubInfoBuffer->Port[ix].PortAttributes |= USB_PORTATTR_NO_OVERCURRENT_UI;
//             }
//         }
//     }
//
//     for (ix = 0; ix < HubInfoBuffer->NumberOfPorts; ++ix)
//     {
//         PortAttrX = 0;
//
//         USBPORT_GetRegistryKeyValueFullInfo(FdoDevice,
//                                             FdoExtension->CommonExtension.LowerPdoDevice,
//                                             FALSE,
//                                             L"PortAttrX",
//                                             sizeof(L"PortAttrX"),
//                                             &PortAttrX,
//                                             sizeof(PortAttrX));
//
//         HubInfoBuffer->Port[ix].PortAttributes |= PortAttrX;
//     }
//
//     *LenDataReturned = sizeof(USB_EXTHUB_INFORMATION_0);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_GetRootHubSymbolicName(IN PVOID BusContext,
//                              IN OUT PVOID HubInfoBuffer,
//                              IN ULONG HubInfoBufferLen,
//                              OUT PULONG HubNameActualLen)
// {
//     PDEVICE_OBJECT PdoDevice;
//     UNICODE_STRING HubName;
//     PUNICODE_STRING InfoBuffer;
//     NTSTATUS Status;
//
//     DPRINT("USBHI_GetRootHubSymbolicName: ...\n");
//
//     PdoDevice = BusContext;
//
//     Status = USBPORT_GetSymbolicName(PdoDevice, &HubName);
//
//     if (HubInfoBufferLen < HubName.Length)
//     {
//         InfoBuffer = HubInfoBuffer;
//         InfoBuffer->Length = 0;
//     }
//     else
//     {
//         RtlCopyMemory(HubInfoBuffer, HubName.Buffer, HubName.Length);
//     }
//
//     *HubNameActualLen = HubName.Length;
//
//     if (NT_SUCCESS(Status))
//         RtlFreeUnicodeString(&HubName);
//
//     return Status;
// }
//
// PVOID
// USB_BUSIFFN
// USBHI_GetDeviceBusContext(IN PVOID BusContext,
//                           IN PVOID DeviceHandle)
// {
//     DPRINT1("USBHI_GetDeviceBusContext: UNIMPLEMENTED. FIXME.\n");
//     return NULL;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_Initialize20Hub(IN PVOID BusContext,
//                       IN PUSB_DEVICE_HANDLE UsbdHubDeviceHandle,
//                       IN ULONG TtCount)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBHI_Initialize20Hub: UsbdHubDeviceHandle - %p, TtCount - %x\n",
//            UsbdHubDeviceHandle,
//            TtCount);
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     return USBPORT_Initialize20Hub(PdoExtension->FdoDevice,
//                                    (PUSBPORT_DEVICE_HANDLE)UsbdHubDeviceHandle,
//                                    TtCount);
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBHI_RootHubInitNotification(IN PVOID BusContext,
//                               IN PVOID CallbackContext,
//                               IN PRH_INIT_CALLBACK CallbackFunction)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//
//     DPRINT("USBHI_RootHubInitNotification\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->RootHubCallbackSpinLock, &OldIrql);
//     PdoExtension->RootHubInitContext = CallbackContext;
//     PdoExtension->RootHubInitCallback = CallbackFunction;
//     KeReleaseSpinLock(&FdoExtension->RootHubCallbackSpinLock, OldIrql);
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// USB_BUSIFFN
// USBHI_FlushTransfers(IN PVOID BusContext,
//                      OUT PUSB_DEVICE_HANDLE UsbdDeviceHandle)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//
//     DPRINT("USBHI_FlushTransfers: ...\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//
//     USBPORT_BadRequestFlush(PdoExtension->FdoDevice);
// }
//
// VOID
// USB_BUSIFFN
// USBHI_SetDeviceHandleData(IN PVOID BusContext,
//                           IN PVOID DeviceHandle,
//                           IN PDEVICE_OBJECT UsbDevicePdo)
// {
//     DPRINT1("USBHI_SetDeviceHandleData: UNIMPLEMENTED. FIXME.\n");
// }
//
// /* USB bus driver Interface functions */
//
// VOID
// USB_BUSIFFN
// USBDI_GetUSBDIVersion(IN PVOID BusContext,
//                       OUT PUSBD_VERSION_INFORMATION VersionInfo,
//                       OUT PULONG HcdCapabilities)
// {
//     DPRINT1("USBDI_GetUSBDIVersion: UNIMPLEMENTED. FIXME.\n");
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBDI_QueryBusTime(IN PVOID BusContext,
//                    OUT PULONG CurrentFrame)
// {
//     DPRINT1("USBDI_QueryBusTime: UNIMPLEMENTED. FIXME.\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBDI_SubmitIsoOutUrb(IN PVOID BusContext,
//                       IN PURB Urb)
// {
//     DPRINT1("USBDI_SubmitIsoOutUrb: UNIMPLEMENTED. FIXME.\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBDI_QueryBusInformation(IN PVOID BusContext,
//                           IN ULONG Level,
//                           OUT PVOID BusInfoBuffer,
//                           OUT PULONG BusInfoBufferLen,
//                           OUT PULONG BusInfoActualLen)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     SIZE_T Length;
//     PUSB_BUS_INFORMATION_LEVEL_1 Buffer1;
//
//     DPRINT("USBDI_QueryBusInformation: Level - %p\n", Level);
//
//     if ((Level != 0) && (Level != 1))
//     {
//         DPRINT1("USBDI_QueryBusInformation: Level should be 0 or 1\n");
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if (Level == 0)
//     {
//         if (BusInfoActualLen)
//             *BusInfoActualLen = sizeof(USB_BUS_INFORMATION_LEVEL_0);
//
//         if (*BusInfoBufferLen < sizeof(USB_BUS_INFORMATION_LEVEL_0))
//         {
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         *BusInfoBufferLen = sizeof(USB_BUS_INFORMATION_LEVEL_0);
//
//         //Buffer0 = BusInfoBuffer;
//         DPRINT1("USBDI_QueryBusInformation: LEVEL_0 UNIMPLEMENTED. FIXME\n");
//         //Buffer0->TotalBandwidth = USBPORT_GetTotalBandwidth();
//         //Buffer0->ConsumedBandwidth = USBPORT_GetAllocatedBandwidth();
//
//         return STATUS_SUCCESS;
//     }
//
//     if (Level == 1)
//     {
//         Length = sizeof(USB_BUS_INFORMATION_LEVEL_1) +
//                  FdoExtension->CommonExtension.SymbolicLinkName.Length;
//
//         if (BusInfoActualLen)
//             *BusInfoActualLen = Length;
//
//         if (*BusInfoBufferLen < Length)
//         {
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         *BusInfoBufferLen = Length;
//
//         Buffer1 = BusInfoBuffer;
//         DPRINT1("USBDI_QueryBusInformation: LEVEL_1 UNIMPLEMENTED. FIXME\n");
//         //Buffer1->TotalBandwidth = USBPORT_GetTotalBandwidth();
//         //Buffer1->ConsumedBandwidth = USBPORT_GetAllocatedBandwidth();
//         Buffer1->ControllerNameLength = FdoExtension->CommonExtension.SymbolicLinkName.Length;
//
//         RtlCopyMemory(&Buffer1->ControllerNameUnicodeString,
//                       FdoExtension->CommonExtension.SymbolicLinkName.Buffer,
//                       FdoExtension->CommonExtension.SymbolicLinkName.Length);
//
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// USB_BUSIFFN
// USBDI_IsDeviceHighSpeed(IN PVOID BusContext)
// {
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//
//     DPRINT("USBDI_IsDeviceHighSpeed: ...\n");
//
//     PdoDevice = BusContext;
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     return (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2) != 0;
// }
//
// NTSTATUS
// USB_BUSIFFN
// USBDI_EnumLogEntry(IN PVOID BusContext,
//                    IN ULONG DriverTag,
//                    IN ULONG EnumTag,
//                    IN ULONG P1,
//                    IN ULONG P2)
// {
//     DPRINT1("USBDI_EnumLogEntry: UNIMPLEMENTED. FIXME.\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_PdoQueryInterface(IN PDEVICE_OBJECT FdoDevice,
//                           IN PDEVICE_OBJECT PdoDevice,
//                           IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//     PUSB_BUS_INTERFACE_HUB_V5 InterfaceHub;
//     PUSB_BUS_INTERFACE_USBDI_V2 InterfaceDI;
//     UNICODE_STRING GuidBuffer;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_PdoQueryInterface: ...\n");
//
//     if (IsEqualGUIDAligned(IoStack->Parameters.QueryInterface.InterfaceType,
//                            &USB_BUS_INTERFACE_HUB_GUID))
//     {
//         /* Get request parameters */
//         InterfaceHub = (PUSB_BUS_INTERFACE_HUB_V5)IoStack->Parameters.QueryInterface.Interface;
//         InterfaceHub->Version = IoStack->Parameters.QueryInterface.Version;
//
//         /* Check version */
//         if (IoStack->Parameters.QueryInterface.Version >= 6)
//         {
//             DPRINT1("USB_BUS_INTERFACE_HUB_GUID version %x not supported!\n",
//                     IoStack->Parameters.QueryInterface.Version);
//
//             return Irp->IoStatus.Status; // Version not supported
//         }
//
//         /* Interface version 0 */
//         InterfaceHub->Size = IoStack->Parameters.QueryInterface.Size;
//         InterfaceHub->BusContext = PdoDevice;
//
//         InterfaceHub->InterfaceReference = USBI_InterfaceReference;
//         InterfaceHub->InterfaceDereference = USBI_InterfaceDereference;
//
//         /* Interface version 1 */
//         if (IoStack->Parameters.QueryInterface.Version >= 1)
//         {
//             InterfaceHub->CreateUsbDevice = USBHI_CreateUsbDevice;
//             InterfaceHub->InitializeUsbDevice = USBHI_InitializeUsbDevice;
//             InterfaceHub->GetUsbDescriptors = USBHI_GetUsbDescriptors;
//             InterfaceHub->RemoveUsbDevice = USBHI_RemoveUsbDevice;
//             InterfaceHub->RestoreUsbDevice = USBHI_RestoreUsbDevice;
//             InterfaceHub->QueryDeviceInformation = USBHI_QueryDeviceInformation;
//         }
//
//         /* Interface version 2 */
//         if (IoStack->Parameters.QueryInterface.Version >= 2)
//         {
//             InterfaceHub->GetControllerInformation = USBHI_GetControllerInformation;
//             InterfaceHub->ControllerSelectiveSuspend = USBHI_ControllerSelectiveSuspend;
//             InterfaceHub->GetExtendedHubInformation = USBHI_GetExtendedHubInformation;
//             InterfaceHub->GetRootHubSymbolicName = USBHI_GetRootHubSymbolicName;
//             InterfaceHub->GetDeviceBusContext = USBHI_GetDeviceBusContext;
//             InterfaceHub->Initialize20Hub = USBHI_Initialize20Hub;
//         }
//
//         /* Interface version 3 */
//         if (IoStack->Parameters.QueryInterface.Version >= 3)
//             InterfaceHub->RootHubInitNotification = USBHI_RootHubInitNotification;
//
//         /* Interface version 4 */
//         if (IoStack->Parameters.QueryInterface.Version >= 4)
//             InterfaceHub->FlushTransfers = USBHI_FlushTransfers;
//
//         /* Interface version 5 */
//         if (IoStack->Parameters.QueryInterface.Version >= 5)
//             InterfaceHub->SetDeviceHandleData = USBHI_SetDeviceHandleData;
//
//         /* Request completed */
//         return STATUS_SUCCESS;
//     }
//     else if (IsEqualGUIDAligned(IoStack->Parameters.QueryInterface.InterfaceType,
//                                 &USB_BUS_INTERFACE_USBDI_GUID))
//     {
//         /* Get request parameters */
//         InterfaceDI = (PUSB_BUS_INTERFACE_USBDI_V2)IoStack->Parameters.QueryInterface.Interface;
//         InterfaceDI->Version = IoStack->Parameters.QueryInterface.Version;
//
//         /* Check version */
//         if (IoStack->Parameters.QueryInterface.Version >= 3)
//         {
//             DPRINT1("USB_BUS_INTERFACE_USBDI_GUID version %x not supported!\n",
//                     IoStack->Parameters.QueryInterface.Version);
//
//             return Irp->IoStatus.Status; // Version not supported
//         }
//
//         /* Interface version 0 */
//         InterfaceDI->Size = IoStack->Parameters.QueryInterface.Size;
//         InterfaceDI->BusContext = PdoDevice;
//         InterfaceDI->InterfaceReference = USBI_InterfaceReference;
//         InterfaceDI->InterfaceDereference = USBI_InterfaceDereference;
//         InterfaceDI->GetUSBDIVersion = USBDI_GetUSBDIVersion;
//         InterfaceDI->QueryBusTime = USBDI_QueryBusTime;
//         InterfaceDI->SubmitIsoOutUrb = USBDI_SubmitIsoOutUrb;
//         InterfaceDI->QueryBusInformation = USBDI_QueryBusInformation;
//
//         /* Interface version 1 */
//         if (IoStack->Parameters.QueryInterface.Version >= 1)
//             InterfaceDI->IsDeviceHighSpeed = USBDI_IsDeviceHighSpeed;
//
//         /* Interface version 2 */
//         if (IoStack->Parameters.QueryInterface.Version >= 2)
//             InterfaceDI->EnumLogEntry = USBDI_EnumLogEntry;
//
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* Convert GUID to string */
//         Status = RtlStringFromGUID(IoStack->Parameters.QueryInterface.InterfaceType,
//                                    &GuidBuffer);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Print interface */
//             DPRINT1("HandleQueryInterface UNKNOWN INTERFACE GUID: %wZ Version %x\n",
//                     &GuidBuffer,
//                     IoStack->Parameters.QueryInterface.Version);
//
//             RtlFreeUnicodeString(&GuidBuffer); // Free GUID buffer
//         }
//     }
//
//     return Irp->IoStatus.Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
