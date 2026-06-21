// urb.rs — URB build/dispatch (event-wait) + the pipe/abort/port RESET error
//          recovery path.
//
// Translated faithfully from drivers/hid/hidusb/hidusb.c:
//   HidUsb_GetInputInterruptInterfaceHandle, HidUsb_GetPortStatus,
//   HidUsb_ResetInterruptPipe, HidUsb_AbortPipe, HidUsb_ResetPort,
//   HidUsb_ResetWorkerRoutine, Hid_PnpCompletion, Hid_DispatchUrb.

use core::mem::{size_of, transmute};
use core::ptr::null_mut;

use ntkrnl_sys::base::{NTSTATUS, ULONG, PVOID, TRUE, FALSE};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, KEVENT, IO_STATUS_BLOCK,
    PIO_COMPLETION_ROUTINE,
    IoBuildDeviceIoControlRequest, KeInitializeEvent, KeWaitForSingleObject,
    ExAllocatePoolWithTag, ExFreePoolWithTag, KeGetCurrentIrql,
    IoFreeWorkItem, IoInvalidateDeviceState,
    _KWAIT_REASON_Executive as Executive,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _POOL_TYPE_NonPagedPool as NonPagedPool,
};
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, PASSIVE_LEVEL};
use ntkrnl_sys::constants::{
    STATUS_PENDING, STATUS_DEVICE_DATA_ERROR, KernelMode, IO_NO_INCREMENT,
};
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{IoGetNextIrpStackLocation, IoSetCompletionRoutine};

use usb_sys::{
    PUSBD_PIPE_INFORMATION, PUSBD_INTERFACE_INFORMATION,
    URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL, URB_FUNCTION_ABORT_PIPE,
    USB_ENDPOINT_DIRECTION_MASK, USB_PORT_STATUS_ENABLE,
    _USBD_PIPE_TYPE_UsbdPipeTypeInterrupt as UsbdPipeTypeInterrupt,
    _URB_PIPE_REQUEST,
};
use crate::glue::PURB;

use crate::types::{
    PHID_DEVICE_EXTENSION, PHID_USB_DEVICE_EXTENSION,
    HID_USB_RESET_CONTEXT, PHID_USB_RESET_CONTEXT,
};
use crate::glue::{
    HIDUSB_TAG, HIDUSB_URB_TAG,
    IOCTL_INTERNAL_USB_SUBMIT_URB, IOCTL_INTERNAL_USB_GET_PORT_STATUS,
    IOCTL_INTERNAL_USB_RESET_PORT,
};

// NT_SUCCESS(Status) == Status >= 0
#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

pub unsafe fn HidUsb_GetInputInterruptInterfaceHandle(
    InterfaceInformation: PUSBD_INTERFACE_INFORMATION,
) -> PUSBD_PIPE_INFORMATION { unsafe {
    let mut Index: ULONG;

    //
    // sanity check
    //
    debug_assert!((*InterfaceInformation).NumberOfPipes != 0);

    Index = 0;
    while Index < (*InterfaceInformation).NumberOfPipes {
        let pipe = (*InterfaceInformation).Pipes.as_ptr().add(Index as usize);
        if (*pipe).PipeType == UsbdPipeTypeInterrupt
            && ((*pipe).EndpointAddress as ULONG & USB_ENDPOINT_DIRECTION_MASK) != 0
        {
            //
            // found handle
            //
            return pipe as PUSBD_PIPE_INFORMATION;
        }
        Index += 1;
    }

    //
    // not found
    //
    null_mut()
}}

