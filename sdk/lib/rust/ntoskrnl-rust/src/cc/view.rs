use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcInitCacheZeroPage, CcRosEnsureVacbResident, CcRosFlushVacb, CcRosFreeOneUnusedVacb, CcRosMarkDirtyVacb, CcRosUnmarkDirtyVacb, CcScheduleLazyWriteScan, ExInitializeNPagedLookasideList, INTERNAL_BCB, _IO_STATUS_BLOCK, IoGetTopLevelIrp, KEVENT, KeAcquireSpinLockAtDpcLevel, KeBugCheck, KeEnterCriticalRegion, KeInitializeEvent, KeInitializeSpinLock, KeLeaveCriticalRegion, KeReleaseSpinLockFromDpcLevel, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, MmCreateSection, MmFlushSegment, MmGetPhysicalAddress, MmIsDataSectionResident, MmMakeDataSectionResident, MmMapViewInSystemSpaceEx, MmPageOutPhysicalAddress, MmUnmapViewInSystemSpace, NPAGED_LOOKASIDE_LIST, ObReferenceObjectByPointer, PCACHE_MANAGER_CALLBACKS, PCC_FILE_SIZES, PCSTR, PFILE_OBJECT, PFSRTL_COMMON_FCB_HEADER, PIO_STATUS_BLOCK, PIRP, PKEVENT, PKIRQL, PLARGE_INTEGER, PPRIVATE_CACHE_MAP, PRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP, PROS_VACB, PSECTION_OBJECT_POINTERS, PUNICODE_STRING, ROS_SHARED_CACHE_MAP, ROS_VACB};
use core::ffi::c_void;

extern "C" {
    fn CcRosVacbIncRefCount(current: PROS_VACB);
    fn IsPointInRange(start: i64, size: i64, point: i64) -> i32;
    fn KdbpPrint(Format: *const u8, ...);
    fn ExFreeToNPagedLookasideList(Lookaside: *mut core::ffi::c_void, Entry: *mut core::ffi::c_void);
    static mut SharedCacheMapLookasideList: core::ffi::c_void;
    fn DbgPrint(format: *const i8, ...) -> i32;
    fn ExAllocateFromNPagedLookasideList(Lookaside: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    static mut CcCleanSharedCacheMapList: _LIST_ENTRY;
    static mut 17i32: i32;
}
extern "C" {
    fn CcRosVacbDecRefCount(Vacb: PROS_VACB) -> ULONG;
    fn CcRosVacbGetRefCount(Vacb: PROS_VACB) -> ULONG;
    fn NT_VERIFY(Condition: BOOLEAN) -> BOOLEAN;
    static mut VacbLookasideList: core::ffi::c_void;
    fn ROUND_DOWN(Value: i64, Alignment: u32) -> i64;
    fn ARM64_BARRIER();
    fn ARM64_UART_TRACE(Format: *const u8);
    static mut VacbLruListHead: _LIST_ENTRY;
    fn BooleanFlagOn(F: u32, On: u32) -> u8;
}

static mut CcTotalDirtyPages: u32 = 0;
static mut CcDirtyPageThreshold: u32 = 0;
static mut MmThrottleTop: u32 = 0;
static mut MmThrottleBottom: u32 = 0;

#[repr(C)]
struct _MM_PAGE_LIST {
    Total: u32,
}

static mut MmModifiedPageListHead: _MM_PAGE_LIST = _MM_PAGE_LIST { Total: 0 };

#[no_mangle]
pub unsafe extern "system" fn CcRosVacbIncRefCount_(vacb: PROS_VACB, file: PCSTR, line: i32) -> u32 {
    let Refs = InterlockedIncrement(&mut (*vacb).ReferenceCount as *mut _ as *mut i32);
    if (*(*vacb).SharedCacheMap).Trace != 0 {
        DbgPrint(
            b"(%s:%i) VACB %p ++RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as PCSTR,
            file,
            line,
            vacb,
            Refs,
            (*vacb).Dirty,
            (*vacb).PageOut,
        );
    }

    Refs as u32
}

#[no_mangle]
pub unsafe extern "system" fn CcRosVacbDecRefCount_(vacb: PROS_VACB, file: PCSTR, line: i32) -> u32 {
    let VacbDirty: u8 = (*vacb).Dirty;
    let VacbTrace: u8 = (*(*vacb).SharedCacheMap).Trace;
    let VacbPageOut: u32 = (*vacb).PageOut;

    let Refs: u32 = InterlockedDecrement(core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut i32) as u32;
    
    if VacbTrace != 0 {
        DbgPrint(b"(%s:%i) VACB %p --RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as *const i8,
                 file, line, vacb, Refs, VacbDirty, VacbPageOut);
    }

    if Refs == 0 {
        CcRosInternalFreeVacb(vacb);
    }

    Refs
}

#[no_mangle]
pub unsafe extern "C" fn CcRosVacbGetRefCount_(
    vacb: PROS_VACB,
    file: PCSTR,
    line: i32,
) -> u32 {
    let Refs: u32;
    
    let mut current = 0i32;
    Refs = core::arch::x86_64::_InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*vacb).ReferenceCount) as *mut i32,
        0,
        0,
    ) as u32;
    
    if (*(*vacb).SharedCacheMap).Trace != 0 {
        DbgPrint(
            b"(%s:%i) VACB %p ==RefCount=%lu, Dirty %u, PageOut %lu\n\0".as_ptr() as *const u8,
            file,
            line,
            vacb,
            Refs,
            (*vacb).Dirty,
            (*vacb).PageOut,
        );
    }
    
    Refs
}

