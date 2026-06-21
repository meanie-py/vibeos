// pnp.rs — KbdHid_Pnp, KbdHid_StartDevice, KbdHid_StartDeviceCompletion,
//           KbdHid_FreeResources, KbdHid_SubmitRequest.
// Translated from drivers/hid/kbdhid/kbdhid.c

use core::mem::size_of;
use ntkrnl_sys::base::{NTSTATUS, PVOID, BOOLEAN, TRUE, FALSE};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, PIO_COMPLETION_ROUTINE,
    KEVENT, IO_STATUS_BLOCK,
    IoBuildDeviceIoControlRequest,
    IoAllocateMdl, IoFreeMdl, IoCancelIrp,
    IoFreeIrp, IoDetachDevice, IoDeleteDevice,
    MmBuildMdlForNonPagedPool,
    ExAllocatePoolWithTag, ExFreePoolWithTag,
    _POOL_TYPE_NonPagedPool as NonPagedPool,
    _KWAIT_REASON_Executive as Executive,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    KeInitializeEvent, KeWaitForSingleObject,
};
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::constants::{
    STATUS_PENDING, KernelMode, IO_NO_INCREMENT,
};
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, IoCallDriver,
    IRP_MN_START_DEVICE, IRP_MN_REMOVE_DEVICE,
    IRP_MN_STOP_DEVICE, IRP_MN_SURPRISE_REMOVAL,
    IRP_MN_CANCEL_REMOVE_DEVICE, IRP_MN_QUERY_STOP_DEVICE,
    IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_QUERY_REMOVE_DEVICE,
};
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation,
    IoCopyCurrentIrpStackLocationToNext, IoSetCompletionRoutine,
};
use hid_sys::{HIDP_CAPS, PHIDP_PREPARSED_DATA, HidP_Input};
use crate::types::{PKBDHID_DEVICE_EXTENSION, USAGE_AND_PAGE, PUSAGE_AND_PAGE};
use crate::glue::{
    KBDHID_TAG,
    IOCTL_HID_GET_COLLECTION_INFORMATION, IOCTL_HID_GET_COLLECTION_DESCRIPTOR,
    HID_COLLECTION_INFORMATION, HID_USAGE_PAGE_KEYBOARD,
    STATUS_MORE_PROCESSING_REQUIRED,
    HidP_GetCaps, HidP_MaxUsageListLength,
    KEYBOARD_TYPEMATIC_RATE_MINIMUM, KEYBOARD_TYPEMATIC_RATE_DEFAULT,
    KEYBOARD_TYPEMATIC_DELAY_MINIMUM, KEYBOARD_TYPEMATIC_DELAY_MAXIMUM,
    MICROSOFT_KBD_FUNC, KEYBOARD_TYPE_UNKNOWN, MICROSOFT_KBD_101_TYPE,
};

