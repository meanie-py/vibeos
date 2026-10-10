//! Macro / FORCEINLINE shims (built during repair; see disk-rust/mountmgr-rust).
//!
//! Two layers:
//!  1. extern "system" decls for real ntoskrnl exports missing from the kernel
//!     bindgen surface (FsRtl* live in ntifs.h, which bindgen doesn't ingest).
//!     All verified against ntoskrnl/ntoskrnl.spec.
//!  2. #[no_mangle] / pub fn implementations of header macros and FORCEINLINEs
//!     that are NOT exports (FsRtlEnterFileSystem, MmPrepareMdlForReuse, ...).
#![allow(non_snake_case, dead_code)]

use ntkrnl_sys::ddk::{
    BOOLEAN, LONGLONG, NTSTATUS, PCHECK_FOR_TRAVERSE_ACCESS, PCOMPLETE_LOCK_IRP_ROUTINE,
    PCUNICODE_STRING, PCWCH, PDEVICE_OBJECT, PEPROCESS, PFILE_LOCK, PFILE_OBJECT,
    PFSRTL_ADVANCED_FCB_HEADER, PFSRTL_STACK_OVERFLOW_ROUTINE, PFS_FILTER_CALLBACKS,
    PIO_STATUS_BLOCK, PIRP, PKEVENT, PLARGE_INTEGER, PLARGE_MCB, PLIST_ENTRY, PLONGLONG,
    PMDL, PNOTIFY_SYNC, POOL_TYPE, POPLOCK, POPLOCK_FS_PREPOST_IRP,
    POPLOCK_WAIT_COMPLETE_ROUTINE, PSECURITY_SUBJECT_CONTEXT, PSTRING, PTUNNEL, PULONG,
    PUNICODE_STRING, PUNLOCK_ROUTINE, PVOID, PWCHAR, ULONG, ULONGLONG,
    _DRIVER_OBJECT,
};
use ntkrnl_sys::ddk::{IofCallDriver, KeEnterCriticalRegion, KeLeaveCriticalRegion,
    MmUnmapLockedPages};

