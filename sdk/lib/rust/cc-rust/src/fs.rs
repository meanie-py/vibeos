use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    LARGE_INTEGER, PVPB, PFILE_OBJECT, PLARGE_INTEGER, PDIRTY_PAGE_ROUTINE,
    PINTERNAL_BCB, PCACHE_MANAGER_CALLBACKS, PCC_FILE_SIZES,
    PSECTION_OBJECT_POINTERS, PCACHE_UNINITIALIZE_EVENT, PFLUSH_TO_LSN,
    PPRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP, PROS_VACB,
    PRIVATE_CACHE_MAP, ROS_VACB, INTERNAL_BCB,
    KeSetEvent, MmExtendSection, MmPurgeSegment,
};
use ntkrnl_sys::ex::ExRaiseStatus;
use ntkrnl_sys::ke::{KeAcquireQueuedSpinLock, KeReleaseQueuedSpinLock, KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut DirtyVacbListHead: LIST_ENTRY;
    fn KeAcquireSpinLockAtDpcLevel(SpinLock: *mut usize);
    fn KeReleaseSpinLockFromDpcLevel(SpinLock: *mut usize);
}
extern "C" {
    fn CcRosVacbGetRefCount(Vacb: PROS_VACB) -> ULONG;
    fn CcRosVacbDecRefCount(Vacb: PROS_VACB) -> ULONG;
    fn CcRosInitializeFileCache(FileObject: PFILE_OBJECT, FileSizes: PCC_FILE_SIZES, PinAccess: BOOLEAN, CallBacks: PCACHE_MANAGER_CALLBACKS, LazyWriterContext: PVOID) -> NTSTATUS;
    fn CcRosReleaseFileCache(FileObject: PFILE_OBJECT) -> NTSTATUS;
    fn CcRosUnmarkDirtyVacb(Vacb: PROS_VACB, LockViews: BOOLEAN) -> BOOLEAN;
}

const VACB_MAPPING_GRANULARITY: i64 = 262144;
const MAXLONGLONG: i64 = 0x7FFFFFFFFFFFFFFF;
pub(crate) const FO_TEMPORARY_FILE: u32 = 0x8000;
const IO_NO_INCREMENT: LONG = 0;
// KSPIN_LOCK_QUEUE_NUMBER values. Spelled out because the translation shipped bare
// integer literals here and they were the WRONG ones: 17 (LockQueueTimerTableLock)
// for the master lock and 6 (LockQueueNonPagedPoolLock) for the work-queue lock,
// so Cc took KiTimerTableLock[0] where the C takes CcMasterSpinLock. Never pass a
// literal to Ke*QueuedSpinLock.
const LockQueueMasterLock: i32 = 0x00000005;

