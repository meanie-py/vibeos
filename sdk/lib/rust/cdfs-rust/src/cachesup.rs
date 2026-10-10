use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{CcMdlReadComplete, CcUninitializeCacheMap, IoGetCurrentIrpStackLocation, MmFlushImageSection, _MMFLUSH_TYPE_MmFlushForWrite, PFILE_OBJECT, PIRP, FSRTL_ADVANCED_FCB_HEADER, RtlTimeFieldsToTime, TIME_FIELDS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DIRENT, DIRENT_ENUM_CONTEXT, FCB, FCB_INDEX, FCB_STATE_INITIALIZED, IRP_CONTEXT, PIRP_CONTEXT, VCB, _TYPE_OF_OPEN_StreamFileOpen as StreamFileOpen};
use ntkrnl_sys::ddk::{FILE_OBJECT, LARGE_INTEGER, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FILE_ATTRIBUTE_HIDDEN, STATUS_FILE_CORRUPT_ERROR};
use crate::types::{CD_ATTRIBUTE_HIDDEN};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdData, CdUnicodeSelfArray};
use crate::ddk_shims::Add2Ptr;

// Verified vs sdk/include/psdk/ntstatus.h: STATUS_UNABLE_TO_DELETE_SECTION 0xC000001B
const STATUS_UNABLE_TO_DELETE_SECTION: NTSTATUS = 0xC000001B_u32 as i32;
// Verified vs drivers/filesystems/cdfs/cd.h: CD_BASE_YEAR (1900), VCB_STATE_HSG (0x1)
const CD_BASE_YEAR: i32 = 1900;
const VCB_STATE_HSG: u32 = 0x00000001;
// Verified vs drivers/filesystems/cdfs/cd.h: GMT_OFFSET_TO_NT ((LONGLONG)15*60*1000*1000*10)
const GMT_OFFSET_TO_NT: i64 = 15 * 60 * 1000 * 1000 * 10;

/// Helper accessor: FSRTL header is the unnamed FSRTL_ADVANCED_FCB_HEADER union member (anon_1).
#[inline]
unsafe fn fcb_header(Fcb: *mut FCB) -> *mut FSRTL_ADVANCED_FCB_HEADER {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_1).cast::<FSRTL_ADVANCED_FCB_HEADER>()
}

/// Helper accessor: FCB_INDEX is the unnamed union member (anon_2) for index/path-table Fcbs.
#[inline]
unsafe fn fcb_index(Fcb: *mut FCB) -> *mut FCB_INDEX {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2).cast::<FCB_INDEX>()
}

// cdprocs.h static INLINE CdVerifyOrCreateDirStreamFile — provided out-of-line for
// the Rust cohab (the C header inline is `static`, so no exported symbol exists, and
// the translated .c that would inline it is commented out). cdecl (no NTAPI).
//   if (NULL == Fcb->FileObject)
//       CdCreateInternalStream(IrpContext, Fcb->Vcb, Fcb,
//                              &Fcb->FileNamePrefix.ExactCaseName.FileName);
#[no_mangle]
pub unsafe extern "C" fn CdVerifyOrCreateDirStreamFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    if (*fcb_index(Fcb)).FileObject.is_null() {
        CdCreateInternalStream(
            IrpContext,
            (*Fcb).Vcb,
            Fcb,
            core::ptr::addr_of_mut!((*Fcb).FileNamePrefix.ExactCaseName.FileName)
                as *mut _UNICODE_STRING,
        );
    }
}

/// Inlined from drivers/filesystems/cdfs/cd.h: CdConvertCdTimeToNtTime(IC,CD,NT) fn-macro.
#[inline]
unsafe fn CdConvertCdTimeToNtTime(
    IrpContext: *mut IRP_CONTEXT,
    CdTime: *mut i8,
    NtTime: *mut LARGE_INTEGER,
) {
    let cd = CdTime as *mut core::ffi::c_void;
    let mut TimeField: TIME_FIELDS = core::mem::zeroed();
    TimeField.Year = (*(cd as *const i8) as i32 + CD_BASE_YEAR) as i16;
    TimeField.Month = *(Add2Ptr(cd, 1) as *const i8) as i16;
    TimeField.Day = *(Add2Ptr(cd, 2) as *const i8) as i16;
    TimeField.Hour = *(Add2Ptr(cd, 3) as *const i8) as i16;
    TimeField.Minute = *(Add2Ptr(cd, 4) as *const i8) as i16;
    TimeField.Second = *(Add2Ptr(cd, 5) as *const i8) as i16;
    TimeField.Milliseconds = 0;
    RtlTimeFieldsToTime(&mut TimeField, NtTime);
    let GmtOffset: i8 = *(Add2Ptr(cd, 6) as *const i8);
    if FlagOn((*(*IrpContext).Vcb).VcbState, VCB_STATE_HSG) == 0
        && GmtOffset != 0
        && GmtOffset >= -48
        && GmtOffset <= 52
    {
        (*NtTime).QuadPart += -(GmtOffset as i64) * GMT_OFFSET_TO_NT;
    }
}

