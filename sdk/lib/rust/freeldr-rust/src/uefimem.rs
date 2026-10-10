use ntkrnl_sys::base::*;
use crate::frldr::*;
// UINT32 comes from crate::frldr::* (keystone EFI surface).
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderBad as LoaderBad;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwarePermanent as LoaderFirmwarePermanent;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFirmwareTemporary as LoaderFirmwareTemporary;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderFree as LoaderFree;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderLoadedProgram as LoaderLoadedProgram;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderReserve as LoaderReserve;
pub use ntkrnl_sys::ddk::_TYPE_OF_MEMORY_LoaderSpecialMemory as LoaderSpecialMemory;

extern "C" {
    fn _exituefi();
    fn TRACE(fmt: *const i8, ...);
    fn FrLdrBugCheckWithMessage(code: u32, file: *const i8, line: u32, fmt: *const i8, ...);
    // cross-module freeldr fn (FLDRAPI empty → cdecl).
    fn AddMemoryDescriptor(
        List: PFREELDR_MEMORY_DESCRIPTOR,
        MaxCount: ULONG,
        BasePage: PFN_NUMBER,
        PageCount: PFN_NUMBER,
        MemoryType: TYPE_OF_MEMORY,
    ) -> ULONG;
    // cross-module data from uefildr.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
    static mut GlobalImageHandle: EFI_HANDLE;
}

// debug.h enum _FRLDR_BUGCHECK_CODES (UEFIBOOT build → index 5, after
// ASSERT_FAILURE; upstream b0bd092c0d4).
const EXIT_BOOTSERVICES_FAILURE: u32 = 5;
const UNUSED_MAX_DESCRIPTOR_COUNT: ULONG = 10000;

// uefimem.c NEXT_MEMORY_DESCRIPTOR(d, sz) — byte-step a descriptor pointer.
#[inline]
unsafe fn NEXT_MEMORY_DESCRIPTOR(
    Descriptor: *mut EFI_MEMORY_DESCRIPTOR,
    DescriptorSize: UINTN,
) -> *mut EFI_MEMORY_DESCRIPTOR {
    (Descriptor as *mut u8).add(DescriptorSize as usize) as *mut EFI_MEMORY_DESCRIPTOR
}

/// Debug print helper: writes a UTF-16 null-terminated string to UEFI ConOut.
/// Only usable BEFORE ExitBootServices. Caller passes a pointer to the wide
/// literal (e.g. via `crate::wide!("...\0")`).
///
/// Kept available (not stripped) because UEFI freeldr has no equivalent to
/// debugcon — when something hangs before the kernel hand-off, sprinkling
/// these is the only way to bisect. Linker-DCE'd when unused.
#[cfg(feature = "uefi")]
#[inline(always)]
#[allow(dead_code)]
pub unsafe fn uefi_dbg_str(s: *const u16) {
    use crate::uefildr::EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
    if !GlobalSystemTable.is_null() {
        let conout = (*GlobalSystemTable).ConOut as *mut EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
        if !conout.is_null() {
            if let Some(out) = (*conout).OutputString {
                out(conout, s as *mut u16);
            }
        }
    }
}

// uefimem.c file-scope globals (external linkage — this TU is the definer).
#[no_mangle]
pub static mut EfiMemoryMap: *mut EFI_MEMORY_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut FreeldrDescCount: UINT32 = 0;
#[no_mangle]
pub static mut OsLoaderBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut OsLoaderSize: SIZE_T = 0;
#[no_mangle]
pub static mut PublicBootHandle: EFI_HANDLE = core::ptr::null_mut();

unsafe fn PUEFI_LoadMemoryMap(
    LocMapKey: *mut UINTN,
    LocMapSize: *mut UINTN,
    LocDescriptorSize: *mut UINTN,
    LocDescriptorVersion: *mut UINT32)
{
    let mut Status: EFI_STATUS;
    let mut AllocationSize: UINTN = 0;
    let mut Count: ULONG = 0;

    Status = ((*(*GlobalSystemTable).BootServices).GetMemoryMap).unwrap()(
        LocMapSize,
        EfiMemoryMap,
        LocMapKey,
        LocDescriptorSize,
        LocDescriptorVersion);

    while Status != EFI_SUCCESS {
        if !EfiMemoryMap.is_null() {
            ((*(*GlobalSystemTable).BootServices).FreePool).unwrap()(EfiMemoryMap as *mut core::ffi::c_void);
        }

        AllocationSize = *LocMapSize + (*LocDescriptorSize * (Count as UINTN));
        ((*(*GlobalSystemTable).BootServices).AllocatePool).unwrap()(
            EfiLoaderData,
            AllocationSize,
            &mut EfiMemoryMap as *mut _ as *mut *mut VOID);
        Status = ((*(*GlobalSystemTable).BootServices).GetMemoryMap).unwrap()(
            LocMapSize,
            EfiMemoryMap,
            LocMapKey,
            LocDescriptorSize,
            LocDescriptorVersion);
        Count += 1;
    }
}

