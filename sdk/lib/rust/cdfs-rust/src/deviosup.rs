#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ERESOURCE_THREAD, ExReleaseResourceForThreadLite, ExSetResourceOwnerPointer, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAllocateMdl, IoBuildDeviceIoControlRequest, IoBuildPartialMdl, IoBuildSynchronousFsdRequest, IoFreeIrp, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMakeAssociatedIrp, IoMarkIrpPending, IoReuseIrp, IoSetCompletionRoutine, IoSetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, MmProbeAndLockPages, MmUnlockPages, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_COMPLETION_ROUTINE, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, PMDL, _LOCK_OPERATION_IoReadAccess as IoReadAccess, _LOCK_OPERATION_IoWriteAccess as IoWriteAccess};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{AUDIO_PLAY_HEADER, CD_IO_CONTEXT, CD_SECTOR_CACHE_CHUNK, FCB, IRP_CONTEXT, PFCB, PIRP_CONTEXT, RIFF_HEADER, TRACK_DATA, VCB};
use ntkrnl_sys::ddk::{CCHAR, DEVICE_OBJECT, IO_STACK_LOCATION, IRP, LARGE_INTEGER, LOCK_OPERATION, MDL, _ETHREAD};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlIsNtstatusExpected};
use crate::glue::{STATUS_VERIFY_REQUIRED};
use crate::types::{RAW_DIRENT, RAW_PATH_ISO, SYSTEM_USE_XA};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{RAW_READ_INFO, TRACK_MODE_TYPE, _TRACK_MODE_TYPE_CDDA as CDDA, _TRACK_MODE_TYPE_XAForm2 as XAForm2, _TRACK_MODE_TYPE_YellowMode2 as YellowMode2};

