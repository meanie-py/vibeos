use crate::prelude::*;
use ntkrnl_sys::ddk::{DeviceNumber, KeStallExecutionProcessor, Mask, PortNumber};
use core::ffi::c_void;

unsafe extern "C" fn AtaAhciPollRegister(
    mut IoBase: PVOID,
    mut Register: AHCI_PORT_REGISTER,
    mut Mask: ULONG,
    mut Value: ULONG,
    mut TimeOut: ULONG,
) -> BOOLEAN {
    let mut i: ULONG = 0;
    let mut Data: ULONG = 0;
    ASSUME!(TimeOut > 0);
    i = 0;
    while i < 10 {
        Data = AHCI_PORT_READ(IoBase, Register);
        if Data & Mask == Value {
            return TRUE;
        }
        KeStallExecutionProcessor(10);
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < TimeOut {
        Data = AHCI_PORT_READ(IoBase, Register);
        if Data & Mask == Value {
            return TRUE;
        }
        AtaSleep();
        i = i.wrapping_add(1);
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciEnableInterrupts(
    ChannelContext: *mut core::ffi::c_void,
    Enable: u8,
) {
    let ChanData: *mut CHANNEL_DATA_AHCI = ChannelContext as *mut CHANNEL_DATA_AHCI;

    INFO!(
        b"CH %lu: %sble interrupts\n\0".as_ptr() as *const i8,
        (*ChanData).__bindgen_anon_1.Channel,
        if Enable != 0 {
            b"Ena\0".as_ptr() as *const i8
        } else {
            b"Disa\0".as_ptr() as *const i8
        },
    );

    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
    AHCI_PORT_WRITE((*ChanData).IoBase, PxInterruptStatus, 0xFFFFFFFF);
    AHCI_HBA_WRITE(
        (*(*ChanData).__bindgen_anon_1.Controller).IoBase,
        HbaInterruptStatus,
        1 << (*ChanData).__bindgen_anon_1.Channel,
    );

    AHCI_PORT_WRITE(
        (*ChanData).IoBase,
        PxInterruptEnable,
        if Enable != 0 {
            AHCI_PORT_INTERRUPT_MASK
        } else {
            0
        },
    );
}

unsafe extern "C" fn AtaAhciIsHbaHotRemoved(mut ChanData: PCHANNEL_DATA_AHCI) -> BOOLEAN {
    let mut Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut WasRemoved: BOOLEAN = 0;
    WasRemoved = (AHCI_HBA_READ((*Controller).IoBase, HbaAhciVersion)
        == 0xFFFFFFFF) as BOOLEAN;
    if WasRemoved != 0 {
    }
    return WasRemoved;
}

unsafe extern "C" fn AtaAhciSendComReset(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut SataControl: ULONG = 0;
    INFO!(
        b"CH %lu: Transmit a COMRESET on the interface\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
    );
    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
    SataControl = AHCI_PORT_READ((*ChanData).IoBase, PxSataControl);
    SataControl &= !AHCI_PXCTL_DET_MASK;
    SataControl |= AHCI_PXCTL_DET_RESET;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataControl, SataControl);
    KeStallExecutionProcessor(1000);
    SataControl = AHCI_PORT_READ((*ChanData).IoBase, PxSataControl);
    SataControl &= !AHCI_PXCTL_DET_MASK;
    SataControl |= AHCI_PXCTL_DET_IDLE;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataControl, SataControl);
    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
}

unsafe extern "C" fn AtaAhciStopCommandListProcess(ChanData: PCHANNEL_DATA_AHCI) {
    let mut CmdStatus: ULONG;

    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    if CmdStatus & AHCI_PXCMD_ST != 0 {
        CmdStatus &= !AHCI_PXCMD_ST;
        AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    }
}

unsafe extern "C" fn AtaAhciStartCommandListProcess(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut i: ULONG = 0;
    let mut CmdStatus: ULONG = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus |= AHCI_PXCMD_ST;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    i = 0;
    while i < 10 {
        CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
        if CmdStatus & AHCI_PXCMD_CR != 0 {
            break;
        }
        KeStallExecutionProcessor(5);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn AtaAhciStopCommandListProcessAndWait(
    ChanData: PCHANNEL_DATA_AHCI,
) -> BOOLEAN {
    AtaAhciStopCommandListProcess(ChanData);

    if AtaAhciIsHbaHotRemoved(ChanData) != 0 {
        return FALSE;
    }

    return AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxCmdStatus,
        AHCI_PXCMD_CR,
        0,
        AHCI_DELAY_CR_START_STOP,
    );
}

unsafe extern "C" fn AtaAhciStopFisReceiveProcess(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut CmdStatus: ULONG = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    if CmdStatus & AHCI_PXCMD_FRE != 0 {
        CmdStatus &= !AHCI_PXCMD_FRE;
        AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    }
}

unsafe extern "C" fn AtaAhciStartFisReceiveProcess(ChanData: PCHANNEL_DATA_AHCI) {
    let mut CmdStatus: ULONG = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus |= AHCI_PXCMD_FRE;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
}

unsafe extern "C" fn AtaAhciStopFisReceiveProcessAndWait(
    ChanData: PCHANNEL_DATA_AHCI,
) -> BOOLEAN {
    AtaAhciStopFisReceiveProcess(ChanData);

    if AtaAhciIsHbaHotRemoved(ChanData) != 0 {
        return FALSE;
    }

    return AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxCmdStatus,
        AHCI_PXCMD_FR,
        0,
        AHCI_DELAY_FR_START_STOP,
    );
}

unsafe extern "C" fn AtaAhciStartFisReceiveProcessAndWait(mut ChanData: PCHANNEL_DATA_AHCI) {
    AtaAhciStartFisReceiveProcess(ChanData);

    if ((AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxCmdStatus,
        AHCI_PXCMD_FR,
        AHCI_PXCMD_FR,
        AHCI_DELAY_FR_START_STOP,
    )) == 0) {
        /* Ignore timeouts, on some AHCI controllers the FR bit never becomes set */
    }
}

unsafe extern "C" fn AtaAhciPhyEnterListenMode(ChanData: PCHANNEL_DATA_AHCI) {
    let mut CmdStatus: ULONG;
    let mut SataControl: ULONG;

    if (*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_SSS == 0 {
        return;
    }

    INFO!(
        b"CH %lu: Enter listen mode\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
    );

    AtaAhciStopCommandListProcessAndWait(ChanData);

    SataControl = AHCI_PORT_READ((*ChanData).IoBase, PxSataControl);
    SataControl &= !AHCI_PXCTL_DET_MASK;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataControl, SataControl);

    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus &= !(AHCI_PXCMD_SUD | AHCI_PXCMD_ICC_MASK);
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
}

unsafe extern "C" fn AtaAhciPerformCommandListOverride(
    mut ChanData: PCHANNEL_DATA_AHCI,
) -> BOOLEAN {
    let mut CmdStatus: ULONG = 0;
    if (*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_SCLO == 0 {
        return TRUE;
    }
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus |= AHCI_PXCMD_CLO;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    return AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxCmdStatus,
        AHCI_PXCMD_CLO,
        0,
        AHCI_DELAY_CLO_CLEAR,
    );
}