unsafe fn CcRosTraceCacheMap(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    Trace: u8,
) {
    #[cfg(debug_assertions)]
    {
        if SharedCacheMap.is_null() {
            return;
        }

        (*SharedCacheMap).Trace = Trace;

        if Trace != 0 {
            let oldirql = KeAcquireQueuedSpinLock(0i32);
            KeAcquireSpinLockAtDpcLevel(&mut (*SharedCacheMap).CacheMapLock);

            let mut current_entry = (*SharedCacheMap).CacheMapVacbListHead.Flink;
            while current_entry != core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
                let current = (current_entry as *mut u8).offset(-(core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
                current_entry = (*current_entry).Flink;
            }

            KeReleaseSpinLockFromDpcLevel(&mut (*SharedCacheMap).CacheMapLock);
            KeReleaseQueuedSpinLock(0i32, oldirql);
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = SharedCacheMap;
        let _ = Trace;
    }
}

unsafe fn CcRosFlushVacb(Vacb: PROS_VACB, Iosb: PIO_STATUS_BLOCK) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut HaveLock: u8 = 0u8;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP = (*Vacb).SharedCacheMap;

    CcRosUnmarkDirtyVacb(Vacb, 1u8);

    if IoGetTopLevelIrp() != 0xFFFFFFFF as PIRP {
        Status = FsRtlAcquireFileForCcFlushEx((*SharedCacheMap).FileObject);
        if NT_SUCCESS(Status) == 0 {
            return Status;
        }
        HaveLock = 1u8;
    } else {
        Status = 0i32;
    }

    Status = MmFlushSegment(
        (*(*SharedCacheMap).FileObject).SectionObjectPointer,
        core::ptr::addr_of_mut!((*Vacb).FileOffset),
        0x1000u32,
        Iosb,
    );

    if HaveLock != 0 {
        FsRtlReleaseFileForCcFlush((*SharedCacheMap).FileObject);
    }

    if NT_SUCCESS(Status) == 0 {
        CcRosMarkDirtyVacb(Vacb);
    } else {
        if (*SharedCacheMap).ValidDataLength.QuadPart
            < ((*Vacb).FileOffset.QuadPart + 0x1000i64)
        {
            (*SharedCacheMap).ValidDataLength.QuadPart =
                (*Vacb).FileOffset.QuadPart + 0x1000i64;
        }
    }

    Status
}

unsafe fn CcRosDeleteFileCache(
    FileObject: PFILE_OBJECT,
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    OldIrql: PKIRQL,
) -> i32 {
    let mut current_entry: PLIST_ENTRY;
    let mut Vacb: PROS_VACB;
    let mut RefCount: u32;
    let mut Iosb: _IO_STATUS_BLOCK;
    let mut Status: i32;

    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    current_entry = (*SharedCacheMap).CacheMapVacbListHead.Flink;
    while current_entry != core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
        Vacb = (current_entry as *mut u8).offset(-(core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;

        RemoveEntryList(core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry));
        InitializeListHead(core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry));

        if (*Vacb).Dirty != 0 {
            CcRosUnmarkDirtyVacb(Vacb, 0u8);
            (*Vacb).Dirty = 1u8;
        }

        current_entry = (*current_entry).Flink;
    }

    (*(*FileObject).SectionObjectPointer).SharedCacheMap = core::ptr::null_mut();
    RemoveEntryList(core::ptr::addr_of_mut!((*SharedCacheMap).SharedCacheMapLinks));

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    KeReleaseQueuedSpinLock(0i32, *OldIrql);

    while !IsListEmpty(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead)) {
        Vacb = (RemoveHeadList(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead)) as *mut u8)
            .offset(-(core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;

        InitializeListHead(core::ptr::addr_of_mut!((*Vacb).CacheMapVacbListEntry));

        if (*Vacb).Dirty != 0 {
            Status = MmFlushSegment(
                (*FileObject).SectionObjectPointer,
                core::ptr::addr_of_mut!((*Vacb).FileOffset),
                0x1000u32,
                core::ptr::addr_of_mut!(Iosb),
            );
            if NT_SUCCESS(Status) == 0 {
            }
            (*Vacb).Dirty = 0u8;
        }

        RefCount = CcRosVacbDecRefCount(Vacb);
    }

    if !(*SharedCacheMap).Section.is_null() {
        ObDereferenceObject((*SharedCacheMap).Section as *mut core::ffi::c_void);
    }
    ObDereferenceObject((*SharedCacheMap).FileObject as *mut core::ffi::c_void);

    ExFreeToNPagedLookasideList(
        core::ptr::addr_of_mut!(SharedCacheMapLookasideList) as *mut core::ffi::c_void,
        SharedCacheMap as *mut core::ffi::c_void,
    );

    *OldIrql = KeAcquireQueuedSpinLock(0i32);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcRosFlushDirtyPages(
    Target: u32,
    Count: *mut u32,
    Wait: u8,
    CalledFromLazy: u8,
) -> i32 {
    let mut current_entry: PLIST_ENTRY;
    let mut Status: i32;
    let mut OldIrql: KIRQL;
    let FlushAll: u8 = if Target == u32::MAX { 1u8 } else { 0u8 };

    *Count = 0;

    KeEnterCriticalRegion();
    OldIrql = KeAcquireQueuedSpinLock(Locked);

    current_entry = (*core::ptr::addr_of_mut!(DirtyVacbListHead)).Flink;
    if current_entry == core::ptr::addr_of_mut!(DirtyVacbListHead) {
    }

    let mut Target = Target;
    while (current_entry != core::ptr::addr_of_mut!(DirtyVacbListHead) && Target > 0) || FlushAll != 0 {
        let mut SharedCacheMap: PROS_SHARED_CACHE_MAP;
        let mut current: PROS_VACB;
        let mut Locked: u8;

        if current_entry == core::ptr::addr_of_mut!(DirtyVacbListHead) {
            if IsListEmpty(core::ptr::addr_of_mut!(DirtyVacbListHead)) != 0 {
                break;
            }
            current_entry = (*core::ptr::addr_of_mut!(DirtyVacbListHead)).Flink;
        }

        current = (current_entry as *mut u8).offset(-(core::mem::offset_of!(_ROS_VACB, DirtyVacbListEntry) as isize)) as PROS_VACB;
        current_entry = (*current_entry).Flink;

        CcRosVacbIncRefCount(current);

        SharedCacheMap = (*current).SharedCacheMap;

        if CalledFromLazy != 0 && BooleanFlagOn((*(*SharedCacheMap).FileObject).Flags, 0x00001000) != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        if CalledFromLazy != 0 && BooleanFlagOn((*SharedCacheMap).Flags, 0x00000004) != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        if ((*SharedCacheMap).Flags & 0x00000008) != 0 {
            CcRosVacbDecRefCount(current);
            continue;
        }

        (*SharedCacheMap).Flags |= 0x00000008;

        (*SharedCacheMap).OpenCount += 1;

        KeReleaseQueuedSpinLock(Locked, OldIrql);

        Locked = ((*(*SharedCacheMap).Callbacks).AcquireForLazyWrite)((*SharedCacheMap).LazyWriteContext, Wait);
        if Locked == 0 {
            CcRosVacbDecRefCount(current);
            OldIrql = KeAcquireQueuedSpinLock(Locked);
            (*SharedCacheMap).Flags &= !0x00000008u32;

            (*SharedCacheMap).OpenCount -= 1;
            if (*SharedCacheMap).OpenCount == 0 {
                CcRosDeleteFileCache((*SharedCacheMap).FileObject, SharedCacheMap, core::ptr::addr_of_mut!(OldIrql));
            }

            continue;
        }

        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        Status = CcRosFlushVacb(current, core::ptr::addr_of_mut!(Iosb));

        ((*(*SharedCacheMap).Callbacks).ReleaseFromLazyWrite)((*SharedCacheMap).LazyWriteContext);

        CcRosVacbDecRefCount(current);
        OldIrql = KeAcquireQueuedSpinLock(Locked);

        (*SharedCacheMap).Flags &= !0x00000008u32;

        (*SharedCacheMap).OpenCount -= 1;
        if (*SharedCacheMap).OpenCount == 0 {
            CcRosDeleteFileCache((*SharedCacheMap).FileObject, SharedCacheMap, core::ptr::addr_of_mut!(OldIrql));
        }

        if NT_SUCCESS(Status) != 0 || Status == 0x00000100i32 || Status == 0xC00000E9i32 {
            let PagesFreed: u32 = (Iosb.Information / PAGE_SIZE as usize) as u32;
            *Count += PagesFreed;

            if Wait == 0 {
                if Target < PagesFreed {
                    Target = 0;
                } else {
                    Target -= PagesFreed;
                }
            }
        }

        current_entry = (*core::ptr::addr_of_mut!(DirtyVacbListHead)).Flink;
    }

    KeReleaseQueuedSpinLock(Locked, OldIrql);
    KeLeaveCriticalRegion();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcRosTrimCache(Target: u32, NrFreed: *mut u32) {
    let mut current_entry: PLIST_ENTRY;
    let mut current: PROS_VACB;
    let mut PagesFreed: u32;
    let mut oldIrql: KIRQL;
    let mut FreeList: LIST_ENTRY = core::mem::zeroed();
    let mut FlushedPages: u8 = 0u8;
    let mut Target = Target;

    InitializeListHead(&mut FreeList);

    *NrFreed = 0;

    'retry: loop {
        oldIrql = KeAcquireQueuedSpinLock(0); // LockQueueMasterLock = 0

        current_entry = (*(&VacbLruListHead as *const LIST_ENTRY as *mut LIST_ENTRY)).Flink;
        while current_entry != &VacbLruListHead as *const LIST_ENTRY as *mut LIST_ENTRY {
            let mut Refs: u32;

            current = (current_entry as *mut u8).offset(
                (core::mem::offset_of!(_ROS_VACB, VacbLruListEntry)) as isize,
            ) as *mut _ROS_VACB as PROS_VACB;

            KeAcquireSpinLockAtDpcLevel(
                &mut (*current).SharedCacheMap as *mut _ as *mut _,
            );

            CcRosVacbIncRefCount(current);

            if core::ptr::read_volatile(&(*current).MappedCount as *const u32) > 0
                && (*current).Dirty == 0u8
            {
            }

            current_entry = (*current_entry).Flink;

            Refs = CcRosVacbDecRefCount(current);

            if Refs < 2 {
                RemoveEntryList(&mut (*current).CacheMapVacbListEntry as *mut LIST_ENTRY);
                RemoveEntryList(&mut (*current).VacbLruListEntry as *mut LIST_ENTRY);
                InitializeListHead(&mut (*current).VacbLruListEntry);
                InsertHeadList(
                    &mut FreeList as *mut LIST_ENTRY,
                    &mut (*current).CacheMapVacbListEntry as *mut LIST_ENTRY,
                );

                PagesFreed = if (0x1000 / PAGE_SIZE) < Target {
                    0x1000 / PAGE_SIZE
                } else {
                    Target
                };
                Target -= PagesFreed;
                *NrFreed += PagesFreed;
            }

            KeReleaseSpinLockFromDpcLevel(
                &mut (*current).SharedCacheMap as *mut _ as *mut _,
            );
        }

        KeReleaseQueuedSpinLock(0, oldIrql); // LockQueueMasterLock = 0

        if (Target > 0) && (FlushedPages == 0u8) {
            CcRosFlushDirtyPages(Target, &mut PagesFreed, 0u8, 0u8);
            FlushedPages = 1u8;

            if PagesFreed < Target {
                Target = PagesFreed;
            }

            if PagesFreed != 0 {
                break 'retry;
            }
        }

        break;
    }

    while !IsListEmpty(&FreeList) {
        let mut Refs: u32;

        current_entry = RemoveHeadList(&mut FreeList as *mut LIST_ENTRY);
        current = (current_entry as *mut u8).offset(
            (core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry)) as isize,
        ) as *mut _ROS_VACB as PROS_VACB;
        InitializeListHead(&mut (*current).CacheMapVacbListEntry);
        Refs = CcRosVacbDecRefCount(current);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcRosReleaseVacb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    Vacb: PROS_VACB,
    Dirty: BOOLEAN,
    Mapped: BOOLEAN,
) -> NTSTATUS {
    let Refs: ULONG;

    if Dirty != 0 && (*Vacb).Dirty == 0 {
        CcRosMarkDirtyVacb(Vacb);
    }

    if Mapped != 0 {
        if InterlockedIncrement(core::ptr::addr_of_mut!((*Vacb).MappedCount) as *mut i32) == 1 {
            CcRosVacbIncRefCount(Vacb);
        }
    }

    Refs = CcRosVacbDecRefCount(Vacb);

    STATUS_SUCCESS
}

unsafe fn CcRosLookupVacb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: i64,
) -> PROS_VACB {
    let mut current_entry: PLIST_ENTRY;
    let mut current: PROS_VACB;
    let mut oldIrql: KIRQL;

    oldIrql = KeAcquireQueuedSpinLock(0u32);
    KeAcquireSpinLockAtDpcLevel(&(*SharedCacheMap).CacheMapLock);

    current_entry = (*SharedCacheMap).CacheMapVacbListHead.Flink;
    loop {
        if current_entry == core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
            break;
        }

        current = (current_entry as usize - core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry)) as PROS_VACB;

        if IsPointInRange(
            (*current).FileOffset.QuadPart,
            0x4000i64,
            FileOffset,
        ) != 0
        {
            CcRosVacbIncRefCount(current);
            KeReleaseSpinLockFromDpcLevel(&(*SharedCacheMap).CacheMapLock);
            KeReleaseQueuedSpinLock(0u32, oldIrql);
            return current;
        }
        if (*current).FileOffset.QuadPart > FileOffset {
            break;
        }
        current_entry = (*current_entry).Flink;
    }

    KeReleaseSpinLockFromDpcLevel(&(*SharedCacheMap).CacheMapLock);
    KeReleaseQueuedSpinLock(0u32, oldIrql);

    core::ptr::null_mut()
}

