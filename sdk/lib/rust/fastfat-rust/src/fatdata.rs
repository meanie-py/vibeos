use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcMdlWriteAbort, ExLocalTimeToSystemTime, PFAST_IO_CHECK_IF_POSSIBLE, FAST_IO_DISPATCH, PFAST_IO_QUERY_BASIC_INFO, PFAST_IO_QUERY_NETWORK_OPEN_INFO, PFAST_IO_QUERY_STANDARD_INFO, FAST_MUTEX, FILE_BASIC_INFORMATION, FILE_NETWORK_OPEN_INFORMATION, FILE_STANDARD_INFORMATION, IoGetCurrentIrpStackLocation, IoGetTopLevelIrp, IoIsSystemThread, IoRaiseInformationalHardError, IoSetTopLevelIrp, KEVENT, KeAreAllApcsDisabled, LARGE_INTEGER, LOGICAL, NPAGED_LOOKASIDE_LIST, PDEVICE_OBJECT, PETHREAD, PEXCEPTION_POINTERS, PFILE_BASIC_INFORMATION, PFILE_NETWORK_OPEN_INFORMATION, PFILE_OBJECT, PFILE_STANDARD_INFORMATION, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PKTHREAD, PLARGE_INTEGER, PMDL, PVPB};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, FAT_BUG_CHECK_FATDATA, FAT_DATA, FAT_TIME_STAMP, FCB, FCB_STATE_DELETE_ON_CLOSE, IRP_CONTEXT, PCCB, PFCB, PIRP_CONTEXT, PVCB, VCB, _TYPE_OF_OPEN as TYPE_OF_OPEN};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, ERESOURCE, EXCEPTION_POINTERS, FILE_OBJECT, IO_STACK_LOCATION, IRP, KTHREAD, _IO_STATUS_BLOCK, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_CATCH_EXCEPTIONS, FAT_NTC_FCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem, FsRtlFastCheckLockForRead, FsRtlFastCheckLockForWrite, FsRtlIsNtstatusExpected};
use ntkrnl_sys::ddk::ExReleaseResourceLite;

extern "system" {
    fn FatBugCheck(arg1: usize, arg2: usize, arg3: usize);
    fn SetFlag(flags: *mut u32, mask: u32);
    fn ClearFlag(flags: *mut u32, mask: u32);
    fn FlagOn(flags: u32, mask: u32) -> i32;
    fn FatDecodeFileObject( FileObject: *mut FILE_OBJECT, Vcb: *mut PVCB, Fcb: *mut PFCB, Ccb: *mut PCCB, ) -> TYPE_OF_OPEN;
    fn NodeType(Node: *mut core::ffi::c_void) -> u32;
    fn FatSetFullFileNameInFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    static FAT_NTC_ROOT_DCB: i32;
    static STATUS_FILE_CORRUPT_ERROR: i32;
}
extern "system" {
    } extern "C" { fn DebugTrace(level: u32, component: u32, format: *const i8, ...); } extern "system" {
    fn DebugDump(str: *const i8, level: u32, p: *mut core::ffi::c_void);
    fn NT_ASSERT(condition: i32);
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, ExceptionCode: NTSTATUS);
    fn FatResetExceptionState(IrpContext: *mut IRP_CONTEXT);
    fn FatUnpinRepinnedBcbs(IrpContext: *mut IRP_CONTEXT);
    fn FatFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatDeleteIrpContext(IrpContext: PIRP_CONTEXT);
    fn ExAcquireResourceSharedLite(resource: *mut ERESOURCE, wait: i32) -> u8;
    // ExReleaseResourceLite via ntkrnl_sys::ddk (kernel FASTCALL export — §9.99)
    fn BooleanFlagOn(Flags: u32, Mask: u32) -> u8;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_FATDATA;
pub const Dbg: u32 = DEBUG_TRACE_CATCH_EXCEPTIONS;

#[no_mangle] pub static mut FatData: FAT_DATA = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatDiskFileSystemDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
#[no_mangle] pub static mut FatCdromFileSystemDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
#[no_mangle] pub static mut FatLargeZero: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
#[no_mangle] pub static mut FatMaxLarge: LARGE_INTEGER = LARGE_INTEGER {
    // MAXULONG | (MAXLONG as i64 << 32) = 0x7FFFFFFF_FFFFFFFF
    QuadPart: 0x7FFF_FFFF_FFFF_FFFFi64,
};
#[no_mangle] pub static mut Fat30Milliseconds: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: -300_000i64,
};
#[no_mangle] pub static mut Fat100Milliseconds: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: -300_000i64,
};
#[no_mangle] pub static mut FatOneDay: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0x0000_00c9_2a69_c000u64 as i64,
};
#[no_mangle] pub static mut FatJanOne1980: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0x01a8e79f_e1d58000u64 as i64,
};
#[no_mangle] pub static mut FatDecThirtyOne1979: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0x01a8e6d6_b76bc000u64 as i64,
};
// FatTimeJanOne1980 — changed from const to static for extern-C linkage
#[no_mangle] pub static FatTimeJanOne1980: FAT_TIME_STAMP = unsafe {
    core::mem::transmute::<[u16; 2], FAT_TIME_STAMP>([0x0000, 0x0021])
};
#[no_mangle] pub static mut FatMagic10000: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0xd1b71758_e219652cu64 as i64,
};
#[no_mangle] pub static mut FatMagic86400000: LARGE_INTEGER = LARGE_INTEGER {
    QuadPart: 0xc6d750eb_fa67b90eu64 as i64,
};
#[no_mangle] pub static mut FatFastIoDispatch: FAST_IO_DISPATCH = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatIrpContextLookasideList: NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatNonPagedFcbLookasideList: NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatEResourceLookasideList: NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatCloseContextSList: SLIST_HEADER = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatCloseQueueMutex: FAST_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatReserveMdl: PMDL = core::ptr::null_mut();
#[no_mangle] pub static mut FatReserveEvent: KEVENT = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut FatDiskAccountingEnabled: LOGICAL = 0;

#[cfg(feature = "FASTFATDBG")]
pub static mut FatDebugTraceLevel: i32 = 0x09;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatDebugTraceIndent: i32 = 0;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatFsdEntryCount: u32 = 0;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatFspEntryCount: u32 = 0;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatIoCallDriverCount: u32 = 0;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatPerformanceTimerLevel: i32 = 0x00;
#[cfg(feature = "FASTFATDBG")]
pub static mut FatTotalTicks: [u32; 32] = [0; 32];
#[cfg(feature = "FASTFATDBG")]
pub static mut FatNull: *mut core::ffi::c_void = core::ptr::null_mut();

// fatdata.c's `#if DBG` globals. Two things were wrong with the translation and
// together they made i386 unlinkable (`undefined symbol:
// _FatBreakOnInterestingIoCompletion`):
//
//  1. `#[cfg(debug_assertions)]` — which cargo's release profile never sets, so
//     these did not exist in ANY shipped configuration, while deviosup.rs
//     declares and reads `FatBreakOnInterestingIoCompletion` unconditionally
//     (:76 / :1811). The C has both sides under `#if DBG` and is compiled with
//     -DDBG=1, so in the C they are both present; gating only the definition
//     reproduced neither arrangement.
//  2. No `#[no_mangle]` — so even when the cfg WAS satisfied they exported a
//     mangled Rust symbol and the C-shaped `extern` reference still could not
//     find them. That is why enabling debug-assertions workspace-wide did not
//     make the link error go away.
//
// Unconditional + #[no_mangle] matches the C's effective build, and is inert:
// the check is `!(NT_SUCCESS(FatBreakOnInterestingIoCompletion) || ...)`, and
// with the default STATUS_SUCCESS the NT_SUCCESS arm is true, so it never
// breaks. fatdata.c is cohab-dead (CMakeLists.txt:54), so there is no C
// definition to collide with.
#[no_mangle]
pub static mut FatTestRaisedStatus: u8 = 0;
#[no_mangle]
pub static mut FatBreakOnInterestingIoCompletion: NTSTATUS = STATUS_SUCCESS;
#[no_mangle]
pub static mut FatBreakOnInterestingExceptionStatus: NTSTATUS = 0;
#[no_mangle]
pub static mut FatBreakOnInterestingIrpCompletion: NTSTATUS = 0;

// Forward declare unknown functions

// Constants – values from Windows headers
const IRP_MJ_WRITE: u32 = 4;
const IRP_MN_COMPLETE_MDL: u32 = 0x6; // iotypes.h:4437 IRP_MN_COMPLETE(4) | IRP_MN_MDL(2) (was 0x3 = MDL|DPC)
const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016_u32 as i32;
const APC_LEVEL: u32 = 1;
const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8_u32 as i32;
const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: u32 = 0x08;  // dummy value
const IRP_CONTEXT_FLAG_RECURSIVE_CALL: u32 = 0x10;          // dummy

const IRP_INPUT_OPERATION: u32 = 0x40;

/*++
Routine Description:

    This routine detects if an Irp is the Top level requestor, ie. if it os OK
    to do a verify or pop-up now.  If TRUE is returned, then no file system
    resources are held above us.

Arguments:

    Irp - Supplies the Irp being processed

    Status - Supplies the status to complete the Irp with

Return Value:

    None.

--*/

const UserFileOpen: i32 = crate::types::_TYPE_OF_OPEN_UserFileOpen as i32; // 2 — bindgen ground truth (was 0)
const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000; // placeholder value

// Constants (typical values, caller may override)
const FAT_NTC_IRP_CONTEXT: u32 = 0x0508;
const FCB_STATE_PAGING_FILE: u32 = 0x00000004;
const FCB_STATE_TEMPORARY: u32 = 0x20;
const FcbGood: u32 = 1;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x0002;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const UserDirectoryOpen: u32 = crate::types::_TYPE_OF_OPEN_UserDirectoryOpen; // 3 — bindgen ground truth (was 1)
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const STATUS_SUCCESS: i32 = 0;

