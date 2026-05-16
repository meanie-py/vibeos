use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA, PCSTR};
// UINT32 comes from crate::frldr::* (keystone EFI surface).
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_AdapterClass as AdapterClass;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_ControllerClass as ControllerClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DisplayController as DisplayController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;

extern "C" {
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn ERR(fmt: *const i8, ...);
    fn FldrCreateComponentKey( Parent: PCONFIGURATION_COMPONENT_DATA, Class: u32, Type: u32, Flags: ULONG, Key: ULONG, Affinity: ULONG, Identifier: *const i8, Data: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA, );
    fn FrLdrHeapAlloc(MemorySize: usize, Tag: ULONG) -> *mut core::ffi::c_void;
    fn TRACE(fmt: *const i8, ...);
    fn FldrCreateSystemKey( SystemKey: *mut PCONFIGURATION_COMPONENT_DATA, SystemClass: *const i8, );
    // cross-module data from uefildr.c / uefimem.c / uefivid.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
    static mut GlobalImageHandle: EFI_HANDLE;
    static mut PcBiosDiskCount: UCHAR;
    static mut EfiMemoryMap: *mut EFI_MEMORY_DESCRIPTOR;
    static mut FreeldrDescCount: UINT32;
    static mut VramAddress: ULONG_PTR;
    static mut VramSize: ULONG;
    static mut FrameBufferData: PCM_FRAMEBUF_DEVICE_DATA;
    // AcpiPresent is #[no_mangle]-owned by crate::hwacpi (hwacpi.c also
    // defines it; mutually-exclusive C build, single Rust definer). extern here.
    static mut AcpiPresent: BOOLEAN;
}

// cmtypes.h resource constants.
const CmResourceTypeMemory: u8 = 3;
const CM_RESOURCE_MEMORY_READ_WRITE: USHORT = 0x0000;

// arc.h enum _IDENTIFIER_FLAG bits (used as FldrCreateComponentKey flags).
const ConsoleOut: ULONG = 0x10;
const Output: ULONG = 0x40;

// uefihw.c `BOOLEAN AcpiPresent = FALSE;` — not redefined here; crate::hwacpi
// owns the #[no_mangle] symbol (extern-declared in the block above).

unsafe fn IsAcpiPresent() -> BOOLEAN {
    AcpiPresent
}

unsafe fn FindAcpiBios() -> *mut RSDP_DESCRIPTOR {
    let mut i: UINTN;
    let mut rsdp: *mut RSDP_DESCRIPTOR = core::ptr::null_mut();
    let acpi2_guid: EFI_GUID = EFI_ACPI_20_TABLE_GUID;

    i = 0;
    while i < (*GlobalSystemTable).NumberOfTableEntries {
        if memcmp(
            core::ptr::addr_of!((*(*GlobalSystemTable).ConfigurationTable.add(((i) as usize))).VendorGuid) as *const core::ffi::c_void,
            core::ptr::addr_of!(acpi2_guid) as *const core::ffi::c_void,
            core::mem::size_of::<EFI_GUID>(),
        ) == 0
        {
            rsdp = (*(*GlobalSystemTable).ConfigurationTable.add(((i) as usize))).VendorTable as *mut RSDP_DESCRIPTOR;
            break;
        }
        i += 1;
    }

    rsdp
}

