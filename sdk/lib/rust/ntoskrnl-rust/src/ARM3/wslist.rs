use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ke::KSPIN_LOCK;
use ntkrnl_sys::ps::PsGetCurrentProcess;

use ntkrnl_sys::ddk::{
    _EPROCESS, _KEVENT, _MMSUPPORT, _MMWSLE, _MMWSL,
    _MMPFN, _MMPTE, _KPROCESS,
    PEPROCESS, PMMSUPPORT, PMMWSL, PMMWSLE,
    ExAcquireRundownProtection, ExReleaseRundownProtection,
    KeAttachProcess, KeDetachProcess, KeInvalidateTlbEntry,
    MiAcquireExpansionLock, MiConvertSharedWorkingSetLockToExclusive,
    MiDecrementShareCount, MiInitializePfnAndMakePteValid,
    MiLockWorkingSetShared, MiReleaseExpansionLock,
    MiUnlockWorkingSet, MiUnlockWorkingSetShared,
    MmGetKernelAddressSpace, PsGetCurrentThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use super::helpers::*;

// MMWSLE_PREVIOUS_FREE_MASK — from mmtypes.h
// On 64-bit: 0x7FFFFFFF (31-bit PreviousFree field); on 32-bit: 0x7FF (11-bit)
#[cfg(target_pointer_width = "64")]
const MMWSLE_PREVIOUS_FREE_MASK: u32 = 0x7FFFFFFF;
#[cfg(target_pointer_width = "32")]
const MMWSLE_PREVIOUS_FREE_MASK: u32 = 0x7FF;

// MI_IS_PROCESS_WORKING_SET: true if it's a process WS (not system cache or session space)
// C: return (WorkingSet != &MmSystemCacheWs) && !WorkingSet->Flags.SessionSpace;
unsafe fn MI_IS_PROCESS_WORKING_SET(WorkingSet: PMMSUPPORT) -> bool {
    let system_cache_ws = core::ptr::addr_of!(MmSystemCacheWs) as *const _MMSUPPORT;
    (WorkingSet as *const _MMSUPPORT != system_cache_ws)
        && ((*WorkingSet).Flags.SessionSpace() == 0)
}

// ExInterlockedInsertTailList — provided by the kernel, not exposed via ntkrnl_sys::ddk
extern "system" {
    fn ExInterlockedInsertTailList(
        ListHead: PLIST_ENTRY,
        ListEntry: PLIST_ENTRY,
        Lock: *mut KSPIN_LOCK,
    ) -> PLIST_ENTRY;
}

// External globals not exposed via ntkrnl-sys hand-written modules
// (MmSystemCacheWorkingSetList, MmSystemCacheWs defined in ARM3/mminit.rs)
// (MmWorkingSetExpansionHead, MmExpansionLock defined in ARM3/session.rs)
// (MmMinimumFreePages, MmPlentyFreePages defined in ARM3/mminit.rs)
extern "C" {
    static mut MmSystemCacheWs: _MMSUPPORT;
    pub static mut MmSystemCacheWorkingSetList: PMMWSL;
    static mut MmWorkingSetExpansionHead: LIST_ENTRY;
    static mut MmExpansionLock: KSPIN_LOCK;
    static mut MmMinimumFreePages: PFN_NUMBER;
    static mut MmPlentyFreePages: PFN_NUMBER;
}

// CONTAINING_RECORD — C macro, implemented as a pointer subtraction
#[inline]
unsafe fn CONTAINING_RECORD_impl(address: PVOID, field_offset: usize) -> PVOID {
    (address as *mut u8).sub(field_offset) as PVOID
}

#[no_mangle]
pub static mut MmWorkingSetList: PMMWSL = core::ptr::null_mut();
#[no_mangle]
pub static mut MmWorkingSetManagerEvent: _KEVENT = unsafe { core::mem::zeroed() };

unsafe fn GetPteTemplateForWsList(WsList: PMMWSL) -> MMPTE {
    if WsList == MmSystemCacheWorkingSetList {
        ValidKernelPte
    } else {
        ValidKernelPteLocal
    }
}

unsafe fn GetNextPageColorForWsList(WsList: PMMWSL) -> ULONG {
    if WsList == MmSystemCacheWorkingSetList {
        MI_GET_NEXT_COLOR()
    } else {
        MI_GET_NEXT_PROCESS_COLOR(PsGetCurrentProcess() as PVOID)
    }
}

// Helper: zero the Long field of a WSLE (erases it)
// The union Long field type is bindgen's ULONG_PTR (c_ulong on i386, c_ulonglong on amd64/arm64).
// Cast with `as _` to let Rust infer the correct concrete type.
#[inline]
unsafe fn wsle_set_long(wsle: PMMWSLE, val: usize) {
    (*wsle).u1.Long = val as _;
}

// Helpers: access Free union variant.
// On i386 (32-bit): NextFree is a bitfield sub-field (accessor method).
// On amd64/arm64 (64-bit): NextFree is a plain struct field.
#[inline]
unsafe fn wsle_free_next(wsle: PMMWSLE) -> LONG {
    #[cfg(target_arch = "x86")]
    { (*wsle).u1.Free.NextFree() }
    #[cfg(not(target_arch = "x86"))]
    { (*wsle).u1.Free.NextFree }
}

#[inline]
unsafe fn wsle_free_prev(wsle: PMMWSLE) -> u32 {
    (*wsle).u1.Free.PreviousFree()
}

#[inline]
unsafe fn wsle_set_free_next(wsle: PMMWSLE, val: LONG) {
    #[cfg(target_arch = "x86")]
    { (*wsle).u1.Free.set_NextFree(val); }
    #[cfg(not(target_arch = "x86"))]
    { (*wsle).u1.Free.NextFree = val; }
}

#[inline]
unsafe fn wsle_set_free_prev(wsle: PMMWSLE, val: u32) {
    (*wsle).u1.Free.set_PreviousFree(val);
}

// Helper: access e1 Valid bit — returns u32 (ULONG from bindgen)
#[inline]
unsafe fn wsle_e1_valid(wsle: PMMWSLE) -> u32 {
    (*wsle).u1.e1.Valid() as u32
}

unsafe fn FreeWsleIndex(WsList: PMMWSL, mut Index: ULONG) {
    let Wsle = (*WsList).Wsle;
    let LastEntry = core::ptr::addr_of_mut!((*WsList).LastEntry);
    let FirstFree = core::ptr::addr_of_mut!((*WsList).FirstFree);
    let LastInitializedWsle = core::ptr::addr_of_mut!((*WsList).LastInitializedWsle);

    // Erase it
    wsle_set_long(Wsle.add(Index as usize), 0);

    if Index == (*LastEntry - 1) {
        // We're freeing the last index of our list.
        while wsle_e1_valid(Wsle.add(Index as usize)) == 0 {
            Index -= 1;
        }

        // Should we bother about the Free entries
        if *FirstFree < Index {
            // Try getting the index of the last free entry
            let PreviousFree = wsle_free_prev(Wsle.add((Index + 1) as usize));
            let mut LastFree = Index + 1 - PreviousFree;

            // MMWSLE_PREVIOUS_FREE_JUMP (32-bit only feature, not enabled)
            // #ifdef MMWSLE_PREVIOUS_FREE_JUMP: skipped

            // Update
            wsle_set_free_prev(
                Wsle.add(*FirstFree as usize),
                (Index + 1 - LastFree) & MMWSLE_PREVIOUS_FREE_MASK,
            );
            wsle_set_free_next(Wsle.add(LastFree as usize), 0);
        } else {
            // No more free entries in our array
            *FirstFree = u32::MAX;
        }

        // This is the new size of our array
        *LastEntry = Index + 1;

        // Should we shrink the alloc?
        while (*LastInitializedWsle - *LastEntry) > (PAGE_SIZE / core::mem::size_of::<_MMWSLE>()) as u32 {
            let PointerPte = MiAddressToPte(Wsle.add((*LastInitializedWsle - 1) as usize) as PVOID);

            let Page = PFN_FROM_PTE(PointerPte);

            {
                // [guard] let OldIrql = MiAcquirePfnLock();
                let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

                let Pfn = MiGetPfnEntry(Page);
                MI_SET_PFN_DELETED(Pfn);
                let pte_frame = (*Pfn).pte_frame();
                MiDecrementShareCount(MiGetPfnEntry(pte_frame) as *mut _MMPFN, pte_frame as _);
                MiDecrementShareCount(Pfn as *mut _MMPFN, Page as _);

                drop(pfn_guard);
            }

            (*PointerPte).Long = 0;

            KeInvalidateTlbEntry(Wsle.add((*LastInitializedWsle - 1) as usize) as PVOID);
            *LastInitializedWsle -= (PAGE_SIZE / core::mem::size_of::<_MMWSLE>()) as u32;
        }
        return;
    }

    if *FirstFree == u32::MAX {
        // We're the first one.
        *FirstFree = Index;
        wsle_set_free_prev(
            Wsle.add(*FirstFree as usize),
            (*LastEntry - *FirstFree) & MMWSLE_PREVIOUS_FREE_MASK,
        );
        return;
    }

    // We must find where to place ourself
    let mut NextFree = *FirstFree;
    let mut PreviousFree = 0u32;
    while NextFree < Index {
        if wsle_free_next(Wsle.add(NextFree as usize)) == 0 {
            break;
        }
        PreviousFree = NextFree;
        NextFree = (NextFree as i32 + wsle_free_next(Wsle.add(NextFree as usize))) as u32;
    }

    if NextFree < Index {
        // This is actually the last free entry
        wsle_set_free_next(Wsle.add(NextFree as usize), (Index - NextFree) as LONG);
        wsle_set_free_prev(
            Wsle.add(Index as usize),
            (Index - NextFree) & MMWSLE_PREVIOUS_FREE_MASK,
        );
        wsle_set_free_prev(
            Wsle.add(*FirstFree as usize),
            (*LastEntry - Index) & MMWSLE_PREVIOUS_FREE_MASK,
        );
        return;
    }

    if PreviousFree == 0 {
        // This is the first free
        wsle_set_free_next(Wsle.add(Index as usize), (*FirstFree - Index) as LONG);
        wsle_set_free_prev(
            Wsle.add(Index as usize),
            wsle_free_prev(Wsle.add(*FirstFree as usize)),
        );
        wsle_set_free_prev(
            Wsle.add(*FirstFree as usize),
            (*FirstFree - Index) & MMWSLE_PREVIOUS_FREE_MASK,
        );
        *FirstFree = Index;
        return;
    }

    // Insert
    wsle_set_free_next(Wsle.add(PreviousFree as usize), (Index - PreviousFree) as LONG);
    wsle_set_free_prev(
        Wsle.add(Index as usize),
        (Index - PreviousFree) & MMWSLE_PREVIOUS_FREE_MASK,
    );
    wsle_set_free_next(Wsle.add(Index as usize), (NextFree - Index) as LONG);
    wsle_set_free_prev(
        Wsle.add(NextFree as usize),
        (NextFree - Index) & MMWSLE_PREVIOUS_FREE_MASK,
    );
}

unsafe fn GetFreeWsleIndex(WsList: PMMWSL) -> ULONG {
    let Index: ULONG;
    if (*WsList).FirstFree != u32::MAX {
        Index = (*WsList).FirstFree;
        let FreeWsle: PMMWSLE = (*WsList).Wsle.add(Index as usize);
        let next_free = wsle_free_next(FreeWsle);
        let prev_free = wsle_free_prev(FreeWsle);
        if next_free != 0 {
            (*WsList).FirstFree = (Index as i32 + next_free) as u32;
            wsle_set_free_prev((*WsList).Wsle.add((*WsList).FirstFree as usize), prev_free);
        } else {
            (*WsList).FirstFree = u32::MAX;
        }
    } else {
        Index = (*WsList).LastEntry;
        (*WsList).LastEntry += 1;
        if Index >= (*WsList).LastInitializedWsle {
            let PointerPte = MiAddressToPte((*WsList).Wsle.add((*WsList).LastInitializedWsle as usize) as PVOID);
            let mut TempPte = GetPteTemplateForWsList(WsList);
            {
                // [guard] let OldIrql = MiAcquirePfnLock();
                let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

                let pfn = MiRemoveAnyPage(GetNextPageColorForWsList(WsList));
                TempPte.set_page_frame_number(pfn);
                MiInitializePfnAndMakePteValid(
                    pfn as _,
                    PointerPte as *mut _MMPTE,
                    core::mem::transmute(TempPte),
                );

                drop(pfn_guard);
            }

            (*WsList).LastInitializedWsle += (PAGE_SIZE / core::mem::size_of::<_MMWSLE>()) as u32;
        }
    }

    wsle_set_long((*WsList).Wsle.add(Index as usize), 0);
    Index
}

unsafe fn RemoveFromWsList(WsList: PMMWSL, Address: PVOID) {
    let PointerPte = MiAddressToPte(Address);

    let Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));

    // u1.WsIndex overlaps with u1.Flink at offset 0
    let ws_index = (*Pfn1).flink() as ULONG;
    FreeWsleIndex(WsList, ws_index);
}

