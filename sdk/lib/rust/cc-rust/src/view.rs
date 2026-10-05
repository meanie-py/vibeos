use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _CC_FILE_SIZES, _CACHE_MANAGER_CALLBACKS, _KEVENT, _IO_STATUS_BLOCK,
    _LARGE_INTEGER, _LIST_ENTRY, _NPAGED_LOOKASIDE_LIST, _INTERNAL_BCB,
    _PRIVATE_CACHE_MAP,
    PFILE_OBJECT, PSECTION_OBJECT_POINTERS, PCC_FILE_SIZES, PCACHE_MANAGER_CALLBACKS,
    PKEVENT, PIO_STATUS_BLOCK, PLARGE_INTEGER, PPRIVATE_CACHE_MAP,
    IoGetTopLevelIrp,
    KeWaitForSingleObject, KeInitializeEvent, KeSetEvent, KeEnterCriticalRegion, KeLeaveCriticalRegion,
    MmCreateSection, MmFlushSegment, MmIsDataSectionResident,
    MmMakeDataSectionResident, MmUnmapViewInSystemSpace,
    ObReferenceObjectByPointer,
    KeBugCheck, MMPFNLIST,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn RtlFillMemory(destination: PVOID, length: SIZE_T, fill: UCHAR);
    fn KeAcquireSpinLockAtDpcLevel(SpinLock: *mut KSPIN_LOCK);
    fn KeReleaseSpinLockFromDpcLevel(SpinLock: *mut KSPIN_LOCK);
    fn ExInitializeNPagedLookasideList(
        Lookaside: *mut _NPAGED_LOOKASIDE_LIST,
        Allocate: Option<unsafe extern "system" fn(ULONG, SIZE_T, ULONG) -> PVOID>,
        Free: Option<unsafe extern "system" fn(PVOID)>,
        Flags: ULONG,
        Size: SIZE_T,
        Tag: ULONG,
        Depth: USHORT,
    );
    fn MmMapViewInSystemSpaceEx(
        Section: PVOID,
        MappedBase: *mut PVOID,
        ViewSize: *mut SIZE_T,
        SectionOffset: PLARGE_INTEGER,
        Flags: usize,
    ) -> NTSTATUS;
}
extern "C" {
    fn ExAllocateFromNPagedLookasideList(list: *mut _NPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToNPagedLookasideList(list: *mut _NPAGED_LOOKASIDE_LIST, entry: PVOID);
}

type INT = i32;

#[inline(always)]
unsafe fn KeInitializeSpinLock(lock: *mut KSPIN_LOCK) {
    *lock = 0;
}

#[repr(C)]
struct ROS_VACB {
    BaseAddress: PVOID,
    Dirty: BOOLEAN,
    PageOut: BOOLEAN,
    MappedCount: ULONG,
    CacheMapVacbListEntry: _LIST_ENTRY,
    DirtyVacbListEntry: _LIST_ENTRY,
    VacbLruListEntry: _LIST_ENTRY,
    FileOffset: _LARGE_INTEGER,
    ReferenceCount: ULONG,
    SharedCacheMap: *mut ROS_SHARED_CACHE_MAP,
}

type PROS_VACB = *mut ROS_VACB;

#[repr(C)]
struct ROS_SHARED_CACHE_MAP {
    NodeTypeCode: USHORT,
    NodeByteSize: USHORT,
    OpenCount: ULONG,
    FileSize: _LARGE_INTEGER,
    BcbList: _LIST_ENTRY,
    SectionSize: _LARGE_INTEGER,
    ValidDataLength: _LARGE_INTEGER,
    FileObject: PFILE_OBJECT,
    DirtyPages: ULONG,
    SharedCacheMapLinks: _LIST_ENTRY,
    Flags: ULONG,
    Section: PVOID,
    CreateEvent: PKEVENT,
    Callbacks: PCACHE_MANAGER_CALLBACKS,
    LazyWriteContext: PVOID,
    PrivateList: _LIST_ENTRY,
    DirtyPageThreshold: ULONG,
    BcbSpinLock: KSPIN_LOCK,
    PrivateCacheMap: _PRIVATE_CACHE_MAP,
    // ROS-specific fields below
    CacheMapVacbListHead: _LIST_ENTRY,
    PinAccess: BOOLEAN,
    CacheMapLock: KSPIN_LOCK,
    FlushCacheLock: KGUARDED_MUTEX,
    Trace: BOOLEAN,
}

type PROS_SHARED_CACHE_MAP = *mut ROS_SHARED_CACHE_MAP;

// Compile-time size assertions to match C layout
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<ROS_SHARED_CACHE_MAP>() == 352);
#[cfg(target_pointer_width = "64")]
const _VACB_SIZE: () = assert!(core::mem::size_of::<ROS_VACB>() == 88);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<ROS_SHARED_CACHE_MAP>() == 240);
#[cfg(target_pointer_width = "32")]
const _VACB_SIZE: () = assert!(core::mem::size_of::<ROS_VACB>() == 56);

const VACB_MAPPING_GRANULARITY: ULONG = 262144;
const NODE_TYPE_SHARED_MAP: USHORT = 0x2DD;
const NODE_TYPE_PRIVATE_MAP: USHORT = 0x2EE;
const READAHEAD_DISABLED: ULONG = 0x01;
const WRITEBEHIND_DISABLED: ULONG = 0x02;
const SHARED_CACHE_MAP_IN_CREATION: ULONG = 0x04;
const SHARED_CACHE_MAP_IN_LAZYWRITE: ULONG = 0x08;
const TAG_BCB: ULONG = 0x63426343;
const TAG_SHARED_CACHE_MAP: ULONG = 0x63536343;
const TAG_VACB: ULONG = 0x61566343;
const TAG_PRIVATE_CACHE_MAP: ULONG = 0x63506343;
const FSRTL_CACHE_TOP_LEVEL_IRP: usize = 0x2;
// This constant was already here and UNUSED: all 35 call sites below passed the
// bare literal 17, which is LockQueueTimerTableLock, so Cc's master lock was
// taking KiTimerTableLock[0]. Never pass a literal to Ke*QueuedSpinLock.
const LockQueueMasterLock: i32 = 0x00000005;
const SEC_RESERVE: ULONG = 0x4000000;

#[no_mangle]
pub static mut DirtyVacbListHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut VacbLruListHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut iBcbLookasideList: _NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
static mut SharedCacheMapLookasideList: _NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };
static mut VacbLookasideList: _NPAGED_LOOKASIDE_LIST = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut CcDirtyPageThreshold: ULONG = 0;
#[no_mangle]
pub static mut CcTotalDirtyPages: ULONG = 0;
#[no_mangle]
pub static mut CcDeferredWrites: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CcDeferredWriteSpinLock: KSPIN_LOCK = unsafe { core::mem::zeroed() };
static mut CcCleanSharedCacheMapList: _LIST_ENTRY = unsafe { core::mem::zeroed() };

struct LazyWriterStruct {
    ScanActive: BOOLEAN,
}

static mut LazyWriter: LazyWriterStruct = LazyWriterStruct { ScanActive: 0 };

extern "system" {
    fn FsRtlAcquireFileForCcFlushEx(file_object: PFILE_OBJECT) -> NTSTATUS;
    fn FsRtlReleaseFileForCcFlush(file_object: PFILE_OBJECT);
    static mut MmThrottleTop: ULONG;
    static mut MmThrottleBottom: ULONG;
    static mut MmModifiedPageListHead: MMPFNLIST;
}
extern "C" {
    fn CcScheduleLazyWriteScan(wait: BOOLEAN);
}

unsafe fn IsPointInRange(start: LONGLONG, length: ULONG, point: LONGLONG) -> BOOLEAN {
    if point >= start && point < start + length as LONGLONG {
        1u8
    } else {
        0u8
    }
}

unsafe fn BooleanFlagOn(flags: ULONG, flag: ULONG) -> BOOLEAN {
    if (flags & flag) != 0 { 1u8 } else { 0u8 }
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbIncRefCount_(vacb: PROS_VACB, file: *const i8, line: INT) -> ULONG {
    let refs = InterlockedIncrement(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG) as ULONG;
    if (*(*vacb).SharedCacheMap).Trace != 0 {
        // DbgPrint(
            // b"(%s:%i) VACB %p ++RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as *const i8,
            // file, line, vacb, refs, (*vacb).Dirty as u32, (*vacb).PageOut
        // );
    }
    refs
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbDecRefCount_(vacb: PROS_VACB, file: *const i8, line: INT) -> ULONG {
    let vacb_dirty = (*vacb).Dirty;
    let vacb_trace = (*(*vacb).SharedCacheMap).Trace;
    let vacb_page_out = (*vacb).PageOut;

    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG) as ULONG;
    
    if refs == 0 && vacb_dirty != 0 {
        DbgBreakPoint();
    }
    
    if vacb_trace != 0 {
        // DbgPrint(
            // b"(%s:%i) VACB %p --RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as *const i8,
            // file, line, vacb, refs, vacb_dirty as u32, vacb_page_out
        // );
    }

    if refs == 0 {
        CcRosInternalFreeVacb(vacb);
    }

    refs
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbGetRefCount_(vacb: PROS_VACB, file: *const i8, line: INT) -> ULONG {
    let refs = InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG,
        0,
        0
    ) as ULONG;
    
    if (*(*vacb).SharedCacheMap).Trace != 0 {
        // DbgPrint(
            // b"(%s:%i) VACB %p ==RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as *const i8,
            // file, line, vacb, refs, (*vacb).Dirty as u32, (*vacb).PageOut
        // );
    }

    refs
}

#[cfg(not(debug_assertions))]
unsafe fn CcRosVacbIncRefCount(vacb: PROS_VACB) -> ULONG {
    InterlockedIncrement(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG) as ULONG
}

#[cfg(not(debug_assertions))]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbDecRefCount(vacb: PROS_VACB) -> ULONG {
    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG) as ULONG;
    if refs == 0 {
        CcRosInternalFreeVacb(vacb);
    }
    refs
}

#[cfg(not(debug_assertions))]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbGetRefCount(vacb: PROS_VACB) -> ULONG {
    InterlockedCompareExchange(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut LONG, 0, 0) as ULONG
}

