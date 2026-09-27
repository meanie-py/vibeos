use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CACHE_MANAGER_CALLBACKS, CCHAR, IO_STATUS_BLOCK, CC_FILE_SIZES, CLONG, CcCanIWrite, ObReferenceObjectByHandle, CcIsThereDirtyData, CcWaitForCurrentLazyWriterActivity, DEVICE_OBJECT, DISK_GEOMETRY, _KWAIT_REASON_Executive, ERESOURCE, FILESYSTEM_STATISTICS, FILE_LOCK, FILE_OBJECT, GUID, _IO_STATUS_BLOCK, IO_STACK_LOCATION, IoAcquireVpbSpinLock, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoCreateDevice, IoDeleteDevice, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoReleaseVpbSpinLock, IoSetTopLevelIrp, IRP, KEVENT, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, LARGE_INTEGER, LARGE_MCB, LUID, MARK_HANDLE_INFO, MOVE_FILE_DATA, _EVENT_TYPE_NotificationEvent, OEM_STRING, OPLOCK, PARTITION_INFORMATION_EX, PDEVICE_OBJECT, PETHREAD, PFILE_OBJECT, PFSCTL_QUERY_FAT_BPB_BUFFER, PHANDLE, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER, PLARGE_MCB, PLONGLONG, PMARK_HANDLE_INFO, PMOVE_FILE_DATA, PRETRIEVAL_POINTERS_BUFFER, PSTARTING_LCN_INPUT_BUFFER, PSTARTING_VCN_INPUT_BUFFER, PUINT32, PVOLUME_BITMAP_BUFFER, PVPB, RETRIEVAL_POINTERS_BUFFER, SECTION_OBJECT_POINTERS, STARTING_LCN_INPUT_BUFFER, STARTING_VCN_INPUT_BUFFER, SeSinglePrivilegeCheck, UINT32, _UNICODE_STRING, VOLUME_BITMAP_BUFFER, VPB};
use core::ffi::c_void;
// IoFileObjectType is a kernel DATA export; this local raw-dylib decl shadows the
// plain `ntkrnl_sys::mm::*` glob import so the reference goes through the __imp_ thunk.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static IoFileObjectType: *mut c_void;
}
use core::ptr::null_mut;
use core::ptr;
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlNotifyVolumeEvent, IoIs32bitProcess, MOVE_FILE_DATA32, PMOVE_FILE_DATA32, PMARK_HANDLE_INFO32, FsRtlAddLargeMcbEntry, FsRtlGetNextLargeMcbEntry, FsRtlInitializeLargeMcb, FsRtlLookupLargeMcbEntry, FsRtlNumberOfRunsInLargeMcb, FsRtlRemoveLargeMcbEntry, FsRtlUninitializeLargeMcb};
// Needed by the FSCTL handlers translated at the end of this file. Imported by
// EXPLICIT path rather than a glob: a glob here would put `VBO` (64-bit in
// ntkrnl_sys, ULONG32 in fastfat) and friends in scope to silently shadow the
// crate's own types — the class that already bit dirctrl.rs.
use crate::glue::{FSRTL_VOLUME_UNLOCK, VPB_DIRECT_WRITES_ALLOWED, VPB_LOCKED};
// MmGetSystemAddressForMdlSafe is a DDK **MACRO** (mmfuncs.h:139), not a function:
// it tests MdlFlags and either returns MappedSystemVa or calls
// MmMapLockedPagesSpecifyCache. Declaring it `extern` produces an undefined symbol
// at link — which is exactly what scan_bugcheck_stubs calls MACRO_AS_EXTERN, and
// what I did here before the linker caught it. ntkrnl_sys carries the faithful
// inline; use that.
use ntkrnl_sys::driver_ddk::MmGetSystemAddressForMdlSafe;
use crate::types::{
    CCB_FLAG_COMPLETE_DISMOUNT, FILE_SYSTEM_STATISTICS, IRP_CONTEXT_FLAG_DISABLE_POPUPS,
    VCB_STATE_FLAG_LOCKED, VCB_STATE_FLAG_MOUNTED_DIRTY, VCB_STATE_FLAG_VOLUME_DIRTY,
    VCB_STATE_FLAG_VOLUME_DISMOUNTED,
};
use crate::types::{BIOS_PARAMETER_BLOCK, CCB, DCB, DIRENT, FAT_BUG_CHECK_FSCTRL, FAT_DATA, FAT_ENTRY, FCB, FCB_CONDITION, FCB_STATE_DELETE_ON_CLOSE, IRP_CONTEXT, PACKED_BIOS_PARAMETER_BLOCK, PACKED_BOOT_SECTOR, PACKED_BOOT_SECTOR_EX, PBCB, PCCB, PDIRENT, PFCB, PIRP_CONTEXT, PVCB, VBO, VCB, VCB_CONDITION, VOLUME_DEVICE_OBJECT, _TYPE_OF_OPEN as TYPE_OF_OPEN, _TYPE_OF_OPEN_UserDirectoryOpen as UserDirectoryOpen, _TYPE_OF_OPEN_UserFileOpen as UserFileOpen};
// Additional imports needed by FatMountVolume (translated from fsctrl.c)
use ntkrnl_sys::ddk::ExReleaseResourceLite;
use crate::types::{PPACKED_BOOT_SECTOR, PPACKED_BOOT_SECTOR_EX, PVBO, PVOLUME_DEVICE_OBJECT, FatMaxLarge,
    FCB_STATE_SYSTEM_FILE, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE, VCB_STATE_FLAG_REMOVABLE_MEDIA,
    VCB_STATE_FLAG_VPB_MUST_BE_FREED, VCB_STATE_FLAG_WRITE_PROTECTED, VCB_STATE_VPB_NOT_ON_DEVICE,
    _VCB_CONDITION_VcbGood as VcbGood, _VCB_CONDITION_VcbBad as VcbBad, _VCB_CONDITION_VcbNotMounted as VcbNotMounted};

extern "system" {
    } extern "C" { fn DebugTrace(level: i32, component: u32, format: *const i8, ...); } extern "system" {
    fn FatUnpackBios(bpb: *mut BIOS_PARAMETER_BLOCK, packed_bpb: *const core::ffi::c_void);
    static mut FatData: FAT_DATA;
    fn FatCloseEaFile(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, f: u8);
    fn FatFlushVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, FlushType: u32) -> i32;
    fn FatHijackIrpAndFlushDevice(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, TargetDeviceObject: *mut DEVICE_OBJECT) -> i32;
    fn FatPurgeReferencedFileObjects(IrpContext: *mut IRP_CONTEXT, RootDcb: *mut DCB, FlushType: u32);
    fn FatMarkVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, VolumeState: i32);
    fn FatToggleMediaEjectDisable(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Disable: u8) -> u32;
    fn CcPurgeCacheSection(SectionObjectPointers: *mut SECTION_OBJECT_POINTERS, FileOffset: *const LARGE_INTEGER, Length: u32, UninitializeCache: i32) -> u8;
    fn KeCancelTimer(Timer: *mut KEVENT) -> u8;
    fn KeRemoveQueueDpc(Dpc: *mut KDPC) -> u8;
    fn ExIsResourceAcquiredExclusiveLite(Resource: *mut ERESOURCE) -> u8;
    fn PAGED_CODE();
    fn NT_ASSERT(condition: u32);
    fn FlagOn(Flags: u32, Flag: u32) -> BOOLEAN;
    fn FatAcquireExclusiveGlobal(IrpContext: *mut IRP_CONTEXT);
    fn FatCheckForDismount( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Ignore: BOOLEAN, ) -> BOOLEAN;
    fn FatReleaseGlobal(IrpContext: *mut IRP_CONTEXT);
    fn ExAcquireResourceExclusiveLite( Resource: *mut ERESOURCE, Wait: BOOLEAN, ) -> BOOLEAN;
    // ExReleaseResourceLite via ntkrnl_sys::ddk (kernel FASTCALL export — §9.99)
    fn FatDecodeFileObject( FileObject: *mut FILE_OBJECT, Vcb: *mut *mut VCB, FcbOrDcb: *mut *mut FCB, Ccb: *mut *mut CCB, ) -> TYPE_OF_OPEN;
    fn FatCompleteRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP, Status: NTSTATUS);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatVerifyFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatReleaseFcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatOpenDirectoryFile(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatAllocateDiskSpace(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, HintCluster: u32, ByteCount: *mut u32, AllocationHint: u8, /* BOOLEAN */ Mcb: *mut LARGE_MCB);
    fn FatGetIndexFromLbo(Vcb: *mut VCB, Lbo: LBO) -> u32;
    fn FatGetLboFromIndex(Vcb: *mut VCB, Index: u32) -> i64; // LBO = LONGLONG (fat.h:34)
    fn FatFlushFatEntries(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Cluster: u32, Count: u32);
    fn FatSetFatEntry( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, SourceCluster: u32, TargetCluster: FAT_ENTRY, );
    fn FatDeallocateDiskSpace(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Mcb: *mut LARGE_MCB, Wait: u8);
    fn FatUnpinRepinnedBcbs(IrpContext: *mut IRP_CONTEXT);
    fn FatFlushDirentForFile(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatSetDirtyBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut PBCB, Vcb: *mut VCB, Wait: u8);
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut PBCB);
    fn FatNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
    fn FatGetDirentFromFcbOrDcb( IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB, Wait: i32, /* BOOLEAN */ Dirent: *mut *mut DIRENT, DirentBcb: *mut PBCB, );
    fn DebugUnwind(Function: *const i8);
    fn ClearFlag(Flags: *mut u32, Mask: u32);
    fn LongToHandle(Value: LONG) -> HANDLE;
    fn PsGetCurrentThread() -> *mut PETHREAD;
    } extern "C" { fn DbgPrint(Format: *const i8, ...); } extern "system" {
    fn NodeType(Ptr: *mut core::ffi::c_void) -> u32;
}
extern "system" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
    fn FatLookupFileAllocationSize(IrpContext: PIRP_CONTEXT, FcbOrDcb: PFCB);
    fn FatIsFat32(Vcb: PVCB) -> u8;
    fn FatLookupFileAllocation( IrpContext: PIRP_CONTEXT, FcbOrDcb: PFCB, FileOffset: u32, RunLbo: *mut i64, RunByteCount: *mut u32, RunAllocated: *mut u8, RunEndOnMax: *mut u8, RunIndex: *mut u32, );
    fn FatAcquireSharedVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u32;
    fn FatQuickVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatReleaseVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn RtlEqualMemory(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, length: u32) -> u8;
    fn FatVerifyIndexIsValid(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, FatIndex: u32);
    fn FatReservedBytes(Bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatAcquireSharedFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8;
    fn FsRtlLookupLastLargeMcbEntryAndIndex(Mcb: *mut LARGE_MCB, LargeVbn: *mut LONGLONG, LargeLbn: *mut LONGLONG, Index: *mut ULONG) -> u8;
    fn FatGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
    fn FatIsFastIoPossible(Fcb: *mut FCB) -> BOOLEAN;
    fn FsRtlOplockFsctrl( Oplock: *mut OPLOCK, Irp: *mut IRP, OpenCount: ULONG, ) -> NTSTATUS;
    fn FsRtlOplockIsSharedRequest(Irp: *mut IRP) -> BOOLEAN;
    fn FsRtlCheckLockForOplockRequest( FileLock: *mut FILE_LOCK, AllocationSize: *const LARGE_INTEGER, ) -> BOOLEAN;
    fn FatBugCheck(param1: ULONG, param2: ULONG, param3: ULONG);
    fn BooleanFlagOn(Flags: u32, Bit: u32) -> bool;
    fn CcUninitializeCacheMap(FileObject: *mut FILE_OBJECT, TruncateSize: *const LARGE_INTEGER, UninitEvent: *mut c_void);
    fn CopyUchar4(dst: *mut u32, src: *const u8);
    fn ExFreePool(p: *mut c_void);
    fn FatAcquireExclusiveVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> BOOLEAN;
    fn FatAcquireExclusiveVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatBytesPerCluster(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatCheckDirtyBit(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatFlushFat(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> NTSTATUS;
    fn FatFsdPostRequest(IrpContext: *mut IRP_CONTEXT, Irp: *mut IRP) -> NTSTATUS;
    fn FatInterpretClusterType(Vcb: *mut VCB, Cluster: u32) -> FAT_CLUSTER_TYPE;
    fn FatMarkFcbCondition(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Condition: u32, Recursive: u32);
    fn FatPerformDevIoCtrl( IrpContext: *mut IRP_CONTEXT, IoControlCode: u32, TargetDeviceObject: *mut DEVICE_OBJECT, InputBuffer: *mut c_void, InputBufferLength: u32, OutputBuffer: *mut c_void, OutputBufferLength: u32, b1: bool, b2: bool, Iosb: *mut _IO_STATUS_BLOCK, ) -> NTSTATUS;
    fn FatReleaseVolume(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatRootDirectoryLbo(bpb: *const BIOS_PARAMETER_BLOCK) -> LARGE_INTEGER;
    fn FatRootDirectorySize(bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatScanForDataTrack(IrpContext: *mut IRP_CONTEXT, TargetDeviceObject: *mut DEVICE_OBJECT) -> bool;
    fn FatSetVcbCondition(Vcb: *mut VCB, Condition: VCB_CONDITION);
    fn FatSetupAllocationSupport(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatTearDownAllocationSupport(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FsRtlAllocatePoolWithTag(PoolType: u32, NumberOfBytes: u32, Tag: u32) -> *mut c_void;
    fn IsBpbFat32(bpb: *const PACKED_BIOS_PARAMETER_BLOCK) -> bool;
    fn KeGetCurrentThread() -> *mut c_void;
    // RtlZeroMemory: imported via ntkrnl_sys::rtl::*
    // FatLockVolume / FatLockVolumeInternal are DEFINED in Rust below.
    // FatUnlockVolume, FatDirtyVolume, FatIsVolumeMounted, FatIsPathnameValid and
    // FatQueryBpb are DEFINED in Rust below (each was a BUGCHECK stub).
    //
    // FatIsPathnameValid was declared here as `-> u8` while the C returns NTSTATUS,
    // and the dispatch arm cast it back with `as i32`. That only ever worked because
    // the function unconditionally returns STATUS_SUCCESS (0): any non-zero status
    // would have been truncated to its low byte on the way out. The Rust definition
    // returns NTSTATUS and the cast is gone.
    fn FatQueryRetrievalPointers(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    // verfysup.rs:484. Returns VOID — it FatRaiseStatus()es rather than reporting.
    fn FatVerifyVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatIsHandleCountZero(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u8;
    fn FatFspClose(Vcb: *mut VCB);
    fn FatGetVolumeBitmap(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn FatGetRetrievalPointers(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn RtlOemStringToCountedUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const OEM_STRING, AllocateDestinationString: BOOLEAN) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
#[cfg(debug_assertions)]
pub static mut FatMoveFileDebug: BOOLEAN = 0;

pub const BugCheckFileId: u32 = FAT_BUG_CHECK_FSCTRL;
pub const Dbg: u32 = DEBUG_TRACE_FSCTRL;

const IOCTL_DISK_IS_WRITABLE: u32 = 0x00070024;
const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
const STATUS_MEDIA_WRITE_PROTECTED: i32 = 0xC00000A2_u32 as i32;

const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;

const UserVolumeOpen: u32 = 4; // _TYPE_OF_OPEN_UserVolumeOpen
const CCB_FLAG_MANAGE_VOLUME_ACCESS: u32 = 0x00020000;
const CCB_FLAG_ALLOW_EXTENDED_DASD_IO: u32 = 0x2000;

const FAT_DIRENT_NEVER_USED: u8 = 0x00;
const FAT_DIRENT_ATTR_ARCHIVE: u8 = 0x20;
const FAT_DIRENT_ATTR_VOLUME_ID: u8 = 0x08;
const FAT_DIRENT_DELETED: u8 = 0xE5;
const FAT_DIRENT_REALLY_0E5: u8 = 0x05;

// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;
// fatprocs.h:47: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: u32 = 4;
const TAG_ENTRY_LOOKUP_BUFFER: u32 = 0x4C746146;

// fastfat's real LBO is LONGLONG (fat.h:34, and crate::types::LBO = LONGLONG), i.e.
// SIGNED. This local alias is unsigned. Same width, so unlike the VBO divergence in
// dirctrl.rs (4 bytes vs 8 — real corruption) this one is benign for every value a
// disk byte-offset actually takes, and the call sites in this file are written
// against the u64 spelling. Left as-is deliberately rather than churned late; it is
// recorded with its sibling on the VBO task. Do NOT copy this pattern: a per-file
// `type` alias that disagrees with crate::types is invisible at every use site.
type LBO = u64;

// Constants (values taken from Windows SDK definitions)
const REQUEST_OPLOCK_INPUT_FLAG_REQUEST: u32 = 0x1;
const REQUEST_OPLOCK_INPUT_FLAG_ACK: u32 = 0x2;
const OPLOCK_LEVEL_CACHE_HANDLE: u32 = 0x2;

const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000002;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)

// Missing types — defined locally from Windows SDK headers
type FAT_FLUSH_TYPE = i32;
type FAT_CLUSTER_TYPE = u32;

#[repr(C)]
struct REQUEST_OPLOCK_INPUT_BUFFER {
    pub StructureVersion: u16,
    pub StructureLength: u16,
    pub RequestedOplockLevel: u32,
    pub Flags: u32,
}

#[repr(C)]
struct REQUEST_OPLOCK_OUTPUT_BUFFER {
    pub StructureVersion: u16,
    pub StructureLength: u16,
    pub OriginalOplockLevel: u32,
    pub NewOplockLevel: u32,
    pub Flags: u32,
    pub AccessMode: u32,
    pub ShareMode: u16,
}

#[repr(C)]
struct RETRIEVAL_POINTER_BASE {
    pub FileAreaOffset: LARGE_INTEGER,
}

#[repr(C)]
struct BOOT_AREA_INFO {
    pub BootSectorCount: u32,
    pub BootSectors: [BootSectorEntry; 2],
}

#[repr(C)]
struct BootSectorEntry {
    pub Offset: LARGE_INTEGER,
}

#[repr(C)]
struct SET_PURGE_FAILURE_MODE_INPUT {
    pub Flags: u32,
}

const SET_PURGE_FAILURE_MODE_ENABLED: u32 = 0x1;
const SET_PURGE_FAILURE_MODE_DISABLED: u32 = 0x2;

const STATUS_WRONG_VOLUME: i32 = 0xC0000012_u32 as i32;
const FAT_CLUSTER_LAST: u32 = 0x0FFFFFFF; // fat.h:258
const FSCTL_LOCK_VOLUME: u32 = 0x00090018;
const FSCTL_UNLOCK_VOLUME: u32 = 0x0009001C;
const FSCTL_DISMOUNT_VOLUME: u32 = 0x00090020;
const FSCTL_MARK_VOLUME_DIRTY: u32 = 0x00090030;
const FSCTL_IS_VOLUME_DIRTY: u32 = 0x00090078;
const FSCTL_IS_VOLUME_MOUNTED: u32 = 0x00090028;
const FSCTL_IS_PATHNAME_VALID: u32 = 0x0009002C;
const FSCTL_QUERY_RETRIEVAL_POINTERS: u32 = 0x0009003B;
const FSCTL_QUERY_FAT_BPB: u32 = 0x00090058;
const FSCTL_FILESYSTEM_GET_STATISTICS: u32 = 0x00090060;
const FSCTL_GET_RETRIEVAL_POINTER_BASE: u32 = 0x00090234; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 141, METHOD_BUFFERED, FILE_ANY_ACCESS)
const FSCTL_GET_BOOT_AREA_INFO: u32 = 0x00090230;
const FSCTL_GET_VOLUME_BITMAP: u32 = 0x0009006F;
const FSCTL_GET_RETRIEVAL_POINTERS: u32 = 0x00090073;
const FSCTL_MOVE_FILE: u32 = 0x00090074; // CTL_CODE(FILE_DEVICE_FILE_SYSTEM, 29, METHOD_BUFFERED, FILE_ANY_ACCESS)
const FSCTL_ALLOW_EXTENDED_DASD_IO: u32 = 0x00090083;
const FSCTL_MARK_HANDLE: u32 = 0x000900FC;
const FSCTL_SET_PURGE_FAILURE_MODE: u32 = 0x00090270;
const FSCTL_SET_ZERO_ON_DEALLOCATION: u32 = 0x00090194;
const FSCTL_REQUEST_OPLOCK: u32 = 0x00090240;
#[no_mangle]
pub unsafe extern "system" fn FatNonSparseMcb(
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    Vbo: *mut VBO,
    ByteCount: *mut LONGLONG,
) -> i32 {
    let _ = Vcb; // UNREFERENCED_PARAMETER(Vcb)
    let mut Lbo: LONGLONG = 0;
    let mut Index: u32 = 0;
    let mut llVbo: LONGLONG = 0;

    while FsRtlGetNextLargeMcbEntry(Mcb, Index, &mut llVbo, &mut Lbo, ByteCount) != 0 {
        *Vbo = llVbo as VBO;
        if (Lbo as u32) == (-1i32 as u32) {
            return 0;
        }
        Index += 1;
    }

    *Vbo = llVbo as VBO;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FatAddMcbEntry(
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    mut Vbo: u32,
    // C signature: `LBO Lbo` where LBO = LONGLONG (fat.h:34, "for Fat32, LBO
    // is >32 bits"). Was u32 — truncated every LBO above 4GB and mismatched
    // the (also-wrong) u64 extern decl in allocsup.rs.
    mut Lbo: i64,
    mut SectorCount: u32,
) -> i32 {
    // PAGED_CODE(); –  ignored

    if SectorCount != 0 {
        SectorCount -= 1;
        SectorCount >>= (*Vcb).AllocationSupport.LogOfBytesPerSector as u32;
        SectorCount += 1;
    }

    Vbo >>= (*Vcb).AllocationSupport.LogOfBytesPerSector as u32;
    Lbo >>= (*Vcb).AllocationSupport.LogOfBytesPerSector as i64;

    ntkrnl_sys::ASSERT!(SectorCount != 0);

    if Mcb != &raw mut (*Vcb).DirtyFatMcb {
        #[cfg(debug_assertions)]
        {
            let mut sparse_vbo: u32 = 0;
            let mut sparse_byte_count: i64 = 0;
            let non_sparse = FatNonSparseMcb(
                Vcb,
                Mcb,
                &mut sparse_vbo,
                &mut sparse_byte_count,
            );
            ntkrnl_sys::ASSERT!(non_sparse != 0
                    || (sparse_vbo == Vbo && sparse_byte_count == SectorCount as i64));
        }
    }

    let result = FsRtlAddLargeMcbEntry(
        Mcb,
        Vbo as i64,
        Lbo,
        SectorCount as i64,
    );

    if Mcb != &raw mut (*Vcb).DirtyFatMcb {
        #[cfg(debug_assertions)]
        {
            let mut sparse_vbo: u32 = 0;
            let mut sparse_byte_count: i64 = 0;
            let non_sparse = FatNonSparseMcb(
                Vcb,
                Mcb,
                &mut sparse_vbo,
                &mut sparse_byte_count,
            );
            ntkrnl_sys::ASSERT!(non_sparse != 0
                    || (sparse_vbo == Vbo && sparse_byte_count == SectorCount as i64));
        }
    }

    (result) as i32
}
#[no_mangle]
pub unsafe extern "system" fn FatLookupMcbEntry(
    Vcb: *mut VCB,
    Mcb: PLARGE_MCB,
    Vbo: u32,
    Lbo: *mut u64,
    ByteCount: *mut u32,
    Index: *mut u32,
) -> BOOLEAN {
    let mcb_scale_log2 = (*Vcb).AllocationSupport.LogOfBytesPerSector as u32;
    let mcb_scale_modulo = (1u32 << mcb_scale_log2) - 1;

    let mut LiLbo: i64 = 0;
    let mut LiSectorCount: i64 = 0;

    let remainder = Vbo & mcb_scale_modulo;

    let sector_count_ptr = if !ByteCount.is_null() {
        &mut LiSectorCount as *mut i64
    } else {
        core::ptr::null_mut()
    };

    let results = FsRtlLookupLargeMcbEntry(
        Mcb,
        (Vbo >> mcb_scale_log2) as i64,
        &mut LiLbo as *mut i64,
        sector_count_ptr,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Index as *mut u32,
    );

    if (LiLbo as u32) != 0xFFFF_FFFFu32 {
        *Lbo = (LiLbo as u64) << mcb_scale_log2;
        if results != 0 {
            *Lbo += remainder as u64;
        }
    } else {
        *Lbo = 0;
    }

    if !ByteCount.is_null() {
        *ByteCount = LiSectorCount as u32;
        if *ByteCount != 0 {
            *ByteCount <<= mcb_scale_log2;
            if *ByteCount == 0 {
                *ByteCount = 0xFFFF_FFFFu32;
            }
            if results != 0 {
                *ByteCount = (*ByteCount).wrapping_sub(remainder);
            }
        }
    }

    results
}
#[no_mangle]
pub unsafe extern "system" fn FatLookupLastMcbEntry(
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    Vbo: *mut VBO,
    Lbo: *mut LBO,
    Index: *mut ULONG,
) -> u8 {
    // PAGED_CODE();
    let mut LiVbo: LONGLONG = 0;
    let mut LiLbo: LONGLONG = 0;
    let mut LocalIndex: ULONG = 0;

    let Results = FsRtlLookupLastLargeMcbEntryAndIndex(
        Mcb,
        &mut LiVbo,
        &mut LiLbo,
        &mut LocalIndex,
    );

    let mcb_scale_log2 = (*Vcb).AllocationSupport.LogOfBytesPerSector as u32;
    let mcb_scale_modulo = (1u32 << mcb_scale_log2) - 1;

    *Vbo = (LiVbo as u32) << mcb_scale_log2;

    if (LiLbo as u32) != u32::MAX {
        *Lbo = ((LiLbo as u32) << mcb_scale_log2) as u64;
        *Lbo += (mcb_scale_modulo) as u64;
        *Vbo += mcb_scale_modulo;
    } else {
        *Lbo = 0;
    }

    if !Index.is_null() {
        *Index = LocalIndex;
    }

    Results
}
#[no_mangle]
//  `OUT PVBO Vbo` is a pointer to a VBO — ULONG32, FOUR bytes (fat.h). Storing a
//  u64 through it overruns the caller's local by 4 bytes; deviosup's NextVbo is
//  exactly such a 4-byte local. The C also truncates BEFORE the shift
//  (`*Vbo = ((VBO) LiVbo) << MCB_SCALE_LOG2`), so a wider store is not even the
//  same value. Take the width from the header, never from the callers.
pub unsafe extern "system" fn FatGetNextMcbEntry(
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    RunIndex: u32,
    Vbo: *mut VBO,
    Lbo: *mut u64,
    ByteCount: *mut u32,
) -> u8 {
    let mut Results: u8;
    let mut LiVbo: i64 = 0;
    let mut LiLbo: i64 = 0;
    let mut LiSectorCount: i64 = 0;

    Results = FsRtlGetNextLargeMcbEntry(Mcb, RunIndex, &mut LiVbo, &mut LiLbo, &mut LiSectorCount);

    if Results != 0 {
        let log: u32 = ((*Vcb).AllocationSupport.LogOfBytesPerSector) as u32;
        *Vbo = (LiVbo as VBO) << log;
        if (LiLbo as u32) != u32::MAX {
            *Lbo = (LiLbo as u64) << log;
        } else {
            *Lbo = 0;
        }
        *ByteCount = (LiSectorCount as u32) << log;
        if *ByteCount == 0 && LiSectorCount != 0 {
            NT_ASSERT((RunIndex == 0) as u32);
            *ByteCount = 0xFFFFFFFF_u32;
        }
    }

    Results
}
#[no_mangle]
pub unsafe extern "system" fn FatRemoveMcbEntry(
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    Vbo: u32,
    SectorCount: u32,
) {
    // PAGED_CODE();
    unsafe {
        let mcb_scale_log2 = (*Vcb).AllocationSupport.LogOfBytesPerSector as u32;
        let mut sector_count = SectorCount;
        let mut vbo = Vbo;

        if (sector_count != 0) && (sector_count != 0xFFFFFFFF_u32) {
            sector_count -= 1;
            sector_count >>= mcb_scale_log2;
            sector_count += 1;
        }

        vbo >>= mcb_scale_log2;

        // C: `#if DBG _SEH2_TRY { ... } _SEH2_EXCEPT(FatBugCheckExceptionFilter(...))`;
        // this crate builds the retail (!DBG) shape, same gate as allocsup.rs.
        #[cfg(DBG)]
        {
            let _ = crate::seh::try_except(&mut (), |_| {
                FsRtlRemoveLargeMcbEntry(Mcb, vbo as i64, sector_count as i64);
            }, |_, pointers| {
                crate::fatdata::FatBugCheckExceptionFilter(pointers as _) as i32
            });
        }
        #[cfg(not(DBG))]
        FsRtlRemoveLargeMcbEntry(Mcb, vbo as i64, sector_count as i64);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatVerifyVolume(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS
    let IrpSp: *mut IO_STACK_LOCATION;
    let mut RootDirectory: *mut DIRENT = core::ptr::null_mut();
    let mut BootSector: *mut PACKED_BOOT_SECTOR = core::ptr::null_mut();
    let mut Bpb: BIOS_PARAMETER_BLOCK = core::mem::zeroed();
    let VolDo: *mut VOLUME_DEVICE_OBJECT;
    let Vcb: *mut VCB;
    let Vpb: *mut VPB;
    let mut SectorSize: u32 = 0;
    let mut ClearVerify: bool = false;
    let mut ReleaseEntireVolume: bool = false;
    let mut VerifyAlreadyDone: bool = false;
    let mut DiskGeometry: DISK_GEOMETRY = core::mem::zeroed();
    let mut RootDirectoryLbo: LARGE_INTEGER = core::mem::zeroed();
    let mut RootDirectorySize: u32 = 0;
    let mut LabelFound: u8 = 0;
    let mut ChangeCount: u32 = 0;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, 0x2000 /* Dbg */, b"FatVerifyVolume\n\0".as_ptr() as *const i8);
    DebugTrace(0, 0x2000, b"DeviceObject = %p\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.VerifyVolume.DeviceObject);
    DebugTrace(0, 0x2000, b"Vpb          = %p\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.VerifyVolume.Vpb);

    VolDo = (*IrpSp).Parameters.VerifyVolume.DeviceObject as *mut VOLUME_DEVICE_OBJECT;
    Vpb = (*IrpSp).Parameters.VerifyVolume.Vpb;
    Vcb = &raw mut (*VolDo).Vcb as *mut VCB;
    (*IrpContext).Vcb = Vcb;

    if FlagOn((*IrpContext).Flags, 0x2 /* IRP_CONTEXT_FLAG_WAIT */) == 0 {
        DebugTrace(0, 0x2000, b"Cannot wait for verify.\n\0".as_ptr() as *const i8);
        Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace(-1, 0x2000, b"FatVerifyVolume -> %08lx\n\0".as_ptr() as *const i8, Status);
        return Status;
    }

    FatAcquireExclusiveGlobal(IrpContext);
    FatAcquireExclusiveVcb(IrpContext, Vcb);

    let Status_ptr: *mut NTSTATUS = &raw mut Status;
    let BootSector_ptr: *mut *mut PACKED_BOOT_SECTOR = &raw mut BootSector;
    let RootDirectory_ptr: *mut *mut DIRENT = &raw mut RootDirectory;
    let ReleaseEntireVolume_ptr: *mut bool = &raw mut ReleaseEntireVolume;
    let VerifyAlreadyDone_ptr: *mut bool = &raw mut VerifyAlreadyDone;
    let ClearVerify_ptr: *mut bool = &raw mut ClearVerify;
    let LabelFound_ptr: *mut u8 = &raw mut LabelFound;


    // _SEH2_TRY / _SEH2_FINALLY. The try_return()s leave the inner closure and
    // land on try_exit, which is still inside the try.
    crate::seh::try_finally(&mut (), |_| {
        let mut try_closure = || {
            // try block
            let AllowRawMount = BooleanFlagOn(((*IrpSp).Flags) as u32, 0x1 /* SL_ALLOW_RAW_MOUNT */);

            NT_ASSERT((*Vcb).VerifyThread.is_null() as u32);
            (*Vcb).VerifyThread = (KeGetCurrentThread()).cast();

            if FlagOn((*(*Vpb).RealDevice).Flags, 0x2 /* DO_VERIFY_VOLUME */) == 0 {
                DebugTrace(0, 0x2000, b"RealDevice has already been verified\n\0".as_ptr() as *const i8);
                *VerifyAlreadyDone_ptr = true;
                *Status_ptr = 0; // STATUS_SUCCESS
                return;
            }

            {
                let mut PartitionInformation: PARTITION_INFORMATION_EX = core::mem::zeroed();
                FatPerformDevIoCtrl(
                    IrpContext,
                    0x00070048 /* IOCTL_DISK_GET_PARTITION_INFO_EX */,
                    (*Vcb).TargetDeviceObject,
                    core::ptr::null_mut(),
                    0,
                    &mut PartitionInformation as *mut _ as *mut c_void,
                    core::mem::size_of::<PARTITION_INFORMATION_EX>() as u32,
                    false,
                    true,
                    &mut Iosb,
                );
            }

            if FlagOn((*(*Vcb).TargetDeviceObject).Characteristics, 0x1 /* FILE_REMOVABLE_MEDIA */) != 0 {
                *Status_ptr = FatPerformDevIoCtrl(
                    IrpContext,
                    0x00074800 /* IOCTL_DISK_CHECK_VERIFY */,
                    (*Vcb).TargetDeviceObject,
                    core::ptr::null_mut(),
                    0,
                    &mut ChangeCount as *mut _ as *mut c_void,
                    core::mem::size_of::<u32>() as u32,
                    false,
                    true,
                    &mut Iosb,
                );
                if *Status_ptr < 0 {
                    if AllowRawMount {
                        *Status_ptr = STATUS_WRONG_VOLUME;
                        return;
                    }
                    FatNormalizeAndRaiseStatus(IrpContext, *Status_ptr);
                }
            }

            if Iosb.Information != (core::mem::size_of::<u32>() as _) {
                ChangeCount = 0;
            }

            (*Vcb).ChangeCount = ChangeCount;

            if (*(*Vcb).TargetDeviceObject).DeviceType == 0x00000002 /* FILE_DEVICE_CD_ROM */
                && !FatScanForDataTrack(IrpContext, (*Vcb).TargetDeviceObject)
            {
                *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                return;
            }

            *Status_ptr = FatPerformDevIoCtrl(
                IrpContext,
                0x00070000 /* IOCTL_DISK_GET_DRIVE_GEOMETRY */,
                (*Vcb).TargetDeviceObject,
                core::ptr::null_mut(),
                0,
                &mut DiskGeometry as *mut _ as *mut c_void,
                core::mem::size_of::<DISK_GEOMETRY>() as u32,
                false,
                true,
                core::ptr::null_mut(),
            );
            if *Status_ptr < 0 {
                if AllowRawMount {
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
                FatNormalizeAndRaiseStatus(IrpContext, *Status_ptr);
            }

            SectorSize = (*Vcb).Bpb.BytesPerSector as u32;
            if SectorSize != DiskGeometry.BytesPerSector {
                *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                return;
            }

            *BootSector_ptr = FsRtlAllocatePoolWithTag(
                0x204 /* NonPagedPoolNxCacheAligned */,
                ROUND_TO_PAGES(SectorSize as usize) as u32,
                0x46495346 /* TAG_VERIFY_BOOTSECTOR */,
            ) as *mut PACKED_BOOT_SECTOR;

            if FatPerformVerifyDiskRead(
                IrpContext,
                Vcb,
                *BootSector_ptr as *mut c_void,
                0u64,
                SectorSize,
                AllowRawMount as i32,
            ) == 0 {
                *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                return;
            }

            if FatIsBootSectorFat(*BootSector_ptr) == 0 {
                DebugTrace(0, 0x2000, b"Not a Fat Volume\n\0".as_ptr() as *const i8);
                *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                return;
            }

            {
                let mut SerialNumber: u32 = 0;
                if IsBpbFat32(&raw const (*BootSector).PackedBpb) {
                    CopyUchar4(&mut SerialNumber, &raw const (*(BootSector as *const PACKED_BOOT_SECTOR_EX)).Id as *const u8);
                } else {
                    CopyUchar4(&mut SerialNumber, &raw const (*BootSector).Id as *const u8);
                }
                if SerialNumber != (*Vpb).SerialNumber {
                    DebugTrace(0, 0x2000, b"Not our serial number\n\0".as_ptr() as *const i8);
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
            }

            RtlZeroMemory(&mut Bpb as *mut _ as *mut c_void, core::mem::size_of::<BIOS_PARAMETER_BLOCK>());
            FatUnpackBios(&mut Bpb, (&raw const (*BootSector).PackedBpb).cast());
            if Bpb.Sectors != 0 { Bpb.LargeSectors = 0; }

            {
                // C macro IsBpbFat32 works on the UNPACKED Bpb here — check the field
                // (SectorsPerFat == 0, fat.h:101) directly instead of casting unpacked→packed.
                let size = if Bpb.SectorsPerFat == 0 {
                    core::mem::size_of::<BIOS_PARAMETER_BLOCK>()
                } else {
                    core::mem::offset_of!(BIOS_PARAMETER_BLOCK, LargeSectorsPerFat)
                };
                if RtlEqualMemory(
                    &Bpb as *const _ as *const c_void,
                    &raw const (*Vcb).Bpb as *const _ as *const c_void,
                    (size) as u32,
                ) == 0 {
                    DebugTrace(0, 0x2000, b"Bpb is different\n\0".as_ptr() as *const i8);
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
            }

            let root_dir_size = if FatRootDirectorySize(&Bpb) > 0 {
                FatRootDirectorySize(&Bpb)
            } else {
                FatBytesPerCluster(&Bpb)
            };
            RootDirectorySize = root_dir_size;

            *RootDirectory_ptr = FsRtlAllocatePoolWithTag(
                0x200,
                (((ROUND_TO_PAGES((((RootDirectorySize as usize) as u32) as u32) as usize) as u32) as usize)) as u32,
                0x46495352 /* TAG_VERIFY_ROOTDIR */,
            ) as *mut DIRENT;

            if !IsBpbFat32(&raw const (*BootSector).PackedBpb) {
                RootDirectoryLbo = FatRootDirectoryLbo(&Bpb);
                if FatPerformVerifyDiskRead(
                    IrpContext,
                    Vcb,
                    *RootDirectory_ptr as *mut c_void,
                    RootDirectoryLbo.QuadPart as u64,
                    RootDirectorySize,
                    AllowRawMount as i32,
                ) == 0 {
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
                *Status_ptr = FatSearchBufferForLabel(IrpContext, Vpb, *RootDirectory_ptr as *mut c_void, RootDirectorySize, LabelFound_ptr);
                if *Status_ptr < 0 {
                    return;
                }
                if *LabelFound_ptr == 0 && (*Vpb).VolumeLabelLength > 0 {
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
            } else {
                let mut RootDirectoryCluster = Bpb.RootDirFirstCluster;
                while RootDirectoryCluster != FAT_CLUSTER_LAST {
                    RootDirectoryLbo.QuadPart = FatGetLboFromIndex(Vcb, RootDirectoryCluster) as i64;
                    if FatPerformVerifyDiskRead(
                        IrpContext,
                        Vcb,
                        *RootDirectory_ptr as *mut c_void,
                        RootDirectoryLbo.QuadPart as u64,
                        RootDirectorySize,
                        AllowRawMount as i32,
                    ) == 0 {
                        *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                        return;
                    }
                    *Status_ptr = FatSearchBufferForLabel(IrpContext, Vpb, *RootDirectory_ptr as *mut c_void, RootDirectorySize, LabelFound_ptr);
                    if *Status_ptr < 0 {
                        return;
                    }
                    if *LabelFound_ptr != 0 {
                        break;
                    }
                    FatVerifyLookupFatEntry(IrpContext, Vcb, RootDirectoryCluster, &mut RootDirectoryCluster);
                    match FatInterpretClusterType(Vcb, RootDirectoryCluster) {
                        0 /* FatClusterAvailable */ | 1 /* FatClusterReserved */ | 2 /* FatClusterBad */ => {
                            FatRaiseStatus(IrpContext, 0xC0000102_u32 as i32 /* STATUS_FILE_CORRUPT_ERROR */);
                        }
                        _ => {}
                    }
                }
                if RootDirectoryCluster == FAT_CLUSTER_LAST && (*Vpb).VolumeLabelLength > 0 {
                    *Status_ptr = STATUS_WRONG_VOLUME; // was 0xC0000061 (STATUS_PRIVILEGE_NOT_HELD)
                    return;
                }
            }
        };

        try_closure();

        // try_exit: NOTHING;

        //  Put the Ea file back in a initial state.
        FatCloseEaFile(IrpContext, Vcb, (*Status_ptr == STATUS_SUCCESS) as u8);

        //  Mark all Fcbs as needing verification, but only if we really have
        //  to do it.
        if !*VerifyAlreadyDone_ptr {
            FatAcquireExclusiveVolume(IrpContext, Vcb);
            *ReleaseEntireVolume_ptr = true;

            FatMarkFcbCondition(IrpContext, (*Vcb).RootDcb, crate::types::_FCB_CONDITION_FcbNeedsToBeVerified as u32, 1);
        }

        //  If the verify didn't succeed, get the volume ready for a
        //  remount or eventual deletion.
        if (*Vcb).VcbCondition == VcbNotMounted {
            //  If the volume was already in an unmounted state, just bail
            //  and make sure we return STATUS_WRONG_VOLUME.
            *Status_ptr = STATUS_WRONG_VOLUME;
        } else if *Status_ptr == STATUS_WRONG_VOLUME {
            //  Grab everything so we can safely transition the volume state.
            if !*ReleaseEntireVolume_ptr {
                FatAcquireExclusiveVolume(IrpContext, Vcb);
                *ReleaseEntireVolume_ptr = true;
            }

            //  Get rid of any cached data, without flushing
            FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, 0 /* NoFlush */);

            //  Uninitialize the volume file cache map.
            CcUninitializeCacheMap(
                (*Vcb).VirtualVolumeFile,
                core::ptr::addr_of!(crate::fatdata::FatLargeZero),
                core::ptr::null_mut(),
            );

            FatTearDownAllocationSupport(IrpContext, Vcb);

            FatSetVcbCondition(Vcb, VcbNotMounted);

            *ClearVerify_ptr = true;
        } else if !*VerifyAlreadyDone_ptr {
            //  Grab everything so we can safely transition the volume state.
            if !*ReleaseEntireVolume_ptr {
                FatAcquireExclusiveVolume(IrpContext, Vcb);
                *ReleaseEntireVolume_ptr = true;
            }

            //  Get rid of any cached data, flushing first.
            FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, 1 /* Flush */);

            //  Flush and Purge the volume file.
            let _ = FatFlushFat(IrpContext, Vcb);
            CcPurgeCacheSection(core::ptr::addr_of_mut!((*Vcb).SectionObjectPointers), core::ptr::null(), 0, 0);

            //  Redo the allocation support with newly paged stuff.
            FatTearDownAllocationSupport(IrpContext, Vcb);
            FatSetupAllocationSupport(IrpContext, Vcb);

            FatCheckDirtyBit(IrpContext, Vcb);

            //  Check for write protected media.
            if FatIsMediaWriteProtected(IrpContext, (*Vcb).TargetDeviceObject) != 0 {
                SetFlag(&mut (*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED);
            } else {
                ClearFlag(&mut (*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED);
            }

            *ClearVerify_ptr = true;
        }

        if *ClearVerify_ptr {
            //  Mark the device as no longer needing verification.
            ClearFlag(&mut (*(*Vpb).RealDevice).Flags, 0x2 /* DO_VERIFY_VOLUME */);
        }
    }, |_, abnormal| {
        DebugUnwind(b"FatVerifyVolume\0".as_ptr() as *const i8);

        //  Free any buffer we may have allocated
        if !(*BootSector_ptr).is_null() {
            ExFreePool(*BootSector_ptr as *mut c_void);
        }
        if !(*RootDirectory_ptr).is_null() {
            ExFreePool(*RootDirectory_ptr as *mut c_void);
        }

        //  Show that we are done with this volume.
        NT_ASSERT(((*Vcb).VerifyThread == (KeGetCurrentThread()).cast()) as u32);
        (*Vcb).VerifyThread = core::ptr::null_mut();

        if *ReleaseEntireVolume_ptr {
            FatReleaseVolume(IrpContext, Vcb);
        }

        FatReleaseVcb(IrpContext, Vcb);
        FatReleaseGlobal(IrpContext);

        //  If this was not an abnormal termination, complete the irp.
        if !abnormal {
            FatCompleteRequest(IrpContext, Irp, *Status_ptr);
        }

        DebugTrace(-1, 0x2000, b"FatVerifyVolume -> %08lx\n\0".as_ptr() as *const i8, *Status_ptr);
    });

    Status
}

// Forward declarations for unknown external functions/statics/constants
const DEBUG_TRACE_FSCTRL: u32 = 0x0400; // unknown value, adjust as needed
#[no_mangle]
pub unsafe extern "system" fn FatIsBootSectorFat(BootSector: *const PACKED_BOOT_SECTOR) -> i32 {
    let mut Result: i32 = 1; // TRUE
    let mut Bpb: BIOS_PARAMETER_BLOCK = core::mem::zeroed();

    DebugTrace(1, DEBUG_TRACE_FSCTRL, b"FatIsBootSectorFat, BootSector = %p\n\0".as_ptr() as *const i8, BootSector);

    FatUnpackBios(&raw mut Bpb, core::ptr::addr_of!((*BootSector).PackedBpb) as *const core::ffi::c_void);

    if Bpb.Sectors != 0 { Bpb.LargeSectors = 0; }

    if ((*BootSector).Jump[0] != 0xe9) &&
       ((*BootSector).Jump[0] != 0xeb) &&
       ((*BootSector).Jump[0] != 0x49) {
        Result = 0;
    } else if (Bpb.BytesPerSector != 128) &&
              (Bpb.BytesPerSector != 256) &&
              (Bpb.BytesPerSector != 512) &&
              (Bpb.BytesPerSector != 1024) &&
              (Bpb.BytesPerSector != 2048) &&
              (Bpb.BytesPerSector != 4096) {
        Result = 0;
    } else if (Bpb.SectorsPerCluster != 1) &&
              (Bpb.SectorsPerCluster != 2) &&
              (Bpb.SectorsPerCluster != 4) &&
              (Bpb.SectorsPerCluster != 8) &&
              (Bpb.SectorsPerCluster != 16) &&
              (Bpb.SectorsPerCluster != 32) &&
              (Bpb.SectorsPerCluster != 64) &&
              (Bpb.SectorsPerCluster != 128) {
        Result = 0;
    } else if Bpb.ReservedSectors == 0 {
        Result = 0;
    } else if Bpb.Fats == 0 {
        Result = 0;
    } else if (Bpb.Sectors == 0) && (Bpb.LargeSectors == 0) {
        Result = 0;
    } else if (Bpb.SectorsPerFat == 0) && (Bpb.LargeSectorsPerFat == 0 || Bpb.FsVersion != 0) {
        Result = 0;
    } else if (Bpb.Media != 0xf0) &&
              (Bpb.Media != 0xf8) &&
              (Bpb.Media != 0xf9) &&
              (Bpb.Media != 0xfb) &&
              (Bpb.Media != 0xfc) &&
              (Bpb.Media != 0xfd) &&
              (Bpb.Media != 0xfe) &&
              (Bpb.Media != 0xff) &&
              ((*core::ptr::addr_of!(FatData)).FujitsuFMR() == 0 ||
               ((Bpb.Media != 0x00) && (Bpb.Media != 0x01) && (Bpb.Media != 0xfa))) {
        Result = 0;
    } else if (Bpb.SectorsPerFat != 0) && (Bpb.RootEntries == 0) {
        Result = 0;
    } else if (Bpb.SectorsPerFat == 0) && (Bpb.__bindgen_anon_1.__bindgen_anon_1.as_ref().MirrorDisabled() != 0) {
        Result = 0;
    }

    DebugTrace(-1, DEBUG_TRACE_FSCTRL, b"FatIsBootSectorFat -> %08lx\n\0".as_ptr() as *const i8, Result);

    Result
}

#[no_mangle]
pub unsafe extern "system" fn FatIsMediaWriteProtected(
    IrpContext: *mut IRP_CONTEXT,
    TargetDeviceObject: *mut DEVICE_OBJECT,
) -> u8 {
    let irp: *mut IRP;
    let mut event: KEVENT = core::mem::MaybeUninit::zeroed().assume_init();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::MaybeUninit::zeroed().assume_init();
    let mut status: i32;

    PAGED_CODE();
    // UNREFERENCED_PARAMETER( IrpContext );
    let _unused = IrpContext;

    KeInitializeEvent(&mut event, _EVENT_TYPE_NotificationEvent, 0);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_IS_WRITABLE,
        TargetDeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0,
        &mut event,
        &mut iosb,
    );

    if irp.is_null() {
        return 0;
    }

    (*(IoGetNextIrpStackLocation(irp))).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;

    status = IoCallDriver(TargetDeviceObject, irp);

    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut event as *mut KEVENT as *mut c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0,
            core::ptr::null_mut(),
        );
        status = iosb.__bindgen_anon_1.Status;
    }

    (status == STATUS_MEDIA_WRITE_PROTECTED) as u8
}
#[no_mangle]
pub unsafe extern "system" fn FatUserFsCtrl(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    let Status: NTSTATUS;
    let FsControlCode: u32;
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    PAGED_CODE();

    FsControlCode = (*IrpSp).Parameters.FileSystemControl.FsControlCode;

    DebugTrace(1, Dbg, b"FatUserFsCtrl...\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"FsControlCode = %08lx\n\0".as_ptr() as *const i8, FsControlCode);

    if (*Irp).RequestorMode != (0i32) as i8 && (FsControlCode & 3) == METHOD_NEITHER {
        SetFlag(
            core::ptr::addr_of_mut!((*IrpContext).Flags),
            IRP_CONTEXT_FLAG_WAIT,
        );
    }

    Status = match FsControlCode {
        FSCTL_REQUEST_OPLOCK_LEVEL_1
        | FSCTL_REQUEST_OPLOCK_LEVEL_2
        | FSCTL_REQUEST_BATCH_OPLOCK
        | FSCTL_OPLOCK_BREAK_ACKNOWLEDGE
        | FSCTL_OPBATCH_ACK_CLOSE_PENDING
        | FSCTL_OPLOCK_BREAK_NOTIFY
        | FSCTL_OPLOCK_BREAK_ACK_NO_2
        | FSCTL_REQUEST_FILTER_OPLOCK
        | FSCTL_REQUEST_OPLOCK => FatOplockRequest(IrpContext, Irp),

        FSCTL_LOCK_VOLUME => FatLockVolume(IrpContext, Irp),
        FSCTL_UNLOCK_VOLUME => FatUnlockVolume(IrpContext, Irp),
        FSCTL_DISMOUNT_VOLUME => FatDismountVolume(IrpContext, Irp),
        FSCTL_MARK_VOLUME_DIRTY => FatDirtyVolume(IrpContext, Irp),
        FSCTL_IS_VOLUME_DIRTY => FatIsVolumeDirty(IrpContext, Irp),
        FSCTL_IS_VOLUME_MOUNTED => FatIsVolumeMounted(IrpContext, Irp),
        FSCTL_IS_PATHNAME_VALID => FatIsPathnameValid(IrpContext, Irp),
        FSCTL_QUERY_RETRIEVAL_POINTERS => FatQueryRetrievalPointers(IrpContext, Irp),
        FSCTL_QUERY_FAT_BPB => FatQueryBpb(IrpContext, Irp),
        FSCTL_FILESYSTEM_GET_STATISTICS => FatGetStatistics(IrpContext, Irp),
        FSCTL_GET_RETRIEVAL_POINTER_BASE => (FatGetRetrievalPointerBase)(IrpContext, Irp),
        FSCTL_GET_BOOT_AREA_INFO => FatGetBootAreaInfo(IrpContext, Irp),
        FSCTL_GET_VOLUME_BITMAP => FatGetVolumeBitmap(IrpContext, Irp),
        FSCTL_GET_RETRIEVAL_POINTERS => FatGetRetrievalPointers(IrpContext, Irp),
        FSCTL_MOVE_FILE => (FatMoveFile)(IrpContext, Irp),
        FSCTL_ALLOW_EXTENDED_DASD_IO => FatAllowExtendedDasdIo(IrpContext, Irp),
        FSCTL_MARK_HANDLE => FatMarkHandle(IrpContext, Irp),
        FSCTL_SET_PURGE_FAILURE_MODE => FatSetPurgeFailureMode(IrpContext, Irp),
        FSCTL_SET_ZERO_ON_DEALLOCATION => FatSetZeroOnDeallocate(IrpContext, Irp),
        _ => {
            DebugTrace(0, Dbg, b"Invalid control code -> %08lx\n\0".as_ptr() as *const i8, FsControlCode);
            (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
            STATUS_INVALID_DEVICE_REQUEST
        }
    };

    DebugTrace(-1, Dbg, b"FatUserFsCtrl -> %08lx\n\0".as_ptr() as *const i8, Status);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn FatOplockRequest(
    IrpContext: *mut IRP_CONTEXT,
    mut Irp: *mut IRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut FsControlCode: ULONG;
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let mut OplockCount: ULONG = 0;
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut AcquiredVcb: BOOLEAN = 0;
    let mut AcquiredFcb: BOOLEAN = 0;
    let mut InputBuffer: *mut REQUEST_OPLOCK_INPUT_BUFFER = core::ptr::null_mut();
    let mut InputBufferLength: ULONG;
    let mut OutputBufferLength: ULONG;
    let TypeOfOpen: TYPE_OF_OPEN;

    PAGED_CODE();

    FsControlCode = (*IrpSp).Parameters.FileSystemControl.FsControlCode;

    TypeOfOpen = (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        &mut Vcb as *mut *mut VCB,
        &mut Fcb as *mut *mut FCB,
        &mut Ccb as *mut *mut CCB,
    );

    DebugTrace(1, Dbg, b"FatOplockRequest...\n\0".as_ptr() as *const i8, 0);
    DebugTrace(
        0,
        Dbg,
        b"FsControlCode = %08lx\n\0".as_ptr() as *const i8,
        FsControlCode,
    );

    if (TypeOfOpen != UserFileOpen) && (TypeOfOpen != UserDirectoryOpen) {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(
            -1,
            Dbg,
            b"FatOplockRequest -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8,
            0,
        );
        return STATUS_INVALID_PARAMETER;
    }

    if FsControlCode == FSCTL_REQUEST_OPLOCK {
        InputBufferLength = (*IrpSp).Parameters.FileSystemControl.InputBufferLength;
        InputBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut REQUEST_OPLOCK_INPUT_BUFFER;
        OutputBufferLength = (*IrpSp).Parameters.FileSystemControl.OutputBufferLength;

        if InputBufferLength < core::mem::size_of::<REQUEST_OPLOCK_INPUT_BUFFER>() as u32
            || OutputBufferLength < core::mem::size_of::<REQUEST_OPLOCK_OUTPUT_BUFFER>() as u32
        {
            (FatCompleteRequest)(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
            DebugTrace(
                -1,
                Dbg,
                b"FatOplockRequest -> STATUS_BUFFER_TOO_SMALL\n\0".as_ptr() as *const i8,
                0,
            );
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    if (TypeOfOpen == UserDirectoryOpen)
        && (FsControlCode != FSCTL_REQUEST_OPLOCK || FsRtlOplockIsSharedRequest(Irp) == 0)
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(
            -1,
            Dbg,
            b"FatOplockRequest -> STATUS_INVALID_PARAMETER\n\0".as_ptr() as *const i8,
            0,
        );
        return STATUS_INVALID_PARAMETER;
    }

    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    let p_status = core::ptr::addr_of_mut!(Status);
    let p_acquired_vcb = core::ptr::addr_of_mut!(AcquiredVcb);
    let p_acquired_fcb = core::ptr::addr_of_mut!(AcquiredFcb);
    let p_irp = core::ptr::addr_of_mut!(Irp);

    //  Use a try finally to free the Fcb/Vcb
    crate::seh::try_finally(&mut (), |_| {
        if (FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_1)
            || (FsControlCode == FSCTL_REQUEST_BATCH_OPLOCK)
            || (FsControlCode == FSCTL_REQUEST_FILTER_OPLOCK)
            || (FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_2)
            || ((FsControlCode == FSCTL_REQUEST_OPLOCK)
                && FlagOn((*InputBuffer).Flags, REQUEST_OPLOCK_INPUT_FLAG_REQUEST) != 0)
        {
            FatAcquireSharedVcb(IrpContext, (*Fcb).Vcb);
            *p_acquired_vcb = 1;
            (FatAcquireExclusiveFcb)(IrpContext, Fcb);
            *p_acquired_fcb = 1;

            if FsRtlOplockIsSharedRequest(*p_irp) != 0 {
                if TypeOfOpen == UserFileOpen {
                    // (ULONG) !FsRtlCheckLockForOplockRequest(...): nonzero when FsRtl
                    // DENIES the oplock. The translation had the negation inverted.
                    OplockCount = (FsRtlCheckLockForOplockRequest(
                        &mut (*Fcb).Specific.Fcb.as_mut().FileLock as *mut FILE_LOCK,
                        &(*Fcb).Header.AllocationSize as *const LARGE_INTEGER,
                    ) == 0) as ULONG;
                }
            } else {
                OplockCount = (*Fcb).UncleanCount;
            }
        } else if (FsControlCode == FSCTL_OPLOCK_BREAK_ACKNOWLEDGE)
            || (FsControlCode == FSCTL_OPBATCH_ACK_CLOSE_PENDING)
            || (FsControlCode == FSCTL_OPLOCK_BREAK_NOTIFY)
            || (FsControlCode == FSCTL_OPLOCK_BREAK_ACK_NO_2)
            || ((FsControlCode == FSCTL_REQUEST_OPLOCK)
                && FlagOn((*InputBuffer).Flags, REQUEST_OPLOCK_INPUT_FLAG_ACK) != 0)
        {
            FatAcquireSharedFcb(IrpContext, Fcb);
            *p_acquired_fcb = 1;
        } else if FsControlCode == FSCTL_REQUEST_OPLOCK {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_leave
        } else {
            FatBugCheck(FsControlCode, 0, 0);
        }

        // Check DELETE_ON_CLOSE
        if (FsControlCode == FSCTL_REQUEST_FILTER_OPLOCK)
            || (FsControlCode == FSCTL_REQUEST_BATCH_OPLOCK)
            || ((FsControlCode == FSCTL_REQUEST_OPLOCK)
                && FlagOn((*InputBuffer).RequestedOplockLevel, OPLOCK_LEVEL_CACHE_HANDLE) != 0)
        {
            if FlagOn((*Fcb).FcbState, FCB_STATE_DELETE_ON_CLOSE) != 0 {
                *p_status = STATUS_DELETE_PENDING;
                return; // try_leave
            }
        }

        *p_status = FsRtlOplockFsctrl(FatGetFcbOplock(Fcb), *p_irp, OplockCount);

        *p_irp = core::ptr::null_mut();

        (*Fcb).Header.IsFastIoPossible = (FatIsFastIoPossible(Fcb)) as u8;
    }, |_, _| {
        DebugUnwind(b"FatOplockRequest\0".as_ptr() as *const i8);

        //  Release all of our resources
        if *p_acquired_vcb != 0 {
            FatReleaseVcb(IrpContext, (*Fcb).Vcb);
        }
        if *p_acquired_fcb != 0 {
            (FatReleaseFcb)(IrpContext, Fcb);
        }

        DebugTrace(
            -1,
            Dbg,
            b"FatOplockRequest -> %08lx\n\0".as_ptr() as *const i8,
            *p_status,
        );
    });

    (FatCompleteRequest)(IrpContext, Irp, Status);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatMoveFileNeedsWriteThrough(
    IrpContext: *mut IRP_CONTEXT,
    FcbOrDcb: *mut FCB,
    OldWriteThroughFlags: u32,
) {
    const FAT_NTC_FCB: u32 = 0x0502; // TODO: correct value needed
    const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04; // TODO: correct value needed
    const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: u32 = 0x08; // TODO: correct value needed

    if NodeType(FcbOrDcb as *mut core::ffi::c_void) == FAT_NTC_FCB {
        if (*FcbOrDcb).Header.ValidDataLength.QuadPart == 0 {
            ClearFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_WRITE_THROUGH);
            SetFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
        } else {
            (*IrpContext).Flags &= !(IRP_CONTEXT_FLAG_WRITE_THROUGH | IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
            (*IrpContext).Flags |= OldWriteThroughFlags;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatMoveFile(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    // Forward declarations for unknown symbols (types assumed from ntkrnl_sys)

    // Constants used in this function
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const IRP_CONTEXT_FLAG_PARENT_BY_CHILD: u32 = 0x80000000_u32;
    const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04;
    const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: u32 = 0x08;
    const CCB_FLAG_MANAGE_VOLUME_ACCESS: u32 = 0x00020000;
    const FCB_STATE_DENY_DEFRAG: u32 = 0x4000;
    const CCB_FLAG_DENY_DEFRAG: u32 = 0x00080000;
    const FAT_NTC_FCB: u32 = 0x0502;
    const FAT_NTC_ROOT_DCB: u32 = 0x0504;
    const FAT_DEFAULT_DEFRAG_CHUNK_IN_BYTES: u32 = 0x00010000; // 8KB
    const TAG_DEFRAG_BUFFER: u32 = 0x47746146; // arbitrary
    const SL_WRITE_THROUGH: u32 = 0x04;
    const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1i64;
    const FAT_ENTRY: u32 = 0x0FFFFFFF; // placeholder for FAT entry type
    const FAT_DIRENT_ATTR_VOLUME_ID: u8 = 0x08;
    const FAT_DIRENT_ATTR_ARCHIVE: u8 = 0x20;
    const FAT_DIRENT_NEVER_USED: u8 = 0x00;

    // Local variable declarations (keeping C names)
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IrpSp: PIO_STACK_LOCATION;
    let mut FileObject: PFILE_OBJECT = null_mut();
    let mut TypeOfOpen: TYPE_OF_OPEN = 0;
    let mut Vcb: *mut VCB = null_mut();
    let mut FcbOrDcb: *mut FCB = null_mut();
    let mut Ccb: *mut CCB = null_mut();
    let mut InputBufferLength: u32;
    let mut InputBuffer: *const MOVE_FILE_DATA;
    let mut ClusterShift: u32;
    let mut MaxClusters: u32;
    let mut FileOffset: u32 = 0;
    let mut TargetLbo: i64 = 0;
    let mut TargetCluster: u32 = 0;
    let mut LargeSourceLbo: LARGE_INTEGER = core::mem::zeroed();
    let mut LargeTargetLbo: LARGE_INTEGER = core::mem::zeroed();
    let mut ByteCount: u32 = 0;
    let mut BytesToWrite: u32 = 0;
    let mut BytesToReallocate: u32 = 0;
    let mut FirstSpliceSourceCluster: u32 = 0;
    let mut FirstSpliceTargetCluster: u32 = 0;
    let mut SecondSpliceSourceCluster: u32 = 0;
    let mut SecondSpliceTargetCluster: u32 = 0;
    let mut SourceMcb: LARGE_MCB = core::mem::zeroed();
    let mut TargetMcb: LARGE_MCB = core::mem::zeroed();
    let mut StackEvent: KEVENT = core::mem::zeroed();
    let mut Buffer: *mut core::ffi::c_void = null_mut();
    let mut BufferSize: u32 = 0;
    let mut SourceMcbInitialized: i32 = 0; /* BOOLEAN */
    let mut TargetMcbInitialized: i32 = 0;
    let mut FcbAcquired: i32 = 0;
    let mut EventArmed: i32 = 0;
    let mut DiskSpaceAllocated: i32 = 0;
    let mut Dirent: *mut DIRENT = null_mut();
    let mut DirentBcb: PBCB = null_mut();
    let mut OldWriteThroughFlags: u32 = 0;
    let mut LocalAbnormalTermination: i32 = 0;
    // Thunking locals
    #[cfg(all(target_pointer_width = "64", feature = "wow64"))]
    let mut LocalMoveFileData: MOVE_FILE_DATA;
    #[cfg(all(target_pointer_width = "64", feature = "wow64"))]
    let mut MoveFileData32: *const MOVE_FILE_DATA32;
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatMoveFile, FsControlCode = %08lx\n\0".as_ptr() as *const i8,
               (*IrpSp).Parameters.FileSystemControl.FsControlCode);

    // The C snapshots these BEFORE forcing WAIT; FatMoveFileNeedsWriteThrough
    // restores them. The translation had left this 0.
    OldWriteThroughFlags = (*IrpContext).Flags & (IRP_CONTEXT_FLAG_WRITE_THROUGH | IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);

    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    //  Everything up to the _SEH2_TRY completes the request and RETURNS on
    //  failure. (The translation funnelled these into the common tail, which
    //  completed the request a second time.)
    TypeOfOpen = FatDecodeFileObject((*IrpSp).FileObject, &mut Vcb, &mut FcbOrDcb, &mut Ccb);

    if TypeOfOpen != UserVolumeOpen {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    InputBufferLength = (*IrpSp).Parameters.FileSystemControl.InputBufferLength;
    InputBuffer = (*Irp).AssociatedIrp.SystemBuffer as *const MOVE_FILE_DATA;

    // ---- WOW64 thunking ----
    #[cfg(all(target_pointer_width = "64", feature = "wow64"))]
    if IoIs32bitProcess(Irp) != 0 {
        if InputBuffer.is_null() || InputBufferLength < core::mem::size_of::<MOVE_FILE_DATA32>() as u32 {
            FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
            return STATUS_BUFFER_TOO_SMALL;
        }
        MoveFileData32 = InputBuffer as *const MOVE_FILE_DATA32;
        LocalMoveFileData = core::mem::zeroed();
        LocalMoveFileData.FileHandle = LongToHandle((*MoveFileData32).FileHandle) as HANDLE;
        LocalMoveFileData.StartingVcn = (*MoveFileData32).StartingVcn;
        LocalMoveFileData.StartingLcn = (*MoveFileData32).StartingLcn;
        LocalMoveFileData.ClusterCount = (*MoveFileData32).ClusterCount;
        InputBuffer = &LocalMoveFileData;
    } else {
        if InputBuffer.is_null() || InputBufferLength < core::mem::size_of::<MOVE_FILE_DATA>() as u32 {
            FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
            return STATUS_BUFFER_TOO_SMALL;
        }
    }
    #[cfg(not(all(target_pointer_width = "64", feature = "wow64")))]
    if InputBuffer.is_null() || InputBufferLength < core::mem::size_of::<MOVE_FILE_DATA>() as u32 {
        FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    MaxClusters = (*Vcb).AllocationSupport.NumberOfClusters;
    TargetCluster = (*InputBuffer).StartingLcn.__bindgen_anon_1.LowPart.wrapping_add(2);

    if (*InputBuffer).StartingVcn.__bindgen_anon_1.HighPart != 0
        || (*InputBuffer).StartingLcn.__bindgen_anon_1.HighPart != 0
        || TargetCluster < 2
        || (TargetCluster.wrapping_add((*InputBuffer).ClusterCount) < TargetCluster)
        || (TargetCluster.wrapping_add((*InputBuffer).ClusterCount) > MaxClusters + 2)
        || (*InputBuffer).StartingVcn.__bindgen_anon_1.LowPart >= MaxClusters
        || (*InputBuffer).ClusterCount == 0
    {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Reference file object from handle
    Status = ObReferenceObjectByHandle(
        (*InputBuffer).FileHandle,
        0,
        IoFileObjectType as *mut _,
        (*Irp).RequestorMode,
        &mut FileObject as *mut *mut FILE_OBJECT as *mut PVOID,
        null_mut(),
    );

    if !NT_SUCCESS(Status) {
        FatCompleteRequest(IrpContext, Irp, Status);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, Status);
        return Status;
    }

    if (*FileObject).Vpb != (*Vcb).Vpb {
        ObDereferenceObject(FileObject as *mut _);
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Decode again with new FileObject
    TypeOfOpen = FatDecodeFileObject(FileObject, &mut Vcb, &mut FcbOrDcb, &mut Ccb);

    if (TypeOfOpen != UserFileOpen && TypeOfOpen != UserDirectoryOpen)
        || (TypeOfOpen == UserDirectoryOpen
            && ((NodeType(FcbOrDcb as _) == FAT_NTC_ROOT_DCB && FatIsFat32(Vcb) == 0)
                || (*InputBuffer).StartingVcn.QuadPart == 0))
    {
        ObDereferenceObject(FileObject as *mut _);
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    (FatMoveFileNeedsWriteThrough)(IrpContext, FcbOrDcb, OldWriteThroughFlags);

    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_PARENT_BY_CHILD);

    ClusterShift = (*Vcb).AllocationSupport.LogOfBytesPerCluster as u32;

    let p_status = core::ptr::addr_of_mut!(Status);
    let p_buffer = core::ptr::addr_of_mut!(Buffer);
    let p_disk_space_allocated = core::ptr::addr_of_mut!(DiskSpaceAllocated);
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_source_mcb_initialized = core::ptr::addr_of_mut!(SourceMcbInitialized);
    let p_target_mcb_initialized = core::ptr::addr_of_mut!(TargetMcbInitialized);
    let p_source_mcb = core::ptr::addr_of_mut!(SourceMcb);
    let p_target_mcb = core::ptr::addr_of_mut!(TargetMcb);
    let p_fcb_acquired = core::ptr::addr_of_mut!(FcbAcquired);
    let p_event_armed = core::ptr::addr_of_mut!(EventArmed);
    let p_stack_event = core::ptr::addr_of_mut!(StackEvent);
    let p_file_object = core::ptr::addr_of_mut!(FileObject);
    let p_local_abnormal_termination = core::ptr::addr_of_mut!(LocalAbnormalTermination);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // Initialize state variables
        FileOffset = (*InputBuffer).StartingVcn.__bindgen_anon_1.LowPart << ClusterShift;
        ByteCount = (*InputBuffer).ClusterCount << ClusterShift;
        TargetLbo = FatGetLboFromIndex(Vcb, TargetCluster);
        LargeTargetLbo.QuadPart = TargetLbo as i64;
        *p_buffer = null_mut();

        if FileOffset.wrapping_add(ByteCount) < FileOffset {
            *p_status = STATUS_INVALID_PARAMETER;
            return; // try_return
        }

        KeInitializeEvent(p_stack_event, _EVENT_TYPE_NotificationEvent, 0); // FALSE -> 0

        FsRtlInitializeLargeMcb(p_source_mcb, PagedPool);
        *p_source_mcb_initialized = 1;

        FsRtlInitializeLargeMcb(p_target_mcb, PagedPool);
        *p_target_mcb_initialized = 1;

        // Handle directory open - switch to stream fileobject
        if TypeOfOpen == UserDirectoryOpen {
            let DirStreamFileObject: PFILE_OBJECT;

            FatAcquireExclusiveFcb(IrpContext, FcbOrDcb);
            *p_fcb_acquired = 1;

            FatVerifyFcb(IrpContext, FcbOrDcb);

            FatOpenDirectoryFile(IrpContext, FcbOrDcb);
            DirStreamFileObject = (*FcbOrDcb).Specific.Dcb.as_ref().DirectoryFile;

            ObReferenceObject(DirStreamFileObject as *mut _);
            ObDereferenceObject((*p_file_object) as *mut _);
            *p_file_object = DirStreamFileObject;

            FatReleaseFcb(IrpContext, FcbOrDcb);
            *p_fcb_acquired = 0;
        }

        // Buffer size calculation
        BufferSize = FAT_DEFAULT_DEFRAG_CHUNK_IN_BYTES;
        if BufferSize < (1u32 << ClusterShift) {
            BufferSize = 1u32 << ClusterShift;
        }

        // Main processing loop
        while ByteCount != 0 {
            let mut TempVbo: VBO = 0;
            let mut TempLbo: LBO = 0;
            let mut TempByteCount: u32 = 0;

            CcCanIWrite(*p_file_object, BufferSize, 1, 0); // TRUE, FALSE

            if (*p_fcb_acquired) == 0 {
                FatAcquireExclusiveFcb(IrpContext, FcbOrDcb);
                *p_fcb_acquired = 1;
                FatVerifyFcb(IrpContext, FcbOrDcb);
            }

            // Check defrag deny flags
            if ((*FcbOrDcb).FcbState & FCB_STATE_DENY_DEFRAG) != 0
                && ((*Ccb).Flags & CCB_FLAG_DENY_DEFRAG) == 0
            {
                DebugTrace(-1, Dbg, b"FatMoveFile -> %08lx\n\0".as_ptr() as *const i8, STATUS_ACCESS_DENIED);
                *p_status = STATUS_ACCESS_DENIED;
                return; // try_return
            }

            if (*p_buffer).is_null() {
                *p_buffer = FsRtlAllocatePoolWithTag(NonPagedPoolNx, BufferSize, TAG_DEFRAG_BUFFER);
            }

            (FatComputeMoveFileParameter)(
                IrpContext,
                FcbOrDcb,
                BufferSize,
                FileOffset,
                &mut ByteCount,
                &mut BytesToReallocate,
                &mut BytesToWrite,
                &mut LargeSourceLbo,
            );

            if ByteCount == 0 {
                break; // exit while loop
            }

            (FatComputeMoveFileSplicePoints)(
                IrpContext,
                FcbOrDcb,
                FileOffset,
                TargetCluster,
                BytesToReallocate,
                &mut FirstSpliceSourceCluster,
                &mut FirstSpliceTargetCluster,
                &mut SecondSpliceSourceCluster,
                &mut SecondSpliceTargetCluster,
                p_source_mcb,
            );

            TempByteCount = BytesToReallocate;
            FatAllocateDiskSpace(IrpContext, Vcb, TargetCluster, &mut TempByteCount, 1, p_target_mcb);
            *p_disk_space_allocated = 1;

            if FsRtlNumberOfRunsInLargeMcb(p_target_mcb) != 1
                || FatGetNextMcbEntry(Vcb, p_target_mcb, 0, &mut TempVbo, &mut TempLbo, &mut TempByteCount) == 0
                || FatGetIndexFromLbo(Vcb, TempLbo) != TargetCluster
                || TempByteCount != BytesToReallocate
            {
                *p_status = STATUS_INVALID_PARAMETER;
                return; // try_return
            }

            // Debug print if enabled
            // (FatMoveFileDebug check omitted)

            if (*FcbOrDcb).Header.ValidDataLength.QuadPart != 0 || NodeType(FcbOrDcb as _) != FAT_NTC_FCB {
                FatFlushFatEntries(
                    IrpContext,
                    Vcb,
                    TargetCluster,
                    BytesToReallocate >> ClusterShift,
                );
            }

            // Acquire PagingIo exclusive and set event
            ExAcquireResourceExclusiveLite((*FcbOrDcb).Header.PagingIoResource, 1);
            (*FcbOrDcb).MoveFileEvent = p_stack_event;
            *p_event_armed = 1;
            ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);

            // Write data if needed
            if BytesToWrite != 0 {
                let mut IoIrp: PIRP = core::ptr::null_mut();
                let mut IoEvent: KEVENT = core::mem::zeroed();
                let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

                KeInitializeEvent(&mut IoEvent, _EVENT_TYPE_NotificationEvent, 0);

                NT_ASSERT((LargeTargetLbo.QuadPart >= (*Vcb).AllocationSupport.FileAreaLbo as i64) as u32);

                IoIrp = IoBuildSynchronousFsdRequest(
                    IRP_MJ_READ as u32,
                    (*Vcb).TargetDeviceObject,
                    *p_buffer,
                    BytesToWrite,
                    &mut LargeSourceLbo,
                    &mut IoEvent,
                    &mut Iosb,
                );

                if IoIrp.is_null() {
                    FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
                    /* FatRaiseStatus never returns */
                    *p_status = STATUS_INSUFFICIENT_RESOURCES;
                    return; // try_return
                }

                *p_status = IoCallDriver((*Vcb).TargetDeviceObject, IoIrp);

                if (*p_status) == STATUS_PENDING {
                    KeWaitForSingleObject(
                        &mut IoEvent as *mut KEVENT as *mut c_void,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0,
                        null_mut(),
                    );
                    *p_status = Iosb.__bindgen_anon_1.Status;
                }

                if !NT_SUCCESS(*p_status) {
                    FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                    return; // try_return
                }

                // Write
                KeClearEvent(&mut IoEvent as *mut KEVENT as *mut c_void);

                IoIrp = IoBuildSynchronousFsdRequest(
                    IRP_MJ_WRITE as u32,
                    (*Vcb).TargetDeviceObject,
                    *p_buffer,
                    BytesToWrite,
                    &mut LargeTargetLbo,
                    &mut IoEvent,
                    &mut Iosb,
                );

                if IoIrp.is_null() {
                    FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
                    *p_status = STATUS_INSUFFICIENT_RESOURCES;
                    return; // try_return
                }

                (*(IoGetNextIrpStackLocation(IoIrp))).Flags |= SL_WRITE_THROUGH as u8;

                *p_status = IoCallDriver((*Vcb).TargetDeviceObject, IoIrp);

                if (*p_status) == STATUS_PENDING {
                    KeWaitForSingleObject(
                        &mut IoEvent as *mut KEVENT as *mut c_void,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0,
                        null_mut(),
                    );
                    *p_status = Iosb.__bindgen_anon_1.Status;
                }

                if !NT_SUCCESS(*p_status) {
                    FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                    return; // try_return
                }
            }

            // Splice fixing
            FatSetFatEntry(
                IrpContext,
                Vcb,
                SecondSpliceSourceCluster,
                SecondSpliceTargetCluster,
            );

            if (*FcbOrDcb).Header.ValidDataLength.QuadPart != 0 || NodeType(FcbOrDcb as _) != FAT_NTC_FCB {
                FatFlushFatEntries(IrpContext, Vcb, SecondSpliceSourceCluster, 1);
            }

            if FirstSpliceSourceCluster == 0 {
                NT_ASSERT((NodeType(FcbOrDcb as _) == FAT_NTC_FCB) as u32);

                FatGetDirentFromFcbOrDcb(
                    IrpContext,
                    FcbOrDcb,
                    0, // FALSE
                    &mut Dirent,
                    p_dirent_bcb,
                );

                (*Dirent).FirstClusterOfFile = FirstSpliceTargetCluster as USHORT;
                if FatIsFat32(Vcb) != 0 {
                    (*Dirent).__bindgen_anon_1.bindgen_union_field = (FirstSpliceTargetCluster >> 16) as u16;
                }

                FatSetDirtyBcb(IrpContext, p_dirent_bcb, Vcb, 1);
                FatUnpinBcb(IrpContext, p_dirent_bcb);
                *p_dirent_bcb = null_mut();

                FatFlushDirentForFile(IrpContext, FcbOrDcb);
                (*FcbOrDcb).FirstClusterOfFile = FirstSpliceTargetCluster;
            } else {
                FatSetFatEntry(
                    IrpContext,
                    Vcb,
                    FirstSpliceSourceCluster,
                    FirstSpliceTargetCluster,
                );

                if (*FcbOrDcb).Header.ValidDataLength.QuadPart != 0 || NodeType(FcbOrDcb as _) != FAT_NTC_FCB {
                    FatFlushFatEntries(IrpContext, Vcb, FirstSpliceSourceCluster, 1);
                }
            }

            *p_disk_space_allocated = 0;

            (FatMoveFileNeedsWriteThrough)(IrpContext, FcbOrDcb, OldWriteThroughFlags);

            FatDeallocateDiskSpace(IrpContext, Vcb, p_source_mcb, 0);
            FatUnpinRepinnedBcbs(IrpContext);

            *p_status = (FatHijackIrpAndFlushDevice)(IrpContext, Irp, (*Vcb).TargetDeviceObject);
            if !NT_SUCCESS(*p_status) {
                FatNormalizeAndRaiseStatus(IrpContext, *p_status);
                return; // try_return
            }

            FatRemoveMcbEntry(Vcb, &mut (*FcbOrDcb).Mcb, FileOffset, BytesToReallocate);
            (FatAddMcbEntry)(
                Vcb,
                &mut (*FcbOrDcb).Mcb,
                FileOffset,
                TargetLbo,
                BytesToReallocate,
            );

            // Event release and reacquire
            KeSetEvent(p_stack_event as *mut KEVENT as *mut c_void, 0, 0);
            ExAcquireResourceExclusiveLite((*FcbOrDcb).Header.PagingIoResource, 1);
            (*FcbOrDcb).MoveFileEvent = null_mut();
            *p_event_armed = 0;
            ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);

            // Release Fcb
            FatReleaseFcb(IrpContext, FcbOrDcb);
            *p_fcb_acquired = 0;

            // Advance variables
            TargetCluster += BytesToReallocate >> ClusterShift;
            FileOffset += BytesToReallocate;
            TargetLbo += BytesToReallocate as i64;
            ByteCount -= BytesToReallocate;
            LargeTargetLbo.QuadPart += BytesToReallocate as i64;

            FatRemoveMcbEntry(Vcb, p_source_mcb, 0, 0xFFFFFFFF);
            FatRemoveMcbEntry(Vcb, p_target_mcb, 0, 0xFFFFFFFF);

            KeClearEvent(p_stack_event as *mut KEVENT as *mut c_void);
        } // end while ByteCount

        *p_status = STATUS_SUCCESS;
        // try_exit: NOTHING;
    }, |_, abnormal| {
        DebugUnwind(b"FatMoveFile\0".as_ptr() as *const i8);

        *p_local_abnormal_termination |= abnormal as i32;

        ClearFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_PARENT_BY_CHILD);

        //  If we allocated a substitute buffer, free it.
        if !(*p_buffer).is_null() {
            ExFreePool(*p_buffer);
        }

        // _SEH2_TRY / _SEH2_FINALLY (nested)
        crate::seh::try_finally(&mut (), |_| {
            //  If we still have some new allocation hanging around, remove it.
            if *p_disk_space_allocated != 0 {
                FatDeallocateDiskSpace(IrpContext, Vcb, p_target_mcb, 0);
                FatUnpinRepinnedBcbs(IrpContext);
            }
        }, |_, abnormal| {
            *p_local_abnormal_termination |= abnormal as i32;

            //  Check on the directory Bcb
            if !(*p_dirent_bcb).is_null() {
                FatUnpinBcb(IrpContext, p_dirent_bcb);
            }

            //  Uninitialize our MCBs
            if *p_source_mcb_initialized != 0 {
                FsRtlUninitializeLargeMcb(p_source_mcb);
            }
            if *p_target_mcb_initialized != 0 {
                FsRtlUninitializeLargeMcb(p_target_mcb);
            }

            //  If this is an abnormal termination then presumably something
            //  bad happened.  Set the Allocation size to unknown and clear
            //  the Mcb, but only if we still own the Fcb.
            if *p_local_abnormal_termination != 0 && *p_fcb_acquired != 0 {
                if (*FcbOrDcb).FirstClusterOfFile == 0 {
                    (*FcbOrDcb).Header.AllocationSize.QuadPart = 0;
                } else {
                    (*FcbOrDcb).Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
                }
                FatRemoveMcbEntry(Vcb, &mut (*FcbOrDcb).Mcb, 0, 0xFFFFFFFF);
            }

            //  If we broke out of the loop with the Event armed, defuse it
            //  in the same way we do it after a write.
            if *p_event_armed != 0 {
                KeSetEvent(p_stack_event as *mut KEVENT as *mut c_void, 0, 0);
                ExAcquireResourceExclusiveLite((*FcbOrDcb).Header.PagingIoResource, 1);
                (*FcbOrDcb).MoveFileEvent = null_mut();
                ExReleaseResourceLite((*FcbOrDcb).Header.PagingIoResource);
            }

            //  Finally release the main file resource.
            if *p_fcb_acquired != 0 {
                FatReleaseFcb(IrpContext, FcbOrDcb);
            }

            //  Now dereference the fileobject.  If the user was a wacko they
            //  could have deleted the handle at this point.
            ObDereferenceObject(*p_file_object as *mut _);
        });
    });

    FatCompleteRequest(IrpContext, Irp, Status);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatComputeMoveFileParameter(
    IrpContext: PIRP_CONTEXT,
    FcbOrDcb: PFCB,
    BufferSize: u32,
    FileOffset: u32,
    ByteCount: *mut u32,
    BytesToReallocate: *mut u32,
    BytesToWrite: *mut u32,
    SourceLbo: PLARGE_INTEGER,
) {
    unsafe {
        let mut ClusterSize: u32;
        let mut AllocationSize: u32;
        let mut ValidDataLength: u32;
        let mut ClusterAlignedVDL: u32;
        let mut RunLbo: i64 = 0;
        let mut RunByteCount: u32 = 0;
        let mut RunIndex: u32 = 0;
        let mut RunAllocated: u8 = 0;
        let mut RunEndOnMax: u8 = 0;

        // PAGED_CODE();

        if (*FcbOrDcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
            FatLookupFileAllocationSize(IrpContext, FcbOrDcb);

            if NodeType(FcbOrDcb as *mut c_void) == 0x503 /* FAT_NTC_DCB */ ||
                (NodeType(FcbOrDcb as *mut c_void) == 0x504 /* FAT_NTC_ROOT_DCB */ && FatIsFat32((*FcbOrDcb).Vcb) != 0) {
                (*FcbOrDcb).Header.FileSize.__bindgen_anon_1.LowPart = (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
            }
        }

        AllocationSize = (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
        ValidDataLength = (*FcbOrDcb).Header.ValidDataLength.__bindgen_anon_1.LowPart;

        if FileOffset + *ByteCount > AllocationSize {
            if FileOffset >= AllocationSize {
                *ByteCount = 0;
                *BytesToReallocate = 0;
                *BytesToWrite = 0;

                return;
            }

            *ByteCount = AllocationSize - FileOffset;
        }

        if (FileOffset & (BufferSize - 1)) + *ByteCount > BufferSize {
            *BytesToReallocate = BufferSize - (FileOffset & (BufferSize - 1));
        } else {
            *BytesToReallocate = *ByteCount;
        }

        FatLookupFileAllocation(
            IrpContext,
            FcbOrDcb,
            FileOffset,
            &mut RunLbo,
            &mut RunByteCount,
            &mut RunAllocated,
            &mut RunEndOnMax,
            &mut RunIndex,
        );

        NT_ASSERT(RunAllocated as u32);

        if RunByteCount < *BytesToReallocate {
            *BytesToReallocate = RunByteCount;
        }

        (*SourceLbo).QuadPart = RunLbo;

        ClusterSize = 1u32 << (*(*FcbOrDcb).Vcb).AllocationSupport.LogOfBytesPerCluster;

        NT_ASSERT((ClusterSize <= BufferSize) as u32);

        ClusterAlignedVDL = (ValidDataLength + (ClusterSize - 1)) & !(ClusterSize - 1);

        if (NodeType(FcbOrDcb as *mut c_void) == 0x502 /* FAT_NTC_FCB */) &&
            (FileOffset + *BytesToReallocate > ClusterAlignedVDL) {
            if FileOffset > ClusterAlignedVDL {
                *BytesToWrite = 0;
            } else {
                *BytesToWrite = ClusterAlignedVDL - FileOffset;
            }
        } else {
            *BytesToWrite = *BytesToReallocate;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatComputeMoveFileSplicePoints(
    IrpContext: *mut IRP_CONTEXT,
    FcbOrDcb: *mut FCB,
    FileOffset: ULONG,
    TargetCluster: ULONG,
    BytesToReallocate: ULONG,
    FirstSpliceSourceCluster: *mut ULONG,
    FirstSpliceTargetCluster: *mut ULONG,
    SecondSpliceSourceCluster: *mut ULONG,
    SecondSpliceTargetCluster: *mut ULONG,
    SourceMcb: *mut LARGE_MCB,
) {

    const FAT_CLUSTER_LAST: ULONG = 0x0FFFFFFF;
    const STATUS_FILE_CORRUPT_ERROR: NTSTATUS = 0xC0000102_u32 as i32;

    let mut SourceVbo: VBO = 0;
    let mut SourceLbo: LBO = 0;
    let mut SourceIndex: ULONG = 0;
    let mut SourceBytesInRun: ULONG = 0;
    let mut SourceBytesRemaining: ULONG = 0;
    let mut SourceMcbVbo: ULONG = 0;
    let mut SourceMcbBytesInRun: ULONG = 0;
    let Vcb: *mut VCB;
    let mut Result: BOOLEAN;

    PAGED_CODE();

    Vcb = (*FcbOrDcb).Vcb;

    if FileOffset == 0 {
        SourceIndex = 0;
        *FirstSpliceSourceCluster = 0;
        Result = FatGetNextMcbEntry(
            Vcb,
            &mut (*FcbOrDcb).Mcb,
            0,
            &mut SourceVbo,
            &mut SourceLbo,
            &mut SourceBytesInRun,
        );
    } else {
        Result = FatLookupMcbEntry(
            Vcb,
            &mut (*FcbOrDcb).Mcb,
            FileOffset - 1,
            &mut SourceLbo,
            &mut SourceBytesInRun,
            &mut SourceIndex,
        );
        *FirstSpliceSourceCluster = FatGetIndexFromLbo(Vcb, SourceLbo);

        if Result != 0 && SourceBytesInRun == 1 {
            SourceIndex += 1;
            Result = FatGetNextMcbEntry(
                Vcb,
                &mut (*FcbOrDcb).Mcb,
                SourceIndex,
                &mut SourceVbo,
                &mut SourceLbo,
                &mut SourceBytesInRun,
            );
        } else {
            SourceVbo = FileOffset;
            SourceLbo += 1;
            SourceBytesInRun -= 1;
        }
    }

    if Result == 0 {
        NT_ASSERT(0u32);
        FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }

    *FirstSpliceTargetCluster = TargetCluster;
    *SecondSpliceSourceCluster = *FirstSpliceTargetCluster
        + (BytesToReallocate >> (*Vcb).AllocationSupport.LogOfBytesPerCluster)
        - 1;

    SourceBytesRemaining = BytesToReallocate;
    SourceMcbVbo = 0;
    while SourceBytesRemaining > 0 {
        if SourceMcbVbo != 0 {
            Result = FatGetNextMcbEntry(
                Vcb,
                &mut (*FcbOrDcb).Mcb,
                SourceIndex,
                &mut SourceVbo,
                &mut SourceLbo,
                &mut SourceBytesInRun,
            );
            NT_ASSERT(Result as u32);
        }

        NT_ASSERT((SourceVbo == SourceMcbVbo + FileOffset) as u32);

        SourceMcbBytesInRun = if SourceBytesInRun < SourceBytesRemaining {
            SourceBytesInRun
        } else {
            SourceBytesRemaining
        };

        FatAddMcbEntry(
            Vcb,
            SourceMcb,
            SourceMcbVbo,
            SourceLbo as i64,
            SourceMcbBytesInRun,
        );

        SourceIndex += 1;
        SourceBytesRemaining -= SourceMcbBytesInRun;
        SourceMcbVbo += SourceMcbBytesInRun;
    }

    if SourceMcbBytesInRun < SourceBytesInRun {
        *SecondSpliceTargetCluster =
            FatGetIndexFromLbo(Vcb, SourceLbo + (SourceMcbBytesInRun) as u64);
    } else {
        if FatGetNextMcbEntry(
            Vcb,
            &mut (*FcbOrDcb).Mcb,
            SourceIndex,
            &mut SourceVbo,
            &mut SourceLbo,
            &mut SourceBytesInRun,
        ) != 0
        {
            *SecondSpliceTargetCluster = FatGetIndexFromLbo(Vcb, SourceLbo);
        } else {
            *SecondSpliceTargetCluster = FAT_CLUSTER_LAST;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatAllowExtendedDasdIo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> i32 {
    // PAGED_CODE();

    let IrpSp = IoGetCurrentIrpStackLocation(Irp) as *mut IO_STACK_LOCATION;

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if (FatDecodeFileObject)((*IrpSp).FileObject, &raw mut Vcb, &raw mut Fcb, &raw mut Ccb) != UserVolumeOpen {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if Ccb.is_null() || !((*Ccb).Flags & CCB_FLAG_MANAGE_VOLUME_ACCESS != 0) {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        DebugTrace(
            -1,
            Dbg,
            b"FatAllowExtendedDasdIo -> %08lx\n\0".as_ptr() as *const i8,
            STATUS_INVALID_PARAMETER as u32,
        );
        return STATUS_INVALID_PARAMETER;
    }

    // SetFlag(Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO);
    (*Ccb).Flags |= CCB_FLAG_ALLOW_EXTENDED_DASD_IO;

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn FatGetRetrievalPointerBase(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IrpSp: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let mut BufferLength: u32 = 0;
    let mut RetrievalPointerBase: *mut RETRIEVAL_POINTER_BASE = core::ptr::null_mut();

    // PAGED_CODE();
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    SetFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    if FatDecodeFileObject(
        (*IrpSp).FileObject,
        &mut Vcb,
        &mut Fcb,
        &mut Ccb,
    ) != UserVolumeOpen
    {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    RetrievalPointerBase = (*Irp).AssociatedIrp.SystemBuffer as *mut RETRIEVAL_POINTER_BASE;
    BufferLength = (*IrpSp).Parameters.FileSystemControl.OutputBufferLength;

    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if BufferLength < (core::mem::size_of::<RETRIEVAL_POINTER_BASE>() as u32) {
        FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    core::ptr::write_bytes(RetrievalPointerBase as *mut u8, 0, BufferLength as usize);

    // try / finally (the translation had dropped the finally: the shared Vcb
    // was never released)
    crate::seh::try_finally(&mut (), |_| {
        FatAcquireSharedVcb(IrpContext, Vcb);
        FatQuickVerifyVcb(IrpContext, Vcb);

        (*RetrievalPointerBase).FileAreaOffset.QuadPart =
            (((*Vcb).AllocationSupport.FileAreaLbo as u64) >> ((*Vcb).AllocationSupport.LogOfBytesPerSector as u32)) as i64;
        (*Irp).IoStatus.Information = core::mem::size_of::<RETRIEVAL_POINTER_BASE>() as _;
    }, |_, _| {
        FatReleaseVcb(IrpContext, Vcb);
    });

    FatCompleteRequest(IrpContext, Irp, STATUS_SUCCESS);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn FatGetBootAreaInfo(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> i32 {

    // Forward declarations of unknown externals

    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Vcb: *mut VCB = ptr::null_mut();
    let mut Fcb: *mut FCB = ptr::null_mut();
    let mut Ccb: *mut CCB = ptr::null_mut();
    let mut BufferLength: u32 = 0;
    let BootAreaInfo: *mut BOOT_AREA_INFO;

    // SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
    SetFlag(
        ptr::addr_of_mut!((*IrpContext).Flags),
        IRP_CONTEXT_FLAG_WAIT,
    );

    // if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen)
    if (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        ptr::addr_of_mut!(Vcb),
        ptr::addr_of_mut!(Fcb),
        ptr::addr_of_mut!(Ccb),
    ) != UserVolumeOpen
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // BootAreaInfo = Irp->AssociatedIrp.SystemBuffer;
    BootAreaInfo = (*Irp).AssociatedIrp.SystemBuffer as *mut BOOT_AREA_INFO;
    BufferLength = (*IrpSp).Parameters.FileSystemControl.OutputBufferLength;

    // if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS ))
    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // if (BufferLength < sizeof(BOOT_AREA_INFO))
    if BufferLength < (core::mem::size_of::<BOOT_AREA_INFO>() as u32) {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    // RtlZeroMemory( BootAreaInfo, BufferLength );
    ptr::write_bytes(BootAreaInfo as *mut u8, 0, BufferLength as usize);

    // try / finally
    crate::seh::try_finally(&mut (), |_| {
        FatAcquireSharedVcb(IrpContext, Vcb);
        FatQuickVerifyVcb(IrpContext, Vcb);

        if FatIsFat32(Vcb) != 0 {
            (*BootAreaInfo).BootSectorCount = 2;
            (*BootAreaInfo).BootSectors[0].Offset.QuadPart = 0;
            (*BootAreaInfo).BootSectors[1].Offset.QuadPart = 6;
        } else {
            (*BootAreaInfo).BootSectorCount = 1;
            (*BootAreaInfo).BootSectors[0].Offset.QuadPart = 0;
        }

        (*Irp).IoStatus.Information = core::mem::size_of::<BOOT_AREA_INFO>() as _;
    }, |_, _| {
        FatReleaseVcb(IrpContext, Vcb);
    });

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FatFlushAndCleanVolume(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
    Vcb: *mut VCB,
    FlushType: FAT_FLUSH_TYPE,
) {
    // macros/constants
    const NoFlush: FAT_FLUSH_TYPE = 0;
    const FlushWithoutPurge: FAT_FLUSH_TYPE = 3; // FAT_FLUSH_TYPE (fatprocs.h) — was 1 = Flush
    const VCB_STATE_FLAG_VOLUME_DIRTY: u32 = 0x04;
    const VCB_STATE_FLAG_MOUNTED_DIRTY: u32 = 0x10;
    const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
    const VCB_STATE_FLAG_BOOT_OR_PAGING_FILE: u32 = 0x0800;
    const VolumeClean: i32 = 0;
    const FALSE: i32 = 0;

    // PAGED_CODE(); - no operation

    // NT_ASSERT(FatVcbAcquiredExclusive(IrpContext, Vcb));
    ntkrnl_sys::ASSERT!(ExIsResourceAcquiredExclusiveLite(&raw mut (*Vcb).Resource) != 0);

    // There is no fail, flush everything.
    if FlushType != NoFlush {
        // (VOID) FatFlushVolume(...)
        let _ = FatFlushVolume(IrpContext, Vcb, (FlushType) as _);
    }

    FatCloseEaFile(IrpContext, Vcb, (FALSE) as _);

    // Now, tell the device to flush its buffers.
    if FlushType != NoFlush {
        // (VOID) FatHijackIrpAndFlushDevice(...)
        let _ = FatHijackIrpAndFlushDevice(IrpContext, Irp, (*Vcb).TargetDeviceObject);
    }

    // Now purge everything in sight.
    if FlushType != FlushWithoutPurge {
        CcPurgeCacheSection(
            &raw mut (*Vcb).SectionObjectPointers,
            core::ptr::null(),
            0,
            FALSE,
        );

        // (VOID) FatPurgeReferencedFileObjects(...)
        let _ = FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, (NoFlush) as _);
    }

    // If the volume was dirty and we were allowed to flush...
    if ((*Vcb).VcbState & VCB_STATE_FLAG_VOLUME_DIRTY) != 0 {
        // (VOID) KeCancelTimer(&Vcb->CleanVolumeTimer);
        let _ = KeCancelTimer((&raw mut (*Vcb).CleanVolumeTimer).cast());
        // (VOID) KeRemoveQueueDpc(&Vcb->CleanVolumeDpc);
        let _ = KeRemoveQueueDpc((&raw mut (*Vcb).CleanVolumeDpc).cast());

        if FlushType != NoFlush {
            // The volume is now clean, note it.
            if ((*Vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                FatMarkVolume(IrpContext, Vcb, VolumeClean);
                // ClearFlag(Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY);
                (*Vcb).VcbState &= !VCB_STATE_FLAG_VOLUME_DIRTY;
            }

            // Unlock the volume if it is removable.
            if ((*Vcb).VcbState & VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0
                && ((*Vcb).VcbState & VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) == 0
            {
                FatToggleMediaEjectDisable(IrpContext, Vcb, (FALSE) as _);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatSetPurgeFailureMode(
    mut IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Forward declarations for functions and constants not in ntkrnl_sys list
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;               // placeholder  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const SET_PURGE_FAILURE_MODE_ENABLED: u32 = 0x1;      // placeholder
    const SET_PURGE_FAILURE_MODE_DISABLED: u32 = 0x2;     // placeholder
    const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BA_u32 as i32;

    // Original: PAGED_CODE();
    // will be added by caller or is no-op

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let irp_sp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    // Force WAIT
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

    // Kernel‑only
    if (*Irp).RequestorMode as u32 != (0i32) as u32 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();

    let type_of_open: u32 = (FatDecodeFileObject)(
        (*irp_sp).FileObject,
        core::ptr::addr_of_mut!(vcb),
        core::ptr::addr_of_mut!(fcb),
        core::ptr::addr_of_mut!(ccb),
    );

    if type_of_open == UserDirectoryOpen {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_FILE_IS_A_DIRECTORY);
        return STATUS_FILE_IS_A_DIRECTORY;
    }
    if type_of_open != UserFileOpen {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Input buffer length check
    if (*irp_sp).Parameters.FileSystemControl.InputBufferLength
        < core::mem::size_of::<SET_PURGE_FAILURE_MODE_INPUT>() as u32
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    let set_purge_input: *mut SET_PURGE_FAILURE_MODE_INPUT =
        (*Irp).AssociatedIrp.SystemBuffer as *mut SET_PURGE_FAILURE_MODE_INPUT;

    let input_flags: u32 = (*set_purge_input).Flags;

    // Check that one of the required flags is set
    let mask: u32 = SET_PURGE_FAILURE_MODE_ENABLED | SET_PURGE_FAILURE_MODE_DISABLED;
    if (input_flags & mask) == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // try / finally
    let mut result_status: NTSTATUS = status;
    let mut fcb_acquired: u8 = 0; /* BOOLEAN */
    let p_result_status = core::ptr::addr_of_mut!(result_status);
    let p_fcb_acquired = core::ptr::addr_of_mut!(fcb_acquired);
    crate::seh::try_finally(&mut (), |_| {
        (FatAcquireExclusiveFcb)(IrpContext, fcb);
        *p_fcb_acquired = 1;
        (FatVerifyFcb)(IrpContext, fcb);

        if (input_flags & SET_PURGE_FAILURE_MODE_ENABLED) != 0 {
            if (*fcb).PurgeFailureModeEnableCount == MAXULONG {
                *p_result_status = STATUS_INVALID_PARAMETER;
                return; // try_return
            }
            (*fcb).PurgeFailureModeEnableCount += 1;
        } else {
            // Original: ASSERT(FlagOn(SetPurgeInput->Flags, SET_PURGE_FAILURE_MODE_DISABLED));
            ntkrnl_sys::ASSERT!((input_flags & SET_PURGE_FAILURE_MODE_DISABLED) != 0);

            if (*fcb).PurgeFailureModeEnableCount == 0 {
                *p_result_status = STATUS_INVALID_PARAMETER;
                return; // try_return
            }
            (*fcb).PurgeFailureModeEnableCount -= 1;
        }
        // try_exit: NOTHING;
    }, |_, _| {
        if *p_fcb_acquired != 0 {
            (FatReleaseFcb)(IrpContext, fcb);
        }
    });

    (FatCompleteRequest)(IrpContext, Irp, result_status);
    return result_status;
}
#[no_mangle]
pub unsafe extern "system" fn FatSearchBufferForLabel(
    IrpContext: *mut IRP_CONTEXT,
    Vpb: *mut VPB,
    Buffer: *mut core::ffi::c_void,
    Size: u32,
    LabelFound: *mut u8,
) -> i32 {
    let mut Status: i32;
    let mut UnicodeBuffer: [u16; 11] = [0u16; 11];
    let mut Dirent: *mut DIRENT;
    let mut TerminationDirent: *mut DIRENT;
    let mut VolumeLabelLength: u16;
    let mut OemBuffer: [u8; 11] = [0u8; 11];
    let mut OemString: OEM_STRING = unsafe { core::mem::zeroed() };
    let mut UnicodeString: _UNICODE_STRING = unsafe { core::mem::zeroed() };

    PAGED_CODE();
    // UNREFERENCED_PARAMETER( IrpContext )
    let _ = IrpContext;

    Dirent = Buffer as *mut DIRENT;
    TerminationDirent = Dirent.add((Size as usize) / core::mem::size_of::<DIRENT>());

    while Dirent < TerminationDirent {
        if (*Dirent).FileName[0] == FAT_DIRENT_NEVER_USED {
            Dirent = TerminationDirent;
            break;
        }

        if ((*Dirent).Attributes & !FAT_DIRENT_ATTR_ARCHIVE) == FAT_DIRENT_ATTR_VOLUME_ID
            && (*Dirent).FileName[0] != FAT_DIRENT_DELETED
        {
            break;
        }

        Dirent = Dirent.offset(1);
    }

    if Dirent >= TerminationDirent {
        *LabelFound = 0u8;
        return STATUS_SUCCESS;
    }

    OemString.Buffer = OemBuffer.as_mut_ptr() as *mut i8;
    OemString.MaximumLength = 11;

    core::ptr::copy_nonoverlapping(
        (*Dirent).FileName.as_ptr() as *const u8,
        OemString.Buffer as *mut u8,
        11,
    );

    if *OemString.Buffer == (FAT_DIRENT_REALLY_0E5) as i8 {
        *OemString.Buffer = (0xE5u8) as i8;
    }

    OemString.Length = 11;
    while OemString.Length > 0 {
        if *OemString.Buffer.add(OemString.Length as usize - 1) != (0x00u8) as i8
            && *OemString.Buffer.add(OemString.Length as usize - 1) != (0x20u8) as i8
        {
            break;
        }
        OemString.Length -= 1;
    }

    UnicodeString.MaximumLength = core::mem::size_of::<[u16; 11]>() as u16;
    UnicodeString.Buffer = UnicodeBuffer.as_mut_ptr();

    Status = RtlOemStringToCountedUnicodeString(
        &raw mut UnicodeString as *mut _UNICODE_STRING,
        &raw const OemString as *const OEM_STRING,
        0,
    );

    if !ntkrnl_sys::base::NT_SUCCESS(Status) {
        return Status;
    }

    VolumeLabelLength = UnicodeString.Length;

    if VolumeLabelLength as u32 != (*Vpb).VolumeLabelLength as u32
        || RtlEqualMemory(
            UnicodeBuffer.as_ptr() as *const core::ffi::c_void,
            (*Vpb).VolumeLabel.as_ptr() as *const core::ffi::c_void,
            VolumeLabelLength as u32,
        ) == 0
    {
        return STATUS_WRONG_VOLUME;
    }

    *LabelFound = 1u8;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn FatVerifyLookupFatEntry(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    FatIndex: u32,
    FatEntry: *mut u32,
) {
    let mut PageEntryOffset: u32;
    let mut OffsetIntoVolumeFile: u32;
    let Buffer: *mut core::ffi::c_void;

    // PAGED_CODE(); -- ignored

    // NT_ASSERT(Vcb->AllocationSupport.FatIndexBitSize == 32);
    // (debug assertion omitted)

    FatVerifyIndexIsValid(IrpContext, Vcb, FatIndex);

    Buffer = FsRtlAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned as POOL_TYPE,
        (PAGE_SIZE) as u32,
        TAG_ENTRY_LOOKUP_BUFFER,
    ) as *mut core::ffi::c_void;

    OffsetIntoVolumeFile = FatReservedBytes(&(*Vcb).Bpb as *const BIOS_PARAMETER_BLOCK)
        + FatIndex * (core::mem::size_of::<u32>() as u32);

    PageEntryOffset = (OffsetIntoVolumeFile % (PAGE_SIZE) as u32) / (core::mem::size_of::<u32>() as u32);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        FatPerformVerifyDiskRead(
            IrpContext,
            Vcb,
            Buffer,
            (OffsetIntoVolumeFile & (!(PAGE_SIZE - 1)) as u32) as LBO,
            (PAGE_SIZE) as u32,
            1, // TRUE
        );

        *FatEntry = *((Buffer as *mut u32).offset(PageEntryOffset as isize));
    }, |_, _| {
        ExFreePool(Buffer);
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatScanForDismountedVcb(IrpContext: PIRP_CONTEXT) {
    PAGED_CODE();

    NT_ASSERT(FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32);

    FatAcquireExclusiveGlobal(IrpContext);

    let fatdata = core::ptr::addr_of_mut!(FatData);
    let mut links: *mut LIST_ENTRY = (*fatdata).VcbQueue.Flink;
    let vcb_queue: *mut LIST_ENTRY = core::ptr::addr_of_mut!((*fatdata).VcbQueue);

    while links != vcb_queue {
        let vcb: *mut VCB = ((links as *mut u8)
            .byte_offset(-(core::mem::offset_of!(VCB, VcbLinks) as isize)))
            as *mut VCB;

        links = (*links).Flink;

        if ExAcquireResourceExclusiveLite(
            core::ptr::addr_of!((*vcb).Resource) as *mut ERESOURCE,
            0,
        ) == 0
        {
            continue;
        }

        let vcb_deleted: BOOLEAN = FatCheckForDismount(IrpContext, vcb, 0);

        if vcb_deleted == 0 {
            ExReleaseResourceLite(
                core::ptr::addr_of!((*vcb).Resource) as *mut ERESOURCE,
            );
        }
    }

    FatReleaseGlobal(IrpContext);

    return;
}
#[no_mangle]
pub unsafe extern "system" fn FatSetZeroOnDeallocate(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Constants that would normally come from headers
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // placeholder  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000; // placeholder
    const FCB_STATE_ZERO_ON_DEALLOCATION: u32 = 0x00080000; // placeholder
    // UserFileOpen imported from crate::types (_TYPE_OF_OPEN_UserFileOpen = 2)

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut FcbOrDcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let IrpSp: *mut IO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut ReleaseFcb: u8 = 0; /* BOOLEAN FALSE */

    // PAGED_CODE(); // no-op

    // This call should always be synchronous.
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

    let TypeOfOpen = (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        &mut Vcb,
        &mut FcbOrDcb,
        &mut Ccb,
    );

    if (TypeOfOpen != UserFileOpen) || (*(*IrpSp).FileObject).WriteAccess == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_ACCESS_DENIED);
        return STATUS_ACCESS_DENIED;
    }

    // Readonly mount should be just that: read only.
    if ((*Vcb).VcbState & VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_MEDIA_WRITE_PROTECTED);
        return STATUS_MEDIA_WRITE_PROTECTED;
    }

    // Acquire main then paging to exclude everyone from this FCB.
    (FatAcquireExclusiveFcb)(IrpContext, FcbOrDcb);
    ReleaseFcb = 1; // TRUE

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        (*FcbOrDcb).FcbState |= FCB_STATE_ZERO_ON_DEALLOCATION;
    }, |_, _| {
        if ReleaseFcb != 0 {
            (FatReleaseFcb)(IrpContext, FcbOrDcb);
        }
    });

    (FatCompleteRequest)(IrpContext, Irp, Status);
    return Status;
}

// ============================================================================
// FSCTL handlers that were BUGCHECK STUBS until now.
//
// Each of these sat in rust_cohab_stub.c as `STUB_BUGCHECK()` while the dispatch
// table above routed a live FSCTL straight at it — e.g. FSCTL_LOCK_VOLUME =>
// FatLockVolume. Any usermode caller issuing one of those codes stopped the box.
// They are ordered here as in fsctrl.c. See scan_bugcheck_stubs.py, which computes
// this set instead of paying a build+boot per discovery.
// ============================================================================

/// `FatLockVolumeInternal` (`drivers/filesystems/fastfat/fsctrl.c:3440`).
///
/// Called by anyone wishing to protect the volume for a long duration (PnP is one
/// such user). The volume must be held exclusive by the caller.
#[no_mangle]
pub unsafe extern "system" fn FatLockVolumeInternal(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    const Flush: u32 = 1;
    const VolumeClean: i32 = 0;
    const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022u32 as i32;

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SavedIrql: u8 = 0;
    // A caller-supplied FileObject is itself one of the Vpb references we expect to
    // still be outstanding; a system lock (NULL) has none.
    let RemainingUserReferences: u32 = if !FileObject.is_null() { 1 } else { 0 };
    let ExpectedOpenCount: CLONG = RemainingUserReferences as CLONG;

    NT_ASSERT(
        (ExIsResourceAcquiredExclusiveLite(&raw mut (*Vcb).Resource as *mut _) != 0
            && ExIsResourceAcquiredExclusiveLite(&raw mut FatData.Resource as *mut _) == 0)
            as u32,
    );

    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

    if FatIsHandleCountZero(IrpContext, Vcb) == 0 {
        return STATUS_ACCESS_DENIED;
    }

    FatFlushFat(IrpContext, Vcb);

    // Purge, drop the Vcb, wait out the lazy writer, re-acquire — TWICE. The second
    // pass is not redundant: releasing the Vcb lets the first wait make progress, and
    // work can be queued behind it, so the C does the whole dance again.
    FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, Flush);
    // Close the EA file's internal stream (fsctrl.c keeps this between the purge and the
    // release). It was dropped, so a FAT12/16 volume carrying an `EA DATA. SF` kept one
    // more Vpb reference than the lock check below allows: FSCTL_LOCK_VOLUME, and the
    // dismount/format paths behind it, failed ACCESS_DENIED. Found by `rustconv body-audit`.
    FatCloseEaFile(IrpContext, Vcb, 1);
    FatReleaseVcb(IrpContext, Vcb);
    Status = CcWaitForCurrentLazyWriterActivity();
    FatAcquireExclusiveVcb(IrpContext, Vcb);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    FatPurgeReferencedFileObjects(IrpContext, (*Vcb).RootDcb, Flush);
    FatReleaseVcb(IrpContext, Vcb);
    Status = CcWaitForCurrentLazyWriterActivity();
    FatAcquireExclusiveVcb(IrpContext, Vcb);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    FatFspClose(Vcb);

    IoAcquireVpbSpinLock(&raw mut SavedIrql);

    if ((*(*Vcb).Vpb).Flags as u32 & VPB_LOCKED) == 0
        && (*(*Vcb).Vpb).ReferenceCount <= (2 + RemainingUserReferences) as _
        && (*Vcb).OpenFileCount == ExpectedOpenCount
    {
        (*(*Vcb).Vpb).Flags |= (VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED) as u16;
        (*Vcb).VcbState |= VCB_STATE_FLAG_LOCKED;
        (*Vcb).FileObjectWithVcbLocked = FileObject;
    } else {
        Status = STATUS_ACCESS_DENIED;
    }

    IoReleaseVpbSpinLock(SavedIrql);

    // Having quiesced the volume, take the chance to mark it clean on disk.
    if NT_SUCCESS(Status)
        && ((*Vcb).VcbState & VCB_STATE_FLAG_VOLUME_DIRTY) != 0
        && ((*Vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) == 0
        && CcIsThereDirtyData((*Vcb).Vpb) == 0
    {
        FatMarkVolume(IrpContext, Vcb, VolumeClean);
        (*Vcb).VcbState &= !VCB_STATE_FLAG_VOLUME_DIRTY;
    }

    NT_ASSERT((!NT_SUCCESS(Status) || (*Vcb).OpenFileCount == ExpectedOpenCount) as u32);

    Status
}

/// `FatLockVolume` (`drivers/filesystems/fastfat/fsctrl.c:3237`) — FSCTL_LOCK_VOLUME.
#[no_mangle]
pub unsafe extern "system" fn FatLockVolume(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    const FSRTL_VOLUME_LOCK: u32 = 3;
    const FSRTL_VOLUME_LOCK_FAILED: u32 = 4;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatLockVolume...\n\0".as_ptr() as *const i8);

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if (FatDecodeFileObject(
        (*IrpSp).FileObject,
        &raw mut Vcb,
        &raw mut Fcb,
        &raw mut Ccb,
    )) != UserVolumeOpen
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Announced BEFORE the attempt; the FAILED event below is the paired retraction.
    FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_LOCK);

    if FatAcquireExclusiveVcb(IrpContext, Vcb) == 0 {
        DebugTrace(0, Dbg, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8);
        let Status = FatFsdPostRequest(IrpContext, Irp);
        return Status;
    }

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let p_status = core::ptr::addr_of_mut!(Status);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        *p_status = FatLockVolumeInternal(IrpContext, Vcb, (*IrpSp).FileObject);
    }, |_, abnormal| {
        //  If the user raced a CloseHandle() with us we will have been raised out
        //  of the acquire logic with STATUS_FILE_CLOSED, and the volume will not
        //  be held.
        if !abnormal || ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*Vcb).Resource) as *mut ERESOURCE) != 0 {
            FatReleaseVcb(IrpContext, Vcb);
        }

        if !NT_SUCCESS(*p_status) || abnormal {
            //  The volume lock will be failing.
            FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_LOCK_FAILED);
        }
    });

    (FatCompleteRequest)(IrpContext, Irp, Status);

    DebugTrace(-1, Dbg, b"FatLockVolume -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

/// `FatUnlockVolumeInternal` (`drivers/filesystems/fastfat/fsctrl.c:3608`).
///
/// The volume must be held exclusive by the caller. Attempting to remove a system
/// lock that did not exist is OK — that is the STATUS_NOT_LOCKED default.
#[no_mangle]
pub unsafe extern "system" fn FatUnlockVolumeInternal(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    FileObject: PFILE_OBJECT,
) -> NTSTATUS {
    // "Attempting to remove a system lock that did not exist is OK" — so this is the
    // DEFAULT, not an error path, and it is what the caller sees when the volume was
    // not locked by this FileObject.
    const STATUS_NOT_LOCKED: NTSTATUS = 0xC000002Au32 as i32;

    let _ = IrpContext; // UNREFERENCED_PARAMETER
    let mut SavedIrql: u8 = 0;
    let mut Status: NTSTATUS = STATUS_NOT_LOCKED;

    IoAcquireVpbSpinLock(&raw mut SavedIrql);

    // Vpb->Flags is USHORT, so the mask is built in u32 and narrowed — matching
    // the existing cleanup.rs:694 spelling of the same clear.
    if ((*(*Vcb).Vpb).Flags as u32 & VPB_LOCKED) != 0 && FileObject == (*Vcb).FileObjectWithVcbLocked
    {
        (*(*Vcb).Vpb).Flags &= !((VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED) as u16);
        (*Vcb).VcbState &= !VCB_STATE_FLAG_LOCKED;
        (*Vcb).FileObjectWithVcbLocked = core::ptr::null_mut();

        Status = STATUS_SUCCESS;
    }

    IoReleaseVpbSpinLock(SavedIrql);

    Status
}

/// `FatUnlockVolume` (`drivers/filesystems/fastfat/fsctrl.c:3361`).
#[no_mangle]
pub unsafe extern "system" fn FatUnlockVolume(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatUnlockVolume...\n\0".as_ptr() as *const i8);

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        &raw mut Vcb,
        &raw mut Fcb,
        &raw mut Ccb,
    ) != UserVolumeOpen
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    let Status = FatUnlockVolumeInternal(IrpContext, Vcb, (*IrpSp).FileObject);

    if NT_SUCCESS(Status) {
        FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_UNLOCK);
    }

    (FatCompleteRequest)(IrpContext, Irp, Status);

    DebugTrace(-1, Dbg, b"FatUnlockVolume -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

/// `FatDirtyVolume` (`drivers/filesystems/fastfat/fsctrl.c:3886`) — FSCTL_MARK_VOLUME_DIRTY.
#[no_mangle]
pub unsafe extern "system" fn FatDirtyVolume(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    const VolumeDirty: i32 = 1;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatDirtyVolume...\n\0".as_ptr() as *const i8);

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        &raw mut Vcb,
        &raw mut Fcb,
        &raw mut Ccb,
    ) != UserVolumeOpen
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    // Disable PopUps, we want to return any error.
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_DISABLE_POPUPS;

    // FatVerifyVcb does not return a status — it FatRaiseStatus()es on a bad or
    // needs-verify Vcb, so there is deliberately nothing to check here.
    FatVerifyVcb(IrpContext, Vcb);

    (*Vcb).VcbState |= VCB_STATE_FLAG_MOUNTED_DIRTY;

    FatMarkVolume(IrpContext, Vcb, VolumeDirty);

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);

    DebugTrace(-1, Dbg, b"FatDirtyVolume -> STATUS_SUCCESS\n\0".as_ptr() as *const i8);
    STATUS_SUCCESS
}

/// `FatIsVolumeMounted` (`drivers/filesystems/fastfat/fsctrl.c:4103`).
#[no_mangle]
pub unsafe extern "system" fn FatIsVolumeMounted(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let Status: NTSTATUS = STATUS_SUCCESS;

    DebugTrace(1, Dbg, b"FatIsVolumeMounted...\n\0".as_ptr() as *const i8);

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    // The C ignores the TYPE_OF_OPEN here and only asserts Vcb != NULL.
    (FatDecodeFileObject)(
        (*IrpSp).FileObject,
        &raw mut Vcb,
        &raw mut Fcb,
        &raw mut Ccb,
    );

    NT_ASSERT(!Vcb.is_null() as u32);

    // Disable PopUps, we want to return any error.
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_DISABLE_POPUPS;

    // Raises on a volume that needs verifying; that IS the answer this FSCTL gives.
    FatVerifyVcb(IrpContext, Vcb);

    (FatCompleteRequest)(IrpContext, Irp, Status);

    DebugTrace(-1, Dbg, b"FatIsVolumeMounted -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

/// `FatIsPathnameValid` (`drivers/filesystems/fastfat/fsctrl.c:4175`).
///
/// Intentionally a no-op in the C, and required to be correct on a FALSE return —
/// the previous FAT implementation violated that and could reject a createable
/// pathname, so returning success unconditionally is the deliberate behaviour.
#[no_mangle]
pub unsafe extern "system" fn FatIsPathnameValid(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    DebugTrace(1, Dbg, b"FatIsPathnameValid...\n\0".as_ptr() as *const i8);

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);

    DebugTrace(-1, Dbg, b"FatIsPathnameValid -> %08lx\n\0".as_ptr() as *const i8);
    STATUS_SUCCESS
}

/// `FatDismountVolume` (`drivers/filesystems/fastfat/fsctrl.c:3670`) — FSCTL_DISMOUNT_VOLUME.
#[no_mangle]
pub unsafe extern "system" fn FatDismountVolume(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    const FSRTL_VOLUME_DISMOUNT: u32 = 1;
    const FSRTL_VOLUME_DISMOUNT_FAILED: u32 = 2;
    const FlushAndInvalidate: FAT_FLUSH_TYPE = 2; // fatprocs.h:1052 enum
    const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022u32 as i32;
    const STATUS_VOLUME_DISMOUNTED: NTSTATUS = 0xC000026Eu32 as i32;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut VcbHeld = false;
    let mut SavedIrql: u8 = 0;

    DebugTrace(1, Dbg, b"FatDismountVolume...\n\0".as_ptr() as *const i8);

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    // The C reaches the shared `fn_return:` tail via goto for every check below
    // EXCEPT the Ccb one, which completes and returns inline. Same observable
    // behaviour; kept as written so a diff against fsctrl.c stays honest.
    'fn_return: {
        if (FatDecodeFileObject(
            (*IrpSp).FileObject,
            &raw mut Vcb,
            &raw mut Fcb,
            &raw mut Ccb,
        )) != UserVolumeOpen
        {
            Status = STATUS_INVALID_PARAMETER;
            break 'fn_return;
        }

        if Ccb.is_null() || FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
            (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
            return STATUS_INVALID_PARAMETER;
        }

        // Refuse to pull the volume out from under the boot or paging file.
        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE) != 0 {
            Status = STATUS_ACCESS_DENIED;
            break 'fn_return;
        }

        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED) != 0 {
            Status = STATUS_VOLUME_DISMOUNTED;
            break 'fn_return;
        }

        FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_DISMOUNT);

        (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT;

        FatAcquireExclusiveGlobal(IrpContext);

        let p_status = core::ptr::addr_of_mut!(Status);
        let p_vcb_held = core::ptr::addr_of_mut!(VcbHeld);

        // _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            //  This can raise if a cleanup on the fileobject we got races in
            //  ahead of us.
            FatAcquireExclusiveVolume(IrpContext, Vcb);
            *p_vcb_held = true;

            // Re-test under the volume lock: the check before the acquire is only an
            // optimization, and another dismount can have completed in between.
            if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED) != 0 {
                *p_status = STATUS_VOLUME_DISMOUNTED;
                return; // try_return
            }

            FatFlushAndCleanVolume(IrpContext, Irp, Vcb, FlushAndInvalidate);

            (*Ccb).Flags |= CCB_FLAG_COMPLETE_DISMOUNT;

            FatSetVcbCondition(Vcb, VcbBad);
            (*Vcb).VcbState |= VCB_STATE_FLAG_VOLUME_DISMOUNTED;

            IoAcquireVpbSpinLock(&raw mut SavedIrql);
            (*(*Vcb).Vpb).Flags |= VPB_DIRECT_WRITES_ALLOWED as u16;
            IoReleaseVpbSpinLock(SavedIrql);

            *p_status = STATUS_SUCCESS;
        }, |_, abnormal| {
            // The NTDDI_WIN8 FsRtlDismountComplete call is NOT translated: this tree
            // targets 0x502, so it is #if'd out of the C too.
            if *p_vcb_held {
                FatReleaseVolume(IrpContext, Vcb);
            }
            FatReleaseGlobal(IrpContext);

            // An already-dismounted volume is NOT a dismount failure — hence the
            // explicit exclusion rather than a plain !NT_SUCCESS test.
            if (!NT_SUCCESS(*p_status) && *p_status != STATUS_VOLUME_DISMOUNTED) || abnormal {
                FsRtlNotifyVolumeEvent((*IrpSp).FileObject, FSRTL_VOLUME_DISMOUNT_FAILED);
            }
        });
    }

    (FatCompleteRequest)(IrpContext, Irp, Status);

    DebugTrace(-1, Dbg, b"FatDismountVolume -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

/// `FatPerformVerifyDiskRead` (`drivers/filesystems/fastfat/fsctrl.c:4576`).
///
/// Reads a byte range straight off the disk, bypassing the cache and the regular I/O
/// path, and **overriding the device's verify-volume flag** — that override is the
/// whole point of the function, which is why it builds the IRP by hand.
///
/// Returns TRUE on success. `ReturnOnError` chooses between reporting a failure and
/// raising it.
#[no_mangle]
pub unsafe extern "system" fn FatPerformVerifyDiskRead(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    Buffer: *mut c_void,
    Lbo: LBO,
    NumberOfBytesToRead: u32,
    ReturnOnError: i32,
) -> i32 {
    // PAGED_CODE();
    const IRP_MJ_READ: u8 = 0x03;
    const SL_OVERRIDE_VERIFY_VOLUME: u8 = 0x02;
    const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as i32;
    const STATUS_INVALID_PARAMETER_C: NTSTATUS = 0xC000000Du32 as i32;
    const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016u32 as i32;
    const STATUS_PENDING: NTSTATUS = 0x00000103;
    const KernelMode: i8 = 0;

    DebugTrace(
        0,
        Dbg,
        b"FatPerformVerifyDiskRead, Lbo = %08lx\n\0".as_ptr() as *const i8,
    );

    let mut Event: KEVENT = core::mem::zeroed();
    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();

    KeInitializeEvent(
        &raw mut Event,
        _EVENT_TYPE_NotificationEvent,
        0, /* FALSE */
    );

    ByteOffset.QuadPart = Lbo as i64;

    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_READ as u32,
        (*Vcb).TargetDeviceObject,
        Buffer,
        NumberOfBytesToRead,
        &raw mut ByteOffset,
        &raw mut Event,
        &raw mut Iosb,
    );

    if Irp.is_null() {
        FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
    }

    // THE point of this routine: tell the disk driver to service the read even though
    // the volume is flagged as needing verification.
    (*IoGetNextIrpStackLocation(Irp)).Flags |= SL_OVERRIDE_VERIFY_VOLUME;

    let mut Status = IoCallDriver((*Vcb).TargetDeviceObject, Irp);

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as *mut c_void,
            _KWAIT_REASON_Executive,
            KernelMode,
            0, /* FALSE */
            core::ptr::null_mut(),
        );
        Status = Iosb.__bindgen_anon_1.Status;
    }

    // The override above is exactly what makes this assertion meaningful: having
    // suppressed verify, the driver must never come back asking for one.
    NT_ASSERT((Status != STATUS_VERIFY_REQUIRED) as u32);

    // STATUS_INVALID_PARAMETER is reported as a plain FALSE regardless of
    // ReturnOnError — a read past the end of a small volume is not exceptional.
    if Status == STATUS_INVALID_PARAMETER_C {
        return 0;
    }

    if !NT_SUCCESS(Status) {
        if ReturnOnError != 0 {
            return 0;
        } else {
            FatNormalizeAndRaiseStatus(IrpContext, Status);
        }
    }

    1 /* TRUE */
}

/// `FatIsVolumeDirty` (`drivers/filesystems/fastfat/fsctrl.c:3972`) — FSCTL_IS_VOLUME_DIRTY.
#[no_mangle]
pub unsafe extern "system" fn FatIsVolumeDirty(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    const VOLUME_IS_DIRTY: u32 = 0x00000001; // iotypes.h:6454
    const MdlMappingNoExecute: u32 = 0x40000000;
    const LowPagePriority: u32 = 0;
    const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as i32;
    const STATUS_INVALID_USER_BUFFER: NTSTATUS = 0xC00000E8u32 as i32;
    const STATUS_VOLUME_DISMOUNTED: NTSTATUS = 0xC000026Eu32 as i32;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    // The output buffer arrives either already mapped (SystemBuffer) or as an MDL.
    // Order matters: SystemBuffer is checked FIRST, and only a request carrying
    // neither is STATUS_INVALID_USER_BUFFER.
    let VolumeState: *mut u32 = if !(*Irp).AssociatedIrp.SystemBuffer.is_null() {
        (*Irp).AssociatedIrp.SystemBuffer as *mut u32
    } else if !(*Irp).MdlAddress.is_null() {
        let p = MmGetSystemAddressForMdlSafe(
            (*Irp).MdlAddress,
            LowPagePriority | MdlMappingNoExecute,
        ) as *mut u32;
        if p.is_null() {
            (FatCompleteRequest)(IrpContext, Irp, STATUS_INSUFFICIENT_RESOURCES);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        p
    } else {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_USER_BUFFER);
        return STATUS_INVALID_USER_BUFFER;
    };

    if ((*IrpSp).Parameters.FileSystemControl.OutputBufferLength as usize)
        < core::mem::size_of::<u32>()
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();

    if (FatDecodeFileObject(
        (*IrpSp).FileObject,
        &raw mut Vcb,
        &raw mut Fcb,
        &raw mut Ccb,
    )) != UserVolumeOpen
    {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    if (*Vcb).VcbCondition != VcbGood {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_VOLUME_DISMOUNTED);
        return STATUS_VOLUME_DISMOUNTED;
    }

    // Disable PopUps, we want to return any error.
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_DISABLE_POPUPS;

    FatVerifyVcb(IrpContext, Vcb);

    // Note the C only ever SETS the bit — it never clears the caller's buffer first.
    if ((*Vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) != 0 {
        *VolumeState |= VOLUME_IS_DIRTY;
    }

    (*Irp).IoStatus.Information = core::mem::size_of::<u32>() as _;

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);
    STATUS_SUCCESS
}

/// `FatGetStatistics` (`drivers/filesystems/fastfat/fsctrl.c:4899`) —
/// FSCTL_FILESYSTEM_GET_STATISTICS. Returns the per-processor performance counters.
#[no_mangle]
pub unsafe extern "system" fn FatGetStatistics(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatGetStatistics...\n\0".as_ptr() as *const i8);

    let BufferLength = (*IrpSp).Parameters.FileSystemControl.OutputBufferLength as u32;
    let Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    // Two different structs, deliberately: the floor is the COMMON header
    // (FILESYSTEM_STATISTICS), while the full payload is one FILE_SYSTEM_STATISTICS
    // per processor. Anything at least the header size is answerable, possibly
    // truncated with STATUS_BUFFER_OVERFLOW.
    if (BufferLength as usize) < core::mem::size_of::<FILESYSTEM_STATISTICS>() {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    let StatsSize =
        (core::mem::size_of::<FILE_SYSTEM_STATISTICS>() as u32).wrapping_mul(FatData.NumberProcessors);

    let (BytesToCopy, Status): (u32, NTSTATUS) = if BufferLength < StatsSize {
        (BufferLength, STATUS_BUFFER_OVERFLOW)
    } else {
        (StatsSize, STATUS_SUCCESS)
    };

    // Vcb is EMBEDDED in the volume device object.
    let Vcb: *mut VCB = &raw mut (*((*IrpSp).DeviceObject as *mut VOLUME_DEVICE_OBJECT)).Vcb;

    RtlCopyMemory(
        Buffer as *mut c_void,
        (*Vcb).Statistics as *const c_void,
        BytesToCopy as usize,
    );

    (*Irp).IoStatus.Information = BytesToCopy as _;

    (FatCompleteRequest)(IrpContext, Irp, Status);

    DebugTrace(-1, Dbg, b"FatGetStatistics -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

/// `FatQueryBpb` (`drivers/filesystems/fastfat/fsctrl.c:4220`) — returns the first
/// 0x24 bytes of sector 0.
#[no_mangle]
pub unsafe extern "system" fn FatQueryBpb(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    // PAGED_CODE();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatQueryBpb...\n\0".as_ptr() as *const i8);

    // Vcb is EMBEDDED in the volume device object, so this is a field address, not
    // a decode of the file object.
    let Vcb: *mut VCB = &raw mut (*((*IrpSp).DeviceObject as *mut VOLUME_DEVICE_OBJECT)).Vcb;

    if (*Vcb).First0x24BytesOfBootSector.is_null() {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let BpbBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFSCTL_QUERY_FAT_BPB_BUFFER;

    // Length check AFTER the NULL check, exactly as the C orders them.
    if ((*IrpSp).Parameters.FileSystemControl.OutputBufferLength as u32) < 0x24 {
        (FatCompleteRequest)(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
        return STATUS_BUFFER_TOO_SMALL;
    }

    RtlCopyMemory(
        (*BpbBuffer).First0x24BytesOfBootSector.as_mut_ptr() as *mut c_void,
        (*Vcb).First0x24BytesOfBootSector as *const c_void,
        0x24,
    );

    (*Irp).IoStatus.Information = 0x24;

    (FatCompleteRequest)(IrpContext, Irp, STATUS_SUCCESS);

    DebugTrace(-1, Dbg, b"FatQueryBpb -> %08lx\n\0".as_ptr() as *const i8);
    STATUS_SUCCESS
}

/// FSCTL_MARK_HANDLE — attach special properties to a user handle (fsctrl.c:7311).
///
/// Was a `STUB_BUGCHECK()` in rust_cohab_stub.c and reachable straight from the
/// dispatch table, so any caller of this FSCTL stopped the box.
///
/// Note the C only ever supports MARK_HANDLE_PROTECT_CLUSTERS; the USN_SOURCE_*
/// bits are accepted and then ignored, deliberately, "to preserve compatibility"
/// with callers that set them. Keep that — rejecting them would be a behaviour
/// change dressed as a tidy-up.
#[no_mangle]
pub unsafe extern "system" fn FatMarkHandle(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    const MARK_HANDLE_PROTECT_CLUSTERS: u32 = 0x00000001; // ntifs.h
    const USN_SOURCE_DATA_MANAGEMENT: u32 = 0x00000001;
    const USN_SOURCE_AUXILIARY_DATA: u32 = 0x00000002;
    const USN_SOURCE_REPLICATION_MANAGEMENT: u32 = 0x00000004;
    const IRP_MN_KERNEL_CALL: u8 = 0x04; // ntifs.h:4421
    const FCB_STATE_DENY_DEFRAG: u32 = 0x00004000; // fatstruc.h:1217
    const CCB_FLAG_DENY_DEFRAG: u32 = 0x00080000; // fatstruc.h:1352

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Vcb: *mut VCB = null_mut();
    let mut Fcb: *mut FCB = null_mut();
    let mut Ccb: *mut CCB = null_mut();
    let mut DasdFcb: *mut FCB = null_mut();
    let mut DasdCcb: *mut CCB = null_mut();
    let mut DasdFileObject: *mut FILE_OBJECT = null_mut();
    let mut ReleaseFcb: bool = false;

    // PAGED_CODE();
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg, b"FatMarkHandle...\n\0".as_ptr() as *const i8);

    //  Always make this a synchronous IRP.
    SetFlag(&raw mut (*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

    let mut TypeOfOpen = FatDecodeFileObject((*IrpSp).FileObject, &mut Vcb, &mut Fcb, &mut Ccb);

    //  Files and directories only.
    if TypeOfOpen != UserFileOpen && TypeOfOpen != UserDirectoryOpen {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    //  Win32/64 thunking. Gated exactly as FatMoveFile's is: the C guards it with
    //  `#if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)`, and on i386
    //  PMARK_HANDLE_INFO32 is only a placeholder (ddk_shims.rs:228).
    let mut LocalMarkHandleInfo: MARK_HANDLE_INFO = core::mem::zeroed();
    let mut thunked = false;
    #[cfg(all(target_pointer_width = "64", feature = "wow64"))]
    {
        if IoIs32bitProcess(Irp) != 0 {
            if ((*IrpSp).Parameters.FileSystemControl.InputBufferLength as usize)
                < core::mem::size_of::<ntkrnl_sys::ddk::MARK_HANDLE_INFO32>()
            {
                FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
                return STATUS_BUFFER_TOO_SMALL;
            }
            let MarkHandle32 = (*Irp).AssociatedIrp.SystemBuffer as PMARK_HANDLE_INFO32;
            LocalMarkHandleInfo.HandleInfo = (*MarkHandle32).HandleInfo;
            LocalMarkHandleInfo.UsnSourceInfo = (*MarkHandle32).UsnSourceInfo;
            //  The C sign-extends through LONG before widening — a 32-bit guest
            //  handle is signed there.
            LocalMarkHandleInfo.VolumeHandle =
                ((*MarkHandle32).VolumeHandle as i32) as isize as HANDLE;
            thunked = true;
        }
    }

    let HandleInfo: PMARK_HANDLE_INFO = if thunked {
        &raw mut LocalMarkHandleInfo
    } else {
        if ((*IrpSp).Parameters.FileSystemControl.InputBufferLength as usize)
            < core::mem::size_of::<MARK_HANDLE_INFO>()
        {
            FatCompleteRequest(IrpContext, Irp, STATUS_BUFFER_TOO_SMALL);
            return STATUS_BUFFER_TOO_SMALL;
        }
        (*Irp).AssociatedIrp.SystemBuffer as PMARK_HANDLE_INFO
    };

    //  Only legal bits may be set. (The middle `FlagOn(..., 0)` term is the C's
    //  own vestigial no-op — it can never be true — and is kept for shape.)
    if FlagOn((*HandleInfo).HandleInfo, !MARK_HANDLE_PROTECT_CLUSTERS) != 0
        || FlagOn(
            (*HandleInfo).UsnSourceInfo,
            !(USN_SOURCE_DATA_MANAGEMENT
                | USN_SOURCE_AUXILIARY_DATA
                | USN_SOURCE_REPLICATION_MANAGEMENT),
        ) != 0
    {
        FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
        return STATUS_INVALID_PARAMETER;
    }

    //  The caller needs a valid volume handle, the manage-volume privilege, or
    //  kernel mode. (The kernel-mode check is only sound because the rdr does not
    //  support this FSCTL — the C says so and it is still true here.)
    if (*Irp).RequestorMode != 0 /* KernelMode == 0 */
        && (*IrpSp).MinorFunction != IRP_MN_KERNEL_CALL
        && FlagOn((*Ccb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0
        && (FlagOn((*HandleInfo).HandleInfo, MARK_HANDLE_PROTECT_CLUSTERS) != 0
            || (*HandleInfo).UsnSourceInfo != 0)
    {
        if (*HandleInfo).VolumeHandle.is_null() {
            FatCompleteRequest(IrpContext, Irp, STATUS_ACCESS_DENIED);
            return STATUS_ACCESS_DENIED;
        }

        Status = ObReferenceObjectByHandle(
            (*HandleInfo).VolumeHandle,
            0,
            IoFileObjectType as *mut _,
            1, /* UserMode */
            &mut DasdFileObject as *mut *mut FILE_OBJECT as *mut PVOID,
            null_mut(),
        );

        if !NT_SUCCESS(Status) {
            FatCompleteRequest(IrpContext, Irp, Status);
            return Status;
        }

        //  Must be the same volume as the handle this FSCTL came in on.
        if (*DasdFileObject).Vpb != (*Vcb).Vpb {
            ObDereferenceObject(DasdFileObject as *mut _);
            FatCompleteRequest(IrpContext, Irp, STATUS_INVALID_PARAMETER);
            return STATUS_INVALID_PARAMETER;
        }

        //  Decode it and verify it really is a volume handle. Dismounts are not
        //  raised on here on purpose; the check below covers that anyway.
        TypeOfOpen = FatDecodeFileObject(DasdFileObject, &mut Vcb, &mut DasdFcb, &mut DasdCcb);
        let _ = TypeOfOpen;

        ObDereferenceObject(DasdFileObject as *mut _);

        if DasdCcb.is_null() || FlagOn((*DasdCcb).Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS) == 0 {
            FatCompleteRequest(IrpContext, Irp, STATUS_ACCESS_DENIED);
            return STATUS_ACCESS_DENIED;
        }
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_release_fcb = core::ptr::addr_of_mut!(ReleaseFcb);
    crate::seh::try_finally(&mut (), |_| {
        FatAcquireExclusiveFcb(IrpContext, Fcb);
        *p_release_fcb = true;

        FatVerifyFcb(IrpContext, Fcb);

        if ((*HandleInfo).HandleInfo & MARK_HANDLE_PROTECT_CLUSTERS) != 0 {
            if ((*Fcb).FcbState & FCB_STATE_DENY_DEFRAG) != 0 {
                //  It's already set, bail out.
                *p_status = STATUS_ACCESS_DENIED;
                return; // try_return
            }

            (*Ccb).Flags |= CCB_FLAG_DENY_DEFRAG;
            (*Fcb).FcbState |= FCB_STATE_DENY_DEFRAG;
        }
        // try_exit: NOTHING
    }, |_, _| {
        if *p_release_fcb {
            FatReleaseFcb(IrpContext, Fcb);
        }
    });

    FatCompleteRequest(IrpContext, Irp, Status);
    DebugTrace(-1, Dbg, b"FatMarkHandle -> %08lx\n\0".as_ptr() as *const i8);
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     FsCtrl.c
//
// Abstract:
//
//     This module implements the File System Control routines for Fat called
//     by the dispatch driver.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_FSCTRL)
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_FSCTRL)
//
// //
// //  Local procedure prototypes
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMountVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT TargetDeviceObject,
//     IN PVPB Vpb,
//     IN PDEVICE_OBJECT FsDeviceObject
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatVerifyVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
/* BOOLEAN */
// FatIsMediaWriteProtected (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT TargetDeviceObject
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatUserFsCtrl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatOplockRequest (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatLockVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatUnlockVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatDismountVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatDirtyVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatIsVolumeDirty (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatIsVolumeMounted (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatIsPathnameValid (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatInvalidateVolumes (
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatScanForDismountedVcb (
//     IN PIRP_CONTEXT IrpContext
//     );
//
/* BOOLEAN */
// FatPerformVerifyDiskRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PVOID Buffer,
//     IN LBO Lbo,
//     IN ULONG NumberOfBytesToRead,
//     IN BOOLEAN ReturnOnError
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatQueryRetrievalPointers (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatQueryBpb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatGetStatistics (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// NTSTATUS
// FatAllowExtendedDasdIo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetBootAreaInfo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetRetrievalPointerBase (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMarkHandle(
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// NTSTATUS
// FatSetZeroOnDeallocate (
//     __in PIRP_CONTEXT IrpContext,
//     __in PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetPurgeFailureMode (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     );
//
// //
// //  Local support routine prototypes
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetVolumeBitmap (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetRetrievalPointers (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatMoveFileNeedsWriteThrough (
//      PIRP_CONTEXT IrpContext,
//      PFCB FcbOrDcb,
//      ULONG OldWriteThroughFlags
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMoveFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     );
//
// VOID
// FatComputeMoveFileSplicePoints (
//     PIRP_CONTEXT IrpContext,
//     PFCB FcbOrDcb,
//     ULONG FileOffset,
//     ULONG TargetCluster,
//     ULONG BytesToReallocate,
//     PULONG FirstSpliceSourceCluster,
//     PULONG FirstSpliceTargetCluster,
//     PULONG SecondSpliceSourceCluster,
//     PULONG SecondSpliceTargetCluster,
//     PLARGE_MCB SourceMcb
// );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatComputeMoveFileParameter (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN ULONG BufferSize,
//     IN ULONG FileOffset,
//     IN OUT PULONG ByteCount,
//     OUT PULONG BytesToReallocate,
//     OUT PULONG BytesToWrite,
//     OUT PLARGE_INTEGER SourceLbo
// );
//
// NTSTATUS
// FatSearchBufferForLabel(
//     IN  PIRP_CONTEXT IrpContext,
//     IN  PVPB  Vpb,
//     IN  PVOID Buffer,
//     IN  ULONG Size,
//     OUT PBOOLEAN LabelFound
// );
//
// VOID
// FatVerifyLookupFatEntry (
//     IN  PIRP_CONTEXT IrpContext,
//     IN  PVCB Vcb,
//     IN  ULONG FatIndex,
//     IN OUT PULONG FatEntry
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatAddMcbEntry)
// #pragma alloc_text(PAGE, FatAllowExtendedDasdIo)
// #pragma alloc_text(PAGE, FatCommonFileSystemControl)
// #pragma alloc_text(PAGE, FatComputeMoveFileParameter)
// #pragma alloc_text(PAGE, FatComputeMoveFileSplicePoints)
// #pragma alloc_text(PAGE, FatDirtyVolume)
// #pragma alloc_text(PAGE, FatFsdFileSystemControl)
// #pragma alloc_text(PAGE, FatGetRetrievalPointerBase)
// #pragma alloc_text(PAGE, FatGetBootAreaInfo)
// #pragma alloc_text(PAGE, FatMarkHandle)
// #pragma alloc_text(PAGE, FatGetRetrievalPointers)
// #pragma alloc_text(PAGE, FatGetStatistics)
// #pragma alloc_text(PAGE, FatGetVolumeBitmap)
// #pragma alloc_text(PAGE, FatIsMediaWriteProtected)
// #pragma alloc_text(PAGE, FatIsPathnameValid)
// #pragma alloc_text(PAGE, FatIsVolumeDirty)
// #pragma alloc_text(PAGE, FatIsVolumeMounted)
// #pragma alloc_text(PAGE, FatLockVolume)
// #pragma alloc_text(PAGE, FatLookupLastMcbEntry)
// #pragma alloc_text(PAGE, FatGetNextMcbEntry)
// #pragma alloc_text(PAGE, FatMountVolume)
// #pragma alloc_text(PAGE, FatMoveFileNeedsWriteThrough)
// #pragma alloc_text(PAGE, FatMoveFile)
// #pragma alloc_text(PAGE, FatOplockRequest)
// #pragma alloc_text(PAGE, FatPerformVerifyDiskRead)
// #pragma alloc_text(PAGE, FatQueryBpb)
// #pragma alloc_text(PAGE, FatQueryRetrievalPointers)
// #pragma alloc_text(PAGE, FatRemoveMcbEntry)
// #pragma alloc_text(PAGE, FatScanForDismountedVcb)
// #pragma alloc_text(PAGE, FatFlushAndCleanVolume)
// #pragma alloc_text(PAGE, FatSearchBufferForLabel)
// #pragma alloc_text(PAGE, FatSetPurgeFailureMode)
// #pragma alloc_text(PAGE, FatUnlockVolume)
// #pragma alloc_text(PAGE, FatUserFsCtrl)
// #pragma alloc_text(PAGE, FatVerifyLookupFatEntry)
// #pragma alloc_text(PAGE, FatVerifyVolume)
// #endif
//
// #if DBG
// pub static mut FatMoveFileDebug: BOOLEAN = 0;  // defined above at top of file
// #endif
//
// //
// //  These wrappers go around the MCB package; we scale the LBO's passed
// //  in (which can be bigger than 32 bits on fat32) by the volume's sector
// //  size.
// //
// //  Note we now use the real large mcb package.  This means these shims
// //  now also convert the -1 unused LBN number to the 0 of the original
// //  mcb package.
// //
//
// #define     MCB_SCALE_LOG2      (Vcb->AllocationSupport.LogOfBytesPerSector)
// #define     MCB_SCALE           (1 << MCB_SCALE_LOG2)
// #define     MCB_SCALE_MODULO    (MCB_SCALE - 1)
//
/* BOOLEAN */
// FatNonSparseMcb(
//      PVCB Vcb,
//      PLARGE_MCB Mcb,
//      PVBO Vbo,
//      PLONGLONG ByteCount
//     )
// {
//     LBO Lbo;
//     ULONG Index = 0;
//     LONGLONG llVbo = 0;
//
//     UNREFERENCED_PARAMETER(Vcb);
//
//     while (FsRtlGetNextLargeMcbEntry(Mcb, Index, &llVbo, &Lbo, ByteCount)) {
//         *Vbo = (VBO)llVbo;
//         if (((ULONG)Lbo) == -1) {
//             return FALSE;
//         }
//
//         Index++;
//     }
//
//     *Vbo = (VBO)llVbo;
//
//     return TRUE;
// }
//
//
/* BOOLEAN */
// FatAddMcbEntry (
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     IN VBO Vbo,
//     IN LBO Lbo,
//     IN ULONG SectorCount
//     )
//
// {
//     BOOLEAN Result;
// #if DBG
//     VBO SparseVbo;
//     LONGLONG SparseByteCount;
// #endif
//
//     PAGED_CODE();
//
//     if (SectorCount) {
//
//         //
//         //  Round up sectors, but be careful as SectorCount approaches 4Gb.
//         //  Note that for x>0, (x+m-1)/m = ((x-1)/m)+(m/m) = ((x-1)/m)+1
//         //
//
//         SectorCount--;
//         SectorCount >>= MCB_SCALE_LOG2;
//         SectorCount++;
//     }
//
//     Vbo >>= MCB_SCALE_LOG2;
//     Lbo >>= MCB_SCALE_LOG2;
//
//     NT_ASSERT( SectorCount != 0 );
//
//     if (Mcb != &Vcb->DirtyFatMcb) {
//         NT_ASSERT( FatNonSparseMcb( Vcb, Mcb, &SparseVbo, &SparseByteCount ) ||
//                 ((SparseVbo == Vbo) && (SparseByteCount == SectorCount )) );
//     }
//
//     Result = FsRtlAddLargeMcbEntry( Mcb,
//                                   ((LONGLONG) Vbo),
//                                   ((LONGLONG) Lbo),
//                                   ((LONGLONG) SectorCount) );
//
//     if (Mcb != &Vcb->DirtyFatMcb) {
//         NT_ASSERT( FatNonSparseMcb( Vcb, Mcb, &SparseVbo, &SparseByteCount ) ||
//                 ((SparseVbo == Vbo) && (SparseByteCount == SectorCount )) );
//     }
//
//     return Result;
// }
//
//
/* BOOLEAN */
// FatLookupMcbEntry (
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     IN VBO Vbo,
//     OUT PLBO Lbo,
//     OUT PULONG ByteCount OPTIONAL,
//     OUT PULONG Index OPTIONAL
//     )
// {
//     BOOLEAN Results;
//     LONGLONG LiLbo;
//     LONGLONG LiSectorCount;
//     ULONG Remainder;
//
//     LiLbo = 0;
//     LiSectorCount = 0;
//
//     Remainder = Vbo & MCB_SCALE_MODULO;
//
//     Results = FsRtlLookupLargeMcbEntry( Mcb,
//                                         (Vbo >> MCB_SCALE_LOG2),
//                                         &LiLbo,
//                                         ARGUMENT_PRESENT(ByteCount) ? &LiSectorCount : NULL,
//                                         NULL,
//                                         NULL,
//                                         Index );
//
//     if ((ULONG) LiLbo != -1) {
//
//         *Lbo = (((LBO) LiLbo) << MCB_SCALE_LOG2);
//
//         if (Results) {
//
//             *Lbo += Remainder;
//         }
//
//     } else {
//
//         *Lbo = 0;
//     }
//
//     if (ARGUMENT_PRESENT(ByteCount)) {
//
//         *ByteCount = (ULONG) LiSectorCount;
//
//         if (*ByteCount) {
//
//             *ByteCount <<= MCB_SCALE_LOG2;
//
//             //
//             //  If ByteCount overflows, then this is likely the case of
//             //  a file of max-supported size (4GiB - 1), allocated in a
//             //  single continuous run.
//             //
//
//             if (*ByteCount == 0) {
//
//                 *ByteCount = 0xFFFFFFFF_u32;
//             }
//
//             if (Results) {
//
//                 *ByteCount -= Remainder;
//             }
//         }
//
//     }
//
//     return Results;
// }
//
// //
// //  NOTE: Vbo/Lbn undefined if MCB is empty & return code false.
// //
//
/* BOOLEAN */
// FatLookupLastMcbEntry (
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     OUT PVBO Vbo,
//     OUT PLBO Lbo,
//     OUT PULONG Index
//     )
//
// {
//     BOOLEAN Results;
//     LONGLONG LiVbo;
//     LONGLONG LiLbo;
//     ULONG LocalIndex;
//
//     PAGED_CODE();
//
//     LiVbo = LiLbo = 0;
//     LocalIndex = 0;
//
//     Results = FsRtlLookupLastLargeMcbEntryAndIndex( Mcb,
//                                                     &LiVbo,
//                                                     &LiLbo,
//                                                     &LocalIndex );
//
//     *Vbo = ((VBO) LiVbo) << MCB_SCALE_LOG2;
//
//     if (((ULONG) LiLbo) != -1) {
//
//         *Lbo = ((LBO) LiLbo) << MCB_SCALE_LOG2;
//
//         *Lbo += (MCB_SCALE - 1);
//         *Vbo += (MCB_SCALE - 1);
//
//     } else {
//
//         *Lbo = 0;
//     }
//
//     if (Index) {
//         *Index = LocalIndex;
//     }
//
//     return Results;
// }
//
//
/* BOOLEAN */
// FatGetNextMcbEntry (
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     IN ULONG RunIndex,
//     OUT PVBO Vbo,
//     OUT PLBO Lbo,
//     OUT PULONG ByteCount
//     )
//
// {
//     BOOLEAN Results;
//     LONGLONG LiVbo;
//     LONGLONG LiLbo;
//     LONGLONG LiSectorCount;
//
//     PAGED_CODE();
//
//     LiVbo = LiLbo = 0;
//
//     Results = FsRtlGetNextLargeMcbEntry( Mcb,
//                                          RunIndex,
//                                          &LiVbo,
//                                          &LiLbo,
//                                          &LiSectorCount );
//
//     if (Results) {
//
//         *Vbo = ((VBO) LiVbo) << MCB_SCALE_LOG2;
//
//         if (((ULONG) LiLbo) != -1) {
//
//             *Lbo = ((LBO) LiLbo) << MCB_SCALE_LOG2;
//
//         } else {
//
//             *Lbo = 0;
//         }
//
//         *ByteCount = ((ULONG) LiSectorCount) << MCB_SCALE_LOG2;
//
//         if ((*ByteCount == 0) && (LiSectorCount != 0)) {
//
//             //
//             //  If 'ByteCount' overflows, then this is likely a file of
//             //  max supported size (2^32 - 1) in one contiguous run.
//             //
//
//             NT_ASSERT( RunIndex == 0 );
//
//             *ByteCount = 0xFFFFFFFF_u32;
//         }
//     }
//
//     return Results;
// }
//
//
// VOID
// FatRemoveMcbEntry (
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     IN VBO Vbo,
//     IN ULONG SectorCount
//     )
// {
//     PAGED_CODE();
//
//     if ((SectorCount) && (SectorCount != 0xFFFFFFFF_u32 as i32)) {
//
//         SectorCount--;
//         SectorCount >>= MCB_SCALE_LOG2;
//         SectorCount++;
//     }
//
//     Vbo >>= MCB_SCALE_LOG2;
//
// #if DBG
//     _SEH2_TRY {
// #endif
//
//         FsRtlRemoveLargeMcbEntry( Mcb,
//                                   (LONGLONG) Vbo,
//                                   (LONGLONG) SectorCount);
//
// #if DBG
//     } _SEH2_EXCEPT(FatBugCheckExceptionFilter( _SEH2_GetExceptionInformation() )) {
//
//           NOTHING;
//     } _SEH2_END;
// #endif
//
// }
//
//
// _Function_class_(IRP_MJ_FILE_SYSTEM_CONTROL)
// _Function_class_(DRIVER_DISPATCH)
// NTSTATUS
// NTAPI
// FatFsdFileSystemControl (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of FileSystem control operations
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the
//         file exists
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The FSD status for the IRP
//
// --*/
//
// {
//     BOOLEAN Wait;
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//
//     DebugTrace(+1, Dbg,"FatFsdFileSystemControl\n", 0);
//
//     //
//     //  Call the common FileSystem Control routine, with blocking allowed if
//     //  synchronous.  This opeation needs to special case the mount
//     //  and verify suboperations because we know they are allowed to block.
//     //  We identify these suboperations by looking at the file object field
//     //  and seeing if its null.
//     //
//
//     if (IoGetCurrentIrpStackLocation(Irp)->FileObject == NULL) {
//
//         Wait = TRUE;
//
//     } else {
//
//         Wait = CanFsdWait( Irp );
//     }
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         PIO_STACK_LOCATION IrpSp;
//
//         IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//         //
//         //  We need to made a special check here for the InvalidateVolumes
//         //  FSCTL as that comes in with a FileSystem device object instead
//         //  of a volume device object.
//         //
//
//         if (FatDeviceIsFatFsdo( IrpSp->DeviceObject) &&
//             (IrpSp->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) &&
//             (IrpSp->MinorFunction == IRP_MN_USER_FS_REQUEST) &&
//             (IrpSp->Parameters.FileSystemControl.FsControlCode ==
//              FSCTL_INVALIDATE_VOLUMES)) {
//
//             Status = FatInvalidateVolumes( Irp );
//
//         } else {
//
//             IrpContext = FatCreateIrpContext( Irp, Wait );
//
//             Status = FatCommonFileSystemControl( IrpContext, Irp );
//         }
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  execption code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdFileSystemControl -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatCommonFileSystemControl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     PIO_STACK_LOCATION IrpSp;
//
//     PAGED_CODE();
//
//     //
//     //  Get a pointer to the current Irp stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg,"FatCommonFileSystemControl\n", 0);
//     DebugTrace( 0, Dbg,"Irp           = %p\n", Irp);
//     DebugTrace( 0, Dbg,"MinorFunction = %08lx\n", IrpSp->MinorFunction);
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
//         Status = FatUserFsCtrl( IrpContext, Irp );
//         break;
//
//     case IRP_MN_MOUNT_VOLUME:
//
//         Status = FatMountVolume( IrpContext,
//                                  IrpSp->Parameters.MountVolume.DeviceObject,
//                                  IrpSp->Parameters.MountVolume.Vpb,
//                                  IrpSp->DeviceObject );
//
//         //
//         //  Complete the request.
//         //
//         //  We do this here because FatMountVolume can be called recursively,
//         //  but the Irp is only to be completed once.
//         //
//         //  NOTE: I don't think this is true anymore (danlo 3/15/1999).  Probably
//         //  an artifact of the old doublespace attempt.
//         //
//
//         FatCompleteRequest( IrpContext, Irp, Status );
//         break;
//
//     case IRP_MN_VERIFY_VOLUME:
//
//         Status = FatVerifyVolume( IrpContext, Irp );
//         break;
//
//     default:
//
//         DebugTrace( 0, Dbg, "Invalid FS Control Minor Function %08lx\n", IrpSp->MinorFunction);
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     DebugTrace(-1, Dbg, "FatCommonFileSystemControl -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMountVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT TargetDeviceObject,
//     IN PVPB Vpb,
//     IN PDEVICE_OBJECT FsDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the mount volume operation.  It is responsible for
//     either completing of enqueuing the input Irp.
//
//     Its job is to verify that the volume denoted in the IRP is a Fat volume,
//     and create the VCB and root DCB structures.  The algorithm it uses is
//     essentially as follows:
//
//     1. Create a new Vcb Structure, and initialize it enough to do cached
//        volume file I/O.
//
//     2. Read the disk and check if it is a Fat volume.
//
//     3. If it is not a Fat volume then free the cached volume file, delete
//        the VCB, and complete the IRP with STATUS_UNRECOGNIZED_VOLUME
//
//     4. Check if the volume was previously mounted and if it was then do a
//        remount operation.  This involves reinitializing the cached volume
//        file, checking the dirty bit, resetting up the allocation support,
//        deleting the VCB, hooking in the old VCB, and completing the IRP.
//
//     5. Otherwise create a root DCB, create Fsp threads as necessary, and
//        complete the IRP.
//
// Arguments:
//
//     TargetDeviceObject - This is where we send all of our requests.
//
//     Vpb - This gives us additional information needed to complete the mount.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//
//     PBCB BootBcb;
//     PPACKED_BOOT_SECTOR BootSector = NULL;
//
//     PBCB DirentBcb;
//     PDIRENT Dirent;
//     ULONG ByteOffset;
//
//     BOOLEAN MountNewVolume = FALSE;
//     BOOLEAN WeClearedVerifyRequiredBit = FALSE;
//     BOOLEAN DoARemount = FALSE;
//
//     PVCB OldVcb = NULL;
//     PVPB OldVpb = NULL;
//
//     PDEVICE_OBJECT RealDevice = NULL;
//     PVOLUME_DEVICE_OBJECT VolDo = NULL;
//     PVCB Vcb = NULL;
//     PFILE_OBJECT RootDirectoryFile = NULL;
//
//     PLIST_ENTRY Links;
//
//     IO_STATUS_BLOCK Iosb = {0};
//     ULONG ChangeCount = 0;
//
//     DISK_GEOMETRY Geometry;
//
//     PARTITION_INFORMATION_EX PartitionInformation;
//     NTSTATUS StatusPartInfo;
//
// #if (NTDDI_VERSION > NTDDI_WIN8)
//     GUID VolumeGuid = {0};
// #endif
//
//
//     PAGED_CODE();
//
//
//     DebugTrace(+1, Dbg, "FatMountVolume\n", 0);
//     DebugTrace( 0, Dbg, "TargetDeviceObject = %p\n", TargetDeviceObject);
//     DebugTrace( 0, Dbg, "Vpb                = %p\n", Vpb);
//
//     NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//     NT_ASSERT( FatDeviceIsFatFsdo( FsDeviceObject));
//
//     //
//     // Only send down IOCTL_DISK_CHECK_VERIFY if it is removable media.
//     //
//
//     if (FlagOn(TargetDeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         //
//         //  Verify that there is a disk here and pick up the change count.
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                       IOCTL_DISK_CHECK_VERIFY,
//                                       TargetDeviceObject,
//                                       NULL,
//                                       0,
//                                       &ChangeCount,
//                                       sizeof(ULONG),
//                                       FALSE,
//                                       TRUE,
//                                       &Iosb );
//
//         if (!NT_SUCCESS( Status )) {
//
//             //
//             //  If we will allow a raw mount then avoid sending the popup.
//             //
//             //  Only send this on "true" disk devices to handle the accidental
//             //  legacy of FAT. No other FS will throw a harderror on empty
//             //  drives.
//             //
//             //  Cmd should really handle this per 9x.
//             //
//
//             if (!FlagOn( IrpSp->Flags, SL_ALLOW_RAW_MOUNT ) &&
//                 Vpb->RealDevice->DeviceType == FILE_DEVICE_DISK) {
//
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//
//             return Status;
//         }
//
//     }
//
//     if (Iosb.Information != sizeof(ULONG)) {
//
//         //
//         //  Be safe about the count in case the driver didn't fill it in
//         //
//
//         ChangeCount = 0;
//     }
//
//     //
//     //  If this is a CD class device,  then check to see if there is a
//     //  'data track' or not.  This is to avoid issuing paging reads which will
//     //  fail later in the mount process (e.g. CD-DA or blank CD media)
//     //
//
//     if ((TargetDeviceObject->DeviceType == FILE_DEVICE_CD_ROM) &&
//         !FatScanForDataTrack( IrpContext, TargetDeviceObject))  {
//
//         return STATUS_UNRECOGNIZED_VOLUME;
//     }
//
//     //
//     //  Ping the volume with a partition query and pick up the partition
//     //  type.  We'll check this later to avoid some scurrilous volumes.
//     //
//
//     StatusPartInfo = FatPerformDevIoCtrl( IrpContext,
//                                           IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                           TargetDeviceObject,
//                                           NULL,
//                                           0,
//                                           &PartitionInformation,
//                                           sizeof(PARTITION_INFORMATION_EX),
//                                           FALSE,
//                                           TRUE,
//                                           &Iosb );
//
//     //
//     //  Make sure we can wait.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     //
//     //  Do a quick check to see if there any Vcb's which can be removed.
//     //
//
//     FatScanForDismountedVcb( IrpContext );
//
//
//     //
//     //  Initialize the Bcbs and our final state so that the termination
//     //  handlers will know what to free or unpin
//     //
//
//     BootBcb = NULL;
//     DirentBcb = NULL;
//
//     Vcb = NULL;
//     VolDo = NULL;
//     MountNewVolume = FALSE;
//
//     _SEH2_TRY {
//
//         //
//         //  Synchronize with FatCheckForDismount(), which modifies the vpb.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//         (VOID)FatAcquireExclusiveGlobal( IrpContext );
//
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//         //
//         //  Create a new volume device object.  This will have the Vcb
//         //  hanging off of its end, and set its alignment requirement
//         //  from the device we talk to.
//         //
//
//         if (!NT_SUCCESS(Status = IoCreateDevice( FatData.DriverObject,
//                                                  sizeof(VOLUME_DEVICE_OBJECT) - sizeof(DEVICE_OBJECT),
//                                                  NULL,
//                                                  FILE_DEVICE_DISK_FILE_SYSTEM,
//                                                  0,
//                                                  FALSE,
//                                                  (PDEVICE_OBJECT *)&VolDo))) {
//
//             try_return( Status );
//         }
//
//
//         //
//         //  Our alignment requirement is the larger of the processor alignment requirement
//         //  already in the volume device object and that in the TargetDeviceObject
//         //
//
//         if (TargetDeviceObject->AlignmentRequirement > VolDo->DeviceObject.AlignmentRequirement) {
//
//             VolDo->DeviceObject.AlignmentRequirement = TargetDeviceObject->AlignmentRequirement;
//         }
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
//         //  We must initialize the stack size in our device object before
//         //  the following reads, because the I/O system has not done it yet.
//         //  This must be done before we clear the device initializing flag
//         //  otherwise a filter could attach and copy the wrong stack size into
//         //  it's device object.
//         //
//
//         VolDo->DeviceObject.StackSize = (CCHAR)(TargetDeviceObject->StackSize + 1);
//
//         //
//         //  We must also set the sector size correctly in our device object
//         //  before clearing the device initializing flag.
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                       IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                       TargetDeviceObject,
//                                       NULL,
//                                       0,
//                                       &Geometry,
//                                       sizeof( DISK_GEOMETRY ),
//                                       FALSE,
//                                       TRUE,
//                                       NULL );
//
//         if (!NT_SUCCESS( Status )) {
//
//             try_return( Status );
//         }
//
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28175, "this is a filesystem driver, touching SectorSize is fine" )
// #endif
//         VolDo->DeviceObject.SectorSize = (USHORT)Geometry.BytesPerSector;
//
//         //
//         //  Indicate that this device object is now completely initialized
//         //
//
//         ClearFlag(VolDo->DeviceObject.Flags, DO_DEVICE_INITIALIZING);
//
//         //
//         //  Now Before we can initialize the Vcb we need to set up the device
//         //  object field in the Vpb to point to our new volume device object.
//         //  This is needed when we create the virtual volume file's file object
//         //  in initialize vcb.
//         //
//
//         Vpb->DeviceObject = (PDEVICE_OBJECT)VolDo;
//
//         //
//         //  If the real device needs verification, temporarily clear the
//         //  field.
//         //
//
//         RealDevice = Vpb->RealDevice;
//
//         if ( FlagOn(RealDevice->Flags, DO_VERIFY_VOLUME) ) {
//
//             ClearFlag(RealDevice->Flags, DO_VERIFY_VOLUME);
//
//             WeClearedVerifyRequiredBit = TRUE;
//         }
//
//         //
//         //  Initialize the new vcb
//         //
//
//         FatInitializeVcb( IrpContext,
//                           &VolDo->Vcb,
//                           TargetDeviceObject,
//                           Vpb,
//                           FsDeviceObject);
//         //
//         //  Get a reference to the Vcb hanging off the end of the device object
//         //
//
//         Vcb = &VolDo->Vcb;
//
//         //
//         //  Read in the boot sector, and have the read be the minumum size
//         //  needed.  We know we can wait.
//         //
//
//         //
//         //  We need to commute errors on CD so that CDFS will get its crack.  Audio
//         //  and even data media may not be universally readable on sector zero.
//         //
//
//         _SEH2_TRY {
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                0,                          // Starting Byte
//                                sizeof(PACKED_BOOT_SECTOR),
//                                &BootBcb,
//                                (PVOID *)&BootSector );
//
//         } _SEH2_EXCEPT( Vpb->RealDevice->DeviceType == FILE_DEVICE_CD_ROM ?
//                   EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//               NOTHING;
//         } _SEH2_END;
//
//
//         //
//         //  Call a routine to check the boot sector to see if it is fat
//         //
//
//         if (BootBcb == NULL || !FatIsBootSectorFat( BootSector)) {
//
//             DebugTrace(0, Dbg, "Not a Fat Volume\n", 0);
//
//             //
//             //  Complete the request and return to our caller
//             //
//
//             try_return( Status = STATUS_UNRECOGNIZED_VOLUME );
//         }
//
// #if (NTDDI_VERSION > NTDDI_WIN8)
//         //
//         //  Initialize the volume guid.
//         //
//
//         if (NT_SUCCESS( IoVolumeDeviceToGuid( Vcb->TargetDeviceObject, &VolumeGuid ))) {
//
//
//             //
//             // Stash a copy away in the VCB.
//             //
//
//             RtlCopyMemory( &Vcb->VolumeGuid, &VolumeGuid, sizeof(GUID));
//
//         }
//
//
//         //
//         // Stash away a copy of the volume GUID path in our VCB.
//         //
//
//         if (Vcb->VolumeGuidPath.Buffer) {
//             ExFreePool( Vcb->VolumeGuidPath.Buffer );
//             Vcb->VolumeGuidPath.Buffer = NULL;
//             Vcb->VolumeGuidPath.Length = Vcb->VolumeGuidPath.MaximumLength = 0;
//         }
//
//         IoVolumeDeviceToGuidPath( Vcb->TargetDeviceObject, &Vcb->VolumeGuidPath );
// #endif
//
//         //
//         //  Unpack the BPB.  We used to do some sanity checking of the FATs at
//         //  this point, but authoring errors on third-party devices prevent
//         //  us from continuing to safeguard ourselves.  We can only hope the
//         //  boot sector check is good enough.
//         //
//         //  (read: digital cameras)
//         //
//         //  Win9x does the same.
//         //
//
//         FatUnpackBios( &Vcb->Bpb, &BootSector->PackedBpb );
//
//         //
//         //  Check if we have an OS/2 Boot Manager partition and treat it as an
//         //  unknown file system.  We'll check the partition type in from the
//         //  partition table and we ensure that it has less than 0x80 sectors,
//         //  which is just a heuristic that will capture all real OS/2 BM partitions
//         //  and avoid the chance we'll discover partitions which erroneously
//         //  (but to this point, harmlessly) put down the OS/2 BM type.
//         //
//         //  Note that this is only conceivable on good old MBR media.
//         //
//         //  The OS/2 Boot Manager boot format mimics a FAT16 partition in sector
//         //  zero but does is not a real FAT16 file system.  For example, the boot
//         //  sector indicates it has 2 FATs but only really has one, with the boot
//         //  manager code overlaying the second FAT.  If we then set clean bits in
//         //  FAT[0] we'll corrupt that code.
//         //
//
//         if (NT_SUCCESS( StatusPartInfo ) &&
//             (PartitionInformation.PartitionStyle == PARTITION_STYLE_MBR &&
//              PartitionInformation.Mbr.PartitionType == PARTITION_OS2BOOTMGR) &&
//             (Vcb->Bpb.Sectors != 0 &&
//              Vcb->Bpb.Sectors < 0x80)) {
//
//             DebugTrace( 0, Dbg, "OS/2 Boot Manager volume detected, volume not mounted. \n", 0 );
//
//             //
//             //  Complete the request and return to our caller
//             //
//
//             try_return( Status = STATUS_UNRECOGNIZED_VOLUME );
//         }
//
//         //
//         //  Verify that the sector size recorded in the Bpb matches what the
//         //  device currently reports it's sector size to be.
//         //
//
//         if ( !NT_SUCCESS( Status) ||
//              (Geometry.BytesPerSector != Vcb->Bpb.BytesPerSector))  {
//
//             try_return( Status = STATUS_UNRECOGNIZED_VOLUME );
//         }
//
//         //
//         //  This is a fat volume, so extract the bpb, serial number.  The
//         //  label we'll get later after we've created the root dcb.
//         //
//         //  Note that the way data caching is done, we set neither the
//         //  direct I/O or Buffered I/O bit in the device object flags.
//         //
//
//         if (Vcb->Bpb.Sectors != 0) { Vcb->Bpb.LargeSectors = 0; }
//
//         if (IsBpbFat32(&BootSector->PackedBpb)) {
//
//             CopyUchar4( &Vpb->SerialNumber, ((PPACKED_BOOT_SECTOR_EX)BootSector)->Id );
//
//         } else  {
//
//             CopyUchar4( &Vpb->SerialNumber, BootSector->Id );
//
//             //
//             //  Allocate space for the stashed boot sector chunk.  This only has meaning on
//             //  FAT12/16 volumes since this only is kept for the FSCTL_QUERY_FAT_BPB and it and
//             //  its users are a bit wierd, thinking that a BPB exists wholly in the first 0x24
//             //  bytes.
//             //
//
//             Vcb->First0x24BytesOfBootSector =
//                 FsRtlAllocatePoolWithTag( PagedPool,
//                                           0x24,
//                                           TAG_STASHED_BPB );
//
//             //
//             //  Stash a copy of the first 0x24 bytes
//             //
//
//             RtlCopyMemory( Vcb->First0x24BytesOfBootSector,
//                            BootSector,
//                            0x24 );
//         }
//
//         //
//         //  Now unpin the boot sector, so when we set up allocation eveything
//         //  works.
//         //
//
//         FatUnpinBcb( IrpContext, BootBcb );
//
//         //
//         //  Compute a number of fields for Vcb.AllocationSupport
//         //
//
//         FatSetupAllocationSupport( IrpContext, Vcb );
//
//         //
//         //  Sanity check the FsInfo information for FAT32 volumes.  Silently deal
//         //  with messed up information by effectively disabling FsInfo updates.
//         //
//
//         if (FatIsFat32( Vcb )) {
//
//             if (Vcb->Bpb.FsInfoSector >= Vcb->Bpb.ReservedSectors) {
//
//                 Vcb->Bpb.FsInfoSector = 0;
//             }
//         }
//
//
//         //
//         //  Create a root Dcb so we can read in the volume label.  If this is FAT32, we can
//         //  discover corruption in the FAT chain.
//         //
//         //  NOTE: this exception handler presumes that this is the only spot where we can
//         //  discover corruption in the mount process.  If this ever changes, this handler
//         //  MUST be expanded.  The reason we have this guy here is because we have to rip
//         //  the structures down now (in the finally below) and can't wait for the outer
//         //  exception handling to do it for us, at which point everything will have vanished.
//         //
//
//         _SEH2_TRY {
//
//             FatCreateRootDcb( IrpContext, Vcb );
//
//         } _SEH2_EXCEPT (_SEH2_GetExceptionCode() == STATUS_FILE_CORRUPT_ERROR ? EXCEPTION_EXECUTE_HANDLER :
//                                                                     EXCEPTION_CONTINUE_SEARCH) {
//
//             //
//             //  The volume needs to be dirtied, do it now.  Note that at this point we have built
//             //  enough of the Vcb to pull this off.
//             //
//
// 	    FatCheckDirtyBit( IrpContext,
// 		              Vcb );
//
//             //
//             // Set the dirty bit if it is not set already
//             //
//
//             if ( !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNT_IN_PROGRESS );
//                 FatMarkVolume( IrpContext, Vcb, VolumeDirty );
//                 ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNT_IN_PROGRESS );
//             }
//
//             //
//             //  Now keep bailing out ...
//             //
//
//             FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//         } _SEH2_END;
//
//         FatLocateVolumeLabel( IrpContext,
//                               Vcb,
//                               &Dirent,
//                               &DirentBcb,
//                               (PVBO)&ByteOffset );
//
//         if (Dirent != NULL) {
//
//             UCHAR OemBuffer[11];
//             OEM_STRING OemString;
//             UNICODE_STRING UnicodeString;
//
//             OemString.Buffer = (PCHAR)&OemBuffer[0];
//             OemString.MaximumLength = 11;
//
//             RtlCopyMemory( OemString.Buffer, Dirent->FileName, 11 );
//
//             //
//             //  Translate the first character from 0x5 to 0xe5.
//             //
//
//             if (OemString.Buffer[0] == FAT_DIRENT_REALLY_0E5) {
//
//                 OemString.Buffer[0] = 0xe5;
//             }
//
//             //
//             //  Compute the length of the volume name
//             //
//
//             for ( OemString.Length = 11;
//                   OemString.Length > 0;
//                   OemString.Length -= 1) {
//
//                 if ( (OemString.Buffer[OemString.Length-1] != 0x00) &&
//                      (OemString.Buffer[OemString.Length-1] != 0x20) ) { break; }
//             }
//
//             UnicodeString.MaximumLength = MAXIMUM_VOLUME_LABEL_LENGTH;
//             UnicodeString.Buffer = &Vcb->Vpb->VolumeLabel[0];
//
//             Status = RtlOemStringToCountedUnicodeString( &UnicodeString,
//                                                          &OemString,
//                                                          FALSE );
//
//             if ( !NT_SUCCESS( Status ) ) {
//
//                 try_return( Status );
//             }
//
//             Vpb->VolumeLabelLength = UnicodeString.Length;
//
//         } else {
//
//             Vpb->VolumeLabelLength = 0;
//         }
//
//         //
//         //  Use the change count we noted initially *before* doing any work.
//         //  If something came along in the midst of this operation, we'll
//         //  verify and discover the problem.
//         //
//
//         Vcb->ChangeCount = ChangeCount;
//
//         //
//         //  Now scan the list of previously mounted volumes and compare
//         //  serial numbers and volume labels off not currently mounted
//         //  volumes to see if we have a match.
//         //
//
//         for (Links = FatData.VcbQueue.Flink;
//              Links != &FatData.VcbQueue;
//              Links = Links->Flink) {
//
//             OldVcb = CONTAINING_RECORD( Links, VCB, VcbLinks );
//             OldVpb = OldVcb->Vpb;
//
//             //
//             //  Skip over ourselves since we're already in the VcbQueue
//             //
//
//             if (OldVpb == Vpb) { continue; }
//
//             //
//             //  Check for a match:
//             //
//             //  Serial Number, VolumeLabel and Bpb must all be the same.
//             //  Also the volume must have failed a verify before (ie.
//             //  VolumeNotMounted), and it must be in the same physical
//             //  drive than it was mounted in before.
//             //
//
//             if ( (OldVpb->SerialNumber == Vpb->SerialNumber) &&
//                  (OldVcb->VcbCondition == VcbNotMounted) &&
//                  (OldVpb->RealDevice == RealDevice) &&
//                  (OldVpb->VolumeLabelLength == Vpb->VolumeLabelLength) &&
//                  (RtlEqualMemory(&OldVpb->VolumeLabel[0],
//                                  &Vpb->VolumeLabel[0],
//                                  Vpb->VolumeLabelLength)) &&
//                  (RtlEqualMemory(&OldVcb->Bpb,
//                                  &Vcb->Bpb,
//                                  IsBpbFat32(&Vcb->Bpb) ?
//                                      sizeof(BIOS_PARAMETER_BLOCK) :
//                                      FIELD_OFFSET(BIOS_PARAMETER_BLOCK,
//                                                   LargeSectorsPerFat) ))) {
//
//                 DoARemount = TRUE;
//
//                 break;
//             }
//         }
//
//         if ( DoARemount ) {
//
//             PVPB *IrpVpb;
//
//             DebugTrace(0, Dbg, "Doing a remount\n", 0);
//             DebugTrace(0, Dbg, "Vcb = %p\n", Vcb);
//             DebugTrace(0, Dbg, "Vpb = %p\n", Vpb);
//             DebugTrace(0, Dbg, "OldVcb = %p\n", OldVcb);
//             DebugTrace(0, Dbg, "OldVpb = %p\n", OldVpb);
//
//             //
//             //  Swap target device objects between the VCBs. That way
//             //  the old VCB will start using the new target device object,
//             //  and the new VCB will be torn down and deference the old
//             //  target device object.
//             //
//
//             Vcb->TargetDeviceObject = OldVcb->TargetDeviceObject;
//             OldVcb->TargetDeviceObject = TargetDeviceObject;
//
//             //
//             //  This is a remount, so link the old vpb in place
//             //  of the new vpb.
//             //
//
//             NT_ASSERT( !FlagOn( OldVcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED ) );
//
//             FatSetVcbCondition( OldVcb, VcbGood);
//             OldVpb->RealDevice = Vpb->RealDevice;
//             ClearFlag( OldVcb->VcbState, VCB_STATE_VPB_NOT_ON_DEVICE);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28175, "touching Vpb is ok for a filesystem" )
// #endif
//             OldVpb->RealDevice->Vpb = OldVpb;
//
//             //
//             //  Use the new changecount.
//             //
//
//             OldVcb->ChangeCount = Vcb->ChangeCount;
//
//             //
//             //  If the new VPB is the VPB referenced in the original Irp, set
//             //  that reference back to the old VPB.
//             //
//
//             IrpVpb = &IoGetCurrentIrpStackLocation(IrpContext->OriginatingIrp)->Parameters.MountVolume.Vpb;
//
//             if (*IrpVpb == Vpb) {
//
//                 *IrpVpb = OldVpb;
//             }
//
//             //
//             //  We do not want to touch this VPB again.  It will get cleaned up when
//             //  the new VCB is cleaned up.
//             //
//
//             NT_ASSERT( Vcb->Vpb == Vpb );
//
//             Vpb = NULL;
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED );
//             FatSetVcbCondition( Vcb, VcbBad );
//
//             //
//             //  Reinitialize the volume file cache and allocation support.
//             //
//
//             {
//                 CC_FILE_SIZES FileSizes;
//
//                 FileSizes.AllocationSize.QuadPart =
//                 FileSizes.FileSize.QuadPart = ( 0x40000 + 0x1000 );
//                 FileSizes.ValidDataLength = FatMaxLarge;
//
//                 DebugTrace(0, Dbg, "Truncate and reinitialize the volume file\n", 0);
//
//                 FatInitializeCacheMap( OldVcb->VirtualVolumeFile,
//                                        &FileSizes,
//                                        TRUE,
//                                        &FatData.CacheManagerNoOpCallbacks,
//                                        Vcb );
//
//                 //
//                 //  Redo the allocation support
//                 //
//
//                 FatSetupAllocationSupport( IrpContext, OldVcb );
//
//                 //
//                 //  Get the state of the dirty bit.
//                 //
//
//                 FatCheckDirtyBit( IrpContext, OldVcb );
//
//                 //
//                 //  Check for write protected media.
//                 //
//
//                 if (FatIsMediaWriteProtected(IrpContext, TargetDeviceObject)) {
//
//                     SetFlag( OldVcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//
//                 } else {
//
//                     ClearFlag( OldVcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//                 }
//             }
//
//             //
//             //  Complete the request and return to our caller
//             //
//
//             try_return( Status = STATUS_SUCCESS );
//         }
//
//         DebugTrace(0, Dbg, "Mount a new volume\n", 0);
//
//         //
//         //  This is a new mount
//         //
//         //  Create a blank ea data file fcb, just not for Fat32.
//         //
//
//         if (!FatIsFat32(Vcb)) {
//
//             DIRENT TempDirent;
//             PFCB EaFcb;
//
//             RtlZeroMemory( &TempDirent, sizeof(DIRENT) );
//             RtlCopyMemory( &TempDirent.FileName[0], "EA DATA  SF", 11 );
//
//             EaFcb = FatCreateFcb( IrpContext,
//                                   Vcb,
//                                   Vcb->RootDcb,
//                                   0,
//                                   0,
//                                   &TempDirent,
//                                   NULL,
//                                   NULL,
//                                   FALSE,
//                                   TRUE );
//
//             //
//             //  Deny anybody who trys to open the file.
//             //
//
//             SetFlag( EaFcb->FcbState, FCB_STATE_SYSTEM_FILE );
//
//             Vcb->EaFcb = EaFcb;
//         }
//
//         //
//         //  Get the state of the dirty bit.
//         //
//
//         FatCheckDirtyBit( IrpContext, Vcb );
//
//
//         //
//         //  Check for write protected media.
//         //
//
//         if (FatIsMediaWriteProtected(IrpContext, TargetDeviceObject)) {
//
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//
//         } else {
//
//             ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//         }
//
//
//         //
//         //  Lock volume in drive if we just mounted the boot drive.
//         //
//
//         if (FlagOn(RealDevice->Flags, DO_SYSTEM_BOOT_PARTITION)) {
//
//             SetFlag(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE);
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA)) {
//
//                 FatToggleMediaEjectDisable( IrpContext, Vcb, TRUE );
//             }
//         }
//
//
//         //
//         //  Indicate to our termination handler that we have mounted
//         //  a new volume.
//         //
//
//         MountNewVolume = TRUE;
//
//         //
//         //  Complete the request
//         //
//
//         Status = STATUS_SUCCESS;
//
//         //
//         //  Ref the root dir stream object so we can send mount notification.
//         //
//
//         RootDirectoryFile = Vcb->RootDcb->Specific.Dcb.DirectoryFile;
//         ObReferenceObject( RootDirectoryFile );
//
//         //
//         //  Remove the extra reference to this target DO made on behalf of us
//         //  by the IO system.  In the remount case, we permit regular Vcb
//         //  deletion to do this work.
//         //
//
//         ObDereferenceObject( TargetDeviceObject );
//
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatMountVolume );
//
//         FatUnpinBcb( IrpContext, BootBcb );
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         //
//         //  Check if a volume was mounted.  If not then we need to
//         //  mark the Vpb not mounted again.
//         //
//
//         if ( !MountNewVolume ) {
//
//             if ( Vcb != NULL ) {
//
//                 //
//                 //  A VCB was created and initialized.  We need to try to tear it down.
//                 //
//
//                 FatCheckForDismount( IrpContext,
//                                      Vcb,
//                                      TRUE );
//
//                 IrpContext->Vcb = NULL;
//
//             } else if (VolDo != NULL) {
//
//                 //
//                 //  The VCB was never initialized, so we need to delete the
//                 //  device right here.
//                 //
//
//                 IoDeleteDevice( &VolDo->DeviceObject );
//             }
//
//             //
//             //  See if a remount failed.
//             //
//
//             if (DoARemount && _SEH2_AbnormalTermination()) {
//
//                 //
//                 //  The remount failed. Try to tear down the old VCB as well.
//                 //
//
//                 FatCheckForDismount( IrpContext,
//                                      OldVcb,
//                                      TRUE );
//             }
//         }
//
//         if ( WeClearedVerifyRequiredBit == TRUE ) {
//
//             SetFlag(RealDevice->Flags, DO_VERIFY_VOLUME);
//         }
//
//         FatReleaseGlobal( IrpContext );
//
//         DebugTrace(-1, Dbg, "FatMountVolume -> %08lx\n", Status);
//     } _SEH2_END;
//
//     //
//     //  Now send mount notification. Note that since this is outside of any
//     //  synchronization since the synchronous delivery of this may go to
//     //  folks that provoke re-entrance to the FS.
//     //
//
//     if (RootDirectoryFile != NULL) {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         if (FatDiskAccountingEnabled) {
//
//             CcSetAdditionalCacheAttributesEx( RootDirectoryFile, CC_ENABLE_DISK_IO_ACCOUNTING );
//         }
// #endif
//
//         FsRtlNotifyVolumeEvent( RootDirectoryFile, FSRTL_VOLUME_MOUNT );
//         ObDereferenceObject( RootDirectoryFile );
//     }
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatVerifyVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the verify volume operation by checking the volume
//     label and serial number physically on the media with the the Vcb
//     currently claiming to have the volume mounted. It is responsible for
//     either completing or enqueuing the input Irp.
//
//     Regardless of whether the verify operation succeeds, the following
//     operations are performed:
//
//         - Set Vcb->VirtualEaFile back to its initial state.
//         - Purge all cached data (flushing first if verify succeeds)
//         - Mark all Fcbs as needing verification
//
//     If the volumes verifies correctly we also must:
//
//         - Check the volume dirty bit.
//         - Reinitialize the allocation support
//         - Flush any dirty data
//
//     If the volume verify fails, it may never be mounted again.  If it is
//     mounted again, it will happen as a remount operation.  In preparation
//     for that, and to leave the volume in a state that can be "lazy deleted"
//     the following operations are performed:
//
//         - Set the Vcb condition to VcbNotMounted
//         - Uninitialize the volume file cachemap
//         - Tear down the allocation support
//
//     In the case of an abnormal termination we haven't determined the state
//     of the volume, so we set the Device Object as needing verification again.
//
// Arguments:
//
//     Irp - Supplies the Irp to process
//
// Return Value:
//
//     NTSTATUS - If the verify operation completes, it will return either
//         STATUS_SUCCESS or STATUS_WRONG_VOLUME, exactly.  If an IO or
//         other error is encountered, that status will be returned.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PDIRENT RootDirectory = NULL;
//     PPACKED_BOOT_SECTOR BootSector = NULL;
//
//     BIOS_PARAMETER_BLOCK Bpb;
//
//     PVOLUME_DEVICE_OBJECT VolDo;
//     PVCB Vcb;
//     PVPB Vpb;
//
//     ULONG SectorSize;
//     BOOLEAN ClearVerify = FALSE;
//     BOOLEAN ReleaseEntireVolume = FALSE;
//     BOOLEAN VerifyAlreadyDone = FALSE;
//
//     DISK_GEOMETRY DiskGeometry;
//
//     LBO RootDirectoryLbo;
//     ULONG RootDirectorySize;
//     BOOLEAN LabelFound;
//
//     ULONG ChangeCount = 0;
//     IO_STATUS_BLOCK Iosb = {0};
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Irp stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatVerifyVolume\n", 0);
//     DebugTrace( 0, Dbg, "DeviceObject = %p\n", IrpSp->Parameters.VerifyVolume.DeviceObject);
//     DebugTrace( 0, Dbg, "Vpb          = %p\n", IrpSp->Parameters.VerifyVolume.Vpb);
//
//     //
//     //  Save some references to make our life a little easier.  Note the Vcb for the purposes
//     //  of exception handling.
//     //
//
//     VolDo = (PVOLUME_DEVICE_OBJECT)IrpSp->Parameters.VerifyVolume.DeviceObject;
//
//     Vpb                   = IrpSp->Parameters.VerifyVolume.Vpb;
//     IrpContext->Vcb = Vcb = &VolDo->Vcb;
//
//     //
//     //  If we cannot wait then enqueue the irp to the fsp and
//     //  return the status to our caller.
//     //
//
//     if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT)) {
//
//         DebugTrace(0, Dbg, "Cannot wait for verify.\n", 0);
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatVerifyVolume -> %08lx\n", Status );
//         return Status;
//     }
//
//     //
//     //  We are serialized at this point allowing only one thread to
//     //  actually perform the verify operation.  Any others will just
//     //  wait and then no-op when checking if the volume still needs
//     //  verification.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     (VOID)FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     (VOID)FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     _SEH2_TRY {
//
//         BOOLEAN AllowRawMount = BooleanFlagOn( IrpSp->Flags, SL_ALLOW_RAW_MOUNT );
//
//         //
//         //  Mark ourselves as verifying this volume so that recursive I/Os
//         //  will be able to complete.
//         //
//
//         NT_ASSERT( Vcb->VerifyThread == NULL );
//         Vcb->VerifyThread = KeGetCurrentThread();
//
//         //
//         //  Check if the real device still needs to be verified.  If it doesn't
//         //  then obviously someone beat us here and already did the work
//         //  so complete the verify irp with success.  Otherwise reenable
//         //  the real device and get to work.
//         //
//
//         if (!FlagOn(Vpb->RealDevice->Flags, DO_VERIFY_VOLUME)) {
//
//             DebugTrace(0, Dbg, "RealDevice has already been verified\n", 0);
//
//             VerifyAlreadyDone = TRUE;
//             try_return( Status = STATUS_SUCCESS );
//         }
//
//         //
//         //  Ping the volume with a partition query to make Jeff happy.
//         //
//
//         {
//             PARTITION_INFORMATION_EX PartitionInformation;
//
//             (VOID) FatPerformDevIoCtrl( IrpContext,
//                                         IOCTL_DISK_GET_PARTITION_INFO_EX,
//                                         Vcb->TargetDeviceObject,
//                                         NULL,
//                                         0,
//                                         &PartitionInformation,
//                                         sizeof(PARTITION_INFORMATION_EX),
//                                         FALSE,
//                                         TRUE,
//                                         &Iosb );
//         }
//
//         //
//         // Only send down IOCTL_DISK_CHECK_VERIFY if it is removable media.
//         //
//
//         if (FlagOn(Vcb->TargetDeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//             //
//             //  Verify that there is a disk here and pick up the change count.
//             //
//
//             Status = FatPerformDevIoCtrl( IrpContext,
//                                           IOCTL_DISK_CHECK_VERIFY,
//                                           Vcb->TargetDeviceObject,
//                                           NULL,
//                                           0,
//                                           &ChangeCount,
//                                           sizeof(ULONG),
//                                           FALSE,
//                                           TRUE,
//                                           &Iosb );
//
//             if (!NT_SUCCESS( Status )) {
//
//                 //
//                 //  If we will allow a raw mount then return WRONG_VOLUME to
//                 //  allow the volume to be mounted by raw.
//                 //
//
//                 if (AllowRawMount) {
//
//                     try_return( Status = STATUS_WRONG_VOLUME );
//                 }
//
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//
//         }
//
//         if (Iosb.Information != sizeof(ULONG)) {
//
//             //
//             //  Be safe about the count in case the driver didn't fill it in
//             //
//
//             ChangeCount = 0;
//         }
//
//         //
//         //  Whatever happens we will have verified this volume at this change
//         //  count, so record that fact.
//         //
//
//         Vcb->ChangeCount = ChangeCount;
//
//         //
//         //  If this is a CD class device,  then check to see if there is a
//         //  'data track' or not.  This is to avoid issuing paging reads which will
//         //  fail later in the mount process (e.g. CD-DA or blank CD media)
//         //
//
//         if ((Vcb->TargetDeviceObject->DeviceType == FILE_DEVICE_CD_ROM) &&
//             !FatScanForDataTrack( IrpContext, Vcb->TargetDeviceObject))  {
//
//             try_return( Status = STATUS_WRONG_VOLUME);
//         }
//
//         //
//         //  Some devices can change sector sizes on the fly.  Obviously, it
//         //  isn't the same volume if that happens.
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                       IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                       Vcb->TargetDeviceObject,
//                                       NULL,
//                                       0,
//                                       &DiskGeometry,
//                                       sizeof( DISK_GEOMETRY ),
//                                       FALSE,
//                                       TRUE,
//                                       NULL );
//
//         if (!NT_SUCCESS( Status )) {
//
//             //
//             //  If we will allow a raw mount then return WRONG_VOLUME to
//             //  allow the volume to be mounted by raw.
//             //
//
//             if (AllowRawMount) {
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//
//             FatNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//
//         //
//         //  Read in the boot sector
//         //
//
//         SectorSize = (ULONG)Vcb->Bpb.BytesPerSector;
//
//         if (SectorSize != DiskGeometry.BytesPerSector) {
//
//             try_return( Status = STATUS_WRONG_VOLUME );
//         }
//
//         BootSector = FsRtlAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                               (ULONG) ROUND_TO_PAGES( SectorSize ),
//                                               TAG_VERIFY_BOOTSECTOR);
//
//         //
//         //  If this verify is on behalf of a DASD open, allow a RAW mount.
//         //
//
//         if (!FatPerformVerifyDiskRead( IrpContext,
//                                        Vcb,
//                                        BootSector,
//                                        0,
//                                        SectorSize,
//                                        AllowRawMount )) {
//
//             try_return( Status = STATUS_WRONG_VOLUME );
//         }
//
//         //
//         //  Call a routine to check the boot sector to see if it is fat.
//         //  If it is not fat then mark the vcb as not mounted tell our
//         //  caller its the wrong volume
//         //
//
//         if (!FatIsBootSectorFat( BootSector )) {
//
//             DebugTrace(0, Dbg, "Not a Fat Volume\n", 0);
//
//             try_return( Status = STATUS_WRONG_VOLUME );
//         }
//
//         //
//         //  This is a fat volume, so extract serial number and see if it is
//         //  ours.
//         //
//
//         {
//             ULONG SerialNumber;
//
//             if (IsBpbFat32(&BootSector->PackedBpb)) {
//                 CopyUchar4( &SerialNumber, ((PPACKED_BOOT_SECTOR_EX)BootSector)->Id );
//             } else {
//                 CopyUchar4( &SerialNumber, BootSector->Id );
//             }
//
//             if (SerialNumber != Vpb->SerialNumber) {
//
//                 DebugTrace(0, Dbg, "Not our serial number\n", 0);
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//         }
//
//         //
//         //  Make sure the Bpbs are not different.  We have to zero out our
//         //  stack version of the Bpb since unpacking leaves holes.
//         //
//
//         RtlZeroMemory( &Bpb, sizeof(BIOS_PARAMETER_BLOCK) );
//
//         FatUnpackBios( &Bpb, &BootSector->PackedBpb );
//         if (Bpb.Sectors != 0) { Bpb.LargeSectors = 0; }
//
//         if ( !RtlEqualMemory( &Bpb,
//                               &Vcb->Bpb,
//                               IsBpbFat32(&Bpb) ?
//                                     sizeof(BIOS_PARAMETER_BLOCK) :
//                                     FIELD_OFFSET(BIOS_PARAMETER_BLOCK,
//                                                  LargeSectorsPerFat) )) {
//
//             DebugTrace(0, Dbg, "Bpb is different\n", 0);
//
//             try_return( Status = STATUS_WRONG_VOLUME );
//         }
//
//         //
//         //  Check the volume label.  We do this by trying to locate the
//         //  volume label, making two strings one for the saved volume label
//         //  and the other for the new volume label and then we compare the
//         //  two labels.
//         //
//
//         if (FatRootDirectorySize(&Bpb) > 0) {
//
//             RootDirectorySize = FatRootDirectorySize(&Bpb);
//
//         } else {
//
//             RootDirectorySize = FatBytesPerCluster(&Bpb);
//         }
//
//         RootDirectory = FsRtlAllocatePoolWithTag( NonPagedPoolNxCacheAligned,
//                                                   (ULONG) ROUND_TO_PAGES( RootDirectorySize ),
//                                                   TAG_VERIFY_ROOTDIR);
//
//         if (!IsBpbFat32(&BootSector->PackedBpb)) {
//
//             //
//             //  The Fat12/16 case is simple -- read the root directory in and
//             //  search it.
//             //
//
//             RootDirectoryLbo = FatRootDirectoryLbo(&Bpb);
//
//             if (!FatPerformVerifyDiskRead( IrpContext,
//                                            Vcb,
//                                            RootDirectory,
//                                            RootDirectoryLbo,
//                                            RootDirectorySize,
//                                            AllowRawMount )) {
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//
//             Status = FatSearchBufferForLabel(IrpContext, Vpb,
//                                              RootDirectory, RootDirectorySize,
//                                              &LabelFound);
//
//             if (!NT_SUCCESS(Status)) {
//
//                 try_return( Status );
//             }
//
//             if (!LabelFound && Vpb->VolumeLabelLength > 0) {
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//
//         } else {
//
//             ULONG RootDirectoryCluster;
//
//             RootDirectoryCluster = Bpb.RootDirFirstCluster;
//
//             while (RootDirectoryCluster != FAT_CLUSTER_LAST) {
//
//                 RootDirectoryLbo = FatGetLboFromIndex(Vcb, RootDirectoryCluster);
//
//                 if (!FatPerformVerifyDiskRead( IrpContext,
//                                                Vcb,
//                                                RootDirectory,
//                                                RootDirectoryLbo,
//                                                RootDirectorySize,
//                                                AllowRawMount )) {
//
//                     try_return( Status = STATUS_WRONG_VOLUME );
//                 }
//
//                 Status = FatSearchBufferForLabel(IrpContext, Vpb,
//                                                  RootDirectory, RootDirectorySize,
//                                                  &LabelFound);
//
//                 if (!NT_SUCCESS(Status)) {
//
//                     try_return( Status );
//                 }
//
//                 if (LabelFound) {
//
//                     //
//                     //  Found a matching label.
//                     //
//
//                     break;
//                 }
//
//                 //
//                 //  Set ourselves up for the next loop iteration.
//                 //
//
//                 FatVerifyLookupFatEntry( IrpContext, Vcb,
//                                          RootDirectoryCluster,
//                                          &RootDirectoryCluster );
//
//                 switch (FatInterpretClusterType(Vcb, RootDirectoryCluster)) {
//
//                 case FatClusterAvailable:
//                 case FatClusterReserved:
//                 case FatClusterBad:
//
//                     //
//                     //  Bail all the way out if we have a bad root.
//                     //
//
//                     FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                     break;
//
//                 default:
//
//                     break;
//                 }
//
//             }
//
//             if (RootDirectoryCluster == FAT_CLUSTER_LAST &&
//                 Vpb->VolumeLabelLength > 0) {
//
//                 //
//                 //  Should have found a label, didn't find any.
//                 //
//
//                 try_return( Status = STATUS_WRONG_VOLUME );
//             }
//         }
//
//
//     try_exit: NOTHING;
//
//         //
//         //  Note that we have previously acquired the Vcb to serialize
//         //  the EA file stuff the marking all the Fcbs as NeedToBeVerified.
//         //
//         //  Put the Ea file back in a initial state.
//         //
//
//         FatCloseEaFile( IrpContext, Vcb, (BOOLEAN)(Status == STATUS_SUCCESS) );
//
//         //
//         //  Mark all Fcbs as needing verification, but only if we really have
//         //  to do it.
//         //
//
//         if (!VerifyAlreadyDone) {
//
//             FatAcquireExclusiveVolume( IrpContext, Vcb );
//             ReleaseEntireVolume = TRUE;
//
//             FatMarkFcbCondition( IrpContext, Vcb->RootDcb, FcbNeedsToBeVerified, TRUE );
//         }
//
//         //
//         //  If the verify didn't succeed, get the volume ready for a
//         //  remount or eventual deletion.
//         //
//
//         if (Vcb->VcbCondition == VcbNotMounted) {
//
//             //
//             //  If the volume was already in an unmounted state, just bail
//             //  and make sure we return STATUS_WRONG_VOLUME.
//             //
//
//             Status = STATUS_WRONG_VOLUME;
//
//         } else if ( Status == STATUS_WRONG_VOLUME ) {
//
//             //
//             //  Grab everything so we can safely transition the volume state without
//             //  having a thread stumble into the torn-down allocation engine.
//             //
//
//             if (!ReleaseEntireVolume) {
//                 FatAcquireExclusiveVolume( IrpContext, Vcb );
//                 ReleaseEntireVolume = TRUE;
//             }
//
//             //
//             //  Get rid of any cached data, without flushing
//             //
//
//             FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, NoFlush );
//
//             //
//             //  Uninitialize the volume file cache map.  Note that we cannot
//             //  do a "FatSyncUninit" because of deadlock problems.  However,
//             //  since this FileObject is referenced by us, and thus included
//             //  in the Vpb residual count, it is OK to do a normal CcUninit.
//             //
//
//             CcUninitializeCacheMap( Vcb->VirtualVolumeFile,
//                                     &FatLargeZero,
//                                     NULL );
//
//             FatTearDownAllocationSupport( IrpContext, Vcb );
//
//             FatSetVcbCondition( Vcb, VcbNotMounted);
//
//             ClearVerify = TRUE;
//
//         } else if (!VerifyAlreadyDone) {
//
//             //
//             //  Grab everything so we can safely transition the volume state without
//             //  having a thread stumble into the torn-down allocation engine.
//             //
//
//             if (!ReleaseEntireVolume) {
//                 FatAcquireExclusiveVolume( IrpContext, Vcb );
//                 ReleaseEntireVolume = TRUE;
//             }
//
//             //
//             //  Get rid of any cached data, flushing first.
//             //
//             //  Future work (and for bonus points, around the other flush points)
//             //  could address the possibility that the dirent filesize hasn't been
//             //  updated yet, causing us to fail the re-verification of a file in
//             //  DetermineAndMark. This is pretty subtle and very very uncommon.
//             //
//
//             FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, Flush );
//
//             //
//             //  Flush and Purge the volume file.
//             //
//
//             (VOID)FatFlushFat( IrpContext, Vcb );
//             CcPurgeCacheSection( &Vcb->SectionObjectPointers, NULL, 0, FALSE );
//
//             //
//             //  Redo the allocation support with newly paged stuff.
//             //
//
//             FatTearDownAllocationSupport( IrpContext, Vcb );
//             FatSetupAllocationSupport( IrpContext, Vcb );
//
//             FatCheckDirtyBit( IrpContext, Vcb );
//
//             //
//             //  Check for write protected media.
//             //
//
//             if (FatIsMediaWriteProtected(IrpContext, Vcb->TargetDeviceObject)) {
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//
//             } else {
//
//                 ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED );
//             }
//
//             ClearVerify = TRUE;
//         }
//
//         if (ClearVerify) {
//
//             //
//             //  Mark the device as no longer needing verification.
//             //
//
//             ClearFlag( Vpb->RealDevice->Flags, DO_VERIFY_VOLUME );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatVerifyVolume );
//
//         //
//         //  Free any buffer we may have allocated
//         //
//
//         if ( BootSector != NULL ) { ExFreePool( BootSector ); }
//         if ( RootDirectory != NULL ) { ExFreePool( RootDirectory ); }
//
//         //
//         //  Show that we are done with this volume.
//         //
//
//         NT_ASSERT( Vcb->VerifyThread == KeGetCurrentThread() );
//         Vcb->VerifyThread = NULL;
//
//         if (ReleaseEntireVolume) {
//
//             FatReleaseVolume( IrpContext, Vcb );
//         }
//
//         FatReleaseVcb( IrpContext, Vcb );
//         FatReleaseGlobal( IrpContext );
//
//         //
//         //  If this was not an abnormal termination, complete the irp.
//         //
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatVerifyVolume -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
/* BOOLEAN */
// FatIsBootSectorFat (
//     IN PPACKED_BOOT_SECTOR BootSector
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if the boot sector is for a fat file volume.
//
// Arguments:
//
//     BootSector - Supplies the packed boot sector to check
//
// Return Value:
//
//     BOOLEAN - TRUE if the volume is Fat and FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN Result;
//     BIOS_PARAMETER_BLOCK Bpb = {0};
//
//     DebugTrace(+1, Dbg, "FatIsBootSectorFat, BootSector = %p\n", BootSector);
//
//     //
//     //  The result is true unless we decide that it should be false
//     //
//
//     Result = TRUE;
//
//     //
//     //  Unpack the bios and then test everything
//     //
//
//     FatUnpackBios( &Bpb, &BootSector->PackedBpb );
//     if (Bpb.Sectors != 0) { Bpb.LargeSectors = 0; }
//
//     if ((BootSector->Jump[0] != 0xe9) &&
//         (BootSector->Jump[0] != 0xeb) &&
//         (BootSector->Jump[0] != 0x49)) {
//
//         Result = FALSE;
//
//     //
//     //  Enforce some sanity on the sector size (easy check)
//     //
//
//     } else if ((Bpb.BytesPerSector !=  128) &&
//                (Bpb.BytesPerSector !=  256) &&
//                (Bpb.BytesPerSector !=  512) &&
//                (Bpb.BytesPerSector != 1024) &&
//                (Bpb.BytesPerSector != 2048) &&
//                (Bpb.BytesPerSector != 4096)) {
//
//         Result = FALSE;
//
//     //
//     //  Likewise on the clustering.
//     //
//
//     } else if ((Bpb.SectorsPerCluster !=  1) &&
//                (Bpb.SectorsPerCluster !=  2) &&
//                (Bpb.SectorsPerCluster !=  4) &&
//                (Bpb.SectorsPerCluster !=  8) &&
//                (Bpb.SectorsPerCluster != 16) &&
//                (Bpb.SectorsPerCluster != 32) &&
//                (Bpb.SectorsPerCluster != 64) &&
//                (Bpb.SectorsPerCluster != 128)) {
//
//         Result = FALSE;
//
//     //
//     //  Likewise on the reserved sectors (must reflect at least the boot sector!)
//     //
//
//     } else if (Bpb.ReservedSectors == 0) {
//
//         Result = FALSE;
//
//     //
//     //  No FATs? Wrong ...
//     //
//
//     } else if (Bpb.Fats == 0) {
//
//         Result = FALSE;
//
//     //
//     // Prior to DOS 3.2 might contains value in both of Sectors and
//     // Sectors Large.
//     //
//
//     } else if ((Bpb.Sectors == 0) && (Bpb.LargeSectors == 0)) {
//
//         Result = FALSE;
//
//     //
//     //  Check that FAT32 (SectorsPerFat == 0) claims some FAT space and
//     //  is of a version we recognize, currently Version 0.0.
//     //
//
//     } else if (Bpb.SectorsPerFat == 0 && ( Bpb.LargeSectorsPerFat == 0 ||
//                                            Bpb.FsVersion != 0 )) {
//
//         Result = FALSE;
//
//     } else if ((Bpb.Media != 0xf0) &&
//                (Bpb.Media != 0xf8) &&
//                (Bpb.Media != 0xf9) &&
//                (Bpb.Media != 0xfb) &&
//                (Bpb.Media != 0xfc) &&
//                (Bpb.Media != 0xfd) &&
//                (Bpb.Media != 0xfe) &&
//                (Bpb.Media != 0xff) &&
//                (!FatData.FujitsuFMR || ((Bpb.Media != 0x00) &&
//                                         (Bpb.Media != 0x01) &&
//                                         (Bpb.Media != 0xfa)))) {
//
//         Result = FALSE;
//
//     //
//     //  If this isn't FAT32, then there better be a claimed root directory
//     //  size here ...
//     //
//
//     } else if (Bpb.SectorsPerFat != 0 && Bpb.RootEntries == 0) {
//
//         Result = FALSE;
//
//     //
//     //  If this is FAT32 (i.e., extended BPB), look for and refuse to mount
//     //  mirror-disabled volumes. If we did, we would need to only write to
//     //  the FAT# indicated in the ActiveFat field. The only user of this is
//     //  the FAT->FAT32 converter after the first pass of protected mode work
//     //  (booting into realmode) and NT should absolutely not be attempting
//     //  to mount such an in-transition volume.
//     //
//
//     } else if (Bpb.SectorsPerFat == 0 && Bpb.MirrorDisabled) {
//
//         Result = FALSE;
//     }
//
//     DebugTrace(-1, Dbg, "FatIsBootSectorFat -> %08lx\n", Result);
//
//     return Result;
// }
//
//
// //
// //  Local Support Routine
// //
//
/* BOOLEAN */
// FatIsMediaWriteProtected (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT TargetDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if the target media is write protected.
//
// Arguments:
//
//     TargetDeviceObject - The target of the query
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK Iosb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Query the partition table
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  See if the media is write protected.  On success or any kind
//     //  of error (possibly illegal device function), assume it is
//     //  writeable, and only complain if he tells us he is write protected.
//     //
//
//     Irp = IoBuildDeviceIoControlRequest( IOCTL_DISK_IS_WRITABLE,
//                                          TargetDeviceObject,
//                                          NULL,
//                                          0,
//                                          NULL,
//                                          0,
//                                          FALSE,
//                                          &Event,
//                                          &Iosb );
//
//     //
//     //  Just return FALSE in the unlikely event we couldn't allocate an Irp.
//     //
//
//     if ( Irp == NULL ) {
//
//         return FALSE;
//     }
//
//     SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//
//     Status = IoCallDriver( TargetDeviceObject, Irp );
//
//     if ( Status == STATUS_PENDING ) {
//
//         (VOID) KeWaitForSingleObject( &Event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       (PLARGE_INTEGER)NULL );
//
//         Status = Iosb.Status;
//     }
//
//     return (BOOLEAN)(Status == STATUS_MEDIA_WRITE_PROTECTED);
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatUserFsCtrl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
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
//     ULONG FsControlCode;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PAGED_CODE();
//
//     //
//     //  Save some references to make our life a little easier
//     //
//
//     FsControlCode = IrpSp->Parameters.FileSystemControl.FsControlCode;
//
//     DebugTrace(+1, Dbg,"FatUserFsCtrl...\n", 0);
//     DebugTrace( 0, Dbg,"FsControlCode = %08lx\n", FsControlCode);
//
//     //
//     //  Some of these Fs Controls use METHOD_NEITHER buffering.  If the previous mode
//     //  of the caller was userspace and this is a METHOD_NEITHER, we have the choice
//     //  of realy buffering the request through so we can possibly post, or making the
//     //  request synchronous.  Since the former was not done by design, do the latter.
//     //
//
//     if (Irp->RequestorMode != KernelMode && (FsControlCode & 3) == METHOD_NEITHER) {
//
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//     }
//
//     //
//     //  Case on the control code.
//     //
//
//     switch ( FsControlCode ) {
//
//     case FSCTL_REQUEST_OPLOCK_LEVEL_1:
//     case FSCTL_REQUEST_OPLOCK_LEVEL_2:
//     case FSCTL_REQUEST_BATCH_OPLOCK:
//     case FSCTL_OPLOCK_BREAK_ACKNOWLEDGE:
//     case FSCTL_OPBATCH_ACK_CLOSE_PENDING:
//     case FSCTL_OPLOCK_BREAK_NOTIFY:
//     case FSCTL_OPLOCK_BREAK_ACK_NO_2:
//     case FSCTL_REQUEST_FILTER_OPLOCK:
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//     case FSCTL_REQUEST_OPLOCK:
// #endif
//         Status = FatOplockRequest( IrpContext, Irp );
//         break;
//
//     case FSCTL_LOCK_VOLUME:
//
//         Status = FatLockVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_UNLOCK_VOLUME:
//
//         Status = FatUnlockVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_DISMOUNT_VOLUME:
//
//         Status = FatDismountVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_MARK_VOLUME_DIRTY:
//
//         Status = FatDirtyVolume( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_VOLUME_DIRTY:
//
//         Status = FatIsVolumeDirty( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_VOLUME_MOUNTED:
//
//         Status = FatIsVolumeMounted( IrpContext, Irp );
//         break;
//
//     case FSCTL_IS_PATHNAME_VALID:
//         Status = FatIsPathnameValid( IrpContext, Irp );
//         break;
//
//     case FSCTL_QUERY_RETRIEVAL_POINTERS:
//         Status = FatQueryRetrievalPointers( IrpContext, Irp );
//         break;
//
//     case FSCTL_QUERY_FAT_BPB:
//         Status = FatQueryBpb( IrpContext, Irp );
//         break;
//
//     case FSCTL_FILESYSTEM_GET_STATISTICS:
//         Status = FatGetStatistics( IrpContext, Irp );
//         break;
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//     case FSCTL_GET_RETRIEVAL_POINTER_BASE:
//         Status = FatGetRetrievalPointerBase( IrpContext, Irp );
//         break;
//
//     case FSCTL_GET_BOOT_AREA_INFO:
//         Status = FatGetBootAreaInfo( IrpContext, Irp );
//         break;
// #endif
//
//     case FSCTL_GET_VOLUME_BITMAP:
//         Status = FatGetVolumeBitmap( IrpContext, Irp );
//         break;
//
//     case FSCTL_GET_RETRIEVAL_POINTERS:
//         Status = FatGetRetrievalPointers( IrpContext, Irp );
//         break;
//
//     case FSCTL_MOVE_FILE:
//         Status = FatMoveFile( IrpContext, Irp );
//         break;
//
//     case FSCTL_ALLOW_EXTENDED_DASD_IO:
//         Status = FatAllowExtendedDasdIo( IrpContext, Irp );
//         break;
//
//     case FSCTL_MARK_HANDLE:
//         Status = FatMarkHandle( IrpContext, Irp );
//         break;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     case FSCTL_SET_PURGE_FAILURE_MODE:
//         Status = FatSetPurgeFailureMode( IrpContext, Irp );
//         break;
//
// #endif
//
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//     case FSCTL_SET_ZERO_ON_DEALLOCATION:
//         Status = FatSetZeroOnDeallocate( IrpContext, Irp );
//         break;
// #endif
//
//     default :
//
//         DebugTrace(0, Dbg, "Invalid control code -> %08lx\n", FsControlCode );
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         Status = STATUS_INVALID_DEVICE_REQUEST;
//         break;
//     }
//
//     DebugTrace(-1, Dbg, "FatUserFsCtrl -> %08lx\n", Status );
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
// FatOplockRequest (
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
//     ULONG FsControlCode;
//     PFCB Fcb;
//     PVCB Vcb;
//     PCCB Ccb;
//
//     ULONG OplockCount = 0;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     BOOLEAN AcquiredVcb = FALSE;
//     BOOLEAN AcquiredFcb = FALSE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//     PREQUEST_OPLOCK_INPUT_BUFFER InputBuffer = NULL;
//     ULONG InputBufferLength;
//     ULONG OutputBufferLength;
// #endif
//
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PAGED_CODE();
//
//     //
//     //  Save some references to make our life a little easier
//     //
//
//     FsControlCode = IrpSp->Parameters.FileSystemControl.FsControlCode;
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     DebugTrace(+1, Dbg, "FatOplockRequest...\n", 0);
//     DebugTrace( 0, Dbg, "FsControlCode = %08lx\n", FsControlCode);
//
//     //
//     //  We permit oplock requests on files and directories.
//     //
//
//     if ((TypeOfOpen != UserFileOpen)
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         &&
//         (TypeOfOpen != UserDirectoryOpen)
// #endif
//         ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         DebugTrace(-1, Dbg, "FatOplockRequest -> STATUS_INVALID_PARAMETER\n", 0);
//         return STATUS_INVALID_PARAMETER;
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
//     //
//     //  Get the input & output buffer lengths and pointers.
//     //
//
//     if (FsControlCode == FSCTL_REQUEST_OPLOCK) {
//
//         InputBufferLength = IrpSp->Parameters.FileSystemControl.InputBufferLength;
//         InputBuffer = (PREQUEST_OPLOCK_INPUT_BUFFER) Irp->AssociatedIrp.SystemBuffer;
//
//         OutputBufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//         //
//         //  Check for a minimum length on the input and ouput buffers.
//         //
//
//         if ((InputBufferLength < sizeof( REQUEST_OPLOCK_INPUT_BUFFER )) ||
//             (OutputBufferLength < sizeof( REQUEST_OPLOCK_OUTPUT_BUFFER ))) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//             DebugTrace(-1, Dbg, "FatOplockRequest -> STATUS_BUFFER_TOO_SMALL\n", 0);
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//     }
//
//     //
//     //  If the oplock request is on a directory it must be for a Read or Read-Handle
//     //  oplock only.
//     //
//
//     if ((TypeOfOpen == UserDirectoryOpen) &&
//         ((FsControlCode != FSCTL_REQUEST_OPLOCK) ||
//          !FsRtlOplockIsSharedRequest( Irp ))) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         DebugTrace(-1, Dbg, "FatOplockRequest -> STATUS_INVALID_PARAMETER\n", 0);
//         return STATUS_INVALID_PARAMETER;
//     }
//
// #endif
//
//     //
//     //  Make this a waitable Irpcontext so we don't fail to acquire
//     //  the resources.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Use a try finally to free the Fcb/Vcb
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  We grab the Fcb exclusively for oplock requests, shared for oplock
//         //  break acknowledgement.
//         //
//
//         if ((FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_1) ||
//             (FsControlCode == FSCTL_REQUEST_BATCH_OPLOCK) ||
//             (FsControlCode == FSCTL_REQUEST_FILTER_OPLOCK) ||
//             (FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_2)
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//             ||
//             ((FsControlCode == FSCTL_REQUEST_OPLOCK) && FlagOn( InputBuffer->Flags, REQUEST_OPLOCK_INPUT_FLAG_REQUEST ))
// #endif
//             ) {
//
//             FatAcquireSharedVcb( IrpContext, Fcb->Vcb );
//             AcquiredVcb = TRUE;
//             FatAcquireExclusiveFcb( IrpContext, Fcb );
//             AcquiredFcb = TRUE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//             if (FsRtlOplockIsSharedRequest( Irp )) {
// #else
//             if (FsControlCode == FSCTL_REQUEST_OPLOCK_LEVEL_2) {
// #endif
//
//                 //
//                 //  Byte-range locks are only valid on files.
//                 //
//
//                 if (TypeOfOpen == UserFileOpen) {
//
//                     //
//                     //  Set OplockCount to nonzero if FsRtl denies access
//                     //  based on current byte-range lock state.
//                     //
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                     OplockCount = (ULONG) !FsRtlCheckLockForOplockRequest( &Fcb->Specific.Fcb.FileLock, &Fcb->Header.AllocationSize );
// #elif (NTDDI_VERSION >= NTDDI_WIN7)
//                     OplockCount = (ULONG) FsRtlAreThereCurrentOrInProgressFileLocks( &Fcb->Specific.Fcb.FileLock );
// #else
//                     OplockCount = (ULONG) FsRtlAreThereCurrentFileLocks( &Fcb->Specific.Fcb.FileLock );
// #endif
//
//                 }
//
//             } else {
//
//                 OplockCount = Fcb->UncleanCount;
//             }
//
//         } else if ((FsControlCode == FSCTL_OPLOCK_BREAK_ACKNOWLEDGE) ||
//                    (FsControlCode == FSCTL_OPBATCH_ACK_CLOSE_PENDING) ||
//                    (FsControlCode == FSCTL_OPLOCK_BREAK_NOTIFY) ||
//                    (FsControlCode == FSCTL_OPLOCK_BREAK_ACK_NO_2)
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//                    ||
//                    ((FsControlCode == FSCTL_REQUEST_OPLOCK) && FlagOn( InputBuffer->Flags, REQUEST_OPLOCK_INPUT_FLAG_ACK ))
// #endif
//                     ) {
//
//             FatAcquireSharedFcb( IrpContext, Fcb );
//             AcquiredFcb = TRUE;
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//         } else if (FsControlCode == FSCTL_REQUEST_OPLOCK) {
//
//             //
//             //  The caller didn't provide either REQUEST_OPLOCK_INPUT_FLAG_REQUEST or
//             //  REQUEST_OPLOCK_INPUT_FLAG_ACK on the input buffer.
//             //
//
//             try_leave( Status = STATUS_INVALID_PARAMETER );
//
//         } else {
// #else
//         } else {
// #endif
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( FsControlCode, 0, 0 );
//         }
//
//         //
//         //  Fail batch, filter, and handle oplock requests if the file is marked
//         //  for delete.
//         //
//
//         if (((FsControlCode == FSCTL_REQUEST_FILTER_OPLOCK) ||
//              (FsControlCode == FSCTL_REQUEST_BATCH_OPLOCK)
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//              ||
//              ((FsControlCode == FSCTL_REQUEST_OPLOCK) && FlagOn( InputBuffer->RequestedOplockLevel, OPLOCK_LEVEL_CACHE_HANDLE ))
// #endif
//             ) &&
//             FlagOn( Fcb->FcbState, FCB_STATE_DELETE_ON_CLOSE )) {
//
//             try_leave( Status = STATUS_DELETE_PENDING );
//         }
//
//         //
//         //  Call the FsRtl routine to grant/acknowledge oplock.
//         //
//
//         Status = FsRtlOplockFsctrl( FatGetFcbOplock(Fcb),
//                                     Irp,
//                                     OplockCount );
//
//         //
//         //  Once we call FsRtlOplockFsctrl, we no longer own the IRP and we should not complete it.
//         //
//
//         Irp = NULL;
//
//         //
//         //  Set the flag indicating if Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = FatIsFastIoPossible( Fcb );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatOplockRequest );
//
//         //
//         //  Release all of our resources
//         //
//
//         if (AcquiredVcb) {
//
//             FatReleaseVcb( IrpContext, Fcb->Vcb );
//         }
//
//         if (AcquiredFcb)  {
//
//             FatReleaseFcb( IrpContext, Fcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatOplockRequest -> %08lx\n", Status );
//     } _SEH2_END;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatLockVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatLockVolume...\n", 0);
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatLockVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatLockVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
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
//     //  Acquire exclusive access to the Vcb and enqueue the Irp if we
//     //  didn't get access.
//     //
//
//     if (!FatAcquireExclusiveVcb( IrpContext, Vcb )) {
//
//         DebugTrace( 0, Dbg, "Cannot acquire Vcb\n", 0);
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatUnlockVolume -> %08lx\n", Status);
//         return Status;
//     }
//
//     _SEH2_TRY {
//
//         Status = FatLockVolumeInternal( IrpContext, Vcb, IrpSp->FileObject );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Since we drop and release the vcb while trying to punch the volume
//         //  down, it may be the case that we decide the operation should not
//         //  continue if the user raced a CloeseHandle() with us (and it finished
//         //  the cleanup) while we were waiting for our closes to finish.
//         //
//         //  In this case, we will have been raised out of the acquire logic with
//         //  STATUS_FILE_CLOSED, and the volume will not be held.
//         //
//
//         if (!_SEH2_AbnormalTermination() || ExIsResourceAcquiredExclusiveLite( &Vcb->Resource )) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         if (!NT_SUCCESS( Status ) || _SEH2_AbnormalTermination()) {
//
//             //
//             //  The volume lock will be failing.
//             //
//
//             FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_LOCK_FAILED );
//         }
//     } _SEH2_END;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatLockVolume -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatUnlockVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatUnlockVolume...\n", 0);
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatUnlockVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatUnlockVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Status = FatUnlockVolumeInternal( IrpContext, Vcb, IrpSp->FileObject );
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
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatUnlockVolume -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatLockVolumeInternal (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_OBJECT FileObject OPTIONAL
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
//     NTSTATUS Status = STATUS_SUCCESS;
//     KIRQL SavedIrql;
//     ULONG RemainingUserReferences = (FileObject? 1: 0);
//
//     NT_ASSERT( ExIsResourceAcquiredExclusiveLite( &Vcb->Resource ) &&
//             !ExIsResourceAcquiredExclusiveLite( &FatData.Resource ));
//     //
//     //  Go synchronous for the rest of the lock operation.  It may be
//     //  reasonable to try to revisit this in the future, but for now
//     //  the purge below expects to be able to wait.
//     //
//     //  We know it is OK to leave the flag up given how we're used at
//     //  the moment.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  If there are any open handles, this will fail.
//     //
//
//     if (!FatIsHandleCountZero( IrpContext, Vcb )) {
//
//         return STATUS_ACCESS_DENIED;
//     }
//
//     //
//     //  Force Mm to get rid of its referenced file objects.
//     //
//
//     FatFlushFat( IrpContext, Vcb );
//
//     FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, Flush );
//
//     FatCloseEaFile( IrpContext, Vcb, TRUE );
//
//     //
//     //  Now back out of our synchronization and wait for the lazy writer
//     //  to finish off any lazy closes that could have been outstanding.
//     //
//     //  Since we flushed, we know that the lazy writer will issue all
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
//     FatReleaseVcb( IrpContext, Vcb );
//
//     Status = CcWaitForCurrentLazyWriterActivity();
//
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     if (!NT_SUCCESS( Status )) {
//
//         return Status;
//     }
//
//     //
//     //  The act of closing and purging may have touched pages in various
//     //  parent DCBs. We handle this by purging a second time.
//     //
//
//     FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, Flush );
//
//     FatReleaseVcb( IrpContext, Vcb );
//
//     Status = CcWaitForCurrentLazyWriterActivity();
//
//     FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     if (!NT_SUCCESS( Status )) {
//
//         return Status;
//     }
//
//     //
//     //  Now rundown the delayed closes one last time.  We appear to be able
//     //  to have additional collisions.
//     //
//
//     FatFspClose( Vcb );
//
//     //
//     //  Check if the Vcb is already locked, or if the open file count
//     //  is greater than 1 (which implies that someone else also is
//     //  currently using the volume, or a file on the volume), and that the
//     //  VPB reference count only includes our residual and the handle (as
//     //  appropriate).
//     //
//     //  We used to only check for the vpb refcount.  This is unreliable since
//     //  the vpb refcount is dropped immediately before final close, meaning
//     //  that even though we had a good refcount, the close was inflight and
//     //  subsequent operations could get confused.  Especially if the PNP path
//     //  was the lock caller, we delete the VCB with an outstanding opencount!
//     //
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     if (!FlagOn(Vcb->Vpb->Flags, VPB_LOCKED) &&
//         (Vcb->Vpb->ReferenceCount <= 2 + RemainingUserReferences) &&
//         (Vcb->OpenFileCount == (CLONG)( FileObject? 1: 0 ))) {
//
//         SetFlag(Vcb->Vpb->Flags, (VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED));
//         SetFlag(Vcb->VcbState, VCB_STATE_FLAG_LOCKED);
//         Vcb->FileObjectWithVcbLocked = FileObject;
//
//     } else {
//
//         Status = STATUS_ACCESS_DENIED;
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     //
//     //  If we successully locked the volume, see if it is clean now.
//     //
//
//     if (NT_SUCCESS( Status ) &&
//         FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY ) &&
//         !FlagOn( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY ) &&
//         !CcIsThereDirtyData(Vcb->Vpb)) {
//
//         FatMarkVolume( IrpContext, Vcb, VolumeClean );
//         ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//     }
//
//     NT_ASSERT( !NT_SUCCESS(Status) || (Vcb->OpenFileCount == (CLONG)( FileObject? 1: 0 )));
//
//     return Status;
// }
//
//
// NTSTATUS
// FatUnlockVolumeInternal (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_OBJECT FileObject OPTIONAL
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
//     KIRQL SavedIrql;
//     NTSTATUS Status = STATUS_NOT_LOCKED;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     if (FlagOn(Vcb->Vpb->Flags, VPB_LOCKED) && FileObject == Vcb->FileObjectWithVcbLocked) {
//
//         //
//         //  This one locked it, unlock the volume
//         //
//
//         ClearFlag( Vcb->Vpb->Flags, (VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED) );
//         ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_LOCKED );
//         Vcb->FileObjectWithVcbLocked = NULL;
//
//         Status = STATUS_SUCCESS;
//     }
//
//     IoReleaseVpbSpinLock( SavedIrql );
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatDismountVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the dismount volume operation.  It is responsible for
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
//     PIO_STACK_LOCATION IrpSp;
//     NTSTATUS Status = STATUS_SUCCESS;
//     BOOLEAN VcbHeld = FALSE;
//     KIRQL SavedIrql;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatDismountVolume...\n", 0);
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens on media that is not boot/paging and is not
//     //  already dismounted ... (but we need to check that stuff while
//     //  synchronized)
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         Status = STATUS_INVALID_PARAMETER;
//         goto fn_return;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatDismountVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Make some unsynchronized checks to see if this operation is possible.
//     //  We will repeat the appropriate ones inside synchronization, but it is
//     //  good to avoid bogus notifications.
//     //
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE )) {
//
//         Status = STATUS_ACCESS_DENIED;
//         goto fn_return;
//     }
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED )) {
//
//         Status = STATUS_VOLUME_DISMOUNTED;
//         goto fn_return;
//     }
//
//     //
//     //  A bit of historical comment is in order.
//     //
//     //  In all versions prior to NT5, we only permitted dismount if the volume had
//     //  previously been locked.  Now we must permit a forced dismount, meaning that
//     //  we grab ahold of the whole kit-n-kaboodle - regardless of activity, open
//     //  handles, etc. - to flush and invalidate the volume.
//     //
//     //  Previously, dismount assumed that lock had come along earlier and done some
//     //  of the work that we are now going to do - i.e., flush, tear down the eas. All
//     //  we had to do here is flush the device out and kill off as many of the orphan
//     //  fcbs as possible. This now changes.
//     //
//     //  In fact, everything is a forced dismount now. This changes one interesting
//     //  aspect, which is that it used to be the case that the handle used to dismount
//     //  could come back, read, and induce a verify/remount. This is just not possible
//     //  now.  The point of forced dismount is that very shortly someone will come along
//     //  and be destructive to the possibility of using the media further - format, eject,
//     //  etc.  By using this path, callers are expected to tolerate the consequences.
//     //
//     //  Note that the volume can still be successfully unlocked by this handle.
//     //
//
//     //
//     //  Send notification.
//     //
//
//     FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_DISMOUNT );
//
//     //
//     //  Force ourselves to wait and grab everything.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     (VOID)FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     _SEH2_TRY {
//
//         //
//         //  Guess what? This can raise if a cleanup on the fileobject we
//         //  got races in ahead of us.
//         //
//
//         FatAcquireExclusiveVolume( IrpContext, Vcb );
//         VcbHeld = TRUE;
//
//         if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED )) {
//
//             try_return( Status = STATUS_VOLUME_DISMOUNTED );
//         }
//
//         FatFlushAndCleanVolume( IrpContext, Irp, Vcb, FlushAndInvalidate );
//
//         //
//         //  We defer the physical dismount until this handle is closed, per symmetric
//         //  implemntation in the other FS. This permits a dismounter to issue IOCTL
//         //  through this handle and perform device manipulation without racing with
//         //  creates attempting to mount the volume again.
//         //
//         //  Raise a flag to tell the cleanup path to complete the dismount.
//         //
//
//         SetFlag( Ccb->Flags, CCB_FLAG_COMPLETE_DISMOUNT );
//
//         //
//         //  Indicate that the volume was dismounted so that we may return the
//         //  correct error code when operations are attempted via open handles.
//         //
//
//         FatSetVcbCondition( Vcb, VcbBad);
//
//         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DISMOUNTED );
//
//         //
//         //  Set a flag in the VPB to let others know that direct volume access is allowed.
//         //
//
//         IoAcquireVpbSpinLock( &SavedIrql );
//         SetFlag( Vcb->Vpb->Flags, VPB_DIRECT_WRITES_ALLOWED );
//         IoReleaseVpbSpinLock( SavedIrql );
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//         FsRtlDismountComplete( Vcb->TargetDeviceObject, Status );
//
// #endif
//
//         if (VcbHeld) {
//
//             FatReleaseVolume( IrpContext, Vcb );
//         }
//
//         FatReleaseGlobal( IrpContext );
//
//         //
//         //  I do not believe it is possible to raise, but for completeness
//         //  notice and send notification of failure.  We absolutely
//         //  cannot have raised in CheckForDismount.
//         //
//         //  We decline to call an attempt to dismount a dismounted volume
//         //  a failure to do so.
//         //
//
//         if ((!NT_SUCCESS( Status ) && Status != STATUS_VOLUME_DISMOUNTED)
//             || _SEH2_AbnormalTermination()) {
//
//             FsRtlNotifyVolumeEvent( IrpSp->FileObject, FSRTL_VOLUME_DISMOUNT_FAILED );
//         }
//     } _SEH2_END;
//
//     fn_return:
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//     DebugTrace(-1, Dbg, "FatDismountVolume -> %08lx\n", Status);
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatDirtyVolume (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine marks the volume as dirty.
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
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatDirtyVolume...\n", 0);
//
//     //
//     //  Decode the file object, the only type of opens we accept are
//     //  user volume opens.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatDirtyVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatDirtyVolume -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//
//     //
//     //  Disable popups, we will just return any error.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS);
//
//     //
//     //  Verify the Vcb.  We want to make sure we don't dirty some
//     //  random chunk of media that happens to be in the drive now.
//     //
//
//     FatVerifyVcb( IrpContext, Vcb );
//
//     SetFlag( Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY );
//
//     FatMarkVolume( IrpContext, Vcb, VolumeDirty );
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     DebugTrace(-1, Dbg, "FatDirtyVolume -> STATUS_SUCCESS\n", 0);
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatIsVolumeDirty (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     PVCB Vcb;
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
//     //  Get a pointer to the output buffer.  Look at the system buffer field in the
//     //  irp first.  Then the Irp Mdl.
//     //
//
//     if (Irp->AssociatedIrp.SystemBuffer != NULL) {
//
//         VolumeState = Irp->AssociatedIrp.SystemBuffer;
//
//     } else if (Irp->MdlAddress != NULL) {
//
//         VolumeState = MmGetSystemAddressForMdlSafe( Irp->MdlAddress, LowPagePriority | MdlMappingNoExecute );
//
//         if (VolumeState == NULL) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_INSUFFICIENT_RESOURCES );
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//     } else {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_USER_BUFFER );
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
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     *VolumeState = 0;
//
//     //
//     //  Decode the file object
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     if (TypeOfOpen != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (Vcb->VcbCondition != VcbGood) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_VOLUME_DISMOUNTED );
//         return STATUS_VOLUME_DISMOUNTED;
//     }
//
//     //
//     //  Disable PopUps, we want to return any error.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS);
//
//     //
//     //  Verify the Vcb.  We want to make double sure that this volume
//     //  is around so that we know our information is good.
//     //
//
//     FatVerifyVcb( IrpContext, Vcb );
//
//     //
//     //  Now set the returned information.  We can avoid probing the disk since
//     //  we know our internal state is in sync.
//     //
//
//     if ( FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY) ) {
//
//         SetFlag( *VolumeState, VOLUME_IS_DIRTY );
//     }
//
//     Irp->IoStatus.Information = sizeof( ULONG );
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatIsVolumeMounted (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
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
//     NTSTATUS Status;
//
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb = NULL;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     Status = STATUS_SUCCESS;
//
//     DebugTrace(+1, Dbg, "FatIsVolumeMounted...\n", 0);
//
//     //
//     //  Decode the file object.
//     //
//
//     (VOID)FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     NT_ASSERT( Vcb != NULL );
//     _Analysis_assume_( Vcb != NULL );
//
//     //
//     //  Disable PopUps, we want to return any error.
//     //
//
//     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_POPUPS);
//
//     //
//     //  Verify the Vcb.
//     //
//
//     FatVerifyVcb( IrpContext, Vcb );
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatIsVolumeMounted -> %08lx\n", Status);
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatIsPathnameValid (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if a pathname is a-priori illegal by inspecting
//     the the characters used.  It is required to be correct on a FALSE return.
//
//     N.B.: current implementation is intentioanlly a no-op.  This may change
//     in the future.  A careful reader of the previous implementation of this
//     FSCTL in FAT would discover that it violated the requirement stated above
//     and could return FALSE for a valid (createable) pathname.
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
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatIsPathnameValid...\n", 0);
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     DebugTrace(-1, Dbg, "FatIsPathnameValid -> %08lx\n", STATUS_SUCCESS);
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatQueryBpb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine simply returns the first 0x24 bytes of sector 0.
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
//     PVCB Vcb;
//
//     PFSCTL_QUERY_FAT_BPB_BUFFER BpbBuffer;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatQueryBpb...\n", 0);
//
//     //
//     //  Get the Vcb.  If we didn't keep the information needed for this call,
//     //  we had a reason ...
//     //
//
//     Vcb = &((PVOLUME_DEVICE_OBJECT)IrpSp->DeviceObject)->Vcb;
//
//     if (Vcb->First0x24BytesOfBootSector == NULL) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_DEVICE_REQUEST );
//         DebugTrace(-1, Dbg, "FatQueryBpb -> %08lx\n", STATUS_INVALID_DEVICE_REQUEST );
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     //
//     // Extract the buffer
//     //
//
//     BpbBuffer = (PFSCTL_QUERY_FAT_BPB_BUFFER)Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Make sure the buffer is big enough.
//     //
//
//     if (IrpSp->Parameters.FileSystemControl.OutputBufferLength < 0x24) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         DebugTrace(-1, Dbg, "FatQueryBpb -> %08lx\n", STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     //  Fill in the output buffer
//     //
//
//     RtlCopyMemory( BpbBuffer->First0x24BytesOfBootSector,
//                    Vcb->First0x24BytesOfBootSector,
//                    0x24 );
//
//     Irp->IoStatus.Information = 0x24;
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     DebugTrace(-1, Dbg, "FatQueryBpb -> %08lx\n", STATUS_SUCCESS);
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatInvalidateVolumes (
//     IN PIRP Irp
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
//     IRP_CONTEXT IrpContext;
//     PIO_STACK_LOCATION IrpSp;
//
//     LUID TcbPrivilege = {SE_TCB_PRIVILEGE, 0};
//
//     HANDLE Handle;
//
//     PLIST_ENTRY Links;
//
//     PFILE_OBJECT FileToMarkBad;
//     PDEVICE_OBJECT DeviceToMarkBad;
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatInvalidateVolumes...\n", 0);
//
//     //
//     //  Check for the correct security access.
//     //  The caller must have the SeTcbPrivilege.
//     //
//
//     if (!SeSinglePrivilegeCheck(TcbPrivilege, Irp->RequestorMode)) {
//
//         FatCompleteRequest( FatNull, Irp, STATUS_PRIVILEGE_NOT_HELD );
//
//         DebugTrace(-1, Dbg, "FatInvalidateVolumes -> %08lx\n", STATUS_PRIVILEGE_NOT_HELD);
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     //
//     //  Try to get a pointer to the device object from the handle passed in.
//     //
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     if (IoIs32bitProcess( Irp )) {
//
//         if (IrpSp->Parameters.FileSystemControl.InputBufferLength != sizeof(UINT32)) {
//
//             FatCompleteRequest( FatNull, Irp, STATUS_INVALID_PARAMETER );
//
//             DebugTrace(-1, Dbg, "FatInvalidateVolumes -> %08lx\n", STATUS_INVALID_PARAMETER);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         Handle = (HANDLE) LongToHandle( (*(PUINT32)Irp->AssociatedIrp.SystemBuffer) );
//     } else {
// #endif
//         if (IrpSp->Parameters.FileSystemControl.InputBufferLength != sizeof(HANDLE)) {
//
//             FatCompleteRequest( FatNull, Irp, STATUS_INVALID_PARAMETER );
//
//             DebugTrace(-1, Dbg, "FatInvalidateVolumes -> %08lx\n", STATUS_INVALID_PARAMETER);
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         Handle = *(PHANDLE)Irp->AssociatedIrp.SystemBuffer;
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     }
// #endif
//
//
//     Status = ObReferenceObjectByHandle( Handle,
//                                         0,
//                                         *IoFileObjectType,
//                                         KernelMode,
// #ifndef __REACTOS__
//                                         &FileToMarkBad,
// #else
//                                         (PVOID *)&FileToMarkBad,
// #endif
//                                         NULL );
//
//     if (!NT_SUCCESS(Status)) {
//
//         FatCompleteRequest( FatNull, Irp, Status );
//
//         DebugTrace(-1, Dbg, "FatInvalidateVolumes -> %08lx\n", Status);
//         return Status;
//
//     } else {
//
//         //
//         //  We only needed the pointer, not a reference.
//         //
//
//         ObDereferenceObject( FileToMarkBad );
//
//         //
//         //  Grab the DeviceObject from the FileObject.
//         //
//
//         DeviceToMarkBad = FileToMarkBad->DeviceObject;
//     }
//
//     RtlZeroMemory( &IrpContext, sizeof(IRP_CONTEXT) );
//
//     SetFlag( IrpContext.Flags, IRP_CONTEXT_FLAG_WAIT );
//     IrpContext.MajorFunction = IrpSp->MajorFunction;
//     IrpContext.MinorFunction = IrpSp->MinorFunction;
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     FatAcquireExclusiveGlobal( &IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     //
//     //  First acquire the FatData resource shared, then walk through all the
//     //  mounted VCBs looking for candidates to mark BAD.
//     //
//     //  On volumes we mark bad, check for dismount possibility (which is
//     //  why we have to get the next link early).
//     //
//
//     Links = FatData.VcbQueue.Flink;
//
//     while (Links != &FatData.VcbQueue) {
//
//         PVCB ExistingVcb;
//
//         ExistingVcb = CONTAINING_RECORD(Links, VCB, VcbLinks);
//
//         Links = Links->Flink;
//
//         //
//         //  If we get a match, mark the volume Bad, and also check to
//         //  see if the volume should go away.
//         //
//
//         if (ExistingVcb->Vpb->RealDevice == DeviceToMarkBad) {
//
//             BOOLEAN VcbDeleted = FALSE;
//
//             //
//             //  Here we acquire the Vcb exclusive and try to purge
//             //  all the open files.  The idea is to have as little as
//             //  possible stale data visible and to hasten the volume
//             //  going away.
//             //
//
//             (VOID)FatAcquireExclusiveVcb( &IrpContext, ExistingVcb );
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28175, "touching Vpb is ok for a filesystem" )
// #endif
//
//             if (ExistingVcb->Vpb == DeviceToMarkBad->Vpb) {
//
//                 KIRQL OldIrql;
//
//                 IoAcquireVpbSpinLock( &OldIrql );
//
//                 if (FlagOn( DeviceToMarkBad->Vpb->Flags, VPB_MOUNTED )) {
//
//                     PVPB NewVpb;
//
//                     NewVpb = ExistingVcb->SwapVpb;
//                     ExistingVcb->SwapVpb = NULL;
//                     SetFlag( ExistingVcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED );
//
//                     RtlZeroMemory( NewVpb, sizeof( VPB ) );
//                     NewVpb->Type = IO_TYPE_VPB;
//                     NewVpb->Size = sizeof( VPB );
//                     NewVpb->RealDevice = DeviceToMarkBad;
//                     NewVpb->Flags = FlagOn( DeviceToMarkBad->Vpb->Flags, VPB_REMOVE_PENDING );
//
//                     DeviceToMarkBad->Vpb = NewVpb;
//                 }
//
//                 NT_ASSERT( DeviceToMarkBad->Vpb->DeviceObject == NULL );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//                 IoReleaseVpbSpinLock( OldIrql );
//             }
//
//             FatSetVcbCondition( ExistingVcb, VcbBad );
//
//             //
//             //  Process the root directory, if it is present.
//             //
//
//             if (ExistingVcb->RootDcb != NULL) {
//
//                 //
//                 //  In order to safely mark all FCBs bad, we must acquire everything.
//                 //
//
//                 FatAcquireExclusiveVolume(&IrpContext, ExistingVcb);
//                 FatMarkFcbCondition( &IrpContext, ExistingVcb->RootDcb, FcbBad, TRUE );
//                 FatReleaseVolume(&IrpContext, ExistingVcb);
//
//                 //
//                 //  Purging the file objects on this volume could result in the memory manager
//                 //  dereferencing it's file pointer which could be the last reference and
//                 //  trigger object deletion and VCB deletion. Protect against that here by
//                 //  temporarily biasing the file count, and later checking for dismount.
//                 //
//
//                 ExistingVcb->OpenFileCount += 1;
//
//                 FatPurgeReferencedFileObjects( &IrpContext,
//                                                ExistingVcb->RootDcb,
//                                                NoFlush );
//
//                 ExistingVcb->OpenFileCount -= 1;
//
//                 VcbDeleted = FatCheckForDismount( &IrpContext, ExistingVcb, FALSE );
//             }
//
//             //
//             //  Only release the VCB if it did not go away.
//             //
//
//             if (!VcbDeleted) {
//
//                 FatReleaseVcb( &IrpContext, ExistingVcb );
//             }
//         }
//     }
//
//     FatReleaseGlobal( &IrpContext );
//
//     FatCompleteRequest( FatNull, Irp, STATUS_SUCCESS );
//
//     DebugTrace(-1, Dbg, "FatInvalidateVolumes -> STATUS_SUCCESS\n", 0);
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Local Support routine
// //
//
/* BOOLEAN */
// FatPerformVerifyDiskRead (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PVOID Buffer,
//     IN LBO Lbo,
//     IN ULONG NumberOfBytesToRead,
//     IN BOOLEAN ReturnOnError
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to read in a range of bytes from the disk.  It
//     bypasses all of the caching and regular I/O logic, and builds and issues
//     the requests itself.  It does this operation overriding the verify
//     volume flag in the device object.
//
// Arguments:
//
//     Vcb - Supplies the target device object for this operation.
//
//     Buffer - Supplies the buffer that will recieve the results of this operation
//
//     Lbo - Supplies the byte offset of where to start reading
//
//     NumberOfBytesToRead - Supplies the number of bytes to read, this must
//         be in multiple of bytes units acceptable to the disk driver.
//
//     ReturnOnError - Indicates that we should return on an error, instead
//         of raising.
//
// Return Value:
//
//     BOOLEAN - TRUE if the operation succeded, FALSE otherwise.
//
// --*/
//
// {
//     KEVENT Event;
//     PIRP Irp;
//     LARGE_INTEGER ByteOffset;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK Iosb;
//
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatPerformVerifyDiskRead, Lbo = %08lx\n", Lbo );
//
//     //
//     //  Initialize the event we're going to use
//     //
//
//     KeInitializeEvent( &Event, NotificationEvent, FALSE );
//
//     //
//     //  Build the irp for the operation and also set the overrride flag
//     //
//
//     ByteOffset.QuadPart = Lbo;
//
//     Irp = IoBuildSynchronousFsdRequest( IRP_MJ_READ,
//                                         Vcb->TargetDeviceObject,
//                                         Buffer,
//                                         NumberOfBytesToRead,
//                                         &ByteOffset,
//                                         &Event,
//                                         &Iosb );
//
//     if ( Irp == NULL ) {
//
//         FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//     }
//
//     SetFlag( IoGetNextIrpStackLocation( Irp )->Flags, SL_OVERRIDE_VERIFY_VOLUME );
//
//     //
//     //  Call the device to do the read and wait for it to finish.
//     //
//
//     Status = IoCallDriver( Vcb->TargetDeviceObject, Irp );
//
//     if (Status == STATUS_PENDING) {
//
//         (VOID)KeWaitForSingleObject( &Event, Executive, KernelMode, FALSE, (PLARGE_INTEGER)NULL );
//
//         Status = Iosb.Status;
//     }
//
//     NT_ASSERT( Status != STATUS_VERIFY_REQUIRED );
//
//     //
//     //  Special case this error code because this probably means we used
//     //  the wrong sector size and we want to reject STATUS_WRONG_VOLUME.
//     //
//
//     if (Status == STATUS_INVALID_PARAMETER) {
//
//         return FALSE;
//     }
//
//     //
//     //  If it doesn't succeed then either return or raise the error.
//     //
//
//     if (!NT_SUCCESS(Status)) {
//
//         if (ReturnOnError) {
//
//             return FALSE;
//
//         } else {
//
//             FatNormalizeAndRaiseStatus( IrpContext, Status );
//         }
//     }
//
//     //
//     //  And return to our caller
//     //
//
//     return TRUE;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatQueryRetrievalPointers (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the query retrieval pointers operation.
//     It returns the retrieval pointers for the specified input
//     file from the start of the file to the request map size specified
//     in the input buffer.
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
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PLARGE_INTEGER RequestedMapSize;
//     PLARGE_INTEGER *MappingPairs;
//
//     ULONG Index;
//     ULONG i;
//     ULONG SectorCount;
//     LBO Lbo;
//     ULONG Vbo;
//     ULONG MapSize;
//     BOOLEAN Result;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Make this a synchronous IRP because we need access to the input buffer and
//     //  this Irp is marked METHOD_NEITHER.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Decode the file object and ensure that it is the paging file
//     //
//     //  Only Kernel mode clients may query retrieval pointer information about
//     //  a file.  Ensure that this is the case for this caller.
//     //
//
//     (VOID)FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     if (Irp->RequestorMode != KernelMode ||
//         Fcb == NULL ||
//         !FlagOn(Fcb->FcbState, FCB_STATE_PAGING_FILE) ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Extract the input and output buffer information.  The input contains
//     //  the requested size of the mappings in terms of VBO.  The output
//     //  parameter will receive a pointer to nonpaged pool where the mapping
//     //  pairs are stored.
//     //
//
//     NT_ASSERT( IrpSp->Parameters.FileSystemControl.InputBufferLength == sizeof(LARGE_INTEGER) );
//     NT_ASSERT( IrpSp->Parameters.FileSystemControl.OutputBufferLength == sizeof(PVOID) );
//
//     RequestedMapSize = IrpSp->Parameters.FileSystemControl.Type3InputBuffer;
//     MappingPairs = Irp->UserBuffer;
//
//     //
//     //  Acquire exclusive access to the Fcb
//     //
//
//     FatAcquireExclusiveFcb( IrpContext, Fcb );
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb is still OK
//         //
//
//         FatVerifyFcb( IrpContext, Fcb );
//
//         //
//         //  Check if the mapping the caller requested is too large
//         //
//
//         if ((*RequestedMapSize).QuadPart > Fcb->Header.FileSize.QuadPart) {
//
//             try_leave( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         //
//         //  Now get the index for the mcb entry that will contain the
//         //  callers request and allocate enough pool to hold the
//         //  output mapping pairs. Mapping should always be present, but handle
//         //  the case where it isn't.
//         //
//
//         Result = FatLookupMcbEntry( Fcb->Vcb,
//                                     &Fcb->Mcb,
//                                     RequestedMapSize->LowPart - 1,
//                                     &Lbo,
//                                     NULL,
//                                     &Index );
//
//         if (!Result) {
//
//             NT_ASSERT(FALSE);
//             try_leave( Status = STATUS_FILE_CORRUPT_ERROR);
//         }
//
//         *MappingPairs = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                   (Index + 2) * (2 * sizeof(LARGE_INTEGER)),
//                                                   TAG_OUTPUT_MAPPINGPAIRS );
//
//         //
//         //  Now copy over the mapping pairs from the mcb
//         //  to the output buffer.  We store in [sector count, lbo]
//         //  mapping pairs and end with a zero sector count.
//         //
//
//         MapSize = RequestedMapSize->LowPart;
//
//         for (i = 0; i <= Index; i += 1) {
//
//             (VOID)FatGetNextMcbEntry( Fcb->Vcb, &Fcb->Mcb, i, (PVBO)&Vbo, &Lbo, &SectorCount );
//
//             if (SectorCount > MapSize) {
//                 SectorCount = MapSize;
//             }
//
//             (*MappingPairs)[ i*2 + 0 ].QuadPart = SectorCount;
//             (*MappingPairs)[ i*2 + 1 ].QuadPart = Lbo;
//
//             MapSize -= SectorCount;
//         }
//
//         (*MappingPairs)[ i*2 + 0 ].QuadPart = 0;
//
//         Status = STATUS_SUCCESS;
//     }
//     _SEH2_FINALLY {
//
//         DebugUnwind( FatQueryRetrievalPointers );
//
//         //
//         //  Release all of our resources
//         //
//
//         FatReleaseFcb( IrpContext, Fcb );
//
//         //
//         //  If this is an abnormal termination then undo our work, otherwise
//         //  complete the irp
//         //
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// NTSTATUS
// FatGetStatistics (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine returns the filesystem performance counters from the
//     appropriate VCB.
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
//     NTSTATUS Status;
//     PVCB Vcb;
//
//     PFILE_SYSTEM_STATISTICS Buffer;
//     ULONG BufferLength;
//     ULONG StatsSize;
//     ULONG BytesToCopy;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatGetStatistics...\n", 0);
//
//     //
//     // Extract the buffer
//     //
//
//     BufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//     //
//     //  Get a pointer to the output buffer.
//     //
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Make sure the buffer is big enough for at least the common part.
//     //
//
//     if (BufferLength < sizeof(FILESYSTEM_STATISTICS)) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//
//         DebugTrace(-1, Dbg, "FatGetStatistics -> %08lx\n", STATUS_BUFFER_TOO_SMALL );
//
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     //  Now see how many bytes we can copy.
//     //
//
//     StatsSize = sizeof(FILE_SYSTEM_STATISTICS) * FatData.NumberProcessors;
//
//     if (BufferLength < StatsSize) {
//
//         BytesToCopy = BufferLength;
//         Status = STATUS_BUFFER_OVERFLOW;
//
//     } else {
//
//         BytesToCopy = StatsSize;
//         Status =  STATUS_SUCCESS;
//     }
//
//     //
//     //  Get the Vcb.
//     //
//
//     Vcb = &((PVOLUME_DEVICE_OBJECT)IrpSp->DeviceObject)->Vcb;
//
//     //
//     //  Fill in the output buffer
//     //
//
//     RtlCopyMemory( Buffer, Vcb->Statistics, BytesToCopy );
//
//     Irp->IoStatus.Information = BytesToCopy;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatGetStatistics -> %08lx\n", Status);
//
//     return Status;
// }
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetVolumeBitmap(
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine returns the volume allocation bitmap.
//
//         Input = the STARTING_LCN_INPUT_BUFFER data structure is passed in
//             through the input buffer.
//         Output = the VOLUME_BITMAP_BUFFER data structure is returned through
//             the output buffer.
//
//     We return as much as the user buffer allows starting the specified input
//     LCN (trucated to a byte).  If there is no input buffer, we start at zero.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     ULONG BytesToCopy;
//     ULONG TotalClusters;
//     ULONG DesiredClusters;
//     ULONG StartingCluster;
//     ULONG EndingCluster;
//     ULONG InputBufferLength;
//     ULONG OutputBufferLength;
//     LARGE_INTEGER StartingLcn;
//     PVOLUME_BITMAP_BUFFER OutputBuffer;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Irp stack location and save some references.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatGetVolumeBitmap, FsControlCode = %08lx\n",
//                IrpSp->Parameters.FileSystemControl.FsControlCode);
//
//     //
//     //  Make this a synchronous IRP because we need access to the input buffer and
//     //  this Irp is marked METHOD_NEITHER.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatGetVolumeBitmap -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     InputBufferLength = IrpSp->Parameters.FileSystemControl.InputBufferLength;
//     OutputBufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//     OutputBuffer = (PVOLUME_BITMAP_BUFFER)FatMapUserBuffer( IrpContext, Irp );
//
//     //
//     //  Check for a minimum length on the input and output buffers.
//     //
//
//     if ((InputBufferLength < sizeof(STARTING_LCN_INPUT_BUFFER)) ||
//         (OutputBufferLength < sizeof(VOLUME_BITMAP_BUFFER))) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     //  Check if a starting cluster was specified.
//     //
//
//     TotalClusters = Vcb->AllocationSupport.NumberOfClusters;
//
//     //
//     //  Check for valid buffers
//     //
//
//     _SEH2_TRY {
//
//         if (Irp->RequestorMode != KernelMode) {
//
//             ProbeForRead( IrpSp->Parameters.FileSystemControl.Type3InputBuffer,
//                           InputBufferLength,
//                           sizeof(UCHAR) );
//
//             ProbeForWrite( OutputBuffer, OutputBufferLength, sizeof(UCHAR) );
//         }
//
//         StartingLcn = ((PSTARTING_LCN_INPUT_BUFFER)IrpSp->Parameters.FileSystemControl.Type3InputBuffer)->StartingLcn;
//
//     } _SEH2_EXCEPT( Irp->RequestorMode != KernelMode ? EXCEPTION_EXECUTE_HANDLER: EXCEPTION_CONTINUE_SEARCH ) {
//
//           Status = _SEH2_GetExceptionCode();
//
//           FatRaiseStatus( IrpContext,
//                           FsRtlIsNtstatusExpected(Status) ?
//                           Status : STATUS_INVALID_USER_BUFFER );
//     } _SEH2_END;
//
//     if (StartingLcn.HighPart || StartingLcn.LowPart >= TotalClusters) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//
//     } else {
//
//         StartingCluster = StartingLcn.LowPart & ~7;
//     }
//
//     (VOID)FatAcquireExclusiveVcb( IrpContext, Vcb );
//
//     //
//     //  Only return what will fit in the user buffer.
//     //
//
//     OutputBufferLength -= FIELD_OFFSET(VOLUME_BITMAP_BUFFER, Buffer);
//     DesiredClusters = TotalClusters - StartingCluster;
//
//     if (OutputBufferLength < (DesiredClusters + 7) / 8) {
//
//         BytesToCopy = OutputBufferLength;
//         Status = STATUS_BUFFER_OVERFLOW;
//
//     } else {
//
//         BytesToCopy = (DesiredClusters + 7) / 8;
//         Status = STATUS_SUCCESS;
//     }
//
//     //
//     //  Use try/finally for cleanup.
//     //
//
//     _SEH2_TRY {
//
//         _SEH2_TRY {
//
//             //
//             //  Verify the Vcb is still OK
//             //
//
//             FatQuickVerifyVcb( IrpContext, Vcb );
//
//             //
//             //  Fill in the fixed part of the output buffer
//             //
//
//             OutputBuffer->StartingLcn.QuadPart = StartingCluster;
//             OutputBuffer->BitmapSize.QuadPart = DesiredClusters;
//
//             if (Vcb->NumberOfWindows == 1) {
//
//                 //
//                 //  Just copy the volume bitmap into the user buffer.
//                 //
//
//                 NT_ASSERT( Vcb->FreeClusterBitMap.Buffer != NULL );
//
//                 RtlCopyMemory( &OutputBuffer->Buffer[0],
//                                (PUCHAR)Vcb->FreeClusterBitMap.Buffer + StartingCluster/8,
//                                BytesToCopy );
//             } else {
//
//                 //
//                 //  Call out to analyze the FAT.  We must bias by two to account for
//                 //  the zero base of this API and FAT's physical reality of starting
//                 //  the file heap at cluster 2.
//                 //
//                 //  Note that the end index is inclusive - we need to subtract one to
//                 //  calculcate it.
//                 //
//                 //  I.e.: StartingCluster 0 for one byte of bitmap means a start cluster
//                 //  of 2 and end cluster of 9, a run of eight clusters.
//                 //
//
//                 EndingCluster = StartingCluster + (BytesToCopy * 8);
//
//                 //
//                 //  Make sure we do not read past the end of the entries.
//                 //
//
//                 if (EndingCluster > TotalClusters) {
//
//                     EndingCluster = TotalClusters;
//                 }
//
//                 FatExamineFatEntries( IrpContext,
//                                       Vcb,
//                                       StartingCluster + 2,
//                                       EndingCluster + 2 - 1,
//                                       FALSE,
//                                       NULL,
//                                       (PULONG)&OutputBuffer->Buffer[0] );
//             }
//
//         } _SEH2_EXCEPT( Irp->RequestorMode != KernelMode ? EXCEPTION_EXECUTE_HANDLER: EXCEPTION_CONTINUE_SEARCH ) {
//
//             Status = _SEH2_GetExceptionCode();
//
//             FatRaiseStatus( IrpContext,
//                             FsRtlIsNtstatusExpected(Status) ?
//                             Status : STATUS_INVALID_USER_BUFFER );
//         } _SEH2_END;
//
//     } _SEH2_FINALLY {
//
//         FatReleaseVcb( IrpContext, Vcb );
//     } _SEH2_END;
//
//     Irp->IoStatus.Information = FIELD_OFFSET(VOLUME_BITMAP_BUFFER, Buffer) +
//                                 BytesToCopy;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     DebugTrace(-1, Dbg, "FatGetVolumeBitmap -> VOID\n", 0);
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetRetrievalPointers (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine scans the MCB and builds an extent list.  The first run in
//     the output extent list will start at the begining of the contiguous
//     run specified by the input parameter.
//
//         Input = STARTING_VCN_INPUT_BUFFER;
//         Output = RETRIEVAL_POINTERS_BUFFER.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB FcbOrDcb;
//     PCCB Ccb;
//     PLARGE_MCB McbToUse = NULL;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     ULONG Index;
//     ULONG ClusterShift = 0;
//     ULONG ClusterSize;
//     LONGLONG AllocationSize = 0;
//
//     ULONG Run;
//     ULONG RunCount;
//     ULONG StartingRun;
//     LARGE_INTEGER StartingVcn;
//
//     ULONG InputBufferLength;
//     ULONG OutputBufferLength;
//
//     VBO   LastVbo;
//     LBO   LastLbo;
//     ULONG LastIndex;
//
//     PRETRIEVAL_POINTERS_BUFFER OutputBuffer;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Irp stack location and save some references.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatGetRetrievalPointers, FsControlCode = %08lx\n",
//                IrpSp->Parameters.FileSystemControl.FsControlCode);
//
//     //
//     //  Make this a synchronous IRP because we need access to the input buffer and
//     //  this Irp is marked METHOD_NEITHER.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &FcbOrDcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen) && (TypeOfOpen != UserDirectoryOpen) && (TypeOfOpen != UserVolumeOpen) ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Get the input and output buffer lengths and pointers.
//     //  Initialize some variables.
//     //
//
//     InputBufferLength = IrpSp->Parameters.FileSystemControl.InputBufferLength;
//     OutputBufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//     OutputBuffer = (PRETRIEVAL_POINTERS_BUFFER)FatMapUserBuffer( IrpContext, Irp );
//
//     //
//     //  Check for a minimum length on the input and ouput buffers.
//     //
//
//     if ((InputBufferLength < sizeof(STARTING_VCN_INPUT_BUFFER)) ||
//         (OutputBufferLength < sizeof(RETRIEVAL_POINTERS_BUFFER))) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     //  Acquire the Fcb and enqueue the Irp if we didn't get access.  Go for
//     //  shared on read-only media so we can allow prototype XIP to get
//     //  recursive, as well as recognizing this is safe anyway.
//     //
//     if( (TypeOfOpen == UserFileOpen) || (TypeOfOpen == UserDirectoryOpen) ) {
//
//         if (FlagOn( FcbOrDcb->Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED )) {
//
//             (VOID)FatAcquireSharedFcb( IrpContext, FcbOrDcb );
//
//         } else {
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, FcbOrDcb );
//         }
//     } else if ((TypeOfOpen == UserVolumeOpen )) {
//
//         if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_ACCESS_DENIED );
//
//             DebugTrace(-1, Dbg, "FatMoveFile -> 0x%x\n", STATUS_ACCESS_DENIED);
//             return STATUS_ACCESS_DENIED;
//         }
//
//         (VOID)FatAcquireExclusiveVcb(IrpContext, Vcb);
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Verify the Fcb is still OK, or if it is a volume handle, the VCB.
//         //
//
//         if( (TypeOfOpen == UserFileOpen) || (TypeOfOpen == UserDirectoryOpen) ) {
//
//             FatVerifyFcb( IrpContext, FcbOrDcb );
//
//             //
//             //  If we haven't yet set the correct AllocationSize, do so.
//             //
//
//             if (FcbOrDcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//                 FatLookupFileAllocationSize( IrpContext, FcbOrDcb );
//
//                 //
//                 //  If this is a non-root directory, we have a bit more to
//                 //  do since it has not gone through FatOpenDirectoryFile().
//                 //
//
//                 if (NodeType(FcbOrDcb) == FAT_NTC_DCB ||
//                     (NodeType(FcbOrDcb) == FAT_NTC_ROOT_DCB && FatIsFat32(Vcb))) {
//
//                     FcbOrDcb->Header.FileSize.LowPart =
//                         FcbOrDcb->Header.AllocationSize.LowPart;
//                 }
//             }
//
//
//             ClusterShift = Vcb->AllocationSupport.LogOfBytesPerCluster;
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "calculate it anyway, in case someone adds code that uses this in the future" )
// #endif
//             ClusterSize = 1 << ClusterShift;
//
//             AllocationSize = FcbOrDcb->Header.AllocationSize.LowPart;
//             McbToUse = &FcbOrDcb->Mcb;
//
//         } else if ((TypeOfOpen == UserVolumeOpen )) {
//
//             FatQuickVerifyVcb( IrpContext, Vcb );
//
//             if (!FlagOn( Vcb->VcbState, VCB_STATE_FLAG_BAD_BLOCKS_POPULATED)) {
//
//                 //
//                 //  If the bad cluster mcb isn't populated, something is wrong. (It should have been
//                 //  populated during mount when we scanned the FAT.
//                 //
//
//                 FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             ClusterShift = Vcb->AllocationSupport.LogOfBytesPerCluster;
//             ClusterSize = 1 << ClusterShift;
//
//             if (!FatLookupLastMcbEntry(Vcb, &Vcb->BadBlockMcb, &LastVbo, &LastLbo, &LastIndex)) {
//                 AllocationSize = 0;
//             } else {
//
//                 //
//                 //  Round the allocation size to a multiple of of the cluster size.
//                 //
//
//                 AllocationSize = (LastVbo + ((LONGLONG)ClusterSize-1)) & ~((LONGLONG)ClusterSize-1);
//             }
//
//             McbToUse = &Vcb->BadBlockMcb;
//
//         }
//
//         //
//         //  Check if a starting cluster was specified.
//         //
//
//         _SEH2_TRY {
//
//             if (Irp->RequestorMode != KernelMode) {
//
//                 ProbeForRead( IrpSp->Parameters.FileSystemControl.Type3InputBuffer,
//                               InputBufferLength,
//                               sizeof(UCHAR) );
//
//                 ProbeForWrite( OutputBuffer, OutputBufferLength, sizeof(UCHAR) );
//             }
//
//             StartingVcn = ((PSTARTING_VCN_INPUT_BUFFER)IrpSp->Parameters.FileSystemControl.Type3InputBuffer)->StartingVcn;
//
//         } _SEH2_EXCEPT( Irp->RequestorMode != KernelMode ? EXCEPTION_EXECUTE_HANDLER: EXCEPTION_CONTINUE_SEARCH ) {
//
//               Status = _SEH2_GetExceptionCode();
//
//               FatRaiseStatus( IrpContext,
//                               FsRtlIsNtstatusExpected(Status) ?
//                               Status : STATUS_INVALID_USER_BUFFER );
//         } _SEH2_END;
//
//         if (StartingVcn.HighPart ||
//             StartingVcn.LowPart >= (AllocationSize >> ClusterShift)) {
//
//             try_return( Status = STATUS_END_OF_FILE );
//
//         } else {
//
//             //
//             //  If we don't find the run, something is very wrong.
//             //
//
//             LBO Lbo;
//
//             if (!FatLookupMcbEntry( Vcb, McbToUse,
//                                     StartingVcn.LowPart << ClusterShift,
//                                     &Lbo,
//                                     NULL,
//                                     &StartingRun)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck( (ULONG_PTR)FcbOrDcb, (ULONG_PTR)McbToUse, StartingVcn.LowPart );
//             }
//         }
//
//         //
//         //  Now go fill in the ouput buffer with run information
//         //
//
//         RunCount = FsRtlNumberOfRunsInLargeMcb( McbToUse );
//
//         for (Index = 0, Run = StartingRun; Run < RunCount; Index++, Run++) {
//
//             ULONG Vcn;
//             LBO Lbo;
//             ULONG ByteLength;
//
//             //
//             //  Check for an exhausted output buffer.
//             //
//
//             if ((ULONG)FIELD_OFFSET(RETRIEVAL_POINTERS_BUFFER, Extents[Index+1]) > OutputBufferLength) {
//
//
//                 //
//                 //  We've run out of space, so we won't be storing as many runs to the
//                 //  user's buffer as we had originally planned.  We need to return the
//                 //  number of runs that we did have room for.
//                 //
//
//                 _SEH2_TRY {
//
//                     OutputBuffer->ExtentCount = Index;
//
//                 } _SEH2_EXCEPT( Irp->RequestorMode != KernelMode ? EXCEPTION_EXECUTE_HANDLER: EXCEPTION_CONTINUE_SEARCH ) {
//
//                     Status = _SEH2_GetExceptionCode();
//
//                     FatRaiseStatus( IrpContext,
//                                     FsRtlIsNtstatusExpected(Status) ?
//                                     Status : STATUS_INVALID_USER_BUFFER );
//                 } _SEH2_END;
//
//                 Irp->IoStatus.Information = FIELD_OFFSET(RETRIEVAL_POINTERS_BUFFER, Extents[Index]);
//                 try_return( Status = STATUS_BUFFER_OVERFLOW );
//             }
//
//             //
//             //  Get the extent.  If it's not there or malformed, something is very wrong.
//             //
//
//             if (!FatGetNextMcbEntry(Vcb, McbToUse, Run, (PVBO)&Vcn, &Lbo, &ByteLength)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//                 FatBugCheck( (ULONG_PTR)FcbOrDcb, (ULONG_PTR)McbToUse, Run );
//             }
//
//             //
//             //  Fill in the next array element.
//             //
//
//             _SEH2_TRY {
//
//                 OutputBuffer->Extents[Index].NextVcn.QuadPart = ((LONGLONG)Vcn + ByteLength) >> ClusterShift;
//                 OutputBuffer->Extents[Index].Lcn.QuadPart = FatGetIndexFromLbo( Vcb, Lbo ) - 2;
//
//                 //
//                 //  If this is the first run, fill in the starting Vcn
//                 //
//
//                 if (Index == 0) {
//                     OutputBuffer->ExtentCount = RunCount - StartingRun;
//                     OutputBuffer->StartingVcn.QuadPart = Vcn >> ClusterShift;
//                 }
//
//             } _SEH2_EXCEPT( Irp->RequestorMode != KernelMode ? EXCEPTION_EXECUTE_HANDLER: EXCEPTION_CONTINUE_SEARCH ) {
//
//                 Status = _SEH2_GetExceptionCode();
//
//                 FatRaiseStatus( IrpContext,
//                                 FsRtlIsNtstatusExpected(Status) ?
//                                 Status : STATUS_INVALID_USER_BUFFER );
//             } _SEH2_END;
//         }
//
//         //
//         //  We successfully retrieved extent info to the end of the allocation.
//         //
//
//         Irp->IoStatus.Information = FIELD_OFFSET(RETRIEVAL_POINTERS_BUFFER, Extents[Index]);
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatGetRetrievalPointers );
//
//         //
//         //  Release resources
//         //
//
//         if( (TypeOfOpen == UserFileOpen) || (TypeOfOpen == UserDirectoryOpen) ) {
//
//             FatReleaseFcb( IrpContext, FcbOrDcb );
//         } else if ((TypeOfOpen == UserVolumeOpen )) {
//
//             FatReleaseVcb(IrpContext, Vcb);
//         }
//
//         //
//         //  If nothing raised then complete the irp.
//         //
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatGetRetrievalPointers -> VOID\n", 0);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatMoveFileNeedsWriteThrough (
//      PIRP_CONTEXT IrpContext,
//      PFCB FcbOrDcb,
//      ULONG OldWriteThroughFlags
//     )
// {
//     PAGED_CODE();
//
//     if (NodeType(FcbOrDcb) == FAT_NTC_FCB) {
//
//
//         if (FcbOrDcb->Header.ValidDataLength.QuadPart == 0) {
//
//
//             ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH );
//             SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH );
//
//         } else {
//
//             IrpContext->Flags &= ~(IRP_CONTEXT_FLAG_WRITE_THROUGH|IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH);
//             IrpContext->Flags |= OldWriteThroughFlags;
//
//         }
//     }
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMoveFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     Routine moves a file to the requested Starting Lcn from Starting Vcn for the length
//     of cluster count. These values are passed in through the the input buffer as a
//     MOVE_DATA structure.
//
//     The call must be made with a DASD handle.  The file to move is passed in as a
//     parameter.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
//
// {
//     NTSTATUS Status;
//     PIO_STACK_LOCATION IrpSp;
//
//     PFILE_OBJECT FileObject;
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB FcbOrDcb;
//     PCCB Ccb;
//
//     ULONG InputBufferLength;
//     PMOVE_FILE_DATA InputBuffer;
//
//     ULONG ClusterShift;
//     ULONG MaxClusters;
//
//     ULONG FileOffset;
//
//     LBO TargetLbo;
//     ULONG TargetCluster;
//     LARGE_INTEGER LargeSourceLbo;
//     LARGE_INTEGER LargeTargetLbo;
//
//     ULONG ByteCount;
//     ULONG BytesToWrite;
//     ULONG BytesToReallocate;
//
//     ULONG FirstSpliceSourceCluster;
//     ULONG FirstSpliceTargetCluster;
//     ULONG SecondSpliceSourceCluster;
//     ULONG SecondSpliceTargetCluster;
//
//     LARGE_MCB SourceMcb;
//     LARGE_MCB TargetMcb;
//
//     KEVENT StackEvent;
//
//     PVOID Buffer = NULL;
//     ULONG BufferSize;
//
//     BOOLEAN SourceMcbInitialized = FALSE;
//     BOOLEAN TargetMcbInitialized = FALSE;
//
//     BOOLEAN FcbAcquired = FALSE;
//     BOOLEAN EventArmed = FALSE;
//     BOOLEAN DiskSpaceAllocated = FALSE;
//
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//
//     ULONG OldWriteThroughFlags = (IrpContext->Flags & (IRP_CONTEXT_FLAG_WRITE_THROUGH|IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH));
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     MOVE_FILE_DATA LocalMoveFileData;
//     PMOVE_FILE_DATA32 MoveFileData32;
// #endif
//
//     ULONG LocalAbnormalTermination = 0;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Irp stack location and save some references.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatMoveFile, FsControlCode = %08lx\n",
//                IrpSp->Parameters.FileSystemControl.FsControlCode);
//
//     //
//     //  Force WAIT to true.  We have a handle in the input buffer which can only
//     //  be referenced within the originating process.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &FcbOrDcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     InputBufferLength = IrpSp->Parameters.FileSystemControl.InputBufferLength;
//     InputBuffer = (PMOVE_FILE_DATA)Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Do a quick check on the input buffer.
//     //
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     if (IoIs32bitProcess( Irp )) {
//
//         if (InputBuffer == NULL || InputBufferLength < sizeof(MOVE_FILE_DATA32)) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         MoveFileData32 = (PMOVE_FILE_DATA32) InputBuffer;
//
//         LocalMoveFileData.FileHandle = (HANDLE) LongToHandle( MoveFileData32->FileHandle );
//         LocalMoveFileData.StartingVcn = MoveFileData32->StartingVcn;
//         LocalMoveFileData.StartingLcn = MoveFileData32->StartingLcn;
//         LocalMoveFileData.ClusterCount = MoveFileData32->ClusterCount;
//
//         InputBuffer = &LocalMoveFileData;
//
//     } else {
// #endif
//         if (InputBuffer == NULL || InputBufferLength < sizeof(MOVE_FILE_DATA)) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//             return STATUS_BUFFER_TOO_SMALL;
//         }
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     }
// #endif
//
//     MaxClusters = Vcb->AllocationSupport.NumberOfClusters;
//     TargetCluster = InputBuffer->StartingLcn.LowPart + 2;
//
//     if (InputBuffer->StartingVcn.HighPart ||
//         InputBuffer->StartingLcn.HighPart ||
//         (TargetCluster < 2) ||
//         (TargetCluster + InputBuffer->ClusterCount < TargetCluster) ||
//         (TargetCluster + InputBuffer->ClusterCount > MaxClusters + 2) ||
//         (InputBuffer->StartingVcn.LowPart >= MaxClusters) ||
//         InputBuffer->ClusterCount == 0
//         ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Try to get a pointer to the file object from the handle passed in.
//     //
//
//     Status = ObReferenceObjectByHandle( InputBuffer->FileHandle,
//                                         0,
//                                         *IoFileObjectType,
//                                         Irp->RequestorMode,
// #ifndef __REACTOS__
//                                         &FileObject,
// #else
//                                         (PVOID *)&FileObject,
// #endif
//                                         NULL );
//
//     if (!NT_SUCCESS(Status)) {
//
//         FatCompleteRequest( IrpContext, Irp, Status );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", Status);
//         return Status;
//     }
//
//     //
//     //  There are three basic ways this could be an invalid attempt, so
//     //  we need to
//     //
//     //    - check that this file object is opened on the same volume as the
//     //      DASD handle used to call this routine.
//     //
//     //    - extract and decode the file object and check for type of open.
//     //
//     //    - if this is a directory, verify that it's not the root and that
//     //      we are not trying to move the first cluster.  We cannot move the
//     //      first cluster because sub-directories have this cluster number
//     //      in them and there is no safe way to simultaneously update them
//     //      all.
//     //
//     //  We'll allow movefile on the root dir if its fat32, since the root dir
//     //  is a real chained file there.
//     //
//
//     if (FileObject->Vpb != Vcb->Vpb) {
//
//         ObDereferenceObject( FileObject );
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     TypeOfOpen = FatDecodeFileObject( FileObject, &Vcb, &FcbOrDcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen &&
//          TypeOfOpen != UserDirectoryOpen) ||
//
//         ((TypeOfOpen == UserDirectoryOpen) &&
//          ((NodeType(FcbOrDcb) == FAT_NTC_ROOT_DCB && !FatIsFat32(Vcb)) ||
//           (InputBuffer->StartingVcn.QuadPart == 0)))) {
//
//         ObDereferenceObject( FileObject );
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  If the VDL of the file is zero, it has no valid data in it anyway.
//     //  So it should be safe to avoid flushing the FAT entries and let them be
//     //  lazily written out.
//     //
//     //  This is  done so that bitlocker's cover file doesn't cause
//     //  unnecessary FAT table I/O when it's moved around.
//     //  (See Win8 bug 106505)
//     //
//
//     //
//     //  If this is a file, and the VDL is zero, clear write through.
//     //
//
//     FatMoveFileNeedsWriteThrough(IrpContext, FcbOrDcb, OldWriteThroughFlags);
//
//
//     //
//     //  Indicate we're getting to parents of this fcb by their child, and that
//     //  this is a sufficient assertion of our ability to by synchronized
//     //  with respect to the parent directory going away.
//     //
//     //  The defrag path is an example of one which arrives at an Fcb by
//     //  a means which would be unreasonable to duplicate in the assertion
//     //  code. See FatOpenDirectoryFile.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_PARENT_BY_CHILD );
//
//     ClusterShift = Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     _SEH2_TRY {
//
//         //
//         //  Initialize our state variables and the event.
//         //
//
//         FileOffset = InputBuffer->StartingVcn.LowPart << ClusterShift;
//
//         ByteCount = InputBuffer->ClusterCount << ClusterShift;
//
//         TargetLbo = FatGetLboFromIndex( Vcb, TargetCluster );
//         LargeTargetLbo.QuadPart = TargetLbo;
//
//         Buffer = NULL;
//
//         //
//         //  Do a quick check on parameters here
//         //
//
//         if (FileOffset + ByteCount < FileOffset) {
//
//             try_return( Status = STATUS_INVALID_PARAMETER );
//         }
//
//         KeInitializeEvent( &StackEvent, NotificationEvent, FALSE );
//
//         //
//         //  Initialize two MCBs we will be using
//         //
//
//         FsRtlInitializeLargeMcb( &SourceMcb, PagedPool );
//         SourceMcbInitialized = TRUE;
//
//         FsRtlInitializeLargeMcb( &TargetMcb, PagedPool );
//         TargetMcbInitialized = TRUE;
//
//         //
//         //  Ok, now if this is a directory open we need to switch to the internal
//         //  stream fileobject since it is set up for caching.  The top-level
//         //  fileobject has no section object pointers in order prevent folks from
//         //  mapping it.
//         //
//
//         if (TypeOfOpen == UserDirectoryOpen) {
//
//             PFILE_OBJECT DirStreamFileObject;
//
//             //
//             //  Open the stream fileobject if neccesary.  We must acquire the Fcb
//             //  now to synchronize with other operations (such as dismount ripping
//             //  apart the allocator).
//             //
//
//             (VOID)FatAcquireExclusiveFcb( IrpContext, FcbOrDcb );
//             FcbAcquired = TRUE;
//
//             FatVerifyFcb( IrpContext, FcbOrDcb );
//
//             FatOpenDirectoryFile( IrpContext, FcbOrDcb );
//             DirStreamFileObject = FcbOrDcb->Specific.Dcb.DirectoryFile;
//
//             //
//             //  Transfer our reference to the internal stream and proceed.  Note that
//             //  if we dereferenced first, the user could sneak a teardown through since
//             //  we'd have no references.
//             //
//
//             ObReferenceObject( DirStreamFileObject );
//             ObDereferenceObject( FileObject );
//             FileObject = DirStreamFileObject;
//
//             //
//             //  We've referenced the DirStreamFileObject, so it should be ok to drop
//             //  the Dcb now.
//             //
//
//             FatReleaseFcb( IrpContext, FcbOrDcb );
//             FcbAcquired = FALSE;
//         }
//
//         //
//         //  Determine the size of the buffer we will use to move data.
//         //
//
//         BufferSize = FAT_DEFAULT_DEFRAG_CHUNK_IN_BYTES;
//
//         if (BufferSize < (ULONG)(1 << ClusterShift)) {
//
//             BufferSize = (1 << ClusterShift);
//         }
//
//         while (ByteCount) {
//
//             VBO TempVbo;
//             LBO TempLbo;
//             ULONG TempByteCount;
//
//             //
//             //  We must throttle our writes.
//             //
//
//             CcCanIWrite( FileObject,
//                          BufferSize,
//                          TRUE,
//                          FALSE );
//
//             //
//             //  Aqcuire file resource exclusive to freeze FileSize and block
//             //  user non-cached I/O.  Verify the integrity of the fcb - the
//             //  media may have changed (or been dismounted) on us.
//             //
//
//             if (FcbAcquired == FALSE) {
//
//                 (VOID)FatAcquireExclusiveFcb( IrpContext, FcbOrDcb );
//                 FcbAcquired = TRUE;
//
//                 FatVerifyFcb( IrpContext, FcbOrDcb );
//             }
//
//             //
//             //  Check if the handle indicates we're allowed to move the file.
//             //
//             //  FCB_STATE_DENY_DEFRAG indicates that someone blocked move file on this FCB.
//             //  CCB_FLAG_DENY_DEFRAG indicates that this handle was the one that blocked move file, and hence
//             //  it still gets to move the file around.
//             //
//
//             if ((FcbOrDcb->FcbState & FCB_STATE_DENY_DEFRAG) && !(Ccb->Flags & CCB_FLAG_DENY_DEFRAG)) {
//                 DebugTrace(-1, Dbg, "FatMoveFile -> %08lx\n", STATUS_ACCESS_DENIED);
//                 try_return( Status = STATUS_ACCESS_DENIED );
//             }
//
//             //
//             //  Allocate our buffer, if we need to.
//             //
//
//             if (Buffer == NULL) {
//
//                 Buffer = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                    BufferSize,
//                                                    TAG_DEFRAG_BUFFER );
//             }
//
//             //
//             //  Analyzes the range of file allocation we are moving
//             //  and determines the actual amount of allocation to be
//             //  moved and how much needs to be written.  In addition
//             //  it guarantees that the Mcb in the file is large enough
//             //  so that later MCB operations cannot fail.
//             //
//
//             FatComputeMoveFileParameter( IrpContext,
//                                          FcbOrDcb,
//                                          BufferSize,
//                                          FileOffset,
//                                          &ByteCount,
//                                          &BytesToReallocate,
//                                          &BytesToWrite,
//                                          &LargeSourceLbo );
//
//             //
//             //  If ByteCount comes back zero, break here.
//             //
//
//             if (ByteCount == 0) {
//                 break;
//             }
//
//             //
//             //  At this point (before actually doing anything with the disk
//             //  meta data), calculate the FAT splice clusters and build an
//             //  MCB describing the space to be deallocated.
//             //
//
//             FatComputeMoveFileSplicePoints( IrpContext,
//                                             FcbOrDcb,
//                                             FileOffset,
//                                             TargetCluster,
//                                             BytesToReallocate,
//                                             &FirstSpliceSourceCluster,
//                                             &FirstSpliceTargetCluster,
//                                             &SecondSpliceSourceCluster,
//                                             &SecondSpliceTargetCluster,
//                                             &SourceMcb );
//
//             //
//             //  Now attempt to allocate the new disk storage using the
//             //  Target Lcn as a hint.
//             //
//
//             TempByteCount = BytesToReallocate;
//             FatAllocateDiskSpace( IrpContext,
//                                   Vcb,
//                                   TargetCluster,
//                                   &TempByteCount,
//                                   TRUE,
//                                   &TargetMcb );
//
//             DiskSpaceAllocated = TRUE;
//
//             //
//             //  If we didn't get EXACTLY what we wanted, return immediately.
//             //
//
//             if ((FsRtlNumberOfRunsInLargeMcb( &TargetMcb ) != 1) ||
//                 !FatGetNextMcbEntry( Vcb, &TargetMcb, 0, &TempVbo, &TempLbo, &TempByteCount ) ||
//                 (FatGetIndexFromLbo( Vcb, TempLbo) != TargetCluster ) ||
//                 (TempByteCount != BytesToReallocate)) {
//
//                 //
//                 //  It would be nice if we could be more specific, but such is life.
//                 //
//                 try_return( Status = STATUS_INVALID_PARAMETER );
//             }
//
// #if DBG
//             //
//             //  We are going to attempt a move, note it.
//             //
//
//             if (FatMoveFileDebug) {
//                 DbgPrint("0x%p: Vcn 0x%lx, Lcn 0x%lx, Count 0x%lx.\n",
//                          PsGetCurrentThread(),
//                          FileOffset >> ClusterShift,
//                          TargetCluster,
//                          BytesToReallocate >> ClusterShift );
//             }
// #endif
//
//             //
//             //  Now attempt to commit the new allocation to disk.  If this
//             //  raises, the allocation will be deallocated.
//             //
//             //  If the VDL of the file is zero, it has no valid data in it anyway.
//             //  So it should be safe to avoid flushing the FAT entries and let them be
//             //  lazily written out.
//             //
//             //  This is  done so that bitlocker's cover file doesn't cause
//             //  unnecessary FAT table I/O when it's moved around.
//             //  (See Win8 bug 106505)
//             //
//
//             if ((FcbOrDcb->Header.ValidDataLength.QuadPart != 0) || (NodeType(FcbOrDcb) != FAT_NTC_FCB)) {
//
//                 FatFlushFatEntries( IrpContext,
//                                     Vcb,
//                                     TargetCluster,
//                                     BytesToReallocate >> ClusterShift );
//             }
//
//             //
//             //  Aqcuire both resources exclusive now, guaranteeing that NOBODY
//             //  is in either the read or write paths.
//             //
//
//             ExAcquireResourceExclusiveLite( FcbOrDcb->Header.PagingIoResource, TRUE );
//
//             //
//             //  This is the first part of some tricky synchronization.
//             //
//             //  Set the Event pointer in the FCB.  Any paging I/O will block on
//             //  this event (if set in FCB) after acquiring the PagingIo resource.
//             //
//             //  This is how I keep ALL I/O out of this path without holding the
//             //  PagingIo resource exclusive for an extended time.
//             //
//
//             FcbOrDcb->MoveFileEvent = &StackEvent;
//             EventArmed = TRUE;
//
//             ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
//
//             //
//             //  Now write out the data, but only if we have to.  We don't have
//             //  to copy any file data if the range being reallocated is wholly
//             //  beyond valid data length.
//             //
//
//             if (BytesToWrite) {
//
//                 PIRP IoIrp;
//                 KEVENT IoEvent;
//                 IO_STATUS_BLOCK Iosb;
//
//                 KeInitializeEvent( &IoEvent,
//                                    NotificationEvent,
//                                    FALSE );
//
//                 NT_ASSERT( LargeTargetLbo.QuadPart >= Vcb->AllocationSupport.FileAreaLbo );
//
//                 //
//                 //  Read in the data that is being moved.
//                 //
//
//                 IoIrp = IoBuildSynchronousFsdRequest( IRP_MJ_READ,
//                                                       Vcb->TargetDeviceObject,
//                                                       Buffer,
//                                                       BytesToWrite,
//                                                       &LargeSourceLbo,
//                                                       &IoEvent,
//                                                       &Iosb );
//
//                 if (IoIrp == NULL) {
//
//                     FatRaiseStatus( IrpContext,
//                                     STATUS_INSUFFICIENT_RESOURCES );
//                 }
//
//                 Status = IoCallDriver( Vcb->TargetDeviceObject, IoIrp );
//
//                 if (Status == STATUS_PENDING) {
//
//                     (VOID)KeWaitForSingleObject( &IoEvent,
//                                                  Executive,
//                                                  KernelMode,
//                                                  FALSE,
//                                                  (PLARGE_INTEGER)NULL );
//
//                     Status = Iosb.Status;
//                 }
//
//                 if (!NT_SUCCESS( Status )) {
//
//                     FatNormalizeAndRaiseStatus( IrpContext,
//                                                 Status );
//                 }
//
//                 //
//                 //  Write the data to its new location.
//                 //
//
//                 KeClearEvent( &IoEvent );
//
//                 IoIrp = IoBuildSynchronousFsdRequest( IRP_MJ_WRITE,
//                                                       Vcb->TargetDeviceObject,
//                                                       Buffer,
//                                                       BytesToWrite,
//                                                       &LargeTargetLbo,
//                                                       &IoEvent,
//                                                       &Iosb );
//
//                 if (IoIrp == NULL) {
//
//                     FatRaiseStatus( IrpContext,
//                                     STATUS_INSUFFICIENT_RESOURCES );
//                 }
//
//                 //
//                 //  Set a flag indicating that we want to write through any
//                 //  cache on the controller.  This eliminates the need for
//                 //  an explicit flush-device after the write.
//                 //
//
//                 SetFlag( IoGetNextIrpStackLocation(IoIrp)->Flags, SL_WRITE_THROUGH );
//
//                 Status = IoCallDriver( Vcb->TargetDeviceObject, IoIrp );
//
//                 if (Status == STATUS_PENDING) {
//
//                     (VOID)KeWaitForSingleObject( &IoEvent,
//                                                  Executive,
//                                                  KernelMode,
//                                                  FALSE,
//                                                  (PLARGE_INTEGER)NULL );
//
//                     Status = Iosb.Status;
//                 }
//
//                 if (!NT_SUCCESS( Status )) {
//
//                     FatNormalizeAndRaiseStatus( IrpContext,
//                                                 Status );
//                 }
//             }
//
//             //
//             //  Now that the file data has been moved successfully, we'll go
//             //  to fix up the links in the FAT table and perhaps change the
//             //  entry in the parent directory.
//             //
//             //  First we'll do the second splice and commit it.  At that point,
//             //  while the volume is in an inconsistent state, the file is
//             //  still OK.
//             //
//
//             FatSetFatEntry( IrpContext,
//                             Vcb,
//                             SecondSpliceSourceCluster,
//                             (FAT_ENTRY)SecondSpliceTargetCluster );
//
//             if ((FcbOrDcb->Header.ValidDataLength.QuadPart != 0) || (NodeType(FcbOrDcb) != FAT_NTC_FCB)) {
//
//                 FatFlushFatEntries( IrpContext, Vcb, SecondSpliceSourceCluster, 1 );
//             }
//
//             //
//             //  Now do the first splice OR update the dirent in the parent
//             //  and flush the respective object.  After this flush the file
//             //  now points to the new allocation.
//             //
//
//             if (FirstSpliceSourceCluster == 0) {
//
//                 NT_ASSERT( NodeType(FcbOrDcb) == FAT_NTC_FCB );
//
//                 //
//                 //  We are moving the first cluster of the file, so we need
//                 //  to update our parent directory.
//                 //
//
//                 FatGetDirentFromFcbOrDcb( IrpContext,
//                                           FcbOrDcb,
//                                           FALSE,
//                                           &Dirent,
//                                           &DirentBcb );
//
//                 Dirent->FirstClusterOfFile = (USHORT)FirstSpliceTargetCluster;
//
//                 if (FatIsFat32(Vcb)) {
//
//                     Dirent->FirstClusterOfFileHi =
//                         (USHORT)(FirstSpliceTargetCluster >> 16);
//
//                 }
//
//                 FatSetDirtyBcb( IrpContext, DirentBcb, Vcb, TRUE );
//
//                 FatUnpinBcb( IrpContext, DirentBcb );
//                 DirentBcb = NULL;
//
//                 FatFlushDirentForFile( IrpContext, FcbOrDcb );
//
//                 FcbOrDcb->FirstClusterOfFile = FirstSpliceTargetCluster;
//
//             } else {
//
//                 FatSetFatEntry( IrpContext,
//                                 Vcb,
//                                 FirstSpliceSourceCluster,
//                                 (FAT_ENTRY)FirstSpliceTargetCluster );
//
//                 if ((FcbOrDcb->Header.ValidDataLength.QuadPart != 0) || (NodeType(FcbOrDcb) != FAT_NTC_FCB)) {
//
//                     FatFlushFatEntries( IrpContext, Vcb, FirstSpliceSourceCluster, 1 );
//                 }
//             }
//
//             //
//             //  This was successfully committed.  We no longer want to free
//             //  this allocation on error.
//             //
//
//             DiskSpaceAllocated = FALSE;
//
//             //
//             //  Check if we need to turn off write through for this file.
//             //
//
//             FatMoveFileNeedsWriteThrough(IrpContext, FcbOrDcb, OldWriteThroughFlags);
//
//             //
//             //  Now we just have to free the orphaned space.  We don't have
//             //  to commit this right now as the integrity of the file doesn't
//             //  depend on it.
//             //
//
//             FatDeallocateDiskSpace( IrpContext, Vcb, &SourceMcb, FALSE );
//
//             FatUnpinRepinnedBcbs( IrpContext );
//
//             Status = FatHijackIrpAndFlushDevice( IrpContext,
//                                                  Irp,
//                                                  Vcb->TargetDeviceObject );
//
//             if (!NT_SUCCESS(Status)) {
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//
//             //
//             //  Finally we must replace the old MCB extent information with
//             //  the new.  If this fails from pool allocation, we fix it in
//             //  the finally clause by resetting the file's Mcb.
//             //
//
//             FatRemoveMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                FileOffset,
//                                BytesToReallocate );
//
//             FatAddMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                             FileOffset,
//                             TargetLbo,
//                             BytesToReallocate );
//
//             //
//             //  Now this is the second part of the tricky synchronization.
//             //
//             //  We drop the paging I/O here and signal the notification
//             //  event which allows all waiters (present or future) to proceed.
//             //  Then we block again on the PagingIo exclusive.  When
//             //  we have it, we again know that there can be nobody in the
//             //  read/write path and thus nobody touching the event, so we
//             //  NULL the pointer to it and then drop the PagingIo resource.
//             //
//             //  This combined with our synchronization before the write above
//             //  guarantees that while we were moving the allocation, there
//             //  was no other I/O to this file and because we do not hold
//             //  the paging resource across a flush, we are not exposed to
//             //  a deadlock.
//             //
//
//             KeSetEvent( &StackEvent, 0, FALSE );
//
//             ExAcquireResourceExclusiveLite( FcbOrDcb->Header.PagingIoResource, TRUE );
//
//             FcbOrDcb->MoveFileEvent = NULL;
//             EventArmed = FALSE;
//
//             ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
//
//             //
//             //  Release the resources and let anyone else access the file before
//             //  looping back.
//             //
//
//             FatReleaseFcb( IrpContext, FcbOrDcb );
//             FcbAcquired = FALSE;
//
//             //
//             //  Advance the state variables.
//             //
//
//             TargetCluster += BytesToReallocate >> ClusterShift;
//
//             FileOffset += BytesToReallocate;
//             TargetLbo += BytesToReallocate;
//             ByteCount -= BytesToReallocate;
//
//             LargeTargetLbo.QuadPart += BytesToReallocate;
//
//             //
//             //  Clear the two Mcbs
//             //
//
//             FatRemoveMcbEntry( Vcb, &SourceMcb, 0, 0xFFFFFFFF );
//             FatRemoveMcbEntry( Vcb, &TargetMcb, 0, 0xFFFFFFFF );
//
//             //
//             //  Make the event blockable again.
//             //
//
//             KeClearEvent( &StackEvent );
//         }
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatMoveFile );
//
//         LocalAbnormalTermination |= _SEH2_AbnormalTermination();
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_PARENT_BY_CHILD );
//
//         //
//         //  Free the data buffer, if it was allocated.
//         //
//
//         if (Buffer != NULL) {
//
//             ExFreePool( Buffer );
//         }
//
//         //
//         //  Use a nested try-finally for cleanup if our unpinrepinned
//         //  encounters write-through errors.  This may even be a re-raise.
//         //
//
//         _SEH2_TRY {
//
//             //
//             //  If we have some new allocation hanging around, remove it.  The
//             //  pages needed to do this are guaranteed to be resident because
//             //  we have already repinned them.
//             //
//
//             if (DiskSpaceAllocated) {
//                 FatDeallocateDiskSpace( IrpContext, Vcb, &TargetMcb, FALSE );
//                 FatUnpinRepinnedBcbs( IrpContext );
//             }
//
//         } _SEH2_FINALLY {
//
//             LocalAbnormalTermination |= _SEH2_AbnormalTermination();
//
//             //
//             //  Check on the directory Bcb
//             //
//
//             if (DirentBcb != NULL) {
//                 FatUnpinBcb( IrpContext, DirentBcb );
//             }
//
//             //
//             //  Uninitialize our MCBs
//             //
//
//             if (SourceMcbInitialized) {
//                 FsRtlUninitializeLargeMcb( &SourceMcb );
//             }
//
//             if (TargetMcbInitialized) {
//                 FsRtlUninitializeLargeMcb( &TargetMcb );
//             }
//
//             //
//             //  If this is an abnormal termination then presumably something
//             //  bad happened.  Set the Allocation size to unknown and clear
//             //  the Mcb, but only if we still own the Fcb.
//             //
//             //  It is important to make sure we use a 64bit form of -1.  This is
//             //  what will convince the fastIO path that it cannot extend the file
//             //  in the cache until we have picked up the mapping pairs again.
//             //
//             //  Also, we have to do this while owning PagingIo or we can tear the
//             //  Mcb down in the midst of the noncached IO path looking up extents
//             //  (after we drop it and let them all in).
//             //
//
//             if (LocalAbnormalTermination && FcbAcquired) {
//
//                 if (FcbOrDcb->FirstClusterOfFile == 0) {
//
//                     FcbOrDcb->Header.AllocationSize.QuadPart = 0;
//
//                 } else {
//
//                     FcbOrDcb->Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
//                 }
//
//                 FatRemoveMcbEntry( Vcb, &FcbOrDcb->Mcb, 0, 0xFFFFFFFF );
//             }
//
//             //
//             //  If we broke out of the loop with the Event armed, defuse it
//             //  in the same way we do it after a write.
//             //
//
//             if (EventArmed) {
//                 KeSetEvent( &StackEvent, 0, FALSE );
//                 ExAcquireResourceExclusiveLite( FcbOrDcb->Header.PagingIoResource, TRUE );
//                 FcbOrDcb->MoveFileEvent = NULL;
//                 ExReleaseResourceLite( FcbOrDcb->Header.PagingIoResource );
//             }
//
//             //
//             //  Finally release the main file resource.
//             //
//
//             if (FcbAcquired) {
//
//                 FatReleaseFcb( IrpContext, FcbOrDcb );
//             }
//
//             //
//             //  Now dereference the fileobject.  If the user was a wacko they could have
//             //  tried to nail us by closing the handle right after they threw this move
//             //  down, so we had to keep the fileobject referenced across the entire
//             //  operation.
//             //
//
//             ObDereferenceObject( FileObject );
//
//         } _SEH2_END;
//     } _SEH2_END;
//
//     //
//     //  Complete the irp if we terminated normally.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
//
// //
// //  Local Support Routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatComputeMoveFileParameter (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN ULONG BufferSize,
//     IN ULONG FileOffset,
//     IN OUT PULONG ByteCount,
//     OUT PULONG BytesToReallocate,
//     OUT PULONG BytesToWrite,
//     OUT PLARGE_INTEGER SourceLbo
// )
//
// /*++
//
// Routine Description:
//
//     This is a helper routine for FatMoveFile that analyses the range of
//     file allocation we are moving and determines the actual amount
//     of allocation to be moved and how much needs to be written.
//
// Arguments:
//
//     FcbOrDcb - Supplies the file and its various sizes.
//
//     BufferSize - Supplies the size of the buffer we are using to store the data
//                  being moved.
//
//     FileOffset - Supplies the beginning Vbo of the reallocation zone.
//
//     ByteCount - Supplies the request length to reallocate.  This will
//         be bounded by allocation size on return.
//
//     BytesToReallocate - Receives ByteCount bounded by the file allocation size
//         and buffer size.
//
//     BytesToWrite - Receives BytesToReallocate bounded by ValidDataLength.
//
//     SourceLbo - Receives the logical byte offset of the source data on the volume.
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     ULONG ClusterSize;
//
//     ULONG AllocationSize;
//     ULONG ValidDataLength;
//     ULONG ClusterAlignedVDL;
//     LBO RunLbo;
//     ULONG RunByteCount;
//     ULONG RunIndex;
//     BOOLEAN RunAllocated;
//     BOOLEAN RunEndOnMax;
//
//     PAGED_CODE();
//
//     //
//     //  If we haven't yet set the correct AllocationSize, do so.
//     //
//
//     if (FcbOrDcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//         FatLookupFileAllocationSize( IrpContext, FcbOrDcb );
//
//         //
//         //  If this is a non-root directory, we have a bit more to
//         //  do since it has not gone through FatOpenDirectoryFile().
//         //
//
//         if (NodeType(FcbOrDcb) == FAT_NTC_DCB ||
//             (NodeType(FcbOrDcb) == FAT_NTC_ROOT_DCB && FatIsFat32(FcbOrDcb->Vcb))) {
//
//             FcbOrDcb->Header.FileSize.LowPart =
//                 FcbOrDcb->Header.AllocationSize.LowPart;
//         }
//     }
//
//     //
//     //  Get the number of bytes left to write and ensure that it does
//     //  not extend beyond allocation size.  We return here if FileOffset
//     //  is beyond AllocationSize which can happn on a truncation.
//     //
//
//     AllocationSize = FcbOrDcb->Header.AllocationSize.LowPart;
//     ValidDataLength = FcbOrDcb->Header.ValidDataLength.LowPart;
//
//     if (FileOffset + *ByteCount > AllocationSize) {
//
//         if (FileOffset >= AllocationSize) {
//             *ByteCount = 0;
//             *BytesToReallocate = 0;
//             *BytesToWrite = 0;
//
//             return;
//         }
//
//         *ByteCount = AllocationSize - FileOffset;
//     }
//
//     //
//     //  If there is more than our max, then reduce the byte count for this
//     //  pass to our maximum. We must also align the file offset to a
//     //  buffer size byte boundary.
//     //
//
//     if ((FileOffset & (BufferSize - 1)) + *ByteCount > BufferSize) {
//
//         *BytesToReallocate = BufferSize - (FileOffset & (BufferSize - 1));
//
//     } else {
//
//         *BytesToReallocate = *ByteCount;
//     }
//
//     //
//     //  Find where this data exists on the volume.
//     //
//
//     FatLookupFileAllocation( IrpContext,
//                              FcbOrDcb,
//                              FileOffset,
//                              &RunLbo,
//                              &RunByteCount,
//                              &RunAllocated,
//                              &RunEndOnMax,
//                              &RunIndex );
//
//     NT_ASSERT( RunAllocated );
//
//     //
//     //  Limit this run to the contiguous length.
//     //
//
//     if (RunByteCount < *BytesToReallocate) {
//
//         *BytesToReallocate = RunByteCount;
//     }
//
//     //
//     //  Set the starting offset of the source.
//     //
//
//     SourceLbo->QuadPart = RunLbo;
//
//     //
//     //  We may be able to skip some (or all) of the write
//     //  if allocation size is significantly greater than valid data length.
//     //
//
//     ClusterSize = 1 << FcbOrDcb->Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     NT_ASSERT( ClusterSize <= BufferSize );
//
//     ClusterAlignedVDL = (ValidDataLength + (ClusterSize - 1)) & ~(ClusterSize - 1);
//
//     if ((NodeType(FcbOrDcb) == FAT_NTC_FCB) &&
//         (FileOffset + *BytesToReallocate > ClusterAlignedVDL)) {
//
//         if (FileOffset > ClusterAlignedVDL) {
//
//             *BytesToWrite = 0;
//
//         } else {
//
//             *BytesToWrite = ClusterAlignedVDL - FileOffset;
//         }
//
//     } else {
//
//         *BytesToWrite = *BytesToReallocate;
//     }
// }
//
//
// //
// //  Local Support Routine
// //
//
// VOID
// FatComputeMoveFileSplicePoints (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN ULONG FileOffset,
//     IN ULONG TargetCluster,
//     IN ULONG BytesToReallocate,
//     OUT PULONG FirstSpliceSourceCluster,
//     OUT PULONG FirstSpliceTargetCluster,
//     OUT PULONG SecondSpliceSourceCluster,
//     OUT PULONG SecondSpliceTargetCluster,
//     IN OUT PLARGE_MCB SourceMcb
// )
//
// /*++
//
// Routine Description:
//
//     This is a helper routine for FatMoveFile that analyzes the range of
//     file allocation we are moving and generates the splice points in the
//     FAT table.
//
// Arguments:
//
//     FcbOrDcb - Supplies the file and thus Mcb.
//
//     FileOffset - Supplies the beginning Vbo of the reallocation zone.
//
//     TargetCluster - Supplies the beginning cluster of the reallocation target.
//
//     BytesToReallocate - Suppies the length of the reallocation zone.
//
//     FirstSpliceSourceCluster - Receives the last cluster in previous allocation
//         or zero if we are reallocating from VBO 0.
//
//     FirstSpliceTargetCluster - Receives the target cluster (i.e. new allocation)
//
//     SecondSpliceSourceCluster - Receives the final target cluster.
//
//     SecondSpliceTargetCluster - Receives the first cluster of the remaining
//         source allocation or FAT_CLUSTER_LAST if the reallocation zone
//         extends to the end of the file.
//
//     SourceMcb - This supplies an MCB that will be filled in with run
//         information describing the file allocation being replaced.  The Mcb
//         must be initialized by the caller.
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     VBO SourceVbo;
//     LBO SourceLbo;
//     ULONG SourceIndex;
//     ULONG SourceBytesInRun;
//     ULONG SourceBytesRemaining;
//
//     ULONG SourceMcbVbo = 0;
//     ULONG SourceMcbBytesInRun = 0;
//
//     PVCB Vcb;
//     BOOLEAN Result;
//
//     PAGED_CODE();
//
//     Vcb = FcbOrDcb->Vcb;
//
//     //
//     //  Get information on the final cluster in the previous allocation and
//     //  prepare to enumerate it in the follow loop.
//     //
//
//     if (FileOffset == 0) {
//
//         SourceIndex = 0;
//         *FirstSpliceSourceCluster = 0;
//         Result = FatGetNextMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                      0,
//                                      &SourceVbo,
//                                      &SourceLbo,
//                                      &SourceBytesInRun );
//
//     } else {
//
//         Result = FatLookupMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                     FileOffset-1,
//                                     &SourceLbo,
//                                     &SourceBytesInRun,
//                                     &SourceIndex);
//
//         *FirstSpliceSourceCluster = FatGetIndexFromLbo( Vcb, SourceLbo );
//
//         if ((Result) && (SourceBytesInRun == 1)) {
//
//             SourceIndex += 1;
//             Result = FatGetNextMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                          SourceIndex,
//                                          &SourceVbo,
//                                          &SourceLbo,
//                                          &SourceBytesInRun);
//
//         } else {
//
//             SourceVbo = FileOffset;
//             SourceLbo += 1;
//             SourceBytesInRun -= 1;
//         }
//     }
//
//     //
//     //  Run should always be present, but don't bugcheck in the case where it's not.
//     //
//
//     if (!Result) {
//
//         NT_ASSERT( FALSE);
//         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR);
//     }
//
//     //
//     //  At this point the variables:
//     //
//     //  - SourceIndex - SourceLbo - SourceBytesInRun -
//     //
//     //  all correctly decribe the allocation to be removed.  In the loop
//     //  below we will start here and continue enumerating the Mcb runs
//     //  until we are finished with the allocation to be relocated.
//     //
//
//     *FirstSpliceTargetCluster = TargetCluster;
//
//     *SecondSpliceSourceCluster =
//          *FirstSpliceTargetCluster +
//          (BytesToReallocate >> Vcb->AllocationSupport.LogOfBytesPerCluster) - 1;
//
//     for (SourceBytesRemaining = BytesToReallocate, SourceMcbVbo = 0;
//
//          SourceBytesRemaining > 0;
//
//          SourceIndex += 1,
//          SourceBytesRemaining -= SourceMcbBytesInRun,
//          SourceMcbVbo += SourceMcbBytesInRun) {
//
//         if (SourceMcbVbo != 0) {
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//             Result = FatGetNextMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                          SourceIndex,
//                                          &SourceVbo,
//                                          &SourceLbo,
//                                          &SourceBytesInRun );
//             NT_ASSERT( Result);
//         }
//
//         NT_ASSERT( SourceVbo == SourceMcbVbo + FileOffset );
//
//         SourceMcbBytesInRun =
//             SourceBytesInRun < SourceBytesRemaining ?
//             SourceBytesInRun : SourceBytesRemaining;
//
//         FatAddMcbEntry( Vcb, SourceMcb,
//                         SourceMcbVbo,
//                         SourceLbo,
//                         SourceMcbBytesInRun );
//     }
//
//     //
//     //  Now compute the cluster of the target of the second
//     //  splice.  If the final run in the above loop was
//     //  more than we needed, then we can just do arithmetic,
//     //  otherwise we have to look up the next run.
//     //
//
//     if (SourceMcbBytesInRun < SourceBytesInRun) {
//
//         *SecondSpliceTargetCluster =
//             FatGetIndexFromLbo( Vcb, SourceLbo + SourceMcbBytesInRun );
//
//     } else {
//
//         if (FatGetNextMcbEntry( Vcb, &FcbOrDcb->Mcb,
//                                 SourceIndex,
//                                 &SourceVbo,
//                                 &SourceLbo,
//                                 &SourceBytesInRun )) {
//
//             *SecondSpliceTargetCluster = FatGetIndexFromLbo( Vcb, SourceLbo );
//
//         } else {
//
//             *SecondSpliceTargetCluster = FAT_CLUSTER_LAST;
//         }
//     }
// }
//
//
// NTSTATUS
// FatAllowExtendedDasdIo(
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
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
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     PIO_STACK_LOCATION IrpSp;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current Irp stack location and save some references.
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//
//         DebugTrace(-1, Dbg, "FatAllowExtendedDasdIo -> %08lx\n", STATUS_INVALID_PARAMETER);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     SetFlag( Ccb->Flags, CCB_FLAG_ALLOW_EXTENDED_DASD_IO );
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetRetrievalPointerBase (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This routine retrieves the sector offset to the first allocation unit.
//
// Arguments:
//
//     IrpContext - Supplies the Irp Context.
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     PIO_STACK_LOCATION              IrpSp = NULL;
//     PVCB                            Vcb = NULL;
//     PFCB                            Fcb = NULL;
//     PCCB                            Ccb = NULL;
//     ULONG                           BufferLength = 0;
//     PRETRIEVAL_POINTER_BASE         RetrievalPointerBase = NULL;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Force WAIT to true.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // Extract the buffer
//     //
//
//     RetrievalPointerBase = Irp->AssociatedIrp.SystemBuffer;
//     BufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//     //
//     // Verify the handle has manage volume access.
//     //
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Validate the output buffer is the right size.
//     //
//     //  Note that the default size of BOOT_AREA_INFO has enough room for 2 boot sectors, so we're fine.
//     //
//
//     if (BufferLength < sizeof(RETRIEVAL_POINTER_BASE)) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // Fill out the offset to the file area.
//     //
//
//     RtlZeroMemory( RetrievalPointerBase, BufferLength );
//
//     try {
//
//         FatAcquireSharedVcb(IrpContext, Vcb);
//         FatQuickVerifyVcb(IrpContext, Vcb);
//
//         RetrievalPointerBase->FileAreaOffset.QuadPart = Vcb->AllocationSupport.FileAreaLbo >> Vcb->AllocationSupport.LogOfBytesPerSector;
//         Irp->IoStatus.Information = sizeof( RETRIEVAL_POINTER_BASE );
//
//     } finally {
//
//         FatReleaseVcb(IrpContext, Vcb);
//
//     }
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     return STATUS_SUCCESS;
//
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatGetBootAreaInfo (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This routine retrieves information about the boot areas of the filesystem.
//
// Arguments:
//
//     IrpContext - Supplies the Irp Context.
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     PIO_STACK_LOCATION              IrpSp = NULL;
//     PVCB                            Vcb = NULL;
//     PFCB                            Fcb = NULL;
//     PCCB                            Ccb = NULL;
//     ULONG                           BufferLength = 0;
//     PBOOT_AREA_INFO                 BootAreaInfo = NULL;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Force WAIT to true.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     if (FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // Extract the buffer
//     //
//
//     BootAreaInfo = Irp->AssociatedIrp.SystemBuffer;
//     BufferLength = IrpSp->Parameters.FileSystemControl.OutputBufferLength;
//
//     //
//     // Verify the handle has manage volume access.
//     //
//
//     if ((Ccb == NULL) || !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // Validate the output buffer is the right size.
//     //
//     // Note that the default size of BOOT_AREA_INFO has enough room for 2 boot sectors, so we're fine.
//     //
//
//     if (BufferLength < sizeof(BOOT_AREA_INFO)) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     //
//     // Fill out our boot areas.
//     //
//
//     RtlZeroMemory( BootAreaInfo, BufferLength );
//
//     try {
//
//         FatAcquireSharedVcb(IrpContext, Vcb);
//         FatQuickVerifyVcb(IrpContext, Vcb);
//
//         if (FatIsFat32( Vcb )) {
//
//             BootAreaInfo->BootSectorCount = 2;
//             BootAreaInfo->BootSectors[0].Offset.QuadPart = 0;
//             BootAreaInfo->BootSectors[1].Offset.QuadPart = 6;
//         } else {
//
//             BootAreaInfo->BootSectorCount = 1;
//             BootAreaInfo->BootSectors[0].Offset.QuadPart = 0;
//         }
//
//         Irp->IoStatus.Information = sizeof( BOOT_AREA_INFO );
//
//     } finally {
//
//         FatReleaseVcb(IrpContext, Vcb);
//     }
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//     return STATUS_SUCCESS;
// }
//
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatMarkHandle (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This routine is used to attach special properties to a user handle.
//
// Arguments:
//
//     IrpContext - Supplies the Irp Context.
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     NTSTATUS                        Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION              IrpSp = NULL;
//     PVCB                            Vcb = NULL;
//     PFCB                            Fcb = NULL;
//     PCCB                            Ccb = NULL;
//     PFCB                            DasdFcb = NULL;
//     PCCB                            DasdCcb = NULL;
//     TYPE_OF_OPEN                    TypeOfOpen;
//     PMARK_HANDLE_INFO               HandleInfo = NULL;
//     PFILE_OBJECT                    DasdFileObject = NULL;
//     BOOLEAN                         ReleaseFcb = FALSE;
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     MARK_HANDLE_INFO                LocalMarkHandleInfo = {0};
// #endif
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Always make this a synchronous IRP.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb ) ;
//
//     //
//     //  We currently support this call for files and directories only.
//     //
//
//     if ((TypeOfOpen != UserFileOpen) &&
//         (TypeOfOpen != UserDirectoryOpen)) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//
//     //
//     //  Win32/64 thunking code
//     //
//
//     if (IoIs32bitProcess( Irp )) {
//
//         PMARK_HANDLE_INFO32 MarkHandle32;
//
//         if (IrpSp->Parameters.FileSystemControl.InputBufferLength < sizeof( MARK_HANDLE_INFO32 )) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//
//         MarkHandle32 = (PMARK_HANDLE_INFO32) Irp->AssociatedIrp.SystemBuffer;
//         LocalMarkHandleInfo.HandleInfo = MarkHandle32->HandleInfo;
//         LocalMarkHandleInfo.UsnSourceInfo = MarkHandle32->UsnSourceInfo;
//         LocalMarkHandleInfo.VolumeHandle = (HANDLE)(ULONG_PTR)(LONG) MarkHandle32->VolumeHandle;
//
//         HandleInfo = &LocalMarkHandleInfo;
//
//     } else {
//
// #endif
//
//     //
//     //  Get the input buffer pointer and check its length.
//     //
//
//     if (IrpSp->Parameters.FileSystemControl.InputBufferLength < sizeof( MARK_HANDLE_INFO )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     HandleInfo = (PMARK_HANDLE_INFO) Irp->AssociatedIrp.SystemBuffer;
//
// #if defined(_WIN64) && defined(BUILD_WOW64_ENABLED)
//     }
// #endif
//
//     //
//     //  Check that only legal bits are being set.
//     //  We currently only support two bits: protect clusters and unprotect clusters.
//     //
//     //  Note that we don't actually support the USN journal, but we must ignore the flags in order
//     //  to preserve compatibility.
//     //
//
//     if (FlagOn( HandleInfo->HandleInfo,
//                 ~(MARK_HANDLE_PROTECT_CLUSTERS)) ||
//         (FlagOn( HandleInfo->HandleInfo,
//                  0 ) &&
//          (IrpSp->MinorFunction != IRP_MN_KERNEL_CALL)) ||
//         FlagOn(HandleInfo->UsnSourceInfo,
//                 ~(USN_SOURCE_DATA_MANAGEMENT |
//                   USN_SOURCE_AUXILIARY_DATA |
//                   USN_SOURCE_REPLICATION_MANAGEMENT) ) ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Check that the user has a valid volume handle or the manage volume
//     //  privilege or is a kernel mode caller
//     //
//     //  NOTE: the kernel mode check is only valid because the rdr doesn't support this
//     //  FSCTL.
//     //
//
//     if ((Irp->RequestorMode != KernelMode) &&
//         (IrpSp->MinorFunction != IRP_MN_KERNEL_CALL) &&
//         !FlagOn( Ccb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS ) &&
//         ( FlagOn( HandleInfo->HandleInfo, MARK_HANDLE_PROTECT_CLUSTERS ) || (HandleInfo->UsnSourceInfo != 0) )) {
//
//         if (HandleInfo->VolumeHandle == 0) {
//             FatCompleteRequest( IrpContext, Irp, STATUS_ACCESS_DENIED );
//             return STATUS_ACCESS_DENIED;
//         }
//
//         Status = ObReferenceObjectByHandle( HandleInfo->VolumeHandle,
//                                             0,
//                                             *IoFileObjectType,
//                                             UserMode,
// #ifndef __REACTOS__
//                                             &DasdFileObject,
// #else
//                                             (PVOID *)&DasdFileObject,
// #endif
//                                             NULL );
//
//         if (!NT_SUCCESS(Status)) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//             return Status;
//         }
//
//         //
//         //  Check that this file object is opened on the same volume as the
//         //  handle used to call this routine.
//         //
//
//         if (DasdFileObject->Vpb != Vcb->Vpb) {
//
//             ObDereferenceObject( DasdFileObject );
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         //
//         //  Now decode this FileObject and verify it is a volume handle.
//         //  We don't care to raise on dismounts here because
//         //  we check for that further down anyway. So send FALSE.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "convenient for debugging" )
// #endif
//         TypeOfOpen = FatDecodeFileObject( DasdFileObject, &Vcb, &DasdFcb, &DasdCcb ) ;
//
//         ObDereferenceObject( DasdFileObject );
//
//         if ((DasdCcb == NULL) || !FlagOn( DasdCcb->Flags, CCB_FLAG_MANAGE_VOLUME_ACCESS )) {
//
//             FatCompleteRequest( IrpContext, Irp, STATUS_ACCESS_DENIED );
//             return STATUS_ACCESS_DENIED;
//         }
//
//     }
//
//     _SEH2_TRY {
//
//         FatAcquireExclusiveFcb(IrpContext, Fcb);
//         ReleaseFcb = TRUE;
//
//         FatVerifyFcb( IrpContext, Fcb );
//
//         if (HandleInfo->HandleInfo & MARK_HANDLE_PROTECT_CLUSTERS) {
//
//             if (Fcb->FcbState & FCB_STATE_DENY_DEFRAG) {
//
//                 //
//                 //  It's already set, bail out.
//                 //
//
//                 try_return( Status = STATUS_ACCESS_DENIED );
//             }
//
//             Ccb->Flags |= CCB_FLAG_DENY_DEFRAG;
//             Fcb->FcbState|= FCB_STATE_DENY_DEFRAG;
//
//         }
//
//         try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         if (ReleaseFcb) {
//
//             FatReleaseFcb(IrpContext, Fcb);
//         }
//
//     } _SEH2_END;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatFlushAndCleanVolume(
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp,
//     IN PVCB Vcb,
//     IN FAT_FLUSH_TYPE FlushType
//     )
// /*++
//
// Routine Description:
//
//     This routine flushes and otherwise preparse a volume to be eligible
//     for deletion.  The dismount and PNP paths share the need for this
//     common work.
//
//     The Vcb will always be valid on return from this function. It is the
//     caller's responsibility to attempt the dismount/deletion, and to setup
//     allocation support again if the volume will be brought back from the
//     brink.
//
// Arguments:
//
//     Irp - Irp for the overlying request
//
//     Vcb - the volume being operated on
//
//     FlushType - specifies the kind of flushing desired
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     PAGED_CODE();
//
//     //
//     //  The volume must be held exclusive.
//     //
//
//     NT_ASSERT( FatVcbAcquiredExclusive( IrpContext, Vcb ));
//
//     //
//     //  There is no fail, flush everything. If invalidating, it is important
//     //  that we invalidate as we flush (eventually, w/ paging io held) so that we
//     //  error out the maximum number of late writes.
//     //
//
//     if (FlushType != NoFlush) {
//
//         (VOID) FatFlushVolume( IrpContext, Vcb, FlushType );
//     }
//
//     FatCloseEaFile( IrpContext, Vcb, FALSE );
//
//     //
//     //  Now, tell the device to flush its buffers.
//     //
//
//     if (FlushType != NoFlush) {
//
//         (VOID)FatHijackIrpAndFlushDevice( IrpContext, Irp, Vcb->TargetDeviceObject );
//     }
//
//     //
//     //  Now purge everything in sight.  We're trying to provoke as many closes as
//     //  soon as possible, this volume may be on its way out.
//     //
//
//     if (FlushType != FlushWithoutPurge) {
//
//         CcPurgeCacheSection( &Vcb->SectionObjectPointers,
//                              NULL,
//                              0,
//                              FALSE );
//
//         (VOID) FatPurgeReferencedFileObjects( IrpContext, Vcb->RootDcb, NoFlush );
//     }
//
//     //
//     //  If the volume was dirty and we were allowed to flush, do the processing that
//     //  the delayed callback would have done.
//     //
//
//     if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY)) {
//
//         //
//         //  Cancel any pending clean volumes.
//         //
//
//         (VOID)KeCancelTimer( &Vcb->CleanVolumeTimer );
//         (VOID)KeRemoveQueueDpc( &Vcb->CleanVolumeDpc );
//
//
//         if (FlushType != NoFlush) {
//
//             //
//             //  The volume is now clean, note it.
//             //
//
//             if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                 FatMarkVolume( IrpContext, Vcb, VolumeClean );
//                 ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//             }
//
//             //
//             //  Unlock the volume if it is removable.
//             //
//
//             if (FlagOn(Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) &&
//                 !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_BOOT_OR_PAGING_FILE)) {
//
//                 FatToggleMediaEjectDisable( IrpContext, Vcb, FALSE );
//             }
//         }
//     }
//
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//
// _Requires_lock_held_(_Global_critical_region_)
// NTSTATUS
// FatSetPurgeFailureMode (
//      PIRP_CONTEXT IrpContext,
//      PIRP Irp
//     )
// /*++
//
//     This routine is used to enable or disable the purge failure mode
//     on a file. When in this mode the file system will propagate purge
//     failures encountered during coherency purges. Normally these are
//     ignored for application compatibilty purposes. Since the normal
//     behavior can lead to cache incoherency there needs to be a way to
//     force error propagation, particulary when a filter has mapped a
//     section for the purposes of scanning the file in the background.
//
//     The purge failure mode is a reference count because it is set
//     per mapped section and there may be multiple sections backed by
//     the file.
//
// Arguments:
//
//     IrpContext - Supplies the Irp Context.
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The return status for the operation.
//
// --*/
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//     TYPE_OF_OPEN TypeOfOpen;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//     PSET_PURGE_FAILURE_MODE_INPUT SetPurgeInput;
//     BOOLEAN FcbAcquired = FALSE;
//
//     PAGED_CODE();
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  Force WAIT to true.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     //
//     //  This has to be a kernel only call. Can't let a user request
//     //  change the purge failure mode count
//     //
//
//     if (Irp->RequestorMode != KernelMode) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Extract and decode the file object and check for type of open.
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     if (TypeOfOpen == UserDirectoryOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_FILE_IS_A_DIRECTORY );
//         return STATUS_FILE_IS_A_DIRECTORY;
//     }
//
//     if (TypeOfOpen != UserFileOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     //  Get the input buffer pointer and check its length.
//     //
//
//     if (IrpSp->Parameters.FileSystemControl.InputBufferLength < sizeof( SET_PURGE_FAILURE_MODE_INPUT )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_BUFFER_TOO_SMALL );
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     SetPurgeInput = (PSET_PURGE_FAILURE_MODE_INPUT) Irp->AssociatedIrp.SystemBuffer;
//
//     if (!FlagOn( SetPurgeInput->Flags, SET_PURGE_FAILURE_MODE_ENABLED | SET_PURGE_FAILURE_MODE_DISABLED )) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_INVALID_PARAMETER );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     try {
//
//         //
//         //  Acquire the FCB exclusively to synchronize with coherency flush
//         //  and purge.
//         //
//
//         FatAcquireExclusiveFcb( IrpContext, Fcb );
//         FcbAcquired = TRUE;
//
//         FatVerifyFcb( IrpContext, Fcb );
//
//         if (FlagOn( SetPurgeInput->Flags, SET_PURGE_FAILURE_MODE_ENABLED )) {
//
//             if (Fcb->PurgeFailureModeEnableCount == MAXULONG) {
//
//                 try_return( Status = STATUS_INVALID_PARAMETER );
//             }
//
//             Fcb->PurgeFailureModeEnableCount += 1;
//
//         } else {
//
//             ASSERT( FlagOn( SetPurgeInput->Flags, SET_PURGE_FAILURE_MODE_DISABLED ));
//
//             if (Fcb->PurgeFailureModeEnableCount == 0) {
//
//                 try_return( Status = STATUS_INVALID_PARAMETER );
//             }
//
//             Fcb->PurgeFailureModeEnableCount -= 1;
//         }
//
//         try_exit: NOTHING;
//
//     } finally {
//
//         if (FcbAcquired) {
//             FatReleaseFcb( IrpContext, Fcb );
//         }
//     }
//
//     //
//     //  Complete the irp if we terminated normally.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//
//     return Status;
// }
//
// #endif
//
//
// NTSTATUS
// FatSearchBufferForLabel(
//     IN  PIRP_CONTEXT IrpContext,
//     IN  PVPB  Vpb,
//     IN  PVOID Buffer,
//     IN  ULONG Size,
//     OUT PBOOLEAN LabelFound
//     )
// /*++
//
// Routine Description:
//
//     Search a buffer (taken from the root directory) for a volume label
//     matching the label in the
//
// Arguments:
//
//     IrpContext - Supplies our irp context
//     Vpb        - Vpb supplying the volume label
//     Buffer     - Supplies the buffer we'll search
//     Size       - The size of the buffer in bytes.
//     LabelFound - Returns whether a label was found.
//
// Return Value:
//
//     There are four interesting cases:
//
//     1) Some random error occurred - that error returned as status, LabelFound
//                                     is indeterminate.
//
//     2) No label was found         - STATUS_SUCCESS returned, LabelFound is FALSE.
//
//     3) A matching label was found - STATUS_SUCCESS returned, LabelFound is TRUE.
//
//     4) A non-matching label found - STATUS_WRONG_VOLUME returned, LabelFound
//                                     is indeterminate.
//
// --*/
//
// {
//     NTSTATUS Status;
//     WCHAR UnicodeBuffer[11];
//
//     PDIRENT Dirent;
//     PDIRENT TerminationDirent;
//     ULONG VolumeLabelLength;
//     UCHAR OemBuffer[11];
//     OEM_STRING OemString;
//     UNICODE_STRING UnicodeString;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     Dirent = Buffer;
//
//     TerminationDirent = Dirent + Size / sizeof(DIRENT);
//
//     while ( Dirent < TerminationDirent ) {
//
//         if ( Dirent->FileName[0] == FAT_DIRENT_NEVER_USED ) {
//
//             Dirent = TerminationDirent;
//             break;
//         }
//
//         //
//         //  If the entry is the non-deleted volume label break from the loop.
//         //
//         //  Note that all out parameters are already correctly set.
//         //
//
//         if (((Dirent->Attributes & ~FAT_DIRENT_ATTR_ARCHIVE) ==
//              FAT_DIRENT_ATTR_VOLUME_ID) &&
//             (Dirent->FileName[0] != FAT_DIRENT_DELETED)) {
//
//             break;
//         }
//
//         Dirent += 1;
//     }
//
//     if (Dirent >= TerminationDirent) {
//
//         //
//         //  We've run out of buffer.
//         //
//
//         *LabelFound = FALSE;
//         return STATUS_SUCCESS;
//     }
//
//
//     OemString.Buffer = (PCHAR)&OemBuffer[0];
//     OemString.MaximumLength = 11;
//
//     RtlCopyMemory( OemString.Buffer, Dirent->FileName, 11 );
//
//     //
//     //  Translate the first character from 0x5 to 0xe5.
//     //
//
//     if (OemString.Buffer[0] == FAT_DIRENT_REALLY_0E5) {
//
//         OemString.Buffer[0] = 0xe5;
//     }
//
//     //
//     //  Compute the length of the volume name
//     //
//
//     for ( OemString.Length = 11;
//           OemString.Length > 0;
//           OemString.Length -= 1) {
//
//         if ( (OemString.Buffer[OemString.Length-1] != 0x00) &&
//              (OemString.Buffer[OemString.Length-1] != 0x20) ) { break; }
//     }
//
//     UnicodeString.MaximumLength = sizeof( UnicodeBuffer );
//     UnicodeString.Buffer = &UnicodeBuffer[0];
//
//     Status = RtlOemStringToCountedUnicodeString( &UnicodeString,
//                                                  &OemString,
//                                                  FALSE );
//
//     if ( !NT_SUCCESS( Status ) ) {
//
//         return Status;
//     }
//
//     VolumeLabelLength = UnicodeString.Length;
//
//     if ( (VolumeLabelLength != (ULONG)Vpb->VolumeLabelLength) ||
//          (!RtlEqualMemory(&UnicodeBuffer[0],
//                           &Vpb->VolumeLabel[0],
//                           VolumeLabelLength)) ) {
//
//         return STATUS_WRONG_VOLUME;
//     }
//
//     //
//     //  We found a matching label.
//     //
//
//     *LabelFound = TRUE;
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// FatVerifyLookupFatEntry (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG FatIndex,
//     IN OUT PULONG FatEntry
//     )
// {
//     ULONG PageEntryOffset;
//     ULONG OffsetIntoVolumeFile;
//     PVOID Buffer;
//
//     PAGED_CODE();
//
//     NT_ASSERT(Vcb->AllocationSupport.FatIndexBitSize == 32);
//
//     FatVerifyIndexIsValid( IrpContext, Vcb, FatIndex);
//
//     Buffer = FsRtlAllocatePoolWithTag( NonPagedPoolNxCacheAligned,
//                                        PAGE_SIZE,
//                                        TAG_ENTRY_LOOKUP_BUFFER );
//
//     OffsetIntoVolumeFile =  FatReservedBytes(&Vcb->Bpb) + FatIndex * sizeof(ULONG);
//     PageEntryOffset = (OffsetIntoVolumeFile % PAGE_SIZE) / sizeof(ULONG);
//
//     _SEH2_TRY {
//
//         FatPerformVerifyDiskRead( IrpContext,
//                                   Vcb,
//                                   Buffer,
//                                   OffsetIntoVolumeFile & ~(PAGE_SIZE - 1),
//                                   PAGE_SIZE,
//                                   TRUE );
//
//         *FatEntry = ((PULONG)(Buffer))[PageEntryOffset];
//
//     } _SEH2_FINALLY {
//
//         ExFreePool( Buffer );
//     } _SEH2_END;
// }
//
// //
// //  Local support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatScanForDismountedVcb (
//     IN PIRP_CONTEXT IrpContext
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
//     BOOLEAN VcbDeleted;
//
//
//     PAGED_CODE();
//
//     //
//     //  Walk through all of the Vcb's attached to the global data.
//     //
//
//     NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//     FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     Links = FatData.VcbQueue.Flink;
//
//     while (Links != &FatData.VcbQueue) {
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
//         //  Try to acquire the VCB for exclusive access.  If we cannot, just skip
//         //  it for now.
//         //
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable:28103,"prefast cannot work out that Vcb->Resource will be released below." )
// #pragma prefast( disable:28109,"prefast cannot work out the Vcb is not already held" );
// #endif
//
//         if (!ExAcquireResourceExclusiveLite( &(Vcb->Resource), FALSE )) {
//
//             continue;
//         }
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//         //
//         //  Check if this Vcb can go away.
//         //
//
//         VcbDeleted = FatCheckForDismount( IrpContext,
//                                           Vcb,
//                                           FALSE );
//
//         //
//         //  If the VCB was not deleted, release it.
//         //
//
//         if (!VcbDeleted) {
//
//             ExReleaseResourceLite( &(Vcb->Resource) );
//         }
//     }
//
//     FatReleaseGlobal( IrpContext);
//
//     return;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN7)
// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //
// //  FatSetZeroOnDeallocate is used when we need to stomp over the contents with zeros when a file is deleted.
// //
//
// NTSTATUS
// FatSetZeroOnDeallocate (
//     __in PIRP_CONTEXT IrpContext,
//     __in PIRP Irp
//     )
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PVCB Vcb;
//     PFCB FcbOrDcb;
//     PCCB Ccb;
//
//     TYPE_OF_OPEN TypeOfOpen;
//
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     BOOLEAN ReleaseFcb = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  This call should always be synchronous.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &FcbOrDcb, &Ccb );
//
//     if ((TypeOfOpen != UserFileOpen) ||
//         (!IrpSp->FileObject->WriteAccess) ) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_ACCESS_DENIED );
//         return STATUS_ACCESS_DENIED;
//     }
//
//     //
//     //  Readonly mount should be just that: read only.
//     //
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_MEDIA_WRITE_PROTECTED );
//         return STATUS_MEDIA_WRITE_PROTECTED;
//     }
//
//     //
//     // Acquire main then paging to exclude everyone from this FCB.
//     //
//
//     FatAcquireExclusiveFcb(IrpContext, FcbOrDcb);
//     ReleaseFcb = TRUE;
//
//     _SEH2_TRY {
//
//         SetFlag( FcbOrDcb->FcbState, FCB_STATE_ZERO_ON_DEALLOCATION );
//
//     } _SEH2_FINALLY {
//
//         if (ReleaseFcb) {
//             FatReleaseFcb(IrpContext, FcbOrDcb);
//         }
//
//     } _SEH2_END;
//
//     FatCompleteRequest( IrpContext, Irp, Status );
//     return Status;
// }
// #endif
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// ============================================================================
// FatMountVolume, FatInvalidateVolumes, FatCommonFileSystemControl,
// FatFsdFileSystemControl — stubs for large commented-out bodies.
// TODO: translate from fsctrl.c commented blocks.
// ============================================================================

use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};

extern "system" {
    fn FatIsIrpTopLevel(Irp: PIRP) -> BOOLEAN;
    fn CanFsdWait(Irp: PIRP) -> BOOLEAN;
    fn FatCreateIrpContext(Irp: PIRP, CanWait: u32) -> PIRP_CONTEXT;
}

// Cross-module functions referenced by FatMountVolume (resolve at link to sibling #[no_mangle]).
extern "system" {
    fn FatDeviceIsFatFsdo(DeviceObject: *mut DEVICE_OBJECT) -> u8;
    fn FatInitializeVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, TargetDeviceObject: *mut DEVICE_OBJECT, Vpb: *mut VPB, FsDeviceObject: *mut DEVICE_OBJECT);
    fn FatReadVolumeFile(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, StartingVbo: u32, ByteCount: u32, Bcb: *mut PBCB, Buffer: *mut *mut core::ffi::c_void);
    fn FatCreateRootDcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
    fn FatLocateVolumeLabel(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Dirent: *mut PDIRENT, Bcb: *mut PBCB, ByteOffset: *mut u32);
    fn FatInitializeCacheMap(FileObject: PFILE_OBJECT, FileSizes: *const CC_FILE_SIZES, PinAccess: BOOLEAN, Callbacks: *const CACHE_MANAGER_CALLBACKS, LazyWriteContext: *mut core::ffi::c_void);
    fn FatCreateFcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, ParentDcb: *mut DCB, LfnOffsetWithinDirectory: u32, DirentOffsetWithinDirectory: u32, Dirent: *mut DIRENT, Lfn: *mut _UNICODE_STRING, OrigLfn: *mut _UNICODE_STRING, IsPagingFile: i32, SingleResource: i32) -> *mut FCB;
}

// NT/IOCTL/fastfat constants used by FatMountVolume — header-faithful values
// (CTL_CODE(IOCTL_DISK_BASE=7,fn,METHOD_BUFFERED=0,access); ntdddisk.h / winioctl.h / nodetype.h).
const IOCTL_DISK_CHECK_VERIFY: u32 = 0x0007_4800;          // CTL_CODE(7,0x200,0,FILE_READ_ACCESS=1)
const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x0007_0000;    // CTL_CODE(7,0,0,0)
const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x0007_0048; // CTL_CODE(7,0x12,0,0)
const STATUS_UNRECOGNIZED_VOLUME: i32 = 0xC000_014Fu32 as i32;
const STATUS_FILE_CORRUPT_ERROR: i32 = 0xC000_0102u32 as i32;
const MAXIMUM_VOLUME_LABEL_LENGTH: u16 = 64;               // 32 * sizeof(WCHAR)
const PARTITION_OS2BOOTMGR: u8 = 0x0A;
const PARTITION_STYLE_MBR: u32 = 0;                        // _PARTITION_STYLE::PARTITION_STYLE_MBR
const TAG_STASHED_BPB: u32 = 0x5374_6146;                  // 'StaF'
const DO_VERIFY_VOLUME: u32 = 2;
const DO_SYSTEM_BOOT_PARTITION: u32 = 256;
const FILE_DEVICE_DISK: u32 = 7;
const FILE_DEVICE_CD_ROM: u32 = 2;
const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 8;
const FSRTL_VOLUME_MOUNT: u32 = 6;
const SL_ALLOW_RAW_MOUNT: u32 = 1;

#[no_mangle]
pub unsafe extern "system" fn FatMountVolume(
    IrpContext: PIRP_CONTEXT,
    TargetDeviceObject: PDEVICE_OBJECT,
    mut Vpb: PVPB,
    FsDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    let mut BootBcb: PBCB = core::ptr::null_mut();
    let mut BootSector: PPACKED_BOOT_SECTOR = core::ptr::null_mut();

    let mut DirentBcb: PBCB = core::ptr::null_mut();
    let mut Dirent: PDIRENT = core::ptr::null_mut();
    let mut ByteOffset: ULONG = 0;

    let mut MountNewVolume: BOOLEAN = 0u8;
    let mut WeClearedVerifyRequiredBit: BOOLEAN = 0u8;
    let mut DoARemount: BOOLEAN = 0u8;

    let mut OldVcb: PVCB = core::ptr::null_mut();
    let mut OldVpb: PVPB = core::ptr::null_mut();

    let mut RealDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut VolDo: PVOLUME_DEVICE_OBJECT = core::ptr::null_mut();
    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut RootDirectoryFile: PFILE_OBJECT = core::ptr::null_mut();

    let mut Links: PLIST_ENTRY = core::ptr::null_mut();

    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ChangeCount: ULONG = 0;

    let mut Geometry: DISK_GEOMETRY = core::mem::zeroed();

    let mut PartitionInformation: PARTITION_INFORMATION_EX = core::mem::zeroed();
    let StatusPartInfo: NTSTATUS;

    DebugTrace(1, Dbg, b"FatMountVolume\n\0".as_ptr() as _, 0);
    DebugTrace(0, Dbg, b"TargetDeviceObject = %p\n\0".as_ptr() as _, TargetDeviceObject as ULONG_PTR);
    DebugTrace(0, Dbg, b"Vpb                = %p\n\0".as_ptr() as _, Vpb as ULONG_PTR);

    NT_ASSERT((FlagOn(((*IrpContext).Flags) as u32, IRP_CONTEXT_FLAG_WAIT) != 0) as u32);
    NT_ASSERT((FatDeviceIsFatFsdo(FsDeviceObject) != 0) as u32);

    if FlagOn(((*TargetDeviceObject).Characteristics) as u32, FILE_REMOVABLE_MEDIA) != 0 {
        Status = FatPerformDevIoCtrl(
            IrpContext,
            IOCTL_DISK_CHECK_VERIFY,
            TargetDeviceObject,
            core::ptr::null_mut(),
            0,
            &mut ChangeCount as *mut _ as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            false,
            true,
            &mut Iosb,
        );

        if !NT_SUCCESS(Status) {
            if FlagOn(((*IrpSp).Flags) as u32, SL_ALLOW_RAW_MOUNT) == 0
                && (*(*Vpb).RealDevice).DeviceType == FILE_DEVICE_DISK
            {
                FatNormalizeAndRaiseStatus(IrpContext, Status);
            }
            return Status;
        }
    }

    if Iosb.Information as usize != core::mem::size_of::<ULONG>() {
        ChangeCount = 0;
    }

    if (*TargetDeviceObject).DeviceType == FILE_DEVICE_CD_ROM
        && !FatScanForDataTrack(IrpContext, TargetDeviceObject)
    {
        return STATUS_UNRECOGNIZED_VOLUME;
    }

    StatusPartInfo = FatPerformDevIoCtrl(
        IrpContext,
        IOCTL_DISK_GET_PARTITION_INFO_EX,
        TargetDeviceObject,
        core::ptr::null_mut(),
        0,
        &mut PartitionInformation as *mut _ as PVOID,
        core::mem::size_of::<PARTITION_INFORMATION_EX>() as ULONG,
        false,
        true,
        &mut Iosb,
    );

    SetFlag(&mut ((*IrpContext).Flags), IRP_CONTEXT_FLAG_WAIT);

    FatScanForDismountedVcb(IrpContext);

    BootBcb = core::ptr::null_mut();
    DirentBcb = core::ptr::null_mut();

    Vcb = core::ptr::null_mut();
    VolDo = core::ptr::null_mut();
    MountNewVolume = 0u8;

    let p_status = core::ptr::addr_of_mut!(Status);
    let p_boot_bcb = core::ptr::addr_of_mut!(BootBcb);
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    let p_mount_new_volume = core::ptr::addr_of_mut!(MountNewVolume);
    let p_vcb = core::ptr::addr_of_mut!(Vcb);
    let p_vol_do = core::ptr::addr_of_mut!(VolDo);
    let p_do_a_remount = core::ptr::addr_of_mut!(DoARemount);
    let p_old_vcb = core::ptr::addr_of_mut!(OldVcb);
    let p_real_device = core::ptr::addr_of_mut!(RealDevice);
    let p_we_cleared = core::ptr::addr_of_mut!(WeClearedVerifyRequiredBit);
    let p_root_directory_file = core::ptr::addr_of_mut!(RootDirectoryFile);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        FatAcquireExclusiveGlobal(IrpContext);

        *p_status = IoCreateDevice(
            FatData.DriverObject,
            (core::mem::size_of::<VOLUME_DEVICE_OBJECT>() - core::mem::size_of::<DEVICE_OBJECT>()) as ULONG,
            core::ptr::null_mut(),
            FILE_DEVICE_DISK_FILE_SYSTEM,
            0,
            0u8,
            &mut (*p_vol_do) as *mut _ as *mut PDEVICE_OBJECT,
        );

        if !NT_SUCCESS(*p_status) {
            return;
        }

        if (*TargetDeviceObject).AlignmentRequirement > (**p_vol_do).DeviceObject.AlignmentRequirement {
            (**p_vol_do).DeviceObject.AlignmentRequirement = (*TargetDeviceObject).AlignmentRequirement;
        }

        (**p_vol_do).OverflowQueueCount = 0;
        InitializeListHead(&mut (**p_vol_do).OverflowQueue);

        (**p_vol_do).PostedRequestCount = 0;
        KeInitializeSpinLock(&mut (**p_vol_do).OverflowQueueSpinLock);

        (**p_vol_do).DeviceObject.StackSize = ((*TargetDeviceObject).StackSize + 1) as CCHAR;

        *p_status = FatPerformDevIoCtrl(
            IrpContext,
            IOCTL_DISK_GET_DRIVE_GEOMETRY,
            TargetDeviceObject,
            core::ptr::null_mut(),
            0,
            &mut Geometry as *mut _ as PVOID,
            core::mem::size_of::<DISK_GEOMETRY>() as ULONG,
            false,
            true,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(*p_status) {
            return;
        }

        (**p_vol_do).DeviceObject.SectorSize = Geometry.BytesPerSector as USHORT;

        ClearFlag(&mut ((**p_vol_do).DeviceObject.Flags), DO_DEVICE_INITIALIZING);

        (*Vpb).DeviceObject = (*p_vol_do) as PDEVICE_OBJECT;

        *p_real_device = (*Vpb).RealDevice;

        if FlagOn(((**p_real_device).Flags) as u32, DO_VERIFY_VOLUME) != 0 {
            ClearFlag(&mut ((**p_real_device).Flags), DO_VERIFY_VOLUME);
            *p_we_cleared = 1u8;
        }

        FatInitializeVcb(IrpContext, &mut (**p_vol_do).Vcb, TargetDeviceObject, Vpb, FsDeviceObject);

        *p_vcb = &mut (**p_vol_do).Vcb;

        //  We need to commute errors on CD so that CDFS will get its crack.
        // _SEH2_TRY / _SEH2_EXCEPT(RealDevice is CD-ROM ? EXECUTE_HANDLER : CONTINUE_SEARCH) { NOTHING; }
        let p_boot_sector = core::ptr::addr_of_mut!(BootSector);
        let _ = crate::seh::try_except(&mut (), |_| {
            FatReadVolumeFile(
                IrpContext,
                *p_vcb,
                0,
                core::mem::size_of::<PACKED_BOOT_SECTOR>() as ULONG,
                p_boot_bcb,
                p_boot_sector as *mut PVOID,
            );
        }, |_, _| {
            if (*(*Vpb).RealDevice).DeviceType == FILE_DEVICE_CD_ROM {
                crate::seh::EXCEPTION_EXECUTE_HANDLER
            } else {
                crate::seh::EXCEPTION_CONTINUE_SEARCH
            }
        });

        if (*p_boot_bcb).is_null() || FatIsBootSectorFat(BootSector) == 0 {
            DebugTrace(0, Dbg, b"Not a Fat Volume\n\0".as_ptr() as _, 0);
            *p_status = STATUS_UNRECOGNIZED_VOLUME;
            return;
        }

        FatUnpackBios(&mut (**p_vcb).Bpb, &(*BootSector).PackedBpb as *const _ as *const core::ffi::c_void);

        if NT_SUCCESS(StatusPartInfo)
            && PartitionInformation.PartitionStyle == PARTITION_STYLE_MBR
            && PartitionInformation.__bindgen_anon_1.Mbr.PartitionType == PARTITION_OS2BOOTMGR
            && (**p_vcb).Bpb.Sectors != 0
            && (**p_vcb).Bpb.Sectors < 0x80
        {
            DebugTrace(0, Dbg, b"OS/2 Boot Manager volume detected, volume not mounted. \n\0".as_ptr() as _, 0);
            *p_status = STATUS_UNRECOGNIZED_VOLUME;
            return;
        }

        if !NT_SUCCESS(*p_status) || Geometry.BytesPerSector != (**p_vcb).Bpb.BytesPerSector as u32 {
            *p_status = STATUS_UNRECOGNIZED_VOLUME;
            return;
        }

        if (**p_vcb).Bpb.Sectors != 0 {
            (**p_vcb).Bpb.LargeSectors = 0;
        }

        if IsBpbFat32(&raw const (*BootSector).PackedBpb) {
            (*Vpb).SerialNumber = u32::from_ne_bytes((*(BootSector as PPACKED_BOOT_SECTOR_EX)).Id);
        } else {
            (*Vpb).SerialNumber = u32::from_ne_bytes((*BootSector).Id);

            (**p_vcb).First0x24BytesOfBootSector = FsRtlAllocatePoolWithTag(PagedPool, 0x24, TAG_STASHED_BPB) as PUCHAR;

            core::ptr::copy_nonoverlapping(
                BootSector as *const u8,
                (**p_vcb).First0x24BytesOfBootSector as *mut u8,
                0x24,
            );
        }

        FatUnpinBcb(IrpContext, &mut (*p_boot_bcb));

        FatSetupAllocationSupport(IrpContext, *p_vcb);

        if FatIsFat32(*p_vcb) != 0 {
            if (**p_vcb).Bpb.FsInfoSector >= (**p_vcb).Bpb.ReservedSectors {
                (**p_vcb).Bpb.FsInfoSector = 0;
            }
        }

        //  Create a root Dcb so we can read in the volume label.  If this is FAT32, we can
        //  discover corruption in the FAT chain.
        // _SEH2_TRY / _SEH2_EXCEPT(code == STATUS_FILE_CORRUPT_ERROR ? EXECUTE_HANDLER : CONTINUE_SEARCH)
        if crate::seh::try_except(&mut (), |_| {
            FatCreateRootDcb(IrpContext, *p_vcb);
        }, |_, pointers| {
            if crate::seh::exception_code(pointers) == STATUS_FILE_CORRUPT_ERROR {
                crate::seh::EXCEPTION_EXECUTE_HANDLER
            } else {
                crate::seh::EXCEPTION_CONTINUE_SEARCH
            }
        }).is_err() {
            //  The volume needs to be dirtied, do it now.  Note that at this point we have built
            //  enough of the Vcb to pull this off.
            FatCheckDirtyBit(IrpContext, *p_vcb);

            // Set the dirty bit if it is not set already
            if FlagOn(((**p_vcb).VcbState) as u32, VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                SetFlag(&mut ((**p_vcb).VcbState), crate::types::VCB_STATE_FLAG_MOUNT_IN_PROGRESS);
                FatMarkVolume(IrpContext, *p_vcb, 1 /* VolumeDirty */);
                ClearFlag(&mut ((**p_vcb).VcbState), crate::types::VCB_STATE_FLAG_MOUNT_IN_PROGRESS);
            }

            //  Now keep bailing out ...
            FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
        }

        FatLocateVolumeLabel(IrpContext, *p_vcb, &mut Dirent, &mut (*p_dirent_bcb), &mut ByteOffset as *mut _ as PVBO);

        if !Dirent.is_null() {
            let mut OemBuffer: [u8; 11] = [0; 11];
            let mut OemString: OEM_STRING = core::mem::zeroed();
            let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();

            OemString.Buffer = OemBuffer.as_mut_ptr() as PCHAR;
            OemString.MaximumLength = 11;

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*Dirent).FileName) as *const u8,
                OemString.Buffer as *mut u8,
                11,
            );

            if *OemString.Buffer.offset(0) == FAT_DIRENT_REALLY_0E5 as i8 {
                *OemString.Buffer.offset(0) = 0xe5u8 as i8;
            }

            OemString.Length = 11;
            while OemString.Length > 0 {
                let idx = (OemString.Length - 1) as isize;
                if *OemString.Buffer.offset(idx) != 0x00 && *OemString.Buffer.offset(idx) != 0x20 {
                    break;
                }
                OemString.Length -= 1;
            }

            UnicodeString.MaximumLength = MAXIMUM_VOLUME_LABEL_LENGTH;
            UnicodeString.Buffer = core::ptr::addr_of_mut!((*Vpb).VolumeLabel).cast();

            *p_status = RtlOemStringToCountedUnicodeString(&mut UnicodeString as *mut UNICODE_STRING as *mut _, &OemString as *const OEM_STRING as *const _, 0u8);

            if !NT_SUCCESS(*p_status) {
                return;
            }

            (*Vpb).VolumeLabelLength = UnicodeString.Length;
        } else {
            (*Vpb).VolumeLabelLength = 0;
        }

        (**p_vcb).ChangeCount = ChangeCount;

        Links = FatData.VcbQueue.Flink;
        while Links != &raw const FatData.VcbQueue as *const _ as PLIST_ENTRY {
            *p_old_vcb = ((Links as usize) - core::mem::offset_of!(VCB, VcbLinks)) as PVCB;
            OldVpb = (**p_old_vcb).Vpb;

            if OldVpb == Vpb {
                Links = (*Links).Flink;
                continue;
            }

            if (*OldVpb).SerialNumber == (*Vpb).SerialNumber
                && (**p_old_vcb).VcbCondition == VcbNotMounted
                && (*OldVpb).RealDevice == (*p_real_device)
                && (*OldVpb).VolumeLabelLength == (*Vpb).VolumeLabelLength
                && RtlEqualMemory(
                    core::ptr::addr_of!((*OldVpb).VolumeLabel).cast(),
                    core::ptr::addr_of!((*Vpb).VolumeLabel).cast(),
                    (*Vpb).VolumeLabelLength as u32,
                ) != 0
                && RtlEqualMemory(
                    &(**p_old_vcb).Bpb as *const _ as PVOID,
                    &(**p_vcb).Bpb as *const _ as PVOID,
                    if (**p_vcb).Bpb.SectorsPerFat == 0 {
                        core::mem::size_of::<BIOS_PARAMETER_BLOCK>() as u32
                    } else {
                        core::mem::offset_of!(BIOS_PARAMETER_BLOCK, LargeSectorsPerFat) as u32
                    },
                ) != 0
            {
                *p_do_a_remount = 1u8;
                break;
            }

            Links = (*Links).Flink;
        }

        if (*p_do_a_remount) != 0 {
            let IrpVpb: *mut PVPB;

            DebugTrace(0, Dbg, b"Doing a remount\n\0".as_ptr() as _, 0);

            (**p_vcb).TargetDeviceObject = (**p_old_vcb).TargetDeviceObject;
            (**p_old_vcb).TargetDeviceObject = TargetDeviceObject;

            NT_ASSERT((FlagOn(((**p_old_vcb).VcbState) as u32, VCB_STATE_FLAG_VPB_MUST_BE_FREED) == 0) as u32);

            FatSetVcbCondition(*p_old_vcb, VcbGood);
            (*OldVpb).RealDevice = (*Vpb).RealDevice;
            ClearFlag(&mut ((**p_old_vcb).VcbState), VCB_STATE_VPB_NOT_ON_DEVICE);

            (*(*OldVpb).RealDevice).Vpb = OldVpb;

            (**p_old_vcb).ChangeCount = (**p_vcb).ChangeCount;

            IrpVpb = core::ptr::addr_of_mut!((*IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp))
                .Parameters
                .MountVolume
                .Vpb);

            if *IrpVpb == Vpb {
                *IrpVpb = OldVpb;
            }

            NT_ASSERT(((**p_vcb).Vpb == Vpb) as u32);

            Vpb = core::ptr::null_mut();
            SetFlag(&mut ((**p_vcb).VcbState), VCB_STATE_FLAG_VPB_MUST_BE_FREED);
            FatSetVcbCondition(*p_vcb, VcbBad);

            {
                let mut FileSizes: CC_FILE_SIZES = core::mem::zeroed();

                FileSizes.AllocationSize.QuadPart = 0x40000 + 0x1000;
                FileSizes.FileSize.QuadPart = 0x40000 + 0x1000;
                FileSizes.ValidDataLength = FatMaxLarge;

                DebugTrace(0, Dbg, b"Truncate and reinitialize the volume file\n\0".as_ptr() as _, 0);

                FatInitializeCacheMap(
                    (**p_old_vcb).VirtualVolumeFile,
                    &FileSizes,
                    1u8,
                    &raw const FatData.CacheManagerNoOpCallbacks,
                    (*p_vcb) as *mut core::ffi::c_void,
                );

                FatSetupAllocationSupport(IrpContext, *p_old_vcb);

                FatCheckDirtyBit(IrpContext, *p_old_vcb);

                if FatIsMediaWriteProtected(IrpContext, TargetDeviceObject) != 0 {
                    SetFlag(&mut ((**p_old_vcb).VcbState), VCB_STATE_FLAG_WRITE_PROTECTED);
                } else {
                    ClearFlag(&mut ((**p_old_vcb).VcbState), VCB_STATE_FLAG_WRITE_PROTECTED);
                }
            }

            *p_status = STATUS_SUCCESS;
            return;
        }

        DebugTrace(0, Dbg, b"Mount a new volume\n\0".as_ptr() as _, 0);

        if FatIsFat32(*p_vcb) == 0 {
            let mut TempDirent: DIRENT = core::mem::zeroed();
            let EaFcb: PFCB;

            core::ptr::write_bytes(&mut TempDirent as *mut _ as *mut u8, 0, core::mem::size_of::<DIRENT>());
            core::ptr::copy_nonoverlapping(
                b"EA DATA  SF".as_ptr(),
                core::ptr::addr_of_mut!(TempDirent.FileName).cast(),
                11,
            );

            EaFcb = FatCreateFcb(
                IrpContext,
                *p_vcb,
                (**p_vcb).RootDcb,
                0,
                0,
                &mut TempDirent,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                1,
            );

            SetFlag(&mut ((*EaFcb).FcbState), FCB_STATE_SYSTEM_FILE);

            (**p_vcb).EaFcb = EaFcb;
        }

        FatCheckDirtyBit(IrpContext, *p_vcb);

        if FatIsMediaWriteProtected(IrpContext, TargetDeviceObject) != 0 {
            SetFlag(&mut ((**p_vcb).VcbState), VCB_STATE_FLAG_WRITE_PROTECTED);
        } else {
            ClearFlag(&mut ((**p_vcb).VcbState), VCB_STATE_FLAG_WRITE_PROTECTED);
        }

        if FlagOn(((**p_real_device).Flags) as u32, DO_SYSTEM_BOOT_PARTITION) != 0 {
            SetFlag(&mut ((**p_vcb).VcbState), VCB_STATE_FLAG_BOOT_OR_PAGING_FILE);

            if FlagOn(((**p_vcb).VcbState) as u32, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0 {
                FatToggleMediaEjectDisable(IrpContext, *p_vcb, 1);
            }
        }

        *p_mount_new_volume = 1u8;

        *p_status = STATUS_SUCCESS;

        *p_root_directory_file = (*(**p_vcb).RootDcb).Specific.Dcb.as_ref().DirectoryFile;
        ObReferenceObject((*p_root_directory_file) as *mut core::ffi::c_void);

        ObDereferenceObject(TargetDeviceObject as *mut core::ffi::c_void);
    }, |_, abnormal| {
        DebugUnwind(b"FatMountVolume\0".as_ptr() as *const i8);
        FatUnpinBcb(IrpContext, &mut (*p_boot_bcb));
        FatUnpinBcb(IrpContext, &mut (*p_dirent_bcb));
        if (*p_mount_new_volume) == 0 {
            if !(*p_vcb).is_null() {
                FatCheckForDismount(IrpContext, *p_vcb, 1u8);
                (*IrpContext).Vcb = core::ptr::null_mut();
            } else if !(*p_vol_do).is_null() {
                IoDeleteDevice(&mut (**p_vol_do).DeviceObject);
            }

            if (*p_do_a_remount) != 0 && abnormal {
                FatCheckForDismount(IrpContext, *p_old_vcb, 1u8);
            }
        }
        if (*p_we_cleared) == 1u8 {
            SetFlag(&mut ((**p_real_device).Flags), DO_VERIFY_VOLUME);
        }
        FatReleaseGlobal(IrpContext);
        DebugTrace(-1, Dbg, b"FatMountVolume -> %08lx\n\0".as_ptr() as _, (*p_status) as ULONG);
        {
            unsafe extern "C" {
                fn DbgPrint(Format: *const i8, ...) -> u32;
            }
            DbgPrint(
                c"[FASTFAT] FatMountVolume -> 0x%x\n".as_ptr(),
                (*p_status) as ULONG,
            );
        }
    });

    if !RootDirectoryFile.is_null() {
        FsRtlNotifyVolumeEvent(RootDirectoryFile, FSRTL_VOLUME_MOUNT);
        ObDereferenceObject(RootDirectoryFile as *mut core::ffi::c_void);
    }

    Status
}

// FatInvalidateVolumes — stub
#[no_mangle]
pub unsafe extern "system" fn FatInvalidateVolumes(Irp: PIRP) -> NTSTATUS {
    let _ = Irp;
    FatBugCheck(FAT_BUG_CHECK_FSCTRL, 1, 0);
    unsafe { core::hint::unreachable_unchecked() }
}

// FatCommonFileSystemControl — dispatch by minor function
#[no_mangle]
pub unsafe extern "system" fn FatCommonFileSystemControl(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let MinorFunction = (*IrpSp).MinorFunction;
    let Status: NTSTATUS;
    match MinorFunction as u32 {
        0 /* IRP_MN_USER_FS_REQUEST */ => {
            Status = FatUserFsCtrl(IrpContext, Irp);
        }
        1 /* IRP_MN_MOUNT_VOLUME */ => {
            Status = FatMountVolume(
                IrpContext,
                (*IrpSp).Parameters.MountVolume.DeviceObject,
                (*IrpSp).Parameters.MountVolume.Vpb,
                (*IrpSp).DeviceObject,
            );
            FatCompleteRequest(IrpContext as *mut IRP_CONTEXT, Irp, Status);
        }
        2 /* IRP_MN_VERIFY_VOLUME */ => {
            Status = FatVerifyVolume(IrpContext, Irp);
        }
        _ => {
            FatCompleteRequest(IrpContext as *mut IRP_CONTEXT, Irp, STATUS_INVALID_DEVICE_REQUEST);
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    Status
}

// FatFsdFileSystemControl — FSD dispatch entry point
#[no_mangle]
pub unsafe extern "system" fn FatFsdFileSystemControl(
    _VolumeDeviceObject: *mut DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    DebugTrace(1, Dbg, b"FatFsdFileSystemControl\n\0".as_ptr() as _, 0);

    //  Mount and verify come in without a file object and are allowed to block.
    let Wait: BOOLEAN = if (*IoGetCurrentIrpStackLocation(Irp)).FileObject.is_null() {
        1u8
    } else {
        CanFsdWait(Irp)
    };

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY / _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...))
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_status = core::ptr::addr_of_mut!(Status);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        let IrpSp = IoGetCurrentIrpStackLocation(Irp);

        //  InvalidateVolumes comes in with a FileSystem device object instead
        //  of a volume device object.
        if FatDeviceIsFatFsdo((*IrpSp).DeviceObject) != 0
            && (*IrpSp).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL
            && (*IrpSp).MinorFunction == 0 /* IRP_MN_USER_FS_REQUEST */
            && (*IrpSp).Parameters.FileSystemControl.FsControlCode == FSCTL_INVALIDATE_VOLUMES
        {
            *p_status = FatInvalidateVolumes(Irp);
        } else {
            *p_irp_context = FatCreateIrpContext(Irp, (Wait) as _);
            *p_status = FatCommonFileSystemControl(*p_irp_context, Irp);
        }
    }) {
        //  We had some trouble trying to perform the requested
        //  operation, so we'll abort the I/O request with
        //  the error status that we get back from the
        //  execption code
        Status = crate::fatdata::FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 { IoSetTopLevelIrp(core::ptr::null_mut()); }
    FsRtlExitFileSystem();

    DebugTrace(-1, Dbg, b"FatFsdFileSystemControl -> %08lx\n\0".as_ptr() as _, Status);

    Status
}
