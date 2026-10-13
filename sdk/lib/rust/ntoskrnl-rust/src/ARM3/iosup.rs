use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

extern "system" {
    fn KeInvalidateAllCaches() -> BOOLEAN;
}

use ntkrnl_sys::ddk::{
    LOGICAL, PHYSICAL_ADDRESS, MEMORY_CACHING_TYPE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// Row 0 is RAM, row 1 is I/O space, indexed by MEMORY_CACHING_TYPE.
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiPlatformCacheAttributes: [[MI_PFN_CACHE_ATTRIBUTE; MmMaximumCacheType as usize]; 2] = [
    [MiNonCached, MiCached_u32, MiWriteCombined, MiCached_u32, MiNonCached, MiWriteCombined],
    [MiNonCached, MiCached_u32, MiWriteCombined, MiCached_u32, MiNonCached, MiWriteCombined],
];
// arm64: uncached RAM is Normal Non-cacheable (MiWriteCombined, MAIR AttrIndx 2), not
// MiNonCached (AttrIndx 1, Device-nGnRnE). Device memory is for MMIO: RAM mapped as Device
// also aliases the cached linear mapping with a mismatched memory TYPE, and every unaligned
// access to it alignment-faults, which compilers emit freely for byte stores they merge.
// atapi's SATL wrote a VPD page into its MmNonCached local buffer with one unaligned `stur`
// and took a 0x1E. I/O space keeps Device.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MiPlatformCacheAttributes: [[MI_PFN_CACHE_ATTRIBUTE; MmMaximumCacheType as usize]; 2] = [
    [MiWriteCombined, MiCached_u32, MiWriteCombined, MiCached_u32, MiWriteCombined, MiWriteCombined],
    [MiNonCached, MiCached_u32, MiWriteCombined, MiCached_u32, MiNonCached, MiWriteCombined],
];

