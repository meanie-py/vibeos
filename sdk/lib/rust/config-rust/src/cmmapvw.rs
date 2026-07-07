use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CM_VIEW_OF_FILE, ExFreePool, PCMHIVE, PCM_VIEW_OF_FILE};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
#[no_mangle]
pub unsafe extern "system" fn CmpInitHiveViewList(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    InitializeListHead(core::ptr::addr_of_mut!((*Hive).LRUViewListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*Hive).PinViewListHead));

    (*Hive).MappedViews = 0;
    (*Hive).PinnedViews = 0;
    (*Hive).UseCount = 0;
}

#[no_mangle]
pub unsafe extern "system" fn CmpDestroyHiveViewList(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    let mut CmView: PCM_VIEW_OF_FILE;
    let mut EntryList: PLIST_ENTRY;

    while !IsListEmpty(core::ptr::addr_of_mut!((*Hive).PinViewListHead)) {
        EntryList = RemoveHeadList(core::ptr::addr_of_mut!((*Hive).PinViewListHead));

        CmView = (EntryList as *mut u8).offset(-(core::mem::offset_of!(CM_VIEW_OF_FILE, PinViewList) as isize)) as PCM_VIEW_OF_FILE;

        ExFreePool(CmView as *mut core::ffi::c_void);

        (*Hive).PinnedViews -= 1;
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*Hive).LRUViewListHead)) {
        EntryList = RemoveHeadList(core::ptr::addr_of_mut!((*Hive).LRUViewListHead));

        CmView = (EntryList as *mut u8).offset(-(core::mem::offset_of!(CM_VIEW_OF_FILE, LRUViewList) as isize)) as PCM_VIEW_OF_FILE;

        ExFreePool(CmView as *mut core::ffi::c_void);

        (*Hive).MappedViews -= 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmmapvw.c
//  * PURPOSE:         Configuration Manager - Map-Viewed Hive Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// CmpInitHiveViewList(IN PCMHIVE Hive)
// {
//     /* Initialize the list heads */
//     InitializeListHead(&Hive->LRUViewListHead);
//     InitializeListHead(&Hive->PinViewListHead);
//
//     /* Reset data */
//     Hive->MappedViews = 0;
//     Hive->PinnedViews = 0;
//     Hive->UseCount = 0;
// }
//
// VOID
// NTAPI
// CmpDestroyHiveViewList(IN PCMHIVE Hive)
// {
//     PCM_VIEW_OF_FILE CmView;
//     PLIST_ENTRY EntryList;
//
//     /* Do NOT destroy the views of read-only hives */
//     ASSERT(Hive->Hive.ReadOnly == FALSE);
//
//     /* Free all the views inside the Pinned View List */
//     while (!IsListEmpty(&Hive->PinViewListHead))
//     {
//         EntryList = RemoveHeadList(&Hive->PinViewListHead);
//
//         CmView = CONTAINING_RECORD(EntryList, CM_VIEW_OF_FILE, PinViewList);
//
//         /* FIXME: Unmap the view if it is mapped */
//
//         ExFreePool(CmView);
//
//         Hive->PinnedViews--;
//     }
//
//     /* The Pinned View List should be empty */
//     ASSERT(IsListEmpty(&Hive->PinViewListHead) == TRUE);
//     ASSERT(Hive->PinnedViews == 0);
//
//     /* Now, free all the views inside the LRU View List */
//     while (!IsListEmpty(&Hive->LRUViewListHead))
//     {
//         EntryList = RemoveHeadList(&Hive->LRUViewListHead);
//
//         CmView = CONTAINING_RECORD(EntryList, CM_VIEW_OF_FILE, LRUViewList);
//
//         /* FIXME: Unmap the view if it is mapped */
//
//         ExFreePool(CmView);
//
//         Hive->MappedViews--;
//     }
//
//     /* The LRU View List should be empty */
//     ASSERT(IsListEmpty(&Hive->LRUViewListHead) == TRUE);
//     ASSERT(Hive->MappedViews == 0);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