// ---------------------------------------------------------------------------
// Layer 1: real ntoskrnl exports absent from bindgen (ntifs.h surface).
// Prototypes transcribed from sdk/include/xdk/fsrtlfuncs.h + ntifs.template.h.
// ---------------------------------------------------------------------------
extern "system" {
    pub fn FsRtlAddLargeMcbEntry(
        Mcb: PLARGE_MCB, Vbn: LONGLONG, Lbn: LONGLONG, SectorCount: LONGLONG) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlAddToTunnelCache(
        Cache: PTUNNEL, DirectoryKey: ULONGLONG, ShortName: PUNICODE_STRING,
        LongName: PUNICODE_STRING, KeyByShortName: BOOLEAN, DataLength: ULONG,
        Data: PVOID);
}
extern "system" {
    pub fn FsRtlAllocatePoolWithQuotaTag(
        PoolType: POOL_TYPE, NumberOfBytes: ULONG, Tag: ULONG) -> PVOID;
}
extern "system" {
    pub fn FsRtlAllocatePoolWithTag(
        PoolType: POOL_TYPE, NumberOfBytes: ULONG, Tag: ULONG) -> PVOID;
}
extern "system" {
    pub fn FsRtlAreNamesEqual(
        Name1: PCUNICODE_STRING, Name2: PCUNICODE_STRING, IgnoreCase: BOOLEAN,
        UpcaseTable: PCWCH) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlBalanceReads(TargetDevice: PDEVICE_OBJECT) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlCheckOplock(
        Oplock: POPLOCK, Irp: PIRP, Context: PVOID,
        CompletionRoutine: POPLOCK_WAIT_COMPLETE_ROUTINE,
        PostIrpRoutine: POPLOCK_FS_PREPOST_IRP) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlCopyRead(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        Wait: BOOLEAN, LockKey: ULONG, Buffer: PVOID, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlCopyWrite(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        Wait: BOOLEAN, LockKey: ULONG, Buffer: PVOID, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlCurrentBatchOplock(Oplock: POPLOCK) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlDeleteKeyFromTunnelCache(Cache: PTUNNEL, DirectoryKey: ULONGLONG);
}
extern "system" {
    pub fn FsRtlDeleteTunnelCache(Cache: PTUNNEL);
}
extern "system" {
    pub fn FsRtlDoesNameContainWildCards(Name: PUNICODE_STRING) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlFastCheckLockForRead(
        FileLock: PFILE_LOCK, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER,
        Key: ULONG, FileObject: PFILE_OBJECT, Process: PVOID) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlFastCheckLockForWrite(
        FileLock: PFILE_LOCK, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER,
        Key: ULONG, FileObject: PFILE_OBJECT, Process: PVOID) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlFastUnlockAll(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, Process: PEPROCESS,
        Context: PVOID) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlFastUnlockAllByKey(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, Process: PEPROCESS,
        Key: ULONG, Context: PVOID) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlFastUnlockSingle(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER,
        Length: PLARGE_INTEGER, Process: PEPROCESS, Key: ULONG, Context: PVOID,
        AlreadySynchronized: BOOLEAN) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlFindInTunnelCache(
        Cache: PTUNNEL, DirectoryKey: ULONGLONG, Name: PUNICODE_STRING,
        ShortName: PUNICODE_STRING, LongName: PUNICODE_STRING, DataLength: PULONG,
        Data: PVOID) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlGetNextLargeMcbEntry(
        Mcb: PLARGE_MCB, RunIndex: ULONG, Vbn: PLONGLONG, Lbn: PLONGLONG,
        SectorCount: PLONGLONG) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlInitializeFileLock(
        FileLock: PFILE_LOCK, CompleteLockIrpRoutine: PCOMPLETE_LOCK_IRP_ROUTINE,
        UnlockRoutine: PUNLOCK_ROUTINE);
}
extern "system" {
    pub fn FsRtlInitializeLargeMcb(Mcb: PLARGE_MCB, PoolType: POOL_TYPE);
}
extern "system" {
    pub fn FsRtlInitializeOplock(Oplock: POPLOCK);
}
extern "system" {
    pub fn FsRtlInitializeTunnelCache(Cache: PTUNNEL);
}
extern "system" {
    pub fn FsRtlIsNameInExpression(
        Expression: PUNICODE_STRING, Name: PUNICODE_STRING, IgnoreCase: BOOLEAN,
        UpcaseTable: PWCHAR) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlIsNtstatusExpected(Ntstatus: NTSTATUS) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlIsTotalDeviceFailure(Status: NTSTATUS) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlLookupLargeMcbEntry(
        Mcb: PLARGE_MCB, Vbn: LONGLONG, Lbn: PLONGLONG, SectorCountFromLbn: PLONGLONG,
        StartingLbn: PLONGLONG, SectorCountFromStartingLbn: PLONGLONG,
        Index: PULONG) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlLookupLastLargeMcbEntry(
        Mcb: PLARGE_MCB, Vbn: PLONGLONG, Lbn: PLONGLONG) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlMdlReadCompleteDev(
        FileObject: PFILE_OBJECT, MdlChain: PMDL, DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlMdlReadDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        LockKey: ULONG, MdlChain: *mut PMDL, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlMdlWriteCompleteDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, MdlChain: PMDL,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlNotifyFullChangeDirectory(
        NotifySync: PNOTIFY_SYNC, NotifyList: PLIST_ENTRY, FsContext: PVOID,
        FullDirectoryName: PSTRING, WatchTree: BOOLEAN, IgnoreBuffer: BOOLEAN,
        CompletionFilter: ULONG, NotifyIrp: PIRP,
        TraverseCallback: PCHECK_FOR_TRAVERSE_ACCESS,
        SubjectContext: PSECURITY_SUBJECT_CONTEXT);
}
extern "system" {
    pub fn FsRtlNotifyInitializeSync(NotifySync: *mut PNOTIFY_SYNC);
}
extern "system" {
    pub fn FsRtlNotifyUninitializeSync(NotifySync: *mut PNOTIFY_SYNC);
}
extern "system" {
    pub fn FsRtlNotifyVolumeEvent(FileObject: PFILE_OBJECT, EventCode: ULONG) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlNumberOfRunsInLargeMcb(Mcb: PLARGE_MCB) -> ULONG;
}
extern "system" {
    pub fn FsRtlOplockIsFastIoPossible(Oplock: POPLOCK) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlPostPagingFileStackOverflow(
        Context: PVOID, Event: PKEVENT,
        StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE);
}
extern "system" {
    pub fn FsRtlPrepareMdlWriteDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        LockKey: ULONG, MdlChain: *mut PMDL, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlProcessFileLock(
        FileLock: PFILE_LOCK, Irp: PIRP, Context: PVOID) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlRegisterFileSystemFilterCallbacks(
        FilterDriverObject: *mut _DRIVER_OBJECT,
        Callbacks: PFS_FILTER_CALLBACKS) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlRemoveLargeMcbEntry(Mcb: PLARGE_MCB, Vbn: LONGLONG, SectorCount: LONGLONG);
}
extern "system" {
    pub fn FsRtlTeardownPerStreamContexts(AdvancedHeader: PFSRTL_ADVANCED_FCB_HEADER);
}
extern "system" {
    pub fn FsRtlTruncateLargeMcb(Mcb: PLARGE_MCB, Vbn: LONGLONG);
}
extern "system" {
    pub fn FsRtlUninitializeFileLock(FileLock: PFILE_LOCK);
}
extern "system" {
    pub fn FsRtlUninitializeLargeMcb(Mcb: PLARGE_MCB);
}

// ---------------------------------------------------------------------------
// Layer 2: header macros / FORCEINLINEs (not exports).
// ---------------------------------------------------------------------------

//  fsrtlfuncs.h: #define FsRtlEnterFileSystem KeEnterCriticalRegion
#[inline]
pub unsafe fn FsRtlEnterFileSystem() {
    KeEnterCriticalRegion();
}

//  fsrtlfuncs.h: #define FsRtlExitFileSystem KeLeaveCriticalRegion
#[inline]
pub unsafe fn FsRtlExitFileSystem() {
    KeLeaveCriticalRegion();
}

//  Win8+ PCW disk-counter export; does not exist in ReactOS ntoskrnl.
//  fastfat only calls it under NTDDI >= WIN8 paths — no-op at our 0x502 profile.
#[inline]
pub unsafe fn FsRtlUpdateDiskCounters(_BytesRead: ULONGLONG, _BytesWritten: ULONGLONG) {}

//  wdm.h IoCallDriver macro -> IofCallDriver (fastcall on i386, handled by bindgen).
#[inline]
pub unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}

//  mmfuncs.h MmPrepareMdlForReuse macro. #[no_mangle] so the per-file
//  `extern "C" { fn MmPrepareMdlForReuse(...) }` decls resolve at link.
const MDL_PARTIAL_HAS_BEEN_MAPPED: u16 = 0x0020; // mmtypes.h:23 (was 0x0004 = MDL_SOURCE_IS_NONPAGED_POOL)

#[no_mangle]
pub unsafe extern "C" fn MmPrepareMdlForReuse(Mdl: PMDL) {
    if (*Mdl).MdlFlags as u16 & MDL_PARTIAL_HAS_BEEN_MAPPED != 0 {
        MmUnmapLockedPages((*Mdl).MappedSystemVa, Mdl);
    }
}

//  ARGUMENT_PRESENT macro: (ptr) != NULL
#[inline]
pub fn ARGUMENT_PRESENT<T>(p: *const T) -> bool {
    !p.is_null()
}

// ===========================================================================
// cdfs additions (shared base above is fastfat's verbatim).
// ===========================================================================
use ntkrnl_sys::ddk::{ANSI_STRING, CCHAR, IofCompleteRequest, PRTL_SPLAY_LINKS};

extern "system" {
    // Real ntoskrnl exports (ntifs.h) cdfs uses beyond fastfat's set.
    pub fn FsRtlCheckLockForReadAccess(FileLock: PFILE_LOCK, Irp: PIRP) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlFreeFileLock(FileLock: PFILE_LOCK);
}
extern "system" {
    pub fn FsRtlIsFatDbcsLegal(DbcsName: ANSI_STRING, WildCardsPermissible: BOOLEAN,
        PathNamePermissible: BOOLEAN, LeadingBackslashPermissible: BOOLEAN) -> BOOLEAN;
}
extern "system" {
    pub fn FsRtlNormalizeNtstatus(Exception: NTSTATUS, GenericException: NTSTATUS) -> NTSTATUS;
}
extern "system" {
    pub fn FsRtlNotifyCleanup(NotifySync: PNOTIFY_SYNC, NotifyList: PLIST_ENTRY, FsContext: PVOID);
}
extern "system" {
    pub fn FsRtlUninitializeOplock(Oplock: POPLOCK);
}

//  wdm.h IoCompleteRequest macro -> IofCompleteRequest (fastcall on i386).
#[inline]
pub unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR) {
    IofCompleteRequest(Irp, PriorityBoost);
}

//  ntddk.h IoIsErrorUserInduced(Status): the user-induced media error set.
#[inline]
pub fn IoIsErrorUserInduced(Status: NTSTATUS) -> u32 {
    const SET: [NTSTATUS; 7] = [
        0xC00000A3u32 as i32, // STATUS_DEVICE_NOT_READY
        0xC00000B5u32 as i32, // STATUS_IO_TIMEOUT
        0xC00000A2u32 as i32, // STATUS_MEDIA_WRITE_PROTECTED
        0xC0000013u32 as i32, // STATUS_NO_MEDIA_IN_DEVICE
        0x80000016u32 as i32, // STATUS_VERIFY_REQUIRED
        0xC0000014u32 as i32, // STATUS_UNRECOGNIZED_MEDIA
        0xC0000012u32 as i32, // STATUS_WRONG_VOLUME
    ];
    (SET.contains(&Status)) as u32
}

//  ntrtl.h splay-link FORCEINLINEs (operate on RTL_SPLAY_LINKS in place).
#[inline]
pub unsafe fn RtlInitializeSplayLinks(Links: PRTL_SPLAY_LINKS) {
    (*Links).Parent = Links;
    (*Links).LeftChild = core::ptr::null_mut();
    (*Links).RightChild = core::ptr::null_mut();
}
#[inline]
pub unsafe fn RtlInsertAsLeftChild(ParentLinks: PRTL_SPLAY_LINKS, ChildLinks: PRTL_SPLAY_LINKS) {
    (*ParentLinks).LeftChild = ChildLinks;
    (*ChildLinks).Parent = ParentLinks;
}
#[inline]
pub unsafe fn RtlInsertAsRightChild(ParentLinks: PRTL_SPLAY_LINKS, ChildLinks: PRTL_SPLAY_LINKS) {
    (*ParentLinks).RightChild = ChildLinks;
    (*ChildLinks).Parent = ParentLinks;
}

// ===========================================================================
// cdfs flag/debug macro shims. The grind declares these per-module as extern
// fns (consistent call shape: SetFlag(&mut x.Flags, F); FlagOn(x.Flags, F) != 0)
// but leaves them undefined at link. #[no_mangle] provides BOTH the path import
// (modules that call bare) AND the link symbol (modules with their own extern decl).
// ===========================================================================
use crate::types::{VCB, IRP_CONTEXT};

// MUST return a full-width zero-extended 0/1 (u32), NOT a u8 BOOLEAN. The grind
// declares these per-module with inconsistent return types — some `-> BOOLEAN`
// (u8, reads only `al`), others `-> i32`/`-> u32` (read the full `eax` via
// `testl %eax,%eax`). A u8 return emits `setne %al` and leaves the upper 24 bits
// of eax as garbage; a u32-declaring caller then sees a stale-nonzero "true" for a
// clear flag. That mis-fired the ALLOC_IO check in CdCleanupIrpContext and freed a
// stack IoContext → BUGCHECK 0x19. Returning u32 forces movzbl, making both `al`
// and `eax` valid for every caller. (i386-specific; harmless on amd64/arm64.)
#[no_mangle] pub extern "C" fn FlagOn(Flags: u32, Flag: u32) -> u32 { ((Flags & Flag) != 0) as u32 }
#[no_mangle] pub extern "C" fn BooleanFlagOn(Flags: u32, Flag: u32) -> u32 { ((Flags & Flag) != 0) as u32 }
#[no_mangle] pub unsafe extern "C" fn SetFlag(Flags: *mut u32, Flag: u32) { *Flags |= Flag; }
#[no_mangle] pub unsafe extern "C" fn ClearFlag(Flags: *mut u32, Flag: u32) { *Flags &= !Flag; }

// cd.h: CdMarkRealDevVerifyOk(DO) = ClearFlag(DO->Flags, DO_VERIFY_VOLUME)
const DO_VERIFY_VOLUME: u32 = 0x00000002;
#[no_mangle] pub unsafe extern "C" fn CdMarkRealDevVerifyOk(DevObj: PDEVICE_OBJECT) {
    (*DevObj).Flags &= !DO_VERIFY_VOLUME;
}

