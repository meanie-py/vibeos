//! Rust translation of `drivers/storage/ide/pciide/pciide.c` — the PCI IDE bus
//! mini-driver. DriverEntry registers PciIdeGetControllerProperties with the
//! pciidex framework (PciIdeXInitialize); the controller-properties callback
//! fills in the channel-enabled / transfer-mode / DMA callbacks and the
//! supported transfer modes. Faithful to the C as of upstream ae2827f4810 (DMA support).

use crate::ide_h::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PUNICODE_STRING};

unsafe extern "system" fn PciIdeChannelEnabled(
    _DeviceExtension: PVOID,
    _Channel: ULONG,
) -> IDE_CHANNEL_STATE {
    // Upstream ae2827f4810 dropped the class/header-type checks: the new pciidex only
    // loads this miniport on a controller it has already matched.
    ChannelStateUnknown
}

unsafe extern "system" fn PciIdeSyncAccessRequired(_DeviceExtension: PVOID) -> BOOLEAN {
    0 /* FALSE */
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
    DeviceExtension: PVOID,
    ControllerProperties: PIDE_CONTROLLER_PROPERTIES,
) -> NTSTATUS {
    if (*ControllerProperties).Size as usize != core::mem::size_of::<IDE_CONTROLLER_PROPERTIES>() {
        return STATUS_REVISION_MISMATCH;
    }

    // PciIdeTransferModeSelect / DefaultPIO / PciIdeUdmaModesSupported are no longer set:
    // the new pciidex picks transfer modes itself from SupportedTransferMode.
    (*ControllerProperties).PciIdeChannelEnabled = Some(PciIdeChannelEnabled);
    (*ControllerProperties).PciIdeSyncAccessRequired = Some(PciIdeSyncAccessRequired);
    (*ControllerProperties).IgnoreActiveBitForAtaDevice = 0 /* FALSE */;
    (*ControllerProperties).AlwaysClearBusMasterInterrupt = 1 /* TRUE */;
    (*ControllerProperties).PciIdeUseDma = Some(PciIdeUseDma);
    (*ControllerProperties).AlignmentRequirement = 1;

    // DMA only when the BIOS left bus mastering on. The C ignores the call's status and
    // reads PciCommand regardless; it is zero-initialised here so a failed read means
    // "no bus master" (PIO only) rather than stack garbage.
    let mut PciCommand: u16 = 0;
    PciIdeXGetBusData(
        DeviceExtension,
        core::ptr::addr_of_mut!(PciCommand) as PVOID,
        PCI_COMMAND_OFFSET,
        core::mem::size_of::<u16>() as ULONG,
    );
    let SupportedMode = if PciCommand & PCI_ENABLE_BUS_MASTER == 0 {
        PIO_MODE0 | PIO_MODE1 | PIO_MODE2 | PIO_MODE3 | PIO_MODE4
    } else {
        PIO_MODE0 | PIO_MODE1 | PIO_MODE2 | PIO_MODE3 | PIO_MODE4
            | SWDMA_MODE0 | SWDMA_MODE1 | SWDMA_MODE2
            | MWDMA_MODE0 | MWDMA_MODE1 | MWDMA_MODE2
            | UDMA_MODE0 | UDMA_MODE1 | UDMA_MODE2
            | UDMA_MODE3 | UDMA_MODE4 | UDMA_MODE5 | UDMA_MODE6
    };
    (*ControllerProperties).SupportedTransferMode[0][0] = SupportedMode;
    (*ControllerProperties).SupportedTransferMode[0][1] = SupportedMode;
    (*ControllerProperties).SupportedTransferMode[1][0] = SupportedMode;
    (*ControllerProperties).SupportedTransferMode[1][1] = SupportedMode;

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
