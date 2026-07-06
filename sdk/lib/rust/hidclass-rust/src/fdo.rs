// fdo.rs — HID class FDO PnP + descriptor-fetch state machine.
// Faithful translation of drivers/hid/hidclass/fdo.c.

use core::mem::{size_of, transmute};
use core::ptr::null_mut;

use ntkrnl_sys::base::{NTSTATUS, PVOID, ULONG, TRUE, FALSE};
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::STATUS_NOT_SUPPORTED;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, _IRP, PIO_COMPLETION_ROUTINE,
    DEVICE_CAPABILITIES, PDEVICE_CAPABILITIES, PDEVICE_RELATIONS, _DEVICE_RELATIONS,
    KEVENT, _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _KWAIT_REASON_Executive as Executive,
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    IoAllocateIrp, IoFreeIrp, IoCreateDevice, IoDeleteDevice, IoAttachDeviceToDeviceStack,
    IoDetachDevice, KeInitializeEvent, KeSetEvent, KeWaitForSingleObject,
    ExAllocatePoolWithTag,
};
use ntkrnl_sys::ob::ObReferenceObject;
use ntkrnl_sys::constants::KernelMode;
use ntkrnl_sys::constants::IO_NO_INCREMENT;
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, IoCallDriver, IRP_MJ_PNP, IRP_MJ_INTERNAL_DEVICE_CONTROL,
    IRP_MN_START_DEVICE, IRP_MN_REMOVE_DEVICE, IRP_MN_QUERY_DEVICE_RELATIONS,
    IRP_MN_QUERY_REMOVE_DEVICE, IRP_MN_QUERY_STOP_DEVICE, IRP_MN_CANCEL_REMOVE_DEVICE,
    IRP_MN_CANCEL_STOP_DEVICE,
};
use ntkrnl_sys::mm::NonPagedPool;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine,
    IoSetNextIrpStackLocation, IoSkipCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext,
};

use crate::glue::{
    HIDCLASS_TAG, MAXULONG, HID_REPORT_DESCRIPTOR_TYPE,
    IOCTL_HID_GET_DEVICE_DESCRIPTOR, IOCTL_HID_GET_DEVICE_ATTRIBUTES,
    IOCTL_HID_GET_REPORT_DESCRIPTOR, HidP_GetCollectionDescription,
};
use crate::types::{
    PHIDCLASS_FDO_EXTENSION, PHIDCLASS_COMMON_DEVICE_EXTENSION,
    HID_DESCRIPTOR, HID_DEVICE_ATTRIBUTES,
};

// IRP_MN_QUERY_CAPABILITIES minor — see glue (not re-exported by constants).
use crate::glue::IRP_MN_QUERY_CAPABILITIES;

type CompletionFn =
    unsafe extern "system" fn(PDEVICE_OBJECT, *mut _IRP, PVOID) -> NTSTATUS;

// ===========================================================================
// HidClassFDO_QueryCapabilitiesCompletionRoutine
// ===========================================================================
unsafe extern "system" fn HidClassFDO_QueryCapabilitiesCompletionRoutine(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: *mut _IRP,
    Context: PVOID,
) -> NTSTATUS {
    // set event
    KeSetEvent(Context as *mut KEVENT, 0, FALSE);
    // completion is done in the HidClassFDO_QueryCapabilities routine
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}

