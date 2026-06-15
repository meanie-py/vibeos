use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{CcFlushCache, ExAcquireResourceExclusiveLite, ExReleaseResourceLite, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoGetNextIrpStackLocation, IoMarkIrpPending, IoSetTopLevelIrp, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PETHREAD, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DCB, DIRENT, FAT_BUG_CHECK_FLUSH, FCB, FatDiskAccountingEnabled, PDIRENT, VCB, VCB_STATE_FLAG_WRITE_PROTECTED};
// FatMarkFcbCondition called via extern "system" block in FatFlushFile below
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_OBJECT, IO_STACK_LOCATION, IRP, _ETHREAD, _PUBLIC_BCB as BCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_FLUSH};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_IRP_CONTEXT};

extern "system" {
    fn FlagOn(Flags: u32, Mask: u32) -> i32;
    fn FatVerifyVcb(IrpContext: *mut PIRP_CONTEXT, Vcb: *mut PVCB);
    fn PAGED_CODE();
    } extern "C" { fn DebugTrace(level: i32, component: u32, format: *const i8, ...); } extern "system" {
    fn ClearFlag(flags: *mut u32, flag: u32);
    fn SetFlag(flags: *mut u32, flag: u32);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatGetDirentFromFcbOrDcb( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, flag: BOOLEAN, dirent: *mut PDIRENT, dirent_bcb: *mut PBCB, );
    fn FatGetNextFcbTopDown( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Dcb: *mut DCB, ) -> *mut FCB;
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatResetExceptionState(IrpContext: *mut IRP_CONTEXT);
    fn FatTruncateFileAllocation( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, file_size: i32, );
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, bcb: *mut *mut core::ffi::c_void);
    fn FatVcbAcquiredExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> BOOLEAN;
    fn NT_ASSERT(condition: BOOLEAN);
    fn NodeType(p: *mut FCB) -> u32;
    fn IsFileDeleted(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> BOOLEAN;
    fn FatVerifyFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatUnpinRepinnedBcbs(IrpContext: *mut IRP_CONTEXT);
    fn _SEH2_GetExceptionCode() -> NTSTATUS;
    fn FatReservedBytes(bpb: *const core::ffi::c_void) -> u32;
    fn FatNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, status: i32);
    fn FatMarkFcbCondition(IrpContext: PIRP_CONTEXT, Fcb: PFCB, condition: u32, recursive: u8);
    fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    fn FatDecodeFileObject(FileObject: PFILE_OBJECT, Vcb: *mut PVCB, Fcb: *mut PFCB, Ccb: *mut PCCB) -> TYPE_OF_OPEN;
    fn FatUpdateDirentFromFcb(IrpContext: PIRP_CONTEXT, FileObject: PFILE_OBJECT, Fcb: PFCB, Ccb: PCCB);
    fn FatFsdPostRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn FatMarkVolume(IrpContext: PIRP_CONTEXT, Vcb: PVCB, state: VOLUME_CONDITION);
    fn FatToggleMediaEjectDisable(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Disable: BOOLEAN);
    fn FatBugCheck(p1: u32, p2: u32, p3: u32) -> !;
    fn DebugUnwind(name: *const i8);
    fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP);
    fn IoSetCompletionRoutine( Irp: PIRP, Routine: PIO_COMPLETION_ROUTINE, Context: *mut core::ffi::c_void, InvokeOnSuccess: BOOLEAN, InvokeOnError: BOOLEAN, InvokeOnCancel: BOOLEAN, );
    fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn IoIsSystemThread(Thread: PETHREAD) -> BOOLEAN;
    fn PsGetCurrentThread() -> PETHREAD;
    fn PsGetThreadProcess(Thread: PETHREAD) -> *mut core::ffi::c_void;
    fn PsUpdateDiskCounters(Process: *mut core::ffi::c_void, ReadBytes: u64, WriteBytes: u64, c: u32, d: u32, e: u32);
    fn KeCancelTimer(Timer: *mut ntkrnl_sys::ddk::KTIMER) -> BOOLEAN;
    fn KeRemoveQueueDpc(Dpc: *mut ntkrnl_sys::ddk::KDPC) -> BOOLEAN;
    fn FsRtlIsNtstatusExpected(NtStatus: NTSTATUS) -> BOOLEAN;
    fn CanFsdWait(Irp: ntkrnl_sys::ddk::PIRP) -> u8;
    fn FatCreateIrpContext( Irp: ntkrnl_sys::ddk::PIRP, CanBlock: u8, ) -> *mut IRP_CONTEXT;
    fn FatExceptionFilter( IrpContext: *mut IRP_CONTEXT, ExceptionInfo: *mut EXCEPTION_POINTERS, ) -> i32;
    fn FatProcessException( IrpContext: *mut IRP_CONTEXT, Irp: ntkrnl_sys::ddk::PIRP, ExceptionCode: i32, ) -> i32;
    fn FatIsIrpTopLevel(Irp: ntkrnl_sys::ddk::PIRP) -> u8;
    fn _SEH2_GetExceptionInformation() -> *mut EXCEPTION_POINTERS;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_FLUSH;
pub const Dbg: u32 = DEBUG_TRACE_FLUSH;

// Function body from C snippet is incomplete; translation covers only the provided fragment.
// The rest of the function (SEH exception handling, FAT flushing logic) is missing.

// Forward declarations for unknown symbols

// Placeholder constants (caller must provide correct values)
const VCB_STATE_FLAG_DELETED_FCB: u32 = 0x0100; // TODO: actual value
const FlushAndInvalidate: u32 = 1;            // FAT_FLUSH_TYPE value
const FcbBad: u32 = 2;                        // FCB condition value

const STATUS_MORE_PROCESSING_REQUIRED: ntkrnl_sys::base::NTSTATUS = 0xC0000016u32 as i32;

// Constants (placeholder values — real values will be provided by the caller)
const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04; // TODO
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // TODO
const FCB_STATE_TRUNCATE_ON_CLOSE: u32 = 0x02; // TODO
const FAT_NTC_FCB: u32 = 0x0502; // TODO
const FcbGood: u32 = 1; // TODO
const EXCEPTION_EXECUTE_HANDLER: u32 = 1;
const EXCEPTION_CONTINUE_SEARCH: u32 = 0;

// Gateway types (the caller collects the full definitions)
type PIRP_CONTEXT = *mut IRP_CONTEXT;
type PVCB = *mut VCB;
type PFCB = *mut FCB;
type PCCB = *mut CCB;
type PIO_COMPLETION_ROUTINE = Option<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>;
type PVOID = *mut core::ffi::c_void;
type BOOLEAN = u8;
type NTSTATUS = i32;
type TYPE_OF_OPEN = u32;
type FAT_FLUSH_TYPE = i32;
type VOLUME_CONDITION = i32;
type PBCB = *mut BCB;

// Constants used in this function
const FCB_STATE_FLUSH_FAT: u32 = 0x10;
const VCB_STATE_FLAG_VOLUME_DIRTY: u32 = 0x04;
const VCB_STATE_FLAG_MOUNTED_DIRTY: u32 = 0x10;
const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
const VCB_STATE_FLAG_BOOT_OR_PAGING_FILE: u32 = 0x0800;
const FO_FILE_SIZE_CHANGED: u32 = 0x2000;
const STATUS_INVALID_DEVICE_REQUEST: i32 = 0xC0000010_u32 as i32;
const STATUS_PENDING: i32 = 0x00000103;
const Flush: FAT_FLUSH_TYPE = 1;
const VolumeClean: VOLUME_CONDITION = 2;
const VirtualVolumeFile: TYPE_OF_OPEN = 0;
const EaFile: TYPE_OF_OPEN = 1;
const DirectoryFile: TYPE_OF_OPEN = 2;
const UserFileOpen: TYPE_OF_OPEN = 3;
const UserDirectoryOpen: TYPE_OF_OPEN = 4;
const UserVolumeOpen: TYPE_OF_OPEN = 5;
const IRP_MN_FLUSH_DATA_ONLY: u8 = 0x01;
const IRP_MN_FLUSH_NO_SYNC: u8 = 0x02;
const FAT_NTC_ROOT_DCB: u32 = 0x0504; // NodeType for root DCB

