use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateMdl, IoCsqInsertIrp, IoMarkIrpPending, MmBuildMdlForNonPagedPool, PDEVICE_OBJECT, PIRP, PMDL};
use core::ffi::c_void;

extern "system" {
    fn USBPORT_DumpingSetupPacket(SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET);
    fn USBPORT_QueueTransferUrb(Urb: PURB);
    fn USBPORT_USBDStatusToNtStatus(Urb: PURB, USBDStatus: USBD_STATUS) -> NTSTATUS;
    fn USBPORT_ValidatePipeHandle( DeviceHandle: PUSBPORT_DEVICE_HANDLE, PipeHandle: PUSBPORT_PIPE_HANDLE, ) -> u8;
    fn USBPORT_AbortEndpoint( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, Irp: PIRP, );
    fn USBPORT_DumpingURB(Urb: PURB);
    fn USBPORT_GetEndpointState(Endpoint: PUSBPORT_ENDPOINT) -> ULONG;
    fn USBPORT_SetEndpointState(Endpoint: PUSBPORT_ENDPOINT, EndpointState: ULONG);
    fn USBPORT_Wait(MiniPortExtension: PVOID, Milliseconds: ULONG,) -> NTSTATUS;
    fn USBPORT_AllocateTransfer( FdoDevice: PDEVICE_OBJECT, Urb: PURB, DeviceHandle: PUSBPORT_DEVICE_HANDLE, Irp: PIRP, Transfer: *mut core::ffi::c_void, ) -> USBD_STATUS;
    fn USBPORT_ValidateDeviceHandle( FdoDevice: PDEVICE_OBJECT, DeviceHandle: PUSBPORT_DEVICE_HANDLE, ) -> BOOLEAN;
    fn USBPORT_HandleSelectConfiguration( FdoDevice: PDEVICE_OBJECT, Irp: PIRP, Urb: PURB, ) -> NTSTATUS;
    fn USBPORT_HandleSelectInterface( FdoDevice: PDEVICE_OBJECT, Irp: PIRP, Urb: PURB, ) -> NTSTATUS;
}
extern "C" {
    fn USBD_TRANSFER_DIRECTION_FLAG(TransferFlags: u32) -> u32;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG  (function-like/empty macro, skipped)
// #define NDEBUG_USBPORT_URB  (empty macro, skipped)


const USBD_FLAG_ALLOCATED_MDL: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleGetConfiguration(Urb: PURB) -> NTSTATUS {

    const BMREQUEST_DEVICE_TO_HOST: u32 = 1;
    const USB_REQUEST_GET_CONFIGURATION: u32 = 0x08;
    const USBD_TRANSFER_DIRECTION_IN: u32 = 1;
    const USBD_SHORT_TRANSFER_OK: u32 = 2;

    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;

    DPRINT_URB!(b"USBPORT_HandleGetConfiguration: Urb - %p\n\0".as_ptr() as *const i8, Urb);

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlGetConfigurationRequest.Reserved1) as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);
    (*SetupPacket).bRequest = USB_REQUEST_GET_CONFIGURATION as u8;
    (*SetupPacket).wValue.W = 0;
    (*SetupPacket).wIndex.W = 0;
    (*SetupPacket).wLength = (*Urb).__bindgen_anon_1.UrbControlGetConfigurationRequest.TransferBufferLength as u16;

    (*Urb).__bindgen_anon_1.UrbControlGetConfigurationRequest.Reserved0 |= USBD_TRANSFER_DIRECTION_IN;
    (*Urb).__bindgen_anon_1.UrbControlGetConfigurationRequest.Reserved0 |= USBD_SHORT_TRANSFER_OK;

    USBPORT_DumpingSetupPacket(SetupPacket);

    USBPORT_QueueTransferUrb(Urb);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleGetCurrentFrame(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let FrameNumber: ULONG;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    FrameNumber = ((*Packet).Get32BitFrameNumber.unwrap())((*FdoExtension).MiniPortExt);
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        OldIrql,
    );

    (*Urb).__bindgen_anon_1.UrbGetCurrentFrameNumber.FrameNumber = FrameNumber;

    DPRINT_URB!(
        b"USBPORT_HandleGetCurrentFrame: FrameNumber - %p\n\0".as_ptr() as *const i8,
        FrameNumber,
    );

    return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AbortPipe(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    const PIPE_HANDLE_FLAG_NULL_PACKET_SIZE: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h // PIPE_HANDLE_FLAG_NULL_PACKET_SIZE
    const USBD_STATUS_SUCCESS: USBD_STATUS = 0x00000000u32 as USBD_STATUS; // USBD_STATUS_SUCCESS
    const USBD_STATUS_INVALID_PIPE_HANDLE: USBD_STATUS = 0x80000600u32 as USBD_STATUS; // USBD_STATUS_INVALID_PIPE_HANDLE

    let Endpoint: PUSBPORT_ENDPOINT;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let Status: NTSTATUS;

    // DPRINT_URB!("USBPORT_AbortPipe: ... \n");

    PipeHandle = ((*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle as *mut _USBPORT_PIPE_HANDLE);
    DeviceHandle = ((*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as *mut _USBPORT_DEVICE_HANDLE);

    if USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle) != 0 {
        if ((*PipeHandle).Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE) == 0 {
            Endpoint = (*PipeHandle).Endpoint;

            Status = STATUS_PENDING;

            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoMarkIrpPending(Irp);

            USBPORT_AbortEndpoint(FdoDevice, Endpoint, Irp);

            return Status;
        }

        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
    } else {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ResetPipe(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    const USBD_STATUS_INVALID_PIPE_HANDLE: USBD_STATUS = 0x80000600u32 as i32;
    const USBD_STATUS_SUCCESS: USBD_STATUS = 0;
    const USBD_STATUS_ERROR_BUSY: USBD_STATUS = 0x80000400u32 as i32;
    const USBD_FLAG_NOT_ISO_TRANSFER: u32 = 0x00000010; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const ENDPOINT_FLAG_QUEUENE_EMPTY: u32 = 0x00000010;
    const USBPORT_ENDPOINT_RUN: u32 = 0;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let Endpoint: PUSBPORT_ENDPOINT;
    let Status: NTSTATUS;

    DPRINT_URB!(b"USBPORT_ResetPipe: ... \n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    PipeHandle = ((*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle as *mut _USBPORT_PIPE_HANDLE);

    if USBPORT_ValidatePipeHandle(
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE,
        PipeHandle,
    ) == 0
    {
        return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);
    }

    Endpoint = (*PipeHandle).Endpoint;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList)) {
        if (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags & USBD_FLAG_NOT_ISO_TRANSFER != 0 {
            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));

            ((*Packet).SetEndpointDataToggle).unwrap()(
                (*FdoExtension).MiniPortExt,
                (Endpoint.offset(1) as *mut c_void),
                0,
            );

            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
        }

        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
    } else {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_ERROR_BUSY);
    }

    (*Endpoint).Flags |= ENDPOINT_FLAG_QUEUENE_EMPTY;

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));

    ((*Packet).SetEndpointStatus).unwrap()(
        (*FdoExtension).MiniPortExt,
        (Endpoint.offset(1) as *mut c_void),
        USBPORT_ENDPOINT_RUN,
    );

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SyncResetPipeAndClearStall(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    const USBD_STATUS_INVALID_PIPE_HANDLE: USBD_STATUS = 0x80000600u32 as i32;
    const USBD_STATUS_SUCCESS: USBD_STATUS = 0x00000000u32 as USBD_STATUS;
    const PIPE_HANDLE_FLAG_NULL_PACKET_SIZE: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0x00000000; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const USBD_FLAG_NOT_ISO_TRANSFER: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const USBPORT_ENDPOINT_PAUSED: ULONG = 2;
    const USBPORT_ENDPOINT_ACTIVE: ULONG = 0x00000003; // FIXED 2026-07-05: translation invented 0x0; verified vs usbport.h/usbmport.h

    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let Endpoint: PUSBPORT_ENDPOINT;
    let mut EndpointState: ULONG;
    let mut Status: NTSTATUS;

    DPRINT_URB!(b"USBPORT_SyncResetPipeAndClearStall: ... \n\0".as_ptr() as *const i8);

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    PipeHandle = (*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle as PUSBPORT_PIPE_HANDLE;

    if USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle) == 0 {
        return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);
    }

    if (*PipeHandle).Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE != 0 {
        return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
    }

    Endpoint = (*PipeHandle).Endpoint;
    InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

    if (*Endpoint).EndpointProperties.TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= USBD_FLAG_NOT_ISO_TRANSFER;
        Status = USBPORT_ClearStall(FdoDevice, Irp, Urb);
    } else {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
    }

    if NT_SUCCESS(Status) {
        Status = USBPORT_ResetPipe(FdoDevice, Irp, Urb);

        if (*Endpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
            while 1 != 0 {
                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
                );

                EndpointState = USBPORT_GetEndpointState(Endpoint);

                if EndpointState == USBPORT_ENDPOINT_PAUSED
                    && IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList))
                {
                    USBPORT_SetEndpointState(Endpoint, USBPORT_ENDPOINT_ACTIVE);
                }

                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                    (*Endpoint).EndpointOldIrql,
                );

                if EndpointState == USBPORT_ENDPOINT_ACTIVE {
                    break;
                }

                USBPORT_Wait((FdoDevice) as _, 1);
            }
        }
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleSetOrClearFeature(Urb: PURB) -> NTSTATUS {

    const BMREQUEST_HOST_TO_DEVICE: u32 = 0;
    const BMREQUEST_TO_DEVICE: u32 = 0;
    const BMREQUEST_TO_INTERFACE: u32 = 1;
    const BMREQUEST_TO_ENDPOINT: u32 = 2;
    const BMREQUEST_TO_OTHER: u32 = 3;
    const USB_REQUEST_SET_FEATURE: u8 = 0x03;
    const USB_REQUEST_CLEAR_FEATURE: u8 = 0x01;
    const URB_FUNCTION_SET_FEATURE_TO_DEVICE: u16 = 0x000D;
    const URB_FUNCTION_SET_FEATURE_TO_INTERFACE: u16 = 0x000E;
    const URB_FUNCTION_SET_FEATURE_TO_ENDPOINT: u16 = 0x000F;
    const URB_FUNCTION_CLEAR_FEATURE_TO_DEVICE: u16 = 0x0010;
    const URB_FUNCTION_CLEAR_FEATURE_TO_INTERFACE: u16 = 0x0011;
    const URB_FUNCTION_CLEAR_FEATURE_TO_ENDPOINT: u16 = 0x0012;
    const URB_FUNCTION_CLEAR_FEATURE_TO_OTHER: u16 = 0x0022;
    const URB_FUNCTION_SET_FEATURE_TO_OTHER: u16 = 0x0023;
    const USBD_TRANSFER_DIRECTION_IN: u32 = 0x00000001;
    const USBD_SHORT_TRANSFER_OK: u32 = 0x00000002;

    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;

    DPRINT_URB!(b"USBPORT_HandleSetOrClearFeature: Urb - %p\n\0".as_ptr() as *const i8, Urb);

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlFeatureRequest.Reserved0)
        as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    (*SetupPacket).wLength = 0;
    (*Urb).__bindgen_anon_1.UrbControlFeatureRequest.Reserved3 = 0; // TransferBufferLength

    (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE as _);

    match (*Urb).__bindgen_anon_1.UrbHeader.Function {
        URB_FUNCTION_SET_FEATURE_TO_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE as _);
        }

        URB_FUNCTION_SET_FEATURE_TO_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE as _);
        }

        URB_FUNCTION_SET_FEATURE_TO_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT as _);
        }

        URB_FUNCTION_CLEAR_FEATURE_TO_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_CLEAR_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE as _);
        }

        URB_FUNCTION_CLEAR_FEATURE_TO_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_CLEAR_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE as _);
        }

        URB_FUNCTION_CLEAR_FEATURE_TO_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_CLEAR_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT as _);
        }

        URB_FUNCTION_CLEAR_FEATURE_TO_OTHER => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_CLEAR_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_OTHER as _);
        }

        URB_FUNCTION_SET_FEATURE_TO_OTHER => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_FEATURE;
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_OTHER as _);
        }

        _ => {}
    }

    (*Urb).__bindgen_anon_1.UrbControlFeatureRequest.Reserved2 &= !USBD_TRANSFER_DIRECTION_IN;
    (*Urb).__bindgen_anon_1.UrbControlFeatureRequest.Reserved2 |= USBD_SHORT_TRANSFER_OK;

    USBPORT_DumpingSetupPacket(SetupPacket);

    USBPORT_QueueTransferUrb(Urb);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleDataTransfers(Urb: PURB) -> NTSTATUS {
    let Endpoint: PUSBPORT_ENDPOINT;

    DPRINT_URB!(b"USBPORT_HandleDataTransfers: Urb - %p\n\0".as_ptr() as *const i8, Urb);

    Endpoint = (*((*Urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.PipeHandle as PUSBPORT_PIPE_HANDLE)).Endpoint;

    if (*Endpoint).EndpointProperties.TransferType != USBPORT_TRANSFER_TYPE_CONTROL {
        if (*Endpoint).EndpointProperties.Direction == USBPORT_TRANSFER_DIRECTION_OUT {
            (*Urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferFlags &= !USBD_TRANSFER_DIRECTION_IN;
        } else {
            (*Urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
        }
    }

    USBPORT_QueueTransferUrb(Urb);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleGetStatus(Irp: PIRP, Urb: PURB) -> NTSTATUS {

    const BMREQUEST_DEVICE_TO_HOST: u32 = 1;
    const USB_REQUEST_GET_STATUS: u32 = 0x00;
    const BMREQUEST_TO_DEVICE: u32 = 0;
    const BMREQUEST_TO_INTERFACE: u32 = 1;
    const BMREQUEST_TO_ENDPOINT: u32 = 2;
    const BMREQUEST_TO_OTHER: u32 = 3;
    const URB_FUNCTION_GET_STATUS_FROM_DEVICE: u16 = 0x0013;
    const URB_FUNCTION_GET_STATUS_FROM_INTERFACE: u16 = 0x0014;
    const URB_FUNCTION_GET_STATUS_FROM_ENDPOINT: u16 = 0x0015;
    const URB_FUNCTION_GET_STATUS_FROM_OTHER: u16 = 0x0021;
    const USBD_SHORT_TRANSFER_OK: u32 = 0x00000002;
    const USBD_TRANSFER_DIRECTION_IN: u32 = 0x00000001;
    const USBD_STATUS_INVALID_PARAMETER: USBD_STATUS = 0x80000300u32 as USBD_STATUS;

    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;
    let Status: NTSTATUS;

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.Reserved1)
        as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    (*SetupPacket).bmRequestType.B = 0;
    (*SetupPacket).bmRequestType.set_Dir((BMREQUEST_DEVICE_TO_HOST as u8));
    (*SetupPacket).bRequest = USB_REQUEST_GET_STATUS as u8;
    (*SetupPacket).wLength = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferLength as u16;
    (*SetupPacket).wValue.W = 0;

    match (*Urb).__bindgen_anon_1.UrbHeader.Function {
        URB_FUNCTION_GET_STATUS_FROM_DEVICE => {
            (*SetupPacket).bmRequestType.set_Recipient((BMREQUEST_TO_DEVICE as u8));
        }
        URB_FUNCTION_GET_STATUS_FROM_INTERFACE => {
            (*SetupPacket).bmRequestType.set_Recipient((BMREQUEST_TO_INTERFACE as u8));
        }
        URB_FUNCTION_GET_STATUS_FROM_ENDPOINT => {
            (*SetupPacket).bmRequestType.set_Recipient((BMREQUEST_TO_ENDPOINT as u8));
        }
        URB_FUNCTION_GET_STATUS_FROM_OTHER => {
            (*SetupPacket).bmRequestType.set_Recipient((BMREQUEST_TO_OTHER as u8));
        }
        _ => {}
    }

    if (*SetupPacket).wLength == 2 {
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;

        if (*SetupPacket).bmRequestType.Dir() != 0 {
            (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
        } else {
            (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags &= !USBD_TRANSFER_DIRECTION_IN;
        }

        //USBPORT_DumpingSetupPacket(SetupPacket);

        USBPORT_QueueTransferUrb(Urb);

        Status = STATUS_PENDING;
    } else {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PARAMETER);

        USBPORT_DumpingSetupPacket(SetupPacket);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleVendorOrClass(Irp: PIRP, Urb: PURB) -> NTSTATUS {

    const USBD_SHORT_TRANSFER_OK: u32 = 0x00000002;
    const URB_FUNCTION_VENDOR_DEVICE: u16 = 0x0017;
    const URB_FUNCTION_VENDOR_INTERFACE: u16 = 0x0018;
    const URB_FUNCTION_VENDOR_ENDPOINT: u16 = 0x0019;
    const URB_FUNCTION_CLASS_DEVICE: u16 = 0x001A;
    const URB_FUNCTION_CLASS_INTERFACE: u16 = 0x001B;
    const URB_FUNCTION_CLASS_ENDPOINT: u16 = 0x001C;
    const URB_FUNCTION_CLASS_OTHER: u16 = 0x001F;
    const URB_FUNCTION_VENDOR_OTHER: u16 = 0x0020;
    const BMREQUEST_VENDOR: u8 = 2;
    const BMREQUEST_CLASS: u8 = 1;
    const BMREQUEST_TO_DEVICE: u8 = 0;
    const BMREQUEST_TO_INTERFACE: u8 = 1;
    const BMREQUEST_TO_ENDPOINT: u8 = 2;
    const BMREQUEST_TO_OTHER: u8 = 3;

    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.Reserved1)
        as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    (*SetupPacket).bmRequestType.set_Dir(USBD_TRANSFER_DIRECTION_FLAG((*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags) as u8);

    (*SetupPacket).wLength = (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferLength as u16;

    (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;

    match (*Urb).__bindgen_anon_1.UrbHeader.Function {
        URB_FUNCTION_VENDOR_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_VENDOR);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE);
        }
        URB_FUNCTION_VENDOR_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_VENDOR);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);
        }
        URB_FUNCTION_VENDOR_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_VENDOR);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT);
        }
        URB_FUNCTION_CLASS_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_CLASS);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE);
        }
        URB_FUNCTION_CLASS_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_CLASS);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);
        }
        URB_FUNCTION_CLASS_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_CLASS);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT);
        }
        URB_FUNCTION_CLASS_OTHER => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_OTHER\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_CLASS);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_OTHER);
        }
        URB_FUNCTION_VENDOR_OTHER => {
            DPRINT_URB!(b"USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_OTHER\n\0".as_ptr() as *const i8);
            (*SetupPacket).bmRequestType.set_Type(BMREQUEST_VENDOR);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_OTHER);
        }
        _ => {}
    }

    USBPORT_DumpingSetupPacket(SetupPacket);

    USBPORT_QueueTransferUrb(Urb);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleGetSetDescriptor(
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    const BMREQUEST_STANDARD: u32 = 0;
    const BMREQUEST_DEVICE_TO_HOST: u32 = 1;
    const BMREQUEST_HOST_TO_DEVICE: u32 = 0;
    const BMREQUEST_TO_DEVICE: u32 = 0;
    const BMREQUEST_TO_INTERFACE: u32 = 1;
    const BMREQUEST_TO_ENDPOINT: u32 = 2;

    const USB_REQUEST_GET_DESCRIPTOR: u8 = 0x06;
    const USB_REQUEST_SET_DESCRIPTOR: u8 = 0x07;

    const URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE: u16 = 0x000B;
    const URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE: u16 = 0x000C;
    const URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT: u16 = 0x0024;
    const URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT: u16 = 0x0025;
    const URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE: u16 = 0x0028;
    const URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE: u16 = 0x0029;

    const USBD_SHORT_TRANSFER_OK: u32 = 0x00000002;
    const USBD_TRANSFER_DIRECTION_IN: u32 = 0x00000001;

    let SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET;

    SetupPacket = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.Reserved1)
        as PUSB_DEFAULT_PIPE_SETUP_PACKET;

    (*SetupPacket).wLength =
        (*Urb).__bindgen_anon_1.UrbControlDescriptorRequest.TransferBufferLength as u16;
    (*SetupPacket).bmRequestType.B = 0; // Clear bmRequestType
    (*SetupPacket).bmRequestType.set_Type(BMREQUEST_STANDARD as u8);

    match (*Urb).__bindgen_anon_1.UrbHeader.Function {
        URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_GET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE as u8);
        }
        URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_DEVICE as u8);
        }
        URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_GET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT as u8);
        }
        URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT as u8);
        }
        URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_GET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE as u8);
        }
        URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE => {
            DPRINT_URB!(b"USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n\0".as_ptr() as *const i8);
            (*SetupPacket).bRequest = USB_REQUEST_SET_DESCRIPTOR;
            (*SetupPacket).bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE as u8);
            (*SetupPacket).bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE as u8);
        }
        _ => {}
    }

    (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;

    if (*SetupPacket).bmRequestType.Dir() != 0 {
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
    } else {
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags &= !USBD_TRANSFER_DIRECTION_IN;
    }

    USBPORT_DumpingSetupPacket(SetupPacket);

    USBPORT_QueueTransferUrb(Urb);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ValidateTransferParametersURB(Urb: PURB) -> NTSTATUS {
    let UrbRequest: *mut _URB_CONTROL_TRANSFER;
    let Mdl: PMDL;

    DPRINT_URB!(b"USBPORT_ValidateTransferParametersURB: Urb - %p\n\0".as_ptr() as *const i8, Urb);

    UrbRequest = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer);

    if (*UrbRequest).TransferBuffer.is_null()
        && (*UrbRequest).TransferBufferMDL.is_null()
        && (*UrbRequest).TransferBufferLength > 0
    {
        // DPRINT removed
        USBPORT_DumpingURB(Urb);
        return STATUS_INVALID_PARAMETER;
    }

    if !(*UrbRequest).TransferBuffer.is_null()
        && !(*UrbRequest).TransferBufferMDL.is_null()
        && (*UrbRequest).TransferBufferLength == 0
    {
        // DPRINT removed
        USBPORT_DumpingURB(Urb);
        return STATUS_INVALID_PARAMETER;
    }

    if !(*UrbRequest).TransferBuffer.is_null()
        && (*UrbRequest).TransferBufferMDL.is_null()
        && (*UrbRequest).TransferBufferLength != 0
    {
        DPRINT_URB!(
            b"USBPORT_ValidateTransferParametersURB: TransferBuffer - %p, TransferBufferLength - %x\n\0".as_ptr() as *const i8,
            (*UrbRequest).TransferBuffer,
            (*UrbRequest).TransferBufferLength,
        );

        Mdl = IoAllocateMdl(
            (*UrbRequest).TransferBuffer,
            (*UrbRequest).TransferBufferLength,
            0,
            0,
            core::ptr::null_mut(),
        );

        if Mdl.is_null() {
            // DPRINT removed
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        MmBuildMdlForNonPagedPool(Mdl);

        (*UrbRequest).TransferBufferMDL = Mdl;
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_MDL;

        DPRINT_URB!(b"USBPORT_ValidateTransferParametersURB: Mdl - %p\n\0".as_ptr() as *const i8, Mdl);
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ValidateURB(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
    IsControlTransfer: BOOLEAN,
    IsNullTransfer: BOOLEAN,
) -> NTSTATUS {

    const USBD_STATUS_INVALID_PARAMETER: USBD_STATUS = 0x80000300u32 as USBD_STATUS;
    const USBD_STATUS_INVALID_PIPE_HANDLE: USBD_STATUS = 0x80000600u32 as USBD_STATUS;
    const USBD_DEFAULT_PIPE_TRANSFER: u32 = 0x00000008;
    const URB_FUNCTION_CONTROL_TRANSFER: u16 = 0x0008;

    let UrbRequest: *mut _URB_CONTROL_TRANSFER;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut Status: NTSTATUS;
    let USBDStatus: USBD_STATUS;

    UrbRequest = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer);

    if !(*UrbRequest).UrbLink.is_null() {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PARAMETER);

        // DPRINT1("USBPORT_ValidateURB: Not valid parameter\n");

        USBPORT_DumpingURB(Urb);
        return Status;
    }

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;

    if IsControlTransfer != 0 {
        (*UrbRequest).TransferFlags |= USBD_DEFAULT_PIPE_TRANSFER;
        (*UrbRequest).PipeHandle = core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle) as *mut core::ffi::c_void;
    }

    if (*UrbRequest).TransferFlags & USBD_DEFAULT_PIPE_TRANSFER != 0 {
        if (*UrbRequest).TransferBufferLength > 0x1000 {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PARAMETER);

            // DPRINT1("USBPORT_ValidateURB: Not valid parameter\n");

            USBPORT_DumpingURB(Urb);
            return Status;
        }

        if (*Urb).__bindgen_anon_1.UrbHeader.Function == URB_FUNCTION_CONTROL_TRANSFER {
            (*UrbRequest).PipeHandle = core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle) as *mut core::ffi::c_void;
        }
    }

    if USBPORT_ValidatePipeHandle(DeviceHandle, ((*UrbRequest).PipeHandle as *mut _USBPORT_PIPE_HANDLE)) == 0 {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);

        // DPRINT1("USBPORT_ValidateURB: Not valid pipe handle\n");

        USBPORT_DumpingURB(Urb);
        return Status;
    }

    (*UrbRequest).hca.Reserved8[0] = core::ptr::null_mut(); // Transfer

    if IsNullTransfer != 0 {
        (*UrbRequest).TransferBuffer = core::ptr::null_mut();
        (*UrbRequest).TransferBufferMDL = core::ptr::null_mut();
        (*UrbRequest).TransferBufferLength = 0;
    } else {
        Status = USBPORT_ValidateTransferParametersURB(Urb);

        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    USBDStatus = USBPORT_AllocateTransfer(FdoDevice, Urb, DeviceHandle, Irp, core::ptr::null_mut());

    Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);

    if !NT_SUCCESS(Status) {
        // DPRINT1("USBPORT_ValidateURB: Not allocated transfer\n");
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleSubmitURB(
    PdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    let PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Function: USHORT;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;

    PdoExtension = (*PdoDevice).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    (*Urb).__bindgen_anon_1.UrbHeader.Status = USBD_STATUS_SUCCESS;
    (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags = 0;

    Function = (*Urb).__bindgen_anon_1.UrbHeader.Function;

    if Function as u32 > URB_FUNCTION_MAX {
        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_URB_FUNCTION);

        DbgPrint(b"USBPORT_HandleSubmitURB: Unknown URB function - %x !!!\n\0".as_ptr() as *const i8, Function as u32);

        return Status;
    }

    if (*FdoExtension).TimerFlags & USBPORT_TMFLAG_RH_SUSPENDED != 0 {
        DbgPrint(b"USBPORT_HandleSubmitURB: Bad Request\n\0".as_ptr() as *const i8);
        USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_DEVICE_GONE);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PENDING;
        IoMarkIrpPending(Irp);
        IoCsqInsertIrp(
            core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsq),
            Irp,
            core::ptr::null_mut(),
        );

        return STATUS_PENDING;
    }

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;

    if DeviceHandle.is_null() {
        DeviceHandle = core::ptr::addr_of_mut!((*PdoExtension).DeviceHandle);
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle = DeviceHandle as *mut core::ffi::c_void;
    }

    if USBPORT_ValidateDeviceHandle((*PdoExtension).FdoDevice, DeviceHandle) == 0 {
        DbgPrint(b"USBPORT_HandleSubmitURB: Not valid device handle\n\0".as_ptr() as *const i8);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PENDING;
        IoMarkIrpPending(Irp);
        IoCsqInsertIrp(
            core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsq),
            Irp,
            core::ptr::null_mut(),
        );

        return STATUS_PENDING;
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

    match Function as u32 {
        URB_FUNCTION_ISOCH_TRANSFER => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_ISOCH_TRANSFER UNIMPLEMENTED. FIXME. \n\0".as_ptr() as *const i8);
        }

        URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER | URB_FUNCTION_CONTROL_TRANSFER => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 0, 0);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleDataTransfers(Urb);
            }
        }

        URB_FUNCTION_VENDOR_DEVICE
        | URB_FUNCTION_VENDOR_INTERFACE
        | URB_FUNCTION_VENDOR_ENDPOINT
        | URB_FUNCTION_CLASS_DEVICE
        | URB_FUNCTION_CLASS_INTERFACE
        | URB_FUNCTION_CLASS_ENDPOINT
        | URB_FUNCTION_CLASS_OTHER
        | URB_FUNCTION_VENDOR_OTHER => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 1, 0);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleVendorOrClass(Irp, Urb);
            }
        }

        URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE
        | URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE
        | URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT
        | URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT
        | URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE
        | URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 1, 0);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleGetSetDescriptor(Irp, Urb);
            }
        }

        URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR (0x2A) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_URB_FUNCTION);
        }

        URB_FUNCTION_GET_STATUS_FROM_DEVICE
        | URB_FUNCTION_GET_STATUS_FROM_INTERFACE
        | URB_FUNCTION_GET_STATUS_FROM_ENDPOINT
        | URB_FUNCTION_GET_STATUS_FROM_OTHER => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 1, 0);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleGetStatus(Irp, Urb);
            }
        }

        URB_FUNCTION_SELECT_CONFIGURATION => {
            Status = USBPORT_HandleSelectConfiguration((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_SELECT_INTERFACE => {
            Status = USBPORT_HandleSelectInterface((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_GET_CONFIGURATION => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 1, 0);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleGetConfiguration(Urb);
            }
        }

        URB_FUNCTION_GET_INTERFACE => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_GET_INTERFACE (0x27) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_URB_FUNCTION);
        }

        URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL => {
            Status = USBPORT_SyncResetPipeAndClearStall((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_SYNC_RESET_PIPE => {
            Status = USBPORT_ResetPipe((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_SYNC_CLEAR_STALL => {
            Status = USBPORT_ClearStall((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_ABORT_PIPE => {
            Status = USBPORT_AbortPipe((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_SET_FEATURE_TO_DEVICE
        | URB_FUNCTION_SET_FEATURE_TO_INTERFACE
        | URB_FUNCTION_SET_FEATURE_TO_ENDPOINT
        | URB_FUNCTION_CLEAR_FEATURE_TO_INTERFACE
        | URB_FUNCTION_CLEAR_FEATURE_TO_ENDPOINT
        | URB_FUNCTION_CLEAR_FEATURE_TO_OTHER
        | URB_FUNCTION_SET_FEATURE_TO_OTHER => {
            Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, 1, 1);

            if !NT_SUCCESS(Status) {
                DbgPrint(b"USBPORT_HandleSubmitURB: Not valid URB\n\0".as_ptr() as *const i8);
            } else {
                Status = USBPORT_HandleSetOrClearFeature(Urb);
            }
        }

        URB_FUNCTION_GET_CURRENT_FRAME_NUMBER => {
            Status = USBPORT_HandleGetCurrentFrame((*PdoExtension).FdoDevice, Irp, Urb);
        }

        URB_FUNCTION_TAKE_FRAME_LENGTH_CONTROL => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_TAKE_FRAME_LENGTH_CONTROL (0x03) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
        }

        URB_FUNCTION_RELEASE_FRAME_LENGTH_CONTROL => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_RELEASE_FRAME_LENGTH_CONTROL (0x04) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
        }

        URB_FUNCTION_GET_FRAME_LENGTH => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_GET_FRAME_LENGTH (0x05) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
        }

        URB_FUNCTION_SET_FRAME_LENGTH => {
            DbgPrint(b"USBPORT_HandleSubmitURB: URB_FUNCTION_SET_FRAME_LENGTH (0x06) NOT_SUPPORTED\n\0".as_ptr() as *const i8);
            return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
        }

        _ => {
            DbgPrint(b"USBPORT_HandleSubmitURB: Unknown URB Function - %x\n\0".as_ptr() as *const i8, Function as u32);
        }
    }

    if Status == STATUS_PENDING {
        return Status;
    }

    if (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags & USBD_FLAG_ALLOCATED_TRANSFER != 0 {
        let Transfer: PUSBPORT_TRANSFER;

        Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
        (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] = core::ptr::null_mut();
        // The C writes `|= ~FLAG`, which SETS every other flag (ALLOCATED_MDL among them,
        // so a reused URB could IoFreeMdl a stale pointer) instead of clearing this one.
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags &= !USBD_FLAG_ALLOCATED_TRANSFER;
        ExFreePoolWithTag(Transfer as *mut core::ffi::c_void, USB_PORT_TAG);
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

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
//  * PURPOSE:     USBPort URB functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_URB
// #include "usbdebug.h"
//
// NTSTATUS
// NTAPI
// USBPORT_HandleGetConfiguration(IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     DPRINT_URB!("USBPORT_HandleGetConfiguration: Urb - %p\n", Urb);
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)
//                    &Urb->UrbControlGetConfigurationRequest.Reserved1;
//
//     SetupPacket->bmRequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//     SetupPacket->bRequest = USB_REQUEST_GET_CONFIGURATION;
//     SetupPacket->wValue.W = 0;
//     SetupPacket->wIndex.W = 0;
//     SetupPacket->wLength = Urb->UrbControlGetConfigurationRequest.TransferBufferLength;
//
//     Urb->UrbControlGetConfigurationRequest.Reserved0 |= USBD_TRANSFER_DIRECTION_IN; // 1;
//     Urb->UrbControlGetConfigurationRequest.Reserved0 |= USBD_SHORT_TRANSFER_OK; // 2
//
//     USBPORT_DumpingSetupPacket(SetupPacket);
//
//     USBPORT_QueueTransferUrb(Urb);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleGetCurrentFrame(IN PDEVICE_OBJECT FdoDevice,
//                               IN PIRP Irp,
//                               IN PURB Urb)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     ULONG FrameNumber;
//     KIRQL OldIrql;
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//     FrameNumber = Packet->Get32BitFrameNumber(FdoExtension->MiniPortExt);
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//     Urb->UrbGetCurrentFrameNumber.FrameNumber = FrameNumber;
//
//     DPRINT_URB!("USBPORT_HandleGetCurrentFrame: FrameNumber - %p\n",
//                FrameNumber);
//
//     return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_AbortPipe(IN PDEVICE_OBJECT FdoDevice,
//                   IN PIRP Irp,
//                   IN PURB Urb)
// {
//     PUSBPORT_ENDPOINT Endpoint;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     NTSTATUS Status;
//
//     DPRINT_URB!("USBPORT_AbortPipe: ... \n");
//
//     PipeHandle = Urb->UrbPipeRequest.PipeHandle;
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//
//     if (USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle))
//     {
//         if (!(PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE))
//         {
//             Endpoint = PipeHandle->Endpoint;
//
//             Status = STATUS_PENDING;
//
//             Irp->IoStatus.Status = Status;
//             IoMarkIrpPending(Irp);
//
//             USBPORT_AbortEndpoint(FdoDevice, Endpoint, Irp);
//
//             return Status;
//         }
//
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//     }
//     else
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_PIPE_HANDLE);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ResetPipe(IN PDEVICE_OBJECT FdoDevice,
//                   IN PIRP Irp,
//                   IN PURB Urb)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSBPORT_ENDPOINT Endpoint;
//     NTSTATUS Status;
//
//     DPRINT_URB!("USBPORT_ResetPipe: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     PipeHandle = Urb->UrbPipeRequest.PipeHandle;
//
//     if (!USBPORT_ValidatePipeHandle((PUSBPORT_DEVICE_HANDLE)Urb->UrbHeader.UsbdDeviceHandle,
//                                     PipeHandle))
//     {
//         return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);
//     }
//
//     Endpoint = PipeHandle->Endpoint;
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     if (IsListEmpty(&Endpoint->TransferList))
//     {
//         if (Urb->UrbHeader.UsbdFlags & USBD_FLAG_NOT_ISO_TRANSFER)
//         {
//             KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//
//             Packet->SetEndpointDataToggle(FdoExtension->MiniPortExt,
//                                           Endpoint + 1,
//                                           0);
//
//             KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//         }
//
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//     }
//     else
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_ERROR_BUSY);
//     }
//
//     Endpoint->Flags |= ENDPOINT_FLAG_QUEUENE_EMPTY;
//
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//
//     Packet->SetEndpointStatus(FdoExtension->MiniPortExt,
//                               Endpoint + 1,
//                               USBPORT_ENDPOINT_RUN);
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ClearStall(IN PDEVICE_OBJECT FdoDevice,
//                    IN PIRP Irp,
//                    IN PURB Urb)
// {
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     USBD_STATUS USBDStatus;
//     PUSBPORT_ENDPOINT Endpoint;
//     NTSTATUS Status;
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     DPRINT_URB!("USBPORT_ClearStall: ... \n");
//
//     PipeHandle = Urb->UrbPipeRequest.PipeHandle;
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//
//     if (!USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle))
//     {
//         return USBPORT_USBDStatusToNtStatus(Urb,
//                                             USBD_STATUS_INVALID_PIPE_HANDLE);
//     }
//
//     Endpoint = PipeHandle->Endpoint;
//
//     RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT);
//     SetupPacket.bRequest = USB_REQUEST_CLEAR_FEATURE;
//     SetupPacket.wValue.W = 0;
//     SetupPacket.wIndex.W = Endpoint->EndpointProperties.EndpointAddress;
//     SetupPacket.wLength = 0;
//
//     USBPORT_SendSetupPacket(DeviceHandle,
//                             FdoDevice,
//                             &SetupPacket,
//                             NULL,
//                             0,
//                             NULL,
//                             &USBDStatus);
//
//     Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_SyncResetPipeAndClearStall(IN PDEVICE_OBJECT FdoDevice,
//                                    IN PIRP Irp,
//                                    IN PURB Urb)
// {
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSBPORT_ENDPOINT Endpoint;
//     ULONG EndpointState;
//     NTSTATUS Status;
//
//     DPRINT_URB!("USBPORT_SyncResetPipeAndClearStall: ... \n");
//
//     ASSERT(Urb->UrbHeader.UsbdDeviceHandle);
//     ASSERT(Urb->UrbHeader.Length == sizeof(struct _URB_PIPE_REQUEST));
//     ASSERT(Urb->UrbPipeRequest.PipeHandle);
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//     PipeHandle = Urb->UrbPipeRequest.PipeHandle;
//
//     if (!USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle))
//     {
//         return USBPORT_USBDStatusToNtStatus(Urb,
//                                             USBD_STATUS_INVALID_PIPE_HANDLE);
//     }
//
//     if (PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE)
//     {
//         return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//     }
//
//     Endpoint = PipeHandle->Endpoint;
//     InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//
//     if (Endpoint->EndpointProperties.TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         Urb->UrbHeader.UsbdFlags |= USBD_FLAG_NOT_ISO_TRANSFER;
//         Status = USBPORT_ClearStall(FdoDevice, Irp, Urb);
//     }
//     else
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         Status = USBPORT_ResetPipe(FdoDevice, Irp, Urb);
//
//         if (Endpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             while (TRUE)
//             {
//                 KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                                   &Endpoint->EndpointOldIrql);
//
//                 EndpointState = USBPORT_GetEndpointState(Endpoint);
//
//                 if (EndpointState == USBPORT_ENDPOINT_PAUSED &&
//                     IsListEmpty(&Endpoint->TransferList))
//                 {
//                     USBPORT_SetEndpointState(Endpoint,
//                                              USBPORT_ENDPOINT_ACTIVE);
//                 }
//
//                 KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                                   Endpoint->EndpointOldIrql);
//
//                 if (EndpointState == USBPORT_ENDPOINT_ACTIVE)
//                 {
//                     break;
//                 }
//
//                 USBPORT_Wait(FdoDevice, 1);
//             }
//         }
//     }
//
//     InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleSetOrClearFeature(IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     DPRINT_URB!("USBPORT_HandleSetOrClearFeature: Urb - %p\n", Urb);
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)
//                   &Urb->UrbControlFeatureRequest.Reserved0;
//
//     SetupPacket->wLength = 0;
//     Urb->UrbControlFeatureRequest.Reserved3 = 0; // TransferBufferLength
//
//     SetupPacket->bmRequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//
//     switch (Urb->UrbHeader.Function)
//     {
//         case URB_FUNCTION_SET_FEATURE_TO_DEVICE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_SET_FEATURE_TO_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_SET_FEATURE_TO_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_CLEAR_FEATURE_TO_DEVICE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT\n");
//             SetupPacket->bRequest = USB_REQUEST_CLEAR_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_CLEAR_FEATURE_TO_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT\n");
//             SetupPacket->bRequest = USB_REQUEST_CLEAR_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_CLEAR_FEATURE_TO_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE\n");
//             SetupPacket->bRequest = USB_REQUEST_CLEAR_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_CLEAR_FEATURE_TO_OTHER:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n");
//             SetupPacket->bRequest = USB_REQUEST_CLEAR_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_OTHER;
//             break;
//
//         case URB_FUNCTION_SET_FEATURE_TO_OTHER:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_FEATURE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_OTHER;
//             break;
//     }
//
//     Urb->UrbControlFeatureRequest.Reserved2 &= ~USBD_TRANSFER_DIRECTION_IN;
//     Urb->UrbControlFeatureRequest.Reserved2 |= USBD_SHORT_TRANSFER_OK;
//
//     USBPORT_DumpingSetupPacket(SetupPacket);
//
//     USBPORT_QueueTransferUrb(Urb);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleDataTransfers(IN PURB Urb)
// {
//     PUSBPORT_ENDPOINT Endpoint;
//
//     DPRINT_URB!("USBPORT_HandleDataTransfers: Urb - %p\n", Urb);
//
//     Endpoint = ((PUSBPORT_PIPE_HANDLE)
//                 (Urb->UrbBulkOrInterruptTransfer.PipeHandle))->Endpoint;
//
//     if (Endpoint->EndpointProperties.TransferType != USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         if (Endpoint->EndpointProperties.Direction == USBPORT_TRANSFER_DIRECTION_OUT)
//         {
//             Urb->UrbBulkOrInterruptTransfer.TransferFlags &= ~USBD_TRANSFER_DIRECTION_IN;
//         }
//         else
//         {
//             Urb->UrbBulkOrInterruptTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
//         }
//     }
//
//     USBPORT_QueueTransferUrb(Urb);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleGetStatus(IN PIRP Irp,
//                         IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     NTSTATUS Status;
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)
//                    &Urb->UrbControlDescriptorRequest.Reserved1;
//
//     SetupPacket->bmRequestType.B = 0;
//     SetupPacket->bmRequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//     SetupPacket->bRequest = USB_REQUEST_GET_STATUS;
//     SetupPacket->wLength = Urb->UrbControlDescriptorRequest.TransferBufferLength;
//     SetupPacket->wValue.W = 0;
//
//     switch (Urb->UrbHeader.Function)
//     {
//         case URB_FUNCTION_GET_STATUS_FROM_DEVICE:
//             DPRINT_URB!("USBPORT_HandleGetStatus: URB_FUNCTION_GET_STATUS_FROM_DEVICE\n");
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_GET_STATUS_FROM_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleGetStatus: URB_FUNCTION_GET_STATUS_FROM_INTERFACE\n");
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_GET_STATUS_FROM_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleGetStatus: URB_FUNCTION_GET_STATUS_FROM_ENDPOINT\n");
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_GET_STATUS_FROM_OTHER:
//             DPRINT_URB!("USBPORT_HandleGetStatus: URB_FUNCTION_GET_STATUS_FROM_OTHER\n");
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_OTHER;
//             break;
//     }
//
//     if (SetupPacket->wLength == 2)
//     {
//         Urb->UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;
//
//         if (SetupPacket->bmRequestType.Dir)
//             Urb->UrbControlTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
//         else
//             Urb->UrbControlTransfer.TransferFlags &= ~USBD_TRANSFER_DIRECTION_IN;
//
//         //USBPORT_DumpingSetupPacket(SetupPacket);
//
//         USBPORT_QueueTransferUrb(Urb);
//
//         Status = STATUS_PENDING;
//     }
//     else
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_PARAMETER);
//
//         DPRINT1("USBPORT_HandleGetStatus: Bad wLength\n");
//         USBPORT_DumpingSetupPacket(SetupPacket);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleVendorOrClass(IN PIRP Irp,
//                             IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     /*
//         Specifies a value, from 4 to 31 inclusive,
//         that becomes part of the request type code in the USB-defined setup packet.
//         This value is defined by USB for a class request or the vendor for a vendor request.
//     */
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)
//                    &Urb->UrbControlDescriptorRequest.Reserved1;
//
//     SetupPacket->bmRequestType.Dir = USBD_TRANSFER_DIRECTION_FLAG
//                                      (Urb->UrbControlTransfer.TransferFlags);
//
//     SetupPacket->wLength = Urb->UrbControlDescriptorRequest.TransferBufferLength;
//
//     Urb->UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;
//
//     switch (Urb->UrbHeader.Function)
//     {
//         case URB_FUNCTION_VENDOR_DEVICE:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_DEVICE\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_VENDOR;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_VENDOR_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_INTERFACE\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_VENDOR;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_VENDOR_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_ENDPOINT\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_VENDOR;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_CLASS_DEVICE:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_DEVICE\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_CLASS;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_CLASS_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_INTERFACE\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_CLASS;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_CLASS_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_ENDPOINT\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_CLASS;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_CLASS_OTHER:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_CLASS_OTHER\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_CLASS;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_OTHER;
//             break;
//
//         case URB_FUNCTION_VENDOR_OTHER:
//             DPRINT_URB!("USBPORT_HandleVendorOrClass: URB_FUNCTION_VENDOR_OTHER\n");
//             SetupPacket->bmRequestType.Type = BMREQUEST_VENDOR;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_OTHER;
//             break;
//     }
//
//     USBPORT_DumpingSetupPacket(SetupPacket);
//
//     USBPORT_QueueTransferUrb(Urb);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleGetSetDescriptor(IN PIRP Irp,
//                                IN PURB Urb)
// {
//     PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     SetupPacket = (PUSB_DEFAULT_PIPE_SETUP_PACKET)
//                    &Urb->UrbControlDescriptorRequest.Reserved1;
//
//     SetupPacket->wLength = Urb->UrbControlDescriptorRequest.TransferBufferLength;
//     SetupPacket->bmRequestType.B = 0; // Clear bmRequestType
//     SetupPacket->bmRequestType.Type = BMREQUEST_STANDARD;
//
//     switch (Urb->UrbHeader.Function)
//     {
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE\n");
//             SetupPacket->bRequest = USB_REQUEST_GET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_DEVICE;
//             break;
//
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT\n");
//             SetupPacket->bRequest = USB_REQUEST_GET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_ENDPOINT;
//             break;
//
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE\n");
//             SetupPacket->bRequest = USB_REQUEST_GET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_DEVICE_TO_HOST;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE:
//             DPRINT_URB!("USBPORT_HandleGetSetDescriptor: URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE\n");
//             SetupPacket->bRequest = USB_REQUEST_SET_DESCRIPTOR;
//             SetupPacket->bmRequestType.Dir = BMREQUEST_HOST_TO_DEVICE;
//             SetupPacket->bmRequestType.Recipient = BMREQUEST_TO_INTERFACE;
//             break;
//     }
//
//     Urb->UrbControlTransfer.TransferFlags |= USBD_SHORT_TRANSFER_OK;
//
//     if (SetupPacket->bmRequestType.Dir)
//         Urb->UrbControlTransfer.TransferFlags |= USBD_TRANSFER_DIRECTION_IN;
//     else
//         Urb->UrbControlTransfer.TransferFlags &= ~USBD_TRANSFER_DIRECTION_IN;
//
//     USBPORT_DumpingSetupPacket(SetupPacket);
//
//     USBPORT_QueueTransferUrb(Urb);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ValidateTransferParametersURB(IN PURB Urb)
// {
//     struct _URB_CONTROL_TRANSFER *UrbRequest;
//     PMDL Mdl;
//
//     DPRINT_URB!("USBPORT_ValidateTransferParametersURB: Urb - %p\n", Urb);
//
//     UrbRequest = &Urb->UrbControlTransfer;
//
//     if (UrbRequest->TransferBuffer == NULL &&
//         UrbRequest->TransferBufferMDL == NULL &&
//         UrbRequest->TransferBufferLength > 0)
//     {
//         DPRINT1("USBPORT_ValidateTransferParametersURB: Not valid parameter\n");
//         USBPORT_DumpingURB(Urb);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((UrbRequest->TransferBuffer != NULL) &&
//         (UrbRequest->TransferBufferMDL != NULL) &&
//         UrbRequest->TransferBufferLength == 0)
//     {
//         DPRINT1("USBPORT_ValidateTransferParametersURB: Not valid parameter\n");
//         USBPORT_DumpingURB(Urb);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (UrbRequest->TransferBuffer != NULL &&
//         UrbRequest->TransferBufferMDL == NULL &&
//         UrbRequest->TransferBufferLength != 0)
//     {
//         DPRINT_URB!("USBPORT_ValidateTransferParametersURB: TransferBuffer - %p, TransferBufferLength - %x\n",
//                    UrbRequest->TransferBuffer,
//                    UrbRequest->TransferBufferLength);
//
//         Mdl = IoAllocateMdl(UrbRequest->TransferBuffer,
//                             UrbRequest->TransferBufferLength,
//                             FALSE,
//                             FALSE,
//                             NULL);
//
//         if (!Mdl)
//         {
//             DPRINT1("USBPORT_ValidateTransferParametersURB: Not allocated Mdl\n");
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         MmBuildMdlForNonPagedPool(Mdl);
//
//         UrbRequest->TransferBufferMDL = Mdl;
//         Urb->UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_MDL;
//
//         DPRINT_URB!("USBPORT_ValidateTransferParametersURB: Mdl - %p\n", Mdl);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ValidateURB(IN PDEVICE_OBJECT FdoDevice,
//                     IN PIRP Irp,
//                     IN PURB Urb,
//                     IN BOOLEAN IsControlTransfer,
//                     IN BOOLEAN IsNullTransfer)
// {
//     struct _URB_CONTROL_TRANSFER *UrbRequest;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     NTSTATUS Status;
//     USBD_STATUS USBDStatus;
//
//     UrbRequest = &Urb->UrbControlTransfer;
//
//     if (UrbRequest->UrbLink)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_PARAMETER);
//
//         DPRINT1("USBPORT_ValidateURB: Not valid parameter\n");
//
//         USBPORT_DumpingURB(Urb);
//         return Status;
//     }
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//
//     if (IsControlTransfer)
//     {
//         UrbRequest->TransferFlags |= USBD_DEFAULT_PIPE_TRANSFER;
//         UrbRequest->PipeHandle = &DeviceHandle->PipeHandle;
//     }
//
//     if (UrbRequest->TransferFlags & USBD_DEFAULT_PIPE_TRANSFER)
//     {
//         if (UrbRequest->TransferBufferLength > 0x1000)
//         {
//             Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                                   USBD_STATUS_INVALID_PARAMETER);
//
//             DPRINT1("USBPORT_ValidateURB: Not valid parameter\n");
//
//             USBPORT_DumpingURB(Urb);
//             return Status;
//         }
//
//         if (Urb->UrbHeader.Function == URB_FUNCTION_CONTROL_TRANSFER)
//         {
//             UrbRequest->PipeHandle = &DeviceHandle->PipeHandle;
//         }
//     }
//
//     if (!USBPORT_ValidatePipeHandle(DeviceHandle, UrbRequest->PipeHandle))
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_PIPE_HANDLE);
//
//         DPRINT1("USBPORT_ValidateURB: Not valid pipe handle\n");
//
//         USBPORT_DumpingURB(Urb);
//         return Status;
//     }
//
//     UrbRequest->hca.Reserved8[0] = NULL; // Transfer
//
//     if (IsNullTransfer)
//     {
//         UrbRequest->TransferBuffer = 0;
//         UrbRequest->TransferBufferMDL = NULL;
//         UrbRequest->TransferBufferLength = 0;
//     }
//     else
//     {
//         Status = USBPORT_ValidateTransferParametersURB(Urb);
//
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     USBDStatus = USBPORT_AllocateTransfer(FdoDevice,
//                                           Urb,
//                                           DeviceHandle,
//                                           Irp,
//                                           NULL);
//
//     Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBPORT_ValidateURB: Not allocated transfer\n");
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleSubmitURB(IN PDEVICE_OBJECT PdoDevice,
//                         IN PIRP Irp,
//                         IN PURB Urb)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     USHORT Function;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     NTSTATUS Status = STATUS_NOT_IMPLEMENTED;
//
//     ASSERT(Urb);
//
//     PdoExtension = PdoDevice->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Urb->UrbHeader.Status = USBD_STATUS_SUCCESS;
//     Urb->UrbHeader.UsbdFlags = 0;
//
//     Function = Urb->UrbHeader.Function;
//
//     if (Function > URB_FUNCTION_MAX)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_URB_FUNCTION);
//
//         DPRINT1("USBPORT_HandleSubmitURB: Unknown URB function - %x !!!\n",
//                Function);
//
//         return Status;
//     }
//
//     if (FdoExtension->TimerFlags & USBPORT_TMFLAG_RH_SUSPENDED)
//     {
//         DPRINT1("USBPORT_HandleSubmitURB: Bad Request\n");
//
//         USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_DEVICE_GONE);
//
//         Irp->IoStatus.Status = STATUS_PENDING;
//         IoMarkIrpPending(Irp);
//         IoCsqInsertIrp(&FdoExtension->BadRequestIoCsq, Irp, NULL);
//
//         return STATUS_PENDING;
//     }
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//
//     if (!DeviceHandle)
//     {
//         DeviceHandle = &PdoExtension->DeviceHandle;
//         Urb->UrbHeader.UsbdDeviceHandle = DeviceHandle;
//     }
//
//     if (!USBPORT_ValidateDeviceHandle(PdoExtension->FdoDevice,
//                                       DeviceHandle))
//     {
//         DPRINT1("USBPORT_HandleSubmitURB: Not valid device handle\n");
//
//         Irp->IoStatus.Status = STATUS_PENDING;
//         IoMarkIrpPending(Irp);
//         IoCsqInsertIrp(&FdoExtension->BadRequestIoCsq, Irp, NULL);
//
//         return STATUS_PENDING;
//     }
//
//     InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//
//     DPRINT_URB!("USBPORT_HandleSubmitURB: Function - 0x%02X, DeviceHandle - %p\n",
//                Function,
//                Urb->UrbHeader.UsbdDeviceHandle);
//
//     switch (Function)
//     {
//         case URB_FUNCTION_ISOCH_TRANSFER:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_ISOCH_TRANSFER UNIMPLEMENTED. FIXME. \n");
//             break;
//
//         case URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER:
//         case URB_FUNCTION_CONTROL_TRANSFER:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, FALSE, FALSE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleDataTransfers(Urb);
//             break;
//
//         case URB_FUNCTION_VENDOR_DEVICE:
//         case URB_FUNCTION_VENDOR_INTERFACE:
//         case URB_FUNCTION_VENDOR_ENDPOINT:
//         case URB_FUNCTION_CLASS_DEVICE:
//         case URB_FUNCTION_CLASS_INTERFACE:
//         case URB_FUNCTION_CLASS_ENDPOINT:
//         case URB_FUNCTION_CLASS_OTHER:
//         case URB_FUNCTION_VENDOR_OTHER:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, TRUE, FALSE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleVendorOrClass(Irp, Urb);
//             break;
//
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_DEVICE:
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_DEVICE:
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT:
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_ENDPOINT:
//         case URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE:
//         case URB_FUNCTION_SET_DESCRIPTOR_TO_INTERFACE:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, TRUE, FALSE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleGetSetDescriptor(Irp, Urb);
//             break;
//
//         case URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_GET_MS_FEATURE_DESCRIPTOR (0x2A) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb,
//                                                 USBD_STATUS_INVALID_URB_FUNCTION);
//
//         case URB_FUNCTION_GET_STATUS_FROM_DEVICE:
//         case URB_FUNCTION_GET_STATUS_FROM_INTERFACE:
//         case URB_FUNCTION_GET_STATUS_FROM_ENDPOINT:
//         case URB_FUNCTION_GET_STATUS_FROM_OTHER:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, TRUE, FALSE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleGetStatus(Irp, Urb);
//             break;
//
//         case URB_FUNCTION_SELECT_CONFIGURATION:
//             Status = USBPORT_HandleSelectConfiguration(PdoExtension->FdoDevice,
//                                                        Irp,
//                                                        Urb);
//             break;
//
//         case URB_FUNCTION_SELECT_INTERFACE:
//             Status = USBPORT_HandleSelectInterface(PdoExtension->FdoDevice,
//                                                    Irp,
//                                                    Urb);
//             break;
//
//         case URB_FUNCTION_GET_CONFIGURATION:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, TRUE, FALSE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleGetConfiguration(Urb);
//             break;
//
//         case URB_FUNCTION_GET_INTERFACE:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_GET_INTERFACE (0x27) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb,
//                                                 USBD_STATUS_INVALID_URB_FUNCTION);
//
//         case URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL:
//             Status = USBPORT_SyncResetPipeAndClearStall(PdoExtension->FdoDevice,
//                                                         Irp,
//                                                         Urb);
//             break;
//
//         case URB_FUNCTION_SYNC_RESET_PIPE:
//             Status = USBPORT_ResetPipe(PdoExtension->FdoDevice,
//                                        Irp,
//                                        Urb);
//             break;
//
//         case URB_FUNCTION_SYNC_CLEAR_STALL:
//             Status = USBPORT_ClearStall(PdoExtension->FdoDevice,
//                                         Irp,
//                                         Urb);
//             break;
//
//         case URB_FUNCTION_ABORT_PIPE:
//             Status = USBPORT_AbortPipe(PdoExtension->FdoDevice,
//                                        Irp,
//                                        Urb);
//             break;
//
//         case URB_FUNCTION_SET_FEATURE_TO_DEVICE:
//         case URB_FUNCTION_SET_FEATURE_TO_INTERFACE:
//         case URB_FUNCTION_SET_FEATURE_TO_ENDPOINT:
//         case URB_FUNCTION_CLEAR_FEATURE_TO_INTERFACE:
//         case URB_FUNCTION_CLEAR_FEATURE_TO_ENDPOINT:
//         case URB_FUNCTION_CLEAR_FEATURE_TO_OTHER:
//         case URB_FUNCTION_SET_FEATURE_TO_OTHER:
//             Status = USBPORT_ValidateURB(FdoDevice, Irp, Urb, TRUE, TRUE);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("USBPORT_HandleSubmitURB: Not valid URB\n");
//                 break;
//             }
//
//             Status = USBPORT_HandleSetOrClearFeature(Urb);
//             break;
//
//         case URB_FUNCTION_GET_CURRENT_FRAME_NUMBER:
//             Status = USBPORT_HandleGetCurrentFrame(PdoExtension->FdoDevice,
//                                                    Irp,
//                                                    Urb);
//             break;
//
//         case URB_FUNCTION_TAKE_FRAME_LENGTH_CONTROL:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_TAKE_FRAME_LENGTH_CONTROL (0x03) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
//
//         case URB_FUNCTION_RELEASE_FRAME_LENGTH_CONTROL:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_RELEASE_FRAME_LENGTH_CONTROL (0x04) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
//
//         case URB_FUNCTION_GET_FRAME_LENGTH:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_GET_FRAME_LENGTH (0x05) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
//
//         case URB_FUNCTION_SET_FRAME_LENGTH:
//             DPRINT1("USBPORT_HandleSubmitURB: URB_FUNCTION_SET_FRAME_LENGTH (0x06) NOT_SUPPORTED\n");
//             return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_NOT_SUPPORTED);
//
//         default:
//             DPRINT1("USBPORT_HandleSubmitURB: Unknown URB Function - %x\n",
//                     Function);
//             //URB_FUNCTION_RESERVED_0X0016
//             //URB_FUNCTION_RESERVE_0X001D
//             //URB_FUNCTION_RESERVE_0X002B
//             //URB_FUNCTION_RESERVE_0X002C
//             //URB_FUNCTION_RESERVE_0X002D
//             //URB_FUNCTION_RESERVE_0X002E
//             //URB_FUNCTION_RESERVE_0X002F
//             break;
//     }
//
//     if (Status == STATUS_PENDING)
//     {
//         return Status;
//     }
//
//     if (Urb->UrbHeader.UsbdFlags & USBD_FLAG_ALLOCATED_TRANSFER)
//     {
//         PUSBPORT_TRANSFER Transfer;
//
//         Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//         Urb->UrbControlTransfer.hca.Reserved8[0] = NULL;
//         Urb->UrbHeader.UsbdFlags |= ~USBD_FLAG_ALLOCATED_TRANSFER;
//         ExFreePoolWithTag(Transfer, USB_PORT_TAG);
//     }
//
//     InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

extern "system" {
    fn USBPORT_SendSetupPacket(
        DeviceHandle: PUSBPORT_DEVICE_HANDLE,
        FdoDevice: PDEVICE_OBJECT,
        SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET,
        Buffer: PVOID,
        Length: ULONG,
        TransferedLen: PULONG,
        pUSBDStatus: *mut USBD_STATUS,
    ) -> NTSTATUS;
}

// Translated from urb.c USBPORT_ClearStall (was a commented lazy-stub in the pure-LLM
// pass; hand-translated to finish usbport at 100% Rust).
#[no_mangle]
pub unsafe extern "system" fn USBPORT_ClearStall(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut USBDStatus: USBD_STATUS = 0;
    let Endpoint: PUSBPORT_ENDPOINT;
    let Status: NTSTATUS;
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();

    PipeHandle = (*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle as PUSBPORT_PIPE_HANDLE;
    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;

    if USBPORT_ValidatePipeHandle(DeviceHandle, PipeHandle) == 0 {
        return USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_PIPE_HANDLE);
    }

    Endpoint = (*PipeHandle).Endpoint;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(SetupPacket) as PVOID,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as _,
    );

    SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_ENDPOINT as u8);
    SetupPacket.bRequest = USB_REQUEST_CLEAR_FEATURE as u8;
    SetupPacket.wValue.W = 0;
    SetupPacket.wIndex.W = (*Endpoint).EndpointProperties.EndpointAddress as u16;
    SetupPacket.wLength = 0;

    USBPORT_SendSetupPacket(
        DeviceHandle,
        FdoDevice,
        core::ptr::addr_of_mut!(SetupPacket),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(USBDStatus),
    );

    Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
    return Status;
}