unsafe extern "C" fn AtaAhciAtapiLedControl(
    ChanData: PCHANNEL_DATA_AHCI,
    DoEnable: BOOLEAN,
) {
    let mut CmdStatus: ULONG;
    let mut NewCmdStatus: ULONG;

    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    if DoEnable != 0 {
        NewCmdStatus = CmdStatus | AHCI_PXCMD_ATAPI;
    } else {
        NewCmdStatus = CmdStatus & !AHCI_PXCMD_ATAPI;
    }

    if CmdStatus != NewCmdStatus {
        AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, NewCmdStatus);
    }
}

unsafe extern "C" fn AtaAhciFbsControl(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut DoEnable: BOOLEAN,
) {
    let mut FbsControl: ULONG = 0;
    let mut NewFbsControl: ULONG = 0;

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_FBS == 0 {
        return;
    }

    (*ChanData).LastFbsDeviceNumber = 0xFF;

    FbsControl = AHCI_PORT_READ((*ChanData).IoBase, PxFisSwitchingControl);

    if DoEnable != 0 {
        NewFbsControl = FbsControl | AHCI_FBS_ENABLE;
    } else {
        NewFbsControl = FbsControl & !AHCI_FBS_ENABLE;
        (*ChanData).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_FBS_ENABLED;
    }

    if (FbsControl & NewFbsControl) ^ AHCI_FBS_ENABLE != 0 {
        AHCI_PORT_WRITE((*ChanData).IoBase, PxFisSwitchingControl, NewFbsControl);
    }

    if DoEnable == 0 {
        return;
    }

    FbsControl = AHCI_PORT_READ((*ChanData).IoBase, PxFisSwitchingControl);

    if FbsControl & AHCI_FBS_ENABLE != 0 {
        (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_FBS_ENABLED;
        INFO!(
            b"CH %lu: FBS enabled\n\0".as_ptr() as *const i8,
            (*ChanData).Channel,
        );
    } else {
    }
}

unsafe extern "C" fn AtaAhciEnterIdleState(ChanData: PCHANNEL_DATA_AHCI) -> BOOLEAN {
    let mut CmdStatus: ULONG = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    if CmdStatus & (AHCI_PXCMD_ST | AHCI_PXCMD_CR | AHCI_PXCMD_FRE | AHCI_PXCMD_FR) == 0 {
        return TRUE;
    }
    if AtaAhciStopCommandListProcessAndWait(ChanData) == 0 {
        return FALSE;
    }
    AtaAhciPerformCommandListOverride(ChanData);
    if AtaAhciStopFisReceiveProcessAndWait(ChanData) == 0 {
        return FALSE;
    }
    return TRUE;
}

unsafe extern "C" fn AtaAhciSetupDmaMemoryAddress(mut ChanData: PCHANNEL_DATA_AHCI) {
    AHCI_PORT_WRITE(
        (*ChanData).IoBase,
        PxCommandListBaseLow,
        (*ChanData).Mem.CommandListPhys as ULONG,
    );
    if (*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_S64A != 0 {
        AHCI_PORT_WRITE(
            (*ChanData).IoBase,
            PxCommandListBaseHigh,
            ((*ChanData).Mem.CommandListPhys >> 32) as ULONG,
        );
    }
    AHCI_PORT_WRITE(
        (*ChanData).IoBase,
        PxFisBaseLow,
        (*ChanData).Mem.ReceivedFisPhys as ULONG,
    );
    if (*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_S64A != 0 {
        AHCI_PORT_WRITE(
            (*ChanData).IoBase,
            PxFisBaseHigh,
            ((*ChanData).Mem.ReceivedFisPhys >> 32) as ULONG,
        );
    }
}

unsafe extern "C" fn AtaAhciSpinUp(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut CmdStatus: ULONG = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus &= !AHCI_PXCMD_PMA;
    CmdStatus &= !AHCI_PXCMD_ICC_MASK;
    CmdStatus |= AHCI_PXCMD_ICC_ACTIVE;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxCmdStatus,
        AHCI_PXCMD_ICC_MASK,
        AHCI_PXCMD_ICC_IDLE,
        AHCI_DELAY_INTERFACE_CHANGE,
    );
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    if (*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_SSS != 0 {
        CmdStatus |= AHCI_PXCMD_SUD;
    }
    if CmdStatus & AHCI_PXCMD_CPD != 0 {
        CmdStatus |= AHCI_PXCMD_POD;
    }
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
}

unsafe extern "C" fn AtaAhciPhyCheckDevicePresence(
    ChanData: *mut CHANNEL_DATA_AHCI,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut SataStatus: ULONG;

    /* Do a quick check first (100 us) */
    i = 0;
    while i < 10 {
        SataStatus = AHCI_PORT_READ((*ChanData).IoBase, PxSataStatus)
            & AHCI_PXSSTS_DET_MASK;
        if SataStatus != AHCI_PXSSTS_DET_NO_DEVICE {
            return TRUE;
        }
        KeStallExecutionProcessor(10);
        i = i.wrapping_add(1);
    }

    /* Retry after the time interval */
    i = 0;
    while i < AHCI_DELAY_DET_PRESENCE {
        SataStatus = AHCI_PORT_READ((*ChanData).IoBase, PxSataStatus)
            & AHCI_PXSSTS_DET_MASK;
        if SataStatus != AHCI_PXSSTS_DET_NO_DEVICE {
            return TRUE;
        }
        AtaSleep();
        i = i.wrapping_add(1);
    }

    return FALSE;
}

unsafe extern "C" fn AtaAhciPhyWaitForReady(ChanData: PCHANNEL_DATA_AHCI) -> BOOLEAN {
    return AtaAhciPollRegister(
        (*ChanData).IoBase,
        PxSataStatus,
        AHCI_PXSSTS_DET_MASK,
        AHCI_PXSSTS_DET_PHY_OK,
        AHCI_DELAY_DET_STABLE,
    );
}

unsafe extern "C" fn AtaAhciWaitForDeviceReady(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut TimeOut: ULONG,
) -> BOOLEAN {
    let mut i: ULONG = 0;
    let mut TaskFileData: ULONG = 0;
    i = 0;
    while i < 10 {
        TaskFileData = AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData);
        if TaskFileData & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) == 0 {
            return TRUE;
        }
        KeStallExecutionProcessor(10);
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < TimeOut {
        TaskFileData = AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData);
        if TaskFileData & (IDE_STATUS_BUSY | IDE_STATUS_DRQ) == 0 {
            return TRUE;
        }
        if i.wrapping_rem(AHCI_DELAY_1_SECOND) == 0 && i != 0 {
            if AtaAhciIsHbaHotRemoved(ChanData) != 0 {
                return FALSE;
            }
            AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
        }
        AtaSleep();
        i = i.wrapping_add(1);
    }
    return FALSE;
}

unsafe extern "C" fn AtaAhciPostRequestPolled(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut TimeOutMs: ULONG,
) -> UCHAR {
    let mut InterruptStatus: ULONG = 0;
    let mut TaskFileData: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;

    'done: {
        if !((*ChanData).__bindgen_anon_1.ActiveSlotsBitmap == 0 as ULONG
            && (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap == 0 as ULONG)
        {
            RtlAssert(
                b"(ChanData->ActiveSlotsBitmap == 0) && (ChanData->ActiveQueuedSlotsBitmap == 0)\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/pciidex/chipset/ahci_hw.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                542 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }

        AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
        AHCI_PORT_WRITE(
            (*ChanData).IoBase,
            PxCommandIssue,
            1 << AHCI_INTERNAL_SLOT,
        );
        if AtaAhciPollRegister(
            (*ChanData).IoBase,
            PxCommandIssue,
            1 << AHCI_INTERNAL_SLOT,
            0,
            TimeOutMs / PORT_TIMER_TICK_MS,
        ) == 0
        {
            INFO!(
                b"CH %lu: Internal request timed out\n\0".as_ptr() as *const i8,
                (*ChanData).__bindgen_anon_1.Channel,
            );

            AtaAhciStopCommandListProcessAndWait(ChanData);
            AtaAhciPerformCommandListOverride(ChanData);
            AtaAhciStartCommandListProcess(ChanData);

            SrbStatus = (SRB_STATUS_TIMEOUT) as u8;
            break 'done;
        }

        InterruptStatus = AHCI_PORT_READ((*ChanData).IoBase, PxInterruptStatus);

        if (InterruptStatus & AHCI_PXIRQ_FATAL_ERROR) != 0 {
            if AtaAhciStopCommandListProcessAndWait(ChanData) == 0 {
                INFO!(
                    b"CH %lu: Failed to stop the command list DMA engine\n\0".as_ptr() as *const i8,
                    (*ChanData).__bindgen_anon_1.Channel,
                );

                SrbStatus = (SRB_STATUS_TIMEOUT) as u8;
                break 'done;
            }

            TaskFileData = AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData);
            if (TaskFileData & (IDE_STATUS_BUSY | IDE_STATUS_DRQ)) != 0 {
                INFO!(
                    b"CH %lu: Busy TFD %08lx\n\0".as_ptr() as *const i8,
                    (*ChanData).__bindgen_anon_1.Channel,
                    TaskFileData,
                );

                SrbStatus = (SRB_STATUS_TIMEOUT) as u8;
                break 'done;
            }

            AHCI_PORT_WRITE(
                (*ChanData).IoBase,
                PxSataError,
                AHCI_PORT_READ((*ChanData).IoBase, PxSataError),
            );

            AtaAhciStartCommandListProcess(ChanData);

            SrbStatus = (SRB_STATUS_ERROR) as u8;
            break 'done;
        }

        SrbStatus = (SRB_STATUS_SUCCESS) as u8;
    }

    if (SrbStatus) as u32 == SRB_STATUS_SUCCESS {
        TRACE!(
            b"CH %lu: Completed internal request\n\0".as_ptr() as *const i8,
            (*ChanData).__bindgen_anon_1.Channel,
        );
    } else {
        INFO!(
            b"CH %lu: Internal request failed %02x\n\0".as_ptr() as *const i8,
            (*ChanData).__bindgen_anon_1.Channel,
            SrbStatus,
        );
    }

    AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);
    AHCI_PORT_WRITE((*ChanData).IoBase, PxInterruptStatus, 0xFFFFFFFF);
    AHCI_HBA_WRITE(
        (*(*ChanData).__bindgen_anon_1.Controller).IoBase,
        HbaInterruptStatus,
        1 << (*ChanData).__bindgen_anon_1.Channel,
    );

    SrbStatus
}

unsafe extern "C" fn AtaAhciSendResetFis(
    ChanData: PCHANNEL_DATA_AHCI,
    PortNumber: UCHAR,
    AssertSRST: BOOLEAN,
    TimeOutMs: ULONG,
) -> UCHAR {
    let CommandTable: PAHCI_COMMAND_TABLE = (*ChanData).CommandTable[AHCI_INTERNAL_SLOT as usize];
    let CommandHeader: PAHCI_COMMAND_HEADER = &mut (*(*ChanData).CommandList).CommandHeader[AHCI_INTERNAL_SLOT as usize];
    let H2dFis: PAHCI_FIS_HOST_TO_DEVICE = &mut (*CommandTable).__bindgen_anon_1.HostToDeviceFis;

    RtlZeroMemory!(
        H2dFis as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>(),
    );
    (*H2dFis).Type = (AHCI_FIS_REGISTER_HOST_TO_DEVICE) as u8;
    (*H2dFis).Flags = PortNumber;
    (*H2dFis).Control = IDE_DC_ALWAYS;

    (*CommandHeader).PrdByteCount = 0;
    (*CommandHeader).Control = ((core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>() / core::mem::size_of::<ULONG>())
        | ((PortNumber as usize) << AHCI_COMMAND_HEADER_PMP_SHIFT)) as ULONG;

    if AssertSRST != 0 {
        (*H2dFis).Control |= (IDE_DC_RESET_CONTROLLER) as u8;
        (*CommandHeader).Control |= AHCI_COMMAND_HEADER_RESET | AHCI_COMMAND_HEADER_CLEAR_BUSY_UPON_OK;
    }

    return AtaAhciPostRequestPolled(ChanData, TimeOutMs);
}

unsafe extern "C" fn AtaAhciPmpRead(
    ChanData: PCHANNEL_DATA_AHCI,
    PortNumber: UCHAR,
    Register: USHORT,
    Result: PULONG,
) -> UCHAR {
    let CommandTable: PAHCI_COMMAND_TABLE = (*ChanData).CommandTable[AHCI_INTERNAL_SLOT as usize];
    let CommandHeader: PAHCI_COMMAND_HEADER =
        (&raw mut (*(*ChanData).CommandList).CommandHeader as *mut AHCI_COMMAND_HEADER)
            .offset(AHCI_INTERNAL_SLOT as isize) as PAHCI_COMMAND_HEADER;
    let H2dFis: PAHCI_FIS_HOST_TO_DEVICE = &raw mut (*CommandTable).__bindgen_anon_1.HostToDeviceFis;
    let D2hFis: PAHCI_FIS_DEVICE_TO_HOST = &raw mut (*(*ChanData).ReceivedFis).DeviceToHostFis;
    let mut SrbStatus: UCHAR;

    RtlZeroMemory!(
        H2dFis as *mut core::ffi::c_void,
        core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>(),
    );
    (*H2dFis).Type = AHCI_FIS_REGISTER_HOST_TO_DEVICE as UCHAR;
    (*H2dFis).Flags = (UPDATE_COMMAND | AHCI_PMP_CONTROL_PORT) as UCHAR;
    (*H2dFis).Control = IDE_DC_ALWAYS as UCHAR;
    (*H2dFis).Device = (IDE_DRIVE_SELECT | (PortNumber as i32) as u32) as UCHAR;
    (*H2dFis).Features = Register as UCHAR;
    (*H2dFis).FeaturesEx = (Register as i32 >> 8) as UCHAR;
    (*H2dFis).Command = IDE_COMMAND_READ_PORT_MULTIPLIER as UCHAR;

    (*CommandHeader).PrdByteCount = 0 as ULONG;
    (*CommandHeader).Control = ((core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>() as usize)
        .wrapping_div(core::mem::size_of::<ULONG>() as usize)
        | ((AHCI_PMP_CONTROL_PORT as usize) << AHCI_COMMAND_HEADER_PMP_SHIFT)) as ULONG;

    SrbStatus = AtaAhciPostRequestPolled(ChanData, 1000 as ULONG);

    *Result = ((*D2hFis).SectorCount as i32
        | ((*D2hFis).LbaLow as i32) << 8
        | ((*D2hFis).LbaMid as i32) << 16
        | ((*D2hFis).LbaHigh as i32) << 24) as ULONG;

    return SrbStatus;
}

unsafe extern "C" fn AtaAhciPmpWrite(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut PortNumber: UCHAR,
    mut Register: USHORT,
    mut Value: ULONG,
) -> UCHAR {
    let mut CommandTable: PAHCI_COMMAND_TABLE =
        (*ChanData).CommandTable[AHCI_INTERNAL_SLOT as usize];
    let mut CommandHeader: PAHCI_COMMAND_HEADER =
        (&raw mut (*(*ChanData).CommandList).CommandHeader as *mut AHCI_COMMAND_HEADER)
            .offset(AHCI_INTERNAL_SLOT as isize) as PAHCI_COMMAND_HEADER;
    let mut H2dFis: PAHCI_FIS_HOST_TO_DEVICE =
        &raw mut (*CommandTable).__bindgen_anon_1.HostToDeviceFis;

    RtlZeroMemory!(
        H2dFis as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>() as ULONG,
    );
    (*H2dFis).Type = AHCI_FIS_REGISTER_HOST_TO_DEVICE as UCHAR;
    (*H2dFis).Flags = (UPDATE_COMMAND | AHCI_PMP_CONTROL_PORT) as UCHAR;
    (*H2dFis).Control = IDE_DC_ALWAYS as UCHAR;
    (*H2dFis).Device = (IDE_DRIVE_SELECT | (PortNumber as ::core::ffi::c_int) as u32) as UCHAR;
    (*H2dFis).Features = Register as UCHAR;
    (*H2dFis).FeaturesEx = (Register as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as UCHAR;
    (*H2dFis).SectorCount = Value as UCHAR;
    (*H2dFis).LbaLow = (Value >> 8 as ::core::ffi::c_int) as UCHAR;
    (*H2dFis).LbaMid = (Value >> 16 as ::core::ffi::c_int) as UCHAR;
    (*H2dFis).LbaHigh = (Value >> 24 as ::core::ffi::c_int) as UCHAR;
    (*H2dFis).Command = IDE_COMMAND_WRITE_PORT_MULTIPLIER as UCHAR;

    (*CommandHeader).PrdByteCount = 0 as ULONG;
    (*CommandHeader).Control = ((::core::mem::size_of::<AHCI_FIS_HOST_TO_DEVICE>() as usize)
        .wrapping_div(::core::mem::size_of::<ULONG>() as usize)
        | (AHCI_PMP_CONTROL_PORT << AHCI_COMMAND_HEADER_PMP_SHIFT) as usize)
        as ULONG;

    return AtaAhciPostRequestPolled(ChanData, 1000 as ULONG);
}

unsafe extern "C" fn AtaAhciPmpDisableSilXmitEarlyAck(
    mut ChanData: PCHANNEL_DATA_AHCI,
) -> BOOLEAN {
    let mut SrbStatus: UCHAR = 0;
    let mut Value: ULONG = 0;
    SrbStatus = AtaAhciPmpRead(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        129,
        &raw mut Value,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }
    if Value & 1 != 0 {
        Value &= !1;
        SrbStatus = AtaAhciPmpWrite(
            ChanData,
            (AHCI_PMP_CONTROL_PORT) as UCHAR,
            129,
            Value,
        );
        if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
            return FALSE;
        }
    }
    return TRUE;
}

unsafe extern "C" fn AtaAhciPmpIdentifyPmp(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut PortCountResult: PULONG,
) -> BOOLEAN {
    let mut ProductId: ULONG = 0;
    let mut RevisionInfo: ULONG = 0;
    let mut PortCount: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;

    *PortCountResult = 0;

    SrbStatus = AtaAhciPmpRead(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        (PmpProductId) as USHORT,
        &mut ProductId,
    );
    if (SrbStatus) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }

    SrbStatus = AtaAhciPmpRead(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        (PmpRevisionInfo) as USHORT,
        &mut RevisionInfo,
    );
    if (SrbStatus) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }

    SrbStatus = AtaAhciPmpRead(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        (PmpPortInfo) as USHORT,
        &mut PortCount,
    );
    if (SrbStatus) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }

    INFO!(
        b"CH %lu: PMP %08lX:%08lX %lu ports\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        ProductId,
        RevisionInfo,
        PortCount,
    );

    PortCount &= AHCI_MAX_PMP_DEVICES;

    match ProductId {
        0x37261095 | 0x38261095 => {
            if AtaAhciPmpDisableSilXmitEarlyAck(ChanData) == 0 {
                return FALSE;
            }
            if PortCount == 6 {
                PortCount = 5;
            }
        }
        0x47261095 => {
            if PortCount == 7 {
                PortCount = 5;
            }
        }
        0x57231095
        | 0x57331095
        | 0x57341095
        | 0x57441095 => {
            if PortCount > 0 {
                PortCount -= 1;
            }
        }
        _ => {}
    }

    *PortCountResult = PortCount;

    return TRUE;
}