/// KbdHid_SubmitRequest — synchronously submit an IOCTL to the lower device.
pub unsafe fn KbdHid_SubmitRequest(
    DeviceObject: PDEVICE_OBJECT,
    IoControlCode: u32,
    InputBufferSize: u32,
    InputBuffer: PVOID,
    OutputBufferSize: u32,
    OutputBuffer: PVOID,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();

    /* init event */
    KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

    /* build request */
    let Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        (*DeviceExtension).NextDeviceObject,
        InputBuffer,
        InputBufferSize,
        OutputBuffer,
        OutputBufferSize,
        FALSE,
        &mut Event as *mut _ as _,
        &mut IoStatus,
    );
    if Irp.is_null() {
        /* no memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* send request */
    let mut Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        /* wait for request to complete */
        KeWaitForSingleObject(
            &mut Event as *mut _ as _,
            Executive,
            KernelMode as _,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    /* done */
    Status
}

/// KbdHid_StartDevice — IRP_MN_START_DEVICE inner logic.
pub unsafe fn KbdHid_StartDevice(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;
    let mut Information: HID_COLLECTION_INFORMATION = core::mem::zeroed();
    let mut Capabilities: HIDP_CAPS = core::mem::zeroed();

    /* query collection information */
    let mut Status = KbdHid_SubmitRequest(
        DeviceObject,
        IOCTL_HID_GET_COLLECTION_INFORMATION,
        0,
        core::ptr::null_mut(),
        size_of::<HID_COLLECTION_INFORMATION>() as u32,
        &mut Information as *mut _ as PVOID,
    );
    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        /* failed to query collection information */
        return Status;
    }

    /* lets allocate space for preparsed data */
    let PreparsedData = ExAllocatePoolWithTag(
        NonPagedPool,
        Information.DescriptorSize as _,
        KBDHID_TAG,
    ) as PHIDP_PREPARSED_DATA;
    if PreparsedData.is_null() {
        /* no memory */
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* now obtain the preparsed data */
    Status = KbdHid_SubmitRequest(
        DeviceObject,
        IOCTL_HID_GET_COLLECTION_DESCRIPTOR,
        0,
        core::ptr::null_mut(),
        Information.DescriptorSize,
        PreparsedData as PVOID,
    );
    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        /* failed to get preparsed data */
        ExFreePoolWithTag(PreparsedData as PVOID, KBDHID_TAG);
        return Status;
    }

    /* lets get the caps */
    Status = HidP_GetCaps(PreparsedData, &mut Capabilities);
    if Status != hid_sys::HIDP_STATUS_SUCCESS {
        /* failed to get capabilities */
        ExFreePoolWithTag(PreparsedData as PVOID, KBDHID_TAG);
        return Status;
    }

    /* init input report */
    (*DeviceExtension).ReportLength = Capabilities.InputReportByteLength as u32;
    debug_assert!((*DeviceExtension).ReportLength != 0);
    (*DeviceExtension).Report = ExAllocatePoolWithTag(
        NonPagedPool,
        (*DeviceExtension).ReportLength as _,
        KBDHID_TAG,
    ) as *mut i8;
    debug_assert!(!(*DeviceExtension).Report.is_null());
    RtlZeroMemory(
        (*DeviceExtension).Report as PVOID,
        (*DeviceExtension).ReportLength as _,
    );

    /* build mdl */
    (*DeviceExtension).ReportMDL = IoAllocateMdl(
        (*DeviceExtension).Report as PVOID,
        (*DeviceExtension).ReportLength,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    debug_assert!(!(*DeviceExtension).ReportMDL.is_null());

    /* init mdl */
    MmBuildMdlForNonPagedPool((*DeviceExtension).ReportMDL);

    /* get max number of buttons */
    let Buttons = HidP_MaxUsageListLength(
        HidP_Input,
        HID_USAGE_PAGE_KEYBOARD,
        PreparsedData,
    );
    debug_assert!(Buttons > 0);

    /* now allocate an array for those buttons (4 usage lists packed in one alloc) */
    let Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (size_of::<USAGE_AND_PAGE>() * 4 * Buttons as usize) as _,
        KBDHID_TAG,
    ) as PUSAGE_AND_PAGE;
    if Buffer.is_null() {
        /* no memory */
        ExFreePoolWithTag(PreparsedData as PVOID, KBDHID_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*DeviceExtension).UsageListBuffer = Buffer as PVOID;

    /* init usage lists */
    RtlZeroMemory(
        Buffer as PVOID,
        (size_of::<USAGE_AND_PAGE>() * 4 * Buttons as usize) as _,
    );
    (*DeviceExtension).CurrentUsageList = Buffer;
    let Buffer = Buffer.offset(Buttons as isize);
    (*DeviceExtension).PreviousUsageList = Buffer;
    let Buffer = Buffer.offset(Buttons as isize);
    (*DeviceExtension).MakeUsageList = Buffer;
    let Buffer = Buffer.offset(Buttons as isize);
    (*DeviceExtension).BreakUsageList = Buffer;

    // FIXME: implement device hacks (UsageMappings, KeyboardTypeOverride, etc.)

    /* store number of buttons */
    (*DeviceExtension).UsageListLength = Buttons as u16;

    /* store preparsed data */
    (*DeviceExtension).PreparsedData = PreparsedData;

    /* completed successfully */
    STATUS_SUCCESS
}

/// KbdHid_StartDeviceCompletion — signals the START_DEVICE synchronisation event.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_StartDeviceCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    KeSetEvent(Context as _, 0, FALSE);
    STATUS_MORE_PROCESSING_REQUIRED
}

