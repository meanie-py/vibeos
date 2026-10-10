use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;

extern "system" {
    fn MiDecrementShareCount(Pfn: PMMPFN, PageFrameIndex: PFN_NUMBER);
}

use ntkrnl_sys::ddk::{
    PHYSICAL_ADDRESS,
    MEMORY_CACHING_TYPE,
    KeEnterGuardedRegion, KeLeaveGuardedRegion,
    MmMapIoSpace, MmUnmapIoSpace,
    MiIsPfnInUse,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

unsafe fn MiFindContiguousPages(
    LowestPfn: PFN_NUMBER,
    HighestPfn: PFN_NUMBER,
    BoundaryPfn: PFN_NUMBER,
    SizeInPages: PFN_NUMBER,
    CacheType: MEMORY_CACHING_TYPE,
) -> PFN_NUMBER {
    let mut Page: PFN_NUMBER;
    let mut PageCount: PFN_NUMBER;
    let mut LastPage: PFN_NUMBER;
    let mut Length: PFN_NUMBER;
    let BoundaryMask: PFN_NUMBER;
    let mut i: ULONG = 0;
    let mut Pfn1: PMMPFN;
    let EndPfn: PMMPFN;
    let OldIrql: KIRQL;

    BoundaryMask = !(BoundaryPfn - 1);

    KeEnterGuardedRegion();

    loop {
        Page = (*(*MmPhysicalMemoryBlock).Run.as_ptr().add(i as usize)).BasePage;
        PageCount = (*(*MmPhysicalMemoryBlock).Run.as_ptr().add(i as usize)).PageCount;

        LastPage = Page + PageCount;

        if (LastPage - 1) > HighestPfn {
            LastPage = HighestPfn + 1;
        }
        if Page < LowestPfn {
            Page = LowestPfn;
        }

        if PageCount == 0 || (Page + SizeInPages) > LastPage {
            i += 1;
            if i == (*MmPhysicalMemoryBlock).NumberOfRuns {
                break;
            }
            continue;
        }

        Length = 0;
        Pfn1 = MiGetPfnEntry(Page);
        while Page < LastPage {
            if MiIsPfnInUse(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN) != 0 {
                Length = 0;
                Pfn1 = Pfn1.add(1);
                Page += 1;
                continue;
            }

            if Length == 0 && BoundaryPfn != 0 && ((Page ^ (Page + SizeInPages - 1)) & BoundaryMask) != 0 {
                Pfn1 = Pfn1.add(1);
                Page += 1;
                continue;
            }

            Length += 1;
            if Length == SizeInPages {
                Pfn1 = Pfn1.sub((Length - 1) as usize);
                Page -= Length - 1;

                let OldIrql = MiAcquirePfnLock();
                loop {
                    if MiIsPfnInUse(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN) != 0 {
                        break;
                    }

                    Length -= 1;
                    if Length == 0 {
                        let mut EndPfn = Pfn1.sub((SizeInPages - 1) as usize);
                        loop {
                            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_CONTINOUS_ALLOCATION);
                            ntkrnl_sys::MI_SET_PROCESS2!("Kernel Driver");
                            MiUnlinkFreeOrZeroedPage(Pfn1);
                            (*Pfn1).set_reference_count(1);
                            (*Pfn1).set_share_count(1 as ULONG_PTR);
                            (*Pfn1).set_page_location(ActiveAndValid);
                            (*Pfn1).set_start_of_allocation(false);
                            (*Pfn1).set_end_of_allocation(false);
                            (*Pfn1).set_prototype_pte(false);
                            (*Pfn1).set_verifier_allocation(false);
                            (*Pfn1).set_pte_address(0xBAADF00DBAADF00Du64 as usize as PMMPTE);

                            if Pfn1 == EndPfn {
                                break;
                            }
                            Pfn1 = Pfn1.sub(1);
                        }

                        (*Pfn1).set_start_of_allocation(true);
                        (*Pfn1.add((SizeInPages - 1) as usize)).set_end_of_allocation(true);

                        MiReleasePfnLock(OldIrql);

                        let EndPfn = Pfn1.add(SizeInPages as usize);

                        Page = Page - SizeInPages + 1;

                        KeLeaveGuardedRegion();
                        return Page;
                    }

                    Pfn1 = Pfn1.add(1);
                    Page += 1;
                }

                MiReleasePfnLock(OldIrql);
                Length = 0;
            }

            Pfn1 = Pfn1.add(1);
            Page += 1;
        }

        i += 1;
        if i == (*MmPhysicalMemoryBlock).NumberOfRuns {
            break;
        }
    }

    KeLeaveGuardedRegion();
    return 0;
}

