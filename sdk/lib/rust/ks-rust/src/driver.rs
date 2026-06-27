use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoAllocateDriverObjectExtension, IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoGetDriverObjectExtension, PDEVICE_OBJECT, PDRIVER_OBJECT, PUNICODE_STRING, PowerDeviceD0, PowerSystemWorking};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 0;

pub type PKSDEVICE = *mut KSDEVICE;
pub type KSDEVICE = core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn KsGetDeviceForDeviceObject(
    FunctionalDeviceObject: PDEVICE_OBJECT,
) -> PKSDEVICE {
    let DeviceExtension: PDEVICE_EXTENSION;
    DeviceExtension = (*FunctionalDeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    return &mut (*(*DeviceExtension).DeviceHeader).KsDevice;
}

#[no_mangle]
pub unsafe extern "system" fn KsGetDevice(Object: PVOID) -> PKSDEVICE {
    let BasicHeader: PKSBASIC_HEADER = (Object as usize)
        .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
        as PKSBASIC_HEADER;

    // DPRINT removed

    if !((*BasicHeader).Type as u32 == KsObjectTypeFilterFactory as u32
        || (*BasicHeader).Type as u32 == KsObjectTypeFilter as u32
        || (*BasicHeader).Type as u32 == KsObjectTypePin as u32)
    {
        RtlAssert(
            b"BasicHeader->Type == KsObjectTypeFilterFactory || BasicHeader->Type == KsObjectTypeFilter || BasicHeader->Type == KsObjectTypePin\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            44u32,
            core::ptr::null_mut(),
        );
    }
    if (*BasicHeader).KsDevice.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            45u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).Descriptor.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->Descriptor\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            46u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).Bag.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->Bag\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            47u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).Context.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->Context\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            48u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).FunctionalDeviceObject.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->FunctionalDeviceObject\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            49u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).PhysicalDeviceObject.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->PhysicalDeviceObject\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            50u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).NextDeviceObject.is_null() {
        RtlAssert(
            b"BasicHeader->KsDevice->NextDeviceObject\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            51u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).Started == 0 {
        RtlAssert(
            b"BasicHeader->KsDevice->Started\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            52u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).SystemPowerState as u32 != PowerSystemWorking as u32 {
        RtlAssert(
            b"BasicHeader->KsDevice->SystemPowerState == PowerSystemWorking\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            53u32,
            core::ptr::null_mut(),
        );
    }
    if (*(*BasicHeader).KsDevice).DevicePowerState as u32 != PowerDeviceD0 as u32 {
        RtlAssert(
            b"BasicHeader->KsDevice->DevicePowerState == PowerDeviceD0\0".as_ptr() as PCHAR,
            b"drivers/ksfilter/ks/driver.c\0".as_ptr() as PCHAR,
            54u32,
            core::ptr::null_mut(),
        );
    }

    (*BasicHeader).KsDevice
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    Descriptor: *const KSDEVICE_DESCRIPTOR,
    ExtensionSize: ULONG,
    Device: *mut PKSDEVICE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_DEVICE_REMOVED;
    let mut FunctionalDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut OldHighestDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut ExtensionSize = ExtensionSize;
    if ExtensionSize == 0 {
        ExtensionSize = core::mem::size_of::<KSDEVICE_HEADER>() as ULONG;
    }

    Status = IoCreateDevice(
        DriverObject,
        ExtensionSize,
        core::ptr::null_mut(),
        FILE_DEVICE_KS as ULONG,
        FILE_DEVICE_SECURE_OPEN as ULONG,
        0u8,
        &mut FunctionalDeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    OldHighestDeviceObject = IoAttachDeviceToDeviceStack(FunctionalDeviceObject, PhysicalDeviceObject);
    if !OldHighestDeviceObject.is_null() {
        Status = KsInitializeDevice(
            FunctionalDeviceObject,
            PhysicalDeviceObject,
            OldHighestDeviceObject,
            Descriptor,
        );
    } else {
        Status = STATUS_DEVICE_REMOVED;
    }

    if !NT_SUCCESS(Status) {
        if !OldHighestDeviceObject.is_null() {
            IoDetachDevice(OldHighestDeviceObject);
        }
        IoDeleteDevice(FunctionalDeviceObject);
        return Status;
    }

    (*FunctionalDeviceObject).Flags |= (DO_DIRECT_IO | DO_POWER_PAGABLE) as ULONG;
    (*FunctionalDeviceObject).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);

    if !Device.is_null() {
        *Device = KsGetDeviceForDeviceObject(FunctionalDeviceObject);
        if ExtensionSize as usize > core::mem::size_of::<KSDEVICE_HEADER>() as usize {
            (**Device).Context = ((*FunctionalDeviceObject).DeviceExtension as usize)
                .wrapping_add(core::mem::size_of::<KSDEVICE_HEADER>() as usize) as PVOID;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DriverObjectExtension: PKS_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut Descriptor: *const KSDEVICE_DESCRIPTOR = core::ptr::null();

    DriverObjectExtension = IoGetDriverObjectExtension(
        DriverObject,
        KsInitializeDriver as PVOID,
    ) as PKS_DRIVER_EXTENSION;

    if !DriverObjectExtension.is_null() {
        Descriptor = (*DriverObjectExtension).Descriptor;
    }

    KsCreateDevice(
        DriverObject,
        PhysicalDeviceObject,
        Descriptor,
        0,
        core::ptr::null_mut(),
    )
}
#[no_mangle]
pub unsafe extern "system" fn KsInitializeDriver(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
    Descriptor: *const KSDEVICE_DESCRIPTOR,
) -> NTSTATUS {
    let mut DriverObjectExtension: PKS_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if !Descriptor.is_null() {
        Status = IoAllocateDriverObjectExtension(
            DriverObject,
            KsInitializeDriver as PVOID,
            core::mem::size_of::<KS_DRIVER_EXTENSION>() as ULONG,
            &mut DriverObjectExtension as *mut PKS_DRIVER_EXTENSION as PVOID,
        );
        if NT_SUCCESS(Status) {
            (*DriverObjectExtension).Descriptor = Descriptor;
        }
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(IKsDevice_Create);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(IKsDevice_Pnp);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(IKsDevice_Power);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = Some(KsDefaultForwardIrp);

    (*DriverObject).DriverUnload = Some(KsNullDriverUnload);
    (*(*DriverObject).DriverExtension).AddDevice = Some(KsAddDevice);

    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CLOSE);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_DEVICE_CONTROL);

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/driver.c
//  * PURPOSE:         KS Driver functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSDEVICE
// NTAPI
// KsGetDeviceForDeviceObject(
//     IN PDEVICE_OBJECT FunctionalDeviceObject)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)FunctionalDeviceObject->DeviceExtension;
//
//     return &DeviceExtension->DeviceHeader->KsDevice;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSDEVICE
// NTAPI
// KsGetDevice(
//     IN PVOID Object)
// {
//     PKSBASIC_HEADER BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)Object - sizeof(KSBASIC_HEADER));
//
//     DPRINT("KsGetDevice Type %lu KsDevice %p\n", BasicHeader->Type, BasicHeader->KsDevice);
//
//     ASSERT(BasicHeader->Type == KsObjectTypeFilterFactory || BasicHeader->Type == KsObjectTypeFilter || BasicHeader->Type == KsObjectTypePin);
//     ASSERT(BasicHeader->KsDevice);
//     ASSERT(BasicHeader->KsDevice->Descriptor);
//     ASSERT(BasicHeader->KsDevice->Bag);
//     ASSERT(BasicHeader->KsDevice->Context);
//     ASSERT(BasicHeader->KsDevice->FunctionalDeviceObject);
//     ASSERT(BasicHeader->KsDevice->PhysicalDeviceObject);
//     ASSERT(BasicHeader->KsDevice->NextDeviceObject);
//     ASSERT(BasicHeader->KsDevice->Started);
//     ASSERT(BasicHeader->KsDevice->SystemPowerState == PowerSystemWorking);
//     ASSERT(BasicHeader->KsDevice->DevicePowerState == PowerDeviceD0);
//
//     return BasicHeader->KsDevice;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsCreateDevice(
//     IN  PDRIVER_OBJECT DriverObject,
//     IN  PDEVICE_OBJECT PhysicalDeviceObject,
//     IN  const KSDEVICE_DESCRIPTOR* Descriptor OPTIONAL,
//     IN  ULONG ExtensionSize OPTIONAL,
//     OUT PKSDEVICE* Device OPTIONAL)
// {
//     NTSTATUS Status = STATUS_DEVICE_REMOVED;
//     PDEVICE_OBJECT FunctionalDeviceObject= NULL;
//     PDEVICE_OBJECT OldHighestDeviceObject;
//     if (!ExtensionSize)
//         ExtensionSize = sizeof(KSDEVICE_HEADER);
//
//     DPRINT("KsCreateDevice Descriptor %p ExtensionSize %lu\n", Descriptor, ExtensionSize);
//
//     Status = IoCreateDevice(DriverObject, ExtensionSize, NULL, FILE_DEVICE_KS, FILE_DEVICE_SECURE_OPEN, FALSE, &FunctionalDeviceObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     OldHighestDeviceObject = IoAttachDeviceToDeviceStack(FunctionalDeviceObject, PhysicalDeviceObject);
//     if (OldHighestDeviceObject)
//     {
//         Status = KsInitializeDevice(FunctionalDeviceObject, PhysicalDeviceObject, OldHighestDeviceObject, Descriptor);
//     }
//     else
//     {
//         Status = STATUS_DEVICE_REMOVED;
//     }
//
//     /* check if all succeeded */
//     if (!NT_SUCCESS(Status))
//     {
//         if (OldHighestDeviceObject)
//             IoDetachDevice(OldHighestDeviceObject);
//
//         IoDeleteDevice(FunctionalDeviceObject);
//         return Status;
//     }
//
//     /* set device flags */
//     FunctionalDeviceObject->Flags |= DO_DIRECT_IO | DO_POWER_PAGABLE;
//     FunctionalDeviceObject->Flags &= ~ DO_DEVICE_INITIALIZING;
//
//     if (Device)
//     {
//         /* get PKSDEVICE struct */
//         *Device = KsGetDeviceForDeviceObject(FunctionalDeviceObject);
//
//         if (ExtensionSize > sizeof(KSDEVICE_HEADER))
//         {
//             /* caller needs a device extension */
//             (*Device)->Context = (PVOID)((ULONG_PTR)FunctionalDeviceObject->DeviceExtension + sizeof(KSDEVICE_HEADER));
//         }
//     }
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsAddDevice(
//     IN  PDRIVER_OBJECT DriverObject,
//     IN  PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     PKS_DRIVER_EXTENSION DriverObjectExtension;
//     const KSDEVICE_DESCRIPTOR *Descriptor = NULL;
//
//     /* get stored driver object extension */
//
//     DriverObjectExtension = IoGetDriverObjectExtension(DriverObject, (PVOID)KsInitializeDriver);
//
//     if (DriverObjectExtension)
//     {
//         /* get the stored descriptor see KsInitializeDriver */
//         Descriptor = DriverObjectExtension->Descriptor;
//     }
//
//     return KsCreateDevice(DriverObject, PhysicalDeviceObject, Descriptor, 0, NULL);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsInitializeDriver(
//     IN PDRIVER_OBJECT  DriverObject,
//     IN PUNICODE_STRING  RegistryPath,
//     IN const KSDEVICE_DESCRIPTOR  *Descriptor OPTIONAL
// )
// {
//     PKS_DRIVER_EXTENSION DriverObjectExtension;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("KsInitializeDriver\n");
//
//     if (Descriptor)
//     {
//         Status = IoAllocateDriverObjectExtension(DriverObject, (PVOID)KsInitializeDriver, sizeof(KS_DRIVER_EXTENSION), (PVOID*)&DriverObjectExtension);
//         if (NT_SUCCESS(Status))
//         {
//             DriverObjectExtension->Descriptor = Descriptor;
//         }
//     }
//
//     /* sanity check */
//     ASSERT(Status == STATUS_SUCCESS);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Setting our IRP handlers */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = IKsDevice_Create;
//     DriverObject->MajorFunction[IRP_MJ_PNP] = IKsDevice_Pnp;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = IKsDevice_Power;
//     DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = KsDefaultForwardIrp;
//
//     /* The driver unload routine */
//     DriverObject->DriverUnload = KsNullDriverUnload;
//
//     /* The driver-supplied AddDevice */
//     DriverObject->DriverExtension->AddDevice = KsAddDevice;
//
//     /* KS handles these */
//     DPRINT1("KsInitializeDriver Setting KS function handlers\n");
//     KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CLOSE);
//     KsSetMajorFunctionHandler(DriverObject, IRP_MJ_DEVICE_CONTROL);
//
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
