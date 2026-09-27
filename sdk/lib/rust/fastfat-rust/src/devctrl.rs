use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, KEVENT, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
// (FatExceptionFilter reached via crate::seh::fat_try_except)
use crate::types::{CCB, FCB, IRP_CONTEXT, PIRP_CONTEXT, PVOLUME_DEVICE_OBJECT, VCB};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_OBJECT, IO_STACK_LOCATION, IRP};
use crate::ddk_shims::{IO_COMPLETION_ROUTINE, IoIs32bitProcess};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{STATUS_MORE_PROCESSING_REQUIRED};

extern "system" {
    } extern "C" { fn DebugTrace(Level: i32, Flags: u32, Format: *const i8, ...); } extern "system" {
    fn FatDecodeFileObject(FileObject: *mut FILE_OBJECT, Vcb: *mut *mut VCB, Fcb: *mut *mut FCB, Ccb: *mut *mut CCB) -> u32;
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    fn FatAcquireExclusiveVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatReleaseVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatFlushAndCleanVolume(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Vcb: *mut VCB, FlushOption: u32);
    fn FlagOn(Flags: u32, Flag: u32) -> u8;
    fn SetFlag(Flags: *mut u32, Flag: u32);
    fn NT_ASSERT(cond: u32);
    fn KeInitializeEvent(Event: *mut KEVENT, Type: u32, State: u8);
    fn KeWaitForSingleObject(Object: *mut KEVENT, WaitReason: u32, WaitMode: u32, Alertable: u8, Timeout: *mut i64) -> NTSTATUS;
    fn IoSetCompletionRoutine(Irp: *mut IRP, Routine: FatCompletionRoutine, Context: *mut c_void, InvokeOnSuccess: u8, InvokeOnError: u8, InvokeOnCancel: u8); // body returns VOID (wdm) — NTSTATUS ret was decl drift
    fn PAGED_CODE();
    fn FatIsIrpTopLevel(Irp: PIRP) -> BOOLEAN;
    fn FatCreateIrpContext(Irp: PIRP, CanWait: u32) -> PIRP_CONTEXT;
    fn CanFsdWait(Irp: PIRP) -> BOOLEAN;
    fn IoSetTopLevelIrp(Irp: PIRP);
    fn FsRtlEnterFileSystem();
    fn FsRtlExitFileSystem();
    fn UNREFERENCED_PARAMETER(p: PVOLUME_DEVICE_OBJECT);
    fn FatProcessException(IrpContext: PIRP_CONTEXT, Irp: PIRP, ExceptionCode: NTSTATUS) -> NTSTATUS;
}

