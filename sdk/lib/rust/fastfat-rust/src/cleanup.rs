#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{IoAcquireVpbSpinLock, IoGetCurrentIrpStackLocation, IoReleaseVpbSpinLock, IoSetTopLevelIrp, LARGE_INTEGER, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER, PSHARE_ACCESS};
use core::ffi::c_void;
use core::ffi::{CStr};
use core::ptr;
use core::slice;
// resolve_imports.py: explicit kernel/private-type imports
use crate::dirsup::{FatUpdateDirentFromFcb};
use crate::types::{CCB_FLAG_COMPLETE_DISMOUNT, CCB_FLAG_DELETE_ON_CLOSE, CCB_FLAG_DENY_DEFRAG, DELETE_CONTEXT, FAT_BUG_CHECK_CLEANUP, FCB_STATE_DELAY_CLOSE, FCB_STATE_DELETE_ON_CLOSE, FCB_STATE_DENY_DEFRAG, FCB_STATE_FLUSH_FAT, FCB_STATE_PAGING_FILE, FCB_STATE_TRUNCATE_ON_CLOSE, FatLargeZero, IRP_CONTEXT, PCCB, PFCB, PIRP_CONTEXT, PVCB, VCB, VCB_STATE_FLAG_DEFERRED_FLUSH, VCB_STATE_FLAG_LOCKED, VCB_STATE_FLAG_SHUTDOWN, VCB_STATE_FLAG_WRITE_PROTECTED, _TYPE_OF_OPEN as TYPE_OF_OPEN, _TYPE_OF_OPEN_DirectoryFile as DirectoryFile, _TYPE_OF_OPEN_EaFile as EaFile, _TYPE_OF_OPEN_UnopenedFileObject as UnopenedFileObject, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen, _TYPE_OF_OPEN_UserVolumeOpen as UserVolumeOpen, _TYPE_OF_OPEN_VirtualVolumeFile as VirtualVolumeFile, _VCB_CONDITION_VcbGood as VcbGood};
use ntkrnl_sys::ddk::{IRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_CLEANUP, DO_VERIFY_VOLUME, FAT_NTC_DCB, FAT_NTC_ROOT_DCB, FILE_ACTION_REMOVED, FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME, FO_CACHE_SUPPORTED, FO_CLEANUP_COMPLETE, FO_FILE_MODIFIED, FSRTL_VOLUME_UNLOCK, VPB_DIRECT_WRITES_ALLOWED, VPB_LOCKED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlIsNtstatusExpected};
use ntkrnl_sys::ddk::ExReleaseResourceLite;

extern "system" {
    fn FatDeviceIsFatFsdo(VolumeDeviceObject: *mut core::ffi::c_void) -> u8;
    } extern "C" { fn DebugTrace(level: i32, flag: u32, format: *const i8, ...); } extern "system" {
    fn FatCreateIrpContext(Irp: *mut IRP, arg2: i32) -> *mut core::ffi::c_void;
    fn FatIsIrpTopLevel(Irp: *mut IRP) -> u8;
    fn FatExceptionFilter(IrpContext: *mut core::ffi::c_void, ExceptionInformation: *mut core::ffi::c_void) -> i32;
    fn FatProcessException(IrpContext: *mut core::ffi::c_void, Irp: *mut IRP, ExceptionCode: i32) -> NTSTATUS;
}
extern "system" {
    fn FatDecodeFileObject( FileObject: PFILE_OBJECT, Vcb: *mut PVCB, Fcb: *mut PFCB, Ccb: *mut PCCB, ) -> TYPE_OF_OPEN;
    fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn FlagOn(flags: u32, mask: u32) -> u32;
    fn ClearFlag(flags: *mut u32, mask: u32);
    fn SetFlag(flags: *mut u32, mask: u32);
    fn NodeType(node: *const c_void) -> u32;
    fn FatAcquireExclusiveFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB) -> u8;
    fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    fn FatReleaseFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    fn FatVerifyFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn FatResetExceptionState(IrpContext: PIRP_CONTEXT);
    fn FsRtlNotifyCleanup(Sync: *mut c_void, List: *mut c_void, Context: *mut c_void);
    fn FsRtlNotifyFullChangeDirectory( Sync: *mut c_void, List: *mut c_void, Context: *mut c_void, FullDirectoryName: *const c_void, WatchTree: BOOLEAN, IgnoreBuffer: BOOLEAN, CompletionFilter: u32, NotifyIrp: PIRP, TraverseCallback: *const c_void, SubjectContext: *const c_void, );
    fn FatBugCheck(arg1: usize, arg2: usize, arg3: usize);
    fn FatFlushFat(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> NTSTATUS;
    fn FatIsFat12(Vcb: PVCB) -> BOOLEAN;
    fn FatHijackIrpAndFlushDevice(IrpContext: PIRP_CONTEXT, Irp: PIRP, TargetDeviceObject: *mut c_void) -> NTSTATUS;
    fn FatCheckForDismount(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Force: BOOLEAN) -> u8;
    fn FatIsDirectoryEmpty(IrpContext: PIRP_CONTEXT, Fcb: PFCB) -> BOOLEAN;
    fn FatTruncateFileAllocation(IrpContext: PIRP_CONTEXT, Fcb: PFCB, Size: i32);
    fn FatDeleteDirent(IrpContext: PIRP_CONTEXT, Fcb: PFCB, DeleteContext: *const DELETE_CONTEXT, IgnoreCase: u32);
    fn FatTunnelFcbOrDcb(FcbOrDcb: *mut c_void, Ccb: PCCB);
    fn FatNotifyReportChange(IrpContext: PIRP_CONTEXT, Vcb: PVCB, Fcb: PFCB, Filter: u32, Action: u32);
    fn FatRemoveNames(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    fn FatSetFileSizeInDirent(IrpContext: PIRP_CONTEXT, Fcb: PFCB, AlternativeFileSize: *const ULONG);
    fn FatZeroData(IrpContext: PIRP_CONTEXT, Vcb: PVCB, FileObject: PFILE_OBJECT, Offset: i32, Length: i32) -> u8;
    fn FatUnpinRepinnedBcbs(IrpContext: PIRP_CONTEXT);
    fn CcFlushCache(SectionObjectPointers: *mut c_void, FileOffset: *const LARGE_INTEGER, Length: u32, IoStatus: *mut c_void);
    fn CcPurgeCacheSection(SectionObjectPointers: *mut c_void, FileOffset: *const LARGE_INTEGER, Length: u32, UninitializeCacheMaps: BOOLEAN) -> u8;
    fn CcSetFileSizes(FileObject: PFILE_OBJECT, FileSizes: *const c_void);
    fn FsRtlFastUnlockAll(Locks: *mut c_void, FileObject: PFILE_OBJECT, ProcessId: *mut c_void, Context: *mut c_void) -> BOOLEAN;
    fn IoGetRequestorProcess(Irp: PIRP) -> *mut c_void;
    fn CcIsFileCached(FileObject: PFILE_OBJECT) -> BOOLEAN;
    fn FatIsFastIoPossible(Fcb: PFCB) -> BOOLEAN;
    fn FsRtlCheckOplock( Oplock: *mut c_void, Irp: PIRP, Context: *mut c_void, CompletionRoutine: *const c_void, PostIrpRoutine: *const c_void, );
    fn FatNormalizeAndRaiseStatus(IrpContext: PIRP_CONTEXT, Status: NTSTATUS);
    fn FatFlushFile(IrpContext: PIRP_CONTEXT, Fcb: PFCB, FlushType: u32) -> NTSTATUS;
    fn FsRtlCheckOplockEx( Oplock: *mut c_void, Irp: PIRP, Flags: u32, Context: *mut c_void, CompletionRoutine: *const c_void, PostIrpRoutine: *const c_void, ) -> NTSTATUS;
    fn FatOplockComplete(IrpContext: PIRP_CONTEXT, Irp: PIRP);
    fn FatPrePostIrp(IrpContext: PIRP_CONTEXT, Irp: PIRP);
    fn FatGetFcbOplock(Fcb: PFCB) -> *mut c_void;
    fn FatIsFileOplockable(Fcb: PFCB) -> BOOLEAN;
    fn FsRtlNotifyVolumeEvent(FileObject: PFILE_OBJECT, Event: u32);
    fn ExAcquireResourceExclusiveLite(Resource: *mut c_void, Wait: BOOLEAN) -> BOOLEAN;
    // ExReleaseResourceLite: kernel FASTCALL export — local "system" decl was a
    // §9.99 fold class; use the ntkrnl-sys bindgen (fastcall) decl instead.
    fn CcUninitializeCacheMap(FileObject: PFILE_OBJECT, TruncateSize: *const LARGE_INTEGER, Vacant: *mut c_void) -> BOOLEAN;
    fn IoRemoveShareAccess(FileObject: PFILE_OBJECT, ShareAccess: *mut c_void);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_CLEANUP;
pub const Dbg: u32 = DEBUG_TRACE_CLEANUP;

// Forward declarations for unknown external symbols

// Gap function expansions (macros)
use crate::seh::{EXCEPTION_CONTINUE_SEARCH, EXCEPTION_EXECUTE_HANDLER};

/// `FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ? EXCEPTION_EXECUTE_HANDLER :
/// EXCEPTION_CONTINUE_SEARCH`, the filter of every "cannot turn back" try here.
unsafe fn expected_status_filter(pointers: *mut c_void) -> i32 {
    if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers)) != 0 {
        EXCEPTION_EXECUTE_HANDLER
    } else {
        EXCEPTION_CONTINUE_SEARCH
    }
}
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_PENDING: NTSTATUS = 0x103;
const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;
const Flush: u32 = 1; // flush type
const FcbGood: u32 = 1; // assumed value
const FcbBad: u32 = 2; // assumed value

