use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    _CM_DELAYED_CLOSE_ENTRY, _CM_DELAY_DEREF_KCB_ITEM, _KDPC, _KGUARDED_MUTEX, _KTIMER,
    _LIST_ENTRY, _WORK_QUEUE_ITEM, _WORK_QUEUE_TYPE_DelayedWorkQueue,
    CmpAcquireKcbLockExclusiveByKey, CmpAllocateDelayItem, CmpCleanUpKcbCacheWithLock,
    CmpDereferenceKeyControlBlock, CmpFreeDelayItem, CmpLockRegistry, CmpReleaseKcbLockByKey,
    CmpUnlockRegistry, ExQueueWorkItem, KeAcquireGuardedMutex, KeInitializeDpc,
    KeInitializeGuardedMutex, KeInitializeTimer, KeReleaseGuardedMutex, KeSetTimer, LARGE_INTEGER,
    PCM_DELAYED_CLOSE_ENTRY, PCM_DELAY_DEREF_KCB_ITEM, PCM_KEY_CONTROL_BLOCK, PKDPC,
    PWORK_QUEUE_ITEM,
};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
// ============================================================================
// Globals — translated from cmdelay.c (cmdelay.c is excluded from C build).
// ============================================================================

#[no_mangle] pub static mut CmpDelayDerefKCBWorkItem: _WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayedCloseSize: ULONG = 2048;

// Registry-controlled values owned by config/cmdelay. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
pub static CMDELAY_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Session Manager\\Configuration Manager", "DelayCloseSize",
            core::ptr::addr_of_mut!(CmpDelayedCloseSize)),
    ] });
#[no_mangle] pub static mut CmpDelayedCloseElements: ULONG = 0;
#[no_mangle] pub static mut CmpDelayedCloseTableLock: _KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayCloseWorkItemActive: BOOLEAN = 0;
#[no_mangle] pub static mut CmpDelayCloseWorkItem: _WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayedLRUListHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayCloseIntervalInSeconds: ULONG = 5;
#[no_mangle] pub static mut CmpDelayCloseDpc: _KDPC = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayCloseTimer: _KTIMER = unsafe { core::mem::zeroed() };

#[no_mangle] pub static mut CmpDelayDerefKCBLock: _KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayDerefKCBWorkItemActive: BOOLEAN = 0;
#[no_mangle] pub static mut CmpDelayDerefKCBListHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayDerefKCBIntervalInSeconds: ULONG = 5;
#[no_mangle] pub static mut CmpDelayDerefKCBDpc: _KDPC = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpDelayDerefKCBTimer: _KTIMER = unsafe { core::mem::zeroed() };

// CMP_ASSERT_KCB_LOCK is a macro in C. With NDEBUG it's a no-op.
#[inline]
unsafe fn CMP_ASSERT_KCB_LOCK(_Kcb: PCM_KEY_CONTROL_BLOCK) {}

extern "C" {
    fn ExInitializeWorkItem(
        Item: PWORK_QUEUE_ITEM,
        Routine: Option<unsafe extern "system" fn(PVOID)>,
        Context: PVOID,
    );
}

// InterlockedCompareExchange — stable atomic CAS replacement for
// `core::intrinsics::atomic_cxchg_acqrel_acquire`.  Returns OLD value.
#[inline]
unsafe fn InterlockedCompareExchange(
    Destination: *mut LONG,
    Exchange: LONG,
    Comparand: LONG,
) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Destination as *const AtomicI32);
    match atomic.compare_exchange(Comparand, Exchange, Ordering::AcqRel, Ordering::Acquire) {
        Ok(old) => old,
        Err(old) => old,
    }
}