unsafe fn CcRosMarkDirtyVacb(Vacb: PROS_VACB) {
    let mut oldIrql: KIRQL;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    SharedCacheMap = (*Vacb).SharedCacheMap;

    oldIrql = KeAcquireQueuedSpinLock(0); // LockQueueMasterLock = 0
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));

    InsertTailList(
        core::ptr::addr_of_mut!(DirtyVacbListHead),
        core::ptr::addr_of_mut!((*Vacb).DirtyVacbListEntry),
    );
    CcTotalDirtyPages += 16 / PAGE_SIZE; // VACB_MAPPING_GRANULARITY = 16 * PAGE_SIZE = 0x4000, so 16 / PAGE_SIZE = 1 on x86
    (*SharedCacheMap).DirtyPages += 16 / PAGE_SIZE;
    CcRosVacbIncRefCount(Vacb);

    RemoveEntryList(core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry));
    InsertTailList(
        core::ptr::addr_of_mut!(VacbLruListHead),
        core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry),
    );

    (*Vacb).Dirty = 1u8;

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));

    if LazyWriter.ScanActive == 0 {
        CcScheduleLazyWriteScan(0u8);
    }
    KeReleaseQueuedSpinLock(0, oldIrql); // LockQueueMasterLock = 0
}

unsafe fn CcRosUnmarkDirtyVacb(Vacb: PROS_VACB, LockViews: u8) {
    let mut oldIrql: KIRQL;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    SharedCacheMap = (*Vacb).SharedCacheMap;

    if LockViews != 0 {
        oldIrql = KeAcquireQueuedSpinLock(17i32);
        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    }

    (*Vacb).Dirty = 0u8;

    RemoveEntryList(core::ptr::addr_of_mut!((*Vacb).DirtyVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*Vacb).DirtyVacbListEntry));

    CcTotalDirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE;
    (*(*Vacb).SharedCacheMap).DirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE;

    CcRosVacbDecRefCount(Vacb);

    if LockViews != 0 {
        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
        KeReleaseQueuedSpinLock(17i32, oldIrql);
    }
}

