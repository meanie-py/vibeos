use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _GENERAL_LOOKASIDE, _KPRCB, _SLIST_HEADER,
    PNPAGED_LOOKASIDE_LIST, PPAGED_LOOKASIDE_LIST, PGENERAL_LOOKASIDE,
    PALLOCATE_FUNCTION, PFREE_FUNCTION, SIZE_T,
    KeInitializeSpinLock,
    // Use the bindgen PKSPIN_LOCK to get the correct concrete type for KeInitializeSpinLock.
    // ke::PKSPIN_LOCK = *mut usize but bindgen PKSPIN_LOCK = *mut u32 on i386 (same size, different type).
    PKSPIN_LOCK as DdkPKSPIN_LOCK,
};

// ExInterlockedInsertTailList is only in bindgen on amd64/arm64.
// On i386 the same function is exported as ExfInterlockedInsertTailList.
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::ddk::ExInterlockedInsertTailList;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[cfg(target_arch = "x86")]
unsafe fn ExInterlockedInsertTailList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    Lock: DdkPKSPIN_LOCK,
) -> PLIST_ENTRY {
    #[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ExfInterlockedInsertTailList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: DdkPKSPIN_LOCK,
        ) -> PLIST_ENTRY;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ExfInterlockedInsertTailList(
            ListHead: PLIST_ENTRY,
            ListEntry: PLIST_ENTRY,
            Lock: DdkPKSPIN_LOCK,
        ) -> PLIST_ENTRY;
}
    ExfInterlockedInsertTailList(ListHead, ListEntry, Lock)
}

// PALLOCATE_FUNCTION / PFREE_FUNCTION are `system` in the island now, which is the stdcall
// they compile to on i386 and the C they compile to on amd64/arm64 — so one wrapper each
// replaces the two cfg-gated sets. (They were written as two GROUPS rather than adjacent
// twins, so collapse_abi_twins.py, which requires the twin to follow its partner, could not
// see them: the item after the x86 allocate wrapper is the x86 free wrapper.)
/// Wrapper matching the PALLOCATE_FUNCTION signature.
unsafe extern "system" fn ExAllocatePoolWithTag_C(
    PoolType: POOL_TYPE,
    NumberOfBytes: SIZE_T,
    Tag: ULONG,
) -> PVOID {
    ExAllocatePoolWithTag(PoolType, NumberOfBytes as usize, Tag)
}

/// Wrapper matching the PFREE_FUNCTION signature.
unsafe extern "system" fn ExFreePool_C(Buffer: PVOID) {
    ExFreePoolWithTag(Buffer, 0);
}

#[inline]
unsafe fn ExInitializeSListHead(ListHead: *mut _SLIST_HEADER) {
    core::ptr::write_bytes(ListHead as *mut u8, 0, core::mem::size_of::<_SLIST_HEADER>());
}

