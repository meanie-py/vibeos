use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, ExAllocatePoolZero, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAllocateIrp, IoBuildSynchronousFsdRequest, IoFreeIrp, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LPGUID, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PINTERFACE, PIO_STACK_LOCATION, PIRP, PKEVENT, GUID, PIO_COMPLETION_ROUTINE};
use ntkrnl_sys::inline::tier1::IoSetCompletionRoutine;
use core::ffi::c_void;

extern "system" {
    static USB_BUS_INTERFACE_USBDI_GUID: GUID;
}

// ---- file-scope preamble (translated) ----
// (no constant or type declarations to emit)

// STATUS_MORE_PROCESSING_REQUIRED (ntstatus.h)
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;
// IRP_MN_QUERY_INTERFACE (wdm.h)
const IRP_MN_QUERY_INTERFACE: u8 = 0x08;

const USB_BULK_GET_MAX_LUN: u32 = 0xFE;
const USBD_TRANSFER_DIRECTION_IN: u32 = 0x00000001;
pub const USB_STOR_TAG: u32 = 0x73627375;
const MAX_LUN: i32 = 0x0F;

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_SyncForwardIrpCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    if (*Irp).PendingReturned != 0 {
        KeSetEvent(Context as *mut c_void, IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);
    }
    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn USBSTOR_GetBusInterface(
    DeviceObject: PDEVICE_OBJECT,
    BusInterface: PUSB_BUS_INTERFACE_USBDI_V2,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: PIO_STACK_LOCATION;

    // ASSERT(DeviceObject);
    // ASSERT(BusInterface);

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0 as BOOLEAN);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatus),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // initialize request
    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_PNP as u8;
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE as u8;
    (*Stack).Parameters.QueryInterface.Size =
        core::mem::size_of::<BUS_INTERFACE_STANDARD>() as u16;
    (*Stack).Parameters.QueryInterface.InterfaceType =
        core::ptr::addr_of!(USB_BUS_INTERFACE_USBDI_GUID) as LPGUID;
    (*Stack).Parameters.QueryInterface.Version = 2 as u16;
    (*Stack).Parameters.QueryInterface.Interface = BusInterface as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_SyncUrbRequest(
    DeviceObject: PDEVICE_OBJECT,
    UrbRequest: PURB,
) -> NTSTATUS {

    const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003;

    let mut Irp: PIRP;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Irp = IoAllocateIrp((*DeviceObject).StackSize, 0);
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);

    IoStack = IoGetNextIrpStackLocation(Irp);

    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
    (*IoStack).Parameters.Others.Argument1 = UrbRequest as *mut core::ffi::c_void;
    (*IoStack).Parameters.DeviceIoControl.InputBufferLength =
        (*UrbRequest).__bindgen_anon_1.UrbHeader.as_ref().Length as u32;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
            PIO_COMPLETION_ROUTINE,
        >(USBSTOR_SyncForwardIrpCompletionRoutine),
        core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    Status = IoCallDriver(DeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i32 as i8,
            0,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    IoFreeIrp(Irp);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AllocateItem(mut PoolType: POOL_TYPE, mut ItemSize: ULONG) -> PVOID {
    const USB_STOR_TAG: u32 = 0x73627375; // 'sbsu'
    return ExAllocatePoolZero(PoolType, ItemSize as _, USB_STOR_TAG as ULONG);
}

#[no_mangle]
pub unsafe extern "C" fn FreeItem(mut Item: PVOID) {
    ExFreePoolWithTag(Item, USB_STOR_TAG as ULONG);
}

const URB_FUNCTION_CLASS_INTERFACE: u32 = 0x001B;
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_ClassRequest(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut RequestType: UCHAR,
    mut Index: USHORT,
    mut TransferFlags: ULONG,
    mut TransferBufferLength: ULONG,
    mut TransferBuffer: PVOID,
) -> NTSTATUS {
    let mut Urb: PURB;
    let mut Status: NTSTATUS;

    Urb = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as ULONG,
    ) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Urb)
        .__bindgen_anon_1
        .UrbControlVendorClassRequest
        .as_mut()
        .Hdr
        .Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    (*Urb)
        .__bindgen_anon_1
        .UrbControlVendorClassRequest
        .as_mut()
        .Hdr
        .Function = URB_FUNCTION_CLASS_INTERFACE as USHORT;
    (*Urb)
        .__bindgen_anon_1
        .UrbControlVendorClassRequest
        .as_mut()
        .TransferFlags = TransferFlags;
    (*Urb)
        .__bindgen_anon_1
        .UrbControlVendorClassRequest
        .as_mut()
        .TransferBufferLength = TransferBufferLength;
    (*Urb)
        .__bindgen_anon_1
        .UrbControlVendorClassRequest
        .as_mut()
        .TransferBuffer = TransferBuffer;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().Request = RequestType;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().Index = Index;

    Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);

    FreeItem(Urb as PVOID);
    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn USBSTOR_GetMaxLUN(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Buffer: PUCHAR;
    let mut Status: NTSTATUS;

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<UCHAR>() as SIZE_T,
        USB_STOR_TAG as ULONG,
    ) as PUCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = USBSTOR_ClassRequest(
        DeviceObject,
        DeviceExtension,
        USB_BULK_GET_MAX_LUN as UCHAR,
        (*(*DeviceExtension).InterfaceInformation).InterfaceNumber as USHORT,
        USBD_TRANSFER_DIRECTION_IN as ULONG,
        core::mem::size_of::<UCHAR>() as ULONG,
        Buffer as PVOID,
    );

    if NT_SUCCESS(Status) {
        if *Buffer as i32 > MAX_LUN {
            // invalid response documented in usb mass storage specification
            Status = STATUS_DEVICE_DATA_ERROR;
        } else {
            // store maxlun
            (*DeviceExtension).MaxLUN = *Buffer as UCHAR;
        }
    } else {
        // "USB Mass Storage Class. Bulk-Only Transport. Revision 1.0"
        // 3.2  Get Max LUN (class-specific request) :
        // Devices that do not support multiple LUNs may STALL this command.
        USBSTOR_ResetDevice((*DeviceExtension).LowerDeviceObject, DeviceExtension);

        (*DeviceExtension).MaxLUN = 0 as UCHAR;
        Status = STATUS_SUCCESS;
    }

    ExFreePoolWithTag(Buffer as PVOID, USB_STOR_TAG as ULONG);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_ResetDevice(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    Status = USBSTOR_ClassRequest(
        DeviceObject,
        DeviceExtension,
        USB_BULK_RESET_DEVICE as UCHAR,
        (*(*DeviceExtension).InterfaceInformation).InterfaceNumber as USHORT,
        USBD_TRANSFER_DIRECTION_OUT as ULONG,
        0 as ULONG,
        core::ptr::null_mut(),
    );
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Storage Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     USB block storage device driver.
//  * COPYRIGHT:   2005-2006 James Tabor
//  *              2011-2012 Michael Martin (michael.martin@reactos.org)
//  *              2011-2013 Johannes Anderwald (johannes.anderwald@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
// IO_COMPLETION_ROUTINE SyncForwardIrpCompletionRoutine;
//
// NTSTATUS
// NTAPI
// USBSTOR_SyncForwardIrpCompletionRoutine(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context)
// {
//     if (Irp->PendingReturned)
//     {
//         KeSetEvent((PKEVENT)Context, IO_NO_INCREMENT, FALSE);
//     }
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// USBSTOR_GetBusInterface(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PUSB_BUS_INTERFACE_USBDI_V2 BusInterface)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     PIRP Irp;
//     IO_STATUS_BLOCK IoStatus;
//     PIO_STACK_LOCATION Stack;
//
//     ASSERT(DeviceObject);
//     ASSERT(BusInterface);
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        DeviceObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatus);
//     if (Irp == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // initialize request
//     Stack = IoGetNextIrpStackLocation(Irp);
//     Stack->MajorFunction = IRP_MJ_PNP;
//     Stack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//     Stack->Parameters.QueryInterface.Size = sizeof(BUS_INTERFACE_STANDARD);
//     Stack->Parameters.QueryInterface.InterfaceType = (LPGUID)&USB_BUS_INTERFACE_USBDI_GUID;
//     Stack->Parameters.QueryInterface.Version = 2;
//     Stack->Parameters.QueryInterface.Interface = (PINTERFACE)BusInterface;
//     Stack->Parameters.QueryInterface.InterfaceSpecificData = NULL;
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatus.Status;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_SyncUrbRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PURB UrbRequest)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//     NTSTATUS Status;
//
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     // initialize stack location
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//     IoStack->Parameters.Others.Argument1 = (PVOID)UrbRequest;
//     IoStack->Parameters.DeviceIoControl.InputBufferLength = UrbRequest->UrbHeader.Length;
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     IoSetCompletionRoutine(Irp, USBSTOR_SyncForwardIrpCompletionRoutine, &Event, TRUE, TRUE, TRUE);
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = Irp->IoStatus.Status;
//     }
//
//     IoFreeIrp(Irp);
//     return Status;
// }
//
// PVOID
// AllocateItem(
//     IN POOL_TYPE PoolType,
//     IN ULONG ItemSize)
// {
//     return ExAllocatePoolZero(PoolType, ItemSize, USB_STOR_TAG);
// }
//
// VOID
// FreeItem(
//     IN PVOID Item)
// {
//     ExFreePoolWithTag(Item, USB_STOR_TAG);
// }
//
// NTSTATUS
// USBSTOR_ClassRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension,
//     IN UCHAR RequestType,
//     IN USHORT Index,
//     IN ULONG TransferFlags,
//     IN ULONG TransferBufferLength,
//     IN PVOID TransferBuffer)
//
// {
//     PURB Urb;
//     NTSTATUS Status;
//
//     Urb = (PURB)AllocateItem(NonPagedPool, sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST));
//     if (!Urb)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Urb->UrbControlVendorClassRequest.Hdr.Length = sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST);
//     Urb->UrbControlVendorClassRequest.Hdr.Function = URB_FUNCTION_CLASS_INTERFACE;
//     Urb->UrbControlVendorClassRequest.TransferFlags = TransferFlags;
//     Urb->UrbControlVendorClassRequest.TransferBufferLength = TransferBufferLength;
//     Urb->UrbControlVendorClassRequest.TransferBuffer = TransferBuffer;
//     Urb->UrbControlVendorClassRequest.Request = RequestType;
//     Urb->UrbControlVendorClassRequest.Index = Index;
//
//     Status = USBSTOR_SyncUrbRequest(DeviceObject, Urb);
//
//     FreeItem(Urb);
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_GetMaxLUN(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     PUCHAR Buffer;
//     NTSTATUS Status;
//
//     Buffer = (PUCHAR)ExAllocatePoolWithTag(NonPagedPool, sizeof(UCHAR), USB_STOR_TAG);
//     if (!Buffer)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = USBSTOR_ClassRequest(DeviceObject, DeviceExtension, USB_BULK_GET_MAX_LUN, DeviceExtension->InterfaceInformation->InterfaceNumber, USBD_TRANSFER_DIRECTION_IN, sizeof(UCHAR), Buffer);
//
//     DPRINT("MaxLUN: %x\n", *Buffer);
//
//     if (NT_SUCCESS(Status))
//     {
//         if (*Buffer > MAX_LUN)
//         {
//             // invalid response documented in usb mass storage specification
//             Status = STATUS_DEVICE_DATA_ERROR;
//         }
//         else
//         {
//             // store maxlun
//             DeviceExtension->MaxLUN = *Buffer;
//         }
//     }
//     else
//     {
//         // "USB Mass Storage Class. Bulk-Only Transport. Revision 1.0"
//         // 3.2  Get Max LUN (class-specific request) :
//         // Devices that do not support multiple LUNs may STALL this command.
//         USBSTOR_ResetDevice(DeviceExtension->LowerDeviceObject, DeviceExtension);
//
//         DeviceExtension->MaxLUN = 0;
//         Status = STATUS_SUCCESS;
//     }
//
//     ExFreePoolWithTag(Buffer, USB_STOR_TAG);
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_ResetDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//
//     Status = USBSTOR_ClassRequest(DeviceObject, DeviceExtension, USB_BULK_RESET_DEVICE, DeviceExtension->InterfaceInformation->InterfaceNumber, USBD_TRANSFER_DIRECTION_OUT, 0, NULL);
//     return Status;
// }
//
// // if somebody wants to add UFI support, here is a useful function
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================