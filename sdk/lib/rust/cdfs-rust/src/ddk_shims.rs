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
    pub fn FsRtlAddToTunnelCache(
        Cache: PTUNNEL, DirectoryKey: ULONGLONG, ShortName: PUNICODE_STRING,
        LongName: PUNICODE_STRING, KeyByShortName: BOOLEAN, DataLength: ULONG,
        Data: PVOID);
    pub fn FsRtlAllocatePoolWithQuotaTag(
        PoolType: POOL_TYPE, NumberOfBytes: ULONG, Tag: ULONG) -> PVOID;
    pub fn FsRtlAllocatePoolWithTag(
        PoolType: POOL_TYPE, NumberOfBytes: ULONG, Tag: ULONG) -> PVOID;
    pub fn FsRtlAreNamesEqual(
        Name1: PCUNICODE_STRING, Name2: PCUNICODE_STRING, IgnoreCase: BOOLEAN,
        UpcaseTable: PCWCH) -> BOOLEAN;
    pub fn FsRtlBalanceReads(TargetDevice: PDEVICE_OBJECT) -> NTSTATUS;
    pub fn FsRtlCheckOplock(
        Oplock: POPLOCK, Irp: PIRP, Context: PVOID,
        CompletionRoutine: POPLOCK_WAIT_COMPLETE_ROUTINE,
        PostIrpRoutine: POPLOCK_FS_PREPOST_IRP) -> NTSTATUS;
    pub fn FsRtlCopyRead(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        Wait: BOOLEAN, LockKey: ULONG, Buffer: PVOID, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlCopyWrite(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        Wait: BOOLEAN, LockKey: ULONG, Buffer: PVOID, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlCurrentBatchOplock(Oplock: POPLOCK) -> BOOLEAN;
    pub fn FsRtlDeleteKeyFromTunnelCache(Cache: PTUNNEL, DirectoryKey: ULONGLONG);
    pub fn FsRtlDeleteTunnelCache(Cache: PTUNNEL);
    pub fn FsRtlDoesNameContainWildCards(Name: PUNICODE_STRING) -> BOOLEAN;
    pub fn FsRtlFastCheckLockForRead(
        FileLock: PFILE_LOCK, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER,
        Key: ULONG, FileObject: PFILE_OBJECT, Process: PVOID) -> BOOLEAN;
    pub fn FsRtlFastCheckLockForWrite(
        FileLock: PFILE_LOCK, FileOffset: PLARGE_INTEGER, Length: PLARGE_INTEGER,
        Key: ULONG, FileObject: PFILE_OBJECT, Process: PVOID) -> BOOLEAN;
    pub fn FsRtlFastUnlockAll(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, Process: PEPROCESS,
        Context: PVOID) -> NTSTATUS;
    pub fn FsRtlFastUnlockAllByKey(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, Process: PEPROCESS,
        Key: ULONG, Context: PVOID) -> NTSTATUS;
    pub fn FsRtlFastUnlockSingle(
        FileLock: PFILE_LOCK, FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER,
        Length: PLARGE_INTEGER, Process: PEPROCESS, Key: ULONG, Context: PVOID,
        AlreadySynchronized: BOOLEAN) -> NTSTATUS;
    pub fn FsRtlFindInTunnelCache(
        Cache: PTUNNEL, DirectoryKey: ULONGLONG, Name: PUNICODE_STRING,
        ShortName: PUNICODE_STRING, LongName: PUNICODE_STRING, DataLength: PULONG,
        Data: PVOID) -> BOOLEAN;
    pub fn FsRtlGetNextLargeMcbEntry(
        Mcb: PLARGE_MCB, RunIndex: ULONG, Vbn: PLONGLONG, Lbn: PLONGLONG,
        SectorCount: PLONGLONG) -> BOOLEAN;
    pub fn FsRtlInitializeFileLock(
        FileLock: PFILE_LOCK, CompleteLockIrpRoutine: PCOMPLETE_LOCK_IRP_ROUTINE,
        UnlockRoutine: PUNLOCK_ROUTINE);
    pub fn FsRtlInitializeLargeMcb(Mcb: PLARGE_MCB, PoolType: POOL_TYPE);
    pub fn FsRtlInitializeOplock(Oplock: POPLOCK);
    pub fn FsRtlInitializeTunnelCache(Cache: PTUNNEL);
    pub fn FsRtlIsNameInExpression(
        Expression: PUNICODE_STRING, Name: PUNICODE_STRING, IgnoreCase: BOOLEAN,
        UpcaseTable: PWCHAR) -> BOOLEAN;
    pub fn FsRtlIsNtstatusExpected(Ntstatus: NTSTATUS) -> BOOLEAN;
    pub fn FsRtlIsTotalDeviceFailure(Status: NTSTATUS) -> BOOLEAN;
    pub fn FsRtlLookupLargeMcbEntry(
        Mcb: PLARGE_MCB, Vbn: LONGLONG, Lbn: PLONGLONG, SectorCountFromLbn: PLONGLONG,
        StartingLbn: PLONGLONG, SectorCountFromStartingLbn: PLONGLONG,
        Index: PULONG) -> BOOLEAN;
    pub fn FsRtlLookupLastLargeMcbEntry(
        Mcb: PLARGE_MCB, Vbn: PLONGLONG, Lbn: PLONGLONG) -> BOOLEAN;
    pub fn FsRtlMdlReadCompleteDev(
        FileObject: PFILE_OBJECT, MdlChain: PMDL, DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlMdlReadDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        LockKey: ULONG, MdlChain: *mut PMDL, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlMdlWriteCompleteDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, MdlChain: PMDL,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlNotifyFullChangeDirectory(
        NotifySync: PNOTIFY_SYNC, NotifyList: PLIST_ENTRY, FsContext: PVOID,
        FullDirectoryName: PSTRING, WatchTree: BOOLEAN, IgnoreBuffer: BOOLEAN,
        CompletionFilter: ULONG, NotifyIrp: PIRP,
        TraverseCallback: PCHECK_FOR_TRAVERSE_ACCESS,
        SubjectContext: PSECURITY_SUBJECT_CONTEXT);
    pub fn FsRtlNotifyInitializeSync(NotifySync: *mut PNOTIFY_SYNC);
    pub fn FsRtlNotifyUninitializeSync(NotifySync: *mut PNOTIFY_SYNC);
    pub fn FsRtlNotifyVolumeEvent(FileObject: PFILE_OBJECT, EventCode: ULONG) -> NTSTATUS;
    pub fn FsRtlNumberOfRunsInLargeMcb(Mcb: PLARGE_MCB) -> ULONG;
    pub fn FsRtlOplockIsFastIoPossible(Oplock: POPLOCK) -> BOOLEAN;
    pub fn FsRtlPostPagingFileStackOverflow(
        Context: PVOID, Event: PKEVENT,
        StackOverflowRoutine: PFSRTL_STACK_OVERFLOW_ROUTINE);
    pub fn FsRtlPrepareMdlWriteDev(
        FileObject: PFILE_OBJECT, FileOffset: PLARGE_INTEGER, Length: ULONG,
        LockKey: ULONG, MdlChain: *mut PMDL, IoStatus: PIO_STATUS_BLOCK,
        DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FsRtlProcessFileLock(
        FileLock: PFILE_LOCK, Irp: PIRP, Context: PVOID) -> NTSTATUS;
    pub fn FsRtlRegisterFileSystemFilterCallbacks(
        FilterDriverObject: *mut _DRIVER_OBJECT,
        Callbacks: PFS_FILTER_CALLBACKS) -> NTSTATUS;
    pub fn FsRtlRemoveLargeMcbEntry(Mcb: PLARGE_MCB, Vbn: LONGLONG, SectorCount: LONGLONG);
    pub fn FsRtlTeardownPerStreamContexts(AdvancedHeader: PFSRTL_ADVANCED_FCB_HEADER);
    pub fn FsRtlTruncateLargeMcb(Mcb: PLARGE_MCB, Vbn: LONGLONG);
    pub fn FsRtlUninitializeFileLock(FileLock: PFILE_LOCK);
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
const MDL_PARTIAL_HAS_BEEN_MAPPED: u16 = 0x0004;

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
    pub fn FsRtlFreeFileLock(FileLock: PFILE_LOCK);
    pub fn FsRtlIsFatDbcsLegal(DbcsName: ANSI_STRING, WildCardsPermissible: BOOLEAN,
        PathNamePermissible: BOOLEAN, LeadingBackslashPermissible: BOOLEAN) -> BOOLEAN;
    pub fn FsRtlNormalizeNtstatus(Exception: NTSTATUS, GenericException: NTSTATUS) -> NTSTATUS;
    pub fn FsRtlNotifyCleanup(NotifySync: PNOTIFY_SYNC, NotifyList: PLIST_ENTRY, FsContext: PVOID);
    pub fn FsRtlUninitializeOplock(Oplock: POPLOCK);
}

//  wdm.h IoCompleteRequest macro -> IofCompleteRequest (fastcall on i386).
#[inline]
pub unsafe fn IoCompleteRequest(Irp: PIRP, PriorityBoost: CCHAR) {
    IofCompleteRequest(Irp, PriorityBoost);
}

//  ntddk.h IoIsErrorUserInduced(Status): the user-induced media error set.
#[inline]
pub fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN {
    const SET: [NTSTATUS; 7] = [
        0xC00000A3u32 as i32, // STATUS_DEVICE_NOT_READY
        0xC00000B5u32 as i32, // STATUS_IO_TIMEOUT
        0xC00000A2u32 as i32, // STATUS_MEDIA_WRITE_PROTECTED
        0xC0000013u32 as i32, // STATUS_NO_MEDIA_IN_DEVICE
        0x80000016u32 as i32, // STATUS_VERIFY_REQUIRED
        0xC0000014u32 as i32, // STATUS_UNRECOGNIZED_MEDIA
        0xC0000012u32 as i32, // STATUS_WRONG_VOLUME
    ];
    (SET.contains(&Status)) as BOOLEAN
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