unsafe extern "system" fn CmpDelayCloseDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    ExQueueWorkItem(
        core::ptr::addr_of_mut!(CmpDelayCloseWorkItem),
        _WORK_QUEUE_TYPE_DelayedWorkQueue,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpDelayCloseWorker(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let mut ListEntry: PCM_DELAYED_CLOSE_ENTRY;
    let mut i: ULONG;
    let mut ConvKey: ULONG;

    CmpLockRegistry();

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

    i = 0;
    while i < (CmpDelayedCloseSize >> 2) {
        if CmpDelayedCloseElements <= CmpDelayedCloseSize {
            break;
        }

        ListEntry = ((CmpDelayedLRUListHead.Blink as *mut u8).offset(
            -(core::mem::offset_of!(_CM_DELAYED_CLOSE_ENTRY, DelayedLRUList) as isize)
        )) as PCM_DELAYED_CLOSE_ENTRY;

        ConvKey = (*(*ListEntry).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.ConvKey;

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

        CmpAcquireKcbLockExclusiveByKey(ConvKey);

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

        ListEntry = ((CmpDelayedLRUListHead.Blink as *mut u8).offset(
            -(core::mem::offset_of!(_CM_DELAYED_CLOSE_ENTRY, DelayedLRUList) as isize)
        )) as PCM_DELAYED_CLOSE_ENTRY;

        if CmpDelayedCloseElements <= CmpDelayedCloseSize {
            CmpReleaseKcbLockByKey(ConvKey);
            break;
        }

        if ConvKey != (*(*ListEntry).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.ConvKey {
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

            CmpReleaseKcbLockByKey(ConvKey);

            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

            i += 1;
            continue;
        }

        ListEntry = RemoveTailList(core::ptr::addr_of_mut!(CmpDelayedLRUListHead)) as PCM_DELAYED_CLOSE_ENTRY;

        ListEntry = ((ListEntry as *mut u8).offset(
            -(core::mem::offset_of!(_CM_DELAYED_CLOSE_ENTRY, DelayedLRUList) as isize)
        )) as PCM_DELAYED_CLOSE_ENTRY;

        if ((*(*ListEntry).KeyControlBlock).RefCount != 0) ||
           ((*(*ListEntry).KeyControlBlock).__bindgen_anon_1.DelayedCloseIndex() != 0) {
            InsertHeadList(
                core::ptr::addr_of_mut!(CmpDelayedLRUListHead),
                core::ptr::addr_of_mut!((*ListEntry).DelayedLRUList),
            );

            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));
        } else {
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

            (*(*ListEntry).KeyControlBlock).DelayCloseEntry = core::ptr::null_mut();

            CmpCleanUpKcbCacheWithLock((*ListEntry).KeyControlBlock, 0u8);

            CmpFreeDelayItem(ListEntry as PVOID);

            InterlockedDecrement(core::ptr::addr_of_mut!(CmpDelayedCloseElements) as *mut i32);
        }

        CmpReleaseKcbLockByKey(ConvKey);

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

        i += 1;
    }

    if CmpDelayedCloseElements <= CmpDelayedCloseSize {
        CmpDelayCloseWorkItemActive = 0u8;
    } else {
        CmpArmDelayedCloseTimer();
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

    CmpUnlockRegistry();
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeDelayedCloseTable() {
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));
    
    ExInitializeWorkItem(
        core::ptr::addr_of_mut!(CmpDelayCloseWorkItem),
        Some(CmpDelayCloseWorker),
        core::ptr::null_mut(),
    );

    InitializeListHead(core::ptr::addr_of_mut!(CmpDelayedLRUListHead));

    KeInitializeDpc(
        core::ptr::addr_of_mut!(CmpDelayCloseDpc),
        core::mem::transmute(Some(CmpDelayCloseDpcRoutine as unsafe extern "system" fn(_, _, _, _))),
        core::ptr::null_mut(),
    );
    KeInitializeTimer(core::ptr::addr_of_mut!(CmpDelayCloseTimer));
}

pub unsafe extern "system" fn CmpDelayDerefKCBDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    ExQueueWorkItem(
        core::ptr::addr_of_mut!(CmpDelayDerefKCBWorkItem),
        _WORK_QUEUE_TYPE_DelayedWorkQueue,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpDelayDerefKCBWorker(Context: PVOID) {
    let Context = opaque_ptr(Context);
    CmpLockRegistry();
    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));

    while !IsListEmpty(core::ptr::addr_of_mut!(CmpDelayDerefKCBListHead)) {
        let entry_list = RemoveHeadList(core::ptr::addr_of_mut!(CmpDelayDerefKCBListHead));

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));

        let Entry = (entry_list as *mut u8).offset(-(core::mem::offset_of!(_CM_DELAY_DEREF_KCB_ITEM, ListEntry) as isize)) as PCM_DELAY_DEREF_KCB_ITEM;
        (*Entry).ListEntry.Flink = core::ptr::null_mut();
        (*Entry).ListEntry.Blink = core::ptr::null_mut();

        CmpDereferenceKeyControlBlock((*Entry).Kcb);
        CmpFreeDelayItem(Entry as PVOID);

        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));
    }

    CmpDelayDerefKCBWorkItemActive = 0u8;
    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));
    CmpUnlockRegistry();
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitDelayDerefKCBEngine() {
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));
    InitializeListHead(core::ptr::addr_of_mut!(CmpDelayDerefKCBListHead));

    ExInitializeWorkItem(
        core::ptr::addr_of_mut!(CmpDelayDerefKCBWorkItem),
        Some(CmpDelayDerefKCBWorker),
        core::ptr::null_mut(),
    );

    KeInitializeDpc(
        core::ptr::addr_of_mut!(CmpDelayDerefKCBDpc),
        core::mem::transmute(Some(CmpDelayDerefKCBDpcRoutine as unsafe extern "system" fn(_, _, _, _))),
        core::ptr::null_mut(),
    );
    KeInitializeTimer(core::ptr::addr_of_mut!(CmpDelayDerefKCBTimer));
}

