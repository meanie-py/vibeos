// pci/express.rs — drivers/bus/pcix/pci/express.c (upstream 7baff0a9338, #9543): PCI
// Express capability discovery and the extended capability list walk. Hand-ported, $0.
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PCI_EXPRESS_CAPABILITY, PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER, PCSTR,
    PPCI_CAPABILITIES_HEADER, PPCI_EXPRESS_ENHANCED_CAPABILITY_HEADER, RtlAssert,
};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

use super::ecam::PciEcamVerified;

const HEADER_SIZE: ULONG = core::mem::size_of::<PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER>() as ULONG;

/* FUNCTIONS ******************************************************************/

fn PciIsExtendedCapabilityOffsetValid(Offset: ULONG) -> BOOLEAN {
    if (Offset & 3) != 0 || Offset < PCI_LEGACY_CONFIG_LENGTH {
        return 0;
    }

    (Offset + HEADER_SIZE <= PCI_EXTENDED_CONFIG_LENGTH) as BOOLEAN
}

/// Finds an extended capability and reads it into the caller's buffer. Returns the
/// offset of the capability, or 0 if it was not found.
#[no_mangle]
pub unsafe extern "system" fn PciReadDeviceExtendedCapability(
    DeviceExtension: PPCI_PDO_EXTENSION,
    CapabilityId: ULONG,
    Buffer: PPCI_EXPRESS_ENHANCED_CAPABILITY_HEADER,
    Length: ULONG,
) -> ULONG {
    let mut Header: PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER = core::mem::zeroed();
    let mut Offset: ULONG;
    let mut EntriesLeft: ULONG;

    if (*DeviceExtension).ExtensionType != PciPdoExtensionType {
        RtlAssert(
            b"DeviceExtension->ExtensionType == PciPdoExtensionType\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/pci/express.c\0".as_ptr() as PVOID,
            46,
            core::ptr::null_mut(),
        );
    }
    if Length < HEADER_SIZE {
        RtlAssert(
            b"Length >= sizeof(PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER)\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/pci/express.c\0".as_ptr() as PVOID,
            47,
            core::ptr::null_mut(),
        );
    }

    if (*DeviceExtension).IsExtendedConfigReachable == 0 {
        return 0;
    }

    /* More entries than aligned offsets means the list loops */
    EntriesLeft = (PCI_EXTENDED_CONFIG_LENGTH - PCI_LEGACY_CONFIG_LENGTH) / core::mem::size_of::<ULONG>() as ULONG;

    Offset = PCI_LEGACY_CONFIG_LENGTH;
    while Offset != 0 {
        if PciIsExtendedCapabilityOffsetValid(Offset) == 0 {
            return 0;
        }

        if EntriesLeft == 0 {
            DbgPrint(
                b"(%s:%d) PCI device %p extended capabilities list is broken\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/pci/express.c\0".as_ptr() as PCSTR,
                65,
                DeviceExtension,
            );
            return 0;
        }
        EntriesLeft -= 1;

        PciReadDeviceConfig(DeviceExtension, &mut Header as *mut _ as PVOID, Offset, HEADER_SIZE);

        /* A function that does not decode extended space reads back all ones */
        if Header.CapabilityID == 0xFFFF {
            return 0;
        }

        if Header.CapabilityID as ULONG == CapabilityId {
            if Length > (PCI_EXTENDED_CONFIG_LENGTH - Offset) {
                return 0;
            }

            PciReadDeviceConfig(DeviceExtension, Buffer as PVOID, Offset, Length);
            return Offset;
        }

        Offset = Header.Next() as ULONG;
    }

    0
}

/// Records the Express capability of a new function and whether its extended
/// configuration space can be reached.
#[no_mangle]
pub unsafe extern "system" fn PciGetExpressCapabilities(PdoExtension: PPCI_PDO_EXTENSION) {
    let mut Express: PCI_EXPRESS_CAPABILITY = core::mem::zeroed();
    let ParentFdo: PPCI_FDO_EXTENSION;
    let ParentBridge: PPCI_PDO_EXTENSION;
    let Offset: UCHAR;
    PAGED_CODE();

    (*PdoExtension).ExpressCapabilityPtr = 0;
    (*PdoExtension).ExpressDeviceType = 0;
    (*PdoExtension).IsExtendedConfigReachable = 0;

    if (*PdoExtension).CapabilitiesPtr == 0 {
        return;
    }

    if ((*PdoExtension).HackFlags as u64 & PCI_HACK_NO_EXPRESS_CAP) != 0 {
        return;
    }

    /* Only the capabilities register is needed */
    Offset = PciReadDeviceCapability(
        PdoExtension,
        (*PdoExtension).CapabilitiesPtr,
        PCI_CAPABILITY_ID_PCI_EXPRESS,
        core::ptr::addr_of_mut!(Express.Header) as PPCI_CAPABILITIES_HEADER,
        (core::mem::offset_of!(PCI_EXPRESS_CAPABILITY, ExpressCapabilities)
            + core::mem::size_of_val(&Express.ExpressCapabilities)) as ULONG,
    );
    if Offset == 0 {
        return;
    }

    (*PdoExtension).ExpressCapabilityPtr = Offset as USHORT;
    (*PdoExtension).ExpressDeviceType = Express.ExpressCapabilities.__bindgen_anon_1.DeviceType() as UCHAR;

    /* Every bridge between the root and this function has to pass extended config cycles */
    ParentFdo = (*PdoExtension).ParentFdoExtension;
    if (*ParentFdo).BusRootFdoExtension == ParentFdo {
        (*PdoExtension).IsExtendedConfigReachable = PciEcamVerified;
    } else {
        ParentBridge = (*(*ParentFdo).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
        ASSERT_PDO(ParentBridge);
        (*PdoExtension).IsExtendedConfigReachable = (*ParentBridge).IsExtendedConfigReachable;
    }

    DbgPrint(
        b"(%s:%d) PCI - Express capability at 0x%x, port type %u, version %u\n\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/pci/express.c\0".as_ptr() as PCSTR,
        144,
        (*PdoExtension).ExpressCapabilityPtr as u32,
        (*PdoExtension).ExpressDeviceType as u32,
        Express.ExpressCapabilities.__bindgen_anon_1.CapabilityVersion() as u32,
    );
}
