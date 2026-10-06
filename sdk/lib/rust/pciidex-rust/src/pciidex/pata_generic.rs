use crate::prelude::*;
use ntkrnl_sys::ddk::{Channel, ExAllocatePoolZero, IoConnectInterrupt, IoCreateController, IoDisconnectInterrupt, IoGetConfigurationInformation, KeInitializeDpc, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PCONFIGURATION_INFORMATION, PDMA_ADAPTER, PDMA_OPERATIONS, PHYSICAL_ADDRESS, PKSERVICE_ROUTINE, Spare, Timing};
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

static mut AtapTimingTable: [ATA_TIMING; 11] = [
    _ATA_TIMING {
        AddressSetup: 70 as ATATIM,
        CmdActive: 290 as ATATIM,
        CmdRecovery: 240 as ATATIM,
        DataActive: 165 as ATATIM,
        DataRecovery: 150 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 50 as ATATIM,
        CmdActive: 290 as ATATIM,
        CmdRecovery: 93 as ATATIM,
        DataActive: 125 as ATATIM,
        DataRecovery: 100 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 30 as ATATIM,
        CmdActive: 290 as ATATIM,
        CmdRecovery: 40 as ATATIM,
        DataActive: 100 as ATATIM,
        DataRecovery: 90 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 30 as ATATIM,
        CmdActive: 80 as ATATIM,
        CmdRecovery: 70 as ATATIM,
        DataActive: 80 as ATATIM,
        DataRecovery: 70 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 25 as ATATIM,
        CmdActive: 70 as ATATIM,
        CmdRecovery: 25 as ATATIM,
        DataActive: 70 as ATATIM,
        DataRecovery: 25 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 120 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 480 as ATATIM,
        DataRecovery: 480 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 90 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 240 as ATATIM,
        DataRecovery: 240 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 60 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 120 as ATATIM,
        DataRecovery: 120 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 60 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 215 as ATATIM,
        DataRecovery: 215 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 45 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 80 as ATATIM,
        DataRecovery: 50 as ATATIM,
    },
    _ATA_TIMING {
        AddressSetup: 25 as ATATIM,
        CmdActive: 0 as ATATIM,
        CmdRecovery: 0 as ATATIM,
        DataActive: 70 as ATATIM,
        DataRecovery: 25 as ATATIM,
    },
];

unsafe extern "C" fn AtaGetClocks(
    mut TimingValueNs: ATATIM,
    mut ClockPeriodPs: ULONG,
) -> ATATIM {
    return ((TimingValueNs as ::core::ffi::c_int * 1000 as ::core::ffi::c_int)
        as ULONG)
        .wrapping_add(ClockPeriodPs)
        .wrapping_sub(1 as ULONG)
        .wrapping_div(ClockPeriodPs) as ATATIM;
}