// Debug ASSERT macros — no-ops at retail (DBG=0). #[no_mangle] for the per-module externs.
#[no_mangle] pub extern "C" fn ASSERT_IRP(_Irp: PIRP) {}
#[no_mangle] pub extern "C" fn ASSERT_EXCLUSIVE_CDDATA() {}
#[no_mangle] pub unsafe extern "C" fn ASSERT_EXCLUSIVE_VCB(_Vcb: *mut VCB) {}
#[no_mangle] pub unsafe extern "C" fn ASSERT_OPTIONAL_IRP_CONTEXT(_IrpContext: *mut IRP_CONTEXT) {}

// cd.h: CdRaiseStatus(IC,S) = CdRaiseStatusEx(IC,S,FALSE,BugCheckFileId,__LINE__).
// CdRaiseStatusEx raises via the SEH path and does not return -> `!`. Centralized here
// so every module imports ONE symbol instead of a private per-module extern (which the
// resolver can't share -> "not accessible" cross-module errors).
extern "C" {
    pub fn CdRaiseStatusEx(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS,
        NormalizeStatus: BOOLEAN, FileId: u32, Line: u32);
}
#[no_mangle]
pub unsafe extern "C" fn CdRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS) -> ! {
    CdRaiseStatusEx(IrpContext, Status, 0, 0, 0);
    loop {}
}

