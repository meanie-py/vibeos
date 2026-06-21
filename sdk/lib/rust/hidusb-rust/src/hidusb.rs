// hidusb.rs — DriverEntry, AddDevice, the MajorFunction dispatch routines, the
//             read-report path + its completion routine (with the reset-recovery
//             work-item queue), the internal-device-control IOCTL switch, and the
//             PnP handler.
//
// Translated faithfully from drivers/hid/hidusb/hidusb.c.

use core::mem::{size_of, transmute};
use core::ptr::null_mut;

use ntkrnl_sys::base::{NTSTATUS, ULONG, PVOID, TRUE, FALSE};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP,
    PDRIVER_OBJECT, PUNICODE_STRING,
    PDRIVER_DISPATCH, PDRIVER_UNLOAD, PDRIVER_ADD_DEVICE,
    PIO_COMPLETION_ROUTINE,
    KEVENT,
    KeInitializeEvent, KeWaitForSingleObject,
    ExAllocatePoolWithTag, ExFreePoolWithTag,
    IoAllocateWorkItem, IoQueueWorkItem, IoFreeWorkItem,
    PIO_WORKITEM_ROUTINE,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _KWAIT_REASON_Executive as Executive,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _POOL_TYPE_NonPagedPool as NonPagedPool,
};
use ntkrnl_sys::base::{
    STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED, STATUS_INSUFFICIENT_RESOURCES,
};
use ntkrnl_sys::constants::{
    STATUS_PENDING, STATUS_CANCELLED, STATUS_DEVICE_NOT_CONNECTED,
    KernelMode, IO_NO_INCREMENT,
    IRP_MJ_CREATE, IRP_MJ_CLOSE,
};
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, IoCallDriver,
    PoStartNextPowerIrp, PoCallDriver,
    IRP_MJ_INTERNAL_DEVICE_CONTROL, IRP_MJ_POWER, IRP_MJ_SYSTEM_CONTROL, IRP_MJ_PNP,
    IRP_MN_REMOVE_DEVICE, IRP_MN_QUERY_STOP_DEVICE, IRP_MN_QUERY_REMOVE_DEVICE,
    IRP_MN_STOP_DEVICE, IRP_MN_START_DEVICE,
};
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext,
    IoSetCompletionRoutine, IoMarkIrpPending,
};

use usb_sys::{
    PUSBD_PIPE_INFORMATION,
    USBD_TRANSFER_DIRECTION_IN, USBD_SHORT_TRANSFER_OK,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE,
    URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT,
    _URB_BULK_OR_INTERRUPT_TRANSFER, _URB_CONTROL_DESCRIPTOR_REQUEST,
};

use crate::types::{
    HID_USB_DEVICE_EXTENSION, PHID_USB_DEVICE_EXTENSION, PHID_DEVICE_EXTENSION,
    HID_USB_RESET_CONTEXT, PHID_USB_RESET_CONTEXT,
    HID_DEVICE_ATTRIBUTES, PHID_DEVICE_ATTRIBUTES, HID_DESCRIPTOR,
    HID_MINIDRIVER_REGISTRATION,
};
use crate::glue::{
    PURB,
    HIDUSB_TAG, HIDUSB_URB_TAG, HID_REVISION,
    HID_REPORT_DESCRIPTOR_TYPE, PNP_DEVICE_NOT_DISABLEABLE,
    STATUS_CONTINUE_COMPLETION, STATUS_MORE_PROCESSING_REQUIRED,
    STATUS_INVALID_BUFFER_SIZE,
    IOCTL_HID_GET_DEVICE_ATTRIBUTES, IOCTL_HID_GET_DEVICE_DESCRIPTOR,
    IOCTL_HID_GET_REPORT_DESCRIPTOR, IOCTL_HID_READ_REPORT,
    IOCTL_HID_WRITE_REPORT, IOCTL_GET_PHYSICAL_DESCRIPTOR,
    IOCTL_HID_SEND_IDLE_NOTIFICATION_REQUEST, IOCTL_HID_GET_FEATURE,
    IOCTL_HID_SET_FEATURE, IOCTL_HID_SET_OUTPUT_REPORT, IOCTL_HID_GET_INPUT_REPORT,
    IOCTL_HID_GET_INDEXED_STRING, IOCTL_HID_GET_MS_GENRE_DESCRIPTOR,
    IOCTL_INTERNAL_USB_SUBMIT_URB,
    UsbBuildInterruptOrBulkTransferRequest,
    HidRegisterMinidriver,
    IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_PNP_DEVICE_STATE,
};
use crate::urb::{
    HidUsb_GetInputInterruptInterfaceHandle, Hid_DispatchUrb, Hid_PnpCompletion,
    HidUsb_ResetWorkerRoutine,
};
use crate::pnp::{Hid_GetDescriptor, Hid_DisableConfiguration, Hid_PnpStart};