#[cfg(debug_assertions)]
unsafe fn CcRosVacbIncRefCount(vacb: PROS_VACB) -> ULONG {
    CcRosVacbIncRefCount_(vacb, b"inline\0".as_ptr() as *const i8, 0)
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbDecRefCount(vacb: PROS_VACB) -> ULONG {
    CcRosVacbDecRefCount_(vacb, b"inline\0".as_ptr() as *const i8, 0)
}

#[cfg(debug_assertions)]
#[no_mangle]
pub unsafe extern "C" fn CcRosVacbGetRefCount(vacb: PROS_VACB) -> ULONG {
    CcRosVacbGetRefCount_(vacb, b"inline\0".as_ptr() as *const i8, 0)
}

#[no_mangle]
pub unsafe extern "C" fn CcRosTraceCacheMap(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    trace: BOOLEAN
) {
    #[cfg(debug_assertions)]
    {
        if shared_cache_map.is_null() {
            return;
        }

        (*shared_cache_map).Trace = trace;

        if trace != 0 {
            let oldirql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

            let mut current_entry = (*shared_cache_map).CacheMapVacbListHead.Flink;
            while current_entry != core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead) {
                let current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
                current_entry = (*current_entry).Flink;
            }

            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, oldirql);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CcRosFlushVacb(
    vacb: PROS_VACB,
    iosb: PIO_STATUS_BLOCK
) -> NTSTATUS {
    let iosb = opaque_ptr(iosb);
    let mut status: NTSTATUS;
    let mut have_lock = 0u8;
    let shared_cache_map = (*vacb).SharedCacheMap;

    // Upstream-sync merge (ntoskrnl/cc/view.c): the return value says whether WE were the
    // thread that pulled this VACB off the dirty list. If a concurrent flush already did,
    // we must not re-mark it on failure -- that thread will. Re-marking regardless calls
    // CcRosMarkDirtyVacb on a VACB that is still linked and still ref-counted, which both
    // corrupts DirtyVacbListHead and leaks a reference, so the shared cache map is never
    // torn down and the owning file object is never dereferenced. We still call
    // MmFlushSegment either way: the caller expects a status that reflects a real write,
    // and the concurrent flush may have failed.
    let was_marked = CcRosUnmarkDirtyVacb(vacb, 1u8);

    if IoGetTopLevelIrp() as usize != FSRTL_CACHE_TOP_LEVEL_IRP {
        status = FsRtlAcquireFileForCcFlushEx((*(*vacb).SharedCacheMap).FileObject);
        if status < 0 {
            if status < 0 {
                if was_marked != 0 {
                    CcRosMarkDirtyVacb(vacb);
                }
            } else {
                if (*shared_cache_map).ValidDataLength.QuadPart < ((*vacb).FileOffset.QuadPart + VACB_MAPPING_GRANULARITY as i64) {
                    (*shared_cache_map).ValidDataLength.QuadPart = (*vacb).FileOffset.QuadPart + VACB_MAPPING_GRANULARITY as i64;
                }
            }
            return status;
        }
        have_lock = 1u8;
    }

    status = MmFlushSegment(
        (*(*shared_cache_map).FileObject).SectionObjectPointer,
        core::ptr::addr_of_mut!((*vacb).FileOffset),
        VACB_MAPPING_GRANULARITY,
        iosb
    );

    if have_lock != 0 {
        FsRtlReleaseFileForCcFlush((*(*vacb).SharedCacheMap).FileObject);
    }

    if status < 0 {
        if was_marked != 0 {
            CcRosMarkDirtyVacb(vacb);
        }
    } else {
        if (*shared_cache_map).ValidDataLength.QuadPart < ((*vacb).FileOffset.QuadPart + VACB_MAPPING_GRANULARITY as i64) {
            (*shared_cache_map).ValidDataLength.QuadPart = (*vacb).FileOffset.QuadPart + VACB_MAPPING_GRANULARITY as i64;
        }
    }

    status
}

unsafe fn CcRosDeleteFileCache(
    file_object: PFILE_OBJECT,
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    old_irql: *mut KIRQL
) -> NTSTATUS {
    // Upstream-sync merge (ntoskrnl/cc/view.c): move every VACB onto a PRIVATE list while
    // still holding the lock. Leaving them on CacheMapVacbListHead lets a concurrent
    // CcRosLookupVacb -- reached by anyone already holding this SharedCacheMap, e.g.
    // CcFlushCache -- find one in the window between the lock release below and the second
    // loop unlinking it, taking a reference that then races our CcRosVacbDecRefCount. The
    // leaked reference keeps the VACB, and so the cache map and its file object, alive.
    let mut local_vacb_list: LIST_ENTRY = core::mem::zeroed();
    InitializeListHead(core::ptr::addr_of_mut!(local_vacb_list));

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

    while !IsListEmpty(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead)) {
        let vacb = ((*shared_cache_map).CacheMapVacbListHead.Flink as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;

        RemoveEntryList(core::ptr::addr_of_mut!((*vacb).VacbLruListEntry));
        InitializeListHead(core::ptr::addr_of_mut!((*vacb).VacbLruListEntry));

        if (*vacb).Dirty != 0 {
            CcRosUnmarkDirtyVacb(vacb, 0u8);
            (*vacb).Dirty = 1u8;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry));
        InsertTailList(
            core::ptr::addr_of_mut!(local_vacb_list),
            core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry)
        );
    }

    (*(*file_object).SectionObjectPointer).SharedCacheMap = core::ptr::null_mut();
    RemoveEntryList(core::ptr::addr_of_mut!((*shared_cache_map).SharedCacheMapLinks));

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, *old_irql);

    while !IsListEmpty(core::ptr::addr_of_mut!(local_vacb_list)) {
        let vacb = (RemoveHeadList(core::ptr::addr_of_mut!(local_vacb_list)) as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        
        InitializeListHead(core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry));

        if (*vacb).Dirty != 0 {
            let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let status = MmFlushSegment(
                (*file_object).SectionObjectPointer,
                core::ptr::addr_of_mut!((*vacb).FileOffset),
                VACB_MAPPING_GRANULARITY,
                &mut iosb
            );
            (*vacb).Dirty = 0u8;
        }

        let ref_count = CcRosVacbDecRefCount(vacb);
    }

    if !(*shared_cache_map).Section.is_null() {
        ObDereferenceObject((*shared_cache_map).Section);
    }
    ObDereferenceObject((*shared_cache_map).FileObject as PVOID);

    ExFreeToNPagedLookasideList(
        core::ptr::addr_of_mut!(SharedCacheMapLookasideList),
        shared_cache_map as PVOID
    );

    *old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn CcRosFlushDirtyPages(
    target: ULONG,
    count: *mut ULONG,
    wait: BOOLEAN,
    called_from_lazy: BOOLEAN
) -> NTSTATUS {
    let count = opaque_ptr(count);
    let mut status: NTSTATUS;
    let mut old_irql: KIRQL;
    let flush_all = (target == MAXULONG) as BOOLEAN;
    let mut local_target = target;

    *count = 0;

    KeEnterCriticalRegion();
    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    let mut current_entry = DirtyVacbListHead.Flink;
    
    while ((current_entry != core::ptr::addr_of_mut!(DirtyVacbListHead)) && (local_target > 0)) || flush_all != 0 {
        if current_entry == core::ptr::addr_of_mut!(DirtyVacbListHead) {
            if IsListEmpty(core::ptr::addr_of_mut!(DirtyVacbListHead)) {
                break;
            }
            current_entry = DirtyVacbListHead.Flink;
        }

        let current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, DirtyVacbListEntry) as isize)) as PROS_VACB;
        current_entry = (*current_entry).Flink;

        CcRosVacbIncRefCount(current);

        let shared_cache_map = (*current).SharedCacheMap;

        if called_from_lazy != 0 && BooleanFlagOn((*(*shared_cache_map).FileObject).Flags, crate::fs::FO_TEMPORARY_FILE) != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        if called_from_lazy != 0 && BooleanFlagOn((*shared_cache_map).Flags, WRITEBEHIND_DISABLED) != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        if (*shared_cache_map).Flags & SHARED_CACHE_MAP_IN_LAZYWRITE != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        (*shared_cache_map).Flags |= SHARED_CACHE_MAP_IN_LAZYWRITE;
        (*shared_cache_map).OpenCount += 1;

        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

        let locked = ((*(*shared_cache_map).Callbacks).AcquireForLazyWrite.unwrap())(
            (*shared_cache_map).LazyWriteContext,
            wait
        );
        
        if locked == 0 {
            CcRosVacbDecRefCount(current);
            old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
            (*shared_cache_map).Flags &= !SHARED_CACHE_MAP_IN_LAZYWRITE;

            (*shared_cache_map).OpenCount -= 1;
            if (*shared_cache_map).OpenCount == 0 {
                CcRosDeleteFileCache((*shared_cache_map).FileObject, shared_cache_map, core::ptr::addr_of_mut!(old_irql));
            }

            continue;
        }

        let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        status = CcRosFlushVacb(current, &mut iosb);

        ((*(*shared_cache_map).Callbacks).ReleaseFromLazyWrite.unwrap())((*shared_cache_map).LazyWriteContext);

        CcRosVacbDecRefCount(current);
        old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

        (*shared_cache_map).Flags &= !SHARED_CACHE_MAP_IN_LAZYWRITE;

        (*shared_cache_map).OpenCount -= 1;
        if (*shared_cache_map).OpenCount == 0 {
            CcRosDeleteFileCache((*shared_cache_map).FileObject, shared_cache_map, core::ptr::addr_of_mut!(old_irql));
        }

        if status >= 0 || status == 0xC0000011u32 as i32 || status == 0xC00000A2u32 as i32 {
            let pages_freed = (iosb.Information as ULONG) / (PAGE_SIZE as ULONG);
            *count += pages_freed;

            if wait == 0 {
                if local_target < pages_freed as ULONG {
                    local_target = 0;
                } else {
                    local_target -= pages_freed as ULONG;
                }
            }
        }

        current_entry = DirtyVacbListHead.Flink;
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
    KeLeaveCriticalRegion();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn CcRosTrimCache(
    target: ULONG,
    nr_freed: *mut ULONG
) {
    let nr_freed = opaque_ptr(nr_freed);
    let mut current_entry: PLIST_ENTRY;
    let mut current: PROS_VACB;
    let mut pages_freed: ULONG = 0;
    let mut old_irql: KIRQL;
    let mut free_list: _LIST_ENTRY = core::mem::zeroed();
    let mut flushed_pages = 0u8;
    let mut local_target = target;

    InitializeListHead(core::ptr::addr_of_mut!(free_list));

    *nr_freed = 0;

    loop {
        old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

        current_entry = VacbLruListHead.Flink;
        while current_entry != core::ptr::addr_of_mut!(VacbLruListHead) {
            current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, VacbLruListEntry) as isize)) as PROS_VACB;

            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*(*current).SharedCacheMap).CacheMapLock) as *mut _);

            CcRosVacbIncRefCount(current);

            if InterlockedCompareExchange(core::ptr::addr_of_mut!((*current).MappedCount) as *mut LONG, 0, 0) > 0 
                && (*current).Dirty == 0 {
            }

            current_entry = (*current_entry).Flink;

            let refs = CcRosVacbDecRefCount(current);

            if refs < 2 {
                RemoveEntryList(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
                RemoveEntryList(core::ptr::addr_of_mut!((*current).VacbLruListEntry));
                InitializeListHead(core::ptr::addr_of_mut!((*current).VacbLruListEntry));
                InsertHeadList(
                    core::ptr::addr_of_mut!(free_list),
                    core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry)
                );

                pages_freed = core::cmp::min(VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG, local_target);
                local_target -= pages_freed;
                *nr_freed += pages_freed;
            }

            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*(*current).SharedCacheMap).CacheMapLock) as *mut _);
        }

        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

        if local_target > 0 && flushed_pages == 0 {
            CcRosFlushDirtyPages(local_target, core::ptr::addr_of_mut!(pages_freed), 0u8, 0u8);
            flushed_pages = 1u8;

            if pages_freed < local_target {
                local_target = pages_freed;
            }

            if pages_freed != 0 {
                continue;
            }
        }

        break;
    }

    while !IsListEmpty(core::ptr::addr_of_mut!(free_list)) {
        current_entry = RemoveHeadList(core::ptr::addr_of_mut!(free_list));
        current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        InitializeListHead(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
        let refs = CcRosVacbDecRefCount(current);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CcRosReleaseVacb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    vacb: PROS_VACB,
    dirty: BOOLEAN,
    mapped: BOOLEAN
) -> NTSTATUS {
    if dirty != 0 && (*vacb).Dirty == 0 {
        CcRosMarkDirtyVacb(vacb);
    }

    if mapped != 0 {
        if InterlockedIncrement(core::ptr::addr_of_mut!((*vacb).MappedCount) as *mut LONG) == 1 {
            CcRosVacbIncRefCount(vacb);
        }
    }

    let refs = CcRosVacbDecRefCount(vacb);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn CcRosLookupVacb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: LONGLONG
) -> PROS_VACB {
    let mut current_entry: PLIST_ENTRY;
    let mut current: PROS_VACB;
    let old_irql: KIRQL;

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

    current_entry = (*shared_cache_map).CacheMapVacbListHead.Flink;
    while current_entry != core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead) {
        current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        
        if IsPointInRange(
            (*current).FileOffset.QuadPart,
            VACB_MAPPING_GRANULARITY,
            file_offset
        ) != 0 {
            CcRosVacbIncRefCount(current);
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
            return current;
        }
        
        if (*current).FileOffset.QuadPart > file_offset {
            break;
        }
        
        current_entry = (*current_entry).Flink;
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn CcRosMarkDirtyVacb(vacb: PROS_VACB) {
    let old_irql: KIRQL;
    let shared_cache_map: PROS_SHARED_CACHE_MAP;

    shared_cache_map = (*vacb).SharedCacheMap;

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

    // Upstream-sync merge (ntoskrnl/cc/view.c): another thread may have re-marked the VACB
    // dirty between a caller's lockless Dirty check and this call (e.g. a concurrent writer
    // via CcRosReleaseVacb). It is already on the dirty list; inserting again would corrupt
    // the list and inc-ref a second time with no matching decrement. Upstream turned a bare
    // ASSERT(!Vacb->Dirty) into this early return.
    if (*vacb).Dirty != 0 {
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
        return;
    }

    InsertTailList(
        core::ptr::addr_of_mut!(DirtyVacbListHead),
        core::ptr::addr_of_mut!((*vacb).DirtyVacbListEntry)
    );
    CcTotalDirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
    (*shared_cache_map).DirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
    CcRosVacbIncRefCount(vacb);

    RemoveEntryList(core::ptr::addr_of_mut!((*vacb).VacbLruListEntry));
    InsertTailList(
        core::ptr::addr_of_mut!(VacbLruListHead),
        core::ptr::addr_of_mut!((*vacb).VacbLruListEntry)
    );

    (*vacb).Dirty = 1u8;

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

    if LazyWriter.ScanActive == 0 {
        CcScheduleLazyWriteScan(0u8);
    }
    
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
}

#[no_mangle]
pub unsafe extern "C" fn CcRosUnmarkDirtyVacb(
    vacb: PROS_VACB,
    lock_views: BOOLEAN
) -> BOOLEAN {
    let mut old_irql: KIRQL = 0;
    let shared_cache_map: PROS_SHARED_CACHE_MAP;

    shared_cache_map = (*vacb).SharedCacheMap;

    if lock_views != 0 {
        old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    }

    // Upstream-sync merge added this guard (ntoskrnl/cc/view.c) against a real race: a
    // caller may check Vacb->Dirty lockless before calling in, and a concurrent flush
    // (e.g. the lazy writer) can unmark the VACB in between. Proceeding here would
    // underflow CcTotalDirtyPages/DirtyPages, drop an extra refcount via
    // CcRosVacbDecRefCount, and could free the VACB prematurely. Our pre-merge
    // translation carried the old, racy behavior with no such check.
    if (*vacb).Dirty == 0 {
        if lock_views != 0 {
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
        }
        return 0u8;
    }

    (*vacb).Dirty = 0u8;

    RemoveEntryList(core::ptr::addr_of_mut!((*vacb).DirtyVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*vacb).DirtyVacbListEntry));

    CcTotalDirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
    (*shared_cache_map).DirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;

    CcRosVacbDecRefCount(vacb);

    if lock_views != 0 {
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CcRosFreeOneUnusedVacb() -> BOOLEAN {
    let old_irql: KIRQL;
    let mut current_entry: PLIST_ENTRY;
    let mut to_free: PROS_VACB = core::ptr::null_mut();

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    current_entry = VacbLruListHead.Flink;
    while (current_entry != core::ptr::addr_of_mut!(VacbLruListHead)) && to_free.is_null() {
        let current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, VacbLruListEntry) as isize)) as PROS_VACB;

        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*(*current).SharedCacheMap).CacheMapLock) as *mut _);

        let refs = CcRosVacbGetRefCount(current);
        if refs < 2 {
            RemoveEntryList(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
            InitializeListHead(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
            RemoveEntryList(core::ptr::addr_of_mut!((*current).VacbLruListEntry));
            InitializeListHead(core::ptr::addr_of_mut!((*current).VacbLruListEntry));

            to_free = current;
        }

        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*(*current).SharedCacheMap).CacheMapLock) as *mut _);

        current_entry = (*current_entry).Flink;
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    if to_free.is_null() {
        return 0u8;
    }

    CcRosVacbDecRefCount(to_free);

    1u8
}

