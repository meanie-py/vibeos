#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_HWDETECT; // hwpci.c: DBG_DEFAULT_CHANNEL(HWDETECT)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_LIST, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA};
// freeldr_ddk_imports dropped (not in ddk): BusNumber
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_AdapterClass as AdapterClass;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_PeripheralClass as PeripheralClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_RealModeIrqRoutingTable as RealModeIrqRoutingTable;
extern "C" {
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS) -> i32;
    fn FrLdrHeapAlloc(size: usize, tag: ULONG) -> *mut u8;
    fn FldrCreateComponentKey( Parent: PCONFIGURATION_COMPONENT_DATA, Class: u32, Type: u32, Flags: ULONG, Key: ULONG, Affinity: ULONG, Identifier: *const i8, ConfigData: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA, ) -> i32;
}

// hwpci.c: FIND_PCI_BIOS FindPciBios = NULL; (external linkage; set by machpc.c).
// machpc.rs views this symbol as *mut c_void — keep that ABI.
pub type FIND_PCI_BIOS = unsafe extern "C" fn(PPCI_REGISTRY_INFO) -> BOOLEAN;
#[no_mangle]
pub static mut FindPciBios: *mut core::ffi::c_void = core::ptr::null_mut();

// arch/pc/hardware.h: #define TAG_HW_RESOURCE_LIST 'lRwH'

// ---- Dropped non-fn content faithfully restored (rustconv drops structs/consts) ----

// xdk/cmtypes.h #define
pub const CmResourceTypeBusNumber: UCHAR = 6;

// reactos/drivers/pci/pci.h (pshpack1)
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct PIN_INFO {
    pub Link: UCHAR,
    pub InterruptMap: USHORT,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct SLOT_INFO {
    pub BusNumber: UCHAR,
    pub DeviceNumber: UCHAR,
    pub PinInfo: [PIN_INFO; 4],
    pub SlotNumber: UCHAR,
    pub Reserved: UCHAR,
}

#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct PCI_IRQ_ROUTING_TABLE {
    pub Signature: ULONG,
    pub Version: USHORT,
    pub TableSize: USHORT,
    pub RouterBus: UCHAR,
    pub RouterDevFunc: UCHAR,
    pub ExclusiveIRQs: USHORT,
    pub CompatibleRouter: ULONG,
    pub MiniportData: ULONG,
    pub Reserved: [UCHAR; 11],
    pub Checksum: UCHAR,
    pub Slot: [SLOT_INFO; 1], // ANYSIZE_ARRAY
}
pub type PPCI_IRQ_ROUTING_TABLE = *mut PCI_IRQ_ROUTING_TABLE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PCI_REGISTRY_INFO {
    pub MajorRevision: UCHAR,
    pub MinorRevision: UCHAR,
    pub NoBuses: UCHAR,
    pub HardwareMechanism: UCHAR,
}
pub type PPCI_REGISTRY_INFO = *mut PCI_REGISTRY_INFO;