unsafe extern "C" fn AtaAhciPmpDetect(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut CheckForPmp: PBOOLEAN,
) -> ATA_CONNECTION_STATUS {
    let mut CmdStatus: ULONG = 0;
    let mut Signature: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;
    CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
    CmdStatus |= AHCI_PXCMD_PMA;
    AHCI_PORT_WRITE((*ChanData).IoBase, PxCmdStatus, CmdStatus);
    AtaAhciFbsControl(ChanData, FALSE);
    AtaAhciPerformCommandListOverride(ChanData);
    AtaAhciStartCommandListProcess(ChanData);
    if AtaAhciWaitForDeviceReady(ChanData, AHCI_DELAY_PMP_READY_DRIVE) == 0 {
        AtaAhciSendComReset(ChanData);
        return CONN_STATUS_FAILURE;
    }
    SrbStatus = AtaAhciSendResetFis(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        TRUE,
        1000,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        INFO!(
            b"CH %lu: Soft reset failed %02x\n\0".as_ptr() as *const i8,
            (*ChanData).Channel,
            SrbStatus,
        );
        if (SrbStatus as ::core::ffi::c_int) as u32 == SRB_STATUS_TIMEOUT {
            *CheckForPmp = FALSE;
            INFO!(
                b"CH %lu: Seems to have no PMP device\n\0".as_ptr() as *const i8,
                (*ChanData).Channel,
            );
        }
        return CONN_STATUS_FAILURE;
    }
    KeStallExecutionProcessor(20);
    SrbStatus = AtaAhciSendResetFis(
        ChanData,
        (AHCI_PMP_CONTROL_PORT) as UCHAR,
        FALSE,
        2000,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        INFO!(
            b"CH %lu: Soft reset failed %02x\n\0".as_ptr() as *const i8,
            (*ChanData).Channel,
            SrbStatus,
        );
        if (SrbStatus as ::core::ffi::c_int) as u32 == SRB_STATUS_TIMEOUT {
            *CheckForPmp = FALSE;
            INFO!(
                b"CH %lu: Seems to have no PMP device\n\0".as_ptr() as *const i8,
                (*ChanData).Channel,
            );
        }
        return CONN_STATUS_FAILURE;
    }
    KeStallExecutionProcessor(20);
    Signature = AHCI_PORT_READ((*ChanData).IoBase, PxSignature);
    INFO!(
        b"CH %lu: Received signature %08lx\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        Signature,
    );
    if Signature != AHCI_PXSIG_PMP {
        return CONN_STATUS_NO_DEVICE;
    }
    return CONN_STATUS_DEV_UNKNOWN;
}

