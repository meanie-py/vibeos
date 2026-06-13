use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcWaitForCurrentLazyWriterActivity, DISK_GEOMETRY, GUID, _IO_STATUS_BLOCK, LUID, PCH, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PVPB, RtlOemStringToCountedUnicodeString, STRING};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, CDROM_TOC_LARGE, CdCheckForDismount, CdFreeDirCache, CdProcessToc, FCB, IRP_CONTEXT, IRP_CONTEXT_FLAG_FORCE_POST, PIRP_CONTEXT, PVCB, VCB, VCB_STATE_NOTIFY_REMOUNT, VOLUME_DEVICE_OBJECT, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FILE_LOCK, FILE_OBJECT, IO_STACK_LOCATION, IRP, POPLOCK, VPB, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlNotifyVolumeEvent};
use crate::glue::{CDFS_BUG_CHECK_FSCTRL, CDFS_RESIDUAL_REFERENCE, Ccb, FSRTL_VOLUME_MOUNT, FSRTL_VOLUME_UNLOCK, Fcb, IrpContext, SL_ALLOW_RAW_MOUNT, STATUS_NOT_LOCKED, STATUS_WRONG_VOLUME, TAG_VOL_DESC, Vcb};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdData};

extern "C" {
    static mut CdData: CdData;
}
extern "system" {
    fn CdPurgeVolume(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB, flag: u32);
    fn CdReleaseVcb(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB);
    fn CdAcquireVcbExclusive(irp_context: *mut IRP_CONTEXT, vcb: *mut VCB, flag: u32);
    fn CdFspClose(vcb: *mut VCB);
    fn IoAcquireVpbSpinLock(SpinLock: *mut u8);
    fn IoReleaseVpbSpinLock(Irql: u8);
    fn CdUpdateVcbCondition(Vcb: *mut VCB, Condition: u32);
    fn CdUpdateMediaChangeCount(Vcb: *mut VCB, MediaChangeCount: u32);
    fn CdDecodeFileObject( IrpContext: *mut IrpContext, FileObject: *mut FILE_OBJECT, Fcb: *mut *mut Fcb, Ccb: *mut *mut Ccb, ) -> /* UserFileOpen or enum */ i32; // return type unknown, assume i32 fn CdCompleteRequest(IrpContext: *mut IrpContext, Irp: *mut IRP, Status: NTSTATUS);
    fn SetFlag(Flags: *mut ULONG, Flag: ULONG);
    fn ClearFlag(Flags: *mut ULONG, Flag: ULONG);
    fn CdAcquireFcbExclusive(IrpContext: *mut IrpContext, Fcb: *mut Fcb, Wait: BOOLEAN);
    fn CdAcquireFcbShared(IrpContext: *mut IrpContext, Fcb: *mut Fcb, Wait: BOOLEAN);
    fn CdVerifyFcbOperation(IrpContext: *mut IrpContext, Fcb: *mut Fcb);
    fn CdGetFcbOplock(Fcb: *mut Fcb) -> POPLOCK;
    fn CdLockFcb(IrpContext: *mut IrpContext, Fcb: *mut Fcb);
    fn CdUnlockFcb(IrpContext: *mut IrpContext, Fcb: *mut Fcb);
    fn CdReleaseFcb(IrpContext: *mut IrpContext, Fcb: *mut Fcb);
    fn CdIsFastIoPossible(Fcb: *mut Fcb) -> BOOLEAN;
    fn FsRtlAreThereCurrentFileLocks(FileLock: *mut FILE_LOCK) -> BOOLEAN;
    fn FsRtlOplockFsctrl(Oplock: POPLOCK, Irp: PIRP, OpenCount: ULONG) -> NTSTATUS;
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    fn CdCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn CdVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn _SEH2_AbnormalTermination() -> BOOLEAN;
    fn PAGED_CODE();
    fn FsRtlDismountComplete(TargetDeviceObject: *mut DEVICE_OBJECT, Status: NTSTATUS);
    fn CdAcquireCdData(IrpContext: *mut IRP_CONTEXT);
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn CdReleaseCdData(IrpContext: *mut IRP_CONTEXT);
    static CdIsoId: u8;
    static CdHsgId: u8;
    fn CdRaiseStatus(IrpContext: *mut IrpContext, Status: NTSTATUS) -> !;
    fn CdPerformDevIoCtrl( IrpContext: *mut IrpContext, IoControlCode: u32, DeviceObject: *mut DEVICE_OBJECT, Buffer: *mut core::ffi::c_void, BufferLength: u32, MinorFunction: i32, Override: i32, Iosb: *mut _IO_STATUS_BLOCK, ) -> NTSTATUS;
    fn CdReadSectors( IrpContext: *mut IrpContext, ByteOffset: i64, LengthInBytes: u32, RaiseOnError: i32, Buffer: *mut i8, TargetDevice: *mut DEVICE_OBJECT, ) -> i32;
    fn CdRvdId(RawIsoVd: *mut i8, Flags: u32) -> *mut i8;
    fn CdRvdVersion(RawIsoVd: *mut i8, Flags: u32) -> u32;
    fn CdRvdDescType(RawIsoVd: *mut i8, Flags: u32) -> u32;
    fn LlBytesFromSectors(SectorCount: u32) -> i64;
    fn RtlEqualMemory(Source1: *const core::ffi::c_void, Source2: *const core::ffi::c_void, Length: usize) -> i32;
    fn CdFreePool(Pool: *mut *mut core::ffi::c_void);
    fn FlagOn(flags: u32, flag: u32) -> i32;
    fn CdSerial32(RawIsoVd: *mut u8, SectorSize: u32) -> u32;
    fn NT_ASSERT(expression: i32);
    fn CdRvdEsc(RawIsoVd: *mut u8, State: u32) -> *mut u8;
    fn CdRvdVolId(RawIsoVd: *mut u8, State: u32) -> *mut u8;
    fn CdConvertBigToLittleEndian(IrpContext: *mut IrpContext, Src: *mut u8, SrcLen: u32, Dst: *mut u8);
    fn RtlOemToUnicodeN(Dest: *mut u16, DestBytes: u32, ResultLength: *mut u32, Src: *const u8, SrcBytes: u32) -> i32;
    fn CdInvalidateVolumes(IrpContext: *mut core::ffi::c_void, Irp: *mut core::ffi::c_void) -> i32;
    fn CdMountVolume(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_FSCTRL;
pub static mut CdDisable: BOOLEAN = 0;
pub static mut CdNoJoliet: BOOLEAN = 0;

// Constants not already provided by the crate (values from typical CDFS driver)
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x0001;
const VPB_LOCKED: u32 = 0x0001;
const VCB_STATE_LOCKED: u32 = 0x8000;
const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;
const CDFS_RESIDUAL_USER_REFERENCE: u32 = 1;

// External functions local to the CDFS driver (not in ntkrnl_sys)

// Forward declarations for unknown externs

// Placeholder constants – values are unknown and must be corrected in later passes
const VcbMounted: u32 = 2;                            // VcbCondition::VcbMounted
const VCB_STATE_VPB_NOT_ON_DEVICE: u32 = 0x80;        // guessed bitmask
const CD_SEC_CACHE_CHUNKS: u32 = 16;                  // guessed
const CD_SEC_CHUNK_BLOCKS: u32 = 32;                  // guessed
const SECTOR_SIZE: usize = 2048;                      // typical CD sector size

// Forward extern declarations for functions not in imports

const UserVolumeOpen: OPEN_TYPE = 0; // actual value from C enum OPEN_TYPE
const FSRTL_VOLUME_LOCK: ULONG = 1;   // typical; actual value may differ
const FSRTL_VOLUME_LOCK_FAILED: ULONG = 2; // typical

const FSRTL_VOLUME_DISMOUNT: u32 = 2u32;
const VcbDismountInProgress: u32 = 2u32;
const VcbInvalid: u32 = 3u32;
const VCB_STATE_DISMOUNTED: u32 = 2u32;
const CCB_FLAG_DISMOUNT_ON_CLOSE: u32 = 8u32;
const STATUS_VOLUME_DISMOUNTED: i32 = 0xC000026Eu32 as i32;

const STATUS_INVALID_USER_BUFFER: i32 = 0xC00000F6u32 as i32;

// Constants: these placeholder values must be replaced by the real definitions from the C code.
const CCB_FLAG_ALLOW_EXTENDED_DASD_IO: u32 = 0;

// Constants that are not imported
const FIRST_VD_SECTOR: u32 = 16;
const VOL_ID_LEN: u32 = 5;
const VERSION_1: u32 = 1;
const VD_TERMINATOR: u32 = 0xFF;
const VD_PRIMARY: u32 = 1;
const TAG_CDROM_TOC: u32 = 0x20434F54; // 'TOC ' in little-endian
const IOCTL_CDROM_GET_LAST_SESSION: u32 = 0x24038; // CTL_CODE(FILE_DEVICE_CD_ROM, 0x000E, METHOD_BUFFERED, FILE_READ_ACCESS)
const CdPagedPool: u32 = 1; // POOL_TYPE::PagedPool

// VCB state flags (values are arbitrary but unique, they will be defined elsewhere,
// we just need compile-time constants here)
const VCB_STATE_AUDIO_DISK: u32 = 0x00000001;
const VCB_STATE_ISO: u32 = 0x00000100;
const VCB_STATE_HSG: u32 = 0x00000200;

// Extern declarations for functions and static data not in the imports

// External declarations for macros/functions used by this function

// Constants not provided — guess / placeholder values
const VcbNotMounted: i32 = 0; // first enum value

// Assume the following global / type definitions exist elsewhere.
// The caller will supply the actual definitions.
// Minimal placeholder type for CdData so we can access .VcbQueue
#[repr(C)]
struct CdData {
    VcbQueue: LIST_ENTRY,
    // other fields omitted
}

// Constants used in this function
const IRP_MN_USER_FS_REQUEST: u8 = 0x00;
const IRP_MN_MOUNT_VOLUME: u8 = 0x01;
const IRP_MN_VERIFY_VOLUME: u8 = 0x02;

// Forward declarations for external functions not in available imports

unsafe fn CdLockVolumeInternal(
    irp_context: *mut IRP_CONTEXT,
    vcb: *mut VCB,
    file_object: *mut FILE_OBJECT,
) -> NTSTATUS {
    let mut saved_irql: KIRQL;
    let mut final_status: NTSTATUS =
        if !file_object.is_null() { STATUS_ACCESS_DENIED } else { STATUS_DEVICE_BUSY };
    let remaining_user_references: u32 = if !file_object.is_null() { 1 } else { 0 };

    CdPurgeVolume(irp_context, vcb, 0);

    CdReleaseVcb(irp_context, vcb);

    let status = CcWaitForCurrentLazyWriterActivity();

    // SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )
    (*irp_context).Flags |= IRP_CONTEXT_FLAG_WAIT;

    CdAcquireVcbExclusive(irp_context, vcb, 0);

    if !NT_SUCCESS(status) {
        return status;
    }

    CdFspClose(vcb);

    IoAcquireVpbSpinLock(&mut saved_irql);

    let vpb = (*vcb).Vpb;
    // if (!FlagOn( Vcb->Vpb->Flags, VPB_LOCKED ) && ...)
    if !((*vpb).Flags & VPB_LOCKED) != 0
        && (*vcb).VcbCleanup == remaining_user_references
        && (*vcb).VcbUserReference == CDFS_RESIDUAL_USER_REFERENCE + remaining_user_references
    {
        // SetFlag( Vcb->VcbState, VCB_STATE_LOCKED )
        (*vcb).VcbState |= VCB_STATE_LOCKED;
        // SetFlag( Vcb->Vpb->Flags, VPB_LOCKED )
        (*vpb).Flags |= VPB_LOCKED;
        (*vcb).VolumeLockFileObject = file_object;
        final_status = STATUS_SUCCESS;
    }

    IoReleaseVpbSpinLock(saved_irql);

    final_status
}
#[no_mangle]
pub unsafe extern "C" fn CdUnlockVolumeInternal(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FileObject: *mut FILE_OBJECT,
) -> i32 {
    let mut status: i32 = STATUS_NOT_LOCKED;
    let mut saved_irql: u8;

    // UNREFERENCED_PARAMETER(IrpContext)
    let _ = IrpContext;

    IoAcquireVpbSpinLock(&mut saved_irql as *mut u8);

    // FlagOn(Vcb->Vpb->Flags, VPB_LOCKED) && FileObject == Vcb->VolumeLockFileObject
    if ((*(*Vcb).Vpb).Flags & VPB_LOCKED) != 0
        && FileObject == (*Vcb).VolumeLockFileObject
    {
        // ClearFlag(Vcb->VcbState, VCB_STATE_LOCKED)
        (*Vcb).VcbState &= !VCB_STATE_LOCKED;
        // ClearFlag(Vcb->Vpb->Flags, VPB_LOCKED)
        (*(*Vcb).Vpb).Flags &= !VPB_LOCKED;
        (*Vcb).VolumeLockFileObject = core::ptr::null_mut();
        status = STATUS_SUCCESS;
    }

    IoReleaseVpbSpinLock(saved_irql);

    status
}
#[no_mangle]
pub unsafe extern "C" fn CdCommonFsControl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let irp_sp = IoGetCurrentIrpStackLocation(Irp);
    PAGED_CODE();
    let minor = (*irp_sp).MinorFunction;

    let status: NTSTATUS;
    match minor {
        IRP_MN_USER_FS_REQUEST => {
            status = CdUserFsctl(IrpContext, Irp);
        }
        IRP_MN_MOUNT_VOLUME => {
            status = CdMountVolume(IrpContext, Irp);
        }
        IRP_MN_VERIFY_VOLUME => {
            status = CdVerifyVolume(IrpContext, Irp);
        }
        _ => {
            CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn CdUserFsctl(IrpContext: *mut core::ffi::c_void, Irp: *mut core::ffi::c_void) -> i32 {
    let Status: i32;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    // PAGED_CODE();

    match (*IrpSp).Parameters.FileSystemControl.FsControlCode {
        FSCTL_REQUEST_OPLOCK_LEVEL_1 |
        FSCTL_REQUEST_OPLOCK_LEVEL_2 |
        FSCTL_REQUEST_BATCH_OPLOCK |
        FSCTL_OPLOCK_BREAK_ACKNOWLEDGE |
        FSCTL_OPBATCH_ACK_CLOSE_PENDING |
        FSCTL_OPLOCK_BREAK_NOTIFY |
        FSCTL_OPLOCK_BREAK_ACK_NO_2 |
        FSCTL_REQUEST_FILTER_OPLOCK => {
            Status = CdOplockRequest(IrpContext, Irp);
        }
        FSCTL_LOCK_VOLUME => {
            Status = CdLockVolume(IrpContext, Irp);
        }
        FSCTL_UNLOCK_VOLUME => {
            Status = CdUnlockVolume(IrpContext, Irp);
        }
        FSCTL_DISMOUNT_VOLUME => {
            Status = CdDismountVolume(IrpContext, Irp);
        }
        FSCTL_IS_VOLUME_DIRTY => {
            Status = CdIsVolumeDirty(IrpContext, Irp);
        }
        FSCTL_IS_VOLUME_MOUNTED => {
            Status = CdIsVolumeMounted(IrpContext, Irp);
        }
        FSCTL_IS_PATHNAME_VALID => {
            Status = CdIsPathnameValid(IrpContext, Irp);
        }
        FSCTL_INVALIDATE_VOLUMES => {
            Status = CdInvalidateVolumes(IrpContext, Irp);
        }
        FSCTL_ALLOW_EXTENDED_DASD_IO => {
            Status = CdAllowExtendedDasdIo(IrpContext, Irp);
        }
        _ => {
            CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdReMountOldVcb(
    IrpContext: *mut IRP_CONTEXT,
    OldVcb: *mut VCB,
    NewVcb: *mut VCB,
    DeviceObjectWeTalkTo: *mut DEVICE_OBJECT,
) {
    let SavedIrql: KIRQL;
    let mut Index: u32;
    let mut Buffer: *mut u8;

    // UNREFERENCED_PARAMETER(IrpContext) - suppress unused parameter; no Rust equivalent needed

    ObDereferenceObject((*OldVcb).TargetDeviceObject as *mut _); // assuming TargetDeviceObject is a pointer

    IoAcquireVpbSpinLock(&mut SavedIrql as *mut KIRQL);

    (*(*NewVcb).Vpb).RealDevice.Vpb = (*OldVcb).Vpb;

    (*OldVcb).Vpb.RealDevice = (*(*NewVcb).Vpb).RealDevice;
    (*OldVcb).TargetDeviceObject = DeviceObjectWeTalkTo;

    CdUpdateVcbCondition(OldVcb, VcbMounted);
    CdUpdateMediaChangeCount(OldVcb, (*NewVcb).MediaChangeCount);

    // Expand ClearFlag macro inline
    (*OldVcb).VcbState &= !VCB_STATE_VPB_NOT_ON_DEVICE;

    Buffer = (*NewVcb).SectorCacheBuffer;
    (*OldVcb).SectorCacheBuffer = Buffer;
    (*NewVcb).SectorCacheBuffer = core::ptr::null_mut();

    if !Buffer.is_null() {
        Index = 0;
        while Index < CD_SEC_CACHE_CHUNKS {
            (*OldVcb).SecCacheChunks[Index as usize].Buffer = Buffer;
            (*OldVcb).SecCacheChunks[Index as usize].BaseLbn = u32::MAX;
            Buffer = Buffer.add((CD_SEC_CHUNK_BLOCKS * SECTOR_SIZE) as usize);
            Index += 1;
        }
    }

    IoReleaseVpbSpinLock(SavedIrql);
}
#[no_mangle]
pub unsafe extern "C" fn CdVerifyVolume(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // Declare all locals exactly as in C
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Vpb: PVPB = (*IrpSp).Parameters.VerifyVolume.Vpb;
    let Vcb: PVCB = &mut (*(*IrpSp).Parameters.VerifyVolume.DeviceObject as *mut VOLUME_DEVICE_OBJECT).Vcb as *mut VCB;

    let mut RawIsoVd: PCHAR = core::ptr::null_mut();
    let mut CdromToc: *mut CDROM_TOC_LARGE = core::ptr::null_mut();
    let mut TocLength: ULONG = 0u32;
    let mut TocTrackCount: ULONG = 0u32;
    let mut TocDiskFlags: ULONG = 0u32;
    let mut MediaChangeCount: ULONG = (*Vcb).MediaChangeCount;
    let mut FileObjectToNotify: PFILE_OBJECT = core::ptr::null_mut();
    let mut ReturnError: BOOLEAN;
    let mut ReleaseVcb: BOOLEAN = 0u8; // FALSE
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut AnsiLabel: STRING;
    let mut UnicodeLabel: _UNICODE_STRING;
    let mut VolumeLabel: [u16; VOLUME_ID_LENGTH] = [0u16; VOLUME_ID_LENGTH];
    let mut VolumeLabelLength: ULONG;
    let mut Index: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE();
    // We'll ignore pageable macro for now.

    // NT_ASSERT( Vpb->RealDevice->DeviceType == FILE_DEVICE_CD_ROM );
    // NT_ASSERT( FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ));
    // We'll skip assertions in translation as they are debug-only.

    // Update real device in IrpContext
    (*IrpContext).RealDevice = (*Vpb).RealDevice;

    // Acquire global resource
    CdAcquireCdData(IrpContext);

    // SEH_TRY block simulation via a loop and break for try_return
    // We'll use an inner loop to allow break to simulate goto try_exit, but carefully.
    // First part inside try block:
    'try_inner: loop {
        CdAcquireVcbExclusive(IrpContext, Vcb, 0u8);
        ReleaseVcb = 1u8;

        // Check if volume is eligible for verification
        if ((*Vcb).VcbCondition == VcbInvalid) || ((*Vcb).VcbCondition == VcbDismountInProgress) {
            Status = STATUS_WRONG_VOLUME;
            break 'try_inner; // try_return
        }

        // Verify that there is a disk here
        Status = CdPerformDevIoCtrl(
            IrpContext,
            IOCTL_CDROM_CHECK_VERIFY,
            (*Vcb).TargetDeviceObject,
            ((&mut MediaChangeCount) as *mut ULONG) as *mut core::ffi::c_void,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
            0u32,
            0u8, // FALSE
            1u8, // TRUE
            &mut Iosb,
        );

        if NT_SUCCESS(Status) == 0 {
            // If we will allow a raw mount then return WRONG_VOLUME
            if (FlagOn((*IrpSp).Flags, SL_ALLOW_RAW_MOUNT)) != 0 {
                Status = STATUS_WRONG_VOLUME;
            }
            break 'try_inner; // try_return( Status )
        }

        if Iosb.Information != core::mem::size_of::<ULONG>() as ULONG {
            MediaChangeCount = 0;
        }

        // Verify that the device saw a change
        if (MediaChangeCount == 0) || ((*Vcb).MediaChangeCount != MediaChangeCount) {
            // Allocate buffer to query TOC
            CdromToc = FsRtlAllocatePoolWithTag(
                CdPagedPool,
                core::mem::size_of::<CDROM_TOC_LARGE>() as ULONG,
                TAG_CDROM_TOC,
            ) as *mut CDROM_TOC_LARGE;

            // RtlZeroMemory
            core::ptr::write_bytes(CdromToc as *mut u8, 0u8, core::mem::size_of::<CDROM_TOC_LARGE>());

            // Query for the Toc now
            Status = CdProcessToc(
                IrpContext,
                (*Vcb).TargetDeviceObject,
                CdromToc,
                &mut TocLength,
                &mut TocTrackCount,
                &mut TocDiskFlags,
            );

            // If failed to read TOC, give up
            if Status != STATUS_SUCCESS {
                if CdIsRawDevice(IrpContext, Status) == 0 {
                    Status = STATUS_WRONG_VOLUME;
                }
                break 'try_inner; // try_return( Status )
            } else if ((*Vcb).TocLength != TocLength) ||
                       ((*Vcb).TrackCount != TocTrackCount) ||
                       ((*Vcb).DiskFlags != TocDiskFlags) ||
                       (RtlEqualMemory(
                           CdromToc as *const core::ffi::c_void,
                           (*Vcb).CdromToc as *const core::ffi::c_void,
                           TocLength as usize,
                       ) == 0)
            {
                Status = STATUS_WRONG_VOLUME;
                break 'try_inner; // try_return( Status = STATUS_WRONG_VOLUME )
            }

            // If audio disk, we have a match already; otherwise check volume descriptor
            if (FlagOn((*Vcb).VcbState, VCB_STATE_AUDIO_DISK)) == 0 {
                // Allocate buffer for sector buffer
                RawIsoVd = FsRtlAllocatePoolWithTag(
                    CdNonPagedPool,
                    ROUND_TO_PAGES(2u32 * SECTOR_SIZE),
                    TAG_VOL_DESC,
                ) as PCHAR;

                // Read the primary volume descriptor
                ReturnError = 0u8; // FALSE
                if (FlagOn((*IrpSp).Flags, SL_ALLOW_RAW_MOUNT)) != 0 {
                    ReturnError = 1u8; // TRUE
                }

                if CdFindPrimaryVd(
                    IrpContext,
                    Vcb,
                    RawIsoVd,
                    (*Vcb).BlockFactor,
                    ReturnError,
                    1u8, // TRUE
                ) == 0
                {
                    Status = STATUS_WRONG_VOLUME;
                    break 'try_inner; // try_return
                } else {
                    // Look for supplementary VD
                    // Copy primary VD to second half of RawIsoVd
                    RtlCopyMemory(
                        Add2Ptr(RawIsoVd as *mut core::ffi::c_void, SECTOR_SIZE) as *mut core::ffi::c_void,
                        RawIsoVd as *const core::ffi::c_void,
                        SECTOR_SIZE,
                    );

                    // Locate secondary volume descriptor if present
                    CdFindActiveVolDescriptor(IrpContext, Vcb, RawIsoVd, 1u8);

                    // Compare serial numbers
                    if (*Vpb).SerialNumber != CdSerial32(RawIsoVd, SECTOR_SIZE) {
                        Status = STATUS_WRONG_VOLUME;
                        break 'try_inner; // try_return
                    }

                    // Verify volume labels
                    if (FlagOn((*Vcb).VcbState, VCB_STATE_JOLIET)) == 0 {
                        // Compute length of volume name
                        AnsiLabel.Buffer = CdRvdVolId(RawIsoVd, (*Vcb).VcbState) as *const u8 as *mut u8 as *mut i8;
                        AnsiLabel.MaximumLength = VOLUME_ID_LENGTH as u16;
                        AnsiLabel.Length = VOLUME_ID_LENGTH as u16;

                        UnicodeLabel.MaximumLength = (VOLUME_ID_LENGTH * core::mem::size_of::<u16>() as u32) as u16;
                        UnicodeLabel.Buffer = VolumeLabel.as_mut_ptr() as *mut u16;

                        // Convert to unicode
                        VolumeLabelLength = 0u32;
                        if NT_SUCCESS(RtlOemStringToCountedUnicodeString(
                            &mut UnicodeLabel,
                            &mut AnsiLabel,
                            0u8, // FALSE
                        )) {
                            VolumeLabelLength = UnicodeLabel.Length as ULONG;
                        }
                    } else {
                        // Convert big-endian to little-endian
                        CdConvertBigToLittleEndian(
                            IrpContext,
                            CdRvdVolId(RawIsoVd, (*Vcb).VcbState),
                            VOLUME_ID_LENGTH,
                            VolumeLabel.as_mut_ptr() as PCHAR,
                        );
                        VolumeLabelLength = VOLUME_ID_LENGTH;
                    }

                    // Strip trailing spaces or zeroes
                    Index = VolumeLabelLength / (core::mem::size_of::<u16>() as ULONG);
                    while Index > 0 {
                        let idx = (Index - 1) as usize;
                        if (VolumeLabel[idx] != 0u16) && (VolumeLabel[idx] != 0x20u16) {
                            break;
                        }
                        Index -= 1;
                    }
                    VolumeLabelLength = (Index * (core::mem::size_of::<u16>() as ULONG)) as u16 as ULONG;

                    // Check that the label matches
                    if ((*Vpb).VolumeLabelLength != VolumeLabelLength) ||
                        (RtlEqualMemory(
                            (*Vpb).VolumeLabel as *const core::ffi::c_void,
                            VolumeLabel.as_ptr() as *const core::ffi::c_void,
                            VolumeLabelLength as usize,
                        ) == 0)
                    {
                        Status = STATUS_WRONG_VOLUME;
                        break 'try_inner; // try_return
                    }
                }
            }
        }

        // Volume is OK, clear the verify bit
        CdUpdateVcbCondition(Vcb, VcbMounted);
        CdMarkRealDevVerifyOk((*Vpb).RealDevice);

        // See if we need to provide notification of remount
        if (FlagOn((*Vcb).VcbState, VCB_STATE_NOTIFY_REMOUNT)) != 0 {
            ClearFlag((*Vcb).VcbState, VCB_STATE_NOTIFY_REMOUNT);
            FileObjectToNotify = (*(*Vcb).RootIndexFcb).FileObject;
            ObReferenceObject(FileObjectToNotify as *mut core::ffi::c_void);
        }

        // Normal exit from try block: break to after the try block but before finally code
        break 'try_inner;
    }

    // Code that was after the try_exit: label and before the finally
    CdUpdateMediaChangeCount(Vcb, MediaChangeCount);

    // Handle volume condition/dis mount
    if (*Vcb).VcbCondition == VcbNotMounted {
        Status = STATUS_WRONG_VOLUME;
    } else if ((*Vcb).VcbCondition == VcbMounted) && (Status == STATUS_WRONG_VOLUME) {
        CdUpdateVcbCondition(Vcb, VcbNotMounted);

        if !(*Vcb).XASector.is_null() {
            CdFreePool(&((*Vcb).XASector) as *const _ as *mut *mut core::ffi::c_void);
            (*Vcb).XASector = 0u32 as *mut core::ffi::c_void;
            (*Vcb).XADiskOffset = 0;
        }

        CdFreeDirCache(IrpContext);

        if (*Vcb).VcbCleanup == 0 {
            if NT_SUCCESS(CdPurgeVolume(IrpContext, Vcb, 0u8)) {
                ReleaseVcb = CdCheckForDismount(IrpContext, Vcb, 0u8);
            }
        }
    }

    // Finally block (SEH_FINALLY)
    // Free TOC buffer if not in Vcb
    if !CdromToc.is_null() {
        CdFreePool(&mut (CdromToc as *mut core::ffi::c_void) as *mut *mut core::ffi::c_void);
    }

    if !RawIsoVd.is_null() {
        CdFreePool(&mut (RawIsoVd as *mut core::ffi::c_void) as *mut *mut core::ffi::c_void);
    }

    if ReleaseVcb != 0 {
        CdReleaseVcb(IrpContext, Vcb);
    }

    CdReleaseCdData(IrpContext);

    // Now send mount notification
    if !FileObjectToNotify.is_null() {
        FsRtlNotifyVolumeEvent(FileObjectToNotify, FSRTL_VOLUME_MOUNT);
        ObDereferenceObject(FileObjectToNotify as *mut core::ffi::c_void);
    }

    // Complete the request if no exception (though we don't handle SEH exceptions)
    CdCompleteRequest(IrpContext, Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdOplockRequest(
    IrpContext: *mut IrpContext,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS is 0
    let mut Fcb: *mut Fcb = core::ptr::null_mut();
    let mut Ccb: *mut Ccb = core::ptr::null_mut();
    let mut OplockCount: ULONG = 0;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    // PAGED_CODE(); omitted as no-op

    if CdDecodeFileObject(
        IrpContext,
        (*(IrpSp as *mut core::ffi::c_void as *mut PIO_STACK_LOCATION)).FileObject,
        &mut Fcb,
        &mut Ccb,
    ) != UserFileOpen
    {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER as NTSTATUS);
        return STATUS_INVALID_PARAMETER as NTSTATUS;
    }

    SetFlag(&mut (*(IrpContext as *mut IrpContext)).Flags as *mut ULONG, IRP_CONTEXT_FLAG_WAIT);
    ClearFlag(
        &mut (*(IrpContext as *mut IrpContext)).Flags as *mut ULONG,
        IRP_CONTEXT_FLAG_FORCE_POST,
    );

    let fs_control_code = (*(IrpSp as *mut PIO_STACK_LOCATION))
        .Parameters
        .FileSystemControl
        .FsControlCode;

    match fs_control_code {
        FSCTL_REQUEST_OPLOCK_LEVEL_1
        | FSCTL_REQUEST_OPLOCK_LEVEL_2
        | FSCTL_REQUEST_BATCH_OPLOCK
        | FSCTL_REQUEST_FILTER_OPLOCK => {
            CdAcquireFcbExclusive(IrpContext, Fcb, 0); // FALSE -> 0

            if fs_control_code == FSCTL_REQUEST_OPLOCK_LEVEL_2 {
                if !(*(Fcb as *const Fcb)).FileLock.is_null() {
                    OplockCount =
                        FsRtlAreThereCurrentFileLocks((*(Fcb as *const Fcb)).FileLock) as ULONG;
                }
            } else {
                OplockCount = (*(Fcb as *const Fcb)).FcbCleanup as ULONG;
            }
        }
        FSCTL_OPLOCK_BREAK_ACKNOWLEDGE
        | FSCTL_OPBATCH_ACK_CLOSE_PENDING
        | FSCTL_OPLOCK_BREAK_NOTIFY
        | FSCTL_OPLOCK_BREAK_ACK_NO_2 => {
            CdAcquireFcbShared(IrpContext, Fcb, 0);
        }
        _ => {
            CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER as NTSTATUS);
            return STATUS_INVALID_PARAMETER as NTSTATUS;
        }
    }

    // try body
    {
        CdVerifyFcbOperation(IrpContext, Fcb);

        Status = FsRtlOplockFsctrl(CdGetFcbOplock(Fcb), Irp, OplockCount);

        CdLockFcb(IrpContext, Fcb);
        (*(Fcb as *mut Fcb)).IsFastIoPossible = CdIsFastIoPossible(Fcb);
        CdUnlockFcb(IrpContext, Fcb);

        Irp = core::ptr::null_mut();
    }
    // finally body
    CdReleaseFcb(IrpContext, Fcb);

    CdCompleteRequest(IrpContext, Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdLockVolume(
    IrpContext: *mut IRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let Fcb: *mut FCB;
    let Ccb: *mut CCB;

    if CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb) != UserVolumeOpen {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_LOCK);

    let Vcb = (*Fcb).Vcb;
    CdAcquireVcbExclusive(IrpContext, Vcb, 0); // FALSE => 0

    let mut _abnormal = true;

    // SEH_TRY block
    {
        CdVerifyVcb(IrpContext, Vcb);
        Status = CdLockVolumeInternal(IrpContext, Vcb, (*IrpSp).FileObject);
        _abnormal = false;
    }
    // SEH_FINALLY block
    CdReleaseVcb(IrpContext, Vcb);
    if _abnormal != 0 || !NT_SUCCESS(Status) {
        FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_LOCK_FAILED);
    }

    CdCompleteRequest(IrpContext, Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdUnlockVolume(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // extern declarations for unknown functions

    // constant FSRTL_VOLUME_UNLOCK (assumed value, unknown exact constant)
    // left as identifier assuming caller provides definition
    // Uncomment and adjust if value known: const FSRTL_VOLUME_UNLOCK: u32 = 2;

    let mut Status: NTSTATUS;
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Vcb: *mut VCB;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    // PAGED_CODE();
    // commented out, macro not expanded

    // Decode the file object
    if CdDecodeFileObject(
        IrpContext,
        (*IrpSp).FileObject,
        &mut Fcb,
        &mut Ccb,
    ) != UserVolumeOpen as i32
    {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Acquire exclusive access to the Vcb
    Vcb = (*Fcb).Vcb;
    CdAcquireVcbExclusive(IrpContext, Vcb, 0); // FALSE

    // Unlock the volume
    Status = CdUnlockVolumeInternal(IrpContext, Vcb, (*IrpSp).FileObject);

    // Release all resources
    CdReleaseVcb(IrpContext, Vcb);

    // Send notification if successful
    if NT_SUCCESS(Status) {
        FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_UNLOCK);
    }

    // Complete the request
    CdCompleteRequest(IrpContext, Irp, Status);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdDismountVolume(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Vcb: *mut VCB;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    PAGED_CODE();

    if CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb) != UserVolumeOpen {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    Vcb = (*Fcb).Vcb;

    FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_DISMOUNT);

    SetFlag(&mut (*IrpContext).Flags as *mut u32, IRP_CONTEXT_FLAG_WAIT);

    CdAcquireCdData(IrpContext);
    CdAcquireVcbExclusive(IrpContext, Vcb, 0i32);

    if (*Vcb).VcbCondition != VcbMounted {
        Status = STATUS_VOLUME_DISMOUNTED;
    } else {
        CdLockVcb(IrpContext, Vcb);

        if (*Vcb).VcbCondition != VcbDismountInProgress {
            CdUpdateVcbCondition(Vcb, VcbInvalid);
        }

        SetFlag(&mut (*Vcb).VcbState as *mut u32, VCB_STATE_DISMOUNTED);

        CdUnlockVcb(IrpContext, Vcb);

        SetFlag(&mut (*Ccb).Flags as *mut u32, CCB_FLAG_DISMOUNT_ON_CLOSE);

        Status = STATUS_SUCCESS;
    }

    CdReleaseVcb(IrpContext, Vcb);
    CdReleaseCdData(IrpContext);

    // #if (NTDDI_VERSION >= NTDDI_WIN8) removed
    FsRtlDismountComplete((*Vcb).TargetDeviceObject, Status);

    CdCompleteRequest(IrpContext, Irp, Status);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn CdIsVolumeDirty(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> i32 {
    // PAGED_CODE();
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let VolumeState: *mut u32;

    if !(*Irp).AssociatedIrp.SystemBuffer.is_null() {
        VolumeState = (*Irp).AssociatedIrp.SystemBuffer as *mut u32;
    } else {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_USER_BUFFER);
        return STATUS_INVALID_USER_BUFFER;
    }

    if (*IrpSp).Parameters.FileSystemControl.OutputBufferLength < (core::mem::size_of::<u32>() as u32) {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    *VolumeState = 0;

    let TypeOfOpen: i32;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    TypeOfOpen = CdDecodeFileObject(
        IrpContext,
        (*IrpSp).FileObject,
        core::ptr::addr_of_mut!(Fcb),
        core::ptr::addr_of_mut!(Ccb),
    );

    if TypeOfOpen != UserVolumeOpen {
        CdCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if (*(*Fcb).Vcb).VcbCondition != VcbMounted {
        CdCompleteRequest(IrpContext, Irp, STATUS_VOLUME_DISMOUNTED);
        return STATUS_VOLUME_DISMOUNTED;
    }

    (*Irp).IoStatus.Information = core::mem::size_of::<u32>() as u64;

    CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CdIsVolumeMounted(
    IrpContext: *mut PIRP_CONTEXT,
    Irp: *mut PIRP,
) -> i32 {
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Fcb: *mut Fcb = core::ptr::null_mut();
    let mut Ccb: *mut Ccb = core::ptr::null_mut();

    CdDecodeFileObject(IrpContext, (*IrpSp).FileObject, &mut Fcb, &mut Ccb);

    if !Fcb.is_null() {
        SetFlag(&mut (*IrpContext).Flags, 0x1 /* IRP_CONTEXT_FLAG_DISABLE_POPUPS */);
        CdVerifyVcb(IrpContext, (*Fcb).Vcb);
    }

    CdCompleteRequest(IrpContext, Irp, 0);
    0
}
#[no_mangle]
pub unsafe extern "C" fn CdIsPathnameValid(IrpContext: *mut IRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CdAllowExtendedDasdIo(irp_context: *mut IRP_CONTEXT, irp: *mut IRP) -> i32 {
    let irp_sp = IoGetCurrentIrpStackLocation(irp);
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();
    let status: i32;

    PAGED_CODE();

    let pfcb: *mut *mut FCB = core::ptr::addr_of_mut!(fcb);
    let pccb: *mut *mut CCB = core::ptr::addr_of_mut!(ccb);

    if CdDecodeFileObject(irp_context, (*irp_sp).FileObject, pfcb, pccb) != UserVolumeOpen {
        status = STATUS_INVALID_PARAMETER;
    } else {
        let flags_ptr: *mut u32 = core::ptr::addr_of_mut!((*ccb).Flags);
        SetFlag(flags_ptr, CCB_FLAG_ALLOW_EXTENDED_DASD_IO);
        status = STATUS_SUCCESS;
    }

    CdCompleteRequest(irp_context, irp, status);
    return status;
}

#[no_mangle]
pub unsafe extern "C" fn CdScanForDismountedVcb(IrpContext: PIRP_CONTEXT) {
    // PAGED_CODE();

    let head: *const LIST_ENTRY = core::ptr::addr_of!(crate::CdData.VcbQueue);
    let mut links: *mut LIST_ENTRY = (*head).Flink;

    while links != head {
        let vcb: *mut VCB = (links as usize - core::mem::offset_of!(VCB, VcbLinks)) as *mut VCB;

        //  Move to the next link now since the current Vcb may be deleted.
        links = (*links).Flink;

        //  If dismount is already underway then check if this Vcb can go away.
        if (*vcb).VcbCondition == VCB_CONDITION::VcbDismountInProgress
            || (*vcb).VcbCondition == VCB_CONDITION::VcbInvalid
            || ((*vcb).VcbCondition == VCB_CONDITION::VcbNotMounted
                && (*vcb).VcbReference <= CDFS_RESIDUAL_REFERENCE)
        {
            CdCheckForDismount(IrpContext, vcb, 0); // FALSE
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdFindPrimaryVd(
    IrpContext: *mut IrpContext,
    Vcb: *mut Vcb,
    RawIsoVd: *mut i8,
    BlockFactor: u32,
    ReturnOnError: i32,
    VerifyVolume: i32,
) -> i32 {
    // PAGED_CODE();
    let mut ThisPass: u32 = 1;
    let mut FoundVd: i32 = 0; // FALSE
    let mut BaseSector: u32;
    let mut SectorOffset: u32;
    let mut CdromToc: *mut CDROM_TOC_LARGE;
    let mut VolumeFlags: u32;
    let mut Status: NTSTATUS;
    let mut Source: *const u8;
    let mut Dest: *mut u8;
    let mut Count: u8;

    // If there are no data tracks, bail out.
    if ((*Vcb).VcbState & VCB_STATE_AUDIO_DISK) != 0 {
        return 0;
    }

    while FoundVd == 0 && ThisPass <= 2 {
        BaseSector = 0;

        if ThisPass == 1 {
            CdromToc = core::ptr::null_mut();

            // (SEH2_TRY transformed manually to avoid exception handling;
            //  the only exception path is STATUS_INSUFFICIENT_RESOURCES, which
            //  we handle with a free and return 0, preserving the mount failure.)
            CdromToc = FsRtlAllocatePoolWithTag(
                CdPagedPool,
                core::mem::size_of::<CDROM_TOC_LARGE>() as u32,
                TAG_CDROM_TOC,
            ) as *mut CDROM_TOC_LARGE;

            RtlZeroMemory(
                CdromToc as *mut core::ffi::c_void,
                core::mem::size_of::<CDROM_TOC_LARGE>() as u32,
            );

            Status = CdPerformDevIoCtrl(
                IrpContext,
                IOCTL_CDROM_GET_LAST_SESSION,
                (*Vcb).TargetDeviceObject,
                CdromToc as *mut core::ffi::c_void,
                core::mem::size_of::<CDROM_TOC_LARGE>() as u32,
                0, // FALSE
                1, // TRUE
                core::ptr::null_mut(),
            );

            if Status == STATUS_INSUFFICIENT_RESOURCES {
                if !CdromToc.is_null() {
                    CdFreePool(core::ptr::addr_of_mut!(CdromToc) as *mut *mut core::ffi::c_void);
                    CdromToc = core::ptr::null_mut();
                }
                // This is the equivalent of raising an exception; it exits the mount.
                return 0;
            }

            if Status >= 0 && ((*CdromToc).FirstTrack != (*CdromToc).LastTrack) {
                // Byte-swap to get BaseSector from big-endian Address[4]
                Count = 4;
                Source = core::ptr::addr_of!((*CdromToc).TrackData[0].Address[3]) as *const u8;
                Dest = core::ptr::addr_of_mut!(BaseSector) as *mut u8;
                loop {
                    *Dest = *Source;
                    Dest = Dest.add(1);
                    Source = Source.sub(1);
                    Count -= 1;
                    if Count == 0 {
                        break;
                    }
                }

                BaseSector /= BlockFactor;
            } else {
                ThisPass += 1; // act as second pass, no need to retry
            }

            // Finally block: free the TOC buffer
            if !CdromToc.is_null() {
                CdFreePool(core::ptr::addr_of_mut!(CdromToc) as *mut *mut core::ffi::c_void);
                CdromToc = core::ptr::null_mut();
            }
        }

        SectorOffset = FIRST_VD_SECTOR;
        VolumeFlags = 0;

        // Inner loop over volume descriptors
        loop {
            if CdReadSectors(
                IrpContext,
                LlBytesFromSectors(BaseSector + SectorOffset),
                SECTOR_SIZE, // assumed constant; we'll just use 2048
                if ThisPass == 1 || ReturnOnError != 0 { 1 } else { 0 },
                RawIsoVd,
                (*Vcb).TargetDeviceObject,
            ) == 0
            {
                break;
            }

            // Check if ISO or HSG volume descriptor
            if RtlEqualMemory(
                core::ptr::addr_of!(CdIsoId) as *const core::ffi::c_void,
                CdRvdId(RawIsoVd, VCB_STATE_ISO) as *const core::ffi::c_void,
                VOL_ID_LEN as usize,
            ) != 0
            {
                VolumeFlags |= VCB_STATE_ISO;
            } else if RtlEqualMemory(
                core::ptr::addr_of!(CdHsgId) as *const core::ffi::c_void,
                CdRvdId(RawIsoVd, VCB_STATE_HSG) as *const core::ffi::c_void,
                VOL_ID_LEN as usize,
            ) != 0
            {
                VolumeFlags |= VCB_STATE_HSG;
            } else {
                break;
            }

            // Check version and terminator
            if CdRvdVersion(RawIsoVd, VolumeFlags) != VERSION_1
                || CdRvdDescType(RawIsoVd, VolumeFlags) == VD_TERMINATOR
            {
                break;
            }

            // If primary volume descriptor, we are done
            if CdRvdDescType(RawIsoVd, VolumeFlags) == VD_PRIMARY {
                if VerifyVolume == 0 {
                    (*Vcb).VcbState |= VolumeFlags;
                    (*Vcb).BaseSector = BaseSector;
                    (*Vcb).VdSectorOffset = SectorOffset;
                    (*Vcb).PrimaryVdSectorOffset = SectorOffset;
                }
                FoundVd = 1;
                break;
            }

            SectorOffset += 1;
        }

        ThisPass += 1;
    }

    FoundVd
}
#[no_mangle]
pub unsafe extern "C" fn CdIsRemount(
    IrpContext: *mut IrpContext,
    Vcb: *mut VCB,
    OldVcb: *mut *mut VCB,
) -> i32 {
    // PAGED_CODE(); (macro call)
    PAGED_CODE();

    // UNREFERENCED_PARAMETER(IrpContext) -> volatile self-assignment
    core::ptr::write_volatile(&mut IrpContext, IrpContext);

    let Vpb: *mut VPB = (*Vcb).Vpb;
    let mut OldVpb: *mut VPB;
    let mut Remount: i32 = 0; // BOOLEAN equivalent

    let head = core::ptr::addr_of_mut!(CdData.VcbQueue);
    let mut Link = (*head).Flink;

    while Link != head {
        // *OldVcb = CONTAINING_RECORD(Link, VCB, VcbLinks);
        *OldVcb = (Link as *mut u8)
            .sub(core::mem::offset_of!(VCB, VcbLinks))
            as *mut VCB;

        // Skip ourselves
        if Vcb == *OldVcb {
            Link = (*Link).Flink;
            continue;
        }

        OldVpb = (**OldVcb).Vpb;

        if (OldVpb != Vpb)
            && ((*OldVpb).RealDevice == (*Vpb).RealDevice)
            && ((**OldVcb).VcbCondition == VcbNotMounted)
        {
            // Current disk is audio disk
            if FlagOn((*Vcb).VcbState, VCB_STATE_AUDIO_DISK) != 0 {
                if FlagOn((**OldVcb).VcbState, VCB_STATE_AUDIO_DISK) != 0 {
                    // Both have TOC – compare lengths and contents
                    if ((*Vcb).TocLength != (**OldVcb).TocLength)
                        || ((*Vcb).TocLength != 0
                            && RtlEqualMemory(
                                (*Vcb).CdromToc as *const core::ffi::c_void,
                                (**OldVcb).CdromToc as *const core::ffi::c_void,
                                (*Vcb).TocLength as usize,
                            ) == 0)
                    {
                        Link = (*Link).Flink;
                        continue;
                    }
                    Remount = 1;
                    break;
                }
            }
            // Current disk is not audio disk – compare serial numbers, labels, TOC
            else if ((*OldVpb).SerialNumber == (*Vpb).SerialNumber)
                && ((*Vcb).TocLength == (**OldVcb).TocLength)
                && ((*Vcb).TocLength == 0
                    || RtlEqualMemory(
                        (*Vcb).CdromToc as *const core::ffi::c_void,
                        (**OldVcb).CdromToc as *const core::ffi::c_void,
                        (*Vcb).TocLength as usize,
                    ) != 0)
                && ((*Vpb).VolumeLabelLength == (*OldVpb).VolumeLabelLength)
                && (RtlEqualMemory(
                    (*OldVpb).VolumeLabel as *const core::ffi::c_void,
                    (*Vpb).VolumeLabel as *const core::ffi::c_void,
                    (*Vpb).VolumeLabelLength as usize,
                ) != 0)
            {
                Remount = 1;
                break;
            }
        }

        Link = (*Link).Flink;
    }

    Remount
}

static CdJolietEscape: [[u8; 3usize]; 3usize];

const VD_SECONDARY: u8 = 2;
const ESC_SEQ_LEN: u32 = 3;
const VOLUME_ID_LENGTH: u32 = 32;
const MAXIMUM_VOLUME_LABEL_LENGTH: u32 = 64;
const VCB_STATE_JOLIET: u32 = 0x02;

unsafe extern "C" fn CdFindActiveVolDescriptor(
    irp_context: *mut IrpContext,
    vcb: *mut Vcb,
    raw_iso_vd: *mut u8,
    verify_volume: u8,
) {
    let mut found_secondary_vd: u8 = 0;
    let mut sector_offset: u32 = FIRST_VD_SECTOR;
    let mut length: u32;
    let mut index: u32;

    // PAGED_CODE(); // omitted as it's a no-op

    if ((((*vcb).VcbState & VCB_STATE_ISO) != 0) || (verify_volume != 0)) && CdNoJoliet == 0 {
        sector_offset = FIRST_VD_SECTOR;

        loop {
            CdReadSectors(
                irp_context,
                LlBytesFromSectors((*vcb).BaseSector + sector_offset),
                SECTOR_SIZE,
                0,
                raw_iso_vd,
                (*vcb).TargetDeviceObject,
            );

            if RtlEqualMemory(
                CdIsoId.as_ptr() as *const core::ffi::c_void,
                CdRvdId(raw_iso_vd, VCB_STATE_JOLIET) as *const core::ffi::c_void,
                VOL_ID_LEN as usize,
            ) == 0
                || CdRvdVersion(raw_iso_vd, VCB_STATE_JOLIET) != VERSION_1
                || CdRvdDescType(raw_iso_vd, VCB_STATE_JOLIET) == VD_TERMINATOR
            {
                break;
            }

            if CdRvdDescType(raw_iso_vd, VCB_STATE_JOLIET) == VD_SECONDARY
                && (RtlEqualMemory(
                    CdRvdEsc(raw_iso_vd, VCB_STATE_JOLIET) as *const core::ffi::c_void,
                    CdJolietEscape[0].as_ptr() as *const core::ffi::c_void,
                    ESC_SEQ_LEN as usize,
                ) != 0
                    || RtlEqualMemory(
                        CdRvdEsc(raw_iso_vd, VCB_STATE_JOLIET) as *const core::ffi::c_void,
                        CdJolietEscape[1].as_ptr() as *const core::ffi::c_void,
                        ESC_SEQ_LEN as usize,
                    ) != 0
                    || RtlEqualMemory(
                        CdRvdEsc(raw_iso_vd, VCB_STATE_JOLIET) as *const core::ffi::c_void,
                        CdJolietEscape[2].as_ptr() as *const core::ffi::c_void,
                        ESC_SEQ_LEN as usize,
                    ) != 0)
            {
                if verify_volume == 0 {
                    (*vcb).VcbState &= !VCB_STATE_ISO;
                    (*vcb).VcbState |= VCB_STATE_JOLIET;
                    (*vcb).VdSectorOffset = sector_offset;
                }
                found_secondary_vd = 1;
                break;
            }

            sector_offset += 1;
        }

        if found_secondary_vd == 0 {
            core::ptr::copy_nonoverlapping(
                raw_iso_vd.add(SECTOR_SIZE as usize),
                raw_iso_vd,
                SECTOR_SIZE as usize,
            );
        }
    }

    if verify_volume != 0 {
        return;
    }

    (*(*vcb).Vpb).SerialNumber = CdSerial32(raw_iso_vd, SECTOR_SIZE);

    NT_ASSERT(
        ((VOLUME_ID_LENGTH * core::mem::size_of::<u16>() as u32) <= MAXIMUM_VOLUME_LABEL_LENGTH) as i32,
    );

    if ((*vcb).VcbState & VCB_STATE_JOLIET) == 0 {
        (*(*vcb).Vpb).VolumeLabelLength = 0;
        if RtlOemToUnicodeN(
            &mut (*(*vcb).Vpb).VolumeLabel[0],
            MAXIMUM_VOLUME_LABEL_LENGTH,
            &mut length,
            CdRvdVolId(raw_iso_vd, (*vcb).VcbState) as *const u8,
            VOLUME_ID_LENGTH,
        ) >= 0
        {
            (*(*vcb).Vpb).VolumeLabelLength = length as u16;
        }
    } else {
        CdConvertBigToLittleEndian(
            irp_context,
            CdRvdVolId(raw_iso_vd, (*vcb).VcbState),
            VOLUME_ID_LENGTH,
            (*(*vcb).Vpb).VolumeLabel.as_mut_ptr() as *mut u8,
        );
        (*(*vcb).Vpb).VolumeLabelLength = (VOLUME_ID_LENGTH * core::mem::size_of::<u16>() as u32) as u16;
    }

    index = (*(*vcb).Vpb).VolumeLabelLength / core::mem::size_of::<u16>() as u16;
    while index > 0 {
        if (*(*vcb).Vpb).VolumeLabel[(index - 1) as usize] != 0
            && (*(*vcb).Vpb).VolumeLabel[(index - 1) as usize] != ' ' as u16
        {
            break;
        }
        index -= 1;
    }
    (*(*vcb).Vpb).VolumeLabelLength = (index * core::mem::size_of::<u16>() as u16) as u16;
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
//     FsCtrl.c
//
// Abstract:
//
//     This module implements the File System Control routines for Cdfs called
//     by the Fsd/Fsp dispatch drivers.
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_FSCTRL)
//
// //
// //  Local constants
// //
//
// BOOLEAN CdDisable = FALSE;
// BOOLEAN CdNoJoliet = FALSE;
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdUserFsctl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// VOID
// CdReMountOldVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB OldVcb,
//      PVCB NewVcb,
//      PDEVICE_OBJECT DeviceObjectWeTalkTo
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdMountVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdVerifyVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdOplockRequest (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdLockVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdUnlockVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdDismountVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// NTSTATUS
// CdIsVolumeDirty (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// NTSTATUS
// CdIsVolumeMounted (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// NTSTATUS
// CdIsPathnameValid (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdInvalidateVolumes (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// NTSTATUS
// CdAllowExtendedDasdIo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdScanForDismountedVcb (
//      PIRP_CONTEXT IrpContext
//     );
//
//
// BOOLEAN
// CdFindPrimaryVd (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PCHAR RawIsoVd,
//      ULONG BlockFactor,
//      BOOLEAN ReturnOnError,
//      BOOLEAN VerifyVolume
//     );
//
//  BOOLEAN
// CdIsRemount (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PVCB *OldVcb
//     );
//
// VOID
// CdFindActiveVolDescriptor (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//     (ROUND_TO_PAGES( SECTOR_SIZE )) PCHAR RawIsoVd,
//      BOOLEAN VerifyVolume
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCommonFsControl)
// #pragma alloc_text(PAGE, CdDismountVolume)
// #pragma alloc_text(PAGE, CdFindActiveVolDescriptor)
// #pragma alloc_text(PAGE, CdFindPrimaryVd)
// #pragma alloc_text(PAGE, CdIsPathnameValid)
// #pragma alloc_text(PAGE, CdIsRemount)
// #pragma alloc_text(PAGE, CdIsVolumeDirty)
// #pragma alloc_text(PAGE, CdIsVolumeMounted)
// #pragma alloc_text(PAGE, CdLockVolume)
// #pragma alloc_text(PAGE, CdMountVolume)
// #pragma alloc_text(PAGE, CdOplockRequest)
// #pragma alloc_text(PAGE, CdAllowExtendedDasdIo)
// #pragma alloc_text(PAGE, CdScanForDismountedVcb)
// #pragma alloc_text(PAGE, CdUnlockVolume)
// #pragma alloc_text(PAGE, CdUserFsctl)
// #pragma alloc_text(PAGE, CdVerifyVolume)
// #endif
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// _Requires_lock_held_(Vcb->VcbResource)
// NTSTATUS
// CdLockVolumeInternal (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the actual lock volume operation.  It will be called
//     by anyone wishing to try to protect the volume for a long duration.  PNP
//     operations are such a user.
//
//     The volume must be held exclusive by the caller.
//
// Arguments:
//
//     Vcb - The volume being locked.
//
//     FileObject - File corresponding to the handle locking the volume.  If this
//         is not specified, a system lock is assumed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     KIRQL SavedIrql;
//     NTSTATUS FinalStatus = (FileObject? STATUS_ACCESS_DENIED: STATUS_DEVICE_BUSY);
//     ULONG RemainingUserReferences = (FileObject? 1: 0);
//
//     //
//     //  The cleanup count for the volume only reflects the fileobject that
//     //  will lock the volume.  Otherwise, we must fail the request.
//     //
//     //  Since the only cleanup is for the provided fileobject, we will try
//     //  to get rid of all of the other user references.  If there is only one
//     //  remaining after the purge then we can allow the volume to be locked.
//     //
//
//     CdPurgeVolume( IrpContext, Vcb, FALSE );
//
//     //
//     //  Now back out of our synchronization and wait for the lazy writer
//     //  to finish off any lazy closes that could have been outstanding.
//     //
//     //  Since we purged, we know that the lazy writer will issue all
//     //  possible lazy closes in the next tick - if we hadn't, an otherwise
//     //  unopened file with a large amount of dirty data could have hung
//     //  around for a while as the data trickled out to the disk.
//     //
//     //  This is even more important now since we send notification to
//     //  alert other folks that this style of check is about to happen so
//     //  that they can close their handles.  We don't want to enter a fast
//     //  race with the lazy writer tearing down his references to the file.
//     //
//
//     CdReleaseVcb( IrpContext, Vcb );
//
//     Status = CcWaitForCurrentLazyWriterActivity();
//
//     //
//     //  This is intentional. If we were able to get the Vcb before, just
//     //  wait for it and take advantage of knowing that it is OK to leave
//     //  the flag up.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     if (!NT_SUCCESS( Status )) {
//
//         return Status;
//     }
//
//     CdFspClose( Vcb );
//
//     //
//     //  If the volume is already explicitly locked then fail.  We use the
//     //  Vpb locked flag as an 'explicit lock' flag in the same way as Fat.
//     //
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     if (!FlagOn( Vcb->Vpb->Flags, VPB_LOCKED ) &&
//         (Vcb->VcbCleanup == RemainingUserReferences) &&
//         (Vcb->VcbUserReference == CDFS_RESIDUAL_USER_REFERENCE + RemainingUserReferences))  {
//
//         SetFlag( Vcb->VcbState, VCB_STATE_LOCKED );
//         SetFlag( Vcb->Vpb->Flags, VPB_LOCKED);
//         Vcb->VolumeLockFileObject = FileObject;
//         FinalStatus = STATUS_SUCCESS;
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     return FinalStatus;
// }
//
//
// NTSTATUS
// CdUnlockVolumeInternal (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the actual unlock volume operation.
//
//     The volume must be held exclusive by the caller.
//
// Arguments:
//
//     Vcb - The volume being locked.
//
//     FileObject - File corresponding to the handle locking the volume.  If this
//         is not specified, a system lock is assumed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
//     Attempting to remove a system lock that did not exist is OK.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_NOT_LOCKED;
//     KIRQL SavedIrql;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Note that we check the VPB_LOCKED flag here rather than the Vcb
//     //  lock flag.  The Vpb flag is only set for an explicit lock request,  not
//     //  for the implicit lock obtained on a volume open with zero share mode.
//     //
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     if (FlagOn(Vcb->Vpb->Flags, VPB_LOCKED) &&
//         (FileObject == Vcb->VolumeLockFileObject))  {
//
//         ClearFlag( Vcb->VcbState, VCB_STATE_LOCKED );
//         ClearFlag( Vcb->Vpb->Flags, VPB_LOCKED);
//         Vcb->VolumeLockFileObject = NULL;
//         Status = STATUS_SUCCESS;
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     return Status;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdCommonFsControl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for doing FileSystem control operations called
//     by both the fsd and fsp threads
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PAGED_CODE();
//
//     //
//     //  We know this is a file system control so we'll case on the
//     //  minor function, and call a internal worker routine to complete
//     //  the irp.
//     //
//
//     switch (IrpSp->MinorFunction) {
//
//     case IRP_MN_USER_FS_REQUEST:
//
//         Status = CdUserFsctl( IrpContext, Irp );
//         break;
//
//     case IRP_MN_MOUNT_VOLUME:
//
//         Status = CdMountVolume( IrpContext, Irp );
//         break;
//
//     case IRP_MN_VERIFY_VOLUME:
//
//         Status = CdVerifyVolume( IrpContext, Irp );
//         break;
//
//     default:
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdUserFsctl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This is the common routine for implementing the user's requests made
//     through NtFsControlFile.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PAGED_CODE();
//
//     //
//     //  Case on the control code.
//     //
//
//     switch ( IrpSp->Parameters.FileSystemControl.FsControlCode ) {
//
//     case FSCTL_REQUEST_OPLOCK_LEVEL_1 :
//     case FSCTL_REQUEST_OPLOCK_LEVEL_2 :
//     case FSCTL_REQUEST_BATCH_OPLOCK :
//     case FSCTL_OPLOCK_BREAK_ACKNOWLEDGE :
//     case FSCTL_OPBATCH_ACK_CLOSE_PENDING :
//     case FSCTL_OPLOCK_BREAK_NOTIFY :
//     case FSCTL_OPLOCK_BREAK_ACK_NO_2 :
//     case FSCTL_REQUEST_FILTER_OPLOCK :
//
//         Status = CdOplockRequest( IrpContext, Irp );
//         break;
//
//     case FSCTL_LOCK_VOLUME :
//
//         Status = CdLockVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_UNLOCK_VOLUME :
//
//         Status = CdUnlockVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_DISMOUNT_VOLUME :
//
//         Status = CdDismountVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_VOLUME_DIRTY :
//
//         Status = CdIsVolumeDirty( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_VOLUME_MOUNTED :
//
//         Status = CdIsVolumeMounted( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_PATHNAME_VALID :
//
//         Status = CdIsPathnameValid( IrpContext, Irp );
//         break;
//
//     case FSCTL_INVALIDATE_VOLUMES :
//
//         Status = CdInvalidateVolumes( IrpContext, Irp );
//         break;
//
//     case FSCTL_ALLOW_EXTENDED_DASD_IO:
//
//         Status = CdAllowExtendedDasdIo( IrpContext, Irp );
//         break;
//
//     //
//     //  We don't support any of the known or unknown requests.
//     //
//
//     default:
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     return Status;
// }
//
//
// VOID
// CdReMountOldVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB OldVcb,
//      PVCB NewVcb,
//      PDEVICE_OBJECT DeviceObjectWeTalkTo
//     )
// {
//     KIRQL SavedIrql;
//     ULONG Index;
//     PUCHAR Buffer;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     ObDereferenceObject( OldVcb->TargetDeviceObject );
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//     NewVcb->Vpb->RealDevice->Vpb = OldVcb->Vpb;
//
//     OldVcb->Vpb->RealDevice = NewVcb->Vpb->RealDevice;
//     OldVcb->TargetDeviceObject = DeviceObjectWeTalkTo;
//
//     CdUpdateVcbCondition( OldVcb, VcbMounted);
//     CdUpdateMediaChangeCount( OldVcb, NewVcb->MediaChangeCount);
//
//     ClearFlag( OldVcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE);
//
//     Buffer = OldVcb->SectorCacheBuffer = NewVcb->SectorCacheBuffer;
//     NewVcb->SectorCacheBuffer = NULL;
//
//     if (NULL != Buffer) {
//
//         for (Index = 0; Index < CD_SEC_CACHE_CHUNKS; Index++) {
//
//             OldVcb->SecCacheChunks[ Index].Buffer = Buffer;
//             OldVcb->SecCacheChunks[ Index].BaseLbn = (ULONG)-1;
//
//             Buffer += CD_SEC_CHUNK_BLOCKS * SECTOR_SIZE;
//         }
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdMountVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the mount volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.
//
//     Its job is to verify that the volume denoted in the IRP is a Cdrom volume,
//     and create the VCB and root DCB structures.  The algorithm it
//     uses is essentially as follows:
//
//     1. Create a new Vcb Structure, and initialize it enough to do I/O
//        through the on-disk volume descriptors.
//
//     2. Read the disk and check if it is a Cdrom volume.
//
//     3. If it is not a Cdrom volume then delete the Vcb and
//        complete the IRP back with an appropriate status.
//
//     4. Check if the volume was previously mounted and if it was then do a
//        remount operation.  This involves deleting the VCB, hook in the
//        old VCB, and complete the IRP.
//
//     5. Otherwise create a Vcb and root DCB for each valid volume descriptor.
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
//     PVOLUME_DEVICE_OBJECT VolDo = NULL;
//     PVCB Vcb = NULL;
//     PVCB OldVcb;
//     UCHAR StackSize;
//
//     BOOLEAN FoundPvd = FALSE;
//     BOOLEAN SetDoVerifyOnFail;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     PDEVICE_OBJECT DeviceObjectWeTalkTo = IrpSp->Parameters.MountVolume.DeviceObject;
//     PVPB Vpb = IrpSp->Parameters.MountVolume.Vpb;
//
//     PFILE_OBJECT FileObjectToNotify = NULL;
//
//     ULONG BlockFactor;
//     DISK_GEOMETRY DiskGeometry;
//
//     IO_SCSI_CAPABILITIES Capabilities;
//
//     IO_STATUS_BLOCK Iosb;
//
//     PCHAR RawIsoVd = NULL;
//
//     PCDROM_TOC_LARGE CdromToc = NULL;
//     ULONG TocLength = 0;
//     ULONG TocTrackCount = 0;
//     ULONG TocDiskFlags = 0;
//     ULONG MediaChangeCount = 0;
//
// #ifdef __REACTOS__
//     DEVICE_TYPE FilesystemDeviceType;
// #endif
//
// #ifdef CDFS_TELEMETRY_DATA
//     GUID VolumeGuid;
//     GUID VolumeCorrelationId = { 0 };
// #endif
//
//     PAGED_CODE();
//
//     //
//     //  Check that we are talking to a Cdrom device.  This request should
//     //  always be waitable.
//     //
//
// #ifdef __REACTOS__
//     if (IrpSp->DeviceObject == CdData.HddFileSystemDeviceObject) {
//         FilesystemDeviceType = FILE_DEVICE_DISK_FILE_SYSTEM;
//     } else {
// #endif
//     NT_ASSERT( Vpb->RealDevice->DeviceType == FILE_DEVICE_CD_ROM );
// #ifdef __REACTOS__
//         FilesystemDeviceType = FILE_DEVICE_CD_ROM_FILE_SYSTEM;
//     }
// #endif
//     NT_ASSERT( FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ));
//
// #ifdef CDFS_TELEMETRY_DATA
//     //
//     //  We don't want a bogus VolumeGuid to show up in our telemetry
//     //
//
//     RtlZeroMemory( &VolumeGuid, sizeof(GUID) );
//
// #endif
//
//     //
//     //  Update the real device in the IrpContext from the Vpb.  There was no available
//     //  file object when the IrpContext was created.
//     //
//
//     IrpContext->RealDevice = Vpb->RealDevice;
//
//     SetDoVerifyOnFail = CdRealDevNeedsVerify( IrpContext->RealDevice);
//
//     //
//     //  Check if we have disabled the mount process.
//     //
//
//     if (CdDisable) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_UNRECOGNIZED_VOLUME );
//         return STATUS_UNRECOGNIZED_VOLUME;
//     }
//
//     //
//     //  If we've shutdown disallow further mounts.
//     //
//
//     if (FlagOn( CdData.Flags, CD_FLAGS_SHUTDOWN )) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_SYSTEM_SHUTDOWN );
//         return STATUS_SYSTEM_SHUTDOWN;
//     }
//
//     //
//     //  Do a CheckVerify here to lift the MediaChange ticker from the driver
//     //
//
//     Status = CdPerformDevIoCtrl( IrpContext,
// #ifndef __REACTOS__
//                                  IOCTL_CDROM_CHECK_VERIFY,
// #else
//                                  (FilesystemDeviceType == FILE_DEVICE_DISK_FILE_SYSTEM ? IOCTL_DISK_CHECK_VERIFY : IOCTL_CDROM_CHECK_VERIFY),
// #endif
//                                  DeviceObjectWeTalkTo,
//                                  &MediaChangeCount,
//                                  sizeof(ULONG),
//                                  FALSE,
//                                  TRUE,
//                                  &Iosb );
//
//     if (!NT_SUCCESS( Status )) {
//
//         CdCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     if (Iosb.Information != sizeof(ULONG)) {
//
//         //
//         //  Be safe about the count in case the driver didn't fill it in
//         //
//
//         MediaChangeCount = 0;
//     }
//
//     //
//     //  Now let's make Jeff delirious and call to get the disk geometry.  This
//     //  will fix the case where the first change line is swallowed.
//     //
//
//     Status = CdPerformDevIoCtrl( IrpContext,
// #ifndef __REACTOS__
//                                  IOCTL_CDROM_GET_DRIVE_GEOMETRY,
// #else
//                                  (FilesystemDeviceType == FILE_DEVICE_DISK_FILE_SYSTEM ? IOCTL_DISK_GET_DRIVE_GEOMETRY : IOCTL_CDROM_GET_DRIVE_GEOMETRY),
// #endif
//                                  DeviceObjectWeTalkTo,
//                                  &DiskGeometry,
//                                  sizeof( DISK_GEOMETRY ),
//                                  FALSE,
//                                  TRUE,
//                                  NULL );
//
//     //
//     //  Return insufficient sources to our caller.
//     //
//
//     if (Status == STATUS_INSUFFICIENT_RESOURCES) {
//
//         CdCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     //
//     //  Now check the block factor for addressing the volume descriptors.
//     //  If the call for the disk geometry failed then assume there is one
//     //  block per sector.
//     //
//
//     BlockFactor = 1;
//
//     if (NT_SUCCESS( Status ) &&
//         (DiskGeometry.BytesPerSector != 0) &&
//         (DiskGeometry.BytesPerSector < SECTOR_SIZE)) {
//
//         BlockFactor = SECTOR_SIZE / DiskGeometry.BytesPerSector;
//     }
//
//     //
//     //  Acquire the global resource to do mount operations.
//     //
//
//     CdAcquireCdData( IrpContext );
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Allocate a buffer to query the TOC.
//         //
//
//         CdromToc = FsRtlAllocatePoolWithTag( CdPagedPool,
//                                              sizeof( CDROM_TOC_LARGE ),
//                                              TAG_CDROM_TOC );
//
//         RtlZeroMemory( CdromToc, sizeof( CDROM_TOC_LARGE ));
//
//         //
//         //  Do a quick check to see if there any Vcb's which can be removed.
//         //
//
//         CdScanForDismountedVcb( IrpContext );
//
//         //
//         //  Get our device object and alignment requirement.
//         //
//
//         Status = IoCreateDevice( CdData.DriverObject,
//                                  sizeof( VOLUME_DEVICE_OBJECT ) - sizeof( DEVICE_OBJECT ),
//                                  NULL,
// #ifndef __REACTOS__
//                                  FILE_DEVICE_CD_ROM_FILE_SYSTEM,
// #else
//                                  FilesystemDeviceType,
// #endif
//                                  0,
//                                  FALSE,
//                                  (PDEVICE_OBJECT *) &VolDo );
//
//         if (!NT_SUCCESS( Status )) { try_leave( Status ); }
//
//         //
//         //  Our alignment requirement is the larger of the processor alignment requirement
//         //  already in the volume device object and that in the DeviceObjectWeTalkTo
//         //
//
//         if (DeviceObjectWeTalkTo->AlignmentRequirement > VolDo->DeviceObject.AlignmentRequirement) {
//
//             VolDo->DeviceObject.AlignmentRequirement = DeviceObjectWeTalkTo->AlignmentRequirement;
//         }
//
//         //
//         //  We must initialize the stack size in our device object before
//         //  the following reads, because the I/O system has not done it yet.
//         //
//
//         ((PDEVICE_OBJECT) VolDo)->StackSize = (CCHAR) (DeviceObjectWeTalkTo->StackSize + 1);
//         StackSize = ((PDEVICE_OBJECT) VolDo)->StackSize;
//
//         ClearFlag( VolDo->DeviceObject.Flags, DO_DEVICE_INITIALIZING );
//
//         //
//         //  Initialize the overflow queue for the volume
//         //
//
//         VolDo->OverflowQueueCount = 0;
//         InitializeListHead( &VolDo->OverflowQueue );
//
//         VolDo->PostedRequestCount = 0;
//         KeInitializeSpinLock( &VolDo->OverflowQueueSpinLock );
//
//         //
//         //  Let's query for the Toc now and handle any error we get from this operation.
//         //
//
//         Status = CdProcessToc( IrpContext,
//                                DeviceObjectWeTalkTo,
//                                CdromToc,
//                                &TocLength,
//                                &TocTrackCount,
//                                &TocDiskFlags );
//
//         //
//         //  If we failed to read the TOC, then bail out.  Probably blank media.
//         //
//
//         if (Status != STATUS_SUCCESS)  {
//
// #ifdef __REACTOS__
//
//             //
//             // Don't bail out if that was a disk based ISO image, it is legit
//             //
//
//             if (FilesystemDeviceType == FILE_DEVICE_DISK_FILE_SYSTEM) {
//                 CdFreePool( &CdromToc );
//                 Status = STATUS_SUCCESS;
//             } else {
// #endif
//             try_leave( Status );
// #ifdef __REACTOS__
//             }
// #endif
//         }
//
//         //
//         //  Now before we can initialize the Vcb we need to set up the
//         //  device object field in the VPB to point to our new volume device
//         //  object.
//         //
//
//         Vpb->DeviceObject = (PDEVICE_OBJECT) VolDo;
//
//         //
//         //  Initialize the Vcb.  This routine will raise on an allocation
//         //  failure.
//         //
//
//         CdInitializeVcb( IrpContext,
//                          &VolDo->Vcb,
//                          DeviceObjectWeTalkTo,
//                          Vpb,
//                          CdromToc,
//                          TocLength,
//                          TocTrackCount,
//                          TocDiskFlags,
//                          BlockFactor,
//                          MediaChangeCount );
//
//         //
//         //  Show that we initialized the Vcb and can cleanup with the Vcb.
//         //
//
//         Vcb = &VolDo->Vcb;
//         VolDo = NULL;
//         Vpb = NULL;
//         CdromToc = NULL;
//
// #ifdef CDFS_TELEMETRY_DATA
//
//         //
//         //  Initialize the volume guid.
//         //
//
//         if (NT_SUCCESS( IoVolumeDeviceToGuid( Vcb->TargetDeviceObject, &VolumeGuid ))) {
//
//             //
//             //  We got a GUID, set it in the Telemetry structure
//             //
//
//             RtlCopyMemory( &CdTelemetryData.VolumeGuid, &VolumeGuid, sizeof(GUID) );
//         }
//
//         //
//         // Initialize the correlation ID.
//         //
//
//         if (NT_SUCCESS( FsRtlVolumeDeviceToCorrelationId( Vcb->TargetDeviceObject, &VolumeCorrelationId ) )) {
//
//             //
//             // Stash a copy away in the VCB.
//             //
//
//             RtlCopyMemory( &Vcb->VolumeCorrelationId, &VolumeCorrelationId, sizeof( GUID ) );
//         }
//
// #endif // CDFS_TELEMETRY_DATA
//
//         // Lock object is acquired and released using internal state
//         _Analysis_suppress_lock_checking_(Vcb->VcbResource);
//
//         //
//         //  Store the Vcb in the IrpContext as we didn't have one before.
//         //
//
//         IrpContext->Vcb = Vcb;
//
//         CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//         //
//         //  Let's reference the Vpb to make sure we are the one to
//         //  have the last dereference.
//         //
//
//         Vcb->Vpb->ReferenceCount += 1;
//
//         //
//         //  Clear the verify bit for the start of mount.
//         //
//
//         CdMarkRealDevVerifyOk( Vcb->Vpb->RealDevice);
//
//         if (!FlagOn( Vcb->VcbState, VCB_STATE_AUDIO_DISK))  {
//
//             //
//             //  Allocate a buffer to read in the volume descriptors.  We allocate a full
//             //  page to make sure we don't hit any alignment problems.
//             //
//
//             RawIsoVd = FsRtlAllocatePoolWithTag( CdNonPagedPool,
//                                                  ROUND_TO_PAGES( SECTOR_SIZE ),
//                                                  TAG_VOL_DESC );
//
//             //
//             //  Try to find the primary volume descriptor.
//             //
//
//             FoundPvd = CdFindPrimaryVd(   IrpContext,
//                                           Vcb,
//                                           RawIsoVd,
//                                           BlockFactor,
//                                           TRUE,
//                                           FALSE );
//
//             if (!FoundPvd)  {
//
//                 //
//                 //  We failed to find a valid VD in the data track,  but there were also
//                 //  audio tracks on this disc,  so we'll try to mount it as an audio CD.
//                 //  Since we're always last in the mount order,  we won't be preventing
//                 //  any other FS from trying to mount the data track.  However if the
//                 //  data track was at the start of the disc,  then we abort,  to avoid
//                 //  having to filter it from our synthesised directory listing later.  We
//                 //  already filtered off any data track at the end.
//                 //
//
//                 if (!(TocDiskFlags & CDROM_DISK_AUDIO_TRACK) ||
//                      BooleanFlagOn( Vcb->CdromToc->TrackData[0].Control, TOC_DATA_TRACK))  {
//
//                     try_leave( Status = STATUS_UNRECOGNIZED_VOLUME);
//                 }
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_AUDIO_DISK | VCB_STATE_CDXA );
//
//                 CdFreePool( &RawIsoVd );
//                 RawIsoVd = NULL;
//             }
//         }
//
//         //
//         //  Look and see if there is a secondary volume descriptor we want to
//         //  use.
//         //
//
//         if (FoundPvd) {
//
//             //
//             //  Store the primary volume descriptor in the second half of
//             //  RawIsoVd.  Then if our search for a secondary fails we can
//             //  recover this immediately.
//             //
//
//             RtlCopyMemory( Add2Ptr( RawIsoVd, SECTOR_SIZE, PVOID ),
//                            RawIsoVd,
//                            SECTOR_SIZE );
//
//             //
//             //  We have the initial volume descriptor.  Locate a secondary
//             //  volume descriptor if present.
//             //
//
//             CdFindActiveVolDescriptor( IrpContext,
//                                        Vcb,
//                                        RawIsoVd,
//                                        FALSE);
//         }
//
//         //
//         //  Allocate a block cache to speed directory operations. We can't
//         //  use the cache if there is any chance the volume has link blocks
//         //  in the data area (i.e. was packet written and then finalized to
//         //  Joliet/9660). So we simply only allow the cache to operate on
//         //  media with a single track - since we're really targetting pressed
//         //  installation media here. We can't be more precise, since D/CD-ROM
//         //  drives don't support READ_TRACK_INFO, which is the only way for
//         //  certain to know whether or not a track was packet written.
//         //
//
//         if (!FlagOn( Vcb->VcbState, VCB_STATE_AUDIO_DISK) &&
// #ifndef __REACTOS__
//             ((Vcb->CdromToc->LastTrack - Vcb->CdromToc->FirstTrack) == 0)) {
// #else
//            ((FilesystemDeviceType == FILE_DEVICE_DISK_FILE_SYSTEM) ||
//             ((Vcb->CdromToc->LastTrack - Vcb->CdromToc->FirstTrack) == 0))) {
// #endif
//
//             ULONG Index;
//             PUCHAR Buffer;
//
//             Buffer =
//             Vcb->SectorCacheBuffer = FsRtlAllocatePool( CdPagedPool,
//                                                         CD_SEC_CACHE_CHUNKS *
//                                                         CD_SEC_CHUNK_BLOCKS *
//                                                         SECTOR_SIZE);
//
//             for (Index = 0; Index < (ULONG)CD_SEC_CACHE_CHUNKS; Index++) {
//
//                 Vcb->SecCacheChunks[ Index].Buffer = Buffer;
//                 Vcb->SecCacheChunks[ Index].BaseLbn = (ULONG)-1;
//
//                 Buffer += CD_SEC_CHUNK_BLOCKS * SECTOR_SIZE;
//             }
//
//             Vcb->SectorCacheIrp = IoAllocateIrp( StackSize, FALSE);
//
//             if (Vcb->SectorCacheIrp == NULL) {
//
//                 try_leave( Status = STATUS_INSUFFICIENT_RESOURCES );
//             }
//
//             IoInitializeIrp( Vcb->SectorCacheIrp,
//                              IoSizeOfIrp( StackSize),
//                              (CCHAR)StackSize);
//
//             KeInitializeEvent( &Vcb->SectorCacheEvent, SynchronizationEvent, FALSE);
//             ExInitializeResourceLite( &Vcb->SectorCacheResource);
//         }
//
//         //
//         //  Check if this is a remount operation.  If so then clean up
//         //  the data structures passed in and created here.
//         //
//
//         if (CdIsRemount( IrpContext, Vcb, &OldVcb )) {
//
//             NT_ASSERT( NULL != OldVcb->SwapVpb );
//
//             //
//             //  Link the old Vcb to point to the new device object that we
//             //  should be talking to, dereferencing the previous.  Call a
//             //  nonpaged routine to do this since we take the Vpb spinlock.
//             //
//
//             CdReMountOldVcb( IrpContext,
//                              OldVcb,
//                              Vcb,
//                              DeviceObjectWeTalkTo);
//
//             //
//             //  See if we will need to provide notification of the remount.  This is the readonly
//             //  filesystem's form of dismount/mount notification - we promise that whenever a
//             //  volume is "dismounted", that a mount notification will occur when it is revalidated.
//             //  Note that we do not send mount on normal remounts - that would duplicate the media
//             //  arrival notification of the device driver.
//             //
//
//             if (FlagOn( OldVcb->VcbState, VCB_STATE_NOTIFY_REMOUNT )) {
//
//                 ClearFlag( OldVcb->VcbState, VCB_STATE_NOTIFY_REMOUNT );
//
//                 FileObjectToNotify = OldVcb->RootIndexFcb->FileObject;
//                 ObReferenceObject( FileObjectToNotify );
//             }
//
//             try_leave( Status = STATUS_SUCCESS );
//         }
//
//         //
//         //  This is a new mount.  Go ahead and initialize the
//         //  Vcb from the volume descriptor.
//         //
//
//         CdUpdateVcbFromVolDescriptor( IrpContext,
//                                       Vcb,
//                                       RawIsoVd );
//
//         //
//         //  Drop an extra reference on the root dir file so we'll be able to send
//         //  notification.
//         //
//
//         if (Vcb->RootIndexFcb) {
//
//             FileObjectToNotify = Vcb->RootIndexFcb->FileObject;
//             ObReferenceObject( FileObjectToNotify );
//         }
//
//         //
//         //  Now check the maximum transfer limits on the device in case we
//         //  get raw reads on this volume.
//         //
//
//         Status = CdPerformDevIoCtrl( IrpContext,
//                                      IOCTL_SCSI_GET_CAPABILITIES,
//                                      DeviceObjectWeTalkTo,
//                                      &Capabilities,
//                                      sizeof( IO_SCSI_CAPABILITIES ),
//                                      FALSE,
//                                      TRUE,
//                                      NULL );
//
//         if (NT_SUCCESS(Status)) {
//
//             Vcb->MaximumTransferRawSectors = Capabilities.MaximumTransferLength / RAW_SECTOR_SIZE;
//             Vcb->MaximumPhysicalPages = Capabilities.MaximumPhysicalPages;
//
//         } else {
//
//             //
//             //  This should never happen, but we can safely assume 64k and 16 pages.
//             //
//
//             Vcb->MaximumTransferRawSectors = (64 * 1024) / RAW_SECTOR_SIZE;
//             Vcb->MaximumPhysicalPages = 16;
//         }
//
//         //
//         //  The new mount is complete.  Remove the additional references on this
//         //  Vcb and the device we are mounted on top of.
//         //
//
//         Vcb->VcbReference -= CDFS_RESIDUAL_REFERENCE;
//         NT_ASSERT( Vcb->VcbReference == CDFS_RESIDUAL_REFERENCE );
//
//         ObDereferenceObject( Vcb->TargetDeviceObject );
//
//         CdUpdateVcbCondition( Vcb, VcbMounted);
//
//         CdReleaseVcb( IrpContext, Vcb );
//         Vcb = NULL;
//
//         Status = STATUS_SUCCESS;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Free the TOC buffer if not in the Vcb.
//         //
//
//         if (CdromToc != NULL) {
//
//             CdFreePool( &CdromToc );
//         }
//
//         //
//         //  Free the sector buffer if allocated.
//         //
//
//         if (RawIsoVd != NULL) {
//
//             CdFreePool( &RawIsoVd );
//         }
//
//         //
//         //  If we are not mounting the device,  then set the verify bit again.
//         //
//
//         if ((_SEH2_AbnormalTermination() || (Status != STATUS_SUCCESS)) &&
//             SetDoVerifyOnFail)  {
//
//             CdMarkRealDevForVerify( IrpContext->RealDevice);
//         }
//
//         //
//         //  If we didn't complete the mount then cleanup any remaining structures.
//         //
//
//         if (Vpb != NULL) { Vpb->DeviceObject = NULL; }
//
//         if (Vcb != NULL) {
//
//             //
//             //  Make sure there is no Vcb in the IrpContext since it could go away
//             //
//
//             IrpContext->Vcb = NULL;
//
//             Vcb->VcbReference -= CDFS_RESIDUAL_REFERENCE;
//
//             if (CdDismountVcb( IrpContext, Vcb )) {
//
//                 CdReleaseVcb( IrpContext, Vcb );
//             }
//
//         } else if (VolDo != NULL) {
//
//             IoDeleteDevice( (PDEVICE_OBJECT) VolDo );
//         }
//
//         //
//         //  Release the global resource.
//         //
//
//         CdReleaseCdData( IrpContext );
//     } _SEH2_END;
//
//     //
//     //  Now send mount notification.
//     //
//
//     if (FileObjectToNotify) {
//
//         FsRtlNotifyVolumeEvent( FileObjectToNotify, FSRTL_VOLUME_MOUNT );
//         ObDereferenceObject( FileObjectToNotify );
//     }
//
// #ifdef CDFS_TELEMETRY_DATA
//
//     //
//     // Send Telemetry
//     //
//
//     CdTelemetryMountSafe( &VolumeCorrelationId, STATUS_SUCCESS, Vcb );
//
// #endif
//
//     //
//     //  Complete the request if no exception.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdVerifyVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the verify volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     PVPB Vpb = IrpSp->Parameters.VerifyVolume.Vpb;
//     PVCB Vcb = &((PVOLUME_DEVICE_OBJECT) IrpSp->Parameters.VerifyVolume.DeviceObject)->Vcb;
//
//     PCHAR RawIsoVd = NULL;
//
//     PCDROM_TOC_LARGE CdromToc = NULL;
//     ULONG TocLength = 0;
//     ULONG TocTrackCount = 0;
//     ULONG TocDiskFlags = 0;
//
//     ULONG MediaChangeCount = Vcb->MediaChangeCount;
//
//     PFILE_OBJECT FileObjectToNotify = NULL;
//
//     BOOLEAN ReturnError;
//     BOOLEAN ReleaseVcb = FALSE;
//
//     IO_STATUS_BLOCK Iosb;
//
//     STRING AnsiLabel;
//     UNICODE_STRING UnicodeLabel;
//
//     WCHAR VolumeLabel[ VOLUME_ID_LENGTH ];
//     ULONG VolumeLabelLength;
//
//     ULONG Index;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     //  We check that we are talking to a Cdrom device.
//     //
//
//     NT_ASSERT( Vpb->RealDevice->DeviceType == FILE_DEVICE_CD_ROM );
//     NT_ASSERT( FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ));
//
//     //
//     //  Update the real device in the IrpContext from the Vpb.  There was no available
//     //  file object when the IrpContext was created.
//     //
//
//     IrpContext->RealDevice = Vpb->RealDevice;
//
//     //
//     //  Acquire the global resource to synchronise against mounts and teardown,
//     //  finally clause releases.
//     //
//
//     CdAcquireCdData( IrpContext );
//
//     _SEH2_TRY {
//
//         CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//         ReleaseVcb = TRUE;
//
//         //
//         //  Check to see if the volume is eligible for verification.
//         //
//
//         if ((Vcb->VcbCondition == VcbInvalid) ||
//             (Vcb->VcbCondition == VcbDismountInProgress)) {
//
//             try_return( Status = STATUS_WRONG_VOLUME );
//         }
//
//         //
//         //  Verify that there is a disk here.
//         //
//
//         Status = CdPerformDevIoCtrl( IrpContext,
//                                      IOCTL_CDROM_CHECK_VERIFY,
//                                      Vcb->TargetDeviceObject,
//                                      &MediaChangeCount,
//                                      sizeof(ULONG),
//                                      FALSE,
//                                      TRUE,
//                                      &Iosb );
//
//         if (!NT_SUCCESS( Status )) {
//
//             //
//             //  If we will allow a raw mount then return WRONG_VOLUME to
//             //  allow the volume to be mounted by raw.
//             //
//
//             if (FlagOn( IrpSp->Flags, SL_ALLOW_RAW_MOUNT )) {
//
//                 Status = STATUS_WRONG_VOLUME;
//             }
//
//             try_return( Status );
//         }
//
//         if (Iosb.Information != sizeof(ULONG)) {
//
//             //
//             //  Be safe about the count in case the driver didn't fill it in
//             //
//
//             MediaChangeCount = 0;
//         }
//
//         //
//         //  Verify that the device actually saw a change. If the driver does not
//         //  support the MCC, then we must verify the volume in any case.
//         //
//
//         if (MediaChangeCount == 0 ||
//             (Vcb->MediaChangeCount != MediaChangeCount)) {
//
//             //
//             //  Allocate a buffer to query the TOC.
//             //
//
//             CdromToc = FsRtlAllocatePoolWithTag( CdPagedPool,
//                                                  sizeof( CDROM_TOC_LARGE ),
//                                                  TAG_CDROM_TOC );
//
//             RtlZeroMemory( CdromToc, sizeof( CDROM_TOC_LARGE ));
//
//             //
//             //  Let's query for the Toc now and handle any error we get from this operation.
//             //
//
//             Status = CdProcessToc( IrpContext,
//                                    Vcb->TargetDeviceObject,
//                                    CdromToc,
//                                    &TocLength,
//                                    &TocTrackCount,
//                                    &TocDiskFlags );
//
//             //
//             //  If we failed to read the TOC,  then give up now.  Drives will fail
//             //  a TOC read on,  for example,  erased CD-RW media.
//             //
//
//             if (Status != STATUS_SUCCESS) {
//
//                 //
//                 //  For any errors other than no media and not ready,  commute the
//                 //  status to ensure that the current VPB is kicked off the device
//                 //  below - there is probably blank media in the drive,  since we got
//                 //  further than the check verify.
//                 //
//
//                 if (!CdIsRawDevice( IrpContext, Status )) {
//
//                     Status = STATUS_WRONG_VOLUME;
//                 }
//
//                 try_return( Status );
//
//             //
//             //  We got a TOC.  Verify that it matches the previous Toc.
//             //
//
//             } else if ((Vcb->TocLength != TocLength) ||
//                        (Vcb->TrackCount != TocTrackCount) ||
//                        (Vcb->DiskFlags != TocDiskFlags) ||
//                        !RtlEqualMemory( CdromToc,
//                                         Vcb->CdromToc,
//                                         TocLength )) {
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//
//             //
//             //  If the disk to verify is an audio disk then we already have a
//             //  match.  Otherwise we need to check the volume descriptor.
//             //
//
//             if (!FlagOn( Vcb->VcbState, VCB_STATE_AUDIO_DISK )) {
//
//                 //
//                 //  Allocate a buffer for the sector buffer.
//                 //
//
//                 RawIsoVd = FsRtlAllocatePoolWithTag( CdNonPagedPool,
//                                                      ROUND_TO_PAGES( 2 * SECTOR_SIZE ),
//                                                      TAG_VOL_DESC );
//
//                 //
//                 //  Read the primary volume descriptor for this volume.  If we
//                 //  get an io error and this verify was a the result of DASD open,
//                 //  commute the Io error to STATUS_WRONG_VOLUME.  Note that if we currently
//                 //  expect a music disk then this request should fail.
//                 //
//
//                 ReturnError = FALSE;
//
//                 if (FlagOn( IrpSp->Flags, SL_ALLOW_RAW_MOUNT )) {
//
//                     ReturnError = TRUE;
//                 }
//
//                 if (!CdFindPrimaryVd( IrpContext,
//                                       Vcb,
//                                       RawIsoVd,
//                                       Vcb->BlockFactor,
//                                       ReturnError,
//                                       TRUE )) {
//
//                     //
//                     //  If the previous Vcb did not represent a raw disk
//                     //  then show this volume was dismounted.
//                     //
//
//                     try_return( Status = STATUS_WRONG_VOLUME );
//
//                 }
//                 else {
//
//                     //
//                     //  Look for a supplementary VD.
//                     //
//                     //  Store the primary volume descriptor in the second half of
//                     //  RawIsoVd.  Then if our search for a secondary fails we can
//                     //  recover this immediately.
//                     //
//
//                     RtlCopyMemory( Add2Ptr( RawIsoVd, SECTOR_SIZE, PVOID ),
//                                    RawIsoVd,
//                                    SECTOR_SIZE );
//
//                     //
//                     //  We have the initial volume descriptor.  Locate a secondary
//                     //  volume descriptor if present.
//                     //
//
//                     CdFindActiveVolDescriptor( IrpContext,
//                                                Vcb,
//                                                RawIsoVd,
//                                                TRUE);
//                     //
//                     //  Compare the serial numbers.  If they don't match, set the
//                     //  status to wrong volume.
//                     //
//
//                     if (Vpb->SerialNumber != CdSerial32( RawIsoVd, SECTOR_SIZE )) {
//
//                         try_return( Status = STATUS_WRONG_VOLUME );
//                     }
//
//                     //
//                     //  Verify the volume labels.
//                     //
//
//                     if (!FlagOn( Vcb->VcbState, VCB_STATE_JOLIET )) {
//
//                         //
//                         //  Compute the length of the volume name
//                         //
//
//                         AnsiLabel.Buffer = (PCHAR)CdRvdVolId( RawIsoVd, Vcb->VcbState );
//                         AnsiLabel.MaximumLength = AnsiLabel.Length = (ULONG)VOLUME_ID_LENGTH;
//
//                         UnicodeLabel.MaximumLength = VOLUME_ID_LENGTH * sizeof( WCHAR );
//                         UnicodeLabel.Buffer = VolumeLabel;
//
//                         //
//                         //  Convert this to unicode.  If we get any error then use a name
//                         //  length of zero.
//                         //
//
//                         VolumeLabelLength = 0;
//
//                         if (NT_SUCCESS( RtlOemStringToCountedUnicodeString( &UnicodeLabel,
//                                                                             &AnsiLabel,
//                                                                             FALSE ))) {
//
//                             VolumeLabelLength = UnicodeLabel.Length;
//                         }
//
//                     //
//                     //  We need to convert from big-endian to little endian.
//                     //
//
//                     } else {
//
//                         CdConvertBigToLittleEndian( IrpContext,
//                                                     (PCHAR) CdRvdVolId( RawIsoVd, Vcb->VcbState ),
//                                                     VOLUME_ID_LENGTH,
//                                                     (PCHAR) VolumeLabel );
//
//                         VolumeLabelLength = VOLUME_ID_LENGTH;
//                     }
//
//                     //
//                     //  Strip the trailing spaces or zeroes from the name.
//                     //
//
//                     Index = VolumeLabelLength / sizeof( WCHAR );
//
//                     while (Index > 0) {
//
//                         if ((VolumeLabel[ Index - 1 ] != L'\0') &&
//                             (VolumeLabel[ Index - 1 ] != L' ')) {
//
//                             break;
//                         }
//
//                         Index -= 1;
//                     }
//
//                     //
//                     //  Now set the final length for the name.
//                     //
//
//                     VolumeLabelLength = (USHORT) (Index * sizeof( WCHAR ));
//
//                     //
//                     //  Now check that the label matches.
//                     //
//                     if ((Vpb->VolumeLabelLength != VolumeLabelLength) ||
//                         !RtlEqualMemory( Vpb->VolumeLabel,
//                                          VolumeLabel,
//                                          VolumeLabelLength )) {
//
//                         try_return( Status = STATUS_WRONG_VOLUME );
//                     }
//                 }
//             }
//         }
//
//         //
//         //  The volume is OK, clear the verify bit.
//         //
//
//         CdUpdateVcbCondition( Vcb, VcbMounted);
//
//         CdMarkRealDevVerifyOk( Vpb->RealDevice);
//
//         //
//         //  See if we will need to provide notification of the remount.  This is the readonly
//         //  filesystem's form of dismount/mount notification.
//         //
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_NOTIFY_REMOUNT )) {
//
//             ClearFlag( Vcb->VcbState, VCB_STATE_NOTIFY_REMOUNT );
//
//             FileObjectToNotify = Vcb->RootIndexFcb->FileObject;
//             ObReferenceObject( FileObjectToNotify );
//         }
//
//     try_exit: NOTHING;
//
//         //
//         //  Update the media change count to note that we have verified the volume
//         //  at this value - regardless of the outcome.
//         //
//
//         CdUpdateMediaChangeCount( Vcb, MediaChangeCount);
//
//         //
//         //  If the volume was already unmounted, nothing more to do.
//         //
//
//         if (Vcb->VcbCondition == VcbNotMounted) {
//
//             Status = STATUS_WRONG_VOLUME;
//
//         //
//         //  If we got the wrong volume then free any remaining XA sector in
//         //  the current Vcb.  Also mark the Vcb as not mounted.
//         //
//
//         } else if ((Vcb->VcbCondition == VcbMounted) && (Status == STATUS_WRONG_VOLUME)) {
//
//             CdUpdateVcbCondition( Vcb, VcbNotMounted);
//
//             if (Vcb->XASector != NULL) {
//
//                 CdFreePool( &Vcb->XASector );
//                 Vcb->XASector = 0;
//                 Vcb->XADiskOffset = 0;
//             }
//
//             CdFreeDirCache( IrpContext);
//
//             //
//             //  Now, if there are no user handles to the volume, try to spark
//             //  teardown by purging the volume.
//             //
//
//             if (Vcb->VcbCleanup == 0) {
//
//                 if (NT_SUCCESS( CdPurgeVolume( IrpContext, Vcb, FALSE ))) {
//
//                     ReleaseVcb = CdCheckForDismount( IrpContext, Vcb, FALSE );
//                 }
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Free the TOC buffer if allocated.
//         //
//
//         if (CdromToc != NULL) {
//
//             CdFreePool( &CdromToc );
//         }
//
//         if (RawIsoVd != NULL) {
//
//             CdFreePool( &RawIsoVd );
//         }
//
//         if (ReleaseVcb) {
//
//             CdReleaseVcb( IrpContext, Vcb );
//         }
//         else {
//             _Analysis_assume_lock_not_held_(Vcb->VcbResource);
//         }
//
//         CdReleaseCdData( IrpContext );
//     } _SEH2_END;
//
//     //
//     //  Now send mount notification.
//     //
//
//     if (FileObjectToNotify) {
//
//         FsRtlNotifyVolumeEvent( FileObjectToNotify, FSRTL_VOLUME_MOUNT );
//         ObDereferenceObject( FileObjectToNotify );
//     }
//
//     //
//     //  Complete the request if no exception.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdOplockRequest (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine to handle oplock requests made via the
//     NtFsControlFile call.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     ULONG OplockCount = 0;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PAGED_CODE();
//
//     //
//     //  We only permit oplock requests on files.
//     //
//
//     if (CdDecodeFileObject( IrpContext,
//                             IrpSp->FileObject,
//                             &Fcb,
//                             &Ccb ) != UserFileOpen ) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Make this a waitable Irpcontext so we don't fail to acquire
//     //  the resources.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//     ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_FORCE_POST );
//
//     //
//     //  Switch on the function control code.  We grab the Fcb exclusively
//     //  for oplock requests, shared for oplock break acknowledgement.
//     //
//
//     switch (IrpSp->Parameters.FileSystemControl.FsControlCode) {
//
//     case FSCTL_REQUEST_OPLOCK_LEVEL_1 :
//     case FSCTL_REQUEST_OPLOCK_LEVEL_2 :
//     case FSCTL_REQUEST_BATCH_OPLOCK :
//     case FSCTL_REQUEST_FILTER_OPLOCK :
//
//         CdAcquireFcbExclusive( IrpContext, Fcb, FALSE );
//
//         if (IrpSp->Parameters.FileSystemControl.FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_2) {
//
//             if (Fcb->FileLock != NULL) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//                 OplockCount = (ULONG) FsRtlAreThereCurrentOrInProgressFileLocks( Fcb->FileLock );
// #else
//                 OplockCount = (ULONG) FsRtlAreThereCurrentFileLocks( Fcb->FileLock );
// #endif
//             }
//
//         } else {
//
//             OplockCount = Fcb->FcbCleanup;
//         }
//
//         break;
//
//     case FSCTL_OPLOCK_BREAK_ACKNOWLEDGE:
//     case FSCTL_OPBATCH_ACK_CLOSE_PENDING:
//     case FSCTL_OPLOCK_BREAK_NOTIFY:
//     case FSCTL_OPLOCK_BREAK_ACK_NO_2:
//
//         CdAcquireFcbShared( IrpContext, Fcb, FALSE );
//         break;
//
//     default:
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Use a try finally to free the Fcb.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb.
//         //
//
//         CdVerifyFcbOperation( IrpContext, Fcb );
//
//         //
//         //  Call the FsRtl routine to grant/acknowledge oplock.
//         //
//
//         Status = FsRtlOplockFsctrl( CdGetFcbOplock(Fcb),
//                                     Irp,
//                                     OplockCount );
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         CdLockFcb( IrpContext, Fcb );
//         Fcb->IsFastIoPossible = CdIsFastIoPossible( Fcb );
//         CdUnlockFcb( IrpContext, Fcb );
//
//         //
//         //  The oplock package will complete the Irp.
//         //
//
//         Irp = NULL;
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release all of our resources
//         //
//
//         CdReleaseFcb( IrpContext, Fcb );
//     } _SEH2_END;
//
//     //
//     //  Complete the request if there was no exception.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdLockVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the lock volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Send our notification so that folks that like to hold handles on
//     //  volumes can get out of the way.
//     //
//
//     FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_LOCK );
//
//     //
//     //  Acquire exclusive access to the Vcb.
//     //
//
//     Vcb = Fcb->Vcb;
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Vcb.
//         //
//
//         CdVerifyVcb( IrpContext, Vcb );
//
//         Status = CdLockVolumeInternal( IrpContext, Vcb, IrpSp->FileObject );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the Vcb.
//         //
//
//         CdReleaseVcb( IrpContext, Vcb );
//
//         if (_SEH2_AbnormalTermination() || !NT_SUCCESS( Status )) {
//
//             FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_LOCK_FAILED );
//         }
//     } _SEH2_END;
//
//     //
//     //  Complete the request if there haven't been any exceptions.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdUnlockVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the unlock volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb ) != UserVolumeOpen ) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Acquire exclusive access to the Vcb.
//     //
//
//     Vcb = Fcb->Vcb;
//
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  We won't check for a valid Vcb for this request.  An unlock will always
//     //  succeed on a locked volume.
//     //
//
//     Status = CdUnlockVolumeInternal( IrpContext, Vcb, IrpSp->FileObject );
//
//     //
//     //  Release all of our resources
//     //
//
//     CdReleaseVcb( IrpContext, Vcb );
//
//     //
//     //  Send notification that the volume is avaliable.
//     //
//
//     if (NT_SUCCESS( Status )) {
//
//         FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_UNLOCK );
//     }
//
//     //
//     //  Complete the request if there haven't been any exceptions.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdDismountVolume (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the dismount volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.  We only dismount a volume which
//     has been locked.  The intent here is that someone has locked the volume (they are the
//     only remaining handle).  We set the verify bit here and the user will close his handle.
//     We will dismount a volume with no user's handles in the verify path.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     if (CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb ) != UserVolumeOpen ) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Vcb = Fcb->Vcb;
//
//     //
//     //  Send dismount notification.
//     //
//
//     FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_DISMOUNT );
//
//     //
//     //  Make this request waitable.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     //
//     //  Acquire exclusive access to the Vcb,  and take the global resource to
//     //  sync. against mounts,  verifies etc.
//     //
//
//     CdAcquireCdData( IrpContext );
//     CdAcquireVcbExclusive( IrpContext, Vcb, FALSE );
//
//     //
//     //  Mark the volume as needs to be verified, but only do it if
//     //  the vcb is locked by this handle and the volume is currently mounted.
//     //
//
//     if (Vcb->VcbCondition != VcbMounted) {
//
//         Status = STATUS_VOLUME_DISMOUNTED;
//
//     } else {
//
//         //
//         //  Invalidate the volume right now.
//         //
//         //  The intent here is to make every subsequent operation
//         //  on the volume fail and grease the rails toward dismount.
//         //  By definition there is no going back from a SURPRISE.
//         //
//
//         CdLockVcb( IrpContext, Vcb );
//
//         if (Vcb->VcbCondition != VcbDismountInProgress) {
//
//             CdUpdateVcbCondition( Vcb, VcbInvalid );
//         }
//
//         SetFlag( Vcb->VcbState, VCB_STATE_DISMOUNTED );
//
//         CdUnlockVcb( IrpContext, Vcb );
//
//
//         //
//         //  Set flag to tell the close path that we want to force dismount
//         //  the volume when this handle is closed.
//         //
//
//         SetFlag( Ccb->Flags, CCB_FLAG_DISMOUNT_ON_CLOSE);
//
//         Status = STATUS_SUCCESS;
//     }
//
//     //
//     //  Release all of our resources
//     //
//
//     CdReleaseVcb( IrpContext, Vcb );
//     CdReleaseCdData( IrpContext );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     FsRtlDismountComplete( Vcb->TargetDeviceObject, Status );
//
// #endif
//
//     //
//     //  Complete the request if there haven't been any exceptions.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdIsVolumeDirty (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if a volume is currently dirty.
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
//     PIO_STACK_LOCATION IrpSp;
//
//     TYPE_OF_OPEN TypeOfOpen;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PULONG VolumeState;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current stack location and extract the output
//     //  buffer information.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Get a pointer to the output buffer.
//     //
//
//     if (Irp->AssociatedIrp.SystemBuffer != NULL) {
//
//         VolumeState = Irp->AssociatedIrp.SystemBuffer;
//
//     } else {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_USER_BUFFER );
//         return STATUS_INVALID_USER_BUFFER;
//     }
//
//     //
//     //  Make sure the output buffer is large enough and then initialize
//     //  the answer to be that the volume isn't dirty.
//     //
//
//     if (IrpSp->Parameters.FileSystemControl.OutputBufferLength < sizeof(ULONG)) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     *VolumeState = 0;
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     if (TypeOfOpen != UserVolumeOpen) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (Fcb->Vcb->VcbCondition != VcbMounted) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_VOLUME_DISMOUNTED );
//         return STATUS_VOLUME_DISMOUNTED;
//     }
//
//     //
//     //  Now set up to return the clean state.  CDs obviously can never be dirty
//     //  but we want to make sure we have enforced the full semantics of this call.
//     //
//
//     Irp->IoStatus.Information = sizeof( ULONG );
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdIsVolumeMounted (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if a volume is currently mounted.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the file object.
//     //
//
//     CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb );
//
//     if (Fcb != NULL) {
//
//         //
//         //  Disable PopUps, we want to return any error.
//         //
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS );
//
//         //
//         //  Verify the Vcb.  This will raise in the error condition.
//         //
//
//         CdVerifyVcb( IrpContext, Fcb->Vcb );
//     }
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdIsPathnameValid (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if pathname is a valid CDFS pathname.
//     We always succeed this request.
//
// Arguments:
//
//     Irp - Supplies the Irp to process.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdInvalidateVolumes (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine searches for all the volumes mounted on the same real device
//     of the current DASD handle, and marks them all bad.  The only operation
//     that can be done on such handles is cleanup and close.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     KIRQL SavedIrql;
//
//     BOOLEAN UnlockVcb = FALSE;
//
//     LUID TcbPrivilege = {SE_TCB_PRIVILEGE, 0};
//
//     HANDLE Handle;
//
//     PVCB Vcb;
//
//     PLIST_ENTRY Links;
//
//     PFILE_OBJECT FileToMarkBad;
//     PDEVICE_OBJECT DeviceToMarkBad;
//
//     //
//     //  We only allow the invalidate call to come in on our file system devices.
//     //
//
// #ifndef __REACTOS__
//     if (IrpSp->DeviceObject != CdData.FileSystemDeviceObject)  {
// #else
//     if (IrpSp->DeviceObject != CdData.FileSystemDeviceObject &&
//         IrpSp->DeviceObject != CdData.HddFileSystemDeviceObject)  {
// #endif
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     //
//     //  Check for the correct security access.
//     //  The caller must have the SeTcbPrivilege.
//     //
//
//     if (!SeSinglePrivilegeCheck( TcbPrivilege, Irp->RequestorMode )) {
//
//         CdCompleteRequest( IrpContext, Irp, STATUS_PRIVILEGE_NOT_HELD );
//
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     //
//     //  Try to get a pointer to the device object from the handle passed in.
//     //
//
// #if defined(_WIN64) && BUILD_WOW64_ENABLED
//
//     if (IoIs32bitProcess( Irp )) {
//
//         if (IrpSp->Parameters.FileSystemControl.InputBufferLength != sizeof( UINT32 )) {
//
//             CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         Handle = (HANDLE) LongToHandle( *((PUINT32) Irp->AssociatedIrp.SystemBuffer) );
//
//     } else
//
// #endif
//
//     {
//         if (IrpSp->Parameters.FileSystemControl.InputBufferLength != sizeof( HANDLE )) {
//
//             CdCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         Handle = *((PHANDLE) Irp->AssociatedIrp.SystemBuffer);
//     }
//
//     Status = ObReferenceObjectByHandle( Handle,
//                                         0,
//                                         *IoFileObjectType,
//                                         KernelMode,
//                                         (PVOID*)&FileToMarkBad, /* ReactOS Change: GCC "passing argument 5 of 'ObReferenceObjectByHandle' from incompatible pointer type" */
//                                         NULL );
//
//     if (!NT_SUCCESS(Status)) {
//
//         CdCompleteRequest( IrpContext, Irp, Status );
//         return Status;
//     }
//
//     //
//     //  Grab the DeviceObject from the FileObject.
//     //
//
//     DeviceToMarkBad = FileToMarkBad->DeviceObject;
//
//     //
//     //  We only needed the device object involved, not a reference to the file.
//     //
//
//     ObDereferenceObject( FileToMarkBad );
//
//     //
//     //  Make sure this request can wait.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//     ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_FORCE_POST );
//
//     //
//     //  Synchronise with pnp/mount/verify paths.
//     //
//
//     CdAcquireCdData( IrpContext );
//
//     //
//     //  Nothing can go wrong now.
//     //
//
//     //
//     //  Now walk through all the mounted Vcb's looking for candidates to
//     //  mark invalid.
//     //
//     //  On volumes we mark invalid, check for dismount possibility (which is
//     //  why we have to get the next link so early).
//     //
//
//     Links = CdData.VcbQueue.Flink;
//
//     while (Links != &CdData.VcbQueue) {
//
//         Vcb = CONTAINING_RECORD( Links, VCB, VcbLinks);
//
//         Links = Links->Flink;
//
//         //
//         //  If we get a match, mark the volume Bad, and also check to
//         //  see if the volume should go away.
//         //
//
//         CdLockVcb( IrpContext, Vcb );
//
//         if (Vcb->Vpb->RealDevice == DeviceToMarkBad) {
//
//             //
//             //  Take the VPB spinlock,  and look to see if this volume is the
//             //  one currently mounted on the actual device.  If it is,  pull it
//             //  off immediately.
//             //
//
//             IoAcquireVpbSpinLock( &SavedIrql );
//
// #ifdef _MSC_VER
// #pragma prefast(suppress: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//             if (DeviceToMarkBad->Vpb == Vcb->Vpb)  {
//
//                 PVPB NewVpb = Vcb->SwapVpb;
//
//                 NT_ASSERT( FlagOn( Vcb->Vpb->Flags, VPB_MOUNTED));
//                 NT_ASSERT( NULL != NewVpb);
//
//                 RtlZeroMemory( NewVpb, sizeof( VPB ) );
//
//                 NewVpb->Type = IO_TYPE_VPB;
//                 NewVpb->Size = sizeof( VPB );
//                 NewVpb->RealDevice = DeviceToMarkBad;
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(disable: 28175, "this is a filesystem driver, touching the vpb is allowed")
// #endif
//                 NewVpb->Flags = FlagOn( DeviceToMarkBad->Vpb->Flags, VPB_REMOVE_PENDING );
//                 DeviceToMarkBad->Vpb = NewVpb;
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//                 Vcb->SwapVpb = NULL;
//             }
//
//             IoReleaseVpbSpinLock( SavedIrql );
//
//             if (Vcb->VcbCondition != VcbDismountInProgress) {
//
//                 CdUpdateVcbCondition( Vcb, VcbInvalid);
//             }
//
//             CdUnlockVcb( IrpContext, Vcb );
//
//             CdAcquireVcbExclusive( IrpContext, Vcb, FALSE);
//
//             CdPurgeVolume( IrpContext, Vcb, FALSE );
//
//             UnlockVcb = CdCheckForDismount( IrpContext, Vcb, FALSE );
//
//             //
//             //  prefast: if UnlockVcb is false, then the VCB was already deleted, so we better not touch the Vcb.
//             //           tell Prefast something nice so it stops complaining about us leaking it.
//             //
//
//             __analysis_assert( UnlockVcb == TRUE );
//
//             if (UnlockVcb)  {
//
//                 CdReleaseVcb( IrpContext, Vcb);
//             }
//
//         } else {
//
//             CdUnlockVcb( IrpContext, Vcb );
//         }
//     }
//
//     CdReleaseCdData( IrpContext );
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdAllowExtendedDasdIo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine marks the CCB to indicate that the handle
//     may be used to read past the end of the volume file.  The
//     handle must be a dasd handle.
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
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (CdDecodeFileObject( IrpContext, IrpSp->FileObject, &Fcb, &Ccb ) != UserVolumeOpen ) {
//
//         Status = STATUS_INVALID_PARAMETER;
//     }
//     else {
//
//         SetFlag( Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO );
//         Status = STATUS_SUCCESS;
//     }
//
//     CdCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdScanForDismountedVcb (
//      PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine walks through the list of Vcb's looking for any which may
//     now be deleted.  They may have been left on the list because there were
//     outstanding references.
//
// Arguments:
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PVCB Vcb;
//     PLIST_ENTRY Links;
//
//     PAGED_CODE();
//
//     //
//     //  Walk through all of the Vcb's attached to the global data.
//     //
//
//     Links = CdData.VcbQueue.Flink;
//
//     while (Links != &CdData.VcbQueue) {
//
//         Vcb = CONTAINING_RECORD( Links, VCB, VcbLinks );
//
//         //
//         //  Move to the next link now since the current Vcb may be deleted.
//         //
//
//         Links = Links->Flink;
//
//         //
//         //  If dismount is already underway then check if this Vcb can
//         //  go away.
//         //
//
//         if ((Vcb->VcbCondition == VcbDismountInProgress) ||
//             (Vcb->VcbCondition == VcbInvalid) ||
//             ((Vcb->VcbCondition == VcbNotMounted) && (Vcb->VcbReference <= CDFS_RESIDUAL_REFERENCE))) {
//
//             CdCheckForDismount( IrpContext, Vcb, FALSE );
//         }
//     }
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// BOOLEAN
// CdFindPrimaryVd (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PCHAR RawIsoVd,
//      ULONG BlockFactor,
//      BOOLEAN ReturnOnError,
//      BOOLEAN VerifyVolume
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to walk through the volume descriptors looking
//     for a primary volume descriptor.  When/if a primary is found a 32-bit
//     serial number is generated and stored into the Vpb.  We also store the
//     location of the primary volume descriptor in the Vcb.
//
// Arguments:
//
//     Vcb - Pointer to the VCB for the volume.
//
//     RawIsoVd - Pointer to a sector buffer which will contain the primary
//                volume descriptor on exit, if successful.
//
//     BlockFactor - Block factor used by the current device for the TableOfContents.
//
//     ReturnOnError - Indicates that we should raise on I/O errors rather than
//         returning a FALSE value.
//
//     VerifyVolume - Indicates if we were called from the verify path.  We
//         do a few things different in this path.  We don't update the Vcb in
//         the verify path.
//
// Return Value:
//
//     BOOLEAN - TRUE if a valid primary volume descriptor found, FALSE
//               otherwise.
//
// --*/
//
// {
//     NTSTATUS Status;
//     ULONG ThisPass = 1;
//     BOOLEAN FoundVd = FALSE;
//
//     ULONG BaseSector;
//     ULONG SectorOffset;
//
//     PCDROM_TOC_LARGE CdromToc;
//
//     ULONG VolumeFlags;
//
//     PAGED_CODE();
//
//     //
//     //  If there are no data tracks, don't even bother hunting for descriptors.
//     //
//     //  This explicitly breaks various non-BlueBook compliant CDs that scribble
//     //  an ISO filesystem on media claiming only audio tracks.  Since these
//     //  disks can cause serious problems in some CDROM units, fail fast.  I admit
//     //  that it is possible that someone can still record the descriptors in the
//     //  audio track, record a data track (but fail to record descriptors there)
//     //  and still have the disk work.  As this form of error worked in NT 4.0, and
//     //  since these disks really do exist, I don't want to change them.
//     //
//     //  If we wished to support all such media (we don't), it would be neccesary
//     //  to clear this flag on finding ISO or HSG descriptors below.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_AUDIO_DISK)) {
//
//         return FALSE;
//     }
//
//     //
//     //  We will make at most two passes through the volume descriptor sequence.
//     //
//     //  On the first pass we will query for the last session.  Using this
//     //  as a starting offset we will attempt to mount the volume.  On any failure
//     //  we will go to the second pass and try without using any multi-session
//     //  information.
//     //
//     //  On the second pass we will start offset from sector zero.
//     //
//
//     while (!FoundVd && (ThisPass <= 2)) {
//
//         //
//         //  If we aren't at pass 1 then we start at sector 0.  Otherwise we
//         //  try to look up the multi-session information.
//         //
//
//         BaseSector = 0;
//
//         if (ThisPass == 1) {
//
//             CdromToc = NULL;
//
//             //
//             //  Check for whether this device supports XA and multi-session.
//             //
//
//             _SEH2_TRY {
//
//                 //
//                 //  Allocate a buffer for the last session information.
//                 //
//
//                 CdromToc = FsRtlAllocatePoolWithTag( CdPagedPool,
//                                                      sizeof( CDROM_TOC_LARGE ),
//                                                      TAG_CDROM_TOC );
//
//                 RtlZeroMemory( CdromToc, sizeof( CDROM_TOC_LARGE ));
//
//                 //
//                 //  Query the last session information from the driver.
//                 //
//
//                 Status = CdPerformDevIoCtrl( IrpContext,
//                                              IOCTL_CDROM_GET_LAST_SESSION,
//                                              Vcb->TargetDeviceObject,
//                                              CdromToc,
//                                              sizeof( CDROM_TOC_LARGE ),
//                                              FALSE,
//                                              TRUE,
//                                              NULL );
//
//                 //
//                 //  Raise an exception if there was an allocation failure.
//                 //
//
//                 if (Status == STATUS_INSUFFICIENT_RESOURCES) {
//
//                     CdRaiseStatus( IrpContext, Status );
//                 }
//
//                 //
//                 //  We don't handle any errors yet.  We will hit that below
//                 //  as we try to scan the disk.  If we have last session information
//                 //  then modify the base sector.
//                 //
//
//                 if (NT_SUCCESS( Status ) &&
//                     (CdromToc->FirstTrack != CdromToc->LastTrack)) {
//
//                     PCHAR Source, Dest;
//                     ULONG Count;
//
//                     Count = 4;
//
//                     //
//                     //  The track address is BigEndian, we need to flip the bytes.
//                     //
//
//                     Source = (PCHAR) &CdromToc->TrackData[0].Address[3];
//                     Dest = (PCHAR) &BaseSector;
//
//                     do {
//
//                         *Dest++ = *Source--;
//
//                     } while (--Count);
//
//                     //
//                     //  Now adjust the base sector by the block factor of the
//                     //  device.
//                     //
//
//                     BaseSector /= BlockFactor;
//
//                 //
//                 //  Make this look like the second pass since we are only using the
//                 //  first session.  No reason to retry on error.
//                 //
//
//                 } else {
//
//                     ThisPass += 1;
//                 }
//
//             } _SEH2_FINALLY {
//
//                 if (CdromToc != NULL) { CdFreePool( &CdromToc ); }
//             } _SEH2_END;
//         }
//
//         //
//         //  Compute the starting sector offset from the start of the session.
//         //
//
//         SectorOffset = FIRST_VD_SECTOR;
//
//         //
//         //  Start by assuming we have neither Hsg or Iso volumes.
//         //
//
//         VolumeFlags = 0;
//
//         //
//         //  Loop until either error encountered, primary volume descriptor is
//         //  found or a terminal volume descriptor is found.
//         //
//
//         while (TRUE) {
//
//             //
//             //  Attempt to read the desired sector. Exit directly if operation
//             //  not completed.
//             //
//             //  If this is pass 1 we will ignore errors in read sectors and just
//             //  go to the next pass.
//             //
//
//             if (!CdReadSectors( IrpContext,
//                                 LlBytesFromSectors( BaseSector + SectorOffset ),
//                                 SECTOR_SIZE,
//                                 (BOOLEAN) ((ThisPass == 1) || ReturnOnError),
//                                 RawIsoVd,
//                                 Vcb->TargetDeviceObject )) {
//
//                 break;
//             }
//
//             //
//             //  Check if either an ISO or HSG volume.
//             //
//
//             if (RtlEqualMemory( CdIsoId,
//                                 CdRvdId( RawIsoVd, VCB_STATE_ISO ),
//                                 VOL_ID_LEN )) {
//
//                 SetFlag( VolumeFlags, VCB_STATE_ISO );
//
//             } else if (RtlEqualMemory( CdHsgId,
//                                        CdRvdId( RawIsoVd, VCB_STATE_HSG ),
//                                        VOL_ID_LEN )) {
//
//                 SetFlag( VolumeFlags, VCB_STATE_HSG );
//
//             //
//             //  We have neither so break out of the loop.
//             //
//
//             } else {
//
//                  break;
//             }
//
//             //
//             //  Break out if the version number is incorrect or this is
//             //  a terminator.
//             //
//
//             if ((CdRvdVersion( RawIsoVd, VolumeFlags ) != VERSION_1) ||
//                 (CdRvdDescType( RawIsoVd, VolumeFlags ) == VD_TERMINATOR)) {
//
//                 break;
//             }
//
//             //
//             //  If this is a primary volume descriptor then our search is over.
//             //
//
//             if (CdRvdDescType( RawIsoVd, VolumeFlags ) == VD_PRIMARY) {
//
//                 //
//                 //  If we are not in the verify path then initialize the
//                 //  fields in the Vcb with basic information from this
//                 //  descriptor.
//                 //
//
//                 if (!VerifyVolume) {
//
//                     //
//                     //  Set the flag for the volume type.
//                     //
//
//                     SetFlag( Vcb->VcbState, VolumeFlags );
//
//                     //
//                     //  Store the base sector and sector offset for the
//                     //  primary volume descriptor.
//                     //
//
//                     Vcb->BaseSector = BaseSector;
//                     Vcb->VdSectorOffset = SectorOffset;
//                     Vcb->PrimaryVdSectorOffset = SectorOffset;
//                 }
//
//                 FoundVd = TRUE;
//                 break;
//             }
//
//             //
//             //  Indicate that we're at the next sector.
//             //
//
//             SectorOffset += 1;
//         }
//
//         ThisPass += 1;
//     }
//
//     return FoundVd;
// }
//
//
// //
// //  Local support routine
// //
//
//  BOOLEAN
// CdIsRemount (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PVCB *OldVcb
//     )
// /*++
//
// Routine Description:
//
//     This routine walks through the links of the Vcb chain in the global
//     data structure.  The remount condition is met when the following
//     conditions are all met:
//
//         If the new Vcb is a device only Mvcb and there is a previous
//         device only Mvcb.
//
//         Otherwise following conditions must be matched.
//
//             1 - The 32 serial in the current VPB matches that in a previous
//                 VPB.
//
//             2 - The volume label in the Vpb matches that in the previous
//                 Vpb.
//
//             3 - The system pointer to the real device object in the current
//                 VPB matches that in the same previous VPB.
//
//             4 - Finally the previous Vcb cannot be invalid or have a dismount
//                 underway.
//
//     If a VPB is found which matches these conditions, then the address of
//     the VCB for that VPB is returned via the pointer Vcb.
//
//     Skip over the current Vcb.
//
// Arguments:
//
//     Vcb - This is the Vcb we are checking for a remount.
//
//     OldVcb -  A pointer to the address to store the address for the Vcb
//               for the volume if this is a remount.  (This is a pointer to
//               a pointer)
//
// Return Value:
//
//     BOOLEAN - TRUE if this is in fact a remount, FALSE otherwise.
//
// --*/
//
// {
//     PLIST_ENTRY Link;
//
//     PVPB Vpb = Vcb->Vpb;
//     PVPB OldVpb;
//
//     BOOLEAN Remount = FALSE;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check whether we are looking for a device only Mvcb.
//     //
//
//     for (Link = CdData.VcbQueue.Flink;
//          Link != &CdData.VcbQueue;
//          Link = Link->Flink) {
//
//         *OldVcb = CONTAINING_RECORD( Link, VCB, VcbLinks );
//
//         //
//         //  Skip ourselves.
//         //
//
//         if (Vcb == *OldVcb) { continue; }
//
//         //
//         //  Look at the Vpb and state of the previous Vcb.
//         //
//
//         OldVpb = (*OldVcb)->Vpb;
//
//         if ((OldVpb != Vpb) &&
//             (OldVpb->RealDevice == Vpb->RealDevice) &&
//             ((*OldVcb)->VcbCondition == VcbNotMounted)) {
//
//             //
//             //  If the current disk is a raw disk then it can match a previous music or
//             //  raw disk.
//             //
//
//             if (FlagOn( Vcb->VcbState, VCB_STATE_AUDIO_DISK)) {
//
//                 if (FlagOn( (*OldVcb)->VcbState, VCB_STATE_AUDIO_DISK )) {
//
//                     //
//                     //  If we have both TOC then fail the remount if the lengths
//                     //  are different or they don't match.
//                     //
//
//                     if ((Vcb->TocLength != (*OldVcb)->TocLength) ||
//                         ((Vcb->TocLength != 0) &&
//                          !RtlEqualMemory( Vcb->CdromToc,
//                                           (*OldVcb)->CdromToc,
//                                           Vcb->TocLength ))) {
//
//                         continue;
//                     }
//
//                     Remount = TRUE;
//                     break;
//                 }
//
//             //
//             //  The current disk is not a raw disk.  Go ahead and compare
//             //  serial numbers, volume label and TOC.
//             //
//
//             }
//             else if ((OldVpb->SerialNumber == Vpb->SerialNumber) &&
//                      (Vcb->TocLength == (*OldVcb)->TocLength) &&
//                      ((Vcb->TocLength == 0) || RtlEqualMemory( Vcb->CdromToc,
//                                                                (*OldVcb)->CdromToc,
//                                                                Vcb->TocLength )) &&
//                      (Vpb->VolumeLabelLength == OldVpb->VolumeLabelLength) &&
//                      (RtlEqualMemory( OldVpb->VolumeLabel,
//                                         Vpb->VolumeLabel,
//                                         Vpb->VolumeLabelLength ))) {
//                 //
//                 //  Remember the old Vcb.  Then set the return value to
//                 //  TRUE and break.
//                 //
//
//                 Remount = TRUE;
//                 break;
//             }
//         }
//     }
//
//     return Remount;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdFindActiveVolDescriptor (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//     (ROUND_TO_PAGES( SECTOR_SIZE )) PCHAR RawIsoVd,
//      BOOLEAN VerifyVolume
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to search for a valid secondary volume descriptor that
//     we will support.  Right now we only support Joliet escape sequences for
//     the secondary descriptor.
//
//     If we don't find the secondary descriptor then we will reread the primary.
//
//     This routine will update the serial number and volume label in the Vpb.
//
// Arguments:
//
//     Vcb - This is the Vcb for the volume being mounted.
//
//     RawIsoVd - Sector buffer used to read the volume descriptors from the disks, but
//                on input should contain the PVD (ISO) in the SECOND 'sector' of the
//                buffer.
//
//     VerifyVolume - indicates we are being called by the verify path, and should
//                    not modify the Vcb fields.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     BOOLEAN FoundSecondaryVd = FALSE;
//     ULONG SectorOffset = FIRST_VD_SECTOR;
//
//     ULONG Length;
//
//     ULONG Index;
//
//     PAGED_CODE();
//
//     //
//     //  We only look for secondary volume descriptors on an Iso disk.
//     //
//
//     if ((FlagOn( Vcb->VcbState, VCB_STATE_ISO) || VerifyVolume) && !CdNoJoliet) {
//
//         //
//         //  Scan the volume descriptors from the beginning looking for a valid
//         //  secondary or a terminator.
//         //
//
//         SectorOffset = FIRST_VD_SECTOR;
//
//         while (TRUE) {
//
//             //
//             //  Read the next sector.  We should never have an error in this
//             //  path.
//             //
//
//             CdReadSectors( IrpContext,
//                            LlBytesFromSectors( Vcb->BaseSector + SectorOffset ),
//                            SECTOR_SIZE,
//                            FALSE,
//                            RawIsoVd,
//                            Vcb->TargetDeviceObject );
//
//             //
//             //  Break out if the version number or standard Id is incorrect.
//             //  Also break out if this is a terminator.
//             //
//
//             if (!RtlEqualMemory( CdIsoId, CdRvdId( RawIsoVd, VCB_STATE_JOLIET ), VOL_ID_LEN ) ||
//                 (CdRvdVersion( RawIsoVd, VCB_STATE_JOLIET ) != VERSION_1) ||
//                 (CdRvdDescType( RawIsoVd, VCB_STATE_JOLIET ) == VD_TERMINATOR)) {
//
//                 break;
//             }
//
//             //
//             //  We have a match if this is a secondary descriptor with a matching
//             //  escape sequence.
//             //
//
//             if ((CdRvdDescType( RawIsoVd, VCB_STATE_JOLIET ) == VD_SECONDARY) &&
//                 (RtlEqualMemory( CdRvdEsc( RawIsoVd, VCB_STATE_JOLIET ),
//                                  CdJolietEscape[0],
//                                  ESC_SEQ_LEN ) ||
//                  RtlEqualMemory( CdRvdEsc( RawIsoVd, VCB_STATE_JOLIET ),
//                                  CdJolietEscape[1],
//                                  ESC_SEQ_LEN ) ||
//                  RtlEqualMemory( CdRvdEsc( RawIsoVd, VCB_STATE_JOLIET ),
//                                  CdJolietEscape[2],
//                                  ESC_SEQ_LEN ))) {
//
//                 if (!VerifyVolume)  {
//
//                     //
//                     //  Update the Vcb with the new volume descriptor.
//                     //
//
//                     ClearFlag( Vcb->VcbState, VCB_STATE_ISO );
//                     SetFlag( Vcb->VcbState, VCB_STATE_JOLIET );
//
//                     Vcb->VdSectorOffset = SectorOffset;
//                 }
//
//                 FoundSecondaryVd = TRUE;
//                 break;
//             }
//
//             //
//             //  Otherwise move on to the next sector.
//             //
//
//             SectorOffset += 1;
//         }
//
//         //
//         //  If we didn't find the secondary then recover the original volume
//         //  descriptor stored in the second half of the RawIsoVd.
//         //
//
//         if (!FoundSecondaryVd) {
//
//             RtlCopyMemory( RawIsoVd,
//                            Add2Ptr( RawIsoVd, SECTOR_SIZE, PVOID ),
//                            SECTOR_SIZE );
//         }
//     }
//
//     //
//     //  If we're in the verify path,  our work is done,  since we don't want
//     //  to update any Vcb/Vpb values.
//     //
//
//     if (VerifyVolume)  {
//
//         return;
//     }
//
//     //
//     //  Compute the serial number and volume label from the volume descriptor.
//     //
//
//     Vcb->Vpb->SerialNumber = CdSerial32( RawIsoVd, SECTOR_SIZE );
//
//     //
//     //  Make sure the CD label will fit in the Vpb.
//     //
//
//     NT_ASSERT( VOLUME_ID_LENGTH * sizeof( WCHAR ) <= MAXIMUM_VOLUME_LABEL_LENGTH );
//
//     //
//     //  If this is not a Unicode label we must convert it to unicode.
//     //
//
//     if (!FlagOn( Vcb->VcbState, VCB_STATE_JOLIET )) {
//
//         //
//         //  Convert the label to unicode.  If we get any error then use a name
//         //  length of zero.
//         //
//
//         Vcb->Vpb->VolumeLabelLength = 0;
//
//         if (NT_SUCCESS( RtlOemToUnicodeN( &Vcb->Vpb->VolumeLabel[0],
//                                           MAXIMUM_VOLUME_LABEL_LENGTH,
//                                           &Length,
//                                           (PCH)CdRvdVolId( RawIsoVd, Vcb->VcbState ),
//                                           VOLUME_ID_LENGTH ))) {
//
//             Vcb->Vpb->VolumeLabelLength = (USHORT) Length;
//         }
//
//     //
//     //  We need to convert from big-endian to little endian.
//     //
//
//     } else {
//
//         CdConvertBigToLittleEndian( IrpContext,
//                                     (PCHAR) CdRvdVolId( RawIsoVd, Vcb->VcbState ),
//                                     VOLUME_ID_LENGTH,
//                                     (PCHAR) Vcb->Vpb->VolumeLabel );
//
//         Vcb->Vpb->VolumeLabelLength = VOLUME_ID_LENGTH * sizeof( WCHAR );
//     }
//
//     //
//     //  Strip the trailing spaces or zeroes from the name.
//     //
//
//     Index = Vcb->Vpb->VolumeLabelLength / sizeof( WCHAR );
//
//     while (Index > 0) {
//
//         if ((Vcb->Vpb->VolumeLabel[ Index - 1 ] != L'\0') &&
//             (Vcb->Vpb->VolumeLabel[ Index - 1 ] != L' ')) {
//
//             break;
//         }
//
//         Index -= 1;
//     }
//
//     //
//     //  Now set the final length for the name.
//     //
//
//     Vcb->Vpb->VolumeLabelLength = (USHORT) (Index * sizeof( WCHAR ));
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
