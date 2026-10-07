use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AtapModeToCycleTime: [ULONG; 18] = [
    600 as ::core::ffi::c_int as ULONG,
    383 as ::core::ffi::c_int as ULONG,
    240 as ::core::ffi::c_int as ULONG,
    180 as ::core::ffi::c_int as ULONG,
    120 as ::core::ffi::c_int as ULONG,
    960 as ::core::ffi::c_int as ULONG,
    480 as ::core::ffi::c_int as ULONG,
    240 as ::core::ffi::c_int as ULONG,
    480 as ::core::ffi::c_int as ULONG,
    150 as ::core::ffi::c_int as ULONG,
    120 as ::core::ffi::c_int as ULONG,
    120 as ::core::ffi::c_int as ULONG,
    80 as ::core::ffi::c_int as ULONG,
    60 as ::core::ffi::c_int as ULONG,
    45 as ::core::ffi::c_int as ULONG,
    30 as ::core::ffi::c_int as ULONG,
    20 as ::core::ffi::c_int as ULONG,
    15 as ::core::ffi::c_int as ULONG,
];

unsafe extern "C" fn AtaTimQueryPioModeSupport(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut AllowedModesMask: ULONG,
) {
    let mut IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut SupportedMode: ULONG = 0;
    let mut BestMode: ULONG = 0;
    let mut CurrentMode: ULONG = 0;
    let mut CycleTime: ULONG = 0;

    if (*IdentifyData).TranslationFieldsValid() & 0x2 != 0 {
        SupportedMode = ((*IdentifyData).AdvancedPIOModes() & 0x3) as ULONG;
        SupportedMode <<= PIO_MODE!(3);
        SupportedMode |= PIO_MODE0 | PIO_MODE1 | PIO_MODE2;
        SupportedMode &= AllowedModesMask;

        if (*IdentifyData).Capabilities.IordySupported() != 0 {
            CycleTime = (*IdentifyData).MinimumPIOCycleTimeIORDY as ULONG;
        } else {
            CycleTime = (*IdentifyData).MinimumPIOCycleTime as ULONG;
        }

        if CycleTime == 0 {
            NT_VERIFY!(_BitScanReverse(&raw mut BestMode, SupportedMode));
            CycleTime = AtapModeToCycleTime[BestMode as usize];
        }
    } else {
        SupportedMode = ((*IdentifyData).ObsoleteWords51[0 as usize]
            >> 8) as ULONG;
        if SupportedMode > PIO_MODE!(4) {
            SupportedMode = PIO_MODE!(0);
        }
        SupportedMode = ((1 << (SupportedMode + 1))
            - 1) as ULONG;
        SupportedMode &= AllowedModesMask;

        NT_VERIFY!(_BitScanReverse(&raw mut BestMode, SupportedMode));
        CycleTime = AtapModeToCycleTime[BestMode as usize];
    }

    (*DevExt).TransferModeSupportedBitmap |= SupportedMode;
    (*DevExt).MinimumPioCycleTime = CycleTime;

    NT_VERIFY!(_BitScanReverse(&raw mut BestMode, SupportedMode));
    CurrentMode = BestMode;
    (*DevExt).TransferModeCurrentBitmap |= 1 << CurrentMode;
}

unsafe extern "C" fn AtaTimQuerySwDmaModeSupport(
    DevExt: PATAPORT_DEVICE_EXTENSION,
    AllowedModesMask: ULONG,
) {
    let IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut SupportedMode: ULONG;
    let mut BestMode: ULONG = core::mem::zeroed();
    let mut CurrentMode: ULONG;
    let mut CycleTime: ULONG;

    CycleTime = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;

    CurrentMode = ((((*IdentifyData).ObsoleteWord62 & 0xFF00) >> 16) & 0x7) as _;
    if CurrentMode != 0 {
        NT_VERIFY!(_BitScanReverse(&raw mut CurrentMode, CurrentMode));
        CurrentMode += SWDMA_MODE!(0);

        (*DevExt).TransferModeCurrentBitmap |= 1 << CurrentMode;
    }

    SupportedMode = (((*IdentifyData).ObsoleteWord62 & 0x00FF) & 0x7) as _;
    SupportedMode <<= SWDMA_MODE!(0);
    SupportedMode &= AllowedModesMask;
    if SupportedMode != 0 {
        (*DevExt).TransferModeSupportedBitmap |= SupportedMode;

        NT_VERIFY!(_BitScanReverse(&raw mut BestMode, SupportedMode));

        CycleTime = AtapModeToCycleTime[BestMode as usize];
    }

    (*DevExt).MinimumSingleWordDmaCycleTime = CycleTime;
}