#[no_mangle]
pub unsafe extern "system" fn CmpDelayDerefKeyControlBlock(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    let mut OldRefCount: i32;
    let mut NewRefCount: i32;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let Entry: PCM_DELAY_DEREF_KCB_ITEM;

    OldRefCount = *(core::ptr::addr_of_mut!((*Kcb).RefCount) as *mut i32);
    NewRefCount = OldRefCount - 1;
    
    if ((NewRefCount & 0xFFFF) > 0) &&
        (InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*Kcb).RefCount) as *mut i32,
            NewRefCount,
            OldRefCount,
        ) == OldRefCount)
    {
        return;
    }

    Entry = CmpAllocateDelayItem() as PCM_DELAY_DEREF_KCB_ITEM;
    if Entry.is_null() {
        return;
    }

    (*Entry).Kcb = Kcb;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));

    InsertTailList(
        core::ptr::addr_of_mut!(CmpDelayDerefKCBListHead),
        core::ptr::addr_of_mut!((*Entry).ListEntry)
    );

    if CmpDelayDerefKCBWorkItemActive == 0 {
        CmpDelayDerefKCBWorkItemActive = 1u8;
        Timeout.QuadPart =(CmpDelayDerefKCBIntervalInSeconds as i64) * -10000000i64;
        KeSetTimer(
            core::ptr::addr_of_mut!(CmpDelayDerefKCBTimer),
            Timeout,
            core::ptr::addr_of_mut!(CmpDelayDerefKCBDpc)
        );
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayDerefKCBLock));
}

