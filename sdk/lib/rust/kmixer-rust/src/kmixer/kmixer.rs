// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_A76C06629BFA: [u16; 15] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x006B, 0x006D,
    0x0069, 0x0078, 0x0065, 0x0072, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn KMix_Pnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut IrpStack: PIO_STACK_LOCATION;

    IrpStack = IoGetCurrentIrpStackLocation(Irp);

    DPRINT!("KMix_Pnp called for func %x\n", (*IrpStack).MinorFunction);

    if (*IrpStack).MinorFunction == IRP_MN_QUERY_PNP_DEVICE_STATE {
        (*Irp).IoStatus.Information |= (PNP_DEVICE_NOT_DISABLEABLE as _);
    }

    return KsDefaultDispatchPnp(DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn KMix_Unload(DriverObject: PDRIVER_OBJECT) {
    DPRINT1!("SysAudio_Unload called\n");
}

#[no_mangle]
pub unsafe extern "system" fn KMix_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_A76C06629BFA.as_ptr());
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceExtension: PKMIXER_DEVICE_EXT = core::ptr::null_mut();
    let mut NextDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    DPRINT1!("KMix_InstallDevice called\n");

    /* create the device */
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<KMIXER_DEVICE_EXT>() as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_KS,
        0,
        FALSE,
        core::ptr::addr_of_mut!(DeviceObject),
    );

    /* check for success */
    if !NT_SUCCESS!(Status) {
        DPRINT!("Failed to create \\Device\\kmixer !\n");
        return Status;
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PKMIXER_DEVICE_EXT;
    /* initialize device extension */
    RtlZeroMemory!(
        DeviceExtension as *mut core::ffi::c_void,
        core::mem::size_of::<KMIXER_DEVICE_EXT>() as SIZE_T,
    );

    'cleanup: {
        Status = KMixAllocateDeviceHeader(DeviceExtension);
        if !NT_SUCCESS!(Status) {
            DPRINT1!("KMixAllocateDeviceHeader failed with %x\n", Status);
            break 'cleanup;
        }

        /* set io flags */
        (*DeviceObject).Flags |= DO_DIRECT_IO | DO_POWER_PAGABLE;

        /* clear initializing flag */
        (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

        NextDeviceObject = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
        KsSetDevicePnpAndBaseObject(
            (*DeviceExtension).KsDeviceHeader,
            NextDeviceObject,
            DeviceObject,
        );

        DPRINT!("KMix_InstallDevice result %x\n", Status);
        return STATUS_SUCCESS;
    }

    /* cleanup: */
    IoDeleteDevice(DeviceObject);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPathName: PUNICODE_STRING,
) -> NTSTATUS {
    DPRINT1!("KMixer.sys loaded\n");

    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CREATE);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_CLOSE);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_WRITE);
    KsSetMajorFunctionHandler(DriverObject, IRP_MJ_DEVICE_CONTROL);

    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        Some(KsDefaultDispatchPower);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        Some(KsDefaultForwardIrp);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        Some(KMix_Pnp);
    (*DriverObject).DriverUnload =
        Some(KMix_Unload);
    (*(*DriverObject).DriverExtension).AddDevice =
        Some(KMix_AddDevice);

    return STATUS_SUCCESS;
}
