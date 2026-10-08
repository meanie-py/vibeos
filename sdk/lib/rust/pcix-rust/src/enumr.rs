#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, DEVICE_RELATIONS, ExIsProcessorFeaturePresent, _KWAIT_REASON_Executive, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_REQUIREMENTS_LIST, KeEnterCriticalRegion, KeIpiGenericCall, KeLeaveCriticalRegion, KeWaitForSingleObject, PCI_AGP_CAPABILITY, PCI_CAPABILITIES_HEADER, PCI_COMMON_HEADER, PCI_PM_CAPABILITY, PCI_SLOT_NUMBER, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_REQUIREMENTS_LIST, PPCI_COMMON_HEADER, RtlCompareMemory};

// Enum constants (bindgen prefixes them; C uses unprefixed names).
use ntkrnl_sys::ddk::INTERFACE_TYPE;
// InitSafeBootMode is an ntoskrnl data export; the bindgen substrate emits a
// plain `extern { static }` that references `_InitSafeBootMode` directly, but the
// import lib only provides `__imp__InitSafeBootMode`. Shadow it with a raw-dylib
// data import so the `__imp_` thunk is generated.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    // ULONG in ntoskrnl/ex/init.c — BOOLEAN under-read was LE-benign but wrong
    static InitSafeBootMode: ULONG;
}
// L"(NULL)"
static NULL_DESCRIPTION: [u16; 7] = [
    b'(' as u16, b'N' as u16, b'U' as u16, b'L' as u16, b'L' as u16, b')' as u16, 0,
];
const PCIBus: INTERFACE_TYPE = 5;
const PowerDeviceUnspecified: ntkrnl_sys::ddk::DEVICE_POWER_STATE = 0;
const PowerDeviceD0: ntkrnl_sys::ddk::DEVICE_POWER_STATE = 1;
const PowerDeviceD1: ntkrnl_sys::ddk::DEVICE_POWER_STATE = 2;
const PowerDeviceD2: ntkrnl_sys::ddk::DEVICE_POWER_STATE = 3;
const PowerDeviceD3: ntkrnl_sys::ddk::DEVICE_POWER_STATE = 4;
const CmResourceShareShared: u8 = 3;
const CmResourceTypeDevicePrivate: u32 = 129;
// FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) == 0x40
const PCI_COMMON_HDR_LENGTH: u32 = 0x40; // nolint: magic-offset (PCI config-space header length, protocol constant)
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{CM_FULL_RESOURCE_DESCRIPTOR, _DEVICE_RELATIONS, _IO_RESOURCE_DESCRIPTOR, _IO_RESOURCE_REQUIREMENTS_LIST, _PCI_COMMON_HEADER};

use crate::glue::{CmResourceTypeInterrupt, PCI_CAPABILITY_ID_AGP};
use crate::glue::{CmResourceTypeMemoryLarge, CmResourceShareDeviceExclusive, CmResourceShareDriverExclusive};
use ntkrnl_sys::ddk::RtlCmDecodeMemIoResource;
use crate::types::{PCI_HACK_FIXUP_AFTER_CONFIGURATION, PCI_HACK_FIXUP_BEFORE_CONFIGURATION, PCI_HACK_FIXUP_BEFORE_UPDATE};
// PCI macros from pci.h / iotypes.h, inlined as helpers.
const PCI_MULTIFUNCTION: u8 = 0x80;
#[inline(always)]
unsafe fn PCI_IS_ROOT_FDO(x: *mut _PCI_FDO_EXTENSION) -> u8 {
    if (*x).BusRootFdoExtension == x { 1 } else { 0 }
}
#[inline(always)]
unsafe fn PCI_CONFIGURATION_TYPE(PciData: PPCI_COMMON_HEADER) -> u8 {
    (*PciData).HeaderType & !PCI_MULTIFUNCTION
}
#[inline(always)]
unsafe fn PCI_MULTIFUNCTION_DEVICE(PciData: PPCI_COMMON_HEADER) -> u8 {
    (((*PciData).HeaderType & PCI_MULTIFUNCTION) != 0) as u8
}
// UNIMPLEMENTED macro (debug-build no-op marker).
macro_rules! UNIMPLEMENTED { () => {{}}; }

extern "C" {
}
extern "C" {
    fn DbgPrint( format: *const i8, ... );
}
unsafe extern "system" {
    fn PciWriteDeviceConfig( PdoExtension: PPCI_PDO_EXTENSION, PciData: PPCI_COMMON_HEADER, Offset: u32, Length: u32, );
}
unsafe extern "system" {
    fn PciDecodeEnable( PdoExtension: PPCI_PDO_EXTENSION, Enable: BOOLEAN, CommandEnables: *mut u16, );
}
unsafe extern "system" {
    fn PciReadDeviceConfig( pdo: PPCI_PDO_EXTENSION, buffer: *mut core::ffi::c_void, offset: u32, length: u32, );
}

