// pnp.rs — MouHid_StartDevice, MouHid_FreeResources, MouHid_Flush, MouHid_Pnp,
//           MouHid_SubmitRequest
// Translated from drivers/hid/mouhid/mouhid.c

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext, IoSkipCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, PRKEVENT,
    IoBuildDeviceIoControlRequest,
    IoFreeMdl,
    IoFreeIrp,
    IoDetachDevice,
    IoDeleteDevice,
    IoCancelIrp,
    IoAllocateMdl,
    MmBuildMdlForNonPagedPool,
    KeWaitForSingleObject,
    KeInitializeEvent,
    ExAllocatePoolWithTag,
    ExFreePoolWithTag,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _KWAIT_REASON_Executive as Executive,
    _POOL_TYPE_NonPagedPool as NonPagedPool,
    IO_STATUS_BLOCK,
    _IO_STACK_LOCATION,
};
use hid_sys::{
    HIDP_CAPS,
    HIDP_VALUE_CAPS,
    HIDP_STATUS_SUCCESS,
    HidP_Input,
};

macro_rules! ASSERT {
    ($e:expr) => { ntkrnl_sys::ASSERT!($e) };
}

use crate::types::{PMOUHID_DEVICE_EXTENSION, PUSAGE, USAGE};
use crate::glue::*;
use crate::read::MouHid_StartDeviceCompletion;

// ---------------------------------------------------------------------------
// MouHid_SubmitRequest — synchronous IOCTL helper
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_SubmitRequest(
    DeviceObject: PDEVICE_OBJECT,
    IoControlCode: ULONG,
    InputBufferSize: ULONG,
    InputBuffer: PVOID,
    OutputBufferSize: ULONG,
    OutputBuffer: PVOID,
) -> NTSTATUS {
    let mut Event: ntkrnl_sys::ddk::KEVENT = core::mem::zeroed();
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();

    let ext = &*DeviceExtension;

    // init event
    KeInitializeEvent(&mut Event, NotificationEvent, FALSE);

    // build request
    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        ext.NextDeviceObject,
        InputBuffer,
        InputBufferSize,
        OutputBuffer,
        OutputBufferSize,
        FALSE,
        &mut Event,
        &mut IoStatus,
    );
    if Irp.is_null() {
        // no memory
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // send request
    Status = IoCallDriver(ext.NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        // wait for request to complete
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            Executive,
            0i8, // KernelMode
            FALSE,
            core::ptr::null_mut(),
        );
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    // done
    Status
}