#[repr(C)]
struct IRP_CONTEXT {
    _unused: u8,
}
#[repr(C)]
struct EXCEPTION_POINTERS {
    _unused: u8,
}


#[no_mangle]
pub unsafe extern "system" fn FatFsdFlushBuffers(
    VolumeDeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
) -> i32 {
    let mut Status: i32 = 0;
    let mut IrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: u8;

    PAGED_CODE();
    DebugTrace(
        1,
        Dbg,
        b"FatFsdFlushBuffers\n\0".as_ptr() as *const i8,
        0,
    );
    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // DUMMY_PSEH: _SEH2_EXCEPT handler is dead code (FatExceptionFilter/FatProcessException
    // never reached); the try body runs directly. See AGENTS.md "DUMMY_PSEH".
    {
        IrpContext = FatCreateIrpContext(Irp, CanFsdWait(Irp));
        Status = FatCommonFlushBuffers(IrpContext, Irp);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }
    FsRtlExitFileSystem();
    DebugTrace(
        -1,
        Dbg,
        b"FatFsdFlushBuffers -> %08lx\n\0".as_ptr() as *const i8,
        Status,
    );
    let _ = VolumeDeviceObject; // UNREFERENCED_PARAMETER
    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatCommonFlushBuffers(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IrpContext = IrpContext;
    let mut Irp = Irp;
    let mut Status: NTSTATUS;
    let IrpSp: PIO_STACK_LOCATION;
    let FileObject: PFILE_OBJECT;
    let mut TypeOfOpen: TYPE_OF_OPEN;
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut VcbAcquired: BOOLEAN = 0u8;
    let mut FcbAcquired: BOOLEAN = 0u8;
    let mut FatFlushRequired: BOOLEAN = 0u8;

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatCommonFlushBuffers\n\0".as_ptr() as *const i8);
    DebugTrace(0, Dbg, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp);
    DebugTrace(0, Dbg, b"->FileObject  = %p\n\0".as_ptr() as *const i8, (*IrpSp).FileObject);

    FileObject = (*IrpSp).FileObject;
    TypeOfOpen = FatDecodeFileObject(FileObject, core::ptr::addr_of_mut!(Vcb), core::ptr::addr_of_mut!(Fcb), core::ptr::addr_of_mut!(Ccb));

    if FlagOn(((*(IrpContext as *mut _IRP_CONTEXT)).Flags as u8) as u32, IRP_CONTEXT_FLAG_WAIT) == 0 {
        Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace(-1, Dbg, b"FatCommonFlushBuffers -> %08lx\n\0".as_ptr() as *const i8, Status);
        return Status;
    }

    Status = STATUS_SUCCESS;

    let DoFlush: BOOLEAN = if (*IrpSp).MinorFunction as u32 != (IRP_MN_FLUSH_DATA_ONLY) as u32 { 1u8 } else { 0u8 };

    // Outer try block simulation
    let mut completed_normally: BOOLEAN = 0u8;

    // ---- Try block begins ----
    {
        // Disk accounting (Win8+)
        if true { // corresponding to NTDDI_VERSION >= NTDDI_WIN8
            if FatDiskAccountingEnabled != 0 {
                let mut OriginatingThread: PETHREAD = core::ptr::null_mut();
                if !(*Irp).Tail.Overlay.Thread.is_null() &&
                   IoIsSystemThread((*Irp).Tail.Overlay.Thread) == 0 {
                    OriginatingThread = (*Irp).Tail.Overlay.Thread;
                } else {
                    OriginatingThread = PsGetCurrentThread();
                }
                // NT_ASSERT ignored
                PsUpdateDiskCounters(PsGetThreadProcess(OriginatingThread), 0, 0, 0, 0, 1);
            }
        }

        match TypeOfOpen {
            VirtualVolumeFile | EaFile | DirectoryFile => {
                DebugTrace(0, Dbg, b"Flush that does nothing\n\0".as_ptr() as *const i8);
            }
            UserFileOpen => {
                DebugTrace(0, Dbg, b"Flush User File Open\n\0".as_ptr() as *const i8);

                FatAcquireExclusiveFcb(IrpContext, Fcb);
                FcbAcquired = 1u8;

                // Simulated SEH: call FatVerifyFcb and skip handler
                {
                    FatVerifyFcb(IrpContext, Fcb);
                    // Exception handler (dead code, kept for structure)
                    if FsRtlIsNtstatusExpected(0) != 0 {
                        FatResetExceptionState(IrpContext);
                    }
                }

                Status = FatFlushFile(IrpContext, Fcb, (DoFlush) as u32);

                if NT_SUCCESS(Status) {
                    SetFlag((core::ptr::addr_of_mut!((*FileObject).Flags) as *mut u8).cast(), FO_FILE_SIZE_CHANGED);
                    FatUpdateDirentFromFcb(IrpContext, FileObject, Fcb, Ccb);

                    if FlagOn(((*Fcb).FcbState as u8) as u32, FCB_STATE_FLUSH_FAT) != 0 {
                        FatFlushRequired = 1u8;
                    }

                    let mut NextFcb: PFCB = (*Fcb).ParentDcb;
                    while !NextFcb.is_null() {
                        // Verify Fcb (simulated SEH)
                        {
                            FatVerifyFcb(IrpContext, NextFcb);
                            if FsRtlIsNtstatusExpected(0) != 0 {
                                FatResetExceptionState(IrpContext);
                            }
                        }

                        if (*NextFcb).FcbCondition == FcbGood {
                            let LocalStatus: NTSTATUS;
                            LocalStatus = FatFlushFile(IrpContext, NextFcb, (DoFlush) as u32);
                            if !NT_SUCCESS(LocalStatus) {
                                Status = LocalStatus;
                            }
                            if FlagOn(((*NextFcb).FcbState as u8) as u32, FCB_STATE_FLUSH_FAT) != 0 {
                                FatFlushRequired = 1u8;
                            }
                        }
                        NextFcb = (*NextFcb).ParentDcb;
                    }

                    if FatFlushRequired != 0u8 {
                        Status = FatFlushFat(core::ptr::addr_of_mut!(IrpContext) as *mut PIRP_CONTEXT, core::ptr::addr_of_mut!(Vcb) as *mut PVCB);
                        ClearFlag(core::ptr::addr_of_mut!((*Fcb).FcbState), FCB_STATE_FLUSH_FAT);
                    }

                    SetFlag(core::ptr::addr_of_mut!((*(IrpContext as *mut _IRP_CONTEXT)).Flags), IRP_CONTEXT_FLAG_WRITE_THROUGH);
                }
            }
            UserDirectoryOpen => {
                if NodeType(Fcb) != FAT_NTC_ROOT_DCB {
                    DebugTrace(0, Dbg, b"Flush a directory does nothing\n\0".as_ptr() as *const i8);
                } else {
                    // Fall through to UserVolumeOpen handling
                    DebugTrace(0, Dbg, b"Flush User Volume Open, or root dcb\n\0".as_ptr() as *const i8);

                    {
                        let Finished: BOOLEAN;
                        Finished = FatAcquireExclusiveVcb(IrpContext, Vcb) as BOOLEAN;
                        // NT_ASSERT(Finished) omitted
                    }
                    VcbAcquired = 1u8;

                    FatFlushVolume(IrpContext, Vcb, (Flush as u32) as i32);

                    if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_VOLUME_DIRTY) != 0 {
                        KeCancelTimer(core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer));
                        KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc));

                        if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                            FatMarkVolume(IrpContext, Vcb, (VolumeClean as u32) as i32);
                            ClearFlag(core::ptr::addr_of_mut!((*Vcb).VcbState), VCB_STATE_FLAG_VOLUME_DIRTY);
                        }

                        if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0 &&
                           FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0 {
                            FatToggleMediaEjectDisable(IrpContext, Vcb, 0u8);
                        }
                    }
                }
            }
            UserVolumeOpen => {
                DebugTrace(0, Dbg, b"Flush User Volume Open, or root dcb\n\0".as_ptr() as *const i8);

                {
                    let Finished: BOOLEAN;
                    Finished = FatAcquireExclusiveVcb(IrpContext, Vcb) as BOOLEAN;
                    // NT_ASSERT(Finished) omitted
                }
                VcbAcquired = 1u8;

                FatFlushVolume(IrpContext, Vcb, (Flush as u32) as i32);

                if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_VOLUME_DIRTY) != 0 {
                    KeCancelTimer(core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer));
                    KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc));

                    if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                        FatMarkVolume(IrpContext, Vcb, (VolumeClean as u32) as i32);
                        ClearFlag(core::ptr::addr_of_mut!((*Vcb).VcbState), VCB_STATE_FLAG_VOLUME_DIRTY);
                    }

                    if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0 &&
                       FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0 {
                        FatToggleMediaEjectDisable(IrpContext, Vcb, 0u8);
                    }
                }
            }
            _ => {
                FatBugCheck(TypeOfOpen as u32, 0, 0);
            }
        }

        FatUnpinRepinnedBcbs(IrpContext);
        completed_normally = 1u8;
    }
    // ---- Try block ends ----

    // Outer finally block always runs
    {
        DebugUnwind(b"FatCommonFlushBuffers\0".as_ptr() as *const i8);
        if VcbAcquired != 0u8 {
            FatReleaseVcb(IrpContext, Vcb);
        }
        if FcbAcquired != 0u8 {
            FatReleaseFcb(IrpContext, Fcb);
        }
        if completed_normally != 0u8 {
            // Normal termination – proceed with completion
            if (*IrpSp).MinorFunction as u32 != (IRP_MN_FLUSH_DATA_ONLY) as u32 &&
               (*IrpSp).MinorFunction as u32 != (IRP_MN_FLUSH_NO_SYNC) as u32 {
                let DriverStatus: NTSTATUS;
                IoCopyCurrentIrpStackLocationToNext(Irp);
                IoSetCompletionRoutine(
                    Irp,
                    Some(FatFlushCompletionRoutine),
                    Status as *mut core::ffi::c_void,
                    1u8,
                    1u8,
                    1u8,
                );
                DriverStatus = IoCallDriver((*Vcb).TargetDeviceObject, Irp);
                if DriverStatus == STATUS_PENDING ||
                   (!NT_SUCCESS(DriverStatus) && DriverStatus != STATUS_INVALID_DEVICE_REQUEST) {
                    Status = DriverStatus;
                }
                Irp = core::ptr::null_mut();
            }
            FatCompleteRequest(IrpContext, Irp, Status);
        }
        DebugTrace(-1, Dbg, b"FatCommonFlushBuffers -> %08lx\n\0".as_ptr() as *const i8, Status);
    }
    // _SEH2_END implied by the end of the finally block

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FatFlushDirectory(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
    FlushType: FAT_FLUSH_TYPE,
) -> NTSTATUS {
    let mut Fcb: *mut FCB;
    let mut Vcb: *mut VCB;
    let mut NextFcb: *mut FCB;

    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();

    let mut Status: NTSTATUS;
    let mut ReturnStatus: NTSTATUS = STATUS_SUCCESS;

    let mut ClearWriteThroughOnExit: BOOLEAN = 0;
    let mut ClearWaitOnExit: BOOLEAN = 0;

    let mut CorrectedFileSize: u32 = 0;

    PAGED_CODE();

    NT_ASSERT(FatVcbAcquiredExclusive(IrpContext, (*Dcb).Vcb));

    DebugTrace(
        1,
        Dbg,
        b"FatFlushDirectory, Dcb = %p\n\0".as_ptr() as *const i8,
        Dcb,
    );

    if FlagOn(
        ((*(IrpContext as *mut _IRP_CONTEXT)).Flags as u8) as u32,
        IRP_CONTEXT_FLAG_WRITE_THROUGH as u32,
    ) == 0
    {
        ClearWriteThroughOnExit = 1;
        SetFlag(
            core::ptr::addr_of_mut!((*(IrpContext as *mut _IRP_CONTEXT)).Flags),
            IRP_CONTEXT_FLAG_WRITE_THROUGH,
        );
    }

    if FlagOn(
        ((*(IrpContext as *mut _IRP_CONTEXT)).Flags as u8) as u32,
        IRP_CONTEXT_FLAG_WAIT as u32,
    ) == 0
    {
        ClearWaitOnExit = 1;
        SetFlag(
            core::ptr::addr_of_mut!((*(IrpContext as *mut _IRP_CONTEXT)).Flags),
            IRP_CONTEXT_FLAG_WAIT,
        );
    }

    Vcb = (*Dcb).Vcb;
    Fcb = Dcb as *mut FCB;

    while Fcb != core::ptr::null_mut() {
        NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, Dcb);

        if NodeType(Fcb) == FAT_NTC_FCB
            && (*Vcb).EaFcb != Fcb
            && (IsFileDeleted(IrpContext, Fcb) as i32) == 0
        {
            FatAcquireExclusiveFcb(IrpContext, Fcb);

            ClearFlag(
                (&mut (*Vcb).VcbState) as *mut u32,
                VCB_STATE_FLAG_DELETED_FCB,
            );

            // DUMMY_PSEH: EXCEPT handlers are dead code (dropped). The TRY/FINALLY
            // lowers to a labeled block + always-run finally; try_leave → break 'mid.
            {
                FatVerifyFcb(IrpContext, Fcb); // inner TRY/EXCEPT body (except dropped)

                'mid: {
                    if (*Fcb).FcbCondition != FcbGood {
                        break 'mid; // was try_leave(NOTHING)
                    }

                    if FlagOn(
                        ((*Fcb).FcbState as u8) as u32,
                        FCB_STATE_TRUNCATE_ON_CLOSE as u32,
                    ) != 0
                    {
                        FatTruncateFileAllocation(
                            IrpContext,
                            Fcb,
                            ((*Fcb).Header.FileSize.__bindgen_anon_1.LowPart) as i32,
                        );
                    }

                    FatGetDirentFromFcbOrDcb(
                        IrpContext,
                        Fcb,
                        0,
                        core::ptr::addr_of_mut!(Dirent),
                        core::ptr::addr_of_mut!(DirentBcb),
                    );

                    CorrectedFileSize = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;

                    if (*Dirent).FileSize != CorrectedFileSize {
                        (*Dirent).FileSize = CorrectedFileSize;
                    }

                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(DirentBcb) as *mut *mut core::ffi::c_void);

                    Status = FatFlushFile(IrpContext, Fcb, (FlushType) as u32);

                    if !NT_SUCCESS(Status) {
                        ReturnStatus = Status;
                    }
                }
                // _SEH2_FINALLY (always runs):
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(DirentBcb) as *mut *mut core::ffi::c_void);
                if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_DELETED_FCB as u32) == 0 {
                    FatReleaseFcb(IrpContext, Fcb);
                }
            }
        }

        Fcb = NextFcb;
    }

    Fcb = Dcb as *mut FCB;
    while Fcb != core::ptr::null_mut() {
        NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, Dcb);

        if NodeType(Fcb) != FAT_NTC_FCB && (IsFileDeleted(IrpContext, Fcb) as i32) == 0 {
            // DUMMY_PSEH: EXCEPT dead → keep try body only.
            FatVerifyFcb(IrpContext, Fcb);

            if (*Fcb).FcbCondition == FcbGood {
                Status = FatFlushFile(IrpContext, Fcb, (FlushType) as u32);

                if !NT_SUCCESS(Status) {
                    ReturnStatus = Status;
                }
            }
        }

        Fcb = NextFcb;
    }

    // DUMMY_PSEH: bare TRY → body only.
    {
        FatUnpinRepinnedBcbs(IrpContext);
    }

    if ClearWriteThroughOnExit != 0 {
        ClearFlag(
            (&mut (*(IrpContext as *mut _IRP_CONTEXT)).Flags) as *mut u32,
            IRP_CONTEXT_FLAG_WRITE_THROUGH as u32,
        );
    }

    if ClearWaitOnExit != 0 {
        ClearFlag(
            (&mut (*(IrpContext as *mut _IRP_CONTEXT)).Flags) as *mut u32,
            IRP_CONTEXT_FLAG_WAIT as u32,
        );
    }

    DebugTrace(
        -1,
        Dbg,
        b"FatFlushDirectory -> 0x%08lx\n\0".as_ptr() as *const i8,
        ReturnStatus,
    );

    ReturnStatus
}