unsafe extern "C" fn AtaAhciPhyCheckConnection(
    mut ChanData: PCHANNEL_DATA_AHCI,
) -> ATA_CONNECTION_STATUS {
    let mut ConnectionStatus: ATA_CONNECTION_STATUS = 0;
    let mut RetryCount: ULONG = 0;

    RetryCount = 0;
    while RetryCount < 2 {
        if AtaAhciEnterIdleState(ChanData) == 0 {
            AtaAhciSendComReset(ChanData);
            ConnectionStatus = CONN_STATUS_FAILURE;
        } else {
            AtaAhciSetupDmaMemoryAddress(ChanData);
            AtaAhciSpinUp(ChanData);
            AtaAhciStartFisReceiveProcessAndWait(ChanData);
            AtaAhciSendComReset(ChanData);

            if AtaAhciPhyCheckDevicePresence(ChanData) == 0 {
                INFO!(
                    b"CH %lu: Device not connected %08lx\n\0".as_ptr() as *const i8,
                    (*ChanData).Channel,
                    AHCI_PORT_READ((*ChanData).IoBase, PxSataStatus),
                );
                ConnectionStatus = CONN_STATUS_NO_DEVICE;
                break;
            }

            INFO!(
                b"CH %lu: Link up %08lx\n\0".as_ptr() as *const i8,
                (*ChanData).Channel,
                AHCI_PORT_READ((*ChanData).IoBase, PxSataStatus),
            );

            if AtaAhciPhyWaitForReady(ChanData) == 0 {
                ConnectionStatus = CONN_STATUS_FAILURE;
            } else {
                AHCI_PORT_WRITE((*ChanData).IoBase, PxSataError, 0xFFFFFFFF);

                if AtaAhciWaitForDeviceReady(ChanData, AHCI_DELAY_READY_DRIVE) == 0 {
                    ConnectionStatus = CONN_STATUS_FAILURE;
                } else {
                    INFO!(
                        b"CH %lu: Device is ready %08lx\n\0".as_ptr() as *const i8,
                        (*ChanData).Channel,
                        AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData),
                    );
                    ConnectionStatus = CONN_STATUS_DEV_UNKNOWN;
                    break;
                }
            }
        }
        RetryCount = RetryCount.wrapping_add(1);
    }

    return ConnectionStatus;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciEnumerateChannel(
    ChannelContext: *mut core::ffi::c_void,
) -> u32 {
    let ChanData: *mut CHANNEL_DATA_AHCI = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let mut RetryCount: u32;
    let mut PortCount: u32;
    let mut ConnectionStatus: ATA_CONNECTION_STATUS;
    let mut CheckForPmp: u8 = (TRUE) as _;

    if AtaAhciIsHbaHotRemoved(ChanData) != 0 {
        return 0;
    }

    AtaChanEnableInterruptsSync((ChanData).cast(), FALSE);

    ((*(*ChanData).__bindgen_anon_1.PortNotification)(
        AtaResetDetected,
        (*ChanData).__bindgen_anon_1.PortContext,
        0xFFFFFFFF,
    ));

    RetryCount = 3;
    while RetryCount > 0 {
        (*ChanData).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_IS_PMP;

        ConnectionStatus = AtaAhciPhyCheckConnection(ChanData);
        if ConnectionStatus != CONN_STATUS_DEV_UNKNOWN {
            PortCount = 0;
            break;
        }

        if ((*(*ChanData).__bindgen_anon_1.Controller).AhciCapabilities & AHCI_CAP_SPM) != 0
            && RetryCount > 1
            && CheckForPmp != 0
        {
            INFO!(
                b"CH %lu: Trying to detect PMP\n\0".as_ptr() as *const i8,
                (*ChanData).__bindgen_anon_1.Channel,
            );

            ConnectionStatus = AtaAhciPmpDetect(ChanData, &mut CheckForPmp);
            if ConnectionStatus == CONN_STATUS_FAILURE {
                RetryCount = RetryCount.wrapping_sub(1);
                continue;
            }

            if ConnectionStatus == CONN_STATUS_DEV_UNKNOWN {
                INFO!(
                    b"CH %lu: Discovered a Port Multiplier\n\0".as_ptr() as *const i8,
                    (*ChanData).__bindgen_anon_1.Channel,
                );

                (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_IS_PMP;

                if AtaAhciPmpIdentifyPmp(ChanData, &mut PortCount) == 0 {
                    RetryCount = RetryCount.wrapping_sub(1);
                    continue;
                }

                break;
            }
        } else {
            AtaAhciStartCommandListProcess(ChanData);
        }

        PortCount = 1;
        break;
    }

    if ConnectionStatus == CONN_STATUS_FAILURE {
        AtaAhciStopCommandListProcessAndWait(ChanData);
        AtaAhciPerformCommandListOverride(ChanData);
    } else if PortCount == 0 {
        AtaAhciPhyEnterListenMode(ChanData);
    }

    AtaChanEnableInterruptsSync((ChanData).cast(), TRUE);

    (*ChanData).TotalPortCount = PortCount;
    PortCount
}

unsafe extern "C" fn AtaAhciPmpCheckSendComReset(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut PortNumber: ULONG,
) -> BOOLEAN {
    let mut SataControl: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;
    INFO!(
        b"CH %lu: Transmit a COMRESET on the interface\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
    );
    SrbStatus = AtaAhciPmpWrite(
        ChanData,
        PortNumber as UCHAR,
        (ATA_SERROR) as USHORT,
        0xFFFFFFFF,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }
    SataControl = AHCI_PXCTL_DET_RESET;
    SrbStatus = AtaAhciPmpWrite(
        ChanData,
        PortNumber as UCHAR,
        (ATA_SCONTROL) as USHORT,
        SataControl,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }
    KeStallExecutionProcessor(1000);
    SataControl &= !AHCI_PXCTL_DET_MASK;
    SataControl |= AHCI_PXCTL_DET_IDLE;
    SrbStatus = AtaAhciPmpWrite(
        ChanData,
        PortNumber as UCHAR,
        (ATA_SCONTROL) as USHORT,
        SataControl,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }
    SrbStatus = AtaAhciPmpWrite(
        ChanData,
        PortNumber as UCHAR,
        (ATA_SERROR) as USHORT,
        0xFFFFFFFF,
    );
    if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
        return FALSE;
    }
    return TRUE;
}