/// read.c/write.c `SafeZeroMemory(IC, AT, BYTE_COUNT)`: zero part of the caller's buffer,
/// turning a fault on it into `CdRaiseStatus(STATUS_INVALID_USER_BUFFER)`. Expanded to a
/// bare RtlZeroMemory, a bad user buffer would reach CdExceptionFilter as an access
/// violation, which it treats as unexpected and bugchecks on.
pub unsafe fn SafeZeroMemory(IrpContext: *mut IRP_CONTEXT, At: *mut core::ffi::c_void, ByteCount: usize) {
    const STATUS_INVALID_USER_BUFFER: NTSTATUS = 0xC00000E8u32 as i32;
    let caught = ntkrnl_sys::seh::try_except(
        &mut (),
        |_| ntkrnl_sys::rtl::RtlZeroMemory(At, ByteCount),
        |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
    );
    if caught.is_err() {
        CdRaiseStatus(IrpContext, STATUS_INVALID_USER_BUFFER);
    }
}

// cd.h integer/pointer helper macros (generic; safe central shims).
//   BlockAlign(V,L) = ((V)+(L-1)) & ~(L-1)   ; LlBlockAlign is the i64 form.
//   Add2Ptr(P,I)    = (PUCHAR)(P) + (I)
#[inline] pub fn BlockAlign(v: u32, l: u32) -> u32 { (v.wrapping_add(l - 1)) & !(l - 1) }
#[inline] pub fn BlockAlignI64(v: i64, l: i64) -> i64 { (v.wrapping_add(l - 1)) & !(l - 1) }
#[inline] pub unsafe fn Add2Ptr(p: *mut core::ffi::c_void, i: usize) -> *mut core::ffi::c_void {
    (p as *mut u8).add(i) as *mut core::ffi::c_void
}

