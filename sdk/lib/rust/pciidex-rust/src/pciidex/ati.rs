use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AtiEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x48 as UCHAR,
        Mask: 0x1 as UCHAR,
        ValueEnabled: 0 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0x48 as UCHAR,
        Mask: 0x8 as UCHAR,
        ValueEnabled: 0 as UCHAR,
    },
];

pub const PCI_DEV_IXP200_IDE: u32 = 0x4349;
pub const PCI_DEV_IXP300_IDE: u32 = 0x4369;
pub const PCI_DEV_IXP400_IDE: u32 = 0x4376;
pub const PCI_DEV_IXP600_IDE: u32 = 0x438C;
pub const PCI_DEV_IXP700_IDE: u32 = 0x439C;
#[no_mangle]
pub unsafe extern "C" fn AtiGetControllerProperties(
    Controller: *mut ATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    PAGED_CODE!();

    if ((*Controller).Pci.DeviceID) as u32 != PCI_DEV_IXP200_IDE
        && ((*Controller).Pci.DeviceID) as u32 != PCI_DEV_IXP300_IDE
        && ((*Controller).Pci.DeviceID) as u32 != PCI_DEV_IXP400_IDE
        && ((*Controller).Pci.DeviceID) as u32 != PCI_DEV_IXP600_IDE
        && ((*Controller).Pci.DeviceID) as u32 != PCI_DEV_IXP700_IDE
    {
        return STATUS_NO_MATCH;
    }

    (*Controller).__bindgen_anon_1.ChannelEnableBits = AtiEnableBits.as_ptr();

    if ((*Controller).Pci.DeviceID) as u32 == PCI_DEV_IXP600_IDE {
        (*Controller).MaxChannels = 1;
    }

    Status = PciIdeCreateChannelData((Controller).cast(), 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: *mut CHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(SvwSetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL
            | MWDMA_ALL
            | UDMA_MODES!(0, 5);

        if ((*Controller).Pci.DeviceID) as u32 == PCI_DEV_IXP200_IDE {
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_MODE5;
        }

        if SvwHasUdmaCable(Controller, i) == 0 {
            INFO!(
                b"CH %lu: BIOS hasn't selected mode faster than UDMA 2, assume 40-conductor cable\n\0"
                    .as_ptr() as *const i8,
                (*ChanData).Channel,
            );
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
