use crate::prelude::*;
use ntkrnl_sys::ddk::{DeviceNumber, KeStallExecutionProcessor};
use core::ffi::c_void;

unsafe extern "C" fn PataIsDevicePresent(
    ChanData: PCHANNEL_DATA_PATA,
    DeviceNumber: ULONG,
) -> BOOLEAN {
    let mut IdeStatus: UCHAR;

    ATA_SELECT_DEVICE(
        ChanData,
        (DeviceNumber) as UCHAR,
        (IDE_DRIVE_SELECT | ((DeviceNumber & 1) << 4)) as UCHAR,
    );

    IdeStatus = (*ChanData).ReadStatus.expect("non-null function pointer")(ChanData);
    INFO!(
        b"CH %lu: Device %lu status %02x\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        DeviceNumber,
        IdeStatus,
    );
    if IdeStatus == 0xFF || IdeStatus == 0x7F {
        return FALSE;
    }

    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
        0x55,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
        0xAA,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
        0x55,
        ChanData,
        MRES_TF,
    );
    if ATA_READ!(
        (*ChanData).Regs.__bindgen_anon_3.ByteCountLow,
        ChanData,
        MRES_TF,
    ) != 0x55
    {
        return FALSE;
    }
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
        0xAA,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
        0x55,
        ChanData,
        MRES_TF,
    );
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
        0xAA,
        ChanData,
        MRES_TF,
    );
    if ATA_READ!(
        (*ChanData).Regs.__bindgen_anon_4.ByteCountHigh,
        ChanData,
        MRES_TF,
    ) != 0xAA
    {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn PataIssueSoftwareReset(ChanData: *mut CHANNEL_DATA_PATA) {
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_6.Control,
        (IDE_DC_RESET_CONTROLLER | (IDE_DC_ALWAYS) as _) as u8,
        ChanData,
        MRES_CTRL,
    );
    KeStallExecutionProcessor(20);
    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_6.Control,
        (IDE_DC_DISABLE_INTERRUPTS | (IDE_DC_ALWAYS) as _) as u8,
        ChanData,
        MRES_CTRL,
    );
    KeStallExecutionProcessor(20);
}