#[no_mangle]
pub unsafe extern "system" fn FatFsdCleanup(
    VolumeDeviceObject: *mut core::ffi::c_void,
    Irp: *mut IRP,
) -> NTSTATUS
{
    const FILE_OPENED: u32 = 0x01;

    let mut Status: NTSTATUS = 0;
    let mut IrpContext: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut TopLevel: u8 = 0;

    PAGED_CODE!();

    if FatDeviceIsFatFsdo(VolumeDeviceObject) != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = (FILE_OPENED) as _;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
        return STATUS_SUCCESS;
    }

    DebugTrace(1, Dbg, b"FatFsdCleanup\n\0".as_ptr() as *const i8, 0);

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, ... ))
    let p_ic = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_ic as *const *mut c_void, || {
        *p_ic = FatCreateIrpContext(Irp, 1);
        *p_status = FatCommonCleanup((*p_ic).cast(), Irp);
    }) {
        //  We had some trouble trying to perform the requested
        //  operation, so we'll abort the I/O request with
        //  the error status that we get back from the
        //  execption code
        Status = FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }
    FsRtlExitFileSystem();

    DebugTrace(-1, Dbg, b"FatFsdCleanup -> %08lx\n\0".as_ptr() as *const i8, Status as i32);

    let _ = VolumeDeviceObject;

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FatCommonCleanup(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {

    // PAGED_CODE(); -> nothing in this translation
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatCommonCleanup\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"Irp           = %p\n\0".as_ptr() as *const i8, Irp as usize);
    DebugTrace(0, Dbg, b"->FileObject  = %p\n\0".as_ptr() as *const i8, (*IrpSp).FileObject as usize);

    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;

    let mut Vcb: PVCB = ptr::null_mut();
    let mut Fcb: PFCB = ptr::null_mut();
    let mut Ccb: PCCB = ptr::null_mut();
    let TypeOfOpen: TYPE_OF_OPEN = FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb);

    if TypeOfOpen == UnopenedFileObject {
        DebugTrace(0, Dbg, b"Unopened File Object\n\0".as_ptr() as *const i8, 0);
        FatCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        DebugTrace(-1, Dbg, b"FatCommonCleanup -> STATUS_SUCCESS\n\0".as_ptr() as *const i8, 0);
        return STATUS_SUCCESS;
    }

    if FlagOn((*FileObject).Flags, FO_CLEANUP_COMPLETE) != 0 {
        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH) != 0
            && FlagOn((*FileObject).Flags, FO_FILE_MODIFIED) != 0
            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
            && TypeOfOpen == UserFileOpen
        {
            let Status: NTSTATUS = FatFlushFile(IrpContext, Fcb, Flush);
            if !NT_SUCCESS(Status) {
                FatNormalizeAndRaiseStatus(IrpContext, Status);
            }
        }
        FatCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
        DebugTrace(-1, Dbg, b"FatCommonCleanup -> STATUS_SUCCESS\n\0".as_ptr() as *const i8, 0);
        return STATUS_SUCCESS;
    }

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SendUnlockNotification: BOOLEAN = FALSE;
    let mut AcquiredVcb: BOOLEAN = FALSE;
    let mut AcquiredFcb: BOOLEAN = FALSE;
    let mut TruncateSize: *const LARGE_INTEGER = ptr::null();
    let mut LocalTruncateSize: LARGE_INTEGER = core::mem::zeroed();
    #[cfg(ntddi_ge_ntddi_win8)]
    let mut ProcessingDeleteOnClose: BOOLEAN = FALSE;
    let mut ShareAccess: *mut c_void = ptr::null_mut();

    if TypeOfOpen == UserFileOpen || TypeOfOpen == UserDirectoryOpen {
        NT_ASSERT!((Fcb != ptr::null_mut()) as u8);
        FatAcquireExclusiveFcb(IrpContext, Fcb);
        AcquiredFcb = TRUE;

        if FlagOn((*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE) != 0 {
            NT_ASSERT!((NodeType(Fcb as *const c_void) != FAT_NTC_ROOT_DCB) as u8);
            SetFlag(&mut (*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE);
            ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_DELETE_ON_CLOSE);
            #[cfg(ntddi_ge_ntddi_win8)]
            {
                ProcessingDeleteOnClose = TRUE;
            }
            if TypeOfOpen == UserDirectoryOpen {
                FsRtlNotifyFullChangeDirectory(
                    (*Vcb).NotifySync as *mut c_void,
                    core::ptr::addr_of_mut!((*Vcb).DirNotifyList) as *mut c_void,
                    (*FileObject).FsContext as *mut c_void,
                    ptr::null(),
                    FALSE,
                    FALSE,
                    0,
                    ptr::null_mut(),
                    ptr::null(),
                    ptr::null(),
                );
            }
        }

        if (*Fcb).UncleanCount == 1
            && FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0
            && (*Fcb).FcbCondition != FcbBad
            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
        {
            FatReleaseFcb(IrpContext, Fcb);
            AcquiredFcb = FALSE;
            FatAcquireExclusiveVcb(IrpContext, Vcb);
            AcquiredVcb = TRUE;
            FatAcquireExclusiveFcb(IrpContext, Fcb);
            AcquiredFcb = TRUE;
        }
    }

    if TypeOfOpen == UserVolumeOpen {
        FatAcquireExclusiveVcb(IrpContext, Vcb);
        AcquiredVcb = TRUE;
    }

    if TypeOfOpen == UserDirectoryOpen {
        FsRtlNotifyCleanup(
            (*Vcb).NotifySync as *mut c_void,
            core::ptr::addr_of_mut!((*Vcb).DirNotifyList) as *mut c_void,
            Ccb as *mut c_void,
        );
    }

    if Fcb != ptr::null_mut() {
        //  Stop any raises from FatVerifyFcb, unless it is REAL bad.
        crate::seh::try_finally(&mut (), |_| {
            if crate::seh::try_except(&mut (), |_| {
                FatVerifyFcb(IrpContext, Fcb);
            }, |_, p| expected_status_filter(p)).is_err() {
                FatResetExceptionState(IrpContext);
            }
        }, |_, abnormal| {
            if abnormal {
                //  We will be raising out of here.
                if AcquiredFcb != FALSE { FatReleaseFcb(IrpContext, Fcb); }
                if AcquiredVcb != FALSE { FatReleaseVcb(IrpContext, Vcb); }
            }
        });
    }

    // _SEH2_TRY { ... } _SEH2_FINALLY { ... }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_send_unlock = core::ptr::addr_of_mut!(SendUnlockNotification);
    crate::seh::try_finally(&mut (), |_| {

        // The big try code
        #[cfg(ntddi_ge_ntddi_win8)]
        if ProcessingDeleteOnClose != 0
            && FatIsFileOplockable(Fcb) != 0
            && (NodeType(Fcb as *const c_void) != FAT_NTC_DCB || FatIsDirectoryEmpty(IrpContext, Fcb) != 0)
        {
            *p_status = FsRtlCheckOplockEx(
                FatGetFcbOplock(Fcb),
                Irp,
                OPLOCK_FLAG_CLOSING_DELETE_ON_CLOSE,
                IrpContext as *mut c_void,
                FatOplockComplete as *const c_void,
                FatPrePostIrp as *const c_void,
            );
            if *p_status != STATUS_SUCCESS {
                if *p_status == STATUS_PENDING {
                    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_CLEANUP_BREAKING_OPLOCK);
                    return; // try_return( Status )
                } else {
                    FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                }
            }
        }

        // switch(TypeOfOpen) { ... }
        // We'll translate switch to match
        match TypeOfOpen {
            DirectoryFile | VirtualVolumeFile => {
                DebugTrace(0, Dbg, b"Cleanup VirtualVolumeFile/DirectoryFile\n\0".as_ptr() as *const i8, 0);
                ShareAccess = ptr::null_mut();
            }
            UserVolumeOpen => {
                DebugTrace(0, Dbg, b"Cleanup UserVolumeOpen\n\0".as_ptr() as *const i8, 0);
                if FlagOn((*Ccb).Flags, CCB_FLAG_COMPLETE_DISMOUNT) != 0 {
                    FatCheckForDismount(IrpContext, Vcb, TRUE);
                } else if (*FileObject).WriteAccess != 0 && FlagOn((*FileObject).Flags, FO_FILE_MODIFIED) != 0 {
                    FatHijackIrpAndFlushDevice(IrpContext, Irp, ((*Vcb).TargetDeviceObject).cast());
                    SetFlag(&mut (*(*(*Vcb).Vpb).RealDevice).Flags, DO_VERIFY_VOLUME);
                }
                if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_LOCKED) != 0 && (*Vcb).FileObjectWithVcbLocked == FileObject {
                    FatAutoUnlock(IrpContext, Vcb);
                    *p_send_unlock = TRUE;
                }
                ShareAccess = core::ptr::addr_of!((*Vcb).ShareAccess) as *mut c_void;
            }
            EaFile => {
                DebugTrace(0, Dbg, b"Cleanup EaFileObject\n\0".as_ptr() as *const i8, 0);
                ShareAccess = ptr::null_mut();
            }
            UserDirectoryOpen => {
                DebugTrace(0, Dbg, b"Cleanup UserDirectoryOpen\n\0".as_ptr() as *const i8, 0);
                ShareAccess = core::ptr::addr_of!((*Fcb).ShareAccess) as *mut c_void;

                if (*Fcb).UncleanCount == 1
                    && (*Fcb).OpenCount == 1
                    && (*Fcb).Specific.Dcb.as_ref().DirectoryFileOpenCount == 0
                    && FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) == 0
                    && (*Fcb).FcbCondition == FcbGood
                {
                    SetFlag(&mut (*Fcb).FcbState, FCB_STATE_DELAY_CLOSE);
                }

                if FlagOn((*Fcb).FcbState, FCB_STATE_DENY_DEFRAG) != 0 && FlagOn((*Ccb).Flags, CCB_FLAG_DENY_DEFRAG) != 0 {
                    ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_DENY_DEFRAG);
                    ClearFlag(&mut (*Fcb).FcbState, FCB_STATE_DENY_DEFRAG);
                }

                if (*Vcb).VcbCondition == VcbGood && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_SHUTDOWN) == 0 {
                    FatUpdateDirentFromFcb(IrpContext, FileObject, Fcb, Ccb);

                    if (*Fcb).UncleanCount == 1
                        && NodeType(Fcb as *const c_void) == FAT_NTC_DCB
                        && FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0
                        && (*Fcb).FcbCondition == FcbGood
                        && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
                    {
                        if FatIsDirectoryEmpty(IrpContext, Fcb) == 0 {
                            ClearFlag(&mut (*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE);
                        } else {
                            //  Even if something goes wrong, we cannot turn back!
                            if crate::seh::try_except(&mut (), |_| {
                                let mut DeleteContext = DELETE_CONTEXT {
                                    FileSize: 0,
                                    FirstClusterOfFile: 0,
                                };
                                DeleteContext.FileSize = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                                DeleteContext.FirstClusterOfFile = (*Fcb).FirstClusterOfFile;

                                ExAcquireResourceExclusiveLite(((*Fcb).Header.PagingIoResource).cast(), TRUE);
                                (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = 0;
                                ExReleaseResourceLite(((*Fcb).Header.PagingIoResource).cast());

                                DebugTrace(0, Dbg, b"Delete File allocation\n\0".as_ptr() as *const i8, 0);
                                FatTruncateFileAllocation(IrpContext, Fcb, 0);

                                if (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart == 0 {
                                    DebugTrace(0, Dbg, b"Delete the directory dirent\n\0".as_ptr() as *const i8, 0);
                                    FatTunnelFcbOrDcb(Fcb as *mut c_void, ptr::null_mut());
                                    FatDeleteDirent(IrpContext, Fcb, &DeleteContext, (TRUE) as _);
                                    FatNotifyReportChange(
                                        IrpContext, Vcb, Fcb,
                                        FILE_NOTIFY_CHANGE_DIR_NAME,
                                        FILE_ACTION_REMOVED,
                                    );
                                }
                            }, |_, p| expected_status_filter(p)).is_err() {
                                FatResetExceptionState(IrpContext);
                            }

                            FatRemoveNames(IrpContext, Fcb);
                            #[cfg(ntddi_ge_ntddi_win8)]
                            {
                                let BreakStatus = FsRtlCheckOplockEx(
                                    FatGetFcbOplock((*Fcb).ParentDcb),
                                    Irp,
                                    OPLOCK_FLAG_PARENT_OBJECT | OPLOCK_FLAG_REMOVING_FILE_OR_LINK,
                                    ptr::null_mut(),
                                    ptr::null(),
                                    ptr::null(),
                                );
                                NT_ASSERT!(BreakStatus != STATUS_PENDING);
                            }
                        }
                    }
                }

                NT_ASSERT!(((*Fcb).UncleanCount != 0) as u8);
                (*Fcb).UncleanCount -= 1;
            }
            UserFileOpen => {
                DebugTrace(0, Dbg, b"Cleanup UserFileOpen\n\0".as_ptr() as *const i8, 0);
                ShareAccess = core::ptr::addr_of!((*Fcb).ShareAccess) as *mut c_void;

                if (*(*FileObject).SectionObjectPointer).DataSectionObject.is_null()
                    && (*(*FileObject).SectionObjectPointer).ImageSectionObject.is_null()
                    && (*Fcb).UncleanCount == 1
                    && (*Fcb).OpenCount == 1
                    && FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) == 0
                    && FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0
                    && (*Fcb).FcbCondition == FcbGood
                {
                    SetFlag(&mut (*Fcb).FcbState, FCB_STATE_DELAY_CLOSE);
                }

                if FlagOn((*Fcb).FcbState, FCB_STATE_DENY_DEFRAG) != 0 && FlagOn((*Ccb).Flags, CCB_FLAG_DENY_DEFRAG) != 0 {
                    ClearFlag(&mut (*Ccb).Flags, CCB_FLAG_DENY_DEFRAG);
                    ClearFlag(&mut (*Fcb).FcbState, FCB_STATE_DENY_DEFRAG);
                }

                FsRtlFastUnlockAll(
                    core::ptr::addr_of_mut!((*Fcb).Specific.Fcb.as_mut().FileLock) as *mut c_void,
                    FileObject,
                    IoGetRequestorProcess(Irp),
                    ptr::null_mut(),
                );

                if (*Vcb).VcbCondition == VcbGood && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_SHUTDOWN) == 0 {
                    if (*Fcb).FcbCondition == FcbGood {
                        FatUpdateDirentFromFcb(IrpContext, FileObject, Fcb, Ccb);
                    }

                    if (*Fcb).UncleanCount == 1 && (*Fcb).FcbCondition == FcbGood {
                        let mut DeleteContext = DELETE_CONTEXT {
                            FileSize: 0,
                            FirstClusterOfFile: 0,
                        };

                        if FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0
                            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
                        {
                            DeleteContext.FileSize = (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                            DeleteContext.FirstClusterOfFile = (*Fcb).FirstClusterOfFile;

                            DebugTrace(0, Dbg, b"Delete File allocation\n\0".as_ptr() as *const i8, 0);

                            ExAcquireResourceExclusiveLite(((*Fcb).Header.PagingIoResource).cast(), TRUE);
                            (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart = 0;
                            (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = 0;
                            (*Fcb).ValidDataToDisk = 0;
                            ExReleaseResourceLite(((*Fcb).Header.PagingIoResource).cast());

                            if crate::seh::try_except(&mut (), |_| {
                                FatSetFileSizeInDirent(IrpContext, Fcb, ptr::null());
                            }, |_, p| expected_status_filter(p)).is_err() {
                                FatResetExceptionState(IrpContext);
                            }

                            (*Fcb).FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
                        } else {
                            if FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0
                                && (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart < (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart
                            {
                                let mut ValidDataLength: u32 = (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart;
                                if ValidDataLength < (*Fcb).ValidDataToDisk {
                                    ValidDataLength = (*Fcb).ValidDataToDisk;
                                }
                                if ValidDataLength < (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart {
                                    if crate::seh::try_except(&mut (), |_| {
                                        let _ = FatZeroData(
                                            IrpContext, Vcb, FileObject,
                                            ValidDataLength as i32,
                                            ((*Fcb).Header.FileSize.__bindgen_anon_1.LowPart - ValidDataLength) as i32,
                                        );
                                        (*Fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart =
                                        (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;
                                        (*Fcb).ValidDataToDisk =
                                        (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart;

                                        if CcIsFileCached(FileObject) != 0 {
                                            CcSetFileSizes(
                                                FileObject,
                                                core::ptr::addr_of!((*Fcb).Header.AllocationSize) as *const c_void,
                                            );
                                        }
                                    }, |_, p| expected_status_filter(p)).is_err() {
                                        FatResetExceptionState(IrpContext);
                                    }
                                }
                            }
                        }

                        // _SEH2_TRY around truncate and delete logic
                        if crate::seh::try_except(&mut (), |_| {
                            if FlagOn((*Fcb).FcbState, FCB_STATE_TRUNCATE_ON_CLOSE) != 0 {
                                DebugTrace(0, Dbg, b"truncate file allocation\n\0".as_ptr() as *const i8, 0);
                                if (*Vcb).VcbCondition == VcbGood {
                                    FatTruncateFileAllocation(
                                        IrpContext, Fcb,
                                        ((*Fcb).Header.FileSize.__bindgen_anon_1.LowPart) as i32,
                                    );
                                }
                                // LocalTruncateSize = FileSize
                                LocalTruncateSize = (*Fcb).Header.FileSize;
                                TruncateSize = core::ptr::addr_of!(LocalTruncateSize);
                                (*Fcb).FcbState &= !FCB_STATE_TRUNCATE_ON_CLOSE;
                            }

                            if FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0
                                && (*Fcb).Header.AllocationSize.__bindgen_anon_1.LowPart == 0
                            {
                                DebugTrace(0, Dbg, b"Delete File\n\0".as_ptr() as *const i8, 0);
                                FatTunnelFcbOrDcb(Fcb as *mut c_void, Ccb);
                                FatDeleteDirent(IrpContext, Fcb, &DeleteContext, (TRUE) as _);
                                FatNotifyReportChange(
                                    IrpContext, Vcb, Fcb,
                                    FILE_NOTIFY_CHANGE_FILE_NAME,
                                    FILE_ACTION_REMOVED,
                                );
                            }
                        }, |_, p| expected_status_filter(p)).is_err() {
                            FatResetExceptionState(IrpContext);
                        }

                        if FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0 {
                            FatRemoveNames(IrpContext, Fcb);
                            #[cfg(ntddi_ge_ntddi_win8)]
                            {
                                let BreakStatus = FsRtlCheckOplockEx(
                                    FatGetFcbOplock((*Fcb).ParentDcb),
                                    Irp,
                                    OPLOCK_FLAG_PARENT_OBJECT | OPLOCK_FLAG_REMOVING_FILE_OR_LINK,
                                    ptr::null_mut(),
                                    ptr::null(),
                                    ptr::null(),
                                );
                                NT_ASSERT!(BreakStatus != STATUS_PENDING);
                            }
                        }
                    }
                }

                NT_ASSERT!(((*Fcb).UncleanCount != 0) as u8);
                (*Fcb).UncleanCount -= 1;
                if FlagOn((*FileObject).Flags, FO_CACHE_SUPPORTED) == 0 {
                    NT_ASSERT!(((*Fcb).NonCachedUncleanCount != 0) as u8);
                    (*Fcb).NonCachedUncleanCount -= 1;
                }

                if FlagOn((*FileObject).Flags, FO_CACHE_SUPPORTED) != 0
                    && (*Fcb).NonCachedUncleanCount != 0
                    && (*Fcb).NonCachedUncleanCount == (*Fcb).UncleanCount
                    && !(*(*Fcb).NonPaged).SectionObjectPointers.DataSectionObject.is_null()
                {
                    CcFlushCache(
                        core::ptr::addr_of!((*(*Fcb).NonPaged).SectionObjectPointers) as *mut c_void,
                        ptr::null(),
                        0,
                        ptr::null_mut(),
                    );
                    ExAcquireResourceExclusiveLite(((*Fcb).Header.PagingIoResource).cast(), TRUE);
                    ExReleaseResourceLite(((*Fcb).Header.PagingIoResource).cast());
                    CcPurgeCacheSection(
                        core::ptr::addr_of!((*(*Fcb).NonPaged).SectionObjectPointers) as *mut c_void,
                        ptr::null(),
                        0,
                        FALSE,
                    );
                }

                if (*Fcb).FcbCondition == FcbBad {
                    TruncateSize = &raw const FatLargeZero as *const LARGE_INTEGER; // assume FatLargeZero global
                }

                CcUninitializeCacheMap(FileObject, TruncateSize, ptr::null_mut());
            }
            _ => {
                FatBugCheck(TypeOfOpen as usize, 0, 0);
            }
        }

        if ShareAccess != ptr::null_mut() {
            DebugTrace(0, Dbg, b"Cleanup the Share access\n\0".as_ptr() as *const i8, 0);
            IoRemoveShareAccess(FileObject, ShareAccess);
        }

        // NTDDI_WIN8+ also tested UserDirectoryOpen; we build at 0x502 so it's excluded.
        if TypeOfOpen == UserFileOpen
        {
            FsRtlCheckOplock(FatGetFcbOplock(Fcb), Irp, IrpContext as *mut c_void, ptr::null(), ptr::null());
            (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb);
        }

        SetFlag(&mut (*FileObject).Flags as *mut u32, FO_CLEANUP_COMPLETE);
        *p_status = STATUS_SUCCESS;
        FatUnpinRepinnedBcbs(IrpContext);

        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH) != 0
            && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
        {
            if TypeOfOpen == UserFileOpen && FlagOn((*FileObject).Flags, FO_FILE_MODIFIED) != 0 {
                *p_status = FatFlushFile(IrpContext, Fcb, Flush);
            }

            if NT_SUCCESS(*p_status) && Fcb != ptr::null_mut() && FatIsFat12(Vcb) == 0 && FlagOn((*Fcb).FcbState, FCB_STATE_FLUSH_FAT) != 0 {
                *p_status = FatFlushFat(IrpContext, Vcb);

                if NT_SUCCESS(*p_status) && (*Fcb).ParentDcb != ptr::null_mut() {
                    *p_status = FatFlushFile(IrpContext, (*Fcb).ParentDcb, Flush);
                }
            }

            if !NT_SUCCESS(*p_status) {
                FatNormalizeAndRaiseStatus(IrpContext, *p_status);
            }
        }
    }, |_, abnormal| {
        // DebugUnwind( FatCommonCleanup );
        if AcquiredFcb != 0 { FatReleaseFcb(IrpContext, Fcb); }
        if AcquiredVcb != 0 { FatReleaseVcb(IrpContext, Vcb); }
        if *p_send_unlock != 0 {
            FsRtlNotifyVolumeEvent(FileObject, FSRTL_VOLUME_UNLOCK);
        }

        //  If this is a normal termination then complete the request
        if !abnormal && *p_status != STATUS_PENDING {
            FatCompleteRequest(IrpContext, Irp, *p_status);
        }

        DebugTrace(-1, Dbg, b"FatCommonCleanup -> %08lx\n\0".as_ptr() as *const i8, *p_status as usize);
    });

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FatAutoUnlock(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    let mut SavedIrql: KIRQL = 0;
    let _ = IrpContext;

    IoAcquireVpbSpinLock(&raw mut SavedIrql);
    (*(*Vcb).Vpb).Flags &= (!(VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED)) as u16;
    (*Vcb).VcbState &= !VCB_STATE_FLAG_LOCKED;
    (*Vcb).FileObjectWithVcbLocked = core::ptr::null_mut();
    IoReleaseVpbSpinLock(SavedIrql);
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
//     Cleanup.c
//
// Abstract:
//
//     This module implements the File Cleanup routine for Fat called by the
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_CLEANUP)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_CLEANUP)
//
// //
// //  The following little routine exists solely because it need a spin lock.
// //
//
// VOID
// FatAutoUnlock (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonCleanup)
// #pragma alloc_text(PAGE, FatFsdCleanup)
// #endif
//
//
// _Function_class_(IRP_MJ_CLEANUP)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdCleanup (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of closing down a handle to a
//     file object.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file being Cleanup exists
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
//     //
//     //  If we were called with our file system device object instead of a
//     //  volume device object, just complete this request with STATUS_SUCCESS
//     //
//
//     if ( FatDeviceIsFatFsdo( VolumeDeviceObject))  {
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//         Irp->IoStatus.Information = FILE_OPENED;
//
//         IoCompleteRequest( Irp, IO_DISK_INCREMENT );
//
//         return STATUS_SUCCESS;
//     }
//
//     DebugTrace(+1, Dbg, "FatFsdCleanup\n", 0);
//
//     //
//     //  Call the common Cleanup routine, with blocking allowed.
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, TRUE );
//
//         Status = FatCommonCleanup( IrpContext, Irp );
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
//     DebugTrace(-1, Dbg, "FatFsdCleanup -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonCleanup (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for cleanup of a file/directory called by both
//     the fsd and fsp threads.
//
//     Cleanup is invoked whenever the last handle to a file object is closed.
//     This is different than the Close operation which is invoked when the last
//     reference to a file object is deleted.
//
//     The function of cleanup is to essentially "cleanup" the file/directory
//     after a user is done with it.  The Fcb/Dcb remains around (because MM
//     still has the file object referenced) but is now available for another
//     user to open (i.e., as far as the user is concerned the is now closed).
//
//     See close for a more complete description of what close does.
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
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN SendUnlockNotification = FALSE;
//
//     PSHARE_ACCESS ShareAccess;
//
//     PLARGE_INTEGER TruncateSize = NULL;
//     LARGE_INTEGER LocalTruncateSize;
//
//     BOOLEAN AcquiredVcb = FALSE;
//     BOOLEAN AcquiredFcb = FALSE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     BOOLEAN ProcessingDeleteOnClose = FALSE;
// #endif
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonCleanup\n", 0);
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
//     //  Special case the unopened file object.  This will occur only when
//     //  we are initializing Vcb and IoCreateStreamFileObject is being
//     //  called.
//     //
//
//     if (TypeOfOpen == UnopenedFileObject) {
//
//         DebugTrace(0, Dbg, "Unopened File Object\n", 0);
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//         DebugTrace(-1, Dbg, "FatCommonCleanup -> STATUS_SUCCESS\n", 0);
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  If this is not our first time through (for whatever reason)
//     //  only see if we have to flush the file.
//     //
//
//     if (FlagOn( FileObject->Flags, FO_CLEANUP_COMPLETE )) {
//
//         if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH) &&
//             FlagOn(FileObject->Flags, FO_FILE_MODIFIED) &&
//             !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) &&
//             (TypeOfOpen == UserFileOpen)) {
//
//             //
//             //  Flush the file.
//             //
//
//             Status = FatFlushFile( IrpContext, Fcb, Flush );
//
//             if (!NT_SUCCESS(Status)) {
//
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//         }
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//         DebugTrace(-1, Dbg, "FatCommonCleanup -> STATUS_SUCCESS\n", 0);
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  If we call change the allocation or call CcUninitialize,
//     //  we have to take the Fcb exclusive
//     //
//
//     if ((TypeOfOpen == UserFileOpen) || (TypeOfOpen == UserDirectoryOpen)) {
//
//         NT_ASSERT( Fcb != NULL );
//
//         (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//
//         AcquiredFcb = TRUE;
//
//         //
//         //  Do a check here if this was a DELETE_ON_CLOSE FileObject, and
//         //  set the Fcb flag appropriately.
//         //
//
//         if (FlagOn(Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE)) {
//
//             NT_ASSERT( NodeType(Fcb) != FAT_NTC_ROOT_DCB );
//
//             //
//             //  Transfer the delete-on-close state to the FCB.  We do this rather
//             //  than leave the CCB_FLAG_DELETE_ON_CLOSE flag set so that if we
//             //  end up breaking an oplock and come in again we won't try to break
//             //  the oplock again (and again, and again...).
//             //
//
//             SetFlag( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//             ClearFlag( Ccb->Flags, CCB_FLAG_DELETE_ON_CLOSE );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             ProcessingDeleteOnClose = TRUE;
// #endif
//
//             //
//             //  Report this to the dir notify package for a directory.
//             //
//
//             if (TypeOfOpen == UserDirectoryOpen) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:6309, "FullDirectoryName may be NULL if NotifyIrp is also NULL. this indicates the object is being deleted." )
// #endif
//                 FsRtlNotifyFullChangeDirectory( Vcb->NotifySync,
//                                                 &Vcb->DirNotifyList,
//                                                 FileObject->FsContext,
//                                                 NULL,
//                                                 FALSE,
//                                                 FALSE,
//                                                 0,
//                                                 NULL,
//                                                 NULL,
//                                                 NULL );
//             }
//         }
//
//         //
//         //  Now if we may delete the file, drop the Fcb and acquire the Vcb
//         //  first.  Note that while we own the Fcb exclusive, a file cannot
//         //  become DELETE_ON_CLOSE and cannot be opened via CommonCreate.
//         //
//
//         if ((Fcb->UncleanCount == 1) &&
//             FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE) &&
//             (Fcb->FcbCondition != FcbBad) &&
//             !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//             FatReleaseFcb( IrpContext, Fcb );
//             AcquiredFcb = FALSE;
//
//             (VOID)FatAcquireExclusiveVcb( IrpContext, Vcb );
//             AcquiredVcb = TRUE;
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, Fcb );
//             AcquiredFcb = TRUE;
//         }
//     }
//
//     //
//     //  For user DASD cleanups, grab the Vcb exclusive.
//     //
//
//     if (TypeOfOpen == UserVolumeOpen) {
//
//         (VOID)FatAcquireExclusiveVcb( IrpContext, Vcb );
//         AcquiredVcb = TRUE;
//     }
//
//     //
//     //  Complete any Notify Irps on this file handle.
//     //
//
//     if (TypeOfOpen == UserDirectoryOpen) {
//
//         FsRtlNotifyCleanup( Vcb->NotifySync,
//                             &Vcb->DirNotifyList,
//                             Ccb );
//     }
//
//     //
//     //  Determine the Fcb state, Good or Bad, for better or for worse.
//     //
//     //  We can only read the volume file if VcbCondition is good.
//     //
//
//     if ( Fcb != NULL) {
//
//         //
//         //  Stop any raises from FatVerifyFcb, unless it is REAL bad.
//         //
//
//         _SEH2_TRY {
//
//             _SEH2_TRY {
//
//                 FatVerifyFcb( IrpContext, Fcb );
//
//             } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                       EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//         } _SEH2_FINALLY {
//
//             if ( _SEH2_AbnormalTermination() ) {
//
//                 //
//                 //  We will be raising out of here.
//                 //
//
//                 if (AcquiredFcb) { FatReleaseFcb( IrpContext, Fcb ); }
//                 if (AcquiredVcb) { FatReleaseVcb( IrpContext, Vcb ); }
//             }
//         } _SEH2_END;
//     }
//
//     _SEH2_TRY {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         //
//         //  See if this is a delete-on-close handle on a file or empty directory.
//         //  If so we may need to break an oplock.  We do this in the try block
//         //  so that resources will be properly released.
//         //
//
//         if (ProcessingDeleteOnClose &&
//             FatIsFileOplockable( Fcb ) &&
//             ((NodeType( Fcb ) != FAT_NTC_DCB) ||
//              FatIsDirectoryEmpty( IrpContext, Fcb ))) {
//
//             Status = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb),
//                                          Irp,
//                                          OPLOCK_FLAG_CLOSING_DELETE_ON_CLOSE,
//                                          IrpContext,
//                                          FatOplockComplete,
//                                          FatPrePostIrp );
//
//             if (Status != STATUS_SUCCESS) {
//
//                 if (Status == STATUS_PENDING) {
//
//                     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_CLEANUP_BREAKING_OPLOCK );
//                     try_return( Status );
//
//                 } else {
//
//                     FatNormalizeAndRaiseStatus( IrpContext, Status );
//                 }
//             }
//         }
// #endif
//
//         //
//         //  Case on the type of open that we are trying to cleanup.
//         //  For all cases we need to set the share access to point to the
//         //  share access variable (if there is one). After the switch
//         //  we then remove the share access and complete the Irp.
//         //  In the case of UserFileOpen we actually have a lot more work
//         //  to do and we have the FsdLockControl complete the Irp for us.
//         //
//
//         switch (TypeOfOpen) {
//
//         case DirectoryFile:
//         case VirtualVolumeFile:
//
//             DebugTrace(0, Dbg, "Cleanup VirtualVolumeFile/DirectoryFile\n", 0);
//
//             ShareAccess = NULL;
//
//             break;
//
//
//         case UserVolumeOpen:
//
//             DebugTrace(0, Dbg, "Cleanup UserVolumeOpen\n", 0);
//
//             if (FlagOn( Ccb->Flags, CCB_FLAG_COMPLETE_DISMOUNT )) {
//
//                 FatCheckForDismount( IrpContext, Vcb, TRUE );
//
//             //
//             //  If this handle had write access, and actually wrote something,
//             //  flush the device buffers, and then set the verify bit now
//             //  just to be safe (in case there is no dismount).
//             //
//
//             } else if (FileObject->WriteAccess &&
//                        FlagOn(FileObject->Flags, FO_FILE_MODIFIED)) {
//
//                 (VOID)FatHijackIrpAndFlushDevice( IrpContext,
//                                                   Irp,
//                                                   Vcb->TargetDeviceObject );
//
//                 SetFlag(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//             }
//
//             //
//             //  If the volume is locked by this file object then release
//             //  the volume and send notification.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_LOCKED) &&
//                 (Vcb->FileObjectWithVcbLocked == FileObject)) {
//
//                 FatAutoUnlock( IrpContext, Vcb );
//                 SendUnlockNotification = TRUE;
//             }
//
//             ShareAccess = &Vcb->ShareAccess;
//
//             break;
//
//         case EaFile:
//
//             DebugTrace(0, Dbg, "Cleanup EaFileObject\n", 0);
//
//             ShareAccess = NULL;
//
//             break;
//
//         case UserDirectoryOpen:
//
//             DebugTrace(0, Dbg, "Cleanup UserDirectoryOpen\n", 0);
//
//             ShareAccess = &Fcb->ShareAccess;
//
//             //
//             //  Determine here if we should try do delayed close.
//             //
//
//             if ((Fcb->UncleanCount == 1) &&
//                 (Fcb->OpenCount == 1) &&
//                 (Fcb->Specific.Dcb.DirectoryFileOpenCount == 0) &&
//                 !FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE) &&
//                 Fcb->FcbCondition == FcbGood) {
//
//                 //
//                 //  Delay our close.
//                 //
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_DELAY_CLOSE );
//             }
//
//             //
//             //  Clear the deny defrag bit, if the handle we're cleaning up was the one that set it.
//             //
//
//             if( FlagOn(Fcb->FcbState, FCB_STATE_DENY_DEFRAG) && FlagOn(Ccb->Flags, CCB_FLAG_DENY_DEFRAG) ) {
//
//                 ClearFlag(Ccb->Flags, CCB_FLAG_DENY_DEFRAG);
//                 ClearFlag(Fcb->FcbState, FCB_STATE_DENY_DEFRAG );
//             }
//
//             if ((VcbGood == Vcb->VcbCondition) &&
//                 !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN)) {
//
//                 FatUpdateDirentFromFcb( IrpContext, FileObject, Fcb, Ccb );
//
//                 //
//                 //  If the directory has a unclean count of 1 then we know
//                 //  that this is the last handle for the file object.  If
//                 //  we are supposed to delete it, do so.
//                 //
//
//                 if ((Fcb->UncleanCount == 1) &&
//                     (NodeType(Fcb) == FAT_NTC_DCB) &&
//                     (FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE)) &&
//                     (Fcb->FcbCondition == FcbGood) &&
//                     !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//                     if (!FatIsDirectoryEmpty(IrpContext, Fcb)) {
//
//                         //
//                         //  If there are files in the directory at this point,
//                         //  forget that we were trying to delete it.
//                         //
//
//                         ClearFlag( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//
//                     } else {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                         NTSTATUS BreakStatus;
// #endif
//
//                         //
//                         //  Even if something goes wrong, we cannot turn back!
//                         //
//
//                         _SEH2_TRY {
//
//                             DELETE_CONTEXT DeleteContext;
//
//
//                             //
//                             //  Before truncating file allocation remember this
//                             //  info for FatDeleteDirent.
//                             //
//
//                             DeleteContext.FileSize = Fcb->Header.FileSize.LowPart;
//                             DeleteContext.FirstClusterOfFile = Fcb->FirstClusterOfFile;
//
//                             //
//                             //  Synchronize here with paging IO
//                             //
//
//                             (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource,
//                                                               TRUE );
//
//                             Fcb->Header.FileSize.LowPart = 0;
//
//                             ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//
//                             //
//                             //  Truncate the file allocation down to zero
//                             //
//
//                             DebugTrace(0, Dbg, "Delete File allocation\n", 0);
//
//                             FatTruncateFileAllocation( IrpContext, Fcb, 0 );
//
//                             if (Fcb->Header.AllocationSize.LowPart == 0) {
//
//                                 //
//                                 //  Tunnel and remove the dirent for the directory
//                                 //
//
//                                 DebugTrace(0, Dbg, "Delete the directory dirent\n", 0);
//
//                                 FatTunnelFcbOrDcb( Fcb, NULL );
//
//                                 FatDeleteDirent( IrpContext, Fcb, &DeleteContext, TRUE );
//
//                                 //
//                                 //  Report that we have removed an entry.
//                                 //
//
//                                 FatNotifyReportChange( IrpContext,
//                                                        Vcb,
//                                                        Fcb,
//                                                        FILE_NOTIFY_CHANGE_DIR_NAME,
//                                                        FILE_ACTION_REMOVED );
//                             }
//
//                         } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                   EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                               FatResetExceptionState( IrpContext );
//                         } _SEH2_END;
//
//                         //
//                         //  Remove the entry from the name table.
//                         //  This will ensure that
//                         //  we will not collide with the Dcb if the user wants
//                         //  to recreate the same file over again before we
//                         //  get a close irp.
//                         //
//
//                         FatRemoveNames( IrpContext, Fcb );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                         //
//                         //  We've removed the names so break any parent directory oplock.
//                         //  Directory oplock breaks are always advisory, so we will never
//                         //  block/get STATUS_PENDING here.
//                         //
//
//                         BreakStatus = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb->ParentDcb),
//                                                           Irp,
//                                                           (OPLOCK_FLAG_PARENT_OBJECT |
//                                                            OPLOCK_FLAG_REMOVING_FILE_OR_LINK),
//                                                           NULL,
//                                                           NULL,
//                                                           NULL );
//
//                         ASSERT( BreakStatus != STATUS_PENDING );
// #endif
//                     }
//                 }
//             }
//
//             //
//             //  Decrement the unclean count.
//             //
//
//             NT_ASSERT( Fcb->UncleanCount != 0 );
//             Fcb->UncleanCount -= 1;
//
//             break;
//
//         case UserFileOpen:
//
//             DebugTrace(0, Dbg, "Cleanup UserFileOpen\n", 0);
//
//             ShareAccess = &Fcb->ShareAccess;
//
//             //
//             //  Determine here if we should do a delayed close.
//             //
//
//             if ((FileObject->SectionObjectPointer->DataSectionObject == NULL) &&
//                 (FileObject->SectionObjectPointer->ImageSectionObject == NULL) &&
//                 (Fcb->UncleanCount == 1) &&
//                 (Fcb->OpenCount == 1) &&
//                 !FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE) &&
//                 !FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE) &&
//                 Fcb->FcbCondition == FcbGood) {
//
//                 //
//                 //  Delay our close.
//                 //
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_DELAY_CLOSE );
//             }
//
//             //
//             //  Clear the deny defrag bit, if the handle we're cleaning up was the one that set it.
//             //
//
//             if( FlagOn(Fcb->FcbState, FCB_STATE_DENY_DEFRAG) && FlagOn(Ccb->Flags, CCB_FLAG_DENY_DEFRAG) ) {
//
//                 ClearFlag(Ccb->Flags, CCB_FLAG_DENY_DEFRAG);
//                 ClearFlag(Fcb->FcbState, FCB_STATE_DENY_DEFRAG );
//             }
//
//             //
//             //  Unlock all outstanding file locks.
//             //
//
//             (VOID) FsRtlFastUnlockAll( &Fcb->Specific.Fcb.FileLock,
//                                        FileObject,
//                                        IoGetRequestorProcess( Irp ),
//                                        NULL );
//
//
//
//             //
//             //  We can proceed with on-disk updates only if the volume is mounted
//             //  and we can still write to it if it hasn't been shutdown. Remember that
//             //  we toss all sections in the failed-verify and dismount cases.
//             //
//
//             if ((Vcb->VcbCondition == VcbGood) &&
//                 !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN)) {
//
//                 if (Fcb->FcbCondition == FcbGood) {
//
//
//                     FatUpdateDirentFromFcb( IrpContext, FileObject, Fcb, Ccb );
//
//                 }
//
//                 //
//                 //  If the file has a unclean count of 1 then we know
//                 //  that this is the last handle for the file object.
//                 //
//
//                 if ( (Fcb->UncleanCount == 1) && (Fcb->FcbCondition == FcbGood) ) {
//
//                     DELETE_CONTEXT DeleteContext;
//
//                     //
//                     //  Check if we should be deleting the file.  The
//                     //  delete operation really deletes the file but
//                     //  keeps the Fcb around for close to do away with.
//                     //
//
//                     if (FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE) &&
//                         !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//                         //
//                         //  Before truncating file allocation remember this
//                         //  info for FatDeleteDirent.
//                         //
//
//                         DeleteContext.FileSize = Fcb->Header.FileSize.LowPart;
//                         DeleteContext.FirstClusterOfFile = Fcb->FirstClusterOfFile;
//
//                         DebugTrace(0, Dbg, "Delete File allocation\n", 0);
//
//                         //
//                         //  Synchronize here with paging IO
//                         //
//
//                         (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource,
//                                                           TRUE );
//
//                         Fcb->Header.FileSize.LowPart = 0;
//                         Fcb->Header.ValidDataLength.LowPart = 0;
//                         Fcb->ValidDataToDisk = 0;
//
//                         ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//
//                         _SEH2_TRY {
//
//                             FatSetFileSizeInDirent( IrpContext, Fcb, NULL );
//
//                         } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                   EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                               FatResetExceptionState( IrpContext );
//                         } _SEH2_END;
//
//                         Fcb->FcbState |= FCB_STATE_TRUNCATE_ON_CLOSE;
//
//                     } else {
//
//                         //
//                         //  We must zero between ValidDataLength and FileSize
//                         //
//
//                         if (!FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE) &&
//                             (Fcb->Header.ValidDataLength.LowPart < Fcb->Header.FileSize.LowPart)) {
//
//                             ULONG ValidDataLength;
//
//                             ValidDataLength = Fcb->Header.ValidDataLength.LowPart;
//
//                             if (ValidDataLength < Fcb->ValidDataToDisk) {
//                                 ValidDataLength = Fcb->ValidDataToDisk;
//                             }
//
//                             //
//                             //  Recheck, VDD can be >= FS
//                             //
//
//                             if (ValidDataLength < Fcb->Header.FileSize.LowPart) {
//
//                                 _SEH2_TRY {
//
//                                     (VOID)FatZeroData( IrpContext,
//                                                        Vcb,
//                                                        FileObject,
//                                                        ValidDataLength,
//                                                        Fcb->Header.FileSize.LowPart -
//                                                        ValidDataLength );
//
//                                     //
//                                     //  Since we just zeroed this, we can now bump
//                                     //  up VDL in the Fcb.
//                                     //
//
//                                     Fcb->ValidDataToDisk =
//                                     Fcb->Header.ValidDataLength.LowPart =
//                                     Fcb->Header.FileSize.LowPart;
//
//                                     //
//                                     //  We inform Cc of the motion so that the cache map is updated.
//                                     //  This prevents optimized zero-page faults in case the cache
//                                     //  structures are re-used for another handle before they are torn
//                                     //  down by our soon-to-occur uninitialize. If they were, a noncached
//                                     //  producer could write into the region we just zeroed and Cc would
//                                     //  be none the wiser, then our async cached reader comes in and takes
//                                     //  the optimized path, and we get bad (zero) data.
//                                     //
//                                     //  If this was memory mapped, we don't have to (can't) tell Cc, it'll
//                                     //  figure it out when a cached handle is opened.
//                                     //
//
//                                     if (CcIsFileCached( FileObject )) {
//                                         CcSetFileSizes( FileObject, (PCC_FILE_SIZES)&Fcb->Header.AllocationSize );
//                                     }
//
//                                 } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                           EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                                       FatResetExceptionState( IrpContext );
//                                 } _SEH2_END;
//                             }
//                         }
//                     }
//
//                     //
//                     //  See if we are supposed to truncate the file on the last
//                     //  close.  If we cannot wait we'll ship this off to the fsp
//                     //
//
//                     _SEH2_TRY {
//
//                         if (FlagOn(Fcb->FcbState, FCB_STATE_TRUNCATE_ON_CLOSE)) {
//
//                             DebugTrace(0, Dbg, "truncate file allocation\n", 0);
//
//                             if (Vcb->VcbCondition == VcbGood) {
//
//
//                                 FatTruncateFileAllocation( IrpContext,
//                                                            Fcb,
//                                                            Fcb->Header.FileSize.LowPart );
//
//
//                             }
//
//                             //
//                             //  We also have to get rid of the Cache Map because
//                             //  this is the only way we have of trashing the
//                             //  truncated pages.
//                             //
//
//                             LocalTruncateSize = Fcb->Header.FileSize;
//                             TruncateSize = &LocalTruncateSize;
//
//                             //
//                             //  Mark the Fcb as having now been truncated, just incase
//                             //  we have to reship this off to the fsp.
//                             //
//
//                             Fcb->FcbState &= ~FCB_STATE_TRUNCATE_ON_CLOSE;
//                         }
//
//                         //
//                         //  Now check again if we are to delete the file and if
//                         //  so then we remove the file from the disk.
//                         //
//
//                         if (FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE) &&
//                             Fcb->Header.AllocationSize.LowPart == 0) {
//
//                             DebugTrace(0, Dbg, "Delete File\n", 0);
//
//                             //
//                             //  Now tunnel and delete the dirent
//                             //
//
//                             FatTunnelFcbOrDcb( Fcb, Ccb );
//
//                             FatDeleteDirent( IrpContext, Fcb, &DeleteContext, TRUE );
//
//                             //
//                             //  Report that we have removed an entry.
//                             //
//
//                             FatNotifyReportChange( IrpContext,
//                                                    Vcb,
//                                                    Fcb,
//                                                    FILE_NOTIFY_CHANGE_FILE_NAME,
//                                                    FILE_ACTION_REMOVED );
//                         }
//
//                     } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                           FatResetExceptionState( IrpContext );
//                     } _SEH2_END;
//
//                     if (FlagOn(Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE)) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                         NTSTATUS BreakStatus;
// #endif
//                         //
//                         //  Remove the entry from the splay table. This will
//                         //  ensure that we will not collide with the Fcb if the
//                         //  user wants to recreate the same file over again
//                         //  before we get a close irp.
//                         //
//                         //  Note that we remove the name even if we couldn't
//                         //  truncate the allocation and remove the dirent above.
//                         //
//
//                         FatRemoveNames( IrpContext, Fcb );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                         //
//                         //  We've removed the names so break any parent directory oplock.
//                         //  Directory oplock breaks are always advisory, so we will never
//                         //  block/get STATUS_PENDING here.
//                         //
//
//                         BreakStatus = FsRtlCheckOplockEx( FatGetFcbOplock(Fcb->ParentDcb),
//                                                           Irp,
//                                                           (OPLOCK_FLAG_PARENT_OBJECT |
//                                                            OPLOCK_FLAG_REMOVING_FILE_OR_LINK),
//                                                           NULL,
//                                                           NULL,
//                                                           NULL );
//
//                         ASSERT( BreakStatus != STATUS_PENDING );
// #endif
//                     }
//                 }
//             }
//
//             //
//             //  We've just finished everything associated with an unclean
//             //  fcb so now decrement the unclean count before releasing
//             //  the resource.
//             //
//
//             NT_ASSERT( Fcb->UncleanCount != 0 );
//             Fcb->UncleanCount -= 1;
//             if (!FlagOn( FileObject->Flags, FO_CACHE_SUPPORTED )) {
//                 NT_ASSERT( Fcb->NonCachedUncleanCount != 0 );
//                 Fcb->NonCachedUncleanCount -= 1;
//             }
//
//             //
//             //  If this was the last cached open, and there are open
//             //  non-cached handles, attempt a flush and purge operation
//             //  to avoid cache coherency overhead from these non-cached
//             //  handles later.  We ignore any I/O errors from the flush.
//             //
//
//             if (FlagOn( FileObject->Flags, FO_CACHE_SUPPORTED ) &&
//                 (Fcb->NonCachedUncleanCount != 0) &&
//                 (Fcb->NonCachedUncleanCount == Fcb->UncleanCount) &&
//                 (Fcb->NonPaged->SectionObjectPointers.DataSectionObject != NULL)) {
//
//                 CcFlushCache( &Fcb->NonPaged->SectionObjectPointers, NULL, 0, NULL );
//
//                 //
//                 //  Grab and release PagingIo to serialize ourselves with the lazy writer.
//                 //  This will work to ensure that all IO has completed on the cached
//                 //  data and we will succesfully tear away the cache section.
//                 //
//
//                 ExAcquireResourceExclusiveLite( Fcb->Header.PagingIoResource, TRUE);
//                 ExReleaseResourceLite( Fcb->Header.PagingIoResource );
//
//                 CcPurgeCacheSection( &Fcb->NonPaged->SectionObjectPointers,
//                                      NULL,
//                                      0,
//                                      FALSE );
//             }
//
//             //
//             //  If the file is invalid, hint to the cache that we should throw everything out.
//             //
//
//             if ( Fcb->FcbCondition == FcbBad ) {
//
//                 TruncateSize = &FatLargeZero;
//             }
//
//             //
//             //  Cleanup the cache map
//             //
//
//             CcUninitializeCacheMap( FileObject, TruncateSize, NULL );
//
//             break;
//
//         default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "if the type of open is unknown then things are very bad." )
// #endif
//             FatBugCheck( TypeOfOpen, 0, 0 );
//         }
//
//         //
//         //  We must clean up the share access at this time, since we may not
//         //  get a Close call for awhile if the file was mapped through this
//         //  File Object.
//         //
//
//         if (ShareAccess != NULL) {
//
//             DebugTrace(0, Dbg, "Cleanup the Share access\n", 0);
//             IoRemoveShareAccess( FileObject, ShareAccess );
//         }
//
//         if ((TypeOfOpen == UserFileOpen)
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             ||
//             (TypeOfOpen == UserDirectoryOpen)
// #endif
//             ) {
//
//             //
//             //  Coordinate the cleanup operation with the oplock state.
//             //  Cleanup operations can always cleanup immediately.
//             //
//
//             FsRtlCheckOplock( FatGetFcbOplock(Fcb),
//                               Irp,
//                               IrpContext,
//                               NULL,
//                               NULL );
//
//             Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//         }
//
//         //
//         //  First set the FO_CLEANUP_COMPLETE flag.
//         //
//
//         SetFlag( FileObject->Flags, FO_CLEANUP_COMPLETE );
//
//         Status = STATUS_SUCCESS;
//
//         //
//         //  Now unpin any repinned Bcbs.
//         //
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//         //
//         //  If this was deferred flush media, flush the volume.
//         //  We used to do this in lieu of write through for all removable
//         //  media.
//         //
//
//         if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH) &&
//             !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED))  {
//
//             //
//             //  Flush the file.
//             //
//
//             if ((TypeOfOpen == UserFileOpen) &&
//                 FlagOn(FileObject->Flags, FO_FILE_MODIFIED)) {
//
//                 Status = FatFlushFile( IrpContext, Fcb, Flush );
//             }
//
//             //
//             //  If that worked ok,  then see if we should flush the FAT as well.
//             //
//
//             if (NT_SUCCESS(Status) && Fcb && !FatIsFat12( Vcb) &&
//                 FlagOn( Fcb->FcbState, FCB_STATE_FLUSH_FAT)) {
//
//                 Status = FatFlushFat( IrpContext, Vcb);
//
//                 //
//                 // Also flush the parent directory.
//                 //
//
//                 if (NT_SUCCESS(Status) && (Fcb->ParentDcb != NULL)) {
//
//                     Status = FatFlushFile( IrpContext, Fcb->ParentDcb, Flush );
//                 }
//             }
//
//             if (!NT_SUCCESS(Status)) {
//
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     try_exit: NOTHING;
//
// #endif
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonCleanup );
//
//         if (AcquiredFcb) { FatReleaseFcb( IrpContext, Fcb ); }
//         if (AcquiredVcb) { FatReleaseVcb( IrpContext, Vcb ); }
//
//         if (SendUnlockNotification) {
//
//             FsRtlNotifyVolumeEvent( FileObject, FSRTL_VOLUME_UNLOCK );
//         }
//
//         //
//         //  If this is a normal termination then complete the request
//         //
//
//         if (!_SEH2_AbnormalTermination() &&
//             (Status != STATUS_PENDING)) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonCleanup -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
// VOID
// FatAutoUnlock (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
// {
//     KIRQL SavedIrql;
//
//     //
//     //  Unlock the volume.
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     ClearFlag( Vcb->Vpb->Flags, (VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED) );
//
//     Vcb->VcbState &= ~VCB_STATE_FLAG_LOCKED;
//     Vcb->FileObjectWithVcbLocked = NULL;
//
//     IoReleaseVpbSpinLock( SavedIrql );
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
