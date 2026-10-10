#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WINDOWS; // wlmemory.c: DBG_DEFAULT_CHANNEL(WINDOWS)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{MEMORY_ALLOCATION_DESCRIPTOR, PCSTR, PLOADER_PARAMETER_BLOCK, PMEMORY_ALLOCATION_DESCRIPTOR};
// freeldr_ddk_imports dropped (not in ddk): Type
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderBBTMemory as LoaderBBTMemory;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderBad as LoaderBad;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderBootDriver as LoaderBootDriver;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderConsoleInDriver as LoaderConsoleInDriver;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderConsoleOutDriver as LoaderConsoleOutDriver;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderErrorLogMemory as LoaderErrorLogMemory;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderExceptionBlock as LoaderExceptionBlock;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwarePermanent as LoaderFirmwarePermanent;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwareTemporary as LoaderFirmwareTemporary;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFree as LoaderFree;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderHALCachedMemory as LoaderHALCachedMemory;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderHalCode as LoaderHalCode;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderLargePageFiller as LoaderLargePageFiller;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderLoadedProgram as LoaderLoadedProgram;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderMemoryData as LoaderMemoryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderNlsData as LoaderNlsData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderOsloaderHeap as LoaderOsloaderHeap;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderOsloaderStack as LoaderOsloaderStack;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderRegistryData as LoaderRegistryData;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderReserve as LoaderReserve;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSpecialMemory as LoaderSpecialMemory;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupDpcStack as LoaderStartupDpcStack;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupKernelStack as LoaderStartupKernelStack;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPanicStack as LoaderStartupPanicStack;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPcrPage as LoaderStartupPcrPage;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderStartupPdrPage as LoaderStartupPdrPage;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSystemBlock as LoaderSystemBlock;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSystemCode as LoaderSystemCode;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderXIPRom as LoaderXIPRom;

extern "C" {
    fn FrLdrHeapCleanupAll();
    fn MmSetMemoryType(BaseAddress: *mut core::ffi::c_void, SizeInPages: SIZE_T, MemoryType: u32);
    fn MmAllocateMemoryWithType(MemorySize: usize, MemoryType: u32) -> *mut core::ffi::c_void;
    fn MmGetMemoryMap(NoEntries: *mut PFN_NUMBER) -> *mut PAGE_LOOKUP_TABLE_ITEM;
    fn UiMessageBox(Text: *const i8);
    fn MempSetupPaging(BasePage: PFN_NUMBER, PageCount: PFN_NUMBER, BootPageTable: u8) -> u8;
    fn MmGetBiosMemoryMap(BiosMemoryMap: *mut *mut FREELDR_MEMORY_DESCRIPTOR) -> u32;
    fn MmGetHighestPhysicalPage() -> PFN_NUMBER;
    fn MmGetLoaderPagesSpanned() -> PFN_NUMBER;
    fn WinLdrpDumpMemoryDescriptors(LoaderBlock: PLOADER_PARAMETER_BLOCK);
    fn List_PaToVa(ListHead: *mut LIST_ENTRY);
    static mut TotalNLSSize: u32;
    static mut NlsDataBase: *mut core::ffi::c_void;
}

// wlmemory.c file-scope globals (this TU is the definer).
#[no_mangle]
pub static mut Mad: *mut MEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut MadCount: u32 = 0;
// wlmemory.c: #define MAX_MAD_COUNT 200
const MAX_MAD_COUNT: usize = 200;
// wlmemory.c: static const PCSTR MemTypeDesc[]
static MemTypeDesc: [&[u8]; 24] = [
    b"ExceptionBlock    \0",
    b"SystemBlock       \0",
    b"Free              \0",
    b"Bad               \0",
    b"LoadedProgram     \0",
    b"FirmwareTemporary \0",
    b"FirmwarePermanent \0",
    b"OsloaderHeap      \0",
    b"OsloaderStack     \0",
    b"SystemCode        \0",
    b"HalCode           \0",
    b"BootDriver        \0",
    b"ConsoleInDriver   \0",
    b"ConsoleOutDriver  \0",
    b"StartupDpcStack   \0",
    b"StartupKernelStack\0",
    b"StartupPanicStack \0",
    b"StartupPcrPage    \0",
    b"StartupPdrPage    \0",
    b"RegistryData      \0",
    b"MemoryData        \0",
    b"NlsData           \0",
    b"SpecialMemory     \0",
    b"BBTMemory         \0",
];

