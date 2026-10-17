use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    KeAttachProcess, KeDetachProcess,
    _EPROCESS,
    PEPROCESS,
    MEMORY_AREA,
    PMEMORY_AREA,
    MM_AVL_TABLE,
    PMM_AVL_TABLE,
    PMMADDRESS_NODE,
    PMMVAD,
    PMMVAD_LONG,
    PMMSUPPORT,
    PMM_FREE_PAGE_FUNC,
    MiCheckForConflictingNode,
    MiFindEmptyAddressRangeDownTree,
    MiFindEmptyAddressRangeInTree,
    MiInsertVad,
    MiLockProcessWorkingSet,
    MiLockProcessWorkingSetUnsafe,
    MiLockWorkingSet,
    MiMakeProtectionMask,
    MiRemoveNode,
    MiRosUnmapViewOfSection,
    MiUnlockProcessWorkingSet,
    MiUnlockProcessWorkingSetUnsafe,
    MiUnlockWorkingSet,
    MmGetAddressSpaceOwner,
    PsGetCurrentThread,
    SWAPENTRY,
};



// Local extern declarations with correct calling convention (NTAPI = extern "system").
// These MUST NOT come from ntkrnl_sys::ddk because bindgen declares some as extern "C",
// producing wrong i386 symbol names (_Foo instead of _Foo@N).
extern "system" {
    fn MmIsPageSwapEntry(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID) -> BOOLEAN;
    fn MmDeletePageFileMapping(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID, SwapEntry: *mut SWAPENTRY);
    fn MmDeletePhysicalMapping(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID, WasDirty: *mut BOOLEAN, Page: *mut ntkrnl_sys::ddk::PFN_NUMBER) -> BOOLEAN;
    // MmDeleteVirtualMapping is NTAPI/stdcall on i386 (extern "system"), NOT cdecl. The
    // no-NTAPI decl in internal/mm.h is MISLEADING — the working ABI is stdcall (@16): the
    // def, and the `VirtFn = extern "system" fn(..)` transmute-call below, are both stdcall.
    // Flipping this to extern "C" desyncs ESP at the VirtFn call → exec-at-0 (field manual §9.99).
    fn MmDeleteVirtualMapping(Process: ntkrnl_sys::ddk::PEPROCESS, Address: PVOID, WasDirty: *mut BOOLEAN, Page: *mut ntkrnl_sys::ddk::PFN_NUMBER) -> BOOLEAN;
}

const MI_STATIC_MEMORY_AREAS: usize = 32;
const TAG_MAREA: u32 = u32::from_be_bytes(*b"Mrea");

#[no_mangle]
pub static mut MiStaticMemoryAreas: [MEMORY_AREA; MI_STATIC_MEMORY_AREAS] = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiStaticMemoryAreaCount: ULONG = 0;

#[no_mangle]
pub static mut MiRosKernelVadRoot: MM_AVL_TABLE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiRosKernelVadRootInitialized: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn MmLocateMemoryAreaByAddress(
    AddressSpace: PMMSUPPORT,
    Address: PVOID,
) -> PMEMORY_AREA {
    let Address = opaque_ptr(Address);
    MmLocateMemoryAreaByRegion(AddressSpace, Address, 1)
}

#[no_mangle]
pub unsafe extern "system" fn MmLocateMemoryAreaByRegion(
    AddressSpace: PMMSUPPORT,
    Address_: PVOID,
    Length: ULONG_PTR,
) -> PMEMORY_AREA {
    let Address_ = opaque_ptr(Address_);
    let StartVpn = Address_ as ULONG_PTR / PAGE_SIZE;
    let EndVpn = (Address_ as ULONG_PTR + Length - 1) / PAGE_SIZE;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let Table: PMM_AVL_TABLE = if !Process.is_null() {
        core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot)
    } else {
        core::ptr::addr_of_mut!(MiRosKernelVadRoot)
    };
    
    let mut Node: PMMADDRESS_NODE = core::ptr::null_mut();
    let Result = MiCheckForConflictingNode(StartVpn as _, EndVpn as _, Table, core::ptr::addr_of_mut!(Node));
    
    if Result != 1 {
        return core::ptr::null_mut();
    }
    
    let Vad = Node as PMMVAD_LONG;
    // MI_IS_MEMORY_AREA_VAD: Spare & 1
    if ((*Vad).u.VadFlags.Spare() & 1) == 0 {
        return core::ptr::null_mut();
    } else {
        Node as PMEMORY_AREA
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmIsAddressRangeFree(
    AddressSpace: PMMSUPPORT,
    Address: PVOID,
    Length: ULONG_PTR,
) -> BOOLEAN {
    let Address = opaque_ptr(Address);
    let StartVpn = Address as ULONG_PTR / PAGE_SIZE;
    let EndVpn = (Address as ULONG_PTR + Length - 1) / PAGE_SIZE;
    let Process = MmGetAddressSpaceOwner(AddressSpace);
    let Table: PMM_AVL_TABLE = if !Process.is_null() {
        core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot)
    } else {
        core::ptr::addr_of_mut!(MiRosKernelVadRoot)
    };
    
    let mut Node: PMMADDRESS_NODE = core::ptr::null_mut();
    let Result = MiCheckForConflictingNode(StartVpn as _, EndVpn as _, Table, core::ptr::addr_of_mut!(Node));
    
    if Result != 1 { 1u8 } else { 0u8 }
}

