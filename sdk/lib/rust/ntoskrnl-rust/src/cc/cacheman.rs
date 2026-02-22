use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcInitView, CcPostWorkQueue, CcScanDpc, CcWorkerThread, ExInitializeFastMutex, ExInitializeNPagedLookasideList, ExIsResourceAcquiredExclusiveLite, ExIsResourceAcquiredSharedLite, ExSetResourceOwnerPointer, INTERNAL_BCB, KeInitializeDpc, KeInitializeTimer, LARGE_INTEGER, MmQuerySystemSize, PFILE_OBJECT, PFSN_PREFETCHER_GLOBALS, PFSRTL_COMMON_FCB_HEADER, PINTERNAL_BCB, PLARGE_INTEGER, PPRIVATE_CACHE_MAP, PROS_SHARED_CACHE_MAP, PSECTION_OBJECT_POINTERS, PWORK_QUEUE_ENTRY, PWORK_QUEUE_ITEM, WORK_QUEUE_ENTRY, WORK_QUEUE_ITEM};
use core::ffi::c_void;

extern "C" {
    fn CCTRACE(level: u32, fmt: *const i8, ...);
    fn SetFlag(flags: *mut u32, flag: u32);
    fn ClearFlag(flags: *mut u32, flag: u32);
    fn BooleanFlagOn(flags: u32, flag: u32) -> u8;
}
extern "C" {
    fn DbgPrintEx(ComponentId: u32, Level: u32, Format: *const i8, ...);
    fn CcBugCheck(a: u32, b: u32, c: u32);
    fn ExInitializeWorkItem(Item: PWORK_QUEUE_ITEM, Routine: Option<unsafe extern "system" fn(*mut core::ffi::c_void)>, Context: *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn CcPfInitializePrefetcher() {

    DbgPrintEx(
        96,
        3,
        b"CCPF: InitializePrefetecher()\n\0".as_ptr() as *const i8,
    );

    InitializeListHead(&mut CcPfGlobals.ActiveTraces);
    InitializeListHead(&mut CcPfGlobals.CompletedTraces);
    ExInitializeFastMutex(&mut CcPfGlobals.CompletedTracesLock);
}

#[no_mangle]
pub unsafe extern "system" fn CcInitializeCacheManager() -> u8 {
    let mut Thread: u32;

    CcInitView();

    InitializeListHead(core::ptr::addr_of_mut!(CcIdleWorkerThreadList));
    InitializeListHead(core::ptr::addr_of_mut!(CcExpressWorkQueue));
    InitializeListHead(core::ptr::addr_of_mut!(CcRegularWorkQueue));
    InitializeListHead(core::ptr::addr_of_mut!(CcPostTickWorkQueue));

    CcCapturedSystemSize = MmQuerySystemSize();
    match CcCapturedSystemSize {
        0 => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
            CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8;
        }
        1 => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
            CcDirtyPageThreshold = MmNumberOfPhysicalPages / 4;
        }
        2 => {
            CcNumberWorkerThreads = ExCriticalWorkerThreads - 2;
            CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8 + MmNumberOfPhysicalPages / 4;
        }
        _ => {
            CcNumberWorkerThreads = 1;
            CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8;
        }
    }

    Thread = 0;
    while Thread < CcNumberWorkerThreads {
        let Item: PWORK_QUEUE_ITEM;

        Item = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<WORK_QUEUE_ITEM>() as u32, 0x71576343) as PWORK_QUEUE_ITEM;
        if Item == core::ptr::null_mut() {
            CcBugCheck(0, 0, 0);
        }

        ExInitializeWorkItem(Item, Some(CcWorkerThread), Item as *mut core::ffi::c_void);
        InsertTailList(core::ptr::addr_of_mut!(CcIdleWorkerThreadList), core::ptr::addr_of_mut!((*Item).List));

        Thread += 1;
    }

    core::ptr::write_bytes(core::ptr::addr_of_mut!(LazyWriter), 0, 1);
    InitializeListHead(core::ptr::addr_of_mut!(LazyWriter.WorkQueue));
    KeInitializeDpc(core::ptr::addr_of_mut!(LazyWriter.ScanDpc), Some(CcScanDpc), core::ptr::null_mut());
    KeInitializeTimer(core::ptr::addr_of_mut!(LazyWriter.ScanTimer));

    ExInitializeNPagedLookasideList(
        core::ptr::addr_of_mut!(CcTwilightLookasideList),
        None,
        None,
        0,
        core::mem::size_of::<WORK_QUEUE_ENTRY>() as u32,
        0x4B576343,
        0
    );

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CcShutdownSystem() {
    /* NOTHING TO DO */
}

