use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExDeleteNPagedLookasideList, ExInitializeNPagedLookasideList, IRP, IoFreeIrp, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoSetCompletionRoutine, IoSetHardErrorOrVerifyDevice, IoStartNextPacket, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FUNCTIONAL_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, SCSI_REQUEST_BLOCK};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, IO_STACK_LOCATION};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_RW};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ServiceTransferRequest};

extern "C" {
    fn TracePrint(level: u32, flag: u32, format: *const i8, ...);
    fn SRB_STATUS(status: u8) -> u8;
    
    fn InterpretSenseInfoWithoutHistory( fdo: *mut DEVICE_OBJECT, irp: *mut IRP, srb: *mut SCSI_REQUEST_BLOCK, major_function: u8, control_code_or_zero: u32, retry_count: u32, status: *mut i32, retry_interval: *mut i64, ) -> u8;
    fn PORT_ALLOCATED_SENSE( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, ) -> u8;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, );
    fn RetryRequest( fdo: *mut DEVICE_OBJECT, irp: *mut IRP, srb: *mut SCSI_REQUEST_BLOCK, associated: u8, time_delta_100ns: i64, );
    fn ClassFreeOrReuseSrb( fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION, srb: *mut SCSI_REQUEST_BLOCK, );
    
    fn IoIsErrorUserInduced(status: i32) -> u8;
    fn ClassAcquireRemoveLock(fdo: *mut DEVICE_OBJECT, irp: *mut IRP) -> u32;
    
    
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _CSCAN_LIST_ENTRY {
    pub Entry: LIST_ENTRY,
    pub BlockNumber: ULONGLONG,
}
pub type CSCAN_LIST_ENTRY = _CSCAN_LIST_ENTRY;
pub type PCSCAN_LIST_ENTRY = *mut _CSCAN_LIST_ENTRY;

// External declarations for unknown functions

// Function constants
const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;
const MAXIMUM_RETRIES: u32 = 0x04;
const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02; // approximate, actual may differ
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016_u32 as i32;
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185_u32 as i32;
const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016_u32 as i32;
const SRB_STATUS_SUCCESS: u8 = 0x01;