// ---- local SCSI type stubs (not in ntkrnl_sys::ddk) ----
#[repr(C)]
pub struct CDB {
    pub AsByte: [u8; 16],
}
#[repr(C)]
pub struct SCSI_PASS_THROUGH {
    pub Length: u16,
    pub ScsiStatus: u8,
    pub PathId: u8,
    pub TargetId: u8,
    pub Lun: u8,
    pub CdbLength: u8,
    pub SenseInfoLength: u8,
    pub DataIn: u8,
    pub DataTransferLength: u32,
    pub TimeOutValue: u32,
    pub DataBufferOffset: usize,
    pub SenseInfoOffset: u32,
    pub Cdb: [u8; 16],
}
#[repr(C)]
pub struct SCSI_PASS_THROUGH32 {
    pub Length: u16,
    pub ScsiStatus: u8,
    pub PathId: u8,
    pub TargetId: u8,
    pub Lun: u8,
    pub CdbLength: u8,
    pub SenseInfoLength: u8,
    pub DataIn: u8,
    pub DataTransferLength: u32,
    pub TimeOutValue: u32,
    pub DataBufferOffset: u32,
    pub SenseInfoOffset: u32,
    pub Cdb: [u8; 16],
}
#[repr(C)]
pub struct SCSI_PASS_THROUGH_EX {
    pub Version: u32,
    pub Length: u32,
    pub CdbLength: u32,
    pub StorAddressLength: u32,
    pub ScsiStatus: u8,
    pub SenseInfoLength: u8,
    pub DataDirection: u8,
    pub Reserved: u8,
    pub TimeOutValue: u32,
    pub StorAddressOffset: u32,
    pub SenseInfoOffset: u32,
    pub DataOutTransferLength: u32,
    pub DataInTransferLength: u32,
    pub DataOutBufferOffset: usize,
    pub DataInBufferOffset: usize,
    pub Cdb: [u8; 1],
}
#[repr(C)]
pub struct SCSI_PASS_THROUGH32_EX {
    pub Version: u32,
    pub Length: u32,
    pub CdbLength: u32,
    pub StorAddressLength: u32,
    pub ScsiStatus: u8,
    pub SenseInfoLength: u8,
    pub DataDirection: u8,
    pub Reserved: u8,
    pub TimeOutValue: u32,
    pub StorAddressOffset: u32,
    pub SenseInfoOffset: u32,
    pub DataOutTransferLength: u32,
    pub DataInTransferLength: u32,
    pub DataOutBufferOffset: u32,
    pub DataInBufferOffset: u32,
    pub Cdb: [u8; 1],
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_DEVCTRL;
// devctrl.c:32 writes `IO_COMPLETION_ROUTINE FatDeviceControlCompletionRoutine;` — a prefast
// annotation that declares the function THROUGH ITS FUNCTION TYPE, with no `*`. That parsed as a
// variable declaration, so this arrived as a null Option and the real body at devctrl.c:390 was
// never translated at all: IoSetCompletionRoutine was handed None, nothing ever signalled
// wait_event, and the KeWaitForSingleObject below (null timeout) waited forever.
//
// Spelled `extern "system"` — identical ABI to the stdcall the i386 kernel calls and to the C
// ABI on 64-bit — so one definition serves all three targets. The IoSetCompletionRoutine decl
// above takes `FatCompletionRoutine` for the same reason; ntkrnl-sys's PIO_COMPLETION_ROUTINE is
// spelled with the arch's literal convention, and Rust compares fn-pointer ABIs nominally.
pub type FatCompletionRoutine = Option<
    unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS,
>;

#[no_mangle]
pub unsafe extern "system" fn FatDeviceControlCompletionRoutine(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Contxt: *mut c_void,
) -> NTSTATUS {
    let Event = Contxt as PKEVENT;

    //
    //  If there is an event, this is a synch request. Signal and
    //  let I/O know this isn't done yet.
    //

    if !Event.is_null() {
        KeSetEvent(Event as *mut c_void, 0, 0u8); // ntkrnl_sys::ke takes the event opaquely
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    let _ = DeviceObject;
    let _ = Irp;

    STATUS_SUCCESS
}

const IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES: u32 = 1;
const IOCTL_DISK_COPY_DATA: u32 = 0x0007C064;
const IOCTL_SCSI_PASS_THROUGH: u32 = 0x0004D004;
const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32 = 0x0004D014;
const IOCTL_SCSI_PASS_THROUGH_EX: u32 = 0x0004D044;
const IOCTL_SCSI_PASS_THROUGH_DIRECT_EX: u32 = 0x0004D048;
const UserVolumeOpen: u32 = crate::types::_TYPE_OF_OPEN_UserVolumeOpen; // 4 — bindgen ground truth (was 0: every DASD ioctl mis-classified)
const FlushWithoutPurge: u32 = 3; // FAT_FLUSH_TYPE (fatprocs.h) — was 0 = NoFlush, silently skipping the flush
const STATUS_PENDING: i32 = 0x103;
const STATUS_INVALID_PARAMETER: i32 = 0xC000000D_u32 as i32;
const STATUS_ACCESS_DENIED: i32 = 0xC0000022_u32 as i32;
const DEBUG_TRACE_DEVCTRL: u32 = 0x00010000;
const FO_FILE_MODIFIED: u32 = 0x1000;
const CCB_FLAG_SENT_FORMAT_UNIT: u32 = 0x00040000;
const CCB_FLAG_COMPLETE_DISMOUNT: u32 = 0x00010000;
const SL_FORCE_DIRECT_WRITE: u32 = 0x10;
const VCB_STATE_FLAG_LOCKED: u32 = 0x00000001;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const SCSIOP_FORMAT_UNIT: u8 = 0x04;
const NotificationEvent: u32 = 0;
const Executive: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn FatFsdDeviceControl(
    VolumeDeviceObject: PVOLUME_DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    const Dbg: u32 = 0; // placeholder for DEBUG_TRACE_DEVCTRL

    PAGED_CODE();
    DebugTrace(1, Dbg, b"FatFsdDeviceControl\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, (CanFsdWait(Irp)) as _);
        *p_status = FatCommonDeviceControl(*p_irp_context, Irp);
    }) {
        Status = FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();
    DebugTrace(
        -1,
        Dbg,
        b"FatFsdDeviceControl -> %08lx\n\0".as_ptr() as *const i8,
        Status,
    );
    UNREFERENCED_PARAMETER(VolumeDeviceObject);

    Status
}


#[no_mangle]
pub unsafe extern "system" fn FatCommonDeviceControl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Irp = Irp;
    unsafe {
        // PAGED_CODE();

        let mut status: NTSTATUS = 0;
        let mut irp_sp: *mut IO_STACK_LOCATION = core::ptr::null_mut();
        let mut wait_event: KEVENT = core::mem::zeroed();
        let mut completion_context: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut vcb: *mut VCB = core::ptr::null_mut();
        let mut fcb: *mut FCB = core::ptr::null_mut();
        let mut ccb: *mut CCB = core::ptr::null_mut();

        irp_sp = ntkrnl_sys::ddk::IoGetCurrentIrpStackLocation(Irp);

        DebugTrace(1, DEBUG_TRACE_DEVCTRL, b"FatCommonDeviceControl\n\0".as_ptr() as *const i8, 0);
        DebugTrace(0, DEBUG_TRACE_DEVCTRL, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as isize);
        DebugTrace(0, DEBUG_TRACE_DEVCTRL, b"MinorFunction = %08lx\n\0".as_ptr() as *const i8, (*irp_sp).MinorFunction as u32);

        if FatDecodeFileObject((*irp_sp).FileObject, core::ptr::addr_of_mut!(vcb), core::ptr::addr_of_mut!(fcb), core::ptr::addr_of_mut!(ccb)) != UserVolumeOpen {
            FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
            DebugTrace(-1, DEBUG_TRACE_DEVCTRL, b"FatCommonDeviceControl -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER as u32);
            return STATUS_INVALID_PARAMETER;
        }

        match ((*irp_sp).Parameters).DeviceIoControl.IoControlCode {
            IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES => {
                SetFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_WAIT);
                FatAcquireExclusiveVolume(IrpContext, vcb);
                FatFlushAndCleanVolume(IrpContext, Irp, vcb, FlushWithoutPurge);
                KeInitializeEvent(core::ptr::addr_of_mut!(wait_event), _EVENT_TYPE_NotificationEvent, 0u8);
                completion_context = core::ptr::addr_of_mut!(wait_event) as *mut core::ffi::c_void;
                IoCopyCurrentIrpStackLocationToNext(Irp);
                IoSetCompletionRoutine(Irp, Some(FatDeviceControlCompletionRoutine), completion_context, 1, 1, 1);
            }
            IOCTL_DISK_COPY_DATA => {
                if FlagOn((*vcb).VcbState, VCB_STATE_FLAG_LOCKED) == 0 &&
                   FlagOn(((*irp_sp).Flags as u8) as u32, SL_FORCE_DIRECT_WRITE) == 0 &&
                   FlagOn((*ccb).Flags, CCB_FLAG_COMPLETE_DISMOUNT) == 0 {
                    FatCompleteRequest(IrpContext, Irp, STATUS_ACCESS_DENIED);
                    DebugTrace(-1, DEBUG_TRACE_DEVCTRL, b"FatCommonDeviceControl -> %08lx\n\0".as_ptr() as *const i8, STATUS_ACCESS_DENIED as u32);
                    return STATUS_ACCESS_DENIED;
                }
            }
            IOCTL_SCSI_PASS_THROUGH | IOCTL_SCSI_PASS_THROUGH_DIRECT | IOCTL_SCSI_PASS_THROUGH_EX | IOCTL_SCSI_PASS_THROUGH_DIRECT_EX => {
                let mut cdb: *mut CDB = core::ptr::null_mut();
                if (FlagOn((*(*irp_sp).FileObject).Flags, FO_FILE_MODIFIED) == 0 || FlagOn((*ccb).Flags, CCB_FLAG_SENT_FORMAT_UNIT) == 0) && !((*Irp).AssociatedIrp.SystemBuffer).is_null() {
                    let io_control_code = ((*irp_sp).Parameters).DeviceIoControl.IoControlCode;
                    let input_buffer_length = ((*irp_sp).Parameters).DeviceIoControl.InputBufferLength;
                    if cfg!(all(target_pointer_width = "64", feature = "BUILD_WOW64_ENABLED")) && IoIs32bitProcess(Irp) != 0 {
                        if io_control_code == IOCTL_SCSI_PASS_THROUGH || io_control_code == IOCTL_SCSI_PASS_THROUGH_DIRECT {
                            if input_buffer_length as usize >= core::mem::size_of::<SCSI_PASS_THROUGH32>() {
                                let spt = (*Irp).AssociatedIrp.SystemBuffer as *mut SCSI_PASS_THROUGH32;
                                cdb = core::ptr::addr_of_mut!((*spt).Cdb) as *mut CDB;
                            }
                        } else {
                            if input_buffer_length as usize >= core::mem::size_of::<SCSI_PASS_THROUGH32_EX>() {
                                let spt = (*Irp).AssociatedIrp.SystemBuffer as *mut SCSI_PASS_THROUGH32_EX;
                                cdb = core::ptr::addr_of_mut!((*spt).Cdb) as *mut CDB;
                            }
                        }
                    } else {
                        if io_control_code == IOCTL_SCSI_PASS_THROUGH || io_control_code == IOCTL_SCSI_PASS_THROUGH_DIRECT {
                            if input_buffer_length as usize >= core::mem::size_of::<SCSI_PASS_THROUGH>() {
                                let spt = (*Irp).AssociatedIrp.SystemBuffer as *mut SCSI_PASS_THROUGH;
                                cdb = core::ptr::addr_of_mut!((*spt).Cdb) as *mut CDB;
                            }
                        } else {
                            if input_buffer_length as usize >= core::mem::size_of::<SCSI_PASS_THROUGH_EX>() {
                                let spt = (*Irp).AssociatedIrp.SystemBuffer as *mut SCSI_PASS_THROUGH_EX;
                                cdb = core::ptr::addr_of_mut!((*spt).Cdb) as *mut CDB;
                            }
                        }
                    }

                    if !cdb.is_null() && (*cdb).AsByte[0] == SCSIOP_FORMAT_UNIT {
                        SetFlag(core::ptr::addr_of_mut!((*ccb).Flags), CCB_FLAG_SENT_FORMAT_UNIT);
                        SetFlag(core::ptr::addr_of_mut!((*(*irp_sp).FileObject).Flags), FO_FILE_MODIFIED);
                    }
                }
                // Fall through to default behavior
                ntkrnl_sys::ddk::IoSkipCurrentIrpStackLocation(Irp);
            }
            _ => {
                ntkrnl_sys::ddk::IoSkipCurrentIrpStackLocation(Irp);
            }
        }

        status = ntkrnl_sys::driver_ddk::IoCallDriver((*vcb).TargetDeviceObject, Irp);

        if status == STATUS_PENDING && !completion_context.is_null() {
            KeWaitForSingleObject(core::ptr::addr_of_mut!(wait_event), _KWAIT_REASON_Executive, (0i32) as u32, 0u8, core::ptr::null_mut());
            status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }

        if !completion_context.is_null() {
            NT_ASSERT(((((*irp_sp).Parameters).DeviceIoControl.IoControlCode == IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES) as i32) as u32);
            FatReleaseVolume(IrpContext, vcb);
        } else {
            Irp = core::ptr::null_mut();
        }

        FatCompleteRequest(IrpContext, Irp, status);
        DebugTrace(-1, DEBUG_TRACE_DEVCTRL, b"FatCommonDeviceControl -> %08lx\n\0".as_ptr() as *const i8, status as u32);

        status
    }
}




// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     DevCtrl.c
//
// Abstract:
//
//     This module implements the File System Device Control routines for Fat
//     called by the dispatch driver.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_DEVCTRL)
//
// //
// //  Local procedure prototypes
// //
//
// //
// //  Tell prefast this is an IO_COMPLETION_ROUTINE
// //
// IO_COMPLETION_ROUTINE FatDeviceControlCompletionRoutine;
//
// NTSTATUS
// NTAPI
// FatDeviceControlCompletionRoutine(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Contxt
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonDeviceControl)
// #pragma alloc_text(PAGE, FatFsdDeviceControl)
// #endif
//
//
// _Function_class_(IRP_MJ_DEVICE_CONTROL)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdDeviceControl (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of Device control operations
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file exists
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The FSD status for the IRP
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdDeviceControl\n", 0);
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ));
//
//         Status = FatCommonDeviceControl( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  exception code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdDeviceControl -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonDeviceControl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for doing Device control operations called
//     by both the fsd and fsp threads
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
//     InFsp - Indicates if this is the fsp thread or someother thread
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//     KEVENT WaitEvent;
//     PVOID CompletionContext = NULL;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Get a pointer to the current Irp stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonDeviceControl\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp);
//     DebugTrace( 0, Dbg, "MinorFunction = %08lx\n", IrpSp->MinorFunction);
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatCommonDeviceControl -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  A few IOCTLs actually require some intervention on our part
//     //
//
//     switch (IrpSp->Parameters.DeviceIoControl.IoControlCode) {
//
//     case IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES:
//
//         //
//         //  This is sent by the Volume Snapshot driver (Lovelace).
//         //  We flush the volume, and hold all file resources
//         //  to make sure that nothing more gets dirty. Then we wait
//         //  for the IRP to complete or cancel.
//         //
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//         FatAcquireExclusiveVolume( IrpContext, Vcb );
//
//         FatFlushAndCleanVolume( IrpContext,
//                                 Irp,
//                                 Vcb,
//                                 FlushWithoutPurge );
//
//         KeInitializeEvent( &WaitEvent, NotificationEvent, FALSE );
//         CompletionContext = &WaitEvent;
//
//         //
//         //  Get the next stack location, and copy over the stack location
//         //
//
//         IoCopyCurrentIrpStackLocationToNext( Irp );
//
//         //
//         //  Set up the completion routine
//         //
//
//         IoSetCompletionRoutine( Irp,
//                                 FatDeviceControlCompletionRoutine,
//                                 CompletionContext,
//                                 TRUE,
//                                 TRUE,
//                                 TRUE );
//         break;
//
//     case IOCTL_DISK_COPY_DATA:
//
//         //
//         //  We cannot allow this IOCTL to be sent unless the volume is locked,
//         //  since this IOCTL allows direct writing of data to the volume.
//         //  We do allow kernel callers to force access via a flag.  A handle that
//         //  issued a dismount can send this IOCTL as well.
//         //
//
//         if (!FlagOn( Vcb->VcbState, VCB_STATE_FLAG_LOCKED ) &&
//             !FlagOn( IrpSp->Flags, SL_FORCE_DIRECT_WRITE ) &&
//             !FlagOn( Ccb->Flags, CCB_FLAG_COMPLETE_DISMOUNT )) {
//
//             FatCompleteRequest( IrpContext,
//                                 Irp,
//                                 STATUS_ACCESS_DENIED );
//
//             DebugTrace(-1, Dbg, "FatCommonDeviceControl -> %08lx\n", STATUS_ACCESS_DENIED);
//             return STATUS_ACCESS_DENIED;
//         }
//
//         break;
//
//     case IOCTL_SCSI_PASS_THROUGH:
//     case IOCTL_SCSI_PASS_THROUGH_DIRECT:
//     case IOCTL_SCSI_PASS_THROUGH_EX:
//     case IOCTL_SCSI_PASS_THROUGH_DIRECT_EX:
//
//         //
//         //  If someone is issuing a format unit command underneath us, then make
//         //  sure we mark the device as needing verification when they close their
//         //  handle.
//         //
//
//         if ((!FlagOn( IrpSp->FileObject->Flags, FO_FILE_MODIFIED ) ||
//              !FlagOn( Ccb->Flags, CCB_FLAG_SENT_FORMAT_UNIT )) &&
//              (Irp->AssociatedIrp.SystemBuffer != NULL)) {
//
//             PCDB  Cdb = NULL;
//
//             //
//             //  If this is a 32 bit application running on 64 bit then thunk the
//             //  input structures to grab the Cdb.
//             //
//
// #if defined (_WIN64) && defined(BUILD_WOW64_ENABLED)
//             if (IoIs32bitProcess(Irp)) {
//
//                 if ( (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_SCSI_PASS_THROUGH) ||
//                      (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT )) {
//
//                     if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= sizeof( SCSI_PASS_THROUGH32 )) {
//
//                         Cdb = (PCDB)((PSCSI_PASS_THROUGH32)(Irp->AssociatedIrp.SystemBuffer))->Cdb;
//                     }
//                 } else {
//
//                     if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= sizeof( SCSI_PASS_THROUGH32_EX )) {
//
//                         Cdb = (PCDB)((PSCSI_PASS_THROUGH32_EX)(Irp->AssociatedIrp.SystemBuffer))->Cdb;
//                     }
//                 }
//
//             } else {
// #endif
//                 if ( (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_SCSI_PASS_THROUGH) ||
//                      (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT )) {
//
//                     if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= sizeof( SCSI_PASS_THROUGH )) {
//
//                         Cdb = (PCDB)((PSCSI_PASS_THROUGH)(Irp->AssociatedIrp.SystemBuffer))->Cdb;
//                     }
//                 } else {
//
//                     if (IrpSp->Parameters.DeviceIoControl.InputBufferLength >= sizeof( SCSI_PASS_THROUGH_EX )) {
//
//                         Cdb = (PCDB)((PSCSI_PASS_THROUGH_EX)(Irp->AssociatedIrp.SystemBuffer))->Cdb;
//                     }
//                 }
//
// #if defined (_WIN64) && defined(BUILD_WOW64_ENABLED)
//             }
// #endif
//
//             if ((Cdb != NULL) && (Cdb->AsByte[0] == SCSIOP_FORMAT_UNIT)) {
//
//                 SetFlag( Ccb->Flags, CCB_FLAG_SENT_FORMAT_UNIT );
//                 SetFlag( IrpSp->FileObject->Flags, FO_FILE_MODIFIED );
//             }
//         }
//
//         //
//         //  Fall through as we do not need to know the outcome of this operation.
//         //
//
//     default:
//
//         //
//         //  FAT doesn't need to see this on the way back, so skip ourselves.
//         //
//
//         IoSkipCurrentIrpStackLocation( Irp );
//         break;
//     }
//
//     //
//     //  Send the request.
//     //
//
//     Status = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//     if (Status == STATUS_PENDING && CompletionContext) {
//
//         KeWaitForSingleObject( &WaitEvent,
//                                Executive,
//                                KernelMode,
//                                FALSE,
//                                NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     //
//     //  If we had a context, the IRP remains for us and we will complete it.
//     //  Handle it appropriately.
//     //
//
//     if (CompletionContext) {
//
//         //
//         //  Release all the resources that we held because of a
//         //  VOLSNAP_FLUSH_AND_HOLD.
//         //
//
//         NT_ASSERT( IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES );
//
//         FatReleaseVolume( IrpContext, Vcb );
//
//         //
//         //  If we had no context, the IRP will complete asynchronously.
//         //
//
//     } else {
//
//         Irp = NULL;
//     }
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatCommonDeviceControl -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI
// FatDeviceControlCompletionRoutine(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// {
//     PKEVENT Event = (PKEVENT) Contxt;
//
//     //
//     //  If there is an event, this is a synch request. Signal and
//     //  let I/O know this isn't done yet.
//     //
//
//     if (Event) {
//
//         KeSetEvent( Event, 0, FALSE );
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Irp );
//
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
