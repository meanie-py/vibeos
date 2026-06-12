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
use ntkrnl_sys::ddk::{ExFreePool, IO_COMPLETION_ROUTINE, IRP, IoAllocateAdapterChannel, IoBuildAsynchronousFsdRequest, IoFlushAdapterBuffers, IoFreeAdapterChannel, IoFreeIrp, IoFreeMapRegisters, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending, IoRequestDpc, IoSetCompletionRoutine, IoStartNextPacket, IoStartPacket, KeCancelTimer, KeInsertByKeyDeviceQueue, KeRemoveByKeyDeviceQueue, KeRemoveDeviceQueue, KeSetTimer, KeSynchronizeExecution, LARGE_INTEGER, MmUnlockPages, PCCHAR, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKDEVICE_QUEUE_ENTRY, PKIRQL};
use core::ffi::c_void;

extern "C" {
    fn SpiGetSrbData( DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION, LunExtension: PSCSI_PORT_LUN_EXTENSION, QueueTag: UCHAR, ) -> PSCSI_REQUEST_BLOCK_INFO;
}
extern "C" {
    fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut KSPIN_LOCK) -> KIRQL;
    fn RtlAssert( FailedAssertion: *const i8, FileName: *const i8, LineNumber: u32, Message: *const i8, );
    fn ScsiPortNotification(NotificationType: i32, HwDeviceExtension: *mut core::ffi::c_void, Srb: *mut _SCSI_REQUEST_BLOCK);
    fn DPRINT(_: *const i8);
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;

pub static mut SpiSenseCompletionRoutine: IO_COMPLETION_ROUTINE = core::ptr::null_mut();

const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;
const SRB_STATUS_TIMEOUT: u8 = 0x09;
const SRB_STATUS_COMMAND_TIMEOUT: u8 = 0x0B;
const SRB_STATUS_BAD_SRB_BLOCK_LENGTH: u8 = 0x15;
const SRB_STATUS_BAD_FUNCTION: u8 = 0x22;
const SRB_STATUS_NO_DEVICE: u8 = 0x08;
const SRB_STATUS_INVALID_LUN: u8 = 0x20;
const SRB_STATUS_INVALID_TARGET_ID: u8 = 0x21;
const SRB_STATUS_NO_HBA: u8 = 0x11;
const SRB_STATUS_DATA_OVERRUN: u8 = 0x12;
const SRB_STATUS_SELECTION_TIMEOUT: u8 = 0x0A;

const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC000009Au32 as i32;
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;

unsafe extern "system" { fn RtlAssert(assertion: *mut core::ffi::c_void, file: *mut core::ffi::c_void, line: u32, message: *mut i8); }

const IRP_MJ_SCSI: u8 = 0x0B;
const SCSIOP_REQUEST_SENSE: u8 = 0x03;
const SRB_FUNCTION_EXECUTE_SCSI: u8 = 0x00;
const SRB_FLAGS_DATA_IN: u32 = 0x40;
const SRB_FLAGS_BYPASS_FROZEN_QUEUE: u32 = 0x02;
const SRB_FLAGS_DISABLE_DISCONNECT: u32 = 0x04;
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x08;
const SRB_FLAGS_BYPASS_LOCKED_QUEUE: u32 = 0x01;
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x10;

const SCSI_PORT_RESET_REQUEST: u32 = 0x04;
const SCSI_PORT_DISABLE_INTERRUPTS: u32 = 0x08;
const SCSISTAT_CHECK_CONDITION: u8 = 0x02;
const SCSISTAT_QUEUE_FULL: u8 = 0x28;
const SRB_STATUS_SUCCESS: u8 = 0x00;
const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
const SRB_STATUS_REQUEST_SENSE_FAILED: u8 = 0x0F;
const SP_UNTAGGED: u8 = 0x00;
const LUNEX_NEED_REQUEST_SENSE: u32 = 0x01;

// Forward declarations for symbols not in the available imports list

