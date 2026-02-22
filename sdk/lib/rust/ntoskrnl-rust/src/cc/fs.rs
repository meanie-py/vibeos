use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{CcRosInitializeFileCache, CcRosReleaseFileCache, CcRosUnmarkDirtyVacb, INTERNAL_BCB, KeAcquireSpinLockAtDpcLevel, KeReleaseSpinLockFromDpcLevel, LARGE_INTEGER, LIST_ENTRY, MmExtendSection, MmPurgeSegment, PCACHE_MANAGER_CALLBACKS, PCACHE_UNINITIALIZE_EVENT, PCC_FILE_SIZES, PDIRTY_PAGE_ROUTINE, PFILE_OBJECT, PFLUSH_TO_LSN, PINTERNAL_BCB, PLARGE_INTEGER, PPRIVATE_CACHE_MAP, PRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP, PROS_VACB, PSECTION_OBJECT_POINTERS, PVPB, ROS_VACB};
use core::ffi::c_void;

extern "C" {
    fn CCTRACE(level: u32, format: *const u8, ...);
    fn CcRosVacbDecRefCount(Vacb: PROS_VACB) -> u32;
    fn CcRosVacbGetRefCount(Vacb: PROS_VACB) -> u32;
    fn min(a: i64, b: i64) -> i64;
}
extern "C" {
    fn BooleanFlagOn(flags: u32, mask: u32) -> u8;
}