extern "C" {
    fn ASSERT_IRP_CONTEXT(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn ASSERT_FCB(Fcb: *mut FCB);
}
extern "C" {
    fn CdLockFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdUnlockFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "system" {
    fn IoCreateStreamFileObjectLite( FileObject: *mut core::ffi::c_void, DeviceObject: *mut core::ffi::c_void, ) -> *mut FILE_OBJECT;
}
extern "C" {
    fn CdSetFileObject( IrpContext: *mut IRP_CONTEXT, FileObject: *mut FILE_OBJECT, Type: u32, Fcb: *mut FCB, Ccb: *mut core::ffi::c_void, );
}
extern "C" {
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdIncrementReferenceCounts( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Count: u32, Flags: u32, );
}
extern "C" {
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "system" {
    fn CcInitializeCacheMap( FileObject: *mut FILE_OBJECT, FileSizes: *mut core::ffi::c_void, PinAccess: BOOLEAN, Callbacks: *mut core::ffi::c_void, LazyWriteContext: *mut core::ffi::c_void, );
}
extern "C" {
    fn FlagOn(Flags: u32, Flag: u32) -> u32;
}
extern "C" {
    fn CdInitializeDirContext( IrpContext: *mut IRP_CONTEXT, DirContext: *mut DIRENT_ENUM_CONTEXT, );
}
extern "C" {
    fn CdInitializeDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    // C header: CdLookupDirent(.., ULONG DirentOffset, ..). MUST be u32 — an i64
    // here pushes 8 bytes on i386, shifting DirContext to the always-zero high
    // dword → NULL Dirent → BUGCHECK 0x1E writing [0x4] in CdLookupDirent.
    fn CdLookupDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StreamOffset: u32, DirContext: *mut DIRENT_ENUM_CONTEXT, );
}
extern "C" {
    fn CdUpdateDirentFromRawDirent( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, DirContext: *const DIRENT_ENUM_CONTEXT, Dirent: *mut DIRENT, );
}
extern "C" {
    fn CdUpdateDirentName(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT, IgnoreCase: u32);
}
extern "C" {
    fn CdRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS) -> !;
}
extern "C" {
    fn BlockAlign(Vcb: *mut VCB, Length: u32) -> u32;
}
extern "system" {
    fn CcSetFileSizes( FileObject: *mut FILE_OBJECT, FileSizes: *mut core::ffi::c_void, );
}
extern "C" {
    // 3rd arg is StartingFileOffset: i64 in the def (allocsup.rs), NOT u32 — a u32 here
    // makes the i386 callee read 4 garbage stack bytes for the high dword → truncates at
    // a bogus file offset. (arg-width ABI bug, same class as CdAddInitialAllocation.)
    fn CdTruncateAllocation( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingFileOffset: i64, );
}
extern "C" {
    // Must match the definition (allocsup.rs): StartingBlock is u32, DataLength is i64.
    // The grind had these swapped (StartingOffset:i64, Length:u32) → on i386 the i64
    // DataLength read picked up 4 garbage stack bytes → Mcb ByteCount got a bogus high
    // dword (0x80000000…) → CdFindMcbEntry's `off < FileOffset+ByteCount` went negative →
    // entry "not found" → BUGCHECK 0x1E STATUS_DISK_CORRUPT_ERROR (allocsup.rs:412).
    fn CdAddInitialAllocation( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingBlock: u32, DataLength: i64, );
}
extern "C" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
}
extern "system" {
    fn CcPurgeCacheSection( SegmentObject: *mut core::ffi::c_void, FileOffset: *mut LARGE_INTEGER, Length: u32, UninitializeCacheMaps: BOOLEAN, ) -> BOOLEAN;
}
extern "C" {
    fn CdCleanupDirContext( IrpContext: *mut IRP_CONTEXT, DirContext: *mut DIRENT_ENUM_CONTEXT, );
}
extern "C" {
    fn CdCleanupDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    fn CdDecrementReferenceCounts( IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Count: u32, Flags: u32, );
}
extern "system" {
    fn ObDereferenceObject(Object: *mut core::ffi::c_void);
}
extern "C" {
    fn CdCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
}
extern "C" {
    fn ASSERT_EXCLUSIVE_VCB(Vcb: *mut VCB);
}
extern "C" {
    fn CdFspClose(Vcb: *mut VCB);
}
extern "C" {
    fn CdAcquireAllFiles(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdGetNextFcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, RestartKey: *mut *mut core::ffi::c_void) -> *mut FCB;
}
extern "C" {
    fn CdTeardownStructures(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, RemovedFcb: *mut BOOLEAN);
}
extern "C" {
    fn CdReleaseAllFiles(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn SafeNodeType(Fcb: *mut FCB) -> u32;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = 0x00030000; // CDFS_BUG_CHECK_CACHESUP

const CDFS_NTC_FCB_DATA: u32 = 0x0305; // nodetype.h
#[no_mangle]
pub unsafe extern "C" fn CdCreateInternalStream(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Fcb: *mut FCB,
    Name: *mut _UNICODE_STRING,
) {
    // Forward declarations for external symbols used in this function

    let mut StreamFile: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut DecrementReference: BOOLEAN = 0;
    let mut CleanupDirContext: BOOLEAN = 0;
    let mut UpdateFcbSizes: BOOLEAN = 0;
    let mut Dirent: DIRENT = core::mem::zeroed();
    let mut DirContext: DIRENT_ENUM_CONTEXT = core::mem::zeroed();

    // PAGED_CODE(); - ignored

    ASSERT_IRP_CONTEXT(IrpContext);
    ASSERT_FCB(Fcb);

    CdLockFcb(IrpContext, Fcb);

    if !(*fcb_index(Fcb)).FileObject.is_null() {
        CdUnlockFcb(IrpContext, Fcb);
        return;
    }

    let mut raise_status: NTSTATUS = 0;

    // _SEH2_TRY block emulated with a loop and break-to-finally
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler. Its own CdRaiseStatus calls are still deferred to raise_status below, which is equivalent.
    let p_CleanupDirContext = core::ptr::addr_of_mut!(CleanupDirContext);
    let p_DirContext = core::ptr::addr_of_mut!(DirContext);
    let p_Dirent = core::ptr::addr_of_mut!(Dirent);
    let p_StreamFile = core::ptr::addr_of_mut!(StreamFile);
    let p_DecrementReference = core::ptr::addr_of_mut!(DecrementReference);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            'seh_try: {
                loop {
                    StreamFile = IoCreateStreamFileObjectLite(
                        core::ptr::null_mut(),
                        (*(*Vcb).Vpb).RealDevice as *mut core::ffi::c_void,
                    );
                    if StreamFile.is_null() {
                        raise_status = STATUS_INSUFFICIENT_RESOURCES;
                        break;
                    }

                    (*StreamFile).ReadAccess = 1;
                    (*StreamFile).WriteAccess = 0;
                    (*StreamFile).DeleteAccess = 0;

                    (*StreamFile).SectionObjectPointer =
                        core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).SegmentObject);

                    CdSetFileObject(
                        IrpContext,
                        StreamFile,
                        StreamFileOpen as u32,
                        Fcb,
                        core::ptr::null_mut(),
                    );

                    (*StreamFile).FileName = *Name;

                    CdLockVcb(IrpContext, Vcb);
                    CdIncrementReferenceCounts(IrpContext, Fcb, 2, 0);
                    CdUnlockVcb(IrpContext, Vcb);
                    DecrementReference = 1;

                    CcInitializeCacheMap(
                        StreamFile,
                        core::ptr::addr_of_mut!((*fcb_header(Fcb)).AllocationSize) as *mut core::ffi::c_void,
                        1,
                        core::ptr::addr_of_mut!(CdData.CacheManagerCallbacks) as *mut core::ffi::c_void,
                        Fcb as *mut core::ffi::c_void,
                    );

                    (*fcb_index(Fcb)).FileObject = StreamFile;
                    StreamFile = core::ptr::null_mut();

                    if FlagOn((*Fcb).FcbState, FCB_STATE_INITIALIZED) == 0 {
                        let NewDataLength: u32;

                        CdInitializeDirContext(IrpContext, &mut DirContext);
                        CdInitializeDirent(IrpContext, &mut Dirent);
                        CleanupDirContext = 1;

                        CdLookupDirent(
                            IrpContext,
                            Fcb,
                            (*fcb_index(Fcb)).StreamOffset as u32,
                            &mut DirContext,
                        );

                        CdUpdateDirentFromRawDirent(
                            IrpContext,
                            Fcb,
                            &DirContext,
                            &mut Dirent,
                        );

                        CdUpdateDirentName(IrpContext, &mut Dirent, 0);

                        if Dirent.CdFileName.FileName.Buffer
                            != (&raw const CdUnicodeSelfArray).cast::<u16>() as *mut u16
                        {
                            raise_status = STATUS_FILE_CORRUPT_ERROR;
                            break;
                        }

                        NewDataLength = BlockAlign(
                            Vcb,
                            Dirent.DataLength + (*fcb_index(Fcb)).StreamOffset,
                        );

                        if NewDataLength == 0 {
                            raise_status = STATUS_FILE_CORRUPT_ERROR;
                            break;
                        }

                        if NewDataLength as i64 != (*fcb_header(Fcb)).FileSize.QuadPart {
                            (*fcb_header(Fcb)).AllocationSize.QuadPart = NewDataLength as i64;
                            (*fcb_header(Fcb)).FileSize.QuadPart = NewDataLength as i64;
                            (*fcb_header(Fcb)).ValidDataLength.QuadPart = NewDataLength as i64;

                            CcSetFileSizes(
                                (*fcb_index(Fcb)).FileObject,
                                core::ptr::addr_of_mut!((*fcb_header(Fcb)).AllocationSize) as *mut core::ffi::c_void,
                            );

                            CdTruncateAllocation(IrpContext, Fcb, 0);
                            CdAddInitialAllocation(
                                IrpContext,
                                Fcb,
                                Dirent.StartingOffset,        // StartingBlock: u32
                                NewDataLength as i64,         // DataLength: i64
                            );

                            UpdateFcbSizes = 1;
                        }

                        if FlagOn(Dirent.DirentFlags as u32, CD_ATTRIBUTE_HIDDEN) != 0 {
                            SetFlag(&mut (*Fcb).FileAttributes, FILE_ATTRIBUTE_HIDDEN);
                        }

                        CdConvertCdTimeToNtTime(
                            IrpContext,
                            Dirent.CdTime,
                            core::ptr::addr_of_mut!((*Fcb).CreationTime) as *mut LARGE_INTEGER,
                        );

                        SetFlag(&mut (*Fcb).FcbState, FCB_STATE_INITIALIZED);

                        CdCleanupDirContext(IrpContext, &mut DirContext);
                        CdCleanupDirent(IrpContext, &mut Dirent);
                        CleanupDirContext = 0;

                        if UpdateFcbSizes != 0 {
                            CcPurgeCacheSection(
                                &mut (*(*Fcb).FcbNonpaged).SegmentObject
                                    as *mut _ as *mut core::ffi::c_void,
                                core::ptr::null_mut(),
                                0,
                                0,
                            );
                        }
                    }

                    break; // normal completion → fall through to finally
                }
            }
        },
        |_, _| {
            if (*p_CleanupDirContext) != 0 {
                CdCleanupDirContext(IrpContext, &mut (*p_DirContext));
                CdCleanupDirent(IrpContext, &mut (*p_Dirent));
            }

            if !(*p_StreamFile).is_null() {
                (*(*p_StreamFile)).FileName.Buffer = core::ptr::null_mut();
                (*(*p_StreamFile)).FileName.MaximumLength = 0;
                (*(*p_StreamFile)).FileName.Length = 0;

                ObDereferenceObject((*p_StreamFile) as *mut core::ffi::c_void);
                (*fcb_index(Fcb)).FileObject = core::ptr::null_mut();
            }

            if (*p_DecrementReference) != 0 {
                CdLockVcb(IrpContext, Vcb);
                CdDecrementReferenceCounts(IrpContext, Fcb, 1, 0);
                CdUnlockVcb(IrpContext, Vcb);
            }

            CdUnlockFcb(IrpContext, Fcb);
        },
    );


    if raise_status != 0 {
        CdRaiseStatus(IrpContext, raise_status);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdDeleteInternalStream(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    // PAGED_CODE(); -- not translated (no behavioral effect)

    // UNREFERENCED_PARAMETER( IrpContext );
    let _ = IrpContext;

    ASSERT_IRP_CONTEXT(IrpContext);

    ASSERT_FCB(Fcb);

    CdLockFcb(IrpContext, Fcb);

    let FileObject: *mut FILE_OBJECT;
    FileObject = (*fcb_index(Fcb)).FileObject;
    (*fcb_index(Fcb)).FileObject = core::ptr::null_mut();

    CdUnlockFcb(IrpContext, Fcb);

    if !FileObject.is_null() {
        if !(*FileObject).PrivateCacheMap.is_null() {
            CcUninitializeCacheMap(FileObject, core::ptr::null_mut(), core::ptr::null_mut());
        }

        (*FileObject).FileName.Buffer = core::ptr::null_mut();
        (*FileObject).FileName.MaximumLength = 0;
        (*FileObject).FileName.Length = 0;

        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdCompleteMdl(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();
    let FileObject: PFILE_OBJECT;
    FileObject = (*IoGetCurrentIrpStackLocation(Irp)).FileObject;
    CcMdlReadComplete(FileObject, (*Irp).MdlAddress);
    (*Irp).MdlAddress = core::ptr::null_mut();
    CdCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CdPurgeVolume(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    DismountUnderway: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS;
    let mut RestartKey: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ThisFcb: *mut FCB = core::ptr::null_mut();
    // NextFcb is not initialized here; will be set inside loop
    let mut RemovedFcb: BOOLEAN = 0;

    unsafe {
        PAGED_CODE!();
        ASSERT_EXCLUSIVE_VCB(Vcb);
        CdFspClose(Vcb);
        CdAcquireAllFiles(IrpContext, Vcb);

        loop {
            CdLockVcb(IrpContext, Vcb);
            let NextFcb = CdGetNextFcb(IrpContext, Vcb, &raw mut RestartKey as *mut *mut core::ffi::c_void);

            if !NextFcb.is_null() {
                (*NextFcb).FcbReference += 1;
            }

            if !ThisFcb.is_null() {
                (*ThisFcb).FcbReference -= 1;
                CdUnlockVcb(IrpContext, Vcb);
                CdTeardownStructures(
                    IrpContext,
                    ThisFcb,
                    core::ptr::addr_of_mut!(RemovedFcb),
                );
            } else {
                CdUnlockVcb(IrpContext, Vcb);
            }

            if NextFcb.is_null() {
                break;
            }

            ThisFcb = NextFcb;

            let segment_ptr = core::ptr::addr_of_mut!((*(*ThisFcb).FcbNonpaged).SegmentObject);
            if !(*(*ThisFcb).FcbNonpaged).SegmentObject.ImageSectionObject.is_null() {
                MmFlushImageSection(segment_ptr, _MMFLUSH_TYPE_MmFlushForWrite);
            }

            if !(*(*ThisFcb).FcbNonpaged).SegmentObject.DataSectionObject.is_null()
                && CcPurgeCacheSection(
                    segment_ptr as *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                    0,
                    0, // FALSE
                ) == 0
                && Status == ntkrnl_sys::base::STATUS_SUCCESS
            {
                Status = STATUS_UNABLE_TO_DELETE_SECTION;
            }

            if DismountUnderway != 0
                && SafeNodeType(ThisFcb) != CDFS_NTC_FCB_DATA
                && !(*fcb_index(ThisFcb)).FileObject.is_null()
            {
                CdDeleteInternalStream(IrpContext, ThisFcb);
            }
        }

        if DismountUnderway != 0 {
            if !(*Vcb).PathTableFcb.is_null() {
                ThisFcb = (*Vcb).PathTableFcb;
                InterlockedIncrement(core::ptr::addr_of_mut!((*ThisFcb).FcbReference) as *mut i32);

                let segment_ptr = core::ptr::addr_of_mut!((*(*ThisFcb).FcbNonpaged).SegmentObject);
                if !(*(*ThisFcb).FcbNonpaged).SegmentObject.DataSectionObject.is_null()
                    && CcPurgeCacheSection(
                        segment_ptr as *mut core::ffi::c_void,
                        core::ptr::null_mut(),
                        0,
                        0,
                    ) == 0
                    && Status == ntkrnl_sys::base::STATUS_SUCCESS
                {
                    Status = STATUS_UNABLE_TO_DELETE_SECTION;
                }

                CdDeleteInternalStream(IrpContext, ThisFcb);

                InterlockedDecrement(core::ptr::addr_of_mut!((*ThisFcb).FcbReference) as *mut i32);
                CdTeardownStructures(
                    IrpContext,
                    ThisFcb,
                    core::ptr::addr_of_mut!(RemovedFcb),
                );
            }

            if !(*Vcb).VolumeDasdFcb.is_null() {
                ThisFcb = (*Vcb).VolumeDasdFcb;
                InterlockedIncrement(core::ptr::addr_of_mut!((*ThisFcb).FcbReference) as *mut i32);

                let segment_ptr = core::ptr::addr_of_mut!((*(*ThisFcb).FcbNonpaged).SegmentObject);
                if !(*(*ThisFcb).FcbNonpaged).SegmentObject.DataSectionObject.is_null()
                    && CcPurgeCacheSection(
                        segment_ptr as *mut core::ffi::c_void,
                        core::ptr::null_mut(),
                        0,
                        0,
                    ) == 0
                    && Status == ntkrnl_sys::base::STATUS_SUCCESS
                {
                    Status = STATUS_UNABLE_TO_DELETE_SECTION;
                }

                InterlockedDecrement(core::ptr::addr_of_mut!((*ThisFcb).FcbReference) as *mut i32);
                CdTeardownStructures(
                    IrpContext,
                    ThisFcb,
                    core::ptr::addr_of_mut!(RemovedFcb),
                );
            }
        }

        CdReleaseAllFiles(IrpContext, Vcb);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1990-2000 Microsoft Corporation
//
// Module Name:
//
//     Cache.c
//
// Abstract:
//
//     This module implements the cache management routines for the Cdfs
//     FSD and FSP, by calling the Common Cache Manager.
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_CACHESUP)
//
// //
// //  Local debug trace level
// //
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdCompleteMdl)
// #pragma alloc_text(PAGE, CdCreateInternalStream)
// #pragma alloc_text(PAGE, CdDeleteInternalStream)
// #pragma alloc_text(PAGE, CdPurgeVolume)
// #endif
//
//
// VOID
// CdCreateInternalStream (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFCB Fcb,
//      PUNICODE_STRING Name
//     )
//
// /*++
//
// Routine Description:
//
//     This function creates an internal stream file for interaction
//     with the cache manager.  The Fcb here can be for either a
//     directory stream or for a path table stream.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     Fcb - Points to the Fcb for this file.  It is either an Index or
//         Path Table Fcb.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFILE_OBJECT StreamFile = NULL;
//     BOOLEAN DecrementReference = FALSE;
//
//     BOOLEAN CleanupDirContext = FALSE;
//     BOOLEAN UpdateFcbSizes = FALSE;
//
//     DIRENT Dirent = {0};
//     DIRENT_ENUM_CONTEXT DirContext = {0};
//
//     PAGED_CODE();
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_FCB( Fcb );
//
//     //
//     //  We may only have the Fcb shared.  Lock the Fcb and do a
//     //  safe test to see if we need to really create the file object.
//     //
//
//     CdLockFcb( IrpContext, Fcb );
//
//     if (Fcb->FileObject != NULL) {
//
//         CdUnlockFcb( IrpContext, Fcb );
//         return;
//     }
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Create the internal stream.  The Vpb should be pointing at our volume
//         //  device object at this point.
//         //
//
//         StreamFile = IoCreateStreamFileObjectLite( NULL, Vcb->Vpb->RealDevice );
//
//         if (StreamFile == NULL) {
//
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         //
//         //  Initialize the fields of the file object.
//         //
//
//         StreamFile->ReadAccess = TRUE;
//         StreamFile->WriteAccess = FALSE;
//         StreamFile->DeleteAccess = FALSE;
//
//         StreamFile->SectionObjectPointer = &Fcb->FcbNonpaged->SegmentObject;
//
//         //
//         //  Set the file object type and increment the Vcb counts.
//         //
//
//         CdSetFileObject( IrpContext,
//                          StreamFile,
//                          StreamFileOpen,
//                          Fcb,
//                          NULL );
//
//         //
//         //  We'll give stream file objects a name to aid IO profiling etc. We
//         //  NULL this in CdDeleteInternalStream before OB deletes the file object,
//         //  and before CdRemovePrefix is called (which frees Fcb names).
//         //
//
//         StreamFile->FileName = *Name;
//
//         //
//         //  We will reference the current Fcb twice to keep it from going
//         //  away in the error path.  Otherwise if we dereference it
//         //  below in the finally clause a close could cause the Fcb to
//         //  be deallocated.
//         //
//
//         CdLockVcb( IrpContext, Vcb );
//         CdIncrementReferenceCounts( IrpContext, Fcb, 2, 0 );
//         CdUnlockVcb( IrpContext, Vcb );
//         DecrementReference = TRUE;
//
//         //
//         //  Initialize the cache map for the file.
//         //
//
//         CcInitializeCacheMap( StreamFile,
//                               (PCC_FILE_SIZES)&Fcb->AllocationSize,
//                               TRUE,
//                               &CdData.CacheManagerCallbacks,
//                               Fcb );
//
//         //
//         //  Go ahead and store the stream file into the Fcb.
//         //
//
//         Fcb->FileObject = StreamFile;
//         StreamFile = NULL;
//
//         //
//         //  If this is the first file object for a directory then we need to
//         //  read the self entry for this directory and update the sizes
//         //  in the Fcb.  We know that the Fcb has been initialized so
//         //  that we have a least one sector available to read.
//         //
//
//         if (!FlagOn( Fcb->FcbState, FCB_STATE_INITIALIZED )) {
//
//             ULONG NewDataLength;
//
//             //
//             //  Initialize the search structures.
//             //
//
//             CdInitializeDirContext( IrpContext, &DirContext );
//             CdInitializeDirent( IrpContext, &Dirent );
//             CleanupDirContext = TRUE;
//
//             //
//             //  Read the dirent from disk and transfer the data to the
//             //  in-memory dirent.
//             //
//
//             CdLookupDirent( IrpContext,
//                             Fcb,
//                             Fcb->StreamOffset,
//                             &DirContext );
//
//             CdUpdateDirentFromRawDirent( IrpContext, Fcb, &DirContext, &Dirent );
//
//             //
//             //  Verify that this really for the self entry.  We do this by
//             //  updating the name in the dirent and then checking that it matches
//             //  one of the hard coded names.
//             //
//
//             CdUpdateDirentName( IrpContext, &Dirent, FALSE );
//
//             if (Dirent.CdFileName.FileName.Buffer != CdUnicodeSelfArray) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             //
//             //  If the data sizes are different then update the header
//             //  and Mcb for this Fcb.
//             //
//
//             NewDataLength = BlockAlign( Vcb, Dirent.DataLength + Fcb->StreamOffset );
//
//             if (NewDataLength == 0) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             if (NewDataLength != Fcb->FileSize.QuadPart) {
//
//                 Fcb->AllocationSize.QuadPart =
//                 Fcb->FileSize.QuadPart =
//                 Fcb->ValidDataLength.QuadPart = NewDataLength;
//
//                 CcSetFileSizes( Fcb->FileObject, (PCC_FILE_SIZES) &Fcb->AllocationSize );
//
//                 CdTruncateAllocation( IrpContext, Fcb, 0 );
//                 CdAddInitialAllocation( IrpContext,
//                                         Fcb,
//                                         Dirent.StartingOffset,
//                                         NewDataLength );
//
//                 UpdateFcbSizes = TRUE;
//             }
//
//             //
//             //  Check for the existence flag and transform to hidden.
//             //
//
//             if (FlagOn( Dirent.DirentFlags, CD_ATTRIBUTE_HIDDEN )) {
//
//                 SetFlag( Fcb->FileAttributes, FILE_ATTRIBUTE_HIDDEN );
//             }
//
//             //
//             //  Convert the time to NT time.
//             //
//
//             CdConvertCdTimeToNtTime( IrpContext,
//                                      Dirent.CdTime,
//                                      (PLARGE_INTEGER) &Fcb->CreationTime );
//
//             //
//             //  Update the Fcb flags to indicate we have read the
//             //  self entry.
//             //
//
//             SetFlag( Fcb->FcbState, FCB_STATE_INITIALIZED );
//
//             //
//             //  If we updated the sizes then we want to purge the file.  Go
//             //  ahead and unpin and then purge the first page.
//             //
//
//             CdCleanupDirContext( IrpContext, &DirContext );
//             CdCleanupDirent( IrpContext, &Dirent );
//             CleanupDirContext = FALSE;
//
//             if (UpdateFcbSizes) {
//
//                 CcPurgeCacheSection( &Fcb->FcbNonpaged->SegmentObject,
//                                      NULL,
//                                      0,
//                                      FALSE );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Cleanup any dirent structures we may have used.
//         //
//
//         if (CleanupDirContext) {
//
//             CdCleanupDirContext( IrpContext, &DirContext );
//             CdCleanupDirent( IrpContext, &Dirent );
//         }
//
//         //
//         //  If we raised then we need to dereference the file object.
//         //
//
//         if (StreamFile != NULL) {
//
//             //
//             //  Null the name pointer, since the stream file object never actually
//             //  'owns' the names, we just point it to existing ones.
//             //
//
//             StreamFile->FileName.Buffer = NULL;
//             StreamFile->FileName.MaximumLength = StreamFile->FileName.Length = 0;
//
//             ObDereferenceObject( StreamFile );
//             Fcb->FileObject = NULL;
//         }
//
//         //
//         //  Dereference and unlock the Fcb.
//         //
//
//         if (DecrementReference) {
//
//             CdLockVcb( IrpContext, Vcb );
//             CdDecrementReferenceCounts( IrpContext, Fcb, 1, 0 );
//             CdUnlockVcb( IrpContext, Vcb );
//         }
//
//         CdUnlockFcb( IrpContext, Fcb );
//     } _SEH2_END;
//
//     return;
// }
//
//
// VOID
// CdDeleteInternalStream (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This function creates an internal stream file for interaction
//     with the cache manager.  The Fcb here can be for either a
//     directory stream or for a path table stream.
//
// Arguments:
//
//     Fcb - Points to the Fcb for this file.  It is either an Index or
//         Path Table Fcb.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFILE_OBJECT FileObject;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     ASSERT_IRP_CONTEXT( IrpContext );
//     ASSERT_FCB( Fcb );
//
//     //
//     //  Lock the Fcb.
//     //
//
//     CdLockFcb( IrpContext, Fcb );
//
//     //
//     //  Capture the file object.
//     //
//
//     FileObject = Fcb->FileObject;
//     Fcb->FileObject = NULL;
//
//     //
//     //  It is now safe to unlock the Fcb.
//     //
//
//     CdUnlockFcb( IrpContext, Fcb );
//
//     //
//     //  Dereference the file object if present.
//     //
//
//     if (FileObject != NULL) {
//
//         if (FileObject->PrivateCacheMap != NULL) {
//
//             CcUninitializeCacheMap( FileObject, NULL, NULL );
//         }
//
//         //
//         //  Null the name pointer, since the stream file object never actually
//         //  'owns' the names, we just point it to existing ones.
//         //
//
//         FileObject->FileName.Buffer = NULL;
//         FileObject->FileName.MaximumLength = FileObject->FileName.Length = 0;
//
//         ObDereferenceObject( FileObject );
//     }
// }
//
//
// NTSTATUS
// CdCompleteMdl (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the function of completing Mdl reads.
//     It should be called only from CdFsdRead.
//
// Arguments:
//
//     Irp - Supplies the originating Irp.
//
// Return Value:
//
//     NTSTATUS - Will always be STATUS_SUCCESS.
//
// --*/
//
// {
//     PFILE_OBJECT FileObject;
//
//     PAGED_CODE();
//
//     //
//     // Do completion processing.
//     //
//
//     FileObject = IoGetCurrentIrpStackLocation( Irp )->FileObject;
//
//     CcMdlReadComplete( FileObject, Irp->MdlAddress );
//
//     //
//     // Mdl is now deallocated.
//     //
//
//     Irp->MdlAddress = NULL;
//
//     //
//     // Complete the request and exit right away.
//     //
//
//     CdCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     return STATUS_SUCCESS;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// CdPurgeVolume (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      BOOLEAN DismountUnderway
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to purge the volume.  The purpose is to make all the stale file
//     objects in the system go away in order to lock the volume.
//
//     The Vcb is already acquired exclusively.  We will lock out all file operations by
//     acquiring the global file resource.  Then we will walk through all of the Fcb's and
//     perform the purge.
//
// Arguments:
//
//     Vcb - Vcb for the volume to purge.
//
//     DismountUnderway - Indicates that we are trying to delete all of the objects.
//         We will purge the Path Table and VolumeDasd and dereference all
//         internal streams.
//
// Return Value:
//
//     NTSTATUS - The first failure of the purge operation.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PVOID RestartKey = NULL;
//     PFCB ThisFcb = NULL;
//     PFCB NextFcb;
//
//     BOOLEAN RemovedFcb;
//
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_VCB( Vcb);
//
//     //
//     //  Force any remaining Fcb's in the delayed close queue to be closed.
//     //
//
//     CdFspClose( Vcb );
//
//     //
//     //  Acquire the global file resource.
//     //
//
//     CdAcquireAllFiles( IrpContext, Vcb );
//
//     //
//     //  Loop through each Fcb in the Fcb Table and perform the flush.
//     //
//
//     while (TRUE) {
//
//         //
//         //  Lock the Vcb to lookup the next Fcb.
//         //
//
//         CdLockVcb( IrpContext, Vcb );
//         NextFcb = CdGetNextFcb( IrpContext, Vcb, &RestartKey );
//
//         //
//         //  Reference the NextFcb if present.
//         //
//
//         if (NextFcb != NULL) {
//
//             NextFcb->FcbReference += 1;
//         }
//
//         //
//         //  If the last Fcb is present then decrement reference count and call teardown
//         //  to see if it should be removed.
//         //
//
//         if (ThisFcb != NULL) {
//
//             ThisFcb->FcbReference -= 1;
//
//             CdUnlockVcb( IrpContext, Vcb );
//
//             CdTeardownStructures( IrpContext, ThisFcb, &RemovedFcb );
//
//         } else {
//
//             CdUnlockVcb( IrpContext, Vcb );
//         }
//
//         //
//         //  Break out of the loop if no more Fcb's.
//         //
//
//         if (NextFcb == NULL) {
//
//             break;
//         }
//
//         //
//         //  Move to the next Fcb.
//         //
//
//         ThisFcb = NextFcb;
//
//         //
//         //  If there is a image section then see if that can be closed.
//         //
//
//         if (ThisFcb->FcbNonpaged->SegmentObject.ImageSectionObject != NULL) {
//
//             MmFlushImageSection( &ThisFcb->FcbNonpaged->SegmentObject, MmFlushForWrite );
//         }
//
//         //
//         //  If there is a data section then purge this.  If there is an image
//         //  section then we won't be able to.  Remember this if it is our first
//         //  error.
//         //
//
//         if ((ThisFcb->FcbNonpaged->SegmentObject.DataSectionObject != NULL) &&
//             !CcPurgeCacheSection( &ThisFcb->FcbNonpaged->SegmentObject,
//                                    NULL,
//                                    0,
//                                    FALSE ) &&
//             (Status == STATUS_SUCCESS)) {
//
//             Status = STATUS_UNABLE_TO_DELETE_SECTION;
//         }
//
//         //
//         //  Dereference the internal stream if dismounting.
//         //
//
//         if (DismountUnderway &&
//             (SafeNodeType( ThisFcb ) != CDFS_NTC_FCB_DATA) &&
//             (ThisFcb->FileObject != NULL)) {
//
//             CdDeleteInternalStream( IrpContext, ThisFcb );
//         }
//     }
//
//     //
//     //  Now look at the path table and volume Dasd Fcb's.
//     //
//
//     if (DismountUnderway) {
//
//         if (Vcb->PathTableFcb != NULL) {
//
//             ThisFcb = Vcb->PathTableFcb;
//             InterlockedIncrement( (LONG*)&Vcb->PathTableFcb->FcbReference );
//
//             if ((ThisFcb->FcbNonpaged->SegmentObject.DataSectionObject != NULL) &&
//                 !CcPurgeCacheSection( &ThisFcb->FcbNonpaged->SegmentObject,
//                                        NULL,
//                                        0,
//                                        FALSE ) &&
//                 (Status == STATUS_SUCCESS)) {
//
//                 Status = STATUS_UNABLE_TO_DELETE_SECTION;
//             }
//
//             CdDeleteInternalStream( IrpContext, ThisFcb );
//
//             InterlockedDecrement( (LONG*)&ThisFcb->FcbReference );
//
//             CdTeardownStructures( IrpContext, ThisFcb, &RemovedFcb );
//         }
//
//         if (Vcb->VolumeDasdFcb != NULL) {
//
//             ThisFcb = Vcb->VolumeDasdFcb;
//             InterlockedIncrement( (LONG*)&ThisFcb->FcbReference );
//
//             if ((ThisFcb->FcbNonpaged->SegmentObject.DataSectionObject != NULL) &&
//                 !CcPurgeCacheSection( &ThisFcb->FcbNonpaged->SegmentObject,
//                                        NULL,
//                                        0,
//                                        FALSE ) &&
//                 (Status == STATUS_SUCCESS)) {
//
//                 Status = STATUS_UNABLE_TO_DELETE_SECTION;
//             }
//
//             InterlockedDecrement( (LONG*)&ThisFcb->FcbReference );
//
//             CdTeardownStructures( IrpContext, ThisFcb, &RemovedFcb );
//         }
//     }
//
//     //
//     //  Release all of the files.
//     //
//
//     CdReleaseAllFiles( IrpContext, Vcb );
//
//     return Status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