unsafe fn MempAddMemoryBlock(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    BasePage: PFN_NUMBER,
    PageCount: PFN_NUMBER,
    Type: u32,
) {

    let mut PageCount = PageCount;

    TRACE!("MempAddMemoryBlock(BasePage=0x%lx, PageCount=0x%lx, Type=%ld)\n", (BasePage) as usize, (PageCount) as usize, (Type) as usize);

    if BasePage >= MM_MAX_PAGE {
        return;
    }

    if BasePage + PageCount > MM_MAX_PAGE {
        PageCount = MM_MAX_PAGE - BasePage;
    }

    if MadCount as usize >= MAX_MAD_COUNT {
        ERR!("Error: no MAD slots left!\n");
        return;
    }

    // MAD BasePage/PageCount are PFN_NUMBER (u32 on i386, u64 on amd64);
    // `as _` coerces to the field's per-arch width (identity on i386).
    (*Mad.add(MadCount as usize)).BasePage = (BasePage) as _ ;
    (*Mad.add(MadCount as usize)).PageCount = (PageCount) as _ ;
    (*Mad.add(MadCount as usize)).MemoryType = Type;

    WinLdrInsertDescriptor(LoaderBlock, Mad.add(MadCount as usize));
    MadCount += 1;
}

unsafe fn MempSetupPagingForRegion(
    BasePage: PFN_NUMBER,
    PageCount: PFN_NUMBER,
    Type: u32)
{
    let mut Status: u8 = 1u8;

    TRACE!("MempSetupPagingForRegion(BasePage=0x%lx, PageCount=0x%lx, Type=%ld)\n", (BasePage) as usize, (PageCount) as usize, (Type) as usize);

    if BasePage + PageCount > MmGetLoaderPagesSpanned() { return; }

    match Type {
        LoaderLoadedProgram | LoaderOsloaderStack | LoaderFirmwareTemporary => {
            Status = MempSetupPaging(BasePage, PageCount, 0u8);
        }

        LoaderExceptionBlock | LoaderSystemBlock | LoaderFirmwarePermanent |
        LoaderSystemCode | LoaderHalCode | LoaderBootDriver |
        LoaderConsoleInDriver | LoaderConsoleOutDriver | LoaderStartupDpcStack |
        LoaderStartupKernelStack | LoaderStartupPanicStack | LoaderStartupPcrPage |
        LoaderStartupPdrPage | LoaderRegistryData | LoaderMemoryData |
        LoaderNlsData | LoaderXIPRom | LoaderOsloaderHeap => {
            Status = MempSetupPaging(BasePage, PageCount, 1u8);
        }

        LoaderFree | LoaderBad => {}

        LoaderSpecialMemory | LoaderHALCachedMemory | LoaderBBTMemory => {}

        LoaderReserve | LoaderLargePageFiller | LoaderErrorLogMemory => {}

        _ => {}
    }

    if Status == 0u8 {
        ERR!("Error during MempSetupPaging\n");
    }
}