unsafe extern "C" fn AtaTimQueryMwDmaModeSupport(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut AllowedModesMask: ULONG,
) {
    let mut IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut SupportedMode: ULONG = 0;
    let mut BestMode: ULONG = 0;
    let mut CurrentMode: ULONG = 0;
    let mut CycleTime: ULONG = 0;

    CycleTime = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;

    CurrentMode = ((*IdentifyData).MultiWordDMAActive() & 0x7) as ULONG;
    if CurrentMode != 0 {
        NT_VERIFY!(_BitScanReverse(&raw mut CurrentMode, CurrentMode));
        CurrentMode = CurrentMode.wrapping_add(MWDMA_MODE!(0));
        (*DevExt).TransferModeCurrentBitmap |= (1 << CurrentMode) as ULONG;
    }

    SupportedMode = ((*IdentifyData).MultiWordDMASupport() & 0x7) as ULONG;
    SupportedMode <<= MWDMA_MODE!(0);
    SupportedMode &= AllowedModesMask;
    if SupportedMode != 0 {
        (*DevExt).TransferModeSupportedBitmap |= SupportedMode;

        NT_VERIFY!(_BitScanReverse(&raw mut BestMode, SupportedMode));

        if ((*IdentifyData).TranslationFieldsValid() & 0x2) != 0
            && (*IdentifyData).MinimumMWXferCycleTime != 0
            && (*IdentifyData).RecommendedMWXferCycleTime != 0
        {
            CycleTime = (*IdentifyData).MinimumMWXferCycleTime as ULONG;
        } else {
            CycleTime = AtapModeToCycleTime[BestMode as usize];
        }
    }

    (*DevExt).MinimumMultiWordDmaCycleTime = CycleTime;
}

unsafe extern "C" fn AtaTimQueryUDmaModeSupport(
    mut DevExt: PATAPORT_DEVICE_EXTENSION,
    mut AllowedModesMask: ULONG,
) {
    let mut IdentifyData: PIDENTIFY_DEVICE_DATA = &raw mut (*DevExt).__bindgen_anon_1.IdentifyDeviceData;
    let mut SupportedMode: ULONG = 0;
    let mut BestMode: ULONG = 0;
    let mut CurrentMode: ULONG = 0;
    let mut CycleTime: ULONG = 0;

    CycleTime = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;

    if !((*IdentifyData).TranslationFieldsValid() & 0x4 == 0) {
        CurrentMode = ((*IdentifyData).UltraDMAActive() & 0x7F) as ULONG;
        if CurrentMode != 0 {
            NT_VERIFY!(_BitScanReverse(
                &raw mut CurrentMode,
                CurrentMode as ::core::ffi::c_ulong,
            ));
            CurrentMode = CurrentMode.wrapping_add(UDMA_MODE!(0));
            (*DevExt).TransferModeCurrentBitmap |= (1 << CurrentMode) as ULONG;
        }

        SupportedMode = ((*IdentifyData).UltraDMASupport() & 0x7F) as ULONG;
        SupportedMode <<= UDMA_MODE!(0);
        SupportedMode &= AllowedModesMask;
        if SupportedMode != 0 {
            (*DevExt).TransferModeSupportedBitmap |= SupportedMode;

            NT_VERIFY!(_BitScanReverse(
                &raw mut BestMode,
                SupportedMode as ::core::ffi::c_ulong,
            ));

            CycleTime = AtapModeToCycleTime[BestMode as usize];
        }
    }

    (*DevExt).MinimumUltraDmaCycleTime = CycleTime;
}