unsafe fn CcRosCreateVacb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: LONGLONG,
    vacb: *mut PROS_VACB
) -> NTSTATUS {
    let mut current: PROS_VACB;
    let mut previous: PROS_VACB;
    let mut current_entry: PLIST_ENTRY;
    let mut status: NTSTATUS;
    let mut old_irql: KIRQL;
    let mut view_size: SIZE_T = VACB_MAPPING_GRANULARITY as SIZE_T;

    current = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(VacbLookasideList)) as PROS_VACB;
    if current.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    
    (*current).BaseAddress = core::ptr::null_mut();
    (*current).Dirty = 0u8;
    (*current).PageOut = 0;
    (*current).FileOffset.QuadPart = (file_offset / VACB_MAPPING_GRANULARITY as i64) * VACB_MAPPING_GRANULARITY as i64;
    (*current).SharedCacheMap = shared_cache_map;
    (*current).MappedCount = 0;
    (*current).ReferenceCount = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*current).DirtyVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*current).VacbLruListEntry));

    CcRosVacbIncRefCount(current);

    loop {
        status = MmMapViewInSystemSpaceEx(
            (*shared_cache_map).Section,
            core::ptr::addr_of_mut!((*current).BaseAddress),
            core::ptr::addr_of_mut!(view_size),
            core::ptr::addr_of_mut!((*current).FileOffset),
            0
        );
        
        if status >= 0 {
            #[cfg(target_arch = "aarch64")]
            {
                core::arch::asm!("dsb sy", "isb", options(nostack));
            }
            break;
        }

        if CcRosFreeOneUnusedVacb() == 0 {
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(VacbLookasideList), current as PVOID);
            return status;
        }
    }

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    *vacb = current;
    
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    current_entry = (*shared_cache_map).CacheMapVacbListHead.Flink;
    previous = core::ptr::null_mut();
    
    while current_entry != core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead) {
        current = (current_entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        
        if IsPointInRange(
            (*current).FileOffset.QuadPart,
            VACB_MAPPING_GRANULARITY,
            file_offset
        ) != 0 {
            CcRosVacbIncRefCount(current);
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

            let refs = CcRosVacbDecRefCount(*vacb);

            *vacb = current;
            return STATUS_SUCCESS;
        }
        
        if (*current).FileOffset.QuadPart < file_offset {
            previous = current;
        }
        
        if (*current).FileOffset.QuadPart > file_offset {
            break;
        }
        
        current_entry = (*current_entry).Flink;
    }
    
    current = *vacb;
    if !previous.is_null() {
        InsertHeadList(
            core::ptr::addr_of_mut!((*previous).CacheMapVacbListEntry),
            core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry)
        );
    } else {
        InsertHeadList(
            core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead),
            core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry)
        );
    }
    
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    InsertTailList(
        core::ptr::addr_of_mut!(VacbLruListHead),
        core::ptr::addr_of_mut!((*current).VacbLruListEntry)
    );

    CcRosVacbIncRefCount(current);

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    status
}

#[no_mangle]
pub unsafe extern "C" fn CcRosEnsureVacbResident(
    vacb: PROS_VACB,
    wait: BOOLEAN,
    no_read: BOOLEAN,
    offset: ULONG,
    length: ULONG
) -> BOOLEAN {
    let shared_cache_map = (*vacb).SharedCacheMap;

    if MmIsDataSectionResident(
        (*(*shared_cache_map).FileObject).SectionObjectPointer,
        (*vacb).FileOffset.QuadPart + offset as i64,
        length
    ) == 0 {
        if wait == 0 {
            return 0u8;
        }

        if no_read == 0 {
            let status = MmMakeDataSectionResident(
                (*(*shared_cache_map).FileObject).SectionObjectPointer,
                (*vacb).FileOffset.QuadPart + offset as i64,
                length,
                core::ptr::addr_of_mut!((*shared_cache_map).ValidDataLength)
            );

            if status < 0 {
                // DUMMY_PSEH: return FALSE instead of raising
                return 0u8;
            }
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CcRosGetVacb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: LONGLONG,
    vacb: *mut PROS_VACB
) -> NTSTATUS {
    let vacb = opaque_ptr(vacb);
    let mut current: PROS_VACB;
    let status: NTSTATUS;
    let mut old_irql: KIRQL;

    current = CcRosLookupVacb(shared_cache_map, file_offset);
    if current.is_null() {
        let status = CcRosCreateVacb(shared_cache_map, file_offset, core::ptr::addr_of_mut!(current));
        if status < 0 {
            return status;
        }
    }

    let refs = CcRosVacbGetRefCount(current);

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    RemoveEntryList(core::ptr::addr_of_mut!((*current).VacbLruListEntry));
    InsertTailList(
        core::ptr::addr_of_mut!(VacbLruListHead),
        core::ptr::addr_of_mut!((*current).VacbLruListEntry)
    );

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    *vacb = current;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn CcRosRequestVacb(
    shared_cache_map: PROS_SHARED_CACHE_MAP,
    file_offset: LONGLONG,
    vacb: *mut PROS_VACB
) -> NTSTATUS {
    let vacb = opaque_ptr(vacb);
    if file_offset % VACB_MAPPING_GRANULARITY as i64 != 0 {
        KeBugCheck(ntkrnl_sys::bugcodes::CACHE_MANAGER);
    }

    CcRosGetVacb(shared_cache_map, file_offset, vacb)
}

#[no_mangle]
pub unsafe extern "C" fn CcRosInternalFreeVacb(vacb: PROS_VACB) -> NTSTATUS {
    let status: NTSTATUS;

    status = MmUnmapViewInSystemSpace((*vacb).BaseAddress);

    RtlFillMemory(vacb as PVOID, core::mem::size_of::<ROS_VACB>(), 0xfd);
    ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(VacbLookasideList), vacb as PVOID);
    
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcFlushCache(
    section_object_pointers: PSECTION_OBJECT_POINTERS,
    file_offset: PLARGE_INTEGER,
    length: ULONG,
    io_status: PIO_STATUS_BLOCK
) {
    let file_offset = opaque_ptr(file_offset);
    let io_status = opaque_ptr(io_status);
    let shared_cache_map: PROS_SHARED_CACHE_MAP;
    let mut flush_start: LONGLONG;
    let mut flush_end: LONGLONG;
    let mut status: NTSTATUS;

    if section_object_pointers.is_null() {
        status = STATUS_INVALID_PARAMETER;
        if !io_status.is_null() {
            (*io_status).__bindgen_anon_1.Status = status;
        }
        return;
    }

    if (*section_object_pointers).SharedCacheMap.is_null() {
        MmFlushSegment(section_object_pointers, file_offset, length, io_status);
        return;
    }

    shared_cache_map = (*section_object_pointers).SharedCacheMap as PROS_SHARED_CACHE_MAP;
    
    if !file_offset.is_null() {
        flush_start = (*file_offset).QuadPart;
        flush_end = flush_start + length as i64;
    } else {
        flush_start = 0;
        flush_end = (*shared_cache_map).FileSize.QuadPart;
    }

    status = STATUS_SUCCESS;
    if !io_status.is_null() {
        (*io_status).Information = 0;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!((*shared_cache_map).FlushCacheLock));

    while flush_start < flush_end {
        let mut dirty_vacb = 0u8;
        let vacb = CcRosLookupVacb(shared_cache_map, flush_start);

        if !vacb.is_null() {
            if (*vacb).Dirty != 0 {
                let mut vacb_iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
                status = CcRosFlushVacb(vacb, &mut vacb_iosb);
                if status < 0 {
                    CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
                    break;
                }
                dirty_vacb = 1u8;

                if !io_status.is_null() {
                    (*io_status).Information += vacb_iosb.Information;
                }
            }

            CcRosReleaseVacb(shared_cache_map, vacb, 0u8, 0u8);
        }

        if dirty_vacb == 0 {
            let mut mm_iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut mm_offset: _LARGE_INTEGER = core::mem::zeroed();

            mm_offset.QuadPart = flush_start;

            if flush_end - (flush_end / VACB_MAPPING_GRANULARITY as i64 * VACB_MAPPING_GRANULARITY as i64) <= flush_start {
                status = MmFlushSegment(
                    section_object_pointers,
                    &mut mm_offset,
                    (flush_end - flush_start) as ULONG,
                    &mut mm_iosb
                );
            } else {
                let mm_length = VACB_MAPPING_GRANULARITY - (flush_start % VACB_MAPPING_GRANULARITY as i64) as ULONG;
                status = MmFlushSegment(
                    section_object_pointers,
                    &mut mm_offset,
                    mm_length,
                    &mut mm_iosb
                );
            }

            if status < 0 {
                break;
            }

            if !io_status.is_null() {
                (*io_status).Information += mm_iosb.Information;
            }

            if (*shared_cache_map).ValidDataLength.QuadPart < flush_end {
                (*shared_cache_map).ValidDataLength.QuadPart = flush_end;
            }
        }

        flush_start += VACB_MAPPING_GRANULARITY as i64;
        flush_start -= flush_start % VACB_MAPPING_GRANULARITY as i64;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*shared_cache_map).FlushCacheLock));

    if !io_status.is_null() {
        (*io_status).__bindgen_anon_1.Status = status;
    }
}

#[no_mangle]
pub unsafe extern "C" fn CcRosReleaseFileCache(file_object: PFILE_OBJECT) -> NTSTATUS {
    let file_object = opaque_ptr(file_object);
    let mut old_irql: KIRQL;
    let mut private_map: PPRIVATE_CACHE_MAP;
    let shared_cache_map: PROS_SHARED_CACHE_MAP;

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    if !(*(*file_object).SectionObjectPointer).SharedCacheMap.is_null() {
        shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;

        private_map = (*file_object).PrivateCacheMap as PPRIVATE_CACHE_MAP;
        (*file_object).PrivateCacheMap = core::ptr::null_mut();

        if !private_map.is_null() {
            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
            RemoveEntryList(core::ptr::addr_of_mut!((*private_map).PrivateLinks));
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

            if private_map != core::ptr::addr_of_mut!((*shared_cache_map).PrivateCacheMap) {
                ExFreePoolWithTag(private_map as PVOID, TAG_PRIVATE_CACHE_MAP);
            } else {
                (*private_map).__bindgen_anon_1.NodeTypeCode = 0;
            }

            (*shared_cache_map).OpenCount -= 1;
            if (*shared_cache_map).OpenCount == 0 {
                CcRosDeleteFileCache(file_object, shared_cache_map, core::ptr::addr_of_mut!(old_irql));
            }
        }
    }
    
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn CcRosInitializeFileCache(
    file_object: PFILE_OBJECT,
    file_sizes: PCC_FILE_SIZES,
    pin_access: BOOLEAN,
    call_backs: PCACHE_MANAGER_CALLBACKS,
    lazy_writer_context: PVOID
) -> NTSTATUS {
    let file_object = opaque_ptr(file_object);
    let lazy_writer_context = opaque_ptr(lazy_writer_context);
    let mut old_irql: KIRQL;
    let mut allocated: BOOLEAN;
    let mut shared_cache_map: PROS_SHARED_CACHE_MAP;

    old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    allocated = 0u8;
    shared_cache_map = (*(*file_object).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
    
    if shared_cache_map.is_null() {
        shared_cache_map = ExAllocateFromNPagedLookasideList(
            core::ptr::addr_of_mut!(SharedCacheMapLookasideList)
        ) as PROS_SHARED_CACHE_MAP;
        
        if shared_cache_map.is_null() {
            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        
        RtlZeroMemory(shared_cache_map as PVOID, core::mem::size_of::<ROS_SHARED_CACHE_MAP>());
        (*shared_cache_map).NodeTypeCode = NODE_TYPE_SHARED_MAP;
        (*shared_cache_map).NodeByteSize = core::mem::size_of::<ROS_SHARED_CACHE_MAP>() as USHORT;
        (*shared_cache_map).FileObject = file_object;
        (*shared_cache_map).Callbacks = call_backs;
        (*shared_cache_map).LazyWriteContext = lazy_writer_context;
        (*shared_cache_map).SectionSize = (*file_sizes).AllocationSize;
        (*shared_cache_map).FileSize = (*file_sizes).FileSize;
        (*shared_cache_map).ValidDataLength = (*file_sizes).ValidDataLength;
        (*shared_cache_map).PinAccess = pin_access;
        (*shared_cache_map).DirtyPageThreshold = 0;
        (*shared_cache_map).DirtyPages = 0;
        InitializeListHead(core::ptr::addr_of_mut!((*shared_cache_map).PrivateList));
        KeInitializeSpinLock(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock));
        InitializeListHead(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead));
        InitializeListHead(core::ptr::addr_of_mut!((*shared_cache_map).BcbList));
        KeInitializeGuardedMutex(core::ptr::addr_of_mut!((*shared_cache_map).FlushCacheLock));

        (*shared_cache_map).Flags = SHARED_CACHE_MAP_IN_CREATION;

        ObReferenceObjectByPointer(
            file_object as PVOID,
            0x1F01FF,
            core::ptr::null_mut(),
            0i8
        );

        allocated = 1u8;
        (*(*file_object).SectionObjectPointer).SharedCacheMap = shared_cache_map as PVOID;
    } else if (*shared_cache_map).Flags & SHARED_CACHE_MAP_IN_CREATION != 0 {
        let previous_waiter = (*shared_cache_map).CreateEvent;
        let mut waiter: _KEVENT = core::mem::zeroed();

        KeInitializeEvent(&mut waiter, 0, 0u8);
        (*shared_cache_map).CreateEvent = &mut waiter;

        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

        KeWaitForSingleObject(&mut waiter as *mut _ as PVOID, 0, 0i8, 0u8, core::ptr::null_mut());

        if !previous_waiter.is_null() {
            KeSetEvent(previous_waiter, 0, 0u8);
        }

        old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    }

    if (*file_object).PrivateCacheMap.is_null() {
        let private_map: PPRIVATE_CACHE_MAP;

        if (*shared_cache_map).PrivateCacheMap.__bindgen_anon_1.NodeTypeCode != 0 {
            private_map = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_PRIVATE_CACHE_MAP>(), TAG_PRIVATE_CACHE_MAP) as PPRIVATE_CACHE_MAP;
        } else {
            private_map = core::ptr::addr_of_mut!((*shared_cache_map).PrivateCacheMap);
        }

        if private_map.is_null() {
            if allocated != 0 {
                RemoveEntryList(core::ptr::addr_of_mut!((*shared_cache_map).SharedCacheMapLinks));
                (*(*file_object).SectionObjectPointer).SharedCacheMap = core::ptr::null_mut();
                ObDereferenceObject(file_object as PVOID);
                ExFreeToNPagedLookasideList(
                    core::ptr::addr_of_mut!(SharedCacheMapLookasideList),
                    shared_cache_map as PVOID
                );
            }

            KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(private_map as PVOID, core::mem::size_of::<_PRIVATE_CACHE_MAP>());
        (*private_map).__bindgen_anon_1.NodeTypeCode = NODE_TYPE_PRIVATE_MAP as i16;
        (*private_map).ReadAheadMask = PAGE_SIZE as ULONG - 1;
        (*private_map).FileObject = file_object;
        (*private_map).ReadAheadSpinLock = 0;

        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
        InsertTailList(
            core::ptr::addr_of_mut!((*shared_cache_map).PrivateList),
            core::ptr::addr_of_mut!((*private_map).PrivateLinks)
        );
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

        (*file_object).PrivateCacheMap = private_map as PVOID;
        (*shared_cache_map).OpenCount += 1;
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    if allocated != 0 {
        let mut status: NTSTATUS;

        status = MmCreateSection(
            core::ptr::addr_of_mut!((*shared_cache_map).Section),
            0x1F001F,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*shared_cache_map).SectionSize),
            PAGE_READWRITE,
            SEC_RESERVE,
            core::ptr::null_mut(),
            file_object
        );

        if status < 0 {
            CcRosReleaseFileCache(file_object);
            return status;
        }

        old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

        InsertTailList(
            core::ptr::addr_of_mut!(CcCleanSharedCacheMapList),
            core::ptr::addr_of_mut!((*shared_cache_map).SharedCacheMapLinks)
        );
        (*shared_cache_map).Flags &= !SHARED_CACHE_MAP_IN_CREATION;

        if !(*shared_cache_map).CreateEvent.is_null() {
            KeSetEvent((*shared_cache_map).CreateEvent, 0, 0u8);
            (*shared_cache_map).CreateEvent = core::ptr::null_mut();
        }

        KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileObjectFromSectionPtrs(
    section_object_pointers: PSECTION_OBJECT_POINTERS
) -> PFILE_OBJECT {
    let shared_cache_map: PROS_SHARED_CACHE_MAP;

    if !section_object_pointers.is_null() && !(*section_object_pointers).SharedCacheMap.is_null() {
        shared_cache_map = (*section_object_pointers).SharedCacheMap as PROS_SHARED_CACHE_MAP;
        return (*shared_cache_map).FileObject;
    }
    
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CcInitView() {
    InitializeListHead(core::ptr::addr_of_mut!(DirtyVacbListHead));
    InitializeListHead(core::ptr::addr_of_mut!(VacbLruListHead));
    InitializeListHead(core::ptr::addr_of_mut!(CcDeferredWrites));
    InitializeListHead(core::ptr::addr_of_mut!(CcCleanSharedCacheMapList));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(CcDeferredWriteSpinLock));
    
    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(iBcbLookasideList),
        None,
        None,
        0,
        core::mem::size_of::<_INTERNAL_BCB>(),
        TAG_BCB,
        20
    );

    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(SharedCacheMapLookasideList),
        None,
        None,
        0,
        core::mem::size_of::<ROS_SHARED_CACHE_MAP>(),
        TAG_SHARED_CACHE_MAP,
        20
    );

    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(VacbLookasideList),
        None,
        None,
        0,
        core::mem::size_of::<ROS_VACB>(),
        TAG_VACB,
        20
    );

    CcInitCacheZeroPage();
}