// ===========================================================================
// cdfs link shims (round 2): debug macros, FORCEINLINEs, and intrinsic-ish
// helpers the grind referenced per-module as extern "system"/"C" fns but left
// undefined. ABI/decoration chosen to match the linker's demanded symbol:
//   stdcall (_Name@N)  -> #[no_mangle] extern "system"
//   cdecl   (_Name)    -> #[no_mangle] extern "C"
// (#[no_mangle] + extern "system" on i686-gnu auto-decorates _Name@N; do NOT
//  use #[export_name], which double-decorates.)
// ===========================================================================
use ntkrnl_sys::ddk::{KeBugCheckEx, ExFreePool, KeInitializeEvent, PFAST_MUTEX,
    PSINGLE_LIST_ENTRY, _EVENT_TYPE_SynchronizationEvent, ULONG_PTR};


// wdm.h NT_ASSERT(exp) — retail no-op (the grind passes the predicate as i32).
#[no_mangle] pub extern "C" fn NT_ASSERT(_Condition: BOOLEAN) {}

// cdfs debug ASSERT macros — retail no-ops.
#[no_mangle] pub extern "C" fn ASSERT_OPTIONAL_IRP(_Irp: PIRP) {}
#[no_mangle] pub extern "C" fn AssertVerifyDevice(_IrpContext: *mut IRP_CONTEXT, _Status: i32) {}
#[no_mangle] pub extern "C" fn AssertVerifyDeviceIrp(_Irp: PIRP) {}

