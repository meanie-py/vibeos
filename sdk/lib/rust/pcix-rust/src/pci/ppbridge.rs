#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
// The crate's own functions: the definition is the declaration.
use crate::utils::PciIsSlotPresentInParentMethod;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, PCI_COMMON_HEADER, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PHYSICAL_ADDRESS, PIO_RESOURCE_DESCRIPTOR, PPCI_COMMON_CONFIG, PPCI_COMMON_HEADER};
use core::ffi::c_void;

extern "system" {
}

const PCI_HACK_SUBTRACTIVE_DECODE: u64 = 0x0000000000040000;
const PCI_COMMON_HDR_LENGTH: usize = 0x40;
const CmResourceTypeDevicePrivate: u32 = 129;
#[no_mangle]
pub unsafe extern "system" fn PciBridgeIoBase(PciData: PPCI_COMMON_HEADER) -> ULONG {
    let mut IoBase: ULONG;

    /* The C reads IOLimit here, so a closed window (base above limit, as firmware
     * leaves an unused one) decoded as a live 4 KB window at the limit, and the
     * bridge's real windows were then reprogrammed around that phantom. */
    let Base = (*PciData).u.type1.IOBase as ULONG;
    let Is32Bit = ((Base & 0xF) == 1) as u8;
    IoBase = (Base & 0xF0) << 8;

    if Is32Bit != 0 {
        IoBase |= ((*PciData).u.type1.IOBaseUpper16 as ULONG) << 16;
    }

    IoBase
}
#[no_mangle]
pub unsafe extern "system" fn PciBridgeIoLimit(PciData: PPCI_COMMON_HEADER) -> ULONG {
    let Limit = (*PciData).u.type1.IOLimit;
    let Is32Bit = (Limit & 0xF) == 1;
    let mut IoLimit = ((Limit & 0xF0) as ULONG) << 8;

    if Is32Bit {
        IoLimit |= ((*PciData).u.type1.IOLimitUpper16 as ULONG) << 16;
    }

    IoLimit | 0xFFF
}
#[no_mangle]
pub unsafe extern "system" fn PciBridgeMemoryBase(PciData: PPCI_COMMON_HEADER) -> u32 {
    ((*PciData).u.type1.MemoryBase as u32) << 16
}

#[no_mangle]
pub unsafe extern "system" fn PciBridgeMemoryLimit(PciData: PPCI_COMMON_HEADER) -> u32 {
    (((*PciData).u.type1.MemoryLimit as u32) << 16) | 0xFFFFF
}

#[no_mangle]
pub unsafe extern "system" fn PciBridgePrefetchMemoryBase(PciData: PPCI_COMMON_HEADER) -> PHYSICAL_ADDRESS {
    let prefetch_base = (*PciData).u.type1.PrefetchBase;
    let is_64bit = (prefetch_base & 0xF) == 1;
    let base_low = ((prefetch_base as u32 & 0xFFF0u32) << 16) as u32;
    let mut base: PHYSICAL_ADDRESS = core::mem::zeroed();
    base.__bindgen_anon_1.LowPart = base_low;
    base.__bindgen_anon_1.HighPart = if is_64bit {
        (*PciData).u.type1.PrefetchBaseUpper32 as i32
    } else {
        0i32
    };
    base
}

#[no_mangle]
pub unsafe extern "system" fn PciBridgePrefetchMemoryLimit(pci_data: PPCI_COMMON_HEADER) -> PHYSICAL_ADDRESS {
    let prefetch_limit = (*pci_data).u.type1.PrefetchLimit;
    let is_64bit = (prefetch_limit & 0xF) == 1;
    let mut low_part: u32 = ((prefetch_limit as u32) << 16) | 0xFFFFFu32;
    let high_part: i32 = if is_64bit {
        (*pci_data).u.type1.PrefetchLimitUpper32 as i32
    } else {
        0i32
    };
    let mut limit: PHYSICAL_ADDRESS = core::mem::zeroed();
    limit.__bindgen_anon_1.LowPart = low_part;
    limit.__bindgen_anon_1.HighPart = high_part;
    limit
}