extern "C" {
}
extern "C" {
    fn SectorOffset(byte_count: u32) -> i32;
}
extern "C" {
    fn SectorsFromBytes(byte_count: u32) -> u32;
}
extern "C" {
    fn CdRaiseStatus(IrpContext: PIRP_CONTEXT, Status: NTSTATUS) -> !;
}
extern "C" {
    fn SafeNodeType(fcb: *mut FCB) -> u32;
}
extern "C" {
    static CdAudioPlayHeader: AUDIO_PLAY_HEADER;
}
extern "C" {
    static CdXAAudioPhileHeader: RIFF_HEADER;
}
extern "C" {
    static CdXAFileHeader: RIFF_HEADER;
}
extern "C" {
    fn FlagOn(flags: u32, mask: u32) -> i32;
}
extern "C" {
    fn CdNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
}
extern "C" {
    fn ASSERT_IRP(Irp: PIRP);
}
extern "C" {
    fn ASSERT_IRP_CONTEXT(IrpContext: PIRP_CONTEXT);
}
extern "C" {
    // these mimic the MSVC __try/__except/__end
    fn _SEH2_TRY();
}
extern "C" {
    fn _SEH2_EXCEPT(code: i32);
}
extern "C" {
    fn _SEH2_END();
}
extern "C" {
    fn _SEH2_GetExceptionCode() -> i32;
}
extern "C" {
    fn CdLookupAllocation(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, CurrentFileOffset: i64, DiskOffset: *mut i64, CurrentByteCount: *mut u32);
}
extern "C" {
    fn LlSectorTruncate(offset: i64) -> i64;
}
extern "C" {
    fn SectorTruncate(size: u32) -> u32;
}
extern "C" {
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdFreePool(pool: *mut *mut c_void);
}
extern "C" {
    fn PagedCode();
}
extern "C" {
    fn CdAcquireCacheForUpdate(ctx: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdReleaseCache(ctx: *mut IRP_CONTEXT);
}
extern "system" {
    fn KeGetCurrentIrql() -> u8;
}
extern "C" {
    fn SectorsFromLlBytes(offset: i64) -> u32;
}
extern "C" {
    fn BytesFromSectors(sectors: u32) -> u32;
}
extern "C" {
    fn LlBytesFromSectors(sectors: u32) -> i64;
}
extern "C" {
    fn Min(a: u32, b: u32) -> u32;
}
extern "C" {
    fn SwapCopyUchar4(dest: *mut u32, src: *mut u8);
}
extern "C" {
    fn CdAcquireCacheForRead(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdConvertCacheToShared(IrpContext: *mut IRP_CONTEXT);
}
extern "system" {
    fn PsGetCurrentThread() -> *mut core::ffi::c_void;
}
extern "C" {
}
extern "C" {
    fn CdFreeIoContext(IoContext: *mut CD_IO_CONTEXT);
}
extern "C" {
}
extern "C" {
}
extern "C" {
    fn CdMapUserBuffer(irp_context: *mut IRP_CONTEXT, user_buffer: *mut *mut core::ffi::c_void);
}
extern "system" {
    fn Add2Ptr(ptr: *mut core::ffi::c_void, offset: u32, _type: *const i8) -> *mut core::ffi::c_void;
}
extern "C" {
    fn ClearFlag(flags: *mut u32, mask: u32);
}
extern "system" {
    fn KeFlushIoBuffers(mdl: *mut MDL, flush_io: i32, flush_cpu_cache: i32);
}
extern "C" {
    fn CdLockUserBuffer(IrpContext: PIRP_CONTEXT, ByteCount: u32, Access: u32);
}

// ---- file-scope preamble (translated) ----
pub const MAX_PARALLEL_IOS: u32 = 5;
pub const BugCheckFileId: u32 = 0x000a0000; // CDFS_BUG_CHECK_DEVIOSUP (nodetype.h:80)

#[repr(C)]
pub struct IO_RUN {
    pub DiskOffset: i64,
    pub DiskByteCount: u32,
    pub UserBuffer: *mut core::ffi::c_void,
    pub TransferBuffer: *mut core::ffi::c_void,
    pub TransferByteCount: u32,
    pub TransferBufferOffset: u32,
    pub TransferMdl: PMDL,
    pub TransferVirtualAddress: *mut core::ffi::c_void,
    pub SavedIrp: PIRP,
}
pub type PIO_RUN = *mut IO_RUN;

const FCB_STATE_MODE2FORM2_FILE: u32 = 0x00000004; // cdstruc.h
const FCB_STATE_MODE2_FILE: u32 = 0x00000008; // cdstruc.h
const FCB_STATE_DA_FILE: u32 = 0x00000010; // cdstruc.h

// SL_OVERRIDE_VERIFY_VOLUME constant (value from WinNT.h)
const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
// EXCEPTION_EXECUTE_HANDLER (value 1, matches ntkrnl_sys hal_ddk definition)
const EXCEPTION_EXECUTE_HANDLER: u32 = 1;

// Forward declarations for functions not available in our imports

const STATUS_INVALID_USER_BUFFER: NTSTATUS = 0xC00000E8u32 as i32; // ntstatus.h STATUS_INVALID_USER_BUFFER (grind fabricated 0xC0000098=STATUS_FILE_INVALID)

#[allow(non_camel_case_types)]
const RAW_SECTOR_SIZE: u32 = 2352; // placeholder, must match CDFS
const SECTOR_SHIFT: u32 = 11;      // log2 of SECTOR_SIZE
const SECTOR_SIZE: u32 = 2048;
const TAG_IO_BUFFER: u32 = 0x66626443; // nolint: const-contradiction ('fbdC' -- cdfs/cdprocs.h; fastfat's is 'OtaF')
const CdNonPagedPool: u32 = 0; // NonPagedPool value

const CD_SEC_CACHE_CHUNKS: u32 = 0x04;
const CD_SEC_CHUNK_BLOCKS: u32 = 0x18;
const PASSIVE_LEVEL: u8 = 0;

const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;

const CDFS_NTC_FCB_PATH_TABLE: u32 = 0x0303;
const CD_ATTRIBUTE_DIRECTORY: u8 = 0x02;
const CdAudioDirentSize: u32 = 32;
const CdAudioDirentsPerSector: u32 = SECTOR_SIZE / CdAudioDirentSize;
const CdAudioFileNameLength: u32 = 12;
const CdAudioFileName: [u8; CdAudioFileNameLength as usize] = [
    b'T', b'r', b'a', b'c', b'k', b'0', b'1', b'.', b'c', b'd', b'a', 0
];
const CdAudioSystemUseOffset: u32 = 24;
const AUDIO_NAME_TENS_OFFSET: u32 = 4;
const AUDIO_NAME_ONES_OFFSET: u32 = 5;
const SYSTEM_USE_XA_DA: u8 = 0x40;
const SYSTEM_XA_SIGNATURE: u8 = b'X';

const STATUS_SUCCESS: i32 = 0;
const CDFS_NTC_FCB_INDEX: u32 = 0x0304; // nodetype.h
const VcbMounted: u32 = 2; // _VCB_CONDITION (cdstruc.h: NotMounted=0,MountInProgress=1,Mounted=2)
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH // cdstruc.h  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
const IRP_CONTEXT_FLAG_TOP_LEVEL: u32 = 0x00000010; // cdstruc.h

// Constants (TODO: replace with correct numeric values)
const IRP_CONTEXT_FLAG_ALLOC_IO: u32 = 0x0100;
const VCB_STATE_AUDIO_DISK: u32 = 0x80;
const NotificationEvent: u32 = 0x00;
const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8u32 as i32; // real value (was fabricated 0xC0002001)
const STATUS_PENDING: NTSTATUS = 0x00000103;
#[no_mangle]
pub unsafe extern "C" fn CdLbnToMmSsFf(mut Blocks: u32, Msf: *mut u8) {
    Blocks += 150;
    *Msf = (Blocks % 75) as u8;
    Blocks /= 75;
    *Msf.add(1) = (Blocks % 60) as u8;
    Blocks /= 60;
    *Msf.add(2) = Blocks as u8;
}
#[no_mangle]
pub unsafe extern "C" fn CdFileTrackMode(fcb: *mut FCB) -> TRACK_MODE_TYPE {
    NT_ASSERT!(FlagOn((*fcb).FcbState, FCB_STATE_MODE2FORM2_FILE | FCB_STATE_MODE2_FILE | FCB_STATE_DA_FILE));
    if FlagOn((*fcb).FcbState, FCB_STATE_MODE2FORM2_FILE) != 0 {
        return XAForm2;
    } else if FlagOn((*fcb).FcbState, FCB_STATE_DA_FILE) != 0 {
        return CDDA;
    }
    return YellowMode2;
}
#[no_mangle]
pub unsafe extern "C" fn CdNonCachedRead(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    StartingOffset: i64,
    ByteCount: u32,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut IoRuns: [IO_RUN; MAX_PARALLEL_IOS as usize] = core::mem::zeroed();
    let mut RunCount: u32 = 0;
    let mut CleanupRunCount: u32 = 0;
    let mut UserBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut UserBufferOffset: u32 = 0;
    let mut CurrentOffset: i64 = StartingOffset;
    let mut RemainingByteCount: u32 = ByteCount;
    let mut ThisByteCount: u32 = 0;
    let mut Unaligned: i32 = 0;
    let mut FlushIoBuffers: i32 = 0;
    let mut FirstPass: i32 = 1;

    // PAGED_CODE(); (macro – no-op in translation)

    let irp = (*IrpContext).Irp;
    if (*irp).MdlAddress.is_null() {
        CdCreateUserMdl(IrpContext, ByteCount, 1, IoWriteAccess as u32);
    }

    CdMapUserBuffer(IrpContext, &mut UserBuffer);

    let vcb_ptr = (*Fcb).Vcb;
    let vcb_state = (*vcb_ptr).VcbState;
    if ((vcb_state & VCB_STATE_AUDIO_DISK) != 0)
        && ((SafeNodeType(Fcb) == CDFS_NTC_FCB_INDEX) || (SafeNodeType(Fcb) == CDFS_NTC_FCB_PATH_TABLE))
    {
        CdReadAudioSystemFile(IrpContext, Fcb, StartingOffset, ByteCount, UserBuffer);
        return STATUS_SUCCESS;
    }

    // Check if sector cache should be marked waitable
    if (SafeNodeType(Fcb) == CDFS_NTC_FCB_INDEX)
        && (!(*vcb_ptr).SectorCacheBuffer.is_null())
        && (VcbMounted == (*(*IrpContext).Vcb).VcbCondition)
    {
        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) == 0 {
            let sync_event: *mut KEVENT = (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT;
            KeInitializeEvent(sync_event, _EVENT_TYPE_NotificationEvent as u32, 0);
            (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;
        }
    }

    // _SEH2_TRY / _SEH2_FINALLY emulation
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_CleanupRunCount = core::ptr::addr_of_mut!(CleanupRunCount);
    let p_IoRuns = core::ptr::addr_of_mut!(IoRuns);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            {
                'try_block: loop {
                    // main do-while loop
                    loop {
                        // Zero the IoRuns array — write_bytes counts ELEMENTS, so cast to
                        // *mut u8 to zero the byte count (else size_of² byte overflow).
                        core::ptr::write_bytes(IoRuns.as_mut_ptr() as *mut u8, 0, core::mem::size_of::<[IO_RUN; MAX_PARALLEL_IOS as usize]>());

                        Unaligned = CdPrepareBuffers(
                            IrpContext,
                            irp,
                            Fcb,
                            UserBuffer,
                            UserBufferOffset,
                            CurrentOffset,
                            RemainingByteCount,
                            IoRuns.as_mut_ptr(),
                            &mut CleanupRunCount,
                            &mut ThisByteCount,
                        );
                        // CdPrepareBuffers returns CANT_WAIT (sentinel, != 0/1) when it hits an
                        // unaligned run on a non-wait request. Propagate so CdCommonRead posts.
                        // CleanupRunCount was written via &mut before the sentinel return, so the
                        // FINALLY below cleans up any partial run.
                        if Unaligned == STATUS_CANT_WAIT {
                            Status = STATUS_CANT_WAIT;
                            break 'try_block;
                        }
                        RunCount = CleanupRunCount;

                        // Async request with insufficient entries
                        if (ThisByteCount < RemainingByteCount) && ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) == 0 {
                            // DUMMY_PSEH: return CANT_WAIT instead of raising; posted by CdCommonRead.
                            Status = STATUS_CANT_WAIT;
                            break 'try_block;
                        }

                        // Entire I/O in a single run
                        if (RunCount == 1) && (Unaligned == 0) && (FirstPass != 0) {
                            CdSingleAsync(IrpContext, &mut IoRuns[0] as *mut IO_RUN, Fcb);
                            CleanupRunCount = 0;

                            if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0  {
                                CdWaitSync(IrpContext);
                                Status = (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status;
                            } else {
                                (*IrpContext).Flags &= !IRP_CONTEXT_FLAG_ALLOC_IO;
                                Status = STATUS_PENDING;
                            }
                            break 'try_block; // try_return
                        }

                        // Multiple I/O runs
                        CdMultipleAsync(IrpContext, Fcb, RunCount, IoRuns.as_mut_ptr());

                        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0 {
                            CdWaitSync(IrpContext);
                            Status = (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status;

                            if (Status as i32) < 0 { // !NT_SUCCESS
                                break 'try_block; // try_return
                            }

                            if Unaligned != 0
                                && CdFinishBuffers(IrpContext, IoRuns.as_mut_ptr(), RunCount, 0, 0) != 0
                            {
                                FlushIoBuffers = 1;
                            }
                            CleanupRunCount = 0;

                            RemainingByteCount -= ThisByteCount;
                            CurrentOffset += ThisByteCount as i64;
                            UserBuffer = (UserBuffer as *mut u8).add(ThisByteCount as usize) as *mut core::ffi::c_void;
                            UserBufferOffset += ThisByteCount;
                        } else {
                            (*IrpContext).Flags &= !IRP_CONTEXT_FLAG_ALLOC_IO;
                            CleanupRunCount = 0;
                            Status = STATUS_PENDING;
                            break 'try_block; // try_return, unreachable break ignored
                        }

                        FirstPass = 0;
                        if RemainingByteCount == 0 {
                            break;
                        }
                    }

                    // Flush hardware cache if needed (after normal loop completion)
                    if FlushIoBuffers != 0 {
                        let mdl = (*(*IrpContext).Irp).MdlAddress;
                        KeFlushIoBuffers(mdl, 1, 0);
                    }
                    break; // normal exit of try block
                }
            };
        },
        |_, _| {
            if (*p_CleanupRunCount) != 0 {
                CdFinishBuffers(IrpContext, (*p_IoRuns).as_mut_ptr(), *p_CleanupRunCount , 1, 0);
            }
        },
    );

    Status // return Status from the try block
}
#[no_mangle]
pub unsafe extern "C" fn CdNonCachedXARead(
    irp_context: *mut IRP_CONTEXT,
    fcb: *mut FCB,
    starting_offset: i64,
    byte_count: u32,
) -> i32 {
    const FCB_STATE_DA_FILE: u32 = 0x00000010; // cdstruc.h
    const FCB_STATE_MODE2_FILE: u32 = 0x00000008; // cdstruc.h
    const FCB_STATE_MODE2FORM2_FILE: u32 = 0x00000004; // cdstruc.h
    const VCB_STATE_AUDIO_DISK: u32 = 0x00000080; // cdstruc.h
    const MAX_PARALLEL_IOS: usize = 5;
    const YellowMode2: TRACK_MODE_TYPE = 0; // ntddcdrm.h _TRACK_MODE_TYPE (first enumerator)

    let mut status: i32 = ntkrnl_sys::base::STATUS_SUCCESS;
    let mut local_riff_header: RIFF_HEADER = core::mem::zeroed();
    let mut riff_header: *mut RIFF_HEADER = core::ptr::null_mut();
    let mut raw_reads: [RAW_READ_INFO; MAX_PARALLEL_IOS] = core::mem::zeroed();
    let mut io_runs: [IO_RUN; MAX_PARALLEL_IOS] = core::mem::zeroed();
    let mut run_count: u32 = 0;
    let mut cleanup_run_count: u32 = 0;
    let mut user_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut user_buffer_offset: u32 = 0;
    let mut current_offset: i64 = starting_offset;
    let mut remaining_byte_count: u32 = byte_count;
    let mut this_byte_count: u32 = 0;
    let mut address: u32 = 0;
    let mut trying_yellowbook_mode2: i32 = 0;
    let mut track_mode: TRACK_MODE_TYPE = core::mem::zeroed();

    PAGED_CODE!();

    if (*(*irp_context).Irp).MdlAddress.is_null() {
        CdCreateUserMdl(irp_context, byte_count, 1, IoWriteAccess);
    }

    if current_offset + remaining_byte_count as i64 > (*fcb).__bindgen_anon_1.Header.as_ref().FileSize.QuadPart {
        remaining_byte_count = ((*fcb).__bindgen_anon_1.Header.as_ref().FileSize.QuadPart - current_offset) as u32;
    }

    CdMapUserBuffer(irp_context, core::ptr::addr_of_mut!(user_buffer));

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_cleanup_run_count = core::ptr::addr_of_mut!(cleanup_run_count);
    let p_io_runs = core::ptr::addr_of_mut!(io_runs);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'try_body: loop {
                if current_offset < core::mem::size_of::<RIFF_HEADER>() as i64 {
                    if FlagOn((*fcb).FcbState, FCB_STATE_DA_FILE) != 0 {
                        if FlagOn((*(*fcb).Vcb).VcbState, VCB_STATE_AUDIO_DISK) != 0 {
                            let audio_play_header: *mut AUDIO_PLAY_HEADER = core::ptr::addr_of_mut!(local_riff_header) as *mut AUDIO_PLAY_HEADER;
                            let mut track_data: *const TRACK_DATA = (core::ptr::addr_of!((*(*(*fcb).Vcb).CdromToc).TrackData) as *const TRACK_DATA)
                                .add((*fcb).XAFileNumber as usize);

                            ntkrnl_sys::rtl::RtlCopyMemory(
                                audio_play_header as *mut core::ffi::c_void,
                                core::ptr::addr_of!(CdAudioPlayHeader) as *const core::ffi::c_void,
                                core::mem::size_of::<AUDIO_PLAY_HEADER>(),
                            );

                            (*audio_play_header).DiskID = (*(*(*fcb).Vcb).Vpb).SerialNumber;
                            (*audio_play_header).TrackNumber = (*track_data).TrackNumber as u16;

                            SwapCopyUchar4(core::ptr::addr_of_mut!(address), core::ptr::addr_of!((*track_data).Address) as *mut u8);
                            CdLbnToMmSsFf(address, core::ptr::addr_of_mut!((*audio_play_header).TrackAddress) as *mut u8);

                            SwapCopyUchar4(
                                core::ptr::addr_of_mut!((*audio_play_header).StartingSector),
                                core::ptr::addr_of!((*track_data).Address) as *mut u8,
                            );

                            track_data = (core::ptr::addr_of!((*(*(*fcb).Vcb).CdromToc).TrackData) as *const TRACK_DATA)
                                .add((*fcb).XAFileNumber as usize + 1);

                            SwapCopyUchar4(
                                core::ptr::addr_of_mut!((*audio_play_header).SectorCount),
                                core::ptr::addr_of!((*track_data).Address) as *mut u8,
                            );

                            if (*audio_play_header).SectorCount < (*audio_play_header).StartingSector {
                                (*audio_play_header).SectorCount = 0;
                            } else {
                                (*audio_play_header).SectorCount -= (*audio_play_header).StartingSector;
                            }

                            address = (*audio_play_header).SectorCount - 150;
                            CdLbnToMmSsFf(address, core::ptr::addr_of_mut!((*audio_play_header).TrackLength) as *mut u8);

                            this_byte_count = core::mem::size_of::<RIFF_HEADER>() as u32 - current_offset as u32;

                            ntkrnl_sys::rtl::RtlCopyMemory(
                                user_buffer,
                                (audio_play_header as *mut u8)
                                    .add(core::mem::size_of::<RIFF_HEADER>() - this_byte_count as usize )
                                    as *const core::ffi::c_void,
                                this_byte_count as usize,
                            );
                        } else {
                            riff_header = core::ptr::addr_of_mut!(local_riff_header);

                            ntkrnl_sys::rtl::RtlCopyMemory(
                                riff_header as *mut core::ffi::c_void,
                                core::ptr::addr_of!(CdXAAudioPhileHeader) as *const core::ffi::c_void,
                                core::mem::size_of::<RIFF_HEADER>(),
                            );

                            (*riff_header).ChunkSize += (*fcb).__bindgen_anon_1.Header.as_ref().FileSize.__bindgen_anon_1.LowPart as i32;
                            (*riff_header).RawSectors += (*fcb).__bindgen_anon_1.Header.as_ref().FileSize.__bindgen_anon_1.LowPart;

                            this_byte_count = core::mem::size_of::<RIFF_HEADER>() as u32 - current_offset as u32;

                            ntkrnl_sys::rtl::RtlCopyMemory(
                                user_buffer,
                                (riff_header as *mut u8)
                                    .add(core::mem::size_of::<RIFF_HEADER>() - this_byte_count as usize )
                                    as *const core::ffi::c_void,
                                this_byte_count as usize,
                            );
                        }
                    } else {
                        NT_ASSERT!(FlagOn((*fcb).FcbState, FCB_STATE_MODE2_FILE | FCB_STATE_MODE2FORM2_FILE));

                        riff_header = core::ptr::addr_of_mut!(local_riff_header);

                        ntkrnl_sys::rtl::RtlCopyMemory(
                            riff_header as *mut core::ffi::c_void,
                            core::ptr::addr_of!(CdXAFileHeader) as *const core::ffi::c_void,
                            core::mem::size_of::<RIFF_HEADER>(),
                        );

                        (*riff_header).ChunkSize += (*fcb).__bindgen_anon_1.Header.as_ref().FileSize.__bindgen_anon_1.LowPart as i32;
                        (*riff_header).RawSectors += (*fcb).__bindgen_anon_1.Header.as_ref().FileSize.__bindgen_anon_1.LowPart;
                        (*riff_header).Attributes = (*fcb).XAAttributes as u16;
                        (*riff_header).FileNumber = (*fcb).XAFileNumber as u8;

                        this_byte_count = core::mem::size_of::<RIFF_HEADER>() as u32 - current_offset as u32;

                        ntkrnl_sys::rtl::RtlCopyMemory(
                            user_buffer,
                            (riff_header as *mut u8)
                                .add(core::mem::size_of::<RIFF_HEADER>() - this_byte_count as usize )
                                as *const core::ffi::c_void,
                            this_byte_count as usize,
                        );
                    }

                    user_buffer = (user_buffer as *mut u8).add(this_byte_count as usize) as *mut core::ffi::c_void;
                    user_buffer_offset += this_byte_count;
                    current_offset += this_byte_count as i64;
                    remaining_byte_count -= this_byte_count;
                }

                track_mode = CdFileTrackMode(fcb);

                while remaining_byte_count != 0 {
                    if trying_yellowbook_mode2 == 0 {
                        ntkrnl_sys::rtl::RtlZeroMemory(
                            io_runs.as_mut_ptr() as *mut core::ffi::c_void,
                            core::mem::size_of_val(&io_runs),
                        );
                        ntkrnl_sys::rtl::RtlZeroMemory(
                            raw_reads.as_mut_ptr() as *mut core::ffi::c_void,
                            core::mem::size_of_val(&raw_reads),
                        );

                        CdPrepareXABuffers(
                            irp_context,
                            (*irp_context).Irp,
                            fcb,
                            user_buffer,
                            user_buffer_offset,
                            current_offset,
                            remaining_byte_count,
                            io_runs.as_mut_ptr(),
                            core::ptr::addr_of_mut!(cleanup_run_count),
                            core::ptr::addr_of_mut!(this_byte_count),
                        );
                    }

                    if cleanup_run_count != 0 {
                        run_count = cleanup_run_count;

                        CdMultipleXAAsync(irp_context, run_count, io_runs.as_mut_ptr(), raw_reads.as_mut_ptr(), track_mode);
                        CdWaitSync(irp_context);

                        status = (*(*irp_context).Irp).IoStatus.__bindgen_anon_1.Status;

                        if !ntkrnl_sys::base::NT_SUCCESS(status) {
                            if trying_yellowbook_mode2 == 0
                                && FlagOn((*fcb).FcbState, FCB_STATE_MODE2FORM2_FILE) != 0
                            {
                                trying_yellowbook_mode2 = 1;
                                track_mode = YellowMode2;
                                (*(*irp_context).__bindgen_anon_1.IoContext).Status = ntkrnl_sys::base::STATUS_SUCCESS;
                                continue;
                            }

                            break 'try_body;
                        }

                        cleanup_run_count = 0;

                        if trying_yellowbook_mode2 != 0 {
                            SetFlag(core::ptr::addr_of_mut!((*fcb).FcbState), FCB_STATE_MODE2_FILE);
                            ClearFlag(core::ptr::addr_of_mut!((*fcb).FcbState), FCB_STATE_MODE2FORM2_FILE);
                            trying_yellowbook_mode2 = 0;
                        }

                        CdFinishBuffers(irp_context, io_runs.as_mut_ptr(), run_count, 0, 1);
                    }

                    remaining_byte_count -= this_byte_count;
                    current_offset += this_byte_count as i64;
                    user_buffer = (user_buffer as *mut u8).add(this_byte_count as usize) as *mut core::ffi::c_void;
                    user_buffer_offset += this_byte_count;
                }

                KeFlushIoBuffers((*(*irp_context).Irp).MdlAddress, 1, 0);

                break 'try_body;
            }
        },
        |_, _| {
            if (*p_cleanup_run_count) != 0 {
                CdFinishBuffers(irp_context, (*p_io_runs).as_mut_ptr(), *p_cleanup_run_count , 1, 0);
            }
        },
    );


    status
}
#[no_mangle]
pub unsafe extern "C" fn CdVolumeDasdWrite(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    StartingOffset: LONGLONG,
    ByteCount: u32,
) -> NTSTATUS {
    // PAGED_CODE();

    let status: NTSTATUS;

    // RtlZeroMemory( &IoRun, sizeof( IoRun ) )
    let mut io_run: IO_RUN = core::mem::zeroed();
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(io_run).cast::<u8>(),
        0,
        core::mem::size_of::<IO_RUN>(),
    );

    io_run.DiskOffset = StartingOffset;
    io_run.DiskByteCount = ByteCount;

    CdLockUserBuffer(IrpContext, ByteCount, IoReadAccess);

    CdSingleAsync(IrpContext, core::ptr::addr_of_mut!(io_run), Fcb);

    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) != 0 {
        CdWaitSync(IrpContext);
        status = (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status;
    } else {
        ClearFlag(
            core::ptr::addr_of_mut!((*IrpContext).Flags),
            IRP_CONTEXT_FLAG_ALLOC_IO,
        );
        status = STATUS_PENDING;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn CdReadSectors(
    IrpContext: *mut IRP_CONTEXT,
    StartingOffset: i64,
    ByteCount: u32,
    ReturnError: i32,
    Buffer: *mut core::ffi::c_void,
    TargetDeviceObject: *mut DEVICE_OBJECT,
) -> i32 {
    // PAGED_CODE();

    let mut event: KEVENT = core::mem::zeroed();
    let irp: *mut IRP;

    KeInitializeEvent(&mut event as *mut _, _EVENT_TYPE_NotificationEvent, 0);

    let io_status_ptr = core::ptr::addr_of_mut!((*(*IrpContext).Irp).IoStatus) as *mut _IO_STATUS_BLOCK;

    irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as u32,
        TargetDeviceObject,
        Buffer,
        ByteCount,
        &StartingOffset as *const i64 as *mut LARGE_INTEGER,
        &mut event as *mut KEVENT,
        io_status_ptr,
    );

    if irp.is_null() {
        CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
    }

    let stack_loc: *mut IO_STACK_LOCATION = IoGetNextIrpStackLocation(irp);
    (*stack_loc).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;

    let mut status: NTSTATUS = IoCallDriver(TargetDeviceObject, irp);

    if status == STATUS_PENDING {
        status = KeWaitForSingleObject(
            &mut event as *mut KEVENT as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(status) {
            status = (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    if !NT_SUCCESS(status) {
        if ReturnError == 0 {
            CdNormalizeAndRaiseStatus(IrpContext, status);
        }
        return 0;
    } else {
        return 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdCreateUserMdl(
    IrpContext: PIRP_CONTEXT,
    BufferLength: u32,
    RaiseOnError: BOOLEAN,
    Operation: LOCK_OPERATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INSUFFICIENT_RESOURCES;
    let Mdl: PMDL;

    // PAGED_CODE(); // omitted – Rust cannot convey pageability

    UNREFERENCED_PARAMETER!(Operation as usize);
    UNREFERENCED_PARAMETER!(IrpContext as usize);

    ASSERT_IRP_CONTEXT(IrpContext);
    ASSERT_IRP((*IrpContext).Irp);
    NT_ASSERT!((*(*IrpContext).Irp).MdlAddress.is_null() as i32);

    //
    // Allocate the Mdl, and Raise if we fail.
    //

    Mdl = IoAllocateMdl(
        (*(*IrpContext).Irp).UserBuffer,
        BufferLength,
        0, // FALSE
        0, // FALSE
        (*IrpContext).Irp,
    );

    if !Mdl.is_null() {
        //
        //  Now probe the buffer described by the Irp.  If we get an exception,
        //  deallocate the Mdl and return the appropriate "expected" status.
        //

        match ntkrnl_sys::seh::try_except(
            &mut (),
            |_| {
                MmProbeAndLockPages(Mdl, (*(*IrpContext).Irp).RequestorMode, IoWriteAccess);
            },
            |_, _| EXCEPTION_EXECUTE_HANDLER as i32,
        ) {
            Ok(()) => Status = STATUS_SUCCESS,
            Err(code) => {
                // A real handler now (ntkrnl_sys::seh). Under DUMMY_PSEH this arm was
                // gated by hand on Status, because the grind emitted it as an
                // unconditional block that freed the freshly locked MDL (cdrom.sys 0x1E).
                Status = code;

                IoFreeMdl(Mdl);
                (*(*IrpContext).Irp).MdlAddress = core::ptr::null_mut();

                if FsRtlIsNtstatusExpected(Status) == 0 {
                    Status = STATUS_INVALID_USER_BUFFER;
                }
            }
        }
    }

    //
    //  Check if we are to raise or return
    //

    if Status != STATUS_SUCCESS {
        if RaiseOnError != 0 {
            CdRaiseStatus(IrpContext, Status);
        }
    }

    //
    //  Return the status code.
    //

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CdPerformDevIoCtrlEx(
    IrpContext: *mut core::ffi::c_void,
    IoControlCode: u32,
    Device: PDEVICE_OBJECT,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: u32,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: u32,
    InternalDeviceIoControl: u8,
    OverrideVerify: u8,
    Iosb: PIO_STATUS_BLOCK,
) -> i32 {
    const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
    const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32; // ntstatus.h STATUS_VERIFY_REQUIRED (grind fabricated 0xC0000020)

    let mut Status: i32;
    let Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut LocalIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let IosbToUse: PIO_STATUS_BLOCK = if !Iosb.is_null() {
        Iosb
    } else {
        &mut LocalIosb as *mut _IO_STATUS_BLOCK
    };

    let _ = IrpContext; // UNREFERENCED_PARAMETER

    (*IosbToUse).__bindgen_anon_1.Status = 0;
    (*IosbToUse).Information = 0;

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        Device,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        InternalDeviceIoControl,
        &mut Event,
        IosbToUse,
    );

    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if OverrideVerify != 0 {
        let stack_loc = IoGetNextIrpStackLocation(Irp);
        (*stack_loc).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;
    }

    Status = IoCallDriver(Device, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );
        Status = (*IosbToUse).__bindgen_anon_1.Status;
    }

    // NT_ASSERT(!(OverrideVerify && (STATUS_VERIFY_REQUIRED == Status)));
    NT_ASSERT!((!(OverrideVerify != 0 && Status == STATUS_VERIFY_REQUIRED)) as i32);

    Status
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CdPerformDevIoCtrl(
    IrpContext: *mut IRP_CONTEXT,
    IoControlCode: u32,
    Device: *mut DEVICE_OBJECT,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: u32,
    InternalDeviceIoControl: u8,
    OverrideVerify: u8,
    Iosb: *mut _IO_STATUS_BLOCK,
) -> i32 {
    PAGED_CODE!();
    CdPerformDevIoCtrlEx(
        IrpContext as *mut c_void,
        IoControlCode,
        Device,
        core::ptr::null_mut(),
        0u32,
        OutputBuffer,
        OutputBufferLength,
        InternalDeviceIoControl,
        OverrideVerify,
        Iosb,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CdPerformDevIoCtrl(
    IrpContext: *mut IRP_CONTEXT,
    IoControlCode: u32,
    Device: *mut DEVICE_OBJECT,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: u32,
    InternalDeviceIoControl: u8,
    OverrideVerify: u8,
    Iosb: *mut _IO_STATUS_BLOCK,
) -> i32 {
    PAGED_CODE!();
    CdPerformDevIoCtrlEx(
        IrpContext as *mut c_void,
        IoControlCode,
        Device,
        core::ptr::null_mut(),
        0u32,
        OutputBuffer,
        OutputBufferLength,
        InternalDeviceIoControl,
        OverrideVerify,
        Iosb,
    )
}
#[no_mangle]
pub unsafe extern "C" fn CdPrepareBuffers(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Fcb: *mut FCB,
    UserBuffer: *mut core::ffi::c_void,
    UserBufferOffset: u32,
    StartingOffset: i64,
    ByteCount: u32,
    IoRuns: *mut IO_RUN,
    RunCount: *mut u32,
    ThisByteCount: *mut u32,
) -> i32 {
    let mut FoundUnaligned: i32 = 0;
    let mut ThisIoRun: *mut IO_RUN = IoRuns;

    let mut RemainingByteCount: u32 = ByteCount;
    let mut CurrentFileOffset: i64 = StartingOffset;

    let mut CurrentUserBuffer: *mut core::ffi::c_void = UserBuffer;
    let mut CurrentUserBufferOffset: u32 = UserBufferOffset;

    let mut DiskOffset: i64 = 0;
    let mut CurrentByteCount: u32 = RemainingByteCount;

    PAGED_CODE!();

    *RunCount = 0;
    *ThisByteCount = 0;

    loop {
        *RunCount += 1;

        (*ThisIoRun).UserBuffer = CurrentUserBuffer;

        CdLookupAllocation(
            IrpContext,
            Fcb,
            CurrentFileOffset,
            &raw mut DiskOffset,
            &raw mut CurrentByteCount,
        );

        if CurrentByteCount > RemainingByteCount {
            CurrentByteCount = RemainingByteCount;
        }

        if ((DiskOffset as u32 & 2047) != 0) || // SECTOR_MASK = SECTOR_SIZE-1 (2047)
           (((CurrentByteCount & 2047) != 0) && (CurrentByteCount < 2048)) // SECTOR_SIZE = 2048
        {
            ntkrnl_sys::ASSERT!(SafeNodeType(Fcb) != 0x0304); // CDFS_NTC_FCB_INDEX

            if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) == 0 {
                // DUMMY_PSEH: cannot raise+catch. Return CANT_WAIT as a sentinel
                // (distinct from the 0/1 FoundUnaligned return) so CdNonCachedRead
                // breaks to its finally and CdCommonRead posts the request to a worker
                // thread (which runs WAIT-set), mirroring C's raise -> CdFsdDispatch
                // _SEH2_EXCEPT -> CdProcessException post. (The mask here was 0x1 =
                // IRP_CONTEXT_FLAG_ON_STACK, NOT WAIT (0x04) — a grind flag-bit bug.)
                return STATUS_CANT_WAIT;
            }

            (*ThisIoRun).TransferBufferOffset = SectorOffset(DiskOffset as u32) as u32;

            (*ThisIoRun).DiskOffset = LlSectorTruncate(DiskOffset);

            (*ThisIoRun).DiskByteCount = SectorAlign((*ThisIoRun).TransferBufferOffset + CurrentByteCount);

            if (*ThisIoRun).DiskByteCount > PAGE_SIZE as u32 {
                (*ThisIoRun).DiskByteCount = PAGE_SIZE as u32;
            }

            if (*ThisIoRun).TransferBufferOffset + CurrentByteCount > (*ThisIoRun).DiskByteCount {
                CurrentByteCount = (*ThisIoRun).DiskByteCount - (*ThisIoRun).TransferBufferOffset;
            }

            (*ThisIoRun).TransferByteCount = CurrentByteCount;

            (*ThisIoRun).TransferBuffer = FsRtlAllocatePoolWithTag(
                0, // CdNonPagedPool (NonPagedPool = 0)
                PAGE_SIZE as u32,
                0x66626443, // TAG_IO_BUFFER 'fbdC' (cdprocs.h)
            );

            (*ThisIoRun).TransferMdl = IoAllocateMdl(
                (*ThisIoRun).TransferBuffer,
                PAGE_SIZE as u32,
                0, // FALSE
                0, // FALSE
                core::ptr::null_mut(),
            );

            (*ThisIoRun).TransferVirtualAddress = (*ThisIoRun).TransferBuffer;

            if (*ThisIoRun).TransferMdl.is_null() {
                (*(*IrpContext).Irp).IoStatus.Information = 0;
                CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
            }

            MmBuildMdlForNonPagedPool((*ThisIoRun).TransferMdl);

            FoundUnaligned = 1; // TRUE

        } else {
            CurrentByteCount = SectorTruncate(CurrentByteCount);

            (*ThisIoRun).DiskOffset = DiskOffset;
            (*ThisIoRun).DiskByteCount = CurrentByteCount;

            (*ThisIoRun).TransferBuffer = CurrentUserBuffer;
            (*ThisIoRun).TransferMdl = (*Irp).MdlAddress;

            // Add2Ptr(Irp->UserBuffer, CurrentUserBufferOffset, PVOID) 
            (*ThisIoRun).TransferVirtualAddress = ((*Irp).UserBuffer as *mut u8)
                .add(CurrentUserBufferOffset as usize) as *mut core::ffi::c_void;
        }

        RemainingByteCount -= CurrentByteCount;

        *ThisByteCount += CurrentByteCount;

        if RemainingByteCount == 0 || *RunCount == 5 { // MAX_PARALLEL_IOS = 5
            break;
        }

        ThisIoRun = ThisIoRun.add(1);

        // Add2Ptr(CurrentUserBuffer, CurrentByteCount, PVOID)
        CurrentUserBuffer = (CurrentUserBuffer as *mut u8)
            .add(CurrentByteCount as usize) as *mut core::ffi::c_void;

        CurrentUserBufferOffset += CurrentByteCount;
        CurrentFileOffset += CurrentByteCount as i64;
    }

    return FoundUnaligned;
}

unsafe fn SectorAlign(x: u32) -> u32 {
    (x + (SECTOR_SIZE - 1)) / SECTOR_SIZE * SECTOR_SIZE
}
unsafe fn RawSectorAlign(x: u32) -> u32 {
    (x + (RAW_SECTOR_SIZE - 1)) / RAW_SECTOR_SIZE * RAW_SECTOR_SIZE
}

#[no_mangle]
pub unsafe extern "C" fn CdPrepareXABuffers(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Fcb: *mut FCB,
    UserBuffer: *mut core::ffi::c_void,
    UserBufferOffset: u32,
    StartingOffset: i64,
    ByteCount: u32,
    IoRuns: *mut IO_RUN,
    RunCount: *mut u32,
    ThisByteCount: *mut u32,
) {
    let mut ThisIoRun: *mut IO_RUN = IoRuns;
    let mut PerformedCopy: i32; // BOOL

    let mut RawSectorOffset: u32;
    let mut RemainingRawByteCount: u32 = ByteCount;
    let mut CurrentRawOffset: i64 = StartingOffset - core::mem::size_of::<RIFF_HEADER>() as i64;

    let mut CurrentCookedOffset: i64;
    let mut RemainingCookedByteCount: u32;

    let mut CurrentUserBuffer: *mut core::ffi::c_void = UserBuffer;
    let mut CurrentUserBufferOffset: u32 = UserBufferOffset;

    let mut DiskOffset: i64 = 0;
    let mut CurrentCookedByteCount: u32 = 0;
    let mut CurrentRawByteCount: u32;

    // PAGED_CODE();

    // Check if high part is zero for 32-bit arithmetic
    if (*(core::ptr::addr_of!(CurrentRawOffset) as *const LARGE_INTEGER)).__bindgen_anon_1.HighPart == 0 {
        let raw_idx = (CurrentRawOffset as u32) / RAW_SECTOR_SIZE;
        CurrentRawOffset = raw_idx as i64;
        CurrentCookedOffset = ((CurrentRawOffset as u32) << SECTOR_SHIFT) as i64;
        CurrentRawOffset = (CurrentRawOffset as u32 * RAW_SECTOR_SIZE) as i64;
    } else {
        CurrentRawOffset /= RAW_SECTOR_SIZE as i64;
        CurrentCookedOffset = CurrentRawOffset << SECTOR_SHIFT;
        CurrentRawOffset *= RAW_SECTOR_SIZE as i64;
    }

    RawSectorOffset = (StartingOffset - CurrentRawOffset - core::mem::size_of::<RIFF_HEADER>() as i64) as u32;
    CurrentRawByteCount = (RawSectorOffset + RemainingRawByteCount + RAW_SECTOR_SIZE - 1) / RAW_SECTOR_SIZE;
    RemainingCookedByteCount = CurrentRawByteCount << SECTOR_SHIFT;

    *RunCount = 0;
    *ThisByteCount = 0;

    loop {
        PerformedCopy = 0; // FALSE
        *RunCount += 1;

        (*ThisIoRun).UserBuffer = CurrentUserBuffer;

        // CdLookupAllocation sets out params
        CdLookupAllocation(
            IrpContext,
            Fcb,
            CurrentCookedOffset,
            core::ptr::addr_of_mut!(DiskOffset),
            core::ptr::addr_of_mut!(CurrentCookedByteCount),
        );

        if DiskOffset == (*(*Fcb).Vcb).XADiskOffset {
            CdLockVcb(IrpContext, (*Fcb).Vcb);

            if DiskOffset == (*(*Fcb).Vcb).XADiskOffset && !(*(*Fcb).Vcb).XASector.is_null() {
                CurrentRawByteCount = RAW_SECTOR_SIZE - RawSectorOffset;
                if CurrentRawByteCount > RemainingRawByteCount {
                    CurrentRawByteCount = RemainingRawByteCount;
                }

                let src = ((*(*Fcb).Vcb).XASector as *mut u8).add(RawSectorOffset as usize);
                core::ptr::copy_nonoverlapping(
                    src as *const u8,
                    CurrentUserBuffer as *mut u8,
                    CurrentRawByteCount as usize,
                );

                CdUnlockVcb(IrpContext, (*Fcb).Vcb);

                *RunCount -= 1;
                ThisIoRun = ThisIoRun.sub(1);

                PerformedCopy = 1; // TRUE
                CurrentCookedByteCount = SECTOR_SIZE;
            } else {
                CdUnlockVcb(IrpContext, (*Fcb).Vcb);
            }
        }

        if PerformedCopy == 0 {
            if CurrentCookedByteCount > RemainingCookedByteCount {
                CurrentCookedByteCount = RemainingCookedByteCount;
            }

            (*ThisIoRun).DiskOffset = DiskOffset;
            (*ThisIoRun).TransferBufferOffset = RawSectorOffset;

            if RawSectorOffset == 0 && RemainingRawByteCount >= RAW_SECTOR_SIZE {
                if CurrentCookedByteCount <= (*(*Fcb).Vcb).MaximumTransferRawSectors * SECTOR_SIZE {
                    let raw_bytes = (SectorAlign(CurrentCookedByteCount) >> SECTOR_SHIFT) * RAW_SECTOR_SIZE;
                    CurrentRawByteCount = raw_bytes;
                } else {
                    CurrentCookedByteCount = (*(*Fcb).Vcb).MaximumTransferRawSectors * SECTOR_SIZE;
                    CurrentRawByteCount = (*(*Fcb).Vcb).MaximumTransferRawSectors * RAW_SECTOR_SIZE;
                }

                while ADDRESS_AND_SIZE_TO_SPAN_PAGES(
                    CurrentUserBuffer,
                    RawSectorAlign(CurrentRawByteCount) as usize,
                ) > (*(*Fcb).Vcb).MaximumPhysicalPages as usize
                {
                    CurrentRawByteCount -= RAW_SECTOR_SIZE;
                    CurrentCookedByteCount -= SECTOR_SIZE;
                }

                while RawSectorAlign(CurrentRawByteCount) > RemainingRawByteCount {
                    CurrentRawByteCount -= RAW_SECTOR_SIZE;
                    CurrentCookedByteCount -= SECTOR_SIZE;
                }

                if CurrentRawByteCount > RemainingRawByteCount {
                    CurrentRawByteCount = RemainingRawByteCount;
                }

                (*ThisIoRun).DiskByteCount = SectorAlign(CurrentCookedByteCount);

                (*ThisIoRun).TransferBuffer = CurrentUserBuffer;
                (*ThisIoRun).TransferMdl = (*Irp).MdlAddress;
                (*ThisIoRun).TransferVirtualAddress =
                    ((*Irp).UserBuffer as *mut u8)
                        .add(CurrentUserBufferOffset as usize) as *mut core::ffi::c_void;
            } else {
                (*ThisIoRun).DiskByteCount = SECTOR_SIZE;
                CurrentCookedByteCount = SECTOR_SIZE;

                let transfer_byte_count = RAW_SECTOR_SIZE - RawSectorOffset;
                (*ThisIoRun).TransferByteCount = transfer_byte_count;
                (*ThisIoRun).TransferBufferOffset = RawSectorOffset;

                if (*ThisIoRun).TransferByteCount > RemainingRawByteCount {
                    (*ThisIoRun).TransferByteCount = RemainingRawByteCount;
                }

                CurrentRawByteCount = (*ThisIoRun).TransferByteCount;

                (*ThisIoRun).TransferBuffer = FsRtlAllocatePoolWithTag(
                    CdNonPagedPool,
                    PAGE_SIZE as u32,
                    TAG_IO_BUFFER,
                );

                (*ThisIoRun).TransferMdl = IoAllocateMdl(
                    (*ThisIoRun).TransferBuffer,
                    PAGE_SIZE as u32,
                    0, // FALSE
                    0, // FALSE
                    core::ptr::null_mut(),
                );

                (*ThisIoRun).TransferVirtualAddress = (*ThisIoRun).TransferBuffer;

                if (*ThisIoRun).TransferMdl.is_null() {
                    (*(*IrpContext).Irp).IoStatus.Information = 0;
                    CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
                }

                MmBuildMdlForNonPagedPool((*ThisIoRun).TransferMdl);
            }
        }

        RemainingRawByteCount -= CurrentRawByteCount;
        *ThisByteCount += CurrentRawByteCount;

        if RemainingRawByteCount == 0 || *RunCount == MAX_PARALLEL_IOS {
            break;
        }

        ThisIoRun = ThisIoRun.add(1);

        CurrentUserBuffer =
            (CurrentUserBuffer as *mut u8)
                .add(CurrentRawByteCount as usize) as *mut core::ffi::c_void;
        CurrentUserBufferOffset += CurrentRawByteCount;

        RawSectorOffset = 0;

        CurrentCookedOffset += CurrentCookedByteCount as i64;
        RemainingCookedByteCount -= CurrentCookedByteCount;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdFinishBuffers(
    IrpContext: *mut IRP_CONTEXT,
    IoRuns: *mut IO_RUN,
    RunCount: u32,
    FinalCleanup: i32,
    mut SaveXABuffer: i32,
) -> i32 {
    let mut FlushIoBuffers: i32 = 0;
    let mut RemainingEntries: u32 = RunCount;
    let mut ThisIoRun: *mut IO_RUN = IoRuns.add((RunCount - 1) as usize);
    let mut Vcb: *mut VCB;

    while RemainingEntries != 0 {
        if (*ThisIoRun).TransferByteCount != 0 {
            if FinalCleanup == 0 {
                let dst = (*ThisIoRun).UserBuffer;
                let src = ((*ThisIoRun).TransferBuffer as *const u8)
                    .add((*ThisIoRun).TransferBufferOffset as usize);
                let len = (*ThisIoRun).TransferByteCount as usize;
                core::ptr::copy_nonoverlapping(src, dst as *mut u8, len);
                FlushIoBuffers = 1;
            }

            if (*ThisIoRun).TransferMdl != (*(*IrpContext).Irp).MdlAddress {
                if (*ThisIoRun).TransferMdl != core::ptr::null_mut() {
                    IoFreeMdl((*ThisIoRun).TransferMdl);
                }

                if (*ThisIoRun).TransferBuffer != core::ptr::null_mut() {
                    if SaveXABuffer != 0 {
                        Vcb = (*IrpContext).Vcb;
                        CdLockVcb(IrpContext, Vcb);

                        if (*Vcb).XASector != core::ptr::null_mut() {
                            CdFreePool(core::ptr::addr_of_mut!((*Vcb).XASector));
                        }

                        (*Vcb).XASector = (*ThisIoRun).TransferBuffer;
                        (*Vcb).XADiskOffset = (*ThisIoRun).DiskOffset;
                        SaveXABuffer = 0;

                        CdUnlockVcb(IrpContext, Vcb);
                    } else {
                        CdFreePool(core::ptr::addr_of_mut!((*ThisIoRun).TransferBuffer));
                    }
                }
            }
        }

        if (*ThisIoRun).SavedIrp != core::ptr::null_mut() {
            let saved_irp = (*ThisIoRun).SavedIrp;
            if (*saved_irp).MdlAddress != core::ptr::null_mut() {
                IoFreeMdl((*saved_irp).MdlAddress);
            }
            IoFreeIrp(saved_irp);
        }

        ThisIoRun = ThisIoRun.sub(1);
        RemainingEntries -= 1;
    }

    FlushIoBuffers
}
#[no_mangle]
// i386: IO_COMPLETION_ROUTINE is NTAPI=stdcall — IofCompleteRequest invokes it
// stdcall (callee does `ret 0xc`). Defining it cdecl ("C") leaves the 3 args on
// the stack on return → ESP drifts by 12 → a later `ret` jumps to pool garbage
// (BUGCHECK 0x1E IN_PAGE_ERROR). Must be "system" (== "C" on amd64/arm64).
pub unsafe extern "system" fn CdSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Contxt: *mut core::ffi::c_void,
) -> i32 {
    let event: *mut KEVENT = Contxt as *mut KEVENT;
    // _Analysis_assume_(Contxt != NULL); // static analysis hint, omitted
    let _ = &Irp;
    let _ = &DeviceObject;
    KeSetEvent(event as *mut c_void, 0, 0);
    return 0xC0000016u32 as i32; // STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn CdFreeDirCache(IrpContext: *mut IRP_CONTEXT) {
    PagedCode();

    let vcb = (*IrpContext).Vcb;
    if !vcb.is_null() {
        let sector_cache = (*vcb).SectorCacheBuffer;
        if !sector_cache.is_null() {
            CdAcquireCacheForUpdate(IrpContext);
            CdFreePool(core::ptr::addr_of_mut!((*vcb).SectorCacheBuffer) as *mut *mut core::ffi::c_void);
            CdReleaseCache(IrpContext);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CdReadDirDataThroughCache(
    IrpContext: *mut IRP_CONTEXT,
    Run: *mut IO_RUN,
) -> u8 {
    let Vcb: *mut VCB = (*IrpContext).Vcb;
    let mut Lbn: u32 = SectorsFromLlBytes((*Run).DiskOffset);
    let mut Remaining: u32 = SectorsFromBytes((*Run).DiskByteCount);
    let mut UserBuffer: *mut u8 = (*Run).TransferBuffer as *mut u8;

    let mut Status: i32 = core::mem::zeroed();
    let mut Found: u32 = core::mem::zeroed();
    let mut BufferSectorOffset: u32 = core::mem::zeroed();
    let mut StartBlock: u32 = core::mem::zeroed();
    let mut EndBlock: u32 = 0; // filled by SwapCopyUchar4 in the reactos TOC branch
    let mut Blocks: u32 = 0;

    let mut IrpSp: *mut IO_STACK_LOCATION = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    let mut TrackData: *mut TRACK_DATA = core::mem::zeroed();

    let _Index: u32;
    let mut Buffer: *mut CD_SECTOR_CACHE_CHUNK = core::mem::zeroed();
    let mut Result: u8 = 0;

    // PAGED_CODE() expansion
    PAGED_CODE!(); // C source: PAGED_CODE() (DBG-only assert); grind fabricated RtlRaiseStatus(0xC0000185=STATUS_IO_DEVICE_ERROR)

    CdAcquireCacheForRead(IrpContext);

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'outer: loop {
                if (*Vcb).SectorCacheBuffer.is_null() {
                    CdRaiseStatus(IrpContext, STATUS_VERIFY_REQUIRED);
                }

                while Remaining != 0 {
                    Buffer = core::ptr::null_mut();

                    for Index in 0..CD_SEC_CACHE_CHUNKS {
                        let chunk = &(*Vcb).SecCacheChunks[Index as usize];
                        if chunk.BaseLbn != u32::MAX && chunk.BaseLbn <= Lbn && chunk.BaseLbn + CD_SEC_CHUNK_BLOCKS > Lbn {
                            Buffer = chunk as *const CD_SECTOR_CACHE_CHUNK as *mut CD_SECTOR_CACHE_CHUNK;
                            break;
                        }
                    }

                    if !Buffer.is_null() {
                        BufferSectorOffset = Lbn - (*Buffer).BaseLbn;
                        Found = Min(CD_SEC_CHUNK_BLOCKS - BufferSectorOffset, Remaining);

                        core::ptr::copy_nonoverlapping(
                            (*Buffer).Buffer.add(BytesFromSectors(BufferSectorOffset) as usize) as *const u8,
                            UserBuffer,
                            BytesFromSectors(Found) as usize,
                        );

                        Remaining -= Found;
                        UserBuffer = UserBuffer.add(BytesFromSectors(Found) as usize);
                        Lbn += Found;
                        continue;
                    }

                    CdReleaseCache(IrpContext);
                    CdAcquireCacheForUpdate(IrpContext);

                    Buffer = &mut (*Vcb).SecCacheChunks[(*Vcb).SecCacheLRUChunkIndex as usize] as *mut CD_SECTOR_CACHE_CHUNK;

                    StartBlock = Lbn - ((Lbn - 16) % CD_SEC_CHUNK_BLOCKS);

                    #[cfg(feature = "reactos")]
                    if !(*Vcb).CdromToc.is_null() {
                        TrackData = &(*Vcb).CdromToc.as_ref().unwrap().TrackData[((*Vcb).CdromToc.as_ref().unwrap().LastTrack
                            - (*Vcb).CdromToc.as_ref().unwrap().FirstTrack
                            + 1) as usize] as *const TRACK_DATA as *mut TRACK_DATA;

                        SwapCopyUchar4(core::ptr::addr_of_mut!(EndBlock), core::ptr::addr_of!((*TrackData).Address) as *const u8 as *mut u8);

                        Blocks = EndBlock - StartBlock;

                        if Blocks > CD_SEC_CHUNK_BLOCKS {
                            Blocks = CD_SEC_CHUNK_BLOCKS;
                        }
                    } else {
                        // The C's `#ifdef __REACTOS__ ... } else { Blocks = CD_SEC_CHUNK_BLOCKS; }`:
                        // a disk-based ISO has no TOC. This arm was translated as
                        // `cfg(not(feature = "reactos"))`, so with `reactos` on (the default) and
                        // no TOC, Blocks stayed 0 and the first sector-cache read raised
                        // STATUS_INVALID_PARAMETER -- every disk-based ISO mount failed, surfacing
                        // as 0x6B/STATUS_FILE_CORRUPT_ERROR at boot.
                        Blocks = CD_SEC_CHUNK_BLOCKS;
                    }
                    #[cfg(not(feature = "reactos"))]
                    {
                        Blocks = CD_SEC_CHUNK_BLOCKS;
                    }

                    if 0 == Blocks || Lbn < 16 {
                        CdRaiseStatus(IrpContext, STATUS_INVALID_PARAMETER);
                    }

                    IoReuseIrp((*Vcb).SectorCacheIrp, 0i32);
                    KeClearEvent(core::ptr::addr_of_mut!((*Vcb).SectorCacheEvent) as *mut c_void);
                    (*(*Vcb).SectorCacheIrp).Tail.Overlay.Thread = PsGetCurrentThread() as *mut _ETHREAD;

                    IrpSp = IoGetNextIrpStackLocation((*Vcb).SectorCacheIrp);
                    (*IrpSp).MajorFunction = IRP_MJ_READ as u8;

                    IoAllocateMdl(
                        (*Buffer).Buffer as *mut core::ffi::c_void,
                        BytesFromSectors(Blocks),
                        0,
                        0,
                        (*Vcb).SectorCacheIrp,
                    );

                    if (*(*Vcb).SectorCacheIrp).MdlAddress.is_null() {
                        (*IrpContext).Irp.as_mut().unwrap().IoStatus.Information = 0;
                        CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
                    }

                    // try { MmProbeAndLockPages } except (EXECUTE_HANDLER) { free the MDL } — the
                    // NULL MdlAddress check below then raises INSUFFICIENT_RESOURCES.
                    if ntkrnl_sys::seh::try_except(
                        &mut (),
                        |_| {
                            MmProbeAndLockPages((*(*Vcb).SectorCacheIrp).MdlAddress, 0i8, IoWriteAccess as u32);
                        },
                        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
                    )
                    .is_err()
                    {
                        IoFreeMdl((*(*Vcb).SectorCacheIrp).MdlAddress);
                        (*(*Vcb).SectorCacheIrp).MdlAddress = core::ptr::null_mut();
                    }

                    if (*(*Vcb).SectorCacheIrp).MdlAddress.is_null() {
                        CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
                    }

                    (*Buffer).BaseLbn = u32::MAX;

                    (*IrpSp).Parameters.Read.Length = BytesFromSectors(Blocks);
                    (*IrpSp).Parameters.Read.ByteOffset.QuadPart = LlBytesFromSectors(StartBlock);

                    IoSetCompletionRoutine(
                        (*Vcb).SectorCacheIrp,
                        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSyncCompletionRoutine as usize),
                        &mut (*Vcb).SectorCacheEvent as *mut KEVENT as *mut core::ffi::c_void,
                        1,
                        1,
                        1,
                    );

                    (*(*Vcb).SectorCacheIrp).UserIosb = &mut Iosb as *mut _IO_STATUS_BLOCK;

                    Status = IoCallDriver((*Vcb).TargetDeviceObject, (*Vcb).SectorCacheIrp);

                    if STATUS_PENDING == Status {
                        KeWaitForSingleObject(
                            &(*Vcb).SectorCacheEvent as *const KEVENT as *mut core::ffi::c_void,
                            _KWAIT_REASON_Executive,
                            0i8,
                            0,
                            core::ptr::null_mut(),
                        );
                        Status = (*(*Vcb).SectorCacheIrp).IoStatus.__bindgen_anon_1.Status;
                    }

                    (*(*Vcb).SectorCacheIrp).UserIosb = core::ptr::null_mut();

                    MmUnlockPages((*(*Vcb).SectorCacheIrp).MdlAddress);
                    IoFreeMdl((*(*Vcb).SectorCacheIrp).MdlAddress);
                    (*(*Vcb).SectorCacheIrp).MdlAddress = core::ptr::null_mut();

                    if !NT_SUCCESS(Status) {
                        // try_leave(Status): Result stays FALSE; the finally releases the cache.
                        break 'outer;
                    }

                    (*Buffer).BaseLbn = StartBlock;
                    (*Vcb).SecCacheLRUChunkIndex = ((*Vcb).SecCacheLRUChunkIndex + 1) % CD_SEC_CACHE_CHUNKS;

                    CdConvertCacheToShared(IrpContext);
                }

                Result = 1;
                break 'outer;
            }
        },
        |_, _| {
            CdReleaseCache(IrpContext);
        },
    );

    Result
}

#[no_mangle]
pub unsafe extern "C" fn CdMultipleAsync(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    RunCount: ULONG,
    IoRuns: *mut IO_RUN,
) {
    // PAGED_CODE();

    let mut CompletionRoutine: PIO_COMPLETION_ROUTINE;
    let mut IrpSp: *mut IO_STACK_LOCATION;
    let mut Mdl: *mut MDL;
    let mut Irp: *mut IRP;
    let MasterIrp: *mut IRP;
    let _UnwindRunCount: u32;
    let mut UseSectorCache: BOOLEAN;

    CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiSyncCompletionRoutine as usize);

    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) == 0  {
        CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiAsyncCompletionRoutine as usize);
    }

    if (SafeNodeType(Fcb) == CDFS_NTC_FCB_INDEX) &&
        ((*Fcb).Vcb as *mut u8).is_null() == false && // Assuming SectorCacheBuffer is at offset? Need to correct.
        // Actually: (NULL != Fcb->Vcb->SectorCacheBuffer)
        // Will implement as:
        !((*(*Fcb).Vcb).SectorCacheBuffer.is_null()) &&
        ((*(*IrpContext).Vcb).VcbCondition == VcbMounted)
    {
        UseSectorCache = 1;
    } else {
        UseSectorCache = 0;
    }

    MasterIrp = (*IrpContext).Irp;

    let mut i = 0;
    while i < RunCount {
        if UseSectorCache != 0 {
            if CdReadDirDataThroughCache(IrpContext, IoRuns.offset(i as isize)) == 0 {
                UseSectorCache = 0;
                i = 0;
                continue;
            }
            i += 1;
            continue;
        }

        let run = &mut *IoRuns.offset(i as isize);
        run.SavedIrp = IoMakeAssociatedIrp(
            MasterIrp,
            ((*(*(*IrpContext).Vcb).TargetDeviceObject).StackSize + 1) as CCHAR,
        );
        Irp = run.SavedIrp;

        if Irp.is_null() {
            (*(*IrpContext).Irp).IoStatus.Information = 0;
            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        Mdl = IoAllocateMdl(
            run.TransferVirtualAddress,
            run.DiskByteCount,
            0,
            0,
            Irp,
        );

        if Mdl.is_null() {
            (*(*IrpContext).Irp).IoStatus.Information = 0;
            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        IoBuildPartialMdl(
            run.TransferMdl,
            Mdl,
            run.TransferVirtualAddress,
            run.DiskByteCount,
        );

        IoSetNextIrpStackLocation(Irp);
        IrpSp = IoGetCurrentIrpStackLocation(Irp);

        (*IrpSp).MajorFunction = IRP_MJ_READ;
        (*IrpSp).Parameters.Read.Length = run.DiskByteCount;
        (*IrpSp).Parameters.Read.ByteOffset.QuadPart = run.DiskOffset;

        IoSetCompletionRoutine(
            Irp,
            CompletionRoutine,
            (*IrpContext).__bindgen_anon_1.IoContext as *mut c_void,
            1,
            1,
            1,
        );

        IrpSp = IoGetNextIrpStackLocation(Irp);

        (*IrpSp).MajorFunction = IRP_MJ_READ;
        (*IrpSp).Parameters.Read.Length = run.DiskByteCount;
        (*IrpSp).Parameters.Read.ByteOffset.QuadPart = run.DiskOffset;

        i += 1;
    }

    if UseSectorCache != 0 {
        if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) != 0 {
            (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            KeSetEvent(
                (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void,
                0,
                0,
            );
        }
        return;
    }

    (*(*IrpContext).__bindgen_anon_1.IoContext).IrpCount = RunCount as i32;
    (*(*IrpContext).__bindgen_anon_1.IoContext).MasterIrp = MasterIrp;

    (*MasterIrp).AssociatedIrp.IrpCount = 1;

    if (FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) == 0) &&
        (FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_TOP_LEVEL) != 0)
    {
        NT_ASSERT!((((*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId as ERESOURCE_THREAD) == PsGetCurrentThread() as ERESOURCE_THREAD) as i32);

        let io_context_ptr = (*IrpContext).__bindgen_anon_1.IoContext as usize;
        (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.__bindgen_anon_1.as_mut().ResourceThreadId = (io_context_ptr | 3) as ERESOURCE_THREAD;

        ExSetResourceOwnerPointer(
            (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().Resource,
            (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId as *mut c_void,
        );
    }

    let mut j = 0;
    while j < RunCount {
        Irp = (*IoRuns.offset(j as isize)).SavedIrp;
        (*IoRuns.offset(j as isize)).SavedIrp = core::ptr::null_mut();

        if !Irp.is_null() {
            IoCallDriver((*(*IrpContext).Vcb).TargetDeviceObject, Irp);
        }
        j += 1;
    }
}

// Forward declarations for undefined symbols

const IOCTL_CDROM_RAW_READ: u32 = 0x2403E;
#[no_mangle]
pub unsafe extern "C" fn CdMultipleXAAsync(
    IrpContext: PIRP_CONTEXT,
    RunCount: u32,
    IoRuns: *mut IO_RUN,
    RawReads: *mut RAW_READ_INFO,
    TrackMode: TRACK_MODE_TYPE,
) {
    let mut IrpSp: *mut IO_STACK_LOCATION;
    let mut Mdl: PMDL;
    let mut Irp: PIRP;
    let MasterIrp: PIRP;
    let _UnwindRunCount: u32;
    let mut RawByteCount: u32;

    let mut ThisIoRun = IoRuns;
    let mut ThisRawRead = RawReads;

    // PAGED_CODE() call
    PAGED_CODE!();

    // Initialize MasterIrp
    MasterIrp = (*IrpContext).Irp;

    // Iterate through the runs, set up associated IRPs
    for _UnwindRunCount in 0..RunCount {
        // Create an associated IRP
        let stack_size = (*(*(*IrpContext).Vcb).TargetDeviceObject).StackSize;
        let stack_count = stack_size.wrapping_add(1) as i8;
        (*ThisIoRun).SavedIrp = IoMakeAssociatedIrp(MasterIrp, stack_count);
        Irp = (*ThisIoRun).SavedIrp;

        if Irp.is_null() {
            (*(*IrpContext).Irp).IoStatus.Information = 0;
            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
            // unreachable
        }

        // Validate byte count
        // C: `NT_ASSERT(!SectorOffset(...))` — logical not, so `== 0`. Spelled with Rust's
        // bitwise `!` this could never be zero and the assertion was vacuous.
        NT_ASSERT!((SectorOffset((*ThisIoRun).DiskByteCount) == 0) as i32);

        RawByteCount = SectorsFromBytes((*ThisIoRun).DiskByteCount) * RAW_SECTOR_SIZE;

        // Allocate Mdl
        Mdl = IoAllocateMdl(
            (*ThisIoRun).TransferVirtualAddress,
            RawByteCount,
            0,
            0,
            Irp,
        );

        if Mdl.is_null() {
            (*(*IrpContext).Irp).IoStatus.Information = 0;
            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        IoBuildPartialMdl(
            (*ThisIoRun).TransferMdl,
            Mdl,
            (*ThisIoRun).TransferVirtualAddress,
            RawByteCount,
        );

        // Get first IRP stack location in associated IRP
        IoSetNextIrpStackLocation(Irp);
        IrpSp = IoGetCurrentIrpStackLocation(Irp);

        // Set up the stack for a cooked read (values won't be used)
        (*IrpSp).MajorFunction = IRP_MJ_READ;
        (*IrpSp).Parameters.Read.Length = (*ThisIoRun).DiskByteCount;
        (*IrpSp).Parameters.Read.ByteOffset.QuadPart = (*ThisIoRun).DiskOffset;

        // Completion routine
        IoSetCompletionRoutine(
            Irp,
            core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiSyncCompletionRoutine as usize),
            (*IrpContext).__bindgen_anon_1.IoContext as *mut c_void,
            1,
            1,
            1,
        );

        // Next IRP stack location for the disk driver
        IrpSp = IoGetNextIrpStackLocation(Irp);

        // Setup raw read
        (*ThisRawRead).DiskOffset.QuadPart = (*ThisIoRun).DiskOffset;
        (*ThisRawRead).SectorCount = (*ThisIoRun).DiskByteCount >> SECTOR_SHIFT;
        (*ThisRawRead).TrackMode = TrackMode;

        (*IrpSp).MajorFunction = IRP_MJ_DEVICE_CONTROL;
        (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength = (*ThisRawRead).SectorCount * RAW_SECTOR_SIZE;
        (*Irp).UserBuffer = (*ThisIoRun).TransferVirtualAddress;
        (*IrpSp).Parameters.DeviceIoControl.InputBufferLength = core::mem::size_of::<RAW_READ_INFO>() as u32;
        (*IrpSp).Parameters.DeviceIoControl.Type3InputBuffer = ThisRawRead as *mut c_void;
        (*IrpSp).Parameters.DeviceIoControl.IoControlCode = IOCTL_CDROM_RAW_READ;

        // Move to next run/rawread (equivalent to pointer increment)
        ThisIoRun = ThisIoRun.add(1);
        ThisRawRead = ThisRawRead.add(1);
    }

    // Set associated IRP count on master IRP (plus one to keep the I/O system from completing)
    (*(*IrpContext).__bindgen_anon_1.IoContext).IrpCount = RunCount as i32;
    (*(*IrpContext).__bindgen_anon_1.IoContext).MasterIrp = MasterIrp;

    // Set master IRP count to 1 so that on last associated IRP completion the master is completed
    (*MasterIrp).AssociatedIrp.IrpCount = 1;

    // Issue all prepared IRPs
    for UnwindRunCount in 0..RunCount {
        Irp = (*IoRuns.add(UnwindRunCount as usize)).SavedIrp;
        (*IoRuns.add(UnwindRunCount as usize)).SavedIrp = core::ptr::null_mut();

        // Call driver; ignore return value as error is handled in completion routine
        let _ = IoCallDriver((*(*IrpContext).Vcb).TargetDeviceObject, Irp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdSingleAsync(
    IrpContext: *mut IRP_CONTEXT,
    Run: *mut IO_RUN,
    Fcb: *mut FCB,
) {
    PAGED_CODE!();

    if (SafeNodeType(Fcb) == CDFS_NTC_FCB_INDEX)
        && !(*(*Fcb).Vcb).SectorCacheBuffer.is_null()
        && (VcbMounted == (*(*IrpContext).Vcb).VcbCondition)
    {
        if CdReadDirDataThroughCache(IrpContext, Run) != 0 {
            if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0 {
                (*(*IrpContext).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                KeSetEvent(
                    (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void ,
                    0,
                    0,
                );
            }
            return;
        }
    }

    let CompletionRoutine: PIO_COMPLETION_ROUTINE;

    if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0 {
        CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSingleSyncCompletionRoutine as usize);
    } else {
        CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSingleAsyncCompletionRoutine as usize);

        if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_TOP_LEVEL) != 0 {
            let io_ctx = (*IrpContext).__bindgen_anon_1.IoContext;
            NT_ASSERT!(if ((*io_ctx).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId as ERESOURCE_THREAD) == PsGetCurrentThread() as ERESOURCE_THREAD
                    { 1 }
                else
                    { 0 });

            (*io_ctx).__bindgen_anon_1.__bindgen_anon_1.as_mut().ResourceThreadId =
                ((io_ctx as ULONG_PTR) | 3) as ERESOURCE_THREAD;

            ExSetResourceOwnerPointer(
                (*io_ctx).__bindgen_anon_1.__bindgen_anon_1.as_ref().Resource,
                (*io_ctx).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId as *mut core::ffi::c_void,
            );
        }
    }

    IoSetCompletionRoutine(
        (*IrpContext).Irp,
        CompletionRoutine,
        (*IrpContext).__bindgen_anon_1.IoContext as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    let IrpSp: PIO_STACK_LOCATION = IoGetNextIrpStackLocation((*IrpContext).Irp);

    (*IrpSp).MajorFunction = (*IrpContext).MajorFunction;
    (*IrpSp).Parameters.Read.Length = (*Run).DiskByteCount;
    (*IrpSp).Parameters.Read.ByteOffset.QuadPart = (*Run).DiskOffset;

    IoCallDriver((*(*IrpContext).Vcb).TargetDeviceObject, (*IrpContext).Irp);
}
#[no_mangle]
pub unsafe extern "C" fn CdWaitSync(IrpContext: *mut IRP_CONTEXT) {
    // PAGED_CODE();  // placeholder for pageable code check
    KeWaitForSingleObject(
        (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void ,
        _KWAIT_REASON_Executive,
        0i8,
        0, // FALSE
        core::ptr::null_mut() // NULL timeout
    );
    KeClearEvent(
        (*(*IrpContext).__bindgen_anon_1.IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void
    );
}
#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — see CdSyncCompletionRoutine note.
pub unsafe extern "system" fn CdMultiSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> i32 {
    unsafe {
        let IoContext = Context as *mut CD_IO_CONTEXT;
        // _Analysis_assume_(Context != NULL) -- intentional no-op

        AssertVerifyDeviceIrp!(Irp);

        if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
            // InterlockedExchange( &IoContext->Status, Irp->IoStatus.Status )
            (*(&raw mut (*IoContext).Status as *mut core::sync::atomic::AtomicI32))
                .swap((*Irp).IoStatus.__bindgen_anon_1.Status, core::sync::atomic::Ordering::SeqCst);
            (*(*IoContext).MasterIrp).IoStatus.Information = 0;
        }

        // We must do this here since IoCompleteRequest won't get a chance on this associated Irp.
        IoFreeMdl((*Irp).MdlAddress);
        IoFreeIrp(Irp);

        if InterlockedDecrement(&raw mut (*IoContext).IrpCount) == 0 {
            (*(*IoContext).MasterIrp).IoStatus.__bindgen_anon_1.Status = (*IoContext).Status;
            KeSetEvent((*IoContext).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void, 0, 0);
        }

        let _ = DeviceObject; // UNREFERENCED_PARAMETER

        STATUS_MORE_PROCESSING_REQUIRED
    }
}
#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — see CdSyncCompletionRoutine note.
pub unsafe extern "system" fn CdMultiAsyncCompletionRoutine(
    _DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let IoContext = Context as *mut CD_IO_CONTEXT;
    // _Analysis_assume_(Context != NULL); (annotation, omitted)

    AssertVerifyDeviceIrp!(Irp);

    // UNREFERENCED_PARAMETER( DeviceObject ); handled by prefix _DeviceObject

    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        let status_ptr: *mut NTSTATUS = core::ptr::addr_of_mut!((*IoContext).Status);
        let new_val = (*Irp).IoStatus.__bindgen_anon_1.Status;
        (*(status_ptr as *mut core::sync::atomic::AtomicI32)).swap(new_val, core::sync::atomic::Ordering::SeqCst);
        // InterlockedExchange equivalent
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*IoContext).IrpCount)) == 0 {
        IoMarkIrpPending((*IoContext).MasterIrp);

        (*(*IoContext).MasterIrp).IoStatus.__bindgen_anon_1.Status = (*IoContext).Status;
        (*(*IoContext).MasterIrp).IoStatus.Information = 0;

        if NT_SUCCESS((*(*IoContext).MasterIrp).IoStatus.__bindgen_anon_1.Status) {
            (*(*IoContext).MasterIrp).IoStatus.Information = (*IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().RequestedByteCount as _;
        }

        // _Analysis_assume_lock_held_(...); (annotation, omitted)
        ExReleaseResourceForThreadLite((*IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().Resource, (*IoContext).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId);

        CdFreeIoContext(IoContext);

        return STATUS_SUCCESS;
    } else {
        IoFreeMdl((*Irp).MdlAddress);
        IoFreeIrp(Irp);

        return 0xC0000016u32 as i32; // STATUS_MORE_PROCESSING_REQUIRED
    }
}
#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — see CdSyncCompletionRoutine note.
pub unsafe extern "system" fn CdSingleSyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> i32 {
    ntkrnl_sys::ASSERT!(!Context.is_null());
    let _ = DeviceObject;

    AssertVerifyDeviceIrp!(Irp);

    if (*Irp).IoStatus.__bindgen_anon_1.Status < 0 {
        (*Irp).IoStatus.Information = 0;
    }

    let cd_context: *mut CD_IO_CONTEXT = Context as *mut CD_IO_CONTEXT;
    let event_ptr = (*cd_context).__bindgen_anon_1.SyncEvent.as_mut() as *mut KEVENT as *mut c_void;
    KeSetEvent(event_ptr, 0, 0);

    // STATUS_MORE_PROCESSING_REQUIRED (0xC0000016) — tells the I/O manager to STOP
    // completing the IRP so the synchronous caller (waiting on SyncEvent) owns it.
    // The grind wrote 0x103 (STATUS_PENDING) with a wrong comment → the IRP got
    // completed/freed under us → use-after-free during the mount descriptor read.
    0xC0000016u32 as i32
}
#[no_mangle]
// i386 NTAPI=stdcall IO_COMPLETION_ROUTINE — see CdSyncCompletionRoutine note.
pub unsafe extern "system" fn CdSingleAsyncCompletionRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let io_context = Context as *mut CD_IO_CONTEXT;

    let _ = &DeviceObject; // UNREFERENCED_PARAMETER

    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

    AssertVerifyDeviceIrp!(Irp);

    (*Irp).IoStatus.Information = 0;

    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        (*Irp).IoStatus.Information = (*io_context).__bindgen_anon_1.__bindgen_anon_1.as_ref().RequestedByteCount as _;
    }

    IoMarkIrpPending(Irp);

    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
    ExReleaseResourceForThreadLite((*io_context).__bindgen_anon_1.__bindgen_anon_1.as_ref().Resource, (*io_context).__bindgen_anon_1.__bindgen_anon_1.as_ref().ResourceThreadId);

    CdFreeIoContext(io_context);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CdReadAudioSystemFile(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    StartingOffset: LONGLONG,
    ByteCount: ULONG,
    SystemBuffer: *mut c_void,
) {
    let RawPath: *mut RAW_PATH_ISO;
    let mut RawDirent: *mut RAW_DIRENT;
    let mut CurrentTrack: u32;
    let mut SectorOffset: u32;
    let mut EntryCount: u32;
    let mut TrackOnes: u8;
    let mut TrackTens: u8;
    let mut ThisTrack: *mut TRACK_DATA;
    let mut CurrentOffset: LONGLONG;
    let mut CurrentSector: *mut u8;
    let mut SystemUse: *mut SYSTEM_USE_XA;
    let mut BytesToCopy: u32;

    const LOCAL_BUF_SIZE: usize = core::mem::offset_of!(RAW_DIRENT, FileId) + 12;
    let mut LocalBuffer: [u8; LOCAL_BUF_SIZE] = [0u8; LOCAL_BUF_SIZE];

    PAGED_CODE!();

    if SafeNodeType(Fcb) == CDFS_NTC_FCB_PATH_TABLE {
        NT_ASSERT!((StartingOffset == 0) as i32);

        RawPath = LocalBuffer.as_mut_ptr() as *mut RAW_PATH_ISO;
        RtlZeroMemory(RawPath as *mut c_void, LOCAL_BUF_SIZE);

        (*RawPath).DirIdLen = 1;
        (*RawPath).ParentNum = 1;
        (*RawPath).DirId[0] = b'\0';

        BytesToCopy = (core::mem::offset_of!(RAW_PATH_ISO, DirId) as u32) + 2;
        if BytesToCopy > ByteCount {
            BytesToCopy = ByteCount;
        }
        RtlCopyMemory(SystemBuffer, RawPath as *const c_void, BytesToCopy as usize);
    } else {
        CurrentTrack = 0;

        if StartingOffset == 0 {
            RawDirent = SystemBuffer as *mut RAW_DIRENT;

            RtlZeroMemory(
                RawDirent as *mut c_void,
                core::mem::offset_of!(RAW_DIRENT, FileId),
            );

            (*RawDirent).DirLen = (core::mem::offset_of!(RAW_DIRENT, FileId) + 1) as u8;
            (*RawDirent).FileIdLen = 1;
            (*RawDirent).FileId[0] = b'\0';
            (*RawDirent).FlagsISO |= CD_ATTRIBUTE_DIRECTORY;

            (*RawDirent).RecordTime[0] = 95;
            (*RawDirent).RecordTime[1] = 1;
            (*RawDirent).RecordTime[2] = 1;

            SectorOffset = (*RawDirent).DirLen as u32;
            RawDirent = (RawDirent as *mut u8).add(SectorOffset as usize) as *mut RAW_DIRENT;

            RtlZeroMemory(
                RawDirent as *mut c_void,
                core::mem::offset_of!(RAW_DIRENT, FileId),
            );

            (*RawDirent).DirLen = (core::mem::offset_of!(RAW_DIRENT, FileId) + 1) as u8;
            (*RawDirent).FileIdLen = 1;
            (*RawDirent).FileId[0] = b'\x01';
            (*RawDirent).FlagsISO |= CD_ATTRIBUTE_DIRECTORY;

            (*RawDirent).RecordTime[0] = 95;
            (*RawDirent).RecordTime[1] = 1;
            (*RawDirent).RecordTime[2] = 1;

            SectorOffset += (*RawDirent).DirLen as u32;
            EntryCount = 2;
        } else {
            CurrentOffset = 0;
            loop {
                CurrentTrack += CdAudioDirentsPerSector;
                CurrentOffset += SECTOR_SIZE as i64;
                if CurrentOffset >= StartingOffset {
                    break;
                }
            }
            CurrentTrack -= 2;
            SectorOffset = 0;
            EntryCount = 0;
        }

        CurrentSector = SystemBuffer as *mut u8;
        BytesToCopy = SECTOR_SIZE;

        loop {
            loop {
                if CurrentTrack >= (*(*IrpContext).Vcb).TrackCount {
                    break;
                }
                ThisTrack = &raw mut (*(*(*IrpContext).Vcb).CdromToc).TrackData[CurrentTrack as usize];

                RawDirent = (CurrentSector.add(SectorOffset as usize)) as *mut RAW_DIRENT;
                RtlZeroMemory(RawDirent as *mut c_void, CdAudioDirentSize as usize);

                (*RawDirent).DirLen = CdAudioDirentSize as u8;
                (*RawDirent).FileIdLen = CdAudioFileNameLength as u8;

                RtlCopyMemory(
                    &raw mut (*RawDirent).FileId[0] as *mut u8 as *mut c_void,
                    &CdAudioFileName as *const u8 as *const c_void,
                    CdAudioFileNameLength as usize,
                );

                (*RawDirent).RecordTime[0] = 95;
                (*RawDirent).RecordTime[1] = 1;
                (*RawDirent).RecordTime[2] = 1;

                let track_number = (*ThisTrack).TrackNumber;
                TrackOnes = (track_number % 10) + b'0';
                TrackTens = track_number / 10;
                TrackTens = (TrackTens % 10) + b'0';

                (*RawDirent).FileId[AUDIO_NAME_TENS_OFFSET as usize] = TrackTens;
                (*RawDirent).FileId[AUDIO_NAME_ONES_OFFSET as usize] = TrackOnes;

                SystemUse = (RawDirent as *mut u8).add(CdAudioSystemUseOffset as usize) as *mut SYSTEM_USE_XA;
                (*SystemUse).Attributes = SYSTEM_USE_XA_DA as u16;
                (*SystemUse).Signature = SYSTEM_XA_SIGNATURE as u16;
                (*SystemUse).FileNumber = CurrentTrack as u8;

                EntryCount += 1;
                SectorOffset += CdAudioDirentSize;
                CurrentTrack += 1;

                if EntryCount >= CdAudioDirentsPerSector {
                    break;
                }
            }

            RtlZeroMemory(
                CurrentSector.add(SectorOffset as usize) as *mut c_void,
                (SECTOR_SIZE - SectorOffset) as usize,
            );

            EntryCount = 0;
            BytesToCopy += SECTOR_SIZE;
            SectorOffset = 0;
            CurrentSector = CurrentSector.add(SECTOR_SIZE as usize);

            if BytesToCopy > ByteCount {
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdHijackIrpAndFlushDevice(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
    TargetDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER( IrpContext );
    let _ = IrpContext;
    // PAGED_CODE();

    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let NextIrpSp: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);

    core::ptr::copy_nonoverlapping(
        IoGetCurrentIrpStackLocation(Irp),
        NextIrpSp,
        1,
    );
    (*NextIrpSp).MajorFunction = IRP_MJ_FLUSH_BUFFERS;
    (*NextIrpSp).MinorFunction = 0u8;

    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0);
    IoSetCompletionRoutine(
        Irp,
        core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSyncCompletionRoutine as usize),
        &mut Event as *mut KEVENT as *mut core::ffi::c_void,
        1,
        1,
        1,
    );

    Status = IoCallDriver(TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut Event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if Status == STATUS_INVALID_DEVICE_REQUEST {
        Status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = 0;
    (*Irp).IoStatus.Information = 0;

    Status
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
//     DevIoSup.c
//
// Abstract:
//
//     This module implements the low lever disk read/write support for Cdfs.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_DEVIOSUP)
//
// //
// //  Local structure definitions
// //
//
// //
// //  An array of these structures is passed to CdMultipleAsync describing
// //  a set of runs to execute in parallel.
// //
//
// typedef struct _IO_RUN {
//
//     //
//     //  Disk offset to read from and number of bytes to read.  These
//     //  must be a multiple of 2048 and the disk offset is also a
//     //  multiple of 2048.
//     //
//
//     LONGLONG DiskOffset;
//     ULONG DiskByteCount;
//
//     //
//     //  Current position in user buffer.  This is the final destination for
//     //  this portion of the Io transfer.
//     //
//
//     PVOID UserBuffer;
//
//     //
//     //  Buffer to perform the transfer to.  If this is the same as the
//     //  user buffer above then we are using the user's buffer.  Otherwise
//     //  we either allocated a temporary buffer or are using a different portion
//     //  of the user's buffer.
//     //
//     //  TransferBuffer - Read full sectors into this location.  This can
//     //      be a pointer into the user's buffer at the exact location the
//     //      data should go.  It can also be an earlier point in the user's
//     //      buffer if the complete I/O doesn't start on a sector boundary.
//     //      It may also be a pointer into an allocated buffer.
//     //
//     //  TransferByteCount - Count of bytes to transfer to user's buffer.  A
//     //      value of zero indicates that we did do the transfer into the
//     //      user's buffer directly.
//     //
//     //  TransferBufferOffset - Offset in this buffer to begin the transfer
//     //      to the user's buffer.
//     //
//
//     PVOID TransferBuffer;
//     ULONG TransferByteCount;
//     ULONG TransferBufferOffset;
//
//     //
//     //  This is the Mdl describing the locked pages in memory.  It may
//     //  be allocated to describe the allocated buffer.  Or it may be
//     //  the Mdl in the originating Irp.  The MdlOffset is the offset of
//     //  the current buffer from the beginning of the buffer described by
//     //  the Mdl below.  If the TransferMdl is not the same as the Mdl
//     //  in the user's Irp then we know we have allocated it.
//     //
//
//     PMDL TransferMdl;
//     PVOID TransferVirtualAddress;
//
//     //
//     //  Associated Irp used to perform the Io.
//     //
//
//     PIRP SavedIrp;
//
// } IO_RUN;
// typedef IO_RUN *PIO_RUN;
//
// #define MAX_PARALLEL_IOS            5
//
// //
// //  Local support routines
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdPrepareBuffers (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PFCB Fcb,
//      PVOID UserBuffer,
//      ULONG UserBufferOffset,
//      LONGLONG StartingOffset,
//      ULONG ByteCount,
//      PIO_RUN IoRuns,
//      PULONG RunCount,
//      PULONG ThisByteCount
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdPrepareXABuffers (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PFCB Fcb,
//      PVOID UserBuffer,
//      ULONG UserBufferOffset,
//      LONGLONG StartingOffset,
//      ULONG ByteCount,
//      PIO_RUN IoRuns,
//      PULONG RunCount,
//      PULONG ThisByteCount
//     );
//
// BOOLEAN
// CdFinishBuffers (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN IoRuns,
//      ULONG RunCount,
//      BOOLEAN FinalCleanup,
//      BOOLEAN SaveXABuffer
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdMultipleAsync (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG RunCount,
//      PIO_RUN IoRuns
//     );
//
// VOID
// CdMultipleXAAsync (
//      PIRP_CONTEXT IrpContext,
//      ULONG RunCount,
//      PIO_RUN IoRuns,
//      PRAW_READ_INFO RawReads,
//      TRACK_MODE_TYPE TrackMode
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdSingleAsync (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN Run,
//      PFCB Fcb
//     );
//
// VOID
// CdWaitSync (
//      PIRP_CONTEXT IrpContext
//     );
//
// //  Tell prefast this is a completion routine.
// IO_COMPLETION_ROUTINE CdMultiSyncCompletionRoutine;
//
// //  Tell prefast this is a completion routine
// IO_COMPLETION_ROUTINE CdMultiAsyncCompletionRoutine;
//
// //  Tell prefast this is a completion routine
// IO_COMPLETION_ROUTINE CdSingleSyncCompletionRoutine;
//
// //  Tell prefast this is a completion routine
// IO_COMPLETION_ROUTINE CdSingleAsyncCompletionRoutine;
//
// (SafeNodeType(Fcb) != CDFS_NTC_FCB_PATH_TABLE && StartingOffset == 0, (ByteCount, (>=, CdAudioDirentSize + sizeof(RAW_DIRENT))))
// (SafeNodeType(Fcb) != CDFS_NTC_FCB_PATH_TABLE && StartingOffset != 0, (ByteCount, ))
// VOID
// CdReadAudioSystemFile (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      LONGLONG StartingOffset,
//       ULONG ByteCount,
//      PVOID SystemBuffer
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdReadDirDataThroughCache (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN Run
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCreateUserMdl)
// #pragma alloc_text(PAGE, CdMultipleAsync)
// #pragma alloc_text(PAGE, CdMultipleXAAsync)
// #pragma alloc_text(PAGE, CdNonCachedRead)
// #pragma alloc_text(PAGE, CdNonCachedXARead)
// #pragma alloc_text(PAGE, CdVolumeDasdWrite)
// #pragma alloc_text(PAGE, CdFinishBuffers)
// #pragma alloc_text(PAGE, CdPerformDevIoCtrl)
// #pragma alloc_text(PAGE, CdPerformDevIoCtrlEx)
// #pragma alloc_text(PAGE, CdPrepareBuffers)
// #pragma alloc_text(PAGE, CdPrepareXABuffers)
// #pragma alloc_text(PAGE, CdReadAudioSystemFile)
// #pragma alloc_text(PAGE, CdReadSectors)
// #pragma alloc_text(PAGE, CdSingleAsync)
// #pragma alloc_text(PAGE, CdWaitSync)
// #pragma alloc_text(PAGE, CdReadDirDataThroughCache)
// #pragma alloc_text(PAGE, CdFreeDirCache)
// #pragma alloc_text(PAGE, CdLbnToMmSsFf)
// #pragma alloc_text(PAGE, CdHijackIrpAndFlushDevice)
// #endif
//
//
// VOID
// CdLbnToMmSsFf (
//      ULONG Blocks,
//      PUCHAR Msf
//     )
//
// /*++
//
// Routine Description:
//
//     Convert Lbn to MSF format.
//
// Arguments:
//
//     Msf - on output, set to 0xMmSsFf representation of blocks.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     Blocks += 150;                  // Lbn 0 == 00:02:00, 1sec == 75 frames.
//
//     Msf[0] = (UCHAR)(Blocks % 75);  // Frames
//     Blocks /= 75;                   // -> Seconds
//     Msf[1] = (UCHAR)(Blocks % 60);  // Seconds
//     Blocks /= 60;                   // -> Minutes
//     Msf[2] = (UCHAR)Blocks;         // Minutes
// }
//
//
// __inline
// TRACK_MODE_TYPE
// CdFileTrackMode (
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts FCB XA file type flags to the track mode
//     used by the device drivers.
//
// Arguments:
//
//     Fcb - Fcb representing the file to read.
//
// Return Value:
//
//     TrackMode of the file represented by the Fcb.
//
// --*/
// {
//     NT_ASSERT( FlagOn( Fcb->FcbState, FCB_STATE_MODE2FORM2_FILE |
//                                    FCB_STATE_MODE2_FILE |
//                                    FCB_STATE_DA_FILE ));
//
//     if (FlagOn( Fcb->FcbState, FCB_STATE_MODE2FORM2_FILE )) {
//
//         return XAForm2;
//
//     } else if (FlagOn( Fcb->FcbState, FCB_STATE_DA_FILE )) {
//
//         return CDDA;
//
//     }
//
//     //
//     //  FCB_STATE_MODE2_FILE
//     //
//
//     return YellowMode2;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdNonCachedRead (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      LONGLONG StartingOffset,
//      ULONG ByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached reads to 'cooked' sectors (2048 bytes
//     per sector).  This is done by performing the following in a loop.
//
//         Fill in the IoRuns array for the next block of Io.
//         Send the Io to the device.
//         Perform any cleanup on the Io runs array.
//
//     We will not do async Io to any request that generates non-aligned Io.
//     Also we will not perform async Io if it will exceed the size of our
//     IoRuns array.  These should be the unusual cases but we will raise
//     or return CANT_WAIT in this routine if we detect this case.
//
// Arguments:
//
//     Fcb - Fcb representing the file to read.
//
//     StartingOffset - Logical offset in the file to read from.
//
//     ByteCount - Number of bytes to read.
//
// Return Value:
//
//     NTSTATUS - Status indicating the result of the operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     IO_RUN IoRuns[MAX_PARALLEL_IOS];
//     ULONG RunCount = 0;
//     ULONG CleanupRunCount = 0;
//
//     PVOID UserBuffer;
//     ULONG UserBufferOffset = 0;
//     LONGLONG CurrentOffset = StartingOffset;
//     ULONG RemainingByteCount = ByteCount;
//     ULONG ThisByteCount;
//
//     BOOLEAN Unaligned;
//     BOOLEAN FlushIoBuffers = FALSE;
//     BOOLEAN FirstPass = TRUE;
//
//     PAGED_CODE();
//
//     //
//     //  We want to make sure the user's buffer is locked in all cases.
//     //
//
//     if (IrpContext->Irp->MdlAddress == NULL) {
//
//         CdCreateUserMdl( IrpContext, ByteCount, TRUE, IoWriteAccess );
//     }
//
//     CdMapUserBuffer( IrpContext, &UserBuffer);
//
//     //
//     //  Special case the root directory and path table for a music volume.
//     //
//
//     if (FlagOn( Fcb->Vcb->VcbState, VCB_STATE_AUDIO_DISK ) &&
//         ((SafeNodeType( Fcb ) == CDFS_NTC_FCB_INDEX) ||
//          (SafeNodeType( Fcb ) == CDFS_NTC_FCB_PATH_TABLE))) {
//
//         CdReadAudioSystemFile( IrpContext,
//                                Fcb,
//                                StartingOffset,
//                                ByteCount,
//                                UserBuffer );
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  If we're going to use the sector cache for this request, then
//     //  mark the request waitable.
//     //
//
//     if ((SafeNodeType( Fcb) == CDFS_NTC_FCB_INDEX) &&
//         (NULL != Fcb->Vcb->SectorCacheBuffer) &&
//         (VcbMounted == IrpContext->Vcb->VcbCondition)) {
//
//         if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//             KeInitializeEvent( &IrpContext->IoContext->SyncEvent,
//                                NotificationEvent,
//                                FALSE );
//
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//         }
//     }
//
//     //
//     //  Use a try-finally to perform the final cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Loop while there are more bytes to transfer.
//         //
//
//         do {
//
//             //
//             //  Call prepare buffers to set up the next entries
//             //  in the IoRuns array.  Remember if there are any
//             //  unaligned entries.  This routine will raise CANT_WAIT
//             //  if there are unaligned entries for an async request.
//             //
//
//             RtlZeroMemory( IoRuns, sizeof( IoRuns ));
//
//             Unaligned = CdPrepareBuffers( IrpContext,
//                                           IrpContext->Irp,
//                                           Fcb,
//                                           UserBuffer,
//                                           UserBufferOffset,
//                                           CurrentOffset,
//                                           RemainingByteCount,
//                                           IoRuns,
//                                           &CleanupRunCount,
//                                           &ThisByteCount );
//
//
//             RunCount = CleanupRunCount;
//
//             //
//             //  If this is an async request and there aren't enough entries
//             //  in the Io array then post the request.
//             //
//
//             if ((ThisByteCount < RemainingByteCount) &&
//                 !FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             //
//             //  If the entire Io is contained in a single run then
//             //  we can pass the Io down to the driver.  Send the driver down
//             //  and wait on the result if this is synchronous.
//             //
//
//             if ((RunCount == 1) && !Unaligned && FirstPass) {
//
//                 CdSingleAsync( IrpContext,&IoRuns[0], Fcb );
//
//                 //
//                 //  No cleanup needed for the IoRuns array here.
//                 //
//
//                 CleanupRunCount = 0;
//
//                 //
//                 //  Wait if we are synchronous, otherwise return
//                 //
//
//                 if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//                     CdWaitSync( IrpContext );
//
//                     Status = IrpContext->Irp->IoStatus.Status;
//
//                 //
//                 //  Our completion routine will free the Io context but
//                 //  we do want to return STATUS_PENDING.
//                 //
//
//                 } else {
//
//                     ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//                     Status = STATUS_PENDING;
//                 }
//
//                 try_return( NOTHING );
//             }
//
//             //
//             //  Otherwise we will perform multiple Io to read in the data.
//             //
//
//             CdMultipleAsync( IrpContext, Fcb, RunCount, IoRuns );
//
//             //
//             //  If this is a synchronous request then perform any necessary
//             //  post-processing.
//             //
//
//             if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//                 //
//                 //  Wait for the request to complete.
//                 //
//
//                 CdWaitSync( IrpContext );
//
//                 Status = IrpContext->Irp->IoStatus.Status;
//
//                 //
//                 //  Exit this loop if there is an error.
//                 //
//
//                 if (!NT_SUCCESS( Status )) {
//
//                     try_return( NOTHING );
//                 }
//
//                 //
//                 //  Perform post read operations on the IoRuns if
//                 //  necessary.
//                 //
//
//                 if (Unaligned &&
//                     CdFinishBuffers( IrpContext, IoRuns, RunCount, FALSE, FALSE )) {
//
//                     FlushIoBuffers = TRUE;
//                 }
//
//                 CleanupRunCount = 0;
//
//                 //
//                 //  Exit this loop if there are no more bytes to transfer
//                 //  or we have any error.
//                 //
//
//                 RemainingByteCount -= ThisByteCount;
//                 CurrentOffset += ThisByteCount;
//                 UserBuffer = Add2Ptr( UserBuffer, ThisByteCount, PVOID );
//                 UserBufferOffset += ThisByteCount;
//
//             //
//             //  Otherwise this is an asynchronous request.  Always return
//             //  STATUS_PENDING.
//             //
//
//             } else {
//
//                 ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//                 CleanupRunCount = 0;
//                 try_return( Status = STATUS_PENDING );
//                 break;
//             }
//
//             FirstPass = FALSE;
//         } while (RemainingByteCount != 0);
//
//         //
//         //  Flush the hardware cache if we performed any copy operations.
//         //
//
//         if (FlushIoBuffers) {
//
//             KeFlushIoBuffers( IrpContext->Irp->MdlAddress, TRUE, FALSE );
//         }
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         //
//         //  Perform final cleanup on the IoRuns if necessary.
//         //
//
//         if (CleanupRunCount != 0) {
//
//             CdFinishBuffers( IrpContext, IoRuns, CleanupRunCount, TRUE, FALSE );
//         }
//     } _SEH2_END;
//
//     return Status;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdNonCachedXARead (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      LONGLONG StartingOffset,
//      ULONG ByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached reads for 'raw' sectors (2352 bytes
//     per sector).  We also prepend a hard-coded RIFF header of 44 bytes to the file.
//     All of this is already reflected in the file size.
//
//     We start by checking whether to prepend any portion of the RIFF header.  Then we check
//     if the last raw sector read was from the beginning portion of this file, deallocating
//     that buffer if necessary.  Finally we do the following in a loop.
//
//         Fill the IoRuns array for the next block of Io.
//         Send the Io to the device driver.
//         Perform any cleanup necessary on the IoRuns array.
//
//     We will not do any async request in this path.  The request would have been
//     posted to a worker thread before getting to this point.
//
// Arguments:
//
//     Fcb - Fcb representing the file to read.
//
//     StartingOffset - Logical offset in the file to read from.
//
//     ByteCount - Number of bytes to read.
//
// Return Value:
//
//     NTSTATUS - Status indicating the result of the operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     RIFF_HEADER LocalRiffHeader;
//     PRIFF_HEADER RiffHeader;
//
//     RAW_READ_INFO RawReads[MAX_PARALLEL_IOS];
//     IO_RUN IoRuns[MAX_PARALLEL_IOS];
//     ULONG RunCount = 0;
//     ULONG CleanupRunCount = 0;
//
//     PVOID UserBuffer;
//     ULONG UserBufferOffset = 0;
//     LONGLONG CurrentOffset = StartingOffset;
//     ULONG RemainingByteCount = ByteCount;
//     ULONG ThisByteCount = 0;
//     ULONG Address = 0;
//
//     BOOLEAN TryingYellowbookMode2 = FALSE;
//
//     TRACK_MODE_TYPE TrackMode;
//
//     PAGED_CODE();
//
//     //
//     //  We want to make sure the user's buffer is locked in all cases.
//     //
//
//     if (IrpContext->Irp->MdlAddress == NULL) {
//
//         CdCreateUserMdl( IrpContext, ByteCount, TRUE, IoWriteAccess );
//     }
//
//     //
//     //  The byte count was rounded up to a logical sector boundary.  It has
//     //  nothing to do with the raw sectors on disk.  Limit the remaining
//     //  byte count to file size.
//     //
//
//     if (CurrentOffset + RemainingByteCount > Fcb->FileSize.QuadPart) {
//
//         RemainingByteCount = (ULONG) (Fcb->FileSize.QuadPart - CurrentOffset);
//     }
//
//     CdMapUserBuffer( IrpContext, &UserBuffer);
//
//     //
//     //  Use a try-finally to perform the final cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  If the initial offset lies within the RIFF header then copy the
//         //  necessary bytes to the user's buffer.
//         //
//
//         if (CurrentOffset < sizeof( RIFF_HEADER )) {
//
//             //
//             //  Copy the appropriate RIFF header.
//             //
//
//             if (FlagOn( Fcb->FcbState, FCB_STATE_DA_FILE )) {
//
//                 //
//                 //  Create the pseudo entries for a music disk.
//                 //
//
//                 if (FlagOn( Fcb->Vcb->VcbState, VCB_STATE_AUDIO_DISK )) {
//
//                     PAUDIO_PLAY_HEADER AudioPlayHeader;
//                     PTRACK_DATA TrackData;
//
//                     AudioPlayHeader = (PAUDIO_PLAY_HEADER) &LocalRiffHeader;
//                     TrackData = &Fcb->Vcb->CdromToc->TrackData[Fcb->XAFileNumber];
//
//                     //
//                     //  Copy the data header into our local buffer.
//                     //
//
//                     RtlCopyMemory( AudioPlayHeader,
//                                    CdAudioPlayHeader,
//                                    sizeof( AUDIO_PLAY_HEADER ));
//
//                     //
//                     //  Copy the serial number into the Id field.  Also
//                     //  the track number in the TOC.
//                     //
//
//                     AudioPlayHeader->DiskID = Fcb->Vcb->Vpb->SerialNumber;
//                     AudioPlayHeader->TrackNumber = TrackData->TrackNumber;
//
//                     //
//                     //  One frame == One sector.
//                     //  One second == 75 frames (winds up being a 44.1khz sample)
//                     //
//                     //  Note: LBN 0 == 0:2:0 (MSF)
//                     //
//
//                     //
//                     //  Fill in the address (both MSF and Lbn format) and length fields.
//                     //
//
//                     SwapCopyUchar4( &Address, TrackData->Address);
//                     CdLbnToMmSsFf( Address, AudioPlayHeader->TrackAddress);
//
//                     SwapCopyUchar4( &AudioPlayHeader->StartingSector, TrackData->Address);
//
//                     //
//                     //  Go to the next track and find the starting point.
//                     //
//
//                     TrackData = &Fcb->Vcb->CdromToc->TrackData[Fcb->XAFileNumber + 1];
//
//                     SwapCopyUchar4( &AudioPlayHeader->SectorCount, TrackData->Address);
//
//                     //
//                     //  Now compute the difference.  If there is an error then use
//                     //  a length of zero.
//                     //
//
//                     if (AudioPlayHeader->SectorCount < AudioPlayHeader->StartingSector) {
//
//                         AudioPlayHeader->SectorCount = 0;
//
//                     } else {
//
//                         AudioPlayHeader->SectorCount -= AudioPlayHeader->StartingSector;
//                     }
//
//                     //
//                     //  Use the sector count to determine the MSF length. Bias by 150 to make
//                     //  it an "lbn" since the conversion routine corrects for Lbn 0 == 0:2:0;
//                     //
//
//                     Address = AudioPlayHeader->SectorCount - 150;
//                     CdLbnToMmSsFf( Address, AudioPlayHeader->TrackLength);
//
//                     ThisByteCount = sizeof( RIFF_HEADER ) - (ULONG) CurrentOffset;
//
//                     RtlCopyMemory( UserBuffer,
//                                    Add2Ptr( AudioPlayHeader,
//                                             sizeof( RIFF_HEADER ) - ThisByteCount,
//                                             PCHAR ),
//                                    ThisByteCount );
//
//                 //
//                 //  CD-XA CDDA
//                 //
//
//                 } else {
//
//                     //
//                     //  The WAVE header format is actually much closer to an audio play
//                     //  header in format but we only need to modify the filesize fields.
//                     //
//
//                     RiffHeader = &LocalRiffHeader;
//
//                     //
//                     //  Copy the data header into our local buffer and add the file size to it.
//                     //
//
//                     RtlCopyMemory( RiffHeader,
//                                    CdXAAudioPhileHeader,
//                                    sizeof( RIFF_HEADER ));
//
//                     RiffHeader->ChunkSize += Fcb->FileSize.LowPart;
//                     RiffHeader->RawSectors += Fcb->FileSize.LowPart;
//
//                     ThisByteCount = sizeof( RIFF_HEADER ) - (ULONG) CurrentOffset;
//                     RtlCopyMemory( UserBuffer,
//                                    Add2Ptr( RiffHeader,
//                                             sizeof( RIFF_HEADER ) - ThisByteCount,
//                                             PCHAR ),
//                                    ThisByteCount );
//                 }
//
//             //
//             //  CD-XA non-audio
//             //
//
//             } else {
//
//                 NT_ASSERT( FlagOn( Fcb->FcbState, FCB_STATE_MODE2_FILE | FCB_STATE_MODE2FORM2_FILE ));
//
//                 RiffHeader = &LocalRiffHeader;
//
//                 //
//                 //  Copy the data header into our local buffer and add the file size to it.
//                 //
//
//                 RtlCopyMemory( RiffHeader,
//                                CdXAFileHeader,
//                                sizeof( RIFF_HEADER ));
//
//                 RiffHeader->ChunkSize += Fcb->FileSize.LowPart;
//                 RiffHeader->RawSectors += Fcb->FileSize.LowPart;
//
//                 RiffHeader->Attributes = (USHORT) Fcb->XAAttributes;
//                 RiffHeader->FileNumber = (UCHAR) Fcb->XAFileNumber;
//
//                 ThisByteCount = sizeof( RIFF_HEADER ) - (ULONG) CurrentOffset;
//                 RtlCopyMemory( UserBuffer,
//                                Add2Ptr( RiffHeader,
//                                         sizeof( RIFF_HEADER ) - ThisByteCount,
//                                         PCHAR ),
//                                ThisByteCount );
//             }
//
//             //
//             //  Adjust the starting offset and byte count to reflect that
//             //  we copied over the RIFF bytes.
//             //
//
//             UserBuffer = Add2Ptr( UserBuffer, ThisByteCount, PVOID );
//             UserBufferOffset += ThisByteCount;
//             CurrentOffset += ThisByteCount;
//             RemainingByteCount -= ThisByteCount;
//         }
//
//         //
//         //  Set up the appropriate trackmode
//         //
//
//         TrackMode = CdFileTrackMode(Fcb);
//
//         //
//         //  Loop while there are more bytes to transfer.
//         //
//
//         while (RemainingByteCount != 0) {
//
//             //
//             //  Call prepare buffers to set up the next entries
//             //  in the IoRuns array.  Remember if there are any
//             //  unaligned entries.  If we're just retrying the previous
//             //  runs with a different track mode,  then don't do anything here.
//             //
//
//             if (!TryingYellowbookMode2)  {
//
//                 RtlZeroMemory( IoRuns, sizeof( IoRuns ));
//                 RtlZeroMemory( RawReads, sizeof( RawReads ));
//
//                 CdPrepareXABuffers( IrpContext,
//                                     IrpContext->Irp,
//                                     Fcb,
//                                     UserBuffer,
//                                     UserBufferOffset,
//                                     CurrentOffset,
//                                     RemainingByteCount,
//                                     IoRuns,
//                                     &CleanupRunCount,
//                                     &ThisByteCount );
//             }
//
//             //
//             //  Perform multiple Io to read in the data.  Note that
//             //  there may be no Io to do if we were able to use an
//             //  existing buffer from the Vcb.
//             //
//
//             if (CleanupRunCount != 0) {
//
//                 RunCount = CleanupRunCount;
//
//                 CdMultipleXAAsync( IrpContext,
//                                    RunCount,
//                                    IoRuns,
//                                    RawReads,
//                                    TrackMode );
//                 //
//                 //  Wait for the request to complete.
//                 //
//
//                 CdWaitSync( IrpContext );
//
//                 Status = IrpContext->Irp->IoStatus.Status;
//
//                 //
//                 //  Exit this loop if there is an error.
//                 //
//
//                 if (!NT_SUCCESS( Status )) {
//
//                     if (!TryingYellowbookMode2 &&
//                         FlagOn( Fcb->FcbState, FCB_STATE_MODE2FORM2_FILE )) {
//
//                         //
//                         //  There are wacky cases where someone has mastered as CD-XA
//                         //  but the sectors they claim are Mode2Form2 are really, according
//                         //  to ATAPI devices, Yellowbook Mode2. We will try once more
//                         //  with these. Kodak PHOTO-CD has been observed to do this.
//                         //
//
//                         TryingYellowbookMode2 = TRUE;
//                         TrackMode = YellowMode2;
//
//                         //
//                         //  Clear our 'cumulative' error status value
//                         //
//
//                         IrpContext->IoContext->Status = STATUS_SUCCESS;
//
//                         continue;
//                     }
//
//                     try_return( NOTHING );
//                 }
//
//                 CleanupRunCount = 0;
//
//                 if (TryingYellowbookMode2) {
//
//                     //
//                     //  We succesfully got data when we tried switching the trackmode,
//                     //  so change the state of the FCB to remember that.
//                     //
//
//                     SetFlag( Fcb->FcbState, FCB_STATE_MODE2_FILE );
//                     ClearFlag( Fcb->FcbState, FCB_STATE_MODE2FORM2_FILE );
//
//                     TryingYellowbookMode2 = FALSE;
//                 }
//
//                 //
//                 //  Perform post read operations on the IoRuns if
//                 //  necessary.
//                 //
//
//                 CdFinishBuffers( IrpContext, IoRuns, RunCount, FALSE, TRUE );
//             }
//
//             //
//             //  Adjust our loop variants.
//             //
//
//             RemainingByteCount -= ThisByteCount;
//             CurrentOffset += ThisByteCount;
//             UserBuffer = Add2Ptr( UserBuffer, ThisByteCount, PVOID );
//             UserBufferOffset += ThisByteCount;
//         }
//
//         //
//         //  Always flush the hardware cache.
//         //
//
//         KeFlushIoBuffers( IrpContext->Irp->MdlAddress, TRUE, FALSE );
//
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         //
//         //  Perform final cleanup on the IoRuns if necessary.
//         //
//
//         if (CleanupRunCount != 0) {
//
//             CdFinishBuffers( IrpContext, IoRuns, CleanupRunCount, TRUE, FALSE );
//         }
//     } _SEH2_END;
//
//     return Status;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdVolumeDasdWrite (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      LONGLONG StartingOffset,
//      ULONG ByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the non-cached writes to 'cooked' sectors (2048 bytes
//     per sector).  This is done by filling the IoRun for the desired request
//     and send it down to the device.
//
// Arguments:
//
//     Fcb - Fcb representing the file to read.
//
//     StartingOffset - Logical offset in the file to read from.
//
//     ByteCount - Number of bytes to read.
//
// Return Value:
//
//     NTSTATUS - Status indicating the result of the operation.
//
// --*/
//
// {
//     NTSTATUS Status;
//     IO_RUN IoRun;
//
//     PAGED_CODE();
//
//     //
//     //  We want to make sure the user's buffer is locked in all cases.
//     //
//
//     CdLockUserBuffer( IrpContext, ByteCount, IoReadAccess );
//
//     //
//     //  The entire Io can be contained in a single run, just pass
//     //  the Io down to the driver.  Send the driver down
//     //  and wait on the result if this is synchronous.
//     //
//
//     RtlZeroMemory( &IoRun, sizeof( IoRun ) );
//
//     IoRun.DiskOffset = StartingOffset;
//     IoRun.DiskByteCount = ByteCount;
//
//     CdSingleAsync( IrpContext, &IoRun, Fcb );
//
//     //
//     //  Wait if we are synchronous, otherwise return
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//         CdWaitSync( IrpContext );
//
//         Status = IrpContext->Irp->IoStatus.Status;
//
//     //
//     //  Our completion routine will free the Io context but
//     //  we do want to return STATUS_PENDING.
//     //
//
//     } else {
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO );
//         Status = STATUS_PENDING;
//     }
//
//     return Status;
// }
//
//
//
// BOOLEAN
// CdReadSectors (
//      PIRP_CONTEXT IrpContext,
//      LONGLONG StartingOffset,
//      ULONG ByteCount,
//      BOOLEAN ReturnError,
//      PVOID Buffer,
//      PDEVICE_OBJECT TargetDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to transfer sectors from the disk to a
//     specified buffer.  It is used for mount and volume verify operations.
//
//     This routine is synchronous, it will not return until the operation
//     is complete or until the operation fails.
//
//     The routine allocates an IRP and then passes this IRP to a lower
//     level driver.  Errors may occur in the allocation of this IRP or
//     in the operation of the lower driver.
//
// Arguments:
//
//     StartingOffset - Logical offset on the disk to start the read.  This
//         must be on a sector boundary, no check is made here.
//
//     ByteCount - Number of bytes to read.  This is an integral number of
//         2K sectors, no check is made here to confirm this.
//
//     ReturnError - Indicates whether we should return TRUE or FALSE
//         to indicate an error or raise an error condition.  This only applies
//         to the result of the IO.  Any other error may cause a raise.
//
//     Buffer - Buffer to transfer the disk data into.
//
//     TargetDeviceObject - The device object for the volume to be read.
//
// Return Value:
//
//     BOOLEAN - Depending on 'RaiseOnError' flag above.  TRUE if operation
//               succeeded, FALSE otherwise.
//
// --*/
//
// {
//     NTSTATUS Status;
//     KEVENT  Event;
//     PIRP Irp;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the event.
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  Attempt to allocate the IRP.  If unsuccessful, raise
//     //  STATUS_INSUFFICIENT_RESOURCES.
//     //
//
//     Irp = IoBuildSynchronousFsdRequest( IRP_MJ_READ,
//                                         TargetDeviceObject,
//                                         Buffer,
//                                         ByteCount,
//                                         (PLARGE_INTEGER) &StartingOffset,
//                                         &Event,
//                                         &IrpContext->Irp->IoStatus );
//
//     if (Irp == NULL) {
//
//         CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//     }
//
//     //
//     //  Ignore the change line (verify) for mount and verify requests
//     //
//
//     SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//
//     //
//     //  Send the request down to the driver.  If an error occurs return
//     //  it to the caller.
//     //
//
//     Status = IoCallDriver( TargetDeviceObject, Irp );
//
//     //
//     //  If the status was STATUS_PENDING then wait on the event.
//     //
//
//     if (Status == STATUS_PENDING) {
//
//         Status = KeWaitForSingleObject( &Event,
//                                         Executive,
//                                         KernelMode,
//                                         FALSE,
//                                         NULL );
//
//         //
//         //  On a successful wait pull the status out of the IoStatus block.
//         //
//
//         if (NT_SUCCESS( Status )) {
//
//             Status = IrpContext->Irp->IoStatus.Status;
//         }
//     }
//
//     //
//     //  Check whether we should raise in the error case.
//     //
//
//     if (!NT_SUCCESS( Status )) {
//
//         if (!ReturnError) {
//
//             CdNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//
//         //
//         //  We don't raise, but return FALSE to indicate an error.
//         //
//
//         return FALSE;
//
//     //
//     //  The operation completed successfully.
//     //
//
//     } else {
//
//         return TRUE;
//     }
// }
//
//
// NTSTATUS
// CdCreateUserMdl (
//      PIRP_CONTEXT IrpContext,
//      ULONG BufferLength,
//      BOOLEAN RaiseOnError,
//      LOCK_OPERATION Operation
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locks the specified buffer for read access (we only write into
//     the buffer).  The file system requires this routine since it does not
//     ask the I/O system to lock its buffers for direct I/O.  This routine
//     may only be called from the Fsd while still in the user context.
//
//     This routine is only called if there is not already an Mdl.
//
// Arguments:
//
//     BufferLength - Length of user buffer.
//
//     RaiseOnError - Indicates if our caller wants this routine to raise on
//         an error condition.
//
//     Operation - IoWriteAccess or IoReadAccess
//
// Return Value:
//
//     NTSTATUS - Status from this routine.  Error status only returned if
//         RaiseOnError is FALSE.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_INSUFFICIENT_RESOURCES;
//     PMDL Mdl;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Operation );
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_IRP( IrpContext->Irp );
//     NT_ASSERT( IrpContext->Irp->MdlAddress == NULL );
//
//     //
//     // Allocate the Mdl, and Raise if we fail.
//     //
//
//     Mdl = IoAllocateMdl( IrpContext->Irp->UserBuffer,
//                          BufferLength,
//                          FALSE,
//                          FALSE,
//                          IrpContext->Irp );
//
//     if (Mdl != NULL) {
//
//         //
//         //  Now probe the buffer described by the Irp.  If we get an exception,
//         //  deallocate the Mdl and return the appropriate "expected" status.
//         //
//
//         _SEH2_TRY {
//
//             MmProbeAndLockPages( Mdl, IrpContext->Irp->RequestorMode, IoWriteAccess );
//
//             Status = STATUS_SUCCESS;
//
// #ifdef _MSC_VER
// #pragma warning(suppress: 6320)
// #endif
//         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//
//             Status = _SEH2_GetExceptionCode();
//
//             IoFreeMdl( Mdl );
//             IrpContext->Irp->MdlAddress = NULL;
//
//             if (!FsRtlIsNtstatusExpected( Status )) {
//
//                 Status = STATUS_INVALID_USER_BUFFER;
//             }
//         } _SEH2_END;
//     }
//
//     //
//     //  Check if we are to raise or return
//     //
//
//     if (Status != STATUS_SUCCESS) {
//
//         if (RaiseOnError) {
//
//             CdRaiseStatus( IrpContext, Status );
//         }
//     }
//
//     //
//     //  Return the status code.
//     //
//
//     return Status;
// }
//
//
// NTSTATUS
// CdPerformDevIoCtrlEx (
//      PIRP_CONTEXT IrpContext,
//      ULONG IoControlCode,
//      PDEVICE_OBJECT Device,
//      PVOID InputBuffer,
//      ULONG InputBufferLength,
//      PVOID OutputBuffer,
//      ULONG OutputBufferLength,
//      BOOLEAN InternalDeviceIoControl,
//      BOOLEAN OverrideVerify,
//      PIO_STATUS_BLOCK Iosb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to perform DevIoCtrl functions internally within
//     the filesystem.  We take the status from the driver and return it to our
//     caller.
//
// Arguments:
//
//     IoControlCode - Code to send to driver.
//
//     Device - This is the device to send the request to.
//
//     OutPutBuffer - Pointer to output buffer.
//
//     OutputBufferLength - Length of output buffer above.
//
//     InternalDeviceIoControl - Indicates if this is an internal or external
//         Io control code.
//
//     OverrideVerify - Indicates if we should tell the driver not to return
//         STATUS_VERIFY_REQUIRED for mount and verify.
//
//     Iosb - If specified, we return the results of the operation here.
//
// Return Value:
//
//     NTSTATUS - Status returned by next lower driver.
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIRP Irp;
//     KEVENT Event;
//     IO_STATUS_BLOCK LocalIosb;
//     PIO_STATUS_BLOCK IosbToUse = &LocalIosb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check if the user gave us an Iosb.
//     //
//
//     if (ARGUMENT_PRESENT( Iosb )) {
//
//         IosbToUse = Iosb;
//     }
//
//     IosbToUse->Status = 0;
//     IosbToUse->Information = 0;
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     Irp = IoBuildDeviceIoControlRequest( IoControlCode,
//                                          Device,
//                                          InputBuffer,
//                                          InputBufferLength,
//                                          OutputBuffer,
//                                          OutputBufferLength,
//                                          InternalDeviceIoControl,
//                                          &Event,
//                                          IosbToUse );
//
//     if (Irp == NULL) {
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (OverrideVerify) {
//
//         SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//     }
//
//     Status = IoCallDriver( Device, Irp );
//
//     //
//     //  We check for device not ready by first checking Status
//     //  and then if status pending was returned, the Iosb status
//     //  value.
//     //
//
//     if (Status == STATUS_PENDING) {
//
//         (VOID) KeWaitForSingleObject( &Event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       (PLARGE_INTEGER)NULL );
//
//         Status = IosbToUse->Status;
//     }
//
//     NT_ASSERT( !(OverrideVerify && (STATUS_VERIFY_REQUIRED == Status)));
//
//     return Status;
// }
//
//
// NTSTATUS
// FASTCALL
// CdPerformDevIoCtrl (
//      PIRP_CONTEXT IrpContext,
//      ULONG IoControlCode,
//      PDEVICE_OBJECT Device,
//      PVOID OutputBuffer,
//      ULONG OutputBufferLength,
//      BOOLEAN InternalDeviceIoControl,
//      BOOLEAN OverrideVerify,
//      PIO_STATUS_BLOCK Iosb
//     )
// {
//     PAGED_CODE();
//
//     return CdPerformDevIoCtrlEx( IrpContext,
//                                  IoControlCode,
//                                  Device,
//                                  NULL,
//                                  0,
//                                  OutputBuffer,
//                                  OutputBufferLength,
//                                  InternalDeviceIoControl,
//                                  OverrideVerify,
//                                  Iosb);
// }
//
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdPrepareBuffers (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PFCB Fcb,
//      PVOID UserBuffer,
//      ULONG UserBufferOffset,
//      LONGLONG StartingOffset,
//      ULONG ByteCount,
//      PIO_RUN IoRuns,
//      PULONG RunCount,
//      PULONG ThisByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the worker routine which looks up each run of an IO
//     request and stores an entry for it in the IoRuns array.  If the run
//     begins on an unaligned disk boundary then we will allocate a buffer
//     and Mdl for the unaligned portion and put it in the IoRuns entry.
//
//     This routine will raise CANT_WAIT if an unaligned transfer is encountered
//     and this request can't wait.
//
// Arguments:
//
//     Irp - Originating Irp for this request.
//
//     Fcb - This is the Fcb for this data stream.  It may be a file, directory,
//         path table or the volume file.
//
//     UserBuffer - Current position in the user's buffer.
//
//     UserBufferOffset - Offset from the start of the original user buffer.
//
//     StartingOffset - Offset in the stream to begin the read.
//
//     ByteCount - Number of bytes to read.  We will fill the IoRuns array up
//         to this point.  We will stop early if we exceed the maximum number
//         of parallel Ios we support.
//
//     IoRuns - Pointer to the IoRuns array.  The entire array is zeroes when
//         this routine is called.
//
//     RunCount - Number of entries in the IoRuns array filled here.
//
//     ThisByteCount - Number of bytes described by the IoRun entries.  Will
//         not exceed the ByteCount passed in.
//
// Return Value:
//
//     BOOLEAN - TRUE if one of the entries in an unaligned buffer (provided
//         this is synchronous).  FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN FoundUnaligned = FALSE;
//     PIO_RUN ThisIoRun = IoRuns;
//
//     //
//     //  Following indicate where we are in the current transfer.  Current
//     //  position in the file and number of bytes yet to transfer from
//     //  this position.
//     //
//
//     ULONG RemainingByteCount = ByteCount;
//     LONGLONG CurrentFileOffset = StartingOffset;
//
//     //
//     //  Following indicate the state of the user's buffer.  We have
//     //  the destination of the next transfer and its offset in the
//     //  buffer.  We also have the next available position in the buffer
//     //  available for a scratch buffer.  We will align this up to a sector
//     //  boundary.
//     //
//
//     PVOID CurrentUserBuffer = UserBuffer;
//     ULONG CurrentUserBufferOffset = UserBufferOffset;
//
//     //
//     //  The following is the next contiguous bytes on the disk to
//     //  transfer.  Read from the allocation package.
//     //
//
//     LONGLONG DiskOffset = 0;
//     ULONG CurrentByteCount = RemainingByteCount;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize the RunCount and ByteCount.
//     //
//
//     *RunCount = 0;
//     *ThisByteCount = 0;
//
//     //
//     //  Loop while there are more bytes to process or there are
//     //  available entries in the IoRun array.
//     //
//
//     while (TRUE) {
//
//         *RunCount += 1;
//
//         //
//         //  Initialize the current position in the IoRuns array.
//         //  Find the user's buffer for this portion of the transfer.
//         //
//
//         ThisIoRun->UserBuffer = CurrentUserBuffer;
//
//         //
//         //  Find the allocation information for the current offset in the
//         //  stream.
//         //
//
//         CdLookupAllocation( IrpContext,
//                             Fcb,
//                             CurrentFileOffset,
//                             &DiskOffset,
//                             &CurrentByteCount );
//
//         //
//         //  Limit ourselves to the data requested.
//         //
//
//         if (CurrentByteCount > RemainingByteCount) {
//
//             CurrentByteCount = RemainingByteCount;
//         }
//
//         //
//         //  Handle the case where this is an unaligned transfer.  The
//         //  following must all be true for this to be an aligned transfer.
//         //
//         //      Disk offset on a 2048 byte boundary (Start of transfer)
//         //
//         //      Byte count is a multiple of 2048 (Length of transfer)
//         //
//         //  If the ByteCount is at least one sector then do the
//         //  unaligned transfer only for the tail.  We can use the
//         //  user's buffer for the aligned portion.
//         //
//
//         if (FlagOn( (ULONG) DiskOffset, SECTOR_MASK ) ||
//             (FlagOn( (ULONG) CurrentByteCount, SECTOR_MASK ) &&
//              (CurrentByteCount < SECTOR_SIZE))) {
//
//             NT_ASSERT( SafeNodeType(Fcb) != CDFS_NTC_FCB_INDEX);
//
//             //
//             //  If we can't wait then raise.
//             //
//
//             if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             //
//             //  Remember the offset and the number of bytes out of
//             //  the transfer buffer to copy into the user's buffer.
//             //  We will truncate the current read to end on a sector
//             //  boundary.
//             //
//
//             ThisIoRun->TransferBufferOffset = SectorOffset( DiskOffset );
//
//             //
//             //  Make sure this transfer ends on a sector boundary.
//             //
//
//             ThisIoRun->DiskOffset = LlSectorTruncate( DiskOffset );
//
//             //
//             //  We need to allocate an auxilary buffer for the next sector.
//             //  Read up to a page containing the partial data.
//             //
//
//             ThisIoRun->DiskByteCount = SectorAlign( ThisIoRun->TransferBufferOffset + CurrentByteCount );
//
//             if (ThisIoRun->DiskByteCount > PAGE_SIZE) {
//
//                 ThisIoRun->DiskByteCount = PAGE_SIZE;
//             }
//
//             if (ThisIoRun->TransferBufferOffset + CurrentByteCount > ThisIoRun->DiskByteCount) {
//
//                 CurrentByteCount = ThisIoRun->DiskByteCount - ThisIoRun->TransferBufferOffset;
//             }
//
//             ThisIoRun->TransferByteCount = CurrentByteCount;
//
//             //
//             //  Allocate a buffer for the non-aligned transfer.
//             //
//
//             ThisIoRun->TransferBuffer = FsRtlAllocatePoolWithTag( CdNonPagedPool, PAGE_SIZE, TAG_IO_BUFFER );
//
//             //
//             //  Allocate and build the Mdl to describe this buffer.
//             //
//
//             ThisIoRun->TransferMdl = IoAllocateMdl( ThisIoRun->TransferBuffer,
//                                                     PAGE_SIZE as u32,
//                                                     FALSE,
//                                                     FALSE,
//                                                     NULL );
//
//             ThisIoRun->TransferVirtualAddress = ThisIoRun->TransferBuffer;
//
//             if (ThisIoRun->TransferMdl == NULL) {
//
//                 IrpContext->Irp->IoStatus.Information = 0;
//                 CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//             }
//
//             MmBuildMdlForNonPagedPool( ThisIoRun->TransferMdl );
//
//             //
//             //  Remember we found an unaligned transfer.
//             //
//
//             FoundUnaligned = TRUE;
//
//         //
//         //  Otherwise we use the buffer and Mdl from the original request.
//         //
//
//         } else {
//
//             //
//             //  Truncate the read length to a sector-aligned value.  We know
//             //  the length must be at least one sector or we wouldn't be
//             //  here now.
//             //
//
//             CurrentByteCount = SectorTruncate( CurrentByteCount );
//
//             //
//             //  Read these sectors from the disk.
//             //
//
//             ThisIoRun->DiskOffset = DiskOffset;
//             ThisIoRun->DiskByteCount = CurrentByteCount;
//
//             //
//             //  Use the user's buffer and Mdl as our transfer buffer
//             //  and Mdl.
//             //
//
//             ThisIoRun->TransferBuffer = CurrentUserBuffer;
//             ThisIoRun->TransferMdl = Irp->MdlAddress;
//             ThisIoRun->TransferVirtualAddress = Add2Ptr( Irp->UserBuffer,
//                                                          CurrentUserBufferOffset,
//                                                          PVOID );
//         }
//
//         //
//         //  Update our position in the transfer and the RunCount and
//         //  ByteCount for the user.
//         //
//
//         RemainingByteCount -= CurrentByteCount;
//
//         //
//         //  Break out if no more positions in the IoRuns array or
//         //  we have all of the bytes accounted for.
//         //
//
//         *ThisByteCount += CurrentByteCount;
//
//         if ((RemainingByteCount == 0) || (*RunCount == MAX_PARALLEL_IOS)) {
//
//             break;
//         }
//
//         //
//         //  Update our pointers for the user's buffer.
//         //
//
//         ThisIoRun += 1;
//         CurrentUserBuffer = Add2Ptr( CurrentUserBuffer, CurrentByteCount, PVOID );
//         CurrentUserBufferOffset += CurrentByteCount;
//         CurrentFileOffset += CurrentByteCount;
//     }
//
//     return FoundUnaligned;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdPrepareXABuffers (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PFCB Fcb,
//      PVOID UserBuffer,
//      ULONG UserBufferOffset,
//      LONGLONG StartingOffset,
//      ULONG ByteCount,
//      PIO_RUN IoRuns,
//      PULONG RunCount,
//      PULONG ThisByteCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the worker routine which looks up the individual runs
//     of an IO request and stores an entry for it in the IoRuns array.  The
//     worker routine is for XA files where we need to convert the raw offset
//     in the file to logical cooked sectors.  We store one raw sector in
//     the Vcb.  If the current read is to that sector then we can simply copy
//     whatever bytes are needed from that sector.
//
// Arguments:
//
//     Irp - Originating Irp for this request.
//
//     Fcb - This is the Fcb for this data stream.  It must be a data stream.
//
//     UserBuffer - Current position in the user's buffer.
//
//     UserBufferOffset - Offset of this buffer from the beginning of the user's
//         buffer for the original request.
//
//     StartingOffset - Offset in the stream to begin the read.
//
//     ByteCount - Number of bytes to read.  We will fill the IoRuns array up
//         to this point.  We will stop early if we exceed the maximum number
//         of parallel Ios we support.
//
//     IoRuns - Pointer to the IoRuns array.  The entire array is zeroes when
//         this routine is called.
//
//     RunCount - Number of entries in the IoRuns array filled here.
//
//     ThisByteCount - Number of bytes described by the IoRun entries.  Will
//         not exceed the ByteCount passed in.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PIO_RUN ThisIoRun = IoRuns;
//     BOOLEAN PerformedCopy;
//
//     //
//     //  The following deal with where we are in the range of raw sectors.
//     //  Note that we will bias the input file offset by the RIFF header
//     //  to deal directly with the raw sectors.
//     //
//
//     ULONG RawSectorOffset;
//     ULONG RemainingRawByteCount = ByteCount;
//     LONGLONG CurrentRawOffset = StartingOffset - sizeof( RIFF_HEADER );
//
//     //
//     //  The following is the offset into the cooked sectors for the file.
//     //
//
//     LONGLONG CurrentCookedOffset;
//     ULONG RemainingCookedByteCount;
//
//     //
//     //  Following indicate the state of the user's buffer.  We have
//     //  the destination of the next transfer and its offset in the
//     //  buffer.  We also have the next available position in the buffer
//     //  available for a scratch buffer.
//     //
//
//     PVOID CurrentUserBuffer = UserBuffer;
//     ULONG CurrentUserBufferOffset = UserBufferOffset;
//
//     //
//     //  The following is the next contiguous bytes on the disk to
//     //  transfer.  These are represented by cooked byte offset and length.
//     //  We also compute the number of raw bytes in the current transfer.
//     //
//
//     LONGLONG DiskOffset = 0;
//     ULONG CurrentCookedByteCount = 0;
//     ULONG CurrentRawByteCount;
//
//     PAGED_CODE();
//
//     //
//     //  We need to maintain our position as we walk through the sectors on the disk.
//     //  We keep separate values for the cooked offset as well as the raw offset.
//     //  These are initialized on sector boundaries and we move through these
//     //  the file sector-by-sector.
//     //
//     //  Try to do 32-bit math.
//     //
//
//     if (((PLARGE_INTEGER) &CurrentRawOffset)->HighPart == 0) {
//
//         //
//         //  Prefix/fast: Note that the following are safe since we only
//         //               take this path for 32bit offsets.
//         //
//
//         CurrentRawOffset = (LONGLONG) ((ULONG) CurrentRawOffset / RAW_SECTOR_SIZE);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: __WARNING_RESULTOFSHIFTCASTTOLARGERSIZE, "This is fine beacuse raw sector size > sector shift" )
// #endif
//         CurrentCookedOffset = (LONGLONG) ((ULONG) CurrentRawOffset << SECTOR_SHIFT );
//
//         CurrentRawOffset = (LONGLONG) ((ULONG) CurrentRawOffset * RAW_SECTOR_SIZE);
//
//     //
//     //  Otherwise we need to do 64-bit math (sigh).
//     //
//
//     } else {
//
//         CurrentRawOffset /= RAW_SECTOR_SIZE;
//
//         CurrentCookedOffset = CurrentRawOffset << SECTOR_SHIFT;
//
//         CurrentRawOffset *= RAW_SECTOR_SIZE;
//     }
//
//     //
//     //  Now compute the full number of sectors to be read.  Count all of the raw
//     //  sectors that need to be read and convert to cooked bytes.
//     //
//
//     RawSectorOffset = (ULONG) ( StartingOffset - CurrentRawOffset) - sizeof( RIFF_HEADER );
//     CurrentRawByteCount = (RawSectorOffset + RemainingRawByteCount + RAW_SECTOR_SIZE - 1) / RAW_SECTOR_SIZE;
//
//     RemainingCookedByteCount = CurrentRawByteCount << SECTOR_SHIFT;
//
//     //
//     //  Initialize the RunCount and ByteCount.
//     //
//
//     *RunCount = 0;
//     *ThisByteCount = 0;
//
//     //
//     //  Loop while there are more bytes to process or there are
//     //  available entries in the IoRun array.
//     //
//
//     while (TRUE) {
//
//         PerformedCopy = FALSE;
//         *RunCount += 1;
//
//         //
//         //  Initialize the current position in the IoRuns array.  Find the
//         //  eventual destination in the user's buffer for this portion of the transfer.
//         //
//
//         ThisIoRun->UserBuffer = CurrentUserBuffer;
//
//         //
//         //  Find the allocation information for the current offset in the
//         //  stream.
//         //
//
//         CdLookupAllocation( IrpContext,
//                             Fcb,
//                             CurrentCookedOffset,
//                             &DiskOffset,
//                             &CurrentCookedByteCount );
//         //
//         //  Maybe we got lucky and this is the same sector as in the
//         //  Vcb.
//         //
//
//         if (DiskOffset == Fcb->Vcb->XADiskOffset) {
//
//             //
//             //  We will perform safe synchronization.  Check again that
//             //  this is the correct sector.
//             //
//
//             CdLockVcb( IrpContext, Fcb->Vcb );
//
//             if ((DiskOffset == Fcb->Vcb->XADiskOffset) &&
//                 (Fcb->Vcb->XASector != NULL)) {
//
//                 //
//                 //  Copy any bytes we can from the current sector.
//                 //
//
//                 CurrentRawByteCount = RAW_SECTOR_SIZE - RawSectorOffset;
//
//                 //
//                 //  Check whether we don't go to the end of the sector.
//                 //
//
//                 if (CurrentRawByteCount > RemainingRawByteCount) {
//
//                     CurrentRawByteCount = RemainingRawByteCount;
//                 }
//
//                 RtlCopyMemory( CurrentUserBuffer,
//                                Add2Ptr( Fcb->Vcb->XASector, RawSectorOffset, PCHAR ),
//                                CurrentRawByteCount );
//
//                 CdUnlockVcb( IrpContext, Fcb->Vcb );
//
//                 //
//                 //  Adjust the run count and pointer in the IoRuns array
//                 //  to show that we didn't use a position.
//                 //
//
//                 *RunCount -= 1;
//                 ThisIoRun -= 1;
//
//                 //
//                 //  Remember that we performed a copy operation.
//                 //
//
//                 PerformedCopy = TRUE;
//
//                 CurrentCookedByteCount = SECTOR_SIZE;
//
//             } else {
//
//                 //
//                 //  The safe test showed no available buffer.  Drop down to common code to
//                 //  perform the Io.
//                 //
//
//                 CdUnlockVcb( IrpContext, Fcb->Vcb );
//             }
//         }
//
//         //
//         //  No work in this pass if we did a copy operation.
//         //
//
//         if (!PerformedCopy) {
//
//             //
//             //  Limit ourselves by the number of remaining cooked bytes.
//             //
//
//             if (CurrentCookedByteCount > RemainingCookedByteCount) {
//
//                 CurrentCookedByteCount = RemainingCookedByteCount;
//             }
//
//             ThisIoRun->DiskOffset = DiskOffset;
//             ThisIoRun->TransferBufferOffset = RawSectorOffset;
//
//             //
//             //  We will always need to perform copy operations for XA files.
//             //  We allocate an auxillary buffer to read the start of the
//             //  transfer.  Then we can use a range of the user's buffer to
//             //  perform the next range of the transfer.  Finally we may
//             //  need to allocate a buffer for the tail of the transfer.
//             //
//             //  We can use the user's buffer (at the current scratch buffer) if the
//             //  following are true:
//             //
//             //      If we are to store the beginning of the raw sector in the user's buffer.
//             //      The current scratch buffer precedes the destination in the user's buffer
//             //          (and hence also lies within it)
//             //      There are enough bytes remaining in the buffer for at least one
//             //          raw sector.
//             //
//
//             if ((RawSectorOffset == 0) &&
//                 (RemainingRawByteCount >= RAW_SECTOR_SIZE)) {
//
//                 //
//                 //  We can use the scratch buffer.  We must ensure we don't send down reads
//                 //  greater than the device can handle, since the driver is unable to split
//                 //  raw requests.
//                 //
//
//                 if (CurrentCookedByteCount <= Fcb->Vcb->MaximumTransferRawSectors * SECTOR_SIZE) {
//
//                     CurrentRawByteCount = (SectorAlign( CurrentCookedByteCount) >> SECTOR_SHIFT) * RAW_SECTOR_SIZE;
//
//                 } else {
//
//                     CurrentCookedByteCount = Fcb->Vcb->MaximumTransferRawSectors * SECTOR_SIZE;
//                     CurrentRawByteCount = Fcb->Vcb->MaximumTransferRawSectors * RAW_SECTOR_SIZE;
//                 }
//
//                 //
//                 //  Now make sure we are within the page transfer limit.
//                 //
//
//                 while (ADDRESS_AND_SIZE_TO_SPAN_PAGES(CurrentUserBuffer, RawSectorAlign( CurrentRawByteCount)) >
//                        Fcb->Vcb->MaximumPhysicalPages )  {
//
//                     CurrentRawByteCount -= RAW_SECTOR_SIZE;
//                     CurrentCookedByteCount -= SECTOR_SIZE;
//                 }
//
//                 //
//                 //  Trim the number of bytes to read if it won't fit into the current buffer. Take
//                 //  account of the fact that we must read in whole raw sector multiples.
//                 //
//
//                 while (RawSectorAlign( CurrentRawByteCount) > RemainingRawByteCount)  {
//
//                     CurrentRawByteCount -= RAW_SECTOR_SIZE;
//                     CurrentCookedByteCount -= SECTOR_SIZE;
//                 }
//
//                 //
//                 //  Now trim the maximum number of raw bytes to the remaining bytes.
//                 //
//
//                 if (CurrentRawByteCount > RemainingRawByteCount) {
//
//                     CurrentRawByteCount = RemainingRawByteCount;
//                 }
//
//                 //
//                 //  Update the IO run array.  We point to the scratch buffer as
//                 //  well as the buffer and Mdl in the original Irp.
//                 //
//
//                 ThisIoRun->DiskByteCount = SectorAlign( CurrentCookedByteCount);
//
//                 //
//                 //  Point to the user's buffer and Mdl for this transfer.
//                 //
//
//                 ThisIoRun->TransferBuffer = CurrentUserBuffer;
//                 ThisIoRun->TransferMdl = Irp->MdlAddress;
//                 ThisIoRun->TransferVirtualAddress = Add2Ptr( Irp->UserBuffer,
//                                                              CurrentUserBufferOffset,
//                                                              PVOID);
//
//             } else {
//
//                 //
//                 //  We need to determine the number of bytes to transfer and the
//                 //  offset into this page to begin the transfer.
//                 //
//                 //  We will transfer only one raw sector.
//                 //
//
//                 ThisIoRun->DiskByteCount = SECTOR_SIZE;
//
//                 CurrentCookedByteCount = SECTOR_SIZE;
//
//                 ThisIoRun->TransferByteCount = RAW_SECTOR_SIZE - RawSectorOffset;
//                 ThisIoRun->TransferBufferOffset = RawSectorOffset;
//
//                 if (ThisIoRun->TransferByteCount > RemainingRawByteCount) {
//
//                     ThisIoRun->TransferByteCount = RemainingRawByteCount;
//                 }
//
//                 CurrentRawByteCount = ThisIoRun->TransferByteCount;
//
//                 //
//                 //  We need to allocate an auxillary buffer.  We will allocate
//                 //  a single page.  Then we will build an Mdl to describe the buffer.
//                 //
//
//                 ThisIoRun->TransferBuffer = FsRtlAllocatePoolWithTag( CdNonPagedPool, PAGE_SIZE, TAG_IO_BUFFER );
//
//                 //
//                 //  Allocate and build the Mdl to describe this buffer.
//                 //
//
//                 ThisIoRun->TransferMdl = IoAllocateMdl( ThisIoRun->TransferBuffer,
//                                                         PAGE_SIZE as u32,
//                                                         FALSE,
//                                                         FALSE,
//                                                         NULL );
//
//                 ThisIoRun->TransferVirtualAddress = ThisIoRun->TransferBuffer;
//
//                 if (ThisIoRun->TransferMdl == NULL) {
//
//                     IrpContext->Irp->IoStatus.Information = 0;
//                     CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//                 }
//
//                 MmBuildMdlForNonPagedPool( ThisIoRun->TransferMdl );
//             }
//         }
//
//         //
//         //  Update the byte count for our caller.
//         //
//
//         RemainingRawByteCount -= CurrentRawByteCount;
//         *ThisByteCount += CurrentRawByteCount;
//
//         //
//         //  Break out if no more positions in the IoRuns array or
//         //  we have all of the bytes accounted for.
//         //
//
//         if ((RemainingRawByteCount == 0) || (*RunCount == MAX_PARALLEL_IOS)) {
//
//             break;
//         }
//
//         //
//         //  Update our local pointers to allow for the current range of bytes.
//         //
//
//         ThisIoRun += 1;
//
//         CurrentUserBuffer = Add2Ptr( CurrentUserBuffer, CurrentRawByteCount, PVOID );
//         CurrentUserBufferOffset += CurrentRawByteCount;
//
//         RawSectorOffset = 0;
//
//         CurrentCookedOffset += CurrentCookedByteCount;
//         RemainingCookedByteCount -= CurrentCookedByteCount;
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
// CdFinishBuffers (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN IoRuns,
//      ULONG RunCount,
//      BOOLEAN FinalCleanup,
//      BOOLEAN SaveXABuffer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to perform any data transferred required for
//     unaligned Io or to perform the final cleanup of the IoRuns array.
//
//     In all cases this is where we will deallocate any buffer and mdl
//     allocated to perform the unaligned transfer.  If this is not the
//     final cleanup then we also transfer the bytes to the user buffer
//     and flush the hardware cache.
//
//     We walk backwards through the run array because we may be shifting data
//     in the user's buffer.  Typical case is where we allocated a buffer for
//     the first part of a read and then used the user's buffer for the
//     next section (but stored it at the beginning of the buffer.
//
// Arguments:
//
//     IoRuns - Pointer to the IoRuns array.
//
//     RunCount - Number of entries in the IoRuns array filled here.
//
//     FinalCleanup - Indicates if we should be deallocating temporary buffers
//         (TRUE) or transferring bytes for a unaligned transfers and
//         deallocating the buffers (FALSE).  Flush the system cache if
//         transferring data.
//
//     SaveXABuffer - TRUE if we should try to save an XA buffer, FALSE otherwise
//
// Return Value:
//
//     BOOLEAN - TRUE if this request needs the Io buffers to be flushed, FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN FlushIoBuffers = FALSE;
//
//     ULONG RemainingEntries = RunCount;
//     PIO_RUN ThisIoRun = &IoRuns[RunCount - 1];
//     PVCB Vcb;
//
//     PAGED_CODE();
//
//     //
//     //  Walk through each entry in the IoRun array.
//     //
//
//     while (RemainingEntries != 0) {
//
//         //
//         //  We only need to deal with the case of an unaligned transfer.
//         //
//
//         if (ThisIoRun->TransferByteCount != 0) {
//
//             //
//             //  If not the final cleanup then transfer the data to the
//             //  user's buffer and remember that we will need to flush
//             //  the user's buffer to memory.
//             //
//
//             if (!FinalCleanup) {
//
//                 RtlCopyMemory( ThisIoRun->UserBuffer,
//                                Add2Ptr( ThisIoRun->TransferBuffer,
//                                         ThisIoRun->TransferBufferOffset,
//                                         PVOID ),
//                                ThisIoRun->TransferByteCount );
//
//                 FlushIoBuffers = TRUE;
//             }
//
//             //
//             //  Free any Mdl we may have allocated.  If the Mdl isn't
//             //  present then we must have failed during the allocation
//             //  phase.
//             //
//
//             if (ThisIoRun->TransferMdl != IrpContext->Irp->MdlAddress) {
//
//                 if (ThisIoRun->TransferMdl != NULL) {
//
//                     IoFreeMdl( ThisIoRun->TransferMdl );
//                 }
//
//                 //
//                 //  Now free any buffer we may have allocated.  If the Mdl
//                 //  doesn't match the original Mdl then free the buffer.
//                 //
//
//                 if (ThisIoRun->TransferBuffer != NULL) {
//
//                     //
//                     //  If this is the final buffer for an XA read then store this buffer
//                     //  into the Vcb so that we will have it when reading any remaining
//                     //  portion of this buffer.
//                     //
//
//                     if (SaveXABuffer) {
//
//                         Vcb = IrpContext->Vcb;
//
//                         CdLockVcb( IrpContext, Vcb );
//
//                         if (Vcb->XASector != NULL) {
//
//                             CdFreePool( &Vcb->XASector );
//                         }
//
//                         Vcb->XASector = ThisIoRun->TransferBuffer;
//                         Vcb->XADiskOffset = ThisIoRun->DiskOffset;
//
//                         SaveXABuffer = FALSE;
//
//                         CdUnlockVcb( IrpContext, Vcb );
//
//                     //
//                     //  Otherwise just free the buffer.
//                     //
//
//                     } else {
//
//                         CdFreePool( &ThisIoRun->TransferBuffer );
//                     }
//                 }
//             }
//         }
//
//         //
//         //  Now handle the case where we failed in the process
//         //  of allocating associated Irps and Mdls.
//         //
//
//         if (ThisIoRun->SavedIrp != NULL) {
//
//             if (ThisIoRun->SavedIrp->MdlAddress != NULL) {
//
//                 IoFreeMdl( ThisIoRun->SavedIrp->MdlAddress );
//             }
//
//             IoFreeIrp( ThisIoRun->SavedIrp );
//         }
//
//         //
//         //  Move to the previous IoRun entry.
//         //
//
//         ThisIoRun -= 1;
//         RemainingEntries -= 1;
//     }
//
//     //
//     //  If we copied any data then flush the Io buffers.
//     //
//
//     return FlushIoBuffers;
// }
//
// //  Tell prefast this is a completion routine.
// IO_COMPLETION_ROUTINE CdSyncCompletionRoutine;
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdSyncCompletionRoutine (
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Contxt
//     )
//
// /*++
//
// Routine Description:
//
//     Completion routine for synchronizing back to dispatch.
//
// Arguments:
//
//     Contxt - pointer to KEVENT.
//
// Return Value:
//
//     STATUS_MORE_PROCESSING_REQUIRED
//
// --*/
//
// {
//     PKEVENT Event = (PKEVENT)Contxt;
//     _Analysis_assume_(Contxt != NULL);
//
//     UNREFERENCED_PARAMETER( Irp );
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     KeSetEvent( Event, 0, FALSE );
//
//     //
//     //  We don't want IO to get our IRP and free it.
//     //
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdFreeDirCache (
//      PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     Safely frees the sector cache buffer.
//
// Arguments:
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
//     if (NULL != IrpContext->Vcb->SectorCacheBuffer) {
//
//         CdAcquireCacheForUpdate( IrpContext);
//         CdFreePool( &IrpContext->Vcb->SectorCacheBuffer);
//         CdReleaseCache( IrpContext);
//     }
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// CdReadDirDataThroughCache (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN Run
//     )
//
// /*++
//
// Routine Description:
//
//     Reads blocks through the sector cache. If the data is present, then it
//     is copied from memory.  If not present, one of the cache chunks will be
//     replaced with a chunk containing the requested region, and the data
//     copied from there.
//
//     Only intended for reading *directory* blocks, for the purpose of pre-caching
//     directory information, by reading a chunk of blocks which hopefully contains
//     other directory blocks, rather than just the (usually) single block requested.
//
// Arguments:
//
//     Run - description of extent required, and buffer to read into.
//
// Return Value:
//
//     None. Raises on error.
//
// --*/
//
// {
//     PVCB Vcb          = IrpContext->Vcb;
//     ULONG Lbn         = SectorsFromLlBytes( Run->DiskOffset);
//     ULONG Remaining   = SectorsFromBytes( Run->DiskByteCount);
//     PUCHAR UserBuffer = Run->TransferBuffer;
//
//     NTSTATUS Status;
//     ULONG Found;
//     ULONG BufferSectorOffset;
//     ULONG StartBlock;
//     ULONG EndBlock;
//     ULONG Blocks;
//
//     PIO_STACK_LOCATION IrpSp;
//     IO_STATUS_BLOCK Iosb;
//
//     PTRACK_DATA TrackData;
//
// #if DBG
//     BOOLEAN JustRead = FALSE;
// #endif
//
//     ULONG Index;
//     PCD_SECTOR_CACHE_CHUNK Buffer;
//     BOOLEAN Result = FALSE;
//
//     PAGED_CODE();
//
//     CdAcquireCacheForRead( IrpContext);
//
//     _SEH2_TRY {
//
//         //
//         //  Check the cache hasn't gone away due to volume verify failure (which
//         //  is the *only* reason it'll go away).  If this is the case we raise
//         //  the same error any I/O would return if the cache weren't here.
//         //
//
//         if (NULL == Vcb->SectorCacheBuffer) {
//
//             CdRaiseStatus( IrpContext, STATUS_VERIFY_REQUIRED);
//         }
//
//         while (Remaining) {
//
//             Buffer = NULL;
//
//             //
//             //  Look to see if any portion is currently cached.
//             //
//
//             for (Index = 0; Index < CD_SEC_CACHE_CHUNKS; Index++) {
//
//                 if ((Vcb->SecCacheChunks[ Index].BaseLbn != -1) &&
//                     (Vcb->SecCacheChunks[ Index].BaseLbn <= Lbn) &&
//                     ((Vcb->SecCacheChunks[ Index].BaseLbn + CD_SEC_CHUNK_BLOCKS) > Lbn)) {
//
//                     Buffer = &Vcb->SecCacheChunks[ Index];
//                     break;
//                 }
//             }
//
//             //
//             //  If we found any, copy it out and continue.
//             //
//
//             if (NULL != Buffer) {
//
//                 BufferSectorOffset = Lbn - Buffer->BaseLbn;
//                 Found = Min( CD_SEC_CHUNK_BLOCKS - BufferSectorOffset, Remaining);
//
//                 RtlCopyMemory( UserBuffer,
//                                Buffer->Buffer + BytesFromSectors( BufferSectorOffset),
//                                BytesFromSectors( Found));
//
//                 Remaining -= Found;
//                 UserBuffer += BytesFromSectors( Found);
//                 Lbn += Found;
// #if DBG
//                 //
//                 //  Update stats.  Don't count a hit if we've just read the data in.
//                 //
//
//                 if (!JustRead) {
//
//                     InterlockedIncrement( (LONG*)&Vcb->SecCacheHits);
//                 }
//
//                 JustRead = FALSE;
// #endif
//                 continue;
//             }
//
//             //
//             //  Missed the cache, so we need to read a new chunk.  Take the cache
//             //  resource exclusive while we do so.
//             //
//
//             CdReleaseCache( IrpContext);
//             CdAcquireCacheForUpdate( IrpContext);
// #if DBG
//             Vcb->SecCacheMisses += 1;
// #endif
//             //
//             //  Select the chunk to replace and calculate the start block of the
//             //  chunk to cache.  We cache blocks which start on Lbns aligned on
//             //  multiples of chunk size, treating block 16 (VRS start) as block
//             //  zero.
//             //
//
//             Buffer = &Vcb->SecCacheChunks[ Vcb->SecCacheLRUChunkIndex];
//
//             StartBlock = Lbn - ((Lbn - 16) % CD_SEC_CHUNK_BLOCKS);
//
//             //
//             //  Make sure we don't try and read past end of the last track.
//             //
//
// #ifdef __REACTOS__
//             if (Vcb->CdromToc) {
// #endif
//             TrackData = &Vcb->CdromToc->TrackData[(Vcb->CdromToc->LastTrack - Vcb->CdromToc->FirstTrack + 1)];
//
//             SwapCopyUchar4( &EndBlock, &TrackData->Address );
//
//             Blocks = EndBlock - StartBlock;
//
//             if (Blocks > CD_SEC_CHUNK_BLOCKS) {
//
//                 Blocks = CD_SEC_CHUNK_BLOCKS;
//             }
// #ifdef __REACTOS__
//             } else {
//                 // HACK!!!!!!!! Might cause reads to overrun the end of the partition, no idea what consequences that can have.
//                 Blocks = CD_SEC_CHUNK_BLOCKS;
//             }
// #endif
//
//             if ((0 == Blocks) || (Lbn < 16)) {
//
//                 CdRaiseStatus( IrpContext, STATUS_INVALID_PARAMETER);
//             }
//
//             //
//             //  Now build / send the read request.
//             //
//
//             IoReuseIrp( Vcb->SectorCacheIrp, STATUS_SUCCESS);
//
//             KeClearEvent( &Vcb->SectorCacheEvent);
//             Vcb->SectorCacheIrp->Tail.Overlay.Thread = PsGetCurrentThread();
//
//             //
//             // Get a pointer to the stack location of the first driver which will be
//             // invoked.  This is where the function codes and the parameters are set.
//             //
//
//             IrpSp = IoGetNextIrpStackLocation( Vcb->SectorCacheIrp);
//             IrpSp->MajorFunction = (UCHAR) IRP_MJ_READ;
//
//             //
//             //  Build an MDL to describe the buffer.
//             //
//
//             IoAllocateMdl( Buffer->Buffer,
//                            BytesFromSectors( Blocks),
//                            FALSE,
//                            FALSE,
//                            Vcb->SectorCacheIrp);
//
//             if (NULL == Vcb->SectorCacheIrp->MdlAddress)  {
//
//                 IrpContext->Irp->IoStatus.Information = 0;
//                 CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES);
//             }
//
//             //
//             //  We're reading/writing into the block cache (paged pool).  Lock the
//             //  pages and update the MDL with physical page information.
//             //
//
//             _SEH2_TRY {
//
//                 MmProbeAndLockPages( Vcb->SectorCacheIrp->MdlAddress,
//                                      KernelMode,
//                                      (LOCK_OPERATION) IoWriteAccess );
//             }
// #ifdef _MSC_VER
// #pragma warning(suppress: 6320)
// #endif
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//
//                 IoFreeMdl( Vcb->SectorCacheIrp->MdlAddress );
//                 Vcb->SectorCacheIrp->MdlAddress = NULL;
//             } _SEH2_END;
//
//             if (NULL == Vcb->SectorCacheIrp->MdlAddress) {
//
//                 CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//             }
//
//             //
//             //  Reset the BaseLbn as we can't trust this Buffer's data until the request
//             //  is successfully completed.
//             //
//
//             Buffer->BaseLbn = (ULONG)-1;
//
//             IrpSp->Parameters.Read.Length = BytesFromSectors( Blocks);
//             IrpSp->Parameters.Read.ByteOffset.QuadPart = LlBytesFromSectors( StartBlock);
//
//             IoSetCompletionRoutine( Vcb->SectorCacheIrp,
//                                     core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSyncCompletionRoutine as usize),
//                                     &Vcb->SectorCacheEvent,
//                                     TRUE,
//                                     TRUE,
//                                     TRUE );
//
//             Vcb->SectorCacheIrp->UserIosb = &Iosb;
//
//             Status = IoCallDriver( Vcb->TargetDeviceObject, Vcb->SectorCacheIrp );
//
//             if (STATUS_PENDING == Status)  {
//
//
//                 (VOID)KeWaitForSingleObject( &Vcb->SectorCacheEvent,
//                                        Executive,
//                                        KernelMode,
//                                        FALSE,
//                                        NULL );
//
//                 Status = Vcb->SectorCacheIrp->IoStatus.Status;
//             }
//
//             Vcb->SectorCacheIrp->UserIosb = NULL;
//
//             //
//             //  Unlock the pages and free the MDL.
//             //
//
//             MmUnlockPages( Vcb->SectorCacheIrp->MdlAddress );
//             IoFreeMdl( Vcb->SectorCacheIrp->MdlAddress );
//             Vcb->SectorCacheIrp->MdlAddress = NULL;
//
//             if (!NT_SUCCESS( Status )) {
//
//                 try_leave( Status );
//             }
//
//             //
//             //  Update the buffer information, and drop the cache resource to shared
//             //  to allow in reads.
//             //
//
//             Buffer->BaseLbn = StartBlock;
//             Vcb->SecCacheLRUChunkIndex = (Vcb->SecCacheLRUChunkIndex + 1) % CD_SEC_CACHE_CHUNKS;
//
//             CdConvertCacheToShared( IrpContext);
// #if DBG
//             JustRead = TRUE;
// #endif
//         }
//
//         Result = TRUE;
//     }
//     _SEH2_FINALLY {
//
//         CdReleaseCache( IrpContext);
//     } _SEH2_END;
//
//     return Result;
// }
//
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdMultipleAsync (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG RunCount,
//      PIO_RUN IoRuns
//     )
//
// /*++
//
// Routine Description:
//
//     This routine first does the initial setup required of a Master IRP that is
//     going to be completed using associated IRPs.  This routine should not
//     be used if only one async request is needed, instead the single read
//     async routines should be called.
//
//     A context parameter is initialized, to serve as a communications area
//     between here and the common completion routine.
//
//     Next this routine reads or writes one or more contiguous sectors from
//     a device asynchronously, and is used if there are multiple reads for a
//     master IRP.  A completion routine is used to synchronize with the
//     completion of all of the I/O requests started by calls to this routine.
//
//     Also, prior to calling this routine the caller must initialize the
//     IoStatus field in the Context, with the correct success status and byte
//     count which are expected if all of the parallel transfers complete
//     successfully.  After return this status will be unchanged if all requests
//     were, in fact, successful.  However, if one or more errors occur, the
//     IoStatus will be modified to reflect the error status and byte count
//     from the first run (by Vbo) which encountered an error.  I/O status
//     from all subsequent runs will not be indicated.
//
// Arguments:
//
//     RunCount - Supplies the number of multiple async requests
//         that will be issued against the master irp.
//
//     IoRuns - Supplies an array containing the Offset and ByteCount for the
//         separate requests.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_COMPLETION_ROUTINE CompletionRoutine;
//     PIO_STACK_LOCATION IrpSp;
//     PMDL Mdl;
//     PIRP Irp;
//     PIRP MasterIrp;
//     ULONG UnwindRunCount;
//     BOOLEAN UseSectorCache;
//
//     PAGED_CODE();
//
//     //
//     //  Set up things according to whether this is truely async.
//     //
//
//     CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiSyncCompletionRoutine as usize);
//
//     if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//         CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiAsyncCompletionRoutine as usize);
//     }
//
//     //
//     //  For directories, use the sector cache.
//     //
//
//     if ((SafeNodeType( Fcb) == CDFS_NTC_FCB_INDEX) &&
//         (NULL != Fcb->Vcb->SectorCacheBuffer) &&
//         (VcbMounted == IrpContext->Vcb->VcbCondition)) {
//
//         UseSectorCache = TRUE;
//     }
//     else {
//
//         UseSectorCache = FALSE;
//     }
//
//     //
//     //  Initialize some local variables.
//     //
//
//     MasterIrp = IrpContext->Irp;
//
//     //
//     //  Itterate through the runs, doing everything that can fail.
//     //  We let the cleanup in CdFinishBuffers clean up on error.
//     //
//
//     for (UnwindRunCount = 0;
//          UnwindRunCount < RunCount;
//          UnwindRunCount += 1) {
//
//         if (UseSectorCache) {
//
//             if (!CdReadDirDataThroughCache( IrpContext, &IoRuns[ UnwindRunCount])) {
//
//                 //
//                 //  Turn off using directory cache and restart all over again.
//                 //
//
//                 UseSectorCache = FALSE;
//                 UnwindRunCount = 0;
//             }
//
//             continue;
//         }
//
//         //
//         //  Create an associated IRP, making sure there is one stack entry for
//         //  us, as well.
//         //
//
//         IoRuns[UnwindRunCount].SavedIrp =
//         Irp = IoMakeAssociatedIrp( MasterIrp, (CCHAR)(IrpContext->Vcb->TargetDeviceObject->StackSize + 1) );
//
//         if (Irp == NULL) {
//
//             IrpContext->Irp->IoStatus.Information = 0;
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         //
//         // Allocate and build a partial Mdl for the request.
//         //
//
//         Mdl = IoAllocateMdl( IoRuns[UnwindRunCount].TransferVirtualAddress,
//                              IoRuns[UnwindRunCount].DiskByteCount,
//                              FALSE,
//                              FALSE,
//                              Irp );
//
//         if (Mdl == NULL) {
//
//             IrpContext->Irp->IoStatus.Information = 0;
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         IoBuildPartialMdl( IoRuns[UnwindRunCount].TransferMdl,
//                            Mdl,
//                            IoRuns[UnwindRunCount].TransferVirtualAddress,
//                            IoRuns[UnwindRunCount].DiskByteCount );
//
//         //
//         //  Get the first IRP stack location in the associated Irp
//         //
//
//         IoSetNextIrpStackLocation( Irp );
//         IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//         //
//         //  Setup the Stack location to describe our read.
//         //
//
//         IrpSp->MajorFunction = IRP_MJ_READ;
//         IrpSp->Parameters.Read.Length = IoRuns[UnwindRunCount].DiskByteCount;
//         IrpSp->Parameters.Read.ByteOffset.QuadPart = IoRuns[UnwindRunCount].DiskOffset;
//
//         //
//         // Set up the completion routine address in our stack frame.
//         //
//
//         IoSetCompletionRoutine( Irp,
//                                 CompletionRoutine,
//                                 IrpContext->IoContext,
//                                 TRUE,
//                                 TRUE,
//                                 TRUE );
//
//         //
//         //  Setup the next IRP stack location in the associated Irp for the disk
//         //  driver beneath us.
//         //
//
//         IrpSp = IoGetNextIrpStackLocation( Irp );
//
//         //
//         //  Setup the Stack location to do a read from the disk driver.
//         //
//
//         IrpSp->MajorFunction = IRP_MJ_READ;
//         IrpSp->Parameters.Read.Length = IoRuns[UnwindRunCount].DiskByteCount;
//         IrpSp->Parameters.Read.ByteOffset.QuadPart = IoRuns[UnwindRunCount].DiskOffset;
//     }
//
//     //
//     //  If we used the cache, we're done.
//     //
//
//     if (UseSectorCache) {
//
//         if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//             IrpContext->Irp->IoStatus.Status = STATUS_SUCCESS;
//             KeSetEvent( &IrpContext->IoContext->SyncEvent, 0, FALSE );
//         }
//
//         return;
//     }
//
//     //
//     //  We only need to set the associated IRP count in the master irp to
//     //  make it a master IRP.  But we set the count to one more than our
//     //  caller requested, because we do not want the I/O system to complete
//     //  the I/O.  We also set our own count.
//     //
//
//     IrpContext->IoContext->IrpCount = RunCount;
//     IrpContext->IoContext->MasterIrp = MasterIrp;
//
//     //
//     //  We set the count in the master Irp to 1 since typically we
//     //  will clean up the associated irps ourselves.  Setting this to one
//     //  means completing the last associated Irp with SUCCESS (in the async
//     //  case) will complete the master irp.
//     //
//
//     MasterIrp->AssociatedIrp.IrpCount = 1;
//
//     //
//     //  If we (FS) acquired locks, transition the lock owners to an object, since
//     //  when we return this thread could go away before request completion, and
//     //  the resource package may otherwise try to boost priority, etc.
//     //
//
//     if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ) &&
//         FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL )) {
//
//         NT_ASSERT( IrpContext->IoContext->ResourceThreadId == (ERESOURCE_THREAD)PsGetCurrentThread() );
//
//         IrpContext->IoContext->ResourceThreadId = ((ULONG_PTR)IrpContext->IoContext) | 3;
//
//         ExSetResourceOwnerPointer( IrpContext->IoContext->Resource,
//                                    (PVOID)IrpContext->IoContext->ResourceThreadId );
//     }
//
//     //
//     //  Now that all the dangerous work is done, issue the Io requests
//     //
//
//     for (UnwindRunCount = 0;
//          UnwindRunCount < RunCount;
//          UnwindRunCount++) {
//
//         Irp = IoRuns[UnwindRunCount].SavedIrp;
//         IoRuns[UnwindRunCount].SavedIrp = NULL;
//
//         if (NULL != Irp) {
//
//             //
//             //  If IoCallDriver returns an error, it has completed the Irp
//             //  and the error will be caught by our completion routines
//             //  and dealt with as a normal IO error.
//             //
//
//             (VOID) IoCallDriver( IrpContext->Vcb->TargetDeviceObject, Irp );
//         }
//     }
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdMultipleXAAsync (
//      PIRP_CONTEXT IrpContext,
//      ULONG RunCount,
//      PIO_RUN IoRuns,
//      PRAW_READ_INFO RawReads,
//      TRACK_MODE_TYPE TrackMode
//     )
//
// /*++
//
// Routine Description:
//
//     This routine first does the initial setup required of a Master IRP that is
//     going to be completed using associated IRPs.  This routine is used to generate
//     the associated Irps used to read raw sectors from the disk.
//
//     A context parameter is initialized, to serve as a communications area
//     between here and the common completion routine.
//
//     Next this routine reads or writes one or more contiguous sectors from
//     a device asynchronously, and is used if there are multiple reads for a
//     master IRP.  A completion routine is used to synchronize with the
//     completion of all of the I/O requests started by calls to this routine.
//
//     Also, prior to calling this routine the caller must initialize the
//     IoStatus field in the Context, with the correct success status and byte
//     count which are expected if all of the parallel transfers complete
//     successfully.  After return this status will be unchanged if all requests
//     were, in fact, successful.  However, if one or more errors occur, the
//     IoStatus will be modified to reflect the error status and byte count
//     from the first run (by Vbo) which encountered an error.  I/O status
//     from all subsequent runs will not be indicated.
//
// Arguments:
//
//     RunCount - Supplies the number of multiple async requests
//         that will be issued against the master irp.
//
//     IoRuns - Supplies an array containing the Offset and ByteCount for the
//         separate requests.
//
//     RawReads - Supplies an array of structures to store in the Irps passed to the
//         device driver to perform the low-level Io.
//
//     TrackMode - Supplies the recording mode of sectors in these IoRuns
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp;
//     PMDL Mdl;
//     PIRP Irp;
//     PIRP MasterIrp;
//     ULONG UnwindRunCount;
//     ULONG RawByteCount;
//
//     PIO_RUN ThisIoRun = IoRuns;
//     PRAW_READ_INFO ThisRawRead = RawReads;
//
//     PAGED_CODE();
//
//     //
//     //  Initialize some local variables.
//     //
//
//     MasterIrp = IrpContext->Irp;
//
//     //
//     //  Itterate through the runs, doing everything that can fail.
//     //  We let the cleanup in CdFinishBuffers clean up on error.
//     //
//
//     for (UnwindRunCount = 0;
//          UnwindRunCount < RunCount;
//          UnwindRunCount += 1, ThisIoRun += 1, ThisRawRead += 1) {
//
//         //
//         //  Create an associated IRP, making sure there is one stack entry for
//         //  us, as well.
//         //
//
//         ThisIoRun->SavedIrp =
//         Irp = IoMakeAssociatedIrp( MasterIrp, (CCHAR)(IrpContext->Vcb->TargetDeviceObject->StackSize + 1) );
//
//         if (Irp == NULL) {
//
//             IrpContext->Irp->IoStatus.Information = 0;
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         //
//         //  Should have been passed a byte count of at least one sector, and
//         //  must be a multiple of sector size
//         //
//
//         NT_ASSERT( ThisIoRun->DiskByteCount && !SectorOffset(ThisIoRun->DiskByteCount));
//
//         RawByteCount = SectorsFromBytes( ThisIoRun->DiskByteCount) * RAW_SECTOR_SIZE;
//
//         //
//         // Allocate and build a partial Mdl for the request.
//         //
//
//         Mdl = IoAllocateMdl( ThisIoRun->TransferVirtualAddress,
//                              RawByteCount,
//                              FALSE,
//                              FALSE,
//                              Irp );
//
//         if (Mdl == NULL) {
//
//             IrpContext->Irp->IoStatus.Information = 0;
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         IoBuildPartialMdl( ThisIoRun->TransferMdl,
//                            Mdl,
//                            ThisIoRun->TransferVirtualAddress,
//                            RawByteCount);
//         //
//         //  Get the first IRP stack location in the associated Irp
//         //
//
//         IoSetNextIrpStackLocation( Irp );
//         IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//         //
//         //  Setup the Stack location to describe our read (using cooked values)
//         //  These values won't be used for the raw read in any case.
//         //
//
//         IrpSp->MajorFunction = IRP_MJ_READ;
//         IrpSp->Parameters.Read.Length = ThisIoRun->DiskByteCount;
//         IrpSp->Parameters.Read.ByteOffset.QuadPart = ThisIoRun->DiskOffset;
//
//         //
//         // Set up the completion routine address in our stack frame.
//         //
//
//         IoSetCompletionRoutine( Irp,
//                                 core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdMultiSyncCompletionRoutine as usize),
//                                 IrpContext->IoContext,
//                                 TRUE,
//                                 TRUE,
//                                 TRUE );
//
//         //
//         //  Setup the next IRP stack location in the associated Irp for the disk
//         //  driver beneath us.
//         //
//
//         IrpSp = IoGetNextIrpStackLocation( Irp );
//
//         //
//         //  Setup the stack location to do a read of raw sectors at this location.
//         //  Note that the storage stack always reads multiples of whole XA sectors.
//         //
//
//         ThisRawRead->DiskOffset.QuadPart = ThisIoRun->DiskOffset;
//         ThisRawRead->SectorCount = ThisIoRun->DiskByteCount >> SECTOR_SHIFT;
//         ThisRawRead->TrackMode = TrackMode;
//
//         IrpSp->MajorFunction = IRP_MJ_DEVICE_CONTROL;
//
//         IrpSp->Parameters.DeviceIoControl.OutputBufferLength = ThisRawRead->SectorCount * RAW_SECTOR_SIZE;
//         Irp->UserBuffer = ThisIoRun->TransferVirtualAddress;
//
//         IrpSp->Parameters.DeviceIoControl.InputBufferLength = sizeof( RAW_READ_INFO );
//         IrpSp->Parameters.DeviceIoControl.Type3InputBuffer = ThisRawRead;
//
//         IrpSp->Parameters.DeviceIoControl.IoControlCode = IOCTL_CDROM_RAW_READ;
//     }
//
//     //
//     //  We only need to set the associated IRP count in the master irp to
//     //  make it a master IRP.  But we set the count to one more than our
//     //  caller requested, because we do not want the I/O system to complete
//     //  the I/O.  We also set our own count.
//     //
//
//     IrpContext->IoContext->IrpCount = RunCount;
//     IrpContext->IoContext->MasterIrp = MasterIrp;
//
//     //
//     //  We set the count in the master Irp to 1 since typically we
//     //  will clean up the associated irps ourselves.  Setting this to one
//     //  means completing the last associated Irp with SUCCESS (in the async
//     //  case) will complete the master irp.
//     //
//
//     MasterIrp->AssociatedIrp.IrpCount = 1;
//
//     //
//     //  Now that all the dangerous work is done, issue the Io requests
//     //
//
//     for (UnwindRunCount = 0;
//          UnwindRunCount < RunCount;
//          UnwindRunCount++) {
//
//         Irp = IoRuns[UnwindRunCount].SavedIrp;
//         IoRuns[UnwindRunCount].SavedIrp = NULL;
//
//         //
//         //
//         //  If IoCallDriver returns an error, it has completed the Irp
//         //  and the error will be caught by our completion routines
//         //  and dealt with as a normal IO error.
//         //
//
//         (VOID) IoCallDriver( IrpContext->Vcb->TargetDeviceObject, Irp );
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
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdSingleAsync (
//      PIRP_CONTEXT IrpContext,
//      PIO_RUN Run,
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine reads one or more contiguous sectors from a device
//     asynchronously, and is used if there is only one read necessary to
//     complete the IRP.  It implements the read by simply filling
//     in the next stack frame in the Irp, and passing it on.  The transfer
//     occurs to the single buffer originally specified in the user request.
//
// Arguments:
//
//     ByteOffset - Supplies the starting Logical Byte Offset to begin reading from
//
//     ByteCount - Supplies the number of bytes to read from the device
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp;
//     PIO_COMPLETION_ROUTINE CompletionRoutine;
//
//     PAGED_CODE();
//
//     //
//     //  For directories, look in the sector cache,
//     //
//
//     if ((SafeNodeType( Fcb) == CDFS_NTC_FCB_INDEX) &&
//         (NULL != Fcb->Vcb->SectorCacheBuffer) &&
//         (VcbMounted == IrpContext->Vcb->VcbCondition)) {
//
//         if (CdReadDirDataThroughCache( IrpContext, Run )) {
//
//             if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//                 IrpContext->Irp->IoStatus.Status = STATUS_SUCCESS;
//                 KeSetEvent( &IrpContext->IoContext->SyncEvent, 0, FALSE );
//             }
//
//             return;
//         }
//     }
//
//     //
//     //  Set up things according to whether this is truely async.
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT )) {
//
//         CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSingleSyncCompletionRoutine as usize);
//
//     } else {
//
//         CompletionRoutine = core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSingleAsyncCompletionRoutine as usize);
//
//         //
//         //  If we (FS) acquired locks, transition the lock owners to an object, since
//         //  when we return this thread could go away before request completion, and
//         //  the resource package may otherwise try to boost priority, etc.
//         //
//
//         if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_TOP_LEVEL )) {
//
//             NT_ASSERT( IrpContext->IoContext->ResourceThreadId == (ERESOURCE_THREAD)PsGetCurrentThread() );
//
//             IrpContext->IoContext->ResourceThreadId = ((ULONG_PTR)IrpContext->IoContext) | 3;
//
//             ExSetResourceOwnerPointer( IrpContext->IoContext->Resource,
//                                        (PVOID)IrpContext->IoContext->ResourceThreadId );
//         }
//     }
//
//     //
//     // Set up the completion routine address in our stack frame.
//     //
//
//     IoSetCompletionRoutine( IrpContext->Irp,
//                             CompletionRoutine,
//                             IrpContext->IoContext,
//                             TRUE,
//                             TRUE,
//                             TRUE );
//
//     //
//     //  Setup the next IRP stack location in the associated Irp for the disk
//     //  driver beneath us.
//     //
//
//     IrpSp = IoGetNextIrpStackLocation( IrpContext->Irp );
//
//     //
//     //  Setup the Stack location to do a read from the disk driver.
//     //
//
//     IrpSp->MajorFunction = IrpContext->MajorFunction;
//     IrpSp->Parameters.Read.Length = Run->DiskByteCount;
//     IrpSp->Parameters.Read.ByteOffset.QuadPart = Run->DiskOffset;
//
//     //
//     //  Issue the Io request
//     //
//
//     //
//     //  If IoCallDriver returns an error, it has completed the Irp
//     //  and the error will be caught by our completion routines
//     //  and dealt with as a normal IO error.
//     //
//
//     (VOID)IoCallDriver( IrpContext->Vcb->TargetDeviceObject, IrpContext->Irp );
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdWaitSync (
//      PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine waits for one or more previously started I/O requests
//     from the above routines, by simply waiting on the event.
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
//     PAGED_CODE();
//
//
//     (VOID)KeWaitForSingleObject( &IrpContext->IoContext->SyncEvent,
//                            Executive,
//                            KernelMode,
//                            FALSE,
//                            NULL );
//
//     KeClearEvent( &IrpContext->IoContext->SyncEvent );
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdMultiSyncCompletionRoutine (
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all synchronous reads
//     started via CdMultipleAsynch.
//
//     The completion routine has has the following responsibilities:
//
//         If the individual request was completed with an error, then
//         this completion routine must see if this is the first error
//         and remember the error status in the Context.
//
//         If the IrpCount goes to 1, then it sets the event in the Context
//         parameter to signal the caller that all of the asynch requests
//         are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//         Irp will no longer be accessible after this routine returns.)
//
//     Context - The context parameter which was specified for all of
//         the multiple asynch I/O requests for this MasterIrp.
//
// Return Value:
//
//     The routine returns STATUS_MORE_PROCESSING_REQUIRED so that we can
//     immediately complete the Master Irp without being in a race condition
//     with the IoCompleteRequest thread trying to decrement the IrpCount in
//     the Master Irp.
//
// --*/
//
// {
//     PCD_IO_CONTEXT IoContext = Context;
//     _Analysis_assume_(Context != NULL);
//
//     AssertVerifyDeviceIrp( Irp );
//
//     //
//     //  If we got an error (or verify required), remember it in the Irp
//     //
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//         InterlockedExchange( &IoContext->Status, Irp->IoStatus.Status );
//         IoContext->MasterIrp->IoStatus.Information = 0;
//     }
//
//     //
//     //  We must do this here since IoCompleteRequest won't get a chance
//     //  on this associated Irp.
//     //
//
//     IoFreeMdl( Irp->MdlAddress );
//     IoFreeIrp( Irp );
//
//     if (InterlockedDecrement( &IoContext->IrpCount ) == 0) {
//
//         //
//         //  Update the Master Irp with any error status from the associated Irps.
//         //
//
//         IoContext->MasterIrp->IoStatus.Status = IoContext->Status;
//         KeSetEvent( &IoContext->SyncEvent, 0, FALSE );
//     }
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdMultiAsyncCompletionRoutine (
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all asynchronous reads
//     started via CdMultipleAsynch.
//
//     The completion routine has has the following responsibilities:
//
//         If the individual request was completed with an error, then
//         this completion routine must see if this is the first error
//         and remember the error status in the Context.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the associated Irp which is being completed.  (This
//         Irp will no longer be accessible after this routine returns.)
//
//     Context - The context parameter which was specified for all of
//              the multiple asynch I/O requests for this MasterIrp.
//
// Return Value:
//
//     Currently always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     PCD_IO_CONTEXT IoContext = Context;
//     _Analysis_assume_(Context != NULL);
//     AssertVerifyDeviceIrp( Irp );
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     //  If we got an error (or verify required), remember it in the Irp
//     //
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//         InterlockedExchange( &IoContext->Status, Irp->IoStatus.Status );
//     }
//
//     //
//     //  Decrement IrpCount and see if it goes to zero.
//     //
//
//     if (InterlockedDecrement( &IoContext->IrpCount ) == 0) {
//
//         //
//         //  Mark the master Irp pending
//         //
//
//         IoMarkIrpPending( IoContext->MasterIrp );
//
//         //
//         //  Update the Master Irp with any error status from the associated Irps.
//         //
//
//         IoContext->MasterIrp->IoStatus.Status = IoContext->Status;
//
//         //
//         //  Update the information field with the correct value.
//         //
//
//         IoContext->MasterIrp->IoStatus.Information = 0;
//
//         if (NT_SUCCESS( IoContext->MasterIrp->IoStatus.Status )) {
//
//             IoContext->MasterIrp->IoStatus.Information = IoContext->RequestedByteCount;
//         }
//
//         //
//         //  Now release the resource
//         //
//
//         _Analysis_assume_lock_held_(*IoContext->Resource);
//         ExReleaseResourceForThreadLite( IoContext->Resource, IoContext->ResourceThreadId );
//
//         //
//         //  and finally, free the context record.
//         //
//
//         CdFreeIoContext( IoContext );
//
//         //
//         //  Return success in this case.
//         //
//
//         return STATUS_SUCCESS;
//
//     } else {
//
//         //
//         //  We need to cleanup the associated Irp and its Mdl.
//         //
//
//         IoFreeMdl( Irp->MdlAddress );
//         IoFreeIrp( Irp );
//
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdSingleSyncCompletionRoutine (
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all reads started via CdSingleAsynch.
//
//     The completion routine has has the following responsibilities:
//
//         It sets the event in the Context parameter to signal the caller
//         that all of the asynch requests are done.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the Irp for this request.  (This Irp will no longer
//         be accessible after this routine returns.)
//
//     Context - The context parameter which was specified in the call to
//         CdSingleAsynch.
//
// Return Value:
//
//     The routine returns STATUS_MORE_PROCESSING_REQUIRED so that we can
//     immediately complete the Master Irp without being in a race condition
//     with the IoCompleteRequest thread trying to decrement the IrpCount in
//     the Master Irp.
//
// --*/
//
// {
//     _Analysis_assume_(Context != NULL);
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     AssertVerifyDeviceIrp( Irp );
//
//     //
//     //  Store the correct information field into the Irp.
//     //
//
//     if (!NT_SUCCESS( Irp->IoStatus.Status )) {
//
//         Irp->IoStatus.Information = 0;
//     }
//
//     KeSetEvent( &((PCD_IO_CONTEXT)Context)->SyncEvent, 0, FALSE );
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdSingleAsyncCompletionRoutine (
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context
//     )
//
// /*++
//
// Routine Description:
//
//     This is the completion routine for all asynchronous reads
//     started via CdSingleAsynch.
//
// Arguments:
//
//     DeviceObject - Pointer to the file system device object.
//
//     Irp - Pointer to the Irp for this request.  (This Irp will no longer
//         be accessible after this routine returns.)
//
//     Context - The context parameter which was specified in the call to
//         CdSingleAsynch.
//
// Return Value:
//
//     Currently always returns STATUS_SUCCESS.
//
// --*/
//
// {
//     PCD_IO_CONTEXT IoContext = Context;
//
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     _Analysis_assume_(IoContext != NULL);
//     AssertVerifyDeviceIrp( Irp );
//
//     //
//     //  Update the information field with the correct value for bytes read.
//     //
//
//     Irp->IoStatus.Information = 0;
//
//     if (NT_SUCCESS( Irp->IoStatus.Status )) {
//
//         Irp->IoStatus.Information = IoContext->RequestedByteCount;
//     }
//
//     //
//     //  Mark the Irp pending
//     //
//
//     IoMarkIrpPending( Irp );
//
//     //
//     //  Now release the resource
//     //
//
//     _Analysis_assume_lock_held_(*IoContext->Resource);
//     ExReleaseResourceForThreadLite( IoContext->Resource, IoContext->ResourceThreadId );
//
//     //
//     //  and finally, free the context record.
//     //
//
//     CdFreeIoContext( IoContext );
//     return STATUS_SUCCESS;
//
// }
//
//
// //
// //  Local support routine
// //
//
// (SafeNodeType(Fcb) != CDFS_NTC_FCB_PATH_TABLE && StartingOffset == 0, (ByteCount, (>=, CdAudioDirentSize + sizeof(RAW_DIRENT))))
// (SafeNodeType(Fcb) != CDFS_NTC_FCB_PATH_TABLE && StartingOffset != 0, (ByteCount, ))
// VOID
// CdReadAudioSystemFile (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      LONGLONG StartingOffset,
//       ULONG ByteCount,
//      PVOID SystemBuffer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to read the pseudo root directory and path
//     table for a music disk.  We build the individual elements on the
//     stack and copy into the cache buffer.
//
// Arguments:
//
//     Fcb - Fcb representing the file to read.
//
//     StartingOffset - Logical offset in the file to read from.
//
//     ByteCount - Number of bytes to read.
//
//     SystemBuffer - Pointer to buffer to fill in.  This will always be page
//         aligned.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PRAW_PATH_ISO RawPath;
//     PRAW_DIRENT RawDirent;
//
//     ULONG CurrentTrack;
//     ULONG SectorOffset;
//     ULONG EntryCount;
//     UCHAR TrackOnes;
//     UCHAR TrackTens;
//     PTRACK_DATA ThisTrack;
//
//     LONGLONG CurrentOffset;
//
//     PVOID CurrentSector;
//
//     PSYSTEM_USE_XA SystemUse;
//
//     ULONG BytesToCopy;
//
//     UCHAR LocalBuffer[FIELD_OFFSET( RAW_DIRENT, FileId ) + 12];
//
//     PAGED_CODE();
//
//     //
//     //  If this is the path table then we just need a single entry.
//     //
//
//     if (SafeNodeType( Fcb ) == CDFS_NTC_FCB_PATH_TABLE) {
//
//         //
//         //  Sanity check that the offset is zero.
//         //
//
//         NT_ASSERT( StartingOffset == 0 );
//
//         //
//         //  Store a pseudo path entry in our local buffer.
//         //
//
//         RawPath = (PRAW_PATH_ISO) LocalBuffer;
//
//         RtlZeroMemory( RawPath, sizeof( LocalBuffer ));
//
//         RawPath->DirIdLen = 1;
//         RawPath->ParentNum = 1;
//         RawPath->DirId[0] = '\0';
//
//         //
//         //  Now copy to the user's buffer.
//         //
//
//         BytesToCopy = FIELD_OFFSET( RAW_PATH_ISO, DirId ) + 2;
//
//         if (BytesToCopy > ByteCount) {
//
//             BytesToCopy = ByteCount;
//         }
//
//         RtlCopyMemory( SystemBuffer,
//                        RawPath,
//                        BytesToCopy );
//
//     //
//     //  We need to deal with the multiple sector case for the root directory.
//     //
//
//     } else {
//
//         //
//         //  Initialize the first track to return to our caller.
//         //
//
//         CurrentTrack = 0;
//
//         //
//         //  If the offset is zero then store the entries for the self and parent
//         //  entries.
//         //
//
//         if (StartingOffset == 0) {
//
//             RawDirent = SystemBuffer;
//
//             //
//             //  Clear all of the fields initially.
//             //
//
//             RtlZeroMemory( RawDirent, FIELD_OFFSET( RAW_DIRENT, FileId ));
//
//             //
//             //  Now fill in the interesting fields.
//             //
//
//             RawDirent->DirLen = FIELD_OFFSET( RAW_DIRENT, FileId ) + 1;
//             RawDirent->FileIdLen = 1;
//             RawDirent->FileId[0] = '\0';
//             SetFlag( RawDirent->FlagsISO, CD_ATTRIBUTE_DIRECTORY );
//
//             //
//             //  Set the time stamp to be Jan 1, 1995
//             //
//
//             RawDirent->RecordTime[0] = 95;
//             RawDirent->RecordTime[1] = 1;
//             RawDirent->RecordTime[2] = 1;
//
//             SectorOffset = RawDirent->DirLen;
//
//             RawDirent = Add2Ptr( RawDirent, SectorOffset, PRAW_DIRENT );
//
//             //
//             //  Clear all of the fields initially.
//             //
//
//             RtlZeroMemory( RawDirent, FIELD_OFFSET( RAW_DIRENT, FileId ));
//
//             //
//             //  Now fill in the interesting fields.
//             //
//
//             RawDirent->DirLen = FIELD_OFFSET( RAW_DIRENT, FileId ) + 1;
//             RawDirent->FileIdLen = 1;
//             RawDirent->FileId[0] = '\1';
//             SetFlag( RawDirent->FlagsISO, CD_ATTRIBUTE_DIRECTORY );
//
//             //
//             //  Set the time stamp to be Jan 1, 1995
//             //
//
//             RawDirent->RecordTime[0] = 95;
//             RawDirent->RecordTime[1] = 1;
//             RawDirent->RecordTime[2] = 1;
//
//             SectorOffset += RawDirent->DirLen;
//             EntryCount = 2;
//
//         //
//         //  Otherwise compute the starting track to write to the buffer.
//         //
//
//         } else {
//
//             //
//             //  Count the tracks in each preceding sector.
//             //
//
//             CurrentOffset = 0;
//
//             do {
//
//                 CurrentTrack += CdAudioDirentsPerSector;
//                 CurrentOffset += SECTOR_SIZE;
//
//             } while (CurrentOffset < StartingOffset);
//
//             //
//             //  Bias the track count to reflect the two default entries.
//             //
//
//             CurrentTrack -= 2;
//
//             SectorOffset = 0;
//             EntryCount = 0;
//         }
//
//         //
//         //  We now know the first track to return as well as where we are in
//         //  the current sector.  We will walk through sector by sector adding
//         //  the entries for the separate tracks in the TOC.  We will zero
//         //  any sectors or partial sectors without data.
//         //
//
//         CurrentSector = SystemBuffer;
//         BytesToCopy = SECTOR_SIZE;
//
//         //
//         //  Loop for each sector.
//         //
//
//         do {
//
//             //
//             //  Add entries until we reach our threshold for each sector.
//             //
//
//             do {
//
//                 //
//                 //  If we are beyond the entries in the TOC then exit.
//                 //
//
//                 if (CurrentTrack >= IrpContext->Vcb->TrackCount) {
//
//                     break;
//                 }
//
//                 ThisTrack = &IrpContext->Vcb->CdromToc->TrackData[CurrentTrack];
//
//                 //
//                 //  Point to the current position in the buffer.
//                 //
//
//                 RawDirent = Add2Ptr( CurrentSector, SectorOffset, PRAW_DIRENT );
//
//                 //
//                 //  Clear all of the fields initially.
//                 //
//
//                 RtlZeroMemory( RawDirent, CdAudioDirentSize );
//
//                 //
//                 //  Now fill in the interesting fields.
//                 //
//
//                 RawDirent->DirLen = (UCHAR) CdAudioDirentSize;
//                 RawDirent->FileIdLen = CdAudioFileNameLength;
//
//                 RtlCopyMemory( RawDirent->FileId,
//                                CdAudioFileName,
//                                CdAudioFileNameLength );
//
//                 //
//                 //  Set the time stamp to be Jan 1, 1995 00:00
//                 //
//
//                 RawDirent->RecordTime[0] = 95;
//                 RawDirent->RecordTime[1] = 1;
//                 RawDirent->RecordTime[2] = 1;
//
//                 //
//                 //  Put the track number into the file name.
//                 //
//
//                 TrackTens = TrackOnes = ThisTrack->TrackNumber;
//
//                 TrackOnes = (TrackOnes % 10) + '0';
//
//                 TrackTens /= 10;
//                 TrackTens = (TrackTens % 10) + '0';
//
//                 RawDirent->FileId[AUDIO_NAME_TENS_OFFSET] = TrackTens;
//                 RawDirent->FileId[AUDIO_NAME_ONES_OFFSET] = TrackOnes;
//
//                 SystemUse = Add2Ptr( RawDirent, CdAudioSystemUseOffset, PSYSTEM_USE_XA );
//
//                 SystemUse->Attributes = SYSTEM_USE_XA_DA;
//                 SystemUse->Signature = SYSTEM_XA_SIGNATURE;
//
//                 //
//                 //  Store the track number as the file number.
//                 //
//
//                 SystemUse->FileNumber = (UCHAR) CurrentTrack;
//
//                 EntryCount += 1;
//                 SectorOffset += CdAudioDirentSize;
//                 CurrentTrack += 1;
//
//             } while (EntryCount < CdAudioDirentsPerSector);
//
//             //
//             //  Zero the remaining portion of this buffer.
//             //
//
//             RtlZeroMemory( Add2Ptr( CurrentSector, SectorOffset, PVOID ),
//                            SECTOR_SIZE - SectorOffset );
//
//             //
//             //  Prepare for the next sector.
//             //
//
//             EntryCount = 0;
//             BytesToCopy += SECTOR_SIZE;
//             SectorOffset = 0;
//             CurrentSector = Add2Ptr( CurrentSector, SECTOR_SIZE, PVOID );
//
//         } while (BytesToCopy <= ByteCount);
//     }
//
//     return;
// }
//
//
// NTSTATUS
// CdHijackIrpAndFlushDevice (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp,
//      PDEVICE_OBJECT TargetDeviceObject
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
//     NextIrpSp = IoGetNextIrpStackLocation( Irp );
//
//     *NextIrpSp = *IoGetCurrentIrpStackLocation( Irp );
//
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
//                             core::mem::transmute::<_, PIO_COMPLETION_ROUTINE>(CdSyncCompletionRoutine as usize),
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
//         (VOID)KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, NULL );
//
//         Status = Irp->IoStatus.Status;
//     }
//
//     //
//     //  If the driver doesn't support flushes, return SUCCESS.
//     //
//
//     if (Status == STATUS_INVALID_DEVICE_REQUEST) {
//
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
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