unsafe extern "C" fn AtaAhciPmpPhyCheckDevicePresence(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut PortNumber: ULONG,
) -> ATA_CONNECTION_STATUS {
    let mut i: ULONG = 0;
    let mut SataStatus: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;
    i = 0;
    while i < AHCI_DELAY_PMP_DET_PRESENSE {
        SrbStatus = AtaAhciPmpRead(
            ChanData,
            PortNumber as UCHAR,
            (ATA_SSTATUS) as USHORT,
            &raw mut SataStatus,
        );
        if (SrbStatus) as u32 == SRB_STATUS_ERROR {
            i = i.wrapping_add(1);
            continue;
        } else if (SrbStatus) as u32 != SRB_STATUS_SUCCESS {
            return CONN_STATUS_FAILURE;
        }
        if (SataStatus & AHCI_PXSSTS_DET_MASK) != AHCI_PXSSTS_DET_NO_DEVICE {
            INFO!(
                b"CH %lu: PMP device %lu link up %08lx\n\0".as_ptr() as *const i8,
                ChanData.Channel,
                PortNumber,
                SataStatus,
            );
            return CONN_STATUS_DEV_UNKNOWN;
        }
        AtaSleep();
        i = i.wrapping_add(1);
    }
    INFO!(
        b"CH %lu: PMP device %lu not connected %08lx\n\0".as_ptr() as *const i8,
        ChanData.Channel,
        PortNumber,
        SataStatus,
    );
    return CONN_STATUS_NO_DEVICE;
}