// ---------------------------------------------------------------------------
// MouHid_StartDevice — IRP_MN_START_DEVICE body
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_StartDevice(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buttons: ULONG;
    let mut Information: HID_COLLECTION_INFORMATION = core::mem::zeroed();
    let mut PreparsedData: PVOID;
    let mut Capabilities: HIDP_CAPS = core::mem::zeroed();
    let mut ValueCapsLength: USHORT;
    let mut ValueCaps: HIDP_VALUE_CAPS = core::mem::zeroed();
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let mut Buffer: PUSAGE;

    let ext = &mut *DeviceExtension;

    // query collection information
    Status = MouHid_SubmitRequest(
        DeviceObject,
        IOCTL_HID_GET_COLLECTION_INFORMATION,
        0,
        core::ptr::null_mut(),
        core::mem::size_of::<HID_COLLECTION_INFORMATION>() as ULONG,
        &mut Information as *mut _ as PVOID,
    );
    if !NT_SUCCESS(Status) {
        // failed to query collection information
        // DPRINT1("[MOUHID] failed to obtain collection information with %x\n", Status)
        return Status;
    }

    // lets allocate space for preparsed data
    PreparsedData = ExAllocatePoolWithTag(
        NonPagedPool,
        Information.DescriptorSize as _,
        MOUHID_TAG,
    );
    if PreparsedData.is_null() {
        // no memory
        // DPRINT1("[MOUHID] no memory size %u\n", Information.DescriptorSize)
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // now obtain the preparsed data
    Status = MouHid_SubmitRequest(
        DeviceObject,
        IOCTL_HID_GET_COLLECTION_DESCRIPTOR,
        0,
        core::ptr::null_mut(),
        Information.DescriptorSize,
        PreparsedData,
    );
    if !NT_SUCCESS(Status) {
        // failed to get preparsed data
        // DPRINT1("[MOUHID] failed to obtain collection information with %x\n", Status)
        ExFreePoolWithTag(PreparsedData, MOUHID_TAG);
        return Status;
    }

    // lets get the caps
    Status = HidP_GetCaps(PreparsedData as _, &mut Capabilities);
    if Status != HIDP_STATUS_SUCCESS {
        // failed to get capabilities
        // DPRINT1("[MOUHID] failed to obtain caps with %x\n", Status)
        ExFreePoolWithTag(PreparsedData, MOUHID_TAG);
        return Status;
    }

    // DPRINT "[MOUHID] Usage %x UsagePage %x InputReportLength %lu\n"

    // verify capabilities
    if (Capabilities.Usage != HID_USAGE_GENERIC_POINTER
        && Capabilities.Usage != HID_USAGE_GENERIC_MOUSE)
        || Capabilities.UsagePage != HID_USAGE_PAGE_GENERIC
    {
        // not supported
        ExFreePoolWithTag(PreparsedData, MOUHID_TAG);
        return STATUS_UNSUCCESSFUL;
    }

    // init input report
    ext.ReportLength = Capabilities.InputReportByteLength as ULONG;
    ASSERT!(ext.ReportLength != 0);
    ext.Report = ExAllocatePoolWithTag(
        NonPagedPool,
        ext.ReportLength as _,
        MOUHID_TAG,
    ) as *mut CHAR;
    ASSERT!(!ext.Report.is_null());
    RtlZeroMemory(ext.Report as PVOID, ext.ReportLength as usize);

    // build mdl
    ext.ReportMDL = IoAllocateMdl(
        ext.Report as PVOID,
        ext.ReportLength,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    ASSERT!(!ext.ReportMDL.is_null());

    // init mdl
    MmBuildMdlForNonPagedPool(ext.ReportMDL);

    // get max number of buttons
    Buttons = HidP_MaxUsageListLength(
        HidP_Input,
        HID_USAGE_PAGE_BUTTON,
        PreparsedData as _,
    );
    // DPRINT "[MOUHID] Buttons %lu\n"
    ASSERT!(Buttons > 0);

    // now allocate an array for those buttons (4 lists × Buttons USAGEs)
    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        ((core::mem::size_of::<USAGE>() as ULONG) * 4 * Buttons) as _,
        MOUHID_TAG,
    ) as PUSAGE;
    if Buffer.is_null() {
        // no memory
        ExFreePoolWithTag(PreparsedData, MOUHID_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ext.UsageListBuffer = Buffer as PVOID;

    // init usage lists
    RtlZeroMemory(
        Buffer as PVOID,
        ((core::mem::size_of::<USAGE>() as ULONG) * 4 * Buttons) as usize,
    );
    ext.CurrentUsageList = Buffer;
    Buffer = Buffer.add(Buttons as usize);
    ext.PreviousUsageList = Buffer;
    Buffer = Buffer.add(Buttons as usize);
    ext.MakeUsageList = Buffer;
    Buffer = Buffer.add(Buttons as usize);
    ext.BreakUsageList = Buffer;

    // store number of buttons
    ext.UsageListLength = Buttons as USHORT;

    // store preparsed data
    ext.PreparsedData = PreparsedData;

    ValueCapsLength = 1;
    HidP_GetSpecificValueCaps(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        HIDP_LINK_COLLECTION_UNSPECIFIED,
        HID_USAGE_GENERIC_X,
        &mut ext.ValueCapsX,
        &mut ValueCapsLength,
        PreparsedData as _,
    );

    ValueCapsLength = 1;
    HidP_GetSpecificValueCaps(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        HIDP_LINK_COLLECTION_UNSPECIFIED,
        HID_USAGE_GENERIC_Y,
        &mut ext.ValueCapsY,
        &mut ValueCapsLength,
        PreparsedData as _,
    );

    // now check for wheel mouse support
    ValueCapsLength = 1;
    Status = HidP_GetSpecificValueCaps(
        HidP_Input,
        HID_USAGE_PAGE_GENERIC,
        HIDP_LINK_COLLECTION_UNSPECIFIED,
        HID_USAGE_GENERIC_WHEEL,
        &mut ValueCaps,
        &mut ValueCapsLength,
        PreparsedData as _,
    );
    if Status == HIDP_STATUS_SUCCESS {
        // mouse has wheel support
        ext.MouseIdentifier = WHEELMOUSE_HID_HARDWARE;
        ext.WheelUsagePage = ValueCaps.UsagePage;
        // DPRINT "[MOUHID] mouse wheel support detected\n"
    } else {
        // check if the mouse has z-axis
        ValueCapsLength = 1;
        Status = HidP_GetSpecificValueCaps(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_Z,
            &mut ValueCaps,
            &mut ValueCapsLength,
            PreparsedData as _,
        );
        if Status == HIDP_STATUS_SUCCESS && ValueCapsLength == 1 {
            // wheel support
            ext.MouseIdentifier = WHEELMOUSE_HID_HARDWARE;
            ext.WheelUsagePage = ValueCaps.UsagePage;
            // DPRINT "[MOUHID] mouse wheel support detected with z-axis\n"
        }
    }

    // check if mice is absolute
    if ext.ValueCapsY.IsAbsolute != 0 && ext.ValueCapsX.IsAbsolute != 0 {
        // mice is absolute
        ext.MouseAbsolute = TRUE;
    }

    // completed successfully
    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// MouHid_FreeResources
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_FreeResources(
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &mut *DeviceExtension;

    // free resources
    if !ext.PreparsedData.is_null() {
        ExFreePoolWithTag(ext.PreparsedData, MOUHID_TAG);
        ext.PreparsedData = core::ptr::null_mut();
    }

    if !ext.UsageListBuffer.is_null() {
        ExFreePoolWithTag(ext.UsageListBuffer, MOUHID_TAG);
        ext.UsageListBuffer = core::ptr::null_mut();
        ext.CurrentUsageList = core::ptr::null_mut();
        ext.PreviousUsageList = core::ptr::null_mut();
        ext.MakeUsageList = core::ptr::null_mut();
        ext.BreakUsageList = core::ptr::null_mut();
    }

    if !ext.ReportMDL.is_null() {
        IoFreeMdl(ext.ReportMDL);
        ext.ReportMDL = core::ptr::null_mut();
    }

    if !ext.Report.is_null() {
        ExFreePoolWithTag(ext.Report as PVOID, MOUHID_TAG);
        ext.Report = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// MouHid_Flush — IRP_MJ_FLUSH_BUFFERS
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Flush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &*DeviceExtension;

    // skip current stack location
    IoSkipCurrentIrpStackLocation(Irp);

    // get next stack location
    let IoStack = IoGetNextIrpStackLocation(Irp);

    // change request to hid flush queue request
    (*IoStack).MajorFunction = IRP_MJ_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_HID_FLUSH_QUEUE;

    // call device
    IoCallDriver(ext.NextDeviceObject, Irp)
}

// ---------------------------------------------------------------------------
// MouHid_Pnp — IRP_MJ_PNP
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Pnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let mut Event: ntkrnl_sys::ddk::KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &mut *DeviceExtension;

    // get current irp stack
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    // DPRINT "[MOUHID] IRP_MJ_PNP Request: %x\n"

    match (*IoStack).MinorFunction {
        IRP_MN_STOP_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
            // free resources
            MouHid_FreeResources(DeviceObject);
            // fall through to common pass-through handling
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoSkipCurrentIrpStackLocation(Irp);
            IoCallDriver(ext.NextDeviceObject, Irp)
        }

        IRP_MN_CANCEL_REMOVE_DEVICE
        | IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE => {
            // indicate success
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            // skip irp stack location
            IoSkipCurrentIrpStackLocation(Irp);

            // dispatch to lower device
            IoCallDriver(ext.NextDeviceObject, Irp)
        }

        IRP_MN_REMOVE_DEVICE => {
            // FIXME synchronization

            // request stop
            ext.StopReadReport = TRUE;

            // cancel irp
            IoCancelIrp(ext.Irp);

            // free resources
            MouHid_FreeResources(DeviceObject);

            // indicate success
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

            // skip irp stack location
            IoSkipCurrentIrpStackLocation(Irp);

            // dispatch to lower device
            Status = IoCallDriver(ext.NextDeviceObject, Irp);

            // wait for completion of stop event
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(ext.ReadCompletionEvent) as PVOID,
                Executive,
                0i8, // KernelMode
                FALSE,
                core::ptr::null_mut(),
            );

            // free irp
            IoFreeIrp(ext.Irp);

            // detach device
            IoDetachDevice(ext.NextDeviceObject);

            // delete device
            IoDeleteDevice(DeviceObject);

            // done
            Status
        }

        IRP_MN_START_DEVICE => {
            // init event
            KeInitializeEvent(&mut Event, NotificationEvent, FALSE);

            // copy stack location
            IoCopyCurrentIrpStackLocationToNext(Irp);

            // set completion routine
            type CompletionFn = unsafe extern "system" fn(
                PDEVICE_OBJECT,
                *mut ntkrnl_sys::ddk::_IRP,
                PVOID,
            ) -> NTSTATUS;
            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<CompletionFn, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(
                    MouHid_StartDeviceCompletion,
                ),
                &mut Event as *mut _ as PVOID,
                TRUE,
                TRUE,
                TRUE,
            );
            (*Irp).IoStatus.__bindgen_anon_1.Status = 0;

            // pass request
            Status = IoCallDriver(ext.NextDeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    Executive,
                    0i8, // KernelMode
                    FALSE,
                    core::ptr::null_mut(),
                );
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }

            if !NT_SUCCESS(Status) {
                // failed
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return Status;
            }

            // lets start the device
            Status = MouHid_StartDevice(DeviceObject);
            // DPRINT "MouHid_StartDevice %x\n"

            // complete request
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);

            // done
            Status
        }

        _ => {
            // skip irp stack location
            IoSkipCurrentIrpStackLocation(Irp);

            // dispatch to lower device
            IoCallDriver(ext.NextDeviceObject, Irp)
        }
    }
}
