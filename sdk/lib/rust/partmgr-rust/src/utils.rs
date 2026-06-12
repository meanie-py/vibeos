// ---- crate-internal glue (private structs, GUID externs, no-op DDK shims) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::guids::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIRP, PKEVENT, UINT32};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{KEVENT, PCSTR, PRKEVENT, RtlAssert};
#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // this part of a structure is identical in both FDO and PDO
    let lower_device: PDEVICE_OBJECT = (*(
        (*DeviceObject).DeviceExtension as PFDO_EXTENSION
    )).LowerDevice;

    if lower_device.is_null() {
        RtlAssert(
            b"LowerDevice\0".as_ptr() as PVOID,
            b"/home/curi/Projects/os-playground/vibeos-wt-win32k/drivers/storage/partmgr/utils.c\0"
                .as_ptr() as PVOID,
            12 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver(lower_device, Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IssueSyncIoControlRequest(
    IoControlCode: UINT32,
    DeviceObject: PDEVICE_OBJECT,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    InternalDeviceIoControl: BOOLEAN,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-win32k/drivers/storage/partmgr/utils.c\0"
                    .as_ptr() as *const i8 as PVOID,
                32 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    let Event = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<KEVENT>(),
        TAG_PARTMGR,
    ) as PKEVENT;
    if Event.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    KeInitializeEvent(Event as PRKEVENT, _EVENT_TYPE_NotificationEvent, 0u8);

    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        DeviceObject,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        InternalDeviceIoControl,
        Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        ExFreePoolWithTag(Event as PVOID, TAG_PARTMGR);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let Status = IoCallDriver(DeviceObject, Irp);
    let final_status = if Status == STATUS_PENDING {
        (KeWaitForSingleObject)(
            Event as PVOID,
            _KWAIT_REASON_Executive,
            0i8, // KernelMode (KPROCESSOR_MODE)
            0u8,
            core::ptr::null_mut(), // Timeout: PLARGE_INTEGER (infer *mut _LARGE_INTEGER)
        );
        IoStatusBlock.__bindgen_anon_1.Status
    } else {
        Status
    };

    ExFreePoolWithTag(Event as PVOID, TAG_PARTMGR);
    final_status
}
