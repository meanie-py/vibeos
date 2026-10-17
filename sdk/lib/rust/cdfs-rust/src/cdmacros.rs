//! cdmacros.rs — Rust shims for cdfs C-header MACROS that the translation
//! pipeline emitted as `extern` CALLS but never defined.
//!
//! Each macro from cdprocs.h / cdstruc.h / fsrtlfuncs.h is reproduced here as a
//! `#[no_mangle] pub unsafe extern "system"` function. The ABI MUST be
//! `extern "system"` because every caller declares these `extern "system"`; a
//! mismatch yields an LLVM `ud2` / STATUS_ILLEGAL_INSTRUCTION at runtime.
//!
//! Faithfully translated from the C macro bodies. Field names verified against
//! `src/types_i386.rs` (bindgen substrate) and existing access patterns in the
//! sibling modules.
#![allow(non_snake_case, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;

use ntkrnl_sys::ddk::{
    ExAcquireFastMutex, ExReleaseFastMutex, ExReleaseResourceLite, IoSetTopLevelIrp,
    PsGetCurrentThread,
    CcUnpinData, ExAllocatePoolWithTag,
    RtlDeleteElementGenericTable, RtlTimeFieldsToTime,
    PDEVICE_OBJECT, PERESOURCE, PFAST_MUTEX, PFILE_LOCK, PFILE_OBJECT, PIO_STATUS_BLOCK,
    PLARGE_INTEGER, PEPROCESS, PIRP, POPLOCK, PRTL_GENERIC_TABLE, PVOID,
    FSRTL_ADVANCED_FCB_HEADER, LARGE_INTEGER, LIST_ENTRY, SINGLE_LIST_ENTRY, TIME_FIELDS,
    _FAST_IO_POSSIBLE_FastIoIsNotPossible, _FAST_IO_POSSIBLE_FastIoIsPossible,
    _FAST_IO_POSSIBLE_FastIoIsQuestionable,
};
use ntkrnl_sys::base::{BOOLEAN, ULONG};
use ntkrnl_sys::rtl::RtlZeroMemory;
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlOplockIsFastIoPossible};

use crate::types::{
    BYTE_COUNT_8_DOT_3, CCB, COMPOUND_PATH_ENTRY, DIRENT, DIRENT_ENUM_CONTEXT, DIRENT_FLAG_ALLOC_BUFFER,
    FCB, FCB_DATA, FCB_INDEX, FCB_NONPAGED, FID_DIR_MASK, FILE_ENUM_CONTEXT, FILE_ID, IRP_CONTEXT,
    PATH_ENTRY_FLAG_ALLOC_BUFFER, PFCB, RAW_HSG_VD, RAW_ISO_VD, SECTOR_SIZE, VCB,
};

// CdAcquireResource is a real Rust fn in resrcsup.rs (cdecl).
use crate::resrcsup::CdAcquireResource;

// CdData lives in cddata.rs as a `pub static mut CD_DATA`.
use crate::cddata::CdData;

// CdGetFcbOplock / CdFreePool are crate-internal helpers defined in
// ddk_shims.rs (extern "C"). FILE_LOCK->FastIoIsQuestionable etc. reachable here.
// CdUnpinData (cdecl) is DEFINED below in this file (it was an undefined macro).
extern "C" {
    fn CdGetFcbOplock(Fcb: *mut FCB) -> POPLOCK;
}
extern "C" {
    fn CdFreePool(Pool: *mut *mut c_void);
}

// FsRtlPrivateLock is a real ntoskrnl export. ntoskrnl.spec:
//   @ stdcall FsRtlPrivateLock(ptr ptr ptr ptr ptr long long long ptr ptr ptr long)
// fsrtlfuncs.h prototype (NTAPI, 12 args).
extern "system" {
    fn FsRtlPrivateLock(
        FileLock: PFILE_LOCK,
        FileObject: PFILE_OBJECT,
        FileOffset: PLARGE_INTEGER,
        Length: PLARGE_INTEGER,
        Process: PEPROCESS,
        Key: ULONG,
        FailImmediately: BOOLEAN,
        ExclusiveLock: BOOLEAN,
        IoStatus: PIO_STATUS_BLOCK,
        Irp: PIRP,
        Context: *mut c_void,
        AlreadySynchronized: BOOLEAN,
    ) -> BOOLEAN;
}

// ---------------------------------------------------------------------------
// cdprocs.h
// ---------------------------------------------------------------------------

// CdLockFcb(IC,F):
//   PVOID t = PsGetCurrentThread();
//   if (t != F->FcbLockThread) { ExAcquireFastMutex(&F->FcbNonpaged->FcbMutex);
//                                F->FcbLockThread = t; }
//   F->FcbLockCount += 1;
#[no_mangle]
pub unsafe extern "C" fn CdLockFcb(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    let t = PsGetCurrentThread() as *mut c_void;
    if t != (*Fcb).FcbLockThread {
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).FcbMutex) as PFAST_MUTEX);
        (*Fcb).FcbLockThread = t;
    }
    (*Fcb).FcbLockCount += 1;
}

// CdUnlockFcb(IC,F):
//   F->FcbLockCount -= 1;
//   if (F->FcbLockCount == 0) { F->FcbLockThread = NULL;
//                               ExReleaseFastMutex(&F->FcbNonpaged->FcbMutex); }
#[no_mangle]
pub unsafe extern "C" fn CdUnlockFcb(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    (*Fcb).FcbLockCount -= 1;
    if (*Fcb).FcbLockCount == 0 {
        (*Fcb).FcbLockThread = null_mut();
        ExReleaseFastMutex(core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).FcbMutex) as PFAST_MUTEX);
    }
}

// CdReleaseFile(IC,F): ExReleaseResourceLite( F->Resource )
//   F->Resource is the PERESOURCE in the FSRTL advanced header spliced as
//   __bindgen_anon_1.Header (see ddk_shims::CdAcquireFileShared).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseFile(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    let resource = (*Fcb).__bindgen_anon_1.Header.as_ref().Resource as PERESOURCE;
    ExReleaseResourceLite(resource);
}

// CdLockCdData(): ExAcquireFastMutex(&CdData.CdDataMutex);
//                 CdData.CdDataLockThread = PsGetCurrentThread();   [zero args -> @0]
#[no_mangle]
pub unsafe extern "C" fn CdLockCdData() {
    ExAcquireFastMutex(core::ptr::addr_of_mut!(CdData.CdDataMutex) as PFAST_MUTEX);
    CdData.CdDataLockThread = PsGetCurrentThread() as *mut c_void;
}

// CdIsFastIoPossible(F) -> BOOLEAN:
//   if ((F->Vcb->VcbCondition != VcbMounted) ||
//       !FsRtlOplockIsFastIoPossible(CdGetFcbOplock(F)))   return FastIoIsNotPossible;
//   else if ((F->FileLock != NULL) && FsRtlAreThereCurrentFileLocks(F->FileLock))
//                                                          return FastIoIsQuestionable;
//   else                                                   return FastIoIsPossible;
// VcbCondition VcbMounted is the 3rd VCB_CONDITION enum value (== 2); see fsctrl.rs.
const VCB_CONDITION_VcbMounted: u32 = 2;
#[no_mangle]
// Returns u32 (not BOOLEAN/u8): callers declare this `-> u32`/`-> i32` and test the
// full eax; a u8 return leaves the upper eax bytes garbage → false branch-taken.
// (i386 return-width hazard — see FlagOn note in ddk_shims.rs.)
pub unsafe extern "C" fn CdIsFastIoPossible(Fcb: *mut FCB) -> u32 {
    if (*(*Fcb).Vcb).VcbCondition != VCB_CONDITION_VcbMounted
        || FsRtlOplockIsFastIoPossible(CdGetFcbOplock(Fcb)) == 0
    {
        return _FAST_IO_POSSIBLE_FastIoIsNotPossible as u32;
    }

    // F->FileLock lives in the FCB_DATA arm of the FCB anonymous union (__bindgen_anon_2).
    let fcb_data = core::ptr::addr_of!((*Fcb).__bindgen_anon_2) as *const FCB_DATA;
    let file_lock = (*fcb_data).FileLock;
    if !file_lock.is_null() && FsRtlAreThereCurrentFileLocks(file_lock as PFILE_LOCK) != 0 {
        return _FAST_IO_POSSIBLE_FastIoIsQuestionable as u32;
    }

    _FAST_IO_POSSIBLE_FastIoIsPossible as u32
}

// CdFreeIoContext(IO): CdFreePool( (PVOID)&(IO) )
//   The C macro takes the address of the caller's IO lvalue; the Rust callers
//   pass the IO-context pointer BY VALUE (one pointer arg). Free that block by
//   handing CdFreePool the address of our local copy. Matches one-pointer ABI.
#[no_mangle]
pub unsafe extern "C" fn CdFreeIoContext(IoContext: *mut c_void) {
    let mut local: *mut c_void = IoContext;
    CdFreePool(core::ptr::addr_of_mut!(local));
}

// CdRestoreThreadContext(IC):
//   IC->ThreadContext->Cdfs = 0;
//   IoSetTopLevelIrp(IC->ThreadContext->SavedTopLevelIrp);
//   IC->ThreadContext = NULL;
#[no_mangle]
pub unsafe extern "C" fn CdRestoreThreadContext(IrpContext: *mut IRP_CONTEXT) {
    let tc = (*IrpContext).ThreadContext;
    (*tc).Cdfs = 0;
    IoSetTopLevelIrp((*tc).SavedTopLevelIrp);
    (*IrpContext).ThreadContext = null_mut();
}

// CdInitializeDirent(IC,D): RtlZeroMemory(D, sizeof(DIRENT))
#[no_mangle]
pub unsafe extern "C" fn CdInitializeDirent(_IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT) {
    RtlZeroMemory(Dirent as *mut c_void, core::mem::size_of::<DIRENT>() as _);
}

// CdInitializeDirContext(IC,DC): RtlZeroMemory(DC, sizeof(DIRENT_ENUM_CONTEXT))
#[no_mangle]
pub unsafe extern "C" fn CdInitializeDirContext(
    _IrpContext: *mut IRP_CONTEXT,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
) {
    RtlZeroMemory(
        DirContext as *mut c_void,
        core::mem::size_of::<DIRENT_ENUM_CONTEXT>() as _,
    );
}

// CdCleanupDirent(IC,D):
//   if (FlagOn(D->Flags, DIRENT_FLAG_ALLOC_BUFFER))
//       CdFreePool(&D->CdFileName.FileName.Buffer);
#[no_mangle]
pub unsafe extern "C" fn CdCleanupDirent(_IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT) {
    if ((*Dirent).Flags as u32 & DIRENT_FLAG_ALLOC_BUFFER) != 0 {
        CdFreePool(
            core::ptr::addr_of_mut!((*Dirent).CdFileName.FileName.Buffer) as *mut *mut c_void,
        );
    }
}

// CdCleanupDirContext(IC,DC): CdUnpinData(IC, &DC->Bcb)
#[no_mangle]
pub unsafe extern "C" fn CdCleanupDirContext(
    IrpContext: *mut IRP_CONTEXT,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
) {
    CdUnpinData(
        IrpContext,
        core::ptr::addr_of_mut!((*DirContext).Bcb) as *mut *mut c_void,
    );
}

