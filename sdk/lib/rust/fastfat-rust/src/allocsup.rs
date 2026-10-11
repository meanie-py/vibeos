#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::base::{PAGE_SIZE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::mm::{RTL_BITMAP};
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::rtl::{RtlClearBits, RtlInitializeBitMap};
use ntkrnl_sys::ddk::{CC_FILE_SIZES, ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExFreePool, ExReleaseResourceLite, _KWAIT_REASON_Executive, _MM_PAGE_PRIORITY_HighPagePriority as HighPagePriority, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoFreeMdl, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, LARGE_MCB, MmUnmapLockedPages, _EVENT_TYPE_NotificationEvent, PCC_FILE_SIZES, PFILE_OBJECT, PIRP, PLARGE_INTEGER, PLARGE_MCB, PMDL, RtlAreBitsClear, RtlFindClearBits, RtlFindLongestRunClear, RtlNumberOfClearBits, RtlSetBits, ULARGE_INTEGER};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CLUSTER_TYPE, DIRENT, FAT_BUG_CHECK_ALLOCSUP, FAT_WINDOW, FCB, IRP_CONTEXT, PBCB, PDIRENT, PFAT_WINDOW};
use ntkrnl_sys::ddk::{FILE_OBJECT, _PUBLIC_BCB as BCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_ALLOCSUP, MDL_MAPPED_TO_SYSTEM_VA, MDL_SOURCE_IS_NONPAGED_POOL, SL_WRITE_THROUGH, STATUS_DISK_FULL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlInitializeLargeMcb, FsRtlNumberOfRunsInLargeMcb, FsRtlTruncateLargeMcb, FsRtlUninitializeLargeMcb};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{STATUS_FILE_CORRUPT_ERROR};
use crate::types::{FAT12_DIRTY_ENTRY, FAT16_DIRTY_ENTRY, FAT32_DIRTY_ENTRY, FAT32_ENTRY_MASK, FAT_CLEAN_ENTRY, FAT_DIRTY_BIT_INDEX, FAT_ENTRY, IRP_CONTEXT_FLAG_DISABLE_DIRTY, IRP_CONTEXT_FLAG_WAIT, PFCB, PIRP_CONTEXT, PLBO, PVCB, VCB_STATE_FLAG_MOUNT_IN_PROGRESS, _CLUSTER_TYPE_FatClusterAvailable as FatClusterAvailable, _CLUSTER_TYPE_FatClusterBad as FatClusterBad, _CLUSTER_TYPE_FatClusterLast as FatClusterLast, _CLUSTER_TYPE_FatClusterNext as FatClusterNext, _CLUSTER_TYPE_FatClusterReserved as FatClusterReserved};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{VCB, RTL_BITMAP_DBG};
use ntkrnl_sys::ddk::{MDL};

extern "system" {
    } extern "C" { fn DebugTrace(level: i32, component: u32, format: *const i8, ...); } extern "system" {
    fn FatRemoveMcbEntry( Vcb: *mut VCB, Mcb: *mut ntkrnl_sys::ddk::LARGE_MCB, Vbo: u32, Bytes: u32, );
    fn FatVerifyIndexIsValid(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, FatIndex: u32);
    fn FatReservedBytes(Bpb: *const core::ffi::c_void) -> u32;
    fn FatBytesPerFat(Bpb: *const core::ffi::c_void) -> u32;
    fn FatLookup12BitEntry(PinnedPage: *mut u8, FatIndex: u32, FatEntry: *mut u32);
    fn FatReadVolumeFile( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, ByteOffset: u32, ByteLength: u32, Bcb: *mut *mut core::ffi::c_void, PinnedPage: *mut *mut u8, );
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
    fn FatBugCheck(value: usize, log: usize, zero: usize);
    fn FatPopUpFileCorrupt(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB);
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: u32);
}
extern "system" {
    fn DbgDoit(value: *mut c_void);
    fn DebugUnwind(FunctionName: *const i8);
    fn FatAddMcbEntry( Vcb: *mut VCB, Mcb: *mut LARGE_MCB, Vbo: u32, Lbo: i64, SectorCount: u32, ) -> i32;
    fn FatLockFreeClusterBitMap(Vcb: *mut VCB);
    fn FatPrepareWriteVolumeFile( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Offset: u32, Length: u32, Bcb: *mut *mut BCB, PinnedFat: *mut *mut c_void, RegularOperation: i32, WriteThrough: i32, );
    fn FatSet12BitEntry(PinnedFat: *mut c_void, FatIndex: u32, FatEntry: u32);
    fn FatUnlockFreeClusterBitMap(Vcb: *mut VCB);
    fn CcRepinBcb(Bcb: *mut BCB);
    fn CcUnpinData(Bcb: *mut BCB);
    fn CcUnpinRepinnedBcb(Bcb: *mut BCB, WriteThrough: i32, IoStatus: *mut _IO_STATUS_BLOCK);
    fn FatGetLboFromIndex(Vcb: *mut VCB, Index: u32) -> i64; // LBO = LONGLONG (fat.h:34); was u32
    fn FatPrefetchPages(IrpContext: *mut IRP_CONTEXT, VirtualVolumeFile: *mut FILE_OBJECT, Page: u32, PrefetchPages: u32) -> u32;
    fn FatLookupMcbEntry( Vcb: PVCB, Mcb: *mut LARGE_MCB, Vbo: u32, Lbo: PLBO, ByteCount: *mut u32, Index: *mut u32, ) -> BOOLEAN;
    fn FatLookupLastMcbEntry( Vcb: PVCB, Mcb: *mut LARGE_MCB, CurrentVbo: *mut u32, CurrentLbo: *mut i64, Runs: *mut u32, ) -> BOOLEAN;
    fn FatGetIndexFromLbo(Vcb: PVCB, Lbo: i64) -> u32;
    fn UInt32x32To64(a: u32, b: u32) -> u64;
    fn FatIsFat32(Vcb: *mut VCB) -> u8;
    // Vbo is `OUT PVBO` — ULONG32, four bytes. This decl said *mut u64 and its
    // locals were widened to match it, which is how a decl that contradicts the
    // header survives: rustc only ever compares a caller against the DECL.
    fn FatGetNextMcbEntry(Vcb: *mut VCB, Mcb: *mut LARGE_MCB, McbIndex: u32, Vbo: *mut u32, Lbo: *mut u64, ByteCount: *mut u32) -> u8;
    fn FatBuildZeroMdl(IrpContext: *mut IRP_CONTEXT, ByteCount: u32) -> *mut MDL;
    fn KeClearEvent(Event: *mut KEVENT);
    fn RtlCheckBit(BitMap: *const RTL_BITMAP, Bit: u32) -> u8;
    fn FatFreeClusters(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, FatIndex: u32, ClusterCount: u32);
    fn FatRootDirectoryLbo(Bpb: *const core::ffi::c_void) -> i64; // LBO
    fn FatRootDirectorySize(Bpb: *const core::ffi::c_void) -> u32;
    fn FatFileAreaLbo(Bpb: *const core::ffi::c_void) -> i64; // LBO
    fn FatNumberOfClusters(Bpb: *const core::ffi::c_void) -> u32;
    // UCHAR in rust_cohab_fat.c: declared u32, the 64-bit arches read 24 undefined bits above
    // it, and a 64 MB FAT16 volume set itself up with 14 clusters (DISK_FULL at file 29).
    fn FatIndexBitSize(Bpb: *const core::ffi::c_void) -> u8;
    fn FatBytesPerCluster(Bpb: *const core::ffi::c_void) -> u32;
    fn FatInitializeCacheMap(FileObject: PFILE_OBJECT, FileSizes: *const CC_FILE_SIZES, Flag: u8, Callbacks: *const core::ffi::c_void, Vcb: *mut VCB);
    fn FatSetDirtyBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut BCB, Vcb: *mut VCB, dirty: u8);
fn FatGetDirentFromFcbOrDcb(irpctx: *mut IRP_CONTEXT, fcb: *mut FCB, arg2: u32, dirent: *mut *mut DIRENT, direntBcb: *mut *mut BCB);
    fn CcIsFileCached(FileObject: *mut FILE_OBJECT) -> u8;
    fn CcSetFileSizes(FileObject: *mut FILE_OBJECT, Sizes: *mut CC_FILE_SIZES);
}

// Ex{Acquire,Release}FastMutexUnsafe are FASTCALL exports on i386 (`@Name@4`, per
// ntoskrnl.spec and the bindgen island); everywhere else FASTCALL is the ordinary
// convention. Declared "system" they asked the linker for `_Name@4`, and
// rust_cohab_fat.c papered over the undefined symbol with stdcall trampolines
// aliased in asm. The declaration was the thing that was wrong.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ExAcquireFastMutexUnsafe(FastMutex: *mut core::ffi::c_void);
    fn ExReleaseFastMutexUnsafe(FastMutex: *mut core::ffi::c_void);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ExAcquireFastMutexUnsafe(FastMutex: *mut core::ffi::c_void);
    fn ExReleaseFastMutexUnsafe(FastMutex: *mut core::ffi::c_void);
}


// Forward declare unknown external functions, constants, and types

// Constants (likely values)
const FAT_CLUSTER_AVAILABLE: u32 = 0;
const FAT_CLUSTER_LAST: u32 = 0x0FFFFFFF;

// Opaque types

const TAG_FAT_WINDOW: u32 = 0x57746146;
const FatMaxLarge: i64 = i64::MAX;

// Module-level debug constant (C: #define Dbg (DEBUG_TRACE_ALLOCSUP))
const Dbg: u32 = DEBUG_TRACE_ALLOCSUP;

// C: #define MAX_CLUSTER_BITMAP_SIZE (1 << 16)
const MAX_CLUSTER_BITMAP_SIZE: u32 = 1 << 16;

// C: #define MAX_ZERO_MDL_SIZE (1*1024*1024)
const MAX_ZERO_MDL_SIZE: u32 = 1 * 1024 * 1024;

// C: #define FatWindowOfCluster(C) (((C) - 2) / MAX_CLUSTER_BITMAP_SIZE)
#[inline(always)]
unsafe fn FatWindowOfCluster(C: u32) -> u32 {
    (C - 2) / MAX_CLUSTER_BITMAP_SIZE
}

// FAT_ENUMERATION_CONTEXT: local pinned-page state for FatLookupFatEntry
#[repr(C)]
struct FAT_ENUMERATION_CONTEXT {
    VboOfPinnedPage: u32,
    Bcb: PBCB,
    PinnedPage: *mut core::ffi::c_void,
}

