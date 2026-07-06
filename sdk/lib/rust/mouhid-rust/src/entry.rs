// entry.rs — DriverEntry, MouHid_AddDevice, MouHid_Unload, MouHid_Create, MouHid_Close
// Translated from drivers/hid/mouhid/mouhid.c

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext, IoSkipCurrentIrpStackLocation, IoSetCompletionRoutine};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PDRIVER_OBJECT, PUNICODE_STRING, PIRP, PRKEVENT,
    IoCreateDevice,
    IoAttachDeviceToDeviceStack,
    IoDeleteDevice,
    IoAllocateIrp,
    IoCancelIrp,
    KeInitializeEvent,
    KeClearEvent,
    KeWaitForSingleObject,
    PDRIVER_DISPATCH,
    PDRIVER_UNLOAD,
    PDRIVER_ADD_DEVICE,
    PIO_COMPLETION_ROUTINE,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _KWAIT_REASON_Executive as Executive,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
    _POWER_STATE,
};

macro_rules! ASSERT {
    ($e:expr) => { debug_assert!($e) };
}

use crate::types::PMOUHID_DEVICE_EXTENSION;
use crate::glue::*;
use crate::read::{MouHid_CreateCompletion, MouHid_InitiateRead};
use crate::ioctl::{MouHid_InternalDeviceControl, MouHid_DeviceControl};
use crate::pnp::{MouHid_Flush, MouHid_Pnp, MouHid_FreeResources};
use crate::power::{MouHid_Power, MouHid_SystemControl};

// PoSetPowerState — exported by ntoskrnl, absent from ntkrnl_sys::ddk bindgen
// (lives in wdm.h, not ntoskrnl.h). Declared locally.
extern "system" {
    fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: u32,              // POWER_STATE_TYPE (enum)
        State: _POWER_STATE,
    ) -> _POWER_STATE;
}

// Dispatch function signature type (for transmute into bindgen PDRIVER_DISPATCH slot)
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS;

// ---------------------------------------------------------------------------
// MouHid_Create — IRP_MJ_CREATE
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Create(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Event: ntkrnl_sys::ddk::KEVENT = core::mem::zeroed();
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;

    // DPRINT "MOUHID: IRP_MJ_CREATE\n"

    // get stack location
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    // copy stack location to next
    IoCopyCurrentIrpStackLocationToNext(Irp);

    // init event
    KeInitializeEvent(&mut Event, NotificationEvent, FALSE);

    // prepare irp
    type CompletionFn = unsafe extern "system" fn(
        PDEVICE_OBJECT,
        *mut ntkrnl_sys::ddk::_IRP,
        PVOID,
    ) -> NTSTATUS;
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<CompletionFn, PIO_COMPLETION_ROUTINE>(
            MouHid_CreateCompletion,
        ),
        &mut Event as *mut _ as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    let ext = &mut *DeviceExtension;

    // call lower driver
    Status = IoCallDriver(ext.NextDeviceObject, Irp);
    if Status == STATUS_PENDING {
        // request pending
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            Executive,
            0i8, // KernelMode
            FALSE,
            core::ptr::null_mut(),
        );
    }

    // check for success
    if !NT_SUCCESS(Status) {
        // failed
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // is the driver already in use
    if ext.FileObject.is_null() {
        // did the caller specify correct attributes
        ASSERT!(!(*IoStack).Parameters.Create.SecurityContext.is_null());
        if (*(*IoStack).Parameters.Create.SecurityContext).DesiredAccess != 0 {
            // store file object
            ext.FileObject = (*IoStack).FileObject;

            // reset event
            KeClearEvent(
                &mut ext.ReadCompletionEvent as *mut _ as PRKEVENT,
            );

            // initiating read
            Status = MouHid_InitiateRead(DeviceExtension);
            // DPRINT "[MOUHID] MouHid_InitiateRead: status %x\n"
            if Status == STATUS_PENDING {
                // report irp is pending
                Status = STATUS_SUCCESS;
            }
        }
    }

    // complete request
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}

