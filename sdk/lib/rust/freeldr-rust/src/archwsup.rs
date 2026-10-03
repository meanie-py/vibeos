use ntkrnl_sys::base::*;
use crate::frldr::*;
// CONFIGURATION_COMPONENT / CONFIGURATION_CLASS / CONFIGURATION_TYPE /
// IDENTIFIER_FLAG / MaximumType / SystemClass / ARC_DISK_SIGNATURE_EX
// come from the keystone (arc.h enums that bindgen prefixes with
// `_CONFIGURATION_*` are unprefixed there).
use ntkrnl_sys::ddk::{CONFIGURATION_COMPONENT, CONFIGURATION_COMPONENT_DATA,
    PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT, PCONFIGURATION_COMPONENT_DATA,
    PCSTR, PSTR};
use core::ffi::c_void;
extern "C" {
    fn TRACE(fmt: *const i8, ...);
}

extern "C" {
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn strlen(s: PCSTR) -> usize;
    fn FrLdrHeapAlloc(size: usize, tag: u32) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
pub const ArcTypes: [PCSTR; (MaximumType + 1) as usize] = [
    b"System\0".as_ptr() as PCSTR,
    b"CentralProcessor\0".as_ptr() as PCSTR,
    b"FloatingPointProcessor\0".as_ptr() as PCSTR,
    b"PrimaryICache\0".as_ptr() as PCSTR,
    b"PrimaryDCache\0".as_ptr() as PCSTR,
    b"SecondaryICache\0".as_ptr() as PCSTR,
    b"SecondaryDCache\0".as_ptr() as PCSTR,
    b"SecondaryCache\0".as_ptr() as PCSTR,
    b"EisaAdapter\0".as_ptr() as PCSTR,
    b"TcAdapter\0".as_ptr() as PCSTR,
    b"ScsiAdapter\0".as_ptr() as PCSTR,
    b"DtiAdapter\0".as_ptr() as PCSTR,
    b"MultifunctionAdapter\0".as_ptr() as PCSTR,
    b"DiskController\0".as_ptr() as PCSTR,
    b"TapeController\0".as_ptr() as PCSTR,
    b"CdRomController\0".as_ptr() as PCSTR,
    b"WormController\0".as_ptr() as PCSTR,
    b"SerialController\0".as_ptr() as PCSTR,
    b"NetworkController\0".as_ptr() as PCSTR,
    b"DisplayController\0".as_ptr() as PCSTR,
    b"ParallelController\0".as_ptr() as PCSTR,
    b"PointerController\0".as_ptr() as PCSTR,
    b"KeyboardController\0".as_ptr() as PCSTR,
    b"AudioController\0".as_ptr() as PCSTR,
    b"OtherController\0".as_ptr() as PCSTR,
    b"DiskPeripheral\0".as_ptr() as PCSTR,
    b"FloppyDiskPeripheral\0".as_ptr() as PCSTR,
    b"TapePeripheral\0".as_ptr() as PCSTR,
    b"ModemPeripheral\0".as_ptr() as PCSTR,
    b"MonitorPeripheral\0".as_ptr() as PCSTR,
    b"PrinterPeripheral\0".as_ptr() as PCSTR,
    b"PointerPeripheral\0".as_ptr() as PCSTR,
    b"KeyboardPeripheral\0".as_ptr() as PCSTR,
    b"TerminalPeripheral\0".as_ptr() as PCSTR,
    b"OtherPeripheral\0".as_ptr() as PCSTR,
    b"LinePeripheral\0".as_ptr() as PCSTR,
    b"NetworkPeripheral\0".as_ptr() as PCSTR,
    b"SystemMemory\0".as_ptr() as PCSTR,
    b"DockingInformation\0".as_ptr() as PCSTR,
    b"RealModeIrqRoutingTable\0".as_ptr() as PCSTR,
    b"RealModePCIEnumeration\0".as_ptr() as PCSTR,
    b"Undefined\0".as_ptr() as PCSTR,
];

pub static mut FldrArcHwTreeRoot: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
pub static mut reactos_disk_count: ULONG = 0;
// reactos_arc_disk_info — C: `ARC_DISK_SIGNATURE_EX reactos_arc_disk_info[32];`
// (BSS zero-init). Use const-block MaybeUninit::zeroed for a const initializer
// (mem::zeroed itself isn't const-stable for non-trivial types).
pub static mut reactos_arc_disk_info: [ARC_DISK_SIGNATURE_EX; 32] =
    [const { unsafe { core::mem::MaybeUninit::<ARC_DISK_SIGNATURE_EX>::zeroed().assume_init() } }; 32];

// Pool tags `'DCwH'` / `'mNwH'` (C). archwsup.c uses TAG_HW_COMPONENT_DATA
// and TAG_HW_NAME via `'XXXX'` multi-char literal. Rust char literal can't
// hold 4 bytes — use byte-string + `from_le_bytes` (matches hwide.rs's
// `*b"dATA"` // 'dATA' convention).
pub const TAG_HW_COMPONENT_DATA: u32 = u32::from_le_bytes(*b"DCwH");
pub const TAG_HW_NAME: u32 = u32::from_le_bytes(*b"mNwH");

#[no_mangle]
pub unsafe extern "C" fn AddReactOSArcDiskInfo(
    ArcName: PSTR,
    // The disk GUID of a GPT disk, else NULL (upstream 7779fb44597). Without it IsGpt stayed
    // 0 and the kernel's IopVerifyDiskSignature could never match a GPT boot disk: 0x7B.
    GptDiskGuid: *const core::ffi::c_void,
    Signature: u32,
    Checksum: u32,
    ValidPartitionTable: u8,
) {

    ntkrnl_sys::ASSERT!((reactos_disk_count as usize) < 32);

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(reactos_arc_disk_info[reactos_disk_count as usize]),
        0,
        1,
    );
    reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.Signature = Signature;
    reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.CheckSum = Checksum;
    reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.ValidPartitionTable = ValidPartitionTable;
    reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.IsGpt = (!GptDiskGuid.is_null()) as u8;
    if !GptDiskGuid.is_null() {
        let Dst = &mut reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.GptSignature;
        core::ptr::copy_nonoverlapping(GptDiskGuid as *const u8, Dst.as_mut_ptr() as *mut u8, Dst.len());
    }

    strcpy(
        reactos_arc_disk_info[reactos_disk_count as usize].ArcName.as_mut_ptr(),
        ArcName,
    );
    reactos_arc_disk_info[reactos_disk_count as usize].DiskSignature.ArcName =
        reactos_arc_disk_info[reactos_disk_count as usize].ArcName.as_mut_ptr();

    reactos_disk_count += 1;
}

