#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCM_PARTIAL_RESOURCE_DESCRIPTOR, PIO_RESOURCE_DESCRIPTOR, PPCI_COMMON_HEADER};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::{DbgPrint};
use ntkrnl_sys::ke::{DbgBreakPoint};

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

#[no_mangle]
pub unsafe extern "system" fn Device_SaveCurrentSettings(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let PciData: PPCI_COMMON_HEADER;
    let mut IoDescriptor: PIO_RESOURCE_DESCRIPTOR;
    let mut CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Resources: PPCI_FUNCTION_RESOURCES;
    let BarArray: PULONG;
    let mut Bar: ULONG;
    let mut BarMask: ULONG;
    let mut i: ULONG;

    /* Get variables from context */
    PciData = (*Context).Current;
    Resources = (*(*Context).PdoExtension).Resources;

    /* Loop all the PCI BARs */
    BarArray = (*PciData).u.type0.BaseAddresses.as_mut_ptr();
    i = 0u32;
    while i <= PCI_TYPE0_ADDRESSES {
        /* Get the resource descriptor and limit descriptor for this BAR */
        CmDescriptor = core::ptr::addr_of_mut!((*Resources).Current[i as usize]);
        IoDescriptor = core::ptr::addr_of_mut!((*Resources).Limit[i as usize]);

        /* Build the resource descriptor based on the limit descriptor */
        (*CmDescriptor).Type = (*IoDescriptor).Type;
        if (*CmDescriptor).Type as u32 == CmResourceTypeNull {
            i = i.wrapping_add(1);
            continue;
        }
        (*CmDescriptor).Flags = (*IoDescriptor).Flags;
        // Divergence from the C, which copies the limit's disposition: the limits never
        // set one (0, undetermined), so the boot configuration claimed a BAR could be
        // shared, and PciComputeNewCurrentSettings skips non-exclusive descriptors when
        // it maps an assigned list back onto BARs. A BAR decodes for one function only;
        // the requirements list already says so (PciBuildRequirementsList).
        (*CmDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
        (*CmDescriptor).u.Generic.Start.__bindgen_anon_1.HighPart = 0;
        (*CmDescriptor).u.Generic.Length = (*IoDescriptor).u.Generic.Length;

        /* Check if we're handling PCI BARs, or the ROM BAR */
        if i < PCI_TYPE0_ADDRESSES {
            /* Read the actual BAR value */
            Bar = *BarArray.add(i as usize);

            /* Check if this is an I/O BAR */
            if Bar & PCI_ADDRESS_IO_SPACE != 0 {
                /* Use the right mask to get the I/O port base address */
                if (*CmDescriptor).Type as u32 != CmResourceTypePort {
                    RtlAssert(
                        b"CmDescriptor->Type == CmResourceTypePort\0".as_ptr() as *const i8 as PVOID,
                        b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8 as PVOID,
                        59u32,
                        core::ptr::null_mut::<i8>(),
                    );
                }
                BarMask = PCI_ADDRESS_IO_ADDRESS_MASK;
            } else {
                /* It's a RAM BAR, use the right mask to get the base address */
                if (*CmDescriptor).Type as u32 != CmResourceTypeMemory
                    && (*CmDescriptor).Type as u32 != CmResourceTypeMemoryLarge
                {
                    RtlAssert(
                        b"(CmDescriptor->Type == CmResourceTypeMemory) || (CmDescriptor->Type == CmResourceTypeMemoryLarge)\0".as_ptr() as *const i8 as PVOID,
                        b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8 as PVOID,
                        65u32,
                        core::ptr::null_mut::<i8>(),
                    );
                }
                BarMask = PCI_ADDRESS_MEMORY_ADDRESS_MASK;

                /* Check if it's a 64-bit BAR */
                if (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
                    /* The next BAR value is actually the high 32-bits */
                    (*CmDescriptor).u.Memory.Start.__bindgen_anon_1.HighPart = *BarArray.add((i + 1) as usize) as LONG;
                } else if (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_20BIT {
                    /* Legacy BAR, don't read more than 20 bits of the address */
                    BarMask = 0xFFFF0u32;
                }
            }
        } else {
            /* Actually a ROM BAR, so read the correct register */
            Bar = (*PciData).u.type0.ROMBaseAddress;

            /* Apply the correct mask for ROM BARs */
            BarMask = PCI_ADDRESS_ROM_ADDRESS_MASK;

            /* Make sure it's enabled */
            if Bar & PCI_ROMADDRESS_ENABLED == 0 {
                /* If it isn't, then a descriptor won't be built for it */
                (*CmDescriptor).Type = CmResourceTypeNull as UCHAR;
                i = i.wrapping_add(1);
                continue;
            }
        }

        /* Now we have the right mask, read the actual address from the BAR */
        Bar &= BarMask;
        (*CmDescriptor).u.Memory.Start.__bindgen_anon_1.LowPart = Bar;

        /* And check for invalid BAR addresses */
        if ((*CmDescriptor).u.Memory.Start.__bindgen_anon_1.HighPart as ULONG | Bar) == 0 {
            /* Skip these descriptors */
            (*CmDescriptor).Type = CmResourceTypeNull as UCHAR;
            DbgPrint(
                b"%s: Invalid BAR\n\0".as_ptr() as PCSTR,
                b"PCI\0".as_ptr() as *const i8,
            );
        }

        i = i.wrapping_add(1);
    }

    /* Also save the sub-IDs that came directly from the PCI header */
    (*(*Context).PdoExtension).SubsystemVendorId = (*PciData).u.type0.SubVendorID;
    (*(*Context).PdoExtension).SubsystemId = (*PciData).u.type0.SubSystemID;
}

#[no_mangle]
pub unsafe extern "system" fn Device_SaveLimits(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let mut Current: PPCI_COMMON_HEADER;
    let mut PciData: PPCI_COMMON_HEADER;
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut BarArray: PULONG;
    let mut Limit: PIO_RESOURCE_DESCRIPTOR;
    let mut i: ULONG;

    PdoExtension = (*Context).PdoExtension;
    Current = (*Context).Current;
    PciData = (*Context).PciData;

    BarArray = core::ptr::addr_of_mut!((*PciData).u.type0.BaseAddresses) as PULONG;

    if (*PdoExtension).BaseClass as i32 == PCI_CLASS_MASS_STORAGE_CTLR as i32
        && (*PdoExtension).SubClass as i32 == PCI_SUBCLASS_MSC_IDE_CTLR as i32
        && ((*PdoExtension).ProgIf as i32 & 5i32) != 5i32
    {
        *BarArray = 0u32;
        *BarArray.add(1) = 0u32;
        *BarArray.add(2) = 0u32;
        *BarArray.add(3) = 0u32;
    } else if (*PdoExtension).VendorId as i32 == 0x5333i32
        && ((*PdoExtension).DeviceId as i32 == 0x88F0i32 || (*PdoExtension).DeviceId as i32 == 0x8880i32)
    {
        i = 0u32;
        while i < PCI_TYPE0_ADDRESSES {
            if *BarArray.add(i as usize) == 0xFE000000u32 {
                *BarArray.add(i as usize) = 0xFC000000u32;
                if DbgPrint(
                    b"(%s:%d) PCI - Adjusted broken S3 requirement from 32MB to 64MB\n\0".as_ptr() as PCSTR,
                    b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                    163i32,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                        b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                        163i32,
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }

    if (*PdoExtension).VendorId as i32 == 0x1013i32 && (*PdoExtension).DeviceId as i32 == 0xA0i32 {
        if *BarArray.add(1) == 0xFC01u32 {
            if (*Current).u.type0.BaseAddresses[1usize] == 1u32 {
                *BarArray.add(1) = 0u32;
                if DbgPrint(
                    b"(%s:%d) PCI - Ignored Cirrus GD54xx broken IO requirement (400 ports)\n\0".as_ptr() as PCSTR,
                    b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                    179i32,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                        b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                        179i32,
                    );
                }
            } else if DbgPrint(
                b"(%s:%d) PCI - Cirrus GD54xx 400 port IO requirement has a valid setting (%08x)\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                185i32,
                (*Current).u.type0.BaseAddresses[1usize],
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                    185i32,
                );
            }
        } else if *BarArray.add(1) != 0 {
            if DbgPrint(
                b"(%s:%d) PCI - Warning Cirrus Adapter 101300a0 has unexpected resource requirement (%08x)\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                192i32,
                *BarArray.add(1),
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8,
                    192i32,
                );
            }
        }
    }

    Limit = core::ptr::addr_of_mut!((*(*PdoExtension).Resources).Limit) as PIO_RESOURCE_DESCRIPTOR;
    i = 0u32;
    while i < PCI_TYPE0_ADDRESSES {
        /* And build them based on the BARs, the last one has no BAR after it for a high half */
        let NextBar: ULONG = if i.wrapping_add(1) < PCI_TYPE0_ADDRESSES {
            *BarArray.add(i as usize + 1)
        } else {
            0
        };

        if PciCreateIoDescriptorFromBarLimit(
            Limit.add(i as usize) as PIO_RESOURCE_DESCRIPTOR,
            *BarArray.add(i as usize),
            NextBar,
            0u8,
        ) != 0
        {
            if !(i.wrapping_add(1u32) < PCI_TYPE0_ADDRESSES) {
                RtlAssert(
                    b"(i + 1) < PCI_TYPE0_ADDRESSES\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/bus/pcix/device.c\0".as_ptr() as *const i8 as PVOID,
                    204u32,
                    core::ptr::null_mut::<i8>(),
                );
            }
            i = i.wrapping_add(1);
            (*Limit.add(i as usize)).Type = CmResourceTypeNull as UCHAR;
        }
        i = i.wrapping_add(1);
    }

    PciCreateIoDescriptorFromBarLimit(
        Limit.add(i as usize) as PIO_RESOURCE_DESCRIPTOR,
        (*PciData).u.type0.ROMBaseAddress,
        0,
        1u8,
    );
}
#[no_mangle]
pub unsafe extern "system" fn Device_MassageHeaderForLimitsDetermination(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let PdoExtension = (*Context).PdoExtension;
    let PciData = (*Context).PciData;
    let BarArray = (*PciData).u.type0.BaseAddresses.as_mut_ptr();
    let mut i: ULONG;

    if (*PdoExtension).BaseClass as u32 == PCI_CLASS_MASS_STORAGE_CTLR as u32
        && (*PdoExtension).SubClass as u32 == PCI_SUBCLASS_MSC_IDE_CTLR as u32
        && ((*PdoExtension).ProgIf as u32 & 5u32) != 5u32
    {
    }

    i = 0;
    while (i as usize) < PCI_TYPE0_ADDRESSES as usize {
        *BarArray.add(i as usize) = 0xFFFFFFFF_u32;
        i += 1;
    }

    (*PciData).u.type0.ROMBaseAddress = PCI_ADDRESS_ROM_ADDRESS_MASK as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn Device_RestoreCurrent(Context: PPCI_CONFIGURATOR_CONTEXT) {
    let _ = Context;
    /* Nothing to do for devices */
}

#[no_mangle]
pub unsafe extern "system" fn Device_GetAdditionalResourceDescriptors(
    mut Context: PPCI_CONFIGURATOR_CONTEXT,
    mut PciData: PPCI_COMMON_HEADER,
    mut IoDescriptor: PIO_RESOURCE_DESCRIPTOR,
) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"Device_GetAdditionalResourceDescriptors\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/bus/pcix/device.c\0" as *const u8
            as *const ::core::ffi::c_char,
        267 as ::core::ffi::c_int,
    );
    DbgPrint(b"\0" as *const u8 as PCSTR);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn Device_ResetDevice(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"Device_ResetDevice\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/device.c\0".as_ptr()
            as *const core::ffi::c_char,
        278i32,
    );
    DbgPrint(b"\0".as_ptr() as *const core::ffi::c_char);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn Device_ChangeResourceSettings(
    PdoExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) {
    let Resources = (*PdoExtension).Resources;
    if Resources.is_null() {
        return;
    }

    /* Write each BAR back with the address that was arbitrated for it */
    let BarArray: PULONG = core::ptr::addr_of_mut!((*PciData).u.type0.BaseAddresses) as PULONG;
    let mut i: ULONG = 0;
    while i <= PCI_TYPE0_ADDRESSES {
        let CmDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            core::ptr::addr_of_mut!((*Resources).Current[i as usize]);

        if i < PCI_TYPE0_ADDRESSES {
            /* A BAR the function does not implement reads back as zero */
            let Bar: ULONG = *BarArray.add(i as usize);
            if (Bar & !PCI_ADDRESS_IO_SPACE) == 0 {
                i += 1;
                continue;
            }

            /* Work out which bits of this BAR are the address */
            let mut BarMask: ULONG;
            if (Bar & PCI_ADDRESS_IO_SPACE) != 0 {
                BarMask = PCI_ADDRESS_IO_ADDRESS_MASK;
            } else {
                BarMask = PCI_ADDRESS_MEMORY_ADDRESS_MASK;

                /* A legacy BAR only decodes the low 20 bits of the address */
                if (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_20BIT {
                    BarMask = 0xFFFF0;
                }
            }

            *BarArray.add(i as usize) = (Bar & !BarMask)
                | ((*CmDescriptor).u.Memory.Start.__bindgen_anon_1.LowPart & BarMask);

            /* A 64-bit BAR keeps the high half of its address in the next one */
            if (Bar & PCI_ADDRESS_IO_SPACE) == 0
                && (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT
            {
                ntkrnl_sys::ASSERT!((i + 1) < PCI_TYPE0_ADDRESSES);
                *BarArray.add(i as usize + 1) =
                    (*CmDescriptor).u.Memory.Start.__bindgen_anon_1.HighPart as ULONG;
                i += 1;
            }
        } else {
            /* The ROM BAR decodes only while its enable bit is set */
            let Bar: ULONG = (*PciData).u.type0.ROMBaseAddress;
            if (*CmDescriptor).Type as u32 == CmResourceTypeNull {
                (*PciData).u.type0.ROMBaseAddress = Bar & !PCI_ROMADDRESS_ENABLED;
            } else {
                (*PciData).u.type0.ROMBaseAddress = (Bar & !PCI_ADDRESS_ROM_ADDRESS_MASK)
                    | ((*CmDescriptor).u.Memory.Start.__bindgen_anon_1.LowPart
                        & PCI_ADDRESS_ROM_ADDRESS_MASK)
                    | PCI_ROMADDRESS_ENABLED;
            }
        }
        i += 1;
    }
}
