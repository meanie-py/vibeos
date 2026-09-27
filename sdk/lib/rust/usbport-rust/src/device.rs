use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateMdl, KEVENT, KeInitializeEvent, KeReleaseSemaphore, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIRP, PMDL, RtlCompareMemory, _KWAIT_REASON_Suspended};
use core::ffi::c_void;

extern "system" {
    fn USBD_SUCCESS(Status: USBD_STATUS) -> bool;
    fn USBPORT_AllocateTransfer( FdoDevice: PDEVICE_OBJECT, Urb: PURB, UsbdPipeHandle: PVOID, Irp: PIRP, Event: *mut KEVENT, ) -> USBD_STATUS;
    fn USBPORT_QueueTransferUrb(Urb: PURB);
    fn USBPORT_USBDStatusToNtStatus(Urb: PURB, USBDStatus: USBD_STATUS) -> NTSTATUS;
    fn USBPORT_ClosePipe( DeviceHandle: PUSBPORT_DEVICE_HANDLE, FdoDevice: PDEVICE_OBJECT, PipeHandle: PUSBPORT_PIPE_HANDLE, );
    fn USBPORT_EndpointHasQueuedTransfers( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, CompletedCount: *mut ULONG, ) -> BOOLEAN;
    fn USB2_InitTT(HcExtension: PUSB2_HC_EXTENSION, Tt: PUSB2_TT,);
    fn USBPORT_UpdateAllocatedBwTt(TtExtension: PUSB2_TT_EXTENSION);
    fn USBPORT_OpenPipe( FdoDevice: PDEVICE_OBJECT, DeviceHandle: PUSBPORT_DEVICE_HANDLE, PipeHandle: PUSBPORT_PIPE_HANDLE, Irp: PIRP, ) -> NTSTATUS;
    fn USBPORT_ReopenPipe(FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT) -> NTSTATUS;
    fn USBPORT_Wait(MiniPortExtension: PVOID, Milliseconds: ULONG,) -> NTSTATUS;
    fn MiniportCloseEndpoint(FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT);
    fn MiniportOpenEndpoint(FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT,) -> MPSTATUS;
    fn USBD_ERROR(Status: USBD_STATUS) -> BOOLEAN;
    fn USBPORT_AddPipeHandle( DeviceHandle: PUSBPORT_DEVICE_HANDLE, PipeHandle: PUSBPORT_PIPE_HANDLE, );
    fn USBPORT_RemovePipeHandle( DeviceHandle: PUSBPORT_DEVICE_HANDLE, PipeHandle: PUSBPORT_PIPE_HANDLE, );
    fn USBPORT_DumpingConfiguration(ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR);
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG  (skipped: no constant value)

// C flexible-array members land in the bindgen types as `[T; 1]`; Rust's `arr[ix]`
// bounds-checks EVEN inside addr_of_mut!, so `Pipes[1]` / `PipeHandle[1]` panics at
// runtime (found live: 2-endpoint usb-storage interface -> RUST PANIC device.rs:386).
// Index flexible arrays through raw-pointer arithmetic instead.
#[inline]
unsafe fn pipe_info_at(
    InterfaceInfo: PUSBD_INTERFACE_INFORMATION,
    ix: usize,
) -> *mut USBD_PIPE_INFORMATION {
    (core::ptr::addr_of_mut!((*InterfaceInfo).Pipes) as *mut USBD_PIPE_INFORMATION).add(ix)
}

#[inline]
unsafe fn pipe_handle_at(
    InterfaceHandle: PUSBPORT_INTERFACE_HANDLE,
    ix: usize,
) -> *mut USBPORT_PIPE_HANDLE {
    (core::ptr::addr_of_mut!((*InterfaceHandle).PipeHandle) as *mut USBPORT_PIPE_HANDLE).add(ix)
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SendSetupPacket(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
    SetupPacket: PUSB_DEFAULT_PIPE_SETUP_PACKET,
    Buffer: PVOID,
    Length: ULONG,
    TransferedLen: PULONG,
    pUSBDStatus: PUSBD_STATUS,
) -> NTSTATUS {

    const USB_PORT_TAG: u32 = 0x70627375;
    const URB_FUNCTION_CONTROL_TRANSFER: u16 = 0x0008;
    const USBD_SHORT_TRANSFER_OK: u32 = 0x00000002;
    const USBD_TRANSFER_DIRECTION: u32 = 0x00000001;
    const USBD_TRANSFER_DIRECTION_IN: u32 = 0x00000001;
    const BMREQUEST_DEVICE_TO_HOST: u32 = 1;
    const USBD_FLAG_ALLOCATED_MDL: u32 = 0x00000002; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h
    const USBD_STATUS_INSUFFICIENT_RESOURCES: USBD_STATUS = 0xC0001000u32 as i32;

    let Urb: PURB;
    let Mdl: PMDL;
    let mut USBDStatus: USBD_STATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_TRANSFER>(),
        USB_PORT_TAG,
    ) as PURB;

    if !Urb.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

        RtlZeroMemory(
            Urb as PVOID,
            core::mem::size_of::<_URB_CONTROL_TRANSFER>(),
        );

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer.SetupPacket) as PVOID,
            SetupPacket as PVOID,
            core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
        );

        (*Urb).__bindgen_anon_1.UrbHeader.Length = core::mem::size_of::<_URB_CONTROL_TRANSFER>() as u16;
        (*Urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_CONTROL_TRANSFER;
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle = DeviceHandle as PVOID;
        (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags = 0;

        (*Urb).__bindgen_anon_1.UrbControlTransfer.PipeHandle =
            core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle) as PVOID;
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength = Length;
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBuffer = Buffer;
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL = core::ptr::null_mut();

        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags =
            USBD_SHORT_TRANSFER_OK | USBD_TRANSFER_DIRECTION;

        if (*SetupPacket).bmRequestType.Dir() != (BMREQUEST_DEVICE_TO_HOST as u8) {
            (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags &= !USBD_TRANSFER_DIRECTION_IN;
        }

        Status = STATUS_SUCCESS;

        if Length != 0 {
            Mdl = IoAllocateMdl(Buffer, Length, 0, 0, core::ptr::null_mut());

            (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL = Mdl;

            if !Mdl.is_null() {
                (*Urb).__bindgen_anon_1.UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_MDL;
                MmBuildMdlForNonPagedPool(Mdl);
            } else {
                Status = USBPORT_USBDStatusToNtStatus(
                    core::ptr::null_mut(),
                    USBD_STATUS_INSUFFICIENT_RESOURCES,
                );
            }
        }

        if NT_SUCCESS(Status) {
            USBDStatus = USBPORT_AllocateTransfer(
                FdoDevice,
                Urb,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Event),
            );

            if USBD_SUCCESS(USBDStatus) {
                InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

                USBPORT_QueueTransferUrb(Urb);

                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    _KWAIT_REASON_Suspended,
                    0i32 as i8,
                    0,
                    core::ptr::null_mut(),
                );

                USBDStatus = (*Urb).__bindgen_anon_1.UrbHeader.Status;
            }

            Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);

            if !TransferedLen.is_null() {
                *TransferedLen = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength;
            }

            if !pUSBDStatus.is_null() {
                *pUSBDStatus = USBDStatus;
            }
        }

        InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));
        ExFreePoolWithTag(Urb as PVOID, USB_PORT_TAG);
    } else {
        if !pUSBDStatus.is_null() {
            *pUSBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = USBPORT_USBDStatusToNtStatus(
            core::ptr::null_mut(),
            USBD_STATUS_INSUFFICIENT_RESOURCES,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetInterfaceLength(
    iDescriptor: PUSB_INTERFACE_DESCRIPTOR,
    EndDescriptors: ULONG_PTR,
) -> ULONG {
    const USB_ENDPOINT_DESCRIPTOR_TYPE: u8 = 0x05;
    const USB_INTERFACE_DESCRIPTOR_TYPE: u8 = 0x04;

    let mut Length: SIZE_T;
    let mut Descriptor: PUSB_ENDPOINT_DESCRIPTOR;
    let mut ix: ULONG;

    // DPRINT("USBPORT_GetInterfaceLength ... \n");

    Length = (*iDescriptor).bLength as SIZE_T;
    Descriptor = ((iDescriptor as ULONG_PTR) + Length) as PUSB_ENDPOINT_DESCRIPTOR;

    if (*iDescriptor).bNumEndpoints != 0 {
        ix = 0;
        while (ix as u32) < (*iDescriptor).bNumEndpoints as u32 {
            while ((*Descriptor).bDescriptorType != USB_ENDPOINT_DESCRIPTOR_TYPE)
                && ((*Descriptor).bLength > 0)
            {
                Length += (*Descriptor).bLength as SIZE_T;
                Descriptor =
                    ((Descriptor as ULONG_PTR) + (*Descriptor).bLength as ULONG_PTR)
                        as PUSB_ENDPOINT_DESCRIPTOR;
            }

            Length += (*Descriptor).bLength as SIZE_T;
            Descriptor = ((Descriptor as ULONG_PTR) + (*Descriptor).bLength as ULONG_PTR)
                as PUSB_ENDPOINT_DESCRIPTOR;

            ix += 1;
        }
    }

    while ((Descriptor as ULONG_PTR) < EndDescriptors)
        && ((*Descriptor).bDescriptorType != USB_INTERFACE_DESCRIPTOR_TYPE)
        && ((*Descriptor).bLength > 0)
    {
        Length += (*Descriptor).bLength as SIZE_T;
        Descriptor = ((Descriptor as ULONG_PTR) + (*Descriptor).bLength as ULONG_PTR)
            as PUSB_ENDPOINT_DESCRIPTOR;
    }

    Length as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ParseConfigurationDescriptor(
    ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    InterfaceNumber: UCHAR,
    Alternate: UCHAR,
    HasAlternates: PBOOLEAN,
) -> PUSB_INTERFACE_DESCRIPTOR {
    let mut TmpDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let mut iDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut OutDescriptor: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let mut Descriptor: ULONG_PTR = ConfigDescriptor as ULONG_PTR;
    let EndDescriptors: ULONG_PTR;
    let mut ix: ULONG;

    if !HasAlternates.is_null() {
        *HasAlternates = 0;
    }

    TmpDescriptor = ((ConfigDescriptor as ULONG_PTR)
        + (*ConfigDescriptor).bLength as ULONG_PTR)
        as PUSB_CONFIGURATION_DESCRIPTOR;
    while (*TmpDescriptor).bDescriptorType != (USB_INTERFACE_DESCRIPTOR_TYPE as u8)
        && (*TmpDescriptor).bLength > 0
    {
        TmpDescriptor = ((TmpDescriptor as ULONG_PTR)
            + (*TmpDescriptor).bLength as ULONG_PTR)
            as PUSB_CONFIGURATION_DESCRIPTOR;
    }

    iDescriptor = TmpDescriptor as PUSB_INTERFACE_DESCRIPTOR;

    EndDescriptors = (ConfigDescriptor as ULONG_PTR)
        + (*ConfigDescriptor).wTotalLength as ULONG_PTR;

    while Descriptor < EndDescriptors && (*iDescriptor).bInterfaceNumber != InterfaceNumber {
        Descriptor = (iDescriptor as ULONG_PTR)
            + USBPORT_GetInterfaceLength(iDescriptor, EndDescriptors) as ULONG_PTR;

        iDescriptor = Descriptor as PUSB_INTERFACE_DESCRIPTOR;
    }

    ix = 0;

    while Descriptor < EndDescriptors && (*iDescriptor).bInterfaceNumber == InterfaceNumber {
        if (*iDescriptor).bAlternateSetting == Alternate {
            OutDescriptor = iDescriptor;
        }

        Descriptor = (iDescriptor as ULONG_PTR)
            + USBPORT_GetInterfaceLength(iDescriptor, EndDescriptors) as ULONG_PTR;

        iDescriptor = Descriptor as PUSB_INTERFACE_DESCRIPTOR;

        ix += 1;
    }

    if ix > 1 && !HasAlternates.is_null() {
        *HasAlternates = 1;
    }

    return OutDescriptor;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_OpenInterface(
    Urb: PURB,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
    ConfigHandle: PUSBPORT_CONFIGURATION_HANDLE,
    InterfaceInfo: PUSBD_INTERFACE_INFORMATION,
    iHandle: *mut PUSBPORT_INTERFACE_HANDLE,
    SendSetInterface: BOOLEAN,
) -> USBD_STATUS {

    const USBD_STATUS_SUCCESS: USBD_STATUS = 0;
    const USBD_STATUS_INSUFFICIENT_RESOURCES: USBD_STATUS = 0xC0001000u32 as USBD_STATUS;
    const BMREQUEST_HOST_TO_DEVICE: UCHAR = 0;
    const BMREQUEST_STANDARD: UCHAR = 0;
    const BMREQUEST_TO_INTERFACE: UCHAR = 1;
    const USB_REQUEST_SET_INTERFACE: UCHAR = 0x0B;
    const PIPE_HANDLE_FLAG_CLOSED: ULONG = 0x00000001; // FIXED 2026-07-05: translation invented 0x80000000; verified vs usbport.h/usbmport.h
    const USB_ENDPOINT_DESCRIPTOR_TYPE: UCHAR = 0x05;
    const USBD_PF_CHANGE_MAX_PACKET: ULONG = 0x00000001;
    const USB_ENDPOINT_TYPE_MASK: UCHAR = 0x03;
    const USB_PORT_TAG: ULONG = 0x70627375; // 'UBrt'

    let InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut InterfaceHandle: PUSBPORT_INTERFACE_HANDLE = core::ptr::null_mut();
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut Descriptor: PUSB_ENDPOINT_DESCRIPTOR;
    let mut PipeInfo: PUSBD_PIPE_INFORMATION;
    let mut HasAlternates: BOOLEAN = 0;
    let NumEndpoints: ULONG;
    let Length: SIZE_T;
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();
    let HandleLength: SIZE_T;
    let mut IsAllocated: BOOLEAN = 0;
    let mut MaxPacketSize: USHORT;
    let mut wMaxPacketSize: USHORT;
    let mut ix: ULONG;
    let mut USBDStatus: USBD_STATUS = USBD_STATUS_SUCCESS;
    let mut Status: NTSTATUS;

    InterfaceDescriptor = USBPORT_ParseConfigurationDescriptor(
        (*ConfigHandle).ConfigurationDescriptor,
        (*InterfaceInfo).InterfaceNumber,
        (*InterfaceInfo).AlternateSetting,
        core::ptr::addr_of_mut!(HasAlternates),
    );

    NumEndpoints = (*InterfaceDescriptor).bNumEndpoints as ULONG;

    Length = (core::mem::offset_of!(USBD_INTERFACE_INFORMATION, Pipes)
        + NumEndpoints as usize * core::mem::size_of::<USBD_PIPE_INFORMATION>())
        as SIZE_T;

    'Exit: loop {
        if HasAlternates != 0 && SendSetInterface != 0 {
            RtlZeroMemory(
                core::ptr::addr_of_mut!(SetupPacket) as PVOID,
                core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as SIZE_T,
            );

            SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
            SetupPacket.bmRequestType.set_Type(BMREQUEST_STANDARD);
            SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);
            SetupPacket.bRequest = USB_REQUEST_SET_INTERFACE;
            SetupPacket.wValue.W = (*InterfaceInfo).AlternateSetting as USHORT;
            SetupPacket.wIndex.W = (*InterfaceInfo).InterfaceNumber as USHORT;
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
            if !USBD_SUCCESS(USBDStatus) {
                break 'Exit;
            }
        }

        if !(*iHandle).is_null() {
            InterfaceHandle = *iHandle;
        } else {
            HandleLength = (core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, PipeHandle)
                + NumEndpoints as usize * core::mem::size_of::<USBPORT_PIPE_HANDLE>())
                as SIZE_T;

            InterfaceHandle = ExAllocatePoolWithTag(NonPagedPool, HandleLength, USB_PORT_TAG)
                as PUSBPORT_INTERFACE_HANDLE;

            if InterfaceHandle.is_null() {
                USBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
                break 'Exit;
            }

            RtlZeroMemory(InterfaceHandle as PVOID, HandleLength);

            ix = 0;
            while ix < NumEndpoints {
                PipeHandle = pipe_handle_at(InterfaceHandle, ix as usize);

                (*PipeHandle).Flags = PIPE_HANDLE_FLAG_CLOSED;
                (*PipeHandle).Endpoint = core::ptr::null_mut();

                ix += 1;
            }

            IsAllocated = 1;
        }

        (*InterfaceHandle).AlternateSetting = (*InterfaceInfo).AlternateSetting;

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*InterfaceHandle).InterfaceDescriptor) as PVOID,
            InterfaceDescriptor as PVOID,
            core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>() as SIZE_T,
        );

        (*InterfaceInfo).Class = (*InterfaceDescriptor).bInterfaceClass;
        (*InterfaceInfo).SubClass = (*InterfaceDescriptor).bInterfaceSubClass;
        (*InterfaceInfo).Protocol = (*InterfaceDescriptor).bInterfaceProtocol;
        (*InterfaceInfo).Reserved = 0;
        (*InterfaceInfo).NumberOfPipes = (*InterfaceDescriptor).bNumEndpoints as ULONG;

        Descriptor = ((InterfaceDescriptor as ULONG_PTR)
            + (*InterfaceDescriptor).bLength as ULONG_PTR)
            as PUSB_ENDPOINT_DESCRIPTOR;

        ix = 0;
        while ix < NumEndpoints {
            PipeHandle = pipe_handle_at(InterfaceHandle, ix as usize);

            while (*Descriptor).bDescriptorType != USB_ENDPOINT_DESCRIPTOR_TYPE {
                if (*Descriptor).bLength == 0 {
                    break;
                } else {
                    Descriptor = ((Descriptor as ULONG_PTR)
                        + (*Descriptor).bLength as ULONG_PTR)
                        as PUSB_ENDPOINT_DESCRIPTOR;
                }
            }

            if (*pipe_info_at(InterfaceInfo, ix as usize)).PipeFlags & USBD_PF_CHANGE_MAX_PACKET != 0 {
                (*Descriptor).wMaxPacketSize =
                    (*pipe_info_at(InterfaceInfo, ix as usize)).MaximumPacketSize as USHORT;
            }

            RtlCopyMemory(
                core::ptr::addr_of_mut!((*PipeHandle).EndpointDescriptor) as PVOID,
                Descriptor as PVOID,
                core::mem::size_of::<USB_ENDPOINT_DESCRIPTOR>() as SIZE_T,
            );

            (*PipeHandle).Flags = PIPE_HANDLE_FLAG_CLOSED;
            (*PipeHandle).PipeFlags = (*pipe_info_at(InterfaceInfo, ix as usize)).PipeFlags;
            (*PipeHandle).Endpoint = core::ptr::null_mut();

            wMaxPacketSize = (*Descriptor).wMaxPacketSize;

            MaxPacketSize =
                (wMaxPacketSize & 0x7FF) * (((wMaxPacketSize >> 11) & 3) + 1);

            (*pipe_info_at(InterfaceInfo, ix as usize)).EndpointAddress =
                (*Descriptor).bEndpointAddress;
            (*pipe_info_at(InterfaceInfo, ix as usize)).PipeType =
                ((*Descriptor).bmAttributes & USB_ENDPOINT_TYPE_MASK) as USBD_PIPE_TYPE;
            (*pipe_info_at(InterfaceInfo, ix as usize)).MaximumPacketSize = MaxPacketSize as USHORT;
            (*pipe_info_at(InterfaceInfo, ix as usize)).PipeHandle = (-1isize) as USBD_PIPE_HANDLE;
            (*pipe_info_at(InterfaceInfo, ix as usize)).Interval = (*Descriptor).bInterval;

            Descriptor = ((Descriptor as ULONG_PTR) + (*Descriptor).bLength as ULONG_PTR)
                as PUSB_ENDPOINT_DESCRIPTOR;

            ix += 1;
        }

        if USBD_SUCCESS(USBDStatus) {
            ix = 0;
            while ix < NumEndpoints {
                PipeInfo = pipe_info_at(InterfaceInfo, ix as usize);
                PipeHandle = pipe_handle_at(InterfaceHandle, ix as usize);

                Status = USBPORT_OpenPipe(
                    FdoDevice,
                    DeviceHandle,
                    PipeHandle,
                    (core::ptr::addr_of_mut!(USBDStatus) as PIRP),
                );

                if !NT_SUCCESS(Status) {
                    break;
                }

                (*PipeInfo).PipeHandle = PipeHandle as USBD_PIPE_HANDLE;

                ix += 1;
            }

            if NumEndpoints != 0 {
                USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
            }
        }

        break 'Exit;
    }

    if USBD_SUCCESS(USBDStatus) {
        (*InterfaceInfo).InterfaceHandle = InterfaceHandle as USBD_INTERFACE_HANDLE;
        *iHandle = InterfaceHandle;
        (*InterfaceInfo).Length = (Length as ULONG as u16);
    } else {
        if !InterfaceHandle.is_null() {
            if NumEndpoints != 0 {
                // DPRINT1("USBPORT_OpenInterface: USBDStatus - %lx\n", USBDStatus);
            }

            if IsAllocated != 0 {
                ExFreePoolWithTag(InterfaceHandle as PVOID, USB_PORT_TAG);
            }
        }
    }

    return USBDStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CloseConfiguration(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
) {

    const USB_PORT_TAG: u32 = 0x70627375; // 'USPb'

    let mut ConfigHandle: PUSBPORT_CONFIGURATION_HANDLE;
    let mut iHandleList: PLIST_ENTRY;
    let mut iHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut NumEndpoints: ULONG;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;

    ConfigHandle = (*DeviceHandle).ConfigHandle;

    if !ConfigHandle.is_null() {
        iHandleList = core::ptr::addr_of_mut!((*ConfigHandle).InterfaceHandleList);

        while !IsListEmpty(iHandleList) {
            iHandle = (((*iHandleList).Flink as *mut u8)
                .offset(-(core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, InterfaceLink) as isize)))
                as *mut USBPORT_INTERFACE_HANDLE;

            RemoveHeadList(iHandleList);

            NumEndpoints = (*iHandle).InterfaceDescriptor.bNumEndpoints as ULONG;

            PipeHandle = core::ptr::addr_of_mut!((*iHandle).PipeHandle[0]);

            while NumEndpoints > 0 {
                USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
                PipeHandle = PipeHandle.offset(1);
                NumEndpoints -= 1;
            }

            ExFreePoolWithTag(iHandle as *mut core::ffi::c_void, USB_PORT_TAG);
        }

        ExFreePoolWithTag(ConfigHandle as *mut core::ffi::c_void, USB_PORT_TAG);
        (*DeviceHandle).ConfigHandle = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InitInterfaceInfo(
    InterfaceInfo: PUSBD_INTERFACE_INFORMATION,
    ConfigHandle: PUSBPORT_CONFIGURATION_HANDLE,
) -> NTSTATUS {
    let Descriptor: PUSB_INTERFACE_DESCRIPTOR;
    let mut Pipe: PUSBD_PIPE_INFORMATION;
    let mut Length: SIZE_T;
    let mut PipeFlags: ULONG;
    let mut NumberOfPipes: ULONG;
    let mut USBDStatus: USBD_STATUS = USBD_STATUS_SUCCESS;

    Descriptor = USBPORT_ParseConfigurationDescriptor(
        (*ConfigHandle).ConfigurationDescriptor,
        (*InterfaceInfo).InterfaceNumber,
        (*InterfaceInfo).AlternateSetting,
        core::ptr::null_mut(),
    );

    Length = core::mem::size_of::<USBD_INTERFACE_INFORMATION>()
        + core::mem::size_of::<USBD_PIPE_INFORMATION>();

    if !Descriptor.is_null() {
        NumberOfPipes = (*Descriptor).bNumEndpoints as ULONG;

        Length = core::mem::offset_of!(USBD_INTERFACE_INFORMATION, Pipes)
            + NumberOfPipes as SIZE_T * core::mem::size_of::<USBD_PIPE_INFORMATION>();

        if (*InterfaceInfo).Length as SIZE_T >= Length {
            (*InterfaceInfo).Class = 0;
            (*InterfaceInfo).SubClass = 0;
            (*InterfaceInfo).Protocol = 0;
            (*InterfaceInfo).Reserved = 0;
            (*InterfaceInfo).InterfaceHandle = core::ptr::null_mut();
            (*InterfaceInfo).NumberOfPipes = NumberOfPipes;

            Pipe = (*InterfaceInfo).Pipes.as_mut_ptr();

            while NumberOfPipes > 0 {
                (*Pipe).EndpointAddress = 0;
                (*Pipe).Interval = 0;
                (*Pipe).PipeType = 0;
                (*Pipe).PipeHandle = core::ptr::null_mut();

                PipeFlags = (*Pipe).PipeFlags;

                if PipeFlags & !USBD_PF_VALID_MASK != 0 {
                    USBDStatus = USBD_STATUS_INVALID_PIPE_FLAGS;
                }

                if PipeFlags & USBD_PF_CHANGE_MAX_PACKET == 0 {
                    (*Pipe).MaximumPacketSize = 0;
                }

                Pipe = Pipe.offset(1);
                NumberOfPipes -= 1;
            }
        } else {
            USBDStatus = USBD_STATUS_BUFFER_TOO_SMALL;
        }
    } else {
        USBDStatus = USBD_STATUS_INTERFACE_NOT_FOUND;
    }

    (*InterfaceInfo).Length = (Length as ULONG as u16);
    return USBDStatus;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleSelectConfiguration(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    const USB_REQUEST_SET_CONFIGURATION: u8 = 0x09;
    const USBD_STATUS_SUCCESS: USBD_STATUS = 0;
    const USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR: USBD_STATUS = 0xC0000F00u32 as i32;
    const USBD_STATUS_INSUFFICIENT_RESOURCES: USBD_STATUS = 0xC0001000u32 as i32;
    const USBD_STATUS_SET_CONFIG_FAILED: USBD_STATUS = 0xC0002000u32 as i32;
    const USB_PORT_TAG: ULONG = 0x70627375;

    let ConfigDescriptor: PUSB_CONFIGURATION_DESCRIPTOR;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut ConfigHandle: PUSBPORT_CONFIGURATION_HANDLE = core::ptr::null_mut();
    let mut InterfaceInfo: PUSBD_INTERFACE_INFORMATION;
    let mut InterfaceHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut iNumber: ULONG;
    let mut ix: ULONG;
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut USBDStatus: USBD_STATUS = 0;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    ConfigDescriptor = (*Urb).__bindgen_anon_1.UrbSelectConfiguration.ConfigurationDescriptor;

    'exit: loop {
        if ConfigDescriptor.is_null() {
            RtlZeroMemory(
                core::ptr::addr_of_mut!(SetupPacket) as PVOID,
                (core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as ULONG as usize),
            );

            SetupPacket.bmRequestType.B = 0;
            SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
            SetupPacket.wValue.W = 0;
            SetupPacket.wIndex.W = 0;
            SetupPacket.wLength = 0;

            USBPORT_SendSetupPacket(
                DeviceHandle,
                FdoDevice,
                core::ptr::addr_of_mut!(SetupPacket),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
            break 'exit;
        }

        USBPORT_DumpingConfiguration(ConfigDescriptor);

        InterfaceInfo = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbSelectConfiguration.Interface);

        iNumber = 0;

        loop {
            iNumber += 1;
            InterfaceInfo = ((InterfaceInfo as ULONG_PTR) + (*InterfaceInfo).Length as ULONG_PTR)
                as PUSBD_INTERFACE_INFORMATION;

            if !((InterfaceInfo as ULONG_PTR)
                < (Urb as ULONG_PTR) + (*Urb).__bindgen_anon_1.UrbHeader.Length as ULONG_PTR)
            {
                break;
            }
        }

        if (iNumber <= 0) || (iNumber != (*ConfigDescriptor).bNumInterfaces as ULONG) {
            Status =
                USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR);
            break 'exit;
        }

        ConfigHandle = ExAllocatePoolWithTag(
            NonPagedPool,
            ((*ConfigDescriptor).wTotalLength as usize
                + core::mem::size_of::<USBPORT_CONFIGURATION_HANDLE>()),
            USB_PORT_TAG,
        ) as PUSBPORT_CONFIGURATION_HANDLE;

        if ConfigHandle.is_null() {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_INSUFFICIENT_RESOURCES);
            break 'exit;
        }

        RtlZeroMemory(
            ConfigHandle as PVOID,
            (((*ConfigDescriptor).wTotalLength as usize
                + core::mem::size_of::<USBPORT_CONFIGURATION_HANDLE>()) as ULONG as usize),
        );

        InitializeListHead(core::ptr::addr_of_mut!((*ConfigHandle).InterfaceHandleList));

        (*ConfigHandle).ConfigurationDescriptor =
            ConfigHandle.offset(1) as PUSB_CONFIGURATION_DESCRIPTOR;

        RtlCopyMemory(
            (*ConfigHandle).ConfigurationDescriptor as PVOID,
            ConfigDescriptor as PVOID,
            ((*ConfigDescriptor).wTotalLength as ULONG as usize),
        );

        RtlZeroMemory(
            core::ptr::addr_of_mut!(SetupPacket) as PVOID,
            (core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as ULONG as usize),
        );

        SetupPacket.bmRequestType.B = 0;
        SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
        SetupPacket.wValue.W = (*ConfigDescriptor).bConfigurationValue as u16;
        SetupPacket.wIndex.W = 0;
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

        if USBD_ERROR(USBDStatus) != 0 {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SET_CONFIG_FAILED);
            break 'exit;
        }

        if iNumber <= 0 {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
            break 'exit;
        }

        InterfaceInfo = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbSelectConfiguration.Interface);

        ix = 0;
        while ix < iNumber {
            USBDStatus = USBPORT_InitInterfaceInfo(InterfaceInfo, ConfigHandle);

            InterfaceHandle = core::ptr::null_mut();

            if USBD_SUCCESS(USBDStatus) {
                USBDStatus = USBPORT_OpenInterface(
                    Urb,
                    DeviceHandle,
                    FdoDevice,
                    ConfigHandle,
                    InterfaceInfo,
                    core::ptr::addr_of_mut!(InterfaceHandle),
                    1,
                );
            }

            if !InterfaceHandle.is_null() {
                InsertTailList(
                    core::ptr::addr_of_mut!((*ConfigHandle).InterfaceHandleList),
                    core::ptr::addr_of_mut!((*InterfaceHandle).InterfaceLink),
                );
            }

            if USBD_ERROR(USBDStatus) != 0 {
                break;
            }

            InterfaceInfo = ((InterfaceInfo as ULONG_PTR)
                + (*InterfaceInfo).Length as ULONG_PTR)
                as PUSBD_INTERFACE_INFORMATION;

            ix += 1;
        }

        if ix >= iNumber {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
        } else {
            Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
        }

        break 'exit;
    }

    if NT_SUCCESS(Status) {
        (*Urb).__bindgen_anon_1.UrbSelectConfiguration.ConfigurationHandle = ConfigHandle as PVOID;
        (*DeviceHandle).ConfigHandle = ConfigHandle;
    } else {
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AddDeviceHandle(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
) {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    InsertTailList(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleList),
        core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLink),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveDeviceHandle(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
) {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLink));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ValidateDeviceHandle(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
) -> BOOLEAN {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut HandleList: PLIST_ENTRY;
    let CurrentHandle: PUSBPORT_DEVICE_HANDLE;
    let mut Result: BOOLEAN = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    if !DeviceHandle.is_null() {
        HandleList = (*FdoExtension).DeviceHandleList.Flink;

        while HandleList != core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleList) {
            let CurrentHandle = (HandleList as *mut u8)
                .offset(-(core::mem::offset_of!(USBPORT_DEVICE_HANDLE, DeviceHandleLink) as isize))
                as PUSBPORT_DEVICE_HANDLE;

            if CurrentHandle == DeviceHandle {
                Result = 1;
                break;
            }

            HandleList = (*HandleList).Flink;
        }
    }
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).DeviceHandleSpinLock) as *mut usize),
        OldIrql,
    );

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DeviceHasTransfers(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
) -> BOOLEAN {

    let mut PipeHandleList: PLIST_ENTRY;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;

    // DPRINT("USBPORT_DeviceHasTransfers: ... \n");

    PipeHandleList = (*DeviceHandle).PipeHandleList.Flink;

    while PipeHandleList != core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList) {
        PipeHandle = (PipeHandleList as *mut u8)
            .offset(-(core::mem::offset_of!(USBPORT_PIPE_HANDLE, PipeLink) as isize))
            as *mut USBPORT_PIPE_HANDLE;

        PipeHandleList = (*PipeHandleList).Flink;

        if ((*PipeHandle).Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE) == 0
            && USBPORT_EndpointHasQueuedTransfers(
                FdoDevice,
                (*PipeHandle).Endpoint,
                core::ptr::null_mut(),
            ) != 0
        {
            return 1;
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetTt(
    FdoDevice: PDEVICE_OBJECT,
    HubDeviceHandle: PUSBPORT_DEVICE_HANDLE,
    OutPort: PUSHORT,
    OutHubDeviceHandle: *mut PUSBPORT_DEVICE_HANDLE,
) -> PUSB2_TT_EXTENSION {
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE = HubDeviceHandle;
    let mut TtCount: ULONG;
    let mut Entry: PLIST_ENTRY;
    let mut TtExtension: PUSB2_TT_EXTENSION = core::ptr::null_mut();

    // DPRINT("USBPORT_GetTt: HubDeviceHandle - %p\n", HubDeviceHandle);

    *OutHubDeviceHandle = core::ptr::null_mut();

    while (*DeviceHandle).DeviceSpeed != UsbHighSpeed {
        // DPRINT("USBPORT_GetTt: DeviceHandle - %p, DeviceHandle->PortNumber - %X\n",
        //        DeviceHandle, DeviceHandle->PortNumber);

        *OutPort = (*DeviceHandle).PortNumber;

        DeviceHandle = (*DeviceHandle).HubDeviceHandle;

        if DeviceHandle.is_null() {
            return core::ptr::null_mut();
        }
    }

    TtCount = (*DeviceHandle).TtCount;

    if TtCount == 0 {
        return core::ptr::null_mut();
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*DeviceHandle).TtList)) {
        return core::ptr::null_mut();
    }

    Entry = (*DeviceHandle).TtList.Flink;

    if TtCount > 1 {
        while Entry != core::ptr::addr_of_mut!((*DeviceHandle).TtList) {
            // ASSERT(Entry != NULL);

            TtExtension = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(USB2_TT_EXTENSION, Link) as isize))
                as *mut USB2_TT_EXTENSION;

            if (*TtExtension).TtNumber as USHORT == *OutPort {
                break;
            }

            Entry = (*Entry).Flink;

            TtExtension = core::ptr::null_mut();
        }
    } else {
        TtExtension = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(USB2_TT_EXTENSION, Link) as isize))
            as *mut USB2_TT_EXTENSION;
    }

    *OutHubDeviceHandle = DeviceHandle;

    return TtExtension;
}


