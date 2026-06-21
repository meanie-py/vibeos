// ioctl.rs — KbdHid_InternalDeviceControl, KbdHid_DeviceControl.
// Translated from drivers/hid/kbdhid/kbdhid.c

use core::mem::size_of;
use ntkrnl_sys::base::{NTSTATUS, PVOID};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP,
    KEYBOARD_ATTRIBUTES, KEYBOARD_INDICATOR_PARAMETERS, KEYBOARD_TYPEMATIC_PARAMETERS,
};
use ntkrnl_sys::constants::{
    STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER,
    STATUS_NOT_SUPPORTED, STATUS_SHARING_VIOLATION, IO_NO_INCREMENT,
};
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::driver_ddk::{IoCompleteRequest, IoCallDriver, CONNECT_DATA};
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use crate::types::{
    PKBDHID_DEVICE_EXTENSION,
    LOCAL_KEYBOARD_INDICATOR_TRANSLATION, INDICATOR_LIST,
};
use crate::glue::{
    IOCTL_KEYBOARD_QUERY_ATTRIBUTES,
    IOCTL_INTERNAL_KEYBOARD_CONNECT,
    IOCTL_INTERNAL_KEYBOARD_DISCONNECT,
    IOCTL_INTERNAL_KEYBOARD_ENABLE,
    IOCTL_INTERNAL_KEYBOARD_DISABLE,
    IOCTL_KEYBOARD_QUERY_INDICATORS,
    IOCTL_KEYBOARD_QUERY_TYPEMATIC,
    IOCTL_KEYBOARD_SET_INDICATORS,
    IOCTL_KEYBOARD_SET_TYPEMATIC,
    IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION,
    KEYBOARD_CAPS_LOCK_ON, KEYBOARD_NUM_LOCK_ON, KEYBOARD_SCROLL_LOCK_ON,
};

// ============================================================================
// Static indicator translation table — file-scope global in kbdhid.c.
// "This structure starts with the same layout as KEYBOARD_INDICATOR_TRANSLATION"
// C: static LOCAL_KEYBOARD_INDICATOR_TRANSLATION IndicatorTranslation = { 3, {
//    {0x3A, KEYBOARD_CAPS_LOCK_ON}, {0x45, KEYBOARD_NUM_LOCK_ON},
//    {0x46, KEYBOARD_SCROLL_LOCK_ON}}};
// ============================================================================
static INDICATOR_TRANSLATION: LOCAL_KEYBOARD_INDICATOR_TRANSLATION =
    LOCAL_KEYBOARD_INDICATOR_TRANSLATION {
        NumberOfIndicatorKeys: 3,
        IndicatorList: [
            INDICATOR_LIST { MakeCode: 0x3A, IndicatorFlags: KEYBOARD_CAPS_LOCK_ON },
            INDICATOR_LIST { MakeCode: 0x45, IndicatorFlags: KEYBOARD_NUM_LOCK_ON },
            INDICATOR_LIST { MakeCode: 0x46, IndicatorFlags: KEYBOARD_SCROLL_LOCK_ON },
        ],
    };