// ===========================================================================
// HidClassFDO_QueryCapabilities
// ===========================================================================
pub unsafe fn HidClassFDO_QueryCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    Capabilities: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // init event
    KeInitializeEvent(&mut Event, NotificationEvent, FALSE);

    // now allocate the irp
    let Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, FALSE);
    if Irp.is_null() {
        // no memory
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // get next stack location
    let IoStack = IoGetNextIrpStackLocation(Irp);

    // init stack location
    (*IoStack).MajorFunction = IRP_MJ_PNP as u8;
    (*IoStack).MinorFunction = IRP_MN_QUERY_CAPABILITIES;
    (*IoStack).Parameters.DeviceCapabilities.Capabilities = Capabilities;

    // set completion routine
    IoSetCompletionRoutine(
        Irp,
        transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
            HidClassFDO_QueryCapabilitiesCompletionRoutine,
        ),
        &mut Event as *mut _ as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    // init capabilities
    RtlZeroMemory(Capabilities as PVOID, size_of::<DEVICE_CAPABILITIES>());
    (*Capabilities).Size = size_of::<DEVICE_CAPABILITIES>() as u16;
    (*Capabilities).Version = 1; // FIXME hardcoded constant
    (*Capabilities).Address = MAXULONG;
    (*Capabilities).UINumber = MAXULONG;

    // pnp irps have default completion code
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    // call lower device
    Status = IoCallDriver(
        (*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject,
        Irp,
    );
    if Status == ntkrnl_sys::constants::STATUS_PENDING {
        // wait for completion
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            Executive,
            KernelMode,
            FALSE,
            null_mut(),
        );
    }

    // get status
    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    // complete request
    IoFreeIrp(Irp);

    Status
}

// ===========================================================================
// HidClassFDO_DispatchRequestSynchronousCompletion
// ===========================================================================
unsafe extern "system" fn HidClassFDO_DispatchRequestSynchronousCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: *mut _IRP,
    Context: PVOID,
) -> NTSTATUS {
    // signal event
    KeSetEvent(Context as *mut KEVENT, 0, FALSE);
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}

// ===========================================================================
// HidClassFDO_DispatchRequestSynchronous
// ===========================================================================
pub unsafe fn HidClassFDO_DispatchRequestSynchronous(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    // init event
    KeInitializeEvent(&mut Event, NotificationEvent, FALSE);

    // get device extension
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    // set completion routine
    IoSetCompletionRoutine(
        Irp,
        transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
            HidClassFDO_DispatchRequestSynchronousCompletion,
        ),
        &mut Event as *mut _ as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    debug_assert!((*Irp).CurrentLocation as i32 > 0);
    // create stack location
    IoSetNextIrpStackLocation(Irp);

    // get next stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // store device object
    (*IoStack).DeviceObject = DeviceObject;

    // sanity check
    // ASSERT(CommonDeviceExtension->DriverExtension->MajorFunction[IoStack->MajorFunction] != NULL);

    // call minidriver (hidusb)
    let dispatch = (*(*CommonDeviceExtension).DriverExtension).MajorFunction
        [(*IoStack).MajorFunction as usize];
    Status = (dispatch.unwrap())(DeviceObject, Irp);

    // wait for the request to finish
    if Status == ntkrnl_sys::constants::STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            Executive,
            KernelMode,
            FALSE,
            null_mut(),
        );

        // update status
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    Status
}

// ===========================================================================
// HidClassFDO_DispatchRequest
// ===========================================================================
pub unsafe fn HidClassFDO_DispatchRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // get device extension
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    debug_assert!((*Irp).CurrentLocation as i32 > 0);

    // create stack location
    IoSetNextIrpStackLocation(Irp);

    // get next stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // store device object
    (*IoStack).DeviceObject = DeviceObject;

    // sanity check
    // ASSERT(CommonDeviceExtension->DriverExtension->MajorFunction[IoStack->MajorFunction] != NULL);

    // call driver
    let dispatch = (*(*CommonDeviceExtension).DriverExtension).MajorFunction
        [(*IoStack).MajorFunction as usize];
    (dispatch.unwrap())(DeviceObject, Irp)
}

