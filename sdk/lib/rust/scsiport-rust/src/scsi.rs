#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, IRP, IoBuildAsynchronousFsdRequest, IoFlushAdapterBuffers, IoFreeAdapterChannel, IoFreeIrp, IoFreeMapRegisters, IoFreeMdl, IoStartNextPacket, IoStartPacket, KeCancelTimer, KeInsertByKeyDeviceQueue, KeRemoveByKeyDeviceQueue, KeRemoveDeviceQueue, KeSetTimer, KeSynchronizeExecution, LARGE_INTEGER, MmUnlockPages, PCCHAR, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKDEVICE_QUEUE_ENTRY, PKIRQL, CCHAR, _LIST_ENTRY, IO_ALLOCATION_ACTION, IO_STACK_LOCATION, PDMA_ADAPTER, _IO_STACK_LOCATION, _IRP, _KDEVICE_QUEUE_ENTRY};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{LUNEX_REQUEST_PENDING, PCDB, PSCSI_PORT_DEVICE_EXTENSION, PSCSI_PORT_INTERRUPT_DATA, PSCSI_PORT_LUN_EXTENSION, PSCSI_REQUEST_BLOCK, PSCSI_REQUEST_BLOCK_INFO, SCSI_PORT_DEVICE_BUSY, SCSI_PORT_DISABLE_INT_REQUESET, SCSI_PORT_DISCONNECT_ALLOWED, SCSI_PORT_ENABLE_INT_REQUEST, SCSI_PORT_FLUSH_ADAPTERS, SCSI_PORT_INTERRUPT_DATA, SCSI_PORT_LU_ACTIVE, SCSI_PORT_MAP_TRANSFER, SCSI_PORT_NEXT_REQUEST_READY, SCSI_PORT_REQUEST_PENDING, SCSI_PORT_RESET_REPORTED, SCSI_PORT_SAVE_INTERRUPT, SCSI_PORT_TIMER_NEEDED, SCSI_REQUEST_BLOCK, SENSE_DATA, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_STATUS_REQUEST_FLUSHED, _SCSI_PORT_COMMON_EXTENSION, _SCSI_PORT_DEVICE_EXTENSION, _SCSI_PORT_LUN_EXTENSION, _SCSI_REQUEST_BLOCK, _SCSI_REQUEST_BLOCK_INFO, LUNEX_BUSY, LUNEX_FULL_QUEUE, SCSISTAT_BUSY, SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_LUN_EXTENSION, SCSI_REQUEST_BLOCK_INFO, SRB_FLAGS_SGLIST_FROM_POOL, ScsiPortAllocateAdapterChannel, SpiAdapterControl};
use crate::glue::{SRB_STATUS_DATA_OVERRUN, SRB_STATUS_SELECTION_TIMEOUT, SRB_FUNCTION_RELEASE_DEVICE, SRB_FUNCTION_RELEASE_QUEUE, SRB_FUNCTION_FLUSH_QUEUE};
// resolve_imports.py: explicit kernel/private-type imports

extern "C" {
    fn SpiGetSrbData(DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION, LunExtension: PSCSI_PORT_LUN_EXTENSION, QueueTag: u8,) -> PSCSI_REQUEST_BLOCK_INFO;
}
extern "C" {
    fn ScsiPortNotification(NotificationType: u32, HwDeviceExtension: *mut core::ffi::c_void, ...);
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ---- file-scope preamble ----
// All #define consts the LLM redefined locally (3 conflicting copies of several, many
// with fabricated values) were deleted — single source of truth is glue.rs (header-
// verified) + the bindgen types. RtlAssert resolves to the ddk_shims no-op (NDEBUG).
pub const NDEBUG: u32 = 1;

// Int32x32To64 — winnt.h 64-bit signed multiply intrinsic (not a real export).
#[inline(always)]
unsafe fn Int32x32To64(a: i32, b: i32) -> i64 { (a as i64) * (b as i64) }
// SpiGetNextRequestFromLun / SpiSaveInterruptData / SpiProcessCompletedRequest /
// ScsiPortStartIo are defined at the bottom of this file (bodies recovered from the
// translation .responses after the assembler dropped them to forward-decls).

// SpiStatusSrbToNt — also assembler-dropped; recovered from .responses/01_*.rs. Const
// preamble omitted (glue.rs is the verified source).
unsafe fn SpiStatusSrbToNt(SrbStatus: u8) -> NTSTATUS {
    let masked_status = SrbStatus & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN);
    match masked_status {
        SRB_STATUS_TIMEOUT | SRB_STATUS_COMMAND_TIMEOUT => return STATUS_IO_TIMEOUT,
        SRB_STATUS_BAD_SRB_BLOCK_LENGTH | SRB_STATUS_BAD_FUNCTION => {
            return STATUS_INVALID_DEVICE_REQUEST;
        }
        SRB_STATUS_NO_DEVICE
        | SRB_STATUS_INVALID_LUN
        | SRB_STATUS_INVALID_TARGET_ID
        | SRB_STATUS_NO_HBA => return STATUS_DEVICE_DOES_NOT_EXIST,
        SRB_STATUS_DATA_OVERRUN => return STATUS_BUFFER_OVERFLOW,
        SRB_STATUS_SELECTION_TIMEOUT => return STATUS_DEVICE_NOT_CONNECTED,
        _ => return STATUS_IO_DEVICE_ERROR,
    };
}