// Fabricated forward-decls removed (LLM local-type-mirror anti-pattern). Real
// VCB/FCB/CCB/IRP_CONTEXT/TYPE_OF_OPEN come from crate::types; ERESOURCE/
// _IO_STATUS_BLOCK/FILE_BASIC_INFORMATION from ntkrnl_sys::ddk (wired by resolve_imports).

// Unknown external functions (the caller will deduplicate)
#[no_mangle]
pub unsafe extern "system" fn FatBugCheckExceptionFilter(
    ExceptionPointer: PEXCEPTION_POINTERS,
) -> u32 {
    // PAGED_CODE();
    if KeGetCurrentIrql() > (APC_LEVEL) as u8 {
        DbgBreakPoint();
    }

    FatBugCheck(
        (*ExceptionPointer).ExceptionRecord as usize,
        (*ExceptionPointer).ContextRecord as usize,
        (*(*ExceptionPointer).ExceptionRecord).ExceptionAddress as usize,
            );

    // return EXCEPTION_EXECUTE_HANDLER; // unreachable
    // Satisfy the return type in the unreachable path
    1
}
#[no_mangle]
pub unsafe extern "system" fn FatExceptionFilter(
    IrpContext: *mut IRP_CONTEXT,
    ExceptionPointer: *mut EXCEPTION_POINTERS,
) -> u32 {
    // Local extern declarations for functions not pre-resolved

    // Constants needed for the function
    const STATUS_IN_PAGE_ERROR: u32 = 0xC0000006;
    const STATUS_CANT_WAIT: u32 = 0xC00000D8;
    const STATUS_VERIFY_REQUIRED: u32 = 0x80000016;
    const EXCEPTION_EXECUTE_HANDLER: u32 = 1;
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // TODO: verify value  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: u32 = 0x08; // TODO: verify value
    const DEBUG_TRACE_UNWIND: u32 = 0x08; // TODO: verify value
    const Dbg: u32 = 0x00000002; // DEBUG_TRACE_CATCH_EXCEPTIONS; TODO: verify value

    let mut ExceptionCode: u32;
    // ExceptionCode = ExceptionPointer->ExceptionRecord->ExceptionCode
    let ExceptionRecord = (*ExceptionPointer).ExceptionRecord;
    ExceptionCode = ((*ExceptionRecord).ExceptionCode) as u32;

    DebugTrace(
        0,
        DEBUG_TRACE_UNWIND,
        c"FatExceptionFilter %X\n".as_ptr() as *const i8,
        ExceptionCode,
    );
    DebugDump(
        c"FatExceptionFilter\n".as_ptr() as *const i8,
        Dbg,
        core::ptr::null_mut(),
    );

    // #if DBG
    #[cfg(debug_assertions)]
    {
        if FatBreakOnInterestingExceptionStatus != 0
            && ExceptionCode == (FatBreakOnInterestingExceptionStatus) as u32
        {
            DbgBreakPoint();
        }
    }

    // if (ExceptionCode == STATUS_IN_PAGE_ERROR)
    if ExceptionCode == STATUS_IN_PAGE_ERROR {
        if (*ExceptionRecord).NumberParameters >= 3 {
            ExceptionCode = (*ExceptionRecord).ExceptionInformation[2] as u32;
        }
    }

    // if ( !ARGUMENT_PRESENT( IrpContext ) )
    if IrpContext.is_null() {
        if FsRtlIsNtstatusExpected(ExceptionCode as i32) == 0 {
            // Diagnostic: the code and PC of the exception this is about to bugcheck on
            // (i386 writable-root boot: FatFsdFlushBuffers, IrpContext still NULL).
            extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
            DbgPrint(
                c"[FASTFAT] unexpected exception %08lx at %p (no IrpContext), p0=%p p1=%p\n".as_ptr(),
                ExceptionCode,
                (*ExceptionRecord).ExceptionAddress,
                (*ExceptionRecord).ExceptionInformation[0] as *mut core::ffi::c_void,
                (*ExceptionRecord).ExceptionInformation[1] as *mut core::ffi::c_void,
            );
            FatBugCheck(
                ExceptionRecord as usize,
                (*ExceptionPointer).ContextRecord as usize,
                (*ExceptionRecord).ExceptionAddress as usize,
            );
        }
        return EXCEPTION_EXECUTE_HANDLER;
    }

    // SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)
    SetFlag(
        &mut (*IrpContext).Flags,
        IRP_CONTEXT_FLAG_WAIT,
    );

    if ExceptionCode != STATUS_CANT_WAIT
        && ExceptionCode != STATUS_VERIFY_REQUIRED
    {
        SetFlag(
            &mut (*IrpContext).Flags,
            IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH,
        );
    }

    if (*IrpContext).ExceptionStatus == 0 {
        if FsRtlIsNtstatusExpected(ExceptionCode as i32) != 0 {
            (*IrpContext).ExceptionStatus = (ExceptionCode) as i32;
            return EXCEPTION_EXECUTE_HANDLER;
        } else {
            FatBugCheck(
                ExceptionRecord as usize,
                (*ExceptionPointer).ContextRecord as usize,
                (*ExceptionRecord).ExceptionAddress as usize,
            );
        }
    } else {
        NT_ASSERT(
            ((*IrpContext).ExceptionStatus == (ExceptionCode) as i32) as i32,
        );
        NT_ASSERT(
            (FsRtlIsNtstatusExpected(ExceptionCode as i32) != 0) as i32,
        );
    }

    EXCEPTION_EXECUTE_HANDLER
}
#[no_mangle]
pub unsafe extern "system" fn FatProcessException(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    ExceptionCode: NTSTATUS,
) -> NTSTATUS {
    // Hand-translated from fatdata.c: the generated body stopped after the post
    // decision and never completed the Irp, performed the verify, raised the
    // popup, or marked a corrupt volume dirty. That was invisible while no
    // handler ever ran; with real SEH every caught raise comes through here.
    const STATUS_PENDING: NTSTATUS = 0x00000103;
    const STATUS_FILE_LOCK_CONFLICT: NTSTATUS = 0xC0000054u32 as i32;
    const STATUS_DRIVER_INTERNAL_ERROR: NTSTATUS = 0xC0000183u32 as i32;
    const STATUS_DISK_CORRUPT_ERROR: NTSTATUS = 0xC0000032u32 as i32;
    const STATUS_FILE_CORRUPT_ERROR_: NTSTATUS = 0xC0000102u32 as i32;
    const STATUS_EA_CORRUPT_ERROR: NTSTATUS = 0xC0000053u32 as i32;
    const STATUS_INVALID_EA_NAME: NTSTATUS = 0x80000013u32 as i32;
    const STATUS_EA_LIST_INCONSISTENT: NTSTATUS = 0x80000014u32 as i32;
    const STATUS_NO_EAS_ON_FILE: NTSTATUS = 0xC0000052u32 as i32;
    const FSRTL_CACHE_TOP_LEVEL_IRP: usize = 2;
    const VolumeDirty: i32 = 1;
    const VolumeDirtyWithSurfaceTest: i32 = 2;
    use crate::types::{_VCB_CONDITION_VcbGood as VcbGood, IRP_CONTEXT_FLAG_DISABLE_POPUPS,
        VCB_STATE_FLAG_MOUNTED_DIRTY};
    extern "system" {
        fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
        fn FatDeviceIsFatFsdo(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
        fn FatPerformVerify(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Device: PDEVICE_OBJECT) -> NTSTATUS;
        fn FatMarkVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, VolumeState: i32);
        fn FatAcquireExclusiveVcb_Real(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, NoOpCheck: BOOLEAN) -> BOOLEAN;
        fn FatReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    }
    use ntkrnl_sys::ddk::{IoGetDeviceToVerify, IoRaiseHardError, IoSetDeviceToVerify,
        PsGetCurrentThread};

    let mut TransitionState: i32 = VolumeDirty;
    let mut SavedFlags: u32 = 0;

    // PAGED_CODE();

    DebugTrace(0, Dbg as u32, b"FatProcessException\n\0".as_ptr() as *const i8, 0);

    // If there is not an irp context, we must have had insufficient resources.
    if IrpContext.is_null() {
        FatCompleteRequest(core::ptr::null_mut(), Irp, ExceptionCode);
        return ExceptionCode;
    }

    // Get the real exception status from IrpContext->ExceptionStatus, and reset it.
    let mut ExceptionCode = (*IrpContext).ExceptionStatus;
    FatResetExceptionState(IrpContext);

    // If this is an Mdl write request, then take care of the Mdl here so that
    // things get cleaned up properly.
    if (*IrpContext).MajorFunction as u32 == IRP_MJ_WRITE
        && ((*IrpContext).MinorFunction as u32 & IRP_MN_COMPLETE_MDL) == IRP_MN_COMPLETE_MDL
        && !(*Irp).MdlAddress.is_null()
    {
        let LocalIrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        CcMdlWriteAbort((*LocalIrpSp).FileObject, (*Irp).MdlAddress);
        (*Irp).MdlAddress = core::ptr::null_mut();
    }

    // If we are going to post the request we may have to lock down the user's
    // buffer, so do it under a try-except that fails the request if that fails.
    // Also unpin any repinned Bcbs, protected by the same filter.
    let ic = IrpContext;
    let p_code = core::ptr::addr_of_mut!(ExceptionCode);
    let p_saved = core::ptr::addr_of_mut!(SavedFlags);
    let caught = crate::seh::fat_try_except(core::ptr::addr_of!(ic), || {
        *p_saved = (*IrpContext).Flags;

        // Make sure we don't try to write through Bcbs
        (*IrpContext).Flags |= IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH;

        FatUnpinRepinnedBcbs(IrpContext);

        (*IrpContext).Flags = *p_saved;

        // If we will have to post the request, do it here. The last thing
        // FatPrePostIrp does is mark the Irp pending, so we must return PENDING.
        // A verify cannot be done at APC level: it waits for Io to complete.
        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_RECURSIVE_CALL) == 0
            && (((*p_code == STATUS_VERIFY_REQUIRED) && KeAreAllApcsDisabled() != 0)
                || (*p_code == STATUS_CANT_WAIT))
        {
            *p_code = FatFsdPostRequest(IrpContext, Irp);
        }
    });
    if caught.is_err() {
        ExceptionCode = (*IrpContext).ExceptionStatus;
        (*IrpContext).ExceptionStatus = 0;
        (*IrpContext).Flags = SavedFlags;
    }

    // If we posted the request, just return here.
    if ExceptionCode == STATUS_PENDING {
        return ExceptionCode;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = ExceptionCode;

    // If this request is not a "top-level" irp, just complete it.
    if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_RECURSIVE_CALL) != 0 {
        // With a cache operation above us, commute verify to a lock conflict so
        // it is retried instead of failing with an unaesthetic error.
        if IoGetTopLevelIrp() as usize == FSRTL_CACHE_TOP_LEVEL_IRP
            && ExceptionCode == STATUS_VERIFY_REQUIRED
        {
            ExceptionCode = STATUS_FILE_LOCK_CONFLICT;
        }

        FatCompleteRequest(IrpContext, Irp, ExceptionCode);
        return ExceptionCode;
    }

    if IoIsErrorUserInduced(ExceptionCode) != 0 {
        // Error conditions that can be caused by, and possibly resolved by, the user.
        if ExceptionCode == STATUS_VERIFY_REQUIRED {
            DebugTrace(0, Dbg as u32, b"Perform Verify Operation\n\0".as_ptr() as *const i8, 0);

            // We are at the top level file system entry point: clear the device to
            // verify out of the thread local storage.
            let mut Device = IoGetDeviceToVerify((*Irp).Tail.Overlay.Thread);
            IoSetDeviceToVerify((*Irp).Tail.Overlay.Thread, core::ptr::null_mut());

            if Device.is_null() {
                Device = IoGetDeviceToVerify(PsGetCurrentThread());
                IoSetDeviceToVerify(PsGetCurrentThread(), core::ptr::null_mut());
            }

            // Some storage drivers set invalid non-NULL devices to verify, so the
            // C ignores the thread's and uses our real device object instead.
            if !(*IrpContext).Vcb.is_null() {
                Device = (*(*(*IrpContext).Vcb).Vpb).RealDevice;
            } else {
                // For FSCTLs IrpContext->Vcb may not be populated.
                Device = (*IrpContext).RealDevice;
            }

            if Device.is_null() {
                ExceptionCode = STATUS_DRIVER_INTERNAL_ERROR;
                FatCompleteRequest(IrpContext, Irp, ExceptionCode);
                return ExceptionCode;
            }

            // FatPerformVerify() will do the right thing with the Irp.
            return FatPerformVerify(IrpContext, Irp, Device);
        }

        // The other user induced conditions generate an error unless they have
        // been disabled for this request.
        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_DISABLE_POPUPS) != 0 {
            FatCompleteRequest(IrpContext, Irp, ExceptionCode);
            return ExceptionCode;
        }

        // Generate a pop-up.
        let IrpFileObject = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;
        let Vpb: PVPB = if !IrpFileObject.is_null() {
            (*IrpFileObject).Vpb
        } else {
            core::ptr::null_mut()
        };

        // The device to verify is either in my thread local storage or that of
        // the thread that owns the Irp.
        let mut Thread: PETHREAD = (*Irp).Tail.Overlay.Thread;
        let mut RealDevice = IoGetDeviceToVerify(Thread);

        if RealDevice.is_null() {
            Thread = PsGetCurrentThread();
            RealDevice = IoGetDeviceToVerify(Thread);
        }

        if !(*IrpContext).Vcb.is_null() {
            RealDevice = (*(*(*IrpContext).Vcb).Vpb).RealDevice;
        } else {
            RealDevice = (*IrpContext).RealDevice;
        }

        if RealDevice.is_null() {
            FatCompleteRequest(IrpContext, Irp, ExceptionCode);
            return ExceptionCode;
        }

        // IoRaiseHardError usually queues an APC to the caller's thread but may
        // complete the request immediately, so the Irp must be marked pending first.
        ntkrnl_sys::inline::tier1::IoMarkIrpPending(Irp);
        IoRaiseHardError(Irp, Vpb, RealDevice);

        // We are handing control back to the caller: reset the saved device.
        IoSetDeviceToVerify(Thread, core::ptr::null_mut());

        // Io completes or resubmits the Irp; either way the IrpContext is ours.
        FatDeleteIrpContext(IrpContext);
        return STATUS_PENDING;
    }

    // A run of the mill error.
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Vcb: *mut VCB = (*IrpContext).Vcb;

    // An unknown Vcb means the error was raised during a mount, before a full Vcb
    // existed, and was really handled there.
    if !Vcb.is_null() {
        if FatDeviceIsFatFsdo((*IrpSp).DeviceObject) == 0
            && ExceptionCode < 0
            && crate::ddk_shims::FsRtlIsTotalDeviceFailure(ExceptionCode) == 0
        {
            TransitionState = VolumeDirtyWithSurfaceTest;
        }

        // A STATUS_FILE_CORRUPT or similar error means nastiness out on the media:
        // mark the volume permanently dirty.
        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_DISABLE_POPUPS) == 0
            && (TransitionState == VolumeDirtyWithSurfaceTest
                || ExceptionCode == STATUS_FILE_CORRUPT_ERROR_
                || ExceptionCode == STATUS_DISK_CORRUPT_ERROR
                || ExceptionCode == STATUS_EA_CORRUPT_ERROR
                || ExceptionCode == STATUS_INVALID_EA_NAME
                || ExceptionCode == STATUS_EA_LIST_INCONSISTENT
                || ExceptionCode == STATUS_NO_EAS_ON_FILE)
        {
            (*Vcb).VcbState |= VCB_STATE_FLAG_MOUNTED_DIRTY;
            (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

            // Do the "dirty" work, ignoring any error. Take the Vcb to synchronize
            // against the verify path tearing things down, since we dropped all
            // synchronization when backing up due to the exception.
            FatAcquireExclusiveVcb_Real(IrpContext, Vcb, 1);

            let _ = crate::seh::fat_try_except(core::ptr::addr_of!(ic), || {
                if (*Vcb).VcbCondition == VcbGood {
                    FatMarkVolume(IrpContext, Vcb, TransitionState);
                }
            });

            FatReleaseVcb(IrpContext, Vcb);
        }
    }

    FatCompleteRequest(IrpContext, Irp, ExceptionCode);

    ExceptionCode
}