extern "system" {
    fn KdDisableDebugger() -> NTSTATUS;
    fn KdEnableDebugger() -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut PciZeroIoResourceRequirements: PIO_RESOURCE_REQUIREMENTS_LIST = core::ptr::null_mut();

pub static PciConfigurators: [PCI_CONFIGURATOR; 3] = [
    PCI_CONFIGURATOR {
        Initialize: Some(Device_MassageHeaderForLimitsDetermination),
        RestoreCurrent: Some(Device_RestoreCurrent),
        SaveLimits: Some(Device_SaveLimits),
        SaveCurrentSettings: Some(Device_SaveCurrentSettings),
        ChangeResourceSettings: Some(Device_ChangeResourceSettings),
        GetAdditionalResourceDescriptors: Some(Device_GetAdditionalResourceDescriptors),
        ResetDevice: Some(Device_ResetDevice),
    },
    PCI_CONFIGURATOR {
        Initialize: Some(PPBridge_MassageHeaderForLimitsDetermination),
        RestoreCurrent: Some(PPBridge_RestoreCurrent),
        SaveLimits: Some(PPBridge_SaveLimits),
        SaveCurrentSettings: Some(PPBridge_SaveCurrentSettings),
        ChangeResourceSettings: Some(PPBridge_ChangeResourceSettings),
        GetAdditionalResourceDescriptors: Some(PPBridge_GetAdditionalResourceDescriptors),
        ResetDevice: Some(PPBridge_ResetDevice),
    },
    PCI_CONFIGURATOR {
        Initialize: Some(Cardbus_MassageHeaderForLimitsDetermination),
        RestoreCurrent: Some(Cardbus_RestoreCurrent),
        SaveLimits: Some(Cardbus_SaveLimits),
        SaveCurrentSettings: Some(Cardbus_SaveCurrentSettings),
        ChangeResourceSettings: Some(Cardbus_ChangeResourceSettings),
        GetAdditionalResourceDescriptors: Some(Cardbus_GetAdditionalResourceDescriptors),
        ResetDevice: Some(Cardbus_ResetDevice),
    },
];

#[no_mangle]
pub unsafe extern "system" fn PciComputeNewCurrentSettings(
    PdoExtension: PPCI_PDO_EXTENSION,
    ResourceList: *mut CM_RESOURCE_LIST,
) -> u8 {
    let mut Partial: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut InterruptResource: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut CurrentDescriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut ResourceArray: [CM_PARTIAL_RESOURCE_DESCRIPTOR; 7] = core::mem::zeroed();
    let mut BarIndex: u32;
    let mut FullList: *mut CM_FULL_RESOURCE_DESCRIPTOR;
    let mut DrainPartial: u32;
    let mut RangeChange: u8 = 0;
    let mut i: u32;
    let mut j: u32;
    let PciResources: *mut PCI_FUNCTION_RESOURCES;

    /* ASSERT((ResourceList == NULL) || (ResourceList->Count == 1)); omitted */

    Partial = core::ptr::null_mut();
    InterruptResource = core::ptr::null_mut();
    BarIndex = 0;

    /* The limits say which BARs this function implements, so the walk below
     * can tell which BAR each assigned descriptor belongs to */
    PciResources = (*PdoExtension).Resources;

    if ResourceList.is_null() || (*ResourceList).Count == 0 {
        return (*PdoExtension).UpdateHardware;
    }

    PciDebugPrintCmResList(ResourceList);

    for i in 0..7 {
        (*ResourceArray.as_mut_ptr().add(i as usize)).Type = CmResourceTypeNull as u8;
    }

    FullList = (*ResourceList).List.as_mut_ptr();
    i = 0;
    while i < (*ResourceList).Count {
        DrainPartial = 0;

        Partial = (*FullList).PartialResourceList.PartialDescriptors.as_mut_ptr();
        j = 0;
        while j < (*FullList).PartialResourceList.Count {
            if DrainPartial != 0 {
                DrainPartial = DrainPartial.wrapping_sub(1);
                j = j.wrapping_add(1);
                Partial = CmiGetNextPartialDescriptor(Partial);
                continue;
            }

            match (*Partial).Type as u32 {
                CmResourceTypePort | CmResourceTypeMemory | CmResourceTypeMemoryLarge => {
                    /*
                     * Skip legacy/shared resources (e.g., VGA ports 0x3B0-0x3DF,
                     * memory 0xA0000): they do not belong to a BAR, so they are
                     * not placed in the ResourceArray.
                     */
                    if (*Partial).ShareDisposition == CmResourceShareDeviceExclusive
                        || (*Partial).ShareDisposition == CmResourceShareDriverExclusive
                    {
                        /*
                         * The assigned descriptors come back in the order the
                         * requirements list asked for them, so walking the limits
                         * in that same order says which BAR this one satisfies.
                         */
                        if !PciResources.is_null() {
                            while (BarIndex as usize) < ResourceArray.len()
                                && (*PciResources).Limit[BarIndex as usize].Type
                                    == CmResourceTypeNull as u8
                            {
                                BarIndex += 1;
                            }

                            if (BarIndex as usize) < ResourceArray.len() {
                                ResourceArray[BarIndex as usize] = *Partial;
                                BarIndex += 1;
                            }
                        }
                    }
                }
                CmResourceTypeInterrupt => {
                    /* ASSERT(InterruptResource == NULL); omitted */
                    /* ASSERT(Partial->u.Interrupt.Level == Partial->u.Interrupt.Vector); omitted */
                    InterruptResource = Partial;

                    if (*Partial).u.Interrupt.Level < 256 {
                        (*PdoExtension).AdjustedInterruptLine = (*Partial).u.Interrupt.Level as u8;
                    } else {
                        (*PdoExtension).AdjustedInterruptLine = 0;
                    }
                }
                CmResourceTypeDevicePrivate => {
                    match (*Partial).u.DevicePrivate.Data[0] {
                        1 => {
                            UNIMPLEMENTED_DBGBREAK();
                        }
                        2 => {
                            UNIMPLEMENTED_DBGBREAK();
                        }
                        3 => {
                            /* A drain request: skip the next Data[1] descriptors */
                            DrainPartial = (*Partial).u.DevicePrivate.Data[1];
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            j = j.wrapping_add(1);
            Partial = CmiGetNextPartialDescriptor(Partial);
        }

        FullList = Partial as *mut CM_FULL_RESOURCE_DESCRIPTOR;
        i = i.wrapping_add(1);
    }

    /* A function with no discovered resources has nothing to update */
    if PciResources.is_null() {
        return 0;
    }

    for i in 0..ResourceArray.len() {
        CurrentDescriptor = (*PciResources).Current.as_mut_ptr().add(i);
        Partial = ResourceArray.as_mut_ptr().add(i);

        if (*Partial).Type != (*CurrentDescriptor).Type
            || ((*Partial).Type != CmResourceTypeNull as u8
                && ((*Partial).u.Generic.Start.QuadPart
                    != (*CurrentDescriptor).u.Generic.Start.QuadPart
                    || RtlCmDecodeMemIoResource(Partial, core::ptr::null_mut())
                        != RtlCmDecodeMemIoResource(CurrentDescriptor, core::ptr::null_mut())))
        {
            RangeChange = 1;

            if (*CurrentDescriptor).Type != CmResourceTypeNull as u8 {
                DbgPrint((b"      Old range-\n\0".as_ptr() as *const u8 as *const i8).cast());
                PciDebugPrintPartialResource(CurrentDescriptor);
            } else {
                DbgPrint(
                    (b"      Previously unset range\n\0".as_ptr() as *const u8 as *const i8).cast(),
                );
            }

            DbgPrint((b"      changed to\n\0".as_ptr() as *const u8 as *const i8).cast());
            PciDebugPrintPartialResource(Partial);

            /* Update to new range */
            (*CurrentDescriptor).Type = (*Partial).Type;
            (*CurrentDescriptor).Flags = (*Partial).Flags;
            (*CurrentDescriptor).u.Generic.Start = (*Partial).u.Generic.Start;
            (*CurrentDescriptor).u.Generic.Length = (*Partial).u.Generic.Length;
        }
    }

    if RangeChange != 0 || (*PdoExtension).UpdateHardware != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn PcipUpdateHardware(
    Context: *mut core::ffi::c_void,
    Context2: *mut core::ffi::c_void,
) {
    let PdoExtension: PPCI_PDO_EXTENSION = Context as PPCI_PDO_EXTENSION;
    let PciData: PPCI_COMMON_HEADER = Context2 as PPCI_COMMON_HEADER;

    // Set command to current enables
    (*PciData).Command = (*PdoExtension).CommandEnables;
    if ((*PdoExtension).HackFlags & (PCI_HACK_PRESERVE_COMMAND as u64)) == 0 {
        // Disable all decodes
        let mask: u16 = !((PCI_ENABLE_IO_SPACE
            | PCI_ENABLE_MEMORY_SPACE
            | PCI_ENABLE_BUS_MASTER
            | PCI_ENABLE_WRITE_AND_INVALIDATE) as u16);
        (*PciData).Command &= mask;

        /* A BAR size can only change with the decodes off, before the BAR gets its address */
        if (*PdoExtension).ResizableBarState.CapabilityPtr != 0 {
            PciWriteDeviceConfig(
                PdoExtension,
                core::ptr::addr_of_mut!((*PciData).Command) as _,
                core::mem::offset_of!(PCI_COMMON_HEADER, Command) as ULONG,
                core::mem::size_of::<USHORT>() as ULONG,
            );
            PciApplyResizableBarSizes(PdoExtension);
        }
    }

    // Update device configuration
    (*PciData).Status = 0;
    PciWriteDeviceConfig(PdoExtension, PciData, 0, PCI_COMMON_HDR_LENGTH); // nolint: magic-offset (PCI config-space header length, protocol constant)

    // Turn decodes back on
    PciDecodeEnable(
        PdoExtension,
        1u8,
        core::ptr::addr_of_mut!((*PdoExtension).CommandEnables),
    );
}

#[no_mangle]
pub unsafe extern "system" fn PciUpdateHardware(
    PdoExtension: *mut PCI_PDO_EXTENSION,
    PciData: *mut PCI_COMMON_HEADER,
) {
    let mut context: PCI_IPI_CONTEXT = core::mem::zeroed();
    if ((*PdoExtension).HackFlags & (PCI_HACK_CRITICAL_DEVICE as u64)) != 0 || ((*PdoExtension).OnDebugPath != 0) {
        context.RunCount = 1;
        context.Barrier = 1;
        context.Context = PciData as *mut core::ffi::c_void;
        context.Function =
            Some(core::mem::transmute::<*const (), _>(PcipUpdateHardware as *const ()));
        context.DeviceExtension = PdoExtension as *mut core::ffi::c_void;
        KeIpiGenericCall(Some(PciExecuteCriticalSystemRoutine), core::ptr::addr_of_mut!(context) as usize as _);
    } else {
        PcipUpdateHardware(PdoExtension as *mut core::ffi::c_void, PciData as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciAllocateIoRequirementsList(Count: u32, BusNumber: u32, SlotNumber: u32) -> PIO_RESOURCE_REQUIREMENTS_LIST {
    let mut size: usize = core::mem::size_of::<_IO_RESOURCE_REQUIREMENTS_LIST>();
    if Count > 1 {
        size = core::mem::size_of::<_IO_RESOURCE_DESCRIPTOR>() * ((Count as usize) - 1)
             + core::mem::size_of::<_IO_RESOURCE_REQUIREMENTS_LIST>();
    }

    let requirements_list = ExAllocatePoolWithTag(PagedPool, size, u32::from_le_bytes(*b"BicP")) as PIO_RESOURCE_REQUIREMENTS_LIST;

    if requirements_list.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(requirements_list as *mut c_void, size);

    (*requirements_list).AlternativeLists = 1;
    (*requirements_list).BusNumber = BusNumber;
    (*requirements_list).SlotNumber = SlotNumber;
    (*requirements_list).InterfaceType = PCIBus;
    (*requirements_list).ListSize = size as u32;
    (*requirements_list).List[0].Count = Count;
    (*requirements_list).List[0].Version = 1u16;
    (*requirements_list).List[0].Revision = 1u16;

    requirements_list
}
#[no_mangle]
pub unsafe extern "system" fn PciAllocateCmResourceList(Count: u32, BusNumber: u32) -> *mut CM_RESOURCE_LIST {
    let mut Size = core::mem::size_of::<CM_RESOURCE_LIST>();
    if Count > 1 {
        Size = core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()
            .wrapping_mul((Count as usize).wrapping_sub(1))
            .wrapping_add(core::mem::size_of::<CM_RESOURCE_LIST>());
    }

    let ResourceList = ExAllocatePoolWithTag(PagedPool, Size, u32::from_be_bytes(*b"BicP")) as *mut CM_RESOURCE_LIST;
    if ResourceList.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(ResourceList as *mut u8, 0, Size);
    (*ResourceList).Count = 1;
    (*ResourceList).List[0].BusNumber = BusNumber;
    (*ResourceList).List[0].InterfaceType = PCIBus;
    (*ResourceList).List[0].PartialResourceList.Version = 1;
    (*ResourceList).List[0].PartialResourceList.Revision = 1;
    (*ResourceList).List[0].PartialResourceList.Count = Count;

    ResourceList
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryResources(
    PdoExtension: PPCI_PDO_EXTENSION,
    Buffer: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: u32 = 0x0000;

    let PciResources: PPCI_FUNCTION_RESOURCES;
    let mut HaveVga: BOOLEAN;
    let HaveMemSpace: BOOLEAN;
    let HaveIoSpace: BOOLEAN;
    let mut BridgeControl: USHORT = 0;
    let mut PciCommand: USHORT = 0;
    let mut Count: ULONG;
    let mut i: ULONG;
    let mut Partial: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let LastResource: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let ResourceList: PCM_RESOURCE_LIST;
    let mut InterruptLine: UCHAR;

    /* Assume failure */
    Count = 0;
    HaveVga = 0u8;
    *Buffer = core::ptr::null_mut();

    /* Make sure there's some resources to query */
    PciResources = (*PdoExtension).Resources;
    if PciResources.is_null() {
        return STATUS_SUCCESS;
    }

    /* Read the decodes */
    PciReadDeviceConfig(
        PdoExtension,
        core::ptr::addr_of_mut!(PciCommand) as *mut core::ffi::c_void,
        core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32,
        core::mem::size_of::<USHORT>() as u32,
    );

    /* Check which ones are turned on */
    HaveIoSpace = (PciCommand as u32 & PCI_ENABLE_IO_SPACE != 0) as BOOLEAN;
    HaveMemSpace = (PciCommand as u32 & PCI_ENABLE_MEMORY_SPACE != 0) as BOOLEAN;

    /* Loop maximum possible descriptors */
    i = 0;
    while i < 7 {
        /* Check if the decode for this descriptor is actually turned on */
        Partial = core::ptr::addr_of_mut!((*PciResources).Current[i as usize]);
        if (HaveMemSpace != 0
            && ((*Partial).Type as u32 == CmResourceTypeMemory
                || (*Partial).Type as u32 == CmResourceTypeMemoryLarge))
            || (HaveIoSpace != 0 && (*Partial).Type as u32 == CmResourceTypePort)
        {
            /* One more fully active descriptor */
            Count += 1;
        }
        i += 1;
    }

    /* If there's an interrupt pin associated, check at least one decode is on */
    if (*PdoExtension).InterruptPin != 0 && (HaveMemSpace != 0 || HaveIoSpace != 0) {
        /* Read the interrupt line for the pin, add a descriptor if it's valid */
        InterruptLine = (*PdoExtension).AdjustedInterruptLine;
        if InterruptLine != 0 && InterruptLine != 0xFF {
            Count += 1;
        }
    }

    /* Check for PCI bridge */
    if (*PdoExtension).HeaderType as u32 == PCI_BRIDGE_TYPE {
        /* Read bridge settings, check if VGA is present */
        PciReadDeviceConfig(
            PdoExtension,
            core::ptr::addr_of_mut!(BridgeControl) as *mut core::ffi::c_void,
            core::mem::offset_of!(PCI_COMMON_HEADER, u.type1.BridgeControl) as u32,
            core::mem::size_of::<USHORT>() as u32,
        );
        if BridgeControl as u32 & PCI_ENABLE_BRIDGE_VGA != 0 {
            /* Remember for later */
            HaveVga = 1u8;

            /* One memory descriptor for 0xA0000, plus the two I/O port ranges */
            if HaveMemSpace != 0 {
                Count += 1;
            }
            if HaveIoSpace != 0 {
                Count += 2;
            }
        }
    }

    /* If there's no descriptors in use, there's no resources, so return */
    if Count == 0 {
        return STATUS_SUCCESS;
    }

    /* Allocate a resource list to hold the resources */
    ResourceList = PciAllocateCmResourceList(
        Count,
        (*(*PdoExtension).ParentFdoExtension).BaseBus as u32,
    );
    if ResourceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* This is where the descriptors will be copied into */
    Resource = (*ResourceList).List[0]
        .PartialResourceList
        .PartialDescriptors
        .as_mut_ptr();
    LastResource = Resource.add((Count + 1) as usize);

    /* Loop maximum possible descriptors */
    i = 0;
    while i < 7 {
        /* Check if the decode for this descriptor is actually turned on */
        Partial = core::ptr::addr_of_mut!((*PciResources).Current[i as usize]);
        if (HaveMemSpace != 0
            && ((*Partial).Type as u32 == CmResourceTypeMemory
                || (*Partial).Type as u32 == CmResourceTypeMemoryLarge))
            || (HaveIoSpace != 0 && (*Partial).Type as u32 == CmResourceTypePort)
        {
            /* Copy the descriptor into the resource list */
            *Resource = *Partial;
            Resource = Resource.add(1);
        }
        i += 1;
    }

    /* Check if earlier the code detected this was a PCI bridge with VGA on it */
    if HaveVga != 0 {
        /* Are the memory decodes enabled? */
        if HaveMemSpace != 0 {
            /* Build a memory descriptor for a 128KB framebuffer at 0xA0000 */
            (*Resource).Flags = CM_RESOURCE_MEMORY_READ_WRITE as USHORT;
            (*Resource).u.Generic.Start.__bindgen_anon_1.HighPart = 0;
            (*Resource).Type = CmResourceTypeMemory as UCHAR;
            (*Resource).u.Generic.Start.__bindgen_anon_1.LowPart = 0xA0000;
            (*Resource).u.Generic.Length = 0x20000;
            Resource = Resource.add(1);
        }

        /* Are the I/O decodes enabled? */
        if HaveIoSpace != 0 {
            /* Build an I/O descriptor for the graphic ports at 0x3B0 */
            (*Resource).Type = CmResourceTypePort as UCHAR;
            (*Resource).Flags =
                (CM_RESOURCE_PORT_POSITIVE_DECODE | CM_RESOURCE_PORT_10_BIT_DECODE) as USHORT;
            (*Resource).u.Port.Start.QuadPart = 0x3B0;
            (*Resource).u.Port.Length = 0xC;
            Resource = Resource.add(1);

            /* Build an I/O descriptor for the graphic ports at 0x3C0 */
            (*Resource).Type = CmResourceTypePort as UCHAR;
            (*Resource).Flags =
                (CM_RESOURCE_PORT_POSITIVE_DECODE | CM_RESOURCE_PORT_10_BIT_DECODE) as USHORT;
            (*Resource).u.Port.Start.QuadPart = 0x3C0;
            (*Resource).u.Port.Length = 0x20;
            Resource = Resource.add(1);
        }
    }

    /* If there's an interrupt pin associated, check at least one decode is on */
    if (*PdoExtension).InterruptPin != 0 && (HaveMemSpace != 0 || HaveIoSpace != 0) {
        /* Read the interrupt line for the pin, check if it's valid */
        InterruptLine = (*PdoExtension).AdjustedInterruptLine;
        if InterruptLine != 0 && InterruptLine != 0xFF {
            let _ = LastResource;

            /* Add the interrupt descriptor */
            (*Resource).Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE as USHORT;
            (*Resource).Type = CmResourceTypeInterrupt as UCHAR;
            (*Resource).ShareDisposition = CmResourceShareShared;
            (*Resource).u.Interrupt.Affinity = !0;
            (*Resource).u.Interrupt.Level = InterruptLine as ULONG;
            (*Resource).u.Interrupt.Vector = InterruptLine as ULONG;
        }
    }

    /* Return the resource list */
    *Buffer = ResourceList;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryTargetDeviceRelations(
    PdoExtension: *mut _PCI_PDO_EXTENSION,
    pDeviceRelations: *mut PDEVICE_RELATIONS,
) -> NTSTATUS {
    if !(*pDeviceRelations).is_null() {
        ExFreePoolWithTag((*pDeviceRelations).cast(), 0);
    }

    let DeviceRelations = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_DEVICE_RELATIONS>(),
        0x50496342u32,
    ) as *mut _DEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1;
    let Pdo = (*PdoExtension).PhysicalDeviceObject;
    core::ptr::write((*DeviceRelations).Objects.as_mut_ptr(), Pdo);
    ObReferenceObject((Pdo).cast());

    *pDeviceRelations = (DeviceRelations).cast();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciQueryEjectionRelations(
    PdoExtension: PPCI_PDO_EXTENSION,
    pDeviceRelations: *mut *mut DEVICE_RELATIONS,
) -> NTSTATUS {
    let _ = PdoExtension;
    let _ = pDeviceRelations;

    UNIMPLEMENTED_DBGBREAK();
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn PciBuildRequirementsList(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
    Buffer: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: u16 = 0x0000; // xdk/cmtypes.h
    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST;
    let mut Descriptor: PIO_RESOURCE_DESCRIPTOR;
    let Resources = (*PdoExtension).Resources;
    let mut Count: u32;

    /* Count the BAR limits that resource discovery found for this function */
    Count = 0;
    if !Resources.is_null() {
        for i in 0..(PCI_TYPE0_ADDRESSES as usize + 1) {
            if (*Resources).Limit[i].Type == CmResourceTypeNull as u8 {
                continue;
            }

            /* A resizable BAR also asks for the larger sizes it can decode */
            Count += 1;
            Count += PciAddResizableBarRequirements(
                PdoExtension,
                i as ULONG,
                core::ptr::addr_of_mut!((*Resources).Limit[i]),
                core::ptr::null_mut(),
            );
        }
    }

    /* A device with an interrupt pin also needs a line routed to it */
    let HaveInterrupt = (*PdoExtension).InterruptPin != 0
        && ((*PdoExtension).HackFlags & (PCI_HACK_NO_ENUM_AT_ALL as u64)) == 0;
    if HaveInterrupt {
        Count += 1;
    }

    /* And a bridge with legacy decodes enabled needs those ranges locked down */
    Count += (*PdoExtension).AdditionalResourceCount as u32;

    /* Check if the function turned out to need nothing after all */
    if Count == 0 {
        /* There aren't, so use the zero descriptor */
        RequirementsList = PciZeroIoResourceRequirements;

        if RequirementsList.is_null() {
            RequirementsList = PciAllocateIoRequirementsList(0, 0, 0);
            PciZeroIoResourceRequirements = RequirementsList;
            if PciZeroIoResourceRequirements.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        *Buffer = RequirementsList;

        DbgPrint(
            b"PCI - build resource reqs - early out, 0 resources\n\0"
                .as_ptr() as *const i8,
        );

        return STATUS_SUCCESS;
    }

    RequirementsList = PciAllocateIoRequirementsList(
        Count,
        (*(*PdoExtension).ParentFdoExtension).BaseBus as u32,
        (*PdoExtension).Slot.u.AsULONG,
    );
    if RequirementsList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Descriptor = (*RequirementsList).List[0].Descriptors.as_mut_ptr();

    /* Emit one descriptor per BAR that decoded something during discovery */
    if !Resources.is_null() {
        for i in 0..(PCI_TYPE0_ADDRESSES as usize + 1) {
            /* Skip the BARs this function does not implement */
            if (*Resources).Limit[i].Type == CmResourceTypeNull as u8 {
                continue;
            }

            /* The larger sizes of a resizable BAR come first, its default size is the fallback */
            let Resized = PciAddResizableBarRequirements(
                PdoExtension,
                i as ULONG,
                core::ptr::addr_of_mut!((*Resources).Limit[i]),
                Descriptor,
            );
            Descriptor = Descriptor.add(Resized as usize);

            /* A BAR decodes for one function only, so it cannot be shared */
            *Descriptor = (*Resources).Limit[i];
            (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
            if Resized != 0 {
                (*Descriptor).Option = IO_RESOURCE_ALTERNATIVE as UCHAR;
            }
            Descriptor = Descriptor.add(1);
        }
    }

    if HaveInterrupt {
        (*Descriptor).Type = CmResourceTypeInterrupt as u8;
        (*Descriptor).ShareDisposition = CmResourceShareShared;
        (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE;
        (*Descriptor).u.Interrupt.MinimumVector = 0;
        (*Descriptor).u.Interrupt.MaximumVector = u32::MAX; // MAXULONG
        Descriptor = Descriptor.add(1);
    }

    /* And finally any extra ranges this kind of function decodes on the side */
    if (*PdoExtension).AdditionalResourceCount != 0 {
        let mut Context: PCI_CONFIGURATOR_CONTEXT = core::mem::zeroed();
        Context.PdoExtension = PdoExtension;
        Context.Current = PciData;
        // Every PciConfigurators row fills this slot (the C calls it unconditionally).
        if let Some(GetAdditional) =
            PciConfigurators[(*PdoExtension).HeaderType as usize].GetAdditionalResourceDescriptors
        {
            GetAdditional(core::ptr::addr_of_mut!(Context), PciData, Descriptor);
        }
    }

    *Buffer = RequirementsList;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciQueryRequirements(
    PdoExtension: PPCI_PDO_EXTENSION,
    RequirementsList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    let mut PciHeader: PCI_COMMON_HEADER = core::mem::zeroed();
    let Status: NTSTATUS;

    if !(*PdoExtension).Resources.is_null() || (*PdoExtension).InterruptPin != 0 {
        PciReadDeviceConfig(
            PdoExtension,
            (core::ptr::addr_of_mut!(PciHeader)).cast(),
            0,
            PCI_COMMON_HDR_LENGTH, // nolint: magic-offset (PCI config-space header length, protocol constant)
        );

        Status = PciBuildRequirementsList(
            PdoExtension,
            core::ptr::addr_of_mut!(PciHeader),
            RequirementsList,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if (PciHeader.VendorID == 0xE11)
            && (PciHeader.DeviceID == 0xA0F7)
            && (PciHeader.RevisionID == 17)
            && (ExIsProcessorFeaturePresent(PF_PAE_ENABLED) != 0)
        {
            UNIMPLEMENTED_DBGBREAK();
        }

        if *RequirementsList == PciZeroIoResourceRequirements {
            *RequirementsList = core::ptr::null_mut();
            DbgPrint(b"Returning NULL requirements list\n\0".as_ptr() as *const i8);
        } else {
            PciDebugPrintIoResReqList(*RequirementsList);
        }
    } else {
        DbgPrint(b"PciQueryRequirements returning NULL requirements list\n\0".as_ptr() as *const i8);
        *RequirementsList = core::ptr::null_mut();
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn PciConfigureIdeController(
    pdo_extension: PPCI_PDO_EXTENSION,
    pci_data: PPCI_COMMON_HEADER,
    initial: u8,
) -> u8 {

    let mut switched: u8 = 0u8;

    let prog_if = (*pci_data).ProgIf;
    let master_mode = (prog_if & 1u8) == 1u8;
    let master_fixed = (prog_if & 2u8) == 0u8;
    let slave_mode = (prog_if & 4u8) == 4u8;
    let slave_fixed = (prog_if & 8u8) == 0u8;

    if (master_mode != slave_mode) || (master_fixed != slave_fixed) {
        DbgPrint(
            b"PCI: Warning unsupported IDE controller configuration for VEN_%04x&DEV_%04x!\0"
                .as_ptr() as *const i8,
            (*pdo_extension).VendorId as u32,
            (*pdo_extension).DeviceId as u32,
        );
        return switched;
    }

    if master_mode && slave_mode {
        if initial != 0u8 || (*pdo_extension).IoSpaceUnderNativeIdeControl != 0u8 {
            let mut command: u16 = 0;
            let command_offset = core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32;
            PciReadDeviceConfig(
                pdo_extension,
                core::ptr::addr_of_mut!(command) as *mut core::ffi::c_void,
                command_offset,
                core::mem::size_of::<u16>() as u32,
            );

            command &= !(PCI_ENABLE_IO_SPACE as u16);
            PciWriteDeviceConfig(
                pdo_extension,
                (core::ptr::addr_of_mut!(command) as *mut core::ffi::c_void).cast(),
                command_offset,
                core::mem::size_of::<u16>() as u32,
            );
            (*pci_data).Command = command;
        }
        switched = 1u8;
    } else if !master_fixed
        && !slave_fixed
        && (*pdo_extension).BIOSAllowsIDESwitchToNativeMode != 0u8
        && ((*pdo_extension).HackFlags & (PCI_HACK_DISABLE_IDE_NATIVE_MODE as u64)) == 0
    {
        PciDecodeEnable(pdo_extension, 0u8, core::ptr::null_mut());

        let command_offset = core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32;
        PciReadDeviceConfig(
            pdo_extension,
            core::ptr::addr_of_mut!((*pci_data).Command) as *mut core::ffi::c_void,
            command_offset,
            core::mem::size_of::<u16>() as u32,
        );

        let mut new_prog_if: u8 = 0;
        let prog_if_offset = core::mem::offset_of!(PCI_COMMON_HEADER, ProgIf) as u32;
        let new_native_prog = prog_if | 5u8;
        PciWriteDeviceConfig(
            pdo_extension,
            (core::ptr::addr_of!(new_native_prog) as *mut core::ffi::c_void).cast(),
            prog_if_offset,
            core::mem::size_of::<u8>() as u32,
        );

        PciReadDeviceConfig(
            pdo_extension,
            core::ptr::addr_of_mut!(new_prog_if) as *mut core::ffi::c_void,
            prog_if_offset,
            core::mem::size_of::<u8>() as u32,
        );

        if new_prog_if == new_native_prog {
            (*pci_data).ProgIf = new_native_prog;
            (*pdo_extension).ProgIf = new_prog_if;

            let base0_offset = core::mem::offset_of!(PCI_COMMON_HEADER, u.type0.BaseAddresses) as u32;
            let size_of_4bars = (4 * core::mem::size_of::<u32>()) as u32;

            (*pci_data).u.type0.BaseAddresses[0] = 0;
            (*pci_data).u.type0.BaseAddresses[1] = 0;
            (*pci_data).u.type0.BaseAddresses[2] = 0;
            (*pci_data).u.type0.BaseAddresses[3] = 0;
            PciWriteDeviceConfig(
                pdo_extension,
                ((*pci_data).u.type0.BaseAddresses.as_mut_ptr() as *mut core::ffi::c_void).cast(),
                base0_offset,
                size_of_4bars,
            );

            PciReadDeviceConfig(
                pdo_extension,
                (*pci_data).u.type0.BaseAddresses.as_mut_ptr() as *mut core::ffi::c_void,
                base0_offset,
                size_of_4bars,
            );

            let intpin_offset = core::mem::offset_of!(PCI_COMMON_HEADER, u.type0.InterruptPin) as u32;
            PciReadDeviceConfig(
                pdo_extension,
                core::ptr::addr_of_mut!((*pci_data).u.type0.InterruptPin) as *mut core::ffi::c_void,
                intpin_offset,
                core::mem::size_of::<u8>() as u32,
            );

            switched = 1u8;
        } else {
            DbgPrint(
                b"PCI: Warning failed switch to native mode for IDE controller VEN_%04x&DEV_%04x!\0"
                    .as_ptr() as *const i8,
                (*pci_data).VendorID as u32,
                (*pci_data).DeviceID as u32,
            );
        }
    }

    switched
}
#[no_mangle]
pub unsafe extern "system" fn PciApplyHacks(
    DeviceExtension: PPCI_FDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
    SlotNumber: PCI_SLOT_NUMBER,
    OperationType: u32,
    PdoExtension: PPCI_PDO_EXTENSION,
) {
    let _ = SlotNumber;

    match OperationType {
        PCI_HACK_FIXUP_BEFORE_CONFIGURATION => {
            if (*PciData).VendorID == 0x8086u16
                && ((*PciData).DeviceID == 0x482u16 || (*PciData).DeviceID == 0x484u16)
            {
                (*PciData).SubClass = if (*PciData).DeviceID == 0x482u16 {
                    PCI_SUBCLASS_BR_EISA as u8
                } else {
                    PCI_SUBCLASS_BR_ISA as u8
                };
                (*PciData).BaseClass = PCI_CLASS_BRIDGE_DEV as u8;

                if !PdoExtension.is_null() {
                    (*(PdoExtension)).ExpectedWritebackFailure = 1u8;
                }
            }
            return;
        }

        PCI_HACK_FIXUP_AFTER_CONFIGURATION => {
            ntkrnl_sys::ASSERT!(!PdoExtension.is_null());

            if (*PciData).VendorID == 0x1045u16 && (*PciData).DeviceID != 0xC621u16 {
                (*PciData).ProgIf &= !5u8;
                (*PciData).u.type0.InterruptPin = 0u8;
                (*(PdoExtension)).ExpectedWritebackFailure = 1u8;
            } else if (*PciData).BaseClass == PCI_CLASS_MASS_STORAGE_CTLR as u8
                && (*PciData).SubClass == PCI_SUBCLASS_MSC_IDE_CTLR as u8
            {
                (*(PdoExtension)).BIOSAllowsIDESwitchToNativeMode = 0u8;

                if PciEnableNativeModeATA == 1u8
                    && InitSafeBootMode == 0u32
                    && PciIsSlotPresentInParentMethod(PdoExtension, u32::from_be_bytes([b'A', b'T', b'A', b'N'])) != 0
                {
                    (*(PdoExtension)).BIOSAllowsIDESwitchToNativeMode = 1u8;
                    (*(PdoExtension)).IDEInNativeMode =
                        PciConfigureIdeController(PdoExtension, PciData, 1u8);
                }

                if ((*PciData).ProgIf & 5u8) != 5u8 {
                    (*PciData).u.type0.InterruptPin = 0u8;
                }
            }

            if ((*(PdoExtension)).HackFlags & (PCI_HACK_VIDEO_LEGACY_DECODE as u64)) != 0
                && PCI_IS_ROOT_FDO(DeviceExtension) != 0
                && (*(DeviceExtension)).BrokenVideoHackApplied == 0u8
            {
                ario_ApplyBrokenVideoHack(DeviceExtension);
            }

            if (*PciData).VendorID == 0xE11u16
                && (*PciData).DeviceID == 0xA0F7u16
                && (*PciData).RevisionID == 17u8
                && ExIsProcessorFeaturePresent(PF_PAE_ENABLED) != 0
            {
                (*PciData).Command &= !(PCI_ENABLE_IO_SPACE
                    | PCI_ENABLE_MEMORY_SPACE
                    | PCI_ENABLE_BUS_MASTER) as u16;
                PciWriteDeviceConfig(
                    PdoExtension,
                    (core::ptr::addr_of_mut!((*PciData).Command)).cast(),
                    core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32,
                    core::mem::size_of::<u16>() as u32,
                );

                (*(PdoExtension)).CommandEnables &= !(PCI_ENABLE_IO_SPACE
                    | PCI_ENABLE_MEMORY_SPACE
                    | PCI_ENABLE_BUS_MASTER) as u16;
                (*(PdoExtension)).HackFlags |= (PCI_HACK_PRESERVE_COMMAND as u64);
            }
        }

        PCI_HACK_FIXUP_BEFORE_UPDATE => {
            ntkrnl_sys::ASSERT!(!PdoExtension.is_null());

            if (*(PdoExtension)).VendorId == 0x1014u16 && (*(PdoExtension)).DeviceId == 0x95u16 {
                let mut Command: u16 = 0;
                let mut RegValue: u8 = 0;

                PciReadDeviceConfig(
                    PdoExtension,
                    (core::ptr::addr_of_mut!(Command)).cast(),
                    core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32,
                    core::mem::size_of::<u16>() as u32,
                );

                PciDecodeEnable(PdoExtension, 0u8, core::ptr::addr_of_mut!(Command));

                PciReadDeviceConfig(PdoExtension, (core::ptr::addr_of_mut!(RegValue)).cast(), 0xE0u32, core::mem::size_of::<u8>() as u32);
                RegValue &= !2u8;
                RegValue |= 1u8;
                PciWriteDeviceConfig(PdoExtension, (core::ptr::addr_of_mut!(RegValue)).cast(), 0xE0u32, core::mem::size_of::<u8>() as u32);

                PciWriteDeviceConfig(
                    PdoExtension,
                    (core::ptr::addr_of_mut!(Command)).cast(),
                    core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32,
                    core::mem::size_of::<u16>() as u32,
                );
            }

            if (*(PdoExtension)).HeaderType == PCI_BRIDGE_TYPE as u8
                && (*(PdoExtension)).Dependent.type1.as_ref().SubtractiveDecode() != 0
                && (*(PdoExtension)).VendorId == 0x8086u16
                && ((*(PdoExtension)).DeviceId == 0x2418u16
                    || (*(PdoExtension)).DeviceId == 0x2428u16
                    || (*(PdoExtension)).DeviceId == 0x244Eu16
                    || (*(PdoExtension)).DeviceId == 0x2448u16)
                && ((*(PdoExtension)).HackFlags & (PCI_HACK_BROKEN_SUBTRACTIVE_DECODE as u64)) == 0
            {
                (*PciData).u.type1.MemoryBase = 0xFFFFu16;
                (*PciData).u.type1.PrefetchBase = 0xFFFFu16;
                (*PciData).u.type1.IOBase = 0xFFu8;
                (*PciData).u.type1.IOLimit = 0u8;
                (*PciData).u.type1.MemoryLimit = 0u16;
                (*PciData).u.type1.PrefetchLimit = 0u16;
                (*PciData).u.type1.PrefetchBaseUpper32 = 0u32;
                (*PciData).u.type1.PrefetchLimitUpper32 = 0u32;
                (*PciData).u.type1.IOBaseUpper16 = 0u16;
                (*PciData).u.type1.IOLimitUpper16 = 0u16;
            }
        }

        _ => return,
    }

    if ((*PciData).HeaderType & 0x7Fu8) == PCI_CARDBUS_BRIDGE_TYPE as u8 {
        let LegacyBaseAddress: u32 = 0;
        let offset: u32 = (core::mem::size_of::<PCI_COMMON_HEADER>() + core::mem::size_of::<u32>()) as u32;
        PciWriteDeviceConfig(
            PdoExtension,
            &LegacyBaseAddress as *const _ as *mut _,
            offset,
            core::mem::size_of::<u32>() as u32,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PcipIsSameDevice(
    DeviceExtension: PPCI_PDO_EXTENSION,
    PciData: *mut PCI_COMMON_HEADER,
) -> u8 {
    let hack_flags = (*DeviceExtension).HackFlags;

    let id_match = (*PciData).VendorID == (*DeviceExtension).VendorId
        && (*PciData).DeviceID == (*DeviceExtension).DeviceId;
    if !id_match {
        return 0u8;
    }

    let rev_match = (hack_flags & (PCI_HACK_NO_REVISION_AFTER_D3 as u64)) != 0
        || (*PciData).RevisionID == (*DeviceExtension).RevisionId;
    if !rev_match {
        return 0u8;
    }

    if ((*PciData).HeaderType & 0x80) != 0 {
        return 1u8;
    }

    if (*DeviceExtension).BaseClass == PCI_CLASS_BRIDGE_DEV as u8 {
        return 1u8;
    }

    let subsys_match = (hack_flags & ((PCI_HACK_NO_SUBSYSTEM as u64) | (PCI_HACK_NO_SUBSYSTEM_AFTER_D3 as u64))) != 0
        || ((*DeviceExtension).SubsystemVendorId == (*PciData).u.type0.SubVendorID
            && (*DeviceExtension).SubsystemId == (*PciData).u.type0.SubSystemID);
    subsys_match as u8
}

#[no_mangle]
pub unsafe extern "system" fn PciSkipThisFunction(
    PciData: PPCI_COMMON_HEADER,
    Slot: PCI_SLOT_NUMBER,
    OperationType: u8,
    HackFlags: u64,
) -> u8 {
    let mut skip = false;

    if OperationType == PCI_SKIP_DEVICE_ENUMERATION as u8 {
        if (HackFlags & (PCI_HACK_NO_ENUM_AT_ALL as u64)) != 0 {
            skip = true;
        }
        if !skip
            && (HackFlags & (PCI_HACK_DOUBLE_DECKER as u64)) != 0
            && Slot.u.bits.DeviceNumber() >= 16
        {
            DbgPrint(
                (b"    Device (Ven %04x Dev %04x (d=0x%x, f=0x%x)) is a ghost.\n\0".as_ptr()
                    as *const i8).cast(),
                (*PciData).VendorID as u32,
                (*PciData).DeviceID as u32,
                Slot.u.bits.DeviceNumber() as u32,
                Slot.u.bits.FunctionNumber() as u32,
            );
            skip = true;
        }
    } else if OperationType == PCI_SKIP_RESOURCE_ENUMERATION as u8 {
        if (HackFlags & (PCI_HACK_ENUM_NO_RESOURCE as u64)) != 0 {
            skip = true;
        }
    } else {
        ASSERTMSG(
            b"PCI Skip Function - Operation type unknown.\n\0".as_ptr() as *const i8,
            0u8,
        );
    }

    if !skip {
        if (*PciData).BaseClass == PCI_CLASS_BRIDGE_DEV as u8
            && (*PciData).SubClass <= PCI_SUBCLASS_BR_MCA as u8
            && OperationType == PCI_SKIP_RESOURCE_ENUMERATION as u8
        {
            skip = true;
        } else if (*PciData).BaseClass == PCI_CLASS_NOT_DEFINED as u8 {
            DbgPrint(
                (b"    Vendor %04x, Device %04x has class code of PCI_CLASS_NOT_DEFINED\n\0"
                    .as_ptr() as *const i8).cast(),
                (*PciData).VendorID as u32,
                (*PciData).DeviceID as u32,
            );
            if (*PciData).VendorID == 0x8086 && (*PciData).DeviceID == 8 {
                skip = true;
            }
        }
    }

    if skip {
        DbgPrint(
            (b"   Device skipped (not enumerated).\n\0".as_ptr() as *const i8).cast(),
        );
        return 1u8;
    }

    let config_type = (*PciData).HeaderType & 0x7F;
    if config_type <= PCI_CARDBUS_BRIDGE_TYPE as u8 {
        return 0u8;
    }

    DbgPrint(
        (b"   Device skipped (not enumerated).\n\0".as_ptr() as *const i8).cast(),
    );
    1u8
}
#[no_mangle]
pub unsafe extern "system" fn PciGetEnhancedCapabilities(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    (*PdoExtension).PowerState.DeviceWakeLevel = PowerDeviceUnspecified;
    if ((*PciData).Status & PCI_STATUS_CAPABILITIES_LIST as u16) == 0 {
        (*PdoExtension).CapabilitiesPtr = 0;
        (*PdoExtension).HackFlags |= (PCI_HACK_NO_PM_CAPS as u64);
    } else {
        let header_type = PCI_CONFIGURATION_TYPE(PciData);
        let cap_ptr = if header_type == PCI_CARDBUS_BRIDGE_TYPE as u8 {
            (*PciData).u.type2.CapabilitiesPtr
        } else {
            (*PciData).u.type0.CapabilitiesPtr
        };
        if (cap_ptr & 0x3) != 0 || (cap_ptr as u32) < PCI_COMMON_HDR_LENGTH { // nolint: magic-offset (PCI config-space header length, protocol constant)
            (*PdoExtension).CapabilitiesPtr = 0;
            (*PdoExtension).HackFlags |= (PCI_HACK_NO_PM_CAPS as u64);
        } else {
            DbgPrint(
                b"Device has capabilities at: %lx\n\0".as_ptr() as *const i8,
                cap_ptr as u32,
            );
            (*PdoExtension).CapabilitiesPtr = cap_ptr;
            if ((*PdoExtension).BaseClass == PCI_CLASS_BRIDGE_DEV as u8)
                && ((*PdoExtension).SubClass == PCI_SUBCLASS_BR_HOST as u8
                    || (*PdoExtension).SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI as u8)
            {
                let target_agp_capability_id = if (*PdoExtension).SubClass
                    == PCI_SUBCLASS_BR_PCI_TO_PCI as u8
                {
                    PCI_CAPABILITY_ID_AGP_TARGET
                } else {
                    PCI_CAPABILITY_ID_AGP
                };
                let mut agp_capability: PCI_CAPABILITIES_HEADER = core::mem::zeroed();
                if PciReadDeviceCapability(
                    PdoExtension,
                    (*PdoExtension).CapabilitiesPtr,
                    target_agp_capability_id,
                    (&mut agp_capability as *mut PCI_CAPABILITIES_HEADER as PVOID).cast(),
                    core::mem::size_of::<PCI_CAPABILITIES_HEADER>() as u32,
                ) != 0
                {
                    DbgPrint(
                        b"AGP ID: %lx\n\0".as_ptr() as *const i8,
                        target_agp_capability_id,
                    );
                    (*PdoExtension).TargetAgpCapabilityId = target_agp_capability_id as u8;
                }
            }
            if ((*PdoExtension).HackFlags & (PCI_HACK_NO_PM_CAPS as u64)) == 0 {
                let mut power_capabilities: PCI_PM_CAPABILITY = core::mem::zeroed();
                if PciReadDeviceCapability(
                    PdoExtension,
                    (*PdoExtension).CapabilitiesPtr,
                    PCI_CAPABILITY_ID_POWER_MANAGEMENT,
                    (&mut power_capabilities.Header as *mut PCI_CAPABILITIES_HEADER as PVOID).cast(),
                    core::mem::size_of::<PCI_PM_CAPABILITY>() as u32,
                ) == 0
                {
                    DbgPrint(b"No PM caps, disabling PM\n\0".as_ptr() as *const i8);
                    (*PdoExtension).HackFlags |= (PCI_HACK_NO_PM_CAPS as u64);
                } else {
                    let mut wake_level = PowerDeviceUnspecified;
                    if power_capabilities.PMC.Capabilities.Support.PMED0() != 0 {
                        wake_level = PowerDeviceD0;
                    }
                    if power_capabilities.PMC.Capabilities.Support.PMED1() != 0 {
                        wake_level = PowerDeviceD1;
                    }
                    if power_capabilities.PMC.Capabilities.Support.PMED2() != 0 {
                        wake_level = PowerDeviceD2;
                    }
                    if power_capabilities.PMC.Capabilities.Support.PMED3Hot() != 0 {
                        wake_level = PowerDeviceD3;
                    }
                    if power_capabilities.PMC.Capabilities.Support.PMED3Cold() != 0 {
                        wake_level = PowerDeviceD3;
                    }
                    (*PdoExtension).PowerState.DeviceWakeLevel = wake_level;
                    (*PdoExtension).PowerState.CurrentDeviceState =
                        power_capabilities.PMCSR.ControlStatus.PowerState() as u32 + 1;
                    (*PdoExtension).PowerCapabilities = power_capabilities.PMC.Capabilities;
                    DbgPrint(
                        b"PM Caps Found! Wake Level: %d Power State: %d\n\0".as_ptr()
                            as *const i8,
                        wake_level,
                        (*PdoExtension).PowerState.CurrentDeviceState,
                    );
                }
            }
        }
    }
    /* Now find out whether this is an Express function, and what kind */
    PciGetExpressCapabilities(PdoExtension);

    /* And whether any of its BARs can be resized */
    PciGetResizableBarCapability(PdoExtension);

    if ((*PdoExtension).HackFlags & (PCI_HACK_NO_PM_CAPS as u64)) != 0 {
        (*PdoExtension).PowerState.CurrentDeviceState = if ((*PciData).Command
            & ((PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as u16))
            != 0
        {
            PowerDeviceD0
        } else {
            PowerDeviceD3
        };
        DbgPrint(
            b"PM is off, so assumed device is: %d based on enables\n\0".as_ptr()
                as *const i8,
            (*PdoExtension).PowerState.CurrentDeviceState,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciWriteLimitsAndRestoreCurrent(
    Reserved: *mut core::ffi::c_void,
    Context2: *mut core::ffi::c_void,
) {
    let _ = Reserved;

    let Context = Context2 as PPCI_CONFIGURATOR_CONTEXT;
    let PdoExtension = (*Context).PdoExtension;
    let Current = (*Context).Current;
    let PciData = (*Context).PciData;

    // Write limit discovery header
    PciWriteDeviceConfig(PdoExtension, PciData, 0, PCI_COMMON_HDR_LENGTH); // nolint: magic-offset (PCI config-space header length, protocol constant)

    // Read what the device indicated the limits are
    PciReadDeviceConfig(PdoExtension, PciData.cast(), 0, PCI_COMMON_HDR_LENGTH); // nolint: magic-offset (PCI config-space header length, protocol constant)

    // Write back the original configuration header
    PciWriteDeviceConfig(PdoExtension, Current, 0, PCI_COMMON_HDR_LENGTH); // nolint: magic-offset (PCI config-space header length, protocol constant)

    // Copy back the original command from the context
    (*Current).Command = (*Context).Command;
    if (*Context).Command != 0u16 {
        // Program it back into the device
        PciWriteDeviceConfig(
            PdoExtension,
            core::ptr::addr_of_mut!((*Context).Command) as *mut PCI_COMMON_HEADER,
            ::core::mem::offset_of!(PCI_COMMON_HEADER, Command) as u32,
            ::core::mem::size_of::<USHORT>() as u32,
        );
    }

    // Copy back the original status
    (*Current).Status = (*Context).Status;

    // Call the configurator to restore any other data that might have changed
    ((*(*Context).Configurator).RestoreCurrent.unwrap())(Context);
}

#[no_mangle]
pub unsafe extern "system" fn PcipGetFunctionLimits(Context: *mut _PCI_CONFIGURATOR_CONTEXT) -> NTSTATUS {
    // PAGED_CODE()
    let pdo_extension: *mut _PCI_PDO_EXTENSION = (*Context).PdoExtension;
    let current: *mut _PCI_COMMON_HEADER = (*Context).Current;
    let pci_data: *mut _PCI_COMMON_HEADER = (*Context).PciData;

    (*Context).Status = (*current).Status;
    (*Context).Command = (*current).Command;

    (*current).Status = 0;
    (*current).Command &= !((PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as u16);

    core::ptr::copy_nonoverlapping(
        current as *const u8,
        pci_data as *mut u8,
        PCI_COMMON_HDR_LENGTH as usize, // nolint: magic-offset (PCI config-space header length, protocol constant)
    );

    let configurator: *mut _PCI_CONFIGURATOR =
        &PciConfigurators[(*pdo_extension).HeaderType as usize] as *const _ as *mut _;
    (*Context).Configurator = configurator;

    ((*configurator).Initialize.unwrap())(Context);

    if ((*pdo_extension).HackFlags & (PCI_HACK_CRITICAL_DEVICE as u64)) != 0 || (*pdo_extension).OnDebugPath != 0 {
        if (*pdo_extension).OnDebugPath != 0 {
            if ((*Context).Command & PCI_ENABLE_BUS_MASTER as u16) != 0 {
                (*pci_data).Command |= PCI_ENABLE_BUS_MASTER as u16;
                (*current).Command |= PCI_ENABLE_BUS_MASTER as u16;
            }
            KdDisableDebugger();
        }

        let mut ipi_context: _PCI_IPI_CONTEXT = core::mem::zeroed();
        ipi_context.Barrier = 1;
        ipi_context.RunCount = 1;
        ipi_context.DeviceExtension = (pdo_extension).cast();
        ipi_context.Function =
            Some(core::mem::transmute::<*const (), _>(PciWriteLimitsAndRestoreCurrent as *const ()));
        ipi_context.Context = Context as *mut core::ffi::c_void;
        KeIpiGenericCall(
            Some(PciExecuteCriticalSystemRoutine),
            core::ptr::addr_of!(ipi_context) as usize as _,
        );

        if (*pdo_extension).OnDebugPath != 0 {
            KdEnableDebugger();
        }
    } else {
        PciWriteLimitsAndRestoreCurrent(
            pdo_extension as *mut core::ffi::c_void,
            Context as *mut core::ffi::c_void,
        );
    }

    if (*pdo_extension).ExpectedWritebackFailure == 0 {
        let header_after =
            (pci_data as *mut u8).add(PCI_COMMON_HDR_LENGTH as usize) as *mut _PCI_COMMON_HEADER; // nolint: magic-offset (PCI config-space header length, protocol constant)
        PciReadDeviceConfig(pdo_extension, (header_after).cast(), 0, PCI_COMMON_HDR_LENGTH); // nolint: magic-offset (PCI config-space header length, protocol constant)

        let offset = RtlCompareMemory(
            header_after as *const core::ffi::c_void,
            current as *const core::ffi::c_void,
            PCI_COMMON_HDR_LENGTH as usize as _, // nolint: magic-offset (PCI config-space header length, protocol constant)
        );
        if offset != PCI_COMMON_HDR_LENGTH as usize as _ { // nolint: magic-offset (PCI config-space header length, protocol constant)
            DbgPrint(
                b"PCI - CFG space write verify failed at offset 0x%x\n\0".as_ptr() as _,
                offset as u32,
            );
            PciDebugDumpCommonConfig(header_after);
            DbgPrint(b"----------\n\0".as_ptr() as _);
            PciDebugDumpCommonConfig(current);
        }
    }

    ntkrnl_sys::ASSERT!((*pdo_extension).Resources.is_null());

    let resources = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_PCI_FUNCTION_RESOURCES>(),
        (0x42696350u32) as u32,
    ) as *mut _PCI_FUNCTION_RESOURCES;
    if resources.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*pdo_extension).Resources = resources;

    core::ptr::write_bytes(
        resources as *mut u8,
        0,
        core::mem::size_of::<_PCI_FUNCTION_RESOURCES>(),
    );

    ((*configurator).SaveLimits.unwrap())(Context);
    ((*configurator).SaveCurrentSettings.unwrap())(Context);

    let limit_ptr = core::ptr::addr_of_mut!((*resources).Limit) as *mut IO_RESOURCE_DESCRIPTOR;
    let mut io_descriptor = limit_ptr.add(PCI_TYPE0_ADDRESSES as usize + 1);
    loop {
        io_descriptor = io_descriptor.sub(1);
        if (*io_descriptor).Type != CmResourceTypeNull as u8 {
            break;
        }
        if io_descriptor == limit_ptr {
            DbgPrint(b"PCI Resources fail!\n\0".as_ptr() as _);
            ExFreePoolWithTag(resources as *mut core::ffi::c_void, 0);
            (*pdo_extension).Resources = core::ptr::null_mut();
            break;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciGetFunctionLimits(
    PdoExtension: *mut _PCI_PDO_EXTENSION,
    Current: *mut _PCI_COMMON_HEADER,
    HackFlags: u64,
) -> NTSTATUS {
    if PciSkipThisFunction(
        Current,
        (*PdoExtension).Slot,
        PCI_SKIP_RESOURCE_ENUMERATION as u8,
        HackFlags,
    ) != 0
    {
        return STATUS_SUCCESS;
    }

    let PciData: *mut _PCI_COMMON_HEADER = ExAllocatePoolWithTag(
        0,
        (2 * PCI_COMMON_HDR_LENGTH) as usize, // nolint: magic-offset (PCI config-space header length, protocol constant)
        u32::from_le_bytes([b'B', b'i', b'c', b'P']),
    ) as *mut _PCI_COMMON_HEADER;
    if PciData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut Context: _PCI_CONFIGURATOR_CONTEXT = core::mem::zeroed();
    Context.Current = Current;
    Context.PciData = PciData;
    Context.PdoExtension = PdoExtension;

    let Status = PcipGetFunctionLimits(&mut Context);

    ExFreePoolWithTag(PciData as *mut core::ffi::c_void, 0);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciProcessBus(device_extension: PPCI_FDO_EXTENSION) {
    // PAGED_CODE();

    let physical_device_object = (*device_extension).PhysicalDeviceObject;
    let mut pdo_extension = (*physical_device_object).DeviceExtension as PPCI_PDO_EXTENSION;

    if PCI_IS_ROOT_FDO(device_extension) == 0u8 {
        UNIMPLEMENTED_DBGBREAK();

        if !pdo_extension.is_null()
            && PciClassifyDeviceType(pdo_extension) == PciTypePciBridge
            && ((*pdo_extension).Dependent.type1.as_ref().IsaBitRequired() != 0
                || (*pdo_extension).Dependent.type1.as_ref().IsaBitSet() != 0)
        {
            UNIMPLEMENTED_DBGBREAK();
        }
    } else {
        pdo_extension = (*device_extension).ChildBridgePdoList;
        while !pdo_extension.is_null() {
            if (*pdo_extension).Dependent.type1.as_ref().VgaBitSet() != 0 {
                UNIMPLEMENTED_DBGBREAK();
            }
            pdo_extension = (*pdo_extension).NextBridge;
        }
    }

    if PciAssignBusNumbers != 0 {
        UNIMPLEMENTED_DBGBREAK();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciScanBus(DeviceExtension: PPCI_FDO_EXTENSION) -> NTSTATUS {
    let mut max_device = PCI_MAX_DEVICES;
    let mut process_flag: u8 = 0;
    let mut i: u32;
    let mut j: u32;
    let mut k: u32;
    let mut size: u32;
    let mut cap_offset: u16;
    let mut temp_offset: u16;
    let mut hack_flags: i64;
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut buffer: [u8; PCI_COMMON_HDR_LENGTH as usize] = [0u8; PCI_COMMON_HDR_LENGTH as usize]; // nolint: magic-offset (PCI config-space header length, protocol constant)
    let mut bios_buffer: [u8; PCI_COMMON_HDR_LENGTH as usize] = [0u8; PCI_COMMON_HDR_LENGTH as usize]; // nolint: magic-offset (PCI config-space header length, protocol constant)
    let pci_data = buffer.as_mut_ptr() as PPCI_COMMON_HEADER;
    let bios_data = bios_buffer.as_mut_ptr() as PPCI_COMMON_HEADER;
    let mut pci_slot: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut name: PCHAR;
    let mut status: NTSTATUS;
    let mut pdo_extension: PPCI_PDO_EXTENSION;
    let mut new_extension: PPCI_PDO_EXTENSION;
    let mut bridge_extension: *mut *mut _PCI_PDO_EXTENSION;
    let mut description_text: PWCHAR;
    let mut sub_vendor_id: u16;
    let mut sub_system_id: u16;
    let mut cap_header: PCI_CAPABILITIES_HEADER = core::mem::zeroed();
    let mut pcix_cap_header: PCI_CAPABILITIES_HEADER = core::mem::zeroed();
    let mut secondary_bus: u8 = 0;

    DbgPrint((b"PCI Scan Bus: FDO Extension @ 0x%p, Base Bus = 0x%x\n\0".as_ptr() as *const i8).cast(),
             DeviceExtension, (*DeviceExtension).BaseBus as u32);

    if PCI_IS_ROOT_FDO(DeviceExtension) == 0 {
        pdo_extension = (*(*(DeviceExtension)).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
        ASSERT_PDO(pdo_extension);

        if ((*pdo_extension).HackFlags & (PCI_HACK_ONE_CHILD as u64)) != 0 {
            max_device = 1;
        }

        PciReadDeviceConfig(pdo_extension,
                            (core::ptr::addr_of_mut!(secondary_bus)).cast(),
                            core::mem::offset_of!(PCI_COMMON_HEADER, u.type1.SecondaryBus) as u32,
                            core::mem::size_of::<u8>() as u32);
        if secondary_bus != (*pdo_extension).Dependent.type1.as_ref().SecondaryBus {
            UNIMPLEMENTED_DBGBREAK();
        }
    }

    pci_slot.u.bits.set_Reserved(0);
    i = (*DeviceExtension).BaseBus as u32;
    for j in 0..max_device {
        pci_slot.u.bits.set_DeviceNumber(j);
        for k in 0..PCI_MAX_FUNCTION {
            pci_slot.u.bits.set_FunctionNumber(k);

            PciReadSlotConfig(DeviceExtension,
                              pci_slot,
                              pci_data.cast(),
                              0,
                              core::mem::size_of::<u16>() as u32);

            if (*pci_data).VendorID == PCI_INVALID_VENDORID as u16 {
                continue;
            }

            PciReadSlotConfig(DeviceExtension,
                              pci_slot,
                              core::ptr::addr_of_mut!((*pci_data).DeviceID).cast(),
                              core::mem::size_of::<u16>() as u32,
                              PCI_COMMON_HDR_LENGTH - core::mem::size_of::<u16>() as u32); // nolint: magic-offset (PCI config-space header length, protocol constant)

            PciApplyHacks(DeviceExtension,
                          pci_data,
                          pci_slot,
                          PCI_HACK_FIXUP_BEFORE_CONFIGURATION,
                          core::ptr::null_mut());

            DbgPrint((b"Scan Found Device 0x%x (b=0x%x, d=0x%x, f=0x%x)\n\0".as_ptr() as *const i8).cast(),
                     pci_slot.u.AsULONG, i, j, k);

            PciDebugDumpCommonConfig(pci_data);

            description_text = PciGetDeviceDescriptionMessage((*pci_data).BaseClass,
                                                              (*pci_data).SubClass);
            DbgPrint((b"Device Description \"%S\".\n\0".as_ptr() as *const i8).cast(),
                     if !description_text.is_null() { description_text } else { NULL_DESCRIPTION.as_ptr() as PWCHAR });
            if !description_text.is_null() {
                ExFreePoolWithTag((description_text).cast(), 0);
            }

            if !WdTable.is_null() {
                UNIMPLEMENTED_DBGBREAK();
            }

            if (PCI_MULTIFUNCTION_DEVICE(pci_data) != 0) || ((*pci_data).BaseClass == PCI_CLASS_BRIDGE_DEV as u8) {
                sub_vendor_id = 0;
                sub_system_id = 0;
            } else {
                sub_vendor_id = (*pci_data).u.type0.SubVendorID;
                sub_system_id = (*pci_data).u.type0.SubSystemID;
            }

            hack_flags = PciGetHackFlags((*pci_data).VendorID,
                                         (*pci_data).DeviceID,
                                         sub_vendor_id,
                                         sub_system_id,
                                         (*pci_data).RevisionID) as i64;

            if PciIsCriticalDeviceClass((*pci_data).BaseClass, (*pci_data).SubClass) != 0 {
                /* The hack database can take a device out of its critical class */
                if (hack_flags & (PCI_HACK_NOT_CRITICAL_DEVICE as i64)) == 0 {
                    DbgPrint((b"Not allowing PM Because device is critical\n\0".as_ptr() as *const i8).cast());

                    /* Probe it with the system held, and keep it decoding while stopped */
                    hack_flags |= (PCI_HACK_CRITICAL_DEVICE | PCI_HACK_KEEP_DECODES_ON_STOP) as i64;

                    /* Only the display can still be powered down */
                    if (*pci_data).BaseClass != PCI_CLASS_DISPLAY_CTLR as u8 {
                        hack_flags |= PCI_HACK_NEVER_POWER_DOWN as i64;
                    }
                }
            }

            if ((*pci_data).BaseClass == PCI_CLASS_BRIDGE_DEV as u8) &&
               ((*pci_data).SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI as u8) &&
               ((*pci_data).u.type1.BridgeControl & PCI_ENABLE_BRIDGE_VGA as u16) != 0 &&
               (hack_flags & (PCI_HACK_NOT_CRITICAL_DEVICE as i64)) == 0 {
                /* They follow the adapter behind them, so they can still be powered down */
                DbgPrint((b"Not allowing PM because device is VGA\n\0".as_ptr() as *const i8).cast());
                hack_flags |= (PCI_HACK_CRITICAL_DEVICE | PCI_HACK_KEEP_DECODES_ON_STOP) as i64;
            }

            if PciSkipThisFunction(pci_data,
                                   pci_slot,
                                   PCI_SKIP_DEVICE_ENUMERATION as u8,
                                   hack_flags as u64) != 0 {
                continue;
            }

            pdo_extension = PciFindPdoByFunction(DeviceExtension,
                                                 pci_slot.u.AsULONG,
                                                 pci_data);
            if !pdo_extension.is_null() {
                UNIMPLEMENTED_DBGBREAK();
            }

            process_flag = 1u8;

            status = PciPdoCreate(DeviceExtension, pci_slot, &mut device_object);
            ntkrnl_sys::ASSERT!(NT_SUCCESS(status));
            new_extension = (*device_object).DeviceExtension as PPCI_PDO_EXTENSION;

            if (hack_flags & (PCI_HACK_FAKE_CLASS_CODE as i64)) != 0 {
                (*pci_data).BaseClass = PCI_CLASS_BASE_SYSTEM_DEV as u8;
                (*pci_data).SubClass = PCI_SUBCLASS_SYS_OTHER as u8;
                (*new_extension).ExpectedWritebackFailure = 1u8;
            }

            (*new_extension).VendorId = (*pci_data).VendorID;
            (*new_extension).DeviceId = (*pci_data).DeviceID;
            (*new_extension).RevisionId = (*pci_data).RevisionID;
            (*new_extension).ProgIf = (*pci_data).ProgIf;
            (*new_extension).SubClass = (*pci_data).SubClass;
            (*new_extension).BaseClass = (*pci_data).BaseClass;
            (*new_extension).HeaderType = PCI_CONFIGURATION_TYPE(pci_data);

            if ((*new_extension).BaseClass == PCI_CLASS_BRIDGE_DEV as u8) &&
               (((*new_extension).SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI as u8) ||
                ((*new_extension).SubClass == PCI_SUBCLASS_BR_CARDBUS as u8)) {
                KeEnterCriticalRegion();
                KeWaitForSingleObject(core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as *mut core::ffi::c_void,
                                      _KWAIT_REASON_Executive,
                                      0i8,
                                      0u8,
                                      core::ptr::null_mut());

                bridge_extension = core::ptr::addr_of_mut!((*DeviceExtension).ChildBridgePdoList);
                while !(*bridge_extension).is_null() {
                    bridge_extension = core::ptr::addr_of_mut!((*(*bridge_extension)).NextBridge);
                }

                *bridge_extension = new_extension;
                ntkrnl_sys::ASSERT!((*new_extension).NextBridge.is_null());

                KeSetEvent(core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as *mut core::ffi::c_void,
                           IO_NO_INCREMENT as i32,
                           0u8);
                KeLeaveCriticalRegion();
            }

            status = PciGetBiosConfig(new_extension, bios_data);
            if NT_SUCCESS(status) {
                DbgPrint((b"Have BIOS configuration!\n\0".as_ptr() as *const i8).cast());
                UNIMPLEMENTED!();

                if PcipIsSameDevice((new_extension).cast(), bios_data) == 0 {
                    status = STATUS_UNSUCCESSFUL;
                } else {
                    if (*bios_data).u.type0.InterruptLine != (*pci_data).u.type0.InterruptLine {
                        PciWriteDeviceConfig(new_extension,
                                             core::ptr::addr_of_mut!((*bios_data).u.type0.InterruptLine).cast(),
                                             core::mem::offset_of!(PCI_COMMON_HEADER, u.type0.InterruptLine) as u32,
                                             core::mem::size_of::<u8>() as u32);
                    }
                    (*new_extension).RawInterruptLine = (*bios_data).u.type0.InterruptLine;
                    (*new_extension).InitialCommand = (*bios_data).Command;
                }
            }

            if !NT_SUCCESS(status) {
                status = PciSaveBiosConfig(new_extension, pci_data);
                ntkrnl_sys::ASSERT!(NT_SUCCESS(status));
                (*new_extension).RawInterruptLine = (*pci_data).u.type0.InterruptLine;
                (*new_extension).InitialCommand = (*pci_data).Command;
            }

            (*new_extension).CommandEnables = (*pci_data).Command;
            (*new_extension).HackFlags = hack_flags as u64;

            PciGetEnhancedCapabilities(new_extension, pci_data);

            status = PciGetFunctionLimits(new_extension, pci_data, hack_flags as u64);

            PciSetPowerManagedDevicePowerState(new_extension, PowerDeviceD0, 0u8);

            PciApplyHacks(DeviceExtension,
                          pci_data,
                          pci_slot,
                          PCI_HACK_FIXUP_AFTER_CONFIGURATION,
                          new_extension);

            (*new_extension).InterruptPin = (*pci_data).u.type0.InterruptPin;

            (*new_extension).AdjustedInterruptLine = PciGetAdjustedInterruptLine(new_extension);

            (*new_extension).OnDebugPath = PciIsDeviceOnDebugPath(new_extension);

            if (hack_flags & (PCI_HACK_NO_SUBSYSTEM as i64)) != 0 {
                (*new_extension).SubsystemVendorId = 0;
                (*new_extension).SubsystemId = 0;
            }

            cap_offset = (*new_extension).CapabilitiesPtr as u16;
            while cap_offset != 0 {
                temp_offset = PciReadDeviceCapability(new_extension,
                                                      cap_offset as u8,
                                                      0,
                                                      core::ptr::addr_of_mut!(cap_header),
                                                      core::mem::size_of::<PCI_CAPABILITIES_HEADER>() as u32) as u16;
                if temp_offset != cap_offset {
                    DbgPrint((b"PCI - Failed to read PCI capability at offset 0x%02x\n\0".as_ptr() as *const i8).cast(),
                             cap_offset as u32);
                    ntkrnl_sys::ASSERT!(temp_offset == cap_offset);
                }

                match cap_header.CapabilityID as u32 {
                    PCI_CAPABILITY_ID_POWER_MANAGEMENT => {
                        name = b"POWER\0".as_ptr() as PCHAR;
                        size = core::mem::size_of::<PCI_PM_CAPABILITY>() as u32;
                    },
                    PCI_CAPABILITY_ID_AGP => {
                        name = b"AGP\0".as_ptr() as PCHAR;
                        size = core::mem::size_of::<PCI_AGP_CAPABILITY>() as u32;
                    },
                    _ => {
                        name = b"UNKNOWN CAPABILITY\0".as_ptr() as PCHAR;
                        size = 0;
                    }
                }

                if size != 0 {
                    temp_offset = PciReadDeviceCapability(new_extension,
                                                          cap_offset as u8,
                                                          cap_header.CapabilityID as u32,
                                                          core::ptr::addr_of_mut!(cap_header),
                                                          size) as u16;
                    if temp_offset != cap_offset {
                        DbgPrint((b"- Failed to read capability data. ***\n\0".as_ptr() as *const i8).cast());
                        ntkrnl_sys::ASSERT!(temp_offset == cap_offset);
                    }
                }

                DbgPrint((b"CAP @%02x ID %02x (%s)\n\0".as_ptr() as *const i8).cast(),
                         cap_offset as u32, cap_header.CapabilityID as u32, name);
                let cap_header_ptr = core::ptr::addr_of!(cap_header) as *const u8;
                i = 0;
                while i < size {
                    let word_ptr = cap_header_ptr.add(i as usize) as *const u16;
                    DbgPrint((b"  %04x\n\0".as_ptr() as *const i8).cast(), *word_ptr as u32);
                    i += 2;
                }
                DbgPrint((b"\n\0".as_ptr() as *const i8).cast());

                cap_offset = cap_header.Next as u16;
            }

            if ((*new_extension).BaseClass == PCI_CLASS_MASS_STORAGE_CTLR as u8) &&
               ((*new_extension).SubClass == PCI_SUBCLASS_MSC_IDE_CTLR as u8) {
                (*new_extension).DisablePowerDown = 1u8;
            }

            if (((*new_extension).BaseClass == PCI_CLASS_BRIDGE_DEV as u8) &&
                (((*new_extension).SubClass == PCI_SUBCLASS_BR_ISA as u8) ||
                 ((*new_extension).SubClass == PCI_SUBCLASS_BR_EISA as u8) ||
                 ((*new_extension).SubClass == PCI_SUBCLASS_BR_MCA as u8))) ||
               (((*new_extension).VendorId == 0x8086) &&
                ((*new_extension).DeviceId == 0x482)) {
                (*new_extension).DisablePowerDown = 1u8;
            }

            if (*pci_data).CacheLineSize == 0 {
                if ((*new_extension).CommandEnables & ((PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as u16)) == 0 {
                    temp_offset = PciReadDeviceCapability(new_extension,
                                                          (*new_extension).CapabilitiesPtr,
                                                          PCI_CAPABILITY_ID_PCIX,
                                                          (core::ptr::addr_of_mut!(pcix_cap_header) as *mut u8).cast(),
                                                          core::mem::size_of::<PCI_CAPABILITIES_HEADER>() as u32) as u16;

                    if ((*pci_data).LatencyTimer == 0) ||
                       ((temp_offset != 0) && ((*pci_data).LatencyTimer == 64)) {
                        DbgPrint((b"PCI - ScanBus, PDOx %p found unconfigured\n\0".as_ptr() as *const i8).cast(),
                                 new_extension);
                        (*new_extension).NeedsHotPlugConfiguration = 1u8;
                    }
                }
            }

            (*new_extension).SavedLatencyTimer = (*pci_data).LatencyTimer;
            (*new_extension).SavedCacheLineSize = (*pci_data).CacheLineSize;

            (*device_object).Flags &= !DO_DEVICE_INITIALIZING;
        }
    }

    if process_flag != 0 {
        PciProcessBus(DeviceExtension);
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciQueryDeviceRelations(
    DeviceExtension: PPCI_FDO_EXTENSION,
    pDeviceRelations: &mut PDEVICE_RELATIONS,
) -> NTSTATUS {
    PAGED_CODE!();

    ntkrnl_sys::ASSERT!((*DeviceExtension).DeviceState == PciStarted as u8);

    let status = PciBeginStateTransition(DeviceExtension, PciSynchronizedOperation);
    if !NT_SUCCESS(status) {
        return status;
    }

    let mut pdo_ext = (*DeviceExtension).ChildPdoList;
    while !pdo_ext.is_null() {
        (*pdo_ext).NotPresent = 1u8;
        pdo_ext = ((*pdo_ext).Next).cast();
    }

    let status = PciScanBus(DeviceExtension);
    ntkrnl_sys::ASSERT!(NT_SUCCESS(status));

    let mut pdo_ext = (*DeviceExtension).ChildPdoList;
    let mut pdo_count: ULONG = 0;
    while !pdo_ext.is_null() {
        if (*pdo_ext).NotPresent != 0 {
            (*pdo_ext).ReportedMissing = 1u8;
            DbgPrint(
                (b"PCI - Old device (pdox) %p not found on rescan.\n\0".as_ptr() as *const i8).cast(),
                pdo_ext,
            );
        } else {
            pdo_count += 1;
        }
        pdo_ext = ((*pdo_ext).Next).cast();
    }

    let device_relations = *pDeviceRelations;
    let header_size = core::mem::size_of::<DEVICE_RELATIONS>();
    let elem_size = core::mem::size_of::<PDEVICE_OBJECT>();
    let mut size = header_size + (pdo_count as usize) * elem_size;
    if !device_relations.is_null() {
        size += ((*device_relations).Count as usize) * elem_size;
    }

    let tag = u32::from_ne_bytes([b'B', b'i', b'c', b'P']);
    let new_relations = ExAllocatePoolWithTag(0, size, tag) as PDEVICE_RELATIONS;
    if new_relations.is_null() {
        PciCancelStateTransition(DeviceExtension, PciSynchronizedOperation);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*new_relations).Count = 0;
    if !device_relations.is_null() {
        let copy_len = header_size + ((*device_relations).Count as usize) * elem_size;
        core::ptr::copy_nonoverlapping(
            device_relations as *const u8,
            new_relations as *mut u8,
            copy_len,
        );
        ExFreePoolWithTag(device_relations as *mut core::ffi::c_void, 0);
    }

    DbgPrint(
        (b"PCI QueryDeviceRelations/BusRelations FDOx %p (bus 0x%02x)\n\0".as_ptr() as *const i8).cast(),
        DeviceExtension,
        (*DeviceExtension).BaseBus as u32,
    );

    let mut pdo_ext = (*DeviceExtension).ChildPdoList;
    let mut object_ptr = core::ptr::addr_of_mut!((*new_relations).Objects[0])
        .add((*new_relations).Count as usize);

    while !pdo_ext.is_null() {
        let not_present = (*pdo_ext).NotPresent != 0;
        DbgPrint(
            (b"  QDR PDO %p (x %p)%s\n\0".as_ptr() as *const i8).cast(),
            (*pdo_ext).PhysicalDeviceObject,
            pdo_ext,
            if not_present {
                b"<Omitted, device flaged not present>\0".as_ptr() as *const i8
            } else {
                b"\0".as_ptr() as *const i8
            },
        );

        if !not_present {
            let dev_obj = (*pdo_ext).PhysicalDeviceObject;
            ObReferenceObject((dev_obj).cast());
            *object_ptr = dev_obj;
            object_ptr = object_ptr.add(1);
        }
        pdo_ext = ((*pdo_ext).Next).cast();
    }

    DbgPrint(
        (b"  QDR Total PDO count = %u (%u already in list)\n\0".as_ptr() as *const i8).cast(),
        (*new_relations).Count + pdo_count,
        (*new_relations).Count,
    );

    (*new_relations).Count += pdo_count;
    *pDeviceRelations = new_relations;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciSetResources(
    PdoExtension: PPCI_PDO_EXTENSION,
    DoReset: u8,
    SomethingSomethingDarkSide: u8,
) -> NTSTATUS {
    let fdo_extension = (*PdoExtension).ParentFdoExtension;
    let mut pci_data: PCI_COMMON_HEADER = core::mem::zeroed();

    PciReadDeviceConfig(
        PdoExtension,
        (core::ptr::addr_of_mut!(pci_data)).cast(),
        0,
        PCI_COMMON_HDR_LENGTH as u32, // nolint: magic-offset (PCI config-space header length, protocol constant)
    );

    if PcipIsSameDevice((PdoExtension).cast(), core::ptr::addr_of_mut!(pci_data)) == 0 {
        ASSERTMSG(
            b"PCI Set resources - not same device.\n\0".as_ptr() as *const i8,
            0u8,
        );
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    if (*PdoExtension).BaseClass == PCI_CLASS_BRIDGE_DEV as u8
        && (*PdoExtension).SubClass == PCI_SUBCLASS_BR_HOST as u8
    {
        return STATUS_SUCCESS;
    }

    if DoReset != 0
        && (*PdoExtension).BaseClass == PCI_CLASS_MASS_STORAGE_CTLR as u8
        && (*PdoExtension).SubClass == PCI_SUBCLASS_MSC_IDE_CTLR as u8
    {
        let native = PciConfigureIdeController(
            PdoExtension,
            core::ptr::addr_of_mut!(pci_data),
            0u8,
        );
        // ASSERT(native == PdoExtension->IDEInNativeMode); omitted (debug only)
    }

    if (*PdoExtension).NeedsHotPlugConfiguration != 0
        && (*fdo_extension).HotPlugParameters.Acquired != 0
    {
        let mut cache_line_read_back: UCHAR = 0;

        /* The firmware never configured this device, so use what _HPP asks for */
        (*PdoExtension).SavedLatencyTimer = (*fdo_extension).HotPlugParameters.LatencyTimer;
        (*PdoExtension).SavedCacheLineSize = (*fdo_extension).HotPlugParameters.CacheLineSize;

        (*PdoExtension).CommandEnables &= !((PCI_ENABLE_PARITY | PCI_ENABLE_SERR) as u16);
        if (*fdo_extension).HotPlugParameters.EnablePERR != 0 {
            (*PdoExtension).CommandEnables |= PCI_ENABLE_PARITY as u16;
        }

        if (*fdo_extension).HotPlugParameters.EnableSERR != 0 {
            (*PdoExtension).CommandEnables |= PCI_ENABLE_SERR as u16;
        }

        /* Only turn on memory write and invalidate if the device keeps the cache line size */
        PciWriteDeviceConfig(
            PdoExtension,
            core::ptr::addr_of_mut!((*PdoExtension).SavedCacheLineSize) as _,
            core::mem::offset_of!(PCI_COMMON_HEADER, CacheLineSize) as ULONG,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
        PciReadDeviceConfig(
            PdoExtension,
            core::ptr::addr_of_mut!(cache_line_read_back) as _,
            core::mem::offset_of!(PCI_COMMON_HEADER, CacheLineSize) as ULONG,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
        if cache_line_read_back != 0 && cache_line_read_back == (*PdoExtension).SavedCacheLineSize {
            (*PdoExtension).CommandEnables |= PCI_ENABLE_WRITE_AND_INVALIDATE as u16;
        } else {
            DbgPrint(
                b"(%s:%d) PCI (pdox %p) cache line size %02x rejected, MWI stays off\n\0".as_ptr() as *const i8,
                b"drivers/bus/pcix/enum.c\0".as_ptr() as *const i8,
                2358,
                PdoExtension,
                (*PdoExtension).SavedCacheLineSize as u32,
            );
        }
    }

    let header_type = (*PdoExtension).HeaderType;
    let configurator = &PciConfigurators[header_type as usize];

    (configurator.ChangeResourceSettings.unwrap())(PdoExtension, core::ptr::addr_of_mut!(pci_data));

    (*PdoExtension).UpdateHardware = 0u8;

    if DoReset != 0 {
        (configurator.ResetDevice.unwrap())(PdoExtension, core::ptr::addr_of_mut!(pci_data));
        pci_data.u.type0.InterruptLine = (*PdoExtension).RawInterruptLine;
    }

    let new_latency_timer = (*PdoExtension).SavedLatencyTimer;
    if pci_data.LatencyTimer != new_latency_timer {
        DbgPrint(
            b"PCI (pdox %p) changing latency from %02x to %02x.\n\0".as_ptr() as *const i8,
            PdoExtension,
            pci_data.LatencyTimer as u32,
            new_latency_timer as u32,
        );
    }

    let new_cache_line_size = (*PdoExtension).SavedCacheLineSize;
    if pci_data.CacheLineSize != new_cache_line_size {
        DbgPrint(
            b"PCI (pdox %p) changing cache line size from %02x to %02x.\n\0".as_ptr()
                as *const i8,
            PdoExtension,
            pci_data.CacheLineSize as u32,
            new_cache_line_size as u32,
        );
    }

    pci_data.LatencyTimer = new_latency_timer;
    pci_data.CacheLineSize = new_cache_line_size;
    pci_data.u.type0.InterruptLine = (*PdoExtension).RawInterruptLine;

    PciApplyHacks(
        fdo_extension,
        core::ptr::addr_of_mut!(pci_data),
        (*PdoExtension).Slot,
        PCI_HACK_FIXUP_BEFORE_UPDATE,
        PdoExtension,
    );

    if (*PdoExtension).IoSpaceNotRequired != 0u8 {
        (*PdoExtension).CommandEnables &= !(PCI_ENABLE_IO_SPACE as u16);
    }

    PciUpdateHardware(PdoExtension, core::ptr::addr_of_mut!(pci_data));

    (*PdoExtension).RawInterruptLine = pci_data.u.type0.InterruptLine;
    (*PdoExtension).NeedsHotPlugConfiguration = 0u8;

    STATUS_SUCCESS
}