#[no_mangle]
pub unsafe extern "system" fn CcGetDirtyPages(
    LogHandle: *mut core::ffi::c_void,
    DirtyPageRoutine: PDIRTY_PAGE_ROUTINE,
    Context1: *mut core::ffi::c_void,
    Context2: *mut core::ffi::c_void,
) -> LARGE_INTEGER {

    CCTRACE(
        1,
        b"LogHandle=%p DirtyPageRoutine=%p Context1=%p Context2=%p\n\0".as_ptr(),
        LogHandle,
        DirtyPageRoutine,
        Context1,
        Context2,
    );

    let mut i = LARGE_INTEGER { QuadPart: 0 };
    i.QuadPart = 0;
    i
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileObjectFromBcb(
    Bcb: *mut core::ffi::c_void
) -> PFILE_OBJECT {
    let iBcb = (Bcb as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;
    
    
    CCTRACE(0x80000000u32, b"Bcb=%p\n\0".as_ptr() as *const i8, Bcb);
    
    (*(*(*iBcb).Vacb).SharedCacheMap).FileObject
}

#[no_mangle]
pub unsafe extern "system" fn CcGetLsnForFileObject(
    FileObject: PFILE_OBJECT,
    OldestLsn: PLARGE_INTEGER,
) -> LARGE_INTEGER {
    let mut i: LARGE_INTEGER = core::mem::zeroed();
    
    i.QuadPart = 0;
    i
}

#[no_mangle]
pub unsafe extern "system" fn CcInitializeCacheMap(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
    PinAccess: BOOLEAN,
    CallBacks: PCACHE_MANAGER_CALLBACKS,
    LazyWriterContext: PVOID,
) {
    let Status: NTSTATUS;

    const CC_API_DEBUG: u32 = 0;

    CCTRACE(
        CC_API_DEBUG,
        b"FileObject=%p FileSizes=%p PinAccess=%d CallBacks=%p LazyWriterContext=%p\n\0".as_ptr() as *const i8,
        FileObject,
        FileSizes,
        PinAccess,
        CallBacks,
        LazyWriterContext,
    );

    Status = CcRosInitializeFileCache(
        FileObject,
        FileSizes,
        PinAccess,
        CallBacks,
        LazyWriterContext,
    );
    if NT_SUCCESS(Status) == 0 {
        ExRaiseStatus(Status);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcIsThereDirtyData(Vpb: PVPB) -> u8 {
    let mut Vacb: PROS_VACB;
    let mut Entry: PLIST_ENTRY;
    let oldIrql: KIRQL;
    let mut Dirty: u8 = 0u8;

    extern {
        static mut DirtyVacbListHead: LIST_ENTRY;
        static 17i32: u32;
        static FO_TEMPORARY_FILE: u32;
    }

    CCTRACE(0x80, b"Vpb=%p\n\0".as_ptr() as *const i8, Vpb);

    oldIrql = KeAcquireQueuedSpinLock(17i32);

    Entry = DirtyVacbListHead.Flink;
    while Entry != core::ptr::addr_of_mut!(DirtyVacbListHead) {
        Vacb = (Entry as *mut u8).offset(-(core::mem::offset_of!(ROS_VACB, DirtyVacbListEntry) as isize)) as PROS_VACB;
        
        if (*(*Vacb).SharedCacheMap).FileObject as *mut u8 != core::ptr::null_mut() &&
           (*(*(*Vacb).SharedCacheMap).FileObject).Vpb != Vpb {
            Entry = (*Entry).Flink;
            continue;
        }

        if BooleanFlagOn((*(*(*Vacb).SharedCacheMap).FileObject).Flags, FO_TEMPORARY_FILE) != 0 {
            Entry = (*Entry).Flink;
            continue;
        }

        if (*Vacb).Dirty != 0 {
            Dirty = 1u8;
            break;
        }

        Entry = (*Entry).Flink;
    }

    KeReleaseQueuedSpinLock(17i32, oldIrql);

    Dirty
}

#[no_mangle]
pub unsafe extern "system" fn CcPurgeCacheSection(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    UninitializeCacheMaps: u8,
) -> u8 {
    let mut SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let mut PrivateCacheMap: PPRIVATE_CACHE_MAP;
    let StartOffset: i64;
    let EndOffset: i64;
    let mut FreeList: LIST_ENTRY = core::mem::zeroed();
    let mut OldIrql: KIRQL;
    let mut ListEntry: PLIST_ENTRY;
    let mut Vacb: PROS_VACB;
    let ViewEnd: i64;
    let mut Success: u8;

    CCTRACE(
        0x00000004,
        b"SectionObjectPointer=%p\n FileOffset=%p Length=%lu UninitializeCacheMaps=%d\0".as_ptr() as *const i8,
        SectionObjectPointer,
        FileOffset,
        Length,
        UninitializeCacheMaps,
    );

    SharedCacheMap = (*SectionObjectPointer).SharedCacheMap;
    if SharedCacheMap.is_null() {
        Success = 1u8;
        return 'purgeMm: {
            if Success != 0 {
                Success = MmPurgeSegment(SectionObjectPointer, FileOffset, Length);
            }
            Success
        };
    }

    if UninitializeCacheMaps != 0 {
        while IsListEmpty(core::ptr::addr_of_mut!((*SharedCacheMap).PrivateList)) == 0 {
            PrivateCacheMap = (((*SharedCacheMap).PrivateList.Flink as usize)
                - core::mem::offset_of!(PRIVATE_CACHE_MAP, PrivateLinks))
                as PPRIVATE_CACHE_MAP;

            CcUninitializeCacheMap((*PrivateCacheMap).FileObject, core::ptr::null_mut(), core::ptr::null_mut());
        }
    }

    StartOffset = if !FileOffset.is_null() {
        (*FileOffset).QuadPart
    } else {
        0
    };

    if Length == 0 || FileOffset.is_null() {
        EndOffset = 0x7FFFFFFFFFFFFFFFi64;
    } else {
        EndOffset = StartOffset + Length as i64;
    }

    InitializeListHead(core::ptr::addr_of_mut!(FreeList));

    Success = 1u8;

    OldIrql = KeAcquireQueuedSpinLock(0);
    KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    ListEntry = (*SharedCacheMap).CacheMapVacbListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapVacbListHead) {
        let Refs: u32;

        Vacb = ((ListEntry as usize) - core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry)) as PROS_VACB;
        ListEntry = (*ListEntry).Flink;

        if (*Vacb).FileOffset.QuadPart < StartOffset {
            continue;
        }

        ViewEnd = min(
            (*Vacb).FileOffset.QuadPart + 262144i64,
            (*SharedCacheMap).SectionSize.QuadPart,
        );
        if ViewEnd >= EndOffset {
            break;
        }

        Refs = CcRosVacbGetRefCount(Vacb);
        if (Refs > 1 && (*Vacb).Dirty == 0) || (Refs > 2 && (*Vacb).Dirty != 0) {
            Success = 0u8;
            break;
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry));
        InitializeListHead(core::ptr::addr_of_mut!((*Vacb).VacbLruListEntry));
        if (*Vacb).Dirty != 0 {
            CcRosUnmarkDirtyVacb(Vacb, 0u8);
        }
        RemoveEntryList(core::ptr::addr_of_mut!((*Vacb).CacheMapVacbListEntry));
        InsertHeadList(
            core::ptr::addr_of_mut!(FreeList),
            core::ptr::addr_of_mut!((*Vacb).CacheMapVacbListEntry),
        );
    }
    KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock));
    KeReleaseQueuedSpinLock(0, OldIrql);

    while IsListEmpty(core::ptr::addr_of_mut!(FreeList)) == 0 {
        let Refs: u32;

        Vacb = ((RemoveHeadList(core::ptr::addr_of_mut!(FreeList)) as usize)
            - core::mem::offset_of!(ROS_VACB, CacheMapVacbListEntry)) as PROS_VACB;
        InitializeListHead(core::ptr::addr_of_mut!((*Vacb).CacheMapVacbListEntry));
        Refs = CcRosVacbDecRefCount(Vacb);
    }

    if Success != 0 {
        Success = MmPurgeSegment(SectionObjectPointer, FileOffset, Length);
    }

    Success
}