#[no_mangle]
pub unsafe extern "system" fn MmMapIoSpace(
    PhysicalAddress: PHYSICAL_ADDRESS,
    NumberOfBytes: SIZE_T,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    let mut Pfn: PFN_NUMBER;
    let mut PageCount: PFN_COUNT;
    let mut PointerPte: PMMPTE;
    let mut BaseAddress: PVOID;
    let mut TempPte: MMPTE;
    let mut Pfn1: PMMPFN = core::ptr::null_mut();
    let CacheAttribute: MI_PFN_CACHE_ATTRIBUTE;
    let IsIoMapping: BOOLEAN;

    ntkrnl_sys::ASSERT!(NumberOfBytes != 0);

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        ntkrnl_sys::ASSERT!(PhysicalAddress.__bindgen_anon_1.HighPart == 0);
    }

    let CacheType = CacheType & 0xFF;
    if CacheType >= MmMaximumCacheType {
        return core::ptr::null_mut();
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(PhysicalAddress.QuadPart as PVOID, NumberOfBytes) as PFN_COUNT;
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(PhysicalAddress.__bindgen_anon_1.LowPart as usize as PVOID, NumberOfBytes) as PFN_COUNT;
    }

    Pfn = (PhysicalAddress.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    Pfn1 = MiGetPfnEntry(Pfn);
    IsIoMapping = if Pfn1.is_null() { 1u8 } else { 0u8 };
    CacheAttribute = MiPlatformCacheAttributes[IsIoMapping as usize][CacheType as usize];

    PointerPte = MiReserveSystemPtes(PageCount, SystemPteSpace);
    if PointerPte.is_null() {
        return core::ptr::null_mut();
    }
    BaseAddress = MiPteToAddress(PointerPte);

    if CacheAttribute != MiCached_u32 {
        KeFlushEntireTb(1u8, 1u8);
        KeInvalidateAllCaches();
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        BaseAddress = (BaseAddress as ULONG_PTR + (PhysicalAddress.QuadPart as usize & (PAGE_SIZE - 1))) as PVOID;
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        BaseAddress = (BaseAddress as ULONG_PTR + (PhysicalAddress.__bindgen_anon_1.LowPart & (PAGE_SIZE - 1) as u32) as ULONG_PTR) as PVOID;
    }

    TempPte = ValidKernelPte;
    if CacheAttribute == MiNonCached {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_THROUGH(&mut TempPte);
    } else if CacheAttribute == MiCached_u32 {
        // leave defaults
    } else if CacheAttribute == MiWriteCombined {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_COMBINED(&mut TempPte);
    } else {
        ntkrnl_sys::ASSERT!(false);
    }

    Pfn = (PhysicalAddress.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    ntkrnl_sys::ASSERT!((Pfn1 == MiGetPfnEntry(Pfn)) || Pfn1.is_null());
    KeFlushEntireTb(1u8, 1u8);
    KeInvalidateAllCaches();

    loop {
        TempPte.set_page_frame_number(Pfn);
        Pfn += 1;
        MI_WRITE_VALID_PTE(PointerPte, TempPte);
        PointerPte = PointerPte.add(1);
        PageCount -= 1;
        if PageCount == 0 {
            break;
        }
    }

    BaseAddress
}

/// MmMapIoSpaceEx (Windows 10): MmMapIoSpace with the caching taken from a page
/// protection. The base protection must be a readable one, optionally with exactly one of
/// PAGE_NOCACHE / PAGE_WRITECOMBINE; no modifier means cached. Anything else is NULL.
/// The access part is checked but not enforced: the view is a kernel read-write mapping,
/// as every MmMapIoSpace view is.
#[no_mangle]
pub unsafe extern "system" fn MmMapIoSpaceEx(
    PhysicalAddress: PHYSICAL_ADDRESS,
    NumberOfBytes: SIZE_T,
    Protect: ULONG,
) -> PVOID {
    let cache = match Protect & (PAGE_NOCACHE | PAGE_WRITECOMBINE) {
        0 => MmCached_val,
        PAGE_NOCACHE => MmNonCached,
        PAGE_WRITECOMBINE => MmWriteCombined,
        _ => return core::ptr::null_mut(),
    };
    match Protect & !(PAGE_NOCACHE | PAGE_WRITECOMBINE) {
        PAGE_READONLY | PAGE_READWRITE | PAGE_EXECUTE | PAGE_EXECUTE_READ
        | PAGE_EXECUTE_READWRITE => {}
        _ => return core::ptr::null_mut(),
    }
    if NumberOfBytes == 0 {
        return core::ptr::null_mut();
    }
    MmMapIoSpace(PhysicalAddress, NumberOfBytes, cache)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapIoSpace(
    BaseAddress: PVOID,
    NumberOfBytes: SIZE_T,
) {
    let BaseAddress = opaque_ptr(BaseAddress);
    let Pfn: PFN_NUMBER;
    let PageCount: PFN_COUNT;
    let PointerPte: PMMPTE;

    ntkrnl_sys::ASSERT!(NumberOfBytes != 0);

    PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(BaseAddress, NumberOfBytes) as PFN_COUNT;

    PointerPte = MiAddressToPte(BaseAddress);
    Pfn = PFN_FROM_PTE(PointerPte);

    if MiGetPfnEntry(Pfn).is_null() {
        core::ptr::write_bytes(PointerPte as *mut u8, 0, PageCount as usize * core::mem::size_of::<MMPTE>());
        KeFlushEntireTb(1u8, 1u8);
    }

    MiReleaseSystemPtes(PointerPte, PageCount as ULONG, 0);
}

#[no_mangle]
pub unsafe extern "system" fn MmMapVideoDisplay(
    PhysicalAddress: PHYSICAL_ADDRESS,
    NumberOfBytes: SIZE_T,
    CacheType: MEMORY_CACHING_TYPE,
) -> PVOID {
    MmMapIoSpace(PhysicalAddress, NumberOfBytes, CacheType)
}

#[no_mangle]
pub unsafe extern "system" fn MmUnmapVideoDisplay(
    BaseAddress: PVOID,
    NumberOfBytes: SIZE_T,
) {
    let BaseAddress = opaque_ptr(BaseAddress);
    MmUnmapIoSpace(BaseAddress, NumberOfBytes);
}

#[no_mangle]
pub unsafe extern "system" fn MmIsIoSpaceActive(
    _StartAddress: PHYSICAL_ADDRESS,
    _NumberOfBytes: SIZE_T,
) -> LOGICAL {
    0 as LOGICAL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/iosup.c
//  * PURPOSE:         ARM Memory Manager I/O Mapping Functionality
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
// /* GLOBALS ********************************************************************/
//
// //
// // Each architecture has its own caching attributes for both I/O and Physical
// // memory mappings.
// //
// // This describes the attributes for the x86 architecture. It eventually needs
// // to go in the appropriate i386 directory.
// //
// MI_PFN_CACHE_ATTRIBUTE MiPlatformCacheAttributes[2][MmMaximumCacheType] =
// {
//     //
//     // RAM
//     //
//     {MiNonCached,MiCached,MiWriteCombined,MiCached,MiNonCached,MiWriteCombined},
//
//     //
//     // Device Memory
//     //
//     {MiNonCached,MiCached,MiWriteCombined,MiCached,MiNonCached,MiWriteCombined},
// };
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmMapIoSpace(IN PHYSICAL_ADDRESS PhysicalAddress,
//              IN SIZE_T NumberOfBytes,
//              IN MEMORY_CACHING_TYPE CacheType)
// {
//
//     PFN_NUMBER Pfn;
//     PFN_COUNT PageCount;
//     PMMPTE PointerPte;
//     PVOID BaseAddress;
//     MMPTE TempPte;
//     PMMPFN Pfn1 = NULL;
//     MI_PFN_CACHE_ATTRIBUTE CacheAttribute;
//     BOOLEAN IsIoMapping;
//
//     //
//     // Must be called with a non-zero count
//     //
//     ASSERT(NumberOfBytes != 0);
//
//     //
//     // Make sure the upper bits are 0 if this system
//     // can't describe more than 4 GB of physical memory.
//     // FIXME: This doesn't respect PAE, but we currently don't
//     // define a PAE build flag since there is no such build.
//     //
// #if !defined(_M_AMD64) && !defined(_M_ARM64)
//     ASSERT(PhysicalAddress.__bindgen_anon_1.HighPart == 0);
// #endif
//
//     //
//     // Normalize and validate the caching attributes
//     //
//     CacheType &= 0xFF;
//     if (CacheType >= MmMaximumCacheType) return NULL;
//
//     //
//     // Calculate page count
//     //
// #if defined(_M_AMD64) || defined(_M_ARM64)
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(PhysicalAddress.QuadPart,
//                                                NumberOfBytes);
// #else
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(PhysicalAddress.__bindgen_anon_1.LowPart,
//                                                NumberOfBytes);
// #endif
//
//     //
//     // Compute the PFN and check if it's a known I/O mapping
//     // Also translate the cache attribute
//     //
//     Pfn = (PFN_NUMBER)(PhysicalAddress.QuadPart >> PAGE_SHIFT);
//     Pfn1 = MiGetPfnEntry(Pfn);
//     IsIoMapping = (Pfn1 == NULL) ? TRUE : FALSE;
//     CacheAttribute = MiPlatformCacheAttributes[IsIoMapping][CacheType];
//
//     //
//     // Now allocate system PTEs for the mapping, and get the VA
//     //
//     PointerPte = MiReserveSystemPtes(PageCount, SystemPteSpace);
//     if (!PointerPte) return NULL;
//     BaseAddress = MiPteToAddress(PointerPte);
//
//     //
//     // Check if this is uncached
//     //
//     if (CacheAttribute != MiCached_u32)
//     {
//         //
//         // Flush all caches
//         //
//         KeFlushEntireTb(TRUE, TRUE);
//         KeInvalidateAllCaches();
//     }
//
//     //
//     // Now compute the VA offset
//     //
// #if defined(_M_AMD64) || defined(_M_ARM64)
//     BaseAddress = (PVOID)((ULONG_PTR)BaseAddress +
//                           BYTE_OFFSET(PhysicalAddress.QuadPart));
// #else
//     BaseAddress = (PVOID)((ULONG_PTR)BaseAddress +
//                           BYTE_OFFSET(PhysicalAddress.__bindgen_anon_1.LowPart));
// #endif
//
//     //
//     // Get the template and configure caching
//     //
//     TempPte = ValidKernelPte;
//     switch (CacheAttribute)
//     {
//         case MiNonCached:
//
//             //
//             // Disable the cache
//             //
//             MI_PAGE_DISABLE_CACHE(&TempPte);
//             MI_PAGE_WRITE_THROUGH(&TempPte);
//             break;
//
//         case MiCached:
//
//             //
//             // Leave defaults
//             //
//             break;
//
//         case MiWriteCombined:
//
//             //
//             // Disable the cache and allow combined writing
//             //
//             MI_PAGE_DISABLE_CACHE(&TempPte);
//             MI_PAGE_WRITE_COMBINED(&TempPte);
//             break;
//
//         default:
//
//             //
//             // Should never happen
//             //
//             ASSERT(FALSE);
//             break;
//     }
//
//     //
//     // Sanity check and re-flush
//     //
//     Pfn = (PFN_NUMBER)(PhysicalAddress.QuadPart >> PAGE_SHIFT);
//     ASSERT((Pfn1 == MiGetPfnEntry(Pfn)) || (Pfn1 == NULL));
//     KeFlushEntireTb(TRUE, TRUE);
//     KeInvalidateAllCaches();
//
//     //
//     // Do the mapping
//     //
//     do
//     {
//         //
//         // Write the PFN
//         //
//         TempPte.set_page_frame_number(Pfn++);
//         MI_WRITE_VALID_PTE(PointerPte++, TempPte);
//     } while (--PageCount);
//
//     //
//     // We're done!
//     //
//     return BaseAddress;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmUnmapIoSpace(IN PVOID BaseAddress,
//                IN SIZE_T NumberOfBytes)
// {
//     PFN_NUMBER Pfn;
//     PFN_COUNT PageCount;
//     PMMPTE PointerPte;
//
//     //
//     // Sanity check
//     //
//     ASSERT(NumberOfBytes != 0);
//
//     //
//     // Get the page count
//     //
//     PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(BaseAddress, NumberOfBytes);
//
//     //
//     // Get the PTE and PFN
//     //
//     PointerPte = MiAddressToPte(BaseAddress);
//     Pfn = PFN_FROM_PTE(PointerPte);
//
//     //
//     // Is this an I/O mapping?
//     //
//     if (!MiGetPfnEntry(Pfn))
//     {
//         //
//         // Destroy the PTE
//         //
//         RtlZeroMemory(PointerPte, PageCount * sizeof(MMPTE));
//
//         //
//         // Blow the TLB
//         //
//         KeFlushEntireTb(TRUE, TRUE);
//     }
//
//     //
//     // Release the PTEs
//     //
//     MiReleaseSystemPtes(PointerPte, PageCount, 0);
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// MmMapVideoDisplay(IN PHYSICAL_ADDRESS PhysicalAddress,
//                   IN SIZE_T NumberOfBytes,
//                   IN MEMORY_CACHING_TYPE CacheType)
// {
//     PAGED_CODE();
//
//     //
//     // Call the real function
//     //
//     return MmMapIoSpace(PhysicalAddress, NumberOfBytes, CacheType);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// MmUnmapVideoDisplay(IN PVOID BaseAddress,
//                     IN SIZE_T NumberOfBytes)
// {
//     //
//     // Call the real function
//     //
//     MmUnmapIoSpace(BaseAddress, NumberOfBytes);
// }
//
// LOGICAL
// NTAPI
// MmIsIoSpaceActive(IN PHYSICAL_ADDRESS StartAddress,
//                   IN SIZE_T NumberOfBytes)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
