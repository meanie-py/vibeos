// entry.rs — DriverEntry, KbdHid_AddDevice, KbdHid_Unload,
//             KbdHid_Create, KbdHid_Close, KbdHid_Flush, KbdHid_CreateCompletion.
// Translated from drivers/hid/kbdhid/kbdhid.c

use core::mem::size_of;
use ntkrnl_sys::base::{NTSTATUS, PVOID, BOOLEAN, TRUE, FALSE};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP,
    PDRIVER_OBJECT, PUNICODE_STRING,
    PDRIVER_DISPATCH, PDRIVER_UNLOAD, PDRIVER_ADD_DEVICE,
    PIO_COMPLETION_ROUTINE,
    KEVENT,
    IoAllocateIrp, IoCreateDevice, IoAttachDeviceToDeviceStack, IoDeleteDevice,
    KeInitializeEvent, KeWaitForSingleObject,
    IoCancelIrp,
};
use ntkrnl_sys::ke::{KeClearEvent, KeSetEvent};
use ntkrnl_sys::constants::{
    STATUS_PENDING, KernelMode,
    STATUS_DEVICE_NOT_CONNECTED,
    IO_NO_INCREMENT, IRP_MJ_CREATE, IRP_MJ_CLOSE, IRP_MJ_FLUSH_BUFFERS,
};
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, IoCallDriver,
    FILE_DEVICE_KEYBOARD,
    DO_BUFFERED_IO, DO_POWER_PAGABLE, DO_DEVICE_INITIALIZING,
    IRP_MJ_DEVICE_CONTROL, IRP_MJ_INTERNAL_DEVICE_CONTROL,
    IRP_MJ_POWER, IRP_MJ_PNP, IRP_MJ_SYSTEM_CONTROL,
};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation,
    IoCopyCurrentIrpStackLocationToNext, IoSetCompletionRoutine,
    IoGetNextIrpStackLocation,
};
use ntkrnl_sys::ddk::{
    _KWAIT_REASON_Executive as Executive,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
};
use crate::types::{PKBDHID_DEVICE_EXTENSION, KBDHID_DEVICE_EXTENSION};
use crate::glue::{
    KBDHID_TAG,
    STATUS_MORE_PROCESSING_REQUIRED,
    KEYBOARD_TYPEMATIC_RATE_MINIMUM, KEYBOARD_TYPEMATIC_RATE_DEFAULT,
    KEYBOARD_TYPEMATIC_DELAY_MINIMUM, KEYBOARD_TYPEMATIC_DELAY_MAXIMUM,
    MICROSOFT_KBD_FUNC, KEYBOARD_TYPE_UNKNOWN, MICROSOFT_KBD_101_TYPE,
    PoSetPowerState, DevicePowerState, PowerDeviceD0,
    IOCTL_HID_FLUSH_QUEUE,
};
use crate::read::KbdHid_InitiateRead;
use crate::ioctl::{KbdHid_InternalDeviceControl, KbdHid_DeviceControl};
use crate::pnp::KbdHid_Pnp;
use crate::power::{KbdHid_Power, KbdHid_SystemControl};

// Dispatch function type alias — used for all MajorFunction slot transmutes.
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS;

/// KbdHid_CreateCompletion — completion routine for the CREATE pass-down.
/// Signals the synchronisation event so KbdHid_Create can unblock.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_CreateCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    KeSetEvent(Context as _, 0, FALSE);
    STATUS_MORE_PROCESSING_REQUIRED
}

/// KbdHid_Create — IRP_MJ_CREATE.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Create(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    /* get device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    /* get stack location */
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    /* copy stack location to next */
    IoCopyCurrentIrpStackLocationToNext(Irp);

    /* init event */
    let mut Event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event as *mut _ as _, NotificationEvent, FALSE);

    /* prepare irp */
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS, PIO_COMPLETION_ROUTINE>(KbdHid_CreateCompletion),
        &mut Event as *mut _ as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    /* call lower driver */
    let mut Status = IoCallDriver((*DeviceExtension).NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        /* request pending */
        KeWaitForSingleObject(
            &mut Event as *mut _ as _,
            Executive,
            KernelMode as _,
            FALSE,
            core::ptr::null_mut(),
        );
    }

    /* check for success */
    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        /* failed */
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    /* is the driver already in use */
    if (*DeviceExtension).FileObject.is_null() {
        /* did the caller specify correct attributes */
        ntkrnl_sys::ASSERT!(!(*IoStack).Parameters.Create.SecurityContext.is_null());
        if (*(*IoStack).Parameters.Create.SecurityContext).DesiredAccess != 0 {
            /* store file object */
            (*DeviceExtension).FileObject = (*IoStack).FileObject;

            /* reset event */
            KeClearEvent(&mut (*DeviceExtension).ReadCompletionEvent as *mut _ as _);

            /* initiating read */
            Status = KbdHid_InitiateRead(DeviceExtension);
            if Status == STATUS_PENDING {
                /* report irp is pending */
                Status = STATUS_SUCCESS;
            }
        }
    }

    /* complete request */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