// nodetype.h: CdBugCheck(A,B,C) = KeBugCheckEx(CDFS_FILE_SYSTEM, BugCheckFileId|__LINE__, A,B,C).
//   CDFS_FILE_SYSTEM = 0x26 (sdk/include/reactos/mc/bugcodes.mc).
//   The per-call-site BugCheckFileId|__LINE__ param was collapsed away when the
//   grind lowered the macro to a 3-arg fn; param1 is passed 0 here (file/line lost,
//   acceptable on this never-returns crash path). Faithful to the KeBugCheckEx call.
const CDFS_FILE_SYSTEM: u32 = 0x26;
#[no_mangle] pub unsafe extern "C" fn CdBugCheck(A: u32, B: u32, C: u32) {
    KeBugCheckEx(CDFS_FILE_SYSTEM, 0 as ULONG_PTR, A as ULONG_PTR, B as ULONG_PTR, C as ULONG_PTR);
}

// cdprocs.h: CdFreePool(x) -> _CdFreePool((PVOID*)x): if *Pool != NULL { ExFreePool(*Pool); *Pool = NULL; }
#[no_mangle] pub unsafe extern "C" fn CdFreePool(Pool: *mut *mut core::ffi::c_void) {
    if !(*Pool).is_null() {
        ExFreePool(*Pool);
        *Pool = core::ptr::null_mut();
    }
}

// ntddk.h PopEntryList FORCEINLINE (bindgen declares but doesn't define). cdecl.
#[no_mangle] pub unsafe extern "C" fn PopEntryList(ListHead: PSINGLE_LIST_ENTRY) -> PSINGLE_LIST_ENTRY {
    let first = (*ListHead).Next;
    if !first.is_null() {
        (*ListHead).Next = (*first).Next;
    }
    first
}

// xdk/exfuncs.h ExInitializeFastMutex FORCEINLINE (bindgen declares but doesn't
// define). FM_LOCK_BIT = 1; KeInitializeEvent(&Event, SynchronizationEvent, FALSE). cdecl.
const FM_LOCK_BIT: i32 = 1;
#[no_mangle] pub unsafe extern "C" fn ExInitializeFastMutex(FastMutex: PFAST_MUTEX) {
    (*FastMutex).Count = FM_LOCK_BIT;
    (*FastMutex).Owner = core::ptr::null_mut();
    (*FastMutex).Contention = 0;
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FastMutex).Event),
        _EVENT_TYPE_SynchronizationEvent,
        0, // FALSE
    );
}

// ---------------------------------------------------------------------------
// cdfs link shims (round 3): more debug-macros / cdprocs.h FORCEINLINEs+macros
// referenced as stdcall fns (_Name@N) by the grind.
// ---------------------------------------------------------------------------
use ntkrnl_sys::ddk::{PIO_STACK_LOCATION, IoIsOperationSynchronous, BOOLEAN as DdkBOOLEAN,
    PERESOURCE};