/// EP0's max packet size from a device descriptor's first 8 bytes, or None if
/// it is not a legal value. USB 3 devices (bcdUSB >= 3.00, attached SuperSpeed)
/// give it as a power of two, and 9 (512 bytes) is the only legal one; USB 2
/// devices give bytes. The C accepts only the USB 2 form, which refused every
/// SuperSpeed device once usbxhci exposed its USB3 ports.
unsafe fn USBPORT_Ep0MaxPacketSize(Descriptor: *const USB_DEVICE_DESCRIPTOR) -> Option<USHORT> {
    let bcd = core::ptr::read_unaligned(core::ptr::addr_of!((*Descriptor).bcdUSB));
    let raw = (*Descriptor).bMaxPacketSize0;
    if bcd >= 0x0300 {
        return if raw == 9 { Some(512) } else { None };
    }
    match raw {
        8 | 16 | 32 | 64 => Some(raw as USHORT),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CreateDevice(
    pUsbdDeviceHandle: *mut PUSB_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
    HubDeviceHandle: PUSBPORT_DEVICE_HANDLE,
    PortStatus: USHORT,
    Port: USHORT,
) -> NTSTATUS {

    const USB_MINIPORT_FLAGS_USB2: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x200; verified vs usbport.h/usbmport.h
    const USB_PORT_STATUS_HIGH_SPEED: USHORT = 0x0400;
    const USB_PORT_STATUS_LOW_SPEED: USHORT = 0x0200;
    const USB_PORT_TAG: ULONG = 0x70627375; // 'Ubp '
    const PIPE_HANDLE_FLAG_CLOSED: ULONG = 0x00000001;
    const USB_ENDPOINT_DESCRIPTOR_TYPE: UCHAR = 0x05;
    const USB_DEFAULT_MAX_PACKET: USHORT = 64;
    const UsbLowSpeed: USB_DEVICE_SPEED = 0;
    const UsbFullSpeed: USB_DEVICE_SPEED = 1;
    const UsbHighSpeed: USB_DEVICE_SPEED = 2;
    const BMREQUEST_DEVICE_TO_HOST: UCHAR = 1;
    const BMREQUEST_HOST_TO_DEVICE: UCHAR = 0;
    const BMREQUEST_TO_OTHER: UCHAR = 3;
    const BMREQUEST_CLASS: UCHAR = 1;
    const USB_REQUEST_GET_DESCRIPTOR: UCHAR = 0x06;
    const USB_REQUEST_RESET_TT: UCHAR = 9;
    const USB_REQUEST_CLEAR_TT_BUFFER: UCHAR = 8;
    const USB_DEVICE_DESCRIPTOR_TYPE: UCHAR = 0x01;

    let mut TtDeviceHandle: PUSBPORT_DEVICE_HANDLE = core::ptr::null_mut();
    let mut TtExtension: PUSB2_TT_EXTENSION = core::ptr::null_mut();
    let mut port: USHORT;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let IsOpenedPipe: BOOL;
    let DeviceDescriptor: PVOID;
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();
    let mut TransferedLen: ULONG;
    let DescriptorMinSize: ULONG;
    let mut MaxPacketSize: UCHAR;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Status: NTSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    if USBPORT_ValidateDeviceHandle(FdoDevice, HubDeviceHandle) == 0 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
            (LOW_REALTIME_PRIORITY as i32),
            1,
            0,
        );

        return STATUS_DEVICE_NOT_CONNECTED;
    }

    port = Port;

    if ((*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2) != 0
        && (PortStatus & USB_PORT_STATUS_HIGH_SPEED) == 0
    {
        TtExtension = USBPORT_GetTt(
            FdoDevice,
            HubDeviceHandle,
            core::ptr::addr_of_mut!(port),
            core::ptr::addr_of_mut!(TtDeviceHandle),
        );
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    DeviceHandle = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USBPORT_DEVICE_HANDLE>(),
        USB_PORT_TAG,
    ) as PUSBPORT_DEVICE_HANDLE;

    if DeviceHandle.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        DeviceHandle as PVOID,
        core::mem::size_of::<USBPORT_DEVICE_HANDLE>(),
    );

    *pUsbdDeviceHandle = core::ptr::null_mut();

    (*DeviceHandle).TtExtension = TtExtension;
    (*DeviceHandle).PortNumber = Port;
    (*DeviceHandle).HubDeviceHandle = HubDeviceHandle;

    if (PortStatus & USB_PORT_STATUS_LOW_SPEED) != 0 {
        (*DeviceHandle).DeviceSpeed = UsbLowSpeed;
    } else if (PortStatus & USB_PORT_STATUS_HIGH_SPEED) != 0 {
        (*DeviceHandle).DeviceSpeed = UsbHighSpeed;
    } else {
        (*DeviceHandle).DeviceSpeed = UsbFullSpeed;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    PipeHandle = core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle);

    (*PipeHandle).Flags = PIPE_HANDLE_FLAG_CLOSED;

    (*PipeHandle).EndpointDescriptor.bLength =
        core::mem::size_of_val(&(*PipeHandle).EndpointDescriptor) as UCHAR;
    (*PipeHandle).EndpointDescriptor.bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;

    if (*DeviceHandle).DeviceSpeed == UsbLowSpeed {
        (*PipeHandle).EndpointDescriptor.wMaxPacketSize = 8;
    } else {
        (*PipeHandle).EndpointDescriptor.wMaxPacketSize = USB_DEFAULT_MAX_PACKET;
    }

    InitializeListHead(core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceHandle).TtList));

    Status = USBPORT_OpenPipe(FdoDevice, DeviceHandle, PipeHandle, core::ptr::null_mut());

    IsOpenedPipe = NT_SUCCESS(Status) as BOOL;

    if NT_ERROR(Status) {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
            (LOW_REALTIME_PRIORITY as i32),
            1,
            0,
        );

        ExFreePoolWithTag(DeviceHandle as PVOID, USB_PORT_TAG);

        return Status;
    }

    'error_exit: loop {
        DeviceDescriptor = ExAllocatePoolWithTag(
            NonPagedPool,
            USB_DEFAULT_MAX_PACKET as usize,
            USB_PORT_TAG,
        );

        if DeviceDescriptor.is_null() {
            break 'error_exit;
        }

        RtlZeroMemory(DeviceDescriptor, USB_DEFAULT_MAX_PACKET as usize);
        RtlZeroMemory(
            core::ptr::addr_of_mut!(SetupPacket) as PVOID,
            core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
        );

        SetupPacket.bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST);
        SetupPacket.bRequest = USB_REQUEST_GET_DESCRIPTOR;
        SetupPacket.wValue.HiByte = USB_DEVICE_DESCRIPTOR_TYPE;
        SetupPacket.wLength = USB_DEFAULT_MAX_PACKET;

        TransferedLen = 0;

        Status = USBPORT_SendSetupPacket(
            DeviceHandle,
            FdoDevice,
            core::ptr::addr_of_mut!(SetupPacket),
            DeviceDescriptor,
            USB_DEFAULT_MAX_PACKET as ULONG,
            core::ptr::addr_of_mut!(TransferedLen),
            core::ptr::null_mut(),
        );

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*DeviceHandle).DeviceDescriptor) as PVOID,
            DeviceDescriptor,
            core::mem::size_of::<USB_DEVICE_DESCRIPTOR>(),
        );

        ExFreePoolWithTag(DeviceDescriptor, USB_PORT_TAG);

        DescriptorMinSize = (core::mem::offset_of!(USB_DEVICE_DESCRIPTOR, bMaxPacketSize0)
            + core::mem::size_of::<UCHAR>()) as ULONG;

        if (TransferedLen == DescriptorMinSize) && !NT_SUCCESS(Status) {
            Status = STATUS_SUCCESS;
        }

        if NT_SUCCESS(Status) && (TransferedLen >= DescriptorMinSize) {
            if ((*DeviceHandle).DeviceDescriptor.bLength as usize
                >= core::mem::size_of::<USB_DEVICE_DESCRIPTOR>())
                && ((*DeviceHandle).DeviceDescriptor.bDescriptorType == USB_DEVICE_DESCRIPTOR_TYPE)
            {
                MaxPacketSize = (*DeviceHandle).DeviceDescriptor.bMaxPacketSize0;

                if USBPORT_Ep0MaxPacketSize(core::ptr::addr_of!((*DeviceHandle).DeviceDescriptor)).is_some() {
                    USBPORT_AddDeviceHandle(FdoDevice, DeviceHandle);

                    *pUsbdDeviceHandle = (DeviceHandle as *mut c_void);

                    KeReleaseSemaphore(
                        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
                        (LOW_REALTIME_PRIORITY as i32),
                        1,
                        0,
                    );

                    return Status;
                }
            }
        }

        break 'error_exit;
    }

    // ErrorExit:

    if !TtExtension.is_null() && !TtDeviceHandle.is_null() {
        SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_OTHER);
        SetupPacket.bmRequestType.set_Reserved(0);
        SetupPacket.bmRequestType.set_Type(BMREQUEST_CLASS);
        SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);

        if TtDeviceHandle == HubDeviceHandle {
            SetupPacket.bRequest = USB_REQUEST_RESET_TT;
        } else {
            SetupPacket.bRequest = USB_REQUEST_CLEAR_TT_BUFFER;
        }

        SetupPacket.wValue.LowByte = 0;
        SetupPacket.wValue.HiByte = 0;
        SetupPacket.wIndex.W = port;
        SetupPacket.wLength = 0;

        USBPORT_SendSetupPacket(
            TtDeviceHandle,
            FdoDevice,
            core::ptr::addr_of_mut!(SetupPacket),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    Status = STATUS_DEVICE_DATA_ERROR;

    if IsOpenedPipe != 0 {
        USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    ExFreePoolWithTag(DeviceHandle as PVOID, USB_PORT_TAG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AllocateUsbAddress(FdoDevice: PDEVICE_OBJECT) -> ULONG {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut BitMapIdx: ULONG;
    let mut BitNumber: ULONG;
    let mut ix: ULONG;

    // DPRINT("USBPORT_AllocateUsbAddress \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    ix = 0;
    while ix < 4 {
        BitMapIdx = 1;

        BitNumber = 0;
        while BitNumber < 32 {
            if ((*FdoExtension).UsbAddressBitMap[ix as usize] & BitMapIdx) == 0 {
                (*FdoExtension).UsbAddressBitMap[ix as usize] |= BitMapIdx;
                return 32 * ix + BitNumber;
            }

            BitMapIdx <<= 2;
            BitNumber += 1;
        }
        ix += 1;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FreeUsbAddress(
    FdoDevice: PDEVICE_OBJECT,
    DeviceAddress: USHORT,
) {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut ix: ULONG;
    let mut BitMapIdx: ULONG;
    let mut BitNumber: ULONG;
    let mut CurrentAddress: USHORT;

    // DPRINT("USBPORT_FreeUsbAddress: DeviceAddress - %x\n", DeviceAddress);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    ix = 0;
    while ix < 4 {
        BitMapIdx = 1;
        CurrentAddress = (32 * ix) as USHORT;

        BitNumber = 0;
        while BitNumber < 32 {
            if CurrentAddress == DeviceAddress {
                (*FdoExtension).UsbAddressBitMap[ix as usize] &= !BitMapIdx;
                return;
            }

            BitMapIdx <<= 2;
            CurrentAddress = CurrentAddress.wrapping_add(1);
            BitNumber += 1;
        }

        ix += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InitializeDevice(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
) -> NTSTATUS {

    const USB_DEFAULT_DEVICE_ADDRESS: USHORT = 0;
    const USB_REQUEST_SET_ADDRESS: UCHAR = 0x05;
    const USB_REQUEST_GET_DESCRIPTOR: UCHAR = 0x06;
    const USB_DEVICE_DESCRIPTOR_TYPE: UCHAR = 0x01;
    const USB_DEVICE_CLASS_HUB: UCHAR = 0x09;
    const DEVICE_HANDLE_FLAG_USB2HUB: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const UsbHighSpeed: i32 = 2; // UsbHighSpeed

    let Endpoint: PUSBPORT_ENDPOINT;
    let mut CtrlSetup: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();
    let mut TransferedLen: ULONG = 0;
    let mut DeviceAddress: USHORT = 0;
    let mut MaxPacketSize: UCHAR;
    let mut Status: NTSTATUS;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    DeviceAddress = USBPORT_AllocateUsbAddress(FdoDevice) as USHORT;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(CtrlSetup) as PVOID,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
    );

    CtrlSetup.bRequest = USB_REQUEST_SET_ADDRESS;
    CtrlSetup.wValue.W = DeviceAddress;

    Status = USBPORT_SendSetupPacket(
        DeviceHandle,
        FdoDevice,
        core::ptr::addr_of_mut!(CtrlSetup),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    'exit_error: loop {
        if !NT_SUCCESS(Status) {
            break 'exit_error;
        }

        (*DeviceHandle).DeviceAddress = DeviceAddress;
        Endpoint = (*DeviceHandle).PipeHandle.Endpoint;

        (*Endpoint).EndpointProperties.TotalMaxPacketSize =
            USBPORT_Ep0MaxPacketSize(core::ptr::addr_of!((*DeviceHandle).DeviceDescriptor))
                .unwrap_or((*DeviceHandle).DeviceDescriptor.bMaxPacketSize0 as USHORT);
        (*Endpoint).EndpointProperties.DeviceAddress = DeviceAddress;

        Status = USBPORT_ReopenPipe(FdoDevice, Endpoint);

        if !NT_SUCCESS(Status) {
            break 'exit_error;
        }

        USBPORT_Wait((FdoDevice) as _, 10);

        RtlZeroMemory(
            core::ptr::addr_of_mut!(CtrlSetup) as PVOID,
            core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
        );

        CtrlSetup.bRequest = USB_REQUEST_GET_DESCRIPTOR;
        CtrlSetup.wValue.HiByte = USB_DEVICE_DESCRIPTOR_TYPE;
        CtrlSetup.wLength = core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as USHORT;
        CtrlSetup.bmRequestType.B = 0x80;

        Status = USBPORT_SendSetupPacket(
            DeviceHandle,
            FdoDevice,
            core::ptr::addr_of_mut!(CtrlSetup),
            core::ptr::addr_of_mut!((*DeviceHandle).DeviceDescriptor) as PVOID,
            core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as ULONG,
            core::ptr::addr_of_mut!(TransferedLen),
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            if (TransferedLen as usize) < core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() {
            }

            MaxPacketSize = (*DeviceHandle).DeviceDescriptor.bMaxPacketSize0;

            if USBPORT_Ep0MaxPacketSize(core::ptr::addr_of!((*DeviceHandle).DeviceDescriptor)).is_none() {
                Status = STATUS_DEVICE_DATA_ERROR;
                break 'exit_error;
            }

            if (*DeviceHandle).DeviceSpeed == (UsbHighSpeed as u32)
                && (*DeviceHandle).DeviceDescriptor.bDeviceClass == USB_DEVICE_CLASS_HUB
            {
                (*DeviceHandle).Flags |= DEVICE_HANDLE_FLAG_USB2HUB;
            }
        } else {
            break 'exit_error;
        }

        break 'exit_error;
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetUsbDescriptor(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
    Type: UCHAR,
    ConfigDesc: PUCHAR,
    ConfigDescSize: PULONG,
) -> NTSTATUS {
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();

    RtlZeroMemory(
        core::ptr::addr_of_mut!(SetupPacket) as PVOID,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as SIZE_T,
    );

    SetupPacket.bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST as u8);
    SetupPacket.bRequest = USB_REQUEST_GET_DESCRIPTOR as u8;
    SetupPacket.wValue.HiByte = Type;
    SetupPacket.wLength = *ConfigDescSize as USHORT;

    return USBPORT_SendSetupPacket(
        DeviceHandle,
        FdoDevice,
        core::ptr::addr_of_mut!(SetupPacket),
        ConfigDesc as PVOID,
        *ConfigDescSize,
        ConfigDescSize,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetInterfaceHandle(
    ConfigurationHandle: PUSBPORT_CONFIGURATION_HANDLE,
    InterfaceNumber: UCHAR,
) -> PUSBPORT_INTERFACE_HANDLE {
    let mut InterfaceHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut iHandleList: PLIST_ENTRY;
    let mut InterfaceNum: UCHAR;

    iHandleList = (*ConfigurationHandle).InterfaceHandleList.Flink;

    while !iHandleList.is_null()
        && iHandleList != core::ptr::addr_of_mut!((*ConfigurationHandle).InterfaceHandleList)
    {
        InterfaceHandle = (iHandleList as *mut u8)
            .offset(-(core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, InterfaceLink) as isize))
            as *mut USBPORT_INTERFACE_HANDLE;

        InterfaceNum = (*InterfaceHandle).InterfaceDescriptor.bInterfaceNumber;

        if InterfaceNum == InterfaceNumber {
            return InterfaceHandle;
        }

        iHandleList = (*InterfaceHandle).InterfaceLink.Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_HandleSelectInterface(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
    Urb: PURB,
) -> NTSTATUS {

    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let ConfigurationHandle: PUSBPORT_CONFIGURATION_HANDLE;
    let Interface: PUSBD_INTERFACE_INFORMATION;
    let InterfaceHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut iHandle: PUSBPORT_INTERFACE_HANDLE;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut USBDStatus: USBD_STATUS;
    let Length: USHORT;
    let mut ix: ULONG;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    ConfigurationHandle = (*Urb).__bindgen_anon_1.UrbSelectInterface.ConfigurationHandle as PUSBPORT_CONFIGURATION_HANDLE;

    Interface = core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbSelectInterface.Interface);

    Length = (*Interface).Length + core::mem::size_of::<USBD_PIPE_INFORMATION>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbHeader.Length = Length;

    USBDStatus = USBPORT_InitInterfaceInfo(Interface, ConfigurationHandle);

    if USBDStatus != 0 {
        (*Interface).InterfaceHandle = (-1isize) as USBD_INTERFACE_HANDLE;
        return USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
    }

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;

    InterfaceHandle = USBPORT_GetInterfaceHandle(ConfigurationHandle, (*Interface).InterfaceNumber);

    if !InterfaceHandle.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*InterfaceHandle).InterfaceLink));

        if (*InterfaceHandle).InterfaceDescriptor.bNumEndpoints != 0 {
            PipeHandle = core::ptr::addr_of_mut!((*InterfaceHandle).PipeHandle[0]);

            ix = 0;
            while ix < (*InterfaceHandle).InterfaceDescriptor.bNumEndpoints as ULONG {
                USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
                PipeHandle = PipeHandle.add(1);
                ix += 1;
            }
        }
    }

    iHandle = core::ptr::null_mut();

    USBDStatus = USBPORT_OpenInterface(
        Urb,
        DeviceHandle,
        FdoDevice,
        ConfigurationHandle,
        Interface,
        core::ptr::addr_of_mut!(iHandle),
        1,
    );

    if USBDStatus != 0 {
        (*Interface).InterfaceHandle = (-1isize) as USBD_INTERFACE_HANDLE;
    } else {
        if !InterfaceHandle.is_null() {
            ExFreePoolWithTag(InterfaceHandle as *mut core::ffi::c_void, USB_PORT_TAG);
        }

        (*Interface).InterfaceHandle = iHandle as USBD_INTERFACE_HANDLE;

        InsertTailList(
            core::ptr::addr_of_mut!((*ConfigurationHandle).InterfaceHandleList),
            core::ptr::addr_of_mut!((*iHandle).InterfaceLink),
        );
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    return USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveDevice(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    Flags: ULONG,
) -> NTSTATUS {

    const USBD_KEEP_DEVICE_DATA: ULONG = 0x00000001;
    const USBD_MARK_DEVICE_BUSY: ULONG = 0x00000002;
    const DEVICE_HANDLE_FLAG_REMOVED: ULONG = 0x00000008; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const DEVICE_HANDLE_FLAG_ROOTHUB: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x10; verified vs usbport.h/usbmport.h
    const USB2_TT_EXTENSION_FLAG_DELETED: ULONG = 0x00000001;
    const USB2_FRAMES: ULONG = 32;
    const USB_PORT_TAG: ULONG = 0x70627375;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut TtExtension: PUSB2_TT_EXTENSION;
    let mut ix: ULONG;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    if (Flags & USBD_KEEP_DEVICE_DATA) != 0 || (Flags & USBD_MARK_DEVICE_BUSY) != 0 {
        return STATUS_SUCCESS;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as *mut _,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    if USBPORT_ValidateDeviceHandle(FdoDevice, DeviceHandle) == 0 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as *mut _,
            (LOW_REALTIME_PRIORITY as i32),
            1,
            0,
        );

        return STATUS_DEVICE_NOT_CONNECTED;
    }

    USBPORT_RemoveDeviceHandle(FdoDevice, DeviceHandle);

    (*DeviceHandle).Flags |= DEVICE_HANDLE_FLAG_REMOVED;

    USBPORT_AbortTransfers(FdoDevice, DeviceHandle);

    while InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock)) >= 0 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));
        USBPORT_Wait((FdoDevice) as _, 100);
    }

    if !(*DeviceHandle).ConfigHandle.is_null() {
        USBPORT_CloseConfiguration(DeviceHandle, FdoDevice);
    }

    USBPORT_ClosePipe(
        DeviceHandle,
        FdoDevice,
        core::ptr::addr_of_mut!((*DeviceHandle).PipeHandle),
    );

    if (*DeviceHandle).DeviceAddress != 0 {
        USBPORT_FreeUsbAddress(FdoDevice, (*DeviceHandle).DeviceAddress);
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*DeviceHandle).TtList)) {
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*DeviceHandle).TtList)) {
        TtExtension = ((*DeviceHandle).TtList.Flink as *mut u8)
            .sub(core::mem::offset_of!(USB2_TT_EXTENSION, Link))
            as PUSB2_TT_EXTENSION;

        RemoveHeadList(core::ptr::addr_of_mut!((*DeviceHandle).TtList));

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        (*TtExtension).Flags |= USB2_TT_EXTENSION_FLAG_DELETED;

        if IsListEmpty(core::ptr::addr_of_mut!((*TtExtension).EndpointList)) {
            USBPORT_UpdateAllocatedBwTt(TtExtension);

            ix = 0;
            while ix < USB2_FRAMES {
                (*FdoExtension).Bandwidth[ix as usize] += (*TtExtension).MaxBandwidth;
                ix += 1;
            }

            ExFreePoolWithTag(TtExtension as *mut _, USB_PORT_TAG);
        }

        KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize), OldIrql);
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as *mut _,
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    if ((*DeviceHandle).Flags & DEVICE_HANDLE_FLAG_ROOTHUB) == 0 {
        ExFreePoolWithTag(DeviceHandle as *mut _, USB_PORT_TAG);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RestoreDevice(
    FdoDevice: PDEVICE_OBJECT,
    OldDeviceHandle: PUSBPORT_DEVICE_HANDLE,
    NewDeviceHandle: PUSBPORT_DEVICE_HANDLE,
) -> NTSTATUS {

    const BMREQUEST_HOST_TO_DEVICE: UCHAR = 0;
    const BMREQUEST_STANDARD: UCHAR = 0;
    const BMREQUEST_TO_INTERFACE: UCHAR = 1;
    const USB_REQUEST_SET_CONFIGURATION: UCHAR = 0x09;
    const USB_REQUEST_SET_INTERFACE: UCHAR = 0x0B;
    const DEVICE_HANDLE_FLAG_USB2HUB: ULONG = 0x00000010; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h
    const PIPE_HANDLE_FLAG_NULL_PACKET_SIZE: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const ENDPOINT_FLAG_NUKE: ULONG = 0x00000008; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
    const ENDPOINT_FLAG_ABORTING: ULONG = 0x00000020; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const USBPORT_ENDPOINT_RUN: ULONG = 0x00000000; // FIXED 2026-07-05: translation invented 0x3; verified vs usbport.h/usbmport.h
    const USBPORT_ENDPOINT_ACTIVE: ULONG = 0x00000003; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
    const USB_PORT_TAG: ULONG = 0x70627375;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut iHandleList: PLIST_ENTRY;
    let Endpoint: PUSBPORT_ENDPOINT;
    let mut EndpointRequirements: USBPORT_ENDPOINT_REQUIREMENTS = core::mem::zeroed();
    let mut SetupPacket: USB_DEFAULT_PIPE_SETUP_PACKET = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut USBDStatus: USBD_STATUS = 0;
    let mut OldIrql: KIRQL = 0;
    let mut InterfaceHandle: PUSBPORT_INTERFACE_HANDLE;
    let PipeHandle: PUSBPORT_PIPE_HANDLE;
    let Packet: PUSBPORT_REGISTRATION_PACKET;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore) as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    if USBPORT_ValidateDeviceHandle(FdoDevice, OldDeviceHandle) == 0 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
            (LOW_REALTIME_PRIORITY as i32),
            1,
            0,
        );

        return STATUS_DEVICE_NOT_CONNECTED;
    }

    if USBPORT_ValidateDeviceHandle(FdoDevice, NewDeviceHandle) == 0 {
        KeReleaseSemaphore(
            core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
            (LOW_REALTIME_PRIORITY as i32),
            1,
            0,
        );

        return STATUS_DEVICE_NOT_CONNECTED;
    }

    USBPORT_RemoveDeviceHandle(FdoDevice, OldDeviceHandle);
    USBPORT_AbortTransfers(FdoDevice, OldDeviceHandle);

    while InterlockedDecrement(core::ptr::addr_of_mut!((*OldDeviceHandle).DeviceHandleLock)) >= 0 {
        InterlockedIncrement(core::ptr::addr_of_mut!((*OldDeviceHandle).DeviceHandleLock));
        USBPORT_Wait((FdoDevice) as _, 100);
    }

    let _devdesc_cmp = RtlCompareMemory(
        core::ptr::addr_of!((*NewDeviceHandle).DeviceDescriptor) as *const core::ffi::c_void,
        core::ptr::addr_of!((*OldDeviceHandle).DeviceDescriptor) as *const core::ffi::c_void,
        core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() as _,
    );
    if core::mem::size_of::<USB_DEVICE_DESCRIPTOR>() == _devdesc_cmp as usize
    {
        (*NewDeviceHandle).ConfigHandle = (*OldDeviceHandle).ConfigHandle;

        if !(*OldDeviceHandle).ConfigHandle.is_null() {
            RtlZeroMemory(
                core::ptr::addr_of_mut!(SetupPacket) as PVOID,
                core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as SIZE_T,
            );

            SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
            SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
            SetupPacket.wValue.W =
                (*(*(*OldDeviceHandle).ConfigHandle).ConfigurationDescriptor).bConfigurationValue
                    as USHORT;
            SetupPacket.wIndex.W = 0;
            SetupPacket.wLength = 0;

            USBPORT_SendSetupPacket(
                NewDeviceHandle,
                FdoDevice,
                core::ptr::addr_of_mut!(SetupPacket),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(USBDStatus),
            );

            if USBD_ERROR(USBDStatus) != 0 {
                Status = USBPORT_USBDStatusToNtStatus(core::ptr::null_mut(), USBDStatus);
            }

            if NT_SUCCESS(Status) {
                iHandleList = (*(*NewDeviceHandle).ConfigHandle).InterfaceHandleList.Flink;

                while !iHandleList.is_null()
                    && iHandleList
                        != core::ptr::addr_of_mut!(
                            (*(*NewDeviceHandle).ConfigHandle).InterfaceHandleList
                        )
                {
                    InterfaceHandle = (iHandleList as *mut u8).offset(
                        -(core::mem::offset_of!(USBPORT_INTERFACE_HANDLE, InterfaceLink) as isize),
                    ) as PUSBPORT_INTERFACE_HANDLE;

                    if (*InterfaceHandle).AlternateSetting != 0 {
                        RtlZeroMemory(
                            core::ptr::addr_of_mut!(SetupPacket) as PVOID,
                            core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as SIZE_T,
                        );

                        SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
                        SetupPacket.bmRequestType.set_Type(BMREQUEST_STANDARD);
                        SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);

                        SetupPacket.bRequest = USB_REQUEST_SET_INTERFACE;
                        SetupPacket.wValue.W =
                            (*InterfaceHandle).InterfaceDescriptor.bAlternateSetting as USHORT;
                        SetupPacket.wIndex.W =
                            (*InterfaceHandle).InterfaceDescriptor.bInterfaceNumber as USHORT;
                        SetupPacket.wLength = 0;

                        USBPORT_SendSetupPacket(
                            NewDeviceHandle,
                            FdoDevice,
                            core::ptr::addr_of_mut!(SetupPacket),
                            core::ptr::null_mut(),
                            0,
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(USBDStatus),
                        );
                    }

                    iHandleList = (*iHandleList).Flink;
                }
            }
        }

        if (*NewDeviceHandle).Flags & DEVICE_HANDLE_FLAG_USB2HUB != 0 {
            (*NewDeviceHandle).TtCount = (*OldDeviceHandle).TtCount;
        }

        while !IsListEmpty(core::ptr::addr_of_mut!((*OldDeviceHandle).PipeHandleList)) {
            let PipeHandle = ((*OldDeviceHandle).PipeHandleList.Flink as *mut u8)
                .offset(-(core::mem::offset_of!(USBPORT_PIPE_HANDLE, PipeLink) as isize))
                as PUSBPORT_PIPE_HANDLE;

            USBPORT_RemovePipeHandle(OldDeviceHandle, PipeHandle);

            if PipeHandle != core::ptr::addr_of_mut!((*OldDeviceHandle).PipeHandle) {
                USBPORT_AddPipeHandle(NewDeviceHandle, PipeHandle);

                if (*PipeHandle).Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE == 0 {
                    let Endpoint = (*PipeHandle).Endpoint;
                    (*Endpoint).DeviceHandle = NewDeviceHandle;
                    (*Endpoint).EndpointProperties.DeviceAddress =
                        (*NewDeviceHandle).DeviceAddress;

                    let Packet = core::ptr::addr_of_mut!(
                        (*(*FdoExtension).MiniPortInterface).Packet
                    );

                    if (*Endpoint).Flags & ENDPOINT_FLAG_NUKE == 0 {
                        KeAcquireSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                            core::ptr::addr_of_mut!(OldIrql),
                        );

                        ((*Packet).ReopenEndpoint.unwrap())(
                            (*FdoExtension).MiniPortExt,
                            core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
                            Endpoint.offset(1) as PVOID,
                        );

                        ((*Packet).SetEndpointDataToggle.unwrap())(
                            (*FdoExtension).MiniPortExt,
                            Endpoint.offset(1) as PVOID,
                            0,
                        );

                        ((*Packet).SetEndpointStatus.unwrap())(
                            (*FdoExtension).MiniPortExt,
                            Endpoint.offset(1) as PVOID,
                            USBPORT_ENDPOINT_RUN,
                        );

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                            OldIrql,
                        );
                    } else {
                        MiniportCloseEndpoint(FdoDevice, Endpoint);

                        RtlZeroMemory(
                            Endpoint.offset(1) as PVOID,
                            (*Packet).MiniPortEndpointSize as SIZE_T,
                        );

                        RtlZeroMemory(
                            (*Endpoint).EndpointProperties.BufferVA as PVOID,
                            (*Endpoint).EndpointProperties.BufferLength as SIZE_T,
                        );

                        KeAcquireSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                            core::ptr::addr_of_mut!(OldIrql),
                        );

                        ((*Packet).QueryEndpointRequirements.unwrap())(
                            (*FdoExtension).MiniPortExt,
                            core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
                            core::ptr::addr_of_mut!(EndpointRequirements),
                        );

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                            OldIrql,
                        );

                        MiniportOpenEndpoint(FdoDevice, Endpoint);

                        (*Endpoint).Flags &= !(ENDPOINT_FLAG_NUKE | ENDPOINT_FLAG_ABORTING);

                        KeAcquireSpinLock(
                            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
                        );

                        if (*Endpoint).StateLast == USBPORT_ENDPOINT_ACTIVE {
                            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!(
                                (*FdoExtension).MiniportSpinLock
                            ) as *mut usize));

                            ((*Packet).SetEndpointState.unwrap())(
                                (*FdoExtension).MiniPortExt,
                                Endpoint.offset(1) as PVOID,
                                USBPORT_ENDPOINT_ACTIVE,
                            );

                            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                                (*FdoExtension).MiniportSpinLock
                            ) as *mut usize));
                        }

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                            (*Endpoint).EndpointOldIrql,
                        );
                    }
                }
            }
        }

        USBPORT_AddPipeHandle(
            OldDeviceHandle,
            core::ptr::addr_of_mut!((*OldDeviceHandle).PipeHandle),
        );
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    USBPORT_ClosePipe(
        OldDeviceHandle,
        FdoDevice,
        core::ptr::addr_of_mut!((*OldDeviceHandle).PipeHandle),
    );

    if (*OldDeviceHandle).DeviceAddress != 0 {
        USBPORT_FreeUsbAddress(FdoDevice, (*OldDeviceHandle).DeviceAddress);
    }

    KeReleaseSemaphore(
        core::ptr::addr_of_mut!((*FdoExtension).DeviceSemaphore),
        (LOW_REALTIME_PRIORITY as i32),
        1,
        0,
    );

    ExFreePoolWithTag(OldDeviceHandle as PVOID, USB_PORT_TAG);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InitializeTT(
    FdoDevice: PDEVICE_OBJECT,
    HubDeviceHandle: PUSBPORT_DEVICE_HANDLE,
    TtNumber: ULONG,
) -> NTSTATUS {
    const USB_PORT_TAG: u32 = 0x70627375; // 'Port'
    const TOTAL_USB11_BUS_BANDWIDTH: ULONG = 12000;
    const USB2_FRAMES: ULONG = 0x00000020; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let TtExtension: PUSB2_TT_EXTENSION;
    let mut ix: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    TtExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USB2_TT_EXTENSION>(),
        USB_PORT_TAG,
    ) as PUSB2_TT_EXTENSION;

    if TtExtension.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(
        TtExtension as *mut core::ffi::c_void,
        core::mem::size_of::<USB2_TT_EXTENSION>(),
    );

    (*TtExtension).DeviceAddress = (*HubDeviceHandle).DeviceAddress;
    (*TtExtension).TtNumber = (TtNumber as u16);
    (*TtExtension).RootHubPdo = (*FdoExtension).RootHubPdo;
    (*TtExtension).BusBandwidth = TOTAL_USB11_BUS_BANDWIDTH;

    InitializeListHead(core::ptr::addr_of_mut!((*TtExtension).EndpointList));

    ix = 0;
    while ix < USB2_FRAMES {
        (*TtExtension).Bandwidth[ix as usize] =
            (*TtExtension).BusBandwidth - (*TtExtension).BusBandwidth / 10;
        ix += 1;
    }

    USBPORT_UpdateAllocatedBwTt(TtExtension);

    ix = 0;
    while ix < USB2_FRAMES {
        (*FdoExtension).Bandwidth[ix as usize] -= (*TtExtension).MaxBandwidth;
        ix += 1;
    }

    USB2_InitTT(
        (*FdoExtension).Usb2Extension,
        core::ptr::addr_of_mut!((*TtExtension).Tt),
    );

    InsertTailList(
        core::ptr::addr_of_mut!((*HubDeviceHandle).TtList),
        core::ptr::addr_of_mut!((*TtExtension).Link),
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_Initialize20Hub(
    FdoDevice: PDEVICE_OBJECT,
    HubDeviceHandle: PUSBPORT_DEVICE_HANDLE,
    TtCount: ULONG,
) -> NTSTATUS {
    const DEVICE_HANDLE_FLAG_ROOTHUB: ULONG = 0x00000002;

    let mut Status: NTSTATUS = 0;
    let mut ix: ULONG;

    if HubDeviceHandle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*HubDeviceHandle).Flags & DEVICE_HANDLE_FLAG_ROOTHUB != 0 {
        return STATUS_SUCCESS;
    }

    if TtCount == 0 {
        (*HubDeviceHandle).TtCount = 0;
        return STATUS_SUCCESS;
    }

    ix = 0;
    while ix < TtCount {
        Status = USBPORT_InitializeTT(FdoDevice, HubDeviceHandle, ix + 1);

        if !NT_SUCCESS(Status) {
            break;
        }

        ix += 1;
    }

    (*HubDeviceHandle).TtCount = TtCount;

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort device functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// USBPORT_SendSetupPacket(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                         IN PDEVICE_OBJECT FdoDevice,
//                         IN PUSB_DEFAULT_PIPE_SETUP_PACKET SetupPacket,
//                         IN PVOID Buffer,
//                         IN ULONG Length,
//                         IN OUT PULONG TransferedLen,
//                         IN OUT PUSBD_STATUS pUSBDStatus)
// {
//     PURB Urb;
//     PMDL Mdl;
//     USBD_STATUS USBDStatus;
//     KEVENT Event;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_SendSetupPacket: DeviceHandle - %p, FdoDevice - %p, SetupPacket - %p, Buffer - %p, Length - %x, TransferedLen - %x, pUSBDStatus - %x\n",
//            DeviceHandle,
//            FdoDevice,
//            SetupPacket,
//            Buffer,
//            Length,
//            TransferedLen,
//            pUSBDStatus);
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Urb = ExAllocatePoolWithTag(NonPagedPool,
//                                 sizeof(struct _URB_CONTROL_TRANSFER),
//                                 USB_PORT_TAG);
//
//     if (Urb)
//     {
//         InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//
//         RtlZeroMemory(Urb, sizeof(struct _URB_CONTROL_TRANSFER));
//
//         RtlCopyMemory(Urb->UrbControlTransfer.SetupPacket,
//                       SetupPacket,
//                       sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//         Urb->UrbHeader.Length = sizeof(struct _URB_CONTROL_TRANSFER);
//         Urb->UrbHeader.Function = URB_FUNCTION_CONTROL_TRANSFER;
//         Urb->UrbHeader.UsbdDeviceHandle = DeviceHandle;
//         Urb->UrbHeader.UsbdFlags = 0;
//
//         Urb->UrbControlTransfer.PipeHandle = &DeviceHandle->PipeHandle;
//         Urb->UrbControlTransfer.TransferBufferLength = Length;
//         Urb->UrbControlTransfer.TransferBuffer = Buffer;
//         Urb->UrbControlTransfer.TransferBufferMDL = NULL;
//
//         Urb->UrbControlTransfer.TransferFlags = USBD_SHORT_TRANSFER_OK |
//                                                 USBD_TRANSFER_DIRECTION;
//
//         if (SetupPacket->bmRequestType.Dir != BMREQUEST_DEVICE_TO_HOST)
//         {
//             Urb->UrbControlTransfer.TransferFlags &= ~USBD_TRANSFER_DIRECTION_IN;
//         }
//
//         Status = STATUS_SUCCESS;
//
//         if (Length)
//         {
//             Mdl = IoAllocateMdl(Buffer, Length, FALSE, FALSE, NULL);
//
//             Urb->UrbControlTransfer.TransferBufferMDL = Mdl;
//
//             if (Mdl)
//             {
//                 Urb->UrbHeader.UsbdFlags |= USBD_FLAG_ALLOCATED_MDL;
//                 MmBuildMdlForNonPagedPool(Mdl);
//             }
//             else
//             {
//                 Status = USBPORT_USBDStatusToNtStatus(NULL,
//                                                       USBD_STATUS_INSUFFICIENT_RESOURCES);
//             }
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             USBDStatus = USBPORT_AllocateTransfer(FdoDevice,
//                                                   Urb,
//                                                   NULL,
//                                                   NULL,
//                                                   &Event);
//
//             if (USBD_SUCCESS(USBDStatus))
//             {
//                 InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//
//                 USBPORT_QueueTransferUrb(Urb);
//
//                 KeWaitForSingleObject(&Event,
//                                       Suspended,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//
//                 USBDStatus = Urb->UrbHeader.Status;
//             }
//
//             Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//
//             if (TransferedLen)
//                 *TransferedLen = Urb->UrbControlTransfer.TransferBufferLength;
//
//             if (pUSBDStatus)
//                 *pUSBDStatus = USBDStatus;
//         }
//
//         InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//         ExFreePoolWithTag(Urb, USB_PORT_TAG);
//     }
//     else
//     {
//         if (pUSBDStatus)
//             *pUSBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
//
//         Status = USBPORT_USBDStatusToNtStatus(NULL,
//                                               USBD_STATUS_INSUFFICIENT_RESOURCES);
//     }
//
//     DPRINT("USBPORT_SendSetupPacket: Status - %x\n", Status);
//     return Status;
// }
//
// ULONG
// NTAPI
// USBPORT_GetInterfaceLength(IN PUSB_INTERFACE_DESCRIPTOR iDescriptor,
//                            IN ULONG_PTR EndDescriptors)
// {
//     SIZE_T Length;
//     PUSB_ENDPOINT_DESCRIPTOR Descriptor;
//     ULONG ix;
//
//     DPRINT("USBPORT_GetInterfaceLength ... \n");
//
//     Length = iDescriptor->bLength;
//     Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)iDescriptor + Length);
//
//     if (iDescriptor->bNumEndpoints)
//     {
//         for (ix = 0; ix < iDescriptor->bNumEndpoints; ix++)
//         {
//             while ((Descriptor->bDescriptorType != USB_ENDPOINT_DESCRIPTOR_TYPE) &&
//                    (Descriptor->bLength > 0))
//             {
//                 Length += Descriptor->bLength;
//                 Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)Descriptor +
//                                                         Descriptor->bLength);
//             }
//
//             Length += Descriptor->bLength;
//             Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)Descriptor +
//                                                     Descriptor->bLength);
//         }
//     }
//
//     while (((ULONG_PTR)Descriptor < EndDescriptors) &&
//            (Descriptor->bDescriptorType != USB_INTERFACE_DESCRIPTOR_TYPE) &&
//            (Descriptor->bLength > 0))
//     {
//         Length += Descriptor->bLength;
//         Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)Descriptor +
//                                                 Descriptor->bLength);
//     }
//
//     return Length;
// }
//
// PUSB_INTERFACE_DESCRIPTOR
// NTAPI
// USBPORT_ParseConfigurationDescriptor(IN PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor,
//                                      IN UCHAR InterfaceNumber,
//                                      IN UCHAR Alternate,
//                                      OUT PBOOLEAN HasAlternates)
// {
//     PUSB_CONFIGURATION_DESCRIPTOR TmpDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR iDescriptor;
//     PUSB_INTERFACE_DESCRIPTOR OutDescriptor = NULL;
//     ULONG_PTR Descriptor = (ULONG_PTR)ConfigDescriptor;
//     ULONG_PTR EndDescriptors;
//     ULONG ix;
//
//     DPRINT("USBPORT_ParseConfigurationDescriptor ... \n");
//
//     if (HasAlternates)
//         *HasAlternates = FALSE;
//
//     for (TmpDescriptor = (PUSB_CONFIGURATION_DESCRIPTOR)((ULONG_PTR)ConfigDescriptor + ConfigDescriptor->bLength);
//          TmpDescriptor->bDescriptorType != USB_INTERFACE_DESCRIPTOR_TYPE && TmpDescriptor->bLength > 0;
//          TmpDescriptor = (PUSB_CONFIGURATION_DESCRIPTOR)((ULONG_PTR)TmpDescriptor + TmpDescriptor->bLength))
//     ;
//
//     iDescriptor = (PUSB_INTERFACE_DESCRIPTOR)TmpDescriptor;
//
//     EndDescriptors = (ULONG_PTR)ConfigDescriptor +
//                                 ConfigDescriptor->wTotalLength;
//
//     while ((Descriptor < EndDescriptors) &&
//            (iDescriptor->bInterfaceNumber != InterfaceNumber))
//     {
//         Descriptor = (ULONG_PTR)iDescriptor +
//                      USBPORT_GetInterfaceLength(iDescriptor, EndDescriptors);
//
//         iDescriptor = (PUSB_INTERFACE_DESCRIPTOR)Descriptor;
//     }
//
//     ix = 0;
//
//     while (Descriptor < EndDescriptors &&
//            iDescriptor->bInterfaceNumber == InterfaceNumber)
//     {
//         if (iDescriptor->bAlternateSetting == Alternate)
//             OutDescriptor = iDescriptor;
//
//         Descriptor = (ULONG_PTR)iDescriptor +
//                      USBPORT_GetInterfaceLength(iDescriptor, EndDescriptors);
//
//         iDescriptor = (PUSB_INTERFACE_DESCRIPTOR)Descriptor;
//
//         ++ix;
//     }
//
//     if ((ix > 1) && HasAlternates)
//         *HasAlternates = TRUE;
//
//     return OutDescriptor;
// }
//
// USBD_STATUS
// NTAPI
// USBPORT_OpenInterface(IN PURB Urb,
//                       IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                       IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBPORT_CONFIGURATION_HANDLE ConfigHandle,
//                       IN PUSBD_INTERFACE_INFORMATION InterfaceInfo,
//                       IN OUT PUSBPORT_INTERFACE_HANDLE *iHandle,
//                       IN BOOLEAN SendSetInterface)
// {
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle = NULL;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSB_ENDPOINT_DESCRIPTOR Descriptor;
//     PUSBD_PIPE_INFORMATION PipeInfo;
//     BOOLEAN HasAlternates;
//     ULONG NumEndpoints;
//     SIZE_T Length;
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     SIZE_T HandleLength;
//     BOOLEAN IsAllocated = FALSE;
//     USHORT MaxPacketSize;
//     USHORT wMaxPacketSize;
//     ULONG ix;
//     USBD_STATUS USBDStatus = USBD_STATUS_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_OpenInterface: ...\n");
//
//     InterfaceDescriptor = USBPORT_ParseConfigurationDescriptor(ConfigHandle->ConfigurationDescriptor,
//                                                                InterfaceInfo->InterfaceNumber,
//                                                                InterfaceInfo->AlternateSetting,
//                                                                &HasAlternates);
//
//     NumEndpoints = InterfaceDescriptor->bNumEndpoints;
//
//     Length = FIELD_OFFSET(USBD_INTERFACE_INFORMATION, Pipes) +
//              NumEndpoints * sizeof(USBD_PIPE_INFORMATION);
//
//     if (HasAlternates && SendSetInterface)
//     {
//         RtlZeroMemory(&SetupPacket, sizeof(SetupPacket));
//
//         SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
//         SetupPacket.bmRequestType.set_Type(BMREQUEST_STANDARD);
//         SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);
//         SetupPacket.bRequest = USB_REQUEST_SET_INTERFACE;
//         SetupPacket.wValue.W = InterfaceInfo->AlternateSetting;
//         SetupPacket.wIndex.W = InterfaceInfo->InterfaceNumber;
//         SetupPacket.wLength = 0;
//
//         USBPORT_SendSetupPacket(DeviceHandle,
//                                 FdoDevice,
//                                 &SetupPacket,
//                                 NULL,
//                                 0,
//                                 NULL,
//                                 &USBDStatus);
//         if (!USBD_SUCCESS(USBDStatus))
//         {
//             goto Exit;
//         }
//     }
//
//     if (*iHandle)
//     {
//         InterfaceHandle = *iHandle;
//     }
//     else
//     {
//         HandleLength = FIELD_OFFSET(USBPORT_INTERFACE_HANDLE, PipeHandle) +
//                        NumEndpoints * sizeof(USBPORT_PIPE_HANDLE);
//
//         InterfaceHandle = ExAllocatePoolWithTag(NonPagedPool,
//                                                 HandleLength,
//                                                 USB_PORT_TAG);
//
//         if (!InterfaceHandle)
//         {
//             USBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
//             goto Exit;
//         }
//
//         RtlZeroMemory(InterfaceHandle, HandleLength);
//
//         for (ix = 0; ix < NumEndpoints; ++ix)
//         {
//             PipeHandle = &InterfaceHandle->PipeHandle[ix];
//
//             PipeHandle->Flags = PIPE_HANDLE_FLAG_CLOSED;
//             PipeHandle->Endpoint = NULL;
//         }
//
//         IsAllocated = TRUE;
//     }
//
//     InterfaceHandle->AlternateSetting = InterfaceInfo->AlternateSetting;
//
//     RtlCopyMemory(&InterfaceHandle->InterfaceDescriptor,
//                   InterfaceDescriptor,
//                   sizeof(USB_INTERFACE_DESCRIPTOR));
//
//     InterfaceInfo->Class = InterfaceDescriptor->bInterfaceClass;
//     InterfaceInfo->SubClass = InterfaceDescriptor->bInterfaceSubClass;
//     InterfaceInfo->Protocol = InterfaceDescriptor->bInterfaceProtocol;
//     InterfaceInfo->Reserved = 0;
//     InterfaceInfo->NumberOfPipes = InterfaceDescriptor->bNumEndpoints;
//
//     Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor +
//                                             InterfaceDescriptor->bLength);
//
//     for (ix = 0; ix < NumEndpoints; ++ix)
//     {
//         PipeHandle = &InterfaceHandle->PipeHandle[ix];
//
//         while (Descriptor->bDescriptorType != USB_ENDPOINT_DESCRIPTOR_TYPE)
//         {
//             if (Descriptor->bLength == 0)
//             {
//                 break;
//             }
//             else
//             {
//                 Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)Descriptor +
//                                                         Descriptor->bLength);
//             }
//         }
//
//         if (InterfaceInfo->Pipes[ix].PipeFlags & USBD_PF_CHANGE_MAX_PACKET)
//         {
//             Descriptor->wMaxPacketSize = InterfaceInfo->Pipes[ix].MaximumPacketSize;
//         }
//
//         RtlCopyMemory(&PipeHandle->EndpointDescriptor,
//                       Descriptor,
//                       sizeof(USB_ENDPOINT_DESCRIPTOR));
//
//         PipeHandle->Flags = PIPE_HANDLE_FLAG_CLOSED;
//         PipeHandle->PipeFlags = InterfaceInfo->Pipes[ix].PipeFlags;
//         PipeHandle->Endpoint = NULL;
//
//         wMaxPacketSize = Descriptor->wMaxPacketSize;
//
//         /* USB 2.0 Specification, 5.9 High-Speed, High Bandwidth Endpoints */
//         MaxPacketSize = (wMaxPacketSize & 0x7FF) * (((wMaxPacketSize >> 11) & 3) + 1);
//
//         InterfaceInfo->Pipes[ix].EndpointAddress = Descriptor->bEndpointAddress;
//         InterfaceInfo->Pipes[ix].PipeType = Descriptor->bmAttributes & USB_ENDPOINT_TYPE_MASK;
//         InterfaceInfo->Pipes[ix].MaximumPacketSize = MaxPacketSize;
//         InterfaceInfo->Pipes[ix].PipeHandle = (USBD_PIPE_HANDLE)-1;
//         InterfaceInfo->Pipes[ix].Interval = Descriptor->bInterval;
//
//         Descriptor = (PUSB_ENDPOINT_DESCRIPTOR)((ULONG_PTR)Descriptor +
//                                                 Descriptor->bLength);
//     }
//
//     if (USBD_SUCCESS(USBDStatus))
//     {
//         for (ix = 0; ix < NumEndpoints; ++ix)
//         {
//             PipeInfo = &InterfaceInfo->Pipes[ix];
//             PipeHandle = &InterfaceHandle->PipeHandle[ix];
//
//             Status = USBPORT_OpenPipe(FdoDevice,
//                                       DeviceHandle,
//                                       PipeHandle,
//                                       &USBDStatus);
//
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             PipeInfo->PipeHandle = PipeHandle;
//         }
//
//         if (NumEndpoints)
//         {
//             USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//         }
//     }
//
// Exit:
//
//     if (USBD_SUCCESS(USBDStatus))
//     {
//         InterfaceInfo->InterfaceHandle = InterfaceHandle;
//         *iHandle = InterfaceHandle;
//         InterfaceInfo->Length = Length;
//     }
//     else
//     {
//         if (InterfaceHandle)
//         {
//             if (NumEndpoints)
//             {
//                 DPRINT1("USBPORT_OpenInterface: USBDStatus - %lx\n", USBDStatus);
//             }
//
//             if (IsAllocated)
//                 ExFreePoolWithTag(InterfaceHandle, USB_PORT_TAG);
//         }
//     }
//
//     return USBDStatus;
// }
//
// VOID
// NTAPI
// USBPORT_CloseConfiguration(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                            IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_CONFIGURATION_HANDLE ConfigHandle;
//     PLIST_ENTRY iHandleList;
//     PUSBPORT_INTERFACE_HANDLE iHandle;
//     ULONG NumEndpoints;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//
//     DPRINT("USBPORT_CloseConfiguration: ... \n");
//
//     ConfigHandle = DeviceHandle->ConfigHandle;
//
//     if (ConfigHandle)
//     {
//         iHandleList = &ConfigHandle->InterfaceHandleList;
//
//         while (!IsListEmpty(iHandleList))
//         {
//             iHandle = CONTAINING_RECORD(iHandleList->Flink,
//                                         USBPORT_INTERFACE_HANDLE,
//                                         InterfaceLink);
//
//             DPRINT("USBPORT_CloseConfiguration: iHandle - %p\n", iHandle);
//
//             RemoveHeadList(iHandleList);
//
//             NumEndpoints = iHandle->InterfaceDescriptor.bNumEndpoints;
//
//             PipeHandle = &iHandle->PipeHandle[0];
//
//             while (NumEndpoints > 0)
//             {
//                 USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
//                 PipeHandle += 1;
//                 --NumEndpoints;
//             }
//
//             ExFreePoolWithTag(iHandle, USB_PORT_TAG);
//         }
//
//         ExFreePoolWithTag(ConfigHandle, USB_PORT_TAG);
//         DeviceHandle->ConfigHandle = NULL;
//     }
// }
//
// NTSTATUS
// NTAPI
// USBPORT_InitInterfaceInfo(IN PUSBD_INTERFACE_INFORMATION InterfaceInfo,
//                           IN PUSBPORT_CONFIGURATION_HANDLE ConfigHandle)
// {
//     PUSB_INTERFACE_DESCRIPTOR Descriptor;
//     PUSBD_PIPE_INFORMATION Pipe;
//     SIZE_T Length;
//     ULONG PipeFlags;
//     ULONG NumberOfPipes;
//     USBD_STATUS USBDStatus = USBD_STATUS_SUCCESS;
//
//     DPRINT("USBPORT_InitInterfaceInfo: InterfaceInfo - %p, ConfigHandle - %p\n",
//            InterfaceInfo,
//            ConfigHandle);
//
//     Descriptor = USBPORT_ParseConfigurationDescriptor(ConfigHandle->ConfigurationDescriptor,
//                                                       InterfaceInfo->InterfaceNumber,
//                                                       InterfaceInfo->AlternateSetting,
//                                                       NULL);
//
//     Length = sizeof(USBD_INTERFACE_INFORMATION) +
//              sizeof(USBD_PIPE_INFORMATION);
//
//     if (Descriptor)
//     {
//         NumberOfPipes = Descriptor->bNumEndpoints;
//
//         Length = FIELD_OFFSET(USBD_INTERFACE_INFORMATION, Pipes) +
//                  NumberOfPipes * sizeof(USBD_PIPE_INFORMATION);
//
//         if (InterfaceInfo->Length >= Length)
//         {
//             InterfaceInfo->Class = 0;
//             InterfaceInfo->SubClass = 0;
//             InterfaceInfo->Protocol = 0;
//             InterfaceInfo->Reserved = 0;
//             InterfaceInfo->InterfaceHandle = 0;
//             InterfaceInfo->NumberOfPipes = NumberOfPipes;
//
//             Pipe = InterfaceInfo->Pipes;
//
//             while (NumberOfPipes > 0)
//             {
//                 Pipe->EndpointAddress = 0;
//                 Pipe->Interval = 0;
//                 Pipe->PipeType = 0;
//                 Pipe->PipeHandle = 0;
//
//                 PipeFlags = Pipe->PipeFlags;
//
//                 if (PipeFlags & ~USBD_PF_VALID_MASK)
//                     USBDStatus = USBD_STATUS_INVALID_PIPE_FLAGS;
//
//                 if (!(PipeFlags & USBD_PF_CHANGE_MAX_PACKET))
//                     Pipe->MaximumPacketSize = 0;
//
//                 Pipe += 1;
//                 --NumberOfPipes;
//             }
//         }
//         else
//         {
//             USBDStatus = USBD_STATUS_BUFFER_TOO_SMALL;
//         }
//     }
//     else
//     {
//         USBDStatus = USBD_STATUS_INTERFACE_NOT_FOUND;
//     }
//
//     InterfaceInfo->Length = Length;
//     return USBDStatus;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleSelectConfiguration(IN PDEVICE_OBJECT FdoDevice,
//                                   IN PIRP Irp,
//                                   IN PURB Urb)
// {
//     PUSB_CONFIGURATION_DESCRIPTOR ConfigDescriptor;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_CONFIGURATION_HANDLE ConfigHandle = NULL;
//     PUSBD_INTERFACE_INFORMATION InterfaceInfo;
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle;
//     ULONG iNumber;
//     ULONG ix;
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     NTSTATUS Status;
//     USBD_STATUS USBDStatus;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_HandleSelectConfiguration: ConfigDescriptor %p\n",
//            Urb->UrbSelectConfiguration.ConfigurationDescriptor);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//     ConfigDescriptor = Urb->UrbSelectConfiguration.ConfigurationDescriptor;
//
//     if (!ConfigDescriptor)
//     {
//         DPRINT("USBPORT_HandleSelectConfiguration: ConfigDescriptor == NULL\n");
//
//         RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//         SetupPacket.bmRequestType.B = 0;
//         SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
//         SetupPacket.wValue.W = 0;
//         SetupPacket.wIndex.W = 0;
//         SetupPacket.wLength = 0;
//
//         USBPORT_SendSetupPacket(DeviceHandle,
//                                 FdoDevice,
//                                 &SetupPacket,
//                                 NULL,
//                                 0,
//                                 NULL,
//                                 NULL);
//
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//         goto Exit;
//     }
//
//     USBPORT_DumpingConfiguration(ConfigDescriptor);
//
//     InterfaceInfo = &Urb->UrbSelectConfiguration.Interface;
//
//     iNumber = 0;
//
//     do
//     {
//         ++iNumber;
//         InterfaceInfo = (PUSBD_INTERFACE_INFORMATION)
//                         ((ULONG_PTR)InterfaceInfo +
//                          InterfaceInfo->Length);
//     }
//     while ((ULONG_PTR)InterfaceInfo < (ULONG_PTR)Urb + Urb->UrbHeader.Length);
//
//     if ((iNumber <= 0) || (iNumber != ConfigDescriptor->bNumInterfaces))
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INVALID_CONFIGURATION_DESCRIPTOR);
//         goto Exit;
//     }
//
//     ConfigHandle = ExAllocatePoolWithTag(NonPagedPool,
//                                          ConfigDescriptor->wTotalLength + sizeof(USBPORT_CONFIGURATION_HANDLE),
//                                          USB_PORT_TAG);
//
//     if (!ConfigHandle)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_INSUFFICIENT_RESOURCES);
//         goto Exit;
//     }
//
//     RtlZeroMemory(ConfigHandle,
//                   ConfigDescriptor->wTotalLength + sizeof(USBPORT_CONFIGURATION_HANDLE));
//
//     InitializeListHead(&ConfigHandle->InterfaceHandleList);
//
//     ConfigHandle->ConfigurationDescriptor = (PUSB_CONFIGURATION_DESCRIPTOR)(ConfigHandle + 1);
//
//     RtlCopyMemory(ConfigHandle->ConfigurationDescriptor,
//                   ConfigDescriptor,
//                   ConfigDescriptor->wTotalLength);
//
//     RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     SetupPacket.bmRequestType.B = 0;
//     SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
//     SetupPacket.wValue.W = ConfigDescriptor->bConfigurationValue;
//     SetupPacket.wIndex.W = 0;
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
//     if (USBD_ERROR(USBDStatus))
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_SET_CONFIG_FAILED);
//         goto Exit;
//     }
//
//     if (iNumber <= 0)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_SUCCESS);
//
//         goto Exit;
//     }
//
//     InterfaceInfo = &Urb->UrbSelectConfiguration.Interface;
//
//     for (ix = 0; ix < iNumber; ++ix)
//     {
//         USBDStatus = USBPORT_InitInterfaceInfo(InterfaceInfo,
//                                                ConfigHandle);
//
//         InterfaceHandle = NULL;
//
//         if (USBD_SUCCESS(USBDStatus))
//         {
//             USBDStatus = USBPORT_OpenInterface(Urb,
//                                                DeviceHandle,
//                                                FdoDevice,
//                                                ConfigHandle,
//                                                InterfaceInfo,
//                                                &InterfaceHandle,
//                                                TRUE);
//         }
//
//         if (InterfaceHandle)
//         {
//             InsertTailList(&ConfigHandle->InterfaceHandleList,
//                            &InterfaceHandle->InterfaceLink);
//         }
//
//         if (USBD_ERROR(USBDStatus))
//             break;
//
//         InterfaceInfo = (PUSBD_INTERFACE_INFORMATION)
//                          ((ULONG_PTR)InterfaceInfo +
//                           InterfaceInfo->Length);
//     }
//
//     if (ix >= iNumber)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb,
//                                               USBD_STATUS_SUCCESS);
//     }
//     else
//     {
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//     }
//
// Exit:
//
//     if (NT_SUCCESS(Status))
//     {
//         Urb->UrbSelectConfiguration.ConfigurationHandle = ConfigHandle;
//         DeviceHandle->ConfigHandle = ConfigHandle;
//     }
//     else
//     {
//         DPRINT1("USBPORT_HandleSelectConfiguration: Status %x\n", Status);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_AddDeviceHandle(IN PDEVICE_OBJECT FdoDevice,
//                         IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_AddDeviceHandle: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     InsertTailList(&FdoExtension->DeviceHandleList,
//                    &DeviceHandle->DeviceHandleLink);
// }
//
// VOID
// NTAPI
// USBPORT_RemoveDeviceHandle(IN PDEVICE_OBJECT FdoDevice,
//                            IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_RemoveDeviceHandle \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->DeviceHandleSpinLock, &OldIrql);
//     RemoveEntryList(&DeviceHandle->DeviceHandleLink);
//     KeReleaseSpinLock(&FdoExtension->DeviceHandleSpinLock, OldIrql);
// }
//
// BOOLEAN
// NTAPI
// USBPORT_ValidateDeviceHandle(IN PDEVICE_OBJECT FdoDevice,
//                              IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//     PLIST_ENTRY HandleList;
//     PUSBPORT_DEVICE_HANDLE CurrentHandle;
//     BOOLEAN Result = FALSE;
//
//     //DPRINT("USBPORT_ValidateDeviceHandle: DeviceHandle - %p\n", DeviceHandle \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->DeviceHandleSpinLock, &OldIrql);
//     if (DeviceHandle)
//     {
//         HandleList = FdoExtension->DeviceHandleList.Flink;
//
//         while (HandleList != &FdoExtension->DeviceHandleList)
//         {
//             CurrentHandle = CONTAINING_RECORD(HandleList,
//                                               USBPORT_DEVICE_HANDLE,
//                                               DeviceHandleLink);
//
//             if (CurrentHandle == DeviceHandle)
//             {
//                 Result = TRUE;
//                 break;
//             }
//
//             HandleList = HandleList->Flink;
//         }
//     }
//     KeReleaseSpinLock(&FdoExtension->DeviceHandleSpinLock, OldIrql);
//
//     return Result;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_DeviceHasTransfers(IN PDEVICE_OBJECT FdoDevice,
//                            IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
// {
//     PLIST_ENTRY PipeHandleList;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//
//     DPRINT("USBPORT_DeviceHasTransfers: ... \n");
//
//     PipeHandleList = DeviceHandle->PipeHandleList.Flink;
//
//     while (PipeHandleList != &DeviceHandle->PipeHandleList)
//     {
//         PipeHandle = CONTAINING_RECORD(PipeHandleList,
//                                        USBPORT_PIPE_HANDLE,
//                                        PipeLink);
//
//         PipeHandleList = PipeHandleList->Flink;
//
//         if (!(PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE) &&
//             USBPORT_EndpointHasQueuedTransfers(FdoDevice, PipeHandle->Endpoint, NULL))
//         {
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// VOID
// NTAPI
// USBPORT_AbortTransfers(IN PDEVICE_OBJECT FdoDevice,
//                        IN PUSBPORT_DEVICE_HANDLE DeviceHandle)
// {
//     PLIST_ENTRY HandleList;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     BOOLEAN Result;
//
//     DPRINT("USBPORT_AbortAllTransfers: ... \n");
//
//     HandleList = DeviceHandle->PipeHandleList.Flink;
//
//     while (HandleList != &DeviceHandle->PipeHandleList)
//     {
//         PipeHandle = CONTAINING_RECORD(HandleList,
//                                        USBPORT_PIPE_HANDLE,
//                                        PipeLink);
//
//         HandleList = HandleList->Flink;
//
//         if (!(PipeHandle->Flags & DEVICE_HANDLE_FLAG_ROOTHUB))
//         {
//             PipeHandle->Endpoint->Flags |= ENDPOINT_FLAG_ABORTING;
//
//             USBPORT_AbortEndpoint(FdoDevice, PipeHandle->Endpoint, NULL);
//             USBPORT_FlushMapTransfers(FdoDevice);
//         }
//     }
//
//     while (TRUE)
//     {
//         Result = USBPORT_DeviceHasTransfers(FdoDevice, DeviceHandle);
//
//         if (!Result)
//             break;
//
//         USBPORT_Wait(FdoDevice, 100);
//     }
// }
//
// PUSB2_TT_EXTENSION
// NTAPI
// USBPORT_GetTt(IN PDEVICE_OBJECT FdoDevice,
//               IN PUSBPORT_DEVICE_HANDLE HubDeviceHandle,
//               OUT PUSHORT OutPort,
//               OUT PUSBPORT_DEVICE_HANDLE * OutHubDeviceHandle)
// {
//     PUSBPORT_DEVICE_HANDLE DeviceHandle = HubDeviceHandle;
//     ULONG TtCount;
//     PLIST_ENTRY Entry;
//     PUSB2_TT_EXTENSION TtExtension = NULL;
//
//     DPRINT("USBPORT_GetTt: HubDeviceHandle - %p\n", HubDeviceHandle);
//
//     *OutHubDeviceHandle = NULL;
//
//     while (DeviceHandle->DeviceSpeed != UsbHighSpeed)
//     {
//         DPRINT("USBPORT_GetTt: DeviceHandle - %p, DeviceHandle->PortNumber - %X\n",
//                DeviceHandle,
//                DeviceHandle->PortNumber);
//
//         *OutPort = DeviceHandle->PortNumber;
//
//         DeviceHandle = DeviceHandle->HubDeviceHandle;
//
//         if (!DeviceHandle)
//             return NULL;
//     }
//
//     TtCount = DeviceHandle->TtCount;
//
//     if (!TtCount)
//         return NULL;
//
//     if (IsListEmpty(&DeviceHandle->TtList))
//         return NULL;
//
//     Entry = DeviceHandle->TtList.Flink;
//
//     if (TtCount > 1)
//     {
//         while (Entry != &DeviceHandle->TtList)
//         {
//             ASSERT(Entry != NULL);
//
//             TtExtension = CONTAINING_RECORD(Entry,
//                                             USB2_TT_EXTENSION,
//                                             Link);
//
//             if (TtExtension->TtNumber == *OutPort)
//                 break;
//
//             Entry = Entry->Flink;
//
//             TtExtension = NULL;
//         }
//     }
//     else
//     {
//         TtExtension = CONTAINING_RECORD(Entry,
//                                         USB2_TT_EXTENSION,
//                                         Link);
//     }
//
//     *OutHubDeviceHandle = DeviceHandle;
//
//     return TtExtension;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_CreateDevice(IN OUT PUSB_DEVICE_HANDLE *pUsbdDeviceHandle,
//                      IN PDEVICE_OBJECT FdoDevice,
//                      IN PUSBPORT_DEVICE_HANDLE HubDeviceHandle,
//                      IN USHORT PortStatus,
//                      IN USHORT Port)
// {
//     PUSBPORT_DEVICE_HANDLE TtDeviceHandle = NULL;
//     PUSB2_TT_EXTENSION TtExtension = NULL;
//     USHORT port;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     BOOL IsOpenedPipe;
//     PVOID DeviceDescriptor;
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     ULONG TransferedLen;
//     ULONG DescriptorMinSize;
//     UCHAR MaxPacketSize;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     NTSTATUS Status;
//
//     DPRINT("USBPORT_CreateDevice: PortStatus - %p, Port - %x\n",
//            PortStatus,
//            Port);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     if (!USBPORT_ValidateDeviceHandle(FdoDevice, HubDeviceHandle))
//     {
//         KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         DPRINT1("USBPORT_CreateDevice: Not valid hub DeviceHandle\n");
//         return STATUS_DEVICE_NOT_CONNECTED;
//     }
//
//     port = Port;
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2 &&
//         !(PortStatus & USB_PORT_STATUS_HIGH_SPEED))
//     {
//         DPRINT1("USBPORT_CreateDevice: USB1 device connected to USB2 port\n");
//
//         TtExtension = USBPORT_GetTt(FdoDevice,
//                                     HubDeviceHandle,
//                                     &port,
//                                     &TtDeviceHandle);
//
//         DPRINT("USBPORT_CreateDevice: TtDeviceHandle - %p, port - %x\n",
//                TtDeviceHandle,
//                port);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     DeviceHandle = ExAllocatePoolWithTag(NonPagedPool,
//                                          sizeof(USBPORT_DEVICE_HANDLE),
//                                          USB_PORT_TAG);
//
//     if (!DeviceHandle)
//     {
//         DPRINT1("USBPORT_CreateDevice: Not allocated DeviceHandle\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(DeviceHandle, sizeof(USBPORT_DEVICE_HANDLE));
//
//     *pUsbdDeviceHandle = NULL;
//
//     DeviceHandle->TtExtension = TtExtension;
//     DeviceHandle->PortNumber = Port;
//     DeviceHandle->HubDeviceHandle = HubDeviceHandle;
//
//     if (PortStatus & USB_PORT_STATUS_LOW_SPEED)
//     {
//         DeviceHandle->DeviceSpeed = UsbLowSpeed;
//     }
//     else if (PortStatus & USB_PORT_STATUS_HIGH_SPEED)
//     {
//         DeviceHandle->DeviceSpeed = UsbHighSpeed;
//     }
//     else
//     {
//         DeviceHandle->DeviceSpeed = UsbFullSpeed;
//     }
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     PipeHandle = &DeviceHandle->PipeHandle;
//
//     PipeHandle->Flags = PIPE_HANDLE_FLAG_CLOSED;
//
//     PipeHandle->EndpointDescriptor.bLength = sizeof(PipeHandle->EndpointDescriptor);
//     PipeHandle->EndpointDescriptor.bDescriptorType = USB_ENDPOINT_DESCRIPTOR_TYPE;
//
//     if (DeviceHandle->DeviceSpeed == UsbLowSpeed)
//     {
//         PipeHandle->EndpointDescriptor.wMaxPacketSize = 8;
//     }
//     else
//     {
//         PipeHandle->EndpointDescriptor.wMaxPacketSize = USB_DEFAULT_MAX_PACKET;
//     }
//
//     InitializeListHead(&DeviceHandle->PipeHandleList);
//     InitializeListHead(&DeviceHandle->TtList);
//
//     Status = USBPORT_OpenPipe(FdoDevice,
//                               DeviceHandle,
//                               PipeHandle,
//                               NULL);
//
//     IsOpenedPipe = NT_SUCCESS(Status);
//
//     if (NT_ERROR(Status))
//     {
//         DPRINT1("USBPORT_CreateDevice: USBPORT_OpenPipe return - %lx\n", Status);
//
//         KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         ExFreePoolWithTag(DeviceHandle, USB_PORT_TAG);
//
//         return Status;
//     }
//
//     DeviceDescriptor = ExAllocatePoolWithTag(NonPagedPool,
//                                              USB_DEFAULT_MAX_PACKET,
//                                              USB_PORT_TAG);
//
//     if (!DeviceDescriptor)
//     {
//         DPRINT1("USBPORT_CreateDevice: Not allocated DeviceDescriptor\n");
//         goto ErrorExit;
//     }
//
//     RtlZeroMemory(DeviceDescriptor, USB_DEFAULT_MAX_PACKET);
//     RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     SetupPacket.bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST);
//     SetupPacket.bRequest = USB_REQUEST_GET_DESCRIPTOR;
//     SetupPacket.wValue.HiByte = USB_DEVICE_DESCRIPTOR_TYPE;
//     SetupPacket.wLength = USB_DEFAULT_MAX_PACKET;
//
//     TransferedLen = 0;
//
//     Status = USBPORT_SendSetupPacket(DeviceHandle,
//                                      FdoDevice,
//                                      &SetupPacket,
//                                      DeviceDescriptor,
//                                      USB_DEFAULT_MAX_PACKET,
//                                      &TransferedLen,
//                                      NULL);
//
//     RtlCopyMemory(&DeviceHandle->DeviceDescriptor,
//                   DeviceDescriptor,
//                   sizeof(USB_DEVICE_DESCRIPTOR));
//
//     ExFreePoolWithTag(DeviceDescriptor, USB_PORT_TAG);
//
//     DescriptorMinSize = RTL_SIZEOF_THROUGH_FIELD(USB_DEVICE_DESCRIPTOR,
//                                                  bMaxPacketSize0);
//
//     if ((TransferedLen == DescriptorMinSize) && !NT_SUCCESS(Status))
//     {
//         Status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(Status) && (TransferedLen >= DescriptorMinSize))
//     {
//         if ((DeviceHandle->DeviceDescriptor.bLength >= sizeof(USB_DEVICE_DESCRIPTOR)) &&
//             (DeviceHandle->DeviceDescriptor.bDescriptorType == USB_DEVICE_DESCRIPTOR_TYPE))
//         {
//             MaxPacketSize = DeviceHandle->DeviceDescriptor.bMaxPacketSize0;
//
//             if (MaxPacketSize == 8 ||
//                 MaxPacketSize == 16 ||
//                 MaxPacketSize == 32 ||
//                 MaxPacketSize == 64)
//             {
//                 USBPORT_AddDeviceHandle(FdoDevice, DeviceHandle);
//
//                 *pUsbdDeviceHandle = DeviceHandle;
//
//                 KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                                    LOW_REALTIME_PRIORITY,
//                                    1,
//                                    FALSE);
//
//                 return Status;
//             }
//         }
//     }
//
//     DPRINT1("USBPORT_CreateDevice: ERROR!!! TransferedLen - %x, Status - %lx\n",
//             TransferedLen,
//             Status);
//
// ErrorExit:
//
//     if (TtExtension && TtDeviceHandle)
//     {
//         SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_OTHER);
//         SetupPacket.bmRequestType.Reserved = 0;
//         SetupPacket.bmRequestType.set_Type(BMREQUEST_CLASS);
//         SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
//
//         /* Table 11-15.  Hub Class Requests */
//         if (TtDeviceHandle == HubDeviceHandle)
//         {
//             SetupPacket.bRequest = USB_REQUEST_RESET_TT;
//         }
//         else
//         {
//             SetupPacket.bRequest = USB_REQUEST_CLEAR_TT_BUFFER;
//         }
//
//         SetupPacket.wValue.LowByte = 0;
//         SetupPacket.wValue.HiByte = 0;
//         SetupPacket.wIndex.W = port;
//         SetupPacket.wLength = 0;
//
//         USBPORT_SendSetupPacket(TtDeviceHandle,
//                                 FdoDevice,
//                                 &SetupPacket,
//                                 NULL,
//                                 0,
//                                 NULL,
//                                 NULL);
//     }
//
//     Status = STATUS_DEVICE_DATA_ERROR;
//
//     if (IsOpenedPipe)
//     {
//         USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     ExFreePoolWithTag(DeviceHandle, USB_PORT_TAG);
//
//     return Status;
// }
//
// ULONG
// NTAPI
// USBPORT_AllocateUsbAddress(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     ULONG BitMapIdx;
//     ULONG BitNumber;
//     ULONG ix;
//
//     DPRINT("USBPORT_AllocateUsbAddress \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     for (ix = 0; ix < 4; ++ix)
//     {
//         BitMapIdx = 1;
//
//         for (BitNumber = 0; BitNumber < 32; ++BitNumber)
//         {
//             if (!(FdoExtension->UsbAddressBitMap[ix] & BitMapIdx))
//             {
//                 FdoExtension->UsbAddressBitMap[ix] |= BitMapIdx;
//                 return 32 * ix + BitNumber;
//             }
//
//             BitMapIdx <<= 2;
//         }
//     }
//
//     return 0;
// }
//
// VOID
// NTAPI
// USBPORT_FreeUsbAddress(IN PDEVICE_OBJECT FdoDevice,
//                        IN USHORT DeviceAddress)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     ULONG ix;
//     ULONG BitMapIdx;
//     ULONG BitNumber;
//     USHORT CurrentAddress;
//
//     DPRINT("USBPORT_FreeUsbAddress: DeviceAddress - %x\n", DeviceAddress);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     for (ix = 0; ix < 4; ++ix)
//     {
//         BitMapIdx = 1;
//         CurrentAddress = 32 * ix;
//
//         for (BitNumber = 0; BitNumber < 32; ++BitNumber)
//         {
//             if (CurrentAddress == DeviceAddress)
//             {
//                 FdoExtension->UsbAddressBitMap[ix] &= ~BitMapIdx;
//                 return;
//             }
//
//             BitMapIdx <<= 2;
//             CurrentAddress++;
//         }
//     }
// }
//
// NTSTATUS
// NTAPI
// USBPORT_InitializeDevice(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                          IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_ENDPOINT Endpoint;
//     USB_DEFAULT_PIPE_SETUP_PACKET CtrlSetup;
//     ULONG TransferedLen;
//     USHORT DeviceAddress = 0;
//     UCHAR MaxPacketSize;
//     NTSTATUS Status;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_InitializeDevice: ... \n");
//
//     ASSERT(DeviceHandle != NULL);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     DeviceAddress = USBPORT_AllocateUsbAddress(FdoDevice);
//     ASSERT(DeviceHandle->DeviceAddress == USB_DEFAULT_DEVICE_ADDRESS);
//
//     RtlZeroMemory(&CtrlSetup, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     CtrlSetup.bRequest = USB_REQUEST_SET_ADDRESS;
//     CtrlSetup.wValue.W = DeviceAddress;
//
//     Status = USBPORT_SendSetupPacket(DeviceHandle,
//                                      FdoDevice,
//                                      &CtrlSetup,
//                                      NULL,
//                                      0,
//                                      NULL,
//                                      NULL);
//
//     DPRINT("USBPORT_InitializeDevice: DeviceAddress - %x. SendSetupPacket Status - %x\n",
//            DeviceAddress,
//            Status);
//
//     if (!NT_SUCCESS(Status))
//         goto ExitError;
//
//     DeviceHandle->DeviceAddress = DeviceAddress;
//     Endpoint = DeviceHandle->PipeHandle.Endpoint;
//
//     Endpoint->EndpointProperties.TotalMaxPacketSize = DeviceHandle->DeviceDescriptor.bMaxPacketSize0;
//     Endpoint->EndpointProperties.DeviceAddress = DeviceAddress;
//
//     Status = USBPORT_ReopenPipe(FdoDevice, Endpoint);
//
//     if (!NT_SUCCESS(Status))
//         goto ExitError;
//
//     USBPORT_Wait(FdoDevice, 10);
//
//     RtlZeroMemory(&CtrlSetup, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     CtrlSetup.bRequest = USB_REQUEST_GET_DESCRIPTOR;
//     CtrlSetup.wValue.HiByte = USB_DEVICE_DESCRIPTOR_TYPE;
//     CtrlSetup.wLength = sizeof(USB_DEVICE_DESCRIPTOR);
//     CtrlSetup.bmRequestType.B = 0x80;
//
//     Status = USBPORT_SendSetupPacket(DeviceHandle,
//                                      FdoDevice,
//                                      &CtrlSetup,
//                                      &DeviceHandle->DeviceDescriptor,
//                                      sizeof(USB_DEVICE_DESCRIPTOR),
//                                      &TransferedLen,
//                                      NULL);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Some devices may return a short transfer here; accept it if the
//            header through bMaxPacketSize0 was received earlier during
//            USBPORT_CreateDevice. */
//         if (TransferedLen < sizeof(USB_DEVICE_DESCRIPTOR))
//         {
//             DPRINT1("USBPORT_InitializeDevice: Short device descriptor len %lu\n",
//                     TransferedLen);
//         }
//
//         /* Use the known bMaxPacketSize0 (was obtained prior to SetAddress) */
//         MaxPacketSize = DeviceHandle->DeviceDescriptor.bMaxPacketSize0;
//
//         if (!(MaxPacketSize == 8 || MaxPacketSize == 16 ||
//               MaxPacketSize == 32 || MaxPacketSize == 64))
//         {
//             DPRINT1("USBPORT_InitializeDevice: Invalid MPS0 %u\n", MaxPacketSize);
//             Status = STATUS_DEVICE_DATA_ERROR;
//             goto ExitError;
//         }
//
//         /* Mark USB2 hub flag when identifiable (bDeviceClass available in header) */
//         if (DeviceHandle->DeviceSpeed == UsbHighSpeed &&
//             DeviceHandle->DeviceDescriptor.bDeviceClass == USB_DEVICE_CLASS_HUB)
//         {
//             DeviceHandle->Flags |= DEVICE_HANDLE_FLAG_USB2HUB;
//         }
//     }
//     else
//     {
// ExitError:
//         DPRINT1("USBPORT_InitializeDevice: ExitError. Status - %x\n", Status);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_GetUsbDescriptor(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                          IN PDEVICE_OBJECT FdoDevice,
//                          IN UCHAR Type,
//                          IN PUCHAR ConfigDesc,
//                          IN PULONG ConfigDescSize)
// {
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//
//     DPRINT("USBPORT_GetUsbDescriptor: Type - %x\n");
//
//     RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     SetupPacket.bmRequestType.set_Dir(BMREQUEST_DEVICE_TO_HOST);
//     SetupPacket.bRequest = USB_REQUEST_GET_DESCRIPTOR;
//     SetupPacket.wValue.HiByte = Type;
//     SetupPacket.wLength = (USHORT)*ConfigDescSize;
//
//     return USBPORT_SendSetupPacket(DeviceHandle,
//                                    FdoDevice,
//                                    &SetupPacket,
//                                    ConfigDesc,
//                                    *ConfigDescSize,
//                                    ConfigDescSize,
//                                    NULL);
// }
//
// PUSBPORT_INTERFACE_HANDLE
// NTAPI
// USBPORT_GetInterfaceHandle(IN PUSBPORT_CONFIGURATION_HANDLE ConfigurationHandle,
//                            IN UCHAR InterfaceNumber)
// {
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle;
//     PLIST_ENTRY iHandleList;
//     UCHAR InterfaceNum;
//
//     DPRINT("USBPORT_GetInterfaceHandle: ConfigurationHandle - %p, InterfaceNumber - %p\n",
//            ConfigurationHandle,
//            InterfaceNumber);
//
//     iHandleList = ConfigurationHandle->InterfaceHandleList.Flink;
//
//     while (iHandleList &&
//            (iHandleList != &ConfigurationHandle->InterfaceHandleList))
//     {
//         InterfaceHandle = CONTAINING_RECORD(iHandleList,
//                                             USBPORT_INTERFACE_HANDLE,
//                                             InterfaceLink);
//
//         InterfaceNum = InterfaceHandle->InterfaceDescriptor.bInterfaceNumber;
//
//         if (InterfaceNum == InterfaceNumber)
//             return InterfaceHandle;
//
//         iHandleList = InterfaceHandle->InterfaceLink.Flink;
//     }
//
//     return NULL;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_HandleSelectInterface(IN PDEVICE_OBJECT FdoDevice,
//                               IN PIRP Irp,
//                               IN PURB Urb)
// {
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_CONFIGURATION_HANDLE ConfigurationHandle;
//     PUSBD_INTERFACE_INFORMATION Interface;
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle;
//     PUSBPORT_INTERFACE_HANDLE iHandle;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     USBD_STATUS USBDStatus;
//     USHORT Length;
//     ULONG ix;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT("USBPORT_HandleSelectInterface: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     ConfigurationHandle = Urb->UrbSelectInterface.ConfigurationHandle;
//
//     Interface = &Urb->UrbSelectInterface.Interface;
//
//     Length = Interface->Length + sizeof(USBD_PIPE_INFORMATION);
//     Urb->UrbHeader.Length = Length;
//
//     USBDStatus = USBPORT_InitInterfaceInfo(Interface, ConfigurationHandle);
//
//     if (USBDStatus)
//     {
//         Interface->InterfaceHandle = (USBD_INTERFACE_HANDLE)-1;
//         return USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
//     }
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//
//     InterfaceHandle = USBPORT_GetInterfaceHandle(ConfigurationHandle,
//                                                  Interface->InterfaceNumber);
//
//     if (InterfaceHandle)
//     {
//         RemoveEntryList(&InterfaceHandle->InterfaceLink);
//
//         if (InterfaceHandle->InterfaceDescriptor.bNumEndpoints)
//         {
//             PipeHandle = &InterfaceHandle->PipeHandle[0];
//
//             for (ix = 0;
//                  ix < InterfaceHandle->InterfaceDescriptor.bNumEndpoints;
//                  ix++)
//             {
//                 USBPORT_ClosePipe(DeviceHandle, FdoDevice, PipeHandle);
//                 PipeHandle += 1;
//             }
//         }
//     }
//
//     iHandle = 0;
//
//     USBDStatus = USBPORT_OpenInterface(Urb,
//                                        DeviceHandle,
//                                        FdoDevice,
//                                        ConfigurationHandle,
//                                        Interface,
//                                        &iHandle,
//                                        TRUE);
//
//     if (USBDStatus)
//     {
//         Interface->InterfaceHandle = (USBD_INTERFACE_HANDLE)-1;
//     }
//     else
//     {
//         if (InterfaceHandle)
//             ExFreePoolWithTag(InterfaceHandle, USB_PORT_TAG);
//
//         Interface->InterfaceHandle = iHandle;
//
//         InsertTailList(&ConfigurationHandle->InterfaceHandleList,
//                        &iHandle->InterfaceLink);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     return USBPORT_USBDStatusToNtStatus(Urb, USBDStatus);
// }
//
// NTSTATUS
// NTAPI
// USBPORT_RemoveDevice(IN PDEVICE_OBJECT FdoDevice,
//                      IN OUT PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                      IN ULONG Flags)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSB2_TT_EXTENSION TtExtension;
//     ULONG ix;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_RemoveDevice: DeviceHandle - %p, Flags - %x\n",
//            DeviceHandle,
//            Flags);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if ((Flags & USBD_KEEP_DEVICE_DATA) ||
//         (Flags & USBD_MARK_DEVICE_BUSY))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     if (!USBPORT_ValidateDeviceHandle(FdoDevice, DeviceHandle))
//     {
//         KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
//         DPRINT1("USBPORT_RemoveDevice: Not valid device handle\n");
//         return STATUS_DEVICE_NOT_CONNECTED;
//     }
//
//     USBPORT_RemoveDeviceHandle(FdoDevice, DeviceHandle);
//
//     DeviceHandle->Flags |= DEVICE_HANDLE_FLAG_REMOVED;
//
//     USBPORT_AbortTransfers(FdoDevice, DeviceHandle);
//
//     DPRINT("USBPORT_RemoveDevice: DeviceHandleLock - %x\n",
//            DeviceHandle->DeviceHandleLock);
//
//     while (InterlockedDecrement(&DeviceHandle->DeviceHandleLock) >= 0)
//     {
//         InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//         USBPORT_Wait(FdoDevice, 100);
//     }
//
//     DPRINT("USBPORT_RemoveDevice: DeviceHandleLock ok\n");
//
//     if (DeviceHandle->ConfigHandle)
//     {
//         USBPORT_CloseConfiguration(DeviceHandle, FdoDevice);
//     }
//
//     USBPORT_ClosePipe(DeviceHandle, FdoDevice, &DeviceHandle->PipeHandle);
//
//     if (DeviceHandle->DeviceAddress)
//     {
//         USBPORT_FreeUsbAddress(FdoDevice, DeviceHandle->DeviceAddress);
//     }
//
//     if (!IsListEmpty(&DeviceHandle->TtList))
//     {
//         DPRINT1("USBPORT_RemoveDevice: DeviceHandle->TtList not empty\n");
//     }
//
//     while (!IsListEmpty(&DeviceHandle->TtList))
//     {
//         TtExtension = CONTAINING_RECORD(DeviceHandle->TtList.Flink,
//                                         USB2_TT_EXTENSION,
//                                         Link);
//
//         RemoveHeadList(&DeviceHandle->TtList);
//
//         DPRINT("USBPORT_RemoveDevice: TtExtension - %p\n", TtExtension);
//
//         KeAcquireSpinLock(&FdoExtension->TtSpinLock, &OldIrql);
//
//         TtExtension->Flags |= USB2_TT_EXTENSION_FLAG_DELETED;
//
//         if (IsListEmpty(&TtExtension->EndpointList))
//         {
//             USBPORT_UpdateAllocatedBwTt(TtExtension);
//
//             for (ix = 0; ix < USB2_FRAMES; ix++)
//             {
//                 FdoExtension->Bandwidth[ix] += TtExtension->MaxBandwidth;
//             }
//
//             DPRINT("USBPORT_RemoveDevice: ExFreePoolWithTag TtExtension - %p\n", TtExtension);
//             ExFreePoolWithTag(TtExtension, USB_PORT_TAG);
//         }
//
//         KeReleaseSpinLock(&FdoExtension->TtSpinLock, OldIrql);
//     }
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     if (!(DeviceHandle->Flags & DEVICE_HANDLE_FLAG_ROOTHUB))
//     {
//         ExFreePoolWithTag(DeviceHandle, USB_PORT_TAG);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_RestoreDevice(IN PDEVICE_OBJECT FdoDevice,
//                       IN OUT PUSBPORT_DEVICE_HANDLE OldDeviceHandle,
//                       IN OUT PUSBPORT_DEVICE_HANDLE NewDeviceHandle)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PLIST_ENTRY iHandleList;
//     PUSBPORT_ENDPOINT Endpoint;
//     USBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements = {0};
//     USB_DEFAULT_PIPE_SETUP_PACKET SetupPacket;
//     NTSTATUS Status = STATUS_SUCCESS;
//     USBD_STATUS USBDStatus;
//     KIRQL OldIrql;
//     PUSBPORT_INTERFACE_HANDLE InterfaceHandle;
//     PUSBPORT_PIPE_HANDLE PipeHandle;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//
//     DPRINT("USBPORT_RestoreDevice: OldDeviceHandle - %p, NewDeviceHandle - %p\n",
//            OldDeviceHandle,
//            NewDeviceHandle);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeWaitForSingleObject(&FdoExtension->DeviceSemaphore,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     if (!USBPORT_ValidateDeviceHandle(FdoDevice, OldDeviceHandle))
//     {
//         KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
//
// #ifndef NDEBUG
//         DPRINT("USBPORT_RestoreDevice: OldDeviceHandle not valid\n");
//         DbgBreakPoint();
// #endif
//         return STATUS_DEVICE_NOT_CONNECTED;
//     }
//
//     if (!USBPORT_ValidateDeviceHandle(FdoDevice, NewDeviceHandle))
//     {
//         KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                            LOW_REALTIME_PRIORITY,
//                            1,
//                            FALSE);
// #ifndef NDEBUG
//         DPRINT("USBPORT_RestoreDevice: NewDeviceHandle not valid\n");
//         DbgBreakPoint();
// #endif
//         return STATUS_DEVICE_NOT_CONNECTED;
//     }
//
//     USBPORT_RemoveDeviceHandle(FdoDevice, OldDeviceHandle);
//     USBPORT_AbortTransfers(FdoDevice, OldDeviceHandle);
//
//     while (InterlockedDecrement(&OldDeviceHandle->DeviceHandleLock) >= 0)
//     {
//         InterlockedIncrement(&OldDeviceHandle->DeviceHandleLock);
//         USBPORT_Wait(FdoDevice, 100);
//     }
//
//     if (sizeof(USB_DEVICE_DESCRIPTOR) == RtlCompareMemory(&NewDeviceHandle->DeviceDescriptor,
//                                                           &OldDeviceHandle->DeviceDescriptor,
//                                                           sizeof(USB_DEVICE_DESCRIPTOR)))
//     {
//         NewDeviceHandle->ConfigHandle = OldDeviceHandle->ConfigHandle;
//
//         if (OldDeviceHandle->ConfigHandle)
//         {
//             RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//             SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
//             SetupPacket.bRequest = USB_REQUEST_SET_CONFIGURATION;
//             SetupPacket.wValue.W = OldDeviceHandle->ConfigHandle->ConfigurationDescriptor->bConfigurationValue;
//             SetupPacket.wIndex.W = 0;
//             SetupPacket.wLength = 0;
//
//             USBPORT_SendSetupPacket(NewDeviceHandle,
//                                     FdoDevice,
//                                     &SetupPacket,
//                                     NULL,
//                                     0,
//                                     NULL,
//                                     &USBDStatus);
//
//             if (USBD_ERROR(USBDStatus))
//                 Status = USBPORT_USBDStatusToNtStatus(NULL, USBDStatus);
//
//             if (NT_SUCCESS(Status))
//             {
//                 iHandleList = NewDeviceHandle->ConfigHandle->InterfaceHandleList.Flink;
//
//                 while (iHandleList &&
//                        iHandleList != &NewDeviceHandle->ConfigHandle->InterfaceHandleList)
//                 {
//                     InterfaceHandle = CONTAINING_RECORD(iHandleList,
//                                                         USBPORT_INTERFACE_HANDLE,
//                                                         InterfaceLink);
//
//                     if (InterfaceHandle->AlternateSetting)
//                     {
//                         RtlZeroMemory(&SetupPacket, sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//                         SetupPacket.bmRequestType.set_Dir(BMREQUEST_HOST_TO_DEVICE);
//                         SetupPacket.bmRequestType.set_Type(BMREQUEST_STANDARD);
//                         SetupPacket.bmRequestType.set_Recipient(BMREQUEST_TO_INTERFACE);
//
//                         SetupPacket.bRequest = USB_REQUEST_SET_INTERFACE;
//                         SetupPacket.wValue.W = InterfaceHandle->InterfaceDescriptor.bAlternateSetting;
//                         SetupPacket.wIndex.W = InterfaceHandle->InterfaceDescriptor.bInterfaceNumber;
//                         SetupPacket.wLength = 0;
//
//                         USBPORT_SendSetupPacket(NewDeviceHandle,
//                                                 FdoDevice,
//                                                 &SetupPacket,
//                                                 NULL,
//                                                 0,
//                                                 NULL,
//                                                 &USBDStatus);
//                     }
//
//                     iHandleList = iHandleList->Flink;
//                 }
//             }
//         }
//
//         if (NewDeviceHandle->Flags & DEVICE_HANDLE_FLAG_USB2HUB)
//         {
//             DPRINT1("USBPORT_RestoreDevice: FIXME Transaction Translator\n");
//             NewDeviceHandle->TtCount = OldDeviceHandle->TtCount;
//
// #ifndef NDEBUG
//             DbgBreakPoint();
// #endif
//         }
//
//         while (!IsListEmpty(&OldDeviceHandle->PipeHandleList))
//         {
//             PipeHandle = CONTAINING_RECORD(OldDeviceHandle->PipeHandleList.Flink,
//                                            USBPORT_PIPE_HANDLE,
//                                            PipeLink);
//
//             DPRINT("USBPORT_RestoreDevice: PipeHandle - %p\n", PipeHandle);
//
//             USBPORT_RemovePipeHandle(OldDeviceHandle, PipeHandle);
//
//             if (PipeHandle != &OldDeviceHandle->PipeHandle)
//             {
//                 USBPORT_AddPipeHandle(NewDeviceHandle, PipeHandle);
//
//                 if (!(PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE))
//                 {
//                     Endpoint = PipeHandle->Endpoint;
//                     Endpoint->DeviceHandle = NewDeviceHandle;
//                     Endpoint->EndpointProperties.DeviceAddress = NewDeviceHandle->DeviceAddress;
//
//                     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//                     if (!(Endpoint->Flags & ENDPOINT_FLAG_NUKE))
//                     {
//                         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock,
//                                           &OldIrql);
//
//                         Packet->ReopenEndpoint(FdoExtension->MiniPortExt,
//                                                &Endpoint->EndpointProperties,
//                                                Endpoint + 1);
//
//                         Packet->SetEndpointDataToggle(FdoExtension->MiniPortExt,
//                                                       Endpoint + 1,
//                                                       0);
//
//                         Packet->SetEndpointStatus(FdoExtension->MiniPortExt,
//                                                   Endpoint + 1,
//                                                   USBPORT_ENDPOINT_RUN);
//
//                         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock,
//                                           OldIrql);
//                     }
//                     else
//                     {
//                         MiniportCloseEndpoint(FdoDevice, Endpoint);
//
//                         RtlZeroMemory(Endpoint + 1, Packet->MiniPortEndpointSize);
//
//                         RtlZeroMemory((PVOID)Endpoint->EndpointProperties.BufferVA,
//                                       Endpoint->EndpointProperties.BufferLength);
//
//                         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//                         Packet->QueryEndpointRequirements(FdoExtension->MiniPortExt,
//                                                           &Endpoint->EndpointProperties,
//                                                           &EndpointRequirements);
//
//                         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock,
//                                           OldIrql);
//
//                         MiniportOpenEndpoint(FdoDevice, Endpoint);
//
//                         Endpoint->Flags &= ~(ENDPOINT_FLAG_NUKE |
//                                              ENDPOINT_FLAG_ABORTING);
//
//                         KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                                           &Endpoint->EndpointOldIrql);
//
//                         if (Endpoint->StateLast == USBPORT_ENDPOINT_ACTIVE)
//                         {
//                             KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//
//                             Packet->SetEndpointState(FdoExtension->MiniPortExt,
//                                                      Endpoint + 1,
//                                                      USBPORT_ENDPOINT_ACTIVE);
//
//                             KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//                         }
//
//                         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                                           Endpoint->EndpointOldIrql);
//                     }
//                 }
//             }
//         }
//
//         USBPORT_AddPipeHandle(OldDeviceHandle, &OldDeviceHandle->PipeHandle);
//     }
//     else
//     {
// #ifndef NDEBUG
//         DPRINT("USBPORT_RestoreDevice: New DeviceDescriptor != Old DeviceDescriptor\n");
//         DbgBreakPoint();
// #endif
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     USBPORT_ClosePipe(OldDeviceHandle, FdoDevice, &OldDeviceHandle->PipeHandle);
//
//     if (OldDeviceHandle->DeviceAddress != 0)
//         USBPORT_FreeUsbAddress(FdoDevice, OldDeviceHandle->DeviceAddress);
//
//     KeReleaseSemaphore(&FdoExtension->DeviceSemaphore,
//                        LOW_REALTIME_PRIORITY,
//                        1,
//                        FALSE);
//
//     ExFreePoolWithTag(OldDeviceHandle, USB_PORT_TAG);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_InitializeTT(IN PDEVICE_OBJECT FdoDevice,
//                      IN PUSBPORT_DEVICE_HANDLE HubDeviceHandle,
//                      IN ULONG TtNumber)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSB2_TT_EXTENSION TtExtension;
//     ULONG ix;
//
//     DPRINT("USBPORT_InitializeTT: HubDeviceHandle - %p, TtNumber - %X\n",
//            HubDeviceHandle,
//            TtNumber);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     TtExtension = ExAllocatePoolWithTag(NonPagedPool,
//                                         sizeof(USB2_TT_EXTENSION),
//                                         USB_PORT_TAG);
//
//     if (!TtExtension)
//     {
//         DPRINT1("USBPORT_InitializeTT: ExAllocatePoolWithTag return NULL\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     DPRINT("USBPORT_InitializeTT: TtExtension - %p\n", TtExtension);
//
//     RtlZeroMemory(TtExtension, sizeof(USB2_TT_EXTENSION));
//
//     TtExtension->DeviceAddress = HubDeviceHandle->DeviceAddress;
//     TtExtension->TtNumber = TtNumber;
//     TtExtension->RootHubPdo = FdoExtension->RootHubPdo;
//     TtExtension->BusBandwidth = TOTAL_USB11_BUS_BANDWIDTH;
//
//     InitializeListHead(&TtExtension->EndpointList);
//
//     /* 90% maximum allowed for periodic endpoints */
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         TtExtension->Bandwidth[ix] = TtExtension->BusBandwidth -
//                                      TtExtension->BusBandwidth / 10;
//     }
//
//     USBPORT_UpdateAllocatedBwTt(TtExtension);
//
//     for (ix = 0; ix < USB2_FRAMES; ix++)
//     {
//         FdoExtension->Bandwidth[ix] -= TtExtension->MaxBandwidth;
//     }
//
//     USB2_InitTT(FdoExtension->Usb2Extension, &TtExtension->Tt);
//
//     InsertTailList(&HubDeviceHandle->TtList, &TtExtension->Link);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_Initialize20Hub(IN PDEVICE_OBJECT FdoDevice,
//                         IN PUSBPORT_DEVICE_HANDLE HubDeviceHandle,
//                         IN ULONG TtCount)
// {
//     NTSTATUS Status;
//     ULONG ix;
//
//     DPRINT("USBPORT_Initialize20Hub: TtCount - %X\n", TtCount);
//
//     if (!HubDeviceHandle)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (HubDeviceHandle->Flags & DEVICE_HANDLE_FLAG_ROOTHUB)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (TtCount == 0)
//     {
//         HubDeviceHandle->TtCount = 0;
//         return STATUS_SUCCESS;
//     }
//
//     for (ix = 0; ix < TtCount; ++ix)
//     {
//         Status = USBPORT_InitializeTT(FdoDevice, HubDeviceHandle, ix + 1);
//
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     HubDeviceHandle->TtCount = TtCount;
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

extern "system" {
    fn USBPORT_AbortEndpoint(FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, Irp: PIRP);
    fn USBPORT_FlushMapTransfers(FdoDevice: PDEVICE_OBJECT);
}

// Translated from device.c USBPORT_AbortTransfers (was a commented lazy-stub in the
// pure-LLM pass; hand-translated to finish usbport at 100% Rust).
#[no_mangle]
pub unsafe extern "system" fn USBPORT_AbortTransfers(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
) {
    let mut HandleList: PLIST_ENTRY;
    let mut PipeHandle: PUSBPORT_PIPE_HANDLE;
    let mut Result: BOOLEAN;

    HandleList = (*DeviceHandle).PipeHandleList.Flink;

    while HandleList != core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList) {
        PipeHandle = (HandleList as *mut u8)
            .offset(-(core::mem::offset_of!(USBPORT_PIPE_HANDLE, PipeLink) as isize))
            as *mut USBPORT_PIPE_HANDLE;

        HandleList = (*HandleList).Flink;

        if ((*PipeHandle).Flags & DEVICE_HANDLE_FLAG_ROOTHUB) == 0 {
            (*(*PipeHandle).Endpoint).Flags |= ENDPOINT_FLAG_ABORTING;
            USBPORT_AbortEndpoint(FdoDevice, (*PipeHandle).Endpoint, core::ptr::null_mut());
            USBPORT_FlushMapTransfers(FdoDevice);
        }
    }

    loop {
        Result = USBPORT_DeviceHasTransfers(FdoDevice, DeviceHandle);
        if Result == 0 {
            break;
        }
        USBPORT_Wait((FdoDevice) as _, 100);
    }
}