unsafe extern "C" fn AtaAhciPmpPhyWaitForReady(
    mut ChanData: PCHANNEL_DATA_AHCI,
    mut PortNumber: ULONG,
) -> ATA_CONNECTION_STATUS {
    let mut i: ULONG = 0;
    let mut SataStatus: ULONG = 0;
    let mut SrbStatus: UCHAR = 0;
    i = 0;
    while i < AHCI_DELAY_PMP_DET_PRESENSE {
        SrbStatus = AtaAhciPmpRead(
            ChanData,
            PortNumber as UCHAR,
            (ATA_SSTATUS) as USHORT,
            &raw mut SataStatus,
        );
        if (SrbStatus as ::core::ffi::c_int) as u32 == SRB_STATUS_ERROR {
            i = i.wrapping_add(1);
            continue;
        } else if (SrbStatus as ::core::ffi::c_int) as u32 != SRB_STATUS_SUCCESS {
            return CONN_STATUS_FAILURE;
        }
        if SataStatus & AHCI_PXSSTS_DET_MASK == AHCI_PXSSTS_DET_PHY_OK {
            INFO!(
                b"CH %lu: PMP device %lu link up %08lx\n\0".as_ptr() as *const i8,
                ChanData.Channel,
                PortNumber,
                SataStatus,
            );
            return CONN_STATUS_DEV_UNKNOWN;
        }
        AtaSleep();
        i = i.wrapping_add(1);
    }
    INFO!(
        b"CH %lu: PMP device %lu unable to establish link %08lx\n\0".as_ptr() as *const i8,
        ChanData.Channel,
        PortNumber,
        SataStatus,
    );
    return CONN_STATUS_NO_DEVICE;
}

unsafe extern "C" fn AtaPmpCheckConnection(
    ChanData: PCHANNEL_DATA_AHCI,
    PortNumber: ULONG,
) -> ATA_CONNECTION_STATUS {
    let mut ConnectionStatus: ATA_CONNECTION_STATUS;

    INFO!(
        b"CH %lu: Reset PMP port %lu\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        PortNumber,
    );

    (*ChanData).__bindgen_anon_1.PortNotification(
        AtaResetDetected,
        (*ChanData).__bindgen_anon_1.PortContext,
        1 << PortNumber,
    );

    if AtaAhciPmpCheckSendComReset(ChanData, PortNumber) == 0 {
        return CONN_STATUS_FAILURE;
    }

    ConnectionStatus = AtaAhciPmpPhyCheckDevicePresence(ChanData, PortNumber);
    if ConnectionStatus != CONN_STATUS_DEV_UNKNOWN {
        return ConnectionStatus;
    }

    ConnectionStatus = AtaAhciPmpPhyWaitForReady(ChanData, PortNumber);
    if ConnectionStatus != CONN_STATUS_DEV_UNKNOWN {
        return ConnectionStatus;
    }

    return CONN_STATUS_DEV_UNKNOWN;
}