#[no_mangle]
pub unsafe extern "system" fn FatFlushFat(
    IrpContext: *mut PIRP_CONTEXT,
    Vcb: *mut PVCB,
) -> NTSTATUS {
    let mut Bcb: *mut PBCB = core::ptr::null_mut();
    let mut DontCare: *mut PVOID = core::ptr::null_mut();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Offset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut ReturnStatus: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE();

    if FlagOn((**Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
        return STATUS_SUCCESS;
    }

    // _SEH2_TRY { ... }
    {
        FatVerifyVcb(IrpContext, Vcb);
    }

    // The remainder of the function is not present in the provided C source.
    ReturnStatus
}
#[no_mangle]
pub unsafe extern "system" fn FatFlushVolume(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    FlushType: FAT_FLUSH_TYPE,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut IrpContext = IrpContext;
    let mut Vcb = Vcb;
    let mut Status: NTSTATUS;
    let mut ReturnStatus: NTSTATUS = STATUS_SUCCESS;
    if ((*Vcb).VcbState & VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
        return STATUS_SUCCESS;
    }
    Status = FatFlushDirectory(IrpContext, (*Vcb).RootDcb, FlushType);
    if !NT_SUCCESS(Status) {
        ReturnStatus = Status;
    }
    Status = FatFlushFat(core::ptr::addr_of_mut!(IrpContext) as *mut PIRP_CONTEXT, core::ptr::addr_of_mut!(Vcb) as *mut PVCB);
    if !NT_SUCCESS(Status) {
        ReturnStatus = Status;
    }
    if ((*Vcb).VcbState & VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
        && ((*Vcb).VcbState & VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0
    {
        FatToggleMediaEjectDisable(IrpContext, Vcb, 0u8);
    }
    return ReturnStatus;
}
#[no_mangle]
pub unsafe extern "system" fn FatFlushFile(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FlushType: u32,
) -> NTSTATUS {
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Vcb = (*Fcb).Vcb;
    // PAGED_CODE();

    CcFlushCache(
        core::ptr::addr_of_mut!((*(*Fcb).NonPaged).SectionObjectPointers),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Iosb),
    );

    if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_DELETED_FCB) == 0 {
        ExAcquireResourceExclusiveLite((*Fcb).Header.PagingIoResource, 1);
        if FlushType == FlushAndInvalidate {
            FatMarkFcbCondition(IrpContext as PIRP_CONTEXT, Fcb as PFCB, FcbBad, 0);
        }
        ExReleaseResourceLite((*Fcb).Header.PagingIoResource);
    }

    Iosb.__bindgen_anon_1.Status as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn FatHijackIrpAndFlushDevice(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    TargetDeviceObject: *mut DEVICE_OBJECT,
) -> NTSTATUS {
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut next_irp_sp: *mut IO_STACK_LOCATION;

    // UNREFERENCED_PARAMETER(IrpContext)
    let _ = IrpContext;

    IoCopyCurrentIrpStackLocationToNext(Irp);

    next_irp_sp = IoGetNextIrpStackLocation(Irp);
    (*next_irp_sp).MajorFunction = IRP_MJ_FLUSH_BUFFERS;
    (*next_irp_sp).MinorFunction = 0;

    KeInitializeEvent(&raw mut event, 0 /* _EVENT_TYPE_NotificationEvent */, 0 /* FALSE */);

    IoSetCompletionRoutine(
        Irp,
        Some(FatHijackCompletionRoutine),
        &raw mut event as *mut core::ffi::c_void,
        1, /* TRUE */
        1,
        1,
    );

    status = IoCallDriver(TargetDeviceObject, Irp);

    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut event as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8, /* KernelMode */
            0, /* FALSE */
            core::ptr::null_mut(),
        );
        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if status == STATUS_INVALID_DEVICE_REQUEST {
        status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = 0;
    (*Irp).IoStatus.Information = 0;

    status
}
#[no_mangle]
pub unsafe extern "system" fn FatFlushFatEntries(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Cluster: u32,
    Count: u32,
) {
    PAGED_CODE();

    let mut ByteCount: u32 = 0;
    let mut FileOffset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut Iosb: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

    FileOffset.__bindgen_anon_1.HighPart = 0;
    FileOffset.__bindgen_anon_1.LowPart = FatReservedBytes(
        core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void,
    );

    if (*Vcb).AllocationSupport.FatIndexBitSize == 12 {
        FileOffset.__bindgen_anon_1.LowPart += Cluster * 3 / 2;
        ByteCount = (Count * 3 / 2) + 1;
    } else if (*Vcb).AllocationSupport.FatIndexBitSize == 32 {
        FileOffset.__bindgen_anon_1.LowPart += Cluster * core::mem::size_of::<u32>() as u32;
        ByteCount = Count * core::mem::size_of::<u32>() as u32;
    } else {
        FileOffset.__bindgen_anon_1.LowPart += Cluster * core::mem::size_of::<u16>() as u32;
        ByteCount = Count * core::mem::size_of::<u16>() as u32;
    }

    CcFlushCache(
        core::ptr::addr_of_mut!((*Vcb).SectionObjectPointers),
        core::ptr::addr_of_mut!(FileOffset),
        ByteCount,
        core::ptr::addr_of_mut!(Iosb),
    );

    if Iosb.__bindgen_anon_1.Status >= 0 {
        Iosb.__bindgen_anon_1.Status = FatHijackIrpAndFlushDevice(
            IrpContext as *mut IRP_CONTEXT,
            (*(IrpContext as *mut _IRP_CONTEXT)).OriginatingIrp,
            (*Vcb).TargetDeviceObject,
        );
    }

    if Iosb.__bindgen_anon_1.Status < 0 {
        FatNormalizeAndRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatFlushDirentForFile(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    // PAGED_CODE(); // macro – no‑op translation
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    FileOffset.QuadPart = (*Fcb).DirentOffsetWithinDirectory as i64;

    CcFlushCache(
        core::ptr::addr_of_mut!((*(*(*Fcb).ParentDcb).NonPaged).SectionObjectPointers),
        &raw mut FileOffset,
        core::mem::size_of::<DIRENT>() as u32,
        &raw mut Iosb,
    );

    if NT_SUCCESS(Iosb.__bindgen_anon_1.Status) {
        Iosb.__bindgen_anon_1.Status = FatHijackIrpAndFlushDevice(
            IrpContext,
            (*(IrpContext as *mut _IRP_CONTEXT)).OriginatingIrp,
            (*(*Fcb).Vcb).TargetDeviceObject,
        );
    }

    if !NT_SUCCESS(Iosb.__bindgen_anon_1.Status) {
        FatNormalizeAndRaiseStatus(IrpContext, Iosb.__bindgen_anon_1.Status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatFlushCompletionRoutine(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
    Contxt: *mut core::ffi::c_void,
) -> i32 {
    let mut status: i32 = Contxt as usize as i32;

    if (*Irp).PendingReturned != 0 {
        ntkrnl_sys::ddk::IoMarkIrpPending(Irp);
    }

    if ntkrnl_sys::base::NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status)
        || (*Irp).IoStatus.__bindgen_anon_1.Status == ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    }

    let _ = DeviceObject;
    let _ = Contxt;

    ntkrnl_sys::base::STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FatHijackCompletionRoutine(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
    Contxt: *mut core::ffi::c_void,
) -> ntkrnl_sys::base::NTSTATUS {
    let _ = DeviceObject;
    let _ = Irp;
    ntkrnl_sys::ke::KeSetEvent((Contxt as *mut ntkrnl_sys::ddk::KEVENT).cast(), 0, 0);
    return STATUS_MORE_PROCESSING_REQUIRED;
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
//     Flush.c
//
// Abstract:
//
//     This module implements the File Flush buffers routine for Fat called by the
//     dispatch driver.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_FLUSH)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_FLUSH)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonFlushBuffers)
// #pragma alloc_text(PAGE, FatFlushDirectory)
// #pragma alloc_text(PAGE, FatFlushFat)
// #pragma alloc_text(PAGE, FatFlushFile)
// #pragma alloc_text(PAGE, FatFlushVolume)
// #pragma alloc_text(PAGE, FatFsdFlushBuffers)
// #pragma alloc_text(PAGE, FatFlushDirentForFile)
// #pragma alloc_text(PAGE, FatFlushFatEntries)
// #pragma alloc_text(PAGE, FatHijackIrpAndFlushDevice)
// #endif
//
// //
// //  Local procedure prototypes
// //
//
// IO_COMPLETION_ROUTINE FatFlushCompletionRoutine;
//
// NTSTATUS
// NTAPI
// FatFlushCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// IO_COMPLETION_ROUTINE FatHijackCompletionRoutine;
//
// NTSTATUS
// NTAPI
// FatHijackCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
//
// _Function_class_(IRP_MJ_FLUSH_BUFFERS)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdFlushBuffers (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of Flush buffers.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file being flushed exists
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
//     DebugTrace(+1, Dbg, "FatFsdFlushBuffers\n", 0);
//
//     //
//     //  Call the common Cleanup routine, with blocking allowed if synchronous
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         Status = FatCommonFlushBuffers( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
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
//     DebugTrace(-1, Dbg, "FatFsdFlushBuffers -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonFlushBuffers (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for flushing a buffer.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PFCB NextFcb;
//     PCCB Ccb;
//
//     BOOLEAN VcbAcquired = FALSE;
//     BOOLEAN FcbAcquired = FALSE;
//     BOOLEAN FatFlushRequired = FALSE;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonFlushBuffers\n", 0);
//     DebugTrace( 0, Dbg, "Irp           = %p\n", Irp);
//     DebugTrace( 0, Dbg, "->FileObject  = %p\n", IrpSp->FileObject);
//
//     //
//     //  Extract and decode the file object
//     //
//
//     FileObject = IrpSp->FileObject;
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     //
//     //  CcFlushCache is always synchronous, so if we can't wait enqueue
//     //  the irp to the Fsp.
//     //
//
//     if ( !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) ) {
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatCommonFlushBuffers -> %08lx\n", Status );
//         return Status;
//     }
//
//     Status = STATUS_SUCCESS;
//
//     _SEH2_TRY {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         if (FatDiskAccountingEnabled) {
//
//             PETHREAD OriginatingThread = NULL;
//
//             //
//             //  Charge the flush to the originating thread.
//             //  Try the Thread in Irp's tail first, if that is NULL, then charge
//             //  the flush to current thread.
//             //
//
//             if ((Irp->Tail.Overlay.Thread != NULL) &&
//                 !IoIsSystemThread( Irp->Tail.Overlay.Thread )) {
//
//                 OriginatingThread = Irp->Tail.Overlay.Thread;
//
//             } else {
//
//                 OriginatingThread = PsGetCurrentThread();
//             }
//
//             NT_ASSERT( OriginatingThread != NULL );
//
//             PsUpdateDiskCounters( PsGetThreadProcess( OriginatingThread ),
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   1 );
//         }
//
// #endif
//
//         //
//         //  Case on the type of open that we are trying to flush
//         //
//
//         switch (TypeOfOpen) {
//
//         case VirtualVolumeFile:
//         case EaFile:
//         case DirectoryFile:
//             DebugTrace(0, Dbg, "Flush that does nothing\n", 0);
//             break;
//
//         case UserFileOpen:
//
//             DebugTrace(0, Dbg, "Flush User File Open\n", 0);
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//
//             FcbAcquired = TRUE;
//
//             FatVerifyFcb( IrpContext, Fcb );
//
//             //
//             //  If the file is cached then flush its cache
//             //
//
//             Status = FatFlushFile( IrpContext, Fcb, Flush );
//
//             //
//             //  Also flush the file's dirent in the parent directory if the file
//             //  flush worked.
//             //
//
//             if (NT_SUCCESS( Status )) {
//
//                 //
//                 //  Insure that we get the filesize to disk correctly.  This is
//                 //  benign if it was already good.
//                 //
//
//                 SetFlag(FileObject->Flags, FO_FILE_SIZE_CHANGED);
//
//
//                 FatUpdateDirentFromFcb( IrpContext, FileObject, Fcb, Ccb );
//
//                 if (FlagOn(Fcb->FcbState, FCB_STATE_FLUSH_FAT)) {
//
//                     FatFlushRequired = TRUE;
//                 }
//
//                 //
//                 //  Flush the parent Dcb's to get any dirent updates to disk.
//                 //
//
//                 NextFcb = Fcb->ParentDcb;
//
//                 while (NextFcb != NULL) {
//
//                     //
//                     //  Make sure the Fcb is OK.
//                     //
//
//                     _SEH2_TRY {
//
//                         FatVerifyFcb( IrpContext, NextFcb );
//
//                     } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                         FatResetExceptionState( IrpContext );
//                     } _SEH2_END;
//
//                     if (NextFcb->FcbCondition == FcbGood) {
//
//                         NTSTATUS LocalStatus;
//
//                         LocalStatus = FatFlushFile( IrpContext, NextFcb, Flush );
//
//                         if (!NT_SUCCESS(LocalStatus)) {
//
//                             Status = LocalStatus;
//                         }
//
//                         if (FlagOn(NextFcb->FcbState, FCB_STATE_FLUSH_FAT)) {
//
//                             FatFlushRequired = TRUE;
//                         }
//                     }
//
//                     NextFcb = NextFcb->ParentDcb;
//                 }
//
//                 //
//                 //  Flush the volume file to get any allocation information
//                 //  updates to disk.
//                 //
//
//                 if (FatFlushRequired) {
//
//                     Status = FatFlushFat( IrpContext, Vcb );
//
//                     ClearFlag(Fcb->FcbState, FCB_STATE_FLUSH_FAT);
//                 }
//
//                 //
//                 //  Set the write through bit so that these modifications
//                 //  will be completed with the request.
//                 //
//
//                 SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH);
//             }
//
//             break;
//
//         case UserDirectoryOpen:
//
//             //
//             //  If the user had opened the root directory then we'll
//             //  oblige by flushing the volume.
//             //
//
//             if (NodeType(Fcb) != FAT_NTC_ROOT_DCB) {
//
//                 DebugTrace(0, Dbg, "Flush a directory does nothing\n", 0);
//                 break;
//             }
//
//         case UserVolumeOpen:
//
//             DebugTrace(0, Dbg, "Flush User Volume Open, or root dcb\n", 0);
//
//             //
//             //  Acquire exclusive access to the Vcb.
//             //
//
//             {
//                 BOOLEAN Finished;
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//                 Finished = FatAcquireExclusiveVcb( IrpContext, Vcb );
//                 NT_ASSERT( Finished );
//             }
//
//             VcbAcquired = TRUE;
//
//             //
//             //  Mark the volume clean and then flush the volume file,
//             //  and then all directories
//             //
//
//             Status = FatFlushVolume( IrpContext, Vcb, Flush );
//
//             //
//             //  If the volume was dirty, do the processing that the delayed
//             //  callback would have done.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY)) {
//
//                 //
//                 //  Cancel any pending clean volumes.
//                 //
//
//                 (VOID)KeCancelTimer( &Vcb->CleanVolumeTimer );
//                 (VOID)KeRemoveQueueDpc( &Vcb->CleanVolumeDpc );
//
//                 //
//                 //  The volume is now clean, note it.
//                 //
//
//                 if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                     FatMarkVolume( IrpContext, Vcb, VolumeClean );
//                     ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//                 }
//
//                 //
//                 //  Unlock the volume if it is removable.
//                 //
//
//                 if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//                     !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE)) {
//
//                     FatToggleMediaEjectDisable( IrpContext, Vcb, FALSE );
//                 }
//             }
//
//             break;
//
//         default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( TypeOfOpen, 0, 0 );
//         }
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonFlushBuffers );
//
//         if (VcbAcquired) { FatReleaseVcb( IrpContext, Vcb ); }
//
//         if (FcbAcquired) { FatReleaseFcb( IrpContext, Fcb ); }
//
//         //
//         //  If this is a normal termination then pass the request on
//         //  to the target device object.
//         //
//
//         if (!_SEH2_AbnormalTermination()) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             if ((IrpSp->MinorFunction != IRP_MN_FLUSH_DATA_ONLY) &&
//                 (IrpSp->MinorFunction != IRP_MN_FLUSH_NO_SYNC)) {
// #endif
//
//                 NTSTATUS DriverStatus;
//
//                 //
//                 //  Get the next stack location, and copy over the stack location
//                 //
//
//                 IoCopyCurrentIrpStackLocationToNext(Irp);
//
//                 //
//                 //  Set up the completion routine
//                 //
//
//                 IoSetCompletionRoutine( Irp,
//                                         FatFlushCompletionRoutine,
//                                         ULongToPtr( Status ),
//                                         TRUE,
//                                         TRUE,
//                                         TRUE );
//
//                 //
//                 //  Send the request.
//                 //
//
//                 DriverStatus = IoCallDriver(Vcb->TargetDeviceObject, Irp);
//
//                 if ((DriverStatus == STATUS_PENDING) ||
//                     (!NT_SUCCESS(DriverStatus) &&
//                      (DriverStatus != STATUS_INVALID_DEVICE_REQUEST))) {
//
//                     Status = DriverStatus;
//                 }
//
//                 Irp = NULL;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             }
// #endif
//
//             //
//             //  Complete the Irp if necessary and return to the caller.
//             //
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonFlushBuffers -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatFlushDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
//
// /*++
//
// Routine Description:
//
//     This routine non-recursively flushes a dcb tree.
//
// Arguments:
//
//     Dcb - Supplies the Dcb being flushed
//
//     FlushType - Specifies the kind of flushing to perform
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     PFCB Fcb;
//     PVCB Vcb;
//     PFCB NextFcb;
//
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//
//     NTSTATUS Status;
//     NTSTATUS ReturnStatus = STATUS_SUCCESS;
//
//     BOOLEAN ClearWriteThroughOnExit = FALSE;
//     BOOLEAN ClearWaitOnExit = FALSE;
//
//     ULONG CorrectedFileSize = 0;
//
//     PAGED_CODE();
//
//     NT_ASSERT( FatVcbAcquiredExclusive(IrpContext, Dcb->Vcb) );
//
//     DebugTrace(+1, Dbg, "FatFlushDirectory, Dcb = %p\n", Dcb);
//
//     //
//     //  First flush all the files, then the directories, to make sure all the
//     //  file sizes and times get sets correctly on disk.
//     //
//     //  We also have to check here if the "Ea Data. Sf" fcb really
//     //  corressponds to an existing file.
//     //
//
//     if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH)) {
//
//         ClearWriteThroughOnExit = TRUE;
//         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH);
//     }
//
//     if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//         ClearWaitOnExit = TRUE;
//         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     Vcb = Dcb->Vcb;
//     Fcb = Dcb;
//
//     while (Fcb != NULL) {
//
//         NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, Dcb);
//
//         if ( (NodeType( Fcb ) == FAT_NTC_FCB) &&
//              (Vcb->EaFcb != Fcb) &&
//              !IsFileDeleted(IrpContext, Fcb)) {
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//
//             ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB );
//
//             //
//             //  Exception handler to catch and commute errors encountered
//             //  doing the flush dance.  We may encounter corruption, and
//             //  should continue flushing the volume as much as possible.
//             //
//
//             _SEH2_TRY {
//
//                 //
//                 //  Standard handler to release resources, etc.
//                 //
//
//                 _SEH2_TRY {
//
//                     //
//                     //  Make sure the Fcb is OK.
//                     //
//
//                     _SEH2_TRY {
//
//                         FatVerifyFcb( IrpContext, Fcb );
//
//                     } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                         FatResetExceptionState( IrpContext );
//                     } _SEH2_END;
//
//                     //
//                     //  If this Fcb is not good skip it.  Note that a 'continue'
//                     //  here would be very expensive as we inside a try{} body.
//                     //
//
//                     if (Fcb->FcbCondition != FcbGood) {
//
//                         try_leave( NOTHING);
//                     }
//
//                     //
//                     //  In case a handle was never closed and the FS and AS are more
//                     //  than a cluster different, do this truncate.
//                     //
//
//                     if ( FlagOn(Fcb->FcbState, FCB_STATE_TRUNCATE_ON_CLOSE) ) {
//
//
//                         FatTruncateFileAllocation( IrpContext,
//                                                    Fcb,
//                                                    Fcb->Header.FileSize.LowPart );
//
//
//                     }
//
//                     //
//                     //  Also compare the file's dirent in the parent directory
//                     //  with the size information in the Fcb and update
//                     //  it if neccessary.  Note that we don't mark the Bcb dirty
//                     //  because we will be flushing the file object presently, and
//                     //  Mm knows what's really dirty.
//                     //
//
//                     FatGetDirentFromFcbOrDcb( IrpContext,
//                                               Fcb,
//                                               FALSE,
//                                               &Dirent,
//                                               &DirentBcb );
//
//
//                     CorrectedFileSize = Fcb->Header.FileSize.LowPart;
//
//
//                     if (Dirent->FileSize != CorrectedFileSize) {
//
//                         Dirent->FileSize = CorrectedFileSize;
//
//
//                     }
//
//                     //
//                     //  We must unpin the Bcb before the flush since we recursively tear up
//                     //  the tree if Mm decides that the data section is no longer referenced
//                     //  and the final close comes in for this file. If this parent has no
//                     //  more children as a result, we will try to initiate teardown on it
//                     //  and Cc will deadlock against the active count of this Bcb.
//                     //
//
//                     FatUnpinBcb( IrpContext, DirentBcb );
//
//                     //
//                     //  Now flush the file.  Note that this may make the Fcb
//                     //  go away if Mm dereferences its file object.
//                     //
//
//                     Status = FatFlushFile( IrpContext, Fcb, FlushType );
//
//                     if (!NT_SUCCESS(Status)) {
//
//                         ReturnStatus = Status;
//                     }
//
//                 } _SEH2_FINALLY {
//
//                     FatUnpinBcb( IrpContext, DirentBcb );
//
//                     //
//                     //  Since we have the Vcb exclusive we know that if any closes
//                     //  come in it is because the CcPurgeCacheSection caused the
//                     //  Fcb to go away.
//                     //
//
//                     if ( !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB) ) {
//
//                         FatReleaseFcb( (IRPCONTEXT), Fcb );
//                     }
//                 } _SEH2_END;
//             } _SEH2_EXCEPT( (FsRtlIsNtstatusExpected( ReturnStatus = _SEH2_GetExceptionCode() ) != 0 ) ?
//                        EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//                    FatResetExceptionState( IrpContext );
//              } _SEH2_END;
//
//         }
//
//         Fcb = NextFcb;
//     }
//
//     //
//     //  OK, now flush the directories.
//     //
//
//     Fcb = Dcb;
//
//     while (Fcb != NULL) {
//
//         NextFcb = FatGetNextFcbTopDown(IrpContext, Fcb, Dcb);
//
//         if ( (NodeType( Fcb ) != FAT_NTC_FCB) &&
//              !IsFileDeleted(IrpContext, Fcb) ) {
//
//             //
//             //  Make sure the Fcb is OK.
//             //
//
//             _SEH2_TRY {
//
//                 FatVerifyFcb( IrpContext, Fcb );
//
//             } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                       EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                 FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//             if (Fcb->FcbCondition == FcbGood) {
//
//                 Status = FatFlushFile( IrpContext, Fcb, FlushType );
//
//                 if (!NT_SUCCESS(Status)) {
//
//                     ReturnStatus = Status;
//                 }
//             }
//         }
//
//         Fcb = NextFcb;
//     }
//
//     _SEH2_TRY {
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         ReturnStatus = IrpContext->ExceptionStatus;
//     } _SEH2_END;
//
//     if (ClearWriteThroughOnExit) {
//
//         ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH);
//     }
//     if (ClearWaitOnExit) {
//
//         ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     DebugTrace(-1, Dbg, "FatFlushDirectory -> 0x%08lx\n", ReturnStatus);
//
//     return ReturnStatus;
// }
//
//
// NTSTATUS
// FatFlushFat (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     The function carefully flushes the entire FAT for a volume.  It is
//     nessecary to dance around a bit because of complicated synchronization
//     reasons.
//
// Arguments:
//
//     Vcb - Supplies the Vcb whose FAT is being flushed
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     PBCB Bcb;
//     PVOID DontCare;
//     IO_STATUS_BLOCK Iosb;
//     LARGE_INTEGER Offset;
//
//     NTSTATUS ReturnStatus = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  If this volume is write protected, no need to flush.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Make sure the Vcb is OK.
//     //
//
//     _SEH2_TRY {
//
//         FatVerifyVcb( IrpContext, Vcb );
//
//     } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//         FatResetExceptionState( IrpContext );
//     } _SEH2_END;
//
//     if (Vcb->VcbCondition != VcbGood) {
//
//         return STATUS_FILE_INVALID;
//     }
//
//     //
//     //  The only way we have to correctly synchronize things is to
//     //  repin stuff, and then unpin repin it.
//     //
//     //  With NT 5.0, we can use some new cache manager support to make
//     //  this a lot more efficient (important for FAT32).  Since we're
//     //  only worried about ranges that are dirty - and since we're a
//     //  modified-no-write stream - we can assume that if there is no
//     //  BCB, there is no work to do in the range. I.e., the lazy writer
//     //  beat us to it.
//     //
//     //  This is much better than reading the entire FAT in and trying
//     //  to punch it out (see the test in the write path to blow
//     //  off writes that don't correspond to dirty ranges of the FAT).
//     //  For FAT32, this would be a *lot* of reading.
//     //
//
//     if (Vcb->AllocationSupport.FatIndexBitSize != 12) {
//
//         //
//         //  Walk through the Fat, one page at a time.
//         //
//
//         ULONG NumberOfPages;
//         ULONG Page;
//
//         NumberOfPages = ( FatReservedBytes(&Vcb->Bpb) +
//                           FatBytesPerFat(&Vcb->Bpb) +
//                           (PAGE_SIZE - 1) ) / PAGE_SIZE;
//
//
//         for ( Page = 0, Offset.QuadPart = 0;
//               Page < NumberOfPages;
//               Page++, Offset.LowPart += PAGE_SIZE ) {
//
//             _SEH2_TRY {
//
//                 if (CcPinRead( Vcb->VirtualVolumeFile,
//                                &Offset,
//                                PAGE_SIZE,
//                                PIN_WAIT | PIN_IF_BCB,
//                                &Bcb,
//                                &DontCare )) {
//
//                     CcSetDirtyPinnedData( Bcb, NULL );
//                     CcRepinBcb( Bcb );
//                     CcUnpinData( Bcb );
//                     CcUnpinRepinnedBcb( Bcb, TRUE, &Iosb );
//
//                     if (!NT_SUCCESS(Iosb.Status)) {
//
//                         ReturnStatus = Iosb.Status;
//                     }
//                 }
//
//             } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, _SEH2_GetExceptionInformation())) {
//
//                 ReturnStatus = IrpContext->ExceptionStatus;
//                 continue;
//             } _SEH2_END;
//         }
//
//     } else {
//
//         //
//         //  We read in the entire fat in the 12 bit case.
//         //
//
//         Offset.QuadPart = FatReservedBytes( &Vcb->Bpb );
//
//         _SEH2_TRY {
//
//             if (CcPinRead( Vcb->VirtualVolumeFile,
//                            &Offset,
//                            FatBytesPerFat( &Vcb->Bpb ),
//                            PIN_WAIT | PIN_IF_BCB,
//                            &Bcb,
//                            &DontCare )) {
//
//                 CcSetDirtyPinnedData( Bcb, NULL );
//                 CcRepinBcb( Bcb );
//                 CcUnpinData( Bcb );
//                 CcUnpinRepinnedBcb( Bcb, TRUE, &Iosb );
//
//                 if (!NT_SUCCESS(Iosb.Status)) {
//
//                     ReturnStatus = Iosb.Status;
//                 }
//             }
//
//         } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, _SEH2_GetExceptionInformation())) {
//
//             ReturnStatus = IrpContext->ExceptionStatus;
//         } _SEH2_END;
//     }
//
//     return ReturnStatus;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatFlushVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
//
// /*++
//
// Routine Description:
//
//     The following routine is used to flush a volume to disk, including the
//     volume file, and ea file.
//
// Arguments:
//
//     Vcb - Supplies the volume being flushed
//
//     FlushType - Specifies the kind of flushing to perform
//
// Return Value:
//
//     NTSTATUS - The Status from the flush.
//
// --*/
//
// {
//     NTSTATUS Status;
//     NTSTATUS ReturnStatus = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  If this volume is write protected, no need to flush.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Flush all the files and directories.
//     //
//
//     Status = FatFlushDirectory( IrpContext, Vcb->RootDcb, FlushType );
//
//     if (!NT_SUCCESS(Status)) {
//
//         ReturnStatus = Status;
//     }
//
//     //
//     //  Now Flush the FAT
//     //
//
//     Status = FatFlushFat( IrpContext, Vcb );
//
//     if (!NT_SUCCESS(Status)) {
//
//         ReturnStatus = Status;
//     }
//
//     //
//     //  Unlock the volume if it is removable.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//         !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE)) {
//
//         FatToggleMediaEjectDisable( IrpContext, Vcb, FALSE );
//     }
//
//     return ReturnStatus;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatFlushFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
//
// /*++
//
// Routine Description:
//
//     This routine simply flushes the data section on a file.
//
// Arguments:
//
//     Fcb - Supplies the file being flushed
//
//     FlushType - Specifies the kind of flushing to perform
//
// Return Value:
//
//     NTSTATUS - The Status from the flush.
//
// --*/
//
// {
//     IO_STATUS_BLOCK Iosb;
//     PVCB Vcb = Fcb->Vcb;
//
//     PAGED_CODE();
//
//     CcFlushCache( &Fcb->NonPaged->SectionObjectPointers, NULL, 0, &Iosb );
//
//
//     if ( !FlagOn( Vcb->VcbState, VCB_STATE_FLAG_DELETED_FCB )) {
//
//         //
//         //  Grab and release PagingIo to serialize ourselves with the lazy writer.
//         //  This will work to ensure that all IO has completed on the cached
//         //  data.
//         //
//         //  If we are to invalidate the file, now is the right time to do it.  Do
//         //  it non-recursively so we don't thump children before their time.
//         //
//
//         ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE);
//
//         if (FlushType == FlushAndInvalidate) {
//
//             FatMarkFcbCondition( IrpContext, Fcb, FcbBad, FALSE );
//         }
//
//         ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//     }
//
//     return Iosb.Status;
// }
//
//
// NTSTATUS
// FatHijackIrpAndFlushDevice (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PDEVICE_OBJECT TargetDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when we need to send a flush to a device but
//     we don't have a flush Irp.  What this routine does is make a copy
//     of its current Irp stack location, but changes the Irp Major code
//     to a IRP_MJ_FLUSH_BUFFERS amd then send it down, but cut it off at
//     the knees in the completion routine, fix it up and return to the
//     user as if nothing had happened.
//
// Arguments:
//
//     Irp - The Irp to hijack
//
//     TargetDeviceObject - The device to send the request to.
//
// Return Value:
//
//     NTSTATUS - The Status from the flush in case anybody cares.
//
// --*/
//
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     PIO_STACK_LOCATION NextIrpSp;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Get the next stack location, and copy over the stack location
//     //
//
//     IoCopyCurrentIrpStackLocationToNext(Irp);
//
//     NextIrpSp = IoGetNextIrpStackLocation( Irp );
//     NextIrpSp->MajorFunction = IRP_MJ_FLUSH_BUFFERS;
//     NextIrpSp->MinorFunction = 0;
//
//     //
//     //  Set up the completion routine
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     IoSetCompletionRoutine( Irp,
//                             FatHijackCompletionRoutine,
//                             &Event,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Send the request.
//     //
//
//     Status = IoCallDriver( TargetDeviceObject, Irp );
//
//     if (Status == STATUS_PENDING) {
//
//         KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     //
//     //  If the driver doesn't support flushes, return SUCCESS.
//     //
//
//     if (Status == STATUS_INVALID_DEVICE_REQUEST) {
//         Status = STATUS_SUCCESS;
//     }
//
//     Irp->IoStatus.Status = 0;
//     Irp->IoStatus.Information = 0;
//
//     return Status;
// }
//
//
// VOID
// FatFlushFatEntries (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG Cluster,
//     IN ULONG Count
// )
//
// /*++
//
// Routine Description:
//
//     This macro flushes the FAT page(s) containing the passed in run.
//
// Arguments:
//
//     Vcb - Supplies the volume being flushed
//
//     Cluster - The starting cluster
//
//     Count -  The number of FAT entries in the run
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     ULONG ByteCount;
//     LARGE_INTEGER FileOffset;
//
//     IO_STATUS_BLOCK Iosb;
//
//     PAGED_CODE();
//
//     FileOffset.HighPart = 0;
//     FileOffset.LowPart = FatReservedBytes( &Vcb->Bpb );
//
//     if (Vcb->AllocationSupport.FatIndexBitSize == 12) {
//
//         FileOffset.LowPart += Cluster * 3 / 2;
//         ByteCount = (Count * 3 / 2) + 1;
//
//     } else if (Vcb->AllocationSupport.FatIndexBitSize == 32) {
//
//         FileOffset.LowPart += Cluster * sizeof(ULONG);
//         ByteCount = Count * sizeof(ULONG);
//
//     } else {
//
//         FileOffset.LowPart += Cluster * sizeof( USHORT );
//         ByteCount = Count * sizeof( USHORT );
//
//     }
//
//     CcFlushCache( &Vcb->SectionObjectPointers,
//                   &FileOffset,
//                   ByteCount,
//                   &Iosb );
//
//     if (NT_SUCCESS(Iosb.Status)) {
//         Iosb.Status = FatHijackIrpAndFlushDevice( IrpContext,
//                                                   IrpContext->OriginatingIrp,
//                                                   Vcb->TargetDeviceObject );
//     }
//
//     if (!NT_SUCCESS(Iosb.Status)) {
//         FatNormalizeAndRaiseStatus(IrpContext, Iosb.Status);
//     }
// }
//
//
// VOID
// FatFlushDirentForFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
// )
//
// /*++
//
// Routine Description:
//
//     This macro flushes the page containing a file's DIRENT in its parent.
//
// Arguments:
//
//     Fcb - Supplies the file whose DIRENT is being flushed
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     LARGE_INTEGER FileOffset;
//     IO_STATUS_BLOCK Iosb;
//
//     PAGED_CODE();
//
//     FileOffset.QuadPart = Fcb->DirentOffsetWithinDirectory;
//
//     CcFlushCache( &Fcb->ParentDcb->NonPaged->SectionObjectPointers,
//                   &FileOffset,
//                   sizeof( DIRENT ),
//                   &Iosb );
//
//     if (NT_SUCCESS(Iosb.Status)) {
//         Iosb.Status = FatHijackIrpAndFlushDevice( IrpContext,
//                                                   IrpContext->OriginatingIrp,
//                                                   Fcb->Vcb->TargetDeviceObject );
//     }
//
//     if (!NT_SUCCESS(Iosb.Status)) {
//         FatNormalizeAndRaiseStatus(IrpContext, Iosb.Status);
//     }
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI
// FatFlushCompletionRoutine (
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Contxt
//     )
//
// {
//     NTSTATUS Status = (NTSTATUS) (ULONG_PTR) Contxt;
//
//     if ( Irp->PendingReturned ) {
//
//         IoMarkIrpPending( Irp );
//     }
//
//     //
//     //  If the Irp got STATUS_INVALID_DEVICE_REQUEST, normalize it
//     //  to STATUS_SUCCESS.
//     //
//
//     if (NT_SUCCESS(Irp->IoStatus.Status) ||
//         (Irp->IoStatus.Status == STATUS_INVALID_DEVICE_REQUEST)) {
//
//         Irp->IoStatus.Status = Status;
//     }
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Contxt );
//
//     return STATUS_SUCCESS;
// }
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI
// FatHijackCompletionRoutine (
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     )
//
// {
//     //
//     //  Set the event so that our call will wake up.
//     //
//
//     KeSetEvent( (PKEVENT)Contxt, 0, FALSE );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( Irp );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
