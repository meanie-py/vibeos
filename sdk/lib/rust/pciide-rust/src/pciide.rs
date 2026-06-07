//! Rust translation of `drivers/storage/ide/pciide/pciide.c` — the PCI IDE bus
//! mini-driver. DriverEntry registers PciIdeGetControllerProperties with the
//! pciidex framework (PciIdeXInitialize); the controller-properties callback
//! fills in the channel-enabled / transfer-mode / DMA callbacks and the
//! supported transfer modes. Faithful to the C.

use crate::ide_h::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PUNICODE_STRING};

unsafe extern "system" fn PciIdeChannelEnabled(
    DeviceExtension: PVOID,
    _Channel: ULONG,
) -> IDE_CHANNEL_STATE {
    let mut PciConfig: PCI_COMMON_CONFIG = core::mem::zeroed();

    let Status = PciIdeXGetBusData(
        DeviceExtension,
        core::ptr::addr_of_mut!(PciConfig) as PVOID,
        0,
        PCI_COMMON_HDR_LENGTH,
    );
    if !NT_SUCCESS(Status) {
        return ChannelStateUnknown;
    }

    // PCI_CONFIGURATION_TYPE(p) == (p->HeaderType & ~PCI_MULTIFUNCTION)
    if (PciConfig.HeaderType & !PCI_MULTIFUNCTION) as u32 != PCI_DEVICE_TYPE {
        return ChannelDisabled;
    }

    if PciConfig.BaseClass as u32 != PCI_CLASS_MASS_STORAGE_CTLR
        || PciConfig.SubClass as u32 != PCI_SUBCLASS_MSC_IDE_CTLR
    {
        return ChannelDisabled;
    }

    ChannelStateUnknown
}

unsafe extern "system" fn PciIdeSyncAccessRequired(_DeviceExtension: PVOID) -> BOOLEAN {
    0 /* FALSE; FIXME */
}

unsafe extern "system" fn PciIdeTransferModeSelect(
    _DeviceExtension: PVOID,
    XferMode: PPCIIDE_TRANSFER_MODE_SELECT,
) -> NTSTATUS {
    let mut i = 0usize;
    while i < MAX_IDE_DEVICE {
        (*XferMode).DevicePresent[i] = 0 /* FALSE; FIXME */;
        i += 1;
    }
    STATUS_SUCCESS
}

unsafe extern "system" fn PciIdeUseDma(
    _DeviceExtension: PVOID,
    _CdbCommand: PUCHAR,
    _Slave: PUCHAR,
) -> ULONG {
    /* Nothing should prevent us to use DMA */
    1
}

unsafe extern "system" fn PciIdeGetControllerProperties(
    _DeviceExtension: PVOID,
    ControllerProperties: PIDE_CONTROLLER_PROPERTIES,
) -> NTSTATUS {
    if (*ControllerProperties).Size as usize != core::mem::size_of::<IDE_CONTROLLER_PROPERTIES>() {
        return STATUS_REVISION_MISMATCH;
    }

    (*ControllerProperties).PciIdeChannelEnabled = Some(PciIdeChannelEnabled);
    (*ControllerProperties).PciIdeSyncAccessRequired = Some(PciIdeSyncAccessRequired);
    (*ControllerProperties).PciIdeTransferModeSelect = Some(PciIdeTransferModeSelect);
    (*ControllerProperties).IgnoreActiveBitForAtaDevice = 0 /* FALSE */;
    (*ControllerProperties).AlwaysClearBusMasterInterrupt = 1 /* TRUE */;
    (*ControllerProperties).PciIdeUseDma = Some(PciIdeUseDma);
    (*ControllerProperties).AlignmentRequirement = 1;
    (*ControllerProperties).DefaultPIO = 0 /* FIXME */;
    (*ControllerProperties).PciIdeUdmaModesSupported = None; /* optional */

    let modes = PIO_MODE0 | PIO_MODE1 | PIO_MODE2 | PIO_MODE3 | PIO_MODE4
        | SWDMA_MODE0 | SWDMA_MODE1 | SWDMA_MODE2
        | MWDMA_MODE0 | MWDMA_MODE1 | MWDMA_MODE2
        | UDMA_MODE0 | UDMA_MODE1 | UDMA_MODE2 | UDMA_MODE3 | UDMA_MODE4;
    (*ControllerProperties).SupportedTransferMode[0][0] = modes;
    (*ControllerProperties).SupportedTransferMode[0][1] = modes;
    (*ControllerProperties).SupportedTransferMode[1][0] = modes;
    (*ControllerProperties).SupportedTransferMode[1][1] = modes;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    PciIdeXInitialize(
        DriverObject,
        RegistryPath,
        Some(PciIdeGetControllerProperties),
        0,
    )
}