unsafe fn GetPciIrqRoutingTable() -> PPCI_IRQ_ROUTING_TABLE {
    let mut Table: PPCI_IRQ_ROUTING_TABLE = 0xF0000 as PPCI_IRQ_ROUTING_TABLE;
    
    while (Table as ULONG_PTR) < 0x100000 {
        if (*Table).Signature == 0x52495024 {
            TRACE!("Found signature\n");

            if ((*Table).TableSize as u32) < crate::FIELD_OFFSET!(PCI_IRQ_ROUTING_TABLE, Slot) ||
                (*Table).TableSize % 16 != 0 {
                ERR!("Invalid routing table size (%u) at 0x%p. Continue searching...\n", ((*Table).TableSize) as usize, (Table) as usize);
                Table = ((Table as ULONG_PTR) + 0x10) as PPCI_IRQ_ROUTING_TABLE;
                continue;
            }

            let Ptr: PUCHAR = Table as PUCHAR;
            let mut Sum: ULONG = 0;
            for i in 0..(*Table).TableSize {
                Sum += *Ptr.add(i as usize) as ULONG;
            }

            if (Sum & 0xFF) != 0 {
                ERR!("Invalid routing table checksum (%#lx) at 0x%p. Continue searching...\n", (Sum & 0xFF) as usize, (Table) as usize);
            } else {
                TRACE!("Valid checksum (%#lx): found routing table at 0x%p\n", (Sum & 0xFF) as usize, (Table) as usize);
                return Table;
            }
        }

        Table = ((Table as ULONG_PTR) + 0x10) as PPCI_IRQ_ROUTING_TABLE;
    }

    ERR!("No valid routing table found!\n");

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn PcFindPciBios(BusData: PPCI_REGISTRY_INFO) -> BOOLEAN {
    let mut RegsIn: REGS = core::mem::zeroed();
    let mut RegsOut: REGS = core::mem::zeroed();

    RegsIn.b.ah = 0xB1;
    RegsIn.b.al = 0x01;

    Int386(0x1A, &mut RegsIn, &mut RegsOut);

    if INT386_SUCCESS(RegsOut) != 0 &&
        RegsOut.d.edx == 0x20494350 &&
        RegsOut.b.ah == 0
    {
        TRACE!("Found PCI bios\n");

        TRACE!("AL: %x\n", (RegsOut.b.al) as usize);
        TRACE!("BH: %x\n", (RegsOut.b.bh) as usize);
        TRACE!("BL: %x\n", (RegsOut.b.bl) as usize);
        TRACE!("CL: %x\n", (RegsOut.b.cl) as usize);

        (*BusData).NoBuses = RegsOut.b.cl + 1;
        (*BusData).MajorRevision = RegsOut.b.bh;
        (*BusData).MinorRevision = RegsOut.b.bl;
        (*BusData).HardwareMechanism = RegsOut.b.al;

        return 1u8;
    }

    TRACE!("No PCI bios found\n");

    return 0u8;
}

unsafe fn DetectPciIrqRoutingTable(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Table: PPCI_IRQ_ROUTING_TABLE;
    let mut TableKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: ULONG;

    Table = GetPciIrqRoutingTable();
    if Table != core::ptr::null_mut() {
        TRACE!("Table size: %u\n", ((*Table).TableSize) as usize);

        Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]) + (((*Table).TableSize) as u32);
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList == core::ptr::null_mut() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as SIZE_T);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 2;

        PartialDescriptor = (*PartialResourceList).PartialDescriptors.as_mut_ptr().add(0);
        (*PartialDescriptor).Type = CmResourceTypeBusNumber;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
        (*PartialDescriptor).u.BusNumber.Start = 0;
        (*PartialDescriptor).u.BusNumber.Length = 1;

        PartialDescriptor = (*PartialResourceList).PartialDescriptors.as_mut_ptr().add(1);
        (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = ((*Table).TableSize) as u32 ;

        RtlCopyMemory(
            (*PartialResourceList).PartialDescriptors.as_mut_ptr().add(2) as *mut core::ffi::c_void,
            Table as *const core::ffi::c_void,
            (*Table).TableSize as SIZE_T
        );

        FldrCreateComponentKey(
            BusKey,
            PeripheralClass,
            RealModeIrqRoutingTable,
            0,
            0,
            0xFFFFFFFF,
            b"PCI Real-mode IRQ Routing Table\0".as_ptr() as PCHAR,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(TableKey)
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn DetectPciBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut BusData: PCI_REGISTRY_INFO = core::mem::zeroed();
    let mut BiosKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut BusKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut Size: ULONG;
    let mut i: ULONG;

    if core::mem::transmute::<_, FIND_PCI_BIOS>(FindPciBios)(core::ptr::addr_of_mut!(BusData)) != 0 {
        Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList == core::ptr::null_mut() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        RtlZeroMemory(PartialResourceList as *mut VOID, (Size) as usize );

        FldrCreateComponentKey(
            SystemKey,
            AdapterClass,
            MultiFunctionAdapter,
            0,
            0,
            0xFFFFFFFF,
            b"PCI BIOS\0".as_ptr() as PCHAR,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(BiosKey)
        );

        *BusNumber = (*BusNumber).wrapping_add(1);

        // The $PIR table lives in the x86 BIOS area (0xF0000); arm64 has no such
        // memory, and its routing comes from the firmware (uefihw's fixup) instead.
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        DetectPciIrqRoutingTable(BiosKey);

        i = 0;
        while i < BusData.NoBuses as ULONG {
            if i == 0 {
                Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + core::mem::size_of::<PCI_REGISTRY_INFO>() as ULONG;
                PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
                if PartialResourceList == core::ptr::null_mut() {
                    ERR!("Failed to allocate resource descriptor! Ignoring remaining PCI buses. (i = %lu, NoBuses = %lu)\n", (i) as usize, (BusData.NoBuses as ULONG) as usize);
                    return;
                }

                RtlZeroMemory(PartialResourceList as *mut VOID, (Size) as usize );
                (*PartialResourceList).Version = 1;
                (*PartialResourceList).Revision = 1;
                (*PartialResourceList).Count = 1;

                PartialDescriptor = (*PartialResourceList).PartialDescriptors.as_mut_ptr().add(0);
                (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
                (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
                (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<PCI_REGISTRY_INFO>() as ULONG;

                RtlCopyMemory(
                    (*PartialResourceList).PartialDescriptors.as_mut_ptr().add(1) as *mut VOID,
                    core::ptr::addr_of!(BusData) as *const VOID,
                    core::mem::size_of::<PCI_REGISTRY_INFO>()
                );
            } else {
                Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
                PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
                if PartialResourceList == core::ptr::null_mut() {
                    ERR!("Failed to allocate resource descriptor! Ignoring remaining PCI buses. (i = %lu, NoBuses = %lu)\n", (i) as usize, (BusData.NoBuses as ULONG) as usize);
                    return;
                }

                RtlZeroMemory(PartialResourceList as *mut VOID, (Size) as usize );
            }

            FldrCreateComponentKey(
                SystemKey,
                AdapterClass,
                MultiFunctionAdapter,
                0,
                0,
                0xFFFFFFFF,
                b"PCI\0".as_ptr() as PCHAR,
                PartialResourceList,
                Size,
                core::ptr::addr_of_mut!(BusKey)
            );

            *BusNumber = (*BusNumber).wrapping_add(1);

            i = i.wrapping_add(1);
        }
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
// FIND_PCI_BIOS FindPciBios = NULL;
//
// static
// PPCI_IRQ_ROUTING_TABLE
// GetPciIrqRoutingTable(VOID)
// {
//     PPCI_IRQ_ROUTING_TABLE Table;
//     PUCHAR Ptr;
//     ULONG Sum;
//     ULONG i;
//
//     Table = (PPCI_IRQ_ROUTING_TABLE)0xF0000;
//     while ((ULONG_PTR)Table < 0x100000)
//     {
//         if (Table->Signature == 'RIP$')
//         {
//             TRACE("Found signature\n");
//
//             if (Table->TableSize < crate::FIELD_OFFSET!(PCI_IRQ_ROUTING_TABLE, Slot) ||
//                 Table->TableSize % 16 != 0)
//             {
//                 ERR("Invalid routing table size (%u) at 0x%p. Continue searching...\n", (Table->TableSize) as usize, (Table) as usize);
//                 Table = (PPCI_IRQ_ROUTING_TABLE)((ULONG_PTR)Table + 0x10);
//                 continue;
//             }
//
//             Ptr = (PUCHAR)Table;
//             Sum = 0;
//             for (i = 0; i < Table->TableSize; i++)
//             {
//                 Sum += Ptr[i];
//             }
//
//             if ((Sum & 0xFF) != 0)
//             {
//                 ERR("Invalid routing table checksum (%#lx) at 0x%p. Continue searching...\n", (Sum & 0xFF) as usize, (Table) as usize);
//             }
//             else
//             {
//                 TRACE("Valid checksum (%#lx): found routing table at 0x%p\n", (Sum & 0xFF) as usize, (Table) as usize);
//                 return Table;
//             }
//         }
//
//         Table = (PPCI_IRQ_ROUTING_TABLE)((ULONG_PTR)Table + 0x10);
//     }
//
//     ERR("No valid routing table found!\n");
//
//     return NULL;
// }
//
//
// BOOLEAN
// PcFindPciBios(PPCI_REGISTRY_INFO BusData)
// {
//     REGS  RegsIn;
//     REGS  RegsOut;
//
//     RegsIn.b.ah = 0xB1; /* Subfunction B1h */
//     RegsIn.b.al = 0x01; /* PCI BIOS present */
//
//     Int386(0x1A, &RegsIn, &RegsOut);
//
//     if (INT386_SUCCESS(RegsOut) &&
//         (RegsOut.d.edx == ' ICP') &&
//         (RegsOut.b.ah == 0))
//     {
//         TRACE("Found PCI bios\n");
//
//         TRACE("AL: %x\n", (RegsOut.b.al) as usize);
//         TRACE("BH: %x\n", (RegsOut.b.bh) as usize);
//         TRACE("BL: %x\n", (RegsOut.b.bl) as usize);
//         TRACE("CL: %x\n", (RegsOut.b.cl) as usize);
//
//         BusData->NoBuses = RegsOut.b.cl + 1;
//         BusData->MajorRevision = RegsOut.b.bh;
//         BusData->MinorRevision = RegsOut.b.bl;
//         BusData->HardwareMechanism = RegsOut.b.al;
//
//         return TRUE;
//     }
//
//     TRACE("No PCI bios found\n");
//
//     return FALSE;
// }
//
//
// static
// VOID
// DetectPciIrqRoutingTable(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PPCI_IRQ_ROUTING_TABLE Table;
//     PCONFIGURATION_COMPONENT_DATA TableKey;
//     ULONG Size;
//
//     Table = GetPciIrqRoutingTable();
//     if (Table != NULL)
//     {
//         TRACE("Table size: %u\n", (Table->TableSize) as usize);
//
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]) + Table->TableSize;
//         PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 2;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypeBusNumber;
//         PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//         PartialDescriptor->u.BusNumber.Start = 0;
//         PartialDescriptor->u.BusNumber.Length = 1;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = Table->TableSize;
//
//         RtlCopyMemory(&PartialResourceList->PartialDescriptors[2],
//                       Table, Table->TableSize);
//
//         FldrCreateComponentKey(BusKey,
//                                PeripheralClass,
//                                RealModeIrqRoutingTable,
//                                0,
//                                0,
//                                0xFFFFFFFF,
//                                "PCI Real-mode IRQ Routing Table",
//                                PartialResourceList,
//                                Size,
//                                &TableKey);
//     }
// }
//
//
// VOID
// DetectPciBios(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCI_REGISTRY_INFO BusData;
//     PCONFIGURATION_COMPONENT_DATA BiosKey;
//     PCONFIGURATION_COMPONENT_DATA BusKey;
//     ULONG Size;
//     ULONG i;
//
//     /* Report the PCI BIOS */
//     if (FindPciBios(&BusData))
//     {
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//         PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//
//         /* Create new bus key */
//         FldrCreateComponentKey(SystemKey,
//                                AdapterClass,
//                                MultiFunctionAdapter,
//                                0,
//                                0,
//                                0xFFFFFFFF,
//                                "PCI BIOS",
//                                PartialResourceList,
//                                Size,
//                                &BiosKey);
//
//         /* Increment bus number */
//         (*BusNumber)++;
//
//         DetectPciIrqRoutingTable(BiosKey);
//
//         /* Report PCI buses */
//         for (i = 0; i < (ULONG)BusData.NoBuses; i++)
//         {
//             /* Check if this is the first bus */
//             if (i == 0)
//             {
//                 /* Set 'Configuration Data' value */
//                 Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) +
//                        sizeof(BusData);
//                 PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST);
//                 if (!PartialResourceList)
//                 {
//                     ERR("Failed to allocate resource descriptor! Ignoring remaining PCI buses. (i = %lu, NoBuses = %lu)\n", (//                         i) as usize, ((ULONG)BusData.NoBuses) as usize);
//                     return;
//                 }
//
//                 /* Initialize resource descriptor */
//                 RtlZeroMemory(PartialResourceList, Size);
//                 PartialResourceList->Version = 1;
//                 PartialResourceList->Revision = 1;
//                 PartialResourceList->Count = 1;
//
//                 PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//                 PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//                 PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//                 PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(BusData);
//
//                 RtlCopyMemory(&PartialResourceList->PartialDescriptors[1],
//                               &BusData, sizeof(BusData));
//             }
//             else
//             {
//                 /* Set 'Configuration Data' value */
//                 Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//                 PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST);
//                 if (!PartialResourceList)
//                 {
//                     ERR("Failed to allocate resource descriptor! Ignoring remaining PCI buses. (i = %lu, NoBuses = %lu)\n", (//                         i) as usize, ((ULONG)BusData.NoBuses) as usize);
//                     return;
//                 }
//
//                 /* Initialize resource descriptor */
//                 RtlZeroMemory(PartialResourceList, Size);
//             }
//
//             /* Create the bus key */
//             FldrCreateComponentKey(SystemKey,
//                                    AdapterClass,
//                                    MultiFunctionAdapter,
//                                    0,
//                                    0,
//                                    0xFFFFFFFF,
//                                    "PCI",
//                                    PartialResourceList,
//                                    Size,
//                                    &BusKey);
//
//             /* Increment bus number */
//             (*BusNumber)++;
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