#[no_mangle]
pub unsafe extern "system" fn CcGetFlushedValidData(
    SectionObjectPointer: PSECTION_OBJECT_POINTERS,
    BcbListHeld: BOOLEAN,
) -> LARGE_INTEGER {
    let mut i: LARGE_INTEGER = core::mem::zeroed();
    
    i.QuadPart = 0;
    i
}

#[no_mangle]
pub unsafe extern "system" fn CcRemapBcb(
    Bcb: PVOID
) -> PVOID
{
    unimplemented!();
    
    0 as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn CcScheduleReadAhead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
) {
    let mut OldIrql: KIRQL = 0;
    let mut NewOffset: LARGE_INTEGER = core::mem::zeroed();
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;
    let PrivateCacheMap: PPRIVATE_CACHE_MAP;
    let mut Length = Length;

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    PrivateCacheMap = (*FileObject).PrivateCacheMap;

    if SharedCacheMap.is_null() || PrivateCacheMap.is_null() ||
        BooleanFlagOn((*SharedCacheMap).Flags, READAHEAD_DISABLED) != 0
    {
        return;
    }

    Length = ROUND_UP(Length, (*PrivateCacheMap).ReadAheadMask + 1);
    *(*NewOffset.QuadPart_mut()) = *(*FileOffset).QuadPart() + Length as i64;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), core::ptr::addr_of_mut!(OldIrql));
    
    if BooleanFlagOn((*FileObject).Flags, FO_SEQUENTIAL_ONLY) != 0 {
        if *(*NewOffset.QuadPart()) < *(*(*PrivateCacheMap).ReadAheadOffset[1]).QuadPart() {
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), OldIrql);
            return;
        }

        *(*(*PrivateCacheMap).ReadAheadOffset[1]).QuadPart_mut() = *(*NewOffset.QuadPart());
        (*PrivateCacheMap).ReadAheadLength[1] = Length;
    } else {
        if *(*(*PrivateCacheMap).FileOffset2).QuadPart() >= *(*(*PrivateCacheMap).FileOffset1).QuadPart() &&
            *(*FileOffset).QuadPart() >= *(*(*PrivateCacheMap).FileOffset2).QuadPart()
        {
            *(*(*PrivateCacheMap).ReadAheadOffset[1]).QuadPart_mut() = *(*NewOffset.QuadPart());
            (*PrivateCacheMap).ReadAheadLength[1] = Length;
        } else {
            KeReleaseSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), OldIrql);
            UNIMPLEMENTED_ONCE();
            return;
        }
    }

    if (*PrivateCacheMap).Flags.ReadAheadActive() == 0 {
        let WorkItem: PWORK_QUEUE_ENTRY;

        InterlockedOr((core::ptr::addr_of_mut!((*PrivateCacheMap).UlongFlags)) as *mut i32, PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE as i32);
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), OldIrql);

        WorkItem = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(CcTwilightLookasideList));
        if !WorkItem.is_null() {
            ObReferenceObject(FileObject as *mut core::ffi::c_void);

            (*WorkItem).Function = ReadAhead;
            (*WorkItem).Parameters.Read.FileObject = FileObject;

            CcPostWorkQueue(WorkItem, core::ptr::addr_of_mut!(CcExpressWorkQueue));

            return;
        }

        KeAcquireSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), core::ptr::addr_of_mut!(OldIrql));
        InterlockedAnd((core::ptr::addr_of_mut!((*PrivateCacheMap).UlongFlags)) as *mut i32, !PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE as i32);
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*PrivateCacheMap).ReadAheadSpinLock), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn CcSetAdditionalCacheAttributes(
    FileObject: PFILE_OBJECT,
    DisableReadAhead: u8,
    DisableWriteBehind: u8,
) {
    let mut OldIrql: KIRQL;
    let SharedCacheMap: PROS_SHARED_CACHE_MAP;

    const CC_API_DEBUG: u32 = 0;
    const READAHEAD_DISABLED: u32 = 0x1;
    const WRITEBEHIND_DISABLED: u32 = 0x2;
    const 17i32: u32 = 0;

    CCTRACE(
        CC_API_DEBUG,
        b"FileObject=%p DisableReadAhead=%d DisableWriteBehind=%d\n\0".as_ptr() as *const i8,
        FileObject,
        DisableReadAhead,
        DisableWriteBehind,
    );

    SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;

    OldIrql = KeAcquireQueuedSpinLock(17i32);

    if DisableReadAhead != 0 {
        SetFlag(core::ptr::addr_of_mut!((*SharedCacheMap).Flags), READAHEAD_DISABLED);
    } else {
        ClearFlag(core::ptr::addr_of_mut!((*SharedCacheMap).Flags), READAHEAD_DISABLED);
    }

    if DisableWriteBehind != 0 {
        SetFlag(core::ptr::addr_of_mut!((*SharedCacheMap).Flags), WRITEBEHIND_DISABLED);
    } else {
        ClearFlag(core::ptr::addr_of_mut!((*SharedCacheMap).Flags), WRITEBEHIND_DISABLED);
    }

    KeReleaseQueuedSpinLock(17i32, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn CcSetBcbOwnerPointer(
    Bcb: PVOID,
    Owner: PVOID,
) {
    let iBcb = (Bcb as *mut u8).offset(-(core::mem::offset_of!(INTERNAL_BCB, PFCB) as isize)) as PINTERNAL_BCB;

    CCTRACE(CC_API_DEBUG, b"Bcb=%p Owner=%p\n\0".as_ptr() as PCHAR, Bcb, Owner);

    if ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*iBcb).Lock)) == 0 && 
       ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!((*iBcb).Lock)) == 0 {
        // DPRINT removed
        return;
    }

    ExSetResourceOwnerPointer(core::ptr::addr_of_mut!((*iBcb).Lock), Owner);
}