unsafe fn UefiSetMemory(
    MemoryMap: PFREELDR_MEMORY_DESCRIPTOR,
    BaseAddress: ULONG_PTR,
    SizeInPages: PFN_COUNT,
    MemoryType: TYPE_OF_MEMORY,
) {
    let BasePage: ULONG_PTR;
    let PageCount: PFN_COUNT;

    BasePage = BaseAddress / ((EFI_PAGE_SIZE) as usize);
    PageCount = SizeInPages;

    FreeldrDescCount = AddMemoryDescriptor(
        MemoryMap,
        UNUSED_MAX_DESCRIPTOR_COUNT,
        BasePage,
        (PageCount) as usize ,
        MemoryType,
    );
}

unsafe fn UefiConvertToFreeldrDesc(EfiMemoryType: EFI_MEMORY_TYPE) -> TYPE_OF_MEMORY {
    match EfiMemoryType {
        EfiReservedMemoryType => LoaderReserve,
        EfiLoaderCode => LoaderLoadedProgram,
        EfiLoaderData => LoaderLoadedProgram,
        EfiBootServicesCode => LoaderFirmwareTemporary,
        EfiBootServicesData => LoaderFirmwareTemporary,
        EfiRuntimeServicesCode => LoaderFirmwarePermanent,
        EfiRuntimeServicesData => LoaderFirmwarePermanent,
        EfiConventionalMemory => LoaderFree,
        EfiUnusableMemory => LoaderBad,
        // Both ACPI kinds are LoaderSpecialMemory (upstream e1853c66214). Reclaim memory was
        // LoaderFirmwareTemporary, which MiIsMemoryTypeFree counts as FREE: the pages holding
        // the RSDT/XSDT/DSDT joined the PFN free list before acpi.sys mapped them.
        EfiACPIReclaimMemory => LoaderSpecialMemory,
        EfiACPIMemoryNVS => LoaderSpecialMemory,
        EfiMemoryMappedIO => LoaderReserve,
        EfiMemoryMappedIOPortSpace => LoaderReserve,
        _ => LoaderReserve,
    }
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiMemGetMemoryMap(MemoryMapSize: *mut ULONG) -> PFREELDR_MEMORY_DESCRIPTOR {
    let mut LoadedImage: *mut EFI_LOADED_IMAGE_PROTOCOL = core::ptr::null_mut();
    let mut DescriptorVersion: UINT32 = 0;
    let FreeldrMemMapSize: SIZE_T;
    let mut DescriptorSize: UINTN = 0;
    let _Status: EFI_STATUS;
    let mut MapSize: UINTN = 0;
    let mut MapKey: UINTN = 0;
    let _Index: UINT32;

    let EfiLoadedImageProtocol: EFI_GUID = EFI_LOADED_IMAGE_PROTOCOL_GUID;
    let mut FreeldrMem: PFREELDR_MEMORY_DESCRIPTOR = core::ptr::null_mut();
    let mut MapEntry: *mut EFI_MEMORY_DESCRIPTOR = core::ptr::null_mut();
    let EntryCount: UINT32;
    FreeldrDescCount = 0;

    let status_handle = ((*(*GlobalSystemTable).BootServices).HandleProtocol).unwrap()(
        GlobalImageHandle,
        core::ptr::addr_of!(EfiLoadedImageProtocol) as *mut EFI_GUID,
        core::ptr::addr_of_mut!(LoadedImage) as *mut *mut core::ffi::c_void,
    );
    if status_handle != EFI_SUCCESS {
        TRACE(b"Failed to find LoadedImageHandle with status: %d\n\0".as_ptr() as PCHAR, (status_handle) as usize);
        UiMessageBoxCritical(b"Unable to initialize memory manager.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }
    OsLoaderBase = (*LoadedImage).ImageBase;
    OsLoaderSize = ((*LoadedImage).ImageSize) as usize ;
    PublicBootHandle = (*LoadedImage).DeviceHandle;
    EfiMemoryMap = core::ptr::null_mut();

    TRACE(b"UefiMemGetMemoryMap: Gather memory map\n\0".as_ptr() as PCHAR);
    PUEFI_LoadMemoryMap(
        core::ptr::addr_of_mut!(MapKey),
        core::ptr::addr_of_mut!(MapSize),
        core::ptr::addr_of_mut!(DescriptorSize),
        core::ptr::addr_of_mut!(DescriptorVersion),
    );

    TRACE(b"Value of MapKey: %d\n\0".as_ptr() as PCHAR, (MapKey) as usize);
    TRACE(b"Value of MapSize: %d\n\0".as_ptr() as PCHAR, (MapSize) as usize);
    TRACE(b"Value of DescriptorSize: %d\n\0".as_ptr() as PCHAR, (DescriptorSize) as usize);
    TRACE(b"Value of DescriptorVersion: %d\n\0".as_ptr() as PCHAR, (DescriptorVersion) as usize);

    EntryCount = (MapSize / DescriptorSize) as UINT32;

    FreeldrMemMapSize = (core::mem::size_of::<FREELDR_MEMORY_DESCRIPTOR>() * EntryCount as usize) as SIZE_T;
    let status_alloc = ((*(*GlobalSystemTable).BootServices).AllocatePool).unwrap()(
        EfiLoaderData,
        (FreeldrMemMapSize) as UINTN ,
        core::ptr::addr_of_mut!(FreeldrMem) as *mut *mut core::ffi::c_void,
    );
    if status_alloc != EFI_SUCCESS {
        TRACE(b"Failed to allocate pool with status %d\n\0".as_ptr() as PCHAR, (status_alloc) as usize);
        UiMessageBoxCritical(b"Unable to initialize memory manager.\0".as_ptr() as PCHAR);
        return core::ptr::null_mut();
    }

    RtlZeroMemory(FreeldrMem as *mut core::ffi::c_void, FreeldrMemMapSize);
    MapEntry = EfiMemoryMap;
    let mut Index: UINT32 = 0;
    while Index < EntryCount {
        let mut MemoryType: TYPE_OF_MEMORY = UefiConvertToFreeldrDesc((*MapEntry).Type);
        if MemoryType == LoaderFree {
            let mut PhysAddr = (*MapEntry).PhysicalStart;
            let status_alloc_pages = ((*(*GlobalSystemTable).BootServices).AllocatePages).unwrap()(
                AllocateAddress,
                EfiLoaderData,
                ((*MapEntry).NumberOfPages) as UINTN ,
                core::ptr::addr_of_mut!(PhysAddr),
            );
            if status_alloc_pages != EFI_SUCCESS {
                MemoryType = LoaderFirmwareTemporary;
            }
        }

        // DIVERGES from the C on i386: the firmware's own loader regions (OVMF keeps them at
        // the top of RAM) are identity-mapped only, never placed in KSEG0, so they must not
        // size the kernel's KSEG0 boot-image reservation. Counted, a 1G i386 guest got
        // MmBootImageSize 0x3F400000, a reservation from 0x80000000 to 0xBF400000 covering
        // the system-view and session ranges; the ROS gap finder then handed out session VAs
        // ARM3 had mapped (BUGCHECK 0x1A in MmDeleteRmap; any i386 UEFI boot with >= 1G).
        // MempSetupPagingForRegion maps identity-only types without the span gate, so they
        // stay mapped. 64-bit KSEG0 has room, and keeps the C behaviour.
        if MemoryType == LoaderLoadedProgram && !cfg!(target_arch = "x86") {
            if (((*MapEntry).PhysicalStart + ((*MapEntry).NumberOfPages * PAGE_SIZE as u64)) >> EFI_PAGE_SHIFT) > ((LoaderPagesSpanned) as u64) {
                LoaderPagesSpanned = ((((*MapEntry).PhysicalStart + ((*MapEntry).NumberOfPages * PAGE_SIZE as u64)) >> EFI_PAGE_SHIFT)) as usize ;
            }
        }

        if MemoryType != LoaderReserve {
            UefiSetMemory(
                FreeldrMem,
                (*MapEntry).PhysicalStart as ULONG_PTR,
                (*MapEntry).NumberOfPages as PFN_COUNT,
                MemoryType,
            );
        }

        MapEntry = NEXT_MEMORY_DESCRIPTOR(MapEntry, DescriptorSize);
        Index += 1;
    }

    UefiSetMemory(FreeldrMem, 0x000000, 1, LoaderFirmwarePermanent);
    *MemoryMapSize = FreeldrDescCount;
    FreeldrMem
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiExitBootServices()
{
    let mut MapKey: UINTN = 0;
    let mut MapSize: UINTN = 0;
    let mut Status: EFI_STATUS;
    let mut DescriptorSize: UINTN = 0;
    let mut DescriptorVersion: UINT32 = 0;

    TRACE(b"Attempting to exit bootsevices\n\0".as_ptr() as *const i8);
    PUEFI_LoadMemoryMap(&mut MapKey,
                        &mut MapSize,
                        &mut DescriptorSize,
                        &mut DescriptorVersion);

    Status = ((*(*GlobalSystemTable).BootServices).ExitBootServices).unwrap()(GlobalImageHandle, MapKey);
    if Status != EFI_SUCCESS {
        Status = ((*(*GlobalSystemTable).BootServices).ExitBootServices).unwrap()(GlobalImageHandle, MapKey);
    }

    if Status != EFI_SUCCESS {
        TRACE(b"Failed to exit boot services with status: %d\n\0".as_ptr() as *const i8, (Status) as usize);
        FrLdrBugCheckWithMessage(EXIT_BOOTSERVICES_FAILURE,
                                 b"uefimem.c\0".as_ptr() as *const i8,
                                 line!(),
                                 b"Failed to exit boot services with status: %d\0".as_ptr() as *const i8,
                                 Status);
    } else {
        TRACE(b"Exited bootservices\n\0".as_ptr() as *const i8);
    }
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiPrepareForReactOS() {
    _exituefi();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Memory Management Functions
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <uefildr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// #define NEXT_MEMORY_DESCRIPTOR(Descriptor, DescriptorSize) \
//     (EFI_MEMORY_DESCRIPTOR*)((char*)(Descriptor) + (DescriptorSize))
// #define EXIT_STACK_SIZE 0x1000
// #define UNUSED_MAX_DESCRIPTOR_COUNT 10000
//
// ULONG
// AddMemoryDescriptor(
//     _Inout_ PFREELDR_MEMORY_DESCRIPTOR List,
//     _In_ ULONG MaxCount,
//     _In_ PFN_NUMBER BasePage,
//     _In_ PFN_NUMBER PageCount,
//     _In_ TYPE_OF_MEMORY MemoryType);
//
// /* GLOBALS *******************************************************************/
//
// extern ULONG LoaderPagesSpanned;
// extern EFI_SYSTEM_TABLE* GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
//
// EFI_MEMORY_DESCRIPTOR* EfiMemoryMap = NULL;
// UINT32 FreeldrDescCount;
// PVOID OsLoaderBase;
// SIZE_T OsLoaderSize;
// EFI_HANDLE PublicBootHandle;
// PVOID ExitStack;
// PVOID EndofExitStack;
//
// void _exituefi(VOID);
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// PUEFI_LoadMemoryMap(
//     _Out_ UINTN*  LocMapKey,
//     _Out_ UINTN*  LocMapSize,
//     _Out_ UINTN*  LocDescriptorSize,
//     _Out_ UINT32* LocDescriptorVersion)
// {
//     EFI_STATUS Status;
//     UINTN AllocationSize = 0;
//     ULONG Count = 0;
//
//     Status = GlobalSystemTable->BootServices->GetMemoryMap(LocMapSize,
//                                                            EfiMemoryMap,
//                                                            LocMapKey,
//                                                            LocDescriptorSize,
//                                                            LocDescriptorVersion);
//
//     /* Reallocate and retrieve again the needed memory map size (since memory
//      * allocated by AllocatePool() counts in the map), until it's OK. */
//     while (Status != EFI_SUCCESS)
//     {
//         /* Reallocate the memory map buffer */
//         if (EfiMemoryMap)
//             GlobalSystemTable->BootServices->FreePool(EfiMemoryMap);
//
//         /* If MapSize never reports the correct size after the first time, increment */
//         AllocationSize = *LocMapSize + (*LocDescriptorSize * (Count as UINTN));
//         GlobalSystemTable->BootServices->AllocatePool(EfiLoaderData, AllocationSize,
//                                                       (VOID**)&EfiMemoryMap);
//         Status = GlobalSystemTable->BootServices->GetMemoryMap(LocMapSize,
//                                                                EfiMemoryMap,
//                                                                LocMapKey,
//                                                                LocDescriptorSize,
//                                                                LocDescriptorVersion);
//         Count++;
//     }
// }
//
// static
// VOID
// UefiSetMemory(
//     _Inout_ PFREELDR_MEMORY_DESCRIPTOR MemoryMap,
//     _In_ ULONG_PTR BaseAddress,
//     _In_ PFN_COUNT SizeInPages,
//     _In_ TYPE_OF_MEMORY MemoryType)
// {
//     ULONG_PTR BasePage, PageCount;
//
//     BasePage = BaseAddress / EFI_PAGE_SIZE;
//     PageCount = SizeInPages;
//
//     /* Add the memory descriptor */
//     FreeldrDescCount = AddMemoryDescriptor(MemoryMap,
//                                            UNUSED_MAX_DESCRIPTOR_COUNT,
//                                            BasePage,
//                                            PageCount,
//                                            MemoryType);
// }
//
// static
// TYPE_OF_MEMORY
// UefiConvertToFreeldrDesc(EFI_MEMORY_TYPE EfiMemoryType)
// {
//     switch (EfiMemoryType)
//     {
//         case EfiReservedMemoryType:
//             return LoaderReserve;
//         case EfiLoaderCode:
//             return LoaderLoadedProgram;
//         case EfiLoaderData:
//             return LoaderLoadedProgram;
//         case EfiBootServicesCode:
//             return LoaderFirmwareTemporary;
//         case EfiBootServicesData:
//             return LoaderFirmwareTemporary;
//         case EfiRuntimeServicesCode:
//             return LoaderFirmwarePermanent;
//         case EfiRuntimeServicesData:
//             return LoaderFirmwarePermanent;
//         case EfiConventionalMemory:
//             return LoaderFree;
//         case EfiUnusableMemory:
//             return LoaderBad;
//         case EfiACPIReclaimMemory:
//             return LoaderFirmwareTemporary;
//         case EfiACPIMemoryNVS:
//             return LoaderReserve;
//         case EfiMemoryMappedIO:
//             return LoaderReserve;
//         case EfiMemoryMappedIOPortSpace:
//             return LoaderReserve;
//         default:
//             break;
//     }
//     return LoaderReserve;
// }
//
// PFREELDR_MEMORY_DESCRIPTOR
// UefiMemGetMemoryMap(ULONG *MemoryMapSize)
// {
//     EFI_LOADED_IMAGE_PROTOCOL *LoadedImage;
//     UINT32 DescriptorVersion;
//     SIZE_T FreeldrMemMapSize;
//     UINTN DescriptorSize;
//     EFI_STATUS Status;
//     UINTN MapSize;
//     UINTN MapKey;
//     UINT32 Index;
//
//     EFI_GUID EfiLoadedImageProtocol = EFI_LOADED_IMAGE_PROTOCOL_GUID;
//     PFREELDR_MEMORY_DESCRIPTOR FreeldrMem = NULL;
//     EFI_MEMORY_DESCRIPTOR* MapEntry = NULL;
//     UINT32 EntryCount = 0;
//     FreeldrDescCount = 0;
//
//     Status = GlobalSystemTable->BootServices->HandleProtocol(GlobalImageHandle,
//                                                              &EfiLoadedImageProtocol,
//                                                              (VOID**)&LoadedImage);
//     if (Status != EFI_SUCCESS)
//     {
//         TRACE("Failed to find LoadedImageHandle with status: %d\n", (Status) as usize);
//         UiMessageBoxCritical("Unable to initialize memory manager.");
//         return NULL;
//     }
//     OsLoaderBase = LoadedImage->ImageBase;
//     OsLoaderSize = LoadedImage->ImageSize;
//     PublicBootHandle = LoadedImage->DeviceHandle;
//     EfiMemoryMap = NULL;
//
//     TRACE("UefiMemGetMemoryMap: Gather memory map\n");
//     PUEFI_LoadMemoryMap(&MapKey,
//                         &MapSize,
//                         &DescriptorSize,
//                         &DescriptorVersion);
//
//     TRACE("Value of MapKey: %d\n", (MapKey) as usize);
//     TRACE("Value of MapSize: %d\n", (MapSize) as usize);
//     TRACE("Value of DescriptorSize: %d\n", (DescriptorSize) as usize);
//     TRACE("Value of DescriptorVersion: %d\n", (DescriptorVersion) as usize);
//
//     EntryCount = (MapSize / DescriptorSize);
//
//     FreeldrMemMapSize = (sizeof(FREELDR_MEMORY_DESCRIPTOR) * EntryCount);
//     Status = GlobalSystemTable->BootServices->AllocatePool(EfiLoaderData,
//                                                            FreeldrMemMapSize,
//                                                            (void**)&FreeldrMem);
//     if (Status != EFI_SUCCESS)
//     {
//         TRACE("Failed to allocate pool with status %d\n", (Status) as usize);
//         UiMessageBoxCritical("Unable to initialize memory manager.");
//         return NULL;
//     }
//
//     RtlZeroMemory(FreeldrMem, FreeldrMemMapSize);
//     MapEntry = EfiMemoryMap;
// 	for (Index = 0; Index < EntryCount; ++Index)
//     {
//         TYPE_OF_MEMORY MemoryType = UefiConvertToFreeldrDesc(MapEntry->Type);
//         if (MemoryType == LoaderFree)
//         {
//             Status = GlobalSystemTable->BootServices->AllocatePages(AllocateAddress,
//                                                                     EfiLoaderData,
//                                                                     MapEntry->NumberOfPages,
//                                                                     &MapEntry->PhysicalStart);
//             if (Status != EFI_SUCCESS)
//             {
//                 /* We failed to reserve the page, so change its type */
//                 MemoryType = LoaderFirmwareTemporary;
//             }
//         }
//
//         /* Sometimes our loader can be loaded into higher memory than we ever allocate */
//         if (MemoryType == LoaderLoadedProgram)
//         {
//             if (((MapEntry->PhysicalStart + (MapEntry->NumberOfPages * PAGE_SIZE)) >> EFI_PAGE_SHIFT) > LoaderPagesSpanned)
//             {
//                 /* This value needs to be adjusted if this occurs */
//                 LoaderPagesSpanned = ((MapEntry->PhysicalStart + (MapEntry->NumberOfPages * PAGE_SIZE)) >> EFI_PAGE_SHIFT);
//             }
//         }
//
//         /* We really don't want to touch these reserved spots at all */
//         if (MemoryType != LoaderReserve)
//         {
//             UefiSetMemory(FreeldrMem,
//                           MapEntry->PhysicalStart,
//                           MapEntry->NumberOfPages,
//                           MemoryType);
//         }
//
//         MapEntry = NEXT_MEMORY_DESCRIPTOR(MapEntry, DescriptorSize);
//     }
//
//     /* Windows expects the first page to be reserved, otherwise it asserts.
//      * However it can be just a free page on some UEFI systems. */
//     UefiSetMemory(FreeldrMem, 0x000000, 1, LoaderFirmwarePermanent);
//     *MemoryMapSize = FreeldrDescCount;
//     return FreeldrMem;
// }
//
// VOID
// UefiExitBootServices(VOID)
// {
//     UINTN MapKey;
//     UINTN MapSize;
//     EFI_STATUS Status;
//     UINTN DescriptorSize;
//     UINT32 DescriptorVersion;
//
//     TRACE("Attempting to exit bootsevices\n");
//     PUEFI_LoadMemoryMap(&MapKey,
//                         &MapSize,
//                         &DescriptorSize,
//                         &DescriptorVersion);
//
//     Status = GlobalSystemTable->BootServices->ExitBootServices(GlobalImageHandle, MapKey);
//     /* UEFI spec demands twice! */
//     if (Status != EFI_SUCCESS)
//         Status = GlobalSystemTable->BootServices->ExitBootServices(GlobalImageHandle, MapKey);
//
//     if (Status != EFI_SUCCESS)
//     {
//         TRACE("Failed to exit boot services with status: %d\n", (Status) as usize);
//         FrLdrBugCheckWithMessage(EXIT_BOOTSERVICES_FAILURE,
//                                  __FILE__,
//                                  __LINE__,
//                                  "Failed to exit boot services with status: %d",
//                                  Status);
//     }
//     else
//     {
//         TRACE("Exited bootservices\n");
//     }
// }
//
// VOID
// UefiPrepareForReactOS(VOID)
// {
//     _exituefi();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