#[no_mangle]
pub unsafe extern "system" fn CcGetDirtyPages(
    _LogHandle: PVOID,
    _DirtyPageRoutine: PDIRTY_PAGE_ROUTINE,
    _Context1: PVOID,
    _Context2: PVOID,
) -> LARGE_INTEGER {
    let _LogHandle = opaque_ptr(_LogHandle);
    let _Context1 = opaque_ptr(_Context1);
    let _Context2 = opaque_ptr(_Context2);
    LARGE_INTEGER { QuadPart: 0 }
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileObjectFromBcb(Bcb: PVOID) -> PFILE_OBJECT {
    let Bcb = opaque_ptr(Bcb);
    let i_bcb = (Bcb as *const u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;
    (*(*i_bcb).Vacb).SharedCacheMap.as_ref().unwrap().FileObject
}

#[no_mangle]
pub unsafe extern "system" fn CcGetLsnForFileObject(
    _FileObject: PFILE_OBJECT,
    _OldestLsn: PLARGE_INTEGER,
) -> LARGE_INTEGER {
    let _FileObject = opaque_ptr(_FileObject);
    let _OldestLsn = opaque_ptr(_OldestLsn);
    LARGE_INTEGER { QuadPart: 0 }
}

#[no_mangle]
pub unsafe extern "system" fn CcInitializeCacheMap(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
    PinAccess: BOOLEAN,
    CallBacks: PCACHE_MANAGER_CALLBACKS,
    LazyWriterContext: PVOID,
) {
    let FileObject = opaque_ptr(FileObject);
    let LazyWriterContext = opaque_ptr(LazyWriterContext);
    let status = CcRosInitializeFileCache(
        FileObject,
        FileSizes,
        PinAccess,
        CallBacks,
        LazyWriterContext,
    );
    if status < 0 {
        // The C raises here. Returning quietly let the file system carry on caching
        // through a FileObject that holds no OpenCount on the shared map, so another
        // handle's close could delete the map -- and unmap its views -- under a reader.
        extern "C" { fn DbgPrint(fmt: *const i8, ...) -> u32; }
        DbgPrint(b"CcInitializeCacheMap: CcRosInitializeFileCache failed 0x%08x, raising\n\0".as_ptr() as _, status);
        ExRaiseStatus(status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcIsThereDirtyData(Vpb: PVPB) -> BOOLEAN {
    let Vpb = opaque_ptr(Vpb);
    let mut dirty = 0u8;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);

    let mut entry = DirtyVacbListHead.Flink;
    while entry != core::ptr::addr_of_mut!(DirtyVacbListHead) as PLIST_ENTRY {
        let vacb = (entry as *const u8).offset(-(core::mem::offset_of!(ROS_VACB, DirtyVacbListEntry) as isize)) as PROS_VACB;

        if (*(*vacb).SharedCacheMap.as_ref().unwrap().FileObject).Vpb != Vpb {
            entry = (*entry).Flink;
            continue;
        }

        if ((*(*vacb).SharedCacheMap.as_ref().unwrap().FileObject).Flags & FO_TEMPORARY_FILE) != 0 {
            entry = (*entry).Flink;
            continue;
        }

        if (*vacb).Dirty != 0 {
            dirty = 1u8;
            break;
        }

        entry = (*entry).Flink;
    }

    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    dirty
}

#[no_mangle]
pub unsafe extern "system" fn CcPurgeCacheSection(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    UninitializeCacheMaps: BOOLEAN,
) -> BOOLEAN {
    let FileOffset = opaque_ptr(FileOffset);
    let shared_cache_map = (*SectionObjectPointer).SharedCacheMap;
    if shared_cache_map.is_null() {
        return MmPurgeSegment(SectionObjectPointer, FileOffset, Length) as BOOLEAN;
    }

    let shared_cache_map = shared_cache_map as PROS_SHARED_CACHE_MAP;

    if UninitializeCacheMaps != 0 {
        while !IsListEmpty(core::ptr::addr_of_mut!((*shared_cache_map).PrivateList) as PLIST_ENTRY) {
            let private_cache_map = ((*shared_cache_map).PrivateList.Flink as *const u8)
                .offset(-(core::mem::offset_of!(PRIVATE_CACHE_MAP, PrivateLinks) as isize)) as PPRIVATE_CACHE_MAP;

            CcUninitializeCacheMap((*private_cache_map).FileObject, core::ptr::null_mut(), core::ptr::null_mut());
        }
    }

    let start_offset = if !FileOffset.is_null() {
        (*FileOffset).QuadPart
    } else {
        0
    };

    let end_offset = if Length == 0 || FileOffset.is_null() {
        MAXLONGLONG
    } else {
        start_offset + Length as i64
    };

    let mut free_list = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    InitializeListHead(core::ptr::addr_of_mut!(free_list) as PLIST_ENTRY);

    let mut success = 1u8;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMasterLock as _);
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);

    let mut list_entry = (*shared_cache_map).CacheMapVacbListHead.Flink;
    while list_entry != core::ptr::addr_of_mut!((*shared_cache_map).CacheMapVacbListHead) as PLIST_ENTRY {
        let vacb = (list_entry as *const u8)
            .offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        list_entry = (*list_entry).Flink;

        if (*vacb).FileOffset.QuadPart < start_offset {
            continue;
        }

        let view_end = core::cmp::min(
            (*vacb).FileOffset.QuadPart + VACB_MAPPING_GRANULARITY,
            (*shared_cache_map).SectionSize.QuadPart,
        );
        if view_end >= end_offset {
            break;
        }

        let refs = CcRosVacbGetRefCount(vacb);
        if (refs > 1 && (*vacb).Dirty == 0) || (refs > 2 && (*vacb).Dirty != 0) {
            success = 0u8;
            break;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*vacb).VacbLruListEntry) as PLIST_ENTRY);
        InitializeListHead(core::ptr::addr_of_mut!((*vacb).VacbLruListEntry) as PLIST_ENTRY);
        if (*vacb).Dirty != 0 {
            CcRosUnmarkDirtyVacb(vacb, 0u8);
        }
        RemoveEntryList(core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry) as PLIST_ENTRY);
        InsertHeadList(
            core::ptr::addr_of_mut!(free_list) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry) as PLIST_ENTRY,
        );
    }

    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _);
    KeReleaseQueuedSpinLock(LockQueueMasterLock as _, old_irql);

    while !IsListEmpty(core::ptr::addr_of_mut!(free_list) as PLIST_ENTRY) {
        let vacb = (RemoveHeadList(core::ptr::addr_of_mut!(free_list) as PLIST_ENTRY) as *const u8)
            .offset(-(core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry) as isize)) as PROS_VACB;
        InitializeListHead(core::ptr::addr_of_mut!((*vacb).CacheMapVacbListEntry) as PLIST_ENTRY);
        CcRosVacbDecRefCount(vacb);
    }

    if success != 0 {
        success = MmPurgeSegment(SectionObjectPointer, FileOffset, Length) as BOOLEAN;
    }

    success
}