#[no_mangle]
pub unsafe extern "system" fn CmpArmDelayedCloseTimer() {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    
    CmpDelayCloseWorkItemActive = 1u8;
    
    Timeout.QuadPart =CmpDelayCloseIntervalInSeconds as i64 * -10000000i64;
    KeSetTimer(
        core::ptr::addr_of_mut!(CmpDelayCloseTimer),
        Timeout,
        core::ptr::addr_of_mut!(CmpDelayCloseDpc),
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpAddToDelayedClose(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    LockHeldExclusively: u8,
) {
    let Kcb = opaque_ptr(Kcb);
    let mut i: u32;
    let mut OldRefCount: u32;
    let NewRefCount: u32;
    let Entry: PCM_DELAYED_CLOSE_ENTRY;

    CMP_ASSERT_KCB_LOCK(Kcb);

    if (*Kcb).__bindgen_anon_1.DelayedCloseIndex() != CmpDelayedCloseSize {
        // ASSERT(FALSE)
    }

    // ASSERT((*Kcb).RefCount == 0);
    // ASSERT(IsListEmpty(core::ptr::addr_of_mut!((*Kcb).KeyBodyListHead)) == 1);
    i = 0;
    while i < 4 {
        // ASSERT((*Kcb).KeyBodyArray[i as usize] == core::ptr::null_mut());
        i += 1;
    }

    Entry = CmpAllocateDelayItem() as PCM_DELAYED_CLOSE_ENTRY;
    if Entry.is_null() {
        CmpCleanUpKcbCacheWithLock(Kcb, LockHeldExclusively);
        return;
    }

    if (*Kcb).InDelayClose != 0 {
        // ASSERT(FALSE)
    }

    OldRefCount = (*Kcb).InDelayClose;
    // ASSERT(OldRefCount == 0);

    NewRefCount = 1;
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*Kcb).InDelayClose) as *mut i32,
        NewRefCount as i32,
        OldRefCount as i32,
    ) != OldRefCount as i32
    {
        // ASSERT(FALSE)
    }

    (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(0);

    (*Kcb).DelayCloseEntry = Entry as PVOID;
    (*Entry).KeyControlBlock = Kcb;

    InterlockedIncrement(core::ptr::addr_of_mut!(CmpDelayedCloseElements) as *mut i32);

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

    InsertHeadList(
        core::ptr::addr_of_mut!(CmpDelayedLRUListHead),
        core::ptr::addr_of_mut!((*Entry).DelayedLRUList),
    );

    if (CmpDelayedCloseElements > CmpDelayedCloseSize) && (CmpDelayCloseWorkItemActive == 0) {
        CmpArmDelayedCloseTimer();
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));
}

#[no_mangle]
pub unsafe extern "system" fn CmpRemoveFromDelayedClose(Kcb: PCM_KEY_CONTROL_BLOCK) {
    let Kcb = opaque_ptr(Kcb);
    let Entry: PCM_DELAYED_CLOSE_ENTRY;
    let NewRefCount: u32;
    let OldRefCount: u32;

    CMP_ASSERT_KCB_LOCK(Kcb);
    if (*Kcb).__bindgen_anon_1.DelayedCloseIndex() == CmpDelayedCloseSize {
        // ASSERT(FALSE)
    }

    Entry = (*Kcb).DelayCloseEntry as PCM_DELAYED_CLOSE_ENTRY;
    // ASSERT(Entry)
    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

    RemoveEntryList(core::ptr::addr_of_mut!((*Entry).DelayedLRUList));

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayedCloseTableLock));

    CmpFreeDelayItem(Entry as PVOID);

    InterlockedDecrement(core::ptr::addr_of_mut!(CmpDelayedCloseElements) as *mut i32);

    if (*Kcb).InDelayClose == 0 {
        // ASSERT(FALSE)
    }

    OldRefCount = (*Kcb).InDelayClose;
    // ASSERT(OldRefCount == 1)

    NewRefCount = 0;
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*Kcb).InDelayClose) as *mut i32,
        NewRefCount as i32,
        OldRefCount as i32,
    ) != OldRefCount as i32
    {
        // ASSERT(FALSE)
    }

    (*Kcb).DelayCloseEntry = core::ptr::null_mut();

    (*Kcb).__bindgen_anon_1.set_DelayedCloseIndex(CmpDelayedCloseSize);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmdelay.c