#[no_mangle]
pub unsafe extern "system" fn PciBridgeMemoryWorstCaseAlignment(Length: u32) -> u32 {
    ntkrnl_sys::ASSERT!(Length != 0);
    let mut Alignment: u32 = 0x80000000;
    while (Length & Alignment) == 0 {
        Alignment >>= 1;
    }
    Alignment
}

#[no_mangle]
pub unsafe extern "system" fn PciBridgeIsPositiveDecode(PdoExtension: *mut core::ffi::c_void) -> u8 {
    PciIsSlotPresentInParentMethod((PdoExtension).cast(), 0x50444543u32)
}
#[no_mangle]
pub unsafe extern "system" fn PciBridgeIsSubtractiveDecode(Context: PPCI_CONFIGURATOR_CONTEXT) -> BOOLEAN {
    let Current: PPCI_COMMON_HEADER = (*Context).Current;
    let PciData: PPCI_COMMON_HEADER = (*Context).PciData;
    let PdoExtension: PPCI_PDO_EXTENSION = (*Context).PdoExtension;

    /* Only valid for PCI-to-PCI bridges */
    ntkrnl_sys::ASSERT!((*Current).BaseClass as u32 == PCI_CLASS_BRIDGE_DEV
            && (*Current).SubClass as u32 == PCI_SUBCLASS_BR_PCI_TO_PCI);

    /* Check for hacks first, then check the ProgIf of the bridge */
    if ((*PdoExtension).HackFlags & PCI_HACK_SUBTRACTIVE_DECODE) == 0
        && (*Current).ProgIf != 1
        && ((*PciData).u.type1.IOLimit & 0xF0) == 0xF0
    {
        /* A subtractive decode bridge would have a ProgIf 1, and no I/O limit */
        return 0u8;
    }

    /*
     * Check for Intel ICH PCI-to-PCI (i82801) bridges (used on the i810,
     * i820, i840, i845 Chipsets) that have subtractive decode broken.
     */
    if (((*PdoExtension).VendorId == 0x8086)
        && (((*PdoExtension).DeviceId == 0x2418)
            || ((*PdoExtension).DeviceId == 0x2428)
            || ((*PdoExtension).DeviceId == 0x244E)
            || ((*PdoExtension).DeviceId == 0x2448)))
        || ((*PdoExtension).HackFlags & PCI_HACK_BROKEN_SUBTRACTIVE_DECODE as u64) != 0
    {
        /* Check if the ACPI BIOS says positive decode should be enabled */
        if PciBridgeIsPositiveDecode(PdoExtension as *mut core::ffi::c_void) != 0 {
            /* Obey ACPI */
            return 0u8;
        }
    }

    /* If we found subtractive decode, we'll need a resource update later */
    (*PdoExtension).UpdateHardware = 1u8;
    1u8
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_SaveCurrentSettings(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let Status: NTSTATUS;
    let mut CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut IoDescriptor: PIO_RESOURCE_DESCRIPTOR;
    let Resources: PPCI_FUNCTION_RESOURCES;
    let mut BiosData: PCI_COMMON_HEADER = core::mem::zeroed();
    let mut Current: PPCI_COMMON_HEADER;
    let SavedConfig: PPCI_COMMON_CONFIG;
    let mut i: ULONG;
    let mut Bar: ULONG;
    let mut BarMask: ULONG;
    let BarArray: PULONG;
    let mut Limit: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Base: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Length: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut HaveIoLimit: BOOLEAN;
    let mut CheckAlignment: BOOLEAN;
    let PdoExtension: PPCI_PDO_EXTENSION;

    /* Get the pointers from the extension */
    PdoExtension = (*Context).PdoExtension;
    Resources = (*PdoExtension).Resources;
    Current = (*Context).Current;

    /* Check if decodes are disabled */
    if ((*Context).Command as u32 & (PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE)) == 0 {
        /* Well, we're going to need them from somewhere, use the registry data */
        Status = PciGetBiosConfig(PdoExtension, &mut BiosData as PPCI_COMMON_HEADER);
        if Status >= 0 {
            Current = &mut BiosData as PPCI_COMMON_HEADER;
        }
    }

    /* Scan all current and limit descriptors for each BAR needed */
    BarArray = (*Current).u.type1.BaseAddresses.as_mut_ptr();
    i = 0u32;
    while i < 6 {
        /* Get the current resource descriptor, and the limit requirement */
        CmDescriptor = core::ptr::addr_of_mut!((*Resources).Current[i as usize]);
        IoDescriptor = core::ptr::addr_of_mut!((*Resources).Limit[i as usize]);

        /* Copy descriptor data, skipping null descriptors */
        (*CmDescriptor).Type = (*IoDescriptor).Type;
        if (*CmDescriptor).Type as u32 == CmResourceTypeNull {
            i = i.wrapping_add(1);
            continue;
        }
        (*CmDescriptor).Flags = (*IoDescriptor).Flags;
        (*CmDescriptor).ShareDisposition = (*IoDescriptor).ShareDisposition;

        /* Initialize the high-parts to zero, since most stuff is 32-bit only */
        Base.QuadPart = 0;
        Limit.QuadPart = 0;
        Length.QuadPart = 0;

        /* Check if we're handling PCI BARs, or the ROM BAR */
        if i < PCI_TYPE1_ADDRESSES || i == 5 {
            /* Is this the ROM BAR? */
            if i == 5 {
                /* Read the correct bar, with the appropriate mask */
                Bar = (*Current).u.type1.ROMBaseAddress;
                BarMask = PCI_ADDRESS_ROM_ADDRESS_MASK;

                /* Decode the base address, and write down the length */
                Base.__bindgen_anon_1.LowPart = Bar & BarMask;
                (*CmDescriptor).u.Memory.Length = (*IoDescriptor).u.Memory.Length;
            } else {
                /* Otherwise, get the BAR from the array */
                Bar = *BarArray.add(i as usize);

                /* Is this an I/O BAR? */
                if Bar & PCI_ADDRESS_IO_SPACE != 0 {
                    /* Set the correct mask */
                    BarMask = PCI_ADDRESS_IO_ADDRESS_MASK;
                } else {
                    /* This is a memory BAR, set the correct base */
                    BarMask = PCI_ADDRESS_MEMORY_ADDRESS_MASK;

                    /* IS this a 64-bit BAR? */
                    if (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
                        /* Read the next 32-bits as well, ie, the next BAR */
                        Base.__bindgen_anon_1.HighPart = *BarArray.add((i + 1) as usize) as LONG;
                    }
                }

                /* Decode the base address, and write down the length */
                Base.__bindgen_anon_1.LowPart = Bar & BarMask;
                (*CmDescriptor).u.Generic.Length = (*IoDescriptor).u.Generic.Length;
            }
        } else {
            /* Reset loop conditions */
            HaveIoLimit = 0u8;
            CheckAlignment = 0u8;

            /* Check which descriptor is being parsed */
            if i == 2 {
                /* I/O Port Requirements */
                Base.__bindgen_anon_1.LowPart = PciBridgeIoBase(Current);
                Limit.__bindgen_anon_1.LowPart = PciBridgeIoLimit(Current);

                /* Do we have any I/O Port data? */
                if Base.__bindgen_anon_1.LowPart == 0 && (*Current).u.type1.IOLimit != 0 {
                    /* There's a limit */
                    HaveIoLimit = 1u8;
                }
            } else if i == 3 {
                /* Memory requirements */
                Base.__bindgen_anon_1.LowPart = PciBridgeMemoryBase(Current);
                Limit.__bindgen_anon_1.LowPart = PciBridgeMemoryLimit(Current);
                CheckAlignment = 1u8;
            } else if i == 4 {
                /* This should only be present for prefetch memory */
                Base = PciBridgePrefetchMemoryBase(Current);
                Limit = PciBridgePrefetchMemoryLimit(Current);
                CheckAlignment = 1u8;
            }

            /* Check for invalid base address */
            if Base.QuadPart >= Limit.QuadPart {
                /* Assume the descriptor is bogus */
                (*CmDescriptor).Type = CmResourceTypeNull as UCHAR;
                (*IoDescriptor).Type = CmResourceTypeNull as UCHAR;
                i = i.wrapping_add(1);
                continue;
            }

            /* Check if there's no memory, and no I/O port either */
            if Base.__bindgen_anon_1.LowPart == 0 && HaveIoLimit == 0 {
                /* This seems like a bogus requirement, ignore it */
                (*CmDescriptor).Type = CmResourceTypeNull as UCHAR;
                i = i.wrapping_add(1);
                continue;
            }

            /* Set the length to be the limit - the base; should always be 32-bit */
            Length.QuadPart = (Limit.__bindgen_anon_1.LowPart as LONGLONG)
                - (Base.__bindgen_anon_1.LowPart as LONGLONG)
                + 1;
            (*CmDescriptor).u.Generic.Length = Length.__bindgen_anon_1.LowPart;

            /* Check if alignment should be set */
            if CheckAlignment != 0 {
                /* Compute the required alignment for this length */
                (*IoDescriptor).u.Memory.Alignment =
                    PciBridgeMemoryWorstCaseAlignment((*CmDescriptor).u.Memory.Length);
            }
        }

        /* Now set the base address */
        (*CmDescriptor).u.Generic.Start.__bindgen_anon_1.LowPart = Base.__bindgen_anon_1.LowPart;

        i = i.wrapping_add(1);
    }

    /* Save PCI settings into the PDO extension for easy access later */
    (*PdoExtension).Dependent.type1.as_mut().PrimaryBus = (*Current).u.type1.PrimaryBus;
    (*PdoExtension).Dependent.type1.as_mut().SecondaryBus = (*Current).u.type1.SecondaryBus;
    (*PdoExtension).Dependent.type1.as_mut().SubordinateBus = (*Current).u.type1.SubordinateBus;

    /* Check for subtractive decode bridges */
    if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
        /* Check if legacy VGA decodes are enabled */
        if (*Current).u.type1.BridgeControl as u32 & PCI_ENABLE_BRIDGE_VGA != 0 {
            /* Save this setting for later */
            (*PdoExtension).Dependent.type1.as_mut().set_VgaBitSet(1u8);
        }
    } else {
        /* Check if legacy VGA decodes are enabled */
        if (*Current).u.type1.BridgeControl as u32 & PCI_ENABLE_BRIDGE_VGA != 0 {
            /* Save this setting for later */
            (*PdoExtension).Dependent.type1.as_mut().set_VgaBitSet(1u8);

            /* And on positive decode, we'll also need extra resources locked */
            (*PdoExtension).AdditionalResourceCount = 4u8;
        }

        /* Check if legacy ISA decoding is enabled */
        if (*Current).u.type1.BridgeControl as u32 & PCI_ENABLE_BRIDGE_ISA != 0 {
            /* Save this setting for later */
            (*PdoExtension).Dependent.type1.as_mut().set_IsaBitSet(1u8);
        }
    }

    /*
     * Check for Intel ICH PCI-to-PCI (i82801) bridges (used on the i810,
     * i820, i840, i845 Chipsets) that have subtractive decode broken.
     */
    if (((*PdoExtension).VendorId as i32 == 0x8086i32)
        && (((*PdoExtension).DeviceId as i32 == 0x2418i32)
            || ((*PdoExtension).DeviceId as i32 == 0x2428i32)
            || ((*PdoExtension).DeviceId as i32 == 0x244Ei32)
            || ((*PdoExtension).DeviceId as i32 == 0x2448i32)))
        || ((*PdoExtension).HackFlags & PCI_HACK_BROKEN_SUBTRACTIVE_DECODE != 0)
    {
        /* Check if subtractive decode is actually enabled */
        if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
            /* We're going to need a copy of the configuration for later use */
            SavedConfig = ExAllocatePoolWithTag(0, PCI_COMMON_HDR_LENGTH as SIZE_T, PCI_POOL_TAG as ULONG)
                as PPCI_COMMON_CONFIG;
            (*(*PdoExtension).ParentFdoExtension).PreservedConfig = SavedConfig;
            if !SavedConfig.is_null() {
                RtlCopyMemory(
                    SavedConfig as PVOID,
                    Current as *const c_void,
                    PCI_COMMON_HDR_LENGTH as SIZE_T,
                );
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_SaveLimits(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let working: PPCI_COMMON_HEADER = (*Context).PciData;
    let pdo_extension: PPCI_PDO_EXTENSION = (*Context).PdoExtension;
    let bar_array: PULONG = (*working).u.type1.BaseAddresses.as_mut_ptr();
    let limit: PIO_RESOURCE_DESCRIPTOR = (*(*pdo_extension).Resources).Limit.as_mut_ptr();
    let mut memory_limit: PHYSICAL_ADDRESS;
    let mut i: u32;

    i = 0;
    while i < PCI_TYPE1_ADDRESSES {
        /* Create a descriptor for their limits, the second BAR has no BAR after it */
        let next_bar: ULONG = if (i + 1) < PCI_TYPE1_ADDRESSES { *bar_array.offset((i + 1) as isize) } else { 0 };

        if unsafe { PciCreateIoDescriptorFromBarLimit(limit.offset(i as isize), *bar_array.offset(i as isize), next_bar, 0u8) } != 0 {
            ntkrnl_sys::ASSERT!((i + 1) < PCI_TYPE1_ADDRESSES);
            i += 1;
            (*limit.offset(i as isize)).Type = CmResourceTypeNull as u8;
        }
        i += 1;
    }

    if unsafe { PciBridgeIsSubtractiveDecode(Context) } != 0 {
        (*pdo_extension).Dependent.type1.as_mut().set_SubtractiveDecode(1u8);
        (*pdo_extension).Dependent.type1.as_mut().set_IsaBitSet(0u8);
        (*pdo_extension).Dependent.type1.as_mut().set_VgaBitSet(0u8);
    }

    if (*pdo_extension).Dependent.type1.as_ref().SubtractiveDecode() == 0 {
        i = PCI_TYPE1_ADDRESSES;
        while i < 5 {
            let limit_ptr = limit.offset(i as isize);
            memory_limit = core::mem::zeroed();
            (*limit_ptr).u.Port.MinimumAddress.QuadPart = 0;

            if i == 2 {
                ntkrnl_sys::ASSERT!((*working).u.type1.IOLimit != 0);
                ntkrnl_sys::ASSERT!((*working).u.type1.IOLimit & 0x0E == 0);
                memory_limit.__bindgen_anon_1.LowPart = unsafe { PciBridgeIoLimit(working) };
                (*limit_ptr).Type = CmResourceTypePort as u8;
                (*limit_ptr).Flags = (CM_RESOURCE_PORT_WINDOW_DECODE | CM_RESOURCE_PORT_POSITIVE_DECODE) as u16;
                (*limit_ptr).u.Port.Alignment = 0x1000;
                (*limit_ptr).u.Port.MinimumAddress.QuadPart = 0;
                (*limit_ptr).u.Port.MaximumAddress = memory_limit;
                (*limit_ptr).u.Port.Length = 0;
            } else if i == 3 {
                ntkrnl_sys::ASSERT!((*working).u.type1.MemoryLimit & 0xF == 0);
                memory_limit.__bindgen_anon_1.LowPart = unsafe { PciBridgeMemoryLimit(working) };
                (*limit_ptr).Type = CmResourceTypeMemory as u8;
                (*limit_ptr).Flags = (CM_RESOURCE_MEMORY_READ_WRITE) as u16;
                (*limit_ptr).u.Memory.Alignment = 0x100000;
                (*limit_ptr).u.Memory.MinimumAddress.QuadPart = 0;
                (*limit_ptr).u.Memory.MaximumAddress = memory_limit;
                (*limit_ptr).u.Memory.Length = 0;
            } else if (*working).u.type1.PrefetchLimit != 0 {
                memory_limit = unsafe { PciBridgePrefetchMemoryLimit(working) };
                (*limit_ptr).Type = CmResourceTypeMemory as u8;
                (*limit_ptr).Flags = (CM_RESOURCE_MEMORY_PREFETCHABLE) as u16;
                (*limit_ptr).u.Memory.Alignment = 0x100000;
                (*limit_ptr).u.Memory.MinimumAddress.QuadPart = 0;
                (*limit_ptr).u.Memory.MaximumAddress = memory_limit;
                (*limit_ptr).u.Memory.Length = 0;
            } else {
                (*limit_ptr).Type = CmResourceTypeNull as u8;
            }

            i += 1;
        }
    }

    if (*working).u.type1.ROMBaseAddress & PCI_ROMADDRESS_ENABLED != 0 {
        let rom_limit_ptr = limit.offset(i as isize);
        unsafe { PciCreateIoDescriptorFromBarLimit(rom_limit_ptr, (*working).u.type1.ROMBaseAddress, 0, 1u8); };
    }
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_MassageHeaderForLimitsDetermination(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let PciData = (*Context).PciData;
    let Current = (*Context).Current;

    let dest_ptr =
        core::ptr::addr_of_mut!((*PciData).u.type1.BaseAddresses) as *mut u8;
    let offset_base =
        core::mem::offset_of!(PCI_COMMON_HEADER, u.type1.BaseAddresses);
    let offset_cap =
        core::mem::offset_of!(PCI_COMMON_HEADER, u.type1.CapabilitiesPtr);
    let length = offset_cap - offset_base;
    core::ptr::write_bytes(dest_ptr, 0xFF, length);

    (*PciData).u.type1.PrimaryBus = (*Current).u.type1.PrimaryBus;
    (*PciData).u.type1.SecondaryBus = (*Current).u.type1.SecondaryBus;
    (*PciData).u.type1.SubordinateBus =
        (*Current).u.type1.SubordinateBus;
    (*PciData).u.type1.SecondaryLatency =
        (*Current).u.type1.SecondaryLatency;

    (*PciData).u.type1.IOBaseUpper16 = 0xFFFEu16;
    (*PciData).u.type1.IOLimitUpper16 = 0xFFFFu16;

    (*Context).SecondaryStatus = (*Current).u.type1.SecondaryStatus;

    (*Current).u.type1.SecondaryStatus = 0;
    (*PciData).u.type1.SecondaryStatus = 0;
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_RestoreCurrent(Context: *mut PCI_CONFIGURATOR_CONTEXT) {
    (*(*Context).Current).u.type1.SecondaryStatus = (*Context).SecondaryStatus;
}

#[no_mangle]
pub unsafe extern "system" fn PPBridge_GetAdditionalResourceDescriptors(
    Context: *mut _PCI_CONFIGURATOR_CONTEXT,
    PciData: PPCI_COMMON_HEADER,
    IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) {
    let _ = Context;

    if (*PciData).u.type1.BridgeControl & PCI_ENABLE_BRIDGE_VGA as u16 != 0 {
        (*IoDescriptor).Type = CmResourceTypeDevicePrivate as u8;
        (*IoDescriptor).u.DevicePrivate.Data[0] = 3;
        *core::ptr::addr_of_mut!((*IoDescriptor).u.DevicePrivate.Data).cast::<ULONG>().add(1) = 3;

        let iodesc1 = IoDescriptor.add(1);
        (*iodesc1).Type = CmResourceTypeMemory as u8;
        (*iodesc1).Flags = (CM_RESOURCE_MEMORY_READ_WRITE) as u16;
        (*iodesc1).u.Port.Length = 0x20000;
        (*iodesc1).u.Port.Alignment = 1;
        (*iodesc1).u.Port.MinimumAddress.QuadPart = 0xA0000;
        (*iodesc1).u.Port.MaximumAddress.QuadPart = 0xBFFFF;

        let iodesc2 = IoDescriptor.add(2);
        (*iodesc2).Type = CmResourceTypePort as u8;
        (*iodesc2).Flags = (CM_RESOURCE_PORT_POSITIVE_DECODE | CM_RESOURCE_PORT_10_BIT_DECODE) as u16;
        (*iodesc2).u.Port.Length = 12;
        (*iodesc2).u.Port.Alignment = 1;
        (*iodesc2).u.Port.MinimumAddress.QuadPart = 0x3B0;
        (*iodesc2).u.Port.MaximumAddress.QuadPart = 0x3BB;

        let iodesc3 = IoDescriptor.add(3);
        (*iodesc3).Type = CmResourceTypePort as u8;
        (*iodesc3).Flags = (CM_RESOURCE_PORT_POSITIVE_DECODE | CM_RESOURCE_PORT_10_BIT_DECODE) as u16;
        (*iodesc3).u.Port.Length = 32;
        (*iodesc3).u.Port.Alignment = 1;
        (*iodesc3).u.Port.MinimumAddress.QuadPart = 0x3C0;
        (*iodesc3).u.Port.MaximumAddress.QuadPart = 0x3DF;
    }
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_ResetDevice(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    UNREFERENCED_PARAMETER!(PdoExtension as *mut core::ffi::c_void);
    UNREFERENCED_PARAMETER!(PciData as *mut core::ffi::c_void);
    UNIMPLEMENTED_DBGBREAK();
}
#[no_mangle]
pub unsafe extern "system" fn PPBridge_ChangeResourceSettings(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    let fdo_extension: PPCI_FDO_EXTENSION;
    let pci_resources: PPCI_FUNCTION_RESOURCES;
    let i: u32;

    // C: ((VendorId==0x8086 && DeviceId∈{2418,2428,244E,2448}
    //      && (!(HackFlags & BROKEN_SUBTRACTIVE_DECODE) || SubtractiveDecode==FALSE))
    //     || SubtractiveDecode==FALSE)
    let is_8086 = (*PdoExtension).VendorId == 0x8086;
    let dev_match = (*PdoExtension).DeviceId == 0x2418
        || (*PdoExtension).DeviceId == 0x2428
        || (*PdoExtension).DeviceId == 0x244E
        || (*PdoExtension).DeviceId == 0x2448;
    let hack_ok = ((*PdoExtension).HackFlags & PCI_HACK_BROKEN_SUBTRACTIVE_DECODE as u64) == 0
        || (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() == 0u8;
    let subtractive = (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() == 0u8;
    if (is_8086 && dev_match && hack_ok) || subtractive
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
    } else {
        fdo_extension = (*PdoExtension).ParentFdoExtension;
        (*PciData).u.type1.IOBase = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.IOBase;
        (*PciData).u.type1.IOLimit = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.IOLimit;
        (*PciData).u.type1.MemoryBase = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.MemoryBase;
        (*PciData).u.type1.MemoryLimit = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.MemoryLimit;
        (*PciData).u.type1.PrefetchBase = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.PrefetchBase;
        (*PciData).u.type1.PrefetchLimit = (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.PrefetchLimit;
        (*PciData).u.type1.PrefetchBaseUpper32 =
            (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.PrefetchBaseUpper32;
        (*PciData).u.type1.PrefetchLimitUpper32 =
            (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.PrefetchLimitUpper32;
        (*PciData).u.type1.IOBaseUpper16 =
            (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.IOBaseUpper16;
        (*PciData).u.type1.IOLimitUpper16 =
            (*((*fdo_extension).PreservedConfig as PPCI_COMMON_HEADER)).u.type1.IOLimitUpper16;
    }

    // Write the assigned BARs and windows back. Upstream left this loop UNIMPLEMENTED,
    // so a positive-decode bridge kept the closed windows written above and forwarded
    // nothing: the OptiPlex's NVMe, behind root port 00:01.0, read all-ones. Current[]
    // holds what PciComputeNewCurrentSettings took from the START resources, indexed
    // as PPBridge_SaveCurrentSettings filled it: 0-1 the BARs, 2 the I/O window, 3 the
    // memory window, 4 the prefetchable window, 5 the ROM BAR.
    pci_resources = (*PdoExtension).Resources;
    if !pci_resources.is_null() {
        let bars = (*PciData).u.type1.BaseAddresses.as_mut_ptr();
        let mut i = 0usize;
        while i < 6 {
            let cm = core::ptr::addr_of!((*pci_resources).Current[i]);
            if (*cm).Type as u32 == CmResourceTypeNull {
                i += 1;
                continue;
            }
            let start = (*cm).u.Generic.Start.QuadPart as u64;
            let end = start + ((*cm).u.Generic.Length as u64).max(1) - 1;
            match i {
                0 | 1 => {
                    let bar = *bars.add(i);
                    if bar & !PCI_ADDRESS_IO_SPACE != 0 {
                        let mask = if bar & PCI_ADDRESS_IO_SPACE != 0 {
                            PCI_ADDRESS_IO_ADDRESS_MASK
                        } else {
                            PCI_ADDRESS_MEMORY_ADDRESS_MASK
                        };
                        *bars.add(i) = (bar & !mask) | (start as u32 & mask);
                        if bar & PCI_ADDRESS_IO_SPACE == 0
                            && bar & PCI_ADDRESS_MEMORY_TYPE_MASK == PCI_TYPE_64BIT
                            && i == 0
                        {
                            *bars.add(1) = (start >> 32) as u32;
                            i += 1;
                        }
                    }
                }
                2 => {
                    // Bits 7:4 of each byte are address 15:12; the low nibble is the
                    // read-only capability (16- or 32-bit decode) and ignores writes.
                    (*PciData).u.type1.IOBase = ((start >> 8) & 0xF0) as u8 | 0x01;
                    (*PciData).u.type1.IOLimit = ((end >> 8) & 0xF0) as u8 | 0x01;
                    (*PciData).u.type1.IOBaseUpper16 = (start >> 16) as u16;
                    (*PciData).u.type1.IOLimitUpper16 = (end >> 16) as u16;
                }
                3 => {
                    (*PciData).u.type1.MemoryBase = ((start >> 16) & 0xFFF0) as u16;
                    (*PciData).u.type1.MemoryLimit = ((end >> 16) & 0xFFF0) as u16;
                }
                4 => {
                    (*PciData).u.type1.PrefetchBase = ((start >> 16) & 0xFFF0) as u16 | 0x1;
                    (*PciData).u.type1.PrefetchLimit = ((end >> 16) & 0xFFF0) as u16 | 0x1;
                    (*PciData).u.type1.PrefetchBaseUpper32 = (start >> 32) as u32;
                    (*PciData).u.type1.PrefetchLimitUpper32 = (end >> 32) as u32;
                }
                _ => {
                    let rom = (*PciData).u.type1.ROMBaseAddress;
                    (*PciData).u.type1.ROMBaseAddress = (rom & !PCI_ADDRESS_ROM_ADDRESS_MASK)
                        | (start as u32 & PCI_ADDRESS_ROM_ADDRESS_MASK)
                        | PCI_ROMADDRESS_ENABLED;
                }
            }
            i += 1;
        }
    }

    (*PciData).u.type1.PrimaryBus = (*PdoExtension).Dependent.type1.as_ref().PrimaryBus;
    (*PciData).u.type1.SecondaryBus = (*PdoExtension).Dependent.type1.as_ref().SecondaryBus;
    (*PciData).u.type1.SubordinateBus = (*PdoExtension).Dependent.type1.as_ref().SubordinateBus;

    if (*PdoExtension).Dependent.type1.as_ref().IsaBitSet() != 0u8 {
        (*PciData).u.type1.BridgeControl |= PCI_ENABLE_BRIDGE_ISA as u16;
    }
    if (*PdoExtension).Dependent.type1.as_ref().VgaBitSet() != 0u8 {
        (*PciData).u.type1.BridgeControl |= PCI_ENABLE_BRIDGE_VGA as u16;
    }
}
