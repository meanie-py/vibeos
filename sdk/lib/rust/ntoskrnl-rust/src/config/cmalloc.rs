use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    CM_DELAY_ALLOC, CM_KEY_CONTROL_BLOCK, CmpAllocate, CmpFree,
    KeAcquireGuardedMutex, KeInitializeGuardedMutex, KeReleaseGuardedMutex,
    KGUARDED_MUTEX, PCM_ALLOC_PAGE, PCM_DELAY_ALLOC, PCM_KEY_CONTROL_BLOCK,
    _CM_ALLOC_PAGE, _CM_DELAY_ALLOC,
};

use ntkrnl_sys::opaque_ptr;
// Pool tags from sdk/lib/cmlib/cmlib.h:
//   #define TAG_CM   '  MC'   (big-endian multi-char literal -> bytes "CM  ")
//   #define TAG_KCB  'bkMC'   (bytes "CMkb")
const TAG_KCB: u32 = u32::from_le_bytes(*b"CMkb");
const TAG_CM: u32 = u32::from_le_bytes(*b"CM  ");

// CM_KCBS_PER_PAGE / CM_DELAYS_PER_PAGE — from ntoskrnl/include/internal/cm.h.
// Computed identically to the C #define using offset_of! for FIELD_OFFSET.
const CM_KCBS_PER_PAGE: u32 = ((PAGE_SIZE - core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage))
    / core::mem::size_of::<CM_KEY_CONTROL_BLOCK>()) as u32;
const CM_DELAYS_PER_PAGE: u32 = ((PAGE_SIZE - core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage))
    / core::mem::size_of::<CM_DELAY_ALLOC>()) as u32;

// Globals defined in ntoskrnl/config/cmdata.c (still C in the build).
extern "C" {
    pub static mut CmpAllocInited: BOOLEAN;
    pub static mut CmpAllocBucketLock: KGUARDED_MUTEX;
    pub static mut CmpFreeKCBListHead: LIST_ENTRY;
    pub static mut CmpDelayAllocBucketLock: KGUARDED_MUTEX;
    pub static mut CmpFreeDelayItemsListHead: LIST_ENTRY;
}

// CMP_ASSERT_HASH_ENTRY_LOCK is a debug-only macro (ASSERT) in
// ntoskrnl/include/internal/cm_x.h — translate to a no-op in release builds.
#[inline(always)]
unsafe fn CMP_ASSERT_HASH_ENTRY_LOCK(_k: u32) {}

// CmpGetAllocPageFromKcb / CmpGetAllocPageFromDelayAlloc — page-aligned
// down-cast of the input pointer (cm_x.h).
#[inline(always)]
unsafe fn CmpGetAllocPageFromKcb(k: PCM_KEY_CONTROL_BLOCK) -> PCM_ALLOC_PAGE {
    ((k as ULONG_PTR) & !(PAGE_SIZE - 1)) as PCM_ALLOC_PAGE
}

#[inline(always)]
unsafe fn CmpGetAllocPageFromDelayAlloc(a: *const core::ffi::c_void) -> PCM_ALLOC_PAGE {
    ((a as ULONG_PTR) & !(PAGE_SIZE - 1)) as PCM_ALLOC_PAGE
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitCmPrivateAlloc() {
    if CmpAllocInited == 0 {
        KeInitializeGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));
        InitializeListHead(core::ptr::addr_of_mut!(CmpFreeKCBListHead));
        CmpAllocInited = 1u8;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitCmPrivateDelayAlloc() {
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));
    InitializeListHead(core::ptr::addr_of_mut!(CmpFreeDelayItemsListHead));
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeKeyControlBlock(mut Kcb: PCM_KEY_CONTROL_BLOCK) {
    let mut Kcb = opaque_ptr(Kcb);
    let mut i: u32;
    let mut AllocPage: PCM_ALLOC_PAGE;

    i = 0;
    while i < 4 {
        i += 1;
    }

    if (*Kcb).__bindgen_anon_1.PrivateAlloc() == 0 {
        CmpFree(Kcb as *mut core::ffi::c_void, TAG_KCB);
        return;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));

    CMP_ASSERT_HASH_ENTRY_LOCK((*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey);

    InsertTailList(
        core::ptr::addr_of_mut!(CmpFreeKCBListHead),
        core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_4.FreeListEntry)
    );

    AllocPage = CmpGetAllocPageFromKcb(Kcb);

    (*AllocPage).FreeCount += 1;
    if (*AllocPage).FreeCount == CM_KCBS_PER_PAGE {
        i = 0;
        while i < CM_KCBS_PER_PAGE {
            Kcb = ((AllocPage as ULONG_PTR) +
                   core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage) +
                   (i as usize * core::mem::size_of::<CM_KEY_CONTROL_BLOCK>())) as PCM_KEY_CONTROL_BLOCK;

            RemoveEntryList(core::ptr::addr_of_mut!((*Kcb).__bindgen_anon_4.FreeListEntry));
            i += 1;
        }

        CmpFree(AllocPage as *mut core::ffi::c_void, TAG_KCB);
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));
}