unsafe fn TrimWsList(WsList: PMMWSL) -> ULONG {
    let mut Ret = 0u32;

    let mut i = (*WsList).FirstDynamic;
    while i < (*WsList).LastEntry {
        let Entry: PMMWSLE = (*WsList).Wsle.add(i as usize);
        if (*Entry).u1.e1.Valid() == 0 {
            i += 1;
            continue;
        }

        // Reconstruct virtual address from VirtualPageNumber bitfield
        let VirtualPageNumber = (*Entry).u1.e1.VirtualPageNumber();
        let EntryVA = ((VirtualPageNumber as ULONG_PTR) << PAGE_SHIFT) as PVOID;

        let PointerPte = MiAddressToPte(EntryVA);

        if (*PointerPte).accessed() {
            (*Entry).u1.e1.set_Age(0);
            // Clear the Accessed bit
            (*PointerPte).Long &= !PTE_ACCESSED;
            KeInvalidateTlbEntry(EntryVA);
            i += 1;
            continue;
        }

        let age = (*Entry).u1.e1.Age();
        if age < 3 {
            (*Entry).u1.e1.set_Age(age + 1);
            i += 1;
            continue;
        }

        if ((*Entry).u1.e1.LockedInMemory() != 0) || ((*Entry).u1.e1.LockedInWs() != 0) {
            i += 1;
            continue;
        }

        if MI_IS_PAGE_TABLE_ADDRESS(EntryVA) {
            i += 1;
            continue;
        }

        let Page = PFN_FROM_PTE(PointerPte);
        {
            // [guard] let OldIrql = MiAcquirePfnLock();
            let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

            let Pfn = MiGetPfnEntry(Page);

            // Check if locked via the PFN's Wsle field
            // MMPFN.Wsle is a MMWSLE struct embedded directly
            let PfnWsle: PMMWSLE = core::ptr::addr_of_mut!(
                (*(Pfn as *mut _MMPFN)).Wsle
            ) as PMMWSLE;
            if (*PfnWsle).u1.e1.LockedInMemory() != 0 || (*PfnWsle).u1.e1.LockedInWs() != 0 {
                // [guard-drop] MiReleasePfnLock(OldIrql);
                i += 1;
                continue;
            }

            let Protection = (*Entry).u1.e1.Protection();
            RemoveFromWsList(WsList, EntryVA);

            // Dirtify the page if needed
            if MI_PTE_IS_DIRTY((*PointerPte).Long as u64) {
                (*Pfn).set_modified(true);
            }

            MI_MAKE_TRANSITION_PTE(&mut *PointerPte, Page, Protection as ULONG);
            // C: KeInvalidateTlbEntry(MiAddressToPte(PointerPte)) — this is the PTE that maps PointerPte
            KeInvalidateTlbEntry(MiAddressToPte(PointerPte as PVOID) as PVOID);
            MiDecrementShareCount(Pfn as *mut _MMPFN, Page as _);

            // [guard-drop] MiReleasePfnLock(OldIrql);
        }

        Ret += 1;
        i += 1;
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertInWorkingSetList(
    Vm: PMMSUPPORT,
    Address: PVOID,
    Protection: ULONG,
) {
    let Address = opaque_ptr(Address);
    let WsList = (*Vm).VmWorkingSetList;

    let PointerPte = MiAddressToPte(Address);

    let pfnNum = PFN_FROM_PTE(PointerPte);
    let Pfn1 = MiGetPfnEntry(pfnNum);

    // Set WsIndex (u1.WsIndex overlaps with u1.Flink at offset 0)
    let new_index = GetFreeWsleIndex(WsList);
    (*Pfn1).set_flink(new_index as PFN_NUMBER);

    let NewWsle: PMMWSLE = (*WsList).Wsle.add(new_index as usize);
    wsle_set_long(NewWsle, 0);
    (*NewWsle).u1.e1.set_VirtualPageNumber(((Address as usize) >> PAGE_SHIFT) as _);
    (*NewWsle).u1.e1.set_Protection(Protection as _);
    (*NewWsle).u1.e1.set_Direct(1);
    (*NewWsle).u1.e1.set_Hashed(0);
    (*NewWsle).u1.e1.set_LockedInMemory(0);
    (*NewWsle).u1.e1.set_LockedInWs(0);
    (*NewWsle).u1.e1.set_Age(0);
    (*NewWsle).u1.e1.set_Valid(1);

    (*Vm).WorkingSetSize = (*Vm).WorkingSetSize.wrapping_add(PAGE_SIZE as u32);
    if (*Vm).WorkingSetSize > (*Vm).PeakWorkingSetSize {
        (*Vm).PeakWorkingSetSize = (*Vm).WorkingSetSize;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiRemoveFromWorkingSetList(
    Vm: PMMSUPPORT,
    Address: PVOID,
) {
    let Address = opaque_ptr(Address);
    RemoveFromWsList((*Vm).VmWorkingSetList, Address);

    (*Vm).WorkingSetSize = (*Vm).WorkingSetSize.wrapping_sub(PAGE_SIZE as u32);
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeWorkingSetList(WorkingSet: PMMSUPPORT) {
    let WsList = (*WorkingSet).VmWorkingSetList;

    (*WsList).FirstFree = u32::MAX;
    // WsList->Wsle = WsList + 1 (pointer to MMWSLE array right after the MMWSL header)
    (*WsList).Wsle = (WsList as *mut u8).add(core::mem::size_of::<_MMWSL>()) as PMMWSLE;
    (*WsList).LastEntry = 0;
    // The first page is already allocated
    (*WsList).LastInitializedWsle = ((PAGE_SIZE - core::mem::size_of::<_MMWSL>()) / core::mem::size_of::<_MMWSLE>()) as u32;

    if MI_IS_PROCESS_WORKING_SET(WorkingSet) {
        // Insert PDE/PPE/PXE base addresses depending on paging level
        #[cfg(target_arch = "x86")]
        MiInsertInWorkingSetList(WorkingSet, PDE_BASE as PVOID, 0u32);

        // On 64-bit: 3+ level paging, insert PPE (PDE of WsList)
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            // 4-level: insert PXE_BASE (PPE of WsList is inserted unconditionally below)
            // For now just do PPE (PDE of WsList) as in original
            // The C code inserts PPE_BASE (3-level) or PXE_BASE (4-level)
            // We mirror the logic: for 4-level paging (arm64/amd64), insert PXE_BASE equivalent
            // For simplicity match C:
            // #if _MI_PAGING_LEVELS == 4: PXE_BASE
            // #elif _MI_PAGING_LEVELS == 3: PPE_BASE
            // Since we target amd64 (4) and arm64 (4), insert PXE_BASE
            #[cfg(target_arch = "aarch64")]
            MiInsertInWorkingSetList(WorkingSet, PXE_BASE as PVOID, 0u32);
            #[cfg(target_arch = "x86_64")]
            MiInsertInWorkingSetList(WorkingSet, PXE_BASE as PVOID, 0u32);
        }
    }

    // Insert page table entries for the working set list itself
    // #if _MI_PAGING_LEVELS == 4: insert PPE
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    MiInsertInWorkingSetList(WorkingSet, MiAddressToPpe((*WorkingSet).VmWorkingSetList as PVOID) as PVOID, 0u32);

    // #if _MI_PAGING_LEVELS >= 3: insert PDE
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    MiInsertInWorkingSetList(WorkingSet, MiAddressToPde((*WorkingSet).VmWorkingSetList as PVOID) as PVOID, 0u32);

    MiInsertInWorkingSetList(WorkingSet, MiAddressToPte((*WorkingSet).VmWorkingSetList as PVOID) as PVOID, 0u32);
    MiInsertInWorkingSetList(WorkingSet, (*WorkingSet).VmWorkingSetList as PVOID, 0u32);

    (*WsList).FirstDynamic = (*WsList).LastEntry;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!(MmWorkingSetExpansionHead) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*WorkingSet).WorkingSetExpansionLinks) as PLIST_ENTRY,
        core::ptr::addr_of_mut!(MmExpansionLock),
    );
}

#[no_mangle]
pub unsafe extern "system" fn MmWorkingSetManager() {
    let mut Vm: PMMSUPPORT = core::ptr::null_mut();

    let mut OldIrql = MiAcquireExpansionLock();

    let mut VmListEntry = (*core::ptr::addr_of_mut!(MmWorkingSetExpansionHead)).Flink;
    while VmListEntry != core::ptr::addr_of_mut!(MmWorkingSetExpansionHead) as PLIST_ENTRY {
        let TrimHard = MmAvailablePages < MmMinimumFreePages;
        let mut Process: PEPROCESS = core::ptr::null_mut();

        if (MmAvailablePages + (*core::ptr::addr_of_mut!(MmModifiedPageListHead)).Total as PFN_NUMBER) >= MmPlentyFreePages {
            break;
        }

        Vm = CONTAINING_RECORD_impl(
            VmListEntry as PVOID,
            core::mem::offset_of!(_MMSUPPORT, WorkingSetExpansionLinks),
        ) as PMMSUPPORT;

        if Vm == MmGetKernelAddressSpace() {
            VmListEntry = (*VmListEntry).Flink;
            continue;
        }

        if MI_IS_PROCESS_WORKING_SET(Vm) {
            Process = CONTAINING_RECORD_impl(
                Vm as PVOID,
                core::mem::offset_of!(_EPROCESS, Vm),
            ) as PEPROCESS;

            if ExAcquireRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect)) == 0 {
                VmListEntry = (*VmListEntry).Flink;
                continue;
            }
            // KeAttachProcess takes *mut _KPROCESS (= PKPROCESS); Pcb is KPROCESS = _KPROCESS
            KeAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb) as *mut _KPROCESS);
        } else {
            VmListEntry = (*VmListEntry).Flink;
            continue;
        }

        MiReleaseExpansionLock(OldIrql);

        MiLockWorkingSetShared(PsGetCurrentThread(), Vm);

        if (((*Vm).WorkingSetSize > (*Vm).MaximumWorkingSetSize) ||
            (TrimHard && ((*Vm).WorkingSetSize > (*Vm).MinimumWorkingSetSize))) &&
            MiConvertSharedWorkingSetLockToExclusive(PsGetCurrentThread(), Vm) != 0
        {
            (*Vm).Flags.set_BeingTrimmed(1);

            let Trimmed = TrimWsList((*Vm).VmWorkingSetList);

            (*Vm).WorkingSetSize = (*Vm).WorkingSetSize.wrapping_sub(
                (Trimmed as u32).wrapping_mul(PAGE_SIZE as u32),
            );
            (*Vm).Flags.set_BeingTrimmed(0);
            MiUnlockWorkingSet(PsGetCurrentThread(), Vm);
        } else {
            MiUnlockWorkingSetShared(PsGetCurrentThread(), Vm);
        }

        OldIrql = MiAcquireExpansionLock();

        if !Process.is_null() {
            KeDetachProcess();
            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*Process).RundownProtect));
        }

        VmListEntry = (*VmListEntry).Flink;
    }

    MiReleaseExpansionLock(OldIrql);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD-3-Clause (https://spdx.org/licenses/BSD-3-Clause)
//  * FILE:            ntoskrnl/mm/ARM3/wslist.cpp
//  * PURPOSE:         Working set list management
//  * PROGRAMMERS:     Jérôme Gardou
//  */
//
// /* INCLUDES *******************************************************************/
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include "miarm.h"
//
// /* GLOBALS ********************************************************************/
// PMMWSL MmWorkingSetList;
// KEVENT MmWorkingSetManagerEvent;
//
// /* LOCAL FUNCTIONS ************************************************************/
//
// static MMPTE GetPteTemplateForWsList(PMMWSL WsList)
// {
//     return (WsList == MmSystemCacheWorkingSetList) ? ValidKernelPte : ValidKernelPteLocal;
// }
//
// static ULONG GetNextPageColorForWsList(PMMWSL WsList)
// {
//     return (WsList == MmSystemCacheWorkingSetList) ? MI_GET_NEXT_COLOR() : MI_GET_NEXT_PROCESS_COLOR(PsGetCurrentProcess());
// }
//
// static void FreeWsleIndex(PMMWSL WsList, ULONG Index)
// {
//     PMMWSLE Wsle = WsList->Wsle;
//     ULONG& LastEntry = WsList->LastEntry;
//     ULONG& FirstFree = WsList->FirstFree;
//     ULONG& LastInitializedWsle = WsList->LastInitializedWsle;
//
//     Wsle[Index].u1.Long = 0;
//
//     if (Index == (LastEntry - 1))
//     {
//         while (Wsle[Index].u1.e1.Valid == 0)
//             Index--;
//
//         if (FirstFree < Index)
//         {
//             ASSERT(Wsle[Index + 1].u1.Free.MustBeZero == 0);
//             ULONG PreviousFree = Wsle[Index + 1].u1.Free.PreviousFree;
//             ASSERT(PreviousFree < LastEntry);
//             ULONG LastFree = Index + 1 - PreviousFree;
// #ifdef MMWSLE_PREVIOUS_FREE_JUMP
//             while (Wsle[LastFree].u1.e1.Valid)
//             {
//                 ASSERT(LastFree > MMWSLE_PREVIOUS_FREE_JUMP);
//                 LastFree -= MMWSLE_PREVIOUS_FREE_JUMP;
//             }
// #endif
//             ASSERT(LastFree >= FirstFree);
//             Wsle[FirstFree].u1.Free.PreviousFree = (Index + 1 - LastFree) & MMWSLE_PREVIOUS_FREE_MASK;
//             Wsle[LastFree].u1.Free.NextFree = 0;
//         }
//         else
//         {
//             FirstFree = ULONG_MAX;
//         }
//         LastEntry = Index + 1;
//         while ((LastInitializedWsle - LastEntry) > (PAGE_SIZE / sizeof(MMWSLE)))
//         {
//             PMMPTE PointerPte = MiAddressToPte(Wsle + LastInitializedWsle - 1);
//             ASSERT(MiPteToAddress(PointerPte) != WsList);
//
//             PFN_NUMBER Page = PFN_FROM_PTE(PointerPte);
//
//             {
//                 ntoskrnl::MiPfnLockGuard PfnLock;
//
//                 PMMPFN Pfn = MiGetPfnEntry(Page);
//                 MI_SET_PFN_DELETED(Pfn);
//                 MiDecrementShareCount(MiGetPfnEntry(Pfn->u4.PteFrame), Pfn->u4.PteFrame);
//                 MiDecrementShareCount(Pfn, Page);
//             }
//
//             PointerPte->u.Long = 0;
//
//             KeInvalidateTlbEntry(Wsle + LastInitializedWsle - 1);
//             LastInitializedWsle -= PAGE_SIZE / sizeof(MMWSLE);
//         }
//         return;
//     }
//
//     if (FirstFree == ULONG_MAX)
//     {
//         FirstFree = Index;
//         Wsle[FirstFree].u1.Free.PreviousFree = (LastEntry - FirstFree) & MMWSLE_PREVIOUS_FREE_MASK;
//         return;
//     }
//
//     ULONG NextFree = FirstFree;
//     ULONG PreviousFree = 0;
//     while (NextFree < Index)
//     {
//         ASSERT(Wsle[NextFree].u1.Free.MustBeZero == 0);
//         if (Wsle[NextFree].u1.Free.NextFree == 0)
//             break;
//         PreviousFree = NextFree;
//         NextFree += Wsle[NextFree].u1.Free.NextFree;
//     }
//
//     if (NextFree < Index)
//     {
//         Wsle[NextFree].u1.Free.NextFree = Index - NextFree;
//         Wsle[Index].u1.Free.PreviousFree = (Index - NextFree) & MMWSLE_PREVIOUS_FREE_MASK;
//         Wsle[FirstFree].u1.Free.PreviousFree = (LastEntry - Index) & MMWSLE_PREVIOUS_FREE_MASK;
//         return;
//     }
//
//     if (PreviousFree == 0)
//     {
//         Wsle[Index].u1.Free.NextFree = FirstFree - Index;
//         Wsle[Index].u1.Free.PreviousFree = Wsle[FirstFree].u1.Free.PreviousFree;
//         Wsle[FirstFree].u1.Free.PreviousFree = (FirstFree - Index) & MMWSLE_PREVIOUS_FREE_MASK;
//         FirstFree = Index;
//         return;
//     }
//
//     Wsle[PreviousFree].u1.Free.NextFree = (Index - PreviousFree);
//     Wsle[Index].u1.Free.PreviousFree = (Index - PreviousFree) & MMWSLE_PREVIOUS_FREE_MASK;
//     Wsle[Index].u1.Free.NextFree = NextFree - Index;
//     Wsle[NextFree].u1.Free.PreviousFree = (NextFree - Index) & MMWSLE_PREVIOUS_FREE_MASK;
// }
//
// static ULONG GetFreeWsleIndex(PMMWSL WsList)
// {
//     ULONG Index;
//     if (WsList->FirstFree != ULONG_MAX)
//     {
//         Index = WsList->FirstFree;
//         ASSERT(Index < WsList->LastInitializedWsle);
//         MMWSLE_FREE_ENTRY& FreeWsle = WsList->Wsle[Index].u1.Free;
//         ASSERT(FreeWsle.MustBeZero == 0);
//         if (FreeWsle.NextFree != 0)
//         {
//             WsList->FirstFree += FreeWsle.NextFree;
//             WsList->Wsle[WsList->FirstFree].u1.Free.PreviousFree = FreeWsle.PreviousFree;
//         }
//         else
//         {
//             WsList->FirstFree = ULONG_MAX;
//         }
//     }
//     else
//     {
//         Index = WsList->LastEntry++;
//         if (Index >= WsList->LastInitializedWsle)
//         {
//             PMMPTE PointerPte = MiAddressToPte(&WsList->Wsle[WsList->LastInitializedWsle]);
//             ASSERT(PointerPte->u.Hard.Valid == 0);
//             MMPTE TempPte = GetPteTemplateForWsList(WsList);
//             {
//                 ntoskrnl::MiPfnLockGuard PfnLock;
//
//                 TempPte.u.Hard.PageFrameNumber = MiRemoveAnyPage(GetNextPageColorForWsList(WsList));
//                 MiInitializePfnAndMakePteValid(TempPte.u.Hard.PageFrameNumber, PointerPte, TempPte);
//             }
//
//             WsList->LastInitializedWsle += PAGE_SIZE / sizeof(MMWSLE);
//         }
//     }
//
//     WsList->Wsle[Index].u1.Long = 0;
//     return Index;
// }
//
// static
// VOID
// RemoveFromWsList(PMMWSL WsList, PVOID Address)
// {
//     ASSERT(MM_ANY_WS_LOCK_HELD_EXCLUSIVE(PsGetCurrentThread()));
//
//     PMMPTE PointerPte = MiAddressToPte(Address);
//
//     ASSERT(PointerPte->u.Hard.Valid == 1);
//     PMMPFN Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
//     ASSERT(Pfn1->u3.e1.PageLocation == ActiveAndValid);
//     ASSERT(Pfn1->u3.e1.PrototypePte == 0);
//     ASSERT(MI_IS_ROS_PFN(Pfn1) == FALSE);
//     ASSERT(Pfn1->u1.WsIndex != 0);
//
//     FreeWsleIndex(WsList, Pfn1->u1.WsIndex);
// }
//
// static
// ULONG
// TrimWsList(PMMWSL WsList)
// {
//     ASSERT(MM_ANY_WS_LOCK_HELD(PsGetCurrentThread()));
//
//     ULONG Ret = 0;
//
//     for (ULONG i = WsList->FirstDynamic; i < WsList->LastEntry; i++)
//     {
//         MMWSLE& Entry = WsList->Wsle[i];
//         if (!Entry.u1.e1.Valid)
//             continue;
//
//         ASSERT(Entry.u1.e1.Direct == 1);
//
//         PMMPTE PointerPte = MiAddressToPte(Entry.u1.VirtualAddress);
//
//         ASSERT(PointerPte->u.Hard.Valid);
//
//         if (PointerPte->u.Hard.Accessed)
//         {
//             Entry.u1.e1.Age = 0;
//             PointerPte->u.Hard.Accessed = 0;
//             KeInvalidateTlbEntry(Entry.u1.VirtualAddress);
//             continue;
//         }
//
//         if (Entry.u1.e1.Age < 3)
//         {
//             Entry.u1.e1.Age++;
//             continue;
//         }
//
//         if ((Entry.u1.e1.LockedInMemory) || (Entry.u1.e1.LockedInWs))
//         {
//             continue;
//         }
//
//         if (MI_IS_PAGE_TABLE_ADDRESS(Entry.u1.VirtualAddress))
//             continue;
//
//         PFN_NUMBER Page = PFN_FROM_PTE(PointerPte);
//         {
//             ntoskrnl::MiPfnLockGuard PfnLock;
//
//             PMMPFN Pfn = MiGetPfnEntry(Page);
//
//             ASSERT(Pfn->u3.e1.PrototypePte == 0);
//             ASSERT(!MI_IS_ROS_PFN(Pfn));
//
//             if (Pfn->Wsle.u1.e1.LockedInMemory || (Pfn->Wsle.u1.e1.LockedInWs))
//             {
//                 continue;
//             }
//
//             ULONG Protection = Entry.u1.e1.Protection;
//             RemoveFromWsList(WsList, Entry.u1.VirtualAddress);
//
// #ifdef _M_ARM64
//             if (!PointerPte->u.Hard.NotDirty)
// #else
//             if (PointerPte->u.Hard.Dirty)
// #endif
//                 Pfn->u3.e1.Modified = 1;
//
//             MI_MAKE_TRANSITION_PTE(PointerPte, Page, Protection);
//             KeInvalidateTlbEntry(MiAddressToPte(PointerPte));
//
//             MiDecrementShareCount(Pfn, Page);
//         }
//
//         Ret++;
//     }
//     return Ret;
// }
//
// /* GLOBAL FUNCTIONS ***********************************************************/
// extern "C"
// {
//
// _Use_decl_annotations_
// VOID
// NTAPI
// MiInsertInWorkingSetList(
//     _Inout_ PMMSUPPORT Vm,
//     _In_ PVOID Address,
//     _In_ ULONG Protection)
// {
//     PMMWSL WsList = Vm->VmWorkingSetList;
//
//     ASSERT(MM_ANY_WS_LOCK_HELD_EXCLUSIVE(PsGetCurrentThread()));
//
//     PMMPTE PointerPte = MiAddressToPte(Address);
//     ASSERT(PointerPte->u.Hard.Valid == 1);
//     PFN_NUMBER pfnNum = PFN_FROM_PTE(PointerPte);
//     PMMPFN Pfn1 = MiGetPfnEntry(pfnNum);
//     ASSERT(Pfn1->u3.e1.PageLocation == ActiveAndValid);
//
//     ASSERT(Pfn1->u1.WsIndex == 0);
//     ASSERT(Pfn1->u3.e1.PrototypePte == 0);
//     ASSERT(MI_IS_ROS_PFN(Pfn1) == FALSE);
//
//     Pfn1->u1.WsIndex = GetFreeWsleIndex(WsList);
//     MMWSLENTRY& NewWsle = WsList->Wsle[Pfn1->u1.WsIndex].u1.e1;
//     NewWsle.VirtualPageNumber = reinterpret_cast<ULONG_PTR>(Address) >> PAGE_SHIFT;
//     NewWsle.Protection = Protection;
//     NewWsle.Direct = 1;
//     NewWsle.Hashed = 0;
//     NewWsle.LockedInMemory = 0;
//     NewWsle.LockedInWs = 0;
//     NewWsle.Age = 0;
//     NewWsle.Valid = 1;
//
//     Vm->WorkingSetSize += PAGE_SIZE;
//     if (Vm->WorkingSetSize > Vm->PeakWorkingSetSize)
//         Vm->PeakWorkingSetSize = Vm->WorkingSetSize;
// }
//
// _Use_decl_annotations_
// VOID
// NTAPI
// MiRemoveFromWorkingSetList(
//     _Inout_ PMMSUPPORT Vm,
//     _In_ PVOID Address)
// {
//     RemoveFromWsList(Vm->VmWorkingSetList, Address);
//     Vm->WorkingSetSize -= PAGE_SIZE;
// }
//
// _Use_decl_annotations_
// VOID
// NTAPI
// MiInitializeWorkingSetList(_Inout_ PMMSUPPORT WorkingSet)
// {
//     PMMWSL WsList = WorkingSet->VmWorkingSetList;
//
//     WsList->FirstFree = ULONG_MAX;
//     WsList->Wsle = reinterpret_cast<PMMWSLE>(WsList + 1);
//     WsList->LastEntry = 0;
//     WsList->LastInitializedWsle = (PAGE_SIZE - sizeof(*WsList)) / sizeof(MMWSLE);
//
//     if (MI_IS_PROCESS_WORKING_SET(WorkingSet))
//     {
//         ASSERT(WorkingSet->VmWorkingSetList == MmWorkingSetList);
// #if _MI_PAGING_LEVELS == 4
//         MiInsertInWorkingSetList(WorkingSet, (PVOID)PXE_BASE, 0U);
// #elif _MI_PAGING_LEVELS == 3
//         MiInsertInWorkingSetList(WorkingSet, (PVOID)PPE_BASE, 0U);
// #elif _MI_PAGING_LEVELS == 2
//         MiInsertInWorkingSetList(WorkingSet, (PVOID)PDE_BASE, 0U);
// #endif
//     }
//
// #if _MI_PAGING_LEVELS == 4
//     MiInsertInWorkingSetList(WorkingSet, MiAddressToPpe(WorkingSet->VmWorkingSetList), 0UL);
// #endif
// #if _MI_PAGING_LEVELS >= 3
//     MiInsertInWorkingSetList(WorkingSet, MiAddressToPde(WorkingSet->VmWorkingSetList), 0UL);
// #endif
//     MiInsertInWorkingSetList(WorkingSet, (PVOID)MiAddressToPte(WorkingSet->VmWorkingSetList), 0UL);
//     MiInsertInWorkingSetList(WorkingSet, (PVOID)WorkingSet->VmWorkingSetList, 0UL);
//
//     WsList->FirstDynamic = WsList->LastEntry;
//
//     ExInterlockedInsertTailList(&MmWorkingSetExpansionHead, &WorkingSet->WorkingSetExpansionLinks, &MmExpansionLock);
// }
//
// VOID
// NTAPI
// MmWorkingSetManager(VOID)
// {
//     PLIST_ENTRY VmListEntry;
//     PMMSUPPORT Vm = NULL;
//     KIRQL OldIrql;
//
//     OldIrql = MiAcquireExpansionLock();
//
//     for (VmListEntry = MmWorkingSetExpansionHead.Flink;
//          VmListEntry != &MmWorkingSetExpansionHead;
//          VmListEntry = VmListEntry->Flink)
//     {
//         BOOLEAN TrimHard = MmAvailablePages < MmMinimumFreePages;
//         PEPROCESS Process = NULL;
//
//         if ((MmAvailablePages + MmModifiedPageListHead.Total) >= MmPlentyFreePages)
//             break;
//
//         Vm = CONTAINING_RECORD(VmListEntry, MMSUPPORT, WorkingSetExpansionLinks);
//
//         if (Vm == MmGetKernelAddressSpace())
//             continue;
//
//         if (MI_IS_PROCESS_WORKING_SET(Vm))
//         {
//             Process = CONTAINING_RECORD(Vm, EPROCESS, Vm);
//
//             if (!ExAcquireRundownProtection(&Process->RundownProtect))
//                 continue;
//             ASSERT(!KeIsAttachedProcess());
//             KeAttachProcess(&Process->Pcb);
//         }
//         else
//         {
//             continue;
//         }
//
//         MiReleaseExpansionLock(OldIrql);
//
//         MiLockWorkingSetShared(PsGetCurrentThread(), Vm);
//
//         if (((Vm->WorkingSetSize > Vm->MaximumWorkingSetSize) ||
//             (TrimHard && (Vm->WorkingSetSize > Vm->MinimumWorkingSetSize))) &&
//             MiConvertSharedWorkingSetLockToExclusive(PsGetCurrentThread(), Vm))
//         {
//             Vm->Flags.BeingTrimmed = 1;
//
//             ULONG Trimmed = TrimWsList(Vm->VmWorkingSetList);
//
//             Vm->WorkingSetSize -= Trimmed * PAGE_SIZE;
//             Vm->Flags.BeingTrimmed = 0;
//             MiUnlockWorkingSet(PsGetCurrentThread(), Vm);
//         }
//         else
//         {
//             MiUnlockWorkingSetShared(PsGetCurrentThread(), Vm);
//         }
//
//         OldIrql = MiAcquireExpansionLock();
//
//         if (Process)
//         {
//             KeDetachProcess();
//             ExReleaseRundownProtection(&Process->RundownProtect);
//         }
//     }
//
//     MiReleaseExpansionLock(OldIrql);
// }
//
// } // extern "C"
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