//  * PURPOSE:         Routines for handling delay close and allocate.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// WORK_QUEUE_ITEM CmpDelayDerefKCBWorkItem;
//
// ULONG CmpDelayedCloseSize = 2048;
// ULONG CmpDelayedCloseElements;
// KGUARDED_MUTEX CmpDelayedCloseTableLock;
// BOOLEAN CmpDelayCloseWorkItemActive;
// WORK_QUEUE_ITEM CmpDelayCloseWorkItem;
// LIST_ENTRY CmpDelayedLRUListHead;
// ULONG CmpDelayCloseIntervalInSeconds = 5;
// KDPC CmpDelayCloseDpc;
// KTIMER CmpDelayCloseTimer;
//
// KGUARDED_MUTEX CmpDelayDerefKCBLock;
// BOOLEAN CmpDelayDerefKCBWorkItemActive;
// LIST_ENTRY CmpDelayDerefKCBListHead;
// ULONG CmpDelayDerefKCBIntervalInSeconds = 5;
// KDPC CmpDelayDerefKCBDpc;
// KTIMER CmpDelayDerefKCBTimer;
//
// /* FUNCTIONS *****************************************************************/
//
// _Function_class_(KDEFERRED_ROUTINE)
// VOID
// NTAPI
// CmpDelayCloseDpcRoutine(IN PKDPC Dpc,
//                         IN PVOID DeferredContext,
//                         IN PVOID SystemArgument1,
//                         IN PVOID SystemArgument2)
// {
//     /* Sanity check */
//     ASSERT(CmpDelayCloseWorkItemActive);
//
//     /* Queue the work item */
//     ExQueueWorkItem(&CmpDelayCloseWorkItem, DelayedWorkQueue);
// }
//
// _Function_class_(WORKER_THREAD_ROUTINE)
// VOID
// NTAPI
// CmpDelayCloseWorker(IN PVOID Context)
// {
//     PCM_DELAYED_CLOSE_ENTRY ListEntry;
//     ULONG i, ConvKey;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT(CmpDelayCloseWorkItemActive);
//
//     /* Lock the registry */
//     CmpLockRegistry();
//
//     /* Acquire the delayed close table lock */
//     KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Iterate */
//     for (i = 0; i < (CmpDelayedCloseSize >> 2); i++)
//     {
//         /* Break out of the loop if there is nothing to process */
//         if (CmpDelayedCloseElements <= CmpDelayedCloseSize) break;
//
//         /* Sanity check */
//         ASSERT(!IsListEmpty(&CmpDelayedLRUListHead));
//
//         /* Get the entry */
//         ListEntry = CONTAINING_RECORD(CmpDelayedLRUListHead.Blink,
//                                       CM_DELAYED_CLOSE_ENTRY,
//                                       DelayedLRUList);
//
//         /* Save the ConvKey value of the KCB */
//         ConvKey = ListEntry->KeyControlBlock->ConvKey;
//
//         /* Release the delayed close table lock */
//         KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//
//         /* Acquire the KCB lock */
//         CmpAcquireKcbLockExclusiveByKey(ConvKey);
//
//         /* Reacquire the delayed close table lock */
//         KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//
//         /* Get the entry */
//         ListEntry = CONTAINING_RECORD(CmpDelayedLRUListHead.Blink,
//                                       CM_DELAYED_CLOSE_ENTRY,
//                                       DelayedLRUList);
//
//         /* Is the entry we have still the first one? */
//         if (CmpDelayedCloseElements <= CmpDelayedCloseSize)
//         {
//             /* No, someone already inserted an entry there */
//             CmpReleaseKcbLockByKey(ConvKey);
//             break;
//         }
//
//         /* Is it a different entry? */
//         if (ConvKey != ListEntry->KeyControlBlock->ConvKey)
//         {
//             /* Release the delayed close table lock */
//             KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//
//             /* Release the KCB lock */
//             CmpReleaseKcbLockByKey(ConvKey);
//
//             /* Reacquire the delayed close table lock */
//             KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//
//             /* Iterate again */
//             continue;
//         }
//
//         /* Remove it from the end of the list */
//         ListEntry =
//             (PCM_DELAYED_CLOSE_ENTRY)RemoveTailList(&CmpDelayedLRUListHead);
//
//         /* Get the containing entry */
//         ListEntry = CONTAINING_RECORD(ListEntry,
//                                       CM_DELAYED_CLOSE_ENTRY,
//                                       DelayedLRUList);
//
//         /* Process the entry */
//         if ((ListEntry->KeyControlBlock->RefCount) ||
//             (ListEntry->KeyControlBlock->DelayedCloseIndex))
//         {
//             /* Add it to the beginning of the list */
//             InsertHeadList(&CmpDelayedLRUListHead, &ListEntry->DelayedLRUList);
//
//             /* Release the delayed close table lock */
//             KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//         }
//         else
//         {
//             /* Release the delayed close table lock */
//             KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//
//             /* Zero out the DelayCloseEntry pointer */
//             ListEntry->KeyControlBlock->DelayCloseEntry = NULL;
//
//             /* Cleanup the KCB cache */
//             CmpCleanUpKcbCacheWithLock(ListEntry->KeyControlBlock, FALSE);
//
//             /* Free the delay item */
//             CmpFreeDelayItem(ListEntry);
//
//             /* Decrement delayed close elements count */
//             InterlockedDecrement((PLONG)&CmpDelayedCloseElements);
//         }
//
//         /* Release the KCB lock */
//         CmpReleaseKcbLockByKey(ConvKey);
//
//         /* Reacquire the delayed close table lock */
//         KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//     }
//
//     if (CmpDelayedCloseElements <= CmpDelayedCloseSize)
//     {
//         /* We're not active anymore */
//         CmpDelayCloseWorkItemActive = FALSE;
//     }
//     else
//     {
//         /* We didn't process all things, so reschedule for the next time */
//         CmpArmDelayedCloseTimer();
//     }
//
//     /* Release the delayed close table lock */
//     KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Unlock the registry */
//     CmpUnlockRegistry();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitializeDelayedCloseTable(VOID)
// {
//
//     /* Setup the delayed close lock */
//     KeInitializeGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Setup the work item */
//     ExInitializeWorkItem(&CmpDelayCloseWorkItem, CmpDelayCloseWorker, NULL);
//
//     /* Setup the list head */
//     InitializeListHead(&CmpDelayedLRUListHead);
//
//     /* Setup the DPC and its timer */
//     KeInitializeDpc(&CmpDelayCloseDpc, CmpDelayCloseDpcRoutine, NULL);
//     KeInitializeTimer(&CmpDelayCloseTimer);
// }
//
// _Function_class_(KDEFERRED_ROUTINE)
// VOID
// NTAPI
// CmpDelayDerefKCBDpcRoutine(IN PKDPC Dpc,
//                            IN PVOID DeferredContext,
//                            IN PVOID SystemArgument1,
//                            IN PVOID SystemArgument2)
// {
//     /* Sanity check */
//     ASSERT(CmpDelayDerefKCBWorkItemActive);
//
//     /* Queue the work item */
//     ExQueueWorkItem(&CmpDelayDerefKCBWorkItem, DelayedWorkQueue);
// }
//
// _Function_class_(WORKER_THREAD_ROUTINE)
// VOID
// NTAPI
// CmpDelayDerefKCBWorker(IN PVOID Context)
// {
//     PCM_DELAY_DEREF_KCB_ITEM Entry;
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT(CmpDelayDerefKCBWorkItemActive);
//
//     /* Lock the registry and and list lock */
//     CmpLockRegistry();
//     KeAcquireGuardedMutex(&CmpDelayDerefKCBLock);
//
//     /* Check if the list is empty */
//     while (!IsListEmpty(&CmpDelayDerefKCBListHead))
//     {
//         /* Grab an entry */
//         Entry = (PVOID)RemoveHeadList(&CmpDelayDerefKCBListHead);
//
//         /* We can release the lock now */
//         KeReleaseGuardedMutex(&CmpDelayDerefKCBLock);
//
//         /* Now grab the actual entry */
//         Entry = CONTAINING_RECORD(Entry, CM_DELAY_DEREF_KCB_ITEM, ListEntry);
//         Entry->ListEntry.Flink = Entry->ListEntry.Blink = NULL;
//
//         /* Dereference and free */
//         CmpDereferenceKeyControlBlock(Entry->Kcb);
//         CmpFreeDelayItem(Entry);
//
//         /* Lock the list again */
//         KeAcquireGuardedMutex(&CmpDelayDerefKCBLock);
//     }
//
//     /* We're done */
//     CmpDelayDerefKCBWorkItemActive = FALSE;
//     KeReleaseGuardedMutex(&CmpDelayDerefKCBLock);
//     CmpUnlockRegistry();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitDelayDerefKCBEngine(VOID)
// {
//     /* Initialize lock and list */
//     KeInitializeGuardedMutex(&CmpDelayDerefKCBLock);
//     InitializeListHead(&CmpDelayDerefKCBListHead);
//
//     /* Setup the work item */
//     ExInitializeWorkItem(&CmpDelayDerefKCBWorkItem,
//                          CmpDelayDerefKCBWorker,
//                          NULL);
//
//     /* Setup the DPC and timer for it */
//     KeInitializeDpc(&CmpDelayDerefKCBDpc, CmpDelayDerefKCBDpcRoutine, NULL);
//     KeInitializeTimer(&CmpDelayDerefKCBTimer);
// }
//
// VOID
// NTAPI
// CmpDelayDerefKeyControlBlock(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     LONG OldRefCount, NewRefCount;
//     LARGE_INTEGER Timeout;
//     PCM_DELAY_DEREF_KCB_ITEM Entry;
//     PAGED_CODE();
//     CMTRACE(CM_REFERENCE_DEBUG,
//             "%s - Dereferencing KCB: %p\n", __FUNCTION__, Kcb);
//
//     /* Get the previous reference count */
//     OldRefCount = *(PLONG)&Kcb->RefCount;
//     NewRefCount = OldRefCount - 1;
//     if (((NewRefCount & 0xFFFF) > 0) &&
//         (InterlockedCompareExchange((PLONG)&Kcb->RefCount,
//                                     NewRefCount,
//                                     OldRefCount) == OldRefCount))
//     {
//         /* KCB still had references, so we're done */
//         return;
//     }
//
//     /* Allocate a delay item */
//     Entry = CmpAllocateDelayItem();
//     if (!Entry) return;
//
//     /* Set the KCB */
//     Entry->Kcb = Kcb;
//
//     /* Acquire the delayed deref table lock */
//     KeAcquireGuardedMutex(&CmpDelayDerefKCBLock);
//
//     /* Insert the entry into the list */
//     InsertTailList(&CmpDelayDerefKCBListHead, &Entry->ListEntry);
//
//     /* Check if we need to enable anything */
//     if (!CmpDelayDerefKCBWorkItemActive)
//     {
//         /* Yes, we have no work item, setup the interval */
//         CmpDelayDerefKCBWorkItemActive = TRUE;
//         Timeout.QuadPart = CmpDelayDerefKCBIntervalInSeconds * -10000000;
//         KeSetTimer(&CmpDelayDerefKCBTimer, Timeout, &CmpDelayDerefKCBDpc);
//     }
//
//     /* Release the table lock */
//     KeReleaseGuardedMutex(&CmpDelayDerefKCBLock);
// }
//
// VOID
// NTAPI
// CmpArmDelayedCloseTimer(VOID)
// {
//     LARGE_INTEGER Timeout;
//     PAGED_CODE();
//
//     /* Set the worker active */
//     CmpDelayCloseWorkItemActive = TRUE;
//
//     /* Setup the interval */
//     Timeout.QuadPart = CmpDelayCloseIntervalInSeconds * -10000000;
//     KeSetTimer(&CmpDelayCloseTimer, Timeout, &CmpDelayCloseDpc);
// }
//
// VOID
// NTAPI
// CmpAddToDelayedClose(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                      IN BOOLEAN LockHeldExclusively)
// {
//     ULONG i;
//     ULONG OldRefCount, NewRefCount;
//     PCM_DELAYED_CLOSE_ENTRY Entry;
//     PAGED_CODE();
//
//     /* Sanity check */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//
//     /* Make sure it's valid */
//     if (Kcb->DelayedCloseIndex != CmpDelayedCloseSize) ASSERT(FALSE);
//
//     /* Sanity checks */
//     ASSERT(Kcb->RefCount == 0);
//     ASSERT(IsListEmpty(&Kcb->KeyBodyListHead) == TRUE);
//     for (i = 0; i < 4; i++) ASSERT(Kcb->KeyBodyArray[i] == NULL);
//
//     /* Allocate a delay item */
//     Entry = CmpAllocateDelayItem();
//     if (!Entry)
//     {
//         /* Cleanup immediately */
//         CmpCleanUpKcbCacheWithLock(Kcb, LockHeldExclusively);
//         return;
//     }
//
//     /* Sanity check */
//     if (Kcb->InDelayClose) ASSERT(FALSE);
//
//     /* Get the previous reference count */
//     OldRefCount = *(PLONG)&Kcb->InDelayClose;
//     ASSERT(OldRefCount == 0);
//
//     /* Write the new one */
//     NewRefCount = 1;
//     if (InterlockedCompareExchange((PLONG)&Kcb->InDelayClose,
//                                    NewRefCount,
//                                    OldRefCount) != OldRefCount)
//     {
//         /* Sanity check */
//         ASSERT(FALSE);
//     }
//
//     /* Reset the delayed close index */
//     Kcb->DelayedCloseIndex = 0;
//
//     /* Set up the close entry */
//     Kcb->DelayCloseEntry = Entry;
//     Entry->KeyControlBlock = Kcb;
//
//     /* Increase the number of elements */
//     InterlockedIncrement((PLONG)&CmpDelayedCloseElements);
//
//     /* Acquire the delayed close table lock */
//     KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Insert the entry into the list */
//     InsertHeadList(&CmpDelayedLRUListHead, &Entry->DelayedLRUList);
//
//     /* Check if we need to enable anything */
//     if ((CmpDelayedCloseElements > CmpDelayedCloseSize) &&
//         !(CmpDelayCloseWorkItemActive))
//     {
//         /* Yes, we have too many elements to close, and no work item */
//         CmpArmDelayedCloseTimer();
//     }
//
//     /* Release the table lock */
//     KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
// }
//
// VOID
// NTAPI
// CmpRemoveFromDelayedClose(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     PCM_DELAYED_CLOSE_ENTRY Entry;
//     ULONG NewRefCount, OldRefCount;
//     PAGED_CODE();
//
//     /* Sanity checks */
//     CMP_ASSERT_KCB_LOCK(Kcb);
//     if (Kcb->DelayedCloseIndex == CmpDelayedCloseSize) ASSERT(FALSE);
//
//     /* Get the entry and lock the table */
//     Entry = Kcb->DelayCloseEntry;
//     ASSERT(Entry);
//     KeAcquireGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Remove the entry */
//     RemoveEntryList(&Entry->DelayedLRUList);
//
//     /* Release the lock */
//     KeReleaseGuardedMutex(&CmpDelayedCloseTableLock);
//
//     /* Free the entry */
//     CmpFreeDelayItem(Entry);
//
//     /* Reduce the number of elements */
//     InterlockedDecrement((PLONG)&CmpDelayedCloseElements);
//
//     /* Sanity check */
//     if (!Kcb->InDelayClose) ASSERT(FALSE);
//
//     /* Get the previous reference count */
//     OldRefCount = *(PLONG)&Kcb->InDelayClose;
//     ASSERT(OldRefCount == 1);
//
//     /* Write the new one */
//     NewRefCount = 0;
//     if (InterlockedCompareExchange((PLONG)&Kcb->InDelayClose,
//                                    NewRefCount,
//                                    OldRefCount) != OldRefCount)
//     {
//         /* Sanity check */
//         ASSERT(FALSE);
//     }
//
//     /* Remove the link to the entry */
//     Kcb->DelayCloseEntry = NULL;
//
//     /* Set new delay size and remove the delete flag */
//     Kcb->DelayedCloseIndex = CmpDelayedCloseSize;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
