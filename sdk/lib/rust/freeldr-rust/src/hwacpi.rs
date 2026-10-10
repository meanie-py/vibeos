#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_HWDETECT; // hwacpi.c: DBG_DEFAULT_CHANNEL(HWDETECT)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA};
// freeldr_ddk_imports dropped (not in ddk): BusNumber
use core::ffi::c_void;

// ---- Dropped non-fn content faithfully restored (rustconv drops structs/statics) ----

// winldr.h: RSDP_DESCRIPTOR (Root System Descriptor Pointer, pshpack1)
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct RSDP_DESCRIPTOR {
    pub signature: [CHAR; 8],
    pub checksum: UCHAR,
    pub oem_id: [CHAR; 6],
    pub revision: UCHAR,
    pub rsdt_physical_address: ULONG,
    pub length: ULONG,
    pub xsdt_physical_address: ULONGLONG,
    pub extended_checksum: UCHAR,
    pub reserved: [CHAR; 3],
}
pub type PRSDP_DESCRIPTOR = *mut RSDP_DESCRIPTOR;

// pcbios.h: ACPI_BIOS_DATA (BIOS_MEMORY_MAP lives in keystone frldr.rs)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ACPI_BIOS_DATA {
    pub RSDTAddress: LARGE_INTEGER,
    pub Count: ULONGLONG,
    pub MemoryMap: [BIOS_MEMORY_MAP; 1],
}
pub type PACPI_BIOS_DATA = *mut ACPI_BIOS_DATA;

// hwacpi.c file-local global with external linkage (IsAcpiPresent consumed elsewhere).
#[no_mangle]
pub static mut AcpiPresent: BOOLEAN = FALSE;

// pcmem.c globals (defined #[no_mangle] in crate::pcmem); machpc.h extern decls.
extern "C" {
    static mut PcBiosMemoryMap: [BIOS_MEMORY_MAP; MAX_BIOS_DESCRIPTORS];
    static mut PcBiosMapCount: ULONG;
}
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_AdapterClass as AdapterClass;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;

extern "C" {
    fn memcmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn FldrCreateComponentKey( Parent: PCONFIGURATION_COMPONENT_DATA, Class: u32, Type: u32, Flags: ULONG, Key: ULONG, Affinity: ULONG, Identifier: *const i8, ConfigData: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA, ) -> i32;
    fn FrLdrHeapAlloc(size: usize, tag: ULONG) -> *mut u8;
}

#[no_mangle]
pub unsafe extern "C" fn IsAcpiPresent() -> BOOLEAN {
    AcpiPresent
}

unsafe fn FindAcpiBios() -> PRSDP_DESCRIPTOR {

    let mut Ptr: PUCHAR = 0xE0000 as PUCHAR;

    while (Ptr as ULONG_PTR) < 0x100000 {
        if memcmp(Ptr as *const i8, b"RSD PTR \0".as_ptr() as *const i8, 8) == 0 {
            TRACE!("ACPI supported\n");
            return Ptr as PRSDP_DESCRIPTOR;
        }

        Ptr = ((Ptr as ULONG_PTR) + 0x10) as PUCHAR;
    }

    TRACE!("ACPI not supported\n");

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn DetectAcpiBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {

    const FIELD_OFFSET_CM_PARTIAL_RESOURCE_LIST_PARTIALDESCRIPTORS: usize =
        core::mem::size_of::<USHORT>() + core::mem::size_of::<USHORT>() + core::mem::size_of::<ULONG>();

    let mut BiosKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Rsdp: PRSDP_DESCRIPTOR;
    let AcpiBiosData: PACPI_BIOS_DATA;
    let TableSize: ULONG;
    let Size: ULONG;

    Rsdp = FindAcpiBios();

    if !Rsdp.is_null() {
        AcpiPresent = 1u8;

        TableSize = PcBiosMapCount * core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG +
            core::mem::size_of::<ACPI_BIOS_DATA>() as ULONG - core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG;

        Size = FIELD_OFFSET_CM_PARTIAL_RESOURCE_LIST_PARTIALDESCRIPTORS as ULONG +
            core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG + TableSize;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 0;
        (*PartialResourceList).Revision = 0;
        (*PartialResourceList).Count = 1;

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = 8u8;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = TableSize;

        AcpiBiosData = (PartialDescriptor.offset(1)) as PACPI_BIOS_DATA;

        if (*Rsdp).revision > 0 {
            TRACE!("ACPI >1.0, using XSDT address\n");
            (*AcpiBiosData).RSDTAddress.QuadPart = ((*Rsdp).xsdt_physical_address) as i64 ;
        } else {
            TRACE!("ACPI 1.0, using RSDT address\n");
            (*AcpiBiosData).RSDTAddress.s.LowPart = (*Rsdp).rsdt_physical_address;
        }

        (*AcpiBiosData).Count = (PcBiosMapCount) as u64 ;
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(PcBiosMemoryMap) as *const u8,
            core::ptr::addr_of_mut!((*AcpiBiosData).MemoryMap[0]) as *mut u8,
            (PcBiosMapCount * core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG) as usize,
        );

        TRACE!("RSDT %p, data size %x\n", ((*Rsdp).rsdt_physical_address as *const u8) as usize, (TableSize) as usize);

        FldrCreateComponentKey(
            SystemKey,
            AdapterClass,
            MultiFunctionAdapter,
            0,
            0,
            0xFFFFFFFF,
            b"ACPI BIOS\0".as_ptr() as *const i8,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(BiosKey),
        );

        *BusNumber = (*BusNumber).wrapping_add(1);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  Copyright (C) 2004  Eric Kohl
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
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
// BOOLEAN AcpiPresent = FALSE;
//
// BOOLEAN IsAcpiPresent(VOID)
// {
//     return AcpiPresent;
// }
//
// static PRSDP_DESCRIPTOR
// FindAcpiBios(VOID)
// {
//     PUCHAR Ptr;
//
//     /* Find the 'Root System Descriptor Table Pointer' */
//     Ptr = (PUCHAR)0xE0000;
//     while ((ULONG_PTR)Ptr < 0x100000)
//     {
//         if (!memcmp(Ptr, "RSD PTR ", 8))
//         {
//             TRACE("ACPI supported\n");
//
//             return (PRSDP_DESCRIPTOR)Ptr;
//         }
//
//         Ptr = (PUCHAR)((ULONG_PTR)Ptr + 0x10);
//     }
//
//     TRACE("ACPI not supported\n");
//
//     return NULL;
// }
//
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
//         TableSize = PcBiosMapCount * sizeof(BIOS_MEMORY_MAP) +
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
//         AcpiBiosData->Count = PcBiosMapCount;
//         RtlCopyMemory(AcpiBiosData->MemoryMap, PcBiosMemoryMap,
//                       PcBiosMapCount * sizeof(BIOS_MEMORY_MAP));
//
//         TRACE("RSDT %p, data size %x\n", (Rsdp->rsdt_physical_address) as usize, (//             TableSize) as usize);
//
//         /* Create new bus key */
//         FldrCreateComponentKey(SystemKey,
//                                AdapterClass,
//                                MultiFunctionAdapter,
//                                0,
//                                0,
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
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