unsafe fn DetectAcpiBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {
    let mut BiosKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut Rsdp: *mut RSDP_DESCRIPTOR;
    let mut AcpiBiosData: PACPI_BIOS_DATA;
    let mut TableSize: ULONG;
    let mut Size: ULONG;

    Rsdp = FindAcpiBios();

    if !Rsdp.is_null() {
        AcpiPresent = 1u8;

        TableSize = FreeldrDescCount * core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG +
            core::mem::size_of::<ACPI_BIOS_DATA>() as ULONG - core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG;

        Size = core::mem::offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG + 
               core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG + TableSize;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
            return;
        }

        core::ptr::write_bytes(PartialResourceList, 0, Size as usize);
        (*PartialResourceList).Version = 0;
        (*PartialResourceList).Revision = 0;
        (*PartialResourceList).Count = 1;

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
        (*PartialDescriptor).ShareDisposition = ((CmResourceShareUndetermined) as u8);
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = TableSize;

        AcpiBiosData = (PartialDescriptor.add(1)) as PACPI_BIOS_DATA;

        if (*Rsdp).revision > 0 {
            TRACE(b"ACPI >1.0, using XSDT address\n\0".as_ptr() as *const i8);
            (*AcpiBiosData).RSDTAddress.QuadPart = (((*Rsdp).xsdt_physical_address) as i64);
        } else {
            TRACE(b"ACPI 1.0, using RSDT address\n\0".as_ptr() as *const i8);
            (*AcpiBiosData).RSDTAddress.u.LowPart = (*Rsdp).rsdt_physical_address;
        }

        (*AcpiBiosData).Count = ((FreeldrDescCount) as u64);
        core::ptr::copy_nonoverlapping(
            EfiMemoryMap as *const u8,
            core::ptr::addr_of_mut!((*AcpiBiosData).MemoryMap) as *mut u8,
            (FreeldrDescCount * ((core::mem::size_of::<BIOS_MEMORY_MAP>()) as u32)) as usize,
        );

        TRACE(b"RSDT %p, data size %x\n\0".as_ptr() as *const i8, ((*Rsdp).rsdt_physical_address) as usize, (TableSize) as usize);

        FldrCreateComponentKey(
            SystemKey,
            AdapterClass,
            MultiFunctionAdapter,
            0x0,
            0x0,
            0xFFFFFFFF,
            b"ACPI BIOS\0".as_ptr() as *const i8,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(BiosKey),
        );

        *BusNumber += 1;
    }
}

unsafe fn DetectDisplayController(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let FramebufData: PCM_FRAMEBUF_DEVICE_DATA;
    let Size: u32;

    if VramAddress == 0 || VramSize == 0 || FrameBufferData.is_null() {
        return;
    }

    Size = (core::mem::size_of::<CM_PARTIAL_RESOURCE_LIST>() 
           - core::mem::size_of::<PCM_PARTIAL_RESOURCE_DESCRIPTOR>() 
           + 2 * core::mem::size_of::<PCM_PARTIAL_RESOURCE_DESCRIPTOR>()
           + core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>()) as u32;
    
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
        return;
    }

    core::ptr::write_bytes(PartialResourceList, 0, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 2;
    (*PartialResourceList).Count = 2;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypeMemory;
    (*PartialDescriptor).ShareDisposition = ((CmResourceShareDeviceExclusive) as u8);
    (*PartialDescriptor).Flags = CM_RESOURCE_MEMORY_READ_WRITE;
    (*PartialDescriptor).u.Memory.Start.QuadPart = ((VramAddress) as i64);
    (*PartialDescriptor).u.Memory.Length = VramSize;

    // flex array: element [1] is past the [1]-sized field — pointer-step it.
    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]).add(1);
    (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
    (*PartialDescriptor).ShareDisposition = ((CmResourceShareUndetermined) as u8);
    (*PartialDescriptor).Flags = 0;
    (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as u32;

    FramebufData = PartialDescriptor.offset(1) as PCM_FRAMEBUF_DEVICE_DATA;
    core::ptr::copy_nonoverlapping(
        FrameBufferData as *const u8,
        FramebufData as *mut u8,
        core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>()
    );
    (*FramebufData).Version = 1;
    (*FramebufData).Revision = 3;
    (*FramebufData).VideoClock = 0;

    FldrCreateComponentKey(
        BusKey,
        ControllerClass,
        DisplayController,
        Output | ConsoleOut,
        0,
        0xFFFFFFFF,
        b"UEFI GOP Framebuffer\0".as_ptr() as *const i8,
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(ControllerKey)
    );
}

unsafe fn DetectInternal(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut u32) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut BusKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: u32;

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as u32;
    PartialResourceList = FrLdrHeapAlloc(Size as usize, 0x74736952) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR(b"Failed to allocate resource descriptor\n\0".as_ptr() as *const i8);
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 0;

    FldrCreateComponentKey(
        SystemKey,
        AdapterClass,
        MultiFunctionAdapter,
        0,
        0,
        0xFFFFFFFF,
        b"UEFI Internal\0".as_ptr() as *const i8,
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(BusKey) as *mut PCONFIGURATION_COMPONENT_DATA,
    );

    *BusNumber = (*BusNumber).wrapping_add(1);

    DetectDisplayController(BusKey);
}