unsafe extern "C" fn AtaTimDumpTimingInfo(
    PortData: PATAPORT_PORT_DATA,
    DevExt: PATAPORT_DEVICE_EXTENSION,
) {
    INFO!(
        b"CH %lu: Device %u '%s'\nCycle time       UDMA[%ld] MWDMA[%ld] SWDMA[%ld] PIO[%ld]\nSupported modes  0x%08lX\nActive modes     0x%08lX\nSelected modes   0x%08lX\nIOREADY          %u\n\0"
            .as_ptr() as *const i8,
        (*PortData).PortNumber,
        (*DevExt).Device.AtaScsiAddress.TargetId,
        (*DevExt).FriendlyName,
        (*DevExt).MinimumUltraDmaCycleTime as i32,
        (*DevExt).MinimumMultiWordDmaCycleTime as i32,
        (*DevExt).MinimumSingleWordDmaCycleTime as i32,
        (*DevExt).MinimumPioCycleTime as i32,
        (*DevExt).TransferModeSupportedBitmap,
        (*DevExt).TransferModeCurrentBitmap,
        (*DevExt).TransferModeSelectedBitmap,
        (*DevExt).IdentifyDeviceData.Capabilities.IordySupported,
    );
}
#[no_mangle]
pub unsafe extern "C" fn AtaPortSelectTimings(
    PortData: PATAPORT_PORT_DATA,
    ForceCompatibleTimings: BOOLEAN,
) {
    let mut ChanExt: PATAPORT_CHANNEL_EXTENSION = core::ptr::null_mut();
    let mut i: ULONG;
    let mut Config: [CHANNEL_DEVICE_CONFIG; (ATA_MAX_DEVICE) as _] = [CHANNEL_DEVICE_CONFIG {
        SupportedModes: 0,
        PioMode: 0,
        DmaMode: 0,
        MinPioCycleTime: 0,
        MinMwDmaCycleTime: 0,
        MinSwDmaCycleTime: 0,
        CurrentModes: 0,
        IsFixedDisk: 0,
        IoReadySupported: 0,
        IsNewDevice: 0,
        FriendlyName: core::ptr::null(),
        IdentifyDeviceData: core::ptr::null_mut(),
    }; (ATA_MAX_DEVICE) as _];
    let mut DeviceList: [PCHANNEL_DEVICE_CONFIG; (ATA_MAX_DEVICE) as _] = [core::ptr::null_mut(); (ATA_MAX_DEVICE) as _];

    if ForceCompatibleTimings != 0 {
        i = 0;
        while i < ATA_MAX_DEVICE {
            DeviceList[i as usize] = &mut Config[i as usize] as *mut CHANNEL_DEVICE_CONFIG;

            Config[i as usize].SupportedModes = PIO_MODE0;
            Config[i as usize].CurrentModes = PIO_MODE0;
            Config[i as usize].MinPioCycleTime = AtapModeToCycleTime[PIO_MODE!(0) as usize];
            Config[i as usize].MinSwDmaCycleTime = AtapModeToCycleTime[SWDMA_MODE!(0) as usize];
            Config[i as usize].MinMwDmaCycleTime = AtapModeToCycleTime[MWDMA_MODE!(0) as usize];
            Config[i as usize].IsFixedDisk = TRUE;
            Config[i as usize].IoReadySupported = FALSE;
            Config[i as usize].IsNewDevice = FALSE;
            Config[i as usize].FriendlyName = b"\0".as_ptr() as *const i8;

            i = i.wrapping_add(1);
        }

        (((*PortData).SetTransferMode).unwrap_unchecked())((*PortData).ChannelContext, DeviceList.as_mut_ptr());
        return;
    }

    ChanExt = CONTAINING_RECORD!(PortData, ATAPORT_CHANNEL_EXTENSION, PortData);

    i = 0;
    while i < ATA_MAX_DEVICE {
        let mut DevExt: PATAPORT_DEVICE_EXTENSION;
        let mut AllowedModesMask: ULONG;

        DevExt = (AtaFdoFindDeviceByPath(
            (ChanExt).cast(),
            AtaMarshallScsiAddress((*PortData).PortNumber, i, 0),
            core::ptr::null_mut(),
        )).cast();
        if DevExt.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        (*DevExt).TransferModeCurrentBitmap = 0;
        (*DevExt).TransferModeSupportedBitmap = 0;

        AllowedModesMask = (*DevExt).TransferModeUserAllowedMask;
        AllowedModesMask &= (*DevExt).TransferModeAllowedMask;
        AllowedModesMask |= PIO_MODE0;

        AtaTimQueryPioModeSupport(DevExt, AllowedModesMask);
        AtaTimQuerySwDmaModeSupport(DevExt, AllowedModesMask);
        AtaTimQueryMwDmaModeSupport(DevExt, AllowedModesMask);
        AtaTimQueryUDmaModeSupport(DevExt, AllowedModesMask);

        DeviceList[i as usize] = &mut Config[i as usize] as *mut CHANNEL_DEVICE_CONFIG;
        Config[i as usize].CurrentModes = (*DevExt).TransferModeCurrentBitmap;
        Config[i as usize].SupportedModes = (*DevExt).TransferModeSupportedBitmap;
        Config[i as usize].MinPioCycleTime = (*DevExt).MinimumPioCycleTime;
        Config[i as usize].MinSwDmaCycleTime = (*DevExt).MinimumSingleWordDmaCycleTime;
        Config[i as usize].MinMwDmaCycleTime = (*DevExt).MinimumMultiWordDmaCycleTime;
        Config[i as usize].IsFixedDisk = !IS_ATAPI!(&(*DevExt).Device) as BOOLEAN;
        Config[i as usize].IoReadySupported = !!(*DevExt).__bindgen_anon_1.IdentifyDeviceData.Capabilities.IordySupported() as BOOLEAN;
        Config[i as usize].IsNewDevice = !((*DevExt).Device.DeviceFlags & DEVICE_PNP_STARTED) as BOOLEAN;
        Config[i as usize].FriendlyName = ((*DevExt).FriendlyName).as_ptr();

        i = i.wrapping_add(1);
    }

    (((*PortData).SetTransferMode).unwrap_unchecked())((*PortData).ChannelContext, DeviceList.as_mut_ptr());

    i = 0;
    while i < ATA_MAX_DEVICE {
        let DeviceConfig: PCHANNEL_DEVICE_CONFIG = DeviceList[i as usize];
        let mut DevExt: PATAPORT_DEVICE_EXTENSION;

        if DeviceConfig.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        DevExt = (AtaFdoFindDeviceByPath(
            (ChanExt).cast(),
            AtaMarshallScsiAddress((*PortData).PortNumber, i, 0),
            core::ptr::null_mut(),
        )).cast();

        (*DevExt).TransferModeSelectedBitmap = 1 << (*DeviceConfig).PioMode;
        if (*DeviceConfig).DmaMode != PIO_MODE!(0) {
            (*DevExt).TransferModeSelectedBitmap |= 1 << (*DeviceConfig).DmaMode;
        }

        AtaTimDumpTimingInfo(PortData, DevExt);

        i = i.wrapping_add(1);
    }
}