unsafe fn FldrSetIdentifier(
    Component: PCONFIGURATION_COMPONENT,
    IdentifierString: PCSTR,
) {
    let IdentifierLength: usize;
    let Identifier: PCHAR;

    IdentifierLength = strlen(IdentifierString) + 1;
    Identifier = FrLdrHeapAlloc(IdentifierLength, TAG_HW_NAME) as PCHAR;
    if Identifier.is_null() {
        return;
    }

    RtlCopyMemory(Identifier as *mut core::ffi::c_void, IdentifierString as *const core::ffi::c_void, IdentifierLength);

    (*Component).IdentifierLength = IdentifierLength as u32;
    (*Component).Identifier = Identifier;
}

#[no_mangle]
pub unsafe extern "C" fn FldrSetConfigurationData(
    ComponentData: PCONFIGURATION_COMPONENT_DATA,
    ResourceList: PCM_PARTIAL_RESOURCE_LIST,
    Size: u32,
) {
    (*ComponentData).ConfigurationData = ResourceList as *mut core::ffi::c_void;
    (*ComponentData).ComponentEntry.ConfigurationDataLength = Size;
}

#[no_mangle]
pub unsafe extern "C" fn FldrCreateSystemKey(
    SystemNode: *mut PCONFIGURATION_COMPONENT_DATA,
    IdentifierString: PCSTR,
) {
    let Component: *mut CONFIGURATION_COMPONENT;

    FldrArcHwTreeRoot = FrLdrHeapAlloc(
        core::mem::size_of::<CONFIGURATION_COMPONENT_DATA>(),
        TAG_HW_COMPONENT_DATA
    ) as PCONFIGURATION_COMPONENT_DATA;
    if FldrArcHwTreeRoot.is_null() {
        return;
    }

    Component = core::ptr::addr_of_mut!((*FldrArcHwTreeRoot).ComponentEntry);
    (*Component).Class = SystemClass;
    (*Component).Type = MaximumType;
    (*Component).ConfigurationDataLength = 0;
    (*Component).Identifier = core::ptr::null_mut();
    (*Component).IdentifierLength = 0;
    (*Component).Flags = 0;
    (*Component).Version = 0;
    (*Component).Revision = 0;
    (*Component).Key = 0;
    (*Component).AffinityMask = 0xFFFFFFFF;

    if !IdentifierString.is_null() {
        FldrSetIdentifier(Component, IdentifierString);
    }

    *SystemNode = FldrArcHwTreeRoot;
}