// CdInitializeFileContext(IC,FC):
//   RtlZeroMemory(FC, sizeof(FILE_ENUM_CONTEXT));
//   FC->PriorDirent   = &FC->Dirents[0];
//   FC->InitialDirent = &FC->Dirents[1];
//   FC->CurrentDirent = &FC->Dirents[2];
//   FC->ShortName.FileName.MaximumLength = BYTE_COUNT_8_DOT_3;
//   FC->ShortName.FileName.Buffer        = FC->ShortNameBuffer;
#[no_mangle]
pub unsafe extern "C" fn CdInitializeFileContext(
    _IrpContext: *mut IRP_CONTEXT,
    Context: *mut FILE_ENUM_CONTEXT,
) {
    RtlZeroMemory(
        Context as *mut c_void,
        core::mem::size_of::<FILE_ENUM_CONTEXT>() as _,
    );
    (*Context).PriorDirent = core::ptr::addr_of_mut!((*Context).Dirents[0]);
    (*Context).InitialDirent = core::ptr::addr_of_mut!((*Context).Dirents[1]);
    (*Context).CurrentDirent = core::ptr::addr_of_mut!((*Context).Dirents[2]);
    (*Context).ShortName.FileName.MaximumLength = BYTE_COUNT_8_DOT_3 as u16;
    (*Context).ShortName.FileName.Buffer =
        core::ptr::addr_of_mut!((*Context).ShortNameBuffer[0]);
}

// CdInitializeCompoundPathEntry(IC,CP): RtlZeroMemory(CP, sizeof(COMPOUND_PATH_ENTRY))
#[no_mangle]
pub unsafe extern "C" fn CdInitializeCompoundPathEntry(
    _IrpContext: *mut IRP_CONTEXT,
    Entry: *mut COMPOUND_PATH_ENTRY,
) {
    RtlZeroMemory(
        Entry as *mut c_void,
        core::mem::size_of::<COMPOUND_PATH_ENTRY>() as _,
    );
}

// CdCleanupCompoundPathEntry(IC,CP):
//   CdUnpinData(IC, &CP->PathContext.Bcb);
//   if (CP->PathContext.AllocatedData) CdFreePool(&CP->PathContext.Data);
//   if (FlagOn(CP->PathEntry.Flags, PATH_ENTRY_FLAG_ALLOC_BUFFER))
//       CdFreePool(&CP->PathEntry.CdDirName.FileName.Buffer);
#[no_mangle]
pub unsafe extern "C" fn CdCleanupCompoundPathEntry(
    IrpContext: *mut IRP_CONTEXT,
    Entry: *mut COMPOUND_PATH_ENTRY,
) {
    CdUnpinData(
        IrpContext,
        core::ptr::addr_of_mut!((*Entry).PathContext.Bcb) as *mut *mut c_void,
    );
    if (*Entry).PathContext.AllocatedData != 0 {
        CdFreePool(
            core::ptr::addr_of_mut!((*Entry).PathContext.Data) as *mut *mut c_void,
        );
    }
    if ((*Entry).PathEntry.Flags & PATH_ENTRY_FLAG_ALLOC_BUFFER) != 0 {
        CdFreePool(
            core::ptr::addr_of_mut!((*Entry).PathEntry.CdDirName.FileName.Buffer)
                as *mut *mut c_void,
        );
    }
}

// ---------------------------------------------------------------------------
// cdstruc.h — FILE_ID accessors (FILE_ID == LARGE_INTEGER; .LowPart / .HighPart
// live in the bindgen anonymous struct member __bindgen_anon_1).
// ---------------------------------------------------------------------------

// CdQueryFidDirentOffset(I) -> ULONG:  (I).LowPart & ~FID_DIR_MASK
#[no_mangle]
pub unsafe extern "C" fn CdQueryFidDirentOffset(FileId: FILE_ID) -> u32 {
    FileId.__bindgen_anon_1.LowPart & !FID_DIR_MASK
}

// CdQueryFidPathTableOffset(I) -> ULONG:  (I).HighPart
#[no_mangle]
pub unsafe extern "C" fn CdQueryFidPathTableOffset(FileId: FILE_ID) -> u32 {
    FileId.__bindgen_anon_1.HighPart as u32
}

// CdFidIsDirectory(I) -> BOOLEAN:  FlagOn((I).LowPart, FID_DIR_MASK)
#[no_mangle]
pub unsafe extern "C" fn CdFidIsDirectory(FileId: FILE_ID) -> BOOLEAN {
    if (FileId.__bindgen_anon_1.LowPart & FID_DIR_MASK) != 0 {
        1
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// cdprocs.h — #define SectorTruncate(L) ((ULONG)(L) & ~(SECTOR_SIZE-1))
// Yields ULONG -> return u32 (per task: use the type the C macro actually yields).
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn SectorTruncate(L: u32) -> u32 {
    L & !(SECTOR_SIZE - 1)
}

// ---------------------------------------------------------------------------
// fsrtlfuncs.h MACROS
// ---------------------------------------------------------------------------

// FsRtlAreThereCurrentFileLocks(FL) -> BOOLEAN:  ((FL)->FastIoIsQuestionable)
#[no_mangle]
pub unsafe extern "C" fn FsRtlAreThereCurrentFileLocks(FileLock: PFILE_LOCK) -> BOOLEAN {
    (*FileLock).FastIoIsQuestionable
}

// FsRtlFastLock(A1..A11) -> BOOLEAN:
//   FsRtlPrivateLock(A1,A2,A3,A4,A5,A6,A7,A8,A9, NULL, A10, A11)
//   i.e. insert NULL as the 10th arg (Irp) before Context (A10) and
//   AlreadySynchronized (A11).
#[no_mangle]
pub unsafe extern "C" fn FsRtlFastLock(
    FileLock: PFILE_LOCK,
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: PLARGE_INTEGER,
    ProcessId: PEPROCESS,
    Key: ULONG,
    FailImmediately: BOOLEAN,
    ExclusiveLock: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
    Wait: BOOLEAN,
) -> BOOLEAN {
    FsRtlPrivateLock(
        FileLock,
        FileObject,
        FileOffset,
        Length,
        ProcessId,
        Key,
        FailImmediately,
        ExclusiveLock,
        IoStatus,
        null_mut(),               // Irp = NULL
        DeviceObject as *mut c_void, // Context = A10 (DeviceObject)
        Wait,                     // AlreadySynchronized = A11 (Wait)
    )
}

// ===========================================================================
// Batch 2 — remaining undefined cdfs macros / inlines.
// ABI per i386 link decoration:
//   _Name@N  -> stdcall -> extern "system"
//   _Name    -> cdecl   -> extern "C"
// ===========================================================================

// cdprocs.h:
//   #define CdUnpinData(IC,B)  if (*(B) != NULL) { CcUnpinData(*(B)); *(B) = NULL; }
// Link wants the CDECL form (_CdUnpinData, no @N). Caller decl is in cdmacros's
// own (now removed) extern "C" import + dirsup/pathsup extern "C" decls. The
// `B` arg is a PBCB* (pointer-to-Bcb-pointer).
#[no_mangle]
pub unsafe extern "C" fn CdUnpinData(_IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut c_void) {
    if !(*Bcb).is_null() {
        CcUnpinData(*Bcb as PVOID);
        *Bcb = null_mut();
    }
}

// cdprocs.h:
//   #define CdUnlockCdData()
//       CdData.CdDataLockThread = NULL;
//       ExReleaseFastMutex(&CdData.CdDataMutex)
// stdcall, zero args -> @0.
#[no_mangle]
pub unsafe extern "C" fn CdUnlockCdData() {
    CdData.CdDataLockThread = null_mut();
    ExReleaseFastMutex(core::ptr::addr_of_mut!(CdData.CdDataMutex) as PFAST_MUTEX);
}

// wdm.h singly-linked list push:
//   Entry->Next = ListHead->Next; ListHead->Next = Entry;
// stdcall (@8). Caller types: PSINGLE_LIST_ENTRY.
#[no_mangle]
pub unsafe extern "C" fn PushEntryList(
    ListHead: *mut SINGLE_LIST_ENTRY,
    Entry: *mut SINGLE_LIST_ENTRY,
) {
    (*Entry).Next = (*ListHead).Next;
    (*ListHead).Next = Entry;
}

// wdm.h doubly-linked tail insert:
//   Blink = ListHead->Blink; Entry->Flink = ListHead; Entry->Blink = Blink;
//   Blink->Flink = Entry; ListHead->Blink = Entry;
// stdcall (@8). Caller types: PLIST_ENTRY.
#[no_mangle]
pub unsafe extern "C" fn InsertTailList(
    ListHead: *mut LIST_ENTRY,
    Entry: *mut LIST_ENTRY,
) {
    let blink = (*ListHead).Blink;
    (*Entry).Flink = ListHead;
    (*Entry).Blink = blink;
    (*blink).Flink = Entry;
    (*ListHead).Blink = Entry;
}

// cdprocs.h:
//   #define BytesFromBlocks(V,B)  ((ULONG)(B) << (V)->BlockToByteShift)
// stdcall (@8). -> ULONG (u32).
#[no_mangle]
pub unsafe extern "C" fn BytesFromBlocks(Vcb: *mut VCB, BlockOffset: u32) -> u32 {
    BlockOffset << (*Vcb).BlockToByteShift
}

// cdprocs.h:
//   #define LlBlockAlign(V,L)
//       ((LONGLONG)(L) + (V)->BlockMask) & (LONGLONG)((LONG)(V)->BlockInverseMask)
// stdcall (@12, i64 by-value = 8 bytes + 4-byte ptr). -> LONGLONG (i64).
#[no_mangle]
pub unsafe extern "C" fn LlBlockAlign(Vcb: *mut VCB, FileSize: i64) -> i64 {
    // BlockInverseMask is ULONG; cast to LONG (i32) then sign-extend to LONGLONG.
    let inv = (*Vcb).BlockInverseMask as i32 as i64;
    (FileSize + (*Vcb).BlockMask as i64) & inv
}

// cdprocs.h:
//   #define SectorBlockOffset(V,LB)  ((ULONG)(LB) & ((V)->BlocksPerSector - 1))
// Link wants the stdcall form (_SectorBlockOffset@8). -> ULONG (u32).
#[no_mangle]
pub unsafe extern "C" fn SectorBlockOffset(Vcb: *mut VCB, DiskOffset: u32) -> u32 {
    DiskOffset & ((*Vcb).BlocksPerSector - 1)
}

// nodetype.h:
//   #define SafeNodeType(Ptr)  (*((PNODE_TYPE_CODE)(Ptr)))
// NODE_TYPE_CODE == USHORT. Callers declare -> u32; read the leading u16 and
// zero-extend. stdcall (@4) — single 4-byte pointer arg regardless of pointee.
#[no_mangle]
pub unsafe extern "C" fn SafeNodeType(Node: *mut c_void) -> u32 {
    *(Node as *const u16) as u32
}

// cd.h fn-macro CdConvertCdTimeToNtTime(IC,CD,NT). stdcall (@12).
// Faithful translation (mirrors cachesup.rs's private inline copy).
const CD_BASE_YEAR: i32 = 1900;
const VCB_STATE_HSG: u32 = 0x00000001;
const GMT_OFFSET_TO_NT: i64 = 15 * 60 * 1000 * 1000 * 10;
#[no_mangle]
pub unsafe extern "C" fn CdConvertCdTimeToNtTime(
    IrpContext: *mut IRP_CONTEXT,
    CdTime: *mut i8,
    NtTime: *mut LARGE_INTEGER,
) {
    let cd = CdTime;
    let mut tf: TIME_FIELDS = core::mem::zeroed();
    tf.Year = (*cd.add(0) as i32 + CD_BASE_YEAR) as i16;
    tf.Month = *cd.add(1) as i16;
    tf.Day = *cd.add(2) as i16;
    tf.Hour = *cd.add(3) as i16;
    tf.Minute = *cd.add(4) as i16;
    tf.Second = *cd.add(5) as i16;
    tf.Milliseconds = 0;
    RtlTimeFieldsToTime(&mut tf, NtTime);
    let gmt = *cd.add(6);
    if ((*(*IrpContext).Vcb).VcbState & VCB_STATE_HSG) == 0
        && gmt != 0
        && gmt >= -48
        && gmt <= 52
    {
        (*NtTime).QuadPart += -(gmt as i64) * GMT_OFFSET_TO_NT;
    }
}

// cdprocs.h:
//   #define CdIncrementReferenceCounts(IC,F,C,UC) {
//       (F)->FcbReference     += (C);
//       (F)->FcbUserReference += (UC);
//       (F)->Vcb->VcbReference     += (C);
//       (F)->Vcb->VcbUserReference += (UC); }
// stdcall (@16). Counts are LONG; callers pass u32 — add as the field types.
#[no_mangle]
pub unsafe extern "C" fn CdIncrementReferenceCounts(
    _IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Count: u32,
    UserCount: u32,
) {
    let c = Count as i32;
    let uc = UserCount as i32;
    (*Fcb).FcbReference += c;
    (*Fcb).FcbUserReference = (*Fcb).FcbUserReference.wrapping_add(uc as u32);
    let vcb = (*Fcb).Vcb;
    (*vcb).VcbReference += c;
    (*vcb).VcbUserReference += uc;
}

// cdprocs.h:
//   #define CdUpdateVcbCondition(V,C)  (V)->VcbCondition = (C)
// stdcall (@8).  VcbCondition is the VCB_CONDITION enum (repr i32/u32).
#[no_mangle]
pub unsafe extern "C" fn CdUpdateVcbCondition(Vcb: *mut VCB, Condition: u32) {
    (*Vcb).VcbCondition = Condition as _;
}

// cdprocs.h:
//   #define CdUpdateMediaChangeCount(V,C)  (V)->MediaChangeCount = (C)
// stdcall (@8).  MediaChangeCount is ULONG.
#[no_mangle]
pub unsafe extern "C" fn CdUpdateMediaChangeCount(Vcb: *mut VCB, MediaChangeCount: u32) {
    (*Vcb).MediaChangeCount = MediaChangeCount;
}

// ---------------------------------------------------------------------------
// strucsup.c allocator fn-macros. Pool/tag/size constants mirror strucsup.rs.
// ---------------------------------------------------------------------------
const CD_PAGED_POOL: u32 = 1; // PagedPool
// NonPagedPool = 0. ReactOS ntoskrnl's ExAllocatePoolWithTag does NOT accept the
// Win8+ NonPagedPoolNx (512) value — passing 512 makes it return a garbage pointer.
// cd.h's CdNonPagedPool nominally maps to NonPagedPoolNx but ReactOS lowers it to
// NonPagedPool at the 0x502 profile (see glue.rs CdNonPagedPool=_POOL_TYPE_NonPagedPool).
const CD_NONPAGED_POOL: u32 = 0; // NonPagedPool
const TAG_CCB: u32 = 0x63636443; // nolint: const-contradiction ('ccdC' -- cdfs/cdprocs.h:85; fastfat's is 'CtaF')
const TAG_FCB_DATA: u32 = 0x64666443; // 'dfdC'
const TAG_FCB_INDEX: u32 = 0x69666443; // 'ifdC'
const TAG_FCB_NONPAGED_LOCAL: u32 = 0x6e666443; // 'nfdC'

// SIZEOF_FCB_DATA / SIZEOF_FCB_INDEX = FIELD_OFFSET(FCB,FcbType) + sizeof(FCB_*).
// FcbType is the first member of the FCB anon union arm (__bindgen_anon_2).
const SIZEOF_FCB_DATA: u32 =
    (core::mem::offset_of!(FCB, __bindgen_anon_2) + core::mem::size_of::<FCB_DATA>()) as u32;
const SIZEOF_FCB_INDEX: u32 =
    (core::mem::offset_of!(FCB, __bindgen_anon_2) + core::mem::size_of::<FCB_INDEX>()) as u32;

// #define CdAllocateFcbData(IC) FsRtlAllocatePoolWithTag(CdPagedPool,SIZEOF_FCB_DATA,TAG_FCB_DATA)
// Link wants CDECL (_CdAllocateFcbData, no @N).
#[no_mangle]
pub unsafe extern "C" fn CdAllocateFcbData(_IrpContext: *mut IRP_CONTEXT) -> *mut FCB {
    FsRtlAllocatePoolWithTag(CD_PAGED_POOL, SIZEOF_FCB_DATA, TAG_FCB_DATA) as *mut FCB
}

// #define CdAllocateFcbIndex(IC) FsRtlAllocatePoolWithTag(CdPagedPool,SIZEOF_FCB_INDEX,TAG_FCB_INDEX)
// Link wants CDECL (_CdAllocateFcbIndex, no @N).
#[no_mangle]
pub unsafe extern "C" fn CdAllocateFcbIndex(_IrpContext: *mut IRP_CONTEXT) -> *mut FCB {
    FsRtlAllocatePoolWithTag(CD_PAGED_POOL, SIZEOF_FCB_INDEX, TAG_FCB_INDEX) as *mut FCB
}

// #define CdAllocateFcbNonpaged(IC) ExAllocatePoolWithTag(CdNonPagedPool,sizeof(FCB_NONPAGED),TAG_FCB_NONPAGED)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdAllocateFcbNonpaged(
    _IrpContext: *mut IRP_CONTEXT,
) -> *mut FCB_NONPAGED {
    ExAllocatePoolWithTag(
        CD_NONPAGED_POOL as _,
        core::mem::size_of::<FCB_NONPAGED>() as _,
        TAG_FCB_NONPAGED_LOCAL,
    ) as *mut FCB_NONPAGED
}

// #define CdAllocateCcb(IC) FsRtlAllocatePoolWithTag(CdPagedPool,sizeof(CCB),TAG_CCB)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdAllocateCcb(_IrpContext: *mut IRP_CONTEXT) -> *mut CCB {
    FsRtlAllocatePoolWithTag(CD_PAGED_POOL, core::mem::size_of::<CCB>() as u32, TAG_CCB)
        as *mut CCB
}

