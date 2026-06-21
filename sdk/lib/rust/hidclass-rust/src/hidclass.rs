// hidclass.rs — HID class main: HidRegisterMinidriver (exported), AddDevice,
//               the IRP_MJ_* dispatch, create/close fileop, read-report path +
//               completion + the HidClass_BuildIrp/GetIrp helpers.
// Faithful translation of drivers/hid/hidclass/hidclass.c.

use core::mem::{size_of, transmute};
use core::ptr::null_mut;

use ntkrnl_sys::base::{NTSTATUS, PVOID, ULONG, UCHAR, BOOLEAN, KIRQL, TRUE, FALSE};
use ntkrnl_sys::base::{
    STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED, STATUS_NO_MEMORY,
};
use ntkrnl_sys::constants::{
    KernelMode, STATUS_PENDING, STATUS_CANCELLED, STATUS_UNSUCCESSFUL, STATUS_INVALID_PARAMETER_1,
    STATUS_INVALID_PARAMETER, STATUS_REVISION_MISMATCH, IO_NO_INCREMENT,
    IRP_MJ_CREATE, IRP_MJ_CLOSE, IRP_MJ_READ, IRP_MJ_WRITE,
};
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, IoCallDriver,
    IRP_MJ_DEVICE_CONTROL, IRP_MJ_INTERNAL_DEVICE_CONTROL, IRP_MJ_POWER, IRP_MJ_PNP,
};
use ntkrnl_sys::ddk::{
    PDRIVER_OBJECT, PDEVICE_OBJECT, PIRP, _IRP, PVOID as DdkPVOID,
    PDRIVER_DISPATCH, PDRIVER_ADD_DEVICE, PDRIVER_UNLOAD, PIO_COMPLETION_ROUTINE,
    KEVENT, KSPIN_LOCK, IO_STATUS_BLOCK,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _EVENT_TYPE_SynchronizationEvent as SynchronizationEvent,
    _KWAIT_REASON_Executive as Executive,
    IoCreateDevice, IoDeleteDevice, IoAttachDeviceToDeviceStack, IoDetachDevice,
    IoAllocateIrp, IoFreeIrp, IoReuseIrp, IoBuildDeviceIoControlRequest,
    IoGetDriverObjectExtension, IoAllocateDriverObjectExtension,
    KeInitializeEvent, KeSetEvent, KeWaitForSingleObject,
    ExAllocatePoolWithTag, ExFreePoolWithTag, RtlInitUnicodeString,
};
use ntkrnl_sys::driver_ddk::{PoStartNextPowerIrp, NormalPagePriority, MmGetSystemAddressForMdlSafe};
use ntkrnl_sys::mm::NonPagedPool;
use ntkrnl_sys::rtl::{RtlZeroMemory, RtlCopyMemory};
use ntkrnl_sys::interlocked::InterlockedIncrement;
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::base::{
    InitializeListHead, IsListEmpty, RemoveHeadList, InsertTailList, RemoveEntryList,
};
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine,
    IoSetNextIrpStackLocation, IoMarkIrpPending, IoCopyCurrentIrpStackLocationToNext,
    IoSkipCurrentIrpStackLocation, CONTAINING_RECORD,
};

use crate::glue::{
    HIDCLASS_TAG, HID_REVISION, IOCTL_HID_READ_REPORT, IOCTL_HID_WRITE_REPORT,
    IOCTL_HID_GET_COLLECTION_INFORMATION, IOCTL_HID_GET_COLLECTION_DESCRIPTOR,
    IOCTL_HID_GET_FEATURE, IOCTL_HID_SET_FEATURE,
    DO_BUFFERED_IO, DO_POWER_PAGABLE, DO_DEVICE_INITIALIZING, FILE_DEVICE_UNKNOWN,
};
use crate::types::{
    PHIDCLASS_DRIVER_EXTENSION, HIDCLASS_DRIVER_EXTENSION,
    PHIDCLASS_COMMON_DEVICE_EXTENSION, PHIDCLASS_FDO_EXTENSION,
    PHIDCLASS_PDO_DEVICE_EXTENSION, PHIDCLASS_FILEOP_CONTEXT, HIDCLASS_FILEOP_CONTEXT,
    PHIDCLASS_IRP_CONTEXT, HIDCLASS_IRP_CONTEXT,
    PHID_MINIDRIVER_REGISTRATION, HID_COLLECTION_INFORMATION, PHID_COLLECTION_INFORMATION,
    HID_XFER_PACKET,
    HIDCLASS_FDO_EXTENSION,
};
use crate::pdo::{
    HidClassPDO_GetCollectionDescription, HidClassPDO_GetReportDescription,
    HidClassPDO_GetReportDescriptionByReportID,
};
use crate::fdo::HidClassFDO_DispatchRequest;

// Client identification address — the C uses the &ClientIdentificationAddress
// pointer (a static LPWSTR) as the IoGetDriverObjectExtension client id.
static mut CLIENT_IDENTIFICATION_ADDRESS: [u16; 9] =
    [b'H' as u16, b'I' as u16, b'D' as u16, b'C' as u16, b'L' as u16, b'A' as u16, b'S' as u16, b'S' as u16, 0];