// NT_SUCCESS(Status) == Status >= 0
#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS;

pub unsafe extern "system" fn HidCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let IoStack;

    //
    // get current irp stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // sanity check for hidclass driver
    //
    debug_assert!(
        (*IoStack).MajorFunction == IRP_MJ_CREATE || (*IoStack).MajorFunction == IRP_MJ_CLOSE
    );

    //
    // complete request
    //
    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    let _ = DeviceObject;

    //
    // done
    //
    STATUS_SUCCESS
}}

pub unsafe extern "system" fn HidUsb_ReadReportCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS { unsafe {
    let Urb: PURB;
    let ResetContext: PHID_USB_RESET_CONTEXT;

    //
    // get urb
    //
    Urb = Context as PURB;
    debug_assert!(!Urb.is_null());

    if (*Irp).PendingReturned != FALSE {
        //
        // mark irp pending
        //
        IoMarkIrpPending(Irp);
    }

    //
    // did the reading report succeed / cancelled
    //
    let irp_status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if nt_success(irp_status)
        || irp_status == STATUS_CANCELLED
        || irp_status == STATUS_DEVICE_NOT_CONNECTED
    {
        //
        // store result length
        //
        (*Irp).IoStatus.Information =
            (*Urb).__bindgen_anon_1.UrbBulkOrInterruptTransfer.TransferBufferLength as _;

        //
        // FIXME handle error
        //
        debug_assert!(
            (*Urb).__bindgen_anon_1.UrbHeader.Status == crate::glue::USBD_STATUS_SUCCESS
                || (*Urb).__bindgen_anon_1.UrbHeader.Status == crate::glue::USBD_STATUS_DEVICE_GONE
        );

        //
        // free the urb
        //
        ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

        //
        // finish completion
        //
        return STATUS_CONTINUE_COMPLETION;
    }

    //
    // allocate reset context
    //
    ResetContext = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<HID_USB_RESET_CONTEXT>() as _,
        HIDUSB_TAG,
    ) as PHID_USB_RESET_CONTEXT;
    if !ResetContext.is_null() {
        //
        // allocate work item
        //
        (*ResetContext).WorkItem = IoAllocateWorkItem(DeviceObject);
        if !(*ResetContext).WorkItem.is_null() {
            //
            // init reset context
            //
            (*ResetContext).Irp = Irp;
            (*ResetContext).DeviceObject = DeviceObject;

            //
            // queue the work item
            //
            type WorkerFn = unsafe extern "system" fn(PDEVICE_OBJECT, PVOID);
            IoQueueWorkItem(
                (*ResetContext).WorkItem,
                transmute::<WorkerFn, PIO_WORKITEM_ROUTINE>(HidUsb_ResetWorkerRoutine),
                DelayedWorkQueue,
                ResetContext as PVOID,
            );

            //
            // free urb
            //
            ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

            //
            // defer completion
            //
            return STATUS_MORE_PROCESSING_REQUIRED;
        }
        //
        // free context
        //
        ExFreePoolWithTag(ResetContext as PVOID, HIDUSB_TAG);
    }

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // complete request
    //
    STATUS_CONTINUE_COMPLETION
}}