#[no_mangle]
pub unsafe extern "system" fn FatCompleteRequest_Real(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    Status: NTSTATUS
) {
    // PAGED_CODE();

    #[cfg(debug_assertions)] {
        if FatBreakOnInterestingIrpCompletion != 0 && Status == FatBreakOnInterestingIrpCompletion {
            DbgBreakPoint();
        }
    }

    if !IrpContext.is_null() {
        if !(*IrpContext).Repinned.Bcb[0].is_null() { DbgBreakPoint(); }
        FatUnpinRepinnedBcbs(IrpContext);
    }

    if !IrpContext.is_null() {
        FatDeleteIrpContext(IrpContext);
    }

    if !Irp.is_null() {
        if (Status < 0) && ((*Irp).Flags & IRP_INPUT_OPERATION) != 0 {
            (*Irp).IoStatus.Information = 0;
        }
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    }

    return;
}
#[no_mangle]
pub unsafe extern "system" fn FatIsIrpTopLevel(Irp: PIRP) -> i32 {
    // PAGED_CODE();
    if IoGetTopLevelIrp().is_null() {
        IoSetTopLevelIrp(Irp);
        return 1;
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatFastIoCheckIfPossible(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    Wait: u8,
    LockKey: u32,
    CheckForReadOperation: u8,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> u8 {
    // PAGED_CODE();
    let _ = DeviceObject;
    let _ = IoStatus;
    let _ = Wait;

    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut LargeLength: LARGE_INTEGER = core::mem::zeroed();

    if FatDecodeFileObject(
        FileObject,
        core::ptr::addr_of_mut!(Vcb),
        core::ptr::addr_of_mut!(Fcb),
        core::ptr::addr_of_mut!(Ccb),
    ) != (UserFileOpen) as u32 {
        return 0u8;
    }

    LargeLength.QuadPart = Length as i64;

    if CheckForReadOperation != 0 {
        if FsRtlFastCheckLockForRead(
            &raw mut (*Fcb).Specific.Fcb.as_mut().FileLock as *mut _,
            FileOffset,
            core::ptr::addr_of_mut!(LargeLength),
            LockKey,
            FileObject,
            (PsGetCurrentProcess()).cast(),
        ) != 0 {
            return 1u8;
        }
    } else {
        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) == 0
            && FsRtlFastCheckLockForWrite(
                &raw mut (*Fcb).Specific.Fcb.as_mut().FileLock as *mut _,
                FileOffset,
                core::ptr::addr_of_mut!(LargeLength),
                LockKey,
                FileObject,
                (PsGetCurrentProcess()).cast(),
            ) != 0
        {
            return 1u8;
        }
    }

    0u8
}


#[no_mangle]
pub unsafe extern "system" fn FatFastQueryBasicInfo(
    file_object: *mut FILE_OBJECT,
    wait: i32,
    buffer: *mut FILE_BASIC_INFORMATION,
    io_status: *mut _IO_STATUS_BLOCK,
    device_object: *mut DEVICE_OBJECT,
) -> i32 {
    let mut results: i32 = 0;
    let mut irp_context: IRP_CONTEXT = core::mem::zeroed();
    let mut type_of_open: TYPE_OF_OPEN;
    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();
    let mut fcb_acquired: i32 = 0;

    // PAGED_CODE(); // no-op
    // UNREFERENCED_PARAMETER(DeviceObject);
    let _ = device_object;

    // RtlZeroMemory(&IrpContext, sizeof(IRP_CONTEXT));
    RtlZeroMemory(
        &mut irp_context as *mut IRP_CONTEXT as *mut core::ffi::c_void,
        core::mem::size_of::<IRP_CONTEXT>(),
    );
    irp_context.NodeTypeCode = (FAT_NTC_IRP_CONTEXT) as u16;
    irp_context.NodeByteSize = (core::mem::size_of::<IRP_CONTEXT>() as u32) as i16;

    if wait != 0 {
        SetFlag(&mut irp_context.Flags, IRP_CONTEXT_FLAG_WAIT);
    } else {
        ClearFlag(&mut irp_context.Flags, IRP_CONTEXT_FLAG_WAIT);
    }

    type_of_open = FatDecodeFileObject(file_object, &mut vcb, &mut fcb, &mut ccb);

    if type_of_open != (UserFileOpen) as u32 && type_of_open != UserDirectoryOpen {
        return results;
    }

    FsRtlEnterFileSystem();

    if FlagOn((*fcb).FcbState, FCB_STATE_PAGING_FILE) == 0 {
        if ExAcquireResourceSharedLite((*fcb).Header.Resource, wait) == 0 {
            FsRtlExitFileSystem();
            return results;
        }
        fcb_acquired = 1;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_results = core::ptr::addr_of_mut!(results);
    crate::seh::try_finally(&mut (), |_| {
        if (*fcb).FcbCondition != FcbGood {
            // try_return(Results); // Results is FALSE at this point
            return;
        }

        (*buffer).FileAttributes = 0;

        if NodeType((fcb).cast()) != (FAT_NTC_ROOT_DCB) as u32 {
            (*buffer).LastWriteTime = (*fcb).LastWriteTime;
            (*buffer).CreationTime = (*fcb).CreationTime;
            (*buffer).LastAccessTime = (*fcb).LastAccessTime;
            (*buffer).ChangeTime.QuadPart = 0;
            (*buffer).FileAttributes = ((*fcb).DirentFatFlags) as u32;
        } else {
            (*buffer).LastWriteTime.QuadPart = 0;
            (*buffer).CreationTime.QuadPart = 0;
            (*buffer).LastAccessTime.QuadPart = 0;
            (*buffer).ChangeTime.QuadPart = 0;
            (*buffer).FileAttributes = FILE_ATTRIBUTE_DIRECTORY;
        }

        if FlagOn((*fcb).FcbState, FCB_STATE_TEMPORARY) != 0 {
            SetFlag(&mut (*buffer).FileAttributes, FILE_ATTRIBUTE_TEMPORARY);
        }

        if (*buffer).FileAttributes == 0 {
            (*buffer).FileAttributes = FILE_ATTRIBUTE_NORMAL;
        }

        (*io_status).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*io_status).Information = core::mem::size_of::<FILE_BASIC_INFORMATION>() as _;

        *p_results = 1;
    }, |_, _abnormal| {
        // _SEH2_FINALLY
        if fcb_acquired != 0 {
            ExReleaseResourceLite((*fcb).Header.Resource);
        }
        FsRtlExitFileSystem();
    });

    results
}

#[no_mangle]
pub unsafe extern "system" fn FatFastQueryStdInfo(
    FileObject: PFILE_OBJECT,
    Wait: u8,
    Buffer: PFILE_STANDARD_INFORMATION,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> u8 {
    let mut Results: u8 = 0;
    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    // Irp context dummy
    let mut IrpContext: IRP_CONTEXT = core::mem::zeroed();
    IrpContext.NodeTypeCode = (FAT_NTC_IRP_CONTEXT) as u16;
    IrpContext.NodeByteSize = (core::mem::size_of::<IRP_CONTEXT>() as u16) as i16;

    if Wait != 0 {
        SetFlag(&mut IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
    } else {
        ClearFlag(&mut IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
    }

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let TypeOfOpen: u32 = FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb);

    if TypeOfOpen != (UserFileOpen) as u32 && TypeOfOpen != UserDirectoryOpen {
        return Results;
    }

    FsRtlEnterFileSystem();

    let fcb_acquired: u8;
    if FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0 {
        if ExAcquireResourceSharedLite((*Fcb).Header.Resource, (Wait) as i32) == 0 {
            FsRtlExitFileSystem();
            return Results;
        }
        fcb_acquired = 1;
    } else {
        fcb_acquired = 0;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_results = core::ptr::addr_of_mut!(Results);
    crate::seh::try_finally(&mut (), |_| {
        if (*Fcb).FcbCondition != FcbGood {
            // try_return(Results) – results already false
            return;
        }

        (*Buffer).NumberOfLinks = 1;
        (*Buffer).DeletePending = BooleanFlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE);

        if NodeType(Fcb as *mut core::ffi::c_void) == FAT_NTC_FCB {
            // FCB_LOOKUP_ALLOCATIONSIZE_HINT is ((LONGLONG) -1), not MAXLONGLONG.
            if (*Fcb).Header.AllocationSize.QuadPart == -1_i64 {
                // try_return(Results)
                return;
            }
            (*Buffer).AllocationSize = (*Fcb).Header.AllocationSize;
            (*Buffer).EndOfFile = (*Fcb).Header.FileSize;
            (*Buffer).Directory = 0;
        } else {
            (*Buffer).AllocationSize = FatLargeZero;
            (*Buffer).EndOfFile = FatLargeZero;
            (*Buffer).Directory = 1;
        }

        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _;
        *p_results = 1;
    }, |_, _abnormal| {
        // _SEH2_FINALLY
        if fcb_acquired != 0 {
            ExReleaseResourceLite((*Fcb).Header.Resource);
        }
        FsRtlExitFileSystem();
    });

    Results
}


#[no_mangle]
pub unsafe extern "system" fn FatFastQueryNetworkOpenInfo(
    FileObject: *mut FILE_OBJECT,
    Wait: i32,
    Buffer: *mut FILE_NETWORK_OPEN_INFORMATION,
    IoStatus: *mut _IO_STATUS_BLOCK,
    DeviceObject: *mut DEVICE_OBJECT,
) -> i32 {
    // Local constants for missing FAT driver definitions
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const FAT_NTC_IRP_CONTEXT: u32 = 0x0508;
    const FCB_STATE_PAGING_FILE: u32 = 0x04;
    const FCB_STATE_TEMPORARY: u32 = 0x20;
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
    const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;
    const FAT_NTC_FCB: u32 = 0x0502;
    const FAT_NTC_ROOT_DCB: u32 = 0x0504;
    const FcbGood: u32 = 1;

    let mut Results: i32 = 0; // FALSE
    let mut IrpContext: IRP_CONTEXT = core::mem::zeroed();
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let mut TypeOfOpen: TYPE_OF_OPEN = 0;
    let mut FcbAcquired: i32 = 0;

    // PAGED_CODE()
    PAGED_CODE!();

    // UNREFERENCED_PARAMETER(DeviceObject)
    let _ = DeviceObject;

    // RtlZeroMemory on IrpContext
    core::ptr::write_bytes(
        &mut IrpContext as *mut IRP_CONTEXT as *mut u8,
        0u8,
        core::mem::size_of::<IRP_CONTEXT>(),
    );
    IrpContext.NodeTypeCode = (FAT_NTC_IRP_CONTEXT) as u16;
    IrpContext.NodeByteSize = (core::mem::size_of::<IRP_CONTEXT>()) as i16;

    if Wait != 0 {
        SetFlag(&mut IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
    } else {
        ClearFlag(&mut IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
    }

    TypeOfOpen = FatDecodeFileObject(FileObject, &mut Vcb, &mut Fcb, &mut Ccb);
    if (TypeOfOpen != (UserFileOpen) as u32) && (TypeOfOpen != UserDirectoryOpen) {
        return Results;
    }

    FsRtlEnterFileSystem();

    // Acquire Fcb only if not paging file
    if FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) == 0 {
        if ExAcquireResourceSharedLite((*Fcb).Header.Resource, Wait) == 0 {
            FsRtlExitFileSystem();
            return Results;
        }
        FcbAcquired = 1;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_results = core::ptr::addr_of_mut!(Results);
    crate::seh::try_finally(&mut (), |_| {
        if (*Fcb).FcbCondition != FcbGood {
            return; // try_return(Results)
        }

        // ChangeTime default
        ExLocalTimeToSystemTime(
            core::ptr::addr_of_mut!(FatJanOne1980),
            &mut (*Buffer).ChangeTime as *mut LARGE_INTEGER,
        );

        (*Buffer).FileAttributes = ((*Fcb).DirentFatFlags) as u32;

        if (*Fcb).Header.NodeTypeCode == (FAT_NTC_ROOT_DCB) as i16 {
            (*Buffer).CreationTime = (*Buffer).ChangeTime;
            (*Buffer).LastAccessTime = (*Buffer).ChangeTime;
            (*Buffer).LastWriteTime = (*Buffer).ChangeTime;
        } else {
            (*Buffer).LastWriteTime = (*Fcb).LastWriteTime;
            (*Buffer).CreationTime = (*Fcb).CreationTime;
            (*Buffer).LastAccessTime = (*Fcb).LastAccessTime;
        }

        // Temporary attribute
        if FlagOn((*Fcb).FcbState, FCB_STATE_TEMPORARY) != 0 {
            SetFlag(&mut (*Buffer).FileAttributes, FILE_ATTRIBUTE_TEMPORARY);
        }

        if (*Buffer).FileAttributes == 0 {
            (*Buffer).FileAttributes = FILE_ATTRIBUTE_NORMAL;
        }

        if NodeType(Fcb as *mut core::ffi::c_void) == FAT_NTC_FCB {
            if (*Fcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
                return; // try_return(Results)
            }
            (*Buffer).AllocationSize = (*Fcb).Header.AllocationSize;
            (*Buffer).EndOfFile = (*Fcb).Header.FileSize;
        } else {
            (*Buffer).AllocationSize = FatLargeZero;
            (*Buffer).EndOfFile = FatLargeZero;
        }

        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = core::mem::size_of::<FILE_NETWORK_OPEN_INFORMATION>() as _;
        *p_results = 1; // TRUE
    }, |_, _abnormal| {
        // _SEH2_FINALLY:
        if FcbAcquired != 0 {
            ExReleaseResourceLite((*Fcb).Header.Resource);
        }
        FsRtlExitFileSystem();
    });

    Results
}

#[no_mangle]
pub unsafe extern "system" fn FatPopUpFileCorrupt(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    PAGED_CODE!();

    // Check if this is the root directory; if so, skip popup
    if NodeType((Fcb).cast()) == (FAT_NTC_ROOT_DCB) as u32 {
        return;
    }

    // Ensure the full file name is cached
    if (*Fcb).FullFileName.Buffer.is_null() {
        FatSetFullFileNameInFcb(IrpContext, Fcb);
    }

    // Determine the thread to raise the error to.
    // We never want to block a system thread.
    let Thread: *mut KTHREAD;
    if IoIsSystemThread(
        (*(*IrpContext).OriginatingIrp).Tail.Overlay.Thread
    ) != 0 {
        Thread = core::ptr::null_mut();
    } else {
        {
            Thread = (*(*IrpContext).OriginatingIrp).Tail.Overlay.Thread as *mut KTHREAD;
        }
    }

    // Raise the informational hard error with STATUS_FILE_CORRUPT_ERROR
    IoRaiseInformationalHardError(
        STATUS_FILE_CORRUPT_ERROR,
        core::ptr::addr_of_mut!((*Fcb).FullFileName).cast(),
        Thread,
    );
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
//     FatData.c
//
// Abstract:
//
//     This module declares the global data used by the Fat file system.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_FATDATA)
//
// //
// //  The debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_CATCH_EXCEPTIONS)
//
// #ifdef ALLOC_PRAGMA
//
// #if DBG
// #ifdef _MSC_VER
// #pragma alloc_text(PAGE, FatBugCheckExceptionFilter)
// #endif
// #endif
//
// #pragma alloc_text(PAGE, FatCompleteRequest_Real)
// #pragma alloc_text(PAGE, FatFastIoCheckIfPossible)
// #pragma alloc_text(PAGE, FatFastQueryBasicInfo)
// #pragma alloc_text(PAGE, FatFastQueryNetworkOpenInfo)
// #pragma alloc_text(PAGE, FatFastQueryStdInfo)
// #pragma alloc_text(PAGE, FatIsIrpTopLevel)
// #pragma alloc_text(PAGE, FatPopUpFileCorrupt)
// #pragma alloc_text(PAGE, FatProcessException)
// #endif
//
//
// //
// //  The global fsd data record, and zero large integer
// //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:22112, "only applies to user mode processes" )
// #endif
// FAT_DATA FatData;
//
// PDEVICE_OBJECT FatDiskFileSystemDeviceObject;
// PDEVICE_OBJECT FatCdromFileSystemDeviceObject;
//
// #ifndef __REACTOS__
// LARGE_INTEGER FatLargeZero = {0,0};
// LARGE_INTEGER FatMaxLarge = {MAXULONG,MAXLONG};
// #else
// LARGE_INTEGER FatLargeZero = {{0,0}};
// LARGE_INTEGER FatMaxLarge = {{MAXULONG,MAXLONG}};
// #endif
//
// #ifndef __REACTOS__
// LARGE_INTEGER Fat30Milliseconds = {(ULONG)(-30 * 1000 * 10), -1};
// LARGE_INTEGER Fat100Milliseconds = {(ULONG)(-30 * 1000 * 10), -1};
// LARGE_INTEGER FatOneDay = {0x2a69c000, 0xc9};
// LARGE_INTEGER FatJanOne1980 = {0xe1d58000,0x01a8e79f};
// LARGE_INTEGER FatDecThirtyOne1979 = {0xb76bc000,0x01a8e6d6};
// #else
// LARGE_INTEGER Fat30Milliseconds = {{(ULONG)(-30 * 1000 * 10), -1}};
// LARGE_INTEGER Fat100Milliseconds = {{(ULONG)(-30 * 1000 * 10), -1}};
// LARGE_INTEGER FatOneDay = {{0x2a69c000, 0xc9}};
// LARGE_INTEGER FatJanOne1980 = {{0xe1d58000,0x01a8e79f}};
// LARGE_INTEGER FatDecThirtyOne1979 = {{0xb76bc000,0x01a8e6d6}};
// #endif
//
// FAT_TIME_STAMP FatTimeJanOne1980 = {{0,0,0},{1,1,0}};
//
// #ifndef __REACTOS__
// LARGE_INTEGER FatMagic10000    = {0xe219652c, 0xd1b71758};
// LARGE_INTEGER FatMagic86400000 = {0xfa67b90e, 0xc6d750eb};
// #else
// LARGE_INTEGER FatMagic10000    = {{0xe219652c, 0xd1b71758}};
// LARGE_INTEGER FatMagic86400000 = {{0xfa67b90e, 0xc6d750eb}};
// #endif
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:22112, "only applies to user mode processes" )
// #endif
// FAST_IO_DISPATCH FatFastIoDispatch;
//
// //
// //  Our lookaside lists.
// //
//
// NPAGED_LOOKASIDE_LIST FatIrpContextLookasideList;
// NPAGED_LOOKASIDE_LIST FatNonPagedFcbLookasideList;
// NPAGED_LOOKASIDE_LIST FatEResourceLookasideList;
//
// SLIST_HEADER FatCloseContextSList;
//
// //
// //  Synchronization for the close queue
// //
//
// FAST_MUTEX FatCloseQueueMutex;
//
// //
// //  Reserve MDL for paging file operations.
// //
//
// #ifndef __REACTOS__
// PMDL FatReserveMdl = NULL;
// #else
// volatile PMDL FatReserveMdl = NULL;
// #endif
// KEVENT FatReserveEvent;
//
// //
// //  Global disk accounting state, enabled or disabled
// //
//
// LOGICAL FatDiskAccountingEnabled = FALSE;
//
//
// #ifdef FASTFATDBG
//
// LONG FatDebugTraceLevel = 0x00000009;
// LONG FatDebugTraceIndent = 0;
//
// ULONG FatFsdEntryCount = 0;
// ULONG FatFspEntryCount = 0;
// ULONG FatIoCallDriverCount = 0;
//
// LONG FatPerformanceTimerLevel = 0x00000000;
//
// ULONG FatTotalTicks[32] = { 0 };
//
// //
// //  I need this because C can't support conditional compilation within
// //  a macro.
// //
//
// PVOID FatNull = NULL;
//
// #endif // FASTFATDBG
//
// #if DBG
//
// BOOLEAN FatTestRaisedStatus = FALSE;
//
// NTSTATUS FatBreakOnInterestingIoCompletion = STATUS_SUCCESS;
// NTSTATUS FatBreakOnInterestingExceptionStatus = 0;
// NTSTATUS FatBreakOnInterestingIrpCompletion = 0;
//
// #endif
//
//
// #if DBG
// ULONG
// FatBugCheckExceptionFilter (
//     IN PEXCEPTION_POINTERS ExceptionPointer
//     )
//
// /*++
//
// Routine Description:
//
//     An exception filter which acts as an assert that the exception should
//     never occur.
//
//     This is only valid on debug builds, we don't want the overhead on retail.
//
// Arguments:
//
//     ExceptionPointers - The result of GetExceptionInformation() in the context
//         of the exception.
//
// Return Value:
//
//     Bugchecks.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     FatBugCheck( (ULONG_PTR)ExceptionPointer->ExceptionRecord,
//                  (ULONG_PTR)ExceptionPointer->ContextRecord,
//                  (ULONG_PTR)ExceptionPointer->ExceptionRecord->ExceptionAddress );
//
// //    return EXCEPTION_EXECUTE_HANDLER; // unreachable code
// }
// #endif
//
//
// ULONG
// FatExceptionFilter (
//     IN PIRP_CONTEXT IrpContext,
//     IN PEXCEPTION_POINTERS ExceptionPointer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to decide if we should or should not handle
//     an exception status that is being raised.  It inserts the status
//     into the IrpContext and either indicates that we should handle
//     the exception or bug check the system.
//
// Arguments:
//
//     ExceptionPointers - The result of GetExceptionInformation() in the context
//         of the exception.
//
// Return Value:
//
//     ULONG - returns EXCEPTION_EXECUTE_HANDLER or bugchecks
//
// --*/
//
// {
//     NTSTATUS ExceptionCode;
//
//     ExceptionCode = ExceptionPointer->ExceptionRecord->ExceptionCode;
//     DebugTrace(0, DEBUG_TRACE_UNWIND, "FatExceptionFilter %X\n", ExceptionCode);
//     DebugDump("FatExceptionFilter\n", Dbg, NULL );
//
// #if DBG
//
//     if( FatBreakOnInterestingExceptionStatus != 0 && ExceptionCode == FatBreakOnInterestingExceptionStatus ) {
//         DbgBreakPoint();
//     }
//
// #endif
//
//     //
//     // If the exception is STATUS_IN_PAGE_ERROR, get the I/O error code
//     // from the exception record.
//     //
//
//     if (ExceptionCode == STATUS_IN_PAGE_ERROR) {
//         if (ExceptionPointer->ExceptionRecord->NumberParameters >= 3) {
//             ExceptionCode = (NTSTATUS)ExceptionPointer->ExceptionRecord->ExceptionInformation[2];
//         }
//     }
//
//     //
//     //  If there is not an irp context, we must have had insufficient resources.
//     //
//
//     if ( !ARGUMENT_PRESENT( IrpContext ) ) {
//
//         if (!FsRtlIsNtstatusExpected( ExceptionCode )) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( (ULONG_PTR)ExceptionPointer->ExceptionRecord,
//                          (ULONG_PTR)ExceptionPointer->ContextRecord,
//                          (ULONG_PTR)ExceptionPointer->ExceptionRecord->ExceptionAddress );
//         }
//
//         return EXCEPTION_EXECUTE_HANDLER;
//     }
//
//     //
//     //  For the purposes of processing this exception, let's mark this
//     //  request as being able to wait and disable  write through if we
//     //  aren't posting it.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     if ( (ExceptionCode != STATUS_CANT_WAIT) &&
//          (ExceptionCode != STATUS_VERIFY_REQUIRED) ) {
//
//         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
//     }
//
//     if ( IrpContext->ExceptionStatus == 0 ) {
//
//         if (FsRtlIsNtstatusExpected( ExceptionCode )) {
//
//             IrpContext->ExceptionStatus = ExceptionCode;
//
//             return EXCEPTION_EXECUTE_HANDLER;
//
//         } else {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( (ULONG_PTR)ExceptionPointer->ExceptionRecord,
//                          (ULONG_PTR)ExceptionPointer->ContextRecord,
//                          (ULONG_PTR)ExceptionPointer->ExceptionRecord->ExceptionAddress );
//         }
//
//     } else {
//
//         //
//         //  We raised this code explicitly ourselves, so it had better be
//         //  expected.
//         //
//
//         NT_ASSERT( IrpContext->ExceptionStatus == ExceptionCode );
//         NT_ASSERT( FsRtlIsNtstatusExpected( ExceptionCode ) );
//     }
//
//     return EXCEPTION_EXECUTE_HANDLER;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatProcessException (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN NTSTATUS ExceptionCode
//     )
//
// /*++
//
// Routine Description:
//
//     This routine process an exception.  It either completes the request
//     with the saved exception status or it sends it off to IoRaiseHardError()
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
//     ExceptionCode - Supplies the normalized exception status being handled
//
// Return Value:
//
//     NTSTATUS - Returns the results of either posting the Irp or the
//         saved completion status.
//
// --*/
//
// {
//     PVCB Vcb;
//     PIO_STACK_LOCATION IrpSp;
//     FAT_VOLUME_STATE TransitionState = VolumeDirty;
//     ULONG SavedFlags = 0;
//
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatProcessException\n", 0);
//
//     //
//     //  If there is not an irp context, we must have had insufficient resources.
//     //
//
//     if ( !ARGUMENT_PRESENT( IrpContext ) ) {
//
//         FatCompleteRequest( FatNull, Irp, ExceptionCode );
//
//         return ExceptionCode;
//     }
//
//     //
//     //  Get the real exception status from IrpContext->ExceptionStatus, and
//     //  reset it.
//     //
//
//     ExceptionCode = IrpContext->ExceptionStatus;
//     FatResetExceptionState( IrpContext );
//
//     //
//     //  If this is an Mdl write request, then take care of the Mdl
//     //  here so that things get cleaned up properly.  Cc now leaves
//     //  the MDL in place so a filesystem can retry after clearing an
//     //  internal condition (FAT does not).
//     //
//
//     if ((IrpContext->MajorFunction == IRP_MJ_WRITE) &&
//         (FlagOn( IrpContext->MinorFunction, IRP_MN_COMPLETE_MDL ) == IRP_MN_COMPLETE_MDL) &&
//         (Irp->MdlAddress != NULL)) {
//
//         PIO_STACK_LOCATION LocalIrpSp = IoGetCurrentIrpStackLocation(Irp);
//
//         CcMdlWriteAbort( LocalIrpSp->FileObject, Irp->MdlAddress );
//         Irp->MdlAddress = NULL;
//     }
//
//     //
//     //  If we are going to post the request, we may have to lock down the
//     //  user's buffer, so do it here in a try except so that we failed the
//     //  request if the LockPages fails.
//     //
//     //  Also unpin any repinned Bcbs, protected by the try {} except {} filter.
//     //
//
//     _SEH2_TRY {
//
//         SavedFlags = IrpContext->Flags;
//
//         //
//         //  Make sure we don't try to write through Bcbs
//         //
//
//         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//         IrpContext->Flags = SavedFlags;
//
//         //
//         //  If we will have to post the request, do it here.  Note
//         //  that the last thing FatPrePostIrp() does is mark the Irp pending,
//         //  so it is critical that we actually return PENDING.  Nothing
//         //  from this point to return can fail, so we are OK.
//         //
//         //  We cannot do a verify operations at APC level because we
//         //  have to wait for Io operations to complete.
//         //
//
//         if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL) &&
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             (((ExceptionCode == STATUS_VERIFY_REQUIRED) && KeAreAllApcsDisabled()) ||
// #else
//             (((ExceptionCode == STATUS_VERIFY_REQUIRED) && (KeGetCurrentIrql() >= APC_LEVEL)) ||
// #endif
//              (ExceptionCode == STATUS_CANT_WAIT))) {
//
//             ExceptionCode = FatFsdPostRequest( IrpContext, Irp );
//         }
//
//     } _SEH2_EXCEPT( FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() ) ) {
//
//         ExceptionCode = IrpContext->ExceptionStatus;
//         IrpContext->ExceptionStatus = 0;
//
//         IrpContext->Flags = SavedFlags;
//     } _SEH2_END;
//
//     //
//     //  If we posted the request, just return here.
//     //
//
//     if (ExceptionCode == STATUS_PENDING) {
//
//         return ExceptionCode;
//     }
//
//     Irp->IoStatus.Status = ExceptionCode;
//
//
//     //
//     //  If this request is not a "top-level" irp, just complete it.
//     //
//
//     if (FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL)) {
//
//         //
//         //  If there is a cache operation above us, commute verify
//         //  to a lock conflict.  This will cause retries so that
//         //  we have a chance of getting through without needing
//         //  to return an unaesthetic error for the operation.
//         //
//
//         if (IoGetTopLevelIrp() == (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP &&
//             ExceptionCode == STATUS_VERIFY_REQUIRED) {
//
//             ExceptionCode = STATUS_FILE_LOCK_CONFLICT;
//         }
//
//         FatCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//         return ExceptionCode;
//     }
//
//     if (IoIsErrorUserInduced(ExceptionCode)) {
//
//         //
//         //  Check for the various error conditions that can be caused by,
//         //  and possibly resolved by the user.
//         //
//
//         if (ExceptionCode == STATUS_VERIFY_REQUIRED) {
//
//             PDEVICE_OBJECT Device = NULL;
//
//             DebugTrace(0, Dbg, "Perform Verify Operation\n", 0);
//
//             //
//             //  Now we are at the top level file system entry point.
//             //
//             //  Grab the device to verify from the thread local storage
//             //  and stick it in the information field for transportation
//             //  to the fsp.  We also clear the field at this time.
//             //
//
//             Device = IoGetDeviceToVerify( Irp->Tail.Overlay.Thread );
//             IoSetDeviceToVerify( Irp->Tail.Overlay.Thread, NULL );
//
//             if ( Device == NULL ) {
//
//                 Device = IoGetDeviceToVerify( PsGetCurrentThread() );
//                 IoSetDeviceToVerify( PsGetCurrentThread(), NULL );
//
//                 NT_ASSERT( Device != NULL );
//             }
//
//             //
//             //  It turns out some storage drivers really do set invalid non-NULL device
//             //  objects to verify.
//             //
//             //  To work around this, completely ignore the device to verify in the thread,
//             //  and just use our real device object instead.
//             //
//
//             if (IrpContext->Vcb) {
//
//                 Device = IrpContext->Vcb->Vpb->RealDevice;
//
//             } else {
//
//                 //
//                 // For FSCTLs, IrpContext->Vcb may not be populated, so get the IrpContext->RealDevice instead
//                 //
//
//                 Device = IrpContext->RealDevice;
//             }
//
//             //
//             //  Let's not BugCheck just because the device to verify is somehow still NULL.
//             //
//
//             if (Device == NULL) {
//
//                 ExceptionCode = STATUS_DRIVER_INTERNAL_ERROR;
//
//                 FatCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//                 return ExceptionCode;
//             }
//
//             //
//             //  FatPerformVerify() will do the right thing with the Irp.
//
//             return FatPerformVerify( IrpContext, Irp, Device );
//         }
//
//         //
//         //  The other user induced conditions generate an error unless
//         //  they have been disabled for this request.
//         //
//
//         if (FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS)) {
//
//             FatCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//             return ExceptionCode;
//
//         } else {
//
//             //
//             //  Generate a pop-up
//             //
//
//             PDEVICE_OBJECT RealDevice = NULL;
//             PVPB Vpb;
//             PETHREAD Thread;
//
//             if (IoGetCurrentIrpStackLocation(Irp)->FileObject != NULL) {
//
//                 Vpb = IoGetCurrentIrpStackLocation(Irp)->FileObject->Vpb;
//
//             } else {
//
//                 Vpb = NULL;
//             }
//
//             //
//             //  The device to verify is either in my thread local storage
//             //  or that of the thread that owns the Irp.
//             //
//
//             Thread = Irp->Tail.Overlay.Thread;
//             RealDevice = IoGetDeviceToVerify( Thread );
//
//             if ( RealDevice == NULL ) {
//
//                 Thread = PsGetCurrentThread();
//                 RealDevice = IoGetDeviceToVerify( Thread );
//
//                 NT_ASSERT( RealDevice != NULL );
//             }
//
//             //
//             //  It turns out some storage drivers really do set invalid non-NULL device
//             //  objects to verify.
//             //
//             //  To work around this, completely ignore the device to verify in the thread,
//             //  and just use our real device object instead.
//             //
//
//             if (IrpContext->Vcb) {
//
//                 RealDevice = IrpContext->Vcb->Vpb->RealDevice;
//
//             } else {
//
//                 //
//                 // For FSCTLs, IrpContext->Vcb may not be populated, so get the IrpContext->RealDevice instead
//                 //
//
//                 RealDevice = IrpContext->RealDevice;
//             }
//
//             //
//             //  Let's not BugCheck just because the device to verify is somehow still NULL.
//             //
//
//             if (RealDevice == NULL) {
//
//                 FatCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//                 return ExceptionCode;
//             }
//
//             //
//             //  This routine actually causes the pop-up.  It usually
//             //  does this by queuing an APC to the callers thread,
//             //  but in some cases it will complete the request immediately,
//             //  so it is very important to IoMarkIrpPending() first.
//             //
//
//             IoMarkIrpPending( Irp );
//             IoRaiseHardError( Irp, Vpb, RealDevice );
//
//             //
//             //  We will be handing control back to the caller here, so
//             //  reset the saved device object.
//             //
//
//             IoSetDeviceToVerify( Thread, NULL );
//
//             //
//             //  The Irp will be completed by Io or resubmitted.  In either
//             //  case we must clean up the IrpContext here.
//             //
//
//             FatDeleteIrpContext( IrpContext );
//             return STATUS_PENDING;
//         }
//     }
//
//     //
//     //  This is just a run of the mill error.  If is a STATUS that we
//     //  raised ourselves, and the information would be use for the
//     //  user, raise an informational pop-up.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     Vcb = IrpContext->Vcb;
//
//     //
//     //  Now, if the Vcb is unknown to us this means that the error was raised
//     //  in the process of a mount and before we even had a chance to build
//     //  a full Vcb - and was really handled there.
//     //
//
//     if (Vcb != NULL) {
//
//         if ( !FatDeviceIsFatFsdo( IrpSp->DeviceObject) &&
//              !NT_SUCCESS(ExceptionCode) &&
//              !FsRtlIsTotalDeviceFailure(ExceptionCode) ) {
//
//             TransitionState = VolumeDirtyWithSurfaceTest;
//         }
//
//         //
//         //  If this was a STATUS_FILE_CORRUPT or similar error indicating some
//         //  nastiness out on the media, then mark the volume permanently dirty.
//         //
//
//         if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS) &&
//             ( TransitionState == VolumeDirtyWithSurfaceTest ||
//               (ExceptionCode == STATUS_FILE_CORRUPT_ERROR) ||
//               (ExceptionCode == STATUS_DISK_CORRUPT_ERROR) ||
//               (ExceptionCode == STATUS_EA_CORRUPT_ERROR) ||
//               (ExceptionCode == STATUS_INVALID_EA_NAME) ||
//               (ExceptionCode == STATUS_EA_LIST_INCONSISTENT) ||
//               (ExceptionCode == STATUS_NO_EAS_ON_FILE) )) {
//
//             NT_ASSERT( NodeType(Vcb) == FAT_NTC_VCB );
//             NT_ASSERT( !FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL));
//
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY );
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//             //
//             //  Do the "dirty" work, ignoring any error.  We need to take the Vcb here
//             //  to synchronize against the verify path tearing things down, since
//             //  we dropped all synchronization when backing up due to the exception.
//             //
//
//             FatAcquireExclusiveVcbNoOpCheck( IrpContext, Vcb);
//
//             _SEH2_TRY {
//
//                 if (VcbGood == Vcb->VcbCondition) {
//
//                     FatMarkVolume( IrpContext, Vcb, TransitionState );
//                 }
//             }
//             _SEH2_EXCEPT( FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() ) ) {
//
//                 NOTHING;
//             } _SEH2_END;
//
//             FatReleaseVcb( IrpContext, Vcb);
//         }
//     }
//
//     FatCompleteRequest( IrpContext, Irp, ExceptionCode );
//
//     return ExceptionCode;
// }
//
//
// VOID
// FatCompleteRequest_Real (
//     IN PIRP_CONTEXT IrpContext OPTIONAL,
//     IN PIRP Irp OPTIONAL,
//     IN NTSTATUS Status
//     )
//
// /*++
//
// Routine Description:
//
//     This routine completes a Irp
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
//     Status - Supplies the status to complete the Irp with
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
// #if DBG
//     if ( (FatBreakOnInterestingIrpCompletion != 0) && (Status == FatBreakOnInterestingIrpCompletion) ) {
//         DbgBreakPoint();
//     }
//
// #endif
//
//     //
//     //  If we have an Irp Context then unpin all of the repinned bcbs
//     //  we might have collected.
//     //
//
//     if (IrpContext != NULL) {
//
//         NT_ASSERT( IrpContext->Repinned.Bcb[0] == NULL );
//
//         FatUnpinRepinnedBcbs( IrpContext );
//     }
//
//     //
//     //  Delete the Irp context before completing the IRP so if
//     //  we run into some of the asserts, we can still backtrack
//     //  through the IRP.
//     //
//
//     if (IrpContext != NULL) {
//
//         FatDeleteIrpContext( IrpContext );
//     }
//
//     //
//     //  If we have an Irp then complete the irp.
//     //
//
//     if (Irp != NULL) {
//
//         //
//         //  We got an error, so zero out the information field before
//         //  completing the request if this was an input operation.
//         //  Otherwise IopCompleteRequest will try to copy to the user's buffer.
//         //
//
//         if ( NT_ERROR(Status) &&
//              FlagOn(Irp->Flags, IRP_INPUT_OPERATION) ) {
//
//             Irp->IoStatus.Information = 0;
//         }
//
//         Irp->IoStatus.Status = Status;
//
//         IoCompleteRequest( Irp, IO_DISK_INCREMENT );
//     }
//
//     return;
// }
//
// BOOLEAN
// FatIsIrpTopLevel (
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine detects if an Irp is the Top level requestor, ie. if it os OK
//     to do a verify or pop-up now.  If TRUE is returned, then no file system
//     resources are held above us.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
//     Status - Supplies the status to complete the Irp with
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     if ( IoGetTopLevelIrp() == NULL ) {
//
//         IoSetTopLevelIrp( Irp );
//
//         return TRUE;
//
//     } else {
//
//         return FALSE;
//     }
// }
//
//
// _Function_class_(FAST_IO_CHECK_IF_POSSIBLE)
// BOOLEAN
// NTAPI
// FatFastIoCheckIfPossible (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     IN BOOLEAN Wait,
//     IN ULONG LockKey,
//     IN BOOLEAN CheckForReadOperation,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if fast i/o is possible for a read/write operation
//
// Arguments:
//
//     FileObject - Supplies the file object used in the query
//
//     FileOffset - Supplies the starting byte offset for the read/write operation
//
//     Length - Supplies the length, in bytes, of the read/write operation
//
//     Wait - Indicates if we can wait
//
//     LockKey - Supplies the lock key
//
//     CheckForReadOperation - Indicates if this is a check for a read or write
//         operation
//
//     IoStatus - Receives the status of the operation if our return value is
//         FastIoReturnError
//
// Return Value:
//
//     BOOLEAN - TRUE if fast I/O is possible and FALSE if the caller needs
//         to take the long route.
//
// --*/
//
// {
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     LARGE_INTEGER LargeLength;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//     UNREFERENCED_PARAMETER( IoStatus );
//     UNREFERENCED_PARAMETER( Wait );
//     //
//     //  Decode the file object to get our fcb, the only one we want
//     //  to deal with is a UserFileOpen
//     //
//
//     if (FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb ) != UserFileOpen) {
//
//         return FALSE;
//     }
//
//     LargeLength.QuadPart = Length;
//
//     //
//     //  Based on whether this is a read or write operation we call
//     //  fsrtl check for read/write
//     //
//
//     if (CheckForReadOperation) {
//
//         if (FsRtlFastCheckLockForRead( &Fcb->Specific.Fcb.FileLock,
//                                        FileOffset,
//                                        &LargeLength,
//                                        LockKey,
//                                        FileObject,
//                                        PsGetCurrentProcess() )) {
//
//             return TRUE;
//         }
//
//     } else {
//
//
//         //
//         //  Also check for a write-protected volume here.
//         //
//
//         if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) &&
//             FsRtlFastCheckLockForWrite( &Fcb->Specific.Fcb.FileLock,
//                                         FileOffset,
//                                         &LargeLength,
//                                         LockKey,
//                                         FileObject,
//                                         PsGetCurrentProcess() )) {
//
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
//
// _Function_class_(FAST_IO_QUERY_BASIC_INFO)
// BOOLEAN
// NTAPI
// FatFastQueryBasicInfo (
//     IN PFILE_OBJECT FileObject,
//     IN BOOLEAN Wait,
//     IN OUT PFILE_BASIC_INFORMATION Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for basic file information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the basic information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     IRP_CONTEXT IrpContext;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN FcbAcquired = FALSE;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Prepare the dummy irp context
//     //
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT) );
//     IrpContext.NodeTypeCode = FAT_NTC_IRP_CONTEXT;
//     IrpContext.NodeByteSize = sizeof(IRP_CONTEXT);
//
//     if (Wait) {
//
//         SetFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     } else {
//
//         ClearFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     //
//     //  Determine the type of open for the input file object and only accept
//     //  the user file or directory open
//     //
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen) && (TypeOfOpen != UserDirectoryOpen)) {
//
//         return Results;
//     }
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Get access to the Fcb but only if it is not the paging file
//     //
//
//     if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE )) {
//
//         if (!ExAcquireResourceSharedLite( Fcb->Header.Resource, Wait )) {
//
//             FsRtlExitFileSystem();
//             return Results;
//         }
//
//         FcbAcquired = TRUE;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  If the Fcb is not in a good state, return FALSE.
//         //
//
//         if (Fcb->FcbCondition != FcbGood) {
//
//             try_return( Results );
//         }
//
//         Buffer->FileAttributes = 0;
//
//         //
//         //  If the fcb is not the root dcb then we will fill in the
//         //  buffer otherwise it is all setup for us.
//         //
//
//         if (NodeType(Fcb) != FAT_NTC_ROOT_DCB) {
//
//             //
//             //  Extract the data and fill in the non zero fields of the output
//             //  buffer
//             //
//
//             Buffer->LastWriteTime = Fcb->LastWriteTime;
//             Buffer->CreationTime = Fcb->CreationTime;
//             Buffer->LastAccessTime = Fcb->LastAccessTime;
//
//             //
//             //  Zero out the field we don't support.
//             //
//
//             Buffer->ChangeTime.QuadPart = 0;
//             Buffer->FileAttributes = Fcb->DirentFatFlags;
//
//         } else {
//
//             Buffer->LastWriteTime.QuadPart = 0;
//             Buffer->CreationTime.QuadPart = 0;
//             Buffer->LastAccessTime.QuadPart = 0;
//             Buffer->ChangeTime.QuadPart = 0;
//
//             Buffer->FileAttributes = FILE_ATTRIBUTE_DIRECTORY;
//         }
//
//
//         //
//         //  If the temporary flag is set, then set it in the buffer.
//         //
//
//         if (FlagOn( Fcb->FcbState, FCB_STATE_TEMPORARY )) {
//
//             SetFlag( Buffer->FileAttributes, FILE_ATTRIBUTE_TEMPORARY );
//         }
//
//         //
//         //  If no attributes were set, set the normal bit.
//         //
//
//         if (Buffer->FileAttributes == 0) {
//
//             Buffer->FileAttributes = FILE_ATTRIBUTE_NORMAL;
//         }
//
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = sizeof(FILE_BASIC_INFORMATION);
//
//         Results = TRUE;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         if (FcbAcquired) { ExReleaseResourceLite( Fcb->Header.Resource ); }
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     //
//     //  And return to our caller
//     //
//
//     return Results;
// }
//
//
// _Function_class_(FAST_IO_QUERY_STANDARD_INFO)
// BOOLEAN
// NTAPI
// FatFastQueryStdInfo (
//     IN PFILE_OBJECT FileObject,
//     IN BOOLEAN Wait,
//     IN OUT PFILE_STANDARD_INFORMATION Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for standard file information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the basic information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     IRP_CONTEXT IrpContext;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN FcbAcquired = FALSE;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Prepare the dummy irp context
//     //
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT) );
//     IrpContext.NodeTypeCode = FAT_NTC_IRP_CONTEXT;
//     IrpContext.NodeByteSize = sizeof(IRP_CONTEXT);
//
//     if (Wait) {
//
//         SetFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     } else {
//
//         ClearFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     //
//     //  Determine the type of open for the input file object and only accept
//     //  the user file or directory open
//     //
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen) && (TypeOfOpen != UserDirectoryOpen)) {
//
//         return Results;
//     }
//
//     //
//     //  Get access to the Fcb but only if it is not the paging file
//     //
//
//     FsRtlEnterFileSystem();
//
//     if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE )) {
//
//         if (!ExAcquireResourceSharedLite( Fcb->Header.Resource, Wait )) {
//
//             FsRtlExitFileSystem();
//             return Results;
//         }
//
//         FcbAcquired = TRUE;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  If the Fcb is not in a good state, return FALSE.
//         //
//
//         if (Fcb->FcbCondition != FcbGood) {
//
//             try_return( Results );
//         }
//
//         Buffer->NumberOfLinks = 1;
//         Buffer->DeletePending = BooleanFlagOn( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE );
//
//         //
//         //  Case on whether this is a file or a directory, and extract
//         //  the information and fill in the fcb/dcb specific parts
//         //  of the output buffer.
//         //
//
//         if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//             //
//             //  If we don't alread know the allocation size, we cannot look
//             //  it up in the fast path.
//             //
//
//             if (Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//                 try_return( Results );
//             }
//
//             Buffer->AllocationSize = Fcb->Header.AllocationSize;
//             Buffer->EndOfFile = Fcb->Header.FileSize;
//
//             Buffer->Directory = FALSE;
//
//         } else {
//
//             Buffer->AllocationSize = FatLargeZero;
//             Buffer->EndOfFile = FatLargeZero;
//
//             Buffer->Directory = TRUE;
//         }
//
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = sizeof(FILE_STANDARD_INFORMATION);
//
//         Results = TRUE;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         if (FcbAcquired) { ExReleaseResourceLite( Fcb->Header.Resource ); }
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     //
//     //  And return to our caller
//     //
//
//     return Results;
// }
//
//
// _Function_class_(FAST_IO_QUERY_NETWORK_OPEN_INFO)
// BOOLEAN
// NTAPI
// FatFastQueryNetworkOpenInfo (
//     IN PFILE_OBJECT FileObject,
//     IN BOOLEAN Wait,
//     IN OUT PFILE_NETWORK_OPEN_INFORMATION Buffer,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is for the fast query call for network open information.
//
// Arguments:
//
//     FileObject - Supplies the file object used in this operation
//
//     Wait - Indicates if we are allowed to wait for the information
//
//     Buffer - Supplies the output buffer to receive the information
//
//     IoStatus - Receives the final status of the operation
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeeded and FALSE if the caller
//         needs to take the long route.
//
// --*/
//
// {
//     BOOLEAN Results = FALSE;
//     IRP_CONTEXT IrpContext;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     BOOLEAN FcbAcquired = FALSE;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  Prepare the dummy irp context
//     //
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT) );
//     IrpContext.NodeTypeCode = FAT_NTC_IRP_CONTEXT;
//     IrpContext.NodeByteSize = sizeof(IRP_CONTEXT);
//
//     if (Wait) {
//
//         SetFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     } else {
//
//         ClearFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     //
//     //  Determine the type of open for the input file object and only accept
//     //  the user file or directory open
//     //
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &Fcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen) && (TypeOfOpen != UserDirectoryOpen)) {
//
//         return Results;
//     }
//
//     FsRtlEnterFileSystem();
//
//     //
//     //  Get access to the Fcb but only if it is not the paging file
//     //
//
//     if (!FlagOn( Fcb->FcbState, FCB_STATE_PAGING_FILE )) {
//
//         if (!ExAcquireResourceSharedLite( Fcb->Header.Resource, Wait )) {
//
//             FsRtlExitFileSystem();
//             return Results;
//         }
//
//         FcbAcquired = TRUE;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  If the Fcb is not in a good state, return FALSE.
//         //
//
//         if (Fcb->FcbCondition != FcbGood) {
//
//             try_return( Results );
//         }
//
//         //
//         //  Extract the data and fill in the non zero fields of the output
//         //  buffer
//         //
//
//         //
//         //  Default the field we don't support to a reasonable value.
//         //
//
//         ExLocalTimeToSystemTime( &FatJanOne1980,
//                                  &Buffer->ChangeTime );
//
//         Buffer->FileAttributes = Fcb->DirentFatFlags;
//
//         if (Fcb->Header.NodeTypeCode == FAT_NTC_ROOT_DCB) {
//
//             //
//             //  Reuse the default for the root dir.
//             //
//
//             Buffer->CreationTime =
//             Buffer->LastAccessTime =
//             Buffer->LastWriteTime = Buffer->ChangeTime;
//
//         } else {
//
//             Buffer->LastWriteTime = Fcb->LastWriteTime;
//             Buffer->CreationTime = Fcb->CreationTime;
//             Buffer->LastAccessTime = Fcb->LastAccessTime;
//
//         }
//
//         //
//         //  If the temporary flag is set, then set it in the buffer.
//         //
//
//         if (FlagOn( Fcb->FcbState, FCB_STATE_TEMPORARY )) {
//
//             SetFlag( Buffer->FileAttributes, FILE_ATTRIBUTE_TEMPORARY );
//         }
//
//
//
//         //
//         //  If no attributes were set, set the normal bit.
//         //
//
//         if (Buffer->FileAttributes == 0) {
//
//             Buffer->FileAttributes = FILE_ATTRIBUTE_NORMAL;
//         }
//
//         if (NodeType(Fcb) == FAT_NTC_FCB) {
//
//             //
//             //  If we don't already know the allocation size, we cannot
//             //  lock it up in the fast path.
//             //
//
//             if (Fcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//                 try_return( Results );
//             }
//
//             Buffer->AllocationSize = Fcb->Header.AllocationSize;
//             Buffer->EndOfFile = Fcb->Header.FileSize;
//
//         } else {
//
//             Buffer->AllocationSize = FatLargeZero;
//             Buffer->EndOfFile = FatLargeZero;
//         }
//
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = sizeof(FILE_NETWORK_OPEN_INFORMATION);
//
//         Results = TRUE;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         if (FcbAcquired) { ExReleaseResourceLite( Fcb->Header.Resource ); }
//
//         FsRtlExitFileSystem();
//     } _SEH2_END;
//
//     //
//     //  And return to our caller
//     //
//
//     return Results;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatPopUpFileCorrupt (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     The Following routine makes an informational popup that the file
//     is corrupt.
//
// Arguments:
//
//     Fcb - The file that is corrupt.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PKTHREAD Thread;
//
//     PAGED_CODE();
//
//     //
//     //  Disable the popup on the root directory.  It is important not
//     //  to generate them on objects which are part of the mount process.
//     //
//
//     if (NodeType(Fcb) == FAT_NTC_ROOT_DCB) {
//
//         return;
//     }
//
//     //
//     //  Got to grab the full filename now.
//     //
//
//     if (Fcb->FullFileName.Buffer == NULL) {
//
//         FatSetFullFileNameInFcb( IrpContext, Fcb );
//     }
//
//     //
//     //  We never want to block a system thread waiting for the user to
//     //  press OK.
//     //
//
//     if (IoIsSystemThread(IrpContext->OriginatingIrp->Tail.Overlay.Thread)) {
//
//        Thread = NULL;
//
//     } else {
//
// #ifndef __REACTOS__
//        Thread = IrpContext->OriginatingIrp->Tail.Overlay.Thread;
// #else
//        Thread = (PKTHREAD)IrpContext->OriginatingIrp->Tail.Overlay.Thread;
// #endif
//     }
//
//     IoRaiseInformationalHardError( STATUS_FILE_CORRUPT_ERROR,
//                                    &Fcb->FullFileName,
//                                    Thread);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