// #define CdDeallocateFcbData(IC,F)  CdFreePool(&(F))
// stdcall (@8). The caller passes the Fcb pointer by value; free the block by
// handing CdFreePool the address of our local copy (matches CdFreeIoContext).
#[no_mangle]
pub unsafe extern "C" fn CdDeallocateFcbData(
    _IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut c_void,
) {
    let mut local: *mut c_void = Fcb;
    CdFreePool(core::ptr::addr_of_mut!(local));
}

// #define CdDeallocateFcbIndex(IC,F)  CdFreePool(&(F))
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdDeallocateFcbIndex(
    _IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut c_void,
) {
    let mut local: *mut c_void = Fcb;
    CdFreePool(core::ptr::addr_of_mut!(local));
}

// ---------------------------------------------------------------------------
// fsrtlfuncs.h inline FsRtlSetupAdvancedHeader(AdvHdr, FMutex).
// Link wants CDECL (_FsRtlSetupAdvancedHeader, no @N).
// i386 pre-Vista layout: no PushLock / FileContextSupportPointer fields; Version
// is the 4-bit bitfield (set to FSRTL_FCB_HEADER_V0 == 0).
// ---------------------------------------------------------------------------
const FSRTL_FLAG_ADVANCED_HEADER: u8 = 0x40;
const FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS: u8 = 0x02;
#[no_mangle]
pub unsafe extern "C" fn FsRtlSetupAdvancedHeader(
    AdvHdr: *mut FSRTL_ADVANCED_FCB_HEADER,
    FastMutex: PFAST_MUTEX,
) {
    (*AdvHdr).Flags |= FSRTL_FLAG_ADVANCED_HEADER;
    (*AdvHdr).Flags2 |= FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS;
    // Version = FSRTL_FCB_HEADER_V0 (0) via the bindgen bitfield setter.
    FSRTL_ADVANCED_FCB_HEADER::set_Version_raw(AdvHdr, 0);
    // InitializeListHead(&AdvHdr->FilterContexts).
    let head = core::ptr::addr_of_mut!((*AdvHdr).FilterContexts);
    (*head).Flink = head;
    (*head).Blink = head;
    if !FastMutex.is_null() {
        (*AdvHdr).FastMutex = FastMutex;
    }
}

// ---------------------------------------------------------------------------
// PSEH3 DUMMY_PSEH: _SEH2_AbnormalTermination() is dead code; always FALSE.
// stdcall, zero args -> @0.  Callers declare -> BOOLEAN (and one as i32; both
// 4-byte returns, same @0 decoration).
// ---------------------------------------------------------------------------
#[no_mangle]
// u32 (not BOOLEAN/u8): callers declare `-> i32` and test full eax — see FlagOn note.
pub unsafe extern "C" fn _SEH2_AbnormalTermination() -> u32 {
    0u32
}

// ===========================================================================
// Batch 3 — second layer of cdfs macros surfaced after batch 2 linked.
// ===========================================================================

const ACQUIRE_EXCLUSIVE: u32 = 0;

// cdprocs.h:
//   #define CdAcquireFcbExclusive(IC,F,I)
//       CdAcquireResource((IC), &(F)->FcbNonpaged->FcbResource, (I), AcquireExclusive)
// stdcall (@12). Returns the CdAcquireResource BOOLEAN result.
#[no_mangle]
pub unsafe extern "C" fn CdAcquireFcbExclusive(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    IgnoreWait: BOOLEAN,
) -> u32 {
    // u32 (not BOOLEAN/u8): callers test full eax — see FlagOn note in ddk_shims.rs.
    CdAcquireResource(
        IrpContext,
        core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).FcbResource) as PERESOURCE,
        IgnoreWait,
        ACQUIRE_EXCLUSIVE,
    ) as u32
}

// cdprocs.h:
//   #define CdReleaseFcb(IC,F)  ExReleaseResourceLite(&(F)->FcbNonpaged->FcbResource)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseFcb(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    ExReleaseResourceLite(
        core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).FcbResource) as PERESOURCE,
    );
}

// cdprocs.h:
//   #define CdLockVcb(IC,V)
//       ExAcquireFastMutex(&(V)->VcbMutex);
//       (V)->VcbLockThread = PsGetCurrentThread()
//   (NT_ASSERT dropped — assert-only.)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdLockVcb(_IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    ExAcquireFastMutex(core::ptr::addr_of_mut!((*Vcb).VcbMutex) as PFAST_MUTEX);
    (*Vcb).VcbLockThread = PsGetCurrentThread() as PVOID;
}

// cdprocs.h:
//   #define CdUnlockVcb(IC,V)
//       (V)->VcbLockThread = NULL;
//       ExReleaseFastMutex(&(V)->VcbMutex)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdUnlockVcb(_IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    (*Vcb).VcbLockThread = null_mut();
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*Vcb).VcbMutex) as PFAST_MUTEX);
}

// cdprocs.h:
//   #define CdDecrementReferenceCounts(IC,F,C,UC) {
//       (F)->FcbReference     -= (C); (F)->FcbUserReference -= (UC);
//       (F)->Vcb->VcbReference -= (C); (F)->Vcb->VcbUserReference -= (UC); }
// stdcall (@16).
#[no_mangle]
pub unsafe extern "C" fn CdDecrementReferenceCounts(
    _IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Count: u32,
    UserCount: u32,
) {
    let c = Count as i32;
    let uc = UserCount as i32;
    (*Fcb).FcbReference -= c;
    (*Fcb).FcbUserReference = (*Fcb).FcbUserReference.wrapping_sub(uc as u32);
    let vcb = (*Fcb).Vcb;
    (*vcb).VcbReference -= c;
    (*vcb).VcbUserReference -= uc;
}