unsafe extern "C" fn AtaAhciPmpIdentifyDeviceBehindPmp(
    ChanData: PCHANNEL_DATA_AHCI,
    PortNumber: ULONG,
) -> ATA_CONNECTION_STATUS {
    let mut ConnectionStatus: ATA_CONNECTION_STATUS;
    let mut SrbStatus: UCHAR = 0;

    if (*ChanData).__bindgen_anon_1.ChanInfo as u32 & 0x4000u32 != 0u32 {
        RtlAssert(
            b"!(ChanData->ChanInfo & CHANNEL_FLAG_FBS_ENABLED)\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"drivers/storage/ide/pciidex/chipset/ahci_hw.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            1196u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    ConnectionStatus = AtaPmpCheckConnection(ChanData, PortNumber);
    if ConnectionStatus != CONN_STATUS_DEV_UNKNOWN {
        return ConnectionStatus;
    }

    'failure: {
        for RetryCount in (0 as u32)..(2 as u32) {
            SrbStatus = AtaAhciPmpWrite(
                ChanData,
                PortNumber as UCHAR,
                (ATA_SERROR) as USHORT,
                0xFFFFFFFF,
            );
            if (SrbStatus as i32) as u32 == SRB_STATUS_ERROR {
                continue;
            } else if (SrbStatus as i32) as u32 != SRB_STATUS_SUCCESS {
                break 'failure;
            }

            SrbStatus = AtaAhciSendResetFis(
                ChanData,
                PortNumber as UCHAR,
                TRUE,
                1000,
            );
            if (SrbStatus as i32) as u32 == SRB_STATUS_ERROR {
                continue;
            } else if (SrbStatus as i32) as u32 != SRB_STATUS_SUCCESS {
                break 'failure;
            }

            KeStallExecutionProcessor(20);

            SrbStatus = AtaAhciSendResetFis(
                ChanData,
                PortNumber as UCHAR,
                FALSE,
                1000,
            );
            if (SrbStatus as i32) as u32 == SRB_STATUS_ERROR {
                continue;
            } else if (SrbStatus as i32) as u32 != SRB_STATUS_SUCCESS {
                break 'failure;
            }
        }
        return CONN_STATUS_DEV_UNKNOWN;
    }

    if DbgPrint(
        b"(%s:%d) %s CH %lu: PMP port %lu soft reset failed %02x\n\0".as_ptr() as *const i8,
        b"drivers/storage/ide/pciidex/chipset/ahci_hw.c\0".as_ptr() as *const i8,
        1231i32,
        b"AtaAhciPmpIdentifyDeviceBehindPmp\0".as_ptr() as *const i8,
        (*ChanData).__bindgen_anon_1.Channel,
        PortNumber,
        SrbStatus as i32,
    ) != 0 {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"drivers/storage/ide/pciidex/chipset/ahci_hw.c\0".as_ptr() as *const i8,
            1231i32,
        );
    }

    return CONN_STATUS_FAILURE;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciIdentifyDevice(
    ChannelContext: *mut core::ffi::c_void,
    DeviceNumber: u32,
) -> i32 {
    let ChanData: *mut CHANNEL_DATA_AHCI = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let Signature: u32;

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IS_PMP != 0 {
        let ConnectionStatus: i32;

        ConnectionStatus = (AtaAhciPmpIdentifyDeviceBehindPmp(ChanData, DeviceNumber)) as i32;

        if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_HAS_FBS != 0
            && DeviceNumber == (*ChanData).TotalPortCount.wrapping_sub(1)
        {
            AtaAhciStopCommandListProcessAndWait(ChanData);
            AtaAhciFbsControl(ChanData, TRUE);
            AtaAhciStartCommandListProcess(ChanData);
        }

        if (ConnectionStatus) as u32 != CONN_STATUS_DEV_UNKNOWN {
            return ConnectionStatus;
        }
    } else {
    }

    Signature = AHCI_PORT_READ((*ChanData).IoBase, PxSignature);
    INFO!(
        b"CH %lu: Received signature %08lx\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        Signature,
    );

    if (Signature & AHCI_PXSIG_MASK) == (AHCI_PXSIG_ATAPI & AHCI_PXSIG_MASK) {
        AtaAhciAtapiLedControl(ChanData, TRUE);
        return (CONN_STATUS_DEV_ATAPI) as i32;
    }

    AtaAhciAtapiLedControl(ChanData, FALSE);
    return (CONN_STATUS_DEV_ATA) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciStopDma(ChanData: PCHANNEL_DATA_AHCI) {
    AtaAhciStopCommandListProcessAndWait(ChanData);
    AtaAhciPerformCommandListOverride(ChanData);
    AtaAhciStopFisReceiveProcessAndWait(ChanData);
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciResetChannel(
    ChannelContext: *mut core::ffi::c_void,
) {
    /* The channel reset is done unconditionally in AtaAhciEnumerateChannel() */
    NOTHING;
}

unsafe extern "C" fn AtaAhciSaveReceivedFisArea(
    mut Fis: *mut AHCI_FIS_DEVICE_TO_HOST,
    mut Request: *mut ATA_DEVICE_REQUEST,
) {
    let mut TaskFile: PATA_TASKFILE = &raw mut (*Request).Output;
    (*TaskFile).SectorCount = (*Fis).SectorCount;
    (*TaskFile).LowLba = (*Fis).LbaLow;
    (*TaskFile).MidLba = (*Fis).LbaMid;
    (*TaskFile).HighLba = (*Fis).LbaHigh;
    (*TaskFile).DriveSelect = (*Fis).Device;
    if (*Request).Flags & REQUEST_FLAG_LBA48 != 0 {
        (*TaskFile).FeatureEx = 0;
        (*TaskFile).SectorCountEx = (*Fis).SectorCountEx;
        (*TaskFile).LowLbaEx = (*Fis).LbaLowEx;
        (*TaskFile).MidLbaEx = (*Fis).LbaMidEx;
        (*TaskFile).HighLbaEx = (*Fis).LbaHighEx;
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciSaveTaskFile(
    ChanData: *mut CHANNEL_DATA_AHCI,
    Request: *mut ATA_DEVICE_REQUEST,
    ProcessErrorStatus: BOOLEAN,
) {
    union FisUnion {
        PioSetup: *mut AHCI_FIS_PIO_SETUP,
        DeviceToHost: *mut AHCI_FIS_DEVICE_TO_HOST,
    }
    let mut Fis: FisUnion = FisUnion {
        PioSetup: core::ptr::null_mut(),
    };
    let mut ReceivedFis: *mut AHCI_RECEIVED_FIS;

    ReceivedFis = (*ChanData).ReceivedFis;

    if (*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_FBS_ENABLED != 0 {
        ReceivedFis = ReceivedFis.offset(DEV_NUMBER!((*Request).Device) as isize);
    }

    (*Request).Flags |= REQUEST_FLAG_HAS_TASK_FILE;

    if ProcessErrorStatus != 0 {
        Fis.DeviceToHost = &raw mut (*ReceivedFis).DeviceToHostFis as *mut AHCI_FIS_DEVICE_TO_HOST;
    } else if (*Request).Flags & REQUEST_FLAG_NCQ != 0 {
        let mut SetDeviceBitsFis: *mut AHCI_FIS_SET_DEVICE_BITS;

        RtlCopyMemory!(
            &raw mut (*Request).Output as *mut core::ffi::c_void,
            &raw mut (*Request).__bindgen_anon_2.TaskFile as *const core::ffi::c_void,
            core::mem::size_of::<ATA_TASKFILE>(),
        );

        SetDeviceBitsFis = &raw mut (*ReceivedFis).SetDeviceBitsFis as *mut AHCI_FIS_SET_DEVICE_BITS;
        (*Request).Output.__bindgen_anon_1.Status = (*SetDeviceBitsFis).Status;
        (*Request).Output.__bindgen_anon_2.Error = (*SetDeviceBitsFis).Error;
        return;
    } else if (*Request).Flags & REQUEST_FLAG_DMA == 0
        && (*Request).Flags & REQUEST_FLAG_DATA_IN != 0
    {
        Fis.PioSetup = &raw mut (*ReceivedFis).PioSetupFis as *mut AHCI_FIS_PIO_SETUP;

        (*Request).Output.__bindgen_anon_1.Status = (*Fis.PioSetup).EStatus;
        (*Request).Output.__bindgen_anon_2.Error = (*Fis.PioSetup).Error;
    } else {
        let mut TaskFileData: ULONG;

        Fis.DeviceToHost = &raw mut (*ReceivedFis).DeviceToHostFis as *mut AHCI_FIS_DEVICE_TO_HOST;

        TaskFileData = AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData);
        (*Request).Output.__bindgen_anon_1.Status = (TaskFileData & AHCI_PXTFD_STATUS_MASK) as UCHAR;
        (*Request).Output.__bindgen_anon_2.Error =
            ((TaskFileData & AHCI_PXTFD_ERROR_MASK) >> AHCI_PXTFD_ERROR_SHIFT) as UCHAR;
    }

    AtaAhciSaveReceivedFisArea(Fis.DeviceToHost, Request);
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciHandleFatalError(mut ChanData: PCHANNEL_DATA_AHCI) {
    let mut Request: PATA_DEVICE_REQUEST = core::ptr::null_mut();
    let mut CurrentCommandSlot: u32 = 0;
    let mut TaskFileData: u32 = 0;
    let mut i: u32 = 0;
    let mut CmdStatus: u32 = 0;
    let mut SlotsBitmap: u32 = 0;
    let mut FailedSlot: u32 = 0;

    'done: {
        Request = core::ptr::null_mut();
        CurrentCommandSlot = AHCI_PXCMD_CCS!(AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus));

        AtaAhciStopCommandListProcess(ChanData);
        if AtaAhciIsHbaHotRemoved(ChanData) != 0 {
            break 'done;
        }

        i = 50000;
        while i > 0 {
            CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
            if (CmdStatus & AHCI_PXCMD_CR) == 0 {
                break;
            }
            KeStallExecutionProcessor(10);
            i = i.wrapping_sub(1);
        }
        if i == 0 {
            break 'done;
        }

        TaskFileData = AHCI_PORT_READ((*ChanData).IoBase, PxTaskFileData);
        if (TaskFileData & (IDE_STATUS_BUSY | IDE_STATUS_DRQ)) != 0 {
            break 'done;
        }

        AHCI_PORT_WRITE(
            (*ChanData).IoBase,
            PxSataError,
            AHCI_PORT_READ((*ChanData).IoBase, PxSataError),
        );

        if (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap != 0 {
            AtaAhciStartCommandListProcess(ChanData);
            NT_VERIFY!(
                (_BitScanForward(&mut FailedSlot, (*ChanData).__bindgen_anon_1.ActiveQueuedSlotsBitmap)
                    != 0) as u8,
            );
            Request = *((*ChanData).__bindgen_anon_1.Slots).add(FailedSlot as usize);
            break 'done;
        }

        if IsPowerOfTwo((*ChanData).__bindgen_anon_1.ActiveSlotsBitmap) != 0 {
            SlotsBitmap = (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap;
        } else if ((*ChanData).__bindgen_anon_1.ActiveSlotsBitmap
            & (1 << CurrentCommandSlot))
            != 0
        {
            SlotsBitmap = 1 << CurrentCommandSlot;
        } else {
            if (*ChanData).__bindgen_anon_1.ActiveSlotsBitmap != 0 {
            } else {
            }
            break 'done;
        }

        AtaAhciStartCommandListProcess(ChanData);
        NT_VERIFY!(
            (_BitScanForward(&mut FailedSlot, SlotsBitmap) != 0) as u8,
        );

        Request = *((*ChanData).__bindgen_anon_1.Slots).add(FailedSlot as usize);

        (*Request).SrbStatus = (SRB_STATUS_ERROR) as u8;
        (*Request).Output.__bindgen_anon_1.Status = ((TaskFileData & AHCI_PXTFD_STATUS_MASK)) as u8;
        (*Request).Output.__bindgen_anon_2.Error = ((TaskFileData & AHCI_PXTFD_ERROR_MASK) >> AHCI_PXTFD_ERROR_SHIFT) as u8;

        if (( (*(*Request).Device).TransportFlags & DEVICE_IS_ATAPI) == 0)
            || (((*Request).Flags & REQUEST_FLAG_SAVE_TASK_FILE) != 0)
        {
            AtaAhciSaveTaskFile(ChanData, Request, TRUE);
        }
    }

    ((*(*ChanData).__bindgen_anon_1.PortNotification))(AtaRequestFailed, (*ChanData).__bindgen_anon_1.PortContext, Request);
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciHandlePortStateChange(
    ChanData: PCHANNEL_DATA_AHCI,
    InterruptStatus: u32,
) {
    unsafe {
        let mut SataStatus: u32 = 0;

        'notify: {
            if InterruptStatus & AHCI_PXIRQ_DMPS != 0 {
                let CmdStatus = AHCI_PORT_READ((*ChanData).IoBase, PxCmdStatus);
                if CmdStatus & AHCI_PXCMD_MPSP != 0 {
                    break 'notify;
                }
            }

            SataStatus = AHCI_PORT_READ((*ChanData).IoBase, PxSataStatus) & AHCI_PXSSTS_DET_MASK;
            if SataStatus == AHCI_PXSSTS_DET_PHY_OK {
                INFO!(
                    b"CH %lu: Device hot plug detected %08lx\n\0".as_ptr() as *const i8,
                    (*ChanData).Channel,
                    SataStatus,
                );
            } else {
                INFO!(
                    b"CH %lu: Device removal detected %08lx\n\0".as_ptr() as *const i8,
                    (*ChanData).Channel,
                    SataStatus,
                );
            }
        }

        ((*ChanData).__bindgen_anon_1.PortNotification)
            .expect("non-null function pointer")(
                AtaBusChangeDetected,
                (*ChanData).__bindgen_anon_1.PortContext,
            );
    }
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciDowngradeInterfaceSpeed(
    ChanData: *mut PCHANNEL_DATA_AHCI,
) -> i32 {
    let mut SataSpeed: u32;
    let mut SataControl: u32;

    SataSpeed = AHCI_PORT_READ((*(*ChanData)).IoBase, PxSataStatus) & AHCI_PXSSTS_SPD_MASK;
    if SataSpeed != AHCI_PXSSTS_SPD_SATA1 {
        SataControl = AHCI_PORT_READ((*(*ChanData)).IoBase, PxSataControl);

        if (SataControl & AHCI_PXCTL_SPD_MASK) != AHCI_PXCTL_SPD_LIMIT_NONE {
            SataControl &= !AHCI_PXCTL_SPD_MASK;
            SataControl |= SataSpeed - AHCI_PXCTL_SPD_LIMIT_LEVEL;

            AHCI_PORT_WRITE((*(*ChanData)).IoBase, PxSataControl, SataControl);
            return (TRUE) as _;
        }
    }

    INFO!(
        b"CH %lu: Unable to downgrade interface speed %08lx\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        SataSpeed,
    );
    return (FALSE) as _;
}
#[no_mangle]
pub unsafe extern "C" fn AtaAhciChannelGetMaximumDeviceCount(
    ChannelContext: *mut core::ffi::c_void,
) -> u32 {
    let ChanData: *mut CHANNEL_DATA_AHCI = ChannelContext as *mut CHANNEL_DATA_AHCI;
    let Controller: *mut ATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;

    if (*Controller).AhciCapabilities & AHCI_CAP_SPM != 0 {
        return AHCI_MAX_PMP_DEVICES;
    }

    return AHCI_MAX_PORT_DEVICES;
}