const SRB_FUNCTION_RESET_BUS: i32 = 0x0A;
const SRB_FUNCTION_ABORT_COMMAND: i32 = 0x10;
const SRB_STATUS_SUCCESS: i32 = 0x01;
const SRB_STATUS_DATA_OVERRUN: i32 = 0x03;
const SRB_STATUS_AUTOSENSE_VALID: u8 = 0x80;
const SRB_STATUS_QUEUE_FROZEN: i32 = 0x40;
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x20;
const LUNEX_FROZEN_QUEUE: u32 = 0x01;
const LUNEX_NEED_REQUEST_SENSE: u32 = 0x02;
const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016_u32 as i32;
const IO_DISK_INCREMENT: i8 = 0x01;
#[no_mangle]
pub unsafe extern "C" fn SpiGetNextRequestFromLun(
    DeviceExtension: ntkrnl_sys::driver_ddk::PSCSI_PORT_DEVICE_EXTENSION,
    LunExtension: ntkrnl_sys::driver_ddk::PSCSI_PORT_LUN_EXTENSION,
    OldIrql: ntkrnl_sys::ddk::PKIRQL,
) ;

unsafe extern "system" {
    fn Int32x32To64(a: i32, b: i32) -> i64;
    #[no_mangle]
    pub unsafe extern "system" fn SpiSaveInterruptData(Context: PVOID) -> BOOLEAN;
    fn RtlAssert(msg: PVOID, file: PVOID, line: u32, empty: *mut i8);
    #[no_mangle]
    pub unsafe extern "system" fn ScsiPortStartIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) ;
    fn SpiGetNextRequestFromLun(
        DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION,
        LunExtension: PSCSI_PORT_LUN_EXTENSION,
        OldIrql: PKIRQL,
    );
    unsafe fn SpiProcessCompletedRequest(
        DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION,
        SrbInfo: PSCSI_REQUEST_BLOCK_INFO,
        NeedToCallStartIo: PBOOLEAN,
    ) ;
}BUSY: NTSTATUS = 0xC0000463; // placeholder
const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;
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

    Irql = KeAcquireSpinLockRaiseToDpc(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));

    if (*Srb).Function as i32 == SRB_FUNCTION_RELEASE_DEVICE {
        (*LunExtension).DeviceClaimed = FALSE;
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), Irql);
        (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
        return STATUS_SUCCESS;
    }

    if (*LunExtension).DeviceClaimed != 0 {
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), Irql);
        (*Srb).SrbStatus = SRB_STATUS_BUSY;
        return STATUS_DEVICE_BUSY;
    }

    DeviceObject = (*LunExtension).Common.DeviceObject;

    if (*Srb).Function as i32 == SRB_FUNCTION_CLAIM_DEVICE {
        (*LunExtension).DeviceClaimed = TRUE;
    }

    if (*Srb).Function as i32 == SRB_FUNCTION_ATTACH_DEVICE {
        (*LunExtension).Common.DeviceObject = (*Srb).DataBuffer as PDEVICE_OBJECT;
    }

    (*Srb).DataBuffer = DeviceObject as PVOID;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), Irql);
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
        let file: *const i8 = b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
        if DbgPrint(
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
            b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0"
                .as_ptr() as *const i8,
            141u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    portExt = (*(*lunExt).Common.LowerDevice).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;

    if Srb.is_null() {
        let file: *const i8 = b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
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
    let file: *const i8 = b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
    if DbgPrint(
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
        let file: *const i8 = b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
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

    match (*Srb).Function as i32 {
        SRB_FUNCTION_SHUTDOWN | SRB_FUNCTION_FLUSH => {
            if (*portExt).CachesData as i32 == FALSE {
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
            KeAcquireSpinLock(&mut (*portExt).SpinLock, &mut Irql);
            if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
                KeReleaseSpinLock(&mut (*portExt).SpinLock, Irql);
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                Status = STATUS_SUCCESS;
            } else {
                (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
                if (*lunExt).SrbInfo.Srb.is_null() {
                    SpiGetNextRequestFromLun(portExt, lunExt, &mut Irql);
                } else {
                    KeReleaseSpinLock(&mut (*portExt).SpinLock, Irql);
                }
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS;
                Status = STATUS_SUCCESS;
            }
        }
        SRB_FUNCTION_FLUSH_QUEUE => {
            KeAcquireSpinLock(&mut (*portExt).SpinLock, &mut Irql);
            if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
                KeReleaseSpinLock(&mut (*portExt).SpinLock, Irql);
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                // ASSERT(lunExt->SrbInfo.Srb == NULL)
                if !(*lunExt).SrbInfo.Srb.is_null() {
                    RtlAssert(
                        b"lunExt->SrbInfo.Srb == NULL\0".as_ptr() as *const i8,
                        b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0"
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
                    let offset = &raw const (*core::ptr::null::<IRP>())
                        .Tail
                        .Overlay
                        .__bindgen_anon_1
                        .DeviceQueueEntry as usize;
                    NextIrp = (Entry as usize).wrapping_sub(offset) as PIRP;
                    Stack = IoGetCurrentIrpStackLocation(NextIrp);
                    Srb = (*Stack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
                    (*Srb).SrbStatus = SRB_STATUS_REQUEST_FLUSHED;
                    (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
                    (*NextIrp).Tail.Overlay.c2rust_unnamed_0.ListEntry.Flink =
                        IrpList as PLIST_ENTRY as *mut _LIST_ENTRY;
                    IrpList = NextIrp;
                }
                (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
                KeReleaseSpinLock(&mut (*portExt).SpinLock, Irql);
                while !IrpList.is_null() {
                    NextIrp = IrpList;
                    IrpList = (*NextIrp).Tail.Overlay.c2rust_unnamed_0.ListEntry.Flink as PIRP;
                    IoCompleteRequest(NextIrp, 0i8 as CCHAR);
                }
                Status = STATUS_SUCCESS;
            }
        }
        _ => {
            let file: *const i8 = b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8;
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
                (*Srb).QueueSortKey,
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
    let lunExt = (*(*ioStack).DeviceObject).DeviceExtension as ntkrnl_sys::driver_ddk::PSCSI_PORT_LUN_EXTENSION;
    let portExt = (*(*lunExt).Common.LowerDevice).DeviceExtension as ntkrnl_sys::driver_ddk::PSCSI_PORT_DEVICE_EXTENSION;
    let Srb: ntkrnl_sys::driver_ddk::PSCSI_REQUEST_BLOCK = Context as ntkrnl_sys::driver_ddk::PSCSI_REQUEST_BLOCK;
    let InitialSrb: ntkrnl_sys::driver_ddk::PSCSI_REQUEST_BLOCK;
    let InitialIrp: ntkrnl_sys::ddk::PIRP;

    if (*Srb).Function as i32 == SRB_FUNCTION_RESET_BUS
        || (*Srb).Function as i32 == SRB_FUNCTION_ABORT_COMMAND
    {
        ExFreePool(Srb as ntkrnl_sys::ddk::PVOID);
        IoFreeIrp(Irp);
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    InitialSrb = *(Srb.add(1) as *mut ntkrnl_sys::ddk::PVOID) as ntkrnl_sys::driver_ddk::PSCSI_REQUEST_BLOCK;
    InitialIrp = (*InitialSrb).OriginalRequest as ntkrnl_sys::ddk::PIRP;

    if ((*Srb).SrbStatus as i32 & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) == SRB_STATUS_SUCCESS
        || ((*Srb).SrbStatus as i32 & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) == SRB_STATUS_DATA_OVERRUN
    {
        (*InitialSrb).SrbStatus = ((*InitialSrb).SrbStatus as i32 | SRB_STATUS_AUTOSENSE_VALID as i32) as u8;
        (*InitialSrb).SenseInfoBufferLength = (*Srb).DataTransferLength as u8;
    }

    if (*InitialSrb).SrbStatus as i32 & SRB_STATUS_QUEUE_FROZEN == 0 {
        RtlAssert(
            b"InitialSrb->SrbStatus & SRB_STATUS_QUEUE_FROZEN\0".as_ptr() as *mut core::ffi::c_void,
            b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *mut core::ffi::c_void,
            496 as u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    if (*InitialSrb).SrbFlags & SRB_FLAGS_NO_QUEUE_FREEZE != 0
        && (*InitialSrb).SrbStatus as i32 & SRB_STATUS_QUEUE_FROZEN != 0
    {
        let mut irql: ntkrnl_sys::base::KIRQL = 0;
        KeAcquireSpinLock(&raw mut (*portExt).SpinLock, &raw mut irql);
        if (*lunExt).Flags & LUNEX_FROZEN_QUEUE == 0 {
            RtlAssert(
                b"lunExt->Flags & LUNEX_FROZEN_QUEUE\0".as_ptr() as *mut core::ffi::c_void,
                b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *mut core::ffi::c_void,
                506 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
        (*lunExt).Flags &= !LUNEX_FROZEN_QUEUE;
        (*lunExt).Flags &= !LUNEX_NEED_REQUEST_SENSE;
        SpiGetNextRequestFromLun(portExt, lunExt, &raw mut irql);
        (*InitialSrb).SrbStatus = ((*InitialSrb).SrbStatus as i32 & !SRB_STATUS_QUEUE_FROZEN) as u8;
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
        (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32)
            .wrapping_add(core::mem::size_of::<PVOID>() as u32),
        TAG_SCSIPORT,
    ) as PSCSI_REQUEST_BLOCK;

    core::ptr::write_bytes(
        Srb as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
    );

    LargeInt.QuadPart = 1;
    Irp = IoBuildAsynchronousFsdRequest(
        IRP_MJ_READ,
        (*LunExtension).Common.DeviceObject,
        (*InitialSrb).SenseInfoBuffer,
        (*InitialSrb).SenseInfoBufferLength,
        &raw mut LargeInt,
        core::ptr::null_mut(),
    );

    IoSetCompletionRoutine(
        Irp,
        Some(SpiSenseCompletionRoutine as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS),
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

    (*Cdb).CDB6INQUIRY.OperationCode = SCSIOP_REQUEST_SENSE;
    (*Cdb).CDB6INQUIRY.set_LogicalUnitNumber(0u8);
    (*Cdb).CDB6INQUIRY.set_Reserved1(0u8);
    (*Cdb).CDB6INQUIRY.PageCode = 0u8;
    (*Cdb).CDB6INQUIRY.IReserved = 0u8;
    (*Cdb).CDB6INQUIRY.AllocationLength = (*InitialSrb).SenseInfoBufferLength;
    (*Cdb).CDB6INQUIRY.Control = 0u8;

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
    (*Srb).DataTransferLength = (*InitialSrb).SenseInfoBufferLength;

    (*Srb).SrbStatus = 0;
    (*Srb).ScsiStatus = (*Srb).SrbStatus;
    (*Srb).NextSrb = core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>();

    IoCallDriver((*LunExtension).Common.DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortStartPacket(Context: *mut core::ffi::c_void) -> i32 {
    // Forward declarations for missing externals

    // Constants
    const TRUE: i32 = 1;
    const FALSE: i32 = 0;
    const SCSI_PORT_RESET_REQUEST: u32 = 0x08;
    const SCSI_PORT_DEVICE_BUSY: u32 = 0x01;
    const SCSI_PORT_DISCONNECT_ALLOWED: u32 = 0x02;
    const SCSI_PORT_LU_ACTIVE: u32 = 0x01;
    const LUNEX_NEED_REQUEST_SENSE: u32 = 0x01;
    const SRB_FLAGS_BYPASS_FROZEN_QUEUE: u32 = 0x100;
    const SRB_FLAGS_DISABLE_DISCONNECT: u32 = 0x40;
    const SRB_FUNCTION_ABORT_COMMAND: i32 = 0x10;
    const SRB_STATUS_ABORT_FAILED: u8 = 0x0A;
    const SP_UNTAGGED: i32 = 0xFF;
    const RequestComplete: i32 = 0x01;
    const NextRequest: i32 = 0x02;

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
                b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8,
                1008u32,
                core::ptr::null_mut(),
            );
        }
    } else {
        LunExtension = (*DeviceObject).DeviceExtension as *mut _SCSI_PORT_LUN_EXTENSION;
        DeviceExtension = (*(*LunExtension).Common.LowerDevice).DeviceExtension as *mut _SCSI_PORT_DEVICE_EXTENSION;
    }

    if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_RESET) != 0 {
        (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_RESET_REQUEST;
        return TRUE;
    }

    (*DeviceExtension).TimerCount = (*Srb).TimeOutValue as i32;
    (*DeviceExtension).Flags |= SCSI_PORT_DEVICE_BUSY;

    if (*LunExtension).RequestTimeout != -1i32 as i32 {
        StartTimer = FALSE;
    } else {
        (*LunExtension).RequestTimeout = (*Srb).TimeOutValue as i32;
        StartTimer = TRUE;
    }

    if ((*Srb).SrbFlags as u32 & SRB_FLAGS_BYPASS_FROZEN_QUEUE) != 0 {
        if (*Srb).Function as i32 == SRB_FUNCTION_ABORT_COMMAND {
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
                ScsiPortNotification(RequestComplete, (*DeviceExtension).MiniPortDeviceExtension, Srb);
                ScsiPortNotification(NextRequest, (*DeviceExtension).MiniPortDeviceExtension);
                IoRequestDpc(
                    (*DeviceExtension).Common.DeviceObject,
                    core::ptr::null_mut::<IRP>(),
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
                return TRUE;
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
        if (*Srb).QueueTag as i32 != SP_UNTAGGED {
            SrbInfo = &mut *((*DeviceExtension).SrbInfo.offset(((*Srb).QueueTag as i32 - 1i32) as isize))
                as *mut _SCSI_REQUEST_BLOCK_INFO;
            if !(*SrbInfo).Requests.Blink.is_null() {
                RtlAssert(
                    b"SrbInfo->Requests.Blink == NULL\0".as_ptr() as *const i8,
                    b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0".as_ptr() as *const i8,
                    1120u32,
                    core::ptr::null_mut(),
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
        hw_start_io(
            core::ptr::addr_of_mut!((*DeviceExtension).MiniPortDeviceExtension) as *mut core::ffi::c_void,
            Srb,
        )
    } else {
        // Function pointer is NULL; this shouldn't happen
        0
    };

    if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
        IoRequestDpc(
            (*DeviceExtension).Common.DeviceObject,
            core::ptr::null_mut::<IRP>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
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
    KeAcquireSpinLockAtDpcLevel(&raw mut (*DeviceExtension).SpinLock);
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
            Some(SpiSaveInterruptData as unsafe extern "C" fn(PVOID) -> BOOLEAN),
            &raw mut Context as PVOID,
        ) == 0
        {
            // Nothing - just return (don't forget to release the spinlock)
            KeReleaseSpinLockFromDpcLevel(&raw mut (*DeviceExtension).SpinLock);
            return;
        }

        // If flush of adapters is needed - do it
        if (InterruptData.Flags as u32 & SCSI_PORT_FLUSH_ADAPTERS) != 0 {
            // TODO: Implement
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1302,
                core::ptr::null_mut(),
            );
        }

        // Check for IoMapTransfer
        if (InterruptData.Flags as u32 & SCSI_PORT_MAP_TRANSFER) != 0 {
            // TODO: Implement
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1309,
                core::ptr::null_mut(),
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
                KeAcquireSpinLockAtDpcLevel(&raw mut (*DeviceExtension).SpinLock);
            }
        } else {
            // Release the spinlock
            KeReleaseSpinLockFromDpcLevel(&raw mut (*DeviceExtension).SpinLock);
        }

        // If we ready for next packet, start it
        if (InterruptData.Flags as u32 & SCSI_PORT_NEXT_REQUEST_READY) != 0 {
            IoStartNextPacket((*DeviceExtension).Common.DeviceObject, 0i32);
        }

        NeedToStartIo = 0i32;

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
            KeAcquireSpinLockAtDpcLevel(&raw mut (*DeviceExtension).SpinLock);

            // TODO: Put SrbExtension to the list of free extensions
            RtlAssert(
                b"FALSE\0" as *const u8 as PVOID,
                b"scsi.c\0" as *const u8 as PVOID,
                1428,
                core::ptr::null_mut(),
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
                    core::ptr::null_mut(),
                );
            }
            ScsiPortStartIo(DpcDeviceObject, (*DpcDeviceObject).CurrentIrp as PIRP);
        }

        // Everything has been done, check
        if (InterruptData.Flags as u32 & SCSI_PORT_ENABLE_INT_REQUEST) != 0 {
            // Synchronize using spinlock
            KeAcquireSpinLockAtDpcLevel(&raw mut (*DeviceExtension).SpinLock);

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
                    core::ptr::null_mut(),
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
                    core::ptr::null_mut(),
                );
            }

            // If we need a notification again - loop
            if ((*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
                continue 'try_again;
            }

            // Release the spinlock
            KeReleaseSpinLockFromDpcLevel(&raw mut (*DeviceExtension).SpinLock);
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
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
    if (*DeviceExtension).NeedSrbDataAlloc != 0 {
        if (*Srb).Function as core::ffi::c_int == SRB_FUNCTION_ABORT_COMMAND {
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
                        b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                        1505 as ULONG,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                }
                (*LunExtension).PendingRequest = (*Srb).OriginalRequest as PIRP;
                (*LunExtension).Flags |=
                    (LUNEX_REQUEST_PENDING | SCSI_PORT_LU_ACTIVE) as core::ffi::c_ulong;
                KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
                return core::ptr::null_mut();
            }
            if !(*LunExtension).SrbInfo.Srb.is_null() {
                RtlAssert(
                    b"LunExtension->SrbInfo.Srb == NULL\0".as_ptr() as *const core::ffi::c_char
                        as *mut core::ffi::c_void,
                    b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0"
                        .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                    1515 as ULONG,
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            SrbInfo = (*DeviceExtension).FreeSrbInfo;
            if SrbInfo.is_null() {
                (*DeviceExtension).Flags |= SCSI_PORT_REQUEST_PENDING as core::ffi::c_ulong;
                KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
                return core::ptr::null_mut();
            }
            (*DeviceExtension).FreeSrbInfo = (*SrbInfo).Requests.Flink as PSCSI_REQUEST_BLOCK_INFO;
            (*Srb).QueueTag = (SrbInfo.offset_from((*DeviceExtension).SrbInfo)
                as core::ffi::c_longlong as UCHAR as core::ffi::c_int
                + 1 as core::ffi::c_int) as UCHAR;
        } else {
            if (IsListEmpty(core::ptr::addr_of_mut!((*LunExtension).SrbInfo.Requests)) == 0
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
                        b"/home/curi/Projects/os-playground/vibeos-wt-scsiport/drivers/storage/port/scsiport/scsi.c\0"
                            .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
                        1543 as ULONG,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                }
                (*LunExtension).Flags |=
                    (LUNEX_REQUEST_PENDING | SCSI_PORT_LU_ACTIVE) as core::ffi::c_ulong;
                (*LunExtension).PendingRequest = (*Srb).OriginalRequest as PIRP;
                KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
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
            if (*Srb).Function as core::ffi::c_int != SRB_FUNCTION_ABORT_COMMAND
                && (*Srb).QueueTag as core::ffi::c_int != SP_UNTAGGED as core::ffi::c_int
            {
                (*SrbInfo).Requests.Blink = core::ptr::null_mut::<LIST_ENTRY>();
                (*SrbInfo).Requests.Flink =
                    (*DeviceExtension).FreeSrbInfo as PLIST_ENTRY as *mut LIST_ENTRY;
                (*DeviceExtension).FreeSrbInfo = SrbInfo;
            }
            (*DeviceExtension).Flags |= SCSI_PORT_REQUEST_PENDING as core::ffi::c_ulong;
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
            return core::ptr::null_mut();
        }
        (*DeviceExtension).FreeSrbExtensions = *(SrbExtension as *mut PVOID);
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
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
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
    }
    return SrbInfo;
}