// FCB_TABLE_ELEMENT layout (cdstruc.h): { FILE_ID FileId; PFCB Fcb; }.
#[repr(C)]
struct FCB_TABLE_ELEMENT {
    FileId: FILE_ID,
    Fcb: PFCB,
}

// strucsup.c fn-macro:
//   #define CdDeleteFcbTable(IC,F) {
//       FCB_TABLE_ELEMENT _Key; _Key.FileId = (F)->FileId;
//       RtlDeleteElementGenericTable(&(F)->Vcb->FcbTable, &_Key); }
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdDeleteFcbTable(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    let mut key: FCB_TABLE_ELEMENT = core::mem::zeroed();
    key.FileId = (*Fcb).FileId;
    RtlDeleteElementGenericTable(
        core::ptr::addr_of_mut!((*(*Fcb).Vcb).FcbTable) as PRTL_GENERIC_TABLE,
        core::ptr::addr_of_mut!(key) as PVOID,
    );
}

// cdstruc.h:
//   #define CdFidSetDirectory(I)  SetFlag((I).LowPart, FID_DIR_MASK)
// Caller passes *mut FILE_ID (the macro mutates I.LowPart in place). stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdFidSetDirectory(FileId: *mut FILE_ID) {
    (*FileId).__bindgen_anon_1.LowPart |= FID_DIR_MASK;
}

// cdstruc.h:
//   #define CdSetFidPathTableOffset(I,P)  ((I).HighPart = P)
// Caller passes *mut FILE_ID + LONGLONG P (truncated to HighPart LONG). stdcall (@12).
#[no_mangle]
pub unsafe extern "C" fn CdSetFidPathTableOffset(FileId: *mut FILE_ID, PathTableOffset: i64) {
    (*FileId).__bindgen_anon_1.HighPart = PathTableOffset as i32;
}

// cdprocs.h:
//   #define LlBytesFromBlocks(V,B)  Int64ShllMod32((LONGLONG)(B), (V)->BlockToByteShift)
// = (LONGLONG)B << BlockToByteShift. stdcall (@8) -> LONGLONG.
#[no_mangle]
pub unsafe extern "C" fn LlBytesFromBlocks(Vcb: *mut VCB, BlockCount: u32) -> i64 {
    (BlockCount as i64) << (*Vcb).BlockToByteShift
}

// cdprocs.h:
//   #define SectorAlign(L)     (((ULONG)(L) + (SECTOR_SIZE-1)) & ~(SECTOR_SIZE-1))
// stdcall (@4) -> ULONG.
#[no_mangle]
pub unsafe extern "C" fn SectorAlign(L: u32) -> u32 {
    (L + (SECTOR_SIZE - 1)) & !(SECTOR_SIZE - 1)
}

// cdprocs.h:
//   #define LlSectorAlign(L)   (((LONGLONG)(L) + (SECTOR_SIZE-1)) & ~(SECTOR_SIZE-1))
// stdcall (@8) -> LONGLONG.
#[no_mangle]
pub unsafe extern "C" fn LlSectorAlign(L: i64) -> i64 {
    (L + (SECTOR_SIZE as i64 - 1)) & !(SECTOR_SIZE as i64 - 1)
}

// cdprocs.h:
//   #define CopyUchar4(Dst,Src)  *((UCHAR4*)(Dst)) = *((UNALIGNED UCHAR4*)(Src));
// Aligned 4-byte copy from a possibly-unaligned source. stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CopyUchar4(Dst: *mut u32, Src: *const u8) {
    let v = core::ptr::read_unaligned(Src as *const u32);
    core::ptr::write(Dst, v);
}

// cdprocs.h:
//   #define SwapCopyUchar4(Dst,Src)  byte-reversed 4-byte copy (big<->little endian).
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn SwapCopyUchar4(Dst: *mut u32, Src: *const u8) {
    let d = Dst as *mut u8;
    *d.add(0) = *Src.add(3);
    *d.add(1) = *Src.add(2);
    *d.add(2) = *Src.add(1);
    *d.add(3) = *Src.add(0);
}

// rtlfuncs.h:
//   #define RtlEqualMemory(d,s,len)  (!memcmp((d),(s),(len)))
// stdcall (@12) -> BOOLEAN.
#[no_mangle]
pub unsafe extern "C" fn RtlEqualMemory(
    Buffer1: *const c_void,
    Buffer2: *const c_void,
    Length: usize,
) -> u32 {
    // u32 (not BOOLEAN/u8): callers test full eax — see FlagOn note in ddk_shims.rs.
    let a = core::slice::from_raw_parts(Buffer1 as *const u8, Length);
    let b = core::slice::from_raw_parts(Buffer2 as *const u8, Length);
    if a == b { 1 } else { 0 }
}

// ---------------------------------------------------------------------------
// cd.h CdRvd* accessors: read a field from the raw Volume Descriptor, picking
// the HSG vs ISO struct layout by FlagOn(VcbState, VCB_STATE_HSG). Both structs
// share the same field names but DIFFERENT offsets, so cast per branch.
//   #define CdRvdXxx(R,F)  FlagOn((F),VCB_STATE_HSG) ? ((PRAW_HSG_VD)R)->Xxx
//                                                     : ((PRAW_ISO_VD)R)->Xxx
// All stdcall (@8). VcbState is the F arg (a ULONG flag word).
// ---------------------------------------------------------------------------

#[inline]
unsafe fn is_hsg(vcb_state: u32) -> bool {
    (vcb_state & VCB_STATE_HSG) != 0
}

// CdRvdBlkSz -> LogicalBlkSzI (USHORT). Caller decl -> u32.
#[no_mangle]
pub unsafe extern "C" fn CdRvdBlkSz(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).LogicalBlkSzI as u32
    } else {
        (*(RawVd as *const RAW_ISO_VD)).LogicalBlkSzI as u32
    }
}

// CdRvdPtLoc -> PathTabLocI[0] (ULONG).
#[no_mangle]
pub unsafe extern "C" fn CdRvdPtLoc(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).PathTabLocI[0]
    } else {
        (*(RawVd as *const RAW_ISO_VD)).PathTabLocI[0]
    }
}

// CdRvdPtSz -> PathTableSzI (ULONG).
#[no_mangle]
pub unsafe extern "C" fn CdRvdPtSz(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).PathTableSzI
    } else {
        (*(RawVd as *const RAW_ISO_VD)).PathTableSzI
    }
}

// CdRvdVolSz -> VolSpaceI (ULONG).
#[no_mangle]
pub unsafe extern "C" fn CdRvdVolSz(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).VolSpaceI
    } else {
        (*(RawVd as *const RAW_ISO_VD)).VolSpaceI
    }
}

// CdRvdDirent -> &RootDe[0] (pointer to the embedded root directory entry).
#[no_mangle]
pub unsafe extern "C" fn CdRvdDirent(RawVd: *mut i8, VcbState: u32) -> *mut c_void {
    if is_hsg(VcbState) {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_HSG_VD)).RootDe) as *mut c_void
    } else {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_ISO_VD)).RootDe) as *mut c_void
    }
}

// ---------------------------------------------------------------------------
// CdAudioLabel — cddata.c global `WCHAR CdAudioLabel[] = L"Audio CD"` (8 wchars,
// 16 bytes). The crate's cddata.rs copy lacks #[no_mangle], so the C symbol
// `_CdAudioLabel` was unresolved; export it here with the exact bytes.
// ---------------------------------------------------------------------------
#[no_mangle]
pub static mut CdAudioLabel: [u16; 8] = [
    b'A' as u16, b'u' as u16, b'd' as u16, b'i' as u16, b'o' as u16,
    b' ' as u16, b'C' as u16, b'D' as u16,
];

// ===========================================================================
// Batch 4 — third macro layer surfaced after batch 3 linked.
// ===========================================================================

use crate::types::{IRP_CONTEXT_LITE, RAW_DIRENT};
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, MmMapLockedPagesSpecifyCache, PMDL};
use crate::ddk_shims::CdRaiseStatus;
use crate::dirsup::{CdLookupDirent, CdUpdateDirentFromRawDirent};

// cddata.c global `CHAR CdXaId[] = "CD-XA001"` (8 bytes). cdecl data symbol.
// (cddata.rs has a non-#[no_mangle] copy; export the C symbol here.)
#[no_mangle]
pub static mut CdXaId: [i8; 8] = [
    b'C' as i8, b'D' as i8, b'-' as i8, b'X' as i8, b'A' as i8,
    b'0' as i8, b'0' as i8, b'1' as i8,
];


// cdprocs.h:
//   #define CdAcquireCdData(IC)  ExAcquireResourceExclusiveLite(&CdData.DataResource, TRUE)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireCdData(_IrpContext: *mut IRP_CONTEXT) {
    ExAcquireResourceExclusiveLite(
        core::ptr::addr_of_mut!(CdData.DataResource) as PERESOURCE,
        1, // TRUE
    );
}

// cdprocs.h:
//   #define CdReleaseCdData(IC)  ExReleaseResourceLite(&CdData.DataResource)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseCdData(_IrpContext: *mut IRP_CONTEXT) {
    ExReleaseResourceLite(core::ptr::addr_of_mut!(CdData.DataResource) as PERESOURCE);
}

// cdprocs.h:
//   #define CdAcquireVcbExclusive(IC,V,I)
//       CdAcquireResource((IC), &(V)->VcbResource, (I), AcquireExclusive)
// Link wants BOTH cdecl (_CdAcquireVcbExclusive) and stdcall (@12) forms.
unsafe fn cd_acquire_vcb_excl(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    IgnoreWait: BOOLEAN,
) -> BOOLEAN {
    CdAcquireResource(
        IrpContext,
        core::ptr::addr_of_mut!((*Vcb).VcbResource) as PERESOURCE,
        IgnoreWait,
        ACQUIRE_EXCLUSIVE,
    )
}
#[no_mangle]
pub unsafe extern "C" fn CdAcquireVcbExclusive(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    IgnoreWait: BOOLEAN,
) -> u32 {
    // u32 (not BOOLEAN/u8): callers test full eax — see FlagOn note in ddk_shims.rs.
    cd_acquire_vcb_excl(IrpContext, Vcb, IgnoreWait) as u32
}
// One caller (verfysup.rs) declares this `extern "C"` (cdecl, _CdAcquireVcbExclusive,
// no @N). Two #[no_mangle] items can't share the pre-decoration name, so emit a
// cdecl→stdcall thunk in asm: re-push the 3 args, call the stdcall body (which pops
// them), then cdecl-ret so the cdecl caller cleans its own copy. No stack double-pop.
// [asm thunk removed — ABIs unified via abi_normalize]

// cdprocs.h:
//   #define CdAcquireVcbShared(IC,V,I)
//       CdAcquireResource((IC), &(V)->VcbResource, (I), AcquireShared)
// stdcall (@12).
const ACQUIRE_SHARED: u32 = 1;
#[no_mangle]
pub unsafe extern "C" fn CdAcquireVcbShared(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    IgnoreWait: BOOLEAN,
) -> BOOLEAN {
    CdAcquireResource(
        IrpContext,
        core::ptr::addr_of_mut!((*Vcb).VcbResource) as PERESOURCE,
        IgnoreWait,
        ACQUIRE_SHARED,
    )
}

// cdprocs.h:
//   #define CdReleaseVcb(IC,V)  ExReleaseResourceLite(&(V)->VcbResource)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseVcb(_IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).VcbResource) as PERESOURCE);
}