#[no_mangle]
pub unsafe extern "C" fn WinLdrSetupMemoryLayout(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> u8 {

    // PAGE_LOOKUP_TABLE_ITEM / FREELDR_MEMORY_DESCRIPTOR / LIST_ENTRY
    // come from the keystone + base (the translator's local re-defs
    // shadowed them and broke the Mm* extern signatures — removed).

    const MM_PAGE_SHIFT: u32 = 12;
    const MM_PAGE_SIZE: u32 = 4096;
    // MAX_MAD_COUNT: use the file-level const (=200, matches C); the
    // translator's wrong-valued local shadow is removed.
    const LoaderFree: u32 = 2; // header value; the hand copy said 0

    let mut i: PFN_NUMBER;
    let _PagesCount: PFN_NUMBER;
    let MemoryMapSizeInPages: PFN_NUMBER;
    let mut NoEntries: PFN_NUMBER = 0;
    let mut LastPageIndex: PFN_NUMBER;
    let MemoryMapStartPage: PFN_NUMBER;
    let MemoryMap: *mut PAGE_LOOKUP_TABLE_ITEM;
    let mut LastPageType: u32;
    let _Status: u8;

    FrLdrHeapCleanupAll();

    if TotalNLSSize > 0 && !NlsDataBase.is_null() {
        MmSetMemoryType(NlsDataBase, (TotalNLSSize * MM_PAGE_SIZE) as SIZE_T, LoaderNlsData);
    }

    Mad = MmAllocateMemoryWithType(
        core::mem::size_of::<MEMORY_ALLOCATION_DESCRIPTOR>() * MAX_MAD_COUNT,
        LoaderMemoryData,
    ) as *mut MEMORY_ALLOCATION_DESCRIPTOR;

    MemoryMap = MmGetMemoryMap(core::ptr::addr_of_mut!(NoEntries));
    if MemoryMap.is_null() {
        UiMessageBox(b"Can not retrieve the current memory map.\0".as_ptr() as *const i8);
        return 0u8;
    }

    MemoryMapStartPage = (MemoryMap as usize >> MM_PAGE_SHIFT) as PFN_NUMBER;
    MemoryMapSizeInPages = ((NoEntries as usize * core::mem::size_of::<PAGE_LOOKUP_TABLE_ITEM>() + MM_PAGE_SIZE as usize - 1) / MM_PAGE_SIZE as usize) as PFN_NUMBER;

    TRACE!("Got memory map with %d entries\n", (NoEntries) as usize);

    let Status = MempSetupPaging(0, 1, 0u8);
    if Status == 0u8 {
        ERR!("Error during MempSetupPaging of first page\n");
        return 0u8;
    }

    LastPageIndex = 1;
    LastPageType = (*MemoryMap.offset(1)).PageAllocated;
    i = 2;
    while i < NoEntries {
        if (*MemoryMap.offset(i as isize)).PageAllocated != LastPageType || i == NoEntries - 1 {
            MempSetupPagingForRegion(LastPageIndex, i - LastPageIndex, LastPageType);
            LastPageIndex = i;
            LastPageType = (*MemoryMap.offset(i as isize)).PageAllocated;
        }
        i += 1;
    }

    let mut PagesCount = 1;
    LastPageIndex = 0;
    LastPageType = (*MemoryMap.offset(0)).PageAllocated;
    i = 1;
    while i < NoEntries {
        if i >= MemoryMapStartPage && i < (MemoryMapStartPage + MemoryMapSizeInPages) {
            (*MemoryMap.offset(i as isize)).PageAllocated = LoaderFree;
        }

        if (*MemoryMap.offset(i as isize)).PageAllocated == LastPageType && i != NoEntries - 1 {
            PagesCount += 1;
        } else {
            MempAddMemoryBlock(LoaderBlock, LastPageIndex, PagesCount, LastPageType);

            LastPageIndex = i;
            LastPageType = (*MemoryMap.offset(i as isize)).PageAllocated;
            PagesCount = 1;
        }
        i += 1;
    }

    // C locals; renamed (keystone exports globals of the same name).
    let mut BiosMemoryMap_l: *mut FREELDR_MEMORY_DESCRIPTOR = core::ptr::null_mut();
    let BiosMemoryMapEntryCount_l: u32;

    BiosMemoryMapEntryCount_l = MmGetBiosMemoryMap(core::ptr::addr_of_mut!(BiosMemoryMap_l));

    i = 0;
    while i < BiosMemoryMapEntryCount_l as PFN_NUMBER {
        if (*BiosMemoryMap_l.offset(i as isize)).BasePage > MmGetHighestPhysicalPage() {
            MempAddMemoryBlock(
                LoaderBlock,
                (*BiosMemoryMap_l.offset(i as isize)).BasePage,
                (*BiosMemoryMap_l.offset(i as isize)).PageCount,
                (*BiosMemoryMap_l.offset(i as isize)).MemoryType,
            );
        }
        i += 1;
    }

    TRACE!("MadCount: %d\n", (MadCount) as usize);

    WinLdrpDumpMemoryDescriptors(LoaderBlock);

    // UEFILDR can be above the 2GB-ish range, we don't want to map the whole area
    #[cfg(feature = "uefi")]
    {
        extern "C" {
            static mut OsLoaderBase: PVOID;
            static mut OsLoaderSize: SIZE_T;
        }
        MempSetupPaging(
            (OsLoaderBase as usize / MM_PAGE_SIZE as usize) as PFN_NUMBER,
            (OsLoaderSize / MM_PAGE_SIZE as usize) as PFN_NUMBER,
            0u8,
        );
    }

    TRACE!("Memory Descriptor List prepared, printing PDE\n");
    List_PaToVa(core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead));

    1u8
}