unsafe fn MiCheckForContiguousMemory(
    BaseAddress: PVOID,
    BaseAddressPages: PFN_NUMBER,
    SizeInPages: PFN_NUMBER,
    LowestPfn: PFN_NUMBER,
    HighestPfn: PFN_NUMBER,
    BoundaryPfn: PFN_NUMBER,
    CacheAttribute: MI_PFN_CACHE_ATTRIBUTE,
) -> PVOID {
    let mut StartPte: PMMPTE;
    let EndPte: PMMPTE;
    let mut PreviousPage: PFN_NUMBER = 0;
    let mut Page: PFN_NUMBER;
    let HighPage: PFN_NUMBER;
    let BoundaryMask: PFN_NUMBER;
    let mut Pages: PFN_NUMBER = 0;

    if LowestPfn > HighestPfn {
        return core::ptr::null_mut();
    }
    if LowestPfn + SizeInPages <= LowestPfn {
        return core::ptr::null_mut();
    }
    if LowestPfn + SizeInPages - 1 > HighestPfn {
        return core::ptr::null_mut();
    }
    if BaseAddressPages < SizeInPages {
        return core::ptr::null_mut();
    }

    HighPage = HighestPfn + 1 - SizeInPages;
    BoundaryMask = !(BoundaryPfn - 1);

    StartPte = MiAddressToPte(BaseAddress);
    let EndPte = StartPte.add(BaseAddressPages as usize);
    while StartPte < EndPte {
        Page = PFN_FROM_PTE(StartPte);

        if Pages == 0 {
            if Page >= LowestPfn && Page <= HighPage {
                if BoundaryPfn == 0 || ((Page ^ (Page + SizeInPages - 1)) & BoundaryMask) == 0 {
                    Pages += 1;
                }
            }

            if Pages == SizeInPages {
                return MiPteToAddress(StartPte);
            }
        } else {
            if Page != (PreviousPage + 1) {
                Pages = 0;
                PreviousPage = Page;
                StartPte = StartPte.add(1);
                continue;
            }

            Pages += 1;
            if Pages == SizeInPages {
                return MiPteToAddress(StartPte.sub((Pages - 1) as usize));
            }
        }

        PreviousPage = Page;
        StartPte = StartPte.add(1);
    }

    return core::ptr::null_mut();
}

unsafe fn MiFindContiguousMemory(
    LowestPfn: PFN_NUMBER,
    HighestPfn: PFN_NUMBER,
    BoundaryPfn: PFN_NUMBER,
    SizeInPages: PFN_NUMBER,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    let Page: PFN_NUMBER;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = unsafe { core::mem::zeroed() };
    let mut Pfn1: PMMPFN;
    let EndPfn: PMMPFN;
    let mut PointerPte: PMMPTE;
    let BaseAddress: PVOID;

    Page = MiFindContiguousPages(LowestPfn, HighestPfn, BoundaryPfn, SizeInPages, CacheType);
    if Page == 0 {
        return core::ptr::null_mut();
    }

    PhysicalAddress.QuadPart = (Page << PAGE_SHIFT) as i64;
    let BaseAddress = MmMapIoSpace(PhysicalAddress, (SizeInPages << PAGE_SHIFT) as ntkrnl_sys::ddk::SIZE_T, CacheType);

    Pfn1 = MiGetPfnEntry(Page);
    let EndPfn = Pfn1.add(SizeInPages as usize);
    PointerPte = MiAddressToPte(BaseAddress);
    loop {
        (*Pfn1).set_pte_address(PointerPte as PVOID as PMMPTE);
        (*Pfn1).set_pte_frame(PFN_FROM_PTE(MiAddressToPte(PointerPte as PVOID)));
        PointerPte = PointerPte.add(1);
        Pfn1 = Pfn1.add(1);
        if Pfn1 >= EndPfn {
            break;
        }
    }

    return BaseAddress;
}