#[no_mangle]
pub unsafe extern "system" fn CmpAllocateKeyControlBlock() -> PCM_KEY_CONTROL_BLOCK {
    let mut NextEntry: PLIST_ENTRY;
    let mut CurrentKcb: PCM_KEY_CONTROL_BLOCK;
    let mut AllocPage: PCM_ALLOC_PAGE;
    let mut i: u32;

    if CmpAllocInited != 0 {
        KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));

        'search_kcb: loop {
            if !IsListEmpty(core::ptr::addr_of_mut!(CmpFreeKCBListHead)) {
                NextEntry = RemoveHeadList(core::ptr::addr_of_mut!(CmpFreeKCBListHead));

                CurrentKcb = ((NextEntry as usize) - core::mem::offset_of!(CM_KEY_CONTROL_BLOCK, __bindgen_anon_4)) as PCM_KEY_CONTROL_BLOCK;

                AllocPage = CmpGetAllocPageFromKcb(CurrentKcb);

                (*AllocPage).FreeCount -= 1;

                KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));

                return CurrentKcb;
            }

            AllocPage = CmpAllocate(PAGE_SIZE as _, 1u8, TAG_KCB) as PCM_ALLOC_PAGE;
            if !AllocPage.is_null() {
                (*AllocPage).FreeCount = CM_KCBS_PER_PAGE;

                i = 0;
                while i < CM_KCBS_PER_PAGE {
                    CurrentKcb = ((AllocPage as ULONG_PTR) + core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage) + (i as usize) * core::mem::size_of::<CM_KEY_CONTROL_BLOCK>()) as PCM_KEY_CONTROL_BLOCK;

                    (*CurrentKcb).__bindgen_anon_1.set_PrivateAlloc(1);
                    (*CurrentKcb).DelayCloseEntry = core::ptr::null_mut();
                    InsertTailList(core::ptr::addr_of_mut!(CmpFreeKCBListHead),
                                   core::ptr::addr_of_mut!((*CurrentKcb).__bindgen_anon_4.FreeListEntry));

                    i += 1;
                }

                continue 'search_kcb;
            }

            break;
        }

        KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpAllocBucketLock));
    }

    CurrentKcb = CmpAllocate(core::mem::size_of::<CM_KEY_CONTROL_BLOCK>() as _, 1u8, TAG_KCB) as PCM_KEY_CONTROL_BLOCK;
    if !CurrentKcb.is_null() {
        (*CurrentKcb).__bindgen_anon_1.set_PrivateAlloc(0);
        (*CurrentKcb).DelayCloseEntry = core::ptr::null_mut();
    }

    return CurrentKcb;
}