// ===========================================================================
// HidClassFDO_GetDescriptors
// ===========================================================================
pub unsafe fn HidClassFDO_GetDescriptors(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // let's allocate irp
    let Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, FALSE);
    if Irp.is_null() {
        // no memory
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // get stack location
    let IoStack = IoGetNextIrpStackLocation(Irp);

    // init stack location
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_HID_GET_DEVICE_DESCRIPTOR;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = size_of::<HID_DESCRIPTOR>() as ULONG;
    (*IoStack).Parameters.DeviceIoControl.InputBufferLength = 0;
    (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer = null_mut();
    (*Irp).UserBuffer = &mut (*FDODeviceExtension).HidDescriptor as *mut _ as PVOID;

    // send request
    let mut Status = HidClassFDO_DispatchRequestSynchronous(DeviceObject, Irp);
    if !nt_success(Status) {
        // failed to get device descriptor
        IoFreeIrp(Irp);
        return Status;
    }

    // let's get device attributes
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_HID_GET_DEVICE_ATTRIBUTES;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength =
        size_of::<HID_DEVICE_ATTRIBUTES>() as ULONG;
    (*Irp).UserBuffer = &mut (*FDODeviceExtension).Common.Attributes as *mut _ as PVOID;

    // send request
    Status = HidClassFDO_DispatchRequestSynchronous(DeviceObject, Irp);
    if !nt_success(Status) {
        // failed to get device descriptor
        IoFreeIrp(Irp);
        return Status;
    }

    // sanity checks
    // ASSERT(HidDescriptor.bLength == sizeof(HID_DESCRIPTOR));
    // ASSERT(HidDescriptor.bNumDescriptors > 0);
    // ASSERT(DescriptorList[0].wReportLength > 0);
    // ASSERT(DescriptorList[0].bReportType == HID_REPORT_DESCRIPTOR_TYPE);
    let _ = HID_REPORT_DESCRIPTOR_TYPE;

    // now allocate space for the report descriptor
    let report_len = (*FDODeviceExtension).HidDescriptor.DescriptorList[0].wReportLength;
    (*FDODeviceExtension).ReportDescriptor =
        ExAllocatePoolWithTag(NonPagedPool, report_len as _, HIDCLASS_TAG) as *mut u8;
    if (*FDODeviceExtension).ReportDescriptor.is_null() {
        // not enough memory
        IoFreeIrp(Irp);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // init stack location
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_HID_GET_REPORT_DESCRIPTOR;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength = report_len as ULONG;
    (*Irp).UserBuffer = (*FDODeviceExtension).ReportDescriptor as PVOID;

    // send request
    Status = HidClassFDO_DispatchRequestSynchronous(DeviceObject, Irp);
    if !nt_success(Status) {
        // failed to get device descriptor
        IoFreeIrp(Irp);
        return Status;
    }

    // completed successfully
    IoFreeIrp(Irp);
    STATUS_SUCCESS
}

// ===========================================================================
// HidClassFDO_StartDevice
// ===========================================================================
pub unsafe fn HidClassFDO_StartDevice(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // query capabilities
    let mut Status =
        HidClassFDO_QueryCapabilities(DeviceObject, &mut (*FDODeviceExtension).Capabilities);
    if !nt_success(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // let's start the lower device too
    IoSkipCurrentIrpStackLocation(Irp);
    Status = HidClassFDO_DispatchRequestSynchronous(DeviceObject, Irp);
    if !nt_success(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // let's get the descriptors
    Status = HidClassFDO_GetDescriptors(DeviceObject);
    if !nt_success(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // now get the the collection description
    Status = HidP_GetCollectionDescription(
        (*FDODeviceExtension).ReportDescriptor,
        (*FDODeviceExtension).HidDescriptor.DescriptorList[0].wReportLength as ULONG,
        NonPagedPool,
        &mut (*FDODeviceExtension).Common.DeviceDescription,
    );
    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[HIDCLASS] GetCollectionDescription -> 0x%x collections=%u reportlen=%u\n\0".as_ptr() as *const i8,
        Status as u32,
        (*FDODeviceExtension).Common.DeviceDescription.CollectionDescLength,
        (*FDODeviceExtension).HidDescriptor.DescriptorList[0].wReportLength as u32,
    );
    if !nt_success(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // complete request
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

// ===========================================================================
// HidClassFDO_RemoveDevice
// ===========================================================================
pub unsafe fn HidClassFDO_RemoveDevice(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    /* FIXME cleanup */

    // dispatch to minidriver
    IoSkipCurrentIrpStackLocation(Irp);
    let Status = HidClassFDO_DispatchRequestSynchronous(DeviceObject, Irp);

    // complete request
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    // detach and delete device
    IoDetachDevice((*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject);
    IoDeleteDevice(DeviceObject);

    Status
}

// ===========================================================================
// HidClassFDO_CopyDeviceRelations
// ===========================================================================
pub unsafe fn HidClassFDO_CopyDeviceRelations(
    DeviceObject: PDEVICE_OBJECT,
    OutRelations: *mut PDEVICE_RELATIONS,
) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // allocate result
    let count = (*(*FDODeviceExtension).DeviceRelations).Count;
    let DeviceRelations = ExAllocatePoolWithTag(
        NonPagedPool,
        (size_of::<_DEVICE_RELATIONS>() + (count as usize - 1) * size_of::<PDEVICE_OBJECT>())
            as _,
        HIDCLASS_TAG,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        // no memory
        *OutRelations = null_mut();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // copy device objects
    let mut Index: ULONG = 0;
    while Index < count {
        // reference pdo
        let obj = *(*(*FDODeviceExtension).DeviceRelations)
            .Objects
            .as_ptr()
            .add(Index as usize);
        ObReferenceObject(obj as PVOID);

        // store object
        *(*DeviceRelations).Objects.as_mut_ptr().add(Index as usize) = obj;

        Index += 1;
    }

    // set object count
    (*DeviceRelations).Count = (*(*FDODeviceExtension).DeviceRelations).Count;

    // store result
    *OutRelations = DeviceRelations;
    STATUS_SUCCESS
}

// ===========================================================================
// HidClassFDO_DeviceRelations
// ===========================================================================
pub unsafe fn HidClassFDO_DeviceRelations(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // get current irp stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // check relations type
    if (*IoStack).Parameters.QueryDeviceRelations.Type != BusRelations {
        // only bus relations are handled
        IoSkipCurrentIrpStackLocation(Irp);
        return IoCallDriver(
            (*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject,
            Irp,
        );
    }

    if (*FDODeviceExtension).DeviceRelations.is_null() {
        // time to create the pdos
        let Status =
            crate::pdo::HidClassPDO_CreatePDO(DeviceObject, &mut (*FDODeviceExtension).DeviceRelations);
        if !nt_success(Status) {
            // failed
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_SUCCESS;
        }
        // sanity check
        // ASSERT(FDODeviceExtension->DeviceRelations->Count > 0);
    }

    // now copy device relations
    let mut DeviceRelations: PDEVICE_RELATIONS = null_mut();
    let Status = HidClassFDO_CopyDeviceRelations(DeviceObject, &mut DeviceRelations);
    // store result
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = DeviceRelations as _;

    // complete request
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

// ===========================================================================
// HidClassFDO_PnP
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn HidClassFDO_PnP(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    // get device extension
    let FDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;
    debug_assert!((*FDODeviceExtension).Common.IsFDO != 0);

    // get current irp stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        x if x == IRP_MN_START_DEVICE => HidClassFDO_StartDevice(DeviceObject, Irp),
        x if x == IRP_MN_REMOVE_DEVICE => HidClassFDO_RemoveDevice(DeviceObject, Irp),
        x if x == IRP_MN_QUERY_DEVICE_RELATIONS => HidClassFDO_DeviceRelations(DeviceObject, Irp),
        x if x == IRP_MN_QUERY_REMOVE_DEVICE
            || x == IRP_MN_QUERY_STOP_DEVICE
            || x == IRP_MN_CANCEL_REMOVE_DEVICE
            || x == IRP_MN_CANCEL_STOP_DEVICE =>
        {
            // set status to success and fall through
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            // default:
            IoCopyCurrentIrpStackLocationToNext(Irp);
            HidClassFDO_DispatchRequest(DeviceObject, Irp)
        }
        _ => {
            // dispatch to mini driver
            IoCopyCurrentIrpStackLocationToNext(Irp);
            HidClassFDO_DispatchRequest(DeviceObject, Irp)
        }
    }
}

#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}