// cdprocs.h:
//   #define CdCreateIrpContextLite(IC)
//       ExAllocatePoolWithTag(CdNonPagedPool, sizeof(IRP_CONTEXT_LITE), TAG_IRP_CONTEXT_LITE)
// stdcall (@4). TAG_IRP_CONTEXT_LITE = 'lidC' = 0x6c696443.
const TAG_IRP_CONTEXT_LITE: u32 = 0x6c696443;
#[no_mangle]
pub unsafe extern "C" fn CdCreateIrpContextLite(
    _IrpContext: *mut IRP_CONTEXT,
) -> *mut IRP_CONTEXT_LITE {
    ExAllocatePoolWithTag(
        CD_NONPAGED_POOL as _,
        core::mem::size_of::<IRP_CONTEXT_LITE>() as _,
        TAG_IRP_CONTEXT_LITE,
    ) as *mut IRP_CONTEXT_LITE
}

// cdprocs.h:
//   #define CdFreeIrpContextLite(ICL)  CdFreePool(&(ICL))
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdFreeIrpContextLite(IrpContextLite: *mut c_void) {
    let mut local: *mut c_void = IrpContextLite;
    CdFreePool(core::ptr::addr_of_mut!(local));
}

// cdprocs.h:
//   #define CdIncrementCleanupCounts(IC,F) {
//       (F)->FcbCleanup += 1; (F)->Vcb->VcbCleanup += 1; }
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdIncrementCleanupCounts(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    (*Fcb).FcbCleanup += 1;
    (*(*Fcb).Vcb).VcbCleanup += 1;
}

// cdprocs.h:
//   #define CdIllegalFcbAccess(IC,T,DA)  BooleanFlagOn(DA, <write-access mask>)
//   mask = (T != UserVolumeOpen ? volume-write bits : 0) | FILE_DELETE_CHILD | DELETE | WRITE_DAC
// stdcall (@12) -> BOOLEAN.
const TYPE_OF_OPEN_USER_VOLUME_OPEN: u32 = 2; // _TYPE_OF_OPEN_UserVolumeOpen
const FILE_WRITE_DATA: u32 = 0x0002;
const FILE_APPEND_DATA: u32 = 0x0004;
const FILE_WRITE_EA: u32 = 0x0010;
const FILE_WRITE_ATTRIBUTES: u32 = 0x0100;
const FILE_ADD_FILE: u32 = 0x0002;
const FILE_ADD_SUBDIRECTORY: u32 = 0x0004;
const FILE_DELETE_CHILD: u32 = 0x0040;
const DELETE: u32 = 0x00010000;
const WRITE_DAC: u32 = 0x00040000;
#[no_mangle]
pub unsafe extern "C" fn CdIllegalFcbAccess(
    _IrpContext: *mut IRP_CONTEXT,
    TypeOfOpen: u32,
    DesiredAccess: u32,
) -> BOOLEAN {
    let vol_bits = if TypeOfOpen != TYPE_OF_OPEN_USER_VOLUME_OPEN {
        FILE_WRITE_ATTRIBUTES
            | FILE_WRITE_DATA
            | FILE_WRITE_EA
            | FILE_ADD_FILE
            | FILE_ADD_SUBDIRECTORY
            | FILE_APPEND_DATA
    } else {
        0
    };
    let mask = vol_bits | FILE_DELETE_CHILD | DELETE | WRITE_DAC;
    if (DesiredAccess & mask) != 0 { 1 } else { 0 }
}

// cdprocs.h fn-macro CdLookupInitialFileDirent(IC,F,FC,DO):
//   CdLookupDirent(IC, F, DO, &FC->InitialDirent->DirContext);
//   CdUpdateDirentFromRawDirent(IC, F, &FC->InitialDirent->DirContext,
//                                       &FC->InitialDirent->Dirent);
// stdcall (@16). DO = DirentOffset (ULONG).
#[no_mangle]
pub unsafe extern "C" fn CdLookupInitialFileDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    FileContext: *mut FILE_ENUM_CONTEXT,
    DirentOffset: u32,
) {
    let initial = (*FileContext).InitialDirent;
    let dir_context = core::ptr::addr_of_mut!((*initial).DirContext) as *mut DIRENT_ENUM_CONTEXT;
    CdLookupDirent(IrpContext, Fcb, DirentOffset, dir_context);
    CdUpdateDirentFromRawDirent(
        IrpContext,
        Fcb,
        dir_context,
        core::ptr::addr_of_mut!((*initial).Dirent) as *mut DIRENT,
    );
}

// cdprocs.h fn-macro CdMapUserBuffer(IC, UB):
//   *UB = (Irp->MdlAddress == NULL) ? Irp->UserBuffer
//                                   : MmGetSystemAddressForMdlSafe(Irp->MdlAddress, ...);
//   if (*UB == NULL) CdRaiseStatus(IC, STATUS_INSUFFICIENT_RESOURCES);
// stdcall (@8). UB is PVOID* (caller decl: *mut *mut u8 / *mut *mut c_void).
const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xC000009Au32 as i32;
const MDL_MAPPED_TO_SYSTEM_VA: i16 = 0x0001;
const MDL_SOURCE_IS_NONPAGED_POOL: i16 = 0x0004;
const NORMAL_PAGE_PRIORITY: u32 = 16;
const MDL_MAPPING_NO_EXECUTE: u32 = 0x40000000;
const KERNEL_MODE: i8 = 0; // _MODE_KernelMode
const MM_CACHED: i32 = 1; // _MEMORY_CACHING_TYPE_MmCached
#[inline]
unsafe fn mm_get_system_address_for_mdl_safe(mdl: PMDL, priority: u32) -> *mut c_void {
    if ((*mdl).MdlFlags as i16 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL)) != 0 {
        (*mdl).MappedSystemVa
    } else {
        MmMapLockedPagesSpecifyCache(
            mdl,
            KERNEL_MODE as _,
            MM_CACHED as _,
            null_mut(),
            0, // FALSE
            priority,
        )
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdMapUserBuffer(
    IrpContext: *mut IRP_CONTEXT,
    UserBuffer: *mut *mut c_void,
) {
    let irp = (*IrpContext).Irp;
    if (*irp).MdlAddress.is_null() {
        *UserBuffer = (*irp).UserBuffer;
    } else {
        *UserBuffer = mm_get_system_address_for_mdl_safe(
            (*irp).MdlAddress as PMDL,
            NORMAL_PAGE_PRIORITY | MDL_MAPPING_NO_EXECUTE,
        );
    }
    if (*UserBuffer).is_null() {
        CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
    }
}

// dirsup.c fn-macro CdRawDirent(IC,DC):
//   Add2Ptr((DC)->Sector, (DC)->SectorOffset, PRAW_DIRENT)
// cdecl (_CdRawDirent, no @N) -> PRAW_DIRENT.
#[no_mangle]
pub unsafe extern "C" fn CdRawDirent(
    _IrpContext: *mut IRP_CONTEXT,
    DirContext: *mut DIRENT_ENUM_CONTEXT,
) -> *mut RAW_DIRENT {
    ((*DirContext).Sector as *mut u8).add((*DirContext).SectorOffset as usize) as *mut RAW_DIRENT
}

// cdstruc.h:
//   #define CdSetFidDirentOffset(I,D)  ((I).LowPart = D)
// Caller passes *mut FILE_ID + D (ULONG). stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdSetFidDirentOffset(FileId: *mut FILE_ID, DirentOffset: u32) {
    (*FileId).__bindgen_anon_1.LowPart = DirentOffset;
}

// cdstruc.h fn-macro CdSetFidFromParentAndDirent(I,F,D):
//   CdSetFidPathTableOffset(I, CdQueryFidPathTableOffset((F)->FileId));
//   CdSetFidDirentOffset(I, (D)->DirentOffset);
//   if (FlagOn((D)->DirentFlags, CD_ATTRIBUTE_DIRECTORY)) CdFidSetDirectory(I);
// stdcall (@12). I = *mut FILE_ID, F = *mut FCB, D = *mut DIRENT.
const CD_ATTRIBUTE_DIRECTORY: u8 = 0x02;
#[no_mangle]
pub unsafe extern "C" fn CdSetFidFromParentAndDirent(
    FileId: *mut FILE_ID,
    Fcb: *mut FCB,
    Dirent: *mut DIRENT,
) {
    // CdQueryFidPathTableOffset((F)->FileId) == FileId.HighPart.
    (*FileId).__bindgen_anon_1.HighPart = (*Fcb).FileId.__bindgen_anon_1.HighPart;
    (*FileId).__bindgen_anon_1.LowPart = (*Dirent).DirentOffset;
    if ((*Dirent).DirentFlags & CD_ATTRIBUTE_DIRECTORY) != 0 {
        (*FileId).__bindgen_anon_1.LowPart |= FID_DIR_MASK;
    }
}

// rtlfuncs.h: #define RtlCopyMemory(d,s,len)  memcpy(d,s,len)
// stdcall (@12).
#[no_mangle]
pub unsafe extern "system" fn RtlCopyMemory(
    Destination: *mut c_void,
    Source: *const c_void,
    Length: usize,
) {
    core::ptr::copy_nonoverlapping(Source as *const u8, Destination as *mut u8, Length);
}

// ---------------------------------------------------------------------------
// CdDecodeFileObject / CdSetFileObject — real C functions in filobsup.c, but
// EXPORTED THERE AS CDECL (_CdDecodeFileObject / _CdSetFileObject). The Rust
// callers reference the stdcall decoration (@16 / @20). Provide stdcall thunks
// that forward to the cdecl C implementations. The cdecl imports live in a
// nested module so the Rust identifier doesn't clash with the thunk definition.
// ---------------------------------------------------------------------------
// NOTE: A Rust `#[no_mangle] extern "system"` thunk that forwards to a same-named
// `extern "C"` import does NOT work — rustc resolves the call to the local
// definition (the thunk), yielding infinite recursion that LLVM folds to `ud2`.
// So the stdcall→cdecl adapters are emitted as raw asm thunks instead: re-push the
// args, `call` the cdecl C body (caller-cleans), then `ret N` (stdcall-pops).
// (amd64/arm64 NTAPI == cdecl, so the stdcall/cdecl decorations coincide and the
//  bare extern import resolves directly; the asm thunks are i386-only.)
// [filobsup_passthrough mod removed — ABIs unified via abi_normalize]

// i386 stdcall(@16/@20) → cdecl adapter thunks for filobsup.c's cdecl exports.
// [asm thunk removed — ABIs unified via abi_normalize]

// ===========================================================================
// Batch 5 — fourth macro layer surfaced after batch 4 linked.
// ===========================================================================

use ntkrnl_sys::ddk::{
    CcWaitForCurrentLazyWriterActivity, IoAcquireVpbSpinLock, IoReleaseVpbSpinLock,
    KIRQL, PIO_COMPLETION_ROUTINE, PVPB,
};
use ntkrnl_sys::base::NT_SUCCESS;
use crate::cachesup::CdPurgeVolume;
use crate::close::CdFspClose;
use crate::deviosup::CdCreateUserMdl;

// cddata.c globals (CHAR[5]). cdecl data symbols.
#[no_mangle]
pub static mut CdHsgId: [i8; 5] =
    [b'C' as i8, b'D' as i8, b'R' as i8, b'O' as i8, b'M' as i8];
#[no_mangle]
pub static mut CdIsoId: [i8; 5] =
    [b'C' as i8, b'D' as i8, b'0' as i8, b'0' as i8, b'1' as i8];