unsafe fn FldrLinkToParent(
    Parent: PCONFIGURATION_COMPONENT_DATA,
    Child: PCONFIGURATION_COMPONENT_DATA)
{
    let mut Sibling: PCONFIGURATION_COMPONENT_DATA;

    Sibling = (*Parent).Child;

    if Sibling.is_null()
    {
        (*Parent).Child = Child;
    }
    else
    {
        let mut Parent = Parent;
        loop
        {
            Parent = Sibling;
            Sibling = (*Sibling).Sibling;
            if Sibling.is_null() {
                break;
            }
        }

        (*Parent).Sibling = Child;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FldrCreateComponentKey(
    SystemNode: PCONFIGURATION_COMPONENT_DATA,
    Class: CONFIGURATION_CLASS,
    Type: CONFIGURATION_TYPE,
    Flags: IDENTIFIER_FLAG,
    Key: u32,
    Affinity: u32,
    IdentifierString: PCSTR,
    ResourceList: PCM_PARTIAL_RESOURCE_LIST,
    Size: u32,
    ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA,
) {
    let ComponentData: PCONFIGURATION_COMPONENT_DATA;
    let Component: PCONFIGURATION_COMPONENT;

    ComponentData = FrLdrHeapAlloc(
        core::mem::size_of::<CONFIGURATION_COMPONENT_DATA>(),
        TAG_HW_COMPONENT_DATA,
    ) as PCONFIGURATION_COMPONENT_DATA;
    if ComponentData.is_null() {
        return;
    }

    (*ComponentData).Parent = SystemNode;

    if !SystemNode.is_null() {
        FldrLinkToParent(SystemNode, ComponentData);
    }

    Component = core::ptr::addr_of_mut!((*ComponentData).ComponentEntry);
    (*Component).Class = Class;
    (*Component).Type = Type;
    (*Component).Flags = Flags;
    (*Component).Key = Key;
    (*Component).AffinityMask = Affinity;

    if !IdentifierString.is_null() {
        FldrSetIdentifier(Component, IdentifierString);
    }

    if !ResourceList.is_null() {
        FldrSetConfigurationData(ComponentData, ResourceList, Size);
    }

    TRACE(
        b"Created key: %s\\%d\n\0".as_ptr() as PCSTR, (ArcTypes[min(Type as usize, MaximumType as usize)]) as usize, (Key) as usize,
    );

    *ComponentKey = ComponentData;
}

#[no_mangle]
pub unsafe extern "C" fn ArcGetDiskCount() -> ULONG {
    reactos_disk_count
}

#[no_mangle]
pub unsafe extern "C" fn ArcGetDiskInfo(Index: u32) -> *mut ARC_DISK_SIGNATURE_EX {
    if Index >= reactos_disk_count {
        return core::ptr::null_mut();
    }

    return core::ptr::addr_of_mut!(reactos_arc_disk_info[Index as usize]);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Boot Loader (FreeLDR)
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            boot/freeldr/freeldr/arch/archwsup.c
//  * PURPOSE:         Routines for ARC Hardware Tree and Configuration Data
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
// /* GLOBALS ********************************************************************/
//
// /* Strings corresponding to CONFIGURATION_TYPE */
// const PCSTR ArcTypes[MaximumType + 1] = // CmTypeName
// {
//     "System",
//     "CentralProcessor",
//     "FloatingPointProcessor",
//     "PrimaryICache",
//     "PrimaryDCache",
//     "SecondaryICache",
//     "SecondaryDCache",
//     "SecondaryCache",
//     "EisaAdapter",
//     "TcAdapter",
//     "ScsiAdapter",
//     "DtiAdapter",
//     "MultifunctionAdapter",
//     "DiskController",
//     "TapeController",
//     "CdRomController",
//     "WormController",
//     "SerialController",
//     "NetworkController",
//     "DisplayController",
//     "ParallelController",
//     "PointerController",
//     "KeyboardController",
//     "AudioController",
//     "OtherController",
//     "DiskPeripheral",
//     "FloppyDiskPeripheral",
//     "TapePeripheral",
//     "ModemPeripheral",
//     "MonitorPeripheral",
//     "PrinterPeripheral",
//     "PointerPeripheral",
//     "KeyboardPeripheral",
//     "TerminalPeripheral",
//     "OtherPeripheral",
//     "LinePeripheral",
//     "NetworkPeripheral",
//     "SystemMemory",
//     "DockingInformation",
//     "RealModeIrqRoutingTable",
//     "RealModePCIEnumeration",
//     "Undefined"
// };
//
// PCONFIGURATION_COMPONENT_DATA FldrArcHwTreeRoot;
//
// // ARC Disk Information
// ULONG reactos_disk_count = 0;
// ARC_DISK_SIGNATURE_EX reactos_arc_disk_info[32];
//
// /* FUNCTIONS ******************************************************************/
//
// #define TAG_HW_COMPONENT_DATA   'DCwH'
// #define TAG_HW_NAME             'mNwH'
//
// VOID
// AddReactOSArcDiskInfo(
//     IN PSTR ArcName,
//     IN ULONG Signature,
//     IN ULONG Checksum,
//     IN BOOLEAN ValidPartitionTable)
// {
//     ASSERT(reactos_disk_count < sizeof(reactos_arc_disk_info)/sizeof(reactos_arc_disk_info[0]));
//
//     /* Fill out the ARC disk block */
//
//     reactos_arc_disk_info[reactos_disk_count].DiskSignature.Signature = Signature;
//     reactos_arc_disk_info[reactos_disk_count].DiskSignature.CheckSum = Checksum;
//     reactos_arc_disk_info[reactos_disk_count].DiskSignature.ValidPartitionTable = ValidPartitionTable;
//
//     strcpy(reactos_arc_disk_info[reactos_disk_count].ArcName, ArcName);
//     reactos_arc_disk_info[reactos_disk_count].DiskSignature.ArcName =
//         reactos_arc_disk_info[reactos_disk_count].ArcName;
//
//     reactos_disk_count++;
// }
//
// //
// // ARC Component Configuration Routines
// //
//
// static VOID
// FldrSetIdentifier(
//     _In_ PCONFIGURATION_COMPONENT Component,
//     _In_ PCSTR IdentifierString)
// {
//     SIZE_T IdentifierLength;
//     PCHAR Identifier;
//
//     /* Allocate memory for the identifier */
//     IdentifierLength = strlen(IdentifierString) + 1;
//     Identifier = FrLdrHeapAlloc(IdentifierLength, TAG_HW_NAME);
//     if (!Identifier) return;
//
//     /* Copy the identifier */
//     RtlCopyMemory(Identifier, IdentifierString, IdentifierLength);
//
//     /* Set component information */
//     Component->IdentifierLength = (ULONG)IdentifierLength;
//     Component->Identifier = Identifier;
// }
//
// VOID
// FldrSetConfigurationData(
//     _Inout_ PCONFIGURATION_COMPONENT_DATA ComponentData,
//     _In_ PCM_PARTIAL_RESOURCE_LIST ResourceList,
//     _In_ ULONG Size)
// {
//     /* Set component information */
//     ComponentData->ConfigurationData = ResourceList;
//     ComponentData->ComponentEntry.ConfigurationDataLength = Size;
// }
//
// VOID
// FldrCreateSystemKey(
//     _Out_ PCONFIGURATION_COMPONENT_DATA* SystemNode,
//     _In_ PCSTR IdentifierString)
// {
//     PCONFIGURATION_COMPONENT Component;
//
//     /* Allocate the root */
//     FldrArcHwTreeRoot = FrLdrHeapAlloc(sizeof(CONFIGURATION_COMPONENT_DATA),
//                                        TAG_HW_COMPONENT_DATA);
//     if (!FldrArcHwTreeRoot) return;
//
//     /* Set it up */
//     Component = &FldrArcHwTreeRoot->ComponentEntry;
//     Component->Class = SystemClass;
//     Component->Type = MaximumType;
//     Component->ConfigurationDataLength = 0;
//     Component->Identifier = 0;
//     Component->IdentifierLength = 0;
//     Component->Flags = 0;
//     Component->Version = 0;
//     Component->Revision = 0;
//     Component->Key = 0;
//     Component->AffinityMask = 0xFFFFFFFF;
//
//     /* Set identifier */
//     if (IdentifierString)
//         FldrSetIdentifier(Component, IdentifierString);
//
//     /* Return the node */
//     *SystemNode = FldrArcHwTreeRoot;
// }
//
// static VOID
// FldrLinkToParent(
//     _In_ PCONFIGURATION_COMPONENT_DATA Parent,
//     _In_ PCONFIGURATION_COMPONENT_DATA Child)
// {
//     PCONFIGURATION_COMPONENT_DATA Sibling;
//
//     /* Get the first sibling */
//     Sibling = Parent->Child;
//
//     /* If no sibling exists, then we are the first child */
//     if (!Sibling)
//     {
//         /* Link us in */
//         Parent->Child = Child;
//     }
//     else
//     {
//         /* Loop each sibling */
//         do
//         {
//             /* This is now the parent */
//             Parent = Sibling;
//         } while ((Sibling = Sibling->Sibling));
//
//         /* Found the lowest sibling; mark us as its sibling too */
//         Parent->Sibling = Child;
//     }
// }
//
// VOID
// FldrCreateComponentKey(
//     _In_ PCONFIGURATION_COMPONENT_DATA SystemNode,
//     _In_ CONFIGURATION_CLASS Class,
//     _In_ CONFIGURATION_TYPE Type,
//     _In_ IDENTIFIER_FLAG Flags,
//     _In_ ULONG Key,
//     _In_ ULONG Affinity,
//     _In_ PCSTR IdentifierString,
//     _In_ PCM_PARTIAL_RESOURCE_LIST ResourceList,
//     _In_ ULONG Size,
//     _Out_ PCONFIGURATION_COMPONENT_DATA* ComponentKey)
// {
//     PCONFIGURATION_COMPONENT_DATA ComponentData;
//     PCONFIGURATION_COMPONENT Component;
//
//     /* Allocate the node for this component */
//     ComponentData = FrLdrHeapAlloc(sizeof(CONFIGURATION_COMPONENT_DATA),
//                                    TAG_HW_COMPONENT_DATA);
//     if (!ComponentData) return;
//
//     /* Now save our parent */
//     ComponentData->Parent = SystemNode;
//
//     /* Link us to the parent */
//     if (SystemNode)
//         FldrLinkToParent(SystemNode, ComponentData);
//
//     /* Set us up */
//     Component = &ComponentData->ComponentEntry;
//     Component->Class = Class;
//     Component->Type = Type;
//     Component->Flags = Flags;
//     Component->Key = Key;
//     Component->AffinityMask = Affinity;
//
//     /* Set identifier */
//     if (IdentifierString)
//         FldrSetIdentifier(Component, IdentifierString);
//
//     /* Set configuration data */
//     if (ResourceList)
//         FldrSetConfigurationData(ComponentData, ResourceList, Size);
//
//     TRACE("Created key: %s\\%d\n", (ArcTypes[min(Type, MaximumType)]) as usize, (Key) as usize);
//
//     /* Return the child */
//     *ComponentKey = ComponentData;
// }
//
// ULONG ArcGetDiskCount(VOID)
// {
//     return reactos_disk_count;
// }
//
// PARC_DISK_SIGNATURE_EX ArcGetDiskInfo(ULONG Index)
// {
//     if (Index >= reactos_disk_count)
//     {
//         return NULL;
//     }
//
//     return &reactos_arc_disk_info[Index];
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
