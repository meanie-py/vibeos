use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExAllocatePoolZero, _KWAIT_REASON_Executive, IoAllocateIrp, IoFreeIrp, IoGetNextIrpStackLocation, IoSetCompletionRoutine, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT};
use core::ffi::c_void;

// drivers/usb/usbccgp/usbccgp.h:40  #define USBCCPG_TAG 'cbsu'
// C multi-char constant 'cbsu' = ('c'<<24)|('b'<<16)|('s'<<8)|'u' = 0x63627375
const USBCCPG_TAG: u32 = 0x63627375;

// ---- file-scope preamble (translated) ----
// (No translatable declarations: #define NDEBUG is an object-like macro with no value, and the IO_COMPLETION_ROUTINE line is a function forward declaration, not a type/constant/table.)
#[no_mangle]
pub unsafe extern "system" fn USBSTOR_SyncForwardIrpCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    if (*Irp).PendingReturned != 0 {
        KeSetEvent(Context as PKEVENT as *mut core::ffi::c_void, IO_NO_INCREMENT as i32, 0);
    }
    return STATUS_MORE_PROCESSING_REQUIRED;
}
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as NTSTATUS;

const IOCTL_INTERNAL_USB_SUBMIT_URB: u32 = 0x00220003;

// USBCCGP_SyncUrbRequest — translated but dropped during assembly (its .responses entry came
// back body-less). Reconstructed faithfully from drivers/usb/usbccgp/misc.c:34. NOT NTAPI in
// the C (no NTAPI keyword) → cdecl (`_USBCCGP_SyncUrbRequest`), matching the fdo.rs/pdo.rs
// call-site declarations. Synchronously submits a URB down the stack via an internal IOCTL.
#[no_mangle]
pub unsafe extern "C" fn USBCCGP_SyncUrbRequest(
    DeviceObject: PDEVICE_OBJECT,
    UrbRequest: PURB,
) -> NTSTATUS {
    const IRP_MJ_INTERNAL_DEVICE_CONTROL: UCHAR = 0x0f;
    const STATUS_PENDING: NTSTATUS = 0x00000103;
    const KernelMode: i8 = 0;

    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();

    /* Allocate irp */
    let Irp: PIRP = IoAllocateIrp((*DeviceObject).StackSize, 0 as BOOLEAN);
    if Irp.is_null() {
        /* No memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Initialize event */
    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        0 as BOOLEAN,
    );

    /* Get next stack location */
    let IoStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);

    /* Initialize stack location */
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
    (*IoStack).Parameters.Others.Argument1 = UrbRequest as PVOID;
    (*IoStack).Parameters.DeviceIoControl.InputBufferLength =
        (*(*UrbRequest).__bindgen_anon_1.UrbHeader.as_ref()).Length as ULONG;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

    /* Setup completion routine */
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(
            USBSTOR_SyncForwardIrpCompletionRoutine as *const (),
        ),
        core::ptr::addr_of_mut!(Event) as PVOID,
        1 as BOOLEAN,
        1 as BOOLEAN,
        1 as BOOLEAN,
    );

    /* Call driver */
    Status = IoCallDriver(DeviceObject, Irp);

    /* Check if request is pending */
    if Status == STATUS_PENDING {
        /* Wait for completion */
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            KernelMode,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );
        /* Update status */
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    /* Free irp */
    IoFreeIrp(Irp);

    /* Done */
    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn AllocateItem(mut PoolType: POOL_TYPE, mut ItemSize: ULONG) -> PVOID {
    /* Allocate, zero and return item */
    return ExAllocatePoolZero(PoolType, ItemSize as _, USBCCPG_TAG as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn FreeItem(mut Item: PVOID) {
    /* Free item */
    ExFreePoolWithTag(Item, USBCCPG_TAG as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn DumpFunctionDescriptor(
    FunctionDescriptor: PUSBC_FUNCTION_DESCRIPTOR,
    FunctionDescriptorCount: ULONG,
) {
    let mut Index: ULONG;
    let mut SubIndex: ULONG;

    Index = 0;
    while Index < FunctionDescriptorCount {
        SubIndex = 0;
        while SubIndex < (*FunctionDescriptor.offset(Index as isize)).NumberOfInterfaces as ULONG {
            SubIndex = SubIndex.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Enhanced Host Controller Interface
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbccgp/misc.c
//  * PURPOSE:     USB  device driver.
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  *              Cameron Gutman
//  */
//
// #include "usbccgp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* Driver verifier */
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
// USBCCGP_SyncUrbRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT PURB UrbRequest)
// {
//     PIRP Irp;
//     PIO_STACK_LOCATION IoStack;
//     KEVENT Event;
//     NTSTATUS Status;
//
//     /* Allocate irp */
//     Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//     if (!Irp)
//     {
//         /* No memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Get next stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     /* Initialize stack location */
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//     IoStack->Parameters.Others.Argument1 = (PVOID)UrbRequest;
//     IoStack->Parameters.DeviceIoControl.InputBufferLength = UrbRequest->UrbHeader.Length;
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//
//     /* Setup completion routine */
//     IoSetCompletionRoutine(Irp,
//                            USBSTOR_SyncForwardIrpCompletionRoutine,
//                            &Event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     /* Call driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//
//     /* Check if request is pending */
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for completion */
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//
//         /* Update status */
//         Status = Irp->IoStatus.Status;
//     }
//
//     /* Free irp */
//     IoFreeIrp(Irp);
//
//     /* Done */
//     return Status;
// }
//
// PVOID
// AllocateItem(
//     IN POOL_TYPE PoolType,
//     IN ULONG ItemSize)
// {
//     /* Allocate, zero and return item */
//     return ExAllocatePoolZero(PoolType, ItemSize, USBCCPG_TAG);
// }
//
// VOID
// FreeItem(
//     IN PVOID Item)
// {
//     /* Free item */
//     ExFreePoolWithTag(Item, USBCCPG_TAG);
// }
//
// VOID
// DumpFunctionDescriptor(
//     IN PUSBC_FUNCTION_DESCRIPTOR FunctionDescriptor,
//     IN ULONG FunctionDescriptorCount)
// {
//     ULONG Index, SubIndex;
//
//     DPRINT("FunctionCount %lu\n", FunctionDescriptorCount);
//     for (Index = 0; Index < FunctionDescriptorCount; Index++)
//     {
//         DPRINT("Function %lu\n", Index);
//         DPRINT("FunctionNumber %lu\n", FunctionDescriptor[Index].FunctionNumber);
//         DPRINT("HardwareId %S\n", FunctionDescriptor[Index].HardwareId.Buffer);
//         DPRINT("CompatibleId %S\n", FunctionDescriptor[Index].CompatibleId.Buffer);
//         DPRINT("FunctionDescription %wZ\n", &FunctionDescriptor[Index].FunctionDescription);
//         DPRINT("NumInterfaces %lu\n", FunctionDescriptor[Index].NumberOfInterfaces);
//
//         for(SubIndex = 0; SubIndex < FunctionDescriptor[Index].NumberOfInterfaces; SubIndex++)
//         {
//             DPRINT(" Index %lu Interface %p\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]);
//             DPRINT(" Index %lu Interface InterfaceNumber %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bInterfaceNumber);
//             DPRINT(" Index %lu Interface Alternate %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bAlternateSetting );
//             DPRINT(" Index %lu bLength %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bLength);
//             DPRINT(" Index %lu bDescriptorType %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bDescriptorType);
//             DPRINT(" Index %lu bInterfaceNumber %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bInterfaceNumber);
//             DPRINT(" Index %lu bAlternateSetting %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bAlternateSetting);
//             DPRINT(" Index %lu bNumEndpoints %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bNumEndpoints);
//             DPRINT(" Index %lu bInterfaceClass %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bInterfaceClass);
//             DPRINT(" Index %lu bInterfaceSubClass %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bInterfaceSubClass);
//             DPRINT(" Index %lu bInterfaceProtocol %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->bInterfaceProtocol);
//             DPRINT(" Index %lu iInterface %x\n", SubIndex, FunctionDescriptor[Index].InterfaceDescriptorList[SubIndex]->iInterface);
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================