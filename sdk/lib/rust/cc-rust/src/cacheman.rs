use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _PFSN_PREFETCHER_GLOBALS, PPRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP,
    PFILE_OBJECT, PSECTION_OBJECT_POINTERS, PWORK_QUEUE_ITEM,
    PWORK_QUEUE_ENTRY, LARGE_INTEGER, PLARGE_INTEGER, INTERNAL_BCB,
    PINTERNAL_BCB, PFSRTL_COMMON_FCB_HEADER, MM_SYSTEMSIZE,
    LAZY_WRITER, WORK_QUEUE_ITEM, WORK_QUEUE_ENTRY,
    _NPAGED_LOOKASIDE_LIST,
    CcInitView, CcWorkerThread, CcScanDpc, CcPostWorkQueue,
    ExInitializeFastMutex,
    ExIsResourceAcquiredExclusiveLite, ExIsResourceAcquiredSharedLite,
    ExSetResourceOwnerPointer, MmQuerySystemSize, KeInitializeDpc,
    KeInitializeTimer,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub static mut CcPfGlobals: _PFSN_PREFETCHER_GLOBALS = unsafe { core::mem::zeroed() };

extern "C" {
    static mut CcIdleWorkerThreadList: LIST_ENTRY;
    static mut CcExpressWorkQueue: LIST_ENTRY;
    static mut CcRegularWorkQueue: LIST_ENTRY;
    static mut CcPostTickWorkQueue: LIST_ENTRY;
    static mut CcNumberWorkerThreads: ULONG;
    static mut CcDirtyPageThreshold: ULONG;
    static mut LazyWriter: LAZY_WRITER;
    static mut CcTwilightLookasideList: _NPAGED_LOOKASIDE_LIST;
    static mut ExCriticalWorkerThreads: ULONG;
    fn CcBugCheck(a: ULONG, b: ULONG, c: ULONG);
    fn ExInitializeWorkItem(Item: PWORK_QUEUE_ITEM, Routine: PVOID, Context: PVOID);
}

extern "system" {
    fn ExInitializeNPagedLookasideList(
        Lookaside: *mut _NPAGED_LOOKASIDE_LIST,
        Allocate: Option<unsafe extern "system" fn(ULONG, SIZE_T, ULONG) -> PVOID>,
        Free: Option<unsafe extern "system" fn(PVOID)>,
        Flags: ULONG,
        Size: SIZE_T,
        Tag: ULONG,
        Depth: USHORT,
    );
}
extern "C" {
    fn ExAllocateFromNPagedLookasideList(list: *mut _NPAGED_LOOKASIDE_LIST) -> PVOID;
}

const DPFLTR_PREFETCHER_ID: ULONG = 63;
const DPFLTR_TRACE_LEVEL: ULONG = 0x2;

const READAHEAD_DISABLED: ULONG = 0x0001;
const WRITEBEHIND_DISABLED: ULONG = 0x0002;
const FO_SEQUENTIAL_ONLY: ULONG = 0x20;
const FSRTL_FLAG_LIMIT_MODIFIED_PAGES: ULONG = 0x4;
// cctypes.h (1 << 16), the bit Flags.ReadAheadActive() reads. The port had 0x1
// (DontUse), so the flag was never seen set and every read posted another read-ahead.
const PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE: ULONG = 1 << 16;

const MmSmallSystem: MM_SYSTEMSIZE = 0;
const MmMediumSystem: MM_SYSTEMSIZE = 1;
const MmLargeSystem: MM_SYSTEMSIZE = 2;

const ReadAhead: u8 = 1;
// KSPIN_LOCK_QUEUE_NUMBER values. Spelled out because the translation shipped bare
// integer literals here and they were the WRONG ones: 17 (LockQueueTimerTableLock)
// for the master lock and 6 (LockQueueNonPagedPoolLock) for the work-queue lock,
// so Cc took KiTimerTableLock[0] where the C takes CcMasterSpinLock. Never pass a
// literal to Ke*QueuedSpinLock.
const LockQueueMasterLock: i32 = 0x00000005;

#[no_mangle]
pub static mut CcPfEnablePrefetcher: BOOLEAN = 0;
#[no_mangle]
pub static mut CcCapturedSystemSize: MM_SYSTEMSIZE = 0;

static mut BugCheckFileId: ULONG = 0x4 << 16;

#[inline(always)]
unsafe fn BooleanFlagOn(Flags: ULONG, Bit: ULONG) -> bool {
    (Flags & Bit) != 0
}

#[inline(always)]
unsafe fn SetFlag(Flags: *mut ULONG, Bit: ULONG) {
    *Flags |= Bit;
}

#[inline(always)]
unsafe fn ClearFlag(Flags: *mut ULONG, Bit: ULONG) {
    *Flags &= !Bit;
}