pub unsafe extern "system" fn HidUsb_ReadReport(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut IoStack;
    let Urb: PURB;
    let PipeInformation: PUSBD_PIPE_INFORMATION;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // sanity checks
    //
    debug_assert!((*IoStack).Parameters.DeviceIoControl.OutputBufferLength != 0);
    debug_assert!(!(*Irp).UserBuffer.is_null());
    debug_assert!(!(*HidDeviceExtension).InterfaceInfo.is_null());

    //
    // get interrupt input pipe
    //
    PipeInformation = HidUsb_GetInputInterruptInterfaceHandle((*HidDeviceExtension).InterfaceInfo);
    debug_assert!(!PipeInformation.is_null());

    //
    // lets allocate urb
    //
    Urb = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>() as _,
        HIDUSB_URB_TAG,
    ) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // init urb
    //
    RtlZeroMemory(Urb as PVOID, size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>() as _);

    //
    // sanity check
    //
    debug_assert!(!(*Irp).UserBuffer.is_null());
    debug_assert!((*IoStack).Parameters.DeviceIoControl.OutputBufferLength != 0);
    debug_assert!(!(*PipeInformation).PipeHandle.is_null());

    //
    // build the urb
    //
    UsbBuildInterruptOrBulkTransferRequest(
        Urb,
        size_of::<_URB_BULK_OR_INTERRUPT_TRANSFER>() as u16,
        (*PipeInformation).PipeHandle,
        (*Irp).UserBuffer,
        null_mut(),
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        USBD_TRANSFER_DIRECTION_IN | USBD_SHORT_TRANSFER_OK,
        null_mut(),
    );

    //
    // store configuration handle
    //
    (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle = (*HidDeviceExtension).ConfigurationHandle;

    //
    // get next location to setup irp
    //
    IoStack = IoGetNextIrpStackLocation(Irp);

    //
    // init irp for lower driver
    //
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
    (*IoStack).Parameters.DeviceIoControl.InputBufferLength = 0;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = 0;
    (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer = null_mut();
    (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;

    //
    // set completion routine
    //
    type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
    IoSetCompletionRoutine(
        Irp,
        transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(HidUsb_ReadReportCompletion),
        Urb as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    //
    // call driver
    //
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}}

pub unsafe fn HidUsb_GetReportDescriptor(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut Report: PVOID = null_mut();
    let mut BufferLength: ULONG;
    let Length: ULONG;
    let IoStack;
    let mut Status: NTSTATUS;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // sanity checks
    //
    debug_assert!(!HidDeviceExtension.is_null());
    debug_assert!(!(*HidDeviceExtension).HidDescriptor.is_null());
    debug_assert!((*(*HidDeviceExtension).HidDescriptor).bNumDescriptors >= 1);
    debug_assert!(
        (*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].bReportType
            == HID_REPORT_DESCRIPTOR_TYPE
    );
    debug_assert!((*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].wReportLength > 0);

    //
    // FIXME: support old hid version
    //
    BufferLength =
        (*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].wReportLength as ULONG;
    Status = Hid_GetDescriptor(
        DeviceObject,
        URB_FUNCTION_GET_DESCRIPTOR_FROM_INTERFACE as u16,
        size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as u16,
        &mut Report,
        &mut BufferLength,
        (*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].bReportType,
        0,
        (*(*HidDeviceExtension).InterfaceInfo).InterfaceNumber as u16,
    );
    if !nt_success(Status) {
        //
        // failed to get descriptor
        // try with old hid version
        //
        BufferLength =
            (*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].wReportLength as ULONG;
        Status = Hid_GetDescriptor(
            DeviceObject,
            URB_FUNCTION_GET_DESCRIPTOR_FROM_ENDPOINT as u16,
            size_of::<_URB_CONTROL_DESCRIPTOR_REQUEST>() as u16,
            &mut Report,
            &mut BufferLength,
            (*(*HidDeviceExtension).HidDescriptor).DescriptorList[0].bReportType,
            0,
            0, /* FIXME */
        );
        if !nt_success(Status) {
            return Status;
        }
    }

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // get length to copy
    //
    Length = core::cmp::min(
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        BufferLength,
    );
    debug_assert!(Length != 0);

    //
    // copy result
    //
    RtlCopyMemory((*Irp).UserBuffer, Report, Length as _);

    //
    // store result length
    //
    (*Irp).IoStatus.Information = Length as _;

    //
    // free the report buffer
    //
    ExFreePoolWithTag(Report, HIDUSB_TAG);

    //
    // done
    //
    Status
}}

pub unsafe extern "system" fn HidInternalDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let IoStack;
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let Attributes: PHID_DEVICE_ATTRIBUTES;
    let Length: ULONG;
    let Status: NTSTATUS;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    let ioctl = (*IoStack).Parameters.DeviceIoControl.IoControlCode;

    if ioctl == IOCTL_HID_GET_DEVICE_ATTRIBUTES {
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
            < size_of::<HID_DEVICE_ATTRIBUTES>() as ULONG
        {
            //
            // invalid request
            //
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_BUFFER_SIZE;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INVALID_BUFFER_SIZE;
        }
        //
        // store result
        //
        debug_assert!(!(*HidDeviceExtension).DeviceDescriptor.is_null());
        (*Irp).IoStatus.Information = size_of::<HID_DESCRIPTOR>() as _;
        Attributes = (*Irp).UserBuffer as PHID_DEVICE_ATTRIBUTES;
        (*Attributes).Size = size_of::<HID_DEVICE_ATTRIBUTES>() as u32;
        (*Attributes).VendorID = (*(*HidDeviceExtension).DeviceDescriptor).idVendor;
        (*Attributes).ProductID = (*(*HidDeviceExtension).DeviceDescriptor).idProduct;
        (*Attributes).VersionNumber = (*(*HidDeviceExtension).DeviceDescriptor).bcdDevice;

        //
        // complete request
        //
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    } else if ioctl == IOCTL_HID_GET_DEVICE_DESCRIPTOR {
        //
        // sanity check
        //
        debug_assert!(!(*HidDeviceExtension).HidDescriptor.is_null());

        //
        // store length
        //
        Length = core::cmp::min(
            (*(*HidDeviceExtension).HidDescriptor).bLength as ULONG,
            (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        );

        //
        // copy descriptor
        //
        RtlCopyMemory(
            (*Irp).UserBuffer,
            (*HidDeviceExtension).HidDescriptor as PVOID,
            Length as _,
        );

        //
        // store result length
        //
        (*Irp).IoStatus.Information = (*(*HidDeviceExtension).HidDescriptor).bLength as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

        /* complete request */
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    } else if ioctl == IOCTL_HID_GET_REPORT_DESCRIPTOR {
        Status = HidUsb_GetReportDescriptor(DeviceObject, Irp);
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    } else if ioctl == IOCTL_HID_READ_REPORT {
        Status = HidUsb_ReadReport(DeviceObject, Irp);
        return Status;
    } else if ioctl == IOCTL_HID_WRITE_REPORT
        || ioctl == IOCTL_GET_PHYSICAL_DESCRIPTOR
        || ioctl == IOCTL_HID_SEND_IDLE_NOTIFICATION_REQUEST
        || ioctl == IOCTL_HID_GET_FEATURE
        || ioctl == IOCTL_HID_SET_FEATURE
        || ioctl == IOCTL_HID_SET_OUTPUT_REPORT
        || ioctl == IOCTL_HID_GET_INPUT_REPORT
        || ioctl == IOCTL_HID_GET_INDEXED_STRING
        || ioctl == IOCTL_HID_GET_MS_GENRE_DESCRIPTOR
    {
        debug_assert!(false);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_IMPLEMENTED;
    } else {
        // UNIMPLEMENTED
        debug_assert!(false);
        let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }
}}