/// KbdHid_FreeResources — free all resources allocated in KbdHid_StartDevice.
pub unsafe fn KbdHid_FreeResources(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    /* free resources */
    if !(*DeviceExtension).PreparsedData.is_null() {
        ExFreePoolWithTag((*DeviceExtension).PreparsedData as PVOID, KBDHID_TAG);
        (*DeviceExtension).PreparsedData = core::ptr::null_mut();
    }

    if !(*DeviceExtension).UsageListBuffer.is_null() {
        ExFreePoolWithTag((*DeviceExtension).UsageListBuffer, KBDHID_TAG);
        (*DeviceExtension).UsageListBuffer = core::ptr::null_mut();
        (*DeviceExtension).CurrentUsageList = core::ptr::null_mut();
        (*DeviceExtension).PreviousUsageList = core::ptr::null_mut();
        (*DeviceExtension).MakeUsageList = core::ptr::null_mut();
        (*DeviceExtension).BreakUsageList = core::ptr::null_mut();
    }

    if !(*DeviceExtension).ReportMDL.is_null() {
        IoFreeMdl((*DeviceExtension).ReportMDL);
        (*DeviceExtension).ReportMDL = core::ptr::null_mut();
    }

    if !(*DeviceExtension).Report.is_null() {
        ExFreePoolWithTag((*DeviceExtension).Report as PVOID, KBDHID_TAG);
        (*DeviceExtension).Report = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}

/// KbdHid_Pnp — IRP_MJ_PNP dispatch.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Pnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let minor = (*IoStack).MinorFunction;

    match minor {
        m if m == IRP_MN_STOP_DEVICE || m == IRP_MN_SURPRISE_REMOVAL => {
            /* free resources */
            KbdHid_FreeResources(DeviceObject);
            /* fall through */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
        }

        m if m == IRP_MN_CANCEL_REMOVE_DEVICE
            || m == IRP_MN_QUERY_STOP_DEVICE
            || m == IRP_MN_CANCEL_STOP_DEVICE
            || m == IRP_MN_QUERY_REMOVE_DEVICE =>
        {
            /* indicate success */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            /* skip irp stack location */
            IoSkipCurrentIrpStackLocation(Irp);
            /* dispatch to lower device */
            IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
        }

        m if m == IRP_MN_REMOVE_DEVICE => {
            /* FIXME synchronization */

            /* cancel irp */
            IoCancelIrp((*DeviceExtension).Irp);

            /* free resources */
            KbdHid_FreeResources(DeviceObject);

            /* indicate success */
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            /* skip irp stack location */
            IoSkipCurrentIrpStackLocation(Irp);

            /* dispatch to lower device */
            let Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);

            IoFreeIrp((*DeviceExtension).Irp);
            IoDetachDevice((*DeviceExtension).NextDeviceObject);
            IoDeleteDevice(DeviceObject);
            Status
        }

        m if m == IRP_MN_START_DEVICE => {
            let mut Event: KEVENT = core::mem::zeroed();

            /* init event */
            KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

            /* copy stack location */
            IoCopyCurrentIrpStackLocationToNext(Irp);

            /* set completion routine */
            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS, PIO_COMPLETION_ROUTINE>(KbdHid_StartDeviceCompletion),
                &mut Event as *mut _ as PVOID,
                TRUE,
                TRUE,
                TRUE,
            );
            (*Irp).IoStatus.__bindgen_anon_1.Status = 0;

            /* pass request */
            let mut Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    &mut Event as *mut _ as _,
                    Executive,
                    KernelMode as _,
                    FALSE,
                    core::ptr::null_mut(),
                );
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }

            if !ntkrnl_sys::base::NT_SUCCESS(Status) {
                /* failed */
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return Status;
            }

            /* lets start the device */
            Status = KbdHid_StartDevice(DeviceObject);

            /* complete request */
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);

            /* done */
            Status
        }

        _ => {
            /* skip irp stack location */
            IoSkipCurrentIrpStackLocation(Irp);
            /* dispatch to lower device */
            IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
        }
    }
}
