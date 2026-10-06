use crate::prelude::*;
use ntkrnl_sys::ddk::PCM_RESOURCE_LIST;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PataGetControllerProperties(
    Controller: *mut PATA_CONTROLLER,
    ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut ChanData: *mut CHANNEL_DATA_PATA;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    (*(*Controller)).MaxChannels = 1;
    (*(*Controller)).ChannelBitmap = 0x1;
    (*(*Controller)).Flags &= !CTRL_FLAG_NATIVE_PCI;
    (*(*Controller)).Flags |= CTRL_FLAG_MANUAL_RES;

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    ChanData = (*(*Controller)).Channels[0 as usize] as *mut CHANNEL_DATA_PATA;
    (*ChanData).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_IO32;
    (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL;

    Status = PciIdeParseResources(ChanData, ResourcesTranslated);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    #[cfg(target_arch = "x86")]
    {
        if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_CBUS != 0 {
            let mut Value: UCHAR;
            let mut Result: UCHAR;

            Value = READ_PORT_UCHAR(PC98_ATA_BANK as *mut UCHAR);
            WRITE_PORT_UCHAR(
                PC98_ATA_BANK as *mut UCHAR,
                Value ^ (PC98_ATA_BANK_32BIT_PORT) as u8,
            );
            Result = READ_PORT_UCHAR(PC98_ATA_BANK as *mut UCHAR);
            if ((Result ^ Value) as u32 & PC98_ATA_BANK_32BIT_PORT) != 0 {
                (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_IO32;
            }
        }
    }

    return STATUS_SUCCESS;
}
