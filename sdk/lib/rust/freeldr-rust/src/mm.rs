#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_MEMORY; // mm.c: DBG_DEFAULT_CHANNEL(MEMORY)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::meminit::*;
use crate::heap::*;
use core::ffi::c_void;
extern "C" {
}

extern "C" {
    fn DbgPrint(fmt: *const i8, ...) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn MmAllocateMemoryWithType(
    MemorySize: usize,
    MemoryType: TYPE_OF_MEMORY,
) -> PVOID {
    let PagesNeeded: PFN_NUMBER;
    let FirstFreePageFromEnd: PFN_NUMBER;
    let MemPointer: PVOID;
    let mut MemorySize = MemorySize;

    if MemorySize == 0 {
        WARN!("MmAllocateMemory() called for 0 bytes. Returning NULL.\n");
        UiMessageBoxCritical(b"Memory allocation failed: MmAllocateMemory() called for 0 bytes.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    MemorySize = ROUND_UP(MemorySize, 4);

    PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;

    if FreePagesInLookupTable < PagesNeeded {
        ERR!("Memory allocation failed in MmAllocateMemory(). Not enough free memory to allocate %d bytes.\n", MemorySize);
        UiMessageBoxCritical(b"Memory allocation failed: out of memory.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    FirstFreePageFromEnd = MmFindAvailablePages(PageLookupTableAddress, TotalPagesInLookupTable, PagesNeeded, 0u8);

    if FirstFreePageFromEnd == 0 {
        ERR!("Memory allocation failed in MmAllocateMemory(). Not enough free memory to allocate %d bytes.\n", MemorySize);
        UiMessageBoxCritical(b"Memory allocation failed: out of memory.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    MmAllocatePagesInLookupTable(PageLookupTableAddress, FirstFreePageFromEnd, PagesNeeded, MemoryType);

    FreePagesInLookupTable -= PagesNeeded;
    MemPointer = (FirstFreePageFromEnd * MM_PAGE_SIZE) as PVOID;

    TRACE!("Allocated %d bytes (%d pages) of memory (type %ld) starting at page 0x%lx.\n", MemorySize, PagesNeeded, MemoryType, FirstFreePageFromEnd);
    TRACE!("Memory allocation pointer: 0x%x\n", MemPointer);

    if ((MemPointer as usize + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT) > LoaderPagesSpanned {
        LoaderPagesSpanned = ((MemPointer as usize + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT) as PFN_NUMBER;
    }

    MemPointer
}

#[no_mangle]
pub unsafe extern "C" fn MmAllocateMemoryAtAddress(MemorySize: usize, DesiredAddress: *mut core::ffi::c_void, MemoryType: u32) -> *mut core::ffi::c_void {
    let PagesNeeded: PFN_NUMBER;
    let StartPageNumber: PFN_NUMBER;
    let MemPointer: *mut core::ffi::c_void;

    if MemorySize == 0 {
        WARN!("MmAllocateMemoryAtAddress() called for 0 bytes. Returning NULL.\n");
        UiMessageBoxCritical(b"Memory allocation failed: MmAllocateMemoryAtAddress() called for 0 bytes.\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;

    StartPageNumber = MmGetPageNumberFromAddress(DesiredAddress);

    if FreePagesInLookupTable < PagesNeeded {
        ERR!("Memory allocation failed in MmAllocateMemoryAtAddress(). Not enough free memory to allocate %d bytes (requesting %d pages but have only %d). \n", MemorySize, PagesNeeded, FreePagesInLookupTable);
        UiMessageBoxCritical(b"Memory allocation failed: out of memory.\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    if MmAreMemoryPagesAvailable(PageLookupTableAddress, TotalPagesInLookupTable, DesiredAddress, PagesNeeded) == 0u8 {
        WARN!("Memory allocation failed in MmAllocateMemoryAtAddress(). Not enough free memory to allocate %d bytes at address %p.\n", MemorySize, DesiredAddress);

        return core::ptr::null_mut();
    }

    MmAllocatePagesInLookupTable(PageLookupTableAddress, StartPageNumber, PagesNeeded, MemoryType);

    FreePagesInLookupTable -= PagesNeeded;
    MemPointer = (StartPageNumber * MM_PAGE_SIZE) as *mut core::ffi::c_void;

    TRACE!("Allocated %d bytes (%d pages) of memory starting at page %d.\n", MemorySize, PagesNeeded, StartPageNumber);
    TRACE!("Memory allocation pointer: 0x%x\n", MemPointer);

    if ((MemPointer as usize + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT) > LoaderPagesSpanned  {
        LoaderPagesSpanned = (MemPointer as usize + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT ;
    }

    MemPointer
}

#[no_mangle]
pub unsafe extern "C" fn MmSetMemoryType(MemoryAddress: PVOID, MemorySize: SIZE_T, NewType: TYPE_OF_MEMORY) {
    let PagesNeeded: PFN_NUMBER;
    let StartPageNumber: PFN_NUMBER;

    PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;

    StartPageNumber = MmGetPageNumberFromAddress(MemoryAddress);

    MmAllocatePagesInLookupTable(PageLookupTableAddress, StartPageNumber, PagesNeeded, NewType);
}

#[no_mangle]
pub unsafe extern "C" fn MmAllocateHighestMemoryBelowAddress(
    MemorySize: SIZE_T,
    DesiredAddress: PVOID,
    MemoryType: TYPE_OF_MEMORY,
) -> PVOID {
    let PagesNeeded: PFN_NUMBER;
    let FirstFreePageFromEnd: PFN_NUMBER;
    let DesiredAddressPageNumber: PFN_NUMBER;
    let MemPointer: PVOID;

    if MemorySize == 0 {
        WARN!("MmAllocateHighestMemoryBelowAddress() called for 0 bytes. Returning NULL.\n");
        UiMessageBoxCritical(b"Memory allocation failed: MmAllocateHighestMemoryBelowAddress() called for 0 bytes.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;

    DesiredAddressPageNumber = (DesiredAddress as ULONG_PTR) / MM_PAGE_SIZE;

    if FreePagesInLookupTable < PagesNeeded {
        ERR!("Memory allocation failed in MmAllocateHighestMemoryBelowAddress(). Not enough free memory to allocate %d bytes.\n", MemorySize);
        UiMessageBoxCritical(b"Memory allocation failed: out of memory.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    FirstFreePageFromEnd = MmFindAvailablePagesBeforePage(PageLookupTableAddress, TotalPagesInLookupTable, PagesNeeded, DesiredAddressPageNumber);

    if FirstFreePageFromEnd == 0 {
        ERR!("Memory allocation failed in MmAllocateHighestMemoryBelowAddress(). Not enough free memory to allocate %d bytes.\n", MemorySize);
        UiMessageBoxCritical(b"Memory allocation failed: out of memory.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    MmAllocatePagesInLookupTable(PageLookupTableAddress, FirstFreePageFromEnd, PagesNeeded, MemoryType);

    FreePagesInLookupTable -= PagesNeeded;
    MemPointer = (FirstFreePageFromEnd * MM_PAGE_SIZE) as PVOID;

    TRACE!("Allocated %d bytes (%d pages) of memory starting at page %d.\n", MemorySize, PagesNeeded, FirstFreePageFromEnd);
    TRACE!("Memory allocation pointer: 0x%x\n", MemPointer);

    if (((MemPointer as ULONG_PTR) + MemorySize) >> PAGE_SHIFT) > LoaderPagesSpanned {
        LoaderPagesSpanned = ((MemPointer as ULONG_PTR) + MemorySize) >> PAGE_SHIFT ;
    }

    MemPointer
}

#[no_mangle]
pub unsafe extern "C" fn MmFreeMemory(_MemoryPointer: PVOID) {
}

unsafe fn DumpMemoryAllocMap() {

    let RealPageLookupTable = PageLookupTableAddress as *mut PAGE_LOOKUP_TABLE_ITEM;

    DbgPrint(b"----------- Memory Allocation Bitmap -----------\n\0".as_ptr() as *const i8);

    let mut Idx: PFN_NUMBER = 0;
    while Idx < TotalPagesInLookupTable {
        if (Idx % 32) == 0 {
            DbgPrint(b"\n\0".as_ptr() as *const i8);
            DbgPrint(b"%08x:\t\0".as_ptr() as *const i8, Idx * MM_PAGE_SIZE);
        } else if (Idx % 4) == 0 {
            DbgPrint(b" \0".as_ptr() as *const i8);
        }

        match (*RealPageLookupTable.add(Idx as usize)).PageAllocated {
            LoaderFree => {
                DbgPrint(b"*\0".as_ptr() as *const i8);
            }
            LoaderBad => {
                DbgPrint(b"-\0".as_ptr() as *const i8);
            }
            LoaderLoadedProgram => {
                DbgPrint(b"O\0".as_ptr() as *const i8);
            }
            LoaderFirmwareTemporary => {
                DbgPrint(b"T\0".as_ptr() as *const i8);
            }
            LoaderFirmwarePermanent => {
                DbgPrint(b"P\0".as_ptr() as *const i8);
            }
            LoaderOsloaderHeap => {
                DbgPrint(b"H\0".as_ptr() as *const i8);
            }
            LoaderOsloaderStack => {
                DbgPrint(b"S\0".as_ptr() as *const i8);
            }
            LoaderSystemCode => {
                DbgPrint(b"K\0".as_ptr() as *const i8);
            }
            LoaderHalCode => {
                DbgPrint(b"L\0".as_ptr() as *const i8);
            }
            LoaderBootDriver => {
                DbgPrint(b"B\0".as_ptr() as *const i8);
            }
            LoaderStartupPcrPage => {
                DbgPrint(b"G\0".as_ptr() as *const i8);
            }
            LoaderRegistryData => {
                DbgPrint(b"R\0".as_ptr() as *const i8);
            }
            LoaderMemoryData => {
                DbgPrint(b"M\0".as_ptr() as *const i8);
            }
            LoaderNlsData => {
                DbgPrint(b"N\0".as_ptr() as *const i8);
            }
            LoaderSpecialMemory => {
                DbgPrint(b"C\0".as_ptr() as *const i8);
            }
            _ => {
                DbgPrint(b"?\0".as_ptr() as *const i8);
            }
        }

        Idx += 1;
    }

    DbgPrint(b"\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "C" fn MmGetMemoryMap(NoEntries: *mut PFN_NUMBER) -> PPAGE_LOOKUP_TABLE_ITEM {
    let RealPageLookupTable = PageLookupTableAddress as PPAGE_LOOKUP_TABLE_ITEM;
    
    *NoEntries = TotalPagesInLookupTable;
    
    return RealPageLookupTable;
}

#[no_mangle]
pub unsafe extern "C" fn MmGetLoaderPagesSpanned() -> PFN_NUMBER {
    LoaderPagesSpanned
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 2006-2008     Aleksey Bragin  <aleksey@reactos.org>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
// #include <debug.h>
//
// #if DBG
// VOID    DumpMemoryAllocMap(VOID);
// #endif // DBG
//
// DBG_DEFAULT_CHANNEL(MEMORY);
//
// PFN_NUMBER LoaderPagesSpanned = 0;
//
// PVOID MmAllocateMemoryWithType(SIZE_T MemorySize, TYPE_OF_MEMORY MemoryType)
// {
//     PFN_NUMBER    PagesNeeded;
//     PFN_NUMBER    FirstFreePageFromEnd;
//     PVOID    MemPointer;
//
//     if (MemorySize == 0)
//     {
//         WARN("MmAllocateMemory() called for 0 bytes. Returning NULL.\n");
//         UiMessageBoxCritical("Memory allocation failed: MmAllocateMemory() called for 0 bytes.");
//         return NULL;
//     }
//
//     MemorySize = ROUND_UP(MemorySize, 4);
//
//     // Find out how many blocks it will take to
//     // satisfy this allocation
//     PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;
//
//     // If we don't have enough available mem
//     // then return NULL
//     if (FreePagesInLookupTable < PagesNeeded)
//     {
//         ERR("Memory allocation failed in MmAllocateMemory(). Not enough free memory to allocate %d bytes.\n", MemorySize);
//         UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     FirstFreePageFromEnd = MmFindAvailablePages(PageLookupTableAddress, TotalPagesInLookupTable, PagesNeeded, FALSE);
//
//     if (FirstFreePageFromEnd == 0)
//     {
//         ERR("Memory allocation failed in MmAllocateMemory(). Not enough free memory to allocate %d bytes.\n", MemorySize);
//         UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     MmAllocatePagesInLookupTable(PageLookupTableAddress, FirstFreePageFromEnd, PagesNeeded, MemoryType);
//
//     FreePagesInLookupTable -= PagesNeeded;
//     MemPointer = (PVOID)((ULONG_PTR)FirstFreePageFromEnd * MM_PAGE_SIZE);
//
//     TRACE("Allocated %d bytes (%d pages) of memory (type %ld) starting at page 0x%lx.\n",
//           MemorySize, PagesNeeded, MemoryType, FirstFreePageFromEnd);
//     TRACE("Memory allocation pointer: 0x%x\n", MemPointer);
//
//     // Update LoaderPagesSpanned count
//     if ((((ULONG_PTR)MemPointer + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT) > LoaderPagesSpanned)
//         LoaderPagesSpanned = (((ULONG_PTR)MemPointer + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT);
//
//     // Now return the pointer
//     return MemPointer;
// }
//
// PVOID MmAllocateMemoryAtAddress(SIZE_T MemorySize, PVOID DesiredAddress, TYPE_OF_MEMORY MemoryType)
// {
//     PFN_NUMBER        PagesNeeded;
//     PFN_NUMBER        StartPageNumber;
//     PVOID    MemPointer;
//
//     if (MemorySize == 0)
//     {
//         WARN("MmAllocateMemoryAtAddress() called for 0 bytes. Returning NULL.\n");
//         UiMessageBoxCritical("Memory allocation failed: MmAllocateMemoryAtAddress() called for 0 bytes.");
//         return NULL;
//     }
//
//     // Find out how many blocks it will take to
//     // satisfy this allocation
//     PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;
//
//     // Get the starting page number
//     StartPageNumber = MmGetPageNumberFromAddress(DesiredAddress);
//
//     // If we don't have enough available mem
//     // then return NULL
//     if (FreePagesInLookupTable < PagesNeeded)
//     {
//         ERR("Memory allocation failed in MmAllocateMemoryAtAddress(). "
//             "Not enough free memory to allocate %d bytes (requesting %d pages but have only %d). "
//             "\n", MemorySize, PagesNeeded, FreePagesInLookupTable);
//         UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     if (MmAreMemoryPagesAvailable(PageLookupTableAddress, TotalPagesInLookupTable, DesiredAddress, PagesNeeded) == FALSE)
//     {
//         WARN("Memory allocation failed in MmAllocateMemoryAtAddress(). "
//              "Not enough free memory to allocate %d bytes at address %p.\n",
//              MemorySize, DesiredAddress);
//
//         // Don't tell this to user since caller should try to alloc this memory
//         // at a different address
//         //UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     MmAllocatePagesInLookupTable(PageLookupTableAddress, StartPageNumber, PagesNeeded, MemoryType);
//
//     FreePagesInLookupTable -= PagesNeeded;
//     MemPointer = (PVOID)((ULONG_PTR)StartPageNumber * MM_PAGE_SIZE);
//
//     TRACE("Allocated %d bytes (%d pages) of memory starting at page %d.\n", MemorySize, PagesNeeded, StartPageNumber);
//     TRACE("Memory allocation pointer: 0x%x\n", MemPointer);
//
//     // Update LoaderPagesSpanned count
//     if ((((ULONG_PTR)MemPointer + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT) > LoaderPagesSpanned)
//         LoaderPagesSpanned = (((ULONG_PTR)MemPointer + MemorySize + PAGE_SIZE - 1) >> PAGE_SHIFT);
//
//     // Now return the pointer
//     return MemPointer;
// }
//
// VOID MmSetMemoryType(PVOID MemoryAddress, SIZE_T MemorySize, TYPE_OF_MEMORY NewType)
// {
//     PFN_NUMBER        PagesNeeded;
//     PFN_NUMBER        StartPageNumber;
//
//     // Find out how many blocks it will take to
//     // satisfy this allocation
//     PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;
//
//     // Get the starting page number
//     StartPageNumber = MmGetPageNumberFromAddress(MemoryAddress);
//
//     // Set new type for these pages
//     MmAllocatePagesInLookupTable(PageLookupTableAddress, StartPageNumber, PagesNeeded, NewType);
// }
//
// PVOID MmAllocateHighestMemoryBelowAddress(SIZE_T MemorySize, PVOID DesiredAddress, TYPE_OF_MEMORY MemoryType)
// {
//     PFN_NUMBER        PagesNeeded;
//     PFN_NUMBER        FirstFreePageFromEnd;
//     PFN_NUMBER        DesiredAddressPageNumber;
//     PVOID    MemPointer;
//
//     if (MemorySize == 0)
//     {
//         WARN("MmAllocateHighestMemoryBelowAddress() called for 0 bytes. Returning NULL.\n");
//         UiMessageBoxCritical("Memory allocation failed: MmAllocateHighestMemoryBelowAddress() called for 0 bytes.");
//         return NULL;
//     }
//
//     // Find out how many blocks it will take to
//     // satisfy this allocation
//     PagesNeeded = ROUND_UP(MemorySize, MM_PAGE_SIZE) / MM_PAGE_SIZE;
//
//     // Get the page number for their desired address
//     DesiredAddressPageNumber = (ULONG_PTR)DesiredAddress / MM_PAGE_SIZE;
//
//     // If we don't have enough available mem
//     // then return NULL
//     if (FreePagesInLookupTable < PagesNeeded)
//     {
//         ERR("Memory allocation failed in MmAllocateHighestMemoryBelowAddress(). Not enough free memory to allocate %d bytes.\n", MemorySize);
//         UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     FirstFreePageFromEnd = MmFindAvailablePagesBeforePage(PageLookupTableAddress, TotalPagesInLookupTable, PagesNeeded, DesiredAddressPageNumber);
//
//     if (FirstFreePageFromEnd == 0)
//     {
//         ERR("Memory allocation failed in MmAllocateHighestMemoryBelowAddress(). Not enough free memory to allocate %d bytes.\n", MemorySize);
//         UiMessageBoxCritical("Memory allocation failed: out of memory.");
//         return NULL;
//     }
//
//     MmAllocatePagesInLookupTable(PageLookupTableAddress, FirstFreePageFromEnd, PagesNeeded, MemoryType);
//
//     FreePagesInLookupTable -= PagesNeeded;
//     MemPointer = (PVOID)((ULONG_PTR)FirstFreePageFromEnd * MM_PAGE_SIZE);
//
//     TRACE("Allocated %d bytes (%d pages) of memory starting at page %d.\n", MemorySize, PagesNeeded, FirstFreePageFromEnd);
//     TRACE("Memory allocation pointer: 0x%x\n", MemPointer);
//
//     // Update LoaderPagesSpanned count
//     if ((((ULONG_PTR)MemPointer + MemorySize) >> PAGE_SHIFT) > LoaderPagesSpanned)
//         LoaderPagesSpanned = (((ULONG_PTR)MemPointer + MemorySize) >> PAGE_SHIFT);
//
//     // Now return the pointer
//     return MemPointer;
// }
//
// VOID MmFreeMemory(PVOID MemoryPointer)
// {
// }
//
// #if DBG
//
// VOID DumpMemoryAllocMap(VOID)
// {
//     PFN_NUMBER    Idx;
//     PPAGE_LOOKUP_TABLE_ITEM        RealPageLookupTable = (PPAGE_LOOKUP_TABLE_ITEM)PageLookupTableAddress;
//
//     DbgPrint("----------- Memory Allocation Bitmap -----------\n");
//
//     for (Idx=0; Idx<TotalPagesInLookupTable; Idx++)
//     {
//         if ((Idx % 32) == 0)
//         {
//             DbgPrint("\n");
//             DbgPrint("%08x:\t", (Idx * MM_PAGE_SIZE));
//         }
//         else if ((Idx % 4) == 0)
//         {
//             DbgPrint(" ");
//         }
//
//         switch (RealPageLookupTable[Idx].PageAllocated)
//         {
//         case LoaderFree:
//             DbgPrint("*");
//             break;
//         case LoaderBad:
//             DbgPrint("-");
//             break;
//         case LoaderLoadedProgram:
//             DbgPrint("O");
//             break;
//         case LoaderFirmwareTemporary:
//             DbgPrint("T");
//             break;
//         case LoaderFirmwarePermanent:
//             DbgPrint("P");
//             break;
//         case LoaderOsloaderHeap:
//             DbgPrint("H");
//             break;
//         case LoaderOsloaderStack:
//             DbgPrint("S");
//             break;
//         case LoaderSystemCode:
//             DbgPrint("K");
//             break;
//         case LoaderHalCode:
//             DbgPrint("L");
//             break;
//         case LoaderBootDriver:
//             DbgPrint("B");
//             break;
//         case LoaderStartupPcrPage:
//             DbgPrint("G");
//             break;
//         case LoaderRegistryData:
//             DbgPrint("R");
//             break;
//         case LoaderMemoryData:
//             DbgPrint("M");
//             break;
//         case LoaderNlsData:
//             DbgPrint("N");
//             break;
//         case LoaderSpecialMemory:
//             DbgPrint("C");
//             break;
//         default:
//             DbgPrint("?");
//             break;
//         }
//     }
//
//     DbgPrint("\n");
// }
// #endif // DBG
//
// PPAGE_LOOKUP_TABLE_ITEM MmGetMemoryMap(PFN_NUMBER *NoEntries)
// {
//     PPAGE_LOOKUP_TABLE_ITEM        RealPageLookupTable = (PPAGE_LOOKUP_TABLE_ITEM)PageLookupTableAddress;
//
//     *NoEntries = TotalPagesInLookupTable;
//
//     return RealPageLookupTable;
// }
//
// PFN_NUMBER
// MmGetLoaderPagesSpanned(VOID)
// {
//     return LoaderPagesSpanned;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