#[no_mangle]
pub unsafe extern "system" fn CcSetFileSizes(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
) {
    let FileObject = opaque_ptr(FileObject);
    let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if shared_cache_map.is_null() {
        return;
    }

    let shared_cache_map = shared_cache_map as PROS_SHARED_CACHE_MAP;

    let mut old_irql: KIRQL = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _, &mut old_irql);
    let old_section_size = (*shared_cache_map).SectionSize;
    (*shared_cache_map).SectionSize = (*FileSizes).AllocationSize;
    (*shared_cache_map).FileSize = (*FileSizes).FileSize;
    (*shared_cache_map).ValidDataLength = (*FileSizes).ValidDataLength;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _, old_irql);

    if (*FileSizes).AllocationSize.QuadPart < old_section_size.QuadPart {
        CcPurgeCacheSection(
            (*FileObject).SectionObjectPointer,
            core::ptr::addr_of_mut!((*FileSizes).AllocationSize),
            0,
            0u8,
        );
    } else {
        MmExtendSection(
            (*shared_cache_map).Section,
            core::ptr::addr_of_mut!((*shared_cache_map).SectionSize),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcSetLogHandleForFile(
    _FileObject: PFILE_OBJECT,
    _LogHandle: PVOID,
    _FlushToLsnRoutine: PFLUSH_TO_LSN,
) {
    let _FileObject = opaque_ptr(_FileObject);
    let _LogHandle = opaque_ptr(_LogHandle);
}

#[no_mangle]
pub unsafe extern "system" fn CcUninitializeCacheMap(
    FileObject: PFILE_OBJECT,
    TruncateSize: PLARGE_INTEGER,
    UninitializeCompleteEvent: PCACHE_UNINITIALIZE_EVENT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let TruncateSize = opaque_ptr(TruncateSize);
    if !TruncateSize.is_null() && !(*(*FileObject).SectionObjectPointer).SharedCacheMap.is_null() {
        let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
        let mut old_irql: KIRQL = 0;
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _, &mut old_irql);
        if (*shared_cache_map).FileSize.QuadPart > (*TruncateSize).QuadPart {
            (*shared_cache_map).FileSize = *TruncateSize;
        }
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*shared_cache_map).CacheMapLock) as *mut _, old_irql);
        CcPurgeCacheSection(
            (*FileObject).SectionObjectPointer,
            TruncateSize,
            0,
            0u8,
        );
    }

    let status = CcRosReleaseFileCache(FileObject);
    if !UninitializeCompleteEvent.is_null() {
        KeSetEvent(
            core::ptr::addr_of_mut!((*UninitializeCompleteEvent).Event),
            IO_NO_INCREMENT,
            0u8,
        );
    }
    (status >= 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileSizes(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let shared_cache_map = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if shared_cache_map.is_null() {
        return 0u8;
    }

    let shared_cache_map = shared_cache_map as PROS_SHARED_CACHE_MAP;
    (*FileSizes).AllocationSize = (*shared_cache_map).SectionSize;
    (*FileSizes).FileSize = (*shared_cache_map).FileSize;
    (*FileSizes).ValidDataLength = (*shared_cache_map).FileSize;
    1u8
}
