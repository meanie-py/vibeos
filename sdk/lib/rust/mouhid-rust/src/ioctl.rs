// ioctl.rs — MouHid_InternalDeviceControl, MouHid_DeviceControl
// Translated from drivers/hid/mouhid/mouhid.c

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP,
};

use crate::types::PMOUHID_DEVICE_EXTENSION;
use crate::glue::*;

// ---------------------------------------------------------------------------
// MouHid_InternalDeviceControl — IRP_MJ_INTERNAL_DEVICE_CONTROL
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_InternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let mut IoStack = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;

    // DPRINT("[MOUHID] InternalDeviceControl %x\n", ...)

    let IoControlCode = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    match IoControlCode {
        IOCTL_MOUSE_QUERY_ATTRIBUTES => {
            // verify output buffer length
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                < core::mem::size_of::<ntkrnl_sys::ddk::MOUSE_ATTRIBUTES>() as u32
            {
                // invalid request
                // DPRINT1("[MOUHID] IOCTL_MOUSE_QUERY_ATTRIBUTES Buffer too small\n")
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                (*Irp).IoStatus.Information = 0;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_BUFFER_TOO_SMALL;
            }

            // get output buffer
            let Attributes: *mut ntkrnl_sys::ddk::MOUSE_ATTRIBUTES =
                (*Irp).AssociatedIrp.SystemBuffer as _;

            let ext = &*DeviceExtension;

            // type of mouse
            (*Attributes).MouseIdentifier = ext.MouseIdentifier;

            // number of buttons
            (*Attributes).NumberOfButtons = ext.UsageListLength;

            // sample rate not used for usb
            (*Attributes).SampleRate = 0;

            // queue length
            (*Attributes).InputDataQueueLength = 2;

            // DPRINT "[MOUHID] MouseIdentifier %x\n" etc.

            // complete request
            (*Irp).IoStatus.Information =
                core::mem::size_of::<ntkrnl_sys::ddk::MOUSE_ATTRIBUTES>() as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_SUCCESS
        }

        IOCTL_INTERNAL_MOUSE_CONNECT => {
            // verify input buffer length
            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
                < core::mem::size_of::<ntkrnl_sys::driver_ddk::CONNECT_DATA>() as u32
            {
                // invalid request
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                (*Irp).IoStatus.Information = 0;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }

            let ext = &mut *DeviceExtension;

            // is it already connected
            if !ext.ClassService.is_null() {
                // already connected
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SHARING_VIOLATION;
                (*Irp).IoStatus.Information = 0;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_SHARING_VIOLATION;
            }

            // get connect data
            let Data: *mut ntkrnl_sys::driver_ddk::CONNECT_DATA =
                (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as _;

            // store connect details
            ext.ClassDeviceObject = (*Data).ClassDeviceObject;
            ext.ClassService = (*Data).ClassService;

            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[MOUHID] mouse connected to class driver\n\0".as_ptr() as *const i8,
            );

            // completed successfully
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_SUCCESS
        }

        IOCTL_INTERNAL_MOUSE_DISCONNECT => {
            // not supported
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_NOT_IMPLEMENTED
        }

        IOCTL_INTERNAL_MOUSE_ENABLE => {
            // not supported
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_NOT_SUPPORTED
        }

        IOCTL_INTERNAL_MOUSE_DISABLE => {
            // not supported
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_INVALID_DEVICE_REQUEST
        }

        _ => {
            // DPRINT1("[MOUHID] Unknown DeviceControl %x\n", ...)
            // unknown request not supported
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_NOT_SUPPORTED
        }
    }
}

// ---------------------------------------------------------------------------
// MouHid_DeviceControl — IRP_MJ_DEVICE_CONTROL (pass-through)
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_DeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &*DeviceExtension;

    // skip stack location
    IoSkipCurrentIrpStackLocation(Irp);

    // pass and forget
    IoCallDriver(ext.NextDeviceObject, Irp)
}