extern "system" {
    // stdcall per ntoskrnl.spec (_DbgBreakPoint@0 on i386); zero args so only the
    // link decoration differs from cdecl — declared right so i386 binds the real export.
    fn DbgBreakPoint();
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
extern "system" {
    fn CcInitCacheZeroPage();
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/view.c
//  * PURPOSE:         Cache manager
//  *
//  * PROGRAMMERS:     David Welch (welch@mcmail.com)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* NOTES **********************************************************************
//  *
//  * This is not the NT implementation of a file cache nor anything much like
//  * it.
//  *
//  * The general procedure for a filesystem to implement a read or write
//  * dispatch routine is as follows
//  *
//  * (1) If caching for the FCB hasn't been initiated then so do by calling
//  * CcInitializeFileCache.
//  *
//  * (2) For each 4k region which is being read or written obtain a cache page
//  * by calling CcRequestCachePage.
//  *
//  * (3) If either the page is being read or not completely written, and it is
//  * not up to date then read its data from the underlying medium. If the read
//  * fails then call CcReleaseCachePage with VALID as FALSE and return a error.
//  *
//  * (4) Copy the data into or out of the page as necessary.
//  *
//  * (5) Release the cache page
//  */
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY DirtyVacbListHead;
// static LIST_ENTRY VacbLruListHead;
//
// NPAGED_LOOKASIDE_LIST iBcbLookasideList;
// static NPAGED_LOOKASIDE_LIST SharedCacheMapLookasideList;
// static NPAGED_LOOKASIDE_LIST VacbLookasideList;
//
// /* Internal vars (MS):
//  * - Threshold above which lazy writer will start action
//  * - Amount of dirty pages
//  * - List for deferred writes
//  * - Spinlock when dealing with the deferred list
//  * - List for "clean" shared cache maps
//  */
// ULONG CcDirtyPageThreshold = 0;
// ULONG CcTotalDirtyPages = 0;
// LIST_ENTRY CcDeferredWrites;
// KSPIN_LOCK CcDeferredWriteSpinLock;
// LIST_ENTRY CcCleanSharedCacheMapList;
//
// #if DBG
// ULONG CcRosVacbIncRefCount_(PROS_VACB vacb, PCSTR file, INT line)
// {
//     ULONG Refs;
//
//     Refs = InterlockedIncrement((PLONG)&vacb->ReferenceCount);
//     if (vacb->SharedCacheMap->Trace)
//     {
//         DbgPrint("(%s:%i) VACB %p ++RefCount=%lu, Dirty %u, PageOut %lu\n",
//                  file, line, vacb, Refs, vacb->Dirty, vacb->PageOut);
//     }
//
//     return Refs;
// }
// ULONG CcRosVacbDecRefCount_(PROS_VACB vacb, PCSTR file, INT line)
// {
//     ULONG Refs;
//     BOOLEAN VacbDirty = vacb->Dirty;
//     BOOLEAN VacbTrace = vacb->SharedCacheMap->Trace;
//     BOOLEAN VacbPageOut = vacb->PageOut;
//
//     Refs = InterlockedDecrement((PLONG)&vacb->ReferenceCount);
//     ASSERT(!(Refs == 0 && VacbDirty));
//     if (VacbTrace)
//     {
//         DbgPrint("(%s:%i) VACB %p --RefCount=%lu, Dirty %u, PageOut %lu\n",
//                  file, line, vacb, Refs, VacbDirty, VacbPageOut);
//     }
//
//     if (Refs == 0)
//     {
//         CcRosInternalFreeVacb(vacb);
//     }
//
//     return Refs;
// }
// ULONG CcRosVacbGetRefCount_(PROS_VACB vacb, PCSTR file, INT line)
// {
//     ULONG Refs;
//
//     Refs = InterlockedCompareExchange((PLONG)&vacb->ReferenceCount, 0, 0);
//     if (vacb->SharedCacheMap->Trace)
//     {
//         DbgPrint("(%s:%i) VACB %p ==RefCount=%lu, Dirty %u, PageOut %lu\n",
//                  file, line, vacb, Refs, vacb->Dirty, vacb->PageOut);
//     }
//
//     return Refs;
// }
// #endif
//
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// CcRosTraceCacheMap (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     BOOLEAN Trace )
// {
// #if DBG
//     KIRQL oldirql;
//     PLIST_ENTRY current_entry;
//     PROS_VACB current;
//
//     if (!SharedCacheMap)
//         return;
//
//     SharedCacheMap->Trace = Trace;
//
//     if (Trace)
//     {
//         DPRINT1("Enabling Tracing for CacheMap 0x%p:\n", SharedCacheMap);
//
//         oldirql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//         KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//
//         current_entry = SharedCacheMap->CacheMapVacbListHead.Flink;
//         while (current_entry != &SharedCacheMap->CacheMapVacbListHead)
//         {
//             current = CONTAINING_RECORD(current_entry, ROS_VACB, CacheMapVacbListEntry);
//             current_entry = current_entry->Flink;
//
//             DPRINT1("  VACB 0x%p enabled, RefCount %lu, Dirty %u, PageOut %lu, BaseAddress %p, FileOffset %I64d\n",
//                     current, current->ReferenceCount, current->Dirty, current->PageOut, current->BaseAddress, current->FileOffset.QuadPart);
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, oldirql);
//     }
//     else
//     {
//         DPRINT1("Disabling Tracing for CacheMap 0x%p:\n", SharedCacheMap);
//     }
//
// #else
//     UNREFERENCED_PARAMETER(SharedCacheMap);
//     UNREFERENCED_PARAMETER(Trace);
// #endif
// }
//
// NTSTATUS
// CcRosFlushVacb (
//     _In_ PROS_VACB Vacb,
//     _Out_opt_ PIO_STATUS_BLOCK Iosb)
// {
//     NTSTATUS Status;
//     BOOLEAN HaveLock = FALSE;
//     PROS_SHARED_CACHE_MAP SharedCacheMap = Vacb->SharedCacheMap;
//
//     CcRosUnmarkDirtyVacb(Vacb, TRUE);
//
//     /* Lock for flush, if we are not already the top-level */
//     if (IoGetTopLevelIrp() != (PIRP)FSRTL_CACHE_TOP_LEVEL_IRP)
//     {
//         Status = FsRtlAcquireFileForCcFlushEx(Vacb->SharedCacheMap->FileObject);
//         if (!NT_SUCCESS(Status))
//             goto quit;
//         HaveLock = TRUE;
//     }
//
//     Status = MmFlushSegment(SharedCacheMap->FileObject->SectionObjectPointer,
//                             &Vacb->FileOffset,
//                             VACB_MAPPING_GRANULARITY,
//                             Iosb);
//
//     if (HaveLock)
//     {
//         FsRtlReleaseFileForCcFlush(Vacb->SharedCacheMap->FileObject);
//     }
//
// quit:
//     if (!NT_SUCCESS(Status))
//         CcRosMarkDirtyVacb(Vacb);
//     else
//     {
//         /* Update VDL */
//         if (SharedCacheMap->ValidDataLength.QuadPart < (Vacb->FileOffset.QuadPart + VACB_MAPPING_GRANULARITY))
//         {
//             SharedCacheMap->ValidDataLength.QuadPart = Vacb->FileOffset.QuadPart + VACB_MAPPING_GRANULARITY;
//         }
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// CcRosDeleteFileCache (
//     PFILE_OBJECT FileObject,
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     PKIRQL OldIrql)
// /*
//  * FUNCTION: Releases the shared cache map associated with a file object
//  */
// {
//     PLIST_ENTRY current_entry;
//
//     ASSERT(SharedCacheMap);
//     ASSERT(SharedCacheMap == FileObject->SectionObjectPointer->SharedCacheMap);
//     ASSERT(SharedCacheMap->OpenCount == 0);
//
//     /* Remove all VACBs from the global lists */
//     KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//     current_entry = SharedCacheMap->CacheMapVacbListHead.Flink;
//     while (current_entry != &SharedCacheMap->CacheMapVacbListHead)
//     {
//         PROS_VACB Vacb = CONTAINING_RECORD(current_entry, ROS_VACB, CacheMapVacbListEntry);
//
//         RemoveEntryList(&Vacb->VacbLruListEntry);
//         InitializeListHead(&Vacb->VacbLruListEntry);
//
//         if (Vacb->Dirty)
//         {
//             CcRosUnmarkDirtyVacb(Vacb, FALSE);
//             /* Mark it as dirty again so we know that we have to flush before freeing it */
//             Vacb->Dirty = TRUE;
//         }
//
//         current_entry = current_entry->Flink;
//     }
//
//     /* Make sure there is no trace anymore of this map */
//     FileObject->SectionObjectPointer->SharedCacheMap = NULL;
//     RemoveEntryList(&SharedCacheMap->SharedCacheMapLinks);
//
//     KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, *OldIrql);
//
//     /* Now that we're out of the locks, free everything for real */
//     while (!IsListEmpty(&SharedCacheMap->CacheMapVacbListHead))
//     {
//         PROS_VACB Vacb = CONTAINING_RECORD(RemoveHeadList(&SharedCacheMap->CacheMapVacbListHead), ROS_VACB, CacheMapVacbListEntry);
//         ULONG RefCount;
//
//         InitializeListHead(&Vacb->CacheMapVacbListEntry);
//
//         /* Flush to disk, if needed */
//         if (Vacb->Dirty)
//         {
//             IO_STATUS_BLOCK Iosb;
//             NTSTATUS Status;
//
//             Status = MmFlushSegment(FileObject->SectionObjectPointer, &Vacb->FileOffset, VACB_MAPPING_GRANULARITY, &Iosb);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Complain. There's not much we can do */
//                 DPRINT1("Failed to flush VACB to disk while deleting the cache entry. Status: 0x%08x\n", Status);
//             }
//             Vacb->Dirty = FALSE;
//         }
//
//         RefCount = CcRosVacbDecRefCount(Vacb);
// #if DBG // CORE-14578
//         if (RefCount != 0)
//         {
//             DPRINT1("Leaking VACB %p attached to %p (%I64d)\n", Vacb, FileObject, Vacb->FileOffset.QuadPart);
//             DPRINT1("There are: %d references left\n", RefCount);
//             DPRINT1("Map: %d\n", Vacb->MappedCount);
//             DPRINT1("Dirty: %d\n", Vacb->Dirty);
//             if (FileObject->FileName.Length != 0)
//             {
//                 DPRINT1("File was: %wZ\n", &FileObject->FileName);
//             }
//             else
//             {
//                 DPRINT1("No name for the file\n");
//             }
//         }
// #else
//         (void)RefCount;
// #endif
//     }
//
//     /* Release the references we own */
//     if(SharedCacheMap->Section)
//         ObDereferenceObject(SharedCacheMap->Section);
//     ObDereferenceObject(SharedCacheMap->FileObject);
//
//     ExFreeToNPagedLookasideList(&SharedCacheMapLookasideList, SharedCacheMap);
//
//     /* Acquire the lock again for our caller */
//     *OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CcRosFlushDirtyPages (
//     ULONG Target,
//     PULONG Count,
//     BOOLEAN Wait,
//     BOOLEAN CalledFromLazy)
// {
//     PLIST_ENTRY current_entry;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     BOOLEAN FlushAll = (Target == MAXULONG);
//
//     DPRINT("CcRosFlushDirtyPages(Target %lu)\n", Target);
//
//     (*Count) = 0;
//
//     KeEnterCriticalRegion();
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     current_entry = DirtyVacbListHead.Flink;
//     if (current_entry == &DirtyVacbListHead)
//     {
//         DPRINT("No Dirty pages\n");
//     }
//
//     while (((current_entry != &DirtyVacbListHead) && (Target > 0)) || FlushAll)
//     {
//         PROS_SHARED_CACHE_MAP SharedCacheMap;
//         PROS_VACB current;
//         BOOLEAN Locked;
//
//         if (current_entry == &DirtyVacbListHead)
//         {
//             ASSERT(FlushAll);
//             if (IsListEmpty(&DirtyVacbListHead))
//                 break;
//             current_entry = DirtyVacbListHead.Flink;
//         }
//
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     DirtyVacbListEntry);
//         current_entry = current_entry->Flink;
//
//         CcRosVacbIncRefCount(current);
//
//         SharedCacheMap = current->SharedCacheMap;
//
//         /* When performing lazy write, don't handle temporary files */
//         if (CalledFromLazy && BooleanFlagOn(SharedCacheMap->FileObject->Flags, FO_TEMPORARY_FILE))
//         {
//             CcRosVacbDecRefCount(current);
//             continue;
//         }
//
//         /* Don't attempt to lazy write the files that asked not to */
//         if (CalledFromLazy && BooleanFlagOn(SharedCacheMap->Flags, WRITEBEHIND_DISABLED))
//         {
//             CcRosVacbDecRefCount(current);
//             continue;
//         }
//
//         ASSERT(current->Dirty);
//
//         /* Do not lazy-write the same file concurrently. Fastfat ASSERTS on that */
//         if (SharedCacheMap->Flags & SHARED_CACHE_MAP_IN_LAZYWRITE)
//         {
//             CcRosVacbDecRefCount(current);
//             continue;
//         }
//
//         SharedCacheMap->Flags |= SHARED_CACHE_MAP_IN_LAZYWRITE;
//
//         /* Keep a ref on the shared cache map */
//         SharedCacheMap->OpenCount++;
//
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//         Locked = SharedCacheMap->Callbacks->AcquireForLazyWrite(SharedCacheMap->LazyWriteContext, Wait);
//         if (!Locked)
//         {
//             DPRINT("Not locked\n");
//             ASSERT(!Wait);
//             CcRosVacbDecRefCount(current);
//             OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//             SharedCacheMap->Flags &= ~SHARED_CACHE_MAP_IN_LAZYWRITE;
//
//             if (--SharedCacheMap->OpenCount == 0)
//                 CcRosDeleteFileCache(SharedCacheMap->FileObject, SharedCacheMap, &OldIrql);
//
//             continue;
//         }
//
//         IO_STATUS_BLOCK Iosb;
//         Status = CcRosFlushVacb(current, &Iosb);
//
//         SharedCacheMap->Callbacks->ReleaseFromLazyWrite(SharedCacheMap->LazyWriteContext);
//
//         /* We release the VACB before acquiring the lock again, because
//          * CcRosVacbDecRefCount might free the VACB, as CcRosFlushVacb dropped a
//          * Refcount. Freeing must be done outside of the lock.
//          * The refcount is decremented atomically. So this is OK. */
//         CcRosVacbDecRefCount(current);
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//         SharedCacheMap->Flags &= ~SHARED_CACHE_MAP_IN_LAZYWRITE;
//
//         if (--SharedCacheMap->OpenCount == 0)
//             CcRosDeleteFileCache(SharedCacheMap->FileObject, SharedCacheMap, &OldIrql);
//
//         if (!NT_SUCCESS(Status) && (Status != STATUS_END_OF_FILE) &&
//             (Status != STATUS_MEDIA_WRITE_PROTECTED))
//         {
//             DPRINT1("CC: Failed to flush VACB.\n");
//         }
//         else
//         {
//             ULONG PagesFreed;
//
//             /* How many pages did we free? */
//             PagesFreed = Iosb.Information / PAGE_SIZE;
//             (*Count) += PagesFreed;
//
//             if (!Wait)
//             {
//                 /* Make sure we don't overflow target! */
//                 if (Target < PagesFreed)
//                 {
//                     /* If we would have, jump to zero directly */
//                     Target = 0;
//                 }
//                 else
//                 {
//                     Target -= PagesFreed;
//                 }
//             }
//         }
//
//         current_entry = DirtyVacbListHead.Flink;
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//     KeLeaveCriticalRegion();
//
//     DPRINT("CcRosFlushDirtyPages() finished\n");
//     return STATUS_SUCCESS;
// }
//
// VOID
// CcRosTrimCache(
//     _In_ ULONG Target,
//     _Out_ PULONG NrFreed)
// /*
//  * FUNCTION: Try to free some memory from the file cache.
//  * ARGUMENTS:
//  *       Target - The number of pages to be freed.
//  *       NrFreed - Points to a variable where the number of pages
//  *                 actually freed is returned.
//  */
// {
//     PLIST_ENTRY current_entry;
//     PROS_VACB current;
//     ULONG PagesFreed;
//     KIRQL oldIrql;
//     LIST_ENTRY FreeList;
//     BOOLEAN FlushedPages = FALSE;
//
//     DPRINT("CcRosTrimCache(Target %lu)\n", Target);
//
//     InitializeListHead(&FreeList);
//
//     *NrFreed = 0;
//
// retry:
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     current_entry = VacbLruListHead.Flink;
//     while (current_entry != &VacbLruListHead)
//     {
//         ULONG Refs;
//
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     VacbLruListEntry);
//
//         KeAcquireSpinLockAtDpcLevel(&current->SharedCacheMap->CacheMapLock);
//
//         /* Reference the VACB */
//         CcRosVacbIncRefCount(current);
//
//         /* Check if it's mapped and not dirty */
//         if (InterlockedCompareExchange((PLONG)&current->MappedCount, 0, 0) > 0 && !current->Dirty)
//         {
//             /* This code is never executed. It is left for reference only. */
// #if 1
//             DPRINT1("MmPageOutPhysicalAddress unexpectedly called\n");
//             ASSERT(FALSE);
// #else
//             ULONG i;
//             PFN_NUMBER Page;
//
//             /* We have to break these locks to call MmPageOutPhysicalAddress */
//             KeReleaseSpinLockFromDpcLevel(&current->SharedCacheMap->CacheMapLock);
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//             /* Page out the VACB */
//             for (i = 0; i < VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG; i++)
//             {
//                 Page = (PFN_NUMBER)(MmGetPhysicalAddress((PUCHAR)current->BaseAddress + (i * PAGE_SIZE)).QuadPart >> PAGE_SHIFT);
//
//                 MmPageOutPhysicalAddress(Page);
//             }
//
//             /* Reacquire the locks */
//             oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//             KeAcquireSpinLockAtDpcLevel(&current->SharedCacheMap->CacheMapLock);
// #endif
//         }
//
//         /* Only keep iterating though the loop while the lock is held */
//         current_entry = current_entry->Flink;
//
//         /* Dereference the VACB */
//         Refs = CcRosVacbDecRefCount(current);
//
//         /* Check if we can free this entry now */
//         if (Refs < 2)
//         {
//             ASSERT(!current->Dirty);
//             ASSERT(!current->MappedCount);
//             ASSERT(Refs == 1);
//
//             RemoveEntryList(&current->CacheMapVacbListEntry);
//             RemoveEntryList(&current->VacbLruListEntry);
//             InitializeListHead(&current->VacbLruListEntry);
//             InsertHeadList(&FreeList, &current->CacheMapVacbListEntry);
//
//             /* Calculate how many pages we freed for Mm */
//             PagesFreed = min(VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG, Target);
//             Target -= PagesFreed;
//             (*NrFreed) += PagesFreed;
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&current->SharedCacheMap->CacheMapLock);
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//     /* Try flushing pages if we haven't met our target */
//     if ((Target > 0) && !FlushedPages)
//     {
//         /* Flush dirty pages to disk */
//         CcRosFlushDirtyPages(Target, &PagesFreed, FALSE, FALSE);
//         FlushedPages = TRUE;
//
//         /* We can only swap as many pages as we flushed */
//         if (PagesFreed < Target) Target = PagesFreed;
//
//         /* Check if we flushed anything */
//         if (PagesFreed != 0)
//         {
//             /* Try again after flushing dirty pages */
//             DPRINT("Flushed %lu dirty cache pages to disk\n", PagesFreed);
//             goto retry;
//         }
//     }
//
//     while (!IsListEmpty(&FreeList))
//     {
//         ULONG Refs;
//
//         current_entry = RemoveHeadList(&FreeList);
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     CacheMapVacbListEntry);
//         InitializeListHead(&current->CacheMapVacbListEntry);
//         Refs = CcRosVacbDecRefCount(current);
//         ASSERT(Refs == 0);
//     }
//
//     DPRINT("Evicted %lu cache pages\n", (*NrFreed));
// }
//
// NTSTATUS
// CcRosReleaseVacb (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     PROS_VACB Vacb,
//     BOOLEAN Dirty,
//     BOOLEAN Mapped)
// {
//     ULONG Refs;
//     ASSERT(SharedCacheMap);
//
//     DPRINT("CcRosReleaseVacb(SharedCacheMap 0x%p, Vacb 0x%p)\n", SharedCacheMap, Vacb);
//
//     if (Dirty && !Vacb->Dirty)
//     {
//         CcRosMarkDirtyVacb(Vacb);
//     }
//
//     if (Mapped)
//     {
//         if (InterlockedIncrement((PLONG)&Vacb->MappedCount) == 1)
//         {
//             CcRosVacbIncRefCount(Vacb);
//         }
//     }
//
//     Refs = CcRosVacbDecRefCount(Vacb);
//     ASSERT(Refs > 0);
//
//     return STATUS_SUCCESS;
// }
//
// /* Returns with VACB Lock Held! */
// PROS_VACB
// CcRosLookupVacb (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     LONGLONG FileOffset)
// {
//     PLIST_ENTRY current_entry;
//     PROS_VACB current;
//     KIRQL oldIrql;
//
//     ASSERT(SharedCacheMap);
//
//     DPRINT("CcRosLookupVacb(SharedCacheMap 0x%p, FileOffset %I64u)\n",
//            SharedCacheMap, FileOffset);
//
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//
//     current_entry = SharedCacheMap->CacheMapVacbListHead.Flink;
//     while (current_entry != &SharedCacheMap->CacheMapVacbListHead)
//     {
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     CacheMapVacbListEntry);
//         if (IsPointInRange(current->FileOffset.QuadPart,
//                            VACB_MAPPING_GRANULARITY,
//                            FileOffset))
//         {
//             CcRosVacbIncRefCount(current);
//             KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//             return current;
//         }
//         if (current->FileOffset.QuadPart > FileOffset)
//             break;
//         current_entry = current_entry->Flink;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//     return NULL;
// }
//
// VOID
// CcRosMarkDirtyVacb (
//     PROS_VACB Vacb)
// {
//     KIRQL oldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     SharedCacheMap = Vacb->SharedCacheMap;
//
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//
//     ASSERT(!Vacb->Dirty);
//
//     InsertTailList(&DirtyVacbListHead, &Vacb->DirtyVacbListEntry);
//     /* FIXME: There is no reason to account for the whole VACB. */
//     CcTotalDirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
//     Vacb->SharedCacheMap->DirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
//     CcRosVacbIncRefCount(Vacb);
//
//     /* Move to the tail of the LRU list */
//     RemoveEntryList(&Vacb->VacbLruListEntry);
//     InsertTailList(&VacbLruListHead, &Vacb->VacbLruListEntry);
//
//     Vacb->Dirty = TRUE;
//
//     KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//
//     /* Schedule a lazy writer run to now that we have dirty VACB */
//     if (!LazyWriter.ScanActive)
//     {
//         CcScheduleLazyWriteScan(FALSE);
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
// }
//
// VOID
// CcRosUnmarkDirtyVacb (
//     PROS_VACB Vacb,
//     BOOLEAN LockViews)
// {
//     KIRQL oldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     SharedCacheMap = Vacb->SharedCacheMap;
//
//     if (LockViews)
//     {
//         oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//         KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//     }
//
//     ASSERT(Vacb->Dirty);
//
//     Vacb->Dirty = FALSE;
//
//     RemoveEntryList(&Vacb->DirtyVacbListEntry);
//     InitializeListHead(&Vacb->DirtyVacbListEntry);
//
//     CcTotalDirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
//     Vacb->SharedCacheMap->DirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE as ULONG;
//
//     CcRosVacbDecRefCount(Vacb);
//
//     if (LockViews)
//     {
//         KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//     }
// }
//
// BOOLEAN
// CcRosFreeOneUnusedVacb(
//     VOID)
// {
//     KIRQL oldIrql;
//     PLIST_ENTRY current_entry;
//     PROS_VACB to_free = NULL;
//
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     /* Browse all the available VACB */
//     current_entry = VacbLruListHead.Flink;
//     while ((current_entry != &VacbLruListHead) && (to_free == NULL))
//     {
//         ULONG Refs;
//         PROS_VACB current;
//
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     VacbLruListEntry);
//
//         KeAcquireSpinLockAtDpcLevel(&current->SharedCacheMap->CacheMapLock);
//
//         /* Only deal with unused VACB, we will free them */
//         Refs = CcRosVacbGetRefCount(current);
//         if (Refs < 2)
//         {
//             ASSERT(!current->Dirty);
//             ASSERT(!current->MappedCount);
//             ASSERT(Refs == 1);
//
//             /* Reset it, this is the one we want to free */
//             RemoveEntryList(&current->CacheMapVacbListEntry);
//             InitializeListHead(&current->CacheMapVacbListEntry);
//             RemoveEntryList(&current->VacbLruListEntry);
//             InitializeListHead(&current->VacbLruListEntry);
//
//             to_free = current;
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&current->SharedCacheMap->CacheMapLock);
//
//         current_entry = current_entry->Flink;
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//     /* And now, free the VACB that we found, if any. */
//     if (to_free == NULL)
//     {
//         return FALSE;
//     }
//
//     /* This must be its last ref */
//     NT_VERIFY(CcRosVacbDecRefCount(to_free) == 0);
//
//     return TRUE;
// }
//
// static
// NTSTATUS
// CcRosCreateVacb (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     LONGLONG FileOffset,
//     PROS_VACB *Vacb)
// {
//     PROS_VACB current;
//     PROS_VACB previous;
//     PLIST_ENTRY current_entry;
//     NTSTATUS Status;
//     KIRQL oldIrql;
//     ULONG Refs;
//     SIZE_T ViewSize = VACB_MAPPING_GRANULARITY;
//
//     ASSERT(SharedCacheMap);
//
//     DPRINT("CcRosCreateVacb()\n");
//
//     ARM64_UART_TRACE("[CC] CreateVacb: alloc\r\n");
//     current = ExAllocateFromNPagedLookasideList(&VacbLookasideList);
//     if (!current)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     ARM64_UART_TRACE("[CC] CreateVacb: alloc done\r\n");
//     current->BaseAddress = NULL;
//     current->Dirty = FALSE;
//     current->PageOut = FALSE;
//     current->FileOffset.QuadPart = ROUND_DOWN(FileOffset, VACB_MAPPING_GRANULARITY);
//     current->SharedCacheMap = SharedCacheMap;
//     current->MappedCount = 0;
//     current->ReferenceCount = 0;
//     InitializeListHead(&current->CacheMapVacbListEntry);
//     InitializeListHead(&current->DirtyVacbListEntry);
//     InitializeListHead(&current->VacbLruListEntry);
//
//     CcRosVacbIncRefCount(current);
//
//
//
//     while (TRUE)
//     {
//         /* Map VACB in system space */
//         ARM64_UART_TRACE("[CC] CreateVacb: MmMapViewInSystemSpaceEx\r\n");
//         Status = MmMapViewInSystemSpaceEx(SharedCacheMap->Section, &current->BaseAddress, &ViewSize, &current->FileOffset, 0);
//         if (NT_SUCCESS(Status))
//         {
// #ifdef _M_ARM64
//             ARM64_BARRIER();
// #endif
//             ARM64_UART_TRACE("[CC] CreateVacb: MapView done\r\n");
//             break;
//         }
//
//         /*
//          * If no space left, try to prune one unused VACB to recover space to map our VACB.
//          * If it succeeds, retry to map, otherwise just fail.
//          */
//         if (!CcRosFreeOneUnusedVacb())
//         {
//             ExFreeToNPagedLookasideList(&VacbLookasideList, current);
//             return Status;
//         }
//     }
//
// #if DBG
//     if (SharedCacheMap->Trace)
//     {
//         DPRINT1("CacheMap 0x%p: new VACB: 0x%p, file offset %I64d, BaseAddress %p\n",
//                 SharedCacheMap, current, current->FileOffset.QuadPart, current->BaseAddress);
//     }
// #endif
//
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     *Vacb = current;
//     /* There is window between the call to CcRosLookupVacb
//      * and CcRosCreateVacb. We must check if a VACB for the
//      * file offset exist. If there is a VACB, we release
//      * our newly created VACB and return the existing one.
//      */
//     KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//     current_entry = SharedCacheMap->CacheMapVacbListHead.Flink;
//     previous = NULL;
//     while (current_entry != &SharedCacheMap->CacheMapVacbListHead)
//     {
//         current = CONTAINING_RECORD(current_entry,
//                                     ROS_VACB,
//                                     CacheMapVacbListEntry);
//         if (IsPointInRange(current->FileOffset.QuadPart,
//                            VACB_MAPPING_GRANULARITY,
//                            FileOffset))
//         {
//             CcRosVacbIncRefCount(current);
//             KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
// #if DBG
//             if (SharedCacheMap->Trace)
//             {
//                 DPRINT1("CacheMap 0x%p: deleting newly created VACB 0x%p ( found existing one 0x%p )\n",
//                         SharedCacheMap,
//                         (*Vacb),
//                         current);
//             }
// #endif
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//             Refs = CcRosVacbDecRefCount(*Vacb);
//             ASSERT(Refs == 0);
//
//             *Vacb = current;
//             return STATUS_SUCCESS;
//         }
//         if (current->FileOffset.QuadPart < FileOffset)
//         {
//             ASSERT(previous == NULL ||
//                    previous->FileOffset.QuadPart < current->FileOffset.QuadPart);
//             previous = current;
//         }
//         if (current->FileOffset.QuadPart > FileOffset)
//             break;
//         current_entry = current_entry->Flink;
//     }
//     /* There was no existing VACB. */
//     current = *Vacb;
//     if (previous)
//     {
//         InsertHeadList(&previous->CacheMapVacbListEntry, &current->CacheMapVacbListEntry);
//     }
//     else
//     {
//         InsertHeadList(&SharedCacheMap->CacheMapVacbListHead, &current->CacheMapVacbListEntry);
//     }
//     KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//     InsertTailList(&VacbLruListHead, &current->VacbLruListEntry);
//
//     /* Reference it to allow release */
//     CcRosVacbIncRefCount(current);
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//     return Status;
// }
//
// BOOLEAN
// CcRosEnsureVacbResident(
//     _In_ PROS_VACB Vacb,
//     _In_ BOOLEAN Wait,
//     _In_ BOOLEAN NoRead,
//     _In_ ULONG Offset,
//     _In_ ULONG Length
// )
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap = Vacb->SharedCacheMap;
//
//     ASSERT((Offset + Length) <= VACB_MAPPING_GRANULARITY);
//
// #if 0
//     if ((Vacb->FileOffset.QuadPart + Offset) > SharedCacheMap->SectionSize.QuadPart)
//     {
//         DPRINT1("Vacb read beyond the file size!\n");
//         return FALSE;
//     }
// #endif
//
//     /* Check if the pages are resident */
//     ARM64_UART_TRACE("[CC] EnsureResident: MmIsDataSectionResident\r\n");
//     if (!MmIsDataSectionResident(SharedCacheMap->FileObject->SectionObjectPointer,
//                                  Vacb->FileOffset.QuadPart + Offset,
//                                  Length))
//     {
//         ARM64_UART_TRACE("[CC] EnsureResident: pages NOT resident\r\n");
//         if (!Wait)
//         {
//             return FALSE;
//         }
//
//         if (!NoRead)
//         {
// #ifdef _M_ARM64
//             ARM64_BARRIER();
// #endif
//             ARM64_UART_TRACE("[CC] EnsureResident: MmMakeDataSectionResident\r\n");
//             NTSTATUS Status = MmMakeDataSectionResident(SharedCacheMap->FileObject->SectionObjectPointer,
//                                                         Vacb->FileOffset.QuadPart + Offset,
//                                                         Length,
//                                                         &SharedCacheMap->ValidDataLength);
//             if (!NT_SUCCESS(Status))
//                 ExRaiseStatus(Status);
//         }
//     }
//
//     return TRUE;
// }
//
//
// NTSTATUS
// CcRosGetVacb (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     LONGLONG FileOffset,
//     PROS_VACB *Vacb)
// {
//     PROS_VACB current;
//     NTSTATUS Status;
//     ULONG Refs;
//     KIRQL OldIrql;
//
//     ASSERT(SharedCacheMap);
//
//     DPRINT("CcRosGetVacb()\n");
//
//     /*
//      * Look for a VACB already mapping the same data.
//      */
//     current = CcRosLookupVacb(SharedCacheMap, FileOffset);
//     if (current == NULL)
//     {
//         /*
//          * Otherwise create a new VACB.
//          */
//         Status = CcRosCreateVacb(SharedCacheMap, FileOffset, &current);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     Refs = CcRosVacbGetRefCount(current);
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     /* Move to the tail of the LRU list */
//     RemoveEntryList(&current->VacbLruListEntry);
//     InsertTailList(&VacbLruListHead, &current->VacbLruListEntry);
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /*
//      * Return the VACB to the caller.
//      */
//     *Vacb = current;
//
//     ASSERT(Refs > 1);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CcRosRequestVacb (
//     PROS_SHARED_CACHE_MAP SharedCacheMap,
//     LONGLONG FileOffset,
//     PROS_VACB *Vacb)
// /*
//  * FUNCTION: Request a page mapping for a shared cache map
//  */
// {
//
//     ASSERT(SharedCacheMap);
//
//     if (FileOffset % VACB_MAPPING_GRANULARITY != 0)
//     {
//         DPRINT1("Bad FileOffset %I64x: should be multiple of %x\n",
//                 FileOffset, VACB_MAPPING_GRANULARITY);
//         KeBugCheck(CACHE_MANAGER);
//     }
//
//     return CcRosGetVacb(SharedCacheMap,
//                         FileOffset,
//                         Vacb);
// }
//
// NTSTATUS
// CcRosInternalFreeVacb (
//     PROS_VACB Vacb)
// /*
//  * FUNCTION: Releases a VACB associated with a shared cache map
//  */
// {
//     NTSTATUS Status;
//
//     DPRINT("Freeing VACB 0x%p\n", Vacb);
// #if DBG
//     if (Vacb->SharedCacheMap->Trace)
//     {
//         DPRINT1("CacheMap 0x%p: deleting VACB: 0x%p\n", Vacb->SharedCacheMap, Vacb);
//     }
// #endif
//
//     if (Vacb->ReferenceCount != 0)
//     {
//         DPRINT1("Invalid free: %ld\n", Vacb->ReferenceCount);
//         if (Vacb->SharedCacheMap->FileObject && Vacb->SharedCacheMap->FileObject->FileName.Length)
//         {
//             DPRINT1("For file: %wZ\n", &Vacb->SharedCacheMap->FileObject->FileName);
//         }
//     }
//
//     ASSERT(Vacb->ReferenceCount == 0);
//     ASSERT(IsListEmpty(&Vacb->CacheMapVacbListEntry));
//     ASSERT(IsListEmpty(&Vacb->DirtyVacbListEntry));
//     ASSERT(IsListEmpty(&Vacb->VacbLruListEntry));
//
//     /* Delete the mapping */
//     Status = MmUnmapViewInSystemSpace(Vacb->BaseAddress);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to unmap VACB from System address space! Status 0x%08X\n", Status);
//         ASSERT(FALSE);
//         /* Proceed with the deĺetion anyway */
//     }
//
//     RtlFillMemory(Vacb, sizeof(*Vacb), 0xfd);
//     ExFreeToNPagedLookasideList(&VacbLookasideList, Vacb);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcFlushCache (
//     IN PSECTION_OBJECT_POINTERS SectionObjectPointers,
//     IN PLARGE_INTEGER FileOffset OPTIONAL,
//     IN ULONG Length,
//     OUT PIO_STATUS_BLOCK IoStatus)
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     LONGLONG FlushStart, FlushEnd;
//     NTSTATUS Status;
//
//     CCTRACE(CC_API_DEBUG, "SectionObjectPointers=%p FileOffset=0x%I64X Length=%lu\n",
//         SectionObjectPointers, FileOffset ? FileOffset->QuadPart : 0LL, Length);
//
//     if (!SectionObjectPointers)
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto quit;
//     }
//
//     if (!SectionObjectPointers->SharedCacheMap)
//     {
//         /* Forward this to Mm */
//         MmFlushSegment(SectionObjectPointers, FileOffset, Length, IoStatus);
//         return;
//     }
//
//     SharedCacheMap = SectionObjectPointers->SharedCacheMap;
//     ASSERT(SharedCacheMap);
//     if (FileOffset)
//     {
//         FlushStart = FileOffset->QuadPart;
//         Status = RtlLongLongAdd(FlushStart, Length, &FlushEnd);
//         if (!NT_SUCCESS(Status))
//             goto quit;
//     }
//     else
//     {
//         FlushStart = 0;
//         FlushEnd = SharedCacheMap->FileSize.QuadPart;
//     }
//
//     Status = STATUS_SUCCESS;
//     if (IoStatus)
//     {
//         IoStatus->Information = 0;
//     }
//
//     KeAcquireGuardedMutex(&SharedCacheMap->FlushCacheLock);
//
//     /*
//      * We flush the VACBs that we find here.
//      * If there is no (dirty) VACB, it doesn't mean that there is no data to flush, so we call Mm to be sure.
//      * This is suboptimal, but this is due to the lack of granularity of how we track dirty cache data
//      */
//     while (FlushStart < FlushEnd)
//     {
//         BOOLEAN DirtyVacb = FALSE;
//         PROS_VACB vacb = CcRosLookupVacb(SharedCacheMap, FlushStart);
//
//         if (vacb != NULL)
//         {
//             if (vacb->Dirty)
//             {
//                 IO_STATUS_BLOCK VacbIosb = { 0 };
//                 Status = CcRosFlushVacb(vacb, &VacbIosb);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     CcRosReleaseVacb(SharedCacheMap, vacb, FALSE, FALSE);
//                     break;
//                 }
//                 DirtyVacb = TRUE;
//
//                 if (IoStatus)
//                     IoStatus->Information += VacbIosb.Information;
//             }
//
//             CcRosReleaseVacb(SharedCacheMap, vacb, FALSE, FALSE);
//         }
//
//         if (!DirtyVacb)
//         {
//             IO_STATUS_BLOCK MmIosb;
//             LARGE_INTEGER MmOffset;
//
//             MmOffset.QuadPart = FlushStart;
//
//             if (FlushEnd - (FlushEnd % VACB_MAPPING_GRANULARITY) <= FlushStart)
//             {
//                 /* The whole range fits within a VACB chunk. */
//                 Status = MmFlushSegment(SectionObjectPointers, &MmOffset, FlushEnd - FlushStart, &MmIosb);
//             }
//             else
//             {
//                 ULONG MmLength = VACB_MAPPING_GRANULARITY - (FlushStart % VACB_MAPPING_GRANULARITY);
//                 Status = MmFlushSegment(SectionObjectPointers, &MmOffset, MmLength, &MmIosb);
//             }
//
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             if (IoStatus)
//                 IoStatus->Information += MmIosb.Information;
//
//             /* Update VDL */
//             if (SharedCacheMap->ValidDataLength.QuadPart < FlushEnd)
//                 SharedCacheMap->ValidDataLength.QuadPart = FlushEnd;
//         }
//
//         if (!NT_SUCCESS(RtlLongLongAdd(FlushStart, VACB_MAPPING_GRANULARITY, &FlushStart)))
//         {
//             /* We're at the end of file ! */
//             break;
//         }
//
//         /* Round down to next VACB start now */
//         FlushStart -= FlushStart % VACB_MAPPING_GRANULARITY;
//     }
//
//     KeReleaseGuardedMutex(&SharedCacheMap->FlushCacheLock);
//
// quit:
//     if (IoStatus)
//     {
//         IoStatus->Status = Status;
//     }
// }
//
// NTSTATUS
// CcRosReleaseFileCache (
//     PFILE_OBJECT FileObject)
// /*
//  * FUNCTION: Called by the file system when a handle to a file object
//  * has been closed.
//  */
// {
//     KIRQL OldIrql;
//     PPRIVATE_CACHE_MAP PrivateMap;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     if (FileObject->SectionObjectPointer->SharedCacheMap != NULL)
//     {
//         SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//         /* Closing the handle, so kill the private cache map
//          * Before you event try to remove it from FO, always
//          * lock the master lock, to be sure not to race
//          * with a potential read ahead ongoing!
//          */
//         PrivateMap = FileObject->PrivateCacheMap;
//         FileObject->PrivateCacheMap = NULL;
//
//         if (PrivateMap != NULL)
//         {
//             /* Remove it from the file */
//             KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//             RemoveEntryList(&PrivateMap->PrivateLinks);
//             KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//
//             /* And free it. */
//             if (PrivateMap != &SharedCacheMap->PrivateCacheMap)
//             {
//                 ExFreePoolWithTag(PrivateMap, TAG_PRIVATE_CACHE_MAP);
//             }
//             else
//             {
//                 PrivateMap->NodeTypeCode = 0;
//             }
//
//             ASSERT(SharedCacheMap->OpenCount > 0);
//
//             SharedCacheMap->OpenCount--;
//             if (SharedCacheMap->OpenCount == 0)
//             {
//                 CcRosDeleteFileCache(FileObject, SharedCacheMap, &OldIrql);
//             }
//         }
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CcRosInitializeFileCache (
//     PFILE_OBJECT FileObject,
//     PCC_FILE_SIZES FileSizes,
//     BOOLEAN PinAccess,
//     PCACHE_MANAGER_CALLBACKS CallBacks,
//     PVOID LazyWriterContext)
// /*
//  * FUNCTION: Initializes a shared cache map for a file object
//  */
// {
//     KIRQL OldIrql;
//     BOOLEAN Allocated;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     DPRINT("CcRosInitializeFileCache(FileObject 0x%p)\n", FileObject);
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     Allocated = FALSE;
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     if (SharedCacheMap == NULL)
//     {
//         SharedCacheMap = ExAllocateFromNPagedLookasideList(&SharedCacheMapLookasideList);
//         if (SharedCacheMap == NULL)
//         {
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//         RtlZeroMemory(SharedCacheMap, sizeof(*SharedCacheMap));
//         SharedCacheMap->NodeTypeCode = NODE_TYPE_SHARED_MAP;
//         SharedCacheMap->NodeByteSize = sizeof(*SharedCacheMap);
//         SharedCacheMap->FileObject = FileObject;
//         SharedCacheMap->Callbacks = CallBacks;
//         SharedCacheMap->LazyWriteContext = LazyWriterContext;
//         SharedCacheMap->SectionSize = FileSizes->AllocationSize;
//         SharedCacheMap->FileSize = FileSizes->FileSize;
//         SharedCacheMap->ValidDataLength = FileSizes->ValidDataLength;
//         SharedCacheMap->PinAccess = PinAccess;
//         SharedCacheMap->DirtyPageThreshold = 0;
//         SharedCacheMap->DirtyPages = 0;
//         InitializeListHead(&SharedCacheMap->PrivateList);
//         KeInitializeSpinLock(&SharedCacheMap->CacheMapLock);
//         InitializeListHead(&SharedCacheMap->CacheMapVacbListHead);
//         InitializeListHead(&SharedCacheMap->BcbList);
//         KeInitializeGuardedMutex(&SharedCacheMap->FlushCacheLock);
//
//         SharedCacheMap->Flags = SHARED_CACHE_MAP_IN_CREATION;
//
//         ObReferenceObjectByPointer(FileObject,
//                                    FILE_ALL_ACCESS,
//                                    NULL,
//                                    KernelMode);
//
//         Allocated = TRUE;
//         FileObject->SectionObjectPointer->SharedCacheMap = SharedCacheMap;
//
//         //CcRosTraceCacheMap(SharedCacheMap, TRUE);
//     }
//     else if (SharedCacheMap->Flags & SHARED_CACHE_MAP_IN_CREATION)
//     {
//         /* The shared cache map is being created somewhere else. Wait for that to happen */
//         KEVENT Waiter;
//         PKEVENT PreviousWaiter = SharedCacheMap->CreateEvent;
//
//         KeInitializeEvent(&Waiter, NotificationEvent, FALSE);
//         SharedCacheMap->CreateEvent = &Waiter;
//
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//         KeWaitForSingleObject(&Waiter, Executive, KernelMode, FALSE, NULL);
//
//         if (PreviousWaiter)
//             KeSetEvent(PreviousWaiter, IO_NO_INCREMENT, FALSE);
//
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     }
//
//     if (FileObject->PrivateCacheMap == NULL)
//     {
//         PPRIVATE_CACHE_MAP PrivateMap;
//
//         /* Allocate the private cache map for this handle */
//         if (SharedCacheMap->PrivateCacheMap.NodeTypeCode != 0)
//         {
//             PrivateMap = ExAllocatePoolWithTag(NonPagedPool, sizeof(PRIVATE_CACHE_MAP), TAG_PRIVATE_CACHE_MAP);
//         }
//         else
//         {
//             PrivateMap = &SharedCacheMap->PrivateCacheMap;
//         }
//
//         if (PrivateMap == NULL)
//         {
//             /* If we also allocated the shared cache map for this file, kill it */
//             if (Allocated)
//             {
//                 RemoveEntryList(&SharedCacheMap->SharedCacheMapLinks);
//
//                 FileObject->SectionObjectPointer->SharedCacheMap = NULL;
//                 ObDereferenceObject(FileObject);
//                 ExFreeToNPagedLookasideList(&SharedCacheMapLookasideList, SharedCacheMap);
//             }
//
//             KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Initialize it */
//         RtlZeroMemory(PrivateMap, sizeof(PRIVATE_CACHE_MAP));
//         PrivateMap->NodeTypeCode = NODE_TYPE_PRIVATE_MAP;
//         PrivateMap->ReadAheadMask = PAGE_SIZE - 1;
//         PrivateMap->FileObject = FileObject;
//         KeInitializeSpinLock(&PrivateMap->ReadAheadSpinLock);
//
//         /* Link it to the file */
//         KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//         InsertTailList(&SharedCacheMap->PrivateList, &PrivateMap->PrivateLinks);
//         KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//
//         FileObject->PrivateCacheMap = PrivateMap;
//         SharedCacheMap->OpenCount++;
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     /* Create the section */
//     if (Allocated)
//     {
//         NTSTATUS Status;
//
//         ASSERT(SharedCacheMap->Section == NULL);
//
//         Status = MmCreateSection(
//             &SharedCacheMap->Section,
//             SECTION_ALL_ACCESS,
//             NULL,
//             &SharedCacheMap->SectionSize,
//             PAGE_READWRITE,
//             SEC_RESERVE,
//             NULL,
//             FileObject);
//
//         ASSERT(NT_SUCCESS(Status));
//
//         if (!NT_SUCCESS(Status))
//         {
//             CcRosReleaseFileCache(FileObject);
//             return Status;
//         }
//
//         OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//         InsertTailList(&CcCleanSharedCacheMapList, &SharedCacheMap->SharedCacheMapLinks);
//         SharedCacheMap->Flags &= ~SHARED_CACHE_MAP_IN_CREATION;
//
//         if (SharedCacheMap->CreateEvent)
//         {
//             KeSetEvent(SharedCacheMap->CreateEvent, IO_NO_INCREMENT, FALSE);
//             SharedCacheMap->CreateEvent = NULL;
//         }
//
//         KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PFILE_OBJECT
// NTAPI
// CcGetFileObjectFromSectionPtrs (
//     IN PSECTION_OBJECT_POINTERS SectionObjectPointers)
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "SectionObjectPointers=%p\n", SectionObjectPointers);
//
//     if (SectionObjectPointers && SectionObjectPointers->SharedCacheMap)
//     {
//         SharedCacheMap = SectionObjectPointers->SharedCacheMap;
//         ASSERT(SharedCacheMap);
//         return SharedCacheMap->FileObject;
//     }
//     return NULL;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CcInitView (
//     VOID)
// {
//     DPRINT("CcInitView()\n");
//
//     InitializeListHead(&DirtyVacbListHead);
//     InitializeListHead(&VacbLruListHead);
//     InitializeListHead(&CcDeferredWrites);
//     InitializeListHead(&CcCleanSharedCacheMapList);
//     KeInitializeSpinLock(&CcDeferredWriteSpinLock);
//     ExInitializeNPagedLookasideList(&iBcbLookasideList,
//                                     NULL,
//                                     NULL,
//                                     0,
//                                     sizeof(INTERNAL_BCB),
//                                     TAG_BCB,
//                                     20);
//     ExInitializeNPagedLookasideList(&SharedCacheMapLookasideList,
//                                     NULL,
//                                     NULL,
//                                     0,
//                                     sizeof(ROS_SHARED_CACHE_MAP),
//                                     TAG_SHARED_CACHE_MAP,
//                                     20);
//     ExInitializeNPagedLookasideList(&VacbLookasideList,
//                                     NULL,
//                                     NULL,
//                                     0,
//                                     sizeof(ROS_VACB),
//                                     TAG_VACB,
//                                     20);
//
//     CcInitCacheZeroPage();
// }
//
// #if DBG && defined(KDBG)
//
// #include <kdbg/kdb.h>
//
// BOOLEAN
// ExpKdbgExtFileCache(ULONG Argc, PCHAR Argv[])
// {
//     PLIST_ENTRY ListEntry;
//     UNICODE_STRING NoName = RTL_CONSTANT_STRING(L"No name for File");
//
//     KdbpPrint("  Usage Summary (in kb)\n");
//     KdbpPrint("Shared\t\tMapped\tDirty\tName\n");
//     /* No need to lock the spin lock here, we're in DBG */
//     for (ListEntry = CcCleanSharedCacheMapList.Flink;
//          ListEntry != &CcCleanSharedCacheMapList;
//          ListEntry = ListEntry->Flink)
//     {
//         PLIST_ENTRY Vacbs;
//         ULONG Mapped = 0, Dirty = 0;
//         PROS_SHARED_CACHE_MAP SharedCacheMap;
//         PUNICODE_STRING FileName;
//         PWSTR Extra = L"";
//
//         SharedCacheMap = CONTAINING_RECORD(ListEntry, ROS_SHARED_CACHE_MAP, SharedCacheMapLinks);
//
//         /* Dirty size */
//         Dirty = (SharedCacheMap->DirtyPages * PAGE_SIZE) / 1024;
//
//         /* First, count for all the associated VACB */
//         for (Vacbs = SharedCacheMap->CacheMapVacbListHead.Flink;
//              Vacbs != &SharedCacheMap->CacheMapVacbListHead;
//              Vacbs = Vacbs->Flink)
//         {
//             Mapped += VACB_MAPPING_GRANULARITY / 1024;
//         }
//
//         /* Setup name */
//         if (SharedCacheMap->FileObject != NULL &&
//             SharedCacheMap->FileObject->FileName.Length != 0)
//         {
//             FileName = &SharedCacheMap->FileObject->FileName;
//         }
//         else if (SharedCacheMap->FileObject != NULL &&
//                  SharedCacheMap->FileObject->FsContext != NULL &&
//                  ((PFSRTL_COMMON_FCB_HEADER)(SharedCacheMap->FileObject->FsContext))->NodeTypeCode == 0x0502 &&
//                  ((PFSRTL_COMMON_FCB_HEADER)(SharedCacheMap->FileObject->FsContext))->NodeByteSize == 0x1F8 &&
//                  ((PUNICODE_STRING)(((PUCHAR)SharedCacheMap->FileObject->FsContext) + 0x100))->Length != 0)
//         {
//             FileName = (PUNICODE_STRING)(((PUCHAR)SharedCacheMap->FileObject->FsContext) + 0x100);
//             Extra = L" (FastFAT)";
//         }
//         else
//         {
//             FileName = &NoName;
//         }
//
//         /* And print */
//         KdbpPrint("%p\t%d\t%d\t%wZ%S\n", SharedCacheMap, Mapped, Dirty, FileName, Extra);
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// ExpKdbgExtDefWrites(ULONG Argc, PCHAR Argv[])
// {
//     KdbpPrint("CcTotalDirtyPages:\t%lu (%lu Kb)\n", CcTotalDirtyPages,
//               (CcTotalDirtyPages * PAGE_SIZE) / 1024);
//     KdbpPrint("CcDirtyPageThreshold:\t%lu (%lu Kb)\n", CcDirtyPageThreshold,
//               (CcDirtyPageThreshold * PAGE_SIZE) / 1024);
//     KdbpPrint("MmAvailablePages:\t%lu (%lu Kb)\n", MmAvailablePages,
//               (MmAvailablePages * PAGE_SIZE) / 1024);
//     KdbpPrint("MmThrottleTop:\t\t%lu (%lu Kb)\n", MmThrottleTop,
//               (MmThrottleTop * PAGE_SIZE) / 1024);
//     KdbpPrint("MmThrottleBottom:\t%lu (%lu Kb)\n", MmThrottleBottom,
//               (MmThrottleBottom * PAGE_SIZE) / 1024);
//     KdbpPrint("MmModifiedPageListHead.Total:\t%lu (%lu Kb)\n", MmModifiedPageListHead.Total,
//               (MmModifiedPageListHead.Total * PAGE_SIZE) / 1024);
//
//     if (CcTotalDirtyPages >= CcDirtyPageThreshold)
//     {
//         KdbpPrint("CcTotalDirtyPages above the threshold, writes should be throttled\n");
//     }
//     else if (CcTotalDirtyPages + 64 >= CcDirtyPageThreshold)
//     {
//         KdbpPrint("CcTotalDirtyPages within 64 (max charge) pages of the threshold, writes may be throttled\n");
//     }
//     else
//     {
//         KdbpPrint("CcTotalDirtyPages below the threshold, writes should not be throttled\n");
//     }
//
//     return TRUE;
// }
//
// #endif // DBG && defined(KDBG)
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<ROS_SHARED_CACHE_MAP>() == 352);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, NodeTypeCode) == 0);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, NodeByteSize) == 2);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, OpenCount) == 4);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FileSize) == 8);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, BcbList) == 16);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, SectionSize) == 32);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, ValidDataLength) == 40);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FileObject) == 48);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, DirtyPages) == 56);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, SharedCacheMapLinks) == 64);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Flags) == 80);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Section) == 88);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CreateEvent) == 96);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Callbacks) == 104);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, LazyWriteContext) == 112);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PrivateList) == 120);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, DirtyPageThreshold) == 136);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, BcbSpinLock) == 144);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PrivateCacheMap) == 152);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CacheMapVacbListHead) == 256);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PinAccess) == 272);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CacheMapLock) == 280);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FlushCacheLock) == 288);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Trace) == 344);
    assert!(core::mem::size_of::<ROS_VACB>() == 88);
    assert!(core::mem::offset_of!(ROS_VACB, BaseAddress) == 0);
    assert!(core::mem::offset_of!(ROS_VACB, Dirty) == 8);
    assert!(core::mem::offset_of!(ROS_VACB, PageOut) == 9);
    assert!(core::mem::offset_of!(ROS_VACB, MappedCount) == 12);
    assert!(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) == 16);
    assert!(core::mem::offset_of!(ROS_VACB, DirtyVacbListEntry) == 32);
    assert!(core::mem::offset_of!(ROS_VACB, VacbLruListEntry) == 48);
    assert!(core::mem::offset_of!(ROS_VACB, FileOffset) == 64);
    assert!(core::mem::offset_of!(ROS_VACB, ReferenceCount) == 72);
    assert!(core::mem::offset_of!(ROS_VACB, SharedCacheMap) == 80);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<ROS_SHARED_CACHE_MAP>() == 240);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, NodeTypeCode) == 0);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, NodeByteSize) == 2);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, OpenCount) == 4);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FileSize) == 8);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, BcbList) == 16);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, SectionSize) == 24);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, ValidDataLength) == 32);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FileObject) == 40);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, DirtyPages) == 44);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, SharedCacheMapLinks) == 48);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Flags) == 56);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Section) == 60);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CreateEvent) == 64);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Callbacks) == 68);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, LazyWriteContext) == 72);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PrivateList) == 76);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, DirtyPageThreshold) == 84);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, BcbSpinLock) == 88);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PrivateCacheMap) == 96);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CacheMapVacbListHead) == 184);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, PinAccess) == 192);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, CacheMapLock) == 196);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, FlushCacheLock) == 200);
    assert!(core::mem::offset_of!(ROS_SHARED_CACHE_MAP, Trace) == 232);
    assert!(core::mem::size_of::<ROS_VACB>() == 56);
    assert!(core::mem::offset_of!(ROS_VACB, BaseAddress) == 0);
    assert!(core::mem::offset_of!(ROS_VACB, Dirty) == 4);
    assert!(core::mem::offset_of!(ROS_VACB, PageOut) == 5);
    assert!(core::mem::offset_of!(ROS_VACB, MappedCount) == 8);
    assert!(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) == 12);
    assert!(core::mem::offset_of!(ROS_VACB, DirtyVacbListEntry) == 20);
    assert!(core::mem::offset_of!(ROS_VACB, VacbLruListEntry) == 28);
    assert!(core::mem::offset_of!(ROS_VACB, FileOffset) == 40);
    assert!(core::mem::offset_of!(ROS_VACB, ReferenceCount) == 48);
    assert!(core::mem::offset_of!(ROS_VACB, SharedCacheMap) == 52);
};
// END OFFSET CHECKS