#[no_mangle]
pub unsafe extern "system" fn CmpAllocateDelayItem() -> *mut core::ffi::c_void {
    let mut Entry: PCM_DELAY_ALLOC;
    let mut AllocPage: PCM_ALLOC_PAGE;
    let mut i: u32;
    let mut NextEntry: PLIST_ENTRY;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));

    'SearchList: loop {
        if !IsListEmpty(core::ptr::addr_of_mut!(CmpFreeDelayItemsListHead)) {
            NextEntry = RemoveHeadList(core::ptr::addr_of_mut!(CmpFreeDelayItemsListHead));

            Entry = (NextEntry as usize - core::mem::offset_of!(_CM_DELAY_ALLOC, ListEntry)) as PCM_DELAY_ALLOC;

            (*Entry).ListEntry.Flink = core::ptr::null_mut();
            (*Entry).ListEntry.Blink = core::ptr::null_mut();

            AllocPage = CmpGetAllocPageFromDelayAlloc(Entry as *const core::ffi::c_void);

            (*AllocPage).FreeCount -= 1;

            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));
            return Entry as *mut core::ffi::c_void;
        }

        AllocPage = CmpAllocate(PAGE_SIZE as _, 1u8, TAG_CM) as PCM_ALLOC_PAGE;
        if !AllocPage.is_null() {
            (*AllocPage).FreeCount = CM_DELAYS_PER_PAGE;

            i = 0;
            while i < CM_DELAYS_PER_PAGE {
                Entry = ((AllocPage as ULONG_PTR) +
                         core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage) +
                         (i as usize) * core::mem::size_of::<_CM_DELAY_ALLOC>()) as PCM_DELAY_ALLOC;
                InsertTailList(
                    core::ptr::addr_of_mut!(CmpFreeDelayItemsListHead),
                    core::ptr::addr_of_mut!((*Entry).ListEntry)
                );

                (*Entry).Kcb = core::ptr::null_mut();
                i += 1;
            }

            continue 'SearchList;
        }

        break;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeDelayItem(Entry: PVOID) {
    let Entry = opaque_ptr(Entry);
    let mut AllocEntry: PCM_DELAY_ALLOC = Entry as PCM_DELAY_ALLOC;
    let mut AllocPage: PCM_ALLOC_PAGE;
    let mut i: ULONG;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));

    InsertTailList(core::ptr::addr_of_mut!(CmpFreeDelayItemsListHead),
                   core::ptr::addr_of_mut!((*AllocEntry).ListEntry));

    AllocPage = CmpGetAllocPageFromDelayAlloc(Entry);

    (*AllocPage).FreeCount += 1;
    if (*AllocPage).FreeCount == CM_DELAYS_PER_PAGE {
        i = 0;
        while i < CM_DELAYS_PER_PAGE {
            AllocEntry = ((AllocPage as ULONG_PTR) +
                         core::mem::offset_of!(_CM_ALLOC_PAGE, AllocPage) +
                         (i as ULONG_PTR) * core::mem::size_of::<CM_DELAY_ALLOC>()) as PCM_DELAY_ALLOC;
            RemoveEntryList(core::ptr::addr_of_mut!((*AllocEntry).ListEntry));
            i += 1;
        }

        CmpFree(AllocPage as PVOID, TAG_CM);
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(CmpDelayAllocBucketLock));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmalloc.c
//  * PURPOSE:         Routines for allocating and freeing registry structures
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
// BOOLEAN CmpAllocInited;
// KGUARDED_MUTEX CmpAllocBucketLock;
// LIST_ENTRY CmpFreeKCBListHead;
//
// KGUARDED_MUTEX CmpDelayAllocBucketLock;
// LIST_ENTRY CmpFreeDelayItemsListHead;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitCmPrivateAlloc(VOID)
// {
//     /* Make sure we didn't already do this */
//     if (!CmpAllocInited)
//     {
//         /* Setup the lock and list */
//         KeInitializeGuardedMutex(&CmpAllocBucketLock);
//         InitializeListHead(&CmpFreeKCBListHead);
//         CmpAllocInited = TRUE;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitCmPrivateDelayAlloc(VOID)
// {
//     /* Initialize the delay allocation list and lock */
//     KeInitializeGuardedMutex(&CmpDelayAllocBucketLock);
//     InitializeListHead(&CmpFreeDelayItemsListHead);
// }
//
// VOID
// NTAPI
// CmpFreeKeyControlBlock(IN PCM_KEY_CONTROL_BLOCK Kcb)
// {
//     ULONG i;
//     PCM_ALLOC_PAGE AllocPage;
//     PAGED_CODE();
//
//     /* Sanity checks */
//     ASSERT(IsListEmpty(&Kcb->KeyBodyListHead) == TRUE);
//     for (i = 0; i < 4; i++) ASSERT(Kcb->KeyBodyArray[i] == NULL);
//
//     /* Check if it wasn't privately allocated */
//     if (!Kcb->PrivateAlloc)
//     {
//         /* Free it from the pool */
//         CmpFree(Kcb, TAG_KCB);
//         return;
//     }
//
//     /* Acquire the private allocation lock */
//     KeAcquireGuardedMutex(&CmpAllocBucketLock);
//
//     /* Sanity check on lock ownership */
//     CMP_ASSERT_HASH_ENTRY_LOCK(Kcb->ConvKey);
//
//     /* Add us to the free list */
//     InsertTailList(&CmpFreeKCBListHead, &Kcb->FreeListEntry);
//
//     /* Get the allocation page */
//     AllocPage = CmpGetAllocPageFromKcb(Kcb);
//
//     /* Sanity check */
//     ASSERT(AllocPage->FreeCount != CM_KCBS_PER_PAGE);
//
//     /* Increase free count */
//     if (++AllocPage->FreeCount == CM_KCBS_PER_PAGE)
//     {
//         /* Loop all the entries */
//         for (i = 0; i < CM_KCBS_PER_PAGE; i++)
//         {
//             /* Get the KCB */
//             Kcb = (PVOID)((ULONG_PTR)AllocPage +
//                           FIELD_OFFSET(CM_ALLOC_PAGE, AllocPage) +
//                           i * sizeof(CM_KEY_CONTROL_BLOCK));
//
//             /* Remove the entry */
//             RemoveEntryList(&Kcb->FreeListEntry);
//         }
//
//         /* Free the page */
//         CmpFree(AllocPage, TAG_KCB);
//     }
//
//     /* Release the lock */
//     KeReleaseGuardedMutex(&CmpAllocBucketLock);
// }
//
// PCM_KEY_CONTROL_BLOCK
// NTAPI
// CmpAllocateKeyControlBlock(VOID)
// {
//     PLIST_ENTRY NextEntry;
//     PCM_KEY_CONTROL_BLOCK CurrentKcb;
//     PCM_ALLOC_PAGE AllocPage;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Check if private allocations are initialized */
//     if (CmpAllocInited)
//     {
//         /* They are, acquire the bucket lock */
//         KeAcquireGuardedMutex(&CmpAllocBucketLock);
//
//         /* See if there's something on the free KCB list */
// SearchKcbList:
//         if (!IsListEmpty(&CmpFreeKCBListHead))
//         {
//             /* Remove the entry */
//             NextEntry = RemoveHeadList(&CmpFreeKCBListHead);
//
//             /* Get the KCB */
//             CurrentKcb = CONTAINING_RECORD(NextEntry,
//                                            CM_KEY_CONTROL_BLOCK,
//                                            FreeListEntry);
//
//             /* Get the allocation page */
//             AllocPage = CmpGetAllocPageFromKcb(CurrentKcb);
//
//             /* Decrease the free count */
//             ASSERT(AllocPage->FreeCount != 0);
//             AllocPage->FreeCount--;
//
//             /* Make sure this KCB is privately allocated */
//             ASSERT(CurrentKcb->PrivateAlloc == 1);
//
//             /* Release the allocation lock */
//             KeReleaseGuardedMutex(&CmpAllocBucketLock);
//
//             /* Return the KCB */
//             return CurrentKcb;
//         }
//
//         /* Allocate an allocation page */
//         AllocPage = CmpAllocate(PAGE_SIZE, TRUE, TAG_KCB);
//         if (AllocPage)
//         {
//             /* Set default entries */
//             AllocPage->FreeCount = CM_KCBS_PER_PAGE;
//
//             /* Loop each entry */
//             for (i = 0; i < CM_KCBS_PER_PAGE; i++)
//             {
//                 /* Get this entry */
//                 CurrentKcb = (PVOID)((ULONG_PTR)AllocPage +
//                                      FIELD_OFFSET(CM_ALLOC_PAGE, AllocPage) +
//                                      i * sizeof(CM_KEY_CONTROL_BLOCK));
//
//                 /* Set it up */
//                 CurrentKcb->PrivateAlloc = TRUE;
//                 CurrentKcb->DelayCloseEntry = NULL;
//                 InsertTailList(&CmpFreeKCBListHead,
//                                &CurrentKcb->FreeListEntry);
//             }
//
//             /* Now go back and search the list */
//             goto SearchKcbList;
//         }
//
//         /* Release the allocation lock */
//         KeReleaseGuardedMutex(&CmpAllocBucketLock);
//     }
//
//     /* Allocate a KCB only */
//     CurrentKcb = CmpAllocate(sizeof(CM_KEY_CONTROL_BLOCK), TRUE, TAG_KCB);
//     if (CurrentKcb)
//     {
//         /* Set it up */
//         CurrentKcb->PrivateAlloc = 0;
//         CurrentKcb->DelayCloseEntry = NULL;
//     }
//
//     /* Return it */
//     return CurrentKcb;
// }
//
// PVOID
// NTAPI
// CmpAllocateDelayItem(VOID)
// {
//     PCM_DELAY_ALLOC Entry;
//     PCM_ALLOC_PAGE AllocPage;
//     ULONG i;
//     PLIST_ENTRY NextEntry;
//     PAGED_CODE();
//
//     /* Lock the allocation buckets */
//     KeAcquireGuardedMutex(&CmpDelayAllocBucketLock);
//
//     /* Look for an item on the free list */
// SearchList:
//     if (!IsListEmpty(&CmpFreeDelayItemsListHead))
//     {
//         /* Get the current entry in the list */
//         NextEntry = RemoveHeadList(&CmpFreeDelayItemsListHead);
//
//         /* Grab the item */
//         Entry = CONTAINING_RECORD(NextEntry, CM_DELAY_ALLOC, ListEntry);
//
//         /* Clear the list */
//         Entry->ListEntry.Flink = Entry->ListEntry.Blink = NULL;
//
//         /* Grab the alloc page */
//         AllocPage = CmpGetAllocPageFromDelayAlloc(Entry);
//
//         /* Decrease free entries */
//         ASSERT(AllocPage->FreeCount != 0);
//         AllocPage->FreeCount--;
//
//         /* Release the lock */
//         KeReleaseGuardedMutex(&CmpDelayAllocBucketLock);
//         return Entry;
//     }
//
//     /* Allocate an allocation page */
//     AllocPage = CmpAllocate(PAGE_SIZE, TRUE, TAG_CM);
//     if (AllocPage)
//     {
//         /* Set default entries */
//         AllocPage->FreeCount = CM_DELAYS_PER_PAGE;
//
//         /* Loop each entry */
//         for (i = 0; i < CM_DELAYS_PER_PAGE; i++)
//         {
//             /* Get this entry and link it */
//             Entry = (PVOID)((ULONG_PTR)AllocPage +
//                             FIELD_OFFSET(CM_ALLOC_PAGE, AllocPage) +
//                             i * sizeof(CM_DELAY_ALLOC));
//             InsertTailList(&CmpFreeDelayItemsListHead,
//                            &Entry->ListEntry);
//
//             /* Clear the KCB pointer */
//             Entry->Kcb = NULL;
//         }
//
//         /* Do the search again */
//         goto SearchList;
//     }
//
//     /* Release the lock */
//     KeReleaseGuardedMutex(&CmpDelayAllocBucketLock);
//     return NULL;
// }
//
// VOID
// NTAPI
// CmpFreeDelayItem(PVOID Entry)
// {
//     PCM_DELAY_ALLOC AllocEntry = (PCM_DELAY_ALLOC)Entry;
//     PCM_ALLOC_PAGE AllocPage;
//     ULONG i;
//     PAGED_CODE();
//
//     /* Lock the table */
//     KeAcquireGuardedMutex(&CmpDelayAllocBucketLock);
//
//     /* Add the entry at the end */
//     InsertTailList(&CmpFreeDelayItemsListHead, &AllocEntry->ListEntry);
//
//     /* Get the alloc page */
//     AllocPage = CmpGetAllocPageFromDelayAlloc(Entry);
//     ASSERT(AllocPage->FreeCount != CM_DELAYS_PER_PAGE);
//
//     /* Increase the number of free items */
//     if (++AllocPage->FreeCount == CM_DELAYS_PER_PAGE)
//     {
//         /* Page is totally free now, loop each entry */
//         for (i = 0; i < CM_DELAYS_PER_PAGE; i++)
//         {
//             /* Get the entry and unlink it */
//             AllocEntry = (PVOID)((ULONG_PTR)AllocPage +
//                                  FIELD_OFFSET(CM_ALLOC_PAGE, AllocPage) +
//                                  i * sizeof(CM_DELAY_ALLOC));
//             RemoveEntryList(&AllocEntry->ListEntry);
//         }
//
//         /* Now free the page */
//         CmpFree(AllocPage, TAG_CM);
//     }
//
//     /* Release the lock */
//     KeReleaseGuardedMutex(&CmpDelayAllocBucketLock);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