unsafe fn MiAllocateContiguousMemory(
    NumberOfBytes: SIZE_T,
    LowestAcceptablePfn: PFN_NUMBER,
    HighestAcceptablePfn: PFN_NUMBER,
    BoundaryPfn: PFN_NUMBER,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    let mut BaseAddress: PVOID;
    let SizeInPages: PFN_NUMBER;
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;

    SizeInPages = BYTES_TO_PAGES(NumberOfBytes);

    CacheAttribute = MiPlatformCacheAttributes[0][CacheType as usize];
    if CacheAttribute == MiCached_u32 {
        BaseAddress = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, NumberOfBytes, u32::from_ne_bytes(*b"mCmM"));
        if !BaseAddress.is_null() {
            if !MiCheckForContiguousMemory(
                BaseAddress,
                SizeInPages,
                SizeInPages,
                LowestAcceptablePfn,
                HighestAcceptablePfn,
                BoundaryPfn,
                CacheAttribute,
            )
            .is_null()
            {
                return BaseAddress;
            }

            ExFreePoolWithTag(BaseAddress, u32::from_ne_bytes(*b"mCmM"));
        }
    }

    if KeGetCurrentIrql() > APC_LEVEL as u8 {
        return core::ptr::null_mut();
    }

    BaseAddress = MiFindContiguousMemory(
        LowestAcceptablePfn,
        HighestAcceptablePfn,
        BoundaryPfn,
        SizeInPages,
        CacheType,
    );
    return BaseAddress;
}