pub unsafe extern "system" fn HidPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let DeviceExtension: PHID_DEVICE_EXTENSION;

    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}}

pub unsafe extern "system" fn HidSystemControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let DeviceExtension: PHID_DEVICE_EXTENSION;

    //
    // get hid device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // skip stack location
    //
    IoSkipCurrentIrpStackLocation(Irp);

    //
    // submit request
    //
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}}

pub unsafe extern "system" fn HidPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS { unsafe {
    let mut Status: NTSTATUS;
    let IoStack;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut Event: KEVENT = core::mem::zeroed();

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // get current stack location
    //
    IoStack = IoGetCurrentIrpStackLocation(Irp);

    //
    // handle requests based on request type
    //
    let minor = (*IoStack).MinorFunction;
    if minor == IRP_MN_REMOVE_DEVICE {
        //
        // unconfigure device
        //
        Hid_DisableConfiguration(DeviceObject);

        //
        // pass request onto lower driver
        //
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);

        Status
    } else if minor == IRP_MN_QUERY_PNP_DEVICE_STATE {
        //
        // device can not be disabled
        //
        (*Irp).IoStatus.Information |= PNP_DEVICE_NOT_DISABLEABLE as ntkrnl_sys::ddk::ULONG_PTR;

        //
        // pass request to next request
        //
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);

        Status
    } else if minor == IRP_MN_QUERY_STOP_DEVICE || minor == IRP_MN_QUERY_REMOVE_DEVICE {
        //
        // we're fine with it
        //
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;

        //
        // pass request to next driver
        //
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);

        Status
    } else if minor == IRP_MN_STOP_DEVICE {
        //
        // unconfigure device
        //
        Hid_DisableConfiguration(DeviceObject);

        //
        // prepare irp
        //
        KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);
        IoCopyCurrentIrpStackLocationToNext(Irp);
        type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
        IoSetCompletionRoutine(
            Irp,
            transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(Hid_PnpCompletion),
            &mut Event as *mut _ as _,
            TRUE,
            TRUE,
            TRUE,
        );

        //
        // send irp and wait for completion
        //
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, null_mut());
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        //
        // done
        //
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        Status
    } else if minor == IRP_MN_QUERY_CAPABILITIES {
        //
        // prepare irp
        //
        KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);
        IoCopyCurrentIrpStackLocationToNext(Irp);
        type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
        IoSetCompletionRoutine(
            Irp,
            transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(Hid_PnpCompletion),
            &mut Event as *mut _ as _,
            TRUE,
            TRUE,
            TRUE,
        );

        //
        // send irp and wait for completion
        //
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, null_mut());
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        if nt_success(Status)
            && !(*IoStack).Parameters.DeviceCapabilities.Capabilities.is_null()
        {
            //
            // don't need to safely remove
            //
            (*(*IoStack).Parameters.DeviceCapabilities.Capabilities).set_SurpriseRemovalOK(TRUE as ULONG);
        }

        //
        // done
        //
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        Status
    } else if minor == IRP_MN_START_DEVICE {
        //
        // prepare irp
        //
        KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);
        IoCopyCurrentIrpStackLocationToNext(Irp);
        type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
        IoSetCompletionRoutine(
            Irp,
            transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(Hid_PnpCompletion),
            &mut Event as *mut _ as _,
            TRUE,
            TRUE,
            TRUE,
        );

        //
        // send irp and wait for completion
        //
        Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, null_mut());
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        //
        // did the device successfully start
        //
        if !nt_success(Status) {
            //
            // failed
            //
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return Status;
        }

        //
        // start device
        //
        Status = Hid_PnpStart(DeviceObject);

        //
        // complete request
        //
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        Status
    } else {
        //
        // forward and forget request
        //
        IoSkipCurrentIrpStackLocation(Irp);
        IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
    }
}}

