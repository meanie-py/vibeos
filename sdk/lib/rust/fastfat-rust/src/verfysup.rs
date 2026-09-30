use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcIsThereDirtyData, _WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue, _DPFLTR_TYPE_DPFLTR_FASTFAT_ID as DPFLTR_FASTFAT_ID, ExQueueWorkItem, _KWAIT_REASON_Executive, _FAST_IO_POSSIBLE_FastIoIsNotPossible as FastIoIsNotPossible, PIO_COMPLETION_ROUTINE, IoAcquireVpbSpinLock, IoGetCurrentIrpStackLocation, IoReleaseVpbSpinLock, IoSetHardErrorOrVerifyDevice, IoVerifyVolume, KEVENT, KeSetTimer, LARGE_INTEGER, LOGICAL, _EVENT_TYPE_NotificationEvent, OEM_STRING, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, PLARGE_INTEGER, PMDL, RtlEqualString, PWORKER_THREAD_ROUTINE, UNICODE_STRING};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{BIOS_PARAMETER_BLOCK, CLEAN_AND_DIRTY_VOLUME_PACKET, DIRENT, FAT_DATA, FCB, FSINFO_SECTOR, IRP_CONTEXT, PACKED_BIOS_PARAMETER_BLOCK, PACKED_BOOT_SECTOR, PACKED_BOOT_SECTOR_EX, PDIRENT, PFCB, PIRP_CONTEXT, PVCB, VCB, VCB_STATE_VPB_NOT_ON_DEVICE, VOLUME_DEVICE_OBJECT};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, EXCEPTION_POINTERS, FILE_OBJECT, IO_STACK_LOCATION, IRP, MDL, _IO_STATUS_BLOCK, _PUBLIC_BCB as BCB, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IO_REMOUNT, IRP_MN_MOUNT_VOLUME, NT_SUCCESS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlRemoveLargeMcbEntry};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlBalanceReads};
use crate::types::{PBCB};
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
use crate::ddk_shims::FsRtlIsNtstatusExpected;