unsafe fn WinLdrInsertDescriptor(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    mut NewDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR,
) {
    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*LoaderBlock).MemoryDescriptorListHead);
    let mut PreviousEntry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut PreviousDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();
    let mut NextDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();

    // C indexes MemTypeDesc[MemoryType] unconditionally (OOB just reads
    // garbage and prints it via %s). In Rust a slice index is bounds-checked
    // and panics — and the arm64 panic handler spins silently, which looked
    // exactly like a WinLdrSetupMemoryLayout loop2 boot hang. Guard the index.
    let mt_idx = (*NewDescriptor).MemoryType as usize;
    let mt_desc: *const u8 = if mt_idx < MemTypeDesc.len() {
        MemTypeDesc[mt_idx].as_ptr()
    } else {
        b"Unknown           \0".as_ptr()
    };
    TRACE!("BP=0x%X PC=0x%X %s\n", ((*NewDescriptor).BasePage) as usize, ((*NewDescriptor).PageCount) as usize, (mt_desc) as usize);

    PreviousEntry = ListHead;
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        NextDescriptor = (NextEntry as *mut u8).offset(
            -(core::mem::offset_of!(MEMORY_ALLOCATION_DESCRIPTOR, ListEntry) as isize)
        ) as PMEMORY_ALLOCATION_DESCRIPTOR;

        if (*NewDescriptor).BasePage < (*NextDescriptor).BasePage {
            break;
        }

        PreviousEntry = NextEntry;
        PreviousDescriptor = NextDescriptor;
        NextEntry = (*NextEntry).Flink;
    }

    if (*NewDescriptor).MemoryType != LoaderFree {
        InsertHeadList(PreviousEntry, core::ptr::addr_of_mut!((*NewDescriptor).ListEntry));
    } else {
        if (PreviousEntry != ListHead)
            && ((*PreviousDescriptor).MemoryType == LoaderFree)
            && (((*PreviousDescriptor).BasePage + (*PreviousDescriptor).PageCount)
                == (*NewDescriptor).BasePage)
        {
            (*PreviousDescriptor).PageCount += (*NewDescriptor).PageCount;
            NewDescriptor = PreviousDescriptor;
        } else {
            InsertHeadList(PreviousEntry, core::ptr::addr_of_mut!((*NewDescriptor).ListEntry));
        }

        if (NextEntry != ListHead)
            && ((*NextDescriptor).MemoryType == LoaderFree)
            && (((*NewDescriptor).BasePage + (*NewDescriptor).PageCount)
                == (*NextDescriptor).BasePage)
        {
            (*NewDescriptor).PageCount += (*NextDescriptor).PageCount;
            RemoveEntryList(core::ptr::addr_of_mut!((*NextDescriptor).ListEntry));
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         EFI Windows Loader
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/windows/wlmemory.c
//  * PURPOSE:         Memory related routines
//  * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <freeldr.h>
// #include "winldr.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WINDOWS);
//
// static const PCSTR MemTypeDesc[] = {
//     "ExceptionBlock    ", // ?
//     "SystemBlock       ", // ?
//     "Free              ",
//     "Bad               ", // used
//     "LoadedProgram     ", // == Free
//     "FirmwareTemporary ", // == Free
//     "FirmwarePermanent ", // == Bad
//     "OsloaderHeap      ", // used
//     "OsloaderStack     ", // == Free
//     "SystemCode        ",
//     "HalCode           ",
//     "BootDriver        ", // not used
//     "ConsoleInDriver   ", // ?
//     "ConsoleOutDriver  ", // ?
//     "StartupDpcStack   ", // ?
//     "StartupKernelStack", // ?
//     "StartupPanicStack ", // ?
//     "StartupPcrPage    ", // ?
//     "StartupPdrPage    ", // ?
//     "RegistryData      ", // used
//     "MemoryData        ", // not used
//     "NlsData           ", // used
//     "SpecialMemory     ", // == Bad
//     "BBTMemory         " // == Bad
//     };
//
// static VOID
// WinLdrInsertDescriptor(IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//                        IN PMEMORY_ALLOCATION_DESCRIPTOR NewDescriptor);
//
// /* GLOBALS ***************************************************************/
//
// MEMORY_ALLOCATION_DESCRIPTOR *Mad;
// ULONG MadCount = 0;
// /* 200 MADs fit into 1 page, that should really be enough! */
// #define MAX_MAD_COUNT 200
//
// /* FUNCTIONS **************************************************************/
//
// VOID
// MempAddMemoryBlock(IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//                    PFN_NUMBER BasePage,
//                    PFN_NUMBER PageCount,
//                    ULONG Type)
// {
//     TRACE("MempAddMemoryBlock(BasePage=0x%lx, PageCount=0x%lx, Type=%ld)\n", (//           BasePage) as usize, (PageCount) as usize, (Type) as usize);
//
//     /* Check for memory block after 4GB - we don't support it yet
//        Note: Even last page before 4GB limit is not supported */
//     if (BasePage >= MM_MAX_PAGE)
//     {
//         /* Just skip this, without even adding to MAD list */
//         return;
//     }
//
//     /* Check if last page is after 4GB limit and shorten this block if needed */
//     if (BasePage + PageCount > MM_MAX_PAGE)
//     {
//         /* Shorten this block */
//         PageCount = MM_MAX_PAGE - BasePage;
//     }
//
//     /* Check if we have slots left */
//     if (MadCount >= MAX_MAD_COUNT)
//     {
//         ERR("Error: no MAD slots left!\n");
//         return;
//     }
//
//     /* Set Base page, page count and type */
//     Mad[MadCount].BasePage = BasePage;
//     Mad[MadCount].PageCount = PageCount;
//     Mad[MadCount].MemoryType = Type;
//
//     /* Add descriptor */
//     WinLdrInsertDescriptor(LoaderBlock, &Mad[MadCount]);
//     MadCount++;
// }
//
// VOID
// MempSetupPagingForRegion(
//     PFN_NUMBER BasePage,
//     PFN_NUMBER PageCount,
//     ULONG Type)
// {
//     BOOLEAN Status = TRUE;
//
//     TRACE("MempSetupPagingForRegion(BasePage=0x%lx, PageCount=0x%lx, Type=%ld)\n", (//           BasePage) as usize, (PageCount) as usize, (Type) as usize);
//
//     /* Make sure we don't map too high */
//     if (BasePage + PageCount > MmGetLoaderPagesSpanned()) return;
//
//     switch (Type)
//     {
//         /* Pages used by the loader */
//         case LoaderLoadedProgram:
//         case LoaderOsloaderStack:
//         case LoaderFirmwareTemporary:
//             /* Map these pages into user mode */
//             Status = MempSetupPaging(BasePage, PageCount, FALSE);
//             break;
//
//         /* Pages used by the kernel */
//         case LoaderExceptionBlock:
//         case LoaderSystemBlock:
//         case LoaderFirmwarePermanent:
//         case LoaderSystemCode:
//         case LoaderHalCode:
//         case LoaderBootDriver:
//         case LoaderConsoleInDriver:
//         case LoaderConsoleOutDriver:
//         case LoaderStartupDpcStack:
//         case LoaderStartupKernelStack:
//         case LoaderStartupPanicStack:
//         case LoaderStartupPcrPage:
//         case LoaderStartupPdrPage:
//         case LoaderRegistryData:
//         case LoaderMemoryData:
//         case LoaderNlsData:
//         case LoaderXIPRom:
//         case LoaderOsloaderHeap: // FIXME
//             /* Map these pages into kernel mode */
//             Status = MempSetupPaging(BasePage, PageCount, TRUE);
//             break;
//
//         /* Pages not in use */
//         case LoaderFree:
//         case LoaderBad:
//             break;
//
//         /* Invisible to kernel */
//         case LoaderSpecialMemory:
//         case LoaderHALCachedMemory:
//         case LoaderBBTMemory:
//             break;
//
//         // FIXME: not known (not used anyway)
//         case LoaderReserve:
//         case LoaderLargePageFiller:
//         case LoaderErrorLogMemory:
//             break;
//     }
//
//     if (!Status)
//     {
//         ERR("Error during MempSetupPaging\n");
//     }
// }
//
// #ifdef _M_ARM
// #define PKTSS PVOID
// #endif
//
// BOOLEAN
// WinLdrSetupMemoryLayout(IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PFN_NUMBER i, PagesCount, MemoryMapSizeInPages, NoEntries;
//     PFN_NUMBER LastPageIndex, MemoryMapStartPage;
//     PPAGE_LOOKUP_TABLE_ITEM MemoryMap;
//     ULONG LastPageType;
//     //PKTSS Tss;
//     BOOLEAN Status;
//
//     /* Cleanup heap */
//     FrLdrHeapCleanupAll();
//
//     /* Re-mark NLS data pages that may have been clobbered by heap cleanup */
//     {
//         extern ULONG TotalNLSSize;
//         extern PVOID NlsDataBase;
//         if (TotalNLSSize > 0 && NlsDataBase != NULL)
//         {
//             MmSetMemoryType(NlsDataBase, TotalNLSSize * MM_PAGE_SIZE, LoaderNlsData);
//         }
//     }
//
//     //
//     // Creating a suitable memory map for Windows can be tricky, so let's
//     // give a few advices:
//     // 1) One must not map the whole available memory pages to PDE!
//     //    Map only what's needed - 16Mb, 24Mb, 32Mb max I think,
//     //    thus occupying 4, 6 or 8 PDE entries for identical mapping,
//     //    the same quantity for KSEG0_BASE mapping, one more entry for
//     //    hyperspace and one more entry for HAL physical pages mapping.
//     // 2) Memory descriptors must map *the whole* physical memory
//     //    showing any memory above 16/24/32 as FirmwareTemporary
//     //
//     // 3) Overall memory blocks count must not exceed 30 (?? why?)
//     //
//
//     //
//     // During MmInitMachineDependent, the kernel zeroes PDE at the following address
//     // 0xC0300000 - 0xC03007FC
//     //
//     // Then it finds the best place for non-paged pool:
//     // StartPde C0300F70, EndPde C0300FF8, NumberOfPages C13, NextPhysPage 3AD
//     //
//
//     // Allocate memory for memory allocation descriptors
//     Mad = MmAllocateMemoryWithType(sizeof(MEMORY_ALLOCATION_DESCRIPTOR) * MAX_MAD_COUNT,
//                                    LoaderMemoryData);
//
//     // Setup an entry for each descriptor
//     MemoryMap = MmGetMemoryMap(&NoEntries);
//     if (MemoryMap == NULL)
//     {
//         UiMessageBox("Can not retrieve the current memory map.");
//         return FALSE;
//     }
//
//     // Calculate parameters of the memory map
//     MemoryMapStartPage = (ULONG_PTR)MemoryMap >> MM_PAGE_SHIFT;
//     MemoryMapSizeInPages = (NoEntries * sizeof(PAGE_LOOKUP_TABLE_ITEM) + MM_PAGE_SIZE - 1) / MM_PAGE_SIZE;
//
//     TRACE("Got memory map with %d entries\n", (NoEntries) as usize);
//
//     // Always map first page of memory
//     Status = MempSetupPaging(0, 1, FALSE);
//     if (!Status)
//     {
//         ERR("Error during MempSetupPaging of first page\n");
//         return FALSE;
//     }
//
//     /* Before creating the map, we need to map pages to kernel mode */
//     LastPageIndex = 1;
//     LastPageType = MemoryMap[1].PageAllocated;
//     for (i = 2; i < NoEntries; i++)
//     {
//         if ((MemoryMap[i].PageAllocated != LastPageType) ||
//             (i == NoEntries - 1))
//         {
//             MempSetupPagingForRegion(LastPageIndex, i - LastPageIndex, LastPageType);
//             LastPageIndex = i;
//             LastPageType = MemoryMap[i].PageAllocated;
//         }
//     }
//
//     // Construct a good memory map from what we've got,
//     // but mark entries which the memory allocation bitmap takes
//     // as free entries (this is done in order to have the ability
//     // to place mem alloc bitmap outside lower 16Mb zone)
//     PagesCount = 1;
//     LastPageIndex = 0;
//     LastPageType = MemoryMap[0].PageAllocated;
//     for (i = 1; i < NoEntries; i++)
//     {
//         // Check if its memory map itself
//         if (i >= MemoryMapStartPage &&
//             i < (MemoryMapStartPage+MemoryMapSizeInPages))
//         {
//             // Exclude it if current page belongs to the memory map
//             MemoryMap[i].PageAllocated = LoaderFree;
//         }
//
//         // Process entry
//         if (MemoryMap[i].PageAllocated == LastPageType &&
//             (i != NoEntries-1) )
//         {
//             PagesCount++;
//         }
//         else
//         {
//             // Add the resulting region
//             MempAddMemoryBlock(LoaderBlock, LastPageIndex, PagesCount, LastPageType);
//
//             // Reset our counter vars
//             LastPageIndex = i;
//             LastPageType = MemoryMap[i].PageAllocated;
//             PagesCount = 1;
//         }
//     }
//
//     // TEMP, DEBUG!
//     // adding special reserved memory zones for vmware workstation
// #if 0
//     {
//         Mad[MadCount].BasePage = 0xfec00;
//         Mad[MadCount].PageCount = 0x10;
//         Mad[MadCount].MemoryType = LoaderSpecialMemory;
//         WinLdrInsertDescriptor(LoaderBlock, &Mad[MadCount]);
//         MadCount++;
//
//         Mad[MadCount].BasePage = 0xfee00;
//         Mad[MadCount].PageCount = 0x1;
//         Mad[MadCount].MemoryType = LoaderSpecialMemory;
//         WinLdrInsertDescriptor(LoaderBlock, &Mad[MadCount]);
//         MadCount++;
//
//         Mad[MadCount].BasePage = 0xfffe0;
//         Mad[MadCount].PageCount = 0x20;
//         Mad[MadCount].MemoryType = LoaderSpecialMemory;
//         WinLdrInsertDescriptor(LoaderBlock, &Mad[MadCount]);
//         MadCount++;
//     }
// #endif
//     PFREELDR_MEMORY_DESCRIPTOR BiosMemoryMap;
//     ULONG BiosMemoryMapEntryCount;
//
//     BiosMemoryMapEntryCount = MmGetBiosMemoryMap(&BiosMemoryMap);
//
//     /* Now we need to add high descriptors from the bios memory map */
//     for (i = 0; i < BiosMemoryMapEntryCount; i++)
//     {
//         /* Check if its higher than the lookup table */
//         if (BiosMemoryMap->BasePage > MmGetHighestPhysicalPage())
//         {
//             /* Copy this descriptor */
//             MempAddMemoryBlock(LoaderBlock,
//                                BiosMemoryMap->BasePage,
//                                BiosMemoryMap->PageCount,
//                                BiosMemoryMap->MemoryType);
//         }
//     }
//
//     TRACE("MadCount: %d\n", (MadCount) as usize);
//
//     WinLdrpDumpMemoryDescriptors(LoaderBlock); //FIXME: Delete!
//
//     // Map our loader image, so we can continue running
//     /*Status = MempSetupPaging(OsLoaderBase >> MM_PAGE_SHIFT, OsLoaderSize >> MM_PAGE_SHIFT);
//     if (!Status)
//     {
//         UiMessageBox("Error during MempSetupPaging.");
//         return;
//     }*/
//
//     // Fill the memory descriptor list and
//     //PrepareMemoryDescriptorList();
//     TRACE("Memory Descriptor List prepared, printing PDE\n");
//     List_PaToVa(&LoaderBlock->MemoryDescriptorListHead);
//
// #if DBG
//     MempDump();
// #endif
//
//     return TRUE;
// }
//
// // Two special things this func does: it sorts descriptors,
// // and it merges free ones
// static VOID
// WinLdrInsertDescriptor(IN OUT PLOADER_PARAMETER_BLOCK LoaderBlock,
//                        IN PMEMORY_ALLOCATION_DESCRIPTOR NewDescriptor)
// {
//     PLIST_ENTRY ListHead = &LoaderBlock->MemoryDescriptorListHead;
//     PLIST_ENTRY PreviousEntry, NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR PreviousDescriptor = NULL, NextDescriptor = NULL;
//
//     TRACE("BP=0x%X PC=0x%X %s\n", (NewDescriptor->BasePage) as usize, (//         NewDescriptor->PageCount) as usize, (MemTypeDesc[NewDescriptor->MemoryType]) as usize);
//
//     /* Find a place where to insert the new descriptor to */
//     PreviousEntry = ListHead;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         NextDescriptor = CONTAINING_RECORD(NextEntry,
//             MEMORY_ALLOCATION_DESCRIPTOR,
//             ListEntry);
//         if (NewDescriptor->BasePage < NextDescriptor->BasePage)
//             break;
//
//         PreviousEntry = NextEntry;
//         PreviousDescriptor = NextDescriptor;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Don't forget about merging free areas */
//     if (NewDescriptor->MemoryType != LoaderFree)
//     {
//         /* Just insert, nothing to merge */
//         InsertHeadList(PreviousEntry, &NewDescriptor->ListEntry);
//     }
//     else
//     {
//         /* Previous block also free? */
//         if ((PreviousEntry != ListHead) && (PreviousDescriptor->MemoryType == LoaderFree) &&
//             ((PreviousDescriptor->BasePage + PreviousDescriptor->PageCount) ==
//             NewDescriptor->BasePage))
//         {
//             /* Just enlarge previous descriptor's PageCount */
//             PreviousDescriptor->PageCount += NewDescriptor->PageCount;
//             NewDescriptor = PreviousDescriptor;
//         }
//         else
//         {
//             /* Nope, just insert */
//             InsertHeadList(PreviousEntry, &NewDescriptor->ListEntry);
//         }
//
//         /* Next block is free ?*/
//         if ((NextEntry != ListHead) &&
//             (NextDescriptor->MemoryType == LoaderFree) &&
//             ((NewDescriptor->BasePage + NewDescriptor->PageCount) == NextDescriptor->BasePage))
//         {
//             /* Enlarge next descriptor's PageCount */
//             NewDescriptor->PageCount += NextDescriptor->PageCount;
//             RemoveEntryList(&NextDescriptor->ListEntry);
//         }
//     }
//
//     return;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
