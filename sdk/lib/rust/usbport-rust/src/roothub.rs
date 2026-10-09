use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PDEVICE_RELATIONS};
use core::ffi::c_void;

extern "system" {
    fn USBPORT_FlushCancelList(Endpoint: PUSBPORT_ENDPOINT);
    pub fn USBPORT_QueueDoneTransfer( Transfer: PUSBPORT_TRANSFER, USBDStatus: USBD_STATUS, ) -> BOOLEAN;
}
extern "system" {
    fn USBPORT_FindCompanionControllers(USB2FdoDevice: PDEVICE_OBJECT, IsObRefer: BOOLEAN, IsFDOsReturned: BOOLEAN,) -> PDEVICE_RELATIONS;
    fn USBPORT_AddDeviceHandle(FdoDevice: PDEVICE_OBJECT, DeviceHandle: PUSBPORT_DEVICE_HANDLE);
    pub fn USBPORT_OpenPipe( FdoDevice: PDEVICE_OBJECT, DeviceHandle: PUSBPORT_DEVICE_HANDLE, PipeHandle: PUSBPORT_PIPE_HANDLE, UsbdStatus: PUSBD_STATUS, ) -> NTSTATUS;
    fn USBPORT_HcQueueWakeDpc(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_InvalidateEndpointHandler( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, Type: ULONG, );
    pub fn USBPORT_Wait(MiniPortExtension: PVOID, Milliseconds: ULONG,) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// (no translatable declarations; NDEBUG macros are build flags)

unsafe extern "C" {
}

const RH_STATUS_NO_CHANGES: RHSTATUS = 0x01;
const RH_STATUS_SUCCESS: RHSTATUS = 0x00;
const RH_STATUS_UNSUCCESSFUL: RHSTATUS = 2;

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RH_SetFeatureUSB2PortPower(
    FdoDevice: PDEVICE_OBJECT,
    Port: USHORT,
) -> MPSTATUS {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let CompanionControllersList: PDEVICE_RELATIONS;
    let mut CompanionPacket: PUSBPORT_REGISTRATION_PACKET;
    let mut CompanionFdoDevice: PDEVICE_OBJECT;
    let mut CompanionFdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut ix: USHORT;
    let mut Entry: *mut PDEVICE_OBJECT;
    let mut NumController: ULONG = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    CompanionControllersList = USBPORT_FindCompanionControllers(FdoDevice, 0, 1);

    if CompanionControllersList.is_null() {
        ((*Packet).RH_SetFeaturePortPower).unwrap()((*FdoExtension).MiniPortExt, Port);
        return MP_STATUS_SUCCESS;
    }

    Entry = core::ptr::addr_of_mut!((*CompanionControllersList).Objects[0]);

    while NumController < (*CompanionControllersList).Count {
        CompanionFdoDevice = *Entry;

        let CompanionFdoExtension_v = (*CompanionFdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
        let CompanionFdoExtension = CompanionFdoExtension_v;
        let CompanionPacket = core::ptr::addr_of_mut!((*(*CompanionFdoExtension).MiniPortInterface).Packet);

        let PdoExtension = (*(*CompanionFdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

        ix = 0;
        while ((*PdoExtension).CommonExtension.PnpStateFlags & USBPORT_PNP_STATE_STARTED) != 0
            && (ix as USHORT) < (*(*PdoExtension).RootHubDescriptors).Descriptor.bNumberOfPorts as USHORT
        {
            ((*CompanionPacket).RH_SetFeaturePortPower).unwrap()(
                (*CompanionFdoExtension).MiniPortExt,
                ix + 1,
            );
            ix += 1;
        }

        NumController += 1;
        Entry = Entry.add(1);
    }

    ((*Packet).RH_SetFeaturePortPower).unwrap()((*FdoExtension).MiniPortExt, Port);

    if !CompanionControllersList.is_null() {
        ExFreePoolWithTag(CompanionControllersList as *mut core::ffi::c_void, USB_PORT_TAG);
    }

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubClassCommand(
    FdoDevice: PDEVICE_OBJECT,
    SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET,
    Buffer: PVOID,
    BufferLength: PULONG,
) -> RHSTATUS {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let Port: USHORT;
    let mut Feature: USHORT;
    let mut MPStatus: MPSTATUS;
    let mut RHStatus: RHSTATUS = RH_STATUS_UNSUCCESSFUL;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoExtension = (*(*FdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    Port = (*SetupPacket).wIndex.W;

    match (*SetupPacket).bRequest as u32 {
        USB_REQUEST_GET_STATUS => {
            if Buffer.is_null() {
                return RHStatus;
            }

            *(Buffer as PULONG) = 0;

            if (*SetupPacket).bmRequestType.Recipient() == (BMREQUEST_TO_OTHER as u8) {
                if Port as u32 > (*(*PdoExtension).RootHubDescriptors).Descriptor.bNumberOfPorts as u32
                    || Port <= 0
                    || (*SetupPacket).wLength < 4
                {
                    return RHStatus;
                }

                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!(OldIrql),
                );

                MPStatus = ((*Packet).RH_GetPortStatus).unwrap()(
                    (*FdoExtension).MiniPortExt,
                    (*SetupPacket).wIndex.W,
                    (Buffer as *mut _USB_PORT_STATUS_AND_CHANGE),
                );

                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    OldIrql,
                );
            } else {
                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!(OldIrql),
                );

                MPStatus = ((*Packet).RH_GetHubStatus).unwrap()(
                    (*FdoExtension).MiniPortExt,
                    (Buffer as *mut _USB_HUB_STATUS_AND_CHANGE),
                );

                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    OldIrql,
                );
            }

            RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
        }

        USB_REQUEST_CLEAR_FEATURE => {
            Feature = (*SetupPacket).wValue.W;

            if (*SetupPacket).bmRequestType.Recipient() != (USBPORT_RECIPIENT_PORT as u8) {
                if Feature as u32 == FEATURE_C_HUB_LOCAL_POWER {
                    RHStatus = RH_STATUS_SUCCESS;
                    return RHStatus;
                }

                if Feature as u32 == FEATURE_C_HUB_OVER_CURRENT {
                    MPStatus = ((*Packet).RH_ClearFeaturePortOvercurrentChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        0,
                    );
                    RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
                    return RHStatus;
                }

                DbgBreakPoint();
                return RHStatus;
            }

            match Feature as u32 {
                FEATURE_PORT_ENABLE => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortEnable).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_PORT_SUSPEND => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortSuspend).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_PORT_POWER => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortPower).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_C_PORT_CONNECTION => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortConnectChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_C_PORT_ENABLE => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortEnableChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_C_PORT_SUSPEND => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortSuspendChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_C_PORT_OVER_CURRENT => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortOvercurrentChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_C_PORT_RESET => {
                    MPStatus = ((*Packet).RH_ClearFeaturePortResetChange).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                _ => {
                    return RHStatus;
                }
            }

            RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
        }

        USB_REQUEST_SET_FEATURE => {
            if (*SetupPacket).bmRequestType.Recipient() != (USBPORT_RECIPIENT_PORT as u8) {
                return RHStatus;
            }

            Feature = (*SetupPacket).wValue.W;
            match Feature as u32 {
                FEATURE_PORT_ENABLE => {
                    MPStatus = ((*Packet).RH_SetFeaturePortEnable).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_PORT_SUSPEND => {
                    MPStatus = ((*Packet).RH_SetFeaturePortSuspend).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_PORT_RESET => {
                    MPStatus = ((*Packet).RH_SetFeaturePortReset).unwrap()(
                        (*FdoExtension).MiniPortExt,
                        Port,
                    );
                }
                FEATURE_PORT_POWER => {
                    if (*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
                        MPStatus = USBPORT_RH_SetFeatureUSB2PortPower(FdoDevice, Port);
                    } else {
                        MPStatus = ((*Packet).RH_SetFeaturePortPower).unwrap()(
                            (*FdoExtension).MiniPortExt,
                            Port,
                        );
                    }
                }
                _ => {
                    return RHStatus;
                }
            }

            RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
        }

        USB_REQUEST_GET_DESCRIPTOR => {
            if !Buffer.is_null()
                && (*SetupPacket).wValue.W == 0
                && (*SetupPacket).bmRequestType.Dir() == (BMREQUEST_DEVICE_TO_HOST as u8)
            {
                let mut DescriptorLength: SIZE_T;

                DescriptorLength =
                    (*(*PdoExtension).RootHubDescriptors).Descriptor.bDescriptorLength as SIZE_T;

                if (*BufferLength as SIZE_T) < DescriptorLength {
                    DescriptorLength = *BufferLength as SIZE_T;
                }

                RtlCopyMemory(
                    Buffer,
                    core::ptr::addr_of!((*(*PdoExtension).RootHubDescriptors).Descriptor) as PVOID,
                    DescriptorLength,
                );

                *BufferLength = DescriptorLength as ULONG;
                RHStatus = RH_STATUS_SUCCESS;
            }
        }

        _ => {}
    }

    return RHStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubStandardCommand(
    FdoDevice: PDEVICE_OBJECT,
    SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET,
    Buffer: PVOID,
    TransferLength: PULONG,
) -> RHSTATUS {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Length: SIZE_T;
    let mut Descriptor: PVOID;
    let mut DescriptorLength: SIZE_T;
    let mut MPStatus: MPSTATUS;
    let mut RHStatus: RHSTATUS = RH_STATUS_UNSUCCESSFUL;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoExtension = (*(*FdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    match (*SetupPacket).bRequest as u32 {
        USB_REQUEST_GET_DESCRIPTOR => {
            if (*SetupPacket).wValue.LowByte != 0 || (*SetupPacket).bmRequestType.Dir() == 0 {
                return RHStatus;
            }

            match (*SetupPacket).wValue.HiByte as u32 {
                USB_DEVICE_DESCRIPTOR_TYPE => {
                    Descriptor = core::ptr::addr_of_mut!(
                        (*(*PdoExtension).RootHubDescriptors).DeviceDescriptor
                    ) as PVOID;
                    DescriptorLength = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>();
                }
                USB_CONFIGURATION_DESCRIPTOR_TYPE => {
                    Descriptor = core::ptr::addr_of_mut!(
                        (*(*PdoExtension).RootHubDescriptors).ConfigDescriptor
                    ) as PVOID;
                    DescriptorLength = core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
                        + core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()
                        + core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>();
                }
                _ => {
                    return RHStatus;
                }
            }

            if Descriptor.is_null() {
                return RHStatus;
            }

            if *TransferLength as SIZE_T >= DescriptorLength {
                Length = DescriptorLength;
            } else {
                Length = *TransferLength as SIZE_T;
            }

            RtlCopyMemory(Buffer, Descriptor, Length);
            *TransferLength = Length as ULONG;

            RHStatus = RH_STATUS_SUCCESS;
        }

        USB_REQUEST_GET_STATUS => {
            if (*SetupPacket).wValue.W == 0
                && (*SetupPacket).wLength as usize == core::mem::size_of::<USHORT>()
                && (*SetupPacket).wIndex.W == 0
                && (*SetupPacket).bmRequestType.Dir() == (BMREQUEST_DEVICE_TO_HOST as u8)
            {
                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!(OldIrql),
                );

                MPStatus = ((*Packet).RH_GetStatus.unwrap())((*FdoExtension).MiniPortExt, (Buffer as *mut u16));

                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                    OldIrql,
                );

                *TransferLength = core::mem::size_of::<USHORT>() as ULONG;
                RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
            }
        }

        USB_REQUEST_GET_CONFIGURATION => {
            if (*SetupPacket).wValue.W != 0
                || (*SetupPacket).wIndex.W != 0
                || (*SetupPacket).wLength != 1
                || (*SetupPacket).bmRequestType.Dir() == (BMREQUEST_HOST_TO_DEVICE as u8)
            {
                return RHStatus;
            }

            Length = 0;

            if *TransferLength >= 1 {
                Length = 1;
                RtlCopyMemory(
                    Buffer,
                    core::ptr::addr_of_mut!((*PdoExtension).ConfigurationValue) as PVOID,
                    Length,
                );
            }

            *TransferLength = Length as ULONG;

            RHStatus = RH_STATUS_SUCCESS;
        }

        USB_REQUEST_SET_CONFIGURATION => {
            if (*SetupPacket).wIndex.W == 0
                && (*SetupPacket).wLength == 0
                && !((*SetupPacket).bmRequestType.Dir() == (BMREQUEST_DEVICE_TO_HOST as u8))
            {
                if (*SetupPacket).wValue.W == 0
                    || (*SetupPacket).wValue.W as u32
                        == (*(*PdoExtension).RootHubDescriptors)
                            .ConfigDescriptor
                            .bConfigurationValue as u32
                {
                    (*PdoExtension).ConfigurationValue = ((*SetupPacket).wValue.LowByte as u32);
                    RHStatus = RH_STATUS_SUCCESS;
                }
            }
        }

        USB_REQUEST_SET_ADDRESS => {
            if (*SetupPacket).wIndex.W == 0
                && (*SetupPacket).wLength == 0
                && (*SetupPacket).bmRequestType.Dir() == 0
            {
                (*PdoExtension).DeviceHandle.DeviceAddress = ((*SetupPacket).wValue.LowByte as u16);
                RHStatus = RH_STATUS_SUCCESS;
            }
        }

        _ => {}
    }

    return RHStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubEndpoint0(
    Transfer: PUSBPORT_TRANSFER,
) -> RHSTATUS {
    let FdoDevice: PDEVICE_OBJECT;
    let mut TransferLength: ULONG;
    let Buffer: PVOID;
    let Urb: PURB;
    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;
    let Type: UCHAR;
    let RHStatus: RHSTATUS;

    TransferLength = (*Transfer).TransferParameters.TransferBufferLength;
    Urb = (*Transfer).Urb;
    FdoDevice = (*(*Transfer).Endpoint).FdoDevice;

    if TransferLength > 0 {
        Buffer = (*(*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL).MappedSystemVa;
    } else {
        Buffer = core::ptr::null_mut();
    }

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer.SetupPacket)
        as *mut _ as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    Type = (*SetupPacket).bmRequestType.Type();

    if Type == (BMREQUEST_STANDARD as u8) {
        RHStatus = USBPORT_RootHubStandardCommand(
            FdoDevice,
            SetupPacket,
            Buffer,
            core::ptr::addr_of_mut!(TransferLength),
        );
    } else if Type == (BMREQUEST_CLASS as u8) {
        RHStatus = USBPORT_RootHubClassCommand(
            FdoDevice,
            SetupPacket,
            Buffer,
            core::ptr::addr_of_mut!(TransferLength),
        );
    } else {
        return RH_STATUS_UNSUCCESSFUL;
    }

    if RHStatus == RH_STATUS_SUCCESS {
        (*Transfer).CompletedTransferLen = TransferLength;
    }

    return RHStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubSCE(Transfer: PUSBPORT_TRANSFER) -> RHSTATUS {
    let Endpoint: PUSBPORT_ENDPOINT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let TransferLength: ULONG;
    let mut PortStatus: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();
    let mut HubStatus: USB_HUB_STATUS_AND_CHANGE = core::mem::zeroed();
    let Buffer: PVOID;
    let AddressBitMap: PULONG;
    let mut Port: ULONG;
    let Urb: PURB;
    let mut RHStatus: RHSTATUS = RH_STATUS_NO_CHANGES;
    let HubDescriptor: PUSB_HUB_DESCRIPTOR;
    let NumberOfPorts: UCHAR;

    // DPRINT("USBPORT_RootHubSCE: Transfer - %p\n", Transfer);

    Endpoint = (*Transfer).Endpoint;

    FdoExtension = (*(*Endpoint).FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoExtension = (*(*FdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    HubDescriptor = core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).Descriptor);
    NumberOfPorts = (*HubDescriptor).bNumberOfPorts;

    PortStatus.AsUlong32 = 0;
    HubStatus.AsUlong32 = 0;

    Urb = (*Transfer).Urb;
    TransferLength = (*Transfer).TransferParameters.TransferBufferLength;

    if TransferLength != 0 {
        Buffer = (*(*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL).MappedSystemVa;
    } else {
        Buffer = core::ptr::null_mut();
    }

    /* Check parameters */

    if Buffer.is_null() {
        /* Not valid parameter */
        // DPRINT1("USBPORT_RootHubSCE: Error! Buffer is NULL\n");
        return RH_STATUS_UNSUCCESSFUL;
    }

    if TransferLength < ((NumberOfPorts as ULONG) / 8 + 1) {
        /* Not valid parameters */
        // DPRINT1(...)
        return RH_STATUS_UNSUCCESSFUL;
    }

    RtlZeroMemory(Buffer, TransferLength as usize);

    AddressBitMap = Buffer as PULONG;

    /* Scan all the ports for changes */
    Port = 1;
    while Port <= NumberOfPorts as ULONG {
        DPRINT_CORE!(b"USBPORT_RootHubSCE: Port - %p\n\0".as_ptr() as *const i8, Port);

        /* Request the port status from miniport */
        if ((*Packet).RH_GetPortStatus.unwrap())(
            (*FdoExtension).MiniPortExt,
            (Port as u16),
            core::ptr::addr_of_mut!(PortStatus),
        ) != 0
        {
            /* Miniport returned an error */
            // DPRINT1("USBPORT_RootHubSCE: RH_GetPortStatus failed\n");
            return RH_STATUS_UNSUCCESSFUL;
        }

        if PortStatus.PortChange.Usb20PortChange.ConnectStatusChange() != 0
            || PortStatus.PortChange.Usb20PortChange.PortEnableDisableChange() != 0
            || PortStatus.PortChange.Usb20PortChange.SuspendChange() != 0
            || PortStatus.PortChange.Usb20PortChange.OverCurrentIndicatorChange() != 0
            || PortStatus.PortChange.Usb20PortChange.ResetChange() != 0
        {
            /* At the port status there is a change */
            *AddressBitMap.offset((Port >> 5) as isize) |= (1 << (Port & 0x1F)) as ULONG;
            RHStatus = RH_STATUS_SUCCESS;
        }

        Port += 1;
    }

    /* Request the hub status from miniport */
    if ((*Packet).RH_GetHubStatus.unwrap())(
        (*FdoExtension).MiniPortExt,
        core::ptr::addr_of_mut!(HubStatus),
    ) == 0
    {
        if HubStatus.HubChange.LocalPowerChange() == 1 || HubStatus.HubChange.OverCurrentChange() == 1
        {
            /* At the hub status there is a change */
            *AddressBitMap.offset(0) |= 1;
            RHStatus = RH_STATUS_SUCCESS;
        }

        if RHStatus == RH_STATUS_SUCCESS {
            /* Done */
            (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength = TransferLength;
            return RH_STATUS_SUCCESS;
        }

        if RHStatus == RH_STATUS_NO_CHANGES {
            /* No changes. Enable IRQs for miniport root hub */
            ((*Packet).RH_EnableIrq.unwrap())((*FdoExtension).MiniPortExt);
        }

        return RHStatus;
    }

    /* Miniport returned an error */
    // DPRINT1("USBPORT_RootHubSCE: RH_GetHubStatus failed\n");
    RH_STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubEndpointWorker(Endpoint: PUSBPORT_ENDPOINT) {

    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let Transfer: PUSBPORT_TRANSFER;
    let RHStatus: RHSTATUS;
    let USBDStatus: USBD_STATUS;
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE!(
        b"USBPORT_RootHubEndpointWorker: Endpoint - %p\n\0".as_ptr() as *const i8,
        Endpoint,
    );

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    if ((*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND) == 0 {
        ((*Packet).CheckController).unwrap()((*FdoExtension).MiniPortExt);
    }
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        OldIrql,
    );

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    Transfer = (((*Endpoint).TransferList.Flink as *mut u8)
        .offset(-(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink) as isize)))
        as PUSBPORT_TRANSFER;

    if IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList))
        || (*Endpoint).TransferList.Flink.is_null()
        || Transfer.is_null()
    {
        if (*Endpoint).StateLast == USBPORT_ENDPOINT_REMOVE {
            ExInterlockedInsertTailList(
                core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedList),
                core::ptr::addr_of_mut!((*Endpoint).CloseLink),
                core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock),
            );
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );

        USBPORT_FlushCancelList(Endpoint);
        return;
    }

    if ((*Transfer).Flags & (TRANSFER_FLAG_ABORTED | TRANSFER_FLAG_CANCELED)) != 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Transfer).TransferLink));
        InsertTailList(
            core::ptr::addr_of_mut!((*Endpoint).CancelList),
            core::ptr::addr_of_mut!((*Transfer).TransferLink),
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );
        USBPORT_FlushCancelList(Endpoint);
        return;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    if (*Endpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_CONTROL {
        RHStatus = USBPORT_RootHubEndpoint0(Transfer);
    } else {
        RHStatus = USBPORT_RootHubSCE(Transfer);
    }

    if RHStatus != RH_STATUS_NO_CHANGES {
        if RHStatus == RH_STATUS_SUCCESS {
            USBDStatus = USBD_STATUS_SUCCESS;
        } else {
            USBDStatus = USBD_STATUS_STALL_PID;
        }

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
        );
        USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );

        USBPORT_FlushCancelList(Endpoint);
        return;
    }

    USBPORT_FlushCancelList(Endpoint);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubCreateDevice(
    FdoDevice: PDEVICE_OBJECT,
    PdoDevice: PDEVICE_OBJECT,
) -> NTSTATUS {

    const USB_PORT_TAG: u32 = 0x70627375; // 'Uptp'
    const UsbFullSpeed: u32 = 0;
    const DEVICE_HANDLE_FLAG_ROOTHUB: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x10; verified vs usbport.h/usbmport.h
    const USB_DEVICE_DESCRIPTOR_TYPE: u8 = 0x01;
    const USB_CONFIGURATION_DESCRIPTOR_TYPE: u8 = 0x02;
    const USB_INTERFACE_DESCRIPTOR_TYPE: u8 = 0x04;
    const USB_ENDPOINT_DESCRIPTOR_TYPE: u8 = 0x05;
    const USB_DEVICE_CLASS_HUB: u8 = 0x09;
    const USB_CONFIG_SELF_POWERED: u8 = 0x40;
    const USB_ENDPOINT_TYPE_INTERRUPT: u8 = 0x03;
    const USB_ENDPOINT_TYPE_CONTROL: u8 = 0x00;
    const USB_MINIPORT_VERSION_OHCI: u32 = 1;
    const USB_MINIPORT_VERSION_UHCI: u32 = 2;
    const USB_MINIPORT_VERSION_EHCI: u32 = 3;
    const USB_MINIPORT_VERSION_XHCI: u32 = 4;
    const USB_20_HUB_DESCRIPTOR_TYPE: u8 = 0x29;
    const USB_30_HUB_DESCRIPTOR_TYPE: u8 = 0x2A;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut RootHubData: USBPORT_ROOT_HUB_DATA = core::mem::zeroed();
    let NumMaskByte: ULONG;
    let DescriptorsLength: ULONG;
    let Descriptors: PUSBPORT_RH_DESCRIPTORS;
    let RH_DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR;
    let RH_ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let RH_InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let RH_EndPointDescriptor: PUSB_ENDPOINT_DESCRIPTOR;
    let RH_HubDescriptor: PUSB_HUB_DESCRIPTOR;
    let mut ix: ULONG;
    let EndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR;
    let Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    DeviceHandle = core::ptr::addr_of_mut!((*PdoExtension).DeviceHandle);
    USBPORT_AddDeviceHandle(FdoDevice, DeviceHandle);

    InitializeListHead(core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList));

    (*DeviceHandle).IsRootHub = 1;
    (*DeviceHandle).DeviceSpeed = UsbFullSpeed as _;
    (*DeviceHandle).Flags = DEVICE_HANDLE_FLAG_ROOTHUB as _;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(RootHubData) as *mut _,
        core::mem::size_of::<USBPORT_ROOT_HUB_DATA>() as _,
    );

    ((*Packet).RH_GetRootHubData.unwrap())(
        (*FdoExtension).MiniPortExt,
        (core::ptr::addr_of_mut!(RootHubData) as *mut c_void),
    );

    NumMaskByte = ((RootHubData.NumberOfPorts as ULONG - 1) / 8) + 1;

    DescriptorsLength = (core::mem::size_of::<USB_DEVICE_DESCRIPTOR>()
        + core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
        + core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()
        + core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>()
        + (core::mem::size_of::<USB_HUB_DESCRIPTOR>() + 2 * NumMaskByte as usize))
        as ULONG;

    Descriptors = ExAllocatePoolWithTag(NonPagedPool, DescriptorsLength as _, USB_PORT_TAG)
        as PUSBPORT_RH_DESCRIPTORS;

    if !Descriptors.is_null() {
        RtlZeroMemory(Descriptors as *mut _, DescriptorsLength as _);

        (*PdoExtension).RootHubDescriptors = Descriptors;

        RH_DeviceDescriptor =
            core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).DeviceDescriptor);

        (*RH_DeviceDescriptor).bLength = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as u8;
        (*RH_DeviceDescriptor).bDescriptorType = USB_DEVICE_DESCRIPTOR_TYPE;
        (*RH_DeviceDescriptor).bcdUSB = 0x100;
        (*RH_DeviceDescriptor).bDeviceClass = USB_DEVICE_CLASS_HUB;
        (*RH_DeviceDescriptor).bDeviceSubClass = 0x01;
        (*RH_DeviceDescriptor).bDeviceProtocol = 0x00;
        (*RH_DeviceDescriptor).bMaxPacketSize0 = 0x08;
        (*RH_DeviceDescriptor).idVendor = (*FdoExtension).VendorID;
        (*RH_DeviceDescriptor).idProduct = (*FdoExtension).DeviceID;
        (*RH_DeviceDescriptor).bcdDevice = ((*FdoExtension).RevisionID as u16);
        (*RH_DeviceDescriptor).iManufacturer = 0x00;
        (*RH_DeviceDescriptor).iProduct = 0x00;
        (*RH_DeviceDescriptor).iSerialNumber = 0x00;
        (*RH_DeviceDescriptor).bNumConfigurations = 0x01;

        RH_ConfigurationDescriptor =
            core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).ConfigDescriptor);

        (*RH_ConfigurationDescriptor).bLength =
            core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>() as u8;
        (*RH_ConfigurationDescriptor).bDescriptorType = USB_CONFIGURATION_DESCRIPTOR_TYPE;

        (*RH_ConfigurationDescriptor).wTotalLength =
            (core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>()
                + core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()
                + core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>()) as u16;

        (*RH_ConfigurationDescriptor).bNumInterfaces = 0x01;
        (*RH_ConfigurationDescriptor).bConfigurationValue = 0x01;
        (*RH_ConfigurationDescriptor).iConfiguration = 0x00;
        (*RH_ConfigurationDescriptor).bmAttributes = USB_CONFIG_SELF_POWERED;
        (*RH_ConfigurationDescriptor).MaxPower = 0x00;

        RH_InterfaceDescriptor =
            core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).InterfaceDescriptor);

        (*RH_InterfaceDescriptor).bLength = core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>() as u8;
        (*RH_InterfaceDescriptor).bDescriptorType = USB_INTERFACE_DESCRIPTOR_TYPE;
        (*RH_InterfaceDescriptor).bInterfaceNumber = 0x00;
        (*RH_InterfaceDescriptor).bAlternateSetting = 0x00;
        (*RH_InterfaceDescriptor).bNumEndpoints = 0x01;
        (*RH_InterfaceDescriptor).bInterfaceClass = USB_DEVICE_CLASS_HUB;
        (*RH_InterfaceDescriptor).bInterfaceSubClass = 0x01;
        (*RH_InterfaceDescriptor).bInterfaceProtocol = 0x00;
        (*RH_InterfaceDescriptor).iInterface = 0x00;

        RH_EndPointDescriptor =
            core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).EndPointDescriptor);

        (*RH_EndPointDescriptor).bLength = core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>() as u8;
        (*RH_EndPointDescriptor).bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;
        (*RH_EndPointDescriptor).bEndpointAddress = 0x81;
        (*RH_EndPointDescriptor).bmAttributes = USB_ENDPOINT_TYPE_INTERRUPT; // SCE endpoint
        (*RH_EndPointDescriptor).wMaxPacketSize = 0x0008;
        (*RH_EndPointDescriptor).bInterval = 0x0C; // 12 msec

        RH_HubDescriptor =
            core::ptr::addr_of_mut!((*(*PdoExtension).RootHubDescriptors).Descriptor);

        (*RH_HubDescriptor).bDescriptorLength =
            (core::mem::offset_of!(USB_HUB_DESCRIPTOR, bRemoveAndPowerMask) + 2 * NumMaskByte as usize)
                as u8;

        if (*Packet).MiniPortVersion == USB_MINIPORT_VERSION_OHCI
            || (*Packet).MiniPortVersion == USB_MINIPORT_VERSION_UHCI
            || (*Packet).MiniPortVersion == USB_MINIPORT_VERSION_EHCI
        {
            (*RH_HubDescriptor).bDescriptorType = USB_20_HUB_DESCRIPTOR_TYPE;
        } else if (*Packet).MiniPortVersion == USB_MINIPORT_VERSION_XHCI {
            // usbxhci's root hub exposes the controller's USB2-protocol ports with
            // USB 2.0 hub semantics (wPortStatus bits, not the USB 3.0 layout), and
            // this descriptor is built with the USB 2.0 layout, so it must say so.
            // The C used the USB 3.0 type here for a SuperSpeed root hub that no
            // miniport implements.
            (*RH_HubDescriptor).bDescriptorType = USB_20_HUB_DESCRIPTOR_TYPE;
        } else {
            DbgBreakPoint();
        }

        (*RH_HubDescriptor).bNumberOfPorts = (RootHubData.NumberOfPorts as u8);
        (*RH_HubDescriptor).wHubCharacteristics = RootHubData.HubCharacteristics.AsUSHORT;
        (*RH_HubDescriptor).bPowerOnToPowerGood = (RootHubData.PowerOnToPowerGood as u8);
        (*RH_HubDescriptor).bHubControlCurrent = (RootHubData.HubControlCurrent as u8);

        ix = 0;
        while ix < NumMaskByte {
            (*RH_HubDescriptor).bRemoveAndPowerMask[ix as usize] = 0;
            (*RH_HubDescriptor).bRemoveAndPowerMask[(ix + 1) as usize] = -1i32 as u8;
            ix += 2;
        }

        EndpointDescriptor =
            core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle.EndpointDescriptor);

        (*EndpointDescriptor).bLength = core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>() as u8;
        (*EndpointDescriptor).bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;
        (*EndpointDescriptor).bEndpointAddress = 0x00;
        (*EndpointDescriptor).bmAttributes = USB_ENDPOINT_TYPE_CONTROL;
        (*EndpointDescriptor).wMaxPacketSize = 0x0040;
        (*EndpointDescriptor).bInterval = 0x00;

        Status = USBPORT_OpenPipe(
            FdoDevice,
            DeviceHandle,
            core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle),
            core::ptr::null_mut(),
        );
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InvalidateRootHub(MiniPortExtension: PVOID) -> ULONG {

    const USBPORT_FLAG_HC_SUSPEND: u32 = 0x00000100; // FIXED 2026-07-05: translation invented 0x80; verified vs usbport.h/usbmport.h
    const USBPORT_FLAG_HC_WAKE_SUPPORT: u32 = 0x00200000; // FIXED 2026-07-05: translation invented 0x40000; verified vs usbport.h/usbmport.h
    const USBPORT_MPFLAG_SUSPENDED: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_TMFLAG_WAKE: u32 = 0x00000020; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h
    const INVALIDATE_ENDPOINT_WORKER_THREAD: u32 = 1;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let FdoDevice: PDEVICE_OBJECT;
    let PdoDevice: PDEVICE_OBJECT;
    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut Endpoint: PUSBPORT_ENDPOINT = core::ptr::null_mut();

    FdoExtension = ((MiniPortExtension as ULONG_PTR)
        - core::mem::size_of::<USBPORT_DEVICE_EXTENSION>() as ULONG_PTR)
        as PUSBPORT_DEVICE_EXTENSION;

    FdoDevice = (*FdoExtension).CommonExtension.SelfDevice;

    if (*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND != 0
        && (*FdoExtension).Flags & USBPORT_FLAG_HC_WAKE_SUPPORT != 0
        && (*FdoExtension).MiniPortFlags & USBPORT_MPFLAG_SUSPENDED != 0
        && (*FdoExtension).TimerFlags & USBPORT_TMFLAG_WAKE != 0
    {
        USBPORT_HcQueueWakeDpc(FdoDevice);
        return 0;
    }

    ((*(*FdoExtension).MiniPortInterface).Packet.RH_DisableIrq).unwrap()(
        (*FdoExtension).MiniPortExt,
    );

    PdoDevice = (*FdoExtension).RootHubPdo;

    if !PdoDevice.is_null() {
        PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
        Endpoint = (*PdoExtension).Endpoint;

        if !Endpoint.is_null() {
            USBPORT_InvalidateEndpointHandler(
                FdoDevice,
                (*PdoExtension).Endpoint,
                INVALIDATE_ENDPOINT_WORKER_THREAD,
            );
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RootHubPowerAndChirpAllCcPorts(FdoDevice: PDEVICE_OBJECT) {

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut RootHubData: USBPORT_ROOT_HUB_DATA = core::mem::zeroed();
    let mut Port: ULONG;
    let CompanionControllersList: PDEVICE_RELATIONS;
    let mut CompanionFdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut CompanionPacket: PUSBPORT_REGISTRATION_PACKET;
    let mut CompanionPorts: ULONG;
    let mut NumController: ULONG;
    let mut Entry: *mut PDEVICE_OBJECT;
    let NumPorts: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    RtlZeroMemory(
        core::ptr::addr_of_mut!(RootHubData) as *mut core::ffi::c_void,
        (core::mem::size_of::<USBPORT_ROOT_HUB_DATA>() as ULONG as usize),
    );

    ((*Packet).RH_GetRootHubData.unwrap())(
        (*FdoExtension).MiniPortExt,
        (core::ptr::addr_of_mut!(RootHubData) as *mut c_void),
    );

    NumPorts = RootHubData.NumberOfPorts;

    Port = 1;
    while Port <= NumPorts {
        ((*Packet).RH_SetFeaturePortPower.unwrap())((*FdoExtension).MiniPortExt, (Port as u16));
        Port += 1;
    }

    USBPORT_Wait((FdoDevice) as _, 10);

    CompanionControllersList = USBPORT_FindCompanionControllers(FdoDevice, 0, 1);

    if !CompanionControllersList.is_null() {
        Entry = core::ptr::addr_of_mut!((*CompanionControllersList).Objects[0]);

        NumController = 0;
        while NumController < (*CompanionControllersList).Count {
            CompanionFdoExtension =
                (**Entry).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

            // The COMPANION's miniport, not ours (upstream da13f3ca820): the EHCI packet's
            // RH_* routines were being handed an OHCI/UHCI MiniPortExt.
            CompanionPacket = core::ptr::addr_of_mut!((*(*CompanionFdoExtension).MiniPortInterface).Packet);

            ((*CompanionPacket).RH_GetRootHubData.unwrap())(
                (*CompanionFdoExtension).MiniPortExt,
                (core::ptr::addr_of_mut!(RootHubData) as *mut c_void),
            );

            CompanionPorts = RootHubData.NumberOfPorts;

            Port = 1;
            while Port <= CompanionPorts {
                ((*CompanionPacket).RH_SetFeaturePortPower.unwrap())(
                    (*CompanionFdoExtension).MiniPortExt,
                    (Port as u16),
                );
                Port += 1;
            }

            Entry = Entry.offset(1);

            NumController += 1;
        }

        ExFreePoolWithTag(
            CompanionControllersList as *mut core::ffi::c_void,
            USB_PORT_TAG,
        );
    }

    USBPORT_Wait((FdoDevice) as _, 100);

    Port = 1;
    while Port <= NumPorts {
        if ((*(*FdoExtension).MiniPortInterface).Version) < 200 {
            break;
        }

        InterlockedIncrement(core::ptr::addr_of_mut!((*FdoExtension).ChirpRootPortLock) as *mut i32);
        ((*Packet).RH_ChirpRootPort.unwrap())((*FdoExtension).MiniPortExt, (Port as u16));
        InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).ChirpRootPortLock) as *mut i32);

        Port += 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort root hub implementation
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
// RHSTATUS
// NTAPI
// USBPORT_MPStatusToRHStatus(IN MPSTATUS MPStatus)
// {
//     RHSTATUS RHStatus = RH_STATUS_SUCCESS;
//
//     //DPRINT("USBPORT_MPStatusToRHStatus: MPStatus - %x\n", MPStatus);
//
//     if (MPStatus)
//     {
//         RHStatus = (MPStatus != MP_STATUS_FAILURE);
//         ++RHStatus;
//     }
//
//     return RHStatus;
// }
//
// MPSTATUS
// NTAPI
// USBPORT_RH_SetFeatureUSB2PortPower(IN PDEVICE_OBJECT FdoDevice,
//                                    IN USHORT Port)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PDEVICE_RELATIONS CompanionControllersList;
//     PUSBPORT_REGISTRATION_PACKET CompanionPacket;
//     PDEVICE_OBJECT CompanionFdoDevice;
//     PUSBPORT_DEVICE_EXTENSION CompanionFdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     USHORT ix;
//     PDEVICE_OBJECT * Entry;
//     ULONG NumController = 0;
//
//     DPRINT("USBPORT_RootHub_PowerUsb2Port: FdoDevice - %p, Port - %p\n",
//            FdoDevice,
//            Port);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     CompanionControllersList = USBPORT_FindCompanionControllers(FdoDevice,
//                                                                 FALSE,
//                                                                 TRUE);
//
//     if (!CompanionControllersList)
//     {
//         Packet->RH_SetFeaturePortPower(FdoExtension->MiniPortExt, Port);
//         return MP_STATUS_SUCCESS;
//     }
//
//     Entry = &CompanionControllersList->Objects[0];
//
//     while (NumController < CompanionControllersList->Count)
//     {
//         CompanionFdoDevice = *Entry;
//
//         CompanionFdoExtension = CompanionFdoDevice->DeviceExtension;
//         CompanionPacket = &CompanionFdoExtension->MiniPortInterface->Packet;
//
//         PdoExtension = CompanionFdoExtension->RootHubPdo->DeviceExtension;
//
//         for (ix = 0;
//              (PdoExtension->CommonExtension.PnpStateFlags & USBPORT_PNP_STATE_STARTED) &&
//               ix < PdoExtension->RootHubDescriptors->Descriptor.bNumberOfPorts;
//              ++ix)
//         {
//             CompanionPacket->RH_SetFeaturePortPower(CompanionFdoExtension->MiniPortExt,
//                                                     ix + 1);
//         }
//
//         ++NumController;
//         ++Entry;
//     }
//
//     Packet->RH_SetFeaturePortPower(FdoExtension->MiniPortExt, Port);
//
//     if (CompanionControllersList)
//     {
//         ExFreePoolWithTag(CompanionControllersList, USB_PORT_TAG);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// RHSTATUS
// NTAPI
// USBPORT_RootHubClassCommand(IN PDEVICE_OBJECT FdoDevice,
//                             IN PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket,
//                             IN PVOID Buffer,
//                             IN PULONG BufferLength)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     USHORT Port;
//     USHORT Feature;
//     MPSTATUS MPStatus;
//     RHSTATUS RHStatus = RH_STATUS_UNSUCCESSFUL;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_RootHubClassCommand: USB command - %x, *BufferLength - %x\n",
//            SetupPacket->bRequest,
//            *BufferLength);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoExtension = FdoExtension->RootHubPdo->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     Port = SetupPacket->wIndex.W;
//
//     switch (SetupPacket->bRequest)
//     {
//         case USB_REQUEST_GET_STATUS:
//         {
//             if (!Buffer)
//             {
//                 return RHStatus;
//             }
//
//             *(PULONG)Buffer = 0;
//
//             if (SetupPacket->bmRequestType.Recipient == BMREQUEST_TO_OTHER)
//             {
//                 ASSERT(*BufferLength >= 4);
//
//                 if (Port > PdoExtension->RootHubDescriptors->Descriptor.bNumberOfPorts ||
//                     Port <= 0  ||
//                     SetupPacket->wLength < 4)
//                 {
//                     return RHStatus;
//                 }
//
//                 KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//                 MPStatus = Packet->RH_GetPortStatus(FdoExtension->MiniPortExt,
//                                                     SetupPacket->wIndex.W,
//                                                     Buffer);
//
//                 KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//             }
//             else
//             {
//                 KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//                 MPStatus = Packet->RH_GetHubStatus(FdoExtension->MiniPortExt,
//                                                    Buffer);
//
//                 KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//             }
//
//             RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
//             break;
//         }
//
//         case USB_REQUEST_CLEAR_FEATURE:
//             Feature = SetupPacket->wValue.W;
//
//             if ((SetupPacket->bmRequestType.Recipient) != USBPORT_RECIPIENT_PORT)
//             {
//                 if (Feature == FEATURE_C_HUB_LOCAL_POWER)
//                 {
//                     RHStatus = RH_STATUS_SUCCESS;
//                     return RHStatus;
//                 }
//
//                 if (Feature == FEATURE_C_HUB_OVER_CURRENT)
//                 {
//                     MPStatus = Packet->RH_ClearFeaturePortOvercurrentChange(FdoExtension->MiniPortExt,
//                                                                             0);
//                     RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
//                     return RHStatus;
//                 }
//
//                 DbgBreakPoint();
//                 return RHStatus;
//             }
//
//             switch (Feature)
//             {
//                 case FEATURE_PORT_ENABLE:
//                     MPStatus = Packet->RH_ClearFeaturePortEnable(FdoExtension->MiniPortExt,
//                                                                  Port);
//                     break;
//
//                 case FEATURE_PORT_SUSPEND:
//                     MPStatus = Packet->RH_ClearFeaturePortSuspend(FdoExtension->MiniPortExt,
//                                                                   Port);
//                     break;
//
//                 case FEATURE_PORT_POWER:
//                     MPStatus = Packet->RH_ClearFeaturePortPower(FdoExtension->MiniPortExt,
//                                                                 Port);
//                     break;
//
//                 case FEATURE_C_PORT_CONNECTION:
//                     MPStatus = Packet->RH_ClearFeaturePortConnectChange(FdoExtension->MiniPortExt,
//                                                                         Port);
//                     break;
//
//                 case FEATURE_C_PORT_ENABLE:
//                      MPStatus = Packet->RH_ClearFeaturePortEnableChange(FdoExtension->MiniPortExt,
//                                                                         Port);
//                     break;
//
//                 case FEATURE_C_PORT_SUSPEND:
//                     MPStatus = Packet->RH_ClearFeaturePortSuspendChange(FdoExtension->MiniPortExt,
//                                                                         Port);
//                     break;
//
//                 case FEATURE_C_PORT_OVER_CURRENT:
//                     MPStatus = Packet->RH_ClearFeaturePortOvercurrentChange(FdoExtension->MiniPortExt,
//                                                                             Port);
//                     break;
//
//                 case FEATURE_C_PORT_RESET:
//                     MPStatus = Packet->RH_ClearFeaturePortResetChange(FdoExtension->MiniPortExt,
//                                                                       Port);
//                     break;
//
//                 default:
//                     DPRINT1("USBPORT_RootHubClassCommand: Not supported feature - %x\n",
//                             Feature);
//                     return RHStatus;
//             }
//
//             RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
//             break;
//
//         case USB_REQUEST_SET_FEATURE:
//             if (SetupPacket->bmRequestType.Recipient != USBPORT_RECIPIENT_PORT)
//             {
//                 return RHStatus;
//             }
//
//             Feature = SetupPacket->wValue.W;
//             switch (Feature)
//             {
//                 case FEATURE_PORT_ENABLE:
//                     MPStatus = Packet->RH_SetFeaturePortEnable(FdoExtension->MiniPortExt,
//                                                                Port);
//                     break;
//
//                 case FEATURE_PORT_SUSPEND:
//                     MPStatus = Packet->RH_SetFeaturePortSuspend(FdoExtension->MiniPortExt,
//                                                                 Port);
//                     break;
//
//                 case FEATURE_PORT_RESET:
//                     MPStatus = Packet->RH_SetFeaturePortReset(FdoExtension->MiniPortExt,
//                                                               Port);
//                     break;
//
//                 case FEATURE_PORT_POWER:
//                     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//                     {
//                         MPStatus = USBPORT_RH_SetFeatureUSB2PortPower(FdoDevice, Port);
//                     }
//                     else
//                     {
//                         MPStatus = Packet->RH_SetFeaturePortPower(FdoExtension->MiniPortExt,
//                                                                   Port);
//                     }
//
//                     break;
//
//                 default:
//                     DPRINT1("USBPORT_RootHubClassCommand: Not supported feature - %x\n",
//                             Feature);
//                     return RHStatus;
//             }
//
//             RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
//             break;
//
//         case USB_REQUEST_GET_DESCRIPTOR:
//             if (Buffer &&
//                 SetupPacket->wValue.W == 0 &&
//                 SetupPacket->bmRequestType.Dir == BMREQUEST_DEVICE_TO_HOST)
//             {
//                 SIZE_T DescriptorLength;
//
//                 DescriptorLength = PdoExtension->RootHubDescriptors->Descriptor.bDescriptorLength;
//
//                 if (*BufferLength < DescriptorLength)
//                     DescriptorLength = *BufferLength;
//
//                 RtlCopyMemory(Buffer,
//                               &PdoExtension->RootHubDescriptors->Descriptor,
//                               DescriptorLength);
//
//                 *BufferLength = DescriptorLength;
//                 RHStatus = RH_STATUS_SUCCESS;
//             }
//
//             break;
//
//         default:
//             DPRINT1("USBPORT_RootHubClassCommand: Not supported USB request - %x\n",
//                     SetupPacket->bRequest);
//             //USB_REQUEST_SET_ADDRESS                   0x05
//             //USB_REQUEST_SET_DESCRIPTOR                0x07
//             //USB_REQUEST_GET_CONFIGURATION             0x08
//             //USB_REQUEST_SET_CONFIGURATION             0x09
//             //USB_REQUEST_GET_INTERFACE                 0x0A
//             //USB_REQUEST_SET_INTERFACE                 0x0B
//             //USB_REQUEST_SYNC_FRAME                    0x0C
//             break;
//     }
//
//     return RHStatus;
// }
//
// RHSTATUS
// NTAPI
// USBPORT_RootHubStandardCommand(IN PDEVICE_OBJECT FdoDevice,
//                                IN PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket,
//                                IN PVOID Buffer,
//                                IN OUT PULONG TransferLength)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     SIZE_T Length;
//     PVOID Descriptor;
//     SIZE_T DescriptorLength;
//     MPSTATUS MPStatus;
//     RHSTATUS RHStatus = RH_STATUS_UNSUCCESSFUL;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_RootHubStandardCommand: USB command - %x, TransferLength - %p\n",
//            SetupPacket->bRequest,
//            TransferLength);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoExtension = FdoExtension->RootHubPdo->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     switch (SetupPacket->bRequest)
//     {
//         case USB_REQUEST_GET_DESCRIPTOR:
//             if (SetupPacket->wValue.LowByte ||
//                 !(SetupPacket->bmRequestType.Dir))
//             {
//                 return RHStatus;
//             }
//
//             switch (SetupPacket->wValue.HiByte)
//             {
//                 case USB_DEVICE_DESCRIPTOR_TYPE:
//                     Descriptor = &PdoExtension->RootHubDescriptors->DeviceDescriptor;
//                     DescriptorLength = sizeof(USB_DEVICE_DESCRIPTOR);
//                     break;
//
//                 case USB_CONFIGURATION_DESCRIPTOR_TYPE:
//                     Descriptor = &PdoExtension->RootHubDescriptors->ConfigDescriptor;
//                     DescriptorLength = sizeof(USB_CONFIGURATION_DESCRIPTOR) +
//                                        sizeof(USB_INTERFACE_DESCRIPTOR) +
//                                        sizeof(USB_ENDPOINT_DESCRIPTOR);
//                     break;
//
//                 default:
//                     DPRINT1("USBPORT_RootHubStandardCommand: Not supported Descriptor Type - %x\n",
//                             SetupPacket->wValue.HiByte);
//                     return RHStatus;
//             }
//
//             if (!Descriptor)
//             {
//                 return RHStatus;
//             }
//
//             if (*TransferLength >= DescriptorLength)
//                 Length = DescriptorLength;
//             else
//                 Length = *TransferLength;
//
//             RtlCopyMemory(Buffer, Descriptor, Length);
//             *TransferLength = Length;
//
//             RHStatus = RH_STATUS_SUCCESS;
//             break;
//
//         case USB_REQUEST_GET_STATUS:
//             if (!SetupPacket->wValue.W &&
//                  SetupPacket->wLength == sizeof(USHORT) &&
//                  !SetupPacket->wIndex.W &&
//                  SetupPacket->bmRequestType.Dir == BMREQUEST_DEVICE_TO_HOST)
//             {
//                 KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//                 MPStatus = Packet->RH_GetStatus(FdoExtension->MiniPortExt,
//                                                 Buffer);
//
//                 KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//                 *TransferLength = sizeof(USHORT);
//                 RHStatus = USBPORT_MPStatusToRHStatus(MPStatus);
//             }
//
//             break;
//
//         case USB_REQUEST_GET_CONFIGURATION:
//             if (SetupPacket->wValue.W ||
//                 SetupPacket->wIndex.W ||
//                 SetupPacket->wLength != 1 ||
//                 SetupPacket->bmRequestType.Dir == BMREQUEST_HOST_TO_DEVICE)
//             {
//                 return RHStatus;
//             }
//
//             Length = 0;
//
//             if (*TransferLength >= 1)
//             {
//                 Length = 1;
//                 RtlCopyMemory(Buffer, &PdoExtension->ConfigurationValue, Length);
//             }
//
//             *TransferLength = Length;
//
//             RHStatus = RH_STATUS_SUCCESS;
//             break;
//
//         case USB_REQUEST_SET_CONFIGURATION:
//             if (!SetupPacket->wIndex.W &&
//                 !SetupPacket->wLength &&
//                 !(SetupPacket->bmRequestType.Dir == BMREQUEST_DEVICE_TO_HOST))
//             {
//                 if (SetupPacket->wValue.W == 0 ||
//                     SetupPacket->wValue.W ==
//                         PdoExtension->RootHubDescriptors->ConfigDescriptor.bConfigurationValue)
//                 {
//                   PdoExtension->ConfigurationValue = SetupPacket->wValue.LowByte;
//                   RHStatus = RH_STATUS_SUCCESS;
//                 }
//             }
//
//             break;
//
//         case USB_REQUEST_SET_ADDRESS:
//             if (!SetupPacket->wIndex.W &&
//                 !SetupPacket->wLength &&
//                 !(SetupPacket->bmRequestType.Dir))
//             {
//                 PdoExtension->DeviceHandle.DeviceAddress = SetupPacket->wValue.LowByte;
//                 RHStatus = RH_STATUS_SUCCESS;
//                 break;
//             }
//
//             break;
//
//         default:
//             DPRINT1("USBPORT_RootHubStandardCommand: Not supported USB request - %x\n",
//                     SetupPacket->bRequest);
//             //USB_REQUEST_CLEAR_FEATURE                 0x01
//             //USB_REQUEST_SET_FEATURE                   0x03
//             //USB_REQUEST_SET_DESCRIPTOR                0x07
//             //USB_REQUEST_GET_INTERFACE                 0x0A
//             //USB_REQUEST_SET_INTERFACE                 0x0B
//             //USB_REQUEST_SYNC_FRAME                    0x0C
//             break;
//     }
//
//     return RHStatus;
// }
//
// RHSTATUS
// NTAPI
// USBPORT_RootHubEndpoint0(IN PUSBPORT_TRANSFER Transfer)
// {
//     PDEVICE_OBJECT FdoDevice;
//     ULONG TransferLength;
//     PVOID Buffer;
//     PURB Urb;
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     UCHAR Type;
//     RHSTATUS RHStatus;
//
//     DPRINT("USBPORT_RootHubEndpoint0: Transfer - %p\n", Transfer);
//
//     TransferLength = Transfer->TransferParameters.TransferBufferLength;
//     Urb = Transfer->Urb;
//     FdoDevice = Transfer->Endpoint->FdoDevice;
//
//     if (TransferLength > 0)
//         Buffer = Urb->UrbControlTransfer.TransferBufferMDL->MappedSystemVa;
//     else
//         Buffer = NULL;
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)Urb->UrbControlTransfer.SetupPacket;
//
//     Type = SetupPacket->bmRequestType.Type;
//
//     if (Type == BMREQUEST_STANDARD)
//     {
//         RHStatus = USBPORT_RootHubStandardCommand(FdoDevice,
//                                                   SetupPacket,
//                                                   Buffer,
//                                                   &TransferLength);
//     }
//     else if (Type == BMREQUEST_CLASS)
//     {
//         RHStatus = USBPORT_RootHubClassCommand(FdoDevice,
//                                                SetupPacket,
//                                                Buffer,
//                                                &TransferLength);
//     }
//     else
//     {
//         return RH_STATUS_UNSUCCESSFUL;
//     }
//
//     if (RHStatus == RH_STATUS_SUCCESS)
//         Transfer->CompletedTransferLen = TransferLength;
//
//     return RHStatus;
// }
//
// RHSTATUS
// NTAPI
// USBPORT_RootHubSCE(IN PUSBPORT_TRANSFER Transfer)
// {
//     PUSBPORT_ENDPOINT Endpoint;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     ULONG TransferLength;
//     USB_PORT_STATUS_AND_CHANGE PortStatus;
//     USB_HUB_STATUS_AND_CHANGE HubStatus;
//     PVOID Buffer;
//     PULONG AddressBitMap;
//     ULONG Port;
//     PURB Urb;
//     RHSTATUS RHStatus = RH_STATUS_NO_CHANGES;
//     PUSB_HUB_DESCRIPTOR HubDescriptor;
//     UCHAR NumberOfPorts;
//
//     DPRINT("USBPORT_RootHubSCE: Transfer - %p\n", Transfer);
//
//     Endpoint = Transfer->Endpoint;
//
//     FdoExtension = Endpoint->FdoDevice->DeviceExtension;
//     PdoExtension = FdoExtension->RootHubPdo->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     HubDescriptor = &PdoExtension->RootHubDescriptors->Descriptor;
//     NumberOfPorts = HubDescriptor->bNumberOfPorts;
//
//     PortStatus.AsUlong32 = 0;
//     HubStatus.AsUlong32 = 0;
//
//     Urb = Transfer->Urb;
//     TransferLength = Transfer->TransferParameters.TransferBufferLength;
//
//     if (TransferLength)
//     {
//         Buffer = Urb->UrbControlTransfer.TransferBufferMDL->MappedSystemVa;
//     }
//     else
//     {
//         Buffer = NULL;
//     }
//
//     /* Check parameters */
//
//     if (!Buffer)
//     {
//         /* Not valid parameter */
//         DPRINT1("USBPORT_RootHubSCE: Error! Buffer is NULL\n");
//         return RH_STATUS_UNSUCCESSFUL;
//     }
//
//     if ((TransferLength < (NumberOfPorts / 8 + 1)))
//     {
//         /* Not valid parameters */
//         DPRINT1("USBPORT_RootHubSCE: Error! TransferLength - %x, NumberOfPorts - %x\n",
//                 TransferLength,
//                 NumberOfPorts);
//
//         return RH_STATUS_UNSUCCESSFUL;
//     }
//
//     RtlZeroMemory(Buffer, TransferLength);
//
//     AddressBitMap = Buffer;
//
//     /* Scan all the ports for changes */
//     for (Port = 1; Port <= NumberOfPorts; Port++)
//     {
//         DPRINT_CORE!("USBPORT_RootHubSCE: Port - %p\n", Port);
//
//         /* Request the port status from miniport */
//         if (Packet->RH_GetPortStatus(FdoExtension->MiniPortExt,
//                                      Port,
//                                      &PortStatus))
//         {
//             /* Miniport returned an error */
//             DPRINT1("USBPORT_RootHubSCE: RH_GetPortStatus failed\n");
//             return RH_STATUS_UNSUCCESSFUL;
//         }
//
//         if (PortStatus.PortChange.Usb20PortChange.ConnectStatusChange ||
//             PortStatus.PortChange.Usb20PortChange.PortEnableDisableChange ||
//             PortStatus.PortChange.Usb20PortChange.SuspendChange ||
//             PortStatus.PortChange.Usb20PortChange.OverCurrentIndicatorChange ||
//             PortStatus.PortChange.Usb20PortChange.ResetChange)
//         {
//             /* At the port status there is a change */
//             AddressBitMap[Port >> 5] |= 1 << (Port & 0x1F);
//             RHStatus = RH_STATUS_SUCCESS;
//         }
//     }
//
//     /* Request the hub status from miniport */
//     if (!Packet->RH_GetHubStatus(FdoExtension->MiniPortExt, &HubStatus))
//     {
//         if (HubStatus.HubChange.LocalPowerChange == 1 ||
//             HubStatus.HubChange.OverCurrentChange == 1)
//         {
//             /* At the hub status there is a change */
//             AddressBitMap[0] |= 1;
//             RHStatus = RH_STATUS_SUCCESS;
//         }
//
//         if (RHStatus == RH_STATUS_SUCCESS)
//         {
//             /* Done */
//             Urb->UrbControlTransfer.TransferBufferLength = TransferLength;
//             return RH_STATUS_SUCCESS;
//         }
//
//         if (RHStatus == RH_STATUS_NO_CHANGES)
//         {
//             /* No changes. Enable IRQs for miniport root hub */
//             Packet->RH_EnableIrq(FdoExtension->MiniPortExt);
//         }
//
//         return RHStatus;
//     }
//
//     /* Miniport returned an error */
//     DPRINT1("USBPORT_RootHubSCE: RH_GetHubStatus failed\n");
//     return RH_STATUS_UNSUCCESSFUL;
// }
//
// VOID
// NTAPI
// USBPORT_RootHubEndpointWorker(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_TRANSFER Transfer;
//     RHSTATUS RHStatus;
//     USBD_STATUS USBDStatus;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_RootHubEndpointWorker: Endpoint - %p\n", Endpoint);
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//     if (!(FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND))
//     {
//         Packet->CheckController(FdoExtension->MiniPortExt);
//     }
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     Transfer = CONTAINING_RECORD(Endpoint->TransferList.Flink,
//                                  USBPORT_TRANSFER,
//                                  TransferLink);
//
//     if (IsListEmpty(&Endpoint->TransferList) ||
//         Endpoint->TransferList.Flink == NULL ||
//         !Transfer)
//     {
//         if (Endpoint->StateLast == USBPORT_ENDPOINT_REMOVE)
//         {
//             ExInterlockedInsertTailList(&FdoExtension->EndpointClosedList,
//                                         &Endpoint->CloseLink,
//                                         &FdoExtension->EndpointClosedSpinLock);
//        }
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//         USBPORT_FlushCancelList(Endpoint);
//         return;
//     }
//
//     if (Transfer->Flags & (TRANSFER_FLAG_ABORTED | TRANSFER_FLAG_CANCELED))
//     {
//         RemoveEntryList(&Transfer->TransferLink);
//         InsertTailList(&Endpoint->CancelList, &Transfer->TransferLink);
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//         USBPORT_FlushCancelList(Endpoint);
//         return;
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     if (Endpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//         RHStatus = USBPORT_RootHubEndpoint0(Transfer);
//     else
//         RHStatus = USBPORT_RootHubSCE(Transfer);
//
//     if (RHStatus != RH_STATUS_NO_CHANGES)
//     {
//         if (RHStatus == RH_STATUS_SUCCESS)
//             USBDStatus = USBD_STATUS_SUCCESS;
//         else
//             USBDStatus = USBD_STATUS_STALL_PID;
//
//         KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//         USBPORT_QueueDoneTransfer(Transfer, USBDStatus);
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//         USBPORT_FlushCancelList(Endpoint);
//         return;
//     }
//
//     USBPORT_FlushCancelList(Endpoint);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_RootHubCreateDevice(IN PDEVICE_OBJECT FdoDevice,
//                             IN PDEVICE_OBJECT PdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     USBPORT_ROOT_HUB_DATA RootHubData;
//     ULONG NumMaskByte;
//     ULONG DescriptorsLength;
//     PUSBPORT_RH_DESCRIPTORS Descriptors;
//     PUSB_DEVICE_DESCRIPTOR RH_DeviceDescriptor;
//     PUSB_CONFIGURATION_DESCRIPTOR RH_ConfigurationDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR RH_InterfaceDescriptor;
//     PUSB_ENDPOINT_DESCRIPTOR RH_EndPointDescriptor;
//     PUSB_HUB_DESCRIPTOR RH_HubDescriptor;
//     ULONG ix;
//     PUSB_ENDPOINT_DESCRIPTOR EndpointDescriptor;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_RootHubCreateDevice: FdoDevice - %p, PdoDevice - %p\n",
//            FdoDevice,
//            PdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     PdoExtension = PdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     DeviceHandle = &PdoExtension->DeviceHandle;
//     USBPORT_AddDeviceHandle(FdoDevice, DeviceHandle);
//
//     InitializeListHead(&DeviceHandle->PipeHandleList);
//
//     DeviceHandle->IsRootHub = TRUE;
//     DeviceHandle->DeviceSpeed = UsbFullSpeed;
//     DeviceHandle->Flags = DEVICE_HANDLE_FLAG_ROOTHUB;
//
//     RtlZeroMemory(&RootHubData, sizeof(RootHubData));
//
//     Packet->RH_GetRootHubData(FdoExtension->MiniPortExt, &RootHubData);
//
//     ASSERT(RootHubData.NumberOfPorts != 0);
//     NumMaskByte = (RootHubData.NumberOfPorts - 1) / 8 + 1;
//
//     DescriptorsLength = sizeof(USB_DEVICE_DESCRIPTOR) +
//                         sizeof(USB_CONFIGURATION_DESCRIPTOR) +
//                         sizeof(USB_INTERFACE_DESCRIPTOR) +
//                         sizeof(USB_ENDPOINT_DESCRIPTOR) +
//                         (sizeof(USB_HUB_DESCRIPTOR) + 2 * NumMaskByte);
//
//     Descriptors = ExAllocatePoolWithTag(NonPagedPool,
//                                         DescriptorsLength,
//                                         USB_PORT_TAG);
//
//     if (Descriptors)
//     {
//         RtlZeroMemory(Descriptors, DescriptorsLength);
//
//         PdoExtension->RootHubDescriptors = Descriptors;
//
//         RH_DeviceDescriptor = &PdoExtension->RootHubDescriptors->DeviceDescriptor;
//
//         RH_DeviceDescriptor->bLength = sizeof(USB_DEVICE_DESCRIPTOR);
//         RH_DeviceDescriptor->bDescriptorType = USB_DEVICE_DESCRIPTOR_TYPE;
//         RH_DeviceDescriptor->bcdUSB = 0x100;
//         RH_DeviceDescriptor->bDeviceClass = USB_DEVICE_CLASS_HUB;
//         RH_DeviceDescriptor->bDeviceSubClass = 0x01;
//         RH_DeviceDescriptor->bDeviceProtocol = 0x00;
//         RH_DeviceDescriptor->bMaxPacketSize0 = 0x08;
//         RH_DeviceDescriptor->idVendor = FdoExtension->VendorID;
//         RH_DeviceDescriptor->idProduct = FdoExtension->DeviceID;
//         RH_DeviceDescriptor->bcdDevice = FdoExtension->RevisionID;
//         RH_DeviceDescriptor->iManufacturer = 0x00;
//         RH_DeviceDescriptor->iProduct = 0x00;
//         RH_DeviceDescriptor->iSerialNumber = 0x00;
//         RH_DeviceDescriptor->bNumConfigurations = 0x01;
//
//         RH_ConfigurationDescriptor = &PdoExtension->RootHubDescriptors->ConfigDescriptor;
//
//         RH_ConfigurationDescriptor->bLength = sizeof(USB_CONFIGURATION_DESCRIPTOR);
//         RH_ConfigurationDescriptor->bDescriptorType = USB_CONFIGURATION_DESCRIPTOR_TYPE;
//
//         RH_ConfigurationDescriptor->wTotalLength = sizeof(USB_CONFIGURATION_DESCRIPTOR) +
//                                                    sizeof(USB_INTERFACE_DESCRIPTOR) +
//                                                    sizeof(USB_ENDPOINT_DESCRIPTOR);
//
//         RH_ConfigurationDescriptor->bNumInterfaces = 0x01;
//         RH_ConfigurationDescriptor->bConfigurationValue = 0x01;
//         RH_ConfigurationDescriptor->iConfiguration = 0x00;
//         RH_ConfigurationDescriptor->bmAttributes = USB_CONFIG_SELF_POWERED;
//         RH_ConfigurationDescriptor->MaxPower = 0x00;
//
//         RH_InterfaceDescriptor = &PdoExtension->RootHubDescriptors->InterfaceDescriptor;
//
//         RH_InterfaceDescriptor->bLength = sizeof(USB_INTERFACE_DESCRIPTOR);
//         RH_InterfaceDescriptor->bDescriptorType = USB_INTERFACE_DESCRIPTOR_TYPE;
//         RH_InterfaceDescriptor->bInterfaceNumber = 0x00;
//         RH_InterfaceDescriptor->bAlternateSetting = 0x00;
//         RH_InterfaceDescriptor->bNumEndpoints = 0x01;
//         RH_InterfaceDescriptor->bInterfaceClass = USB_DEVICE_CLASS_HUB;
//         RH_InterfaceDescriptor->bInterfaceSubClass = 0x01;
//         RH_InterfaceDescriptor->bInterfaceProtocol = 0x00;
//         RH_InterfaceDescriptor->iInterface = 0x00;
//
//         RH_EndPointDescriptor = &PdoExtension->RootHubDescriptors->EndPointDescriptor;
//
//         RH_EndPointDescriptor->bLength = sizeof(USB_ENDPOINT_DESCRIPTOR);
//         RH_EndPointDescriptor->bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;
//         RH_EndPointDescriptor->bEndpointAddress = 0x81;
//         RH_EndPointDescriptor->bmAttributes = USB_ENDPOINT_TYPE_INTERRUPT; // SCE endpoint
//         RH_EndPointDescriptor->wMaxPacketSize = 0x0008;
//         RH_EndPointDescriptor->bInterval = 0x0C; // 12 msec
//
//         RH_HubDescriptor = &PdoExtension->RootHubDescriptors->Descriptor;
//
//         RH_HubDescriptor->bDescriptorLength = FIELD_OFFSET(USB_HUB_DESCRIPTOR, bRemoveAndPowerMask) + 2 * NumMaskByte;
//
//         if (Packet->MiniPortVersion == USB_MINIPORT_VERSION_OHCI ||
//             Packet->MiniPortVersion == USB_MINIPORT_VERSION_UHCI ||
//             Packet->MiniPortVersion == USB_MINIPORT_VERSION_EHCI)
//         {
//             RH_HubDescriptor->bDescriptorType = USB_20_HUB_DESCRIPTOR_TYPE;
//         }
//         else if (Packet->MiniPortVersion == USB_MINIPORT_VERSION_XHCI)
//         {
//             RH_HubDescriptor->bDescriptorType = USB_30_HUB_DESCRIPTOR_TYPE;
//         }
//         else
//         {
//             DPRINT1("USBPORT_RootHubCreateDevice: Unknown MiniPortVersion - %x\n",
//                     Packet->MiniPortVersion);
//
//             DbgBreakPoint();
//         }
//
//         RH_HubDescriptor->bNumberOfPorts = RootHubData.NumberOfPorts;
//         RH_HubDescriptor->wHubCharacteristics = RootHubData.HubCharacteristics.AsUSHORT;
//         RH_HubDescriptor->bPowerOnToPowerGood = RootHubData.PowerOnToPowerGood;
//         RH_HubDescriptor->bHubControlCurrent = RootHubData.HubControlCurrent;
//
//         for (ix = 0; ix < NumMaskByte; ix += 2)
//         {
//             RH_HubDescriptor->bRemoveAndPowerMask[ix] = 0;
//             RH_HubDescriptor->bRemoveAndPowerMask[ix + 1] = -1;
//         }
//
//         EndpointDescriptor = &DeviceHandle->PipeHandle.EndpointDescriptor;
//
//         EndpointDescriptor->bLength = sizeof(USB_ENDPOINT_DESCRIPTOR);
//         EndpointDescriptor->bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;
//         EndpointDescriptor->bEndpointAddress = 0x00;
//         EndpointDescriptor->bmAttributes = USB_ENDPOINT_TYPE_CONTROL;
//         EndpointDescriptor->wMaxPacketSize = 0x0040;
//         EndpointDescriptor->bInterval = 0x00;
//
//         Status = USBPORT_OpenPipe(FdoDevice,
//                                   DeviceHandle,
//                                   &DeviceHandle->PipeHandle,
//                                   NULL);
//     }
//     else
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return Status;
// }
//
// ULONG
// NTAPI
// USBPORT_InvalidateRootHub(PVOID MiniPortExtension)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PDEVICE_OBJECT PdoDevice;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_ENDPOINT Endpoint = NULL;
//
//     DPRINT("USBPORT_InvalidateRootHub ... \n");
//
//     FdoExtension = (PUSBPORT_DEVICE_EXTENSION)((ULONG_PTR)MiniPortExtension -
//                                                sizeof(USBPORT_DEVICE_EXTENSION));
//
//     FdoDevice = FdoExtension->CommonExtension.SelfDevice;
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND &&
//         FdoExtension->Flags & USBPORT_FLAG_HC_WAKE_SUPPORT &&
//         FdoExtension->MiniPortFlags & USBPORT_MPFLAG_SUSPENDED &&
//         FdoExtension->TimerFlags & USBPORT_TMFLAG_WAKE)
//     {
//         USBPORT_HcQueueWakeDpc(FdoDevice);
//         return 0;
//     }
//
//     FdoExtension->MiniPortInterface->Packet.RH_DisableIrq(FdoExtension->MiniPortExt);
//
//     PdoDevice = FdoExtension->RootHubPdo;
//
//     if (PdoDevice)
//     {
//         PdoExtension = PdoDevice->DeviceExtension;
//         Endpoint = PdoExtension->Endpoint;
//
//         if (Endpoint)
//         {
//             USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                               PdoExtension->Endpoint,
//                                               INVALIDATE_ENDPOINT_WORKER_THREAD);
//         }
//     }
//
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_RootHubPowerAndChirpAllCcPorts(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     USBPORT_ROOT_HUB_DATA RootHubData;
//     ULONG Port;
//     PDEVICE_RELATIONS CompanionControllersList;
//     PUSBPORT_DEVICE_EXTENSION CompanionFdoExtension;
//     PUSBPORT_REGISTRATION_PACKET CompanionPacket;
//     ULONG CompanionPorts;
//     ULONG NumController;
//     PDEVICE_OBJECT * Entry;
//     ULONG NumPorts;
//
//     DPRINT("USBPORT_RootHub_PowerAndChirpAllCcPorts: FdoDevice - %p\n",
//            FdoDevice);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     RtlZeroMemory(&RootHubData, sizeof(RootHubData));
//
//     Packet->RH_GetRootHubData(FdoExtension->MiniPortExt,
//                               &RootHubData);
//
//     NumPorts = RootHubData.NumberOfPorts;
//
//     for (Port = 1; Port <= NumPorts; ++Port)
//     {
//         Packet->RH_SetFeaturePortPower(FdoExtension->MiniPortExt, Port);
//     }
//
//     USBPORT_Wait(FdoDevice, 10);
//
//     CompanionControllersList = USBPORT_FindCompanionControllers(FdoDevice,
//                                                                 FALSE,
//                                                                 TRUE);
//
//     if (CompanionControllersList)
//     {
//         Entry = &CompanionControllersList->Objects[0];
//
//         for (NumController = 0;
//              NumController < CompanionControllersList->Count;
//              NumController++)
//         {
//             CompanionPacket = &FdoExtension->MiniPortInterface->Packet;
//
//             CompanionFdoExtension = (*Entry)->DeviceExtension;
//
//             CompanionPacket->RH_GetRootHubData(CompanionFdoExtension->MiniPortExt,
//                                                &RootHubData);
//
//             CompanionPorts = RootHubData.NumberOfPorts;
//
//             for (Port = 1; Port <= CompanionPorts; ++Port)
//             {
//                 CompanionPacket->RH_SetFeaturePortPower(CompanionFdoExtension->MiniPortExt,
//                                                         Port);
//             }
//
//             ++Entry;
//         }
//
//         ExFreePoolWithTag(CompanionControllersList, USB_PORT_TAG);
//     }
//
//     USBPORT_Wait(FdoDevice, 100);
//
//     for (Port = 1; Port <= NumPorts; ++Port)
//     {
//         if (FdoExtension->MiniPortInterface->Version < 200)
//         {
//             break;
//         }
//
//         InterlockedIncrement((PLONG)&FdoExtension->ChirpRootPortLock);
//         Packet->RH_ChirpRootPort(FdoExtension->MiniPortExt, Port);
//         InterlockedDecrement((PLONG)&FdoExtension->ChirpRootPortLock);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