extern "system" {
    fn PAGED_CODE();
    } extern "C" { fn DebugTrace(level: i32, component: u32, format: *const i8, ...); } extern "system" {
    fn BooleanFlagOn(Flags: u32, Flag: u32) -> u8;
    fn FatNormalizeAndRaiseStatus(IrpContext: *const IRP_CONTEXT, Status: i32);
}
extern "system" {
    fn ClearFlag(state: *mut u32, flag: u32);
    fn DbgDoit(condition: i32);
    fn SetFlag(value: *mut u8, flag: u8);
    fn FlagOn(value: u32, flag: u32) -> i32;
    fn FatBytesPerFat(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatReservedBytes(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatSetFatEntry(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Index: u32, Value: u32);
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
    fn FatIsBootSectorFat(Sector: *mut PACKED_BOOT_SECTOR) -> u8;
    fn FatIsFat12(Vcb: *mut VCB) -> u8;
    fn FatIsFat32(Vcb: *mut VCB) -> u8;
    fn NT_ASSERT(condition: i32);
    fn KeInitializeEvent(Event: *mut KEVENT, Type: u32, State: u8);
    // Param 4 is `Flags: ULONG` (PIN_WAIT/PIN_EXCLUSIVE/PIN_NO_READ/PIN_IF_BCB), NOT
    // the NT4-era `BOOLEAN Wait`. Declared as u8 the callee read 32 bits where the
    // caller set 8, so the three flags above PIN_WAIT came from whatever was in the
    // register -- benign only because PIN_WAIT is 1 and TRUE is 1.
    fn CcPinRead( FileObject: *mut FILE_OBJECT, FileOffset: *const LARGE_INTEGER, Length: u32, Flags: u32, Bcb: *mut *mut c_void, Buffer: *mut *mut c_void, ) -> u8;
    fn IoBuildAsynchronousFsdRequest( MajorFunction: u32, DeviceObject: *mut DEVICE_OBJECT, Buffer: *mut c_void, Length: u32, StartingOffset: *const LARGE_INTEGER, IoStatusBlock: *mut _IO_STATUS_BLOCK, ) -> *mut IRP;
    fn IoCallDriver(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> NTSTATUS;
    fn KeWaitForSingleObject( Object: *mut c_void, WaitReason: u32, WaitMode: u32, Alertable: u8, Timeout: *const LARGE_INTEGER, ) -> NTSTATUS;
    fn MmUnlockPages(Mdl: *mut MDL);
    fn IoFreeMdl(Mdl: *mut MDL);
    fn IoFreeIrp(Irp: *mut IRP);
    fn FatReadVolumeFile( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, ByteOffset: u32, Length: u32, Bcb: *mut *mut BCB, Buffer: *mut *mut core::ffi::c_void, ); // VBO = u32 (was i64: i386 stdcall stack imbalance)
    // IsBpbFat32 defined below in Rust (was a C stub reading the WRONG offset)
    } extern "C" {
    fn KdPrintEx(ComponentId: u32, Level: u32, Format: *const u8, ...);
    // wdm FORCEINLINE helpers: cdecl bodies (`_IoSetCompletionRoutine`), bindgen
    // class — the "system" decls here were §9.99 folds.
    fn IoSetCompletionRoutine( Irp: *mut IRP, Routine: extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS, Context: *mut c_void, InvokeOnSuccess: u8, InvokeOnError: u8, InvokeOnCancel: u8, );
    fn IoGetNextIrpStackLocation(Irp: *mut IRP) -> *mut IO_STACK_LOCATION;
    } extern "system" {
    fn paged_code();
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn NodeType(Fcb: PFCB) -> u32;
    fn UNREFERENCED_PARAMETER(x: *mut core::ffi::c_void);
    fn FatAcquireExclusiveGlobal(IrpContext: PIRP_CONTEXT);
    fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    fn FatAcquireSharedVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> u32;
    fn FatCheckForDismount( IrpContext: PIRP_CONTEXT, Vcb: PVCB, Force: BOOLEAN, ) -> BOOLEAN;
    fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn FatExceptionFilter( IrpContext: PIRP_CONTEXT, ExceptionInfo: *mut EXCEPTION_POINTERS, ) -> LONG;
    fn FatFsdPostRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn FatProcessException( IrpContext: PIRP_CONTEXT, Irp: PIRP, ExceptionCode: NTSTATUS, ) -> NTSTATUS;
    fn FatReleaseGlobal(IrpContext: PIRP_CONTEXT);
    fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
    fn DebugDump(Message: *const i8, Component: u32, Object: *mut core::ffi::c_void);
    fn FatBugCheck(ExitCode: u32, Param1: u32, Param2: u32) -> !;
    fn FatAcquireSharedGlobal(irp_context: *mut IRP_CONTEXT);
    fn FatToggleMediaEjectDisable(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB, enable: u8) -> u32;
    fn FatUnpinRepinnedBcbs(irp_context: *mut IRP_CONTEXT);
    fn IoSetTopLevelIrp(irp: *mut core::ffi::c_void);
    fn ExFreePool(pool: *mut core::ffi::c_void);
    static FatData: FAT_DATA;
    fn KeSetEvent(event: PKEVENT, increment: i32, wait: i32);
    fn IoCompleteRequest(irp: PIRP, priority_boost: i32);
    fn FatVcbAcquiredExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u8;
    fn FatIsFastIoPossible(Fcb: *mut FCB) -> u8;
    fn FatGetNextFcbTopDown( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, OriginalFcb: *mut FCB, ) -> *mut FCB;
    fn FatGetDirentFromFcbOrDcb(irpctx: *mut IRP_CONTEXT, fcb: *mut FCB, arg2: u32, dirent: *mut *mut DIRENT, direntBcb: *mut *mut BCB);
    fn Fat8dot3ToString(irpctx: *mut IRP_CONTEXT, dirent: *mut DIRENT, arg3: u32, name: *mut OEM_STRING);
    // IsFileDeleted defined below in Rust (was an always-FALSE C stub)
    fn KeGetCurrentThread() -> *mut c_void;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = /* FAT_BUG_CHECK_VERFYSUP */ 0;
pub const Dbg: u32 = /* DEBUG_TRACE_VERFYSUP */ 0;
// ---- re-stitched from bank: 20260612-145724-verfysup.c/06_FatResetFcb.rs ----
extern "system" {
}

const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
const FCB_STATE_PAGING_FILE: u32 = 0x04;
const FAT_NTC_ROOT_DCB: u32 = 0x0504;
const FAT_NTC_DCB: u32 = 0x0503;
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;

#[no_mangle]
pub unsafe extern "system" fn FatResetFcb(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
) {
    let IsRealPagingFile: u8;

    PAGED_CODE();
    let _ = IrpContext;

    if FlagOn(((*(*Fcb).Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_REMOVABLE_MEDIA) == 0 &&
        FlagOn(((*Fcb).FcbState as u8) as u32, FCB_STATE_PAGING_FILE) != 0
    {
        IsRealPagingFile = 1; // TRUE
    } else {
        IsRealPagingFile = 0; // FALSE
    }

    if (NodeType(Fcb) != FAT_NTC_ROOT_DCB || FatIsFat32((*Fcb).Vcb) != 0) &&
        IsRealPagingFile == 0
    {
        FsRtlRemoveLargeMcbEntry(
            core::ptr::addr_of_mut!((*Fcb).Mcb),
            0,
            0xFFFFFFFF,
        );

        if (*Fcb).FirstClusterOfFile == 0 {
            (*Fcb).Header.AllocationSize.QuadPart = 0;
        } else {
            (*Fcb).Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
        }
    }

    if NodeType(Fcb) == FAT_NTC_DCB || NodeType(Fcb) == FAT_NTC_ROOT_DCB {
        (*Fcb).Specific.Dcb.as_mut().UnusedDirentVbo = 0xffffffff;
        (*Fcb).Specific.Dcb.as_mut().DeletedDirentHint = 0xffffffff;
    }
}

// ---- re-stitched from bank: 20260612-145724-verfysup.c/07_FatMatchFileSize.rs ----
extern "system" {
}

#[no_mangle]
pub unsafe extern "system" fn FatMatchFileSize(
    IrpContext: PIRP_CONTEXT,
    Dirent: PDIRENT,
    Fcb: PFCB,
) -> BOOLEAN {
    const FAT_NTC_FCB: u32 = 0x0502;

    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);

    if NodeType(Fcb) != FAT_NTC_FCB {
        return 1;
    }

    if (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart != (*Dirent).FileSize {
        return 0;
    }

    return 1;
}

// ---- re-stitched from bank: 20260612-145724-verfysup.c/15_FatDetermineAndMarkFcbCondition.rs ----
// NOTE: FatMarkFcbCondition is defined in full below (authoritative re-stitched body);
// its forward extern decl was removed to avoid E0428 redefinition.

#[no_mangle]
pub unsafe extern "system" fn FatDetermineAndMarkFcbCondition(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut Name: OEM_STRING = OEM_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: [i8; 16] = [0i8; 16];
    // PAGED_CODE(); (comment)

    //
    //  If this is the Root Dcb, special case it.  That is, we know
    //  by definition that it is good since it is fixed in the volume
    //  structure.
    //
    if NodeType(Fcb) == FAT_NTC_ROOT_DCB {
        FatMarkFcbCondition(IrpContext, Fcb, FcbGood as i32, 0/*FALSE*/);
        return;
    }

    //  The first thing we need to do to verify ourselves is
    //  locate the dirent on the disk.
    //
    FatGetDirentFromFcbOrDcb(
        IrpContext,
        Fcb,
        (1u8) as _, /* TRUE */
        &mut Dirent,
        &mut DirentBcb,
    );

    //
    //  If we couldn't get the dirent, this fcb must be bad (case of
    //  enclosing directory shrinking during the time it was ejected).
    //
    if DirentBcb.is_null() {
        FatMarkFcbCondition(IrpContext, Fcb, FcbBad as i32, 0/*FALSE*/);
        return;
    }

    //
    //  We located the dirent for ourselves now make sure it
    //  is really ours by comparing the Name and FatFlags.
    //  Then for a file we also check the file size.
    //
    //  Note that we have to unpin the Bcb before calling FatResetFcb
    //  in order to avoid a deadlock in CcUninitializeCacheMap.
    //
    // _SEH2_TRY / _SEH2_FINALLY
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    crate::seh::try_finally(&mut (), |_| {
        Name.MaximumLength = 16;
        Name.Buffer = Buffer.as_mut_ptr();

        Fat8dot3ToString(
            IrpContext,
            Dirent,
            (0u8) as _, /* FALSE */
            &mut Name,
        );

        //
        //  We need to calculate the first cluster 'cause FAT32 splits
        //  this field across the dirent.
        //
        let mut FirstClusterOfFile: u32 = ((*Dirent).FirstClusterOfFile) as u32;
        if FatIsFat32((*Fcb).Vcb) != 0 {
            FirstClusterOfFile += (*(*Dirent).__bindgen_anon_1.FirstClusterOfFileHi.as_ref() as u32) << 16;
        }

        if RtlEqualString(
            &Name as *const OEM_STRING as *const ntkrnl_sys::ddk::STRING,
            (*Fcb).ShortName.Name.Oem.as_ref() as *const OEM_STRING as *const ntkrnl_sys::ddk::STRING,
            1u8, /* TRUE */
        ) == 0
            || FatMatchFileSize(IrpContext, Dirent, Fcb) == 0
            || FirstClusterOfFile != (*Fcb).FirstClusterOfFile
            || (*Dirent).Attributes != (*Fcb).DirentFatFlags
        {
            FatMarkFcbCondition(IrpContext, Fcb, FcbBad as i32, 0/*FALSE*/);
        } else {
            //
            //  We passed.  Get the Fcb ready to use again.
            //
            FatMarkFcbCondition(IrpContext, Fcb, FcbGood as i32, 0/*FALSE*/);
        }
    }, |_, _| {
        FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
    });
}

// ---- re-stitched from bank: 20260612-145724-verfysup.c/11_FatDeferredCleanVolume.rs ----
extern "system" {
}

// fsrtltypes.h value; a wrong copy of these flags leaks a bogus top-level IRP that
// FatCommonWrite then dereferences (see resrcsup.rs FSRTL_CACHE_TOP_LEVEL_IRP).
const FSRTL_FSP_TOP_LEVEL_IRP: *mut core::ffi::c_void = 0x01 as *mut core::ffi::c_void;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
// VCB_CONDITION is 1-based (bindgen ground truth: VcbGood=1, VcbNotMounted=2, VcbBad=3).
// The old 0-based guesses made FatVerifyVcb treat a GOOD vcb as NOT-MOUNTED.
const VcbGood: u32 = crate::types::_VCB_CONDITION_VcbGood;
const VolumeClean: u32 = 0;
const VolumeDirty: u32 = 1;
const VCB_STATE_FLAG_SHUTDOWN: u32 = 0x40;
const VCB_STATE_FLAG_MOUNTED_DIRTY: u32 = 0x10;
const VCB_STATE_FLAG_VOLUME_DIRTY: u32 = 0x00000004;
const VCB_STATE_FLAG_BOOT_OR_PAGING_FILE: u32 = 0x0800;

#[no_mangle]
pub unsafe extern "system" fn FatDeferredCleanVolume(
    Parameter: *mut core::ffi::c_void,
) {
    // PAGED_CODE();

    DebugTrace(1, Dbg, c"FatDeferredCleanVolume\n".as_ptr() as *const i8, 0);

    let packet = Parameter as *mut CLEAN_AND_DIRTY_VOLUME_PACKET;
    let vcb = (*packet).Vcb;

    // Make us appear as a top level FSP request
    IoSetTopLevelIrp((FSRTL_FSP_TOP_LEVEL_IRP as PIRP).cast());

    // Dummy up an Irp Context
    let mut irp_context: IRP_CONTEXT = core::mem::zeroed();
    core::ptr::write_bytes(
        &mut irp_context as *mut IRP_CONTEXT as *mut u8,
        0,
        core::mem::size_of::<IRP_CONTEXT>(),
    );
    irp_context.Flags |= IRP_CONTEXT_FLAG_WAIT; // SetFlag

    // Acquire shared access to the global lock
    FatAcquireSharedGlobal(&raw mut irp_context);

    // Scan through the Vcb queue to see if this Vcb still exists
    let mut vcb_exists: u8 = 0;
    let mut links = FatData.VcbQueue.Flink;
    let vcb_queue_ptr = &raw const FatData.VcbQueue as *const LIST_ENTRY;
    while links as *const LIST_ENTRY != vcb_queue_ptr {
        let existing_vcb = (links as *const u8)
            .sub(core::mem::offset_of!(VCB, VcbLinks))
            as *mut VCB;
        if vcb == existing_vcb {
            vcb_exists = 1;
            break;
        }
        links = (*links).Flink;
    }

    // If the Vcb is good then mark it clean
    if vcb_exists != 0
        && (*vcb).VcbCondition == VcbGood
        && ((*vcb).VcbState & VCB_STATE_FLAG_SHUTDOWN) == 0
    {
        // _SEH2_TRY / _SEH2_EXCEPT(FsRtlIsNtstatusExpected(code) ? EXECUTE_HANDLER : CONTINUE_SEARCH)
        let _ = crate::seh::try_except(&mut (), |_| {
            if ((*vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                FatMarkVolume(&raw mut irp_context, vcb, VolumeClean as i32);
            }

            // Check for a pathological race condition and fix it
            if ((*vcb).VcbState & VCB_STATE_FLAG_VOLUME_DIRTY) != 0 {
                FatMarkVolume(&raw mut irp_context, vcb, VolumeDirty as i32);
            } else {
                // Unlock the volume if it is removable
                if ((*vcb).VcbState & VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
                    && ((*vcb).VcbState & VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0
                {
                    FatToggleMediaEjectDisable(&raw mut irp_context, vcb, ((0u8) as i32) as _);
                }
            }
        }, |_, pointers| {
            if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers)) != 0 {
                crate::seh::EXCEPTION_EXECUTE_HANDLER
            } else {
                crate::seh::EXCEPTION_CONTINUE_SEARCH
            }
        });
        // NOTHING;
    }

    // Release the global resource
    FatReleaseGlobal((&raw mut irp_context).cast());

    // Unpin repinned Bcbs
    // _SEH2_TRY / _SEH2_EXCEPT(FsRtlIsNtstatusExpected(code) ? EXECUTE_HANDLER : CONTINUE_SEARCH)
    let _ = crate::seh::try_except(&mut (), |_| {
        FatUnpinRepinnedBcbs(&raw mut irp_context);
    }, |_, pointers| {
        if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers)) != 0 {
            crate::seh::EXCEPTION_EXECUTE_HANDLER
        } else {
            crate::seh::EXCEPTION_CONTINUE_SEARCH
        }
    });
    // NOTHING;

    IoSetTopLevelIrp(core::ptr::null_mut());

    // Free the packet
    ExFreePool(packet as *mut core::ffi::c_void);
}


// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;

const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000;
const VolumeDirtyWithSurfaceTest: i32 = 2;
const FAT_BOOT_SECTOR_DIRTY: u8 = 1;
const FAT_BOOT_SECTOR_TEST_SURFACE: u8 = 2;
const SL_WRITE_THROUGH: u32 = 0x04;
const FAT_DIRTY_BIT_INDEX: u32 = 1;
const FAT_DIRTY_VOLUME: u32 = 0xF0000001; // ~FAT32_ENTRY_MASK | 1 (fat.h); was wrongly 0x0FFFFFF7 (a cluster marker)
const FAT_CLEAN_VOLUME: u32 = 0xF0000000; // ~FAT32_ENTRY_MASK | 0 (fat.h); was wrongly 0x0FFFFFF8 (a cluster marker)
const FSINFO_SECTOR_BEGIN_SIGNATURE: u32 = 0x41615252;
const FSINFO_SIGNATURE: u32 = 0x61417272;
const FSINFO_SECTOR_END_SIGNATURE: u32 = 0xAA550000;

const MAXIMUM_VOLUME_LABEL_LENGTH: u16 = 0x40; // xdk/iotypes.h (32 * sizeof(WCHAR))
const IRP_MN_VERIFY_VOLUME: u8 = 0x02;
const DPFLTR_INFO_LEVEL: u32 = 3;

const FO_CLEANUP_COMPLETE: u32 = 0x4000;
const STATUS_FILE_CLOSED: NTSTATUS = 0xC0000128_u32 as i32;
const IRP_PAGING_IO: u32 = 0x00000002;
const IRP_MJ_CLOSE: u8 = 0x02;
const IRP_MJ_SET_INFORMATION: u8 = 0x06;
const IRP_MJ_QUERY_INFORMATION: u8 = 0x05;
const IRP_MJ_READ: u8 = 0x03;
const IRP_MJ_WRITE: u8 = 0x04;
const IRP_MN_COMPLETE: u8 = 0x04;


const DO_VERIFY_VOLUME: u32 = 0x02;
const IRP_MJ_SET_EA: u32 = 8;
const IRP_MJ_SET_VOLUME_INFORMATION: u32 = 11;
const IRP_MN_USER_FS_REQUEST: u8 = 0x00;
const FSCTL_MARK_VOLUME_DIRTY: u32 = 0x00090030;
const VCB_STATE_FLAG_VOLUME_DISMOUNTED: u32 = 0x00010000;
const VcbNotMounted: i32 = crate::types::_VCB_CONDITION_VcbNotMounted as i32;
const VcbBad: i32 = crate::types::_VCB_CONDITION_VcbBad as i32;
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016_u32 as i32;
const STATUS_WRONG_VOLUME: i32 = 0xC0000012_u32 as i32;
const STATUS_FILE_INVALID: i32 = 0xC0000098_u32 as i32;
const STATUS_VOLUME_DISMOUNTED: i32 = 0xC000026E_u32 as i32;


const IO_DISK_INCREMENT: i32 = 1;

// Forward declarations for unknown functions

// Constants needed by the function
const FAT_NTC_FCB: u32 = 0x0502; // placeholder
const FcbNeedsToBeVerified: i32 = 3; // placeholder
const FcbBad: i32 = 2; // placeholder
const FcbGood: i32 = 1; // placeholder


// Note: VCB_STATE_VPB_NOT_ON_DEVICE is assumed to be in scope (defined elsewhere).
#[no_mangle]
pub unsafe extern "system" fn FatMarkDevForVerifyIfVcbMounted(Vcb: *mut VCB) -> i32 {
    let mut Marked: i32 = 0; // FALSE
    let mut SavedIrql: KIRQL = 0;

    IoAcquireVpbSpinLock(&mut SavedIrql as *mut KIRQL);

    if (*(*(*Vcb).Vpb).RealDevice).Vpb == (*Vcb).Vpb {
        // SetFlag( Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME );
        (*(*(*Vcb).Vpb).RealDevice).Flags |= DO_VERIFY_VOLUME;
        Marked = 1; // TRUE
    } else {
        // SetFlag( Vcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE );
        (*Vcb).VcbState |= VCB_STATE_VPB_NOT_ON_DEVICE;
    }

    IoReleaseVpbSpinLock(SavedIrql);

    Marked
}


// Typed replacements for two rust_cohab_stub.c helpers that used GUESSED byte
// offsets (magic-offset class): FatSetVcbCondition wrote at Vcb+344 (wrong →
// VcbCondition stayed 0 → "Invalid VcbCondition" FatBugCheck once the
// VCB_CONDITION consts were corrected), and IsBpbFat32 read PackedBpb+28
// (HiddenSectors!) instead of RootEntries at +6.
#[no_mangle]
pub unsafe extern "system" fn FatSetVcbCondition(Vcb: *mut VCB, Condition: crate::types::VCB_CONDITION) {
    (*Vcb).VcbCondition = Condition;
}

#[no_mangle]
pub unsafe extern "system" fn IsFileDeleted(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8 {
    // fatprocs.h macro; the C stub conservatively returned FALSE always.
    let _ = IrpContext;
    (((*Fcb).FcbState & crate::types::FCB_STATE_DELETE_ON_CLOSE) != 0
        && (*Fcb).UncleanCount == 0) as u8
}

#[no_mangle]
pub unsafe extern "system" fn IsBpbFat32(Bpb: *const PACKED_BIOS_PARAMETER_BLOCK) -> u8 {
    // C macro: (*(PUSHORT)&(bpb)->RootEntries == 0) — packed layout, unaligned-safe
    let root_entries = (*Bpb).RootEntries;
    ((root_entries[0] == 0) && (root_entries[1] == 0)) as u8
}

#[no_mangle]
pub unsafe extern "system" fn FatVerifyVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    let mut DevMarkedForVerify: i32 = 0;

    PAGED_CODE();

    DebugTrace(1, Dbg, c"FatVerifyVcb, Vcb = %p\n".as_ptr() as *const i8, Vcb);

    DevMarkedForVerify = BooleanFlagOn(
        (*(*(*Vcb).Vpb).RealDevice).Flags,
        DO_VERIFY_VOLUME,
    ) as i32;

    if DevMarkedForVerify == 0
        && (*IrpContext).MajorFunction == IRP_MJ_CREATE
        && !(*IrpContext).OriginatingIrp.is_null()
    {
        let IrpSp = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp) as *const IO_STACK_LOCATION;

        if (*(*IrpSp).FileObject).RelatedFileObject.is_null()
            && (*Vcb).VcbCondition == (VcbNotMounted) as u32
        {
            DevMarkedForVerify = 1;
        }
    }

    if DevMarkedForVerify != 0 {
        DebugTrace(0, Dbg, c"The Vcb needs to be verified\n".as_ptr() as *const i8, 0i32);

        IoSetHardErrorOrVerifyDevice(
            (*IrpContext).OriginatingIrp,
            (*(*Vcb).Vpb).RealDevice,
        );

        FatNormalizeAndRaiseStatus(IrpContext, STATUS_VERIFY_REQUIRED);
    }

    FatQuickVerifyVcb(IrpContext, Vcb);

    DebugTrace(-1, Dbg, c"FatVerifyVcb -> VOID\n".as_ptr() as *const i8, 0i32);
}
#[no_mangle]
pub unsafe extern "system" fn FatVerifyFcb(
    IrpContext: *mut c_void,
    Fcb: *mut FCB,
) {
    let mut CurrentFcb: *mut FCB;

    PAGED_CODE();

    DebugTrace(1, Dbg, b"FatVerifyFcb, Vcb = %p\n\0".as_ptr() as *const i8, Fcb as *const c_void);

    // Always refuse operations on dismounted volumes.
    if FlagOn((*(*Fcb).Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED) != 0 {
        FatRaiseStatus((IrpContext).cast(), STATUS_VOLUME_DISMOUNTED);
    }

    // If this is the Fcb of a deleted dirent or our parent is deleted...
    if IsFileDeleted(IrpContext.cast(), Fcb) != 0 ||
        (NodeType(Fcb) != FAT_NTC_ROOT_DCB) &&
        IsFileDeleted(IrpContext.cast(), (*Fcb).ParentDcb) != 0 {
        return;
    }

    // If we are not in the process of doing a verify...
    if (*(*Fcb).Vcb).VerifyThread != (KeGetCurrentThread()).cast() {
        FatQuickVerifyVcb((IrpContext).cast(), (*Fcb).Vcb);
    }

    // Now based on the condition of the Fcb...
    match (*Fcb).FcbCondition {
        __n if __n == FcbGood as u32 => {
            DebugTrace(0, Dbg, b"The Fcb is good\n\0".as_ptr() as *const i8);
        }
        __n if __n == FcbBad as u32 => {
            FatRaiseStatus((IrpContext).cast(), STATUS_FILE_INVALID);
        }
        __n if __n == FcbNeedsToBeVerified as u32 => {
            CurrentFcb = Fcb;

            while (*CurrentFcb).FcbCondition == (FcbNeedsToBeVerified) as u32 {
                FatDetermineAndMarkFcbCondition((IrpContext).cast(), CurrentFcb);

                if (*CurrentFcb).FcbCondition != (FcbGood) as u32 ||
                    NodeType(CurrentFcb) == FAT_NTC_ROOT_DCB {
                    break;
                }

                CurrentFcb = (*CurrentFcb).ParentDcb;
            }

            if (*Fcb).FcbCondition != (FcbGood) as u32 {
                FatRaiseStatus((IrpContext).cast(), STATUS_FILE_INVALID);
            }
        }
        _ => {
            DebugDump(b"Invalid FcbCondition\n\0".as_ptr() as *const i8, 0, Fcb as *mut core::ffi::c_void);
            FatBugCheck((*Fcb).FcbCondition as u32, 0, 0);
        }
    }

    DebugTrace(-1, Dbg, b"FatVerifyFcb -> VOID\n\0".as_ptr() as *const i8);
    // return;
}
#[no_mangle]
pub unsafe extern "system" fn FatCleanVolumeDpc(
    Dpc: *mut KDPC,
    DeferredContext: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let mut Vcb: *mut VCB;
    let mut Packet: *mut CLEAN_AND_DIRTY_VOLUME_PACKET;

    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    Vcb = DeferredContext as *mut VCB;

    if CcIsThereDirtyData((*Vcb).Vpb) != 0 {
        let mut TwoSecondsFromNow: LARGE_INTEGER = core::mem::zeroed();
        TwoSecondsFromNow.QuadPart = -20_000_000i64;

        KeSetTimer(
            core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer),
            TwoSecondsFromNow,
            core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc),
        );

        return;
    }

    let tag: u32 = u32::from_le_bytes([b' ', b't', b'a', b'F']);
    Packet = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        core::mem::size_of::<CLEAN_AND_DIRTY_VOLUME_PACKET>(),
        tag,
    ) as *mut CLEAN_AND_DIRTY_VOLUME_PACKET;

    if !Packet.is_null() {
        (*Packet).Vcb = Vcb;
        (*Packet).Irp = core::ptr::null_mut();

        ClearFlag(
            core::ptr::addr_of_mut!((*(*Packet).Vcb).VcbState),
            VCB_STATE_FLAG_VOLUME_DIRTY,
        );

        ExInitializeWorkItem(
            core::ptr::addr_of_mut!((*Packet).Item),
            Some(unsafe { core::mem::transmute::<*const (), _>(FatDeferredCleanVolume as *const ()) }),
            Packet as *mut core::ffi::c_void,
        );

        ExQueueWorkItem(core::ptr::addr_of_mut!((*Packet).Item), CriticalWorkQueue);
    }

    return;
}
#[no_mangle]
pub unsafe extern "system" fn FatFspMarkVolumeDirtyWithRecover(Parameter: *mut core::ffi::c_void) {
    let mut Packet: *mut CLEAN_AND_DIRTY_VOLUME_PACKET = Parameter as *mut CLEAN_AND_DIRTY_VOLUME_PACKET;
    let Vcb: PVCB = (*Packet).Vcb;
    let Irp: PIRP = (*Packet).Irp;

    DebugTrace(1, Dbg, b"FatFspMarkVolumeDirtyWithRecover\n\0".as_ptr() as *const i8, 0);

    // Zero-initialize IrpContext (RtlZeroMemory)
    let mut irpcontext_uninit = core::mem::MaybeUninit::<IRP_CONTEXT>::uninit();
    core::ptr::write_bytes(
        irpcontext_uninit.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<IRP_CONTEXT>(),
    );
    let mut IrpContext = irpcontext_uninit.assume_init();

    // SetFlag: IRP_CONTEXT_FLAG_WAIT
    IrpContext.Flags |= IRP_CONTEXT_FLAG_WAIT;
    IrpContext.OriginatingIrp = Irp;

    IoSetTopLevelIrp((FSRTL_FSP_TOP_LEVEL_IRP as *mut IRP).cast());

    // _SEH2_TRY / _SEH2_EXCEPT(FatExceptionFilter(&IrpContext, ...)) { NOTHING; }
    let p_irp_context: *mut IRP_CONTEXT = core::ptr::addr_of_mut!(IrpContext);
    let _ = crate::seh::fat_try_except(core::ptr::addr_of!(p_irp_context), || {
        // SetFlag: VCB_STATE_FLAG_MOUNTED_DIRTY
        (*Vcb).VcbState |= VCB_STATE_FLAG_MOUNTED_DIRTY;
        FatMarkVolume(
            p_irp_context,
            Vcb,
            (VolumeDirtyWithSurfaceTest) as i32,
        );
    });

    IoSetTopLevelIrp(core::ptr::null_mut());

    if !(*Packet).Event.is_null() {
        KeSetEvent((*Packet).Event, 0, 0); // FALSE = 0
    } else {
        IoCompleteRequest(Irp, IO_DISK_INCREMENT);
    }

    DebugTrace(-1, Dbg, b"FatFspMarkVolumeDirtyWithRecover\n\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatCheckDirtyBit(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    paged_code();

    let mut BootSector: *mut PACKED_BOOT_SECTOR = core::ptr::null_mut();
    let mut BootSectorBcb: PBCB = core::ptr::null_mut();

    FatReadVolumeFile(
        IrpContext,
        Vcb,
        0,
        core::mem::size_of::<PACKED_BOOT_SECTOR>() as u32,
        &mut BootSectorBcb as *mut PBCB as *mut *mut BCB ,
        (&mut BootSector as *mut *mut PACKED_BOOT_SECTOR as *mut *mut u8).cast(),
    );

    // _SEH2_TRY / _SEH2_FINALLY
    let p_boot_sector_bcb = core::ptr::addr_of_mut!(BootSectorBcb);
    crate::seh::try_finally(&mut (), |_| {
        let Dirty: bool;

        if IsBpbFat32(core::ptr::addr_of!((*BootSector).PackedBpb)) != 0 {
            Dirty = BooleanFlagOn(
                (*((BootSector as *mut PACKED_BOOT_SECTOR_EX))).CurrentHead as u32,
                FAT_BOOT_SECTOR_DIRTY as u32,
            ) != 0;
        } else {
            Dirty = BooleanFlagOn(
                (*BootSector).CurrentHead as u32,
                FAT_BOOT_SECTOR_DIRTY as u32,
            ) != 0;
        }

        let VolumeLabel = _UNICODE_STRING {
            Length: (*(*Vcb).Vpb).VolumeLabelLength,
            MaximumLength: MAXIMUM_VOLUME_LABEL_LENGTH,
            Buffer: core::ptr::addr_of!((*(*Vcb).Vpb).VolumeLabel) as *mut u16,
        };

        if Dirty {
            if !((*IrpContext).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL
                && (*IrpContext).MinorFunction == IRP_MN_VERIFY_VOLUME
                && FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) != 0)
            {
                KdPrintEx(
                    DPFLTR_FASTFAT_ID,
                    DPFLTR_INFO_LEVEL,
                    (b"FASTFAT: WARNING! Mounting Dirty Volume %Z\0".as_ptr() as *const i8).cast(),
                    &VolumeLabel as *const _UNICODE_STRING,
                );
                SetFlag(
                    (&mut (*Vcb).VcbState) as *mut u32 as *mut u8,
                    VCB_STATE_FLAG_MOUNTED_DIRTY as u8,
                );
            }
        } else {
            if FlagOn(((*Vcb).VcbState as u8) as u32, VCB_STATE_FLAG_MOUNTED_DIRTY) != 0 {
                KdPrintEx(
                    DPFLTR_FASTFAT_ID,
                    DPFLTR_INFO_LEVEL,
                    (b"FASTFAT: Volume %Z has been cleaned.\n\0".as_ptr() as *const i8).cast(),
                    &VolumeLabel as *const _UNICODE_STRING,
                );
                ClearFlag(&mut (*Vcb).VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY as u32);
            } else {
                let _ = FsRtlBalanceReads((*Vcb).TargetDeviceObject);
            }
        }
    }, |_, _| {
        FatUnpinBcb(IrpContext, p_boot_sector_bcb as *mut *mut core::ffi::c_void);
    });
}

#[no_mangle]
pub unsafe extern "system" fn FatVerifyOperationIsLegal(
    IrpContext: *mut IRP_CONTEXT,
) {
    PAGED_CODE();

    let irp: *mut IRP = (*IrpContext).OriginatingIrp;

    if irp.is_null() {
        return;
    }

    let file_object: *mut FILE_OBJECT = (*IoGetCurrentIrpStackLocation(irp)).FileObject;

    if file_object.is_null() {
        return;
    }

    if FlagOn((*file_object).Flags, FO_CLEANUP_COMPLETE) != 0 {
        let irp_sp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(irp);

        if FlagOn((*irp).Flags, IRP_PAGING_IO) != 0
            || (*irp_sp).MajorFunction == IRP_MJ_CLOSE
            || (*irp_sp).MajorFunction == IRP_MJ_SET_INFORMATION
            || (*irp_sp).MajorFunction == IRP_MJ_QUERY_INFORMATION
            || (((*irp_sp).MajorFunction == IRP_MJ_READ || (*irp_sp).MajorFunction == IRP_MJ_WRITE)
                && FlagOn((*irp_sp).MinorFunction as u32, IRP_MN_COMPLETE as u32) != 0)
        {
            // NOTHING
        } else {
            FatRaiseStatus(IrpContext, STATUS_FILE_CLOSED);
        }
    }

    return;
}
#[no_mangle]
pub unsafe extern "system" fn FatQuickVerifyVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    PAGED_CODE();

    let real_device = (*(*Vcb).Vpb).RealDevice;
    let flags = (*real_device).Flags;
    if FlagOn(flags, DO_VERIFY_VOLUME) != 0 {
        DebugTrace(
            0,
            Dbg,
            b"The Vcb needs to be verified\n\0".as_ptr() as *const i8,
            0,
        );
        IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, real_device);
        FatRaiseStatus(IrpContext, STATUS_VERIFY_REQUIRED);
    }

    let vcb_condition = (*Vcb).VcbCondition;
    match vcb_condition {
        VcbGood => {
            DebugTrace(0, Dbg, b"The Vcb is good\n\0".as_ptr() as *const i8, 0);

            let write_protected = FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0;
            let major = (*IrpContext).MajorFunction;
            let minor = (*IrpContext).MinorFunction;
            let is_modify = write_protected
                && (major == IRP_MJ_WRITE
                    || major == IRP_MJ_SET_INFORMATION
                    || major == (IRP_MJ_SET_EA) as u8
                    || major == IRP_MJ_FLUSH_BUFFERS
                    || major == (IRP_MJ_SET_VOLUME_INFORMATION) as u8
                    || (major == IRP_MJ_FILE_SYSTEM_CONTROL
                        && minor == IRP_MN_USER_FS_REQUEST
                        && {
                            let irp_sp = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);
                            (*irp_sp).Parameters.FileSystemControl.FsControlCode
                                == FSCTL_MARK_VOLUME_DIRTY
                        }));
            if is_modify {
                IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, real_device);
                FatMarkDevForVerifyIfVcbMounted(Vcb);
                FatRaiseStatus(IrpContext, STATUS_MEDIA_WRITE_PROTECTED);
            }
        }
        __n if __n == VcbNotMounted as u32 => {
            DebugTrace(0, Dbg, b"The Vcb is not mounted\n\0".as_ptr() as *const i8, 0);
            IoSetHardErrorOrVerifyDevice((*IrpContext).OriginatingIrp, real_device);
            FatRaiseStatus(IrpContext, STATUS_WRONG_VOLUME);
        }
        __n if __n == VcbBad as u32 => {
            DebugTrace(0, Dbg, b"The Vcb is bad\n\0".as_ptr() as *const i8, 0);
            if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED) != 0 {
                FatRaiseStatus(IrpContext, STATUS_VOLUME_DISMOUNTED);
            } else {
                FatRaiseStatus(IrpContext, STATUS_FILE_INVALID);
            }
        }
        _ => {
            DebugDump(
                b"Invalid VcbCondition\n\0".as_ptr() as *const i8,
                0,
                Vcb as *mut core::ffi::c_void,
            );
            // #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
            FatBugCheck((*Vcb).VcbCondition, 0, 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatPerformVerify(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    Device: PDEVICE_OBJECT,
) -> NTSTATUS {
    // The translation took all three parameters as pointer-to-pointer and ran the
    // try body through a no-op catch_unwind; the C passes them by value and this
    // is the path FatProcessException takes for every STATUS_VERIFY_REQUIRED.
    // PAGED_CODE();
    let Vcb: *mut VCB;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IrpSp: PIO_STACK_LOCATION;
    let FileObject: PFILE_OBJECT = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;
    let mut AllowRawMount: BOOLEAN = 0;
    let mut Irp = Irp;

    // Skip the IoVerifyVolume if this is a mount or verify request itself: a
    // recursive mount deadlocks on the DeviceObject->DeviceLock.
    if ((*IrpContext).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL)
        && (((*IrpContext).MinorFunction == IRP_MN_MOUNT_VOLUME as u8)
            || ((*IrpContext).MinorFunction == IRP_MN_VERIFY_VOLUME))
    {
        return FatFsdPostRequest(IrpContext, Irp);
    }

    DebugTrace(
        0,
        0x0120, /* Dbg - DEBUG_TRACE_VERFYSUP */
        b"Verify Required, DeviceObject = %p\n\0".as_ptr() as *const i8,
        Device,
    );

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    // Vcb = &CONTAINING_RECORD( IrpSp->DeviceObject, VOLUME_DEVICE_OBJECT, DeviceObject )->Vcb;
    {
        let offset = core::mem::offset_of!(VOLUME_DEVICE_OBJECT, DeviceObject);
        let container_ptr = ((*IrpSp).DeviceObject as *mut u8).sub(offset) as *mut VOLUME_DEVICE_OBJECT;
        Vcb = core::ptr::addr_of_mut!((*container_ptr).Vcb);
    }

    let ic = IrpContext;
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let caught = crate::seh::fat_try_except(core::ptr::addr_of!(ic), || {
        // Allow Raw to mount this volume if this was a DASD open.
        if ((*IrpContext).MajorFunction == IRP_MJ_CREATE)
            && ((*(*IrpSp).FileObject).FileName.Length == 0)
            && ((*(*IrpSp).FileObject).RelatedFileObject as *mut core::ffi::c_void).is_null()
        {
            AllowRawMount = 1;
        }

        // Send down the verify. This could be going to a different filesystem.
        *p_status = IoVerifyVolume(Device, AllowRawMount);

        // Acquire the Vcb so we're working with a stable Vcb condition.
        FatAcquireSharedVcb(IrpContext, Vcb);

        if (*p_status == STATUS_WRONG_VOLUME) && ((*Vcb).VcbCondition == VcbGood) {
            *p_status = STATUS_SUCCESS;
        } else if (*p_status == STATUS_SUCCESS) && ((*Vcb).VcbCondition != VcbGood) {
            *p_status = STATUS_WRONG_VOLUME;
        }

        // Quick unprotected check; FatCheckForDismount does the safe one.
        if (VcbGood != (*Vcb).VcbCondition) && (0 == (*Vcb).OpenFileCount) {
            FatReleaseVcb(IrpContext, Vcb);

            FatAcquireExclusiveGlobal(IrpContext);
            FatAcquireExclusiveVcb(IrpContext, Vcb);

            let VcbDeleted = FatCheckForDismount(IrpContext, Vcb, 0);

            if VcbDeleted == 0 {
                FatReleaseVcb(IrpContext, Vcb);
            }

            FatReleaseGlobal(IrpContext);
        } else {
            FatReleaseVcb(IrpContext, Vcb);
        }

        // If the IopMount in IoVerifyVolume did something, and this is an
        // absolute open, force a reparse.
        if ((*IrpContext).MajorFunction == IRP_MJ_CREATE)
            && ((*FileObject).RelatedFileObject as *mut core::ffi::c_void).is_null()
            && (*p_status == STATUS_SUCCESS || *p_status == STATUS_WRONG_VOLUME)
        {
            (**p_irp).IoStatus.Information = IO_REMOUNT as _;
            FatCompleteRequest(IrpContext, *p_irp, STATUS_REPARSE);
            *p_status = STATUS_REPARSE;
            *p_irp = core::ptr::null_mut();
        }

        if !(*p_irp).is_null() && *p_status < 0 {
            // Fill in the device object if required.
            if IoIsErrorUserInduced(*p_status) != 0 {
                IoSetHardErrorOrVerifyDevice(*p_irp, Device);
            }

            NT_ASSERT((STATUS_VERIFY_REQUIRED != *p_status) as i32);

            FatNormalizeAndRaiseStatus(IrpContext, *p_status);
        }

        // If there is still an Irp, send it off to an Ex Worker thread.
        if !(*p_irp).is_null() {
            *p_status = FatFsdPostRequest(IrpContext, *p_irp);
        }
    });
    if let Err(code) = caught {
        // Trouble performing the verify, or an error we raised ourselves: abort
        // the request with the status from the exception.
        Status = FatProcessException(IrpContext, Irp, code);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatMarkVolumeCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: PVOID,
) -> NTSTATUS {
    // Set the event so that our call will wake up.
    KeSetEvent(Contxt as *mut KEVENT, 0, 0);
    let _ = DeviceObject;
    let _ = Irp;
    const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;
    STATUS_MORE_PROCESSING_REQUIRED
}



// ==== re-stitched (genuine drop, authoritative) ====
#[no_mangle]
pub unsafe extern "system" fn FatMarkFcbCondition(
    IrpContext: *mut IRP_CONTEXT,
    mut Fcb: *mut FCB,
    FcbCondition: i32,
    Recursive: i32,
) {
    const Dbg: u32 = /* DEBUG_TRACE_VERFYSUP */ 0;
    const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
    const FCB_STATE_PAGING_FILE: u32 = 0x04;
    const FAT_NTC_FCB: i16 = 0x0502;
    const FAT_NTC_DCB: i16 = 0x0503;
    const FAT_NTC_ROOT_DCB: i16 = 0x0504;
    const FcbNeedsToBeVerified: i32 = 3;
    const FcbBad: i32 = 2;
    const FcbGood: u32 = 1;
    // PAGED_CODE();  -- skipped, no equivalent

    DebugTrace(
        1,
        Dbg,
        b"FatMarkFcbCondition, Fcb = %p\n\0".as_ptr() as *const i8,
        Fcb as usize,
    );

    // Assert that if we are not verifying, we hold the Vcb exclusive
    NT_ASSERT(if FcbCondition != FcbNeedsToBeVerified {
        1
    } else {
        FatVcbAcquiredExclusive(IrpContext, (*Fcb).Vcb) as i32
    });

    // Paging file special case
    if FlagOn((*(*Fcb).Vcb).VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) == 0
        && FlagOn((*Fcb).FcbState, FCB_STATE_PAGING_FILE) != 0
    {
        (*Fcb).FcbCondition = FcbGood;
        return;
    }

    (*Fcb).FcbCondition = FcbCondition as u32;

    DebugTrace(
        0,
        Dbg,
        b"MarkFcb: %wZ\n\0".as_ptr() as *const i8,
        &(*Fcb).FullFileName as *const UNICODE_STRING as usize,
    );

    // Update FastIo possibility if this is a regular FCB
    if (*Fcb).Header.NodeTypeCode == FAT_NTC_FCB {
        (*Fcb).Header.IsFastIoPossible = FatIsFastIoPossible(Fcb) as u8;
    }

    if FcbCondition == FcbNeedsToBeVerified {
        FatResetFcb(IrpContext, Fcb);
    }

    // Recursively mark children if needed
    if ((FcbCondition == FcbNeedsToBeVerified) || (FcbCondition == FcbBad))
        && Recursive != 0
        && ((*Fcb).Header.NodeTypeCode == FAT_NTC_DCB
            || (*Fcb).Header.NodeTypeCode == FAT_NTC_ROOT_DCB)
    {
        let OriginalFcb = Fcb;

        loop {
            Fcb = FatGetNextFcbTopDown(IrpContext, Fcb, OriginalFcb);
            if Fcb.is_null() {
                break;
            }

            DebugTrace(
                0,
                Dbg,
                b"MarkFcb: %wZ\n\0".as_ptr() as *const i8,
                &(*Fcb).FullFileName as *const UNICODE_STRING as usize,
            );

            (*Fcb).FcbCondition = FcbCondition as u32;

            // FastIo is not possible when propagating a parent's bad/verify flag
            (*Fcb).Header.IsFastIoPossible = FastIoIsNotPossible as u8;

            if FcbCondition == FcbNeedsToBeVerified {
                FatResetFcb(IrpContext, Fcb);
            }
        }
    }

    DebugTrace(
        -1,
        Dbg,
        b"FatMarkFcbCondition -> VOID\n\0".as_ptr() as *const i8,
        0usize,
    );
}

// ==== re-stitched (genuine drop, authoritative) ====
#[no_mangle]
pub unsafe extern "system" fn FatMarkVolume(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    VolumeState: i32,
) {
    const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000;
    const VolumeClean: i32 = 0;
    const VolumeDirty: i32 = 1;
    const VolumeDirtyWithSurfaceTest: i32 = 2;
    const FAT_BOOT_SECTOR_DIRTY: u8 = 1;
    const FAT_BOOT_SECTOR_TEST_SURFACE: u8 = 2;
    const SL_WRITE_THROUGH: u8 = 0x04;
    const FAT_DIRTY_BIT_INDEX: u32 = 1;
    const FAT_DIRTY_VOLUME: u32 = 0xF0000001; // ~FAT32_ENTRY_MASK | 1 (fat.h); was wrongly 0x0FFFFFF7 (a cluster marker)
    const FAT_CLEAN_VOLUME: u32 = 0xF0000000; // ~FAT32_ENTRY_MASK | 0 (fat.h); was wrongly 0x0FFFFFF8 (a cluster marker)
    const FSINFO_SECTOR_BEGIN_SIGNATURE: u32 = 0x41615252;
    const FSINFO_SIGNATURE: u32 = 0x61417272;
    const FSINFO_SECTOR_END_SIGNATURE: u32 = 0xAA550000;
    const Dbg: u32 = 0;
    let mut Sector: *mut u8 = core::ptr::null_mut();
    let mut Bcb: *mut BCB = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: *mut IRP = core::ptr::null_mut();
    let mut FsInfoUpdate: i32 = 0;
    let mut FsInfoOffset: u32 = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut abort: i32 = 0;

    DebugTrace(1, Dbg, b"FatMarkVolume, Vcb = %p\n\0".as_ptr() as *const i8, Vcb as *const _);

    let p_sector = core::ptr::addr_of_mut!(Sector);
    let p_bcb = core::ptr::addr_of_mut!(Bcb);
    let p_event = core::ptr::addr_of_mut!(Event);
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let p_fs_info_update = core::ptr::addr_of_mut!(FsInfoUpdate);
    let p_fs_info_offset = core::ptr::addr_of_mut!(FsInfoOffset);
    let p_offset = core::ptr::addr_of_mut!(Offset);
    let p_abort = core::ptr::addr_of_mut!(abort);

    if ((*Vcb).VcbState & VCB_STATE_FLAG_WRITE_PROTECTED) != 0 || FatIsFat12(Vcb) != 0 {
        return;
    }

    for this_pass in 0u32..2 {
        if this_pass == 1
            && (!(FatIsFat32(Vcb) != 0)
                || VolumeState != VolumeClean
                || *p_fs_info_update != 0
                || (*Vcb).Bpb.FsInfoSector == 0)
        {
            break;
        }

        //  Bail if we get an IO error.
        // _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            let mut PinLength: u32;
            let mut WriteLength: u32;
            (*p_offset).QuadPart = 0;

            if (*Vcb).AllocationSupport.FatIndexBitSize == 12 {
                PinLength =
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb)) + FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb));
                WriteLength = (*Vcb).Bpb.BytesPerSector as u32;
            } else {
                WriteLength = (*Vcb).Bpb.BytesPerSector as u32;
                PinLength = WriteLength;
                if FatIsFat32(Vcb) != 0 && VolumeState == VolumeClean {
                    if (*Vcb).Bpb.FsInfoSector == 1 {
                        NT_ASSERT((this_pass == 0) as i32);
                        *p_fs_info_update = 1;
                        *p_fs_info_offset = (*Vcb).Bpb.BytesPerSector as u32;
                        WriteLength = (*Vcb).Bpb.BytesPerSector as u32 * 2;
                        PinLength = WriteLength;
                    } else if this_pass == 1 {
                        *p_fs_info_update = 1;
                        *p_fs_info_offset = 0;
                        (*p_offset).QuadPart = ((*Vcb).Bpb.BytesPerSector as u64
                            * (*Vcb).Bpb.FsInfoSector as u64)
                            as i64;
                    }
                }
            }

            CcPinRead(
                (*Vcb).VirtualVolumeFile,
                p_offset,
                PinLength,
                1,
                p_bcb as *mut *mut c_void,
                p_sector as *mut *mut c_void,
            );

            let pin_count = {
                (*IrpContext).PinCount += 1;
                (*IrpContext).PinCount
            };
            DbgDoit(pin_count as i32);

            if this_pass == 0 {
                if FatIsBootSectorFat(*p_sector as *mut PACKED_BOOT_SECTOR) == 0 {
                    *p_abort = 1;
                    return; // _SEH2_LEAVE
                }

                let CurrentHead: *mut u8 = if FatIsFat32(Vcb) != 0 {
                    core::ptr::addr_of_mut!(
                        (*(*p_sector as *mut PACKED_BOOT_SECTOR_EX)).CurrentHead
                    ) as *mut u8
                } else {
                    core::ptr::addr_of_mut!(
                        (*(*p_sector as *mut PACKED_BOOT_SECTOR)).CurrentHead
                    ) as *mut u8
                };

                if VolumeState == VolumeClean {
                    *CurrentHead &= !FAT_BOOT_SECTOR_DIRTY;
                } else {
                    *CurrentHead |= FAT_BOOT_SECTOR_DIRTY;
                    if VolumeState == VolumeDirtyWithSurfaceTest {
                        *CurrentHead |= FAT_BOOT_SECTOR_TEST_SURFACE;
                    }
                }
            }

            if *p_fs_info_update != 0 {
                let FsInfoSector = (*p_sector)
                    .add(*p_fs_info_offset as usize) as *mut FSINFO_SECTOR;
                (*FsInfoSector).SectorBeginSignature = FSINFO_SECTOR_BEGIN_SIGNATURE;
                (*FsInfoSector).FsInfoSignature = FSINFO_SIGNATURE;
                (*FsInfoSector).FreeClusterCount =
                    (*Vcb).AllocationSupport.NumberOfFreeClusters;
                (*FsInfoSector).NextFreeCluster = (*Vcb).ClusterHint;
                (*FsInfoSector).SectorEndSignature = FSINFO_SECTOR_END_SIGNATURE;
            }

            KeInitializeEvent(p_event, _EVENT_TYPE_NotificationEvent as u32, 0);

            *p_irp = IoBuildAsynchronousFsdRequest(
                IRP_MJ_WRITE as u32,
                (*Vcb).TargetDeviceObject,
                *p_sector as *mut c_void,
                WriteLength,
                p_offset,
                core::ptr::null_mut(),
            );

            if (*p_irp).is_null() {
                return; // try_return(NOTHING)
            }

            let stack_loc = IoGetNextIrpStackLocation(*p_irp);
            *core::ptr::addr_of_mut!((*stack_loc).Flags) |= SL_WRITE_THROUGH;

            IoSetCompletionRoutine(
                *p_irp,
                core::mem::transmute::<
                    unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS,
                    extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut c_void) -> NTSTATUS,
                >(FatMarkVolumeCompletionRoutine),
                p_event as *mut c_void,
                1,
                1,
                1,
            );

            let Status: NTSTATUS = IoCallDriver((*Vcb).TargetDeviceObject, *p_irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    p_event as *mut c_void,
                    _KWAIT_REASON_Executive as u32,
                    KernelMode as u32,
                    0,
                    core::ptr::null(),
                );
            }
        }, |_, _| {
            //  Clean up the Irp and Mdl
            let Irp = *p_irp;
            if !Irp.is_null() {
                while !(*Irp).MdlAddress.is_null() {
                    let next_mdl = (*(*Irp).MdlAddress).Next;
                    MmUnlockPages((*Irp).MdlAddress);
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = next_mdl;
                }
                IoFreeIrp(Irp);
            }
            if !(*p_bcb).is_null() {
                FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);
            }
        });
    }

    if *p_abort == 0 {
        if VolumeState == VolumeDirty {
            FatSetFatEntry(IrpContext, Vcb, FAT_DIRTY_BIT_INDEX, FAT_DIRTY_VOLUME);
        } else {
            FatSetFatEntry(IrpContext, Vcb, FAT_DIRTY_BIT_INDEX, FAT_CLEAN_VOLUME);
        }
    }

    DebugTrace(-1, Dbg, b"FatMarkVolume -> VOID\n\0".as_ptr() as *const i8, 0);
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
//     VerfySup.c
//
// Abstract:
//
//     This module implements the Fat Verify volume and fcb/dcb support
//     routines
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_VERFYSUP)
//
// //
// //  The Debug trace level for this module
// //
//
// #define Dbg                              (DEBUG_TRACE_VERFYSUP)
//
// //
// //  Local procedure prototypes
// //
//
// VOID
// FatResetFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     );
//
// BOOLEAN
// FatMatchFileSize (
//     __in PIRP_CONTEXT IrpContext,
//     __in PDIRENT Dirent,
//     __in PFCB Fcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDetermineAndMarkFcbCondition (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     );
//
// WORKER_THREAD_ROUTINE FatDeferredCleanVolume;
//
// VOID
// NTAPI
// FatDeferredCleanVolume (
//      PVOID Parameter
//     );
//
// IO_COMPLETION_ROUTINE FatMarkVolumeCompletionRoutine;
//
// NTSTATUS
// NTAPI
// FatMarkVolumeCompletionRoutine(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Contxt
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCheckDirtyBit)
// #pragma alloc_text(PAGE, FatVerifyOperationIsLegal)
// #pragma alloc_text(PAGE, FatDeferredCleanVolume)
// #pragma alloc_text(PAGE, FatMatchFileSize)
// #pragma alloc_text(PAGE, FatDetermineAndMarkFcbCondition)
// #pragma alloc_text(PAGE, FatQuickVerifyVcb)
// #pragma alloc_text(PAGE, FatPerformVerify)
// #pragma alloc_text(PAGE, FatMarkFcbCondition)
// #pragma alloc_text(PAGE, FatResetFcb)
// #pragma alloc_text(PAGE, FatVerifyVcb)
// #pragma alloc_text(PAGE, FatVerifyFcb)
// #endif
//
//
// VOID
// FatMarkFcbCondition (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN FCB_CONDITION FcbCondition,
//     IN BOOLEAN Recursive
//     )
//
// /*++
//
// Routine Description:
//
//     This routines marks the entire Fcb/Dcb structure from Fcb down with
//     FcbCondition.
//
// Arguments:
//
//     Fcb - Supplies the Fcb/Dcb being marked
//
//     FcbCondition - Supplies the setting to use for the Fcb Condition
//
//     Recursive - Specifies whether this condition should be applied to
//         all children (see the case where we are invalidating a live volume
//         for a case where this is now desireable).
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
//     DebugTrace(+1, Dbg, "FatMarkFcbCondition, Fcb = %p\n", Fcb );
//
//     //
//     //  If we are marking this Fcb something other than Good, we will need
//     //  to have the Vcb exclusive.
//     //
//
//     NT_ASSERT( FcbCondition != FcbNeedsToBeVerified ? TRUE :
//             FatVcbAcquiredExclusive(IrpContext, Fcb->Vcb) );
//
//     //
//     //  If this is a PagingFile it has to be good unless media underneath is
//     //  removable. The "removable" check was added specifically for ReadyBoost,
//     //  which opens its cache file on a removable device as a paging file and
//     //  relies on the file system to validate its mapping information after a
//     //  power transition.
//     //
//
//     if (!FlagOn(Fcb->Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//         FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE)) {
//
//         Fcb->FcbCondition = FcbGood;
//         return;
//     }
//
//     //
//     //  Update the condition of the Fcb.
//     //
//
//     Fcb->FcbCondition = FcbCondition;
//
//     DebugTrace(0, Dbg, "MarkFcb: %wZ\n", &Fcb->FullFileName);
//
//     //
//     //  This FastIo flag is based on FcbCondition, so update it now.  This only
//     //  applies to regular FCBs, of course.
//     //
//
//     if (Fcb->Header.NodeTypeCode == FAT_NTC_FCB) {
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//     }
//
//     if (FcbCondition == FcbNeedsToBeVerified) {
//         FatResetFcb( IrpContext, Fcb );
//     }
//
//     //
//     //  Now if we marked NeedsVerify or Bad a directory then we also need to
//     //  go and mark all of our children with the same condition.
//     //
//
//     if ( ((FcbCondition == FcbNeedsToBeVerified) ||
//           (FcbCondition == FcbBad)) &&
//          Recursive &&
//          ((Fcb->Header.NodeTypeCode == FAT_NTC_DCB) ||
//           (Fcb->Header.NodeTypeCode == FAT_NTC_ROOT_DCB)) ) {
//
//         PFCB OriginalFcb = Fcb;
//
//         while ( (Fcb = FatGetNextFcbTopDown(IrpContext, Fcb, OriginalFcb)) != NULL ) {
//
//             DebugTrace(0, Dbg, "MarkFcb: %wZ\n", &Fcb->FullFileName);
//
//             Fcb->FcbCondition = FcbCondition;
//
//             //
//             //  We already know that FastIo is not possible since we are propagating
//             //  a parent's bad/verify flag down the tree - IO to the children must
//             //  take the long route for now.
//             //
//
//             Fcb->Header.IsFastIoPossible = FastIoIsNotPossible;
//
//             //
//             //  Leave all the Fcbs in a condition to be verified.
//             //
//
//             if (FcbCondition == FcbNeedsToBeVerified) {
//                 FatResetFcb( IrpContext, Fcb );
//             }
//
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatMarkFcbCondition -> VOID\n", 0);
//
//     return;
// }
//
// BOOLEAN
// FatMarkDevForVerifyIfVcbMounted(
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks to see if the specified Vcb is currently mounted on
//     the device or not.  If it is,  it sets the verify flag on the device, if
//     not then the state is noted in the Vcb.
//
// Arguments:
//
//     Vcb - This is the volume to check.
//
// Return Value:
//
//     TRUE if the device has been marked for verify here,  FALSE otherwise.
//
// --*/
// {
//     BOOLEAN Marked = FALSE;
//     KIRQL SavedIrql;
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28175, "touching Vpb is ok for a filesystem" )
// #endif
//
//     if (Vcb->Vpb->RealDevice->Vpb == Vcb->Vpb)  {
//
//         SetFlag( Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//         Marked = TRUE;
//     }
//     else {
//
//         //
//         //  Flag this to avoid the VPB spinlock in future passes.
//         //
//
//         SetFlag( Vcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE);
//     }
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     return Marked;
// }
//
//
// VOID
// FatVerifyVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routines verifies that the Vcb still denotes a valid Volume
//     If the Vcb is bad it raises an error condition.
//
// Arguments:
//
//     Vcb - Supplies the Vcb being verified
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     BOOLEAN DevMarkedForVerify;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatVerifyVcb, Vcb = %p\n", Vcb );
//
//     //
//     //  If the verify volume flag in the device object is set
//     //  this means the media has potentially changed.
//     //
//     //  Note that we only force this ping for create operations.
//     //  For others we take a sporting chance.  If in the end we
//     //  have to physically access the disk, the right thing will happen.
//     //
//
//     DevMarkedForVerify = BooleanFlagOn(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME);
//
//     //
//     //  We ALWAYS force CREATE requests on unmounted volumes through the
//     //  verify path.  These requests could have been in limbo between
//     //  IoCheckMountedVpb and us, when a verify/mount took place and caused
//     //  a completely different fs/volume to be mounted.  In this case the
//     //  checks above may not have caught the condition, since we may already
//     //  have verified (wrong volume) and decided that we have nothing to do.
//     //  We want the requests to be re routed to the currently mounted volume,
//     //  since they were directed at the 'drive',  not our volume.  So we take
//     //  the verify path for synchronisation,  and the request will eventually
//     //  be bounced back to IO with STATUS_REPARSE by our verify handler.
//     //
//
//     if (!DevMarkedForVerify &&
//         (IrpContext->MajorFunction == IRP_MJ_CREATE) &&
//         (IrpContext->OriginatingIrp != NULL)) {
//
//         PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp);
//
//         if ((IrpSp->FileObject->RelatedFileObject == NULL) &&
//             (Vcb->VcbCondition == VcbNotMounted)) {
//
//             DevMarkedForVerify = TRUE;
//         }
//     }
//
//     //
//     //  Raise any error condition otherwise.
//     //
//
//     if (DevMarkedForVerify) {
//
//         DebugTrace(0, Dbg, "The Vcb needs to be verified\n", 0);
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                       Vcb->Vpb->RealDevice );
//
//         FatNormalizeAndRaiseStatus( IrpContext, STATUS_VERIFY_REQUIRED );
//     }
//
//     //
//     //  Check the operation is legal for current Vcb state.
//     //
//
//     FatQuickVerifyVcb( IrpContext, Vcb );
//
//     DebugTrace(-1, Dbg, "FatVerifyVcb -> VOID\n", 0);
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatVerifyFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routines verifies that the Fcb still denotes the same file.
//     If the Fcb is bad it raises a error condition.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being verified
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFCB CurrentFcb;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatVerifyFcb, Vcb = %p\n", Fcb );
//
//     //
//     //  Always refuse operations on dismounted volumes.
//     //
//
//     if (FlagOn( Fcb->Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED )) {
//
//         FatRaiseStatus( IrpContext, STATUS_VOLUME_DISMOUNTED );
//     }
//
//     //
//     //  If this is the Fcb of a deleted dirent or our parent is deleted,
//     //  no-op this call with the hope that the caller will do the right thing.
//     //  The only caller we really have to worry about is the AdvanceOnly
//     //  callback for setting valid data length from Cc, this will happen after
//     //  cleanup (and file deletion), just before the SCM is ripped down.
//     //
//
//     if (IsFileDeleted( IrpContext, Fcb ) ||
//         ((NodeType(Fcb) != FAT_NTC_ROOT_DCB) &&
//          IsFileDeleted( IrpContext, Fcb->ParentDcb ))) {
//
//         return;
//     }
//
//     //
//     //  If we are not in the process of doing a verify,
//     //  first do a quick spot check on the Vcb.
//     //
//
//     if ( Fcb->Vcb->VerifyThread != KeGetCurrentThread() ) {
//
//         FatQuickVerifyVcb( IrpContext, Fcb->Vcb );
//     }
//
//     //
//     //  Now based on the condition of the Fcb we'll either return
//     //  immediately to the caller, raise a condition, or do some work
//     //  to verify the Fcb.
//     //
//
//     switch (Fcb->FcbCondition) {
//
//     case FcbGood:
//
//         DebugTrace(0, Dbg, "The Fcb is good\n", 0);
//         break;
//
//     case FcbBad:
//
//         FatRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         break;
//
//     case FcbNeedsToBeVerified:
//
//         //
//         //  We loop here checking our ancestors until we hit an Fcb which
//         //  is either good or bad.
//         //
//
//         CurrentFcb = Fcb;
//
//         while (CurrentFcb->FcbCondition == FcbNeedsToBeVerified) {
//
//             FatDetermineAndMarkFcbCondition(IrpContext, CurrentFcb);
//
//             //
//             //  If this Fcb didn't make it, or it was the Root Dcb, exit
//             //  the loop now, else continue with out parent.
//             //
//
//             if ( (CurrentFcb->FcbCondition != FcbGood) ||
//                  (NodeType(CurrentFcb) == FAT_NTC_ROOT_DCB) ) {
//
//                 break;
//             }
//
//             CurrentFcb = CurrentFcb->ParentDcb;
//         }
//
//         //
//         //  Now we can just look at ourselves to see how we did.
//         //
//
//         if (Fcb->FcbCondition != FcbGood) {
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         }
//
//         break;
//
//     default:
//
//         DebugDump("Invalid FcbCondition\n", 0, Fcb);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( Fcb->FcbCondition, 0, 0 );
//     }
//
//     DebugTrace(-1, Dbg, "FatVerifyFcb -> VOID\n", 0);
//
//     return;
// }
//
//
// VOID
// NTAPI
// FatDeferredCleanVolume (
//      PVOID Parameter
//     )
//
// /*++
//
// Routine Description:
//
//     This is the routine that performs the actual FatMarkVolumeClean call.
//     It assures that the target volume still exists as there ia a race
//     condition between queueing the ExWorker item and volumes going away.
//
// Arguments:
//
//     Parameter - Points to a clean volume packet that was allocated from pool
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PCLEAN_AND_DIRTY_VOLUME_PACKET Packet;
//     PLIST_ENTRY Links;
//     PVCB Vcb;
//     IRP_CONTEXT IrpContext;
//     BOOLEAN VcbExists = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeferredCleanVolume\n", 0);
//
//     Packet = (PCLEAN_AND_DIRTY_VOLUME_PACKET)Parameter;
//
//     Vcb = Packet->Vcb;
//
//     //
//     //  Make us appear as a top level FSP request so that we will
//     //  receive any errors from the operation.
//     //
//
//     IoSetTopLevelIrp( (PIRP)FSRTL_FSP_TOP_LEVEL_IRP );
//
//     //
//     //  Dummy up and Irp Context so we can call our worker routines
//     //
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT));
//
//     SetFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     //
//     //  Acquire shared access to the global lock and make sure this volume
//     //  still exists.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//     FatAcquireSharedGlobal( &IrpContext );
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     for (Links = FatData.VcbQueue.Flink;
//          Links != &FatData.VcbQueue;
//          Links = Links->Flink) {
//
//         PVCB ExistingVcb;
//
//         ExistingVcb = CONTAINING_RECORD(Links, VCB, VcbLinks);
//
//         if ( Vcb == ExistingVcb ) {
//
//             VcbExists = TRUE;
//             break;
//         }
//     }
//
//     //
//     //  If the vcb is good then mark it clean.  Ignore any problems.
//     //
//
//     if ( VcbExists &&
//          (Vcb->VcbCondition == VcbGood) &&
//          !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_SHUTDOWN) ) {
//
//         _SEH2_TRY {
//
//             if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                 FatMarkVolume( &IrpContext, Vcb, VolumeClean );
//             }
//
//             //
//             //  Check for a pathological race condition, and fix it.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY)) {
//
//                 FatMarkVolume( &IrpContext, Vcb, VolumeDirty );
//
//             } else {
//
//                 //
//                 //  Unlock the volume if it is removable.
//                 //
//
//                 if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//                     !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE)) {
//
//                     FatToggleMediaEjectDisable( &IrpContext, Vcb, FALSE );
//                 }
//             }
//
//         } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                   EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//               NOTHING;
//         } _SEH2_END;
//     }
//
//     //
//     //  Release the global resource, unpin and repinned Bcbs and return.
//     //
//
//     FatReleaseGlobal( &IrpContext );
//
//     _SEH2_TRY {
//
//         FatUnpinRepinnedBcbs( &IrpContext );
//
//     } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//               EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//           NOTHING;
//     } _SEH2_END;
//
//     IoSetTopLevelIrp( NULL );
//
//     //
//     //  and finally free the packet.
//     //
//
//     ExFreePool( Packet );
//
//     return;
// }
//
//
//
// VOID
// NTAPI
// FatCleanVolumeDpc (
//      PKDPC Dpc,
//      PVOID DeferredContext,
//      PVOID SystemArgument1,
//      PVOID SystemArgument2
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is dispatched 5 seconds after the last disk structure was
//     modified in a specific volume, and exqueues an execuative worker thread
//     to perform the actual task of marking the volume dirty.
//
// Arguments:
//
//     DefferedContext - Contains the Vcb to process.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PVCB Vcb;
//     PCLEAN_AND_DIRTY_VOLUME_PACKET Packet;
//
//     UNREFERENCED_PARAMETER( SystemArgument1 );
//     UNREFERENCED_PARAMETER( SystemArgument2 );
//     UNREFERENCED_PARAMETER( Dpc );
//
//     Vcb = (PVCB)DeferredContext;
//
//
//     //
//     //  If there is still dirty data (highly unlikely), set the timer for a
//     //  second in the future.
//     //
//
//     if (CcIsThereDirtyData(Vcb->Vpb)) {
//
//         LARGE_INTEGER TwoSecondsFromNow;
//
//         TwoSecondsFromNow.QuadPart = (LONG)-2*1000*1000*10;
//
//         KeSetTimer( &Vcb->CleanVolumeTimer,
//                     TwoSecondsFromNow,
//                     &Vcb->CleanVolumeDpc );
//
//         return;
//     }
//
//     //
//     //  If we couldn't get pool, oh well....
//     //
//
//     Packet = ExAllocatePoolWithTag(NonPagedPoolNx, sizeof(CLEAN_AND_DIRTY_VOLUME_PACKET), ' taF');
//
//     if ( Packet ) {
//
//         Packet->Vcb = Vcb;
//         Packet->Irp = NULL;
//
//         //
//         //  Clear the dirty flag now since we cannot synchronize after this point.
//         //
//
//         ClearFlag( Packet->Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//
//         ExInitializeWorkItem( &Packet->Item, &FatDeferredCleanVolume, Packet );
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "prefast indicates this is an obsolete API, but it is ok for fastfat to keep using it" )
// #endif
//         ExQueueWorkItem( &Packet->Item, CriticalWorkQueue );
//     }
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatMarkVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN FAT_VOLUME_STATE VolumeState
//     )
//
// /*++
//
// Routine Description:
//
//     This routine moves the physically marked volume state between the clean
//     and dirty states.  For compatibility with Win9x, we manipulate both the
//     historical DOS (on==clean in index 1 of the FAT) and NT (on==dirty in
//     the CurrentHead field of the BPB) dirty bits.
//
// Arguments:
//
//     Vcb - Supplies the Vcb being modified
//
//     VolumeState - Supplies the state the volume is transitioning to
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PCHAR Sector;
//     PBCB Bcb = NULL;
//     KEVENT Event;
//     PIRP Irp = NULL;
//     NTSTATUS Status;
//     BOOLEAN FsInfoUpdate = FALSE;
//     ULONG FsInfoOffset = 0;
//     ULONG ThisPass;
//     LARGE_INTEGER Offset;
//     BOOLEAN abort = FALSE;
//
//     DebugTrace(+1, Dbg, "FatMarkVolume, Vcb = %p\n", Vcb);
//
//     //
//     //  We had best not be trying to scribble dirty/clean bits if the
//     //  volume is write protected.  The responsibility lies with the
//     //  callers to make sure that operations that could cause a state
//     //  change cannot happen.  There are a few, though, that show it
//     //  just doesn't make sense to force everyone to do the dinky
//     //  check.
//     //
//
//     //
//     //  If we were called for FAT12 or readonly media, return immediately.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) ||
//         FatIsFat12( Vcb )) {
//
//         return;
//     }
//
//     //
//     //  We have two possible additional tasks to do to mark a volume
//     //
//     //      Pass 0) Flip the dirty bit in the Bpb
//     //      Pass 1) Rewrite the FsInfo sector for FAT32 if needed
//     //
//     //  In most cases we can collapse these two either because the volume
//     //  is either not FAT32 or the FsInfo sector is adjacent to the boot sector.
//     //
//
//     for (ThisPass = 0; ThisPass < 2; ThisPass++) {
//
//         //
//         //  If this volume is being dirtied, or isn't FAT32, or if it is and
//         //  we were able to perform the fast update, or the bpb lied to us
//         //  about where the FsInfo went, we're done - no FsInfo to update in
//         //  a seperate write.
//         //
//
//         if (ThisPass == 1 && (!FatIsFat32( Vcb ) ||
//                               VolumeState != VolumeClean ||
//                               FsInfoUpdate ||
//                               Vcb->Bpb.FsInfoSector == 0)) {
//
//             break;
//         }
//
//         //
//         //  Bail if we get an IO error.
//         //
//
//         _SEH2_TRY {
//
//             ULONG PinLength;
//             ULONG WriteLength;
//
//             //
//             // If the FAT table is 12-bit then our strategy is to pin the entire
//             // thing when any of it is modified.  Here we're going to pin the
//             // first page, so in the 12-bit case we also want to pin the rest
//             // of the FAT table.
//             //
//
//             Offset.QuadPart = 0;
//
//             if (Vcb->AllocationSupport.FatIndexBitSize == 12) {
//
//                 //
//                 //  But we only write back the first sector.
//                 //
//
//                 PinLength = FatReservedBytes(&Vcb->Bpb) + FatBytesPerFat(&Vcb->Bpb);
//                 WriteLength = Vcb->Bpb.BytesPerSector;
//
//             } else {
//
//                 WriteLength = PinLength = Vcb->Bpb.BytesPerSector;
//
//                 //
//                 //  If this is a FAT32 volume going into the clean state,
//                 //  see about doing the FsInfo sector.
//                 //
//
//                 if (FatIsFat32( Vcb ) && VolumeState == VolumeClean) {
//
//                     //
//                     //  If the FsInfo sector immediately follows the boot sector,
//                     //  we can do this in a single operation by rewriting both
//                     //  sectors at once.
//                     //
//
//                     if (Vcb->Bpb.FsInfoSector == 1) {
//
//                         NT_ASSERT( ThisPass == 0 );
//
//                         FsInfoUpdate = TRUE;
//                         FsInfoOffset = Vcb->Bpb.BytesPerSector;
//                         WriteLength = PinLength = Vcb->Bpb.BytesPerSector * 2;
//
//                     } else if (ThisPass == 1) {
//
//                         //
//                         //  We are doing an explicit write to the FsInfo sector.
//                         //
//
//                         FsInfoUpdate = TRUE;
//                         FsInfoOffset = 0;
//
//                         Offset.QuadPart = Vcb->Bpb.BytesPerSector * Vcb->Bpb.FsInfoSector;
//                     }
//                 }
//             }
//
//             //
//             //  Call Cc directly here so that we can avoid overhead and push this
//             //  right down to the disk.
//             //
//
//             CcPinRead( Vcb->VirtualVolumeFile,
//                        &Offset,
//                        PinLength,
//                        TRUE,
//                        &Bcb,
//                        (PVOID *)&Sector );
//
//             DbgDoit( IrpContext->PinCount += 1 )
//
//             //
//             //  Set the Bpb on Pass 0 always
//             //
//
//             if (ThisPass == 0) {
//
//                 PCHAR CurrentHead;
//
//                 //
//                 //  Before we do anything, doublecheck that this still looks like a
//                 //  FAT bootsector.  If it doesn't, something remarkable happened
//                 //  and we should avoid touching the volume.
//                 //
//                 //  THIS IS TEMPORARY (but may last a while)
//                 //
//
//                 if (!FatIsBootSectorFat( (PPACKED_BOOT_SECTOR) Sector )) {
//                     abort = TRUE;
//                     _SEH2_LEAVE;
//                 }
//
//                 if (FatIsFat32( Vcb )) {
//
//                     CurrentHead = (PCHAR)&((PPACKED_BOOT_SECTOR_EX) Sector)->CurrentHead;
//
//                 } else {
//
//                     CurrentHead = (PCHAR)&((PPACKED_BOOT_SECTOR) Sector)->CurrentHead;
//                 }
//
//                 if (VolumeState == VolumeClean) {
//
//                     ClearFlag( *CurrentHead, FAT_BOOT_SECTOR_DIRTY );
//
//                 } else {
//
//                     SetFlag( *CurrentHead, FAT_BOOT_SECTOR_DIRTY );
//
//                     //
//                     //  In addition, if this request received an error that may indicate
//                     //  media corruption, have autochk perform a surface test.
//                     //
//
//                     if ( VolumeState == VolumeDirtyWithSurfaceTest ) {
//
//                         SetFlag( *CurrentHead, FAT_BOOT_SECTOR_TEST_SURFACE );
//                     }
//                 }
//             }
//
//             //
//             //  Update the FsInfo as appropriate.
//             //
//
//             if (FsInfoUpdate) {
//
//                 PFSINFO_SECTOR FsInfoSector = (PFSINFO_SECTOR) ((PCHAR)Sector + FsInfoOffset);
//
//                 //
//                 //  We just rewrite all of the spec'd fields.  Note that we don't
//                 //  care to synchronize with the allocation package - this will be
//                 //  quickly taken care of by a re-dirtying of the volume if a change
//                 //  is racing with us.  Remember that this is all a compatibility
//                 //  deference for Win9x FAT32 - NT will never look at this information.
//                 //
//
//                 FsInfoSector->SectorBeginSignature = FSINFO_SECTOR_BEGIN_SIGNATURE;
//                 FsInfoSector->FsInfoSignature = FSINFO_SIGNATURE;
//                 FsInfoSector->FreeClusterCount = Vcb->AllocationSupport.NumberOfFreeClusters;
//                 FsInfoSector->NextFreeCluster = Vcb->ClusterHint;
//                 FsInfoSector->SectorEndSignature = FSINFO_SECTOR_END_SIGNATURE;
//             }
//
//             //
//             //  Initialize the event we're going to use
//             //
//
//             KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//             //
//             //  Build the irp for the operation and also set the override flag.
//             //  Note that we may be at APC level, so do this asyncrhonously and
//             //  use an event for synchronization as normal request completion
//             //  cannot occur at APC level.
//             //
//
//             Irp = IoBuildAsynchronousFsdRequest( IRP_MJ_WRITE,
//                                                  Vcb->TargetDeviceObject,
//                                                  (PVOID)Sector,
//                                                  WriteLength,
//                                                  &Offset,
//                                                  NULL );
//
//             if ( Irp == NULL ) {
//
//                 try_return(NOTHING);
//             }
//
//             //
//             //  Make this operation write-through.  It never hurts to try to be
//             //  safer about this, even though we aren't logged.
//             //
//
//             SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_WRITE_THROUGH );
//
//             //
//             //  Set up the completion routine
//             //
//
//             IoSetCompletionRoutine( Irp,
//                                     FatMarkVolumeCompletionRoutine,
//                                     &Event,
//                                     TRUE,
//                                     TRUE,
//                                     TRUE );
//
//             //
//             //  Call the device to do the write and wait for it to finish.
//             //  Igmore any return status.
//             //
//
//             Status = IoCallDriver( Vcb->TargetDeviceObject, Irp );
//
//             if (Status == STATUS_PENDING) {
//
//                 (VOID)KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, (PLARGE_INTEGER)NULL );
//             }
//
//         try_exit: NOTHING;
//         } _SEH2_FINALLY {
//
//             //
//             //  Clean up the Irp and Mdl
//             //
//
//
//             if (Irp) {
//
//                 //
//                 //  If there is an MDL (or MDLs) associated with this I/O
//                 //  request, Free it (them) here.  This is accomplished by
//                 //  walking the MDL list hanging off of the IRP and deallocating
//                 //  each MDL encountered.
//                 //
//
//                 while (Irp->MdlAddress != NULL) {
//
//                     PMDL NextMdl;
//
//                     NextMdl = Irp->MdlAddress->Next;
//
//                     MmUnlockPages( Irp->MdlAddress );
//
//                     IoFreeMdl( Irp->MdlAddress );
//
//                     Irp->MdlAddress = NextMdl;
//                 }
//
//                 IoFreeIrp( Irp );
//             }
//
//             if (Bcb != NULL) {
//
//                 FatUnpinBcb( IrpContext, Bcb );
//             }
//         } _SEH2_END;
//     }
//
//     if (!abort) {
//
//         //
//         //  Flip the dirty bit in the FAT
//         //
//
//         if (VolumeState == VolumeDirty) {
//
//            FatSetFatEntry( IrpContext, Vcb, FAT_DIRTY_BIT_INDEX, FAT_DIRTY_VOLUME);
//
//         } else {
//
//            FatSetFatEntry( IrpContext, Vcb, FAT_DIRTY_BIT_INDEX, FAT_CLEAN_VOLUME);
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatMarkVolume -> VOID\n", 0);
//
//     return;
// }
//
//
// VOID
// NTAPI
// FatFspMarkVolumeDirtyWithRecover(
//     PVOID Parameter
//     )
//
// /*++
//
// Routine Description:
//
//     This is the routine that performs the actual FatMarkVolume Dirty call
//     on a paging file Io that encounters a media error.  It is responsible
//     for completing the PagingIo Irp as soon as this is done.
//
//     Note:  this routine (and thus FatMarkVolume()) must be resident as
//            the paging file might be damaged at this point.
//
// Arguments:
//
//     Parameter - Points to a dirty volume packet that was allocated from pool
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PCLEAN_AND_DIRTY_VOLUME_PACKET Packet;
//     PVCB Vcb;
//     IRP_CONTEXT IrpContext;
//     PIRP Irp;
//
//     DebugTrace(+1, Dbg, "FatFspMarkVolumeDirtyWithRecover\n", 0);
//
//     Packet = (PCLEAN_AND_DIRTY_VOLUME_PACKET)Parameter;
//
//     Vcb = Packet->Vcb;
//     Irp = Packet->Irp;
//
//     //
//     //  Dummy up the IrpContext so we can call our worker routines
//     //
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT));
//
//     SetFlag(IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT);
//     IrpContext.OriginatingIrp = Irp;
//
//     //
//     //  Make us appear as a top level FSP request so that we will
//     //  receive any errors from the operation.
//     //
//
//     IoSetTopLevelIrp( (PIRP)FSRTL_FSP_TOP_LEVEL_IRP );
//
//     //
//     //  Try to write out the dirty bit.  If something goes wrong, we
//     //  tried.
//     //
//
//     _SEH2_TRY {
//
//         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY );
//
//         FatMarkVolume( &IrpContext, Vcb, VolumeDirtyWithSurfaceTest );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( &IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         NOTHING;
//     } _SEH2_END;
//
//     IoSetTopLevelIrp( NULL );
//
//     //
//     //  Now complete the originating Irp or set the synchronous event.
//     //
//
//     if (Packet->Event) {
//         KeSetEvent( Packet->Event, 0, FALSE );
//     } else {
//         IoCompleteRequest( Irp, IO_DISK_INCREMENT );
//     }
//
//     DebugTrace(-1, Dbg, "FatFspMarkVolumeDirtyWithRecover -> VOID\n", 0);
// }
//
//
// VOID
// FatCheckDirtyBit (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine looks at the volume dirty bit, and depending on the state of
//     VCB_STATE_FLAG_MOUNTED_DIRTY, the appropriate action is taken.
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     BOOLEAN Dirty;
//
//     PPACKED_BOOT_SECTOR BootSector;
//     PBCB BootSectorBcb;
//
//     UNICODE_STRING VolumeLabel;
//
//     PAGED_CODE();
//
//     //
//     //  Look in the boot sector
//     //
//
//     FatReadVolumeFile( IrpContext,
//                        Vcb,
//                        0,
//                        sizeof(PACKED_BOOT_SECTOR),
//                        &BootSectorBcb,
//                        (PVOID *)&BootSector );
//
//     _SEH2_TRY {
//
//         //
//         //  Check if the magic bit is set
//         //
//
//         if (IsBpbFat32(&BootSector->PackedBpb)) {
//             Dirty = BooleanFlagOn( ((PPACKED_BOOT_SECTOR_EX)BootSector)->CurrentHead,
//                                    FAT_BOOT_SECTOR_DIRTY );
//         } else {
//             Dirty = BooleanFlagOn( BootSector->CurrentHead, FAT_BOOT_SECTOR_DIRTY );
//         }
//
//         //
//         //  Setup the VolumeLabel string
//         //
//
//         VolumeLabel.Length = Vcb->Vpb->VolumeLabelLength;
//         VolumeLabel.MaximumLength = MAXIMUM_VOLUME_LABEL_LENGTH;
//         VolumeLabel.Buffer = &Vcb->Vpb->VolumeLabel[0];
//
//         if ( Dirty ) {
//
//             //
//             //  Do not trigger the mounted dirty bit if this is a verify
//             //  and the volume is a boot or paging device.  We know that
//             //  a boot or paging device cannot leave the system, and thus
//             //  that on its mount we will have figured this out correctly.
//             //
//             //  This logic is a reasonable change.  Why?
//             //  'cause setup cracked a non-exclusive DASD handle near the
//             //  end of setup, wrote some data, closed the handle and we
//             //  set the verify bit ... came back around and saw that other
//             //  arbitrary activity had left the volume in a temporarily dirty
//             //  state.
//             //
//             //  Of course, the real problem is that we don't have a journal.
//             //
//
//             if (!(IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL &&
//                   IrpContext->MinorFunction == IRP_MN_VERIFY_VOLUME &&
//                   FlagOn( Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE))) {
//
//                 KdPrintEx((DPFLTR_FASTFAT_ID,
//                            DPFLTR_INFO_LEVEL,
//                            "FASTFAT: WARNING! Mounting Dirty Volume %Z\n",
//                            &VolumeLabel));
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY );
//             }
//
//         } else {
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                 KdPrintEx((DPFLTR_FASTFAT_ID,
//                            DPFLTR_INFO_LEVEL,
//                            "FASTFAT: Volume %Z has been cleaned.\n",
//                            &VolumeLabel));
//
//                 ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY );
//
//             } else {
//
//                 (VOID)FsRtlBalanceReads( Vcb->TargetDeviceObject );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, BootSectorBcb );
//     } _SEH2_END;
// }
//
//
// VOID
// FatVerifyOperationIsLegal (
//     IN PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines is the requested operation should be allowed to
//     continue.  It either returns to the user if the request is Okay, or
//     raises an appropriate status.
//
// Arguments:
//
//     Irp - Supplies the Irp to check
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIRP Irp;
//     PFILE_OBJECT FileObject;
//
//     PAGED_CODE();
//
//     Irp = IrpContext->OriginatingIrp;
//
//     //
//     //  If the Irp is not present, then we got here via close.
//     //
//     //
//
//     if ( Irp == NULL ) {
//
//         return;
//     }
//
//     FileObject = IoGetCurrentIrpStackLocation(Irp)->FileObject;
//
//     //
//     //  If there is not a file object, we cannot continue.
//     //
//
//     if ( FileObject == NULL ) {
//
//         return;
//     }
//
//     //
//     //  If the file object has already been cleaned up, and
//     //
//     //  A) This request is a paging io read or write, or
//     //  B) This request is a close operation, or
//     //  C) This request is a set or query info call (for Lou)
//     //  D) This is an MDL complete
//     //
//     //  let it pass, otherwise return STATUS_FILE_CLOSED.
//     //
//
//     if ( FlagOn(FileObject->Flags, FO_CLEANUP_COMPLETE) ) {
//
//         PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//         if ( (FlagOn(Irp->Flags, IRP_PAGING_IO)) ||
//              (IrpSp->MajorFunction == IRP_MJ_CLOSE ) ||
//              (IrpSp->MajorFunction == IRP_MJ_SET_INFORMATION) ||
//              (IrpSp->MajorFunction == IRP_MJ_QUERY_INFORMATION) ||
//              ( ( (IrpSp->MajorFunction == IRP_MJ_READ) ||
//                  (IrpSp->MajorFunction == IRP_MJ_WRITE) ) &&
//                FlagOn(IrpSp->MinorFunction, IRP_MN_COMPLETE) ) ) {
//
//             NOTHING;
//
//         } else {
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_CLOSED );
//         }
//     }
//
//     return;
// }
//
//
//
// //
// //  Internal support routine
// //
//
// VOID
// FatResetFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when an Fcb has been marked as needs to be verified.
//
//     It does the following tasks:
//
//         - Reset Mcb mapping information
//         - For directories, reset dirent hints
//         - Set allocation size to unknown
//
// Arguments:
//
//     Fcb - Supplies the Fcb to reset
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     LOGICAL IsRealPagingFile;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Don't do the two following operations for the Root Dcb
//     //  of a non FAT32 volume or paging files.  Paging files!?
//     //  Yes, if someone diddles a volume we try to reverify all
//     //  of the Fcbs just in case; however, there is no safe way
//     //  to chuck and retrieve the mapping pair information for
//     //  a real paging file. Lose it and die.
//     //
//     //  An exception is made for ReadyBoost cache files, which
//     //  are created as paging files on removable devices and
//     //  require validation after a power transition.
//     //
//
//     if (!FlagOn(Fcb->Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//         FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE)) {
//
//         IsRealPagingFile = TRUE;
//
//     } else {
//
//         IsRealPagingFile = FALSE;
//     }
//
//     if ( (NodeType(Fcb) != FAT_NTC_ROOT_DCB ||
//           FatIsFat32( Fcb->Vcb ))  &&
//          !IsRealPagingFile ) {
//
//         //
//         //  Reset the mcb mapping.
//         //
//
//         FsRtlRemoveLargeMcbEntry( &Fcb->Mcb, 0, 0xFFFFFFFF );
//
//         //
//         //  Reset the allocation size to 0 or unknown
//         //
//
//         if ( Fcb->FirstClusterOfFile == 0 ) {
//
//             Fcb->Header.AllocationSize.QuadPart = 0;
//
//         } else {
//
//             Fcb->Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
//         }
//     }
//
//     //
//     //  If this is a directory, reset the hints.
//     //
//
//     if ( (NodeType(Fcb) == FAT_NTC_DCB) ||
//          (NodeType(Fcb) == FAT_NTC_ROOT_DCB) ) {
//
//         //
//         //  Force a rescan of the directory
//         //
//
//         Fcb->Specific.Dcb.UnusedDirentVbo = 0xffffffff;
//         Fcb->Specific.Dcb.DeletedDirentHint = 0xffffffff;
//     }
// }
//
//
//
// BOOLEAN
// FatMatchFileSize (
//     __in PIRP_CONTEXT IrpContext,
//     __in PDIRENT Dirent,
//     __in PFCB Fcb
//     )
// {
//
//     UNREFERENCED_PARAMETER(IrpContext);
//
//     if (NodeType(Fcb) != FAT_NTC_FCB) {
//         return TRUE;
//     }
//
//
//         if (Fcb->Header.FileSize.LowPart != Dirent->FileSize) {
//             return FALSE;
//         }
//
//
//     return TRUE;
// }
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDetermineAndMarkFcbCondition (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks a specific Fcb to see if it is different from what's
//     on the disk.  The following things are checked:
//
//         - File Name
//         - File Size (if not directory)
//         - First Cluster Of File
//         - Dirent Attributes
//
// Arguments:
//
//     Fcb - Supplies the Fcb to examine
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PDIRENT Dirent;
//     PBCB DirentBcb;
//     ULONG FirstClusterOfFile;
//
//     OEM_STRING Name;
//     CHAR Buffer[16];
//
//     PAGED_CODE();
//
//     //
//     //  If this is the Root Dcb, special case it.  That is, we know
//     //  by definition that it is good since it is fixed in the volume
//     //  structure.
//     //
//
//     if ( NodeType(Fcb) == FAT_NTC_ROOT_DCB ) {
//
//         FatMarkFcbCondition( IrpContext, Fcb, FcbGood, FALSE );
//
//         return;
//     }
//
//     //  The first thing we need to do to verify ourselves is
//     //  locate the dirent on the disk.
//     //
//
//     FatGetDirentFromFcbOrDcb( IrpContext,
//                               Fcb,
//                               TRUE,
//                               &Dirent,
//                               &DirentBcb );
//     //
//     //  If we couldn't get the dirent, this fcb must be bad (case of
//     //  enclosing directory shrinking during the time it was ejected).
//     //
//
//     if (DirentBcb == NULL) {
//
//         FatMarkFcbCondition( IrpContext, Fcb, FcbBad, FALSE );
//
//         return;
//     }
//
//     //
//     //  We located the dirent for ourselves now make sure it
//     //  is really ours by comparing the Name and FatFlags.
//     //  Then for a file we also check the file size.
//     //
//     //  Note that we have to unpin the Bcb before calling FatResetFcb
//     //  in order to avoid a deadlock in CcUninitializeCacheMap.
//     //
//
//     _SEH2_TRY {
//
//         Name.MaximumLength = 16;
//         Name.Buffer = &Buffer[0];
//
//         Fat8dot3ToString( IrpContext, Dirent, FALSE, &Name );
//
//         //
//         //  We need to calculate the first cluster 'cause FAT32 splits
//         //  this field across the dirent.
//         //
//
//         FirstClusterOfFile = Dirent->FirstClusterOfFile;
//
//         if (FatIsFat32( Fcb->Vcb )) {
//
//             FirstClusterOfFile += Dirent->FirstClusterOfFileHi << 16;
//         }
//
//         if (!RtlEqualString( &Name, &Fcb->ShortName.Name.Oem, TRUE )
//
//                 ||
//
//              !FatMatchFileSize(IrpContext, Dirent, Fcb )
//
//                 ||
//
//              (FirstClusterOfFile != Fcb->FirstClusterOfFile)
//
//                 ||
//
//               (Dirent->Attributes != Fcb->DirentFatFlags) ) {
//
//             FatMarkFcbCondition( IrpContext, Fcb, FcbBad, FALSE );
//
//         } else {
//
//             //
//             //  We passed.  Get the Fcb ready to use again.
//             //
//
//             FatMarkFcbCondition( IrpContext, Fcb, FcbGood, FALSE );
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//     } _SEH2_END;
//
//     return;
// }
//
//
//
// //
// //  Internal support routine
// //
//
// VOID
// FatQuickVerifyVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routines just checks the verify bit in the real device and the
//     Vcb condition and raises an appropriate exception if so warented.
//     It is called when verifying both Fcbs and Vcbs.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to check the condition of.
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
//     //
//     //  If the real device needs to be verified we'll set the
//     //  DeviceToVerify to be our real device and raise VerifyRequired.
//     //
//
//     if (FlagOn(Vcb->Vpb->RealDevice->Flags, DO_VERIFY_VOLUME)) {
//
//         DebugTrace(0, Dbg, "The Vcb needs to be verified\n", 0);
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                       Vcb->Vpb->RealDevice );
//
//         FatRaiseStatus( IrpContext, STATUS_VERIFY_REQUIRED );
//     }
//
//     //
//     //  Based on the condition of the Vcb we'll either return to our
//     //  caller or raise an error condition
//     //
//
//     switch (Vcb->VcbCondition) {
//
//     case VcbGood:
//
//         DebugTrace(0, Dbg, "The Vcb is good\n", 0);
//
//         //
//         //  Do a check here of an operation that would try to modify a
//         //  write protected media.
//         //
//
//         if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) &&
//             ((IrpContext->MajorFunction == IRP_MJ_WRITE) ||
//              (IrpContext->MajorFunction == IRP_MJ_SET_INFORMATION) ||
//              (IrpContext->MajorFunction == IRP_MJ_SET_EA) ||
//              (IrpContext->MajorFunction == IRP_MJ_FLUSH_BUFFERS) ||
//              (IrpContext->MajorFunction == IRP_MJ_SET_VOLUME_INFORMATION) ||
//              (IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL &&
//               IrpContext->MinorFunction == IRP_MN_USER_FS_REQUEST &&
//               IoGetCurrentIrpStackLocation(IrpContext->OriginatingIrp)->Parameters.FileSystemControl.FsControlCode ==
//                 FSCTL_MARK_VOLUME_DIRTY))) {
//
//             //
//             //  Set the real device for the pop-up info, and set the verify
//             //  bit in the device object, so that we will force a verify
//             //  in case the user put the correct media back in.
//             //
//
//
//             IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                           Vcb->Vpb->RealDevice );
//
//             FatMarkDevForVerifyIfVcbMounted(Vcb);
//
//             FatRaiseStatus( IrpContext, STATUS_MEDIA_WRITE_PROTECTED );
//         }
//
//         break;
//
//     case VcbNotMounted:
//
//         DebugTrace(0, Dbg, "The Vcb is not mounted\n", 0);
//
//         //
//         //  Set the real device for the pop-up info, and set the verify
//         //  bit in the device object, so that we will force a verify
//         //  in case the user put the correct media back in.
//         //
//
//         IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                       Vcb->Vpb->RealDevice );
//
//         FatRaiseStatus( IrpContext, STATUS_WRONG_VOLUME );
//
//         break;
//
//     case VcbBad:
//
//         DebugTrace(0, Dbg, "The Vcb is bad\n", 0);
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED )) {
//
//             FatRaiseStatus( IrpContext, STATUS_VOLUME_DISMOUNTED );
//
//         } else {
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_INVALID );
//         }
//         break;
//
//     default:
//
//         DebugDump("Invalid VcbCondition\n", 0, Vcb);
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( Vcb->VcbCondition, 0, 0 );
//     }
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatPerformVerify (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PDEVICE_OBJECT Device
//     )
//
// /*++
//
// Routine Description:
//
//     This routines performs an IoVerifyVolume operation and takes the
//     appropriate action.  After the Verify is complete the originating
//     Irp is sent off to an Ex Worker Thread.  This routine is called
//     from the exception handler.
//
// Arguments:
//
//     Irp - The irp to send off after all is well and done.
//
//     Device - The real device needing verification.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PVCB Vcb;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//     PFILE_OBJECT FileObject = IoGetCurrentIrpStackLocation(Irp)->FileObject;
//     BOOLEAN AllowRawMount = FALSE;
//     BOOLEAN VcbDeleted = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Check if this Irp has a status of Verify required and if it does
//     //  then call the I/O system to do a verify.
//     //
//     //  Skip the IoVerifyVolume if this is a mount or verify request
//     //  itself.  Trying a recursive mount will cause a deadlock with
//     //  the DeviceObject->DeviceLock.
//     //
//
//     if ( (IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) &&
//          ((IrpContext->MinorFunction == IRP_MN_MOUNT_VOLUME) ||
//           (IrpContext->MinorFunction == IRP_MN_VERIFY_VOLUME)) ) {
//
//         return FatFsdPostRequest( IrpContext, Irp );
//     }
//
//     DebugTrace(0, Dbg, "Verify Required, DeviceObject = %p\n", Device);
//
//     //
//     //  Extract a pointer to the Vcb from the VolumeDeviceObject.
//     //  Note that since we have specifically excluded mount,
//     //  requests, we know that IrpSp->DeviceObject is indeed a
//     //  volume device object.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation(Irp);
//
//     Vcb = &CONTAINING_RECORD( IrpSp->DeviceObject,
//                               VOLUME_DEVICE_OBJECT,
//                               DeviceObject )->Vcb;
//
//     //
//     //  Check if the volume still thinks it needs to be verified,
//     //  if it doesn't then we can skip doing a verify because someone
//     //  else beat us to it.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  We will allow Raw to mount this volume if we were doing a
//         //  a DASD open.
//         //
//
//         if ( (IrpContext->MajorFunction == IRP_MJ_CREATE) &&
//              (IrpSp->FileObject->FileName.Length == 0) &&
//              (IrpSp->FileObject->RelatedFileObject == NULL) ) {
//
//             AllowRawMount = TRUE;
//         }
//
//         //
//         //  Send down the verify.  This could be going to a different
//         //  filesystem.
//         //
//
//         Status = IoVerifyVolume( Device, AllowRawMount );
//
//         //
//         //  If the verify operation completed it will return
//         //  either STATUS_SUCCESS or STATUS_WRONG_VOLUME, exactly.
//         //
//         //  If FatVerifyVolume encountered an error during
//         //  processing, it will return that error.  If we got
//         //  STATUS_WRONG_VOLUME from the verfy, and our volume
//         //  is now mounted, commute the status to STATUS_SUCCESS.
//         //
//         //  Acquire the Vcb so we're working with a stable Vcb condition.
//         //
//
//         FatAcquireSharedVcb(IrpContext, Vcb);
//
//         if ( (Status == STATUS_WRONG_VOLUME) &&
//              (Vcb->VcbCondition == VcbGood) ) {
//
//             Status = STATUS_SUCCESS;
//         }
//         else if ((STATUS_SUCCESS == Status) && (Vcb->VcbCondition != VcbGood)) {
//
//             Status = STATUS_WRONG_VOLUME;
//         }
//
//         //
//         //  Do a quick unprotected check here.  The routine will do
//         //  a safe check.  After here we can release the resource.
//         //  Note that if the volume really went away, we will be taking
//         //  the Reparse path.
//         //
//
//         if ((VcbGood != Vcb->VcbCondition) &&
//             (0 == Vcb->OpenFileCount) ) {
//
//             FatReleaseVcb( IrpContext, Vcb);
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193 )
// #endif
//             FatAcquireExclusiveGlobal( IrpContext );
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//             FatAcquireExclusiveVcb( IrpContext,
//                                     Vcb );
//
//             VcbDeleted = FatCheckForDismount( IrpContext,
//                                               Vcb,
//                                               FALSE );
//
//             if (!VcbDeleted) {
//
//                 FatReleaseVcb( IrpContext,
//                                Vcb );
//             }
//
//             FatReleaseGlobal( IrpContext );
//         }
//         else {
//
//             FatReleaseVcb( IrpContext, Vcb);
//         }
//
//         //
//         //  If the IopMount in IoVerifyVolume did something, and
//         //  this is an absolute open, force a reparse.
//         //
//
//         if ((IrpContext->MajorFunction == IRP_MJ_CREATE) &&
//             (FileObject->RelatedFileObject == NULL) &&
//             ((Status == STATUS_SUCCESS) || (Status == STATUS_WRONG_VOLUME))) {
//
//             Irp->IoStatus.Information = IO_REMOUNT;
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_REPARSE );
//             Status = STATUS_REPARSE;
//             Irp = NULL;
//         }
//
//         if ( (Irp != NULL) && !NT_SUCCESS(Status) ) {
//
//             //
//             //  Fill in the device object if required.
//             //
//
//             if ( IoIsErrorUserInduced( Status ) ) {
//
//                 IoSetHardErrorOrVerifyDevice( Irp, Device );
//             }
//
//             NT_ASSERT( STATUS_VERIFY_REQUIRED != Status);
//
//             FatNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//
//         //
//         //  If there is still an Irp, send it off to an Ex Worker thread.
//         //
//
//         if ( Irp != NULL ) {
//
//             Status = FatFsdPostRequest( IrpContext, Irp );
//         }
//
//     }
//     _SEH2_EXCEPT (FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the verify or raised
//         //  an error ourselves.  So we'll abort the I/O request with
//         //  the error status that we get back from the execption code.
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     return Status;
// }
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI
// FatMarkVolumeCompletionRoutine(
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
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