#[no_mangle]
pub static mut ExpNonPagedLookasideListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut ExpNonPagedLookasideListLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut ExpPagedLookasideListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut ExpPagedLookasideListLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut ExSystemLookasideListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut ExPoolLookasideListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut ExpSmallNPagedPoolLookasideLists: [_GENERAL_LOOKASIDE; NUMBER_POOL_LOOKASIDE_LISTS as usize] = 
    unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ExpSmallPagedPoolLookasideLists: [_GENERAL_LOOKASIDE; NUMBER_POOL_LOOKASIDE_LISTS as usize] = 
    unsafe { core::mem::zeroed() };

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExInitializeSystemLookasideList(
    List: PGENERAL_LOOKASIDE,
    Type: POOL_TYPE,
    Size: ULONG,
    Tag: ULONG,
    MaximumDepth: USHORT,
    ListHead: PLIST_ENTRY,
) {
    let ListHead = opaque_ptr(ListHead);
    (*List).Tag = Tag;
    (*List).Type = Type;
    (*List).Size = Size;
    InsertHeadList(ListHead, core::ptr::addr_of_mut!((*List).ListEntry) as PLIST_ENTRY);
    (*List).MaximumDepth = MaximumDepth;
    (*List).Depth = 2;
    (*List).__bindgen_anon_4.Allocate = Some(ExAllocatePoolWithTag_C);
    (*List).__bindgen_anon_5.Free = Some(ExFreePool_C);
    InitializeSListHead(core::ptr::addr_of_mut!((*List).__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);
    (*List).TotalAllocates = 0;
    (*List).__bindgen_anon_2.AllocateHits = 0;
    (*List).TotalFrees = 0;
    (*List).__bindgen_anon_3.FreeHits = 0;
    (*List).LastTotalAllocates = 0;
    (*List).__bindgen_anon_6.LastAllocateHits = 0;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExInitPoolLookasidePointers() {
    let Prcb = KeGetCurrentPrcb() as *mut _KPRCB;

    for i in 0..NUMBER_POOL_LOOKASIDE_LISTS as usize {
        let Entry = core::ptr::addr_of_mut!(ExpSmallNPagedPoolLookasideLists[i]);
        InitializeSListHead(core::ptr::addr_of_mut!((*Entry).__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);

        (*Prcb).PPNPagedLookasideList[i].P = Entry as PGENERAL_LOOKASIDE;
        (*Prcb).PPNPagedLookasideList[i].L = Entry as PGENERAL_LOOKASIDE;

        let Entry = core::ptr::addr_of_mut!(ExpSmallPagedPoolLookasideLists[i]);
        InitializeSListHead(core::ptr::addr_of_mut!((*Entry).__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);

        (*Prcb).PPPagedLookasideList[i].P = Entry as PGENERAL_LOOKASIDE;
        (*Prcb).PPPagedLookasideList[i].L = Entry as PGENERAL_LOOKASIDE;
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitLookasideLists() {
    InitializeListHead(core::ptr::addr_of_mut!(ExpNonPagedLookasideListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(ExpPagedLookasideListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(ExSystemLookasideListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(ExPoolLookasideListHead) as PLIST_ENTRY);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ExpNonPagedLookasideListLock) as *mut _ as DdkPKSPIN_LOCK);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(ExpPagedLookasideListLock) as *mut _ as DdkPKSPIN_LOCK);

    for i in 0..NUMBER_POOL_LOOKASIDE_LISTS as usize {
        ExInitializeSystemLookasideList(
            core::ptr::addr_of_mut!(ExpSmallNPagedPoolLookasideLists[i]),
            NonPagedPool,
            ((i + 1) * 8) as ULONG,
            u32::from_le_bytes([b'P', b'o', b'o', b'l']),
            256,
            core::ptr::addr_of_mut!(ExPoolLookasideListHead) as PLIST_ENTRY,
        );

        ExInitializeSystemLookasideList(
            core::ptr::addr_of_mut!(ExpSmallPagedPoolLookasideLists[i]),
            PagedPool,
            ((i + 1) * 8) as ULONG,
            u32::from_le_bytes([b'P', b'o', b'o', b'l']),
            256,
            core::ptr::addr_of_mut!(ExPoolLookasideListHead) as PLIST_ENTRY,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExiAllocateFromPagedLookasideList(
    Lookaside: PPAGED_LOOKASIDE_LIST,
) -> PVOID {
    (*Lookaside).L.TotalAllocates += 1;
    let Entry = InterlockedPopEntrySList(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);
    if Entry.is_null() {
        (*Lookaside).L.__bindgen_anon_2.AllocateMisses += 1;
        let allocate_fn = (*Lookaside).L.__bindgen_anon_4.Allocate.unwrap();
        allocate_fn(
            (*Lookaside).L.Type,
            (*Lookaside).L.Size as SIZE_T,
            (*Lookaside).L.Tag,
        )
    } else {
        Entry as PVOID
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExiFreeToPagedLookasideList(
    Lookaside: PPAGED_LOOKASIDE_LIST,
    Entry: PVOID,
) {
    let Entry = opaque_ptr(Entry);
    (*Lookaside).L.TotalFrees += 1;
    if ExQueryDepthSList(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) >= (*Lookaside).L.Depth as u16 {
        (*Lookaside).L.__bindgen_anon_3.FreeMisses += 1;
        let free_fn = (*Lookaside).L.__bindgen_anon_5.Free.unwrap();
        free_fn(Entry);
    } else {
        InterlockedPushEntrySList(
            core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER,
            Entry as PSLIST_ENTRY,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExDeleteNPagedLookasideList(Lookaside: PNPAGED_LOOKASIDE_LIST) {
    let mut OldIrql: KIRQL = 0;

    loop {
        let Entry = InterlockedPopEntrySList(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);
        if Entry.is_null() {
            break;
        }
        let free_fn = (*Lookaside).L.__bindgen_anon_5.Free.unwrap();
        free_fn(Entry as PVOID);
    }

    // [guard] KeAcquireSpinLock(addr_of_mut!(ExpNonPagedLookasideListLock), core::ptr::addr_of_mut!(OldIrql));
    let lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!(ExpNonPagedLookasideListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };
    RemoveEntryList(core::ptr::addr_of_mut!((*Lookaside).L.ListEntry) as PLIST_ENTRY);
    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(lock_guard);
}

#[no_mangle]
pub unsafe extern "system" fn ExDeletePagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST) {
    let mut OldIrql: KIRQL = 0;

    loop {
        let Entry = InterlockedPopEntrySList(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER);
        if Entry.is_null() {
            break;
        }
        let free_fn = (*Lookaside).L.__bindgen_anon_5.Free.unwrap();
        free_fn(Entry as PVOID);
    }

    // [guard] KeAcquireSpinLock(addr_of_mut!(ExpPagedLookasideListLock), core::ptr::addr_of_mut!(OldIrql));
    let lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!(ExpPagedLookasideListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };
    RemoveEntryList(core::ptr::addr_of_mut!((*Lookaside).L.ListEntry) as PLIST_ENTRY);
    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(lock_guard);
}

#[no_mangle]
pub unsafe extern "system" fn ExInitializeNPagedLookasideList(
    Lookaside: PNPAGED_LOOKASIDE_LIST,
    Allocate: PALLOCATE_FUNCTION,
    Free: PFREE_FUNCTION,
    Flags: ULONG,
    Size: usize,
    Tag: ULONG,
    Depth: USHORT,
) {
    ExInitializeSListHead(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead));
    (*Lookaside).L.TotalAllocates = 0;
    (*Lookaside).L.__bindgen_anon_2.AllocateMisses = 0;
    (*Lookaside).L.TotalFrees = 0;
    (*Lookaside).L.__bindgen_anon_3.FreeMisses = 0;
    (*Lookaside).L.Type = NonPagedPool | Flags;
    (*Lookaside).L.Tag = Tag;
    (*Lookaside).L.Size = Size as ULONG;
    (*Lookaside).L.Depth = 4;
    (*Lookaside).L.MaximumDepth = 256;
    (*Lookaside).L.LastTotalAllocates = 0;
    (*Lookaside).L.__bindgen_anon_6.LastAllocateMisses = 0;

    if Allocate.is_some() {
        (*Lookaside).L.__bindgen_anon_4.Allocate = Allocate;
    } else {
        (*Lookaside).L.__bindgen_anon_4.Allocate = Some(ExAllocatePoolWithTag_C);
    }

    if Free.is_some() {
        (*Lookaside).L.__bindgen_anon_5.Free = Free;
    } else {
        (*Lookaside).L.__bindgen_anon_5.Free = Some(ExFreePool_C);
    }

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(ExpNonPagedLookasideListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Lookaside).L.ListEntry) as PLIST_ENTRY,
        core::ptr::addr_of_mut!(ExpNonPagedLookasideListLock) as *mut _ as DdkPKSPIN_LOCK,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExInitializePagedLookasideList(
    Lookaside: PPAGED_LOOKASIDE_LIST,
    Allocate: PALLOCATE_FUNCTION,
    Free: PFREE_FUNCTION,
    Flags: ULONG,
    Size: usize,
    Tag: ULONG,
    Depth: USHORT,
) {
    ExInitializeSListHead(core::ptr::addr_of_mut!((*Lookaside).L.__bindgen_anon_1.ListHead));
    (*Lookaside).L.TotalAllocates = 0;
    (*Lookaside).L.__bindgen_anon_2.AllocateMisses = 0;
    (*Lookaside).L.TotalFrees = 0;
    (*Lookaside).L.__bindgen_anon_3.FreeMisses = 0;
    (*Lookaside).L.Type = PagedPool | Flags;
    (*Lookaside).L.Tag = Tag;
    (*Lookaside).L.Size = Size as ULONG;
    (*Lookaside).L.Depth = 4;
    (*Lookaside).L.MaximumDepth = 256;
    (*Lookaside).L.LastTotalAllocates = 0;
    (*Lookaside).L.__bindgen_anon_6.LastAllocateMisses = 0;

    if Allocate.is_some() {
        (*Lookaside).L.__bindgen_anon_4.Allocate = Allocate;
    } else {
        (*Lookaside).L.__bindgen_anon_4.Allocate = Some(ExAllocatePoolWithTag_C);
    }

    if Free.is_some() {
        (*Lookaside).L.__bindgen_anon_5.Free = Free;
    } else {
        (*Lookaside).L.__bindgen_anon_5.Free = Some(ExFreePool_C);
    }

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(ExpPagedLookasideListHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Lookaside).L.ListEntry) as PLIST_ENTRY,
        core::ptr::addr_of_mut!(ExpPagedLookasideListLock) as *mut _ as DdkPKSPIN_LOCK,
    );
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ex/lookas.c
// * PURPOSE:         Lookaside Lists
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY ExpNonPagedLookasideListHead;
// KSPIN_LOCK ExpNonPagedLookasideListLock;
// LIST_ENTRY ExpPagedLookasideListHead;
// KSPIN_LOCK ExpPagedLookasideListLock;
// LIST_ENTRY ExSystemLookasideListHead;
// LIST_ENTRY ExPoolLookasideListHead;
// GENERAL_LOOKASIDE ExpSmallNPagedPoolLookasideLists[NUMBER_POOL_LOOKASIDE_LISTS];
// GENERAL_LOOKASIDE ExpSmallPagedPoolLookasideLists[NUMBER_POOL_LOOKASIDE_LISTS];
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExInitializeSystemLookasideList(IN PGENERAL_LOOKASIDE List,
//                                 IN POOL_TYPE Type,
//                                 IN ULONG Size,
//                                 IN ULONG Tag,
//                                 IN USHORT MaximumDepth,
//                                 IN PLIST_ENTRY ListHead)
// {
//     /* Initialize the list */
//     List->Tag = Tag;
//     List->Type = Type;
//     List->Size = Size;
//     InsertHeadList(ListHead, &List->ListEntry);
//     List->MaximumDepth = MaximumDepth;
//     List->Depth = 2;
//     List->Allocate = ExAllocatePoolWithTag;
//     List->Free = ExFreePool;
//     InitializeSListHead(&List->ListHead);
//     List->TotalAllocates = 0;
//     List->AllocateHits = 0;
//     List->TotalFrees = 0;
//     List->FreeHits = 0;
//     List->LastTotalAllocates = 0;
//     List->LastAllocateHits = 0;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExInitPoolLookasidePointers(VOID)
// {
//     ULONG i;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PGENERAL_LOOKASIDE Entry;
//
//     /* Loop for all pool lists */
//     for (i = 0; i < NUMBER_POOL_LOOKASIDE_LISTS; i++)
//     {
//         /* Initialize the non-paged list */
//         Entry = &ExpSmallNPagedPoolLookasideLists[i];
//         InitializeSListHead(&Entry->ListHead);
//
//         /* Bind to PRCB */
//         Prcb->PPNPagedLookasideList[i].P = Entry;
//         Prcb->PPNPagedLookasideList[i].L = Entry;
//
//         /* Initialize the paged list */
//         Entry = &ExpSmallPagedPoolLookasideLists[i];
//         InitializeSListHead(&Entry->ListHead);
//
//         /* Bind to PRCB */
//         Prcb->PPPagedLookasideList[i].P = Entry;
//         Prcb->PPPagedLookasideList[i].L = Entry;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// ExpInitLookasideLists(VOID)
// {
//     ULONG i;
//
//     /* Initialize locks and lists */
//     InitializeListHead(&ExpNonPagedLookasideListHead);
//     InitializeListHead(&ExpPagedLookasideListHead);
//     InitializeListHead(&ExSystemLookasideListHead);
//     InitializeListHead(&ExPoolLookasideListHead);
//     KeInitializeSpinLock(&ExpNonPagedLookasideListLock);
//     KeInitializeSpinLock(&ExpPagedLookasideListLock);
//
//     /* Initialize the system lookaside lists */
//     for (i = 0; i < NUMBER_POOL_LOOKASIDE_LISTS; i++)
//     {
//         /* Initialize the non-paged list */
//         ExInitializeSystemLookasideList(&ExpSmallNPagedPoolLookasideLists[i],
//                                         NonPagedPool,
//                                         (i + 1) * 8,
//                                         'looP',
//                                         256,
//                                         &ExPoolLookasideListHead);
//
//         /* Initialize the paged list */
//         ExInitializeSystemLookasideList(&ExpSmallPagedPoolLookasideLists[i],
//                                         PagedPool,
//                                         (i + 1) * 8,
//                                         'looP',
//                                         256,
//                                         &ExPoolLookasideListHead);
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// ExiAllocateFromPagedLookasideList(IN PPAGED_LOOKASIDE_LIST Lookaside)
// {
//     PVOID Entry;
//
//     Lookaside->L.TotalAllocates++;
//     Entry = InterlockedPopEntrySList(&Lookaside->L.ListHead);
//     if (!Entry)
//     {
//         Lookaside->L.AllocateMisses++;
//         Entry = (Lookaside->L.Allocate)(Lookaside->L.Type,
//                                         Lookaside->L.Size,
//                                         Lookaside->L.Tag);
//     }
//     return Entry;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExiFreeToPagedLookasideList(IN PPAGED_LOOKASIDE_LIST  Lookaside,
//                             IN PVOID  Entry)
// {
//     Lookaside->L.TotalFrees++;
//     if (ExQueryDepthSList(&Lookaside->L.ListHead) >= Lookaside->L.Depth)
//     {
//         Lookaside->L.FreeMisses++;
//         (Lookaside->L.Free)(Entry);
//     }
//     else
//     {
//         InterlockedPushEntrySList(&Lookaside->L.ListHead, (PSLIST_ENTRY)Entry);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExDeleteNPagedLookasideList(IN PNPAGED_LOOKASIDE_LIST Lookaside)
// {
//     KIRQL OldIrql;
//     PVOID Entry;
//
//     /* Pop all entries off the stack and release their resources */
//     for (;;)
//     {
//         Entry = InterlockedPopEntrySList(&Lookaside->L.ListHead);
//         if (!Entry) break;
//         (*Lookaside->L.Free)(Entry);
//     }
//
//     /* Remove from list */
//     KeAcquireSpinLock(&ExpNonPagedLookasideListLock, &OldIrql);
//     RemoveEntryList(&Lookaside->L.ListEntry);
//     KeReleaseSpinLock(&ExpNonPagedLookasideListLock, OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExDeletePagedLookasideList(IN PPAGED_LOOKASIDE_LIST Lookaside)
// {
//     KIRQL OldIrql;
//     PVOID Entry;
//
//     /* Pop all entries off the stack and release their resources */
//     for (;;)
//     {
//         Entry = InterlockedPopEntrySList(&Lookaside->L.ListHead);
//         if (!Entry) break;
//         (*Lookaside->L.Free)(Entry);
//     }
//
//     /* Remove from list */
//     KeAcquireSpinLock(&ExpPagedLookasideListLock, &OldIrql);
//     RemoveEntryList(&Lookaside->L.ListEntry);
//     KeReleaseSpinLock(&ExpPagedLookasideListLock, OldIrql);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExInitializeNPagedLookasideList(IN PNPAGED_LOOKASIDE_LIST Lookaside,
//                                 IN PALLOCATE_FUNCTION Allocate OPTIONAL,
//                                 IN PFREE_FUNCTION Free OPTIONAL,
//                                 IN ULONG Flags,
//                                 IN SIZE_T Size,
//                                 IN ULONG Tag,
//                                 IN USHORT Depth)
// {
//     /* Initialize the Header */
//     ExInitializeSListHead(&Lookaside->L.ListHead);
//     Lookaside->L.TotalAllocates = 0;
//     Lookaside->L.AllocateMisses = 0;
//     Lookaside->L.TotalFrees = 0;
//     Lookaside->L.FreeMisses = 0;
//     Lookaside->L.Type = NonPagedPool | Flags;
//     Lookaside->L.Tag = Tag;
//     Lookaside->L.Size = (ULONG)Size;
//     Lookaside->L.Depth = 4;
//     Lookaside->L.MaximumDepth = 256;
//     Lookaside->L.LastTotalAllocates = 0;
//     Lookaside->L.LastAllocateMisses = 0;
//
//     /* Set the Allocate/Free Routines */
//     if (Allocate)
//     {
//         Lookaside->L.Allocate = Allocate;
//     }
//     else
//     {
//         Lookaside->L.Allocate = ExAllocatePoolWithTag;
//     }
//
//     if (Free)
//     {
//         Lookaside->L.Free = Free;
//     }
//     else
//     {
//         Lookaside->L.Free = ExFreePool;
//     }
//
//     /* Insert it into the list */
//     ExInterlockedInsertTailList(&ExpNonPagedLookasideListHead,
//                                 &Lookaside->L.ListEntry,
//                                 &ExpNonPagedLookasideListLock);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExInitializePagedLookasideList(IN PPAGED_LOOKASIDE_LIST Lookaside,
//                                IN PALLOCATE_FUNCTION Allocate OPTIONAL,
//                                IN PFREE_FUNCTION Free OPTIONAL,
//                                IN ULONG Flags,
//                                IN SIZE_T Size,
//                                IN ULONG Tag,
//                                IN USHORT Depth)
// {
//     /* Initialize the Header */
//     ExInitializeSListHead(&Lookaside->L.ListHead);
//     Lookaside->L.TotalAllocates = 0;
//     Lookaside->L.AllocateMisses = 0;
//     Lookaside->L.TotalFrees = 0;
//     Lookaside->L.FreeMisses = 0;
//     Lookaside->L.Type = PagedPool | Flags;
//     Lookaside->L.Tag = Tag;
//     Lookaside->L.Size = (ULONG)Size;
//     Lookaside->L.Depth = 4;
//     Lookaside->L.MaximumDepth = 256;
//     Lookaside->L.LastTotalAllocates = 0;
//     Lookaside->L.LastAllocateMisses = 0;
//
//     /* Set the Allocate/Free Routines */
//     if (Allocate)
//     {
//         Lookaside->L.Allocate = Allocate;
//     }
//     else
//     {
//         Lookaside->L.Allocate = ExAllocatePoolWithTag;
//     }
//
//     if (Free)
//     {
//         Lookaside->L.Free = Free;
//     }
//     else
//     {
//         Lookaside->L.Free = ExFreePool;
//     }
//
//     /* Insert it into the list */
//     ExInterlockedInsertTailList(&ExpPagedLookasideListHead,
//                                 &Lookaside->L.ListEntry,
//                                 &ExpPagedLookasideListLock);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