static mut HID_CLASS_DEVICE_NUMBER: ULONG = 0;

type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, *mut _IRP) -> NTSTATUS;
type AddDeviceFn = unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS;
type UnloadFn = unsafe extern "system" fn(PDRIVER_OBJECT);
type CompletionFn = unsafe extern "system" fn(PDEVICE_OBJECT, *mut _IRP, PVOID) -> NTSTATUS;

#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

// Compile-time L"..." → null-terminated [u16; N+1], yielding a *const u16.
macro_rules! wstr {
    ($s:expr) => {{
        const S: &str = $s;
        const N: usize = S.len();
        const ARR: [u16; N + 1] = {
            let bytes = S.as_bytes();
            let mut a = [0u16; N + 1];
            let mut i = 0;
            while i < N {
                a[i] = bytes[i] as u16;
                i += 1;
            }
            a
        };
        ARR.as_ptr()
    }};
}

// ===========================================================================
// DllInitialize / DllUnload
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn DllInitialize(_RegistryPath: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DllUnload() -> NTSTATUS {
    STATUS_SUCCESS
}

// ===========================================================================
// HidClassAddDevice
// ===========================================================================
unsafe extern "system" fn HidClassAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut CharDeviceName: [u16; 64] = [0; 64];
    let mut DeviceName: ntkrnl_sys::ddk::UNICODE_STRING = core::mem::zeroed();
    let mut NewDeviceObject: PDEVICE_OBJECT = null_mut();

    // increment device number
    InterlockedIncrement(core::ptr::addr_of_mut!(HID_CLASS_DEVICE_NUMBER) as *mut i32);

    // construct device name
    swprintf_wstr(
        CharDeviceName.as_mut_ptr(),
        wstr!("\\Device\\_HID%08x"),
        HID_CLASS_DEVICE_NUMBER,
    );

    // initialize device name
    RtlInitUnicodeString(&mut DeviceName, CharDeviceName.as_ptr());

    // get driver object extension
    let DriverExtension = IoGetDriverObjectExtension(
        DriverObject,
        core::ptr::addr_of_mut!(CLIENT_IDENTIFICATION_ADDRESS) as PVOID,
    ) as PHIDCLASS_DRIVER_EXTENSION;
    if DriverExtension.is_null() {
        // device removed
        return crate::glue::STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    // calculate device extension size
    let DeviceExtensionSize: ULONG =
        size_of::<HIDCLASS_FDO_EXTENSION>() as ULONG + (*DriverExtension).DeviceExtensionSize;

    // now create the device
    let mut Status = IoCreateDevice(
        DriverObject,
        DeviceExtensionSize,
        &mut DeviceName,
        FILE_DEVICE_UNKNOWN,
        0,
        FALSE,
        &mut NewDeviceObject,
    );
    if !nt_success(Status) {
        // failed to create device object
        return Status;
    }

    // get device extension
    let FDODeviceExtension = (*NewDeviceObject).DeviceExtension as PHIDCLASS_FDO_EXTENSION;

    // zero device extension
    RtlZeroMemory(
        FDODeviceExtension as PVOID,
        size_of::<HIDCLASS_FDO_EXTENSION>(),
    );

    // initialize device extension
    (*FDODeviceExtension).Common.IsFDO = TRUE;
    (*FDODeviceExtension).Common.DriverExtension = DriverExtension;
    (*FDODeviceExtension).Common.HidDeviceExtension.PhysicalDeviceObject = PhysicalDeviceObject;
    (*FDODeviceExtension).Common.HidDeviceExtension.MiniDeviceExtension =
        (FDODeviceExtension as usize + size_of::<HIDCLASS_FDO_EXTENSION>()) as PVOID;
    (*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject =
        IoAttachDeviceToDeviceStack(NewDeviceObject, PhysicalDeviceObject);
    if (*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject.is_null() {
        // no PDO
        IoDeleteDevice(NewDeviceObject);
        return crate::glue::STATUS_DEVICE_REMOVED;
    }

    // sanity check
    // ASSERT(NextDeviceObject);

    // increment stack size
    (*NewDeviceObject).StackSize += 1;

    // init device object
    (*NewDeviceObject).Flags |= DO_BUFFERED_IO | DO_POWER_PAGABLE;
    (*NewDeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    // now call driver provided add device routine
    // ASSERT(DriverExtension->AddDevice != 0);
    Status = ((*DriverExtension).AddDevice.unwrap())(DriverObject, NewDeviceObject);
    if !nt_success(Status) {
        // failed
        IoDetachDevice((*FDODeviceExtension).Common.HidDeviceExtension.NextDeviceObject);
        IoDeleteDevice(NewDeviceObject);
        return Status;
    }

    // succeeded
    Status
}

// ===========================================================================
// HidClassDriverUnload
// ===========================================================================
unsafe extern "system" fn HidClassDriverUnload(_DriverObject: PDRIVER_OBJECT) {
    // UNIMPLEMENTED;
}

// ===========================================================================
// HidClass_Create
// ===========================================================================
unsafe extern "system" fn HidClass_Create(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    // get device extension
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;
    if (*CommonDeviceExtension).IsFDO != 0 {
        // only supported for PDO
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_UNSUCCESSFUL;
    }

    // must be a PDO
    let PDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_PDO_DEVICE_EXTENSION;

    // get stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // allocate context
    let Context = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<HIDCLASS_FILEOP_CONTEXT>() as _,
        HIDCLASS_TAG,
    ) as PHIDCLASS_FILEOP_CONTEXT;
    if Context.is_null() {
        // no memory
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // init context
    RtlZeroMemory(Context as PVOID, size_of::<HIDCLASS_FILEOP_CONTEXT>());
    (*Context).DeviceExtension = PDODeviceExtension;
    KeInitializeSpinLock(&mut (*Context).Lock as *mut _ as *mut _);
    InitializeListHead(&mut (*Context).ReadPendingIrpListHead);
    InitializeListHead(&mut (*Context).IrpCompletedListHead);
    KeInitializeEvent(&mut (*Context).IrpReadComplete, NotificationEvent, FALSE);

    // store context
    // ASSERT(IoStack->FileObject);
    (*(*IoStack).FileObject).FsContext = Context as PVOID;

    // done
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_SUCCESS
}

// ===========================================================================
// HidClass_Close
// ===========================================================================
unsafe extern "system" fn HidClass_Close(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let mut IsRequestPending: BOOLEAN = FALSE;
    let mut OldLevel: KIRQL = 0;

    // get device extension
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    // is it a FDO request
    if (*CommonDeviceExtension).IsFDO != 0 {
        // how did the request get there
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER_1;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INVALID_PARAMETER_1;
    }

    // get stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // get irp context
    let IrpContext = (*(*IoStack).FileObject).FsContext as PHIDCLASS_FILEOP_CONTEXT;

    // acquire lock
    KeAcquireSpinLock(&mut (*IrpContext).Lock as *mut _ as *mut _, &mut OldLevel);

    if !IsListEmpty(&mut (*IrpContext).ReadPendingIrpListHead) {
        // FIXME cancel irp
        IsRequestPending = TRUE;
    }

    // signal stop
    (*IrpContext).StopInProgress = TRUE;

    // release lock
    KeReleaseSpinLock(&mut (*IrpContext).Lock as *mut _ as *mut _, OldLevel);

    if IsRequestPending != 0 {
        // wait for request to complete
        KeWaitForSingleObject(
            &mut (*IrpContext).IrpReadComplete as *mut _ as PVOID,
            Executive,
            KernelMode,
            FALSE,
            null_mut(),
        );
    }

    // acquire lock
    KeAcquireSpinLock(&mut (*IrpContext).Lock as *mut _ as *mut _, &mut OldLevel);

    // sanity check
    // ASSERT(IsListEmpty(&ReadPendingIrpListHead));

    // now free all irps
    while !IsListEmpty(&mut (*IrpContext).IrpCompletedListHead) {
        // remove head irp
        let Entry = RemoveHeadList(&mut (*IrpContext).IrpCompletedListHead);

        // get irp (CONTAINING_RECORD(Entry, IRP, Tail.Overlay.ListEntry))
        let ListIrp = CONTAINING_RECORD(
            Entry as PVOID,
            size_of::<_IRP>(),
            irp_tail_overlay_listentry_offset(),
        ) as PIRP;

        // free the irp
        IoFreeIrp(ListIrp);
    }

    // release lock
    KeReleaseSpinLock(&mut (*IrpContext).Lock as *mut _ as *mut _, OldLevel);

    // remove context
    (*(*IoStack).FileObject).FsContext = null_mut();

    // free context
    ExFreePoolWithTag(IrpContext as PVOID, HIDCLASS_TAG);

    // complete request
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_SUCCESS
}

// ===========================================================================
// HidClass_ReadCompleteIrp — IO completion routine for the dispatched read IRP.
// ===========================================================================
unsafe extern "system" fn HidClass_ReadCompleteIrp(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: *mut _IRP,
    Ctx: PVOID,
) -> NTSTATUS {
    let mut OldLevel: KIRQL = 0;
    let mut Offset: ULONG;

    // get irp context
    let IrpContext = Ctx as PHIDCLASS_IRP_CONTEXT;

    // copy result
    if (*Irp).IoStatus.Information != 0 {
        // get address
        let Address = MmGetSystemAddressForMdlSafe(
            (*(*IrpContext).OriginalIrp).MdlAddress,
            NormalPagePriority,
        ) as *mut UCHAR;
        if !Address.is_null() {
            // reports may have a report id prepended
            Offset = 0;

            // get collection description
            let CollectionDescription = HidClassPDO_GetCollectionDescription(
                &mut (*(*(*IrpContext).FileOp).DeviceExtension).Common.DeviceDescription,
                (*(*(*IrpContext).FileOp).DeviceExtension).CollectionNumber,
            );

            // get report description
            let ReportDescription = HidClassPDO_GetReportDescription(
                &mut (*(*(*IrpContext).FileOp).DeviceExtension).Common.DeviceDescription,
                (*(*(*IrpContext).FileOp).DeviceExtension).CollectionNumber,
            );

            if !CollectionDescription.is_null() && !ReportDescription.is_null() {
                // calculate offset
                // ASSERT(InputLength >= InputLength);
                Offset = (*CollectionDescription).InputLength as ULONG
                    - (*ReportDescription).InputLength as ULONG;
            }

            // copy result
            RtlCopyMemory(
                Address.add(Offset as usize) as PVOID,
                (*IrpContext).InputReportBuffer,
                (*IrpContext).InputReportBufferLength as usize,
            );
        }
    }

    // copy result status
    (*(*IrpContext).OriginalIrp).IoStatus.__bindgen_anon_1.Status =
        (*Irp).IoStatus.__bindgen_anon_1.Status;
    (*(*IrpContext).OriginalIrp).IoStatus.Information = (*Irp).IoStatus.Information;

    // free input report buffer
    ExFreePoolWithTag((*IrpContext).InputReportBuffer, HIDCLASS_TAG);

    // remove us from pending list
    KeAcquireSpinLock(&mut (*(*IrpContext).FileOp).Lock as *mut _ as *mut _, &mut OldLevel);

    // remove from pending list
    RemoveEntryList(irp_listentry(Irp));

    // is list empty
    let IsEmpty = IsListEmpty(&mut (*(*IrpContext).FileOp).ReadPendingIrpListHead);

    // insert into completed list
    InsertTailList(
        &mut (*(*IrpContext).FileOp).IrpCompletedListHead,
        irp_listentry(Irp),
    );

    // release lock
    KeReleaseSpinLock(&mut (*(*IrpContext).FileOp).Lock as *mut _ as *mut _, OldLevel);

    // complete original request
    IoCompleteRequest((*IrpContext).OriginalIrp, IO_NO_INCREMENT);

    if (*(*IrpContext).FileOp).StopInProgress != 0 && IsEmpty {
        // last pending irp
        KeSetEvent(&mut (*(*IrpContext).FileOp).IrpReadComplete, 0, FALSE);
    }

    if (*(*IrpContext).FileOp).StopInProgress != 0 && IsEmpty {
        // last pending irp
        KeSetEvent(&mut (*(*IrpContext).FileOp).IrpReadComplete, 0, FALSE);
    }

    // free irp context
    ExFreePoolWithTag(IrpContext as PVOID, HIDCLASS_TAG);

    // done
    crate::glue::STATUS_MORE_PROCESSING_REQUIRED
}

// ===========================================================================
// HidClass_GetIrp
// ===========================================================================
unsafe fn HidClass_GetIrp(Context: PHIDCLASS_FILEOP_CONTEXT) -> PIRP {
    let mut OldLevel: KIRQL = 0;
    let mut Irp: PIRP = null_mut();

    // acquire lock
    KeAcquireSpinLock(&mut (*Context).Lock as *mut _ as *mut _, &mut OldLevel);

    // is list empty?
    if !IsListEmpty(&mut (*Context).IrpCompletedListHead) {
        // grab first entry
        let ListEntry = RemoveHeadList(&mut (*Context).IrpCompletedListHead);

        // get irp
        Irp = CONTAINING_RECORD(
            ListEntry as PVOID,
            size_of::<_IRP>(),
            irp_tail_overlay_listentry_offset(),
        ) as PIRP;
    }

    // release lock
    KeReleaseSpinLock(&mut (*Context).Lock as *mut _ as *mut _, OldLevel);

    Irp
}

// ===========================================================================
// HidClass_BuildIrp
// ===========================================================================
unsafe fn HidClass_BuildIrp(
    DeviceObject: PDEVICE_OBJECT,
    RequestIrp: PIRP,
    Context: PHIDCLASS_FILEOP_CONTEXT,
    DeviceIoControlCode: ULONG,
    _BufferLength: ULONG,
    OutIrp: *mut PIRP,
    OutIrpContext: *mut PHIDCLASS_IRP_CONTEXT,
) -> NTSTATUS {
    // get an irp from fresh list
    let mut Irp = HidClass_GetIrp(Context);
    if Irp.is_null() {
        // build new irp
        Irp = IoAllocateIrp((*DeviceObject).StackSize, FALSE);
        if Irp.is_null() {
            // no memory
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        // re-use irp
        IoReuseIrp(Irp, STATUS_SUCCESS);
    }

    // allocate completion context
    let IrpContext = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<HIDCLASS_IRP_CONTEXT>() as _,
        HIDCLASS_TAG,
    ) as PHIDCLASS_IRP_CONTEXT;
    if IrpContext.is_null() {
        // no memory
        IoFreeIrp(Irp);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // get device extension
    let _PDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_PDO_DEVICE_EXTENSION;

    // init irp context
    RtlZeroMemory(IrpContext as PVOID, size_of::<HIDCLASS_IRP_CONTEXT>());
    (*IrpContext).OriginalIrp = RequestIrp;
    (*IrpContext).FileOp = Context;

    // get collection description
    let _CollectionDescription = HidClassPDO_GetCollectionDescription(
        &mut (*(*(*IrpContext).FileOp).DeviceExtension).Common.DeviceDescription,
        (*(*(*IrpContext).FileOp).DeviceExtension).CollectionNumber,
    );

    // get report description
    let ReportDescription = HidClassPDO_GetReportDescription(
        &mut (*(*(*IrpContext).FileOp).DeviceExtension).Common.DeviceDescription,
        (*(*(*IrpContext).FileOp).DeviceExtension).CollectionNumber,
    );

    // sanity check
    // ASSERT(CollectionDescription->InputLength >= ReportDescription->InputLength);

    if (*Context).StopInProgress != 0 {
        // stop in progress
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_CANCELLED;
    }

    // store report length
    (*IrpContext).InputReportBufferLength = (*ReportDescription).InputLength as ULONG;

    // allocate buffer
    (*IrpContext).InputReportBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (*IrpContext).InputReportBufferLength as _,
        HIDCLASS_TAG,
    );
    if (*IrpContext).InputReportBuffer.is_null() {
        // no memory
        IoFreeIrp(Irp);
        ExFreePoolWithTag(IrpContext as PVOID, HIDCLASS_TAG);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // get stack location
    let IoStack = IoGetNextIrpStackLocation(Irp);

    // init stack location
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = DeviceIoControlCode;
    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength =
        (*IrpContext).InputReportBufferLength;
    (*IoStack).Parameters.DeviceIoControl.InputBufferLength = 0;
    (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer = null_mut();
    (*Irp).UserBuffer = (*IrpContext).InputReportBuffer;
    (*IoStack).DeviceObject = DeviceObject;

    // store result
    *OutIrp = Irp;
    *OutIrpContext = IrpContext;

    STATUS_SUCCESS
}

// ===========================================================================
// HidClass_Read
// ===========================================================================
unsafe extern "system" fn HidClass_Read(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let mut OldLevel: KIRQL = 0;
    let mut NewIrp: PIRP = null_mut();
    let mut NewIrpContext: PHIDCLASS_IRP_CONTEXT = null_mut();

    // get current stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // get device extension
    let _CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    // get context
    let Context = (*(*IoStack).FileObject).FsContext as PHIDCLASS_FILEOP_CONTEXT;

    // FIXME support polled devices
    // ASSERT(DevicesArePolled == FALSE);

    if (*Context).StopInProgress != 0 {
        // stop in progress
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_CANCELLED;
    }

    // build irp request
    let Status = HidClass_BuildIrp(
        DeviceObject,
        Irp,
        Context,
        IOCTL_HID_READ_REPORT,
        (*IoStack).Parameters.Read.Length,
        &mut NewIrp,
        &mut NewIrpContext,
    );
    if !nt_success(Status) {
        // failed
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // acquire lock
    KeAcquireSpinLock(&mut (*Context).Lock as *mut _ as *mut _, &mut OldLevel);

    // insert irp into pending list
    InsertTailList(&mut (*Context).ReadPendingIrpListHead, irp_listentry(NewIrp));

    // set completion routine
    IoSetCompletionRoutine(
        NewIrp,
        transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(HidClass_ReadCompleteIrp),
        NewIrpContext as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    // make next location current
    IoSetNextIrpStackLocation(NewIrp);

    // release spin lock
    KeReleaseSpinLock(&mut (*Context).Lock as *mut _ as *mut _, OldLevel);

    // mark irp pending
    IoMarkIrpPending(Irp);

    // let's dispatch the request
    let dispatch = (*(*(*Context).DeviceExtension).Common.DriverExtension).MajorFunction
        [IRP_MJ_INTERNAL_DEVICE_CONTROL as usize];
    (dispatch.unwrap())((*(*Context).DeviceExtension).FDODeviceObject, NewIrp);

    // complete
    STATUS_PENDING
}

// ===========================================================================
// HidClass_Write
// ===========================================================================
unsafe extern "system" fn HidClass_Write(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut XferPacket: HID_XFER_PACKET = core::mem::zeroed();

    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let Length = (*IoStack).Parameters.Write.Length;
    if Length < 1 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INVALID_PARAMETER;
    }

    RtlZeroMemory(&mut XferPacket as *mut _ as PVOID, size_of::<HID_XFER_PACKET>());
    XferPacket.reportBufferLen = Length;
    XferPacket.reportBuffer = (*Irp).UserBuffer as *mut UCHAR;
    XferPacket.reportId = *XferPacket.reportBuffer.add(0);

    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;
    let SubIrp = IoBuildDeviceIoControlRequest(
        IOCTL_HID_WRITE_REPORT,
        (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
        null_mut(),
        0,
        null_mut(),
        0,
        TRUE,
        &mut Event,
        &mut IoStatusBlock,
    );
    if SubIrp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NOT_IMPLEMENTED;
    }
    (*SubIrp).UserBuffer = &mut XferPacket as *mut _ as PVOID;
    KeInitializeEvent(&mut Event, SynchronizationEvent, FALSE);
    let mut Status = IoCallDriver(
        (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
        SubIrp,
    );
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            Executive,
            KernelMode,
            FALSE,
            null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

// ===========================================================================
// HidClass_DeviceControl
// ===========================================================================
unsafe extern "system" fn HidClass_DeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut _IRP,
) -> NTSTATUS {
    // get device extension
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    // only PDO are supported
    if (*CommonDeviceExtension).IsFDO != 0 {
        // invalid request
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER_1;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INVALID_PARAMETER_1;
    }

    // get pdo device extension
    let PDODeviceExtension = (*DeviceObject).DeviceExtension as PHIDCLASS_PDO_DEVICE_EXTENSION;

    // get stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*IoStack).Parameters.DeviceIoControl.IoControlCode {
        x if x == IOCTL_HID_GET_COLLECTION_INFORMATION => {
            // check if output buffer is big enough
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                < size_of::<HID_COLLECTION_INFORMATION>() as ULONG
            {
                // invalid buffer size
                (*Irp).IoStatus.__bindgen_anon_1.Status = crate::glue::STATUS_INVALID_BUFFER_SIZE;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return crate::glue::STATUS_INVALID_BUFFER_SIZE;
            }

            // get output buffer
            let CollectionInformation =
                (*Irp).AssociatedIrp.SystemBuffer as PHID_COLLECTION_INFORMATION;

            // get collection description
            let CollectionDescription = HidClassPDO_GetCollectionDescription(
                &mut (*CommonDeviceExtension).DeviceDescription,
                (*PDODeviceExtension).CollectionNumber,
            );

            // init result buffer
            (*CollectionInformation).DescriptorSize =
                (*CollectionDescription).PreparsedDataLength as ULONG;
            (*CollectionInformation).Polled =
                (*(*CommonDeviceExtension).DriverExtension).DevicesArePolled;
            (*CollectionInformation).VendorID = (*CommonDeviceExtension).Attributes.VendorID;
            (*CollectionInformation).ProductID = (*CommonDeviceExtension).Attributes.ProductID;
            (*CollectionInformation).VersionNumber =
                (*CommonDeviceExtension).Attributes.VersionNumber;

            // complete request
            (*Irp).IoStatus.Information = size_of::<HID_COLLECTION_INFORMATION>() as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_SUCCESS
        }
        x if x == IOCTL_HID_GET_COLLECTION_DESCRIPTOR => {
            // get collection description
            let CollectionDescription = HidClassPDO_GetCollectionDescription(
                &mut (*CommonDeviceExtension).DeviceDescription,
                (*PDODeviceExtension).CollectionNumber,
            );

            // check if output buffer is big enough
            if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                < (*CollectionDescription).PreparsedDataLength as ULONG
            {
                // invalid buffer size
                (*Irp).IoStatus.__bindgen_anon_1.Status = crate::glue::STATUS_INVALID_BUFFER_SIZE;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return crate::glue::STATUS_INVALID_BUFFER_SIZE;
            }

            // copy result
            RtlCopyMemory(
                (*Irp).UserBuffer,
                (*CollectionDescription).PreparsedData as PVOID,
                (*CollectionDescription).PreparsedDataLength as usize,
            );

            // complete request
            (*Irp).IoStatus.Information = (*CollectionDescription).PreparsedDataLength as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_SUCCESS
        }
        x if x == IOCTL_HID_GET_FEATURE => {
            let mut Event: KEVENT = core::mem::zeroed();
            let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
            let mut XferPacket: HID_XFER_PACKET = core::mem::zeroed();

            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < 1 {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }
            let ReportDescription = HidClassPDO_GetReportDescriptionByReportID(
                &mut (*PDODeviceExtension).Common.DeviceDescription,
                *((*Irp).AssociatedIrp.SystemBuffer as *mut UCHAR).add(0),
            );
            if ReportDescription.is_null()
                || (*IoStack).Parameters.DeviceIoControl.OutputBufferLength
                    < (*ReportDescription).FeatureLength as ULONG
            {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }

            RtlZeroMemory(&mut XferPacket as *mut _ as PVOID, size_of::<HID_XFER_PACKET>());
            XferPacket.reportBufferLen = (*ReportDescription).FeatureLength as ULONG;
            XferPacket.reportBuffer =
                MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority) as *mut UCHAR;
            XferPacket.reportId = *((*Irp).AssociatedIrp.SystemBuffer as *mut UCHAR).add(0);
            if XferPacket.reportBuffer.is_null() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            let SubIrp = IoBuildDeviceIoControlRequest(
                IOCTL_HID_GET_FEATURE,
                (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
                null_mut(),
                0,
                null_mut(),
                0,
                TRUE,
                &mut Event,
                &mut IoStatusBlock,
            );
            if SubIrp.is_null() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_NOT_IMPLEMENTED;
            }
            (*SubIrp).UserBuffer = &mut XferPacket as *mut _ as PVOID;
            KeInitializeEvent(&mut Event, SynchronizationEvent, FALSE);
            let mut Status = IoCallDriver(
                (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
                SubIrp,
            );
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    &mut Event as *mut _ as PVOID,
                    Executive,
                    KernelMode,
                    FALSE,
                    null_mut(),
                );
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            Status
        }
        x if x == IOCTL_HID_SET_FEATURE => {
            let mut Event: KEVENT = core::mem::zeroed();
            let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
            let mut XferPacket: HID_XFER_PACKET = core::mem::zeroed();

            if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < 1 {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }
            let ReportDescription = HidClassPDO_GetReportDescriptionByReportID(
                &mut (*PDODeviceExtension).Common.DeviceDescription,
                *((*Irp).AssociatedIrp.SystemBuffer as *mut UCHAR).add(0),
            );
            if ReportDescription.is_null()
                || (*IoStack).Parameters.DeviceIoControl.InputBufferLength
                    < (*ReportDescription).FeatureLength as ULONG
            {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_INVALID_PARAMETER;
            }

            RtlZeroMemory(&mut XferPacket as *mut _ as PVOID, size_of::<HID_XFER_PACKET>());
            XferPacket.reportBufferLen = (*ReportDescription).FeatureLength as ULONG;
            XferPacket.reportBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut UCHAR;
            XferPacket.reportId = *XferPacket.reportBuffer.add(0);

            let SubIrp = IoBuildDeviceIoControlRequest(
                IOCTL_HID_SET_FEATURE,
                (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
                null_mut(),
                0,
                null_mut(),
                0,
                TRUE,
                &mut Event,
                &mut IoStatusBlock,
            );
            if SubIrp.is_null() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_NOT_IMPLEMENTED;
            }
            (*SubIrp).UserBuffer = &mut XferPacket as *mut _ as PVOID;
            KeInitializeEvent(&mut Event, SynchronizationEvent, FALSE);
            let mut Status = IoCallDriver(
                (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
                SubIrp,
            );
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    &mut Event as *mut _ as PVOID,
                    Executive,
                    KernelMode,
                    FALSE,
                    null_mut(),
                );
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            Status
        }
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            STATUS_NOT_IMPLEMENTED
        }
    }
}

// ===========================================================================
// HidClass_InternalDeviceControl
// ===========================================================================
unsafe extern "system" fn HidClass_InternalDeviceControl(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: *mut _IRP,
) -> NTSTATUS {
    // UNIMPLEMENTED; ASSERT(FALSE);
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_NOT_IMPLEMENTED
}

// ===========================================================================
// HidClass_Power
// ===========================================================================
unsafe extern "system" fn HidClass_Power(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    if (*CommonDeviceExtension).IsFDO != 0 {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        HidClassFDO_DispatchRequest(DeviceObject, Irp)
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        PoStartNextPowerIrp(Irp);
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        STATUS_SUCCESS
    }
}

// ===========================================================================
// HidClass_PnP
// ===========================================================================
unsafe extern "system" fn HidClass_PnP(DeviceObject: PDEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    if (*CommonDeviceExtension).IsFDO != 0 {
        crate::fdo::HidClassFDO_PnP(DeviceObject, Irp)
    } else {
        crate::pdo::HidClassPDO_PnP(DeviceObject, Irp)
    }
}

// ===========================================================================
// HidClass_DispatchDefault
// ===========================================================================
unsafe extern "system" fn HidClass_DispatchDefault(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut _IRP,
) -> NTSTATUS {
    let CommonDeviceExtension =
        (*DeviceObject).DeviceExtension as PHIDCLASS_COMMON_DEVICE_EXTENSION;

    // FIXME: support PDO
    // ASSERT(IsFDO == TRUE);

    // skip current irp stack location
    IoSkipCurrentIrpStackLocation(Irp);

    // dispatch to lower device object
    IoCallDriver(
        (*CommonDeviceExtension).HidDeviceExtension.NextDeviceObject,
        Irp,
    )
}

// ===========================================================================
// HidClassDispatch
// ===========================================================================
unsafe extern "system" fn HidClassDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut _IRP,
) -> NTSTATUS {
    // get current stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // dispatch request based on major function
    match (*IoStack).MajorFunction {
        x if x == IRP_MJ_CREATE => HidClass_Create(DeviceObject, Irp),
        x if x == IRP_MJ_CLOSE => HidClass_Close(DeviceObject, Irp),
        x if x == IRP_MJ_READ => HidClass_Read(DeviceObject, Irp),
        x if x == IRP_MJ_WRITE => HidClass_Write(DeviceObject, Irp),
        x if x == IRP_MJ_DEVICE_CONTROL => HidClass_DeviceControl(DeviceObject, Irp),
        x if x == IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            HidClass_InternalDeviceControl(DeviceObject, Irp)
        }
        x if x == IRP_MJ_POWER => HidClass_Power(DeviceObject, Irp),
        x if x == IRP_MJ_PNP => HidClass_PnP(DeviceObject, Irp),
        _ => HidClass_DispatchDefault(DeviceObject, Irp),
    }
}

// ===========================================================================
// HidRegisterMinidriver — EXPORTED (hidusb links against this).
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn HidRegisterMinidriver(
    MinidriverRegistration: PHID_MINIDRIVER_REGISTRATION,
) -> NTSTATUS {
    let mut DriverExtension: PHIDCLASS_DRIVER_EXTENSION = null_mut();

    /* check if the version matches */
    if (*MinidriverRegistration).Revision > HID_REVISION {
        /* revision mismatch */
        return STATUS_REVISION_MISMATCH;
    }

    /* now allocate the driver object extension */
    let Status = IoAllocateDriverObjectExtension(
        (*MinidriverRegistration).DriverObject,
        core::ptr::addr_of_mut!(CLIENT_IDENTIFICATION_ADDRESS) as PVOID,
        size_of::<HIDCLASS_DRIVER_EXTENSION>() as ULONG,
        &mut DriverExtension as *mut _ as *mut DdkPVOID,
    );
    if !nt_success(Status) {
        /* failed to allocate driver extension */
        return Status;
    }

    /* zero driver extension */
    RtlZeroMemory(DriverExtension as PVOID, size_of::<HIDCLASS_DRIVER_EXTENSION>());

    /* init driver extension */
    (*DriverExtension).DriverObject = (*MinidriverRegistration).DriverObject;
    (*DriverExtension).DeviceExtensionSize = (*MinidriverRegistration).DeviceExtensionSize;
    (*DriverExtension).DevicesArePolled = (*MinidriverRegistration).DevicesArePolled;
    (*DriverExtension).AddDevice =
        (*(*(*MinidriverRegistration).DriverObject).DriverExtension).AddDevice;
    (*DriverExtension).DriverUnload = (*(*MinidriverRegistration).DriverObject).DriverUnload;

    /* copy driver dispatch routines */
    RtlCopyMemory(
        (*DriverExtension).MajorFunction.as_mut_ptr() as PVOID,
        (*(*MinidriverRegistration).DriverObject).MajorFunction.as_ptr() as PVOID,
        size_of::<PDRIVER_DISPATCH>() * (glue_irp_mj_maximum() + 1),
    );

    /* initialize lock */
    KeInitializeSpinLock(&mut (*DriverExtension).Lock as *mut _ as *mut _);

    /* now replace dispatch routines */
    // DriverExtension->DriverObject->DriverExtension->AddDevice = HidClassAddDevice;
    (*(*(*DriverExtension).DriverObject).DriverExtension).AddDevice =
        transmute::<AddDeviceFn, PDRIVER_ADD_DEVICE>(HidClassAddDevice);
    (*(*DriverExtension).DriverObject).DriverUnload =
        transmute::<UnloadFn, PDRIVER_UNLOAD>(HidClassDriverUnload);
    let mf = &mut (*(*DriverExtension).DriverObject).MajorFunction;
    mf[IRP_MJ_CREATE as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_CLOSE as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_READ as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_WRITE as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_DEVICE_CONTROL as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_POWER as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);
    mf[IRP_MJ_PNP as usize] = transmute::<DispatchFn, PDRIVER_DISPATCH>(HidClassDispatch);

    /* done */
    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// Helpers for IRP layout offset + swprintf wrapper + IRP_MJ_MAXIMUM_FUNCTION.
// ---------------------------------------------------------------------------
// Tail.Overlay.ListEntry is a LIST_ENTRY that, in the C union, overlaps
// Tail.Overlay.DeviceQueueEntry at offset 0 of the Overlay anon-union.
// bindgen exposes that anon-union as `Tail.Overlay.__bindgen_anon_1`, so the
// address of ListEntry == addr_of(Tail.Overlay.__bindgen_anon_1).
#[inline]
unsafe fn irp_listentry(Irp: PIRP) -> *mut ntkrnl_sys::ddk::LIST_ENTRY {
    core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1)
        as *mut ntkrnl_sys::ddk::LIST_ENTRY
}

#[inline]
fn irp_tail_overlay_listentry_offset() -> usize {
    // offset of Tail.Overlay.ListEntry (== Tail.Overlay.__bindgen_anon_1) in _IRP.
    core::mem::offset_of!(_IRP, Tail)
        + core::mem::offset_of!(ntkrnl_sys::ddk::_IRP__bindgen_ty_3, Overlay)
}

extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}

#[inline]
unsafe fn swprintf_wstr(buffer: *mut u16, format: *const u16, arg: ULONG) {
    swprintf(buffer, format, arg);
}

// IRP_MJ_MAXIMUM_FUNCTION + 1 = 28 (MajorFunction array length).
#[inline]
fn glue_irp_mj_maximum() -> usize {
    0x1B
}
