//! ONE declaration per symbol fastfat links against outside its own Rust.
//!
//! Each module used to declare these itself, each with its own guess at the signature, and
//! rustc's `clashing_extern_declarations` counted the disagreements (a u16 return for a
//! ULONG, a PVOID where the C takes an int, parameters renamed into the wrong slot). The
//! kernel exports come from ntkrnl-sys, which is bindgen of the real headers; the rest are
//! fastfat's C cohab shims (rust_cohab_stub.c / rust_cohab_fat.c), transcribed here from
//! their definitions. Those are compiled `-mrtd` on i386, so every non-variadic one is
//! stdcall: `extern "system"`.

use core::ffi::c_void;
use crate::types::{BIOS_PARAMETER_BLOCK, LBO, PBCB, PFCB, PIRP_CONTEXT, PVCB};
use ntkrnl_sys::ddk::{
    BOOLEAN, LARGE_INTEGER, NTSTATUS, OEM_STRING, PDEVICE_OBJECT, PEPROCESS, PFAST_MUTEX,
    PIRP, POPLOCK, POPLOCK_FS_PREPOST_IRP, POPLOCK_WAIT_COMPLETE_ROUTINE, UNICODE_STRING,
    PVOID, ULONG, ULONGLONG,
};

// Kernel exports, as the headers declare them.
pub use ntkrnl_sys::ddk::{
    CcFlushCache, CcPurgeCacheSection, CcRepinBcb, CcSetDirtyPinnedData, CcSetFileSizes,
    CcUninitializeCacheMap, CcUnpinData, CcUnpinRepinnedBcb, ExAcquireResourceExclusiveLite,
    ExAcquireResourceSharedLite, ExIsResourceAcquiredExclusiveLite, IoRemoveShareAccess,
    IoSetCompletionRoutine, IoSetTopLevelIrp, KeCancelTimer, KeRemoveQueueDpc,
    KeWaitForSingleObject, MmFlushImageSection, PsGetCurrentThread,
};
pub use ntkrnl_sys::driver_ddk::DbgPrint;

// FASTCALL kernel exports (ntoskrnl.spec `@ fastcall`): on i386 a "system" declaration asks
// the linker for `_Name@4`; everywhere else FASTCALL is the ordinary convention.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn ExAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    pub fn ExReleaseFastMutexUnsafe(FastMutex: PFAST_MUTEX);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn ExAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    pub fn ExReleaseFastMutexUnsafe(FastMutex: PFAST_MUTEX);
}

extern "system" {
    // rust_cohab_stub.c
    // ntoskrnl RtlAssert (PVOID, PVOID, ULONG, PCHAR): read-only, so the literals pass as-is.
    pub fn RtlAssert(
        FailedAssertion: *const c_void,
        FileName: *const c_void,
        LineNumber: ULONG,
        Message: *const i8,
    );
    pub fn FatBugCheck(A: usize, B: usize, C: usize) -> !; // ULONG_PTR
    pub fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    pub fn FatVcbAcquiredExclusive(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    pub fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN;
    pub fn FatReleaseFcb(IrpContext: PIRP_CONTEXT, Fcb: PFCB);
    pub fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    pub fn FatAcquireExclusiveGlobal(IrpContext: PIRP_CONTEXT) -> BOOLEAN;
    pub fn FatReleaseGlobal(IrpContext: PIRP_CONTEXT);
    pub fn FatIsIoRangeValid(Vcb: PVCB, Start: LARGE_INTEGER, Length: ULONG) -> BOOLEAN;
    pub fn FAT_LFN_DIRENTS_NEEDED(Name: *const UNICODE_STRING) -> ULONG;
    pub fn CanFsdWait(Irp: PIRP) -> BOOLEAN;
    pub fn ExGetCurrentResourceThread() -> i64;
    pub fn FsRtlCheckOplockEx(
        Oplock: POPLOCK,
        Irp: PIRP,
        Flags: ULONG,
        Context: PVOID,
        CompletionRoutine: POPLOCK_WAIT_COMPLETE_ROUTINE,
        PostIrpRoutine: POPLOCK_FS_PREPOST_IRP,
    ) -> NTSTATUS;
    pub fn PsUpdateDiskCounters(
        Process: PEPROCESS,
        ReadBytes: u64,
        WriteBytes: u64,
        ReadOps: ULONG,
        WriteOps: ULONG,
        FlushOps: ULONG,
    );

    // rust_cohab_fat.c: the fatprocs.h macros, compiled against the real layouts
    pub fn FatBytesPerCluster(Bpb: *const BIOS_PARAMETER_BLOCK) -> ULONG;
    pub fn FatBytesPerFat(Bpb: *const BIOS_PARAMETER_BLOCK) -> ULONG;
    pub fn FatReservedBytes(Bpb: *const BIOS_PARAMETER_BLOCK) -> ULONG;
    pub fn FatRootDirectoryLbo(Bpb: *const BIOS_PARAMETER_BLOCK) -> LBO;
    pub fn FatRootDirectorySize(Bpb: *const BIOS_PARAMETER_BLOCK) -> ULONG;
    pub fn FatDeviceIsFatFsdo(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN;
    pub fn FatDirectoryKey(FcbOrDcb: PFCB) -> ULONGLONG;
    pub fn FatGetFcbOplock(Fcb: PFCB) -> POPLOCK;
    pub fn FatGetIndexFromLbo(Vcb: PVCB, Lbo: LBO) -> ULONG;
    pub fn FatIsFat32(Vcb: PVCB) -> BOOLEAN;
    pub fn FatUnpinBcb(IrpContext: PIRP_CONTEXT, Bcb: *mut PBCB);
    pub fn FatIsNameShortOemValid(
        IrpContext: PIRP_CONTEXT,
        OemName: OEM_STRING,
        CanContainWildCards: BOOLEAN,
        PathNameOk: BOOLEAN,
        LeadingBackslashOk: BOOLEAN,
    ) -> BOOLEAN;
    pub fn FsRtlIsAnsiCharacterLegalFat(Char: u8, WildCardsPermissible: BOOLEAN) -> BOOLEAN;
}