unsafe fn SpiHandleAttachRelease(
    mut LunExtension: PSCSI_PORT_LUN_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION =
        (*(*LunExtension).Common.LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Irql: KIRQL;
    let mut IrpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Srb: PSCSI_REQUEST_BLOCK = (*IrpStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    Irql = KeAcquireSpinLockRaiseToDpc((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());

    if (*Srb).Function as i32 == (SRB_FUNCTION_RELEASE_DEVICE) as i32 {
        (*LunExtension).DeviceClaimed = FALSE;
        KeReleaseSpinLock((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast(), Irql);
        (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
        return STATUS_SUCCESS;
    }

    if (*LunExtension).DeviceClaimed != 0 {
        KeReleaseSpinLock((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast(), Irql);
        (*Srb).SrbStatus = SRB_STATUS_BUSY;
        return STATUS_DEVICE_BUSY;
    }

    DeviceObject = (*LunExtension).Common.DeviceObject;

    if (*Srb).Function as i32 == (SRB_FUNCTION_CLAIM_DEVICE) as i32 {
        (*LunExtension).DeviceClaimed = TRUE;
    }

    if (*Srb).Function as i32 == (SRB_FUNCTION_ATTACH_DEVICE) as i32 {
        (*LunExtension).Common.DeviceObject = (*Srb).DataBuffer as PDEVICE_OBJECT;
    }

    (*Srb).DataBuffer = DeviceObject as PVOID;
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast(), Irql);
    (*Srb).SrbStatus = SRB_STATUS_SUCCESS;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn ScsiPortDispatchScsi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut portExt: PSCSI_PORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut lunExt: PSCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut Irql: KIRQL = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut NextIrp: PIRP = core::ptr::null_mut();
    let mut IrpList: PIRP = core::ptr::null_mut();
    let mut Entry: PKDEVICE_QUEUE_ENTRY = core::ptr::null_mut();

    // DPRINT entry
    {
        let file: *const i8 = b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
        if crate::glue::dprint_noop!(
            b"ScsiPortDispatchScsi(DeviceObject %p  Irp %p)\n\0".as_ptr() as *const i8,
            DeviceObject,
            Irp,
        ) != 0
        {
            DbgPrint(b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8, file, 138i32);
        }
    }

    Stack = IoGetCurrentIrpStackLocation(Irp);
    Srb = (*Stack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    lunExt = (*DeviceObject).DeviceExtension as PSCSI_PORT_LUN_EXTENSION;

    // ASSERT(!lunExt->Common.IsFDO)
    if !((*lunExt).Common.IsFDO == 0) {
        RtlAssert(
            b"!lunExt->Common.IsFDO\0".as_ptr() as *const i8,
            b"drivers/storage/port/scsiport/scsi.c\0"
                .as_ptr() as *const i8,
            141u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    portExt = (*(*lunExt).Common.LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;

    if Srb.is_null() {
        let file: *const i8 = b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
        if DbgPrint(
            b"(%s:%d) ScsiPortDispatchScsi() called with Srb = NULL!\n\0".as_ptr() as *const i8,
            file,
            146i32,
        ) != 0
        {
            DbgPrint(b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8, file, 146i32);
        }
        Status = STATUS_UNSUCCESSFUL;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    // DPRINT: "Srb: %p, Srb->Function: %lu"
    let file: *const i8 = b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
    if crate::glue::dprint_noop!(
        b"Srb: %p, Srb->Function: %lu\n\0".as_ptr() as *const i8,
        Srb,
        (*Srb).Function as u32,
    ) != 0
    {
        DbgPrint(b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8, file, 150i32);
    }

    (*Srb).PathId = (*lunExt).PathId;
    (*Srb).TargetId = (*lunExt).TargetId;
    (*Srb).Lun = (*lunExt).Lun;

    if lunExt.is_null() {
        let file: *const i8 = b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
        if DbgPrint(
            b"(%s:%d) ScsiPortDispatchScsi() called with an invalid LUN\n\0".as_ptr()
                as *const i8,
            file,
            157i32,
        ) != 0
        {
            DbgPrint(b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8, file, 157i32);
        }
        Status = STATUS_NO_SUCH_DEVICE;
        (*Srb).SrbStatus = SRB_STATUS_NO_DEVICE;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return Status;
    }

    let mut pending_execute_scsi: bool = false;

    match (*Srb).Function {
        SRB_FUNCTION_SHUTDOWN | SRB_FUNCTION_FLUSH => {
            if (*portExt).CachesData as i32 == (FALSE) as i32 {
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                IoCompleteRequest(Irp, IO_NO_INCREMENT);
                return STATUS_SUCCESS;
            }
            pending_execute_scsi = true;
        }
        SRB_FUNCTION_EXECUTE_SCSI | SRB_FUNCTION_IO_CONTROL => {
            pending_execute_scsi = true;
        }
        SRB_FUNCTION_CLAIM_DEVICE | SRB_FUNCTION_ATTACH_DEVICE => {
            Status = SpiHandleAttachRelease(lunExt, Irp);
        }
        SRB_FUNCTION_RELEASE_DEVICE => {
            Status = SpiHandleAttachRelease(lunExt, Irp);
        }
        SRB_FUNCTION_RELEASE_QUEUE => {
            KeAcquireSpinLock((&raw mut (*portExt).SpinLock).cast(), &mut Irql);
            if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
                KeReleaseSpinLock((&raw mut (*portExt).SpinLock).cast(), Irql);
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                Status = STATUS_SUCCESS;
            } else {
                (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
                if (*lunExt).SrbInfo.Srb.is_null() {
                    SpiGetNextRequestFromLun(portExt, lunExt, &mut Irql);
                } else {
                    KeReleaseSpinLock((&raw mut (*portExt).SpinLock).cast(), Irql);
                }
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                Status = STATUS_SUCCESS;
            }
        }
        SRB_FUNCTION_FLUSH_QUEUE => {
            KeAcquireSpinLock((&raw mut (*portExt).SpinLock).cast(), &mut Irql);
            if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
                KeReleaseSpinLock((&raw mut (*portExt).SpinLock).cast(), Irql);
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                // ASSERT(lunExt->SrbInfo.Srb == NULL)
                if !(*lunExt).SrbInfo.Srb.is_null() {
                    RtlAssert(
                        b"lunExt->SrbInfo.Srb == NULL\0".as_ptr() as *const i8,
                        b"drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const i8,
                        290u32,
                        core::ptr::null_mut::<i8>(),
                    );
                }
                IrpList = core::ptr::null_mut();
                loop {
                    Entry = KeRemoveDeviceQueue(&mut (*lunExt).DeviceQueue);
                    if Entry.is_null() {
                        break;
                    }
                    let offset = core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry);
                    NextIrp = (Entry as usize).wrapping_sub(offset) as PIRP;
                    Stack = IoGetCurrentIrpStackLocation(NextIrp);
                    Srb = (*Stack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
                    (*Srb).SrbStatus = (SRB_STATUS_REQUEST_FLUSHED) as u8;
                    (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
                    (*NextIrp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink =
                        IrpList as PLIST_ENTRY as *mut _LIST_ENTRY;
                    IrpList = NextIrp;
                }
                (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
                KeReleaseSpinLock((&raw mut (*portExt).SpinLock).cast(), Irql);
                while !IrpList.is_null() {
                    NextIrp = IrpList;
                    IrpList = (*NextIrp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink as PIRP;
                    IoCompleteRequest(NextIrp, 0i8 as CCHAR);
                }
                Status = STATUS_SUCCESS;
            }
        }
        _ => {
            let file: *const i8 = b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
            if DbgPrint(
                b"(%s:%d) SRB function not implemented (Function %lu)\n\0".as_ptr()
                    as *const i8,
                file,
                330i32,
                (*Srb).Function as u32,
            ) != 0
            {
                DbgPrint(b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8, file, 330i32);
            }
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    if pending_execute_scsi {
        IoMarkIrpPending(Irp);
        if (*Srb).SrbFlags & SRB_FLAGS_BYPASS_FROZEN_QUEUE != 0 {
            IoStartPacket(
                (*portExt).Common.DeviceObject,
                Irp,
                core::ptr::null_mut::<core::ffi::c_ulong>(),
                None,
            );
        } else {
            let mut oldIrql: KIRQL = 0;
            KeRaiseIrql(DISPATCH_LEVEL, &mut oldIrql);
            if KeInsertByKeyDeviceQueue(
                &mut (*lunExt).DeviceQueue,
                &mut (*Irp).Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry,
                *(*Srb).__bindgen_anon_1.QueueSortKey.as_ref(),
            ) == 0
            {
                IoStartPacket(
                    (*portExt).Common.DeviceObject,
                    Irp,
                    core::ptr::null_mut::<core::ffi::c_ulong>(),
                    None,
                );
            }
            KeLowerIrql(oldIrql);
        }
        return STATUS_PENDING;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SpiSenseCompletionRoutine(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
    Context: ntkrnl_sys::ddk::PVOID,
) -> ntkrnl_sys::base::NTSTATUS {
    let ioStack = IoGetNextIrpStackLocation(Irp);
    let lunExt = (*(*ioStack).DeviceObject).DeviceExtension as crate::types::PSCSI_PORT_LUN_EXTENSION;
    let portExt = (*(*lunExt).Common.LowerDevice).DeviceExtension as crate::types::PSCSI_PORT_DEVICE_EXTENSION;
    let Srb: crate::types::PSCSI_REQUEST_BLOCK = Context as crate::types::PSCSI_REQUEST_BLOCK;
    let InitialSrb: crate::types::PSCSI_REQUEST_BLOCK;
    let InitialIrp: ntkrnl_sys::ddk::PIRP;

    if (*Srb).Function as i32 == (SRB_FUNCTION_RESET_BUS) as i32
        || (*Srb).Function as i32 == (SRB_FUNCTION_ABORT_COMMAND) as i32
    {
        ExFreePool(Srb as ntkrnl_sys::ddk::PVOID);
        IoFreeIrp(Irp);
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    InitialSrb = *(Srb.add(1) as *mut ntkrnl_sys::ddk::PVOID) as crate::types::PSCSI_REQUEST_BLOCK;
    InitialIrp = (*InitialSrb).OriginalRequest as ntkrnl_sys::ddk::PIRP;

    if ((*Srb).SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_SUCCESS) as i32
        || ((*Srb).SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_DATA_OVERRUN) as i32
    {
        (*InitialSrb).SrbStatus = ((*InitialSrb).SrbStatus as i32 | SRB_STATUS_AUTOSENSE_VALID as i32) as u8;
        (*InitialSrb).SenseInfoBufferLength = (*Srb).DataTransferLength as u8;
    }

    if (*InitialSrb).SrbStatus as i32 & (SRB_STATUS_QUEUE_FROZEN) as i32 == 0 {
        RtlAssert(
            b"InitialSrb->SrbStatus & SRB_STATUS_QUEUE_FROZEN\0".as_ptr() as *mut core::ffi::c_void,
            b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *mut core::ffi::c_void,
            496 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (*InitialSrb).SrbFlags & SRB_FLAGS_NO_QUEUE_FREEZE != 0
        && (*InitialSrb).SrbStatus as i32 & (SRB_STATUS_QUEUE_FROZEN) as i32 != 0
    {
        let mut irql: ntkrnl_sys::base::KIRQL = 0;
        KeAcquireSpinLock((&raw mut (*portExt).SpinLock).cast(), &raw mut irql);
        if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
            RtlAssert(
                b"lunExt->Flags & LUNEX_FROZEN_QUEUE\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *mut core::ffi::c_void,
                506 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
        (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
        (*lunExt).Flags &= !LUNEX_NEED_REQUEST_SENSE;
        SpiGetNextRequestFromLun(portExt, lunExt, &raw mut irql);
        (*InitialSrb).SrbStatus = ((*InitialSrb).SrbStatus as i32 & (!SRB_STATUS_QUEUE_FROZEN) as i32) as u8;
    }

    IoCompleteRequest(InitialIrp, IO_DISK_INCREMENT as i8);

    ExFreePool(Srb as ntkrnl_sys::ddk::PVOID);

    if !(*Irp).MdlAddress.is_null() {
        MmUnlockPages((*Irp).MdlAddress);
        IoFreeMdl((*Irp).MdlAddress);
        (*Irp).MdlAddress = core::ptr::null_mut();
    }

    IoFreeIrp(Irp);
    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe fn SpiSendRequestSense(
    LunExtension: PSCSI_PORT_LUN_EXTENSION,
    InitialSrb: PSCSI_REQUEST_BLOCK,
) {
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut Cdb: PCDB = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut LargeInt: LARGE_INTEGER = core::mem::zeroed();
    let mut Ptr: *mut PVOID = core::ptr::null_mut();

    Srb = ExAllocatePoolWithTag(
        NonPagedPool,
        ((core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32)
            .wrapping_add(core::mem::size_of::<PVOID>() as u32)) as usize,
        TAG_SCSIPORT,
    ) as PSCSI_REQUEST_BLOCK;

    core::ptr::write_bytes(
        Srb as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
    );

    LargeInt.QuadPart = 1;
    Irp = IoBuildAsynchronousFsdRequest(
        IRP_MJ_READ as u32,
        (*LunExtension).Common.DeviceObject,
        (*InitialSrb).SenseInfoBuffer,
        (*InitialSrb).SenseInfoBufferLength as u32,
        &raw mut LargeInt,
        core::ptr::null_mut(),
    );

    IoSetCompletionRoutine(
        Irp,
        Some(core::mem::transmute(SpiSenseCompletionRoutine as *const ())),
        Srb as PVOID,
        1u8,
        1u8,
        1u8,
    );

    if Srb.is_null() {
        return;
    }

    IrpStack = IoGetNextIrpStackLocation(Irp);
    (*IrpStack).MajorFunction = IRP_MJ_SCSI;
    (*IrpStack).Parameters.Others.Argument1 = Srb as PVOID;

    (*Srb).OriginalRequest = Irp as PVOID;

    Ptr = Srb.offset(1) as *mut PVOID;
    *Ptr = InitialSrb as PVOID;

    (*Srb).CdbLength = 6;
    Cdb = &raw mut (*Srb).Cdb as *mut u8 as PCDB;

    (*Cdb).CDB6INQUIRY.as_mut().OperationCode = SCSIOP_REQUEST_SENSE;
    (*Cdb).CDB6INQUIRY.as_mut().set_LogicalUnitNumber(0u8);
    (*Cdb).CDB6INQUIRY.as_mut().set_Reserved1(0u8);
    (*Cdb).CDB6INQUIRY.as_mut().PageCode = 0u8;
    (*Cdb).CDB6INQUIRY.as_mut().IReserved = 0u8;
    (*Cdb).CDB6INQUIRY.as_mut().AllocationLength = (*InitialSrb).SenseInfoBufferLength;
    (*Cdb).CDB6INQUIRY.as_mut().Control = 0u8;

    (*Srb).TargetId = (*InitialSrb).TargetId;
    (*Srb).Lun = (*InitialSrb).Lun;
    (*Srb).PathId = (*InitialSrb).PathId;

    (*Srb).Function = SRB_FUNCTION_EXECUTE_SCSI;
    (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
    (*Srb).TimeOutValue = 2;
    (*Srb).SenseInfoBufferLength = 0;
    (*Srb).SenseInfoBuffer = core::ptr::null_mut();

    (*Srb).SrbFlags = (SRB_FLAGS_DATA_IN | SRB_FLAGS_BYPASS_FROZEN_QUEUE | SRB_FLAGS_DISABLE_DISCONNECT) as u32;

    if ((*InitialSrb).SrbFlags as u32) & SRB_FLAGS_DISABLE_SYNCH_TRANSFER != 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
    }
    if ((*InitialSrb).SrbFlags as u32) & SRB_FLAGS_BYPASS_LOCKED_QUEUE != 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_BYPASS_LOCKED_QUEUE;
    }
    if ((*InitialSrb).SrbFlags as u32) & SRB_FLAGS_NO_QUEUE_FREEZE != 0 {
        (*Srb).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
    }

    (*Srb).DataBuffer = (*InitialSrb).SenseInfoBuffer;
    (*Srb).DataTransferLength = ((*InitialSrb).SenseInfoBufferLength) as u32;

    (*Srb).SrbStatus = 0;
    (*Srb).ScsiStatus = (*Srb).SrbStatus;
    (*Srb).NextSrb = core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>();

    IoCallDriver((*LunExtension).Common.DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortStartPacket(Context: *mut core::ffi::c_void) -> i32 {
    // (LLM-fabricated local consts deleted — glue.rs/types are the verified source.)
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Srb: *mut _SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = Context as PDEVICE_OBJECT;
    let mut CommonExtension: *mut _SCSI_PORT_COMMON_EXTENSION =
        (*DeviceObject).DeviceExtension as *mut _SCSI_PORT_COMMON_EXTENSION;
    let mut DeviceExtension: *mut _SCSI_PORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LunExtension: *mut _SCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();
    let mut SrbInfo: *mut _SCSI_REQUEST_BLOCK_INFO = core::ptr::null_mut();
    let mut Result: i32 = 0;
    let mut StartTimer: i32 = 0;

    // DPRINT removed

    IrpStack = IoGetCurrentIrpStackLocation((*DeviceObject).CurrentIrp as *mut IRP);
    Srb = (*IrpStack).Parameters.Scsi.Srb as *mut _SCSI_REQUEST_BLOCK;

    if (*CommonExtension).IsFDO != 0 {
        DeviceExtension = (*DeviceObject).DeviceExtension as *mut _SCSI_PORT_DEVICE_EXTENSION;
        LunExtension = (*(*IrpStack).DeviceObject).DeviceExtension as *mut _SCSI_PORT_LUN_EXTENSION;
        if !(!LunExtension.is_null() && (*LunExtension).Common.IsFDO == 0) {
            RtlAssert(
                b"LunExtension && !LunExtension->Common.IsFDO\0".as_ptr() as *const i8,
                b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8,
                1008u32,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }
    } else {
        LunExtension = (*DeviceObject).DeviceExtension as *mut _SCSI_PORT_LUN_EXTENSION;
        DeviceExtension = (*(*LunExtension).Common.LowerDevice).DeviceExtension as *mut _SCSI_PORT_DEVICE_EXTENSION;
    }

    if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_RESET) != 0 {
        (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_RESET_REQUEST;
        return (TRUE) as i32;
    }

    (*DeviceExtension).TimerCount = (*Srb).TimeOutValue as i32;
    (*DeviceExtension).Flags |= SCSI_PORT_DEVICE_BUSY;

    if (*LunExtension).RequestTimeout != -1i32 as i32 {
        StartTimer = (FALSE) as i32;
    } else {
        (*LunExtension).RequestTimeout = (*Srb).TimeOutValue as i32;
        StartTimer = (TRUE) as i32;
    }

    if ((*Srb).SrbFlags as u32 & SRB_FLAGS_BYPASS_FROZEN_QUEUE) != 0 {
        if (*Srb).Function as i32 == (SRB_FUNCTION_ABORT_COMMAND) as i32 {
            SrbInfo = SpiGetSrbData(DeviceExtension, LunExtension, (*Srb).QueueTag);
            if SrbInfo.is_null()
                || (*SrbInfo).Srb.is_null()
                || ((*(*SrbInfo).Srb).SrbFlags as u32 & SRB_FLAGS_IS_ACTIVE) == 0
            {
                (*Srb).SrbFlags |= SRB_FLAGS_IS_ACTIVE;
                if StartTimer != 0 {
                    (*LunExtension).RequestTimeout = -1i32 as i32;
                }
                // DPRINT removed
                (*Srb).SrbStatus = SRB_STATUS_ABORT_FAILED;
                ScsiPortNotification(RequestComplete, (*DeviceExtension).MiniPortDeviceExtension.as_mut_ptr() as *mut core::ffi::c_void, Srb);
                ScsiPortNotification(NextRequest, (*DeviceExtension).MiniPortDeviceExtension.as_mut_ptr() as *mut core::ffi::c_void);
                IoRequestDpc(
                    (*DeviceExtension).Common.DeviceObject,
                    core::ptr::null_mut::<IRP>(),
                    core::ptr::null_mut(),
                );
                return (TRUE) as i32;
            }
        } else {
            (*LunExtension).QueueCount = (*LunExtension).QueueCount.wrapping_add(1);
        }

        (*LunExtension).Flags &= !LUNEX_NEED_REQUEST_SENSE;
        if ((*Srb).SrbFlags as u32 & SRB_FLAGS_DISABLE_DISCONNECT) != 0 {
            (*DeviceExtension).Flags &= !SCSI_PORT_DISCONNECT_ALLOWED;
        }
        (*LunExtension).RequestTimeout = (*Srb).TimeOutValue as i32;
    } else {
        if ((*Srb).SrbFlags as u32 & SRB_FLAGS_DISABLE_DISCONNECT) != 0 {
            (*DeviceExtension).Flags &= !SCSI_PORT_DISCONNECT_ALLOWED;
        }
        (*LunExtension).Flags |= SCSI_PORT_LU_ACTIVE;
        (*LunExtension).QueueCount = (*LunExtension).QueueCount.wrapping_add(1);
        if (*Srb).QueueTag as i32 != (SP_UNTAGGED) as i32 {
            SrbInfo = &mut *((*DeviceExtension).SrbInfo.offset(((*Srb).QueueTag as i32 - 1i32) as isize))
                as *mut _SCSI_REQUEST_BLOCK_INFO;
            if !(*SrbInfo).Requests.Blink.is_null() {
                RtlAssert(
                    b"SrbInfo->Requests.Blink == NULL\0".as_ptr() as *const i8,
                    b"drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8,
                    1120u32,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            InsertTailList(
                core::ptr::addr_of_mut!((*LunExtension).SrbInfo.Requests),
                core::ptr::addr_of_mut!((*SrbInfo).Requests),
            );
        }
    }

    (*Srb).SrbFlags |= SRB_FLAGS_IS_ACTIVE;
    Result = if let Some(hw_start_io) = (*DeviceExtension).HwStartIo {
        (hw_start_io(
            core::ptr::addr_of_mut!((*DeviceExtension).MiniPortDeviceExtension) as *mut core::ffi::c_void,
            Srb,
        )) as i32
    } else {
        // Function pointer is NULL; this shouldn't happen
        0
    };

    if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
        IoRequestDpc(
            (*DeviceExtension).Common.DeviceObject,
            core::ptr::null_mut::<IRP>(),
            core::ptr::null_mut(),
        );
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn ScsiPortDpcForIsr(
    Dpc: PKDPC,
    DpcDeviceObject: PDEVICE_OBJECT,
    DpcIrp: PIRP,
    DpcContext: PVOID,
) {
    let DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION =
        (*DpcDeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
    let mut InterruptData: SCSI_PORT_INTERRUPT_DATA = core::mem::zeroed();
    let mut Context: SCSI_PORT_SAVE_INTERRUPT = core::mem::zeroed();
    let mut LunExtension: PSCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();
    let mut NeedToStartIo: BOOLEAN = 0;
    let mut SrbInfo: PSCSI_REQUEST_BLOCK_INFO = core::ptr::null_mut();
    let mut TimerValue: LARGE_INTEGER = core::mem::zeroed();

    // We need to acquire spinlock
    KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
    core::ptr::write_bytes(
        &raw mut InterruptData as *mut u8,
        0,
        core::mem::size_of::<SCSI_PORT_INTERRUPT_DATA>(),
    );

    'try_again: loop {
        // Interrupt structure must be snapshotted, and only then analyzed
        Context.InterruptData = &raw mut InterruptData as PSCSI_PORT_INTERRUPT_DATA;
        Context.DeviceExtension = DeviceExtension;

        if KeSynchronizeExecution(
            (*DeviceExtension).Interrupt[0],
            Some(core::mem::transmute(SpiSaveInterruptData as *const ())),
            &raw mut Context as PVOID,
        ) == 0
        {
            // Nothing - just return (don't forget to release the spinlock)
            KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            return;
        }

        // If flush of adapters is needed - do it
        if (InterruptData.Flags as u32 & SCSI_PORT_FLUSH_ADAPTERS) != 0 {
            // TODO: Implement
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1302,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }

        // Check for IoMapTransfer
        if (InterruptData.Flags as u32 & SCSI_PORT_MAP_TRANSFER) != 0 {
            // TODO: Implement
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1309,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }

        // Check if timer is needed
        if (InterruptData.Flags as u32 & SCSI_PORT_TIMER_NEEDED) != 0 {
            // Save the timer routine
            (*DeviceExtension).HwScsiTimer = InterruptData.HwScsiTimer;

            if InterruptData.MiniportTimerValue == 0 {
                // Cancel the timer
                KeCancelTimer(&raw mut (*DeviceExtension).MiniportTimer);
            } else {
                // Convert timer value
                TimerValue.QuadPart = Int32x32To64(InterruptData.MiniportTimerValue as i32, -10i32) as i64;

                // Set the timer
                KeSetTimer(
                    &raw mut (*DeviceExtension).MiniportTimer,
                    TimerValue,
                    &raw mut (*DeviceExtension).MiniportTimerDpc,
                );
            }
        }

        // A miniport reported a bus change: have PnP re-query our bus
        // relations, which rescans the adapter. LowerDevice is the PDO that
        // ScsiPortInitialize got from IoReportDetectedDevice.
        if (InterruptData.Flags as u32 & crate::scsiport::SCSI_PORT_BUS_CHANGE_DETECTED) != 0
            && !(*DeviceExtension).Common.LowerDevice.is_null()
        {
            ntkrnl_sys::ddk::IoInvalidateDeviceRelations(
                (*DeviceExtension).Common.LowerDevice,
                ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations,
            );
        }

        // If it's ready for the next request
        if (InterruptData.Flags as u32 & SCSI_PORT_NEXT_REQUEST_READY) != 0 {
            // Check for a duplicate request (NextRequest+NextLuRequest)
            if ((*DeviceExtension).Flags as u32
                & (SCSI_PORT_DEVICE_BUSY | SCSI_PORT_DISCONNECT_ALLOWED))
                == (SCSI_PORT_DEVICE_BUSY | SCSI_PORT_DISCONNECT_ALLOWED)
            {
                // Clear busy flag set by ScsiPortStartPacket()
                (*DeviceExtension).Flags &= !SCSI_PORT_DEVICE_BUSY;

                if (InterruptData.Flags as u32 & SCSI_PORT_RESET) == 0 {
                    // Ready for next, and no reset is happening
                    (*DeviceExtension).TimerCount = -1i32 as i32;
                }
            } else {
                // Not busy, but not ready for the next request
                (*DeviceExtension).Flags &= !SCSI_PORT_DEVICE_BUSY;
                InterruptData.Flags &= !SCSI_PORT_NEXT_REQUEST_READY;
            }
        }

        // Any resets?
        if (InterruptData.Flags as u32 & SCSI_PORT_RESET_REPORTED) != 0 {
            // Hold for a bit
            (*DeviceExtension).TimerCount = 4i32 as i32;
        }

        // Any ready LUN?
        if !InterruptData.ReadyLun.is_null() {
            // Process all LUNs from the list
            loop {
                // Remove it from the list first (as processed)
                LunExtension = InterruptData.ReadyLun;
                InterruptData.ReadyLun = (*LunExtension).ReadyLun;
                (*LunExtension).ReadyLun = core::ptr::null_mut();

                // Get next request for this LUN
                SpiGetNextRequestFromLun(DeviceExtension, LunExtension, core::ptr::null_mut());

                // Still ready requests exist? If yes - get spinlock, if no - stop here
                if InterruptData.ReadyLun.is_null() {
                    break;
                }
                KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            }
        } else {
            // Release the spinlock
            KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        }

        // If we ready for next packet, start it
        if (InterruptData.Flags as u32 & SCSI_PORT_NEXT_REQUEST_READY) != 0 {
            IoStartNextPacket((*DeviceExtension).Common.DeviceObject, (0i32) as u8);
        }

        NeedToStartIo = (0i32) as u8;

        // Loop the completed request list
        while !InterruptData.CompletedRequests.is_null() {
            // Remove the request
            SrbInfo = InterruptData.CompletedRequests;
            InterruptData.CompletedRequests = (*SrbInfo).CompletedRequests;
            (*SrbInfo).CompletedRequests = core::ptr::null_mut();

            // Process it
            SpiProcessCompletedRequest(DeviceExtension, SrbInfo, &raw mut NeedToStartIo);
        }

        // Loop abort request list
        while !InterruptData.CompletedAbort.is_null() {
            LunExtension = InterruptData.CompletedAbort;

            // Remove the request
            InterruptData.CompletedAbort = (*LunExtension).CompletedAbortRequests;

            // Get spinlock since we're going to change flags
            KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());

            // TODO: Put SrbExtension to the list of free extensions
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1428,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }

        // If we need - call StartIo routine
        if NeedToStartIo != 0 {
            // Make sure CurrentIrp is not null!
            if (*DpcDeviceObject).CurrentIrp.is_null() {
                RtlAssert(
                    b"DpcDeviceObject->CurrentIrp != NULL\0" as *const u8 as PVOID,
                    b"scsi.c\0" as *const u8 as PVOID,
                    1435,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            ScsiPortStartIo(DpcDeviceObject, (*DpcDeviceObject).CurrentIrp as PIRP);
        }

        // Everything has been done, check
        if (InterruptData.Flags as u32 & SCSI_PORT_ENABLE_INT_REQUEST) != 0 {
            // Synchronize using spinlock
            KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());

            // Request an interrupt
            (*DeviceExtension)
                .HwInterrupt
                .expect("non-null function pointer")(&raw mut (*DeviceExtension)
                .MiniPortDeviceExtension as *mut u8 as PVOID);

            // ASSERT(DeviceExtension->Flags & SCSI_PORT_DISABLE_INT_REQUESET)
            if ((*DeviceExtension).Flags as u32 & SCSI_PORT_DISABLE_INT_REQUESET) == 0 {
                RtlAssert(
                    b"DeviceExtension->Flags & SCSI_PORT_DISABLE_INT_REQUESET\0" as *const u8 as PVOID,
                    b"scsi.c\0" as *const u8 as PVOID,
                    1448,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }

            // Should interrupts be enabled again?
            if ((*DeviceExtension).Flags as u32 & SCSI_PORT_DISABLE_INT_REQUESET) != 0 {
                // Clear this flag
                (*DeviceExtension).Flags &= !SCSI_PORT_DISABLE_INT_REQUESET;

                // Call a special routine to do this
                RtlAssert(
                    b"FALSE\0" as *const u8 as PVOID,
                    b"scsi.c\0" as *const u8 as PVOID,
                    1457,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }

            // If we need a notification again - loop
            if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
                continue 'try_again;
            }

            // Release the spinlock
            KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            break;
        }
        break;
    }
}
unsafe fn SpiAllocateSrbStructures(
    DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION,
    LunExtension: PSCSI_PORT_LUN_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
) -> PSCSI_REQUEST_BLOCK_INFO {
    let mut SrbExtension: PCHAR = core::ptr::null_mut();
    let mut SrbInfo: PSCSI_REQUEST_BLOCK_INFO = core::ptr::null_mut();
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
    if (*DeviceExtension).NeedSrbDataAlloc != 0 {
        if (*Srb).Function as core::ffi::c_int == (SRB_FUNCTION_ABORT_COMMAND) as i32 {
            SrbInfo = SpiGetSrbData(DeviceExtension, LunExtension, (*Srb).QueueTag);
        } else if (*Srb).SrbFlags as core::ffi::c_ulong
            & (SRB_FLAGS_QUEUE_ACTION_ENABLE | SRB_FLAGS_NO_QUEUE_FREEZE) as core::ffi::c_ulong
            != 0
            && (*Srb).SrbFlags as core::ffi::c_ulong
                & SRB_FLAGS_DISABLE_DISCONNECT as core::ffi::c_ulong
                == 0
        {
            if (*LunExtension).Flags as core::ffi::c_ulong
                & LUNEX_NEED_REQUEST_SENSE as core::ffi::c_ulong
                != 0
            {
                if (*LunExtension).Flags as core::ffi::c_ulong & 0x20 as core::ffi::c_ulong != 0 {
                    RtlAssert(
                        b"!(LunExtension->Flags & LUNEX_REQUEST_PENDING)\0".as_ptr()
                            as *const core::ffi::c_char as *mut core::ffi::c_void,
                        b"drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                        1505 as ULONG,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                }
                (*LunExtension).PendingRequest = (*Srb).OriginalRequest as PIRP;
                (*LunExtension).Flags |=
                    (LUNEX_REQUEST_PENDING | SCSI_PORT_LU_ACTIVE) as core::ffi::c_ulong;
                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
                return core::ptr::null_mut();
            }
            if !(*LunExtension).SrbInfo.Srb.is_null() {
                RtlAssert(
                    b"LunExtension->SrbInfo.Srb == NULL\0".as_ptr() as *const core::ffi::c_char
                        as *mut core::ffi::c_void,
                    b"drivers/storage/port/scsiport/scsi.c\0"
                        .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                    1515 as ULONG,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            SrbInfo = (*DeviceExtension).FreeSrbInfo;
            if SrbInfo.is_null() {
                (*DeviceExtension).Flags |= SCSI_PORT_REQUEST_PENDING as core::ffi::c_ulong;
                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
                return core::ptr::null_mut();
            }
            (*DeviceExtension).FreeSrbInfo = (*SrbInfo).Requests.Flink as PSCSI_REQUEST_BLOCK_INFO;
            (*Srb).QueueTag = (SrbInfo.offset_from((*DeviceExtension).SrbInfo)
                as core::ffi::c_longlong as UCHAR as core::ffi::c_int
                + 1 as core::ffi::c_int) as UCHAR;
        } else {
            if (!IsListEmpty(core::ptr::addr_of_mut!((*LunExtension).SrbInfo.Requests))
                || (*LunExtension).Flags as core::ffi::c_ulong
                    & LUNEX_NEED_REQUEST_SENSE as core::ffi::c_ulong
                    != 0)
                && (*Srb).SrbFlags as core::ffi::c_ulong
                    & SRB_FLAGS_BYPASS_FROZEN_QUEUE as core::ffi::c_ulong
                    == 0
            {
                if (*LunExtension).Flags as core::ffi::c_ulong & 0x20 as core::ffi::c_ulong != 0 {
                    RtlAssert(
                        b"!(LunExtension->Flags & LUNEX_REQUEST_PENDING)\0".as_ptr()
                            as *const core::ffi::c_char as *mut core::ffi::c_void,
                        b"drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                        1543 as ULONG,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                }
                (*LunExtension).Flags |=
                    (LUNEX_REQUEST_PENDING | SCSI_PORT_LU_ACTIVE) as core::ffi::c_ulong;
                (*LunExtension).PendingRequest = (*Srb).OriginalRequest as PIRP;
                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
                return core::ptr::null_mut();
            }
            (*Srb).QueueTag = SP_UNTAGGED;
            SrbInfo = core::ptr::addr_of_mut!((*LunExtension).SrbInfo) as PSCSI_REQUEST_BLOCK_INFO;
        }
    } else {
        (*Srb).QueueTag = SP_UNTAGGED;
        SrbInfo = core::ptr::addr_of_mut!((*LunExtension).SrbInfo) as PSCSI_REQUEST_BLOCK_INFO;
    }
    if (*DeviceExtension).NeedSrbExtensionAlloc != 0 {
        SrbExtension = (*DeviceExtension).FreeSrbExtensions as PCHAR;
        if SrbExtension.is_null() {
            if (*Srb).Function as core::ffi::c_int != (SRB_FUNCTION_ABORT_COMMAND) as i32
                && (*Srb).QueueTag as core::ffi::c_int != SP_UNTAGGED as core::ffi::c_int
            {
                (*SrbInfo).Requests.Blink = core::ptr::null_mut::<LIST_ENTRY>();
                (*SrbInfo).Requests.Flink =
                    (*DeviceExtension).FreeSrbInfo as PLIST_ENTRY as *mut LIST_ENTRY;
                (*DeviceExtension).FreeSrbInfo = SrbInfo;
            }
            (*DeviceExtension).Flags |= SCSI_PORT_REQUEST_PENDING as core::ffi::c_ulong;
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
            return core::ptr::null_mut();
        }
        (*DeviceExtension).FreeSrbExtensions = *(SrbExtension as *mut PVOID);
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
        (*Srb).SrbExtension = SrbExtension as PVOID;
        if !(*Srb).SenseInfoBuffer.is_null()
            && (*DeviceExtension).SupportsAutoSense as core::ffi::c_int != 0
        {
            (*SrbInfo).SaveSenseRequest = (*Srb).SenseInfoBuffer;
            if (*Srb).SenseInfoBufferLength as usize > core::mem::size_of::<SENSE_DATA>() as usize {
                (*Srb).SrbFlags |= SRB_FLAGS_DISABLE_AUTOSENSE as core::ffi::c_ulong;
            } else {
                (*Srb).SenseInfoBuffer =
                    SrbExtension.offset((*DeviceExtension).SrbExtensionSize as isize) as PVOID;
            }
        }
    } else {
        (*Srb).SrbExtension = core::ptr::null_mut::<core::ffi::c_void>();
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
    }
    return SrbInfo;
}


// ── Recovered bodies (assembler dropped these 4 to forward-decls; re-spliced from
// tools/rustconv/.responses/20260612-145033-scsi.c — $0, no re-translate). ──
#[no_mangle]
pub unsafe extern "C" fn SpiGetNextRequestFromLun(
    mut DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION,
    mut LunExtension: PSCSI_PORT_LUN_EXTENSION,
    mut OldIrql: PKIRQL,
) {
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut::<_IO_STACK_LOCATION>();
    let mut NextIrp: PIRP = core::ptr::null_mut::<_IRP>();
    let mut Entry: PKDEVICE_QUEUE_ENTRY = core::ptr::null_mut::<_KDEVICE_QUEUE_ENTRY>();
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>();
    if (*LunExtension).QueueCount >= (*LunExtension).MaxQueueCount
        || ((*LunExtension).Flags & SCSI_PORT_LU_ACTIVE) == 0
    {
        if !OldIrql.is_null() {
            KeReleaseSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), *OldIrql);
        } else {
            KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        }
        return;
    }

    if ((*LunExtension).Flags
        & (LUNEX_NEED_REQUEST_SENSE
            | LUNEX_BUSY
            | LUNEX_FULL_QUEUE
            | LUNEX_FROZEN_QUEUE
            | LUNEX_REQUEST_PENDING))
        != 0
    {
        if IsListEmpty(&raw mut (*LunExtension).SrbInfo.Requests)
            && ((*LunExtension).Flags
                & (LUNEX_BUSY | LUNEX_FROZEN_QUEUE | LUNEX_FULL_QUEUE | LUNEX_NEED_REQUEST_SENSE))
                == 0
        {
            if !(*LunExtension).SrbInfo.Srb.is_null() {
                RtlAssert(
                    b"LunExtension->SrbInfo.Srb == NULL\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                    b"scsi.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                    377u32,
                    core::ptr::null_mut::<i8>(),
                );
            }
            (*LunExtension).Flags &= !(LUNEX_REQUEST_PENDING | SCSI_PORT_LU_ACTIVE);
            NextIrp = (*LunExtension).PendingRequest;
            (*LunExtension).PendingRequest = core::ptr::null_mut::<_IRP>();
            (*LunExtension).AttemptCount = 0u32;
            if !OldIrql.is_null() {
                KeReleaseSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), *OldIrql);
            } else {
                KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            }
            IoStartPacket(
                (*DeviceExtension).Common.DeviceObject,
                NextIrp,
                core::ptr::null_mut::<u32>(),
                None,
            );
            return;
        } else {
            if !OldIrql.is_null() {
                KeReleaseSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), *OldIrql);
            } else {
                KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            }
            return;
        }
    }

    (*LunExtension).Flags &= !SCSI_PORT_LU_ACTIVE;
    (*LunExtension).AttemptCount = 0u32;

    Entry = KeRemoveByKeyDeviceQueue(
        &raw mut (*LunExtension).DeviceQueue,
        (*LunExtension).SortKey,
    );

    if !Entry.is_null() {
        let offset = core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry);
        NextIrp = (Entry as usize).wrapping_sub(offset) as *mut IRP as PIRP;

        IrpStack = IoGetCurrentIrpStackLocation(NextIrp);
        Srb = (*IrpStack).Parameters.Others.Argument1 as PSCSI_REQUEST_BLOCK;

        (*LunExtension).SortKey = *(*Srb).__bindgen_anon_1.QueueSortKey.as_ref();
        (*LunExtension).SortKey = (*LunExtension).SortKey.wrapping_add(1);

        if !OldIrql.is_null() {
            KeReleaseSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), *OldIrql);
        } else {
            KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        }

        IoStartPacket(
            (*DeviceExtension).Common.DeviceObject,
            NextIrp,
            core::ptr::null_mut::<u32>(),
            None,
        );
    } else if !OldIrql.is_null() {
        KeReleaseSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), *OldIrql);
    } else {
        KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
    }
}

pub unsafe extern "system" fn SpiSaveInterruptData(Context: *mut core::ffi::c_void) -> i32 {
    let mut InterruptContext: *mut SCSI_PORT_SAVE_INTERRUPT = Context as *mut SCSI_PORT_SAVE_INTERRUPT;
    let mut LunExtension: *mut SCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();
    let mut Srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut SrbInfo: *mut SCSI_REQUEST_BLOCK_INFO = core::ptr::null_mut();
    let mut NextSrbInfo: *mut SCSI_REQUEST_BLOCK_INFO = core::ptr::null_mut();
    let mut DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut IsTimed: i32 = 0;

    DeviceExtension = (*InterruptContext).DeviceExtension;

    if ((*DeviceExtension).InterruptData.Flags & SCSI_PORT_NOTIFICATION_NEEDED) == 0 {
        return 0;
    }

    core::ptr::write((*InterruptContext).InterruptData, core::ptr::read(&raw const (*DeviceExtension).InterruptData));

    (*DeviceExtension).InterruptData.Flags &=
        (SCSI_PORT_RESET | SCSI_PORT_RESET_REQUEST | SCSI_PORT_DISABLE_INTERRUPTS);
    (*DeviceExtension).InterruptData.CompletedAbort = core::ptr::null_mut();
    (*DeviceExtension).InterruptData.ReadyLun = core::ptr::null_mut();
    (*DeviceExtension).InterruptData.CompletedRequests = core::ptr::null_mut();

    SrbInfo = (*(*InterruptContext).InterruptData).CompletedRequests;

    while !SrbInfo.is_null() {
        if (*SrbInfo).Srb.is_null() {
            RtlAssert(
                b"SrbInfo->Srb\0".as_ptr() as *const i8,
                b"drivers/storage/port/scsiport/scsi.c\0"
                    .as_ptr() as *const i8,
                1175u32,
                core::ptr::null() as *const i8,
            );
        }

        Srb = (*SrbInfo).Srb;

        let IoStack = IoGetCurrentIrpStackLocation((*Srb).OriginalRequest as PIRP);
        LunExtension = (*(*IoStack).DeviceObject).DeviceExtension as *mut SCSI_PORT_LUN_EXTENSION;

        if !(!LunExtension.is_null() && (*LunExtension).Common.IsFDO == 0) {
            RtlAssert(
                b"LunExtension && !LunExtension->Common.IsFDO\0".as_ptr() as *const i8,
                b"drivers/storage/port/scsiport/scsi.c\0"
                    .as_ptr() as *const i8,
                1182u32,
                core::ptr::null() as *const i8,
            );
        }

        if (*Srb).SrbStatus != SRB_STATUS_SUCCESS {
            if !(*Srb).SenseInfoBuffer.is_null()
                && (*Srb).SenseInfoBufferLength != 0
                && (*Srb).ScsiStatus == SCSISTAT_CHECK_CONDITION
                && (*Srb).SrbStatus & SRB_STATUS_AUTOSENSE_VALID == 0
            {
                if (*LunExtension).Flags & LUNEX_NEED_REQUEST_SENSE != 0 {
                    (*Srb).ScsiStatus = 0;
                    (*Srb).SrbStatus = SRB_STATUS_REQUEST_SENSE_FAILED;
                } else {
                    (*LunExtension).Flags |= LUNEX_NEED_REQUEST_SENSE;
                }
            }
            if (*Srb).ScsiStatus == SCSISTAT_QUEUE_FULL {
                if 0 == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as *const i8,
                        b"drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const i8,
                        1212u32,
                        core::ptr::null() as *const i8,
                    );
                }
            }
        }

        if (*Srb).QueueTag == SP_UNTAGGED {
            IsTimed = 1;
        } else {
            if (*LunExtension).SrbInfo.Requests.Flink == &raw mut (*SrbInfo).Requests {
                IsTimed = 1;
            } else {
                IsTimed = 0;
            }
            RemoveEntryList(&raw mut (*SrbInfo).Requests);
        }

        if IsTimed != 0 {
            if IsListEmpty(&raw mut (*LunExtension).SrbInfo.Requests) {
                (*LunExtension).RequestTimeout = -1i32;
            } else {
                NextSrbInfo = ((*LunExtension).SrbInfo.Requests.Flink as usize).wrapping_sub(
                    core::mem::offset_of!(SCSI_REQUEST_BLOCK_INFO, Requests),
                ) as *mut SCSI_REQUEST_BLOCK_INFO;
                Srb = (*NextSrbInfo).Srb;
                (*LunExtension).RequestTimeout = (*Srb).TimeOutValue as i32;
            }
        }

        SrbInfo = (*SrbInfo).CompletedRequests;
    }

    return 1;
}

unsafe fn SpiProcessCompletedRequest(
    DeviceExtension: *mut _SCSI_PORT_DEVICE_EXTENSION,
    mut SrbInfo: *mut _SCSI_REQUEST_BLOCK_INFO,
    NeedToCallStartIo: *mut u8,
) {
    let mut Srb: *mut _SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut LunExtension: *mut _SCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();
    let mut Result: i32 = 0;
    let mut Irp: *mut IRP = core::ptr::null_mut();

    Srb = (*SrbInfo).Srb;
    Irp = (*Srb).OriginalRequest as *mut IRP;
    let IoStack: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    LunExtension = (*(*IoStack).DeviceObject).DeviceExtension as *mut _SCSI_PORT_LUN_EXTENSION;

    if !(!LunExtension.is_null() && (*LunExtension).Common.IsFDO == 0) {
        RtlAssert(
            b"LunExtension && !LunExtension->Common.IsFDO\0".as_ptr() as *const i8,
            b"drivers/storage/port/scsiport/scsi.c\0"
                .as_ptr() as *const i8,
            663u32,
            core::ptr::null::<core::ffi::c_void>(),
        );
    }

    if (*Srb).SrbFlags as u32 & SRB_FLAGS_UNSPECIFIED_DIRECTION != 0
        && (*DeviceExtension).MapBuffers != 0
        && !(*Irp).MdlAddress.is_null()
    {
        (*Srb).DataBuffer = (((*(*Irp).MdlAddress).StartVa as *mut i8)
            .offset((*(*Irp).MdlAddress).ByteOffset as isize)
            .offset(
                ((*Srb).DataBuffer as *mut i8)
                    .offset_from((*SrbInfo).DataOffset as *mut i8) as isize,
            )).cast();
        (*Srb).SrbFlags as u32 & SRB_FLAGS_DATA_IN != 0;
    }

    if !(*SrbInfo).BaseOfMapRegister.is_null() && !(*SrbInfo).ScatterGather.is_null() {
        let mut transferLen: u32 = 0;
        let mut isWrite: u8 = ((*Srb).SrbFlags as u32 & SRB_FLAGS_DATA_OUT != 0) as u8;
        let mut i: u32 = 0;
        while i < (*SrbInfo).NumberOfMapRegisters && transferLen < (*Srb).DataTransferLength {
            transferLen = transferLen.wrapping_add(
                (*(*SrbInfo).ScatterGather.offset(i as isize)).Length as u32,
            );
            i = i.wrapping_add(1);
        }
        IoFlushAdapterBuffers(
            (*DeviceExtension).AdapterObject,
            (*Irp).MdlAddress,
            (*SrbInfo).BaseOfMapRegister,
            (*Srb).DataBuffer,
            transferLen,
            isWrite,
        );
    }

    (*SrbInfo).Srb = core::ptr::null_mut();

    if (*Srb).SrbFlags as u32 & SRB_FLAGS_DISABLE_DISCONNECT != 0 {
        ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        (*DeviceExtension).Flags |= SCSI_PORT_DISCONNECT_ALLOWED;
        if (*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_RESET == 0 {
            (*DeviceExtension).TimerCount = -1i32;
        }
        ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        if (*DeviceExtension).Flags & SCSI_PORT_REQUEST_PENDING == 0
            && (*DeviceExtension).Flags & SCSI_PORT_DEVICE_BUSY == 0
            && *NeedToCallStartIo == 0
        {
            IoStartNextPacket(
                (*DeviceExtension).Common.DeviceObject,
                0u8,
            );
        }
    }

    if (*Srb).SrbFlags as u32 & SRB_FLAGS_SGLIST_FROM_POOL != 0 {
        ntkrnl_sys::ex::ExFreePoolWithTag(
            (*SrbInfo).ScatterGather as *mut core::ffi::c_void,
            TAG_SCSIPORT,
        );
        (*SrbInfo).ScatterGather = core::ptr::null_mut();
    }

    if (*SrbInfo).NumberOfMapRegisters != 0 {
        IoFreeMapRegisters(
            (*DeviceExtension).AdapterObject,
            (*SrbInfo).BaseOfMapRegister,
            (*SrbInfo).NumberOfMapRegisters,
        );
    }

    ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
    if !(*Srb).SrbExtension.is_null() {
        if !(*Srb).SenseInfoBuffer.is_null()
            && (*DeviceExtension).SupportsAutoSense != 0
        {
            if !((*Srb).SenseInfoBuffer.is_null()
                || !(*SrbInfo).SaveSenseRequest.is_null())
            {
                RtlAssert(
                    b"Srb->SenseInfoBuffer == NULL || SrbInfo->SaveSenseRequest != NULL\0"
                        .as_ptr() as *const i8,
                    b"drivers/storage/port/scsiport/scsi.c\0"
                        .as_ptr() as *const i8,
                    755u32,
                    core::ptr::null::<core::ffi::c_void>(),
                );
            }
            if (*Srb).SrbStatus as i32 & (SRB_STATUS_AUTOSENSE_VALID) as i32 != 0 {
                ntkrnl_sys::rtl::RtlCopyMemory(
                    (*SrbInfo).SaveSenseRequest as *mut core::ffi::c_void,
                    (*Srb).SenseInfoBuffer as *const core::ffi::c_void,
                    (*Srb).SenseInfoBufferLength as usize,
                );
            }
            (*Srb).SenseInfoBuffer = (*SrbInfo).SaveSenseRequest;
        }
        let fresh5 = &mut *((*Srb).SrbExtension as *mut *mut core::ffi::c_void);
        *fresh5 = (*DeviceExtension).FreeSrbExtensions;
        (*DeviceExtension).FreeSrbExtensions = (*Srb).SrbExtension;
    }

    (*Irp).IoStatus.Information = ((*Srb).DataTransferLength as usize) as _;
    (*SrbInfo).SequenceNumber = 0u32;
    (*LunExtension).QueueCount = (*LunExtension).QueueCount.wrapping_sub(1u32);

    if (*Srb).QueueTag as i32 != SP_UNTAGGED as i32 {
        (*SrbInfo).Requests.Blink = core::ptr::null_mut();
        (*SrbInfo).Requests.Flink =
            (*DeviceExtension).FreeSrbInfo as *mut _LIST_ENTRY;
        (*DeviceExtension).FreeSrbInfo = SrbInfo;
    }

    SrbInfo = core::ptr::null_mut();

    if (*DeviceExtension).Flags & SCSI_PORT_REQUEST_PENDING != 0 {
        (*DeviceExtension).Flags &= !SCSI_PORT_REQUEST_PENDING;
        *NeedToCallStartIo = 1u8;
    }
    if (*Srb).SrbStatus as i32
        & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32
        == (SRB_STATUS_SUCCESS) as i32
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        if (*Srb).SrbFlags as u32 & SRB_FLAGS_BYPASS_FROZEN_QUEUE == 0
            && (*LunExtension).RequestTimeout == (-1i64) as i32
        {
            SpiGetNextRequestFromLun(
                DeviceExtension,
                LunExtension,
                core::ptr::null_mut(),
            );
        } else {
            ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        }
        ntkrnl_sys::driver_ddk::IoCompleteRequest(
            Irp,
            IO_DISK_INCREMENT as i8,
        );
        Result = ntkrnl_sys::interlocked::InterlockedDecrement(
            &raw mut (*DeviceExtension).ActiveRequestCounter,
        ) as i32;
        if Result < 0
            && (*DeviceExtension).MapRegisters == 0
            && !(*DeviceExtension).AdapterObject.is_null()
        {
            (*DeviceExtension).MapRegisterBase = core::ptr::null_mut();
            IoFreeAdapterChannel(
                (*DeviceExtension).AdapterObject,
            );
        }
        return;
    }

    Result = ntkrnl_sys::interlocked::InterlockedDecrement(
        &raw mut (*DeviceExtension).ActiveRequestCounter,
    ) as i32;
    if Result < 0
        && (*DeviceExtension).MapRegisters == 0
        && !(*DeviceExtension).AdapterObject.is_null()
    {
        (*DeviceExtension).MapRegisterBase = core::ptr::null_mut();
        IoFreeAdapterChannel(
            (*DeviceExtension).AdapterObject,
        );
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = SpiStatusSrbToNt((*Srb).SrbStatus);

    if ((*Srb).ScsiStatus as i32 == (SCSISTAT_BUSY) as i32
        || (*Srb).SrbStatus as i32 == (SRB_STATUS_BUSY) as i32
        || (*Srb).ScsiStatus as i32 == (SCSISTAT_QUEUE_FULL) as i32)
        && (*Srb).SrbFlags as u32 & SRB_FLAGS_BYPASS_FROZEN_QUEUE == 0
    {
        let mut current_block_113: u64;
        if (*LunExtension).Flags & (LUNEX_FROZEN_QUEUE | LUNEX_BUSY) != 0 {
            (*Srb).SrbStatus = SRB_STATUS_PENDING as u8;
            (*Srb).ScsiStatus = 0u8;
            if KeInsertByKeyDeviceQueue(
                &raw mut (*LunExtension).DeviceQueue,
                &raw mut (*Irp).Tail.Overlay.__bindgen_anon_1.DeviceQueueEntry,
                *(*Srb).__bindgen_anon_1.QueueSortKey.as_ref(),
            ) == 0
            {
                (*Srb).SrbStatus = SRB_STATUS_ERROR as u8;
                (*Srb).ScsiStatus = SCSISTAT_BUSY as u8;
                if 0i32 == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as *const i8,
                        b"drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const i8,
                        881u32,
                        core::ptr::null::<core::ffi::c_void>(),
                    );
                }
                current_block_113 = 12182388840246571894;
            } else {
                ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(
                    (&raw mut (*DeviceExtension).SpinLock).cast(),
                );
                current_block_113 = 178030534879405462;
            }
        } else {
            let fresh6 = (*LunExtension).AttemptCount;
            (*LunExtension).AttemptCount = (*LunExtension).AttemptCount.wrapping_add(1);
            if fresh6 < 20u32 {
                (*Srb).ScsiStatus = 0u8;
                (*Srb).SrbStatus = SRB_STATUS_PENDING as u8;
                (*LunExtension).BusyRequest = Irp;
                (*LunExtension).Flags |= LUNEX_BUSY;
                ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(
                    (&raw mut (*DeviceExtension).SpinLock).cast(),
                );
                current_block_113 = 178030534879405462;
            } else {
                current_block_113 = 12182388840246571894;
            }
        }
        match current_block_113 {
            12182388840246571894 => {
                (*Srb).SrbStatus =
                    ((*Srb).SrbStatus as i32 | (SRB_STATUS_QUEUE_FROZEN) as i32) as u8;
                (*LunExtension).Flags |= LUNEX_FROZEN_QUEUE;
                (*LunExtension).Flags &= !LUNEX_FULL_QUEUE;
                ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(
                    (&raw mut (*DeviceExtension).SpinLock).cast(),
                );
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_NOT_READY;
                ntkrnl_sys::driver_ddk::IoCompleteRequest(
                    Irp,
                    IO_DISK_INCREMENT as i8,
                );
            }
            _ => {}
        }
        return;
    }

    if ((*Srb).ScsiStatus as i32 != (SCSISTAT_CHECK_CONDITION) as i32
        || (*Srb).SrbStatus as i32 & (SRB_STATUS_AUTOSENSE_VALID) as i32 != 0
        || (*Srb).SenseInfoBuffer.is_null()
        || (*Srb).SenseInfoBufferLength == 0)
        && (*Srb).SrbFlags as u32 & SRB_FLAGS_NO_QUEUE_FREEZE != 0
    {
        if (*LunExtension).RequestTimeout == (-1i64) as i32 {
            SpiGetNextRequestFromLun(
                DeviceExtension,
                LunExtension,
                core::ptr::null_mut(),
            );
        } else {
            ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
        }
    } else {
        (*Srb).SrbStatus =
            ((*Srb).SrbStatus as i32 | (SRB_STATUS_QUEUE_FROZEN) as i32) as u8;
        (*LunExtension).Flags |= LUNEX_FROZEN_QUEUE;
        if (*Srb).ScsiStatus as i32 == (SCSISTAT_CHECK_CONDITION) as i32
            && (*Srb).SrbStatus as i32 & (SRB_STATUS_AUTOSENSE_VALID) as i32 == 0
            && !(*Srb).SenseInfoBuffer.is_null()
            && (*Srb).SenseInfoBufferLength != 0
        {
            if (*LunExtension).Flags & LUNEX_BUSY != 0 {
                if KeInsertByKeyDeviceQueue(
                    &raw mut (*LunExtension).DeviceQueue,
                    &raw mut (*(*LunExtension).BusyRequest)
                        .Tail
                        .Overlay
                        .__bindgen_anon_1
                        .DeviceQueueEntry,
                    *(*Srb).__bindgen_anon_1.QueueSortKey.as_ref(),
                ) == 0
                {
                    if 0i32 == 0 {
                        RtlAssert(
                            b"FALSE\0".as_ptr() as *const i8,
                            b"drivers/storage/port/scsiport/scsi.c\0"
                                .as_ptr() as *const i8,
                            954u32,
                            core::ptr::null::<core::ffi::c_void>(),
                        );
                    }
                    ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(
                        (&raw mut (*DeviceExtension).SpinLock).cast(),
                    );
                    ntkrnl_sys::driver_ddk::IoCompleteRequest(
                        Irp,
                        IO_DISK_INCREMENT as i8,
                    );
                    return;
                }
                (*LunExtension).Flags &= !(LUNEX_FULL_QUEUE | LUNEX_BUSY);
            }
            ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
            SpiSendRequestSense(LunExtension, Srb);
            return;
        }
        ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel((&raw mut (*DeviceExtension).SpinLock).cast());
    }

    ntkrnl_sys::driver_ddk::IoCompleteRequest(Irp, IO_DISK_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortStartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    // (Inner extern block deleted — SpiAllocateSrbStructures is defined in this module,
    // ScsiPortNotification is the crate's varargs export. LLM-guessed local consts
    // deleted — several were fabricated; glue.rs/types are the verified source.)
    let mut DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION;
    let mut LunExtension: PSCSI_PORT_LUN_EXTENSION;
    let mut IrpStack: PIO_STACK_LOCATION;
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut SrbInfo: PSCSI_REQUEST_BLOCK_INFO;
    let mut CounterResult: i32; // LONG
    let mut Status: NTSTATUS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
    IrpStack = IoGetCurrentIrpStackLocation(Irp);
    LunExtension = (*(*IrpStack).DeviceObject).DeviceExtension as PSCSI_PORT_LUN_EXTENSION;

    Srb = (*IrpStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;

    (*Srb).SrbFlags |= (*DeviceExtension).SrbFlags as u32;

    if (*DeviceExtension).NeedSrbDataAlloc != 0 || (*DeviceExtension).NeedSrbExtensionAlloc != 0 {
        SrbInfo = SpiAllocateSrbStructures(DeviceExtension, LunExtension, Srb);
        if SrbInfo.is_null() {
            if (*LunExtension).Flags as u32 & LUNEX_REQUEST_PENDING != 0 {
                IoStartNextPacket(DeviceObject, 0); // FALSE = 0
            }
            return;
        }
    } else {
        SrbInfo = core::ptr::addr_of_mut!((*LunExtension).SrbInfo) as PSCSI_REQUEST_BLOCK_INFO;
        (*Srb).SrbExtension = core::ptr::null_mut::<c_void>();
        (*Srb).QueueTag = SP_UNTAGGED;
    }

    if (*SrbInfo).SequenceNumber == 0 {
        (*DeviceExtension).SequenceNumber = (*DeviceExtension).SequenceNumber.wrapping_add(1);
        (*SrbInfo).SequenceNumber = (*DeviceExtension).SequenceNumber;
    }

    if (*Srb).Function as u8 != SRB_FUNCTION_ABORT_COMMAND {
        (*SrbInfo).Srb = Srb;
    }

    if (*Srb).SrbFlags as u32 & SRB_FLAGS_UNSPECIFIED_DIRECTION != 0 {
        (*SrbInfo).DataOffset = MmGetMdlVirtualAddress((*Irp).MdlAddress) as *mut i8;

        if (*DeviceExtension).MapBuffers != 0 {
            (*SrbInfo).DataOffset = MmGetSystemAddressForMdl((*Irp).MdlAddress) as *mut i8;
            let virtual_base = MmGetMdlVirtualAddress((*Irp).MdlAddress) as *mut u8;
            (*Srb).DataBuffer = (*SrbInfo).DataOffset.offset(
                ((*Srb).DataBuffer as *mut u8).offset_from(virtual_base) as isize,
            ) as *mut c_void;
        }
        // Original had if (DeviceExtension->AdapterObject) { KeFlushIoBuffers(...); }
        // Anchor preserved only the condition expression, no call.
        !(*DeviceExtension).AdapterObject.is_null();

        if (*DeviceExtension).MapRegisters != 0 {
            (*SrbInfo).NumberOfMapRegisters = ADDRESS_AND_SIZE_TO_SPAN_PAGES(
                (*Srb).DataBuffer as *mut c_void,
                ((*Srb).DataTransferLength) as usize,
            ) as u32;
            Status = IoAllocateAdapterChannel(
                (*DeviceExtension).AdapterObject as PDMA_ADAPTER,
                (*DeviceExtension).Common.DeviceObject,
                (*SrbInfo).NumberOfMapRegisters,
                Some(core::mem::transmute(crate::scsiport::SpiAdapterControl as *const ())),
                SrbInfo as PVOID,
            );
            if !NT_SUCCESS(Status) {
                (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST;
                ((ScsiPortNotification))(
                    RequestComplete,
                    DeviceExtension.offset(1) as PVOID,
                    Srb as PVOID,
                );
                ScsiPortNotification(
                    NextRequest,
                    DeviceExtension.offset(1) as PVOID,
                );
                IoRequestDpc(
                    (*DeviceExtension).Common.DeviceObject,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            }
            return;
        }
    }

    CounterResult = InterlockedIncrement(
        core::ptr::addr_of_mut!((*DeviceExtension).ActiveRequestCounter) as *mut i32,
    );
    if CounterResult == 0
        && !(*DeviceExtension).AdapterObject.is_null()
        && (*DeviceExtension).MapRegisters == 0
    {
        IoAllocateAdapterChannel(
            (*DeviceExtension).AdapterObject as PDMA_ADAPTER,
            DeviceObject,
            (*DeviceExtension).PortCapabilities.MaximumPhysicalPages,
            Some(core::mem::transmute(crate::scsiport::ScsiPortAllocateAdapterChannel as *const ())),
            LunExtension as PVOID,
        );
        return;
    }

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());

    if KeSynchronizeExecution(
        (*DeviceExtension).Interrupt[0],
        Some(core::mem::transmute(ScsiPortStartPacket as *const ())),
        DeviceObject as PVOID,
    ) == 0
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        (*Irp).IoStatus.Information = 0;
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
    } else {
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
    }
}

