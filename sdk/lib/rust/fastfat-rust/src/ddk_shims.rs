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
#[no_mangle]
pub unsafe extern "C" fn MmPrepareMdlForReuse(Mdl: PMDL) {
    ntkrnl_sys::inline::mm::MmPrepareMdlForReuse(Mdl as _)
}

//  ARGUMENT_PRESENT macro: (ptr) != NULL
#[inline]
pub fn ARGUMENT_PRESENT<T>(p: *const T) -> bool {
    !p.is_null()
}

// ---------------------------------------------------------------------------
// i386 compatibility shims — types/functions that exist on amd64/arm64 bindgen
// but are absent from the i386 bindgen surface.  Provide them here so callers
// can import from crate::ddk_shims on all arches.
// ---------------------------------------------------------------------------

// IO_COMPLETION_ROUTINE: Option<unsafe fn(DevObj, Irp, Context) -> NTSTATUS>
// On amd64/arm64 bindgen: IO_COMPLETION_ROUTINE (extern "C")
// On i386 bindgen: PIO_COMPLETION_ROUTINE (extern "stdcall")
// Both are the same Option<fn> type; alias the i386 name to the non-P name.
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::IO_COMPLETION_ROUTINE;
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE as IO_COMPLETION_ROUTINE;

// ExInterlockedAddUlong: real export on i386 ntoskrnl (absent from i386 bindgen).
// Verified: ntoskrnl.spec / windows DDK i386 header.
#[cfg(target_arch = "x86")]
extern "system" {
    pub fn ExInterlockedAddUlong(Addend: *mut u32, Increment: u32, Lock: *mut u32) -> u32;
}
// On amd64/arm64 it's in the bindgen; re-export from there.
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::ExInterlockedAddUlong;

// IoIs32bitProcess: only meaningful on 64-bit OS (WOW64).
// On amd64 it's in the bindgen. On i386 all processes are native 32-bit,
// so this path is unreachable — return 0 (= not a 32-bit-on-64-bit process).
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::IoIs32bitProcess;
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn IoIs32bitProcess(_Irp: *mut ntkrnl_sys::ddk::_IRP) -> ntkrnl_sys::ddk::BOOLEAN {
    0  // i386 is always native 32-bit; WOW64 path is dead
}

// MOVE_FILE_DATA32 / PMOVE_FILE_DATA32 / PMARK_HANDLE_INFO32:
// WOW64 thunk structures — only exist on 64-bit kernels in the amd64 bindgen.
// On i386 the WOW64 ioctl paths are unreachable (no 32-bit-on-32-bit thunking).
// Provide zero-sized placeholder types so code compiles; the paths are dead.
#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct MOVE_FILE_DATA32 { _opaque: [u8; 0] }
#[cfg(target_arch = "x86")]
pub type PMOVE_FILE_DATA32 = *mut MOVE_FILE_DATA32;
#[cfg(target_arch = "x86")]
pub type PMARK_HANDLE_INFO32 = *mut u8;  // placeholder, dead path
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::{MOVE_FILE_DATA32, PMOVE_FILE_DATA32, PMARK_HANDLE_INFO32};