#[inline(always)]
unsafe fn ROUND_UP(value: ULONG, alignment: ULONG) -> ULONG {
    ((value + alignment - 1) / alignment) * alignment
}

/// InterlockedOr — atomic OR on i32
#[inline]
unsafe fn InterlockedOr(target: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.fetch_or(value, Ordering::SeqCst)
}

/// InterlockedAnd — atomic AND on i32
#[inline]
unsafe fn InterlockedAnd(target: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.fetch_and(value, Ordering::SeqCst)
}

#[no_mangle]
pub unsafe extern "system" fn CcPfInitializePrefetcher() {
    crate::dbg_print_ex!(
        DPFLTR_PREFETCHER_ID,
        DPFLTR_TRACE_LEVEL,
        b"CCPF: InitializePrefetecher()\n",
    );

    InitializeListHead(core::ptr::addr_of_mut!(CcPfGlobals.ActiveTraces));
    InitializeListHead(core::ptr::addr_of_mut!(CcPfGlobals.CompletedTraces));
    ExInitializeFastMutex(core::ptr::addr_of_mut!(CcPfGlobals.CompletedTracesLock));
}

#[no_mangle]
pub unsafe extern "C" fn CcInitializeCacheManager() -> BOOLEAN {
    CcInitView();

    InitializeListHead(core::ptr::addr_of_mut!(CcIdleWorkerThreadList));
    InitializeListHead(core::ptr::addr_of_mut!(CcExpressWorkQueue));
    InitializeListHead(core::ptr::addr_of_mut!(CcRegularWorkQueue));
    InitializeListHead(core::ptr::addr_of_mut!(CcPostTickWorkQueue));

    CcCapturedSystemSize = MmQuerySystemSize();
    match CcCapturedSystemSize {
        x if x == MmSmallSystem => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
            CcDirtyPageThreshold = (MmNumberOfPhysicalPages / 8) as ULONG;
        }
        x if x == MmMediumSystem => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
            CcDirtyPageThreshold = (MmNumberOfPhysicalPages / 4) as ULONG;
        }
        x if x == MmLargeSystem => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 2;
            CcDirtyPageThreshold = (MmNumberOfPhysicalPages / 8 + MmNumberOfPhysicalPages / 4) as ULONG;
        }
        _ => {
            CcNumberWorkerThreads = 1;
            CcDirtyPageThreshold = (MmNumberOfPhysicalPages / 8) as ULONG;
        }
    }

    for _thread in 0..CcNumberWorkerThreads {
        let item = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<WORK_QUEUE_ITEM>() as SIZE_T,
            u32::from_be_bytes(*b"CcWq"),
        ) as PWORK_QUEUE_ITEM;
        if item.is_null() {
            CcBugCheck(0, 0, 0);
        }

        ExInitializeWorkItem(item, CcWorkerThread as PVOID, item as PVOID);
        InsertTailList(
            core::ptr::addr_of_mut!(CcIdleWorkerThreadList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*item).List) as PLIST_ENTRY,
        );
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(LazyWriter) as PVOID,
        core::mem::size_of::<LAZY_WRITER>() as SIZE_T,
    );
    InitializeListHead(core::ptr::addr_of_mut!(LazyWriter.WorkQueue));
    KeInitializeDpc(
        core::ptr::addr_of_mut!(LazyWriter.ScanDpc),
        Some(CcScanDpc),
        core::ptr::null_mut(),
    );
    KeInitializeTimer(core::ptr::addr_of_mut!(LazyWriter.ScanTimer));

    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(CcTwilightLookasideList),
        None,
        None,
        0,
        core::mem::size_of::<WORK_QUEUE_ENTRY>(),
        u32::from_be_bytes(*b"CcWK"),
        0,
    );

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CcShutdownSystem() {
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFlushedValidData(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    BcbListHeld: BOOLEAN,
) -> LARGE_INTEGER {
    let mut i = core::mem::zeroed::<LARGE_INTEGER>();
    i.QuadPart = 0;
    i
}

#[no_mangle]
pub unsafe extern "system" fn CcRemapBcb(Bcb: PVOID) -> PVOID {
    let Bcb = opaque_ptr(Bcb);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CcScheduleReadAhead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
) {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let mut old_irql: KIRQL = 0;
    let mut new_offset = core::mem::zeroed::<LARGE_INTEGER>();

    let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
    let private_cache_map = (*FileObject).PrivateCacheMap as PPRIVATE_CACHE_MAP;

    if shared_cache_map.is_null()
        || private_cache_map.is_null()
        || BooleanFlagOn((*shared_cache_map).Flags, READAHEAD_DISABLED)
    {
        return;
    }

    let length = ROUND_UP(Length, (*private_cache_map).ReadAheadMask + 1);
    new_offset.QuadPart = (*FileOffset).QuadPart + length as i64;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
        &mut old_irql,
    );

    if BooleanFlagOn((*FileObject).Flags, FO_SEQUENTIAL_ONLY) {
        if new_offset.QuadPart
            < (*private_cache_map).ReadAheadOffset[1].QuadPart
        {
            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
                old_irql,
            );
            return;
        }

        (*private_cache_map).ReadAheadOffset[1].QuadPart =
            new_offset.QuadPart;
        (*private_cache_map).ReadAheadLength[1] = length;
    } else {
        if (*private_cache_map).FileOffset2.QuadPart
            >= (*private_cache_map).FileOffset1.QuadPart
            && (*FileOffset).QuadPart
                >= (*private_cache_map).FileOffset2.QuadPart
        {
            (*private_cache_map).ReadAheadOffset[1].QuadPart =
                new_offset.QuadPart;
            (*private_cache_map).ReadAheadLength[1] = length;
        } else {
            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
                old_irql,
            );
            return;
        }
    }

    if (*private_cache_map).__bindgen_anon_1.Flags.ReadAheadActive() == 0 {
        InterlockedOr(
            core::ptr::addr_of_mut!((*private_cache_map).__bindgen_anon_1.UlongFlags) as *mut i32,
            PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE as i32,
        );
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
            old_irql,
        );

        let work_item = ExAllocateFromNPagedLookasideList(
            core::ptr::addr_of_mut!(CcTwilightLookasideList),
        ) as PWORK_QUEUE_ENTRY;
        if !work_item.is_null() {
            ObReferenceObject(FileObject as PVOID);

            (*work_item).Function = ReadAhead;
            (*work_item).Parameters.Read.FileObject = FileObject as *mut _;

            CcPostWorkQueue(work_item, core::ptr::addr_of_mut!(CcExpressWorkQueue));
            return;
        }

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
            &mut old_irql,
        );
        InterlockedAnd(
            core::ptr::addr_of_mut!((*private_cache_map).__bindgen_anon_1.UlongFlags) as *mut i32,
            !(PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE as i32),
        );
    }

    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*private_cache_map).ReadAheadSpinLock) as *mut _,
        old_irql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CcSetAdditionalCacheAttributes(
    FileObject: PFILE_OBJECT,
    DisableReadAhead: BOOLEAN,
    DisableWriteBehind: BOOLEAN,
) {
    let FileObject = opaque_ptr(FileObject);
    let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    if DisableReadAhead != 0 {
        SetFlag(
            core::ptr::addr_of_mut!((*shared_cache_map).Flags),
            READAHEAD_DISABLED,
        );
    } else {
        ClearFlag(
            core::ptr::addr_of_mut!((*shared_cache_map).Flags),
            READAHEAD_DISABLED,
        );
    }

    if DisableWriteBehind != 0 {
        SetFlag(
            core::ptr::addr_of_mut!((*shared_cache_map).Flags),
            WRITEBEHIND_DISABLED,
        );
    } else {
        ClearFlag(
            core::ptr::addr_of_mut!((*shared_cache_map).Flags),
            WRITEBEHIND_DISABLED,
        );
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
}

#[no_mangle]
pub unsafe extern "system" fn CcSetBcbOwnerPointer(Bcb: PVOID, Owner: PVOID) {
    let Bcb = opaque_ptr(Bcb);
    let Owner = opaque_ptr(Owner);
    let i_bcb = (Bcb as usize - core::mem::offset_of!(INTERNAL_BCB, PFCB)) as PINTERNAL_BCB;

    if ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*i_bcb).Lock)) == 0
        && ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!((*i_bcb).Lock)) == 0
    {
        return;
    }

    ExSetResourceOwnerPointer(core::ptr::addr_of_mut!((*i_bcb).Lock), Owner);
}

#[no_mangle]
pub unsafe extern "system" fn CcSetDirtyPageThreshold(
    FileObject: PFILE_OBJECT,
    DirtyPageThreshold: ULONG,
) {
    let FileObject = opaque_ptr(FileObject);
    let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
    if !shared_cache_map.is_null() {
        (*shared_cache_map).DirtyPageThreshold = DirtyPageThreshold;
    }

    let fcb = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    if ((*fcb).Flags as ULONG & FSRTL_FLAG_LIMIT_MODIFIED_PAGES) == 0 {
        (*fcb).Flags |= FSRTL_FLAG_LIMIT_MODIFIED_PAGES as UCHAR;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcSetReadAheadGranularity(
    FileObject: PFILE_OBJECT,
    Granularity: ULONG,
) {
    let FileObject = opaque_ptr(FileObject);
    let private_map = (*FileObject).PrivateCacheMap as PPRIVATE_CACHE_MAP;
    (*private_map).ReadAheadMask = Granularity - 1;
}