use crate::types::{FCB, TYPE_OF_ACQUIRE, _TYPE_OF_ACQUIRE_AcquireShared, PIRP_CONTEXT};

// ntddk.h: UNREFERENCED_PARAMETER(P) — retail no-op (grind lowered to stdcall fn).

// cdfs debug ASSERT macros — retail no-ops.
#[no_mangle] pub extern "C" fn ASSERT_IRP_CONTEXT(_Irp: *mut IRP_CONTEXT) {}
#[no_mangle] pub extern "C" fn ASSERT_FILE_OBJECT(_FileObject: PFILE_OBJECT) {}

// wdm.h FORCEINLINE IoGetCurrentIrpStackLocation: Irp->Tail.Overlay.CurrentStackLocation.
// IoGetCurrentIrpStackLocation is a C FORCEINLINE (no real ntoskrnl export), so we
// provide the symbol here. The bindgen `ddk` declaration — used by the majority of
// cdfs callers and the canonical substrate — is `extern "C"` (cdecl), so this def
// MUST be cdecl too. A stdcall (`extern "system"`) def gave the cdecl bindgen
// `declare` and this `define` the same unmangled IR name with mismatched calling
// conventions in one codegen unit → LLVM treats the cdecl callsite as UB and emits
// `ud2` (STATUS_ILLEGAL_INSTRUCTION at runtime). Every local decl must match (cdecl).
#[no_mangle] pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(Irp)
}

// wdm.h FORCEINLINE IoMarkIrpPending — sets SL_PENDING_RETURNED on the current
// stack location. Not a real ntoskrnl export; bindgen declares it `extern "C"`,
// so this providing def must be cdecl too (same reasoning as above).
#[no_mangle] pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoMarkIrpPending(Irp)
}

// cdprocs.h: CanFsdWait(I) = IoIsOperationSynchronous(I).
#[no_mangle] pub unsafe extern "C" fn CanFsdWait(Irp: PIRP) -> DdkBOOLEAN {
    IoIsOperationSynchronous(Irp)
}

// cdprocs.h (NTDDI < WIN8): CdGetFcbOplock(F) = &(F)->Oplock.  Oplock is the
// first field of the FCB_DATA arm of the FCB's anonymous data union
// (__bindgen_anon_2 in types_<arch>.rs), so its address == &__bindgen_anon_2.
#[no_mangle] pub unsafe extern "C" fn CdGetFcbOplock(Fcb: *mut FCB) -> PVOID {
    core::ptr::addr_of_mut!((*Fcb).__bindgen_anon_2) as PVOID
}

// cdprocs.h: CdAcquireFileShared(IC,F) = CdAcquireResource(IC, (F)->Resource, FALSE, AcquireShared).
//   (F)->Resource lives in the FSRTL advanced header spliced as __bindgen_anon_1.Header.
#[no_mangle] pub unsafe extern "C" fn CdAcquireFileShared(IrpContext: PIRP_CONTEXT, Fcb: *mut FCB) {
    let resource = (*Fcb).__bindgen_anon_1.Header.as_ref().Resource as PERESOURCE;
    CdAcquireResource(IrpContext, resource, 0 /* FALSE */, _TYPE_OF_ACQUIRE_AcquireShared as TYPE_OF_ACQUIRE);
}

// CdAcquireResource: NTAPI cdfs internal (defined in resrcsup -> Rust). Declared in
// types_<arch>.rs as extern "system"; re-declare here for the shim above.
extern "C" {
    fn CdAcquireResource(IrpContext: PIRP_CONTEXT, Resource: PERESOURCE,
        IgnoreWait: DdkBOOLEAN, Type: TYPE_OF_ACQUIRE) -> DdkBOOLEAN;
}

// KeInitializeSpinLock is a real exported ntoskrnl function on i386 but a DDK
// FORCEINLINE on amd64/arm64 (no out-of-line symbol), so cdfs.sys fails to link
// there with `undefined symbol: KeInitializeSpinLock`. Provide the inline body
// (the spinlock is a single ULONG_PTR; init == zero it) on non-i386, matching the
// disk-rust / classpnp-rust / mountmgr-rust cohabs. i386 keeps the real export.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