unsafe extern "C" fn PataDrainDeviceBuffer(ChanData: PCHANNEL_DATA_PATA) {
    let mut i: ULONG;
    let mut LocalBuffer: USHORT;

    i = 0;
    while i < ATA_MAX_TRANSFER_LENGTH / core::mem::size_of::<USHORT>() as ULONG {
        let IdeStatus: UCHAR = (*ChanData).ReadStatus.expect("non-null function pointer")(ChanData);

        if IdeStatus as i32 & IDE_STATUS_DRQ == 0 {
            break;
        }

        ATA_READ_BLOCK_16!(
            (*ChanData).Regs.Data as PUSHORT,
            &raw mut LocalBuffer,
            1,
            ChanData,
            MRES_TF,
        );

        i = i.wrapping_add(1);
    }

    if i > 0 {
        INFO!(
            b"CH %lu: Total drained %lu bytes\n\0".as_ptr() as *const i8,
            (*ChanData).Channel,
            i * core::mem::size_of::<USHORT>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn PataChannelGetMaximumDeviceCount(
    ChannelContext: *mut core::ffi::c_void,
) -> u32 {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;

    #[cfg(target_arch = "x86")]
    {
        if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_CBUS != 0 {
            return 4;
        }
    }

    return if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_NO_SLAVE != 0 {
        1
    } else {
        2
    };
}
#[no_mangle]
pub unsafe extern "C" fn PataResetChannel(ChannelContext: *mut core::ffi::c_void) {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let mut i: u32;
    let mut DeviceNumber: u32;
    let mut DeviceCount: u32;
    let mut DeviceBitmap: u32 = 0;

    (*ChanData).IsPollingActive = FALSE;

    if !(*ChanData).Regs.Dma.is_null() {
        PciIdeDmaStop(ChanData);
    }

    PataDrainDeviceBuffer(ChanData);

    DeviceCount = PataChannelGetMaximumDeviceCount((ChanData).cast());
    DeviceNumber = 0;
    while DeviceNumber < DeviceCount {
        if PataIsDevicePresent(ChanData, DeviceNumber) != 0 {
            DeviceBitmap |= 1 << DeviceNumber;
        }
        DeviceNumber = DeviceNumber.wrapping_add(1);
    }

    ATA_SELECT_DEVICE(ChanData, 0, (IDE_DRIVE_SELECT) as UCHAR);

    PataIssueSoftwareReset(ChanData);

    DeviceNumber = 0;
    while DeviceNumber < DeviceCount {
        let mut IdeStatus: u8;

        if (DeviceNumber % 2) != 0 {
            if !(DeviceBitmap & (1 << DeviceNumber)) != 0 {
                DeviceNumber = DeviceNumber.wrapping_add(1);
                continue;
            }

            i = ATA_TIME_RESET_SELECT;
            while i > 0 {
                ATA_SELECT_DEVICE(
                    ChanData,
                    (DeviceNumber) as UCHAR,
                    (IDE_DRIVE_SELECT | ((DeviceNumber & 1) << 4)) as UCHAR,
                );

                ATA_WRITE!((*ChanData).Regs.__bindgen_anon_3.ByteCountLow, 0xAA, ChanData, MRES_TF);
                ATA_WRITE!((*ChanData).Regs.__bindgen_anon_3.ByteCountLow, 0x55, ChanData, MRES_TF);
                ATA_WRITE!((*ChanData).Regs.__bindgen_anon_3.ByteCountLow, 0xAA, ChanData, MRES_TF);
                if ATA_READ!((*ChanData).Regs.__bindgen_anon_3.ByteCountLow, ChanData, MRES_TF) == 0xAA {
                    break;
                }

                AtaSleep();
                i = i.wrapping_sub(1);
            }
            if i == 0 {
                DeviceNumber = DeviceNumber.wrapping_add(1);
                continue;
            }
        }

        i = 0;
        while i < ATA_TIME_BUSY_RESET {
            IdeStatus = (*ChanData).ReadStatus.expect("non-null function pointer")(ChanData);
            if IdeStatus == 0xFF {
                break;
            }

            if !(IdeStatus & (IDE_STATUS_BUSY as u8)) != 0 {
                break;
            }

            AtaSleep();
            i = i.wrapping_add(1);
        }

        if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        } else {
            INFO!(
                b"CH %lu: Device %lu online with status %02x\n\0".as_ptr() as *const i8,
                (*ChanData).Channel,
                DeviceNumber,
                IdeStatus,
            );
        }

        DeviceNumber = DeviceNumber.wrapping_add(1);
    }

    AtaChanEnableInterruptsSync((ChanData).cast(), TRUE);
}
#[no_mangle]
pub unsafe extern "C" fn PataIdentifyDevice(
    ChannelContext: *mut core::ffi::c_void,
    DeviceNumber: u32,
) -> i32 {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let mut IdeStatus: u8;

    if PataIsDevicePresent(ChanData, DeviceNumber) == 0 {
        return (CONN_STATUS_NO_DEVICE) as i32;
    }

    IdeStatus = ATA_WAIT(
        ChanData,
        ATA_TIME_BUSY_SELECT,
        (IDE_STATUS_BUSY) as UCHAR,
        0,
    );
    if (IdeStatus as u32) & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) != 0 {
        return (CONN_STATUS_FAILURE) as i32;
    }

    return (CONN_STATUS_DEV_UNKNOWN) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn PataEnumerateChannel(ChannelContext: *mut core::ffi::c_void) -> u32 {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    return PataChannelGetMaximumDeviceCount(ChanData as *mut core::ffi::c_void);
}