#[no_mangle]
pub unsafe extern "system" fn CcSetFileSizes(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
) {
    let mut OldIrql: KIRQL;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let mut OldSectionSize: LARGE_INTEGER;

    CCTRACE(1, b"FileObject=%p FileSizes=%p\n\0".as_ptr() as *const i8, FileObject, FileSizes);

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if SharedCacheMap == core::ptr::null_mut() {
        return;
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock), core::ptr::addr_of_mut!(OldIrql));
    OldSectionSize = (*SharedCacheMap).SectionSize;
    (*SharedCacheMap).SectionSize = (*FileSizes).AllocationSize;
    (*SharedCacheMap).FileSize = (*FileSizes).FileSize;
    (*SharedCacheMap).ValidDataLength = (*FileSizes).ValidDataLength;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock), OldIrql);

    if (*FileSizes).AllocationSize.QuadPart < OldSectionSize.QuadPart {
        CcPurgeCacheSection(
            (*FileObject).SectionObjectPointer,
            core::ptr::addr_of_mut!((*FileSizes).AllocationSize),
            0,
            0u8,
        );
    } else {
        MmExtendSection((*SharedCacheMap).Section, core::ptr::addr_of_mut!((*SharedCacheMap).SectionSize));
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcSetLogHandleForFile(
    FileObject: PFILE_OBJECT,
    LogHandle: *mut core::ffi::c_void,
    FlushToLsnRoutine: PFLUSH_TO_LSN,
) {
    
    CCTRACE(
        0x80000000u32,
        b"FileObject=%p LogHandle=%p FlushToLsnRoutine=%p\n\0".as_ptr(),
        FileObject,
        LogHandle,
        FlushToLsnRoutine,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CcUninitializeCacheMap(
    FileObject: PFILE_OBJECT,
    TruncateSize: PLARGE_INTEGER,
    UninitializeCompleteEvent: PCACHE_UNINITIALIZE_EVENT,
) -> u8 {
    let Status: i32;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let mut OldIrql: KIRQL = core::mem::zeroed();

    if !TruncateSize.is_null() && !(*(*FileObject).SectionObjectPointer).SharedCacheMap.is_null() {
        SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PROS_SHARED_CACHE_MAP;
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock), core::ptr::addr_of_mut!(OldIrql));
        if (*SharedCacheMap).FileSize.QuadPart > (*TruncateSize).QuadPart {
            (*SharedCacheMap).FileSize = *TruncateSize;
        }
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*SharedCacheMap).CacheMapLock), OldIrql);
        CcPurgeCacheSection(
            (*FileObject).SectionObjectPointer,
            TruncateSize,
            0,
            0u8,
        );
    }

    Status = CcRosReleaseFileCache(FileObject);
    if !UninitializeCompleteEvent.is_null() {
        KeSetEvent(core::ptr::addr_of_mut!((*UninitializeCompleteEvent).Event), 0, 0u8);
    }
    NT_SUCCESS(Status) as u8
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFileSizes(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
) -> u8 {
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    if SharedCacheMap.is_null() {
        return 0u8;
    }

    (*FileSizes).AllocationSize = (*SharedCacheMap).SectionSize;
    (*FileSizes).FileSize = (*SharedCacheMap).FileSize;
    (*FileSizes).ValidDataLength = (*SharedCacheMap).FileSize;
    return 1u8;
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/fs.c
//  * PURPOSE:         Implements cache managers functions useful for File Systems
//  *
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @unimplemented
//  */
// LARGE_INTEGER
// NTAPI
// CcGetDirtyPages (
//     IN PVOID LogHandle,
//     IN PDIRTY_PAGE_ROUTINE DirtyPageRoutine,
//     IN PVOID Context1,
//     IN PVOID Context2)
// {
//     LARGE_INTEGER i;
//
//     CCTRACE(CC_API_DEBUG, "LogHandle=%p DirtyPageRoutine=%p Context1=%p Context2=%p\n",
//         LogHandle, DirtyPageRoutine, Context1, Context2);
//
//     UNIMPLEMENTED;
//     i.QuadPart = 0;
//     return i;
// }
//
// /*
//  * @implemented
//  */
// PFILE_OBJECT
// NTAPI
// CcGetFileObjectFromBcb (
//     IN PVOID Bcb)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p\n", Bcb);
//
//     return iBcb->Vacb->SharedCacheMap->FileObject;
// }
//
// /*
//  * @unimplemented
//  */
// LARGE_INTEGER
// NTAPI
// CcGetLsnForFileObject (
//     IN PFILE_OBJECT FileObject,
//     OUT PLARGE_INTEGER OldestLsn OPTIONAL)
// {
//     LARGE_INTEGER i;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p\n", FileObject);
//
//     UNIMPLEMENTED;
//     i.QuadPart = 0;
//     return i;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcInitializeCacheMap (
//     IN PFILE_OBJECT FileObject,
//     IN PCC_FILE_SIZES FileSizes,
//     IN BOOLEAN PinAccess,
//     IN PCACHE_MANAGER_CALLBACKS CallBacks,
//     IN PVOID LazyWriterContext)
// {
//     NTSTATUS Status;
//
//     ASSERT(FileObject);
//     ASSERT(FileSizes);
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileSizes=%p PinAccess=%d CallBacks=%p LazyWriterContext=%p\n",
//         FileObject, FileSizes, PinAccess, CallBacks, LazyWriterContext);
//
//     /* Call old ROS cache init function */
//     Status = CcRosInitializeFileCache(FileObject,
//                                       FileSizes,
//                                       PinAccess,
//                                       CallBacks,
//                                       LazyWriterContext);
//     if (!NT_SUCCESS(Status))
//         ExRaiseStatus(Status);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// CcIsThereDirtyData (
//     IN PVPB Vpb)
// {
//     PROS_VACB Vacb;
//     PLIST_ENTRY Entry;
//     KIRQL oldIrql;
//     /* Assume no dirty data */
//     BOOLEAN Dirty = FALSE;
//
//     CCTRACE(CC_API_DEBUG, "Vpb=%p\n", Vpb);
//
//     oldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     /* Browse dirty VACBs */
//     for (Entry = DirtyVacbListHead.Flink; Entry != &DirtyVacbListHead; Entry = Entry->Flink)
//     {
//         Vacb = CONTAINING_RECORD(Entry, ROS_VACB, DirtyVacbListEntry);
//         /* Look for these associated with our volume */
//         if (Vacb->SharedCacheMap->FileObject->Vpb != Vpb)
//         {
//             continue;
//         }
//
//         /* From now on, we are associated with our VPB */
//
//         /* Temporary files are not counted as dirty */
//         if (BooleanFlagOn(Vacb->SharedCacheMap->FileObject->Flags, FO_TEMPORARY_FILE))
//         {
//             continue;
//         }
//
//         /* A single dirty VACB is enough to have dirty data */
//         if (Vacb->Dirty)
//         {
//             Dirty = TRUE;
//             break;
//         }
//     }
//
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, oldIrql);
//
//     return Dirty;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcPurgeCacheSection (
//     IN PSECTION_OBJECT_POINTERS SectionObjectPointer,
//     IN PLARGE_INTEGER FileOffset OPTIONAL,
//     IN ULONG Length,
//     IN BOOLEAN UninitializeCacheMaps)
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     PPRIVATE_CACHE_MAP PrivateCacheMap;
//     LONGLONG StartOffset;
//     LONGLONG EndOffset;
//     LIST_ENTRY FreeList;
//     KIRQL OldIrql;
//     PLIST_ENTRY ListEntry;
//     PROS_VACB Vacb;
//     LONGLONG ViewEnd;
//     BOOLEAN Success;
//
//     CCTRACE(CC_API_DEBUG, "SectionObjectPointer=%p\n FileOffset=%p Length=%lu UninitializeCacheMaps=%d",
//         SectionObjectPointer, FileOffset, Length, UninitializeCacheMaps);
//
//     /* Obtain the shared cache from the section */
//     SharedCacheMap = SectionObjectPointer->SharedCacheMap;
//     if (!SharedCacheMap)
//     {
//         Success = TRUE;
//         goto purgeMm;
//     }
//
//     if (UninitializeCacheMaps)
//     {
//         /*
//          * We have gotten the acknowledgement that
//          * the caller wants to unintialize the private
//          * cache maps so let's do this. Since we already
//          * have the shared cache map from above, iterate
//          * over that cache's private lists.
//          */
//         while (!IsListEmpty(&SharedCacheMap->PrivateList))
//         {
//             /*
//              * This list is not empty, grab the
//              * private cache map.
//              */
//             PrivateCacheMap = CONTAINING_RECORD(SharedCacheMap->PrivateList.Flink, PRIVATE_CACHE_MAP, PrivateLinks);
//
//             /* Unintialize the private cache now */
//             CcUninitializeCacheMap(PrivateCacheMap->FileObject, NULL, NULL);
//         }
//     }
//
//     StartOffset = FileOffset != NULL ? FileOffset->QuadPart : 0;
//     if (Length == 0 || FileOffset == NULL)
//     {
//         EndOffset = MAXLONGLONG;
//     }
//     else
//     {
//         EndOffset = StartOffset + Length;
//         ASSERT(EndOffset > StartOffset);
//     }
//
//     InitializeListHead(&FreeList);
//
//     /* Assume success */
//     Success = TRUE;
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//     KeAcquireSpinLockAtDpcLevel(&SharedCacheMap->CacheMapLock);
//     ListEntry = SharedCacheMap->CacheMapVacbListHead.Flink;
//     while (ListEntry != &SharedCacheMap->CacheMapVacbListHead)
//     {
//         ULONG Refs;
//
//         Vacb = CONTAINING_RECORD(ListEntry, ROS_VACB, CacheMapVacbListEntry);
//         ListEntry = ListEntry->Flink;
//
//         /* Skip VACBs outside the range, or only partially in range */
//         if (Vacb->FileOffset.QuadPart < StartOffset)
//         {
//             continue;
//         }
//         ViewEnd = min(Vacb->FileOffset.QuadPart + VACB_MAPPING_GRANULARITY,
//                       SharedCacheMap->SectionSize.QuadPart);
//         if (ViewEnd >= EndOffset)
//         {
//             break;
//         }
//
//         /* Still in use, it cannot be purged, fail
//          * Allow one ref: VACB is supposed to be always 1-referenced
//          */
//         Refs = CcRosVacbGetRefCount(Vacb);
//         if ((Refs > 1 && !Vacb->Dirty) ||
//             (Refs > 2 && Vacb->Dirty))
//         {
//             Success = FALSE;
//             break;
//         }
//
//         /* This VACB is in range, so unlink it and mark for free */
//         ASSERT(Refs == 1 || Vacb->Dirty);
//         RemoveEntryList(&Vacb->VacbLruListEntry);
//         InitializeListHead(&Vacb->VacbLruListEntry);
//         if (Vacb->Dirty)
//         {
//             CcRosUnmarkDirtyVacb(Vacb, FALSE);
//         }
//         RemoveEntryList(&Vacb->CacheMapVacbListEntry);
//         InsertHeadList(&FreeList, &Vacb->CacheMapVacbListEntry);
//     }
//     KeReleaseSpinLockFromDpcLevel(&SharedCacheMap->CacheMapLock);
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
//
//     while (!IsListEmpty(&FreeList))
//     {
//         ULONG Refs;
//
//         Vacb = CONTAINING_RECORD(RemoveHeadList(&FreeList),
//                                  ROS_VACB,
//                                  CacheMapVacbListEntry);
//         InitializeListHead(&Vacb->CacheMapVacbListEntry);
//         Refs = CcRosVacbDecRefCount(Vacb);
//         ASSERT(Refs == 0);
//     }
//
//     /* Now make sure that Mm doesn't hold some pages here. */
// purgeMm:
//     if (Success)
//         Success = MmPurgeSegment(SectionObjectPointer, FileOffset, Length);
//
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// VOID NTAPI
// CcSetFileSizes (
//     IN PFILE_OBJECT FileObject,
//     IN PCC_FILE_SIZES FileSizes)
// {
//     KIRQL OldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     LARGE_INTEGER OldSectionSize;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileSizes=%p\n",
//         FileObject, FileSizes);
//
//     DPRINT("CcSetFileSizes(FileObject 0x%p, FileSizes 0x%p)\n",
//            FileObject, FileSizes);
//     DPRINT("AllocationSize %I64d, FileSize %I64d, ValidDataLength %I64d\n",
//            FileSizes->AllocationSize.QuadPart,
//            FileSizes->FileSize.QuadPart,
//            FileSizes->ValidDataLength.QuadPart);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//     /*
//      * It is valid to call this function on file objects that weren't
//      * initialized for caching. In this case it's simple no-op.
//      */
//     if (SharedCacheMap == NULL)
//         return;
//
//     /* Update the relevant fields */
//     KeAcquireSpinLock(&SharedCacheMap->CacheMapLock, &OldIrql);
//     OldSectionSize = SharedCacheMap->SectionSize;
//     SharedCacheMap->SectionSize = FileSizes->AllocationSize;
//     SharedCacheMap->FileSize = FileSizes->FileSize;
//     SharedCacheMap->ValidDataLength = FileSizes->ValidDataLength;
//     KeReleaseSpinLock(&SharedCacheMap->CacheMapLock, OldIrql);
//
//     if (FileSizes->AllocationSize.QuadPart < OldSectionSize.QuadPart)
//     {
//         CcPurgeCacheSection(FileObject->SectionObjectPointer,
//                             &FileSizes->AllocationSize,
//                             0,
//                             FALSE);
//     }
//     else
//     {
//         /* Extend our section object */
//         MmExtendSection(SharedCacheMap->Section, &SharedCacheMap->SectionSize);
//     }
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcSetLogHandleForFile (
//     IN PFILE_OBJECT FileObject,
//     IN PVOID LogHandle,
//     IN PFLUSH_TO_LSN FlushToLsnRoutine)
// {
//     CCTRACE(CC_API_DEBUG, "FileObject=%p LogHandle=%p FlushToLsnRoutine=%p\n",
//         FileObject, LogHandle, FlushToLsnRoutine);
//
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// CcUninitializeCacheMap (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER TruncateSize OPTIONAL,
//     IN PCACHE_UNINITIALIZE_EVENT UninitializeCompleteEvent OPTIONAL)
// {
//     NTSTATUS Status;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     KIRQL OldIrql;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p TruncateSize=%p UninitializeCompleteEvent=%p\n",
//         FileObject, TruncateSize, UninitializeCompleteEvent);
//
//     if (TruncateSize != NULL &&
//         FileObject->SectionObjectPointer->SharedCacheMap != NULL)
//     {
//         SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//         KeAcquireSpinLock(&SharedCacheMap->CacheMapLock, &OldIrql);
//         if (SharedCacheMap->FileSize.QuadPart > TruncateSize->QuadPart)
//         {
//             SharedCacheMap->FileSize = *TruncateSize;
//         }
//         KeReleaseSpinLock(&SharedCacheMap->CacheMapLock, OldIrql);
//         CcPurgeCacheSection(FileObject->SectionObjectPointer,
//                             TruncateSize,
//                             0,
//                             FALSE);
//     }
//
//     Status = CcRosReleaseFileCache(FileObject);
//     if (UninitializeCompleteEvent)
//     {
//         KeSetEvent(&UninitializeCompleteEvent->Event, IO_NO_INCREMENT, FALSE);
//     }
//     return NT_SUCCESS(Status);
// }
//
// BOOLEAN
// NTAPI
// CcGetFileSizes (
//     IN PFILE_OBJECT FileObject,
//     IN PCC_FILE_SIZES FileSizes)
// {
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//     if (!SharedCacheMap)
//         return FALSE;
//
//     FileSizes->AllocationSize = SharedCacheMap->SectionSize;
//     FileSizes->FileSize = FileSizes->ValidDataLength = SharedCacheMap->FileSize;
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