#[no_mangle]
pub unsafe extern "system" fn CcSetDirtyPageThreshold(
    FileObject: PFILE_OBJECT,
    DirtyPageThreshold: u32,
) {

    const CC_API_DEBUG: u32 = 0x00000001;
    const FSRTL_FLAG_LIMIT_MODIFIED_PAGES: u32 = 0x00000010;

    CCTRACE(
        CC_API_DEBUG,
        b"FileObject=%p DirtyPageThreshold=%lu\n\0".as_ptr() as *const i8,
        FileObject,
        DirtyPageThreshold,
    );

    let SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap;
    if !SharedCacheMap.is_null() {
        (*SharedCacheMap).DirtyPageThreshold = DirtyPageThreshold;
    }

    let Fcb: PFSRTL_COMMON_FCB_HEADER = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    if BooleanFlagOn((*Fcb).Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES) == 0 {
        SetFlag(core::ptr::addr_of_mut!((*Fcb).Flags), FSRTL_FLAG_LIMIT_MODIFIED_PAGES);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcSetReadAheadGranularity(
    FileObject: PFILE_OBJECT,
    Granularity: u32,
) {

    CCTRACE(
        0x00000001u32,
        b"FileObject=%p Granularity=%lu\n\0".as_ptr() as *const i8,
        FileObject,
        Granularity,
    );

    let PrivateMap: PPRIVATE_CACHE_MAP = (*FileObject).PrivateCacheMap;
    (*PrivateMap).ReadAheadMask = Granularity.wrapping_sub(1);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/cacheman.c
//  * PURPOSE:         Cache manager
//  *
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// BOOLEAN CcPfEnablePrefetcher;
// PFSN_PREFETCHER_GLOBALS CcPfGlobals;
// MM_SYSTEMSIZE CcCapturedSystemSize;
//
// static ULONG BugCheckFileId = 0x4 << 16;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CcPfInitializePrefetcher(VOID)
// {
//     /* Notify debugger */
//     DbgPrintEx(DPFLTR_PREFETCHER_ID,
//                DPFLTR_TRACE_LEVEL,
//                "CCPF: InitializePrefetecher()\n");
//
//     /* Setup the Prefetcher Data */
//     InitializeListHead(&CcPfGlobals.ActiveTraces);
//     InitializeListHead(&CcPfGlobals.CompletedTraces);
//     ExInitializeFastMutex(&CcPfGlobals.CompletedTracesLock);
//
//     /* FIXME: Setup the rest of the prefetecher */
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// CcInitializeCacheManager(VOID)
// {
//     ULONG Thread;
//
//     CcInitView();
//
//     /* Initialize lazy-writer lists */
//     InitializeListHead(&CcIdleWorkerThreadList);
//     InitializeListHead(&CcExpressWorkQueue);
//     InitializeListHead(&CcRegularWorkQueue);
//     InitializeListHead(&CcPostTickWorkQueue);
//
//     /* Define lazy writer threshold and the amount of workers,
//       * depending on the system type
//       */
//     CcCapturedSystemSize = MmQuerySystemSize();
//     switch (CcCapturedSystemSize)
//     {
//         case MmSmallSystem:
//             CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
//             CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8;
//             break;
//
//         case MmMediumSystem:
//             CcNumberWorkerThreads = ExCriticalWorkerThreads - 1;
//             CcDirtyPageThreshold = MmNumberOfPhysicalPages / 4;
//             break;
//
//         case MmLargeSystem:
//             CcNumberWorkerThreads = ExCriticalWorkerThreads - 2;
//             CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8 + MmNumberOfPhysicalPages / 4;
//             break;
//
//         default:
//             CcNumberWorkerThreads = 1;
//             CcDirtyPageThreshold = MmNumberOfPhysicalPages / 8;
//             break;
//     }
//
//     /* Allocate a work item for all our threads */
//     for (Thread = 0; Thread < CcNumberWorkerThreads; ++Thread)
//     {
//         PWORK_QUEUE_ITEM Item;
//
//         Item = ExAllocatePoolWithTag(NonPagedPool, sizeof(WORK_QUEUE_ITEM), 'qWcC');
//         if (Item == NULL)
//         {
//             CcBugCheck(0, 0, 0);
//         }
//
//         /* By default, it's obviously idle */
//         ExInitializeWorkItem(Item, CcWorkerThread, Item);
//         InsertTailList(&CcIdleWorkerThreadList, &Item->List);
//     }
//
//     /* Initialize our lazy writer */
//     RtlZeroMemory(&LazyWriter, sizeof(LazyWriter));
//     InitializeListHead(&LazyWriter.WorkQueue);
//     /* Delay activation of the lazy writer */
//     KeInitializeDpc(&LazyWriter.ScanDpc, CcScanDpc, NULL);
//     KeInitializeTimer(&LazyWriter.ScanTimer);
//
//     /* Lookaside list for our work items */
//     ExInitializeNPagedLookasideList(&CcTwilightLookasideList, NULL, NULL, 0, sizeof(WORK_QUEUE_ENTRY), 'KWcC', 0);
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// CcShutdownSystem(VOID)
// {
//     /* NOTHING TO DO */
// }
//
// /*
//  * @unimplemented
//  */
// LARGE_INTEGER
// NTAPI
// CcGetFlushedValidData (
//     IN PSECTION_OBJECT_POINTERS SectionObjectPointer,
//     IN BOOLEAN BcbListHeld
//     )
// {
// 	LARGE_INTEGER i;
//
// 	UNIMPLEMENTED;
//
// 	i.QuadPart = 0;
// 	return i;
// }
//
// /*
//  * @unimplemented
//  */
// PVOID
// NTAPI
// CcRemapBcb (
//     IN PVOID Bcb
//     )
// {
// 	UNIMPLEMENTED;
//
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcScheduleReadAhead (
// 	IN	PFILE_OBJECT		FileObject,
// 	IN	PLARGE_INTEGER		FileOffset,
// 	IN	ULONG			Length
// 	)
// {
//     KIRQL OldIrql;
//     LARGE_INTEGER NewOffset;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//     PPRIVATE_CACHE_MAP PrivateCacheMap;
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     PrivateCacheMap = FileObject->PrivateCacheMap;
//
//     /* If file isn't cached, or if read ahead is disabled, this is no op */
//     if (SharedCacheMap == NULL || PrivateCacheMap == NULL ||
//         BooleanFlagOn(SharedCacheMap->Flags, READAHEAD_DISABLED))
//     {
//         return;
//     }
//
//     /* Round read length with read ahead mask */
//     Length = ROUND_UP(Length, PrivateCacheMap->ReadAheadMask + 1);
//     /* Compute the offset we'll reach */
//     NewOffset.QuadPart = FileOffset->QuadPart + Length;
//
//     /* Lock read ahead spin lock */
//     KeAcquireSpinLock(&PrivateCacheMap->ReadAheadSpinLock, &OldIrql);
//     /* Easy case: the file is sequentially read */
//     if (BooleanFlagOn(FileObject->Flags, FO_SEQUENTIAL_ONLY))
//     {
//         /* If we went backward, this is no go! */
//         if (NewOffset.QuadPart < PrivateCacheMap->ReadAheadOffset[1].QuadPart)
//         {
//             KeReleaseSpinLock(&PrivateCacheMap->ReadAheadSpinLock, OldIrql);
//             return;
//         }
//
//         /* FIXME: hackish, but will do the job for now */
//         PrivateCacheMap->ReadAheadOffset[1].QuadPart = NewOffset.QuadPart;
//         PrivateCacheMap->ReadAheadLength[1] = Length;
//     }
//     /* Other cases: try to find some logic in that mess... */
//     else
//     {
//         /* Let's check if we always read the same way (like going down in the file)
//          * and pretend it's enough for now
//          */
//         if (PrivateCacheMap->FileOffset2.QuadPart >= PrivateCacheMap->FileOffset1.QuadPart &&
//             FileOffset->QuadPart >= PrivateCacheMap->FileOffset2.QuadPart)
//         {
//             /* FIXME: hackish, but will do the job for now */
//             PrivateCacheMap->ReadAheadOffset[1].QuadPart = NewOffset.QuadPart;
//             PrivateCacheMap->ReadAheadLength[1] = Length;
//         }
//         else
//         {
//             /* FIXME: handle the other cases */
//             KeReleaseSpinLock(&PrivateCacheMap->ReadAheadSpinLock, OldIrql);
//             UNIMPLEMENTED_ONCE;
//             return;
//         }
//     }
//
//     /* If read ahead isn't active yet */
//     if (!PrivateCacheMap->Flags.ReadAheadActive)
//     {
//         PWORK_QUEUE_ENTRY WorkItem;
//
//         /* It's active now!
//          * Be careful with the mask, you don't want to mess with node code
//          */
//         InterlockedOr((volatile long *)&PrivateCacheMap->UlongFlags, PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE);
//         KeReleaseSpinLock(&PrivateCacheMap->ReadAheadSpinLock, OldIrql);
//
//         /* Get a work item */
//         WorkItem = ExAllocateFromNPagedLookasideList(&CcTwilightLookasideList);
//         if (WorkItem != NULL)
//         {
//             /* Reference our FO so that it doesn't go in between */
//             ObReferenceObject(FileObject);
//
//             /* We want to do read ahead! */
//             WorkItem->Function = ReadAhead;
//             WorkItem->Parameters.Read.FileObject = FileObject;
//
//             /* Queue in the read ahead dedicated queue */
//             CcPostWorkQueue(WorkItem, &CcExpressWorkQueue);
//
//             return;
//         }
//
//         /* Fail path: lock again, and revert read ahead active */
//         KeAcquireSpinLock(&PrivateCacheMap->ReadAheadSpinLock, &OldIrql);
//         InterlockedAnd((volatile long *)&PrivateCacheMap->UlongFlags, ~PRIVATE_CACHE_MAP_READ_AHEAD_ACTIVE);
//     }
//
//     /* Done (fail) */
//     KeReleaseSpinLock(&PrivateCacheMap->ReadAheadSpinLock, OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcSetAdditionalCacheAttributes (
// 	IN	PFILE_OBJECT	FileObject,
// 	IN	BOOLEAN		DisableReadAhead,
// 	IN	BOOLEAN		DisableWriteBehind
// 	)
// {
//     KIRQL OldIrql;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p DisableReadAhead=%d DisableWriteBehind=%d\n",
//         FileObject, DisableReadAhead, DisableWriteBehind);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//
//     OldIrql = KeAcquireQueuedSpinLock(LockQueueMasterLock);
//
//     if (DisableReadAhead)
//     {
//         SetFlag(SharedCacheMap->Flags, READAHEAD_DISABLED);
//     }
//     else
//     {
//         ClearFlag(SharedCacheMap->Flags, READAHEAD_DISABLED);
//     }
//
//     if (DisableWriteBehind)
//     {
//         /* FIXME: also set flag 0x200 */
//         SetFlag(SharedCacheMap->Flags, WRITEBEHIND_DISABLED);
//     }
//     else
//     {
//         ClearFlag(SharedCacheMap->Flags, WRITEBEHIND_DISABLED);
//     }
//     KeReleaseQueuedSpinLock(LockQueueMasterLock, OldIrql);
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcSetBcbOwnerPointer (
// 	IN	PVOID	Bcb,
// 	IN	PVOID	Owner
// 	)
// {
//     PINTERNAL_BCB iBcb = CONTAINING_RECORD(Bcb, INTERNAL_BCB, PFCB);
//
//     CCTRACE(CC_API_DEBUG, "Bcb=%p Owner=%p\n",
//         Bcb, Owner);
//
//     if (!ExIsResourceAcquiredExclusiveLite(&iBcb->Lock) && !ExIsResourceAcquiredSharedLite(&iBcb->Lock))
//     {
//         DPRINT1("Current thread doesn't own resource!\n");
//         return;
//     }
//
//     ExSetResourceOwnerPointer(&iBcb->Lock, Owner);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcSetDirtyPageThreshold (
// 	IN	PFILE_OBJECT	FileObject,
// 	IN	ULONG		DirtyPageThreshold
// 	)
// {
//     PFSRTL_COMMON_FCB_HEADER Fcb;
//     PROS_SHARED_CACHE_MAP SharedCacheMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p DirtyPageThreshold=%lu\n",
//         FileObject, DirtyPageThreshold);
//
//     SharedCacheMap = FileObject->SectionObjectPointer->SharedCacheMap;
//     if (SharedCacheMap != NULL)
//     {
//         SharedCacheMap->DirtyPageThreshold = DirtyPageThreshold;
//     }
//
//     Fcb = FileObject->FsContext;
//     if (!BooleanFlagOn(Fcb->Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES))
//     {
//         SetFlag(Fcb->Flags, FSRTL_FLAG_LIMIT_MODIFIED_PAGES);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcSetReadAheadGranularity (
// 	IN	PFILE_OBJECT	FileObject,
// 	IN	ULONG		Granularity
// 	)
// {
//     PPRIVATE_CACHE_MAP PrivateMap;
//
//     CCTRACE(CC_API_DEBUG, "FileObject=%p Granularity=%lu\n",
//         FileObject, Granularity);
//
//     PrivateMap = FileObject->PrivateCacheMap;
//     PrivateMap->ReadAheadMask = Granularity - 1;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