unsafe fn CcRosFreeOneUnusedVacb() -> BOOLEAN {
    let mut oldIrql: KIRQL = 0;
    let mut current_entry: PLIST_ENTRY;
    let mut to_free: PROS_VACB = core::ptr::null_mut();

    oldIrql = KeAcquireQueuedSpinLock(0); // LockQueueMasterLock

    current_entry = (*VacbLruListHead.Flink);
    while (current_entry != core::ptr::addr_of_mut!(VacbLruListHead)) && (to_free == core::ptr::null_mut()) {
        let Refs: ULONG;
        let current: PROS_VACB;

        let current_ptr = (current_entry as *mut u8).offset(-(core::mem::offset_of!(ROS_VACB, VacbLruListEntry) as isize)) as PROS_VACB;
        current = current_ptr;

        KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*current).SharedCacheMap.CacheMapLock));

        Refs = CcRosVacbGetRefCount(current);
        if Refs < 2 {
            RemoveEntryList(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
            InitializeListHead(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
            RemoveEntryList(core::ptr::addr_of_mut!((*current).VacbLruListEntry));
            InitializeListHead(core::ptr::addr_of_mut!((*current).VacbLruListEntry));

            to_free = current;
        }

        KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*current).SharedCacheMap.CacheMapLock));

        current_entry = (*current_entry).Flink;
    }

    KeReleaseQueuedSpinLock(0, oldIrql); // LockQueueMasterLock

    if to_free == core::ptr::null_mut() {
        return 0u8;
    }

    NT_VERIFY((CcRosVacbDecRefCount(to_free) == 0) as BOOLEAN);

    return 1u8;
}

unsafe fn CcRosCreateVacb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: i64,
    Vacb: *mut PROS_VACB,
) -> i32 {
    let mut current: PROS_VACB;
    let mut previous: PROS_VACB;
    let mut current_entry: PLIST_ENTRY;
    let mut Status: i32;
    let mut oldIrql: KIRQL;
    let mut Refs: u32;
    let mut ViewSize: usize = 0x10000;

    ARM64_UART_TRACE(b"[CC] CreateVacb: alloc\r\n".as_ptr());
    current = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(VacbLookasideList)) as PROS_VACB;
    if current == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ARM64_UART_TRACE(b"[CC] CreateVacb: alloc done\r\n".as_ptr());

    (*current).BaseAddress = core::ptr::null_mut();
    (*current).Dirty = 0u8;
    (*current).PageOut = 0u8;
    (*current).FileOffset.QuadPart = ROUND_DOWN(FileOffset, 0x10000);
    (*current).SharedCacheMap = SharedCacheMap;
    (*current).MappedCount = 0;
    (*current).ReferenceCount = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*current).DirtyVacbListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*current).VacbLruListEntry));

    CcRosVacbIncRefCount(current);

    loop {
        ARM64_UART_TRACE(b"[CC] CreateVacb: MmMapViewInSystemSpaceEx\r\n".as_ptr());
        Status = MmMapViewInSystemSpaceEx(
            (*SharedCacheMap).Section,
            core::ptr::addr_of_mut!((*current).BaseAddress),
            core::ptr::addr_of_mut!(ViewSize),
            core::ptr::addr_of_mut!((*current).FileOffset),
            0,
        );
        if NT_SUCCESS(Status) != 0 {
            #[cfg(target_arch = "aarch64")]
            ARM64_BARRIER();
            ARM64_UART_TRACE(b"[CC] CreateVacb: MapView done\r\n".as_ptr());
            break;
        }

        if CcRosFreeOneUnusedVacb() == 0 {
            ExFreeToNPagedLookasideList(core::ptr::addr_of_mut!(VacbLookasideList), current as *mut core::ffi::c_void);
            return Status;
        }
    }

    oldIrql = KeAcquireQueuedSpinLock(0);

    *Vacb = current;
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    current_entry = (*SharedCacheMap).CacheMapVacbListHead.Flink;
    previous = core::ptr::null_mut();

    while current_entry != core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
        current = (current_entry as usize - core::mem::offset_of!(_ROS_VACB, CacheMapVacbListEntry)) as PROS_VACB;

        if IsPointInRange((*current).FileOffset.QuadPart, 0x10000, FileOffset) != 0 {
            CcRosVacbIncRefCount(current);
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
            KeReleaseQueuedSpinLock(0, oldIrql);

            Refs = CcRosVacbDecRefCount(*Vacb);

            *Vacb = current;
            return STATUS_SUCCESS;
        }

        if (*current).FileOffset.QuadPart < FileOffset {
            previous = current;
        }

        if (*current).FileOffset.QuadPart > FileOffset {
            break;
        }

        current_entry = (*current_entry).Flink;
    }

    current = *Vacb;
    if previous != core::ptr::null_mut() {
        InsertHeadList(core::ptr::addr_of_mut!((*previous).CacheMapVacbListEntry), core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
    } else {
        InsertHeadList(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead), core::ptr::addr_of_mut!((*current).CacheMapVacbListEntry));
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    InsertTailList(core::ptr::addr_of_mut!(VacbLruListHead), core::ptr::addr_of_mut!((*current).VacbLruListEntry));

    CcRosVacbIncRefCount(current);

    KeReleaseQueuedSpinLock(0, oldIrql);

    Status
}