// ---------------------------------------------------------------------------
// MouHid_Close — IRP_MJ_CLOSE
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Close(
    DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as _;
    let ext = &mut *DeviceExtension;

    // DPRINT "[MOUHID] IRP_MJ_CLOSE ReadReportActive %x\n"

    if ext.ReadReportActive != 0 {
        // request stopping of the report cycle
        ext.StopReadReport = TRUE;

        // wait until the reports have been read
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(ext.ReadCompletionEvent) as PVOID,
            Executive,
            0i8, // KernelMode
            FALSE,
            core::ptr::null_mut(),
        );

        // cancel irp
        IoCancelIrp(ext.Irp);
    }

    // DPRINT "[MOUHID] IRP_MJ_CLOSE ReadReportActive %x\n"

    // remove file object
    ext.FileObject = core::ptr::null_mut();

    // skip location
    IoSkipCurrentIrpStackLocation(Irp);

    // pass irp to down the stack
    IoCallDriver(ext.NextDeviceObject, Irp)
}

// ---------------------------------------------------------------------------
// MouHid_AddDevice — called by PnP manager when a new mouse HID PDO appears
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let NextDeviceObject: PDEVICE_OBJECT;
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION;
    let mut State: _POWER_STATE = core::mem::zeroed();

    // create device object
    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<crate::types::MOUHID_DEVICE_EXTENSION>() as ULONG,
        core::ptr::null_mut(),
        FILE_DEVICE_MOUSE,
        0,
        FALSE,
        &mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        // failed to create device object
        return Status;
    }

    // now attach it
    NextDeviceObject = IoAttachDeviceToDeviceStack(DeviceObject, PhysicalDeviceObject);
    if NextDeviceObject.is_null() {
        // failed to attach
        IoDeleteDevice(DeviceObject);
        return STATUS_DEVICE_NOT_CONNECTED;
    }

    // get device extension
    DeviceExtension = (*DeviceObject).DeviceExtension as _;
    let ext = &mut *DeviceExtension;

    // zero extension
    RtlZeroMemory(
        DeviceExtension as PVOID,
        core::mem::size_of::<crate::types::MOUHID_DEVICE_EXTENSION>(),
    );

    // init device extension
    ext.MouseIdentifier = MOUSE_HID_HARDWARE;
    ext.WheelUsagePage = 0;
    ext.NextDeviceObject = NextDeviceObject;
    KeInitializeEvent(
        &mut ext.ReadCompletionEvent as *mut _ as PRKEVENT,
        NotificationEvent,
        FALSE,
    );
    ext.Irp = IoAllocateIrp((*NextDeviceObject).StackSize, FALSE);

    // FIXME handle allocation error
    ASSERT!(!ext.Irp.is_null());

    // FIXME query parameter 'FlipFlopWheel', 'WheelScalingFactor'

    // set power state to D0
    State.DeviceState = PowerDeviceD0;
    PoSetPowerState(DeviceObject, DevicePowerState, State);

    // init device object
    (*DeviceObject).Flags |= DO_BUFFERED_IO | DO_POWER_PAGABLE;
    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    // completed successfully
    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// MouHid_Unload
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_Unload(
    _DriverObject: PDRIVER_OBJECT,
) {
    // UNIMPLEMENTED
}

// ---------------------------------------------------------------------------
// DriverEntry
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegPath: PUNICODE_STRING,
) -> NTSTATUS {
    ntkrnl_sys::driver_ddk::DbgPrint(b"[MOUHID] DriverEntry\n\0".as_ptr() as *const i8);
    // FIXME check for parameters 'UseOnlyMice', 'TreatAbsoluteAsRelative',
    //       'TreatAbsolutePointerAsAbsolute'

    // initialize driver object
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_Create);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_Close);
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_Flush);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_DeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_InternalDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_Power);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_Pnp);
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(MouHid_SystemControl);
    (*DriverObject).DriverUnload =
        core::mem::transmute::<
            unsafe extern "system" fn(PDRIVER_OBJECT),
            PDRIVER_UNLOAD,
        >(MouHid_Unload);
    (*(*DriverObject).DriverExtension).AddDevice =
        core::mem::transmute::<
            unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS,
            PDRIVER_ADD_DEVICE,
        >(MouHid_AddDevice);

    // done
    STATUS_SUCCESS
}
