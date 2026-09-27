use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{_DEVICE_RELATION_TYPE_BusRelations as BusRelations, DEVICE_RELATIONS, INT32, IoAllocateWorkItem, IoCopyCurrentIrpStackLocationToNext, IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, IoStopTimer, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP};
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
}

// USB_STOR_TAG — pool tag, faithful to drivers/usb/usbstor/usbstor.h: 'sbsu'
// = (('s'<<24)|('b'<<16)|('s'<<8)|'u') = 0x73627375.
const USB_STOR_TAG: u32 = 0x73627375;

// IRP_MN_QUERY_CAPABILITIES — absent from driver_ddk's u8 IRP_MN set (lives in
// hal_ddk as u32). Faithful to <iotypes.h>: 0x09. Declared u8 to match the
// MinorFunction match scrutinee (the other IRP_MN_* arms are u8).
const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;

// ---- file-scope preamble (translated) ----
// #define NDEBUG
unsafe fn USBSTOR_DumpDeviceDescriptor(mut DeviceDescriptor: PUSB_DEVICE_DESCRIPTOR) {
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_FdoHandleDeviceRelations(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceCount: INT32 = 0 as INT32;
    let mut Index: LONG;
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    // FDO always only handles bus relations
    if (*IoStack).Parameters.QueryDeviceRelations.Type as u32 == BusRelations as u32 {
        // go through array and count device objects
        Index = 0 as LONG;
        while Index
            < (if (*DeviceExtension).MaxLUN as i32 > 1i32 {
                (*DeviceExtension).MaxLUN as i32
            } else {
                1i32
            }) as i64 as LONG
        {
            if !(*DeviceExtension).ChildPDO[Index as usize].is_null() {
                DeviceCount += 1;
            }
            Index += 1;
        }

        DeviceRelations = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<DEVICE_RELATIONS>() as SIZE_T).wrapping_add(
                ((DeviceCount as i32 - 1i32) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<PDEVICE_OBJECT>() as SIZE_T),
            ),
            USB_STOR_TAG as ULONG,
        ) as PDEVICE_RELATIONS;
        if DeviceRelations.is_null() {
            (*Irp).IoStatus.Information = 0 as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*DeviceRelations).Count = 0 as ULONG;

        // add device objects
        Index = 0 as LONG;
        while Index
            < (if (*DeviceExtension).MaxLUN as i32 > 1i32 {
                (*DeviceExtension).MaxLUN as i32
            } else {
                1i32
            }) as i64 as LONG
        {
            if !(*DeviceExtension).ChildPDO[Index as usize].is_null() {
                // store child pdo
                let fresh0 = (core::ptr::addr_of_mut!((*DeviceRelations).Objects)
                    as *mut PDEVICE_OBJECT)
                    .offset((*DeviceRelations).Count as isize);
                *fresh0 = (*DeviceExtension).ChildPDO[Index as usize];

                // add reference
                ObReferenceObject((*DeviceExtension).ChildPDO[Index as usize] as PVOID);

                (*DeviceRelations).Count = (*DeviceRelations).Count.wrapping_add(1);
            }
            Index += 1;
        }

        (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }

    IoCopyCurrentIrpStackLocationToNext(Irp);

    return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_FdoHandleRemoveDevice(
    DeviceObject: PDEVICE_OBJECT,
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Index: ULONG;

    // FIXME: wait for devices finished processing
    Index = 0 as ULONG;
    while Index < USB_MAXCHILDREN as ULONG {
        if !(*DeviceExtension).ChildPDO[Index as usize].is_null() {
            IoDeleteDevice((*DeviceExtension).ChildPDO[Index as usize]);
        }
        Index = Index.wrapping_add(1);
    }

    // Freeing everything in DeviceExtension
    if !(*DeviceExtension).DeviceDescriptor.is_null() {
        ExFreePoolWithTag(
            (*DeviceExtension).DeviceDescriptor as PVOID,
            USB_STOR_TAG as ULONG,
        );
    }
    if !(*DeviceExtension).ConfigurationDescriptor.is_null() {
        ExFreePoolWithTag(
            (*DeviceExtension).ConfigurationDescriptor as PVOID,
            USB_STOR_TAG as ULONG,
        );
    }
    if !(*DeviceExtension).InterfaceInformation.is_null() {
        ExFreePoolWithTag(
            (*DeviceExtension).InterfaceInformation as PVOID,
            USB_STOR_TAG as ULONG,
        );
    }
    if !(*DeviceExtension).ResetDeviceWorkItem.is_null() {
        IoFreeWorkItem((*DeviceExtension).ResetDeviceWorkItem);
    }
    if !(*DeviceExtension).SerialNumber.is_null() {
        ExFreePoolWithTag(
            (*DeviceExtension).SerialNumber as PVOID,
            USB_STOR_TAG as ULONG,
        );
    }

    // Send the IRP down the stack
    IoSkipCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    Status = IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);

    // Detach from the device stack
    IoDetachDevice((*DeviceExtension).LowerDeviceObject);

    IoDeleteDevice(DeviceObject);

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_FdoHandleStartDevice(
    mut DeviceObject: PDEVICE_OBJECT,
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut InterfaceDesc: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Index: UCHAR = 0 as UCHAR;
    let mut WorkItem: PIO_WORKITEM;

    // forward irp to lower device
    if IoForwardIrpSynchronously((*DeviceExtension).LowerDeviceObject, Irp) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if !NT_SUCCESS(Status) {
        DbgPrint(b"USBSTOR_FdoHandleStartDevice Lower device failed to start %x\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    if (*DeviceExtension).ResetDeviceWorkItem.is_null() {
        WorkItem = IoAllocateWorkItem(DeviceObject);
        (*DeviceExtension).ResetDeviceWorkItem = WorkItem;

        if WorkItem.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // initialize irp queue
    USBSTOR_QueueInitialize(DeviceExtension);

    // first get device & configuration & string descriptor
    Status = USBSTOR_GetDescriptors(DeviceObject);
    if !NT_SUCCESS(Status) {
        DbgPrint(b"USBSTOR_FdoHandleStartDevice failed to get device descriptor with %x\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    USBSTOR_DumpDeviceDescriptor((*DeviceExtension).DeviceDescriptor);

    // Check that this device uses bulk transfers and is SCSI
    InterfaceDesc = ((*DeviceExtension).ConfigurationDescriptor as usize)
        .wrapping_add(core::mem::size_of::<USB_CONFIGURATION_DESCRIPTOR>())
        as PUSB_INTERFACE_DESCRIPTOR;
    // ASSERT(InterfaceDesc->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE);
    if !((*InterfaceDesc).bDescriptorType as i32 == 0x4) {
        RtlAssert(
            b"InterfaceDesc->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE\0".as_ptr() as PVOID,
            b"fdo.c\0".as_ptr() as PVOID,
            198 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    // ASSERT(InterfaceDesc->bLength == sizeof(USB_INTERFACE_DESCRIPTOR));
    if !((*InterfaceDesc).bLength as usize == core::mem::size_of::<USB_INTERFACE_DESCRIPTOR>()) {
        RtlAssert(
            b"InterfaceDesc->bLength == sizeof(USB_INTERFACE_DESCRIPTOR)\0".as_ptr() as PVOID,
            b"fdo.c\0".as_ptr() as PVOID,
            199 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    // DPRINT("bInterfaceSubClass %x\n", InterfaceDesc->bInterfaceSubClass);
    if (*InterfaceDesc).bInterfaceProtocol as i32 != USB_PROTOCOL_BULK as i32 {
        DbgPrint(b"USB Device is not a bulk only device and is not currently supported\n\0".as_ptr() as *const i8);
        return STATUS_NOT_SUPPORTED;
    }

    if (*InterfaceDesc).bInterfaceSubClass as i32 == USB_SUBCLASS_UFI as i32 {
        DbgPrint(b"USB Floppy devices are not supported\n\0".as_ptr() as *const i8);
        return STATUS_NOT_SUPPORTED;
    }

    // now select an interface
    Status = USBSTOR_SelectConfigurationAndInterface(DeviceObject, DeviceExtension);
    if !NT_SUCCESS(Status) {
        DbgPrint(b"USBSTOR_FdoHandleStartDevice failed to select configuration / interface with %x\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    // check if we got a bulk in + bulk out endpoint
    Status = USBSTOR_GetPipeHandles(DeviceExtension);
    if !NT_SUCCESS(Status) {
        DbgPrint(b"USBSTOR_FdoHandleStartDevice no pipe handles %x\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    Status = USBSTOR_GetMaxLUN((*DeviceExtension).LowerDeviceObject, DeviceExtension);
    if !NT_SUCCESS(Status) {
        DbgPrint(b"USBSTOR_FdoHandleStartDevice failed to get max lun %x\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    // now create for each LUN a device object, 1 minimum
    loop {
        Status = USBSTOR_CreatePDO(DeviceObject, Index);

        if !NT_SUCCESS(Status) {
            DbgPrint(b"USBSTOR_FdoHandleStartDevice USBSTOR_CreatePDO failed for Index %lu with Status %x\n\0".as_ptr() as *const i8, Index as u32, Status as u32);
            return Status;
        }

        Index = Index.wrapping_add(1);
        (*DeviceExtension).InstanceCount = (*DeviceExtension).InstanceCount.wrapping_add(1);

        if !((Index as i32) < (*DeviceExtension).MaxLUN as i32) {
            break;
        }
    }

    // IoStartTimer(DeviceObject);

    // DPRINT("USBSTOR_FdoHandleStartDevice FDO is initialized\n");
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn USBSTOR_FdoHandlePnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION;
    let mut Status: NTSTATUS = 0;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    // ASSERT(DeviceExtension->Common.IsFDO);

    match (*IoStack).MinorFunction {
        IRP_MN_SURPRISE_REMOVAL => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            // forward irp to next device object
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            return USBSTOR_FdoHandleDeviceRelations(DeviceExtension, Irp);
        }
        IRP_MN_STOP_DEVICE => {
            IoStopTimer(DeviceObject);
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            // forward irp to next device object
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            return USBSTOR_FdoHandleRemoveDevice(DeviceObject, DeviceExtension, Irp);
        }
        IRP_MN_QUERY_CAPABILITIES => {
            // FIXME: set custom capabilities
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
        }
        IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
            if (*DeviceExtension).IrpPendingCount != 0 || !(*DeviceExtension).ActiveSrb.is_null() {
                /* We have pending requests */
                Status = STATUS_UNSUCCESSFUL;
            } else {
                /* We're all clear */
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

                IoSkipCurrentIrpStackLocation(Irp);
                return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
            }
        }
        IRP_MN_START_DEVICE => {
            Status = USBSTOR_FdoHandleStartDevice(DeviceObject, DeviceExtension, Irp);
        }
        _ => {
            // forward irp to next device object
            IoSkipCurrentIrpStackLocation(Irp);
            return IoCallDriver((*DeviceExtension).LowerDeviceObject, Irp);
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

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
//  *              2019 Victor Perevertkin (victor.perevertkin@reactos.org)
//  */
//
// #include "usbstor.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// static
// VOID
// USBSTOR_DumpDeviceDescriptor(PUSB_DEVICE_DESCRIPTOR DeviceDescriptor)
// {
//     DPRINT("Dumping Device Descriptor %p\n", DeviceDescriptor);
//     DPRINT("bLength %x\n", DeviceDescriptor->bLength);
//     DPRINT("bDescriptorType %x\n", DeviceDescriptor->bDescriptorType);
//     DPRINT("bcdUSB %x\n", DeviceDescriptor->bcdUSB);
//     DPRINT("bDeviceClass %x\n", DeviceDescriptor->bDeviceClass);
//     DPRINT("bDeviceSubClass %x\n", DeviceDescriptor->bDeviceSubClass);
//     DPRINT("bDeviceProtocol %x\n", DeviceDescriptor->bDeviceProtocol);
//     DPRINT("bMaxPacketSize0 %x\n", DeviceDescriptor->bMaxPacketSize0);
//     DPRINT("idVendor %x\n", DeviceDescriptor->idVendor);
//     DPRINT("idProduct %x\n", DeviceDescriptor->idProduct);
//     DPRINT("bcdDevice %x\n", DeviceDescriptor->bcdDevice);
//     DPRINT("iManufacturer %x\n", DeviceDescriptor->iManufacturer);
//     DPRINT("iProduct %x\n", DeviceDescriptor->iProduct);
//     DPRINT("iSerialNumber %x\n", DeviceDescriptor->iSerialNumber);
//     DPRINT("bNumConfigurations %x\n", DeviceDescriptor->bNumConfigurations);
// }
//
// NTSTATUS
// USBSTOR_FdoHandleDeviceRelations(
//     IN PFDO_DEVICE_EXTENSION DeviceExtension,
//     IN OUT PIRP Irp)
// {
//     INT32 DeviceCount = 0;
//     LONG Index;
//     PDEVICE_RELATIONS DeviceRelations;
//     PIO_STACK_LOCATION IoStack;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     // FDO always only handles bus relations
//     if (IoStack->Parameters.QueryDeviceRelations.Type == BusRelations)
//     {
//         // go through array and count device objects
//         for (Index = 0; Index < max(DeviceExtension->MaxLUN, 1); Index++)
//         {
//             if (DeviceExtension->ChildPDO[Index])
//             {
//                 DeviceCount++;
//             }
//         }
//
//         DeviceRelations = ExAllocatePoolWithTag(PagedPool, sizeof(DEVICE_RELATIONS) + (DeviceCount - 1) * sizeof(PDEVICE_OBJECT), USB_STOR_TAG);
//         if (!DeviceRelations)
//         {
//             Irp->IoStatus.Information = 0;
//             Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         DeviceRelations->Count = 0;
//
//         // add device objects
//         for (Index = 0; Index < max(DeviceExtension->MaxLUN, 1); Index++)
//         {
//             if (DeviceExtension->ChildPDO[Index])
//             {
//                 // store child pdo
//                 DeviceRelations->Objects[DeviceRelations->Count] = DeviceExtension->ChildPDO[Index];
//
//                 // add reference
//                 ObReferenceObject(DeviceExtension->ChildPDO[Index]);
//
//                 DeviceRelations->Count++;
//             }
//         }
//
//         Irp->IoStatus.Information = (ULONG_PTR)DeviceRelations;
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//     }
//
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//     return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
// }
//
// NTSTATUS
// USBSTOR_FdoHandleRemoveDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension,
//     IN OUT PIRP Irp)
// {
//     NTSTATUS Status;
//     ULONG Index;
//
//     DPRINT("Handling FDO removal %p\n", DeviceObject);
//
//     // FIXME: wait for devices finished processing
//     for (Index = 0; Index < USB_MAXCHILDREN; Index++)
//     {
//         if (DeviceExtension->ChildPDO[Index] != NULL)
//         {
//             DPRINT("Deleting PDO %p RefCount %x AttachedDevice %p \n", DeviceExtension->ChildPDO[Index], DeviceExtension->ChildPDO[Index]->ReferenceCount, DeviceExtension->ChildPDO[Index]->AttachedDevice);
//             IoDeleteDevice(DeviceExtension->ChildPDO[Index]);
//         }
//     }
//
//     // Freeing everything in DeviceExtension
//     if (DeviceExtension->DeviceDescriptor)
//         ExFreePoolWithTag(DeviceExtension->DeviceDescriptor, USB_STOR_TAG);
//     if (DeviceExtension->ConfigurationDescriptor)
//         ExFreePoolWithTag(DeviceExtension->ConfigurationDescriptor, USB_STOR_TAG);
//     if (DeviceExtension->InterfaceInformation)
//         ExFreePoolWithTag(DeviceExtension->InterfaceInformation, USB_STOR_TAG);
//     if (DeviceExtension->ResetDeviceWorkItem)
//         IoFreeWorkItem(DeviceExtension->ResetDeviceWorkItem);
//     if (DeviceExtension->SerialNumber)
//         ExFreePoolWithTag(DeviceExtension->SerialNumber, USB_STOR_TAG);
//
//     // Send the IRP down the stack
//     IoSkipCurrentIrpStackLocation(Irp);
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Status = IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//
//     // Detach from the device stack
//     IoDetachDevice(DeviceExtension->LowerDeviceObject);
//
//     IoDeleteDevice(DeviceObject);
//
//     return Status;
// }
//
// NTSTATUS
// USBSTOR_FdoHandleStartDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFDO_DEVICE_EXTENSION DeviceExtension,
//     IN OUT PIRP Irp)
// {
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDesc;
//     NTSTATUS Status;
//     UCHAR Index = 0;
//     PIO_WORKITEM WorkItem;
//
//     // forward irp to lower device
//     if (!IoForwardIrpSynchronously(DeviceExtension->LowerDeviceObject, Irp))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     Status = Irp->IoStatus.Status;
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBSTOR_FdoHandleStartDevice Lower device failed to start %x\n", Status);
//         return Status;
//     }
//
//     if (!DeviceExtension->ResetDeviceWorkItem)
//     {
//         WorkItem = IoAllocateWorkItem(DeviceObject);
//         DeviceExtension->ResetDeviceWorkItem = WorkItem;
//
//         if (!WorkItem)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     // initialize irp queue
//     USBSTOR_QueueInitialize(DeviceExtension);
//
//     // first get device & configuration & string descriptor
//     Status = USBSTOR_GetDescriptors(DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBSTOR_FdoHandleStartDevice failed to get device descriptor with %x\n", Status);
//         return Status;
//     }
//
//     USBSTOR_DumpDeviceDescriptor(DeviceExtension->DeviceDescriptor);
//
//     // Check that this device uses bulk transfers and is SCSI
//
//     InterfaceDesc = (PUSB_INTERFACE_DESCRIPTOR)((ULONG_PTR)DeviceExtension->ConfigurationDescriptor + sizeof(USB_CONFIGURATION_DESCRIPTOR));
//     ASSERT(InterfaceDesc->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE);
//     ASSERT(InterfaceDesc->bLength == sizeof(USB_INTERFACE_DESCRIPTOR));
//
//     DPRINT("bInterfaceSubClass %x\n", InterfaceDesc->bInterfaceSubClass);
//     if (InterfaceDesc->bInterfaceProtocol != USB_PROTOCOL_BULK)
//     {
//         DPRINT1("USB Device is not a bulk only device and is not currently supported\n");
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     if (InterfaceDesc->bInterfaceSubClass == USB_SUBCLASS_UFI)
//     {
//         DPRINT1("USB Floppy devices are not supported\n");
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     // now select an interface
//     Status = USBSTOR_SelectConfigurationAndInterface(DeviceObject, DeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         // failed to get device descriptor
//         DPRINT1("USBSTOR_FdoHandleStartDevice failed to select configuration / interface with %x\n", Status);
//         return Status;
//     }
//
//     // check if we got a bulk in + bulk out endpoint
//     Status = USBSTOR_GetPipeHandles(DeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBSTOR_FdoHandleStartDevice no pipe handles %x\n", Status);
//         return Status;
//     }
//
//     Status = USBSTOR_GetMaxLUN(DeviceExtension->LowerDeviceObject, DeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("USBSTOR_FdoHandleStartDevice failed to get max lun %x\n", Status);
//         return Status;
//     }
//
//     // now create for each LUN a device object, 1 minimum
//     do
//     {
//         Status = USBSTOR_CreatePDO(DeviceObject, Index);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("USBSTOR_FdoHandleStartDevice USBSTOR_CreatePDO failed for Index %lu with Status %x\n", Index, Status);
//             return Status;
//         }
//
//         Index++;
//         DeviceExtension->InstanceCount++;
//
//     } while(Index < DeviceExtension->MaxLUN);
//
//
//     //IoStartTimer(DeviceObject);
//
//     DPRINT("USBSTOR_FdoHandleStartDevice FDO is initialized\n");
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// USBSTOR_FdoHandlePnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN OUT PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(DeviceExtension->Common.IsFDO);
//
//     switch(IoStack->MinorFunction)
//     {
//         case IRP_MN_SURPRISE_REMOVAL:
//         {
//             DPRINT("IRP_MN_SURPRISE_REMOVAL %p\n", DeviceObject);
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             // forward irp to next device object
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//         {
//             DPRINT("IRP_MN_QUERY_DEVICE_RELATIONS %p Type: %u\n", DeviceObject, IoStack->Parameters.QueryDeviceRelations.Type);
//             return USBSTOR_FdoHandleDeviceRelations(DeviceExtension, Irp);
//         }
//         case IRP_MN_STOP_DEVICE:
//         {
//             DPRINT1("USBSTOR_FdoHandlePnp: IRP_MN_STOP_DEVICE unimplemented\n");
//             IoStopTimer(DeviceObject);
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//
//             // forward irp to next device object
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//         }
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             DPRINT("IRP_MN_REMOVE_DEVICE\n");
//
//             return USBSTOR_FdoHandleRemoveDevice(DeviceObject, DeviceExtension, Irp);
//         }
//         case IRP_MN_QUERY_CAPABILITIES:
//         {
//             // FIXME: set custom capabilities
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_STOP_DEVICE:
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//         {
//             if (DeviceExtension->IrpPendingCount != 0 || DeviceExtension->ActiveSrb != NULL)
//             {
//                 /* We have pending requests */
//                 DPRINT1("Failing removal/stop request due to pending requests present\n");
//                 Status = STATUS_UNSUCCESSFUL;
//             }
//             else
//             {
//                 /* We're all clear */
//                 Irp->IoStatus.Status = STATUS_SUCCESS;
//
//                 IoSkipCurrentIrpStackLocation(Irp);
//                 return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//             }
//             break;
//         }
//         case IRP_MN_START_DEVICE:
//         {
//             Status = USBSTOR_FdoHandleStartDevice(DeviceObject, DeviceExtension, Irp);
//             break;
//         }
//         default:
//         {
//             // forward irp to next device object
//             IoSkipCurrentIrpStackLocation(Irp);
//             return IoCallDriver(DeviceExtension->LowerDeviceObject, Irp);
//         }
//     }
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Status = Status;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================