/// KbdHid_InternalDeviceControl — IRP_MJ_INTERNAL_DEVICE_CONTROL dispatch.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_InternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    /* get current stack location */
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* get device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    let ioctl = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if ioctl == IOCTL_KEYBOARD_QUERY_ATTRIBUTES {
        /* verify output buffer length */
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < size_of::<KEYBOARD_ATTRIBUTES>() as u32
        {
            /* invalid request */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_BUFFER_TOO_SMALL;
        }

        /* get output buffer */
        let Attributes = (*Irp).AssociatedIrp.SystemBuffer as *mut KEYBOARD_ATTRIBUTES;

        /* copy attributes */
        RtlCopyMemory(
            Attributes as PVOID,
            &(*DeviceExtension).Attributes as *const _ as *const _,
            size_of::<KEYBOARD_ATTRIBUTES>(),
        );

        /* complete request */
        (*Irp).IoStatus.Information = size_of::<KEYBOARD_ATTRIBUTES>() as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if ioctl == IOCTL_INTERNAL_KEYBOARD_CONNECT {
        /* verify input buffer length */
        if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
            < size_of::<CONNECT_DATA>() as u32
        {
            /* invalid request */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INVALID_PARAMETER;
        }

        /* is it already connected */
        if !(*DeviceExtension).ClassService.is_null() {
            /* already connected */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SHARING_VIOLATION;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_SHARING_VIOLATION;
        }

        /* get connect data */
        let Data = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as *mut CONNECT_DATA;

        /* store connect details */
        (*DeviceExtension).ClassDeviceObject = (*Data).ClassDeviceObject;
        (*DeviceExtension).ClassService = (*Data).ClassService;

        /* completed successfully */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if ioctl == IOCTL_INTERNAL_KEYBOARD_DISCONNECT {
        /* not implemented */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_IMPLEMENTED;
    }

    if ioctl == IOCTL_INTERNAL_KEYBOARD_ENABLE {
        /* not supported */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_SUPPORTED;
    }

    if ioctl == IOCTL_INTERNAL_KEYBOARD_DISABLE {
        /* not supported */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_SUPPORTED;
    }

    if ioctl == IOCTL_KEYBOARD_QUERY_INDICATORS {
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < size_of::<KEYBOARD_INDICATOR_PARAMETERS>() as u32
        {
            /* buffer too small */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_BUFFER_TOO_SMALL;
        }

        /* copy indicators */
        RtlCopyMemory(
            (*Irp).AssociatedIrp.SystemBuffer as PVOID,
            &(*DeviceExtension).KeyboardIndicator as *const _ as *const _,
            size_of::<KEYBOARD_INDICATOR_PARAMETERS>(),
        );

        /* complete request */
        // NOTE: C source faithfully returns STATUS_NOT_IMPLEMENTED here (preserved bug)
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = size_of::<KEYBOARD_INDICATOR_PARAMETERS>() as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_IMPLEMENTED;
    }

    if ioctl == IOCTL_KEYBOARD_QUERY_TYPEMATIC {
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < size_of::<KEYBOARD_TYPEMATIC_PARAMETERS>() as u32
        {
            /* buffer too small */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_BUFFER_TOO_SMALL;
        }

        /* copy indicators */
        RtlCopyMemory(
            (*Irp).AssociatedIrp.SystemBuffer as PVOID,
            &(*DeviceExtension).KeyboardTypematic as *const _ as *const _,
            size_of::<KEYBOARD_TYPEMATIC_PARAMETERS>(),
        );

        /* done */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = size_of::<KEYBOARD_TYPEMATIC_PARAMETERS>() as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if ioctl == IOCTL_KEYBOARD_SET_INDICATORS {
        if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
            < size_of::<KEYBOARD_INDICATOR_PARAMETERS>() as u32
        {
            /* invalid parameter */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INVALID_PARAMETER;
        }

        /* copy indicators */
        RtlCopyMemory(
            &mut (*DeviceExtension).KeyboardIndicator as *mut _ as PVOID,
            (*Irp).AssociatedIrp.SystemBuffer as *const _,
            size_of::<KEYBOARD_INDICATOR_PARAMETERS>(),
        );

        /* done */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if ioctl == IOCTL_KEYBOARD_SET_TYPEMATIC {
        if (*IoStack).Parameters.DeviceIoControl.InputBufferLength
            < size_of::<KEYBOARD_TYPEMATIC_PARAMETERS>() as u32
        {
            /* invalid parameter */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INVALID_PARAMETER;
        }

        /* copy indicators */
        RtlCopyMemory(
            &mut (*DeviceExtension).KeyboardTypematic as *mut _ as PVOID,
            (*Irp).AssociatedIrp.SystemBuffer as *const _,
            size_of::<KEYBOARD_TYPEMATIC_PARAMETERS>(),
        );

        /* done */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    if ioctl == IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION {
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>() as u32
        {
            /* buffer too small */
            // NOTE: C faithfully returns STATUS_INVALID_PARAMETER here despite Status = TOO_SMALL
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            (*Irp).IoStatus.Information = 0;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INVALID_PARAMETER;
        }

        /* copy translations */
        RtlCopyMemory(
            (*Irp).AssociatedIrp.SystemBuffer as PVOID,
            &INDICATOR_TRANSLATION as *const _ as *const _,
            size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>(),
        );

        /* done */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>() as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    /* unknown control code */
    /* unknown request not supported */
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_NOT_SUPPORTED
}

/// KbdHid_DeviceControl — IRP_MJ_DEVICE_CONTROL: pass through to lower driver.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_DeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    /* get device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    /* skip stack location */
    IoSkipCurrentIrpStackLocation(Irp);

    /* pass and forget */
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}