#[no_mangle]
pub unsafe extern "system" fn ClassSplitRequest(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP,
    MaximumBytes: ULONG,
) {
    let fdoExt = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExt).PrivateFdoData;
    if MaximumBytes > (*fdoData).HwMaxXferLen {
        TracePrint(
            (2i32) as u32,
            TRACE_FLAG_RW,
            b"ClassSplitRequest - driver requesting split to size that hardware is unable to handle!\n\0"
                .as_ptr() as PCHAR,
        );
    }
    if MaximumBytes < (*fdoData).HwMaxXferLen {
        TracePrint(
            (3i32) as u32,
            TRACE_FLAG_RW,
            b"ClassSplitRequest - driver requesting smaller HwMaxXferLen than required\0"
                .as_ptr() as PCHAR,
        );
        (*fdoData).HwMaxXferLen = if MaximumBytes > (PAGE_SIZE) as u32 {
            MaximumBytes
        } else {
            (PAGE_SIZE) as u32
        };
    }
    ServiceTransferRequest(Fdo, Irp, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ClassIoCompleteAssociated(
    Fdo: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let fdo_extension: *mut FUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let mut irp_stack: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let srb: *mut SCSI_REQUEST_BLOCK = Context as *mut SCSI_REQUEST_BLOCK;
    let original_irp: *mut IRP = (*Irp).AssociatedIrp.MasterIrp;
    let mut irp_count: i32;
    let mut status: i32 = 0;
    let mut retry: u8 = 0;

    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassIoCompleteAssociated is OBSOLETE !\0".as_ptr() as *const i8);

    if SRB_STATUS((*srb).SrbStatus) != SRB_STATUS_SUCCESS {
        let mut retry_interval: i64 = 0;
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassIoCompleteAssociated: IRP %p, SRB %p\0".as_ptr() as *const i8, Irp as usize, srb as usize);

        if ((*srb).SrbStatus & SRB_STATUS_QUEUE_FROZEN) != 0 {
            ClassReleaseQueue(Fdo);
        }

        let retry_count = {
            let arg4_val = (*irp_stack).Parameters.Others.Argument4 as usize as u32;
            MAXIMUM_RETRIES - arg4_val
        };

        retry = InterpretSenseInfoWithoutHistory(
            Fdo,
            Irp,
            srb,
            (*irp_stack).MajorFunction,
            if (*irp_stack).MajorFunction == IRP_MJ_DEVICE_CONTROL {
                (*irp_stack).Parameters.DeviceIoControl.IoControlCode
            } else {
                0
            },
            retry_count,
            &mut status as *mut i32,
            &mut retry_interval as *mut i64,
        ) as u8;

        if ((*irp_stack).Flags & SL_OVERRIDE_VERIFY_VOLUME) != 0 &&
            status == STATUS_VERIFY_REQUIRED {
            status = STATUS_IO_DEVICE_ERROR;
            retry = 1u8;
        }

        let arg4_as_u32 = core::ptr::addr_of_mut!((*irp_stack).Parameters.Others.Argument4) as *mut _ as *mut u32;
        if retry != 0 && *arg4_as_u32 != 0 {
            *arg4_as_u32 -= 1;

            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"Retry request %p\n\0".as_ptr() as *const i8, Irp as usize);

            if PORT_ALLOCATED_SENSE(fdo_extension, srb) != 0 {
                FREE_PORT_ALLOCATED_SENSE_BUFFER(fdo_extension, srb);
            }

            RetryRequest(Fdo, Irp, srb, 1u8, retry_interval);

            return STATUS_MORE_PROCESSING_REQUIRED;
        }
    } else {
        status = STATUS_SUCCESS;
    }

    if PORT_ALLOCATED_SENSE(fdo_extension, srb) != 0 {
        FREE_PORT_ALLOCATED_SENSE_BUFFER(fdo_extension, srb);
    }

    ClassFreeOrReuseSrb(fdo_extension, srb);

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassIoCompleteAssociated: Partial xfer IRP %p\n\0".as_ptr() as *const i8, Irp as usize);

    irp_stack = IoGetNextIrpStackLocation(original_irp);

    if !NT_SUCCESS(status) {
        (*original_irp).IoStatus.__bindgen_anon_1.Status = status;
        (*original_irp).IoStatus.Information = 0;

        if IoIsErrorUserInduced(status) != 0 &&
            (*original_irp).Tail.Overlay.Thread != core::ptr::null_mut() {
            IoSetHardErrorOrVerifyDevice(original_irp, Fdo);
        }
    }

    irp_count = InterlockedDecrement(
        core::ptr::addr_of!((*irp_stack).Parameters.Others.Argument1) as *const _ as *mut i32
    );

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassIoCompleteAssociated: Partial IRPs left %d\n\0".as_ptr() as *const i8, irp_count as i32);

    NT_ASSERT!(irp_count >= 0);

    if irp_count == 0 {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassIoCompleteAssociated: All partial IRPs complete %p\n\0".as_ptr() as *const i8, original_irp as usize);

        let driver_ext = (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension;
        if (*driver_ext).InitData.ClassStartIo.is_some() {
            let mut unique_address: u8 = 0;
            ClassAcquireRemoveLock(Fdo, &mut unique_address as *mut u8 as *mut IRP);
            ClassReleaseRemoveLock(Fdo, original_irp);
            ClassCompleteRequest(Fdo, original_irp, ((IO_DISK_INCREMENT) as i32) as _);

            let mut old_irql: u8 = 0;
            KeRaiseIrql(DISPATCH_LEVEL, &mut old_irql);
            IoStartNextPacket(Fdo, 1u8);
            KeLowerIrql(old_irql);

            ClassReleaseRemoveLock(Fdo, &mut unique_address as *mut u8 as *mut IRP);
        } else {
            ClassReleaseRemoveLock(Fdo, original_irp);
            ClassCompleteRequest(Fdo, original_irp, ((IO_DISK_INCREMENT) as i32) as _);
        }
    }

    IoFreeIrp(Irp);
    return STATUS_MORE_PROCESSING_REQUIRED;
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn InterlockedDecrement(addend: *mut i32) -> i32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn InterlockedDecrement(addend: *mut i32) -> i32;
}
extern "system" {
    fn ClassReleaseQueue(fdo: *mut DEVICE_OBJECT);
    fn ClassReleaseRemoveLock(fdo: *mut DEVICE_OBJECT, irp: *mut IRP);
    fn ClassCompleteRequest(fdo: *mut DEVICE_OBJECT, irp: *mut IRP, priority_boost: u8);
}