#[no_mangle]
pub unsafe extern "C" fn UefiHwDetect(
    Options: PCSTR,
) -> PCONFIGURATION_COMPONENT_DATA {
    let mut SystemKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut BusNumber: ULONG = 0;

    TRACE(b"DetectHardware()\n\0".as_ptr() as *const i8);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    FldrCreateSystemKey(
        core::ptr::addr_of_mut!(SystemKey),
        b"AT/AT COMPATIBLE\0".as_ptr() as *const i8,
    );

    #[cfg(target_arch = "aarch64")]
    FldrCreateSystemKey(
        core::ptr::addr_of_mut!(SystemKey),
        b"ARM processor family\0".as_ptr() as *const i8,
    );

    DetectInternal(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    DetectAcpiBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));

    TRACE(b"DetectHardware() Done\n\0".as_ptr() as *const i8);
    SystemKey
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Hardware detection routines
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <uefildr.h>
// #include "../vidfb.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(WARNING);
//
// /* GLOBALS *******************************************************************/
//
// extern EFI_SYSTEM_TABLE * GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
// extern UCHAR PcBiosDiskCount;
// extern EFI_MEMORY_DESCRIPTOR* EfiMemoryMap;
// extern UINT32 FreeldrDescCount;
//
// /* From uefivid.c */
// extern ULONG_PTR VramAddress;
// extern ULONG VramSize;
// extern PCM_FRAMEBUF_DEVICE_DATA FrameBufferData;
//
// BOOLEAN AcpiPresent = FALSE;
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN IsAcpiPresent(VOID)
// {
//     return AcpiPresent;
// }
//
// static
// PRSDP_DESCRIPTOR
// FindAcpiBios(VOID)
// {
//     UINTN i;
//     RSDP_DESCRIPTOR* rsdp = NULL;
//     EFI_GUID acpi2_guid = EFI_ACPI_20_TABLE_GUID;
//
//     for (i = 0; i < GlobalSystemTable->NumberOfTableEntries; i++)
//     {
//         if (!memcmp(&GlobalSystemTable->ConfigurationTable[i].VendorGuid,
//                     &acpi2_guid, sizeof(acpi2_guid)))
//         {
//             rsdp = (RSDP_DESCRIPTOR*)GlobalSystemTable->ConfigurationTable[i].VendorTable;
//             break;
//         }
//     }
//
//     return rsdp;
// }
//
// VOID
// DetectAcpiBios(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCONFIGURATION_COMPONENT_DATA BiosKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PRSDP_DESCRIPTOR Rsdp;
//     PACPI_BIOS_DATA AcpiBiosData;
//     ULONG TableSize, Size;
//
//     Rsdp = FindAcpiBios();
//
//     if (Rsdp)
//     {
//         /* Set up the flag in the loader block */
//         AcpiPresent = TRUE;
//
//         /* Calculate the table size */
//         TableSize = FreeldrDescCount * sizeof(BIOS_MEMORY_MAP) +
//             sizeof(ACPI_BIOS_DATA) - sizeof(BIOS_MEMORY_MAP);
//
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + TableSize;
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 0;
//         PartialResourceList->Revision = 0;
//         PartialResourceList->Count = 1;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = TableSize;
//
//         /* Fill the table */
//         AcpiBiosData = (PACPI_BIOS_DATA)(PartialDescriptor + 1);
//
//         if (Rsdp->revision > 0)
//         {
//             TRACE("ACPI >1.0, using XSDT address\n");
//             AcpiBiosData->RSDTAddress.QuadPart = Rsdp->xsdt_physical_address;
//         }
//         else
//         {
//             TRACE("ACPI 1.0, using RSDT address\n");
//             AcpiBiosData->RSDTAddress.LowPart = Rsdp->rsdt_physical_address;
//         }
//
//         AcpiBiosData->Count = FreeldrDescCount;
//         RtlCopyMemory(AcpiBiosData->MemoryMap, EfiMemoryMap,
//                       FreeldrDescCount * sizeof(BIOS_MEMORY_MAP));
//
//         TRACE("RSDT %p, data size %x\n", (Rsdp->rsdt_physical_address) as usize, (TableSize) as usize);
//
//         /* Create new bus key */
//         FldrCreateComponentKey(SystemKey,
//                                AdapterClass,
//                                MultiFunctionAdapter,
//                                0x0,
//                                0x0,
//                                0xFFFFFFFF,
//                                "ACPI BIOS",
//                                PartialResourceList,
//                                Size,
//                                &BiosKey);
//
//         /* Increment bus number */
//         (*BusNumber)++;
//     }
// }
//
// static VOID
// DetectDisplayController(
//     _In_ PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_FRAMEBUF_DEVICE_DATA FramebufData;
//     ULONG Size;
//
//     if (!VramAddress || (VramSize == 0) || !FrameBufferData)
//         return;
//
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]) + sizeof(*FramebufData);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version  = 1;
//     PartialResourceList->Revision = 2;
//     PartialResourceList->Count = 2;
//
//     /* Set Memory */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypeMemory;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_MEMORY_READ_WRITE;
//     PartialDescriptor->u.Memory.Start.QuadPart = VramAddress;
//     PartialDescriptor->u.Memory.Length = VramSize;
//
//     /* Set framebuffer-specific data */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//     PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = 0;
//     PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*FramebufData);
//
//     /* Get pointer to framebuffer-specific data */
//     FramebufData = (PCM_FRAMEBUF_DEVICE_DATA)(PartialDescriptor + 1);
//     RtlCopyMemory(FramebufData, FrameBufferData, sizeof(*FrameBufferData));
//     FramebufData->Version  = 1;
//     FramebufData->Revision = 3;
//     FramebufData->VideoClock = 0; // FIXME: Use EDID
//
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            DisplayController,
//                            Output | ConsoleOut,
//                            0,
//                            0xFFFFFFFF,
//                            "UEFI GOP Framebuffer",
//                            PartialResourceList,
//                            Size,
//                            &ControllerKey);
//
//     // NOTE: Don't add a MonitorPeripheral for now.
//     // We should use EDID data for it.
// }
//
// static
// VOID
// DetectInternal(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCONFIGURATION_COMPONENT_DATA BusKey;
//     ULONG Size;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version  = 1;
//     PartialResourceList->Revision = 1;
//     PartialResourceList->Count = 0;
//
//     /* Create new bus key */
//     FldrCreateComponentKey(SystemKey,
//                            AdapterClass,
//                            MultiFunctionAdapter,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "UEFI Internal",
//                            PartialResourceList,
//                            Size,
//                            &BusKey);
//
//     /* Increment bus number */
//     (*BusNumber)++;
//
//     /* Detect devices that do not belong to "standard" buses */
//     DetectDisplayController(BusKey);
//
//     /* FIXME: Detect more devices */
// }
//
// PCONFIGURATION_COMPONENT_DATA
// UefiHwDetect(
//     _In_opt_ PCSTR Options)
// {
//     PCONFIGURATION_COMPONENT_DATA SystemKey;
//     ULONG BusNumber = 0;
//
//     TRACE("DetectHardware()\n");
//
//     /* Create the 'System' key */
// #if defined(_M_IX86) || defined(_M_AMD64)
//     FldrCreateSystemKey(&SystemKey, "AT/AT COMPATIBLE");
// #elif defined(_M_IA64)
//     FldrCreateSystemKey(&SystemKey, "Intel Itanium processor family");
// #elif defined(_M_ARM) || defined(_M_ARM64)
//     FldrCreateSystemKey(&SystemKey, "ARM processor family");
// #else
//     #error Please define a system key for your architecture
// #endif
//
//     /* Detect buses */
//     DetectInternal(SystemKey, &BusNumber);
//     // TODO: DetectPciBios
//     DetectAcpiBios(SystemKey, &BusNumber);
//
//     TRACE("DetectHardware() Done\n");
//     return SystemKey;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
