// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
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
use ntkrnl_sys::ddk::{DISK_GEOMETRY, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, KEVENT, KeInitializeEvent, KeWaitForSingleObject, PARTITION_INFORMATION, PDEVICE_OBJECT, PIRP, PLARGE_INTEGER, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert, _DEVICE_OBJECT, _DISK_GEOMETRY, _IRP, _KEVENT, _LARGE_INTEGER};

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

#[no_mangle]
pub unsafe extern "system" fn FsRecGetDeviceSectors(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: ULONG,
    SectorCount: PLARGE_INTEGER,
) -> BOOLEAN {
    let mut PartitionInfo: PARTITION_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut Remainder: ULONG = 0;

    PAGED_CODE!();

    /* Only needed for disks */
    if (*DeviceObject).DeviceType != FILE_DEVICE_DISK as u32 {
        return FALSE as BOOLEAN;
    }

    /* Build the information IRP */
    KeInitializeEvent(&mut Event, _EVENT_TYPE_SynchronizationEvent, FALSE as BOOLEAN);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_GET_PARTITION_INFO,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        &mut PartitionInfo as *mut _ as PVOID,
        core::mem::size_of::<PARTITION_INFORMATION>() as ULONG,
        FALSE as BOOLEAN,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return FALSE as BOOLEAN;
    }

    /* Override verification */
    (*IoGetNextIrpStackLocation(Irp)).Flags |= (SL_OVERRIDE_VERIFY_VOLUME) as u8;

    /* Do the request */
    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        /* Wait for completion */
        (KeWaitForSingleObject)(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    /* Fail if we couldn't get the data */
    if !NT_SUCCESS(Status) {
        return FALSE as BOOLEAN;
    }

    /* Otherwise, return the number of sectors */
    *SectorCount = RtlExtendedLargeIntegerDivide(
        PartitionInfo.PartitionLength,
        SectorSize,
        &mut Remainder as *mut ULONG,
    );
    TRUE as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn FsRecGetDeviceSectorSize(
    DeviceObject: *mut _DEVICE_OBJECT,
    SectorSize: *mut u32,
) -> u8 {
    let mut DiskGeometry: _DISK_GEOMETRY = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: _KEVENT = core::mem::zeroed();
    let Irp: *mut _IRP;
    let mut Status: NTSTATUS;
    let ControlCode: u32;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            (KeGetCurrentIrql()) as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/filesystems/fs_rec/blockdev.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            89,
            core::ptr::null_mut::<i8>(),
        );
    }

    // Device type check
    match (*DeviceObject).DeviceType {
        2 => ControlCode = IOCTL_CDROM_GET_DRIVE_GEOMETRY,
        7 => ControlCode = IOCTL_DISK_GET_DRIVE_GEOMETRY,
        _ => return FALSE as u8,
    }

    // Build IRP
    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_SynchronizationEvent,
        FALSE as u8,
    );
    Irp = IoBuildDeviceIoControlRequest(
        ControlCode,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(DiskGeometry) as *mut core::ffi::c_void,
        core::mem::size_of::<_DISK_GEOMETRY>() as u32,
        FALSE as u8,
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp.is_null() {
        return FALSE as u8;
    }

    // Override verification
    let next_stack = IoGetNextIrpStackLocation(Irp);
    (*next_stack).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;

    // Call driver
    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0i32 as u8) as i8,
            FALSE as u8,
            core::ptr::null_mut::<_LARGE_INTEGER>(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    // Check success and sector size
    if !(Status >= 0) {
        return FALSE as u8;
    }
    if DiskGeometry.BytesPerSector == 0 {
        return FALSE as u8;
    }

    *SectorSize = DiskGeometry.BytesPerSector;
    TRUE as u8
}
#[no_mangle]
pub unsafe extern "system" fn FsRecReadBlock(
    DeviceObject: PDEVICE_OBJECT,
    Offset: PLARGE_INTEGER,
    Length: ULONG,
    SectorSize: ULONG,
    Buffer: *mut PVOID,
    DeviceError: PBOOLEAN,
) -> BOOLEAN {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: _KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/filesystems/fs_rec/blockdev.c\0"
                    .as_ptr() as PVOID,
                163,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    if !DeviceError.is_null() {
        *DeviceError = 0u8;
    }
    let mut alloc_length = Length;
    if alloc_length < SectorSize {
        alloc_length = SectorSize;
    } else {
        alloc_length = crate::inlines::ROUND_UP(alloc_length, SectorSize);
    }
    if (*Buffer).is_null() {
        *Buffer = ExAllocatePoolWithTag(
            NonPagedPool,
            ROUND_TO_PAGES(alloc_length as SIZE_T),
            FSREC_TAG,
        ) as PVOID;
        if (*Buffer).is_null() {
            return 0u8;
        }
    }
    KeInitializeEvent(&mut Event, _EVENT_TYPE_SynchronizationEvent, 0u8);
    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as ULONG,
        DeviceObject,
        *Buffer,
        alloc_length,
        Offset,
        &mut Event,
        &mut IoStatusBlock,
    );
    if Irp.is_null() {
        return 0u8;
    }
    let flags = &mut (*IoGetNextIrpStackLocation(Irp)).Flags;
    *flags = (*flags as i32 | (SL_OVERRIDE_VERIFY_VOLUME) as i32) as u8;
    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut _ as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as i8,
            0u8,
            core::ptr::null_mut::<_LARGE_INTEGER>(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }
    if !NT_SUCCESS(Status) {
        if !DeviceError.is_null() {
            *DeviceError = 1u8;
        }
        return 0u8;
    }
    return 1u8;
}
