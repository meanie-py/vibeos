use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    _HARDWARE_PTE_X86 as HARDWARE_PTE, PHARDWARE_PTE_X86 as PHARDWARE_PTE, PHYSICAL_ADDRESS,
    KeDisableInterrupts, KeRestoreInterrupts,
    MmAllocateContiguousMemory, MmFreeContiguousMemory, MmGetPhysicalAddress,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const PDE_BITS: usize = 10;
const PTE_BITS: usize = 10;

#[repr(C)]
pub struct LARGE_IDENTITY_MAP {
    pub TopLevelDirectory: PHARDWARE_PTE,
    pub Cr3: ULONG,
    pub StartAddress: ULONG_PTR,
    pub PagesCount: ULONG,
    pub PagesList: [PVOID; 30],
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn Ki386EnableGlobalPage(Context: ULONG_PTR) -> ULONG_PTR {
    let Enable: BOOLEAN;

    Enable = KeDisableInterrupts();

    #[cfg(all(feature = "global_pages_are_awesome", target_arch = "x86"))]
    {
        use core::arch::x86::*;
        let Cr4: u32;
        const CR4_PGE: u32 = 0x80;

        Cr4 = __readcr4();
        ntkrnl_sys::ASSERT!((Cr4 & CR4_PGE) == 0);

        __writecr3(__readcr3());

        __writecr4(Cr4 | CR4_PGE);
    }

    #[cfg(all(feature = "global_pages_are_awesome", target_arch = "x86_64"))]
    {
        use core::arch::x86_64::*;
        let Cr4: u64;
        const CR4_PGE: u64 = 0x80;

        Cr4 = __readcr4();
        ntkrnl_sys::ASSERT!((Cr4 & CR4_PGE) == 0);

        __writecr3(__readcr3());

        __writecr4(Cr4 | CR4_PGE);
    }

    KeRestoreInterrupts(Enable);
    0
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializePAT() {
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn Ki386EnableTargetLargePage(Context: ULONG_PTR) -> ULONG_PTR {
    let IdentityMap = Context as *mut LARGE_IDENTITY_MAP;

    Ki386EnableCurrentLargePage((*IdentityMap).StartAddress, (*IdentityMap).Cr3);

    0
}

unsafe fn Ki386AllocateContiguousMemory(
    IdentityMap: *mut LARGE_IDENTITY_MAP,
    PagesCount: ULONG,
    InLower4Gb: BOOLEAN,
) -> PVOID {
    let mut AddressMask: PHYSICAL_ADDRESS;
    let Result: PVOID;
    let SizeInBytes = PagesCount * PAGE_SIZE as ULONG;

    AddressMask = PHYSICAL_ADDRESS { QuadPart: 0 };
    AddressMask.QuadPart = 0xFFFFFFFFFFFFFFFF_u64 as i64;

    if InLower4Gb != 0 {
        AddressMask.QuadPart = 0xFFFFFFFF_u64 as i64;
    }

    Result = MmAllocateContiguousMemory(SizeInBytes, AddressMask);
    if Result.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(Result, SizeInBytes as usize);

    (*IdentityMap).PagesList[(*IdentityMap).PagesCount as usize] = Result;
    (*IdentityMap).PagesCount += 1;

    Result
}

#[no_mangle]
pub unsafe extern "system" fn Ki386BuildIdentityBuffer(
    IdentityMap: *mut LARGE_IDENTITY_MAP,
    StartPtr: PVOID,
    Length: ULONG,
) -> PHYSICAL_ADDRESS {
    let IdentityMap = opaque_ptr(IdentityMap);
    let StartPtr = opaque_ptr(StartPtr);
    MmGetPhysicalAddress(StartPtr)
}

#[no_mangle]
pub unsafe extern "system" fn Ki386IdentityMapMakeValid(
    IdentityMap: *mut LARGE_IDENTITY_MAP,
    Pde: PHARDWARE_PTE,
    PageTable: *mut PHARDWARE_PTE,
) -> BOOLEAN {
    let IdentityMap = opaque_ptr(IdentityMap);
    let PageTable = opaque_ptr(PageTable);
    let NewPage: ULONG;

    if (*Pde).Valid() == 0 {
        NewPage = Ki386AllocateContiguousMemory(IdentityMap, 1, 0u8) as ULONG;
        if NewPage == 0 {
            return 0u8;
        }

        (*Pde).set_PageFrameNumber((NewPage >> PAGE_SHIFT) as _);
        (*Pde).set_Valid(1);

        if !PageTable.is_null() {
            *PageTable = NewPage as PHARDWARE_PTE;
        }
    } else {
        if !PageTable.is_null() {
            *PageTable = ((*Pde).PageFrameNumber() << PAGE_SHIFT) as PHARDWARE_PTE;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn Ki386MapAddress(
    IdentityMap: *mut LARGE_IDENTITY_MAP,
    VirtualPtr: ULONG_PTR,
    PhysicalPtr: PHYSICAL_ADDRESS,
) -> BOOLEAN {
    let IdentityMap = opaque_ptr(IdentityMap);
    let Pde: PHARDWARE_PTE;
    let Pte: PHARDWARE_PTE;
    let mut PageTable: PHARDWARE_PTE = core::ptr::null_mut();

    if (*IdentityMap).TopLevelDirectory.is_null() {
        (*IdentityMap).TopLevelDirectory = Ki386AllocateContiguousMemory(IdentityMap, 1, 1) as PHARDWARE_PTE;
        if (*IdentityMap).TopLevelDirectory.is_null() {
            return 0u8;
        }
    }

    Pde = ((*IdentityMap).TopLevelDirectory as *mut HARDWARE_PTE)
        .add((VirtualPtr >> 22) & ((1 << PDE_BITS) - 1));
    if Ki386IdentityMapMakeValid(IdentityMap, Pde, &mut PageTable) == 0 {
        return 0u8;
    }
    (*Pde).set_Write(1);

    Pte = PageTable.add((VirtualPtr >> 12) & ((1 << PTE_BITS) - 1));
    (*Pte).set_Valid(1);
    (*Pte).set_PageFrameNumber((PhysicalPtr.QuadPart as u64 >> PAGE_SHIFT) as _);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn Ki386ConvertPte(Pte: PHARDWARE_PTE) {
    let VirtualPtr: PVOID;
    let PhysicalPtr: PHYSICAL_ADDRESS;

    VirtualPtr = ((*Pte).PageFrameNumber() << PAGE_SHIFT) as PVOID;
    PhysicalPtr = MmGetPhysicalAddress(VirtualPtr);

    (*Pte).set_PageFrameNumber((PhysicalPtr.QuadPart as u64 >> PAGE_SHIFT) as _);
}

#[no_mangle]
pub unsafe extern "system" fn Ki386CreateIdentityMap(
    IdentityMap: *mut LARGE_IDENTITY_MAP,
    StartPtr: PVOID,
    PagesCount: ULONG,
) -> BOOLEAN {
    let IdentityMap = opaque_ptr(IdentityMap);
    let StartPtr = opaque_ptr(StartPtr);
    let mut Ptr: ULONG_PTR;
    let mut PteIndex: ULONG;
    let mut IdentityPtr: PHYSICAL_ADDRESS;

    RtlZeroMemory(
        IdentityMap as PVOID,
        core::mem::size_of::<LARGE_IDENTITY_MAP>(),
    );

    IdentityPtr = Ki386BuildIdentityBuffer(IdentityMap, StartPtr, PagesCount);
    (*IdentityMap).StartAddress = IdentityPtr.QuadPart as ULONG_PTR;
    if (*IdentityMap).StartAddress == 0 {
        return 0u8;
    }

    Ptr = StartPtr as ULONG_PTR;
    while Ptr < (StartPtr as ULONG_PTR) + (PagesCount as usize * PAGE_SIZE) {
        if Ki386MapAddress(IdentityMap, Ptr, IdentityPtr) == 0 {
            return 0u8;
        }

        if Ki386MapAddress(IdentityMap, IdentityPtr.QuadPart as ULONG_PTR, IdentityPtr) == 0 {
            return 0u8;
        }

        Ptr += PAGE_SIZE as ULONG_PTR;
        IdentityPtr.QuadPart += PAGE_SIZE as i64;
    }

    PteIndex = 0;
    while PteIndex < (PAGE_SIZE / core::mem::size_of::<HARDWARE_PTE>()) as ULONG {
        if (*((*IdentityMap).TopLevelDirectory as *mut HARDWARE_PTE).add(PteIndex as usize))
            .Valid() != 0
        {
            Ki386ConvertPte(
                ((*IdentityMap).TopLevelDirectory as *mut HARDWARE_PTE).add(PteIndex as usize),
            );
        }
        PteIndex += 1;
    }

    (*IdentityMap).Cr3 = MmGetPhysicalAddress((*IdentityMap).TopLevelDirectory as PVOID).QuadPart as ULONG;

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn Ki386FreeIdentityMap(IdentityMap: *mut LARGE_IDENTITY_MAP) {
    let IdentityMap = opaque_ptr(IdentityMap);
    let mut Page: ULONG;

    Page = 0;
    while Page < (*IdentityMap).PagesCount {
        MmFreeContiguousMemory((*IdentityMap).PagesList[Page as usize]);
        Page += 1;
    }
}

extern "system" {
    fn Ki386EnableCurrentLargePage(StartAddress: ULONG_PTR, Cr3: ULONG);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ke/i386/patpge.c
// * PURPOSE:         Support for PAT and PGE (Large Pages)
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define PDE_BITS 10
// #define PTE_BITS 10
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// Ki386EnableGlobalPage(IN ULONG_PTR Context)
// {
//     //PLONG Count;
// #if defined(_GLOBAL_PAGES_ARE_AWESOME_)
//     ULONG Cr4;
// #endif
//     BOOLEAN Enable;
//
//     /* Disable interrupts */
//     Enable = KeDisableInterrupts();
//
//     /* Spin until other processors are ready */
//     //Count = (PLONG)Context;
//     //InterlockedDecrement(Count);
//     //while (*Count) YieldProcessor();
//
// #if defined(_GLOBAL_PAGES_ARE_AWESOME_)
//
//     /* Get CR4 and ensure global pages are disabled */
//     Cr4 = __readcr4();
//     ASSERT(!(Cr4 & CR4_PGE));
//
//     /* Reset CR3 to flush the TLB */
//     __writecr3(__readcr3());
//
//     /* Now enable PGE */
//     __writecr4(Cr4 | CR4_PGE);
//
// #endif
//
//     /* Restore interrupts and return */
//     KeRestoreInterrupts(Enable);
//     return 0;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializePAT(VOID)
// {
//     /* FIXME: Support this */
//     DPRINT("PAT support detected but not yet taken advantage of\n");
// }
//
// CODE_SEG("INIT")
// ULONG_PTR
// NTAPI
// Ki386EnableTargetLargePage(IN ULONG_PTR Context)
// {
//     PLARGE_IDENTITY_MAP IdentityMap = (PLARGE_IDENTITY_MAP)Context;
//
//     /* Call helper function with the start address and temporary page table pointer */
//     Ki386EnableCurrentLargePage(IdentityMap->StartAddress, IdentityMap->Cr3);
//
//     return 0;
// }
//
// PVOID
// NTAPI
// Ki386AllocateContiguousMemory(PLARGE_IDENTITY_MAP IdentityMap,
//                               ULONG PagesCount,
//                               BOOLEAN InLower4Gb)
// {
//     PHYSICAL_ADDRESS AddressMask;
//     PVOID Result;
//     ULONG SizeInBytes = PagesCount * PAGE_SIZE;
//
//     /* Initialize acceptable address mask to any possible address */
//     AddressMask.LowPart = 0xFFFFFFFF;
//     AddressMask.HighPart = 0xFFFFFFFF;
//
//     /* Mark out higher 4Gb if caller asked so */
//     if (InLower4Gb) AddressMask.HighPart = 0;
//
//     /* Try to allocate the memory */
//     Result = MmAllocateContiguousMemory(SizeInBytes, AddressMask);
//     if (!Result) return NULL;
//
//     /* Zero it out */
//     RtlZeroMemory(Result, SizeInBytes);
//
//     /* Track allocated pages in the IdentityMap structure */
//     IdentityMap->PagesList[IdentityMap->PagesCount] = Result;
//     IdentityMap->PagesCount++;
//
//     return Result;
// }
//
// PHYSICAL_ADDRESS
// NTAPI
// Ki386BuildIdentityBuffer(PLARGE_IDENTITY_MAP IdentityMap,
//                          PVOID StartPtr,
//                          ULONG Length)
// {
//     // TODO: Check whether all pages are below 4GB boundary
//     return MmGetPhysicalAddress(StartPtr);
// }
//
// BOOLEAN
// NTAPI
// Ki386IdentityMapMakeValid(PLARGE_IDENTITY_MAP IdentityMap,
//                           PHARDWARE_PTE Pde,
//                           PHARDWARE_PTE *PageTable)
// {
//     ULONG NewPage;
//
//     if (Pde->Valid == 0)
//     {
//         /* Invalid, so allocate a new page for it */
//         NewPage = (ULONG)Ki386AllocateContiguousMemory(IdentityMap, 1, FALSE);
//         if (!NewPage) return FALSE;
//
//         /* Set PFN to its virtual address and mark it as valid */
//         Pde->PageFrameNumber = NewPage >> PAGE_SHIFT;
//         Pde->Valid = 1;
//
//         /* Pass page table address to the caller */
//         if (PageTable) *PageTable = (PHARDWARE_PTE)NewPage;
//     }
//     else
//     {
//         /* Valid entry, just pass the page table address to the caller */
//         if (PageTable)
//             *PageTable = (PHARDWARE_PTE)(Pde->PageFrameNumber << PAGE_SHIFT);
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// Ki386MapAddress(PLARGE_IDENTITY_MAP IdentityMap,
//                 ULONG_PTR VirtualPtr,
//                 PHYSICAL_ADDRESS PhysicalPtr)
// {
//     PHARDWARE_PTE Pde, Pte;
//     PHARDWARE_PTE PageTable;
//
//     /* Allocate page directory on demand */
//     if (!IdentityMap->TopLevelDirectory)
//     {
//         IdentityMap->TopLevelDirectory = Ki386AllocateContiguousMemory(IdentityMap, 1, 1);
//         if (!IdentityMap->TopLevelDirectory) return FALSE;
//     }
//
//     /* Get PDE of VirtualPtr and make it writable and valid */
//     Pde = &IdentityMap->TopLevelDirectory[(VirtualPtr >> 22) & ((1 << PDE_BITS) - 1)];
//     if (!Ki386IdentityMapMakeValid(IdentityMap, Pde, &PageTable)) return FALSE;
//     Pde->Write = 1;
//
//     /* Get PTE of VirtualPtr, make it valid, and map PhysicalPtr there */
//     Pte = &PageTable[(VirtualPtr >> 12) & ((1 << PTE_BITS) - 1)];
//     Pte->Valid = 1;
//     Pte->PageFrameNumber = (PFN_NUMBER)(PhysicalPtr.QuadPart >> PAGE_SHIFT);
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// Ki386ConvertPte(PHARDWARE_PTE Pte)
// {
//     PVOID VirtualPtr;
//     PHYSICAL_ADDRESS PhysicalPtr;
//
//     /* Get virtual and physical addresses */
//     VirtualPtr = (PVOID)(Pte->PageFrameNumber << PAGE_SHIFT);
//     PhysicalPtr = MmGetPhysicalAddress(VirtualPtr);
//
//     /* Map its physical address in the page table provided by the caller */
//     Pte->PageFrameNumber = (PFN_NUMBER)(PhysicalPtr.QuadPart >> PAGE_SHIFT);
// }
//
// BOOLEAN
// NTAPI
// Ki386CreateIdentityMap(PLARGE_IDENTITY_MAP IdentityMap, PVOID StartPtr, ULONG PagesCount)
// {
//     ULONG_PTR Ptr;
//     ULONG PteIndex;
//     PHYSICAL_ADDRESS IdentityPtr;
//
//     /* Zero out the IdentityMap contents */
//     RtlZeroMemory(IdentityMap, sizeof(LARGE_IDENTITY_MAP));
//
//     /* Get the pointer to the physical address and save it in the struct */
//     IdentityPtr = Ki386BuildIdentityBuffer(IdentityMap, StartPtr, PagesCount);
//     IdentityMap->StartAddress = IdentityPtr.LowPart;
//     if (IdentityMap->StartAddress == 0)
//     {
//         DPRINT1("Failed to get buffer for large pages identity mapping\n");
//         return FALSE;
//     }
//     DPRINT("IdentityMap->StartAddress %p\n", IdentityMap->StartAddress);
//
//     /* Map all pages */
//     for (Ptr = (ULONG_PTR)StartPtr;
//          Ptr < (ULONG_PTR)StartPtr + PagesCount * PAGE_SIZE;
//          Ptr += PAGE_SIZE, IdentityPtr.QuadPart += PAGE_SIZE)
//     {
//         /* Map virtual address */
//         if (!Ki386MapAddress(IdentityMap, Ptr, IdentityPtr)) return FALSE;
//
//         /* Map physical address */
//         if (!Ki386MapAddress(IdentityMap, IdentityPtr.LowPart, IdentityPtr)) return FALSE;
//     }
//
//     /* Convert all PTEs in the page directory from virtual to physical,
//        because Ki386IdentityMapMakeValid mapped only virtual addresses */
//     for (PteIndex = 0; PteIndex < (PAGE_SIZE / sizeof(HARDWARE_PTE)); PteIndex++)
//     {
//         if (IdentityMap->TopLevelDirectory[PteIndex].Valid != 0)
//             Ki386ConvertPte(&IdentityMap->TopLevelDirectory[PteIndex]);
//     }
//
//     /* Save the page directory address (allocated by Ki386MapAddress) */
//     IdentityMap->Cr3 = MmGetPhysicalAddress(IdentityMap->TopLevelDirectory).LowPart;
//
//     DPRINT("IdentityMap->Cr3 0x%x\n", IdentityMap->Cr3);
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// Ki386FreeIdentityMap(PLARGE_IDENTITY_MAP IdentityMap)
// {
//     ULONG Page;
//
//     DPRINT("Freeing %lu pages allocated for identity mapping\n", IdentityMap->PagesCount);
//
//     /* Free all allocated pages, if any */
//     for (Page = 0; Page < IdentityMap->PagesCount; Page++)
//         MmFreeContiguousMemory(IdentityMap->PagesList[Page]);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