unsafe fn CcRosEnsureVacbResident(
    Vacb: PROS_VACB,
    Wait: u8,
    NoRead: u8,
    Offset: u32,
    Length: u32,
) -> u8 {
    let SharedCacheMap = (*Vacb).SharedCacheMap;

    if MmIsDataSectionResident(
        (*(*SharedCacheMap).FileObject).SectionObjectPointer,
        (*Vacb).FileOffset.QuadPart + Offset as i64,
        Length,
    ) == 0
    {
        if Wait == 0 {
            return 0u8;
        }

        if NoRead == 0 {
            #[cfg(target_arch = "aarch64")]
            {
                ARM64_BARRIER();
            }

            let Status = MmMakeDataSectionResident(
                (*(*SharedCacheMap).FileObject).SectionObjectPointer,
                (*Vacb).FileOffset.QuadPart + Offset as i64,
                Length,
                core::ptr::addr_of_mut!((*SharedCacheMap).ValidDataLength),
            );
            if NT_SUCCESS(Status) == 0 {
                ExRaiseStatus(Status);
            }
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CcRosGetVacb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: i64,
    Vacb: *mut PROS_VACB,
) -> NTSTATUS {
    let mut current: PROS_VACB;
    let Status: NTSTATUS;
    let Refs: ULONG;
    let mut OldIrql: KIRQL;

    current = CcRosLookupVacb(SharedCacheMap, FileOffset);
    if current == core::ptr::null_mut() {
        Status = CcRosCreateVacb(SharedCacheMap, FileOffset, &mut current);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Refs = CcRosVacbGetRefCount(current);

    OldIrql = KeAcquireQueuedSpinLock(0u32);

    RemoveEntryList(&mut (*current).VacbLruListEntry);
    InsertTailList(&mut VacbLruListHead, &mut (*current).VacbLruListEntry);

    KeReleaseQueuedSpinLock(0u32, OldIrql);

    *Vacb = current;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcRosRequestVacb(
    SharedCacheMap: PROS_SHARED_CACHE_MAP,
    FileOffset: i64,
    Vacb: *mut PROS_VACB,
) -> NTSTATUS {
    if FileOffset % VACB_MAPPING_GRANULARITY as i64 != 0 {
        KeBugCheck(CACHE_MANAGER);
    }

    CcRosGetVacb(SharedCacheMap, FileOffset, Vacb)
}

#[no_mangle]
pub unsafe extern "system" fn CcRosInternalFreeVacb(Vacb: PROS_VACB) -> NTSTATUS {
    let Status: NTSTATUS;

    if (*Vacb).ReferenceCount != 0 {
        if !(*(*Vacb).SharedCacheMap).FileObject.is_null() 
            && (*(*(*Vacb).SharedCacheMap).FileObject).FileName.Length != 0
        {
        }
    }

    Status = MmUnmapViewInSystemSpace((*Vacb).BaseAddress);
    if !NT_SUCCESS(Status) {
    }

    core::ptr::write_bytes(Vacb as *mut u8, 0xfdu8, core::mem::size_of_val(&*Vacb));
    ExFreeToNPagedLookasideList(&VacbLookasideList, Vacb as *mut core::ffi::c_void);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcFlushCache(
    SectionObjectPointers: PSECTION_OBJECT_POINTERS,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let mut SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let mut FlushStart: LONGLONG;
    let mut FlushEnd: LONGLONG;
    let mut Status: NTSTATUS;

    if SectionObjectPointers.is_null() {
        Status = STATUS_INVALID_PARAMETER;
        goto_quit(IoStatus, Status);
        return;
    }

    if (*SectionObjectPointers).SharedCacheMap.is_null() {
        MmFlushSegment(SectionObjectPointers, FileOffset, Length, IoStatus);
        return;
    }

    SharedCacheMap = (*SectionObjectPointers).SharedCacheMap;
    
    if !FileOffset.is_null() {
        FlushStart = (*FileOffset).QuadPart;
        Status = RtlLongLongAdd(FlushStart, Length as LONGLONG, core::ptr::addr_of_mut!(FlushEnd));
        if NT_SUCCESS(Status) == 0 {
            goto_quit(IoStatus, Status);
            return;
        }
    } else {
        FlushStart = 0;
        FlushEnd = (*SharedCacheMap).FileSize.QuadPart;
    }

    Status = STATUS_SUCCESS;
    if !IoStatus.is_null() {
        (*IoStatus).Information = 0;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!((*SharedCacheMap).FlushCacheLock));

    while FlushStart < FlushEnd {
        let mut DirtyVacb: BOOLEAN = 0u8;
        let vacb: PROS_VACB = CcRosLookupVacb(SharedCacheMap, FlushStart);

        if !vacb.is_null() {
            if (*vacb).Dirty != 0 {
                let mut VacbIosb: _IO_STATUS_BLOCK = core::mem::zeroed();
                Status = CcRosFlushVacb(vacb, core::ptr::addr_of_mut!(VacbIosb));
                if NT_SUCCESS(Status) == 0 {
                    CcRosReleaseVacb(SharedCacheMap, vacb, 0u8, 0u8);
                    break;
                }
                DirtyVacb = 1u8;

                if !IoStatus.is_null() {
                    (*IoStatus).Information += VacbIosb.Information;
                }
            }

            CcRosReleaseVacb(SharedCacheMap, vacb, 0u8, 0u8);
        }

        if DirtyVacb == 0 {
            let mut MmIosb: _IO_STATUS_BLOCK;
            let mut MmOffset: LARGE_INTEGER;

            MmOffset.QuadPart = FlushStart;

            if FlushEnd - (FlushEnd % VACB_MAPPING_GRANULARITY as LONGLONG) <= FlushStart {
                Status = MmFlushSegment(
                    SectionObjectPointers,
                    core::ptr::addr_of_mut!(MmOffset),
                    (FlushEnd - FlushStart) as ULONG,
                    core::ptr::addr_of_mut!(MmIosb),
                );
            } else {
                let MmLength: ULONG = (VACB_MAPPING_GRANULARITY as LONGLONG - (FlushStart % VACB_MAPPING_GRANULARITY as LONGLONG)) as ULONG;
                Status = MmFlushSegment(
                    SectionObjectPointers,
                    core::ptr::addr_of_mut!(MmOffset),
                    MmLength,
                    core::ptr::addr_of_mut!(MmIosb),
                );
            }

            if NT_SUCCESS(Status) == 0 {
                break;
            }

            if !IoStatus.is_null() {
                (*IoStatus).Information += MmIosb.Information;
            }

            if (*SharedCacheMap).ValidDataLength.QuadPart < FlushEnd {
                (*SharedCacheMap).ValidDataLength.QuadPart = FlushEnd;
            }
        }

        if NT_SUCCESS(RtlLongLongAdd(
            FlushStart,
            VACB_MAPPING_GRANULARITY as LONGLONG,
            core::ptr::addr_of_mut!(FlushStart),
        )) == 0
        {
            break;
        }

        FlushStart -= FlushStart % VACB_MAPPING_GRANULARITY as LONGLONG;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*SharedCacheMap).FlushCacheLock));

    goto_quit(IoStatus, Status);
}

unsafe fn goto_quit(IoStatus: PIO_STATUS_BLOCK, Status: NTSTATUS) {
    if !IoStatus.is_null() {
        (*IoStatus).__bindgen_anon_1.Status = Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcRosReleaseFileCache(FileObject: PFILE_OBJECT) -> i32 {
    let mut OldIrql: KIRQL;
    let mut PrivateMap: PPRIVATE_CACHE_MAP;
    let mut SharedCacheMap: PROS_SHARED_CACHE_MAP;

    OldIrql = KeAcquireQueuedSpinLock(17i32);

    if !(*(*(*FileObject).SectionObjectPointer).SharedCacheMap).is_null() {
        SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

        PrivateMap = (*FileObject).PrivateCacheMap;
        (*FileObject).PrivateCacheMap = core::ptr::null_mut();

        if !PrivateMap.is_null() {
            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
            RemoveEntryList(core::ptr::addr_of_mut!((*PrivateMap).PrivateLinks));
            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));

            if PrivateMap != core::ptr::addr_of_mut!((*SharedCacheMap).PrivateCacheMap) {
                ExFreePoolWithTag(PrivateMap as *mut core::ffi::c_void, TAG_PRIVATE_CACHE_MAP);
            } else {
                (*PrivateMap).NodeTypeCode = 0;
            }

            (*SharedCacheMap).OpenCount -= 1;
            if (*SharedCacheMap).OpenCount == 0 {
                CcRosDeleteFileCache(FileObject, SharedCacheMap, core::ptr::addr_of_mut!(OldIrql));
            }
        }
    }
    KeReleaseQueuedSpinLock(17i32, OldIrql);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcRosInitializeFileCache(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
    PinAccess: u8,
    CallBacks: PCACHE_MANAGER_CALLBACKS,
    LazyWriterContext: *mut core::ffi::c_void,
) -> i32 {
    let mut OldIrql: KIRQL;
    let mut Allocated: u8;
    let mut SharedCacheMap: PROS_SHARED_CACHE_MAP;

    OldIrql = KeAcquireQueuedSpinLock(17i32);

    Allocated = 0u8;
    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    if SharedCacheMap.is_null() {
        SharedCacheMap = ExAllocateFromNPagedLookasideList(&SharedCacheMapLookasideList) as PROS_SHARED_CACHE_MAP;
        if SharedCacheMap.is_null() {
            KeReleaseQueuedSpinLock(17i32, OldIrql);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        RtlZeroMemory(SharedCacheMap as *mut core::ffi::c_void, core::mem::size_of::<_ROS_SHARED_CACHE_MAP>());
        (*SharedCacheMap).NodeTypeCode = NODE_TYPE_SHARED_MAP;
        (*SharedCacheMap).NodeByteSize = core::mem::size_of::<_ROS_SHARED_CACHE_MAP>();
        (*SharedCacheMap).FileObject = FileObject;
        (*SharedCacheMap).Callbacks = CallBacks;
        (*SharedCacheMap).LazyWriteContext = LazyWriterContext;
        (*SharedCacheMap).SectionSize = (*FileSizes).AllocationSize;
        (*SharedCacheMap).FileSize = (*FileSizes).FileSize;
        (*SharedCacheMap).ValidDataLength = (*FileSizes).ValidDataLength;
        (*SharedCacheMap).PinAccess = PinAccess;
        (*SharedCacheMap).DirtyPageThreshold = 0;
        (*SharedCacheMap).DirtyPages = 0;
        InitializeListHead(&mut (*SharedCacheMap).PrivateList);
        KeInitializeSpinLock(&mut (*SharedCacheMap).CacheMapLock);
        InitializeListHead(&mut (*SharedCacheMap).CacheMapVacbListHead);
        InitializeListHead(&mut (*SharedCacheMap).BcbList);
        KeInitializeGuardedMutex(&mut (*SharedCacheMap).FlushCacheLock);

        (*SharedCacheMap).Flags = SHARED_CACHE_MAP_IN_CREATION;

        ObReferenceObjectByPointer(FileObject, FILE_ALL_ACCESS, core::ptr::null_mut(), 0i32);

        Allocated = 1u8;
        (*(*FileObject).SectionObjectPointer).SharedCacheMap = SharedCacheMap;
    } else if ((*SharedCacheMap).Flags & SHARED_CACHE_MAP_IN_CREATION) != 0 {
        let mut Waiter: KEVENT;
        let mut PreviousWaiter: PKEVENT = (*SharedCacheMap).CreateEvent;

        KeInitializeEvent(&mut Waiter, _EVENT_TYPE_NotificationEvent, 0u8);
        (*SharedCacheMap).CreateEvent = &mut Waiter;

        KeReleaseQueuedSpinLock(17i32, OldIrql);

        KeWaitForSingleObject(&mut Waiter, _KWAIT_REASON_Executive, 0i32, 0u8, core::ptr::null_mut());

        if !PreviousWaiter.is_null() {
            KeSetEvent(PreviousWaiter, IO_NO_INCREMENT, 0u8);
        }

        OldIrql = KeAcquireQueuedSpinLock(17i32);
    }

    if (*FileObject).PrivateCacheMap.is_null() {
        let mut PrivateMap: PPRIVATE_CACHE_MAP;

        if (*SharedCacheMap).PrivateCacheMap.NodeTypeCode != 0 {
            PrivateMap = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<PRIVATE_CACHE_MAP>(), TAG_PRIVATE_CACHE_MAP) as PPRIVATE_CACHE_MAP;
        } else {
            PrivateMap = &mut (*SharedCacheMap).PrivateCacheMap;
        }

        if PrivateMap.is_null() {
            if Allocated != 0 {
                RemoveEntryList(&mut (*SharedCacheMap).SharedCacheMapLinks);

                (*(*FileObject).SectionObjectPointer).SharedCacheMap = core::ptr::null_mut();
                ObDereferenceObject(FileObject as *mut core::ffi::c_void);
                ExFreeToNPagedLookasideList(&SharedCacheMapLookasideList, SharedCacheMap as *mut core::ffi::c_void);
            }

            KeReleaseQueuedSpinLock(17i32, OldIrql);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlZeroMemory(PrivateMap as *mut core::ffi::c_void, core::mem::size_of::<PRIVATE_CACHE_MAP>());
        (*PrivateMap).NodeTypeCode = NODE_TYPE_PRIVATE_MAP;
        (*PrivateMap).ReadAheadMask = PAGE_SIZE - 1;
        (*PrivateMap).FileObject = FileObject;
        KeInitializeSpinLock(&mut (*PrivateMap).ReadAheadSpinLock);

        KeAcquireSpinLockAtDpcLevel(&mut (*SharedCacheMap).CacheMapLock);
        InsertTailList(&mut (*SharedCacheMap).PrivateList, &mut (*PrivateMap).PrivateLinks);
        KeReleaseSpinLockFromDpcLevel(&mut (*SharedCacheMap).CacheMapLock);

        (*FileObject).PrivateCacheMap = PrivateMap;
        (*SharedCacheMap).OpenCount += 1;
    }

    KeReleaseQueuedSpinLock(17i32, OldIrql);

    if Allocated != 0 {
        let mut Status: i32;

        Status = MmCreateSection(
            &mut (*SharedCacheMap).Section,
            SECTION_ALL_ACCESS,
            core::ptr::null_mut(),
            &mut (*SharedCacheMap).SectionSize,
            PAGE_READWRITE,
            SEC_RESERVE,
            core::ptr::null_mut(),
            FileObject,
        );

        if !NT_SUCCESS(Status) {
            CcRosReleaseFileCache(FileObject);
            return Status;
        }

        OldIrql = KeAcquireQueuedSpinLock(17i32);

        InsertTailList(&mut CcCleanSharedCacheMapList, &mut (*SharedCacheMap).SharedCacheMapLinks);
        (*SharedCacheMap).Flags &= !SHARED_CACHE_MAP_IN_CREATION;

        if !(*SharedCacheMap).CreateEvent.is_null() {
            KeSetEvent((*SharedCacheMap).CreateEvent, IO_NO_INCREMENT, 0u8);
            (*SharedCacheMap).CreateEvent = core::ptr::null_mut();
        }

        KeReleaseQueuedSpinLock(17i32, OldIrql);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileObjectFromSectionPtrs(
    SectionObjectPointers: PSECTION_OBJECT_POINTERS,
) -> PFILE_OBJECT {
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    CCTRACE(CC_API_DEBUG, b"SectionObjectPointers=%p\n\0".as_ptr() as PCHAR, SectionObjectPointers as *const core::ffi::c_void);

    if !SectionObjectPointers.is_null() && !(*SectionObjectPointers).SharedCacheMap.is_null() {
        SharedCacheMap = (*SectionObjectPointers).SharedCacheMap;
        return (*SharedCacheMap).FileObject;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CcInitView() {
    InitializeListHead(&mut DirtyVacbListHead);
    InitializeListHead(&mut VacbLruListHead);
    InitializeListHead(&mut CcDeferredWrites);
    InitializeListHead(&mut CcCleanSharedCacheMapList);
    KeInitializeSpinLock(&mut CcDeferredWriteSpinLock);
    ExInitializeNPagedLookasideList(
        &mut iBcbLookasideList,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        core::mem::size_of::<INTERNAL_BCB>(),
        TAG_BCB,
        20,
    );
    ExInitializeNPagedLookasideList(
        &mut SharedCacheMapLookasideList,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        core::mem::size_of::<ROS_SHARED_CACHE_MAP>(),
        TAG_SHARED_CACHE_MAP,
        20,
    );
    ExInitializeNPagedLookasideList(
        &mut VacbLookasideList,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        core::mem::size_of::<ROS_VACB>(),
        TAG_VACB,
        20,
    );
    CcInitCacheZeroPage();
}

#[no_mangle]
pub unsafe extern "system" fn ExpKdbgExtFileCache(Argc: ntkrnl_sys::base::ULONG, Argv: *mut ntkrnl_sys::base::PCHAR) -> ntkrnl_sys::base::BOOLEAN {
    let mut ListEntry: ntkrnl_sys::base::PLIST_ENTRY;
    let NoName: ntkrnl_sys::base::_UNICODE_STRING = ntkrnl_sys::base::_UNICODE_STRING {
        Length: 32u16,
        MaximumLength: 34u16,
        Buffer: b"No name for File\0".as_ptr() as ntkrnl_sys::base::PWSTR,
    };

    KdbpPrint(b"  Usage Summary (in kb)\n\0".as_ptr() as ntkrnl_sys::base::PCHAR);
    KdbpPrint(b"Shared\t\tMapped\tDirty\tName\n\0".as_ptr() as ntkrnl_sys::base::PCHAR);

    ListEntry = CcCleanSharedCacheMapList.Flink;
    while ListEntry != core::ptr::addr_of_mut!(CcCleanSharedCacheMapList) {
        let mut Vacbs: ntkrnl_sys::base::PLIST_ENTRY;
        let mut Mapped: ntkrnl_sys::base::ULONG = 0;
        let mut Dirty: ntkrnl_sys::base::ULONG = 0;
        let SharedCacheMap: ntkrnl_sys::ddk::PROS_SHARED_CACHE_MAP;
        let FileName: ntkrnl_sys::base::PUNICODE_STRING;
        let mut Extra: ntkrnl_sys::base::PWSTR = b"\0".as_ptr() as ntkrnl_sys::base::PWSTR;

        SharedCacheMap = (ListEntry as ntkrnl_sys::base::PUCHAR).offset(-(core::mem::offset_of!(ntkrnl_sys::ddk::ROS_SHARED_CACHE_MAP, SharedCacheMapLinks) as isize)) as ntkrnl_sys::ddk::PROS_SHARED_CACHE_MAP;

        Dirty = ((*SharedCacheMap).DirtyPages * ntkrnl_sys::base::PAGE_SIZE) / 1024;

        Vacbs = (*SharedCacheMap).CacheMapVacbListHead.Flink;
        while Vacbs != core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
            Mapped += 65536 / 1024;
            Vacbs = (*Vacbs).Flink;
        }

        if !(*SharedCacheMap).FileObject.is_null() && (*(*SharedCacheMap).FileObject).FileName.Length != 0 {
            FileName = core::ptr::addr_of_mut!((*(*SharedCacheMap).FileObject).FileName);
        } else if !(*SharedCacheMap).FileObject.is_null() && !(*(*SharedCacheMap).FileObject).FsContext.is_null() &&
                  (*((*(*SharedCacheMap).FileObject).FsContext as ntkrnl_sys::ddk::PFSRTL_COMMON_FCB_HEADER)).NodeTypeCode == 0x0502 &&
                  (*((*(*SharedCacheMap).FileObject).FsContext as ntkrnl_sys::ddk::PFSRTL_COMMON_FCB_HEADER)).NodeByteSize == 0x1F8 &&
                  (*((((*(*SharedCacheMap).FileObject).FsContext as ntkrnl_sys::base::PUCHAR).add(0x100)) as ntkrnl_sys::base::PUNICODE_STRING)).Length != 0 {
            FileName = (((*(*SharedCacheMap).FileObject).FsContext as ntkrnl_sys::base::PUCHAR).add(0x100)) as ntkrnl_sys::base::PUNICODE_STRING;
            Extra = b" (FastFAT)\0".as_ptr() as ntkrnl_sys::base::PWSTR;
        } else {
            FileName = core::ptr::addr_of_mut!(NoName) as ntkrnl_sys::base::PUNICODE_STRING;
        }

        KdbpPrint(b"%p\t%d\t%d\t%wZ%S\n\0".as_ptr() as ntkrnl_sys::base::PCHAR, SharedCacheMap, Mapped, Dirty, FileName, Extra);

        ListEntry = (*ListEntry).Flink;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExpKdbgExtDefWrites(Argc: u32, Argv: *mut *mut u8) -> u8 {
    KdbpPrint(
        b"CcTotalDirtyPages:\t%lu (%lu Kb)\n\0".as_ptr(),
        CcTotalDirtyPages,
        (CcTotalDirtyPages.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );
    KdbpPrint(
        b"CcDirtyPageThreshold:\t%lu (%lu Kb)\n\0".as_ptr(),
        CcDirtyPageThreshold,
        (CcDirtyPageThreshold.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );
    KdbpPrint(
        b"MmAvailablePages:\t%lu (%lu Kb)\n\0".as_ptr(),
        MmAvailablePages,
        (MmAvailablePages.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );
    KdbpPrint(
        b"MmThrottleTop:\t\t%lu (%lu Kb)\n\0".as_ptr(),
        MmThrottleTop,
        (MmThrottleTop.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );
    KdbpPrint(
        b"MmThrottleBottom:\t%lu (%lu Kb)\n\0".as_ptr(),
        MmThrottleBottom,
        (MmThrottleBottom.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );
    KdbpPrint(
        b"MmModifiedPageListHead.Total:\t%lu (%lu Kb)\n\0".as_ptr(),
        MmModifiedPageListHead.Total,
        (MmModifiedPageListHead.Total.wrapping_mul(PAGE_SIZE as u32)).wrapping_div(1024),
    );

    if CcTotalDirtyPages >= CcDirtyPageThreshold {
        KdbpPrint(b"CcTotalDirtyPages above the threshold, writes should be throttled\n\0".as_ptr());
    } else if CcTotalDirtyPages.wrapping_add(64) >= CcDirtyPageThreshold {
        KdbpPrint(b"CcTotalDirtyPages within 64 (max charge) pages of the threshold, writes may be throttled\n\0".as_ptr());
    } else {
        KdbpPrint(b"CcTotalDirtyPages below the threshold, writes should not be throttled\n\0".as_ptr());
    }

    1u8
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
//             _IO_STATUS_BLOCK Iosb;
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
//         _IO_STATUS_BLOCK Iosb;
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
//             for (i = 0; i < VACB_MAPPING_GRANULARITY / PAGE_SIZE; i++)
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
//             PagesFreed = min(VACB_MAPPING_GRANULARITY / PAGE_SIZE, Target);
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
//     CcTotalDirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE;
//     Vacb->SharedCacheMap->DirtyPages += VACB_MAPPING_GRANULARITY / PAGE_SIZE;
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
//     CcTotalDirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE;
//     Vacb->SharedCacheMap->DirtyPages -= VACB_MAPPING_GRANULARITY / PAGE_SIZE;
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
//                 _IO_STATUS_BLOCK VacbIosb = { 0 };
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
//             _IO_STATUS_BLOCK MmIosb;
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
//     _UNICODE_STRING NoName = RTL_CONSTANT_STRING(L"No name for File");
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