// SAL no-ops in the cohab build (concurrencysal.h expands them to nothing).
#[no_mangle]
pub unsafe extern "C" fn PagedCode() {}

// cdprocs.h:
//   #define CdAcquireCacheForUpdate(IC)
//       ExAcquireResourceExclusiveLite(&(IC)->Vcb->SectorCacheResource, TRUE)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireCacheForUpdate(IrpContext: *mut IRP_CONTEXT) {
    ExAcquireResourceExclusiveLite(
        core::ptr::addr_of_mut!((*(*IrpContext).Vcb).SectorCacheResource) as PERESOURCE,
        1,
    );
}

// cdprocs.h:
//   #define CdReleaseCache(IC)  ExReleaseResourceLite(&(IC)->Vcb->SectorCacheResource)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseCache(IrpContext: *mut IRP_CONTEXT) {
    ExReleaseResourceLite(
        core::ptr::addr_of_mut!((*(*IrpContext).Vcb).SectorCacheResource) as PERESOURCE,
    );
}

// cdprocs.h fn-macro CdLockUserBuffer(IC, BL, OP):
//   if (IC->Irp->MdlAddress == NULL) (VOID) CdCreateUserMdl(IC, BL, TRUE, OP);
// stdcall (@12). OP = LOCK_OPERATION (caller passes u32 DesiredAccess slot).
#[no_mangle]
pub unsafe extern "C" fn CdLockUserBuffer(
    IrpContext: *mut IRP_CONTEXT,
    ByteCount: u32,
    Operation: u32,
) {
    if (*(*IrpContext).Irp).MdlAddress.is_null() {
        let _ = CdCreateUserMdl(IrpContext, ByteCount, 1, Operation as _);
    }
}

// cdprocs.h:
//   #define CdIsRawDevice(IC,S)
//       ((S) == STATUS_DEVICE_NOT_READY) || ((S) == STATUS_NO_MEDIA_IN_DEVICE)
// stdcall (@8) -> BOOLEAN.
const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
const STATUS_NO_MEDIA_IN_DEVICE: i32 = 0xC0000013u32 as i32;
#[no_mangle]
pub unsafe extern "C" fn CdIsRawDevice(_IrpContext: *mut IRP_CONTEXT, Status: i32) -> BOOLEAN {
    if Status == STATUS_DEVICE_NOT_READY || Status == STATUS_NO_MEDIA_IN_DEVICE {
        1
    } else {
        0
    }
}

// cdprocs.h:
//   #define LlBytesFromSectors(L)  Int64ShllMod32((LONGLONG)(L), SECTOR_SHIFT)
//   = (LONGLONG)L << SECTOR_SHIFT. stdcall (@4) -> LONGLONG.
const SECTOR_SHIFT: u32 = 11;
#[no_mangle]
pub unsafe extern "C" fn LlBytesFromSectors(SectorCount: u32) -> i64 {
    (SectorCount as i64) << SECTOR_SHIFT
}

// cdprocs.h:
//   #define LlSectorsFromBytes(L)  Int64ShraMod32((LONGLONG)(L), SECTOR_SHIFT)
//   = (LONGLONG)L >> SECTOR_SHIFT (arithmetic). stdcall (@8) -> LONGLONG.
#[no_mangle]
pub unsafe extern "C" fn LlSectorsFromBytes(Bytes: i64) -> i64 {
    Bytes >> SECTOR_SHIFT
}

// ---------------------------------------------------------------------------
// Io* IRP-stack inline macros — bindgen wrongly emitted them as extern fns;
// forward to the real inline implementations in ntkrnl_sys::inline::tier1.
// cdecl (no @N), matching the bindgen `extern "C"` decls in ntkrnl_sys::ddk.
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(Irp);
}

#[no_mangle]
pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoCopyCurrentIrpStackLocationToNext(Irp);
}

#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: PIRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    InvokeOnSuccess: BOOLEAN,
    InvokeOnError: BOOLEAN,
    InvokeOnCancel: BOOLEAN,
) {
    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        Irp,
        CompletionRoutine,
        Context,
        InvokeOnSuccess,
        InvokeOnError,
        InvokeOnCancel,
    );
}

// ---------------------------------------------------------------------------
// cd.h CdRvd* string/array accessors. Same HSG/ISO branch pattern as batch 3.
//   CdRvdId/Esc/VolId -> pointer to a UCHAR[] field (caller decl -> *mut i8).
//   CdRvdVersion/DescType -> a single UCHAR (caller decl -> u32).
// All stdcall (@8).
// ---------------------------------------------------------------------------

// CdRvdId -> &StandardId[0] (UCHAR[5]).
#[no_mangle]
pub unsafe extern "C" fn CdRvdId(RawVd: *mut i8, VcbState: u32) -> *mut i8 {
    if is_hsg(VcbState) {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_HSG_VD)).StandardId) as *mut i8
    } else {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_ISO_VD)).StandardId) as *mut i8
    }
}

// CdRvdVersion -> Version (UCHAR).
#[no_mangle]
pub unsafe extern "C" fn CdRvdVersion(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).Version as u32
    } else {
        (*(RawVd as *const RAW_ISO_VD)).Version as u32
    }
}

// CdRvdDescType -> DescType (UCHAR).
#[no_mangle]
pub unsafe extern "C" fn CdRvdDescType(RawVd: *mut i8, VcbState: u32) -> u32 {
    if is_hsg(VcbState) {
        (*(RawVd as *const RAW_HSG_VD)).DescType as u32
    } else {
        (*(RawVd as *const RAW_ISO_VD)).DescType as u32
    }
}

// CdRvdEsc -> &CharSet[0] (UCHAR[32]).
#[no_mangle]
pub unsafe extern "C" fn CdRvdEsc(RawVd: *mut i8, VcbState: u32) -> *mut i8 {
    if is_hsg(VcbState) {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_HSG_VD)).CharSet) as *mut i8
    } else {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_ISO_VD)).CharSet) as *mut i8
    }
}

// CdRvdVolId -> &VolumeId[0] (UCHAR[32]).
#[no_mangle]
pub unsafe extern "C" fn CdRvdVolId(RawVd: *mut i8, VcbState: u32) -> *mut i8 {
    if is_hsg(VcbState) {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_HSG_VD)).VolumeId) as *mut i8
    } else {
        core::ptr::addr_of_mut!((*(RawVd as *mut RAW_ISO_VD)).VolumeId) as *mut i8
    }
}

// ---------------------------------------------------------------------------
// CdLockVolumeInternal — defined in fsctrl.rs as a PRIVATE `unsafe fn` (a
// translation bug: it should have been #[no_mangle] pub extern). Cross-module
// callers (pnp.rs, the bindgen `pub fn` decl) reference the stdcall symbol
// `_CdLockVolumeInternal@12`, which is therefore unresolved. Reimplemented here
// faithfully from the visible fsctrl.rs body, exported with the right ABI.
// (All callees below are exported / available; constants mirror fsctrl.rs.)
// ---------------------------------------------------------------------------
const STATUS_SUCCESS: i32 = 0;
const STATUS_ACCESS_DENIED: i32 = 0xC0000022u32 as i32;
const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
const VCB_STATE_LOCKED: u32 = 0x00000010;
const VPB_LOCKED: u16 = 0x0002;
const CDFS_RESIDUAL_USER_REFERENCE: u32 = 3; // cddata.h (grind had 1)
#[no_mangle]
pub unsafe extern "C" fn CdLockVolumeInternal(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FileObject: PFILE_OBJECT,
) -> i32 {
    let mut saved_irql: KIRQL = 0;
    let mut final_status: i32 =
        if !FileObject.is_null() { STATUS_ACCESS_DENIED } else { STATUS_DEVICE_BUSY };
    let remaining_user_references: u32 = if !FileObject.is_null() { 1 } else { 0 };

    CdPurgeVolume(IrpContext, Vcb, 0);
    CdReleaseVcb(IrpContext, Vcb);

    let status = CcWaitForCurrentLazyWriterActivity();

    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;
    CdAcquireVcbExclusive(IrpContext, Vcb, 0);

    if !NT_SUCCESS(status) {
        return status;
    }

    CdFspClose(Vcb);

    IoAcquireVpbSpinLock(&mut saved_irql);

    let vpb: PVPB = (*Vcb).Vpb;
    if ((*vpb).Flags & VPB_LOCKED) == 0
        && (*Vcb).VcbCleanup == remaining_user_references
        && (*Vcb).VcbUserReference
            == (CDFS_RESIDUAL_USER_REFERENCE + remaining_user_references) as i32
    {
        (*Vcb).VcbState |= VCB_STATE_LOCKED;
        (*vpb).Flags |= VPB_LOCKED;
        (*Vcb).VolumeLockFileObject = FileObject;
        final_status = STATUS_SUCCESS;
    }

    IoReleaseVpbSpinLock(saved_irql);

    final_status
}

// ===========================================================================
// Batch 6 — fifth macro layer surfaced after batch 5 linked.
// ===========================================================================

use crate::types::{PATH_ENUM_CONTEXT, RAW_PATH_ENTRY, RAW_PATH_HSG};
use ntkrnl_sys::ddk::PIO_STACK_LOCATION;

// SAL / PSEH no-ops in the cohab (DUMMY_PSEH) build.
// _SEH2_TRY: cdecl (__SEH2_TRY, no @N).
#[no_mangle]
pub unsafe extern "C" fn _SEH2_TRY() {}


// cdprocs.h:
//   #define BlockOffset(V,L)  ((ULONG)(L) & (V)->BlockMask)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn BlockOffset(Vcb: *mut VCB, L: u32) -> u32 {
    L & (*Vcb).BlockMask
}

// cdprocs.h:
//   #define SectorOffset(L)  ((ULONG)(ULONG_PTR)(L) & SECTOR_MASK)
//   SECTOR_MASK = SECTOR_SIZE - 1. stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn SectorOffset(L: u32) -> u32 {
    L & (SECTOR_SIZE - 1)
}

// cdprocs.h:
//   #define WordAlign(Ptr)  (((ULONG)(Ptr) + 1) & 0xfffffffe)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn WordAlign(Ptr: u32) -> u32 {
    (Ptr + 1) & 0xfffffffe
}

// cdprocs.h:
//   #define CopyUchar2(Dst,Src)  *((UCHAR2*)Dst) = *((UNALIGNED UCHAR2*)Src);
// Aligned 2-byte copy from possibly-unaligned source. stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CopyUchar2(Dst: *mut u16, Src: *const u16) {
    let v = core::ptr::read_unaligned(Src as *const u8 as *const u16);
    core::ptr::write(Dst, v);
}

// cdprocs.h:
//   #define CdAcquireAllFiles(IC,V)
//       CdAcquireResource((IC), &(V)->FileResource, FALSE, AcquireExclusive)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireAllFiles(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    CdAcquireResource(
        IrpContext,
        core::ptr::addr_of_mut!((*Vcb).FileResource) as PERESOURCE,
        0, // FALSE
        ACQUIRE_EXCLUSIVE,
    );
}

// cdprocs.h:
//   #define CdReleaseAllFiles(IC,V)  ExReleaseResourceLite(&(V)->FileResource)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdReleaseAllFiles(_IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) {
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).FileResource) as PERESOURCE);
}

// pathsup.c:
//   #define CdRawPathEntry(IC, PC)  Add2Ptr((PC)->Data, (PC)->DataOffset, PRAW_PATH_ENTRY)
// cdecl (_CdRawPathEntry, no @N).
#[no_mangle]
pub unsafe extern "C" fn CdRawPathEntry(
    _IrpContext: *mut IRP_CONTEXT,
    PathContext: *const PATH_ENUM_CONTEXT,
) -> *mut RAW_PATH_ENTRY {
    ((*PathContext).Data as *mut u8).add((*PathContext).DataOffset as usize)
        as *mut RAW_PATH_ENTRY
}