// C: FatAllocateClusters macro
#[inline(always)]
unsafe fn FatAllocateClusters(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FatIndex: u32,
    ClusterCount: u32,
) {
    if ClusterCount == 1 {
        FatSetFatEntry(IrpContext, Vcb, FatIndex, 0x0FFFFFFFu32 /* FAT_CLUSTER_LAST */);
    } else {
        FatSetFatRun(IrpContext, Vcb, FatIndex, ClusterCount, 1 /* TRUE */);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatSelectBestWindow(Vcb: *mut VCB) -> u32 {
    let mut i: u32;
    let mut Fave: u32 = 0;
    let mut MaxFree: u32 = 0;
    let mut FirstEmpty: u32 = u32::MAX;
    let ClustersPerWindow: u32 = 0x10000;

    NT_ASSERT!((1 != (*Vcb).NumberOfWindows) as i32);

    let windows_ptr = (*Vcb).Windows;
    let number_of_windows = (*Vcb).NumberOfWindows;

    i = 0;
    while i < number_of_windows {
        let cluster_free = (*windows_ptr.add(i as usize)).ClustersFree;
        if cluster_free == ClustersPerWindow {
            if FirstEmpty == u32::MAX {
                FirstEmpty = i;
            }
        } else if cluster_free > MaxFree {
            MaxFree = cluster_free;
            Fave = i;
            if MaxFree >= (ClustersPerWindow >> 1) {
                break;
            }
        }
        i += 1;
    }

    if (MaxFree < (ClustersPerWindow >> 1)) && (FirstEmpty != u32::MAX) {
        Fave = FirstEmpty;
    }

    return Fave;
}

#[no_mangle]
pub unsafe extern "system" fn FatSetupAllocationSupport(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    DebugTrace(1, Dbg, b"FatSetupAllocationSupport\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Vcb = %p\n\0".as_ptr() as *const i8, Vcb as usize);

    (*Vcb).AllocationSupport.RootDirectoryLbo = FatRootDirectoryLbo(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as i64;
    (*Vcb).AllocationSupport.RootDirectorySize = FatRootDirectorySize(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void);
    (*Vcb).AllocationSupport.FileAreaLbo = FatFileAreaLbo(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as i64;
    (*Vcb).AllocationSupport.NumberOfClusters = FatNumberOfClusters(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void);
    (*Vcb).AllocationSupport.FatIndexBitSize = FatIndexBitSize(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as u8;
    (*Vcb).AllocationSupport.LogOfBytesPerSector = FatLogOf((*Vcb).Bpb.BytesPerSector as u32);
    (*Vcb).AllocationSupport.LogOfBytesPerCluster = FatLogOf(FatBytesPerCluster(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void));
    (*Vcb).AllocationSupport.NumberOfFreeClusters = 0;

    let mut ClustersDescribableByFat: u32;
    {
        let numerator = if FatIsFat32(Vcb) != 0 {
            (*Vcb).Bpb.LargeSectorsPerFat
        } else {
            (*Vcb).Bpb.SectorsPerFat as u32
        };
        ClustersDescribableByFat = (((numerator as u64) *
                                   (*Vcb).Bpb.BytesPerSector as u64 * 8) /
                                  (FatIndexBitSize(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as u64)) as u32 - 2;
    }

    if (*Vcb).AllocationSupport.NumberOfClusters > ClustersDescribableByFat {
        (*Vcb).AllocationSupport.NumberOfClusters = ClustersDescribableByFat;
    }

    {
        let mut FileSizes: CC_FILE_SIZES = core::mem::zeroed();
        let size = (FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as i64) +
                   (FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) as i64);
        FileSizes.AllocationSize.QuadPart = size;
        FileSizes.FileSize.QuadPart = size;
        FileSizes.ValidDataLength.QuadPart = FatMaxLarge;

        if (*(*Vcb).VirtualVolumeFile).PrivateCacheMap.is_null() {
            FatInitializeCacheMap(
                (*Vcb).VirtualVolumeFile,
                &FileSizes,
                1,
                // The C passes the no-op callback table; NULL here would hand Cc's lazy
                // writer / read-ahead a NULL AcquireForLazyWrite to call.
                core::ptr::addr_of!(crate::fatdata::FatData.CacheManagerNoOpCallbacks) as *const core::ffi::c_void,
                Vcb
            );
        } else {
            ntkrnl_sys::ddk::CcSetFileSizes((*Vcb).VirtualVolumeFile, &mut FileSizes);
        }
    }

    crate::seh::try_finally(
        &mut (),
        |_| {
            if FatIsFat32(Vcb) != 0 &&
               (*Vcb).AllocationSupport.NumberOfClusters > MAX_CLUSTER_BITMAP_SIZE {
                (*Vcb).NumberOfWindows = ((*Vcb).AllocationSupport.NumberOfClusters + MAX_CLUSTER_BITMAP_SIZE - 1) /
                                         MAX_CLUSTER_BITMAP_SIZE;
            } else {
                (*Vcb).NumberOfWindows = 1;
            }

            (*Vcb).Windows = FsRtlAllocatePoolWithTag(
                ntkrnl_sys::mm::PagedPool,
                ((*Vcb).NumberOfWindows as u32) * (core::mem::size_of::<FAT_WINDOW>() as u32),
                TAG_FAT_WINDOW
            ) as *mut FAT_WINDOW;

            RtlInitializeBitMap(
                &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP,
                core::ptr::null_mut(),
                0
            );

            if (*Vcb).NumberOfWindows > 1 {
                FatExamineFatEntries(
                    IrpContext,
                    Vcb,
                    2,
                    (*Vcb).AllocationSupport.NumberOfClusters + 2 - 1,
                    1,
                    core::ptr::null_mut(),
                    core::ptr::null_mut()
                );

                (*Vcb).CurrentWindow = (*Vcb).Windows.add(FatSelectBestWindow(Vcb) as usize);
            } else {
                (*Vcb).CurrentWindow = (*Vcb).Windows.add(0);
                (*(*Vcb).CurrentWindow).FirstCluster = 2;
                (*(*Vcb).CurrentWindow).LastCluster = (*Vcb).AllocationSupport.NumberOfClusters + 2 - 1;
            }

            FatExamineFatEntries(
                IrpContext,
                Vcb,
                0,
                0,
                0,
                (*Vcb).CurrentWindow,
                core::ptr::null_mut()
            );

            let free_bitmap = &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP;
            let clear_bit = ntkrnl_sys::ddk::RtlFindClearBits(free_bitmap.cast(), 1, 0);
            if clear_bit != core::u32::MAX {
                (*Vcb).ClusterHint = clear_bit + 2;
            } else {
                (*Vcb).ClusterHint = 2;
            }
        },
        |_, abnormal| {
            DebugUnwind(b"FatSetupAllocationSupport\0".as_ptr() as *const i8);

            // If we hit an exception, back out.
            if abnormal {
                FatTearDownAllocationSupport(IrpContext, Vcb);
            }
        },
    );

    return;
}
#[no_mangle]
pub unsafe extern "system" fn FatTearDownAllocationSupport(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    let Vcb = Vcb as PVCB;

    DebugTrace(
        1,
        0, /* Dbg */
        b"FatTearDownAllocationSupport\n\0".as_ptr() as *const i8,
        0usize,
    );
    DebugTrace(
        0,
        0, /* Dbg */
        b"  Vcb = %p\n\0".as_ptr() as *const i8,
        Vcb as usize,
    );

    // PAGED_CODE();  (omitted)

    if !(*Vcb).Windows.is_null() {
        ExFreePool((*Vcb).Windows as *mut core::ffi::c_void);
        (*Vcb).Windows = core::ptr::null_mut();
    }

    if !(*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() {
        ExFreePool((*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer as *mut core::ffi::c_void);
        (*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_mut().Buffer = core::ptr::null_mut();
    }

    FatRemoveMcbEntry(
        (Vcb).cast(),
        core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb),
        0,
        0xFFFFFFFFu32,
    );

    DebugTrace(
        -1,
        0, /* Dbg */
        b"FatTearDownAllocationSupport -> (VOID)\n\0".as_ptr() as *const i8,
        0usize,
    );

    UNREFERENCED_PARAMETER!(IrpContext as usize);
}

#[no_mangle]
pub unsafe extern "system" fn FatLookupFileAllocation(
    IrpContext: PIRP_CONTEXT,
    FcbOrDcb: PFCB,
    Vbo: u32,
    Lbo: PLBO,
    ByteCount: *mut u32,
    Allocated: PBOOLEAN,
    EndOnMax: PBOOLEAN,
    Index: *mut u32,
) {
    let mut CurrentVbo: u32 = 0;
    let mut CurrentLbo: i64 = 0;
    let mut PriorLbo: i64 = 0;

    let mut FirstVboOfCurrentRun: u32 = 0;
    let mut FirstLboOfCurrentRun: i64 = 0;

    let mut LastCluster: BOOLEAN = 0;
    let mut Runs: u32 = 0;

    let Vcb: PVCB;
    let mut FatEntry: FAT_ENTRY = 0;
    let mut BytesPerCluster: u32 = 0;
    let mut BytesOnVolume: u64 = 0;

    let mut Context: FAT_ENUMERATION_CONTEXT = core::mem::zeroed();

    // PAGED_CODE();

    Vcb = (*(FcbOrDcb as *mut FCB)).Vcb;

    DebugTrace(
        1,
        Dbg,
        b"FatLookupFileAllocation\n\0".as_ptr() as *const i8,
        0,
    );
    DebugTrace(
        0,
        Dbg,
        b"  FcbOrDcb  = %p\n\0".as_ptr() as *const i8,
        FcbOrDcb as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  Vbo       = %8lx\n\0".as_ptr() as *const i8,
        (Vbo) as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  pLbo       = %8lx\n\0".as_ptr() as *const i8,
        Lbo as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  pByteCount = %8lx\n\0".as_ptr() as *const i8,
        ByteCount as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  pAllocated = %8lx\n\0".as_ptr() as *const i8,
        Allocated as usize,
    );

    Context.Bcb = core::ptr::null_mut();

    *EndOnMax = 0;

    //
    //  Check the trivial case that the mapping is already in our Mcb.
    //

    if FatLookupMcbEntry(
        Vcb,
        core::ptr::addr_of_mut!((*(FcbOrDcb as *mut FCB)).Mcb),
        Vbo,
        Lbo,
        ByteCount,
        Index,
    ) != 0
    {
        *Allocated = 1;

        NT_ASSERT!(((*ByteCount != 0) as BOOLEAN) as i32);

        //
        //  Detect the overflow case, trim and claim the condition.
        //

        if Vbo.wrapping_add(*ByteCount) == 0 {
            *EndOnMax = 1;
        }

        DebugTrace(
            0,
            Dbg,
            b"Found run in Mcb.\n\0".as_ptr() as *const i8,
            0,
        );
        DebugTrace(
            -1,
            Dbg,
            b"FatLookupFileAllocation -> (VOID)\n\0".as_ptr() as *const i8,
            0,
        );
        return;
    }

    //
    //  Initialize the Vcb, the cluster size, LastCluster, and
    //  FirstLboOfCurrentRun (to be used as an indication of the first
    //  iteration through the following while loop).
    //

    BytesPerCluster = 1u32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

    BytesOnVolume = UInt32x32To64(
        (*Vcb).AllocationSupport.NumberOfClusters,
        BytesPerCluster,
    );

    LastCluster = 0;
    FirstLboOfCurrentRun = 0;

    //
    //  Discard the case that the request extends beyond the end of
    //  allocation.
    //

    if Vbo >= (*(FcbOrDcb as *mut FCB)).Header.AllocationSize.__bindgen_anon_1.LowPart {
        *Allocated = 0;

        DebugTrace(
            0,
            Dbg,
            b"Vbo beyond end of file.\n\0".as_ptr() as *const i8,
            0,
        );
        DebugTrace(
            -1,
            Dbg,
            b"FatLookupFileAllocation -> (VOID)\n\0".as_ptr() as *const i8,
            0,
        );
        return;
    }

    //
    //  The Vbo is beyond the last Mcb entry.
    //

    if FatLookupLastMcbEntry(
        Vcb,
        core::ptr::addr_of_mut!((*(FcbOrDcb as *mut FCB)).Mcb),
        core::ptr::addr_of_mut!(CurrentVbo),
        core::ptr::addr_of_mut!(CurrentLbo),
        core::ptr::addr_of_mut!(Runs),
    ) != 0
    {
        DebugTrace(
            0,
            Dbg,
            b"Current Mcb size = %8lx.\n\0".as_ptr() as *const i8,
            (CurrentVbo.wrapping_add(1)) as usize,
        );

        CurrentVbo = CurrentVbo.wrapping_sub(BytesPerCluster.wrapping_sub(1));
        CurrentLbo = CurrentLbo.wrapping_sub((BytesPerCluster.wrapping_sub(1)) as i64);

        //
        //  Convert an index to a count.
        //

        Runs = Runs.wrapping_add(1);
    } else {
        DebugTrace(0, Dbg, b"Mcb empty.\n\0".as_ptr() as *const i8, 0);

        //
        //  Check for an FcbOrDcb that has no allocation
        //

        if (*(FcbOrDcb as *mut FCB)).FirstClusterOfFile == 0 {
            *Allocated = 0;

            DebugTrace(
                0,
                Dbg,
                b"File has no allocation.\n\0".as_ptr() as *const i8,
                0,
            );
            DebugTrace(
                -1,
                Dbg,
                b"FatLookupFileAllocation -> (VOID)\n\0".as_ptr() as *const i8,
                0,
            );
            return;
        } else {
            CurrentVbo = 0;
            CurrentLbo = (FatGetLboFromIndex(
                (Vcb).cast(),
                (*(FcbOrDcb as *mut FCB)).FirstClusterOfFile,
            )) as i64;
            FirstVboOfCurrentRun = CurrentVbo;
            FirstLboOfCurrentRun = CurrentLbo;

            Runs = 0;

            DebugTrace(
                0,
                Dbg,
                b"First Lbo of file = %8lx\n\0".as_ptr() as *const i8,
                (CurrentLbo) as usize,
            );
        }
    }

    //
    //  Now we know that we are looking up a valid Vbo, but it is
    //  not in the Mcb.  We have to go to the Fat, and update
    //  the Mcb as we go.
    //

    *Allocated = 0;

    let p_Context = core::ptr::addr_of_mut!(Context);
    crate::seh::try_finally(
        &mut (),
        |_| {
            'sehtry: {
                FatEntry = FatGetIndexFromLbo(Vcb, CurrentLbo) as FAT_ENTRY;

                //
                //  ASSERT that CurrentVbo and CurrentLbo are now cluster aligned.
                //

                NT_ASSERT!(((((CurrentLbo
                        - (*Vcb).AllocationSupport.FileAreaLbo)
                        % (BytesPerCluster as i64)
                        == 0)
                        && (CurrentVbo % BytesPerCluster == 0)) as BOOLEAN) as i32);

                //
                //  Starting from the first Vbo after the last Mcb entry, scan through
                //  the Fat looking for our Vbo.
                //

                while LastCluster == 0 {
                    //
                    //  Get the next fat entry, and update our Current variables.
                    //

                    FatLookupFatEntry(
                        IrpContext,
                        (Vcb).cast(),
                        FatEntry,
                        core::ptr::addr_of_mut!(FatEntry),
                        p_Context,
                    );

                    PriorLbo = CurrentLbo;
                    CurrentLbo = (FatGetLboFromIndex((Vcb).cast(), FatEntry)) as i64;
                    CurrentVbo = CurrentVbo.wrapping_add(BytesPerCluster);

                    match FatInterpretClusterType((Vcb).cast(), FatEntry) {
                        //
                        //  Check for a break in the Fat allocation chain.
                        //

                        FatClusterAvailable | FatClusterReserved | FatClusterBad => {
                            DebugTrace(
                                0,
                                Dbg,
                                b"Break in allocation chain, entry = %d\n\0".as_ptr() as *const i8,
                                (FatEntry) as usize,
                            );
                            DebugTrace(
                                -1,
                                Dbg,
                                b"FatLookupFileAllocation -> Fat Corrupt.  Raise Status.\n\0".as_ptr()
                                    as *const i8,
                                0,
                            );

                            FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
                            FatRaiseStatus(IrpContext, (STATUS_FILE_CORRUPT_ERROR) as u32);
                        }

                        //
                        //  If this is the last cluster, we must update the Mcb and
                        //  exit the loop.
                        //

                        FatClusterLast => {
                            DebugTrace(
                                0,
                                Dbg,
                                b"Read last cluster of file.\n\0".as_ptr() as *const i8,
                                0,
                            );

                            //
                            //  Detect the case of the maximal file.
                            //

                            if CurrentVbo == 0 {
                                *EndOnMax = 1;
                                CurrentVbo = CurrentVbo.wrapping_sub(1);
                            }

                            LastCluster = 1;

                            if FirstLboOfCurrentRun != 0 {
                                DebugTrace(
                                    0,
                                    Dbg,
                                    b"Adding a run to the Mcb.\n\0".as_ptr() as *const i8,
                                    0,
                                );
                                DebugTrace(
                                    0,
                                    Dbg,
                                    b"  Vbo    = %08lx.\n\0".as_ptr() as *const i8,
                                    (FirstVboOfCurrentRun) as usize,
                                );
                                DebugTrace(
                                    0,
                                    Dbg,
                                    b"  Lbo    = %08lx.\n\0".as_ptr() as *const i8,
                                    (FirstLboOfCurrentRun) as usize,
                                );
                                DebugTrace(
                                    0,
                                    Dbg,
                                    b"  Length = %08lx.\n\0".as_ptr() as *const i8,
                                    (CurrentVbo.wrapping_sub(FirstVboOfCurrentRun)) as usize,
                                );

                                FatAddMcbEntry(
                                    Vcb,
                                    core::ptr::addr_of_mut!(
                                        (*(FcbOrDcb as *mut FCB)).Mcb
                                    ),
                                    FirstVboOfCurrentRun,
                                    FirstLboOfCurrentRun,
                                    CurrentVbo.wrapping_sub(FirstVboOfCurrentRun),
                                );

                                Runs = Runs.wrapping_add(1);
                            }

                            //
                            //  Being at the end of allocation, make sure we have found
                            //  the Vbo.
                            //

                            if Vbo == MAXULONG.wrapping_sub(1) {
                                *Allocated = 0;

                                (*(FcbOrDcb as *mut FCB))
                                    .Header
                                    .AllocationSize
                                    .QuadPart = CurrentVbo as i64;

                                DebugTrace(
                                    0,
                                    Dbg,
                                    b"New file allocation size = %08lx.\n\0".as_ptr() as *const i8,
                                    (CurrentVbo) as usize,
                                );
                                break 'sehtry; // try_return(NOTHING)
                            }

                            //
                            //  We will lie ever so slightly if we really terminated on the
                            //  maximal byte of a file.
                            //

                            if Vbo >= CurrentVbo && *EndOnMax == 0 {
                                *Allocated = 0;
                                break 'sehtry; // try_return(NOTHING)
                            }
                        }

                        //
                        //  This is a continuation in the chain.
                        //

                        FatClusterNext => {
                            //
                            //  This is the loop check.
                            //

                            let BytesOnVolume_HighPart: u32 = (BytesOnVolume >> 32) as u32;
                            let BytesOnVolume_LowPart: u32 = BytesOnVolume as u32;

                            if CurrentVbo == 0
                                || (BytesOnVolume_HighPart == 0
                                    && CurrentVbo > BytesOnVolume_LowPart)
                            {
                                FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
                                FatRaiseStatus(IrpContext, (STATUS_FILE_CORRUPT_ERROR) as u32);
                            }

                            if PriorLbo.wrapping_add(BytesPerCluster as i64) != CurrentLbo {
                                //
                                //  Note that on the first time through the loop
                                //  (FirstLboOfCurrentRun == 0), we don't add the
                                //  run to the Mcb since it corresponds to the last
                                //  run already stored in the Mcb.
                                //

                                if FirstLboOfCurrentRun != 0 {
                                    DebugTrace(
                                        0,
                                        Dbg,
                                        b"Adding a run to the Mcb.\n\0".as_ptr() as *const i8,
                                        0,
                                    );
                                    DebugTrace(
                                        0,
                                        Dbg,
                                        b"  Vbo    = %08lx.\n\0".as_ptr() as *const i8,
                                        (FirstVboOfCurrentRun) as usize,
                                    );
                                    DebugTrace(
                                        0,
                                        Dbg,
                                        b"  Lbo    = %08lx.\n\0".as_ptr() as *const i8,
                                        (FirstLboOfCurrentRun) as usize,
                                    );
                                    DebugTrace(
                                        0,
                                        Dbg,
                                        b"  Length = %08lx.\n\0".as_ptr() as *const i8,
                                        (CurrentVbo.wrapping_sub(FirstVboOfCurrentRun)) as usize,
                                    );

                                    FatAddMcbEntry(
                                        Vcb,
                                        core::ptr::addr_of_mut!(
                                            (*(FcbOrDcb as *mut FCB)).Mcb
                                        ),
                                        FirstVboOfCurrentRun,
                                        FirstLboOfCurrentRun,
                                        CurrentVbo.wrapping_sub(FirstVboOfCurrentRun),
                                    );

                                    Runs = Runs.wrapping_add(1);
                                }

                                //
                                //  Since we are at a run boundary, check if the run
                                //  we just added encompasses the desired Vbo.
                                //

                                if CurrentVbo > Vbo {
                                    LastCluster = 1;
                                } else {
                                    FirstVboOfCurrentRun = CurrentVbo;
                                    FirstLboOfCurrentRun = CurrentLbo;
                                }
                            }
                        }

                        _ => {
                            {
                                extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
                                DbgPrint(b"[FASTFAT] LookupAlloc ILLEGAL entry=%08lx fcb=%p vbo=%08lx curvbo=%08lx bits=%u\n\0".as_ptr() as *const i8,
                                         FatEntry, FcbOrDcb, Vbo, CurrentVbo,
                                         (*(*FcbOrDcb).Vcb).AllocationSupport.FatIndexBitSize as u32);
                            }
                            FatBugCheck(0, 0, 0);
                        }
                    } // match
                } // while

                //
                //  Load up the return parameters.
                //

                *Allocated = 1;

                *Lbo = FirstLboOfCurrentRun + (Vbo.wrapping_sub(FirstVboOfCurrentRun) as i64);

                *ByteCount = CurrentVbo.wrapping_sub(Vbo);

                if !Index.is_null() {
                    //
                    //  Note that Runs only needs to be accurate with respect to where we
                    //  ended.
                    //

                    *Index = Runs.wrapping_sub(1);
                }

            // try_exit: NOTHING;
            }
        },
        |_, _| {
            DebugUnwind(b"FatLookupFileAllocation\0".as_ptr() as *const i8);

            // We are done reading the Fat, so unpin the last page of fat
            // that is hanging around.
            FatUnpinBcb((IrpContext).cast(), core::ptr::addr_of_mut!((*p_Context).Bcb) as *mut *mut core::ffi::c_void);

            DebugTrace(
                -1,
                Dbg,
                b"FatLookupFileAllocation -> (VOID)\n\0".as_ptr() as *const i8,
                0,
            );
        },
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatAddFileAllocation(
    IrpContext: *mut IRP_CONTEXT,
    FcbOrDcb: *mut FCB,
    FileObject: *mut FILE_OBJECT,
    DesiredAllocationSize: u32,
) {
    // Gap constants (unknown values; preserve names)
    const Dbg: u32 = 0; // DEBUG_TRACE_ALLOCSUP
    const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;
    const FcbGood: u32 = 1;
    const FCB_STATE_FLUSH_FAT: u32 = 0x10;

    // Locals
    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut new_mcb: LARGE_MCB = core::mem::zeroed();
    let mut mcb_to_cleanup: *mut LARGE_MCB = core::ptr::null_mut();
    let mut dirent: *mut DIRENT = core::ptr::null_mut();
    let mut new_allocation: u32 = 0;
    let mut bcb: *mut BCB = core::ptr::null_mut();
    let mut unwind_we_allocated_disk_space: i32 = 0;
    let mut unwind_allocation_size_set: i32 = 0;
    let mut unwind_cache_manager_informed: i32 = 0;
    let mut unwind_we_initialized_mcb: i32 = 0;

    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatAddFileAllocation\n\0".as_ptr() as *const i8, (0i32) as usize);
    DebugTrace(0, Dbg, b"  FcbOrDcb  =             %p\n\0".as_ptr() as *const i8, FcbOrDcb as usize);
    DebugTrace(
        0,
        Dbg,
        b"  DesiredAllocationSize = %8lx\n\0".as_ptr() as *const i8,
        DesiredAllocationSize as usize,
    );

    vcb = ((*FcbOrDcb).Vcb).cast();

    if (*FcbOrDcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
        FatLookupFileAllocationSize(IrpContext, FcbOrDcb);
    }

    if DesiredAllocationSize <= (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
        DebugTrace(0, Dbg, b"Desired size within current allocation.\n\0".as_ptr() as *const i8, (0i32) as usize);
        DebugTrace(-1, Dbg, b"FatAddFileAllocation -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
        return;
    }

    DebugTrace(
        0,
        Dbg,
        b"InitialAllocation = %08lx.\n\0".as_ptr() as *const i8,
        (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart as usize,
    );

    // The finally reads the unwind state through raw pointers.
    let p_dirent = core::ptr::addr_of_mut!(dirent);
    let p_bcb = core::ptr::addr_of_mut!(bcb);
    let p_new_allocation = core::ptr::addr_of_mut!(new_allocation);
    let p_mcb_to_cleanup = core::ptr::addr_of_mut!(mcb_to_cleanup);
    let p_unwind_we_allocated_disk_space = core::ptr::addr_of_mut!(unwind_we_allocated_disk_space);
    let p_unwind_allocation_size_set = core::ptr::addr_of_mut!(unwind_allocation_size_set);
    let p_unwind_cache_manager_informed = core::ptr::addr_of_mut!(unwind_cache_manager_informed);
    let p_unwind_we_initialized_mcb = core::ptr::addr_of_mut!(unwind_we_initialized_mcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            if (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart == 0 {
                let mut first_lbo_of_file: i64 = 0;

                ntkrnl_sys::ASSERT!((*FcbOrDcb).FcbCondition == FcbGood);

                FatGetDirentFromFcbOrDcb(
                    IrpContext,
                    FcbOrDcb,
                    (0u8) as _,
                    &mut dirent,
                    &mut bcb,
                );

                FatSetDirtyBcb(IrpContext, bcb, vcb, ((1u8) as i32) as _);

                let mut desired = DesiredAllocationSize;
                FatAllocateDiskSpace(
                    IrpContext,
                    vcb,
                    0,
                    &mut desired,
                    0u8,
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                );
                new_allocation = desired;

                unwind_we_allocated_disk_space = 1;
                mcb_to_cleanup = &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB;

                FatLookupMcbEntry(
                    (*FcbOrDcb).Vcb,
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    0,
                    &mut first_lbo_of_file,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                DebugTrace(
                    0,
                    Dbg,
                    b"First Lbo of file will be %08lx.\n\0".as_ptr() as *const i8,
                    first_lbo_of_file as usize,
                );

                (*FcbOrDcb).FirstClusterOfFile = FatGetIndexFromLbo(vcb, first_lbo_of_file);
                (*dirent).FirstClusterOfFile = (*FcbOrDcb).FirstClusterOfFile as u16;

                if FatIsFat32(vcb) != 0 {
                    (*dirent).__bindgen_anon_1.bindgen_union_field =
                        ((*FcbOrDcb).FirstClusterOfFile >> 16) as u16;
                }

            } else {
                let mut last_allocated_lbo: i64 = 0;
                let mut dont_care: u32 = 0;
                let hint: u32;

                FatLookupLastMcbEntry(
                    (*FcbOrDcb).Vcb,
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    &mut dont_care,
                    &mut last_allocated_lbo,
                    core::ptr::null_mut(),
                );

                new_allocation =
                    DesiredAllocationSize - (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart;

                FsRtlInitializeLargeMcb(&mut new_mcb, PagedPool);
                unwind_we_initialized_mcb = 1;
                mcb_to_cleanup = &mut new_mcb as *mut LARGE_MCB;

                hint = if last_allocated_lbo != -1 {
                    FatGetIndexFromLbo(vcb, last_allocated_lbo + 1)
                } else {
                    0
                };

                FatAllocateDiskSpace(
                    IrpContext,
                    vcb,
                    hint,
                    &mut new_allocation,
                    0u8,
                    &mut new_mcb as *mut LARGE_MCB,
                );

                unwind_we_allocated_disk_space = 1;
            }

            let low_part = &mut (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
            *low_part += new_allocation;

            if *low_part == 0 {
                new_allocation -= 1;
                *low_part = 0xffffffff;
            }

            unwind_allocation_size_set = 1;

            if !FileObject.is_null() && CcIsFileCached(FileObject) != 0 {
                CcSetFileSizes(
                    FileObject,
                    core::ptr::addr_of_mut!((*FcbOrDcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                );
                unwind_cache_manager_informed = 1;
            }

            if (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart != new_allocation {
                FatMergeAllocation(
                    IrpContext,
                    (vcb).cast(),
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    &mut new_mcb as *mut LARGE_MCB,
                );
            }

        },
        |_, abnormal| {
            DebugUnwind(b"FatAddFileAllocation\0".as_ptr() as *const i8);

            // Give FlushFileBuffer/Cleanup a clue here, regardless of success/fail.
            SetFlag!((*FcbOrDcb).FcbState, FCB_STATE_FLUSH_FAT);

            // If we were dogged trying to complete this operation, back things out.
            if abnormal {
                if *p_unwind_allocation_size_set != 0 {
                    (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart -= *p_new_allocation;
                }

                if *p_unwind_cache_manager_informed != 0 {
                    CcSetFileSizes(
                        FileObject,
                        core::ptr::addr_of_mut!((*FcbOrDcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                    );
                }

                if (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart == 0 {
                    if !(*p_dirent).is_null() {
                        (*FcbOrDcb).FirstClusterOfFile = 0;
                        (**p_dirent).FirstClusterOfFile = 0;

                        if FatIsFat32(vcb) != 0 {
                            (**p_dirent).__bindgen_anon_1.bindgen_union_field = 0;
                        }
                    }
                }

                // ... and drop the dirent Bcb if we got it.
                FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);

                crate::seh::try_finally(
                    &mut (),
                    |_| {
                        // Note this can re-raise.
                        if *p_unwind_we_allocated_disk_space != 0 {
                            FatDeallocateDiskSpace(IrpContext, vcb, *p_mcb_to_cleanup, (0i32) as u8);
                        }
                    },
                    |_, _| {
                        // Always clean up the non-initial allocation temporary Mcb,
                        // otherwise truncate the Fcb's Mcb away.
                        if *p_unwind_we_initialized_mcb != 0 {
                            FsRtlUninitializeLargeMcb(*p_mcb_to_cleanup);
                        } else if !(*p_mcb_to_cleanup).is_null() {
                            FsRtlTruncateLargeMcb(*p_mcb_to_cleanup, 0);
                        }
                    },
                );
            }

            DebugTrace(-1, Dbg, b"FatAddFileAllocation -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
        },
    );

    // Non-exceptional cleanup we always want to do (was missing from the
    // translation: leaked the dirent Bcb pin and the temporary Mcb's pool).
    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(bcb) as *mut *mut core::ffi::c_void);

    if unwind_we_initialized_mcb != 0 {
        FsRtlUninitializeLargeMcb(&mut new_mcb);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatTruncateFileAllocation(
    IrpContext: PIRP_CONTEXT,
    FcbOrDcb: PFCB,
    mut DesiredAllocationSize: ULONG,
) {
    // local constants (guesses for undefined symbols)
    const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;
    const FCB_STATE_ZERO_ON_DEALLOCATION: u32 = 0x00080000;
    const FCB_STATE_FLUSH_FAT: u32 = 0x10;
    const FcbGood: i32 = 1;
    let _local_dbg: i32 = 0; // DEBUG_TRACE_ALLOCSUP (shadowing intentional)

    let Vcb: PVCB = (*FcbOrDcb).Vcb;
    let mut Bcb: PBCB = core::ptr::null_mut();
    let mut RemainingMcb: LARGE_MCB = core::mem::zeroed();
    let mut BytesPerCluster: ULONG = 0;
    let mut Dirent: PDIRENT = core::ptr::null_mut();
    let mut UpdatedDirent: BOOLEAN = 0;

    let mut UnwindInitialAllocationSize: ULONG = 0;
    let mut UnwindInitialFirstClusterOfFile: ULONG = 0;
    let mut UnwindWeAllocatedMcb: BOOLEAN = 0;

    // PAGED_CODE(); (skipped)

    DebugTrace(1, (Dbg) as u32, b"FatTruncateFileAllocation\n\0".as_ptr() as *const i8, (0i32) as usize);
    DebugTrace(0, (Dbg) as u32, b"  FcbOrDcb  =             %p\n\0".as_ptr() as *const i8, FcbOrDcb as usize);
    DebugTrace(0, (Dbg) as u32, b"  DesiredAllocationSize = %8lx\n\0".as_ptr() as *const i8, DesiredAllocationSize as usize);

    NT_ASSERT!(if (*FcbOrDcb).FcbCondition == (FcbGood) as u32 { 1 } else { 0 });

    if (*FcbOrDcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
        FatLookupFileAllocationSize(IrpContext, FcbOrDcb);
    }

    BytesPerCluster = 1u32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

    if DesiredAllocationSize != 0 {
        DesiredAllocationSize = (DesiredAllocationSize.wrapping_add(BytesPerCluster - 1)) & !(BytesPerCluster - 1);

        if DesiredAllocationSize == 0
            || DesiredAllocationSize >= (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart
        {
            DebugTrace(0, (Dbg) as u32, b"Desired size within current allocation.\n\0".as_ptr() as *const i8, (0i32) as usize);
            DebugTrace(-1, (Dbg) as u32, b"FatTruncateFileAllocation -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
            return;
        }
    }

    if DesiredAllocationSize == (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
        DebugTrace(0, (Dbg) as u32, b"Desired size equals current allocation.\n\0".as_ptr() as *const i8, (0i32) as usize);
        DebugTrace(-1, (Dbg) as u32, b"FatTruncateFileAllocation -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
        return;
    }

    UnwindInitialAllocationSize = (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
    UnwindInitialFirstClusterOfFile = (*FcbOrDcb).FirstClusterOfFile;

    let p_Dirent = core::ptr::addr_of_mut!(Dirent);
    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    let p_UpdatedDirent = core::ptr::addr_of_mut!(UpdatedDirent);
    let p_RemainingMcb = core::ptr::addr_of_mut!(RemainingMcb);
    let p_UnwindWeAllocatedMcb = core::ptr::addr_of_mut!(UnwindWeAllocatedMcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            (*FcbOrDcb).Header.AllocationSize.QuadPart = DesiredAllocationSize as i64;

            if DesiredAllocationSize == 0 {
                NT_ASSERT!(if (*FcbOrDcb).FcbCondition == (FcbGood) as u32 { 1 } else { 0 });

                FatGetDirentFromFcbOrDcb(
                    IrpContext,
                    FcbOrDcb,
                    (0u8) as _,
                    &mut Dirent as *mut PDIRENT,
                    (&mut Bcb as *mut PBCB).cast::<*mut BCB>(),
                );

                (*Dirent).FirstClusterOfFile = 0;

                if FatIsFat32((Vcb).cast()) != 0 {
                    (*Dirent).__bindgen_anon_1.bindgen_union_field = 0;
                }

                (*FcbOrDcb).FirstClusterOfFile = 0;

                FatSetDirtyBcb((IrpContext).cast(), Bcb as *mut BCB, (Vcb).cast(), 1);
                UpdatedDirent = 1;

                FatDeallocateDiskSpace(
                    IrpContext,
                    (Vcb).cast(),
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    if ((*FcbOrDcb).FcbState & FCB_STATE_ZERO_ON_DEALLOCATION) != 0 { 1 } else { 0 },
                );

                FatRemoveMcbEntry(
                    ((*FcbOrDcb).Vcb).cast(),
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    0,
                    0xFFFFFFFFu32,
                );
            } else {
                FsRtlInitializeLargeMcb(&mut RemainingMcb as *mut LARGE_MCB, ntkrnl_sys::mm::PagedPool);
                UnwindWeAllocatedMcb = 1;

                FatSplitAllocation(
                    IrpContext,
                    Vcb,
                    &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB,
                    DesiredAllocationSize,
                    &mut RemainingMcb as *mut LARGE_MCB,
                );

                FatDeallocateDiskSpace(
                    IrpContext,
                    (Vcb).cast(),
                    &mut RemainingMcb as *mut LARGE_MCB,
                    if ((*FcbOrDcb).FcbState & FCB_STATE_ZERO_ON_DEALLOCATION) != 0 { 1 } else { 0 },
                );

                FsRtlUninitializeLargeMcb(&mut RemainingMcb as *mut LARGE_MCB);
            }
        },
        |_, abnormal| {
            DebugUnwind(b"FatTruncateFileAllocation\0".as_ptr() as *const i8);

            if abnormal {
                (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart = UnwindInitialAllocationSize;

                if DesiredAllocationSize == 0 && !(*p_Dirent).is_null() {
                    if *p_UpdatedDirent != 0 {
                        FatRemoveMcbEntry((Vcb).cast(), &mut (*FcbOrDcb).Mcb as *mut LARGE_MCB, 0, 0xFFFFFFFFu32);
                        (*FcbOrDcb).Header.AllocationSize.QuadPart = 0;
                    } else if (*FcbOrDcb).FirstClusterOfFile == 0 {
                        (**p_Dirent).FirstClusterOfFile = UnwindInitialFirstClusterOfFile as u16;
                        if FatIsFat32((Vcb).cast()) != 0 {
                            (**p_Dirent).__bindgen_anon_1.bindgen_union_field = (UnwindInitialFirstClusterOfFile >> 16) as u16;
                        }
                        (*FcbOrDcb).FirstClusterOfFile = UnwindInitialFirstClusterOfFile;
                    }
                }

                if *p_UnwindWeAllocatedMcb != 0 {
                    FsRtlUninitializeLargeMcb(p_RemainingMcb);
                }
            }

            FatUnpinBcb((IrpContext).cast(), p_Bcb as *mut *mut core::ffi::c_void);
            SetFlag!((*FcbOrDcb).FcbState, FCB_STATE_FLUSH_FAT);
            DebugTrace(-1, (Dbg) as u32, b"FatTruncateFileAllocation -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
        },
    );
}
#[no_mangle]
pub unsafe extern "system" fn FatLookupFileAllocationSize(
    IrpContext: *mut IRP_CONTEXT,
    FcbOrDcb: *mut FCB,
) {
    // PAGED_CODE();  (macro expansion ignored)
    let mut Lbo: i64 = 0;
    let mut ByteCount: u32 = 0;
    let mut DontCare: u8 = 0;

    DebugTrace(1, Dbg, b"FatLookupAllocationSize\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  FcbOrDcb  =      %p\n\0".as_ptr() as *const i8, FcbOrDcb as usize);

    FatLookupFileAllocation(
        IrpContext,
        FcbOrDcb,
        (MAXULONG - 1) as u32,
        core::ptr::addr_of_mut!(Lbo),
        core::ptr::addr_of_mut!(ByteCount),
        core::ptr::addr_of_mut!(DontCare),
        core::ptr::addr_of_mut!(DontCare),
        core::ptr::null_mut(),
    );

    if (*FcbOrDcb).Header.FileSize.__bindgen_anon_1.LowPart > (*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
        FatPopUpFileCorrupt(IrpContext, FcbOrDcb);
        FatRaiseStatus(IrpContext, (STATUS_FILE_CORRUPT_ERROR as i32) as u32);
    }

    DebugTrace(-1, Dbg, b"FatLookupFileAllocationSize -> (VOID)\n\0".as_ptr() as *const i8, 0);
    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatAllocateDiskSpace(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    AbsoluteClusterHint: ULONG,
    ByteCount: *mut ULONG,
    ExactMatchRequired: BOOLEAN,
    Mcb: PLARGE_MCB,
) {
    let mut LogOfBytesPerCluster: UCHAR = 0;
    let mut BytesPerCluster: ULONG = 0;
    let mut StartingCluster: ULONG = 0;
    let mut ClusterCount: ULONG = 0;
    let mut WindowRelativeHint: ULONG = 0;
    #[cfg(DBG)]
    let mut PreviousClear: ULONG = 0;
    let mut Window: PFAT_WINDOW = core::ptr::null_mut();
    let mut Wait: BOOLEAN = 0;
    let mut Result: BOOLEAN = 1;

    PAGED_CODE!();

    let IrpContext = IrpContext as *mut IRP_CONTEXT;
    let Vcb = Vcb as *mut VCB;

    // C: DebugTrace(+1, Dbg, ...) -- a trace, compiled out unless tracing is on. As an
    // unconditional DbgPrint it put two serial lines on every allocation (setup's file copy).
    DebugTrace(1, Dbg, b"FatAllocateDiskSpace\n\0".as_ptr() as *const i8, 0);

    ntkrnl_sys::ASSERT!((AbsoluteClusterHint <= (*Vcb).AllocationSupport.NumberOfClusters + 2) && (1 != AbsoluteClusterHint));

    if *ByteCount == 0 {
        DebugTrace(0, Dbg, b"Nothing to allocate.\n\0".as_ptr() as *const i8, 0);
        DebugTrace(-1, Dbg, b"FatAllocateDiskSpace -> (VOID)\n\0".as_ptr() as *const i8, 0);
        return;
    }

    LogOfBytesPerCluster = (*Vcb).AllocationSupport.LogOfBytesPerCluster;
    BytesPerCluster = 1u32 << LogOfBytesPerCluster;
    *ByteCount = (*ByteCount + (BytesPerCluster - 1)) & !(BytesPerCluster - 1);

    if *ByteCount == 0 {
        *ByteCount = 0xffffffff;
        ClusterCount = 1u32 << (32u32 - LogOfBytesPerCluster as u32);
    } else {
        ClusterCount = *ByteCount >> LogOfBytesPerCluster;
    }

    // _Analysis_assume_(ClusterCount > 0);

    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource), 1);
    FatLockFreeClusterBitMap(Vcb);

    if ClusterCount <= (*Vcb).AllocationSupport.NumberOfFreeClusters {
        (*Vcb).AllocationSupport.NumberOfFreeClusters -= ClusterCount;
    } else {
        FatUnlockFreeClusterBitMap(Vcb);
        ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));
        DbgPrint(
            b"Disk Full.  Raise Status. want=%lu clusters (ByteCount %08lx) free=%lu of %lu hint=%lu\n\0".as_ptr() as *const i8,
            ClusterCount, *ByteCount, (*Vcb).AllocationSupport.NumberOfFreeClusters,
            (*Vcb).AllocationSupport.NumberOfClusters, AbsoluteClusterHint,
        );
        FatRaiseStatus(IrpContext, (STATUS_DISK_FULL) as u32);
    }

    if (0 != AbsoluteClusterHint) && (AbsoluteClusterHint < ((*Vcb).AllocationSupport.NumberOfClusters + 2)) {
        if (*Vcb).NumberOfWindows > 1 {
            if AbsoluteClusterHint < (*(*Vcb).CurrentWindow).FirstCluster
                || AbsoluteClusterHint > (*(*Vcb).CurrentWindow).LastCluster
            {
                let BucketNum: ULONG = FatWindowOfCluster(AbsoluteClusterHint);
                ntkrnl_sys::ASSERT!(BucketNum < (*Vcb).NumberOfWindows);

                FatUnlockFreeClusterBitMap(Vcb);
                ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));
                ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource), 1);
                FatLockFreeClusterBitMap(Vcb);

                Window = (*Vcb).Windows.add(BucketNum as usize);

                if Window != (*Vcb).CurrentWindow {
                    let p_Wait = core::ptr::addr_of_mut!(Wait);
                    crate::seh::try_finally(
                        &mut (),
                        |_| {
                            *p_Wait = if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0 { 1 } else { 0 };
                            SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

                            // Change to the new window and scan it.
                            FatExamineFatEntries(
                                IrpContext,
                                Vcb,
                                0,
                                0,
                                0,
                                Window,
                                core::ptr::null_mut(),
                            );
                        },
                        |_, abnormal| {
                            if *p_Wait == 0 {
                                ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);
                            }
                            if abnormal {
                                // We raised failing to pick up the FAT chunk for this
                                // window move: release and return the cluster count.
                                (*Vcb).AllocationSupport.NumberOfFreeClusters += ClusterCount;
                                FatUnlockFreeClusterBitMap(Vcb);
                                ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));
                            }
                        },
                    );
                }
            }

            WindowRelativeHint = AbsoluteClusterHint - (*(*Vcb).CurrentWindow).FirstCluster + 2;
        } else {
            WindowRelativeHint = AbsoluteClusterHint;
        }
    } else {
        // No hint supplied, or out of range — grab from Vcb
        WindowRelativeHint = (*Vcb).ClusterHint;
        // AbsoluteClusterHint = 0 already (parameter)
        if WindowRelativeHint == 0xFFFFFFFF_u32 {
            WindowRelativeHint = 2;
        }
    }

    // NT_ASSERT: WindowRelativeHint >= 2 and < SizeOfBitMap + 2
    // Keep track of window we're allocating from
    Window = (*Vcb).CurrentWindow;

    // Try to find a run large enough in one shot
    // FatFindFreeClusterRun macro expansion:
    {
        let bm = &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP;
        StartingCluster = if ClusterCount == 1
            && (RtlCheckBit(bm as *const RTL_BITMAP, WindowRelativeHint - 2) == 0)
        {
            WindowRelativeHint
        } else {
            RtlFindClearBits(bm.cast(), ClusterCount, WindowRelativeHint - 2) + 2
        };
    }

    // If found a run that matches (StartingCluster != 1, and either no hint or hint matched)
    if (StartingCluster != 1) && ((0 == AbsoluteClusterHint) || (StartingCluster == WindowRelativeHint)) {
        // FatReserveClusters macro expansion:
        {
            let after_run = StartingCluster + ClusterCount;
            let bm_ptr = &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP;
            RtlSetBits(bm_ptr.cast(), StartingCluster - 2, ClusterCount);
            let after_run2 = if after_run - 2 >= (*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap {
                2
            } else {
                after_run
            };
            if RtlCheckBit(bm_ptr as *const RTL_BITMAP, after_run2 - 2) != 0 {
                let hint = RtlFindClearBits(bm_ptr.cast(), 1, after_run2 - 2) + 2;
                (*Vcb).ClusterHint = if hint == 1 { 2 } else { hint };
            } else {
                (*Vcb).ClusterHint = after_run2;
            }
        }

        (*Window).ClustersFree -= ClusterCount;

        // Convert from window-relative to absolute cluster number
        StartingCluster += (*Window).FirstCluster;
        StartingCluster -= 2;

        FatUnlockFreeClusterBitMap(Vcb);

        FatAddMcbEntry(
            Vcb,
            Mcb,
            0u32,
            FatGetLboFromIndex(Vcb, StartingCluster),
            *ByteCount,
        );

        crate::seh::try_finally(
            &mut (),
            |_| {
                // Update the fat.
                FatAllocateClusters(IrpContext, Vcb, StartingCluster, ClusterCount);
            },
            |_, abnormal| {
                DebugUnwind(b"FatAllocateDiskSpace\0".as_ptr() as *const i8);

                // If the allocate clusters failed, remove the run from the Mcb,
                // unreserve the clusters, and reset the free cluster count.
                if abnormal {
                    FatRemoveMcbEntry(Vcb, Mcb, 0, *ByteCount);
                    FatLockFreeClusterBitMap(Vcb);
                    if Window == (*Vcb).CurrentWindow {
                        // FatUnreserveClusters macro expansion:
                        let fat_index = StartingCluster - (*Window).FirstCluster + 2;
                        RtlClearBits(
                            &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP,
                            fat_index - 2,
                            ClusterCount,
                        );
                        if fat_index < (*Vcb).ClusterHint {
                            (*Vcb).ClusterHint = fat_index;
                        }
                    }
                    (*Window).ClustersFree += ClusterCount;
                    (*Vcb).AllocationSupport.NumberOfFreeClusters += ClusterCount;
                    FatUnlockFreeClusterBitMap(Vcb);
                }

                ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));
            },
        );
    } else {
        // Fragmented allocation path: collect bits from multiple runs
        let mut Index: ULONG = 0;
        let mut Cluster: ULONG = 0;
        let mut CurrentVbo: ULONG = 0;
        let mut PriorLastCluster: ULONG = 0;
        let mut BytesFound: ULONG = 0;
        let mut ClustersFound: ULONG = 0;
        let mut ClustersRemaining: ULONG = 0;
        let mut LockedBitMap: BOOLEAN = 0;
        let mut SelectNextContigWindow: BOOLEAN = 0;

        // Upgrade to exclusive lock for window swap
        FatUnlockFreeClusterBitMap(Vcb);
        ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));
        ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource), 1);
        FatLockFreeClusterBitMap(Vcb);
        LockedBitMap = 1;

        let p_Result = core::ptr::addr_of_mut!(Result);
        let p_LockedBitMap = core::ptr::addr_of_mut!(LockedBitMap);
        let p_Window = core::ptr::addr_of_mut!(Window);
        let p_Cluster = core::ptr::addr_of_mut!(Cluster);
        let p_ClustersFound = core::ptr::addr_of_mut!(ClustersFound);
        let p_CurrentVbo = core::ptr::addr_of_mut!(CurrentVbo);
        let p_BytesFound = core::ptr::addr_of_mut!(BytesFound);
        crate::seh::try_finally(
            &mut (),
            |_| {
                'alloc_try: loop {
                    if ExactMatchRequired != 0 && (*Vcb).NumberOfWindows == 1 {
                        // Can't satisfy exact match from single-window volume
                        Result = 0;
                        break 'alloc_try;
                    }

                    ClustersRemaining = ClusterCount;
                    CurrentVbo = 0;
                    PriorLastCluster = 0;

                    while ClustersRemaining != 0 {
                        if LockedBitMap == 0 {
                            FatLockFreeClusterBitMap(Vcb);
                            LockedBitMap = 1;
                        }

                        ClustersFound = 0;

                        if SelectNextContigWindow == 0 {
                            if WindowRelativeHint != 0 {
                                let desired = {
                                    let bitmap_size = (*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap;
                                    let avail = bitmap_size.wrapping_sub(WindowRelativeHint - 2);
                                    if avail > ClustersRemaining { ClustersRemaining } else { avail }
                                };

                                if RtlAreBitsClear(
                                    (&mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast(),
                                    WindowRelativeHint - 2,
                                    desired,
                                ) != 0 {
                                    Index = WindowRelativeHint - 2;
                                    ClustersFound = desired;
                                    if FatIsFat32(Vcb) != 0 {
                                        SelectNextContigWindow = 1;
                                        WindowRelativeHint = 2;
                                    } else {
                                        WindowRelativeHint = 0;
                                    }
                                } else {
                                    if ExactMatchRequired != 0 {
                                        Result = 0;
                                        break 'alloc_try;
                                    }
                                    WindowRelativeHint = 0;
                                }
                            }

                            if WindowRelativeHint == 0 && ClustersFound == 0 {
                                if ClustersRemaining <= (*(*Vcb).CurrentWindow).ClustersFree {
                                    Index = RtlFindClearBits(
                                        (&mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast(),
                                        ClustersRemaining,
                                        0,
                                    );
                                    if Index != 0xFFFFFFFF_u32 {
                                        ClustersFound = ClustersRemaining;
                                    }
                                }

                                if ClustersFound == 0 {
                                    let mut start_idx: core::ffi::c_ulong = 0;
                                    ClustersFound = RtlFindLongestRunClear(
                                        (&mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast(),
                                        &mut start_idx as *mut core::ffi::c_ulong,
                                    );
                                    Index = start_idx as ULONG;
                                }

                                if ClustersFound >= ClustersRemaining {
                                    ClustersFound = ClustersRemaining;
                                } else {
                                    let bitmap_size = (*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap;
                                    if (Index + ClustersFound == bitmap_size) && FatIsFat32(Vcb) != 0 {
                                        SelectNextContigWindow = 1;
                                        WindowRelativeHint = 2;
                                    }
                                }
                            }
                        } // if !SelectNextContigWindow

                        if ClustersFound == 0 {
                            // Need to switch to a different window
                            let mut FaveWindow: ULONG = 0;
                            let mut SelectedWindow: BOOLEAN = 0;

                            if (*Vcb).NumberOfWindows == 1 {
                                FatBugCheck(0, 5, 0);
                            }

                            if SelectNextContigWindow != 0 {
                                let next_window = ((*Vcb).CurrentWindow as usize - (*Vcb).Windows as usize)
                                    / core::mem::size_of::<FAT_WINDOW>() + 1;

                                if (next_window < (*Vcb).NumberOfWindows as usize)
                                    && (*(*Vcb).Windows.add(next_window)).ClustersFree > 0
                                {
                                    FaveWindow = next_window as ULONG;
                                    SelectedWindow = 1;
                                } else {
                                    if ExactMatchRequired != 0 {
                                        Result = 0;
                                        break 'alloc_try;
                                    }
                                    WindowRelativeHint = 0;
                                }
                                SelectNextContigWindow = 0;
                            }

                            if SelectedWindow == 0 {
                                FaveWindow = FatSelectBestWindow(Vcb);
                            }

                            if (*(*Vcb).Windows.add(FaveWindow as usize)).ClustersFree == 0 {
                                FatBugCheck(0, 5, 1);
                            }

                            Wait = if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_WAIT) != 0 { 1 } else { 0 };
                            SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);

                            FatExamineFatEntries(
                                IrpContext,
                                Vcb,
                                0,
                                0,
                                0,
                                (*Vcb).Windows.add(FaveWindow as usize),
                                core::ptr::null_mut(),
                            );

                            if Wait == 0 {
                                ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);
                            }
                            // loop back to retry with new window
                        } else {
                            // Got ClustersFound clusters starting at Index
                            Window = (*Vcb).CurrentWindow;

                            // FatReserveClusters macro:
                            {
                                let fat_index = Index + 2;
                                let after_run = fat_index + ClustersFound;
                                let bm_ptr2 = &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP;
                                RtlSetBits(bm_ptr2.cast(), fat_index - 2, ClustersFound);
                                let after_run2 = if after_run - 2 >= (*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap {
                                    2
                                } else {
                                    after_run
                                };
                                if RtlCheckBit(bm_ptr2 as *const RTL_BITMAP, after_run2 - 2) != 0 {
                                    let hint = RtlFindClearBits(bm_ptr2.cast(), 1, after_run2 - 2) + 2;
                                    (*Vcb).ClusterHint = if hint == 1 { 2 } else { hint };
                                } else {
                                    (*Vcb).ClusterHint = after_run2;
                                }
                            }

                            Cluster = Index + (*Window).FirstCluster;

                            (*Window).ClustersFree -= ClustersFound;

                            FatUnlockFreeClusterBitMap(Vcb);
                            LockedBitMap = 0;

                            BytesFound = ClustersFound << LogOfBytesPerCluster;

                            FatAddMcbEntry(
                                Vcb,
                                Mcb,
                                CurrentVbo,
                                FatGetLboFromIndex(Vcb, Cluster),
                                BytesFound,
                            );

                            if PriorLastCluster != 0 {
                                FatSetFatEntry(IrpContext, Vcb, PriorLastCluster, Cluster as u32);
                            }

                            FatAllocateClusters(IrpContext, Vcb, Cluster, ClustersFound);

                            CurrentVbo += BytesFound;
                            ClustersRemaining -= ClustersFound;
                            PriorLastCluster = Cluster + ClustersFound - 1;
                        }
                    } // while ClustersRemaining

                    break 'alloc_try;
                } // 'alloc_try

            },
            |_, abnormal| {
                DebugUnwind(b"FatAllocateDiskSpace\0".as_ptr() as *const i8);

                ExReleaseResourceLite(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));

                // Is there any unwinding to do?
                if abnormal || *p_Result == 0 {
                    // Abnormal termination or exact match failure — unwind
                    *ByteCount = 0;

                    if (*p_LockedBitMap) == 0 {
                        FatLockFreeClusterBitMap(Vcb);

                        if (*p_Window) == (*Vcb).CurrentWindow {
                            // FatUnreserveClusters:
                            let fat_index = (*p_Cluster) - (*(*p_Window)).FirstCluster + 2;
                            RtlClearBits(
                                &mut (*Vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP,
                                fat_index - 2,
                                *p_ClustersFound ,
                            );
                            if fat_index < (*Vcb).ClusterHint {
                                (*Vcb).ClusterHint = fat_index;
                            }
                        }

                        (*(*p_Window)).ClustersFree += *p_ClustersFound ;
                        (*Vcb).AllocationSupport.NumberOfFreeClusters += *p_ClustersFound ;

                        FatUnlockFreeClusterBitMap(Vcb);

                        FatRemoveMcbEntry(Vcb, Mcb, *p_CurrentVbo , *p_BytesFound );
                    } else {
                        FatUnlockFreeClusterBitMap(Vcb);
                    }

                    crate::seh::try_finally(
                        &mut (),
                        |_| {
                            // Now send the Mcb off to deallocate disk space.
                            FatDeallocateDiskSpace(IrpContext, Vcb, Mcb, 0u8);
                        },
                        |_, _| {
                            // Now finally (really), remove all the entries from the mcb.
                            FatRemoveMcbEntry(Vcb, Mcb, 0, 0xFFFFFFFF);
                        },
                    );
                }
            },
        );
    }

    DebugTrace(-1, Dbg, b"FatAllocateDiskSpace -> (VOID)\n\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatDeallocateDiskSpace(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    Mcb: *mut LARGE_MCB,
    ZeroOnDeallocate: u8,
) {
    let mut Lbo: u64 = 0;
    let mut Vbo: u32 = 0; // VBO (ULONG32) — only ever an out-param of FatGetNextMcbEntry
    let mut RunsInMcb: u32 = 0;
    let mut ByteCount: u32 = 0;
    let mut ClusterCount: u32 = 0;
    let mut ClusterIndex: u32 = 0;
    let mut McbIndex: u32 = 0;
    let mut LogOfBytesPerCluster: u8 = 0;
    let mut Window: *mut FAT_WINDOW = core::ptr::null_mut();
    let mut ZeroingStatus: i32 = STATUS_SUCCESS;

    const MdlMappingNoExecute: u32 = 0; // fatprocs.h downgrades it to 0 under __REACTOS__

    // DebugTrace calls (mapped to DbgPrint for illustration; real code would use DebugTrace)
    // We'll keep DebugTrace calls as per original but with a stub.
    DebugTrace(1, Dbg, b"FatDeallocateDiskSpace\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Vcb = %p\n\0".as_ptr() as *const i8, Vcb as usize);
    DebugTrace(0, Dbg, b"  Mcb = %p\n\0".as_ptr() as *const i8, Mcb as usize);

    LogOfBytesPerCluster = (*Vcb).AllocationSupport.LogOfBytesPerCluster;

    RunsInMcb = FsRtlNumberOfRunsInLargeMcb(Mcb);

    if RunsInMcb == 0 {
        DebugTrace(-1, Dbg, b"FatDeallocateDiskSpace -> (VOID)\n\0".as_ptr() as *const i8, 0);
        return;
    }

    // ----- Zero on deallocate block (SEH2_TRY) -----
    if ZeroOnDeallocate != 0 {
        // try { ... } except (FatExceptionFilter(NULL, ...)): if we failed to zero
        // for some reason, still go ahead and deallocate the clusters.
        let NoIrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
        let zeroing = crate::seh::fat_try_except(core::ptr::addr_of!(NoIrpContext), || {
            let mut IoIrp: PIRP = core::ptr::null_mut();
            let mut IoEvent: KEVENT = core::mem::zeroed();
            let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut Buffer: *mut core::ffi::c_void = core::ptr::null_mut();
            let mut Mdl: *mut MDL = core::ptr::null_mut();
            let mut ByteCountToZero: u32 = 0;
            let mut MdlSizeMapped: u32 = 0;

            KeInitializeEvent(
                &mut IoEvent as *mut _,
                _EVENT_TYPE_NotificationEvent as u32,
                0, // FALSE
            );

            for McbIndex in 0..RunsInMcb {
                FatGetNextMcbEntry(Vcb, Mcb, McbIndex, &mut Vbo, &mut Lbo, &mut ByteCount);

                // Assert no holes
                // NT_ASSERT(Lbo != 0) - we'll skip as debug-only; could add assert!()
                // ntkrnl_sys::ASSERT!(Lbo != 0);

                // Build MDL
                if ByteCount > MAX_ZERO_MDL_SIZE {
                    Mdl = FatBuildZeroMdl((IrpContext).cast(), MAX_ZERO_MDL_SIZE);
                } else {
                    Mdl = FatBuildZeroMdl((IrpContext).cast(), ByteCount);
                }

                if Mdl.is_null() {
                    ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
                    return; // goto try_exit
                }

                // Inner try/finally: always unmap and free the Mdl.
                crate::seh::try_finally(
                    &mut (),
                    |_| {
                        'inner_try: loop {
                            // Map MDL
                            Buffer = MmGetSystemAddressForMdlSafe((Mdl).cast(), HighPagePriority | MdlMappingNoExecute);
                            if Buffer.is_null() {
                                // NT_ASSERT(FALSE)
                                ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
                                break 'inner_try; // goto try_exit2
                            }

                            MdlSizeMapped = core::cmp::min(ByteCount, (*Mdl).ByteCount);
                            ByteCountToZero = ByteCount;

                            while ByteCountToZero != 0 {
                                KeClearEvent(&mut IoEvent);

                                IoIrp = IoBuildSynchronousFsdRequest(
                                    IRP_MJ_WRITE as u32,
                                    (*Vcb).TargetDeviceObject,
                                    Buffer,
                                    MdlSizeMapped,
                                    &mut Lbo as *mut u64 as *mut LARGE_INTEGER,
                                    core::ptr::addr_of_mut!(IoEvent),
                                    core::ptr::addr_of_mut!(Iosb),
                                );

                                if IoIrp.is_null() {
                                    // NT_ASSERT(FALSE)
                                    ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
                                    break 'inner_try; // goto try_exit2
                                }

                                // Set SL_WRITE_THROUGH flag
                                let stack = IoGetNextIrpStackLocation(IoIrp);
                                (*stack).Flags |= SL_WRITE_THROUGH as u8;

                                ZeroingStatus = IoCallDriver((*Vcb).TargetDeviceObject, IoIrp);

                                if ZeroingStatus == STATUS_PENDING {
                                    KeWaitForSingleObject(
                                        &mut IoEvent as *mut _ as *mut core::ffi::c_void,
                                        _KWAIT_REASON_Executive,
                                        0i8,
                                        0u8, // FALSE
                                        core::ptr::null_mut(),
                                    );
                                    ZeroingStatus = Iosb.__bindgen_anon_1.Status;
                                }

                                if !NT_SUCCESS(ZeroingStatus) {
                                    // NT_ASSERT(FALSE)
                                    break 'inner_try; // goto try_exit2
                                }

                                Lbo += MdlSizeMapped as u64;
                                ByteCountToZero = ByteCountToZero.wrapping_sub(MdlSizeMapped);

                                if ByteCountToZero < MdlSizeMapped {
                                    MdlSizeMapped = ByteCountToZero;
                                }
                            }

                            break 'inner_try; // normal exit from try block to finally
                        }

                    },
                    |_, _| {
                        // Condition: !FlagOn( Mdl->MdlFlags, MDL_SOURCE_IS_NONPAGED_POOL ) && FlagOn( Mdl->MdlFlags, MDL_MAPPED_TO_SYSTEM_VA )
                        let flags = (*Mdl).MdlFlags as u32;
                        if FlagOn(flags, MDL_MAPPED_TO_SYSTEM_VA) != 0 && FlagOn(flags, MDL_SOURCE_IS_NONPAGED_POOL) == 0 {
                            MmUnmapLockedPages((*Mdl).MappedSystemVa, (Mdl).cast());
                        }
                        IoFreeMdl((Mdl).cast());
                },
                );
            }
        });
        if let Err(code) = zeroing {
            ZeroingStatus = code;
        }
    }

    // Assert zeroing succeeded
    // NT_ASSERT( NT_SUCCESS(ZeroingStatus) );
    if cfg!(debug_assertions) {
        ntkrnl_sys::ASSERT!(ZeroingStatus >= 0);
    }

    // ----- Deallocation phase (SEH2_TRY/FINALLY) -----
    let p_McbIndex = core::ptr::addr_of_mut!(McbIndex);
    let p_ByteCount = core::ptr::addr_of_mut!(ByteCount);
    crate::seh::try_finally(
        &mut (),
        |_| {
            // Shared lock
            ExAcquireResourceSharedLite(&mut (*Vcb).ChangeBitMapResource, 1 /* TRUE */);

            RunsInMcb = FsRtlNumberOfRunsInLargeMcb(Mcb);

            // First pass: free clusters in FAT.  This loop uses the function-level
            // McbIndex: the unwind below re-allocates runs 0..=McbIndex.
            *p_McbIndex = 0;
            while *p_McbIndex < RunsInMcb {
                FatGetNextMcbEntry(Vcb, Mcb, *p_McbIndex, &mut Vbo, &mut Lbo, p_ByteCount);

                // NT_ASSERT(Lbo != 0)

                if *p_ByteCount == 0xFFFFFFFF_u32 {
                    ClusterCount = 1u32 << (32u32 - LogOfBytesPerCluster as u32);
                } else {
                    ClusterCount = *p_ByteCount >> LogOfBytesPerCluster;
                }

                ClusterIndex = FatGetIndexFromLbo(Vcb, Lbo as i64);

                // Expand FatFreeClusters macro
                if ClusterCount == 1 {
                    FatSetFatEntry((IrpContext).cast(), Vcb, ClusterIndex, FAT_CLUSTER_AVAILABLE);
                } else {
                    FatSetFatRun((IrpContext).cast(), Vcb, ClusterIndex, ClusterCount, 0 /* FALSE */);
                }

                *p_McbIndex += 1;
            }

            // Lock bitmap and update
            // Expand FatLockFreeClusterBitMap(Vcb)
            {
                // NT_ASSERT(KeAreApcsDisabled()); // omit
                ExAcquireFastMutexUnsafe(&mut (*(Vcb)).FreeClusterBitMapMutex as *mut _ as *mut core::ffi::c_void);
                // ASSERT_CURRENT_WINDOW_GOOD(Vcb) - we'll inline:
                if cfg!(debug_assertions) {
                    ntkrnl_sys::ASSERT!((*(Vcb)).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() == false);
                    let clear = RtlNumberOfClearBits((&mut (*(Vcb)).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast());
                    if (*(*Vcb).CurrentWindow).ClustersFree != clear {
                        DbgPrint((b"FAT: ClustersFree %x h != FreeClusterBitMapClear %x h\n\0".as_ptr() as *const u8).cast(), (*(*Vcb).CurrentWindow).ClustersFree, clear);
                    }
                    ntkrnl_sys::ASSERT!((*(*Vcb).CurrentWindow).ClustersFree == clear);
                }
            }

            // Second pass: bitmap and window counts
            for McbIndex in 0..RunsInMcb {
                let mut ClusterEnd: u32 = 0;
                let mut MyStart: u32 = 0;
                let mut MyLength: u32 = 0;
                let mut count: u32 = 0;

                FatGetNextMcbEntry(Vcb, Mcb, McbIndex, &mut Vbo, &mut Lbo, p_ByteCount);

                if *p_ByteCount == 0xFFFFFFFF_u32 {
                    ClusterCount = 1u32 << (32u32 - LogOfBytesPerCluster as u32);
                } else {
                    ClusterCount = *p_ByteCount >> LogOfBytesPerCluster;
                }

                ClusterIndex = FatGetIndexFromLbo(Vcb, Lbo as i64);
                Window = (*(Vcb)).CurrentWindow;

                ClusterEnd = ClusterIndex + ClusterCount - 1;

                if !(ClusterIndex > (*(Window)).LastCluster || ClusterEnd < (*(Window)).FirstCluster) {
                    // Overlap with current window
                    if ClusterIndex < (*(Window)).FirstCluster && ClusterEnd > (*(Window)).LastCluster {
                        MyStart = (*(Window)).FirstCluster;
                        MyLength = (*(Window)).LastCluster - (*(Window)).FirstCluster + 1;
                    } else if ClusterIndex < (*(Window)).FirstCluster {
                        MyStart = (*(Window)).FirstCluster;
                        MyLength = ClusterEnd - (*(Window)).FirstCluster + 1;
                    } else {
                        MyStart = ClusterIndex;
                        if ClusterEnd <= (*(Window)).LastCluster {
                            MyLength = ClusterCount;
                        } else {
                            MyLength = (*(Window)).LastCluster - ClusterIndex + 1;
                        }
                    }

                    if MyLength == 0 {
                        continue;
                    }

                    // Debug checks (if DBG)
                    if cfg!(debug_assertions) {
                        let previous_clear = RtlNumberOfClearBits((&mut (*(Vcb)).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast());
                        ntkrnl_sys::ASSERT!(MyStart + MyLength - (*(Window)).FirstCluster <= (*(Vcb)).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap);
                        for i in 0..MyLength {
                            let bit_index = MyStart - (*(Window)).FirstCluster + i;
                            ntkrnl_sys::ASSERT!(RtlCheckBit(&(*(Vcb)).FreeClusterBitMap as *const RTL_BITMAP_DBG as *const RTL_BITMAP, bit_index) == 1);
                        }
                    }

                    // Expand FatUnreserveClusters macro
                    {
                        let fat_index = MyStart - (*(Window)).FirstCluster + 2;
                        let cluster_count = MyLength;
                        ntkrnl_sys::ASSERT!(fat_index + cluster_count - 2 <= (*(Vcb)).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap);
                        ntkrnl_sys::ASSERT!(fat_index >= 2);
                        RtlClearBits(&mut (*(Vcb)).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP, fat_index - 2, cluster_count);
                        if fat_index < (*(Vcb)).ClusterHint {
                            (*(Vcb)).ClusterHint = fat_index;
                        }
                    }
                }

                // Adjust ClustersFree for all windows
                if FatIsFat32(Vcb) != 0 {
                    Window = (*(Vcb)).Windows.add(((ClusterIndex - 2) / MAX_CLUSTER_BITMAP_SIZE) as usize);
                } else {
                    Window = (*(Vcb)).Windows.add(0);
                }

                MyStart = ClusterIndex;
                MyLength = ClusterCount;
                while MyLength > 0 {
                    count = if (*(Window)).LastCluster - MyStart + 1 < MyLength {
                        (*(Window)).LastCluster - MyStart + 1
                    } else {
                        MyLength
                    };
                    (*(Window)).ClustersFree += count;

                    if MyLength != count {
                        Window = Window.add(1);
                        MyStart = (*(Window)).FirstCluster;
                    }
                    MyLength -= count;
                }

                (*(Vcb)).AllocationSupport.NumberOfFreeClusters += ClusterCount;
            }

            // Debug consistency check
            if cfg!(debug_assertions) {
                let clear_count = RtlNumberOfClearBits((&mut (*(Vcb)).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast());
                if (*(*Vcb).CurrentWindow).ClustersFree != clear_count {
                    DbgPrint((b"%x vs %x\n\0".as_ptr() as *const u8).cast(), (*(*Vcb).CurrentWindow).ClustersFree, clear_count);
                    DbgPrint((b"%x for %x\n\0".as_ptr() as *const u8).cast(), ClusterIndex, ClusterCount);
                }
            }

            // Expand FatUnlockFreeClusterBitMap(Vcb)
            {
                // ASSERT_CURRENT_WINDOW_GOOD as above
                if cfg!(debug_assertions) {
                    ntkrnl_sys::ASSERT!((*(Vcb)).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() == false);
                    let clear = RtlNumberOfClearBits((&mut (*(Vcb)).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast());
                    if (*(*Vcb).CurrentWindow).ClustersFree != clear {
                        DbgPrint((b"FAT: ClustersFree %x h != FreeClusterBitMapClear %x h\n\0".as_ptr() as *const u8).cast(), (*(*Vcb).CurrentWindow).ClustersFree, clear);
                    }
                    ntkrnl_sys::ASSERT!((*(*Vcb).CurrentWindow).ClustersFree == clear);
                }
                ExReleaseFastMutexUnsafe(&mut (*(Vcb)).FreeClusterBitMapMutex as *mut _ as *mut core::ffi::c_void);
            }

        },
        |_, abnormal| {
            DebugUnwind(b"FatDeallocateDiskSpace\0".as_ptr() as *const i8);
            ExReleaseResourceLite(&mut (*Vcb).ChangeBitMapResource);

            // Is there any unwinding to do?
            if abnormal {
                // For each entry we already deallocated, reallocate it, chaining
                // together as necessary (up to and including the one that failed).
                let mut LocalLbo: u64 = 0;
                let mut LocalVbo: u32 = 0; // VBO (ULONG32)
                let mut Clusters: u32 = 0;
                let mut FatIndex: u32 = 0;
                let mut PriorLastIndex: u32 = 0;

                for Index in 0..=*p_McbIndex {
                    FatGetNextMcbEntry(Vcb, Mcb, Index, &mut LocalVbo, &mut LocalLbo, p_ByteCount);

                    if *p_ByteCount == 0xFFFFFFFF_u32 {
                        Clusters = 1 << (32 - LogOfBytesPerCluster);
                    } else {
                        Clusters = *p_ByteCount >> LogOfBytesPerCluster;
                    }

                    FatIndex = FatGetIndexFromLbo(Vcb, LocalLbo as i64);

                    if PriorLastIndex != 0 {
                        FatSetFatEntry((IrpContext).cast(), Vcb, PriorLastIndex, FatIndex);
                    }

                    if Index < *p_McbIndex {
                        FatAllocateClusters(IrpContext, Vcb, FatIndex, Clusters);
                        PriorLastIndex = FatIndex + Clusters - 1;
                    }
                }
            }
        },
    );

    DebugTrace(-1, Dbg, b"FatDeallocateDiskSpace -> (VOID)\n\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatSplitAllocation(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Mcb: PLARGE_MCB,
    SplitAtVbo: u32,
    RemainingMcb: PLARGE_MCB,
) {
    let mut SourceVbo: u32 = 0;
    let mut TargetVbo: u32 = 0;
    let mut DontCare: u32 = 0;
    let mut Lbo: i64 = 0;
    let mut ByteCount: u32 = 0;

    DebugTrace(1, Dbg, b"FatSplitAllocation\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Vcb          = %p\n\0".as_ptr() as *const i8, Vcb as usize);
    DebugTrace(0, Dbg, b"  Mcb          = %p\n\0".as_ptr() as *const i8, Mcb as usize);
    DebugTrace(
        0,
        Dbg,
        b"  SplitAtVbo   = %8lx\n\0".as_ptr() as *const i8,
        SplitAtVbo as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  RemainingMcb = %p\n\0".as_ptr() as *const i8,
        RemainingMcb as usize,
    );

    let BytesPerCluster: u32 = (1u32 << (*(Vcb as *mut VCB)).AllocationSupport.LogOfBytesPerCluster) as u32;

    NT_ASSERT!((((SplitAtVbo & (BytesPerCluster - 1)) == 0) as u8) as i32);

    NT_ASSERT!(((!((SplitAtVbo != 0) && (FsRtlNumberOfRunsInLargeMcb(Mcb) == 0))) as u8) as i32);

    //   NT_ASSERT( FsRtlNumberOfRunsInMcb( RemainingMcb ) == 0 );

    crate::seh::try_finally(
        &mut (),
        |_| {
            SourceVbo = SplitAtVbo;
            TargetVbo = 0;

            loop {
                let found = FatLookupMcbEntry(
                    (Vcb as *mut VCB).cast(),
                    Mcb,
                    SourceVbo,
                    &raw mut Lbo,
                    &raw mut ByteCount,
                    core::ptr::null_mut(),
                );
                if found == 0 {
                    break;
                }
                FatAddMcbEntry(Vcb, RemainingMcb, TargetVbo, Lbo, ByteCount);
                FatRemoveMcbEntry((Vcb).cast(), Mcb, SourceVbo, ByteCount);
                TargetVbo += ByteCount;
                SourceVbo += ByteCount;
                if SourceVbo == 0 {
                    break;
                }
            }

            if SplitAtVbo != 0 {
                FatLookupLastMcbEntry(
                    (Vcb as *mut VCB).cast(),
                    Mcb,
                    &raw mut DontCare,
                    &raw mut Lbo,
                    core::ptr::null_mut(),
                );
                FatSetFatEntry(
                    IrpContext,
                    (Vcb).cast(),
                    FatGetIndexFromLbo(Vcb, Lbo),
                    0x0FFFFFFF as u32, // FAT_CLUSTER_LAST
                );
            }

        },
        |_, abnormal| {
            DebugUnwind(b"FatSplitAllocation\0".as_ptr() as *const i8);

            // If we got an exception, we must glue back together the Mcbs.
            if abnormal {
                let mut TargetVbo: u32 = SplitAtVbo;
                let mut SourceVbo: u32 = 0;
                let mut Lbo: i64 = 0;
                let mut ByteCount: u32 = 0;
                loop {
                    let found = FatLookupMcbEntry(
                        (Vcb as *mut VCB).cast(),
                        RemainingMcb,
                        SourceVbo,
                        &raw mut Lbo,
                        &raw mut ByteCount,
                        core::ptr::null_mut(),
                    );
                    if found == 0 {
                        break;
                    }
                    FatAddMcbEntry(Vcb, Mcb, TargetVbo, Lbo, ByteCount);
                    FatRemoveMcbEntry((Vcb).cast(), RemainingMcb, SourceVbo, ByteCount);
                    TargetVbo += ByteCount;
                    SourceVbo += ByteCount;
                }
            }

            DebugTrace(
                -1,
                Dbg,
                b"FatSplitAllocation -> (VOID)\n\0".as_ptr() as *const i8,
                0,
            );
        },
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatMergeAllocation(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Mcb: PLARGE_MCB,
    SecondMcb: PLARGE_MCB,
) {
    let mut SpliceVbo: u32 = 0;
    let mut SpliceLbo: i64 = 0;
    let mut SourceVbo: u32 = 0;
    let mut TargetVbo: u32 = 0;
    let mut Lbo: i64 = 0;
    let mut ByteCount: u32 = 0;

    // PAGED_CODE(); // could be empty or indicated

    DebugTrace(1, 0, b"FatMergeAllocation\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, 0, b"  Vcb       = %p\n\0".as_ptr() as *const i8, Vcb as usize);
    DebugTrace(0, 0, b"  Mcb       = %p\n\0".as_ptr() as *const i8, Mcb as usize);
    DebugTrace(0, 0, b"  SecondMcb = %p\n\0".as_ptr() as *const i8, SecondMcb as usize);

    let p_SpliceVbo = core::ptr::addr_of_mut!(SpliceVbo);
    let p_TargetVbo = core::ptr::addr_of_mut!(TargetVbo);
    crate::seh::try_finally(
        &mut (),
        |_| {
            // Append the runs from SecondMcb to Mcb
            FatLookupLastMcbEntry(Vcb, Mcb, &mut SpliceVbo, &mut SpliceLbo, core::ptr::null_mut());
            SourceVbo = 0;
            TargetVbo = SpliceVbo + 1;

            while FatLookupMcbEntry(
                Vcb,
                SecondMcb,
                SourceVbo,
                &mut Lbo,
                &mut ByteCount,
                core::ptr::null_mut(),
            ) != 0
            {
                FatAddMcbEntry(Vcb, Mcb, TargetVbo, Lbo, ByteCount);
                SourceVbo += ByteCount;
                TargetVbo += ByteCount;
            }

            // Link the last pre-merge cluster to the first cluster of SecondMcb
            FatLookupMcbEntry(
                Vcb,
                SecondMcb,
                0,
                &mut Lbo,
                core::ptr::null_mut(), // (PULONG)NULL
                core::ptr::null_mut(),
            );

            FatSetFatEntry(
                IrpContext,
                Vcb,
                FatGetIndexFromLbo(Vcb, SpliceLbo),
                FatGetIndexFromLbo(Vcb, Lbo) as u32,
            );
        },
        |_, abnormal| {
            DebugUnwind(b"FatMergeAllocation\0".as_ptr() as *const i8);

            // If we got an exception, we must remove the runs added to Mcb.
            if abnormal {
                let CutLength = (*p_TargetVbo).wrapping_sub(*p_SpliceVbo + 1);
                if CutLength != 0 {
                    FatRemoveMcbEntry((Vcb).cast(), Mcb, *p_SpliceVbo + 1, CutLength);
                }
            }
        },
    );

    DebugTrace(-1, 0, b"FatMergeAllocation -> (VOID)\n\0".as_ptr() as *const i8, 0);
}
#[no_mangle]
pub unsafe extern "system" fn FatInterpretClusterType(
    Vcb: *mut VCB,
    mut Entry: FAT_ENTRY,
) -> CLUSTER_TYPE {
    // Declare unknown externals

    const FAT32_ENTRY_MASK: FAT_ENTRY = 0x0FFFFFFF;
    const FAT_CLUSTER_AVAILABLE: FAT_ENTRY = 0x00000000; // fat.h:255
    const FAT_CLUSTER_RESERVED: FAT_ENTRY = 0x0ffffff0;
    const FAT_CLUSTER_BAD: FAT_ENTRY = 0x0ffffff7;

    unsafe {
        DebugTrace(1, Dbg as u32, b"InterpretClusterType\n\0".as_ptr() as *const i8, 0);
        DebugTrace(0, Dbg as u32, b"  Vcb   = %p\n\0".as_ptr() as *const i8, Vcb as usize);
        DebugTrace(0, Dbg as u32, b"  Entry = %8lx\n\0".as_ptr() as *const i8, (Entry) as usize);
        PAGED_CODE!();

        match (*Vcb).AllocationSupport.FatIndexBitSize {
            32 => {
                Entry &= FAT32_ENTRY_MASK;
            }
            12 => {
                NT_ASSERT!(((Entry <= 0xfff) as u8) as i32);
                if Entry >= 0x0ff0 {
                    Entry |= 0x0FFFF000;
                }
            }
            16 | _ => {
                NT_ASSERT!(((Entry <= 0xffff) as u8) as i32);
                if Entry >= 0x0fff0 {
                    Entry |= 0x0FFF0000;
                }
            }
        }

        if Entry == FAT_CLUSTER_AVAILABLE {
            DebugTrace(-1, Dbg as u32, b"FatInterpretClusterType -> FatClusterAvailable\n\0".as_ptr() as *const i8, 0);
            return FatClusterAvailable;
        } else if Entry < FAT_CLUSTER_RESERVED {
            DebugTrace(-1, Dbg as u32, b"FatInterpretClusterType -> FatClusterNext\n\0".as_ptr() as *const i8, 0);
            return FatClusterNext;
        } else if Entry < FAT_CLUSTER_BAD {
            DebugTrace(-1, Dbg as u32, b"FatInterpretClusterType -> FatClusterReserved\n\0".as_ptr() as *const i8, 0);
            return FatClusterReserved;
        } else if Entry == FAT_CLUSTER_BAD {
            DebugTrace(-1, Dbg as u32, b"FatInterpretClusterType -> FatClusterBad\n\0".as_ptr() as *const i8, 0);
            return FatClusterBad;
        } else {
            DebugTrace(-1, Dbg as u32, b"FatInterpretClusterType -> FatClusterLast\n\0".as_ptr() as *const i8, 0);
            return FatClusterLast;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatLookupFatEntry(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FatIndex: u32,
    FatEntry: *mut u32,
    Context: *mut FAT_ENUMERATION_CONTEXT,
) {
    unsafe {
        PAGED_CODE!();

        DebugTrace(1, Dbg as u32, b"FatLookupFatEntry\n\0".as_ptr() as *const i8, (0i32) as usize);
        DebugTrace(0, Dbg as u32, b"  Vcb      = %p\n\0".as_ptr() as *const i8, Vcb as usize);
        DebugTrace(0, Dbg as u32, b"  FatIndex = %4x\n\0".as_ptr() as *const i8, (FatIndex) as usize);
        DebugTrace(0, Dbg as u32, b"  FatEntry = %8lx\n\0".as_ptr() as *const i8, FatEntry as usize);

        FatVerifyIndexIsValid(IrpContext, Vcb, FatIndex);

        if (*Vcb).AllocationSupport.FatIndexBitSize == 12  {
            if (*Context).Bcb.is_null() {
                FatReadVolumeFile(
                    IrpContext,
                    Vcb,
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void),
                    FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void),
                    core::ptr::addr_of_mut!((*Context).Bcb).cast::<*mut core::ffi::c_void>(),
                    core::ptr::addr_of_mut!((*Context).PinnedPage).cast::<*mut u8>(),
                );
            }

            FatLookup12BitEntry((*Context).PinnedPage as *mut u8, FatIndex, FatEntry);
        } else if (*Vcb).AllocationSupport.FatIndexBitSize == 32  {
            let OffsetIntoVolumeFile: u32;
            let PageEntryOffset: u32;

            OffsetIntoVolumeFile = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void)
                + FatIndex * core::mem::size_of::<FAT_ENTRY>() as u32;
            PageEntryOffset = (OffsetIntoVolumeFile % (PAGE_SIZE) as u32) / core::mem::size_of::<FAT_ENTRY>() as u32;

            if ((*Context).Bcb.is_null())
                || (OffsetIntoVolumeFile / PAGE_SIZE as u32
                    != (*Context).VboOfPinnedPage / PAGE_SIZE as u32)
            {
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*Context).Bcb) as *mut *mut core::ffi::c_void);

                FatReadVolumeFile(
                    IrpContext,
                    Vcb,
                    OffsetIntoVolumeFile & !(PAGE_SIZE as u32 - 1),
                    PAGE_SIZE as u32,
                    core::ptr::addr_of_mut!((*Context).Bcb).cast::<*mut core::ffi::c_void>(),
                    core::ptr::addr_of_mut!((*Context).PinnedPage).cast::<*mut u8>(),
                );

                (*Context).VboOfPinnedPage = OffsetIntoVolumeFile & !(PAGE_SIZE as u32 - 1);
            }

            *FatEntry = ((*Context).PinnedPage as *mut u32)
                .add(PageEntryOffset as usize)
                .read()
                & FAT32_ENTRY_MASK;
        } else {
            let OffsetIntoVolumeFile: u32;
            let PageEntryOffset: u32;

            OffsetIntoVolumeFile = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void)
                + FatIndex * core::mem::size_of::<u16>() as u32;
            PageEntryOffset = (OffsetIntoVolumeFile % (PAGE_SIZE) as u32) / core::mem::size_of::<u16>() as u32;

            if ((*Context).Bcb.is_null())
                || (OffsetIntoVolumeFile / PAGE_SIZE as u32
                    != (*Context).VboOfPinnedPage / PAGE_SIZE as u32)
            {
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!((*Context).Bcb) as *mut *mut core::ffi::c_void);

                FatReadVolumeFile(
                    IrpContext,
                    Vcb,
                    OffsetIntoVolumeFile & !(PAGE_SIZE as u32 - 1),
                    PAGE_SIZE as u32,
                    core::ptr::addr_of_mut!((*Context).Bcb).cast::<*mut core::ffi::c_void>(),
                    core::ptr::addr_of_mut!((*Context).PinnedPage).cast::<*mut u8>(),
                );

                (*Context).VboOfPinnedPage = OffsetIntoVolumeFile & !(PAGE_SIZE as u32 - 1);
            }

            *FatEntry = ((*Context).PinnedPage as *mut u16)
                .add(PageEntryOffset as usize)
                .read() as u32;
        }

        DebugTrace(-1, Dbg as u32, b"FatLookupFatEntry -> (VOID)\n\0".as_ptr() as *const i8, (0i32) as usize);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatSetFatEntry(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FatIndex: u32,
    mut FatEntry: u32,
) {
    let mut Lbo: u64 = 0;
    let mut Bcb: *mut BCB = core::ptr::null_mut();
    let mut SectorSize: u32 = 0;
    let mut OffsetIntoVolumeFile: u32 = 0;
    let mut WasWait: i32 = 1; // TRUE
    let mut RegularOperation: i32 = 1; // TRUE
    let mut CleaningOperation: i32 = 0; // FALSE
    let mut ReleaseMutex: i32 = 0; // FALSE

    // PAGED_CODE();

    // DebugTrace calls – preserving as comments due to variadic args
    // DebugTrace(+1, Dbg, "FatSetFatEntry\n", 0);
    // DebugTrace( 0, Dbg, "  Vcb      = %p\n", Vcb);
    // DebugTrace( 0, Dbg, "  FatIndex = %4x\n", FatIndex);
    // DebugTrace( 0, Dbg, "  FatEntry = %4x\n", FatEntry);

    // Handle dirty bit index or regular index
    if FatIndex == FAT_DIRTY_BIT_INDEX {
        RegularOperation = 0; // FALSE

        // FAT_CLEAN_VOLUME = ~FAT32_ENTRY_MASK | 0 = 0xF0000000; FAT_DIRTY_VOLUME = 0xF0000001
        const FAT_CLEAN_VOLUME_U32: u32 = 0xF0000000u32;
        const FAT_DIRTY_VOLUME_U32: u32 = 0xF0000001u32;
        if FatEntry == FAT_CLEAN_VOLUME_U32 {
            FatEntry = (FAT_CLEAN_ENTRY) as u32;
            CleaningOperation = 1; // TRUE
        } else if FatEntry == FAT_DIRTY_VOLUME_U32 {
            match (*Vcb).AllocationSupport.FatIndexBitSize {
                12 => FatEntry = FAT12_DIRTY_ENTRY,
                32 => FatEntry = FAT32_DIRTY_ENTRY,
                _ => FatEntry = FAT16_DIRTY_ENTRY,
            }
        } else {
            FatRaiseStatus(IrpContext, (STATUS_FILE_CORRUPT_ERROR) as u32);
            // unreachable
        }

        WasWait = FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);
        SetFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_DISABLE_DIRTY);
    } else {
        ntkrnl_sys::ASSERT!((FatEntry & !0x0FFFFFFF) == 0); // !FAT32_ENTRY_MASK = 0xF0000000 approx
        FatVerifyIndexIsValid(IrpContext, Vcb, FatIndex);
    }

    SectorSize = 1 << (*Vcb).AllocationSupport.LogOfBytesPerSector;

    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    let p_ReleaseMutex = core::ptr::addr_of_mut!(ReleaseMutex);
    crate::seh::try_finally(
        &mut (),
        |_| {
            if (*Vcb).AllocationSupport.FatIndexBitSize == 12 {
                let mut PinnedFat: *mut c_void = core::ptr::null_mut();

                // 12-bit entry mask
                let FatEntry = FatEntry & 0xFFF;

                OffsetIntoVolumeFile = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) + FatIndex * 3 / 2;

                FatPrepareWriteVolumeFile(
                    IrpContext,
                    Vcb,
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void),
                    FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void),
                    &mut Bcb,
                    &mut PinnedFat as *mut *mut c_void,
                    RegularOperation,
                    0,
                );

                Lbo = (OffsetIntoVolumeFile & !(SectorSize - 1)) as u64;
                FatAddMcbEntry(Vcb, &mut (*Vcb).DirtyFatMcb, Lbo as u32, Lbo as i64, SectorSize);

                if (OffsetIntoVolumeFile & (SectorSize - 1)) == (SectorSize - 1) {
                    Lbo += SectorSize as u64;
                    FatAddMcbEntry(Vcb, &mut (*Vcb).DirtyFatMcb, Lbo as u32, Lbo as i64, SectorSize);
                }

                FatLockFreeClusterBitMap(Vcb);
                ReleaseMutex = 1; // TRUE

                FatSet12BitEntry(PinnedFat, FatIndex, FatEntry);

                FatUnlockFreeClusterBitMap(Vcb);
                ReleaseMutex = 0; // FALSE

            } else if (*Vcb).AllocationSupport.FatIndexBitSize == 32 {
                let mut PinnedFatEntry32: *mut u32 = core::ptr::null_mut();

                OffsetIntoVolumeFile =
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) + FatIndex * core::mem::size_of::<u32>() as u32;

                FatPrepareWriteVolumeFile(
                    IrpContext,
                    Vcb,
                    OffsetIntoVolumeFile,
                    core::mem::size_of::<u32>() as u32,
                    &mut Bcb,
                    &mut PinnedFatEntry32 as *mut *mut u32 as *mut *mut c_void,
                    RegularOperation,
                    0,
                );

                Lbo = (OffsetIntoVolumeFile & !(SectorSize - 1)) as u64;
                FatAddMcbEntry(Vcb, &mut (*Vcb).DirtyFatMcb, Lbo as u32, Lbo as i64, SectorSize);

                // Mutex for Alpha (platform-dependent, but we mimic)
                // if cfg!(target_arch = "alpha") { ... }
                // but for transliteration we include the calls as in C
                #[cfg(target_arch = "alpha")]
                {
                    FatLockFreeClusterBitMap(Vcb);
                    ReleaseMutex = 1;
                }

                if FatIndex != FAT_DIRTY_BIT_INDEX {
                    *PinnedFatEntry32 = (*PinnedFatEntry32 & !0x0FFFFFFF) | FatEntry;
                } else {
                    *PinnedFatEntry32 = FatEntry;
                }

                #[cfg(target_arch = "alpha")]
                {
                    FatUnlockFreeClusterBitMap(Vcb);
                    ReleaseMutex = 0;
                }
            } else {
                // 16 bit case
                let mut PinnedFatEntry: *mut u16 = core::ptr::null_mut();

                OffsetIntoVolumeFile =
                    FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) + FatIndex * core::mem::size_of::<u16>() as u32;

                FatPrepareWriteVolumeFile(
                    IrpContext,
                    Vcb,
                    OffsetIntoVolumeFile,
                    core::mem::size_of::<u16>() as u32,
                    &mut Bcb,
                    &mut PinnedFatEntry as *mut *mut u16 as *mut *mut c_void,
                    RegularOperation,
                    0,
                );

                Lbo = (OffsetIntoVolumeFile & !(SectorSize - 1)) as u64;
                FatAddMcbEntry(Vcb, &mut (*Vcb).DirtyFatMcb, Lbo as u32, Lbo as i64, SectorSize);

                #[cfg(target_arch = "alpha")]
                {
                    FatLockFreeClusterBitMap(Vcb);
                    ReleaseMutex = 1;
                }

                *PinnedFatEntry = FatEntry as u16;

                #[cfg(target_arch = "alpha")]
                {
                    FatUnlockFreeClusterBitMap(Vcb);
                    ReleaseMutex = 0;
                }
            }
        },
        |_, abnormal| {
            DebugUnwind(b"FatSetFatEntry\0".as_ptr() as *const i8);

            // Clear the disable-dirty flag
            ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_DISABLE_DIRTY);

            // Restore asynchronous state if needed
            if WasWait == 0 {
                ClearFlag!((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT);
            }

            // If we still hold the mutex (only if abnormal termination)
            if *p_ReleaseMutex != 0 {
                NT_ASSERT!(abnormal as i32);
                FatUnlockFreeClusterBitMap(Vcb);
            }

            // Unpin the Bcb, handling cleaning or mount-in-progress cases
            if (CleaningOperation != 0
                || FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_MOUNT_IN_PROGRESS) != 0)
                && !(*p_Bcb).is_null()
            {
                let mut IgnoreStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
                CcRepinBcb(*p_Bcb);
                CcUnpinData(*p_Bcb);
                DbgDoit(&mut ((*IrpContext).PinCount) as *mut _ as *mut c_void); // approximate
                CcUnpinRepinnedBcb(*p_Bcb, 1, &mut IgnoreStatus);
            } else {
                FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
            }
        },
    );

    // DebugTrace(-1, Dbg, "FatSetFatEntry -> (VOID)\n", 0);
}
#[no_mangle]
pub unsafe extern "system" fn FatSetFatRun(
    IrpContext: PIRP_CONTEXT,
    Vcb: *mut VCB,
    mut StartingFatIndex: u32,
    mut ClusterCount: u32,
    ChainTogether: u8,
) {
    const MAXCOUNTCLUS: u32 = 0x10000;
    const COUNTSAVEDBCBS: usize = ((MAXCOUNTCLUS as usize * core::mem::size_of::<u32>() /* FAT_ENTRY size */) / PAGE_SIZE as usize) + 2;
    let mut SavedBcbs: [[*mut core::ffi::c_void; 2]; COUNTSAVEDBCBS] = [[core::ptr::null_mut(); 2]; COUNTSAVEDBCBS];

    let mut SectorSize: u32 = 0;
    let mut Cluster: u32 = 0;
    let mut StartSectorLbo: u32 = 0;
    let mut FinalSectorLbo: u32 = 0;
    let mut Lbo: u32 = 0;
    let mut PinnedFat: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ReleaseMutex: u8 = 0;
    let SavedStartingFatIndex: u32 = StartingFatIndex;

    DebugTrace(1, Dbg, b"FatSetFatRun\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Vcb              = %p\n\0".as_ptr() as *const i8, Vcb as usize);
    DebugTrace(0, Dbg, b"  StartingFatIndex = %8x\n\0".as_ptr() as *const i8, (StartingFatIndex) as usize);
    DebugTrace(0, Dbg, b"  ClusterCount     = %8lx\n\0".as_ptr() as *const i8, (ClusterCount) as usize);
    DebugTrace(0, Dbg, b"  ChainTogether    = %s\n\0".as_ptr() as *const i8, (if ChainTogether != 0 { b"TRUE\0".as_ptr() } else { b"FALSE\0".as_ptr() }) as usize);

    FatVerifyIndexIsValid(IrpContext as *mut IRP_CONTEXT, Vcb, StartingFatIndex);
    FatVerifyIndexIsValid(IrpContext as *mut IRP_CONTEXT, Vcb, StartingFatIndex + ClusterCount - 1);

    if ClusterCount == 0 {
        DebugTrace(-1, Dbg, b"FatSetFatRun -> (VOID)\n\0".as_ptr() as *const i8, 0);
        return;
    }

    SectorSize = 1u32 << (*Vcb).AllocationSupport.LogOfBytesPerSector;

    let p_ReleaseMutex = core::ptr::addr_of_mut!(ReleaseMutex);
    let p_SavedBcbs = core::ptr::addr_of_mut!(SavedBcbs);
    let p_StartingFatIndex = core::ptr::addr_of_mut!(StartingFatIndex);
    crate::seh::try_finally(
        &mut (),
        |_| {
            if (*Vcb).AllocationSupport.FatIndexBitSize == 12 {
                // Zero first two rows of SavedBcbs
                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!(SavedBcbs[0][0]) as *mut u8,
                    0,
                    2 * core::mem::size_of::<*mut core::ffi::c_void>() * 2,
                );

                let fat_reserved = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void);
                let fat_bytes = FatBytesPerFat(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void);
                FatPrepareWriteVolumeFile(
                    IrpContext,
                    Vcb,
                    fat_reserved,
                    fat_bytes,
                    core::ptr::addr_of_mut!(SavedBcbs[0][0]).cast::<*mut BCB>(),
                    core::ptr::addr_of_mut!(PinnedFat),
                    1,
                    0,
                );

                StartSectorLbo = (fat_reserved + StartingFatIndex * 3 / 2) & !(SectorSize - 1);
                FinalSectorLbo = (fat_reserved + ((StartingFatIndex + ClusterCount) * 3 + 2) / 2) & !(SectorSize - 1);
                Lbo = StartSectorLbo;
                while Lbo <= FinalSectorLbo {
                    FatAddMcbEntry(
                        Vcb,
                        core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb),
                        Lbo as u32,
                        Lbo as i64,
                        SectorSize,
                    );
                    Lbo += SectorSize;
                }

                FatLockFreeClusterBitMap(Vcb);
                ReleaseMutex = 1;

                Cluster = StartingFatIndex;
                while Cluster < StartingFatIndex + ClusterCount - 1 {
                    FatSet12BitEntry(
                        PinnedFat,
                        Cluster,
                        if ChainTogether != 0 { ((Cluster + 1) as u16) as u32 } else { (FAT_CLUSTER_AVAILABLE as u16) as u32 },
                    );
                    Cluster += 1;
                }

                FatSet12BitEntry(
                    PinnedFat,
                    Cluster,
                    if ChainTogether != 0 { ((FAT_CLUSTER_LAST & 0xfff) as u16) as u32 } else { (FAT_CLUSTER_AVAILABLE as u16) as u32 },
                );

                FatUnlockFreeClusterBitMap(Vcb);
                ReleaseMutex = 0;
            } else if (*Vcb).AllocationSupport.FatIndexBitSize == 32 {
                loop {
                    let mut StartOffsetInVolume: u32;
                    let mut FinalOffsetInVolume: u32;
                    let mut Page: u32;
                    let mut FinalCluster: u32;
                    let mut FatEntry: *mut u32 = core::ptr::null_mut();
                    let ClusterCountThisRun: u32;

                    StartOffsetInVolume = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) +
                        StartingFatIndex * core::mem::size_of::<u32>() as u32;

                    if ClusterCount > MAXCOUNTCLUS {
                        ClusterCountThisRun = MAXCOUNTCLUS;
                    } else {
                        ClusterCountThisRun = ClusterCount;
                    }

                    FinalOffsetInVolume = StartOffsetInVolume + (ClusterCountThisRun - 1) * core::mem::size_of::<u32>() as u32;

                    {
                        let NumberOfPages = (FinalOffsetInVolume / PAGE_SIZE as u32) - (StartOffsetInVolume / PAGE_SIZE as u32) + 1;
                        let mut Offset = StartOffsetInVolume & !(PAGE_SIZE as u32 - 1);
                        let zero_count = (NumberOfPages as usize + 1) * 2 * core::mem::size_of::<*mut core::ffi::c_void>();
                        // zero_count is in BYTES (C RtlZeroMemory); writing it as a pointer count
                        // zeroed 8x past SavedBcbs once NumberOfPages >= 8.
                        core::ptr::write_bytes(core::ptr::addr_of_mut!(SavedBcbs[0][0]) as *mut u8, 0, zero_count);

                        Page = 0;
                        while Page < NumberOfPages {
                            FatPrepareWriteVolumeFile(
                                IrpContext,
                                Vcb,
                                Offset,
                                PAGE_SIZE as u32,
                                core::ptr::addr_of_mut!(SavedBcbs[Page as usize][0]).cast::<*mut BCB>(),
                                core::ptr::addr_of_mut!(SavedBcbs[Page as usize][1]) as *mut *mut core::ffi::c_void,
                                1,
                                0,
                            );

                            if Page == 0 {
                                FatEntry = (SavedBcbs[0][1] as *mut u8).add((StartOffsetInVolume % PAGE_SIZE as u32) as usize) as *mut u32;
                            }

                            Page += 1;
                            Offset += PAGE_SIZE as u32;
                        }
                    }

                    StartSectorLbo = StartOffsetInVolume & !(SectorSize - 1);
                    FinalSectorLbo = FinalOffsetInVolume & !(SectorSize - 1);
                    Lbo = StartSectorLbo;
                    while Lbo <= FinalSectorLbo {
                        FatAddMcbEntry(
                            Vcb,
                            core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb),
                            Lbo as u32,
                            Lbo as i64,
                            SectorSize,
                        );
                        Lbo += SectorSize;
                    }

                    // ALPHA mutex omitted

                    FinalCluster = StartingFatIndex + ClusterCountThisRun - 1;
                    Page = 0;
                    Cluster = StartingFatIndex;
                    while Cluster <= FinalCluster {
                        if ((FatEntry as usize & (PAGE_SIZE - 1)) == 0) && (Cluster != StartingFatIndex) {
                            Page += 1;
                            FatEntry = SavedBcbs[Page as usize][1] as *mut u32;
                        }

                        *FatEntry = if ChainTogether != 0 {
                            (Cluster + 1) as u32
                        } else {
                            FAT_CLUSTER_AVAILABLE as u32
                        };

                        Cluster += 1;
                        FatEntry = FatEntry.add(1);
                    }

                    if (ClusterCount <= MAXCOUNTCLUS) && (ChainTogether != 0) {
                        *FatEntry.sub(1) = FAT_CLUSTER_LAST;
                    }

                    {
                        let mut i = 0;
                        while i < COUNTSAVEDBCBS && !SavedBcbs[i][0].is_null() {
                            FatUnpinBcb(IrpContext as *mut IRP_CONTEXT, core::ptr::addr_of_mut!(SavedBcbs[i][0]) as *mut *mut core::ffi::c_void);
                            SavedBcbs[i][0] = core::ptr::null_mut();
                            i += 1;
                        }
                    }

                    if ClusterCount <= MAXCOUNTCLUS {
                        break;
                    } else {
                        StartingFatIndex += MAXCOUNTCLUS;
                        ClusterCount -= MAXCOUNTCLUS;
                    }
                }
            } else {
                // 16-bit FAT
                let StartOffsetInVolume = FatReservedBytes(core::ptr::addr_of!((*Vcb).Bpb) as *const core::ffi::c_void) +
                    StartingFatIndex * core::mem::size_of::<u16>() as u32;
                let FinalOffsetInVolume = StartOffsetInVolume + (ClusterCount - 1) * core::mem::size_of::<u16>() as u32;
                let NumberOfPages = (FinalOffsetInVolume / PAGE_SIZE as u32) - (StartOffsetInVolume / PAGE_SIZE as u32) + 1;
                let mut Offset = StartOffsetInVolume & !(PAGE_SIZE as u32 - 1);
                let zero_count = (NumberOfPages as usize + 1) * 2 * core::mem::size_of::<*mut core::ffi::c_void>();
                // zero_count is in BYTES (C RtlZeroMemory); writing it as a pointer count
                // zeroed 8x past SavedBcbs once NumberOfPages >= 8.
                core::ptr::write_bytes(core::ptr::addr_of_mut!(SavedBcbs[0][0]) as *mut u8, 0, zero_count);

                let mut Page = 0;
                let mut FatEntry: *mut u16 = core::ptr::null_mut();
                while Page < NumberOfPages {
                    FatPrepareWriteVolumeFile(
                        IrpContext,
                        Vcb,
                        Offset,
                        PAGE_SIZE as u32,
                        core::ptr::addr_of_mut!(SavedBcbs[Page as usize][0]).cast::<*mut BCB>(),
                        core::ptr::addr_of_mut!(SavedBcbs[Page as usize][1]) as *mut *mut core::ffi::c_void,
                        1,
                        0,
                    );

                    if Page == 0 {
                        FatEntry = (SavedBcbs[0][1] as *mut u8).add((StartOffsetInVolume % PAGE_SIZE as u32) as usize) as *mut u16;
                    }

                    Page += 1;
                    Offset += PAGE_SIZE as u32;
                }

                StartSectorLbo = StartOffsetInVolume & !(SectorSize - 1);
                FinalSectorLbo = FinalOffsetInVolume & !(SectorSize - 1);
                Lbo = StartSectorLbo;
                while Lbo <= FinalSectorLbo {
                    FatAddMcbEntry(
                        Vcb,
                        core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb),
                        Lbo as u32,
                        Lbo as i64,
                        SectorSize,
                    );
                    Lbo += SectorSize;
                }

                // ALPHA mutex omitted

                let mut Cluster = StartingFatIndex;
                let FinalCluster = StartingFatIndex + ClusterCount - 1;
                Page = 0;
                while Cluster <= FinalCluster {
                    if ((FatEntry as usize & (PAGE_SIZE - 1)) == 0) && (Cluster != StartingFatIndex) {
                        Page += 1;
                        FatEntry = SavedBcbs[Page as usize][1] as *mut u16;
                    }

                    *FatEntry = if ChainTogether != 0 {
                        (Cluster + 1) as u16
                    } else {
                        FAT_CLUSTER_AVAILABLE as u16
                    };

                    Cluster += 1;
                    FatEntry = FatEntry.add(1);
                }

                if ChainTogether != 0 {
                    *FatEntry.sub(1) = FAT_CLUSTER_LAST as u16;
                }
            }
        },
        |_, abnormal| {
            let mut i: usize = 0;
            DebugUnwind(b"FatSetFatRun\0".as_ptr() as *const i8);

            // If we still somehow have the Mutex, release it.
            if *p_ReleaseMutex != 0 {
                NT_ASSERT!(abnormal as i32);
                FatUnlockFreeClusterBitMap(Vcb);
            }

            while i < COUNTSAVEDBCBS && !(*p_SavedBcbs)[i][0].is_null() {
                FatUnpinBcb(IrpContext as *mut IRP_CONTEXT, core::ptr::addr_of_mut!((*p_SavedBcbs)[i][0]) as *mut *mut core::ffi::c_void);
                i += 1;
            }

            // Fat32 unwind: walk back over each FAT entry we have changed,
            // setting it back to the previous value.
            if abnormal && (*Vcb).AllocationSupport.FatIndexBitSize == 32 {
                while *p_StartingFatIndex > SavedStartingFatIndex {
                    *p_StartingFatIndex -= 1;
                    FatSetFatEntry(
                        IrpContext as *mut IRP_CONTEXT,
                        Vcb,
                        *p_StartingFatIndex,
                        if ChainTogether != 0 {
                            *p_StartingFatIndex + 1
                        } else {
                            FAT_CLUSTER_AVAILABLE
                        },
                    );
                }
            }

            DebugTrace(-1, Dbg, b"FatSetFatRun -> (VOID)\n\0".as_ptr() as *const i8, 0);
        },
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatLogOf(mut value: u32) -> u8 {
    let mut log: u8 = 0;
    let orig_value: u32 = value;

    while (value & 0xfffffffe) != 0 {
        log += 1;
        value >>= 1;
    }

    if value != 0x1 {
        DebugTrace(1, Dbg, c"LogOf\n".as_ptr() as *const i8, 0);
        DebugTrace(0, Dbg, c"  Value = %8lx\n".as_ptr() as *const i8, (orig_value) as usize);
        DebugTrace(0, Dbg, c"Received non power of 2.\n".as_ptr() as *const i8, 0);
        DebugTrace(-1, Dbg, c"LogOf -> %8lx\n".as_ptr() as *const i8, (log as u32) as usize);
        FatBugCheck(value as usize, log as usize, 0);
    }

    log
}
#[no_mangle]
pub unsafe extern "system" fn FatExamineFatEntries(
    irp_context: *mut IRP_CONTEXT,
    vcb: *mut VCB,
    mut start_index: u32,
    mut end_index: u32,
    setup_windows: i32,
    switch_to_window: *mut FAT_WINDOW,
    bit_map_buffer: *mut u32,
) {
    // Forward declarations for unknown symbols

    // Known extern functions from ntkrnl_sys

    // Types and constants
    type FAT_ENTRY = u32;
    const FAT_CLUSTER_AVAILABLE: u32 = 0;
    const FAT32_ENTRY_MASK: u32 = 0x0FFFFFFF;
    const MAX_CLUSTER_BITMAP_SIZE: u32 = 1 << 16;
    const FAT_PREFETCH_PAGE_COUNT: u32 = 0x100;
    const VCB_STATE_FLAG_BAD_BLOCKS_POPULATED: u32 = 0x00100000;
    const TAG_FAT_BITMAP: u32 = 0x42746146; // 'FTBM' in little-endian
    #[repr(u32)]
    enum CLUSTER_TYPE {
        FatClusterBad = 7,
    }
    const FAT_CLUSTER_BAD: u32 = 0x0ffffff7; // fat.h; the CLUSTER_TYPE discriminant is not the FAT entry value
    type PUSHORT = *mut u16;
    type PVCB = *mut VCB; // opaque

    // Opaque Bcb type
    #[allow(dead_code, non_camel_case_types)]
    struct Bcb;
    type PBCB = *mut Bcb;

    // Unknown function declarations

    // Local variables
    let mut fat_index_bit_size: u32 = 0;
    let mut page: u32 = 0;
    let mut offset: u32 = 0;
    let mut fat_index: u32 = 0;
    let mut fat_entry: u32 = FAT_CLUSTER_AVAILABLE;
    let mut first_fat_entry: u32 = FAT_CLUSTER_AVAILABLE;
    let mut fat_buffer: PUSHORT = core::ptr::null_mut();
    let mut pv: *mut u8 = core::ptr::null_mut();
    let mut bcb: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut entries_per_window: u32 = 0;

    let mut clusters_this_run: u32 = 0;
    let mut start_index_of_this_run: u32 = 0;

    let mut free_cluster_count: *mut u32 =
        &mut (*vcb).AllocationSupport.NumberOfFreeClusters;

    let mut current_window: *mut FAT_WINDOW = core::ptr::null_mut();
    let mut new_bit_map_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut bit_map: *mut RTL_BITMAP = core::ptr::null_mut();
    let mut private_bit_map: RTL_BITMAP = core::mem::zeroed();

    let mut cluster_size: u32 = 0;
    let mut prefetch_pages: u32 = 0;
    let mut fat_pages: u32 = 0;

    let mut bad_cluster_vbo: u32 = 0;
    let mut lbo: i64 = 0;

    #[repr(u32)]
    #[derive(PartialEq, Clone, Copy)]
    enum RunType {
        FreeClusters,
        AllocatedClusters,
        UnknownClusters,
    }
    let mut current_run: RunType = RunType::UnknownClusters;

    PAGED_CODE!();

    // ----------------------- START OF TRY BLOCK -----------------------
    {
        // --- Setup phase ---
        fat_index_bit_size = (*vcb).AllocationSupport.FatIndexBitSize as u32;

        NT_ASSERT!(!(setup_windows != 0 && (!switch_to_window.is_null() || !bit_map_buffer.is_null())) as i32);
        NT_ASSERT!(!(setup_windows != 0 && fat_index_bit_size != 32) as i32);

        if (*vcb).NumberOfWindows > 1 {
            entries_per_window = MAX_CLUSTER_BITMAP_SIZE;
        } else {
            entries_per_window = (*vcb).AllocationSupport.NumberOfClusters;
        }

        free_cluster_count = &mut (*vcb).AllocationSupport.NumberOfFreeClusters;

        if setup_windows != 0 {
            NT_ASSERT!(bit_map_buffer.is_null() as i32);

            bit_map = core::ptr::null_mut();
            current_window = (*vcb).Windows.add(0);
            (*current_window).FirstCluster = start_index;
            (*current_window).ClustersFree = 0;
        } else if bit_map_buffer.is_null() {
            // SwitchToWindow must be provided
            NT_ASSERT!(!switch_to_window.is_null() as i32);

            current_window = switch_to_window;
            start_index = (*current_window).FirstCluster;
            end_index = (*current_window).LastCluster;

            bit_map = &mut private_bit_map;
            new_bit_map_buffer = FsRtlAllocatePoolWithTag(
                PagedPool,
                (entries_per_window + 7) / 8,
                TAG_FAT_BITMAP,
            );

            RtlInitializeBitMap(
                &mut private_bit_map,
                new_bit_map_buffer as *mut u32,
                end_index - start_index + 1,
            );

            if fat_index_bit_size == 32 && (*vcb).NumberOfWindows > 1 {
                free_cluster_count = core::ptr::null_mut();
            }
        } else {
            bit_map = &mut private_bit_map;
            RtlInitializeBitMap(
                &mut private_bit_map,
                bit_map_buffer,
                end_index - start_index + 1,
            );

            free_cluster_count = core::ptr::null_mut();
        }

        NT_ASSERT!((start_index >= 2) as i32);
    }

    let p_bcb = core::ptr::addr_of_mut!(bcb);
    let p_new_bit_map_buffer = core::ptr::addr_of_mut!(new_bit_map_buffer);
    crate::seh::try_finally(
        &mut (),
        |_| {

            // --- Read initial FAT entry(es) ---
            if fat_index_bit_size == 12 {
                FatReadVolumeFile(
                    irp_context,
                    vcb,
                    FatReservedBytes(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void),
                    FatBytesPerFat(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void),
                    &mut bcb as *mut *mut core::ffi::c_void,
                    &mut pv as *mut *mut u8,
                );
                fat_buffer = pv as PUSHORT;
                FatLookup12BitEntry(fat_buffer as *mut u8, 0, &mut first_fat_entry);
            } else {
                let bytes_per_entry: u32 = fat_index_bit_size >> 3;
                fat_pages = (FatReservedBytes(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void) + FatBytesPerFat(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void) + ((PAGE_SIZE - 1)) as u32) / (PAGE_SIZE) as u32;
                page = (FatReservedBytes(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void) + start_index * bytes_per_entry) / (PAGE_SIZE) as u32;
                offset = page * (PAGE_SIZE) as u32;

                prefetch_pages = fat_pages - page;
                if prefetch_pages > FAT_PREFETCH_PAGE_COUNT {
                    prefetch_pages = ((page + FAT_PREFETCH_PAGE_COUNT - 1) & !(FAT_PREFETCH_PAGE_COUNT - 1)) - page;
                }

                // NTDDI_VERSION >= NTDDI_WIN8 is assumed true
                FatPrefetchPages(irp_context, (*vcb).VirtualVolumeFile, page, prefetch_pages);

                FatReadVolumeFile(irp_context, vcb, offset, PAGE_SIZE as u32, &mut bcb as *mut *mut core::ffi::c_void, &mut pv as *mut *mut u8);

                if fat_index_bit_size == 32 {
                    let pu8 = pv as *mut u8;
                    let offset_within_page = (FatReservedBytes(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void) + start_index * bytes_per_entry) % (PAGE_SIZE) as u32;
                    let pu32 = (pu8.add(offset_within_page as usize)) as *mut u32;
                    let tmp = *pu32;
                    first_fat_entry = tmp & FAT32_ENTRY_MASK;
                    fat_buffer = pu32 as PUSHORT; // store as u16* for iteration
                } else {
                    let pu8 = pv as *mut u8;
                    let pu16 = (pu8.add((FatReservedBytes(core::ptr::addr_of!((*vcb).Bpb) as *const core::ffi::c_void) % (PAGE_SIZE) as u32) as usize)) as PUSHORT;
                    fat_buffer = pu16.add(2); // skip two u16 entries
                    // C reads the first entry AFTER skipping the two reserved ones.
                    first_fat_entry = *fat_buffer as u32;
                    // Comment: Actually the C code:
                    // FatBuffer = (PUSHORT)((PUCHAR)pv + FatReservedBytes(&Vcb->Bpb) % PAGE_SIZE) + 2;
                    // FirstFatEntry = *FatBuffer;
                }
            }

            cluster_size = 1 << (*vcb).AllocationSupport.LogOfBytesPerCluster;

            current_run = if first_fat_entry == FAT_CLUSTER_AVAILABLE {
                RunType::FreeClusters
            } else {
                RunType::AllocatedClusters
            };
            start_index_of_this_run = start_index;

            // --- Main scan loop ---
            fat_index = start_index;
            while fat_index <= end_index { {
                // Read current fat entry
                if fat_index_bit_size == 12 {
                    FatLookup12BitEntry(fat_buffer as *mut u8, fat_index, &mut fat_entry);
                } else {
                    // FAT32 window boundary handling (setup mode)
                    if setup_windows != 0
                        && fat_index > start_index
                        && (fat_index - 2) % entries_per_window == 0
                    {
                        (*current_window).LastCluster = fat_index - 1;

                        if current_run == RunType::FreeClusters {
                            NT_ASSERT!(!free_cluster_count.is_null() as i32);
                            clusters_this_run = fat_index - start_index_of_this_run;
                            (*current_window).ClustersFree += clusters_this_run;
                            if !free_cluster_count.is_null() {
                                *free_cluster_count += clusters_this_run;
                            }
                        } else {
                            NT_ASSERT!((current_run == RunType::AllocatedClusters) as i32);
                        }

                        start_index_of_this_run = fat_index;
                        current_run = RunType::UnknownClusters;

                        current_window = current_window.add(1);
                        (*current_window).ClustersFree = 0;
                        (*current_window).FirstCluster = fat_index;
                    }

                    // Page boundary crossing
                    if (fat_buffer as usize & (PAGE_SIZE as usize - 1)) == 0 {
                        FatUnpinBcb(irp_context, core::ptr::addr_of_mut!((bcb)) as *mut *mut core::ffi::c_void);

                        page += 1;
                        offset += (PAGE_SIZE) as u32;

                        // Prefetch next chunk if exhausted
                        prefetch_pages -= 1;
                        if prefetch_pages == 0 {
                            prefetch_pages = fat_pages - page;
                            if prefetch_pages > FAT_PREFETCH_PAGE_COUNT {
                                prefetch_pages = FAT_PREFETCH_PAGE_COUNT;
                            }
                            FatPrefetchPages(irp_context, (*vcb).VirtualVolumeFile, page, prefetch_pages);
                        }

                        FatReadVolumeFile(irp_context, vcb, offset, PAGE_SIZE as u32, &mut bcb as *mut *mut core::ffi::c_void, &mut pv as *mut *mut u8);
                        fat_buffer = pv as PUSHORT;
                    }

                    // Decode entry based on FAT width
                    if fat_index_bit_size == 32 {
                        let pu32 = fat_buffer as *mut u32;
                        fat_entry = *pu32 & FAT32_ENTRY_MASK;
                        fat_buffer = fat_buffer.add(2);
                    } else {
                        fat_entry = (*fat_buffer) as u32;
                        fat_buffer = fat_buffer.add(1);
                    }
                } // end non-12-bit read

                if current_run == RunType::UnknownClusters {
                    current_run = if fat_entry == FAT_CLUSTER_AVAILABLE {
                        RunType::FreeClusters
                    } else {
                        RunType::AllocatedClusters
                    };
                }

                // Transition: Free -> Allocated
                if current_run == RunType::FreeClusters && fat_entry != FAT_CLUSTER_AVAILABLE {
                    clusters_this_run = fat_index - start_index_of_this_run;

                    if !free_cluster_count.is_null() {
                        *free_cluster_count += clusters_this_run;
                        (*current_window).ClustersFree += clusters_this_run;
                    }

                    if !bit_map.is_null() {
                        RtlClearBits(bit_map, start_index_of_this_run - start_index, clusters_this_run);
                    }

                    current_run = RunType::AllocatedClusters;
                    start_index_of_this_run = fat_index;
                }

                // Transition: Allocated -> Free
                if current_run == RunType::AllocatedClusters && fat_entry == FAT_CLUSTER_AVAILABLE {
                    clusters_this_run = fat_index - start_index_of_this_run;

                    if !bit_map.is_null() {
                        RtlSetBits((bit_map).cast(), start_index_of_this_run - start_index, clusters_this_run);
                    }

                    current_run = RunType::FreeClusters;
                    start_index_of_this_run = fat_index;
                }

                // Mark bad clusters
                if (setup_windows != 0 || (*vcb).NumberOfWindows == 1)
                    && FatInterpretClusterType(vcb, fat_entry) == FatClusterBad
                {
                    lbo = FatGetLboFromIndex(vcb, fat_index);
                    FatAddMcbEntry(
                        vcb,
                        &mut (*vcb).BadBlockMcb as *mut LARGE_MCB,
                        bad_cluster_vbo,
                        lbo,
                        cluster_size,
                    );
                    bad_cluster_vbo += cluster_size;
                }
            } // end loop body
            fat_index += 1;
            } // end while body
            // (fat_index is now end_index + 1 after the loop)

            // --- Post-loop finalization ---
            SetFlag!((*vcb).VcbState, VCB_STATE_FLAG_BAD_BLOCKS_POPULATED);

            clusters_this_run = fat_index - start_index_of_this_run;

            if current_run == RunType::FreeClusters {
                if !free_cluster_count.is_null() {
                    *free_cluster_count += clusters_this_run;
                    if !current_window.is_null() {
                        (*current_window).ClustersFree += clusters_this_run;
                    }
                }
                if !bit_map.is_null() {
                    RtlClearBits(bit_map, start_index_of_this_run - start_index, clusters_this_run);
                }
            } else {
                if !bit_map.is_null() {
                    RtlSetBits((bit_map).cast(), start_index_of_this_run - start_index, clusters_this_run);
                }
            }

            if setup_windows != 0 {
                if !current_window.is_null() {
                    (*current_window).LastCluster = fat_index - 1;
                }
            }

            // --- Switch active window if needed ---
            if !switch_to_window.is_null() {
                if !(*vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() {
                    ExFreePool((*vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer as *mut core::ffi::c_void);
                }

                RtlInitializeBitMap(
                    &mut (*vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP,
                    new_bit_map_buffer as *mut u32,
                    end_index - start_index + 1,
                );

                new_bit_map_buffer = core::ptr::null_mut();

                (*vcb).CurrentWindow = switch_to_window;
                (*vcb).ClusterHint = u32::MAX; // -1 cast to unsigned

                if !free_cluster_count.is_null() {
                    NT_ASSERT!((setup_windows == 0) as i32);
                    (*(*vcb).CurrentWindow).ClustersFree = *free_cluster_count;
                }
            }

            // --- Post-conditions ---
            if setup_windows == 0 && bit_map_buffer.is_null() {
                // ASSERT_CURRENT_WINDOW_GOOD inline expansion
                NT_ASSERT!(!(*vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() as i32);
                let free_cluster_bitmap_clear = RtlNumberOfClearBits((&mut (*vcb).FreeClusterBitMap as *mut RTL_BITMAP_DBG as *mut RTL_BITMAP).cast());
                let current_free = (*current_window).ClustersFree;
                if current_free != free_cluster_bitmap_clear {
                    DbgPrint(
                        b"FAT: ClustersFree %x h != FreeClusterBitMapClear %x h\n\0"
                            .as_ptr() as *const i8,
                        current_free,
                        free_cluster_bitmap_clear,
                    );
                }
                NT_ASSERT!((current_free == free_cluster_bitmap_clear) as i32);
            }

            NT_ASSERT!(((*vcb).AllocationSupport.NumberOfFreeClusters
                    <= (*vcb).AllocationSupport.NumberOfClusters)
                    as i32);
        },
        |_, _| {
            // Unpin the last bcb and drop the temporary bitmap buffer if it exists.
            FatUnpinBcb(irp_context, p_bcb as *mut *mut core::ffi::c_void);

            if !(*p_new_bit_map_buffer).is_null() {
                ExFreePool(*p_new_bit_map_buffer);
            }
        },
    );
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
//     AllocSup.c
//
// Abstract:
//
//     This module implements the Allocation support routines for Fat.
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_ALLOCSUP)
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_ALLOCSUP)
//
// #define FatMin(a, b)    ((a) < (b) ? (a) : (b))
//
// //
// //  Define prefetch page count for the FAT
// //
//
// #define FAT_PREFETCH_PAGE_COUNT          0x100
//
// //
// //  Local support routine prototypes
// //
//
// VOID
// FatLookupFatEntry(
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG FatIndex,
//     IN OUT PULONG FatEntry,
//     IN OUT PFAT_ENUMERATION_CONTEXT Context
//     );
//
// VOID
// FatSetFatRun(
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG StartingFatIndex,
//     IN ULONG ClusterCount,
//     IN BOOLEAN ChainTogether
//     );
//
// UCHAR
// FatLogOf(
//     IN ULONG Value
//     );
//
// //
// //  Note that the KdPrint below will ONLY fire when the assert does. Leave it
// //  alone.
// //
//
// #if DBG
// #define ASSERT_CURRENT_WINDOW_GOOD(VCB) {                                               \
//     ULONG FreeClusterBitMapClear;                                                       \
//     NT_ASSERT( (VCB)->FreeClusterBitMap.Buffer != NULL );                               \
//     FreeClusterBitMapClear = RtlNumberOfClearBits(&(VCB)->FreeClusterBitMap);           \
//     if ((VCB)->CurrentWindow->ClustersFree != FreeClusterBitMapClear) {                 \
//         KdPrint(("FAT: ClustersFree %x h != FreeClusterBitMapClear %x h\n",             \
//                  (VCB)->CurrentWindow->ClustersFree,                                    \
//                  FreeClusterBitMapClear));                                              \
//     }                                                                                   \
//     NT_ASSERT( (VCB)->CurrentWindow->ClustersFree == FreeClusterBitMapClear );          \
// }
// #else
// #define ASSERT_CURRENT_WINDOW_GOOD(VCB)
// #endif
//
// //
// //  The following macros provide a convenient way of hiding the details
// //  of bitmap allocation schemes.
// //
//
//
// //
// //  VOID
// //  FatLockFreeClusterBitMap (
// //      IN PVCB Vcb
// //      );
// //
//
// #define FatLockFreeClusterBitMap(VCB) {                         \
//     NT_ASSERT(KeAreApcsDisabled());                             \
//     ExAcquireFastMutexUnsafe( &(VCB)->FreeClusterBitMapMutex ); \
//     ASSERT_CURRENT_WINDOW_GOOD(VCB)                             \
// }
//
// //
// //  VOID
// //  FatUnlockFreeClusterBitMap (
// //      IN PVCB Vcb
// //      );
// //
//
// #define FatUnlockFreeClusterBitMap(VCB) {                       \
//     ASSERT_CURRENT_WINDOW_GOOD(VCB)                             \
//     NT_ASSERT(KeAreApcsDisabled());                             \
//     ExReleaseFastMutexUnsafe( &(VCB)->FreeClusterBitMapMutex ); \
// }
//
// //
// //  BOOLEAN
// //  FatIsClusterFree (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG FatIndex
// //      );
// //
//
// #define FatIsClusterFree(IRPCONTEXT,VCB,FAT_INDEX)                            \
//     (RtlCheckBit(&(VCB)->FreeClusterBitMap,(FAT_INDEX)-2) == 0)
//
// //
// //  VOID
// //  FatFreeClusters  (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG FatIndex,
// //      IN ULONG ClusterCount
// //      );
// //
//
// #define FatFreeClusters(IRPCONTEXT,VCB,FAT_INDEX,CLUSTER_COUNT) {             \
//     if ((CLUSTER_COUNT) == 1) {                                               \
//         FatSetFatEntry((IRPCONTEXT),(VCB),(FAT_INDEX),FAT_CLUSTER_AVAILABLE); \
//     } else {                                                                  \
//         FatSetFatRun((IRPCONTEXT),(VCB),(FAT_INDEX),(CLUSTER_COUNT),FALSE);   \
//     }                                                                         \
// }
//
// //
// //  VOID
// //  FatAllocateClusters  (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG FatIndex,
// //      IN ULONG ClusterCount
// //      );
// //
//
// #define FatAllocateClusters(IRPCONTEXT,VCB,FAT_INDEX,CLUSTER_COUNT) {      \
//     if ((CLUSTER_COUNT) == 1) {                                            \
//         FatSetFatEntry((IRPCONTEXT),(VCB),(FAT_INDEX),FAT_CLUSTER_LAST);   \
//     } else {                                                               \
//         FatSetFatRun((IRPCONTEXT),(VCB),(FAT_INDEX),(CLUSTER_COUNT),TRUE); \
//     }                                                                      \
// }
//
// //
// //  VOID
// //  FatUnreserveClusters  (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG FatIndex,
// //      IN ULONG ClusterCount
// //      );
// //
//
// #define FatUnreserveClusters(IRPCONTEXT,VCB,FAT_INDEX,CLUSTER_COUNT) {                      \
//     NT_ASSERT( (FAT_INDEX) + (CLUSTER_COUNT) - 2 <= (VCB)->FreeClusterBitMap.SizeOfBitMap );\
//     NT_ASSERT( (FAT_INDEX) >= 2);                                                           \
//     RtlClearBits(&(VCB)->FreeClusterBitMap,(FAT_INDEX)-2,(CLUSTER_COUNT));                  \
//     if ((FAT_INDEX) < (VCB)->ClusterHint) {                                                 \
//         (VCB)->ClusterHint = (FAT_INDEX);                                                   \
//     }                                                                                       \
// }
//
// //
// //  VOID
// //  FatReserveClusters  (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG FatIndex,
// //      IN ULONG ClusterCount
// //      );
// //
// //  Handle wrapping the hint back to the front.
// //
//
// #define FatReserveClusters(IRPCONTEXT,VCB,FAT_INDEX,CLUSTER_COUNT) {                        \
//     ULONG _AfterRun = (FAT_INDEX) + (CLUSTER_COUNT);                                        \
//     NT_ASSERT( (FAT_INDEX) + (CLUSTER_COUNT) - 2 <= (VCB)->FreeClusterBitMap.SizeOfBitMap );\
//     NT_ASSERT( (FAT_INDEX) >= 2);                                                           \
//     RtlSetBits(&(VCB)->FreeClusterBitMap,(FAT_INDEX)-2,(CLUSTER_COUNT));                    \
//                                                                                             \
//     if (_AfterRun - 2 >= (VCB)->FreeClusterBitMap.SizeOfBitMap) {                           \
//         _AfterRun = 2;                                                                      \
//     }                                                                                       \
//     if (RtlCheckBit(&(VCB)->FreeClusterBitMap, _AfterRun - 2))  {                                   \
//         (VCB)->ClusterHint = RtlFindClearBits( &(VCB)->FreeClusterBitMap, 1, _AfterRun - 2) + 2;    \
//         if (1 == (VCB)->ClusterHint)  {                                                             \
//             (VCB)->ClusterHint = 2;                                                         \
//         }                                                                                   \
//     }                                                                                       \
//     else {                                                                                  \
//         (VCB)->ClusterHint = _AfterRun;                                                     \
//     }                                                                                       \
// }
//
// //
// //  ULONG
// //  FatFindFreeClusterRun (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PVCB Vcb,
// //      IN ULONG ClusterCount,
// //      IN ULONG AlternateClusterHint
// //      );
// //
// //  Do a special check if only one cluster is desired.
// //
//
// #define FatFindFreeClusterRun(IRPCONTEXT,VCB,CLUSTER_COUNT,CLUSTER_HINT) ( \
//     (CLUSTER_COUNT == 1) &&                                                \
//     FatIsClusterFree((IRPCONTEXT), (VCB), (CLUSTER_HINT)) ?                \
//         (CLUSTER_HINT) :                                                   \
//         RtlFindClearBits( &(VCB)->FreeClusterBitMap,                       \
//                           (CLUSTER_COUNT),                                 \
//                           (CLUSTER_HINT) - 2) + 2                          \
// )
//
// //
// //  FAT32: Define the maximum size of the FreeClusterBitMap to be the
// //  maximum size of a FAT16 FAT.  If there are more clusters on the
// //  volume than can be represented by this many bytes of bitmap, the
// //  FAT will be split into "buckets", each of which does fit.
// //
// //  Note this count is in clusters/bits of bitmap.
// //
//
// #define MAX_CLUSTER_BITMAP_SIZE         (1 << 16)
//
// //
// //  Calculate the window a given cluster number is in.
// //
//
// #define FatWindowOfCluster(C)           (((C) - 2) / MAX_CLUSTER_BITMAP_SIZE)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatAddFileAllocation)
// #pragma alloc_text(PAGE, FatAllocateDiskSpace)
// #pragma alloc_text(PAGE, FatDeallocateDiskSpace)
// #pragma alloc_text(PAGE, FatExamineFatEntries)
// #pragma alloc_text(PAGE, FatInterpretClusterType)
// #pragma alloc_text(PAGE, FatLogOf)
// #pragma alloc_text(PAGE, FatLookupFatEntry)
// #pragma alloc_text(PAGE, FatLookupFileAllocation)
// #pragma alloc_text(PAGE, FatLookupFileAllocationSize)
// #pragma alloc_text(PAGE, FatMergeAllocation)
// #pragma alloc_text(PAGE, FatSetFatEntry)
// #pragma alloc_text(PAGE, FatSetFatRun)
// #pragma alloc_text(PAGE, FatSetupAllocationSupport)
// #pragma alloc_text(PAGE, FatSplitAllocation)
// #pragma alloc_text(PAGE, FatTearDownAllocationSupport)
// #pragma alloc_text(PAGE, FatTruncateFileAllocation)
// #endif
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// ULONG
// FatSelectBestWindow(
//     IN PVCB Vcb
//     )
// /*++
//
// Routine Description:
//
//     Choose a window to allocate clusters from.   Order of preference is:
//
//     1.  First window with >50% free clusters
//     2.  First empty window
//     3.  Window with greatest number of free clusters.
//
// Arguments:
//
//     Vcb - Supplies the Vcb for the volume
//
// Return Value:
//
//     'Best window' number (index into Vcb->Windows[])
//
// --*/
// {
//     ULONG i, Fave = 0;
//     ULONG MaxFree = 0;
//     ULONG FirstEmpty = (ULONG)-1;
//     ULONG ClustersPerWindow = MAX_CLUSTER_BITMAP_SIZE;
//
//     NT_ASSERT( 1 != Vcb->NumberOfWindows);
//
//     for (i = 0; i < Vcb->NumberOfWindows; i++) {
//
//         if (Vcb->Windows[i].ClustersFree == ClustersPerWindow)  {
//
//             if (-1 == FirstEmpty)  {
//
//                 //
//                 //  Keep note of the first empty window on the disc
//                 //
//
//                 FirstEmpty = i;
//             }
//         }
//         else if (Vcb->Windows[i].ClustersFree > MaxFree)  {
//
//             //
//             //  This window has the most free clusters,  so far
//             //
//
//             MaxFree = Vcb->Windows[i].ClustersFree;
//             Fave = i;
//
//             //
//             //  If this window has >50% free clusters,  then we will take it,
//             //  so don't bother considering more windows.
//             //
//
//             if (MaxFree >= (ClustersPerWindow >> 1))  {
//
//                 break;
//             }
//         }
//     }
//
//     //
//     //  If there were no windows with 50% or more freespace,  then select the
//     //  first empty window on the disc,  if any - otherwise we'll just go with
//     //  the one with the most free clusters.
//     //
//
//     if ((MaxFree < (ClustersPerWindow >> 1)) && (-1 != FirstEmpty))  {
//
//         Fave = FirstEmpty;
//     }
//
//     return Fave;
// }
//
//
// VOID
// FatSetupAllocationSupport (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine fills in the Allocation Support structure in the Vcb.
//     Most entries are computed using fat.h macros supplied with data from
//     the Bios Parameter Block.  The free cluster count, however, requires
//     going to the Fat and actually counting free sectors.  At the same time
//     the free cluster bit map is initalized.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to fill in.
//
// --*/
//
// {
//     ULONG BitIndex;
//     ULONG ClustersDescribableByFat;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSetupAllocationSupport\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb = %p\n", Vcb);
//
//     //
//     //  Compute a number of fields for Vcb.AllocationSupport
//     //
//
//     Vcb->AllocationSupport.RootDirectoryLbo = FatRootDirectoryLbo( &Vcb->Bpb );
//     Vcb->AllocationSupport.RootDirectorySize = FatRootDirectorySize( &Vcb->Bpb );
//
//     Vcb->AllocationSupport.FileAreaLbo = FatFileAreaLbo( &Vcb->Bpb );
//
//     Vcb->AllocationSupport.NumberOfClusters = FatNumberOfClusters( &Vcb->Bpb );
//
//     Vcb->AllocationSupport.FatIndexBitSize = FatIndexBitSize( &Vcb->Bpb );
//
//     Vcb->AllocationSupport.LogOfBytesPerSector = FatLogOf(Vcb->Bpb.BytesPerSector);
//     Vcb->AllocationSupport.LogOfBytesPerCluster = FatLogOf(FatBytesPerCluster( &Vcb->Bpb ));
//     Vcb->AllocationSupport.NumberOfFreeClusters = 0;
//
//
//     //
//     //  Deal with a bug in DOS 5 format, if the Fat is not big enough to
//     //  describe all the clusters on the disk, reduce this number.  We expect
//     //  that fat32 volumes will not have this problem.
//     //
//     //  Turns out this was not a good assumption.  We have to do this always now.
//     //
//
//     ClustersDescribableByFat = ( ((FatIsFat32(Vcb)? Vcb->Bpb.LargeSectorsPerFat :
//                                                     Vcb->Bpb.SectorsPerFat) *
//                                   Vcb->Bpb.BytesPerSector * 8)
//                                  / FatIndexBitSize(&Vcb->Bpb) ) - 2;
//
//     if (Vcb->AllocationSupport.NumberOfClusters > ClustersDescribableByFat) {
//
//         Vcb->AllocationSupport.NumberOfClusters = ClustersDescribableByFat;
//     }
//
//     //
//     //  Extend the virtual volume file to include the Fat
//     //
//
//     {
//         CC_FILE_SIZES FileSizes;
//
//         FileSizes.AllocationSize.QuadPart =
//         FileSizes.FileSize.QuadPart = (FatReservedBytes( &Vcb->Bpb ) +
//                                        FatBytesPerFat( &Vcb->Bpb ));
//         FileSizes.ValidDataLength = FatMaxLarge;
//
//         if ( Vcb->VirtualVolumeFile->PrivateCacheMap == NULL ) {
//
//             FatInitializeCacheMap( Vcb->VirtualVolumeFile,
//                                    &FileSizes,
//                                    TRUE,
//                                    &FatData.CacheManagerNoOpCallbacks,
//                                    Vcb );
//
//         } else {
//
//             CcSetFileSizes( Vcb->VirtualVolumeFile, &FileSizes );
//         }
//     }
//
//     _SEH2_TRY {
//
//         if (FatIsFat32(Vcb) &&
//             Vcb->AllocationSupport.NumberOfClusters > MAX_CLUSTER_BITMAP_SIZE) {
//
//             Vcb->NumberOfWindows = (Vcb->AllocationSupport.NumberOfClusters +
//                                     MAX_CLUSTER_BITMAP_SIZE - 1) /
//                                    MAX_CLUSTER_BITMAP_SIZE;
//
//         } else {
//
//             Vcb->NumberOfWindows = 1;
//         }
//
//         Vcb->Windows = FsRtlAllocatePoolWithTag( PagedPool,
//                                                  Vcb->NumberOfWindows * sizeof(FAT_WINDOW),
//                                                  TAG_FAT_WINDOW );
//
//         RtlInitializeBitMap( &Vcb->FreeClusterBitMap,
//                              NULL,
//                              0 );
//
//         //
//         //  Chose a FAT window to begin operation in.
//         //
//
//         if (Vcb->NumberOfWindows > 1) {
//
//             //
//             //  Read the fat and count up free clusters.  We bias by the two reserved
//             //  entries in the FAT.
//             //
//
//             FatExamineFatEntries( IrpContext, Vcb,
//                                   2,
//                                   Vcb->AllocationSupport.NumberOfClusters + 2 - 1,
//                                   TRUE,
//                                   NULL,
//                                   NULL);
//
//
//             //
//             //  Pick a window to begin allocating from
//             //
//
//             Vcb->CurrentWindow = &Vcb->Windows[ FatSelectBestWindow( Vcb)];
//
//         } else {
//
//             Vcb->CurrentWindow = &Vcb->Windows[0];
//
//             //
//             //  Carefully bias ourselves by the two reserved entries in the FAT.
//             //
//
//             Vcb->CurrentWindow->FirstCluster = 2;
//             Vcb->CurrentWindow->LastCluster = Vcb->AllocationSupport.NumberOfClusters + 2 - 1;
//         }
//
//         //
//         //  Now transition to the FAT window we have chosen.
//         //
//
//         FatExamineFatEntries( IrpContext, Vcb,
//                               0,
//                               0,
//                               FALSE,
//                               Vcb->CurrentWindow,
//                               NULL);
//
//         //
//         //  Now set the ClusterHint to the first free bit in our favorite
//         //  window (except the ClusterHint is off by two).
//         //
//
//         Vcb->ClusterHint =
//             (BitIndex = RtlFindClearBits( &Vcb->FreeClusterBitMap, 1, 0 )) != -1 ?
//                 BitIndex + 2 : 2;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetupAllocationSupport );
//
//         //
//         //  If we hit an exception, back out.
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             FatTearDownAllocationSupport( IrpContext, Vcb );
//         }
//     } _SEH2_END;
//
//     return;
// }
//
//
// VOID
// FatTearDownAllocationSupport (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine prepares the volume for closing.  Specifically, we must
//     release the free fat bit map buffer, and uninitialize the dirty fat
//     Mcb.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to fill in.
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
//     DebugTrace(+1, Dbg, "FatTearDownAllocationSupport\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb = %p\n", Vcb);
//
//     PAGED_CODE();
//
//     //
//     //  If there are FAT buckets, free them.
//     //
//
//     if ( Vcb->Windows != NULL ) {
//
//         ExFreePool( Vcb->Windows );
//         Vcb->Windows = NULL;
//     }
//
//     //
//     //  Free the memory associated with the free cluster bitmap.
//     //
//
//     if ( Vcb->FreeClusterBitMap.Buffer != NULL ) {
//
//         ExFreePool( Vcb->FreeClusterBitMap.Buffer );
//
//         //
//         //  NULL this field as an flag.
//         //
//
//         Vcb->FreeClusterBitMap.Buffer = NULL;
//     }
//
//     //
//     //  And remove all the runs in the dirty fat Mcb
//     //
//
//     FatRemoveMcbEntry( Vcb, &Vcb->DirtyFatMcb, 0, 0xFFFFFFFF );
//
//     DebugTrace(-1, Dbg, "FatTearDownAllocationSupport -> (VOID)\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatLookupFileAllocation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN VBO Vbo,
//     OUT PLBO Lbo,
//     OUT PULONG ByteCount,
//     OUT PBOOLEAN Allocated,
//     OUT PBOOLEAN EndOnMax,
//     OUT PULONG Index
//     )
//
// /*++
//
// Routine Description:
//
//     This routine looks up the existing mapping of VBO to LBO for a
//     file/directory.  The information it queries is either stored in the
//     mcb field of the fcb/dcb or it is stored on in the fat table and
//     needs to be retrieved and decoded, and updated in the mcb.
//
// Arguments:
//
//     FcbOrDcb - Supplies the Fcb/Dcb of the file/directory being queried
//
//     Vbo - Supplies the VBO whose LBO we want returned
//
//     Lbo - Receives the LBO corresponding to the input Vbo if one exists
//
//     ByteCount - Receives the number of bytes within the run the run
//                 that correpond between the input vbo and output lbo.
//
//     Allocated - Receives TRUE if the Vbo does have a corresponding Lbo
//                 and FALSE otherwise.
//
//     EndOnMax - Receives TRUE if the run ends in the maximal FAT cluster,
//                 which results in a fractional bytecount.
//
//     Index - Receives the Index of the run
//
// --*/
//
// {
//     VBO CurrentVbo;
//     LBO CurrentLbo;
//     LBO PriorLbo;
//
//     VBO FirstVboOfCurrentRun = 0;
//     LBO FirstLboOfCurrentRun;
//
//     BOOLEAN LastCluster;
//     ULONG Runs;
//
//     PVCB Vcb;
//     FAT_ENTRY FatEntry;
//     ULONG BytesPerCluster;
//     ULARGE_INTEGER BytesOnVolume;
//
//     FAT_ENUMERATION_CONTEXT Context;
//
//     PAGED_CODE();
//
//     Vcb = FcbOrDcb->Vcb;
//
//
//     DebugTrace(+1, Dbg, "FatLookupFileAllocation\n", 0);
//     DebugTrace( 0, Dbg, "  FcbOrDcb  = %p\n", FcbOrDcb);
//     DebugTrace( 0, Dbg, "  Vbo       = %8lx\n", Vbo);
//     DebugTrace( 0, Dbg, "  pLbo       = %8lx\n", Lbo);
//     DebugTrace( 0, Dbg, "  pByteCount = %8lx\n", ByteCount);
//     DebugTrace( 0, Dbg, "  pAllocated = %8lx\n", Allocated);
//
//     Context.Bcb = NULL;
//
//     *EndOnMax = FALSE;
//
//     //
//     //  Check the trivial case that the mapping is already in our
//     //  Mcb.
//     //
//
//     if ( FatLookupMcbEntry(Vcb, &FcbOrDcb->Mcb, Vbo, Lbo, ByteCount, Index) ) {
//
//         *Allocated = TRUE;
//
//         NT_ASSERT( *ByteCount != 0 );
//
//         //
//         //  Detect the overflow case, trim and claim the condition.
//         //
//
//         if (Vbo + *ByteCount == 0) {
//
//             *EndOnMax = TRUE;
//         }
//
//         DebugTrace( 0, Dbg, "Found run in Mcb.\n", 0);
//         DebugTrace(-1, Dbg, "FatLookupFileAllocation -> (VOID)\n", 0);
//         return;
//     }
//
//     //
//     //  Initialize the Vcb, the cluster size, LastCluster, and
//     //  FirstLboOfCurrentRun (to be used as an indication of the first
//     //  iteration through the following while loop).
//     //
//
//     BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     BytesOnVolume.QuadPart = UInt32x32To64( Vcb->AllocationSupport.NumberOfClusters, BytesPerCluster );
//
//     LastCluster = FALSE;
//     FirstLboOfCurrentRun = 0;
//
//     //
//     //  Discard the case that the request extends beyond the end of
//     //  allocation.  Note that if the allocation size if not known
//     //  AllocationSize is set to 0xffffffff.
//     //
//
//     if ( Vbo >= FcbOrDcb->Header.AllocationSize.LowPart ) {
//
//         *Allocated = FALSE;
//
//         DebugTrace( 0, Dbg, "Vbo beyond end of file.\n", 0);
//         DebugTrace(-1, Dbg, "FatLookupFileAllocation -> (VOID)\n", 0);
//         return;
//     }
//
//     //
//     //  The Vbo is beyond the last Mcb entry.  So we adjust Current Vbo/Lbo
//     //  and FatEntry to describe the beginning of the last entry in the Mcb.
//     //  This is used as initialization for the following loop.
//     //
//     //  If the Mcb was empty, we start at the beginning of the file with
//     //  CurrentVbo set to 0 to indicate a new run.
//     //
//
//     if (FatLookupLastMcbEntry( Vcb, &FcbOrDcb->Mcb, &CurrentVbo, &CurrentLbo, &Runs )) {
//
//         DebugTrace( 0, Dbg, "Current Mcb size = %8lx.\n", CurrentVbo + 1);
//
//         CurrentVbo -= (BytesPerCluster - 1);
//         CurrentLbo -= (BytesPerCluster - 1);
//
//         //
//         //  Convert an index to a count.
//         //
//
//         Runs += 1;
//
//     } else {
//
//         DebugTrace( 0, Dbg, "Mcb empty.\n", 0);
//
//         //
//         //  Check for an FcbOrDcb that has no allocation
//         //
//
//         if (FcbOrDcb->FirstClusterOfFile == 0) {
//
//             *Allocated = FALSE;
//
//             DebugTrace( 0, Dbg, "File has no allocation.\n", 0);
//             DebugTrace(-1, Dbg, "FatLookupFileAllocation -> (VOID)\n", 0);
//             return;
//
//         } else {
//
//             CurrentVbo = 0;
//             CurrentLbo = FatGetLboFromIndex( Vcb, FcbOrDcb->FirstClusterOfFile );
//             FirstVboOfCurrentRun = CurrentVbo;
//             FirstLboOfCurrentRun = CurrentLbo;
//
//             Runs = 0;
//
//             DebugTrace( 0, Dbg, "First Lbo of file = %8lx\n", CurrentLbo);
//         }
//     }
//
//     //
//     //  Now we know that we are looking up a valid Vbo, but it is
//     //  not in the Mcb, which is a monotonically increasing list of
//     //  Vbo's.  Thus we have to go to the Fat, and update
//     //  the Mcb as we go.  We use a try-finally to unpin the page
//     //  of fat hanging around.  Also we mark *Allocated = FALSE, so that
//     //  the caller wont try to use the data if we hit an exception.
//     //
//
//     *Allocated = FALSE;
//
//     _SEH2_TRY {
//
//         FatEntry = (FAT_ENTRY)FatGetIndexFromLbo( Vcb, CurrentLbo );
//
//         //
//         //  ASSERT that CurrentVbo and CurrentLbo are now cluster alligned.
//         //  The assumption here, is that only whole clusters of Vbos and Lbos
//         //  are mapped in the Mcb.
//         //
//
//         NT_ASSERT( ((CurrentLbo - Vcb->AllocationSupport.FileAreaLbo)
//                                                     % BytesPerCluster == 0) &&
//                 (CurrentVbo % BytesPerCluster == 0) );
//
//         //
//         //  Starting from the first Vbo after the last Mcb entry, scan through
//         //  the Fat looking for our Vbo. We continue through the Fat until we
//         //  hit a noncontiguity beyond the desired Vbo, or the last cluster.
//         //
//
//         while ( !LastCluster ) {
//
//             //
//             //  Get the next fat entry, and update our Current variables.
//             //
//
//             FatLookupFatEntry( IrpContext, Vcb, FatEntry, (PULONG)&FatEntry, &Context );
//
//             PriorLbo = CurrentLbo;
//             CurrentLbo = FatGetLboFromIndex( Vcb, FatEntry );
//             CurrentVbo += BytesPerCluster;
//
//             switch ( FatInterpretClusterType( Vcb, FatEntry )) {
//
//             //
//             //  Check for a break in the Fat allocation chain.
//             //
//
//             case FatClusterAvailable:
//             case FatClusterReserved:
//             case FatClusterBad:
//
//                 DebugTrace( 0, Dbg, "Break in allocation chain, entry = %d\n", FatEntry);
//                 DebugTrace(-1, Dbg, "FatLookupFileAllocation -> Fat Corrupt.  Raise Status.\n", 0);
//
//                 FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                 break;
//
//             //
//             //  If this is the last cluster, we must update the Mcb and
//             //  exit the loop.
//             //
//
//             case FatClusterLast:
//
//                 //
//                 //  Assert we know where the current run started.  If the
//                 //  Mcb was empty when we were called, thenFirstLboOfCurrentRun
//                 //  was set to the start of the file.  If the Mcb contained an
//                 //  entry, then FirstLboOfCurrentRun was set on the first
//                 //  iteration through the loop.  Thus if FirstLboOfCurrentRun
//                 //  is 0, then there was an Mcb entry and we are on our first
//                 //  iteration, meaing that the last cluster in the Mcb was
//                 //  really the last allocated cluster, but we checked Vbo
//                 //  against AllocationSize, and found it OK, thus AllocationSize
//                 //  must be too large.
//                 //
//                 //  Note that, when we finally arrive here, CurrentVbo is actually
//                 //  the first Vbo beyond the file allocation and CurrentLbo is
//                 //  meaningless.
//                 //
//
//                 DebugTrace( 0, Dbg, "Read last cluster of file.\n", 0);
//
//                 //
//                 //  Detect the case of the maximal file.  Note that this really isn't
//                 //  a proper Vbo - those are zero-based, and this is a one-based number.
//                 //  The maximal file, of 2^32 - 1 bytes, has a maximum byte offset of
//                 //  2^32 - 2.
//                 //
//                 //  Just so we don't get confused here.
//                 //
//
//                 if (CurrentVbo == 0) {
//
//                     *EndOnMax = TRUE;
//                     CurrentVbo -= 1;
//                 }
//
//                 LastCluster = TRUE;
//
//                 if (FirstLboOfCurrentRun != 0 ) {
//
//                     DebugTrace( 0, Dbg, "Adding a run to the Mcb.\n", 0);
//                     DebugTrace( 0, Dbg, "  Vbo    = %08lx.\n", FirstVboOfCurrentRun);
//                     DebugTrace( 0, Dbg, "  Lbo    = %08lx.\n", FirstLboOfCurrentRun);
//                     DebugTrace( 0, Dbg, "  Length = %08lx.\n", CurrentVbo - FirstVboOfCurrentRun);
//
//                     (VOID)FatAddMcbEntry( Vcb,
//                                           &FcbOrDcb->Mcb,
//                                           FirstVboOfCurrentRun,
//                                           FirstLboOfCurrentRun,
//                                           CurrentVbo - FirstVboOfCurrentRun );
//
//                     Runs += 1;
//                 }
//
//                 //
//                 //  Being at the end of allocation, make sure we have found
//                 //  the Vbo.  If we haven't, seeing as we checked VBO
//                 //  against AllocationSize, the real disk allocation is less
//                 //  than that of AllocationSize.  This comes about when the
//                 //  real allocation is not yet known, and AllocaitonSize
//                 //  contains MAXULONG.
//                 //
//                 //  KLUDGE! - If we were called by FatLookupFileAllocationSize
//                 //  Vbo is set to MAXULONG - 1, and AllocationSize to the lookup
//                 //  hint. Thus we merrily go along looking for a match that isn't
//                 //  there, but in the meantime building an Mcb.  If this is
//                 //  the case, fill in AllocationSize and return.
//                 //
//
//                 if ( Vbo == MAXULONG - 1 ) {
//
//                     *Allocated = FALSE;
//
//                     FcbOrDcb->Header.AllocationSize.QuadPart = CurrentVbo;
//
//                     DebugTrace( 0, Dbg, "New file allocation size = %08lx.\n", CurrentVbo);
//                     try_return ( NOTHING );
//                 }
//
//                 //
//                 //  We will lie ever so slightly if we really terminated on the
//                 //  maximal byte of a file.  It is really allocated.
//                 //
//
//                 if (Vbo >= CurrentVbo && !*EndOnMax) {
//
//                     *Allocated = FALSE;
//                     try_return ( NOTHING );
//                 }
//
//                 break;
//
//             //
//             //  This is a continuation in the chain.  If the run has a
//             //  discontiguity at this point, update the Mcb, and if we are beyond
//             //  the desired Vbo, this is the end of the run, so set LastCluster
//             //  and exit the loop.
//             //
//
//             case FatClusterNext:
//
//                 //
//                 //  This is the loop check.  The Vbo must not be bigger than the size of
//                 //  the volume, and the Vbo must not have a) wrapped and b) not been at the
//                 //  very last cluster in the chain, for the case of the maximal file.
//                 //
//
//                 if ( CurrentVbo == 0 ||
//                      (BytesOnVolume.HighPart == 0 && CurrentVbo > BytesOnVolume.LowPart)) {
//
//                     FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//                     FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                 }
//
//                 if ( PriorLbo + BytesPerCluster != CurrentLbo ) {
//
//                     //
//                     //  Note that on the first time through the loop
//                     //  (FirstLboOfCurrentRun == 0), we don't add the
//                     //  run to the Mcb since it curresponds to the last
//                     //  run already stored in the Mcb.
//                     //
//
//                     if ( FirstLboOfCurrentRun != 0 ) {
//
//                         DebugTrace( 0, Dbg, "Adding a run to the Mcb.\n", 0);
//                         DebugTrace( 0, Dbg, "  Vbo    = %08lx.\n", FirstVboOfCurrentRun);
//                         DebugTrace( 0, Dbg, "  Lbo    = %08lx.\n", FirstLboOfCurrentRun);
//                         DebugTrace( 0, Dbg, "  Length = %08lx.\n", CurrentVbo - FirstVboOfCurrentRun);
//
//                         FatAddMcbEntry( Vcb,
//                                         &FcbOrDcb->Mcb,
//                                         FirstVboOfCurrentRun,
//                                         FirstLboOfCurrentRun,
//                                         CurrentVbo - FirstVboOfCurrentRun );
//
//                         Runs += 1;
//                     }
//
//                     //
//                     //  Since we are at a run boundry, with CurrentLbo and
//                     //  CurrentVbo being the first cluster of the next run,
//                     //  we see if the run we just added encompases the desired
//                     //  Vbo, and if so exit.  Otherwise we set up two new
//                     //  First*boOfCurrentRun, and continue.
//                     //
//
//                     if (CurrentVbo > Vbo) {
//
//                         LastCluster = TRUE;
//
//                     } else {
//
//                         FirstVboOfCurrentRun = CurrentVbo;
//                         FirstLboOfCurrentRun = CurrentLbo;
//                     }
//                 }
//                 break;
//
//             default:
//
//                 DebugTrace(0, Dbg, "Illegal Cluster Type.\n", FatEntry);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "we bugcheck here because our internal data structures are seriously corrupted if this happens" )
// #endif
//                 FatBugCheck( 0, 0, 0 );
//
//                 break;
//
//             } // switch()
//         } // while()
//
//         //
//         //  Load up the return parameters.
//         //
//         //  On exit from the loop, Vbo still contains the desired Vbo, and
//         //  CurrentVbo is the first byte after the run that contained the
//         //  desired Vbo.
//         //
//
//         *Allocated = TRUE;
//
//         *Lbo = FirstLboOfCurrentRun + (Vbo - FirstVboOfCurrentRun);
//
//         *ByteCount = CurrentVbo - Vbo;
//
//         if (ARGUMENT_PRESENT(Index)) {
//
//             //
//             //  Note that Runs only needs to be accurate with respect to where we
//             //  ended.  Since partial-lookup cases will occur without exclusive
//             //  synchronization, the Mcb itself may be much bigger by now.
//             //
//
//             *Index = Runs - 1;
//         }
//
//     try_exit: NOTHING;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatLookupFileAllocation );
//
//         //
//         //  We are done reading the Fat, so unpin the last page of fat
//         //  that is hanging around
//         //
//
//         FatUnpinBcb( IrpContext, Context.Bcb );
//
//         DebugTrace(-1, Dbg, "FatLookupFileAllocation -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatAddFileAllocation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN PFILE_OBJECT FileObject OPTIONAL,
//     IN ULONG DesiredAllocationSize
//     )
//
// /*++
//
// Routine Description:
//
//     This routine adds additional allocation to the specified file/directory.
//     Additional allocation is added by appending clusters to the file/directory.
//
//     If the file already has a sufficient allocation then this procedure
//     is effectively a noop.
//
// Arguments:
//
//     FcbOrDcb - Supplies the Fcb/Dcb of the file/directory being modified.
//                This parameter must not specify the root dcb.
//
//     FileObject - If supplied inform the cache manager of the change.
//
//     DesiredAllocationSize - Supplies the minimum size, in bytes, that we want
//                             allocated to the file/directory.
//
// --*/
//
// {
//     PVCB Vcb;
//     LARGE_MCB NewMcb = {0};
//     PLARGE_MCB McbToCleanup = NULL;
//     PDIRENT Dirent = NULL;
//     ULONG NewAllocation = 0;
//     PBCB Bcb = NULL;
//     BOOLEAN UnwindWeAllocatedDiskSpace = FALSE;
//     BOOLEAN UnwindAllocationSizeSet = FALSE;
//     BOOLEAN UnwindCacheManagerInformed = FALSE;
//     BOOLEAN UnwindWeInitializedMcb = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatAddFileAllocation\n", 0);
//     DebugTrace( 0, Dbg, "  FcbOrDcb  =             %p\n", FcbOrDcb);
//     DebugTrace( 0, Dbg, "  DesiredAllocationSize = %8lx\n", DesiredAllocationSize);
//
//     Vcb = FcbOrDcb->Vcb;
//
//     //
//     //  If we haven't yet set the correct AllocationSize, do so.
//     //
//
//     if (FcbOrDcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT)  {
//
//         FatLookupFileAllocationSize( IrpContext, FcbOrDcb );
//     }
//
//     //
//     //  Check for the benign case that the desired allocation is already
//     //  within the allocation size.
//     //
//
//     if (DesiredAllocationSize <= FcbOrDcb->Header.AllocationSize.LowPart) {
//
//         DebugTrace(0, Dbg, "Desired size within current allocation.\n", 0);
//
//         DebugTrace(-1, Dbg, "FatAddFileAllocation -> (VOID)\n", 0);
//         return;
//     }
//
//     DebugTrace( 0, Dbg, "InitialAllocation = %08lx.\n", FcbOrDcb->Header.AllocationSize.LowPart);
//
//     //
//     //  Get a chunk of disk space that will fullfill our needs.  If there
//     //  was no initial allocation, start from the hint in the Vcb, otherwise
//     //  try to allocate from the cluster after the initial allocation.
//     //
//     //  If there was no initial allocation to the file, we can just use the
//     //  Mcb in the FcbOrDcb, otherwise we have to use a new one, and merge
//     //  it to the one in the FcbOrDcb.
//     //
//
//     _SEH2_TRY {
//
//         if (FcbOrDcb->Header.AllocationSize.LowPart == 0) {
//
//             LBO FirstLboOfFile;
//
//             NT_ASSERT( FcbOrDcb->FcbCondition == FcbGood );
//
//             FatGetDirentFromFcbOrDcb( IrpContext,
//                                       FcbOrDcb,
//                                       FALSE,
//                                       &Dirent,
//                                       &Bcb );
//             //
//             //  Set this dirty right now since this call can fail.
//             //
//
//             FatSetDirtyBcb( IrpContext, Bcb, Vcb, TRUE );
//
//             FatAllocateDiskSpace( IrpContext,
//                                   Vcb,
//                                   0,
//                                   &DesiredAllocationSize,
//                                   FALSE,
//                                   &FcbOrDcb->Mcb );
//
//             UnwindWeAllocatedDiskSpace = TRUE;
//             McbToCleanup = &FcbOrDcb->Mcb;
//
//             //
//             //  We have to update the dirent and FcbOrDcb copies of
//             //  FirstClusterOfFile since before it was 0
//             //
//
//             FatLookupMcbEntry( FcbOrDcb->Vcb,
//                                &FcbOrDcb->Mcb,
//                                0,
//                                &FirstLboOfFile,
//                                (PULONG)NULL,
//                                NULL );
//
//             DebugTrace( 0, Dbg, "First Lbo of file will be %08lx.\n", FirstLboOfFile );
//
//             FcbOrDcb->FirstClusterOfFile = FatGetIndexFromLbo( Vcb, FirstLboOfFile );
//
//             Dirent->FirstClusterOfFile = (USHORT)FcbOrDcb->FirstClusterOfFile;
//
//             if ( FatIsFat32(Vcb) ) {
//
//                 Dirent->FirstClusterOfFileHi = (USHORT)(FcbOrDcb->FirstClusterOfFile >> 16);
//             }
//
//             //
//             //   Note the size of the allocation we need to tell the cache manager about.
//             //
//
//             NewAllocation = DesiredAllocationSize;
//
//         } else {
//
//             LBO LastAllocatedLbo;
//             VBO DontCare;
//
//             //
//             //  Get the first cluster following the current allocation.  It is possible
//             //  the Mcb is empty (or short, etc.) so we need to be slightly careful
//             //  about making sure we don't lie with the hint.
//             //
//
//             (void)FatLookupLastMcbEntry( FcbOrDcb->Vcb, &FcbOrDcb->Mcb, &DontCare, &LastAllocatedLbo, NULL );
//
//             //
//             //  Try to get some disk space starting from there.
//             //
//
//             NewAllocation = DesiredAllocationSize - FcbOrDcb->Header.AllocationSize.LowPart;
//
//             FsRtlInitializeLargeMcb( &NewMcb, PagedPool );
//             UnwindWeInitializedMcb = TRUE;
//             McbToCleanup = &NewMcb;
//
//             FatAllocateDiskSpace( IrpContext,
//                                   Vcb,
//                                   (LastAllocatedLbo != ~0  ?
//                                    FatGetIndexFromLbo(Vcb,LastAllocatedLbo + 1) :
//                                    0),
//                                   &NewAllocation,
//                                   FALSE,
//                                   &NewMcb );
//
//             UnwindWeAllocatedDiskSpace = TRUE;
//         }
//
//         //
//         //  Now that we increased the allocation of the file, mark it in the
//         //  FcbOrDcb.  Carefully prepare to handle an inability to grow the cache
//         //  structures.
//         //
//
//         FcbOrDcb->Header.AllocationSize.LowPart += NewAllocation;
//
//         //
//         //  Handle the maximal file case, where we may have just wrapped.  Note
//         //  that this must be the precise boundary case wrap, i.e. by one byte,
//         //  so that the new allocation is actually one byte "less" as far as we're
//         //  concerned.  This is important for the extension case.
//         //
//
//         if (FcbOrDcb->Header.AllocationSize.LowPart == 0) {
//
//             NewAllocation -= 1;
//             FcbOrDcb->Header.AllocationSize.LowPart = 0xffffffff;
//         }
//
//         UnwindAllocationSizeSet = TRUE;
//
//         //
//         //  Inform the cache manager to increase the section size
//         //
//
//         if ( ARGUMENT_PRESENT(FileObject) && CcIsFileCached(FileObject) ) {
//
//             CcSetFileSizes( FileObject,
//                             (PCC_FILE_SIZES)&FcbOrDcb->Header.AllocationSize );
//             UnwindCacheManagerInformed = TRUE;
//         }
//
//         //
//         //  In the extension case, we have held off actually gluing the new
//         //  allocation onto the file.  This simplifies exception cleanup since
//         //  if it was already added and the section grow failed, we'd have to
//         //  do extra work to unglue it.  This way, we can assume that if we
//         //  raise the only thing we need to do is deallocate the disk space.
//         //
//         //  Merge the allocation now.
//         //
//
//         if (FcbOrDcb->Header.AllocationSize.LowPart != NewAllocation) {
//
//             //
//             //  Tack the new Mcb onto the end of the FcbOrDcb one.
//             //
//
//             FatMergeAllocation( IrpContext,
//                                 Vcb,
//                                 &FcbOrDcb->Mcb,
//                                 &NewMcb );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatAddFileAllocation );
//
//         //
//         //  Give FlushFileBuffer/Cleanup a clue here, regardless of success/fail..
//         //
//
//         SetFlag(FcbOrDcb->FcbState, FCB_STATE_FLUSH_FAT);
//
//         //
//         //  If we were dogged trying to complete this operation, we need to go
//         //  back various things out.
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             //
//             //  Pull off the allocation size we tried to add to this object if
//             //  we failed to grow cache structures or Mcb structures.
//             //
//
//             if (UnwindAllocationSizeSet) {
//
//                 FcbOrDcb->Header.AllocationSize.LowPart -= NewAllocation;
//             }
//
//             if (UnwindCacheManagerInformed) {
//
//                 CcSetFileSizes( FileObject,
//                                 (PCC_FILE_SIZES)&FcbOrDcb->Header.AllocationSize );
//             }
//
//             //
//             //  In the case of initial allocation, we used the Fcb's Mcb and have
//             //  to clean that up as well as the FAT chain references.
//             //
//
//             if (FcbOrDcb->Header.AllocationSize.LowPart == 0) {
//
//                 if (Dirent != NULL) {
//
//                     FcbOrDcb->FirstClusterOfFile = 0;
//                     Dirent->FirstClusterOfFile = 0;
//
//                     if ( FatIsFat32(Vcb) ) {
//
//                         Dirent->FirstClusterOfFileHi = 0;
//                     }
//                 }
//             }
//
//             //
//             //  ... and drop the dirent Bcb if we got it.  Do it now
//             //  so we can afford to take the exception if we have to.
//             //
//
//             FatUnpinBcb( IrpContext, Bcb );
//
//             _SEH2_TRY {
//
//                 //
//                 //  Note this can re-raise.
//                 //
//
//                 if ( UnwindWeAllocatedDiskSpace ) {
//
//                     FatDeallocateDiskSpace( IrpContext, Vcb, McbToCleanup, FALSE );
//                 }
//
//             } _SEH2_FINALLY {
//
//                 //
//                 //  We always want to clean up the non-initial allocation temporary Mcb,
//                 //  otherwise we have the Fcb's Mcb and we just truncate it away.
//                 //
//
//                 if (UnwindWeInitializedMcb == TRUE) {
//
//                     //
//                     //  Note that we already know a raise is in progress.  No danger
//                     //  of encountering the normal case code below and doing this again.
//                     //
//
//                     FsRtlUninitializeLargeMcb( McbToCleanup );
//
//                 } else {
//
//                     if (McbToCleanup) {
//
//                         FsRtlTruncateLargeMcb( McbToCleanup, 0 );
//                     }
//                 }
//             } _SEH2_END;
//         }
//
//         DebugTrace(-1, Dbg, "FatAddFileAllocation -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     //
//     //  Non-exceptional cleanup we always want to do.  In handling the re-raise possibilities
//     //  during exceptions we had to make sure these two steps always happened there beforehand.
//     //  So now we handle the usual case.
//     //
//
//     FatUnpinBcb( IrpContext, Bcb );
//
//     if (UnwindWeInitializedMcb == TRUE) {
//
//         FsRtlUninitializeLargeMcb( &NewMcb );
//     }
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatTruncateFileAllocation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN ULONG DesiredAllocationSize
//     )
//
// /*++
//
// Routine Description:
//
//     This routine truncates the allocation to the specified file/directory.
//
//     If the file is already smaller than the indicated size then this procedure
//     is effectively a noop.
//
//
// Arguments:
//
//     FcbOrDcb - Supplies the Fcb/Dcb of the file/directory being modified
//                This parameter must not specify the root dcb.
//
//     DesiredAllocationSize - Supplies the maximum size, in bytes, that we want
//                             allocated to the file/directory.  It is rounded
//                             up to the nearest cluster.
//
// Return Value:
//
//     VOID - TRUE if the operation completed and FALSE if it had to
//         block but could not.
//
// --*/
//
// {
//     PVCB Vcb;
//     PBCB Bcb = NULL;
//     LARGE_MCB RemainingMcb = {0};
//     ULONG BytesPerCluster;
//     PDIRENT Dirent = NULL;
//     BOOLEAN UpdatedDirent = FALSE;
//
//     ULONG UnwindInitialAllocationSize;
//     ULONG UnwindInitialFirstClusterOfFile;
//     BOOLEAN UnwindWeAllocatedMcb = FALSE;
//
//     PAGED_CODE();
//
//     Vcb = FcbOrDcb->Vcb;
//
//     DebugTrace(+1, Dbg, "FatTruncateFileAllocation\n", 0);
//     DebugTrace( 0, Dbg, "  FcbOrDcb  =             %p\n", FcbOrDcb);
//     DebugTrace( 0, Dbg, "  DesiredAllocationSize = %8lx\n", DesiredAllocationSize);
//
//     //
//     //  If the Fcb isn't in good condition, we have no business whacking around on
//     //  the disk after "its" clusters.
//     //
//     //  Inspired by a Prefix complaint.
//     //
//
//     NT_ASSERT( FcbOrDcb->FcbCondition == FcbGood );
//
//     //
//     //  If we haven't yet set the correct AllocationSize, do so.
//     //
//
//     if (FcbOrDcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT)  {
//
//         FatLookupFileAllocationSize( IrpContext, FcbOrDcb );
//     }
//
//     //
//     //  Round up the Desired Allocation Size to the next cluster size
//     //
//
//     BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     //
//     //  Note if the desired allocation is zero, to distinguish this from
//     //  the wrap case below.
//     //
//
//     if (DesiredAllocationSize != 0) {
//
//         DesiredAllocationSize = (DesiredAllocationSize + (BytesPerCluster - 1)) &
//                                 ~(BytesPerCluster - 1);
//         //
//         //  Check for the benign case that the file is already smaller than
//         //  the desired truncation.  Note that if it wraps, then a) it was
//         //  specifying an offset in the maximally allocatable cluster and
//         //  b) we're not asking to extend the file, either.  So stop.
//         //
//
//         if (DesiredAllocationSize == 0 ||
//             DesiredAllocationSize >= FcbOrDcb->Header.AllocationSize.LowPart) {
//
//             DebugTrace(0, Dbg, "Desired size within current allocation.\n", 0);
//
//             DebugTrace(-1, Dbg, "FatTruncateFileAllocation -> (VOID)\n", 0);
//             return;
//         }
//
//     }
//
//     //
//     //  This is a no-op if the allocation size is already what we want.
//     //
//
//     if (DesiredAllocationSize == FcbOrDcb->Header.AllocationSize.LowPart) {
//
//         DebugTrace(0, Dbg, "Desired size equals current allocation.\n", 0);
//         DebugTrace(-1, Dbg, "FatTruncateFileAllocation -> (VOID)\n", 0);
//         return;
//     }
//
//     UnwindInitialAllocationSize = FcbOrDcb->Header.AllocationSize.LowPart;
//     UnwindInitialFirstClusterOfFile = FcbOrDcb->FirstClusterOfFile;
//
//     //
//     //  Update the FcbOrDcb allocation size.  If it is now zero, we have the
//     //  additional task of modifying the FcbOrDcb and Dirent copies of
//     //  FirstClusterInFile.
//     //
//     //  Note that we must pin the dirent before actually deallocating the
//     //  disk space since, in unwind, it would not be possible to reallocate
//     //  deallocated disk space as someone else may have reallocated it and
//     //  may cause an exception when you try to get some more disk space.
//     //  Thus FatDeallocateDiskSpace must be the final dangerous operation.
//     //
//
//     _SEH2_TRY {
//
//         FcbOrDcb->Header.AllocationSize.QuadPart = DesiredAllocationSize;
//
//         //
//         //  Special case 0
//         //
//
//         if (DesiredAllocationSize == 0) {
//
//             //
//             //  We have to update the dirent and FcbOrDcb copies of
//             //  FirstClusterOfFile since before it was 0
//             //
//
//             NT_ASSERT( FcbOrDcb->FcbCondition == FcbGood );
//
//             FatGetDirentFromFcbOrDcb( IrpContext, FcbOrDcb, FALSE, &Dirent, &Bcb );
//
//             Dirent->FirstClusterOfFile = 0;
//
//             if (FatIsFat32(Vcb)) {
//
//                 Dirent->FirstClusterOfFileHi = 0;
//             }
//
//             FcbOrDcb->FirstClusterOfFile = 0;
//
//             FatSetDirtyBcb( IrpContext, Bcb, Vcb, TRUE );
//             UpdatedDirent = TRUE;
//
//             FatDeallocateDiskSpace( IrpContext, Vcb, &FcbOrDcb->Mcb, ((FcbOrDcb->FcbState & FCB_STATE_ZERO_ON_DEALLOCATION) != 0));
//
//             FatRemoveMcbEntry( FcbOrDcb->Vcb, &FcbOrDcb->Mcb, 0, 0xFFFFFFFF );
//
//         } else {
//
//             //
//             //  Split the existing allocation into two parts, one we will keep, and
//             //  one we will deallocate.
//             //
//
//             FsRtlInitializeLargeMcb( &RemainingMcb, PagedPool );
//             UnwindWeAllocatedMcb = TRUE;
//
//             FatSplitAllocation( IrpContext,
//                                 Vcb,
//                                 &FcbOrDcb->Mcb,
//                                 DesiredAllocationSize,
//                                 &RemainingMcb );
//
//             FatDeallocateDiskSpace( IrpContext, Vcb, &RemainingMcb, ((FcbOrDcb->FcbState & FCB_STATE_ZERO_ON_DEALLOCATION) != 0) );
//
//             FsRtlUninitializeLargeMcb( &RemainingMcb );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatTruncateFileAllocation );
//
//         //
//         //  Is this really the right backout strategy?  It would be nice if we could
//         //  pretend the truncate worked if we knew that the file had gotten into
//         //  a consistent state.  Leaving dangled clusters is probably quite preferable.
//         //
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             FcbOrDcb->Header.AllocationSize.LowPart = UnwindInitialAllocationSize;
//
//             if ( (DesiredAllocationSize == 0) && (Dirent != NULL)) {
//
//                 if (UpdatedDirent)  {
//
//                     //
//                     //  If the dirent has been updated ok and marked dirty,  then we
//                     //  failed in deallocatediscspace,  and don't know what state
//                     //  the on disc fat chain is in.  So we throw away the mcb,
//                     //  and potentially loose a few clusters until the next
//                     //  chkdsk.  The operation has succeeded,  but the exception
//                     //  will still propogate.  5.1
//                     //
//
//                     FatRemoveMcbEntry( Vcb, &FcbOrDcb->Mcb, 0, 0xFFFFFFFF );
//                     FcbOrDcb->Header.AllocationSize.QuadPart = 0;
//                 }
//                 else if (FcbOrDcb->FirstClusterOfFile == 0) {
//
//                     Dirent->FirstClusterOfFile = (USHORT)UnwindInitialFirstClusterOfFile;
//
//                     if ( FatIsFat32(Vcb) ) {
//
//                         Dirent->FirstClusterOfFileHi =
//                                 (USHORT)(UnwindInitialFirstClusterOfFile >> 16);
//                     }
//
//                     FcbOrDcb->FirstClusterOfFile = UnwindInitialFirstClusterOfFile;
//                 }
//             }
//
//             if ( UnwindWeAllocatedMcb ) {
//
//                 FsRtlUninitializeLargeMcb( &RemainingMcb );
//             }
//
//             //
//             //  Note that in the non zero truncation case,  we will also
//             //  leak clusters.  However, apart from this, the in memory and on disc
//             //  structures will agree.
//         }
//
//         FatUnpinBcb( IrpContext, Bcb );
//
//         //
//         //  Give FlushFileBuffer/Cleanup a clue here,  regardless of success/fail.
//         //
//
//         SetFlag(FcbOrDcb->FcbState, FCB_STATE_FLUSH_FAT);
//
//         DebugTrace(-1, Dbg, "FatTruncateFileAllocation -> (VOID)\n", 0);
//     } _SEH2_END;
//
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatLookupFileAllocationSize (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine retrieves the current file allocatio size for the
//     specified file/directory.
//
// Arguments:
//
//     FcbOrDcb - Supplies the Fcb/Dcb of the file/directory being modified
//
// --*/
//
// {
//     LBO Lbo;
//     ULONG ByteCount;
//     BOOLEAN DontCare;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLookupAllocationSize\n", 0);
//     DebugTrace( 0, Dbg, "  FcbOrDcb  =      %p\n", FcbOrDcb);
//
//     //
//     //  We call FatLookupFileAllocation with Vbo of 0xffffffff - 1.
//     //
//
//     FatLookupFileAllocation( IrpContext,
//                              FcbOrDcb,
//                              MAXULONG - 1,
//                              &Lbo,
//                              &ByteCount,
//                              &DontCare,
//                              &DontCare,
//                              NULL );
//
//     //
//     //  FileSize was set at Fcb creation time from the contents of the directory entry,
//     //  and we are only now looking up the real length of the allocation chain.  If it
//     //  cannot be contained, this is trash.  Probably more where that came from.
//     //
//
//     if (FcbOrDcb->Header.FileSize.LowPart > FcbOrDcb->Header.AllocationSize.LowPart) {
//
//         FatPopUpFileCorrupt( IrpContext, FcbOrDcb );
//         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//     }
//
//     DebugTrace(-1, Dbg, "FatLookupFileAllocationSize -> (VOID)\n", 0);
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatAllocateDiskSpace (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG AbsoluteClusterHint,
//     IN PULONG ByteCount,
//     IN BOOLEAN ExactMatchRequired,
//     OUT PLARGE_MCB Mcb
//     )
//
// /*++
//
// Routine Description:
//
//     This procedure allocates additional disk space and builds an mcb
//     representing the newly allocated space.  If the space cannot be
//     allocated then this procedure raises an appropriate status.
//
//     Searching starts from the hint index in the Vcb unless an alternative
//     non-zero hint is given in AlternateClusterHint.  If we are using the
//     hint field in the Vcb, it is set to the cluster following our allocation
//     when we are done.
//
//     Disk space can only be allocated in cluster units so this procedure
//     will round up any byte count to the next cluster boundary.
//
//     Pictorially what is done is the following (where ! denotes the end of
//     the fat chain (i.e., FAT_CLUSTER_LAST)):
//
//
//         Mcb (empty)
//
//     becomes
//
//         Mcb |--a--|--b--|--c--!
//
//                             ^
//         ByteCount ----------+
//
// Arguments:
//
//     Vcb - Supplies the VCB being modified
//
//     AbsoluteClusterHint  - Supplies an alternate hint index to start the
//                            search from.  If this is zero we use, and update,
//                            the Vcb hint field.
//
//     ByteCount - Supplies the number of bytes that we are requesting, and
//                 receives the number of bytes that we got.
//
//     ExactMatchRequired - Caller should set this to TRUE if only the precise run requested
//                  is acceptable.
//
//     Mcb - Receives the MCB describing the newly allocated disk space.  The
//           caller passes in an initialized Mcb that is filled in by this procedure.
//
//  Return Value:
//
//     TRUE  - Allocated ok
//     FALSE - Failed to allocate exactly as requested (=> ExactMatchRequired was TRUE)
//
// --*/
//
// {
//     UCHAR LogOfBytesPerCluster;
//     ULONG BytesPerCluster;
//     ULONG StartingCluster;
//     ULONG ClusterCount;
//     ULONG WindowRelativeHint;
// #if DBG
//     ULONG PreviousClear = 0;
// #endif
//
//     PFAT_WINDOW Window;
//     BOOLEAN Wait = FALSE;
//     BOOLEAN Result = TRUE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatAllocateDiskSpace\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb        = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  *ByteCount = %8lx\n", *ByteCount);
//     DebugTrace( 0, Dbg, "  Mcb        = %p\n", Mcb);
//     DebugTrace( 0, Dbg, "  Hint       = %8lx\n", AbsoluteClusterHint);
//
//     NT_ASSERT((AbsoluteClusterHint <= Vcb->AllocationSupport.NumberOfClusters + 2) && (1 != AbsoluteClusterHint));
//
//     //
//     //  Make sure byte count is not zero
//     //
//
//     if (*ByteCount == 0) {
//
//         DebugTrace(0, Dbg, "Nothing to allocate.\n", 0);
//
//         DebugTrace(-1, Dbg, "FatAllocateDiskSpace -> (VOID)\n", 0);
//         return;
//     }
//
//     //
//     //  Compute the cluster count based on the byte count, rounding up
//     //  to the next cluster if there is any remainder.  Note that the
//     //  pathalogical case BytesCount == 0 has been eliminated above.
//     //
//
//     LogOfBytesPerCluster = Vcb->AllocationSupport.LogOfBytesPerCluster;
//     BytesPerCluster = 1 << LogOfBytesPerCluster;
//
//     *ByteCount = (*ByteCount + (BytesPerCluster - 1))
//                             & ~(BytesPerCluster - 1);
//
//     //
//     //  If ByteCount is NOW zero, then we were asked for the maximal
//     //  filesize (or at least for bytes in the last allocatable sector).
//     //
//
//     if (*ByteCount == 0) {
//
//         *ByteCount = 0xffffffff;
//         ClusterCount =  1 << (32 - LogOfBytesPerCluster);
//
//     } else {
//
//         ClusterCount = (*ByteCount >> LogOfBytesPerCluster);
//     }
//
//     //
//     //  Analysis tools don't figure out that ClusterCount is not zero because
//     //  of the ByteCount == 0 checks, so give them a hint.
//     //
//     _Analysis_assume_(ClusterCount > 0);
//
//     //
//     //  Make sure there are enough free clusters to start with, and
//     //  take them now so that nobody else takes them from us.
//     //
//
//     ExAcquireResourceSharedLite(&Vcb->ChangeBitMapResource, TRUE);
//     FatLockFreeClusterBitMap( Vcb );
//
//     if (ClusterCount <= Vcb->AllocationSupport.NumberOfFreeClusters) {
//
//         Vcb->AllocationSupport.NumberOfFreeClusters -= ClusterCount;
//
//     } else {
//
//         FatUnlockFreeClusterBitMap( Vcb );
//         ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//
//         DebugTrace(0, Dbg, "Disk Full.  Raise Status.\n", 0);
//         FatRaiseStatus( IrpContext, STATUS_DISK_FULL );
//     }
//
//     //
//     //  Did the caller supply a hint?
//     //
//
//     if ((0 != AbsoluteClusterHint) && (AbsoluteClusterHint < (Vcb->AllocationSupport.NumberOfClusters + 2)))  {
//
//         if (Vcb->NumberOfWindows > 1)  {
//
//             //
//             //  If we're being called upon to allocate clusters outside the
//             //  current window (which happens only via MoveFile), it's a problem.
//             //  We address this by changing the current window to be the one which
//             //  contains the alternate cluster hint.  Note that if the user's
//             //  request would cross a window boundary, he doesn't really get what
//             //  he wanted.
//             //
//
//             if (AbsoluteClusterHint < Vcb->CurrentWindow->FirstCluster ||
//                 AbsoluteClusterHint > Vcb->CurrentWindow->LastCluster) {
//
//                 ULONG BucketNum = FatWindowOfCluster( AbsoluteClusterHint );
//
//                 NT_ASSERT( BucketNum < Vcb->NumberOfWindows);
//
//                 //
//                 //  Drop our shared lock on the ChangeBitMapResource,  and pick it up again
//                 //  exclusive in preparation for making the window swap.
//                 //
//
//                 FatUnlockFreeClusterBitMap(Vcb);
//                 ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//                 ExAcquireResourceExclusiveLite(&Vcb->ChangeBitMapResource, TRUE);
//                 FatLockFreeClusterBitMap(Vcb);
//
//                 Window = &Vcb->Windows[BucketNum];
//
//                 //
//                 //  Again,  test the current window against the one we want - some other
//                 //  thread could have sneaked in behind our backs and kindly set it to the one
//                 //  we need,  when we dropped and reacquired the ChangeBitMapResource above.
//                 //
//
//                 if (Window != Vcb->CurrentWindow)  {
//
//                     _SEH2_TRY {
//
//                         Wait = BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//                         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//                         //
//                         //  Change to the new window (update Vcb->CurrentWindow) and scan it
//                         //  to build up a freespace bitmap etc.
//                         //
//
//                         FatExamineFatEntries( IrpContext, Vcb,
//                                               0,
//                                               0,
//                                               FALSE,
//                                               Window,
//                                               NULL);
//
//                     } _SEH2_FINALLY {
//
//                         if (!Wait) {
//
//                             ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//                         }
//
//                         if (_SEH2_AbnormalTermination()) {
//
//                             //
//                             //  We will have raised as a result of failing to pick up the
//                             //  chunk of the FAT for this window move.  Release our resources
//                             //  and return the cluster count to the volume.
//                             //
//
//                             Vcb->AllocationSupport.NumberOfFreeClusters += ClusterCount;
//
//                             FatUnlockFreeClusterBitMap( Vcb );
//                             ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//                         }
//                     } _SEH2_END;
//                 }
//             }
//
//             //
//             //  Make the hint cluster number relative to the base of the current window...
//             //
//             //  Currentwindow->Firstcluster is baised by +2 already,  so we will lose the
//             //  bias already in AbsoluteClusterHint.  Put it back....
//             //
//
//             WindowRelativeHint = AbsoluteClusterHint - Vcb->CurrentWindow->FirstCluster + 2;
//         }
//         else {
//
//             //
//             //  Only one 'window',  ie fat16/12.  No modification necessary.
//             //
//
//             WindowRelativeHint = AbsoluteClusterHint;
//         }
//     }
//     else {
//
//         //
//         //  Either no hint supplied,  or it was out of range,  so grab one from the Vcb
//         //
//         //  NOTE: Clusterhint in the Vcb is not guaranteed to be set (may be -1)
//         //
//
//         WindowRelativeHint = Vcb->ClusterHint;
//         AbsoluteClusterHint = 0;
//
//         //
//         //  Vcb hint may not have been initialized yet.  Force to valid cluster.
//         //
//
//         if (-1 == WindowRelativeHint)  {
//
//             WindowRelativeHint = 2;
//         }
//     }
//
//     NT_ASSERT((WindowRelativeHint >= 2) && (WindowRelativeHint < Vcb->FreeClusterBitMap.SizeOfBitMap + 2));
//
//     //
//     //  Keep track of the window we're allocating from, so we can clean
//     //  up correctly if the current window changes after we unlock the
//     //  bitmap.
//     //
//
//     Window = Vcb->CurrentWindow;
//
//     //
//     //  Try to find a run of free clusters large enough for us.
//     //
//
//     StartingCluster = FatFindFreeClusterRun( IrpContext,
//                                              Vcb,
//                                              ClusterCount,
//                                              WindowRelativeHint );
//     //
//     //  If the above call was successful, we can just update the fat
//     //  and Mcb and exit.  Otherwise we have to look for smaller free
//     //  runs.
//     //
//     //  This test is a bit funky. Note that the error return from
//     //  RtlFindClearBits is -1, and adding two to that is 1.
//     //
//
//     if ((StartingCluster != 1) &&
//         ((0 == AbsoluteClusterHint) || (StartingCluster == WindowRelativeHint))
//     )  {
//
// #if DBG
//         PreviousClear = RtlNumberOfClearBits( &Vcb->FreeClusterBitMap );
// #endif // DBG
//
//         //
//         //  Take the clusters we found, and unlock the bit map.
//         //
//
//         FatReserveClusters(IrpContext, Vcb, StartingCluster, ClusterCount);
//
//         Window->ClustersFree -= ClusterCount;
//
//         StartingCluster += Window->FirstCluster;
//         StartingCluster -= 2;
//
//         NT_ASSERT( PreviousClear - ClusterCount == Window->ClustersFree );
//
//         FatUnlockFreeClusterBitMap( Vcb );
//
//         //
//         //  Note that this call will never fail since there is always
//         //  room for one entry in an empty Mcb.
//         //
//
//         FatAddMcbEntry( Vcb, Mcb,
//                         0,
//                         FatGetLboFromIndex( Vcb, StartingCluster ),
//                         *ByteCount);
//         _SEH2_TRY {
//
//             //
//             //  Update the fat.
//             //
//
//             FatAllocateClusters(IrpContext, Vcb,
//                                 StartingCluster,
//                                 ClusterCount);
//
//         } _SEH2_FINALLY {
//
//             DebugUnwind( FatAllocateDiskSpace );
//
//             //
//             //  If the allocate clusters failed, remove the run from the Mcb,
//             //  unreserve the clusters, and reset the free cluster count.
//             //
//
//             if (_SEH2_AbnormalTermination()) {
//
//                 FatRemoveMcbEntry( Vcb, Mcb, 0, *ByteCount );
//
//                 FatLockFreeClusterBitMap( Vcb );
//
//                 //  Only clear bits if the bitmap window is the same.
//
//                 if (Window == Vcb->CurrentWindow) {
//
//                     //  Both values (startingcluster and window->firstcluster) are
//                     //  already biased by 2,  so will cancel,  so we need to add in the 2 again.
//
//                     FatUnreserveClusters( IrpContext, Vcb,
//                                           StartingCluster - Window->FirstCluster + 2,
//                                           ClusterCount );
//                 }
//
//                 Window->ClustersFree += ClusterCount;
//                 Vcb->AllocationSupport.NumberOfFreeClusters += ClusterCount;
//
//                 FatUnlockFreeClusterBitMap( Vcb );
//             }
//
//             ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//         } _SEH2_END;
//
//     } else {
//
//         //
//         //  Note that Index is a zero-based window-relative number.  When appropriate
//         //  it'll get converted into a true cluster number and put in Cluster, which
//         //  will be a volume relative true cluster number.
//         //
//
//         ULONG Index = 0;
//         ULONG Cluster = 0;
//         ULONG CurrentVbo = 0;
//         ULONG PriorLastCluster = 0;
//         ULONG BytesFound = 0;
//
//         ULONG ClustersFound = 0;
//         ULONG ClustersRemaining = 0;
//
//         BOOLEAN LockedBitMap = FALSE;
//         BOOLEAN SelectNextContigWindow = FALSE;
//
//         //
//         //  Drop our shared lock on the ChangeBitMapResource,  and pick it up again
//         //  exclusive in preparation for making a window swap.
//         //
//
//         FatUnlockFreeClusterBitMap(Vcb);
//         ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//         ExAcquireResourceExclusiveLite(&Vcb->ChangeBitMapResource, TRUE);
//         FatLockFreeClusterBitMap(Vcb);
//         LockedBitMap = TRUE;
//
//         _SEH2_TRY {
//
//             if ( ExactMatchRequired && (1 == Vcb->NumberOfWindows))  {
//
//                 //
//                 //  Give up right now,  there are no more windows to search!  RtlFindClearBits
//                 //  searchs the whole bitmap,  so we would have found any contiguous run
//                 //  large enough.
//                 //
//
//                 try_leave( Result = FALSE);
//             }
//
//             //
//             //  While the request is still incomplete, look for the largest
//             //  run of free clusters, mark them taken, allocate the run in
//             //  the Mcb and Fat, and if this isn't the first time through
//             //  the loop link it to prior run on the fat.  The Mcb will
//             //  coalesce automatically.
//             //
//
//             ClustersRemaining = ClusterCount;
//             CurrentVbo = 0;
//             PriorLastCluster = 0;
//
//             while (ClustersRemaining != 0) {
//
//                 //
//                 //  If we just entered the loop, the bit map is already locked
//                 //
//
//                 if ( !LockedBitMap ) {
//
//                     FatLockFreeClusterBitMap( Vcb );
//                     LockedBitMap = TRUE;
//                 }
//
//                 //
//                 //  Find the largest run of free clusters.  If the run is
//                 //  bigger than we need, only use what we need.  Note that
//                 //  this will then be the last while() iteration.
//                 //
//
//                 // 12/3/95: need to bias bitmap by 2 bits for the defrag
//                 // hooks and the below macro became impossible to do without in-line
//                 // procedures.
//                 //
//                 // ClustersFound = FatLongestFreeClusterRun( IrpContext, Vcb, &Index );
//
//                 ClustersFound = 0;
//
//                 if (!SelectNextContigWindow)  {
//
//                     if ( 0 != WindowRelativeHint)  {
//
//                         ULONG Desired = Vcb->FreeClusterBitMap.SizeOfBitMap - (WindowRelativeHint - 2);
//
//                         //
//                         //  We will try to allocate contiguously.  Try from the current hint the to
//                         //  end of current window.  Don't try for more than we actually need.
//                         //
//
//                         if (Desired > ClustersRemaining)  {
//
//                             Desired = ClustersRemaining;
//                         }
//
//                         if (RtlAreBitsClear(  &Vcb->FreeClusterBitMap,
//                                               WindowRelativeHint - 2,
//                                               Desired))
//                         {
//                             //
//                             //  Clusters from hint->...windowend are free.  Take them.
//                             //
//
//                             Index = WindowRelativeHint - 2;
//                             ClustersFound = Desired;
//
//                             if (FatIsFat32(Vcb))  {
//
//                                 //
//                                 //  We're now up against the end of the current window,  so indicate that we
//                                 //  want the next window in the sequence next time around.  (If we're not up
//                                 //  against the end of the window,  then we got what we needed and won't be
//                                 //  coming around again anyway).
//                                 //
//
//                                 SelectNextContigWindow = TRUE;
//                                 WindowRelativeHint = 2;
//                             }
//                             else {
//
//                                 //
//                                 //  FAT 12/16 - we've run up against the end of the volume.  Clear the
//                                 //  hint,  since we now have no idea where to look.
//                                 //
//
//                                 WindowRelativeHint = 0;
//                             }
// #if DBG
//                             PreviousClear = RtlNumberOfClearBits( &Vcb->FreeClusterBitMap );
// #endif // DBG
//                         }
//                         else  {
//
//                             if (ExactMatchRequired)  {
//
//                                 //
//                                 //  If our caller required an exact match,  then we're hosed.  Bail out now.
//                                 //
//
//                                 try_leave( Result = FALSE);
//                             }
//
//                             //
//                             //  Hint failed,  drop back to pot luck
//                             //
//
//                             WindowRelativeHint = 0;
//                         }
//                     }
//
//                     if ((0 == WindowRelativeHint) && (0 == ClustersFound)) {
//
//                         if (ClustersRemaining <= Vcb->CurrentWindow->ClustersFree)  {
//
//                             //
//                             //  The remaining allocation could be satisfied entirely from this
//                             //  window.  We will ask only for what we need,  to try and avoid
//                             //  unnecessarily fragmenting large runs of space by always using
//                             //  (part of) the largest run we can find.  This call will return the
//                             //  first run large enough.
//                             //
//
//                             Index = RtlFindClearBits( &Vcb->FreeClusterBitMap,  ClustersRemaining,  0);
//
//                             if (-1 != Index)  {
//
//                                 ClustersFound = ClustersRemaining;
//                             }
//                         }
//
//                         if (0 == ClustersFound)  {
//
//                             //
//                             //  Still nothing,  so just take the largest free run we can find.
//                             //
//
//                             ClustersFound = RtlFindLongestRunClear( &Vcb->FreeClusterBitMap, &Index );
//
//                         }
// #if DBG
//                         PreviousClear = RtlNumberOfClearBits( &Vcb->FreeClusterBitMap );
// #endif // DBG
//                         if (ClustersFound >= ClustersRemaining) {
//
//                             ClustersFound = ClustersRemaining;
//                         }
//                         else {
//
//                             //
//                             //  If we just ran up to the end of a window,  set up a hint that
//                             //  we'd like the next consecutive window after this one. (FAT32 only)
//                             //
//
//                             if ( ((Index + ClustersFound) == Vcb->FreeClusterBitMap.SizeOfBitMap) &&
//                                  FatIsFat32( Vcb)
//                                )  {
//
//                                 SelectNextContigWindow = TRUE;
//                                 WindowRelativeHint = 2;
//                             }
//                         }
//                     }
//                 }
//
//                 if (ClustersFound == 0) {
//
//                     ULONG FaveWindow = 0;
//                     BOOLEAN SelectedWindow;
//
//                     //
//                     //  If we found no free clusters on a single-window FAT,
//                     //  there was a bad problem with the free cluster count.
//                     //
//
//                     if (1 == Vcb->NumberOfWindows) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "we bugcheck here because our internal data structures are seriously corrupted if this happens" )
// #endif
//                         FatBugCheck( 0, 5, 0 );
//                     }
//
//                     //
//                     //  Switch to a new bucket.  Possibly the next one if we're
//                     //  currently on a roll (allocating contiguously)
//                     //
//
//                     SelectedWindow = FALSE;
//
//                     if ( SelectNextContigWindow)  {
//
//                         ULONG NextWindow;
//
//                         NextWindow = (((ULONG)((PUCHAR)Vcb->CurrentWindow - (PUCHAR)Vcb->Windows)) / sizeof( FAT_WINDOW)) + 1;
//
//                         if ((NextWindow < Vcb->NumberOfWindows) &&
//                             ( Vcb->Windows[ NextWindow].ClustersFree > 0)
//                            )  {
//
//                             FaveWindow = NextWindow;
//                             SelectedWindow = TRUE;
//                         }
//                         else  {
//
//                             if (ExactMatchRequired)  {
//
//                                 //
//                                 //  Some dope tried to allocate a run past the end of the volume...
//                                 //
//
//                                 try_leave( Result = FALSE);
//                             }
//
//                             //
//                             //  Give up on the contiguous allocation attempts
//                             //
//
//                             WindowRelativeHint = 0;
//                         }
//
//                         SelectNextContigWindow = FALSE;
//                     }
//
//                     if (!SelectedWindow)  {
//
//                         //
//                         //  Select a new window to begin allocating from
//                         //
//
//                         FaveWindow = FatSelectBestWindow( Vcb);
//                     }
//
//                     //
//                     //  By now we'd better have found a window with some free clusters
//                     //
//
//                     if (0 == Vcb->Windows[ FaveWindow].ClustersFree) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "we bugcheck here because our internal data structures are seriously corrupted if this happens" )
// #endif
//                         FatBugCheck( 0, 5, 1 );
//                     }
//
//                     Wait = BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//                     SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//
//                     FatExamineFatEntries( IrpContext, Vcb,
//                                           0,
//                                           0,
//                                           FALSE,
//                                           &Vcb->Windows[FaveWindow],
//                                           NULL);
//
//                     if (!Wait) {
//
//                         ClearFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT);
//                     }
//
//                     //
//                     //  Now we'll just go around the loop again, having switched windows,
//                     //  and allocate....
//                     //
// #if DBG
//                     PreviousClear = RtlNumberOfClearBits( &Vcb->FreeClusterBitMap );
// #endif //DBG
//                 }       // if (clustersfound == 0)
//                 else  {
//
//                     //
//                     //  Take the clusters we found, convert our index to a cluster number
//                     //  and unlock the bit map.
//                     //
//
//                     Window = Vcb->CurrentWindow;
//
//                     FatReserveClusters( IrpContext, Vcb, (Index + 2), ClustersFound );
//
//                     Cluster = Index + Window->FirstCluster;
//
//                     Window->ClustersFree -= ClustersFound;
//                     NT_ASSERT( PreviousClear - ClustersFound == Window->ClustersFree );
//
//                     FatUnlockFreeClusterBitMap( Vcb );
//                     LockedBitMap = FALSE;
//
//                     //
//                     //  Add the newly alloced run to the Mcb.
//                     //
//
//                     BytesFound = ClustersFound << LogOfBytesPerCluster;
//
//                     FatAddMcbEntry( Vcb, Mcb,
//                                     CurrentVbo,
//                                     FatGetLboFromIndex( Vcb, Cluster ),
//                                     BytesFound );
//
//                     //
//                     //  Connect the last allocated run with this one, and allocate
//                     //  this run on the Fat.
//                     //
//
//                     if (PriorLastCluster != 0) {
//
//                         FatSetFatEntry( IrpContext,
//                                         Vcb,
//                                         PriorLastCluster,
//                                         (FAT_ENTRY)Cluster );
//                     }
//
//                     //
//                     //  Update the fat
//                     //
//
//                     FatAllocateClusters( IrpContext, Vcb, Cluster, ClustersFound );
//
//                     //
//                     //  Prepare for the next iteration.
//                     //
//
//                     CurrentVbo += BytesFound;
//                     ClustersRemaining -= ClustersFound;
//                     PriorLastCluster = Cluster + ClustersFound - 1;
//                 }
//             }  // while (clustersremaining)
//
//         } _SEH2_FINALLY {
//
//             DebugUnwind( FatAllocateDiskSpace );
//
//             ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//
//             //
//             //  Is there any unwinding to do?
//             //
//
//             if ( _SEH2_AbnormalTermination() || (FALSE == Result)) {
//
//                 //
//                 //  Flag to the caller that they're getting nothing
//                 //
//
//                 *ByteCount = 0;
//
//                 //
//                 //  There are three places we could have taken this exception:
//                 //  when switching the window (FatExamineFatEntries), adding
//                 //  a found run to the Mcb (FatAddMcbEntry), or when writing
//                 //  the changes to the FAT (FatSetFatEntry).  In the first case
//                 //  we don't have anything to unwind before deallocation, and
//                 //  can detect this by seeing if we have the ClusterBitmap
//                 //  mutex out.
//
//                 if (!LockedBitMap) {
//
//                     FatLockFreeClusterBitMap( Vcb );
//
//                     //
//                     //  In these cases, we have the possiblity that the FAT
//                     //  window is still in place and we need to clear the bits.
//                     //  If the Mcb entry isn't there (we raised trying to add
//                     //  it), the effect of trying to remove it is a noop.
//                     //
//
//                     if (Window == Vcb->CurrentWindow) {
//
//                         //
//                         //  Cluster reservation works on cluster 2 based window-relative
//                         //  numbers, so we must convert.  The subtraction will lose the
//                         //  cluster 2 base, so bias the result.
//                         //
//
//                         FatUnreserveClusters( IrpContext, Vcb,
//                                               (Cluster - Window->FirstCluster) + 2,
//                                               ClustersFound );
//                     }
//
//                     //
//                     //  Note that FatDeallocateDiskSpace will take care of adjusting
//                     //  to account for the entries in the Mcb.  All we have to account
//                     //  for is the last run that didn't make it.
//                     //
//
//                     Window->ClustersFree += ClustersFound;
//                     Vcb->AllocationSupport.NumberOfFreeClusters += ClustersFound;
//
//                     FatUnlockFreeClusterBitMap( Vcb );
//
//                     FatRemoveMcbEntry( Vcb, Mcb, CurrentVbo, BytesFound );
//
//                 } else {
//
//                     //
//                     //  Just drop the mutex now - we didn't manage to do anything
//                     //  that needs to be backed out.
//                     //
//
//                     FatUnlockFreeClusterBitMap( Vcb );
//                 }
//
//                 _SEH2_TRY {
//
//                     //
//                     //  Now we have tidied up, we are ready to just send the Mcb
//                     //  off to deallocate disk space
//                     //
//
//                     FatDeallocateDiskSpace( IrpContext, Vcb, Mcb, FALSE );
//
//                 } _SEH2_FINALLY {
//
//                     //
//                     //  Now finally (really), remove all the entries from the mcb
//                     //
//
//                     FatRemoveMcbEntry( Vcb, Mcb, 0, 0xFFFFFFFF );
//                 } _SEH2_END;
//             }
//
//             DebugTrace(-1, Dbg, "FatAllocateDiskSpace -> (VOID)\n", 0);
//
//         } _SEH2_END; // finally
//     }
//
//     return;
// }
//
//
//
// //
// // Limit our zeroing writes to 1 MB.
// //
//
// #define MAX_ZERO_MDL_SIZE (1*1024*1024)
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDeallocateDiskSpace (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PLARGE_MCB Mcb,
//     IN BOOLEAN ZeroOnDeallocate
//     )
//
// /*++
//
// Routine Description:
//
//     This procedure deallocates the disk space denoted by an input
//     mcb.  Note that the input MCB does not need to necessarily describe
//     a chain that ends with a FAT_CLUSTER_LAST entry.
//
//     Pictorially what is done is the following
//
//         Fat |--a--|--b--|--c--|
//         Mcb |--a--|--b--|--c--|
//
//     becomes
//
//         Fat |--0--|--0--|--0--|
//         Mcb |--a--|--b--|--c--|
//
// Arguments:
//
//     Vcb - Supplies the VCB being modified
//
//     Mcb - Supplies the MCB describing the disk space to deallocate.  Note
//           that Mcb is unchanged by this procedure.
//
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     LBO Lbo;
//     VBO Vbo;
//
//     ULONG RunsInMcb;
//     ULONG ByteCount;
//     ULONG ClusterCount = 0;
//     ULONG ClusterIndex = 0;
//     ULONG McbIndex = 0;
//
//     UCHAR LogOfBytesPerCluster;
//
//     PFAT_WINDOW Window;
//
//     NTSTATUS ZeroingStatus = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeallocateDiskSpace\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  Mcb = %p\n", Mcb);
//
//     LogOfBytesPerCluster = Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     RunsInMcb = FsRtlNumberOfRunsInLargeMcb( Mcb );
//
//     if ( RunsInMcb == 0 ) {
//
//         DebugTrace(-1, Dbg, "FatDeallocateDiskSpace -> (VOID)\n", 0);
//         return;
//     }
//
//     //
//     //  If we are supposed to zero out the allocation before freeing it, do so.
//     //
//
//     if (ZeroOnDeallocate) {
//
//         _SEH2_TRY {
//
//             PIRP IoIrp;
//             KEVENT IoEvent;
//             IO_STATUS_BLOCK Iosb;
//             PVOID Buffer = NULL;
//             PMDL Mdl;
//             ULONG ByteCountToZero;
//             ULONG MdlSizeMapped;
//
//             //
//             // Issue the writes down for each run in the Mcb
//             //
//
//             KeInitializeEvent( &IoEvent,
//                                NotificationEvent,
//                                FALSE );
//
//             for ( McbIndex = 0; McbIndex < RunsInMcb; McbIndex++ ) {
//
//                 FatGetNextMcbEntry( Vcb, Mcb, McbIndex, &Vbo, &Lbo, &ByteCount );
//
//                 //
//                 //  Assert that Fat files have no holes.
//                 //
//
//                 NT_ASSERT( Lbo != 0 );
//
//                 //
//                 // Setup our MDL for the this run.
//                 //
//
//                 if (ByteCount > MAX_ZERO_MDL_SIZE) {
//                     Mdl = FatBuildZeroMdl( IrpContext, MAX_ZERO_MDL_SIZE);
//                 } else {
//                     Mdl = FatBuildZeroMdl( IrpContext, ByteCount);
//                 }
//
//                 if (!Mdl) {
//                     ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
//                     goto try_exit;
//                 }
//
//                 _SEH2_TRY {
//
//                     //
//                     // Map the MDL.
//                     //
//
//                     Buffer = MmGetSystemAddressForMdlSafe(Mdl, HighPagePriority|MdlMappingNoExecute);
//                     if (!Buffer) {
//                         NT_ASSERT( FALSE );
//                         ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
//                         goto try_exit2;
//                     }
//
//                     //
//                     // We might not have not been able to get an MDL big enough to map the whole
//                     // run. In this case, break up the write.
//                     //
//
//                     MdlSizeMapped = min( ByteCount, Mdl->ByteCount );
//                     ByteCountToZero = ByteCount;
//
//                     //
//                     // Loop until there are no bytes left to write
//                     //
//
//                     while (ByteCountToZero != 0) {
//
//                         //
//                         //  Write zeros to each run.
//                         //
//
//                         KeClearEvent( &IoEvent );
//
//                         IoIrp = IoBuildSynchronousFsdRequest( IRP_MJ_WRITE,
//                                                               Vcb->TargetDeviceObject,
//                                                               Buffer,
//                                                               MdlSizeMapped,
//                                                               (PLARGE_INTEGER)&Lbo,
//                                                               &IoEvent,
//                                                               &Iosb );
//
//                         if (IoIrp == NULL) {
//                             NT_ASSERT( FALSE );
//                             ZeroingStatus = STATUS_INSUFFICIENT_RESOURCES;
//                             goto try_exit2;
//                         }
//
//                         //
//                         //  Set a flag indicating that we want to write through any
//                         //  cache on the controller.  This eliminates the need for
//                         //  an explicit flush-device after the write.
//                         //
//
//                         SetFlag( IoGetNextIrpStackLocation(IoIrp)->Flags, SL_WRITE_THROUGH );
//
//                         ZeroingStatus = IoCallDriver( Vcb->TargetDeviceObject, IoIrp );
//
//                         if (ZeroingStatus == STATUS_PENDING) {
//
//                             (VOID)KeWaitForSingleObject( &IoEvent,
//                                                          Executive,
//                                                          KernelMode,
//                                                          FALSE,
//                                                          (PLARGE_INTEGER)NULL );
//
//                             ZeroingStatus = Iosb.Status;
//                         }
//
//                         if (!NT_SUCCESS( ZeroingStatus )) {
//                             NT_ASSERT( FALSE );
//                             goto try_exit2;
//                         }
//
//                         //
//                         // Increment the starting offset where we will zero.
//                         //
//
//                         Lbo += MdlSizeMapped;
//
//                         //
//                         // Decrement ByteCount
//                         //
//
//                         ByteCountToZero -= MdlSizeMapped;
//
//                         if (ByteCountToZero < MdlSizeMapped) {
//                             MdlSizeMapped = ByteCountToZero;
//                         }
//
//                     }
//
//                 try_exit2:
//
//                     NOTHING;
//
//                 } _SEH2_FINALLY {
//
//                     if (!FlagOn( Mdl->MdlFlags, MDL_SOURCE_IS_NONPAGED_POOL) &&
//                         FlagOn( Mdl->MdlFlags, MDL_MAPPED_TO_SYSTEM_VA )) {
//
//                         MmUnmapLockedPages( Mdl->MappedSystemVa, Mdl );
//                     }
//                     IoFreeMdl( Mdl );
//                 } _SEH2_END;
//
//             }
//
//         try_exit:
//
//             NOTHING;
//
//         } _SEH2_EXCEPT(FatExceptionFilter( NULL, _SEH2_GetExceptionInformation() )) {
//
//             //
//             // If we failed to zero for some reason, still go ahead and deallocate
//             // the clusters. Otherwise we'll leak space from the volume.
//             //
//
//             ZeroingStatus = _SEH2_GetExceptionCode();
//
//         } _SEH2_END;
//
//     }
//
//     NT_ASSERT( NT_SUCCESS(ZeroingStatus) );
//
//     _SEH2_TRY {
//
//         //
//         //  Run though the Mcb, freeing all the runs in the fat.
//         //
//         //  We do this in two steps (first update the fat, then the bitmap
//         //  (which can't fail)) to prevent other people from taking clusters
//         //  that we need to re-allocate in the event of unwind.
//         //
//
//         ExAcquireResourceSharedLite(&Vcb->ChangeBitMapResource, TRUE);
//
//         RunsInMcb = FsRtlNumberOfRunsInLargeMcb( Mcb );
//
//         for ( McbIndex = 0; McbIndex < RunsInMcb; McbIndex++ ) {
//
//             FatGetNextMcbEntry( Vcb, Mcb, McbIndex, &Vbo, &Lbo, &ByteCount );
//
//             //
//             //  Assert that Fat files have no holes.
//             //
//
//             NT_ASSERT( Lbo != 0 );
//
//             //
//             //  Write FAT_CLUSTER_AVAILABLE to each cluster in the run.
//             //
//
//             if (ByteCount == 0xFFFFFFFF_u32) {
//
//                 //
//                 //  Special case the computation of ClusterCout
//                 //  when file is of max size (4GiB - 1).
//                 //
//
//                 ClusterCount = (1 << (32 - LogOfBytesPerCluster));
//
//             } else {
//
//                 ClusterCount = ByteCount >> LogOfBytesPerCluster;
//             }
//
//             ClusterIndex = FatGetIndexFromLbo( Vcb, Lbo );
//
//             FatFreeClusters( IrpContext, Vcb, ClusterIndex, ClusterCount );
//         }
//
//         //
//         //  From now on, nothing can go wrong .... (as in raise)
//         //
//
//         FatLockFreeClusterBitMap( Vcb );
//
//         for ( McbIndex = 0; McbIndex < RunsInMcb; McbIndex++ ) {
//
//             ULONG ClusterEnd;
//             ULONG MyStart, MyLength, count;
// #if DBG
// #ifndef __REACTOS__
//             ULONG PreviousClear = 0;
// #endif
//             ULONG i = 0;
// #endif
//
//             FatGetNextMcbEntry( Vcb, Mcb, McbIndex, &Vbo, &Lbo, &ByteCount );
//
//             //
//             //  Mark the bits clear in the FreeClusterBitMap.
//             //
//
//             if (ByteCount == 0xFFFFFFFF_u32) {
//
//                 //
//                 //  Special case the computation of ClusterCout
//                 //  when file is of max size (2^32 - 1).
//                 //
//
//                 ClusterCount = (1 << (32 - LogOfBytesPerCluster));
//
//             } else {
//
//                 ClusterCount = ByteCount >> LogOfBytesPerCluster;
//             }
//
//             ClusterIndex = FatGetIndexFromLbo( Vcb, Lbo );
//
//             Window = Vcb->CurrentWindow;
//
//             //
//             //  If we've divided the bitmap, elide bitmap manipulation for
//             //  runs that are outside the current bucket.
//             //
//
//             ClusterEnd = ClusterIndex + ClusterCount - 1;
//
//             if (!(ClusterIndex > Window->LastCluster ||
//                   ClusterEnd  < Window->FirstCluster)) {
//
//                 //
//                 //  The run being freed overlaps the current bucket, so we'll
//                 //  have to clear some bits.
//                 //
//
//                 if (ClusterIndex < Window->FirstCluster &&
//                     ClusterEnd > Window->LastCluster) {
//
//                     MyStart = Window->FirstCluster;
//                     MyLength = Window->LastCluster - Window->FirstCluster + 1;
//
//                 } else if (ClusterIndex < Window->FirstCluster) {
//
//                     MyStart = Window->FirstCluster;
//                     MyLength = ClusterEnd - Window->FirstCluster + 1;
//
//                 } else {
//
//                     //
//                     //  The range being freed starts in the bucket, and may possibly
//                     //  extend beyond the bucket.
//                     //
//
//                     MyStart = ClusterIndex;
//
//                     if (ClusterEnd <= Window->LastCluster) {
//
//                         MyLength = ClusterCount;
//
//                     } else {
//
//                         MyLength = Window->LastCluster - ClusterIndex + 1;
//                     }
//                 }
//
//                 if (MyLength == 0) {
//
//                     continue;
//                 }
//
// #if DBG
// #ifndef __REACTOS__
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "this is DBG build only" )
// #endif
//                 PreviousClear = RtlNumberOfClearBits( &Vcb->FreeClusterBitMap );
// #endif
//
//
//                 //
//                 //  Verify that the Bits are all really set.
//                 //
//
//                 NT_ASSERT( MyStart + MyLength - Window->FirstCluster <= Vcb->FreeClusterBitMap.SizeOfBitMap );
//
//                 for (i = 0; i < MyLength; i++) {
//
//                     NT_ASSERT( RtlCheckBit(&Vcb->FreeClusterBitMap,
//                             MyStart - Window->FirstCluster + i) == 1 );
//                 }
// #endif // DBG
//
//                 FatUnreserveClusters( IrpContext, Vcb,
//                                       MyStart - Window->FirstCluster + 2,
//                                       MyLength );
//             }
//
//             //
//             //  Adjust the ClustersFree count for each bitmap window, even the ones
//             //  that are not the current window.
//             //
//
//             if (FatIsFat32(Vcb)) {
//
//                 Window = &Vcb->Windows[FatWindowOfCluster( ClusterIndex )];
//
//             } else {
//
//                 Window = &Vcb->Windows[0];
//             }
//
//             MyStart = ClusterIndex;
//
//             for (MyLength = ClusterCount; MyLength > 0; MyLength -= count) {
//
//                 count = FatMin(Window->LastCluster - MyStart + 1, MyLength);
//                 Window->ClustersFree += count;
//
//                 //
//                 //  If this was not the last window this allocation spanned,
//                 //  advance to the next.
//                 //
//
//                 if (MyLength != count) {
//
//                     Window++;
//                     MyStart = Window->FirstCluster;
//                 }
//             }
//
//             //
//             //  Deallocation is now complete.  Adjust the free cluster count.
//             //
//
//             Vcb->AllocationSupport.NumberOfFreeClusters += ClusterCount;
//         }
//
// #if DBG
//         if (Vcb->CurrentWindow->ClustersFree !=
//                RtlNumberOfClearBits(&Vcb->FreeClusterBitMap)) {
//
//             DbgPrint("%x vs %x\n",  Vcb->CurrentWindow->ClustersFree,
//                 RtlNumberOfClearBits(&Vcb->FreeClusterBitMap));
//
//             DbgPrint("%x for %x\n", ClusterIndex, ClusterCount);
//         }
// #endif
//
//         FatUnlockFreeClusterBitMap( Vcb );
//
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatDeallocateDiskSpace );
//
//         //
//         //  Is there any unwinding to do?
//         //
//
//         ExReleaseResourceLite(&Vcb->ChangeBitMapResource);
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             LBO LocalLbo;
//             VBO LocalVbo;
//
//             ULONG Index;
//             ULONG Clusters;
//             ULONG FatIndex;
//             ULONG PriorLastIndex;
//
//             //
//             //  For each entry we already deallocated, reallocate it,
//             //  chaining together as nessecary.  Note that we continue
//             //  up to and including the last "for" iteration even though
//             //  the SetFatRun could not have been successful.  This
//             //  allows us a convienent way to re-link the final successful
//             //  SetFatRun.
//             //
//             //  It is possible that the reason we got here will prevent us
//             //  from succeeding in this operation.
//             //
//
//             PriorLastIndex = 0;
//
//             for (Index = 0; Index <= McbIndex; Index++) {
//
//                 FatGetNextMcbEntry(Vcb, Mcb, Index, &LocalVbo, &LocalLbo, &ByteCount);
//
//                 if (ByteCount == 0xFFFFFFFF_u32) {
//
//                     //
//                     //  Special case the computation of ClusterCout
//                     //  when file is of max size (2^32 - 1).
//                     //
//
//                     Clusters = (1 << (32 - LogOfBytesPerCluster));
//
//                 } else {
//
//                     Clusters = ByteCount >> LogOfBytesPerCluster;
//                 }
//
//                 FatIndex = FatGetIndexFromLbo( Vcb, LocalLbo );
//
//                 //
//                 //  We must always restore the prior iteration's last
//                 //  entry, pointing it to the first cluster of this run.
//                 //
//
//                 if (PriorLastIndex != 0) {
//
//                     FatSetFatEntry( IrpContext,
//                                     Vcb,
//                                     PriorLastIndex,
//                                     (FAT_ENTRY)FatIndex );
//                 }
//
//                 //
//                 //  If this is not the last entry (the one that failed)
//                 //  then reallocate the disk space on the fat.
//                 //
//
//                 if ( Index < McbIndex ) {
//
//                     FatAllocateClusters(IrpContext, Vcb, FatIndex, Clusters);
//
//                     PriorLastIndex = FatIndex + Clusters - 1;
//                 }
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatDeallocateDiskSpace -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSplitAllocation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN OUT PLARGE_MCB Mcb,
//     IN VBO SplitAtVbo,
//     OUT PLARGE_MCB RemainingMcb
//     )
//
// /*++
//
// Routine Description:
//
//     This procedure takes a single mcb and splits its allocation into
//     two separate allocation units.  The separation must only be done
//     on cluster boundaries, otherwise we bugcheck.
//
//     On the disk this actually works by inserting a FAT_CLUSTER_LAST into
//     the last index of the first part being split out.
//
//     Pictorially what is done is the following (where ! denotes the end of
//     the fat chain (i.e., FAT_CLUSTER_LAST)):
//
//
//         Mcb          |--a--|--b--|--c--|--d--|--e--|--f--|
//
//                                         ^
//         SplitAtVbo ---------------------+
//
//         RemainingMcb (empty)
//
//     becomes
//
//         Mcb          |--a--|--b--|--c--!
//
//
//         RemainingMcb |--d--|--e--|--f--|
//
// Arguments:
//
//     Vcb - Supplies the VCB being modified
//
//     Mcb - Supplies the MCB describing the allocation being split into
//           two parts.  Upon return this Mcb now contains the first chain.
//
//     SplitAtVbo - Supplies the VBO of the first byte for the second chain
//                  that we creating.
//
//     RemainingMcb - Receives the MCB describing the second chain of allocated
//                    disk space.  The caller passes in an initialized Mcb that
//                    is filled in by this procedure STARTING AT VBO 0.
//
// Return Value:
//
//     VOID - TRUE if the operation completed and FALSE if it had to
//                block but could not.
//
// --*/
//
// {
//     VBO SourceVbo;
//     VBO TargetVbo;
//     VBO DontCare;
//
//     LBO Lbo;
//
//     ULONG ByteCount;
//
// #if DBG
//     ULONG BytesPerCluster;
// #endif
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSplitAllocation\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb          = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  Mcb          = %p\n", Mcb);
//     DebugTrace( 0, Dbg, "  SplitAtVbo   = %8lx\n", SplitAtVbo);
//     DebugTrace( 0, Dbg, "  RemainingMcb = %p\n", RemainingMcb);
//
// #if DBG
//     BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
// #endif
//
//     //
//     //  Assert that the split point is cluster alligned
//     //
//
//     NT_ASSERT( (SplitAtVbo & (BytesPerCluster - 1)) == 0 );
//
//     //
//     //  We should never be handed an empty source MCB and asked to split
//     //  at a non zero point.
//     //
//
//     NT_ASSERT( !((0 != SplitAtVbo) && (0 == FsRtlNumberOfRunsInLargeMcb( Mcb))));
//
//     //
//     //  Assert we were given an empty target Mcb.
//     //
//
//     //
//     //  This assert is commented out to avoid hitting in the Ea error
//     //  path.  In that case we will be using the same Mcb's to split the
//     //  allocation that we used to merge them.  The target Mcb will contain
//     //  the runs that the split will attempt to insert.
//     //
//     //
//     //  NT_ASSERT( FsRtlNumberOfRunsInMcb( RemainingMcb ) == 0 );
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Move the runs after SplitAtVbo from the souce to the target
//         //
//
//         SourceVbo = SplitAtVbo;
//         TargetVbo = 0;
//
//         while (FatLookupMcbEntry(Vcb, Mcb, SourceVbo, &Lbo, &ByteCount, NULL)) {
//
//             FatAddMcbEntry( Vcb, RemainingMcb, TargetVbo, Lbo, ByteCount );
//
//             FatRemoveMcbEntry( Vcb, Mcb, SourceVbo, ByteCount );
//
//             TargetVbo += ByteCount;
//             SourceVbo += ByteCount;
//
//             //
//             //  If SourceVbo overflows, we were actually snipping off the end
//             //  of the maximal file ... and are now done.
//             //
//
//             if (SourceVbo == 0) {
//
//                 break;
//             }
//         }
//
//         //
//         //  Mark the last pre-split cluster as a FAT_LAST_CLUSTER
//         //
//
//         if ( SplitAtVbo != 0 ) {
//
//             FatLookupLastMcbEntry( Vcb, Mcb, &DontCare, &Lbo, NULL );
//
//             FatSetFatEntry( IrpContext,
//                             Vcb,
//                             FatGetIndexFromLbo( Vcb, Lbo ),
//                             FAT_CLUSTER_LAST );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSplitAllocation );
//
//         //
//         //  If we got an exception, we must glue back together the Mcbs
//         //
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             TargetVbo = SplitAtVbo;
//             SourceVbo = 0;
//
//             while (FatLookupMcbEntry(Vcb, RemainingMcb, SourceVbo, &Lbo, &ByteCount, NULL)) {
//
//                 FatAddMcbEntry( Vcb, Mcb, TargetVbo, Lbo, ByteCount );
//
//                 FatRemoveMcbEntry( Vcb, RemainingMcb, SourceVbo, ByteCount );
//
//                 TargetVbo += ByteCount;
//                 SourceVbo += ByteCount;
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatSplitAllocation -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatMergeAllocation (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN OUT PLARGE_MCB Mcb,
//     IN PLARGE_MCB SecondMcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes two separate allocations described by two MCBs and
//     joins them together into one allocation.
//
//     Pictorially what is done is the following (where ! denotes the end of
//     the fat chain (i.e., FAT_CLUSTER_LAST)):
//
//
//         Mcb       |--a--|--b--|--c--!
//
//         SecondMcb |--d--|--e--|--f--|
//
//     becomes
//
//         Mcb       |--a--|--b--|--c--|--d--|--e--|--f--|
//
//         SecondMcb |--d--|--e--|--f--|
//
//
// Arguments:
//
//     Vcb - Supplies the VCB being modified
//
//     Mcb - Supplies the MCB of the first allocation that is being modified.
//           Upon return this Mcb will also describe the newly enlarged
//           allocation
//
//     SecondMcb - Supplies the ZERO VBO BASED MCB of the second allocation
//                 that is being appended to the first allocation.  This
//                 procedure leaves SecondMcb unchanged.
//
// Return Value:
//
//     VOID - TRUE if the operation completed and FALSE if it had to
//         block but could not.
//
// --*/
//
// {
//     VBO SpliceVbo = 0;
//     LBO SpliceLbo;
//
//     VBO SourceVbo;
//     VBO TargetVbo = 0;
//
//     LBO Lbo;
//
//     ULONG ByteCount;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatMergeAllocation\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb       = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  Mcb       = %p\n", Mcb);
//     DebugTrace( 0, Dbg, "  SecondMcb = %p\n", SecondMcb);
//
//     _SEH2_TRY {
//
//         //
//         //  Append the runs from SecondMcb to Mcb
//         //
//
//         (void)FatLookupLastMcbEntry( Vcb, Mcb, &SpliceVbo, &SpliceLbo, NULL );
//
//         SourceVbo = 0;
//         TargetVbo = SpliceVbo + 1;
//
//         while (FatLookupMcbEntry(Vcb, SecondMcb, SourceVbo, &Lbo, &ByteCount, NULL)) {
//
//             FatAddMcbEntry( Vcb, Mcb, TargetVbo, Lbo, ByteCount );
//
//             SourceVbo += ByteCount;
//             TargetVbo += ByteCount;
//         }
//
//         //
//         //  Link the last pre-merge cluster to the first cluster of SecondMcb
//         //
//
//         FatLookupMcbEntry( Vcb, SecondMcb, 0, &Lbo, (PULONG)NULL, NULL );
//
//         FatSetFatEntry( IrpContext,
//                         Vcb,
//                         FatGetIndexFromLbo( Vcb, SpliceLbo ),
//                         (FAT_ENTRY)FatGetIndexFromLbo( Vcb, Lbo ) );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatMergeAllocation );
//
//         //
//         //  If we got an exception, we must remove the runs added to Mcb
//         //
//
//         if ( _SEH2_AbnormalTermination() ) {
//
//             ULONG CutLength;
//
//             if ((CutLength = TargetVbo - (SpliceVbo + 1)) != 0) {
//
//                 FatRemoveMcbEntry( Vcb, Mcb, SpliceVbo + 1, CutLength);
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatMergeAllocation -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// //
// //  Internal support routine
// //
//
// CLUSTER_TYPE
// FatInterpretClusterType (
//     IN PVCB Vcb,
//     IN FAT_ENTRY Entry
//     )
//
// /*++
//
// Routine Description:
//
//     This procedure tells the caller how to interpret the input fat table
//     entry.  It will indicate if the fat cluster is available, resereved,
//     bad, the last one, or the another fat index.  This procedure can deal
//     with both 12 and 16 bit fat.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to examine, yields 12/16 bit info
//
//     Entry - Supplies the fat entry to examine
//
// Return Value:
//
//     CLUSTER_TYPE - Is the type of the input Fat entry
//
// --*/
//
// {
//     DebugTrace(+1, Dbg, "InterpretClusterType\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb   = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  Entry = %8lx\n", Entry);
//
//     PAGED_CODE();
//
//     switch(Vcb->AllocationSupport.FatIndexBitSize ) {
//     case 32:
//         Entry &= FAT32_ENTRY_MASK;
//         break;
//
//     case 12:
//         NT_ASSERT( Entry <= 0xfff );
//         if (Entry >= 0x0ff0) {
//             Entry |= 0x0FFFF000;
//         }
//         break;
//
//     default:
//     case 16:
//         NT_ASSERT( Entry <= 0xffff );
//         if (Entry >= 0x0fff0) {
//             Entry |= 0x0FFF0000;
//         }
//         break;
//     }
//
//     if (Entry == FAT_CLUSTER_AVAILABLE) {
//
//         DebugTrace(-1, Dbg, "FatInterpretClusterType -> FatClusterAvailable\n", 0);
//
//         return FatClusterAvailable;
//
//     } else if (Entry < FAT_CLUSTER_RESERVED) {
//
//         DebugTrace(-1, Dbg, "FatInterpretClusterType -> FatClusterNext\n", 0);
//
//         return FatClusterNext;
//
//     } else if (Entry < FAT_CLUSTER_BAD) {
//
//         DebugTrace(-1, Dbg, "FatInterpretClusterType -> FatClusterReserved\n", 0);
//
//         return FatClusterReserved;
//
//     } else if (Entry == FAT_CLUSTER_BAD) {
//
//         DebugTrace(-1, Dbg, "FatInterpretClusterType -> FatClusterBad\n", 0);
//
//         return FatClusterBad;
//
//     } else {
//
//         DebugTrace(-1, Dbg, "FatInterpretClusterType -> FatClusterLast\n", 0);
//
//         return FatClusterLast;
//     }
// }
//
//
// //
// //  Internal support routine
// //
//
// VOID
// FatLookupFatEntry (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG FatIndex,
//     IN OUT PULONG FatEntry,
//     IN OUT PFAT_ENUMERATION_CONTEXT Context
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes an index into the fat and gives back the value
//     in the Fat at this index.  At any given time, for a 16 bit fat, this
//     routine allows only one page per volume of the fat to be pinned in
//     memory.  For a 12 bit bit fat, the entire fat (max 6k) is pinned.  This
//     extra layer of caching makes the vast majority of requests very
//     fast.  The context for this caching stored in a structure in the Vcb.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to examine, yields 12/16 bit info,
//           fat access context, etc.
//
//     FatIndex - Supplies the fat index to examine.
//
//     FatEntry - Receives the fat entry pointed to by FatIndex.  Note that
//                it must point to non-paged pool.
//
//     Context - This structure keeps track of a page of pinned fat between calls.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLookupFatEntry\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb      = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  FatIndex = %4x\n", FatIndex);
//     DebugTrace( 0, Dbg, "  FatEntry = %8lx\n", FatEntry);
//
//     //
//     //  Make sure they gave us a valid fat index.
//     //
//
//     FatVerifyIndexIsValid(IrpContext, Vcb, FatIndex);
//
//     //
//     //  Case on 12 or 16 bit fats.
//     //
//     //  In the 12 bit case (mostly floppies) we always have the whole fat
//     //  (max 6k bytes) pinned during allocation operations.  This is possibly
//     //  a wee bit slower, but saves headaches over fat entries with 8 bits
//     //  on one page, and 4 bits on the next.
//     //
//     //  The 16 bit case always keeps the last used page pinned until all
//     //  operations are done and it is unpinned.
//     //
//
//     //
//     //  DEAL WITH 12 BIT CASE
//     //
//
//     if (Vcb->AllocationSupport.FatIndexBitSize == 12) {
//
//         //
//         //  Check to see if the fat is already pinned, otherwise pin it.
//         //
//
//         if (Context->Bcb == NULL) {
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                FatReservedBytes( &Vcb->Bpb ),
//                                FatBytesPerFat( &Vcb->Bpb ),
//                                &Context->Bcb,
//                                &Context->PinnedPage );
//         }
//
//         //
//         //  Load the return value.
//         //
//
//
//         FatLookup12BitEntry( Context->PinnedPage, FatIndex, FatEntry );
//
//     } else if (Vcb->AllocationSupport.FatIndexBitSize == 32) {
//
//         //
//         //  DEAL WITH 32 BIT CASE
//         //
//
//         ULONG PageEntryOffset;
//         ULONG OffsetIntoVolumeFile;
//
//         //
//         //  Initialize two local variables that help us.
//         //
//         OffsetIntoVolumeFile = FatReservedBytes(&Vcb->Bpb) + FatIndex * sizeof(FAT_ENTRY);
//         PageEntryOffset = (OffsetIntoVolumeFile % PAGE_SIZE) / sizeof(FAT_ENTRY);
//
//         //
//         //  Check to see if we need to read in a new page of fat
//         //
//
//         if ((Context->Bcb == NULL) ||
//             (OffsetIntoVolumeFile / PAGE_SIZE != Context->VboOfPinnedPage / PAGE_SIZE)) {
//
//             //
//             //  The entry wasn't in the pinned page, so must we unpin the current
//             //  page (if any) and read in a new page.
//             //
//
//             FatUnpinBcb( IrpContext, Context->Bcb );
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                OffsetIntoVolumeFile & ~(PAGE_SIZE - 1),
//                                PAGE_SIZE,
//                                &Context->Bcb,
//                                &Context->PinnedPage );
//
//             Context->VboOfPinnedPage = OffsetIntoVolumeFile & ~(PAGE_SIZE - 1);
//         }
//
//         //
//         //  Grab the fat entry from the pinned page, and return
//         //
//
//         *FatEntry = ((PULONG)(Context->PinnedPage))[PageEntryOffset] & FAT32_ENTRY_MASK;
//
//     } else {
//
//         //
//         //  DEAL WITH 16 BIT CASE
//         //
//
//         ULONG PageEntryOffset;
//         ULONG OffsetIntoVolumeFile;
//
//         //
//         //  Initialize two local variables that help us.
//         //
//
//         OffsetIntoVolumeFile = FatReservedBytes(&Vcb->Bpb) + FatIndex * sizeof(USHORT);
//         PageEntryOffset = (OffsetIntoVolumeFile % PAGE_SIZE) / sizeof(USHORT);
//
//         //
//         //  Check to see if we need to read in a new page of fat
//         //
//
//         if ((Context->Bcb == NULL) ||
//             (OffsetIntoVolumeFile / PAGE_SIZE != Context->VboOfPinnedPage / PAGE_SIZE)) {
//
//             //
//             //  The entry wasn't in the pinned page, so must we unpin the current
//             //  page (if any) and read in a new page.
//             //
//
//             FatUnpinBcb( IrpContext, Context->Bcb );
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                OffsetIntoVolumeFile & ~(PAGE_SIZE - 1),
//                                PAGE_SIZE,
//                                &Context->Bcb,
//                                &Context->PinnedPage );
//
//             Context->VboOfPinnedPage = OffsetIntoVolumeFile & ~(PAGE_SIZE - 1);
//         }
//
//         //
//         //  Grab the fat entry from the pinned page, and return
//         //
//
//         *FatEntry = ((PUSHORT)(Context->PinnedPage))[PageEntryOffset];
//     }
//
//     DebugTrace(-1, Dbg, "FatLookupFatEntry -> (VOID)\n", 0);
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSetFatEntry (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG FatIndex,
//     IN FAT_ENTRY FatEntry
//     )
//
// /*++
//
// Routine Description:
//
//     This routine takes an index into the fat and puts a value in the Fat
//     at this index.  The routine special cases 12, 16 and 32 bit fats.  In
//     all cases we go to the cache manager for a piece of the fat.
//
//     We have a special form of this call for setting the DOS-style dirty bit.
//     Unlike the dirty bit in the boot sector, we do not go to special effort
//     to make sure that this hits the disk synchronously - if the system goes
//     down in the window between the dirty bit being set in the boot sector
//     and the FAT index zero dirty bit being lazy written, then life is tough.
//
//     The only possible scenario is that Win9x may see what it thinks is a clean
//     volume that really isn't (hopefully Memphis will pay attention to our dirty
//     bit as well). The dirty bit will get out quickly, and if heavy activity is
//     occurring, then the dirty bit should actually be there virtually all of the
//     time since the act of cleaning the volume is the "rare" occurance.
//
//     There are synchronization concerns that would crop up if we tried to make
//     this synchronous. This thread may already own the Bcb shared for the first
//     sector of the FAT (so we can't get it exclusive for a writethrough). This
//     would require some more serious replumbing to work around than I want to
//     consider at this time.
//
//     We can and do, however, synchronously set the bit clean.
//
//     At this point the reader should understand why the NT dirty bit is where it is.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to examine, yields 12/16/32 bit info, etc.
//
//     FatIndex - Supplies the destination fat index.
//
//     FatEntry - Supplies the source fat entry.
//
// --*/
//
// {
//     LBO Lbo;
//     PBCB Bcb = NULL;
//     ULONG SectorSize;
//     ULONG OffsetIntoVolumeFile;
//     ULONG WasWait = TRUE;
//     BOOLEAN RegularOperation = TRUE;
//     BOOLEAN CleaningOperation = FALSE;
//     BOOLEAN ReleaseMutex = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSetFatEntry\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb      = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  FatIndex = %4x\n", FatIndex);
//     DebugTrace( 0, Dbg, "  FatEntry = %4x\n", FatEntry);
//
//     //
//     //  Make sure they gave us a valid fat index if this isn't the special
//     //  clean-bit modifying call.
//     //
//
//     if (FatIndex == FAT_DIRTY_BIT_INDEX) {
//
//         //
//         //  We are setting the clean bit state.  Of course, we could
//         //  have corruption that would cause us to try to fiddle the
//         //  reserved index - we guard against this by having the
//         //  special entry values use the reserved high 4 bits that
//         //  we know that we'll never try to set.
//         //
//
//         //
//         //  We don't want to repin the FAT pages involved here.  Just
//         //  let the lazy writer hit them when it can.
//         //
//
//         RegularOperation = FALSE;
//
//         switch (FatEntry) {
//             case FAT_CLEAN_VOLUME:
//                 FatEntry = (FAT_ENTRY)FAT_CLEAN_ENTRY;
//                 CleaningOperation = TRUE;
//                 break;
//
//             case FAT_DIRTY_VOLUME:
//                 switch (Vcb->AllocationSupport.FatIndexBitSize) {
//                     case 12:
//                         FatEntry = FAT12_DIRTY_ENTRY;
//                         break;
//
//                     case 32:
//                         FatEntry = FAT32_DIRTY_ENTRY;
//                         break;
//
//                     default:
//                         FatEntry = FAT16_DIRTY_ENTRY;
//                         break;
//                 }
//                 break;
//
//             default:
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                 break;
//         }
//
//         //
//         //  Disable dirtying semantics for the duration of this operation.  Force this
//         //  operation to wait for the duration.
//         //
//
//         WasWait = FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//         SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_DISABLE_DIRTY );
//
//     } else {
//
//         NT_ASSERT( !(FatEntry & ~FAT32_ENTRY_MASK) );
//         FatVerifyIndexIsValid(IrpContext, Vcb, FatIndex);
//     }
//
//     //
//     //  Set Sector Size
//     //
//
//     SectorSize = 1 << Vcb->AllocationSupport.LogOfBytesPerSector;
//
//     //
//     //  Case on 12 or 16 bit fats.
//     //
//     //  In the 12 bit case (mostly floppies) we always have the whole fat
//     //  (max 6k bytes) pinned during allocation operations.  This is possibly
//     //  a wee bit slower, but saves headaches over fat entries with 8 bits
//     //  on one page, and 4 bits on the next.
//     //
//     //  In the 16 bit case we only read the page that we need to set the fat
//     //  entry.
//     //
//
//     //
//     //  DEAL WITH 12 BIT CASE
//     //
//
//     _SEH2_TRY {
//
//         if (Vcb->AllocationSupport.FatIndexBitSize == 12) {
//
//             PVOID PinnedFat;
//
//             //
//             //  Make sure we have a valid entry
//             //
//
//             FatEntry &= 0xfff;
//
//             //
//             //  We read in the entire fat.  Note that using prepare write marks
//             //  the bcb pre-dirty, so we don't have to do it explicitly.
//             //
//
//             OffsetIntoVolumeFile = FatReservedBytes( &Vcb->Bpb ) + FatIndex * 3 / 2;
//
//             FatPrepareWriteVolumeFile( IrpContext,
//                                        Vcb,
//                                        FatReservedBytes( &Vcb->Bpb ),
//                                        FatBytesPerFat( &Vcb->Bpb ),
//                                        &Bcb,
//                                        &PinnedFat,
//                                        RegularOperation,
//                                        FALSE );
//
//             //
//             //  Mark the sector(s) dirty in the DirtyFatMcb.  This call is
//             //  complicated somewhat for the 12 bit case since a single
//             //  entry write can span two sectors (and pages).
//             //
//             //  Get the Lbo for the sector where the entry starts, and add it to
//             //  the dirty fat Mcb.
//             //
//
//             Lbo = OffsetIntoVolumeFile & ~(SectorSize - 1);
//
//             FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize);
//
//             //
//             //  If the entry started on the last byte of the sector, it continues
//             //  to the next sector, so mark the next sector dirty as well.
//             //
//             //  Note that this entry will simply coalese with the last entry,
//             //  so this operation cannot fail.  Also if we get this far, we have
//             //  made it, so no unwinding will be needed.
//             //
//
//             if ( (OffsetIntoVolumeFile & (SectorSize - 1)) == (SectorSize - 1) ) {
//
//                 Lbo += SectorSize;
//
//                 FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize );
//             }
//
//             //
//             //  Store the entry into the fat; we need a little synchonization
//             //  here and can't use a spinlock since the bytes might not be
//             //  resident.
//             //
//
//             FatLockFreeClusterBitMap( Vcb );
//             ReleaseMutex = TRUE;
//
//             FatSet12BitEntry( PinnedFat, FatIndex, FatEntry );
//
//             FatUnlockFreeClusterBitMap( Vcb );
//             ReleaseMutex = FALSE;
//
//         } else if (Vcb->AllocationSupport.FatIndexBitSize == 32) {
//
//             //
//             //  DEAL WITH 32 BIT CASE
//             //
//
//             PULONG PinnedFatEntry32;
//
//             //
//             //  Read in a new page of fat
//             //
//
//             OffsetIntoVolumeFile = FatReservedBytes( &Vcb->Bpb ) +
//                                    FatIndex * sizeof( FAT_ENTRY );
//
//             FatPrepareWriteVolumeFile( IrpContext,
//                                        Vcb,
//                                        OffsetIntoVolumeFile,
//                                        sizeof(FAT_ENTRY),
//                                        &Bcb,
//                                        (PVOID *)&PinnedFatEntry32,
//                                        RegularOperation,
//                                        FALSE );
//             //
//             //  Mark the sector dirty in the DirtyFatMcb
//             //
//
//             Lbo = OffsetIntoVolumeFile & ~(SectorSize - 1);
//
//             FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize);
//
//             //
//             //  Store the FatEntry to the pinned page.
//             //
//             //  Preserve the reserved bits in FAT32 entries in the file heap.
//             //
//
// #ifdef ALPHA
//             FatLockFreeClusterBitMap( Vcb );
//             ReleaseMutex = TRUE;
// #endif // ALPHA
//
//             if (FatIndex != FAT_DIRTY_BIT_INDEX) {
//
//                 *PinnedFatEntry32 = ((*PinnedFatEntry32 & ~FAT32_ENTRY_MASK) | FatEntry);
//
//             } else {
//
//                 *PinnedFatEntry32 = FatEntry;
//             }
//
// #ifdef ALPHA
//             FatUnlockFreeClusterBitMap( Vcb );
//             ReleaseMutex = FALSE;
// #endif // ALPHA
//
//         } else {
//
//             //
//             //  DEAL WITH 16 BIT CASE
//             //
//
//             PUSHORT PinnedFatEntry;
//
//             //
//             //  Read in a new page of fat
//             //
//
//             OffsetIntoVolumeFile = FatReservedBytes( &Vcb->Bpb ) +
//                                    FatIndex * sizeof(USHORT);
//
//             FatPrepareWriteVolumeFile( IrpContext,
//                                        Vcb,
//                                        OffsetIntoVolumeFile,
//                                        sizeof(USHORT),
//                                        &Bcb,
//                                        (PVOID *)&PinnedFatEntry,
//                                        RegularOperation,
//                                        FALSE );
//             //
//             //  Mark the sector dirty in the DirtyFatMcb
//             //
//
//             Lbo = OffsetIntoVolumeFile & ~(SectorSize - 1);
//
//             FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize);
//
//             //
//             //  Store the FatEntry to the pinned page.
//             //
//             //  We need extra synchronization here for broken architectures
//             //  like the ALPHA that don't support atomic 16 bit writes.
//             //
//
// #ifdef ALPHA
//             FatLockFreeClusterBitMap( Vcb );
//             ReleaseMutex = TRUE;
// #endif // ALPHA
//
//             *PinnedFatEntry = (USHORT)FatEntry;
//
// #ifdef ALPHA
//             FatUnlockFreeClusterBitMap( Vcb );
//             ReleaseMutex = FALSE;
// #endif // ALPHA
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetFatEntry );
//
//         //
//         //  Re-enable volume dirtying in case this was a dirty bit operation.
//         //
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_DIRTY );
//
//         //
//         //  Make this operation asynchronous again if needed.
//         //
//
//         if (!WasWait) {
//
//             ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//         }
//
//         //
//         //  If we still somehow have the Mutex, release it.
//         //
//
//         if (ReleaseMutex) {
//
//             NT_ASSERT( _SEH2_AbnormalTermination() );
//
//             FatUnlockFreeClusterBitMap( Vcb );
//         }
//
//         //
//         //  Unpin the Bcb.  For cleaning operations or if the corruption was detected while mounting we make this write-through.
//         //
//
//         if ((CleaningOperation ||
//              FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNT_IN_PROGRESS)) &&
//              Bcb) {
//
//             IO_STATUS_BLOCK IgnoreStatus;
//
//             CcRepinBcb( Bcb );
//             CcUnpinData( Bcb );
//             DbgDoit( IrpContext->PinCount -= 1 );
//             CcUnpinRepinnedBcb( Bcb, TRUE, &IgnoreStatus );
//
//         } else {
//
//             FatUnpinBcb(IrpContext, Bcb);
//         }
//
//         DebugTrace(-1, Dbg, "FatSetFatEntry -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// //
// //  Internal support routine
// //
//
// VOID
// FatSetFatRun (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG StartingFatIndex,
//     IN ULONG ClusterCount,
//     IN BOOLEAN ChainTogether
//     )
//
// /*++
//
// Routine Description:
//
//     This routine sets a continuous run of clusters in the fat.  If ChainTogether
//     is TRUE, then the clusters are linked together as in normal Fat fasion,
//     with the last cluster receiving FAT_CLUSTER_LAST.  If ChainTogether is
//     FALSE, all the entries are set to FAT_CLUSTER_AVAILABLE, effectively
//     freeing all the clusters in the run.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to examine, yields 12/16 bit info, etc.
//
//     StartingFatIndex - Supplies the destination fat index.
//
//     ClusterCount - Supplies the number of contiguous clusters to work on.
//
//     ChainTogether - Tells us whether to fill the entries with links, or
//                     FAT_CLUSTER_AVAILABLE
//
//
// Return Value:
//
//     VOID
//
// --*/
//
// {
// #define MAXCOUNTCLUS 0x10000
// #define COUNTSAVEDBCBS ((MAXCOUNTCLUS * sizeof(FAT_ENTRY) / PAGE_SIZE) + 2)
//     PBCB SavedBcbs[COUNTSAVEDBCBS][2];
//
//     ULONG SectorSize;
//     ULONG Cluster;
//
//     LBO StartSectorLbo;
//     LBO FinalSectorLbo;
//     LBO Lbo;
//
//     PVOID PinnedFat;
//
//     BOOLEAN ReleaseMutex = FALSE;
//
//     ULONG SavedStartingFatIndex = StartingFatIndex;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSetFatRun\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb              = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  StartingFatIndex = %8x\n", StartingFatIndex);
//     DebugTrace( 0, Dbg, "  ClusterCount     = %8lx\n", ClusterCount);
//     DebugTrace( 0, Dbg, "  ChainTogether    = %s\n", ChainTogether ? "TRUE":"FALSE");
//
//     //
//     //  Make sure they gave us a valid fat run.
//     //
//
//     FatVerifyIndexIsValid(IrpContext, Vcb, StartingFatIndex);
//     FatVerifyIndexIsValid(IrpContext, Vcb, StartingFatIndex + ClusterCount - 1);
//
//     //
//     //  Check special case
//     //
//
//     if (ClusterCount == 0) {
//
//         DebugTrace(-1, Dbg, "FatSetFatRun -> (VOID)\n", 0);
//         return;
//     }
//
//     //
//     //  Set Sector Size
//     //
//
//     SectorSize = 1 << Vcb->AllocationSupport.LogOfBytesPerSector;
//
//     //
//     //  Case on 12 or 16 bit fats.
//     //
//     //  In the 12 bit case (mostly floppies) we always have the whole fat
//     //  (max 6k bytes) pinned during allocation operations.  This is possibly
//     //  a wee bit slower, but saves headaches over fat entries with 8 bits
//     //  on one page, and 4 bits on the next.
//     //
//     //  In the 16 bit case we only read one page at a time, as needed.
//     //
//
//     //
//     //  DEAL WITH 12 BIT CASE
//     //
//
//     _SEH2_TRY {
//
//         if (Vcb->AllocationSupport.FatIndexBitSize == 12) {
//
//             //
//             //  We read in the entire fat.  Note that using prepare write marks
//             //  the bcb pre-dirty, so we don't have to do it explicitly.
//             //
//
//             RtlZeroMemory( &SavedBcbs[0][0], 2 * sizeof(PBCB) * 2);
//
//             FatPrepareWriteVolumeFile( IrpContext,
//                                        Vcb,
//                                        FatReservedBytes( &Vcb->Bpb ),
//                                        FatBytesPerFat( &Vcb->Bpb ),
//                                        &SavedBcbs[0][0],
//                                        &PinnedFat,
//                                        TRUE,
//                                        FALSE );
//
//             //
//             //  Mark the affected sectors dirty.  Note that FinalSectorLbo is
//             //  the Lbo of the END of the entry (Thus * 3 + 2).  This makes sure
//             //  we catch the case of a dirty fat entry straddling a sector boundry.
//             //
//             //  Note that if the first AddMcbEntry succeeds, all following ones
//             //  will simply coalese, and thus also succeed.
//             //
//
//             StartSectorLbo = (FatReservedBytes( &Vcb->Bpb ) + StartingFatIndex * 3 / 2)
//                              & ~(SectorSize - 1);
//
//             FinalSectorLbo = (FatReservedBytes( &Vcb->Bpb ) + ((StartingFatIndex +
//                              ClusterCount) * 3 + 2) / 2) & ~(SectorSize - 1);
//
//             for (Lbo = StartSectorLbo; Lbo <= FinalSectorLbo; Lbo += SectorSize) {
//
//                 FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize );
//             }
//
//             //
//             //  Store the entries into the fat; we need a little
//             //  synchonization here and can't use a spinlock since the bytes
//             //  might not be resident.
//             //
//
//             FatLockFreeClusterBitMap( Vcb );
//             ReleaseMutex = TRUE;
//
//             for (Cluster = StartingFatIndex;
//                  Cluster < StartingFatIndex + ClusterCount - 1;
//                  Cluster++) {
//
//                 FatSet12BitEntry( PinnedFat,
//                                   Cluster,
//                                   ChainTogether ? Cluster + 1 : FAT_CLUSTER_AVAILABLE );
//             }
//
//             //
//             //  Save the last entry
//             //
//
//             FatSet12BitEntry( PinnedFat,
//                               Cluster,
//                               ChainTogether ?
//                               FAT_CLUSTER_LAST & 0xfff : FAT_CLUSTER_AVAILABLE );
//
//             FatUnlockFreeClusterBitMap( Vcb );
//             ReleaseMutex = FALSE;
//
//         } else if (Vcb->AllocationSupport.FatIndexBitSize == 32) {
//
//             //
//             //  DEAL WITH 32 BIT CASE
//             //
//
//             for (;;) {
//
//                 VBO StartOffsetInVolume;
//                 VBO FinalOffsetInVolume;
//
//                 ULONG Page;
//                 ULONG FinalCluster;
//                 PULONG FatEntry = NULL;
//                 ULONG ClusterCountThisRun;
//
//                 StartOffsetInVolume = FatReservedBytes(&Vcb->Bpb) +
//                                             StartingFatIndex * sizeof(FAT_ENTRY);
//
//                 if (ClusterCount > MAXCOUNTCLUS) {
//                     ClusterCountThisRun = MAXCOUNTCLUS;
//                 } else {
//                     ClusterCountThisRun = ClusterCount;
//                 }
//
//                 FinalOffsetInVolume = StartOffsetInVolume +
//                                             (ClusterCountThisRun -  1) * sizeof(FAT_ENTRY);
//
//                 {
//                     ULONG NumberOfPages;
//                     ULONG Offset;
//
//                     NumberOfPages = (FinalOffsetInVolume / PAGE_SIZE) -
//                                     (StartOffsetInVolume / PAGE_SIZE) + 1;
//
//                     RtlZeroMemory( &SavedBcbs[0][0], (NumberOfPages + 1) * sizeof(PBCB) * 2 );
//
//                     for ( Page = 0, Offset = StartOffsetInVolume & ~(PAGE_SIZE - 1);
//                           Page < NumberOfPages;
//                           Page++, Offset += PAGE_SIZE ) {
//
//                         FatPrepareWriteVolumeFile( IrpContext,
//                                                    Vcb,
//                                                    Offset,
//                                                    PAGE_SIZE,
//                                                    &SavedBcbs[Page][0],
//                                                    (PVOID *)&SavedBcbs[Page][1],
//                                                    TRUE,
//                                                    FALSE );
//
//                         if (Page == 0) {
//
//                             FatEntry = (PULONG)((PUCHAR)SavedBcbs[0][1] +
//                                                 (StartOffsetInVolume % PAGE_SIZE));
//                         }
//                     }
//                 }
//
//                 //
//                 //  Mark the run dirty
//                 //
//
//                 StartSectorLbo = StartOffsetInVolume & ~(SectorSize - 1);
//                 FinalSectorLbo = FinalOffsetInVolume & ~(SectorSize - 1);
//
//                 for (Lbo = StartSectorLbo; Lbo <= FinalSectorLbo; Lbo += SectorSize) {
//
//                     FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO)Lbo, Lbo, SectorSize );
//                 }
//
//                 //
//                 //  Store the entries
//                 //
//                 //  We need extra synchronization here for broken architectures
//                 //  like the ALPHA that don't support atomic 16 bit writes.
//                 //
//
// #ifdef ALPHA
//                 FatLockFreeClusterBitMap( Vcb );
//                 ReleaseMutex = TRUE;
// #endif // ALPHA
//
//                 FinalCluster = StartingFatIndex + ClusterCountThisRun - 1;
//                 Page = 0;
//
//                 for (Cluster = StartingFatIndex;
//                      Cluster <= FinalCluster;
//                      Cluster++, FatEntry++) {
//
//                     //
//                     //  If we just crossed a page boundry (as opposed to starting
//                     //  on one), update our idea of FatEntry.
//
//                     if ( (((ULONG_PTR)FatEntry & (PAGE_SIZE-1)) == 0) &&
//                          (Cluster != StartingFatIndex) ) {
//
//                         Page += 1;
//                         FatEntry = (PULONG)SavedBcbs[Page][1];
//                     }
//
//                     *FatEntry = ChainTogether ? (FAT_ENTRY)(Cluster + 1) :
//                                                 FAT_CLUSTER_AVAILABLE;
//                 }
//
//                 //
//                 //  Fix up the last entry if we were chaining together
//                 //
//
//                 if ((ClusterCount <= MAXCOUNTCLUS) &&
//                     ChainTogether ) {
//
//                     *(FatEntry-1) = FAT_CLUSTER_LAST;
//                 }
//
// #ifdef ALPHA
//                 FatUnlockFreeClusterBitMap( Vcb );
//                 ReleaseMutex = FALSE;
// #endif // ALPHA
//
//                 {
//                     ULONG i;
//
//                     //
//                     //  Unpin the Bcbs
//                     //
//
//                     for (i = 0; (i < COUNTSAVEDBCBS) && (SavedBcbs[i][0] != NULL); i++) {
//
//                         FatUnpinBcb( IrpContext, SavedBcbs[i][0] );
//                         SavedBcbs[i][0] = NULL;
//                     }
//                 }
//
//                 if (ClusterCount <= MAXCOUNTCLUS) {
//
//                     break;
//
//                 } else {
//
//                     StartingFatIndex += MAXCOUNTCLUS;
//                     ClusterCount -= MAXCOUNTCLUS;
//                 }
//             }
//
//         } else {
//
//             //
//             //  DEAL WITH 16 BIT CASE
//             //
//
//             VBO StartOffsetInVolume;
//             VBO FinalOffsetInVolume;
//
//             ULONG Page;
//             ULONG FinalCluster;
//             PUSHORT FatEntry = NULL;
//
//             StartOffsetInVolume = FatReservedBytes(&Vcb->Bpb) +
//                                         StartingFatIndex * sizeof(USHORT);
//
//             FinalOffsetInVolume = StartOffsetInVolume +
//                                         (ClusterCount - 1) * sizeof(USHORT);
//
//             //
//             //  Read in one page of fat at a time.  We cannot read in the
//             //  all of the fat we need because of cache manager limitations.
//             //
//             //  SavedBcb was initialized to be able to hold the largest
//             //  possible number of pages in a fat plus and extra one to
//             //  accomadate the boot sector, plus one more to make sure there
//             //  is enough room for the RtlZeroMemory below that needs the mark
//             //  the first Bcb after all the ones we will use as an end marker.
//             //
//
//             {
//                 ULONG NumberOfPages;
//                 ULONG Offset;
//
//                 NumberOfPages = (FinalOffsetInVolume / PAGE_SIZE) -
//                                 (StartOffsetInVolume / PAGE_SIZE) + 1;
//
//                 RtlZeroMemory( &SavedBcbs[0][0], (NumberOfPages + 1) * sizeof(PBCB) * 2 );
//
//                 for ( Page = 0, Offset = StartOffsetInVolume & ~(PAGE_SIZE - 1);
//                       Page < NumberOfPages;
//                       Page++, Offset += PAGE_SIZE ) {
//
//                     FatPrepareWriteVolumeFile( IrpContext,
//                                                Vcb,
//                                                Offset,
//                                                PAGE_SIZE,
//                                                &SavedBcbs[Page][0],
//                                                (PVOID *)&SavedBcbs[Page][1],
//                                                TRUE,
//                                                FALSE );
//
//                     if (Page == 0) {
//
//                         FatEntry = (PUSHORT)((PUCHAR)SavedBcbs[0][1] +
//                                             (StartOffsetInVolume % PAGE_SIZE));
//                     }
//                 }
//             }
//
//             //
//             //  Mark the run dirty
//             //
//
//             StartSectorLbo = StartOffsetInVolume & ~(SectorSize - 1);
//             FinalSectorLbo = FinalOffsetInVolume & ~(SectorSize - 1);
//
//             for (Lbo = StartSectorLbo; Lbo <= FinalSectorLbo; Lbo += SectorSize) {
//
//                 FatAddMcbEntry( Vcb, &Vcb->DirtyFatMcb, (VBO) Lbo, Lbo, SectorSize );
//             }
//
//             //
//             //  Store the entries
//             //
//             //  We need extra synchronization here for broken architectures
//             //  like the ALPHA that don't support atomic 16 bit writes.
//             //
//
// #ifdef ALPHA
//             FatLockFreeClusterBitMap( Vcb );
//             ReleaseMutex = TRUE;
// #endif // ALPHA
//
//             FinalCluster = StartingFatIndex + ClusterCount - 1;
//             Page = 0;
//
//             for (Cluster = StartingFatIndex;
//                  Cluster <= FinalCluster;
//                  Cluster++, FatEntry++) {
//
//                 //
//                 //  If we just crossed a page boundry (as opposed to starting
//                 //  on one), update our idea of FatEntry.
//
//                 if ( (((ULONG_PTR)FatEntry & (PAGE_SIZE-1)) == 0) &&
//                      (Cluster != StartingFatIndex) ) {
//
//                     Page += 1;
//                     FatEntry = (PUSHORT)SavedBcbs[Page][1];
//                 }
//
//                 *FatEntry = (USHORT) (ChainTogether ? (FAT_ENTRY)(Cluster + 1) :
//                                                       FAT_CLUSTER_AVAILABLE);
//             }
//
//             //
//             //  Fix up the last entry if we were chaining together
//             //
//
//             if ( ChainTogether ) {
//
// #ifdef _MSC_VER
// #pragma warning( suppress: 4310 )
// #endif
//                 *(FatEntry-1) = (USHORT)FAT_CLUSTER_LAST;
//
//             }
// #ifdef ALPHA
//             FatUnlockFreeClusterBitMap( Vcb );
//             ReleaseMutex = FALSE;
// #endif // ALPHA
//         }
//
//     } _SEH2_FINALLY {
//
//         ULONG i;
//
//         DebugUnwind( FatSetFatRun );
//
//         //
//         //  If we still somehow have the Mutex, release it.
//         //
//
//         if (ReleaseMutex) {
//
//             NT_ASSERT( _SEH2_AbnormalTermination() );
//
//             FatUnlockFreeClusterBitMap( Vcb );
//         }
//
//         //
//         //  Unpin the Bcbs
//         //
//
//         for (i = 0; (i < COUNTSAVEDBCBS) && (SavedBcbs[i][0] != NULL); i++) {
//
//             FatUnpinBcb( IrpContext, SavedBcbs[i][0] );
//         }
//
//         //
//         //  At this point nothing in this finally clause should have raised.
//         //  So, now comes the unsafe (sigh) stuff.
//         //
//
//         if ( _SEH2_AbnormalTermination() &&
//             (Vcb->AllocationSupport.FatIndexBitSize == 32) ) {
//
//             //
//             //  Fat32 unwind
//             //
//             //  This case is more complex because the FAT12 and FAT16 cases
//             //  pin all the needed FAT pages (128K max), after which it
//             //  can't fail, before changing any FAT entries.  In the Fat32
//             //  case, it may not be practical to pin all the needed FAT
//             //  pages, because that could span many megabytes.  So Fat32
//             //  attacks in chunks, and if a failure occurs once the first
//             //  chunk has been updated, we have to back out the updates.
//             //
//             //  The unwind consists of walking back over each FAT entry we
//             //  have changed, setting it back to the previous value.  Note
//             //  that the previous value with either be FAT_CLUSTER_AVAILABLE
//             //  (if ChainTogether==TRUE) or a simple link to the successor
//             //  (if ChainTogether==FALSE).
//             //
//             //  We concede that any one of these calls could fail too; our
//             //  objective is to make this case no more likely than the case
//             //  for a file consisting of multiple disjoint runs.
//             //
//
//             while ( StartingFatIndex > SavedStartingFatIndex ) {
//
//                 StartingFatIndex--;
//
//                 FatSetFatEntry( IrpContext, Vcb, StartingFatIndex,
//                     ChainTogether ?
//                         StartingFatIndex + 1 : FAT_CLUSTER_AVAILABLE );
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatSetFatRun -> (VOID)\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// //
// //  Internal support routine
// //
//
// UCHAR
// FatLogOf (
//     IN ULONG Value
//     )
//
// /*++
//
// Routine Description:
//
//     This routine just computes the base 2 log of an integer.  It is only used
//     on objects that are know to be powers of two.
//
// Arguments:
//
//     Value - The value to take the base 2 log of.
//
// Return Value:
//
//     UCHAR - The base 2 log of Value.
//
// --*/
//
// {
//     UCHAR Log = 0;
//
// #if FASTFATDBG
//     ULONG OrigValue = Value;
// #endif
//
//     PAGED_CODE();
//
//     //
//     //  Knock bits off until we we get a one at position 0
//     //
//
//     while ( (Value & 0xfffffffe) != 0 ) {
//
//         Log++;
//         Value >>= 1;
//     }
//
//     //
//     //  If there was more than one bit set, the file system messed up,
//     //  Bug Check.
//     //
//
//     if (Value != 0x1) {
//
//         DebugTrace(+1, Dbg, "LogOf\n", 0);
//         DebugTrace( 0, Dbg, "  Value = %8lx\n", OrigValue);
//
//         DebugTrace( 0, Dbg, "Received non power of 2.\n", 0);
//
//         DebugTrace(-1, Dbg, "LogOf -> %8lx\n", Log);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "we bugcheck here because our internal data structures are seriously corrupted if this happens" )
// #endif
//         FatBugCheck( Value, Log, 0 );
//     }
//
//     return Log;
// }
//
//
// VOID
// FatExamineFatEntries(
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG StartIndex OPTIONAL,
//     IN ULONG EndIndex OPTIONAL,
//     IN BOOLEAN SetupWindows,
//     IN PFAT_WINDOW SwitchToWindow OPTIONAL,
//     IN PULONG BitMapBuffer OPTIONAL
//     )
// /*++
//
// Routine Description:
//
//     This routine handles scanning a segment of the FAT into in-memory structures.
//
//     There are three fundamental cases, with variations depending on the FAT type:
//
//     1) During volume setup, FatSetupAllocations
//
//         1a) for FAT12/16, read the FAT into our free clusterbitmap
//         1b) for FAT32, perform the initial scan for window free cluster counts
//
//     2) Switching FAT32 windows on the fly during system operation
//
//     3) Reading arbitrary segments of the FAT for the purposes of the GetVolumeBitmap
//         call (only for FAT32)
//
//     There really is too much going on in here. At some point this should be
//     substantially rewritten.
//
// Arguments:
//
//     Vcb - Supplies the volume involved
//
//     StartIndex - Supplies the starting cluster, ignored if SwitchToWindow supplied
//
//     EndIndex - Supplies the ending cluster, ignored if SwitchToWindow supplied
//
//     SetupWindows - Indicates if we are doing the initial FAT32 scan
//
//     SwitchToWindow - Supplies the FAT window we are examining and will switch to
//
//     BitMapBuffer - Supplies a specific bitmap to fill in, if not supplied we fill
//         in the volume free cluster bitmap if !SetupWindows
//
// Return Value:
//
//     None.  Lots of side effects.
//
// --*/
// {
//     ULONG FatIndexBitSize;
//     ULONG Page = 0;
//     ULONG Offset = 0;
//     ULONG FatIndex;
//     FAT_ENTRY FatEntry = FAT_CLUSTER_AVAILABLE;
//     FAT_ENTRY FirstFatEntry = FAT_CLUSTER_AVAILABLE;
//     PUSHORT FatBuffer;
//     PVOID pv;
//     PBCB Bcb = NULL;
//     ULONG EntriesPerWindow;
//
//     ULONG ClustersThisRun;
//     ULONG StartIndexOfThisRun;
//
//     PULONG FreeClusterCount = NULL;
//
//     PFAT_WINDOW CurrentWindow = NULL;
//
//     PVOID NewBitMapBuffer = NULL;
//     PRTL_BITMAP BitMap = NULL;
//     RTL_BITMAP PrivateBitMap;
//
//     ULONG ClusterSize = 0;
//     ULONG PrefetchPages = 0;
//     ULONG FatPages = 0;
//
//     VBO BadClusterVbo = 0;
//     LBO Lbo = 0;
//
//     enum RunType {
//         FreeClusters,
//         AllocatedClusters,
//         UnknownClusters
//     } CurrentRun;
//
//     PAGED_CODE();
//
//
//     //
//     //  Now assert correct usage.
//     //
//
//     FatIndexBitSize = Vcb->AllocationSupport.FatIndexBitSize;
//
//     NT_ASSERT( !(SetupWindows && (SwitchToWindow || BitMapBuffer)));
//     NT_ASSERT( !(SetupWindows && FatIndexBitSize != 32));
//
//     if (Vcb->NumberOfWindows > 1) {
//
//         //
//         //  FAT32: Calculate the number of FAT entries covered by a window.  This is
//         //  equal to the number of bits in the freespace bitmap,  the size of which
//         //  is hardcoded.
//         //
//
//         EntriesPerWindow = MAX_CLUSTER_BITMAP_SIZE;
//
//     } else {
//
//         EntriesPerWindow = Vcb->AllocationSupport.NumberOfClusters;
//     }
//
//     //
//     //  We will also fill in the cumulative count of free clusters for
//     //  the entire volume.  If this is not appropriate, NULL it out
//     //  shortly.
//     //
//
//     FreeClusterCount = &Vcb->AllocationSupport.NumberOfFreeClusters;
//
//     if (SetupWindows) {
//
//         NT_ASSERT(BitMapBuffer == NULL);
//
//         //
//         //  In this case we're just supposed to scan the fat and set up
//         //  the information regarding where the buckets fall and how many
//         //  free clusters are in each.
//         //
//         //  It is fine to monkey with the real windows, we must be able
//         //  to do this to activate the volume.
//         //
//
//         BitMap = NULL;
//
//         CurrentWindow = &Vcb->Windows[0];
//         CurrentWindow->FirstCluster = StartIndex;
//         CurrentWindow->ClustersFree = 0;
//
//         //
//         //  We always wish to calculate total free clusters when
//         //  setting up the FAT windows.
//         //
//
//     } else if (BitMapBuffer == NULL) {
//
//         //
//         //  We will be filling in the free cluster bitmap for the volume.
//         //  Careful, we can raise out of here and be hopelessly hosed if
//         //  we built this up in the main bitmap/window itself.
//         //
//         //  For simplicity's sake, we'll do the swap for everyone. FAT32
//         //  provokes the need since we can't tolerate partial results
//         //  when switching windows.
//         //
//
//         NT_ASSERT( SwitchToWindow );
//
//         CurrentWindow = SwitchToWindow;
//         StartIndex = CurrentWindow->FirstCluster;
//         EndIndex = CurrentWindow->LastCluster;
//
//         BitMap = &PrivateBitMap;
//         NewBitMapBuffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                     (EntriesPerWindow + 7) / 8,
//                                                     TAG_FAT_BITMAP );
//
//         RtlInitializeBitMap( &PrivateBitMap,
//                              NewBitMapBuffer,
//                              EndIndex - StartIndex + 1);
//
//         if ((FatIndexBitSize == 32) &&
//             (Vcb->NumberOfWindows > 1)) {
//
//             //
//             //  We do not wish count total clusters here.
//             //
//
//             FreeClusterCount = NULL;
//
//         }
//
//     } else {
//
//         BitMap = &PrivateBitMap;
//         RtlInitializeBitMap(&PrivateBitMap,
//                             BitMapBuffer,
//                             EndIndex - StartIndex + 1);
//
//         //
//         //  We do not count total clusters here.
//         //
//
//         FreeClusterCount = NULL;
//     }
//
//     //
//     //  Now, our start index better be in the file heap.
//     //
//
//     NT_ASSERT( StartIndex >= 2 );
//
//
//     _SEH2_TRY {
//
//         //
//         //  Pick up the initial chunk of the FAT and first entry.
//         //
//
//         if (FatIndexBitSize == 12) {
//
//             //
//             //  We read in the entire fat in the 12 bit case.
//             //
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                FatReservedBytes( &Vcb->Bpb ),
//                                FatBytesPerFat( &Vcb->Bpb ),
//                                &Bcb,
//                                (PVOID *)&FatBuffer );
//
//             FatLookup12BitEntry(FatBuffer, 0, &FirstFatEntry);
//
//         } else {
//
//             //
//             //  Read in one page of fat at a time.  We cannot read in the
//             //  all of the fat we need because of cache manager limitations.
//             //
//
//             ULONG BytesPerEntry = FatIndexBitSize >> 3;
//
//             FatPages = (FatReservedBytes(&Vcb->Bpb) + FatBytesPerFat(&Vcb->Bpb) + (PAGE_SIZE - 1)) / PAGE_SIZE;
//             Page = (FatReservedBytes(&Vcb->Bpb) + StartIndex * BytesPerEntry) / PAGE_SIZE;
//
//             Offset = Page * PAGE_SIZE;
//
//             //
//             //  Prefetch the FAT entries in memory for optimal performance.
//             //
//
//             PrefetchPages = FatPages - Page;
//
//             if (PrefetchPages > FAT_PREFETCH_PAGE_COUNT) {
//
//                 PrefetchPages = ALIGN_UP_BY(Page, FAT_PREFETCH_PAGE_COUNT) - Page;
//             }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//             FatPrefetchPages( IrpContext,
//                               Vcb->VirtualVolumeFile,
//                               Page,
//                               PrefetchPages );
// #endif
//
//
//             FatReadVolumeFile( IrpContext,
//                                Vcb,
//                                Offset,
//                                PAGE_SIZE,
//                                &Bcb,
//                                &pv);
//
//             if (FatIndexBitSize == 32) {
//
//                 FatBuffer = (PUSHORT)((PUCHAR)pv +
//                             (FatReservedBytes(&Vcb->Bpb) + StartIndex * BytesPerEntry) %
//                                  PAGE_SIZE);
//
//                 FirstFatEntry = *((PULONG)FatBuffer);
//                 FirstFatEntry = FirstFatEntry & FAT32_ENTRY_MASK;
//
//             } else {
//
//                 FatBuffer = (PUSHORT)((PUCHAR)pv +
//                             FatReservedBytes(&Vcb->Bpb) % PAGE_SIZE) + 2;
//
//                 FirstFatEntry = *FatBuffer;
//             }
//
//         }
//
//         ClusterSize = 1 << (Vcb->AllocationSupport.LogOfBytesPerCluster);
//
//         CurrentRun = (FirstFatEntry == FAT_CLUSTER_AVAILABLE) ?
//                      FreeClusters : AllocatedClusters;
//
//         StartIndexOfThisRun = StartIndex;
//
//
//         for (FatIndex = StartIndex; FatIndex <= EndIndex; FatIndex++) {
//
//
//             if (FatIndexBitSize == 12) {
//
//                 FatLookup12BitEntry(FatBuffer, FatIndex, &FatEntry);
//
//             } else {
//
//                 //
//                 //  If we are setting up the FAT32 windows and have stepped into a new
//                 //  bucket, finalize this one and move forward.
//                 //
//
//                 if (SetupWindows &&
//                     FatIndex > StartIndex &&
//                     (FatIndex - 2) % EntriesPerWindow == 0) {
//
//                     CurrentWindow->LastCluster = FatIndex - 1;
//
//                     if (CurrentRun == FreeClusters) {
//
//                         //
//                         //  We must be counting clusters in order to modify the
//                         //  contents of the window.
//                         //
//
//                         NT_ASSERT( FreeClusterCount );
//
//                         ClustersThisRun = FatIndex - StartIndexOfThisRun;
//                         CurrentWindow->ClustersFree += ClustersThisRun;
//
//                         if (FreeClusterCount) {
//                             *FreeClusterCount += ClustersThisRun;
//                         }
//
//                     } else {
//
//                         NT_ASSERT(CurrentRun == AllocatedClusters);
//
//                     }
//
//                     StartIndexOfThisRun = FatIndex;
//                     CurrentRun = UnknownClusters;
//
//                     CurrentWindow++;
//                     CurrentWindow->ClustersFree = 0;
//                     CurrentWindow->FirstCluster = FatIndex;
//                 }
//
//                 //
//                 //  If we just stepped onto a new page, grab a new pointer.
//                 //
//
//                 if (((ULONG_PTR)FatBuffer & (PAGE_SIZE - 1)) == 0) {
//
//                     FatUnpinBcb( IrpContext, Bcb );
//
//                     Page++;
//                     Offset += PAGE_SIZE;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                     //
//                     //  If we have exhausted all the prefetch pages, prefetch the next chunk.
//                     //
//
//                     if (--PrefetchPages == 0) {
//
//                         PrefetchPages = FatPages - Page;
//
//                         if (PrefetchPages > FAT_PREFETCH_PAGE_COUNT) {
//
//                             PrefetchPages = FAT_PREFETCH_PAGE_COUNT;
//                         }
//
//                         FatPrefetchPages( IrpContext,
//                                           Vcb->VirtualVolumeFile,
//                                           Page,
//                                           PrefetchPages );
//                     }
// #endif
//
//
//                     FatReadVolumeFile( IrpContext,
//                                        Vcb,
//                                        Offset,
//                                        PAGE_SIZE,
//                                        &Bcb,
//                                        &pv );
//
//                     FatBuffer = (PUSHORT)pv;
//                 }
//
//                 if (FatIndexBitSize == 32) {
//
// #ifndef __REACTOS__
// #ifdef _MSC_VER
// #pragma warning( suppress: 4213 )
// #endif
//                     FatEntry = *((PULONG)FatBuffer)++;
//                     FatEntry = FatEntry & FAT32_ENTRY_MASK;
// #else
//                     FatEntry = *((PULONG)FatBuffer);
//                     FatBuffer += 2; /* PUSHORT FatBuffer */
//                     FatEntry = FatEntry & FAT32_ENTRY_MASK;
// #endif
//
//                 } else {
//
//                     FatEntry = *FatBuffer;
//                     FatBuffer += 1;
//                 }
//             }
//
//             if (CurrentRun == UnknownClusters) {
//
//                 CurrentRun = (FatEntry == FAT_CLUSTER_AVAILABLE) ?
//                               FreeClusters : AllocatedClusters;
//             }
//
//             //
//             //  Are we switching from a free run to an allocated run?
//             //
//
//             if (CurrentRun == FreeClusters &&
//                 FatEntry != FAT_CLUSTER_AVAILABLE) {
//
//                 ClustersThisRun = FatIndex - StartIndexOfThisRun;
//
//                 if (FreeClusterCount) {
//
//                     *FreeClusterCount += ClustersThisRun;
//                     CurrentWindow->ClustersFree += ClustersThisRun;
//                 }
//
//                 if (BitMap) {
//
//                     RtlClearBits( BitMap,
//                                   StartIndexOfThisRun - StartIndex,
//                                   ClustersThisRun );
//                 }
//
//                 CurrentRun = AllocatedClusters;
//                 StartIndexOfThisRun = FatIndex;
//             }
//
//             //
//             //  Are we switching from an allocated run to a free run?
//             //
//
//             if (CurrentRun == AllocatedClusters &&
//                 FatEntry == FAT_CLUSTER_AVAILABLE) {
//
//                 ClustersThisRun = FatIndex - StartIndexOfThisRun;
//
//                 if (BitMap) {
//
//                     RtlSetBits( BitMap,
//                                 StartIndexOfThisRun - StartIndex,
//                                 ClustersThisRun );
//                 }
//
//                 CurrentRun = FreeClusters;
//                 StartIndexOfThisRun = FatIndex;
//             }
//
//             //
//             //  If the entry is marked bad, add it to the bad block MCB
//             //
//
//             if ((SetupWindows || (Vcb->NumberOfWindows == 1)) &&
//                 (FatInterpretClusterType( Vcb, FatEntry ) == FatClusterBad)) {
//
//                 //
//                 //  This cluster is marked bad.
//                 //  Add it to the BadBlockMcb.
//                 //
//
//                 Lbo = FatGetLboFromIndex( Vcb, FatIndex );
//                 FatAddMcbEntry( Vcb, &Vcb->BadBlockMcb, BadClusterVbo, Lbo, ClusterSize );
//                 BadClusterVbo += ClusterSize;
//             }
//         }
//
//
//
//         //
//         //  If we finished the scan, then we know about all the possible bad clusters.
//         //
//
//         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_BAD_BLOCKS_POPULATED);
//
//         //
//         //  Now we have to record the final run we encountered
//         //
//
//         ClustersThisRun = FatIndex - StartIndexOfThisRun;
//
//         if (CurrentRun == FreeClusters) {
//
//             if (FreeClusterCount) {
//
//                 *FreeClusterCount += ClustersThisRun;
//                 CurrentWindow->ClustersFree += ClustersThisRun;
//             }
//
//             if (BitMap) {
//
//                 RtlClearBits( BitMap,
//                               StartIndexOfThisRun - StartIndex,
//                               ClustersThisRun );
//             }
//
//         } else {
//
//             if (BitMap) {
//
//                 RtlSetBits( BitMap,
//                             StartIndexOfThisRun - StartIndex,
//                             ClustersThisRun );
//             }
//         }
//
//         //
//         //  And finish the last window if we are in setup.
//         //
//
//         if (SetupWindows) {
//
//             CurrentWindow->LastCluster = FatIndex - 1;
//         }
//
//         //
//         //  Now switch the active window if required.  We've succesfully gotten everything
//         //  nailed down.
//         //
//         //  If we were tracking the free cluster count, this means we should update the
//         //  window.  This is the case of FAT12/16 initialization.
//         //
//
//         if (SwitchToWindow) {
//
//             if (Vcb->FreeClusterBitMap.Buffer) {
//
//                 ExFreePool( Vcb->FreeClusterBitMap.Buffer );
//             }
//
//             RtlInitializeBitMap( &Vcb->FreeClusterBitMap,
//                                  NewBitMapBuffer,
//                                  EndIndex - StartIndex + 1 );
//
//             NewBitMapBuffer = NULL;
//
//             Vcb->CurrentWindow = SwitchToWindow;
//             Vcb->ClusterHint = (ULONG)-1;
//
//             if (FreeClusterCount) {
//
//                 NT_ASSERT( !SetupWindows );
//
//                 Vcb->CurrentWindow->ClustersFree = *FreeClusterCount;
//             }
//         }
//
//         //
//         //  Make sure plausible things occured ...
//         //
//
//         if (!SetupWindows && BitMapBuffer == NULL) {
//
//             ASSERT_CURRENT_WINDOW_GOOD( Vcb );
//         }
//
//         NT_ASSERT(Vcb->AllocationSupport.NumberOfFreeClusters <= Vcb->AllocationSupport.NumberOfClusters);
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Unpin the last bcb and drop the temporary bitmap buffer if it exists.
//         //
//
//         FatUnpinBcb( IrpContext, Bcb);
//
//         if (NewBitMapBuffer) {
//
//             ExFreePool( NewBitMapBuffer );
//         }
//     } _SEH2_END;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// fatprocs.h flag macros, inline at the declared widths (were C stubs taking
// ULONG_PTR, which read garbage upper bits from a narrower argument).
#[inline(always)]
unsafe fn FlagOn(Flags: u32, Mask: u32) -> i32 {
    (Flags as u64 & Mask as u64) as i32
}