pub unsafe fn HidUsb_GetPortStatus(
    DeviceObject: PDEVICE_OBJECT,
    PortStatus: *mut ULONG,
) -> NTSTATUS { unsafe {
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let IoStack;
    let Status: NTSTATUS;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // init result
    //
    *PortStatus = 0;

    //
    // init event
    //
    KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

    //
    // build irp
    //
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_GET_PORT_STATUS,
        (*DeviceExtension).NextDeviceObject,
        null_mut(),
        0,
        null_mut(),
        0,
        TRUE,
        &mut Event as *mut _ as _,
        &mut IoStatus,
    );
    if Irp.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // get stack location
    //
    IoStack = IoGetNextIrpStackLocation(Irp);

    //
    // store result buffer
    //
    (*IoStack).Parameters.Others.Argument1 = PortStatus as PVOID;

    //
    // call driver
    //
    Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        //
        // wait for completion
        //
        KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, 0, null_mut());
        return IoStatus.__bindgen_anon_1.Status;
    }

    //
    // done
    //
    Status
}}

pub unsafe fn HidUsb_ResetInterruptPipe(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let PipeInformation: PUSBD_PIPE_INFORMATION;
    let Urb: PURB;
    let Status: NTSTATUS;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // get interrupt pipe handle
    //
    debug_assert!(!(*HidDeviceExtension).InterfaceInfo.is_null());
    PipeInformation = HidUsb_GetInputInterruptInterfaceHandle((*HidDeviceExtension).InterfaceInfo);
    debug_assert!(!PipeInformation.is_null());
    debug_assert!(!(*PipeInformation).PipeHandle.is_null());

    //
    // allocate urb
    //
    Urb = ExAllocatePoolWithTag(NonPagedPool, size_of::<_URB_PIPE_REQUEST>() as _, HIDUSB_URB_TAG) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // init urb
    //
    RtlZeroMemory(Urb as PVOID, size_of::<_URB_PIPE_REQUEST>() as _);
    (*Urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_SYNC_RESET_PIPE_AND_CLEAR_STALL as u16;
    (*Urb).__bindgen_anon_1.UrbHeader.Length = size_of::<_URB_PIPE_REQUEST>() as u16;
    (*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle = (*PipeInformation).PipeHandle;

    //
    // dispatch request
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // done
    //
    Status
}}

pub unsafe fn HidUsb_AbortPipe(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let HidDeviceExtension: PHID_USB_DEVICE_EXTENSION;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let Urb: PURB;
    let Status: NTSTATUS;
    let PipeInformation: PUSBD_PIPE_INFORMATION;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;
    HidDeviceExtension = (*DeviceExtension).MiniDeviceExtension as PHID_USB_DEVICE_EXTENSION;

    //
    // allocate urb
    //
    Urb = ExAllocatePoolWithTag(NonPagedPool, size_of::<_URB_PIPE_REQUEST>() as _, HIDUSB_URB_TAG) as PURB;
    if Urb.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // get pipe information
    //
    PipeInformation = HidUsb_GetInputInterruptInterfaceHandle((*HidDeviceExtension).InterfaceInfo);
    debug_assert!(!PipeInformation.is_null());
    debug_assert!(!(*PipeInformation).PipeHandle.is_null());

    //
    // init urb
    //
    RtlZeroMemory(Urb as PVOID, size_of::<_URB_PIPE_REQUEST>() as _);
    (*Urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_ABORT_PIPE as u16;
    (*Urb).__bindgen_anon_1.UrbHeader.Length = size_of::<_URB_PIPE_REQUEST>() as u16;
    (*Urb).__bindgen_anon_1.UrbPipeRequest.PipeHandle = (*PipeInformation).PipeHandle;

    //
    // dispatch request
    //
    Status = Hid_DispatchUrb(DeviceObject, Urb);

    //
    // free urb
    //
    ExFreePoolWithTag(Urb as PVOID, HIDUSB_URB_TAG);

    //
    // done
    //
    Status
}}

pub unsafe fn HidUsb_ResetPort(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS { unsafe {
    let mut Event: KEVENT = core::mem::zeroed();
    let Irp: PIRP;
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // init event
    //
    KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

    //
    // build irp
    //
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_RESET_PORT,
        (*DeviceExtension).NextDeviceObject,
        null_mut(),
        0,
        null_mut(),
        0,
        TRUE,
        &mut Event as *mut _ as _,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // send the irp
    //
    Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        //
        // wait for request completion
        //
        KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, null_mut());
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    let _ = Status;

    //
    // done
    //
    IoStatusBlock.__bindgen_anon_1.Status
}}

pub unsafe extern "system" fn HidUsb_ResetWorkerRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Ctx: PVOID,
) { unsafe {
    let mut Status: NTSTATUS;
    let mut PortStatus: ULONG = 0;
    let ResetContext: PHID_USB_RESET_CONTEXT;
    let DeviceExtension: PHID_DEVICE_EXTENSION;

    //
    // get context
    //
    ResetContext = Ctx as PHID_USB_RESET_CONTEXT;

    //
    // get device extension
    //
    DeviceExtension = (*(*ResetContext).DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // get port status
    //
    Status = HidUsb_GetPortStatus((*ResetContext).DeviceObject, &mut PortStatus);
    if nt_success(Status) {
        if (PortStatus & USB_PORT_STATUS_ENABLE) == 0 {
            //
            // port is disabled
            //
            Status = HidUsb_ResetInterruptPipe((*ResetContext).DeviceObject);
            let _ = Status;
        } else {
            //
            // abort pipe
            //
            Status = HidUsb_AbortPipe((*ResetContext).DeviceObject);
            if nt_success(Status) {
                //
                // reset port
                //
                Status = HidUsb_ResetPort((*ResetContext).DeviceObject);
                if Status == STATUS_DEVICE_DATA_ERROR {
                    //
                    // invalidate device state
                    //
                    IoInvalidateDeviceState((*DeviceExtension).PhysicalDeviceObject);
                }

                //
                // reset interrupt pipe
                //
                if nt_success(Status) {
                    //
                    // reset pipe
                    //
                    Status = HidUsb_ResetInterruptPipe((*ResetContext).DeviceObject);
                    let _ = Status;
                }
            }
        }
    }

    //
    // cleanup
    //
    debug_assert!(KeGetCurrentIrql() == PASSIVE_LEVEL);
    IoFreeWorkItem((*ResetContext).WorkItem);
    IoCompleteRequest((*ResetContext).Irp, IO_NO_INCREMENT);
    ExFreePoolWithTag(ResetContext as PVOID, HIDUSB_TAG);

    let _ = DeviceObject;
}}

pub unsafe extern "system" fn Hid_PnpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS { unsafe {
    //
    // signal event
    //
    KeSetEvent(Context as _, 0, FALSE);

    let _ = (DeviceObject, Irp);

    //
    // done
    //
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}}

pub unsafe fn Hid_DispatchUrb(DeviceObject: PDEVICE_OBJECT, Urb: PURB) -> NTSTATUS { unsafe {
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let DeviceExtension: PHID_DEVICE_EXTENSION;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let IoStack;
    let mut Status: NTSTATUS;

    //
    // init event
    //
    KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

    //
    // get device extension
    //
    DeviceExtension = (*DeviceObject).DeviceExtension as PHID_DEVICE_EXTENSION;

    //
    // build irp
    //
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_USB_SUBMIT_URB,
        (*DeviceExtension).NextDeviceObject,
        null_mut(),
        0,
        null_mut(),
        0,
        TRUE,
        &mut Event as *mut _ as _,
        &mut IoStatus,
    );
    if Irp.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // get next stack location
    //
    IoStack = IoGetNextIrpStackLocation(Irp);

    //
    // store urb
    //
    (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;

    //
    // set completion routine
    //
    type CompletionFn =
        unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS;
    IoSetCompletionRoutine(
        Irp,
        transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(Hid_PnpCompletion),
        &mut Event as *mut _ as _,
        TRUE,
        TRUE,
        TRUE,
    );

    //
    // call driver
    //
    Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);

    //
    // wait for the request to finish
    //
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(&mut Event as *mut _ as _, Executive, KernelMode, FALSE, null_mut());
    }

    //
    // complete request
    //
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    if Status == STATUS_PENDING {
        //
        // get final status
        //
        Status = IoStatus.__bindgen_anon_1.Status;
    }

    //
    // done
    //
    Status
}}
