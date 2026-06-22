use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PDRIVER_ADD_DEVICE, PDRIVER_DISPATCH, PDRIVER_UNLOAD, IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, KeInitializeSpinLock, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING};
// IoCallDriver / IoCompleteRequest arrive via the ntkrnl_sys::driver_ddk::* glob (above) —
// they are inline wrappers over the fastcall Iof* exports, same as fdo.rs/pdo.rs use them.
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
    // FDO_Dispatch (fdo.rs) / PDO_Dispatch (pdo.rs) — cross-module cdecl handlers.
    fn FDO_Dispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn PDO_Dispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// --- USBCCGP_CreateClose / USBCCGP_Dispatch / USBCCGP_Unload / DriverEntry were dropped by
// the translator (only USBCCGP_AddDevice survived to .rs; the rest came back as empty
// .partial files). Reconstructed faithfully from usbccgp.c (the verbatim C is preserved in
// the ORIGINAL C SOURCE comment block below). ---

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_CreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BBu32 as NTSTATUS;
    const IO_NO_INCREMENT: i8 = 0;

    /* Get common device extension */
    let DeviceExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    /* Is it a fdo */
    if (*DeviceExtension).IsFDO != 0 {
        /* Forward and forget */
        IoSkipCurrentIrpStackLocation(Irp);
        let FDODeviceExtension: PFDO_DEVICE_EXTENSION =
            (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
        /* Call lower driver */
        return IoCallDriver((*FDODeviceExtension).NextDeviceObject, Irp);
    } else {
        /* Pdo not supported */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_SUPPORTED;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_Dispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const IRP_MJ_CREATE: u8 = 0x00;
    const IRP_MJ_CLOSE: u8 = 0x02;

    /* Get common device extension */
    let DeviceExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    /* Get current stack location */
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).MajorFunction == IRP_MJ_CREATE || (*IoStack).MajorFunction == IRP_MJ_CLOSE {
        /* Dispatch to default handler */
        return USBCCGP_CreateClose(DeviceObject, Irp);
    }

    if (*DeviceExtension).IsFDO != 0 {
        /* Handle request for FDO */
        return FDO_Dispatch(DeviceObject, Irp);
    } else {
        /* Handle request for PDO */
        return PDO_Dispatch(DeviceObject, Irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_Unload(_DriverObject: PDRIVER_OBJECT) {
    /* DPRINT("[USBCCGP] Unload\n") — NDEBUG no-op */
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    const IRP_MJ_CREATE: usize = 0x00;
    const IRP_MJ_CLOSE: usize = 0x02;
    const IRP_MJ_DEVICE_CONTROL: usize = 0x0e;
    const IRP_MJ_INTERNAL_DEVICE_CONTROL: usize = 0x0f;
    const IRP_MJ_POWER: usize = 0x16;
    const IRP_MJ_PNP: usize = 0x1b;
    const IRP_MJ_SYSTEM_CONTROL: usize = 0x17;

    /* Initialize driver object */
    (*(*DriverObject).DriverExtension).AddDevice =
        core::mem::transmute::<_, PDRIVER_ADD_DEVICE>(USBCCGP_AddDevice as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_CREATE] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_POWER] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_PNP] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL] =
        core::mem::transmute::<_, PDRIVER_DISPATCH>(USBCCGP_Dispatch as *const ());
    (*DriverObject).DriverUnload =
        core::mem::transmute::<_, PDRIVER_UNLOAD>(USBCCGP_Unload as *const ());

    /* FIXME query GenericCompositeUSBDeviceString */
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn USBCCGP_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    const FILE_DEVICE_USB: u32 = 0x22;
    const STATUS_DEVICE_REMOVED: NTSTATUS = 0xC00002B6u32 as NTSTATUS;

    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let FDODeviceExtension: PFDO_DEVICE_EXTENSION;

    /* Lets create the device */
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as ULONG,
        core::ptr::null_mut(),
        FILE_DEVICE_USB,
        FILE_AUTOGENERATED_DEVICE_NAME,
        0 as BOOLEAN,
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        /* Failed to create device */
        DbgPrint(
            b"USBCCGP_AddDevice failed to create device with %x\n\0".as_ptr() as *const i8,
            Status,
        );
        return Status;
    }

    /* Get device extension */
    FDODeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Init device extension */
    RtlZeroMemory(
        FDODeviceExtension as *mut core::ffi::c_void,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>(),
    );
    (*FDODeviceExtension).Common.IsFDO = 1 as BOOLEAN;
    (*FDODeviceExtension).DriverObject = DriverObject;
    (*FDODeviceExtension).PhysicalDeviceObject = PhysicalDeviceObject;
    InitializeListHead(core::ptr::addr_of_mut!((*FDODeviceExtension).ResetPortListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*FDODeviceExtension).CyclePortListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*FDODeviceExtension).Lock));

    (*FDODeviceExtension).NextDeviceObject =
        IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
    if (*FDODeviceExtension).NextDeviceObject.is_null() {
        /* Failed to attach */
        DbgPrint(b"USBCCGP_AddDevice failed to attach device\n\0".as_ptr() as *const i8);
        IoDeleteDevice(DeviceObject);
        return STATUS_DEVICE_REMOVED;
    }

    /* Set device flags */
    (*DeviceObject).Flags |= (DO_BUFFERED_IO | DO_POWER_PAGABLE) as u32;

    /* Device is initialized */
    (*DeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as u32);

    /* Device initialized */
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Universal Serial Bus Bulk Enhanced Host Controller Interface
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/usb/usbccgp/usbccgp.c
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
// DRIVER_ADD_DEVICE USBCCGP_AddDevice;
//
// NTSTATUS
// NTAPI
// USBCCGP_AddDevice(
//     PDRIVER_OBJECT DriverObject,
//     PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     NTSTATUS Status;
//     PDEVICE_OBJECT DeviceObject;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Lets create the device */
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(FDO_DEVICE_EXTENSION),
//                             NULL,
//                             FILE_DEVICE_USB,
//                             FILE_AUTOGENERATED_DEVICE_NAME,
//                             FALSE,
//                             &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed to create device */
//         DPRINT1("USBCCGP_AddDevice failed to create device with %x\n", Status);
//         return Status;
//     }
//
//     /* Get device extension */
//     FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* Init device extension */
//     RtlZeroMemory(FDODeviceExtension, sizeof(FDO_DEVICE_EXTENSION));
//     FDODeviceExtension->Common.IsFDO = TRUE;
//     FDODeviceExtension->DriverObject = DriverObject;
//     FDODeviceExtension->PhysicalDeviceObject = PhysicalDeviceObject;
//     InitializeListHead(&FDODeviceExtension->ResetPortListHead);
//     InitializeListHead(&FDODeviceExtension->CyclePortListHead);
//     KeInitializeSpinLock(&FDODeviceExtension->Lock);
//
//     FDODeviceExtension->NextDeviceObject = IoAttachDeviceToDeviceStack(DeviceObject,
//                                                                        PhysicalDeviceObject);
//     if (!FDODeviceExtension->NextDeviceObject)
//     {
//         /* Failed to attach */
//         DPRINT1("USBCCGP_AddDevice failed to attach device\n");
//         IoDeleteDevice(DeviceObject);
//         return STATUS_DEVICE_REMOVED;
//     }
//
//     /* Set device flags */
//     DeviceObject->Flags |= DO_BUFFERED_IO | DO_POWER_PAGABLE;
//
//     /* Device is initialized */
//     DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     /* Device initialized */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBCCGP_CreateClose(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PCOMMON_DEVICE_EXTENSION DeviceExtension;
//     PFDO_DEVICE_EXTENSION FDODeviceExtension;
//
//     /* Get common device extension */
//     DeviceExtension = (PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* Is it a fdo */
//     if (DeviceExtension->IsFDO)
//     {
//         /* Forward and forget */
//         IoSkipCurrentIrpStackLocation(Irp);
//
//         /* Get fdo */
//         FDODeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//         /* Call lower driver */
//         return IoCallDriver(FDODeviceExtension->NextDeviceObject, Irp);
//     }
//     else
//     {
//         /* Pdo not supported */
//         Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_NOT_SUPPORTED;
//     }
// }
//
// NTSTATUS
// NTAPI
// USBCCGP_Dispatch(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PCOMMON_DEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION IoStack;
//
//     /* Get common device extension */
//     DeviceExtension = (PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* Get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->MajorFunction == IRP_MJ_CREATE || IoStack->MajorFunction == IRP_MJ_CLOSE)
//     {
//         /* Dispatch to default handler */
//         return USBCCGP_CreateClose(DeviceObject, Irp);
//     }
//
//     if (DeviceExtension->IsFDO)
//     {
//         /* Handle request for FDO */
//         return FDO_Dispatch(DeviceObject, Irp);
//     }
//     else
//     {
//         /* Handle request for PDO */
//         return PDO_Dispatch(DeviceObject, Irp);
//     }
// }
//
// VOID
// NTAPI
// USBCCGP_Unload(PDRIVER_OBJECT DriverObject)
// {
//     DPRINT("[USBCCGP] Unload\n");
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(
//     PDRIVER_OBJECT DriverObject,
//     PUNICODE_STRING RegistryPath)
// {
//
//     /* Initialize driver object */
//     DPRINT("[USBCCGP] DriverEntry\n");
//     DriverObject->DriverExtension->AddDevice = USBCCGP_AddDevice;
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_PNP] = USBCCGP_Dispatch;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = USBCCGP_Dispatch;
//     DriverObject->DriverUnload = USBCCGP_Unload;
//
//     /* FIMXE query GenericCompositeUSBDeviceString */
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================