unsafe extern "C" fn AtaCalculateTimings(
    mut Device: PCHANNEL_DEVICE_CONFIG,
    mut Timing: PATA_TIMING,
    mut Mode: ULONG,
    mut ClockPeriodPs: ULONG,
) {
    let mut DataCycleTime: ATATIM = 0;
    let mut CmdCycleTime: ATATIM = 0;

    (*Timing).AddressSetup = AtaGetClocks(AtapTimingTable[Mode as usize].AddressSetup, ClockPeriodPs);
    (*Timing).CmdActive = AtaGetClocks(AtapTimingTable[Mode as usize].CmdActive, ClockPeriodPs);
    (*Timing).CmdRecovery = AtaGetClocks(AtapTimingTable[Mode as usize].CmdRecovery, ClockPeriodPs);
    (*Timing).DataActive = AtaGetClocks(AtapTimingTable[Mode as usize].DataActive, ClockPeriodPs);
    (*Timing).DataRecovery = AtaGetClocks(AtapTimingTable[Mode as usize].DataRecovery, ClockPeriodPs);

    if Mode >= MWDMA_MODE!(0) {
        DataCycleTime = (*Device).MinMwDmaCycleTime as ATATIM;
    } else if Mode >= SWDMA_MODE!(0) {
        DataCycleTime = (*Device).MinSwDmaCycleTime as ATATIM;
    } else {
        DataCycleTime = (*Device).MinPioCycleTime as ATATIM;
        if Mode == PIO_MODE!(2) {
            CmdCycleTime = max(DataCycleTime, 330);
        } else {
            CmdCycleTime = max(DataCycleTime, (AtapModeToCycleTime[Mode as usize]) as ATATIM);
        }
    }
    DataCycleTime = max(DataCycleTime, (AtapModeToCycleTime[Mode as usize]) as ATATIM);

    DataCycleTime = AtaGetClocks(DataCycleTime, ClockPeriodPs);
    if ((*Timing).DataActive + (*Timing).DataRecovery) < DataCycleTime {
        (*Timing).DataActive += (DataCycleTime - ((*Timing).DataActive + (*Timing).DataRecovery)) / 2;
        (*Timing).DataRecovery = DataCycleTime - (*Timing).DataActive;
    }
    if Mode < SWDMA_MODE!(0) {
        CmdCycleTime = AtaGetClocks(CmdCycleTime, ClockPeriodPs);
        if ((*Timing).CmdActive + (*Timing).CmdRecovery) < CmdCycleTime {
            (*Timing).CmdActive += (CmdCycleTime - ((*Timing).CmdActive + (*Timing).CmdRecovery)) / 2;
            (*Timing).CmdRecovery = CmdCycleTime - (*Timing).CmdActive;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaSelectTimings(
    DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
    Timings: PATA_TIMING,
    ClockPeriodPs: ULONG,
    Flags: ULONG,
) {
    let mut i: ULONG = 0;
    while i < MAX_IDE_DEVICE {
        let Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let Timing: PATA_TIMING = Timings.offset(i as isize);
        let mut DmaTiming: ATA_TIMING = ATA_TIMING {
            AddressSetup: 0,
            CmdActive: 0,
            CmdRecovery: 0,
            DataActive: 0,
            DataRecovery: 0,
        };

        if Device.is_null() {
            RtlZeroMemory!(
                Timing as *mut core::ffi::c_void,
                core::mem::size_of::<ATA_TIMING>(),
            );
        } else {
            AtaCalculateTimings(Device, Timing, (*Device).PioMode, ClockPeriodPs);

            if !((*Device).DmaMode == PIO_MODE!(0)
                || (*Device).DmaMode >= UDMA_MODE!(0))
            {
                AtaCalculateTimings(Device, &mut DmaTiming, (*Device).DmaMode, ClockPeriodPs);

                (*Timing).AddressSetup = max((*Timing).AddressSetup, DmaTiming.AddressSetup);
                (*Timing).CmdActive = max((*Timing).CmdActive, DmaTiming.CmdActive);
                (*Timing).CmdRecovery = max((*Timing).CmdRecovery, DmaTiming.CmdRecovery);
                (*Timing).DataActive = max((*Timing).DataActive, DmaTiming.DataActive);
                (*Timing).DataRecovery = max((*Timing).DataRecovery, DmaTiming.DataRecovery);
            }
        }

        i = i.wrapping_add(1);
    }

    if Flags & SHARED_ADDR_TIMINGS != 0 {
        (*Timings.offset(0 as isize)).AddressSetup = max(
            (*Timings.offset(0 as isize)).AddressSetup,
            (*Timings.offset(1 as isize)).AddressSetup,
        );
        (*Timings.offset(1 as isize)).AddressSetup =
            (*Timings.offset(0 as isize)).AddressSetup;
    }

    if Flags & SHARED_CMD_TIMINGS != 0 {
        (*Timings.offset(0 as isize)).CmdActive = max(
            (*Timings.offset(0 as isize)).CmdActive,
            (*Timings.offset(1 as isize)).CmdActive,
        );
        (*Timings.offset(0 as isize)).CmdRecovery = max(
            (*Timings.offset(0 as isize)).CmdRecovery,
            (*Timings.offset(1 as isize)).CmdRecovery,
        );
        (*Timings.offset(1 as isize)).CmdActive =
            (*Timings.offset(0 as isize)).CmdActive;
        (*Timings.offset(1 as isize)).CmdRecovery =
            (*Timings.offset(0 as isize)).CmdRecovery;
    }

    if Flags & SHARED_DATA_TIMINGS != 0 {
        (*Timings.offset(0 as isize)).DataActive = max(
            (*Timings.offset(0 as isize)).DataActive,
            (*Timings.offset(1 as isize)).DataActive,
        );
        (*Timings.offset(0 as isize)).DataRecovery = max(
            (*Timings.offset(0 as isize)).DataRecovery,
            (*Timings.offset(1 as isize)).DataRecovery,
        );
        (*Timings.offset(1 as isize)).DataActive =
            (*Timings.offset(0 as isize)).DataActive;
        (*Timings.offset(1 as isize)).DataRecovery =
            (*Timings.offset(0 as isize)).DataRecovery;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SataSetTransferMode(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
    DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    UNREFERENCED_PARAMETER!(Controller);
    UNREFERENCED_PARAMETER!(Channel);
    UNREFERENCED_PARAMETER!(DeviceList);

    NOTHING;
}

unsafe extern "C" fn AtaAcpiFindModeForCycleTime(
    mut GtmCycleTime: u32,
    mut SupportedModesBitmap: u32,
    mut CurrentModesBitmap: u32,
    mut MinimumMode: u32,
    mut MaximumMode: u32,
    mut BestCycleTime: *mut u32,
    mut BestMode: *mut u32,
) {
    let mut i: i32;

    if GtmCycleTime == IDE_ACPI_TIMING_MODE_NOT_SUPPORTED {
        *BestCycleTime = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;
        *BestMode = PIO_MODE!(0);
        return;
    }

    i = MaximumMode as i32;
    while i >= MinimumMode as i32 {
        if (SupportedModesBitmap & (1 << i)) != 0
            && AtapModeToCycleTime[i as usize] >= GtmCycleTime
        {
            *BestCycleTime = AtapModeToCycleTime[i as usize];
            *BestMode = i as u32;
            return;
        }
        i -= 1;
    }

    INFO!(
        b"Failed to find mode for %ld ns cycle time from %lu in 0x%08lX\n\0".as_ptr() as *const i8,
        GtmCycleTime as i32,
        MaximumMode,
        SupportedModesBitmap,
    );

    if _BitScanReverse(BestMode, CurrentModesBitmap) != 0 {
        *BestCycleTime = AtapModeToCycleTime[*BestMode as usize];
    } else {
        *BestCycleTime = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;
        *BestMode = PIO_MODE!(0);
    }
}

unsafe extern "C" fn PciIdeAcpiSetTransferMode(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    let mut CurrentMode: PIDE_ACPI_TIMING_MODE_BLOCK = &raw mut (*ChanData).CurrentTimingMode;
    let mut NewMode: IDE_ACPI_TIMING_MODE_BLOCK = IDE_ACPI_TIMING_MODE_BLOCK {
        Drive: [
            IDE_ACPI_TIMING_MODE_DRIVE {
                PioSpeed: 0,
                DmaSpeed: 0,
            },
            IDE_ACPI_TIMING_MODE_DRIVE {
                PioSpeed: 0,
                DmaSpeed: 0,
            },
        ],
        ModeFlags: 0,
    };
    let mut Status: NTSTATUS = 0;
    let mut i: ULONG = 0;

    NewMode.ModeFlags = (*CurrentMode).ModeFlags;

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut HasUltraDMA: BOOLEAN = 0;

        if Device.is_null() {
            NewMode.Drive[i as usize].PioSpeed = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;
            NewMode.Drive[i as usize].DmaSpeed = IDE_ACPI_TIMING_MODE_NOT_SUPPORTED;
        } else {
            AtaAcpiFindModeForCycleTime(
                (*CurrentMode).Drive[i as usize].PioSpeed,
                (*Device).SupportedModes,
                (*Device).CurrentModes & PIO_ALL,
                PIO_MODE!(0),
                PIO_MODE!(4),
                &raw mut NewMode.Drive[i as usize].PioSpeed,
                &raw mut (*Device).PioMode,
            );

            HasUltraDMA = ((*CurrentMode).ModeFlags
                & IDE_ACPI_TIMING_MODE_FLAG_UDMA(i)
                != 0) as BOOLEAN;

            AtaAcpiFindModeForCycleTime(
                (*CurrentMode).Drive[i as usize].DmaSpeed,
                (*Device).SupportedModes,
                (*Device).CurrentModes & !(PIO_ALL | SWDMA_ALL),
                MWDMA_MODE!(0),
                if HasUltraDMA != 0 {
                    UDMA_MODE!(6)
                } else {
                    MWDMA_MODE!(2)
                },
                &raw mut NewMode.Drive[i as usize].DmaSpeed,
                &raw mut (*Device).DmaMode,
            );

            if (*Device).DmaMode >= UDMA_MODE!(0) {
                NewMode.ModeFlags |= IDE_ACPI_TIMING_MODE_FLAG_UDMA(i);
            } else {
                NewMode.ModeFlags &= !IDE_ACPI_TIMING_MODE_FLAG_UDMA(i);
            }
        }

        i = i.wrapping_add(1);
    }

    if NewMode.ModeFlags & IDE_ACPI_TIMING_MODE_FLAG_INDEPENDENT_TIMINGS == 0
        && !(*DeviceList.offset(0 as isize)).is_null()
        && !(*DeviceList.offset(1 as isize)).is_null()
    {
        if NewMode.Drive[0 as usize].DmaSpeed != IDE_ACPI_TIMING_MODE_NOT_SUPPORTED
            && NewMode.Drive[1 as usize].DmaSpeed != IDE_ACPI_TIMING_MODE_NOT_SUPPORTED
        {
            NewMode.Drive[0 as usize].DmaSpeed = max(
                NewMode.Drive[0 as usize].DmaSpeed,
                NewMode.Drive[1 as usize].DmaSpeed,
            );
            (**DeviceList.offset(0 as isize)).DmaMode = min(
                (**DeviceList.offset(0 as isize)).DmaMode,
                (**DeviceList.offset(1 as isize)).DmaMode,
            );

            NewMode.Drive[1 as usize].DmaSpeed =
                NewMode.Drive[0 as usize].DmaSpeed;
            (**DeviceList.offset(1 as isize)).DmaMode =
                (**DeviceList.offset(0 as isize)).DmaMode;
        }

        NewMode.Drive[0 as usize].PioSpeed = max(
            NewMode.Drive[0 as usize].PioSpeed,
            NewMode.Drive[1 as usize].PioSpeed,
        );
        (**DeviceList.offset(0 as isize)).PioMode = min(
            (**DeviceList.offset(0 as isize)).PioMode,
            (**DeviceList.offset(1 as isize)).PioMode,
        );

        NewMode.Drive[1 as usize].PioSpeed =
            NewMode.Drive[0 as usize].PioSpeed;
        (**DeviceList.offset(1 as isize)).PioMode =
            (**DeviceList.offset(0 as isize)).PioMode;
    }

    INFO!(
        b"CH %lu: Old timings %-2lX 0:%-3ld 0:%-3ld 1:%-3ld 1:%-3ld\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        (*CurrentMode).ModeFlags,
        (*CurrentMode).Drive[0 as usize].PioSpeed as LONG,
        (*CurrentMode).Drive[0 as usize].DmaSpeed as LONG,
        (*CurrentMode).Drive[1 as usize].PioSpeed as LONG,
        (*CurrentMode).Drive[1 as usize].DmaSpeed as LONG,
    );

    INFO!(
        b"CH %lu: New timings %-2lX 0:%-3ld 0:%-3ld 1:%-3ld 1:%-3ld\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        NewMode.ModeFlags,
        NewMode.Drive[0 as usize].PioSpeed as LONG,
        NewMode.Drive[0 as usize].DmaSpeed as LONG,
        NewMode.Drive[1 as usize].PioSpeed as LONG,
        NewMode.Drive[1 as usize].DmaSpeed as LONG,
    );

    Status = AtaAcpiSetTimingMode(
        (*(*ChanData).PdoExt).Common.Self,
        &raw mut NewMode,
        if !(*DeviceList.offset(0 as isize)).is_null() {
            (**DeviceList.offset(0 as isize)).IdentifyDeviceData
        } else {
            core::ptr::null_mut()
        },
        if !(*DeviceList.offset(1 as isize)).is_null() {
            (**DeviceList.offset(1 as isize)).IdentifyDeviceData
        } else {
            core::ptr::null_mut()
        },
    );

    if !NT_SUCCESS!(Status) {
        i = 0;
        while i < MAX_IDE_DEVICE {
            let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);

            if Device.is_null() {
                i = i.wrapping_add(1);
                continue;
            }

            (*Device).DmaMode = PIO_MODE!(0);
            NT_VERIFY!(_BitScanReverse(
                &raw mut (*Device).PioMode,
                (*Device).CurrentModes & PIO_ALL,
            ));

            i = i.wrapping_add(1);
        }
        return;
    }

    AtaAcpiGetTimingMode((*(*ChanData).PdoExt).Common.Self, CurrentMode);
}

unsafe extern "C" fn PciIdeBiosSetTransferMode(
    ChanData: PCHANNEL_DATA_PATA,
    DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    let mut i: ULONG = 0;
    while i < MAX_IDE_DEVICE {
        let Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut DmaFlags: ULONG;

        if Device.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        if _BitScanReverse(
            &raw mut (*Device).DmaMode,
            (*Device).CurrentModes & !PIO_ALL,
        ) == 0
        {
            (*Device).DmaMode = PIO_MODE!(0);
        }
        NT_VERIFY!(_BitScanReverse(
            &raw mut (*Device).PioMode,
            (*Device).CurrentModes & PIO_ALL,
        ));

        if i == 0 {
            DmaFlags = CHANNEL_FLAG_DRIVE0_DMA_CAPABLE;
        } else {
            DmaFlags = CHANNEL_FLAG_DRIVE1_DMA_CAPABLE;
        }

        if (*ChanData).__bindgen_anon_1.ChanInfo & DmaFlags == 0 {
            (*Device).DmaMode = PIO_MODE!(0);
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn PciIdeGenericSetTransferMode(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
    DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    let ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_ACPI_GTM != 0 {
        PciIdeAcpiSetTransferMode(ChanData, DeviceList);
    } else {
        PciIdeBiosSetTransferMode(ChanData, DeviceList);
    }
}

unsafe extern "C" fn PataReleaseLegacyAddressRanges(mut ChanData: PCHANNEL_DATA_PATA) {
    let mut ConfigInfo: PCONFIGURATION_INFORMATION = IoGetConfigurationInformation();

    PAGED_CODE!();

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_PRIMARY_ADDRESS_CLAIMED != 0 {
        (*ConfigInfo).AtDiskPrimaryAddressClaimed = FALSE;
    }
    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_SECONDARY_ADDRESS_CLAIMED != 0 {
        (*ConfigInfo).AtDiskSecondaryAddressClaimed = FALSE;
    }
    (*ChanData).__bindgen_anon_1.ChanInfo &= !(CHANNEL_FLAG_PRIMARY_ADDRESS_CLAIMED | CHANNEL_FLAG_SECONDARY_ADDRESS_CLAIMED);
}

unsafe extern "C" fn PataClaimLegacyAddressRanges(mut ChanData: PCHANNEL_DATA_PATA) {
    let mut ConfigInfo: PCONFIGURATION_INFORMATION = IoGetConfigurationInformation();

    PAGED_CODE!();

    if IsNEC_98 != 0 {
        if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_CBUS != 0 {
            (*ConfigInfo).AtDiskPrimaryAddressClaimed = TRUE;
            (*ConfigInfo).AtDiskSecondaryAddressClaimed = TRUE;

            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_PRIMARY_ADDRESS_CLAIMED |
                                    CHANNEL_FLAG_SECONDARY_ADDRESS_CLAIMED;
        }
    } else if (*ChanData).Regs.Data == UlongToPtr!(PCIIDE_LEGACY_PRIMARY_COMMAND_BASE) as *mut u8 {
        (*ConfigInfo).AtDiskPrimaryAddressClaimed = TRUE;
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_PRIMARY_ADDRESS_CLAIMED;
    } else if (*ChanData).Regs.Data == UlongToPtr!(PCIIDE_LEGACY_SECONDARY_COMMAND_BASE) as *mut u8 {
        (*ConfigInfo).AtDiskSecondaryAddressClaimed = TRUE;
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_SECONDARY_ADDRESS_CLAIMED;
    }
}

unsafe extern "C" fn PciIdeFreeMemory(ChannelContext: PVOID) {
    let ChanData: PCHANNEL_DATA_PATA = ChannelContext as PCHANNEL_DATA_PATA;
    let DmaAdapter: PDMA_ADAPTER = (*ChanData).__bindgen_anon_1.DmaAdapter;
    let DmaOperations: PDMA_OPERATIONS = (*(*ChanData).__bindgen_anon_1.DmaAdapter).DmaOperations;
    let mut PrdTablePhysicalAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };

    PAGED_CODE!();

    if (*ChanData).PrdTable.is_null() {
        return;
    }

    PrdTablePhysicalAddress.QuadPart = ((*ChanData).PrdTablePhysicalAddress) as _;

    ((*DmaOperations).FreeCommonBuffer)(
        DmaAdapter,
        (core::mem::size_of::<*mut _PCIIDE_PRD_TABLE_ENTRY>() * (*ChanData).__bindgen_anon_1.MaximumPhysicalPages as usize) as ULONG,
        PrdTablePhysicalAddress,
        (*ChanData).PrdTable as PVOID,
        TRUE,
    );
    (*ChanData).PrdTable = core::ptr::null_mut::<_PCIIDE_PRD_TABLE_ENTRY>();
}

unsafe extern "C" fn PciIdeAllocateMemory(ChannelContext: PVOID) -> NTSTATUS {
    let ChanData: PCHANNEL_DATA_PATA = ChannelContext as PCHANNEL_DATA_PATA;
    let DmaOperations: PDMA_OPERATIONS = (*(*ChanData).__bindgen_anon_1.DmaAdapter).DmaOperations;
    let mut LogicalAddress: PHYSICAL_ADDRESS = { let mut __lit: PHYSICAL_ADDRESS = core::mem::zeroed(); __lit.__bindgen_anon_1.LowPart = 0; __lit.__bindgen_anon_1.HighPart = 0; __lit };
    let mut BlockSize: ULONG;

    PAGED_CODE!();

    BlockSize = (core::mem::size_of::<PPCIIDE_PRD_TABLE_ENTRY>() as ULONG)
        .wrapping_mul((*ChanData).__bindgen_anon_1.MaximumPhysicalPages as ULONG);
    (*ChanData).PrdTable = (*DmaOperations)
        .AllocateCommonBuffer
        .expect("non-null function pointer")(
        (*ChanData).__bindgen_anon_1.DmaAdapter,
        BlockSize,
        &raw mut LogicalAddress,
        TRUE,
    ) as PPCIIDE_PRD_TABLE_ENTRY;
    if (*ChanData).PrdTable.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlZeroMemory!((*ChanData).PrdTable as *mut core::ffi::c_void, BlockSize as usize);

    (*ChanData).PrdTablePhysicalAddress = LogicalAddress.__bindgen_anon_1.LowPart;

    if !(LogicalAddress.__bindgen_anon_1.HighPart == 0 as LONG) {
        RtlAssert(
            b"LogicalAddress.__bindgen_anon_1.HighPart == 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/storage/ide/pciidex/chipset/pata_generic.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            533 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if !((LogicalAddress.__bindgen_anon_1.LowPart as usize).wrapping_rem(core::mem::size_of::<ULONG>()) == 0 as usize) {
        RtlAssert(
            b"(LogicalAddress.__bindgen_anon_1.LowPart % sizeof(ULONG)) == 0\0" as *const u8
                as *const core::ffi::c_char as PVOID,
            b"drivers/storage/ide/pciidex/chipset/pata_generic.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            536 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeIsDmaStatusValid(mut DmaStatus: u8) -> i32 {
    if (KeGetCurrentIrql() as i32) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/chipset/pata_generic.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                547 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }
    if (DmaStatus as u32) & (PCIIDE_DMA_STATUS_RESERVED1 | PCIIDE_DMA_STATUS_RESERVED2) != 0 {
        return (FALSE) as _;
    }
    return (TRUE) as _;
}

unsafe extern "C" fn PciIdeControllerInitDma(mut Controller: PATA_CONTROLLER) -> BOOLEAN {
    let mut i: ULONG = 0;

    PAGED_CODE!();

    if (*Controller).Pci.Command as ::core::ffi::c_int & PCI_ENABLE_BUS_MASTER == 0 {
        INFO!(b"PCI bus mastering disabled\n\0" as *const u8 as *const ::core::ffi::c_char);
        return FALSE;
    }

    if ((*Controller).Pci.BaseClass as ::core::ffi::c_int) as u32 == PCI_CLASS_MASS_STORAGE_CTLR
        && (*Controller).Pci.ProgIf as ::core::ffi::c_int & PCIIDE_PROGIF_DMA_CAPABLE == 0
    {
        INFO!(b"Non DMA capable controller detected\n\0" as *const u8 as *const ::core::ffi::c_char);
        return FALSE;
    }

    if (*Controller).Flags & CTRL_FLAG_MANUAL_RES as ULONG == 0 {
        let mut IoBase: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();

        IoBase = AtaCtrlPciMapBar(Controller, PCIIDE_DMA_IO_BAR, PCIIDE_DMA_IO_RANGE_LENGTH) as PUCHAR;
        if IoBase.is_null() {
            if (*Controller).AccessRange[PCIIDE_DMA_IO_BAR as usize].Flags & RANGE_IS_VALID as ULONG != 0 {
                ERR!(b"Failed to map bus master registers\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                INFO!(b"No bus master registers\0" as *const u8 as *const ::core::ffi::c_char);
            }
            return FALSE;
        }

        i = 0;
        while i < (*Controller).MaxChannels {
            let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;

            if (*Controller).AccessRange[PCIIDE_DMA_IO_BAR as usize].Flags & RANGE_IS_MAPPED as ULONG != 0 {
                (*ChanData).ChanInfo |= CHANNEL_FLAG_MRES_DMA;
            }

            if !IS_PRIMARY_CHANNEL!(ChanData) {
                IoBase = IoBase.offset(PCIIDE_DMA_SECONDARY_CHANNEL_OFFSET as isize);
            }

            (*ChanData).Regs.Dma = IoBase;

            if (*Controller).Flags & CTRL_FLAG_DMA_INTERRUPT as ULONG != 0 {
                (*ChanData).CheckInterrupt = Some(
                    PciIdeCheckInterrupt as unsafe extern "C" fn(PCHANNEL_DATA_PATA) -> BOOLEAN,
                ) as PCHANNEL_CHECK_INTERRUPT;
            }
            i = i.wrapping_add(1);
        }
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;
        let mut DmaStatus: UCHAR = 0;

        if !((*ChanData).TransferModeSupported & !PIO_ALL as ULONG == 0) {
            DmaStatus = ATA_READ!(
                (*ChanData).Regs.Dma.offset(PCIIDE_DMA_STATUS as isize),
                ChanData,
                MRES_DMA,
            );
            if PciIdeIsDmaStatusValid(DmaStatus) == 0 {
                (*ChanData).TransferModeSupported &= PIO_ALL;
            } else {
                INFO!(
                    b"CH %lu: %p DMA 0x%02X\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ChanData).Channel,
                    (*ChanData).Regs.Dma,
                    DmaStatus as ::core::ffi::c_int,
                );

                if i == 0 && (*Controller).MaxChannels > 1 {
                    if DmaStatus as ::core::ffi::c_int & PCIIDE_DMA_STATUS_SIMPLEX != 0 {
                        (*Controller).Flags |= CTRL_FLAG_IS_SIMPLEX;
                    }
                }

                if DmaStatus as ::core::ffi::c_int & PCIIDE_DMA_STATUS_DRIVE0_DMA_CAPABLE != 0 {
                    (*ChanData).ChanInfo |= CHANNEL_FLAG_DRIVE0_DMA_CAPABLE;
                }
                if DmaStatus as ::core::ffi::c_int & PCIIDE_DMA_STATUS_DRIVE1_DMA_CAPABLE != 0 {
                    (*ChanData).ChanInfo |= CHANNEL_FLAG_DRIVE1_DMA_CAPABLE;
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeInitTaskFileIoResources(
    ChanData: *mut CHANNEL_DATA_PATA,
    CommandPortBase: usize,
    ControlPortBase: usize,
    CommandBlockSpare: u32,
) {
    let Registers: *mut IDE_REGISTERS = core::ptr::addr_of_mut!((*ChanData).Regs);

    PAGED_CODE!();

    (*Registers).Data = ((CommandPortBase + 0 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).__bindgen_anon_1.Error = ((CommandPortBase + 1 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).__bindgen_anon_2.SectorCount = ((CommandPortBase + 2 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).LbaLow = ((CommandPortBase + 3 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).__bindgen_anon_3.LbaMid = ((CommandPortBase + 4 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).__bindgen_anon_4.LbaHigh = ((CommandPortBase + 5 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).Device = ((CommandPortBase + 6 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();
    (*Registers).__bindgen_anon_1.Error = ((CommandPortBase + 7 * CommandBlockSpare as usize) as *mut core::ffi::c_void).cast();

    (*Registers).__bindgen_anon_6.Control = (ControlPortBase as *mut core::ffi::c_void).cast();
}

unsafe extern "C" fn PciIdeAssignNativeResources(
    mut Controller: PATA_CONTROLLER,
    mut ChanData: PCHANNEL_DATA_PATA,
) -> NTSTATUS {
    let mut CommandPortBase: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ControlPortBase: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let BarIndex: ULONG = (*ChanData).__bindgen_anon_1.Channel.wrapping_mul(2);

    PAGED_CODE!();

    CommandPortBase = AtaCtrlPciMapBar(Controller, BarIndex, PCIIDE_COMMAND_IO_RANGE_LENGTH);
    if CommandPortBase.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ControlPortBase = AtaCtrlPciMapBar(
        Controller,
        BarIndex.wrapping_add(1),
        PCIIDE_CONTROL_IO_RANGE_LENGTH,
    );
    if ControlPortBase.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*Controller).AccessRange[BarIndex as usize].Flags & RANGE_IS_MAPPED != 0 {
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_MRES_TF;
    }

    if (*Controller).AccessRange[BarIndex.wrapping_add(1) as usize].Flags
        & RANGE_IS_MAPPED
        != 0
    {
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_MRES_CTRL;
    }

    PciIdeInitTaskFileIoResources(
        ChanData,
        (CommandPortBase as ULONG_PTR) as _,
        ((ControlPortBase as ULONG_PTR).wrapping_add((PCIIDE_CONTROL_IO_BAR_OFFSET) as _)) as _,
        1,
    );

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeAssignLegacyResources(
    mut ChanData: PCHANNEL_DATA_PATA,
    mut ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let mut CommandPortDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut ControlPortDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut InterruptDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut Spare: ULONG;

    PAGED_CODE!();

    if ResourcesTranslated.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < (*ResourcesTranslated).List[0].PartialResourceList.Count {
        let mut Desc: PCM_PARTIAL_RESOURCE_DESCRIPTOR;

        Desc = &mut (*ResourcesTranslated).List[0]
            .PartialResourceList
            .PartialDescriptors[i as usize];

        match (*Desc).Type {
            CmResourceTypePort | CmResourceTypeMemory => {
                if IsNEC_98 != 0
                    && (*ResourcesTranslated).List[0]
                        .PartialResourceList
                        .Count
                        >= 12
                {
                    if (*Desc).u.Port.Start.QuadPart == 0x640 {
                        CommandPortDesc = Desc;
                        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_CBUS;
                    } else if (*Desc).u.Port.Start.QuadPart == 0x74C {
                        ControlPortDesc = Desc;
                        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_CBUS;
                    }
                } else if (*Desc).u.Port.Length == PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH {
                    if ControlPortDesc.is_null() {
                        ControlPortDesc = Desc;
                    }
                } else if (*Desc).u.Port.Length == PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH {
                    if CommandPortDesc.is_null() {
                        CommandPortDesc = Desc;
                    }
                }
            }
            CmResourceTypeInterrupt => {
                if InterruptDesc.is_null() {
                    InterruptDesc = Desc;
                }
            }
            _ => {}
        }

        i = i.wrapping_add(1);
    }

    if CommandPortDesc.is_null() || ControlPortDesc.is_null() || InterruptDesc.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    if (*CommandPortDesc).Type == CmResourceTypeMemory {
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_MRES_TF;
    }

    if (*ControlPortDesc).Type == CmResourceTypeMemory {
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_MRES_CTRL;
    }

    RtlCopyMemory!(
        &mut (*ChanData).__bindgen_anon_1.InterruptDesc as *mut _ as *mut core::ffi::c_void,
        InterruptDesc as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
    );

    #[cfg(target_arch = "x86")]
    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_CBUS != 0 {
        Spare = 2;
    } else {
        Spare = 1;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        Spare = 1;
    }

    PciIdeInitTaskFileIoResources(
        ChanData,
        ((*CommandPortDesc).u.Port.Start.QuadPart as ULONG_PTR) as _,
        ((*ControlPortDesc).u.Port.Start.QuadPart as ULONG_PTR) as _,
        Spare,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeParseResources(
    ChanData: *mut CHANNEL_DATA_PATA,
    ResourcesTranslated: *mut CM_RESOURCE_LIST,
) -> i32 {
    let Controller = (*ChanData).__bindgen_anon_1.Controller;
    let Status: i32;

    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_NATIVE_PCI != 0 {
        Status = PciIdeAssignNativeResources(Controller, ChanData);
    } else {
        Status = PciIdeAssignLegacyResources(ChanData, ResourcesTranslated);
    }
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    PataClaimLegacyAddressRanges(ChanData);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeFreeResources(ChannelContext: *mut core::ffi::c_void) {
    let mut ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;

    PAGED_CODE!();

    if !(*ChanData).__bindgen_anon_1.InterruptObject.is_null() {
        AtaChanEnableInterruptsSync((ChanData).cast(), FALSE);

        IoDisconnectInterrupt((*ChanData).__bindgen_anon_1.InterruptObject);
        (*ChanData).__bindgen_anon_1.InterruptObject = core::ptr::null_mut();
    }

    (*ChanData).Regs.Dma = core::ptr::null_mut();

    (*ChanData).__bindgen_anon_1.ChanInfo &= !(CHANNEL_FLAG_DRIVE0_DMA_CAPABLE | CHANNEL_FLAG_DRIVE1_DMA_CAPABLE);

    PataReleaseLegacyAddressRanges(ChanData);
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeGetChannelState(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
) -> IDE_CHANNEL_STATE {
    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_NATIVE_PCI != 0 {
        if (*Controller).Flags & CTRL_FLAG_MANUAL_RES == 0 {
            let BarIndex: ULONG = Channel.wrapping_mul(2);

            if (*Controller).AccessRange[BarIndex as usize].Flags & RANGE_IS_VALID == 0 {
                return ChannelDisabled;
            }

            if (*Controller).AccessRange[BarIndex.wrapping_add(1) as usize].Flags
                & RANGE_IS_VALID
                == 0
            {
                return ChannelDisabled;
            }
        }
    }

    if !(*Controller).__bindgen_anon_1.ChannelEnableBits.is_null() {
        if (*Controller).Flags & CTRL_FLAG_USE_TEST_FUNCTION != 0 {
            return (*Controller).__bindgen_anon_1.ChannelEnabledTest.expect("non-null function pointer")(
                Controller,
                Channel,
            );
        } else {
            let EnableBits: *const ATA_PCI_ENABLE_BITS =
                (*Controller).__bindgen_anon_1.ChannelEnableBits.offset(Channel as isize);
            let mut RegisterValue: UCHAR;

            RegisterValue = PciRead8(Controller, ((*EnableBits).Register) as ULONG);
            if RegisterValue as i32 & (*EnableBits).Mask as i32
                == (*EnableBits).ValueEnabled as i32
            {
                return ChannelEnabled;
            } else {
                return ChannelDisabled;
            }
        }
    }

    return ChannelStateUnknown;
}

unsafe extern "C" fn PataEnableInterrupts(
    mut ChannelContext: *mut core::ffi::c_void,
    mut Enable: i32,
) {
    let mut ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let mut Control: u8;

    INFO!(
        b"CH %lu: %sable interrupts\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        if Enable != 0 {
            b"En\0".as_ptr() as *const i8
        } else {
            b"Dis\0".as_ptr() as *const i8
        },
    );

    Control = IDE_DC_ALWAYS;
    if Enable != 0 {
        Control |= (IDE_DC_REENABLE_CONTROLLER) as u8;
    } else {
        Control |= (IDE_DC_DISABLE_INTERRUPTS) as u8;
    }

    #[cfg(target_arch = "x86")]
    {
        if (*ChanData).ChanInfo & CHANNEL_FLAG_CBUS != 0 {
            WRITE_PORT_UCHAR(
                PC98_ATA_BANK as *mut u8,
                1,
            );
            ATA_WRITE!(
                (*ChanData).Regs.__bindgen_anon_6.Control,
                Control,
                ChanData as *mut core::ffi::c_void,
                MRES_CTRL,
            );

            if !(*ChanData).CheckInterrupt.is_null() {
                (((*ChanData).CheckInterrupt).unwrap_unchecked())((ChanData as *mut core::ffi::c_void).cast());
            }
            (((*ChanData).ReadStatus).unwrap_unchecked())((ChanData as *mut core::ffi::c_void).cast());

            WRITE_PORT_UCHAR(
                PC98_ATA_BANK as *mut u8,
                0,
            );
            (*ChanData).LastAtaBankId = 0xFF;
        }
    }

    ATA_WRITE!(
        (*ChanData).Regs.__bindgen_anon_6.Control,
        Control,
        ChanData as *mut core::ffi::c_void,
        MRES_CTRL,
    );

    if !(*ChanData).CheckInterrupt.is_null() {
        (((*ChanData).CheckInterrupt).unwrap_unchecked())((ChanData as *mut core::ffi::c_void).cast());
    }
    (((*ChanData).ReadStatus).unwrap_unchecked())((ChanData as *mut core::ffi::c_void).cast());

    if !(*ChanData).Regs.Dma.is_null() {
        let mut Status: u8 = ATA_READ!(
            (*ChanData).Regs.Dma.add(PCIIDE_DMA_STATUS as usize),
            ChanData as *mut core::ffi::c_void,
            MRES_DMA,
        );

        ATA_WRITE!(
            (*ChanData).Regs.Dma.add(PCIIDE_DMA_STATUS as usize),
            Status,
            ChanData as *mut core::ffi::c_void,
            MRES_DMA,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeConnectInterrupt(ChannelContext: *mut core::ffi::c_void) -> i32 {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut InterruptDesc: *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut IsrHandler: PKSERVICE_ROUTINE;
    let Status: i32;

    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_NATIVE_PCI != 0 {
        InterruptDesc = &raw mut (*Controller).InterruptDesc as *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    } else {
        InterruptDesc = &raw mut (*ChanData).__bindgen_anon_1.InterruptDesc as *mut PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    }

    if !(*ChanData).Regs.Dma.is_null() {
        IsrHandler = Some(PciIdeChannelIsr);
    } else {
        IsrHandler = Some(PataChannelIsr);
    }

    Status = IoConnectInterrupt(
        &raw mut (*ChanData).__bindgen_anon_1.InterruptObject,
        IsrHandler,
        ChanData as *mut core::ffi::c_void,
        core::ptr::null_mut(),
        (*(*InterruptDesc)).u.Interrupt.Vector,
        ((*(*InterruptDesc)).u.Interrupt.Level) as u8,
        ((*(*InterruptDesc)).u.Interrupt.Level) as u8,
        if (*(*InterruptDesc)).Flags as u32 & CM_RESOURCE_INTERRUPT_LATCHED != 0 {
            Latched
        } else {
            LevelSensitive
        },
        ((*(*InterruptDesc)).ShareDisposition == CmResourceShareShared) as u8,
        (*(*InterruptDesc)).u.Interrupt.Affinity,
        FALSE,
    );

    if !NT_SUCCESS!(Status) {
        return Status;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeAttachChannel(
    ChannelContext: *mut core::ffi::c_void,
    Attach: u8,
) -> i32 {
    let ChanData: *mut CHANNEL_DATA_PATA = ChannelContext as *mut CHANNEL_DATA_PATA;

    PAGED_CODE!();

    AtaChanEnableInterruptsSync(ChanData as *mut core::ffi::c_void, Attach);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeCreateChannelData(
    Controller: *mut PATA_CONTROLLER,
    HwExtensionSize: u32,
) -> i32 {
    let mut ChanData: *mut CHANNEL_DATA_PATA;
    let mut i: u32;
    let mut Size: u32;

    Size = core::mem::offset_of!(CHANNEL_DATA_PATA, HwExt) as u32 + HwExtensionSize;
    ChanData = ExAllocatePoolZero(
        NonPagedPool,
        ((Size * (*(*Controller)).MaxChannels) as usize) as _,
        TAG_PCIIDEX,
    ) as *mut CHANNEL_DATA_PATA;
    if ChanData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*(*Controller)).ChanDataBlock = ChanData as *mut core::ffi::c_void;
    (*(*Controller)).ChannelBitmap = NUM_TO_BITMAP!((*(*Controller)).MaxChannels);

    i = 0;
    while i < (*(*Controller)).MaxChannels {
        (*(*Controller)).Channels[i as usize] = ChanData as *mut core::ffi::c_void;

        (*ChanData).__bindgen_anon_1.Channel = i;
        (*ChanData).__bindgen_anon_1.Controller = Controller as *mut _ATA_CONTROLLER;

        (*ChanData).__bindgen_anon_1.ChanInfo = CHANNEL_FLAG_IO32;
        (*ChanData).__bindgen_anon_1.AllocateMemory = Some(PciIdeAllocateMemory);
        (*ChanData).__bindgen_anon_1.FreeMemory = Some(PciIdeFreeMemory);
        (*ChanData).__bindgen_anon_1.EnableInterrupts = Some(PataEnableInterrupts as _);
        (*ChanData).__bindgen_anon_1.PreparePrdTable = Some(PciIdePreparePrdTable);
        (*ChanData).__bindgen_anon_1.PrepareIo = Some(PataPrepareIo as _);
        (*ChanData).__bindgen_anon_1.StartIo = Some(PataStartIo as _);
        (*ChanData).LoadTaskFile = Some(PataLoadTaskFile as PCHANNEL_LOAD_TASK_FILE);
        (*ChanData).SaveTaskFile = Some(PataSaveTaskFile as PCHANNEL_SAVE_TASK_FILE);
        (*ChanData).ReadStatus = Some(PataReadStatus);
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(PciIdeGenericSetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported =
            PIO_ALL | SWDMA_ALL | MWDMA_ALL | UDMA_ALL;

        KeInitializeDpc(
            core::ptr::addr_of_mut!((*ChanData).PollingTimerDpc),
            Some(PataPollingTimerDpc),
            ChanData as *mut core::ffi::c_void,
        );

        ChanData = (ChanData as usize + Size as usize) as *mut CHANNEL_DATA_PATA;
        i += 1;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeGetControllerProperties(
    Controller: *mut PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    PAGED_CODE!();

    (*(*Controller)).MaxChannels = MAX_IDE_CHANNEL;

    match ((*(*Controller)).Pci.VendorID) as u32 {
        PCI_VEN_ATI => {
            Status = AtiGetControllerProperties((Controller).cast());
        }
        PCI_VEN_AMD | PCI_VEN_NVIDIA => {
            Status = AmdGetControllerProperties((Controller).cast());
        }
        PCI_VEN_CMD => {
            Status = CmdGetControllerProperties((Controller).cast());
            if Status == STATUS_NO_MATCH {
                Status = Sil680GetControllerProperties(Controller);
            }
        }
        PCI_VEN_INTEL => {
            Status = IntelGetControllerProperties((Controller).cast());
        }
        PCI_VEN_PC_TECH => {
            Status = PcTechGetControllerProperties((Controller).cast());
        }
        PCI_VEN_SERVERWORKS => {
            Status = SvwPataGetControllerProperties((Controller).cast());
            if Status == STATUS_NO_MATCH {
                Status = SvwSataGetControllerProperties((Controller).cast());
            }
        }
        PCI_VEN_TOSHIBA => {
            Status = ToshibaGetControllerProperties((Controller).cast());
        }
        PCI_VEN_VIA => {
            Status = ViaGetControllerProperties(Controller);
        }
        _ => {
            Status = STATUS_NO_MATCH;
        }
    }

    if Status == STATUS_NO_MATCH {
        if (((*(*Controller)).Pci.BaseClass) as u32 != PCI_CLASS_MASS_STORAGE_CTLR)
            || (((*(*Controller)).Pci.SubClass) as u32 != PCI_SUBCLASS_MSC_IDE_CTLR
                && ((*(*Controller)).Pci.SubClass) as u32 != PCI_SUBCLASS_MSC_RAID_CTLR)
        {
            return Status;
        }

        Status = PciIdeCreateChannelData(Controller, 0);
        if !NT_SUCCESS!(Status) {
            return Status;
        }

        i = 0;
        while i < (*(*Controller)).MaxChannels {
            let ChanData: *mut PCHANNEL_DATA_PATA = ((*(*Controller)).Channels[i as usize]).cast();
            (*(*ChanData)).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_IO32;
            i = i.wrapping_add(1);
        }
    }

    if !NT_SUCCESS!(Status) {
        return Status;
    }

    if ((*(*Controller)).Pci.BaseClass) as u32 == PCI_CLASS_MASS_STORAGE_CTLR
        && ((*(*Controller)).Pci.SubClass) as u32 == PCI_SUBCLASS_MSC_IDE_CTLR
    {
        if ((*(*Controller)).Pci.ProgIf as u32 & PCIIDE_PROGIF_PRIMARY_CHANNEL_NATIVE_MODE) == 0
            || ((*(*Controller)).Pci.ProgIf as u32 & PCIIDE_PROGIF_SECONDARY_CHANNEL_NATIVE_MODE) == 0
        {
            (*(*Controller)).Flags &= !CTRL_FLAG_NATIVE_PCI;
        }
    }

    if (*(*Controller)).Flags & CTRL_FLAG_NATIVE_PCI != 0 {
        if (*(*Controller)).InterruptDesc.Type != CmResourceTypeInterrupt {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if PciIdeControllerInitDma((Controller).cast()) == 0 {
        i = 0;
        while i < (*(*Controller)).MaxChannels {
            let ChanData: *mut PCHANNEL_DATA_PATA = ((*(*Controller)).Channels[i as usize]).cast();
            (*(*ChanData)).__bindgen_anon_1.TransferModeSupported &= PIO_ALL;
            i = i.wrapping_add(1);
        }
    }

    if (*(*Controller)).Flags & CTRL_FLAG_IS_SIMPLEX != 0 {
        (*(*Controller)).HwSyncObject = IoCreateController(0);
        if (*(*Controller)).HwSyncObject.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    return STATUS_SUCCESS;
}