pub unsafe extern "system" fn HidAddDevice(
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // init event
    //
    KeInitializeEvent(
        &mut (*HidDeviceExtension).Event as *mut _ as _,
        NotificationEvent,
        FALSE,
    );

    let _ = DriverObject;

    //
    // done
    //
    STATUS_SUCCESS
}}

pub unsafe extern "system" fn Hid_Unload(DriverObject: PDRIVER_OBJECT) {
    // UNIMPLEMENTED
    let _ = DriverObject;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegPath: PUNICODE_STRING,
) -> NTSTATUS { unsafe {
    let mut Registration: HID_MINIDRIVER_REGISTRATION = core::mem::zeroed();
    let Status: NTSTATUS;

    //
    // initialize driver object
    //
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidCreate);
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidInternalDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidPower);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidSystemControl);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidPnp);
    type AddDeviceFn = unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS;
    (*(*DriverObject).DriverExtension).AddDevice =
        transmute::<AddDeviceFn, PDRIVER_ADD_DEVICE>(HidAddDevice);
    type UnloadFn = unsafe extern "system" fn(PDRIVER_OBJECT);
    (*DriverObject).DriverUnload = transmute::<UnloadFn, PDRIVER_UNLOAD>(Hid_Unload);

    //
    // prepare registration info
    //
    RtlZeroMemory(
        &mut Registration as *mut _ as PVOID,
        size_of::<HID_MINIDRIVER_REGISTRATION>() as _,
    );

    //
    // fill in registration info
    //
    Registration.Revision = HID_REVISION;
    Registration.DriverObject = DriverObject;
    Registration.RegistryPath = RegPath;
    Registration.DeviceExtensionSize = size_of::<HID_USB_DEVICE_EXTENSION>() as u32;
    Registration.DevicesArePolled = FALSE;

    //
    // register driver
    //
    Status = HidRegisterMinidriver(&mut Registration);

    Status
}}
