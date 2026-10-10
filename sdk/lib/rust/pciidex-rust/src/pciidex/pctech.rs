use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const PCI_DEV_RZ1000: u32 = 0x1000;
pub const PCI_DEV_RZ1001: u32 = 0x1001;
pub const REG_CONFIG: u32 = 0x40;
pub const CONFIG_PREFETCH: u32 = 0x2000;

unsafe extern "C" fn PcTechControllerStart(Controller: PATA_CONTROLLER) {
    let mut ConfigReg: USHORT;

    ConfigReg = PciRead16(Controller, REG_CONFIG);
    ConfigReg &= (!CONFIG_PREFETCH) as u16;
    PciWrite16(Controller, REG_CONFIG, ConfigReg);
}
#[no_mangle]
pub unsafe extern "C" fn PcTechGetControllerProperties(
    Controller: *mut ATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID as u32 == PCI_VEN_PC_TECH);

    if (*Controller).Pci.DeviceID != (PCI_DEV_RZ1000 as u16) && (*Controller).Pci.DeviceID != (PCI_DEV_RZ1001 as u16) {
        return STATUS_NO_MATCH;
    }

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Controller).Start = Some(PcTechControllerStart);

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: *mut CHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as *mut CHANNEL_DATA_PATA;

        (*ChanData).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_IO32;
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