unsafe fn MmInsertMemoryArea(
    AddressSpace: PMMSUPPORT,
    marea: PMEMORY_AREA,
    Protect: ULONG,
) {
    let Process = MmGetAddressSpaceOwner(AddressSpace) as *mut _EPROCESS;
    
    (*marea).VadNode.u.VadFlags.set_Protection(MiMakeProtectionMask(Protect) as _);
    
    if ((*marea).VadNode.EndingVpn as ULONG_PTR) + 1 < MmSystemRangeStart as ULONG_PTR >> PAGE_SHIFT {
        if (*marea).Type != MEMORY_AREA_OWNED_BY_ARM3 {
            MiLockProcessWorkingSetUnsafe(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
            MiInsertVad(core::ptr::addr_of_mut!((*marea).VadNode), core::ptr::addr_of_mut!((*Process).VadRoot));
            MiUnlockProcessWorkingSetUnsafe(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
        }
    } else {
        if MiRosKernelVadRootInitialized == 0 {
            MiRosKernelVadRoot.BalancedRoot.u1.Parent = core::ptr::addr_of_mut!(MiRosKernelVadRoot.BalancedRoot) as *mut ntkrnl_sys::ddk::_MMADDRESS_NODE;
            MiRosKernelVadRoot.set_Unused(1);
            MiRosKernelVadRootInitialized = 1;
        }
        
        MiLockWorkingSet(PsGetCurrentThread(), (core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT));
        MiInsertVad(core::ptr::addr_of_mut!((*marea).VadNode), core::ptr::addr_of_mut!(MiRosKernelVadRoot));
        MiUnlockWorkingSet(PsGetCurrentThread(), (core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT));
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmFindGap(
    AddressSpace: PMMSUPPORT,
    Length: ULONG_PTR,
    Granularity: ULONG_PTR,
    TopDown: BOOLEAN,
) -> PVOID {
    let Process = MmGetAddressSpaceOwner(AddressSpace) as *mut _EPROCESS;
    if TopDown != 0 {
        let HighestAddress: ULONG_PTR = if !Process.is_null() {
            MM_HIGHEST_VAD_ADDRESS as ULONG_PTR
        } else {
            ULONG_PTR::MAX
        };
        return MmFindGapBelow(AddressSpace, Length, Granularity, HighestAddress);
    }

    let mut Parent: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut StartingAddress: ULONG_PTR = 0;
    let Result = MiFindEmptyAddressRangeInTree(
        Length as _,
        Granularity as _,
        mm_vad_root(Process),
        core::ptr::addr_of_mut!(Parent),
        core::ptr::addr_of_mut!(StartingAddress) as ntkrnl_sys::ddk::PULONG_PTR,
    );
    mm_gap_result(Result as u32, StartingAddress)
}

unsafe fn mm_vad_root(Process: *mut _EPROCESS) -> PMM_AVL_TABLE {
    if !Process.is_null() {
        core::ptr::addr_of_mut!((*Process).VadRoot)
    } else {
        core::ptr::addr_of_mut!(MiRosKernelVadRoot)
    }
}

/// The C returns NULL only for TableFoundNode (1), the searches' "no room" answer. The port
/// tested `== 0`, which is TableEmptyTree: an empty tree's valid address became NULL.
fn mm_gap_result(Result: u32, StartingAddress: ULONG_PTR) -> PVOID {
    if Result == ntkrnl_sys::ddk::_TABLE_SEARCH_RESULT_TableFoundNode as u32 {
        core::ptr::null_mut()
    } else {
        StartingAddress as PVOID
    }
}

/// A top-down gap whose last byte is at or below `HighestAddress`; NULL if none. The ROS
/// section views use it for ZeroBits, which MmFindGap has no way to express.
pub unsafe fn MmFindGapBelow(
    AddressSpace: PMMSUPPORT,
    Length: ULONG_PTR,
    Granularity: ULONG_PTR,
    HighestAddress: ULONG_PTR,
) -> PVOID {
    let Process = MmGetAddressSpaceOwner(AddressSpace) as *mut _EPROCESS;
    let mut Parent: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut StartingAddress: ULONG_PTR = 0;
    let Result = MiFindEmptyAddressRangeDownTree(
        Length as _,
        HighestAddress as _,
        Granularity as _,
        mm_vad_root(Process),
        core::ptr::addr_of_mut!(StartingAddress) as ntkrnl_sys::ddk::PULONG_PTR,
        core::ptr::addr_of_mut!(Parent),
    );
    mm_gap_result(Result as u32, StartingAddress)
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeMemoryArea(
    AddressSpace: PMMSUPPORT,
    MemoryArea: PMEMORY_AREA,
    _KWAIT_REASON_FreePage: PMM_FREE_PAGE_FUNC,
    FreePageContext: PVOID,
) -> NTSTATUS {
    let FreePageContext = opaque_ptr(FreePageContext);
    if (*MemoryArea).Type != MEMORY_AREA_OWNED_BY_ARM3 {
        let CurrentProcess = PsGetCurrentProcess();
        let Process = MmGetAddressSpaceOwner(AddressSpace);
        
        if !Process.is_null() && Process != CurrentProcess as *mut ntkrnl_sys::ddk::_EPROCESS {
            KeAttachProcess(core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).Pcb));
        }
        
        let EndAddress = ((((*MemoryArea).VadNode.EndingVpn as ULONG_PTR) + 1) << PAGE_SHIFT) as PVOID;
        let mut Address = (((*MemoryArea).VadNode.StartingVpn as ULONG_PTR) << PAGE_SHIFT) as ULONG_PTR;
        
        // Use volatile reads of function pointers (via usize) to prevent the
        // optimizer from performing interprocedural analysis that eliminates
        // conditional branches. Without this, rustc/LLVM at opt-level>=1 analyzes
        // MmIsPageSwapEntry's body, proves certain branches are dead, and eliminates
        // MmDeletePhysicalMapping/MmDeleteVirtualMapping calls despite their page
        // table modification side effects (atomic PTE exchanges, TLB invalidation).
        type IsSwapFn = unsafe extern "system" fn(PEPROCESS, PVOID) -> BOOLEAN;
        type PageFileFn = unsafe extern "system" fn(PEPROCESS, PVOID, *mut SWAPENTRY);
        type PhysFn = unsafe extern "system" fn(PEPROCESS, PVOID, *mut BOOLEAN, *mut ntkrnl_sys::ddk::PFN_NUMBER) -> BOOLEAN;
        type VirtFn = unsafe extern "system" fn(PEPROCESS, PVOID, *mut BOOLEAN, *mut ntkrnl_sys::ddk::PFN_NUMBER) -> BOOLEAN;
        // Cast fn items → usize, volatile-read to make opaque, transmute back.
        // The usize intermediate avoids calling convention type mismatches between
        // bindgen's extern "stdcall"/"C" and our extern "system" type aliases.
        let is_swap_fn: IsSwapFn = {
            let mut p = MmIsPageSwapEntry as usize;
            p = core::ptr::read_volatile(&p);
            core::mem::transmute(p)
        };
        let pagefile_fn: PageFileFn = {
            let mut p = MmDeletePageFileMapping as usize;
            p = core::ptr::read_volatile(&p);
            core::mem::transmute(p)
        };
        let phys_fn: PhysFn = {
            let mut p = MmDeletePhysicalMapping as usize;
            p = core::ptr::read_volatile(&p);
            core::mem::transmute(p)
        };
        let virt_fn: VirtFn = {
            let mut p = MmDeleteVirtualMapping as usize;
            p = core::ptr::read_volatile(&p);
            core::mem::transmute(p)
        };

        while Address < EndAddress as ULONG_PTR {
            let mut Dirty: BOOLEAN = 0;
            let mut SwapEntry: SWAPENTRY = 0;
            let mut Page: ntkrnl_sys::ddk::PFN_NUMBER = 0;
            let DoFree: BOOLEAN;

            if is_swap_fn(Process, Address as PVOID) != 0 {
                pagefile_fn(Process, Address as PVOID, core::ptr::addr_of_mut!(SwapEntry));
                DoFree = 1;
            } else if _KWAIT_REASON_FreePage.is_none() {
                DoFree = phys_fn(
                    Process,
                    Address as PVOID,
                    core::ptr::addr_of_mut!(Dirty),
                    core::ptr::addr_of_mut!(Page),
                );
            } else {
                DoFree = virt_fn(
                    Process,
                    Address as PVOID,
                    core::ptr::addr_of_mut!(Dirty),
                    core::ptr::addr_of_mut!(Page),
                );
            };

            if DoFree != 0 && _KWAIT_REASON_FreePage.is_some() {
                (_KWAIT_REASON_FreePage.unwrap())(
                    FreePageContext,
                    MemoryArea,
                    Address as PVOID,
                    Page,
                    SwapEntry,
                    Dirty,
                );
            }

            Address += PAGE_SIZE;
        }
        
        if ((*MemoryArea).VadNode.StartingVpn as ULONG_PTR) < MmSystemRangeStart as ULONG_PTR >> PAGE_SHIFT {
            MiLockProcessWorkingSet(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
            MiRemoveNode(
                core::ptr::addr_of_mut!((*MemoryArea).VadNode) as PMMADDRESS_NODE,
                core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot),
            );
            MiUnlockProcessWorkingSet(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
        } else {
            MiLockWorkingSet(PsGetCurrentThread(), (core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT));
            MiRemoveNode(
                core::ptr::addr_of_mut!((*MemoryArea).VadNode) as PMMADDRESS_NODE,
                core::ptr::addr_of_mut!(MiRosKernelVadRoot),
            );
            MiUnlockWorkingSet(PsGetCurrentThread(), (core::ptr::addr_of_mut!(MmSystemCacheWs) as ntkrnl_sys::ddk::PMMSUPPORT));
        }
        
        if !Process.is_null() && Process != CurrentProcess as *mut ntkrnl_sys::ddk::_EPROCESS {
            KeDetachProcess();
        }
    }
    
    ExFreePoolWithTag(MemoryArea as PVOID, TAG_MAREA);
    
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateMemoryArea(
    AddressSpace: PMMSUPPORT,
    Type: ULONG,
    BaseAddress: *mut PVOID,
    Length: ULONG_PTR,
    Protect: ULONG,
    Result: *mut PMEMORY_AREA,
    AllocationFlags: ULONG,
    Granularity: ULONG,
) -> NTSTATUS {
    let BaseAddress = opaque_ptr(BaseAddress);
    let Result = opaque_ptr(Result);
    let MemoryArea: PMEMORY_AREA;
    
    if (Type & MEMORY_AREA_STATIC) != 0 {
        MemoryArea = core::ptr::addr_of_mut!(MiStaticMemoryAreas[MiStaticMemoryAreaCount as usize]);
        MiStaticMemoryAreaCount += 1;
    } else {
        MemoryArea = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<MEMORY_AREA>() as SIZE_T,
            TAG_MAREA,
        ) as PMEMORY_AREA;
    }
    
    if MemoryArea.is_null() {
        return STATUS_NO_MEMORY;
    }
    
    RtlZeroMemory(MemoryArea as PVOID, core::mem::size_of::<MEMORY_AREA>() as SIZE_T);
    (*MemoryArea).Type = Type & !MEMORY_AREA_STATIC;
    (*MemoryArea).Flags = AllocationFlags;
    (*MemoryArea).Magic = u32::from_be_bytes(*b"erAM");
    (*MemoryArea).DeleteInProgress = 0;
    // MI_SET_MEMORY_AREA_VAD: Spare |= 1
    let spare = (*MemoryArea).VadNode.u.VadFlags.Spare();
    (*MemoryArea).VadNode.u.VadFlags.set_Spare(spare | 1);

    if (*MemoryArea).Type != MEMORY_AREA_OWNED_BY_ARM3 {
        // MI_SET_ROSMM_VAD: Spare |= 2
        let spare = (*MemoryArea).VadNode.u.VadFlags.Spare();
        (*MemoryArea).VadNode.u.VadFlags.set_Spare(spare | 2);
    }
    
    if (*BaseAddress).is_null() {
        let tmpLength = ((Length + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;
        let found_base = MmFindGap(
            AddressSpace,
            tmpLength,
            Granularity as ULONG_PTR,
            if (AllocationFlags & MEM_TOP_DOWN as ULONG) == MEM_TOP_DOWN as ULONG { 1 } else { 0 },
        );
        core::ptr::write_volatile(BaseAddress, found_base);
        
        if found_base.is_null() {
            if (Type & MEMORY_AREA_STATIC) == 0 {
                ExFreePoolWithTag(MemoryArea as PVOID, TAG_MAREA);
            }
            return STATUS_NO_MEMORY;
        }
        
        (*MemoryArea).VadNode.StartingVpn = (found_base as ULONG_PTR >> PAGE_SHIFT) as _;
        (*MemoryArea).VadNode.EndingVpn = ((found_base as ULONG_PTR + tmpLength - 1) >> PAGE_SHIFT) as _;
        MmInsertMemoryArea(AddressSpace, MemoryArea, Protect);
    } else {
        let input_base = core::ptr::read_volatile(BaseAddress);
        let EndingAddress = (input_base as ULONG_PTR + Length - 1) | (PAGE_SIZE - 1);
        let aligned_base = (input_base as ULONG_PTR & !(Granularity as usize - 1)) as PVOID;
        core::ptr::write_volatile(BaseAddress, aligned_base);
        let tmpLength = EndingAddress + 1 - aligned_base as ULONG_PTR;
        
        if MmGetAddressSpaceOwner(AddressSpace).is_null() && aligned_base < MmSystemRangeStart {
            if (Type & MEMORY_AREA_STATIC) == 0 {
                ExFreePoolWithTag(MemoryArea as PVOID, TAG_MAREA);
            }
            return STATUS_ACCESS_VIOLATION;
        }
        
        if !MmGetAddressSpaceOwner(AddressSpace).is_null() &&
            aligned_base as ULONG_PTR + (tmpLength as ULONG_PTR) > MmSystemRangeStart as ULONG_PTR {
            if (Type & MEMORY_AREA_STATIC) == 0 {
                ExFreePoolWithTag(MemoryArea as PVOID, TAG_MAREA);
            }
            return STATUS_ACCESS_VIOLATION;
        }
        
        if (*MemoryArea).Type != MEMORY_AREA_OWNED_BY_ARM3 {
            if MmIsAddressRangeFree(AddressSpace, aligned_base, tmpLength) == 0 {
                if (Type & MEMORY_AREA_STATIC) == 0 {
                    ExFreePoolWithTag(MemoryArea as PVOID, TAG_MAREA);
                }
                return STATUS_CONFLICTING_ADDRESSES;
            }
        }
        
        (*MemoryArea).VadNode.StartingVpn = (aligned_base as ULONG_PTR >> PAGE_SHIFT) as _;
        (*MemoryArea).VadNode.EndingVpn = ((aligned_base as ULONG_PTR + tmpLength - 1) >> PAGE_SHIFT) as _;
        MmInsertMemoryArea(AddressSpace, MemoryArea, Protect);
    }
    
    // ARM64: a freshly reserved USER range must not inherit page-table content.
    // Recycled FreeLDR page-table pages leave valid-looking descriptors behind;
    // see MiPurgeStaleUserPtes for the full mechanism and what it costs when it
    // is missing. Both branches above have set Starting/EndingVpn by here, so
    // this is the one place that covers VirtualAlloc and section views alike.
    #[cfg(target_arch = "aarch64")]
    {
        if !MmGetAddressSpaceOwner(AddressSpace).is_null() {
            let start = ((*MemoryArea).VadNode.StartingVpn as ULONG_PTR) << PAGE_SHIFT;
            let end = (((*MemoryArea).VadNode.EndingVpn as ULONG_PTR) + 1) << PAGE_SHIFT;
            crate::arm64::page::MiPurgeStaleUserPtes(start, end);
        }
    }

    core::ptr::write_volatile(Result, MemoryArea);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MiRosCleanupMemoryArea(
    Process: PEPROCESS,
    Vad: PMMVAD,
) {
    let Process = opaque_ptr(Process);
    let Vad = opaque_ptr(Vad);
    let MemoryArea = Vad as PMEMORY_AREA;
    let BaseAddress = (((*MemoryArea).VadNode.StartingVpn as ULONG_PTR) << PAGE_SHIFT) as PVOID;
    
    let Status = if (*MemoryArea).Type == MEMORY_AREA_SECTION_VIEW {
        MiRosUnmapViewOfSection(Process, MemoryArea, BaseAddress, (*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.ProcessExiting() as BOOLEAN)
    } else {
        STATUS_SUCCESS
    };
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright (C) 1998-2005 ReactOS Team (and the authors from the programmers section)
//  *
//  * This program is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU General Public License
//  * as published by the Free Software Foundation; either version 2
//  * of the License, or (at your option) any later version.
//  *
//  * This program is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  * GNU General Public License for more details.
//  *
//  * You should have received a copy of the GNU General Public License
//  * along with this program; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.
//  *
//  *
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/marea.c
//  * PURPOSE:         Implements memory areas
//  *
//  * PROGRAMMERS:     Rex Jolliff
//  *                  David Welch
//  *                  Eric Kohl
//  *                  Philip Susi
//  *                  Casper Hornstrup
//  *                  Eric Kohl
//  *                  Ge van Geldorp
//  *                  Royce Mitchell III
//  *                  Aleksey Bragin
//  *                  Jason Filby
//  *                  Thomas Weidenmueller
//  *                  Gunnar Andre' Dalsnes
//  *                  Mike Nordell
//  *                  Alex Ionescu
//  *                  Filip Navara
//  *                  Herve Poussineau
//  *                  Steven Edwards
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <cache/section/newmm.h>
// #include <debug.h>
//
// #include "ARM3/miarm.h"
//
// MEMORY_AREA MiStaticMemoryAreas[MI_STATIC_MEMORY_AREAS];
// ULONG MiStaticMemoryAreaCount;
//
// MM_AVL_TABLE MiRosKernelVadRoot;
// BOOLEAN MiRosKernelVadRootInitialized;
//
// /* FUNCTIONS *****************************************************************/
//
// PMEMORY_AREA
// NTAPI
// MmLocateMemoryAreaByAddress(
//     PMMSUPPORT AddressSpace,
//     PVOID Address)
// {
//     /* Do it the simple way */
//     return MmLocateMemoryAreaByRegion(AddressSpace, Address, 1);
// }
//
// PMEMORY_AREA
// NTAPI
// MmLocateMemoryAreaByRegion(
//     PMMSUPPORT AddressSpace,
//     PVOID Address_,
//     ULONG_PTR Length)
// {
//     ULONG_PTR StartVpn = (ULONG_PTR)Address_ / PAGE_SIZE;
//     ULONG_PTR EndVpn = ((ULONG_PTR)Address_ + Length - 1) / PAGE_SIZE;
//     PEPROCESS Process;
//     PMM_AVL_TABLE Table;
//     PMMADDRESS_NODE Node;
//     PMEMORY_AREA MemoryArea;
//     TABLE_SEARCH_RESULT Result;
//     PMMVAD_LONG Vad;
//
//     Process = MmGetAddressSpaceOwner(AddressSpace);
//     Table = (Process != NULL) ? &Process->VadRoot : &MiRosKernelVadRoot;
//
//     Result = MiCheckForConflictingNode(StartVpn as _, EndVpn as _, Table, &Node);
//     if (Result != TableFoundNode)
//     {
//         return NULL;
//     }
//
//     Vad = (PMMVAD_LONG)Node;
//     if (!MI_IS_MEMORY_AREA_VAD(Vad))
//     {
//         /* This is an ARM3 VAD, we don't return it. */
//         return NULL;
//     }
//     else
//     {
//         MemoryArea = (PMEMORY_AREA)Node;
//     }
//
//     ASSERT(MemoryArea != NULL);
//     return MemoryArea;
// }
//
// BOOLEAN
// NTAPI
// MmIsAddressRangeFree(
//     _In_ PMMSUPPORT AddressSpace,
//     _In_ PVOID Address,
//     _In_ ULONG_PTR Length)
// {
//     ULONG_PTR StartVpn = (ULONG_PTR)Address / PAGE_SIZE;
//     ULONG_PTR EndVpn = ((ULONG_PTR)Address + Length - 1) / PAGE_SIZE;
//     PEPROCESS Process;
//     PMM_AVL_TABLE Table;
//     PMMADDRESS_NODE Node;
//     TABLE_SEARCH_RESULT Result;
//
//     Process = MmGetAddressSpaceOwner(AddressSpace);
//     Table = (Process != NULL) ? &Process->VadRoot : &MiRosKernelVadRoot;
//
//     Result = MiCheckForConflictingNode(StartVpn as _, EndVpn as _, Table, &Node);
//
//     return (Result != TableFoundNode);
// }
//
// VOID
// NTAPI
// MiInsertVad(IN PMMVAD Vad,
//             IN PMM_AVL_TABLE VadRoot);
//
// ULONG
// NTAPI
// MiMakeProtectionMask(
//     IN ULONG Protect
// );
//
//
// static VOID
// MmInsertMemoryArea(
//     PMMSUPPORT AddressSpace,
//     PMEMORY_AREA marea,
//     ULONG Protect)
// {
//     PEPROCESS Process = MmGetAddressSpaceOwner(AddressSpace);
//
//     marea->VadNode.u.VadFlags.Protection() = MiMakeProtectionMask(Protect);
//
//     /* Build a lame VAD if this is a user-space allocation */
//     if (marea->VadNode.EndingVpn + 1 < (ULONG_PTR)MmSystemRangeStart >> PAGE_SHIFT)
//     {
//         ASSERT(Process != NULL);
//         if (marea->Type != MEMORY_AREA_OWNED_BY_ARM3)
//         {
// #ifdef NEWCC
//             ASSERT(marea->Type == MEMORY_AREA_SECTION_VIEW || marea->Type == MEMORY_AREA_CACHE);
// #else
//             ASSERT(marea->Type == MEMORY_AREA_SECTION_VIEW);
// #endif
//
//             /* Insert the VAD */
//             MiLockProcessWorkingSetUnsafe(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
//             MiInsertVad(&marea->VadNode, &Process->VadRoot);
//             MiUnlockProcessWorkingSetUnsafe(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
//         }
//     }
//     else
//     {
//         ASSERT(Process == NULL);
//
//         if (!MiRosKernelVadRootInitialized)
//         {
//             MiRosKernelVadRoot.BalancedRoot.u1.Parent = &MiRosKernelVadRoot.BalancedRoot;
//             MiRosKernelVadRoot.set_Unused(1);
//             MiRosKernelVadRootInitialized = TRUE;
//         }
//
//         /* Insert the VAD */
//         MiLockWorkingSet(PsGetCurrentThread(), &MmSystemCacheWs);
//         MiInsertVad(&marea->VadNode, &MiRosKernelVadRoot);
//         MiUnlockWorkingSet(PsGetCurrentThread(), &MmSystemCacheWs);
//     }
// }
//
// PVOID NTAPI
// MmFindGap(
//     PMMSUPPORT AddressSpace,
//     ULONG_PTR Length,
//     ULONG_PTR Granularity,
//     BOOLEAN TopDown)
// {
//     PEPROCESS Process;
//     PMM_AVL_TABLE VadRoot;
//     TABLE_SEARCH_RESULT Result;
//     PMMADDRESS_NODE Parent;
//     ULONG_PTR StartingAddress, HighestAddress;
//
//     Process = MmGetAddressSpaceOwner(AddressSpace);
//     VadRoot = Process ? &Process->VadRoot : &MiRosKernelVadRoot;
//     if (TopDown)
//     {
//         /* Find an address top-down */
//         HighestAddress = Process ? (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS : (LONG_PTR)-1;
//         Result = MiFindEmptyAddressRangeDownTree(Length,
//                                                  HighestAddress,
//                                                  Granularity,
//                                                  VadRoot,
//                                                  &StartingAddress,
//                                                  &Parent);
//     }
//     else
//     {
//         Result = MiFindEmptyAddressRangeInTree(Length,
//                                                Granularity,
//                                                VadRoot,
//                                                &Parent,
//                                                &StartingAddress);
//     }
//
//     if (Result == TableFoundNode)
//     {
//         return NULL;
//     }
//
//     return (PVOID)StartingAddress;
// }
//
// VOID
// NTAPI
// MiRemoveNode(IN PMMADDRESS_NODE Node,
//              IN PMM_AVL_TABLE Table);
//
//
// /**
//  * @name MmFreeMemoryArea
//  *
//  * Free an existing memory area.
//  *
//  * @param AddressSpace
//  *        Address space to free the area from.
//  * @param MemoryArea
//  *        Memory area we're about to free.
//  * @param FreePage
//  *        Callback function for each freed page.
//  * @param FreePageContext
//  *        Context passed to the callback function.
//  *
//  * @return Status
//  *
//  * @remarks Lock the address space before calling this function.
//  */
//
// NTSTATUS NTAPI
// MmFreeMemoryArea(
//     PMMSUPPORT AddressSpace,
//     PMEMORY_AREA MemoryArea,
//     PMM_FREE_PAGE_FUNC FreePage,
//     PVOID FreePageContext)
// {
//     ULONG_PTR Address;
//     PVOID EndAddress;
//
//     /* Make sure we own the address space lock! */
//     ASSERT(CONTAINING_RECORD(AddressSpace, EPROCESS, Vm)->AddressCreationLock.Owner == KeGetCurrentThread());
//
//     /* Check magic */
//     ASSERT(MemoryArea->Magic == 'erAM');
//
//     if (MemoryArea->Type != MEMORY_AREA_OWNED_BY_ARM3)
//     {
//         PEPROCESS CurrentProcess = PsGetCurrentProcess();
//         PEPROCESS Process = MmGetAddressSpaceOwner(AddressSpace);
//
//         if ((Process != NULL) && (Process != CurrentProcess))
//         {
//             KeAttachProcess(&Process->Pcb);
//         }
//
//         EndAddress = MM_ROUND_UP(MA_GetEndingAddress(MemoryArea), PAGE_SIZE);
//         for (Address = MA_GetStartingAddress(MemoryArea);
//                 Address < (ULONG_PTR)EndAddress;
//                 Address += PAGE_SIZE)
//         {
//             BOOLEAN Dirty = FALSE;
//             SWAPENTRY SwapEntry = 0;
//             PFN_NUMBER Page = 0;
//             BOOLEAN DoFree;
//
//             if (MmIsPageSwapEntry(Process, (PVOID)Address))
//             {
//                 MmDeletePageFileMapping(Process, (PVOID)Address, &SwapEntry);
//                 /* We'll have to do some cleanup when we're on the page file */
//                 DoFree = TRUE;
//             }
//             else if (FreePage == NULL)
//             {
//                 DoFree = MmDeletePhysicalMapping(Process, (PVOID)Address, &Dirty, &Page);
//             }
//             else
//             {
//                 DoFree = MmDeleteVirtualMapping(Process, (PVOID)Address, &Dirty, &Page);
//             }
//             if (DoFree && (FreePage != NULL))
//             {
//                 FreePage(FreePageContext, MemoryArea, (PVOID)Address,
//                          Page, SwapEntry, (BOOLEAN)Dirty);
//             }
//         }
//
//         if (MemoryArea->VadNode.StartingVpn < (ULONG_PTR)MmSystemRangeStart >> PAGE_SHIFT)
//         {
//             ASSERT(MemoryArea->VadNode.EndingVpn + 1 < (ULONG_PTR)MmSystemRangeStart >> PAGE_SHIFT);
// #ifdef NEWCC
//             ASSERT(MemoryArea->Type == MEMORY_AREA_SECTION_VIEW || MemoryArea->Type == MEMORY_AREA_CACHE);
// #else
//             ASSERT(MemoryArea->Type == MEMORY_AREA_SECTION_VIEW);
// #endif
//
//             /* We do not have fake ARM3 memory areas anymore. */
//             ASSERT(MI_IS_MEMORY_AREA_VAD(&MemoryArea->VadNode));
//             MiLockProcessWorkingSet(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
//             MiRemoveNode((PMMADDRESS_NODE)&MemoryArea->VadNode, &Process->VadRoot);
//             MiUnlockProcessWorkingSet(PsGetCurrentProcess() as ntkrnl_sys::ddk::PEPROCESS, PsGetCurrentThread() as ntkrnl_sys::ddk::PETHREAD);
//         }
//         else
//         {
//             MiLockWorkingSet(PsGetCurrentThread(), &MmSystemCacheWs);
//             MiRemoveNode((PMMADDRESS_NODE)&MemoryArea->VadNode, &MiRosKernelVadRoot);
//             MiUnlockWorkingSet(PsGetCurrentThread(), &MmSystemCacheWs);
//         }
//
//         if ((Process != NULL) && (Process != CurrentProcess))
//         {
//             KeDetachProcess();
//         }
//     }
//
// #if DBG
//     MemoryArea->Magic = 'daeD';
// #endif
//     ExFreePoolWithTag(MemoryArea, TAG_MAREA);
//
//     DPRINT("MmFreeMemoryArea() succeeded\n");
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @name MmCreateMemoryArea
//  *
//  * Create a memory area.
//  *
//  * @param AddressSpace
//  *        Address space to create the area in.
//  * @param Type
//  *        Type of the memory area.
//  * @param BaseAddress
//  *        Base address for the memory area we're about the create. On
//  *        input it contains either 0 (auto-assign address) or preferred
//  *        address. On output it contains the starting address of the
//  *        newly created area.
//  * @param Length
//  *        Length of the area to allocate.
//  * @param Attributes
//  *        Protection attributes for the memory area.
//  * @param Result
//  *        Receives a pointer to the memory area on successful exit.
//  *
//  * @return Status
//  *
//  * @remarks Lock the address space before calling this function.
//  */
//
// NTSTATUS NTAPI
// MmCreateMemoryArea(PMMSUPPORT AddressSpace,
//                    ULONG Type,
//                    PVOID *BaseAddress,
//                    ULONG_PTR Length,
//                    ULONG Protect,
//                    PMEMORY_AREA *Result,
//                    ULONG AllocationFlags,
//                    ULONG Granularity)
// {
//     ULONG_PTR tmpLength;
//     PMEMORY_AREA MemoryArea;
//     ULONG_PTR EndingAddress;
//
//     DPRINT("MmCreateMemoryArea(Type 0x%lx, BaseAddress %p, "
//            "*BaseAddress %p, Length %p, AllocationFlags %x, "
//            "Result %p)\n",
//            Type, BaseAddress, *BaseAddress, Length, AllocationFlags,
//            Result);
//
//     /* Is this a static memory area? */
//     if (Type & MEMORY_AREA_STATIC)
//     {
//         /* Use the static array instead of the pool */
//         ASSERT(MiStaticMemoryAreaCount < MI_STATIC_MEMORY_AREAS);
//         MemoryArea = &MiStaticMemoryAreas[MiStaticMemoryAreaCount++];
//     }
//     else
//     {
//         /* Allocate the memory area from nonpaged pool */
//         MemoryArea = ExAllocatePoolWithTag(NonPagedPool,
//                                            sizeof(MEMORY_AREA),
//                                            TAG_MAREA);
//     }
//
//     if (!MemoryArea)
//     {
//         DPRINT1("Not enough memory.\n");
//         return STATUS_NO_MEMORY;
//     }
//
//     RtlZeroMemory(MemoryArea, sizeof(MEMORY_AREA));
//     MemoryArea->Type = Type & ~MEMORY_AREA_STATIC;
//     MemoryArea->Flags = AllocationFlags;
//     MemoryArea->Magic = 'erAM';
//     MemoryArea->DeleteInProgress = FALSE;
//     MI_SET_MEMORY_AREA_VAD(&MemoryArea->VadNode);
//     if (MemoryArea->Type != MEMORY_AREA_OWNED_BY_ARM3)
//     {
//         MI_SET_ROSMM_VAD(&MemoryArea->VadNode);
//     }
//
//     if (*BaseAddress == 0)
//     {
//         tmpLength = (ULONG_PTR)MM_ROUND_UP(Length, PAGE_SIZE);
//         *BaseAddress = MmFindGap(AddressSpace,
//                                  tmpLength,
//                                  Granularity,
//                                  (AllocationFlags & MEM_TOP_DOWN) == MEM_TOP_DOWN);
//         if ((*BaseAddress) == 0)
//         {
//             DPRINT("No suitable gap\n");
//             if (!(Type & MEMORY_AREA_STATIC)) ExFreePoolWithTag(MemoryArea, TAG_MAREA);
//             return STATUS_NO_MEMORY;
//         }
//
//         MemoryArea->VadNode.StartingVpn = (ULONG_PTR)*BaseAddress >> PAGE_SHIFT;
//         MemoryArea->VadNode.EndingVpn = ((ULONG_PTR)*BaseAddress + tmpLength - 1) >> PAGE_SHIFT;
//         MmInsertMemoryArea(AddressSpace, MemoryArea, Protect);
//     }
//     else
//     {
//         EndingAddress = ((ULONG_PTR)*BaseAddress + Length - 1) | (PAGE_SIZE - 1);
//         *BaseAddress = ALIGN_DOWN_POINTER_BY(*BaseAddress, Granularity);
//         tmpLength = EndingAddress + 1 - (ULONG_PTR)*BaseAddress;
//
//         if (!MmGetAddressSpaceOwner(AddressSpace) && *BaseAddress < MmSystemRangeStart)
//         {
//             ASSERT(FALSE);
//             if (!(Type & MEMORY_AREA_STATIC)) ExFreePoolWithTag(MemoryArea, TAG_MAREA);
//             return STATUS_ACCESS_VIOLATION;
//         }
//
//         if (MmGetAddressSpaceOwner(AddressSpace) &&
//                 (ULONG_PTR)(*BaseAddress) + tmpLength > (ULONG_PTR)MmSystemRangeStart)
//         {
//             DPRINT("Memory area for user mode address space exceeds MmSystemRangeStart\n");
//             if (!(Type & MEMORY_AREA_STATIC)) ExFreePoolWithTag(MemoryArea, TAG_MAREA);
//             return STATUS_ACCESS_VIOLATION;
//         }
//
//         /* No need to check ARM3 owned memory areas, the range MUST be free */
//         if (MemoryArea->Type != MEMORY_AREA_OWNED_BY_ARM3)
//         {
//             if (!MmIsAddressRangeFree(AddressSpace, *BaseAddress, tmpLength))
//             {
//                 DPRINT("Memory area already occupied\n");
//                 if (!(Type & MEMORY_AREA_STATIC)) ExFreePoolWithTag(MemoryArea, TAG_MAREA);
//                 return STATUS_CONFLICTING_ADDRESSES;
//             }
//         }
//
//         MemoryArea->VadNode.StartingVpn = (ULONG_PTR)*BaseAddress >> PAGE_SHIFT;
//         MemoryArea->VadNode.EndingVpn = ((ULONG_PTR)*BaseAddress + tmpLength - 1) >> PAGE_SHIFT;
//         MmInsertMemoryArea(AddressSpace, MemoryArea, Protect);
//     }
//
//     *Result = MemoryArea;
//
//     DPRINT("MmCreateMemoryArea() succeeded (%p)\n", *BaseAddress);
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiRosCleanupMemoryArea(
//     PEPROCESS Process,
//     PMMVAD Vad)
// {
//     PMEMORY_AREA MemoryArea;
//     PVOID BaseAddress;
//     NTSTATUS Status;
//
//     /* We must be called from MmCleanupAddressSpace and nowhere else!
//        Make sure things are as expected... */
//     ASSERT(Process == PsGetCurrentProcess());
//     ASSERT(Process->VmDeleted == TRUE);
//     ASSERT(((PsGetCurrentThread()->ThreadsProcess == Process) &&
//             (Process->ActiveThreads == 1)) ||
//            (Process->ActiveThreads == 0));
//
//     MemoryArea = (PMEMORY_AREA)Vad;
//     BaseAddress = (PVOID)MA_GetStartingAddress(MemoryArea);
//
//     if (MemoryArea->Type == MEMORY_AREA_SECTION_VIEW)
//     {
//         Status = MiRosUnmapViewOfSection(Process, MemoryArea, BaseAddress, Process->ProcessExiting);
//     }
// #ifdef NEWCC
//     else if (MemoryArea->Type == MEMORY_AREA_CACHE)
//     {
//         Status = MmUnmapViewOfCacheSegment(&Process->Vm, BaseAddress);
//     }
// #endif
//     else
//     {
//         /* There shouldn't be anything else! */
//         ASSERT(FALSE);
//     }
//
//     /* Make sure this worked! */
//     ASSERT(NT_SUCCESS(Status));
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