unsafe fn MiFreeContiguousMemory(BaseAddress: PVOID) {
    let OldIrql: KIRQL;
    let mut PageFrameIndex: PFN_NUMBER;
    let LastPage: PFN_NUMBER;
    let PageCount: PFN_NUMBER;
    let mut Pfn1: PMMPFN;
    let StartPfn: PMMPFN;
    let mut PointerPte: PMMPTE;

    if (BaseAddress >= MmNonPagedPoolStart
        && BaseAddress < (MmNonPagedPoolStart as ULONG_PTR + MmSizeOfNonPagedPoolInBytes) as PVOID)
        || (BaseAddress >= MmNonPagedPoolExpansionStart && BaseAddress < MmNonPagedPoolEnd)
    {
        ExFreePoolWithTag(BaseAddress, u32::from_ne_bytes(*b"mCmM"));
        return;
    }

    PointerPte = MiAddressToPte(BaseAddress);
    PageFrameIndex = PFN_FROM_PTE(PointerPte);

    Pfn1 = MiGetPfnEntry(PageFrameIndex);
    if Pfn1.is_null() || !(*Pfn1).start_of_allocation() {
        KeBugCheckEx(BAD_POOL_CALLER, 0x60, BaseAddress as ULONG_PTR, 0, 0);
    }

    let StartPfn = Pfn1;
    (*Pfn1).set_start_of_allocation(false);

    loop {
        MI_SET_PFN_DELETED(Pfn1);
        PointerPte = PointerPte.add(1);
        if (*Pfn1).end_of_allocation() {
            break;
        }
        Pfn1 = Pfn1.add(1);
    }

    (*Pfn1).set_end_of_allocation(false);

    let PageCount = (Pfn1 as usize - StartPfn as usize) / core::mem::size_of::<MMPFN>() + 1;

    MmUnmapIoSpace(BaseAddress, (PageCount << PAGE_SHIFT) as ntkrnl_sys::ddk::SIZE_T);

    let OldIrql = MiAcquirePfnLock();

    let LastPage = PageFrameIndex + PageCount as PFN_NUMBER;
    Pfn1 = MiGetPfnEntry(PageFrameIndex);
    loop {
        MiDecrementShareCount(Pfn1, PageFrameIndex);
        Pfn1 = Pfn1.add(1);
        PageFrameIndex += 1;
        if PageFrameIndex >= LastPage {
            break;
        }
    }

    MiReleasePfnLock(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocateContiguousMemorySpecifyCache(
    NumberOfBytes: SIZE_T,
    LowestAcceptableAddress: PHYSICAL_ADDRESS,
    HighestAcceptableAddress: PHYSICAL_ADDRESS,
    BoundaryAddressMultiple: PHYSICAL_ADDRESS,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    let mut LowestPfn: PFN_NUMBER;
    let mut HighestPfn: PFN_NUMBER;
    let BoundaryPfn: PFN_NUMBER;

    LowestPfn = (LowestAcceptableAddress.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    if (LowestAcceptableAddress.QuadPart as u32 & (PAGE_SIZE as u32 - 1)) != 0 {
        LowestPfn += 1;
    }

    if (BoundaryAddressMultiple.QuadPart as u32 & (PAGE_SIZE as u32 - 1)) != 0 {
        return core::ptr::null_mut();
    }
    BoundaryPfn = (BoundaryAddressMultiple.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;

    HighestPfn = (HighestAcceptableAddress.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    if HighestPfn > MmHighestPhysicalPage {
        HighestPfn = MmHighestPhysicalPage;
    }

    if LowestPfn > HighestPfn {
        return core::ptr::null_mut();
    }

    return MiAllocateContiguousMemory(NumberOfBytes, LowestPfn, HighestPfn, BoundaryPfn, CacheType);
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocateContiguousMemory(
    NumberOfBytes: SIZE_T,
    HighestAcceptableAddress: PHYSICAL_ADDRESS,
) -> PVOID {
    let mut HighestPfn: PFN_NUMBER;

    HighestPfn = (HighestAcceptableAddress.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    if HighestPfn > MmHighestPhysicalPage {
        HighestPfn = MmHighestPhysicalPage;
    }

    return MiAllocateContiguousMemory(NumberOfBytes, 0, HighestPfn, 0, MmCached_val);
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeContiguousMemory(BaseAddress: PVOID) {
    let BaseAddress = opaque_ptr(BaseAddress);
    MiFreeContiguousMemory(BaseAddress);
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeContiguousMemorySpecifyCache(
    BaseAddress: PVOID,
    NumberOfBytes: SIZE_T,
    CacheType: MEMORY_CACHING_TYPE,
) {
    let BaseAddress = opaque_ptr(BaseAddress);
    MiFreeContiguousMemory(BaseAddress);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/contmem.c
//  * PURPOSE:         ARM Memory Manager Contiguous Memory Allocator
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// PFN_NUMBER
// NTAPI
// MiFindContiguousPages(IN PFN_NUMBER LowestPfn,
//                       IN PFN_NUMBER HighestPfn,
//                       IN PFN_NUMBER BoundaryPfn,
//                       IN PFN_NUMBER SizeInPages,
//                       IN MEMORY_CACHING_TYPE CacheType)
// {
//     PFN_NUMBER Page, PageCount, LastPage, Length, BoundaryMask;
//     ULONG i = 0;
//     PMMPFN Pfn1, EndPfn;
//     KIRQL OldIrql;
//     PAGED_CODE();
//     ASSERT(SizeInPages != 0);
//
//     //
//     // Convert the boundary PFN into an alignment mask
//     //
//     BoundaryMask = ~(BoundaryPfn - 1);
//
//     /* Disable APCs */
//     KeEnterGuardedRegion();
//
//     //
//     // Loop all the physical memory blocks
//     //
//     do
//     {
//         //
//         // Capture the base page and length of this memory block
//         //
//         Page = MmPhysicalMemoryBlock->Run[i].BasePage;
//         PageCount = MmPhysicalMemoryBlock->Run[i].PageCount;
//
//         //
//         // Check how far this memory block will go
//         //
//         LastPage = Page + PageCount;
//
//         //
//         // Trim it down to only the PFNs we're actually interested in
//         //
//         if ((LastPage - 1) > HighestPfn) LastPage = HighestPfn + 1;
//         if (Page < LowestPfn) Page = LowestPfn;
//
//         //
//         // Skip this run if it's empty or fails to contain all the pages we need
//         //
//         if (!(PageCount) || ((Page + SizeInPages) > LastPage)) continue;
//
//         //
//         // Now scan all the relevant PFNs in this run
//         //
//         Length = 0;
//         for (Pfn1 = MI_PFN_ELEMENT(Page); Page < LastPage; Page++, Pfn1++)
//         {
//             //
//             // If this PFN is in use, ignore it
//             //
//             if (MiIsPfnInUse(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN))
//             {
//                 Length = 0;
//                 continue;
//             }
//
//             //
//             // If we haven't chosen a start PFN yet and the caller specified an
//             // alignment, make sure the page matches the alignment restriction
//             //
//             if ((!(Length) && (BoundaryPfn)) &&
//                 (((Page ^ (Page + SizeInPages - 1)) & BoundaryMask)))
//             {
//                 //
//                 // It does not, so bail out
//                 //
//                 continue;
//             }
//
//             //
//             // Increase the number of valid pages, and check if we have enough
//             //
//             if (++Length == SizeInPages)
//             {
//                 //
//                 // It appears we've amassed enough legitimate pages, rollback
//                 //
//                 Pfn1 -= (Length - 1);
//                 Page -= (Length - 1);
//
//                 //
//                 // Acquire the PFN lock
//                 //
//                 OldIrql = MiAcquirePfnLock();
//                 do
//                 {
//                     //
//                     // Things might've changed for us. Is the page still free?
//                     //
//                     if (MiIsPfnInUse(Pfn1 as *mut ntkrnl_sys::ddk::_MMPFN)) break;
//
//                     //
//                     // So far so good. Is this the last confirmed valid page?
//                     //
//                     if (!--Length)
//                     {
//                         //
//                         // Sanity check that we didn't go out of bounds
//                         //
//                         ASSERT(i != MmPhysicalMemoryBlock->NumberOfRuns);
//
//                         //
//                         // Loop until all PFN entries have been processed
//                         //
//                         EndPfn = Pfn1 - SizeInPages + 1;
//                         do
//                         {
//                             //
//                             // This PFN is now a used page, set it up
//                             //
//                             MI_SET_USAGE(MI_USAGE_CONTINOUS_ALLOCATION);
//                             MI_SET_PROCESS2("Kernel Driver");
//                             MiUnlinkFreeOrZeroedPage(Pfn1);
//                             Pfn1->u3.e2.ReferenceCount = 1;
//                             Pfn1->u2.ShareCount = 1;
//                             Pfn1->u3.e1.PageLocation = ActiveAndValid;
//                             Pfn1->u3.e1.StartOfAllocation = 0;
//                             Pfn1->u3.e1.EndOfAllocation = 0;
//                             Pfn1->u3.e1.PrototypePte = 0;
//                             Pfn1->u4.VerifierAllocation = 0;
//                             Pfn1->PteAddress = (PVOID)(ULONG_PTR)0xBAADF00DBAADF00DULL;
//
//                             //
//                             // Check if this is the last PFN, otherwise go on
//                             //
//                             if (Pfn1 == EndPfn) break;
//                             Pfn1--;
//                         } while (TRUE);
//
//                         //
//                         // Mark the first and last PFN so we can find them later
//                         //
//                         Pfn1->u3.e1.StartOfAllocation = 1;
//                         (Pfn1 + SizeInPages - 1)->u3.e1.EndOfAllocation = 1;
//
//                         //
//                         // Now it's safe to let go of the PFN lock
//                         //
//                         MiReleasePfnLock(OldIrql);
//
//                         //
//                         // Quick sanity check that the last PFN is consistent
//                         //
//                         EndPfn = Pfn1 + SizeInPages;
//                         ASSERT(EndPfn == MI_PFN_ELEMENT(Page + 1));
//
//                         //
//                         // Compute the first page, and make sure it's consistent
//                         //
//                         Page = Page - SizeInPages + 1;
//                         ASSERT(Pfn1 == MI_PFN_ELEMENT(Page));
//                         ASSERT(Page != 0);
//
//                         /* Enable APCs and return the page */
//                         KeLeaveGuardedRegion();
//                         return Page;
//                     }
//
//                     //
//                     // Keep going. The purpose of this loop is to reconfirm that
//                     // after acquiring the PFN lock these pages are still usable
//                     //
//                     Pfn1++;
//                     Page++;
//                 } while (TRUE);
//
//                 //
//                 // If we got here, something changed while we hadn't acquired
//                 // the PFN lock yet, so we'll have to restart
//                 //
//                 MiReleasePfnLock(OldIrql);
//                 Length = 0;
//             }
//         }
//     } while (++i != MmPhysicalMemoryBlock->NumberOfRuns);
//
//     //
//     // And if we get here, it means no suitable physical memory runs were found
//     //
//     KeLeaveGuardedRegion();
//     return 0;
// }
//
// PVOID
// NTAPI
// MiCheckForContiguousMemory(IN PVOID BaseAddress,
//                            IN PFN_NUMBER BaseAddressPages,
//                            IN PFN_NUMBER SizeInPages,
//                            IN PFN_NUMBER LowestPfn,
//                            IN PFN_NUMBER HighestPfn,
//                            IN PFN_NUMBER BoundaryPfn,
//                            IN MI_PFN_CACHE_ATTRIBUTE CacheAttribute)
// {
//     PMMPTE StartPte, EndPte;
//     PFN_NUMBER PreviousPage = 0, Page, HighPage, BoundaryMask, Pages = 0;
//
//     //
//     // Okay, first of all check if the PFNs match our restrictions
//     //
//     if (LowestPfn > HighestPfn) return NULL;
//     if (LowestPfn + SizeInPages <= LowestPfn) return NULL;
//     if (LowestPfn + SizeInPages - 1 > HighestPfn) return NULL;
//     if (BaseAddressPages < SizeInPages) return NULL;
//
//     //
//     // This is the last page we need to get to and the boundary requested
//     //
//     HighPage = HighestPfn + 1 - SizeInPages;
//     BoundaryMask = ~(BoundaryPfn - 1);
//
//     //
//     // And here's the PTEs for this allocation. Let's go scan them.
//     //
//     StartPte = MiAddressToPte(BaseAddress);
//     EndPte = StartPte + BaseAddressPages;
//     while (StartPte < EndPte)
//     {
//         //
//         // Get this PTE's page number
//         //
//         ASSERT (StartPte->u.Hard.Valid == 1);
//         Page = PFN_FROM_PTE(StartPte);
//
//         //
//         // Is this the beginning of our adventure?
//         //
//         if (!Pages)
//         {
//             //
//             // Check if this PFN is within our range
//             //
//             if ((Page >= LowestPfn) && (Page <= HighPage))
//             {
//                 //
//                 // It is! Do you care about boundary (alignment)?
//                 //
//                 if (!(BoundaryPfn) ||
//                     (!((Page ^ (Page + SizeInPages - 1)) & BoundaryMask)))
//                 {
//                     //
//                     // You don't care, or you do care but we deliver
//                     //
//                     Pages++;
//                 }
//             }
//
//             //
//             // Have we found all the pages we need by now?
//             // Incidently, this means you only wanted one page
//             //
//             if (Pages == SizeInPages)
//             {
//                 //
//                 // Mission complete
//                 //
//                 return MiPteToAddress(StartPte);
//             }
//         }
//         else
//         {
//             //
//             // Have we found a page that doesn't seem to be contiguous?
//             //
//             if (Page != (PreviousPage + 1))
//             {
//                 //
//                 // Ah crap, we have to start over
//                 //
//                 Pages = 0;
//                 continue;
//             }
//
//             //
//             // Otherwise, we're still in the game. Do we have all our pages?
//             //
//             if (++Pages == SizeInPages)
//             {
//                 //
//                 // We do! This entire range was contiguous, so we'll return it!
//                 //
//                 return MiPteToAddress(StartPte - Pages + 1);
//             }
//         }
//
//         //
//         // Try with the next PTE, remember this PFN
//         //
//         PreviousPage = Page;
//         StartPte++;
//         continue;
//     }
//
//     //
//     // All good returns are within the loop...
//     //
//     return NULL;
// }
//
// PVOID
// NTAPI
// MiFindContiguousMemory(IN PFN_NUMBER LowestPfn,
//                        IN PFN_NUMBER HighestPfn,
//                        IN PFN_NUMBER BoundaryPfn,
//                        IN PFN_NUMBER SizeInPages,
//                        IN MEMORY_CACHING_TYPE CacheType)
// {
//     PFN_NUMBER Page;
//     PHYSICAL_ADDRESS PhysicalAddress;
//     PMMPFN Pfn1, EndPfn;
//     PMMPTE PointerPte;
//     PVOID BaseAddress;
//     PAGED_CODE();
//     ASSERT(SizeInPages != 0);
//
//     //
//     // Our last hope is to scan the free page list for contiguous pages
//     //
//     Page = MiFindContiguousPages(LowestPfn,
//                                  HighestPfn,
//                                  BoundaryPfn,
//                                  SizeInPages,
//                                  CacheType);
//     if (!Page) return NULL;
//
//     //
//     // We'll just piggyback on the I/O memory mapper
//     //
//     PhysicalAddress.QuadPart = Page << PAGE_SHIFT;
//     BaseAddress = MmMapIoSpace(PhysicalAddress, SizeInPages << PAGE_SHIFT, CacheType);
//     ASSERT(BaseAddress);
//
//     /* Loop the PFN entries */
//     Pfn1 = MiGetPfnEntry(Page);
//     EndPfn = Pfn1 + SizeInPages;
//     PointerPte = MiAddressToPte(BaseAddress);
//     do
//     {
//         /* Write the PTE address */
//         Pfn1->PteAddress = PointerPte;
//         Pfn1->u4.PteFrame = PFN_FROM_PTE(MiAddressToPte(PointerPte++));
//     } while (++Pfn1 < EndPfn);
//
//     /* Return the address */
//     return BaseAddress;
// }
//
// PVOID
// NTAPI
// MiAllocateContiguousMemory(IN SIZE_T NumberOfBytes,
//                            IN PFN_NUMBER LowestAcceptablePfn,
//                            IN PFN_NUMBER HighestAcceptablePfn,
//                            IN PFN_NUMBER BoundaryPfn,
//                            IN MEMORY_CACHING_TYPE CacheType)
// {
//     PVOID BaseAddress;
//     PFN_NUMBER SizeInPages;
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//
//     //
//     // Verify count and cache type
//     //
//     ASSERT(NumberOfBytes != 0);
//     ASSERT(CacheType <= MmWriteCombined);
//
//     //
//     // Compute size requested
//     //
//     SizeInPages = BYTES_TO_PAGES(NumberOfBytes);
//
//     //
//     // Convert the cache attribute and check for cached requests
//     //
//     CacheAttribute = MiPlatformCacheAttributes[FALSE][CacheType];
//     if (CacheAttribute == MiCached_u32)
//     {
//         //
//         // Because initial nonpaged pool is supposed to be contiguous, go ahead
//         // and try making a nonpaged pool allocation first.
//         //
//         BaseAddress = ExAllocatePoolWithTag(NonPagedPoolCacheAligned,
//                                             NumberOfBytes,
//                                             'mCmM');
//         if (BaseAddress)
//         {
//             //
//             // Now make sure it's actually contiguous (if it came from expansion
//             // it might not be).
//             //
//             if (MiCheckForContiguousMemory(BaseAddress,
//                                            SizeInPages,
//                                            SizeInPages,
//                                            LowestAcceptablePfn,
//                                            HighestAcceptablePfn,
//                                            BoundaryPfn,
//                                            CacheAttribute))
//             {
//                 //
//                 // Sweet, we're in business!
//                 //
//                 return BaseAddress;
//             }
//
//             //
//             // No such luck
//             //
//             ExFreePoolWithTag(BaseAddress, 'mCmM');
//         }
//     }
//
//     //
//     // According to MSDN, the system won't try anything else if you're higher
//     // than APC level.
//     //
//     if (KeGetCurrentIrql() > APC_LEVEL) return NULL;
//
//     //
//     // Otherwise, we'll go try to find some
//     //
//     BaseAddress = MiFindContiguousMemory(LowestAcceptablePfn,
//                                          HighestAcceptablePfn,
//                                          BoundaryPfn,
//                                          SizeInPages,
//                                          CacheType);
//     if (!BaseAddress)
//     {
//         DPRINT1("Unable to allocate contiguous memory for %d bytes (%d pages), out of memory!\n", NumberOfBytes, SizeInPages);
//     }
//     return BaseAddress;
// }
//
// VOID
// NTAPI
// MiFreeContiguousMemory(IN PVOID BaseAddress)
// {
//     KIRQL OldIrql;
//     PFN_NUMBER PageFrameIndex, LastPage, PageCount;
//     PMMPFN Pfn1, StartPfn;
//     PMMPTE PointerPte;
//     PAGED_CODE();
//
//     //
//     // First, check if the memory came from initial nonpaged pool, or expansion
//     //
//     if (((BaseAddress >= MmNonPagedPoolStart) &&
//          (BaseAddress < (PVOID)((ULONG_PTR)MmNonPagedPoolStart +
//                                 MmSizeOfNonPagedPoolInBytes))) ||
//         ((BaseAddress >= MmNonPagedPoolExpansionStart) &&
//          (BaseAddress < MmNonPagedPoolEnd)))
//     {
//         //
//         // It did, so just use the pool to free this
//         //
//         ExFreePoolWithTag(BaseAddress, 'mCmM');
//         return;
//     }
//
//     /* Get the PTE and frame number for the allocation*/
//     PointerPte = MiAddressToPte(BaseAddress);
//     PageFrameIndex = PFN_FROM_PTE(PointerPte);
//
//     //
//     // Now get the PFN entry for this, and make sure it's the correct one
//     //
//     Pfn1 = MiGetPfnEntry(PageFrameIndex);
//     if ((!Pfn1) || (Pfn1->u3.e1.StartOfAllocation == 0))
//     {
//         //
//         // This probably means you did a free on an address that was in between
//         //
//         KeBugCheckEx(BAD_POOL_CALLER,
//                      0x60,
//                      (ULONG_PTR)BaseAddress,
//                      0,
//                      0);
//     }
//
//     //
//     // Now this PFN isn't the start of any allocation anymore, it's going out
//     //
//     StartPfn = Pfn1;
//     Pfn1->u3.e1.StartOfAllocation = 0;
//
//     /* Loop the PFNs until we find the one that marks the end of the allocation */
//     do
//     {
//         /* Make sure these are the pages we setup in the allocation routine */
//         ASSERT(Pfn1->u3.e2.ReferenceCount == 1);
//         ASSERT(Pfn1->u2.ShareCount == 1);
//         ASSERT(Pfn1->PteAddress == PointerPte);
//         ASSERT(Pfn1->u3.e1.PageLocation == ActiveAndValid);
//         ASSERT(Pfn1->u4.VerifierAllocation == 0);
//         ASSERT(Pfn1->u3.e1.PrototypePte == 0);
//
//         /* Set the special pending delete marker */
//         MI_SET_PFN_DELETED(Pfn1);
//
//         /* Keep going for assertions */
//         PointerPte++;
//     } while (Pfn1++->u3.e1.EndOfAllocation == 0);
//
//     //
//     // Found it, unmark it
//     //
//     Pfn1--;
//     Pfn1->u3.e1.EndOfAllocation = 0;
//
//     //
//     // Now compute how many pages this represents
//     //
//     PageCount = (ULONG)(Pfn1 - StartPfn + 1);
//
//     //
//     // So we can know how much to unmap (recall we piggyback on I/O mappings)
//     //
//     MmUnmapIoSpace(BaseAddress, PageCount << PAGE_SHIFT);
//
//     //
//     // Lock the PFN database
//     //
//     OldIrql = MiAcquirePfnLock();
//
//     //
//     // Loop all the pages
//     //
//     LastPage = PageFrameIndex + PageCount;
//     Pfn1 = MiGetPfnEntry(PageFrameIndex);
//     do
//     {
//         /* Decrement the share count and move on */
//         MiDecrementShareCount(Pfn1++, PageFrameIndex++);
//     } while (PageFrameIndex < LastPage);
//
//     //
//     // Release the PFN lock
//     //
//     MiReleasePfnLock(OldIrql);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmAllocateContiguousMemorySpecifyCache(IN SIZE_T NumberOfBytes,
//                                        IN PHYSICAL_ADDRESS LowestAcceptableAddress OPTIONAL,
//                                        IN PHYSICAL_ADDRESS HighestAcceptableAddress,
//                                        IN PHYSICAL_ADDRESS BoundaryAddressMultiple OPTIONAL,
//                                        IN MEMORY_CACHING_TYPE CacheType OPTIONAL)
// {
//     PFN_NUMBER LowestPfn, HighestPfn, BoundaryPfn;
//
//     //
//     // Verify count and cache type
//     //
//     ASSERT(NumberOfBytes != 0);
//     ASSERT(CacheType <= MmWriteCombined);
//
//     //
//     // Convert the lowest address into a PFN
//     //
//     LowestPfn = (PFN_NUMBER)(LowestAcceptableAddress.QuadPart >> PAGE_SHIFT);
//     if (BYTE_OFFSET(LowestAcceptableAddress.__bindgen_anon_1.__bindgen_anon_1.LowPart)) LowestPfn++;
//
//     //
//     // Convert and validate the boundary address into a PFN
//     //
//     if (BYTE_OFFSET(BoundaryAddressMultiple.__bindgen_anon_1.__bindgen_anon_1.LowPart)) return NULL;
//     BoundaryPfn = (PFN_NUMBER)(BoundaryAddressMultiple.QuadPart >> PAGE_SHIFT);
//
//     //
//     // Convert the highest address into a PFN
//     //
//     HighestPfn = (PFN_NUMBER)(HighestAcceptableAddress.QuadPart >> PAGE_SHIFT);
//     if (HighestPfn > MmHighestPhysicalPage) HighestPfn = MmHighestPhysicalPage;
//
//     //
//     // Validate the PFN bounds
//     //
//     if (LowestPfn > HighestPfn) return NULL;
//
//     //
//     // Let the contiguous memory allocator handle it
//     //
//     return MiAllocateContiguousMemory(NumberOfBytes,
//                                       LowestPfn,
//                                       HighestPfn,
//                                       BoundaryPfn,
//                                       CacheType);
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmAllocateContiguousMemory(IN SIZE_T NumberOfBytes,
//                            IN PHYSICAL_ADDRESS HighestAcceptableAddress)
// {
//     PFN_NUMBER HighestPfn;
//
//     //
//     // Verify byte count
//     //
//     ASSERT(NumberOfBytes != 0);
//
//     //
//     // Convert and normalize the highest address into a PFN
//     //
//     HighestPfn = (PFN_NUMBER)(HighestAcceptableAddress.QuadPart >> PAGE_SHIFT);
//     if (HighestPfn > MmHighestPhysicalPage) HighestPfn = MmHighestPhysicalPage;
//
//     //
//     // Let the contiguous memory allocator handle it
//     //
//     return MiAllocateContiguousMemory(NumberOfBytes, 0, HighestPfn, 0, MmCached);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmFreeContiguousMemory(IN PVOID BaseAddress)
// {
//     //
//     // Let the contiguous memory allocator handle it
//     //
//     MiFreeContiguousMemory(BaseAddress);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmFreeContiguousMemorySpecifyCache(IN PVOID BaseAddress,
//                                    IN SIZE_T NumberOfBytes,
//                                    IN MEMORY_CACHING_TYPE CacheType)
// {
//     //
//     // Just call the non-cached version (there's no cache issues for freeing)
//     //
//     MiFreeContiguousMemory(BaseAddress);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