/// KbdHid_Close — IRP_MJ_CLOSE.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Close(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    /* get device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    if (*DeviceExtension).ReadReportActive != 0 {
        /* request stopping of the report cycle */
        (*DeviceExtension).StopReadReport = TRUE as u8;

        /* wait until the reports have been read */
        KeWaitForSingleObject(
            &mut (*DeviceExtension).ReadCompletionEvent as *mut _ as _,
            Executive,
            KernelMode as _,
            FALSE,
            core::ptr::null_mut(),
        );

        /* cancel irp */
        IoCancelIrp((*DeviceExtension).Irp);
    }

    /* remove file object */
    (*DeviceExtension).FileObject = core::ptr::null_mut();

    /* skip location */
    IoSkipCurrentIrpStackLocation(Irp);

    /* pass irp to down the stack */
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}

/// KbdHid_Flush — IRP_MJ_FLUSH_BUFFERS: morph to IOCTL_HID_FLUSH_QUEUE and forward.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Flush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    /* skip current stack location */
    IoSkipCurrentIrpStackLocation(Irp);

    /* get next stack location */
    let IoStack = IoGetNextIrpStackLocation(Irp);

    /* change request to hid flush queue request */
    (*IoStack).MajorFunction = IRP_MJ_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_HID_FLUSH_QUEUE;

    /* call device */
    IoCallDriver((*DeviceExtension).NextDeviceObject, Irp)
}

/// KbdHid_Unload — UNIMPLEMENTED in C.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_Unload(_DriverObject: PDRIVER_OBJECT) {
    // UNIMPLEMENTED
}

/// KbdHid_AddDevice — creates the FDO and attaches it to the device stack.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    /* create device object */
    let mut Status = IoCreateDevice(
        DriverObject,
        size_of::<KBDHID_DEVICE_EXTENSION>() as u32,
        core::ptr::null_mut(), // no name
        FILE_DEVICE_KEYBOARD,
        0,
        FALSE,
        &mut DeviceObject,
    );
    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        /* failed to create device object */
        return Status;
    }

    /* now attach it */
    let NextDeviceObject = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
    if NextDeviceObject.is_null() {
        /* failed to attach */
        IoDeleteDevice(DeviceObject);
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    /* get device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PKBDHID_DEVICE_EXTENSION;

    /* zero extension */
    RtlZeroMemory(DeviceExtension as PVOID, size_of::<KBDHID_DEVICE_EXTENSION>());

    /* init device extension */
    (*DeviceExtension).NextDeviceObject = NextDeviceObject;
    KeInitializeEvent(
        &mut (*DeviceExtension).ReadCompletionEvent as *mut _ as _,
        NotificationEvent,
        FALSE,
    );

    /* init keyboard attributes */
    (*DeviceExtension).Attributes.KeyboardIdentifier.Type    = KEYBOARD_TYPE_UNKNOWN;
    (*DeviceExtension).Attributes.KeyboardIdentifier.Subtype = MICROSOFT_KBD_101_TYPE;
    (*DeviceExtension).Attributes.NumberOfFunctionKeys       = MICROSOFT_KBD_FUNC;
    (*DeviceExtension).Attributes.NumberOfIndicators         = 3; // caps, num lock, scroll lock
    (*DeviceExtension).Attributes.NumberOfKeysTotal          = 101;
    (*DeviceExtension).Attributes.InputDataQueueLength       = 1;
    (*DeviceExtension).Attributes.KeyRepeatMinimum.Rate      = KEYBOARD_TYPEMATIC_RATE_MINIMUM;
    (*DeviceExtension).Attributes.KeyRepeatMinimum.Delay     = KEYBOARD_TYPEMATIC_DELAY_MINIMUM;
    (*DeviceExtension).Attributes.KeyRepeatMaximum.Rate      = KEYBOARD_TYPEMATIC_RATE_DEFAULT;
    (*DeviceExtension).Attributes.KeyRepeatMaximum.Delay     = KEYBOARD_TYPEMATIC_DELAY_MAXIMUM;

    /* allocate irp */
    (*DeviceExtension).Irp = IoAllocateIrp((*NextDeviceObject).StackSize, FALSE);

    /* FIXME handle allocation error */
    ntkrnl_sys::ASSERT!(!(*DeviceExtension).Irp.is_null());

    /* set power state to D0 */
    // C: State.DeviceState = PowerDeviceD0; PoSetPowerState(DO, DevicePowerState, State);
    // POWER_STATE is a pointer-sized union; DeviceState (first member) = PowerDeviceD0 = 1.
    PoSetPowerState(DeviceObject, DevicePowerState, PowerDeviceD0);

    /* init device object */
    (*DeviceObject).Flags |= DO_BUFFERED_IO | DO_POWER_PAGABLE;
    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    /* completed successfully */
    STATUS_SUCCESS
}

/// DriverEntry — wire up the driver's dispatch table and AddDevice.
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegPath: PUNICODE_STRING,
) -> NTSTATUS {
    ntkrnl_sys::driver_ddk::DbgPrint(b"[KBDHID] DriverEntry\n\0".as_ptr() as *const i8);
    /* initialize driver object */
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_Create);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_Close);
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_Flush);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_DeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_InternalDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_Power);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_Pnp);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(KbdHid_SystemControl);
    (*DriverObject).DriverUnload = core::mem::transmute::<
        unsafe extern "system" fn(PDRIVER_OBJECT),
        PDRIVER_UNLOAD,
    >(KbdHid_Unload);
    (*(*DriverObject).DriverExtension).AddDevice = core::mem::transmute::<
        unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS,
        PDRIVER_ADD_DEVICE,
    >(KbdHid_AddDevice);

    /* done */
    STATUS_SUCCESS
}