// cd.h:
//   #define CdRawPathIdLen(IC, RP)
//       FlagOn(IC->Vcb->VcbState, VCB_STATE_HSG) ? ((PRAW_PATH_HSG)RP)->DirIdLen : RP->DirIdLen
// stdcall (@8) -> ULONG.
#[no_mangle]
pub unsafe extern "C" fn CdRawPathIdLen(
    IrpContext: *mut IRP_CONTEXT,
    RawPathEntry: *const RAW_PATH_ENTRY,
) -> u32 {
    if ((*(*IrpContext).Vcb).VcbState & VCB_STATE_HSG) != 0 {
        (*(RawPathEntry as *const RAW_PATH_HSG)).DirIdLen as u32
    } else {
        (*RawPathEntry).DirIdLen as u32
    }
}

// cd.h: CdRawPathXar -> XarLen (UCHAR). stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdRawPathXar(
    IrpContext: *mut IRP_CONTEXT,
    RawPathEntry: *const RAW_PATH_ENTRY,
) -> u32 {
    if ((*(*IrpContext).Vcb).VcbState & VCB_STATE_HSG) != 0 {
        (*(RawPathEntry as *const RAW_PATH_HSG)).XarLen as u32
    } else {
        (*RawPathEntry).XarLen as u32
    }
}

// cd.h: CdRawPathLoc -> &DirLoc[0] (PUSHORT). stdcall (@8) -> *const ULONG (per caller).
#[no_mangle]
pub unsafe extern "C" fn CdRawPathLoc(
    IrpContext: *mut IRP_CONTEXT,
    RawPathEntry: *const RAW_PATH_ENTRY,
) -> *const u32 {
    if ((*(*IrpContext).Vcb).VcbState & VCB_STATE_HSG) != 0 {
        core::ptr::addr_of!((*(RawPathEntry as *const RAW_PATH_HSG)).DirLoc) as *const u32
    } else {
        core::ptr::addr_of!((*RawPathEntry).DirLoc) as *const u32
    }
}

// cdprocs.h:
//   #define CdMarkRealDevForVerify(DO)  SetFlag((DO)->Flags, DO_VERIFY_VOLUME)
// cdecl (_CdMarkRealDevForVerify, no @N).
const DO_VERIFY_VOLUME: u32 = 0x00000002;
#[no_mangle]
pub unsafe extern "C" fn CdMarkRealDevForVerify(DeviceObject: PDEVICE_OBJECT) {
    (*DeviceObject).Flags |= DO_VERIFY_VOLUME;
}

// cdprocs.h:
//   #define CdRealDevNeedsVerify(DO)  BooleanFlagOn((DO)->Flags, DO_VERIFY_VOLUME)
// cdecl (_CdRealDevNeedsVerify, no @N) -> BOOLEAN.
#[no_mangle]
pub unsafe extern "C" fn CdRealDevNeedsVerify(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN {
    if ((*DeviceObject).Flags & DO_VERIFY_VOLUME) != 0 { 1 } else { 0 }
}

// cdprocs.h inline CdOperationIsDasdOpen(IC):
//   IrpSp = IoGetCurrentIrpStackLocation(IC->Irp);
//   return (IC->MajorFunction == IRP_MJ_CREATE) &&
//          (IrpSp->FileObject->FileName.Length == 0) &&
//          (IrpSp->FileObject->RelatedFileObject == NULL);
// cdecl (_CdOperationIsDasdOpen, no @N) -> BOOLEAN.
const IRP_MJ_CREATE: u8 = 0x00;
#[no_mangle]
// u32 (not BOOLEAN/u8): callers test full eax — see FlagOn note in ddk_shims.rs.
pub unsafe extern "C" fn CdOperationIsDasdOpen(IrpContext: *mut IRP_CONTEXT) -> u32 {
    let irp_sp: PIO_STACK_LOCATION =
        ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation((*IrpContext).Irp);
    let file_object = (*irp_sp).FileObject;
    if (*IrpContext).MajorFunction == IRP_MJ_CREATE
        && (*file_object).FileName.Length == 0
        && (*file_object).RelatedFileObject.is_null()
    {
        1
    } else {
        0
    }
}

// ntbasedef.h: #define Int32x32To64(a,b)  ((LONGLONG)(a) * (b))
// cdecl (_Int32x32To64, no @N).
#[no_mangle]
pub unsafe extern "C" fn Int32x32To64(a: i32, b: i32) -> i64 {
    (a as i64) * (b as i64)
}

// ---------------------------------------------------------------------------
// IoVerifyVolume / IoSetHardErrorOrVerifyDevice — real ntoskrnl exports, but
// verfysup.rs declares them extern "C" (cdecl _IoVerifyVolume), while ntoskrnl's
// i386 import lib exports the stdcall decoration (_IoVerifyVolume@8 / @8). Provide
// cdecl→stdcall adapter thunks forwarding to the real (stdcall) imports. The
// stdcall imports live in a nested module so the Rust ident doesn't clash.
// ---------------------------------------------------------------------------
// amd64/arm64: NTAPI == cdecl, the bare cdecl import resolves to the real export.
// [ntoskrnl_verify_passthrough mod removed — ABIs unified via abi_normalize]

// i386 cdecl → stdcall(@8) adapter thunks for the real ntoskrnl exports.
// (same-name Rust thunk would self-recurse → ud2; emit raw asm instead.)
// [asm thunk removed — ABIs unified via abi_normalize]

// ===========================================================================
// Batch 7 — sixth macro layer surfaced after batch 6 linked.
// ===========================================================================

use ntkrnl_sys::ddk::{
    ExAcquireResourceSharedLite, ExConvertExclusiveToSharedLite, MDL,
};
use crate::ddk_shims::CdRaiseStatusEx;

// FsRtlLegalAnsiCharacterArray is a real ntoskrnl DATA export. The EXPORTED symbol is a
// SINGLE pointer to the byte array — `const UCHAR * const FsRtlLegalAnsiCharacterArray`
// (fsrtlfuncs.h:1600/1606, ntoskrnl fsrtlpc.c:154 / fsrtlpc.rs:176, value = array base).
// (The `* const *` form in fsrtlfuncs.h:1610 is only the __CREATE_NTOS_DATA_IMPORT_ALIAS
// variant; it is NOT what ntoskrnl exports.) Declaring it `*const *const u8` + double-deref
// over-derefs by one level: the raw-dylib __imp_ load already yields the symbol's value (the
// array base), so the extra `*` reads array bytes AS a pointer → access violation in
// CdIsLegalName. Declare a single pointer and index it directly.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static FsRtlLegalAnsiCharacterArray: *const u8;
}

// PSEH3 DUMMY no-ops / exception accessors (SEH bodies are dead code).
//   _SEH2_END()                 -> __SEH2_END@0
//   _SEH2_EXCEPT(code)          -> __SEH2_EXCEPT@4
//   _SEH2_GetExceptionCode()    -> __SEH2_GetExceptionCode@0, must return 0
//   _Analysis_assume_(cond)     -> __Analysis_assume_@4
#[no_mangle]
pub unsafe extern "C" fn _SEH2_END() {}
#[no_mangle]
pub unsafe extern "C" fn _SEH2_EXCEPT(_Code: i32) {}
#[no_mangle]
pub unsafe extern "C" fn _SEH2_GetExceptionCode() -> i32 {
    0
}

// KeFlushIoBuffers is a MACRO, per arch: empty on x86/amd64 (cache-coherent DMA,
// xdk/x86/ke.h:188), `_DataSynchronizationBarrier()` on arm64 (xdk/arm64/ke.h:106).
// Only 32-bit ARM exports a real function, so the driver carries its own. This was
// empty on every arch, which dropped the arm64 barrier after a DMA read. (fastfat gets
// it right by capturing the real macro in C: rust_cohab_fat.c.) Caller declares it
// extern "system" -> _KeFlushIoBuffers@12 on i386.
#[no_mangle]
pub unsafe extern "system" fn KeFlushIoBuffers(
    _Mdl: *mut MDL,
    _ReadOperation: i32,
    _DmaOperation: i32,
) {
    // Not `nomem`: the barrier must also order the compiler's own memory accesses.
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}

// cdprocs.h: #define BytesFromSectors(L)  ((ULONG)(L) << SECTOR_SHIFT). stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn BytesFromSectors(SectorCount: u32) -> u32 {
    SectorCount << SECTOR_SHIFT
}

// cdprocs.h: #define SectorsFromBytes(L)  ((ULONG)(L) >> SECTOR_SHIFT). cdecl (no @N).
#[no_mangle]
pub unsafe extern "C" fn SectorsFromBytes(ByteCount: u32) -> u32 {
    ByteCount >> SECTOR_SHIFT
}

// cdprocs.h static INLINE SectorsFromLlBytes(Bytes) { return (ULONG)(Bytes >> SECTOR_SHIFT); }
// Caller decl returns u32; stdcall (@8) (i64 arg = 8 bytes).
#[no_mangle]
pub unsafe extern "C" fn SectorsFromLlBytes(Bytes: i64) -> u32 {
    (Bytes >> SECTOR_SHIFT) as u32
}

// cdprocs.h: #define Min(a,b)  ((a) < (b) ? (a) : (b)). stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn Min(a: u32, b: u32) -> u32 {
    if a < b { a } else { b }
}

// cdprocs.h:
//   #define CdAcquireCacheForRead(IC)
//       ExAcquireResourceSharedLite(&(IC)->Vcb->SectorCacheResource, TRUE)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireCacheForRead(IrpContext: *mut IRP_CONTEXT) {
    ExAcquireResourceSharedLite(
        core::ptr::addr_of_mut!((*(*IrpContext).Vcb).SectorCacheResource) as PERESOURCE,
        1,
    );
}

// cdprocs.h:
//   #define CdConvertCacheToShared(IC)
//       ExConvertExclusiveToSharedLite(&(IC)->Vcb->SectorCacheResource)
// stdcall (@4).
#[no_mangle]
pub unsafe extern "C" fn CdConvertCacheToShared(IrpContext: *mut IRP_CONTEXT) {
    ExConvertExclusiveToSharedLite(
        core::ptr::addr_of_mut!((*(*IrpContext).Vcb).SectorCacheResource) as PERESOURCE,
    );
}

// cdprocs.h:
//   #define CdNormalizeAndRaiseStatus(IC,S)  CdRaiseStatusEx(IC, S, TRUE, BugCheckFileId, __LINE__)
// stdcall (@8). Raises (does not return). NormalizeStatus=TRUE.
#[no_mangle]
pub unsafe extern "C" fn CdNormalizeAndRaiseStatus(
    IrpContext: *mut IRP_CONTEXT,
    Status: i32,
) {
    CdRaiseStatusEx(IrpContext, Status, 1, 0, 0);
}

// cd.h:
//   #define CdRawDirentFlags(IC,RD)
//       FlagOn(IC->Vcb->VcbState, VCB_STATE_HSG) ? (RD)->FlagsHSG : (RD)->FlagsISO
// cdecl (_CdRawDirentFlags, no @N) -> ULONG.
#[no_mangle]
pub unsafe extern "C" fn CdRawDirentFlags(
    IrpContext: *mut IRP_CONTEXT,
    RawDirent: *mut RAW_DIRENT,
) -> u32 {
    if ((*(*IrpContext).Vcb).VcbState & VCB_STATE_HSG) != 0 {
        (*RawDirent).FlagsHSG as u32
    } else {
        (*RawDirent).FlagsISO as u32
    }
}

// ---------------------------------------------------------------------------
// Io*NextIrpStackLocation — bindgen emitted these inline macros as extern fns;
// forward to the real inline impls. cdecl (no @N).
// ---------------------------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetNextIrpStackLocation(Irp: PIRP) {
    // wdm.h: IoSetNextIrpStackLocation = --CurrentLocation; --CurrentStackLocation.
    (*Irp).CurrentLocation -= 1;
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(-1);
}

// ---------------------------------------------------------------------------
// FsRtl ANSI-char legality macros (fsrtlfuncs.h). HPFS legality uses the
// ntoskrnl-exported FsRtlLegalAnsiCharacterArray; lead-DBCS uses NLS state we
// don't track in cdfs (cdfs only feeds ASCII names), so it returns FALSE.
// ---------------------------------------------------------------------------
const FSRTL_HPFS_LEGAL: u8 = 2;
const FSRTL_WILD_CHARACTER: u8 = 8;
// FsRtlTestAnsiCharacter(C, TRUE, WILD, FSRTL_HPFS_LEGAL):
//   ((SCHAR)C < 0) ? TRUE : FlagOn(LegalArray[C], FSRTL_HPFS_LEGAL | (WILD?WILD_CHAR:0))
// stdcall (@8) -> BOOLEAN.
#[no_mangle]
pub unsafe extern "C" fn FsRtlIsAnsiCharacterLegalHpfs(Char: u16, WildOk: i32) -> u32 {
    let c = Char as u8;
    if (c as i8) < 0 {
        return 1; // DEFAULT_RET = TRUE
    }
    let flags = FSRTL_HPFS_LEGAL | if WildOk != 0 { FSRTL_WILD_CHARACTER } else { 0 };
    // LEGAL_ANSI_CHARACTER_ARRAY = FsRtlLegalAnsiCharacterArray (single ptr); index by C.
    let legal = *FsRtlLegalAnsiCharacterArray.add(c as usize);
    if (legal & flags) != 0 { 1 } else { 0 }
}

// FsRtlIsLeadDbcsCharacter(C): (UCHAR)C < 0x80 ? FALSE : (NLS_MB_CODE_PAGE_TAG && ...).
// cdfs only handles single-byte/Unicode names here; conservatively FALSE for the
// ASCII range and FALSE otherwise (no DBCS code page tracked in this shim).
// stdcall (@4) -> BOOLEAN.
#[no_mangle]
pub unsafe extern "C" fn FsRtlIsLeadDbcsCharacter(_DbcsChar: u8) -> BOOLEAN {
    0
}

// cddata.c: LONG CdAudioPlayHeader[] = { RIFF chunk for a CDDA WAV header }.
// Data export (cdecl _CdAudioPlayHeader). 11 LONGs (matches AUDIO_PLAY_HEADER).
#[no_mangle]
pub static mut CdAudioPlayHeader: [u32; 11] = [
    0x46464952, // 'RIFF'
    4 * 11 - 8, // chunk size
    0x41444443, // 'CDDA'
    0x20746d66, // 'fmt '
    24,         // fmt chunk size
    0x00000001, // FormatTag=1, TrackNumber=0
    0x00000000, // DiskID
    0x00000000, // StartingSector
    0x00000000, // SectorCount
    0x00000000, // TrackAddress
    0x00000000, // TrackLength
];

// ===========================================================================
// Batch 8 — remaining cdfs data globals + the IRP_CONTEXT_FSP_FLAGS constant
// that a caller declared as an extern static.
// ===========================================================================

// cdstruc.h: IRP_CONTEXT_FSP_FLAGS = WAIT|TOP_LEVEL|TOP_LEVEL_CDFS|IN_FSP.
//   0x04 | 0x10 | 0x20 | 0x40 = 0x74. Declared `extern static u32` by workque.rs.
#[no_mangle]
pub static IRP_CONTEXT_FSP_FLAGS: u32 = 0x04 | 0x10 | 0x20 | 0x40;

// cddata.c: LONG CdXAAudioPhileHeader[11] — RIFF/WAVE header for CD-XA audio.
#[no_mangle]
pub static mut CdXAAudioPhileHeader: [i32; 11] = [
    0x46464952,             // 'RIFF'
    -8,                     // chunk size (ADJUST1)
    0x45564157,             // 'WAVE'
    0x20746d66,             // 'fmt '
    16,                     // fmt chunk size
    0x00020001,             // FormatTag / nChannels
    44100,                  // nSamplesPerSecond
    2352 * 75,              // nAvgBytesPerSec
    0x00100004,             // nBlockAlign / nBitsPerSample
    0x61746164,             // 'data'
    -44,                    // raw data (ADJUST2)
];

// cddata.c: LONG CdXAFileHeader[11] — RIFF/CDXA header for CD-XA files.
#[no_mangle]
pub static mut CdXAFileHeader: [i32; 11] = [
    0x46464952,             // 'RIFF'
    -8,                     // chunk size (ADJUST)
    0x41584443,             // 'CDXA'
    0x20746d66,             // 'fmt '
    16,                     // CDXA chunk size
    0,                      // Owner ID
    0x41580000u32 as i32,   // Attributes / signature 'X','A'
    0,                      // File Number
    0,                      // Reserved[7] start
    0x61746164,             // 'data'
    -44,                    // raw sectors (ADJUST)
];

// ===========================================================================
// Batch 9 — seventh macro layer surfaced after batch 7/8 linked.
// ===========================================================================

use crate::types::CD_IO_CONTEXT;

const ACQUIRE_SHARED_STARVE_EXCLUSIVE: u32 = 2;
const TAG_IO_CONTEXT: u32 = 0x6f696443; // 'oidC'

// cdprocs.h:
//   #define CdAcquireFcbShared(IC,F,I)
//       CdAcquireResource((IC), &(F)->FcbNonpaged->FcbResource, (I), AcquireShared)
// stdcall (@12).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireFcbShared(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    IgnoreWait: BOOLEAN,
) -> BOOLEAN {
    CdAcquireResource(
        IrpContext,
        core::ptr::addr_of_mut!((*(*Fcb).FcbNonpaged).FcbResource) as PERESOURCE,
        IgnoreWait,
        ACQUIRE_SHARED,
    )
}

// cdprocs.h:
//   #define CdAcquireFileExclusive(IC,F)
//       CdAcquireResource((IC), (F)->Resource, FALSE, AcquireExclusive)
//   (F)->Resource is the FSRTL advanced-header Resource (FCB anon_1 union).
// stdcall (@8).
#[inline]
unsafe fn fcb_resource(Fcb: *mut FCB) -> PERESOURCE {
    (*Fcb).__bindgen_anon_1.Header.as_ref().Resource as PERESOURCE
}
#[no_mangle]
pub unsafe extern "C" fn CdAcquireFileExclusive(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    CdAcquireResource(IrpContext, fcb_resource(Fcb), 0, ACQUIRE_EXCLUSIVE);
}

// cdprocs.h:
//   #define CdAcquireFileSharedStarveExclusive(IC,F)
//       CdAcquireResource((IC), (F)->Resource, FALSE, AcquireSharedStarveExclusive)
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdAcquireFileSharedStarveExclusive(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {
    CdAcquireResource(IrpContext, fcb_resource(Fcb), 0, ACQUIRE_SHARED_STARVE_EXCLUSIVE);
}

// cdstruc.h:
//   #define CdAllocateIoContext()
//       FsRtlAllocatePoolWithTag(CdNonPagedPool, sizeof(CD_IO_CONTEXT), TAG_IO_CONTEXT)
// stdcall (@0).
#[no_mangle]
pub unsafe extern "C" fn CdAllocateIoContext() -> *mut CD_IO_CONTEXT {
    FsRtlAllocatePoolWithTag(
        CD_NONPAGED_POOL,
        core::mem::size_of::<CD_IO_CONTEXT>() as u32,
        TAG_IO_CONTEXT,
    ) as *mut CD_IO_CONTEXT
}

// cdprocs.h:
//   #define CdDecrementCleanupCounts(IC,F) {
//       (F)->FcbCleanup -= 1; (F)->Vcb->VcbCleanup -= 1; }
// stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn CdDecrementCleanupCounts(_IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) {
    (*Fcb).FcbCleanup -= 1;
    (*(*Fcb).Vcb).VcbCleanup -= 1;
}

// exfuncs.h: #define ExGetCurrentResourceThread() ((ULONG_PTR)PsGetCurrentThread())
// stdcall (@0) -> ERESOURCE_THREAD (ptr-sized).
#[no_mangle]
pub unsafe extern "C" fn ExGetCurrentResourceThread() -> *mut c_void {
    PsGetCurrentThread() as *mut c_void
}

// iofuncs.h: #define IoIsErrorUserInduced(Status)  one of a set of media statuses.
// stdcall (@4) -> BOOLEAN.
const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
const STATUS_MEDIA_WRITE_PROTECTED: i32 = 0xC00000A2u32 as i32;
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
const STATUS_UNRECOGNIZED_MEDIA: i32 = 0xC0000014u32 as i32;
const STATUS_WRONG_VOLUME: i32 = 0xC0000012u32 as i32;
#[no_mangle]
pub unsafe extern "system" fn IoIsErrorUserInduced(Status: i32) -> BOOLEAN {
    match Status {
        s if s == STATUS_DEVICE_NOT_READY
            || s == STATUS_IO_TIMEOUT
            || s == STATUS_MEDIA_WRITE_PROTECTED
            || s == STATUS_NO_MEDIA_IN_DEVICE
            || s == STATUS_VERIFY_REQUIRED
            || s == STATUS_UNRECOGNIZED_MEDIA
            || s == STATUS_WRONG_VOLUME =>
        {
            1
        }
        _ => 0,
    }
}

// FsRtlDismountComplete(TargetDeviceObject, Status) — a real FsRtl notification API
// that THIS ntoskrnl build does not export (only FsRtlNotifyVolumeEvent@8 exists;
// FsRtlDismountComplete is absent from ntoskrnl.spec). It is a fire-and-forget
// post-dismount notification with no return value; provide a no-op stub so the
// dismount path links and behaves as on the older ntoskrnl that predates the API.
// (Reported to the user — ideally ntoskrnl should export the real one.) stdcall (@8).
#[no_mangle]
pub unsafe extern "C" fn FsRtlDismountComplete(
    _TargetDeviceObject: *mut c_void,
    _Status: i32,
) {
}

// ---------------------------------------------------------------------------
// Final macro layer: arithmetic + SAL/SEH no-ops missed by earlier batches.
// All extern "C" (cdfs-internal; abi_normalize keeps def + decls consistent).
// ---------------------------------------------------------------------------

// cdprocs.h: #define BlockAlign(V,L) (((ULONG)(L) + (V)->BlockMask) & (V)->BlockInverseMask)
#[no_mangle]
pub unsafe extern "C" fn BlockAlign(Vcb: *mut VCB, Length: u32) -> u32 {
    (Length.wrapping_add((*Vcb).BlockMask)) & (*Vcb).BlockInverseMask
}

// cdprocs.h: #define LlSectorTruncate(L) (((LONGLONG)(L)) & ~(SECTOR_SIZE - 1))
#[no_mangle]
pub unsafe extern "C" fn LlSectorTruncate(L: i64) -> i64 {
    L & !((SECTOR_SIZE as i64) - 1)
}


// PSEH: under DUMMY_PSEH the filter never runs; return NULL exception info.
#[no_mangle]
pub unsafe extern "C" fn _SEH2_GetExceptionInformation() -> *mut c_void {
    null_mut()
}
